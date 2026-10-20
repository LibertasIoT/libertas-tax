use crate::tests::support::assert::{accepted, rejected_with};
use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// Roth IRA distributions on the 1099-R (unit u6-roth, 2026-09-28): box 7a
// codes Q, J and T on the Retirement and Social Security topic, onto the Roth
// income kind, and the topic's "Roth IRA history" kind (Form 8606 Part III
// lines 22 and 24). Each form is reached through the interview; every
// expected amount is worked by hand from the 2026 draft Form 8606 and its
// instructions (lines 19-25c), the 2026 draft Form 5329 instructions (line 1,
// the recapture amount and its example), Pub. 590-B (2025) and the Form 1040
// lines 4a-4b instructions (Exception 2: code Q, or code T with a first Roth
// year 2021 or earlier for 2026, is 4b zero with no Form 8606). The base
// return is a single filer with a $60,000 W-2 ($6,500 withheld): line 15
// 60,000 - 16,100 = $43,900; line 16 row $43,900-$43,950, midpoint $43,925:
// 1,240 + 12% x 31,525 = $5,023.
use super::*;
use crate::tests::cross::interview::from_review;

fn retirement_page(s: &mut Interview) -> P {
    match from_review(s, TaxSection::Retirement) {
        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
        other => panic!("expected the retirement topic, got {other:?}"),
    }
}
fn save_retirement(s: &mut Interview, documents: Vec<IncomeDocumentItem>) -> Reply {
    let mut request = retirement_page(s);
    let P::SaveIncomes { documents: page, .. } = &mut request else { unreachable!() };
    page.documents = documents;
    send(s, request)
}
fn retirement_documents(s: &mut Interview) -> Vec<IncomeDocumentItem> {
    let P::SaveIncomes { documents, .. } = retirement_page(s) else { unreachable!() };
    documents.documents
}
fn item(owner: Owner, document: IncomeDocument) -> IncomeDocumentItem {
    unanswered_document(owner, 0, document)
}
// A Roth IRA 1099-R: box 2a left blank is entered as box 1.
fn roth_1099(gross: i64, withholding: i64, roth: RothIraCode) -> R1099 {
    let code = DistributionCode::RothIra { sequence: None, rolled_over: 0, roth };
    R1099 {
        withholding,
        allowed_methods: vec![0],
        ..crate::tests::cross::interview::r1099(gross, code, false)
    }
}
fn early(distributed: u32) -> RothIraCode {
    RothIraCode::Early { repaid: None, distributed, exception: None }
}
fn exception_applies(distributed: u32, reason: RothReason) -> RothIraCode {
    RothIraCode::ExceptionApplies { distributed, reason }
}
fn history(first_year: u32, regular: i64, conversions: Vec<RothConversion>) -> RothIraHistory {
    RothIraHistory { first_year, amounts: Some(RothOrdering { regular, conversions }) }
}
fn conversion(year: u32, taxable: i64, nontaxable: i64) -> RothConversion {
    RothConversion { year, taxable, nontaxable }
}
fn roth(r: R1099) -> IncomeDocumentItem {
    item(Owner::Taxpayer, IncomeDocument::R1099(r))
}
fn roth_history_item(h: RothIraHistory) -> IncomeDocumentItem {
    item(Owner::Taxpayer, IncomeDocument::RothHistory(h))
}
fn filer_born(birth: u32) -> Interview {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    Interview::new(TEST_USER, d)
}
// Saves the page and returns the estimate of the stored return.
fn saved(s: &mut Interview, documents: Vec<IncomeDocumentItem>) -> FederalResult {
    accepted(&save_retirement(s, documents));
    supported(&s.draft)
}
fn early_tax(r: &FederalResult) -> i64 {
    r.lines.iter().find(|l| l.id == "early_distribution_tax").map_or(0, |l| l.amount / 100)
}

// ---------------------------------------------------------------------------
// Fixtures through the interview.

