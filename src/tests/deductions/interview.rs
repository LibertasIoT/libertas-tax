use crate::tests::support::client::income_pages::open_document_center;
// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::first_lender_mortgage;
use crate::tests::cross::interview::{
    PAGE_ADJUSTMENTS, PAGE_DEDUCTIONS, PAGE_INCOME, PAGE_ITEMIZED, PAGE_REVIEW,
    assert_available_sections, at_the_deductions_page, at_the_gifts_page, back, checklist_of,
    continue_hub, deductions_value, empty_facts, gifts, go_on, go_to, goods_gift, hub_rows,
    itemized_hub, navigate, not_started_rows, open_itemized_topic, past_credits, refusal, row_of,
    save_deductions, save_gift_carryovers, save_medical, save_often_missed, save_spouse_boxes,
    send_past_less_common, shown, stored_facts, walk_on_to_review,
};

#[test]
fn educator_and_1098e_pages_map_onto_their_claims() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let sources = open_document_center(&mut s);
    accepted(&send(&mut s, sources));
    // Educator: one row reaches the taxpayer's claim.
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::Educator);
    let Reply::Edit(P::SaveEducator { allowed_owners, .. }) = send(&mut s, request) else {
        panic!("expected the educator page")
    };
    let request = save_educator(
        &s.draft,
        allowed_owners,
        EducatorExpenses {
            people: vec![Educator {
                owner: Owner::Taxpayer,
                role: EducatorRole::Educator { classroom: 30_000, instructional: 0 },
            }],
        },
    );
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    assert_eq!(
        s.draft.adjustments.as_ref().unwrap().educators,
        EducatorClaim {
            taxpayer: Some(EducatorRole::Educator { classroom: 30_000, instructional: 0 }),
            spouse: None,
        }
    );
    // Reopening shows the same row.
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::Educator);
    let Reply::Edit(P::SaveEducator { value, .. }) = send(&mut s, request) else {
        panic!("expected the educator page")
    };
    assert_eq!(
        value.people,
        vec![Educator {
            owner: Owner::Taxpayer,
            role: EducatorRole::Educator { classroom: 30_000, instructional: 0 },
        }]
    );
    // An empty page is No.
    let request =
        save_educator(&s.draft, vec![Owner::Taxpayer as i32], EducatorExpenses { people: vec![] });
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    assert_eq!(s.draft.adjustments.as_ref().unwrap().educators, EducatorClaim::NONE);

    // Student loan: two Forms 1098-E, one with box 2 checked, and interest
    // paid with no form are stored as entered and reopen unchanged (T2-33x).
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::StudentLoan);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveStudentLoan { .. })));
    let first_form_id = s.draft.next_id;
    let mut forms = vec![box_1(30_000), box_2(45_000, 5_000)];
    for form in &mut forms {
        form.document.borrower = fixture_document_names(&s.draft, -1).unwrap().recipient;
    }
    let mut center = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut center else { unreachable!() };
    documents.common.student_loans.documents = student_documents(&forms);
    accepted(&send(&mut s, center));
    for (index, form) in forms.iter_mut().enumerate() {
        form.id = first_form_id + index as i64;
    }
    let navigation = go_to(&s.draft, PAGE_REVIEW, TaxSection::StudentLoan);
    let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(&mut s, navigation) else {
        panic!("expected student loan topic")
    };
    let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
    value.other_interest = 12_000;
    value.forms[1].additional_interest =
        crate::StudentLoanAdditionalInterest::Checked { additional_interest: 5_000 };
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    let adjustments = s.draft.adjustments.as_ref().unwrap();
    assert_eq!(
        adjustments.student_loan,
        StudentLoanClaim {
            student_loan_interest: 12_000,
            forms: forms
                .iter()
                .cloned()
                .map(|source| crate::StudentLoanSource {
                    assigned: crate::DocumentRecipient::Taxpayer,
                    owner_source: Some(crate::OwnerSource::Matched),
                    ..source
                })
                .collect(),
            employer_paid_interest: 0,
        }
    );
    // 300.00 + 450.00 + 50.00 box 2 + 120.00 with no form.
    assert_eq!(adjustments.student_loan_interest(), 92_000);
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::StudentLoan);
    let Reply::Edit(P::SaveStudentLoan { value, .. }) = send(&mut s, request) else {
        panic!("expected the student loan page")
    };
    assert_eq!(
        value.forms.into_iter().map(|row| row.value).collect::<Vec<_>>(),
        forms.iter().map(|source| source.document.clone()).collect::<Vec<_>>()
    );
    assert_eq!(value.other_interest, 12_000);
    // Interest with no form alone is a claim.
    let mut center = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut center else { unreachable!() };
    documents.common.student_loans.documents.clear();
    accepted(&send(&mut s, center));
    let request = save_student_loan(
        &s.draft,
        StudentLoanInterest { forms: vec![], other_interest: 40_000, employer_paid_interest: 0 },
    );
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    assert_eq!(
        s.draft.adjustments.as_ref().unwrap().student_loan,
        StudentLoanClaim {
            student_loan_interest: 40_000,
            forms: vec![],
            employer_paid_interest: 0,
        }
    );
    // An empty page is No.
    let request = save_student_loan(
        &s.draft,
        StudentLoanInterest { forms: vec![], other_interest: 0, employer_paid_interest: 0 },
    );
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    assert_eq!(s.draft.adjustments.as_ref().unwrap().student_loan, StudentLoanClaim::NONE);
}

