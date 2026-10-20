// IRA and Roth histories, Form 8606 and Form 5329 corner cases (unit B11 IRA, map
// tax-forms/ira-roth-histories): the twenty fixtures F1-F20 with their hand arithmetic from the 2026
// draft Instructions for Forms 8606 and 5329, Pub. 590-A/B and the Forms 1099-R instructions.
// Amounts in the comments are dollars (the records carry cents). Unless stated: single filer, one
// $60,000 W-2 with $6,500 withheld, box 4 0, every confirmation Yes.
use super::*;
use crate::tests::cross::interview::{accepted, document, open_topic, save_topic};
use crate::{
    ExceptionRepayment, IraKind, IraRepaymentDeposit, LateCorrection, ReservistPayment,
    ReservistRepayment, ReturnedContribution, SimpleDisposition,
};
use alloc::format;

fn filer(birth: u32) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    d
}

// One 1099-R in dollars: box 1, box 2a, the printed box 7a and box 7b and the branch.
fn form(printed: &str, ira: bool, gross: i64, taxable: i64, code: DistributionCode) -> R1099 {
    let mut r = printed_as(crate::tests::cross::interview::r1099(gross * 100, code, ira), printed);
    r.taxable = taxable * 100;
    r
}
fn normal() -> DistributionCode {
    DistributionCode::Normal { what_happened: Disposition::Kept }
}
fn doc(r: R1099) -> IncomeDocumentItem {
    document(0, IncomeDocument::R1099(r))
}
fn form_8606_item(basis: i64, adjustment: i64, year_end: i64) -> IncomeDocumentItem {
    document(
        0,
        IncomeDocument::Form8606(Form8606 {
            basis: basis * 100,
            adjustment: adjustment * 100,
            year_end: year_end * 100,
            reservist_repayments: vec![],
        }),
    )
}
fn roth_history_item(
    first_year: u32,
    regular: i64,
    conversions: Vec<(u32, i64, i64)>,
) -> IncomeDocumentItem {
    document(
        0,
        IncomeDocument::RothHistory(RothIraHistory {
            first_year,
            amounts: Some(RothOrdering {
                regular: regular * 100,
                conversions: conversions
                    .into_iter()
                    .map(|(year, taxable, nontaxable)| RothConversion {
                        year,
                        taxable: taxable * 100,
                        nontaxable: nontaxable * 100,
                    })
                    .collect(),
            }),
        }),
    )
}
fn roth_row(
    gross: i64,
    printed: &str,
    ira: bool,
    roth: RothIraCode,
    rolled_over: i64,
) -> IncomeDocumentItem {
    doc(form(
        printed,
        ira,
        gross,
        gross,
        DistributionCode::RothIra { sequence: None, roth, rolled_over: rolled_over * 100 },
    ))
}
fn early_j(distributed: u32, exception: Option<RetirementException>) -> RothIraCode {
    RothIraCode::Early { distributed, exception, repaid: None }
}
fn save(d: &FederalDraft, documents: Vec<IncomeDocumentItem>) -> Interview {
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&send(&mut s, save_topic(d, TaxSection::Retirement, documents)));
    s
}
fn refusal(d: &FederalDraft, documents: Vec<IncomeDocumentItem>) -> Vec<u8> {
    let mut s = Interview::new(TEST_USER, d.clone());
    let Reply::Response(P::Problem { message, .. }) =
        send(&mut s, save_topic(d, TaxSection::Retirement, documents))
    else {
        panic!("expected a refused save")
    };
    message
}
fn key(k: &str) -> Vec<u8> {
    libertas::libertas_formatted_text(k, &[])
}
fn has_note(r: &FederalResult, key: &str) -> bool {
    r.review_notes.iter().any(|n| n.display.key == key)
}
fn has_issue(r: &FederalResult, key: &str) -> bool {
    r.issues.iter().any(|i| i.display.key == key)
}
fn early(r: &FederalResult, suffix: &str) -> i64 {
    r.lines
        .iter()
        .find(|l| l.id == format!("early_taxpayer_{suffix}"))
        .map_or(0, |l| l.amount / 100)
}
fn tax(r: &FederalResult) -> i64 {
    r.lines.iter().find(|l| l.id == "early_distribution_tax").map_or(0, |l| l.amount / 100)
}
fn excess_page(
    traditional_value: i64,
    roth_value: i64,
    prior_traditional: i64,
    returned_prior_excess: i64,
) -> IraExcess {
    IraExcess {
        traditional_value: Some(traditional_value * 100),
        roth_value: roth_value * 100,
        prior_traditional: prior_traditional * 100,
        prior_roth: 0,
        returned_prior_excess: returned_prior_excess * 100,
    }
}

// F1: Form 5329 line 10 absorbs a carried traditional excess (R3). Owner born 1980-05-05,
// compensation 60,000, not covered. Excess record: prior_traditional 2,000 (2025 line 16; line 17
// showed 120), traditional value 40,000. 2026 traditional contributions 5,000. Line 9 2,000; line 10
// = 7,500 - 5,000 = 2,500; lines 11 and 12 0; line 13 2,500; line 14 0; line 17 0. Absorbed =
// min(2,500, 2,000) = 2,000 -> IRA Deduction Worksheet line 11: 5,000 + 2,000 = 7,000 = Schedule 1
// line 20. (b) Covered at work, MAGI 85,000: phase_limit(7,500, 85,000, 81,000, 10,000) = 4,500:
// deduction 4,500, Form 8606 line 1 = 7,000 - 4,500 = 2,500. (c) Contributions 7,500: line 10 0,
// line 14 2,000, line 16 2,000, line 17 = 6% x min(2,000, 40,000) = 120.
#[test]
fn f1_line_10_absorbs_a_carried_traditional_excess_into_the_deduction() {
    let mut d = filer(19800505);
    let mut facts = ira_facts(500_000, 0, Answer::No);
    facts.taxpayer.excess = Some(excess_page(40_000, 0, 2_000, 0));
    d.adjustments.as_mut().unwrap().iras = facts.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7_000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line10_unused_traditional_room"), 2_500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 0);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 0);
    // (b) Covered at work with MAGI 85,000: 4,500 deductible, 2,500 nondeductible basis.
    d.income[0] = covered_wage(1, Owner::Taxpayer, 8_500_000, 0);
    facts.taxpayer.covered_at_work = true;
    d.adjustments.as_mut().unwrap().iras = facts;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 4_500);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 2_500);
    // (c) The whole limit used: nothing to absorb, 6% of the 2,000 carried.
    let mut d = filer(19800505);
    let mut facts = ira_facts(750_000, 0, Answer::No);
    facts.taxpayer.excess = Some(excess_page(40_000, 0, 2_000, 0));
    d.adjustments.as_mut().unwrap().iras = facts;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7_500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line14_prior_traditional_remaining"), 2_000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 120);
}

