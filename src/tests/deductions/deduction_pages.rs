use crate::tests::support::assert::{accepted, rejected_with};
use crate::tests::support::client::survey::survey_preview_document;
// T3-deduct (wave 5, 2026-09-28): the pages for the deductions no page
// asked before. Each form is reached through the interview (the page's save
// and its reopen after a restart), with hand-computed totals from the IRS
// instructions in comments; boundary tests follow each form's limits, and the
// server twins of the pages' client rules.
use super::*;
use crate::tests::cross::interview::from_review;
use alloc::format;

fn open(s: &mut Interview, section: TaxSection) -> P {
    match from_review(s, section) {
        Reply::Edit(page) => page,
        other => panic!("expected the {section:?} page, got {other:?}"),
    }
}
fn refused(reply: &Reply) -> bool {
    matches!(reply, Reply::Response(P::Problem { .. }))
}
// The deductions page opened from the review, changed and saved.
fn deductions_request_with(s: &mut Interview, change: impl FnOnce(&mut Deductions)) -> P {
    let mut page = open(s, TaxSection::Deductions);
    let P::SaveDeductions { value, .. } = &mut page else {
        panic!("expected the deductions page, got {page:?}")
    };
    change(value);
    page
}
fn save_deductions_with(s: &mut Interview, change: impl FnOnce(&mut Deductions)) -> Reply {
    let request = deductions_request_with(s, change);
    send(s, request)
}
fn deductions_shown(s: &mut Interview) -> (Deductions, Vec<i32>) {
    match open(s, TaxSection::Deductions) {
        P::SaveDeductions { value, allowed_repayment, .. } => (value, allowed_repayment),
        other => panic!("expected the deductions page, got {other:?}"),
    }
}
fn save_gifts_with(s: &mut Interview, change: impl FnOnce(&mut GiftCarryovers)) -> Reply {
    let mut page = open(s, TaxSection::GiftCarryovers);
    let P::SaveGiftCarryovers { value, .. } = &mut page else {
        panic!("expected the gifts page, got {page:?}")
    };
    change(value);
    send(s, page)
}
fn other_request_with(s: &mut Interview, change: impl FnOnce(&mut OtherItemized)) -> P {
    let mut page = open(s, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value, .. } = &mut page else {
        panic!("expected the other itemized page, got {page:?}")
    };
    change(value);
    page
}
fn save_other_with(s: &mut Interview, change: impl FnOnce(&mut OtherItemized)) -> Reply {
    let request = other_request_with(s, change);
    send(s, request)
}
// A complete return whose deductions page chose to itemize.
fn itemizing(d: FederalDraft) -> Interview {
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save_deductions_with(&mut s, |v| v.route = DeductionRoute::Itemize));
    s
}
fn casualty(label: &str, basis: i64) -> PersonalCasualty {
    personal_disaster(label, basis)
}

// Schedule 1-A Part IV (2026 draft lines 28-36), through the deductions
// page. Single, W-2 60,000, so MAGI (line 31) is 60,000, below 100,000.
// Loan A: 1098-VLI box 1 4,200, 700 deducted on Schedule C (column ii), new
// and assembled in the United States: column (iii) 3,500. Loan B: a used
// car (question (1) No): nothing. Line 29 3,500; line 30 min(3,500, 10,000)
// = 3,500; line 33 is below zero, so line 36 = 3,500. Taxable income
// 60,000 - 16,100 standard - 3,500 = 40,400; AGI stays 60,000.
#[test]
fn vehicle_loans_reach_schedule_1a_through_the_deductions_page() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let loans = vec![
        FederalVehicleInterest {
            business: 70_000,
            ..vehicle_loan("Synthetic Credit Union", 420_000)
        },
        FederalVehicleInterest {
            original_use: false,
            ..vehicle_loan("Synthetic Used Car Finance", 100_000)
        },
    ];
    let saved = loans.clone();
    accepted(&save_deductions_with(&mut s, |v| v.car_loans = saved));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "vehicle_interest_deduction"), 3_500);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "taxable_income"), 40_400);
    // The car loan tick follows the entry.
    assert_ne!(s.draft.checklist.deductions & DeductionTopics::CAR_LOAN.0, 0);
    // Round trip: stored, restarted and reopened unchanged.
    let mut again = restart(s.draft.clone());
    assert_eq!(again.draft, s.draft);
    assert_eq!(deductions_shown(&mut again).0.car_loans, loans);
    // No on a question takes no deduction.
    let mut d = s.draft.clone();
    d.deductions.as_mut().unwrap().vehicle_loans[0].assembled_in_us = false;
    assert_eq!(value(&supported(&d), "vehicle_interest_deduction"), 0);
}

// Line 30 caps the interest at 10,000.
#[test]
fn vehicle_interest_is_capped_at_10000() {
    let mut d = complete(FederalStatus::Single);
    d.deductions.as_mut().unwrap().vehicle_loans = vec![vehicle_loan("Lender", 1_000_000)];
    assert_eq!(value(&supported(&d), "vehicle_interest_deduction"), 10_000);
    d.deductions.as_mut().unwrap().vehicle_loans = vec![vehicle_loan("Lender", 1_200_000)];
    assert_eq!(value(&supported(&d), "vehicle_interest_deduction"), 10_000);
}

// Lines 32-35: 200 for each 1,000 or part of 1,000 of MAGI above 100,000
// (200,000 joint). 3,500 of interest: MAGI 100,000 keeps 3,500; 100,001 is
// one part (0.001 rounds up to 1), 3,300; 117,001 is 18 parts (3,600), 0.
#[test]
fn vehicle_interest_phases_out_by_200_per_started_1000() {
    for (status, wages, expected) in [
        (FederalStatus::Single, 10_000_000, 3_500),
        (FederalStatus::Single, 10_000_100, 3_300),
        (FederalStatus::Single, 11_700_100, 0),
        (FederalStatus::Joint, 20_000_000, 3_500),
        (FederalStatus::Joint, 20_000_100, 3_300),
    ] {
        let mut d = complete(status);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        d.deductions.as_mut().unwrap().vehicle_loans = vec![vehicle_loan("Lender", 350_000)];
        assert_eq!(
            value(&estimate(&d), "vehicle_interest_deduction"),
            expected,
            "{status:?} {wages}"
        );
    }
}