#[test]
fn automatic_deduction_is_the_default() {
    // The deductions page follows the adjustments hub and opens complete on
    // the standard deduction with no gifts: M1's stored "nothing applies".
    let (mut s, page) = at_the_deductions_page(FederalStatus::Single);
    let P::SaveDeductions { page: at, previous, progress, .. } = &page else { unreachable!() };
    // Back opens adjustments. Documents and income without a form precede
    // the checklist and financial sections; payments appears only when needed.
    assert_eq!((*at, *previous), (PAGE_DEDUCTIONS, PAGE_ADJUSTMENTS));
    assert!(shown(progress).contains("2026 federal · section 9 of 12"));
    let value = deductions_value(&page);
    assert_eq!(
        value,
        &Deductions {
            car_refunds: Default::default(),
            itemize_override: None,
            cash_gifts: 0,
            fund_cash_gifts: 0,
            other_cash_gifts: 0,
            car_loans: vec![],
            casualties: vec![],
            repayment: RepaymentClaim::None,
        }
    );
    // Back returns to the income hub.
    let request = back(&s.draft, PAGE_INCOME);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Hub { page: PAGE_INCOME, .. })));
    // Next preserves M1's record and passes the empty credits summary to
    // the last required question.
    let before = s.draft.clone();
    assert!(matches!(
        send(&mut s, page),
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
    let m1 = complete(FederalStatus::Single);
    assert_eq!(s.draft.deductions, m1.deductions);
    assert_eq!(s.draft.setup, m1.setup);
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Documents,
            TaxSection::Deductions,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
        ]
    );
    // The empty standard route was already completed during the ordinary walk.
    // Confirming it again is an exact no-op.
    assert_eq!(s.draft, before);
    assert_eq!(estimate(&s.draft).return_lines, estimate(&m1).return_lines);
    // The itemized hub does not exist for the standard deduction.
    let request = navigate(&s.draft, TaxSection::Itemized);
    assert!(refusal(send(&mut s, request)).contains("This section is not available yet."));
}

#[test]
fn automatic_deductions_walk_accepted_medical_and_investment_plans() {
    for (bit, topic) in [
        (MissedBenefits::MEDICAL, TaxSection::Medical),
        (MissedBenefits::INVESTMENT_INTEREST, TaxSection::OtherItemized),
    ] {
        let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
        let mut planned = s.draft.often_missed.clone();
        planned.benefits |= bit.0;
        accepted(&save_often_missed(&mut s, planned.clone()));
        assert!(crate::interview::planned_sections(&s.draft).contains(&topic));
        let request = save_deductions(&s.draft, None, 0);
        let reply = send_past_less_common(&mut s, request);
        assert!(
            matches!(
                (&reply, topic),
                (Reply::Edit(P::SaveMedical { .. }), TaxSection::Medical)
                    | (Reply::Edit(P::SaveOtherItemized { .. }), TaxSection::OtherItemized)
            ),
            "{topic:?}: {reply:?}"
        );
        assert_eq!(s.draft.deductions.as_ref().unwrap().itemize_override, None);
        assert_eq!(stored_facts(&s), empty_facts());
        planned.benefits &= !bit.0;
        accepted(&save_often_missed(&mut s, planned));
        assert!(!crate::interview::planned_sections(&s.draft).contains(&topic));
        assert_eq!(s.draft.deductions.as_ref().unwrap().itemize_override, None);
        let request = navigate(&s.draft, TaxSection::Itemized);
        assert!(matches!(send(&mut s, request), Reply::Response(P::Problem { .. })));
        assert_eq!(restart(s.draft.clone()).draft, s.draft);
    }
}

