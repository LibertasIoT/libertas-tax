use crate::tests::support::assert::{accepted, rejected_with};
use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// Pages for the stored answers no page asked (unit u5-adjust, 2026-09-28):
// alimony paid on More adjustments; on the Retirement and Social Security
// topic the 1099-R's retired public safety officer question, the Social
// Security lump-sum election and the missed required minimum distribution.
// Each form is reached through the interview; every expected amount is
// worked by hand from the 2025 instructions (Pub. 504, Pub. 575, Pub. 915
// Worksheets 1, 2 and 4, Form 5329 Part IX) with the 2026 amounts the App
// uses ($16,100 standard deduction; 10% to $12,400, 12% to $50,400, 22%
// above; Tax Table rows $50 wide taxed at their midpoint).
use super::*;
use crate::tests::cross::interview::from_review;

pub(in crate::tests) fn problem(reply: Reply) {
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

// ---------------------------------------------------------------------------
// More adjustments: alimony paid (Schedule 1 line 19a).

// The More adjustments page as the review opens it.
fn more_adjustments(s: &mut Interview) -> P {
    match from_review(s, TaxSection::MoreAdjustments) {
        Reply::Edit(request @ P::SaveMoreAdjustments { .. }) => request,
        other => panic!("expected More adjustments, got {other:?}"),
    }
}
fn with_adjustments(mut request: P, items: Vec<MoreAdjustment>) -> P {
    let P::SaveMoreAdjustments { value, .. } = &mut request else { unreachable!() };
    value.adjustments = items;
    request
}
fn alimony(owner: Owner, payment: AlimonyPayment) -> MoreAdjustment {
    MoreAdjustment::AlimonyPaid(AlimonyItem { owner, payment })
}
fn save_alimony(s: &mut Interview, items: Vec<MoreAdjustment>) -> Reply {
    let request = with_adjustments(more_adjustments(s), items);
    send(s, request)
}

#[test]
fn alimony_paid_through_more_adjustments_by_hand() {
    // Single filer, W-2 $60,000. A 2018 decree: $18,000 paid in 2026,
    // $12,000 alimony and $8,000 child support due. Child support is paid
    // first (Pub. 504): 18,000 - 8,000 = $10,000, no more than the $12,000
    // due, so Schedule 1 line 19a = $10,000.
    //   AGI (line 11) = 60,000 - 10,000             = $50,000
    //   line 15 = 50,000 - 16,100                   = $33,900
    //   line 16: row $33,900-$33,950, midpoint $33,925:
    //     1,240 + 12% x (33,925 - 12,400) = 1,240 + 2,583 = $3,823
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_alimony(&mut s, vec![alimony(Owner::Taxpayer, alimony_record(20181231))]));
    let stored = &s.draft.adjustments.as_ref().unwrap().alimony_paid;
    assert_eq!(stored, &vec![paid_alimony(alimony_record(20181231))]);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alimony_paid_deduction"), 10000);
    assert_eq!(return_line(&r, "11a / 11b"), 5_000_000);
    assert_eq!(return_line(&r, "15"), 3_390_000);
    assert_eq!(return_line(&r, "16"), 382_300);
    // Round trip: stored, restarted and reopened, the page shows the same
    // agreement, and saving it unchanged keeps the revision.
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let P::SaveMoreAdjustments { value, .. } = more_adjustments(&mut again) else { unreachable!() };
    assert_eq!(value.adjustments, vec![alimony(Owner::Taxpayer, alimony_record(20181231))]);
    let request = more_adjustments(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft.revision, saved.revision);
    assert_eq!(again.draft.adjustments, saved.adjustments);
}

#[test]
fn alimony_agreement_date_boundary_is_january_1_2019() {
    // An agreement signed December 31, 2018 deducts $10,000; one signed
    // January 1, 2019 deducts nothing (Pub. 504), AGI $60,000.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_alimony(&mut s, vec![alimony(Owner::Taxpayer, alimony_record(20190101))]));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    accepted(&save_alimony(&mut s, vec![alimony(Owner::Taxpayer, alimony_record(20181231))]));
    assert_eq!(return_line(&supported(&s.draft), "11a / 11b"), 5_000_000);
}