// Column (ii) above column (i)'s interest is refused on both sides.
#[test]
fn vehicle_business_part_above_interest_is_refused() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    rejected_with(
        save_deductions_with(&mut s, |v| {
            v.car_loans = vec![FederalVehicleInterest {
                business: 100_001,
                ..vehicle_loan("Lender", 100_000)
            }]
        }),
        "VEHICLE_BUSINESS_ABOVE_INTEREST",
    );
    assert_eq!(s.draft, before);
    // Equal is allowed and deducts nothing on Schedule 1-A.
    accepted(&save_deductions_with(&mut s, |v| {
        v.car_loans =
            vec![FederalVehicleInterest { business: 100_000, ..vehicle_loan("Lender", 100_000) }]
    }));
    assert_eq!(value(&supported(&s.draft), "vehicle_interest_deduction"), 0);
}

// Form 4684 Section A through the deductions page, itemizing. Single, AGI
// 60,000. A federally declared storm destroys property with basis and value
// 20,000 and no insurance: line 4 20,000 (smaller of basis and the drop in
// value), line 11 reduces it by 100 to 19,900, line 17 subtracts 10% of AGI
// (6,000): 13,900 on Schedule A line 16. Schedule A total 13,900; the
// Itemize route (Schedule A line 19) uses it although the standard 16,100
// is larger, so taxable income is 60,000 - 13,900 = 46,100.
#[test]
fn a_declared_disaster_loss_reaches_schedule_a_through_the_deductions_page() {
    let mut s = itemizing(complete(FederalStatus::Single));
    let mut event = casualty("March windstorm", 2_000_000);
    let saved = event.clone();
    accepted(&save_deductions_with(&mut s, |v| v.casualties = vec![saved.try_into().unwrap()]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "personal_casualty_deduction"), 13_900);
    assert_eq!(return_line(&r, "12e"), 1_390_000);
    assert_eq!(value(&r, "taxable_income"), 46_100);
    let mut again = restart(s.draft.clone());
    assert_eq!(again.draft, s.draft);
    // Server-issued keys are the only new values; every entered fact survives.
    let stored = &s.draft.deductions.as_ref().unwrap().personal_casualties[0];
    event.id = stored.id;
    event.properties[0].id = stored.properties[0].id;
    assert_eq!(stored, &event);
    assert_eq!(
        deductions_shown(&mut again).0.casualties,
        vec![PersonalCasualtyPage::try_from(event).unwrap()]
    );
}

// Line 11's 100 reduction per event, the 10%-of-AGI reduction (line 17),
// and the standard deduction route, which allows no declared-disaster loss
// that is not a qualified disaster loss.
#[test]
fn casualty_floors_and_the_standard_route() {
    let itemized = |basis: i64| {
        let mut d = complete(FederalStatus::Single);
        let ded = d.deductions.as_mut().unwrap();
        ded.personal_casualties = vec![casualty("Storm", basis)];
        ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
        d
    };
    // 100 lost: nothing after the event reduction.
    let r = estimate(&itemized(10_000));
    assert_eq!(value(&r, "personal_casualty_deduction"), 0);
    // 6,100 lost: 6,000 after the 100, not above 10% of AGI (6,000).
    assert_eq!(value(&estimate(&itemized(610_000)), "personal_casualty_deduction"), 0);
    // 6,101 lost: 1 above it.
    assert_eq!(value(&estimate(&itemized(610_100)), "personal_casualty_deduction"), 1);
    let mut d = itemized(2_000_000);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    assert_eq!(value(&supported(&d), "personal_casualty_deduction"), 0);
}

// The casualty list's client rules, checked again on save.
#[test]
fn casualty_rules_are_refused_on_both_sides() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut no_zip = casualty("Storm", 100_000);
    no_zip.zip.clear();
    rejected_with(
        save_deductions_with(&mut s, |v| v.casualties = vec![no_zip.try_into().unwrap()]),
        "CASUALTY_ZIP",
    );
    let mut undeclared_zip = casualty("Theft", 100_000);
    undeclared_zip.declaration = CasualtyDeclaration::Undeclared;
    rejected_with(
        save_deductions_with(&mut s, |v| v.casualties = vec![undeclared_zip.try_into().unwrap()]),
        "CASUALTY_ZIP",
    );
    let mut rising = casualty("Storm", 100_000);
    rising.properties[0].after = 100_001;
    rejected_with(
        save_deductions_with(&mut s, |v| v.casualties = vec![rising.try_into().unwrap()]),
        "CASUALTY_AFTER_ABOVE_BEFORE",
    );
    // 2026-10-17: Source identity and association uses saved keys, not
    // event labels. Distinct same-named events are valid; repeated keys are not.
    accepted(&save_deductions_with(&mut s, |v| {
        v.casualties = vec![
            casualty("Storm", 100_000).try_into().unwrap(),
            casualty("Storm", 200_000).try_into().unwrap(),
        ];
    }));
    let mut repeated = deductions_shown(&mut s).0.casualties;
    repeated[1].id = repeated[0].id;
    rejected_with(
        save_deductions_with(&mut s, |v| v.casualties = repeated),
        "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
    );
    // A part received cannot exceed the reimbursement (line 4 against line 3).
    let mut received = casualty("Storm", 100_000);
    received.properties[0].received = 1;
    rejected_with(
        save_deductions_with(&mut s, |v| v.casualties = vec![received.try_into().unwrap()]),
        "CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT",
    );
    // An undeclared event with no ZIP code is accepted.
    let mut theft = casualty("Theft", 100_000);
    theft.declaration = CasualtyDeclaration::Undeclared;
    theft.zip.clear();
    accepted(&save_deductions_with(&mut s, |v| v.casualties = vec![theft.try_into().unwrap()]));
}

fn with_recorded_repayment(amount: i64) -> FederalDraft {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    d.deductions.as_mut().unwrap().repayment = RepaymentClaim::None;
    let FederalIncome::Unemployment { data } = &mut d.income[1].income else { panic!() };
    data.repayments[0].amount = amount;
    d
}
fn comparison() -> RepaymentClaim {
    repayment_comparison_draft(RepaymentMethod::Automatic).deductions.unwrap().repayment
}