#[test]
fn automatic_and_override_preserve_the_same_itemized_facts() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    // 2026-11-02: automatic comparison uses the planned deduction topics.
    s.draft.checklist.paid |= PaidTopics::TAXES.0;
    let request = save_deductions(&s.draft, None, 0);
    let hub =
        crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    assert_eq!(s.draft.deductions.as_ref().unwrap().itemize_override, None);
    assert_eq!(stored_facts(&s), empty_facts());
    let P::Hub { previous, suggested, allowed_sections, progress, .. } = &hub else {
        unreachable!()
    };
    // The planned tax payment makes Taxes the next suggested topic.
    assert_eq!(*previous, PAGE_DEDUCTIONS);
    assert_eq!(*suggested, TaxSection::Taxes);
    // The available destinations include the checklist, all five itemized
    // topics, and credits as the next shown section.
    assert_available_sections(
        allowed_sections,
        &[
            TaxSection::Deductions,
            TaxSection::Checklist,
            TaxSection::Medical,
            TaxSection::Taxes,
            TaxSection::Mortgage,
            TaxSection::GiftCarryovers,
            TaxSection::OtherItemized,
            TaxSection::Credits,
        ]
        .map(|t| t as i32),
    );
    // Setup, you, people, documents, income without a form, checklist, income,
    // adjustments, deductions, credits, other taxes and review; returning
    // from the tax editor displays the itemized section summary.
    assert!(
        shown(progress).contains("section 9 of 12 · Itemized deductions"),
        "{}",
        shown(progress)
    );
    assert!(!shown(progress).contains("You're done with itemized deductions"));
    // Taxes is planned; the other topics are not started. No topic has
    // entered facts or a saved visit yet, and every destination is a row.
    assert_eq!(
        hub_rows(&hub),
        core::iter::once(("Taxes you paid".into(), "Planned".into(), TaxSection::Taxes))
            .chain(not_started_rows(&[
                TaxSection::Medical,
                TaxSection::Mortgage,
                TaxSection::GiftCarryovers,
                TaxSection::OtherItemized
            ]))
            .collect::<Vec<_>>()
    );
    // Medical: the expenses and one premium for you, whose age at the end of
    // 2026 (born June 15, 1980: 46) comes from your page.
    let page = open_itemized_topic(&mut s, TaxSection::Medical);
    let P::SaveMedical { value, previous, .. } = &page else {
        panic!("expected the medical page, got {page:?}")
    };
    assert_eq!(*previous, PAGE_ITEMIZED);
    assert_eq!(value.expenses, 0);
    assert!(value.long_term_care.is_empty());
    let labels: Vec<_> = value.people.iter().map(|p| status_text(&p.label)).collect();
    assert_eq!(labels, vec!["You"]);
    let request = save_medical(
        &s.draft,
        Medical {
            expenses: 1_200_000,
            long_term_care: vec![CarePremium {
                qualified: CareContractQualification::Qualified,
                person: -1,
                premiums: 120_000,
            }],
            people: value.people.clone(),
        },
    );
    let hub =
        crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    let facts = stored_facts(&s);
    assert_eq!(facts.medical, 1_200_000);
    // The premium names its person by identity; the age that sets its limit
    // is read from the people page (46 here), shared with the other coverage
    // records by that identity.
    assert_eq!(
        facts.long_term_care,
        vec![MedicalCarePremium {
            person: -1,
            premiums: 120_000,
            qualified: CareContractQualification::Qualified,
        }]
    );
    assert_eq!(crate::interview::person_age(&s.draft, -1), Some(46));
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Taxes);
    assert_eq!(
        hub_rows(&hub)[0],
        row_of("Medical and dental expenses", "$13200.00", TaxSection::Medical)
    );
    // Reopened, the premium names its person again.
    let P::SaveMedical { value, .. } = open_itemized_topic(&mut s, TaxSection::Medical) else {
        panic!("expected the medical page")
    };
    assert_eq!(
        value.long_term_care,
        vec![CarePremium {
            qualified: CareContractQualification::Qualified,
            person: -1,
            premiums: 120_000
        }]
    );
    // Taxes: state income tax and property tax onto the SALT election.
    let page = open_itemized_topic(&mut s, TaxSection::Taxes);
    let P::SaveTaxes { value, .. } = &page else { panic!("expected the taxes page, got {page:?}") };
    assert_eq!(
        value,
        &Taxes {
            income_tax_owners: None,
            investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
            kind: SaltElection::IncomeTax { paid: 0 },
            real_estate: 0,
            personal_property: 0,
        }
    );
    let request = P::SaveTaxes {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: PAGE_ITEMIZED,
        previous: PAGE_ITEMIZED,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value: Taxes {
            income_tax_owners: None,
            investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
            kind: SaltElection::IncomeTax { paid: 800_000 },
            real_estate: 700_000,
            personal_property: 0,
        },
    };
    let hub =
        crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    let facts = stored_facts(&s);
    assert_eq!(
        facts.personal_tax,
        PersonalTaxElection::Income {
            paid: 800_000,
            allocation: TaxIncomeAllocation::CurrentReturn,
        }
    );
    assert_eq!(facts.real_estate_taxes, 700_000);
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Credits);
    // Home mortgage: one Form 1098, within the Pub 936 limit.
    let loan_id = s.draft.next_id;
    let page = open_itemized_topic(&mut s, TaxSection::Mortgage);
    let P::SaveMortgages { value, .. } = &page else {
        panic!("expected the mortgage page, got {page:?}")
    };
    assert!(value.loans.is_empty());
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let borrower = fixture_document_names(&s.draft, -1).unwrap().recipient;
    documents.common.mortgages.documents.push(mortgage_document(Mortgage1098 {
        borrower: borrower.clone(),
        ..first_lender_mortgage(900_000, 40_000_000, AverageBalance::WithinLimit)
    }));
    accepted(&send(&mut s, request));
    let request = navigate(&s.draft, TaxSection::Itemized);
    let hub = itemized_hub(send(&mut s, request));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().mortgage.iter().map(mortgage_loan).collect::<Vec<_>>(),
        vec![MortgageLoan {
            printed: Some(crate::PrintedDocumentNames {
                recipient: borrower,
                issuer: Some(Default::default()),
                assigned: crate::DocumentRecipient::Taxpayer,
                owner_source: Some(crate::OwnerSource::Matched),
                ..Default::default()
            }),
            id: loan_id,
            property: "".into(),
            lender: MortgageLender::Institution,
            interest: 900_000,
            outstanding_principal: 40_000_000,
            incurred: 20200601,
            refund_of_interest: MortgageInterestRefund::No,
            mortgage_insurance_premiums: 0,
            points: MortgagePoints::No,
            details: MortgageLoanDetails::FirstLender(MortgageLedger {
                home: MortgageHome::Main,
                purpose: MortgageUse::Home,
                average: MortgageAverageBalance::WithinLimit
            }),
        }]
    );
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Credits);
    // Reopened, the loan comes back unchanged.
    let P::SaveMortgages { value, .. } = open_itemized_topic(&mut s, TaxSection::Mortgage) else {
        panic!("expected the mortgage page")
    };
    assert_eq!(value.loans.len(), 1);
    assert_eq!(value.loans[0].value.box_1, 900_000);
    // Charity carryovers: one prior-year balance.
    let page = open_itemized_topic(&mut s, TaxSection::GiftCarryovers);
    let P::SaveGiftCarryovers { value, .. } = &page else {
        panic!("expected the gift carryovers page, got {page:?}")
    };
    assert!(value.carryovers.is_empty());
    let request = save_gift_carryovers(
        &s.draft,
        GiftCarryovers {
            gifts: vec![],
            vehicles: vec![],
            appraisals: vec![],
            appraisal_choices: vec![],
            basis_election: false,
            carryovers: vec![
                CharityCarryover {
                    research_parts: None,
                    year: 2025,
                    class: CharityClass::Cash60,
                    balance: 500_000,
                    appreciation: None,
                    same_filers: true,
                }
                .into(),
            ],
            volunteer: vec![],
            less_common: vec![],
        },
    );
    let hub =
        crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().charity_carryovers,
        vec![CharityCarryover {
            research_parts: None,
            year: 2025,
            class: CharityClass::Cash60,
            balance: 500_000,
            appreciation: None,
            same_filers: true
        }]
    );
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Credits);
    // Investment interest with a carryover and investment property taxes.
    let request = P::SaveOtherItemized {
        investment_k1_sources: crate::investment_interest::k1_sources(&s.draft),
        investment_origin_choices: crate::investment_interest::origin_choices(&s.draft),
        interest_sources: crate::investment_interest::sources(&s.draft),
        allowed_owners: vec![0],
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: PAGE_ITEMIZED,
        previous: PAGE_ITEMIZED,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value: OtherItemized {
            investment_property_tax_owners: None,
            investment_interest: Some(InvestmentBorrowingPage {
                loans: vec![InvestmentLoanPage {
                    paid_owners: None,
                    carry_owners: None,
                    originating_k1: None,
                    allowed_origin_choices: vec![],
                    label: "Portfolio loan".into(),
                    source: 0,
                    paid: 100_000,
                    carry: 20_000,
                    amt_carry: 15_000,
                    private_activity: 0,
                }],
                election: InvestmentElection::None,
                income_use: InvestmentInterestUse::Unresolved,
                limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
            }),
            investment_property_taxes: 30_000,
            annuity_losses: vec![],
            contracts: vec![],
            other_lines: vec![],

            bond_instruments: Vec::new(),
        },
    };
    let hub =
        crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    let facts = stored_facts(&s);
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().investment_interest(),
        Some(InvestmentInterest {
            paid: 100_000,
            carryover: 20_000,
            amt_carryover: 15_000,
            private_activity: 0,
            election: InvestmentElection::None,
            income_use: InvestmentInterestUse::Unresolved,
        })
    );
    assert_eq!(facts.investment_property_taxes, 30_000);
    // Continue records this section and walks the later sections to Review,
    // including each applicable less-common page.
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Credits);
    let reply = send(&mut s, go_on(&hub));
    walk_on_to_review(&mut s, reply);
    assert!(s.draft.finished_sections.contains(&TaxSection::Itemized));
    // Itemize is the line 19 election with the same facts.
    let compared = stored_facts(&s);
    let request = save_deductions(&s.draft, Some(ItemizeOverride::Itemize), 0);
    itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().itemize_override,
        Some(ItemizeOverride::Itemize)
    );
    assert_eq!(stored_facts(&s), compared);
    let request = navigate(&s.draft, TaxSection::Deductions);
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the deductions page") };
    assert_eq!(deductions_value(&page).itemize_override, Some(ItemizeOverride::Itemize));
    // The accepted source editor and each record topic save unchanged.
    let before = s.draft.clone();
    let source = open_document_center(&mut s);
    accepted(&send(&mut s, source));
    for topic in [
        TaxSection::Medical,
        TaxSection::Taxes,
        TaxSection::GiftCarryovers,
        TaxSection::OtherItemized,
    ] {
        let page = open_itemized_topic(&mut s, topic);
        itemized_hub(send_past_less_common(&mut s, page));
    }
    assert_eq!(s.draft, before);
}