#[test]
fn alimony_paid_no_more_than_child_support_due_deducts_nothing() {
    // Paid $8,000 = the child support due: nothing is left for alimony.
    // Paid $8,000.01: one cent of alimony (rounded away on the return).
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut payment = alimony_record(20181231);
    payment.paid = 800_000;
    accepted(&save_alimony(&mut s, vec![alimony(Owner::Taxpayer, payment.clone())]));
    assert_eq!(return_line(&supported(&s.draft), "11a / 11b"), 6_000_000);
    payment.paid = 2_000_000 + 800_000;
    accepted(&save_alimony(&mut s, vec![alimony(Owner::Taxpayer, payment)]));
    // $20,000 left after child support, capped at the $12,000 due.
    assert_eq!(return_line(&supported(&s.draft), "11a / 11b"), 4_800_000);
}

#[test]
fn alimony_page_rules_refuse_on_the_server() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = s.draft.clone();
    // A spouse's agreement on a single return is outside allowed_owners.
    problem({
        let request = with_adjustments(
            more_adjustments(&mut s),
            vec![alimony(Owner::Spouse, alimony_record(20181231))],
        );
        current_source_refusal(
            "secondary-admission-9489",
            &mut s,
            request,
            "MESSAGE_ALIMONY_YOUR_SPOUSE_PAID_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            Some("[value/adjustments/0/owner]"),
            SourceScope::Secondary(SecondaryCorrection::AdjustmentOwner),
        )
    });
    // The recapture exclusions above the year's payments.
    let mut payment = recapture_alimony();
    if let AlimonyRecapture::ThirdYear { first_excluded, .. } = &mut payment.recapture {
        *first_excluded = 5_000_001;
    }
    rejected_with(
        save_alimony(&mut s, vec![alimony(Owner::Taxpayer, payment)]),
        "ALIMONY_2024_EXCLUDED_ABOVE_PAID",
    );
    let mut payment = recapture_alimony();
    if let AlimonyRecapture::ThirdYear { second_excluded, .. } = &mut payment.recapture {
        *second_excluded = 3_900_001;
    }
    rejected_with(
        save_alimony(&mut s, vec![alimony(Owner::Taxpayer, payment)]),
        "ALIMONY_2025_EXCLUDED_ABOVE_PAID",
    );
    let mut payment = alimony_record(20181231);
    payment.recapture = crate::AlimonyRecapture::ThirdYear {
        first: 3_000_000,
        second: 2_000_000,
        first_excluded: 0,
        second_excluded: 0,
        current_excluded: payment.amount() + 1,
        death_or_remarriage: false,
    };
    let request =
        with_adjustments(more_adjustments(&mut s), vec![alimony(Owner::Taxpayer, payment)]);
    current_source_refusal(
        "secondary-alimony-current-excluded",
        &mut s,
        request,
        "MESSAGE_ALIMONY_PAID_ENTER_THE_AGREEMENT_DATE_BEFORE_2027_AND_AMOUNTS_WITH_THIS_YEAR",
        Some("[value/adjustments/0/payment/recapture/current_excluded]"),
        SourceScope::Secondary(SecondaryCorrection::AlimonyRecapture),
    );
    assert_eq!(s.draft, before);
}

#[test]
fn more_adjustments_saved_with_nothing_clears_its_alimony() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_alimony(&mut s, vec![alimony(Owner::Taxpayer, alimony_record(20181231))]));
    assert_eq!(s.draft.adjustments.as_ref().unwrap().alimony_paid.len(), 1);
    let P::SaveMoreAdjustments { value, .. } = more_adjustments(&mut s) else { unreachable!() };
    assert_eq!(value.adjustments, vec![alimony(Owner::Taxpayer, alimony_record(20181231))]);
    // Saved with nothing on it, the page's answers are absent again.
    accepted(&save_alimony(&mut s, vec![]));
    assert!(s.draft.adjustments.as_ref().unwrap().alimony_paid.is_empty());
}

// ---------------------------------------------------------------------------
// The Retirement and Social Security topic.