// Claim of right (IRC 1341, Pub. 525), through the deductions page. Single,
// W-2 60,000; 5,000 of 2025 unemployment repaid in 2026. The 2025 return:
// wages 60,000, unemployment 15,000, standard 15,750, taxable 59,250, tax
// from the 2025 table (59,250-59,300 row, midpoint 59,275) 1,192.50 +
// 4,386 + 0.22 x 10,800 = 7,954.50, 7,955. Without the 5,000: taxable
// 54,250, midpoint 54,275, 1,192.50 + 4,386 + 0.22 x 5,800 = 6,854.50,
// 6,855. Method 2 credit 7,955 - 6,855 = 1,100 (Schedule 3 line 13b,
// Form 1040 line 31), which beats a 5,000 itemized deduction below the
// 16,100 standard deduction.
#[test]
fn repayment_comparison_through_the_deductions_page() {
    let mut s = Interview::new(TEST_USER, with_recorded_repayment(500_000));
    let (value_shown, allowed) = deductions_shown(&mut s);
    assert_eq!(value_shown.repayment, RepaymentClaim::None);
    assert_eq!(allowed, vec![0, 1]);
    accepted(&save_deductions_with(&mut s, |v| v.repayment = comparison()));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "repayment_credit"), 1_100);
    assert_eq!(return_line(&r, "31"), 110_000);
    let mut again = restart(s.draft.clone());
    assert_eq!(again.draft, s.draft);
    assert_eq!(deductions_shown(&mut again).0.repayment, comparison());
}

// Pub. 525's $3,000 test: at 3,000 the comparison is not offered (no
// deduction or credit), the page refuses it, and a stored comparison is
// erased with the warning; one cent more offers it.
#[test]
fn repayment_comparison_is_offered_only_above_3000() {
    let mut s = Interview::new(TEST_USER, with_recorded_repayment(300_000));
    assert_eq!(deductions_shown(&mut s).1, vec![0]);
    let before = s.draft.clone();
    let request = deductions_request_with(&mut s, |v| v.repayment = comparison());
    assert!(refused(&current_source_refusal(
        "secondary-admission-352",
        &mut s,
        request,
        "MESSAGE_THE_REPAYMENT_COMPARISON_IS_ASKED_ONLY_WHEN_YOUR_INCOME_ENTRIES_RECORD_MORE",
        Some("[value/repayment]"),
        SourceScope::Secondary(SecondaryCorrection::RepaymentChoice)
    )));
    assert_eq!(s.draft, before);
    let mut s = Interview::new(TEST_USER, with_recorded_repayment(300_001));
    assert_eq!(deductions_shown(&mut s).1, vec![0, 1]);
    // Level 1: the repayment reduced to 3,000 takes the comparison away.
    let mut d = with_recorded_repayment(300_000);
    d.deductions.as_mut().unwrap().repayment = comparison();
    let pruned = crate::interview::prune(&mut d);
    assert_eq!(d.deductions.as_ref().unwrap().repayment, RepaymentClaim::None);
    assert!(
        pruned
            .removed
            .iter()
            .any(|item| item.key() == "SAVING_REMOVES_YOUR_EARLIER_YEAR_REPAYMENT_COMPARISON"),
        "{pruned:?}"
    );
}

// Schedule A line 11 with volunteer costs, through the gifts page. Single,
// AGI 60,000, itemizing, 500 of cash gifts on the deductions page. Volunteer
// car travel: 1,000 miles at the statutory 14 cents (140) plus 10 of
// parking: 150, a cash-class contribution (Pub. 526). Cash class 650, less
// the 2026 floor of 0.5% of AGI (300): 350.
#[test]
fn volunteer_costs_reach_schedule_a_through_the_gifts_page() {
    let mut s = itemizing(complete(FederalStatus::Single));
    accepted(&save_deductions_with(&mut s, |v| v.cash_gifts = 50_000));
    let cost = volunteer_travel(VolunteerCarCost::Mileage { miles: 1_000 }, 1_000);
    let saved = cost.clone();
    accepted(&save_gifts_with(&mut s, |v| v.volunteer = vec![saved]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "charity_volunteer_1_expense"), 150);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 350);
    // Round trip, with a 2025 household carryover among the less common.
    accepted(&save_gifts_with(&mut s, |v| v.carryovers = vec![prior_household_carry().into()]));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().charity_carryovers,
        vec![prior_household_carry()]
    );
    let mut again = restart(s.draft.clone());
    assert_eq!(again.draft, s.draft);
    let P::SaveGiftCarryovers { value, .. } = open(&mut again, TaxSection::GiftCarryovers) else {
        unreachable!()
    };
    assert_eq!(value.volunteer, vec![cost]);
    assert_eq!(value.carryovers, vec![CharityCarryoverPage::from(prior_household_carry())]);
}

// The gift page's less common rules, on both sides, and its owner list.
#[test]
fn less_common_gift_rules_are_refused_on_both_sides() {
    let mut s = itemizing(complete(FederalStatus::Joint));
    let household = prior_household_carry();
    rejected_with(
        save_gifts_with(&mut s, |v| {
            v.carryovers = vec![household.clone().into(), household.into()]
        }),
        "CHARITY_CARRY_TWICE",
    );
    let captain = |owner: Owner| {
        let c =
            whaling_claim(vec![whaling_expense(WhalingExpenseKind::CrewAndProvisions, 100_000)]);
        LessCommonGift::WhalingCaptain(WhalingCaptain {
            owner,
            captain_record_reference: c.captain_record_reference,
            aewc_report_reference: c.aewc_report_reference,
            trip_log_reference: c.trip_log_reference,
            expenses: c.expenses,
            eligibility_and_records_confirmed: true,
        })
    };
    rejected_with(
        save_gifts_with(&mut s, |v| {
            v.less_common = vec![captain(Owner::Spouse), captain(Owner::Spouse)]
        }),
        "WHALING_CAPTAIN_TWICE",
    );
    accepted(&save_gifts_with(&mut s, |v| {
        v.less_common = vec![captain(Owner::Taxpayer), captain(Owner::Spouse)]
    }));
    assert_eq!(s.draft.deductions.as_ref().unwrap().whaling_captains.len(), 2);
    // On a single return the spouse is not an allowed captain.
    let mut single = itemizing(complete(FederalStatus::Single));
    assert!(refused(&save_gifts_with(&mut single, |v| {
        v.less_common = vec![captain(Owner::Spouse)]
    })));
}