// 2026-11-02: spouse itemizing is an independent required separate-return fact.
#[test]
fn spouse_itemizing_is_required_only_on_a_separate_return_and_survives_restart() {
    use crate::tests::support::builders::spouse_itemizes_request;
    let (mut s, mut page) = at_the_deductions_page(FederalStatus::Separate);
    s.draft.deductions.as_mut().unwrap().spouse_itemizes = None;
    let P::SaveDeductions { cookie, revision, .. } = &mut page else { unreachable!() };
    *cookie = s.draft.cookie.clone();
    *revision = s.draft.revision;
    assert!(matches!(send(&mut s, page), Reply::Response(P::BeginSpouseItemizes { .. })));
    assert!(
        estimate(&s.draft).issues.iter().any(|i| i.display.key == "ISSUE_ANSWER_SPOUSE_ITEMIZES")
    );
    let request = spouse_itemizes_request(&s.draft, true);
    accepted(&send(&mut s, request));
    assert!(s.draft.spouse_itemizes());
    assert_eq!(
        supported(&s.draft).return_lines,
        supported(&restart(s.draft.clone()).draft).return_lines
    );
    let request = spouse_itemizes_request(&s.draft, false);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.deductions.as_ref().unwrap().spouse_itemizes, Some(false));
    assert!(!s.draft.spouse_itemizes());
    let (mut single, _) = at_the_deductions_page(FederalStatus::Single);
    let before = single.draft.clone();
    let request = spouse_itemizes_request(&single.draft, true);
    assert!(matches!(send(&mut single, request), Reply::Response(P::Problem { .. })));
    assert_eq!(single.draft, before);
}

