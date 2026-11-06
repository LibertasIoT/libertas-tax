use crate::tests::support::assert::{accepted, rejected_with};
use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// Alimony received (unit u6-alimony, 2026-09-28): the "Alimony received"
// kind of the Prizes, royalties, equipment rent and alimony topic, onto
// FederalIncome::Alimony (Schedule 1 line 2a, and line 19a for the
// recipient's third-year recapture deduction). Each amount is worked by hand
// from Pub. 504 (2025) and the 2025 Form 1040 instructions (line 2a: an
// agreement entered into on or before December 31, 2018) with the 2026
// amounts the App uses ($16,100 standard deduction; 10% to $12,400, 12% to
// $50,400, 22% above; Tax Table rows $50 wide taxed at their midpoint).
// Every return is `complete(Single)`: one W-2 of $60,000.
use super::*;
use crate::tests::cross::interview::from_review;

fn problem(reply: Reply) {
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

// The topic page as the review opens it.
fn topic_page(s: &mut Interview) -> P {
    match from_review(s, TaxSection::OtherIncome) {
        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
        other => panic!("expected the prizes, royalties and alimony topic, got {other:?}"),
    }
}
fn received(owner: Owner, payment: AlimonyPayment) -> IncomeDocumentItem {
    unanswered_document(
        owner,
        0,
        IncomeSource::Record { value: crate::IncomeRecord::AlimonyReceived(payment) },
    )
}
fn save(s: &mut Interview, documents: Vec<IncomeDocumentItem>) -> Reply {
    let mut request = topic_page(s);
    let P::SaveIncomes { documents: page, .. } = &mut request else { unreachable!() };
    page.documents = documents
        .into_iter()
        .map(|row| crate::IncomeRecordItem::from_income(row).expect("alimony record"))
        .collect();
    send(s, request)
}
fn single() -> Interview {
    Interview::new(TEST_USER, complete(FederalStatus::Single))
}
// Received in 2026 under a 2018 decree, no child support and no recapture.
fn plain(received: i64) -> AlimonyPayment {
    AlimonyPayment {
        paid: received,
        alimony_due: received,
        child_support_due: 0,
        ..alimony_record(20181231)
    }
}
// Third-year recapture: first qualifying payments in 2024.
fn third_year(first: i64, second: i64, current: i64) -> AlimonyPayment {
    AlimonyPayment {
        recapture: AlimonyRecapture::ThirdYear {
            first,
            second,
            first_excluded: 0,
            second_excluded: 0,
            current_excluded: 0,
            death_or_remarriage: false,
        },
        ..plain(current)
    }
}
fn agi(s: &Interview) -> i64 {
    return_line(&supported(&s.draft), "11a / 11b")
}

#[test]
fn alimony_received_through_the_topic_by_hand() {
    // A 2018 decree: $18,000 received in 2026, $12,000 alimony and $8,000
    // child support due. Child support is satisfied first (Pub. 504,
    // Underpayment): 18,000 - 8,000 = $10,000, no more than the $12,000
    // due, so Schedule 1 line 2a = $10,000.
    //   line 8 = $10,000; AGI = 60,000 + 10,000        = $70,000
    //   line 15 = 70,000 - 16,100                       = $53,900
    //   line 16: row $53,900-$53,950, midpoint $53,925:
    //     1,240 + 4,560 + 22% x (53,925 - 50,400) = 5,800 + 775.50
    //                                             = $6,575.50 -> $6,576
    let mut s = single();
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, alimony_record(20181231))]));
    let stored: Vec<_> = s
        .draft
        .income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::Alimony { .. }))
        .cloned()
        .collect();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].owner, crate::DocumentOwner::Yours);
    // An unnamed source stores no name; its kind and identity name it.
    assert_eq!(stored[0].label, "");
    assert_eq!(stored[0].income, FederalIncome::Alimony { data: alimony_record(20181231) });
    let r = supported(&s.draft);
    assert_eq!(value(&r, "taxable_alimony"), 10_000);
    assert_eq!(return_line(&r, "8"), 1_000_000);
    assert_eq!(return_line(&r, "11a / 11b"), 7_000_000);
    assert_eq!(return_line(&r, "15"), 5_390_000);
    assert_eq!(return_line(&r, "16"), 657_600);
    // The row preview is the line 2a amount.
    let P::SaveIncomes { documents, .. } = topic_page(&mut s) else { unreachable!() };
    assert_eq!(documents.documents.len(), 1);
    // Round trip: stored, restarted and reopened, the page shows the same
    // agreement, and saving it unchanged keeps the revision.
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let P::SaveIncomes { documents, .. } = topic_page(&mut again) else { unreachable!() };
    assert_eq!(documents.documents.len(), 1);
    assert_eq!(documents.documents[0].owner, Owner::Taxpayer);
    assert_eq!(
        documents.documents[0].document,
        crate::IncomeRecord::AlimonyReceived(alimony_record(20181231))
    );
    let request = topic_page(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved);
}