// The standard deduction takes the itemized pages out of the walk: the
// gifts page's lists go with the warning; the deductions page's own data
// answers (vehicle loans, casualties) stay.
#[test]
fn standard_route_erases_the_gift_page_but_keeps_deductions_page_answers() {
    let mut s = itemizing(complete(FederalStatus::Single));
    accepted(&save_deductions_with(&mut s, |v| {
        v.car_loans = vec![vehicle_loan("Lender", 100_000)];
        v.casualties = vec![casualty("Storm", 100_000).try_into().unwrap()];
    }));
    accepted(&save_gifts_with(&mut s, |v| {
        v.volunteer = vec![volunteer_travel(VolunteerCarCost::Mileage { miles: 10 }, 0)];
        v.carryovers = vec![prior_household_carry().into()];
    }));
    let mut d = s.draft.clone();
    d.deductions.as_mut().unwrap().choice =
        DeductionSelection::Automatic { expenses: ItemizedChoice::No };
    let pruned = crate::interview::prune(&mut d);
    let ded = d.deductions.as_ref().unwrap();
    assert!(ded.volunteer_expenses.is_empty());
    assert!(ded.charity_carryovers.is_empty());
    assert_eq!(ded.vehicle_loans.len(), 1);
    assert_eq!(ded.personal_casualties.len(), 1);
    assert!(
        pruned
            .removed
            .iter()
            .any(|item| item.key() == "SAVING_REMOVES_YOUR_GIFTS_OF_PROPERTY_AND_CARRYOVERS"),
        "{pruned:?}"
    );
}

fn final_return_with_pension() -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { ordinary: true, ..death(20260615).unwrap() });
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        10_000,
        RetirementTreatment::Ordinary,
    );
    e.label = "Synthetic Pension Plan".into();
    if let FederalIncome::Retirement { data } = &mut e.income {
        data.taxable = RetirementTaxable::Payer { amount: 700_000 };
    }
    let id = d.next_id;
    add(&mut d, e);
    (d, id)
}
fn ended_annuity(contract: i64) -> FinalAnnuityLoss {
    FinalAnnuityLoss {
        contract,
        started: 20200101,
        cost: 5_000_000,
        recovered: 1_000_000,
        refunds: 200_000,
        eligible: true,
    }
}

// Schedule A line 17i through the other itemized page. The decedent's
// pension: net investment 50,000, recovered tax free through 2025 10,000,
// a 2,000 tax-free refund, and the 2026 payment's tax-free part 3,000
// (gross 10,000 less the payer's taxable 7,000). Unrecovered investment
// 50,000 - 10,000 - 2,000 - 3,000 = 35,000 (Pub. 575, Pub. 939).
#[test]
fn unrecovered_annuity_cost_reaches_line_17i_through_the_other_itemized_page() {
    let (d, id) = final_return_with_pension();
    let mut s = itemizing(d);
    let P::SaveOtherItemized { value: shown, .. } = open(&mut s, TaxSection::OtherItemized) else {
        unreachable!()
    };
    assert_eq!(
        shown.contracts,
        vec![AnnuityContract {
            id,
            label: libertas::libertas_formatted_text(
                "TEXT",
                &[libertas::LibertasMessageArgument::LiteralText("Synthetic Pension Plan")]
            )
        }]
    );
    accepted(&save_other_with(&mut s, |v| v.annuity_losses = vec![ended_annuity(id)]));
    let stored = &s.draft.deductions.as_ref().unwrap().annuity_losses;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].income_id, id);
    let r = estimate(&s.draft);
    assert_eq!(value(&r, &alloc::format!("annuity_loss_{id}")), 35_000);
    let mut again = restart(s.draft.clone());
    assert_eq!(again.draft, s.draft);
    let P::SaveOtherItemized { value, .. } = open(&mut again, TaxSection::OtherItemized) else {
        unreachable!()
    };
    assert_eq!(value.annuity_losses, vec![ended_annuity(id)]);
}

// The other itemized page's rules and catalog: the recovery and refunds
// cannot pass the cost (equal is allowed), one row per contract, and a
// living filer's pension is not listed.
#[test]
fn annuity_cost_rules_and_catalog() {
    let (d, id) = final_return_with_pension();
    let mut s = itemizing(d);
    // A stale key: the contract is not in the catalog, so the cost is refused.
    let request = other_request_with(&mut s, |v| v.annuity_losses = vec![ended_annuity(id + 1000)]);
    rejected_with(
        current_source_refusal(
            "secondary-admission-307",
            &mut s,
            request,
            "MESSAGE_AN_ANNUITY_COST_NAMES_RETIREMENT_INCOME_THAT_IS_NOT_ON_THIS_RETURN_CHOOSE_IT",
            Some("[value/annuity_losses/0/contract]"),
            SourceScope::Secondary(SecondaryCorrection::AnnuitySource),
        ),
        "MESSAGE_AN_ANNUITY_COST_NAMES_RETIREMENT_INCOME_THAT_IS_NOT_ON_THIS_RETURN_CHOOSE_IT",
    );
    rejected_with(
        save_other_with(&mut s, |v| {
            v.annuity_losses =
                vec![FinalAnnuityLoss { recovered: 5_000_001, refunds: 0, ..ended_annuity(id) }]
        }),
        "ANNUITY_RECOVERED_ABOVE_COST",
    );
    rejected_with(
        save_other_with(&mut s, |v| {
            v.annuity_losses = vec![FinalAnnuityLoss { refunds: 4_000_001, ..ended_annuity(id) }]
        }),
        "ANNUITY_REFUNDS_ABOVE_COST",
    );
    rejected_with(
        save_other_with(&mut s, |v| v.annuity_losses = vec![ended_annuity(id), ended_annuity(id)]),
        "ANNUITY_CONTRACT_TWICE",
    );
    accepted(&save_other_with(&mut s, |v| {
        v.annuity_losses = vec![FinalAnnuityLoss { refunds: 4_000_000, ..ended_annuity(id) }]
    }));
    // Recovered in full: nothing is left to deduct.
    let mut r = estimate(&s.draft);
    assert_eq!(crate::federal_rules::final_annuity_losses(&s.draft, &mut r), 0);
    // A living filer: no catalog, and a stored row is erased.
    let (mut d, _) = final_return_with_pension();
    d.people.as_mut().unwrap().taxpayer.died = None;
    let mut living = itemizing(d);
    let P::SaveOtherItemized { value, .. } = open(&mut living, TaxSection::OtherItemized) else {
        unreachable!()
    };
    assert!(value.contracts.is_empty());
    let mut d = s.draft.clone();
    d.people.as_mut().unwrap().taxpayer.died = None;
    let pruned = crate::interview::prune(&mut d);
    assert!(d.deductions.as_ref().unwrap().annuity_losses.is_empty());
    assert!(
        pruned
            .removed
            .iter()
            .any(|item| item.key() == "SAVING_REMOVES_YOUR_UNRECOVERED_ANNUITY_COST"),
        "{pruned:?}"
    );
}

