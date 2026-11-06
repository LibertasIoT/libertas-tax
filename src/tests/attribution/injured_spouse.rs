use crate::tests::support::client::income_pages::{open_documents, qsbs_bool_fact};
use crate::tests::support::client::{draft_from_native_records, native_draft_records};
// Form 8379 (T3-8379): Part III allocation and the estimate of the injured
// spouse's share, every expected amount worked by hand from the Form 8379
// final instructions (November2024), December2026 draft instructions and
// IRM21.4.6.5.6.1-3 (tax/EIC October1,2024; refund May7,2026).
use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
// Existing topic paths retain a facade over the one canonical builder.
pub(in crate::tests) use crate::tests::support::builders::{allocation, injured_return};

#[path = "unit13_current_nua_sources.rs"]
mod unit13_current_nua_sources;
#[path = "unit13_document_owner_client_lifecycle.rs"]
mod unit13_document_owner_client_lifecycle;
#[path = "unit13_ebl_sources.rs"]
mod unit13_ebl_sources;
#[path = "unit13_financial_income_context.rs"]
mod unit13_financial_income_context;
#[path = "unit13_hobby_niit_sources.rs"]
mod unit13_hobby_niit_sources;
#[path = "unit13_niit_mixed_sources.rs"]
mod unit13_niit_mixed_sources;
#[path = "unit13_niit_partial_survivor.rs"]
mod unit13_niit_partial_survivor;
#[path = "unit13_resolved_income_categories.rs"]
mod unit13_resolved_income_categories;
#[path = "unit13_rest_rounding.rs"]
mod unit13_rest_rounding;
#[path = "unit13_settled_adoption_sources.rs"]
mod unit13_settled_adoption_sources;
#[path = "unit13_settled_care_sources.rs"]
mod unit13_settled_care_sources;
#[path = "unit13_shared_amount_client_lifecycle.rs"]
mod unit13_shared_amount_client_lifecycle;
#[path = "unit13_shared_amount_construction.rs"]
mod unit13_shared_amount_construction;
#[path = "unit13_statutory_w2_sources.rs"]
mod unit13_statutory_w2_sources;
#[path = "unit13_unanswered_shared_rows.rs"]
mod unit13_unanswered_shared_rows;

#[path = "unit13_earlier_property_sources.rs"]
mod unit13_earlier_property_sources;
#[path = "unit13_farm_rental_niit_sources.rs"]
mod unit13_farm_rental_niit_sources;
#[path = "unit13_ltc_niit_sources.rs"]
mod unit13_ltc_niit_sources;
#[path = "unit13_native_withholding_sources.rs"]
mod unit13_native_withholding_sources;
#[path = "unit13_niit_noninvestment_sources.rs"]
mod unit13_niit_noninvestment_sources;

fn share_cents(r: &FederalResult) -> i64 {
    r.lines
        .iter()
        .find(|l| l.id == "form_8379_injured_refund_estimate")
        .unwrap_or_else(|| panic!("no estimate: {r:?}"))
        .amount
}

fn unavailable(d: &FederalDraft, key: &str) {
    let r = supported(d);
    assert!(r.review_notes.iter().any(|n| n.display.key == key), "{key}: {:?}", r.review_notes);
    assert!(r.lines.iter().all(|l| l.id != "form_8379_injured_refund_estimate"));
}

// The fixture. You (the injured spouse): W-2 $60,000, $5,000 withheld. Your
// spouse, whose child support is past due: W-2 $20,000, $500 withheld, and a
// 1099-INT of $1,000. Joint estimated payments $1,000, split by the formula.
//
// Joint return: total income 60,000 + 20,000 + 1,000 = 81,000 = AGI;
// standard deduction 32,200; taxable 48,800. 2026 Tax Table (MFJ), row
// 48,800-48,850, midpoint 48,825: 2,480 + 12% x (48,825 - 24,800) = 5,363.
// Payments 5,500 + 1,000 = 6,500; refund 1,137.
//
// Part III (b | c): 13a 60,000 | 20,000; 13b 0 | 1,000; 14 0 | 0; 15 half
// the basic standard deduction each, 16,100 | 16,100; 16-18 0; 19 5,000 |
// 500.
// Separate (MFS) taxable income: you 60,000 - 16,100 = 43,900; spouse
// 21,000 - 16,100 = 4,900. Tax Table (single/MFS): 43,900-43,950, midpoint
// 43,925: 1,240 + 12% x 31,525 = 5,023; 4,900-4,950, midpoint 4,925: 10% =
// 492.50, 493. Injured percentage 5,023 / 5,516 = 0.9106 (four decimals).
// Line 20 by the formula: 1,000 x 5,023 / 5,516 = 910.62, so 911 | 89.
// Injured share of the joint tax: 5,363 x 0.9106 = 4,883.5478, $4,883.55.
// Contribution: 5,000 + 911 = 5,911.00. Share of the refund: 5,911.00 -
// 4,883.55 = $1,027.45, below the $1,137 refund.
#[test]
fn injured_spouse_fixture_matches_hand_computation() {
    let d = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
    let mut spouse_interest = income(savings_income(1_000, 0));
    spouse_interest.owner = crate::DocumentOwner::Spouses;
    let mut d = d;
    add(&mut d, spouse_interest);
    let r = supported(&d);
    assert_eq!(return_line(&r, "16") / 100, 5_363);
    assert_eq!(r.refund, Some(113_700));
    for (line, a, b, c) in [
        ("13a", 80_000, 60_000, 20_000),
        ("13b", 1_000, 0, 1_000),
        ("14", 0, 0, 0),
        ("15", 32_200, 16_100, 16_100),
        ("16", 0, 0, 0),
        ("17", 0, 0, 0),
        ("18", 0, 0, 0),
        ("19", 5_500, 5_000, 500),
        ("20", 1_000, 911, 89),
    ] {
        assert_eq!(
            (column(&r, line, "a"), column(&r, line, "b"), column(&r, line, "c")),
            (a, b, c),
            "line {line}"
        );
    }
    assert_eq!(
        r.lines.iter().find(|l| l.id == "form_8379_injured_share_of_tax").unwrap().amount,
        488_355
    );
    let explanation = &r
        .lines
        .iter()
        .find(|line| line.id == "form_8379_injured_share_of_tax")
        .unwrap()
        .explanation;
    assert!(
        matches!(&explanation.args[..], [MessageArg::Decimal { units: 9106, scale: 2 }, MessageArg::Currency { currency_code: a, scale: 0, .. }, MessageArg::Currency { currency_code: b, scale: 0, .. }] if a == "USD" && b == "USD")
    );
    let encoded = explanation.encode();
    let mut offset = 0;
    let decoded = <crate::common::FormattedMessage as libertas::AvroDecode>::avro_decode(
        &encoded,
        &mut offset,
    )
    .unwrap();
    assert_eq!(offset, encoded.len());
    assert_eq!(decoded.0, "EXPLAIN_FORM_N_INJURED_SHARE_OF_TAX");
    assert!(
        matches!(decoded.1[0], libertas::LibertasMessageArgumentDecode::Double(value) if (value - 91.06).abs() < 0.000001)
    );
    assert_eq!(share_cents(&r), 102_745);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("about $1027") && n.contains("The IRS makes the final computation")),
        "{:?}",
        r.review_notes
    );
    let message = r
        .review_notes
        .iter()
        .map(|note| &note.display)
        .find(|message| message.key == "NOTE_INJURED_SPOUSE_IRS_ESTIMATE")
        .unwrap();
    assert_eq!(message.args, vec![MessageArg::usd_cents(102_745)]);
    // Form 8379 never changes the joint return.
    let mut plain = d.clone();
    plain.often_missed.unprepared.clear();
    plain.injured_spouse = None;
    assert_eq!(supported(&plain).refund, r.refund);
}

// Boundary: the share is limited to the joint refund. You: $60,000, $12,000
// withheld; your spouse: $40,000, nothing withheld. Joint taxable 100,000 -
// 32,200 = 67,800; Tax Table 67,800-67,850: 2,480 + 12% x 43,025 = 7,643;
// refund 12,000 - 7,643 = 4,357. MFS: you 43,900 -> 5,023; spouse 23,900,
// midpoint 23,925: 1,240 + 12% x 11,525 = 2,623; 5,023 / 7,646 = 0.6569;
// share of tax 7,643 x 0.6569 = 5,020.69; 12,000 - 5,020.69 = 6,979.31,
// above the refund: the estimate is the whole $4,357.
#[test]
fn injured_spouse_share_is_limited_to_the_joint_refund() {
    let d = injured_return((60_000, 12_000), (40_000, 0), 0, allocation());
    let r = supported(&d);
    assert_eq!(r.refund, Some(435_700));
    assert_eq!(
        r.lines.iter().find(|l| l.id == "form_8379_injured_share_of_tax").unwrap().amount,
        502_069
    );
    assert_eq!(share_cents(&r), 435_700);
}

// Boundary: the share is never below zero. You: $60,000, $1,000 withheld;
// your spouse: $20,000, $10,000 withheld. Joint taxable 80,000 - 32,200 =
// 47,800, row 47,800-47,850, midpoint 47,825: 2,480 + 12% x 23,025 =
// 5,243; refund 11,000 - 5,243 = 5,757. MFS: you
// 5,023; spouse 20,000 - 16,100 = 3,900, midpoint 3,925: 392.50, 393;
// 5,023 / 5,416 = 0.9274; share of tax 5,243 x 0.9274 = 4,862.36, above
// your $1,000 withheld: the estimate is $0.
#[test]
fn injured_spouse_share_is_never_below_zero() {
    let d = injured_return((60_000, 1_000), (20_000, 10_000), 0, allocation());
    let r = supported(&d);
    assert_eq!(return_line(&r, "16") / 100, 5_243);
    assert_eq!(r.refund, Some(575_700));
    assert_eq!(
        r.lines.iter().find(|l| l.id == "form_8379_injured_share_of_tax").unwrap().amount,
        486_236
    );
    assert_eq!(share_cents(&r), 0);
}

// Boundary: an agreed split of the joint payments. The fixture's return with
// the $1,000 all agreed to be yours: line 20 is 1,000 | 0, contribution
// 6,000.00, share 6,000.00 - 4,883.55 = 1,116.45.
#[test]
fn injured_spouse_agreed_estimated_payments_follow_the_agreement() {
    let mut a = allocation();
    a.estimated = EstimatedSplit::Agreed { injured: 100_000 };
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 1_000, a);
    let mut spouse_interest = income(savings_income(1_000, 0));
    spouse_interest.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse_interest);
    let r = supported(&d);
    assert_eq!((column(&r, "20", "b"), column(&r, "20", "c")), (1_000, 0));
    assert_eq!(share_cents(&r), 111_645);
}

// Community property (line 5 Yes, child support, no separate items): half of
// the refund other than the earned income credit. The fixture's refund
// $1,137 (no EIC at this income) gives $568.50.
#[test]
fn injured_spouse_community_property_takes_half_the_refund() {
    let mut a = allocation();
    a.community =
        InjuredCommunityFacts::Yes { states: vec![5], debt_facts: InjuredDebtFacts::NotApplicable };
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 1_000, a);
    let mut spouse_interest = income(savings_income(1_000, 0));
    spouse_interest.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse_interest);
    for entry in &mut d.income {
        entry.community = CommunityShare::Community;
    }
    for payment in crate::tests::support::builders::dated_payments_mut(d.payments.as_mut().unwrap())
    {
        payment.refund_property = Some(RefundSourcePropertyFacts::Community {
            management: RefundManagement::NotApplicable,
        });
    }
    let r = supported(&d);
    assert_eq!(share_cents(&r), 56_850);
    let message = r
        .review_notes
        .iter()
        .map(|note| &note.display)
        .find(|message| message.key == "NOTE_INJURED_SPOUSE_COMMUNITY_ESTIMATE")
        .unwrap();
    assert_eq!(message.args, vec![MessageArg::usd_cents(56_850)]);
}

#[test]
fn injured_spouse_itemized_allocation_requires_a_direct_choice() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    itemized(d.deductions.as_mut().unwrap()).medical = 6_000_000;
    let r = supported(&d);
    let row = r
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|r| r.component == "INJURED_COMPONENT_ITEMIZED")
        .unwrap();
    assert_eq!(row.label, Message::new("INJURED_COMPONENT_ITEMIZED", vec![]).encode());
    assert_eq!((row.minimum, row.maximum), (0, row.joint));
    let joint = row.joint;
    d.injured_spouse
        .as_mut()
        .unwrap()
        .allocations
        .push(InjuredAmount { component: "INJURED_COMPONENT_ITEMIZED".into(), injured: joint / 2 });
    let r = supported(&d);
    assert_eq!(column(&r, "15", "b"), crate::kernel::ratio::dollars(joint / 2));
}

// The age and blindness additions go to their own spouse (the instructions'
// worksheet): your spouse is 65 or older, so the 2026 standard deduction is
// 32,200 + 1,650 = 33,850, split 16,100 | 17,750. The senior deduction
// (Schedule 1-A, $6,000 before the phaseout at this income) is your spouse's.
#[test]
fn injured_spouse_standard_deduction_additions_follow_the_person() {
    let d = crate::tests::support::builders::injured_original::fixture("i-standard-taxpayer");
    let r = supported(&d);
    let senior = value(&r, "senior_deduction");
    assert_eq!(senior, 6_000);
    assert_eq!(
        (column(&r, "15", "a"), column(&r, "15", "b"), column(&r, "15", "c")),
        (33_850, 16_100, 17_750)
    );
}

#[test]
fn injured_spouse_unavailable_benefit_keeps_the_ordinary_estimate() {
    let base = || injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let mut d = base();
    d.injured_spouse.as_mut().unwrap().debt = OffsetDebt::NoneOfThese;
    unavailable(&d, "ISSUE_8379_SPOUSE_ONLY_DEBT_REQUIRED");
    let mut d = base();
    d.injured_spouse.as_mut().unwrap().obligated = true;
    unavailable(&d, "ISSUE_8379_INJURED_SPOUSE_OWES_DEBT");
    let mut d = base();
    d.injured_spouse.as_mut().unwrap().community = InjuredCommunityFacts::Yes {
        states: vec![1, 5],
        debt_facts: InjuredDebtFacts::NotApplicable,
    };
    unavailable(&d, "WARN_8379_COMMUNITY_ESTIMATE_UNAVAILABLE");
    // 2026-10-19 acceptance defect 8: the notice links the Injured spouse page.
    let note = supported(&d)
        .review_notes
        .into_iter()
        .find(|n| n.display.key == "WARN_8379_COMMUNITY_ESTIMATE_UNAVAILABLE")
        .unwrap();
    assert_eq!(note.context, Some(IssueContext::Section(TaxSection::InjuredSpouse)));
    let mut d = injured_return((60_000, 5_000), (0, 0), 0, allocation());
    d.income.truncate(1);
    d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
    unavailable(&d, "ISSUE_8379_NO_ELIGIBLE_REFUND_SHARE");
}

// The earned income credit by each spouse's separate credit (IRM
// 21.4.6.5.6.2), with the joint return's one child and married-filing-
// jointly column. You: W-2 $15,000, $300 withheld; your spouse: W-2
// $10,000. Joint AGI 25,000, below the $31,160 phaseout: EIC $4,427 (one
// child, the maximum); taxable income 0, so no tax and the $2,200 child tax
// credit is refundable up to $1,700 (15% x 22,500 = 3,375 is more): ACTC
// $1,700. Refund 300 + 4,427 + 1,700 = 6,427.
// Separate EIC: you 15,000 -> 4,427; your spouse 10,000, row 10,000-10,050,
// midpoint 10,025 x 34% = 3,408.50, 3,409. Your percentage 4,427 / 7,836 =
// 0.5650; your EIC 4,427 x 0.5650 = 2,501.255, $2,501.26. The ACTC is
// assigned to the taxpayer who claims the child (1,700 | 0). Both separate
// taxes are0, so the joint-tax share is0 and the injured refund estimate is
// 300 + 1,700 + 2,501.26 = $4,501.26.
#[test]
fn injured_spouse_earned_income_credit_follows_separate_credits() {
    let mut d = injured_return((15_000, 300), (10_000, 0), 0, allocation());
    add_child(&mut d);
    d.injured_spouse.as_mut().unwrap().claimants =
        vec![InjuredClaimantFacts { person: d.dependents[0].id, claimant: Owner::Taxpayer }];
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a") / 100, 4_427);
    assert_eq!(return_line(&r, "28") / 100, 1_700);
    assert_eq!(r.refund, Some(642_700));
    assert_eq!(
        (column(&r, "17", "a"), column(&r, "17", "b"), column(&r, "17", "c")),
        (1_700, 1_700, 0)
    );
    assert_eq!(r.lines.iter().find(|l| l.id == "form_8379_injured_eic").unwrap().amount, 250_126);
    assert_eq!(share_cents(&r), 450_126);
}

// Line 14 allocates each adjustment to the spouse who would have claimed it
// on a separate return (Instructions for Form 8379, line 14). You: educator
// expenses $200.00. Your spouse: educator expenses $100.00, a reservist's
// travel of $900.00 (Form 2106 line 10, Schedule 1 line 12) and an Armed
// Forces move with goods of $1,800.00 and no code P (Schedule 1 line 14).
// Joint line 10 = 300 + 900 + 1,800 = 3,000; you 200, your spouse
// 100 + 900 + 1,800 = 2,800. Nothing falls into the halved "other" bucket.
#[test]
fn injured_spouse_line_14_follows_moving_educator_and_employee_expenses() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
    let a = d.adjustments.as_mut().unwrap();
    a.set_educator_taxpayer(20_000);
    a.educators.spouse = Some(EducatorRole::Educator { classroom: 10_000, instructional: 0 });
    a.moving.push(MovingClaim {
        owner: Owner::Spouse,
        record: MilitaryMove {
            reason: MoveReason::ArmedForcesOrders,
            goods: 180_000,
            trip: None,
            reimbursed: 0,
        },
    });
    a.employee_expenses.push(EmployeeExpenseClaim {
        improvement_buildings: Vec::new(),
        equipment_assets: vec![],
        dot_hours_of_service: false,
        leased_vehicles: vec![],
        owner: Owner::Spouse,
        occupation: "Army Reserve sergeant".into(),
        capacity: EmployeeCapacity::Reservist,
        employers: vec![2],
        local_transportation: 0,
        travel: 90_000,
        other: 0,
        meals: 0,
        reimbursements: 0,
        mileage: vec![],
        vehicle_assets: vec![],
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "10") / 100, 3_000);
    assert_eq!(
        (column(&r, "14", "a"), column(&r, "14", "b"), column(&r, "14", "c")),
        (3_000, 200, 2_800)
    );
    assert!(
        r.review_notes.iter().all(|n| n.display.key != "NOTE_INJURED_HALF_OTHER_ADJUSTMENTS"
            && n.display.key != "NOTE_INJURED_HALF_ADJUSTMENT"
            && n.display.key != "NOTE_INJURED_HALF_EDUCATOR"),
        "{:?}",
        r.review_notes
    );
}

// Rev. Rul. 2004-71 through -74, 2004-30 IRB, published community refund
// arithmetic adapted to synthetic2026 wages producing a $1,000 overpayment.
// The original rulings' historical law/debt assumptions are not shifted.
use crate::tests::support::builders::injured_original::community_ruling;
#[test]
fn injured_spouse_published_community_ruling_arithmetic() {
    for (state, debt_facts, own, liable, expected) in [
        (1, InjuredDebtFacts::BeforeMarriage, 250, 750, 12_500),
        (9, InjuredDebtFacts::DuringMarriageNotFamily, 250, 750, 50_000),
        (9, InjuredDebtFacts::DuringMarriageFamily, 250, 750, 0),
        (2, InjuredDebtFacts::BeforeMarriage, 250, 750, 0),
        (3, InjuredDebtFacts::BeforeMarriage, 250, 750, 0),
        (4, InjuredDebtFacts::BeforeMarriage, 250, 750, 0),
        (5, InjuredDebtFacts::BeforeMarriage, 250, 750, 50_000),
        (6, InjuredDebtFacts::BeforeMarriage, 250, 750, 50_000),
        (8, InjuredDebtFacts::BeforeMarriage, 250, 750, 50_000),
        (7, InjuredDebtFacts::BeforeMarriage, 250, 750, 12_500),
        (7, InjuredDebtFacts::BeforeMarriage, 750, 250, 37_500),
        (7, InjuredDebtFacts::BeforeMarriage, 1_000, 0, 50_000),
    ] {
        let r = supported(&community_ruling(state, debt_facts, own, liable));
        assert_eq!(r.refund, Some(100_000), "state{state}");
        assert_eq!(share_cents(&r), expected, "state{state}: {debt_facts:?}");
        assert_eq!((column(&r, "13a", "b"), column(&r, "13a", "c")), (7_500, 22_500));
    }
}
#[test]
fn injured_spouse_nonfederal_separate_property_refund_is_not_halved() {
    let mut d = community_ruling(5, InjuredDebtFacts::NotApplicable, 200, 800);
    d.injured_spouse.as_mut().unwrap().debt = OffsetDebt::ChildSupport;
    d.income[0].community = CommunityShare::Separate;
    let r = supported(&d);
    // Injured separate source200 plus half community800 gives600.
    assert_eq!(share_cents(&r), 60_000);
}
#[test]
fn injured_spouse_joint_zero_tax_ratio_needs_agreement_but_separate_payment_does_not() {
    let mut d = injured_return((20_000, 0), (10_000, 0), 1_000, allocation());
    // Separate taxes: taxpayer on3,900 is393, so first lower wages to15k each.
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        edit_wage(data).wages = 1_500_000;
    }
    if let FederalIncome::Wage { data } = &mut d.income[1].income {
        edit_wage(data).wages = 1_500_000;
    }
    unavailable(&d, "WARN_8379_JOINT_PAYMENT_SPLIT_NEEDED");
    d.injured_spouse.as_mut().unwrap().estimated = EstimatedSplit::Agreed { injured: 30_000 };
    assert_eq!(column(&supported(&d), "20", "b"), 300);
    crate::tests::support::builders::dated_payments_mut(d.payments.as_mut().unwrap())[0].payer =
        PaymentPayer::Spouse;
    d.injured_spouse.as_mut().unwrap().estimated = EstimatedSplit::IrsFormula;
    let r = supported(&d);
    assert_eq!((column(&r, "20", "b"), column(&r, "20", "c")), (0, 1_000));
}
#[test]
fn injured_spouse_loan_exclusive_borrower_owns_actual_joint_deduction() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    d.adjustments.as_mut().unwrap().set_student_loan_interest(150_000);
    d.injured_spouse.as_mut().unwrap().student_loan_borrower = Some(InjuredLoanBorrower::Spouse);
    let r = supported(&d);
    assert_eq!(return_line(&r, "10"), 150_000);
    assert_eq!(
        (column(&r, "14", "a"), column(&r, "14", "b"), column(&r, "14", "c")),
        (1_500, 0, 1_500)
    );
    assert!(
        r.injured_spouse
            .unwrap()
            .required
            .iter()
            .all(|r| r.component != "schedule_1_student_loan_interest")
    );
}
#[test]
fn injured_spouse_withholding_categories_round_independently() {
    let mut d = injured_return((60_000, 0), (20_000, 0), 0, allocation());
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        edit_wage(data).withholding = 500_049;
    }
    if let FederalIncome::Wage { data } = &mut d.income[1].income {
        edit_wage(data).withholding = 50_049;
    }
    let mut entry = income(savings_income(1_000, 0));
    if let FederalIncome::Interest { data } = &mut entry.income {
        edit_interest(data).withholding = 149;
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "25a"), 550_100);
    assert_eq!(return_line(&r, "25b"), 100);
    assert_eq!(
        (column(&r, "19", "a"), column(&r, "19", "b"), column(&r, "19", "c")),
        (5_502, 5_001, 501)
    );
}

// Runtime owner feeds must retain the actually allowed joint deductions.
// Two uncovered IRAs: $6,000 + $2,000, both below annual/compensation caps.
#[test]
fn injured_spouse_ira_final_deduction_keeps_both_owners() {
    let mut d = injured_return((60_000, 10_000), (20_000, 1_000), 0, allocation());
    let mut facts = ira_facts(600_000, 0, Answer::No);
    facts.spouse = IraSpouse::Joint { facts: ira_facts(200_000, 0, Answer::No).taxpayer };
    d.adjustments.as_mut().unwrap().iras = facts;
    let r = supported(&d);
    assert_eq!(return_line(&r, "10"), 800_000);
    assert_eq!(
        (column(&r, "14", "a"), column(&r, "14", "b"), column(&r, "14", "c")),
        (8_000, 6_000, 2_000)
    );
    assert!(r.lines.iter().any(|l| l.id == "form_8379_injured_refund_estimate"));
}

// The prior deductible-contribution adjustment reduces the taxpayer's $9,000
// gifts to a $5,000 exclusion. It must not subtract the preliminary $9,000
// entry assignments. Two taxpayer IRAs are $3,000/$7,000, spouse IRA $10,000
// and spouse gift $2,000: final taxable ownership is $5,000/$8,000.
#[test]
fn injured_spouse_qcd_uses_final_partial_exclusion_by_owner() {
    let mut d = injured_return((60_000, 10_000), (20_000, 1_000), 0, allocation());
    let p = d.people.as_mut().unwrap();
    p.taxpayer.birth_date = 19500101;
    p.spouse.as_mut().unwrap().birth_date = 19500101;
    for (owner, amount) in
        [(Owner::Taxpayer, 3_000), (Owner::Taxpayer, 7_000), (Owner::Spouse, 10_000)]
    {
        add(
            &mut d,
            retirement_entry(owner, RetirementKind::Ira, amount, RetirementTreatment::Ordinary),
        );
    }
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261215, 9_000, 4_000), qcd(Owner::Spouse, 20261215, 2_000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 7_000);
    assert_eq!(return_line(&r, "4b"), 1_300_000);
    assert_eq!(
        (column(&r, "13b", "a"), column(&r, "13b", "b"), column(&r, "13b", "c")),
        (13_000, 5_000, 8_000)
    );
    let mut probe = r.clone();
    let q = crate::retirement_return::qualified_charity(
        &d,
        &mut probe,
        None,
        &crate::hsa_funding::FundingResult::default(),
    );
    assert_eq!(q.excluded_by_owner, [500_000, 200_000]);
    assert_eq!(q.excluded, 700_000);
}

// Pub504 recipient recapture: year2 excess39,000-(0+15,000)=24,000;
// year1 excess50,000-((39,000-24,000+0)/2+15,000)=27,500.
// The $51,500 deduction includes no current payer deduction. The collector's
// same-invocation recipient result now owns this deduction independently of
// any current payer source. Keep the original ordinary deduction and tax.
#[test]
fn injured_spouse_unrepresented_recapture_deduction_is_not_guessed() {
    let mut d = injured_return((120_000, 25_000), (40_000, 5_000), 0, allocation());
    let mut received = recapture_alimony();
    received.paid = 0;
    received.alimony_due = 0;
    let mut entry = income(FederalIncome::Alimony { data: received });
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "10"), 5_150_000);
    assert_eq!(column(&r, "14", "a"), 51_500);
    assert_eq!(column(&r, "14", "b"), 0);
    assert_eq!(column(&r, "14", "c"), 51_500);
    assert!(
        !r.review_notes
            .iter()
            .any(|note| { note.display.key == "WARN_8379_ADJUSTMENT_OWNER_UNAVAILABLE" })
    );
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    assert_eq!(r.refund, supported(&ordinary).refund);
}