#[test]
fn print_ira_excess_keeps_filer_copies_and_absorbed_prior_excess_subtotal() {
    // Taxpayer: prior2000 is absorbed by7500-5000=2500 unused room.
    // Spouse: prior1000 plus current8000-7500=500 leaves1500;6%=90.
    // Each files their own PartIII. Neither has a Roth excess requiring PartIV.
    let mut d = complete(FederalStatus::Joint);
    let mut facts = ira_facts(500_000, 0, Answer::No);
    facts.taxpayer.excess = Some(excess_page(40_000, 0, 2_000, 0));
    let mut spouse = facts.taxpayer.clone();
    spouse.traditional = 800_000;
    spouse.excess = Some(excess_page(40_000, 0, 1_000, 0));
    facts.spouse = IraSpouse::Joint { facts: spouse };
    d.adjustments.as_mut().unwrap().iras = facts;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_excess_taxpayer_line13_traditional_total_reduction"), 2500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 0);
    assert_eq!(value(&r, "ira_excess_spouse_line17_traditional_excess_tax"), 90);
    let review: Vec<_> = crate::draft_return(&r)
        .into_iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form5329))
        .collect();
    let starts: Vec<_> = review
        .iter()
        .enumerate()
        .filter_map(|(index, row)| (row.form_line.as_ref().unwrap().line == "9").then_some(index))
        .collect();
    assert_eq!(starts.len(), 2);
    assert!(
        review[starts[0]..starts[1]]
            .iter()
            .any(|row| { row.form_line.as_ref().unwrap().line == "13" && row.amount == 250_000 })
    );
    assert!(
        review[starts[1]..]
            .iter()
            .any(|row| { row.form_line.as_ref().unwrap().line == "17" && row.amount == 9_000 })
    );
    assert!(!review.iter().any(|row| {
        (18..=25).contains(&row.form_line.as_ref().unwrap().line.parse::<u32>().unwrap())
    }));
    // An optional value record with no prior/current excess creates no form.
    let mut ordinary = complete(FederalStatus::Single);
    let mut facts = ira_facts(500_000, 0, Answer::No);
    facts.taxpayer.excess = Some(excess_page(40_000, 0, 0, 0));
    ordinary.adjustments.as_mut().unwrap().iras = facts;
    let r = supported(&ordinary);
    assert!(
        !r.lines.iter().any(|row| { row.id.starts_with("ira_excess_") && row.form_line.is_some() })
    );
}

// F2: line 12, a pre-2026 excess paid back in 2026 (R3). Owner born 1966-02-01, no compensation;
// 2025 Form 5329 line 16 7,000 (never deducted, within those years' limits), line 17 420. Excess
// record: prior_traditional 7,000, returned_prior_excess 7,000, traditional value 40,000. Form
// 1099-R code 7, box 7b, box 1 = box 2a 7,000. Line 9 7,000; line 10 0; line 11 0; line 12 7,000;
// line 13 7,000; line 14 0; line 17 0. Form 1040 line 4a 7,000, line 4b 0; no Form 8606 line 7.
// (b) Box 1 10,000: line 4b 3,000, line 11 3,000. (c) 8,000 returned: IRA_RETURNED_EXCESS_ABOVE_CARRIED.
// (d) 7,000 entered with box 1 5,000: IRA_RETURNED_EXCESS_ABOVE_DISTRIBUTIONS (the IRA page rules are
// in tests/interview.rs).
#[test]
fn f2_a_pre_2026_excess_paid_back_in_2026_is_not_income_or_form_8606_distribution() {
    let mut d = filer(19660201);
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    let mut facts = ira_facts(0, 0, Answer::No);
    facts.taxpayer.excess = Some(excess_page(40_000, 0, 7_000, 7_000));
    d.adjustments.as_mut().unwrap().iras = facts.clone();
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            7_000,
            RetirementTreatment::Ordinary,
        ),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 700_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line12_returned_prior_excess"), 7_000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line14_prior_traditional_remaining"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 0);
    // (b) 10,000 withdrawn: 3,000 is ordinary income and line 11.
    d.income.pop();
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10_000,
            RetirementTreatment::Ordinary,
        ),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 300_000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line11_traditional_distributions"), 3_000);
}

// F3: a 2026 Roth contribution returned in 2027 (R2). Owner born 1990-03-03, wages 200,000 (Roth
// limit 0). 7,500 contributed on 2026-06-01, returned 2027-03-10 with 400 earnings. IRA page: Roth 0;
// returned_in_following_year: Roth, principal 7,500, earnings 400. Line 4a 7,900, line 4b 400, no Form 5329
// Part I (exception 21), Part IV line 23 0. (b) Earnings -200: 4a 7,300, 4b 0. (c) Returned 2027-06-01
// with no route: IRA_RETURN_AFTER_DUE_DATE_NEEDS_ROUTE; with the timely-return route: accepted with
// NOTE_IRA_RETURN_AMENDED.
#[test]
fn f3_a_roth_contribution_returned_in_2027_reports_cash_and_positive_earnings() {
    let mut d = filer(19900303);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    let mut facts = ira_facts(0, 0, Answer::No);
    let returned = |earnings, returned_on, late| ReturnedContribution {
        kind: IraKind::Roth,
        principal: 750_000,
        earnings,
        returned_on,
        late,
    };
    facts.taxpayer.returned_next_year = vec![returned(40_000, 20270310, None)];
    d.adjustments.as_mut().unwrap().iras = facts.clone();
    let base = {
        let mut b = d.clone();
        b.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
        supported(&b)
    };
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 790_000);
    assert_eq!(return_line(&r, "4b"), 40_000);
    assert_eq!(tax(&r), 0);
    assert_eq!(value(&r, "agi"), value(&base, "agi") + 400);
    // A loss adds nothing taxable and is not a 2026 excess.
    facts.taxpayer.returned_next_year = vec![returned(-20_000, 20270310, None)];
    d.adjustments.as_mut().unwrap().iras = facts.clone();
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 730_000);
    assert_eq!(return_line(&r, "4b"), 0);
    // After April 15, 2027 the route must be named; the timely-return route adds a note.
    let late = returned(40_000, 20270601, None);
    assert_eq!(late.rule(), Some("IRA_RETURN_AFTER_DUE_DATE_NEEDS_ROUTE"));
    let routed = returned(40_000, 20270601, Some(LateCorrection::TimelyFiledReturn));
    assert_eq!(routed.rule(), None);
    facts.taxpayer.returned_next_year = vec![routed];
    d.adjustments.as_mut().unwrap().iras = facts;
    assert!(has_note(&supported(&d), "NOTE_IRA_RETURN_AMENDED"));
    assert_eq!(
        returned(40_000, 20270310, Some(LateCorrection::FilingExtension)).rule(),
        Some("IRA_RETURN_ROUTE_WITHOUT_LATE_DATE")
    );
    assert_eq!(returned(-800_000, 20270310, None).rule(), Some("IRA_RETURN_LOSS_ABOVE_PRINCIPAL"));
}