#[test]
fn code_q_is_qualified_with_no_history_by_hand() {
    // Code Q, box 1 $12,000, box 4 $1,200. Form 1040 Exception 2(b): line
    // 4a $12,000, line 4b $0, no Form 8606 and no history needed; line 25b
    // $1,200. Line 16 stays $5,023.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let r = saved(&mut s, vec![roth(roth_1099(1_200_000, 120_000, RothIraCode::Qualified))]);
    assert_eq!(return_line(&r, "4a"), 1_200_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(return_line(&r, "25b"), 120_000);
    assert_eq!(return_line(&r, "16"), 502_300);
    assert_eq!(early_tax(&r), 0);
    assert!(
        r.lines
            .iter()
            .all(|row| { row.form_line.as_ref().is_none_or(|f| f.form != TaxForm::Form8606) })
    );
    // Stored as the Roth income kind, code Q.
    let FederalIncome::Roth { data } = &s.draft.income.last().unwrap().income else {
        panic!("a Roth IRA 1099-R is the Roth income kind")
    };
    assert_eq!(data.code, RothCode::Qualified);
    assert!(s.draft.adjustments.as_ref().unwrap().roth_history.is_empty());
}

#[test]
fn code_j_covered_by_contribution_basis_by_hand() {
    // Age 46, code J, $8,000 on March 10, 2026. History: first year 2019,
    // $20,000 of regular contributions not yet taken out.
    //   Form 8606 line 19 $8,000; line 22 $20,000; line 23 = 0, so lines 24-25c
    //   are skipped: line 4a $8,000, line 4b $0, no Form 5329 tax.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let r = saved(
        &mut s,
        vec![
            roth(roth_1099(800_000, 0, early(20260310))),
            roth_history_item(history(2019, 2_000_000, vec![])),
        ],
    );
    assert_eq!(return_line(&r, "4a"), 800_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(early_tax(&r), 0);
    let id = s.draft.income.last().unwrap().id;
    assert_eq!(value(&r, &alloc::format!("roth_{id}_regular_recovered")), 8_000);
    assert_eq!(return_line(&r, "16"), 502_300);
}

#[test]
fn code_j_into_conversions_inside_five_years_is_the_form_5329_example() {
    // The 2026 draft Form 5329 instructions' example: 2022 conversion
    // $20,000 (line 18 taxable $15,000, line 17 $5,000), 2023 conversion
    // $10,000 ($7,000 taxable, $3,000 not), $2,000 contributed for each of
    // 2022 and 2023. On July 10, 2026, at age 53, a $33,000 code J
    // distribution.
    //   Form 8606 line 19 $33,000; line 22 $4,000; line 23 $29,000; line 24
    //   $30,000; line 25a = 0, so line 4b $0.
    //   Allocation: $4,000 contributions; $15,000 2022 taxable; $5,000 2022
    //   nontaxable; $7,000 2023 taxable; $2,000 2023 nontaxable. Recapture
    //   = 15,000 + 7,000 = $22,000 on Form 5329 line 1; 10% = $2,200.
    let mut s = filer_born(19730101);
    let r = saved(
        &mut s,
        vec![
            roth(roth_1099(3_300_000, 0, early(20260710))),
            roth_history_item(history(
                2022,
                400_000,
                vec![conversion(2022, 1_500_000, 500_000), conversion(2023, 700_000, 300_000)],
            )),
        ],
    );
    let id = s.draft.income.last().unwrap().id;
    assert_eq!(return_line(&r, "4a"), 3_300_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, &alloc::format!("roth_{id}_conversion_recapture")), 22_000);
    assert_eq!(value(&r, "early_taxpayer_taxable"), 22_000);
    assert_eq!(early_tax(&r), 2_200);
    assert_eq!(return_line(&r, "16"), 502_300);
}