// New December2026 draft line18 includes Schedule2 line1z. At $160,000
// joint income (>400% of the two-person FPL), this one-month policy's entire
// $1,000 advance is repayable. Chosen injured allocation $700/$300 is kept;
// the same $1,000 must leave the proportionately shared base tax once.
#[test]
fn injured_spouse_line18_includes_aptc_and_excludes_it_from_base_once() {
    let mut d = crate::tests::support::builders::injured_original::fixture("h-taxpayer");
    d.injured_spouse.as_mut().unwrap().allocations = vec![InjuredAmount {
        component: "schedule_2_premium_credit_repayment".into(),
        injured: 70_000,
    }];
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_2_credit_repayments_total"), 1_000);
    assert_eq!(return_line(&r, "23"), 0);
    assert_eq!(
        (column(&r, "18", "a"), column(&r, "18", "b"), column(&r, "18", "c")),
        (1_000, 700, 300)
    );
    let mut no_advance = d.clone();
    let claim = marketplace_claim(&mut no_advance, vec![marketplace_month(1, 1_000, 1_000, 0)]);
    no_advance.credits.as_mut().unwrap().marketplace = claim;
    no_advance.injured_spouse.as_mut().unwrap().allocations.clear();
    let base = supported(&no_advance);
    assert_eq!(
        value(&r, "form_8379_injured_share_of_tax"),
        value(&base, "form_8379_injured_share_of_tax")
    );
    assert_eq!(share_cents(&base) - share_cents(&r), 70_000);
    d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
    // The direct choice is keyed to the injured spouse, so reverse it too.
    d.injured_spouse.as_mut().unwrap().allocations[0].injured = 30_000;
    let reversed = supported(&d);
    assert_eq!(
        (column(&reversed, "18", "a"), column(&reversed, "18", "b"), column(&reversed, "18", "c")),
        (1_000, 300, 700)
    );
}

#[test]
fn injured_spouse_estimated_and_extension_categories_round_separately() {
    let mut d = injured_return((60_000, 10_000), (20_000, 1_000), 0, allocation());
    let p = d.payments.as_mut().unwrap();

    p.estimated = crate::EstimatedPaymentEntryRecord::Sources {
        payments: vec![crate::DatedPayment {
            date: 20260415,
            amount: 49,
            payer: PaymentPayer::Taxpayer,
            refund_property: None,
        }],
    };
    p.extension = 49;
    p.extension_payer = Some(PaymentPayer::Spouse);
    let r = supported(&d);
    assert_eq!((column(&r, "20", "a"), column(&r, "20", "b"), column(&r, "20", "c")), (0, 0, 0));
    assert!(!r.review_notes.iter().any(|n| n.display.key == "WARN_8379_PAYMENT_OWNER_UNAVAILABLE"));
    assert!(r.lines.iter().any(|l| l.id == "form_8379_injured_refund_estimate"));
}

// Concurrent 1z sources: APTC repayment$1,000 and the spouse's new clean
// vehicle dealer-transfer repayment$7,500 (resale within30days). Only APTC
// takes the explicit $700 injured choice; the buyer owes the vehicle tax.
#[test]
fn injured_spouse_aptc_and_vehicle_repayments_have_distinct_owners() {
    let mut d = injured_return((120_000, 30_000), (40_000, 15_000), 0, allocation());
    let mut vehicle = new_clean_vehicle();
    let FederalCleanVehicle::New { details, .. } = &mut vehicle else { unreachable!() };
    details.buyer = Owner::Spouse;
    details.dealer_transfer = Some(750_000);
    details.disposition = crate::CleanVehicleDisposition::ResoldWithin30Days;
    let claim = marketplace_claim(&mut d, vec![marketplace_month(1, 1_000, 1_000, 1_000)]);
    let credits = d.credits.as_mut().unwrap();
    credits.clean_vehicles = Some(clean_vehicle_claim(vec![vehicle]));
    credits.marketplace = claim;
    d.injured_spouse.as_mut().unwrap().allocations = vec![InjuredAmount {
        component: "schedule_2_premium_credit_repayment".into(),
        injured: 70_000,
    }];
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_2_credit_repayments_total"), 8_500);
    assert_eq!(
        (column(&r, "18", "a"), column(&r, "18", "b"), column(&r, "18", "c")),
        (8_500, 700, 7_800)
    );
    let required = &r.injured_spouse.as_ref().unwrap().required;
    assert_eq!(required.len(), 1);
    assert_eq!(required[0].component, "schedule_2_premium_credit_repayment");
    assert_eq!(required[0].joint, 100_000);
    let mut base = d.clone();
    let claim = marketplace_claim(&mut base, vec![marketplace_month(1, 1_000, 1_000, 0)]);
    base.credits.as_mut().unwrap().marketplace = claim;
    base.credits.as_mut().unwrap().clean_vehicles = None;
    base.injured_spouse.as_mut().unwrap().allocations.clear();
    let ordinary = supported(&base);
    assert_eq!(
        value(&r, "form_8379_injured_share_of_tax"),
        value(&ordinary, "form_8379_injured_share_of_tax")
    );
    assert_eq!(share_cents(&ordinary) - share_cents(&r), 70_000);
}

#[test]
fn injured_spouse_hsa_deductions_keep_account_owners() {
    let mut d = injured_return((60_000, 10_000), (20_000, 1_000), 0, allocation());
    d.adjustments.as_mut().unwrap().hsa = vec![
        hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 200_000),
        hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 100_000),
    ];
    let r = supported(&d);
    assert_eq!(
        (column(&r, "14", "a"), column(&r, "14", "b"), column(&r, "14", "c")),
        (3_000, 2_000, 1_000)
    );
}

#[test]
fn injured_spouse_supporting_rows_reopen_unchanged_without_property_questions() {
    use crate::tests::cross::interview::{accepted, document, from_review, rejected_with};
    let mut a = allocation();
    a.community =
        InjuredCommunityFacts::Yes { states: vec![5], debt_facts: InjuredDebtFacts::NotApplicable };
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, a);
    for e in &mut d.income {
        e.community = CommunityShare::Community;
    }
    let mut s = Interview::new(TEST_USER, d);
    // Real income retains the three applicable property choices.
    let P::SaveDocuments { documents, .. } = open_documents(&mut s) else { panic!() };
    assert!(documents.common.wages.documents.iter().all(|r| r.allowed_community
        == vec![
            crate::community::index(CommunityShare::Community),
            crate::community::index(CommunityShare::Separate),
            crate::community::index(CommunityShare::Partly { percent: 50 }),
        ]));
    for (case, topic, fact) in [
        (
            "income-source-1027",
            TaxSection::Retirement,
            IncomeSource::Record {
                value: crate::IncomeRecord::Form8606(Form8606 {
                    adjustment: 0,
                    reservist_repayments: vec![],
                    basis: 100_000,
                    year_end: 1_000_000,
                }),
            },
        ),
        (
            "income-source-1070",
            TaxSection::SelfEmployment,
            IncomeSource::Record {
                value: crate::IncomeRecord::QbiLossCarryforward(QbiLossCarryforward {
                    business: 100_000,
                    reit_ptp: 0,
                    injured_owners: None,
                }),
            },
        ),
    ] {
        let Reply::Edit(mut page) = from_review(&mut s, topic) else { panic!() };
        let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
        documents.documents.push(crate::IncomeRecordItem::from_income(document(0, fact)).unwrap());
        accepted(&send(&mut s, page));
        assert!(s.draft.income.iter().all(|e| e.community == CommunityShare::Community));
        let wages = open_documents(&mut s);
        accepted(&send(&mut s, wages));
        let Reply::Edit(page) = from_review(&mut s, topic) else { panic!() };
        let P::SaveIncomes { documents, .. } = &page else { panic!() };
        assert_eq!(documents.documents.len(), 1);
        assert_eq!(documents.documents[0].community, CommunityShare::NotNeeded);
        assert_eq!(
            documents.documents[0].allowed_community,
            vec![crate::community::index(CommunityShare::NotNeeded)]
        );
        assert_eq!(documents.documents[0].refund_management_needed, Some(false));
        // The actual reply can be submitted unchanged.
        accepted(&send(&mut s, page));
        // A changed income topic normally reopens its section. Complete the
        // displayed public walk, preserving every editor's supplied answers.
        let mut reply = from_review(&mut s, TaxSection::Income);
        let mut reviewed = false;
        for _ in 0..30 {
            reply = crate::tests::cross::interview::past_less_common(&mut s, reply);
            reply = match reply {
                Reply::Edit(page) => send(&mut s, page),
                Reply::Response(hub @ P::Hub { .. }) => {
                    let next = crate::tests::cross::interview::hub_action(&hub);
                    send(&mut s, next)
                }
                Reply::Response(P::Review { .. }) => {
                    reviewed = true;
                    break;
                }
                other => panic!("unexpected public continuation: {other:?}"),
            };
        }
        assert!(reviewed, "public continuation did not reach Review");
        let r = supported(&s.draft);
        assert_eq!(r.tax, Some(524_300));
        assert_eq!(r.refund, Some(25_700));
        assert_eq!(share_cents(&r), 12_850);
        // A forged catalog cannot turn supporting history into community income.
        let Reply::Edit(mut page) = from_review(&mut s, topic) else { panic!() };
        let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
        documents.documents[0].community = CommunityShare::Community;
        documents.documents[0].allowed_community =
            vec![crate::community::index(CommunityShare::Community)];
        let before = s.draft.clone();
        let dirty = s.dirty;
        rejected_with(
            current_income_refusal(
                case,
                &mut s,
                page,
                "COMMUNITY_ANSWER_NOT_ALLOWED",
                "[documents/documents/0/community]",
            ),
            "COMMUNITY_ANSWER_NOT_ALLOWED",
        );
        assert_eq!(s.draft, before);
        assert_eq!(s.dirty, dirty);
    }
    assert_eq!(s.draft.income.len(), 2);
    assert_eq!(s.draft.adjustments.as_ref().unwrap().ira_basis[0].prior, 100_000);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    // Turning off the overlay changes the allowed property choices. Correct
    // the retained source answers through their owning editor.
    let Reply::Edit(mut page) = crate::tests::support::client::native_editor(
        from_review(&mut s, TaxSection::InjuredSpouse),
        &[],
    ) else {
        panic!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
    value.community = InjuredCommunity::No;
    let mut reply = send(&mut s, page);
    for _ in 0..3 {
        match reply {
            Reply::Edit(next @ P::SaveInjuredSpouse { .. }) => reply = send(&mut s, next),
            _ => break,
        }
    }
    accepted(&reply);
    assert!(matches!(
        s.draft.injured_spouse.as_ref().unwrap().community,
        InjuredCommunityFacts::No
    ));
    let mut sources = crate::tests::cross::interview::open_topic(&mut s, TaxSection::Wages);
    let P::SaveWagesAnswers { followups, .. } = &mut sources else { unreachable!() };
    for row in followups {
        let crate::WagesDocumentAnswer::Community { value } = &mut row.answer;
        *value = CommunityShare::NotNeeded;
    }
    accepted(&send(&mut s, sources));
    assert!(s.draft.income.iter().all(|e| e.community == CommunityShare::NotNeeded));
}

#[test]
fn injured_spouse_property_choices_are_row_bound_and_do_not_mutate_the_draft() {
    let mut a = allocation();
    a.community =
        InjuredCommunityFacts::Yes { states: vec![5], debt_facts: InjuredDebtFacts::NotApplicable };
    let mut s = Interview::new(TEST_USER, injured_return((60_000, 5_000), (20_000, 500), 0, a));
    let before = s.draft.clone();
    let dirty = s.dirty;
    for (document, allowed) in [
        (
            IncomeSource::Record {
                value: crate::IncomeRecord::Form8606(Form8606 {
                    adjustment: 0,
                    reservist_repayments: vec![],
                    basis: 100_000,
                    year_end: 1_000_000,
                }),
            },
            vec![crate::community::index(CommunityShare::NotNeeded)],
        ),
        (
            IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                    1_000_000,
                ))),
            },
            vec![
                crate::community::index(CommunityShare::Community),
                crate::community::index(CommunityShare::Separate),
                crate::community::index(CommunityShare::Partly { percent: 50 }),
            ],
        ),
    ] {
        if matches!(document, IncomeSource::Wage { .. }) {
            let page =
                crate::tests::support::client::income_pages::open_topic(&mut s, TaxSection::Wages);
            let P::SaveWagesAnswers { followups, .. } = page else { unreachable!() };
            let row = followups
                .iter()
                .find(|row| row.key == "income/1/community")
                .expect("community answer belongs to the W-2 topic row");
            assert_eq!(row.allowed_values, allowed);
            assert_eq!(s.draft, before);
            assert_eq!(s.dirty, dirty);
            continue;
        }
        let Reply::Response(P::DocumentPropertyChoices { allowed_community }) = send(
            &mut s,
            match document {
                IncomeSource::Record { value: document } => P::DocumentPropertyOptions {
                    source_choices: crate::IncomeSourceChoices::from_kinds(
                        &crate::tests::support::client::income_pages::topic_kinds(
                            TaxSection::Retirement,
                        ),
                    ),
                    initial_asset_amt_carry_choices: Vec::new(),
                    initial_interest_limit_choices: Vec::new(),
                    initial_rental_recovery_choices: Vec::new(),
                    rental_energy_buildings: Vec::new(),
                    casualty_properties: Vec::new(),
                    allowed_continuing_risk: ScorpRiskStart::continuing_choices(),
                    allowed_injured_source_facts: vec![],
                    allowed_injured_retained_owners: vec![],
                    ltc_payee_sources: vec![],

                    wage_scorporations: Vec::new(),
                    allowed_wage_scorporations: Vec::new(),
                    initial_k1_investment_choices: alloc::vec![0, 1],
                    research_businesses: Vec::new(),
                    retained_research_choices: Vec::new(),
                    k1_credit_lines: Vec::new(),
                    research_credit_lines: Vec::new(),
                    renewable_credit_lines: Vec::new(),
                    transferable_credit_lines: Vec::new(),
                    user: TEST_USER,
                    people: vec![],
                    farms: vec![],
                    businesses: vec![],
                    rentals: vec![],
                    mortgage_loans: vec![],
                    basis_assets: vec![],
                    farm_sources: vec![],
                    farm_codes: vec![],
                    farm_limitations: vec![],
                    farm_method_changes: vec![],
                    annuity_sources: vec![],
                    recovery_sources: vec![],
                    cooperative_activities: vec![],

                    cooperative_properties: vec![],
                    foreign_countries: vec![],
                    interest_statements: vec![],
                    royalty_properties: vec![],
                    equipment_rentals: vec![],
                    hobbies: vec![],
                    ccc_lots: vec![],
                    inherited_iras: vec![],
                    inherited_roth_iras: vec![],
                    ltc_insured: vec![],
                    allowed_business_source: vec![],
                    allowed_farm_source: vec![],
                    allowed_partner_source: vec![],
                    allowed_home_spouse: vec![],
                    allowed_other_parent: vec![],
                    children: vec![],
                    document,
                    qbi_businesses: vec![],
                },
                _ => unreachable!(),
            },
        ) else {
            panic!()
        };
        assert_eq!(allowed_community, allowed);
        assert_eq!(s.draft, before);
        assert_eq!(s.dirty, dirty);
    }
}

// Historical decision witness retained as the explicit unknown-answer case.
// Choosing no established spouse shares still cannot infer the historical
// owner from the page row owner. Joint AGI remains $79,000; the benefit
// stays unavailable until the source-owned surviving shares are supplied.
#[test]
fn unit13_joint_nol_unestablished_owner_is_not_inferred_from_page_owner() {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let mut drafts = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut s = Interview::new(TEST_USER, d.clone());
        let mut row = document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::NetOperatingLoss(LossCarryforward {
                    regular: vec![
                        NolYear {
                            ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                            research_parts: None,
                            year: 2017,
                            balance: 100_000,
                            injured_owners: None,
                        }
                        .into(),
                    ],
                    amt: vec![],
                    source: LossSource::Business,
                }),
            },
        );
        row.owner = crate::DocumentOwner::from(owner);
        let request = save_topic(&mut s, TaxSection::SelfEmployment, vec![row]);
        accepted(&send(&mut s, request));
        let result = supported(&s.draft);
        assert_eq!(value(&result, "prior_nol_deduction"), 1000);
        assert_eq!(value(&result, "agi"), 79_000);
        assert!(noted(&result, "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE"));
        assert_eq!(restart(s.draft.clone()).draft, s.draft);
        drafts.push(s.draft);
    }
    assert_eq!(drafts[0], drafts[1]);
}

#[test]
fn unit13_approved_capital_and_qbi_owner_facts_save_reopen_and_keep_joint_totals() {
    use crate::tests::cross::interview::{
        accepted, document, open_topic, page_documents, rejected_with, save_topic,
    };
    let d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let prior = PriorCapitalReturn {
        research_parts: None,
        owners: PriorReturnOwners::Joint,
        status: FederalStatus::Joint,
        agi: 10_000_000,
        deductions: 3_000_000,
        senior_deduction: 0,
        short: -1_000_000,
        long: 0,
        deducted_loss: 300_000,
        section_1256: 0,
        amt: None,
        origins: vec![],
        ordinary_history: true,
        investment_parts: None,
        niit_difference: Some(0),
        injured_niit_owners: None,
        injured_owners: Some(InjuredCapitalLossOwners::Shares {
            taxpayer_short: 200_000,
            spouse_short: 500_000,
            taxpayer_long: 0,
            spouse_long: 0,
        }),
    };
    assert_eq!(crate::extended_rules::regular_capital_carry(&prior), [7000, 0]);
    let mut s = Interview::new(TEST_USER, d);
    let request = save_topic(
        &mut s,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::CapitalLossCarryover(prior.clone().into()),
            },
        )],
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().capital_history[0], prior);
    let before = supported(&s.draft);
    let mut reopened = restart(s.draft.clone());
    let page = open_topic(&mut reopened, TaxSection::InvestmentSales);
    assert!(
        page_documents(&page)
            .iter()
            .any(|v| matches!(&v.document, IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(v) } if crate::PriorCapitalReturn::from(v.clone()) == prior))
    );
    let mut invalid = prior.clone();
    invalid.injured_owners = Some(InjuredCapitalLossOwners::Shares {
        taxpayer_short: 700_001,
        spouse_short: 0,
        taxpayer_long: 0,
        spouse_long: 0,
    });
    let request = save_topic(
        &mut reopened,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::CapitalLossCarryover(invalid.into()),
            },
        )],
    );
    rejected_with(send(&mut reopened, request), "INJURED_CAPITAL_OWNER_TOTAL");
    assert_eq!(reopened.draft, s.draft);
    let qbi = QbiLossCarryforward {
        business: 100_000,
        reit_ptp: 50_000,
        injured_owners: Some(InjuredQbiLossOwners::Shares {
            taxpayer_business: 25_000,
            spouse_business: 75_000,
            taxpayer_reit_ptp: 50_000,
            spouse_reit_ptp: 0,
        }),
    };
    let request = save_topic(
        &mut s,
        TaxSection::SelfEmployment,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(qbi.clone()) },
        )],
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening.as_ref().unwrap().qbi_history.injured_owners, qbi.injured_owners);
    let after = supported(&s.draft);
    assert_eq!(value(&before, "agi"), value(&after, "agi"));
    let mut reopened = restart(s.draft.clone());
    let page = open_topic(&mut reopened, TaxSection::SelfEmployment);
    assert!(
        page_documents(&page)
            .iter()
            .any(|v| matches!(&v.document, IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(v) } if v == &qbi))
    );
}

// Actual ownership changes only Form 8379 columns, never the joint deduction.
#[test]
fn unit13_regular_nol_actual_owners_save_reopen_restart_and_prune() {
    use crate::tests::cross::interview::{
        accepted, document, open_topic, page_documents, rejected_with, save_topic,
    };
    let d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let mut expected_joint = None;
    for (taxpayer, spouse) in [(100_000, 0), (0, 100_000), (30_000, 70_000)] {
        let mut s = Interview::new(TEST_USER, d.clone());
        let row = document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::NetOperatingLoss(LossCarryforward {
                    regular: vec![
                        NolYear {
                            ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                            research_parts: None,
                            year: 2017,
                            balance: 100_000,
                            injured_owners: Some(InjuredNolOwners::Shares { taxpayer, spouse }),
                        }
                        .into(),
                    ],
                    amt: vec![],
                    source: LossSource::Business,
                }),
            },
        );
        accepted(&{
            let request = save_topic(&mut s, TaxSection::SelfEmployment, vec![row]);
            send(&mut s, request)
        });
        let r = supported(&s.draft);
        assert_eq!(value(&r, "prior_nol_deduction"), 1000);
        assert_eq!(value(&r, "agi"), 79_000);
        assert_eq!(column(&r, "13b", "a"), -1000);
        assert_eq!(column(&r, "13b", "b"), -taxpayer / 100);
        assert_eq!(column(&r, "13b", "c"), -spouse / 100);
        assert!(!noted(&r, "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE"));
        let joint = (r.tax, r.refund, r.balance);
        assert_eq!(*expected_joint.get_or_insert(joint), joint);
        let mut reopened = restart(s.draft.clone());
        let page = open_topic(&mut reopened, TaxSection::SelfEmployment);
        let mut rows = page_documents(&page).to_vec();
        let IncomeSource::Record { value: crate::IncomeRecord::NetOperatingLoss(losses) } =
            &mut rows[0].document
        else {
            panic!()
        };
        assert_eq!(
            losses.regular[0].injured_owners.as_ref().unwrap().amounts(),
            [taxpayer, spouse]
        );
        losses.regular[0].injured_owners =
            Some(InjuredNolOwners::Shares { taxpayer: 1, spouse: 1 });
        rejected_with(
            {
                let request = save_topic(&mut reopened, TaxSection::SelfEmployment, rows);
                send(&mut reopened, request)
            },
            "INJURED_NOL_OWNER_TOTAL",
        );
        assert_eq!(reopened.draft, s.draft);
        let mut no_claim = s.draft.clone();
        no_claim.injured_spouse = None;
        no_claim.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
        crate::injured_spouse_sources::clear_unused(&mut no_claim);
        let NolHistory::Records { years, .. } = &no_claim.screening.as_ref().unwrap().nol else {
            panic!()
        };
        assert_eq!(years[0].balance, 100_000);
        assert!(years[0].injured_owners.is_none());
        assert_eq!(supported(&no_claim).tax, r.tax);
    }
}

#[test]
fn unit13_regular_nol_joint_limit_uses_deduction_not_absorption() {
    // MFJ taxable before NOL: 80,000 - 32,200 = 47,800.
    // Modern NOL deduction: 47,800 * 80% = 38,240, all owned by spouse.
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    d.screening.as_mut().unwrap().nol = NolHistory::Records {
        confirmed: true,
        years: vec![NolYear {
            ptp_ebl: crate::PtpEblNol::NoPtpEbl,
            research_parts: None,
            year: 2024,
            balance: 10_000_000,
            injured_owners: Some(InjuredNolOwners::Shares { taxpayer: 0, spouse: 10_000_000 }),
        }],
    };
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 38_240);
    assert_eq!(column(&r, "13b", "a"), -38_240);
    assert_eq!(column(&r, "13b", "b"), 0);
    assert_eq!(column(&r, "13b", "c"), -38_240);
    // Mixed ownership keeps the same deduction; a direct limited allocation
    // has bounds from the actual surviving balances, not an inferred owner.
    let NolHistory::Records { years, .. } = &mut d.screening.as_mut().unwrap().nol else {
        panic!()
    };
    years[0].injured_owners =
        Some(InjuredNolOwners::Shares { taxpayer: 3_000_000, spouse: 7_000_000 });
    d.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
        component: "regular_nol_2024_20240101_20241231".into(),
        injured: 2_000_000,
    });
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "prior_nol_deduction"), 38_240);
    assert_eq!(column(&mixed, "13b", "b"), -20_000);
    assert_eq!(column(&mixed, "13b", "c"), -18_240);
    assert_eq!(mixed.tax, r.tax);
}

#[test]
fn unit13_ltc_actual_period_sources_change_columns_without_joint_tax_change() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    append_injured_two_period_ltc_sources(&mut d);
    assert_eq!(d.income[2].id, 3);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 3770);
    assert_eq!(column(&r, "13b", "a"), 3770);
    assert_eq!(column(&r, "13b", "b"), 2100);
    assert_eq!(column(&r, "13b", "c"), 1670);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::LtcInsured { data } = &mut d.income[2].income else { panic!() };
    let LtcPeriods::Several { periods, .. } = &mut data.periods else { panic!() };
    for period in periods {
        let InjuredLtcPeriodPayees::Payments { payments } = period.injured_payees.as_mut().unwrap();
        payments[0].source = 9 - payments[0].source;
    }
    let swapped = supported(&d);
    assert_eq!(column(&swapped, "13b", "b"), 1670);
    assert_eq!(column(&swapped, "13b", "c"), 2100);
    assert_eq!((swapped.tax, swapped.refund, swapped.balance), (r.tax, r.refund, r.balance));
    let FederalIncome::LtcInsured { data } = &mut d.income[2].income else { panic!() };
    let LtcPeriods::Several { periods, .. } = &mut data.periods else { panic!() };
    for period in periods {
        period.injured_payees = None;
    }
    let unknown = supported(&d);
    assert!(noted(&unknown, "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE"));
    assert_eq!(unknown.tax, r.tax);
}

#[test]
fn unit13_adoption_refund_uses_the_approved_direct_credit_allocation() {
    let mut a = allocation();
    a.allocations
        .push(InjuredAmount { component: "adoption_refundable_credit".into(), injured: 70_000 });
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, a);
    d.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic { finalized: AdoptionFinalization::Pending, special_needs: false },
        &[(2025, 3000)],
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_refundable_credit"), 3000);
    assert_eq!(column(&r, "17", "a"), 3000);
    assert_eq!(column(&r, "17", "b"), 700);
    assert_eq!(column(&r, "17", "c"), 2300);
    assert!(!noted(&r, "WARN_8379_ADOPTION_OWNER_UNAVAILABLE"));
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    let joint = supported(&ordinary);
    assert_eq!((r.tax, r.refund, r.balance), (joint.tax, joint.refund, joint.balance));
}

#[test]
fn unit13_fund_refundable_credit_follows_actual_statement_owner() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let mut fund = income(undistributed_gain());
    fund.owner = crate::DocumentOwner::Spouses;
    add(&mut d, fund);
    let r = supported(&d);
    assert_eq!(column(&r, "17", "a"), 1050);
    assert_eq!(column(&r, "17", "b"), 0);
    assert_eq!(column(&r, "17", "c"), 1050);
    assert!(!noted(&r, "WARN_8379_REFUNDABLE_OWNER_UNAVAILABLE"));
}