// Your spouse's standard deduction boxes are asked on their own step after the
// deductions page of a separate return while the standard deduction can
// apply: unanswered on a client-built editor until saved, never seeded and
// never on the deductions page, then reopened with the saved answer. A spouse
// who itemizes leaves them unasked.
#[test]
fn separate_return_asks_spouse_boxes_on_their_own_step_unanswered() {
    let issue = "ISSUE_ANSWER_YOUR_SPOUSE_STANDARD_DEDUCTION_BOXES";
    let asks = |d: &FederalDraft| estimate(d).issues.iter().any(|i| i.display.key == issue);
    let boxes = |s: &Interview| s.draft.setup.as_ref().unwrap().separate_spouse;
    let (mut s, _) = at_the_deductions_page(FederalStatus::Separate);
    s.draft.setup.as_mut().unwrap().separate_spouse = crate::SeparateSpouseBoxes::NotAnswered;
    assert!(asks(&s.draft));
    let request = save_deductions(&s.draft, None, 0);
    accepted(&send(&mut s, request));
    let request = crate::tests::support::builders::spouse_itemizes_request(&s.draft, false);
    let Reply::Response(P::BeginSpouseBoxes { page, previous, .. }) = send(&mut s, request) else {
        panic!("the step opens unanswered after the deductions page")
    };
    assert_eq!((page, previous), (PAGE_DEDUCTIONS, PAGE_DEDUCTIONS));
    assert_eq!(boxes(&s), crate::SeparateSpouseBoxes::NotAnswered);
    // The estimate's issue opens the same step.
    let request = navigate(&s.draft, TaxSection::Deductions);
    assert!(matches!(send(&mut s, request), Reply::Response(P::BeginSpouseBoxes { .. })));
    let request = save_spouse_boxes(&s.draft, SpouseBoxes::Blind);
    accepted(&send(&mut s, request));
    assert_eq!(boxes(&s), crate::SeparateSpouseBoxes::Blind);
    assert!(!asks(&s.draft));
    // The saved answer reopens by EditRequest after the deductions page.
    let request = navigate(&s.draft, TaxSection::Deductions);
    let Reply::Edit(mut page @ P::SaveDeductions { .. }) = send(&mut s, request) else {
        panic!("expected the deductions page")
    };
    // Its Next as the walk reaches it (not back to the review).
    let P::SaveDeductions { review, .. } = &mut page else { unreachable!() };
    *review = false;
    assert!(matches!(
        {
            accepted(&send(&mut s, page));
            let request = crate::tests::support::builders::spouse_itemizes_request(&s.draft, false);
            send(&mut s, request)
        },
        Reply::Edit(P::SaveSpouseBoxes { value: SpouseBoxes::Blind, .. })
    ));
    // A spouse who itemizes files, so the boxes are no longer asked.
    let request = crate::tests::support::builders::spouse_itemizes_request(&s.draft, true);
    send(&mut s, request);
    assert_eq!(boxes(&s), crate::SeparateSpouseBoxes::NotAnswered);
    assert!(!asks(&s.draft));
    // No other return has the step.
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_spouse_boxes(&s.draft, SpouseBoxes::NoneOfThese);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Problem { .. })));
}