pub(in crate::tests) fn retirement_page(s: &mut Interview) -> P {
    match from_review(s, TaxSection::Retirement) {
        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
        other => panic!("expected the retirement topic, got {other:?}"),
    }
}
pub(in crate::tests) fn save_retirement(
    s: &mut Interview,
    documents: Vec<IncomeDocumentItem>,
) -> Reply {
    if documents.is_empty() {
        let mut request = crate::tests::support::client::income_pages::open_document_center(s);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        documents.common.retirement.documents.clear();
        return send(s, request);
    }
    crate::tests::support::client::save_retirement_fixture(s, documents)
}
pub(in crate::tests) fn item(owner: Owner, document: IncomeSource) -> IncomeDocumentItem {
    unanswered_document(owner, 0, document)
}

// A governmental pension 1099-R, code 7: box 1 = box 2a.
pub(in crate::tests) fn pension(
    gross: i64,
    public_safety: Vec<PublicSafetyPaid>,
) -> crate::Form1099RPaper {
    let mut form = crate::tests::cross::interview::r1099(
        gross,
        DistributionCode::Normal { what_happened: Disposition::Kept },
        false,
    );
    form.set_public_safety(public_safety);
    retirement_document(form)
}
pub(in crate::tests) fn officer(premiums: i64, funded: i64) -> Vec<PublicSafetyPaid> {
    vec![PublicSafetyPaid { coverage: PublicSafetyPaidCoverage::Medical, premiums, funded }]
}

#[test]
fn removing_the_1099_r_removes_its_premiums() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_retirement(
        &mut s,
        vec![item(
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(pension(
                    3_000_000,
                    officer(500_000, 400_000),
                )),
            },
        )],
    ));
    accepted(&save_retirement(&mut s, vec![]));
    assert!(s.draft.screening.as_ref().unwrap().public_safety.is_empty());
}

// Social Security: a single filer with a W-2 of $30,000 and an SSA-1099
// with box 5 $20,000, of which $8,000 was paid in 2026 for 2024.
pub(in crate::tests) fn lump_sum_return(earlier_agi: i64) -> (Interview, Vec<IncomeDocumentItem>) {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.next_id = 2;
    name_accepted_income_documents(&mut d);
    let s = Interview::new(TEST_USER, d);
    let year = SocialBenefitYear {
        year: 2024,
        status: FederalStatus::Single,
        apart: false,
        received: 800_000,
        benefits: 0,
        earlier_lumps: 0,
        agi: earlier_agi,
        taxable: 0,
        refigured: 0,
        exempt: 0,
        addbacks: 0,
    };
    let documents = vec![
        item(
            Owner::Taxpayer,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(2_000_000, 0))),
            },
        ),
        item(
            Owner::Taxpayer,
            IncomeSource::Record {
                value: crate::IncomeRecord::SocialLumpSums(LumpSumElection {
                    printed_years: Vec::new(),
                    years: vec![year],
                    complete: true,
                }),
            },
        ),
    ];
    (s, documents)
}

#[test]
fn social_security_lump_sum_election_by_hand() {
    // Worksheet 1 (regular): line 1 $20,000; 2 = $10,000; 3 = $30,000;
    //   6 = $40,000; 8 = $40,000; 9 = $25,000; 10 = $15,000; 11 = $9,000;
    //   12 = $6,000; 13 = $9,000; 14 = $4,500; 15 = $4,500; 16 = $5,100;
    //   17 = $9,600; 18 = $17,000; taxable = $9,600.
    // Worksheet 2 (2024; AGI $10,000, no benefits that year): line 1 =
    //   0 + 8,000 = $8,000; 2 = $4,000; 6 = 4,000 + 10,000 = $14,000;
    //   8 = $14,000, not above $25,000: line 21 = $0.
    // Worksheet 4: line 1 = 20,000 - 8,000 = $12,000; 2 = $6,000;
    //   6 = $36,000; 8 = $36,000; 10 = $11,000; 12 = $2,000; 13 = $9,000;
    //   14 = $4,500; 15 = $4,500; 16 = $1,700; 17 = $6,200; 18 = $10,200;
    //   19 = $6,200; 20 = $0; 21 = $6,200, smaller than $9,600: elected.
    //   line 6b = $6,200; AGI = 30,000 + 6,200       = $36,200
    //   line 15 = 36,200 - 16,100                    = $20,100
    //   line 16: row $20,100-$20,150, midpoint $20,125:
    //     1,240 + 12% x (20,125 - 12,400) = 1,240 + 927 = $2,167
    let (mut s, documents) = lump_sum_return(1_000_000);
    accepted(&save_retirement(&mut s, documents));
    assert!(matches!(
        s.draft.adjustments.as_ref().unwrap().social_election,
        SocialElection::Compare { .. }
    ));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "6b"), 620_000);
    assert_eq!(return_line(&r, "11a / 11b"), 3_620_000);
    assert_eq!(return_line(&r, "15"), 2_010_000);
    assert_eq!(return_line(&r, "16"), 216_700);
    // Round trip.
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let request = retirement_page(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved);
}