#[test]
fn unit13_taxable_social_security_uses_approved_source_default_and_direct_override() {
    let mut d = injured_return((60_000, 8_000), (20_000, 2_000), 0, allocation());
    let mut first = income(social_security_net(2_000_000, 0));
    first.owner = crate::DocumentOwner::Yours;
    add(&mut d, first);
    let mut second = income(social_security_net(1_000_000, 0));
    second.owner = crate::DocumentOwner::Spouses;
    add(&mut d, second);
    // Provisional income is well above the upper joint threshold: taxable
    // benefits are85% of$30,000=$25,500. The approved default follows the
    // actual2:1 source-benefit ratio, with the joint amount preserved.
    let before = supported(&d);
    assert_eq!(value(&before, "taxable_social_security"), 25_500);
    assert_eq!((column(&before, "13b", "b"), column(&before, "13b", "c")), (17_000, 8_500));
    let default = before
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|p| p.component == "taxable_social_security")
        .unwrap();
    assert_eq!((default.minimum, default.maximum, default.injured), (0, 2_550_000, 1_700_000));
    assert!(
        !before.review_notes.iter().any(|n| n.display.key == "WARN_8379_ALLOCATION_CHOICES_NEEDED")
    );
    d.injured_spouse
        .as_mut()
        .unwrap()
        .allocations
        .push(InjuredAmount { component: "taxable_social_security".into(), injured: 1_000_000 });
    let after = supported(&d);
    assert_eq!((column(&after, "13b", "b"), column(&after, "13b", "c")), (10_000, 15_500));
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(after.tax, before.tax);
    assert_eq!(after.refund, before.refund);
}

#[test]
fn unit13_social_default_uses_injured_identity_and_joint_dollar_rounding() {
    let mut a = allocation();
    a.injured = Owner::Spouse;
    let mut d = injured_return((60_000, 8_000), (20_000, 2_000), 0, a);
    let mut first = income(social_security_net(2_000_000, 0));
    first.owner = crate::DocumentOwner::Yours;
    add(&mut d, first);
    let mut second = income(social_security_net(1_000_100, 0));
    second.owner = crate::DocumentOwner::Spouses;
    add(&mut d, second);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_social_security"), 25_501);
    assert_eq!((column(&r, "13b", "b"), column(&r, "13b", "c")), (8_501, 17_000));
    let choice = r
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|p| p.component == "taxable_social_security")
        .unwrap();
    assert_eq!(choice.injured, 850_100);
}

fn injured_two_students(d: &mut FederalDraft, method: EducationMethod, expenses: [i64; 2]) {
    set_education(
        d.credits.as_mut().unwrap(),
        [-1, -2]
            .into_iter()
            .zip(expenses)
            .map(|(student, expense)| {
                let mut pair = native_education(
                    student,
                    education_expenses(expense * 100),
                    education_choice(method, Answer::Yes),
                );
                pair.0.id = 0;
                pair.0.half_time = true;
                pair.0.graduate = method == EducationMethod::LifetimeLearning;
                pair.1.more.american_opportunity_used_up = Some(false);
                pair
            })
            .collect(),
    );
}

#[test]
fn unit13_education_credits_follow_each_actual_student_after_the_joint_phaseout() {
    for (wages, expected) in [(80_000, [1_500, 600, 1_000, 400]), (170_000, [750, 300, 500, 200])] {
        let mut d = injured_return((wages - 20_000, 8_000), (20_000, 2_000), 0, allocation());
        injured_two_students(&mut d, EducationMethod::AmericanOpportunity, [4_000, 1_000]);
        // AOTC bases:2500/1000; at170k joint AGI the common phaseout is50%.
        // Refundable40% follows each actual adult student, and the remaining
        // allowed nonrefundable credit is60%; no MFS eligibility recomputation.
        let r = supported(&d);
        assert_eq!((column(&r, "16", "b"), column(&r, "16", "c")), (expected[0], expected[1]));
        assert_eq!((column(&r, "17", "b"), column(&r, "17", "c")), (expected[2], expected[3]));
        assert!(
            !r.review_notes
                .iter()
                .any(|n| n.display.key == "WARN_8379_DEPENDENT_CREDIT_OWNER_UNAVAILABLE")
        );
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
        let joint = supported(&ordinary);
        assert_eq!(r.tax, joint.tax);
        assert_eq!(r.refund, joint.refund);
        assert_eq!(value(&r, "agi"), value(&joint, "agi"));
        d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
        let switched = supported(&d);
        assert_eq!(
            (column(&switched, "16", "b"), column(&switched, "17", "b")),
            (expected[1], expected[3])
        );
    }
}

#[test]
fn unit13_llc_uses_the_actual_joint_expenses_consumed_for_each_student() {
    let mut d = injured_return((60_000, 8_000), (20_000, 2_000), 0, allocation());
    injured_two_students(&mut d, EducationMethod::LifetimeLearning, [8_000, 4_000]);
    let r = supported(&d);
    // The owning calculator already consumes the$10,000 return cap in list
    // order:8000/2000, at20%, with no phaseout at joint AGI80,000.
    assert_eq!(value(&r, "education_credit"), 2_000);
    assert_eq!((column(&r, "16", "b"), column(&r, "16", "c")), (1_600, 400));
    assert_eq!(value(&r, "refundable_education_credit"), 0);
}

#[test]
fn unit13_education_joint_tax_limit_exposes_only_claimant_bounded_allocation() {
    let mut d = injured_return((25_000, 2_000), (15_000, 1_000), 0, allocation());
    injured_two_students(&mut d, EducationMethod::AmericanOpportunity, [4_000, 1_000]);
    let before = supported(&d);
    // Joint taxable income7800 uses the table midpoint7825: tax783. The
    // actual nonrefundable credit is783, with student ceilings1500/600.
    assert_eq!(value(&before, "education_credit"), 783);
    let question = before
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|p| p.component == "education_nonrefundable_credit")
        .unwrap();
    assert_eq!((question.minimum, question.maximum), (18_300, 78_300));
    d.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
        component: "education_nonrefundable_credit".into(),
        injured: 40_000,
    });
    let after = supported(&d);
    assert_eq!((column(&after, "16", "b"), column(&after, "16", "c")), (400, 383));
    assert_eq!((column(&after, "17", "b"), column(&after, "17", "c")), (1_000, 400));
    assert_eq!(after.tax, before.tax);
    assert_eq!(after.refund, before.refund);
}

#[test]
fn unit13_reit_qbi_uses_actual_retained_loss_owners_and_keeps_printed_line15() {
    let mut d = injured_return((60_000, 20_000), (20_000, 8_000), 0, allocation());
    for (owner, amount) in [(Owner::Taxpayer, 30_000), (Owner::Spouse, 20_000)] {
        let mut entry = income(reit_income(amount, amount, true));
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    let history = &mut d.screening.as_mut().unwrap().qbi_history;
    history.reit_ptp = 1_000_000;
    history.injured_owners = Some(InjuredQbiLossOwners::Shares {
        taxpayer_business: 0,
        spouse_business: 0,
        taxpayer_reit_ptp: 1_000_000,
        spouse_reit_ptp: 0,
    });
    let r = supported(&d);
    // Actual joint qualified REIT pool:50000-10000=40000; deduction8000.
    // Actual retained loss belongs to TP: bases20000/20000, shares4000/4000.
    assert_eq!(value(&r, "qbi_deduction"), 8_000);
    assert_eq!(
        (value(&r, "form_8379_supplemental_qbi_b"), value(&r, "form_8379_supplemental_qbi_c")),
        (4_000, 4_000)
    );
    assert_eq!(
        (column(&r, "15", "a"), column(&r, "15", "b"), column(&r, "15", "c")),
        (32_200, 16_100, 16_100)
    );
    assert!(!r.review_notes.iter().any(|n| n.display.key == "WARN_8379_QBI_OWNER_UNAVAILABLE"));
    d.screening.as_mut().unwrap().qbi_history.injured_owners = Some(InjuredQbiLossOwners::Shares {
        taxpayer_business: 0,
        spouse_business: 0,
        taxpayer_reit_ptp: 0,
        spouse_reit_ptp: 1_000_000,
    });
    let swapped = supported(&d);
    // Same joint facts, actual bases30000/10000: shares6000/2000.
    assert_eq!(
        (
            value(&swapped, "form_8379_supplemental_qbi_b"),
            value(&swapped, "form_8379_supplemental_qbi_c")
        ),
        (6_000, 2_000)
    );
    assert_eq!(value(&swapped, "agi"), value(&r, "agi"));
    assert_eq!(swapped.tax, r.tax);
    assert_eq!(swapped.refund, r.refund);
    d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
    let reversed = supported(&d);
    assert_eq!(
        (
            value(&reversed, "form_8379_supplemental_qbi_b"),
            value(&reversed, "form_8379_supplemental_qbi_c")
        ),
        (2_000, 6_000)
    );
    d.screening.as_mut().unwrap().qbi_history.injured_owners = None;
    unavailable(&d, "WARN_8379_QBI_OWNER_UNAVAILABLE");
}

#[test]
fn unit13_business_qbi_follows_final_source_after_owner_half_se() {
    let mut d = injured_return((60_000, 15_000), (20_000, 8_000), 0, allocation());
    for (owner, amount) in [(Owner::Taxpayer, 10_000), (Owner::Spouse, 20_000)] {
        let mut entry = income(cash_business(amount));
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    let r = supported(&d);
    // SE earnings9235/18470; SE tax1413/2826; deductible halves707/1413.
    // Actual source QBI9293/18587, joint27880, deduction5576.
    assert_eq!(value(&r, "qbi_deduction"), 5_576);
    assert_eq!(
        (value(&r, "form_8379_supplemental_qbi_b"), value(&r, "form_8379_supplemental_qbi_c")),
        (1_859, 3_717)
    );
    assert!(!r.review_notes.iter().any(|n| n.display.key == "WARN_8379_QBI_OWNER_UNAVAILABLE"));
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let joint = supported(&ordinary);
    assert_eq!(r.tax, joint.tax);
    assert_eq!(r.refund, joint.refund);
    assert_eq!(value(&r, "agi"), value(&joint, "agi"));
}

fn injured_care_people(d: &mut FederalDraft, costs: [i64; 2]) {
    for (owner, cost) in [Owner::Taxpayer, Owner::Spouse].into_iter().zip(costs) {
        add_child(d);
        let child = d.dependents.last_mut().unwrap();
        child.set_care_expenses(cost * 100);
        d.injured_spouse
            .as_mut()
            .unwrap()
            .claimants
            .push(InjuredClaimantFacts { person: child.id, claimant: owner });
    }
    d.credits.as_mut().unwrap().care.providers.push(CareProvider {
        id: 0,
        name: "Synthetic eligible daycare".into(),
        address: "1 Example Way".into(),
        identification: ProviderIdentification::Organization,
        household_employee: false,
        paid: costs.iter().sum::<i64>() * 100,
        relationship: ProviderRelationship::NotRelated,
    });
}

#[test]
fn unit13_current_and_prior_care_follow_actual_people_and_different_year_rates() {
    let mut d = injured_return((60_000, 8_000), (20_000, 2_000), 0, allocation());
    injured_care_people(&mut d, [900, 100]);
    d.credits.as_mut().unwrap().care.prior_year = Some(PriorYearCare {
        two_or_more: true,
        prior_line_3: 0,
        paid_in_return_year: 100_000,
        prior_benefits: 0,
        prior_earned_smaller: 5_000_000,
        prior_line_6: 0,
        prior_agi: 5_000_000,
        injured_claimants: Some(InjuredPriorCareClaimants::People {
            people: vec![
                InjuredPriorCarePerson {
                    person: InjuredPriorCareIdentity::Other { name: "Prior P".into() },
                    claimant: Owner::Taxpayer,
                    expense: 10_000,
                },
                InjuredPriorCarePerson {
                    person: InjuredPriorCareIdentity::Other { name: "Prior Q".into() },
                    claimant: Owner::Spouse,
                    expense: 90_000,
                },
            ],
        }),
    });
    let r = supported(&d);
    // Current2026 rate35%:315/35; retained2025 rate20%:20/180.
    // Each claimant also has an independently allowed2200 child credit.
    assert_eq!(value(&r, "care_credit"), 550);
    assert_eq!((column(&r, "16", "b"), column(&r, "16", "c")), (2_535, 2_415));
    assert!(
        !r.review_notes
            .iter()
            .any(|n| n.display.key == "WARN_8379_DEPENDENT_CREDIT_OWNER_UNAVAILABLE")
    );
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let joint = supported(&ordinary);
    assert_eq!(r.tax, joint.tax);
    assert_eq!(r.refund, joint.refund);
    d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
    let reversed = supported(&d);
    assert_eq!((column(&reversed, "16", "b"), column(&reversed, "16", "c")), (2_415, 2_535));
}

#[test]
fn unit13_joint_care_expense_limit_exposes_claimant_source_bounds() {
    let mut d = injured_return((60_000, 8_000), (20_000, 2_000), 0, allocation());
    injured_care_people(&mut d, [5_000, 3_000]);
    let r = supported(&d);
    // Joint6000 expense ceiling at35%=2100. Actual claimant source
    // ceilings1750/1050 constrain the injured part to1050..1750.
    assert_eq!(value(&r, "care_credit"), 2_100);
    let q = r
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|q| q.component == "care_nonrefundable_credit")
        .unwrap();
    assert_eq!((q.joint, q.minimum, q.maximum), (210_000, 105_000, 175_000));
    d.injured_spouse
        .as_mut()
        .unwrap()
        .allocations
        .push(InjuredAmount { component: q.component.clone(), injured: 150_000 });
    // 2026-10-15: Required answers and page construction (doc/app-contract.md):
    // the former expected2443/2800 implicitly used the CTC lower bound943.
    // State that answer and the native1257 ACTC pool's zero injured share
    // explicitly; unanswered bounds are not ownership facts.
    d.injured_spouse.as_mut().unwrap().allocations.extend([
        InjuredAmount { component: "ctc_odc_pool".into(), injured: 94_300 },
        InjuredAmount { component: "actc_pool".into(), injured: 0 },
    ]);
    let chosen = supported(&d);
    assert_eq!((column(&chosen, "16", "b"), column(&chosen, "16", "c")), (2_443, 2_800));
    assert_eq!(chosen.tax, r.tax);
    assert_eq!(chosen.refund, r.refund);
}

// Full-map diagnostic: regular surviving owner shares do not identify which
// spouse owns the independent retained Form8960 excluded-loss difference.
#[test]
fn unit13_retained_niit_difference_owner_is_not_encoded_by_joint_capital_shares() {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let prior = PriorCapitalReturn {
        research_parts: None,
        owners: PriorReturnOwners::Joint,
        status: FederalStatus::Joint,
        agi: 10_000_000,
        deductions: 3_000_000,
        senior_deduction: 0,
        short: -2_300_000,
        long: 0,
        deducted_loss: 300_000,
        section_1256: 0,
        amt: None,
        origins: vec![],
        ordinary_history: true,
        investment_parts: None,
        niit_difference: Some(1_000_000),
        injured_niit_owners: None,
        injured_owners: Some(InjuredCapitalLossOwners::Shares {
            taxpayer_short: 1_000_000,
            spouse_short: 1_000_000,
            taxpayer_long: 0,
            spouse_long: 0,
        }),
    };
    assert_eq!(crate::extended_rules::regular_capital_carry(&prior), [20_000, 0]);
    let mut drafts = vec![];
    let mut results = vec![];
    for page_owner in [Owner::Taxpayer, Owner::Spouse] {
        let d = injured_return((300_000, 60_000), (100_000, 20_000), 0, allocation());
        let mut session = Interview::new(TEST_USER, d);
        let mut row = document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::CapitalLossCarryover(prior.clone().into()),
            },
        );
        row.owner = crate::DocumentOwner::from(page_owner);
        let page = save_topic(&mut session, TaxSection::InvestmentSales, vec![row]);
        accepted(&send(&mut session, page));
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut entry = income(ordinary_investment_sale(30_000, 0, GainTerm::Short));
            entry.owner = crate::DocumentOwner::from(owner);
            add(&mut session.draft, entry);
        }
        let r = supported(&session.draft);
        assert_eq!(value(&r, "capital_gain"), 40_000);
        assert_eq!(value(&r, "niit_disposition_adjustment"), 10_000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 50_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 1_900);
        results.push((r.tax, r.refund));
        drafts.push(session.draft);
    }
    assert_eq!(drafts[0], drafts[1]);
    assert_eq!(results[0], results[1]);
    // Independent source worlds: regular capital shares20000/20000 in both.
    // Retained excluded-loss difference10000 belongs to TP or SP. With MAGI
    //440000, the190000 threshold excess does not bind either50000 NII pool.
    // Actual source NII30000/20000 versus20000/30000 gives1140/760 versus
    //760/1140 at3.8%. No field in the accepted joint history records that axis.
    let expected = [[1_140, 760], [760, 1_140]];
    for (source, taxes) in [[30_000, 20_000], [20_000, 30_000]].into_iter().zip(expected) {
        assert_eq!(source.map(|nii| crate::kernel::ratio::fraction(nii, 38, 1000)), taxes);
        assert_eq!(taxes.iter().sum::<i64>(), 1_900);
    }
    assert_ne!(expected[0], expected[1]);
}

#[test]
fn unit13_contract_carryback_ownership_is_not_encoded_by_surviving_prior_shares() {
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let prior = PriorCapitalReturn {
        research_parts: None,
        owners: PriorReturnOwners::Joint,
        status: FederalStatus::Joint,
        agi: 10_000_000,
        deductions: 3_000_000,
        senior_deduction: 0,
        short: -1_100_000,
        long: -1_200_000,
        deducted_loss: 300_000,
        section_1256: 1_000_000,
        amt: None,
        origins: vec![],
        ordinary_history: true,
        investment_parts: None,
        niit_difference: Some(0),
        injured_niit_owners: None,
        injured_owners: Some(InjuredCapitalLossOwners::Shares {
            taxpayer_short: 320_000,
            spouse_short: 480_000,
            taxpayer_long: 480_000,
            spouse_long: 720_000,
        }),
    };
    assert_eq!(crate::extended_rules::regular_capital_carry(&prior), [8_000, 12_000]);
    let mut drafts = vec![];
    let mut results = vec![];
    for page_owner in [Owner::Taxpayer, Owner::Spouse] {
        let d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
        let mut session = Interview::new(TEST_USER, d);
        let history = document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::CapitalLossCarryover(prior.clone().into()),
            },
        );
        let mut election = document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Section1256Carryback(Section1256Carryback {
                    amount: 800_000,
                    injured_owners: None,
                }),
            },
        );
        election.owner = crate::DocumentOwner::from(page_owner);
        let page = save_topic(&mut session, TaxSection::InvestmentSales, vec![history, election]);
        accepted(&send(&mut session, page));
        let gain = income(ordinary_investment_sale(2_000, 0, GainTerm::Long));
        add(&mut session.draft, gain);
        let mut loss = income(FederalIncome::Section1256 {
            data: contracts_source(FederalSection1256 {
                aggregate: -100_000,
                withholding: 0,
                special_treatment: false,
            }),
        });
        loss.owner = crate::DocumentOwner::Spouses;
        add(&mut session.draft, loss);
        let r = supported(&session.draft);
        assert_eq!(value(&r, "form_6781_net"), -1_000);
        assert_eq!(value(&r, "form_6781_carryback"), 8_000);
        assert_eq!(value(&r, "section_1256_net_loss"), 8_000);
        assert_eq!(value(&r, "capital_gain"), -3_000);
        assert_eq!(value(&r, "capital_loss_carryforward"), 8_000);
        assert_eq!(value(&r, "agi"), 77_000);
        results.push((r.tax, r.refund));
        drafts.push(session.draft);
    }
    assert_eq!(drafts[0], drafts[1]);
    assert_eq!(results[0], results[1]);
    // Independent source worlds: before the2026 election regular surviving
    // balances areTP8000/SP12000 in both, and the section1256 subset10000
    // may beTP8000/SP2000 orTP0/SP10000. An actual absorbed8000 carryback
    // can belong wholly to TP or SP, respectively. Include the accepted
    // TP2000 current gain and SP1000 current contract loss once.
    let before = [2_000 - 8_000, -1_000 - 12_000];
    let worlds = [[before[0] + 8_000, before[1]], [before[0], before[1] + 8_000]];
    assert_eq!(worlds, [[2_000, -13_000], [-6_000, -5_000]]);
    assert_eq!(worlds.map(|parts| parts.iter().sum::<i64>()), [-11_000; 2]);
    // In the first world the only loss owner is SP: preserving TP's2000
    // gain gives capital columns2000/-5000 under the actual joint3000 loss
    // limit. In the second both actual owner nets are negative. Thus the
    // post-carryback source bases differ before the annual allocation. This
    // witness does not prescribe a unique split of the joint3000 loss.
    assert_eq!(2_000 + -5_000, -3_000);
    assert!(worlds[1].iter().all(|v| *v < 0));
}

#[test]
fn unit13_approved_retained_niit_owners_save_without_changing_joint_tax() {
    use crate::tests::cross::interview::{accepted, document, rejected_with, save_topic};
    let mut ordinary = None;
    for source in [
        InjuredRetainedOwners::Taxpayer,
        InjuredRetainedOwners::Spouse,
        InjuredRetainedOwners::Both { taxpayer: 400_000, spouse: 600_000 },
    ] {
        let d = injured_return((300_000, 60_000), (100_000, 20_000), 0, allocation());
        let mut session = Interview::new(TEST_USER, d);
        let mut prior = retained_owner_source();
        prior.injured_niit_owners = Some(source.clone());
        let request = save_topic(
            &mut session,
            TaxSection::InvestmentSales,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::CapitalLossCarryover(prior.clone().into()),
                },
            )],
        );
        accepted(&send(&mut session, request));
        let record = &session.draft.screening.as_ref().unwrap().capital_history[0];
        assert_eq!(record.injured_niit_owners, Some(source));
        let r = supported(&session.draft);
        let tax = (
            r.tax,
            r.refund,
            value(&r, "capital_gain"),
            value(&r, "form_8960_net_investment_income"),
        );
        if let Some(before) = ordinary {
            assert_eq!(tax, before);
        } else {
            ordinary = Some(tax);
        }
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
        let before = session.draft.clone();
        prior.injured_niit_owners =
            Some(InjuredRetainedOwners::Both { taxpayer: 500_000, spouse: 600_000 });
        let bad = save_topic(
            &mut session,
            TaxSection::InvestmentSales,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::CapitalLossCarryover(prior.into()),
                },
            )],
        );
        rejected_with(send(&mut session, bad), "INJURED_NIIT_OWNER_TOTAL");
        assert_eq!(session.draft, before);
    }
    let mut capped = retained_owner_source();
    capped.injured_owners = Some(InjuredCapitalLossOwners::Shares {
        taxpayer_short: 300_000,
        spouse_short: 1_700_000,
        taxpayer_long: 0,
        spouse_long: 0,
    });
    capped.injured_niit_owners = Some(InjuredRetainedOwners::Taxpayer);
    assert!(!crate::injured_spouse_sources::niit_owners_valid(&capped));
}

#[test]
fn unit13_approved_absorbed_carryback_owners_save_restart_and_reject_forged_total() {
    use crate::tests::cross::interview::{accepted, document, rejected_with, save_topic};
    let mut ordinary = None;
    for owner in [
        InjuredRetainedOwners::Taxpayer,
        InjuredRetainedOwners::Spouse,
        InjuredRetainedOwners::Both { taxpayer: 300_000, spouse: 500_000 },
    ] {
        let d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
        let mut session = Interview::new(TEST_USER, d);
        let prior = retained_owner_source();
        let election =
            Section1256Carryback { amount: 800_000, injured_owners: Some(owner.clone()) };
        let request = save_topic(
            &mut session,
            TaxSection::InvestmentSales,
            vec![
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::CapitalLossCarryover(prior.clone().into()),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Section1256Carryback(election.clone()),
                    },
                ),
            ],
        );
        accepted(&send(&mut session, request));
        let record = session.draft.screening.as_ref().unwrap();
        assert_eq!(record.section_1256_carryback, Some(800_000));
        assert_eq!(record.section_1256_carryback_owners, Some(owner));
        let mut contracts = income(FederalIncome::Section1256 {
            data: contracts_source(FederalSection1256 {
                aggregate: -100_000,
                withholding: 0,
                special_treatment: false,
            }),
        });
        contracts.owner = crate::DocumentOwner::Spouses;
        add(&mut session.draft, contracts);
        let r = supported(&session.draft);
        let tax = (r.tax, r.refund, value(&r, "capital_gain"));
        if let Some(before) = ordinary {
            assert_eq!(tax, before);
        } else {
            ordinary = Some(tax);
        }
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
        let mut bad = election;
        bad.injured_owners =
            Some(InjuredRetainedOwners::Both { taxpayer: 400_000, spouse: 500_000 });
        let before = session.draft.clone();
        let request = save_topic(
            &mut session,
            TaxSection::InvestmentSales,
            vec![
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::CapitalLossCarryover(prior.into()),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Section1256Carryback(bad) },
                ),
            ],
        );
        rejected_with(send(&mut session, request), "INJURED_CARRYBACK_OWNER_TOTAL");
        assert_eq!(session.draft, before);
        session.draft.injured_spouse = None;
        session.draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
        crate::injured_spouse_sources::clear_unused(&mut session.draft);
        assert!(session.draft.screening.as_ref().unwrap().section_1256_carryback_owners.is_none());
        assert_eq!(session.draft.screening.as_ref().unwrap().section_1256_carryback, Some(800_000));
    }
}

#[test]
fn unit13_final_capital_sources_keep_signed_owner_nets_and_prior_loss_character() {
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    for (owner, gain) in [(Owner::Taxpayer, 6_000), (Owner::Spouse, 4_000)] {
        let mut sale = income(ordinary_investment_sale(gain, 0, GainTerm::Short));
        sale.owner = crate::DocumentOwner::from(owner);
        add(&mut d, sale);
    }
    let mut prior = retained_owner_source();
    prior.short = -1_000_000;
    prior.section_1256 = 0;
    prior.niit_difference = Some(0);
    prior.injured_owners = Some(InjuredCapitalLossOwners::Shares {
        taxpayer_short: 200_000,
        spouse_short: 500_000,
        taxpayer_long: 0,
        spouse_long: 0,
    });
    d.screening.as_mut().unwrap().capital_history = vec![prior];
    let r = supported(&d);
    // Current6000/4000 less actual prior2000/5000:4000/-1000.
    // The capital net3000 is positive, so no annual loss floor applies.
    assert_eq!(value(&r, "capital_gain"), 3_000);
    assert_eq!((column(&r, "13b", "b"), column(&r, "13b", "c")), (4_000, -1_000));
    let mut plain = d.clone();
    plain.injured_spouse = None;
    let joint = supported(&plain);
    assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unit13_joint_capital_loss_floor_requires_only_a_bounded_actual_loss_choice() {
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    for (owner, basis, term) in
        [(Owner::Taxpayer, 10_000, GainTerm::Short), (Owner::Spouse, 5_000, GainTerm::Long)]
    {
        let mut sale = income(ordinary_investment_sale(0, basis, term));
        sale.owner = crate::DocumentOwner::from(owner);
        add(&mut d, sale);
    }
    let unresolved = supported(&d);
    assert_eq!(value(&unresolved, "capital_gain"), -3_000);
    let row = unresolved
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|r| r.component == "capital_loss_limit")
        .unwrap();
    assert_eq!((row.joint, row.minimum, row.maximum), (-300_000, -300_000, 0));
    assert!(unresolved.lines.iter().all(|r| r.id != "form_8379_injured_refund_estimate"));
    d.injured_spouse
        .as_mut()
        .unwrap()
        .allocations
        .push(InjuredAmount { component: "capital_loss_limit".into(), injured: -100_000 });
    let chosen = supported(&d);
    assert_eq!((column(&chosen, "13b", "b"), column(&chosen, "13b", "c")), (-1_000, -2_000));
    assert_eq!((chosen.tax, chosen.refund), (unresolved.tax, unresolved.refund));
    // Switching the injured identity changes column identity, not source tax.
    d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
    d.injured_spouse.as_mut().unwrap().allocations[0].injured = -200_000;
    let reversed = supported(&d);
    assert_eq!((column(&reversed, "13b", "b"), column(&reversed, "13b", "c")), (-2_000, -1_000));
    assert_eq!((reversed.tax, reversed.refund), (unresolved.tax, unresolved.refund));
    d.injured_spouse.as_mut().unwrap().allocations[0].injured = -300_100;
    let forged = supported(&d);
    assert!(forged.lines.iter().all(|r| r.id != "form_8379_injured_refund_estimate"));
    assert!(
        forged.review_notes.iter().any(|n| n.display.key == "WARN_8379_ALLOCATION_CHOICES_NEEDED")
    );
}