#[test]
fn code_t_for_age_taxes_earnings_without_the_additional_tax_by_hand() {
    // Born March 1, 1964 (59½ on September 1, 2023), code T for age, $9,000
    // on May 1, 2026. First Roth year 2023 (the five-tax-year period ends
    // December 31, 2027): not qualified. Regular contributions $5,000.
    //   Line 19 $9,000; line 22 $5,000; line 23 $4,000; line 24 $0; line
    //   25c $4,000 on line 4b. Age 59½: no 10% tax.
    //   AGI 64,000; line 15 = 64,000 - 16,100 = $47,900; row midpoint
    //   $47,925: 1,240 + 12% x 35,525 = 1,240 + 4,263 = $5,503.
    let mut s = filer_born(19640301);
    let r = saved(
        &mut s,
        vec![
            roth(roth_1099(900_000, 0, exception_applies(20260501, RothReason::Age))),
            roth_history_item(history(2023, 500_000, vec![])),
        ],
    );
    assert_eq!(return_line(&r, "4a"), 900_000);
    assert_eq!(return_line(&r, "4b"), 400_000);
    assert_eq!(early_tax(&r), 0);
    assert_eq!(return_line(&r, "16"), 550_300);
    // Exception 2(a): code T with a first Roth year of 2021 or earlier is
    // line 4b $0, and the amounts are not needed.
    let r = saved(
        &mut s,
        vec![
            roth(roth_1099(900_000, 0, exception_applies(20260501, RothReason::Age))),
            roth_history_item(RothIraHistory { first_year: 2021, amounts: None }),
        ],
    );
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(return_line(&r, "16"), 502_300);
}

#[test]
fn code_j_and_history_round_trip() {
    // Saved, restarted and reopened, the page shows the same 1099-R (with
    // a disability exception and box 4) and history; saving it unchanged
    // keeps the return.
    let mut s = filer_born(19730101);
    let code = RothIraCode::Early {
        repaid: None,
        distributed: 20260710,
        exception: Some(RetirementException::Disability { began: 20250301 }),
    };
    let sent = vec![
        roth(roth_1099(3_300_000, 330_000, code)),
        roth_history_item(history(2022, 400_000, vec![conversion(2022, 1_500_000, 500_000)])),
    ];
    let r = saved(&mut s, sent.clone());
    // Disabled: no recapture tax, the conversion basis still recovered.
    assert_eq!(early_tax(&r), 0);
    let saved_draft = s.draft.clone();
    let mut again = restart(saved_draft.clone());
    let documents = retirement_documents(&mut again);
    assert_eq!(
        documents.iter().map(|d| &d.document).collect::<Vec<_>>(),
        sent.iter().map(|d| &d.document).collect::<Vec<_>>()
    );
    let request = retirement_page(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved_draft);
    // Every code reopens as it was entered, a refused one included.
    for code in [
        RothIraCode::Qualified,
        exception_applies(20260501, RothReason::Disabled { since: 20240101 }),
        exception_applies(20260501, RothReason::Inherited { account: None }),
        RothIraCode::Early {
            repaid: None,
            distributed: 20260201,
            exception: Some(RetirementException::ReachedAge),
        },
        early(20260201),
    ] {
        let mut s = filer_born(19730101);
        accepted(&save_retirement(&mut s, vec![roth(roth_1099(100_000, 0, code.clone()))]));
        let mut again = restart(s.draft.clone());
        assert_eq!(
            retirement_documents(&mut again)[0].document,
            IncomeDocument::R1099(roth_1099(100_000, 0, code))
        );
    }
}

// ---------------------------------------------------------------------------
// Boundaries.

// A single $1,000 code J withdrawal with no contributions left, for a filer
// born January 15, 1967, whose first Roth year is 2010.
fn age_boundary(distributed: u32) -> FederalResult {
    let mut s = filer_born(19670115);
    saved(
        &mut s,
        vec![
            roth(roth_1099(100_000, 0, early(distributed))),
            roth_history_item(history(2010, 0, vec![])),
        ],
    )
}

#[test]
fn age_59_and_a_half_is_six_months_after_the_59th_birthday() {
    // 59½ on July 15, 2026. July 14: earnings $1,000 on line 4b and $100 of
    // Form 5329 tax. July 15: qualified (first year 2010), line 4b $0.
    let r = age_boundary(20260714);
    assert_eq!(return_line(&r, "4b"), 100_000);
    assert_eq!(early_tax(&r), 100);
    let r = age_boundary(20260715);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(early_tax(&r), 0);
}

