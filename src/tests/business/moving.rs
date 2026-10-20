use crate::tests::support::assert::{accepted, rejected_with};
// Form 3903 moving expenses on More adjustments (unit form-3903, 2026-10-09).
// Fixtures A to E follow the form map (Armed Forces and intelligence-community
// moves only, W-2 box 12 code P as line 4). Every amount is worked by hand with
// the 2026 constants the App uses ($16,100 single standard deduction; 10% to
// $12,400 and 12% to $50,400; Tax Table rows $50 wide taxed at their midpoint;
// moving mileage 20.5 cents through June 30 and 23.5 cents from July 1).
use super::*;
use crate::tests::cross::interview::{document, from_review, save_documents, w2};

fn page(s: &mut Interview) -> P {
    match from_review(s, TaxSection::MoreAdjustments) {
        Reply::Edit(request @ P::SaveMoreAdjustments { .. }) => request,
        other => panic!("expected More adjustments, got {other:?}"),
    }
}
fn move_request(s: &mut Interview, items: Vec<MoreAdjustment>) -> P {
    let mut request = page(s);
    let P::SaveMoreAdjustments { value, .. } = &mut request else { unreachable!() };
    value.adjustments = items;
    request
}
fn save(s: &mut Interview, items: Vec<MoreAdjustment>) -> Reply {
    let request = move_request(s, items);
    send(s, request)
}
fn trip(paid: i64, miles: u32, arrived: u32) -> Option<MoveTrip> {
    Some(MoveTrip { paid, miles, arrived })
}
fn mv(owner: Owner, reason: MoveReason, goods: i64, trip: Option<MoveTrip>) -> MoreAdjustment {
    MoreAdjustment::MovingExpenses(MovingItem {
        owner,
        record: MilitaryMove { reason, goods, trip, reimbursed: 0 },
    })
}
// A W-2 code P total on the owner's wage document.
fn code_p(d: &mut FederalDraft, owner: Owner, amount: i64) {
    let entry = d.income.iter_mut().find(|e| e.owner == owner).unwrap();
    let FederalIncome::Wage { data } = &mut entry.income else { panic!() };
    data.other_box12.push(W2CodeAmount { code: W2Code::P, amount });
}
fn single(p: i64) -> Interview {
    let mut d = complete(FederalStatus::Single);
    code_p(&mut d, Owner::Taxpayer, p);
    Interview::new(TEST_USER, d)
}

#[test]
fn fixture_a_armed_forces_move_deducts_the_expenses_above_code_p() {
    // Single, W-2 box 1 $60,000, box 12 P $2,400. One PCS move: goods
    // $3,150.00; trip paid $640.00 and 1,200 miles, arrived 2026-08-10.
    //   mileage = (1,200 x 47 half-cents + 1) / 2 = 28,200 cents = $282.00 (23.5 cents)
    //   line 2  = 640.00 + 282.00            = $922.00
    //   line 3  = 3,150.00 + 922.00          = $4,072.00
    //   line 4  = $2,400.00 (code P)         line 5 = $1,672.00 -> Schedule 1 line 14
    //   AGI     = 60,000 - 1,672             = $58,328
    //   line 15 = 58,328 - 16,100            = $42,228
    //   line 16: row $42,200-$42,250, midpoint $42,225:
    //     1,240 + 12% x (42,225 - 12,400) = 1,240 + 3,579 = $4,819
    let mut s = single(240_000);
    let before = s.draft.revision;
    accepted(&save(
        &mut s,
        vec![mv(
            Owner::Taxpayer,
            MoveReason::ArmedForcesOrders,
            315_000,
            trip(64_000, 1_200, 20260810),
        )],
    ));
    assert!(s.draft.revision > before);
    let stored = &s.draft.adjustments.as_ref().unwrap().moving;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].record.reimbursed, 240_000);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "moving_expense_deduction"), 1_672);
    assert_eq!(value(&r, "moving_excess_reimbursement"), 0);
    assert_eq!(value(&r, "schedule_1_moving_expenses"), 1_672);
    assert_eq!(return_line(&r, "1h"), 0);
    assert_eq!(return_line(&r, "11a / 11b"), 5_832_800);
    assert_eq!(return_line(&r, "15"), 4_222_800);
    assert_eq!(return_line(&r, "16"), 481_900);
    // The page reopens with the same row (code P shown read-only), and
    // saving it unchanged keeps the revision.
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let P::SaveMoreAdjustments { value, .. } = page(&mut again) else { unreachable!() };
    let MoreAdjustment::MovingExpenses(item) = &value.adjustments[0] else { panic!("{value:?}") };
    assert_eq!(item.record, stored[0].record);
    let request = page(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft.revision, saved.revision);
    assert_eq!(again.draft.adjustments, saved.adjustments);
    // The hub row counts the move.
    let row = crate::interview::more_adjustments_row(&saved).unwrap();
    assert_eq!(status_text(&row.status), "1 move");
}