#[test]
fn unit13_capital_loss_floor_preserves_the_other_spouses_actual_positive_gain() {
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    for (owner, proceeds, basis) in [(Owner::Taxpayer, 2_000, 0), (Owner::Spouse, 0, 15_000)] {
        let mut sale = income(ordinary_investment_sale(proceeds, basis, GainTerm::Short));
        sale.owner = crate::DocumentOwner::from(owner);
        add(&mut d, sale);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3_000);
    assert_eq!((column(&r, "13b", "b"), column(&r, "13b", "c")), (2_000, -5_000));
    assert!(
        r.injured_spouse
            .as_ref()
            .unwrap()
            .required
            .iter()
            .all(|r| r.component != "capital_loss_limit")
    );
    assert_eq!(value(&r, "capital_loss_carryforward"), 10_000);
}

#[test]
fn unit13_independent_retained_niit_owners_reach_actual_line18_without_joint_change() {
    let mut prior = retained_owner_source();
    prior.section_1256 = 0;
    let mut a = allocation();
    // The existing direct joint AdditionalMedicare choice is independent of
    // the actual NII source split under test:900/450 of the joint1350.
    a.allocations
        .push(InjuredAmount { component: "additional_medicare_tax".into(), injured: 90_000 });
    let mut d = injured_return((300_000, 60_000), (100_000, 20_000), 0, a);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut sale = income(ordinary_investment_sale(30_000, 0, GainTerm::Short));
        sale.owner = crate::DocumentOwner::from(owner);
        add(&mut d, sale);
    }
    let mut totals = None;
    for (source, niit) in [
        (InjuredRetainedOwners::Taxpayer, [1_140, 760]),
        (InjuredRetainedOwners::Spouse, [760, 1_140]),
        (InjuredRetainedOwners::Both { taxpayer: 400_000, spouse: 600_000 }, [912, 988]),
    ] {
        prior.injured_niit_owners = Some(source);
        d.screening.as_mut().unwrap().capital_history = vec![prior.clone()];
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 40_000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 50_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 1_900);
        assert_eq!((column(&r, "13b", "b"), column(&r, "13b", "c")), (20_000, 20_000));
        assert_eq!((column(&r, "18", "b"), column(&r, "18", "c")), (niit[0] + 900, niit[1] + 450));
        assert!(
            !r.review_notes
                .iter()
                .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        if let Some(joint) = totals {
            assert_eq!((r.tax, r.refund), joint);
        } else {
            totals = Some((r.tax, r.refund));
        }
        assert_eq!(restart(d.clone()).draft, d);
    }
    // The same regular20k carry and10k independent difference never
    // establish its owner when the retained joint source answer is absent.
    d.screening.as_mut().unwrap().capital_history[0].injured_niit_owners = None;
    let unknown = supported(&d);
    assert!(
        unknown
            .review_notes
            .iter()
            .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
    assert_eq!((unknown.tax, unknown.refund), totals.unwrap());
    assert_eq!((column(&unknown, "13b", "b"), column(&unknown, "13b", "c")), (20_000, 20_000));
}

#[test]
fn unit13_absorbed_carryback_owners_reach_actual_capital_loss_bounds() {
    let mut prior = retained_owner_source();
    prior.short = -1_100_000;
    prior.long = -1_200_000;
    prior.niit_difference = Some(0);
    prior.injured_owners = Some(InjuredCapitalLossOwners::Shares {
        taxpayer_short: 320_000,
        spouse_short: 480_000,
        taxpayer_long: 480_000,
        spouse_long: 720_000,
    });
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    add(&mut d, income(ordinary_investment_sale(2_000, 0, GainTerm::Long)));
    let mut loss = income(FederalIncome::Section1256 {
        data: contracts_source(FederalSection1256 {
            aggregate: -100_000,
            withholding: 0,
            special_treatment: false,
        }),
    });
    loss.owner = crate::DocumentOwner::Spouses;
    add(&mut d, loss);
    let screening = d.screening.as_mut().unwrap();
    screening.capital_history = vec![prior];
    screening.section_1256_carryback = Some(800_000);
    screening.section_1256_carryback_owners = Some(InjuredRetainedOwners::Taxpayer);
    let taxpayer = supported(&d);
    assert_eq!((column(&taxpayer, "13b", "b"), column(&taxpayer, "13b", "c")), (2_000, -5_000));
    assert_eq!(value(&taxpayer, "capital_loss_carryforward"), 8_000);
    d.screening.as_mut().unwrap().section_1256_carryback_owners =
        Some(InjuredRetainedOwners::Spouse);
    let spouse = supported(&d);
    let row = spouse
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|r| r.component == "capital_loss_limit")
        .unwrap();
    // Actual source nets are now-6000/-5000. IRS does not prescribe a
    // unique split of this joint annual3000 loss; ask the direct bound.
    assert_eq!((row.minimum, row.maximum), (-300_000, 0));
    d.injured_spouse
        .as_mut()
        .unwrap()
        .allocations
        .push(InjuredAmount { component: "capital_loss_limit".into(), injured: -100_000 });
    let chosen = supported(&d);
    assert_eq!((column(&chosen, "13b", "b"), column(&chosen, "13b", "c")), (-1_000, -2_000));
    assert_eq!((chosen.tax, chosen.refund), (taxpayer.tax, taxpayer.refund));
    assert_eq!(value(&chosen, "agi"), 77_000);
}

#[test]
fn unit13_retained_installment_and_exchange_gains_keep_their_filers() {
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    let screening = d.screening.as_mut().unwrap();
    screening.installment_payments = vec![EarlierInstallment {
        owner: Owner::Spouse,
        sale: InstallmentPayments {
            property: "Synthetic investment parcel".into(),
            sold: 20230601,
            gross_profit: 600_000,
            contract_price: 1_000_000,
            earlier_payments: 100_000,
            payments: 500_000,
            kind: PropertyCharacter::LongTermCapital,
            special: false,
            qbi: InstallmentQbi::NotQualified,
            livestock: None,
        },
    }];
    screening.related_exchanges = vec![EarlierRelatedExchange {
        owner: Owner::Taxpayer,
        exchange: RelatedExchange {
            property: "Synthetic investment exchange".into(),
            last_transfer: 20240501,
            disposed: 20260430,
            exception: RelatedException::None,
            deferred: 200_000,
            kind: PropertyCharacter::LongTermCapital,
            special: false,
        },
    }];
    let r = supported(&d);
    //6000/10000 gross-profit ratio times5000 current installment=3000,
    // plus2000 actual related-party recognition, in different owner records.
    assert_eq!(value(&r, "capital_gain"), 5_000);
    assert_eq!((column(&r, "13b", "b"), column(&r, "13b", "c")), (2_000, 3_000));
    let mut plain = d.clone();
    plain.injured_spouse = None;
    let joint = supported(&plain);
    assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unit13_retained_owners_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{
        accepted, document, open_topic, page_documents, rejected_with,
    };
    let selected_kind = std::env::var("TAX_UNIT13_RETAINED_KIND").ok();
    let selected_state = std::env::var("TAX_UNIT13_RETAINED_STATE").ok();
    for kind in ["niit", "carryback"] {
        if selected_kind.as_deref().is_some_and(|v| v != kind) {
            continue;
        }
        for state in ["taxpayer", "spouse", "both", "null"] {
            if selected_state.as_deref().is_some_and(|v| v != state) {
                continue;
            }
            let limited = kind == "niit"
                && std::env::var("TAX_UNIT13_RETAINED_LIMITED").as_deref() == Ok("1");
            let (own, spouse) = if limited {
                ((200_000, 60_000), (30_000, 10_000))
            } else if kind == "niit" {
                ((300_000, 60_000), (100_000, 20_000))
            } else {
                ((60_000, 10_000), (20_000, 5_000))
            };
            let mut a = allocation();
            if kind == "niit" && !limited {
                a.allocations.push(InjuredAmount {
                    component: "additional_medicare_tax".into(),
                    injured: 90_000,
                });
            }
            let mut d = injured_return(own, spouse, 0, a);
            let mut prior = retained_owner_source();
            if kind == "niit" {
                prior.section_1256 = 0;
                for owner in [Owner::Taxpayer, Owner::Spouse] {
                    let mut e = income(ordinary_investment_sale(30_000, 0, GainTerm::Short));
                    e.owner = crate::DocumentOwner::from(owner);
                    add(&mut d, e);
                }
            } else {
                prior.short = -1_100_000;
                prior.long = -1_200_000;
                prior.niit_difference = Some(0);
                prior.injured_owners = Some(InjuredCapitalLossOwners::Shares {
                    taxpayer_short: 320_000,
                    spouse_short: 480_000,
                    taxpayer_long: 480_000,
                    spouse_long: 720_000,
                });
                add(&mut d, income(ordinary_investment_sale(2_000, 0, GainTerm::Long)));
                let mut loss = income(FederalIncome::Section1256 {
                    data: contracts_source(FederalSection1256 {
                        aggregate: -100_000,
                        withholding: 0,
                        special_treatment: false,
                    }),
                });
                loss.owner = crate::DocumentOwner::Spouses;
                add(&mut d, loss);
            }
            let source = match state {
                "taxpayer" => Some(InjuredRetainedOwners::Taxpayer),
                "spouse" => Some(InjuredRetainedOwners::Spouse),
                "both" => Some(InjuredRetainedOwners::Both {
                    taxpayer: if kind == "niit" { 400_000 } else { 300_000 },
                    spouse: if kind == "niit" { 600_000 } else { 500_000 },
                }),
                "null" => None,
                _ => unreachable!(),
            };
            let mut session = Interview::new(TEST_USER, d);
            let page = open_topic(&mut session, TaxSection::InvestmentSales);
            let native = NativeInterviewCase("TAX_UNIT13_RETAINED");
            if native.required_calculation(&mut session) {
                return;
            }
            let seed_directory = std::env::var("TAX_UNIT13_RETAINED_SEED_DIR").ok();
            if let Some(dir) = &seed_directory {
                let dir = std::path::Path::new(dir);
                std::fs::create_dir_all(dir).unwrap();
                std::fs::write(dir.join("seed.avro"), page.to_avro()).unwrap();
            }
            let mut answers = page;
            let P::SaveIncomes { documents, .. } = &mut answers else {
                panic!();
            };
            prior.injured_niit_owners = if kind == "niit" { source.clone() } else { None };
            documents.documents.push(
                crate::IncomeRecordItem::from_income(document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::CapitalLossCarryover(prior.into()),
                    },
                ))
                .expect("record fixture"),
            );
            if kind == "carryback" {
                documents.documents.push(
                    crate::IncomeRecordItem::from_income(document(
                        0,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::Section1256Carryback(
                                Section1256Carryback {
                                    amount: 800_000,
                                    injured_owners: source.clone(),
                                },
                            ),
                        },
                    ))
                    .expect("record fixture"),
                );
            }
            if let Some(dir) = seed_directory {
                std::fs::write(std::path::Path::new(&dir).join("answers.avro"), answers.to_avro())
                    .unwrap();
                continue;
            }
            let request = if let Ok(input) = std::env::var("TAX_UNIT13_RETAINED_PAYLOAD") {
                P::from_avro(
                    &std::fs::read(input).expect("genuine fresh Studio source-row payload"),
                )
                .unwrap()
            } else {
                answers
            };
            accepted(&send(&mut session, request));
            let source = if let Ok(input) = std::env::var("TAX_UNIT13_RETAINED_CORRECTION") {
                assert_eq!(state, "both");
                let before = supported(&session.draft);
                let request = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
                accepted(&send(&mut session, request));
                let after = supported(&session.draft);
                assert_eq!((after.tax, after.refund), (before.tax, before.refund));
                Some(InjuredRetainedOwners::Both {
                    taxpayer: if kind == "niit" { 400_100 } else { 300_100 },
                    spouse: if kind == "niit" { 599_900 } else { 499_900 },
                })
            } else {
                source
            };
            let screening = session.draft.screening.as_ref().unwrap();
            assert_eq!(screening.capital_history.len(), 1);
            assert_eq!(
                if kind == "niit" {
                    &screening.capital_history[0].injured_niit_owners
                } else {
                    &screening.section_1256_carryback_owners
                },
                &source
            );
            let result = supported(&session.draft);
            if kind == "niit" {
                assert_eq!(value(&result, "capital_gain"), 40_000);
                assert_eq!(
                    value(&result, "net_investment_income_tax"),
                    if limited { 760 } else { 1_900 }
                );
                if state != "null" {
                    let niit = match (state, limited) {
                        ("taxpayer", true) => [456, 304],
                        ("spouse", true) => [304, 456],
                        (_, true) => [365, 395],
                        ("taxpayer", false) => [1140, 760],
                        ("spouse", false) => [760, 1140],
                        (_, false) => [912, 988],
                    };
                    assert_eq!(
                        (column(&result, "18", "b"), column(&result, "18", "c")),
                        (
                            niit[0] + if limited { 0 } else { 900 },
                            niit[1] + if limited { 0 } else { 450 }
                        )
                    );
                }
            } else {
                assert_eq!(value(&result, "capital_gain"), -3_000);
                assert_eq!(value(&result, "agi"), 77_000);
            }
            let mut plain = session.draft.clone();
            plain.injured_spouse = None;
            let joint = supported(&plain);
            assert_eq!((result.tax, result.refund), (joint.tax, joint.refund));
            let tax_year = session.draft.tax_year;
            let persisted = native_draft_records(&session.draft);
            let decoded = draft_from_native_records(tax_year, &persisted);
            assert_eq!(native_draft_records(&decoded), persisted);
            assert_eq!(decoded, session.draft);
            let mut reopened = restart(session.draft.clone());
            let shown = open_topic(&mut reopened, TaxSection::InvestmentSales);
            assert_eq!(
                page_documents(&shown).iter().find_map(|r| match &r.document {
                    IncomeSource::Record {
                        value: crate::IncomeRecord::CapitalLossCarryover(p),
                    } if kind == "niit" => Some(p.injured_niit_owners.clone()),
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Section1256Carryback(p),
                    } if kind == "carryback" => Some(p.injured_owners.clone()),
                    _ => None,
                }),
                Some(source.clone())
            );
            if let Ok(dir) = std::env::var("TAX_UNIT13_RETAINED_OUTPUT") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(std::path::Path::new(&dir).join("reopened.avro"), shown.to_avro())
                    .unwrap();
            }
            let before = reopened.draft.clone();
            let mut forged = shown;
            let P::SaveIncomes { documents, .. } = &mut forged else {
                panic!();
            };
            for row in &mut documents.documents {
                match &mut row.document {
                    crate::IncomeRecord::CapitalLossCarryover(p) if kind == "niit" => {
                        p.injured_niit_owners =
                            Some(InjuredRetainedOwners::Both { taxpayer: 500_000, spouse: 600_000 })
                    }
                    crate::IncomeRecord::Section1256Carryback(p) if kind == "carryback" => {
                        p.injured_owners =
                            Some(InjuredRetainedOwners::Both { taxpayer: 400_000, spouse: 500_000 })
                    }
                    _ => {}
                }
            }
            rejected_with(
                send(&mut reopened, forged),
                if kind == "niit" {
                    "INJURED_NIIT_OWNER_TOTAL"
                } else {
                    "INJURED_CARRYBACK_OWNER_TOTAL"
                },
            );
            assert_eq!(reopened.draft, before);
        }
    }
}

#[test]
fn unit13_niit_magi_ceiling_preserves_the_actual_net_investment_income_allocation() {
    let mut prior = retained_owner_source();
    prior.section_1256 = 0;
    let mut d = injured_return((200_000, 60_000), (30_000, 10_000), 0, allocation());
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(ordinary_investment_sale(30_000, 0, GainTerm::Short));
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    let mut ordinary = None;
    for (source, expected) in [
        (InjuredRetainedOwners::Taxpayer, [456, 304]),
        (InjuredRetainedOwners::Spouse, [304, 456]),
        (InjuredRetainedOwners::Both { taxpayer: 400_000, spouse: 600_000 }, [365, 395]),
    ] {
        prior.injured_niit_owners = Some(source);
        d.screening.as_mut().unwrap().capital_history = vec![prior.clone()];
        let result = supported(&d);
        // Actual joint AGI270k gives a20k MAGI excess, smaller than50k NII.
        // Instructions8379 line18 retains each actual NII source percentage.
        assert_eq!(value(&result, "agi"), 270_000);
        assert_eq!(value(&result, "capital_gain"), 40_000);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 50_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 760);
        assert_eq!(
            (column(&result, "18", "b"), column(&result, "18", "c")),
            (expected[0], expected[1])
        );
        assert!(
            !result
                .review_notes
                .iter()
                .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        if let Some(before) = ordinary {
            assert_eq!((result.tax, result.refund), before);
        } else {
            ordinary = Some((result.tax, result.refund));
        }
        d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
        let reversed = supported(&d);
        assert_eq!(
            (column(&reversed, "18", "b"), column(&reversed, "18", "c")),
            (expected[1], expected[0])
        );
        assert_eq!((reversed.tax, reversed.refund), (result.tax, result.refund));
        d.injured_spouse.as_mut().unwrap().injured = Owner::Taxpayer;
    }
    // A current tax ceiling cannot establish missing historical source ownership.
    d.screening.as_mut().unwrap().capital_history[0].injured_niit_owners = None;
    let unknown = supported(&d);
    assert_eq!(value(&unknown, "net_investment_income_tax"), 760);
    assert_eq!((unknown.tax, unknown.refund), ordinary.unwrap());
    assert!(
        unknown
            .review_notes
            .iter()
            .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
}

#[test]
fn unit13_liability_owner_api_uses_actual_sources_without_a_claim_or_debt_gate() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let mut interest = income(savings_income(1_000, 0));
    interest.owner = crate::DocumentOwner::Spouses;
    add(&mut d, interest);
    d.injured_spouse = None;
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let result = supported(&d);
    let original_result = result.clone();
    let facts = liability_portfolio_native_facts(&d, Some([0; 2]), Some(&result));
    let totals = actual_liability_totals(&result);
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &result,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    // Existing hand computation: MFS taxable43900/4900, tax5023/493.
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), Some([5_023, 493]));
    assert_eq!(result, original_result); // Pure: no source/form/notice emission.
    let mut claim = allocation();
    claim.debt = OffsetDebt::NoneOfThese;
    claim.obligated = true;
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &result,
        Some(&claim),
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), Some([5_023, 493]));
}

#[test]
fn unit13_liability_owner_api_does_not_feed_current_business_credit_back_into_itself() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    let mut interest = income(savings_income(1_000, 0));
    interest.owner = crate::DocumentOwner::Spouses;
    add(&mut d, interest);
    d.injured_spouse = None;
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let mut joint_tax = None;
    for amount in [0, 100_000] {
        d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
            carryforwards: if amount == 0 {
                vec![]
            } else {
                vec![BusinessCreditCarryforward {
                    credit: BusinessCredit::WorkOpportunity,
                    origin_year: 2025,
                    amount,
                    passive: None,
                    filer: CarryFiler::SameFilers,
                    research_source: None,
                }]
            },
            carrybacks: vec![],
            spouse_without_credits: false,
            credits: vec![],
        });
        let result = supported(&d);
        assert_eq!(
            result
                .lines
                .iter()
                .find(|row| row.id == "schedule_3_general_business_credit")
                .map_or(0, |row| row.amount / 100),
            amount / 100
        );
        let totals = actual_liability_totals(&result);
        assert_eq!(totals.other_nonrefundable, 0);
        let facts = liability_portfolio_native_facts(&d, Some([0; 2]), Some(&result));
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &d,
                &crate::related_forms::effective_claimants(&d),
            ),
            &result,
            None,
            &facts,
            Some(&totals),
        )
        .unwrap();
        assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), Some([5_023, 493]));
        if let Some(before) = joint_tax {
            assert_eq!(result.tax.unwrap(), before - amount);
        } else {
            joint_tax = result.tax;
        }
    }
}

#[test]
fn unit13_liability_owner_api_refuses_incomplete_sources_or_inconsistent_joint_footing() {
    let mut d = injured_return((60_000, 5_000), (20_000, 500), 0, allocation());
    d.injured_spouse = None;
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let result = supported(&d);
    let facts = liability_portfolio_native_facts(&d, None, Some(&result));
    let totals = actual_liability_totals(&result);
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &result,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), None);
    let facts = liability_portfolio_native_facts(&d, Some([0; 2]), Some(&result));
    let mut forged = actual_liability_totals(&result);
    forged.taxable += 1;
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &result,
        None,
        &facts,
        Some(&forged),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), None);
    forged = actual_liability_totals(&result);
    forged.qualified_dividends = 1;
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &result,
        None,
        &facts,
        Some(&forged),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), None);
    // An absent native component is not a computed zero, even when its
    // omission leaves the same aggregate taxable-income footing.
    forged = actual_liability_totals(&result);
    forged.calculation_amounts.remove("schedule_1_ira_deduction");
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &result,
        None,
        &facts,
        Some(&forged),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), None);
    let zero = injured_return((0, 0), (0, 0), 0, allocation());
    let result = supported(&zero);
    let facts = liability_portfolio_native_facts(&zero, Some([0; 2]), Some(&result));
    let totals = actual_liability_totals(&result);
    let projection = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &zero,
            &crate::related_forms::effective_claimants(&zero),
        ),
        &result,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&zero, &projection), None);
}