#[test]
fn five_tax_year_period_for_qualification_starts_with_the_first_year() {
    // Age 66, code T for age, $1,000 with no contributions left: a first
    // year of 2021 completes five tax years by December 31, 2025, so the
    // distribution is qualified (4b $0); 2022 is not (4b $1,000, no 10%).
    for (first_year, taxable) in [(2021, 0), (2022, 100_000)] {
        let mut s = filer_born(19600101);
        let r = saved(
            &mut s,
            vec![
                roth(roth_1099(100_000, 0, exception_applies(20260601, RothReason::Age))),
                roth_history_item(history(first_year, 0, vec![])),
            ],
        );
        assert_eq!(return_line(&r, "4b"), taxable, "{first_year}");
        assert_eq!(early_tax(&r), 0);
    }
}

#[test]
fn conversion_five_year_clock_ends_with_its_fifth_tax_year() {
    // Age 46, code J, $2,000 against a $2,000 taxable conversion: from 2022
    // it is inside its five years (recapture $2,000, $200 tax); from 2021 it
    // is not. Line 4b is $0 either way.
    for (year, tax) in [(2022, 200), (2021, 0)] {
        let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let r = saved(
            &mut s,
            vec![
                roth(roth_1099(200_000, 0, early(20260601))),
                roth_history_item(history(2010, 0, vec![conversion(year, 200_000, 0)])),
            ],
        );
        assert_eq!(return_line(&r, "4b"), 0);
        assert_eq!(early_tax(&r), tax, "{year}");
    }
}

#[test]
fn contribution_basis_exactly_used_leaves_nothing_taxable() {
    // Age 46, code J, $5,000 of contributions: a $5,000 withdrawal is all
    // basis (4b $0, no tax); $5,100 leaves $100 of earnings on 4b and $10
    // of Form 5329 tax.
    for (gross, taxable, tax) in [(500_000, 0, 0), (510_000, 10_000, 10)] {
        let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let r = saved(
            &mut s,
            vec![
                roth(roth_1099(gross, 0, early(20260601))),
                roth_history_item(history(2019, 500_000, vec![])),
            ],
        );
        assert_eq!(return_line(&r, "4b"), taxable);
        assert_eq!(early_tax(&r), tax);
    }
}

// ---------------------------------------------------------------------------
// Refusals by name and the page's rules.

fn refused(documents: Vec<IncomeDocumentItem>, needle: &str) {
    refused_in(documents, "Roth IRA", needle);
}
fn refused_in(documents: Vec<IncomeDocumentItem>, section: &str, needle: &str) {
    let mut s = filer_born(19730101);
    accepted(&save_retirement(&mut s, documents));
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(
        r.issues.iter().any(|i| i.section == section && i.message.contains(needle)),
        "{needle}: {:?}",
        r.issues
    );
    // The review opens the Retirement topic for it.
    assert_eq!(
        crate::interview::issue_page(
            &s.draft,
            &crate::IssueContext::Section(TaxSection::Retirement),
            None
        ),
        Some(TaxSection::Retirement)
    );
}

#[test]
fn roth_unknowns_and_the_removed_refusals_take_the_conservative_path() {
    let with_history = |r: R1099| vec![roth(r), roth_history_item(history(2015, 0, vec![]))];
    // Code J or T without a history, and a nonqualified one without
    // amounts: no basis is assumed, with a warning (batch 6).
    for (documents, key) in [
        (vec![roth(roth_1099(100_000, 0, early(20260601)))], "WARN_ROTH_HISTORY_ASSUMED_NONE"),
        (
            vec![
                roth(roth_1099(100_000, 0, early(20260601))),
                roth_history_item(RothIraHistory { first_year: 2015, amounts: None }),
            ],
            "WARN_ROTH_BASIS_ASSUMED_ZERO",
        ),
    ] {
        let mut s = filer_born(19730101);
        accepted(&save_retirement(&mut s, documents));
        assert!(noted(&supported(&s.draft), key));
    }
    // Code T for age dated before the owner turned 59½ (age 53).
    refused(
        with_history(roth_1099(100_000, 0, exception_applies(20260601, RothReason::Age))),
        "code T for age",
    );
}