#[test]
fn fixture_b_reimbursement_above_expenses_is_wages_on_line_1h() {
    // Single, W-2 box 1 $60,000, code P $3,000. Intelligence reassignment:
    // goods $1,500.00; trip paid $400.00, 1,000 miles, arrived 2026-05-15.
    //   mileage = (1,000 x 41 + 1) / 2 = 20,500 cents = $205.00 (20.5 cents)
    //   line 3  = 1,500 + 400 + 205         = $2,105.00; line 4 = $3,000.00
    //   line 5 "No": 3,000 - 2,105          = $895.00 -> Form 1040 line 1h
    //   line 1z = 60,000 + 895              = $60,895 = AGI
    //   line 15 = 60,895 - 16,100           = $44,795
    //   line 16: row $44,750-$44,800, midpoint $44,775:
    //     1,240 + 12% x (44,775 - 12,400) = 1,240 + 3,885 = $5,125
    let mut s = single(300_000);
    accepted(&save(
        &mut s,
        vec![mv(
            Owner::Taxpayer,
            MoveReason::IntelligenceAssignment,
            150_000,
            trip(40_000, 1_000, 20260515),
        )],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "moving_excess_reimbursement"), 895);
    assert_eq!(value(&r, "moving_expense_deduction"), 0);
    assert_eq!(return_line(&r, "1h"), 89_500);
    assert_eq!(return_line(&r, "11a / 11b"), 6_089_500);
    assert_eq!(return_line(&r, "16"), 512_500);
}

#[test]
fn fixture_c_equal_amounts_and_the_mileage_rate_date_boundary() {
    // Single, W-2 box 1 $60,000, code P $2,204.80. PCS move: goods
    // $2,000.00; nothing else paid, 999 miles, arrived 2026-06-30.
    //   mileage = (999 x 41 + 1) / 2 = 40,960 / 2 = 20,480 cents = $204.80
    //     (999 x 20.5 = 20,479.5 rounds half a cent up)
    //   line 3  = 2,000.00 + 204.80 = $2,204.80 = line 4: line 5 = 0, 1h = 0
    //   AGI = $60,000
    let mut s = single(220_480);
    let boundary = |arrived| {
        vec![mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 200_000, trip(0, 999, arrived))]
    };
    accepted(&save(&mut s, boundary(20260630)));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "moving_expense_deduction"), 0);
    assert_eq!(value(&r, "moving_excess_reimbursement"), 0);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    // One day later the 23.5 cent rate applies:
    //   mileage = (999 x 47 + 1) / 2 = 46,954 / 2 = 23,477 cents = $234.77
    //   line 3  = $2,234.77; line 5 = 2,234.77 - 2,204.80 = $29.97 -> $30
    //   AGI = 60,000 - 30 = $59,970
    accepted(&save(&mut s, boundary(20260701)));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "moving_expense_deduction"), 30);
    assert_eq!(return_line(&r, "11a / 11b"), 5_997_000);
}

fn joint_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![
        wage(1, Owner::Taxpayer, 5_000_000, 500_000),
        wage(2, Owner::Spouse, 4_000_000, 400_000),
    ];
    d.next_id = 3;
    code_p(&mut d, Owner::Spouse, 100_000);
    d
}

#[test]
fn fixture_d_joint_spouse_move_and_a_move_left_out() {
    // Joint, taxpayer W-2 $50,000 (no code P), spouse W-2 $40,000 with code P
    // $1,000. Spouse PCS move: goods $1,800.00, no trip (the government flew
    // the family). Taxpayer "another reason" move: goods $5,000.00.
    //   spouse:   line 3 = 1,800.00; line 4 = 1,000.00; line 5 = $800.00
    //   taxpayer: left out with WARN_MOVING_LEFT_OUT
    //   Schedule 1 line 14 = 800; AGI = 50,000 + 40,000 - 800 = $89,200
    let mut s = Interview::new(TEST_USER, joint_return());
    accepted(&save(
        &mut s,
        vec![
            mv(Owner::Spouse, MoveReason::ArmedForcesOrders, 180_000, None),
            mv(Owner::Taxpayer, MoveReason::Other, 500_000, None),
        ],
    ));
    // Both rows are kept as entered; only the estimate leaves the second out.
    assert_eq!(s.draft.adjustments.as_ref().unwrap().moving.len(), 2);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "moving_expense_deduction"), 800);
    assert_eq!(return_line(&r, "11a / 11b"), 8_920_000);
    let notes: Vec<_> =
        r.review_notes.iter().filter(|n| n.display.key == "WARN_MOVING_LEFT_OUT").collect();
    assert_eq!(notes.len(), 1);
    assert_eq!(
        crate::interview::issue_pages(&s.draft, notes[0].context.as_ref().unwrap(), None),
        vec![TaxSection::MoreAdjustments]
    );
    assert_eq!(
        notes[0].context.as_ref().unwrap().identity(),
        IssueContext::owner(TaxSection::MoreAdjustments, Owner::Taxpayer).identity()
    );
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
}