#[test]
fn unit13_niit_annuity_uses_its_actual_taxable_owner_and_excludes_qualified_pension() {
    for annuity_owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut claim = allocation();
        // Existing direct allocation: the two-worker Medicare pool has no
        // prescribed unique owner ratio. This independent choice is explicit.
        claim
            .allocations
            .push(InjuredAmount { component: "additional_medicare_tax".into(), injured: 4_800 });
        let mut d = injured_return((140_000, 30_000), (120_000, 25_000), 0, claim);
        let mut entry = retirement_entry(
            annuity_owner,
            RetirementKind::Nonqualified,
            30_000,
            RetirementTreatment::Ordinary,
        );
        let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
        edit_retirement(data).box_2a = 2_500_000;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
        // Printed D identifies the nonqualified source when the ordinary
        // source page saves; code7 alone is a qualified pension statement.
        edit_retirement(data).box7 = DistributionCodes::CODE_7.0 | DistributionCodes::CODE_D.0;
        add(&mut d, entry);
        add(
            &mut d,
            retirement_entry(
                if annuity_owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer },
                RetirementKind::Pension,
                10_000,
                RetirementTreatment::Ordinary,
            ),
        );
        let result = supported(&d);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 25_000);
        assert_eq!(value(&result, "nonqualified_annuity_investment_income"), 25_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 950);
        // The actual explicit Medicare choice divides90 as48/42.
        let expected = if annuity_owner == Owner::Taxpayer { [998, 42] } else { [48, 992] };
        assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], expected);
        assert!(
            !result
                .review_notes
                .iter()
                .any(|note| note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        // Complete public Save/reopen/restart/correction on the existing
        // source page, rather than changing an aggregate Form8960 answer.
        use crate::tests::cross::interview::{accepted, page_documents};
        let mut session = Interview::new(TEST_USER, d.clone());
        let page = open_documents(&mut session);
        accepted(&send(&mut session, page));
        assert_eq!(supported(&session.draft).tax, result.tax);
        let mut session = restart(session.draft);
        let mut page = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
        let item = documents
            .common
            .retirement
            .documents
            .iter_mut()
            .find(|item| match &item.document {
                crate::RetirementDocument::R1099(record) => record.gross == 3_000_000,
                _ => false,
            })
            .unwrap();
        let crate::RetirementDocument::R1099(record) = &mut item.document else { unreachable!() };
        assert_eq!(record.taxable, Some(2_500_000));
        record.taxable = Some(2_000_000);
        accepted(&send(&mut session, page));
        let corrected = supported(&session.draft);
        assert_eq!(value(&corrected, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&corrected, "net_investment_income_tax"), 760);
        let expected_corrected =
            if annuity_owner == Owner::Taxpayer { [808, 42] } else { [48, 802] };
        assert_eq!(
            [column(&corrected, "18", "b"), column(&corrected, "18", "c")],
            expected_corrected
        );
        let mut session = restart(session.draft);
        let reopened = open_documents(&mut session);
        assert!(page_documents(&reopened).iter().any(|item| matches!(&item.document,
            IncomeSource::Retirement { value: crate::RetirementDocument::R1099(record) } if record.gross == 3_000_000 && record.taxable == Some(2_000_000))));
        d.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
        d.injured_spouse.as_mut().unwrap().allocations[0].injured = 4_200;
        let reversed = supported(&d);
        assert_eq!(
            [column(&reversed, "18", "b"), column(&reversed, "18", "c")],
            [expected[1], expected[0]]
        );
        assert_eq!((reversed.tax, reversed.refund), (result.tax, result.refund));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn unit13_niit_savings_penalty_follows_its_source_and_preserves_joint_floor() {
    let mut claim = allocation();
    claim
        .allocations
        .push(InjuredAmount { component: "additional_medicare_tax".into(), injured: 4_800 });
    let mut d = injured_return((140_000, 30_000), (120_000, 25_000), 0, claim);
    let first = income(savings_income(2_000, 1_500));
    add(&mut d, first);
    let mut second = income(savings_income(1_000, 0));
    second.owner = crate::DocumentOwner::Spouses;
    add(&mut d, second);
    let result = supported(&d);
    assert_eq!(value(&result, "form_8960_deductions_and_modifications"), 1_500);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 1_500);
    assert_eq!(value(&result, "net_investment_income_tax"), 57);
    assert_eq!([column(&result, "14", "b"), column(&result, "14", "c")], [1_500, 0]);
    // Source NII500/1000 consumes the same existing57 joint tax pool;
    // source tax19/38 plus explicit Medicare48/42 gives67/80.
    assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [67, 80]);
    assert!(
        !result
            .review_notes
            .iter()
            .any(|note| note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
    assert_eq!(restart(d.clone()).draft, d);
    use crate::tests::cross::interview::{accepted, page_documents};
    let mut session = Interview::new(TEST_USER, d.clone());
    let page = open_documents(&mut session);
    accepted(&send(&mut session, page));
    assert_eq!(supported(&session.draft).tax, result.tax);
    let mut session = restart(session.draft);
    let reopened = open_documents(&mut session);
    assert_eq!(
        page_documents(&reopened)
            .iter()
            .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
            .count(),
        2
    );
    assert_eq!(
        [
            column(&supported(&session.draft), "18", "b"),
            column(&supported(&session.draft), "18", "c")
        ],
        [67, 80]
    );
    // A penalty exceeding its owner's income leaves a signed source. Keep
    // that allocation unavailable; do not invent zero or shift the expense.
    let FederalIncome::Interest { data } = &mut d.income[2].income else { unreachable!() };
    edit_interest(data).early_penalty = 250_000;
    let signed = supported(&d);
    assert_eq!(value(&signed, "form_8960_net_investment_income"), 500);
    assert_eq!(value(&signed, "net_investment_income_tax"), 19);
    assert!(
        signed
            .review_notes
            .iter()
            .any(|note| note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
}

#[test]
fn unit13_native_other_income_sources_keep_taxable_parts_and_exclusions() {
    use crate::tests::cross::interview::{accepted, open_topic, page_documents};
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    let sources = [
        (Owner::Taxpayer, OtherIncomeKind::CardInsurance { premiums: 600_000 }, 9_000),
        (
            Owner::Spouse,
            OtherIncomeKind::Prize { noncash: 100_000, disposition: PrizeDisposition::Received },
            2_000,
        ),
        (Owner::Taxpayer, OtherIncomeKind::DigitalAssets, 5_000),
        (Owner::Taxpayer, OtherIncomeKind::MedicaidWaiver { in_your_home: true }, 1_000),
        (Owner::Spouse, OtherIncomeKind::MedicaidWaiver { in_your_home: false }, 2_000),
        (Owner::Spouse, OtherIncomeKind::UnemploymentFund { contributions: 100_000 }, 3_000),
        (Owner::Taxpayer, OtherIncomeKind::AlaskaDividend, 500),
        (Owner::Spouse, OtherIncomeKind::InmatePay, 100),
        (Owner::Taxpayer, OtherIncomeKind::HouseholdWages, 400),
        (Owner::Spouse, OtherIncomeKind::StrikeBenefits, 200),
        (
            Owner::Taxpayer,
            OtherIncomeKind::Other { description: "Synthetic research study".into() },
            300,
        ),
        (
            Owner::Spouse,
            OtherIncomeKind::Prize { noncash: 0, disposition: PrizeDisposition::Refused },
            4_000,
        ),
    ];
    for (owner, kind, amount) in sources {
        let mut entry = income(FederalIncome::OtherIncome {
            data: OtherIncomeItem { kind, amount: amount * 100, withholding: 0 },
        });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    let result = supported(&d);
    assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [9_200, 7_300]);
    assert_eq!(value(&result, "agi"), 96_500);
    assert!(
        result
            .review_notes
            .iter()
            .all(|note| note.display.key != "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE")
    );
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    assert_eq!(
        (supported(&ordinary).tax, supported(&ordinary).refund),
        (result.tax, result.refund)
    );
    let mut session = Interview::new(TEST_USER, d);
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    assert_eq!(page_documents(&page).len(), 12);
    accepted(&send(&mut session, page));
    let mut session = restart(session.draft);
    let reopened = open_topic(&mut session, TaxSection::OtherIncome);
    assert_eq!(page_documents(&reopened).len(), 12);
    let reopened_result = supported(&session.draft);
    assert_eq!(
        [column(&reopened_result, "13b", "b"), column(&reopened_result, "13b", "c")],
        [9_200, 7_300]
    );
    assert_eq!((reopened_result.tax, reopened_result.refund), (result.tax, result.refund));
}

#[test]
fn unit13_native_routed_1099_sources_count_destination_income_only_once() {
    use crate::tests::cross::interview::{accepted, open_topic};
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    add(&mut d, income(cash_business(10_000)));
    let business_id = d.income[2].id;
    for (compensation, belongs) in [(4_000, Some(IncomeUse::NotForProfit)), (2_000, None)] {
        add(
            &mut d,
            income(FederalIncome::Nec1099 {
                data: nec_source(FederalForm1099Nec {
                    cash_tips: 0,
                    tip_occupation_codes: Default::default(),
                    overtime: 0,
                    compensation: compensation * 100,
                    withholding: 0,
                    golden_parachute: 0,
                    belongs,
                }),
            }),
        );
    }
    let mut misc = income(FederalIncome::Misc1099 {
        data: misc_source(FederalForm1099Misc {
            cash_tips: 0,
            tip_occupation_codes: Default::default(),
            overtime: 0,
            crop_insurance: 0,
            component_ids: Default::default(),
            rents: 0,

            royalties: 0,

            other: 600_000,
            other_use: Some(IncomeUse::Income {
                kind: OtherIncomeKind::CardInsurance { premiums: 250_000 },
            }),
            withholding: 0,
            fishing: 0,
            medical: 0,
            substitute: 50_000,
            attorney: 0,
            client_funds: 0,
            fish_sales: 0,
            deferred: 0,
        }),
    });
    misc.owner = crate::DocumentOwner::Spouses;
    add(&mut d, misc);
    add(
        &mut d,
        income(FederalIncome::K1099 {
            data: k_source(FederalForm1099K {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                gross: 1_100_000,
                withholding: 0,
                parts: vec![
                    FederalKPart {
                        id: 0,
                        amount: 1_000_000,
                        belongs: Some(FederalKUse::NotIncome),
                        refunded: 0,
                    },
                    FederalKPart { id: 0, amount: 100_000, belongs: None, refunded: 20_000 },
                ],
            }),
        }),
    );
    let keys =
        crate::related_forms::catalog_for(&d, business_id).iter().map(|source| source.id).collect();
    let FederalIncome::Business { data } = &mut d.income[2].income else { panic!() };
    data.related = keys;
    let result = supported(&d);
    // Business10000 +NEC2000 +K(1000-200)=12800 is counted by the
    // destination owner. NEC hobby4000 and MISC(6000-2500+500)=4000
    // retain their own source owners. The K nonincome10000 contributes0.
    assert_eq!(value(&result, "schedule_1_business_income"), 12_800);
    assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [16_800, 4_000]);
    assert!(
        result
            .review_notes
            .iter()
            .all(|note| note.display.key != "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE")
    );
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    assert_eq!(
        (supported(&ordinary).tax, supported(&ordinary).refund),
        (result.tax, result.refund)
    );
    let mut session = Interview::new(TEST_USER, d);
    for topic in [TaxSection::SelfEmployment, TaxSection::OtherIncome] {
        let page = open_topic(&mut session, topic);
        accepted(&send(&mut session, page));
        session = restart(session.draft);
    }
    let restored = supported(&session.draft);
    assert_eq!([column(&restored, "13b", "b"), column(&restored, "13b", "c")], [16_800, 4_000]);
    assert_eq!((restored.tax, restored.refund), (result.tax, result.refund));
}

#[test]
fn unit13_native_other_projection_uses_final_recovery_and_preserves_missing_sale() {
    let mut recovery = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    add(
        &mut recovery,
        income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                kind: OtherIncomeKind::Recovery {
                    worksheet: PriorYearRecovery {
                        year: 2025,
                        deducted: 400_000,
                        itemized: 3_500_000,
                        standard: 3_000_000,
                        taxable_income: 5_000_000,
                        ordinary: true,
                    },
                },
                amount: 300_000,
                withholding: 0,
            },
        }),
    );
    assert_eq!(value(&supported(&recovery), "agi"), 83_000);
    let result = supported(&recovery);
    assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [3_000, 0]);
    let mut refund = income(FederalIncome::GovernmentPayment {
        data: tax_refund_source(crate::FederalTaxRefund {
            amount: 300_000,
            withholding: 0,
            history: TaxRefundHistory::IncomeTax {
                deduction_year: 2025,
                income_paid: 1_000_000,
                sales_available: 0,
                salt_paid: 1_000_000,
                salt_deducted: 1_000_000,
                itemized: 3_500_000,
                standard: 3_000_000,
                overall_limit: None,
                property_refunded: 0,
                ordinary: true,
            },
        }),
    });
    refund.owner = crate::DocumentOwner::Spouses;
    add(&mut recovery, refund);
    // The actual combined Worksheet2 ceiling is5000, not two independent
    // stand-alone3000 recoveries. Its native A/B/C split gives2500 each.
    let combined = supported(&recovery);
    assert_eq!(value(&combined, "agi"), 85_000);
    assert_eq!([column(&combined, "13b", "b"), column(&combined, "13b", "c")], [2_500, 2_500]);
    assert!(
        combined
            .review_notes
            .iter()
            .all(|note| note.display.key != "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE")
    );
    let restored_draft = crate::tests::support::client::save_reopen_native_sources(
        recovery,
        &[TaxSection::OtherIncome, TaxSection::Documents],
        &[TaxSection::OtherIncome, TaxSection::Documents],
    );
    let restored = supported(&restored_draft);
    assert_eq!([column(&restored, "13b", "b"), column(&restored, "13b", "c")], [2_500, 2_500]);
    assert_eq!((restored.tax, restored.refund), (combined.tax, combined.refund));
    let mut missing = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    add(
        &mut missing,
        income(FederalIncome::K1099 {
            data: k_source(FederalForm1099K {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                gross: 100_000,
                withholding: 0,
                parts: vec![FederalKPart {
                    id: 0,
                    amount: 100_000,
                    belongs: Some(FederalKUse::PersonalGain),
                    refunded: 0,
                }],
            }),
        }),
    );
    let result = supported(&missing);
    assert_eq!(value(&result, "agi"), 80_000);
    assert!(
        result.review_notes.iter().any(|note| note.display.key == "NOTE_1099K_GAIN_NEEDS_SALE")
    );
    unavailable(&missing, "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE");
}

#[test]
fn unit13_native_olympic_exclusion_follows_the_actual_recipient_after_joint_agi_test() {
    use crate::tests::cross::interview::{accepted, open_topic};
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    for (owner, amount) in [(Owner::Taxpayer, 1_500), (Owner::Spouse, 500)] {
        let mut entry = income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                kind: OtherIncomeKind::OlympicPrize,
                amount: amount * 100,
                withholding: 0,
            },
        });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 80_000);
    assert_eq!(value(&result, "olympic_exclusion"), 2_000);
    for line in ["13b", "14"] {
        assert_eq!([column(&result, line, "b"), column(&result, line, "c")], [1_500, 500]);
    }
    assert!(
        result
            .review_notes
            .iter()
            .all(|note| note.display.key != "WARN_8379_ADJUSTMENT_OWNER_UNAVAILABLE")
    );
    let mut session = Interview::new(TEST_USER, d);
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    accepted(&send(&mut session, page));
    let restored = supported(&restart(session.draft).draft);
    assert_eq!([column(&restored, "14", "b"), column(&restored, "14", "c")], [1_500, 500]);
    assert_eq!((restored.tax, restored.refund), (result.tax, result.refund));
}

#[test]
fn unit13_native_substitute_payments_keep_niit_source_separate_from_other_income() {
    let mut claim = allocation();
    claim
        .allocations
        .push(InjuredAmount { component: "additional_medicare_tax".into(), injured: 4_800 });
    let mut d = injured_return((140_000, 30_000), (120_000, 25_000), 0, claim);
    add(&mut d, income(savings_income(2_000, 0)));
    let mut entry = income(FederalIncome::Misc1099 {
        data: misc_source(FederalForm1099Misc {
            cash_tips: 0,
            tip_occupation_codes: Default::default(),
            overtime: 0,
            crop_insurance: 0,
            component_ids: Default::default(),
            rents: 0,

            royalties: 0,

            other: 600_000,
            other_use: Some(IncomeUse::Income {
                kind: OtherIncomeKind::CardInsurance { premiums: 250_000 },
            }),
            withholding: 0,
            fishing: 0,
            medical: 0,
            substitute: 50_000,
            attorney: 0,
            client_funds: 0,
            fish_sales: 0,
            deferred: 0,
        }),
    });
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut d, entry);
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 266_000);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 2_500);
    assert_eq!(value(&result, "net_investment_income_tax"), 95);
    assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [2_000, 4_000]);
    // The existing native NIIT owner includes interest2000 and substitute500,
    // not the card insurance3500. Actual tax76/19 plus chosen Medicare48/42.
    assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [124, 61]);
    assert!(
        result
            .review_notes
            .iter()
            .all(|note| note.display.key != "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
}

// The final activity owner, not gross rent or a hypothetical MFS return,
// supplies these losses. At90,000 joint rental MAGI the25,000 allowance plus
// 10,000 passive profit admits all26,000; at170,000 only10,000 is admitted.
#[test]
fn unit13_native_schedule_e_sources_follow_the_joint_passive_limit() {
    for (wages, allowed, agi) in [(80_000, 26_000, 64_000), (160_000, 10_000, 160_000)] {
        let mut d = injured_return((wages - 20_000, 20_000), (20_000, 10_000), 0, allocation());
        let mut profit = income(FederalIncome::Rental { data: rental(20_000, 0) });
        profit.owner = crate::DocumentOwner::Yours;
        add(&mut d, profit);
        let profit_id = d.income.last().unwrap().id;
        let mut loss = income(FederalIncome::Rental { data: rental(0, 16_000) });
        loss.owner = crate::DocumentOwner::Spouses;
        add(&mut d, loss);
        let loss_id = d.income.last().unwrap().id;
        let result = supported(&d);
        assert_eq!(value(&result, "agi"), agi);
        assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [10_000, -allowed]);
        let mut output = FederalResult::empty(d.revision);
        let native = crate::extended_rules::rentals(&d, wages + 10_000, &mut output);
        assert_eq!(native.income_by_id.get(&profit_id), Some(&10_000));
        assert_eq!(native.income_by_id.get(&loss_id), Some(&-allowed));
        use crate::tests::cross::interview::{accepted, open_topic};
        let mut session = Interview::new(TEST_USER, d);
        let page = open_topic(&mut session, TaxSection::Rentals);
        accepted(&send(&mut session, page));
        session = restart(session.draft);
        let restored = supported(&session.draft);
        assert_eq!(
            [column(&restored, "13b", "b"), column(&restored, "13b", "c")],
            [10_000, -allowed]
        );
        assert_eq!((restored.tax, restored.refund), (result.tax, result.refund));
        let _ = share_cents(&restored);
    }
}

#[test]
fn unit13_native_k1_schedule_e_and_portfolio_parts_keep_distinct_source_owners() {
    let mut d = injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation());
    let mut partner =
        income(FederalIncome::Partnership { data: ordinary_partner_source(6_000, false) });
    partner.owner = crate::DocumentOwner::Yours;
    add(&mut d, partner);
    let partner_id = d.income.last().unwrap().id;
    let mut source = complete_ordinary_scorp(-10_000, false);
    source.interest = 300_000;
    source.dividends = 400_000;
    source.qualified_dividends = 200_000;
    source.amt = Some(amt_records(&source));
    let mut scorp = income(FederalIncome::Scorp { data: scorp_source(source) });
    scorp.owner = crate::DocumentOwner::Spouses;
    add(&mut d, scorp);
    let scorp_id = d.income.last().unwrap().id;
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 87_000);
    assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [6_000, 1_000]);
    let mut output = FederalResult::empty(d.revision);
    let native = crate::extended_rules::rentals(&d, 87_000, &mut output);
    assert_eq!(native.income_by_id.get(&partner_id), Some(&6_000));
    assert_eq!(native.income_by_id.get(&scorp_id), Some(&-6_000));
    // In the injured spouse's separate-tax projection, the qualified dividend
    // belongs to the actual shareholder, whose taxable income is4,900 here.
    // All2,000 qualifies for the separate0% bracket. Ordinary2,900 uses
    // the IRS Tax Table's2,900–2,925 row (25-wide below3,000): tax291.
    let explanation = &result
        .lines
        .iter()
        .find(|row| row.id == "form_8379_injured_share_of_tax")
        .unwrap()
        .explanation;
    assert!(
        matches!(
            &explanation.args[..],
            [
                _,
                MessageArg::Currency { minor_units: 5743, .. },
                MessageArg::Currency { minor_units: 291, .. }
            ]
        ),
        "actual separate-tax explanation: {:?}; QBI: {}",
        explanation.args,
        value(&result, "qbi_deduction")
    );
    let _ = share_cents(&result);
    use crate::tests::cross::interview::accepted;
    let mut session = Interview::new(TEST_USER, d);
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let page = open_documents(&mut session);
    accepted(&send(&mut session, page));
    session = restart(session.draft);
    let restored = supported(&session.draft);
    assert_eq!([column(&restored, "13b", "b"), column(&restored, "13b", "c")], [6_000, 1_000]);
    assert_eq!((restored.tax, restored.refund), (result.tax, result.refund));
}

#[test]
fn unit13_native_former_rental_source_does_not_count_nonpassive_income_twice() {
    let mut d = injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation());
    // Self-rental uses the existing39-year building deduction7,051.
    // 20,000 - 2,949 - 7,051 =10,000 current, less6,000 own prior loss.
    let mut source = rental(20_000, 2_949);
    source.property_type = RentalPropertyType::SelfRental;
    source.history = RentalHistory { regular: 600_000, amt: 600_000, active_in_loss_year: true };
    let mut entry = income(FederalIncome::Rental { data: source });
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut d, entry);
    let id = d.income.last().unwrap().id;
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 84_000);
    assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [0, 4_000]);
    let mut output = FederalResult::empty(d.revision);
    let native = crate::extended_rules::rentals(&d, 84_000, &mut output);
    assert_eq!(native.income_by_id.get(&id), Some(&4_000));
    assert_eq!(native.income, 4_000);
    let _ = share_cents(&result);
}

// Independent2026 QDCGT worksheet line3 and MFS TaxTable oracles. Short-only
// gain receives ordinary rates. Long gain10,000 leaves ordinary43,900 and
// only5,550 of0% room, so remaining4,450 contributes668 to table tax5,023.
#[test]
fn unit13_capital_character_keeps_short_gains_at_ordinary_rates() {
    for (short, long, tax) in
        [(10_000, 0, 6_576), (0, 10_000, 5_691), (10_000, -3_000, 5_916), (-3_000, 10_000, 5_241)]
    {
        let mut d = injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation());
        for (term, gain) in [(GainTerm::Short, short), (GainTerm::Long, long)] {
            if gain != 0 {
                add(&mut d, income(ordinary_investment_sale(gain.max(0), (-gain).max(0), term)));
            }
        }
        let result = supported(&d);
        assert_eq!(value(&result, "agi"), 80_000 + short + long);
        assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [short + long, 0]);
        let explanation = &result
            .lines
            .iter()
            .find(|row| row.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert_eq!(
            explanation.args[1],
            MessageArg::usd_dollars(tax),
            "short{short}/long{long}: {:?}",
            explanation.args
        );
        assert_eq!(explanation.args[2], MessageArg::usd_dollars(393));
        let _ = share_cents(&result);
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
        let joint = supported(&ordinary);
        assert_eq!((result.tax, result.refund), (joint.tax, joint.refund));
        let mut capital = crate::injured_spouse::CapitalSources::default();
        capital.add(Some(Owner::Taxpayer), [short * 100, long * 100]);
        let mut facts =
            liability_portfolio_native_facts(&ordinary, Some([short + long, 0]), Some(&joint));
        facts.capital = capital.owners(&ordinary, [short, long], [0; 2]);
        let totals = actual_liability_totals(&joint);
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &ordinary,
                &crate::related_forms::effective_claimants(&ordinary),
            ),
            &joint,
            None,
            &facts,
            Some(&totals),
        )
        .unwrap();
        assert_eq!(crate::injured_spouse::separate_taxes(&ordinary, &projection), Some([tax, 393]));
        use crate::tests::cross::interview::{accepted, open_topic};
        let mut session = Interview::new(TEST_USER, d);
        let page = open_topic(&mut session, TaxSection::InvestmentSales);
        accepted(&send(&mut session, page));
        session = restart(session.draft);
        let restored = supported(&session.draft);
        assert_eq!((restored.tax, restored.refund), (result.tax, result.refund));
        let explanation = &restored
            .lines
            .iter()
            .find(|row| row.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars(tax));
    }
}

// At243900 taxable income, ordinary base tax is51304. A10000 gain adds
//1500 at ordinary long rates,2500 at the25% maximum, or2800 at28%; short
//gain adds3200 at the actual ordinary bracket. These are separate-source
//controls, not a prorated joint special pool.
#[test]
fn unit13_capital_special_rates_follow_the_actual_source_owner() {
    for (collectible, unrecaptured, tax) in
        [(0, 0, 52_804), (10_000, 0, 54_104), (0, 10_000, 53_804), (4_000, 6_000, 53_924)]
    {
        let mut d = injured_return((0, 0), (60_000, 20_000), 0, allocation());
        let mut interest = savings_income(250_000, 0);
        let FederalIncome::Interest { data } = &mut interest else { unreachable!() };
        edit_interest(data).withholding = 7_000_000;
        add(&mut d, income(interest));
        let item = income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                capital_distributions: 1_000_000,
                collectible_distributions: collectible * 100,
                unrecaptured_distributions: unrecaptured * 100,
                ..Default::default()
            }),
        });
        add(&mut d, item);
        let result = supported(&d);
        assert_eq!(value(&result, "agi"), 320_000);
        assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], [260_000, 0]);
        let explanation = &result
            .lines
            .iter()
            .find(|row| row.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert_eq!(
            explanation.args[1],
            MessageArg::usd_dollars(tax),
            "{collectible}/{unrecaptured}: {:?}",
            explanation.args
        );
        assert_eq!(explanation.args[2], MessageArg::usd_dollars(5_023));
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
        let joint = supported(&ordinary);
        assert_eq!((result.tax, result.refund), (joint.tax, joint.refund));
        let mut sources = crate::injured_spouse::CapitalSources::default();
        sources.add(Some(Owner::Taxpayer), [0, 1_000_000]);
        sources.add_special(Some(Owner::Taxpayer), [collectible * 100, unrecaptured * 100]);
        let mut facts =
            liability_portfolio_native_facts(&ordinary, Some([10_000, 0]), Some(&joint));
        facts.capital = sources.owners(&ordinary, [0, 10_000], [collectible, unrecaptured]);
        let totals = actual_liability_totals(&joint);
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &ordinary,
                &crate::related_forms::effective_claimants(&ordinary),
            ),
            &joint,
            None,
            &facts,
            Some(&totals),
        )
        .unwrap();
        assert_eq!(
            crate::injured_spouse::separate_taxes(&ordinary, &projection),
            Some([tax + 2_660, 5_023])
        );
        use crate::tests::cross::interview::{accepted, open_topic};
        let mut session = Interview::new(TEST_USER, d);
        let page = open_topic(&mut session, TaxSection::InvestmentSales);
        accepted(&send(&mut session, page));
        session = restart(session.draft);
        let restored = supported(&session.draft);
        assert_eq!((restored.tax, restored.refund), (result.tax, result.refund));
        let explanation = &restored
            .lines
            .iter()
            .find(|row| row.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars(tax));
    }
}

#[test]
fn unit13_special_source_class_survives_an_opposite_owner_loss() {
    let mut d = injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation());
    let mut gain = ordinary_investment_sale(10_000, 0, GainTerm::Long);
    let FederalIncome::Sale { data } = &mut gain else { unreachable!() };
    data.form.collectible = true;
    add(&mut d, income(gain));
    let mut loss = income(ordinary_investment_sale(0, 10_000, GainTerm::Long));
    loss.owner = crate::DocumentOwner::Spouses;
    add(&mut d, loss);
    let r = supported(&d);
    assert_eq!(return_line(&r, "7a"), 0);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [10_000, -10_000]);
    let explanation =
        &r.lines.iter().find(|row| row.id == "form_8379_injured_share_of_tax").unwrap().explanation;
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(6_576));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(0));
}

#[test]
fn unit13_two_positive_capital_sources_keep_their_distinct_classes() {
    let mut d = injured_return((0, 0), (60_000, 20_000), 0, allocation());
    let mut interest = savings_income(250_000, 0);
    let FederalIncome::Interest { data } = &mut interest else { unreachable!() };
    edit_interest(data).withholding = 7_000_000;
    add(&mut d, income(interest));
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                capital_distributions: 1_000_000,
                collectible_distributions: 1_000_000,
                ..Default::default()
            }),
        }),
    );
    let mut spouse_gain = income(ordinary_investment_sale(10_000, 0, GainTerm::Long));
    spouse_gain.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse_gain);
    let r = supported(&d);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [260_000, 10_000]);
    let explanation =
        &r.lines.iter().find(|row| row.id == "form_8379_injured_share_of_tax").unwrap().explanation;
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(54_104));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(5_691));
    use crate::tests::cross::interview::{accepted, open_topic};
    let mut session = Interview::new(TEST_USER, d);
    let page = open_topic(&mut session, TaxSection::InvestmentSales);
    accepted(&send(&mut session, page));
    session = restart(session.draft);
    let restored = supported(&session.draft);
    assert_eq!((restored.tax, restored.refund), (r.tax, r.refund));
    let explanation = &restored
        .lines
        .iter()
        .find(|row| row.id == "form_8379_injured_share_of_tax")
        .unwrap()
        .explanation;
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(54_104));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(5_691));
}

#[test]
fn unit13_single_owner_elections_change_rates_without_changing_income() {
    for (qualified, capital) in [(10_000, 0), (0, 10_000)] {
        let d = unit13_election_fixture(qualified, capital);
        let result = supported(&d);
        assert_eq!(value(&result, "agi"), 90_000);
        assert_eq!(return_line(&result, "3a"), qualified * 100);
        assert_eq!(return_line(&result, "7a"), capital * 100);
        let explanation = &result
            .lines
            .iter()
            .find(|r| r.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars(6_576));
        assert_eq!(explanation.args[2], MessageArg::usd_dollars(393));
        assert!(
            !result
                .injured_spouse
                .as_ref()
                .unwrap()
                .required
                .iter()
                .any(|r| r.component.starts_with("investment_elected"))
        );
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
        let joint = supported(&ordinary);
        assert_eq!((result.tax, result.refund), (joint.tax, joint.refund));
        let mut facts =
            liability_portfolio_native_facts(&ordinary, Some([capital, 0]), Some(&joint));
        let mut capital_sources = crate::injured_spouse::CapitalSources::default();
        capital_sources.add(Some(Owner::Taxpayer), [0, capital * 100]);
        facts.capital = capital_sources.owners(&ordinary, [0, capital], [0; 2]);
        let mut investment_sources = crate::injured_spouse::InvestmentSources::default();
        investment_sources.add(Some(Owner::Taxpayer), [0, capital * 100, 0, 0, 0]);
        facts.investment_election = crate::injured_spouse::InvestmentElectionSources {
            elected: [
                value(&joint, "investment_elected_capital_gain"),
                value(&joint, "investment_elected_qualified_dividends"),
            ],
            capital: investment_sources.owners(&ordinary, [0, capital, 0, 0, 0], [0; 2], capital),
        };
        let totals = actual_liability_totals(&joint);
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &ordinary,
                &crate::related_forms::effective_claimants(&ordinary),
            ),
            &joint,
            None,
            &facts,
            Some(&totals),
        )
        .unwrap();
        assert_eq!(
            crate::injured_spouse::separate_taxes(&ordinary, &projection),
            Some([6_576, 393])
        );
        use crate::tests::cross::interview::{accepted, from_review};
        let mut session = Interview::new(TEST_USER, d);
        let Reply::Edit(page) = crate::tests::support::client::native_editor(
            from_review(&mut session, TaxSection::InjuredSpouse),
            &[],
        ) else {
            panic!()
        };
        let reply = send(&mut session, page);
        accepted(&reply);
        assert!(matches!(reply, Reply::Response(P::Review { .. })));
        let restored = supported(&restart(session.draft).draft);
        assert_eq!((restored.tax, restored.refund), (result.tax, result.refund));
    }
}

