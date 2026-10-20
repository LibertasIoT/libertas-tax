// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::first_lender_mortgage;
use crate::tests::cross::interview::{
    PAGE_ADJUSTMENTS, PAGE_DEDUCTIONS, PAGE_INCOME, PAGE_ITEMIZED, PAGE_REVIEW,
    assert_available_sections, at_the_deductions_page, at_the_gifts_page, back, checklist_of,
    continue_hub, deductions_value, empty_facts, gifts, go_on, go_to, goods_gift, hub_rows,
    hub_tail, itemized_hub, navigate, open_itemized_topic, past_credits, refusal, row_of,
    save_deductions, save_gift_carryovers, save_medical, save_mortgages, save_setup,
    send_past_less_common, shown, stored_facts, walk_on_to_review,
};

#[test]
fn educator_and_1098e_pages_map_onto_their_claims() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
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
    let forms = vec![
        Form1098E { interest: 30_000, box_2: Form1098EBox2::NotChecked },
        Form1098E {
            interest: 45_000,
            box_2: Form1098EBox2::Checked { additional_interest: 5_000 },
        },
    ];
    let request = save_student_loan(
        &s.draft,
        StudentLoanInterest {
            forms: forms.clone(),
            other_interest: 12_000,
            employer_paid_interest: 0,
        },
    );
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    let adjustments = s.draft.adjustments.as_ref().unwrap();
    assert_eq!(
        adjustments.student_loan,
        StudentLoanClaim {
            student_loan_interest: 12_000,
            forms: forms.clone(),
            employer_paid_interest: 0,
        }
    );
    // 300.00 + 450.00 + 50.00 box 2 + 120.00 with no form.
    assert_eq!(adjustments.student_loan_interest(), 92_000);
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::StudentLoan);
    let Reply::Edit(P::SaveStudentLoan { value, .. }) = send(&mut s, request) else {
        panic!("expected the student loan page")
    };
    assert_eq!(value.forms, forms);
    assert_eq!(value.other_interest, 12_000);
    // Interest with no form alone is a claim.
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
fn standard_route_is_the_m1_default() {
    // The deductions page follows the adjustments hub and opens complete on
    // the standard deduction with no gifts: M1's stored "nothing applies".
    let (mut s, page) = at_the_deductions_page(FederalStatus::Single);
    let P::SaveDeductions { page: at, previous, progress, .. } = &page else { unreachable!() };
    // M3-P19: Back opens the previous shown section, the adjustments hub
    // (every return walks it, gate fixes batch A); setup, you, people, the
    // checklist, income, adjustments, deductions, credits, other taxes,
    // federal payments and the review.
    assert_eq!((*at, *previous), (PAGE_DEDUCTIONS, PAGE_ADJUSTMENTS));
    assert!(shown(progress).contains("2026 federal · section 7 of 11"));
    let (value, routes) = deductions_value(&page);
    assert_eq!(
        value,
        &Deductions {
            route: DeductionRoute::Standard,
            cash_gifts: 0,
            fund_cash_gifts: 0,
            other_cash_gifts: 0,
            car_loans: vec![],
            casualties: vec![],
            repayment: RepaymentClaim::None,
        }
    );
    assert_eq!(routes, &[0, 1, 2]);
    // Back returns to the income hub.
    let request = back(&s.draft, PAGE_INCOME);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Hub { page: PAGE_INCOME, .. })));
    // Next stores exactly M1's record, records the section and, with no
    // itemizing, reaches the credits section (amended for M3-P11), which
    // every return walks for More credits (gate fixes batch A).
    let before = s.draft.clone();
    assert!(matches!(send(&mut s, page), Reply::Edit(P::SaveMoreCredits { .. })));
    let m1 = complete(FederalStatus::Single);
    assert_eq!(s.draft.deductions, m1.deductions);
    assert_eq!(s.draft.setup, m1.setup);
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Deductions,
        ]
    );
    assert_eq!(s.draft.revision, before.revision + 1);
    assert_eq!(estimate(&s.draft).return_lines, estimate(&m1).return_lines);
    // The itemized hub does not exist for the standard deduction.
    let request = navigate(&s.draft, TaxSection::Itemized);
    assert!(refusal(send(&mut s, request)).contains("This section is not available yet."));
}