#[test]
fn fixture_d_page_rules_refuse_on_the_server() {
    let spouse_row = mv(Owner::Spouse, MoveReason::ArmedForcesOrders, 180_000, None);
    // (i) A spouse's move on a single return.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    rejected_with(
        {
            let request = move_request(&mut s, vec![spouse_row]);
            current_source_refusal(
                "secondary-admission-8762",
                &mut s,
                request,
                "MESSAGE_MOVING_YOUR_SPOUSES_MOVE_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
                Some("[value/adjustments/0/owner]"),
                SourceScope::Secondary(SecondaryCorrection::AdjustmentOwner),
            )
        },
        "MESSAGE_MOVING_YOUR_SPOUSES_MOVE_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
    );
    // (ii) Nothing paid.
    rejected_with(
        save(&mut s, vec![mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 0, None)]),
        "MOVING_ENTER_AN_EXPENSE",
    );
    // (iii) A trip that ended in 2025 (and the last day before and after 2026).
    for arrived in [20251230, 20251231, 20270101] {
        rejected_with(
            save(
                &mut s,
                vec![mv(
                    Owner::Taxpayer,
                    MoveReason::ArmedForcesOrders,
                    0,
                    trip(10_000, 0, arrived),
                )],
            ),
            "MOVING_ARRIVAL_OUTSIDE_CURRENT_YEAR",
        );
    }
    // A date that is not on the calendar is outside the ranges.
    rejected_with(
        {
            let request = move_request(
                &mut s,
                vec![mv(
                    Owner::Taxpayer,
                    MoveReason::ArmedForcesOrders,
                    0,
                    trip(10_000, 0, 20260231),
                )],
            );
            current_source_refusal(
                "secondary-admission-8809",
                &mut s,
                request,
                "MESSAGE_MOVING_ENTER_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE",
                None,
                SourceScope::Fieldless,
            )
        },
        "MESSAGE_MOVING_ENTER_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE",
    );
    rejected_with(
        {
            let request = move_request(
                &mut s,
                vec![mv(
                    Owner::Taxpayer,
                    MoveReason::ArmedForcesOrders,
                    0,
                    trip(10_000, 20_001, 20260105),
                )],
            );
            current_source_refusal(
                "secondary-admission-8822",
                &mut s,
                request,
                "MESSAGE_MOVING_ENTER_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE",
                None,
                SourceScope::Fieldless,
            )
        },
        "MESSAGE_MOVING_ENTER_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE",
    );
    assert_eq!(s.draft, before);
    // D-2026-10-15-13: entered records have no arbitrary count limit.
    // All nine $10 moves remain present and contribute $90 in total.
    let nine: Vec<_> =
        (0..9).map(|_| mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 1_000, None)).collect();
    let request = move_request(&mut s, nine);
    accepted(&current_source_acceptance("secondary-admission-8827", &mut s, request));
    assert_eq!(s.draft.income, before.income);
    assert_eq!(s.draft.next_id, before.next_id);
    let stored = &s.draft.adjustments.as_ref().unwrap().moving;
    assert_eq!(stored.len(), 9);
    assert!(stored.iter().all(|claim| {
        claim.owner == Owner::Taxpayer
            && claim.record.reason == MoveReason::ArmedForcesOrders
            && claim.record.goods == 1_000
            && claim.record.trip.is_none()
            && claim.record.reimbursed == crate::moving::code_p(&s.draft, claim.owner)
    }));
    assert_eq!(value(&supported(&s.draft), "moving_expense_deduction"), 90);
    // The first and last days of 2026 are accepted.
    for arrived in [20260101, 20261231] {
        accepted(&save(
            &mut s,
            vec![mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 0, trip(10_000, 0, arrived))],
        ));
    }
}

