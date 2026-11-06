// Unit u5-losses: the pages for the earlier-year operating losses (Form 172,
// Schedule 1 line 8a; Form 6251 line 2f) and Form 8615's parent facts,
// reached through the interview. Every expected amount is worked by hand
// from the 2026 draft instructions (i172, i6251, i8615) in its comment.
use super::*;
use crate::tests::cross::interview::{accepted, from_review, rejected_with, save_you};
use crate::tests::federal::losses::{parent, parent_request, save_parent, young_with_interest};

// ---------------------------------------------------------------------------
// Form 8615's parent facts: the parent's return page.

fn parent_page(s: &mut Interview) -> ParentReturn {
    match from_review(s, TaxSection::ParentTax) {
        Reply::Edit(P::SaveParentTax { value, .. }) => value,
        other => panic!("expected the parent's return page, got {other:?}"),
    }
}

#[test]
fn parent_page_matches_the_hand_computed_form_8615() {
    // Child 16, dependent, $10,000 of interest; parent's joint return with
    // taxable income $150,000; no brothers or sisters (2026 draft i8615).
    //  1 10,000   2 2,700   3 7,300
    //  4 taxable 10,000 - 1,350 (dependent standard deduction) = 8,650
    //  5 min(7,300, 8,650) = 7,300
    //  6 150,000  7 0  8 157,300
    //  9 tax on 157,300 (joint, worksheet): 2,480 + 9,120 + 22% x 56,500
    //    (12,430) = 24,030
    // 10 tax on 150,000: 2,480 + 9,120 + 22% x 49,200 (10,824) = 22,424
    // 11 1,606   12c 1.0000   13 8,650 - 7,300 = 1,350
    // 14 Tax Table 1,350-1,375 (single): 136   15 1,742
    // 16 Tax Table 8,650-8,700: 868   17 larger of 15 and 16 = 1,742
    let mut s = Interview::new(TEST_USER, young_with_interest(10_000));
    let shown_first = parent_page(&mut s);
    assert_eq!(shown_first.special, ParentSituation::Nothing);
    let entered = parent(FederalStatus::Joint, 150_000);
    let reply = save_parent(&mut s, entered.clone());
    assert!(matches!(reply, Reply::Response(P::Review { .. })), "{reply:?}");
    let r = supported(&s.draft);
    assert_eq!(value(&r, "kiddie_tax_unearned_income"), 10_000);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 7_300);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 1_606);
    assert_eq!(value(&r, "regular_income_tax"), 1_742);
    // Round trip: saved, restarted and reopened unchanged.
    let mut again = restart(s.draft.clone());
    assert_eq!(again.draft, s.draft);
    assert_eq!(parent_page(&mut again), entered);
}

#[test]
fn form_8615_starts_above_2700_of_unearned_income() {
    // "More than $2,700": $2,700 does not ask the parent's return; $2,701
    // does.
    let at = young_with_interest(2_700);
    assert!(!crate::interview::parent_tax_in_walk(&at));
    assert!(supported(&at).issues.is_empty());
    let above = young_with_interest(2_701);
    assert!(crate::interview::parent_tax_in_walk(&above));
    assert_eq!(estimate(&above).state, ResultState::Unsupported);
}

#[test]
fn form_8615_person_tests_decide_the_parent_page() {
    // A full-time student under 24 who does not provide over half of their
    // support is asked (born January 2, 2003: 23 at the end of 2026); at 24
    // the age test fails; a joint return never is.
    let mut d = young_with_interest(10_000);
    let you = &mut d.people.as_mut().unwrap().taxpayer;
    you.birth_date = 20030102;
    you.full_time_student = true;
    you.support_share = SupportShare::LessThanHalf;
    assert!(crate::interview::parent_tax_in_walk(&d));
    // Born January 1, 2003, a filer is 24 at the end of 2026 (the day
    // before the birthday counts).
    d.people.as_mut().unwrap().taxpayer.birth_date = 20030101;
    assert!(!crate::interview::parent_tax_in_walk(&d));
    let mut joint = young_with_interest(10_000);
    joint.setup.as_mut().unwrap().status = FilingChoice::Joint;
    assert!(!crate::interview::parent_tax_in_walk(&joint));
}