// The deduction, credit and payment pages holding AI Import forms (1098,
// 1098-E, 1098-T, 1098-C, 1098-VLI, 1097-BTC, 8986, 1095-A) each carry the
// hidden year fact their import checks.
#[test]
fn deduction_and_credit_editors_supply_hidden_document_year() {
    let mut itemizer = itemizing(complete(FederalStatus::Single));
    let mut standard = Interview::new(TEST_USER, complete(FederalStatus::Single));
    for (itemized, section) in [
        (true, TaxSection::Mortgage),
        (true, TaxSection::GiftCarryovers),
        (false, TaxSection::StudentLoan),
        (false, TaxSection::Deductions),
        (false, TaxSection::Education),
        (false, TaxSection::MoreCredits),
        (false, TaxSection::Marketplace),
    ] {
        let s = if itemized { &mut itemizer } else { &mut standard };
        let facts = match from_review(s, section) {
            Reply::Edit(
                P::SaveMortgages { document_facts, .. }
                | P::SaveStudentLoan { document_facts, .. }
                | P::SaveEducation { document_facts, .. }
                | P::SaveGiftCarryovers { document_facts, .. }
                | P::SaveDeductions { document_facts, .. }
                | P::SaveMoreCredits { document_facts, .. }
                | P::SaveMarketplace { document_facts, .. },
            ) => document_facts,
            other => panic!("expected the {section:?} editor, got {other:?}"),
        };
        assert_eq!(facts, DocumentFacts { year: 2026 }, "{section:?}");
    }
}

// Form 4952 (2026 draft, Created 5/28/26), lines 1–8, and its separate
// AMT worksheet. $1,000 dividends less $500 qualified plus $400 elected
// gives $900 income. Regular costs $1,500+$300 leave $900 carry; separate
// AMT costs $1,500+$600+$200 leave $1,400. No AMT carry is inferred.
#[test]
fn investment_borrowing_page_keeps_complete_facts_across_deduction_routes() {
    let mut s = itemizing(complete(FederalStatus::Single));
    let mut page =
        crate::tests::cross::interview::open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    documents.documents.push(crate::tests::cross::interview::document(
        0,
        IncomeDocument::Div1099(native_div1099("Synthetic Portfolio Fund".into(), 100_000, 50_000)),
    ));
    accepted(&send(&mut s, page));
    let borrowing = InvestmentBorrowingPage {
        loans: vec![InvestmentLoanPage {
            paid_owners: None,
            carry_owners: None,
            originating_k1: None,
            allowed_origin_choices: vec![],
            label: "Portfolio margin loan".into(),
            source: 0,
            paid: 150_000,
            carry: 30_000,
            amt_carry: 60_000,
            private_activity: 20_000,
        }],
        election: InvestmentElection::Allocated { capital: 0, dividends: 40_000 },
        income_use: InvestmentInterestUse::Mixed { paid: 30_000, carryover: 10_000 },
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    };
    accepted(&save_other_with(&mut s, |v| v.investment_interest = Some(borrowing.clone())));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "investment_interest_deduction"), 900);
    assert_eq!(value(&r, "investment_interest_carryforward"), 900);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 900);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 1400);
    let mut restarted = restart(s.draft.clone());
    let P::SaveOtherItemized { value: shown, .. } = open(&mut restarted, TaxSection::OtherItemized)
    else {
        unreachable!()
    };
    assert_eq!(shown.investment_interest, Some(borrowing.clone()));
    accepted(&save_deductions_with(&mut restarted, |v| v.route = DeductionRoute::Standard));
    assert!(matches!(
        restarted.draft.deductions.as_ref().unwrap().choice,
        DeductionSelection::Standard
            | DeductionSelection::Automatic { expenses: ItemizedChoice::No }
    ));
    let page = open(&mut restarted, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value: shown, page: parent, .. } = &page else { unreachable!() };
    assert_eq!(*parent, TaxSection::Deductions as i32);
    assert_eq!(shown.investment_interest, Some(borrowing));
    accepted(&send(&mut restarted, page));
    assert_eq!(value(&supported(&restarted.draft), "investment_interest_deduction"), 0);
    assert_eq!(value(&supported(&restarted.draft), "investment_elected_qualified_dividends"), 400);
}