#[test]
fn fixture_d_code_p_without_a_move_is_a_blocking_remaining_item() {
    // (iv) The spouse's W-2 keeps code P while no qualifying spouse move is
    // entered: More adjustments is a remaining item and a derived topic of
    // the adjustments hub, and the estimate adds no income for the code.
    let mut d = joint_return();
    d.finished_sections.retain(|t| *t != TaxSection::MoreAdjustments);
    let mut s = Interview::new(TEST_USER, d);
    let r = estimate(&s.draft);
    let items = crate::interview::remaining_items(&s.draft, &r, TaxSection::Review);
    assert!(
        items.iter().any(|i| i.section == TaxSection::MoreAdjustments
            && status_message(&i.status).key == "REVIEW_MOVING_EXPENSES_NEEDED"),
        "{items:?}"
    );
    assert!(
        crate::interview::derived_topics(&s.draft, TaxSection::Adjustments)
            .contains(&TaxSection::MoreAdjustments)
    );
    // An "another reason" move is not a qualifying move for the code.
    accepted(&save(&mut s, vec![mv(Owner::Spouse, MoveReason::Other, 180_000, None)]));
    assert!(crate::moving::claim_missing(&s.draft));
    // Entering the move compares the code with it and clears the item.
    accepted(&save(&mut s, vec![mv(Owner::Spouse, MoveReason::ArmedForcesOrders, 180_000, None)]));
    assert!(!crate::moving::claim_missing(&s.draft));
    let r = supported(&s.draft);
    let items = crate::interview::remaining_items(&s.draft, &r, TaxSection::Review);
    assert!(
        !items.iter().any(|i| status_message(&i.status).key == "REVIEW_MOVING_EXPENSES_NEEDED")
    );
}

#[test]
fn fixture_d_w2_import_of_code_p_lands_as_the_stored_code_and_rederives_the_move() {
    // (v) A W-2 saved with box 12 code P stores it in other_box12 as P, and
    // the move's read-only reimbursement follows the next W-2 save.
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    accepted(&save(
        &mut s,
        vec![mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 400_000, None)],
    ));
    assert_eq!(s.draft.adjustments.as_ref().unwrap().moving[0].record.reimbursed, 0);
    let mut w = w2(6_000_000);
    w.more.box12 = vec![Box12Entry { code: W2Code::P, amount: 150_000 }];
    let request = save_documents(&s.draft, vec![document(0, IncomeDocument::W2(w))]);
    accepted(&send(&mut s, request));
    let FederalIncome::Wage { data } = &s.draft.income[0].income else { unreachable!() };
    assert_eq!(data.other_box12, vec![W2CodeAmount { code: W2Code::P, amount: 150_000 }]);
    assert_eq!(s.draft.adjustments.as_ref().unwrap().moving[0].record.reimbursed, 150_000);
    // Code P has a calculated effect, so the Wages page raises no refusal for
    // it; line 3 $4,000 less line 4 $1,500 = $2,500.
    let r = supported(&s.draft);
    assert_eq!(value(&r, "moving_expense_deduction"), 2_500);
    // The page lists P once: a second P is the code-twice rule.
    let mut twice = w2(6_000_000);
    twice.more.box12 = vec![
        Box12Entry { code: W2Code::P, amount: 100 },
        Box12Entry { code: W2Code::P, amount: 200 },
    ];
    let before = s.draft.clone();
    rejected_with(
        send(&mut s, save_documents(&before, vec![document(0, IncomeDocument::W2(twice))])),
        "W2_BOX12_CODE_TWICE",
    );
    assert_eq!(s.draft, before);
}

#[test]
fn fixture_e_two_moves_of_one_owner_share_the_owners_code_p() {
    // Single, W-2 $60,000, code P $1,200; two PCS moves, goods $1,000 and
    // $500, no trips.
    //   owner total line 3 = 1,500; line 4 = 1,200; line 5 = $300; 1h = 0
    let mut s = single(120_000);
    accepted(&save(
        &mut s,
        vec![
            mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 100_000, None),
            mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 50_000, None),
        ],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "moving_expense_deduction"), 300);
    assert_eq!(value(&r, "moving_excess_reimbursement"), 0);
    assert_eq!(return_line(&r, "1h"), 0);
    assert_eq!(return_line(&r, "11a / 11b"), 5_970_000);
    // Both rows show the owner's one total, read-only.
    let moves = &s.draft.adjustments.as_ref().unwrap().moving;
    assert!(moves.iter().all(|m| m.record.reimbursed == 120_000));
    // Hub row: a count of the moves, joined with the refusal kinds chosen.
    accepted(&save(
        &mut s,
        vec![
            mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 100_000, None),
            mv(Owner::Taxpayer, MoveReason::ArmedForcesOrders, 50_000, None),
            MoreAdjustment::OtherAdjustments,
        ],
    ));
    assert_eq!(s.draft.adjustment_refusals, vec![MoreAdjustment::OtherAdjustments]);
    let row = crate::interview::more_adjustments_row(&s.draft).unwrap();
    assert!(status_text(&row.status).starts_with("2 moves; "), "{}", status_text(&row.status));
}

