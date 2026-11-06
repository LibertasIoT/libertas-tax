use crate::tests::support::assert::line_amount as cents;
// Form 1099-R special codes and distribution histories (unit B9-4, 2026-10-10): the fifteen fixtures
// of the map retirement-special-codes, the routing of every box 7a code and Table 1 pair, the
// stored round trip of the new branches and a corpus check that no test still reaches a removed
// refusal. Each fixture is saved through the Retirement page and read back from the estimate; the
// expected amounts are worked by hand from the cited instruction. Amounts in the comments are
// dollars (the records carry cents). Unless stated: single filer, one 1099-R, box 4 0, box 7b
// unchecked, every confirmation Yes; the base return carries a $60,000 W-2 with $6,500 withheld.
use super::*;
use crate::tests::cross::interview::{accepted, document, open_topic, save_topic};
use crate::tests::support::client::income_pages::{
    open_document_center, open_documents, save_documents,
};
use crate::{
    ExceptionRepayment, InheritedLumpSumTax, IraRepaymentDeposit, LumpSumMethod,
    RolloverDestination, RolloverTarget, TaxableMethod as M,
};
use alloc::format;

fn filer(birth: u32) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    d
}
// One 1099-R in dollars: box 1, box 2a, the printed box 7a and box 7b and the branch.
fn form(
    printed: &str,
    ira: bool,
    gross: i64,
    taxable: i64,
    code: DistributionCode,
) -> R1099Answered {
    let mut r = printed_as(crate::tests::cross::interview::r1099(gross * 100, code, ira), printed);
    r.source_mut().form.taxable = Some(taxable * 100);
    r
}
fn normal() -> DistributionCode {
    DistributionCode::Normal { what_happened: Disposition::Kept }
}
fn early() -> DistributionCode {
    DistributionCode::Early { what_happened: Disposition::Kept, exception: EarlyException::None }
}
fn early_with(facts: RetirementException, distributed: u32) -> DistributionCode {
    DistributionCode::Early {
        what_happened: Disposition::Kept,
        exception: EarlyException::Applies { second: None, distributed, facts, repaid: None },
    }
}
fn doc(r: R1099Answered) -> IncomeDocumentItem {
    document(
        0,
        IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(r)),
        },
    )
}
fn form_8606(basis: i64, year_end: i64) -> IncomeDocumentItem {
    document(
        0,
        IncomeSource::Record {
            value: crate::IncomeRecord::Form8606(Form8606 {
                adjustment: 0,
                reservist_repayments: vec![],
                basis: basis * 100,
                year_end: year_end * 100,
            }),
        },
    )
}
fn gift(date: u32, amount: i64, split: bool) -> IncomeDocumentItem {
    document(
        0,
        IncomeSource::Record {
            value: crate::IncomeRecord::Qcd(QcdGift {
                date,
                amount: amount * 100,
                inherited: None,
                split_interest: split,
            }),
        },
    )
}
// Saves sources and their retained facts through their owning editors.
fn save(d: &FederalDraft, documents: Vec<IncomeDocumentItem>) -> Interview {
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&crate::tests::support::client::save_retirement_fixture(&mut s, documents));
    s
}
// A malformed source is refused against the accepted prerequisite facts.
fn refusal(d: &FederalDraft, documents: Vec<IncomeDocumentItem>) -> Vec<u8> {
    let mut s = Interview::new(TEST_USER, d.clone());
    let baseline = s.draft.clone();
    let dirty = s.dirty;
    let Reply::Response(P::Problem { message, .. }) =
        crate::tests::support::client::save_retirement_fixture(&mut s, documents)
    else {
        panic!("expected refused retirement save")
    };
    assert_eq!(s.draft, baseline);
    assert_eq!(s.dirty, dirty);
    message
}

fn source_refusal_key(
    case: &str,
    d: &FederalDraft,
    sources: Vec<IncomeDocumentItem>,
    key: &str,
    path: &str,
) -> Vec<u8> {
    let mut session = Interview::new(TEST_USER, d.clone());
    let baseline = open_document_center(&mut session);
    accepted(&send(&mut session, baseline));
    let request = save_documents(&mut session, sources.clone());
    let Reply::Response(P::Problem { message, .. }) =
        crate::tests::support::client::current_income_refusal(
            case,
            &mut session,
            request,
            key,
            path,
        )
    else {
        unreachable!()
    };
    message
}
fn record_refusal_key(
    case: &str,
    d: &FederalDraft,
    documents: Vec<IncomeDocumentItem>,
    key: &str,
    path: &str,
) -> Vec<u8> {
    let mut session = Interview::new(TEST_USER, d.clone());
    let (records, sources): (Vec<_>, Vec<_>) =
        documents.into_iter().partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    assert!(!sources.is_empty() && !records.is_empty());
    let request = save_documents(&mut session, sources.clone());
    accepted(&send(&mut session, request));
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &sources,
    );
    let baseline = open_topic(&mut session, TaxSection::Retirement);
    accepted(&send(&mut session, baseline));
    let request = save_topic(&mut session, TaxSection::Retirement, records);
    let Reply::Response(P::Problem { message, .. }) =
        crate::tests::support::client::current_income_refusal(
            case,
            &mut session,
            request,
            key,
            path,
        )
    else {
        unreachable!()
    };
    message
}
fn key(k: &str) -> Vec<u8> {
    libertas::libertas_formatted_text(k, &[])
}
// The stored 1099-R records, in order.
fn stored(s: &Interview) -> Vec<FederalRetirement> {
    s.draft
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Retirement { data } => Some(FederalRetirement::from(data)),
            _ => None,
        })
        .collect()
}
fn has_issue(r: &FederalResult, key: &str) -> bool {
    r.issues.iter().any(|i| i.display.key == key)
}
fn has_note(r: &FederalResult, key: &str) -> bool {
    r.review_notes.iter().any(|n| n.display.key == key)
}
fn additional_tax(r: &FederalResult) -> i64 {
    r.lines.iter().find(|l| l.id == "early_distribution_tax").map_or(0, |l| l.amount / 100)
}
// Both owning editors were accepted during setup; unchanged Saves preserve the complete draft.
fn round_trips(s: &Interview) {
    let mut again = restart(s.draft.clone());
    let request = open_document_center(&mut again);
    accepted(&send(&mut again, request));
    let request = open_topic(&mut again, TaxSection::Retirement);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, s.draft);
}

// The keys this unit removed from src/lib.rs: a refusal that no longer exists.
const REMOVED: [&str; 28] = [
    "ISSUE_1099R_CODE_8",
    "ISSUE_1099R_CODE_9",
    "ISSUE_1099R_CODE_A",
    "ISSUE_1099R_CODE_B",
    "ISSUE_1099R_CODE_E",
    "ISSUE_1099R_CODE_H",
    "ISSUE_1099R_CODE_K",
    "ISSUE_1099R_CODE_L",
    "ISSUE_1099R_CODE_M",
    "ISSUE_1099R_CODE_N",
    "ISSUE_1099R_CODE_P",
    "ISSUE_1099R_CODE_R",
    "ISSUE_1099R_CODE_U",
    "ISSUE_1099R_CODE_W",
    "ISSUE_1099R_CODES_8_J",
    "ISSUE_1099R_CODES_P_J",
    "ISSUE_1099R_CODES_WITH_QCD",
    "ISSUE_8915_HOME_PURCHASE_DISTRIBUTION",
    "ISSUE_8915_NONQUALIFIED_ANNUITY",
    "ISSUE_8915_CONFLICTING_RETIREMENT_TREATMENT",
    "ISSUE_8915_UNSUPPORTED_DISTRIBUTION_COST",
    "ISSUE_8915_DISTRIBUTION_WITH_QCD",
    "ISSUE_8915_DISTRIBUTION_OUTSIDE_DISASTER_PERIOD",
    "WARN_DIRECT_PLAN_TRANSFER_LEFT_OUT",
    "WARN_LUMP_SUM_DEATH_BENEFIT_NOT_APPLIED",
    "WARN_LUMP_SUM_ESTATE_TAX_NOT_APPLIED",
    "R_BOX_9A_NOT_SUPPORTED",
    "R_BOX_10_NOT_SUPPORTED",
];
fn hits_removed(r: &FederalResult) -> bool {
    REMOVED.iter().any(|k| {
        has_issue(r, k) || has_note(r, k) || r.issues.iter().any(|i| i.display.key.starts_with(k))
    })
}