#[test]
fn investment_borrowing_save_rejects_stale_source_catalog_and_keeps_the_draft() {
    let mut s = itemizing(complete(FederalStatus::Single));
    let mut page = open(&mut s, TaxSection::OtherItemized);
    let before = s.draft.clone();
    let dirty = s.dirty;
    let P::SaveOtherItemized { interest_sources, .. } = &mut page else { unreachable!() };
    interest_sources.clear();
    rejected_with(
        current_source_refusal(
            "secondary-admission-305",
            &mut s,
            page,
            "INVESTMENT_BORROWING_RECORDS",
            None,
            SourceScope::Fieldless,
        ),
        "INVESTMENT_BORROWING_RECORDS",
    );
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
    let mut page = open(&mut s, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value, .. } = &mut page else { unreachable!() };
    value.investment_interest = Some(InvestmentBorrowingPage {
        loans: vec![InvestmentLoanPage {
            paid_owners: None,
            carry_owners: None,
            originating_k1: None,
            allowed_origin_choices: vec![],
            label: "Deleted source".into(),
            source: 999,
            paid: 100_000,
            carry: 0,
            amt_carry: 0,
            private_activity: 0,
        }],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
    rejected_with(
        current_source_refusal(
            "secondary-admission-308",
            &mut s,
            page,
            "INVESTMENT_BORROWING_RECORDS",
            Some("[value/investment_interest/loans/0/source]"),
            SourceScope::Secondary(SecondaryCorrection::InvestmentSource),
        ),
        "INVESTMENT_BORROWING_RECORDS",
    );
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
}

// Form4952 line5 uses the allowed directly connected deduction, not every
// income-tax payment or a second tax expense. $10,000 investment income less
// $3,000 directly connected tax gives $7,000 regular allowance/$5,000 carry.
// State taxes aren't AMT expenses: $10,000 AMT allowance/$2,000 AMT carry.
#[test]
fn investment_income_tax_page_drives_one_regular_expense_and_preserves_amt() {
    let mut s = itemizing(complete(FederalStatus::Single));
    let mut page =
        crate::tests::cross::interview::open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    documents.documents.push(crate::tests::cross::interview::document(
        0,
        IncomeDocument::Int1099(native_int1099("Synthetic Investment Bank".into(), 1_000_000)),
    ));
    accepted(&send(&mut s, page));
    accepted(&save_other_with(&mut s, |v| {
        v.investment_interest = Some(InvestmentBorrowingPage {
            loans: vec![InvestmentLoanPage {
                paid_owners: None,
                carry_owners: None,
                originating_k1: None,
                allowed_origin_choices: vec![],
                label: "Portfolio loan".into(),
                source: 0,
                paid: 1_200_000,
                carry: 0,
                amt_carry: 0,
                private_activity: 0,
            }],
            election: InvestmentElection::None,
            income_use: InvestmentInterestUse::Ordinary,
            limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
        })
    }));
    let mut page = open(&mut s, TaxSection::Taxes);
    let P::SaveTaxes { value: taxes, .. } = &mut page else { unreachable!() };
    taxes.kind = SaltElection::IncomeTax { paid: 600_000 };
    taxes.investment_income_tax = InvestmentIncomeTax::Records {
        paid_part: 300_000,
        capped_allocation: InvestmentTaxCapAllocation::SourceDetermined,
    };
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 70_000);
    assert_eq!(value(&r, "investment_interest_deduction"), 7000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 5000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 2000);
    assert!(!noted(&r, "NOTE_INVESTMENT_TAX_ALLOCATION"));
    let mut again = restart(s.draft.clone());
    let page = open(&mut again, TaxSection::Taxes);
    let P::SaveTaxes { value: shown, .. } = &page else { unreachable!() };
    assert_eq!(
        shown.investment_income_tax,
        InvestmentIncomeTax::Records {
            paid_part: 300_000,
            capped_allocation: InvestmentTaxCapAllocation::SourceDetermined
        }
    );
    accepted(&send(&mut again, page));
    assert_eq!(estimate(&again.draft), r);
    let before = again.draft.clone();
    let dirty = again.dirty;
    for kind in
        [SaltElection::IncomeTax { paid: 299_999 }, SaltElection::SalesTax { paid: 600_000 }]
    {
        let mut page = open(&mut again, TaxSection::Taxes);
        let P::SaveTaxes { value: taxes, .. } = &mut page else { unreachable!() };
        taxes.kind = kind;
        rejected_with(send(&mut again, page), "INVESTMENT_TAX_ABOVE_PAID");
        assert_eq!(again.draft, before);
        assert_eq!(again.dirty, dirty);
    }
}

// Derived from the Schedule D carryover worksheet and 2026 draft Form4952
// lines4d-4f, not a published complete-return example. The ordinary carry is
// $4,000 ST/$10,000 LT; only $1,000/$2,000 came from investment property.
// Separate AMT carry is $6,000/$20,000, of which $2,000/$3,000 is investment.
// Current $20,000 ST investment gain gives $17,000 regular/$15,000 AMT
// investment income. $18,000 interest leaves $1,000/$3,000 future interest.
#[test]
fn investment_capital_portions_use_public_source_facts_and_independent_amt_caps() {
    use crate::tests::cross::interview::{document, open_topic, page_documents, save_topic};
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut prior = capital_history();
    prior.investment_parts = Some(InvestmentCapitalParts {
        short: 100_000,
        long: 200_000,
        amt: Some(InvestmentAmtCapitalParts { short: 200_000, long: 300_000 }),
    });
    prior.amt = Some(PriorAmtCapitalReturn {
        taxable_income: 10_000_000,
        short: -900_000,
        long: -2_000_000,
        deducted_loss: 300_000,
    });
    let sale = Sale1099B {
        digital_asset: false,
        market_discount: 0,
        several_sales: false,
        state_details: Default::default(),
        description: "Investment sale".into(),
        term: Some(GainTerm::Short),
        proceeds: 2_000_000,
        basis: Some(0),
        reporting: BasisReporting::NotReported,
        use_kind: SaleUse::Investment { wash_sale: 0 },
        collectible: false,
        withholding: 0,
        adjustment: SaleAdjustment::None,
        special: SaleSpecial::NoneOfThese,
    };
    let request = save_topic(
        &s.draft,
        TaxSection::InvestmentSales,
        vec![
            document(0, IncomeDocument::Sale1099B(sale)),
            document(0, IncomeDocument::CapitalLossCarryover(prior.clone().into())),
        ],
    );
    accepted(&send(&mut s, request));
    accepted(&save_deductions_with(&mut s, |v| v.route = DeductionRoute::Itemize));
    accepted(&save_other_with(&mut s, |v| {
        v.investment_interest = Some(InvestmentBorrowingPage {
            loans: vec![InvestmentLoanPage {
                paid_owners: None,
                carry_owners: None,
                originating_k1: None,
                allowed_origin_choices: vec![],
                label: "Portfolio".into(),
                source: 0,
                paid: 1_800_000,
                carry: 0,
                amt_carry: 0,
                private_activity: 0,
            }],
            election: InvestmentElection::None,
            income_use: InvestmentInterestUse::Ordinary,
            limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
        })
    }));
    accepted(&save_deductions_with(&mut s, |v| v.route = DeductionRoute::Itemize));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain"), 6000);
    assert_eq!(value(&r, "investment_interest_deduction"), 17000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 1000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 15000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 3000);
    assert!(!noted(&r, "NOTE_INVESTMENT_CAPITAL_PARTS"));
    let mut s = restart(s.draft.clone());
    let page = open_topic(&mut s, TaxSection::InvestmentSales);
    let rows = page_documents(&page);
    let row = rows
        .iter()
        .find(|row| matches!(row.document, IncomeDocument::CapitalLossCarryover(_)))
        .unwrap();
    let expected_caps = InvestmentCapitalCap {
        short: 400_000,
        long: 1_000_000,
        amt_short: 600_000,
        amt_long: 2_000_000,
    };
    assert_eq!(row.investment_caps, Some(expected_caps.clone()));
    let before = s.draft.clone();
    let before_dirty = s.dirty;
    // Forged preview caps cannot enlarge the accepted source allowance.
    let mut forged = page.clone();
    let P::SaveIncomes { documents, .. } = &mut forged else { panic!() };
    let row = documents
        .documents
        .iter_mut()
        .find(|row| matches!(row.document, IncomeDocument::CapitalLossCarryover(_)))
        .unwrap();
    row.investment_caps = Some(InvestmentCapitalCap { short: 10_000_000, ..expected_caps.clone() });
    let IncomeDocument::CapitalLossCarryover(v) = &mut row.document else { panic!() };
    v.investment_parts.as_mut().unwrap().short = 400_001;
    assert!(refused(&send(&mut s, forged)));
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, before_dirty);
    // The actual edited source, not the echoed cap, owns pure calculation.
    let mut edited = prior;
    edited.short = -600_000; // unchanged taxable worksheet: ST carry $5,000.
    let reply =
        send(&mut s, survey_preview_document(IncomeDocument::CapitalLossCarryover(edited.into())));
    let Reply::Response(P::IncomePreview { preview: None, investment_caps: Some(caps) }) = reply
    else {
        panic!("{reply:?}")
    };
    assert_eq!(
        caps,
        InvestmentCapitalCap {
            short: 500_000,
            long: 1_000_000,
            amt_short: 600_000,
            amt_long: 2_000_000
        }
    );
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, before_dirty);
    accepted(&send(&mut s, page));
    let after = supported(&s.draft);
    assert_eq!(after.tax, r.tax);
    assert_eq!(value(&after, "investment_interest_deduction"), 17000);
    assert_eq!(value(&after, "investment_interest_carryforward"), 1000);
    assert_eq!(value(&after, "amt_investment_interest_deduction"), 15000);
    assert_eq!(value(&after, "amt_investment_interest_carryforward"), 3000);
}