#[test]
fn clearing_itemize_override_preserves_facts_and_topic_completion() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    s.draft.checklist.paid |= PaidTopics::TAXES.0;
    let request = save_deductions(&s.draft, None, 0);
    crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    let request = P::SaveTaxes {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: PAGE_ITEMIZED,
        previous: PAGE_ITEMIZED,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value: Taxes {
            income_tax_owners: None,
            investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
            kind: SaltElection::SalesTax { paid: 150_000 },
            real_estate: 400_000,
            personal_property: 0,
        },
    };
    itemized_hub(send_past_less_common(&mut s, request));
    // Continue opens the next shown section, then the remaining walk reaches review.
    let request = continue_hub(&s.draft, PAGE_ITEMIZED);
    let reply = send(&mut s, request);
    walk_on_to_review(&mut s, reply);
    assert!(s.draft.finished_sections.contains(&TaxSection::Itemized));
    assert!(s.draft.finished_sections.contains(&TaxSection::Taxes));
    // Moving between the itemizing routes keeps the facts.
    let facts = stored_facts(&s);
    let request = save_deductions(&s.draft, Some(ItemizeOverride::Itemize), 0);
    itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(stored_facts(&s), facts);
    assert_eq!(facts.personal_tax, PersonalTaxElection::Sales { paid: 150_000 });
    assert_eq!(value(&supported(&s.draft), "deduction"), 5_500);
    // 2026-11-02: returning to automatic retains entered facts and saved topics.
    let finished = s.draft.finished_sections.clone();
    let request = save_deductions(&s.draft, None, 0);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.deductions.as_ref().unwrap().itemize_override, None);
    assert_eq!(stored_facts(&s), facts);
    assert_eq!(value(&supported(&s.draft), "deduction"), 16_100);
    assert_eq!(s.draft.finished_sections, finished);
    let mut restarted = restart(s.draft.clone());
    let page = open_itemized_topic(&mut restarted, TaxSection::Taxes);
    let P::SaveTaxes { value: taxes, .. } = &page else { unreachable!() };
    assert_eq!(taxes.kind, SaltElection::SalesTax { paid: 150_000 });
    assert_eq!(taxes.real_estate, 400_000);
    let mut larger = page;
    let P::SaveTaxes { value: taxes, .. } = &mut larger else { unreachable!() };
    taxes.real_estate = 2_000_000;
    accepted(&send(&mut restarted, larger));
    assert_eq!(value(&supported(&restarted.draft), "deduction"), 21_500);
    assert_eq!(restarted.draft.deductions.as_ref().unwrap().itemize_override, None);
    assert_eq!(
        supported(&restart(restarted.draft.clone()).draft).return_lines,
        supported(&restarted.draft).return_lines
    );
}

#[test]
fn cash_gifts_reach_the_non_itemizer_deduction() {
    // 2026 non-itemizer charitable deduction (section 170(p), Form 1040 line
    // 12f): cash gifts to public charities up to $1,000, or $2,000 on a
    // joint return, beside the standard deduction. By hand:
    //   single, gifts $1,500: min(1,500, 1,000) = $1,000
    //   single, gifts $1,000: min(1,000, 1,000) = $1,000 (at the limit)
    //   single, gifts $600:   min(600, 1,000)   = $600
    //   joint,  gifts $1,500: min(1,500, 2,000) = $1,500
    //   joint,  gifts $2,500: min(2,500, 2,000) = $2,000
    for (status, gifts, expected) in [
        (FederalStatus::Single, 150_000, 100_000),
        (FederalStatus::Single, 100_000, 100_000),
        (FederalStatus::Single, 60_000, 60_000),
        (FederalStatus::Joint, 150_000, 150_000),
        (FederalStatus::Joint, 250_000, 200_000),
    ] {
        let (mut s, _) = at_the_deductions_page(status);
        let request = save_deductions(&s.draft, None, gifts);
        let reply = send(&mut s, request);
        let Reply::Response(P::Review { result, .. }) = past_credits(&mut s, reply) else {
            panic!("expected the review")
        };
        assert_eq!(s.draft.deductions.as_ref().unwrap().cash_charity, gifts);
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
        assert_eq!(return_line(&result, "12f"), expected, "{status:?} {gifts}");
        // The page reopens with the gift.
        let request = navigate(&s.draft, TaxSection::Deductions);
        let Reply::Edit(page) = send(&mut s, request) else {
            panic!("expected the deductions page")
        };
        assert_eq!(deductions_value(&page).cash_gifts, gifts);
    }
    // No gift means no claim and no line 12f amount.
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_deductions(&s.draft, None, 0);
    let reply = send(&mut s, request);
    let Reply::Response(P::Review { result, .. }) = past_credits(&mut s, reply) else {
        panic!("expected the review")
    };
    assert_eq!(s.draft.deductions.as_ref().unwrap().cash_charity, 0);
    assert_eq!(return_line(&result, "12f"), 0);
}