#[test]
fn unit13_capital_election_uses_only_the_native_investment_pool() {
    let mut d = unit13_election_fixture(0, 10_000);
    add(&mut d, income(personal_sale(20_000, 0, GainTerm::Long)));
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_net_capital_gain"), 10_000);
    assert_eq!(return_line(&r, "7a"), 3_000_000);
    let explanation =
        &r.lines.iter().find(|r| r.id == "form_8379_injured_share_of_tax").unwrap().explanation;
    // TI73900: ordinary53900 uses6576 table tax; the remaining personal
    // long gain20000 adds3000. It was never eligible for this election.
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(9_576));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(393));
}

fn unit13_shared_election_fixture(own: i64, total: i64) -> FederalDraft {
    let mut claim = allocation();
    claim.allocations = vec![
        InjuredAmount { component: "additional_medicare_tax".into(), injured: 54_000 },
        InjuredAmount {
            component: "investment_elected_qualified_dividends".into(),
            injured: own * 100,
        },
    ];
    let mut d = injured_return((250_000, 80_000), (60_000, 20_000), 0, claim);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut row = income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                ordinary: 1_000_000,
                qualified: 1_000_000,
                ..Default::default()
            }),
        });
        row.owner = crate::DocumentOwner::from(owner);
        add(&mut d, row);
    }
    investment_borrowing(&mut d, 1, 0, 0);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: 0, dividends: total * 100 };
    d
}

#[test]
fn unit13_shared_election_choices_conserve_joint_figures_and_survive_save() {
    let mut joint_figures = None;
    for (own, taxes) in [(10_000, [54_504, 5_691]), (0, [52_804, 6_576])] {
        let d = unit13_shared_election_fixture(own, 10_000);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 330_000);
        assert_eq!(return_line(&r, "3a"), 2_000_000);
        assert_eq!(value(&r, "investment_elected_qualified_dividends"), 10_000);
        let q = r
            .injured_spouse
            .as_ref()
            .unwrap()
            .required
            .iter()
            .find(|q| q.component == "investment_elected_qualified_dividends")
            .unwrap();
        assert_eq!(
            (q.joint, q.minimum, q.maximum, q.injured),
            (1_000_000, 0, 1_000_000, own * 100)
        );
        let explanation =
            &r.lines.iter().find(|r| r.id == "form_8379_injured_share_of_tax").unwrap().explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars(taxes[0]));
        assert_eq!(explanation.args[2], MessageArg::usd_dollars(taxes[1]));
        let figures = (r.tax, r.refund);
        assert_eq!(*joint_figures.get_or_insert(figures), figures);
        use crate::tests::cross::interview::{accepted, from_review};
        let mut session = Interview::new(TEST_USER, d);
        let Reply::Edit(page) = crate::tests::support::client::native_editor(
            from_review(&mut session, TaxSection::InjuredSpouse),
            &[],
        ) else {
            panic!()
        };
        let reply = send(&mut session, page);
        accepted(&reply);
        assert!(matches!(reply, Reply::Response(P::Review { .. })));
        let mut session = restart(session.draft);
        let Reply::Edit(page) = crate::tests::support::client::native_editor(
            from_review(&mut session, TaxSection::InjuredSpouse),
            &[],
        ) else {
            panic!()
        };
        let P::SaveInjuredSpouse { value, .. } = &page else { panic!() };
        assert_eq!(
            value
                .allocations
                .iter()
                .find(|q| q.component == "investment_elected_qualified_dividends")
                .unwrap()
                .injured,
            own * 100
        );
        let reply = send(&mut session, page);
        accepted(&reply);
        assert!(matches!(reply, Reply::Response(P::Review { .. })));
        let restored = supported(&session.draft);
        assert_eq!((restored.tax, restored.refund), figures);
    }
}

#[test]
fn unit13_shared_election_bounds_and_server_validation_use_actual_capacities() {
    use crate::tests::cross::interview::from_review;
    let d = unit13_shared_election_fixture(10_000, 15_000);
    let r = supported(&d);
    let q = r
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|q| q.component == "investment_elected_qualified_dividends")
        .unwrap();
    assert_eq!((q.minimum, q.maximum), (500_000, 1_000_000));
    for (duplicate, amount) in [(false, 499_999), (false, 1_000_001), (true, 750_000)] {
        let mut session = Interview::new(TEST_USER, d.clone());
        let Reply::Edit(mut page) = crate::tests::support::client::native_editor(
            from_review(&mut session, TaxSection::InjuredSpouse),
            &[],
        ) else {
            panic!()
        };
        let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
        let q = value
            .allocations
            .iter_mut()
            .find(|q| q.component == "investment_elected_qualified_dividends")
            .unwrap();
        q.injured = amount;
        if duplicate {
            let q = q.clone();
            value.allocations.push(q);
        }
        assert_eq!(
            crate::injured_spouse::allocation_rule(value, &session.draft, &r),
            Err("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
        );
        let before = session.draft.clone();
        // A stale amount refreshes the editor; invalid duplicate rows can
        // instead report the existing server Problem. Neither is a Save.
        match send(&mut session, page) {
            Reply::Response(P::BeginInjuredSpouse { value, .. }) => {
                let q = value
                    .new_allocations
                    .iter()
                    .find(|q| q.component == "investment_elected_qualified_dividends")
                    .unwrap();
                assert_eq!((q.minimum, q.maximum), (500_000, 1_000_000));
                assert!(value.allocations.iter().all(|row| row.component != q.component));
            }
            reply @ Reply::Response(P::Problem { .. }) => {
                crate::tests::cross::interview::rejected_with(
                    reply,
                    "INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT",
                );
            }
            _ => panic!("an invalid election allocation must not be saved"),
        }
        assert_eq!(session.draft, before);
    }
}

#[test]
fn unit13_shared_capital_election_uses_the_same_direct_source_route() {
    for (own, taxes) in [(10_000, [54_504, 5_691]), (0, [52_804, 6_576])] {
        let mut d = unit13_shared_election_fixture(own, 10_000);
        for entry in &mut d.income {
            if let FederalIncome::Dividend { data } = &mut entry.income {
                edit_dividend(data).ordinary = 0;
                edit_dividend(data).qualified = 0;
                edit_dividend(data).capital_distributions = 1_000_000;
            }
        }
        d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
            InvestmentElection::Allocated { capital: 1_000_000, dividends: 0 };
        let row = &mut d.injured_spouse.as_mut().unwrap().allocations[1];
        row.component = "investment_elected_capital_gain".into();
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_net_capital_gain"), 20_000);
        assert_eq!(return_line(&r, "3a"), 0);
        assert_eq!(return_line(&r, "7a"), 2_000_000);
        let q = r
            .injured_spouse
            .as_ref()
            .unwrap()
            .required
            .iter()
            .find(|q| q.component == "investment_elected_capital_gain")
            .unwrap();
        assert_eq!((q.minimum, q.maximum, q.injured), (0, 1_000_000, own * 100));
        let explanation =
            &r.lines.iter().find(|r| r.id == "form_8379_injured_share_of_tax").unwrap().explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars(taxes[0]));
        assert_eq!(explanation.args[2], MessageArg::usd_dollars(taxes[1]));
        use crate::tests::cross::interview::{accepted, from_review};
        let mut session = Interview::new(TEST_USER, d);
        let Reply::Edit(page) = crate::tests::support::client::native_editor(
            from_review(&mut session, TaxSection::InjuredSpouse),
            &[],
        ) else {
            panic!()
        };
        let reply = send(&mut session, page);
        accepted(&reply);
        assert!(matches!(reply, Reply::Response(P::Review { .. })));
        let restored = supported(&restart(session.draft).draft);
        assert_eq!((restored.tax, restored.refund), (r.tax, r.refund));
    }
}

#[test]
fn unit13_investment_capacities_keep_joint_cross_source_netting_once() {
    let d = injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation());
    let mut sources = crate::injured_spouse::InvestmentSources::default();
    // TP short+10000/long10000, SP short-5000/long10000. Joint net short
    //5000 leaves the actual joint long pool20000 eligible. A separately
    //rerun SP loss limit would incorrectly cap its source at5000.
    sources.add(Some(Owner::Taxpayer), [1_000_000, 1_000_000, 0, 0, 0]);
    sources.add(Some(Owner::Spouse), [-500_000, 1_000_000, 0, 0, 0]);
    assert_eq!(
        sources.owners(&d, [5_000, 20_000, 0, 0, 0], [0; 2], 20_000),
        Some([[10_000, 10_000]; 2])
    );
    // A real native stock-loss input bounds the one joint pool15000; both
    //long source capacities stay10000, so E15000 has bounds5000..10000.
    let mut limited = sources.clone();
    limited.add(Some(Owner::Taxpayer), [0, 0, 0, 0, 1_000_000]);
    let pool = crate::investment_interest::gain_pool(5_000, 20_000, 0, 10_000, 0);
    assert_eq!(pool, [15_000, 15_000]);
    assert_eq!(
        limited.owners(&d, [5_000, 20_000, 0, 0, 10_000], [0; 2], pool[1]),
        Some([[10_000, 10_000]; 2])
    );
    assert_eq!(sources.owners(&d, [5_000, 20_001, 0, 0, 0], [0; 2], 20_000), None);
}

// Actual Studio acceptance bridge: normal calculation replies and a complete
// protocol Save are replayed through the native public handler. With no
// harness environment this still exercises all three source/control flows.
#[test]
fn unit13_election_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::from_review;
    let selected = std::env::var("TAX_UNIT13_ELECTION_CASE").ok();
    for case in ["qualified", "capital", "bounded"] {
        if selected.as_deref().is_some_and(|v| v != case) {
            continue;
        }
        let bounded = case == "bounded";
        let mut d = unit13_shared_election_fixture(
            if bounded { 5_000 } else { 0 },
            if bounded { 15_000 } else { 10_000 },
        );
        let key = if case == "capital" {
            for entry in &mut d.income {
                if let FederalIncome::Dividend { data } = &mut entry.income {
                    edit_dividend(data).ordinary = 0;
                    edit_dividend(data).qualified = 0;
                    edit_dividend(data).capital_distributions = 1_000_000;
                }
            }
            d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
                InvestmentElection::Allocated { capital: 1_000_000, dividends: 0 };
            d.injured_spouse.as_mut().unwrap().allocations[1].component =
                "investment_elected_capital_gain".into();
            "investment_elected_capital_gain"
        } else {
            "investment_elected_qualified_dividends"
        };
        let expected = if bounded { [54_079, 6_401] } else { [54_504, 5_691] };
        let chosen = if bounded { 750_000 } else { 1_000_000 };
        let initial = supported(&d);
        let mut session = Interview::new(TEST_USER, d);
        crate::tests::support::client::admit_native_fixture_facts(&mut session);
        let source_page = open_documents(&mut session);
        accepted(&send(&mut session, source_page));
        // Confirm the retained borrowing answers in their actual editor after
        // the source inventory changes, before saving the allocation.
        let Reply::Edit(other @ P::SaveOtherItemized { .. }) =
            from_review(&mut session, TaxSection::OtherItemized)
        else {
            panic!("expected the investment-interest editor")
        };
        let mut reply = send(&mut session, other);
        // Continue the completed sections until the actual allocation editor
        // opens; that question is the next answer, before Review.
        while let Reply::Response(hub @ P::Hub { .. }) = &reply {
            let request = crate::tests::cross::interview::hub_action(hub);
            reply = send(&mut session, request);
        }
        // The walk has nothing left (the empty itemized hub is passed), so the
        // answered allocation reopens from the review.
        let reply = if matches!(reply, Reply::Response(P::Review { .. })) {
            from_review(&mut session, TaxSection::InjuredSpouse)
        } else {
            reply
        };
        let reply = crate::tests::support::client::native_editor(reply, &[]);
        let Reply::Edit(page @ P::SaveInjuredSpouse { .. }) = reply else {
            panic!("expected the injured-spouse allocation editor: {reply:?}")
        };
        let native = NativeInterviewCase("TAX_UNIT13_ELECTION");
        if native.required_checked_calculation(&mut session) {
            return;
        }
        if native.seed(&page) {
            return;
        }
        let mut request = if let Some(request) = native.checked_input("PAYLOAD") {
            request
        } else {
            let mut page = page;
            let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
            value.allocations.iter_mut().find(|r| r.component == key).unwrap().injured = chosen;
            page
        };
        let P::SaveInjuredSpouse { value, .. } = &request else { panic!() };
        assert_eq!(value.allocations.iter().find(|r| r.component == key).unwrap().injured, chosen);
        let reply = send(&mut session, request.clone());
        assert!(matches!(reply, Reply::Response(P::Review { .. })), "{reply:?}");
        let result = supported(&session.draft);
        assert_eq!((result.tax, result.refund), (initial.tax, initial.refund));
        let explanation = &result
            .lines
            .iter()
            .find(|r| r.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars(expected[0]));
        assert_eq!(explanation.args[2], MessageArg::usd_dollars(expected[1]));
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let decoded = draft_from_native_records(tax_year, &stored);
        assert_eq!(native_draft_records(&decoded), stored);
        assert_eq!(decoded, session.draft);
        let mut reopened = restart(session.draft);
        let Reply::Edit(shown) = crate::tests::support::client::native_editor(
            from_review(&mut reopened, TaxSection::InjuredSpouse),
            &[],
        ) else {
            panic!()
        };
        let P::SaveInjuredSpouse { value, .. } = &shown else { panic!() };
        assert_eq!(value.allocations.iter().find(|r| r.component == key).unwrap().injured, chosen);
        native.output(&shown, &stored);
        assert!(matches!(send(&mut reopened, shown), Reply::Response(P::Review { .. })));
        // A forged server request cannot escape the native regenerated bounds.
        let P::SaveInjuredSpouse { value, .. } = &mut request else { panic!() };
        value.allocations.iter_mut().find(|r| r.component == key).unwrap().injured = 1_000_001;
        let before = reopened.draft.clone();
        let source_case = match case {
            "qualified" => "secondary-admission-1008",
            "capital" => "secondary-admission-1030",
            "bounded" => "secondary-admission-1047",
            _ => unreachable!(),
        };
        assert!(!matches!(
            current_source_refusal(
                source_case,
                &mut reopened,
                request,
                "PROBLEM_ACCEPTED_ANSWERS_CHANGED",
                None,
                SourceScope::Fieldless
            ),
            Reply::Response(P::Review { .. })
        ));
        assert_eq!(reopened.draft, before);
    }
}

fn unit13_rounding_fixture(injured: Owner) -> FederalDraft {
    let mut claim = allocation();
    claim.injured = injured;
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, claim);
    d.income = vec![
        wage(1, Owner::Taxpayer, 6_000_040, 1_000_040),
        wage(2, Owner::Spouse, 2_000_040, 500_040),
    ];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut interest = savings_income(500, 0);
        let FederalIncome::Interest { data } = &mut interest else { unreachable!() };
        edit_interest(data).taxable += 40;
        let mut entry = income(interest);
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
        let mut entry = income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                ordinary: 100_040,
                qualified: 100_040,
                ..Default::default()
            }),
        });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
        let mut sale = personal_sale(1_000, 0, GainTerm::Long);
        let FederalIncome::Sale { data } = &mut sale else { unreachable!() };
        data.form.proceeds += 40;
        let mut entry = income(sale);
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    d
}
fn unit13_assert_rounding(r: &FederalResult, injured: Owner) {
    let injured_wage = if injured == Owner::Spouse { 20_000 } else { 60_000 };
    let injured_withholding = if injured == Owner::Spouse { 5_000 } else { 10_000 };
    // Approved R3 correction,2026-10-14: each native category owns its
    // rounding. Selected interest500.40->500, dividends1000.40->1000,
    // capital1000.40->1000 total2500; joint categories total5003.
    // The earlier combined selected amount2501 and its failure witnesses
    // remain in the dated R2 record and Git history. Both tests below use
    // this corrected helper; no ordinary joint figure changes.
    for (line, joint, injured_amount) in
        [("13a", 80_001, injured_wage), ("13b", 5_003, 2_500), ("19", 15_001, injured_withholding)]
    {
        assert_eq!(column(r, line, "a"), joint);
        assert_eq!(column(r, line, "b"), injured_amount, "{injured:?} line{line}");
        assert_eq!(column(r, line, "c"), joint - injured_amount);
    }
}
// Original R2: round the selected injured source first, then conserve the joint row.
#[test]
fn unit13_selected_injured_column_rounds_before_the_other_residual() {
    for injured in [Owner::Spouse, Owner::Taxpayer] {
        let d = unit13_rounding_fixture(injured);
        let result = supported(&d);
        unit13_assert_rounding(&result, injured);
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let plain = supported(&ordinary);
        assert_eq!((result.tax, result.refund), (plain.tax, plain.refund));
    }
}
#[test]
fn unit13_selected_injured_rounding_preserves_signed_joint_capital_loss() {
    for injured in [Owner::Spouse, Owner::Taxpayer] {
        let mut claim = allocation();
        claim.injured = injured;
        let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, claim);
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut sale = personal_sale(0, 1_000, GainTerm::Long);
            let FederalIncome::Sale { data } = &mut sale else { unreachable!() };
            data.form.basis = Some(100_040);
            data.form.use_kind = SaleUse::Investment { wash_sale: 0 };
            let mut entry = income(sale);
            entry.owner = crate::DocumentOwner::from(owner);
            add(&mut d, entry);
        }
        let result = supported(&d);
        assert_eq!(
            (column(&result, "13b", "a"), column(&result, "13b", "b"), column(&result, "13b", "c")),
            (-2_001, -1_000, -1_001)
        );
        let mut ordinary = d;
        ordinary.injured_spouse = None;
        let plain = supported(&ordinary);
        assert_eq!((result.tax, result.refund), (plain.tax, plain.refund));
    }
}
#[test]
fn unit13_rounding_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::from_review;
    let mut d = unit13_rounding_fixture(Owner::Taxpayer);
    let initial = supported(&d);
    name_accepted_income_documents(&mut d);
    let mut session = Interview::new(TEST_USER, d);
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let source_page = open_documents(&mut session);
    accepted(&send(&mut session, source_page));
    let Reply::Edit(page) = crate::tests::support::client::native_editor(
        from_review(&mut session, TaxSection::InjuredSpouse),
        &[],
    ) else {
        panic!()
    };
    let native = NativeInterviewCase("TAX_UNIT13_ROUNDING");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if native.seed(&page) {
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        let mut request = page;
        let P::SaveInjuredSpouse { value, .. } = &mut request else { panic!() };
        value.injured = Owner::Spouse;
        request
    };
    let P::SaveInjuredSpouse { value, .. } = &request else { panic!() };
    assert_eq!(value.injured, Owner::Spouse);
    assert!(matches!(send(&mut session, request), Reply::Response(P::Review { .. })));
    let result = supported(&session.draft);
    unit13_assert_rounding(&result, Owner::Spouse);
    assert_eq!((result.tax, result.refund), (initial.tax, initial.refund));
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let decoded = draft_from_native_records(tax_year, &stored);
    assert_eq!(native_draft_records(&decoded), stored);
    assert_eq!(decoded, session.draft);
    let mut reopened = restart(session.draft);
    let Reply::Edit(shown) = crate::tests::support::client::native_editor(
        from_review(&mut reopened, TaxSection::InjuredSpouse),
        &[],
    ) else {
        panic!()
    };
    let P::SaveInjuredSpouse { value, .. } = &shown else { panic!() };
    assert_eq!(value.injured, Owner::Spouse);
    unit13_assert_rounding(&supported(&reopened.draft), Owner::Spouse);
    native.output(&shown, &stored);
    assert!(matches!(send(&mut reopened, shown), Reply::Response(P::Review { .. })));
}

fn unit13_qsbs_source_fixture(
    first_date: u32,
    second_date: u32,
    third: bool,
    cents: bool,
) -> FederalDraft {
    let mut d = injured_return((0, 0), (60_000, 20_000), 0, allocation());
    let mut interest = savings_income(250_000, 0);
    let FederalIncome::Interest { data } = &mut interest else { unreachable!() };
    edit_interest(data).withholding = 7_000_000;
    add(&mut d, income(interest));
    let mut lots = vec![
        (Owner::Taxpayer, first_date, if cents { 98 } else { 1_000_000 }),
        (Owner::Spouse, second_date, if cents { 98 } else { 1_000_000 }),
    ];
    if third {
        lots.push((Owner::Spouse, 20100927, 800_000));
    }
    for (owner, issued, gain) in lots {
        let data =
            qsb_source_sale("Synthetic shared issuer", issued, gain, BasisReporting::ReportedToIrs);
        let mut entry = income(FederalIncome::QualifiedSmallBusinessSale { data });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    d
}
#[test]
fn unit13_qsbs_native_cohorts_feed_actual_owner_liabilities_once() {
    for (first_date, second_date, third, expected_income, expected_tax, expected_niit) in [
        (20080602, 20150302, false, [255_000, 0], [52_704, 5_023], [2_470, 0]),
        (20080602, 20150302, true, [255_000, 2_000], [52_704, 5_263], [2_526, 20]),
        (20150302, 20080602, false, [250_000, 5_000], [51_304, 5_623], [2_422, 48]),
    ] {
        let mut d = unit13_qsbs_source_fixture(first_date, second_date, third, false);
        let joint = {
            let mut d = d.clone();
            d.injured_spouse = None;
            supported(&d)
        };
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            d.injured_spouse.as_mut().unwrap().injured = injured;
            let result = supported(&d);
            let i = usize::from(injured == Owner::Spouse);
            assert_eq!(
                (column(&result, "13b", "b"), column(&result, "13b", "c")),
                (expected_income[i], expected_income[1 - i])
            );
            assert_eq!(
                (column(&result, "18", "b"), column(&result, "18", "c")),
                (expected_niit[i], expected_niit[1 - i])
            );
            let explanation = &result
                .lines
                .iter()
                .find(|r| r.id == "form_8379_injured_share_of_tax")
                .unwrap()
                .explanation;
            assert_eq!(explanation.args[1], MessageArg::usd_dollars(expected_tax[i]));
            assert_eq!(explanation.args[2], MessageArg::usd_dollars(expected_tax[1 - i]));
            assert_eq!((result.tax, result.refund), (joint.tax, joint.refund));
        }
        // The genuine no-claim API gets actual native QSBS source cents;
        // no synthetic claim or reconstructed issuer-wide source pool.
        let mut ordinary = d;
        ordinary.injured_spouse = None;
        let native =
            crate::qsbs_return::calculate(&ordinary, &mut FederalResult::empty(ordinary.revision));
        let mut sources = crate::injured_spouse::CapitalSources::default();
        for source in &native.sources {
            sources.add(Some(source.owner), [0, source.taxable_gain]);
            sources.add_special(Some(source.owner), [source.rate_28_gain.unwrap(), 0]);
        }
        let mut facts = liability_portfolio_native_facts(&ordinary, None, Some(&joint));
        facts.capital = sources.owners(
            &ordinary,
            [0, native.taxable_gain / 100],
            [native.rate_28_gain / 100, 0],
        );
        let totals = actual_liability_totals(&joint);
        let projection = crate::injured_spouse::owners::project_sources(
            crate::related_forms::RelatedFormContext::original(
                &ordinary,
                &crate::related_forms::effective_claimants(&ordinary),
            ),
            &joint,
            None,
            &facts,
            Some(&totals),
        )
        .unwrap();
        assert_eq!(
            crate::injured_spouse::separate_taxes(&ordinary, &projection),
            Some([expected_tax[0] + expected_niit[0], expected_tax[1] + expected_niit[1]])
        );
    }
}
#[test]
fn unit13_qsbs_source_cents_use_the_selected_injured_rounding_view() {
    let mut d = unit13_qsbs_source_fixture(20080602, 20080602, false, true);
    let joint = {
        let mut d = d.clone();
        d.injured_spouse = None;
        supported(&d)
    };
    // Each actual native taxable/28% source is49c. The one joint pool is98c,
    // rounded1. Select either source first; its49c rounds0, other gets1.
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let result = supported(&d);
        let own = if injured == Owner::Taxpayer { 250_000 } else { 0 };
        assert_eq!(column(&result, "13b", "b"), own);
        assert_eq!(column(&result, "13b", "c"), 250_001 - own);
        assert_eq!((result.tax, result.refund), (joint.tax, joint.refund));
    }
}
#[test]
fn unit13_qsbs_owner_source_survives_save_reopen_and_date_correction() {
    use crate::tests::cross::interview::accepted;
    let mut session =
        Interview::new(TEST_USER, unit13_qsbs_source_fixture(20080602, 20150302, true, false));
    let page = open_documents(&mut session);
    accepted(&send(&mut session, page));
    session = restart(session.draft);
    let before = supported(&session.draft);
    assert_eq!((column(&before, "13b", "b"), column(&before, "13b", "c")), (255_000, 2_000));
    let identities: Vec<_> = session.draft.income.iter().map(|v| (v.id, v.owner)).collect();
    let mut page = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { panic!() };
    let row = documents.common.investment_sales.documents
        .iter_mut()
        .find(|v| {
            v.owner == crate::DocumentOwner::Spouses
                && matches!(&v.document,crate::InvestmentSaleDocument::QsbsSale(d) if d.lots[0].issued == 20150302)
        })
        .unwrap();
    let corrected_id = row.id;
    let crate::InvestmentSaleDocument::QsbsSale(data) = &mut row.document else { panic!() };
    data.lots[0].issued = 20100927;
    accepted(&send(&mut session, page));
    let FederalIncome::QualifiedSmallBusinessSale { data } =
        &session.draft.income.iter().find(|entry| entry.id == corrected_id).unwrap().income
    else {
        panic!()
    };
    assert_eq!(
        (data.issuer_qualified, data.active_business, data.no_disqualification),
        (None, None, None)
    );
    for field in ["qsbs_issuer_qualified", "qsbs_active_business", "qsbs_no_disqualification"] {
        qsbs_bool_fact(&mut session, &alloc::format!("income/{corrected_id}/{field}"), true);
    }
    session = restart(session.draft);
    let after = supported(&session.draft);
    assert_eq!((column(&after, "13b", "b"), column(&after, "13b", "c")), (255_000, 4_500));
    let explanation =
        &after.lines.iter().find(|r| r.id == "form_8379_injured_share_of_tax").unwrap().explanation;
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(52_704));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(5_563));
    assert_eq!(
        session.draft.income.iter().map(|v| (v.id, v.owner)).collect::<Vec<_>>(),
        identities
    );
    let mut ordinary = session.draft.clone();
    ordinary.injured_spouse = None;
    let plain = supported(&ordinary);
    assert_eq!((after.tax, after.refund), (plain.tax, plain.refund));
    let page = open_documents(&mut session);
    accepted(&send(&mut session, page));
    let reopened = supported(&restart(session.draft).draft);
    assert_eq!((after.tax, after.refund), (reopened.tax, reopened.refund));
}