// 2026 draft f4952 printed pp.3-4, lines4a/5/8; derived destination fixtures.
// The IRS does not publish these synthetic dollar examples. Public saves
// retain one borrowing record; Schedule E interest changes AGI before limits.
fn royalty_borrowing_page(
    s: &mut Interview,
    portfolio: i64,
    royalty_paid: i64,
    royalty_carry: i64,
    royalty_amt_carry: i64,
) {
    let source = s
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Royalty { .. }))
        .unwrap()
        .id;
    let mut loans = vec![InvestmentLoanPage {
        paid_owners: None,
        carry_owners: None,
        originating_k1: None,
        allowed_origin_choices: vec![],
        label: "Royalty acquisition loan".into(),
        source,
        paid: royalty_paid * 100,
        carry: royalty_carry * 100,
        amt_carry: royalty_amt_carry * 100,
        private_activity: 0,
    }];
    if portfolio != 0 {
        loans.push(InvestmentLoanPage {
            paid_owners: None,
            carry_owners: None,
            originating_k1: None,
            allowed_origin_choices: vec![],
            label: "Portfolio margin loan".into(),
            source: 0,
            paid: portfolio * 100,
            carry: 0,
            amt_carry: 0,
            private_activity: 0,
        });
    }
    accepted(&save_other_with(s, |v| {
        v.investment_interest = Some(InvestmentBorrowingPage {
            loans,
            election: InvestmentElection::None,
            income_use: InvestmentInterestUse::Ordinary,
            limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
        })
    }));
}
fn royalty_page_return(receipts: i64, costs: i64) -> Interview {
    use crate::tests::cross::interview::{document, save_topic};
    let mut s = itemizing(complete(FederalStatus::Single));
    let page = save_topic(
        &s.draft,
        TaxSection::Rentals,
        vec![document(
            0,
            IncomeDocument::Royalties(RoyaltyProperty {
                kind: crate::RoyaltyKind::Intangible,
                receipts: receipts * 100,
                advance: 0,
                expenses: RentalExpenses {
                    legal: costs * 100,
                    ..crate::tests::cross::interview::no_rental_expenses()
                },
                depletion: crate::RoyaltyDepletion::No,
                special: false,
            }),
        )],
    );
    accepted(&send(&mut s, page));
    s
}
#[test]
fn investment_royalty_source_routes_once_and_survives_standard_deduction() {
    // Gross royalties 11,000; allowed collection cost1,000; capacity10,000.
    // Portfolio4,000 + royalty2,000 are fully allowed. Schedule A9=4,000;
    // Schedule E13=2,000, E26=8,000; AGI=60,000+8,000=68,000.
    let mut s = royalty_page_return(11_000, 1_000);
    royalty_borrowing_page(&mut s, 4_000, 2_000, 0, 0);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "investment_form_line4a"), 11_000);
    assert_eq!(value(&r, "investment_form_line5"), 1_000);
    assert_eq!(value(&r, "investment_form_line8"), 6_000);
    assert_eq!(value(&r, "investment_interest_deduction"), 4_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 8_000);
    assert_eq!(value(&r, "agi"), 68_000);
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Royalty { .. }))
        .unwrap()
        .id;
    assert_eq!(value(&r, &format!("royalty_{source}_line13_interest")), 2_000);
    assert_eq!(value(&r, &format!("investment_source_{source}_carryforward")), 0);
    assert_eq!(printed_line(&r, "investment_form_line8"), (TaxForm::Form4952, "8".into()));
    let before = s.draft.clone();
    s = restart(before.clone());
    accepted(&save_other_with(&mut s, |_| {}));
    assert_eq!(s.draft, before);
    accepted(&save_deductions_with(&mut s, |v| v.route = DeductionRoute::Standard));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "investment_form_line8"), 6_000);
    assert_eq!(value(&r, "investment_form_line5"), 1_000);
    assert_eq!(value(&r, "agi"), 68_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 8_000);
    assert!(!noted(&r, "NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"));
}
#[test]
fn investment_exclusive_royalty_keeps_separate_amt_carry_and_line2c() {
    // Capacity3,000. Regular cost1,000+5,000carry=>allowed3,000/carry3,000;
    // AMT cost1,000+500carry=>allowed1,500/carry0. Standard deduction does
    // not remove the royalty deduction; Form6251 2c=+1,500 (i6251 p.3).
    let mut s = royalty_page_return(4_000, 1_000);
    royalty_borrowing_page(&mut s, 0, 1_000, 5_000, 500);
    accepted(&save_deductions_with(&mut s, |v| v.route = DeductionRoute::Standard));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "investment_form_line8"), 3_000);
    assert_eq!(value(&r, "investment_form_line7"), 3_000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 0);
    assert_eq!(value(&r, "amt_investment_form_line1"), 1_000);
    assert_eq!(value(&r, "amt_investment_form_line2"), 500);
    assert_eq!(value(&r, "amt_investment_form_line3"), 1_500);
    assert_eq!(value(&r, "amt_investment_form_line4a"), 4_000);
    assert_eq!(value(&r, "amt_investment_form_line5"), 1_000);
    assert_eq!(value(&r, "amt_investment_form_line6"), 3_000);
    assert_eq!(value(&r, "amt_investment_form_line7"), 0);
    assert_eq!(value(&r, "amt_investment_form_line8"), 1_500);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 61_500);
    assert_eq!(value(&r, "amt_investment_interest_adjustment"), 1_500);
    assert_eq!(
        printed_line(&r, "amt_investment_interest_adjustment"),
        (TaxForm::Form6251, "2c".into())
    );
}
#[test]
fn investment_limited_mixed_destinations_leave_only_unavailable_benefit_out() {
    // Capacity3,000 against4,000portfolio+2,000royalty cannot determine
    // which source retains the3,000carry. An affirmative source enum is not
    // an allocation record. No guessed deduction or computed carry is shown.
    let mut s = royalty_page_return(4_000, 1_000);
    royalty_borrowing_page(&mut s, 4_000, 2_000, 0, 0);
    let r = supported(&s.draft);
    assert!(noted(&r, "NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"));
    assert_eq!(value(&r, "agi"), 63_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 3_000);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert!(!r.lines.iter().any(
        |row| row.id == "investment_form_line7" || row.id == "investment_interest_carryforward"
    ));
    let retained = s.draft.clone();
    s = restart(retained.clone());
    accepted(&save_other_with(&mut s, |_| {}));
    assert_eq!(s.draft, retained);
    // Removing only the other-source borrowing resolves the exclusive route.
    accepted(&save_other_with(&mut s, |v| {
        v.investment_interest.as_mut().unwrap().loans.retain(|loan| loan.source != 0)
    }));
    let r = supported(&s.draft);
    assert!(!noted(&r, "NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"));
    assert_eq!(value(&r, "agi"), 61_000);
    assert_eq!(value(&r, "investment_form_line8"), 2_000);
}

