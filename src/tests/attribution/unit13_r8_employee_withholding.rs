use super::*;
use crate::tests::cross::interview::{accepted, open_topic, rejected_with};
use crate::tests::support::assert::injured_credit::r8_employee_withholding::{
    estimate_if_known, money,
};
use crate::tests::support::builders::injured_credit::r8_employee_withholding::fixture;
use crate::tests::support::client::injured_credit::r8_employee_withholding::{
    employee_id, public_roundtrip,
};

#[test]
fn r8_employee_nevada_nonfederal_sources_preserve_money_then_overlay_and_correction() {
    let mut observed = vec![];
    let mut expected = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = public_roundtrip(fixture(owner, injured, reversed, false), owner);
                let id = employee_id(&d);
                let r = money(&d, owner);
                // All1000 source Dollars are community:426 refund has a213 protected half.
                observed.push((owner, injured, reversed, false, estimate_if_known(&r)));
                expected.push((owner, injured, reversed, false, Some(21300)));
                let mut s = restart(d);
                let mut p = open_topic(&mut s, TaxSection::Wages);
                let P::SaveWagesAnswers { followups, .. } = &mut p else { panic!() };
                let row = followups
                    .iter_mut()
                    .find(|row| {
                        row.key.starts_with(&alloc::format!("income/{id}/"))
                            && matches!(row.answer, crate::WagesDocumentAnswer::Community { .. })
                    })
                    .expect("employee property is answered on its topic");
                row.answer =
                    crate::WagesDocumentAnswer::Community { value: CommunityShare::Separate };
                accepted(&send(&mut s, p));
                let d = public_roundtrip(s.draft, owner);
                let r = money(&d, owner);
                // Own separate250 +half remainingcommunity750 gives625/1000;
                // other spouse's separate250 leaves only375/1000 protected.
                observed.push((owner, injured, reversed, true, estimate_if_known(&r)));
                expected.push((
                    owner,
                    injured,
                    reversed,
                    true,
                    Some(if owner == injured { 26625 } else { 15975 }),
                ));
            }
        }
    }
    assert_eq!(
        observed, expected,
        "all16 independent ordinary/publicSave/TaxAppData/restart profiles passed before R8 source assertions"
    );
}

#[test]
fn r8_employee_texas_existing_management_source_is_needed_and_native_save_checked() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let d = fixture(owner, Owner::Taxpayer, false, true);
        money(&d, owner);
        let id = employee_id(&d);
        let mut s = restart(d);
        let mut p = crate::tests::support::client::income_pages::open_topic(
            &mut s,
            TaxSection::InjuredSpouse,
        );
        let P::SaveInjuredSpouse { followups, .. } = &mut p else { panic!() };
        let row = followups
            .iter_mut()
            .find(|row| {
                matches!(row.answer, crate::InjuredSpouseDocumentAnswer::Management { .. })
                    && row.key.starts_with(&alloc::format!("income/{id}/"))
            })
            .expect("existing employee source needs management answer");
        row.answer = crate::InjuredSpouseDocumentAnswer::Management { value: None };
        let before = s.draft.clone();
        let dirty = s.dirty;
        rejected_with(send(&mut s, p), "INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED");
        assert_eq!(s.draft, before);
        assert_eq!(s.dirty, dirty);
    }
}

#[test]
fn r8_employee_texas_actual_management_and_owner_sources_survive_restart() {
    let mut observed = vec![];
    let mut expected = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = public_roundtrip(fixture(owner, injured, reversed, true), owner);
                let id = employee_id(&d);
                let r = money(&d, owner);
                observed.push((owner, injured, reversed, false, estimate_if_known(&r)));
                expected.push((
                    owner,
                    injured,
                    reversed,
                    false,
                    Some(if owner == injured { 5325 } else { 15975 }),
                ));
                let mut s = restart(d);
                let mut p = open_topic(&mut s, TaxSection::InjuredSpouse);
                let P::SaveInjuredSpouse { followups, .. } = &mut p else { panic!() };
                let row = followups
                    .iter_mut()
                    .find(|row| {
                        row.key.starts_with(&alloc::format!("income/{id}/"))
                            && matches!(
                                row.answer,
                                crate::InjuredSpouseDocumentAnswer::Management { .. }
                            )
                    })
                    .expect("employee management is answered on its topic");
                row.answer = crate::InjuredSpouseDocumentAnswer::Management {
                    value: Some(RefundManagement::Joint),
                };
                accepted(&send(&mut s, p));
                let d = public_roundtrip(s.draft, owner);
                let r = money(&d, owner);
                observed.push((owner, injured, reversed, true, estimate_if_known(&r)));
                expected.push((
                    owner,
                    injured,
                    reversed,
                    true,
                    Some(if owner == injured { 0 } else { 15975 }),
                ));
            }
        }
    }
    assert_eq!(
        observed, expected,
        "all16 independent ordinary/publicSave/TaxAppData/restart profiles passed before Texas management assertions"
    );
}

// Who managed the withholding funds is a required question with no default:
// asked first on its own page, only for a Texas employee source, never as an
// option meaning "not answered yet".
#[test]
fn r8_employee_texas_management_is_a_required_question_asked_only_when_it_applies() {
    use crate::tests::cross::interview::from_review;
    use crate::tests::support::client::own_questions::{
        answer_questions, save_questions, wire_page,
    };
    // Nevada: no management question, the page opens.
    let d =
        public_roundtrip(fixture(Owner::Taxpayer, Owner::Taxpayer, false, false), Owner::Taxpayer);
    let mut s = restart(d);
    let reply = from_review(&mut s, TaxSection::InjuredSpouse);
    let Reply::Response(P::BeginInjuredSpouse { questions, followups, .. }) =
        wire_page(&mut s, reply)
    else {
        panic!("without a management question the injured-spouse page opens")
    };
    assert!(questions.is_empty() && followups.is_empty());

    let d =
        public_roundtrip(fixture(Owner::Taxpayer, Owner::Taxpayer, false, true), Owner::Taxpayer);
    let id = employee_id(&d);
    let mut s = restart(d);
    // The fixture already answered it; a source without the answer is asked.
    s.draft.income.iter_mut().find(|entry| entry.id == id).unwrap().refund_management = None;
    let reply = from_review(&mut s, TaxSection::InjuredSpouse);
    let begin = match wire_page(&mut s, reply) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("expected the questions page, got {other:?}"),
    };
    let P::BeginDocumentQuestions { topic, questions, .. } = &begin else { unreachable!() };
    assert_eq!(*topic, TaxSection::InjuredSpouse);
    assert!(questions.iter().all(|row| row.key.starts_with(&alloc::format!("income/{id}/"))
        && row.key.ends_with("management")));
    assert_eq!(questions.len(), 1);
    // The question blocks Save: nothing is stored without an answer.
    let before = s.draft.clone();
    assert!(matches!(
        save_questions(&mut s, &begin, Vec::new()),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(s.draft, before);

    let answer = crate::DocumentStrictAnswer::Management { value: RefundManagement::Joint };
    let reply = answer_questions(&mut s, &begin, vec![answer.clone()]);
    let Reply::Response(P::BeginInjuredSpouse { questions, followups, .. }) = reply else {
        panic!("expected the injured-spouse page, got {reply:?}")
    };
    assert!(followups.is_empty());
    // The page repeats every answered management question, the new one included.
    assert!(questions.iter().any(|row| crate::OwnAsked::parts(row.clone()).1 == answer));
    let entry = s.draft.income.iter().find(|entry| entry.id == id).unwrap();
    assert_eq!(entry.refund_management, Some(RefundManagement::Joint));
}