#[test]
fn compare_and_itemize_map_onto_the_selection() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    // Compare keeps the automatic comparison and opens the itemized hub.
    let request = save_deductions(&s.draft, DeductionRoute::Compare, 0);
    let hub = itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().choice,
        DeductionSelection::Automatic { expenses: ItemizedChoice::Yes { facts: empty_facts() } }
    );
    let P::Hub { previous, suggested, allowed_sections, progress, .. } = &hub else {
        unreachable!()
    };
    // M3-P19: an itemizer's topics are walked only when planned; with
    // nothing planned and More itemized deductions passed, the hub suggests
    // Continue to the next shown section, credits (gate fixes batch A).
    assert_eq!(*previous, PAGE_DEDUCTIONS);
    assert_eq!(*suggested, TaxSection::Credits);
    // The deductions page, all five topics (M3-P10 opens the last two), and
    // the next section (the credits hub since M3-P11).
    // M3-P15: more itemized deductions before the next section.
    // M3-P18: and the checklist after the deductions page.
    // M3-P19: the next section is the next shown one, credits.
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
            TaxSection::MoreItemized,
            TaxSection::Credits,
        ]
        .map(|t| t as i32),
    );
    // Setup, you, people, the checklist, income, adjustments, deductions,
    // credits, other taxes, federal payments and the review; the walk has
    // nothing left, so the hub is the section summary.
    assert!(
        shown(progress).contains("section 7 of 11 · You're done with itemized deductions"),
        "{}",
        shown(progress)
    );
    // Nothing is entered, visited or suggested: every topic is "Not
    // started" (amended for M3-N1: every destination is a row).
    assert_eq!(
        hub_rows(&hub),
        hub_tail(
            &[
                TaxSection::Medical,
                TaxSection::Taxes,
                TaxSection::Mortgage,
                TaxSection::GiftCarryovers,
                TaxSection::OtherItemized
            ],
            TaxSection::MoreItemized
        )
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
    let hub = itemized_hub(send_past_less_common(&mut s, request));
    let facts = stored_facts(&s);
    assert_eq!(facts.medical, 1_200_000);
    // The premium names its person by identity; the age that sets its limit
    // is read from the people page (46 here), shared with the other coverage
    // records by that identity (F3, 2026-10-09).
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
    assert_eq!(*suggested, TaxSection::Credits);
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
        estimate: vec![],
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
    let hub = itemized_hub(send_past_less_common(&mut s, request));
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
    let request = save_mortgages(
        &s.draft,
        Mortgages {
            loans: vec![first_lender_mortgage(900_000, 40_000_000, AverageBalance::WithinLimit)],
        },
    );
    let hub = itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().mortgage,
        vec![MortgageLoan {
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
    assert_eq!(value.loans[0].interest, 900_000);
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
    let hub = itemized_hub(send_past_less_common(&mut s, request));
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
        estimate: vec![],
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
    let hub = itemized_hub(send_past_less_common(&mut s, request));
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
    // The hub suggests continuing, which records it and walks the later
    // sections to the review (each asks its less-common page, gate fixes
    // batch A).
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Credits);
    let reply = send(&mut s, go_on(&hub));
    walk_on_to_review(&mut s, reply);
    assert!(s.draft.finished_sections.contains(&TaxSection::Itemized));
    // Itemize is the line 19 election with the same facts.
    let compared = stored_facts(&s);
    let request = save_deductions(&s.draft, DeductionRoute::Itemize, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().choice,
        DeductionSelection::Itemized { facts: compared }
    );
    let request = navigate(&s.draft, TaxSection::Deductions);
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the deductions page") };
    assert_eq!(deductions_value(&page).0.route, DeductionRoute::Itemize);
    // Each topic page saved unchanged stores nothing new.
    let before = s.draft.clone();
    for topic in [
        TaxSection::Medical,
        TaxSection::Taxes,
        TaxSection::Mortgage,
        TaxSection::GiftCarryovers,
        TaxSection::OtherItemized,
    ] {
        let page = open_itemized_topic(&mut s, topic);
        itemized_hub(send_past_less_common(&mut s, page));
    }
    assert_eq!(s.draft, before);
}

#[test]
fn spouse_itemizes_is_offered_only_on_a_separate_return() {
    // A single return: three routes, and the fourth is refused on save.
    let (mut s, page) = at_the_deductions_page(FederalStatus::Single);
    assert_eq!(deductions_value(&page).1, &[0, 1, 2]);
    let before = s.draft.clone();
    let request = save_deductions(&s.draft, DeductionRoute::SpouseItemizes, 0);
    error(current_source_refusal(
        "secondary-admission-3160",
        &mut s,
        request,
        "MESSAGE_YOUR_SPOUSE_S_ITEMIZING_DECIDES_YOUR_DEDUCTION_ONLY_ON_A_MARRIED_FILING_SEPA",
        Some("[value/route]"),
        SourceScope::Secondary(SecondaryCorrection::DeductionChoice),
    ));
    assert_eq!(s.draft, before);
    // A separate return offers it; it is Itemize plus the spouse answer.
    let (mut s, page) = at_the_deductions_page(FederalStatus::Separate);
    assert_eq!(deductions_value(&page).1, &[0, 1, 2, 3]);
    let request = save_deductions(&s.draft, DeductionRoute::SpouseItemizes, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    assert!(s.draft.deductions.as_ref().unwrap().spouse_itemizes);
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().choice,
        DeductionSelection::Itemized { facts: empty_facts() }
    );
    let request = go_to(&s.draft, PAGE_ITEMIZED, TaxSection::Deductions);
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the deductions page") };
    assert_eq!(deductions_value(&page).0.route, DeductionRoute::SpouseItemizes);
    // Itemize clears the spouse answer and keeps the election.
    let request = save_deductions(&s.draft, DeductionRoute::Itemize, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    assert!(!s.draft.spouse_itemizes());
    assert!(matches!(
        s.draft.deductions.as_ref().unwrap().choice,
        DeductionSelection::Itemized { .. }
    ));
    // Leaving the separate return keeps "my spouse itemizes" and the
    // itemized facts, and reopens the deductions page (walk-owned data,
    // level 2): its own save refuses the route on a single return, so it
    // opens with the route to change.
    let request = save_deductions(&s.draft, DeductionRoute::SpouseItemizes, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    assert!(!s.draft.spouse_itemizes());
    assert!(s.draft.deductions.as_ref().unwrap().spouse_itemizes);
    assert!(!s.draft.finished_sections.contains(&TaxSection::Deductions));
    let request = navigate(&s.draft, TaxSection::Deductions);
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the deductions page") };
    let (value, routes) = deductions_value(&page);
    assert_eq!(value.route, DeductionRoute::SpouseItemizes);
    assert_eq!(routes, &[0, 1, 2]);
}

#[test]
fn standard_erases_the_itemized_facts() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_deductions(&s.draft, DeductionRoute::Compare, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    let request = P::SaveTaxes {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: PAGE_ITEMIZED,
        previous: PAGE_ITEMIZED,
        review: false,
        estimate: vec![],
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
    // M3-P19: Continue opens the next shown section, credits, walked with
    // the rest to the review (gate fixes batch A).
    let request = continue_hub(&s.draft, PAGE_ITEMIZED);
    let reply = send(&mut s, request);
    walk_on_to_review(&mut s, reply);
    assert!(s.draft.finished_sections.contains(&TaxSection::Itemized));
    assert!(s.draft.finished_sections.contains(&TaxSection::Taxes));
    // Moving between the itemizing routes keeps the facts.
    let facts = stored_facts(&s);
    let request = save_deductions(&s.draft, DeductionRoute::Itemize, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(stored_facts(&s), facts);
    assert_eq!(facts.personal_tax, PersonalTaxElection::Sales { paid: 150_000 });
    // Standard erases them and forgets the itemized hub and its topics.
    let request = save_deductions(&s.draft, DeductionRoute::Standard, 0);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().choice,
        DeductionSelection::Automatic { expenses: ItemizedChoice::No }
    );
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::MoreCredits,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
        ]
    );
    let request = navigate(&s.draft, TaxSection::Taxes);
    error(send(&mut s, request));
    // Comparing again starts from nothing; with nothing planned the hub
    // suggests Continue (M3-P19; M3-P9 suggested the first topic).
    let request = save_deductions(&s.draft, DeductionRoute::Compare, 0);
    let hub = itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(stored_facts(&s), empty_facts());
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Credits);
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
        let request = save_deductions(&s.draft, DeductionRoute::Standard, gifts);
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
        assert_eq!(deductions_value(&page).0.cash_gifts, gifts);
    }
    // No gift means no claim and no line 12f amount.
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_deductions(&s.draft, DeductionRoute::Standard, 0);
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
fn unit15_standard_route_keeps_all_cash_splits_and_checklist_tracks_each() {
    for (fund, other) in [(123_400, 0), (0, 234_500)] {
        let mut s = at_the_gifts_page(DeductionRoute::Itemize, 0);
        let request = save_gift_carryovers(
            &s.draft,
            gifts(vec![goods_gift(GoodsKind::Books, 50_000, 20_000)], vec![], vec![]),
        );
        itemized_hub(send_past_less_common(&mut s, request));
        let mut request = save_deductions(&s.draft, DeductionRoute::Standard, 0);
        let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
        value.fund_cash_gifts = fund;
        value.other_cash_gifts = other;
        accepted(&send(&mut s, request));
        let ded = s.draft.deductions.as_ref().unwrap();
        assert!(ded.property_gifts.is_empty());
        assert_eq!(
            (ded.cash_charity, ded.fund_cash_charity, ded.other_cash_charity),
            (0, fund, other)
        );
        let saved = s.draft.clone();
        let reply = send(&mut s, navigate(&saved, TaxSection::Checklist));
        assert_eq!(
            checklist_of(&reply).deductions & DeductionTopics::GIFTS.0,
            DeductionTopics::GIFTS.0
        );
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