#[test]
fn alimony_received_recapture_deduction_by_hand() {
    // Pub. 504's example from the recipient's side: $50,000 received in
    // 2024, $39,000 in 2025 and $28,000 in 2026 (Worksheet 1):
    //   line 5  = 39,000 - (28,000 + 15,000) < 0        -> $0
    //   line 7  = 39,000 - 0                             = $39,000
    //   line 10 = (39,000 + 28,000) / 2                  = $33,500
    //   line 13 = 50,000 - (33,500 + 15,000)             = $1,500
    //   line 14 = 0 + 1,500                              = $1,500
    // Line 2a = $28,000; the $1,500 is deducted on line 19a.
    //   AGI = 60,000 + 28,000 - 1,500                    = $86,500
    //   line 15 = 86,500 - 16,100                        = $70,400
    //   line 16: row $70,400-$70,450, midpoint $70,425:
    //     5,800 + 22% x (70,425 - 50,400) = 5,800 + 4,405.50
    //                                     = $10,205.50 -> $10,206
    let mut s = single();
    accepted(&save(
        &mut s,
        vec![received(Owner::Taxpayer, third_year(5_000_000, 3_900_000, 2_800_000))],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "taxable_alimony"), 28_000);
    assert_eq!(value(&r, "alimony_recapture_deduction"), 1_500);
    assert_eq!(value(&r, "alimony_paid_deduction"), 1_500);
    assert_eq!(return_line(&r, "11a / 11b"), 8_650_000);
    assert_eq!(return_line(&r, "15"), 7_040_000);
    assert_eq!(return_line(&r, "16"), 1_020_600);
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let request = topic_page(&mut again);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, saved);
}

#[test]
fn alimony_received_agreement_date_boundary_is_january_1_2019() {
    // Line 2a: an agreement entered into on or before December 31, 2018 is
    // taxable ($10,000); one entered into January 1, 2019 is not.
    let mut s = single();
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, alimony_record(20190101))]));
    assert_eq!(agi(&s), 6_000_000);
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, alimony_record(20181231))]));
    assert_eq!(agi(&s), 7_000_000);
}

#[test]
fn alimony_received_changed_to_the_new_rules() {
    // Changed before 2026 to adopt the post-2018 rules (Pub. 504 Example
    // 2's later years): nothing is taxable. Changed during 2026: the year
    // splits at the change, refused by name and opening this topic.
    let mut s = single();
    let mut payment = alimony_record(20181231);
    payment.new_rules = AlimonyNewRules::BeforeReturnYear;
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment.clone())]));
    assert_eq!(agi(&s), 6_000_000);
    payment.new_rules = AlimonyNewRules::DuringReturnYear;
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment.clone())]));
    unsupported(&s.draft, "Alimony received");
    assert_eq!(
        crate::interview::issue_page(
            &s.draft,
            &crate::IssueContext::Section(TaxSection::OtherIncome),
            None
        ),
        Some(TaxSection::OtherIncome)
    );
    // A 2019 agreement is outside the old rules whatever the answer.
    payment.agreement = 20190101;
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment)]));
    assert_eq!(agi(&s), 6_000_000);
}