// F1: code 1 with a SECURE 2.0 exception and a repayment (R14). Owner born 1990-04-02. Box 1 5,000
// = box 2a, code 1, box 7b. Early, kept; exception qualified birth or adoption (child "Child A", born
// 2026-03-10, earlier 0), distributed 2026-05-01; repaid to an IRA 2026-11-20 2,000, accepted Yes.
// Form 8606: basis 0, December 31 value 40,000 (including the deposit).
//   Form 8606 line 6 = 40,000 - 2,000 = 38,000; line 7 5,000; line 9 43,000; line 10 0 / 43,000 = 0;
//   line 13 0; line 15a 5,000. 2026 Line 15c Worksheet: 1 = 5,000; 2 = 5,000; 3a = 5,000;
//   3b = 1.000; 3c = 5,000; 3d = 2,000; 3e = min(5,000, 2,000) = 2,000; 4 = 5,000; 5 = 3,000.
//   Form 1040 line 4a 5,000, line 4b 3,000. Form 5329 line 1 3,000, line 2 3,000 (exception 19):
//   additional tax 0.
fn repaid_birth_adoption(confirmed: bool) -> DistributionCode {
    DistributionCode::Early {
        what_happened: Disposition::Kept,
        exception: EarlyException::Applies {
            second: None,
            distributed: 20260501,
            facts: RetirementException::BirthAdoption {
                child: "Child A".into(),
                event: 20260310,
                prior_used: 0,
                eligible: true,
            },
            repaid: Some(ExceptionRepayment {
                payments: vec![IraRepaymentDeposit { date: 20261120, amount: 200_000 }],
                confirmed,
            }),
        },
    }
}
#[test]
fn f1_an_ira_repayment_of_a_birth_or_adoption_distribution_lowers_the_taxable_amount() {
    let d = filer(19900402);
    let row = |confirmed| form("1", true, 5_000, 5_000, repaid_birth_adoption(confirmed));
    let s = save(&d, vec![doc(row(true)), form_8606(0, 40_000)]);
    // The repayment is kept in the owner's Form 8606 record, with the entry it repays.
    let adjustments = s.draft.adjustments.as_ref().unwrap();
    assert_eq!(adjustments.ira_basis[0].special_repayments.len(), 1);
    assert_eq!(adjustments.ira_basis[0].special_repayments[0].income, s.draft.income[1].id);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "taxpayer_ira_repayment_year_end"), 38_000);
    assert_eq!(value(&r, "taxpayer_ira_repayment_income_reduction"), 2_000);
    assert_eq!(return_line(&r, "4a"), 500_000);
    assert_eq!(return_line(&r, "4b"), 300_000);
    assert_eq!(additional_tax(&r), 0);
    assert!(!hits_removed(&r));
    round_trips(&s);
    // (b) A repayment needs the owner's Form 8606.
    assert_eq!(
        source_refusal_key(
            "income-source-9211",
            &d,
            vec![doc(row(true))],
            "R_REPAYMENT_NEEDS_FORM_8606",
            "[documents/common/retirement/documents/0/document/code/exception/repaid]"
        ),
        key("R_REPAYMENT_NEEDS_FORM_8606")
    );
    // (c) No credits nothing: line 4b 5,000, the exception stands.
    let s = save(&d, vec![doc(row(false)), form_8606(0, 40_000)]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 500_000);
    assert_eq!(additional_tax(&r), 0);
    // (d) The same repayment on an employer-plan row is not calculated yet.
    assert_eq!(
        refusal(
            &d,
            vec![
                doc(form("1", false, 5_000, 5_000, repaid_birth_adoption(true))),
                form_8606(0, 40_000)
            ]
        ),
        key("R_REPAYMENT_NOT_SPECIAL")
    );
}

// F2: code G in-plan Roth rollover (R4) and a later early distribution (R3). Owner born 1984-07-01.
// (a) 401(k) box 1 30,000, box 2a 30,000, code G, rolled over to a Roth IRA or designated Roth
// account: line 5a 30,000, line 5b 30,000, no Form 5329 (section 402A(c)(4)(B)). Before this unit
// the same form stored a rollover of 30,000 and line 5b was 0. (b) After-tax money sent to a Roth
// IRA: box 1 10,000, box 2a 0, code G: 5a 10,000, 5b 0. (c) Code H, box 1 25,000, box 2a 0: 5a
// 25,000, 5b 0; box 2a 100 is refused. (d) Codes 1 and B, an early designated Roth distribution two
// years after a 2024 in-plan rollover: box 1 12,000, box 2a 3,000, box 10 4,000: line 5b 3,000, Form
// 5329 line 1 3,000 + 4,000 = 7,000, additional tax 700; box 10 without B is refused.
fn to_roth() -> DistributionCode {
    DistributionCode::DirectRollover { to: RolloverTarget::Roth }
}
#[test]
fn f2_roth_rollovers_are_taxed_and_a_designated_roth_early_distribution_adds_box_10() {
    let d = filer(19840701);
    // (a)
    let s = save(&d, vec![doc(form("G", false, 30_000, 30_000, to_roth()))]);
    assert_eq!(
        stored(&s)[0].ordinary_distribution,
        Some(RetirementTreatment::Rollover {
            destination: RolloverDestination::Roth,
            amount: 3_000_000,
            early: false
        })
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 3_000_000);
    assert_eq!(return_line(&r, "5b"), 3_000_000);
    assert_eq!(additional_tax(&r), 0);
    assert!(!hits_removed(&r));
    round_trips(&s);
    // (b)
    let s = save(&d, vec![doc(form("G", false, 10_000, 0, to_roth()))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 1_000_000);
    assert_eq!(return_line(&r, "5b"), 0);
    // (c)
    let s = save(&d, vec![doc(form("H", false, 25_000, 0, to_roth()))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 2_500_000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(
        refusal(&d, vec![doc(form("H", false, 25_000, 100, to_roth()))]),
        key("R_BOX7_H_TAXABLE")
    );
    // (d)
    let mut row = form("1B", false, 12_000, 3_000, early());
    row.source_mut().form.box_10 = Some(400_000);
    row.source_mut().box_10_basis = Some(crate::InPlanRothBasis::Known { amount: 0 });
    let s = save(&d, vec![doc(row.clone())]);
    assert_eq!(stored(&s)[0].box_10, 400_000);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 300_000);
    assert_eq!(value(&r, "early_taxpayer_taxable"), 7_000);
    assert_eq!(additional_tax(&r), 700);
    round_trips(&s);
    let mut plain = form("1", false, 12_000, 3_000, early());
    plain.source_mut().form.box_10 = Some(400_000);
    assert_eq!(refusal(&d, vec![doc(plain)]), key("R_BOX10_WITHOUT_B"));
}

// Pub. 575 (Figuring your recapture amount; Example Recapture Allocation Chart) and the Form 5329
// instructions, line 1: only the part of box 10 allocated to the Taxable column of the chart is the
// recapture amount. Box 1 35,000, box 2a 3,500, box 10 31,500 of which 1,500 was not taxable when
// rolled in: line 1 is 3,500 + 30,000 = 33,500 and the 10% tax 3,350. An answer left empty keeps the
// tax uncalculated (never assumed 0), and the part cannot exceed box 10 or stand without it.
#[test]
fn f2_box_10_recapture_leaves_out_the_part_not_taxable_when_rolled_in() {
    let d = filer(19690615);
    let mut row = form("1B", false, 35_000, 3_500, early());
    row.source_mut().form.box_10 = Some(3_150_000);
    row.source_mut().box_10_basis = Some(crate::InPlanRothBasis::Known { amount: 150_000 });
    let s = save(&d, vec![doc(row.clone())]);
    assert_eq!(stored(&s)[0].box_10_basis, Some(150_000));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "early_taxpayer_taxable"), 33_500);
    assert_eq!(additional_tax(&r), 3_350);
    round_trips(&s);
    row.source_mut().box_10_basis = None;
    let unknown = estimate(&save(&d, vec![doc(row.clone())]).draft);
    assert!(unknown.issues.iter().any(|i| i.display.key == "ISSUE_BOX_10_BASIS_UNKNOWN"));
    row.source_mut().box_10_basis = Some(crate::InPlanRothBasis::Known { amount: 3_150_001 });
    assert_eq!(refusal(&d, vec![doc(row.clone())]), key("R_BOX10_BASIS_ABOVE_BOX10"));
    row.source_mut().form.box_10 = Some(0);
    row.source_mut().box_10_basis = Some(crate::InPlanRothBasis::Known { amount: 1 });
    assert_eq!(DistributionCodes::treatment_rule(&row), Some("R_BOX10_BASIS_WITHOUT_BOX10"));
    // No printed box 10 calls for this personal answer; Source cannot supply it.
    let s = save(&d, vec![doc(row)]);
    assert_eq!(stored(&s)[0].box_10_basis, None);
}