#[test]
fn roth_page_rules_reject_on_the_server() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    let h = || roth_history_item(history(2015, 0, vec![]));
    rejected_with(save_retirement(&mut s, vec![h(), h()]), "ROTH_HISTORY_TWICE");
    rejected_with(
        save_retirement(
            &mut s,
            vec![roth_history_item(history(
                2015,
                0,
                vec![conversion(2020, 1, 0), conversion(2020, 2, 0)],
            ))],
        ),
        "ROTH_CONVERSION_YEAR_TWICE",
    );
    rejected_with(
        save_retirement(
            &mut s,
            vec![roth_history_item(history(2015, 0, vec![conversion(2014, 1, 0)]))],
        ),
        "ROTH_CONVERSION_BEFORE_FIRST_YEAR",
    );
    for code in [
        RothIraCode::Early {
            repaid: None,
            distributed: 20260301,
            exception: Some(RetirementException::Disability { began: 20260302 }),
        },
        exception_applies(20260301, RothReason::Disabled { since: 20260302 }),
    ] {
        rejected_with(
            save_retirement(&mut s, vec![roth(roth_1099(100_000, 0, code))]),
            "ROTH_DISABLED_AFTER_DISTRIBUTION",
        );
    }
    let mut other = roth_1099(100_000, 0, RothIraCode::Qualified);
    other.more = RMore { box_10: 5_000, ..RMore::default() };
    rejected_with(save_retirement(&mut s, vec![roth(other)]), "R_ROTH_OTHER_BOXES");
    let mut pso = roth_1099(100_000, 0, RothIraCode::Qualified);
    pso.public_safety = vec![PublicSafetyPaid {
        coverage: PublicSafetyPaidCoverage::Medical,
        premiums: 10_000,
        funded: 10_000,
    }];
    rejected_with(save_retirement(&mut s, vec![roth(pso)]), "PSO_NOT_FROM_AN_IRA");
    // A spouse's history belongs on a joint return only.
    let reply = current_document_refusal(
        "income-source-9392",
        &mut s,
        vec![item(Owner::Spouse, IncomeDocument::RothHistory(history(2015, 0, vec![])))],
        "MESSAGE_ROTH_IRA_HISTORY_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTL",
        "[documents/documents/0/owner]",
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(s.draft, before);
}

// ---------------------------------------------------------------------------
// Walk ownership.

#[test]
fn leaving_joint_erases_the_spouses_roth_history_and_1099_r() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19730101;
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save_retirement(
        &mut s,
        vec![
            item(Owner::Spouse, IncomeDocument::R1099(roth_1099(100_000, 0, early(20260601)))),
            item(Owner::Spouse, IncomeDocument::RothHistory(history(2015, 200_000, vec![]))),
            roth_history_item(history(2012, 0, vec![])),
        ],
    ));
    let mut d = s.draft.clone();
    assert_eq!(d.adjustments.as_ref().unwrap().roth_history.len(), 2);
    assert!(crate::interview::prune(&mut d.clone()).removed.is_empty());
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    let result = crate::interview::prune(&mut d);
    let histories = &d.adjustments.as_ref().unwrap().roth_history;
    assert_eq!(histories.len(), 1);
    assert_eq!(histories[0].owner, Owner::Taxpayer);
    assert!(!d.income.iter().any(|e| e.owner == Owner::Spouse));
    assert!(
        result.removed.iter().any(|item| item.key() == "SAVING_KIND_ROTH_IRA_HISTORY"),
        "{result:?}"
    );
}

#[test]
fn unticking_retirement_removes_the_roth_history() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_retirement(&mut s, vec![roth_history_item(history(2012, 0, vec![]))]));
    let mut d = crate::interview::with_ticks(s.draft.clone());
    assert!(!d.adjustments.as_ref().unwrap().roth_history.is_empty());
    d.checklist.income &= !IncomeTopics::RETIREMENT.0;
    crate::interview::prune(&mut d);
    assert!(d.adjustments.as_ref().unwrap().roth_history.is_empty());
}