// F4: Form 8606 line 2 adjustment (R4). Form 8606 item: basis 12,000 (imported line 14), adjustment
// +1,000 (line 6 of the 2025 Line 15c Worksheet), year-end 90,000. Code 7 IRA distribution 10,000
// under the IRA basis method. Line 2 13,000; line 5 13,000; line 6 90,000; line 7 10,000; line 9
// 100,000; line 10 0.130; line 12 1,300; line 14 11,700; line 15c 8,700 -> line 4b 8,700. (b)
// Adjustment +5,000 (basis received in a divorce): line 10 0.170, line 12 1,700, line 15c 8,300, with
// NOTE_FORM_8606_DIVORCE_STATEMENT. (c) Adjustment -13,000: FORM_8606_ADJUSTMENT_BELOW_BASIS.
#[test]
fn f4_the_form_8606_line_2_adjustment_joins_the_prior_basis() {
    let mut d = filer(19700101);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10_000, RetirementTreatment::Ordinary));
    let mut basis = prior_ira_basis(Owner::Taxpayer, 12_000, 90_000);
    basis.adjustment = 100_000;
    d.adjustments.as_mut().unwrap().ira_basis = vec![basis.clone()];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 870_000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 11_700);
    assert!(has_note(&r, "NOTE_FORM_8606_DIVORCE_STATEMENT"));
    basis.adjustment = 500_000;
    d.adjustments.as_mut().unwrap().ira_basis = vec![basis];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 830_000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 15_300);
    // (c) The page refuses an adjustment below the basis, on the client and the server.
    let page = Form8606 {
        basis: 1_200_000,
        adjustment: -1_300_000,
        year_end: 9_000_000,
        reservist_repayments: vec![],
    };
    assert_eq!(page.rule(), Some("FORM_8606_ADJUSTMENT_BELOW_BASIS"));
    let base = filer(19700101);
    assert_eq!(
        refusal(&base, vec![document(0, IncomeDocument::Form8606(page))]),
        key("FORM_8606_ADJUSTMENT_BELOW_BASIS")
    );
}

// F5: final return, the date-of-death value (R4). The taxpayer died 2026-08-20; distribution
// 2026-05-01 of 20,000; the Form 8606 item's value field (the date-of-death value) 180,000; basis
// 30,000. Line 9 200,000; line 10 0.150; line 12 3,000; line 15c 17,000. The December 31 value is
// never asked.
#[test]
fn f5_a_final_return_uses_the_date_of_death_value() {
    let mut d = filer(19500101);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, estate_medical: 0, date: 20260820, ordinary: true });
    add(&mut d, basis_distribution(Owner::Taxpayer, 20_000, RetirementTreatment::Ordinary));
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 30_000, 180_000)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 1_700_000);
}