#[test]
fn alimony_received_no_more_than_child_support_due_is_not_taxable() {
    // Received $8,000 = the child support due: nothing is alimony.
    // Received $8,001: $1 of alimony.
    let mut s = single();
    let mut payment = alimony_record(20181231);
    payment.paid = 800_000;
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment.clone())]));
    assert_eq!(agi(&s), 6_000_000);
    payment.paid = 800_100;
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment)]));
    assert_eq!(agi(&s), 6_000_100);
}

#[test]
fn recapture_second_year_threshold_is_15000() {
    // Worksheet 1 line 5: the third year falls from the second by more
    // than $15,000. First = second = $45,000 keeps line 13 at zero:
    //   third $30,000: line 5 = 45,000 - 45,000 = 0; line 13 =
    //     45,000 - ((45,000 + 30,000) / 2 + 15,000) < 0 -> recapture $0
    //   third $29,999: line 5 = $1; line 7 = 44,999; line 13 =
    //     45,000 - ((44,999 + 29,999) / 2 + 15,000) = 45,000 - 52,499 < 0
    //     -> recapture $1
    for (third, recaptured) in [(3_000_000, 0), (2_999_900, 1)] {
        let mut s = single();
        accepted(&save(
            &mut s,
            vec![received(Owner::Taxpayer, third_year(4_500_000, 4_500_000, third))],
        ));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "alimony_recapture_deduction"), recaptured, "{third}");
        assert_eq!(return_line(&r, "11a / 11b"), 6_000_000 + third - recaptured * 100, "{third}");
    }
}

#[test]
fn recapture_first_year_threshold_is_15000_over_the_average() {
    // Worksheet 1 line 13: second = third = $30,000 (line 5 = 0, line 10 =
    // $30,000, line 12 = $45,000). First $45,000 -> $0; first $45,001 -> $1.
    for (first, recaptured) in [(4_500_000, 0), (4_500_100, 1)] {
        let mut s = single();
        accepted(&save(
            &mut s,
            vec![received(Owner::Taxpayer, third_year(first, 3_000_000, 3_000_000))],
        ));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "alimony_recapture_deduction"), recaptured, "{first}");
    }
}

#[test]
fn recapture_ends_with_death_or_remarriage() {
    // Payments that stopped because the recipient remarried: no recapture
    // (Pub. 504), though Worksheet 1 would give $51,500 for $50,000,
    // $39,000 and nothing in 2026.
    let mut payment = third_year(5_000_000, 3_900_000, 0);
    let mut s = single();
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment.clone())]));
    assert_eq!(value(&supported(&s.draft), "alimony_recapture_deduction"), 51_500);
    let AlimonyRecapture::ThirdYear { death_or_remarriage, .. } = &mut payment.recapture else {
        unreachable!()
    };
    *death_or_remarriage = true;
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment.clone())]));
    assert_eq!(value(&supported(&s.draft), "alimony_recapture_deduction"), 0);
    let AlimonyRecapture::ThirdYear { death_or_remarriage, .. } = &mut payment.recapture else {
        unreachable!()
    };
    *death_or_remarriage = false;
    // Not sure whether the payments qualify: the recipient is taxed on them
    // as alimony, deducts no recapture, and the assumption is noted.
    payment.eligible = false;
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alimony_recapture_deduction"), 0);
}