// F3: code 7 annuity under the Simplified Method, 2026 start (B5, pinned). Owner born 1960-01-15. Box
// 1 12,000 (ten payments of 1,200, March to December), box 2b checked, box 2a entered 12,000, code 7.
// Simplified Method: started 2026-03-01, single life, age 66 at the start: Pub. 575 Table 1 (after
// November 18, 1996) "66-70": 210 payments; cost (box 9b) 31,000; 10 months. Monthly exclusion
// 3,100,000 + 105 / 210 = 14,762 cents = 147.62 (rounded to cents); excluded 10 x 147.62 =
// 1,476.20; line 5a 12,000, line 5b 10,523.80 (10,524 on the return). (b) Joint and survivor,
// survivor age 64: combined 130, Table 2 "121-130": 310; monthly 31,000 / 310 = 100.00; excluded
// 1,000.00; line 5b 11,000.
fn simplified(duration: PensionDuration) -> M {
    M::Simplified {
        data: PensionBasis {
            earlier_months: 0,
            changes: vec![],
            share: None,
            started: 20260301,
            cost: Some(3_100_000),
            months: 10,
            history: PensionHistory::New { duration },
            eligible: true,
        },
    }
}
#[test]
fn f3_the_simplified_method_for_a_2026_start_is_pinned() {
    let d = filer(19600115);
    let mut row = form("7", false, 12_000, 12_000, normal());
    // The payer did not settle box 2a; this fixture exercises its worksheet.
    row.source_mut().form.taxable_not_determined = true;
    row.source_mut().taxable_method = Some(simplified(PensionDuration::Single { age: 66 }));
    let s = save(&d, vec![doc(row)]);
    let id = s.draft.income[1].id;
    let r = supported(&s.draft);
    assert_eq!(cents(&r, &format!("pension_{id}_income_exclusion")), 147_620);
    assert_eq!(return_line(&r, "5a"), 1_200_000);
    assert_eq!(return_line(&r, "5b"), 1_052_400);
    round_trips(&s);
    let mut row = form("7", false, 12_000, 12_000, normal());
    // The payer did not settle box 2a; this fixture exercises its worksheet.
    row.source_mut().form.taxable_not_determined = true;
    row.source_mut().taxable_method =
        Some(simplified(PensionDuration::Joint { age: 66, survivor_age: 64 }));
    let s = save(&d, vec![doc(row)]);
    let id = s.draft.income[1].id;
    let r = supported(&s.draft);
    assert_eq!(cents(&r, &format!("pension_{id}_income_exclusion")), 100_000);
    assert_eq!(return_line(&r, "5b"), 1_100_000);
}

// F4: code 4 inherited IRA under the 10-year rule with an RMD (F3/B6a and rmd_inherited.rs, pinned).
// Beneficiary born 1975-06-01. Form 1099-R box 1 10,000, box 2a 10,000, code 4, box 7b: Inherited with
// the Inherited IRA record ("Parent A", died 2024-03-15, traditional, no basis): line 4a 10,000, line
// 4b 10,000, no additional tax (death). Missed RMD: Inherited minimum with owner born 1948-02-10
// (required beginning date 2019-04-01, so the death is after it and annual minimums apply in years
// 1 to 9), balance December 31, 2025 300,000, no additional status. First beneficiary year 2025:
// beneficiary factor Single Life age 50 = 36.2 - 1 = 35.2; owner factor age 76 = 14.1 - 2 = 12.1;
// factor max = 35.2; deadline 2034. Required 300,000 / 35.2 = 8,522.73; timely 10,000: shortfall 0.
// (b) Timely 5,000, uncorrected: shortfall 3,522.73; Form 5329 Part IX 25% = 880.68 (881 on the
// form). (c) Corrected 2027-02-01 with the return planned 2027-04-15: 10% = 352.27 (352).
#[test]
fn f4_an_inherited_ira_under_the_ten_year_rule_with_an_rmd_is_pinned() {
    let mut d =
        beneficiary_rmd(19480210, 20240315, 19750601, InheritedIraKind::Traditional, 300_000);
    d.adjustments.as_mut().unwrap().required_distributions[0].timely = 1_000_000;
    add_inherited_ira(&mut d, 10_000, 0, 0);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    assert_eq!(additional_tax(&r), 0);
    assert_eq!(value(&r, "rmd_0_inherited_minimum"), 8_522);
    assert_eq!(value(&r, "required_distribution_tax"), 0);
    d.adjustments.as_mut().unwrap().required_distributions[0].timely = 500_000;
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 881);
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    d.adjustments.as_mut().unwrap().required_distributions[0].correction =
        rmd_corrected(20270201, 20270415);
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 352);
}

// F5: code J Roth distribution with the ordering rules (pinned; the 2026 draft Form 5329
// instructions' example). Owner born 1973-03-01 (53). Roth IRA history: contributions 2,000 for 2022
// and 2,000 for 2023; conversions 2022 20,000 (Form 8606 line 17 5,000, line 18 15,000) and 2023
// 10,000 (line 17 3,000, line 18 7,000). Form 1099-R box 1 33,000, code J, distributed 2026-07-10.
// Form 8606 line 19 33,000; line 22 4,000; line 23 29,000; line 24 30,000; line 25a 0, so line 4b 0.
// Ordering: 4,000 contributions; 15,000 (2022 taxable) + 5,000 (2022 nontaxable); 7,000 (2023
// taxable) + 2,000 (2023 nontaxable). Form 5329 line 1 recapture 15,000 + 7,000 = 22,000 (both
// conversions inside their 5-year periods); additional tax 2,200; 1,000 of the 2023 nontaxable
// conversion remains for the next distribution.
#[test]
fn f5_a_roth_distribution_follows_the_ordering_rules_and_recaptures_the_conversions() {
    let d = filer(19730301);
    let code = DistributionCode::RothIra {
        sequence: None,
        rolled_over: 0,
        roth: RothIraCode::Early { repaid: None, distributed: 20260710, exception: None },
    };
    let history = RothIraHistory {
        first_year: 2022,
        amounts: Some(RothOrdering {
            regular: 400_000,
            conversions: vec![
                RothConversion { year: 2022, taxable: 1_500_000, nontaxable: 500_000 },
                RothConversion { year: 2023, taxable: 700_000, nontaxable: 300_000 },
            ],
        }),
    };
    let s = save(
        &d,
        vec![
            doc(crate::tests::cross::interview::r1099(3_300_000, code, false)),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::RothHistory(history) }),
        ],
    );
    let id = s.draft.income.last().unwrap().id;
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 3_300_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, &format!("roth_{id}_conversion_recapture")), 22_000);
    assert_eq!(value(&r, "early_taxpayer_taxable"), 22_000);
    assert_eq!(additional_tax(&r), 2_200);
}