// F6: Roth first home (R7). Owner born 1992-01-01; Roth history first year 2018, regular 20,000.
// Code J, box 1 28,000, distributed 2026-04-10, exception First home: costs 15,000 paid 2026-05-01,
// prior 0. Line 19 28,000; line 20 = min(28,000, 15,000, 10,000) = 10,000; line 21 18,000; line 22
// 20,000; line 23 0 -> line 25c 0, line 4b 0. Form 5329 line 1 10,000, line 2 10,000 (09), tax 0;
// regular basis left 2,000. (b) First year 2023: line 20 unavailable; line 21 28,000; line 23 8,000;
// line 25c 8,000; Form 5329 line 1 8,000; exception 09 on the taxable share of the $10,000 slice,
// 8,000 x 10,000 / 28,000 = 2,857; line 3 5,143; tax 514. (c) Exception Separation:
// R_ROTH_EXCEPTION_NOT_IRA.
fn first_home() -> RetirementException {
    RetirementException::FirstHome {
        costs: 1_500_000,
        paid_on: 20260501,
        prior_used: 0,
        eligible: true,
    }
}
#[test]
fn f6_a_roth_first_home_distribution_comes_out_first_and_after_five_years_only() {
    let d = filer(19920101);
    let row = || roth_row(28_000, "J", false, early_j(20260410, Some(first_home())), 0);
    let s = save(&d, vec![row(), roth_history_item(2018, 20_000, vec![])]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 2_800_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_roth_form_line19"), 28_000);
    assert_eq!(value(&r, "taxpayer_roth_form_line20"), 10_000);
    assert_eq!(value(&r, "taxpayer_roth_form_line21"), 18_000);
    assert_eq!(value(&r, "taxpayer_roth_form_line22"), 20_000);
    assert!(r.lines.iter().all(|row| row.id != "taxpayer_roth_form_line24"));
    assert_eq!(early(&r, "taxable"), 10_000);
    assert_eq!(early(&r, "exceptions"), 10_000);
    assert_eq!(tax(&r), 0);
    // (b) A first Roth year of 2023: the five-year period is not met.
    let s = save(&d, vec![row(), roth_history_item(2023, 20_000, vec![])]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 800_000);
    assert_eq!(early(&r, "taxable"), 8_000);
    assert_eq!(early(&r, "exceptions"), 2_857);
    assert_eq!(early(&r, "subject"), 5_143);
    assert_eq!(tax(&r), 514);
    // (c) A plan-only exception is no exception of a Roth IRA.
    let separation =
        RetirementException::Separation { separated: 20260101, work: SeparationWork::Ordinary };
    assert_eq!(
        refusal(
            &d,
            vec![
                roth_row(28_000, "J", false, early_j(20260410, Some(separation)), 0),
                roth_history_item(2018, 20_000, vec![])
            ]
        ),
        key("R_ROTH_EXCEPTION_NOT_IRA")
    );
}

// F7: a Roth SIMPLE IRA inside its two years (R8). Owner born 1985-06-06; Roth history first year
// 2025, regular 6,000 (2025 Roth SIMPLE deferrals), conversions 2025 taxable 1,500 (the employer match
// reported as converted). Form 1099-R codes J and S, box 7b, box 1 9,000, distributed 2026-07-01, no
// exception. Line 19 9,000; line 22 6,000; line 23 3,000; line 24 1,500; line 25c 1,500 -> line 4b
// 1,500. Form 5329 line 1 = 1,500 + recapture 1,500 = 3,000; 25% -> 750. (b) Code J alone with box
// 7b (after the two years): 10% -> 300. (c) Codes J and S without box 7b: R_BOX7B_NEEDED (the map's
// R_ROTH_SIMPLE_NEEDS_BOX7B would repeat that existing rule).
#[test]
fn f7_a_roth_simple_ira_distribution_inside_two_years_is_taxed_at_25_percent() {
    let d = filer(19850606);
    let history = || roth_history_item(2025, 6_000, vec![(2025, 1_500, 0)]);
    let s = save(&d, vec![roth_row(9_000, "JS", true, early_j(20260701, None), 0), history()]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 150_000);
    assert_eq!(early(&r, "taxable"), 3_000);
    assert_eq!(tax(&r), 750);
    let s = save(&d, vec![roth_row(9_000, "J", true, early_j(20260701, None), 0), history()]);
    assert_eq!(tax(&supported(&s.draft)), 300);
    assert_eq!(
        refusal(&d, vec![roth_row(9_000, "JS", false, early_j(20260701, None), 0), history()]),
        key("R_BOX7B_NEEDED")
    );
}

// F8: Roth basis adjustments (R5). Owner born 1975-02-02; first year 2012; regular entered 37,000
// (30,000 contributions + 15,000 designated Roth basis rolled in - 8,000 transferred in a divorce);
// conversions 2021 taxable 10,000. Code J, box 1 50,000, 2026-09-01. Line 19 50,000; line 22 37,000;
// line 23 13,000; line 24 10,000; line 25a 3,000; line 25c 3,000 -> line 4b 3,000. Form 5329 line 1
// 3,000 (the 2021 conversion is outside 2022-2026); tax 300.
#[test]
fn f8_the_roth_basis_the_person_keeps_includes_every_adjustment() {
    let d = filer(19750202);
    let s = save(
        &d,
        vec![
            roth_row(50_000, "J", false, early_j(20260901, None), 0),
            roth_history_item(2012, 37_000, vec![(2021, 10_000, 0)]),
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 300_000);
    assert_eq!(early(&r, "taxable"), 3_000);
    assert_eq!(tax(&r), 300);
    assert!(!has_issue(
        &r,
        "ISSUE_A_ROTH_IRA_HISTORY_WITH_A_RECHARACTERIZATION_A_CORRECTED_EXCESS_CON_2657C1FF"
    ));
}

// F9: a Roth qualified disaster distribution (R9). Owner born 1980-03-03; first year 2019; regular
// 20,000; conversions 2024 taxable 12,000. Code J, box 1 30,000, distributed 2026-06-01; disaster
// began 2026-05-01, declared 2026-05-20, designated 22,000. Line 19 30,000; line 22 20,000; line 23
// 10,000; line 24 12,000; line 25a 0 -> line 25b 0, line 25c 0. Recapture 10,000; designated share
// 10,000 x 22,000 / 30,000 = 7,333 exempt; Form 5329 line 1 2,667, tax 267. (b) The whole 22,000
// distribution designated (the per-disaster limit is 22,000, so the map's 30,000 cannot be entered):
// everything is designated, tax 0.
fn disaster(amount: i64) -> DisasterDistribution {
    DisasterDistribution::Qualified {
        fema: 4900,
        began: 20260501,
        declared: 20260520,
        distributed: 20260601,
        amount: amount * 100,
        earlier: 0,
        spread: crate::DisasterSpread::OverThreeYears,
        repaid: crate::DisasterRepayment::No,
    }
}
#[test]
fn f9_a_designated_roth_disaster_share_leaves_form_5329() {
    let d = filer(19800303);
    let history = || roth_history_item(2019, 20_000, vec![(2024, 12_000, 0)]);
    let mut row = form(
        "J",
        false,
        30_000,
        30_000,
        DistributionCode::RothIra { sequence: None, roth: early_j(20260601, None), rolled_over: 0 },
    );
    row.disaster = disaster(22_000);
    let s = save(&d, vec![doc(row), history()]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(early(&r, "taxable"), 2_667);
    assert_eq!(tax(&r), 267);
    let mut row = form(
        "J",
        false,
        22_000,
        22_000,
        DistributionCode::RothIra { sequence: None, roth: early_j(20260601, None), rolled_over: 0 },
    );
    row.disaster = disaster(22_000);
    let s = save(&d, vec![doc(row), history()]);
    let r = supported(&s.draft);
    assert_eq!(tax(&r), 0);
    assert_eq!(early(&r, "taxable"), 0);
}

// F10: a QCD from an IRA with basis (R16; Pub. 590-B, Amy). Owner born 1951-01-10. Form 8606 item:
// basis 10,000, year-end 5,000. Form 1099-R codes 7 and Y, box 7b, box 1 25,000; QCD gift 25,000 on
// 2026-12-23. Pool = 5,000 + 25,000 - 10,000 = 20,000 -> QCD 20,000; line 7 5,000; line 6 5,000; line
// 5 10,000; line 9 10,000; line 10 1.000; line 12 5,000; line 14 5,000; line 15c 0. Line 4a 25,000,
// line 4b 0; cash gift to Schedule A 5,000. (The map says the App excluded 25,000 here; the pool cap
// was already in qualified_charity, and this fixture pins it.)
#[test]
fn f10_a_qcd_from_an_ira_with_basis_is_capped_at_the_pretax_pool() {
    let d = filer(19510110);
    let s = save(
        &d,
        vec![
            doc(form("7Y", true, 25_000, 25_000, normal())),
            form_8606_item(10_000, 0, 5_000),
            document(
                0,
                IncomeDocument::Qcd(QcdGift {
                    date: 20261223,
                    amount: 2_500_000,
                    inherited: None,
                    split_interest: false,
                }),
            ),
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "qcd_exclusion"), 20_000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 5_000);
    assert_eq!(return_line(&r, "4a"), 2_500_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 5_000);
}

// F11: reservist repayments on the Form 8606 item (R6). Owner born 1988-08-08. Repayment record:
// ordered 2024-02-01 for 400 days, released 2025-03-06, distributed 2024-06-01, gross 12,000, prior 0;
// payments 2026-04-01 5,000 and 2026-09-01 3,000. Form 8606 basis 0, year-end 50,000; code 7 IRA
// distribution 10,000. Line 1 8,000; line 5 8,000; line 6 50,000; line 7 10,000; line 9 60,000; line
// 10 0.133; line 12 1,333; line 14 6,667; line 15c 8,667. (b) A payment dated 2027-04-01 (after
// 2027-03-06): RESERVIST_REPAYMENT_LATE. (c) Payments 5,000 + 8,000: RESERVIST_REPAYMENT_ABOVE_GROSS.
fn reservist(payments: Vec<(u32, i64)>) -> Form8606 {
    Form8606 {
        basis: 0,
        adjustment: 0,
        year_end: 5_000_000,
        reservist_repayments: vec![ReservistRepayment {
            ordered: 20240201,
            term: ReserveTerm::Days { days: 400 },
            released: 20250306,
            distributed: 20240601,
            gross: 1_200_000,
            prior: 0,
            payments: payments
                .into_iter()
                .map(|(date, amount)| ReservistPayment { date, amount: amount * 100 })
                .collect(),
        }],
    }
}
#[test]
fn f11_reservist_repayments_on_the_form_8606_item_add_nondeductible_basis() {
    let d = filer(19880808);
    let s = save(
        &d,
        vec![
            doc(form("7", true, 10_000, 10_000, normal())),
            document(
                0,
                IncomeDocument::Form8606(reservist(vec![(20260401, 5_000), (20260901, 3_000)])),
            ),
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 866_700);
    assert!(
        !r.issues
            .iter()
            .any(|i| i.display.key.contains("RESERVIST") || i.display.key.contains("RESERVE"))
    );
    assert_eq!(
        refusal(
            &d,
            vec![
                doc(form("7", true, 10_000, 10_000, normal())),
                document(0, IncomeDocument::Form8606(reservist(vec![(20270401, 5_000)]))),
            ]
        ),
        key("RESERVIST_REPAYMENT_LATE")
    );
    assert_eq!(
        refusal(
            &d,
            vec![
                doc(form("7", true, 10_000, 10_000, normal())),
                document(
                    0,
                    IncomeDocument::Form8606(reservist(vec![(20260401, 5_000), (20260901, 8_000)]))
                ),
            ]
        ),
        key("RESERVIST_REPAYMENT_ABOVE_GROSS")
    );
}

// F12: a SIMPLE IRA rolled into another SIMPLE IRA (R15). Owner born 1990-01-01; code S, box 7b, box
// 1 = box 2a 10,000; "Rolled it over into another SIMPLE IRA" 6,000. Line 4a 10,000, line 4b 4,000;
// 25% -> 1,000. (b) Kept: line 4b 10,000, 2,500.
#[test]
fn f12_a_simple_ira_rolled_into_another_simple_ira_is_tax_free() {
    let d = filer(19900101);
    let s = save(
        &d,
        vec![doc(form(
            "S",
            true,
            10_000,
            10_000,
            DistributionCode::EarlySimple {
                what_happened: SimpleDisposition::RolledOver { amount: 600_000 },
            },
        ))],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 400_000);
    assert_eq!(tax(&r), 1_000);
    let s = save(
        &d,
        vec![doc(form(
            "S",
            true,
            10_000,
            10_000,
            DistributionCode::EarlySimple { what_happened: SimpleDisposition::Kept },
        ))],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    assert_eq!(tax(&r), 2_500);
}

// F13: two exceptions on one distribution and the long-term-care cap (R10). Owner born 1975-05-05;
// code 1 IRA, box 1 = box 2a 20,000, distributed 2026-03-01; AGI 60,000 (wages 40,000). Exception
// Medical 9,000 (floor 4,500 -> relief 4,500) with second Health insurance while unemployed 6,000.
// Line 1 20,000; line 2 10,500 (99); line 3 9,500; tax 950. The long-term-care cap across plans is
// pinned by care_distribution_coordinates_two_plans_without_reusing_premiums (tests/retirement.rs):
// 2,600 of two rows, tax on the rest.
#[test]
fn f13_a_second_exception_on_one_distribution_adds_to_the_first() {
    let mut d = filer(19750505);
    d.income[0] = wage(1, Owner::Taxpayer, 4_000_000, 0);
    let row = |second: Option<RetirementException>| {
        doc(form(
            "1",
            true,
            20_000,
            20_000,
            DistributionCode::Early {
                what_happened: Disposition::Kept,
                exception: EarlyException::Applies {
                    distributed: 20260301,
                    facts: RetirementException::Medical { expenses: 900_000 },
                    second,
                    repaid: None,
                },
            },
        ))
    };
    let insurance = RetirementException::UnemploymentInsurance {
        premiums: 600_000,
        qualification: UnemploymentQualification::Received { confirmed: true },
        benefit_year: 2026,
        weeks: 14,
        reemployment: Reemployment::None,
    };
    let s = save(&d, vec![row(Some(insurance))]);
    let r = supported(&s.draft);
    assert_eq!(early(&r, "taxable"), 20_000);
    assert_eq!(early(&r, "exceptions"), 10_500);
    assert_eq!(tax(&r), 950);
    // The medical exception alone covers only 4,500.
    let s = save(&d, vec![row(None)]);
    let r = supported(&s.draft);
    assert_eq!(early(&r, "exceptions"), 4_500);
    assert_eq!(tax(&r), 1_550);
    // Periodic payments and age 59½ are not a second exception.
    assert_eq!(
        refusal(&d, vec![row(Some(RetirementException::ReachedAge))]),
        key("R_SECOND_EXCEPTION_KIND")
    );
}

// F14: a 529-to-Roth rollover beside an excess (R11). Owner born 2000-04-04, compensation 9,000;
// Roth contributions 5,000; Form 1099-Q box 4b: basis 3,000, earnings 1,000 (seasoned, confirmed);
// Roth value 12,000. Room = 7,500 - 5,000 = 2,500 -> rollover excess 1,500 -> Part IV line 23 1,500,
// line 25 = 6% x min(1,500, 12,000) = 90; the non-qualifying share's earnings 1,500 x 1,000 / 4,000 =
// 375 on Schedule 1 line 8z, Form 5329 line 5 375, line 8 38. (b) Roth contributions 3,000: room 4,500
// >= 4,000, no excess, regular basis +3,000.
#[test]
fn f14_a_529_to_roth_rollover_above_the_year_room_is_a_roth_excess_and_a_taxable_withdrawal() {
    let mut d = filer(20000404);
    d.income[0] = wage(1, Owner::Taxpayer, 900_000, 0);
    let mut transfer = qtp_roth_rollover();
    transfer.contributions = 300_000;
    transfer.earnings = 100_000;
    transfer.rollover.seasoned_available = 600_000;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer.clone()]);
    let mut facts = ira_facts(0, 500_000, Answer::No);
    facts.taxpayer.excess = Some(excess_page(0, 12_000, 0, 0));
    d.adjustments.as_mut().unwrap().iras = facts.clone();
    let r = supported(&d);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    assert_eq!(value(&r, "ira_excess_taxpayer_line23_roth_excess"), 1_500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line25_roth_excess_tax"), 90);
    assert_eq!(value(&r, "account_taxpayer_taxable"), 375);
    assert_eq!(value(&r, "account_taxpayer_tax"), 38);
    // (b) Regular contributions leave room for the whole transfer.
    facts.taxpayer.roth = 300_000;
    d.adjustments.as_mut().unwrap().iras = facts;
    let r = supported(&d);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    assert_eq!(value(&r, "ira_excess_taxpayer_line23_roth_excess"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
}

// F15: a Social Security recipient with the lump-sum election (R13). Owner born 1960-06-06, wages
// 40,000, SSA-1099 box 5 24,000 (8,000 for 2025), covered at work, traditional contribution 7,500,
// election Compare. Appendix B Worksheet 1 takes all 24,000 under the regular method: line 1 40,000;
// line 2 24,000; line 3 12,000; line 6 52,000; line 7 25,000; line 8 27,000; line 9 9,000; line 10
// 18,000; line 11 9,000; line 12 4,500; line 13 4,500; line 14 15,300; line 15 19,800; line 16
// 20,400; line 17 19,800 -> MAGI 59,800 < 81,000 -> deduction 7,500.
#[test]
fn f15_the_ira_magi_worksheet_ignores_the_lump_sum_election() {
    let mut d = filer(19600606);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 4_000_000, 0);
    add(&mut d, income(social_security_net(2_400_000, 0)));
    let a = d.adjustments.as_mut().unwrap();
    a.social_election = SocialElection::Compare {
        years: vec![SocialBenefitYear { received: 800_000, ..social_prior_year() }],
        complete: true,
    };
    a.iras = ira_facts(750_000, 0, Answer::Yes);
    let r = supported(&d);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    assert_eq!(value(&r, "ira_deduction"), 7_500);
}

// F16: box 9b as the Simplified Method cost (R14). B9-4's F3 annuity (box 1 12,000, started
// 2026-03-01, single life) with box 9b 31,000 imported and the original after-tax cost left empty ->
// cost 31,000 -> line 5b 10,523.80 as in B9-4 F3. (b) Started 2024-03-01 with the cost empty:
// R_COST_NEEDS_BOX_9B.
#[test]
fn f16_box_9b_is_the_simplified_method_cost_for_an_annuity_that_started_in_2026() {
    let d = filer(19600115);
    let simplified = |started: u32, cost: Option<i64>, history: PensionHistory| {
        let mut row = form("7", false, 12_000, 12_000, normal());
        row.more.box_9b = Some(3_100_000);
        row.more.taxable_method = TaxableMethod::Simplified {
            data: PensionBasis {
                earlier_months: 0,
                changes: vec![],
                share: None,
                started,
                cost,
                months: 10,
                history,
                eligible: true,
            },
        };
        row
    };
    let new = PensionHistory::New { duration: PensionDuration::Single { age: 66 } };
    let s = save(&d, vec![doc(simplified(20260301, None, new))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 1_052_400);
    // The saved worksheet holds the resolved cost.
    let FederalIncome::Retirement { data } = &s.draft.income[1].income else { unreachable!() };
    let RetirementTaxable::Simplified { data } = &data.taxable else { panic!() };
    assert_eq!(data.cost, Some(3_100_000));
    let prior = PensionHistory::Prior { monthly: 14_762, recovered: 0 };
    assert_eq!(
        refusal(&d, vec![doc(simplified(20240301, None, prior))]),
        key("R_COST_NEEDS_BOX_9B")
    );
}

// F17: exception 12 during 2026 (already, imp-O103); pinned by tests/fix_f17.rs and
// tests/line_audit_f10.rs d5329_1. The plain-early cases here: born 1966-09-01 (59½ on 2026-03-01),
// code 1 IRA 10,000 distributed 2026-04-01 with "Paid on or after the day you turned 59½" -> tax 0;
// plain Early with no exception -> 1,000 (the automatic test needs 59½ before 2026). Born 1966-05-01,
// plain Early -> 0.
#[test]
fn f17_exception_12_in_the_year_the_owner_turns_59_and_a_half() {
    let with = |birth: u32, exception: EarlyException| {
        let d = filer(birth);
        let s = save(
            &d,
            vec![doc(form(
                "1",
                true,
                10_000,
                10_000,
                DistributionCode::Early { what_happened: Disposition::Kept, exception },
            ))],
        );
        tax(&supported(&s.draft))
    };
    let reached = EarlyException::Applies {
        distributed: 20260401,
        facts: RetirementException::ReachedAge,
        second: None,
        repaid: None,
    };
    assert_eq!(with(19660901, reached), 0);
    assert_eq!(with(19660901, EarlyException::None), 1_000);
    assert_eq!(with(19660501, EarlyException::None), 0);
}

// F18: a missed RMD without the confirmations (already, C6). Administrator minimum 20,000; timely
// 12,000; waiver 5,000 with its explanation; corrected 2027-02-10, return planned 2027-04-10 ->
// shortfall 8,000, waived 5,000, 10% x 3,000 = 300; uncorrected -> 25% x 3,000 = 750.
#[test]
fn f18_a_missed_rmd_needs_no_confirmations() {
    let mut d = complete(FederalStatus::Single);
    let mut e = rmd_record(20_000, 12_000);
    e.waiver =
        Some(RmdWaiver { amount: 500_000, explanation: "Custodian error, corrected".into() });
    e.correction = RmdCorrection::Corrected {
        received: 20270210,
        filed: RmdFiling::Planned { date: 20270410 },
        notice: None,
        assessed: None,
    };
    d.adjustments.as_mut().unwrap().required_distributions = vec![e.clone()];
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 300);
    e.correction = RmdCorrection::Uncorrected;
    d.adjustments.as_mut().unwrap().required_distributions = vec![e];
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 750);
}

// F19: a Roth-to-Roth rollover and an employer Roth SIMPLE contribution (R7, R8). (a) Code J, box 1
// 20,000, rolled over 20,000 -> line 4a 20,000, line 4b 0, nothing on line 19; a second 60-day
// IRA-to-IRA rollover by the owner in 2026 -> ROTH_ROLLOVER_ONCE_A_YEAR. (b) Form 1099-R code 2, box
// 7b, box 1 = box 2a 1,500, "Employer contribution to my Roth SEP or Roth SIMPLE IRA" -> line 4a 1,500,
// line 4b 1,500, no additional tax; the owner's Form 8606 (basis 10,000, year-end 90,000, no other
// distribution) shows line 7 0; a later 2026 code J withdrawal of 1,500 with no other Roth basis
// recaptures it: Form 5329 line 1 1,500, tax 150.
#[test]
fn f19_a_roth_to_roth_rollover_and_an_employer_roth_contribution_reported_as_converted() {
    let d = filer(19750101);
    let s = save(
        &d,
        vec![
            roth_row(20_000, "J", false, early_j(20260601, None), 20_000),
            roth_history_item(2015, 0, vec![]),
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 2_000_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(tax(&r), 0);
    // A second 60-day IRA-to-IRA rollover in the year.
    let rolled = doc(form(
        "7",
        true,
        5_000,
        5_000,
        DistributionCode::Normal {
            what_happened: Disposition::RolledOver {
                amount: 500_000,
                into: crate::RolloverDestination::Ira,
            },
        },
    ));
    assert_eq!(
        current_income_key(
            "income-source-8149",
            &d,
            vec![
                roth_row(20_000, "J", false, early_j(20260601, None), 20_000),
                roth_history_item(2015, 0, vec![]),
                rolled
            ],
            "ROTH_ROLLOVER_ONCE_A_YEAR",
            "[documents/documents/2/document/code]"
        ),
        key("ROTH_ROLLOVER_ONCE_A_YEAR")
    );
    // (b) The employer's contribution is income, a conversion for the ordering and not a Part I
    // distribution.
    let employer =
        || doc(form("2", true, 1_500, 1_500, DistributionCode::EmployerRothContribution));
    let s = save(&d, vec![employer(), form_8606_item(10_000, 0, 90_000)]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 150_000);
    assert_eq!(return_line(&r, "4b"), 150_000);
    assert_eq!(tax(&r), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10_000);
    // A 2026 withdrawal of that 1,500 recaptures it (a conversion inside the five years).
    let s = save(
        &d,
        vec![
            employer(),
            roth_row(1_500, "J", false, early_j(20260901, None), 0),
            roth_history_item(2015, 0, vec![]),
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(early(&r, "taxable"), 1_500);
    assert_eq!(tax(&r), 150);
}

// F20: a Roth repayment, the 2026 Line 25c Worksheet (C4). Owner born 1990-04-02; no Roth history
// (assumed first year 2026, no basis, with the existing warning); code J, box 1 5,000, distributed
// 2026-05-01, exception Qualified birth or adoption (child born 2026-03-10, prior 0); repaid
// 2026-11-20 2,000, accepted Yes. Line 19 5,000; line 22 0; line 25a 5,000; worksheet line 2 5,000,
// line 3 2,000, line 4 3,000 -> line 25c 3,000, line 4b 3,000; Form 5329 line 1 3,000, line 2 3,000
// (19), tax 0. (b) Repaid 6,000: R_REPAYMENT_ABOVE_GROSS. (c) Regular basis 3,000 and repaid 5,000:
// line 25a 2,000, worksheet line 4 0, line 5 3,000 -> NOTE_ROTH_REPAYMENT_NEXT_YEAR_BASIS.
#[test]
fn f20_a_roth_repayment_reduces_line_25c_and_carries_the_excess_to_next_year() {
    let d = filer(19900402);
    let birth = RetirementException::BirthAdoption {
        child: "Child A".into(),
        event: 20260310,
        prior_used: 0,
        eligible: true,
    };
    let repaid = |amount: i64| {
        Some(ExceptionRepayment {
            payments: vec![IraRepaymentDeposit { date: 20261120, amount: amount * 100 }],
            confirmed: true,
        })
    };
    let row = |paid: i64| {
        roth_row(
            5_000,
            "J",
            false,
            RothIraCode::Early {
                distributed: 20260501,
                exception: Some(birth.clone()),
                repaid: repaid(paid),
            },
            0,
        )
    };
    let s = save(&d, vec![row(2_000), roth_history_item(2026, 0, vec![])]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 300_000);
    assert_eq!(early(&r, "taxable"), 3_000);
    assert_eq!(value(&r, "taxpayer_roth_form_line25a"), 5_000);
    assert_eq!(value(&r, "taxpayer_roth_form_line25c"), 3_000);
    assert_eq!(early(&r, "exceptions"), 3_000);
    assert_eq!(tax(&r), 0);
    assert_eq!(
        refusal(&d, vec![row(6_000), roth_history_item(2026, 0, vec![])]),
        key("R_REPAYMENT_ABOVE_GROSS")
    );
    // (c) Regular basis 3,000 takes 3,000 of the 5,000: line 25a 2,000, repaid 5,000.
    let s = save(&d, vec![row(5_000), roth_history_item(2020, 3_000, vec![])]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(next_year_basis_notes(&r, "NOTE_ROTH_REPAYMENT_NEXT_YEAR_BASIS"), vec![3_000]);
    // A Roth repayment needs the owner's Roth history.
    assert_eq!(
        current_income_key(
            "income-source-8221",
            &d,
            vec![row(2_000)],
            "R_ROTH_REPAYMENT_NEEDS_HISTORY",
            "[documents/documents/0/document/code]"
        ),
        key("R_ROTH_REPAYMENT_NEEDS_HISTORY")
    );
}

// Every new answer reopens as entered and saves back unchanged: the Roth row with its rollover,
// exception, repayment, disaster and Roth SIMPLE pair, the employer Roth contribution, the SIMPLE
// rollover, a second exception, the Form 8606 adjustment with its reservist repayments, and a
// Simplified Method cost resolved from box 9b.
#[test]
fn the_new_answers_round_trip_through_the_retirement_page() {
    let d = filer(19900402);
    let mut disaster_row = form(
        "JS",
        true,
        30_000,
        30_000,
        DistributionCode::RothIra {
            sequence: None,
            roth: RothIraCode::Early {
                distributed: 20260601,
                exception: Some(RetirementException::Disability { began: 20250101 }),
                repaid: None,
            },
            rolled_over: 100_000,
        },
    );
    disaster_row.disaster = disaster(22_000);
    let repaid_row = roth_row(
        5_000,
        "J",
        false,
        RothIraCode::Early {
            distributed: 20260501,
            exception: Some(RetirementException::BirthAdoption {
                child: "Child A".into(),
                event: 20260310,
                prior_used: 0,
                eligible: true,
            }),
            repaid: Some(ExceptionRepayment {
                payments: vec![IraRepaymentDeposit { date: 20261120, amount: 200_000 }],
                confirmed: true,
            }),
        },
        0,
    );
    let s = save(
        &d,
        vec![
            doc(disaster_row),
            repaid_row,
            doc(form("2", true, 1_500, 1_500, DistributionCode::EmployerRothContribution)),
            doc(form(
                "1",
                true,
                20_000,
                20_000,
                DistributionCode::Early {
                    what_happened: Disposition::Kept,
                    exception: EarlyException::Applies {
                        distributed: 20260301,
                        facts: RetirementException::Medical { expenses: 900_000 },
                        second: Some(RetirementException::Education {
                            education: 100_000,
                            room_board: 0,
                            allowance: 0,
                            school_housing: 0,
                            half_time: false,
                            assistance: 0,
                            eligible: true,
                        }),
                        repaid: None,
                    },
                },
            )),
            document(
                0,
                IncomeDocument::Form8606(Form8606 {
                    adjustment: 100_000,
                    ..reservist(vec![(20260401, 5_000)])
                }),
            ),
            roth_history_item(2015, 1_000, vec![]),
        ],
    );
    let simple = save(
        &d,
        vec![doc(form(
            "S",
            true,
            10_000,
            10_000,
            DistributionCode::EarlySimple {
                what_happened: SimpleDisposition::RolledOver { amount: 600_000 },
            },
        ))],
    );
    let mut again = Interview::new(TEST_USER, simple.draft.clone());
    let request = open_topic(&mut again, TaxSection::Retirement);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, simple.draft);
    let mut again = Interview::new(TEST_USER, s.draft.clone());
    let request = open_topic(&mut again, TaxSection::Retirement);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, s.draft);
}

#[test]
fn print_roth_form_keeps_available_basis_and_spouse_copies_separate() {
    let d = complete(FederalStatus::Joint);
    let mut spouse = roth_row(15_000, "J", false, early_j(20260901, None), 0);
    spouse.owner = Owner::Spouse;
    let mut spouse_history = roth_history_item(2018, 2_000, vec![(2019, 3_000, 0)]);
    spouse_history.owner = Owner::Spouse;
    let s = save(
        &d,
        vec![
            roth_row(8_000, "J", false, early_j(20260901, None), 0),
            roth_history_item(2018, 20_000, vec![]),
            spouse,
            spouse_history,
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 2_300_000);
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    assert_eq!(value(&r, "taxpayer_roth_form_line22"), 20_000);
    assert_eq!(value(&r, "spouse_roth_form_line22"), 2_000);
    assert_eq!(value(&r, "spouse_roth_form_line24"), 3_000);
    assert_eq!(value(&r, "spouse_roth_form_line25a"), 10_000);
    assert_eq!(value(&r, "spouse_roth_form_line25c"), 10_000);
    let shown: Vec<_> = draft_return(&r)
        .into_iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8606))
        .collect();
    let starts: Vec<_> = shown
        .iter()
        .enumerate()
        .filter_map(|(i, row)| (row.form_line.as_ref().unwrap().line == "19").then_some(i))
        .collect();
    assert_eq!(starts.len(), 2);
    assert_eq!(starts[0], 0);
    assert_eq!(shown[starts[0]].amount, 800_000);
    assert_eq!(shown[starts[1]].amount, 1_500_000);
    for (copy, basis) in [(&shown[..starts[1]], 2_000_000), (&shown[starts[1]..], 200_000)] {
        assert!(
            copy.iter()
                .any(|row| row.form_line.as_ref().unwrap().line == "22" && row.amount == basis)
        );
    }
}

#[test]
fn print_roth_form_stops_after_basis_when_first_home_covers_the_distribution() {
    let d = filer(19920101);
    let s = save(
        &d,
        vec![
            roth_row(6_000, "J", false, early_j(20260410, Some(first_home())), 0),
            roth_history_item(2018, 2_000, vec![]),
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 600_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_roth_form_line20"), 6_000);
    assert_eq!(value(&r, "taxpayer_roth_form_line21"), 0);
    assert_eq!(value(&r, "taxpayer_roth_form_line22"), 2_000);
    assert!(r.lines.iter().all(|row| row.id != "taxpayer_roth_form_line23"));
}

// Dispatcher27e0864 authorizes this figure correction. Pub590-B Worksheet1-1
// uses the actual5000 contribution for its preliminary exclusion: round(1000
// *5000/41000)=122. Final Form8606 line5 is7000 or2000, both >=122, so the
// reporting branch retains122 and skips6–12. Absorbed prior excess never
// becomes an actual payment made next year (Form8606 line4).
fn assert_absorbed_prior_ira_excess_payment_year(paid_next: i64) {
    let mut d = filer(19800505);
    d.income = vec![covered_wage(1, Owner::Taxpayer, 10_000_000, 0)];
    add(&mut d, basis_distribution(Owner::Taxpayer, 1_000, RetirementTreatment::Early));
    let mut facts = ira_facts(500_000, 0, Answer::Yes);
    facts.taxpayer.excess = Some(excess_page(40_000, 0, 3_000, 0));
    let a = d.adjustments.as_mut().unwrap();
    a.iras = facts;
    let mut b = prior_ira_basis(Owner::Taxpayer, 0, 40_000);
    b.paid_next_year = paid_next;
    a.ira_basis = vec![b];
    let r = supported(&d);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 7_000);
    assert_eq!(value(&r, "taxpayer_ira_form_line1"), 7_000);
    assert_eq!(value(&r, "taxpayer_ira_form_line4") * 100, paid_next);
    assert_eq!(value(&r, "taxpayer_ira_form_line5"), 7_000 - paid_next / 100);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 122);
    assert_eq!(value(&r, "taxpayer_ira_basis_taxable"), 878);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 6_878);
    assert_eq!(value(&r, "taxpayer_ira_form_line13"), 122);
    assert_eq!(value(&r, "taxpayer_ira_form_line14"), 6_878);
    assert_eq!(value(&r, "taxpayer_ira_form_line15a"), 878);
    assert_eq!(return_line(&r, "4b"), 87_800);
    assert_eq!(value(&r, "agi"), 100_878);
    assert!(r.lines.iter().all(|row| row.id != "taxpayer_ira_form_line6"));
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn absorbed_prior_ira_excess_with_no_next_year_payment_is_current_basis() {
    assert_absorbed_prior_ira_excess_payment_year(0);
}

#[test]
fn absorbed_prior_ira_excess_does_not_inflate_the_actual_next_year_payment() {
    assert_absorbed_prior_ira_excess_payment_year(500_000);
}

#[test]
fn reservist_exact_overpayment_keeps_date_then_original_gross_priority() {
    use super::support::client::{estimate_documents_from_save, exchange};
    let d = filer(19880808);
    let mut page = reservist(vec![(20260401, 0), (20260901, 0)]);
    page.year_end = i64::MAX;
    page.reservist_repayments[0].gross = i64::MAX;
    page.reservist_repayments[0].payments[0].amount = i64::MAX;
    page.reservist_repayments[0].payments[1].amount = 1;
    for late in [true, false] {
        let mut bad = page.clone();
        if late {
            bad.reservist_repayments[0].payments[1].date = 20270401;
        }
        let mut session = restart(d.clone());
        let request = save_topic(
            &session.draft,
            TaxSection::Retirement,
            vec![document(0, IncomeDocument::Form8606(bad))],
        );
        for request in [estimate_documents_from_save(&request), request] {
            let Reply::Response(P::Problem { message, field }) =
                exchange(&mut session, request, "reservist exact original ceiling")
            else {
                panic!("expected the actual source refusal")
            };
            assert_eq!(
                message,
                key(if late {
                    "RESERVIST_REPAYMENT_LATE"
                } else {
                    "RESERVIST_REPAYMENT_ABOVE_GROSS"
                })
            );
            assert_eq!(
                field.as_deref(),
                Some(if late {
                    "[documents/documents/0/document/reservist_repayments/0/payments/1/date]"
                } else {
                    "[documents/documents/0/document/reservist_repayments/0/prior]"
                })
            );
        }
        assert_eq!(session.draft, d);
        page.reservist_repayments[0].payments[1].amount = 0;
        let corrected = save_topic(
            &session.draft,
            TaxSection::Retirement,
            vec![document(0, IncomeDocument::Form8606(page.clone()))],
        );
        accepted(&send(&mut session, corrected));
        assert_eq!(
            value(&supported(&session.draft), "taxpayer_ira_reservist_basis"),
            92_233_720_368_547_758
        );
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
        // Restore the actual overpayment for the second original-priority control.
        page.reservist_repayments[0].payments[1].amount = 1;
    }
}

// D-2026-10-19-12/-14: Form 8606 Part III foots on its printed lines and line 25c crosses to
// Form 1040 line 4b. A 5,000 distribution against 3,000.50 of regular basis prints line 22 as
// 3,001 and line 23 as 1,999 (the exact 1,999.50 used to print and carry 2,000).
#[test]
fn print_roth_form_foots_on_printed_lines_and_line_25c_is_the_taxable_amount() {
    let d = complete(FederalStatus::Single);
    let history = document(
        0,
        IncomeDocument::RothHistory(RothIraHistory {
            first_year: 2018,
            amounts: Some(RothOrdering { regular: 300_050, conversions: vec![] }),
        }),
    );
    let s = save(&d, vec![roth_row(5_000, "J", false, early_j(20260901, None), 0), history]);
    let r = supported(&s.draft);
    let form: Vec<_> = r
        .lines
        .iter()
        .filter(|l| l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8606))
        .collect();
    assert!(form.iter().all(|l| l.amount % 100 == 0), "whole dollars");
    let at = |n: &str| value(&r, &format!("taxpayer_roth_form_line{n}"));
    assert_eq!((at("21"), at("22"), at("23")), (5_000, 3_001, 1_999));
    assert_eq!((at("25a"), at("25c")), (1_999, 1_999));
    assert_eq!(return_line(&r, "4b"), at("25c") * 100);
}