// ---------------------------------------------------------------------------
// The ownership table's new lines (interview/walk.rs).

#[test]
fn leaving_joint_erases_the_spouses_rows_on_the_new_pages() {
    // A joint return with the spouse's alimony agreement, missed required
    // distribution and pension with public safety officer premiums; on a
    // single return the topic-owned rows are outside allowed_owners. The
    // reported pension and its source premiums remain for source correction.
    let mut d = complete(FederalStatus::Joint);
    let id = d.next_id;
    let mut entry = retirement_entry(
        Owner::Spouse,
        RetirementKind::Pension,
        20_000,
        RetirementTreatment::Ordinary,
    );
    entry.label = "".into();
    add(&mut d, entry);
    public_safety(&mut d, id, 2_000, 1_500, PublicSafetyCoverage::Medical);
    let a = d.adjustments.as_mut().unwrap();
    a.alimony_paid.push(AlimonyPaid { owner: Owner::Spouse, payment: alimony_record(20150301) });
    a.required_distributions
        .push(RequiredDistribution { owner: Owner::Spouse, ..rmd_record(5_000, 1_000) });
    let mut d = crate::interview::with_ticks(d);
    let mut stored = d.clone();
    assert!(crate::interview::prune(&mut stored).removed.is_empty());
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    let pension = d.income.last().unwrap().clone();
    let premiums = d.screening.as_ref().unwrap().public_safety.clone();
    let result = crate::interview::prune(&mut d);
    assert!(d.income.contains(&pension));
    assert_eq!(d.screening.as_ref().unwrap().public_safety, premiums);
    let a = d.adjustments.as_ref().unwrap();
    assert!(a.alimony_paid.is_empty() && a.required_distributions.is_empty());
    for key in ["SAVING_KIND_ALIMONY_RECORD", "SAVING_KIND_REQUIRED_DISTRIBUTION"] {
        assert!(result.removed.iter().any(|item| item.key() == key), "{key}: {result:?}");
    }
    assert!(
        result
            .removed
            .iter()
            .all(|item| item.key() != "SAVING_REMOVES_YOUR_PUBLIC_SAFETY_OFFICER_PREMIUMS")
    );
}

#[test]
fn answers_no_page_asks_are_erased() {
    // Refused by name, never stored by a page: section 501(c)(18)(D)
    // contributions (the Roth IRA history has its page since u6-roth, src/tests/retirement/roth_pages.rs;
    // the Archer MSA contributions and the ABLE contributions have theirs since unit B6a,
    // src/tests/health/account_documents.rs).
    let mut d = complete(FederalStatus::Single);
    let a = d.adjustments.as_mut().unwrap();
    a.pension_contributions.push(PensionContributions {
        owner: Owner::Taxpayer,
        contributions: 100_000,
        compensation: 6_000_000,
        other_deferrals: 0,
        confirmed: true,
    });
    let mut plain = complete(FederalStatus::Single);
    let before = crate::interview::prune(&mut plain).removed.len();
    let result = crate::interview::prune(&mut d);
    assert_eq!(d, plain);
    assert_eq!(result.removed.len(), before + 1, "{result:?}");
}