#[test]
fn educator_twice_is_rejected_on_save() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let before = s.draft.clone();
    let request = save_educator(
        &before,
        vec![Owner::Taxpayer as i32, Owner::Spouse as i32],
        EducatorExpenses {
            people: vec![
                Educator {
                    owner: Owner::Spouse,
                    role: EducatorRole::Educator { classroom: 10_000, instructional: 0 },
                },
                Educator {
                    owner: Owner::Spouse,
                    role: EducatorRole::Educator { classroom: 20_000, instructional: 0 },
                },
            ],
        },
    );
    rejected_with(send(&mut s, request), "EDUCATOR_TWICE");
    assert_eq!(s.draft, before);
}

#[test]
fn unit15_automatic_keeps_gift_facts_and_all_cash_splits() {
    for (fund, other) in [(123_400, 0), (0, 234_500)] {
        let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
        let request = save_gift_carryovers(
            &s.draft,
            gifts(vec![goods_gift(GoodsKind::Books, 50_000, 20_000)], vec![], vec![]),
        );
        itemized_hub(send_past_less_common(&mut s, request));
        let property = s.draft.deductions.as_ref().unwrap().property_gifts.clone();
        let mut request = save_deductions(&s.draft, None, 0);
        let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
        value.fund_cash_gifts = fund;
        value.other_cash_gifts = other;
        accepted(&send(&mut s, request));
        let ded = s.draft.deductions.as_ref().unwrap();
        // 2026-11-02: automatic selection retains the accepted property facts.
        assert_eq!(ded.property_gifts, property);
        assert_eq!(
            (ded.cash_charity, ded.fund_cash_charity, ded.other_cash_charity),
            (0, fund, other)
        );
        let saved = s.draft.clone();
        let reply = send(&mut s, navigate(&saved, TaxSection::Checklist));
        assert_eq!(checklist_of(&reply).paid & PaidTopics::GIFTS.0, PaidTopics::GIFTS.0);
        assert_eq!(return_line(&supported(&s.draft), "12f"), 0);
        let mut again = restart(s.draft.clone());
        assert_eq!(again.draft, s.draft);
        let saved = again.draft.clone();
        let Reply::Edit(P::SaveDeductions { value, .. }) =
            send(&mut again, navigate(&saved, TaxSection::Deductions))
        else {
            unreachable!()
        };
        assert_eq!((value.fund_cash_gifts, value.other_cash_gifts), (fund, other));
    }
}

#[test]
fn mortgage_box4_calls_all_existing_refund_origins_only_for_a_positive_refund() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    for (refund, expected) in
        [(0, vec![]), (1, vec![0, 1, 2]), (50_000, vec![0, 1, 2]), (0, vec![])]
    {
        let mut request = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        if documents.common.mortgages.documents.is_empty() {
            let mut mortgage =
                first_lender_mortgage(100_000, 20_000_000, AverageBalance::WithinLimit);
            mortgage.borrower = accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
            documents.common.mortgages.documents.push(mortgage_document(mortgage));
        }
        documents.common.mortgages.documents[0].form.box_4 = refund;
        accepted(&send(&mut session, request));
        let before = session.draft.clone();
        let P::SaveMortgages { value, .. } =
            crate::tests::cross::interview::linked_mortgage_editor(&mut session)
        else {
            unreachable!()
        };
        assert_eq!(value.loans[0].allowed_refund_origins, expected);
        assert_eq!(session.draft, before);
    }
}