// F6: codes 1 and M, a qualified plan loan offset (R5). Owner born 1985-05-05; severance 2026-10-15. Box
// 1 8,000, box 2a 8,000. Early, rolled over: 8,000 into a traditional IRA deposited in 2027 (before the
// 2026 return's due date with extensions) -> line 5a 8,000, line 5b 0, tax 0. (b) 5,000 rolled over
// -> line 5b 3,000, 10% 300. (c) Codes 1 and L (a deemed loan) instead: kept only: line 5b 8,000, 10%
// 800; rolled over is refused.
#[test]
fn f6_a_loan_offset_can_be_rolled_over_and_a_deemed_loan_cannot() {
    let d = filer(19850505);
    let rolled = |printed: &str, amount: i64| {
        form(
            printed,
            false,
            8_000,
            8_000,
            DistributionCode::Early {
                what_happened: Disposition::RolledOver {
                    amount: amount * 100,
                    into: RolloverDestination::IraNextYear,
                },
                exception: EarlyException::None,
            },
        )
    };
    let s = save(&d, vec![doc(rolled("1M", 8_000))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5a"), 800_000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(additional_tax(&r), 0);
    assert!(!hits_removed(&r));
    round_trips(&s);
    let s = save(&d, vec![doc(rolled("1M", 5_000))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 300_000);
    assert_eq!(additional_tax(&r), 300);
    let s = save(&d, vec![doc(form("1L", false, 8_000, 8_000, early()))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 800_000);
    assert_eq!(additional_tax(&r), 800);
    assert_eq!(refusal(&d, vec![doc(rolled("1L", 8_000))]), key("R_LOAN_KEPT"));
}

// F7: code 8 with the W-2 excess deferral (R7). Owner born 1986-02-02. W-2 box 12 code D 26,000: excess
// 26,000 - 24,500 = 1,500 on line 1h. Form 1099-R from the 401(k): box 1 1,560, box 2a 1,560, code 8;
// part of box 2a that is a 2026 excess deferral 1,500 -> line 1h adds 60; line 1h total 1,560, not
// 3,060; additional tax 0. (b) 1,600 entered is refused. (c) An IRA return under section 408(d)(4):
// codes 1 and 8, box 7b, box 1 7,620, box 2a 120 -> line 4a 7,620, line 4b 120, no additional tax
// (exception 21), not on Form 8606 line 7. (d) Codes 8 and J: box 1 7,650, box 2a 150 -> 4b 150, no
// additional tax.
fn corrective(excess_deferral: i64) -> DistributionCode {
    DistributionCode::Corrective { excess_deferral: excess_deferral * 100 }
}
#[test]
fn f7_a_corrective_distribution_is_wages_less_the_excess_already_counted_or_an_iras_earnings() {
    let mut d = filer(19860202);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!("a wage") };
    edit_wage(data).other_box12.push(W2CodeAmount { code: W2Code::D, amount: 2_600_000 });
    let base_1h = return_line(&supported(&d), "1h");
    assert_eq!(base_1h, 150_000);
    let s = save(&d, vec![doc(form("8", false, 1_560, 1_560, corrective(1_500)))]);
    assert!(matches!(
        stored(&s)[0].ordinary_distribution,
        Some(RetirementTreatment::Corrective { excess_deferral: 150_000 })
    ));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "1h"), 156_000);
    assert_eq!(return_line(&r, "5a"), 0);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(additional_tax(&r), 0);
    assert!(!hits_removed(&r));
    round_trips(&s);
    // (b)
    assert_eq!(
        source_refusal_key(
            "income-source-9223",
            &d,
            vec![doc(form("8", false, 1_600, 1_600, corrective(1_600)))],
            "R_EXCESS_DEFERRAL_ABOVE_W2_EXCESS",
            "[documents/common/retirement/documents/0/document/code/excess_deferral]"
        ),
        key("R_EXCESS_DEFERRAL_ABOVE_W2_EXCESS")
    );
    assert_eq!(
        refusal(&d, vec![doc(form("8", false, 1_560, 1_560, corrective(1_561)))]),
        key("R_EXCESS_DEFERRAL_ABOVE_TAXABLE")
    );
    // (c)
    let s = save(&filer(19860202), vec![doc(form("18", true, 7_620, 120, corrective(0)))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 762_000);
    assert_eq!(return_line(&r, "4b"), 12_000);
    assert_eq!(additional_tax(&r), 0);
    // (d) A Roth IRA payer leaves box 7b unchecked; the printed J makes it an IRA amount.
    let s = save(&filer(19860202), vec![doc(form("8J", false, 7_650, 150, corrective(0)))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 765_000);
    assert_eq!(return_line(&r, "4b"), 15_000);
    assert_eq!(additional_tax(&r), 0);
    round_trips(&s);
}

// F8: not 2026 income (R8). (a) Code P, box 7b, box 1 6,300, box 2a 300, box 4 30 -> lines 4a and 4b 0;
// line 25b includes 30; a note names the earlier year. (b) Code N, box 1 3,300, box 2a 0 -> line 4a
// 3,300, 4b 0; a note asks for the statement; box 2a 50 is refused. (c) Code R, box 1 5,400 ->
// nothing on the return.
#[test]
fn f8_recharacterizations_and_an_earlier_years_return_are_not_2026_income() {
    let d = filer(19860202);
    let before = supported(&d);
    let mut row = form("P", true, 6_300, 300, DistributionCode::NotIncome);
    row.source_mut().form.withholding = Some(3_000);
    let s = save(&d, vec![doc(row)]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 0);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(return_line(&r, "25b"), return_line(&before, "25b") + 3_000);
    assert!(has_note(&r, "NOTE_1099R_EARLIER_YEAR"));
    assert!(!hits_removed(&r));
    round_trips(&s);
    let s = save(&d, vec![doc(form("N", false, 3_300, 0, DistributionCode::NotIncome))]);
    assert_eq!(stored(&s)[0].kind, RetirementKind::Ira);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 330_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert!(has_note(&r, "NOTE_1099R_RECHARACTERIZED"));
    round_trips(&s);
    assert_eq!(
        refusal(&d, vec![doc(form("N", false, 3_300, 50, DistributionCode::NotIncome))]),
        key("R_BOX7_EXCHANGE_TAXABLE")
    );
    let s = save(&d, vec![doc(form("R", false, 5_400, 0, DistributionCode::NotIncome))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), return_line(&before, "4a"));
    assert_eq!(return_line(&r, "5a"), return_line(&before, "5a"));
    assert_eq!(return_line(&r, "16"), return_line(&before, "16"));
}

// F9: code 5, a prohibited transaction (R6). Box 7b, box 1 = box 2a 80,000. (a) Owner born 1970-05-01 (55
// on January 1, 2026): line 4b 80,000, additional tax 8,000. (b) Born 1966-03-01: 59 1/2 on 2025-09-01:
// tax 0. (c) Born 1966-08-01: 59 1/2 on 2026-02-01, after January 1: 8,000. (d) Box 7b unchecked (a
// Roth IRA): refused by name.
#[test]
fn f9_a_prohibited_transaction_is_an_early_distribution_unless_the_owner_was_59_and_a_half() {
    let row = || {
        form(
            "5",
            true,
            80_000,
            80_000,
            DistributionCode::ProhibitedTransaction { exception: EarlyException::None },
        )
    };
    let s = save(&filer(19700501), vec![doc(row())]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 8_000_000);
    assert_eq!(additional_tax(&r), 8_000);
    assert!(!hits_removed(&r));
    round_trips(&s);
    let r = supported(&save(&filer(19660301), vec![doc(row())]).draft);
    assert_eq!(additional_tax(&r), 0);
    assert_eq!(return_line(&r, "4b"), 8_000_000);
    let r = supported(&save(&filer(19660801), vec![doc(row())]).draft);
    assert_eq!(additional_tax(&r), 8_000);
    let mut roth = form("5", false, 80_000, 80_000, normal());
    roth.source_mut().code = Some(DistributionCode::Other);
    let s = save(&filer(19700501), vec![doc(roth)]);
    let r = estimate(&s.draft);
    assert!(has_issue(&r, "ISSUE_1099R_CODE_5"), "{r:?}");
}

// F10: codes 7 and Y with a split-interest QCD (R9). Owner born 1951-01-10 (75). Box 1 60,000, box 2a
// 60,000, codes 7 and Y, box 7b: Normal. QCD gifts: 10,000 on 2026-06-01 and 50,000 on 2026-06-02 to a
// split-interest entity: excluded 60,000 (within 111,000; split-interest 50,000 within 55,000): line
// 4a 60,000, line 4b 0. (b) Gifts 4,000 and 56,000 split-interest are refused. Before this unit codes
// 7 and Y refused.
#[test]
fn f10_a_qcd_with_the_one_time_split_interest_election_is_excluded() {
    let d = filer(19510110);
    let row = || form("7Y", true, 60_000, 60_000, normal());
    let s = save(&d, vec![doc(row()), gift(20260601, 10_000, false), gift(20260602, 50_000, true)]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4a"), 6_000_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert!(!hits_removed(&r));
    round_trips(&s);
    let payments = &s.draft.adjustments.as_ref().unwrap().qualified_charity[0].payments;
    assert_eq!(payments.iter().filter(|p| p.split_interest).count(), 1);
    assert_eq!(
        record_refusal_key(
            "income-source-9177",
            &d,
            vec![doc(row()), gift(20260601, 4_000, false), gift(20260602, 56_000, true)],
            "QCD_SPLIT_INTEREST_ABOVE_LIMIT",
            "[documents/documents/1/document/amount]"
        ),
        key("QCD_SPLIT_INTEREST_ABOVE_LIMIT")
    );
}

// F11: Form 8915-F Part IV (R11). Owner born 1986-09-09. 401(k) hardship distribution, code 1, box 1 =
// box 2a 15,000, distributed 2026-02-10; yes, to buy or build a main home: DR 4900, began 2026-03-01,
// ended 2026-03-20, declared 2026-04-15, repaid 10,000. Window 2025-09-02 (180 days before 2026-03-01)
// to 2026-04-19 (30 days after 2026-03-20) holds 2026-02-10; the repayment period ends 2026-10-12
// (180 days after 2026-04-15). Lines 28 15,000; 29 0; 30 15,000; 31 10,000; 32 5,000 -> line 5b 5,000;
// Form 5329 line 1 5,000 -> 500. (b) Distributed 2026-04-25 is refused.
fn home(distributed: u32) -> DisasterDistribution {
    DisasterDistribution::HomePurchase {
        fema: 4900,
        began: 20260301,
        ended: 20260320,
        declared: 20260415,
        distributed,
        taxable_repaid: 1_000_000,
    }
}
#[test]
fn f11_a_distribution_for_a_main_home_in_a_disaster_area_is_part_iv() {
    let d = filer(19860909);
    let row = |distributed| {
        let mut r = form("1", false, 15_000, 15_000, early());
        r.source_mut().disaster = Some(home(distributed));
        r
    };
    let s = save(&d, vec![doc(row(20260210))]);
    let r = supported(&s.draft);
    for (line, amount) in
        [(28, 1_500_000), (29, 0), (30, 1_500_000), (31, 1_000_000), (32, 500_000)]
    {
        assert_eq!(
            cents(&r, &format!("form_8915_taxpayer_2026_line_{line}")),
            amount,
            "line {line}"
        );
    }
    assert_eq!(return_line(&r, "5b"), 500_000);
    assert_eq!(additional_tax(&r), 500);
    assert!(!hits_removed(&r));
    round_trips(&s);
    assert_eq!(refusal(&d, vec![doc(row(20260425))]), key("DISASTER_HOME_OUTSIDE_PERIOD"));
}

// F12: Form 8915-F Part I legal outcomes and the split (R11). (a) Began 2026-01-10, declared 2026-01-20,
// distributed 2026-07-25: the period ends 2026-07-18 (179 days after 2026-01-20), so the page refuses
// it (before this unit the estimate refused). (b) Code 1, box 1 = box 2a 30,000, distributed
// 2026-05-01 inside the period; designated 21,000 over three years; the rest, 9,000, rolled over to
// an IRA: designated taxable 21,000 (line 11 one third 7,000), rolled 9,000 -> 0; line 5b 7,000; Form
// 5329 none. (c) Code 7D with a designation is refused.
fn qualified_for(distributed: u32, began: u32, declared: u32) -> DisasterDistribution {
    DisasterDistribution::Qualified {
        fema: 4900,
        began,
        declared,
        distributed,
        amount: 2_100_000,
        earlier: 0,
        spread: DisasterSpread::OverThreeYears,
        repaid: crate::DisasterRepayment::No,
    }
}
#[test]
fn f12_the_undesignated_rest_keeps_its_rollover_and_the_period_is_a_rule() {
    let d = filer(19860909);
    let mut late = form("1", false, 30_000, 30_000, early());
    late.source_mut().disaster = Some(qualified_for(20260725, 20260110, 20260120));
    assert_eq!(refusal(&d, vec![doc(late)]), key("DISASTER_OUTSIDE_PERIOD"));
    let mut inside = form(
        "1",
        false,
        30_000,
        30_000,
        DistributionCode::Early {
            what_happened: Disposition::RolledOver {
                amount: 900_000,
                into: RolloverDestination::Ira,
            },
            exception: EarlyException::None,
        },
    );
    inside.source_mut().disaster = Some(qualified_for(20260501, 20260310, 20260320));
    let s = save(&d, vec![doc(inside.clone())]);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_line_11"), 7_000);
    assert_eq!(return_line(&r, "5b"), 700_000);
    assert_eq!(additional_tax(&r), 0);
    assert!(!hits_removed(&r));
    round_trips(&s);
    // Designated and rolled over together cannot pass box 1.
    let DisasterDistribution::Qualified { amount, .. } =
        inside.source_mut().disaster.as_mut().unwrap()
    else {
        unreachable!()
    };
    *amount = 2_200_000;
    assert_eq!(refusal(&d, vec![doc(inside)]), key("DISASTER_AND_ROLLOVER_ABOVE_GROSS"));
    let mut annuity = form("7D", false, 30_000, 30_000, normal());
    annuity.source_mut().disaster = Some(qualified_for(20260501, 20260310, 20260320));
    assert_eq!(DistributionCodes::treatment_rule(&annuity), Some("R_DISASTER_NONQUALIFIED"));
    // An annuity's printed codes do not offer a disaster election on its topic.
    let s = save(&d, vec![doc(annuity)]);
    assert_eq!(stored(&s)[0].disaster, DisasterDistribution::No);
}

// A gift to charity out of the same 1099-R is excluded first and the designation applies to the rest
// (R11). Owner born 1951-01-10; IRA code 7Y, box 1 = box 2a 60,000; QCD 10,000; designated 20,000
// (inside the period). Designated taxable = (60,000 - 10,000) x 20,000 / (60,000 - 10,000) = 20,000;
// line 22 one third 6,667. Line 4b = 60,000 - 10,000 - 20,000 + 6,667 = 36,667.
#[test]
fn a_gift_to_charity_comes_out_before_the_disaster_designation() {
    let d = filer(19510110);
    let mut row = form("7Y", true, 60_000, 60_000, normal());
    row.source_mut().disaster = Some(qualified_for(20260501, 20260310, 20260320));
    let DisasterDistribution::Qualified { amount, .. } =
        row.source_mut().disaster.as_mut().unwrap()
    else {
        unreachable!()
    };
    *amount = 2_000_000;
    let s = save(&d, vec![doc(row.clone()), gift(20260601, 10_000, false)]);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_line_20"), 20_000);
    assert_eq!(cents(&r, "form_8915_taxpayer_2026_line_22"), 666_667);
    assert_eq!(return_line(&r, "4b"), 3_666_700);
    assert!(!hits_removed(&r));
    round_trips(&s);
    // The designation cannot reach into the gift.
    let DisasterDistribution::Qualified { amount, .. } =
        row.source_mut().disaster.as_mut().unwrap()
    else {
        unreachable!()
    };
    *amount = 2_200_000;
    row.source_mut().form.gross = 2_500_000;
    row.source_mut().form.taxable = Some(2_500_000);
    assert_eq!(
        record_refusal_key(
            "income-source-9187",
            &d,
            vec![doc(row), gift(20260601, 10_000, false)],
            "DISASTER_ABOVE_QCD_REST",
            "[documents/documents/0/document/amount]"
        ),
        key("DISASTER_ABOVE_QCD_REST")
    );
}

// F13: Form 4972 line 9 and line 18 (R12). (a) Codes 4 and A, box 2a 60,000; participant born
// 1930-02-01, died 1995-06-01; sole recipient; 10-year method, no capital gain election; share of the
// death benefit exclusion 5,000. Lines 8 60,000; 9 5,000; 10 55,000; 11 0; 12 55,000; 13 10,000; 14
// 35,000; 15 7,000; 16 3,000; 17 52,000; 18 0; 19 52,000; 23 5,200; 24 576.90 + 15% x (5,200 - 4,530)
// = 677.40; 25 6,774.00 = line 30. Without the exclusion: line 12 60,000, 16 2,000, 17 58,000, 23
// 5,800, 24 767.40, 25 7,674.00. Death 2000-01-01 with 5,000 entered is refused. (b) Box 9a 50%, death
// 2000-01-01, box 2a 30,000, share of estate tax 1,000: line 8 = 30,000 / 0.50 = 60,000; line 18 =
// 1,000 / 0.50 = 2,000; 19 = 58,000 - 2,000 = 56,000; 23 5,600; 24 576.90 + 15% x 1,070 = 737.40; 25
// 7,374; Step 5: A 7,374 x B 0.50 = line 29 3,687.00. Before this unit (no line 18): 7,674 x 0.50 =
// 3,837.00.
fn decedent_tax(death_benefit: i64, estate_tax: i64) -> InheritedLumpSumTax {
    InheritedLumpSumTax::Form4972 {
        born: 19300201,
        estate_tax: estate_tax * 100,
        death_benefit: death_benefit * 100,
        method: LumpSumMethod::TenYear,
        include_nua: false,
        first_payment: 20260301,
        last_prior_payment: None,
        entire_balance: true,
        confirmed: true,
    }
}
fn four_a(died: u32, taxable: i64, tax: InheritedLumpSumTax) -> R1099Answered {
    form("4A", false, taxable, taxable, DistributionCode::DeathLumpSum { died, tax })
}
#[test]
fn f13_form_4972_takes_the_death_benefit_exclusion_and_the_shared_estate_tax() {
    let d = filer(19600101);
    let s = save(&d, vec![doc(four_a(19950601, 60_000, decedent_tax(5_000, 0)))]);
    let id = s.draft.income[1].id;
    let r = supported(&s.draft);
    for (line, amount) in [
        (8, 6_000_000),
        (9, 500_000),
        (10, 5_500_000),
        (12, 5_500_000),
        (16, 300_000),
        (17, 5_200_000),
        (18, 0),
        (19, 5_200_000),
        (25, 677_400),
        (30, 677_400),
    ] {
        assert_eq!(cents(&r, &format!("lump_sum_{id}_line_{line}")), amount, "line {line}");
    }
    assert_eq!(value(&r, "lump_sum_tax"), 6_774);
    assert!(!hits_removed(&r));
    round_trips(&s);
    // Without the exclusion.
    let s = save(&d, vec![doc(four_a(19950601, 60_000, decedent_tax(0, 0)))]);
    let id = s.draft.income[1].id;
    let r = supported(&s.draft);
    assert_eq!(cents(&r, &format!("lump_sum_{id}_line_16")), 200_000);
    assert_eq!(cents(&r, &format!("lump_sum_{id}_line_17")), 5_800_000);
    assert_eq!(value(&r, "lump_sum_tax"), 7_674);
    // A death after August 20, 1996 has no exclusion.
    assert_eq!(
        refusal(&d, vec![doc(four_a(20000101, 60_000, decedent_tax(5_000, 0)))]),
        key("LUMP_SUM_DEATH_BENEFIT_AFTER_1996")
    );
    // (b)
    let mut shared = four_a(20000101, 30_000, decedent_tax(0, 1_000));
    shared.source_mut().form.box_9a_percent = Some(50.0);
    let s = save(&d, vec![doc(shared)]);
    let id = s.draft.income[1].id;
    let r = supported(&s.draft);
    assert_eq!(cents(&r, &format!("lump_sum_{id}_line_8")), 6_000_000);
    assert_eq!(cents(&r, &format!("lump_sum_{id}_line_18")), 200_000);
    assert_eq!(cents(&r, &format!("lump_sum_{id}_line_19")), 5_600_000);
    assert_eq!(cents(&r, &format!("lump_sum_{id}_line_25")), 737_400);
    assert_eq!(cents(&r, &format!("lump_sum_{id}_line_29")), 368_700);
    assert_eq!(value(&r, "lump_sum_tax"), 3_687);
    round_trips(&s);
    assert!(!has_note(&r, "WARN_LUMP_SUM_ESTATE_TAX_NOT_APPLIED"));
}

// F14: substantially equal periodic payments (R13). Owner born 1974-04-04 (52). IRA, code 2, box 1 = box
// 2a 24,000; Early, exception 02: first payment 2023-02-01, not changed, confirmed Yes, distributed
// 2026-02-01 -> line 4b 24,000, additional tax 0. (b) Changed Yes is refused as the recapture tax.
// (c) Code 1 at age 60 with the paid-on-or-after-59 1/2 exception -> tax 0; distributed before the 59
// 1/2 date -> the existing date-qualification issue.
fn periodic(modified: bool, eligible: bool) -> RetirementException {
    RetirementException::PeriodicPayments { first_payment: 20230201, modified, eligible }
}
#[test]
fn f14_periodic_payments_and_a_distribution_after_59_and_a_half_are_exceptions() {
    let d = filer(19740404);
    let row = |facts| form("2", true, 24_000, 24_000, early_with(facts, 20260201));
    let s = save(&d, vec![doc(row(periodic(false, true)))]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 2_400_000);
    assert_eq!(additional_tax(&r), 0);
    assert!(!hits_removed(&r));
    round_trips(&s);
    // No applies no exception.
    let r = estimate(&save(&d, vec![doc(row(periodic(false, false)))]).draft);
    assert_eq!(additional_tax(&r), 2_400);

    // (b)
    let r = estimate(&save(&d, vec![doc(row(periodic(true, true)))]).draft);
    assert!(has_issue(&r, "ISSUE_SEPP_RECAPTURE"), "{r:?}");
    // The first payment cannot follow the distribution.
    let later = RetirementException::PeriodicPayments {
        first_payment: 20260301,
        modified: false,
        eligible: true,
    };
    assert_eq!(refusal(&d, vec![doc(row(later))]), key("SEPP_FIRST_PAYMENT_AFTER_DISTRIBUTION"));
    // (c)
    let age = filer(19660101);
    let aged = |distributed| {
        form("1", true, 10_000, 10_000, early_with(RetirementException::ReachedAge, distributed))
    };
    let s = save(&age, vec![doc(aged(20260601))]);
    let r = supported(&s.draft);
    assert_eq!(additional_tax(&r), 0);
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    round_trips(&s);
    let young = filer(19660801);
    let r = estimate(&save(&young, vec![doc(aged(20260115))]).draft);
    assert!(
        r.issues.iter().any(|i| i.display.key.starts_with("ISSUE_THE_EXCEPTION_S_DATE")),
        "{r:?}"
    );
}

// F15: one rollover a year (R15). Two 2026 IRA rows of one owner, each rolled over into a traditional
// IRA, are refused; one of them converted instead is accepted.
#[test]
fn f15_only_one_ira_to_ira_rollover_is_tax_free_in_a_year() {
    let d = filer(19600101);
    let rolled = |into| {
        form(
            "7",
            true,
            5_000,
            5_000,
            DistributionCode::Normal {
                what_happened: Disposition::RolledOver { amount: 500_000, into },
            },
        )
    };
    assert_eq!(
        source_refusal_key(
            "income-source-9154",
            &d,
            vec![
                doc(rolled(RolloverDestination::Ira)),
                doc(rolled(RolloverDestination::IraNextYear))
            ],
            "IRA_ROLLOVER_ONCE_A_YEAR",
            "[documents/common/retirement/documents/1/document/code]"
        ),
        key("IRA_ROLLOVER_ONCE_A_YEAR")
    );
    let converted = form(
        "7",
        true,
        5_000,
        5_000,
        DistributionCode::Normal { what_happened: Disposition::Converted { amount: 500_000 } },
    );
    let s = save(&d, vec![doc(rolled(RolloverDestination::Ira)), doc(converted)]);
    assert_eq!(stored(&s).len(), 2);
    round_trips(&s);
}

// Routing: each Form 1099-R box 7a code alone and each of Table 1's 47 pairs leads to the branch the
// map gives (the discriminants of DistributionCode: 0 Normal, 1 Early, 2 Disability, 3 Death, 4
// DirectRollover, 5 EarlySimple, 6 Nonqualified, 7 RothIra, 8 Other, 9 NormalLumpSum, 10
// DeathLumpSum, 11 Exchange, 12 Inherited, 13 TrumpDeath, 14 TrumpToAble, 15 Corrective, 16
// NotIncome, 17 NoAdditionalTax, 18 ProhibitedTransaction, 19 QualifiedCareCharge, 20 EmployerRothContribution).
#[test]
fn every_box_7a_code_and_table_1_pair_routes_to_its_branch() {
    let routes: &[(&str, bool, &[i32])] = &[
        // The codes alone.
        ("1", false, &[1]),
        ("2", false, &[1]),
        ("3", false, &[2]),
        ("4", false, &[3]),
        ("4", true, &[12]),
        ("5", true, &[18]),
        ("5", false, &[8]),
        ("6", false, &[11]),
        ("7", false, &[0]),
        ("8", false, &[15]),
        ("9", false, &[17]),
        ("A", false, &[9, 10]),
        ("A", true, &[0, 12]),
        ("B", false, &[0]),
        ("C", false, &[8]),
        ("D", false, &[6]),
        ("E", false, &[17]),
        ("F", false, &[17]),
        ("G", false, &[4]),
        ("H", false, &[4]),
        ("J", false, &[7]),
        ("K", true, &[0]),
        ("L", false, &[0]),
        ("M", false, &[0]),
        ("N", false, &[16]),
        ("P", false, &[16]),
        ("Q", false, &[7]),
        ("R", false, &[16]),
        ("S", true, &[5]),
        ("T", false, &[7]),
        ("U", false, &[17]),
        ("W", false, &[19]),
        // The 47 pairs.
        ("18", false, &[15]),
        ("28", false, &[15]),
        ("48", false, &[15]),
        ("4A", false, &[10]),
        ("4A", true, &[12]),
        ("7A", false, &[9]),
        ("7A", true, &[0]),
        ("1B", false, &[1]),
        ("2B", false, &[1]),
        ("4B", false, &[3]),
        ("7B", false, &[0]),
        ("8B", false, &[15]),
        ("BG", false, &[4]),
        ("BL", false, &[0]),
        ("BM", false, &[0]),
        ("BP", false, &[16]),
        ("BU", false, &[17]),
        ("1D", false, &[1]),
        ("2D", false, &[1]),
        ("3D", false, &[2]),
        ("4D", false, &[3]),
        ("7D", false, &[0]),
        ("CD", false, &[8]),
        ("1K", true, &[1]),
        ("2K", true, &[1]),
        ("4K", true, &[12]),
        ("7K", true, &[0]),
        ("8K", true, &[15]),
        ("GK", true, &[4]),
        ("KY", true, &[0]),
        ("1L", false, &[1]),
        ("2L", false, &[1]),
        ("4L", false, &[3]),
        ("7L", false, &[0]),
        ("1M", false, &[1]),
        ("2M", false, &[1]),
        ("4M", false, &[3]),
        ("7M", false, &[0]),
        ("4G", false, &[4]),
        ("4H", false, &[4]),
        ("4P", false, &[16]),
        ("1P", false, &[16]),
        ("2P", false, &[16]),
        ("JP", false, &[16]),
        ("8J", false, &[15]),
        ("JS", true, &[7]),
        ("6W", false, &[19]),
        ("4Y", true, &[12]),
        ("7Y", true, &[0]),
        // The same codes where box 7b and box 7c change the branch.
        ("1", true, &[1]),
        // 2 or 7 with box 7b also offers an employer's Roth SEP or Roth SIMPLE contribution.
        ("7", true, &[0, 20]),
        ("G", true, &[4]),
        ("J", true, &[7]),
        ("P", true, &[16]),
        ("8", true, &[15]),
        ("E", true, &[17]),
        ("4G", true, &[4]),
        ("4P", true, &[16]),
        ("1P", true, &[16]),
        ("GK", false, &[4]),
        ("1K", false, &[1]),
        ("2K", false, &[1]),
        ("7K", false, &[0]),
        ("3", true, &[2]),
        ("2", true, &[1, 20]),
        ("S", false, &[5]),
        ("Q", true, &[7]),
        ("T", true, &[7]),
    ];
    for &(printed, ira, branches) in routes {
        let box7 = box7(printed);
        assert_eq!(
            DistributionCodes::treatments(box7, ira, false),
            branches,
            "{printed} box 7b {ira}"
        );
    }
    // Every printed pair Table 1 allows (with box 7b as the pair needs it) is in the table.
    let codes: Vec<char> = "123456789ABCDEFGHJKLMNPQRSTUWY".chars().collect();
    let mut pairs = 0;
    for (i, a) in codes.iter().enumerate() {
        for b in &codes[i + 1..] {
            let printed: alloc::string::String = [*a, *b].iter().collect();
            let box7 = box7(&printed);
            let valid = |ira| DistributionCodes::rule(box7, ira).is_none();
            if valid(false) || valid(true) {
                pairs += 1;
                assert!(routes.iter().any(|(p, ..)| *p == printed), "{printed} is not routed");
            }
        }
    }
    assert_eq!(pairs, 47);
}

// The new branches reopen as they were saved, the A-alone branches stay distinct and the page's Roth
// destination survives a save.
#[test]
fn the_new_branches_round_trip_through_the_page() {
    let d = filer(19600101);
    let rows = vec![
        doc(form("8", false, 1_000, 1_000, corrective(0))),
        doc(form("E", false, 500, 500, DistributionCode::NoAdditionalTax)),
        doc(form("U", false, 400, 400, DistributionCode::NoAdditionalTax)),
        doc(form("9", false, 300, 300, DistributionCode::NoAdditionalTax)),
        doc(form("W", false, 200, 0, DistributionCode::QualifiedCareCharge)),
        doc(form("G", false, 3_000, 3_000, to_roth())),
        doc(form(
            "A",
            false,
            2_000,
            2_000,
            DistributionCode::NormalLumpSum {
                tax: crate::LumpSumTax::WithOtherIncome { what_happened: Disposition::Kept },
            },
        )),
        doc(form(
            "A",
            false,
            1_000,
            1_000,
            DistributionCode::DeathLumpSum {
                died: 20250101,
                tax: InheritedLumpSumTax::WithOtherIncome,
            },
        )),
    ];
    let s = save(&d, rows);
    assert_eq!(stored(&s).len(), 8);
    assert!(matches!(
        stored(&s)[0].ordinary_distribution,
        Some(RetirementTreatment::Corrective { excess_deferral: 0 })
    ));
    assert!(matches!(
        stored(&s)[4].ordinary_distribution,
        Some(RetirementTreatment::QualifiedCareCharge)
    ));
    assert!(matches!(
        stored(&s)[5].ordinary_distribution,
        Some(RetirementTreatment::Rollover { destination: RolloverDestination::Roth, .. })
    ));
    let r = estimate(&s.draft);
    assert!(!hits_removed(&r), "{r:?}");
    round_trips(&s);
    // The W row is not taxed and not reported.
    assert_eq!(
        return_line(
            &supported(
                &save(
                    &d,
                    vec![doc(form("W", false, 200, 0, DistributionCode::QualifiedCareCharge))]
                )
                .draft
            ),
            "5a"
        ),
        0
    );
}

// Box 9a is accepted with every treatment: boxes 1 and 2a are already this recipient's share.
#[test]
fn box_9a_is_accepted_with_every_treatment_and_stored_only_on_form_4972() {
    let d = filer(19600101);
    let mut row = form("7", false, 5_000, 5_000, normal());
    row.source_mut().form.box_9a_percent = Some(25.0);
    let s = save(&d, vec![doc(row)]);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "5b"), 500_000);
    // A Roth IRA row accepts it too.
    let mut roth = crate::tests::cross::interview::r1099(
        100_000,
        DistributionCode::RothIra { sequence: None, rolled_over: 0, roth: RothIraCode::Qualified },
        false,
    );
    roth.source_mut().form.box_9a_percent = Some(50.0);
    accepted(&{
        let mut session = Interview::new(TEST_USER, d.clone());
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![doc(roth)],
        );
        send(&mut session, request)
    });
}

// No test of this crate still names a refusal this unit removed: the keys appear only in this list.
#[test]
fn no_test_still_expects_a_removed_refusal() {
    extern crate std;
    let mut offenders = Vec::new();
    let mut dirs = alloc::vec![std::path::PathBuf::from("src/tests")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs")
                || path.ends_with("retirement_special_codes.rs")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for key in REMOVED {
                let quoted = alloc::format!("\"{key}\"");
                if text.contains(&quoted) {
                    offenders.push(alloc::format!("{} names {key}", path.display()));
                }
            }
        }
    }
    assert!(offenders.is_empty(), "{offenders:?}");
}