#[test]
fn unit13_qsbs_actual_client_source_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::accepted;
    let d = unit13_qsbs_source_fixture(20080602, 20150302, true, false);
    let mut session = Interview::new(TEST_USER, d);
    let corrected_id = session
        .draft
        .income
        .iter()
        .find(|entry| {
            matches!(&entry.income,
        FederalIncome::QualifiedSmallBusinessSale { data } if data.lots[0].issued == 20150302)
        })
        .unwrap()
        .id;
    let page = open_documents(&mut session);
    let native = NativeInterviewCase("TAX_UNIT13_QSBS");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if native.seed(&page) {
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        let mut page = page;
        let P::SaveDocuments { documents, .. } = &mut page else { panic!() };
        let row=documents.common.investment_sales.documents.iter_mut().find(|v|v.owner==crate::DocumentOwner::Spouses &&
            matches!(&v.document,crate::InvestmentSaleDocument::QsbsSale(d) if d.lots[0].issued==20150302)).unwrap();
        let crate::InvestmentSaleDocument::QsbsSale(d) = &mut row.document else { panic!() };
        d.lots[0].issued = 20100927;
        page
    };
    let P::SaveDocuments { documents, .. } = &request else { panic!() };
    assert!(
        !documents.common.investment_sales.documents.iter().any(
            |v| matches!(&v.document,crate::InvestmentSaleDocument::QsbsSale(d) if d.lots[0].issued==20150302)
        )
    );
    accepted(&send(&mut session, request));
    let FederalIncome::QualifiedSmallBusinessSale { data } =
        &session.draft.income.iter().find(|entry| entry.id == corrected_id).unwrap().income
    else {
        panic!()
    };
    assert_eq!(
        (data.issuer_qualified, data.active_business, data.no_disqualification),
        (None, None, None)
    );
    for field in ["qsbs_issuer_qualified", "qsbs_active_business", "qsbs_no_disqualification"] {
        qsbs_bool_fact(&mut session, &alloc::format!("income/{corrected_id}/{field}"), true);
    }
    let result = supported(&session.draft);
    assert_eq!((column(&result, "13b", "b"), column(&result, "13b", "c")), (255_000, 4_500));
    let explanation = &result
        .lines
        .iter()
        .find(|r| r.id == "form_8379_injured_share_of_tax")
        .unwrap()
        .explanation;
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(52_704));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(5_563));
    let mut ordinary = session.draft.clone();
    ordinary.injured_spouse = None;
    let plain = supported(&ordinary);
    assert_eq!((result.tax, result.refund), (plain.tax, plain.refund));
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let decoded = draft_from_native_records(tax_year, &stored);
    assert_eq!(native_draft_records(&decoded), stored);
    assert_eq!(decoded, session.draft);
    let mut reopened = restart(session.draft);
    let shown = open_documents(&mut reopened);
    native.output(&shown, &stored);
    accepted(&send(&mut reopened, shown));
    let after = supported(&reopened.draft);
    assert_eq!((result.tax, result.refund), (after.tax, after.refund));
    assert_eq!((column(&after, "13b", "b"), column(&after, "13b", "c")), (255_000, 4_500));
}

// Owner taxes below are the native per-beneficiary Form5329 results, not a
// fresh percentage of the joint excess or a split by wages/contributors.
fn unit13_assert_account_tax_owners(mut d: FederalDraft, tax: [i64; 2]) {
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    let joint = supported(&ordinary);
    assert_eq!(return_line(&joint, "23"), tax.iter().sum::<i64>() * 100);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!((column(&r, "18", "b"), column(&r, "18", "c")), (tax[i], tax[1 - i]));
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
        assert!(r.lines.iter().any(|v| v.id == "form_8379_injured_share_of_tax"));
    }
}

fn unit13_ira_tax_fixture() -> FederalDraft {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    let mut facts = ira_facts(750000, 0, Answer::No);
    facts.taxpayer.excess = Some(IraExcess {
        traditional_value: Some(4000000),
        roth_value: 0,
        prior_traditional: 200000,
        prior_roth: 0,
        returned_prior_excess: 0,
    });
    let mut spouse = ira_facts(0, 750000, Answer::No).taxpayer;
    spouse.excess = Some(IraExcess {
        traditional_value: Some(0),
        roth_value: 4000000,
        prior_traditional: 0,
        prior_roth: 100000,
        returned_prior_excess: 0,
    });
    facts.spouse = IraSpouse::Joint { facts: spouse };
    d.adjustments.as_mut().unwrap().iras = facts;
    d
}

#[test]
fn unit13_ira_excess_tax_retains_traditional_and_roth_owners() {
    unit13_assert_account_tax_owners(unit13_ira_tax_fixture(), [120, 60]);
}

#[test]
fn unit13_ira_excess_tax_keeps_each_native_account_value_cap_and_rounding() {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    let mut facts = ira_facts(750000, 0, Answer::No);
    facts.taxpayer.excess = Some(IraExcess {
        traditional_value: Some(949),
        roth_value: 0,
        prior_traditional: 100000,
        prior_roth: 0,
        returned_prior_excess: 0,
    });
    facts.spouse = IraSpouse::Joint { facts: facts.taxpayer.clone() };
    d.adjustments.as_mut().unwrap().iras = facts;
    // Each native value is $9; each native6% tax rounds to $1. The joint
    // tax is $2, even though6% of the combined $19 value would round to $1.
    unit13_assert_account_tax_owners(d, [1, 1]);
}

fn unit13_coverdell_tax_fixture() -> FederalDraft {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    for (beneficiary, contributor, excess) in
        [(Owner::Taxpayer, Owner::Spouse, 100000), (Owner::Spouse, Owner::Taxpayer, 200000)]
    {
        d.coverdell_excess.push(CoverdellExcess {
            beneficiary,
            contributors: vec![CoverdellContributor {
                label: "Actual opposite-owner contributor".into(),
                person: CoverdellContributorKind::Filer { owner: contributor },
                contributions: vec![CoverdellContribution {
                    account: "Actual ESA".into(),
                    paid: 20261201,
                    amount: 200000 + excess,
                }],
            }],
            corrections: vec![],
            prior_excess: 0,
            prior_tax: 0,
            distributions: 0,
            value: 1000000,
            special_needs: true,
        });
    }
    d
}

#[test]
fn unit13_coverdell_excess_tax_follows_beneficiary_not_contributor() {
    unit13_assert_account_tax_owners(unit13_coverdell_tax_fixture(), [60, 120]);
}

fn unit13_trump_tax_fixture() -> FederalDraft {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100301;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 20100401;
    for (owner, contributions, returned, earnings) in
        [(Owner::Taxpayer, 800000, 50000, 2049), (Owner::Spouse, 700000, 100000, 3050)]
    {
        d.screening.as_mut().unwrap().trump_accounts.push(TrumpAccount {
            owner,
            contributions,
            exempt: 100000,
            returns: vec![TrumpExcessReturn {
                document_id: None,
                principal: returned,
                earnings,
                received: 20261201,
                deadline: 20270415,
                confirmed: true,
            }],
            confirmed: true,
        });
    }
    // Taxpayer:6% of1500 +20; spouse: no remaining excess +31. The
    // returned earnings incur their native100% tax, not ordinary income.
    d
}

#[test]
fn unit13_trump_account_excess_and_returned_earnings_taxes_keep_beneficiary() {
    unit13_assert_account_tax_owners(unit13_trump_tax_fixture(), [110, 31]);
}

#[test]
fn unit13_additional_account_tax_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{PAGE_REVIEW, accepted, go_to};
    fn open(session: &mut Interview, topic: TaxSection) -> P {
        let request = go_to(&session.draft, PAGE_REVIEW, topic);
        let Reply::Edit(page) = send(session, request) else { panic!() };
        page
    }
    let profile = std::env::var("TAX_UNIT13_ACCOUNT_TAX_PROFILE").unwrap_or_else(|_| "ira".into());
    let (draft, topic, expected) = match profile.as_str() {
        "ira" => (unit13_ira_tax_fixture(), TaxSection::IraContributions, [60, 60]),
        "coverdell" => (unit13_coverdell_tax_fixture(), TaxSection::OtherTaxes, [1, 120]),
        "trump" => (unit13_trump_tax_fixture(), TaxSection::OtherTaxes, [110, 41]),
        "household" => (unit13_household_tax_fixture(), TaxSection::OtherTaxes, [557, 0]),
        _ => panic!("unknown account profile"),
    };
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open(&mut session, topic);
    let native = NativeInterviewCase("TAX_UNIT13_ACCOUNT_TAX");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if native.seed(&page) {
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        match &mut page {
            P::SaveIraContributions { value, .. } => {
                value
                    .people
                    .iter_mut()
                    .find(|v| v.owner == Owner::Taxpayer)
                    .unwrap()
                    .more
                    .excess
                    .as_mut()
                    .unwrap()
                    .prior_traditional = 100000;
            }
            P::SaveOtherTaxes { value, .. } => {
                for tax in &mut value.taxes {
                    match tax {
                        OtherTax::CoverdellExcess { beneficiaries } => {
                            beneficiaries
                                .iter_mut()
                                .find(|v| v.beneficiary == Owner::Taxpayer)
                                .unwrap()
                                .value = 949;
                        }
                        OtherTax::TrumpAccounts { accounts } => {
                            accounts
                                .iter_mut()
                                .find(|v| v.owner == Owner::Spouse)
                                .unwrap()
                                .returns[0]
                                .earnings = 4050;
                        }
                        OtherTax::HouseholdEmployees { employers } => {
                            employers[0].owner = Owner::Taxpayer;
                        }
                        _ => {}
                    }
                }
            }
            _ => panic!(),
        }
        page
    };
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(
        (column(&result, "18", "b"), column(&result, "18", "c")),
        (expected[0], expected[1])
    );
    unit13_assert_account_tax_owners(session.draft.clone(), expected);
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let draft = draft_from_native_records(tax_year, &stored);
    assert_eq!(draft, session.draft);
    let mut reopened = restart(draft);
    let shown = open(&mut reopened, topic);
    native.output(&shown, &stored);
    let accepted_draft = reopened.draft.clone();
    accepted(&send(&mut reopened, shown));
    assert_eq!(reopened.draft, accepted_draft);
    let after = supported(&reopened.draft);
    assert_eq!((after.tax, after.refund), (result.tax, result.refund));
    assert_eq!((column(&after, "18", "b"), column(&after, "18", "c")), (expected[0], expected[1]));
}

fn unit13_household_tax_fixture() -> FederalDraft {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    let mut employer = household_employer(3500);
    employer.owner = Owner::Spouse;
    d.screening.as_mut().unwrap().household_employment.push(employer);
    d
}

#[test]
fn unit13_household_native_tax_uses_employer_identity_not_row_or_wage_share() {
    let mut d = unit13_household_tax_fixture();
    let records = &mut d.screening.as_mut().unwrap().household_employment;
    records[0].workers[0].withholding = 10049;
    let mut other = household_employer(5000);
    other.workers[0].withholding = 10050;
    records.push(other);
    // Spouse:434 Social Security +102 Medicare +21 FUTA +100 withheld =657.
    // Taxpayer:620 +145 +30 +101 =896. Employee withholding is owed
    // through ScheduleH; it is not the employers' personal withholding.
    for reverse in [false, true] {
        if reverse {
            d.screening.as_mut().unwrap().household_employment.reverse();
        }
        let r = supported(&d);
        assert_eq!(value(&r, "household_0_tax"), 896);
        assert_eq!(value(&r, "household_1_tax"), 657);
        assert_eq!((column(&r, "19", "b"), column(&r, "19", "c")), (10000, 5000));
        unit13_assert_account_tax_owners(d.clone(), [896, 657]);
    }
}

#[test]
fn unit13_household_inapplicable_payroll_taxes_remain_zero_without_a_split() {
    let mut d = unit13_household_tax_fixture();
    let mut employer = household_employer(999);
    employer.owner = Owner::Spouse;
    d.screening.as_mut().unwrap().household_employment = vec![employer];
    unit13_assert_account_tax_owners(d, [0, 0]);
}

#[test]
fn unit13_household_before_business_projection_requires_actual_applicable_sources() {
    let mut d = unit13_household_tax_fixture();
    d.screening.as_mut().unwrap().household_employment.push(household_employer(5000));
    d.injured_spouse = None;
    d.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
    let r = supported(&d);
    let facts = liability_portfolio_native_facts(&d, Some([0; 2]), Some(&r));
    let mut totals = actual_liability_totals(&r);
    // An applicable component absent from the supplied source map remains
    // unknown. It must not silently become zero or an invented claim.
    let incomplete = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &r,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &incomplete), None);
    for id in ["household_0_tax", "household_1_tax"] {
        totals.calculation_amounts.insert(id, value(&r, id));
    }
    let complete = crate::injured_spouse::owners::project_sources(
        crate::related_forms::RelatedFormContext::original(
            &d,
            &crate::related_forms::effective_claimants(&d),
        ),
        &r,
        None,
        &facts,
        Some(&totals),
    )
    .unwrap();
    // Native MFS income taxes5023/393 plus actual employer taxes795/557.
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &complete), Some([5818, 950]));
}

fn unit13_education_source_fixture() -> FederalDraft {
    use crate::tests::family::education_accounts::{
        entered, expense_record, q1099_form, qtp, student_row, withdrawal,
    };
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    add(
        &mut d,
        q1099_form(
            Owner::Spouse,
            4000,
            qtp(1000, 3000),
            withdrawal(),
            FederalEducationBeneficiary::Recipient,
        ),
    );
    add(
        &mut d,
        expense_record(vec![student_row(-1, entered(0, 0, 0)), student_row(-2, entered(0, 0, 0))]),
    );
    d
}

#[test]
fn unit13_final_education_recipient_feeds_income_and_account_tax() {
    let mut d = unit13_education_source_fixture();
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    let joint = supported(&ordinary);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        let first = usize::from(injured == Owner::Spouse);
        let income = [0, 1000];
        let account_tax = [0, 100];
        assert_eq!(
            (column(&r, "13b", "b"), column(&r, "13b", "c")),
            (income[first], income[1 - first])
        );
        assert_eq!(
            (column(&r, "18", "b"), column(&r, "18", "c")),
            (account_tax[first], account_tax[1 - first])
        );
        let explanation =
            &r.lines.iter().find(|v| v.id == "form_8379_injured_share_of_tax").unwrap().explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars([5023, 493][first]));
        assert_eq!(explanation.args[2], MessageArg::usd_dollars([5023, 493][1 - first]));
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
    }
}

fn unit13_medical_source(
    owner: Owner,
    account: MedicalSaAccount,
    cents: i64,
) -> FederalIncomeEntry {
    let mut entry = income(FederalIncome::MedicalDistribution {
        data: crate::SaSource {
            medicare_prior: if matches!(&account, MedicalSaAccount::MedicareAdvantage) {
                Some(MedicareMsaPrior::None)
            } else {
                None
            },
            medicare_from_spouse: if matches!(&account, MedicalSaAccount::MedicareAdvantage) {
                Some(false)
            } else {
                None
            },
            payment: Some(MedicalSaPayment::Own {
                nonmedical: vec![MedicalAccountWithdrawal { date: 20260601, amount: cents }],
                medical: 0,
                rollover: None,
                disabled_since: None,
            }),
            ..sa_source(Sa1099 {
                origin: HealthDocumentOrigin::Form1099Sa,
                gross: cents,
                box2_earnings: None,
                box3_code: Some(1),
                box4_value: None,
                account: account.clone(),
                ..Default::default()
            })
        },
    });
    entry.owner = crate::DocumentOwner::from(owner);
    entry
}

#[test]
fn unit13_medical_sources_preserve_each_native_income_and_tax_rounding_boundary() {
    for (
        first_account,
        second_account,
        cents,
        native_income,
        native_tax,
        selected_income,
        selected_tax,
    ) in [
        (MedicalSaAccount::Hsa, MedicalSaAccount::Archer, [249, 49], 2, 0, [2, 0], [0, 0]),
        (MedicalSaAccount::Hsa, MedicalSaAccount::Hsa, [49, 49], 0, 0, [0, 0], [0, 0]),
        (MedicalSaAccount::Archer, MedicalSaAccount::Archer, [249, 249], 5, 1, [2, 2], [0, 0]),
        (MedicalSaAccount::Archer, MedicalSaAccount::Archer, [250, 250], 5, 1, [3, 3], [1, 1]),
        (
            MedicalSaAccount::MedicareAdvantage,
            MedicalSaAccount::MedicareAdvantage,
            [249, 249],
            4,
            2,
            [2, 2],
            [1, 1],
        ),
    ] {
        let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
        add(&mut d, unit13_medical_source(Owner::Taxpayer, first_account, cents[0]));
        add(&mut d, unit13_medical_source(Owner::Spouse, second_account, cents[1]));
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let joint = supported(&ordinary);
        assert_eq!(value(&joint, "agi"), 80000 + native_income);
        assert_eq!(return_line(&joint, "23"), native_tax * 100);
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let i = usize::from(injured == Owner::Spouse);
            d.injured_spouse.as_mut().unwrap().injured = injured;
            let r = supported(&d);
            assert_eq!(
                (column(&r, "13b", "b"), column(&r, "13b", "c")),
                (selected_income[i], native_income - selected_income[i])
            );
            assert_eq!(
                (column(&r, "18", "b"), column(&r, "18", "c")),
                (selected_tax[i], native_tax - selected_tax[i])
            );
            assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
            assert!(r.lines.iter().any(|v| v.id == "form_8379_injured_share_of_tax"));
        }
    }
}

#[test]
fn unit13_hsa_correction_components_keep_actual_owners_before_joint_rounding() {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    for (owner, earnings) in [(Owner::Taxpayer, 49), (Owner::Spouse, 51)] {
        let mut h = hsa_entry(owner, HsaCoverage::SelfOnly, 500000);
        h.corrections = vec![hsa_correction(2026, 20261231, 60000, earnings)];
        d.adjustments.as_mut().unwrap().hsa.push(h);
    }
    let mut ordinary = d.clone();
    ordinary.injured_spouse = None;
    let joint = supported(&ordinary);
    assert_eq!(value(&joint, "hsa_correction_earnings"), 1);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!(column(&r, "13b", "b"), [0, 1][i]);
        assert_eq!((column(&r, "14", "b"), column(&r, "14", "c")), (4400, 4400));
        assert_eq!(column(&r, "18", "b"), 0);
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
        let explanation =
            &r.lines.iter().find(|v| v.id == "form_8379_injured_share_of_tax").unwrap().explanation;
        assert_eq!(explanation.args[1], MessageArg::usd_dollars([4495, 0][i]));
    }
}

#[test]
fn unit13_hsa_testing_recapture_and_tax_keep_the_native_owner() {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    for (owner, amount) in [(Owner::Taxpayer, 600000), (Owner::Spouse, 300000)] {
        let mut h = hsa_entry(owner, HsaCoverage::None, 0);
        h.funding_history = vec![hsa_funding(20251231, 20260615, amount)];
        d.adjustments.as_mut().unwrap().hsa.push(h);
    }
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!(
            (column(&r, "13b", "b"), column(&r, "13b", "c")),
            ([6000, 3000][i], [6000, 3000][1 - i])
        );
        assert_eq!(
            (column(&r, "18", "b"), column(&r, "18", "c")),
            ([600, 300][i], [600, 300][1 - i])
        );
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let joint = supported(&ordinary);
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
    }
}

#[test]
fn unit13_unlinked_archer_correction_has_native_owner_without_an_income_id() {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    let mut owners = vec![];
    for (owner, earnings) in [(Owner::Taxpayer, 49), (Owner::Spouse, 51)] {
        let mut v = archer_contributor(owner, ArcherCoverage::None, 3000);
        v.corrections.push(ArcherCorrection {
            failed_deposit: None,
            document: None,
            year: 2026,
            contributed_on: 20260601,
            principal: 300000,
            earnings,
            returned: 20261231,
            deadline: 20270415,
            payer: ArcherCorrectionPayer::Personal,
        });
        owners.push(v);
    }
    set_archer(&mut d, owners, ArcherSharing::Equal);
    assert_eq!(d.income.len(), 2);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!(value(&r, "agi"), 80001);
        assert_eq!(column(&r, "13b", "b"), [0, 1][i]);
        assert_eq!(column(&r, "18", "b"), 0);
        assert!(r.lines.iter().any(|v| v.id == "form_8379_injured_share_of_tax"));
    }
}

#[test]
fn unit13_able_native_distribution_and_contribution_tax_have_their_actual_owner() {
    let mut d = injured_return((60000, 10000), (20000, 5000), 0, allocation());
    let mut distribution = able_distribution(1000, 300, 500);
    let FederalIncome::Able { data } = &mut distribution else { unreachable!() };
    data.form.earnings = 30049;
    data.form.basis = data.form.gross - data.form.earnings;
    let mut source = income(distribution);
    source.owner = crate::DocumentOwner::Spouses;
    add(&mut d, source);
    let mut contributions = able_contributions(
        crate::year_constants::ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS + 1000,
        0,
        1000,
    );
    contributions.owner = Owner::Taxpayer;
    d.screening.as_mut().unwrap().able_contributions.push(contributions);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!(column(&r, "13b", "b"), [0, 150][i]);
        assert_eq!((column(&r, "18", "b"), column(&r, "18", "c")), ([60, 15][i], [60, 15][1 - i]));
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let joint = supported(&ordinary);
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
        assert!(r.lines.iter().any(|v| v.id == "form_8379_injured_share_of_tax"));
    }
}

#[test]
fn unit13_archer_rational_projection_matches_the_native_boundary_without_cent_rerounding() {
    use crate::kernel::ratio::{dollars, fraction, project_fraction_pair};
    for source in 0..5000 {
        let native = fraction(dollars(source), 1, 5);
        assert_eq!(fraction(source, 1, 500), native);
        assert_eq!(project_fraction_pair(native, [source, 0], 1, 500, 0), Some([native, 0]));
    }
    assert_eq!(project_fraction_pair(1, [249, 249], 1, 500, 0), Some([0, 1]));
    assert_eq!(project_fraction_pair(1, [250, 250], 1, 500, 1), Some([0, 1]));
    assert_eq!(project_fraction_pair(1, [249, 249], 1, 500, 2), None);
    assert_eq!(project_fraction_pair(1, [249, 249], 1, 0, 0), None);
    assert_eq!(project_fraction_pair(0, [249, 249], 1, 500, 0), None);
}

#[test]
fn unit13_account_income_raises_native_niit_magi_without_becoming_investment_income() {
    let mut d = unit13_education_source_fixture();
    d.income.retain(|e| e.id != 1);
    let mut interest_data = savings_income(250000, 0);
    let FederalIncome::Interest { data } = &mut interest_data else { unreachable!() };
    edit_interest(data).withholding = 7000000;
    let mut interest = income(interest_data);
    interest.owner = crate::DocumentOwner::Yours;
    add(&mut d, interest);
    *d.income.iter_mut().find(|e| e.id == 2).unwrap() = wage(2, Owner::Spouse, 6000000, 2000000);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        let i = usize::from(injured == Owner::Spouse);
        // Joint MAGI311000 gives61000 ceiling; native NII is only250000
        // interest. The2318 NIIT stays with its actual taxpayer source;
        // the spouse owns only its100 Form5329 distribution additional tax.
        assert_eq!(value(&r, "net_investment_income_tax"), 2318);
        assert_eq!(
            (column(&r, "18", "b"), column(&r, "18", "c")),
            ([2318, 100][i], [2318, 100][1 - i])
        );
        assert_eq!(column(&r, "13b", "b"), [250000, 1000][i]);
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let joint = supported(&ordinary);
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
        assert!(r.lines.iter().any(|v| v.id == "form_8379_injured_share_of_tax"));
    }
}

#[test]
fn unit13_account_actual_client_recipient_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::accepted;
    let mut session = Interview::new(TEST_USER, unit13_education_source_fixture());
    let page = open_documents(&mut session);
    let native = NativeInterviewCase("TAX_UNIT13_ACCOUNT");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if native.seed(&page) {
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        let mut page = page;
        let P::SaveDocuments { documents, .. } = &mut page else { panic!() };
        documents
            .other
            .other_income
            .documents
            .iter_mut()
            .find(|v| matches!(v.document, crate::OtherIncomeDocument::Q1099(_)))
            .unwrap()
            .owner = crate::DocumentOwner::Yours;
        fixture_owner_inputs(&session.draft, page)
    };
    let P::SaveDocuments { documents, .. } = &request else { panic!() };
    let changed = documents
        .other
        .other_income
        .documents
        .iter()
        .find(|v| matches!(v.document, crate::OtherIncomeDocument::Q1099(_)))
        .unwrap();
    assert_eq!(changed.owner, crate::DocumentOwner::Yours);
    let id = changed.id;
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!((column(&result, "13b", "b"), column(&result, "13b", "c")), (1000, 0));
    assert_eq!((column(&result, "18", "b"), column(&result, "18", "c")), (100, 0));
    let explanation = &result
        .lines
        .iter()
        .find(|v| v.id == "form_8379_injured_share_of_tax")
        .unwrap()
        .explanation;
    assert_eq!(explanation.args[1], MessageArg::usd_dollars(5143));
    assert_eq!(explanation.args[2], MessageArg::usd_dollars(393));
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let decoded = draft_from_native_records(tax_year, &stored);
    assert_eq!(native_draft_records(&decoded), stored);
    assert_eq!(decoded, session.draft);
    let mut reopened = restart(session.draft);
    let shown = open_documents(&mut reopened);
    native.output(&shown, &stored);
    assert!(reopened.draft.income.iter().any(|v| v.id == id
        && v.owner == crate::DocumentOwner::Yours
        && matches!(v.income, FederalIncome::EducationDistribution { .. })));
    accepted(&send(&mut reopened, shown));
    let after = supported(&reopened.draft);
    assert_eq!((after.tax, after.refund), (result.tax, result.refund));
    assert_eq!(column(&after, "13b", "b"), 1000);
}

#[path = "unit13_k1_deferred_tax_sources.rs"]
mod unit13_k1_deferred_tax_sources;
#[path = "../credits/unit16_clergy_household_sources.rs"]
mod unit16_clergy_household_sources;

#[path = "unit13_royalty_sources.rs"]
mod unit13_royalty_sources;

fn unit13_operating_niit_fixture(business_owner: Owner, portfolio_owner: Owner) -> FederalDraft {
    let mut draft = injured_return((60_000, 30_000), (0, 0), 0, allocation());
    let mut business = income(cash_business(20_000));
    business.owner = crate::DocumentOwner::from(business_owner);
    add(&mut draft, business);
    let mut portfolio = income(savings_income(220_000, 0));
    portfolio.owner = crate::DocumentOwner::from(portfolio_owner);
    add(&mut draft, portfolio);
    draft
}