#[test]
fn mortgage_refund_choices_rebuild_on_reopen_without_changing_loan_facts_or_save_checks() {
    use crate::tests::cross::interview::{
        linked_mortgage_editor, linked_mortgage_form, linked_mortgage_rows,
        save_linked_mortgage_rows,
    };

    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut loan = linked_mortgage_form("Synthetic home");
    // The server uses actual box 4, never the editor's echoed allowed set.
    loan.more.allowed_refund_origins = vec![99];
    accepted(&save_linked_mortgage_rows(&mut session, vec![loan]));
    let mut loan = linked_mortgage_rows(&mut session).remove(0);
    assert!(loan.more.allowed_refund_origins.is_empty());
    let loan_facts = loan.more.details.clone();
    let lender = loan.more.lender.clone();
    let identity = loan.id;
    loan.refund = 10_000;
    let before = session.draft.clone();
    rejected_with(
        save_linked_mortgage_rows(&mut session, vec![loan.clone()]),
        "MORTGAGE_REFUND_ORIGIN_NEEDED",
    );
    assert_eq!(session.draft, before);
    // A withheld row never changes native refund treatment or weakens Save.
    loan.more.refund_origin = Some(MortgageRefundOrigin::ThisYear);
    accepted(&save_linked_mortgage_rows(&mut session, vec![loan]));
    assert_eq!(
        mortgage_loan(&session.draft.deductions.as_ref().unwrap().mortgage[0]).refund_of_interest,
        MortgageInterestRefund::Yes {
            amount: 10_000,
            origin: Some(MortgageRefundOrigin::ThisYear)
        },
    );
    let mut session = restart(session.draft);
    let P::SaveMortgages { .. } = linked_mortgage_editor(&mut session) else { unreachable!() };
    let mut loan = mortgage_snapshot(&session.draft.deductions.as_ref().unwrap().mortgage[0]);
    assert_eq!(loan.more.allowed_refund_origins, vec![0, 1, 2]);
    assert_eq!(loan.more.refund_origin, Some(MortgageRefundOrigin::ThisYear));
    assert_eq!(
        (loan.id, loan.more.details.clone(), loan.more.lender.clone()),
        (identity, loan_facts, lender)
    );
    loan.refund = 0;
    accepted(&save_linked_mortgage_rows(&mut session, vec![loan]));
    let loan = linked_mortgage_rows(&mut session).remove(0);
    assert!(loan.more.allowed_refund_origins.is_empty());
    assert!(loan.more.refund_origin.is_none());
    assert_eq!(
        mortgage_loan(&session.draft.deductions.as_ref().unwrap().mortgage[0]).refund_of_interest,
        MortgageInterestRefund::No,
    );
}

// A Form 1098 entered since Home mortgage interest was saved leaves the
// loan's own facts unanswered (its details, the share you paid, the insurance
// contract date and the points treatment): the page opens by WorkflowRequest
// (BeginMortgages) with the answered loans as saved and the new one carrying
// only what its paper settles. Once answered it reopens by EditRequest.
#[test]
fn a_new_form_1098_starts_unanswered_beside_the_saved_loans() {
    use crate::tests::support::builders::native_income::mortgage_source;
    let mut d = complete(FederalStatus::Single);
    let mut saved =
        mortgage_source(first_lender_mortgage(900_000, 40_000_000, AverageBalance::WithinLimit));
    saved.id = d.next_id;
    let mut new = crate::Mortgage1098Source::printed(saved.form.clone());
    new.id = saved.id + 1;
    // A smaller second loan: both together stay within the acquisition limit.
    new.form.box_1 = 100_000;
    new.form.box_2 = 5_000_000;
    new.assigned = crate::DocumentRecipient::Taxpayer;
    new.owner_source = Some(crate::OwnerSource::Added);
    assert!(saved.answered && !new.answered);
    let deductions = d.deductions.as_mut().unwrap();
    deductions.mortgage = vec![saved.clone(), new.clone()];
    d.next_id = d.next_id.max(new.id + 1);
    let mut s = Interview::new(TEST_USER, d);
    let request = navigate(&s.draft, TaxSection::Mortgage);
    let Reply::Response(P::BeginMortgages { value, .. }) = send(&mut s, request) else {
        panic!("a new loan's facts are asked by WorkflowRequest")
    };
    assert_eq!(value.loans.iter().map(|loan| loan.id).collect::<Vec<_>>(), vec![saved.id]);
    assert_eq!(value.loans[0].details, saved.details.clone().into());
    assert_eq!(value.new_loans.iter().map(|loan| loan.id).collect::<Vec<_>>(), vec![new.id]);
    // The paper settles the lender and the box 3 date; the rest is asked.
    assert_eq!(value.new_loans[0].lender, crate::MortgageLender::Institution);
    assert_eq!(value.new_loans[0].debt_date, crate::MortgageDebtDate::Origination);
    let page = crate::tests::cross::interview::linked_mortgage_editor(&mut s);
    accepted(&send(&mut s, page));
    let loans = &s.draft.deductions.as_ref().unwrap().mortgage;
    assert!(loans.iter().all(|loan| loan.answered));
    assert_eq!(loans[0].details, saved.details);
    let request = navigate(&s.draft, TaxSection::Mortgage);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveMortgages { .. })));
}