#[test]
fn retirement_external_answers_survive_source_correction_and_write_with_same_save_form8606() {
    use crate::tests::support::client::income_pages::{information_return_answers, page_documents};
    let mut session = Interview::new(TEST_USER, filer(19900402));
    let mut pension = form("7", false, 30_000, 30_000, normal());
    pension.set_public_safety(vec![crate::PublicSafetyPaid {
        coverage: crate::PublicSafetyPaidCoverage::Medical,
        premiums: 500_000,
        funded: 400_000,
    }]);
    let desired =
        vec![doc(pension), doc(form("1", true, 5_000, 5_000, repaid_birth_adoption(true)))];
    let request = save_documents(&mut session, desired.clone());
    accepted(&send(&mut session, request));
    // Both the actual Form 8606 and its distribution's actual repayment are entered in one topic Save.
    let P::SaveIncomes { documents: records, .. } =
        save_topic(&mut session, TaxSection::Retirement, vec![form_8606(0, 40_000)])
    else {
        unreachable!()
    };
    let mut request = information_return_answers(&mut session, &desired);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    *documents = records;
    accepted(&send(&mut session, request));
    let premiums = session.draft.screening.as_ref().unwrap().public_safety.clone();
    let repayments =
        session.draft.adjustments.as_ref().unwrap().ira_basis[0].special_repayments.clone();
    assert_eq!(premiums.len(), 1);
    assert_eq!((premiums[0].premiums, premiums[0].funded), (500_000, 400_000));
    assert_eq!(repayments.len(), 1);
    assert_eq!(
        repayments[0].payments,
        vec![IraRepaymentDeposit { date: 20261120, amount: 200_000 }]
    );
    let page = open_documents(&mut session);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.screening.as_ref().unwrap().public_safety, premiums);
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().ira_basis[0].special_repayments,
        repayments
    );
    let mut sources = page_documents(&open_documents(&mut session));
    for source in &mut sources {
        let IncomeSource::Retirement { value: crate::RetirementDocument::R1099(row) } =
            &mut source.document
        else {
            continue;
        };
        row.withholding = Some(1_000);
    }
    let request = save_documents(&mut session, sources);
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    assert_eq!(session.draft.screening.as_ref().unwrap().public_safety, premiums);
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().ira_basis[0].special_repayments,
        repayments
    );
    let request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
    assert!(followups.iter().any(|row| matches!(
        &row.answer,
        crate::DocumentAnswer::RetirementPublicSafety { value: Some(_) }
    )));
    assert!(followups.iter().any(|row| matches!(
        &row.answer,
        crate::DocumentAnswer::RetirementTreatment {
            code: Some(crate::RetirementTreatmentView::Early {
                exception: EarlyException::Applies { repaid: Some(_), .. },
                ..
            })
        }
    )));
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.screening.as_ref().unwrap().public_safety, premiums);
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().ira_basis[0].special_repayments,
        repayments
    );
    // Removing the physical sources removes their external answer records as well.
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.retirement.documents.clear();
    accepted(&send(&mut session, request));
    assert!(session.draft.screening.as_ref().unwrap().public_safety.is_empty());
    assert!(session.draft.adjustments.as_ref().unwrap().ira_basis[0].special_repayments.is_empty());
}