#[test]
fn unit13_operating_niit_uses_investment_owner_not_active_business_or_magi_owner() {
    for operator in [Owner::Taxpayer, Owner::Spouse] {
        for investor in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = unit13_operating_niit_fixture(operator, investor);
            let result = supported(&draft);
            // SE:20,000*.9235*.153=2,825.91 ->2,826; half=1,413.
            // AGI=60,000+20,000+220,000-1,413=298,587.
            // NII=220,000; joint MAGI excess48,587; NIIT=1,846.306 ->1,846.
            assert_eq!(value(&result, "self_employment_tax"), 2826);
            assert_eq!(value(&result, "agi"), 298_587);
            assert_eq!(value(&result, "form_8960_net_investment_income"), 220_000);
            assert_eq!(value(&result, "net_investment_income_tax"), 1846);
            let mut owners = [0; 2];
            owners[usize::from(operator == Owner::Spouse)] += 2826;
            owners[usize::from(investor == Owner::Spouse)] += 1846;
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                draft.injured_spouse.as_mut().unwrap().injured = injured;
                let projected = supported(&draft);
                let first = usize::from(injured == Owner::Spouse);
                assert_eq!(column(&projected, "18", "a"), 4672);
                assert_eq!(column(&projected, "18", "b"), owners[first]);
                assert_eq!(column(&projected, "18", "c"), owners[1 - first]);
                assert!(
                    !projected
                        .review_notes
                        .iter()
                        .any(|note| note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
                );
                let mut plain = draft.clone();
                plain.injured_spouse = None;
                let ordinary = supported(&plain);
                assert_eq!(
                    (projected.tax, projected.refund, projected.balance),
                    (ordinary.tax, ordinary.refund, ordinary.balance)
                );
            }
            draft.income.reverse();
            let reordered = supported(&draft);
            assert_eq!((reordered.tax, reordered.refund), (result.tax, result.refund));
            assert_eq!(restart(draft.clone()).draft, draft);
        }
    }
}

#[test]
fn unit13_operating_niit_preserves_passive_business_se_and_investment_ownership() {
    let mut draft = unit13_operating_niit_fixture(Owner::Taxpayer, Owner::Spouse);
    let native = supported(&draft);
    let FederalIncome::Business { data } = &mut draft.income[2].income else { panic!() };
    data.material_participation =
        crate::ParticipationFacts { hours: 0, test: crate::ParticipationTest::None };
    let result = supported(&draft);
    // 2026-10-24: current passive C enters469 and retains1401 character.
    // Reg1.1411-9(a) excludes the SE items, not the spouse's portfolio.
    assert_eq!(value(&result, "agi"), 298_587);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 220_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 1846);
    assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [2826, 1846]);
    assert_eq!(result.tax, native.tax);
    assert!(!result.review_notes.iter().any(|note| note.display.key == "WARN_BUSINESS_LEFT_OUT"));
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary.tax, ordinary.refund, ordinary.balance)
    );
}

#[test]
fn unit13_passive_notary_niit_preserves_native_source_ownership() {
    let mut draft = unit13_operating_niit_fixture(Owner::Taxpayer, Owner::Taxpayer);
    draft.income[2].income = passive_business(20_000);
    let FederalIncome::Business { data } = &mut draft.income[2].income else { panic!() };
    data.notary_profit = 2_000_000;
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 300_000);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 240_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 1900);
    assert_eq!(column(&result, "18", "a"), 1900);
    assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [1900, 0]);
    assert!(
        !result
            .review_notes
            .iter()
            .any(|note| { note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE" })
    );
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary.tax, ordinary.refund, ordinary.balance)
    );
}

#[test]
fn unit13_passive_statutory_wages_preserve_native_niit_ownership() {
    let mut draft = unit13_operating_niit_fixture(Owner::Taxpayer, Owner::Taxpayer);
    draft.income[2].income = cash_business(0);
    let id = draft.income[2].id;
    source_statutory_wage(&mut draft, Owner::Taxpayer, Some(id));
    let native = supported(&draft);
    let FederalIncome::Business { data } = &mut draft.income[2].income else { panic!() };
    data.material_participation =
        crate::ParticipationFacts { hours: 0, test: crate::ParticipationTest::None };
    add(&mut draft, income(passive_business(-1000)));
    let loss = draft.income.last().unwrap().id;
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 280_000);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 220_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 1140);
    assert_eq!(column(&result, "13a", "a"), 60_000);
    assert_eq!([column(&result, "13a", "b"), column(&result, "13a", "c")], [60_000, 0]);
    assert_eq!(column(&result, "18", "a"), 1140);
    assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [1140, 0]);
    assert_eq!(value(&result, &alloc::format!("business_{loss}_passive_current")), -1000);
    assert_eq!(value(&result, &alloc::format!("business_{loss}_passive_suspended")), 1000);
    assert_eq!(value(&result, &alloc::format!("business_{loss}_passive_allowed")), 0);
    assert!(
        !result.lines.iter().any(|line| line.id == alloc::format!("business_{id}_passive_allowed"))
    );
    assert_eq!(result.tax, native.tax);
    assert!(
        !result
            .review_notes
            .iter()
            .any(|note| { note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE" })
    );
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let ordinary = supported(&plain);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary.tax, ordinary.refund, ordinary.balance)
    );
}

#[test]
fn unit13_operating_niit_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let profile =
        std::env::var("TAX_UNIT13_OPERATING_NIIT_PROFILE").unwrap_or_else(|_| "business".into());
    let draft = unit13_operating_niit_fixture(Owner::Taxpayer, Owner::Spouse);
    let topic = if profile == "business" {
        TaxSection::SelfEmployment
    } else {
        TaxSection::InterestDividends
    };
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = if profile == "business" {
        open_topic(&mut session, topic)
    } else {
        open_documents(&mut session)
    };
    let native = NativeInterviewCase("TAX_UNIT13_OPERATING_NIIT");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if native.seed(&page) {
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        if profile == "business" {
            let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
            let row = &mut documents.documents[0];
            row.owner = Owner::Spouse;
            let crate::IncomeRecord::Business(data) = &mut row.document else { panic!() };
            data.receipts = 3_000_000;
        } else {
            let P::SaveDocuments { documents, .. } = &mut page else { panic!() };
            documents.common.interest_dividends.documents[0].owner = crate::DocumentOwner::Yours;
        }
        page
    };
    accepted(&send(&mut session, request));
    let check = |draft: &FederalDraft| {
        let result = supported(draft);
        let expected = if profile == "business" { [0, 6438] } else { [4672, 0] };
        assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], expected);
        assert_eq!(
            value(&result, "net_investment_income_tax"),
            if profile == "business" { 2199 } else { 1846 }
        );
        assert!(
            !result
                .review_notes
                .iter()
                .any(|note| note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        let mut plain = draft.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(
            (result.tax, result.refund, result.balance),
            (ordinary.tax, ordinary.refund, ordinary.balance)
        );
    };
    check(&session.draft);
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let draft = draft_from_native_records(tax_year, &stored);
    assert_eq!(draft, session.draft);
    let mut reopened = restart(draft);
    let shown = if profile == "business" {
        open_topic(&mut reopened, topic)
    } else {
        open_documents(&mut reopened)
    };
    native.output(&shown, &stored);
    let accepted_draft = reopened.draft.clone();
    accepted(&send(&mut reopened, shown));
    assert_eq!(reopened.draft, accepted_draft);
    check(&reopened.draft);
}

fn unit13_mortgage_gift(
    owners: Option<MortgageSubsidyGiftOwners>,
    label: &str,
) -> MortgageSubsidyEntry {
    let mut home = crate::tests::federal::other_taxes::gift(20200601, 300_000, 200_000, 0, 60);
    home.label = label.into();
    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { owners: value, .. },
        ..
    } = &mut home.disposition
    else {
        unreachable!()
    };
    *value = owners;
    home
}

#[test]
fn unit13_mortgage_gift_source_owner_retains_native_recapture_tax() {
    for (owner, expected) in [
        (MortgageSubsidyGiftOwners::Taxpayer, [7_500, 0]),
        (MortgageSubsidyGiftOwners::Spouse, [0, 7_500]),
    ] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut answer = allocation();
            answer.injured = injured;
            let mut draft = injured_return((60_000, 20_000), (20_000, 10_000), 0, answer);
            draft.credits.as_mut().unwrap().mortgage_subsidies =
                vec![unit13_mortgage_gift(Some(owner), "Maple Street")];
            let result = supported(&draft);
            assert_eq!(value(&result, "mortgage_subsidy_recapture_tax"), 7_500);
            assert_eq!(
                [column(&result, "18", "b"), column(&result, "18", "c")],
                if injured == Owner::Taxpayer { expected } else { [expected[1], expected[0]] }
            );
            assert!(result.lines.iter().any(|line| line.id == "form_8379_injured_refund_estimate"));
            let mut plain = draft.clone();
            plain.injured_spouse = None;
            plain.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
            let ordinary = supported(&plain);
            assert_eq!(
                (ordinary.tax, ordinary.refund, ordinary.balance),
                (result.tax, result.refund, result.balance)
            );
            assert_eq!(supported(&restart(draft.clone()).draft), result);
        }
    }
}

#[test]
fn unit13_mortgage_shared_interest_uses_existing_bounded_amount() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut answer = allocation();
        answer.injured = injured;
        let mut draft = injured_return((60_000, 20_000), (20_000, 10_000), 0, answer);
        draft.credits.as_mut().unwrap().mortgage_subsidies = vec![
            unit13_mortgage_gift(Some(MortgageSubsidyGiftOwners::Taxpayer), "Exclusive interest"),
            unit13_mortgage_gift(Some(MortgageSubsidyGiftOwners::Both), "Shared interest"),
        ];
        let incomplete = supported(&draft);
        let plan = incomplete.injured_spouse.as_ref().unwrap();
        let row = plan
            .required
            .iter()
            .find(|row| row.component == "mortgage_subsidy_recapture_tax")
            .unwrap();
        assert_eq!(row.joint, 1_500_000);
        assert_eq!(
            (row.minimum, row.maximum),
            if injured == Owner::Taxpayer { (750_000, 1_500_000) } else { (0, 750_000) }
        );
        draft.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
            component: "mortgage_subsidy_recapture_tax".into(),
            injured: if injured == Owner::Taxpayer { 1_000_000 } else { 500_000 },
        });
        let complete = supported(&draft);
        assert_eq!(
            [column(&complete, "18", "b"), column(&complete, "18", "c")],
            if injured == Owner::Taxpayer { [10_000, 5_000] } else { [5_000, 10_000] }
        );
        assert!(complete.lines.iter().any(|line| line.id == "form_8379_injured_refund_estimate"));
        assert_eq!(
            (complete.tax, complete.refund, complete.balance),
            (incomplete.tax, incomplete.refund, incomplete.balance)
        );
    }
    let mut missing = injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation());
    missing.credits.as_mut().unwrap().mortgage_subsidies =
        vec![unit13_mortgage_gift(None, "Unestablished interest")];
    unavailable(&missing, "WARN_8379_MORTGAGE_TAX_OWNER_UNAVAILABLE");
    let review =
        crate::interview::remaining_items(&missing, &supported(&missing), TaxSection::Review);
    assert!(review.iter().any(|row| row.section == TaxSection::OtherTaxes
        && row.status == libertas::libertas_formatted_text("MORTGAGE_GIFT_OWNER_REQUIRED", &[])));
    assert_eq!(value(&supported(&missing), "mortgage_subsidy_recapture_tax"), 7_500);
}

#[test]
fn unit13_mortgage_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_other_taxes};
    let profile = std::env::var("TAX_UNIT13_MORTGAGE_PROFILE").unwrap_or_else(|_| "joint".into());
    let mut draft = if profile == "joint" {
        injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation())
    } else {
        complete(FederalStatus::Single)
    };
    draft.credits.as_mut().unwrap().mortgage_subsidies =
        vec![unit13_mortgage_gift(None, "Maple Street")];
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_other_taxes(&mut session);
    let native = NativeInterviewCase("TAX_UNIT13_MORTGAGE");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if native.seed(&page) {
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        let P::SaveOtherTaxes { value, .. } = &mut page else { panic!() };
        let OtherTax::MortgageSubsidyRecapture { homes } = &mut value.taxes[0] else { panic!() };
        let MortgageSubsidyDispositionView::Ordinary {
            transfer: MortgageSubsidyTransferView::GiftToOther { owners, fair_market_value, .. },
            ..
        } = &mut homes[0].disposition
        else {
            panic!()
        };
        *owners = if profile == "joint" { Some(MortgageSubsidyGiftOwners::Spouse) } else { None };
        *fair_market_value = 40_000_000;
        page
    };
    if std::env::var_os("TAX_UNIT13_MORTGAGE_REJECT").is_some() {
        let before = session.draft.clone();
        let Reply::Response(P::Problem { message, .. }) = send(&mut session, request) else {
            panic!()
        };
        assert_eq!(message, libertas::libertas_formatted_text("MORTGAGE_GIFT_OWNER_REQUIRED", &[]));
        assert_eq!(session.draft, before);
        return;
    }
    accepted(&send(&mut session, request));
    let check = |draft: &FederalDraft| {
        let result = supported(draft);
        assert_eq!(value(&result, "mortgage_subsidy_recapture_tax"), 7_500);
        let MortgageSubsidyDisposition::Ordinary {
            transfer: MortgageSubsidyTransfer::GiftToOther { owners, fair_market_value, .. },
            ..
        } = &draft.credits.as_ref().unwrap().mortgage_subsidies[0].disposition
        else {
            panic!()
        };
        assert_eq!(*fair_market_value, 40_000_000);
        assert_eq!(
            *owners,
            if profile == "joint" { Some(MortgageSubsidyGiftOwners::Spouse) } else { None }
        );
        if profile == "joint" {
            assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [0, 7_500]);
            let mut ordinary = draft.clone();
            ordinary.injured_spouse = None;
            ordinary.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
            let plain = supported(&ordinary);
            assert_eq!(
                (result.tax, result.refund, result.balance),
                (plain.tax, plain.refund, plain.balance)
            );
        }
        result
    };
    let expected = check(&session.draft);
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let draft = draft_from_native_records(tax_year, &stored);
    assert_eq!(draft, session.draft);
    let mut again = restart(draft);
    let shown = open_other_taxes(&mut again);
    native.output(&shown, &stored);
    let before = again.draft.clone();
    accepted(&send(&mut again, shown));
    assert_eq!(again.draft, before);
    assert_eq!(check(&again.draft), expected);
}

#[path = "unit13_k1_portfolio_niit_sources.rs"]
mod unit13_k1_portfolio_niit_sources;
#[path = "unit13_niit_binding_cap.rs"]
mod unit13_niit_binding_cap;
#[path = "unit13_niit_deduction_sources.rs"]
mod unit13_niit_deduction_sources;
#[path = "unit13_niit_expense_person_sources.rs"]
mod unit13_niit_expense_person_sources;
#[path = "unit13_niit_zero_allowed.rs"]
mod unit13_niit_zero_allowed;

#[path = "unit13_child_debt_sources.rs"]
mod unit13_child_debt_sources;
#[path = "unit13_election_noninvestment_property.rs"]
mod unit13_election_noninvestment_property;
#[path = "unit13_foreign_sources.rs"]
mod unit13_foreign_sources;
#[path = "unit13_personal_rental_sources.rs"]
mod unit13_personal_rental_sources;
#[path = "unit13_wage_classification_sources.rs"]
mod unit13_wage_classification_sources;

#[path = "unit13_additional_medicare_contributors.rs"]
mod unit13_additional_medicare_contributors;
#[path = "unit13_alimony_recapture_sources.rs"]
mod unit13_alimony_recapture_sources;
#[path = "unit13_bonds_cooperative_sources.rs"]
mod unit13_bonds_cooperative_sources;
#[path = "unit13_estate_estimated_payment_sources.rs"]
mod unit13_estate_estimated_payment_sources;
#[path = "unit13_home_employee_niit_sources.rs"]
mod unit13_home_employee_niit_sources;
#[path = "unit13_home_payroll_actual_clients.rs"]
mod unit13_home_payroll_actual_clients;
#[path = "unit13_k1_activity_niit_sources.rs"]
mod unit13_k1_activity_niit_sources;
#[path = "unit13_ltc_cent_actual_clients.rs"]
mod unit13_ltc_cent_actual_clients;
#[path = "unit13_ltc_cent_sources.rs"]
mod unit13_ltc_cent_sources;
#[path = "unit13_medicare_actual_clients.rs"]
mod unit13_medicare_actual_clients;
#[path = "unit13_medicare_withholding_sources.rs"]
mod unit13_medicare_withholding_sources;
#[path = "unit13_native_payroll_other_tax_sources.rs"]
mod unit13_native_payroll_other_tax_sources;
#[path = "unit13_niit_relief_completeness.rs"]
mod unit13_niit_relief_completeness;
#[path = "unit13_niit_supporting_sources.rs"]
mod unit13_niit_supporting_sources;
#[path = "unit13_reit_owner_footing.rs"]
mod unit13_reit_owner_footing;
#[path = "unit13_typed_information_return_sources.rs"]
mod unit13_typed_information_return_sources;

#[path = "unit13_business_credit_owner_sources.rs"]
mod unit13_business_credit_owner_sources;
#[path = "unit13_elective_credit_mixed_sources.rs"]
mod unit13_elective_credit_mixed_sources;
#[path = "unit13_elective_credit_owner_sources.rs"]
mod unit13_elective_credit_owner_sources;
#[path = "unit13_financial_omission_scope.rs"]
mod unit13_financial_omission_scope;
#[path = "unit13_foreign_credit_ordinary_sources.rs"]
mod unit13_foreign_credit_ordinary_sources;
#[path = "unit13_fuel_credit_owner_sources.rs"]
mod unit13_fuel_credit_owner_sources;
#[path = "unit13_original_map_boundaries.rs"]
mod unit13_original_map_boundaries;
#[path = "unit13_pre_gbc_credit_sources.rs"]
mod unit13_pre_gbc_credit_sources;
#[path = "unit13_r8_category_rounding.rs"]
mod unit13_r8_category_rounding;
#[path = "unit13_r8_credit_relevance.rs"]
mod unit13_r8_credit_relevance;
#[path = "unit13_r8_employee_withholding.rs"]
mod unit13_r8_employee_withholding;
#[path = "unit13_r8_sale_withholding.rs"]
mod unit13_r8_sale_withholding;
#[path = "unit13_r9_actual_clients.rs"]
mod unit13_r9_actual_clients;

#[path = "unit13_original_map_actual_clients.rs"]
mod unit13_original_map_actual_clients;

#[test]
fn applied_credit_preserves_pre_election_injured_spouse_offset_allocation() {
    // IRM 21.4.6.5.2: offset allocation precedes the credit election.
    let mut d = injured_return((60_000, 12_000), (40_000, 0), 0, allocation());
    let before = supported(&d);
    let rows = |r: &FederalResult| {
        r.lines.iter().filter(|line| line.id.starts_with("form_8379_")).cloned().collect::<Vec<_>>()
    };
    for credit in [100_050, 435_700] {
        let pay = d.payments.as_mut().unwrap();
        pay.credit_applied_next_year = credit;
        pay.credit_recipient = Some(OverpaymentRecipient::SpouseAccount);
        let r = supported(&d);
        assert_eq!(rows(&r), rows(&before));
        assert_eq!(r.refund, Some(435_700 - crate::kernel::ratio::dollars(credit) * 100));
        assert_eq!(r.credit_capacity, Some(435_700));
    }
}

#[path = "unit13_estate_carry_sources.rs"]
mod unit13_estate_carry_sources;

// 2026-10-19 acceptance defect 3: community Yes keeps the accepted allocation answers.
#[test]
fn injured_community_yes_resave_keeps_child_credit_answers() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, from_review};
    use crate::tests::support::client::native_editor;
    let mut draft = complete(FederalStatus::Joint);
    draft.income = vec![
        wage(1, Owner::Taxpayer, 5_000_000, 800_000),
        wage(2, Owner::Spouse, 2_000_000, 100_000),
    ];
    draft.dependents = vec![child(3), child(4)];
    draft.next_id = 5;
    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    let mut session = restart(draft);
    let Reply::Edit(mut page) =
        native_editor(from_review(&mut session, TaxSection::InjuredSpouse), &[])
    else {
        panic!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
    value.claimants[1].claimant = Owner::Spouse;
    let refreshed = send(&mut session, page);
    let Reply::Response(P::BeginInjuredSpouse { value, .. }) = &refreshed else { panic!() };
    let ctc = value.new_allocations.iter().find(|r| r.component == "ctc_odc_pool").unwrap().minimum;
    let actc = value.child_credit_cap_injured - value.known_child_credit_injured - ctc;
    let Reply::Edit(save) = native_editor(refreshed, &[("ctc_odc_pool", ctc), ("actc_pool", actc)])
    else {
        panic!()
    };
    accepted(&send(&mut session, save));
    let stored = session.draft.injured_spouse.clone().unwrap().allocations;
    assert_eq!(stored.len(), 2);
    let Reply::Edit(mut page) =
        native_editor(from_review(&mut session, TaxSection::InjuredSpouse), &[])
    else {
        panic!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
    value.community =
        InjuredCommunity::Yes { states: vec![5], debt_facts: InjuredDebtFacts::NotApplicable };
    send(&mut session, page);
    // The unanswered income rows leave no plan, so the page reopens without rows;
    // saving it unchanged keeps the record and its accepted answers.
    let reopened = from_review(&mut session, TaxSection::InjuredSpouse);
    let before = session.draft.clone();
    let Reply::Edit(page) = native_editor(reopened, &[]) else { panic!() };
    send(&mut session, page);
    assert_eq!(session.draft, before);
    assert_eq!(session.draft.injured_spouse.as_ref().unwrap().allocations, stored);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

// 2026-10-19 acceptance defect 2: a community federal tax debt can be entered.
#[test]
fn injured_community_federal_tax_debt_receives_its_choices_on_save() {
    use crate::tests::cross::interview::from_review;
    use crate::tests::support::client::native_editor;
    let mut draft = complete(FederalStatus::Joint);
    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    let mut session = restart(draft);
    let Reply::Edit(mut page) =
        native_editor(from_review(&mut session, TaxSection::InjuredSpouse), &[])
    else {
        panic!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
    value.debt = OffsetDebt::FederalTax;
    value.community =
        InjuredCommunity::Yes { states: vec![5], debt_facts: InjuredDebtFacts::NotApplicable };
    let before = session.draft.clone();
    let Reply::Response(refreshed @ P::BeginInjuredSpouse { .. }) = send(&mut session, page) else {
        panic!("a stale choice list is refreshed, not refused")
    };
    assert_eq!(session.draft, before);
    let P::BeginInjuredSpouse { value, .. } = &refreshed else { unreachable!() };
    assert_eq!(value.allowed_debt_facts.len(), 3);
    let Reply::Edit(mut page) = native_editor(Reply::Response(refreshed), &[]) else { panic!() };
    // Still unanswered: NotApplicable is not a choice for this debt.
    crate::tests::support::assert::rejected_with(
        send(&mut session, page.clone()),
        "INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT",
    );
    let P::SaveInjuredSpouse { value, .. } = &mut page else { unreachable!() };
    value.community =
        InjuredCommunity::Yes { states: vec![5], debt_facts: InjuredDebtFacts::BeforeMarriage };
    send(&mut session, page);
    assert!(matches!(
        session.draft.injured_spouse.as_ref().unwrap().community,
        InjuredCommunityFacts::Yes { debt_facts: InjuredDebtFacts::BeforeMarriage, .. }
    ));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

// 2026-10-19 acceptance defect 7: a row never shows a community choice outside its own set.
#[test]
fn injured_community_yes_wage_rows_show_a_value_inside_their_allowed_set() {
    use crate::tests::cross::interview::{from_review, page_documents};
    use crate::tests::support::client::native_editor;
    let mut draft = complete(FederalStatus::Joint);
    draft.income = vec![wage(1, Owner::Taxpayer, 5_000_000, 800_000)];
    draft.next_id = 2;
    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    let mut session = restart(draft);
    let Reply::Edit(mut page) =
        native_editor(from_review(&mut session, TaxSection::InjuredSpouse), &[])
    else {
        panic!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
    value.community =
        InjuredCommunity::Yes { states: vec![5], debt_facts: InjuredDebtFacts::NotApplicable };
    send(&mut session, page);
    let wages = open_documents(&mut session);
    let rows = page_documents(&wages);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].allowed_community.contains(&crate::community::index(rows[0].community)));
    assert!(rows[0].allowed_community.contains(&crate::community::index(CommunityShare::Separate)));
}

#[test]
fn estimated_annual_funding_keeps_actual_payers_and_community_parts_when_timing_is_unknown() {
    let mut claim = allocation();
    claim.community =
        InjuredCommunityFacts::Yes { states: vec![5], debt_facts: InjuredDebtFacts::NotApplicable };
    let mut draft = injured_return((60_000, 10_000), (20_000, 1_000), 0, claim);
    for source in &mut draft.income {
        source.community = CommunityShare::Separate;
    }
    let parts = vec![
        DatedPayment {
            date: 20260415,
            amount: 70_000,
            payer: PaymentPayer::Taxpayer,
            refund_property: Some(RefundSourcePropertyFacts::Partly {
                community: 20_000,
                separate_owner: Owner::Taxpayer,
                management: RefundManagement::NotApplicable,
            }),
        },
        DatedPayment {
            date: 20260615,
            amount: 30_000,
            payer: PaymentPayer::Spouse,
            refund_property: Some(RefundSourcePropertyFacts::Separate { owner: Owner::Spouse }),
        },
    ];
    draft.payments.as_mut().unwrap().estimated =
        EstimatedPaymentEntryRecord::Sources { payments: parts.clone() };
    let dated = supported(&draft);
    let annual = parts
        .into_iter()
        .map(|source| AnnualEstimatedAmountRecord {
            amount: source.amount,
            payer: source.payer,
            refund_property: source.refund_property,
        })
        .collect::<Vec<_>>();
    for equal in [false, true] {
        draft.payments.as_mut().unwrap().estimated = if equal {
            EstimatedPaymentEntryRecord::EqualTimely { payments: annual.clone() }
        } else {
            EstimatedPaymentEntryRecord::TimingUnknown { payments: annual.clone() }
        };
        let result = supported(&draft);
        assert_eq!(result, dated);
        assert_eq!(return_line(&result, "26"), 100_000);
        assert_eq!(share_cents(&result), share_cents(&dated));
        assert_eq!(
            crate::injured_spouse::joint_estimated(&draft),
            crate::kernel::ratio::exact::whole(0)
        );
    }
    let EstimatedPaymentEntryRecord::EqualTimely { payments } =
        &mut draft.payments.as_mut().unwrap().estimated
    else {
        unreachable!()
    };
    payments[0].refund_property = None;
    unavailable(&draft, "WARN_8379_COMMUNITY_ESTIMATE_UNAVAILABLE");
}