#[test]
fn investment_zero_capacity_determines_each_source_carry_without_priority() {
    // Gross royalties1,000 minus allowed costs1,000 gives zero capacity.
    // Mixed sources need no invented priority: both allowances are zero,
    // and each source retains all its own regular/AMT costs.
    let mut s = royalty_page_return(1_000, 1_000);
    royalty_borrowing_page(&mut s, 4_000, 2_000, 300, 500);
    let r = supported(&s.draft);
    assert!(!noted(&r, "NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"));
    assert_eq!(value(&r, "investment_form_line6"), 0);
    assert_eq!(value(&r, "investment_form_line7"), 6_300);
    assert_eq!(value(&r, "amt_investment_form_line7"), 6_500);
    assert_eq!(value(&r, "investment_source_0_carryforward"), 4_000);
    let id = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Royalty { .. }))
        .unwrap()
        .id;
    assert_eq!(value(&r, &format!("investment_source_{id}_carryforward")), 2_300);
    assert_eq!(value(&r, &format!("investment_source_{id}_amt_carryforward")), 2_500);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(estimate(&restart(s.draft).draft), r);
}

#[test]
fn investment_untyped_bond_premium_preserves_ordinary_deduction_and_income() {
    // Baseline BondPremium stores no pre1986/annual/terminal character.
    // Its ordinary Schedule A17f amount stays1,200; no guessed4952line5
    // expense or second premium amount is manufactured from that number.
    let mut s = royalty_page_return(4_000, 1_000);
    royalty_borrowing_page(&mut s, 0, 2_000, 0, 0);
    accepted(&save_other_with(&mut s, |v| {
        v.other_lines.push(OtherItemizedInput::BondPremium {
            source: BondPremiumSelection::OtherDocumented {
                instrument: "Untyped documented bond premium".into(),
            },
            amount: 120_000,
        });
    }));
    let r = supported(&s.draft);
    assert!(noted(&r, "NOTE_INVESTMENT_PREMIUM_CHARACTER"));
    assert_eq!(value(&r, "bond_premium_deduction"), 1_200);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 3_000);
    assert_eq!(value(&r, "agi"), 63_000);
    assert!(!r.lines.iter().any(|row| row.id == "investment_form_line7"));
    let retained = s.draft.clone();
    s = restart(retained.clone());
    accepted(&save_other_with(&mut s, |_| {}));
    assert_eq!(s.draft, retained);
    assert_eq!(supported(&s.draft), r);
}

#[test]
fn investment_royalty_gross_and_allowed_cost_round_on_separate_printed_lines() {
    use crate::tests::cross::interview::{document, save_topic};
    // Gross4,000.49=>4,000 and allowed costs1,000.50=>1,001 give
    // Form4952capacity2,999, not3,000 from rounded-net plus rounded-cost.
    // Existing Schedule E net is rounded from actual cents to3,000;
    // interest2,999 leaves1 of royalty income and1 of interest carry.
    let mut s = itemizing(complete(FederalStatus::Single));
    let page = save_topic(
        &s.draft,
        TaxSection::Rentals,
        vec![document(
            0,
            IncomeDocument::Royalties(RoyaltyProperty {
                kind: crate::RoyaltyKind::Intangible,
                receipts: 400_049,
                advance: 0,
                expenses: RentalExpenses {
                    legal: 100_050,
                    ..crate::tests::cross::interview::no_rental_expenses()
                },
                depletion: crate::RoyaltyDepletion::No,
                special: false,
            }),
        )],
    );
    accepted(&send(&mut s, page));
    royalty_borrowing_page(&mut s, 0, 3_000, 0, 0);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "investment_form_line4a"), 4_000);
    assert_eq!(value(&r, "investment_form_line5"), 1_001);
    assert_eq!(value(&r, "investment_form_line6"), 2_999);
    assert_eq!(value(&r, "investment_form_line8"), 2_999);
    assert_eq!(value(&r, "investment_form_line7"), 1);
    assert_eq!(value(&r, "amt_investment_form_line8"), 2_999);
    assert_eq!(value(&r, "agi"), 60_001);
    assert_eq!(estimate(&restart(s.draft).draft), r);
}