#[test]
fn alimony_received_rules_refuse_on_the_server() {
    let mut s = single();
    let before = s.draft.clone();
    // A spouse's agreement on a single return is outside allowed_owners.
    problem(current_document_refusal(
        "income-source-114",
        &mut s,
        vec![received(Owner::Spouse, alimony_record(20181231))],
        "MESSAGE_ALIMONY_YOUR_SPOUSE_RECEIVED_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "[documents/documents/0/owner]",
    ));
    // The recapture exclusions above the years' payments ($50,000.01 of
    // $50,000; $39,000.01 of $39,000).
    let mut payment = recapture_alimony();
    if let AlimonyRecapture::ThirdYear { first_excluded, .. } = &mut payment.recapture {
        *first_excluded = 5_000_001;
    }
    rejected_with(
        save(&mut s, vec![received(Owner::Taxpayer, payment)]),
        "ALIMONY_2024_EXCLUDED_ABOVE_PAID",
    );
    let mut payment = recapture_alimony();
    if let AlimonyRecapture::ThirdYear { second_excluded, .. } = &mut payment.recapture {
        *second_excluded = 3_900_001;
    }
    rejected_with(
        save(&mut s, vec![received(Owner::Taxpayer, payment)]),
        "ALIMONY_2025_EXCLUDED_ABOVE_PAID",
    );
    // This year's excluded part above this year's alimony ($28,000.01).
    let mut payment = recapture_alimony();
    if let AlimonyRecapture::ThirdYear { current_excluded, .. } = &mut payment.recapture {
        *current_excluded = 2_800_001;
    }
    problem(current_document_refusal(
        "income-source-142",
        &mut s,
        vec![received(Owner::Taxpayer, payment)],
        "MESSAGE_ALIMONY_RECEIVED_ENTER_THE_AGREEMENT_DATE_BEFORE_2027_AND_AMOUNTS_WITH_THIS",
        "[documents/documents/0/document/recapture/current_excluded]",
    ));
    assert_eq!(s.draft, before);
    // Exactly the years' payments is accepted.
    let mut payment = recapture_alimony();
    if let AlimonyRecapture::ThirdYear { first_excluded, second_excluded, .. } =
        &mut payment.recapture
    {
        *first_excluded = 5_000_000;
        *second_excluded = 3_900_000;
    }
    accepted(&save(&mut s, vec![received(Owner::Taxpayer, payment)]));
}

#[test]
fn the_topic_offers_alimony_received_and_less_common_income_no_longer_names_it() {
    let mut s = single();
    let Reply::Edit(P::SaveIncomes { allowed_documents, .. }) =
        from_review(&mut s, TaxSection::OtherIncome)
    else {
        panic!("expected the topic")
    };
    assert!(allowed_documents.contains(
        &(libertas_macros::variant_index!(crate::IncomeSourceKind::AlimonyReceived) as i32)
    ));
}

#[test]
fn a_spouse_agreement_on_a_joint_return_and_leaving_joint_erases_it() {
    // On a joint return the spouse's agreement is accepted and taxable
    // ($10,000 each: AGI 60,000 + 20,000 on complete(Joint)'s W-2).
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let before = agi(&s);
    accepted(&save(
        &mut s,
        vec![
            received(Owner::Taxpayer, alimony_record(20181231)),
            received(Owner::Spouse, plain(1_000_000)),
        ],
    ));
    assert_eq!(agi(&s), before + 2_000_000);
    // Leaving the joint return: the spouse's agreement is outside
    // allowed_owners and goes, named; the filer's stays.
    let mut d = s.draft.clone();
    let mut kept = d.clone();
    assert!(crate::interview::prune(&mut kept).removed.is_empty());
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    let result = crate::interview::prune(&mut d);
    let owners: Vec<Owner> = d
        .income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::Alimony { .. }))
        .map(|e| e.owner.owned())
        .collect();
    assert_eq!(owners, vec![Owner::Taxpayer]);
    assert!(
        result.removed.iter().any(|item| item.key() == "SAVING_KIND_ALIMONY_AGREEMENT"),
        "{result:?}"
    );
}