#[test]
fn moving_excess_counts_as_earned_income() {
    // The line 1h excess is wages: it is the owner's earned income (here the
    // only earned income, so $895 of earned income for the credit and IRA).
    let mut d = empty_return(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 0, 0)];
    code_p(&mut d, Owner::Taxpayer, 300_000);
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save(
        &mut s,
        vec![mv(
            Owner::Taxpayer,
            MoveReason::IntelligenceAssignment,
            150_000,
            trip(40_000, 1_000, 20260515),
        )],
    ));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "1h"), 89_500);
    assert_eq!(return_line(&r, "11a / 11b"), 89_500);
}

#[test]
fn search_destination_text_names_moving_and_no_longer_a_refusal() {
    let source = include_str!("../../core/protocol.rs");
    let definition = &source[source.find("pub enum TaxSection").unwrap()
        ..source.find("pub enum NavigationDestination").unwrap()];
    let more = &definition[definition.find("    /// More adjustments").unwrap()..];
    let more = &more[..more.find("MoreAdjustments,").unwrap()];
    for phrase in [
        "Moving expenses of Armed Forces members and intelligence community employees (Form 3903, Schedule 1 line 14)",
        "permanent change of station (PCS)",
        "intelligence reassignment",
        "household goods",
        "mileage",
        "storage",
        "W-2 box 12 code P",
    ] {
        assert!(more.contains(phrase), "missing {phrase}");
    }
    assert!(!more.contains("Armed Forces moving expenses (Form 3903)"));
    // Moving is no longer named as not calculated yet.
    let named = &more[more.find("Also named as not calculated yet").unwrap()..];
    assert!(!named.contains("3903"));
}

#[test]
fn moving_rules_and_their_server_twins_agree() {
    let base =
        MilitaryMove { reason: MoveReason::ArmedForcesOrders, goods: 1, trip: None, reimbursed: 0 };
    assert_eq!(base.rule(), None);
    assert!(base.valid());
    let empty = MilitaryMove { goods: 0, ..base.clone() };
    assert_eq!(empty.rule(), Some("MOVING_ENTER_AN_EXPENSE"));
    let outside = MilitaryMove { trip: trip(0, 0, 20251231), ..empty.clone() };
    assert_eq!(outside.rule(), Some("MOVING_ARRIVAL_OUTSIDE_CURRENT_YEAR"));
    assert!(!outside.valid());
    // The mileage rate by arrival date, half a cent rounded up.
    let miles = |arrived, miles| trip(0, miles, arrived).unwrap().mileage_cents();
    assert_eq!(miles(20260630, 1), 21); // 20.5 cents -> 21
    assert_eq!(miles(20260701, 1), 24); // 23.5 cents -> 24
    assert_eq!(miles(20260630, 2), 41);
    assert_eq!(miles(20260701, 2), 47);
    assert_eq!(miles(20260101, 0), 0);
}

// Pub. 536 (NOL, Step 2, line 6) names "moving expenses for members of the
// Armed Forces on active duty" among the business deductions, so the Form 3903
// deduction is not a nonbusiness deduction: it adds to the NOL dollar for
// dollar even where the standard deduction (nonbusiness) is already above the
// return's nonbusiness income. A business loss of $80,000 on a return with no
// nonbusiness income has the same NOL with and without the move; the move
// adds its own $1,800.00 (goods $1,800.00, no code P).
#[test]
fn moving_deduction_is_a_business_deduction_in_the_nol() {
    let mut without = complete(FederalStatus::Single);
    add(&mut without, income(cash_business(-80000)));
    let before = value(&supported(&without), "current_net_operating_loss");
    let mut with = without.clone();
    with.adjustments.as_mut().unwrap().moving.push(MovingClaim {
        owner: Owner::Taxpayer,
        record: MilitaryMove {
            reason: MoveReason::ArmedForcesOrders,
            goods: 180_000,
            trip: None,
            reimbursed: 0,
        },
    });
    let r = supported(&with);
    assert_eq!(value(&r, "moving_expense_deduction"), 1_800);
    assert_eq!(value(&r, "current_net_operating_loss"), before + 1_800);
}