#[test]
fn parent_page_rules_are_checked_on_both_sides() {
    let mut s = Interview::new(TEST_USER, young_with_interest(10_000));
    let before = s.draft.clone();
    let brother =
        |net: i64, gains: Option<ChildGains>| OtherChildTax { label: "Brother".into(), net, gains };
    let mut twice = parent(FederalStatus::Joint, 150_000);
    twice.others = vec![brother(100_000, None), brother(200_000, None)];
    rejected_with(save_parent(&mut s, twice), "PARENT_CHILD_TWICE");
    let mut above = parent(FederalStatus::Joint, 150_000);
    above.others = vec![brother(
        100_000,
        Some(ChildGains {
            investment_election: 0,
            qualified: 60_000,
            capital: 50_000,
            special: None,
        }),
    )];
    rejected_with(save_parent(&mut s, above), "PARENT_CHILD_GAINS_ABOVE_NET");
    let mut source = parent(FederalStatus::Joint, 150_000);
    source.others = vec![brother(
        100_000,
        Some(ChildGains {
            investment_election: 0,
            qualified: 0,
            capital: 50_000,
            special: Some(ChildSpecialGains {
                net_capital: 40_000,
                collectibles: 0,
                unrecaptured: 0,
            }),
        }),
    )];
    rejected_with(save_parent(&mut s, source), "PARENT_CHILD_GAIN_ABOVE_SOURCE");
    assert_eq!(s.draft, before);
    // At the limits both are accepted.
    let mut limits = parent(FederalStatus::Joint, 150_000);
    limits.others = vec![brother(
        100_000,
        Some(ChildGains {
            investment_election: 0,
            qualified: 50_000,
            capital: 50_000,
            special: Some(ChildSpecialGains {
                net_capital: 50_000,
                collectibles: 0,
                unrecaptured: 0,
            }),
        }),
    )];
    accepted(&save_parent(&mut s, limits));
}

#[test]
fn parent_page_refusals_name_their_form_8615_lines() {
    let mut s = Interview::new(TEST_USER, young_with_interest(10_000));
    let mut unsettled = parent(FederalStatus::Joint, 150_000);
    unsettled.special = ParentSituation::Unsettled;
    accepted(&save_parent(&mut s, unsettled));
    let r = estimate(&s.draft);
    let issue =
        r.issues.iter().find(|i| i.message.contains("Form 8615 lines A–C, 6 and 7")).unwrap();
    assert_eq!(
        crate::interview::issue_page(
            &s.draft,
            &issue.context,
            Some({
                let m = &issue.display;
                m.key.as_str()
            })
        ),
        Some(TaxSection::ParentTax)
    );
    let mut worksheets = parent(FederalStatus::Joint, 150_000);
    worksheets.special = ParentSituation::FamilyWorksheets;
    accepted(&save_parent(&mut s, worksheets.clone()));
    let r = estimate(&s.draft);
    assert!(r.issues.iter().any(|i| {
        i.message.contains("Form 8615 lines 9 and 15 with Schedule J on your parent's")
    }));
    assert_eq!(parent_page(&mut s), worksheets);
}

#[test]
fn the_parent_page_is_not_offered_outside_form_8615() {
    let mut s = Interview::new(TEST_USER, young_with_interest(2_000));
    let reply = from_review(&mut s, TaxSection::ParentTax);
    assert!(matches!(&reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let before = s.draft.clone();
    let request = parent_request(&s, parent(FederalStatus::Joint, 150_000));
    let reply = current_source_refusal(
        "secondary-admission-8697",
        &mut s,
        request,
        "PROBLEM_SECTION_UNAVAILABLE_YET",
        None,
        SourceScope::Fieldless,
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })));
    assert_eq!(s.draft, before);
    // A change that ends Form 8615 removes the parent's return.
    let mut s = Interview::new(TEST_USER, young_interest_return());
    let you = crate::interview::filer(&s.draft.people.as_ref().unwrap().taxpayer);
    let mut adult = you.clone();
    adult.birth_date = 19900101;
    let request = save_you(&s.draft.clone(), adult);
    accepted(&send(&mut s, request));
    assert!(s.draft.people.as_ref().unwrap().parent_tax.is_none());
}
