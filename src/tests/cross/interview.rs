// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
pub(in crate::tests) use super::support::assert::{accepted, rejected_with};
use super::support::builders::native_income::empty_business_credit_history;
use super::support::client::unit50_interview_native::NativeInterviewCase;
use super::*;
use crate::interview::{document_income, estimate_text, filer, return_setup, section_name};

// Section numbers as pages: TaxSection is declared in sequence order, each
// hub followed by its topics (M3-P19).
pub(in crate::tests) const PAGE_SETUP: i32 = TaxSection::Setup as i32;
pub(in crate::tests) const PAGE_PEOPLE: i32 = TaxSection::People as i32;
pub(in crate::tests) const PAGE_CHILDREN: i32 = TaxSection::Children as i32;
pub(in crate::tests) const PAGE_INCOME: i32 = TaxSection::Income as i32;
pub(in crate::tests) const PAGE_ADJUSTMENTS: i32 = TaxSection::Adjustments as i32;
pub(in crate::tests) const PAGE_DEDUCTIONS: i32 = TaxSection::Deductions as i32;
pub(in crate::tests) const PAGE_CREDITS: i32 = TaxSection::Credits as i32;
pub(in crate::tests) const PAGE_ITEMIZED: i32 = TaxSection::Itemized as i32;
pub(in crate::tests) const PAGE_PAYMENTS: i32 = TaxSection::Payments as i32;
pub(in crate::tests) const PAGE_REVIEW: i32 = TaxSection::Review as i32;
pub(in crate::tests) const PAGE_SPOUSE: i32 = TaxSection::Spouse as i32;
pub(in crate::tests) const PAGE_OTHER_TAXES: i32 = TaxSection::OtherTaxes as i32;
pub(in crate::tests) const PAGE_CHECKLIST: i32 = TaxSection::Checklist as i32;
// Shared native producers and page traversal; keep established topic-call paths.
pub(in crate::tests) use super::support::builders::native_income::{
    document, first_lender_mortgage, native_w2 as w2, owned,
};
pub(in crate::tests) use super::support::client::income_pages::{
    all_documents, catalog_estimate_documents, catalog_preview_document, open_documents,
    open_topic, page_documents, save_documents, save_each_topic, save_topic, topic_for,
    topic_kinds,
};

#[test]
fn preview_is_pure_and_excludes_unrelated_income_fields() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let request = catalog_preview_document(IncomeDocument::W2(w2(6_200_000)));
    for _ in 0..2 {
        assert!(matches!(
            send(&mut session, request.clone()),
            Reply::Response(P::IncomePreview { preview: Some(6_200_000), .. })
        ));
    }
    assert_eq!(session.draft, before);
    assert!(!session.dirty);
}

#[test]
fn m1_documents_page_maps_onto_income_records() {
    let original = empty_return(FederalStatus::Joint);
    let mut session = Interview::new(TEST_USER, original.clone());
    let P::SaveIncomes { allowed_documents, allowed_owners, documents, .. } =
        open_documents(&mut session)
    else {
        unreachable!()
    };
    // The Wages page offers the W-2 and Form 8919 pay; the 1099s are the
    // Interest and dividends page's.
    assert_eq!(allowed_documents, topic_kinds(TaxSection::Wages));
    assert_eq!(allowed_documents[0], 0);
    let P::SaveIncomes { allowed_documents, .. } =
        open_topic(&mut session, TaxSection::InterestDividends)
    else {
        unreachable!()
    };
    assert_eq!(allowed_documents, topic_kinds(TaxSection::InterestDividends));
    assert_eq!(allowed_owners, vec![Owner::Taxpayer as i32, Owner::Spouse as i32]);
    assert!(documents.documents.is_empty());
    let int = Int1099 {
        more: IntMore {
            early_penalty: 1_000,
            special: InterestSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_int1099("Synthetic Bank".into(), 30_000)
    };
    let div = native_div1099("".into(), 50_000, 40_000);
    save_each_topic(
        &mut session,
        vec![
            owned(Owner::Spouse, 0, IncomeDocument::W2(w2(5_000_000))),
            owned(Owner::Spouse, 0, IncomeDocument::Int1099(int.clone())),
            document(0, IncomeDocument::Div1099(div.clone())),
        ],
    );
    let first = original.next_id;
    let expected = vec![
        FederalIncomeEntry {
            refund_management: None,
            state_source: Default::default(),
            id: first,
            label: "".into(),
            owner: Owner::Spouse,
            income: FederalIncome::Wage {
                data: FederalWage {
                    abroad: 0,
                    deferred_pension: 0,
                    inmate_pay: 0,
                    wages: 5_000_000,
                    disability: DisabilityIncome::None,
                    withholding: 500_000,
                    social_wages: 5_000_000,
                    social_tips: 0,
                    employee_tips: None,
                    work_deductions: FederalWorkDeductions::NONE,
                    social_tax: 310_000,
                    medicare_wages: 5_000_000,
                    medicare_tax: 72_500,
                    uncollected_tax: None,
                    medicaid_waiver: 0,
                    statutory_business: None,
                    paying_scorporation: None,
                    payroll: crate::W2Payroll::SocialSecurity,
                    coverage: crate::W2Coverage::Ordinary,
                    care_benefits: 0,
                    statutory_employee: false,
                    retirement_plan: false,
                    other_box12: Vec::new(),
                },
            },
            community: CommunityShare::NotNeeded,
        },
        FederalIncomeEntry {
            refund_management: None,
            state_source: Default::default(),
            id: first + 1,
            label: "Synthetic Bank".into(),
            owner: Owner::Spouse,
            income: FederalIncome::Interest { data: native_interest(30_000, 1_000) },
            community: CommunityShare::NotNeeded,
        },
        FederalIncomeEntry {
            refund_management: None,
            state_source: Default::default(),
            id: first + 2,
            label: "".into(),
            owner: Owner::Taxpayer,
            income: FederalIncome::Dividend {
                data: FederalDividend {
                    section199a: ReitDividends::None,
                    ..native_dividend(50_000, 40_000, 0)
                },
            },
            community: CommunityShare::NotNeeded,
        },
    ];
    assert_eq!(session.draft.income, expected);
    assert!(session.draft.finished_sections.contains(&TaxSection::Income));
    // One revision per topic page.
    assert_eq!(session.draft.revision, original.revision + 2);
    // Reopening rebuilds the same pages from the draft, and saving them
    // unchanged changes nothing.
    let mut session = restart(session.draft);
    let accepted = session.draft.clone();
    let rebuilt: Vec<_> = all_documents(&mut session)
        .iter()
        .map(|item| (item.id, item.owner, item.document.clone()))
        .collect();
    assert_eq!(
        rebuilt,
        vec![
            (first, Owner::Spouse, IncomeDocument::W2(w2(5_000_000))),
            (first + 1, Owner::Spouse, IncomeDocument::Int1099(int)),
            (first + 2, Owner::Taxpayer, IncomeDocument::Div1099(div)),
        ]
    );
    for topic in [TaxSection::Wages, TaxSection::InterestDividends] {
        let page = open_topic(&mut session, topic);
        send(&mut session, page);
    }
    assert_eq!(session.draft, accepted);
    assert!(!session.dirty);
    // A single return offers only the taxpayer as a document owner.
    let mut single = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let P::SaveIncomes { allowed_owners, .. } = open_documents(&mut single) else { unreachable!() };
    assert_eq!(allowed_owners, vec![Owner::Taxpayer as i32]);
}

#[test]
fn document_more_nothing_leaves_defaults() {
    let original = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    save_each_topic(
        &mut session,
        vec![
            document(0, IncomeDocument::W2(w2(4_000_000))),
            document(0, IncomeDocument::Int1099(native_int1099("".into(), 20_000))),
            document(0, IncomeDocument::Div1099(native_div1099("".into(), 10_000, 5_000))),
        ],
    );
    let income = &session.draft.income;
    assert_eq!(income.len(), 3);
    let FederalIncome::Wage { data } = &income[0].income else { panic!("W-2 expected") };
    assert_eq!(data.payroll, W2Payroll::SocialSecurity);
    assert_eq!(data.employee_tips, None);
    assert_eq!(data.work_deductions, FederalWorkDeductions::NONE);
    assert_eq!(data.uncollected_tax, None);
    assert_eq!(data.disability, DisabilityIncome::None);
    assert_eq!(data.social_tips, 0);
    assert_eq!(income[0].label, "");
    let FederalIncome::Interest { data } = &income[1].income else { panic!("1099-INT expected") };
    assert_eq!(
        (data.early_penalty, data.exempt, data.private_activity, data.withholding, data.special),
        (0, 0, 0, 0, InterestSpecial::NoneOfThese)
    );
    assert_eq!(income[1].label, "");
    let FederalIncome::Dividend { data } = &income[2].income else { panic!("1099-DIV expected") };
    assert_eq!(data.section199a, ReitDividends::None);
    assert_eq!(
        (
            data.capital_distributions,
            data.collectible_distributions,
            data.unrecaptured_distributions,
            data.withholding,
            data.special
        ),
        (0, 0, 0, 0, crate::DividendSpecial::NoneOfThese)
    );
    assert_eq!(income[2].label, "");
    // A record with any other field set reopens with that box filled.
    let mut d = session.draft.clone();
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        data.social_tips = 12_300;
    }
    let mut session = Interview::new(TEST_USER, d);
    let documents = all_documents(&mut session);
    let IncomeDocument::W2(w) = &documents[0].document else { panic!("W-2 expected") };
    assert!(matches!(w.more, W2More { social_tips: 12_300, .. }));
    assert!(matches!(
        &documents[1].document,
        IncomeDocument::Int1099(Int1099 {
            payer,
            more,
            ..
        }) if payer.is_empty() && *more == IntMore::default()
    ));
}

#[test]
fn preview_document_calculation_fills_the_row() {
    let d = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, d.clone());
    let preview = |session: &mut Interview, document: IncomeDocument| match send(
        session,
        catalog_preview_document(document),
    ) {
        Reply::Response(P::IncomePreview { preview, .. }) => preview,
        other => panic!("expected a preview, got {other:?}"),
    };
    let wage = IncomeDocument::W2(w2(4_000_000));
    let interest = IncomeDocument::Int1099(native_int1099("".into(), 20_000));
    let dividend = IncomeDocument::Div1099(Div1099 {
        more: DivMore {
            capital_distributions: 2_500,
            special: crate::DividendSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_div1099("".into(), 10_000, 5_000)
    });
    assert_eq!(preview(&mut session, wage.clone()), Some(4_000_000));
    assert_eq!(preview(&mut session, interest.clone()), Some(20_000));
    assert_eq!(preview(&mut session, dividend.clone()), Some(12_500));
    // The page built from the draft carries the same amounts on each row.
    save_each_topic(
        &mut session,
        vec![document(0, wage), document(0, interest), document(0, dividend)],
    );
    let previews: Vec<_> = all_documents(&mut session).iter().map(|i| i.preview).collect();
    assert_eq!(previews, vec![Some(4_000_000), Some(20_000), Some(12_500)]);
}

#[test]
fn box12_code_twice_is_rejected() {
    for (form, key) in super::support::builders::native_income::rejected_w2_cases() {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let request = save_documents(&original, vec![document(0, IncomeDocument::W2(form))]);
        rejected_with(send(&mut session, request), key);
        assert_eq!(session.draft, original, "{key}");
        assert!(!session.dirty, "{key}");
    }
}

// Unit M3-P2: box 2b/2a, 2d/2a and 13/12 are each independent relationships,
// not an additive sum; every side is checked.
#[test]
fn div_box_rules_reject_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let reject = |more: DivMore, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let div = Div1099 { more, ..native_div1099("".into(), 100_000, 0) };
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::Div1099(div))]),
        );
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    // Box 2b (unrecaptured section 1250 gain) cannot exceed box 2a.
    reject(
        DivMore {
            capital_distributions: 10_000,
            unrecaptured_distributions: 10_001,
            special: crate::DividendSpecial::NoneOfThese,
            ..Default::default()
        },
        "DIV_2B_ABOVE_2A",
    );
    // Box 2d (collectibles) cannot exceed box 2a either, independently of 2b.
    reject(
        DivMore {
            capital_distributions: 10_000,
            collectible_distributions: 10_001,
            special: crate::DividendSpecial::NoneOfThese,
            ..Default::default()
        },
        "DIV_2D_ABOVE_2A",
    );
    // Box 13 cannot exceed box 12.
    reject(
        DivMore {
            exempt_dividends: 5_000,
            private_activity_dividends: 5_001,
            special: crate::DividendSpecial::NoneOfThese,
            ..Default::default()
        },
        "DIV_13_ABOVE_12",
    );
}

#[test]
fn refund_year_is_the_deduction_year() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let refund = Refund1099G {
        state_details: Default::default(),
        amount: 500_000,
        year: 2019,
        withholding: 0,
        deducted: RefundDeduction::IncomeTax {
            income_paid: 800_000,
            salt_paid: 800_000,
            salt_deducted: 800_000,
            itemized: 1_500_000,
            standard: 1_220_000,
            sales_available: 0,
            property_refunded: 0,
        },
    };
    send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::Refund1099G(refund.clone()))]),
    );
    let FederalIncome::TaxRefund { amount, history, withholding } = &session.draft.income[1].income
    else {
        unreachable!()
    };
    assert_eq!(*amount, 500_000);
    assert_eq!(*withholding, 0);
    assert_eq!(
        *history,
        TaxRefundHistory::IncomeTax {
            deduction_year: 2019,
            income_paid: 800_000,
            sales_available: 0,
            salt_paid: 800_000,
            salt_deducted: 800_000,
            itemized: 1_500_000,
            standard: 1_220_000,
            overall_limit: None,
            property_refunded: 0,
            ordinary: true,
        }
    );
    // Hand computation, Pub 525's refund worksheet for the 2019 deduction
    // year: revised 2019 taxes after removing the refund = max($8,000 -
    // $5,000, $0) = $3,000; lost 2019 deduction = $8,000 - $3,000 = $5,000,
    // capped at 2019's itemizing benefit of $15,000 - $12,200 = $2,800. The
    // $2,800 is taxable on Schedule 1 line 1.
    let r = supported(&session.draft);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 2_800);
    // Reopening restores the deduction year and its facts.
    let mut session = restart(session.draft);
    let documents =
        page_documents(&open_topic(&mut session, TaxSection::GovernmentBenefits)).to_vec();
    assert_eq!(documents[0].document, IncomeDocument::Refund1099G(refund));
    // "Payments made over several years" is accepted and stored (not
    // rejected), then left out with a notice because the page cannot
    // detail the multi-year allocation the deduction needs.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        save_documents(
            &original,
            vec![document(
                0,
                IncomeDocument::Refund1099G(Refund1099G {
                    state_details: Default::default(),
                    amount: 500_000,
                    year: 2025,
                    withholding: 0,
                    deducted: RefundDeduction::SeveralYears,
                }),
            )],
        ),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_TAX_REFUND_LEFT_OUT"), "{r:?}");
    // Reopening a several-years refund reopens as the same choice.
    let mut session = restart(session.draft);
    let documents =
        page_documents(&open_topic(&mut session, TaxSection::GovernmentBenefits)).to_vec();
    let IncomeDocument::Refund1099G(reopened) = &documents[0].document else {
        panic!("expected a refund document, got {:?}", documents[0].document)
    };
    assert_eq!(reopened.deducted, RefundDeduction::SeveralYears);
}

pub(in crate::tests) fn r1099(gross: i64, code: DistributionCode, ira: bool) -> R1099 {
    let methods = DistributionCodes::methods(box7_of(&code), ira, false);
    R1099 {
        state_details: Default::default(),
        gross,
        taxable: gross,
        withholding: 0,
        box7: box7_of(&code),
        allowed_treatments: DistributionCodes::treatments(box7_of(&code), ira, false),
        allowed_methods: methods,
        code,
        ira,
        trump: false,
        more: RMore::default(),
        disaster: DisasterDistribution::No,
        public_safety: vec![],
        after_death: None,
    }
}
pub(in crate::tests) fn normal_kept() -> DistributionCode {
    DistributionCode::Normal { what_happened: Disposition::Kept }
}
pub(in crate::tests) fn early_kept() -> DistributionCode {
    DistributionCode::Early { what_happened: Disposition::Kept, exception: EarlyException::None }
}
// A 1099-R whose box 7a codes lead to Another code.
pub(in crate::tests) fn other_1099(gross: i64, printed: &str, ira: bool) -> R1099 {
    let r = R1099 {
        code: DistributionCode::Other,
        ..r1099(gross, DistributionCode::Nonqualified, ira)
    };
    printed_as(r, printed)
}

#[test]
fn unsupported_code_is_refused_by_name() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    let reply = send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::R1099(r.clone()))]),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(data.ordinary_distribution, RetirementTreatment::Unsupported);
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Unsupported, "{result:?}");
    assert!(
        result.issues.iter().any(|i| i.message.contains(
            "Form 1099-R code 5 from a Roth IRA (prohibited transaction) is not calculated yet"
        )),
        "{result:?}"
    );
    // Reopening rebuilds the identical page: a refusal is stored, not rejected.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_topic(&mut session, TaxSection::Retirement)).to_vec();
    assert_eq!(documents[0].document, IncomeDocument::R1099(r));
}

#[test]
fn validation_audit_f02_w2_social_tax_includes_reported_tips_and_reopens() {
    for (social_wages, social_tips, social_tax) in [
        (10_000, 1_000_000, 62_620),
        (10_000, 1_000_000, 1_010_000),
        (0, 1_000_000, 62_000),
        (200_000, 0, 12_400),
        (0, 0, 0),
    ] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut form = w2(social_wages + social_tips);
        form.social_wages = social_wages;
        form.more.social_tips = social_tips;
        form.social_tax = social_tax;
        // The audited printed W-2: boxes 1/5 $10,100, box 3 $100,
        // box 7 $10,000, box 4 $626.20 and box 6 $146.45.
        if social_wages == 10_000 && social_tips == 1_000_000 {
            form.medicare_tax = 14_645;
        }
        accepted(&send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::W2(form.clone()))]),
        ));
        let page = open_documents(&mut session);
        assert_eq!(page_documents(&page)[0].document, IncomeDocument::W2(form));
        let revision = session.draft.revision;
        accepted(&send(&mut session, page));
        assert_eq!(session.draft.revision, revision);
    }
}

#[test]
fn validation_audit_f02_w2_social_tax_above_combined_source_rejects_unchanged() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut form = w2(1_010_000);
    form.social_wages = 10_000;
    form.more.social_tips = 1_000_000;
    form.social_tax = 1_010_001;
    rejected_with(
        send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::W2(form.clone()))]),
        ),
        "W2_SOCIAL_TAX_ABOVE_WAGES",
    );
    assert_eq!(session.draft, original);
    assert!(!session.dirty);

    form.social_tax = 62_620;
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(form))]),
    ));
    let before = session.draft.clone();
    let dirty = session.dirty;
    let mut reopened = open_documents(&mut session);
    let P::SaveIncomes { documents, .. } = &mut reopened else { panic!("expected the wages page") };
    let IncomeDocument::W2(form) = &mut documents.documents[0].document else {
        panic!("expected the W-2")
    };
    form.social_tax = 1_010_001;
    rejected_with(send(&mut session, reopened), "W2_SOCIAL_TAX_ABOVE_WAGES");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    let reopened = open_documents(&mut session);
    let IncomeDocument::W2(form) = &page_documents(&reopened)[0].document else { unreachable!() };
    assert_eq!(form.social_tax, 62_620);
}

#[test]
fn income_array_saves_atomically() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let records = page_documents(&open_documents(&mut session)).to_vec();
    let request = save_documents;
    send(&mut session, request(&original, records.clone()));
    assert_eq!(session.draft, original);
    assert!(!session.dirty);
    let mut invalid = records.clone();
    invalid.push(records[0].clone());
    error(send(&mut session, request(&original, invalid)));
    assert_eq!(session.draft, original);
    let mut changed = records;
    if let IncomeDocument::W2(w) = &mut changed[0].document {
        w.wages = 6_100_000;
    }
    let save = request(&original, changed);
    send(&mut session, save.clone());
    assert_eq!(session.draft.income.len(), 1);
    assert_eq!(session.draft.income[0].id, 1);
    assert_eq!(session.draft.revision, original.revision + 1);
    error(send(&mut session, save));
    let accepted = session.draft.clone();
    send(&mut session, request(&accepted, vec![]));
    assert!(session.draft.income.is_empty());
    // Saving a topic page never finishes or reopens the income hub.
    assert!(session.draft.finished_sections.contains(&TaxSection::Income));
}
#[test]
fn response_cannot_be_submitted_as_request() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert!(s.handle(P::IncomePreview { investment_caps: None, preview: Some(1) }).is_none());
}
#[test]
fn new_workflow_restarts_after_each_accepted_section() {
    let fixture = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, FederalDraft::empty(fixture.cookie.clone()));
    let mut reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    // Amended for M3-P13: one more section, the federal payments page;
    // M3-P15: and the other taxes page; M3-P18: and the opening checklist.
    for _ in 0..24 {
        let p = match reply {
            Reply::Response(P::BeginSetup {
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                progress,
            }) => P::SaveSetup {
                user: TEST_USER,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: vec![],
                progress,
                value: return_setup(fixture.setup.as_ref().unwrap()),
            },
            Reply::Response(P::BeginPeople {
                professional_election,
                allowed_professional_elections,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                progress,
            }) => P::SavePeople {
                professional_election,
                allowed_professional_elections,
                user: TEST_USER,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: vec![],
                progress,
                you: filer(&fixture.people.as_ref().unwrap().taxpayer),
            },
            // Amended for M3-P1: the income hub's suggestion opens the Wages
            // page, which saves the fixture's W-2; once it is saved the
            // suggestion continues, which records the hub finished.
            Reply::Response(hub @ P::Hub { .. }) => match send(&mut s, go_on(&hub)) {
                Reply::Edit(editor @ P::SaveIncomes { .. }) => {
                    assert!(!s.dirty);
                    answer(Reply::Edit(editor), Filing::Single)
                }
                // Amended for M3-P7: continuing from the income hub opens
                // the adjustments hub, and continuing from it (M3-P9) the
                // deductions page.
                // Amended for M3-P13: continuing from the credits hub opens
                // the federal payments page.
                // M3-P15: and the credits hub the other taxes page.
                Reply::Response(P::Review { .. } | P::Hub { .. } | P::BeginSituations { .. })
                | Reply::Edit(
                    P::SaveDeductions { .. }
                    | P::SavePayments { .. }
                    | P::SaveMoreAdjustments { .. }
                    | P::SaveMoreCredits { .. }
                    | P::SaveOtherTaxes { .. },
                ) => {
                    assert!(s.dirty);
                    go_on(&hub)
                }
                other => panic!("unexpected {other:?}"),
            },
            // Amended for M3-P9: the deductions page opens complete; Next
            // saves it and continues to the review. M3-P13: so does the
            // federal payments page.
            Reply::Edit(
                editor @ (P::SaveDeductions { .. }
                | P::SavePayments { .. }
                | P::SaveMoreAdjustments { .. }
                | P::SaveMoreCredits { .. }
                | P::SaveOtherTaxes { .. }),
            ) => editor,
            // M3-P15: the other taxes page opens empty; Next saves it. Gate
            // fixes batch A: so does each less-common page.
            reply @ Reply::Response(P::BeginSituations { .. }) => save_empty_situations(reply),
            // The people page, asked on every return (gate fixes batch A).
            reply @ Reply::Response(P::BeginChildren { .. }) => answer(reply, Filing::Single),
            // M3-P18: the opening checklist plans W-2 wages.
            reply @ Reply::Edit(P::SaveChecklist { .. }) => {
                tick(reply, income_ticks(IncomeTopics::WAGES.0))
            }
            // M3-P19: resume opens the planned Wages page directly.
            Reply::Edit(editor @ P::SaveIncomes { .. }) => {
                answer(Reply::Edit(editor), Filing::Single)
            }
            Reply::Response(P::Review { result, .. }) => {
                assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
                return;
            }
            other => panic!("unexpected {other:?}"),
        };
        let _ = send(&mut s, p);
        assert!(s.dirty);
        s = restart(s.draft);
        reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    }
    panic!("workflow did not complete");
}
#[test]
fn failed_deletion_at_revision_limit_is_not_reported_as_success() {
    let mut d = complete(FederalStatus::Single);
    d.revision = i64::MAX - 1;
    let mut s = Interview::new(TEST_USER, d.clone());
    error(send(&mut s, save_documents(&d, vec![])));
    assert_eq!(s.draft, d);
    assert!(!s.dirty);
}

#[test]
fn unchanged_history_next_preserves_every_saved_page_and_completion() {
    let mut original = complete(FederalStatus::Single);
    original.finished = true;
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        P::SavePeople {
            professional_election: crate::ProfessionalRentalElection::SeparateActivities,
            allowed_professional_elections: vec![0, 1],
            user: TEST_USER,
            cookie: original.cookie.clone(),
            revision: original.revision,
            page: PAGE_PEOPLE,
            previous: PAGE_SETUP,
            review: false,
            estimate: vec![],
            warnings: vec![],
            progress: vec![],
            you: filer(&original.people.as_ref().unwrap().taxpayer),
        },
    );
    // The people page follows your page on every return (gate fixes batch A).
    assert!(matches!(reply, Reply::Response(P::BeginChildren { .. })));
    assert_eq!(session.draft, original);
    assert!(!session.dirty);
    assert_eq!(restart(session.draft).draft, original);
}

#[test]
fn changed_history_replaces_page_atomically_and_keeps_later_pages() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut people = original.people.clone().unwrap();
    people.taxpayer.first_name = "Another".into();
    people.taxpayer.last_name = "Synthetic Filer".into();
    // A changed filer page erases nothing after it; Next follows the page order.
    let reply = send(
        &mut session,
        P::SavePeople {
            professional_election: crate::ProfessionalRentalElection::SeparateActivities,
            allowed_professional_elections: vec![0, 1],
            user: TEST_USER,
            cookie: original.cookie.clone(),
            revision: original.revision,
            page: PAGE_PEOPLE,
            previous: PAGE_SETUP,
            review: false,
            estimate: vec![],
            warnings: vec![],
            progress: vec![],
            you: filer(&people.taxpayer),
        },
    );
    // The people page follows your page on every return (gate fixes batch A).
    assert!(matches!(reply, Reply::Response(P::BeginChildren { .. })));
    let saved = &session.draft;
    assert_eq!(saved.setup, original.setup);
    assert_eq!(saved.people, Some(people));
    assert_eq!(saved.income, original.income);
    assert!(saved.finished_sections.contains(&TaxSection::Income));
    // The sections M2 does not ask keep their "nothing applies" answers.
    assert!(saved.dependents.is_empty());
    assert_eq!(saved.adjustments, original.adjustments);
    assert_eq!(saved.deductions, original.deductions);
    assert_eq!(saved.credits, original.credits);
    assert_eq!(saved.screening, original.screening);
    // Payments are answered on the situations list, not by a page, so they stay.
    assert_eq!(saved.payments, original.payments);
    assert!(!saved.finished);
    assert_eq!(saved.next_id, original.next_id);
    assert_eq!(saved.revision, original.revision + 1);
    assert!(session.dirty);
    let mut restored = restart(session.draft.clone());
    assert!(matches!(
        send(&mut restored, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    error(current_source_refusal(
        "secondary-admission-1284",
        &mut session,
        P::SavePeople {
            professional_election: crate::ProfessionalRentalElection::SeparateActivities,
            allowed_professional_elections: vec![0, 1],
            user: TEST_USER,
            cookie: original.cookie,
            revision: original.revision,
            page: PAGE_PEOPLE,
            previous: PAGE_SETUP,
            review: false,
            estimate: vec![],
            warnings: vec![],
            progress: vec![],
            you: filer(&original.people.unwrap().taxpayer),
        },
        "PROBLEM_ACCEPTED_ANSWERS_CHANGED",
        None,
        SourceScope::Fieldless,
    ));
    assert_eq!(session.draft, restored.draft);
}

#[test]
fn landing_forwards_start_or_resume_without_changing_saved_answers() {
    let mut finished = complete(FederalStatus::Single);
    finished.finished = true;
    for (state, draft) in [
        ("new", FederalDraft::empty("landing".into())),
        ("unfinished", at_the_income_hub().0.draft),
        ("complete", complete(FederalStatus::Single)),
        ("finished", finished),
    ] {
        let mut interview = Interview::new(TEST_USER, draft);
        let before = interview.draft.clone();
        let Reply::Response(P::Actions { allowed_actions }) =
            send(&mut interview, P::Enter { user: TEST_USER })
        else {
            panic!("actions expected for {state}")
        };
        assert_eq!(
            allowed_actions,
            vec![libertas_macros::variant_index!(P::OpenInterview) as i32],
            "{state}"
        );
        match (state, send(&mut interview, P::OpenInterview { user: TEST_USER })) {
            ("new", Reply::Response(P::BeginSetup { .. }))
            | ("unfinished", Reply::Edit(P::SaveIncomes { topic: TaxSection::Wages, .. }))
            | ("complete" | "finished", Reply::Response(P::Review { .. })) => {}
            (_, reply) => panic!("unexpected landing destination for {state}: {reply:?}"),
        }
        assert_eq!(interview.draft, before, "{state}");
        assert!(!interview.dirty, "{state}");
    }
}

#[test]
fn dc_homebuyer_carryforward_uses_2026_form_8859_limit_and_survives_restart() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(600_000);
    let r = supported(&d);
    // $60,000 wage income less the $16,100 standard deduction leaves
    // $43,900 taxable, whose 2026 single Tax Table amount is $5,023.
    assert_eq!(value(&r, "income_tax"), 5023);
    assert_eq!(value(&r, "dc_homebuyer_credit"), 5023);
    assert_eq!(value(&r, "dc_homebuyer_carryforward_2027"), 977);
    assert_eq!(return_line(&r, "20"), 502_300);
    assert_eq!(return_line(&r, "22"), 0);
    assert_eq!(r.tax, Some(0));
    let after_restart = supported(&restart(d.clone()).draft);
    assert_eq!(after_restart.tax, r.tax);
    assert_eq!(value(&after_restart, "dc_homebuyer_carryforward_2027"), 977);

    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(-100);
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
}

#[test]
fn requests_are_routed_only_to_the_verified_users_return() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // The Hub verifies the field against the session; the App still refuses to
    // answer a request carrying another identity from this user's return.
    assert!(session.handle(P::Enter { user: TEST_USER + 1 }).is_none());
    assert!(session.handle(P::ReviewAccepted { user: 0 }).is_none());
    assert!(matches!(
        send(&mut session, P::Enter { user: TEST_USER }),
        Reply::Response(P::Actions { .. })
    ));
    // Every editor the App supplies carries the same verified user, so the
    // client resubmits it unchanged under the Fixed current-user constraint.
    let back = P::Back {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        previous: PAGE_SETUP,
    };
    match send(&mut session, back) {
        Reply::Edit(request) => assert_eq!(request.user(), Some(TEST_USER)),
        other => panic!("expected the saved page, got {other:?}"),
    }
    assert_eq!(P::Actions { allowed_actions: vec![] }.user(), None);
}

pub(in crate::tests) fn save_setup(d: &FederalDraft, value: ReturnSetup) -> P {
    P::SaveSetup {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_SETUP,
        previous: PAGE_SETUP,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        value,
    }
}
pub(in crate::tests) fn save_you(d: &FederalDraft, you: Filer) -> P {
    P::SavePeople {
        professional_election: crate::ProfessionalRentalElection::SeparateActivities,
        allowed_professional_elections: vec![0, 1],
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_PEOPLE,
        previous: PAGE_SETUP,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        you,
    }
}
pub(in crate::tests) fn save_spouse(d: &FederalDraft, spouse: Filer) -> P {
    P::SaveSpouse {
        professional_election: crate::ProfessionalRentalElection::SeparateActivities,
        allowed_professional_elections: vec![0, 1],
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_PEOPLE,
        previous: PAGE_PEOPLE,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        spouse: spouse.into(),
    }
}
// Next on the opening checklist (M3-P18) with these ticks, opened in the
// sequence after the last people page `previous`.
pub(in crate::tests) fn save_checklist(d: &FederalDraft, previous: i32, value: Checklist) -> P {
    P::SaveChecklist {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_CHECKLIST,
        previous,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        joint: d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Joint),
        value,
    }
}
// The checklist with only these income ticks.
pub(in crate::tests) fn income_ticks(income: u32) -> Checklist {
    Checklist { income, ..Checklist::NONE }
}
// Next on the checklist page a reply opened, with these ticks.
pub(in crate::tests) fn tick(reply: Reply, value: Checklist) -> P {
    let Reply::Edit(P::SaveChecklist {
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        progress,
        joint,
        ..
    }) = reply
    else {
        panic!("expected the checklist, got {reply:?}")
    };
    P::SaveChecklist {
        user: TEST_USER,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        warnings: vec![],
        progress,
        joint,
        value,
    }
}
// Continue the displayed section after submitting its topic editors.
pub(in crate::tests) fn continue_hub(d: &FederalDraft, page: i32) -> P {
    let (user, cookie, revision) = (TEST_USER, d.cookie.clone(), d.revision);
    match page {
        PAGE_INCOME => P::ContinueIncome { user, cookie, revision },
        PAGE_ADJUSTMENTS => P::ContinueAdjustments { user, cookie, revision },
        PAGE_ITEMIZED => P::ContinueItemized { user, cookie, revision },
        PAGE_CREDITS => P::ContinueCredits { user, cookie, revision },
        _ => panic!("not a section hub"),
    }
}
// Check local destinations within the global navigation ceiling; the grouped
// chooser's complete membership is checked by navigation_search.
pub(in crate::tests) fn assert_available_sections(allowed: &[i32], required: &[i32]) {
    for section in required {
        assert!(allowed.contains(section), "missing {section}");
    }
    assert!(allowed.contains(&(TaxSection::Deductions as i32)));
    assert!(allowed.contains(&(TaxSection::Education as i32)));
}
// Go to, from the review.
pub(in crate::tests) fn navigate(d: &FederalDraft, section: TaxSection) -> P {
    go_to(d, PAGE_REVIEW, section)
}
// Go to, from the review, sent.
pub(in crate::tests) fn from_review(s: &mut Interview, section: TaxSection) -> Reply {
    let request = navigate(&s.draft, section);
    send(s, request)
}
// Go to, from the page `from` (a hub or the review).
pub(in crate::tests) fn go_to(d: &FederalDraft, from: i32, section: TaxSection) -> P {
    P::Navigate {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        from,
        section: section.into(),
        navigation: NavigationAvailability::default(),
    }
}
pub(in crate::tests) fn refusal(reply: Reply) -> alloc::string::String {
    let Reply::Response(P::Problem { message, .. }) = reply else {
        panic!("expected a refusal, got {reply:?}")
    };
    shown(&message)
}
pub(in crate::tests) fn adult(first: &str, last: &str, birth_date: u32) -> Filer {
    Filer {
        first_name: first.into(),
        last_name: last.into(),
        birth_date,
        blind: false,
        claimable: Claimable::Nobody,
        identification: Identification::SocialSecurityNumber,
        more: MoreAboutYou::Nothing,
        real_estate_professional: None,
    }
}

#[test]
fn m1_setup_and_filer_pages_reach_the_draft() {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m1-joint".into()));
    assert!(matches!(
        send(&mut s, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::BeginSetup { page: PAGE_SETUP, .. })
    ));
    let setup = ReturnSetup { filing: Filing::MarriedJointly };
    let request = save_setup(&s.draft, setup.clone());
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Response(P::BeginPeople { page: PAGE_PEOPLE, previous: PAGE_SETUP, .. })
    ));
    assert_eq!(
        s.draft.setup,
        Some(FederalSetup {
            separate_spouse: SeparateSpouseBoxes::NoneOfThese,
            label: crate::Message::new(
                "LABEL_RETURN",
                alloc::vec![crate::MessageArg::Number(2026)]
            )
            .encode(),
            basis: AmountBasis::Actual,
            status: FilingChoice::Joint,
            marital_state: MaritalState::Married { living: SpouseLiving::Together },
            paid_over_half_home: false,
            community_property: CommunityProperty::No,
        })
    );

    // Your page: the name is stored as Form 1040 prints it, and Nobody stores
    // the conservative answers to every dependency test.
    let you = adult("Alex Q", "Sample", 19850301);
    let request = save_you(&s.draft, you.clone());
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Response(P::BeginSpouse { page: PAGE_PEOPLE, previous: PAGE_PEOPLE, .. })
    ));
    let people = s.draft.people.clone().unwrap();
    assert!(people.spouse.is_none() && people.parent_tax.is_none());
    assert_eq!(
        people.taxpayer,
        FederalPerson {
            birth_date: 19850301,
            blind: false,
            died: None,
            another_eitc_child: false,
            dependency: FilerDependency::Independent,
            valid_ssn: true,
            valid_tin: true,
            us_home_over_half_year: true,
            full_time_student: false,
            disabled: false,
            incapable_self_care: false,
            support_share: SupportShare::MoreThanHalf,
            parent_alive: false,
            first_name: "Alex Q".into(),
            last_name: "Sample".into(),
            real_estate_professional: None,
        }
    );
    // Restarting before the spouse page resumes at it; Back returns to your page.
    let mut restarted = restart(s.draft.clone());
    assert!(matches!(
        send(&mut restarted, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::BeginSpouse { .. })
    ));
    let back = P::Back {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        previous: PAGE_PEOPLE,
    };
    let Reply::Edit(P::SavePeople { you: reopened, .. }) = send(&mut s, back) else {
        panic!("expected your page")
    };
    assert_eq!(reopened, you);

    // The spouse page keeps every dependency test of a Possibly answer.
    let spouse = Filer {
        blind: true,
        claimable: Claimable::Possibly {
            another_eitc_child: false,
            dependency: FilerDependency::CanBeClaimed { claimed: false },
            us_home_over_half_year: true,
            full_time_student: true,
            disabled: false,
            incapable_self_care: false,
            support_share: SupportShare::LessThanHalf,
            parent_alive: true,
        },
        ..adult("Jordan", "Sample", 20030720)
    };
    let request = save_spouse(&s.draft, spouse.clone());
    let reply = send(&mut s, request);
    // The people page follows the spouse page on every return (gate fixes
    // batch A), whose Back returns to it.
    assert!(matches!(
        reply,
        Reply::Response(P::BeginChildren { page: PAGE_CHILDREN, previous: PAGE_SPOUSE, .. })
    ));
    let saved = s.draft.people.clone().unwrap().spouse.as_ref().unwrap().clone();
    assert_eq!(status_text(&saved.name()), "Jordan Sample");
    assert!(saved.blind && saved.full_time_student && saved.parent_alive);
    assert!(saved.dependency.claimable());
    assert_eq!(saved.support_share, SupportShare::LessThanHalf);
    assert_eq!(s.draft.revision, 3);

    // Reopening rebuilds each page from the draft; unchanged Next walks setup,
    // you and spouse without a new revision.
    let accepted = s.draft.clone();
    let back = P::Back {
        user: TEST_USER,
        cookie: accepted.cookie.clone(),
        revision: accepted.revision,
        previous: PAGE_SETUP,
    };
    let Reply::Edit(request @ P::SaveSetup { .. }) = send(&mut s, back) else {
        panic!("expected the setup page")
    };
    assert!(matches!(&request, P::SaveSetup { value, .. } if *value == setup));
    let Reply::Edit(request @ P::SavePeople { .. }) = send(&mut s, request) else {
        panic!("expected your page")
    };
    assert!(matches!(&request, P::SavePeople { you: v, .. } if *v == you));
    let Reply::Edit(request @ P::SaveSpouse { .. }) = send(&mut s, request) else {
        panic!("expected the spouse page")
    };
    assert!(matches!(&request, P::SaveSpouse { spouse: v, .. } if *v == spouse.clone().into()));
    assert!(matches!(send(&mut s, request), Reply::Response(P::BeginChildren { .. })));
    assert_eq!(s.draft, accepted);
    assert_eq!(restart(s.draft.clone()).draft, accepted);
}

#[test]
fn single_filer_never_sees_spouse_page() {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m1-single".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    assert_eq!(s.draft.setup.as_ref().unwrap().marital_state, MaritalState::Unmarried);
    let request = save_you(&s.draft, adult("Casey", "Sample", 19900101));
    let reply = send(&mut s, request);
    // The people page follows your page on every return (gate fixes batch A).
    assert!(matches!(
        reply,
        Reply::Response(P::BeginChildren { page: PAGE_CHILDREN, previous: PAGE_PEOPLE, .. })
    ));
    assert!(s.draft.people.as_ref().unwrap().spouse.is_none());
    let accepted = s.draft.clone();
    // Going back to your page and pressing Next goes straight past the spouse page.
    let back = P::Back {
        user: TEST_USER,
        cookie: accepted.cookie.clone(),
        revision: accepted.revision,
        previous: PAGE_PEOPLE,
    };
    let Reply::Edit(request) = send(&mut s, back) else { panic!("expected your page") };
    assert!(matches!(send(&mut s, request), Reply::Response(P::BeginChildren { .. })));
    // Nor can the spouse page be opened on a single return.
    error(send(&mut s, navigate(&accepted, TaxSection::Spouse)));
    assert!(matches!(
        send(&mut restart(accepted.clone()), P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::BeginChildren { .. })
    ));
    assert_eq!(s.draft, accepted);
}

#[test]
fn identification_supplies_ssn_and_tin_answers() {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m2-identification".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::MarriedJointly });
    send(&mut s, request);
    for (identification, valid_ssn, valid_tin) in [
        (Identification::SocialSecurityNumber, true, true),
        (Identification::Itin, false, true),
        (Identification::NoneByDueDate, false, false),
    ] {
        // Your page with Nobody, the spouse page with Possibly: both read
        // the identification, and both reopen with it.
        let you = Filer { identification, ..adult("Alex", "Sample", 19850301) };
        let spouse = Filer {
            identification,
            claimable: Claimable::Possibly {
                another_eitc_child: false,
                dependency: FilerDependency::CanBeClaimed { claimed: false },
                us_home_over_half_year: true,
                full_time_student: true,
                disabled: false,
                incapable_self_care: false,
                support_share: SupportShare::LessThanHalf,
                parent_alive: true,
            },
            ..adult("Jordan", "Sample", 20030720)
        };
        let request = save_you(&s.draft, you.clone());
        send(&mut s, request);
        let request = save_spouse(&s.draft, spouse.clone());
        send(&mut s, request);
        let people = s.draft.people.clone().unwrap();
        let saved = people.spouse.as_ref().unwrap();
        assert_eq!((people.taxpayer.valid_ssn, people.taxpayer.valid_tin), (valid_ssn, valid_tin));
        assert_eq!((saved.valid_ssn, saved.valid_tin), (valid_ssn, valid_tin));
        assert_eq!(filer(&people.taxpayer), you);
        assert_eq!(filer(saved), spouse);
    }
}

#[test]
fn review_line_25_is_the_documents_withholding() {
    let mut d = complete(FederalStatus::Single);
    // Hand computation: W-2 box 2 $6,500.00 (complete()) + a second W-2's box 2
    // $2,000.00 = $8,500.00 on line 25a; 1099-INT box 4 $100.00 + 1099-DIV
    // box 4 $50.00 = $150.00 on line 25b. No page asks for withholding.
    add(&mut d, wage(0, Owner::Taxpayer, 1_000_000, 200_000));
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: FederalInterest {
                taxable: 30_000,
                withholding: 10_000,
                special: InterestSpecial::NoneOfThese,
                ..Default::default()
            },
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: FederalDividend {
                ordinary: 20_000,
                withholding: 5_000,
                special: crate::DividendSpecial::NoneOfThese,
                ..Default::default()
            },
        }),
    );
    let mut session = Interview::new(TEST_USER, d);
    let Reply::Response(P::Review { result, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("review expected")
    };
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(return_line(&result, "25a"), 850_000);
    assert_eq!(return_line(&result, "25b"), 15_000);
}

#[test]
fn every_rare_situation_refuses_with_its_name() {
    // The table lists every current variant in Avro index order. Its
    // single-byte union tag is the zigzag-encoded index; refusal checks
    // below cover each current variant's name and destination.
    for (index, situation) in RareSituation::ALL.iter().enumerate() {
        assert_eq!(usize::from(situation.to_avro()[0]), 2 * index);
    }
    // Unit B2 removed FarmRental (the Farming topic's Farm rental kind).
    assert_eq!(RareSituation::ALL.last(), Some(&RareSituation::Casualty));
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let mut names = alloc::collections::BTreeSet::new();
    let items = super::every_refusal_item();
    for item in &items {
        let refusal = item.refused();
        let name = refusal.lines;
        assert!(names.insert(name), "{item:?} repeats {name}");
        assert_eq!(
            APP_STRINGS.iter().find(|(key, _)| *key == refusal.name_key).map(|(_, text)| *text),
            Some(name),
            "{item:?} name resource"
        );
        let explanation = if matches!(item, Chosen::Tax(OtherTax::OtherAdditionalTaxes { .. })) {
            alloc::format!(
                "{}: the App does not calculate the required source history ({name}). This additional tax must be resolved before a federal estimate can be completed.",
                refusal.topic
            )
        } else {
            alloc::format!("{name} is not calculated yet.")
        };
        assert_eq!(
            APP_STRINGS.iter().find(|(key, _)| *key == refusal.issue_key).map(|(_, text)| *text),
            Some(explanation.as_str()),
            "{item:?} issue resource"
        );
        assert_situation_refused(&d, item.clone(), &[name]);
        // Calculated forms are never refused as a whole: Form 8615 is the
        // Your parent's return page, Form 8915-F the 1099-R fold and the
        // Retirement topic's Earlier Form 8915-F.
        assert!(!name.contains("Form 8615") && !name.contains("(Form 8915-F)"), "{item:?}");
    }
    // Every item at once names each one.
    let mut all = d.clone();
    for item in &items {
        item.choose(&mut all);
    }
    assert!(all.valid_stored());
    let all_result = estimate(&all);
    assert_eq!(
        all_result
            .issues
            .iter()
            .filter(|i| {
                (i.message.contains("is not calculated yet.")
                    || i.message.contains("must be resolved before a federal estimate"))
                    && (i.display.key.starts_with("ISSUE_SITUATION_")
                        || i.display.key.starts_with("ISSUE_RARE_"))
            })
            .count(),
        items.len()
    );
    assert!(
        all_result
            .issues
            .iter()
            .filter(|i| {
                (i.message.contains("is not calculated yet.")
                    || i.message.contains("must be resolved before a federal estimate"))
                    && (i.display.key.starts_with("ISSUE_SITUATION_")
                        || i.display.key.starts_with("ISSUE_RARE_"))
            })
            .all(|i| {
                let message = &i.display;
                items.iter().any(|item| item.refused().issue_key == message.key)
            })
    );
    assert_eq!(estimate(&d), baseline);
}

// Saves the checklist, opened from the review, with these whole-return ticks.
pub(in crate::tests) fn save_whole_return(s: &mut Interview, whole_return: u32) -> Reply {
    let reply = send(s, go_to(&s.draft, PAGE_REVIEW, TaxSection::Checklist));
    let Reply::Edit(P::SaveChecklist { value, .. }) = &reply else {
        panic!("expected the checklist, got {reply:?}")
    };
    let value = Checklist { whole_return, ..*value };
    send(s, tick(reply, value))
}
// Gate fixes batch A: the whole return is the checklist's seventh list,
// asked once. Two items are refused by name under the checklist, two are
// notices that leave the tax unchanged, and the joint-only items need a
// joint return on both sides (the client rule's server twin).
#[test]
fn whole_return_list_refuses_notices_and_needs_a_joint_return() {
    let save = save_whole_return;
    let single = complete(FederalStatus::Single);
    for (bit, form) in [
        (WholeReturnTopics::DUAL_STATUS.0, "Form 1040-NR"),
        (WholeReturnTopics::COMMUNITY_PROPERTY.0, "Form 8958"),
    ] {
        let mut s = Interview::new(TEST_USER, single.clone());
        save(&mut s, bit);
        assert_eq!(s.draft.checklist.whole_return, bit);
        let r = estimate(&s.draft);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(
            r.issues.iter().any(|i| i.section == "Your 2026 at a glance"
                && i.message.contains(form)
                && i.message.ends_with("is not calculated yet.")),
            "{r:?}"
        );
        // Review names it once, on the checklist.
        let items = crate::interview::remaining_items(&s.draft, &r, TaxSection::Review);
        assert_eq!(items.iter().filter(|i| i.section == TaxSection::Checklist).count(), 1);
        // Unticking it restores the supported estimate.
        save(&mut s, 0);
        assert_eq!(supported(&s.draft).tax, supported(&single).tax);
    }
    // The joint-only items are refused off a joint return.
    for bit in [
        WholeReturnTopics::NONRESIDENT_SPOUSE.0,
        WholeReturnTopics::INJURED_SPOUSE.0,
        WholeReturnTopics::INNOCENT_SPOUSE.0,
    ] {
        let mut s = Interview::new(TEST_USER, single.clone());
        let Reply::Response(P::Problem { message, .. }) = save(&mut s, bit) else {
            panic!("expected a refusal")
        };
        assert!(shown(&message).starts_with("The nonresident spouse choice"));
        assert_eq!(s.draft.checklist.whole_return, 0);
    }
    // On a joint return the nonresident spouse choice and innocent spouse
    // relief are notices on the checklist; the tax is the ordinary joint
    // return's.
    let joint = complete(FederalStatus::Joint);
    let mut s = Interview::new(TEST_USER, joint.clone());
    save(&mut s, WholeReturnTopics::NONRESIDENT_SPOUSE.0 | WholeReturnTopics::INNOCENT_SPOUSE.0);
    let r = supported(&s.draft);
    assert_eq!(r.tax, supported(&joint).tax);
    for key in [
        "NOTE_NONRESIDENT_SPOUSE_TREATED_AS_A_RESIDENT",
        "NOTE_INNOCENT_SPOUSE_RELIEF_IS_A_SEPARATE_REQUEST",
    ] {
        assert!(
            r.review_notes.iter().any(|n| n.display.key == key
                && n.context == Some(crate::IssueContext::Section(TaxSection::Checklist))),
            "{key}"
        );
    }
    // Leaving the joint return removes the joint-only ticks, named.
    let mut d = s.draft.clone();
    d.setup = complete(FederalStatus::Single).setup;
    let pruned = crate::interview::prune(&mut d);
    assert_eq!(d.checklist.whole_return, 0);
    for key in [
        "SAVING_REMOVES_TOPIC_NONRESIDENT_SPOUSE_TREATED_AS_A_RESIDENT",
        "SAVING_REMOVES_TOPIC_INNOCENT_SPOUSE_RELIEF",
    ] {
        assert!(pruned.removed.iter().any(|r| r.key() == key), "{key}");
    }
}

// The text a formatted-text field shows.
pub(in crate::tests) fn shown(message: &[u8]) -> alloc::string::String {
    use libertas::AvroDecode;
    let mut offset = 0;
    if let Ok(tuple) = crate::common::FormattedMessage::avro_decode(message, &mut offset)
        && offset == message.len()
        && crate::APP_STRINGS.iter().any(|(key, _)| *key == tuple.0)
    {
        return super::status_text(message);
    }
    alloc::string::String::from_utf8_lossy(message).into_owned()
}
pub(in crate::tests) fn back(d: &FederalDraft, previous: i32) -> P {
    P::Back { user: TEST_USER, cookie: d.cookie.clone(), revision: d.revision, previous }
}
// The page a reply opens, by name, with its progress text; a problem panics.
pub(in crate::tests) fn opened(reply: &Reply) -> (&'static str, alloc::string::String) {
    let (name, progress) = match reply {
        Reply::Response(P::BeginSetup { progress, .. })
        | Reply::Edit(P::SaveSetup { progress, .. }) => ("setup", progress),
        Reply::Response(P::BeginPeople { progress, .. })
        | Reply::Edit(P::SavePeople { progress, .. }) => ("you", progress),
        Reply::Response(P::BeginSpouse { progress, .. })
        | Reply::Edit(P::SaveSpouse { progress, .. }) => ("spouse", progress),
        Reply::Response(P::BeginChildren { progress, .. }) => ("people", progress),
        Reply::Edit(P::SaveChildren { progress, .. }) => {
            ("people", progress.as_ref().expect("supplied progress"))
        }
        Reply::Edit(P::SaveChecklist { progress, .. }) => ("checklist", progress),
        Reply::Response(P::Hub { progress, page: PAGE_INCOME, .. }) => ("income", progress),
        Reply::Response(P::Hub { progress, page: PAGE_ITEMIZED, .. }) => ("itemized", progress),
        Reply::Response(P::Hub { progress, page: PAGE_CREDITS, .. }) => ("credits", progress),
        Reply::Edit(P::SaveDeductions { progress, .. }) => ("deductions", progress),
        Reply::Edit(P::SavePayments { progress, .. }) => ("payments", progress),
        Reply::Response(P::BeginSituations { progress, topic: TaxSection::MoreIncome, .. }) => {
            ("less common income", progress)
        }
        Reply::Response(P::BeginSituations {
            progress, topic: TaxSection::MoreItemized, ..
        }) => ("more itemized", progress),
        Reply::Edit(P::SaveMoreAdjustments { progress, .. }) => ("more adjustments", progress),
        Reply::Edit(P::SaveMoreCredits { progress, .. }) => {
            ("more credits", progress.as_ref().expect("supplied progress"))
        }
        Reply::Edit(P::SaveOtherTaxes { progress, .. }) => {
            ("other taxes", progress.as_ref().expect("supplied progress"))
        }
        Reply::Response(P::Hub { progress, .. }) => ("adjustments", progress),
        Reply::Edit(P::SaveHsa { progress, .. }) => ("hsa", progress),
        Reply::Edit(P::SaveCare { progress, .. }) => ("care", progress),
        Reply::Edit(P::SaveIncomes { progress, topic: TaxSection::Wages, .. }) => {
            ("wages", progress.as_ref().expect("native supplied progress"))
        }
        Reply::Edit(P::SaveIncomes { progress, .. }) => {
            ("interest", progress.as_ref().expect("native supplied progress"))
        }
        Reply::Response(P::Review { .. }) => return ("review", "".into()),
        other => panic!("unexpected page {other:?}"),
    };
    (name, shown(progress))
}
// Follows the guided suggestion: choose a topic, or Continue after its walk.
pub(in crate::tests) fn go_on(hub: &P) -> P {
    let P::Hub { topics, suggested, .. } = hub else { panic!("expected a hub") };
    if !topics.items.iter().any(|row| row.section == *suggested) {
        return hub_action(hub);
    }
    choose_suggestion(hub)
}
// Chooses the hub's suggestion in Go to, without completing a section.
fn choose_suggestion(hub: &P) -> P {
    let P::Hub { cookie, revision, page, suggested, navigation, .. } = hub else {
        panic!("expected a hub, got {hub:?}")
    };
    P::Navigate {
        user: TEST_USER,
        cookie: cookie.clone(),
        revision: *revision,
        from: *page,
        section: (*suggested).into(),
        navigation: navigation.clone(),
    }
}
// Presses Next on an empty situations page (the other taxes page of the
// sequence, M3-P15): saves it with nothing chosen.
pub(in crate::tests) fn save_empty_situations(reply: Reply) -> P {
    let Reply::Response(P::BeginSituations {
        topic,
        allowed_situations,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        progress,
    }) = reply
    else {
        panic!("expected an empty situations page, got {reply:?}")
    };
    P::SaveSituations {
        user: TEST_USER,
        topic,
        allowed_situations,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        warnings: vec![],
        progress,
        value: Situations { situations: vec![] },
    }
}
// Walks on from a reply to the review: each less-common page passed with
// "None of these", each hub continued.
pub(in crate::tests) fn walk_on_to_review(s: &mut Interview, mut reply: Reply) {
    loop {
        reply = match past_less_common(s, reply) {
            Reply::Response(hub @ P::Hub { .. }) => send(s, hub_action(&hub)),
            Reply::Response(P::Review { .. }) => return,
            Reply::Edit(mut page @ P::SaveCare { .. }) => {
                fill_care_provider(&mut page);
                send(s, page)
            }
            Reply::Edit(page @ (P::SaveDeductions { .. } | P::SavePayments { .. })) => {
                send(s, page)
            }
            other => panic!("unexpected {other:?}"),
        };
    }
}
// Passes the people page a reply opens with nobody on it (gate fixes batch
// A: every return asks it once), returning the reply that follows.
pub(in crate::tests) fn past_people(s: &mut Interview, reply: Reply) -> Reply {
    match reply {
        Reply::Response(P::BeginChildren { .. }) => {
            let request = save_children(&s.draft, vec![]);
            send(s, request)
        }
        other => other,
    }
}
// Passes each less-common page a reply opens with its "None of these"
// unchanged (gate fixes batch A: a step of its section's walk), returning
// the reply that follows.
pub(in crate::tests) fn past_less_common(s: &mut Interview, mut reply: Reply) -> Reply {
    loop {
        let request = match reply {
            Reply::Response(P::BeginSituations { .. }) => save_empty_situations(reply),
            Reply::Edit(
                page @ (P::SaveMoreAdjustments { .. }
                | P::SaveMoreCredits { .. }
                | P::SaveOtherTaxes { .. }),
            ) => page,
            other => return other,
        };
        reply = send(s, request);
    }
}
// Sends a request and passes the less-common pages it opens.
pub(in crate::tests) fn send_past_less_common(s: &mut Interview, request: P) -> Reply {
    let reply = send(s, request);
    past_less_common(s, reply)
}
// Presses Next on the page a reply opened, answering it the way the filer of
// a single or joint return with one W-2 would; on a hub, takes the suggestion.
pub(in crate::tests) fn answer(reply: Reply, filing: Filing) -> P {
    match reply {
        Reply::Response(P::BeginSetup {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
        }) => P::SaveSetup {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            warnings: vec![],
            progress,
            value: ReturnSetup { filing },
        },
        Reply::Response(P::BeginPeople {
            professional_election,
            allowed_professional_elections,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
        }) => P::SavePeople {
            professional_election,
            allowed_professional_elections,
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            warnings: vec![],
            progress,
            you: adult("Casey", "Sample", 19850315),
        },
        Reply::Response(P::BeginSpouse {
            professional_election,
            allowed_professional_elections,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
        }) => P::SaveSpouse {
            professional_election,
            allowed_professional_elections,
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            warnings: vec![],
            progress,
            spouse: adult("Jordan", "Sample", 19860101).into(),
        },
        Reply::Edit(P::SaveIncomes {
            empty_k1_activity_choices,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            topic,
            allowed_documents,
            allowed_owners,
            people,
            farms,
            cooperative_activities,
            cooperative_properties,
            businesses,
            rentals,
            royalty_properties,
            equipment_rentals,
            hobbies,
            ccc_lots,
            farm_codes,
            farm_limitations,
            farm_sources,
            basis_assets,
            farm_method_changes,
            inherited_iras,
            inherited_roth_iras,
            ltc_insured,
            allowed_business_source,
            allowed_farm_source,
            allowed_partner_source,
            allowed_home_spouse,
            allowed_other_parent,
            ..
        }) => P::SaveIncomes {
            initial_rental_recovery_choices: Vec::new(),
            rental_energy_buildings: Vec::new(),
            empty_k1_activity_choices,
            qbi_sources: Vec::new(),
            initial_carbon_oxide_origins: vec![],
            initial_renewable_facility_origins: Vec::new(),
            allowed_continuing_risk: ScorpRiskStart::continuing_choices(),
            allowed_injured_source_facts: vec![],
            allowed_injured_retained_owners: vec![],
            ltc_payee_sources: vec![],

            wage_scorporations: Vec::new(),
            allowed_wage_scorporations: Vec::new(),
            initial_k1_investment_choices: alloc::vec![0, 1],
            research_businesses: Vec::new(),
            retained_research_choices: Vec::new(),
            k1_credit_lines: crate::business_credit_page::k1_credit_lines(),
            research_credit_lines: crate::business_credit_page::research_credit_lines(),
            renewable_credit_lines: crate::business_credit_page::renewable_credit_lines(),
            transferable_credit_lines: crate::business_credit_page::transferable_credit_lines(),
            mortgage_loans: Vec::new(),
            allowed_refund_management: vec![],
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            warnings: vec![],
            progress,
            topic,
            allowed_documents,
            allowed_owners,
            people,
            farms,
            cooperative_activities,
            cooperative_properties,
            businesses,
            rentals,
            royalty_properties,
            equipment_rentals,
            hobbies,
            ccc_lots,
            farm_codes,
            farm_limitations,
            farm_sources,
            basis_assets,
            farm_method_changes,
            inherited_iras,
            inherited_roth_iras,
            ltc_insured,
            annuity_sources: vec![],
            recovery_sources: vec![],
            allowed_business_source,
            allowed_farm_source,
            allowed_partner_source,
            allowed_home_spouse,
            allowed_other_parent,
            children: vec![],
            document_facts: DocumentFacts { year: 2026 },
            documents: IncomeDocuments {
                qbi_aggregations: Vec::new(),
                receipt_context: crate::BusinessReceiptContext::default(),
                documents: vec![document(0, IncomeDocument::W2(w2(6_000_000)))],
            },
            allowed_community: Vec::new(),

            foreign_countries: crate::foreign_accounts::countries(),

            interest_statements: Vec::new(),
            qbi_businesses: vec![],
        },
        // Gate fixes batch A: every return asks the people page once; these
        // filers support nobody.
        Reply::Response(P::BeginChildren {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            allowed_lived,
            days_alive,
        }) => P::SaveChildren {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: Some(estimate),
            warnings: vec![],
            progress: Some(progress),
            dependents: Dependents { people: vec![] },
            allowed_lived,
            days_alive,
        },
        Reply::Edit(mut page @ P::SaveCare { .. }) => {
            fill_care_provider(&mut page);
            page
        }
        Reply::Response(hub @ P::Hub { .. }) => go_on(&hub),
        // The opening checklist (M3-P18): the filer ticks W-2 wages.
        reply @ Reply::Edit(P::SaveChecklist { .. }) => {
            tick(reply, income_ticks(IncomeTopics::WAGES.0))
        }
        // The deductions page opens complete: Next saves it unchanged.
        Reply::Edit(page @ P::SaveDeductions { .. }) => page,
        // So does the federal payments page (M3-P13).
        Reply::Edit(page @ P::SavePayments { .. }) => page,
        // Gate fixes batch A: each less-common page opens with "None of
        // these"; Next saves it unchanged.
        Reply::Edit(
            page @ (P::SaveMoreAdjustments { .. }
            | P::SaveMoreCredits { .. }
            | P::SaveOtherTaxes { .. }),
        ) => page,
        // The other taxes page opens empty (M3-P15).
        reply @ Reply::Response(P::BeginSituations { .. }) => save_empty_situations(reply),
        other => panic!("unexpected page {other:?}"),
    }
}

#[test]
fn m1_sequence_is_four_pages() {
    // The guided walk opens planned Wages, then the income hub summary.
    // It also shows less-common income, adjustments, credits and other
    // taxes pages, plus the summaries in the sequence below. Progress
    // counts the shown sections, including initial filer, people and
    // checklist pages, before the final review.
    let walked = [
        "wages",
        "less common income",
        "income",
        "more adjustments",
        "adjustments",
        "more credits",
        "credits",
        "other taxes",
        "payments",
        "review",
    ];
    for (filing, expected) in [
        (
            Filing::Single,
            ["setup", "you", "people", "checklist"].into_iter().chain(walked).collect::<Vec<_>>(),
        ),
        (
            Filing::MarriedJointly,
            ["setup", "you", "spouse", "people", "checklist"].into_iter().chain(walked).collect(),
        ),
    ] {
        let joint = filing == Filing::MarriedJointly;
        let mut s = Interview::new(TEST_USER, FederalDraft::empty("m1-sequence".into()));
        let mut reply = send(&mut s, P::OpenInterview { user: TEST_USER });
        let mut visited = Vec::new();
        let mut progress = Vec::new();
        loop {
            let (name, text) = opened(&reply);
            visited.push(name);
            if name == "review" {
                break;
            }
            progress.push(text);
            if name == "income" {
                // The hub's previous section is the checklist (M3-P18),
                // whose previous is the last filer page.
                let Reply::Response(P::Hub { previous, .. }) = &reply else { unreachable!() };
                let request = back(&s.draft, *previous);
                let before = send(&mut s, request);
                assert_eq!(opened(&before).0, "checklist");
                let Reply::Edit(P::SaveChecklist { previous, .. }) = &before else {
                    unreachable!()
                };
                let request = back(&s.draft, *previous);
                let before = send(&mut s, request);
                assert_eq!(opened(&before).0, "people");
            }
            assert!(visited.len() < 20, "{visited:?}");
            let request = answer(reply, filing);
            reply = send(&mut s, request);
        }
        assert_eq!(visited, expected);
        // The count is setup, you, people, the checklist, the four sections
        // that always ask and the review.
        let filers = if joint {
            vec![(1, 10), (2, 10), (2, 10), (3, 10)]
        } else {
            vec![(1, 10), (2, 10), (3, 10)]
        };
        let sections: Vec<_> = filers
            .into_iter()
            .chain([
                (0, 0),
                (5, 10),
                (5, 10),
                (5, 10),
                (6, 10),
                (6, 10),
                (7, 10),
                (7, 10),
                (8, 10),
                (9, 10),
            ])
            .collect();
        assert_eq!(progress.len(), sections.len());
        for (shown, (n, total)) in progress.iter().zip(&sections) {
            if *n == 0 {
                // Opened before any tick: the sections that always ask.
                assert!(
                    shown.contains(
                        "Your return: Income, Adjustments to income, Credits, Other taxes, Federal payments, Review"
                    ),
                    "{shown}"
                );
                continue;
            }
            let text = alloc::format!("2026 federal · section {n} of {total}");
            assert!(shown.contains(text.as_str()), "{shown}");
        }
        let at = |name: &str| visited.iter().position(|v| *v == name).unwrap();
        assert!(progress[at("wages")].contains("· Wages"));
        assert!(progress[at("income")].contains("· You're done with income"));
        // Their draft records hold the "nothing applies" answers, so the
        // return is complete and estimated.
        let d = &s.draft;
        assert!(d.dependents.is_empty());
        assert_eq!(
            d.finished_sections,
            vec![
                TaxSection::Children,
                TaxSection::Checklist,
                TaxSection::Wages,
                TaxSection::MoreIncome,
                TaxSection::Income,
                TaxSection::MoreAdjustments,
                TaxSection::Adjustments,
                TaxSection::Deductions,
                TaxSection::MoreCredits,
                TaxSection::Credits,
                TaxSection::OtherTaxes,
                TaxSection::Payments
            ]
        );
        let nothing = complete(FederalStatus::Single);
        assert_eq!(d.adjustments, nothing.adjustments);
        assert_eq!(d.deductions, nothing.deductions);
        assert_eq!(d.screening, nothing.screening);
        assert!(d.credits.as_ref().is_some_and(|c| c.education.is_empty()));
        assert_eq!(estimate(d).state, ResultState::EstimatedForSupportedScope);
        // A section this return has not reached cannot be opened: the
        // income by period page needs box C on the payments page.
        for section in [TaxSection::Itemized, TaxSection::IncomeByPeriod] {
            let request = navigate(&s.draft, section);
            let message = refusal(send(&mut s, request));
            assert!(message.contains("This section is not available yet."), "{message}");
        }
    }
}

#[test]
fn every_page_calculates_the_running_estimate() {
    // The m1-single-w2 fixture's documents; its line 34 is the refund.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/returns/m1-single-w2.json"))
            .unwrap();
    let cents = |v: &serde_json::Value| (v.as_f64().unwrap() * 100.0).round() as i64;
    let box_ = |name: &str| cents(&fixture["w2"][0][name]);
    let documents = IncomeDocuments {
        qbi_aggregations: Vec::new(),
        receipt_context: crate::BusinessReceiptContext::default(),
        documents: vec![
            document(
                0,
                IncomeDocument::W2(W2 {
                    answers: Default::default(),
                    state_details: Default::default(),
                    wages: box_("box1_wages"),
                    withholding: box_("box2_withheld"),
                    social_wages: box_("box3"),
                    social_tax: box_("box4"),
                    medicare_wages: box_("box5"),
                    medicare_tax: box_("box6"),
                    more: W2More::default(),
                }),
            ),
            document(
                0,
                IncomeDocument::Int1099(native_int1099(
                    "".into(),
                    cents(&fixture["int1099"][0]["box1_interest"]),
                )),
            ),
        ],
    };
    let refund = cents(&fixture["expected"]["lines"]["34"]);
    let expected = alloc::format!("${}.{:02}", refund / 100, refund % 100);
    assert_eq!(expected, "$141.00");
    let setup = ReturnSetup { filing: Filing::Single };
    let you = adult("Casey", "Sample", 19850315);
    // Amended for M3-P1: the documents are entered on two topic pages of the
    // income hub, and the progress counts the design's ten sections (nine
    // without dependents) with the topic's name.
    let wages = &documents.documents[..1];
    let interest = &documents.documents[1..];
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m1-single-w2".into()));
    page_walk_initial(&mut s, &setup, &you);
    page_walk_wages(&mut s, wages);
    page_walk_interest(&mut s, interest, &expected);
    page_walk_adjustments(&mut s, &expected);
    page_walk_deductions(&mut s, &expected);
    page_walk_credits_and_other_taxes(&mut s, &expected);
    let payments_value = page_walk_payments(&mut s, &expected);
    page_walk_accepted(&mut s, &setup, &you, wages, interest, payments_value, &expected);
    page_walk_changed_setup(&mut s);
    page_walk_spouse();
}

#[inline(never)]
fn page_walk_running(
    s: &mut Interview,
    request: P,
) -> (alloc::string::String, alloc::string::String) {
    match send(s, request) {
        Reply::Response(
            P::RunningEstimate { estimate, progress, .. }
            | P::PaymentsEstimate { estimate, progress, .. }
            | P::IncomeRunningEstimate { estimate, progress, .. },
        ) => (shown(&estimate), shown(&progress)),
        other => panic!("expected the running estimate, got {other:?}"),
    }
}

#[inline(never)]
fn page_walk_estimate_setup(value: &ReturnSetup) -> P {
    P::EstimateSetup { user: TEST_USER, value: value.clone() }
}

#[inline(never)]
fn page_walk_estimate_you(you: &Filer) -> P {
    P::EstimateYou {
        professional_election: crate::ProfessionalRentalElection::SeparateActivities,
        allowed_professional_elections: vec![0, 1],
        user: TEST_USER,
        you: you.clone(),
    }
}

#[inline(never)]
fn page_walk_estimate_documents(topic: TaxSection, documents: &[IncomeDocumentItem]) -> P {
    catalog_estimate_documents(
        topic,
        (IncomeDocuments {
            qbi_aggregations: Vec::new(),
            receipt_context: crate::BusinessReceiptContext::default(),
            documents: documents.to_vec(),
        })
        .into(),
    )
}

#[inline(never)]
fn page_walk_enqueue(
    requests: &mut Vec<(P, &'static str)>,
    make: impl FnOnce() -> P,
    section: &'static str,
) {
    requests.push((make(), section));
}

#[inline(never)]
fn page_walk_initial(s: &mut Interview, setup: &ReturnSetup, you: &Filer) {
    // Nothing entered yet: no estimate.
    let (text, progress) = page_walk_running(s, page_walk_estimate_setup(setup));
    assert!(text.contains("Estimate not available"), "{text}");
    // M3-P19: progress counts the sections this return shows: setup, you,
    // people, the checklist, the four sections every return walks (gate
    // fixes batch A) and the review.
    assert!(progress.contains("section 1 of 10"), "{progress}");
    let request = save_setup(&s.draft, setup.clone());
    send(s, request);
    let (text, _) = page_walk_running(s, page_walk_estimate_you(you));
    assert!(text.contains("Estimate not available"), "{text}");
    let request = save_you(&s.draft, you.clone());
    send(s, request);
    let request = save_children(&s.draft, vec![]);
    send(s, request);
    // M3-P18: the opening checklist plans the two topics; its calculation
    // replies with the estimate and the planned path as its progress.
    let ticks = income_ticks(IncomeTopics::WAGES.0 | IncomeTopics::INTEREST_DIVIDENDS.0);
    let (_, progress) =
        page_walk_running(s, P::EstimateChecklist { user: TEST_USER, value: ticks });
    assert!(
        progress
            .ends_with("Your return: Income, Adjustments to income, Credits, Other taxes, Federal payments, Review"),
        "{progress}"
    );
    let request = save_checklist(&s.draft, PAGE_CHILDREN, ticks);
    // M3-P19: its Next opens the first planned topic, Wages.
    assert!(matches!(
        send(s, request),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::Wages, .. })
    ));
}

#[inline(never)]
fn page_walk_wages(s: &mut Interview, wages: &[IncomeDocumentItem]) {
    // A topic page estimates its unsaved documents merged into the accepted
    // return, as if the filer continued past the hub.
    s.dirty = false; // persisted by the task
    let before = s.draft.clone();
    let (text, progress) =
        page_walk_running(s, page_walk_estimate_documents(TaxSection::Wages, wages));
    assert!(text.contains("Estimated federal refund"), "{text}");
    assert!(progress.contains("section 5 of 10 · Wages"), "{progress}");
    // A calculation is pure.
    assert_eq!(s.draft, before);
    assert!(!s.dirty);
    // Saving it opens the next planned topic (M3-P19).
    let request = save_topic(&s.draft, TaxSection::Wages, wages.to_vec());
    assert!(matches!(
        send(s, request),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::InterestDividends, .. })
    ));
}

#[inline(never)]
fn page_walk_interest(s: &mut Interview, interest: &[IncomeDocumentItem], expected: &str) {
    let (text, progress) =
        page_walk_running(s, page_walk_estimate_documents(TaxSection::InterestDividends, interest));
    assert!(text.contains(expected), "{text}");
    assert!(progress.contains("section 5 of 10 · Interest and dividends"), "{progress}");
    let request = save_topic(&s.draft, TaxSection::InterestDividends, interest.to_vec());
    let Reply::Response(hub @ P::Hub { .. }) = send_past_less_common(s, request) else {
        panic!("expected the income hub")
    };
    let P::Hub { estimate: text, .. } = &hub else { unreachable!() };
    assert!(shown(text).contains(expected));
    // Amended for M3-P19: the income hub is the section summary; its OK
    // walks each later section's less-common page (gate fixes batch A) and
    // passes every other later section, which has nothing for this return,
    // to the review. Each of those pages, opened from the review, shows and
    // calculates the same estimate, and counts its own section among the
    // nine shown (a section not shown counts itself: ten).
    let reply = send(s, go_on(&hub));
    walk_on_to_review(s, reply);
}

#[inline(never)]
fn page_walk_adjustments(s: &mut Interview, expected: &str) {
    let Reply::Response(hub @ P::Hub { page: PAGE_ADJUSTMENTS, .. }) =
        from_review(s, TaxSection::Adjustments)
    else {
        panic!("expected the adjustments hub")
    };
    let P::Hub { estimate: text, progress, .. } = &hub else { unreachable!() };
    assert!(shown(text).contains(expected));
    assert!(shown(progress).contains("section 6 of 10"), "{}", shown(progress));
    let (text, progress) = page_walk_running(
        s,
        P::EstimateHsa {
            user: TEST_USER,
            hsa_failed_deposits: vec![],
            archer_failed_deposits: vec![],
            hsa_return_documents: vec![],
            archer_return_documents: vec![],
            other_coverage_allowed: false,
            allowed_owners: vec![Owner::Taxpayer as i32],
            allowed_family_split: vec![0],
            ira_sources: vec![],
            archer_income: vec![],
            value: HsaPage {
                employer_sources: vec![],
                linked_deposits: vec![],
                other_coverage: None,
                family_method: HsaAllocationMethod::Monthly,
                accounts: vec![],
                family_split: FamilySplit::NotApplicable,
                funding: vec![],
                archer: ArcherContributionsPage::None,
            },
        },
    );
    assert!(text.contains(expected), "{text}");
    assert!(
        progress.contains("section 6 of 10 · Health and medical savings accounts"),
        "{progress}"
    );
}

#[inline(never)]
fn page_walk_deductions(s: &mut Interview, expected: &str) {
    let Reply::Edit(deductions @ P::SaveDeductions { .. }) = from_review(s, TaxSection::Deductions)
    else {
        panic!("expected the deductions page")
    };
    let P::SaveDeductions { estimate: text, progress, .. } = &deductions else { unreachable!() };
    assert!(shown(text).contains(expected));
    assert!(shown(progress).contains("section 7 of 11"));
    let (text, progress) = page_walk_running(
        s,
        P::EstimateDeductions {
            user: TEST_USER,
            allowed_routes: vec![0, 1, 2],
            allowed_repayment: vec![0],
            home_sites: vec![],
            value: Deductions {
                route: DeductionRoute::Standard,
                cash_gifts: 0,
                fund_cash_gifts: 0,
                other_cash_gifts: 0,
                car_loans: vec![],
                casualties: vec![],
                repayment: RepaymentClaim::None,
            },
        },
    );
    assert!(text.contains(expected), "{text}");
    assert!(progress.contains("section 7 of 11"), "{progress}");
    // Opened from the review, its unchanged Next returns there.
    assert!(matches!(send(s, deductions), Reply::Response(P::Review { .. })));
}

#[inline(never)]
fn page_walk_credits_and_other_taxes(s: &mut Interview, expected: &str) {
    let Reply::Response(credits @ P::Hub { .. }) = from_review(s, TaxSection::Credits) else {
        panic!("credits hub expected")
    };
    let P::Hub { estimate: text, progress, .. } = &credits else { unreachable!() };
    assert!(shown(text).contains(expected));
    assert!(shown(progress).contains("section 7 of 10"));
    let other_taxes = from_review(s, TaxSection::OtherTaxes);
    let Reply::Edit(P::SaveOtherTaxes { estimate: text, progress, .. }) = &other_taxes else {
        panic!("other taxes page expected, got {other_taxes:?}")
    };
    assert!(shown(text.as_ref().expect("supplied estimate")).contains(expected));
    assert!(shown(progress.as_ref().expect("supplied progress")).contains("section 8 of 10"));
}

#[inline(never)]
fn page_walk_payments(s: &mut Interview, expected: &str) -> Payments {
    let Reply::Edit(payments @ P::SavePayments { .. }) = from_review(s, TaxSection::Payments)
    else {
        panic!("payments page expected")
    };
    let P::SavePayments { estimate: text, progress, value: payments_value, .. } = &payments else {
        unreachable!()
    };
    assert!(shown(text).contains(expected));
    assert!(shown(progress).contains("section 9 of 10"));
    let payments_value = payments_value.clone();
    assert!(matches!(send(s, payments), Reply::Response(P::Review { .. })));
    payments_value
}

#[inline(never)]
fn page_walk_accepted(
    s: &mut Interview,
    setup: &ReturnSetup,
    you: &Filer,
    wages: &[IncomeDocumentItem],
    interest: &[IncomeDocumentItem],
    payments_value: Payments,
    expected: &str,
) {
    // With the return accepted, every page's calculation on the same answers
    // shows the fixture's refund, and so does every page as it is built.
    s.dirty = false; // persisted by the task
    let accepted = s.draft.clone();
    let mut requests = Vec::with_capacity(6);
    page_walk_enqueue(&mut requests, || page_walk_estimate_setup(setup), "section 1 of 10");
    page_walk_enqueue(&mut requests, || page_walk_estimate_you(you), "section 2 of 10");
    page_walk_enqueue(
        &mut requests,
        || page_walk_estimate_documents(TaxSection::Wages, wages),
        "section 5 of 10 · Wages",
    );
    page_walk_enqueue(
        &mut requests,
        || page_walk_estimate_documents(TaxSection::InterestDividends, interest),
        "section 5 of 10 · Interest and dividends",
    );
    page_walk_enqueue(
        &mut requests,
        || P::EstimatePayments {
            allowed_refund_management: vec![],

            user: TEST_USER,
            allowed_prior_returns: vec![0],
            allowed_farmer: vec![0],
            allowed_farmer_next_year: vec![0, 1, 2],
            value: payments_value.into(),
        },
        "section 9 of 10",
    );
    page_walk_enqueue(
        &mut requests,
        || P::EstimateOtherTaxes {
            user: TEST_USER,
            allowed_owners: vec![0],
            mortgage_gift_owners: vec![],
            coverdell_corrections: vec![],
            value: OtherTaxes { taxes: vec![] },
        },
        "section 8 of 10",
    );
    for (request, section) in requests {
        let (text, progress) = page_walk_running(s, request);
        assert!(text.contains(expected), "{text}");
        assert!(progress.contains(section), "{progress}");
    }
    for previous in [PAGE_SETUP, PAGE_PEOPLE, PAGE_INCOME] {
        let (Reply::Edit(P::SaveSetup { estimate, .. })
        | Reply::Edit(P::SavePeople { estimate, .. })
        | Reply::Response(P::Hub { estimate, .. })) = send(s, back(&accepted, previous))
        else {
            panic!("expected a saved page")
        };
        assert!(shown(&estimate).contains(expected));
    }
    let P::SaveIncomes { estimate: text, .. } = open_documents(s) else { unreachable!() };
    assert!(shown(text.as_ref().expect("native supplied estimate")).contains(expected));
    assert_eq!(s.draft, accepted);
    assert!(!s.dirty);
}

#[inline(never)]
fn page_walk_changed_setup(s: &mut Interview) {
    // A changed answer is estimated as its save would store it: a joint
    // status whose spouse page is not answered yet has no estimate.
    let (text, _) = page_walk_running(
        s,
        page_walk_estimate_setup(&ReturnSetup { filing: Filing::MarriedJointly }),
    );
    assert!(text.contains("Estimate not available"), "{text}");
    // A refused answer gets the reply its save would send.
    error(send(
        s,
        page_walk_estimate_setup(&ReturnSetup {
            filing: Filing::HeadOfHousehold {
                paid_over_half_home: false,
                marital: HeadMarital::Unmarried,
            },
        }),
    ));
}

#[inline(never)]
fn page_walk_spouse() {
    // The spouse page of a joint return calculates in the same way.
    let joint = complete(FederalStatus::Joint);
    let mut s = Interview::new(TEST_USER, joint.clone());
    let spouse = filer(joint.people.as_ref().unwrap().spouse.as_ref().unwrap());
    let (text, progress) = page_walk_running(
        &mut s,
        P::EstimateSpouse {
            professional_election: crate::ProfessionalRentalElection::SeparateActivities,
            allowed_professional_elections: vec![0, 1],
            user: TEST_USER,
            spouse: spouse.into(),
        },
    );
    assert!(text.contains("Estimated federal balance due"), "{text}");
    assert_eq!(text, shown(&estimate_text(&estimate(&joint))));
    assert!(progress.contains("section 2 of 10"), "{progress}");
    assert_eq!(s.draft, joint);
}

#[test]
fn review_lists_answers_with_actions() {
    let review = |s: &mut Interview| match send(s, P::ReviewAccepted { user: TEST_USER }) {
        Reply::Response(P::Review { estimate, answers, result, allowed_sections, .. }) => {
            (shown(&estimate), answers, result, allowed_sections)
        }
        other => panic!("expected the review, got {other:?}"),
    };
    let d = complete(FederalStatus::Joint);
    let mut s = Interview::new(TEST_USER, d.clone());
    let (text, answers, result, allowed_sections) = review(&mut s);
    // The estimate and the typed on-screen Form 1040.
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(text, shown(&estimate_text(&estimate(&d))));
    assert!(text.contains("Estimated federal balance due"), "{text}");
    assert!(!result.return_lines.is_empty());
    // The calculated draft return lists every form's lines, Form 1040's
    // first (M3 acceptance fix, 2026-09-26).
    let computed = estimate(&d);
    assert_eq!(result.return_lines, crate::draft_return(&computed));
    assert_eq!(result.return_lines[..computed.return_lines.len()], computed.return_lines[..]);
    // One row per page, each opened by OpenSection with its section, and
    // the "Nothing else applies" row opening the situations list.
    let displayed_rows: Vec<_> =
        answers.iter().map(|r| (status_text(&r.item), r.section)).collect();
    let rows: Vec<_> =
        displayed_rows.iter().map(|(label, section)| (label.as_str(), *section)).collect();
    assert_eq!(
        rows,
        vec![
            ("Your 2026 return", TaxSection::Setup),
            ("About you", TaxSection::People),
            ("About your spouse", TaxSection::Spouse),
            // Gate fixes batch A: the people page is asked on every return.
            ("Children and people you support", TaxSection::Children),
            // Amended for M3-P18.
            ("Your 2026 at a glance", TaxSection::Checklist),
            ("Income", TaxSection::Income),
            ("Adjustments to income", TaxSection::Adjustments),
            ("Deductions", TaxSection::Deductions),
            // Amended for M3-P11.
            ("Credits", TaxSection::Credits),
            // Amended for M3-P15.
            ("Other taxes", TaxSection::OtherTaxes),
            // Amended for M3-P13.
            ("Federal payments", TaxSection::Payments),
        ]
    );
    assert_eq!(status_text(&answers[3].status), "Nothing entered");
    assert_eq!(status_text(&answers[9].status), "Nothing entered");
    assert_eq!(status_text(&answers[10].status), "Nothing entered");
    assert_eq!(status_text(&answers[0].status), "Married filing jointly");
    assert_eq!(status_text(&answers[1].status), "Synthetic Filer");
    // M3-P18: the W-2 is shown ticked from its entry.
    assert_eq!(status_text(&answers[4].status), "1 topic");
    // Amended for M3-P1: the income row summarizes its hub's topics.
    assert_eq!(status_text(&answers[5].status), "1 W-2");
    for row in &answers {
        assert!(allowed_sections.contains(&(row.section as i32)));
    }
    let open = |s: &mut Interview, section: TaxSection| {
        let d = s.draft.clone();
        send(
            s,
            P::Navigate {
                user: TEST_USER,
                cookie: d.cookie.clone(),
                revision: d.revision,
                from: PAGE_REVIEW,
                section: section.into(),
                navigation: NavigationAvailability::default(),
            },
        )
    };
    // Reopening a page from the review and pressing Next returns to the review.
    for section in [TaxSection::Setup, TaxSection::People, TaxSection::Spouse, TaxSection::Wages] {
        let Reply::Edit(editor) = open(&mut s, section) else { panic!("expected the saved page") };
        assert!(matches!(
            editor,
            P::SaveSetup { review: true, .. }
                | P::SavePeople { review: true, .. }
                | P::SaveSpouse { review: true, .. }
                | P::SaveIncomes { review: true, .. }
        ));
        assert!(matches!(send(&mut s, editor), Reply::Response(P::Review { .. })));
    }
    // The income row opens its hub.
    assert!(matches!(
        open(&mut s, TaxSection::Income),
        Reply::Response(P::Hub { review: true, .. })
    ));
    assert_eq!(s.draft, d);
    // Finish follows the review, unchanged.
    let finish = P::StartFinish { user: TEST_USER, cookie: d.cookie.clone(), revision: d.revision };
    assert!(matches!(send(&mut s, finish), Reply::Response(P::BeginFinish { .. })));
}

// A person on the people page with the defaults of a child who lives with
// you all year.
pub(in crate::tests) fn row(first: &str, last: &str, birth_date: u32) -> Dependent {
    Dependent {
        use_for: PersonUse::TaxDependent,
        id: 0,
        first_name: first.into(),
        last_name: last.into(),
        birth_date,
        death_date: None,
        relationship: Relationship::Child,
        lived: LivedWithYou::AllYear,
        situation: DependentSituation::ChildUnder19,
        identification: Identification::SocialSecurityNumber,
        care: CareCosts::No,
        facts: 0,
        competing: CompetingClaim::NoOne,
    }
}
pub(in crate::tests) fn save_children(d: &FederalDraft, people: Vec<Dependent>) -> P {
    P::SaveChildren {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_CHILDREN,
        previous: PAGE_PEOPLE,
        review: false,
        estimate: Some(vec![]),
        warnings: vec![],
        progress: Some(vec![]),
        dependents: Dependents { people },
        allowed_lived: Vec::new(),
        days_alive: 365,
    }
}
// A complete return, whose people page lists nobody yet.
pub(in crate::tests) fn supporting(status: FederalStatus) -> Interview {
    Interview::new(TEST_USER, complete(status))
}
// The people page as the client receives it after it has been saved.
pub(in crate::tests) fn reopened_people(s: &mut Interview) -> Vec<Dependent> {
    let request = navigate(&s.draft, TaxSection::Children);
    match send(s, request) {
        Reply::Edit(P::SaveChildren { dependents, .. }) => dependents.people,
        other => panic!("expected the people page, got {other:?}"),
    }
}
// The record fields a child who lives with you all year stores.
pub(in crate::tests) fn dependent_record(
    id: i64,
    first: &str,
    last: &str,
    birth_date: u32,
) -> FederalDependent {
    FederalDependent {
        use_for: PersonUse::TaxDependent,
        id,
        first_name: first.into(),
        last_name: last.into(),
        birth_date,
        death_date: None,
        relationship: Relationship::Child,
        residency: DependentResidency::Days { days: 365 },
        us_home_over_half_year: true,
        incapable_self_care: false,
        care: DependentCare::No,
        valid_ssn: true,
        valid_tin: true,
        situation: DependentSituation::ChildUnder19,
        facts: PersonFacts::NONE.0,
        competing: CompetingClaim::NoOne,
    }
}

// The M2 fixture's W-2: box 1 $38,000, box 2 $1,600, boxes 3–6.
fn m2_w2() -> IncomeDocumentItem {
    document(
        0,
        IncomeDocument::W2(W2 {
            answers: Default::default(),
            state_details: Default::default(),
            wages: 3_800_000,
            withholding: 160_000,
            social_wages: 3_800_000,
            social_tax: 235_600,
            medicare_wages: 3_800_000,
            medicare_tax: 55_100,
            more: W2More::default(),
        }),
    )
}
fn head_of_household() -> ReturnSetup {
    ReturnSetup {
        filing: Filing::HeadOfHousehold {
            paid_over_half_home: true,
            marital: HeadMarital::Unmarried,
        },
    }
}
// Two children, care paid for the younger.
pub(in crate::tests) fn m2_children() -> Vec<Dependent> {
    vec![
        row("Ava", "Sample", 20180214),
        Dependent {
            care: CareCosts::Yes { expenses: 480_000, incapable_self_care: false },
            ..row("Ben", "Sample", 20210930)
        },
    ]
}
// The deductions page's save as the sequence opened it (M3-P9); the echoed
// allowed routes are never trusted, so they are sent empty.
pub(in crate::tests) fn save_deductions(d: &FederalDraft, route: DeductionRoute, gifts: i64) -> P {
    P::SaveDeductions {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_DEDUCTIONS,
        previous: PAGE_ADJUSTMENTS,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        allowed_routes: vec![],
        allowed_repayment: vec![],
        home_sites: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value: Deductions {
            route,
            cash_gifts: gifts,
            fund_cash_gifts: 0,
            other_cash_gifts: 0,
            car_loans: vec![],
            casualties: vec![],
            repayment: RepaymentClaim::None,
        },
    }
}
// A head-of-household return entered page by page, saved through the review.
pub(in crate::tests) fn household_return(people: Vec<Dependent>) -> Interview {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m2-household".into()));
    for request in [
        |d: &FederalDraft, _: &[Dependent]| save_setup(d, head_of_household()),
        |d: &FederalDraft, _: &[Dependent]| save_you(d, adult("Casey", "Sample", 19880610)),
        |d: &FederalDraft, people: &[Dependent]| save_children(d, people.to_vec()),
        // M3-P18: the opening checklist plans W-2 wages.
        |d: &FederalDraft, _: &[Dependent]| {
            save_checklist(d, PAGE_CHILDREN, income_ticks(IncomeTopics::WAGES.0))
        },
        |d: &FederalDraft, _: &[Dependent]| save_documents(d, vec![m2_w2()]),
    ] {
        let request = request(&s.draft.clone(), &people);
        let reply = send(&mut s, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    }
    // Continue past the income summary, then each later section: the
    // less-common pages with "None of these" (gate fixes batch A), the rest
    // passed with nothing for this return (M3-P19).
    let request = continue_hub(&s.draft.clone(), PAGE_INCOME);
    let reply = send(&mut s, request);
    walk_on_to_review(&mut s, reply);
    s.dirty = false; // persisted by the task
    s
}

#[test]
fn m2_sequence_is_five_pages_with_dependents() {
    // Amended for M3-P1: the documents page is the income hub and its Wages
    // topic, and the progress counts the design's ten sections. Amended for
    // M3-P7: the adjustments hub is section 5 and costs one Next. Amended
    // for M3-P9: the deductions page is section 6 and costs one Next.
    // Amended for M3-P11: the credits hub is section 7 and costs one Next
    // (its care row lists the people page; nothing is suggested). Amended
    // for M3-P13: the federal payments page is section 8 and costs one Next.
    // Amended for M3-P15: the other taxes page is section 8, before the
    // payments page (9), and costs one Next.
    // Amended for M3-P18: the opening checklist is section 4, after the
    // people page, and costs one Next (nothing ticked: the fixture's W-2 is
    // entered from the hub); every later section counts one more, of 12.
    // Amended for M3-P19, the guided walk: the checklist ticks W-2 wages,
    // its Next opens the Wages page and the save the income summary, whose
    // OK passes every later section (the credits hub too: care is answered
    // on the people page) to the review. Progress counts the shown
    // sections: setup, you, people, the checklist, income and the review.
    // Gate fixes batch A: each section's less-common page is a walk step and
    // the adjustments, credits and other taxes sections are always shown.
    for (filing, expected, sections) in [
        (
            head_of_household().filing,
            vec![
                "setup",
                "you",
                "people",
                "checklist",
                "wages",
                "less common income",
                "income",
                "more adjustments",
                "adjustments",
                "care",
                "more credits",
                "credits",
                "other taxes",
                "payments",
                "review",
            ],
            vec![
                (1, 10),
                (2, 10),
                (3, 10),
                (4, 0),
                (5, 10),
                (5, 10),
                (5, 10),
                (6, 10),
                (6, 10),
                (7, 10),
                (7, 10),
                (7, 10),
                (8, 10),
                (9, 10),
            ],
        ),
        (
            Filing::MarriedJointly,
            vec![
                "setup",
                "you",
                "spouse",
                "people",
                "checklist",
                "wages",
                "less common income",
                "income",
                "more adjustments",
                "adjustments",
                "care",
                "more credits",
                "credits",
                "other taxes",
                "payments",
                "review",
            ],
            vec![
                (1, 10),
                (2, 10),
                (2, 10),
                (3, 10),
                (4, 0),
                (5, 10),
                (5, 10),
                (5, 10),
                (6, 10),
                (6, 10),
                (7, 10),
                (7, 10),
                (7, 10),
                (8, 10),
                (9, 10),
            ],
        ),
    ] {
        let joint = filing == Filing::MarriedJointly;
        let mut s = Interview::new(TEST_USER, FederalDraft::empty("m2-sequence".into()));
        let mut reply = send(&mut s, P::OpenInterview { user: TEST_USER });
        let mut visited = Vec::new();
        let mut progress = Vec::new();
        loop {
            let (name, text) = opened(&reply);
            visited.push(name);
            if name == "review" {
                break;
            }
            progress.push(text);
            assert!(visited.len() < 20, "{visited:?}");
            let request = match reply {
                Reply::Response(P::BeginSetup { .. }) => {
                    save_setup(&s.draft, ReturnSetup { filing })
                }
                Reply::Response(P::BeginChildren {
                    cookie,
                    revision,
                    page,
                    previous,
                    review,
                    estimate,
                    progress,
                    allowed_lived,
                    days_alive,
                }) => {
                    // Back from the people page returns to the last filer page.
                    let request = back(&s.draft, previous);
                    let before = send(&mut s, request);
                    assert_eq!(opened(&before).0, if joint { "spouse" } else { "you" });
                    P::SaveChildren {
                        user: TEST_USER,
                        cookie,
                        revision,
                        page,
                        previous,
                        review,
                        estimate: Some(estimate),
                        warnings: vec![],
                        progress: Some(progress),
                        dependents: Dependents { people: m2_children() },
                        allowed_lived,
                        days_alive,
                    }
                }
                Reply::Response(hub @ P::Hub { previous, page: PAGE_INCOME, .. }) => {
                    // The income hub's previous section is the checklist
                    // (M3-P18).
                    let request = back(&s.draft, previous);
                    let before = send(&mut s, request);
                    assert_eq!(opened(&before).0, "checklist");
                    go_on(&hub)
                }
                other => answer(other, filing),
            };
            reply = send(&mut s, request);
        }
        assert_eq!(visited, expected);
        // The people page is the third section on every return, and the
        // income, adjustments, credits and other taxes sections are always
        // shown (gate fixes batch A): nine sections.
        assert_eq!(progress.len(), sections.len());
        assert!(progress[0].contains("2026 federal · section 1 of 10"));
        for (shown, (n, total)) in progress.iter().zip(&sections).skip(1) {
            // The checklist's progress is its planned path (M3-P18).
            if shown.contains("Your return: ") {
                assert_eq!(*n, 4);
                continue;
            }
            let text = alloc::format!("2026 federal · section {n} of {total}");
            assert!(shown.contains(text.as_str()), "{shown}");
        }
        let d = &s.draft;
        assert_eq!(d.dependents().len(), 2);
        assert_eq!(estimate(d).state, ResultState::EstimatedForSupportedScope);
    }
}

#[test]
fn review_lists_the_people_row() {
    let review = |s: &mut Interview| match send(s, P::ReviewAccepted { user: TEST_USER }) {
        Reply::Response(P::Review { answers, result, allowed_sections, .. }) => {
            (answers, result, allowed_sections)
        }
        other => panic!("expected the review, got {other:?}"),
    };
    let mut s = household_return(m2_children());
    let (answers, result, allowed_sections) = review(&mut s);
    let shown_rows: Vec<_> =
        answers.iter().map(|r| (status_text(&r.item), status_text(&r.status), r.section)).collect();
    let rows: Vec<_> = shown_rows
        .iter()
        .map(|(item, status, section)| (item.as_str(), status.as_str(), *section))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Your 2026 return", "Head of household", TaxSection::Setup),
            ("About you", "Casey Sample", TaxSection::People),
            ("Children and people you support", "2 people", TaxSection::Children),
            // Amended for M3-P18: nothing was ticked; the W-2 ticks wages.
            ("Your 2026 at a glance", "1 topic", TaxSection::Checklist),
            ("Income", "1 W-2", TaxSection::Income),
            ("Adjustments to income", "Nothing entered", TaxSection::Adjustments),
            ("Deductions", "Standard deduction", TaxSection::Deductions),
            // Amended for M3-P11: the credits row counts the care topic.
            ("Credits", "1 topic", TaxSection::Credits),
            // Amended for M3-P15.
            ("Other taxes", "Nothing entered", TaxSection::OtherTaxes),
            ("Federal payments", "Nothing entered", TaxSection::Payments),
        ]
    );
    assert!(allowed_sections.contains(&(TaxSection::Children as i32)));
    // The on-screen return carries the M2 lines: Form 1040 12e, 19, 20, 27a
    // (the Schedule EIC credit) and 28, and the Schedule 8812 and Form 2441
    // typed lines.
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
    let coverage = crate::form_coverage(&reviewed(&s));
    for line in ["12e", "19", "20", "27a", "28"] {
        assert!(coverage[&TaxForm::Form1040].contains(line), "{line}");
    }
    assert!(coverage.contains_key(&TaxForm::Schedule8812), "{coverage:?}");
    assert!(coverage.contains_key(&TaxForm::Form2441), "{coverage:?}");
    let amount = |line: &str| {
        result
            .return_lines
            .iter()
            .find(|l| l.form_line.as_ref().is_some_and(|f| f.line == line))
            .unwrap()
            .amount
    };
    for line in ["12e", "27a", "28"] {
        assert!(amount(line) > 0, "{line}");
    }
    // Reopening the people page from the review and pressing Next returns to it.
    let request = navigate(&s.draft, TaxSection::Children);
    let Reply::Edit(editor) = send(&mut s, request) else {
        panic!("expected the saved people page")
    };
    assert!(matches!(editor, P::SaveChildren { review: true, .. }));
    assert!(matches!(send(&mut s, editor), Reply::Response(P::Review { .. })));
    // One person reads "1 person"; a return without dependents reads
    // "Nothing entered", as every return asks the page (gate fixes batch A).
    let mut s = household_return(vec![row("Ava", "Sample", 20180214)]);
    let (answers, _, _) = review(&mut s);
    assert_eq!(status_text(&answers[2].status), "1 person");
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let (answers, _, allowed_sections) = review(&mut s);
    assert!(
        answers
            .iter()
            .any(|r| r.section == TaxSection::Children
                && status_text(&r.status) == "Nothing entered")
    );
    assert!(allowed_sections.contains(&(TaxSection::Children as i32)));
}

#[test]
fn m2_situations_refuse_by_name() {
    // Prior-year and spouse care are now calculated Care-page facts, not
    // MoreCredit refusal variants. The exhaustive remaining refusal inventory
    // must not retain either old source choice.
    // The implemented care credit is no longer a situation: no refusal names
    // Form 2441 as a whole.
    assert!(
        super::every_refusal_item()
            .iter()
            .all(|item| item.refused().lines != "The child and dependent care credit (Form 2441)")
    );
}

// A page save as sent by an editor the review opened.
pub(in crate::tests) fn reviewing(mut request: P) -> P {
    if let P::SaveSetup { review, .. }
    | P::SavePeople { review, .. }
    | P::SaveChildren { review, .. } = &mut request
    {
        *review = true;
    }
    request
}

#[test]
fn a_changed_page_erases_only_what_depends_on_it() {
    let review = |s: &mut Interview, request: P| {
        let reply = send(s, request);
        assert!(matches!(reply, Reply::Response(P::Review { .. })), "{reply:?}");
    };
    // Joint to single drops the spouse and the spouse's documents only.
    let mut joint = complete(FederalStatus::Joint);
    joint.dependents = vec![dependent_record(joint.next_id, "Ava", "Sample", 20180214)];
    joint.next_id += 1;
    let id = joint.next_id;
    add(&mut joint, wage(id, Owner::Spouse, 2_000_000, 200_000));
    let mut s = Interview::new(TEST_USER, joint.clone());
    review(&mut s, reviewing(save_setup(&joint, ReturnSetup { filing: Filing::Single })));
    assert_eq!(s.draft.people.as_ref().unwrap().spouse, SpouseFiler::No);
    assert_eq!(s.draft.income, joint.income[..1]);
    assert_eq!(s.draft.dependents, joint.dependents);
    // Amended for M3-P11 and F3: the credits are kept, and the student catalog is
    // read from the people on the return (the spouse left it); nothing about it
    // is stored.
    assert_eq!(s.draft.credits, joint.credits);
    assert_eq!(
        crate::federal_rules::people_students(&s.draft).iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![-1, joint.dependents[0].id]
    );
    assert_eq!(s.draft.payments, joint.payments);
    // A filer birth-date change keeps everything else.
    let d = s.draft.clone();
    let mut you = filer(&d.people.as_ref().unwrap().taxpayer);
    you.birth_date = 19700101;
    review(&mut s, reviewing(save_you(&d, you)));
    let mut expected = d.clone();
    expected.people.as_mut().unwrap().taxpayer.birth_date = 19700101;
    expected.revision += 1;
    assert_eq!(s.draft, expected);
    // A people change keeps the documents.
    let d = s.draft.clone();
    let mut shown = reopened_people(&mut s);
    shown.push(row("Ben", "Sample", 20210930));
    review(&mut s, reviewing(save_children(&d, shown)));
    assert_eq!(s.draft.dependents().len(), 2);
    assert_eq!(s.draft.income, d.income);
    assert!(s.draft.finished_sections.contains(&TaxSection::Income));
}

// M3-P1: the income hub, its topic pages and navigation.

// A single return whose setup and filer pages are saved; the next page is
// the income hub.
fn at_the_income_hub() -> (Interview, P) {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m3-hub".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let request = save_you(&s.draft, adult("Casey", "Sample", 19850315));
    let reply = send(&mut s, request);
    let checklist = past_people(&mut s, reply);
    // M3-P18: the checklist plans W-2 wages.
    let request = tick(checklist, income_ticks(IncomeTopics::WAGES.0));
    // M3-P19: the walk opens the planned Wages page; its Back opens the hub.
    let Reply::Edit(P::SaveIncomes { topic: TaxSection::Wages, previous, .. }) =
        send(&mut s, request)
    else {
        panic!("expected the Wages page")
    };
    let request = back(&s.draft, previous);
    let Reply::Response(hub @ P::Hub { .. }) = send(&mut s, request) else {
        panic!("expected the income hub")
    };
    (s, hub)
}
// The hub's topics; navigation is outside this list.
pub(in crate::tests) fn hub_rows(
    hub: &P,
) -> Vec<(alloc::string::String, alloc::string::String, TaxSection)> {
    let P::Hub { topics, .. } = hub else { panic!("expected a hub, got {hub:?}") };
    topics.items.iter().map(|r| (status_text(&r.item), status_text(&r.status), r.section)).collect()
}
pub(in crate::tests) fn told_row(hub: &P) -> (alloc::string::String, alloc::string::String) {
    let P::Hub { told, .. } = hub else { panic!("expected a hub, got {hub:?}") };
    assert_eq!(told.section, TaxSection::Checklist, "{told:?}");
    (status_text(&told.item), status_text(&told.status))
}
pub(in crate::tests) fn row_of(
    item: &str,
    status: &str,
    section: TaxSection,
) -> (alloc::string::String, alloc::string::String, TaxSection) {
    (item.into(), status.into(), section)
}
type Row = (alloc::string::String, alloc::string::String, TaxSection);
// The rows every hub ends with since M3-N1: the offered topics not listed
// above as "Not started", the less-common destination ("None" until
// something is chosen).
pub(in crate::tests) fn hub_tail(not_started: &[TaxSection], more: TaxSection) -> Vec<Row> {
    let mut rows: Vec<Row> =
        not_started.iter().map(|t| row_of(section_name(*t), "Not started", *t)).collect();
    rows.push(row_of(section_name(more), "None", more));
    rows
}
// The same, before the walk has asked the less-common page (gate fixes
// batch A): it is not started yet.
pub(in crate::tests) fn hub_tail_unasked(not_started: &[TaxSection], more: TaxSection) -> Vec<Row> {
    let mut rows = hub_tail(not_started, more);
    rows.pop();
    rows.push(row_of(section_name(more), "Not started", more));
    rows
}
pub(in crate::tests) fn with_tail(mut rows: Vec<Row>, tail: Vec<Row>) -> Vec<Row> {
    rows.extend(tail);
    rows
}
const ALL_INCOME_TOPICS: [TaxSection; 14] = [
    TaxSection::Wages,
    TaxSection::InterestDividends,
    TaxSection::InvestmentSales,
    TaxSection::Retirement,
    TaxSection::SelfEmployment,
    TaxSection::Rentals,
    TaxSection::GovernmentBenefits,
    TaxSection::OtherIncome,
    TaxSection::Gambling,
    TaxSection::Farming,
    TaxSection::PassThrough,
    TaxSection::CanceledDebt,
    TaxSection::WorkAbroad,
    TaxSection::HomeOffices,
];
// The income topics other than `listed`, in hub order.
pub(in crate::tests) fn income_topics_but(listed: &[TaxSection]) -> Vec<TaxSection> {
    ALL_INCOME_TOPICS.into_iter().filter(|t| !listed.contains(t)).collect()
}
pub(in crate::tests) fn interest(taxable: i64) -> IncomeDocumentItem {
    document(0, IncomeDocument::Int1099(native_int1099("".into(), taxable)))
}

#[test]
fn income_hub_suggests_wages_then_continue() {
    let (mut s, hub) = at_the_income_hub();
    let P::Hub { page, previous, review, suggested, allowed_sections, progress, .. } = &hub else {
        unreachable!()
    };
    // The hub separates Actions from Go to, and each bound topic link
    // opens its section in one tap.
    // Amended for M3-P18: the previous section is the opening checklist,
    // which planned W-2 wages: the hub suggests Wages because it is planned
    // (a return with nothing ticked suggests Continue), and its first row
    // is what the checklist says.
    assert_eq!((*page, *previous, *review), (PAGE_INCOME, PAGE_CHECKLIST, false));
    // The global navigation ceiling includes the local income destinations
    // and eligible cross-section pages; their identities retain schema order.
    assert_available_sections(
        allowed_sections,
        &[
            TaxSection::Checklist,
            TaxSection::Wages,
            TaxSection::InterestDividends,
            TaxSection::InvestmentSales,
            TaxSection::Retirement,
            TaxSection::SelfEmployment,
            TaxSection::Rentals,
            TaxSection::GovernmentBenefits,
            TaxSection::OtherIncome,
            TaxSection::Gambling,
            TaxSection::Farming,
            TaxSection::PassThrough,
            TaxSection::CanceledDebt,
            TaxSection::WorkAbroad,
            TaxSection::HomeOffices,
            TaxSection::MoreIncome,
            TaxSection::Review,
        ]
        .map(|t| t as i32),
    );
    assert_eq!(*suggested, TaxSection::Wages);
    // Setup, you, people, the checklist, income, adjustments, credits,
    // other taxes and the review.
    assert!(shown(progress).contains("2026 federal · section 5 of 10"));
    assert_eq!(
        told_row(&hub),
        (
            "What you told us".into(),
            "W-2 wages, or employee pay a firm reported on a 1099 (Form 8919)".into()
        )
    );
    // Amended for M3-N1: the planned topic first, then every other topic.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of("Wages", "Planned", TaxSection::Wages)],
            hub_tail_unasked(&income_topics_but(&[TaxSection::Wages]), TaxSection::MoreIncome)
        )
    );
    // OK opens the Wages page; saving a W-2 walks on to Less common income
    // (gate fixes batch A), whose Next ends the walk and returns to the hub,
    // which now suggests continuing to the next section. Each row carries
    // its count and amount (M3-P19): box 1 $60,000.00.
    let Reply::Edit(editor) = send(&mut s, go_on(&hub)) else { panic!("expected the Wages page") };
    let request = answer(Reply::Edit(editor), Filing::Single);
    let Reply::Response(hub) = send_past_less_common(&mut s, request) else {
        panic!("expected the hub")
    };
    let P::Hub { suggested, .. } = &hub else { panic!("expected the hub, got {hub:?}") };
    assert_eq!(*suggested, TaxSection::Adjustments);
    // Amended for M3-N1: every other topic is listed "Not started", and
    // Continue is the last row.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of("Wages", "1 W-2 · $60000.00", TaxSection::Wages)],
            hub_tail(&income_topics_but(&[TaxSection::Wages]), TaxSection::MoreIncome)
        )
    );
    // A topic with documents shows its count and amount.
    let request = save_topic(&s.draft, TaxSection::InterestDividends, vec![interest(20_000)]);
    let Reply::Response(hub) = send(&mut s, request) else { panic!("expected the hub") };
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![
                row_of("Wages", "1 W-2 · $60000.00", TaxSection::Wages),
                row_of(
                    "Interest and dividends",
                    "1 1099-INT · $200.00",
                    TaxSection::InterestDividends
                ),
            ],
            hub_tail(
                &income_topics_but(&[TaxSection::Wages, TaxSection::InterestDividends]),
                TaxSection::MoreIncome
            )
        )
    );
    // Go to also admits the saved setup page across sections.
    let request = go_to(&s.draft, PAGE_INCOME, TaxSection::Setup);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveSetup { .. })));
}

#[test]
fn topic_page_opens_by_edit_request_and_returns_to_its_hub() {
    let (mut s, _) = at_the_income_hub();
    // The first open is a complete editor with an empty list, not a Begin
    // response.
    let editor = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes {
        page,
        previous,
        review,
        topic,
        allowed_documents,
        allowed_owners,
        documents,
        progress,
        ..
    } = &editor
    else {
        unreachable!()
    };
    assert_eq!((*page, *previous, *review), (PAGE_INCOME, PAGE_INCOME, false));
    assert_eq!(*topic, TaxSection::InterestDividends);
    assert_eq!(allowed_documents, &topic_kinds(TaxSection::InterestDividends));
    assert_eq!(allowed_owners, &vec![Owner::Taxpayer as i32]);
    assert!(documents.documents.is_empty());
    assert!(
        shown(progress.as_ref().expect("native supplied progress"))
            .contains("section 5 of 10 · Interest and dividends")
    );
    // Back opens the hub; the hub is a response and has no Back of its own.
    let request = back(&s.draft, *previous);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Hub { .. })));
    // Saving, empty or not, continues the walk (M3-P19): the planned Wages
    // page is still to visit, so it opens; with the walk ended the save
    // returns to the hub (revisit_from_the_hub_returns_to_it).
    let wages_next = |reply: Reply| {
        matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::Wages, .. }))
    };
    assert!(wages_next(send(&mut s, editor.clone())));
    let request = save_topic(&s.draft, TaxSection::InterestDividends, vec![interest(20_000)]);
    assert!(wages_next(send(&mut s, request)));
    // The saved documents reopen the same way.
    let reopened = open_topic(&mut s, TaxSection::InterestDividends);
    assert_eq!(page_documents(&reopened).len(), 1);
    // A topic page takes only its own kinds.
    let request = save_topic(
        &s.draft,
        TaxSection::InterestDividends,
        vec![document(0, IncomeDocument::W2(w2(1_000_000)))],
    );
    let before = s.draft.clone();
    assert!(refusal(send(&mut s, request)).contains("another income topic"));
    assert_eq!(s.draft, before);
}

#[test]
fn topic_is_part_of_the_draft_identity() {
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    // The two topic editors share the return identity, revision and review
    // flag; only the topic tells their device drafts apart.
    let identity = |p: &P| match p {
        P::SaveIncomes { cookie, revision, review, topic, .. } => {
            (cookie.clone(), *revision, *review, *topic)
        }
        other => panic!("expected a topic page, got {other:?}"),
    };
    let wages = identity(&open_topic(&mut s, TaxSection::Wages));
    let interest_page = identity(&open_topic(&mut s, TaxSection::InterestDividends));
    assert_eq!((&wages.0, wages.1, wages.2), (&interest_page.0, interest_page.1, interest_page.2));
    assert_eq!(wages.3, TaxSection::Wages);
    assert_eq!(interest_page.3, TaxSection::InterestDividends);
    // The topic decides what a save or its running estimate may carry: the
    // same 1099-INT is refused under the Wages topic and accepted under its own.
    let documents = vec![interest(20_000)];
    let estimate_as = |topic| {
        catalog_estimate_documents(
            topic,
            (IncomeDocuments {
                qbi_aggregations: Vec::new(),
                receipt_context: crate::BusinessReceiptContext::default(),
                documents: documents.clone(),
            })
            .into(),
        )
    };
    error(send(&mut s, estimate_as(TaxSection::Wages)));
    assert!(matches!(
        send(&mut s, estimate_as(TaxSection::InterestDividends)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    let before = s.draft.clone();
    error(send(&mut s, save_topic(&before, TaxSection::Wages, documents.clone())));
    assert_eq!(s.draft, before);
    // A section that is not an income topic is not a documents page.
    error(send(&mut s, save_topic(&before, TaxSection::Review, documents)));
    assert_eq!(s.draft, before);
}

// The AI Import forms of wave 7 (1099-R, SSA-1099, W-2G, Form 3921, ISO
// 1099-B, 1099-S, 1099-C, 1099-A) validate against the same hidden year.

#[test]
fn page_opened_from_the_review_returns_to_it() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d.clone());
    let Reply::Response(P::Review { page, suggested, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected the review")
    };
    // Go to starts at the first section once every page is answered, and
    // carries the review as its origin.
    assert_eq!((page, suggested), (PAGE_REVIEW, TaxSection::Setup));
    // A section page and a topic page opened from the review return to it,
    // unchanged or changed.
    for section in [TaxSection::People, TaxSection::Wages] {
        let Reply::Edit(editor) = send(&mut s, go_to(&d, page, section)) else {
            panic!("expected the saved page")
        };
        assert!(matches!(
            editor,
            P::SavePeople { review: true, .. } | P::SaveIncomes { review: true, .. }
        ));
        assert!(matches!(send(&mut s, editor), Reply::Response(P::Review { .. })));
    }
    assert_eq!(s.draft, d);
    let Reply::Edit(mut editor) = send(&mut s, go_to(&d, page, TaxSection::Wages)) else {
        panic!("expected the Wages page")
    };
    if let P::SaveIncomes { documents, .. } = &mut editor {
        documents.documents.push(document(0, IncomeDocument::W2(w2(1_000_000))));
    }
    assert!(matches!(send(&mut s, editor), Reply::Response(P::Review { .. })));
    assert_eq!(s.draft.income.len(), 2);
    // The income hub opened from the review keeps the review flag; its topic
    // pages return to the hub, and continuing through the next hub reaches
    // the review.
    let d = s.draft.clone();
    let Reply::Response(hub @ P::Hub { review: true, .. }) =
        send(&mut s, go_to(&d, page, TaxSection::Income))
    else {
        panic!("expected the hub")
    };
    let editor = open_topic(&mut s, TaxSection::Wages);
    assert!(matches!(editor, P::SaveIncomes { review: false, .. }));
    assert!(matches!(send(&mut s, editor), Reply::Response(P::Hub { .. })));
    // Amended for M3-P7 and M3-P9: continuing passed the adjustments hub
    // and the deductions page; M3-P19: they have nothing for this return,
    // so Continue opens the next shown section, the review.
    assert!(matches!(send(&mut s, go_on(&hub)), Reply::Response(P::Review { .. })));
    // A page opened from the hub returns to the hub: the checklist, the
    // income hub's previous section since M3-P18.
    let Reply::Edit(editor) = send(&mut s, go_to(&d, PAGE_INCOME, TaxSection::Checklist)) else {
        panic!("expected the checklist")
    };
    assert!(matches!(editor, P::SaveChecklist { review: false, previous: PAGE_INCOME, .. }));
    assert!(matches!(send(&mut s, editor), Reply::Response(P::Hub { .. })));
    assert_eq!(s.draft, d);
}

#[test]
fn continuing_the_income_hub_records_it_finished() {
    let (mut s, _) = at_the_income_hub();
    let request = save_topic(
        &s.draft,
        TaxSection::Wages,
        vec![document(0, IncomeDocument::W2(w2(6_000_000)))],
    );
    // The walk goes on to Less common income (gate fixes batch A), whose
    // Next opens the summary.
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Response(P::BeginSituations { topic: TaxSection::MoreIncome, .. })
    ));
    let Reply::Response(hub_after) = send(&mut s, save_empty_situations(reply)) else {
        panic!("expected the hub")
    };
    // Saving a topic does not finish the hub: resume returns to it, and the
    // review lists the income section as not answered.
    // (M3-P18: only the opening checklist is recorded; M3-P19: and the
    // planned Wages topic, visited; gate fixes batch A: the people page and
    // Less common income, visited.)
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Wages,
            TaxSection::MoreIncome
        ]
    );
    let mut resumed = restart(s.draft.clone());
    assert!(matches!(
        send(&mut resumed, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::Hub { .. })
    ));
    // Going back or into a topic records nothing.
    s.dirty = false;
    let before = s.draft.clone();
    for section in [TaxSection::Checklist, TaxSection::InterestDividends] {
        send(&mut s, go_to(&before, PAGE_INCOME, section));
    }
    assert_eq!(s.draft, before);
    assert!(!s.dirty);
    // Continuing records the section, as one accepted revision, and opens
    // the next shown section's walk: More adjustments, where resume now
    // starts.
    assert!(matches!(send(&mut s, go_on(&hub_after)), Reply::Edit(P::SaveMoreAdjustments { .. })));
    let passed = vec![
        TaxSection::Children,
        TaxSection::Checklist,
        TaxSection::Wages,
        TaxSection::MoreIncome,
        TaxSection::Income,
    ];
    assert_eq!(s.draft.finished_sections, passed);
    assert_eq!(s.draft.revision, before.revision + 1);
    assert!(s.dirty);
    let mut resumed = restart(s.draft.clone());
    assert!(matches!(
        send(&mut resumed, P::OpenInterview { user: TEST_USER }),
        Reply::Edit(P::SaveMoreAdjustments { .. })
    ));
    // Continuing again changes nothing: the section is recorded once.
    s.dirty = false;
    let finished = s.draft.clone();
    send(&mut s, go_on(&hub_after));
    assert_eq!(s.draft.revision, finished.revision);
    assert_eq!(s.draft.finished_sections, passed);
    assert!(!s.dirty);
}

// The fixture's return entered as a filer does: every hub step takes the
// service's suggestion, except choosing Interest and dividends, a structure
// step. Returns the review's estimate.
// Enters a fixture through the hubs; also returns every request sent (one
// per Next or Go to).
fn enter_through_the_hubs(name: &str, raw: &str) -> (Interview, FederalResult, Vec<P>) {
    let fixture: serde_json::Value = serde_json::from_str(raw).unwrap();
    let cents = |v: &serde_json::Value| (v.as_f64().unwrap() * 100.0).round() as i64;
    let date = |v: &serde_json::Value| v.as_str().unwrap().replace('-', "").parse::<u32>().unwrap();
    let w2s: Vec<_> = fixture["w2"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| {
            document(
                0,
                IncomeDocument::W2(W2 {
                    answers: Default::default(),
                    state_details: Default::default(),
                    wages: cents(&w["box1_wages"]),
                    withholding: cents(&w["box2_withheld"]),
                    social_wages: cents(&w["box3"]),
                    social_tax: cents(&w["box4"]),
                    medicare_wages: cents(&w["box5"]),
                    medicare_tax: cents(&w["box6"]),
                    more: W2More::default(),
                }),
            )
        })
        .collect();
    let ints: Vec<_> = fixture["int1099"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| interest(cents(&v["box1_interest"])))
        .collect();
    let people: Vec<Dependent> = fixture["dependents"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|p| {
                    let person = row(
                        p["first_name"].as_str().unwrap(),
                        p["last_name"].as_str().unwrap(),
                        date(&p["birth_date"]),
                    );
                    match p.get("care_expenses") {
                        Some(amount) => Dependent {
                            care: CareCosts::Yes {
                                expenses: cents(amount),
                                incapable_self_care: false,
                            },
                            ..person
                        },
                        None => person,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let setup = match fixture["filing_status"].as_str().unwrap() {
        "Single" => ReturnSetup { filing: Filing::Single },
        "Head" => head_of_household(),
        other => panic!("fixture status {other}"),
    };
    let mut s = Interview::new(TEST_USER, FederalDraft::empty(name.into()));
    let mut reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    let mut sent = Vec::new();
    for _ in 0..32 {
        let request = match reply {
            Reply::Response(P::BeginSetup { .. }) => save_setup(&s.draft, setup.clone()),
            Reply::Response(P::BeginPeople { .. }) => save_you(
                &s.draft,
                adult("Casey", "Sample", date(&fixture["birth_dates"]["taxpayer"])),
            ),
            Reply::Response(P::BeginChildren { .. }) => save_children(&s.draft, people.clone()),
            // M3-P18: the checklist ticks W-2 wages, and interest when the
            // fixture has a 1099-INT; the income hub then suggests each in
            // turn, then Continue.
            reply @ Reply::Edit(P::SaveChecklist { .. }) => {
                let interest = if ints.is_empty() { 0 } else { IncomeTopics::INTEREST_DIVIDENDS.0 };
                tick(reply, income_ticks(IncomeTopics::WAGES.0 | interest))
            }
            // The summary's task-selected Continue action is tapped.
            Reply::Response(hub @ P::Hub { .. }) => hub_action(&hub),
            Reply::Edit(P::SaveIncomes { topic, .. }) => {
                let documents = if topic == TaxSection::Wages { w2s.clone() } else { ints.clone() };
                save_topic(&s.draft, topic, documents)
            }
            Reply::Edit(mut page @ P::SaveCare { .. }) => {
                fill_care_provider(&mut page);
                page
            }
            // The deductions page opens complete: Next saves it unchanged.
            Reply::Edit(page @ P::SaveDeductions { .. }) => page,
            // So does the federal payments page (M3-P13).
            Reply::Edit(page @ P::SavePayments { .. }) => page,
            // Each section's less-common page opens with "None of these"
            // (gate fixes batch A): Next saves it unchanged.
            Reply::Edit(
                page @ (P::SaveMoreAdjustments { .. }
                | P::SaveMoreCredits { .. }
                | P::SaveOtherTaxes { .. }),
            ) => page,
            reply @ Reply::Response(P::BeginSituations { .. }) => save_empty_situations(reply),
            Reply::Response(P::Review { .. }) => {
                let result = reviewed(&s);
                return (s, result, sent);
            }
            other => panic!("unexpected {other:?}"),
        };
        sent.push(request.clone());
        reply = send(&mut s, request);
    }
    panic!("the interview did not reach the review")
}
fn assert_fixture_lines(raw: &str, result: &FederalResult) {
    let fixture: serde_json::Value = serde_json::from_str(raw).unwrap();
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
    for (line, amount) in fixture["expected"]["lines"].as_object().unwrap() {
        let expected = (amount.as_f64().unwrap() * 100.0).round() as i64;
        assert_eq!(return_line(result, line), expected, "line {line}");
    }
}

#[test]
fn m1_fixture_completes_through_the_hubs() {
    let raw = include_str!("../../../tests/fixtures/returns/m1-single-w2.json");
    let (s, result, _) = enter_through_the_hubs("m1-single-w2", raw);
    assert_fixture_lines(raw, &result);
    // M3-P19: the two planned topics are recorded as visited; each section
    // walks its less-common page with "None of these" (gate fixes batch A).
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Wages,
            TaxSection::InterestDividends,
            TaxSection::MoreIncome,
            TaxSection::Income,
            TaxSection::MoreAdjustments,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::MoreCredits,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments
        ]
    );
    assert_eq!(s.draft.income.len(), 2);
}

#[test]
fn m2_fixture_completes_through_the_hubs() {
    let raw = include_str!("../../../tests/fixtures/returns/m2-hoh-two-children.json");
    let (s, result, _) = enter_through_the_hubs("m2-hoh-two-children", raw);
    assert_fixture_lines(raw, &result);
    assert_eq!(s.draft.dependents().len(), 2);
    // M3-P19: care is answered on the people page, and the child tax credit
    // and the EIC need no page; the credits hub walks only More credits.
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Wages,
            TaxSection::MoreIncome,
            TaxSection::Income,
            TaxSection::MoreAdjustments,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::Care,
            TaxSection::MoreCredits,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments
        ]
    );
}

// M3-P5: the self-employment topic (Schedule C page).

pub(in crate::tests) fn no_expenses() -> BusinessExpenses {
    BusinessExpenses {
        pension_plans: 0,
        advertising: 0,
        commissions: 0,
        contract_labor: 0,
        employee_benefits: 0,
        insurance: 0,
        mortgage_interest: 0,
        other_interest: 0,
        legal_and_professional: 0,
        office: 0,
        vehicle_and_equipment_rent: 0,
        other_rent: 0,
        repairs: 0,
        supplies: 0,
        taxes_and_licenses: 0,
        travel: 0,
        meals: vec![],
        utilities: 0,
        wages: 0,
        other: vec![],
    }
}
// A business with only its gross receipts entered and the required
// specified-service question answered No; every other field holds the
// page's default.
pub(in crate::tests) fn business(receipts: i64) -> Business {
    Business {
        principal_business: Default::default(),
        business_code: None,
        ein: Default::default(),
        address: Default::default(),
        started_current_year: false,
        information_returns: InformationReturnFiling::NoneRequired,
        returns: 0,
        other_income: vec![],
        specified_service: false,
        name: "".into(),
        receipts,
        accounting: BusinessAccounting::Cash,
        expenses: no_expenses(),
        vehicles: vec![],
        assets: vec![],
        more: business_details(),
    }
}
// The "More about this business" record holding the page's defaults.
pub(in crate::tests) fn business_details() -> BusinessDetails {
    BusinessDetails {
        research_property_choices: Vec::new(),
        de_minimis: None,
        depletion: None,
        overtime: None,
        ministerial: false,
        notary_profit: 0,
        notary_gross: None,

        inventory: BusinessInventory::None,
        startup: BusinessStartup::None,
        method_changes: Vec::new(),
        intangibles: Vec::new(),
        timber: Vec::new(),
        other_amortization: Vec::new(),
        payroll: BusinessPayroll::No,
        work_opportunity: None,
        retirement_plan: None,
        tip_fica: None,
        small_employer_health: None,
        paid_leave: None,
        fuel_tax_credit: None,
        cash_fuel_tax_recovery: None,
        tips: None,
        material_participation: true,
        at_risk: crate::RentalAtRisk::All,
        qbi_group: "".into(),
        income_source: BusinessIncomeChoice::NotNeeded,
        gambling: None,
        source: BusinessSource::UnitedStates,
        special: false,
        commercial_fishing: false,
        sale: None,
    }
}
// An asset with the page's defaults: 100% business use, bought when placed
// in service, new in 2026 with bonus depreciation.
pub(in crate::tests) fn asset(cost: i64, placed: u32, class: AssetClass) -> Asset {
    Asset {
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost,
        credit_reduction: 0,
        placed,
        class,
        business_percent: 100,
        acquired: None,
        history: AssetHistory::New { first_year: FirstYearDeduction::Bonus },
        disposal: crate::AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    }
}
pub(in crate::tests) fn with_first_year(mut asset: Asset, first_year: FirstYearDeduction) -> Asset {
    asset.history = AssetHistory::New { first_year };
    asset
}
pub(in crate::tests) fn save_businesses(
    session: &mut Interview,
    businesses: Vec<Business>,
) -> Reply {
    let request = save_topic(
        &session.draft.clone(),
        TaxSection::SelfEmployment,
        businesses.into_iter().map(|b| document(0, IncomeDocument::Business(b))).collect(),
    );
    send(session, request)
}

// The stored business of the session's only business entry.
pub(in crate::tests) fn stored_business(session: &Interview) -> (i64, FederalBusiness) {
    session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Business { data } => Some((e.id, data.clone())),
            _ => None,
        })
        .expect("a stored business")
}
// The business pages a restarted session reopens.
pub(in crate::tests) fn reopened_businesses(draft: FederalDraft) -> Vec<Business> {
    let mut session = restart(draft);
    page_documents(&open_topic(&mut session, TaxSection::SelfEmployment))
        .iter()
        .map(|item| {
            assert_eq!(item.preview, None);
            match &item.document {
                IncomeDocument::Business(b) => b.clone(),
                other => panic!("expected a business, got {other:?}"),
            }
        })
        .collect()
}
// Saving a business page and reopening it reproduces the page, and saving
// the reopened page again stores no new revision.
pub(in crate::tests) fn assert_round_trip(session: &mut Interview, page: &[Business]) {
    let mut expected = page.to_vec();
    let stored: Vec<_> = session
        .draft
        .income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::Business { data }
                if matches!(data.activity, BusinessActivity::Ordinary { .. }) =>
            {
                Some(data)
            }
            _ => None,
        })
        .collect();
    assert_eq!(stored.len(), expected.len());
    for (page, data) in expected.iter_mut().zip(stored) {
        bind_saved_property_ids(&mut page.assets, &mut page.vehicles, &data.assets);
        crate::tests::support::client::bind_saved_mileage_ids(&mut page.vehicles, &data.vehicles);
        page.more.research_property_choices = crate::cohorts::property_choices(data);
    }
    let reopened = reopened_businesses(session.draft.clone());
    assert_eq!(reopened, expected);
    let revision = session.draft.revision;
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::SelfEmployment)).to_vec();
    let request = save_topic(&again.draft.clone(), TaxSection::SelfEmployment, items);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft.revision, revision);
    assert!(!again.dirty);
}

// M3-P6: the rental property topic (Schedule E page) and the real estate
// professional situation.

pub(in crate::tests) fn no_rental_expenses() -> RentalExpenses {
    RentalExpenses {
        advertising: 0,
        auto_and_travel: 0,
        cleaning: 0,
        commissions: 0,
        insurance: 0,
        legal: 0,
        management: 0,
        mortgage_interest: 0,
        vehicle_loan_interest: 0,
        other_interest: 0,
        repairs: 0,
        supplies: 0,
        taxes: 0,
        utilities: 0,
        other: vec![],
    }
}
// The m3-self-employed-rental fixture's house: $260,000 including $40,000
// of land, first available to rent March 1, 2021.
fn fixture_building() -> Building {
    Building {
        commercial_1993_transition: None,
        allowed_commercial_1993_transition: Vec::new(),
        casualty: None,
        id: 0,
        energy: None,
        sale_history: None,
        credit_reduction: 0,
        acquired: crate::RentalAcquisition::Purchased,
        cost: 26_000_000,
        land: 4_000_000,
        in_service: 20210301,
        partial: None,
    }
}
// An improvement placed in service March 1, 2022 for $12,000.
pub(in crate::tests) fn fixture_improvement() -> Improvement {
    Improvement {
        commercial_1993_transition: None,
        allowed_commercial_1993_transition: Vec::new(),
        casualty: None,
        id: 0,
        energy: None,
        sale_history: None,
        credit_reduction: 0,
        cost: 1_200_000,
        in_service: 20220301,
        rehabilitation: None,
        partial: None,
    }
}
// A rental with its rent and building entered; every other field holds the
// page's default.
pub(in crate::tests) fn rental_page(rent: i64) -> Rental {
    Rental {
        mortgage_loans: Vec::new(),
        assets: alloc::vec::Vec::new(),
        address: "".into(),
        kind: RentalPropertyType::SingleFamily,
        rent,
        expenses: no_rental_expenses(),
        building: fixture_building(),
        personal_use: PersonalUse::No,
        more: rental_details(),
        sale: crate::RentalSale::No,
        statement: None,
    }
}
// The "More about this property" record holding the page's defaults.
pub(in crate::tests) fn rental_details() -> RentalDetails {
    RentalDetails {
        energy_buildings: Vec::new(),
        property_qbi_history: Vec::new(),
        improvements: vec![],
        participation: crate::RentalParticipation::Active,
        suspended: SuspendedLoss { regular: 0, amt: 0, active_in_loss_year: true },
        method_changes: Vec::new(),
        home_operating_carryover: 0,
        //2026-10-19: synthetic prior worksheets explicitly establish these independent records.
        home_depreciation_carryover: crate::SourceCents::Reported { amount: 0 },
        amt_home_depreciation_carryover: crate::SourceCents::Reported { amount: 0 },
        home_casualty_carryover: crate::SourceCents::NotReported,
        amt_home_casualty_carryover: crate::SourceCents::NotReported,
        qbi: RentalBusiness::No,
        qbi_group: "".into(),
        payroll: BusinessPayroll::No,
        at_risk: Some(crate::RentalAtRisk::All),
        special: false,
    }
}
pub(in crate::tests) fn save_rentals(session: &mut Interview, rentals: Vec<Rental>) -> Reply {
    let request = save_topic(
        &session.draft.clone(),
        TaxSection::Rentals,
        rentals.into_iter().map(|r| document(0, IncomeDocument::Rental(r))).collect(),
    );
    send(session, request)
}
pub(in crate::tests) fn stored_rentals(
    session: &Interview,
) -> Vec<(i64, alloc::string::String, FederalRental)> {
    session
        .draft
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Rental { data } => Some((e.id, e.label.clone(), data.clone())),
            _ => None,
        })
        .collect()
}
// The rental pages a restarted session reopens, with their previews.
pub(in crate::tests) fn reopened_rentals(draft: FederalDraft) -> Vec<(Rental, Option<i64>)> {
    let mut session = restart(draft);
    page_documents(&open_topic(&mut session, TaxSection::Rentals))
        .iter()
        .map(|item| match &item.document {
            IncomeDocument::Rental(r) => (r.clone(), item.preview),
            other => panic!("expected a rental, got {other:?}"),
        })
        .collect()
}
// Saving the reopened rental page again stores no new revision.
pub(in crate::tests) fn assert_rentals_round_trip(session: &mut Interview, page: &[Rental]) {
    let reopened: Vec<Rental> =
        reopened_rentals(session.draft.clone()).into_iter().map(|(r, _)| r).collect();
    assert_eq!(reopened, page);
    let revision = session.draft.revision;
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::Rentals)).to_vec();
    let request = save_topic(&again.draft.clone(), TaxSection::Rentals, items);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft.revision, revision);
    assert!(!again.dirty);
}
// A situations page (M3-P15: a hub's less-common page, other taxes or
// "Anything else?") as the review opens it: its page, offered set and
// stored choices.
fn open_situations(
    session: &mut Interview,
    topic: TaxSection,
) -> (i32, Vec<i32>, Vec<SituationItem>) {
    match send(session, navigate(&session.draft.clone(), topic)) {
        Reply::Response(P::BeginSituations { topic: shown, page, allowed_situations, .. })
            if shown == topic =>
        {
            (page, allowed_situations, vec![])
        }
        Reply::Edit(P::SaveSituations {
            topic: shown, page, allowed_situations, value, ..
        }) if shown == topic => (page, allowed_situations, value.situations),
        other => panic!("expected the {topic:?} situations page, got {other:?}"),
    }
}
// A save of `chosen` composed from the opened page's hidden fields.
fn situations_request(session: &mut Interview, topic: TaxSection, chosen: Vec<RareSituation>) -> P {
    let (page, allowed_situations, _) = open_situations(session, topic);
    let d = session.draft.clone();
    P::SaveSituations {
        user: TEST_USER,
        topic,
        allowed_situations,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page,
        previous: PAGE_REVIEW,
        review: true,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        value: Situations {
            situations: chosen.into_iter().map(|situation| SituationItem { situation }).collect(),
        },
    }
}
fn save_situations(
    session: &mut Interview,
    topic: TaxSection,
    chosen: Vec<RareSituation>,
) -> Reply {
    let request = situations_request(session, topic, chosen);
    send(session, request)
}
// The other taxes page (u5-credits), opened from the review.
pub(in crate::tests) fn open_other_taxes(session: &mut Interview) -> P {
    match send(session, navigate(&session.draft.clone(), TaxSection::OtherTaxes)) {
        Reply::Edit(page @ P::SaveOtherTaxes { .. }) => page,
        other => panic!("expected the other taxes page, got {other:?}"),
    }
}
pub(in crate::tests) fn other_taxes_request(page: &P, d: &FederalDraft, taxes: Vec<OtherTax>) -> P {
    let P::SaveOtherTaxes {
        page,
        previous,
        review,
        allowed_owners,
        coverdell_corrections,
        mortgage_gift_owners,
        ..
    } = page
    else {
        panic!("expected the other taxes page, got {page:?}")
    };
    P::SaveOtherTaxes {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: *page,
        previous: *previous,
        review: *review,
        estimate: Some(vec![]),
        warnings: vec![],
        progress: Some(vec![]),
        allowed_owners: allowed_owners.clone(),
        mortgage_gift_owners: mortgage_gift_owners.clone(),
        coverdell_corrections: coverdell_corrections.clone(),
        value: OtherTaxes { taxes },
    }
}
// A save of `taxes` composed from the page opened from the review.
pub(in crate::tests) fn save_other_taxes(session: &mut Interview, taxes: Vec<OtherTax>) -> Reply {
    let page = open_other_taxes(session);
    let request = other_taxes_request(&page, &session.draft.clone(), taxes);
    send(session, request)
}
// The real estate professional status the People and Spouse pages recorded.
pub(in crate::tests) fn professional_status(
    session: &Interview,
    owner: Owner,
) -> Option<ProfessionalRentals> {
    session.draft.people.as_ref().and_then(|p| p.professional(owner)).cloned()
}

#[test]
fn land_above_cost_is_rejected() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let mut r = rental_page(2_400_000);
    r.building.land = r.building.cost + 1;
    // The client rule's key, on save, in the running estimate and in the
    // row's preview.
    rejected_with(save_rentals(&mut session, vec![r.clone()]), "RENTAL_LAND_ABOVE_COST");
    assert_eq!(session.draft, before);
    let request = {
        let mut request = catalog_estimate_documents(
            TaxSection::Rentals,
            (IncomeDocuments {
                qbi_aggregations: Vec::new(),
                receipt_context: crate::BusinessReceiptContext::default(),
                documents: vec![document(0, IncomeDocument::Rental(r.clone()))],
            })
            .into(),
        );
        let P::EstimateDocuments { allowed_documents, allowed_owners, .. } = &mut request else {
            unreachable!()
        };
        *allowed_documents = vec![libertas_macros::variant_index!(IncomeDocument::Rental) as i32];
        *allowed_owners = vec![0];
        request
    };
    rejected_with(send(&mut session, request), "RENTAL_LAND_ABOVE_COST");
    let request = catalog_preview_document(IncomeDocument::Rental(r));
    rejected_with(send(&mut session, request), "RENTAL_LAND_ABOVE_COST");
    // An improvement carries no land, so only the building can fail the rule.
    // Land equal to the cost (a lot with no depreciable building) is accepted.
    let mut r = rental_page(2_400_000);
    r.building.land = r.building.cost;
    accepted(&save_rentals(&mut session, vec![r]));
    let (_, _, data) = &stored_rentals(&session)[0];
    assert_eq!(data.buildings[0].land, data.buildings[0].cost);
    // Personal-use days above the year are the other page rule.
    let mut r = rental_page(2_400_000);
    r.personal_use = PersonalUse::Yes {
        fair_rental_days: 300,
        personal_days: 66,
        overlap_days: 0,
        direct_expenses: 0,
    };
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_DAYS_ABOVE_365");
    // Rented days that were also personal days cannot exceed the rented days
    // (the page rule and its server twin).
    let mut r = rental_page(2_400_000);
    r.personal_use = PersonalUse::Yes {
        fair_rental_days: 10,
        personal_days: 0,
        overlap_days: 11,
        direct_expenses: 0,
    };
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_OVERLAP_DAYS_ABOVE_RENTAL_DAYS");
    // At the boundary the third count is stored and reopens unchanged.
    let mut r = rental_page(2_400_000);
    r.personal_use = PersonalUse::Yes {
        fair_rental_days: 85,
        personal_days: 14,
        overlap_days: 2,
        direct_expenses: 0,
    };
    accepted(&save_rentals(&mut session, vec![r]));
    let (_, _, data) = &stored_rentals(&session)[0];
    assert!(matches!(
        data.personal_use,
        RentalPersonalUse::Yes { fair_rental_days: 85, personal_days: 14, overlap_days: 2, .. }
    ));
}

// M3-P7: the adjustments hub and the HSA page.

// A W-2 whose box 12 reports code W employer HSA contributions.
pub(in crate::tests) fn w2_with_box_12_w(wages: i64, w: i64) -> W2 {
    W2 {
        more: W2More {
            payroll: W2Payroll::SocialSecurity,
            box12: vec![Box12Entry { code: W2Code::W, amount: w }],
            ..Default::default()
        },
        ..w2(wages)
    }
}
pub(in crate::tests) fn hsa_account(
    owner: Owner,
    coverage: HsaCoverageYear,
    personal: i64,
) -> HsaAccount {
    HsaAccount {
        owner,
        coverage,
        personal,
        employer: 0,
        corrections: vec![],
        late_excess: vec![],
        prior_employer_excess: vec![],
        excess_accounting: None,
        last_month_loss: None,
        last_month_history: None,
        funding_history: vec![],
        uses: vec![],
    }
}
// The HSA page's save as its hub opened it; the echoed allowed lists are
// never trusted, so they are sent empty.
pub(in crate::tests) fn save_hsa(
    d: &FederalDraft,
    accounts: Vec<HsaAccount>,
    family_split: FamilySplit,
) -> P {
    // Neutral completion of the preexisting one-account fixture: the other
    // filer is ineligible unless a test supplies different actual context.
    let other_coverage = (accounts.len() == 1
        && d.setup.as_ref().is_some_and(|s| {
            matches!(s.filing_status(), FederalStatus::Joint | FederalStatus::Separate)
        }))
    .then_some(HsaEligibility::FullYear { coverage: HsaCoverage::None });
    // These preexisting synthetic fixtures designate the reported principal
    // for the current contribution year; make that source fact explicit.
    let mut employer_sources =
        crate::health_employer::page_rows(d, crate::health_employer::bindings(d)).unwrap();
    for source in &mut employer_sources {
        source.allocation = HealthEmployerYear::CurrentYear;
    }
    P::SaveHsa {
        user: TEST_USER,
        hsa_failed_deposits: vec![],
        archer_failed_deposits: vec![],
        hsa_return_documents: vec![],
        archer_return_documents: vec![],
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_ADJUSTMENTS,
        previous: PAGE_ADJUSTMENTS,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        other_coverage_allowed: false,
        allowed_owners: vec![],
        allowed_family_split: vec![],
        ira_sources: vec![],
        archer_income: vec![],
        value: HsaPage {
            employer_sources,
            linked_deposits: vec![],
            other_coverage,
            family_method: HsaAllocationMethod::Monthly,
            accounts,
            family_split,
            funding: vec![],
            archer: ArcherContributionsPage::None,
        },
    }
}
pub(in crate::tests) fn hsa_options(accounts: Vec<HsaAccount>) -> P {
    P::HsaOptions {
        hsa_failed_deposits: vec![],
        archer_failed_deposits: vec![],
        hsa_return_documents: vec![],
        archer_return_documents: vec![],
        other_coverage: None,
        family_method: HsaAllocationMethod::Monthly,
        user: TEST_USER,
        allowed_owners: vec![0, 1],
        accounts,
    }
}
pub(in crate::tests) fn family_options(reply: Reply) -> Vec<i32> {
    match reply {
        Reply::Response(P::HsaFamilyOptions { allowed_family_split }) => allowed_family_split,
        other => panic!("expected the family split choices, got {other:?}"),
    }
}
// A complete return still at its adjustments hub, whose one W-2 carries
// the given box 12 code W amount (none when zero).
pub(in crate::tests) fn at_the_adjustments_hub(status: FederalStatus, w: i64) -> Interview {
    let mut d = complete(status);
    d.finished_sections = vec![TaxSection::Children, TaxSection::Checklist, TaxSection::Income];
    let mut s = Interview::new(TEST_USER, d);
    let wage = if w == 0 { w2(6_000_000) } else { w2_with_box_12_w(6_000_000, w) };
    let request =
        save_topic(&s.draft, TaxSection::Wages, vec![document(0, IncomeDocument::W2(wage))]);
    accepted(&send(&mut s, request));
    s
}
// The hub behind a topic page the walk opened (M3-P19): a hub entered by
// the sequence opens its first planned or derived topic directly, whose
// Back opens the hub. Any other reply is returned unchanged.
pub(in crate::tests) fn to_the_hub(s: &mut Interview, reply: Reply) -> Reply {
    let hubs = [PAGE_INCOME, PAGE_ADJUSTMENTS, PAGE_ITEMIZED, PAGE_CREDITS];
    let previous = match &reply {
        Reply::Edit(
            P::SaveIncomes { previous, .. }
            | P::SaveHsa { previous, .. }
            | P::SaveSelfEmployedBenefits { previous, .. }
            | P::SaveIraContributions { previous, .. }
            | P::SaveEducator { previous, .. }
            | P::SaveStudentLoan { previous, .. }
            | P::SaveMedical { previous, .. }
            | P::SaveTaxes { previous, .. }
            | P::SaveMortgages { previous, .. }
            | P::SaveGiftCarryovers { previous, .. }
            | P::SaveOtherItemized { previous, .. }
            | P::SaveEducation { previous, .. }
            | P::SaveCare { previous, .. }
            | P::SaveSaver { previous, .. }
            | P::SaveMoreCredits { previous, .. }
            | P::SaveMoreAdjustments { previous, .. }
            | P::SaveSituations { previous, .. },
        )
        | Reply::Response(P::BeginSituations { previous, .. })
            if hubs.contains(previous) =>
        {
            *previous
        }
        _ => return reply,
    };
    let request = back(&s.draft, previous);
    send(s, request)
}
// Opens the adjustments hub: Continue from the income hub while the section
// has something for the return, which walks into its first derived topic
// (M3-P19) whose Back opens the hub; a section passed with nothing is
// opened from the review.
pub(in crate::tests) fn open_adjustments_hub(s: &mut Interview) -> P {
    let from = if s.draft.finished_sections.contains(&TaxSection::Adjustments) {
        PAGE_REVIEW
    } else {
        PAGE_INCOME
    };
    let request = go_to(&s.draft, from, TaxSection::Adjustments);
    let reply = send(s, request);
    match to_the_hub(s, reply) {
        Reply::Response(hub @ P::Hub { page: PAGE_ADJUSTMENTS, .. }) => hub,
        other => panic!("expected the adjustments hub, got {other:?}"),
    }
}
pub(in crate::tests) fn open_hsa_page(s: &mut Interview) -> P {
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::Hsa);
    match send(s, request) {
        Reply::Edit(page @ P::SaveHsa { .. }) => page,
        other => panic!("expected the HSA page, got {other:?}"),
    }
}
pub(in crate::tests) fn hsa_page_value(page: &P) -> (&HsaPage, &[i32]) {
    let P::SaveHsa { value, allowed_family_split, .. } = page else {
        panic!("expected the HSA page, got {page:?}")
    };
    (value, allowed_family_split)
}

#[test]
fn adjustments_hub_suggests_hsa_when_box_12_w_exists() {
    // Without box 12 W the section is still walked for its More adjustments
    // page (gate fixes batch A): its hub suggests that page.
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 0);
    assert!(!s.draft.finished_sections.contains(&TaxSection::Adjustments));
    let hub = open_adjustments_hub(&mut s);
    let P::Hub { page, previous, suggested, allowed_sections, progress, .. } = &hub else {
        unreachable!()
    };
    assert_eq!((*page, *previous), (PAGE_ADJUSTMENTS, PAGE_INCOME));
    // The income hub, the HSA, IRA, educator and student loan topics (no
    // business, so self-employed benefits is not offered) and the next
    // section.
    // Amended for M3-P9: the next section is the deductions page.
    // Amended for M3-P15: more adjustments before the next section.
    // M3-P18: and the checklist after the income hub.
    // M3-P19: the next shown section, here credits (gate fixes batch A).
    assert_available_sections(
        allowed_sections,
        &[
            TaxSection::Income,
            TaxSection::Checklist,
            TaxSection::Hsa,
            TaxSection::IraContributions,
            TaxSection::Educator,
            TaxSection::StudentLoan,
            TaxSection::MoreAdjustments,
            TaxSection::Credits,
        ]
        .map(|t| t as i32),
    );
    assert_eq!(*suggested, TaxSection::MoreAdjustments);
    // The sections counted: setup, you, people, the checklist, income,
    // adjustments, credits, other taxes and the review.
    assert!(shown(progress).contains("2026 federal · section 6 of 10"));
    // Amended for M3-N1: every offered topic and Continue are rows.
    assert_eq!(
        hub_rows(&hub),
        hub_tail_unasked(
            &[
                TaxSection::Hsa,
                TaxSection::IraContributions,
                TaxSection::Educator,
                TaxSection::StudentLoan,
                TaxSection::EmployeeExpenses
            ],
            TaxSection::MoreAdjustments
        )
    );
    // A W-2 with box 12 W makes the HSA topic necessary.
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 100_000);
    let hub = open_adjustments_hub(&mut s);
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Hsa);
    // Amended for M3-P16: the missing account is the topic's attention.
    // Amended for M3-N1: then every other topic; Continue is a separate action.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of(
                "Health and medical savings accounts",
                "Needs attention: Employer HSA contributions or a linked failed cash deposit need this owner’s matching health-account facts",
                TaxSection::Hsa
            )],
            hub_tail_unasked(
                &[
                    TaxSection::IraContributions,
                    TaxSection::Educator,
                    TaxSection::StudentLoan,
                    TaxSection::EmployeeExpenses
                ],
                TaxSection::MoreAdjustments
            )
        )
    );
    // OK opens the HSA page as a complete editor with an empty list.
    let Reply::Edit(page) = send(&mut s, go_on(&hub)) else { panic!("expected the HSA page") };
    let P::SaveHsa { page: at, previous, progress, allowed_owners, .. } = &page else {
        panic!("expected the HSA page, got {page:?}")
    };
    assert_eq!((*at, *previous), (PAGE_ADJUSTMENTS, PAGE_ADJUSTMENTS));
    assert_eq!(allowed_owners, &vec![Owner::Taxpayer as i32]);
    assert!(shown(progress).contains("section 6 of 10 · Health and medical savings accounts"));
    let (value, allowed_family_split) = hsa_page_value(&page);
    assert!(value.accounts.is_empty());
    assert_eq!(value.family_split, FamilySplit::NotApplicable);
    assert_eq!(allowed_family_split, &[0]);
    // Back returns to the hub.
    let request = back(&s.draft, PAGE_ADJUSTMENTS);
    assert!(matches!(
        send(&mut s, request),
        Reply::Response(P::Hub { page: PAGE_ADJUSTMENTS, .. })
    ));
    // Saving the account walks on to More adjustments (gate fixes batch A),
    // whose Back opens the hub, which counts the account.
    let request = save_hsa(
        &s.draft,
        vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 200_000)],
        FamilySplit::NotApplicable,
    );
    let reply = send(&mut s, request);
    assert!(matches!(reply, Reply::Edit(P::SaveMoreAdjustments { .. })));
    let Reply::Response(hub) = to_the_hub(&mut s, reply) else {
        panic!("expected the adjustments hub")
    };
    let P::Hub { suggested, .. } = &hub else {
        panic!("expected the adjustments hub, got {hub:?}")
    };
    assert_eq!(*suggested, TaxSection::MoreAdjustments);
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of("Health and medical savings accounts", "1 account", TaxSection::Hsa)],
            hub_tail_unasked(
                &[
                    TaxSection::IraContributions,
                    TaxSection::Educator,
                    TaxSection::StudentLoan,
                    TaxSection::EmployeeExpenses
                ],
                TaxSection::MoreAdjustments
            )
        )
    );
    // Continuing records the section and walks on to the review, passing
    // each later section's less-common page (gate fixes batch A). The
    // derived HSA topic, once saved, is recorded as visited.
    let reply = send(&mut s, hub_action(&hub));
    walk_on_to_review(&mut s, reply);
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Hsa,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::MoreCredits,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
        ]
    );
}

#[test]
fn employer_contributions_come_from_box_12_w() {
    // The m3-itemizing-homeowner fixture's HSA share: a joint return, box 12
    // W $1,000 on the taxpayer's W-2, family coverage all year and $3,000 of
    // personal contributions. By hand (Form 8889): line 3 $8,750 (family
    // limit), line 6 $8,750, line 8 $8,750, line 9 $1,000 (box 12 W), line 12
    // $8,750 - $1,000 = $7,750, line 2 $3,000, line 13 = min($3,000, $7,750)
    // = $3,000 on Schedule 1 line 13.
    let mut s = at_the_adjustments_hub(FederalStatus::Joint, 100_000);
    open_adjustments_hub(&mut s);
    // The W-2 amount is linked, not reentered. The separate outside-W-2
    // amount is actually zero in this fixture.
    let account = hsa_account(Owner::Taxpayer, HsaCoverageYear::Family, 300_000);
    let request = save_hsa(&s.draft, vec![account], FamilySplit::NotApplicable);
    accepted(&send(&mut s, request));
    let stored = &s.draft.adjustments.as_ref().unwrap().hsa;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].employer, 100_000);
    assert_eq!(stored[0].personal, 300_000);
    assert_eq!(stored[0].coverage, HsaEligibility::FullYear { coverage: HsaCoverage::Family });
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(value(&r, "adjustments"), 3_000);
    // The page retains outside principal separately from the linked source.
    let page = open_hsa_page(&mut s);
    let (value, _) = hsa_page_value(&page);
    assert_eq!(value.accounts[0].employer, 0);
    assert_eq!(value.employer_sources[0].reported, 100_000);
    // Saving the reopened page stores nothing new.
    let revision = s.draft.revision;
    accepted(&send(&mut s, page.clone()));
    assert_eq!(s.draft.revision, revision);
    // A changed W-2 re-derives the stored amount; nothing is erased.
    let request = save_topic(
        &s.draft,
        TaxSection::Wages,
        vec![document(
            s.draft.income[0].id,
            IncomeDocument::W2(w2_with_box_12_w(6_000_000, 150_000)),
        )],
    );
    accepted(&send(&mut s, request));
    let stored = &s.draft.adjustments.as_ref().unwrap().hsa;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].employer, 150_000);
    // Box 12 W reaches only its owner's account: the spouse's account shows
    // none from the taxpayer's W-2.
    let request = save_hsa(
        &s.draft,
        vec![
            hsa_account(Owner::Taxpayer, HsaCoverageYear::Family, 300_000),
            hsa_account(Owner::Spouse, HsaCoverageYear::Family, 0),
        ],
        FamilySplit::Equal,
    );
    accepted(&send(&mut s, request));
    let employer: Vec<_> =
        s.draft.adjustments.as_ref().unwrap().hsa.iter().map(|a| (a.owner, a.employer)).collect();
    assert_eq!(employer, vec![(Owner::Taxpayer, 150_000), (Owner::Spouse, 0)]);
}

#[test]
fn family_split_uses_either_family_policy_and_overlapping_eligible_months() {
    let mut s = at_the_adjustments_hub(FederalStatus::Joint, 0);
    let family = |owner| hsa_account(owner, HsaCoverageYear::Family, 100_000);
    let self_only = |owner| hsa_account(owner, HsaCoverageYear::SelfOnly, 100_000);
    let changed = |owner| {
        let mut months = vec![HsaCoverage::SelfOnly; 12];
        months[11] = HsaCoverage::Family;
        hsa_account(owner, HsaCoverageYear::Changed { months }, 100_000)
    };
    // The calculated allowed set, from the accounts alone.
    for (accounts, expected) in [
        (vec![], vec![0]),
        (vec![family(Owner::Taxpayer)], vec![0]),
        (vec![family(Owner::Taxpayer), self_only(Owner::Spouse)], vec![1, 2]),
        (vec![family(Owner::Taxpayer), family(Owner::Taxpayer)], vec![0]),
        (vec![family(Owner::Taxpayer), family(Owner::Spouse)], vec![1, 2]),
        (vec![changed(Owner::Taxpayer), family(Owner::Spouse)], vec![1, 2]),
    ] {
        assert_eq!(family_options(send(&mut s, hsa_options(accounts))), expected);
    }
    // Only a joint return has two filers' accounts to split between.
    let mut single = at_the_adjustments_hub(FederalStatus::Single, 0);
    assert_eq!(
        family_options(send(
            &mut single,
            hsa_options(vec![family(Owner::Taxpayer), family(Owner::Spouse)])
        )),
        vec![0]
    );
    // The save checks the same set again.
    let before = s.draft.clone();
    let request = save_hsa(&before, vec![family(Owner::Taxpayer)], FamilySplit::Equal);
    assert!(refusal(send(&mut s, request)).contains("Split the family contribution limit"));
    assert_eq!(s.draft, before);
    // Two family accounts: Equally is the monthly method, one part each.
    let two = vec![family(Owner::Taxpayer), family(Owner::Spouse)];
    let request = save_hsa(&s.draft, two.clone(), FamilySplit::Equal);
    accepted(&send(&mut s, request));
    let share = s.draft.adjustments.as_ref().unwrap().hsa_family_share.clone();
    assert_eq!(
        share,
        Some(HsaFamilyShare {
            method: HsaAllocationMethod::Monthly,
            taxpayer_parts: 1,
            spouse_parts: 1,
            other_coverage: None,
        })
    );
    let page = open_hsa_page(&mut s);
    let (value, allowed) = hsa_page_value(&page);
    assert_eq!(allowed, &[1, 2]);
    assert_eq!(value.family_split, FamilySplit::Equal);
    // As agreed keeps the parts and reopens as entered.
    let agreed = FamilySplit::Agreed { you: 3, spouse: 1 };
    let request = save_hsa(&s.draft, two.clone(), agreed);
    accepted(&send(&mut s, request));
    let share = s.draft.adjustments.as_ref().unwrap().hsa_family_share.clone();
    assert_eq!(share.map(|v| (v.taxpayer_parts, v.spouse_parts)), Some((3, 1)));
    assert_eq!(hsa_page_value(&open_hsa_page(&mut s)).0.family_split, agreed);
    // One account per owner, checked on save.
    let before = s.draft.clone();
    let request = save_hsa(
        &before,
        vec![family(Owner::Taxpayer), self_only(Owner::Taxpayer)],
        FamilySplit::NotApplicable,
    );
    rejected_with(send(&mut s, request), "HSA_ACCOUNT_TWICE");
    assert_eq!(s.draft, before);
    // Leaving the joint return drops the spouse's account and the split.
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let adjustments = s.draft.adjustments.as_ref().unwrap();
    assert_eq!(adjustments.hsa.len(), 1);
    assert_eq!(adjustments.hsa[0].owner, Owner::Taxpayer);
    assert_eq!(
        adjustments.hsa_family_share,
        Some(HsaFamilyShare {
            method: HsaAllocationMethod::Monthly,
            taxpayer_parts: 1,
            spouse_parts: 1,
            other_coverage: None,
        })
    );
    // A separate return now supplies actual other-filer eligibility without
    // importing a spouse account or requiring an allocation agreement.
    let mut separate = at_the_adjustments_hub(FederalStatus::Separate, 0);
    let before = separate.draft.clone();
    let request = save_hsa(&before, vec![family(Owner::Taxpayer)], FamilySplit::NotApplicable);
    accepted(&send(&mut separate, request));
    assert_eq!(super::value(&supported(&separate.draft), "adjustments"), 1000);
    let request =
        save_hsa(&separate.draft, vec![self_only(Owner::Taxpayer)], FamilySplit::NotApplicable);
    accepted(&send(&mut separate, request));
}

// M3-P8: self-employed benefits, IRA contributions, educator expenses and
// student loan interest.
pub(in crate::tests) fn at_the_adjustments_hub_with_business(receipts: i64) -> Interview {
    let mut d = empty_return(FederalStatus::Single);
    d.finished_sections = vec![TaxSection::Children, TaxSection::Checklist, TaxSection::Income];
    let mut s = Interview::new(TEST_USER, d);
    save_businesses(&mut s, vec![business(receipts)]);
    s
}

// M3-P9: the deductions page and the itemized hub.

// A complete return whose adjustments hub was left: resuming opens the
// deductions page, returned with it.
pub(in crate::tests) fn at_the_deductions_page(status: FederalStatus) -> (Interview, P) {
    let mut d = complete(status);
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Adjustments,
    ];
    let mut s = Interview::new(TEST_USER, d);
    match send(&mut s, P::OpenInterview { user: TEST_USER }) {
        Reply::Edit(page @ P::SaveDeductions { .. }) => (s, page),
        other => panic!("expected the deductions page, got {other:?}"),
    }
}
pub(in crate::tests) fn deductions_value(page: &P) -> (&Deductions, &[i32]) {
    let P::SaveDeductions { value, allowed_routes, .. } = page else {
        panic!("expected the deductions page, got {page:?}")
    };
    (value, allowed_routes)
}
// The credits hub (M3-P11), which follows the deductions page and the
// itemized hub; its Continue reaches the review.
pub(in crate::tests) fn credits_hub(reply: Reply) -> P {
    match reply {
        Reply::Response(hub @ P::Hub { page: PAGE_CREDITS, .. }) => hub,
        other => panic!("expected the credits hub, got {other:?}"),
    }
}
pub(in crate::tests) fn itemized_hub(reply: Reply) -> P {
    match reply {
        Reply::Response(hub @ P::Hub { page: PAGE_ITEMIZED, .. }) => hub,
        other => panic!("expected the itemized hub, got {other:?}"),
    }
}
pub(in crate::tests) fn open_itemized_topic(s: &mut Interview, topic: TaxSection) -> P {
    let request = go_to(&s.draft, PAGE_ITEMIZED, topic);
    match send(s, request) {
        Reply::Edit(page) => page,
        other => panic!("expected the {topic:?} page, got {other:?}"),
    }
}
// The medical page's save as the itemized hub opened it.
pub(in crate::tests) fn save_medical(d: &FederalDraft, value: Medical) -> P {
    P::SaveMedical {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_ITEMIZED,
        previous: PAGE_ITEMIZED,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        value,
    }
}
// The mortgage page's save as the itemized hub opened it (M3-P10).
pub(in crate::tests) fn save_mortgages(d: &FederalDraft, value: Mortgages) -> P {
    P::SaveMortgages {
        mortgage_loans: vec![],
        rental_mortgage_loans: vec![],
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_ITEMIZED,
        previous: PAGE_ITEMIZED,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
// The charity carryovers page's save as the itemized hub opened it (M3-P10).
pub(in crate::tests) fn save_gift_carryovers(d: &FederalDraft, value: GiftCarryovers) -> P {
    P::SaveGiftCarryovers {
        retained_charity_k1s: crate::investment_interest::k1_sources(d),
        retained_charity_origins: crate::retained_charity::origin_choices(d),
        retained_research_choices: crate::retained_research::available(d),
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_ITEMIZED,
        previous: PAGE_ITEMIZED,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        allowed_owners: vec![],
        appraisal_choices: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
pub(in crate::tests) fn stored_facts(s: &Interview) -> ItemizedExpenses {
    match &s.draft.deductions.as_ref().unwrap().choice {
        DeductionSelection::Automatic { expenses: ItemizedChoice::Yes { facts } }
        | DeductionSelection::Itemized { facts } => facts.clone(),
        other => panic!("expected itemized facts, got {other:?}"),
    }
}
pub(in crate::tests) fn empty_facts() -> ItemizedExpenses {
    ItemizedExpenses {
        income_tax_owners: None,
        investment_property_tax_owners: None,
        investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
        real_estate_taxes: 0,
        other_lines: vec![],
        medical: 0,
        long_term_care: vec![],
        personal_tax: PersonalTaxElection::None,
        personal_property_taxes: 0,
        investment_property_taxes: 0,
    }
}

// A loan into the itemizing hub's mortgage topic, one Next away from the
// deductions page (M3-P10).
fn mortgage_loan_page(mut s: Interview) -> Interview {
    let request = save_deductions(&s.draft, DeductionRoute::Compare, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    s
}

#[test]
fn within_limit_default_deducts_all_interest() {
    // Pub 936 Table 1: a post-2017 $400,000 loan deducts all $18,000.
    // A $900,000 loan needs its actual average; last balance cannot exceed box 2.
    for (interest, principal, average, refusal) in [
        (1_800_000, 40_000_000, AverageBalance::WithinLimit, None),
        (4_000_000, 90_000_000, AverageBalance::WithinLimit, Some("MORTGAGE_AVERAGE_NEEDED")),
        (
            1_000_000,
            20_000_000,
            AverageBalance::FirstAndLast { last_balance: 21_000_000 },
            Some("MORTGAGE_LAST_BALANCE_ABOVE_BOX_2"),
        ),
    ] {
        let mut s = mortgage_loan_page(at_the_deductions_page(FederalStatus::Single).0);
        let before = s.draft.clone();
        let loan = first_lender_mortgage(interest, principal, average);
        let request = save_mortgages(&s.draft, Mortgages { loans: vec![loan] });
        if let Some(key) = refusal {
            rejected_with(send(&mut s, request), key);
            assert_eq!(s.draft, before);
        } else {
            itemized_hub(send_past_less_common(&mut s, request));
            assert_eq!(value(&estimate(&s.draft), "mortgage_interest_8a"), 18_000);
        }
    }
}

#[test]
fn points_branches_map_onto_mortgage_points() {
    // Purchase points (main-home acquisition, every test confirmed, not a
    // refinance) deduct in full: MortgagePoints::Yes with the purchase
    // defaults (refinance No, full_year_tests Yes, a 360-month term, closed
    // on the loan's own origination date). Amortized points (a refinance,
    // or points that fail a purchase test) keep the entered term and date.
    let mut s = mortgage_loan_page(at_the_deductions_page(FederalStatus::Single).0);
    let request = save_mortgages(
        &s.draft,
        Mortgages {
            loans: vec![
                Mortgage1098 {
                    id: 0,
                    interest: 1_000_000,
                    principal: 20_000_000,
                    originated: 20260201,
                    refund: 0,
                    insurance: 0,
                    points: Points::Purchase { amount: 300_000 },
                    more: MortgageMore {
                        details: MortgageDetails::FirstLender(MortgageFirstLender {
                            home: MortgageHome::Main,
                            purpose: MortgageUse::Home,
                            average: AverageBalance::WithinLimit,
                        }),
                        ..MortgageMore::default()
                    },
                },
                Mortgage1098 {
                    id: 0,
                    interest: 500_000,
                    principal: 10_000_000,
                    originated: 20240101,
                    refund: 0,
                    insurance: 0,
                    points: Points::Amortized { amount: 240_000, ended: false, term_months: 240 },
                    more: MortgageMore {
                        details: MortgageDetails::FirstLender(MortgageFirstLender {
                            home: MortgageHome::Second,
                            purpose: MortgageUse::Home,
                            average: AverageBalance::WithinLimit,
                        }),
                        ..MortgageMore::default()
                    },
                },
            ],
        },
    );
    itemized_hub(send_past_less_common(&mut s, request));
    let loans = &s.draft.deductions.as_ref().unwrap().mortgage;
    assert_eq!(loans[0].points, MortgagePoints::Purchase { amount: 300_000 });
    assert_eq!(
        loans[1].points,
        MortgagePoints::Amortized { amount: 240_000, term_months: 240, ended: false }
    );
}

#[test]
fn carryovers_map_onto_prior_cash_charity() {
    // Each CashCarryover becomes a complete PriorCashCharityCarry entry: the
    // page always supplies both confirmations and its own record reference.
    let mut s = mortgage_loan_page(at_the_deductions_page(FederalStatus::Single).0);
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
                    year: 2022,
                    class: CharityClass::Cash60,
                    balance: 400_000,
                    appreciation: None,
                    same_filers: true,
                }
                .into(),
                CharityCarryover {
                    research_parts: None,
                    year: 2024,
                    class: CharityClass::Cash60,
                    balance: 150_000,
                    appreciation: None,
                    same_filers: true,
                }
                .into(),
            ],
            volunteer: vec![],
            less_common: vec![],
        },
    );
    itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(
        s.draft.deductions.as_ref().unwrap().charity_carryovers,
        vec![
            CharityCarryover {
                research_parts: None,
                year: 2022,
                class: CharityClass::Cash60,
                balance: 400_000,
                appreciation: None,
                same_filers: true
            },
            CharityCarryover {
                research_parts: None,
                year: 2024,
                class: CharityClass::Cash60,
                balance: 150_000,
                appreciation: None,
                same_filers: true
            },
        ]
    );
    // Reopened, the balances come back unchanged.
    let P::SaveGiftCarryovers { value, .. } =
        open_itemized_topic(&mut s, TaxSection::GiftCarryovers)
    else {
        panic!("expected the gift carryovers page")
    };
    assert_eq!(
        value.carryovers,
        vec![
            CharityCarryoverPage::from(CharityCarryover {
                research_parts: None,
                year: 2022,
                class: CharityClass::Cash60,
                balance: 400_000,
                appreciation: None,
                same_filers: true
            }),
            CharityCarryoverPage::from(CharityCarryover {
                research_parts: None,
                year: 2024,
                class: CharityClass::Cash60,
                balance: 150_000,
                appreciation: None,
                same_filers: true
            }),
        ]
    );
    // Standard clears the carryovers along with the itemized facts.
    let request = save_deductions(&s.draft, DeductionRoute::Standard, 0);
    send(&mut s, request);
    assert!(s.draft.deductions.as_ref().unwrap().charity_carryovers.is_empty());
}

#[test]
fn inline_choices_mark_every_answer_field() {
    // Unit M3-N2: `Answer` and `Owner` carry `libertas_inline_choices` on their
    // type declarations in common.rs, so every field of either type on every
    // page renders inline radios; the small "Other boxes" unions and the
    // other choice types with at most four record-or-unit choices carry it in
    // federal_model.rs. A field-level marker on an Answer or Owner field would
    // be redundant and is rejected here.
    fn type_is_marked(source: &str, name: &str) -> bool {
        let decl = alloc::format!("pub enum {name} {{");
        let at = source.find(&decl).unwrap_or_else(|| panic!("{name} not found"));
        source[..at]
            .rsplit("\n\n")
            .next()
            .map(|block| block.contains("#[libertas_inline_choices]"))
            .unwrap_or(false)
    }
    let common = include_str!("../../core/common.rs");
    assert!(type_is_marked(common, "Answer"));
    assert!(type_is_marked(common, "Owner"));
    let model = include_str!("../../federal/federal_model.rs");
    for name in [
        "DeductionRoute",
        "SaltElection",
        "FamilySplit",
        "Points",
        "AverageBalance",
        "VehicleMethod",
        "PenaltyChoice",
        "PriorReturn",
        "WithholdingTiming",
        "IncomeTiming",
        "Farmer",
        "Waiver",
        "GoodsKind",
        "VehicleUse",
    ] {
        assert!(type_is_marked(model, name), "{name} lacks libertas_inline_choices");
    }
    // Canonical HOME replaces the embedded HomeOffice choice.
    assert!(type_is_marked(include_str!("../../home/sites.rs"), "OfficeMethod"));
    let mut previous_marked = false;
    for line in model.lines() {
        let trimmed = line.trim();
        if trimmed == "#[libertas_inline_choices]" {
            previous_marked = true;
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("///") {
            continue;
        }
        if previous_marked && (trimmed.ends_with(": Answer,") || trimmed.ends_with(": Owner,")) {
            panic!("redundant field-level marker before `{trimmed}`");
        }
        previous_marked = false;
    }
}

// M3-P11: the credits hub, education and the retirement savings credit.

// A saved return at its credits hub: every earlier section is finished.
pub(in crate::tests) fn at_the_credits_hub(mut d: FederalDraft) -> (Interview, P) {
    d.finished_sections.retain(|s| *s != TaxSection::Credits);
    let mut s = Interview::new(TEST_USER, d);
    let reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    let hub = credits_hub(to_the_hub(&mut s, reply));
    (s, hub)
}
pub(in crate::tests) fn open_credits_topic(s: &mut Interview, topic: TaxSection) -> P {
    let request = go_to(&s.draft, PAGE_CREDITS, topic);
    match send(s, request) {
        Reply::Edit(page) => page,
        other => panic!("expected the {topic:?} page, got {other:?}"),
    }
}
pub(in crate::tests) fn education_value(page: &P) -> &EducationCredits {
    let P::SaveEducation { value, .. } = page else {
        panic!("expected the education page, got {page:?}")
    };
    value
}
pub(in crate::tests) fn save_education(d: &FederalDraft, value: EducationCredits) -> P {
    P::SaveEducation {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_CREDITS,
        previous: PAGE_CREDITS,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
pub(in crate::tests) fn more_credits_value(page: &P) -> &MoreCredits {
    let P::SaveMoreCredits { value, .. } = page else {
        panic!("expected the more credits page, got {page:?}")
    };
    value
}
pub(in crate::tests) fn save_more_credits(d: &FederalDraft, value: MoreCredits) -> P {
    P::SaveMoreCredits {
        initial_carbon_oxide_origins: vec![],
        initial_renewable_facility_origins: Vec::new(),
        clean_vehicle_properties: vec![],
        businesses: vec![],
        farms: vec![],
        rentals: vec![],
        research_businesses: vec![],
        entered_credits: vec![],
        carryback_credits: vec![],
        allowed_carry_filers: vec![],
        initial_credit_elections: vec![],
        initial_research_choices: vec![],
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_CREDITS,
        previous: PAGE_CREDITS,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: None,
        allowed_owners: vec![Owner::Taxpayer as i32],
        foreign_businesses: vec![],
        capital_sales: vec![],
        foreign_situations: vec![],
        mortgage_loans: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
pub(in crate::tests) fn student(student: i64, tuition: i64) -> StudentCredit {
    StudentCredit {
        student,
        tuition,
        scholarships: 0,
        half_time: true,
        graduate: false,
        more: StudentMore::default(),
    }
}

#[test]
fn validation_audit_f07_education_duplicate_student_rejects_before_persistence() {
    let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Single));
    let mut page = open_credits_topic(&mut session, TaxSection::Education);
    let P::SaveEducation { value, .. } = &mut page else { panic!("expected the education page") };
    value.students = vec![student(-1, 300_000), student(-1, 200_000)];
    let before = session.draft.clone();
    let dirty = session.dirty;
    rejected_with(
        current_source_refusal(
            "secondary-admission-7443",
            &mut session,
            page,
            "ISSUE_USE_ONE_CREDIT_ENTRY_PER_STUDENT_DO_NOT_DUPLICATE_EXPENSES_ACROSS_C_D52B32DA",
            Some("[value/students/1/student]"),
            SourceScope::Secondary(SecondaryCorrection::StudentDuplicate),
        ),
        "ISSUE_USE_ONE_CREDIT_ENTRY_PER_STUDENT_DO_NOT_DUPLICATE_EXPENSES_ACROSS_C_D52B32DA",
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn validation_audit_f07_education_distinct_students_and_empty_list_save_and_reopen() {
    let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Joint));
    let mut page = open_credits_topic(&mut session, TaxSection::Education);
    let expected = vec![student(-1, 300_000), student(-2, 200_000)];
    let P::SaveEducation { value, .. } = &mut page else { panic!("expected the education page") };
    value.students = expected.clone();
    accepted(&send(&mut session, page));
    supported(&session.draft);
    let mut reopened = open_credits_topic(&mut session, TaxSection::Education);
    assert_eq!(education_value(&reopened).students, expected);
    let P::SaveEducation { value, .. } = &mut reopened else { unreachable!() };
    value.students[1].student = -1;
    let before = session.draft.clone();
    let dirty = session.dirty;
    rejected_with(
        current_source_refusal(
            "secondary-admission-7485",
            &mut session,
            reopened,
            "ISSUE_USE_ONE_CREDIT_ENTRY_PER_STUDENT_DO_NOT_DUPLICATE_EXPENSES_ACROSS_C_D52B32DA",
            Some("[value/students/1/student]"),
            SourceScope::Secondary(SecondaryCorrection::StudentDuplicate),
        ),
        "ISSUE_USE_ONE_CREDIT_ENTRY_PER_STUDENT_DO_NOT_DUPLICATE_EXPENSES_ACROSS_C_D52B32DA",
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    let mut reopened = open_credits_topic(&mut session, TaxSection::Education);
    assert_eq!(education_value(&reopened).students, expected);
    let P::SaveEducation { value, .. } = &mut reopened else { unreachable!() };
    value.students.clear();
    accepted(&send(&mut session, reopened));
    assert!(
        education_value(&open_credits_topic(&mut session, TaxSection::Education))
            .students
            .is_empty()
    );
    supported(&session.draft);
}
// Estimates the accepted return with the credits hub continued.
pub(in crate::tests) fn continued(s: &mut Interview) -> FederalResult {
    let mut d = s.draft.clone();
    d.finished_sections.push(TaxSection::Credits);
    supported(&d)
}
pub(in crate::tests) fn value_of(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount / 100)
}

#[test]
fn graduate_or_part_time_student_gets_lifetime_learning() {
    // Form 8863 Part II by hand: joint, MAGI $100,000 (below $160,000):
    //   you, a graduate student: tuition 4,000 + books from the school 500
    //   (other required books 300 do not count for lifetime learning) = 4,500
    //   your spouse, less than half-time: tuition 3,000
    //   line 10 = 7,500; line 11 = min(7,500, 10,000); line 18 = 20% = 1,500,
    //   nothing refundable.
    let (mut s, _) = at_the_credits_hub(complete(FederalStatus::Joint));
    let page = open_credits_topic(&mut s, TaxSection::Education);
    let people = education_value(&page).people.clone();
    let labels: Vec<_> = people.iter().map(|p| status_text(&p.label)).collect();
    assert_eq!(labels, vec!["You", "Your spouse"]);
    let graduate = StudentCredit {
        graduate: true,
        more: StudentMore {
            books_from_school: 50_000,
            other_books: 30_000,
            scholarship_in_income: ScholarshipInclusion::None,
            ..Default::default()
        },
        ..student(-1, 400_000)
    };
    let part_time = StudentCredit { half_time: false, ..student(-2, 300_000) };
    let request = save_education(
        &s.draft,
        EducationCredits { students: vec![graduate, part_time.clone()], people: people.clone() },
    );
    credits_hub(send_past_less_common(&mut s, request));
    let methods: Vec<_> =
        s.draft.credits.as_ref().unwrap().education.iter().map(|e| e.method).collect();
    assert_eq!(methods, vec![EducationChoice::LifetimeLearning, EducationChoice::LifetimeLearning]);
    let r = continued(&mut s);
    assert_eq!(value_of(&r, "education_credit"), 1_500);
    assert_eq!(value_of(&r, "refundable_education_credit"), 0);
    // The American opportunity credit already used up is lifetime learning
    // too, even for a half-time undergraduate; without that answer the same
    // row is the American opportunity credit.
    for (used_up, expected) in
        [(true, EducationChoice::LifetimeLearning), (false, EducationChoice::AmericanOpportunity)]
    {
        let row = StudentCredit {
            half_time: true,
            more: StudentMore {
                scholarship_in_income: ScholarshipInclusion::None,
                american_opportunity_used_up: used_up,
                ..Default::default()
            },
            ..part_time.clone()
        };
        let request = save_education(
            &s.draft,
            EducationCredits { students: vec![row], people: people.clone() },
        );
        credits_hub(send_past_less_common(&mut s, request));
        assert_eq!(s.draft.credits.as_ref().unwrap().education[0].method, expected);
    }
    // A catalog that no longer matches the return, or a student outside it,
    // is refused.
    let request = save_education(
        &s.draft,
        EducationCredits { students: vec![student(999, 100_000)], people: people.clone() },
    );
    error(current_source_refusal(
        "secondary-admission-1875",
        &mut s,
        request,
        "MESSAGE_A_STUDENT_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_STUDENT_AGAIN",
        Some("[value/students/0/student]"),
        SourceScope::Secondary(SecondaryCorrection::StudentSource),
    ));
    let request = save_education(
        &s.draft,
        EducationCredits { students: vec![], people: people[..1].to_vec() },
    );
    error(current_source_refusal(
        "secondary-admission-1876",
        &mut s,
        request,
        "MESSAGE_THE_PEOPLE_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
        None,
        SourceScope::Fieldless,
    ));
}

#[test]
fn removed_person_drops_their_education_row() {
    // Two children with education rows; removing the first on the people
    // page drops its row and moves the second's to its new catalog position.
    let s = household_return(m2_children());
    let (mut s, _) = at_the_credits_hub(s.draft.clone());
    let page = open_credits_topic(&mut s, TaxSection::Education);
    let people = education_value(&page).people.clone();
    assert_eq!(people.len(), 3);
    let first = people[1].id;
    let second = people[2].id;
    let request = save_education(
        &s.draft,
        EducationCredits {
            students: vec![student(first, 100_000), student(second, 200_000)],
            people,
        },
    );
    credits_hub(send_past_less_common(&mut s, request));
    let d = s.draft.clone();
    let mut shown = reopened_people(&mut s);
    shown.remove(0);
    let reply = send(&mut s, reviewing(save_children(&d, shown)));
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let credits = s.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education.len(), 1);
    assert_eq!(credits.education[0].expenses.tuition, 200_000);
    assert_eq!(credits.education[0].student, second);
    let page = open_credits_topic(&mut s, TaxSection::Education);
    assert_eq!(education_value(&page).students, vec![student(second, 200_000)]);
}

// M3-P13: the federal payments page.

// The page as a new return opens it: no payments, the IRS figures any
// penalty. The withholding shown is derived and ignored on save.
pub(in crate::tests) fn no_payments() -> Payments {
    Payments {
        credit_applied_next_year: 0,
        credit_recipient: None,
        credit_capacity: None,
        extension_payer: None,
        extension_refund_property: None,
        withheld: Some(Withheld { w2: 0, forms_1099: 0, other: 0, total: 0 }),
        estimated: vec![],
        extension: 0,
        penalty: PenaltyChoice::IrsFigures,
        next_year: NextYearEstimate::NotNeeded,
    }
}
// The payments page after the credits hub (M3-P13) opens complete; Next
// saves it unchanged and goes on.
// Past the other taxes page (M3-P15, saved empty) and the payments page
// (saved as it opens).
// Continues past the credits hub when the sequence shows it, then past the
// other taxes and payments pages when shown; sections with nothing for the
// return are passed by the service (M3-P19).
pub(in crate::tests) fn past_credits(s: &mut Interview, reply: Reply) -> Reply {
    // The credits walk asks More credits first (gate fixes batch A).
    let reply = past_less_common(s, reply);
    let reply = match reply {
        Reply::Response(hub @ P::Hub { page: PAGE_CREDITS, .. }) => send(s, go_on(&hub)),
        other => other,
    };
    past_payments(s, reply)
}
pub(in crate::tests) fn past_payments(s: &mut Interview, reply: Reply) -> Reply {
    // The other taxes page opens with "None of these" (gate fixes batch A).
    let reply = past_less_common(s, reply);
    match reply {
        Reply::Edit(page @ P::SavePayments { .. }) => send(s, page),
        other => other,
    }
}
pub(in crate::tests) fn save_payments(d: &FederalDraft, value: Payments) -> P {
    P::SavePayments {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_PAYMENTS,
        previous: PAGE_OTHER_TAXES,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        // An echoed list never widens what the service offers.
        allowed_prior_returns: vec![0, 1],
        allowed_farmer: vec![0, 1],
        allowed_farmer_next_year: vec![0, 1, 2, 3],
        allowed_refund_management: vec![],
        value,
    }
}
// Form 2210 with a $5,000.00 2025 tax on a $60,000.00 AGI, a full 2025
// year, even income and the given answers.
pub(in crate::tests) fn figure_with(
    prior_return: PriorReturn,
    withholding: WithholdingTiming,
    farmer: Farmer,
    waiver: Waiver,
) -> PenaltyChoice {
    PenaltyChoice::Figure {
        prior_tax: 500_000,
        prior_agi: Some(6_000_000),
        full_year: true,
        prior_return,
        withholding,
        income: IncomeTiming::Evenly,
        farmer,
        waiver,
    }
}
// Saves the payments page against the session's accepted revision.
pub(in crate::tests) fn submit(s: &mut Interview, value: Payments) -> Reply {
    let request = save_payments(&s.draft, value);
    send(s, request)
}
pub(in crate::tests) fn page_paid(date: u32, amount: i64) -> crate::InjuredDatedPayment {
    crate::InjuredDatedPayment::from_record(&paid(date, amount))
}
pub(in crate::tests) fn paid(date: u32, amount: i64) -> DatedPayment {
    DatedPayment { payer: crate::PaymentPayer::Joint, refund_property: None, date, amount }
}
// A saved return at its payments page: every earlier section is finished,
// so opening the interview resumes there.
pub(in crate::tests) fn at_the_payments_page(mut d: FederalDraft) -> (Interview, P) {
    d.finished_sections.retain(|s| *s != TaxSection::Payments);
    let mut s = Interview::new(TEST_USER, d);
    let Reply::Edit(page @ P::SavePayments { .. }) =
        send(&mut s, P::OpenInterview { user: TEST_USER })
    else {
        panic!("expected the payments page")
    };
    (s, page)
}
pub(in crate::tests) fn payments_value(page: &P) -> (&Payments, &[i32], &[i32], &[i32]) {
    let P::SavePayments {
        value,
        allowed_prior_returns,
        allowed_farmer,
        allowed_farmer_next_year,
        ..
    } = page
    else {
        panic!("expected the payments page, got {page:?}")
    };
    (value, allowed_prior_returns, allowed_farmer, allowed_farmer_next_year)
}
pub(in crate::tests) fn open_payments(s: &mut Interview) -> P {
    let request = navigate(&s.draft, TaxSection::Payments);
    let Reply::Edit(page @ P::SavePayments { .. }) = send(s, request) else {
        panic!("expected the payments page")
    };
    page
}

// Form 2210-F has one payment, due January 15, 2027, so a farmer or fisher
// (Yes for 2025 or Yes for 2026) has neither box D nor box C: the Payments
// client rule FARMER_EVEN_TIMING and its server twin, checked before the
// farmer set.
#[test]
fn farmers_must_keep_withholding_and_income_even() {
    assert!(APP_STRINGS.iter().any(|(key, _)| *key == "FARMER_EVEN_TIMING"));
    let figure = |farmer: Farmer, withholding: WithholdingTiming, income: IncomeTiming| Payments {
        penalty: PenaltyChoice::Figure {
            prior_tax: 500_000,
            prior_agi: Some(6_000_000),
            full_year: true,
            prior_return: PriorReturn::SameReturn,
            withholding,
            income,
            farmer,
            waiver: Waiver::None,
        },
        ..no_payments()
    };
    // The four periods total complete()'s $6,500.00 of withholding, so only
    // the farmer rule can refuse them.
    let by_period = WithholdingTiming::ByPeriod {
        first: 150_000,
        second: 150_000,
        third: 150_000,
        fourth: 200_000,
    };
    let yes = || Farmer::Yes { paid_by_march_1: false };
    // 2025 Yes (no farm on the return).
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    let before = s.draft.clone();
    for (withholding, income) in [
        (by_period.clone(), IncomeTiming::Evenly),
        (WithholdingTiming::Evenly, IncomeTiming::ByPeriod),
        (by_period.clone(), IncomeTiming::ByPeriod),
    ] {
        rejected_with(
            send(&mut s, save_payments(&before, figure(yes(), withholding, income))),
            "FARMER_EVEN_TIMING",
        );
        assert_eq!(s.draft, before);
    }
    accepted(&submit(&mut s, figure(yes(), WithholdingTiming::Evenly, IncomeTiming::Evenly)));
    // Not a farmer: both are accepted.
    accepted(&submit(&mut s, figure(Farmer::No, by_period.clone(), IncomeTiming::Evenly)));
    assert!(matches!(
        submit(&mut s, figure(Farmer::No, WithholdingTiming::Evenly, IncomeTiming::ByPeriod)),
        Reply::Edit(P::SaveIncomeByPeriod { .. })
    ));

    // 2026 Yes, as this return shows it ($120,000 of farm gross income of
    // $180,000 is exactly two-thirds).
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(120_000, 120_000)));
    let (mut s, _) = at_the_payments_page(d);
    let before = s.draft.clone();
    let this_return = || Farmer::ThisReturn { paid_by_march_1: false };
    let next_year = NextYearEstimate::SameAsThisReturn {
        farmer_fisherman_next_year: FarmerNextYear::NotNeeded,
    };
    for (withholding, income) in [
        (by_period.clone(), IncomeTiming::Evenly),
        (WithholdingTiming::Evenly, IncomeTiming::ByPeriod),
    ] {
        let value =
            Payments { next_year: next_year.clone(), ..figure(this_return(), withholding, income) };
        rejected_with(send(&mut s, save_payments(&before, value)), "FARMER_EVEN_TIMING");
        assert_eq!(s.draft, before);
    }
    accepted(&submit(
        &mut s,
        Payments {
            next_year,
            ..figure(this_return(), WithholdingTiming::Evenly, IncomeTiming::Evenly)
        },
    ));
}

// M3-P15: every situation finds its hub.

// Opens a section through the review's Go to.
pub(in crate::tests) fn go_to_from_review(s: &mut Interview, section: TaxSection) -> Reply {
    let request = navigate(&s.draft, section);
    send(s, request)
}

#[test]
fn every_situation_has_one_destination() {
    use RareSituation as R;
    use TaxSection as T;
    // The interview map's cross-check table ("Where every situation goes"),
    // after its renames, splits and new variants, in the map's order.
    let table = [
        (R::RailroadRepresentative, T::Wages, "Wages"),
        (R::RemicResidual, T::MoreIncome, "REMIC residual interest"),
        (R::InstallmentSale, T::MoreIncome, "Investment sales"),
        (R::Straddles, T::MoreIncome, "Investment sales"),
        (R::SpecialDepreciation, T::MoreIncome, "Self-employment"),
        (R::TipsNotOnW2, T::MoreIncome, "Self-employment"),
        (R::AssetDisposed, T::MoreIncome, "Business or rental property sold"),
        (R::LikeKindExchange, T::MoreIncome, "Business or rental property sold"),
        (R::TaxTreaty, T::MoreIncome, "Tax treaties"),
        (R::Chapter11Debtor, T::MoreIncome, "Chapter 11 bankruptcy"),
        (R::ForeignCorporation, T::MoreIncome, "Foreign corporations and PFICs"),
        (R::Casualty, T::MoreItemized, "Casualty and theft of business or investment property"),
    ];
    // The table is the enum: every variant once, in declaration order.
    assert_eq!(table.iter().map(|(s, _, _)| *s).collect::<Vec<_>>(), RareSituation::ALL.to_vec());
    for (situation, section, topic) in table {
        assert_eq!(situation.destination(), (section, topic), "{situation:?}");
    }
    // Each page's offered set is its destinations; together they are every
    // variant once (u5-credits made the home kerosene credit
    // and Schedule H data items; T2-45 added FarmRental to the income page;
    // T2-46 removed EicAfterDisallowance from More credits, where Form 8862
    // is now a calculated record; T2-27p removed Marketplace, T2-44p
    // Gambling, T2-45p Farm and T2-31p PassThrough; u5-adjust
    // PublicSafetyPremiums and AlimonyPaid; u5-losses NetOperatingLoss;
    // T3-deduct CarLoanInterest; gate fixes batch A moved the five
    // whole-return items to the checklist; storage format 88 moved the
    // adjustment, credit and other-tax items to their page enums and
    // removed DisasterDistribution and OtherTax::ParentsReturn; Form 8814
    // removed ChildInvestmentIncome; Form 4972 removed LumpSumAveraging; unit B2
    // removed FarmRental; unit B6b removed ReliefPayments; unit B6a removed
    // AccountDistributions).
    let mut offered: Vec<i32> = Vec::new();
    for section in [T::Wages, T::MoreIncome, T::MoreItemized] {
        let allowed = RareSituation::allowed(section);
        assert!(!allowed.is_empty(), "{section:?}");
        offered.extend(allowed);
    }
    offered.sort_unstable();
    assert_eq!(offered, (0..RareSituation::ALL.len() as i32).collect::<Vec<i32>>());
    // The page enums' refusal kinds, each on its own page and topic.
    let mut page_items = vec![
        (MoreAdjustment::OtherAdjustments.refused(), T::MoreAdjustments, "Other adjustments"),
        (OtherTax::AlternativeMinimumTax.refused(), T::OtherTaxes, "Alternative minimum tax items"),
        (
            OtherTax::HousingCreditRecapture.refused(),
            T::OtherTaxes,
            "Credit recapture and look-back interest",
        ),
        (OtherTax::Section965.refused(), T::OtherTaxes, "Deferred section 965 tax"),
    ];
    page_items.extend(RareAdditionalTax::ALL.map(|kind| {
        let (topic, _, _, _, _) = kind.refusal_info();
        (OtherTax::OtherAdditionalTaxes { kind }.refused(), T::OtherTaxes, topic)
    }));
    assert_eq!(page_items.len(), super::every_refusal_item().len() - RareSituation::ALL.len());
    for (refusal, section, topic) in page_items {
        let refusal = refusal.expect("a refusal kind");
        assert_eq!((refusal.section, refusal.topic), (section, topic));
    }
    // No variant carries data: each encodes as its union index alone.
    for situation in RareSituation::ALL {
        assert_eq!(situation.to_avro().len(), 1, "{situation:?}");
    }
}

#[test]
fn less_common_income_saves_only_its_hubs_situations() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // A situation of another page, saved first.
    accepted(&save_other_taxes(&mut s, vec![OtherTax::Section965]));
    // The income hub's less-common page is a topic of the hub (page 3),
    // offering only the income variants.
    let (page, allowed, _) = open_situations(&mut s, TaxSection::MoreIncome);
    assert_eq!(page, TaxSection::Income as i32);
    assert_eq!(allowed, RareSituation::allowed(TaxSection::MoreIncome));
    // The income variants are the first ones (T3-982 removed CanceledDebt,
    // u5-adjust PublicSafetyPremiums, u5-losses NetOperatingLoss, u6-home
    // HomeSale, storage format 88 DisasterDistribution, Form 8814
    // ChildInvestmentIncome, Form 4972 LumpSumAveraging, unit B2 FarmRental,
    // unit B6b ReliefPayments, unit B6a AccountDistributions).
    assert_eq!(allowed, (1..=allowed.len() as i32).collect::<Vec<i32>>());
    assert!(
        RareSituation::ALL[allowed.len() + 1..]
            .iter()
            .all(|situation| situation.destination().0 != TaxSection::MoreIncome)
    );
    accepted(&save_situations(
        &mut s,
        TaxSection::MoreIncome,
        vec![RareSituation::TaxTreaty, RareSituation::SpecialDepreciation],
    ));
    // Stored in declaration order; the other page's item stays its own.
    assert_eq!(
        s.draft.situations,
        vec![RareSituation::SpecialDepreciation, RareSituation::TaxTreaty]
    );
    assert_eq!(s.draft.tax_refusals, vec![OtherTaxRefusal::Section965]);
    // The hub's row names what was chosen.
    let Reply::Response(hub @ P::Hub { .. }) = go_to_from_review(&mut s, TaxSection::Income) else {
        panic!("expected the income hub")
    };
    assert!(hub_rows(&hub).contains(&row_of(
        "Less common income",
        "Self-employment, Tax treaties — not calculated yet",
        TaxSection::MoreIncome
    )));
    let P::Hub { allowed_sections, .. } = &hub else { unreachable!() };
    assert!(allowed_sections.contains(&(TaxSection::MoreIncome as i32)));
    // Another page's variant is refused on this page.
    let before = s.draft.clone();
    let request = situations_request(&mut s, TaxSection::MoreIncome, vec![RareSituation::Casualty]);
    error(current_source_refusal(
        "secondary-admission-2479",
        &mut s,
        request,
        "MESSAGE_THIS_SITUATION_BELONGS_TO_ANOTHER_SECTION",
        Some("[value/situations/0/situation]"),
        SourceScope::Secondary(SecondaryCorrection::SituationChoice),
    ));
    let request =
        situations_request(&mut s, TaxSection::MoreIncome, vec![RareSituation::RemicResidual; 2]);
    current_source_refusal(
        "secondary-situation-duplicate",
        &mut s,
        request,
        "MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE",
        Some("[value/situations/1/situation]"),
        SourceScope::Secondary(SecondaryCorrection::SituationChoice),
    );
    assert_eq!(s.draft, before);
    // Clearing the page clears only its own situations.
    accepted(&save_situations(&mut s, TaxSection::MoreIncome, vec![]));
    assert!(s.draft.situations.is_empty());
    assert_eq!(s.draft.tax_refusals, vec![OtherTaxRefusal::Section965]);
    // Saved from the hub, the page returns to the hub.
    let d = s.draft.clone();
    let reply = send(
        &mut s,
        P::SaveSituations {
            user: TEST_USER,
            topic: TaxSection::MoreIncome,
            allowed_situations: RareSituation::allowed(TaxSection::MoreIncome),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: TaxSection::Income as i32,
            previous: TaxSection::Income as i32,
            review: false,
            estimate: vec![],
            warnings: vec![],
            progress: vec![],
            value: Situations {
                situations: vec![SituationItem { situation: RareSituation::TaxTreaty }],
            },
        },
    );
    assert!(matches!(reply, Reply::Response(P::Hub { page: PAGE_INCOME, .. })));
}

// A Form 3921 (T2-iso): box 3 $10.00, box 4 $210.00, box 5 1,000 shares.
pub(in crate::tests) fn form_3921(more: Form3921More) -> Form3921 {
    Form3921 {
        transferor: "Synthetic Employer Inc".into(),
        granted: 20240315,
        exercised: 20260310,
        exercise_price: 1_000,
        market_price: 21_000,
        shares: 1000.0,
        corporation: "".into(),
        more,
    }
}

#[test]
fn final_return_page_maps_the_stored_final_return_both_ways() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut you = filer(&s.draft.people.as_ref().unwrap().taxpayer);
    assert_eq!(you.more, MoreAboutYou::Nothing);
    you.more = MoreAboutYou::Died {
        date: 20260630,
        estate_medical: 1_200_000,
        bonds: None,
        ordinary: false,
    };
    let request = save_you(&s.draft.clone(), you.clone());
    accepted(&send(&mut s, request));
    let stored = s.draft.people.as_ref().unwrap().taxpayer.died.clone().unwrap();
    assert_eq!(
        stored,
        FinalReturn { bonds: None, estate_medical: 1_200_000, date: 20260630, ordinary: false }
    );
    assert!(s.draft.situations.is_empty());
    // The return computes, with the notice that the death adjustments are left out.
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT"));
    // It reopens on the page, and "Nothing else" clears it.
    assert_eq!(filer(&restart(s.draft.clone()).draft.people.unwrap().taxpayer).more, you.more);
    you.more = MoreAboutYou::Nothing;
    let request = save_you(&s.draft.clone(), you);
    accepted(&send(&mut s, request));
    assert_eq!(supported(&s.draft).tax, supported(&complete(FederalStatus::Single)).tax);
}

#[test]
fn death_date_rules_are_checked_on_save_with_the_client_keys() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let birth = s.draft.people.as_ref().unwrap().taxpayer.birth_date;
    for (date, key) in [
        (20251231, "DEATH_DATE_OUTSIDE_2026"),
        (20270101, "DEATH_DATE_OUTSIDE_2026"),
        (20260230, "DEATH_DATE_OUTSIDE_2026"),
    ] {
        let mut you = filer(&s.draft.people.as_ref().unwrap().taxpayer);
        you.more = died_more(date);
        let request = save_you(&s.draft.clone(), you);
        rejected_with(send(&mut s, request), key);
    }
    let mut you = filer(&s.draft.people.as_ref().unwrap().taxpayer);
    you.birth_date = 20260615;
    you.more = died_more(20260614);
    let request = save_you(&s.draft.clone(), you);
    rejected_with(send(&mut s, request), "DEATH_BEFORE_BIRTH");
    assert!(birth < 20260101);
}

#[test]
fn person_fold_maps_citizenship_and_joint_return() {
    let mut s = supporting(FederalStatus::Single);
    let mut person = row("Ava", "Sample", 20180214);
    person.facts = PersonFacts::ABROAD.0 | PersonFacts::FILED_JOINTLY.0;
    let request = save_children(&s.draft.clone(), vec![person.clone()]);
    accepted(&send(&mut s, request));
    let record = &s.draft.dependents[0];
    assert!(!record.us_citizen_resident());
    assert!(record.joint_return_nonrefund());
    assert_eq!(record.competing, CompetingClaim::NoOne);
    // The two tests are calculated, not refused.
    assert!(estimate(&s.draft).issues.iter().all(|i| !i.message.contains("Form 8332")));
    let reopened = reopened_people(&mut s);
    assert_eq!(reopened[0].facts, person.facts);
    // The competing claim is stored on the person and computed: the other
    // person's claim takes none of the five benefits for this child.
    person.facts = PersonFacts::NONE.0;
    person.competing = CompetingClaim::Other;
    let request = save_children(&s.draft.clone(), vec![person.clone()]);
    accepted(&send(&mut s, request));
    let record = &s.draft.dependents[0];
    assert_eq!(
        (record.us_citizen_resident(), record.joint_return_nonrefund(), record.competing),
        (true, false, CompetingClaim::Other)
    );
    let r = estimate(&s.draft);
    assert!(noted(&r, "NOTE_COMPETING_CLAIM_OTHER_PERSON_CLAIMS"));
    assert!(r.issues.iter().all(|i| !i.message.contains("tie-breaker")));
    assert_eq!(reopened_people(&mut s)[0].competing, CompetingClaim::Other);
    // The default is "None of these": nothing is added to the M2 path.
    assert_eq!(row("Ava", "Sample", 20180214).facts, PersonFacts::NONE.0);
}

#[test]
fn refusals_name_their_topic() {
    let d = complete(FederalStatus::Single);
    for situation in super::every_refusal_item() {
        let refusal = situation.refused();
        let (section, topic, name) = (refusal.section, refusal.topic, refusal.lines);
        let mut refused = d.clone();
        situation.choose(&mut refused);
        let r = estimate(&refused);
        // The issue names the page it is chosen on.
        assert!(
            r.issues.iter().any(|i| i.section == section_name(section) && i.message.contains(name)),
            "{situation:?}: {r:?}"
        );
        // A hub's less-common row names the topic.
        if matches!(
            section,
            TaxSection::MoreIncome | TaxSection::MoreAdjustments | TaxSection::MoreItemized
        ) {
            let mut s = Interview::new(TEST_USER, refused.clone());
            let hub = match section {
                TaxSection::MoreIncome => TaxSection::Income,
                TaxSection::MoreAdjustments => TaxSection::Adjustments,
                _ => {
                    s.draft.deductions.as_mut().unwrap().choice =
                        DeductionSelection::Itemized { facts: empty_facts() };
                    s.draft.finished_sections.push(TaxSection::Itemized);
                    TaxSection::Itemized
                }
            };
            let Reply::Response(hub @ P::Hub { .. }) = send(&mut s, navigate(&refused, hub)) else {
                panic!("{situation:?}: expected its hub")
            };
            assert!(
                hub_rows(&hub).contains(&row_of(
                    section_name(section),
                    &alloc::format!("{topic} — not calculated yet"),
                    section
                )),
                "{situation:?}: {hub:?}"
            );
        }
    }
    // The P12 finding: the residual Form 3800 credits are refusal kinds of
    // More credits, stored as chosen (Form 8862 was one until
    // T2-46 made it a calculated record).
    let mut s = Interview::new(TEST_USER, d.clone());
    let Reply::Edit(P::SaveMoreCredits { value, .. }) =
        send(&mut s, navigate(&d, TaxSection::MoreCredits))
    else {
        panic!("expected the more credits page")
    };
    let request = P::SaveMoreCredits {
        initial_carbon_oxide_origins: vec![],
        initial_renewable_facility_origins: Vec::new(),
        clean_vehicle_properties: vec![],
        businesses: vec![],
        farms: vec![],
        rentals: vec![],
        research_businesses: vec![],
        entered_credits: vec![],
        carryback_credits: vec![],
        allowed_carry_filers: vec![],
        initial_credit_elections: vec![],
        initial_research_choices: vec![],
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_CREDITS,
        previous: PAGE_CREDITS,
        review: true,
        estimate: None,
        warnings: vec![],
        progress: None,
        allowed_owners: vec![Owner::Taxpayer as i32],
        foreign_businesses: vec![],
        capital_sales: vec![],
        foreign_situations: vec![],
        mortgage_loans: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value: MoreCredits {
            credits: value.credits.into_iter().chain([MoreCredit::OtherCredit]).collect(),

            credit_income_sources: Vec::new(),
        },
    };
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.credit_refusals, vec![MoreCreditRefusal::OtherCredit]);
    let r = supported(&s.draft);
    assert!(r.review_notes.iter().any(|n| n.display.key == "WARN_OTHER_CREDIT_LEFT_OUT"));
    assert_eq!(r.tax, supported(&d).tax);
    let Reply::Edit(P::SaveMoreCredits { value, .. }) =
        go_to_from_review(&mut s, TaxSection::MoreCredits)
    else {
        panic!("expected the more credits page")
    };
    assert_eq!(value.credits, vec![MoreCredit::OtherCredit]);
}

#[test]
fn m1_and_m2_fixtures_keep_12_and_20_fields() {
    // The fixtures retain 12/20 source fields. Filer/dependent folds keep
    // their defaults; deductions and other taxes are saved as opened.
    // The checklist guides Wages, then Interest when selected, followed
    // by the income summary's Continue. Later less-common pages and
    // summaries have their normal Next/Continue; every return asks People.
    // Both paths send 14 requests, including the final refund election page.
    // Checklist ticks and
    // navigation are structure taps rather than source fields.
    // The Care editor adds a provider/identification workflow step;
    // its actual fields have separate tests.
    let m2 = include_str!("../../../tests/fixtures/returns/m2-hoh-two-children.json");
    let fixture: serde_json::Value = serde_json::from_str(m2).unwrap();
    assert_eq!(fixture["fields_touched"], 20);
    for (name, raw, taps) in [
        ("m1-single-w2", include_str!("../../../tests/fixtures/returns/m1-single-w2.json"), 14),
        ("m2-hoh-two-children", m2, 14),
    ] {
        let (mut s, result, sent) = enter_through_the_hubs(name, raw);
        assert_fixture_lines(raw, &result);
        assert_eq!(sent.len(), taps, "{name}");
        for request in &sent {
            match request {
                P::SavePeople { you, .. } => assert_eq!(you.more, MoreAboutYou::Nothing),
                P::SaveChildren { dependents, .. } => {
                    assert!(dependents.people.iter().all(|p| p.facts == PersonFacts::NONE.0))
                }
                P::SaveDeductions { value, .. } => assert!(value.car_loans.is_empty()),
                P::SaveChecklist { value, .. } => assert_eq!(
                    *value,
                    income_ticks(if name == "m1-single-w2" {
                        IncomeTopics::WAGES.0 | IncomeTopics::INTEREST_DIVIDENDS.0
                    } else {
                        IncomeTopics::WAGES.0
                    })
                ),
                P::SaveSituations { topic, value, .. } => {
                    assert_eq!(*topic, TaxSection::MoreIncome);
                    assert!(value.situations.is_empty());
                }
                _ => {}
            }
        }
        assert!(s.draft.situations.is_empty());
        assert!(s.draft.finished_sections.contains(&TaxSection::OtherTaxes));
        // M3-P16: nothing remains after the fixture's entries, so the review
        // offers Finish; the review adds no page, so the taps are unchanged.
        let (items, _, actions, _) = review_state(&mut s);
        assert!(items.is_empty(), "{name}: {items:?}");
        assert!(actions.contains(&FINISH), "{name}");
    }
}

// M3-P16: the review's remaining items.

pub(in crate::tests) fn review_state(
    s: &mut Interview,
) -> (Vec<ReviewItem>, TaxSection, Vec<i32>, Vec<ReviewItem>) {
    match send(s, P::ReviewAccepted { user: TEST_USER }) {
        Reply::Response(P::Review {
            remaining_items, suggested, allowed_actions, answers, ..
        }) => (remaining_items, suggested, allowed_actions, answers),
        other => panic!("expected the review, got {other:?}"),
    }
}
pub(in crate::tests) fn items_of(
    s: &mut Interview,
) -> Vec<(alloc::string::String, alloc::string::String, TaxSection)> {
    review_state(s)
        .0
        .into_iter()
        .map(|i| (status_text(&i.item), status_text(&i.status), i.section))
        .collect()
}
fn has_item(
    items: &[(alloc::string::String, alloc::string::String, TaxSection)],
    page: &str,
    section: TaxSection,
    text: &str,
) -> bool {
    items.iter().any(|(item, status, s)| item == page && *s == section && status.contains(text))
}
// A single return whose Interest and dividends topic holds these documents.
fn with_interest(documents: Vec<IncomeDocumentItem>) -> Interview {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(&s.draft, TaxSection::InterestDividends, documents);
    accepted(&send(&mut s, request));
    s
}
const FINISH: i32 = libertas_macros::variant_index!(P::StartFinish) as i32;

#[test]
fn each_issue_points_at_its_page() {
    // A newly entered box10 source requires its actual plan maximum on Care.
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let mut w = w2(6_000_000);
    w.more = W2More {
        payroll: W2Payroll::SocialSecurity,
        care_benefits: 40_000,
        statutory: true,
        ..Default::default()
    };
    accepted(&send(&mut s, save_documents(&original, vec![document(0, IncomeDocument::W2(w))])));
    let items = items_of(&mut s);
    assert!(
        has_item(&items, "Child and dependent care", TaxSection::Care, "combined maximum"),
        "{items:?}"
    );
    // An unsupported 1099-R code, stored on the retirement page.
    let mut s = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    accepted(&send(&mut s, save_documents(&original, vec![document(0, IncomeDocument::R1099(r))])));
    let items = items_of(&mut s);
    assert!(
        has_item(
            &items,
            "Retirement and Social Security",
            TaxSection::Retirement,
            "Form 1099-R code 5"
        ),
        "{items:?}"
    );
    // The household check against page 1: a single filer who is married.
    let mut d = complete(FederalStatus::Single);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    let items = items_of(&mut Interview::new(TEST_USER, d));
    assert!(
        has_item(&items, "Your 2026 return", TaxSection::Setup, "household facts do not establish"),
        "{items:?}"
    );
    // A later answer makes the mortgage balance method necessary: box 2
    // $500,000 fits the $750,000 limit of a single return but not the
    // $375,000 of a separate one (Pub 936 Table 1 line 11).
    let mut s = mortgage_loan_page(at_the_deductions_page(FederalStatus::Single).0);
    let request = save_mortgages(
        &s.draft,
        Mortgages {
            loans: vec![first_lender_mortgage(2_000_000, 50_000_000, AverageBalance::WithinLimit)],
        },
    );
    accepted(&send(&mut s, request));
    assert!(!items_of(&mut s).iter().any(|(_, _, section)| *section == TaxSection::Mortgage));
    let mut d = s.draft.clone();
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Separate);
    let items = items_of(&mut Interview::new(TEST_USER, d));
    assert!(
        has_item(
            &items,
            "Home mortgage interest",
            TaxSection::Mortgage,
            "choose an average-balance method"
        ),
        "{items:?}"
    );
}

#[test]
fn refused_situations_name_their_topic() {
    let mut d = complete(FederalStatus::Single);
    d.situations = vec![RareSituation::TaxTreaty];
    d.tax_refusals = vec![OtherTaxRefusal::Section965];
    d.checklist.whole_return = WholeReturnTopics::DUAL_STATUS.0;
    let items = items_of(&mut Interview::new(TEST_USER, d));
    // Each situation once, named by the page that holds it, in sequence
    // order; the estimate's own copy of the refusal is not repeated. The
    // whole-return items are the checklist's (gate fixes batch A).
    assert_eq!(
        items,
        vec![
            (
                "Your 2026 at a glance".into(),
                "Part-year residency, dual-status and U.S. territory returns (Form 1040 line 12c, Form 1040-NR, Forms 4563 and 8689) is not calculated yet".into(),
                TaxSection::Checklist
            ),
            (
                "Less common income".into(),
                alloc::format!(
                    "{} is not calculated yet",
                    RareSituation::TaxTreaty.refused().lines
                ),
                TaxSection::MoreIncome
            ),
            (
                "Other taxes".into(),
                alloc::format!(
                    "{} is not calculated yet",
                    OtherTax::Section965.refused().unwrap().lines
                ),
                TaxSection::OtherTaxes
            ),
        ]
    );
}

#[test]
fn hub_attention_and_review_item_share_one_text() {
    // $2,000 of interest from a 1099-INT saved without a payer name.
    let mut s = with_interest(vec![interest(200_000)]);
    let (items, ..) = review_state(&mut s);
    let item = items.iter().find(|i| i.section == TaxSection::InterestDividends).unwrap();
    assert_eq!(status_text(&item.item), "Interest and dividends");
    let request = go_to(&s.draft.clone(), PAGE_REVIEW, TaxSection::Income);
    let Reply::Response(hub @ P::Hub { .. }) = send(&mut s, request) else {
        panic!("expected the income hub")
    };
    assert!(hub_rows(&hub).contains(&row_of(
        "Interest and dividends",
        &alloc::format!("Needs attention: {}", status_text(&item.status)),
        TaxSection::InterestDividends
    )));
    // The adjustments hub's HSA row and its review item, too.
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 100_000);
    let hub = open_adjustments_hub(&mut s);
    let (items, ..) = review_state(&mut s);
    let item = items.iter().find(|i| i.section == TaxSection::Hsa).unwrap();
    assert_eq!(
        hub_rows(&hub)[0].1,
        alloc::format!("Needs attention: {}", status_text(&item.status))
    );
}

#[test]
fn finish_is_offered_only_with_nothing_remaining() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
    // A payer name the return now needs: the estimate is still available,
    // but Finish is neither offered nor accepted.
    let mut s = with_interest(vec![interest(200_000)]);
    assert_eq!(estimate(&s.draft).state, ResultState::EstimatedForSupportedScope);
    let (items, _, actions, _) = review_state(&mut s);
    assert_eq!(items.len(), 1);
    assert!(!actions.contains(&FINISH));
    let d = s.draft.clone();
    let finish = P::StartFinish { user: TEST_USER, cookie: d.cookie.clone(), revision: d.revision };
    assert!(matches!(send(&mut s, finish), Reply::Response(P::Problem { .. })));
    // Naming the payer clears the list and offers Finish again.
    let mut s = with_interest(vec![document(
        0,
        IncomeDocument::Int1099(native_int1099("Synthetic Bank".into(), 200_000)),
    )]);
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
}

#[test]
fn review_offers_resume_for_an_unfinished_interview_step() {
    let resume = libertas_macros::variant_index!(P::OpenInterview) as i32;
    let (mut s, _) = at_the_income_hub();
    let (items, _, actions, _) = review_state(&mut s);
    assert!(!items.is_empty());
    assert!(actions.contains(&resume));
    assert!(matches!(
        send(&mut s, P::OpenInterview { user: TEST_USER }),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::Wages, review: false, .. })
    ));

    // A missing spouse belongs to the open filers section even after the
    // other interview sections have been answered.
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse = None.into();
    let mut s = Interview::new(TEST_USER, d);
    let (_, _, actions, _) = review_state(&mut s);
    assert!(actions.contains(&resume));
    assert!(matches!(
        send(&mut s, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::BeginSpouse { review: false, .. })
    ));
}

#[test]
fn review_hides_resume_when_the_interview_walk_is_complete() {
    let resume = libertas_macros::variant_index!(P::OpenInterview) as i32;
    for (mut s, has_issues) in [
        (Interview::new(TEST_USER, complete(FederalStatus::Single)), false),
        (with_interest(vec![interest(200_000)]), true),
    ] {
        let original = s.draft.clone();
        let (items, _, actions, _) = review_state(&mut s);
        assert_eq!(!items.is_empty(), has_issues);
        assert!(!actions.contains(&resume));
        assert_eq!(actions.contains(&FINISH), !has_issues);
        // Existing endpoint entry still accepts OpenInterview, but it only
        // returns Review, so offering it again on Review would be redundant.
        assert!(matches!(
            send(&mut s, P::OpenInterview { user: TEST_USER }),
            Reply::Response(P::Review { .. })
        ));
        assert_eq!(s.draft, original);
    }
}

#[test]
fn review_hides_resume_after_finishing() {
    let mut d = complete(FederalStatus::Single);
    d.finished = true;
    let mut s = Interview::new(TEST_USER, d.clone());
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty());
    assert!(!actions.contains(&(libertas_macros::variant_index!(P::OpenInterview) as i32)));
    assert!(actions.contains(&FINISH));
    assert!(matches!(
        send(&mut s, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(s.draft, d);
    assert!(!s.dirty);
}

#[test]
fn review_suggests_the_first_item() {
    // A unsupported 1099-R code5 (Retirement) and a whole-return situation: the Retirement item
    // comes first in sequence order and is the suggestion.
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    accepted(&send(&mut s, save_documents(&original, vec![document(0, IncomeDocument::R1099(r))])));
    let mut d = s.draft.clone();
    d.tax_refusals = vec![OtherTaxRefusal::Section965];
    let mut s = Interview::new(TEST_USER, d);
    let (items, suggested, ..) = review_state(&mut s);
    assert_eq!(items[0].section, TaxSection::Retirement);
    assert_eq!(items.last().unwrap().section, TaxSection::OtherTaxes);
    assert_eq!(suggested, TaxSection::Retirement);
    // Go to opens it: the suggestion is an allowed section.
    let request = go_to(&s.draft.clone(), PAGE_REVIEW, suggested);
    assert!(matches!(
        send(&mut s, request),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::Retirement, .. })
    ));
}

#[test]
fn skipped_sections_read_nothing_entered() {
    // A return with no documents, adjustments, credits, other taxes or
    // payments: each passed section reads "Nothing entered".
    let d = empty_return(FederalStatus::Single);
    let (_, _, _, answers) = review_state(&mut Interview::new(TEST_USER, d));
    let status = |section: TaxSection| {
        answers.iter().find(|a| a.section == section).map(|a| status_text(&a.status)).unwrap()
    };
    for section in [
        TaxSection::Income,
        TaxSection::Adjustments,
        TaxSection::Credits,
        TaxSection::OtherTaxes,
        TaxSection::Payments,
    ] {
        assert_eq!(status(section), "Nothing entered", "{section:?}");
    }
    // A section not yet passed still reads "Not answered yet".
    let mut d = complete(FederalStatus::Single);
    d.finished_sections.retain(|s| *s != TaxSection::Payments);
    let (_, _, _, answers) = review_state(&mut Interview::new(TEST_USER, d));
    let payments = answers.iter().find(|a| a.section == TaxSection::Payments).unwrap();
    assert_eq!(status_text(&payments.status), "Not answered yet");
}

#[test]
fn payer_names_are_asked_only_above_1500() {
    // Schedule B is required only above $1,500 of taxable interest or of
    // ordinary dividends (Schedule B instructions; the estimate emits the
    // Schedule B line only then).
    let payer_item = |s: &mut Interview| {
        items_of(s).into_iter().any(|(_, status, section)| {
            section == TaxSection::InterestDividends
                && status == crate::interview::resource_text("REVIEW_PAYER_NAMES_NEEDED")
        })
    };
    // Exactly $1,500 of interest: no Schedule B, no payer name needed.
    assert!(!payer_item(&mut with_interest(vec![interest(150_000)])));
    // $1,501: Schedule B lists every payer, so an unnamed 1099-INT is an item.
    assert!(payer_item(&mut with_interest(vec![interest(150_100)])));
    // $1,501 from a named payer: nothing left.
    assert!(!payer_item(&mut with_interest(vec![document(
        0,
        IncomeDocument::Int1099(native_int1099("Synthetic Bank".into(), 150_100)),
    )])));
    // Dividends follow their own $1,500 test: $1,501 of box 1a unnamed.
    let dividend =
        |ordinary| document(0, IncomeDocument::Div1099(native_div1099("".into(), ordinary, 0)));
    assert!(!payer_item(&mut with_interest(vec![dividend(150_000)])));
    assert!(payer_item(&mut with_interest(vec![dividend(150_100)])));
}

// M3-P18: the opening checklist, "Your 2026 at a glance".

// A new single return saved through your page: the reply is the checklist.
fn at_the_checklist() -> (Interview, Reply) {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m3-checklist".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let request = save_you(&s.draft, adult("Casey", "Sample", 19850315));
    let reply = send(&mut s, request);
    let reply = past_people(&mut s, reply);
    (s, reply)
}
pub(in crate::tests) fn checklist_of(reply: &Reply) -> Checklist {
    match reply {
        Reply::Edit(P::SaveChecklist { value, .. }) => *value,
        other => panic!("expected the checklist, got {other:?}"),
    }
}
fn income_hub(reply: Reply) -> P {
    match reply {
        Reply::Response(hub @ P::Hub { page: PAGE_INCOME, .. }) => hub,
        other => panic!("expected the income hub, got {other:?}"),
    }
}
pub(in crate::tests) fn suggestion(hub: &P) -> TaxSection {
    let P::Hub { suggested, .. } = hub else { panic!("expected a hub, got {hub:?}") };
    *suggested
}
// A complete single return (one W-2) whose income hub is the first open
// section, with these planned ticks.
fn planning(planned: Checklist) -> Interview {
    let mut d = complete(FederalStatus::Single);
    d.finished_sections = vec![TaxSection::Children, TaxSection::Checklist];
    d.checklist = planned;
    Interview::new(TEST_USER, d)
}

#[test]
fn checklist_follows_the_people_pages_and_opens_with_nothing_ticked() {
    let (s, reply) = at_the_checklist();
    // The page after the people page, opened by EditRequest complete:
    // nothing is ticked ("None of these"), Back returns to the people page.
    let Reply::Edit(P::SaveChecklist { page, previous, review, value, progress, .. }) = &reply
    else {
        panic!("expected the checklist, got {reply:?}")
    };
    assert_eq!((*page, *previous, *review), (PAGE_CHECKLIST, PAGE_CHILDREN, false));
    assert_eq!(*value, Checklist::NONE);
    assert!(shown(progress).contains("Your return: "), "{}", shown(progress));
    // Resume stops at the unanswered checklist; manual navigation can already
    // open income once the filers are known.
    let mut resumed = restart(s.draft.clone());
    assert!(matches!(
        send(&mut resumed, P::OpenInterview { user: TEST_USER }),
        Reply::Edit(P::SaveChecklist { .. })
    ));
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::Income);
    assert!(matches!(
        send(&mut resumed, request),
        Reply::Response(P::Hub { page: PAGE_INCOME, .. })
    ));
    // Next with nothing ticked records the section; each section that
    // always asks opens its less-common page with "None of these" (gate
    // fixes batch A), every other later section has nothing for this return
    // and is recorded and not sent, and the walk reaches the review, where
    // each reads "Nothing entered" and opens its page. The income hub
    // opened from there suggests continuing; the review row reads "None of
    // these".
    let mut s = s;
    let reply = send(&mut s, tick(reply, Checklist::NONE));
    assert!(matches!(
        reply,
        Reply::Response(P::BeginSituations { topic: TaxSection::MoreIncome, .. })
    ));
    walk_on_to_review(&mut s, reply);
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    assert_eq!(told_row(&hub).1, "None of these");
    // Amended for M3-N1: every topic and Continue are rows.
    assert_eq!(hub_rows(&hub), hub_tail(&ALL_INCOME_TOPICS, TaxSection::MoreIncome));
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::MoreIncome,
            TaxSection::Income,
            TaxSection::MoreAdjustments,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::MoreCredits,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
        ]
    );
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    let (_, _, _, answers) = review_state(&mut s);
    let row = answers.iter().find(|r| r.section == TaxSection::Checklist).unwrap();
    assert_eq!(
        (status_text(&row.item).as_str(), status_text(&row.status).as_str()),
        ("Your 2026 at a glance", "None of these")
    );
}

#[test]
fn ticks_become_planned_topics() {
    let (mut s, reply) = at_the_checklist();
    let ticks = Checklist {
        income: IncomeTopics::WAGES.0 | IncomeTopics::RETIREMENT.0,
        payments: PaymentTopics::EXTENSION.0,
        ..Checklist::NONE
    };
    // M3-P19: the walk opens the first planned topic; its Back opens the hub.
    let reply = send(&mut s, tick(reply, ticks));
    let hub = income_hub(to_the_hub(&mut s, reply));
    // Each tick is stored as planned, exactly as ticked.
    assert_eq!(crate::interview::planned(&s.draft), ticks);
    assert_eq!(
        told_row(&hub),
        (
            "What you told us".into(),
            "W-2 wages, or employee pay a firm reported on a 1099 (Form 8919), Retirement or Social Security".into()
        )
    );
    // Amended for M3-N1: the planned topics first, then every other topic.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![
                row_of("Wages", "Planned", TaxSection::Wages),
                row_of("Retirement and Social Security", "Planned", TaxSection::Retirement),
            ],
            hub_tail_unasked(
                &income_topics_but(&[TaxSection::Wages, TaxSection::Retirement]),
                TaxSection::MoreIncome
            )
        )
    );
    assert_eq!(suggestion(&hub), TaxSection::Wages);
    // Entering the W-2 moves its tick from planned to the entries.
    let request = save_topic(
        &s.draft,
        TaxSection::Wages,
        vec![document(0, IncomeDocument::W2(w2(6_000_000)))],
    );
    let reply = send(&mut s, request);
    let hub = income_hub(to_the_hub(&mut s, reply));
    assert_eq!(crate::interview::planned(&s.draft).income, IncomeTopics::RETIREMENT.0);
    assert_eq!(crate::interview::planned(&s.draft).payments, PaymentTopics::EXTENSION.0);
    assert_eq!(suggestion(&hub), TaxSection::Retirement);
    // The "What you told us" row reopens the checklist from the hub, with
    // the W-2 ticked from its entry; unticking retirement returns to the
    // hub, which now suggests the walk's last step, Less common income
    // (gate fixes batch A).
    let request = go_to(&s.draft, PAGE_INCOME, TaxSection::Checklist);
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Edit(P::SaveChecklist { previous: PAGE_INCOME, review: false, .. })
    ));
    assert_eq!(checklist_of(&reply), ticks);
    let fewer = Checklist { income: IncomeTopics::WAGES.0, ..ticks };
    let hub = income_hub(send(&mut s, tick(reply, fewer)));
    assert_eq!(suggestion(&hub), TaxSection::MoreIncome);
    assert_eq!(crate::interview::planned(&s.draft), Checklist { income: 0, ..ticks });
    // A changed checklist erases nothing.
    assert_eq!(s.draft.income.len(), 1);
}

#[test]
fn topics_with_entries_open_ticked() {
    // A W-2 and a 1099-INT, a less-common income situation, a Form 1095-A,
    // a household employee and an extension payment: every one is shown
    // ticked from its entry, and nothing is planned.
    let mut d = complete(FederalStatus::Single);
    d.income.push(FederalIncomeEntry {
        refund_management: None,
        state_source: Default::default(),
        id: 2,
        owner: Owner::Taxpayer,
        label: "Bank".into(),
        income: document_income(&IncomeDocument::Int1099(native_int1099("Bank".into(), 20_000))),
        community: CommunityShare::NotNeeded,
    });
    d.next_id = 3;
    d.situations = vec![RareSituation::TaxTreaty];
    d.screening.as_mut().unwrap().household_employment = vec![household_employer(3_500)];
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 600, 0)).collect());
    d.payments.as_mut().unwrap().extension = 50_000;
    // As every save leaves it: each topic with entries ticked.
    let d = crate::interview::with_ticks(d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, navigate(&d, TaxSection::Checklist));
    assert!(matches!(reply, Reply::Edit(P::SaveChecklist { review: true, .. })));
    assert_eq!(
        checklist_of(&reply),
        Checklist {
            income: IncomeTopics::WAGES.0 | IncomeTopics::INTEREST_DIVIDENDS.0,
            credit_topics: CreditTopics::MARKETPLACE.0,
            other_taxes: OtherTaxTopics::HOUSEHOLD_EMPLOYEE.0,
            payments: PaymentTopics::EXTENSION.0,
            ..Checklist::NONE
        }
    );
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    // Saving it unchanged stores nothing twice: planned stays empty and
    // the review follows, unchanged.
    let value = checklist_of(&reply);
    assert!(matches!(send(&mut s, tick(reply, value)), Reply::Response(P::Review { .. })));
    assert_eq!(s.draft, d);
    let (_, _, _, answers) = review_state(&mut s);
    assert!(
        answers
            .iter()
            .any(|r| r.section == TaxSection::Checklist && status_text(&r.status) == "5 topics")
    );
}

#[test]
fn undeclared_checklist_ticks_are_refused() {
    // Unticking a topic with entries removes them (walk-owned data,
    // src/tests/cross/change_audit.rs); a tick the catalogs do not declare is refused.
    let d = crate::interview::with_ticks(complete(FederalStatus::Single));
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, navigate(&d, TaxSection::Checklist));
    let undeclared = Checklist { income: IncomeTopics::WAGES.0 | 1 << 20, ..Checklist::NONE };
    let message = refusal(current_source_refusal(
        "secondary-admission-3250",
        &mut s,
        tick(reply, undeclared),
        "MESSAGE_CHOOSE_ONLY_THE_LISTED_ITEMS",
        None,
        SourceScope::Fieldless,
    ));
    assert!(message.contains("Choose only the listed items."), "{message}");
    assert_eq!(s.draft, d);
}

#[test]
fn derived_topics_need_no_tick() {
    // W-2 box 12 W: the adjustments hub suggests the HSA with nothing
    // ticked.
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 100_000);
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(suggestion(&hub), TaxSection::Hsa);
    assert_eq!(told_row(&hub).1, "None of these");
    // A full-time student: the credits hub suggests education.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    let (_, hub) = at_the_credits_hub(d);
    assert_eq!(suggestion(&hub), TaxSection::Education);
    assert_eq!(told_row(&hub).1, "None of these");
    // Without a tick or a derived need, an empty topic is never suggested:
    // Continue goes to the next shown section, other taxes (M3-P19; every
    // return walks it since gate fixes batch A).
    let (_, hub) = at_the_credits_hub(complete(FederalStatus::Single));
    assert_eq!(suggestion(&hub), TaxSection::OtherTaxes);
}

#[test]
fn hub_suggests_attention_then_planned_then_continue() {
    // Interest and dividends is planned; a refused 1099-R code needs
    // attention on the later Retirement topic, which is suggested first.
    let mut s = planning(income_ticks(IncomeTopics::INTEREST_DIVIDENDS.0));
    let r = other_1099(1_000_000, "5", false);
    let request =
        save_topic(&s.draft, TaxSection::Retirement, vec![document(0, IncomeDocument::R1099(r))]);
    let reply = send(&mut s, request);
    let hub = income_hub(to_the_hub(&mut s, reply));
    assert_eq!(suggestion(&hub), TaxSection::Retirement);
    let rows = hub_rows(&hub);
    assert!(rows.contains(&row_of(
        "Interest and dividends",
        "Planned",
        TaxSection::InterestDividends
    )));
    assert!(
        rows.iter().any(|(_, status, section)| *section == TaxSection::Retirement
            && status.starts_with("Needs attention")),
        "{rows:?}"
    );
    // Without the attention, the planned topic.
    let request = save_topic(&s.draft, TaxSection::Retirement, vec![]);
    let reply = send(&mut s, request);
    let hub = income_hub(to_the_hub(&mut s, reply));
    assert_eq!(suggestion(&hub), TaxSection::InterestDividends);
    // Once it is entered, the walk's last step, Less common income (gate
    // fixes batch A); then Continue (M3-P19). A save while the walk has a
    // topic left opens that topic, whose Back opens the hub (to_the_hub).
    let request = save_topic(&s.draft, TaxSection::InterestDividends, vec![interest(20_000)]);
    let reply = send(&mut s, request);
    let hub = income_hub(to_the_hub(&mut s, reply));
    assert_eq!(suggestion(&hub), TaxSection::MoreIncome);
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
}

#[test]
fn itemizable_ticks_open_deductions_on_compare() {
    let opened = |planned: Checklist| {
        let mut d = complete(FederalStatus::Single);
        d.finished_sections = vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Adjustments,
        ];
        d.checklist = planned;
        let mut s = Interview::new(TEST_USER, d);
        match send(&mut s, P::OpenInterview { user: TEST_USER }) {
            Reply::Edit(page @ P::SaveDeductions { .. }) => (s, page),
            other => panic!("expected the deductions page, got {other:?}"),
        }
    };
    let route = |page: &P| deductions_value(page).0.route;
    // Nothing ticked, gifts only or car loan interest only: the standard
    // deduction, M1's default.
    for deductions in [0, DeductionTopics::GIFTS.0, DeductionTopics::CAR_LOAN.0] {
        let (_, page) = opened(Checklist { deductions, ..Checklist::NONE });
        assert_eq!(route(&page), DeductionRoute::Standard, "{deductions}");
    }
    // Any itemizable tick answers the route: Compare.
    let (mut s, page) =
        opened(Checklist { deductions: DeductionTopics::MORTGAGE.0, ..Checklist::NONE });
    assert_eq!(route(&page), DeductionRoute::Compare);
    // The planned mortgage is no item needing review.
    assert!(items_of(&mut s).is_empty());
    // Once the page is saved its stored route is shown, even Standard.
    let request = save_deductions(&s.draft, DeductionRoute::Standard, 0);
    accepted(&send(&mut s, request));
    let request = navigate(&s.draft, TaxSection::Deductions);
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the deductions page") };
    assert_eq!(route(&page), DeductionRoute::Standard);
    // Saved as it opened, Compare enters the itemized hub, whose walk opens
    // the planned mortgage page directly (M3-P19); its Back opens the hub.
    let (mut s, page) =
        opened(Checklist { deductions: DeductionTopics::MORTGAGE.0, ..Checklist::NONE });
    let reply = send(&mut s, page);
    assert!(matches!(reply, Reply::Edit(P::SaveMortgages { .. })), "{reply:?}");
    let hub = itemized_hub(to_the_hub(&mut s, reply));
    assert_eq!(suggestion(&hub), TaxSection::Mortgage);
    assert_eq!(told_row(&hub), ("What you told us".into(), "Mortgage interest".into()));
    assert!(items_of(&mut s).is_empty());
}

#[test]
fn checklist_estimate_lists_the_planned_sections() {
    let (mut s, _) = at_the_checklist();
    s.dirty = false; // persisted by the task
    let before = s.draft.clone();
    let path = |s: &mut Interview, value: Checklist| match send(
        s,
        P::EstimateChecklist { user: TEST_USER, value },
    ) {
        Reply::Response(P::RunningEstimate { progress, .. }) => {
            let text = shown(&progress);
            let start = text.find("Your return: ").unwrap();
            alloc::string::String::from(text[start..].trim_end_matches('\0'))
        }
        other => panic!("expected the running estimate, got {other:?}"),
    };
    // Gate fixes batch A: income, adjustments, credits and other taxes are
    // always shown, each asking its less-common page.
    assert_eq!(
        path(&mut s, Checklist::NONE),
        "Your return: Income, Adjustments to income, Credits, Other taxes, Federal payments, Review"
    );
    assert_eq!(
        path(
            &mut s,
            Checklist {
                income: IncomeTopics::WAGES.0,
                deductions: DeductionTopics::MORTGAGE.0,
                payments: PaymentTopics::ESTIMATED.0,
                ..Checklist::NONE
            }
        ),
        "Your return: Income, Adjustments to income, Deductions, Credits, Other taxes, Federal payments, Review"
    );
    assert_eq!(
        path(
            &mut s,
            Checklist {
                adjustments: AdjustmentTopics::IRA_CONTRIBUTIONS.0,
                credit_topics: CreditTopics::TUITION.0,
                other_taxes: OtherTaxTopics::STOCK_OPTIONS.0,
                ..Checklist::NONE
            }
        ),
        "Your return: Income, Adjustments to income, Credits, Other taxes, Federal payments, Review"
    );
    // The stock-options tick is answered on Investment sales (Income); an
    // estimated payment adds Federal payments.
    assert_eq!(
        path(&mut s, Checklist { payments: PaymentTopics::ESTIMATED.0, ..Checklist::NONE }),
        "Your return: Income, Adjustments to income, Credits, Other taxes, Federal payments, Review"
    );
    // The calculation is pure.
    assert_eq!(s.draft, before);
    assert!(!s.dirty);
}

#[test]
fn a_plan_is_never_a_remaining_item() {
    // Plans are not completeness: a tick with nothing entered is no item
    // needing review and does not block Finish; its topic row reads
    // "Planned" (a concrete plan).
    let mut d = complete(FederalStatus::Single);
    d.checklist = income_ticks(IncomeTopics::RETIREMENT.0);
    let mut s = Interview::new(TEST_USER, d.clone());
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
    let hub = hub_of(from_review(&mut s, TaxSection::Income));
    assert!(hub_rows(&hub).contains(&row_of(
        "Retirement and Social Security",
        "Planned",
        TaxSection::Retirement
    )));
    // Entering it: the row shows the entry and the tick stays.
    let request = save_topic(
        &s.draft,
        TaxSection::Retirement,
        vec![document(
            0,
            IncomeDocument::R1099(r1099(
                1_000_000,
                DistributionCode::Normal { what_happened: Disposition::Kept },
                true,
            )),
        )],
    );
    accepted(&send(&mut s, request));
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    assert_ne!(s.draft.checklist.income & IncomeTopics::RETIREMENT.0, 0);
    let (items, _, _, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
}

// M3-P19: the guided walk, section summaries and skipped sections.

// The pages of a fixture's path, by the request that answered each.
fn path_of(sent: &[P]) -> Vec<&'static str> {
    sent.iter()
        .map(|request| match request {
            P::SaveSetup { .. } => "setup",
            P::SavePeople { .. } => "you",
            P::SaveChildren { .. } => "people",
            P::SaveChecklist { .. } => "checklist",
            P::SaveIncomes { topic: TaxSection::Wages, .. } => "wages",
            P::SaveIncomes { topic: TaxSection::InterestDividends, .. } => "interest",
            P::ContinueIncome { .. } => "income summary Continue",
            P::SaveSituations { topic: TaxSection::MoreIncome, .. } => "less common income",
            P::SaveMoreAdjustments { .. } => "more adjustments",
            P::ContinueAdjustments { .. } => "adjustments summary Continue",
            P::SaveCare { .. } => "care",
            P::SaveMoreCredits { .. } => "more credits",
            P::ContinueCredits { .. } => "credits summary Continue",
            P::SaveOtherTaxes { .. } => "other taxes",
            P::SavePayments { .. } => "payments",
            other => panic!("unexpected request {other:?}"),
        })
        .collect()
}
// The hub a reply shows, or a panic.
pub(in crate::tests) fn hub_of(reply: Reply) -> P {
    match reply {
        Reply::Response(hub @ P::Hub { .. }) => hub,
        other => panic!("expected a hub, got {other:?}"),
    }
}
pub(in crate::tests) fn progress_of(reply: &Reply) -> alloc::string::String {
    match reply {
        Reply::Edit(P::SaveIncomes { progress, .. }) => {
            shown(progress.as_ref().expect("native supplied progress"))
        }
        Reply::Response(
            P::Hub { progress, .. }
            | P::BeginSituations { progress, .. }
            | P::BeginChildren { progress, .. },
        )
        | Reply::Edit(
            P::SavePayments { progress, .. }
            | P::SaveHsa { progress, .. }
            | P::SaveEducation { progress, .. }
            | P::SaveChecklist { progress, .. },
        ) => shown(progress),
        other => panic!("unexpected page {other:?}"),
    }
}
// A new single return at the checklist, ticked with `ticks`: the reply to
// its Next.
fn walked_from_the_checklist(ticks: Checklist) -> (Interview, Reply) {
    let (mut s, reply) = at_the_checklist();
    let reply = send(&mut s, tick(reply, ticks));
    (s, reply)
}
// Saves the income topic page a reply opened with `documents`.
fn save_opened_topic(
    s: &mut Interview,
    reply: &Reply,
    documents: Vec<IncomeDocumentItem>,
) -> Reply {
    let Reply::Edit(P::SaveIncomes { topic, .. }) = reply else {
        panic!("expected an income topic page, got {reply:?}")
    };
    let request = save_topic(&s.draft, *topic, documents);
    send(s, request)
}

#[test]
fn m1_path_is_checklist_wages_interest_summary_review() {
    // The m1-single-w2 fixture through the walk: after the filer pages the
    // checklist (income row, the W-2 and interest ticks, OK, Next), the two
    // planned pages in the hub's order, the income summary's Continue action,
    // and the review. Adjustments, deductions, credits, other taxes and payments
    // have nothing for this return; Payments still asks the refund election.
    // Design (b), B′: 6
    // navigation taps on 2 navigation screens (the checklist and the
    // summary); the 12 fields are unchanged. Gate fixes batch A: the people
    // page (empty) and each section's less-common page are asked once, and
    // the adjustments and credits summaries continue.
    let raw = include_str!("../../../tests/fixtures/returns/m1-single-w2.json");
    let (mut s, result, sent) = enter_through_the_hubs("m1-single-w2", raw);
    assert_fixture_lines(raw, &result);
    assert_eq!(
        path_of(&sent),
        [
            "setup",
            "you",
            "people",
            "checklist",
            "wages",
            "interest",
            "less common income",
            "income summary Continue",
            "more adjustments",
            "adjustments summary Continue",
            "more credits",
            "credits summary Continue",
            "other taxes",
            "payments"
        ]
    );
    // The income summary's Continue action was its only tap, and Go to was
    // never needed.
    assert_eq!(sent.iter().filter(|p| matches!(p, P::ContinueIncome { .. })).count(), 1);
    assert!(!sent.iter().any(|p| matches!(p, P::Navigate { .. })));
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
}

#[test]
fn m2_path_completes_the_care_editor_before_credits_summary() {
    // The m2-hoh-two-children fixture: the care costs are answered on the
    // people page and the child tax credit and the EIC need no page, so the
    // credits hub walks only More credits (gate fixes batch A).
    // Design (b), A′: 5 navigation taps (income row, the W-2 tick, OK, Next;
    // the summary's Continue action) on 2 screens; the 20 fields are unchanged.
    let raw = include_str!("../../../tests/fixtures/returns/m2-hoh-two-children.json");
    let (mut s, result, sent) = enter_through_the_hubs("m2-hoh-two-children", raw);
    assert_fixture_lines(raw, &result);
    assert_eq!(
        path_of(&sent),
        [
            "setup",
            "you",
            "people",
            "checklist",
            "wages",
            "less common income",
            "income summary Continue",
            "more adjustments",
            "adjustments summary Continue",
            "care",
            "more credits",
            "credits summary Continue",
            "other taxes",
            "payments"
        ]
    );
    assert!(s.draft.finished_sections.contains(&TaxSection::Credits));
    // The review lists it, and it opens from there with its care row.
    let (_, _, _, answers) = review_state(&mut s);
    let credits = answers.iter().find(|r| r.section == TaxSection::Credits).unwrap();
    // Care is the one recorded topic; empty More credits adds none.
    assert_eq!(status_text(&credits.status), "1 topic");
    let hub = hub_of(from_review(&mut s, TaxSection::Credits));
    assert!(hub_rows(&hub).contains(&row_of(
        "Child and dependent care",
        "1 provider/benefit record",
        TaxSection::Care
    )));
    assert_eq!(suggestion(&hub), TaxSection::OtherTaxes);
}

#[test]
fn walk_follows_planned_topics_in_hub_order() {
    // Ticked in any order, the planned income topics open in the hub's
    // order: Wages, Retirement and Social Security, then Unemployment and
    // state refunds; the unplanned topics between them are passed.
    let (mut s, mut reply) = walked_from_the_checklist(income_ticks(
        IncomeTopics::GOVERNMENT_BENEFITS.0 | IncomeTopics::RETIREMENT.0 | IncomeTopics::WAGES.0,
    ));
    let mut walked = Vec::new();
    let hub = loop {
        match &reply {
            Reply::Edit(P::SaveIncomes { topic, .. }) => {
                walked.push(*topic);
                // A W-2 on Wages; the other two are saved with nothing.
                let documents = if *topic == TaxSection::Wages {
                    vec![document(0, IncomeDocument::W2(w2(6_000_000)))]
                } else {
                    vec![]
                };
                reply = save_opened_topic(&mut s, &reply, documents);
            }
            _ => break income_hub(past_less_common(&mut s, reply)),
        }
    };
    assert_eq!(walked, [TaxSection::Wages, TaxSection::Retirement, TaxSection::GovernmentBenefits]);
    // The walk ends, after Less common income, at the hub, shown as the
    // section summary: it continues to the next shown section, adjustments
    // (gate fixes batch A).
    let P::Hub { progress, .. } = &hub else { unreachable!() };
    assert!(shown(progress).contains("· You're done with income"), "{}", shown(progress));
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    // A planned topic saved with nothing is visited: it stays planned (a
    // remaining item), listed "Not started", and the walk passes it.
    assert_eq!(
        crate::interview::planned(&s.draft).income,
        IncomeTopics::RETIREMENT.0 | IncomeTopics::GOVERNMENT_BENEFITS.0
    );
    assert!(s.draft.finished_sections.contains(&TaxSection::Retirement));
    assert!(s.draft.finished_sections.contains(&TaxSection::GovernmentBenefits));
    assert!(hub_rows(&hub).contains(&row_of(
        "Retirement and Social Security",
        "Planned",
        TaxSection::Retirement
    )));
    let mut resumed = restart(s.draft.clone());
    income_hub(send(&mut resumed, P::OpenInterview { user: TEST_USER }));
}

#[test]
fn empty_sections_are_skipped_and_listed_on_the_review() {
    let raw = include_str!("../../../tests/fixtures/returns/m1-single-w2.json");
    let (mut s, _, sent) = enter_through_the_hubs("m1-single-w2", raw);
    // Deductions with nothing is skipped; the final Payments election is asked.
    // The less-common pages are
    // each section's walk since gate fixes batch A, and Less common income
    // is the only situations page sent.
    assert!(!sent.iter().any(|p| matches!(p, P::SaveDeductions { .. })));
    assert_eq!(sent.iter().filter(|p| matches!(p, P::SavePayments { .. })).count(), 1);
    assert!(sent.iter().all(|p| match p {
        P::SaveSituations { topic, .. } => *topic == TaxSection::MoreIncome,
        _ => true,
    }));
    // ...yet each is recorded as passed, and the review lists every section,
    // a skipped one reading "Nothing entered" (the deductions row names the
    // standard deduction the return takes).
    for section in [
        TaxSection::Adjustments,
        TaxSection::Deductions,
        TaxSection::Credits,
        TaxSection::OtherTaxes,
        TaxSection::Payments,
    ] {
        assert!(s.draft.finished_sections.contains(&section), "{section:?}");
    }
    let (_, _, _, answers) = review_state(&mut s);
    let status = |section: TaxSection| {
        answers
            .iter()
            .find(|r| r.section == section)
            .map(|r| status_text(&r.status))
            .unwrap_or_else(|| panic!("no {section:?} row: {answers:?}"))
    };
    for section in
        [TaxSection::Adjustments, TaxSection::Credits, TaxSection::OtherTaxes, TaxSection::Payments]
    {
        assert_eq!(status(section), "Nothing entered", "{section:?}");
    }
    assert_eq!(status(TaxSection::Deductions), "Standard deduction");
    // Each row opens its page from the review.
    let hub = hub_of(from_review(&mut s, TaxSection::Adjustments));
    assert!(matches!(hub, P::Hub { page: PAGE_ADJUSTMENTS, review: true, .. }));
    assert!(matches!(
        from_review(&mut s, TaxSection::Payments),
        Reply::Edit(P::SavePayments { .. })
    ));
}

#[test]
fn a_section_with_attention_is_never_skipped() {
    // The HSA page was visited and saved with no account, so the HSA is no
    // longer a derived topic; a W-2 with box 12 W and no account still
    // leaves a remaining item on it, which keeps the adjustments section.
    let after_wages = |w: i64| {
        let mut d = complete(FederalStatus::Single);
        d.finished_sections = vec![
            TaxSection::Children,
            TaxSection::Checklist,
            TaxSection::Hsa,
            TaxSection::MoreAdjustments,
            TaxSection::Income,
        ];
        let mut s = Interview::new(TEST_USER, d);
        let wage = if w == 0 { w2(6_000_000) } else { w2_with_box_12_w(6_000_000, w) };
        let request =
            save_topic(&s.draft, TaxSection::Wages, vec![document(0, IncomeDocument::W2(wage))]);
        accepted(&send(&mut s, request));
        s
    };
    let mut s = after_wages(100_000);
    assert!(!s.draft.finished_sections.contains(&TaxSection::Adjustments));
    // Resume opens the hub (nothing left to walk), suggesting the topic
    // that needs attention.
    let hub = hub_of(send(&mut s, P::OpenInterview { user: TEST_USER }));
    assert!(matches!(hub, P::Hub { page: PAGE_ADJUSTMENTS, .. }));
    assert_eq!(suggestion(&hub), TaxSection::Hsa);
    assert!(hub_rows(&hub).iter().any(|(_, status, section)| {
        *section == TaxSection::Hsa && status.starts_with("Needs attention")
    }));
    // Without box 12 W the section is still shown, as every adjustments
    // section asks More adjustments (gate fixes batch A), with nothing
    // needing attention.
    let mut s = after_wages(0);
    assert!(!s.draft.finished_sections.contains(&TaxSection::Adjustments));
    let hub = hub_of(send(&mut s, P::OpenInterview { user: TEST_USER }));
    assert!(hub_rows(&hub).iter().all(|(_, status, _)| !status.starts_with("Needs attention")));
}

#[test]
fn revisit_from_the_hub_returns_to_it() {
    // The walk has ended (Wages, the one planned topic, is saved): a topic
    // opened from the summary, planned or not, returns to the summary.
    let (mut s, reply) = walked_from_the_checklist(income_ticks(IncomeTopics::WAGES.0));
    let reply =
        save_opened_topic(&mut s, &reply, vec![document(0, IncomeDocument::W2(w2(6_000_000)))]);
    let hub = income_hub(past_less_common(&mut s, reply));
    // An unplanned topic, with a document.
    let request = go_to(&s.draft, PAGE_INCOME, TaxSection::InterestDividends);
    let page = send(&mut s, request);
    let reply = save_opened_topic(&mut s, &page, vec![interest(20_000)]);
    let hub_after = income_hub(reply);
    // Amended for M3-N1: then every other topic; Continue is a separate action.
    assert_eq!(
        hub_rows(&hub_after),
        with_tail(
            vec![
                row_of("Wages", "1 W-2 · $60000.00", TaxSection::Wages),
                row_of(
                    "Interest and dividends",
                    "1 1099-INT · $200.00",
                    TaxSection::InterestDividends
                ),
            ],
            hub_tail(
                &income_topics_but(&[TaxSection::Wages, TaxSection::InterestDividends]),
                TaxSection::MoreIncome
            )
        )
    );
    assert_eq!(suggestion(&hub_after), suggestion(&hub));
    // The planned topic again, saved unchanged: the hub, and no revision.
    let revision = s.draft.revision;
    let request = go_to(&s.draft, PAGE_INCOME, TaxSection::Wages);
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the Wages page") };
    income_hub(send(&mut s, page));
    assert_eq!(s.draft.revision, revision);
}

// A new single return that plans W-2 wages and estimated payments, at the
// Wages page the walk opened.
fn wages_and_estimated_payments() -> (Interview, Reply) {
    walked_from_the_checklist(Checklist {
        income: IncomeTopics::WAGES.0,
        payments: PaymentTopics::ESTIMATED.0,
        ..Checklist::NONE
    })
}

#[test]
fn progress_counts_only_shown_sections() {
    // Before any tick: setup, you, people, the checklist, the four sections
    // every return walks for their less-common pages (gate fixes batch A)
    // and the review.
    let (_, reply) = at_the_checklist();
    assert!(progress_of(&reply).contains(
        "Your return: Income, Adjustments to income, Credits, Other taxes, Federal payments, Review"
    ));
    // Planned: federal payments joins the count, 10 sections.
    let (mut s, reply) = wages_and_estimated_payments();
    assert!(
        progress_of(&reply).contains("2026 federal · section 5 of 10 · Wages"),
        "{}",
        progress_of(&reply)
    );
    let reply =
        save_opened_topic(&mut s, &reply, vec![document(0, IncomeDocument::W2(w2(6_000_000)))]);
    assert!(progress_of(&reply).contains("section 5 of 10 · Less common income"));
    let reply = past_less_common(&mut s, reply);
    assert!(progress_of(&reply).contains("section 5 of 10 · You're done with income"));
    // A skipped section opened from the review counts itself: deductions,
    // with nothing for this return.
    let reply = from_review(&mut s, TaxSection::Deductions);
    let Reply::Edit(P::SaveDeductions { progress, .. }) = &reply else {
        panic!("expected the deductions page, got {reply:?}")
    };
    assert!(shown(progress).contains("section 7 of 11"), "{}", shown(progress));
}

#[test]
fn back_passes_skipped_sections() {
    let (mut s, reply) = wages_and_estimated_payments();
    let reply =
        save_opened_topic(&mut s, &reply, vec![document(0, IncomeDocument::W2(w2(6_000_000)))]);
    let hub = income_hub(past_less_common(&mut s, reply));
    // The summary continues to the next shown section: adjustments, which
    // every return walks for More adjustments (gate fixes batch A).
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    // The payments page's Back opens the previous shown section, other
    // taxes, which every return walks too.
    let Reply::Edit(P::SavePayments { previous, .. }) = from_review(&mut s, TaxSection::Payments)
    else {
        panic!("expected the payments page")
    };
    assert_eq!(previous, PAGE_OTHER_TAXES);
    // A hub's previous section is the previous shown one too: the credits
    // hub, opened from the review, goes back to the adjustments hub,
    // passing deductions (nothing there).
    let hub = hub_of(from_review(&mut s, TaxSection::Credits));
    let P::Hub { previous, allowed_sections, .. } = &hub else { unreachable!() };
    assert_eq!(*previous, PAGE_ADJUSTMENTS);
    assert!(allowed_sections.contains(&PAGE_ADJUSTMENTS));
}

#[test]
fn resume_opens_the_next_step() {
    // Wages and interest planned.
    let (mut s, reply) = walked_from_the_checklist(Checklist {
        income: IncomeTopics::WAGES.0 | IncomeTopics::INTEREST_DIVIDENDS.0,
        ..Checklist::NONE
    });
    let resumed = |s: &Interview| {
        let mut resumed = restart(s.draft.clone());
        send(&mut resumed, P::OpenInterview { user: TEST_USER })
    };
    // Mid-walk, resume opens the next planned topic of the current hub.
    let reply =
        save_opened_topic(&mut s, &reply, vec![document(0, IncomeDocument::W2(w2(6_000_000)))]);
    assert!(matches!(
        reply,
        Reply::Edit(P::SaveIncomes { topic: TaxSection::InterestDividends, .. })
    ));
    assert!(matches!(
        resumed(&s),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::InterestDividends, .. })
    ));
    // With the walk ended, the hub: its summary.
    let reply = save_opened_topic(&mut s, &reply, vec![interest(20_000)]);
    let hub = income_hub(past_less_common(&mut s, reply));
    assert!(progress_of(&resumed(&s)).contains("You're done with income"));
    // Then, with nothing else planned, the sections every return walks
    // (gate fixes batch A) to the review.
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    let reply = send(&mut s, go_on(&hub));
    walk_on_to_review(&mut s, reply);
    assert!(matches!(resumed(&s), Reply::Response(P::Review { .. })));
}

#[test]
fn navigation_groups_preserve_topic_and_subtopic_identities() {
    // Existing section IDs remain stable; new pages append to the enum.
    // Navigation groups separately place those pages beside their topics.
    let sequence = [
        TaxSection::Setup,
        TaxSection::People,
        TaxSection::Spouse,
        TaxSection::Children,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Wages,
        TaxSection::InterestDividends,
        TaxSection::InvestmentSales,
        TaxSection::Retirement,
        TaxSection::SelfEmployment,
        TaxSection::Rentals,
        TaxSection::GovernmentBenefits,
        TaxSection::OtherIncome,
        TaxSection::Gambling,
        TaxSection::Farming,
        TaxSection::PassThrough,
        TaxSection::CanceledDebt,
        TaxSection::WorkAbroad,
        TaxSection::MoreIncome,
        TaxSection::Adjustments,
        TaxSection::Hsa,
        TaxSection::SelfEmployedBenefits,
        TaxSection::IraContributions,
        TaxSection::Educator,
        TaxSection::StudentLoan,
        TaxSection::EmployeeExpenses,
        TaxSection::MoreAdjustments,
        TaxSection::Deductions,
        TaxSection::Itemized,
        TaxSection::Medical,
        TaxSection::Taxes,
        TaxSection::Mortgage,
        TaxSection::GiftCarryovers,
        TaxSection::OtherItemized,
        TaxSection::MoreItemized,
        TaxSection::Credits,
        TaxSection::Education,
        TaxSection::Saver,
        TaxSection::Marketplace,
        TaxSection::MoreCredits,
        TaxSection::OtherTaxes,
        TaxSection::ParentTax,
        TaxSection::Payments,
        TaxSection::IncomeByPeriod,
        TaxSection::InjuredSpouse,
        TaxSection::Review,
        TaxSection::Care,
        TaxSection::HomeOffices,
    ];
    for (i, section) in sequence.iter().enumerate() {
        assert_eq!(*section as usize, i, "{section:?}");
    }
    let (_, income) = at_the_income_hub();
    let P::Hub { navigation, .. } = income else { unreachable!() };
    assert_eq!(
        navigation.income,
        (TaxSection::Income as i32..=TaxSection::SelfEmployment as i32)
            .chain([TaxSection::HomeOffices as i32])
            .chain(TaxSection::Rentals as i32..=TaxSection::MoreIncome as i32)
            .collect::<Vec<_>>()
    );
    // Mortgage source records remain available without itemizing.
    assert_eq!(
        navigation.deductions,
        vec![TaxSection::Deductions as i32, TaxSection::Mortgage as i32]
    );
    assert!(!navigation.return_sections.contains(&(TaxSection::Spouse as i32)));
    assert!(navigation.credits.contains(&(TaxSection::MoreCredits as i32)));
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_deductions(&s.draft, DeductionRoute::Compare, 0);
    let itemized = itemized_hub(send_past_less_common(&mut s, request));
    let P::Hub { navigation, .. } = itemized else { unreachable!() };
    assert_eq!(
        navigation.deductions,
        (TaxSection::Deductions as i32..=TaxSection::MoreItemized as i32).collect::<Vec<_>>()
    );
}

// M3-N1: tappable hub and review rows.

// The rows of a hub or the review (remaining items, then "Your answers").
fn rows_of(screen: &P) -> Vec<ReviewItem> {
    match screen {
        // A hub's rows as the screen shows them: "What you told us", the
        // topic list, then the previous-section link.
        P::Hub { told, topics, previous_section, .. } => core::iter::once(told)
            .chain(&topics.items)
            .chain(core::iter::once(previous_section))
            .cloned()
            .collect(),
        P::Review { remaining_items, answers, .. } => {
            remaining_items.iter().chain(answers).cloned().collect()
        }
        other => panic!("expected a hub or the review, got {other:?}"),
    }
}
// The request a tap on `row` constructs: OpenSection's CopyFrom reads the
// screen's cookie, revision and page, and the bound row's section.
fn tap_row(screen: &P, row: ReviewItem) -> P {
    let (cookie, revision, page) = match screen {
        P::Hub { cookie, revision, page, .. } | P::Review { cookie, revision, page, .. } => {
            (cookie.clone(), *revision, *page)
        }
        other => panic!("expected a hub or the review, got {other:?}"),
    };
    assert!(rows_of(screen).contains(&row), "{row:?} is not a row");
    P::OpenSection { user: TEST_USER, cookie, revision, from: page, section: row.section }
}
// Construct OpenSection with its CopyFrom fallback: without a bound row,
// `^<ReviewItem>.section` is absent and `$.suggested` applies. This link-only
// request is not offered in Actions.
fn open_without_bound_row(screen: &P) -> P {
    let suggested = match screen {
        P::Hub { suggested, .. } | P::Review { suggested, .. } => *suggested,
        other => panic!("expected a hub or the review, got {other:?}"),
    };
    let P::OpenSection { user, cookie, revision, from, .. } =
        tap_row(screen, rows_of(screen)[0].clone())
    else {
        unreachable!()
    };
    P::OpenSection { user, cookie, revision, from, section: suggested }
}
fn row_for(screen: &P, section: TaxSection) -> ReviewItem {
    rows_of(screen)
        .into_iter()
        .find(|r| r.section == section)
        .unwrap_or_else(|| panic!("no {section:?} row on {screen:?}"))
}
// The ordinary Actions request, constructed from the displayed hub.
pub(in crate::tests) fn hub_action(hub: &P) -> P {
    let P::Hub { cookie, revision, page, primary_actions, .. } = hub else {
        panic!("expected a hub")
    };
    let user = TEST_USER;
    let cookie = cookie.clone();
    let revision = *revision;
    if primary_actions.contains(&(libertas_macros::variant_index!(P::ReviewAccepted) as i32)) {
        P::ReviewAccepted { user }
    } else {
        let expected = match *page {
            PAGE_INCOME => libertas_macros::variant_index!(P::ContinueIncome),
            PAGE_ADJUSTMENTS => libertas_macros::variant_index!(P::ContinueAdjustments),
            PAGE_ITEMIZED => libertas_macros::variant_index!(P::ContinueItemized),
            PAGE_CREDITS => libertas_macros::variant_index!(P::ContinueCredits),
            _ => panic!("not a section hub"),
        };
        assert_eq!(
            primary_actions,
            &vec![expected as i32],
            "the displayed hub must offer its own Continue request"
        );
        match *page {
            PAGE_INCOME => P::ContinueIncome { user, cookie, revision },
            PAGE_ADJUSTMENTS => P::ContinueAdjustments { user, cookie, revision },
            PAGE_ITEMIZED => P::ContinueItemized { user, cookie, revision },
            PAGE_CREDITS => P::ContinueCredits { user, cookie, revision },
            _ => panic!("not a section hub"),
        }
    }
}
// A protocol variant's source, from `start` to the end of its fields.
fn declaration<'a>(source: &'a str, start: &str) -> &'a str {
    let at = source.find(start).unwrap_or_else(|| panic!("{start}"));
    let rest = &source[at..];
    &rest[..rest.find("\n    },").unwrap_or(rest.len())]
}

#[test]
fn issue_navigation_uses_record_identity_and_localized_labels() {
    // An unnamed source stores no label; its output context is the localized
    // kind and the real identity, and its persisted bytes stay unchanged.
    let defaults = [
        ("", cash_business(0), "LABEL_ENTRY_BUSINESS"),
        ("", farm_business(vec![], 0), "LABEL_ENTRY_FARM"),
        ("", FederalIncome::Rental { data: rental(0, 0) }, "LABEL_ENTRY_RENTAL"),
        (
            "",
            FederalIncome::Gambling {
                data: FederalGambling {
                    winnings: 0,
                    losses: 100,
                    withholding: 0,
                    own_records: true,
                    ordinary: true,
                },
            },
            "LABEL_ENTRY_GAMBLING_LOSSES_ENTRY",
        ),
    ];
    for (label, income, key) in defaults {
        let mut entry = complete(FederalStatus::Single).income.remove(0);
        entry.label = label.into();
        entry.income = income;
        let original = entry.clone();
        assert_eq!(
            crate::interview::income_issue_context(&entry).label(),
            Message::new(key, vec![MessageArg::Number(entry.id)])
        );
        assert_eq!(entry, original);
        entry.label = "User's {0} name".into();
        assert_eq!(
            crate::interview::income_issue_context(&entry).label(),
            Message::new("TEXT", vec![MessageArg::Text(entry.label.clone())])
        );
    }
    let mut d = complete(FederalStatus::Single);
    d.income[0].label = "Same name {0}".into();
    let mut bank = d.income[0].clone();
    bank.id += 1;
    bank.income = savings_income(100, 0);
    d.income.push(bank);
    let wage = crate::interview::income_issue_context(&d.income[0]);
    let bank = crate::interview::income_issue_context(&d.income[1]);
    assert_ne!(wage, bank, "equal labels are different record identities");
    assert_eq!(crate::interview::issue_page(&d, &wage, None), Some(TaxSection::Wages));
    assert_eq!(crate::interview::issue_page(&d, &bank, None), Some(TaxSection::InterestDividends));
    d.income[1].label = "Renamed bank".into();
    assert_eq!(crate::interview::issue_page(&d, &bank, None), Some(TaxSection::InterestDividends));
    d.income.remove(1);
    assert_eq!(crate::interview::issue_page(&d, &bank, None), None);
    let mut session = Interview::new(TEST_USER, d);
    let (_, _, _, answers) = review_state(&mut session);
    let you = answers.iter().find(|r| r.section == TaxSection::People).unwrap();
    assert_eq!(status_message(&you.item), Message::new("SECTION_PEOPLE", vec![]));
    let roundtrip = ReviewItem::from_avro(&you.to_avro()).unwrap();
    assert_eq!(roundtrip, *you);
    // Identical issue text on two equal-name documents must remain two items.
    let mut second_wage = session.draft.income[0].clone();
    second_wage.id += 1;
    session.draft.income.push(second_wage);
    let mut result = estimate(&session.draft);
    for entry in &session.draft.income {
        crate::federal_rules::issue_localized(
            &mut result,
            &crate::interview::income_issue_context(entry),
            "Income: Same name {0}",
            "ISSUE_EMPLOYEE_WAGES_RECORDS",
            "Check the employee wage records.",
        );
    }
    let remaining = crate::interview::remaining_items(&session.draft, &result, TaxSection::Review);
    assert_eq!(remaining.iter().filter(|item| item.section == TaxSection::Wages && matches!(status_message(&item.status).args.get(1), Some(MessageArg::Nested(issue)) if issue.key == "ISSUE_EMPLOYEE_WAGES_RECORDS")).count(), 2, "{remaining:?}");
}

#[test]
fn hub_row_binds_its_topic() {
    // Schema: every hub and review row is a ReviewItem, whose type links to
    // OpenSection; OpenSection has no input (every field hidden), reads the
    // tapped row's section and falls back to the suggestion; the hub
    // declares separate Actions and Go to controls; every
    // Response carrying rows lists OpenSection and keeps Navigate.
    let protocol = include_str!("../../core/protocol.rs");
    assert!(protocol.contains("#[libertas_link(\"OpenSection\")]\npub struct ReviewItem {"));
    let open = declaration(protocol, "    OpenSection {");
    assert!(open.contains("#[libertas_copy_from(\"^<ReviewItem>.section,$.suggested\")]"));
    let fields = open.matches(": ").count();
    assert_eq!(open.matches("#[libertas_hidden]").count(), fields, "{open}");
    let before_hub = &protocol[..protocol.find("    Hub {").unwrap()];
    let hub_attrs = &before_hub[before_hub.rfind("/// Topics").unwrap()..];
    assert!(hub_attrs.contains("ContinueIncome,ContinueAdjustments,ContinueItemized,ContinueCredits,ReviewAccepted,Navigate,OpenSection"));
    assert!(hub_attrs.contains("libertas_constrained_by"));
    assert!(declaration(protocol, "    Hub {").contains("allowed_actions"));
    assert!(
        protocol.contains(
            "#[libertas_next_request(\"Navigate,OpenSection,StartFinish,OpenInterview\")]"
        )
    );
    // Server: the income hub suggests the planned Wages; a tap on another
    // row opens that topic at once, and saving it returns to the hub.
    let (mut s, hub) = at_the_income_hub();
    assert_eq!(suggestion(&hub), TaxSection::Wages);
    let request = tap_row(&hub, row_for(&hub, TaxSection::InterestDividends));
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the interest page") };
    let P::SaveIncomes { topic, .. } = &page else {
        panic!("expected the interest page, got {page:?}")
    };
    assert_eq!(*topic, TaxSection::InterestDividends);
    // "What you told us" opens the checklist, and the less-common row its
    // page, each in one tap.
    let request = tap_row(&hub, row_for(&hub, TaxSection::Checklist));
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveChecklist { .. })));
    let request = tap_row(&hub, row_for(&hub, TaxSection::MoreIncome));
    assert!(matches!(
        send(&mut s, request),
        Reply::Response(P::BeginSituations { topic: TaxSection::MoreIncome, .. })
            | Reply::Edit(P::SaveSituations { topic: TaxSection::MoreIncome, .. })
    ));
    // A section the hub does not offer is refused as for Go to.
    let mut forged = tap_row(&hub, row_for(&hub, TaxSection::Wages));
    if let P::OpenSection { section, .. } = &mut forged {
        *section = TaxSection::Spouse;
    }
    assert!(refusal(send(&mut s, forged)).contains("not available"));
}

#[test]
fn open_section_fallback_and_navigate_use_the_suggestion() {
    // OpenSection accepts the unbound CopyFrom fallback. Go to's popup
    // starts at the same suggestion and opens the same page.
    let (mut s, hub) = at_the_income_hub();
    assert_eq!(suggestion(&hub), TaxSection::Wages);
    let by_fallback = send(&mut s, open_without_bound_row(&hub));
    let by_popup = send(&mut s, go_on(&hub));
    for reply in [by_fallback, by_popup] {
        assert!(
            matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::Wages, .. })),
            "{reply:?}"
        );
    }
    // The popup offers the hub's destinations, the suggestion among them.
    let P::Navigate { section, navigation, .. } = choose_suggestion(&hub) else { unreachable!() };
    assert!(navigation.groups.contains(&section.group_index()));
    assert_eq!(section.section(), suggestion(&hub));
}

#[test]
fn review_item_row_opens_its_page() {
    // A unsupported 1099-R code5 is a remaining item on Retirement: its row opens that page
    // in one tap. An answers row opens its section. With the interview walk
    // complete, only Navigate and link-only OpenSection remain allowed;
    // Finish joins only with nothing remaining.
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    accepted(&send(&mut s, save_documents(&original, vec![document(0, IncomeDocument::R1099(r))])));
    let Reply::Response(review @ P::Review { .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected the review")
    };
    let P::Review { remaining_items, allowed_actions, .. } = &review else { unreachable!() };
    let always = [
        libertas_macros::variant_index!(P::Navigate) as i32,
        libertas_macros::variant_index!(P::OpenSection) as i32,
    ];
    assert_eq!(allowed_actions, &always.to_vec());
    let item = remaining_items[0].clone();
    assert_eq!(item.section, TaxSection::Retirement);
    assert!(matches!(
        send(&mut s, tap_row(&review, item)),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::Retirement, review: true, .. })
    ));
    let request = tap_row(&review, row_for(&review, TaxSection::People));
    assert!(matches!(
        send(&mut s, request),
        Reply::Edit(P::SavePeople { .. }) | Reply::Response(P::BeginPeople { .. })
    ));
    // With nothing remaining, Finish joins the two.
    let s2 = &mut Interview::new(TEST_USER, complete(FederalStatus::Single));
    let (items, _, actions, _) = review_state(s2);
    assert!(items.is_empty());
    for action in always {
        assert!(actions.contains(&action));
    }
    assert!(actions.contains(&FINISH));
}

#[test]
fn income_continue_action_replaces_the_next_step_row() {
    let (mut s, hub) = at_the_income_hub();
    let P::Hub { topics, previous_section, allowed_actions, .. } = &hub else { unreachable!() };
    assert_eq!(topics.items.last().unwrap().section, TaxSection::MoreIncome);
    assert_eq!(previous_section.section, TaxSection::Checklist);
    assert_eq!(status_text(&previous_section.item), "Previous step");
    assert!(allowed_actions.contains(&(libertas_macros::variant_index!(P::ContinueIncome) as i32)));
    assert!(!topics.items.iter().any(|row| row.section == TaxSection::Review));
    assert!(!s.draft.finished_sections.contains(&TaxSection::Income));
    // Continue opens the next shown section, adjustments, at its walk's
    // More adjustments page (gate fixes batch A).
    let reply = send(&mut s, hub_action(&hub));
    assert!(matches!(reply, Reply::Edit(P::SaveMoreAdjustments { .. })), "{reply:?}");
    assert!(s.draft.finished_sections.contains(&TaxSection::Income));
}

#[test]
fn each_hub_continue_owns_its_section_path() {
    for section in
        [TaxSection::Income, TaxSection::Adjustments, TaxSection::Itemized, TaxSection::Credits]
    {
        let mut draft = complete(FederalStatus::Single);
        draft.finished_sections.retain(|s| *s != section);
        match section {
            TaxSection::Income => draft.checklist.adjustments = AdjustmentTopics::STUDENT_LOAN.0,
            TaxSection::Adjustments => draft.checklist.deductions = DeductionTopics::GIFTS.0,
            TaxSection::Itemized => {
                draft.deductions.as_mut().unwrap().choice =
                    DeductionSelection::Itemized { facts: empty_facts() };
                draft.checklist.credit_topics = CreditTopics::MARKETPLACE.0;
            }
            TaxSection::Credits => draft.checklist.payments = PaymentTopics::ESTIMATED.0,
            _ => unreachable!(),
        }
        let mut session = Interview::new(TEST_USER, draft);
        let accepted_income = session.draft.income.clone();
        let request = back(&session.draft, section as i32);
        let hub = hub_of(send(&mut session, request));
        let P::Hub { previous, previous_section, allowed_actions, .. } = &hub else {
            unreachable!()
        };
        assert_eq!(previous_section.section as i32, *previous);
        assert_eq!(allowed_actions.len(), 3);
        assert!(
            allowed_actions.contains(&(libertas_macros::variant_index!(P::OpenSection) as i32))
        );
        // A previous link remains a leading-page operation, not a detail node.
        let reply = send(&mut session, tap_row(&hub, previous_section.clone()));
        accepted(&reply);
        let reply = send(&mut session, hub_action(&hub));
        assert!(
            match section {
                TaxSection::Income => matches!(reply, Reply::Edit(P::SaveStudentLoan { .. })),
                TaxSection::Adjustments => matches!(reply, Reply::Edit(P::SaveDeductions { .. })),
                TaxSection::Itemized => matches!(reply, Reply::Edit(P::SaveMarketplace { .. })),
                // Other taxes is shown on every return (gate fixes batch A).
                TaxSection::Credits => matches!(reply, Reply::Edit(P::SaveOtherTaxes { .. })),
                _ => unreachable!(),
            },
            "{section:?}: {reply:?}"
        );
        assert!(session.draft.finished_sections.contains(&section));
        assert_eq!(session.draft.income, accepted_income);
    }
}

// The less-common page of a hub (gate fixes batch A).
fn hub_topics_last(hub: TaxSection) -> Option<TaxSection> {
    match hub {
        TaxSection::Income => Some(TaxSection::MoreIncome),
        TaxSection::Adjustments => Some(TaxSection::MoreAdjustments),
        TaxSection::Itemized => Some(TaxSection::MoreItemized),
        TaxSection::Credits => Some(TaxSection::MoreCredits),
        _ => None,
    }
}
// Start at an unfinished hub, with every later section unfinished too. Each
// destination is planned independently of the production navigation helpers.
fn continuation_draft(section: TaxSection, destination: TaxSection) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.finished_sections.retain(|s| (*s as i32) < section as i32);
    if section == TaxSection::Itemized {
        draft.deductions.as_mut().unwrap().choice =
            DeductionSelection::Itemized { facts: empty_facts() };
    }
    match destination {
        TaxSection::Adjustments => {
            draft.checklist.adjustments = AdjustmentTopics::STUDENT_LOAN.0;
        }
        TaxSection::Deductions => draft.checklist.deductions = DeductionTopics::GIFTS.0,
        TaxSection::Credits => draft.checklist.credit_topics = CreditTopics::MARKETPLACE.0,
        TaxSection::OtherTaxes => {
            draft.checklist.other_taxes = OtherTaxTopics::HOUSEHOLD_EMPLOYEE.0;
        }
        other => panic!("not a continuation destination: {other:?}"),
    }
    draft
}

#[test]
fn hub_continue_reaches_each_later_section_on_an_unfinished_walk() {
    use TaxSection::*;
    // Gate fixes batch A: the income, adjustments, credits and other taxes
    // sections are shown on every return (each asks its less-common page),
    // so Continue reaches the next of them unless a deductions plan
    // intervenes.
    let cases: &[(TaxSection, &[TaxSection])] = &[
        (Income, &[Adjustments]),
        (Adjustments, &[Deductions, Credits]),
        (Itemized, &[Credits]),
        (Credits, &[OtherTaxes]),
    ];
    for &(section, destinations) in cases {
        for &destination in destinations {
            let mut session = Interview::new(TEST_USER, continuation_draft(section, destination));
            let request = back(&session.draft, section as i32);
            let hub = hub_of(send(&mut session, request));
            let reply = send(&mut session, hub_action(&hub));
            assert!(
                match destination {
                    Adjustments => matches!(reply, Reply::Edit(P::SaveStudentLoan { .. })),
                    Deductions => matches!(reply, Reply::Edit(P::SaveDeductions { .. })),
                    Credits => matches!(reply, Reply::Edit(P::SaveMarketplace { .. })),
                    OtherTaxes => matches!(reply, Reply::Edit(P::SaveOtherTaxes { .. })),
                    _ => unreachable!(),
                },
                "{section:?} should continue to {destination:?}: {reply:?}"
            );
            assert!(session.draft.finished_sections.contains(&section));
            assert!(
                !session.draft.finished_sections.contains(&destination),
                "Continue must not also finish {destination:?}"
            );
        }
    }
}

#[test]
fn hub_continue_opens_the_next_summary_when_its_topic_was_visited() {
    for (section, destination, topic) in [
        (TaxSection::Income, TaxSection::Adjustments, TaxSection::StudentLoan),
        (TaxSection::Adjustments, TaxSection::Credits, TaxSection::Marketplace),
        (TaxSection::Itemized, TaxSection::Credits, TaxSection::Marketplace),
    ] {
        let mut draft = continuation_draft(section, destination);
        // Saving a planned topic with nothing entered visits it; the tick
        // remains, but the walkthrough must not open that editor again. The
        // destination's less-common page was passed too (gate fixes batch A).
        draft.finished_sections.push(topic);
        draft.finished_sections.extend(hub_topics_last(destination));
        let mut session = Interview::new(TEST_USER, draft);
        let request = back(&session.draft, section as i32);
        let hub = hub_of(send(&mut session, request));
        let reply = send(&mut session, hub_action(&hub));
        assert!(
            matches!(reply, Reply::Response(P::Hub { page, .. }) if page == destination as i32),
            "{section:?} should reach the {destination:?} summary: {reply:?}"
        );
    }
}

#[test]
fn hub_continue_leaves_the_section_even_when_a_topic_is_suggested() {
    for (section, topic) in [
        (TaxSection::Income, TaxSection::Retirement),
        (TaxSection::Adjustments, TaxSection::StudentLoan),
        (TaxSection::Itemized, TaxSection::GiftCarryovers),
        (TaxSection::Credits, TaxSection::Marketplace),
    ] {
        // Each continues to the next shown section (gate fixes batch A).
        let next = match section {
            TaxSection::Income => TaxSection::Adjustments,
            TaxSection::Credits => TaxSection::OtherTaxes,
            _ => TaxSection::Credits,
        };
        let mut draft = continuation_draft(section, next);
        match section {
            TaxSection::Income => draft.checklist.income = IncomeTopics::RETIREMENT.0,
            TaxSection::Adjustments => {
                draft.checklist.adjustments = AdjustmentTopics::STUDENT_LOAN.0;
            }
            TaxSection::Itemized => {
                draft.checklist.deductions = DeductionTopics::GIFT_CARRYOVERS.0;
            }
            TaxSection::Credits => {
                draft.checklist.credit_topics = CreditTopics::MARKETPLACE.0;
            }
            _ => unreachable!(),
        }
        let mut session = Interview::new(TEST_USER, draft);
        let request = back(&session.draft, section as i32);
        let hub = hub_of(send(&mut session, request));
        assert_eq!(suggestion(&hub), topic);
        let reply = send(&mut session, hub_action(&hub));
        accepted(&reply);
        assert!(
            session.draft.finished_sections.contains(&section),
            "{section:?} Continue must leave the suggested {topic:?}: {reply:?}"
        );
        assert!(!session.draft.finished_sections.contains(&topic));
    }
}

#[test]
fn hub_continue_enters_derived_topics_without_checklist_ticks() {
    for education in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        draft.finished_sections.retain(|s| (*s as i32) < TaxSection::Income as i32);
        draft.people.as_mut().unwrap().taxpayer.full_time_student = education;
        let mut session = Interview::new(TEST_USER, draft);
        let wages = if education { w2(6_000_000) } else { w2_with_box_12_w(6_000_000, 100_000) };
        let request = save_topic(
            &session.draft,
            TaxSection::Wages,
            vec![document(0, IncomeDocument::W2(wages))],
        );
        let hub = income_hub(send_past_less_common(&mut session, request));
        assert_eq!(session.draft.checklist.adjustments, 0);
        assert_eq!(session.draft.checklist.credit_topics, 0);
        let mut reply = send(&mut session, hub_action(&hub));
        // The adjustments section comes first on every return (gate fixes
        // batch A); for the student its walk is only More adjustments.
        if education {
            let hub = hub_of(past_less_common(&mut session, reply));
            reply = send(&mut session, hub_action(&hub));
        }
        assert!(
            if education {
                matches!(reply, Reply::Edit(P::SaveEducation { .. }))
            } else {
                matches!(reply, Reply::Edit(P::SaveHsa { .. }))
            },
            "education={education}: {reply:?}"
        );
    }
}

#[test]
fn hub_continue_walks_a_new_return_through_saved_pages() {
    // Start from actual saved setup, filer and checklist pages. No later
    // completion marks are supplied by a pre-completed fixture.
    let (mut session, reply) = walked_from_the_checklist(Checklist {
        income: IncomeTopics::WAGES.0,
        adjustments: AdjustmentTopics::STUDENT_LOAN.0,
        credit_topics: CreditTopics::MARKETPLACE.0,
        payments: PaymentTopics::ESTIMATED.0,
        ..Checklist::NONE
    });
    let reply = save_opened_topic(
        &mut session,
        &reply,
        vec![document(0, IncomeDocument::W2(w2(6_000_000)))],
    );
    let income = income_hub(past_less_common(&mut session, reply));
    let Reply::Edit(student_loan @ P::SaveStudentLoan { .. }) =
        send(&mut session, hub_action(&income))
    else {
        panic!("Income Continue must open the planned student loan page")
    };
    // Each walk ends with its section's less-common page, passed with "None
    // of these" (gate fixes batch A).
    let adjustments = hub_of(send_past_less_common(&mut session, student_loan));
    let Reply::Edit(marketplace @ P::SaveMarketplace { .. }) =
        send(&mut session, hub_action(&adjustments))
    else {
        panic!("Adjustments Continue must skip standard deductions and open Marketplace")
    };
    let credits = hub_of(send_past_less_common(&mut session, marketplace));
    let Reply::Edit(other_taxes @ P::SaveOtherTaxes { .. }) =
        send(&mut session, hub_action(&credits))
    else {
        panic!("Credits Continue must open Other taxes, asked on every return")
    };
    let Reply::Edit(payments @ P::SavePayments { .. }) = send(&mut session, other_taxes) else {
        panic!("Other taxes must open Payments")
    };
    assert!(matches!(send(&mut session, payments), Reply::Response(P::Review { .. })));
}

#[test]
fn hub_continue_from_an_earlier_section_does_not_use_the_resume_destination() {
    let mut draft = complete(FederalStatus::Single);
    draft.finished_sections.retain(|s| *s != TaxSection::Payments);
    draft.finished_sections.push(TaxSection::Marketplace);
    draft.checklist.credit_topics = CreditTopics::MARKETPLACE.0;
    draft.checklist.payments = PaymentTopics::ESTIMATED.0;
    let mut session = Interview::new(TEST_USER, draft);
    assert!(matches!(
        send(&mut session, P::OpenInterview { user: TEST_USER }),
        Reply::Edit(P::SavePayments { .. })
    ));
    let request = back(&session.draft, PAGE_INCOME);
    let income = income_hub(send(&mut session, request));
    let unchanged = session.draft.clone();
    let reply = send(&mut session, hub_action(&income));
    assert!(
        matches!(reply, Reply::Response(P::Hub { page: PAGE_ADJUSTMENTS, .. })),
        "Income Continue follows the shown Adjustments section even when resume opens Payments: {reply:?}"
    );
    assert_eq!(session.draft, unchanged);
}

#[test]
fn hub_action_switches_when_the_task_completes_or_reopens_the_walk() {
    let (mut session, hub) = at_the_income_hub();
    assert!(matches!(hub_action(&hub), P::ContinueIncome { .. }));
    // Continuing walks each later section's less-common page (gate fixes
    // batch A) to the review.
    let reply = send(&mut session, hub_action(&hub));
    walk_on_to_review(&mut session, reply);
    let request = back(&session.draft, PAGE_INCOME);
    let hub = hub_of(send(&mut session, request));
    assert!(matches!(hub_action(&hub), P::ReviewAccepted { .. }));
    let unchanged = session.draft.clone();
    assert!(matches!(send(&mut session, hub_action(&hub)), Reply::Response(P::Review { .. })));
    assert_eq!(session.draft, unchanged);
    // A newly accepted checklist answer reopens work; no frontend history is involved.
    let checklist = from_review(&mut session, TaxSection::Checklist);
    let mut value = checklist_of(&checklist);
    value.adjustments = AdjustmentTopics::STUDENT_LOAN.0;
    let request = tick(checklist, value);
    accepted(&send(&mut session, request));
    let request = back(&session.draft, PAGE_INCOME);
    let reopened = hub_of(send(&mut session, request));
    assert!(matches!(hub_action(&reopened), P::ContinueIncome { .. }));
}

#[test]
fn hub_continue_rejects_wrong_identity_stale_revision_and_unavailable_sections() {
    let (mut session, hub) = at_the_income_hub();
    let request = hub_action(&hub);
    let mut wrong_user = request.clone();
    if let P::ContinueIncome { user, .. } = &mut wrong_user {
        *user += 1;
    }
    let unchanged = session.draft.clone();
    assert!(session.handle(wrong_user).is_none());
    let mut stale = request.clone();
    if let P::ContinueIncome { revision, .. } = &mut stale {
        *revision -= 1;
    }
    assert!(refusal(send(&mut session, stale)).contains("changed"));
    let mut wrong_return = request.clone();
    if let P::ContinueIncome { cookie, .. } = &mut wrong_return {
        cookie.push_str("wrong");
    }
    assert!(refusal(send(&mut session, wrong_return)).contains("return"));
    let not_itemizing = P::ContinueItemized {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
    };
    assert!(refusal(send(&mut session, not_itemizing)).contains("not available"));
    assert_eq!(session.draft, unchanged);
    let mut reply = send(&mut session, request.clone());
    assert!(refusal(send(&mut session, request)).contains("changed"));
    // Each later section walks its less-common page (gate fixes batch A);
    // with the walk at the review, Continue is no longer offered.
    loop {
        reply = match past_less_common(&mut session, reply) {
            Reply::Response(hub @ P::Hub { .. }) => send(&mut session, hub_action(&hub)),
            Reply::Response(P::Review { .. }) => break,
            Reply::Edit(page @ P::SavePayments { .. }) => send(&mut session, page),
            other => panic!("unexpected {other:?}"),
        };
    }
    let completed = session.draft.clone();
    let no_longer_offered = P::ContinueIncome {
        user: TEST_USER,
        cookie: completed.cookie.clone(),
        revision: completed.revision,
    };
    assert!(refusal(send(&mut session, no_longer_offered)).contains("not available"));
    assert_eq!(session.draft, completed);
}

#[test]
fn hub_action_schema_keeps_continuation_separate_from_editor_next() {
    let source = include_str!("../../core/protocol.rs");
    for name in ["ContinueIncome", "ContinueAdjustments", "ContinueItemized", "ContinueCredits"] {
        let block = declaration(source, &alloc::format!("    {name} {{"));
        assert_eq!(block.matches("#[libertas_hidden]").count(), 3);
        assert!(block.contains("#[libertas_fixed(\"current-user\")]"));
        assert!(block.contains("#[libertas_copy_from(\"$.cookie\")]"));
        assert!(block.contains("#[libertas_copy_from(\"$.revision\")]"));
        assert!(!block.contains("section:"));
    }
    let template: serde_json::Value =
        serde_json::from_str(include_str!("../../../display.json")).unwrap();
    let rules = template["rules"].as_array().unwrap();
    assert!(
        rules.iter().any(|r| r["select"] == "^<Hub>"
            && r["button_labels"]["actions"]["source"] == "single_action")
    );
    let panes =
        rules.iter().find(|r| r["select"] == "^<Hub>" && r["right_pane"].is_array()).unwrap();
    assert_eq!(panes["detail"], serde_json::json!(["^<Hub.topics.items>[]"]));
}

// The refresh request of the hub's master-detail arrangement, built from the
// hub as shown: every field copies from the hub, none is editable.
fn reload(hub: &P) -> P {
    let P::Hub { cookie, revision, page, review, .. } = hub else {
        panic!("expected a hub, got {hub:?}")
    };
    P::OpenHub {
        user: TEST_USER,
        cookie: cookie.clone(),
        revision: *revision,
        page: *page,
        review: *review,
    }
}
#[test]
fn open_hub_reloads_the_topics_and_stores_nothing() {
    // Declaration: a request that only navigates, never offered as an
    // action, answered by the hub; display.json names it as the refresh.
    let protocol = include_str!("../../core/protocol.rs");
    let before_open = &protocol[..protocol.find("    OpenHub {").unwrap()];
    let attrs = &before_open[before_open.rfind("/// Reload the topics").unwrap()..];
    assert!(attrs.contains("#[libertas_next_response(\"Hub,Problem\")]"));
    assert!(attrs.contains("#[libertas_exclude_ui]"));
    let open = declaration(protocol, "    OpenHub {");
    assert_eq!(open.matches("#[libertas_hidden]").count(), 5, "{open}");
    let display: serde_json::Value =
        serde_json::from_str(include_str!("../../../display.json")).unwrap();
    let rule = display["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["select"] == "^<Hub>" && r["right_pane"].is_array())
        .unwrap();
    assert_eq!(rule["select"], "^<Hub>");
    assert_eq!(rule["refresh"], "OpenHub");
    // Server: the reply is the hub as it is now, and nothing is stored.
    let (mut s, hub) = at_the_income_hub();
    let stored = s.draft.clone();
    let Reply::Response(again) = send(&mut s, reload(&hub)) else { panic!("expected the hub") };
    assert_eq!(again, hub);
    assert_eq!(s.draft, stored);
    // A hub shown before a save carries an earlier revision; the request is
    // admitted by identity and answers with the current one.
    let mut earlier = reload(&hub);
    let P::OpenHub { revision, .. } = &mut earlier else { unreachable!() };
    *revision -= 1;
    let Reply::Response(current) = send(&mut s, earlier) else { panic!("expected the hub") };
    assert_eq!(current, hub);
    assert_eq!(s.draft, stored);
    // Only a hub has topics to reload.
    let mut forged = reload(&hub);
    let P::OpenHub { page, .. } = &mut forged else { unreachable!() };
    *page = TaxSection::Setup as i32;
    assert!(refusal(send(&mut s, forged)).contains("not available"));
    assert_eq!(s.draft, stored);
}

// M3 acceptance fix (2026-09-26): the review's calculated draft return
// lists every form's lines, grouped by form.
pub(in crate::tests) fn review_result(d: FederalDraft) -> FederalResult {
    let mut s = Interview::new(TEST_USER, d);
    match send(&mut s, P::ReviewAccepted { user: TEST_USER }) {
        Reply::Response(P::Review { .. }) => reviewed(&s),
        other => panic!("expected the review, got {other:?}"),
    }
}
fn form_of(row: &FederalAmount) -> TaxForm {
    row.form_line.as_ref().unwrap().form
}

#[test]
fn review_lists_schedule_lines_grouped_by_form() {
    // The M3-F1 self-employed rental return: its 1040 lines come first,
    // unchanged, then the other forms' tagged lines in the review's form
    // order (Schedules 1, C, E, SE, then Forms 4562 and 8995). Hand-computed
    // in src/tests/business/business.rs m3_self_employed_rental_return_matches_hand_computation:
    // Schedule C line 9 car and truck 8,000 x $0.725 + 6,000 x $0.76 =
    // $10,360; line 31 net profit $88,000 - $27,660 - $1,000 home office =
    // $59,340. Schedule E line 21/22 and line 26: rent $24,000 less $27,000
    // of expenses and depreciation = -$3,000, fully allowed (MAGI $55,148,
    // under $100,000). Schedule SE line 12: $59,340 x 92.35% = $54,800 x
    // 15.3% = $8,384. Form 8995 line 15: the smaller of line 10 ($55,148 x
    // 20% = $11,030) and line 14 (($52,148 AGI - $16,100) x 20% = $7,210).
    // The rental's mid-month depreciation, $220,000 / 27.5 = $8,000, is on
    // Schedule E line 18 alone: its building was placed in service in 2021,
    // so the rental files no Form 4562 (i4562 "Who Must File"). The 2026
    // computer does: $2,400 cost x 100% bonus (acquired after January 19,
    // 2025) = $2,400 on Part II line 14a and on line 22, the same $2,400 as
    // Schedule C line 13. Schedule 2 line 4 is the $8,384 self-employment tax.
    let d = crate::tests::business::business::m3_self_employed_rental_draft();
    let computed = estimate(&d);
    let rows = review_result(d).return_lines;
    let typed = computed.return_lines.len();
    assert_eq!(rows[..typed], computed.return_lines[..]);
    assert!(rows[..typed].iter().all(|r| form_of(r) == TaxForm::Form1040));
    let rest: Vec<(alloc::string::String, i64)> =
        rows[typed..].iter().map(|r| (status_text(&r.label), r.amount)).collect();
    let position = |label: &str, amount: i64| {
        rest.iter()
            .position(|row| row.0 == label && row.1 == amount)
            .unwrap_or_else(|| panic!("{label} {amount} missing from {rest:?}"))
    };
    let expected = [
        ("Schedule 1 3 — Schedule 1 business income", 5_934_000),
        ("Schedule 2 4 — Self-employment tax", 838_400),
        ("Schedule C 9 — Business 2 car and truck", 1_036_000),
        ("Schedule C 13 — Business 2 depreciation", 240_000),
        ("Schedule C 31 — Business 2 net profit", 5_934_000),
        ("Schedule E 18 — Rental 3 line 18 depreciation", 800_000),
        ("Schedule E 26 — Schedule E rental royalty total", -300_000),
        ("Schedule SE 12 — Taxpayer self-employment tax", 838_400),
        ("Form 4562 14a — Business 2 asset 0 bonus", 240_000),
        ("Form 4562 22 — Business 2 depreciation total", 240_000),
        ("Form 8995 15 — QBI limited deduction", 721_000),
    ];
    let positions: Vec<usize> =
        expected.iter().map(|(label, amount)| position(label, *amount)).collect();
    assert!(positions.windows(2).all(|p| p[0] < p[1]), "{rest:?}");
    // Forms appear in the review's order, never interleaved, and each
    // form's lines in printed order (one business, one property, one
    // person here).
    let forms: Vec<(u8, TaxForm)> = rows[typed..]
        .iter()
        .map(|r| {
            let form = form_of(r);
            let group = match form {
                TaxForm::Schedule1 => 1,
                TaxForm::Schedule2 => 2,
                TaxForm::ScheduleC => 6,
                TaxForm::ScheduleE => 8,
                TaxForm::ScheduleSe => 9,
                _ => 10,
            };
            (group, form)
        })
        .collect();
    assert!(forms.windows(2).all(|f| f[0] <= f[1]), "{rest:?}");
    let lines = |form: TaxForm| -> Vec<&str> {
        rows[typed..]
            .iter()
            .filter(|r| form_of(r) == form)
            .map(|r| r.form_line.as_ref().unwrap().line.as_str())
            .collect()
    };
    assert_eq!(
        lines(TaxForm::ScheduleC),
        vec![
            "1", "3", "5", "7", "8", "9", "11", "13", "15", "18", "22", "23", "24b", "25", "28",
            "29", "30", "31"
        ]
    );
    assert_eq!(
        lines(TaxForm::ScheduleE),
        vec![
            "3", "9", "11", "12", "14", "16", "17", "18", "20", "21", "22", "23a", "23c", "23d",
            "23e", "25", "26"
        ]
    );
    assert_eq!(lines(TaxForm::ScheduleSe), vec!["2", "3", "4a", "4c", "12", "13"]);
    assert_eq!(
        lines(TaxForm::Form8995),
        vec!["1", "2", "4", "5", "10", "11", "13", "14", "15", "16", "17"]
    );
    assert_eq!(lines(TaxForm::Schedule2), vec!["4", "15", "21"]);
    // Only the computer's Form 4562: no 19i or line 17 row for the 2021
    // rental building.
    assert_eq!(lines(TaxForm::Form4562), vec!["14a", "22"]);
    assert!(rest.iter().all(|(label, _)| !label.contains("rental_3_depreciation")));
    // The grouping: Form 1040, Schedules 1, 2, 3, A, B, C, D, E, SE, then
    // the other forms in declaration order; within a form one copy (one
    // business) at a time in printed line order, and two lines on one form
    // line (two businesses) both listed.
    let mut result = computed.clone();
    let tagged = |form: TaxForm, line: &str, id: &str, amount: i64| CalculationLine {
        id: id.into(),
        amount,
        kind: crate::LineKind::Money,
        inputs: vec![],
        rule: alloc::format!("US-2026.{id}"),
        source: "test".into(),
        explanation: crate::Message::new("TEXT", vec![crate::MessageArg::Text("test".into())]),
        form_line: Some(FormLine::new(form, line)),
    };
    result.lines = vec![
        tagged(TaxForm::Form8995, "15", "qbi_deduction", 721_000),
        tagged(TaxForm::ScheduleSe, "12", "self_employment_tax", 838_400),
        tagged(TaxForm::ScheduleE, "26", "rental_income", -300_000),
        tagged(TaxForm::ScheduleC, "31", "business_2_net_profit", 5_934_000),
        tagged(TaxForm::Schedule1A, "38", "senior_deduction", 100),
        tagged(TaxForm::Schedule1, "10", "additional_income", 5_634_000),
        tagged(TaxForm::ScheduleC, "31", "business_4_net_profit", 100),
        tagged(TaxForm::ScheduleC, "1", "business_4_gross_receipts", 200),
        tagged(TaxForm::ScheduleC, "1", "business_2_gross_receipts", 8_800_000),
        tagged(TaxForm::Form1040, "8", "additional_income", 5_634_000),
    ];
    let rows = crate::draft_return(&result);
    let rest: Vec<alloc::string::String> =
        rows[typed..].iter().map(|r| status_text(&r.label)).collect();
    assert_eq!(
        rest,
        vec![
            "Schedule 1 10 — Additional income",
            "Schedule C 1 — Business 2 gross receipts",
            "Schedule C 31 — Business 2 net profit",
            "Schedule C 1 — Business 4 gross receipts",
            "Schedule C 31 — Business 4 net profit",
            "Schedule E 26 — Rental income",
            "Schedule SE 12 — Self-employment tax",
            "Schedule 1-A 38 — Senior deduction",
            "Form 8995 15 — QBI deduction",
        ]
    );
    // A calculation line on a typed Form 1040 line is not listed twice.
    assert_eq!(
        rows.iter().filter(|r| r.form_line == Some(FormLine::new(TaxForm::Form1040, "8"))).count(),
        1
    );
}

#[test]
fn schedule_1_lines_are_tagged() {
    // M3-F1 (src/tests/business/business.rs m3_self_employed_rental_return_matches_hand_computation):
    // line 3 is Schedule C line 31, $88,000 - $27,660 - $1,000 = $59,340;
    // line 5 is Schedule E line 26, $24,000 - $27,000 = -$3,000 (allowed in
    // full); line 15 is half of Schedule SE line 12, $8,384 / 2 = $4,192;
    // line 10 is 59,340 - 3,000 = $56,340 and line 26 is $4,192.
    let rows = review_result(crate::tests::business::business::m3_self_employed_rental_draft())
        .return_lines;
    let schedule_1: Vec<(&str, i64)> = rows
        .iter()
        .filter(|r| form_of(r) == TaxForm::Schedule1)
        .map(|r| (r.form_line.as_ref().unwrap().line.as_str(), r.amount))
        .collect();
    assert_eq!(
        schedule_1,
        vec![
            ("3", 5_934_000),
            ("5", -300_000),
            ("10", 5_634_000),
            ("15", 419_200),
            ("26", 419_200),
        ]
    );
}

#[test]
fn review_hides_zero_lines_outside_form_1040() {
    // The M3-F2 itemizing homeowner: both loans have Forms 1098, so Schedule
    // A line 8b (seller-financed interest) is a $0 calculation line; the
    // review drops it but keeps 8a, 8c and 8e, and keeps Form 1040's zero
    // lines (line 2a tax-exempt interest, nothing entered).
    let d = crate::tests::deductions::deductions::m3_itemizing_homeowner_draft();
    let computed = estimate(&d);
    let line_8b = FormLine::new(TaxForm::ScheduleA, "8b");
    assert!(computed.lines.iter().any(|l| l.form_line.as_ref() == Some(&line_8b) && l.amount == 0));
    let rows = review_result(d).return_lines;
    assert!(!rows.iter().any(|r| r.form_line.as_ref() == Some(&line_8b)));
    assert!(rows.iter().any(|r| status_text(&r.label).starts_with("Schedule A 8a — ")));
    assert!(rows.iter().any(|r| status_text(&r.label).starts_with("Schedule A 8e — ")));
    assert!(
        rows.iter()
            .filter(|r| form_of(r) != TaxForm::Form1040)
            .all(|r| r.amount != 0 || r.money.is_none())
    );
    let line_2a = rows.iter().find(|r| status_text(&r.label).starts_with("1040 2a — ")).unwrap();
    assert_eq!(line_2a.amount, 0);
    assert_eq!(
        rows.iter().filter(|r| form_of(r) == TaxForm::Form1040).count(),
        computed.return_lines.len()
    );
}

// T2-34: gifts of goods, publicly traded stock and vehicles on the itemized
// hub's gifts of property and carryovers page (Form 8283 Section A, Form
// 1098-C, Schedule A lines 11-14).

// The hub's gifts page, reached with the given route and cash gifts.
pub(in crate::tests) fn at_the_gifts_page(route: DeductionRoute, gifts: i64) -> Interview {
    let mut s = at_the_deductions_page(FederalStatus::Single).0;
    let request = save_deductions(&s.draft, route, gifts);
    itemized_hub(send_past_less_common(&mut s, request));
    s
}
pub(in crate::tests) fn gifts(
    goods: Vec<PropertyGift>,
    stock: Vec<PropertyGift>,
    vehicles: Vec<PropertyGift>,
) -> GiftCarryovers {
    GiftCarryovers {
        gifts: goods.into_iter().chain(stock).chain(vehicles).map(Into::into).collect(),
        appraisals: vec![],
        appraisal_choices: vec![],
        basis_election: false,
        carryovers: vec![],
        volunteer: vec![],
        less_common: vec![],
    }
}
pub(in crate::tests) fn goods_gift(kind: GoodsKind, cost: i64, value: i64) -> PropertyGift {
    let mut g = used_goods(value);
    g.description = "Bags of used items".into();
    g.charity = "Synthetic Thrift Charity".into();
    g.given = 20260321;
    g.acquired = 20210615;
    g.basis = cost;
    g.kind = GiftKind::Goods { kind, condition: GoodsCondition::Good };
    g
}
fn stock_gift(acquired: u32, basis: i64, value: i64) -> PropertyGift {
    let mut g = public_stock_gift();
    g.description = "40 shares of Example Corp.".into();
    g.charity = "Synthetic Public Charity".into();
    g.given = 20261115;
    g.acquired = acquired;
    g.basis = basis;
    g.value = value;
    g
}
pub(in crate::tests) fn vehicle_gift(value: i64, treatment: VehicleUse) -> PropertyGift {
    let mut g = donated_personal_vehicle(treatment);
    g.description = "2014 Example sedan".into();
    g.charity = "Synthetic Vehicle Charity".into();
    g.given = 20260601;
    g.acquired = 20140301;
    g.basis = 2_000_000;
    g.value = value;
    g.kind = match g.kind {
        GiftKind::Vehicle { treatment, .. } => {
            GiftKind::Vehicle { vin: "TESTVIN0000000001".into(), treatment }
        }
        _ => unreachable!(),
    };
    g
}
pub(in crate::tests) fn sold(proceeds: i64) -> VehicleUse {
    VehicleUse::Sold { sold: 20260710, proceeds }
}

#[test]
fn form_8283_fixture_goods_stock_and_vehicle_through_the_page() {
    // Hand-worked single return, wages $60,000, no adjustments: AGI $60,000.
    // Itemizing (Schedule A line 19), cash gifts $1,000; the page enters
    // clothing ($300 value, $1,200 cost), household items ($1,700 value,
    // $4,000 cost), 40 purchased shares held since 2020 (basis $2,000, value
    // $15,000: long-term appreciated, 30% class at value) and a car the
    // charity sold for $2,600 (value $4,000; box 4c above $500, so the claim is
    // the smaller of value and proceeds, $2,600).
    // Pub 526 limits: cash $1,000 <= 60% x 60,000; the 50% class (goods
    // $2,000 + car $2,600 = $4,600) <= 50% x 60,000 - $1,000 = $29,000;
    // stock $15,000 <= min(30% x 60,000 = $18,000, $30,000 - $1,000 - $4,600 =
    // $24,400). Allowed $20,600; the 0.5% floor min($20,600, $300) = $300 is
    // charged to the 30% stock class first: stock $14,700, noncash $4,600,
    // cash $1,000; no percentage limit is exceeded, so nothing carries.
    // Schedule A total $20,300; taxable income $60,000 - $20,300 = $39,700.
    // Form 8283 Section A (noncash $19,600 > $500, every group <= $5,000,
    // the sold car stays in Section A): $2,000 + $2,600 + $15,000 = $19,600.
    let mut s = at_the_gifts_page(DeductionRoute::Itemize, 100_000);
    let request = save_gift_carryovers(
        &s.draft,
        gifts(
            vec![
                goods_gift(GoodsKind::Clothing, 120_000, 30_000),
                goods_gift(GoodsKind::Household, 400_000, 170_000),
            ],
            vec![stock_gift(20200302, 200_000, 1_500_000)],
            vec![vehicle_gift(400_000, sold(260_000))],
        ),
    );
    itemized_hub(send_past_less_common(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 1_000);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 4_600);
    assert_eq!(value(&r, "charity_capital_gain_thirty_current_deduction"), 14_700);
    assert_eq!(value(&r, "charity_capital_gain_thirty_return_year_carryforward"), 0);
    assert_eq!(value(&r, "charity_property_4_claim"), 2_600);
    assert_eq!(value(&r, "form_8283_section_a_value"), 19_600);
    assert!(!r.lines.iter().any(|l| l.id == "form_8283_section_b_value"));
    assert_eq!(value(&r, "deduction"), 20_300);
    assert_eq!(value(&r, "taxable_income"), 39_700);
}

#[test]
fn property_gifts_map_onto_the_form_8283_records_and_reopen() {
    let mut s = at_the_gifts_page(DeductionRoute::Compare, 0);
    let value = gifts(
        vec![goods_gift(GoodsKind::Books, 50_000, 20_000)],
        vec![
            stock_gift(20200302, 200_000, 1_500_000),
            // Held exactly one year: not long-term, the smaller of value and basis.
            stock_gift(20251115, 100_000, 300_000),
        ],
        vec![
            vehicle_gift(300_000, VehicleUse::NeedyPerson),
            vehicle_gift(
                150_000,
                VehicleUse::SignificantUse {
                    details: "Delivers meals weekly for two years".into(),
                },
            ),
        ],
    );
    let request = save_gift_carryovers(&s.draft, value.clone());
    itemized_hub(send_past_less_common(&mut s, request));
    let ded = s.draft.deductions.as_ref().unwrap();
    assert_eq!(ded.property_gifts.len(), 5);
    let book = &ded.property_gifts[0];
    assert!(matches!(book.kind, GiftKind::Goods { kind: GoodsKind::Books, .. }));
    assert_eq!((book.basis, book.value), (50_000, 20_000));
    assert!(book.appraisal.is_none());
    assert_eq!(
        ded.property_gifts[1..3].iter().map(|g| g.claim(false)).collect::<Vec<_>>(),
        vec![(CharityClass::CapitalGain30, 1_500_000), (CharityClass::Property50, 100_000),]
    );
    assert!(matches!(
        ded.property_gifts[3].kind,
        GiftKind::Vehicle { treatment: VehicleUse::NeedyPerson, .. }
    ));
    assert_eq!(ded.property_gifts[4].claim(false).1, 150_000);
    // Reopened, the page shows exactly what was saved.
    let P::SaveGiftCarryovers { value: reopened, .. } =
        open_itemized_topic(&mut s, TaxSection::GiftCarryovers)
    else {
        panic!("expected the gifts page")
    };
    assert_eq!(reopened, value);
    // Standard clears the gifts along with the itemized facts.
    let request = save_deductions(&s.draft, DeductionRoute::Standard, 0);
    send(&mut s, request);
    let ded = s.draft.deductions.as_ref().unwrap();
    assert!(ded.property_gifts.is_empty());
    assert!(ded.property_gifts.is_empty());
    assert!(ded.property_gifts.is_empty());
}

#[test]
fn gift_page_rules_reject_on_both_sides() {
    let mut s = at_the_gifts_page(DeductionRoute::Compare, 0);
    let before = s.draft.clone();
    let request = save_gift_carryovers(
        &s.draft,
        gifts(vec![goods_gift(GoodsKind::Clothing, 10_000, 10_001)], vec![], vec![]),
    );
    rejected_with(send(&mut s, request), "GIFT_VALUE_ABOVE_COST");
    let mut vehicle = vehicle_gift(2_000_001, sold(100_000));
    // The canonical owner permits appreciated vehicles and derives the applicable basis reduction.
    assert_eq!(vehicle.claim(false).1, 100_000);
    vehicle.benefit_received = Some(2_000_001);
    let request = save_gift_carryovers(&s.draft, gifts(vec![], vec![], vec![vehicle]));
    rejected_with(send(&mut s, request), "GIFT_BENEFIT_ABOVE_VALUE");
    let mut late = stock_gift(20261116, 100_000, 200_000);
    late.given = 20261115;
    let request = save_gift_carryovers(&s.draft, gifts(vec![], vec![late], vec![]));
    rejected_with(send(&mut s, request), "GIFT_BOUGHT_AFTER_GIVEN");
    let early = VehicleUse::Sold { sold: 20260531, proceeds: 100_000 };
    let request =
        save_gift_carryovers(&s.draft, gifts(vec![], vec![], vec![vehicle_gift(300_000, early)]));
    rejected_with(send(&mut s, request), "VEHICLE_SOLD_BEFORE_GIVEN");
    assert_eq!(s.draft, before);
}

#[test]
fn similar_goods_above_5000_save_with_actual_section_b_requirements() {
    // Form 8283 Section B starts above $5,000 per similar-property group
    // across all gifts: two clothing gifts totalling exactly $5,000 stay in
    // Section A; one more cent needs a qualified appraisal; missing substantiation omits the gifts.
    // A different group does not add to the clothing total.
    let mut s = at_the_gifts_page(DeductionRoute::Itemize, 0);
    let request = save_gift_carryovers(
        &s.draft,
        gifts(
            vec![
                goods_gift(GoodsKind::Clothing, 400_000, 300_000),
                goods_gift(GoodsKind::Clothing, 400_000, 200_001),
            ],
            vec![],
            vec![],
        ),
    );
    itemized_hub(send_past_less_common(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "charity_deduction"), 0);
    assert!(noted(&r, "WARN_GIFT_LEFT_OUT"));
    let request = save_gift_carryovers(
        &s.draft,
        gifts(
            vec![
                goods_gift(GoodsKind::Clothing, 400_000, 300_000),
                goods_gift(GoodsKind::Clothing, 400_000, 200_000),
                goods_gift(GoodsKind::Household, 400_000, 100_000),
            ],
            vec![],
            vec![],
        ),
    );
    itemized_hub(send_past_less_common(&mut s, request));
    // $6,000 of goods: Section A $6,000; AGI $60,000, floor $300: $5,700.
    let r = supported(&s.draft);
    assert_eq!(value(&r, "form_8283_section_a_value"), 6_000);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 5_700);
}

// ---- T2-35: the sale folds and the business-property losses kind ----

pub(in crate::tests) fn continuing_page_asset(disposal: crate::AssetDisposal) -> Asset {
    Asset {
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost: 1_000_000,
        credit_reduction: 0,
        placed: 20240301,
        class: AssetClass::Five,
        business_percent: 100,
        acquired: None,
        history: AssetHistory::Continuing {
            prior_depreciation: 520_000,
            first_year_179: 0,
            first_year_bonus: 0,
            carryforward: 0,
            mid_quarter: false,
        },
        disposal,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    }
}
fn page_sale(date: u32, proceeds: i64) -> crate::AssetDisposal {
    crate::AssetDisposal::Disposed {
        date,
        how: crate::DisposalKind::Sold {
            livestock: crate::Livestock::No,
            proceeds,
            selling_expenses: 0,
            payment: crate::SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
        },
    }
}

#[test]
fn asset_disposal_fold_maps_onto_the_asset_and_back() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(15_000_000);
    b.assets = vec![continuing_page_asset(page_sale(20260715, 1_200_000))];
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    assert_eq!(data.assets[0].disposal, page_sale(20260715, 1_200_000));
    assert_round_trip(&mut session, &[b]);
    // Sold above its cost, part of the gain is section 1231 gain: without
    // the business-property losses answer it cannot be characterized.
    let r = estimate(&session.draft);
    assert!(r.issues.iter().any(|i| i.section == "Section 1231 lookback"), "{r:?}");
    // The date sold must follow the date placed in service, on the page and
    // on save.
    let before = session.draft.clone();
    let mut early = business(15_000_000);
    early.assets = vec![continuing_page_asset(page_sale(20240301, 600_000))];
    rejected_with(save_businesses(&mut session, vec![early]), "ASSET_DISPOSED_BEFORE_PLACED");
    let mut early = business(15_000_000);
    early.vehicles = vec![Vehicle {
        method: VehicleMethod::Actual {
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            allowed_amt_carry: Vec::new(),
            cost: 3_000_000,
            credit_reduction: 0,
            acquired: None,
            placed: 20250301,
            total_miles: 10_000,
            business_miles: 8_000,
            operating_costs: 0,
            loan_interest: 0,
            passenger_automobile: true,
            history: AssetHistory::Continuing {
                prior_depreciation: 480_000,
                first_year_179: 0,
                first_year_bonus: 0,
                carryforward: 0,
                mid_quarter: false,
            },
            disposal: page_sale(20250301, 1_000_000),
            earlier_business_percent: None,
        },
    }];
    rejected_with(save_businesses(&mut session, vec![early]), "ASSET_DISPOSED_BEFORE_PLACED");
    assert_eq!(session.draft, before);
}

// Exact raw source parts; this helper builds inputs, not expected tax amounts.
pub(in crate::tests) fn patr_parts(v: &mut Patr1099, personal: i64) {
    let data = FederalCooperative {
        patronage: v.patronage,
        nonpatronage: v.nonpatronage,
        per_unit_retains: v.per_unit_retains,
        redeemed: v.redeemed,
        qualified_items: v.more.qualified_items,
        qualified_payments: v.more.qualified_payments,
        sstb_items: v.more.sstb_items,
        section_199a_g: v.more.section_199a_g,
        ..Default::default()
    };
    let mut a = crate::cooperative::automatic(&data, None);
    a.personal.patronage = personal;
    a.personal_qualification = Some(CoopQualification::default());
    // These synthetic fixtures describe original-patron notices redeemed at
    // stated value; box 5 alone would not establish those history facts.
    if v.redeemed > 0 {
        a.redemptions.push(CoopRedemption {
            kind: CoopNoticeKind::Patronage,
            distribution: CoopDistributionBox::Redeemed,
            amount: v.redeemed,
            stated_value: v.redeemed,
            holding: CoopNoticeHolding::OriginalOrInherited,
            source: "Synthetic original-patron notice redeemed at stated value".into(),
        });
    }
    v.more.allocations = vec![a.into()];
}

// T2-49: the 2027 estimated tax choice on the payments page.
#[test]
fn next_year_estimate_round_trips_through_the_payments_page() {
    let (mut s, page) = at_the_payments_page(complete(FederalStatus::Single));
    // A new return opens with the estimate not needed.
    assert_eq!(payments_value(&page).0.next_year, NextYearEstimate::NotNeeded);
    // Expected changes are stored as entered and shown again; the review's
    // draft return then lists the Form 1040-ES lines (complete()'s $6,500 of
    // withholding covers the required payment, so the worksheet ends at 14a).
    let changes = NextYearEstimate::WithChanges {
        agi: Some(9_000_000),
        deductions: None,
        credits: Some(50_000),
        withholding: None,
        farmer_fisherman_next_year: FarmerNextYear::Yes,
    };
    let entered = Payments { next_year: changes.clone(), ..no_payments() };
    match submit(&mut s, entered) {
        Reply::Response(P::Review { result, .. }) => {
            assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
            assert!(
                result
                    .return_lines
                    .iter()
                    .any(|l| status_text(&l.label).starts_with("Form 1040-ES 12c —")),
                "{:?}",
                result.return_lines
            );
        }
        other => panic!("expected the review, got {other:?}"),
    }
    assert_eq!(s.draft.payments.as_ref().unwrap().next_year, changes.clone().into());
    assert_eq!(payments_value(&open_payments(&mut s)).0.next_year, changes);
    // T2-49b: the 2027 farmer answer belongs to the worksheet; the 2025
    // answer stays No (the IRS figures the penalty).
    assert!(!s.draft.payments.as_ref().unwrap().farmer_fisherman_prior_year);
    // Not needed again clears it and the review shows no Form 1040-ES line.
    accepted(&submit(&mut s, no_payments()));
    assert_eq!(s.draft.payments.as_ref().unwrap().next_year, NextYearEstimateRecord::NotNeeded);
    assert!(!estimate(&s.draft).lines.iter().any(|l| l.id.starts_with("form_1040_es_line_")));
    // An expected amount outside the schema's range is refused on the server too.
    let before = s.draft.clone();
    let out_of_range = Payments {
        next_year: NextYearEstimate::WithChanges {
            agi: None,
            deductions: Some(-1),
            credits: None,
            withholding: None,
            farmer_fisherman_next_year: FarmerNextYear::No,
        },
        ..no_payments()
    };
    error(send(&mut s, save_payments(&before, out_of_range)));
    assert_eq!(s.draft, before);
}

// Canonical Form1095-A source lifecycle through actual public pages.
pub(in crate::tests) fn marketplace_page_of(page: &P) -> &MarketplaceFactsPage {
    let P::SaveMarketplace { value, page: at, previous, .. } = page else {
        panic!("expected Marketplace editor: {page:?}")
    };
    assert_eq!((*at, *previous), (PAGE_CREDITS, PAGE_CREDITS));
    value
}
pub(in crate::tests) fn save_marketplace(d: &FederalDraft, value: MarketplaceFactsPage) -> P {
    P::SaveMarketplace {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_CREDITS,
        previous: PAGE_CREDITS,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        people: vec![],
        ptc_people: vec![],
        ptc_families: vec![],
        ptc_policies: vec![],
        ptc_w2: vec![],
        ptc_employees: vec![],
        ptc_nonreturn_people: vec![],
        ptc_claimed_dependents: vec![],
        allowed_marriage: vec![],
        allowed_separate: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
pub(in crate::tests) fn at_the_marketplace_page(
    d: FederalDraft,
) -> (Interview, MarketplaceFactsPage) {
    let (mut session, _) = at_the_credits_hub(d);
    let request = open_credits_topic(&mut session, TaxSection::Marketplace);
    let value = marketplace_page_of(&request).clone();
    (session, value)
}
pub(in crate::tests) fn stored_marketplace(s: &Interview) -> &MarketplaceFacts {
    crate::marketplace::facts(&s.draft.credits.as_ref().unwrap().marketplace)
        .expect("saved sources")
}
pub(in crate::tests) fn all_year(
    issuer: &str,
    premium: i64,
    benchmark: i64,
    advance: i64,
) -> MarketplacePolicyRecordPage {
    MarketplacePolicyRecord {
        id: 0,
        issuer: issuer.into(),
        policy_number: "000123".into(),
        amounts: PolicyAmounts::AllYear { premium, benchmark: Some(benchmark), advance },
        lawful_premiums: vec![],
        enrolling: MarketplaceFamilyRef::ThisReturn,
        status: MarketplaceStatementStatus::Original,
        coverage: (1..=12)
            .map(|month| MarketplacePolicyCoverage { month, people: vec![-1], entry: vec![] })
            .collect(),
        allocation: vec![],
        payments: vec![],
        medical_costs: vec![],
    }
    .into()
}
pub(in crate::tests) fn marketplace_fixture_page(
    forms: Vec<MarketplacePolicyRecordPage>,
) -> MarketplaceFactsPage {
    // Explicit neutral completion for arithmetic fixtures: taxpayer-only
    // ordinary coverage and known eligible months, never a production default.
    MarketplaceFactsPage {
        eligibility: (1..=12)
            .filter(|month| {
                forms
                    .iter()
                    .any(|p| crate::marketplace_sources::policy_month(&p.amounts, *month).is_some())
            })
            .map(|month| MarketplaceEligibilityMonthPage {
                month,
                person: -1,
                status: MarketplacePersonEligibility::Eligible,
            })
            .collect(),
        forms,
        ..MarketplaceFactsPage::default()
    }
}
pub(in crate::tests) fn save_marketplace_and_reopen(
    session: &mut Interview,
    value: MarketplaceFactsPage,
) -> MarketplaceFactsPage {
    let request = save_marketplace(&session.draft, value);
    credits_hub(send_past_less_common(session, request));
    let request = open_credits_topic(session, TaxSection::Marketplace);
    marketplace_page_of(&request).clone()
}

// T2-44p: the W-2G and gambling losses kinds on the Gambling winnings topic.
pub(in crate::tests) fn w2g(winnings: i64, withholding: i64) -> IncomeDocument {
    IncomeDocument::W2g(W2g {
        state_details: Default::default(),
        winnings,
        withholding,
        special: false,
    })
}
pub(in crate::tests) fn gambling_losses(other_winnings: i64, losses: i64) -> IncomeDocument {
    IncomeDocument::GamblingLosses(GamblingLosses { other_winnings, losses, special: false })
}

// T2-40p: the Schedule K-1 (Form 1041) kind on the Schedule K-1 topic, and
// the shared K-1 page types.
pub(in crate::tests) fn k1_1041(name: &str) -> K1EstateTrust {
    K1EstateTrust {
        entity_period: None,
        final_character: None,
        amt_reconciliation: None,
        investment_income: None,
        reported_credits: vec![],
        foreign_statement: None,
        investment_property: vec![],
        ein: "012345678".into(),
        name: name.into(),
        estate_activities: vec![],
        estate_mortgages: vec![],
        qbi: crate::EstateQbiStatementsPage { statements: vec![] },
        gross_farming_fishing_income: 0,
        interest: 0,
        ordinary_dividends: 0,
        qualified_dividends: 0,
        short_gain: 0,
        long_gain: 0,
        rate_28_gain: 0,
        unrecaptured_gain: 0,
        royalties: K1EstateTrustRoyalties::None,
        business: K1EstateTrustBusiness::None,
        more: k1_1041_more(),
        special: false,
    }
}
pub(in crate::tests) fn estate_activity_page(
    name: &str,
    kind: crate::EstateActivityKind,
    income: i64,
) -> crate::EstateActivityPage {
    crate::EstateActivityPage {
        id: 0,
        name: name.into(),
        kind,
        income,
        depreciation: 0,
        depletion: 0,
        amortization: 0,
        participation: crate::EstateParticipation::Passive,
        carried: crate::EstatePassiveCarry { regular: 0, alternative: 0 },
        amt_difference: 0,
    }
}
fn k1_1041_more() -> K1EstateTrustOtherBoxes {
    k1_1041_more_with(K1EstateTrustAmt::None, K1EstateTrustEstimatedTax::None, vec![])
}
pub(in crate::tests) fn k1_1041_more_with(
    amt: K1EstateTrustAmt,
    estimated_tax: K1EstateTrustEstimatedTax,
    unentered: Vec<EstateTrustK1Item>,
) -> K1EstateTrustOtherBoxes {
    K1EstateTrustOtherBoxes {
        minimum_tax_exclusion: None,
        ird_estate_tax: 0,
        final_year: K1FinalYear::NotFinal,
        amt,
        estimated_tax,
        backup_withholding: 0,
        tax_exempt_interest: 0,
        private_activity_interest: 0,
        niit: None,
        section199a_dividends: 0,
        unentered,
    }
}
// Saves the K-1s on the Schedule K-1 topic of a fresh single return.
pub(in crate::tests) fn save_k1s(pages: Vec<K1EstateTrust>) -> (Interview, Reply) {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        save_topic(
            &original,
            TaxSection::PassThrough,
            pages.into_iter().map(|p| document(0, IncomeDocument::K1EstateTrust(p))).collect(),
        ),
    );
    (session, reply)
}
pub(in crate::tests) fn reopened_k1s(session: &mut Interview) -> Vec<IncomeDocument> {
    page_documents(&open_topic(session, TaxSection::PassThrough))
        .iter()
        .map(|item| item.document.clone())
        .collect()
}

// ---- T2-45p: the Farming topic (see farm_page.rs). ----
fn farm_expenses() -> FarmExpenses {
    FarmExpenses {
        chemicals: 0,
        custom_hire: 0,
        employee_benefits: 0,
        feed: 0,
        fertilizers: 0,
        freight: 0,
        fuel: 0,
        insurance: 0,
        mortgage_interest: 0,
        other_interest: 0,
        labor: 0,
        pension: 0,
        equipment_rent: 0,
        other_rent: 0,
        repairs: 0,
        seeds: 0,
        storage: 0,
        supplies: 0,
        taxes: 0,
        utilities: 0,
        veterinary: 0,
        meals: 0,
        other: vec![],
    }
}
// A cash grain farm: line 2 sales and one expense line (feed).
pub(in crate::tests) fn cash_farm(raised: i64, feed: i64) -> Farm {
    Farm {
        state_documents: Default::default(),
        name: "".into(),
        accounting: FarmMethod::Cash {
            resale: 0,
            resale_cost: 0,
            raised,
            program: 0,
            custom_hire: 0,
            other: 0,
        },
        insurance: CropInsurance::None,
        expenses: FarmExpenses { feed, ..farm_expenses() },
        vehicles: vec![],
        assets: vec![],
        more: farm_more(),
    }
}
// The More fold with every answer at its default.
pub(in crate::tests) fn farm_more() -> FarmDetails {
    FarmDetails {
        specials: (&farm_specials()).into(),
        land_basis: vec![],
        header: (&farm_header()).into(),
        ccc: CccLoans::None,
        crp: vec![],
        cost_sharing: 0,
        supplies: PrepaidSupplies::None,
        conservation: Conservation::None,
        land_sales: vec![],
        startup: BusinessStartup::None,
        method_changes: Vec::new(),
        payroll: BusinessPayroll::No,
        withholding: 0,
        optional_se: false,
        material_participation: true,
        at_risk: true,
        qbi_group: "".into(),
        income_source: FarmIncomeSource::NotNeeded,
        fuel_tax_credit: None,
        cash_fuel_tax_recovery: None,
        tips: None,
    }
}
pub(in crate::tests) fn with_more(mut farm: Farm, edit: impl FnOnce(&mut FarmDetails)) -> Farm {
    let mut more = farm_more();
    edit(&mut more);
    farm.more = more;
    farm
}
pub(in crate::tests) fn base_year(agi: i64) -> BaseYear {
    BaseYear {
        status: FederalStatus::Single,
        source: BaseYearSource::Return {
            agi,
            deductions: 1_460_000,
            tax: 200_000,
            other_tax: vec![],
            loss: None,
        },
        dividends: 0,
        capital: AveragingCapital::NONE,
        foreign: None,
        investment: None,
    }
}
pub(in crate::tests) fn averaging_election() -> FarmAveragingElection {
    FarmAveragingElection {
        elected: 1_000_000,
        base_third_prior_year: base_year(3_000_000),
        base_second_prior_year: base_year(3_100_000),
        base_prior_year: base_year(3_200_000),
        selection: FarmAveragingSelectionInput {
            capital_elected: 0,
            unrecaptured_elected: 0,
            sources: vec![],
            history_scope: FarmAveragingHistoryScope::CompleteSupported,
        },
    }
}
pub(in crate::tests) fn farm_save(d: &FederalDraft, documents: Vec<IncomeDocumentItem>) -> P {
    save_topic(d, TaxSection::Farming, documents)
}
pub(in crate::tests) fn farm_only_return() -> FederalDraft {
    empty_return(FederalStatus::Single)
}
// A return Form 8615 applies to (T2-cy): not joint, the taxpayer 16 at the
// end of 2026 (under 18) and a parent alive, so the calculation reads every
// income source.
pub(in crate::tests) fn form_8615_return(mut d: FederalDraft) -> FederalDraft {
    let taxpayer = &mut d.people.as_mut().unwrap().taxpayer;
    taxpayer.birth_date = 20100101;
    taxpayer.parent_alive = true;
    d
}
pub(in crate::tests) fn stored_farm(d: &FederalDraft) -> (&FederalBusiness, &FarmIncome) {
    let data = d
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Business { data } => Some(data),
            _ => None,
        })
        .expect("a farm");
    let BusinessActivity::Farming { data: farm } = &data.activity else {
        panic!("not a farm: {data:?}")
    };
    (data, farm)
}
// Reopens the Farming page and returns its documents without previews.
pub(in crate::tests) fn reopened_farming(
    session: &mut Interview,
) -> (Vec<i32>, Vec<IncomeDocument>) {
    let P::SaveIncomes { allowed_documents, documents, .. } =
        open_topic(session, TaxSection::Farming)
    else {
        unreachable!()
    };
    (allowed_documents, documents.documents.into_iter().map(|item| item.document).collect())
}

#[test]
fn young_filer_answers_what_produces_the_farms_income() {
    // F15 as T2-cy: on a return Form 8615 applies to, Not needed is refused
    // on save once the More fold is open (boundaries in
    // income_source_sets_follow_the_calculation).
    let mut original = form_8615_return(farm_only_return());
    let attempt = |original: &FederalDraft, source: FarmIncomeSource| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            farm_save(
                original,
                vec![document(
                    0,
                    IncomeDocument::Farm(with_more(cash_farm(1_000_000, 0), |more| {
                        more.income_source = source;
                    })),
                )],
            ),
        );
        (reply, session)
    };
    let reply = {
        let mut source = Interview::new(TEST_USER, original.clone());
        current_document_refusal(
            "income-source-7609",
            &mut source,
            vec![document(
                0,
                IncomeDocument::Farm(with_more(cash_farm(1_000_000, 0), |more| {
                    more.income_source = FarmIncomeSource::NotNeeded;
                })),
            )],
            "SOURCE_FARM_NEEDED",
            "[documents/documents/0/document/more/income_source]",
        )
    };
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let (reply, mut session) = attempt(&original, FarmIncomeSource::Capital);
    accepted(&reply);
    let (data, _) = stored_farm(&session.draft);
    assert_eq!(data.income_source, BusinessIncomeSource::Capital);
    let (_, reopened) = reopened_farming(&mut session);
    let IncomeDocument::Farm(farm) = &reopened[0] else { unreachable!() };
    assert_eq!(farm.more.income_source, FarmIncomeSource::Capital);
    // No parent alive: Form 8615 does not apply, Not needed passes.
    original.people.as_mut().unwrap().taxpayer.parent_alive = false;
    accepted(&attempt(&original, FarmIncomeSource::NotNeeded).0);
}

// T2-31p: the Schedule K-1 (Form 1120-S) kind on the Schedule K-1 topic.
pub(in crate::tests) fn k1_1120s(name: &str) -> K1Scorp {
    K1Scorp {
        reporting_period: None,
        qbi_group: alloc::string::String::new(),
        disposition: ScorpDisposition::Kept,
        final_k1: false,
        schedule_k3: crate::K3Attached::NotAttached,
        rentals: K1RentalBoxes {
            real_estate_section1231: 0,
            real_estate_unrecaptured: 0,
            other_section1231: 0,
            other_unrecaptured: 0,
            disposed: Vec::new(),
            real_estate: 0,
            other: 0,
            participation: K1RentalParticipation::NotActive,
            real_estate_qbi: None,
            other_qbi: None,
            carryover_active: true,
        },
        name: name.into(),
        ordinary: 0,
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        distributions: 0,
        qbi_records: K1ScorpQbiRecords {
            ordinary: Default::default(),
            property: Default::default(),
            property_complete: true,
        },
        qbi: K1ScorpQbi::Statement {
            ptp_income: 0,
            reit_dividends: 0,
            income: 0,
            wages: 0,
            property: 0,
            service: false,
            current_loss: 0,
            property_qbi: no_property_qbi(),
        },
        participation: Participation::Investor { prior_years: 0 },
        more: k1_1120s_more(),
        basis: K1ScorpBasis {
            start: K1ScorpStart::New,
            additions: 0,
            loans: vec![],
            more_at_risk: k1_1120s_all_at_risk(),
        },
        amt: ScorpAmtRecords::SameAsRegular,
        special: Vec::new(),
    }
}
fn k1_1120s_more() -> K1ScorpOtherBoxes {
    K1ScorpOtherBoxes {
        amt_section179_income_carry: None,
        allowed_investment_confirmations: Vec::new(),
        section179_carryover_sold: 0,
        section179_sales: alloc::vec::Vec::new(),
        deductions: Vec::new(),
        income: Vec::new(),
        foreign_taxes: 0,
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        unrecaptured_business: 0,
        section1231: 0,
        section179: 0,
        section179_carryover: 0,
        cash_charity: 0,
        withholding: 0,
        private_activity: 0,
        exempt_interest: 0,
        exempt: 0,
        nondeductible: 0,
        stock_term: K1StockTerm::NotNeeded,
        trading: false,
        credits: Vec::new(),
        unentered: vec![],
        investment_property: Vec::new(),
        investment_confirmed: None,
        housing_confirmed: None,
    }
}
// A continuing shareholder with `opening` stock basis and a retained 2025
// Form 6198 line 19b of `capacity`, and no losses carried from 2025.
pub(in crate::tests) fn k1_1120s_continuing(opening: i64, capacity: i64) -> K1ScorpStart {
    K1ScorpStart::Continuing {
        opening,
        risk_history: ScorpRiskStart::Detailed { capacity, profit: 0, loss: 0 },
        carried: k1_1120s_carried_zero(),
    }
}
// The "Losses carried from 2025" branch with every amount zero.
pub(in crate::tests) fn k1_1120s_carried_zero() -> K1ScorpCarried {
    K1ScorpCarried {
        rental_real_estate_passive_loss: 0,
        other_rental_passive_loss: 0,
        rental_real_estate_section1231_passive_loss: 0,
        other_rental_section1231_passive_loss: 0,
        rental_real_estate_basis_loss: 0,
        rental_real_estate_risk_loss: 0,
        other_rental_basis_loss: 0,
        other_rental_risk_loss: 0,
        basis_section179: 0,
        basis_charity: 0,
        basis_foreign_taxes: 0,
        basis_loss: 0,
        basis_nondeductible: 0,
        losses_first: false,
        basis_short: 0,
        basis_long: 0,
        basis_collectibles: 0,
        risk_short: 0,
        risk_long: 0,
        risk_collectibles: 0,
        section1231_basis_loss: 0,
        section1231_risk_loss: 0,
        section1231_passive_loss: 0,
        risk_loss: 0,
        risk_nondeductible: 0,
        deducted: 0,
        recaptured: 0,
        passive_loss: 0,
    }
}
pub(in crate::tests) fn save_k1_1120s(pages: Vec<K1Scorp>) -> (Interview, Reply) {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        save_topic(
            &original,
            TaxSection::PassThrough,
            pages.into_iter().map(|p| document(0, IncomeDocument::K1Scorp(p))).collect(),
        ),
    );
    (session, reply)
}
pub(in crate::tests) fn k1_1120s_more_mut(page: &mut K1Scorp) -> &mut K1ScorpOtherBoxes {
    &mut page.more
}
// The "Amounts not at risk" record with every amount zero.
fn k1_1120s_all_at_risk() -> K1ScorpAtRisk {
    K1ScorpAtRisk { excluded_funding: 0, protected: 0, released: 0, excluded_returns: 0 }
}
// The statement's section 1231 QBI amounts when the K-1 has no section 1231 item.
pub(in crate::tests) fn no_property_qbi() -> K1ScorpPropertyQbi {
    K1ScorpPropertyQbi { gain: 0, qualified_1250: None, current: 0 }
}

// The T2-31 and T2-29b shareholder hand fixtures (src/tests/passthrough/passthrough.rs
// shareholder_k1_items_and_distribution_above_basis_reach_the_return_by_hand
// and shareholder_section179_reaches_form_4562_schedule_e_basis_and_qbi_by_hand)
// entered through the page: single, W-2 wages $60,000 ($6,500 withheld);
// a materially participating shareholder (more than 500 hours), opening
// stock basis $10,000, one $3,000 written note at full basis, retained 2025
// Form 6198 line 19b $13,000, no carryovers, a statement QBI equal to box 1.
pub(in crate::tests) fn k1_1120s_fixture(profit: i64) -> K1Scorp {
    let mut page = k1_1120s("Synthetic S Corporation");
    page.ordinary = profit * 100;
    page.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: profit * 100,
        wages: 0,
        property: 0,
        service: false,
        current_loss: 0,
        property_qbi: no_property_qbi(),
    };
    page.participation = Participation::Over500Hours { hours: 501 };
    page.basis.start = k1_1120s_continuing(1_000_000, 1_300_000);
    page.basis.loans = vec![ScorpLoan {
        kind: ScorpLoanKind::FormalNote,
        repayment_term: ScorpNoteHolding::NoRepayment,
        principal: 300_000,
        basis: 300_000,
        advances: 0,
        repayments: 0,
    }];
    page
}

// T2-29p: the Schedule K-1 (Form 1065) kind on the Schedule K-1 topic.
pub(in crate::tests) fn k1_1065(name: &str) -> K1Partnership {
    K1Partnership {
        reporting_period: None,
        qbi_group: alloc::string::String::new(),
        disposition: PartnerDisposition::Kept,
        final_k1: false,
        schedule_k3: crate::K3Attached::NotAttached,
        rentals: K1RentalBoxes {
            real_estate_section1231: 0,
            real_estate_unrecaptured: 0,
            other_section1231: 0,
            other_unrecaptured: 0,
            disposed: Vec::new(),
            real_estate: 0,
            other: 0,
            participation: K1RentalParticipation::NotActive,
            real_estate_qbi: None,
            other_qbi: None,
            carryover_active: true,
        },
        name: name.into(),
        publicly_traded: false,
        interest: PartnerInterest::General {
            participation: Participation::Investor { prior_years: 0 },
        },
        liabilities_start: 0,
        liabilities_end: 0,
        ordinary: 0,
        services: 0,
        capital: 0,
        se: 0,
        se_gross: SeGross::None,
        distributions: 0,
        qbi_records: PartnerQbiRecords { qualified_se_gross: 0, history: None },
        qbi: PartnerQbi::Statement {
            section1231_qbi: 0,
            ptp_income: 0,
            reit_dividends: 0,
            income: 0,
            wages: 0,
            property: 0,
            service: false,
        },
        more: k1_1065_more(),
        basis: PartnerBasis {
            start: BasisStart::New,
            contributions: 0,
            debt_start: 0,
            debt_end: 0,
            more_at_risk: partner_all_at_risk(),
        },
        income_source: K1IncomeSource::NotNeeded,
        amt: PartnerAmtRecords::SameAsRegular,
        special: Vec::new(),
    }
}
pub(in crate::tests) fn k1_1065_more() -> K1PartnershipOtherBoxes {
    K1PartnershipOtherBoxes {
        amt_section179_income_carry: None,
        allowed_investment_confirmations: Vec::new(),
        section179_carryover_sold: 0,
        section179_sales: alloc::vec::Vec::new(),
        section1231: 0,
        unrecaptured_business: 0,
        income: Vec::new(),
        sale_ordinary: 0,
        sale_collectibles: 0,
        sale_unrecaptured: 0,
        term: K1StockTerm::NotNeeded,
        crp_payments: 0,
        deductions: Vec::new(),
        deferred_compensation_tax: 0,
        foreign_taxes: 0,
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        section179: 0,
        section179_carryover: 0,
        cash_charity: 0,
        medical_insurance: 0,
        pensions: 0,
        backup_withholding: 0,
        exempt_interest: 0,
        exempt_other: 0,
        nondeductible: 0,
        fuel_credit: None,
        prior_fuel_credit: None,
        trading: false,
        credits: Vec::new(),
        unentered: vec![],
        investment_property: Vec::new(),
        investment_confirmed: None,
        housing_confirmed: None,
    }
}
pub(in crate::tests) fn k1_1065_continuing(
    opening: i64,
    risk_history: ScorpRiskStart,
) -> BasisStart {
    BasisStart::Continuing { opening, risk_history, carried: k1_1065_carried_zero() }
}
// The partnership "Amounts not at risk" record with every amount zero.
fn partner_all_at_risk() -> PartnerAtRisk {
    PartnerAtRisk {
        excluded_contributions: 0,
        excluded_distributions: 0,
        protected: 0,
        released: 0,
    }
}
// The "Losses carried from 2025" record with every amount zero.
pub(in crate::tests) fn k1_1065_carried_zero() -> PartnerCarried {
    PartnerCarried {
        section1231_basis_loss: 0,
        section1231_risk_loss: 0,
        section1231_passive_loss: 0,
        rental_real_estate_passive_loss: 0,
        other_rental_passive_loss: 0,
        rental_real_estate_section1231_passive_loss: 0,
        other_rental_section1231_passive_loss: 0,
        basis_short: 0,
        basis_long: 0,
        risk_short: 0,
        risk_long: 0,
        rental_real_estate_basis_loss: 0,
        rental_real_estate_risk_loss: 0,
        other_rental_basis_loss: 0,
        other_rental_risk_loss: 0,
        basis_section179: 0,
        basis_charity: 0,
        basis_foreign_taxes: 0,
        basis_loss: 0,
        basis_nondeductible: 0,
        risk_loss: 0,
        risk_nondeductible: 0,
        deducted: 0,
        recaptured: 0,
        passive_loss: 0,
        se_basis: 0,
        se_risk: 0,
        se_passive: 0,
    }
}
pub(in crate::tests) fn save_1065s(
    draft: &FederalDraft,
    pages: Vec<K1Partnership>,
) -> (Interview, Reply) {
    let mut session = Interview::new(TEST_USER, draft.clone());
    let reply = send(
        &mut session,
        save_topic(
            draft,
            TaxSection::PassThrough,
            pages.into_iter().map(|p| document(0, IncomeDocument::K1Partnership(p))).collect(),
        ),
    );
    (session, reply)
}

#[test]
fn protocol_has_no_screening_page() {
    // The retired screening variants are gone, and every NextResponse list
    // names only declared variants.
    let source = include_str!("../../core/protocol.rs");
    assert!(!source.contains("BeginScreening"));
    assert!(!source.contains("SaveScreening"));
    let declared = |name: &str| {
        source.contains(&alloc::format!("\n    {name} {{"))
            || source.contains(&alloc::format!("\n    {name},"))
    };
    let mut lists = 0;
    for chunk in source.split("#[libertas_next_response(").skip(1) {
        let list = chunk.split('"').nth(1).unwrap();
        for name in list.split(',') {
            assert!(declared(name), "{name} is not a protocol variant");
        }
        lists += 1;
    }
    assert!(lists > 30);
}

// Part C cleanups (unit T2-cx, 2026-09-27).

// C8: every duplicate-entry rule the server checks on save is declared as a
// client rule with the same key, so the client refuses it before sending.

pub(in crate::tests) fn ira_row(
    owner: Owner,
    traditional: i64,
    excess: Option<IraExcessEntry>,
) -> IraContributor {
    IraContributor {
        owner,
        traditional,
        roth: 0,
        covered: false,
        form_8606_value: None,
        more: match excess {
            None => IraMore::default(),
            Some(excess) => IraMore {
                returned_in_following_year: vec![],
                nondeductible: 0,
                paid_next_year: None,
                spouse_covered_if_separate: false,
                excess: Some(excess),
                qtp_roth_prior: 0,
            },
        },
    }
}
pub(in crate::tests) fn open_ira_page(s: &mut Interview) -> (Vec<IraContributor>, Vec<i32>) {
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::IraContributions);
    let Reply::Edit(P::SaveIraContributions { value, allowed_owners, .. }) = send(s, request)
    else {
        panic!("expected the IRA contributions page")
    };
    (value.people, allowed_owners)
}

// C18: the traditional value has one home and the confirmation is a branch.
pub(in crate::tests) fn excess_entry(traditional_value: Option<i64>) -> IraExcessEntry {
    IraExcessEntry {
        traditional_value,
        roth_value: 0,
        prior_traditional: 0,
        prior_roth: 0,
        returned_prior_excess: 0,
    }
}
pub(in crate::tests) fn with_form_8606(session: &mut Interview, year_end: i64) {
    let draft = session.draft.clone();
    accepted(&send(
        session,
        save_topic(
            &draft,
            TaxSection::Retirement,
            vec![document(
                0,
                IncomeDocument::Form8606(Form8606 {
                    adjustment: 0,
                    reservist_repayments: vec![],
                    basis: 100_000,
                    year_end,
                }),
            )],
        ),
    ));
}

// C19, C21: defaults and inline choices declared on the page types.
#[test]
fn form_8862_groups_default_to_never_and_retirement_plans_are_inline() {
    let form_8862 = include_str!("../../forms/form_8862.rs");
    for field in ["earned_income", "child_credits", "education_credit"] {
        assert!(
            form_8862.contains(&alloc::format!("#[libertas_default(Never)]\n    pub {field}:")),
            "{field}"
        );
    }
    let model = include_str!("../../federal/federal_model.rs");
    assert!(model.contains("#[libertas_inline_choices]\npub enum SeRetirement {"));
}

pub(in crate::tests) fn k1_1065_benefits(
    name: &str,
    medical: i64,
    pensions: i64,
    distributions: i64,
) -> K1Partnership {
    let mut page = k1_1065(name);
    page.services = medical;
    page.distributions = distributions;
    page.basis.start = k1_1065_continuing(
        10_000_000,
        ScorpRiskStart::Detailed { capacity: 10_000_000, profit: 0, loss: 0 },
    );
    if medical != 0 || pensions != 0 {
        page.more = k1_1065_more();
        {
            let K1PartnershipOtherBoxes { medical_insurance, pensions: box_13r, .. } =
                &mut page.more;
            *medical_insurance = medical;
            *box_13r = pensions;
        }
    }
    page
}

pub(in crate::tests) fn partner_benefits_session(pages: Vec<K1Partnership>) -> Interview {
    let mut d = complete(FederalStatus::Single);
    d.finished_sections = vec![TaxSection::Children, TaxSection::Checklist, TaxSection::Income];
    let mut s = Interview::new(TEST_USER, d.clone());
    let request = save_topic(
        &d,
        TaxSection::PassThrough,
        pages.into_iter().map(|p| document(0, IncomeDocument::K1Partnership(p))).collect(),
    );
    accepted(&send(&mut s, request));
    s
}

pub(in crate::tests) fn open_benefits(s: &mut Interview) -> SelfEmployedBenefits {
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::SelfEmployedBenefits);
    match send(s, request) {
        Reply::Edit(P::SaveSelfEmployedBenefits { value, .. }) => value,
        other => panic!("expected the self-employed benefits page, got {other:?}"),
    }
}

pub(in crate::tests) fn benefits_request(draft: &FederalDraft, value: SelfEmployedBenefits) -> P {
    P::SaveSelfEmployedBenefits {
        user: TEST_USER,
        ptc_policies: vec![],
        health_activities: vec![],
        health_people: vec![],
        health_wages: vec![],
        cookie: draft.cookie.clone(),
        revision: draft.revision,
        page: PAGE_ADJUSTMENTS,
        previous: PAGE_ADJUSTMENTS,
        review: false,
        estimate: Some(vec![]),
        warnings: vec![],
        progress: Some(vec![]),
        value,
    }
}
pub(in crate::tests) fn save_benefits(s: &mut Interview, value: SelfEmployedBenefits) -> Reply {
    let request = benefits_request(&s.draft, value);
    send(s, request)
}

// T2-cy: the income-source allowed sets of the income page.
fn source_sets(d: &FederalDraft) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut session = Interview::new(TEST_USER, d.clone());
    let P::SaveIncomes {
        allowed_business_source, allowed_farm_source, allowed_partner_source, ..
    } = open_topic(&mut session, TaxSection::SelfEmployment)
    else {
        unreachable!()
    };
    (allowed_business_source, allowed_farm_source, allowed_partner_source)
}
pub(in crate::tests) fn problem_text(reply: Reply) -> alloc::string::String {
    let Reply::Response(P::Problem { message, .. }) = reply else {
        panic!("expected a refusal, got {reply:?}")
    };
    status_text(&message)
}

#[test]
fn income_source_sets_follow_the_calculation() {
    // Business (BusinessIncomeChoice) and farm (FarmIncomeSource) sources
    // enter only Form 8615 (not joint, the age test, a parent alive); a
    // partnership's also enters the Standard Deduction Worksheet for
    // Dependents when someone can claim a filer (Form 1040 line 12e).
    let not_needed = (vec![0], vec![0], vec![0]);
    let needed = (vec![1, 2, 3], vec![1, 2], vec![1, 2, 3]);
    let adult = complete(FederalStatus::Single);
    assert_eq!(source_sets(&adult), not_needed);
    let young = form_8615_return(complete(FederalStatus::Single));
    assert_eq!(source_sets(&young), needed);
    // Boundary: no parent alive.
    let mut d = young.clone();
    d.people.as_mut().unwrap().taxpayer.parent_alive = false;
    assert_eq!(source_sets(&d), not_needed);
    // Boundary: a joint return (the taxpayer 16) does not file Form 8615.
    let d = form_8615_return(complete(FederalStatus::Joint));
    assert_eq!(source_sets(&d), not_needed);
    // Boundary: 18 at the end of 2026 (born January 2, 2008; one born
    // January 1 is 19, a birthday counting the day before) takes Form 8615
    // only without earned income over half of support (exactly half is not
    // over).
    let mut d = young.clone();
    d.people.as_mut().unwrap().taxpayer.birth_date = 20080101;
    d.people.as_mut().unwrap().taxpayer.support_share = SupportShare::ExactlyHalf;
    assert_eq!(source_sets(&d), not_needed);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20080102;
    d.people.as_mut().unwrap().taxpayer.support_share = SupportShare::MoreThanHalf;
    assert_eq!(source_sets(&d), not_needed);
    d.people.as_mut().unwrap().taxpayer.support_share = SupportShare::ExactlyHalf;
    assert_eq!(source_sets(&d), needed);
    // A filer someone can claim: only the partnership's source is read.
    let mut d = adult.clone();
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(source_sets(&d), (vec![0], vec![0], vec![1, 2, 3]));
    // ... or the spouse on a joint return (T2-cy item 5).
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(source_sets(&d), (vec![0], vec![0], vec![1, 2, 3]));
}

// The attribute lines directly above a field declaration (up to its doc
// comment), from the schema-owning source.
fn field_attributes<'a>(source: &'a str, declaration: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = source.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.trim() == declaration)
        .unwrap_or_else(|| panic!("{declaration} not found"));
    lines[..at].iter().rev().map(|l| l.trim()).take_while(|l| l.starts_with("#[")).collect()
}
// A ConstrainedBy field whose allowed set can be a singleton declares no
// Default: the client selects the single choice itself, and a Default
// outside the set would block Submit (protocols.md, Backend-constrained
// choices, singleton allowed set).
#[test]
fn constrained_fields_with_a_singleton_set_declare_no_default() {
    for (source, declaration, set) in [
        (include_str!("../../federal/federal_model.rs"), "farmer: Farmer,", "allowed_farmer"),
        (
            include_str!("../../federal/federal_model.rs"),
            "pub income_source: crate::BusinessIncomeChoice,",
            "allowed_business_source",
        ),
        (
            include_str!("../../farm/farm_page.rs"),
            "pub income_source: FarmIncomeSource,",
            "allowed_farm_source",
        ),
        (
            include_str!("../../passthrough/k1_page.rs"),
            "pub income_source: K1IncomeSource,",
            "allowed_partner_source",
        ),
    ] {
        let attributes = field_attributes(source, declaration);
        assert!(attributes.iter().any(|a| a.contains(set)), "{declaration}: {attributes:?}");
        assert!(
            !attributes.iter().any(|a| a.starts_with("#[libertas_default")),
            "{declaration}: {attributes:?}"
        );
    }
}

// T3-8915: the 1099-R fold "This was a qualified disaster distribution" and
// the Earlier Form 8915-F kind reach the stored return and reopen unchanged.
fn disaster_fold(amount: i64, earlier: i64, spread: DisasterSpread) -> DisasterDistribution {
    DisasterDistribution::Qualified {
        fema: 4900,
        began: 20260310,
        declared: 20260320,
        distributed: 20260501,
        amount,
        earlier,
        spread,
        repaid: crate::DisasterRepayment::No,
    }
}
fn disaster_r1099(amount: i64, earlier: i64, spread: DisasterSpread) -> R1099 {
    R1099 {
        disaster: disaster_fold(amount, earlier, spread),
        ..r1099(
            3_000_000,
            DistributionCode::Early {
                what_happened: Disposition::Kept,
                exception: EarlyException::None,
            },
            false,
        )
    }
}
fn earlier_8915(year: Form8915Year, disaster_year: u32) -> EarlierForm8915 {
    EarlierForm8915 {
        year,
        disaster_year,
        line_11: 400_000,
        line_22: 150_000,
        repayments: Form8915Repayments::default(),
    }
}

#[test]
fn qualified_disaster_fold_and_earlier_form_8915_round_trip() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let fold = disaster_r1099(2_000_000, 0, DisasterSpread::OverThreeYears);
    let earlier = earlier_8915(
        Form8915Year::Filed2025 { line_13: 0, line_14: 0, line_24: 0, line_25: 0 },
        2025,
    );
    send(
        &mut session,
        save_topic(
            &original,
            TaxSection::Retirement,
            vec![
                document(0, IncomeDocument::R1099(fold.clone())),
                document(0, IncomeDocument::Form8915(earlier.clone())),
            ],
        ),
    );
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Retirement { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(stored.disaster, fold.disaster);
    assert_eq!(
        session.draft.screening.as_ref().unwrap().disaster_carries,
        vec![DisasterCarry { owner: Owner::Taxpayer, form: earlier.clone() }]
    );
    let mut restarted = restart(session.draft.clone());
    let page = open_topic(&mut restarted, TaxSection::Retirement);
    let documents: Vec<_> =
        page_documents(&page).iter().map(|item| item.document.clone()).collect();
    assert!(documents.contains(&IncomeDocument::R1099(fold)));
    assert!(documents.contains(&IncomeDocument::Form8915(earlier)));
}

#[test]
fn qualified_disaster_rules_reject_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let reject = |documents: Vec<IncomeDocument>, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_topic(
                &original,
                TaxSection::Retirement,
                documents.into_iter().map(|d| document(0, d)).collect(),
            ),
        );
        rejected_with(reply, key);
        assert_eq!(session.draft, original);
    };
    // Client rules of one 1099-R, checked again on save.
    let mut over_gross = disaster_r1099(2_000_000, 0, DisasterSpread::OverThreeYears);
    over_gross.gross = 1_000_000;
    over_gross.taxable = 1_000_000;
    reject(vec![IncomeDocument::R1099(over_gross)], "R_DISASTER_ABOVE_GROSS");
    reject(
        vec![IncomeDocument::R1099(disaster_r1099(
            2_000_000,
            200_001,
            DisasterSpread::OverThreeYears,
        ))],
        "DISASTER_ABOVE_LIMIT",
    );
    let mut over_repaid = disaster_r1099(1_000_000, 0, DisasterSpread::OverThreeYears);
    if let DisasterDistribution::Qualified { repaid, .. } = &mut over_repaid.disaster {
        *repaid = crate::DisasterRepayment::Yes { total: 1_000_001, in_return_year: 0 };
    }
    reject(vec![IncomeDocument::R1099(over_repaid)], "DISASTER_REPAID_ABOVE_AMOUNT");
    // Across the page's 1099-Rs (server only): one limit per disaster, one
    // line 1b amount and one election per disaster year.
    reject(
        vec![
            IncomeDocument::R1099(disaster_r1099(2_000_000, 0, DisasterSpread::OverThreeYears)),
            IncomeDocument::R1099(disaster_r1099(200_001, 0, DisasterSpread::OverThreeYears)),
        ],
        "DISASTER_ABOVE_LIMIT",
    );
    reject(
        vec![
            IncomeDocument::R1099(disaster_r1099(100_000, 0, DisasterSpread::OverThreeYears)),
            IncomeDocument::R1099(disaster_r1099(100_000, 0, DisasterSpread::AllInReturnYear)),
        ],
        "DISASTER_ELECTION_DIFFERS",
    );
    reject(
        vec![
            IncomeDocument::R1099(disaster_r1099(100_000, 0, DisasterSpread::OverThreeYears)),
            IncomeDocument::R1099(disaster_r1099(100_000, 5_000, DisasterSpread::OverThreeYears)),
        ],
        "DISASTER_EARLIER_DIFFERS",
    );
    // Earlier Form 8915-F rules.
    reject(
        vec![IncomeDocument::Form8915(earlier_8915(Form8915Year::Filed2024, 2025))],
        "FORM_8915_DISASTER_AFTER_FORM",
    );
    let mut carried = earlier_8915(
        Form8915Year::Filed2025 { line_13: 200_000, line_14: 500_000, line_24: 0, line_25: 0 },
        2025,
    );
    carried.repayments = Form8915Repayments { plan_carried_back: 300_001, ..Default::default() };
    reject(vec![IncomeDocument::Form8915(carried)], "FORM_8915_CARRIED_BACK_ABOVE_EXCESS");
    let twice = earlier_8915(Form8915Year::Filed2024, 2024);
    reject(
        vec![IncomeDocument::Form8915(twice.clone()), IncomeDocument::Form8915(twice)],
        "FORM_8915_TWICE",
    );
}

#[test]
fn stock_options_tick_is_answered_on_investment_sales() {
    // The tick with nothing entered is no remaining item (plans are not
    // completeness); entering a Form 3921 on Investment sales answers it.
    let mut d = complete(FederalStatus::Single);
    d.checklist = Checklist { other_taxes: OtherTaxTopics::STOCK_OPTIONS.0, ..Checklist::NONE };
    let mut s = Interview::new(TEST_USER, d.clone());
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::Form3921(form_3921(Form3921More::default())))],
    );
    accepted(&send(&mut s, request));
    // The entry answers the tick: planned drops it, the checklist shows it
    // from the entry, and the review has no item for it.
    assert_eq!(crate::interview::planned(&s.draft).other_taxes, 0);
    let (items, _, _, _) = review_state(&mut s);
    assert!(items.iter().all(|i| !status_text(&i.status).contains("incentive stock")), "{items:?}");
    let draft = s.draft.clone();
    let reply = send(&mut s, navigate(&draft, TaxSection::Checklist));
    assert_eq!(checklist_of(&reply).other_taxes, OtherTaxTopics::STOCK_OPTIONS.0);
}

pub(in crate::tests) fn earlier_installment(
    kind: crate::PropertyCharacter,
    earlier: i64,
    payments: i64,
) -> crate::InstallmentPayments {
    crate::InstallmentPayments {
        property: "Lakeside lot".into(),
        sold: 20230601,
        gross_profit: 6_000_000,
        contract_price: 15_000_000,
        earlier_payments: earlier,
        payments,
        kind,
        special: false,
        qbi: crate::InstallmentQbi::NotQualified,
    }
}

// T3-3800: employer credits on the business page, K-1 credit codes and the
// Form 3800 carryovers item of More credits.
pub(in crate::tests) fn employer_credit_business() -> Business {
    let mut b = business(2_415_000);
    b.expenses.wages = 3_000_000;
    b.expenses.legal_and_professional = 300_000;
    let w2 = |label: &str, wages| PayrollW2 { label: label.into(), wages, medicare: wages };
    let employee =
        |label: &str, group, began, hours, first_2026, first_2025| WorkOpportunityEmployee {
            label: label.into(),
            group,
            began,
            first_year_hours: hours,
            first_year_paid_in_year: first_2026,
            first_year_paid_in_prior_year: first_2025,
        };
    b.more.payroll = BusinessPayroll::Yes {
        statements: vec![
            w2("Employee A", 900_000),
            w2("Employee B", 1_000_000),
            w2("Employee C", 1_100_000),
        ],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    b.more.work_opportunity = Some(WorkOpportunityCredit {
        employees: vec![
            employee("Employee A", WorkOpportunityGroup::Standard, 20250701, 600, 500_000, 200_000),
            employee(
                "Employee B",
                WorkOpportunityGroup::LongTermFamilyAssistance {
                    second_year_paid_in_year: 800_000,
                    second_year_paid_in_prior_year: 500_000,
                },
                20240301,
                1_000,
                0,
                0,
            ),
            employee(
                "Employee C",
                WorkOpportunityGroup::DisabledVeteran,
                20251001,
                300,
                900_000,
                300_000,
            ),
        ],
        confirmed: true,
    });
    b.more.retirement_plan = Some(RetirementPlanCredit {
        startup: Some(PlanStartupCosts {
            first_credit_year: 2025,
            employees: 12,
            eligible_employees: 10,
            costs: 300_000,
        }),
        auto_enrollment_since: Some(2025),
        confirmed: true,
    });
    b
}

// Form 8941 on the business page: fixture A of the map (four employees, one
// seasonal, one partial year) entered as the person would.
//   Premiums paid $20,160 (line 4), capped at $23,800 (line 5); credit
//   10,080 (line 12); Schedule C line 14 $20,160 - $10,080 = $10,080, wages
//   $72,000: net profit $150,000 - $82,080 = $67,920.
pub(in crate::tests) fn health_credit_business() -> Business {
    let mut b = business(15_000_000);
    b.expenses.wages = 7_200_000;
    b.expenses.employee_benefits = 2_016_000;
    let w2 = |label: &str, wages| PayrollW2 { label: label.into(), wages, medicare: wages };
    b.more.payroll = BusinessPayroll::Yes {
        statements: vec![
            w2("E1", 2_600_000),
            w2("E2", 2_400_000),
            w2("E3", 1_400_000),
            w2("E4", 800_000),
        ],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    let enrolled = |tier, months, total: i64, paid: i64| {
        Some(EmployeeHealthCoverage {
            tier,
            months,
            total_premium: total * 100,
            employer_paid: paid * 100,
        })
    };
    let employee = |label: &str, service, coverage| HealthCreditEmployee {
        label: label.into(),
        service,
        coverage,
    };
    b.more.small_employer_health = Some(SmallEmployerHealthCredit {
        coverage: HealthCreditCoverage::Marketplace,
        first_credit_year: 2026,
        employee_only_average: 800_000,
        family_average: 2_200_000,
        employees: vec![
            employee(
                "E1",
                HealthCreditService::Hours { hours: 2_080 },
                enrolled(HealthCoverageTier::EmployeeOnly, 12, 7_200, 5_040),
            ),
            employee(
                "E2",
                HealthCreditService::Hours { hours: 2_080 },
                enrolled(HealthCoverageTier::Family, 12, 18_000, 12_600),
            ),
            employee(
                "E3",
                HealthCreditService::Hours { hours: 1_040 },
                enrolled(HealthCoverageTier::EmployeeOnly, 6, 3_600, 2_520),
            ),
            employee("E4", HealthCreditService::Seasonal, None),
        ],
        state_subsidies: 0,
        confirmed: true,
    });
    b
}

// Form 8994 on the business page: fixture A of the map (two employees on
// leave) entered as the person would, with the insurance premiums of fixture C.
//   Credit wages A $4,000 at 100% = $1,000.00, B $1,500 at 75% = $281.25:
//   line 1a $1,281; premiums $6,000 at 60% = line 1b $900. Schedule C line 26
//   $104,000 - $1,281 = $102,719, line 14 $8,000 - $900 = $7,100: net profit
//   $300,000 - $102,719 - $7,100 = $190,181.
pub(in crate::tests) fn paid_leave_business() -> Business {
    let mut b = business(30_000_000);
    b.expenses.wages = 10_400_000;
    b.expenses.employee_benefits = 800_000;
    let w2 = |label: &str, wages| PayrollW2 { label: label.into(), wages, medicare: wages };
    b.more.payroll = BusinessPayroll::Yes {
        statements: vec![w2("A", 5_200_000), w2("B", 5_200_000)],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    let employee = |label: &str, hours, wages, rate| PaidLeaveEmployee {
        label: label.into(),
        weekly_hours: 40,
        prior_year_compensation: 5_200_000,
        normal_hourly_rate: 2_500,
        leave_hours_hundredths: hours,
        leave_wages: wages,
        rate_hundredths: rate,
    };
    b.more.paid_leave = Some(PaidLeaveCredit {
        employees: vec![
            employee("A", 16_000, 400_000, 10_000),
            employee("B", 8_000, 150_000, 7_500),
        ],
        insurance: Some(PaidLeaveInsurance { premiums: 600_000, rate_hundredths: 6_000 }),
        confirmed: true,
    });
    b
}

#[test]
fn paid_leave_page_rules_are_checked_on_save() {
    // Exercise the actual native Save owner; declarations are server-only.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let edited = |edit: &dyn Fn(&mut PaidLeaveCredit)| {
        let mut b = paid_leave_business();
        edit(b.more.paid_leave.as_mut().unwrap());
        b
    };
    // An employee entered twice.
    rejected_with(
        save_businesses(&mut session, vec![edited(&|c| c.employees[1].label = "A".into())]),
        "PAID_LEAVE_EMPLOYEE_TWICE",
    );
    // Nothing claimed.
    rejected_with(
        save_businesses(
            &mut session,
            vec![edited(&|c| {
                c.employees.clear();
                c.insurance = None;
            })],
        ),
        "PAID_LEAVE_NOTHING_CLAIMED",
    );
    // More than 12 weeks of an employee's 40 hours a week: 48,001 hundredths.
    rejected_with(
        save_businesses(
            &mut session,
            vec![edited(&|c| c.employees[0].leave_hours_hundredths = 48_001)],
        ),
        "PAID_LEAVE_ABOVE_12_WEEKS",
    );
    accepted(&save_businesses(
        &mut session,
        vec![edited(&|c| c.employees[0].leave_hours_hundredths = 48_000)],
    ));
    // Premiums only are accepted.
    accepted(&save_businesses(&mut session, vec![edited(&|c| c.employees.clear())]));
    // The credit needs the payroll.
    let mut b = paid_leave_business();
    b.more.payroll = BusinessPayroll::No;
    rejected_with(save_businesses(&mut session, vec![b]), "EMPLOYER_CREDITS_NEED_PAYROLL");
}

// T3-982: the Canceled debt topic, its Form 1099-C and Form 1099-A kinds.
pub(in crate::tests) fn car_1099c(amount: i64, treatment: CanceledDebtChoice) -> IncomeDocument {
    IncomeDocument::Cancel1099C(Cancel1099C {
        creditor: "Synthetic Auto Credit".into(),
        date: 20260315,
        amount,
        liable: true,
        property: PropertyTaken::None,
        treatment,
    })
}
fn canceled_debt_documents() -> Vec<IncomeDocumentItem> {
    vec![
        document(
            0,
            IncomeDocument::Cancel1099C(Cancel1099C {
                creditor: "Synthetic Auto Credit".into(),
                date: 20260315,
                amount: 150_000,
                liable: true,
                property: PropertyTaken::Taken {
                    value: 700_000,
                    property: TransferredProperty::PersonalUse {
                        basis: 1_200_000,
                        term: GainTerm::Long,
                        collectible: false,
                    },
                },
                treatment: CanceledDebtChoice::Taxable,
            }),
        ),
        document(
            0,
            IncomeDocument::Cancel1099C(Cancel1099C {
                creditor: "Synthetic Farm Credit".into(),
                date: 20260601,
                amount: 100_000,
                liable: true,
                property: PropertyTaken::None,
                treatment: CanceledDebtChoice::FarmDebt {
                    solvency: DebtSolvency::Insolvent {
                        liabilities: 1_100_000,
                        assets: 1_050_000,
                        basis_and_cash_after: 900_000,
                        amt_basis_and_cash_after: None,
                    },
                    property: vec![DebtProperty {
                        label: "Tractor".into(),
                        kind: DebtPropertyKind::Equipment { secured: true },
                        basis: 500_000,
                        amt_basis: Some(450_000),
                    }],
                    depreciable_first: 0,
                },
            }),
        ),
        document(
            0,
            IncomeDocument::Acquisition1099A(Acquisition1099A {
                lender: "Synthetic Land Bank".into(),
                date: 20260801,
                balance: 5_000_000,
                value: 3_000_000,
                liable: false,
                property: AcquiredProperty::Property {
                    kind: TransferredProperty::Investment {
                        basis: 4_000_000,
                        term: GainTerm::Long,
                        collectible: false,
                    },
                },
            }),
        ),
    ]
}

#[test]
fn canceled_debt_kinds_map_onto_records_and_reopen_unchanged() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&send(
        &mut session,
        save_topic(&original, TaxSection::CanceledDebt, canceled_debt_documents()),
    ));
    let stored = &session.draft.income[1..];
    assert_eq!(
        stored.iter().map(|e| e.label.as_str()).collect::<Vec<_>>(),
        vec!["Synthetic Auto Credit", "Synthetic Farm Credit", "Synthetic Land Bank"]
    );
    // Box 7 is the transfer.
    let FederalIncome::PersonalDebt { data: car } = &stored[0].income else { panic!() };
    assert_eq!(
        (car.personally_liable, car.transfer.as_ref().map(|t| t.value)),
        (true, Some(700_000))
    );
    let FederalIncome::PersonalDebt { data: farm } = &stored[1].income else { panic!() };
    let PersonalDebtTreatment::FarmDebt { data: facts } = &farm.treatment else { panic!() };
    assert_eq!(facts.insolvency.as_ref().map(|i| i.liabilities), Some(1_100_000));
    let FederalIncome::PropertyAcquisition { data: land } = &stored[2].income else { panic!() };
    assert_eq!((land.realized(), land.personally_liable), (5_000_000, false));
    // The return: the $1,500 car debt is income; the farm debt is excluded
    // ($11,000 - $10,500 = $500 by insolvency, which finds no attribute and
    // no basis room; the other $500 as farm debt within the $5,000 tractor's
    // basis, line 11a); the nonrecourse land realizes the whole
    // $50,000 debt against its $40,000 basis, a $10,000 gain.
    let r = supported(&session.draft);
    assert_eq!(value(&r, "agi"), value(&supported(&original), "agi") + 11_500);
    assert_eq!(value(&r, "debt_3_form982_line2"), 1000);
    // Reopening rebuilds the same rows; saving them unchanged stores the same
    // records.
    let page = open_topic(&mut session, TaxSection::CanceledDebt);
    assert_eq!(
        page_documents(&page).iter().map(|r| r.document.clone()).collect::<Vec<_>>(),
        canceled_debt_documents().into_iter().map(|r| r.document).collect::<Vec<_>>()
    );
    let before = session.draft.income.clone();
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.income, before);
}

#[test]
fn canceled_debt_rules_reject_on_both_sides_at_their_boundaries() {
    let original = complete(FederalStatus::Single);
    let attempt = |document: IncomeDocument| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_topic(&original, TaxSection::CanceledDebt, vec![self::document(0, document)]),
        );
        (reply, session.draft)
    };
    let insolvent = |liabilities, depreciable_first| CanceledDebtChoice::Insolvent {
        liabilities,
        assets: 0,
        basis_and_cash_after: 0,
        amt_basis_and_cash_after: None,
        business_debt: false,
        property: vec![],
        depreciable_first,
    };
    let seller = |basis, liabilities, sale: Option<u32>| CanceledDebtChoice::SellerReduction {
        basis,
        assets: 10_000_000,
        liabilities,
        sale: sale.map(|date| DebtPropertySale {
            date,
            term: GainTerm::Long,
            collectible: false,
            proceeds: 0,
            withholding: 0,
        }),
    };
    let real = |principal| CanceledDebtChoice::RealPropertyBusiness {
        solvency: DebtSolvency::Solvent,
        principal,
        value: 0,
        other_secured: 0,
        depreciable_basis: 10_000_000,
        property: vec![],
        depreciable_first: 0,
    };
    let farm_insolvent = |liabilities| CanceledDebtChoice::FarmDebt {
        solvency: DebtSolvency::Insolvent {
            liabilities,
            assets: 0,
            basis_and_cash_after: 0,
            amt_basis_and_cash_after: None,
        },
        property: vec![],
        depreciable_first: 0,
    };
    // Box 2 is $1,500.00; each pair is (accepted at the boundary, one cent
    // past it) with the client rule's key.
    for (ok, bad, key) in [
        (insolvent(150_000, 0), insolvent(149_999, 0), "DEBT_LIABILITIES_BELOW_CANCELED"),
        (farm_insolvent(150_000), farm_insolvent(149_999), "DEBT_LIABILITIES_BELOW_CANCELED"),
        (
            seller(150_000, 150_000, None),
            seller(149_999, 150_000, None),
            "DEBT_PRICE_REDUCTION_ABOVE_BASIS",
        ),
        (
            seller(150_000, 150_000, None),
            seller(150_000, 149_999, None),
            "DEBT_PRICE_REDUCTION_ABOVE_LIABILITIES",
        ),
        (
            seller(150_000, 150_000, Some(20260315)),
            seller(150_000, 150_000, Some(20260314)),
            "DEBT_SALE_BEFORE_REDUCTION",
        ),
        (real(150_000), real(149_999), "DEBT_PRINCIPAL_BELOW_CANCELED"),
        (insolvent(150_000, 150_000), insolvent(150_000, 150_001), "DEBT_ELECTION_ABOVE_CANCELED"),
    ] {
        accepted(&attempt(car_1099c(150_000, ok)).0);
        let (reply, draft) = attempt(car_1099c(150_000, bad.clone()));
        rejected_with(reply, key);
        assert_eq!(draft, original);
        let mut session = Interview::new(TEST_USER, original.clone());
        rejected_with(send(&mut session, catalog_preview_document(car_1099c(150_000, bad))), key);
    }
}

#[test]
fn canceled_debt_refusals_are_stored_named_and_open_the_page() {
    // Something else is stored as the named situation and refused with its
    // form line; the Form 982 refusals open the Canceled debt topic, which
    // the checklist lists and the Less common income list no longer offers.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&send(
        &mut session,
        save_topic(
            &original,
            TaxSection::CanceledDebt,
            vec![document(
                0,
                car_1099c(
                    150_000,
                    CanceledDebtChoice::SomethingElse {
                        situation: UnsupportedDebt::PrincipalResidence,
                    },
                ),
            )],
        ),
    ));
    let FederalIncome::PersonalDebt { data } = &session.draft.income[1].income else { panic!() };
    assert_eq!(
        data.treatment,
        PersonalDebtTreatment::Unsupported { situation: UnsupportedDebt::PrincipalResidence }
    );
    let r = estimate(&session.draft);
    let issue = r.issues.iter().find(|i| i.message.contains("Form 982 lines 1e and 10b")).unwrap();
    assert_eq!(
        crate::interview::issue_page(
            &session.draft,
            &issue.context,
            Some({
                let m = &issue.display;
                m.key.as_str()
            })
        ),
        Some(TaxSection::CanceledDebt)
    );
    assert_eq!(
        crate::interview::issue_page(
            &session.draft,
            &crate::IssueContext::Section(TaxSection::CanceledDebt),
            None
        ),
        Some(TaxSection::CanceledDebt)
    );
    assert!(
        !RareSituation::allowed(TaxSection::MoreIncome).iter().any(|i| alloc::format!(
            "{:?}",
            RareSituation::ALL[*i as usize]
        )
        .contains("Canceled"))
    );
}

// The resource key and arguments of a row status.
fn status_key(value: &[u8]) -> (alloc::string::String, usize) {
    use libertas::AvroDecode;
    let mut offset = 0;
    let name = crate::common::FormattedMessage::avro_decode(value, &mut offset)
        .expect("a FormattedText status");
    (name.0, name.1.len())
}

#[test]
fn review_row_statuses_are_keyed_formatted_text() {
    // Every hub and review row status is a FormattedText of a declared
    // resource. A fixed state is its argument-free REVIEW_* key, the key
    // display.json's resource([status]) compares; text the App computes is
    // TEXT, and never spells a fixed state.
    let raw = include_str!("../../../tests/fixtures/returns/m1-single-w2.json");
    let (mut s, _, _) = enter_through_the_hubs("m1-single-w2", raw);
    let (items, _, _, answers) = review_state(&mut s);
    let mut rows = items;
    rows.extend(answers);
    let mut hubs = Vec::new();
    for section in [TaxSection::Income, TaxSection::Adjustments, TaxSection::Credits] {
        let hub = hub_of(from_review(&mut s, section));
        rows.extend(rows_of(&hub));
        hubs.push(hub);
    }
    let fixed: Vec<(&str, &str)> = APP_STRINGS
        .iter()
        .copied()
        .filter(|(k, t)| k.starts_with("REVIEW_") && !t.contains('{'))
        .collect();
    for row in &rows {
        let (key, arguments) = status_key(&row.status);
        assert!(APP_STRINGS.iter().any(|(k, _)| *k == key), "{key}");
        let text = status_text(&row.status);
        if let Some((state, _)) = fixed.iter().find(|(_, t)| *t == text) {
            assert_eq!((key.as_str(), arguments), (*state, 0), "{row:?}");
        }
    }
    // Each hub lists its untouched topics as not started, its less-common
    // destination as none, and the previous-section link by section name.
    for hub in &hubs {
        let rows = rows_of(hub);
        let more = rows[rows.len() - 2].clone();
        assert_eq!(status_key(&more.status), ("REVIEW_NONE".into(), 0), "{more:?}");
        assert!(
            rows.iter().any(|r| status_key(&r.status) == ("REVIEW_NOT_STARTED".into(), 0)),
            "{rows:?}"
        );
        assert!(status_key(&rows.last().unwrap().status).0.starts_with("SECTION_"));
    }
}

#[test]
fn attention_wraps_each_remaining_item_form() {
    use crate::interview::attention_status;
    use libertas::{LibertasMessageArgument as A, libertas_formatted_text as formatted};
    // A fixed remaining item keeps its full message as a nested argument.
    let unanswered = formatted("REVIEW_NOT_ANSWERED", &[]);
    let fixed = attention_status(&unanswered);
    assert_eq!(fixed, formatted("REVIEW_ATTENTION", &[A::formatted_text(&unanswered)]));
    assert_eq!(status_text(&fixed), "Needs attention: Not answered yet");
    // App text stays literal.
    let issue = attention_status(&formatted("TEXT", &[A::LiteralText("Enter the payer")]));
    assert_eq!(status_text(&issue), "Needs attention: Enter the payer");
    // A situation not calculated yet keeps its name argument.
    let refused = formatted("REVIEW_NOT_CALCULATED", &[A::LiteralText("Farming")]);
    assert_eq!(status_text(&refused), "Farming is not calculated yet");
    let wrapped = attention_status(&refused);
    assert_eq!(status_key(&wrapped).0, "REVIEW_ATTENTION_NOT_CALCULATED");
    assert_eq!(status_text(&wrapped), "Needs attention: Farming is not calculated yet");
}

#[test]
fn review_row_display_rule_compares_resource_keys() {
    // Display conditions are locale independent: the review row rule
    // compares the status's resource key, never its text.
    let display: serde_json::Value =
        serde_json::from_str(include_str!("../../../display.json")).unwrap();
    let rule = display["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["select"] == "^<ReviewItem>" && r["switch"]["cases"].is_array())
        .expect("the review row rule");
    for case in rule["switch"]["cases"].as_array().unwrap() {
        let condition = case["condition"].as_str().unwrap();
        assert!(!condition.contains("[status] =="), "{condition}");
        assert!(!condition.contains("[status] !="), "{condition}");
        let mut rest = condition;
        let mut keys = 0;
        while let Some(at) = rest.find("resource([status]) == \"") {
            rest = &rest[at + "resource([status]) == \"".len()..];
            let key = &rest[..rest.find('"').unwrap()];
            let text = APP_STRINGS
                .iter()
                .find(|(k, _)| *k == key)
                .unwrap_or_else(|| panic!("undeclared {key}"))
                .1;
            assert!(!text.contains('{'), "{key} takes no arguments");
            keys += 1;
        }
        // Every quoted literal is a compared key.
        assert_eq!(condition.matches('"').count(), keys * 2, "{condition}");
        assert!(keys > 0, "{condition}");
    }
}

// The topic sections of each hub, in order, for a return: the list only,
// without "What you told us" and previous-section navigation.
fn hub_topic_sets(d: FederalDraft) -> Vec<Vec<TaxSection>> {
    let mut s = Interview::new(TEST_USER, d);
    [TaxSection::Income, TaxSection::Adjustments, TaxSection::Credits]
        .into_iter()
        .map(|section| {
            let P::Hub { told, topics, previous_section, .. } =
                hub_of(from_review(&mut s, section))
            else {
                unreachable!()
            };
            // "What you told us" and previous navigation are their own fields, never
            // list members.
            assert_eq!(told.section, TaxSection::Checklist);
            assert_eq!(status_text(&previous_section.item), "Previous step");
            // The list is titled by its section's name, a keyed resource.
            let title = match section {
                TaxSection::Income => "SECTION_INCOME",
                TaxSection::Adjustments => "SECTION_ADJUSTMENTS",
                _ => "SECTION_CREDITS",
            };
            assert_eq!(status_key(&topics.name), (title.into(), 0));
            assert!(
                topics
                    .items
                    .iter()
                    .all(|t| t.section != TaxSection::Checklist
                        && status_text(&t.item) != "Continue")
            );
            topics.items.iter().map(|t| t.section).collect()
        })
        .collect()
}

#[test]
fn hubs_list_every_topic_the_facts_allow_whatever_the_ticks() {
    // A topic's own tick is a plan: it orders the walk and sets the row's
    // status, never whether the row is listed. The same return with nothing
    // ticked and with the adjustments and credits ticks lists the same
    // topics, the "More" rows included.
    let unticked = complete(FederalStatus::Single);
    let mut ticked = unticked.clone();
    ticked.checklist.adjustments = AdjustmentTopics::STUDENT_LOAN.0;
    ticked.checklist.credit_topics = CreditTopics::TUITION.0;
    // Ticked topics come first (the walk's order); the set is the same.
    let sorted = |sets: Vec<Vec<TaxSection>>| {
        sets.into_iter()
            .map(|mut set| {
                set.sort_by_key(|t| *t as i32);
                set
            })
            .collect::<Vec<_>>()
    };
    let sets = hub_topic_sets(unticked.clone());
    assert_eq!(sorted(sets.clone()), sorted(hub_topic_sets(ticked)));
    let mut income = ALL_INCOME_TOPICS.to_vec();
    income.push(TaxSection::MoreIncome);
    assert_eq!(
        sets,
        vec![
            income,
            // Self-employed benefits need a business or partnership (a
            // fact); the saver's credit cannot apply at a $60,000 AGI.
            vec![
                TaxSection::Hsa,
                TaxSection::IraContributions,
                TaxSection::Educator,
                TaxSection::StudentLoan,
                TaxSection::EmployeeExpenses,
                TaxSection::MoreAdjustments,
            ],
            vec![
                TaxSection::Care,
                TaxSection::Education,
                TaxSection::Marketplace,
                TaxSection::MoreCredits,
            ],
        ]
    );
    // Nothing ticked or entered: every row reads not started, and the
    // catch-all rows none.
    let mut s = Interview::new(TEST_USER, unticked);
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(
        hub_rows(&hub),
        hub_tail(
            &[
                TaxSection::Hsa,
                TaxSection::IraContributions,
                TaxSection::Educator,
                TaxSection::StudentLoan,
                TaxSection::EmployeeExpenses,
            ],
            TaxSection::MoreAdjustments
        )
    );
}

#[test]
fn a_less_common_adjustment_saved_without_its_tick_ticks_it() {
    // Nothing ticked: More adjustments is listed, opens from its row, and
    // saving an alimony agreement there ticks "Something less common"
    // (follow_ticks); the row then counts the agreement.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert_eq!(s.draft.checklist.adjustments, 0);
    let hub = open_adjustments_hub(&mut s);
    let more = row_for(&hub, TaxSection::MoreAdjustments);
    assert_eq!(status_key(&more.status), ("REVIEW_NONE".into(), 0));
    let Reply::Edit(mut request @ P::SaveMoreAdjustments { .. }) =
        send(&mut s, tap_row(&hub, more))
    else {
        panic!("expected More adjustments")
    };
    let P::SaveMoreAdjustments { value, .. } = &mut request else { unreachable!() };
    value.adjustments = vec![MoreAdjustment::AlimonyPaid(AlimonyItem {
        owner: Owner::Taxpayer,
        payment: alimony_record(20181231),
    })];
    let reply = send(&mut s, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(
        status_text(&row_for(&hub, TaxSection::MoreAdjustments).status),
        "1 alimony agreement"
    );
}

// Reply sizes for a complete joint return (Hub, a topic page, Review): the
// review carries the estimate; every other page only what it shows.
#[test]
fn replies_carry_only_what_their_page_shows() {
    extern crate std;
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let size = |r: &Reply| match r {
        Reply::Response(p) | Reply::Edit(p) => p.to_avro().len(),
    };
    let review = send(&mut s, P::ReviewAccepted { user: TEST_USER });
    let hub = from_review(&mut s, TaxSection::Income);
    let wages = from_review(&mut s, TaxSection::Wages);
    std::println!(
        "reply bytes: review {}, hub {}, wages page {}",
        size(&review),
        size(&hub),
        size(&wages)
    );
    assert!(matches!(review, Reply::Response(P::Review { .. })));
    assert!(size(&review) < 32_000, "{}", size(&review));
    assert!(size(&hub) < 2_000 && size(&wages) < 1_000);
}

// Plans are not completeness (walk-owned data): the tick navigates the
// first walk only and afterwards follows its data.

// The student loan page opened from the adjustments hub's row, saved with
// `forms` and `other` interest; the hub it returns to.
fn student_loan_from_the_hub(s: &mut Interview, forms: Vec<Form1098E>, other: i64) -> P {
    let hub = open_adjustments_hub(s);
    let row = row_for(&hub, TaxSection::StudentLoan);
    let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(s, tap_row(&hub, row)) else {
        panic!("expected the student loan page")
    };
    let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
    value.forms = forms;
    value.other_interest = other;
    accepted(&send(s, request));
    open_adjustments_hub(s)
}
fn form_1098_e(interest: i64) -> Form1098E {
    Form1098E { interest, box_2: Form1098EBox2::NotChecked }
}
fn student_loan_row(hub: &P) -> alloc::string::String {
    status_text(&row_for(hub, TaxSection::StudentLoan).status)
}
fn student_loan_ticked(s: &Interview) -> bool {
    s.draft.checklist.adjustments & AdjustmentTopics::STUDENT_LOAN.0 != 0
}

#[test]
fn an_entry_without_its_tick_sets_the_tick() {
    // Unticked on the checklist, the topic is still listed; a 1098-E saved
    // through its row ticks it, the row shows the entry, and the review has
    // no item for it.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert!(!student_loan_ticked(&s));
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(student_loan_row(&hub), "Not started");
    let hub = student_loan_from_the_hub(&mut s, vec![form_1098_e(60_000)], 0);
    assert!(student_loan_ticked(&s));
    assert_eq!(student_loan_row(&hub), "Recorded");
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
}

#[test]
fn a_tick_follows_its_entries_after_the_first_walk() {
    // Ticked with nothing entered: "Planned", a soft reminder, never an
    // item needing review. An entry replaces it; removing the last entry
    // clears the tick and the row reads "Not started".
    let mut d = complete(FederalStatus::Single);
    d.checklist.adjustments = AdjustmentTopics::STUDENT_LOAN.0;
    let mut s = Interview::new(TEST_USER, d);
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(student_loan_row(&hub), "Planned");
    let (items, _, _, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    let hub = student_loan_from_the_hub(&mut s, vec![form_1098_e(60_000)], 0);
    assert_eq!(student_loan_row(&hub), "Recorded");
    assert!(student_loan_ticked(&s));
    let hub = student_loan_from_the_hub(&mut s, vec![], 0);
    assert!(!student_loan_ticked(&s));
    assert_eq!(student_loan_row(&hub), "Not started");
    let (items, _, _, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
}

#[test]
fn a_plan_navigates_the_first_walk_only() {
    // The first walk visits the planned topic; afterwards Next (resume)
    // never re-enters it for its tick, though it stays planned. A tick newly
    // set on the checklist is walked like a first-walk plan.
    let mut d = complete(FederalStatus::Single);
    d.finished_sections.retain(|s| !matches!(s, TaxSection::Adjustments | TaxSection::StudentLoan));
    d.checklist.adjustments = AdjustmentTopics::STUDENT_LOAN.0;
    let mut s = Interview::new(TEST_USER, d);
    let Reply::Edit(request @ P::SaveStudentLoan { .. }) =
        send(&mut s, P::OpenInterview { user: TEST_USER })
    else {
        panic!("the first walk opens the planned topic")
    };
    // Next with nothing entered: the walk passes the topic.
    accepted(&send(&mut s, request));
    assert!(s.draft.finished_sections.contains(&TaxSection::StudentLoan));
    let draft = s.draft.clone();
    let reply = send(&mut s, go_to(&draft, PAGE_ADJUSTMENTS, TaxSection::Review));
    accepted(&reply);
    assert!(student_loan_ticked(&s));
    let reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    assert!(!matches!(reply, Reply::Edit(P::SaveStudentLoan { .. })), "{reply:?}");
    // Unticked and ticked again on the checklist: walked again.
    let draft = s.draft.clone();
    let reply = send(&mut s, navigate(&draft, TaxSection::Checklist));
    let value = checklist_of(&reply);
    let reply = send(&mut s, tick(reply, Checklist { adjustments: 0, ..value }));
    accepted(&reply);
    assert!(!student_loan_ticked(&s));
    let draft = s.draft.clone();
    let reply = send(&mut s, navigate(&draft, TaxSection::Checklist));
    let value = checklist_of(&reply);
    accepted(&send(
        &mut s,
        tick(reply, Checklist { adjustments: AdjustmentTopics::STUDENT_LOAN.0, ..value }),
    ));
    assert!(!s.draft.finished_sections.contains(&TaxSection::StudentLoan));
    let reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    assert!(matches!(reply, Reply::Edit(P::SaveStudentLoan { .. })), "{reply:?}");
}

mod navigation_search {
    extern crate std;
    use super::*;

    #[test]
    fn navigation_grouping_preserves_every_stored_section_identity() {
        // Every stored section identity, counted from the type (a new topic
        // changes no expectation here).
        let sections: Vec<_> =
            (0..64u8).map_while(|id| TaxSection::from_avro(&[id * 2]).ok()).collect();
        assert_eq!(sections.len(), crate::interview::SEQUENCE.len());
        let availability = NavigationAvailability::from_sections(&sections);
        assert_eq!(availability.groups, (0..7).collect::<Vec<_>>());
        // The seven groups partition the sections.
        assert_eq!(
            [
                availability.return_sections.len(),
                availability.income.len(),
                availability.adjustments.len(),
                availability.deductions.len(),
                availability.credits.len(),
                availability.other_taxes.len(),
                availability.payments.len(),
            ]
            .iter()
            .sum::<usize>(),
            sections.len()
        );
        for (id, section) in sections.into_iter().enumerate() {
            assert_eq!(section as usize, id);
            let destination = NavigationDestination::from(section);
            assert_eq!(
                NavigationDestination::from_avro(&destination.to_avro()).unwrap(),
                destination
            );
            assert_eq!(destination.section(), section);
        }
    }

    #[test]
    fn all_five_search_entries_expose_the_same_eligible_destinations() {
        let (mut session, _) = at_the_income_hub();
        let d = session.draft.clone();
        let Reply::Edit(editor @ P::SaveDeductions { .. }) =
            send(&mut session, go_to(&d, PAGE_INCOME, TaxSection::Deductions))
        else {
            panic!("deductions must open before finishing income")
        };
        let mut editor = editor;
        if let P::SaveDeductions { value, .. } = &mut editor {
            value.route = DeductionRoute::Compare;
        }
        send(&mut session, editor);
        let d = session.draft.clone();
        assert!(!d.finished_sections.contains(&TaxSection::Income));
        let mut expected = None;
        for section in [
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Itemized,
            TaxSection::Credits,
            TaxSection::Review,
        ] {
            let Reply::Response(page) = send(&mut session, go_to(&d, PAGE_REVIEW, section)) else {
                panic!("expected hub or review")
            };
            let (allowed, ceiling, primary, go_to_actions, navigation, shown) = match &page {
                P::Hub {
                    allowed_sections,
                    allowed_actions,
                    primary_actions,
                    go_to_actions,
                    navigation,
                    told,
                    topics,
                    previous_section,
                    ..
                } => (
                    allowed_sections,
                    allowed_actions,
                    primary_actions,
                    go_to_actions,
                    navigation,
                    topics
                        .items
                        .iter()
                        .chain([told, previous_section])
                        .map(|row| row.section)
                        .collect::<Vec<_>>(),
                ),
                P::Review {
                    allowed_sections,
                    allowed_actions,
                    primary_actions,
                    go_to_actions,
                    navigation,
                    remaining_items,
                    answers,
                    ..
                } => (
                    allowed_sections,
                    allowed_actions,
                    primary_actions,
                    go_to_actions,
                    navigation,
                    remaining_items
                        .iter()
                        .chain(answers)
                        .map(|row| row.section)
                        .collect::<Vec<_>>(),
                ),
                _ => panic!("unexpected page"),
            };
            assert_eq!(go_to_actions, &vec![libertas_macros::variant_index!(P::Navigate) as i32]);
            assert!(primary.iter().all(|id| ceiling.contains(id)
                && !go_to_actions.contains(id)
                && *id != libertas_macros::variant_index!(P::OpenSection) as i32));
            assert!(shown.iter().all(|section| allowed.contains(&(*section as i32))));
            let mut offered: Vec<_> = [
                &navigation.return_sections,
                &navigation.income,
                &navigation.adjustments,
                &navigation.deductions,
                &navigation.credits,
                &navigation.other_taxes,
                &navigation.payments,
            ]
            .into_iter()
            .flatten()
            .copied()
            .collect();
            offered.sort();
            offered.dedup();
            let mut allowed = allowed.clone();
            allowed.sort();
            assert_eq!(offered, allowed);
            assert!(allowed.contains(&(TaxSection::SelfEmployment as i32)));
            assert!(allowed.contains(&(TaxSection::Mortgage as i32)));
            assert!(allowed.contains(&(TaxSection::MoreCredits as i32)));
            assert!(!allowed.contains(&(TaxSection::Spouse as i32)));
            assert!(!allowed.contains(&(TaxSection::SelfEmployedBenefits as i32)));
            assert!(!allowed.contains(&(TaxSection::IncomeByPeriod as i32)));
            if let Some(expected) = &expected {
                assert_eq!(&allowed, expected);
            } else {
                expected = Some(allowed);
            }
            assert_eq!(P::from_avro(&page.to_avro()).unwrap(), page);
            // Optional local fixture for checking actual schema/client assembly.
            if let Ok(directory) = std::env::var("TAX_NAVIGATION_FIXTURES") {
                std::fs::create_dir_all(&directory).unwrap();
                std::fs::write(
                    std::path::Path::new(&directory).join(alloc::format!("{section:?}.avro")),
                    page.to_avro(),
                )
                .unwrap();
            }
        }
        assert_eq!(session.draft, d);
    }

    #[test]
    fn cross_section_open_and_save_preserve_unfinished_income() {
        let (mut session, _) = at_the_income_hub();
        let before = session.draft.clone();
        session.dirty = false;
        let Reply::Edit(mut editor @ P::SaveStudentLoan { .. }) =
            send(&mut session, go_to(&before, PAGE_INCOME, TaxSection::StudentLoan))
        else {
            panic!("cross-section choice must open the existing editor")
        };
        assert_eq!(session.draft, before);
        assert!(!session.dirty);
        if let P::SaveStudentLoan { value, .. } = &mut editor {
            value.other_interest = 125_000;
        }
        let reply = send(&mut session, editor);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(session.draft.revision, before.revision + 1);
        assert!(!session.draft.finished_sections.contains(&TaxSection::Income));
        assert!(matches!(
            session.draft.adjustments.as_ref().map(|value| &value.student_loan),
            Some(StudentLoanClaim { student_loan_interest: 125_000, .. })
        ));
        let d = session.draft.clone();
        send(&mut session, go_to(&d, PAGE_ADJUSTMENTS, TaxSection::Review));
        assert_eq!(session.draft, d);
    }

    #[test]
    fn navigation_rejects_wrong_group_and_unavailable_prerequisites() {
        let (mut session, _) = at_the_income_hub();
        let before = session.draft.clone();
        for section in [
            TaxSection::Spouse,
            TaxSection::Itemized,
            TaxSection::SelfEmployedBenefits,
            TaxSection::IncomeByPeriod,
            TaxSection::InjuredSpouse,
        ] {
            let request = go_to(&before, PAGE_INCOME, section);
            assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
        }
        let mut wrong = go_to(&before, PAGE_INCOME, TaxSection::Wages);
        if let P::Navigate { section, navigation, .. } = &mut wrong {
            *section = NavigationDestination::Deductions { section: TaxSection::Wages };
            *navigation = NavigationAvailability::from_sections(&[TaxSection::Wages]);
            navigation.groups = (0..8).collect();
            navigation.deductions.push(TaxSection::Wages as i32);
        }
        assert!(matches!(send(&mut session, wrong), Reply::Response(P::Problem { .. })));
        assert_eq!(session.draft, before);
        let mut before_filers =
            Interview::new(TEST_USER, FederalDraft::empty("search-prerequisite".into()));
        let d = before_filers.draft.clone();
        assert!(matches!(
            send(&mut before_filers, go_to(&d, PAGE_REVIEW, TaxSection::SelfEmployment)),
            Reply::Response(P::Problem { .. })
        ));
    }

    #[test]
    fn every_search_entry_uses_alias_and_exact_identifier_guidance() {
        let source = include_str!("../../core/protocol.rs");
        assert_eq!(
            source.matches("#[libertas_ai_navigation_prompt(\"TAX_NAVIGATION_PROMPT\")]").count(),
            2
        );
        let prompt = APP_STRINGS.iter().find(|(key, _)| *key == "TAX_NAVIGATION_PROMPT").unwrap().1;
        for phrase in [
            "every offered page",
            "bare identifiers case-insensitively",
            "exact documented identifier",
            "199a",
            "8995",
            "8995-A",
            "QBI deduction",
            "Self-employment page first",
            "return no match",
        ] {
            assert!(prompt.contains(phrase), "missing {phrase}");
        }
        let definition = &source[source.find("pub enum TaxSection").unwrap()
            ..source.find("pub enum NavigationDestination").unwrap()];
        assert!(
            definition.contains("Section 199A (199a), Form 8995 (8995) or Form 8995-A (8995-A)")
        );
        for description in definition.split("\n    /// ").skip(1) {
            assert!(!description.trim().is_empty());
        }
    }
}

// Gate fixes, batch B: what only the person supplies sits beside the imported
// boxes, in a record no import selects, so applying an import result (the
// selected boxes merged, the box 12 list replaced whole, as the import
// contract applies them) leaves it as the person typed it.
pub(in crate::tests) fn import_unselected(source: &str, item: &str) -> bool {
    let start = source.find(item).expect("item");
    let body = &source[start..];
    !body[..body.find("\n}").expect("item end")].contains("libertas_llm_doc_field")
}

#[test]
fn div_import_keeps_box_5_eligibility_answer() {
    let model = include_str!("../../federal/federal_model.rs");
    assert!(import_unselected(model, "pub struct DivAnswers {"));
    assert!(!model.contains("#[libertas_llm_doc_field]\n    pub answers: DivAnswers"));
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let dividend = Div1099 {
        answers: DivAnswers { reit_eligible: Some(true), after_death: None },
        more: DivMore { section199a: 40_000, ..Default::default() },
        ..native_div1099("".into(), 100_000, 0)
    };
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::Div1099(dividend))]),
    ));
    // The import: box 5 printed with a new amount, the answer untouched.
    let reopened =
        page_documents(&open_topic(&mut session, TaxSection::InterestDividends)).to_vec();
    let IncomeDocument::Div1099(mut imported) = reopened[0].document.clone() else {
        panic!("expected the 1099-DIV")
    };
    imported.ordinary = 120_000;
    imported.more.section199a = 50_000;
    let request = save_documents(
        &session.draft.clone(),
        vec![document(reopened[0].id, IncomeDocument::Div1099(imported))],
    );
    accepted(&send(&mut session, request));
    let data = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Dividend { data } => Some(data),
            _ => None,
        })
        .expect("the dividend");
    assert_eq!(data.section199a, ReitDividends::Reported { amount: 50_000, eligible: true });
    let page = open_topic(&mut session, TaxSection::InterestDividends);
    let IncomeDocument::Div1099(again) = &page_documents(&page)[0].document else {
        panic!("expected the 1099-DIV")
    };
    assert_eq!(again.answers.reit_eligible, Some(true));
    // Box 5 with no answer is refused on the server too.
    let mut unanswered = again.clone();
    unanswered.answers.reit_eligible = None;
    let request = save_documents(
        &session.draft.clone(),
        vec![document(reopened[0].id, IncomeDocument::Div1099(unanswered))],
    );
    rejected_with(send(&mut session, request), "DIV_REIT_ELIGIBILITY_NEEDED");
}

#[test]
fn blank_prior_agi_is_stored_blank_and_reopens_blank() {
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    let mut choice =
        figure_with(PriorReturn::SameReturn, WithholdingTiming::Evenly, Farmer::No, Waiver::None);
    if let PenaltyChoice::Figure { prior_agi, .. } = &mut choice {
        *prior_agi = None;
    }
    let blank = Payments {
        estimated: vec![page_paid(20260415, 100_000)],
        penalty: choice,
        ..no_payments()
    };
    accepted(&submit(&mut s, blank.clone()));
    let pay = s.draft.payments.clone().unwrap();
    assert_eq!(pay.prior_year.as_ref().map(|p| p.agi), Some(None));
    let page = open_payments(&mut s);
    assert_eq!(payments_value(&page).0.penalty, blank.penalty);
}

// Form 8815 on the Interest and dividends page (F7). The savings-bond record
// names each person by identity, the key of the page's people catalog, so
// reopening the page returns the same rows whatever the catalog's order.
#[test]
fn savings_bonds_page_round_trips_people_by_identity_and_checks_its_rules() {
    use crate::tests::investment::investment::bond_return;
    let mut d = bond_return(FederalStatus::Joint, 100_000, 3_000);
    let child = d.dependents[0].id;
    // The page's 1099-INT replaces the fixture's statement.
    d.income.pop();
    let mut session = Interview::new(TEST_USER, d);
    let page = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { people, allowed_documents, .. } = &page else { unreachable!() };
    // You, your spouse and the dependent, keyed by identity; the filers' labels
    // are FormattedText resources, not typed text.
    assert_eq!(people.iter().map(|p| p.id).collect::<Vec<_>>(), vec![-1, -2, child]);
    assert_eq!(status_text(&people[0].label), "You");
    assert_eq!(status_text(&people[1].label), "Your spouse");
    assert_eq!(status_text(&people[2].label), alloc::format!("Child {child}"));
    assert!(allowed_documents.contains(&document_index_of_savings_bonds()));

    let int = Int1099 {
        more: IntMore { us_obligations: 300_000, ..Default::default() },
        ..native_int1099("Treasury".into(), 0)
    };
    let row = |person: i64, tuition: i64| SavingsBondStudent {
        person,
        expenses: SavingsBondExpenses::Entered { tuition, benefits: 0 },
        contributions: 5_000,
    };
    let mut bonds = SavingsBonds {
        proceeds: 1_200_000,
        redemption_interest: 300_000,
        earliest_issue_date: 20100601,
        interest_statement: 2,
        students: vec![row(child, 400_000), row(-2, 500_000), row(-1, 600_000)],
    };
    let owned = |owner, id, document| IncomeDocumentItem {
        refund_management: None,
        refund_management_needed: None,
        allowed_community: Vec::new(),
        investment_caps: None,
        id,
        owner,
        document,
        preview: None,
        community: CommunityShare::NotNeeded,
    };
    let request = save_topic(
        &session.draft,
        TaxSection::InterestDividends,
        vec![owned(Owner::Spouse, 0, IncomeDocument::Int1099(int))],
    );
    accepted(&send(&mut session, request));
    let interest_row = page_documents(&open_topic(&mut session, TaxSection::InterestDividends))
        .iter()
        .find(|r| matches!(r.document, IncomeDocument::Int1099(_)))
        .unwrap()
        .clone();
    bonds.interest_statement = interest_row.id;
    let documents =
        vec![interest_row, owned(Owner::Spouse, 0, IncomeDocument::SavingsBonds(bonds.clone()))];
    let request = save_topic(&session.draft, TaxSection::InterestDividends, documents);
    let reply = send(&mut session, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    // Stored by identity, in the page's order, owned by the spouse.
    let entry = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::SavingsBonds { .. }))
        .expect("the savings bond record");
    assert_eq!(entry.owner, Owner::Spouse);
    assert_eq!(entry.label, "");
    let FederalIncome::SavingsBonds { data } = &entry.income else { unreachable!() };
    assert_eq!(data.students.iter().map(|s| s.person).collect::<Vec<_>>(), vec![child, -2, -1]);
    assert_eq!(data.proceeds, 1_200_000);
    assert_eq!(data.earliest_issue_date, 20100601);
    // The stored return reopens as the same page row.
    session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
    let page = open_topic(&mut session, TaxSection::InterestDividends);
    let reopened: Vec<_> = page_documents(&page)
        .iter()
        .filter_map(|item| match &item.document {
            IncomeDocument::SavingsBonds(v) => Some((item.owner, v.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(reopened, vec![(Owner::Spouse, bonds.clone())]);
    // No amount is previewed: the exclusion depends on the whole return.
    assert!(
        page_documents(&page)
            .iter()
            .filter(|item| matches!(item.document, IncomeDocument::SavingsBonds(_)))
            .all(|item| item.preview.is_none())
    );

    // The page rules, on the server with their client keys.
    let reject = |bonds: SavingsBonds, twice: bool, key: &str| {
        let mut session = Interview::new(TEST_USER, session.draft.clone());
        let mut documents =
            vec![owned(Owner::Taxpayer, 0, IncomeDocument::SavingsBonds(bonds.clone()))];
        if twice {
            documents.push(owned(Owner::Taxpayer, 0, IncomeDocument::SavingsBonds(bonds)));
        }
        let before = session.draft.clone();
        let reply =
            send(&mut session, save_topic(&before, TaxSection::InterestDividends, documents));
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, before);
    };
    reject(bonds.clone(), true, "SAVINGS_BONDS_TWICE");
    reject(
        SavingsBonds { redemption_interest: 1_200_001, ..bonds.clone() },
        false,
        "SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS",
    );
    reject(
        SavingsBonds { earliest_issue_date: 19891231, ..bonds.clone() },
        false,
        "SAVINGS_BONDS_ISSUED_BEFORE_1990",
    );
    reject(
        SavingsBonds { earliest_issue_date: 20270101, ..bonds.clone() },
        false,
        "SAVINGS_BONDS_ISSUED_AFTER_2026",
    );
    // Preview: a pure request with no amount.
    let preview = {
        let mut request = catalog_preview_document(IncomeDocument::SavingsBonds(bonds));
        let P::PreviewDocument { people: request_people, .. } = &mut request else {
            unreachable!()
        };
        *request_people = people.clone();
        request
    };
    assert!(matches!(
        send(&mut session, preview),
        Reply::Response(P::IncomePreview { preview: None, .. })
    ));
}
fn document_index_of_savings_bonds() -> i32 {
    libertas_macros::variant_index!(IncomeDocument::SavingsBonds) as i32
}

// Form 8818 import: the AI import reads the three printed facts (line 6, line
// 8 and the earliest column (b) issue date); the owner, the earlier-return
// source link and the people stay the filer's.
#[test]
fn savings_bonds_import_reads_form_8818_and_leaves_personal_records() {
    let model = include_str!("../../federal/federal_model.rs");
    let page = &model[model.find("pub struct SavingsBonds {").unwrap()..];
    let page = &page[..page.find("\n}").unwrap()];
    let head = &model[..model.find("pub struct SavingsBonds {").unwrap()];
    let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    assert!(action.contains("Form 8818"));
    assert!(action.contains("line 6 total redemption proceeds into proceeds"));
    assert!(action.contains("line 8 total interest into redemption_interest"));
    assert!(action.contains("earliest one as earliest_issue_date"));
    assert!(action.contains("validation = \"$.document_facts\""));
    let imported: Vec<&str> = page
        .lines()
        .scan(false, |marked, line| {
            let text = line.trim();
            if text.starts_with("#[libertas_llm_doc_field") {
                *marked = true;
                return Some(None);
            }
            if *marked && !text.starts_with("#[") && !text.starts_with("//") {
                *marked = false;
                return Some(text.trim_start_matches("pub ").split_once(':').map(|(n, _)| n));
            }
            Some(None)
        })
        .flatten()
        .collect();
    assert_eq!(imported, vec!["proceeds", "redemption_interest", "earliest_issue_date"]);
    assert_eq!(page.matches("(allow_empty = false)").count(), 3);
    // An import replaces exactly those fields of a row and keeps the rest.
    let mut row = SavingsBonds {
        proceeds: 1,
        redemption_interest: 0,
        earliest_issue_date: 20000101,
        interest_statement: 2,
        students: vec![SavingsBondStudent {
            person: -1,
            expenses: SavingsBondExpenses::EducationCredit,
            contributions: 0,
        }],
    };
    let kept = (row.interest_statement, row.students.clone());
    (row.proceeds, row.redemption_interest, row.earliest_issue_date) =
        (1_200_000, 300_000, 20100601);
    assert_eq!((row.interest_statement, row.students.clone()), kept);
}

// Form 8814 (src/child_election.rs): the Interest and dividends page.
pub(in crate::tests) fn election_page(child: i64, statements: Vec<ChildStatement>) -> Form8814 {
    Form8814 {
        child,
        statements,
        alaska_dividend: 0,
        nontaxable_social: 0,
        other_parent: OtherParentChoice::LivedApart,
        facts: 0,
    }
}
pub(in crate::tests) fn election_interest(taxable: i64) -> Int1099 {
    native_int1099("Child's Bank".into(), taxable)
}

// Form 6781 (unit B2): the 1099-B contracts kind and the carryback kind.
fn contracts_page(broker: &str, aggregate: i64) -> Contracts1099B {
    Contracts1099B {
        broker: broker.into(),
        aggregate,
        withholding: 12_345,
        special: false,
        state_details: Default::default(),
    }
}

#[test]
fn contracts_1099b_round_trips_onto_the_section_1256_record() {
    // F7: the broker is the entry label, box 11 and box 4 are the record,
    // the state boxes are kept in state_source, and a blank broker reopens
    // blank under the default label.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    s.draft.income.clear();
    let mut named = contracts_page("Example Futures Broker", -250_000);
    named.state_details = crate::StateWithholdingFields {
        payer: Default::default(),
        states: vec![crate::StateTaxWithholding {
            state: crate::TaxState::CA,
            payer_state_id: "0000123".into(),
            withholding: 777,
        }],
    };
    let blank = contracts_page("", 400_000);
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![
            document(0, IncomeDocument::Contracts1099B(named.clone())),
            owned(Owner::Taxpayer, 0, IncomeDocument::Contracts1099B(blank.clone())),
        ],
    );
    accepted(&send(&mut s, request));
    let stored = &s.draft.income;
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].label, "Example Futures Broker");
    assert_eq!(stored[1].label, "");
    assert_eq!(
        stored[0].income,
        FederalIncome::Section1256 {
            data: crate::FederalSection1256 {
                aggregate: -250_000,
                withholding: 12_345,
                special_treatment: false,
            }
        }
    );
    assert_eq!(
        stored[0].state_source,
        Some(crate::IncomeStateSource::Withholding(named.state_details.clone()))
    );
    assert_eq!(stored[1].state_source, None);
    // Reopened, the page reproduces both rows and saving them unchanged
    // reproduces the same records.
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    let rows: Vec<_> = page_documents(&reopened).iter().map(|item| item.document.clone()).collect();
    assert_eq!(
        rows,
        vec![IncomeDocument::Contracts1099B(named), IncomeDocument::Contracts1099B(blank)]
    );
    let before = s.draft.income.clone();
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        page_documents(&reopened).to_vec(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income, before);
    // The preview is the Form 6781 line 1 amount; a statement left out
    // previews zero through the supported working copy; the carryback kind
    // shows no amount.
    let preview = |s: &mut Interview, document: IncomeDocument| match send(
        s,
        catalog_preview_document(document),
    ) {
        Reply::Response(P::IncomePreview { preview, .. }) => preview,
        other => panic!("expected a preview, got {other:?}"),
    };
    assert_eq!(
        preview(&mut s, IncomeDocument::Contracts1099B(contracts_page("", -250_000))),
        Some(-250_000)
    );
    let mut left_out = contracts_page("", 400_000);
    left_out.special = true;
    assert_eq!(preview(&mut s, IncomeDocument::Contracts1099B(left_out)), Some(0));
    assert_eq!(
        preview(
            &mut s,
            IncomeDocument::Section1256Carryback(crate::Section1256Carryback {
                amount: 1,
                injured_owners: None
            })
        ),
        None
    );
}

#[test]
fn contracts_1099b_import_reads_box_11_box_4_and_the_state_boxes_only() {
    let model = include_str!("../../federal/federal_model.rs");
    let start = model.find("pub struct Contracts1099B {").unwrap();
    let page = &model[start..];
    let page = &page[..page.find("\n}").unwrap()];
    let head = &model[..start];
    let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    assert!(action.contains("boxes 8 through 11"));
    assert!(action.contains("Read box 11 aggregate profit or (loss) into aggregate"));
    assert!(action.contains("never figure it from boxes 8, 9 and 10"));
    assert!(action.contains("Read box 4 federal income tax withheld"));
    assert!(action.contains("Do not read boxes 1a, 8, 9 or 10"));
    assert!(action.contains("state boxes 14 through 16"));
    assert!(action.contains("validation = \"$.document_facts\""));
    let imported: Vec<&str> = page
        .lines()
        .scan(false, |marked, line| {
            let text = line.trim();
            if text.starts_with("#[libertas_llm_doc_field") {
                *marked = true;
                return Some(None);
            }
            if *marked && !text.starts_with("#[") && !text.starts_with("//") {
                *marked = false;
                return Some(text.trim_start_matches("pub ").split_once(':').map(|(n, _)| n));
            }
            Some(None)
        })
        .flatten()
        .collect();
    // The filer's hedge, election and dealer answer is never imported.
    assert_eq!(imported, vec!["broker", "aggregate", "withholding", "state_details"]);
    assert_eq!(page.matches("(allow_empty = false)").count(), 1);
    // A 1099-B sale entry sends a contracts form to the contracts kind.
    let sale = &model[..model.find("pub struct Sale1099B {").unwrap()];
    let sale = &sale[sale.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    assert!(sale.contains("belongs in a 1099-B futures and section 1256 options entry"));
}

// Form 8824 (unit B2): the rental page's third disposition choice.
pub(in crate::tests) fn exchange_fold() -> crate::RentalExchange {
    crate::RentalExchange {
        transferred: 20260620,
        identified: 20260725,
        received: 20260910,
        extended: None,
        replacement: crate::ExchangeReplacement {
            address: "9 Oak Lane".into(),
            value: 52_000_000,
            land: 12_000_000,
            used: crate::ReplacementUse::Rental {
                kind: RentalPropertyType::SingleFamily,
                in_service: 20260915,
            },
        },
        cash_received: 0,
        other_received: 0,
        debt_relieved: 15_000_000,
        debt_assumed: 20_000_000,
        cash_paid: 0,
        expenses: 2_800_000,
        land_value: 9_000_000,
        party: crate::ExchangeParty::Unrelated,
        special: false,
    }
}

pub(in crate::tests) fn related_exchange_kind(disposed: u32) -> crate::RelatedExchange {
    crate::RelatedExchange {
        property: "12 Elm St".into(),
        last_transfer: 20240501,
        disposed,
        exception: crate::RelatedException::None,
        deferred: 9_000_000,
        kind: crate::PropertyCharacter::LongTermCapital,
        special: false,
    }
}

pub(in crate::tests) fn roth_item_for_tests() -> R1099 {
    R1099 {
        code: DistributionCode::RothIra {
            sequence: None,
            rolled_over: 0,
            roth: RothIraCode::Qualified,
        },
        ..printed_as(r1099(1_000_000, DistributionCode::Nonqualified, false), "Q")
    }
}

// Form 1099-Q and the 529 and Coverdell expenses record on the Prizes,
// royalties, alimony and 529 plans page (unit B3). The page and the stored
// record name a student by identity, the key of the page's people catalog.
pub(in crate::tests) fn education_session() -> (Interview, i64) {
    let mut d = complete(FederalStatus::Joint);
    d.checklist.income |= IncomeTopics::OTHER_INCOME.0;
    add_child(&mut d);
    let child = d.dependents[0].id;
    (Interview::new(TEST_USER, d), child)
}
pub(in crate::tests) fn q1099_page(
    gross: i64,
    account: EducationAccount,
    transfer: Q1099Transfer,
    beneficiary: Q1099Beneficiary,
) -> Q1099 {
    Q1099 { gross, account, transfer, beneficiary }
}
pub(in crate::tests) fn plain_withdrawal() -> Q1099Transfer {
    Q1099Transfer::Withdrawal { reason: DistributionReason::Withdrawal, redeposited: 0 }
}

// Form 1099-Q import: the AI import reads the printed boxes (1, 2, 3, 4a/4b,
// 5, 6, 7 and its code) and leaves the filer's records alone.
#[test]
fn q1099_import_reads_the_printed_boxes_and_leaves_personal_records() {
    let source = include_str!("../../family/education_account.rs");
    let page = &source[source.find("pub struct Q1099 {").unwrap()..];
    let head = &source[..source.find("pub struct Q1099 {").unwrap()];
    let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    assert!(action.contains("Form 1099-Q"));
    assert!(action.contains("return Qtp when box 5a (private QTP) or 5b (state QTP) is checked"));
    assert!(action.contains("ValueOnly with the box 7 fair market value"));
    assert!(action.contains("TrusteeTransfer when box 4a is checked, RothIra when box 4b"));
    assert!(action.contains(
        "ExcessThisYear for code 2, ExcessLastYear for code 3, ProhibitedTransaction for code 6"
    ));
    assert!(action.contains("reason Disability for code 4, Death for code 5"));
    assert!(
        action.contains("return Recipient when it is not checked and Student when it is checked")
    );
    assert!(action.contains("validation = \"$.document_facts\""));
    // Everything up to the expenses record, whose rows are never imported.
    let page = &page[..page.find("/// 529 and Coverdell expenses").unwrap()];
    let imported: Vec<&str> = page
        .lines()
        .scan(false, |marked, line| {
            let text = line.trim();
            if text.starts_with("#[libertas_llm_doc_field") {
                *marked = true;
                return Some(None);
            }
            if *marked && !text.starts_with("#[") && !text.starts_with("//") {
                *marked = false;
                return Some(text.trim_start_matches("pub ").split_once(':').map(|(n, _)| n));
            }
            Some(None)
        })
        .flatten()
        .collect();
    assert_eq!(
        imported,
        vec![
            "gross",
            "account",
            "transfer",
            "beneficiary",
            "earnings",
            "basis",
            "amounts",
            "earnings",
            "basis",
            "value",
            "reason"
        ]
    );
    // Everything else (the account's basis and contributions, the redeposit,
    // the non-qualifying part, the 529-to-Roth facts, the student and the
    // costs) is the filer's: no other field is marked.
}

pub(in crate::tests) fn rental_sold_page(date: u32, price: i64) -> Rental {
    let mut r = rental_page(1_200_000);
    r.sale = crate::RentalSale::Sold {
        date,
        price,
        land_price: 8_000_000,
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    r
}

// Unit k1-rentals: boxes 2 and 3 on both K-1 kinds. The participation answer
// is the filer's own; a limited partner cannot actively participate
// (section 469(i)(6)(C)), on both sides.
pub(in crate::tests) fn k1_rental_boxes(real_estate: i64, other: i64) -> K1RentalBoxes {
    K1RentalBoxes {
        real_estate_section1231: 0,
        real_estate_unrecaptured: 0,
        other_section1231: 0,
        other_unrecaptured: 0,
        disposed: Vec::new(),
        real_estate,
        other,
        participation: K1RentalParticipation::Active,
        real_estate_qbi: Some(K1RentalQbi { wages: 120_000, property: 5_000_000 }),
        other_qbi: None,
        carryover_active: true,
    }
}

// ---------------------------------------------------------------------------
// Labels (unit labels, 2026-10-09): a label the App produces is FormattedText
// built from a string-resource key and typed arguments; a plain String label
// is only what the person typed. Every converted label is checked for its key
// and arguments, its English rendering and the round trip through storage, and
// every place that used a label as an identity is checked with equal names.

// A stored label as the key and typed arguments it carries.
fn label_message(bytes: &[u8]) -> Message {
    status_message(bytes)
}
pub(in crate::tests) fn numbered(key: &str, number: i64) -> Message {
    Message::new(key, vec![MessageArg::Number(number)])
}

#[test]
fn return_label_is_the_return_resource_with_the_tax_year() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let request = save_setup(&original, ReturnSetup { filing: Filing::MarriedJointly });
    send(&mut session, request);
    let label = &session.draft.setup.as_ref().unwrap().label;
    assert_eq!(label_message(label), numbered("LABEL_RETURN", 2026));
    assert_eq!(status_text(label), "2026 federal return");
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn person_labels_are_person_resources_with_the_typed_names() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    let students = crate::federal_rules::people_students(&d);
    assert_eq!(students.len(), 3);
    assert_eq!(label_message(&students[0].label), Message::new("PERSON_YOU", vec![]));
    assert_eq!(label_message(&students[1].label), Message::new("PERSON_SPOUSE", vec![]));
    assert_eq!(students[2].label, d.dependents[0].name());
    assert_eq!(
        label_message(&students[2].label),
        Message::new(
            "PERSON_NAME",
            vec![
                MessageArg::Text("Child".into()),
                MessageArg::Text(alloc::format!("{}", d.dependents[0].id)),
            ],
        )
    );
    assert_eq!(
        students.iter().map(|p| status_text(&p.label)).collect::<Vec<_>>(),
        vec![
            alloc::string::String::from("You"),
            alloc::string::String::from("Your spouse"),
            alloc::format!("Child {}", d.dependents[0].id),
        ]
    );
    // The filers' names are the same resource over the first and last name
    // each typed, derived where shown rather than stored (F3, 2026-10-09).
    let people = d.people.as_ref().unwrap();
    assert_eq!(
        people.taxpayer.name(),
        crate::interview::person_name(&people.taxpayer.first_name, &people.taxpayer.last_name)
    );
    assert_eq!(
        status_text(&people.taxpayer.name()),
        alloc::format!("{} {}", people.taxpayer.first_name, people.taxpayer.last_name)
    );
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn equally_named_payers_keep_their_own_foreign_tax_box() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let bank = |foreign_tax| Int1099 {
        more: IntMore { foreign_tax, special: InterestSpecial::NoneOfThese, ..Default::default() },
        ..native_int1099("Same Bank".into(), 50_000)
    };
    accepted(&send(
        &mut session,
        save_documents(
            &original,
            vec![
                document(0, IncomeDocument::Int1099(bank(10_000))),
                document(0, IncomeDocument::Int1099(bank(5_000))),
            ],
        ),
    ));
    // The amounts stay on the documents, so equal payers keep their own box 6;
    // the payee-statement taxes sum them.
    assert_eq!(crate::extended_rules::payee_statement_taxes(&session.draft), 15_000);
    let reopened: Vec<i64> = page_documents(&open_topic(
        &mut restart(session.draft.clone()),
        TaxSection::InterestDividends,
    ))
    .iter()
    .map(|item| match &item.document {
        IncomeDocument::Int1099(v) => v.more.foreign_tax,
        other => panic!("expected a 1099-INT, got {other:?}"),
    })
    .collect();
    assert_eq!(reopened, vec![10_000, 5_000]);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn equally_named_insured_people_keep_their_own_premiums() {
    let mut d = complete(FederalStatus::Single);
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Adjustments,
    ];
    let first = d.next_id;
    d.dependents.push(dependent_record(first, "Twin", "Sample", 20180101));
    d.dependents.push(dependent_record(first + 1, "Twin", "Sample", 20180101));
    d.next_id += 2;
    let mut s = Interview::new(TEST_USER, d);
    let page = match send(&mut s, P::OpenInterview { user: TEST_USER }) {
        Reply::Edit(page @ P::SaveDeductions { .. }) => page,
        other => panic!("expected the deductions page, got {other:?}"),
    };
    assert!(matches!(page, P::SaveDeductions { .. }));
    let request = save_deductions(&s.draft, DeductionRoute::Compare, 0);
    itemized_hub(send_past_less_common(&mut s, request));
    let P::SaveMedical { value, .. } = open_itemized_topic(&mut s, TaxSection::Medical) else {
        panic!("expected the medical page")
    };
    assert_eq!(
        value.people.iter().map(|p| status_text(&p.label)).collect::<Vec<_>>(),
        vec![
            alloc::string::String::from("You"),
            alloc::string::String::from("Twin Sample"),
            alloc::string::String::from("Twin Sample")
        ]
    );
    let request = save_medical(
        &s.draft,
        Medical {
            expenses: 0,
            long_term_care: vec![CarePremium {
                qualified: CareContractQualification::Qualified,
                person: first + 1,
                premiums: 60_000,
            }],
            people: value.people.clone(),
        },
    );
    itemized_hub(send_past_less_common(&mut s, request));
    let stored = stored_facts(&s).long_term_care;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].person, first + 1);
    // Reopening the page puts the premium on the second twin, not the first.
    let P::SaveMedical { value, .. } = open_itemized_topic(&mut s, TaxSection::Medical) else {
        panic!("expected the medical page")
    };
    assert_eq!(value.long_term_care.len(), 1);
    assert_eq!(value.long_term_care[0].person, first + 1);
}

// Entry display names: an unnamed income entry reads as its kind's
// capitalized noun over its identity, built in one owner.
#[test]
fn unnamed_income_entries_read_as_their_kind_noun_over_the_identity() {
    let mut d = complete(FederalStatus::Single);
    d.income[0].label = "".into();
    d.income.push(FederalIncomeEntry {
        refund_management: None,
        state_source: Default::default(),
        id: 5,
        label: "".into(),
        owner: Owner::Taxpayer,
        income: cash_business(0),
        community: CommunityShare::NotNeeded,
    });
    d.income.push(FederalIncomeEntry {
        refund_management: None,
        state_source: Default::default(),
        id: 6,
        label: "".into(),
        owner: Owner::Taxpayer,
        income: FederalIncome::Section1256 {
            data: crate::FederalSection1256 {
                aggregate: 100_000,
                withholding: 0,
                special_treatment: false,
            },
        },
        community: CommunityShare::NotNeeded,
    });
    let label = |entry: &FederalIncomeEntry| crate::interview::income_issue_context(entry).label();
    assert_eq!(label(&d.income[0]), numbered("LABEL_ENTRY_W2", 1));
    assert_eq!(status_text(&label(&d.income[0]).encode()), "W-2 1");
    assert_eq!(label(&d.income[1]), numbered("LABEL_ENTRY_BUSINESS", 5));
    assert_eq!(status_text(&label(&d.income[1]).encode()), "Business 5");
    assert_eq!(label(&d.income[2]), numbered("LABEL_ENTRY_1099_B_CONTRACTS", 6));
    assert_eq!(status_text(&label(&d.income[2]).encode()), "1099-B section 1256 contracts 6");
    // A name the person typed stays literal.
    d.income[1].label = "Acme".into();
    assert_eq!(label(&d.income[1]), Message::new("TEXT", vec![MessageArg::Text("Acme".into())]));
}

#[test]
fn a_form_8814_election_is_named_by_its_child_everywhere_it_is_shown() {
    let mut s = supporting(FederalStatus::Single);
    let original = s.draft.clone();
    accepted(&send(&mut s, save_children(&original, vec![row("Ava", "Sample", 20180615)])));
    let child = s.draft.dependents()[0].id;
    let request = save_documents(
        &s.draft.clone(),
        vec![document(
            0,
            IncomeDocument::Form8814(election_page(
                child,
                vec![ChildStatement::Int1099(election_interest(300_000))],
            )),
        )],
    );
    accepted(&send(&mut s, request));
    let entry = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::ChildElection { .. }))
        .unwrap();
    assert_eq!(entry.label, "");
    // Without the people page the election reads as its kind and identity.
    assert_eq!(
        crate::interview::income_issue_context(entry).label(),
        numbered("LABEL_ENTRY_FORM_8814", entry.id)
    );
    // The resolved context (what the review shows) and the entry's display
    // name read as the child's PERSON_NAME, never as the form and identity.
    let context = crate::interview::resolved_issue_context(
        &s.draft,
        &crate::interview::income_issue_context(entry),
    );
    assert_eq!(
        context.label(),
        Message::new("TEXT", vec![MessageArg::Encoded(s.draft.dependents()[0].name())])
    );
    assert_eq!(status_text(&context.label().encode()), "Ava Sample");
    assert_eq!(
        crate::interview::income_context(entry, s.draft.dependents()).label(),
        context.label()
    );
}

// Row 236: the payee-statement election has no typed statement list; the
// documents (1099 boxes and K-3 taxes) are its only source, and a save of
// Interest and dividends keeps a confirmed answer while a 1099 shows a tax.
#[test]
fn the_election_answer_survives_a_save_and_the_documents_are_its_only_source() {
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let claim = ForeignTaxClaim::Statements { eligible: Some(true) };
    assert!(crate::extended_rules::valid_credit(&claim));
    accepted(&send(
        &mut s,
        save_more_credits(
            &original,
            MoreCredits {
                credits: vec![MoreCredit::ForeignTax(claim.clone().try_into().unwrap())],

                credit_income_sources: Vec::new(),
            },
        ),
    ));
    assert_eq!(s.draft.credits.as_ref().unwrap().foreign_tax, claim);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    let bank = Int1099 {
        more: IntMore { special: InterestSpecial::NoneOfThese, ..Default::default() },
        ..native_int1099("Synthetic Bank".into(), 50_000)
    };
    // With no document showing a foreign tax, a save of the topic ends the
    // election: nothing else holds an amount.
    let request =
        save_documents(&s.draft.clone(), vec![document(0, IncomeDocument::Int1099(bank.clone()))]);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.credits.as_ref().unwrap().foreign_tax, ForeignTaxClaim::No);
    let request = save_more_credits(
        &s.draft.clone(),
        MoreCredits {
            credit_income_sources: Vec::new(),
            credits: vec![MoreCredit::ForeignTax(claim.clone().try_into().unwrap())],
        },
    );
    accepted(&send(&mut s, request));
    // A 1099 with a foreign tax keeps the confirmed answer through a save.
    let mut bank = bank;
    bank.more.foreign_tax = 5_000;
    let request =
        save_documents(&s.draft.clone(), vec![document(0, IncomeDocument::Int1099(bank))]);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.credits.as_ref().unwrap().foreign_tax, claim);
    assert_eq!(crate::extended_rules::payee_statement_taxes(&s.draft), 5_000);
}

// Forms 3468 and 4255 (unit form-3468): the investment credit on an asset
// row, a rental improvement row and a K-1, and the carryback year, through
// the pages. Page amounts are cents.
pub(in crate::tests) fn historic_claim(
    start: u32,
    expenditures: i64,
    confirmed: bool,
) -> crate::RehabilitationCredit {
    crate::RehabilitationCredit {
        period_start: start,
        phased: false,
        building_basis: 20_000_000,
        period_expenditures: expenditures,
        earlier: vec![],
        freed: 0,
        confirmed,
    }
}
pub(in crate::tests) fn solar_asset(began: u32) -> Asset {
    Asset {
        acquired: Some(20251001),
        investment_credit: crate::InvestmentCredit::NewEnergy {
            kind: crate::EnergyProperty::Solar,
            construction_began: began,
            increased_rate: true,
            domestic_content: false,
            energy_community: true,
            confirmed: true,
        },
        ..asset(12_000_000, 20260501, AssetClass::Five)
    }
}

// Form 3800 carryback years on the More credits page: the energy credit may
// be carried back to 2023, an ordinary credit only to 2025.
#[test]
fn the_carryback_year_is_chosen_per_credit() {
    let (mut s, _) = at_the_credits_hub(complete(FederalStatus::Single));
    let history = |credit: BusinessCredit, year: u32| BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![],
        carrybacks: vec![BusinessCreditCarryback {
            research_source: None,
            credit,
            year,
            amount: 100_000,
        }],
        spouse_without_credits: false,
    };
    let save = |s: &mut Interview, history: BusinessCreditHistory| {
        let request = save_more_credits(
            &s.draft,
            MoreCredits {
                credits: vec![MoreCredit::BusinessCredits(history.into())],

                credit_income_sources: Vec::new(),
            },
        );
        send(s, request)
    };
    rejected_with(
        save(&mut s, history(BusinessCredit::WorkOpportunity, 2024)),
        "BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED",
    );
    let before = s.draft.clone();
    // A carryback with no 2026 credit to carry is a calculation refusal, but
    // the page accepts the energy credit's earlier years.
    accepted(&save(&mut s, history(BusinessCredit::Energy, 2023)));
    assert_ne!(s.draft, before);
    let page = open_credits_topic(&mut s, TaxSection::MoreCredits);
    assert_eq!(
        more_credits_value(&page).credits,
        vec![MoreCredit::BusinessCredits(history(BusinessCredit::Energy, 2023).into())]
    );
}

pub(in crate::tests) fn saved_setup(s: &mut Interview, value: ReturnSetup) {
    let request = save_setup(&s.draft.clone(), value);
    accepted(&send(s, request));
}

#[test]
fn nonresident_spouse_election_f3_is_an_ordinary_joint_return_and_excludes_dual_status() {
    // Casey (citizen) W-2 $90,000, withheld $9,000; the spouse is a nonresident
    // alien with no income who joins in the section 6013(g) choice. Joint:
    // taxable 90,000 - 32,200 = 57,800; midpoint 57,825 = 2,480 + 12% x 33,025
    // = 6,443; refund 9,000 - 6,443 = 2,557.
    let mut joint = complete(FederalStatus::Joint);
    joint.income[0] = wage(1, Owner::Taxpayer, 9_000_000, 900_000);
    let mut s = Interview::new(TEST_USER, joint);
    save_whole_return(&mut s, WholeReturnTopics::NONRESIDENT_SPOUSE.0);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "15"), 5_780_000);
    assert_eq!(return_line(&r, "16"), 644_300);
    assert_eq!(r.refund, Some(255_700));
    assert!(noted(&r, "NOTE_NONRESIDENT_SPOUSE_TREATED_AS_A_RESIDENT"));
    // Without the choice the couple files separately: taxable 90,000 - 16,100 =
    // 73,900; midpoint 73,925 = 1,240 + 4,560 + 22% x 23,525 = 10,975.50.
    let mut separate = complete(FederalStatus::Separate);
    separate.income[0] = wage(1, Owner::Taxpayer, 9_000_000, 900_000);
    let r = supported(&separate);
    assert_eq!(return_line(&r, "15"), 7_390_000);
    assert_eq!(return_line(&r, "16"), 1_097_600);
    // The choice and a dual-status return exclude each other, on the page and on save.
    let both = WholeReturnTopics::NONRESIDENT_SPOUSE.0 | WholeReturnTopics::DUAL_STATUS.0;
    let before = s.draft.clone();
    rejected_with(save_whole_return(&mut s, both), "WHOLE_RETURN_RESIDENT_CHOICE_OR_DUAL_STATUS");
    assert_eq!(s.draft.checklist, before.checklist);
}

// A mortgage remains an accepted source record whether this return itemizes or not.
pub(in crate::tests) fn linked_mortgage_form(property: &str) -> Mortgage1098 {
    let mut loan = first_lender_mortgage(600_000, 10_000_000, AverageBalance::WithinLimit);
    loan.more.property = property.into();
    loan
}
pub(in crate::tests) fn linked_mortgage_editor(s: &mut Interview) -> P {
    let request = navigate(&s.draft, TaxSection::Mortgage);
    match send(s, request) {
        Reply::Edit(page @ P::SaveMortgages { .. }) => page,
        other => panic!("expected mortgage editor, got {other:?}"),
    }
}
pub(in crate::tests) fn save_linked_mortgage_rows(
    s: &mut Interview,
    loans: Vec<Mortgage1098>,
) -> Reply {
    let mut request = linked_mortgage_editor(s);
    let P::SaveMortgages { value, .. } = &mut request else { unreachable!() };
    value.loans = loans;
    send(s, request)
}
pub(in crate::tests) fn linked_mortgage_rows(s: &mut Interview) -> Vec<Mortgage1098> {
    let P::SaveMortgages { value, .. } = linked_mortgage_editor(s) else { unreachable!() };
    value.loans
}
pub(in crate::tests) fn accepted_linked_mortgage(s: &mut Interview) -> Mortgage1098 {
    let reply = save_linked_mortgage_rows(s, vec![linked_mortgage_form("Original loan")]);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    linked_mortgage_rows(s).remove(0)
}

#[test]
fn asset_page_keeps_the_earlier_use_the_partial_and_the_whole_business_sale() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(8_800_000);
    b.assets = vec![Asset {
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        // A prior election offers Records; the actual AMT balance stays unknown.
        allowed_amt_carry: vec![0],
        cost: 1_400_000,
        credit_reduction: 0,
        placed: 20230601,
        class: AssetClass::Seven,
        business_percent: 40,
        earlier_business_percent: Some(100),
        acquired: None,
        history: AssetHistory::Continuing {
            prior_depreciation: 1_400_000,
            first_year_179: 1_400_000,
            first_year_bonus: 0,
            carryforward: 0,
            mid_quarter: false,
        },
        disposal: crate::AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        partial: None,
    }];
    b.more.sale = Some(crate::BusinessSale {
        date: 20261101,
        inventory: 1_000_000,
        goodwill: 5_000_000,
        covenant: 500_000,
        special: false,
    });
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    assert_eq!(data.assets[0].earlier_business_percent, Some(100));
    assert_eq!(data.sale, b.more.sale);
    assert_round_trip(&mut session, &[b.clone()]);
    // The earlier use is refused on an asset used more than 50% now, and a
    // partial disposition on a class that is not a building.
    let mut high = b.clone();
    high.assets[0].business_percent = 80;
    rejected_with(
        save_businesses(&mut session, vec![high]),
        "ASSET_EARLIER_USE_ONLY_AFTER_USE_FELL",
    );
    let mut part = b.clone();
    part.assets[0].earlier_business_percent = None;
    part.assets[0].partial =
        Some(crate::PartialDisposition { date: 20260610, cost_of_part: 100_000, proceeds: 0 });
    rejected_with(
        save_businesses(&mut session, vec![part]),
        "PARTIAL_DISPOSITION_NEEDS_A_BUILDING_ROW",
    );
}

pub(in crate::tests) fn estate_final_source(
    nii: i64,
    in_h: bool,
) -> crate::EstateFinalCharacterPage {
    crate::EstateFinalCharacterPage {
        entity_terminated: true,
        final_beneficiary_k1: true,
        successor: true,
        section67e_nii: nii,
        section67e_in_h: in_h,
        additional: vec![],
        capital: crate::EstateFinalCapital {
            alternative_short: 0,
            alternative_long: 0,
            rate28: 0,
            unrecaptured: 0,
            niit_difference: None,
            regular_investment: None,
            alternative_investment: None,
            regular_business: None,
            alternative_business: None,
        },
    }
}
pub(in crate::tests) fn estate_final_amounts(section_67e: i64) -> K1FinalYear {
    K1FinalYear::Final {
        section_67e,
        personal_property_tax: 0,
        real_estate_tax: 0,
        state_income_tax: 0,
        state_sales_tax: 0,
        short_carry: 0,
        long_carry: 0,
        nol: None,
    }
}
pub(in crate::tests) fn main_estate_qbi(income: i64, wages: i64) -> crate::EstateQbiStatementsPage {
    crate::EstateQbiStatementsPage {
        statements: vec![crate::EstateQbiStatementPage {
            allocations: Vec::new(),
            name: "Fiduciary allocated business".into(),
            income,
            wages,
            property: 0,
            service: false,
            kind: crate::EstateQbiKind::Business,
            activities: vec![],
            passive_history: vec![],
        }],
    }
}

pub(in crate::tests) fn estate_nol_page(
    name: &str,
    year: u32,
    regular: i64,
    alternative: i64,
    remaining_years: Option<u32>,
) -> K1EstateTrust {
    let mut page = k1_1041(name);
    page.final_character = Some(estate_final_source(0, false));
    page.more.final_year = K1FinalYear::Final {
        section_67e: 0,
        personal_property_tax: 0,
        real_estate_tax: 0,
        state_income_tax: 0,
        state_sales_tax: 0,
        short_carry: 0,
        long_carry: 0,
        nol: Some(crate::FinalEstateTrustNol {
            terminated_on: None,
            years: vec![crate::FinalEstateTrustNolYear {
                original_period: None,
                year,
                remaining_years,
                regular_balance: regular * 100,
                alternative_balance: alternative * 100,
            }],
            fiduciary_history_complete: true,
            taxable_periods: vec![],
        }),
    };
    page
}

pub(in crate::tests) fn estate_nol_original_period(
    page: &mut K1EstateTrust,
    began: u32,
    ended: u32,
) {
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.terminated_on = Some(20261231);
    nol.years[0].original_period = Some(FinalEstateTrustLossPeriod { began, ended });
    let mut dates = vec![(began, ended)];
    if ended % 10000 != 1231 {
        let month = ended / 100 % 100;
        assert_eq!(ended % 100, crate::dates::days_in_month(ended / 10000, month));
        dates.push((ended / 10000 * 10000 + (month + 1) * 100 + 1, ended / 10000 * 10000 + 1231));
    }
    dates.extend((ended / 10000 + 1..=2026).map(|y| (y * 10000 + 101, y * 10000 + 1231)));
    nol.taxable_periods = dates
        .into_iter()
        .map(|(began, ended)| FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod { began, ended },
            accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
        })
        .collect();
}

pub(in crate::tests) fn estate_nol_weeks_history(page: &mut K1EstateTrust, include_pre2018: bool) {
    // Actual Friday-last-December annual boundaries. A normal364/371-day
    // year beginning lateDecember belongs to the next deemedJanuary year
    // for tax-law applicability, Reg1.441-2(c)(1), not its wire-date year.
    let mut dates = vec![
        (20171230, 20181228, 20181231),
        (20181229, 20191227, 20191231),
        (20191228, 20201225, 20201231),
        (20201226, 20211231, 20211231),
        (20220101, 20221230, 20221231),
        (20221231, 20231229, 20231231),
        (20231230, 20241227, 20241231),
        (20241228, 20251226, 20251231),
        (20251227, 20261225, 20261231),
    ];
    if include_pre2018 {
        dates.insert(0, (20161231, 20171229, 20171231));
    }
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.terminated_on = Some(20261225);
    nol.years[0].original_period =
        Some(FinalEstateTrustLossPeriod { began: dates[0].0, ended: dates[0].1 });
    nol.taxable_periods = dates
        .into_iter()
        .map(|(began, ended, reference_ended)| FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod { began, ended },
            accounting: FinalEstateTrustAccountingYear::Weeks52Or53 {
                annual_began: began,
                annual_ended: ended,
                reference_ended,
                closing: FinalEstateTrustWeekClosing::Last,
            },
        })
        .collect();
}

// Neutral explicit establishment facts for the existing synthetic fixtures.
// Published source dollars and expected worksheet arithmetic are unchanged.
pub(in crate::tests) fn se_health_test_months(
    months: Vec<BusinessHealthMonth>,
    partner: bool,
) -> SeHealth {
    SeHealth::Plan {
        establishment: if partner {
            HealthEstablishmentPage::Partnership { payment: EntityPremiumPayment::EntityPaid }
        } else {
            HealthEstablishmentPage::SoleProprietor
        },
        months,
        long_term_care: vec![],
    }
}
pub(in crate::tests) fn se_health_test_year(premiums: i64, partner: bool) -> SeHealth {
    se_health_test_months(
        (1..=12)
            .map(|month| BusinessHealthMonth {
                month,
                premiums: premiums / 12 + i64::from(i64::from(month) <= premiums % 12),
                child_increase: 0,
                employer_eligible: false,
                reimbursed: None,
            })
            .collect(),
        partner,
    )
}

/// Derived actual source allocation under Notice2008-1 Examples2-4 and
/// instructions7206 (2025 final; same establishment rule in2026draft).
/// https://www.irs.gov/irb/2008-02_IRB
/// https://www.irs.gov/instructions/i7206
/// Neutral completion: single adult45, one active S-corp K-1$10000, existing
/// W-2$11000/Medicare wages$5000. Its wages already include the actual$800
/// reimbursement. Ordinary premiums600 (child increase100) plus employer-
/// eligible February200; qualified LTC500. The entity actually funded100,
/// 200 and500 respectively, rather than a proportional$800/$1300 share.
#[test]
fn partial_entity_reimbursement_uses_actual_month_contract_and_child_subsets() {
    let mut k1 = k1_1120s("Actual reimbursement source");
    k1.ordinary = 1_000_000;
    k1.participation = Participation::Over500Hours { hours: 600 };
    let (mut s, reply) = save_k1_1120s(vec![k1]);
    accepted(&reply);
    s.draft.people.as_mut().unwrap().taxpayer.birth_date = 19810102;
    let mut wage = w2(1_100_000);
    wage.social_wages = 500_000;
    wage.social_tax = 31_000;
    wage.medicare_wages = 500_000;
    wage.medicare_tax = 7_250;
    let request =
        save_topic(&s.draft, TaxSection::Wages, vec![document(0, IncomeDocument::W2(wage))]);
    accepted(&send(&mut s, request));
    let wage =
        s.draft.income.iter().find(|e| matches!(e.income, FederalIncome::Wage { .. })).unwrap().id;
    let mut benefits = open_benefits(&mut s);
    benefits.scorporations[0].health = SeHealth::Plan {
        establishment: HealthEstablishmentPage::Scorporation {
            payment: EntityPremiumPayment::Reimbursed { amount: 80_000, included_in_income: true },
            wage,
        },
        months: vec![
            BusinessHealthMonth {
                month: 1,
                premiums: 60_000,
                child_increase: 10_000,
                employer_eligible: false,
                reimbursed: Some(ReimbursedHealthPremiums { premiums: 10_000, child_increase: 0 }),
            },
            BusinessHealthMonth {
                month: 2,
                premiums: 20_000,
                child_increase: 0,
                employer_eligible: true,
                reimbursed: Some(ReimbursedHealthPremiums { premiums: 20_000, child_increase: 0 }),
            },
        ],
        long_term_care: vec![BusinessCarePremiumPage {
            person: -1,
            qualified: CareContractQualification::Qualified,
            months: vec![BusinessCareMonth {
                month: 1,
                premiums: 50_000,
                reimbursed: Some(50_000),
                employer_eligible: false,
            }],
        }],
    };
    accepted(&save_benefits(&mut s, benefits.clone()));
    assert_eq!(open_benefits(&mut s), benefits);
    let activity = benefits.scorporations[0].activity;
    let medical_id = alloc::format!("business_{activity}_medical_premiums");
    let result = supported(&s.draft);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 600);
    assert_eq!(value(&result, &medical_id), 600);
    let before = s.draft.clone();
    let mut restarted = restart(before.clone());
    assert_eq!(open_benefits(&mut restarted), benefits);
    let mut reordered = benefits.clone();
    let SeHealth::Plan { months, .. } = &mut reordered.scorporations[0].health else { panic!() };
    months.reverse();
    accepted(&save_benefits(&mut s, reordered));
    assert_eq!(value(&supported(&s.draft), "schedule_1_self_employed_health_insurance"), 600);
    let mut wrong_kind = benefits.clone();
    let SeHealth::Plan { establishment, .. } = &mut wrong_kind.scorporations[0].health else {
        panic!()
    };
    *establishment = HealthEstablishmentPage::Scorporation {
        payment: EntityPremiumPayment::PartnershipPolicyYouPaid,
        wage,
    };
    let before = s.draft.clone();
    error(save_benefits(&mut s, wrong_kind));
    assert_eq!(s.draft, before);
    let mut missing = benefits.clone();
    let SeHealth::Plan { months, .. } = &mut missing.scorporations[0].health else { panic!() };
    months[0].reimbursed = None;
    error(current_source_refusal(
        "health-context-12",
        &mut s,
        benefits_request(&before, missing),
        "PROBLEM_INVALID_HEALTH_PLAN_SOURCE",
        Some("[value/scorporations/0/health]"),
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before);
    for (case, ordinary, child, care) in [
        (Some("health-context-13"), 10_001, 0, 50_000),
        (None, 10_000, 10_001, 50_000),
        (None, 10_000, 0, 50_001),
    ] {
        let mut invalid = benefits.clone();
        let SeHealth::Plan { months, long_term_care, .. } = &mut invalid.scorporations[0].health
        else {
            panic!()
        };
        months[0].reimbursed =
            Some(ReimbursedHealthPremiums { premiums: ordinary, child_increase: child });
        long_term_care[0].months[0].reimbursed = Some(care);
        let before = s.draft.clone();
        // Original13 crosses the first month's ordinary reimbursement by
        // one cent; the child and care bounds remain their own source cases.
        let reply = if let Some(case) = case {
            current_source_refusal(
                case,
                &mut s,
                benefits_request(&before, invalid),
                "PROBLEM_INVALID_HEALTH_PLAN_SOURCE",
                Some("[value/scorporations/0/health]"),
                SourceScope::Health,
            )
        } else {
            save_benefits(&mut s, invalid)
        };
        error(reply);
        assert_eq!(s.draft, before);
    }
    let mut child_funded = benefits.clone();
    let SeHealth::Plan { months, .. } = &mut child_funded.scorporations[0].health else { panic!() };
    months[0].reimbursed.as_mut().unwrap().child_increase = 10_000;
    accepted(&save_benefits(&mut s, child_funded));
    let result = supported(&s.draft);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 600);
    assert_eq!(value(&result, &medical_id), 700);
    let mut unknown = benefits;
    let SeHealth::Plan { long_term_care, .. } = &mut unknown.scorporations[0].health else {
        panic!()
    };
    long_term_care[0].qualified = CareContractQualification::Nonqualified;
    accepted(&save_benefits(&mut s, unknown.clone()));
    assert_eq!(open_benefits(&mut s), unknown);
    let result = supported(&s.draft);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 100);
    assert_eq!(value(&result, &medical_id), 600);
    assert!(noted(&result, "WARN_LTC_PREMIUM_LEFT_OUT"));
}

#[test]
fn validation_audit_f06_hsa_family_parts_zero_rejects_without_changing_saved_answers() {
    let mut session = at_the_adjustments_hub(FederalStatus::Joint, 0);
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value, .. } = &mut page else { panic!("expected the HSA page") };
    // Fresh account rows retain their ordinary null optional answers.
    value.accounts = vec![
        hsa_account(Owner::Taxpayer, HsaCoverageYear::Family, 0),
        hsa_account(Owner::Spouse, HsaCoverageYear::SelfOnly, 0),
    ];
    value.family_split = FamilySplit::Agreed { you: 0, spouse: 0 };
    let before = session.draft.clone();
    let dirty = session.dirty;
    rejected_with(send(&mut session, page), "HSA_FAMILY_PARTS_ZERO");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn validation_audit_f06_hsa_family_parts_keep_zero_for_either_spouse_and_reopen() {
    for (you, spouse) in [(0, 1), (1, 0), (1, 1), (100, 0), (0, 100)] {
        let mut session = at_the_adjustments_hub(FederalStatus::Joint, 0);
        let mut page = open_hsa_page(&mut session);
        let P::SaveHsa { value, .. } = &mut page else { panic!("expected the HSA page") };
        value.accounts = vec![
            hsa_account(Owner::Taxpayer, HsaCoverageYear::Family, 0),
            hsa_account(Owner::Spouse, HsaCoverageYear::SelfOnly, 0),
        ];
        value.family_split = FamilySplit::Agreed { you, spouse };
        accepted(&send(&mut session, page));
        let share = session.draft.adjustments.as_ref().unwrap().hsa_family_share.as_ref().unwrap();
        assert_eq!((share.taxpayer_parts, share.spouse_parts), (you, spouse));
        supported(&session.draft);
        let mut reopened = open_hsa_page(&mut session);
        let expected = if (you, spouse) == (1, 1) {
            FamilySplit::Equal
        } else {
            FamilySplit::Agreed { you, spouse }
        };
        assert_eq!(hsa_page_value(&reopened).0.family_split, expected);
        let P::SaveHsa { value, .. } = &mut reopened else { unreachable!() };
        value.family_split = FamilySplit::Agreed { you: 0, spouse: 0 };
        let before = session.draft.clone();
        let dirty = session.dirty;
        rejected_with(send(&mut session, reopened), "HSA_FAMILY_PARTS_ZERO");
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        assert_eq!(hsa_page_value(&open_hsa_page(&mut session)).0.family_split, expected);
    }
}

pub(in crate::tests) fn k1_sale_input_statement() -> crate::K1Section179Sale {
    {
        let amounts = crate::K1Section179SaleAmounts {
            price: 3_000_000,
            cost: 5_000_000,
            depreciation: 800_000,
            section179: 2_000_000,
        };
        crate::K1Section179Sale {
            description: "Packaging line".into(),
            acquired: 20230301,
            sold: 20260630,
            regular: Some(amounts.clone()),
            alternative: crate::K1Section179SaleAmtSource::NotReported,
            id: 0,
        }
    }
}

fn care_charge_document(printed: &str, reduction: i64) -> R1099 {
    let mut source =
        printed_as(r1099(1_000_000, DistributionCode::QualifiedCareCharge, false), printed);
    source.taxable = 0;
    source.more.box_8a = reduction;
    source
}

/// IRS final 2026 Form1099-R, CopyB recipient instructions, PDF p5 Box1;
/// final 2026 i1099r (printed June17), pp17/21: W/6W is not reported as
/// pension gross or taxable income; box8a is the actual investment reduction.
/// Synthetic completion: ordinary single wage return, no premium expense.
#[test]
fn code_w_actual_investment_reduction_round_trips_without_income_or_premium() {
    for (printed, reduction) in [("W", 0), ("W", 70_000), ("6W", 70_000)] {
        let original = complete(FederalStatus::Single);
        let baseline = supported(&original);
        let mut session = Interview::new(TEST_USER, original.clone());
        let source = care_charge_document(printed, reduction);
        let request = save_documents(
            &session.draft,
            vec![document(0, IncomeDocument::R1099(source.clone()))],
        );
        accepted(&send(&mut session, request));
        let entry = session
            .draft
            .income
            .iter()
            .find(|e| matches!(e.income, FederalIncome::Retirement { .. }))
            .unwrap();
        let id = entry.id;
        let FederalIncome::Retirement { data } = &entry.income else { unreachable!() };
        assert_eq!(entry.owner, Owner::Taxpayer);
        assert_eq!(data.gross, source.gross);
        assert_eq!(data.kind, RetirementKind::Nonqualified);
        assert_eq!(
            data.ordinary_distribution,
            RetirementTreatment::QualifiedCareCharge { investment_reduction: reduction }
        );
        assert!(!crate::care_benefits::annuity_source(&session.draft, id, Owner::Taxpayer, 1));
        let result = supported(&session.draft);
        assert_eq!(result.tax, baseline.tax);
        assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
        assert_eq!(
            value(&result, &alloc::format!("retirement_{id}_ltc_investment_reduction")),
            reduction / 100
        );
        for line in ["5a", "5b"] {
            let amount: i64 = result
                .return_lines
                .iter()
                .filter(|row| {
                    row.form_line
                        .as_ref()
                        .is_some_and(|form| form.form == TaxForm::Form1040 && form.line == line)
                })
                .map(|row| row.amount)
                .sum();
            assert_eq!(amount, 0, "{printed} {line}");
        }
        let mut session = restart(session.draft);
        let page = open_topic(&mut session, TaxSection::Retirement);
        let got = &page_documents(&page)[0];
        assert_eq!(got.id, id);
        assert_eq!(got.document, IncomeDocument::R1099(source));
        let P::SaveIncomes { annuity_sources, .. } =
            open_topic(&mut session, TaxSection::OtherIncome)
        else {
            unreachable!()
        };
        assert!(!annuity_sources.iter().any(|choice| choice.id == id));
    }
}

/// Actual printed W facts must agree on both the editable page and retained
/// record. A forged echoed treatment catalog does not admit another branch.
#[test]
fn code_w_rejects_conflicting_printed_or_retained_facts() {
    let original = complete(FederalStatus::Single);
    let mut cases = vec![];
    let mut v = care_charge_document("W", 1_000_001);
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.taxable = 1;
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.ira = true;
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.trump = true;
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.more.box_3 = 1;
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.code = DistributionCode::Nonqualified;
    cases.push(v.clone());
    v = care_charge_document("7W", 0);
    cases.push(v);
    for source in cases {
        assert!(DistributionCodes::treatment_rule(&source).is_some());
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut request =
            save_documents(&session.draft, vec![document(0, IncomeDocument::R1099(source))]);
        if let P::SaveIncomes { documents, .. } = &mut request
            && let IncomeDocument::R1099(row) = &mut documents.documents[0].document
        {
            row.allowed_treatments = vec![0, 6, 15];
            row.allowed_methods = vec![0, 1, 2];
        }
        assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
        assert_eq!(session.draft, original);
    }
    let mut session = Interview::new(TEST_USER, original);
    let request = save_documents(
        &session.draft,
        vec![document(0, IncomeDocument::R1099(care_charge_document("W", 70_000)))],
    );
    accepted(&send(&mut session, request));
    for change in 0..4 {
        let mut forged = session.draft.clone();
        let FederalIncome::Retirement { data } = &mut forged
            .income
            .iter_mut()
            .find(|e| matches!(e.income, FederalIncome::Retirement { .. }))
            .unwrap()
            .income
        else {
            unreachable!()
        };
        match change {
            0 => data.taxable = RetirementTaxable::Payer { amount: 1 },
            1 => data.kind = RetirementKind::Ira,
            2 => data.ordinary_distribution = RetirementTreatment::Ordinary,
            _ => {
                data.ordinary_distribution = RetirementTreatment::QualifiedCareCharge {
                    investment_reduction: data.gross + 1,
                }
            }
        }
        assert!(!forged.valid_stored(), "{change}");
    }
}

/// Notice2011-68 section3.02 reduces investment, not a second premium
/// deduction. Existing withdrawal inputs are already adjusted actual source
/// facts; the separate W record must not subtract its box8a again.
#[test]
fn code_w_does_not_subtract_already_adjusted_other_payment_basis_again() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original);
    let mut withdrawal = r1099(700_000, DistributionCode::Nonqualified, false);
    withdrawal.more.taxable_method = TaxableMethod::AnnuityWithdrawal {
        data: crate::AnnuityDistribution {
            loan_pledge: false,
            related: vec![],
            value: 1_000_000,
            investment: 600_000,
            eligible: true,
        },
    };
    let request = save_documents(
        &session.draft,
        vec![document(0, IncomeDocument::R1099(withdrawal.clone()))],
    );
    accepted(&send(&mut session, request));
    let baseline = supported(&session.draft);
    let mut documents = page_documents(&open_topic(&mut session, TaxSection::Retirement)).to_vec();
    documents.push(document(0, IncomeDocument::R1099(care_charge_document("W", 70_000))));
    let request = save_documents(&session.draft, documents);
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(result.tax, baseline.tax);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
}

#[test]
fn code_w_preview_is_transient_and_uses_the_same_actual_source_guards() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let preview = |source| {
        let mut request = catalog_preview_document(IncomeDocument::R1099(source));
        let P::PreviewDocument { foreign_countries, .. } = &mut request else { unreachable!() };
        *foreign_countries = vec![];
        request
    };
    assert!(matches!(
        send(&mut session, preview(care_charge_document("6W", 70_000))),
        Reply::Response(P::IncomePreview { preview: Some(0), .. })
    ));
    let mut invalid = care_charge_document("W", 70_000);
    invalid.taxable = 1;
    rejected_with(send(&mut session, preview(invalid)), "R_CARE_CHARGE_SOURCE_INVALID");
    assert_eq!(session.draft, before);
    assert!(!session.dirty);
}

pub(in crate::tests) fn fill_care_provider(page: &mut P) {
    let P::SaveCare { value, .. } = page else { panic!("Care page") };
    let paid = value.qualifying_people.iter().map(|p| p.expenses).sum::<i64>();
    if paid > 0 && value.providers.is_empty() {
        value.providers.push(CareProvider {
            id: 0,
            name: "Synthetic care provider".into(),
            address: "1 Example Way".into(),
            identification: ProviderIdentification::Organization,
            household_employee: false,
            paid,
            relationship: ProviderRelationship::NotRelated,
        });
    }
}
pub(in crate::tests) fn accept_care_provider(s: &mut Interview) {
    let mut page = open_credits_topic(s, TaxSection::Care);
    fill_care_provider(&mut page);
    accepted(&send(s, page));
}

include!("../passthrough/unit22_interview_registration.rs");
include!("../federal/unit47/interview_registration.rs");

#[test]
fn unit15_appraisal_catalog_normal_save_reopen_error_and_restart() {
    let mut s = at_the_gifts_page(DeductionRoute::Itemize, 0);
    let mut page = gifts(vec![], vec![], vec![]);
    page.appraisals.push(gift_appraisal(0, 20261115));
    page.appraisals[0].declaration_date = 20261201;
    // The signed PartIV declaration date is independent of the report/effective dates.
    let request = save_gift_carryovers(&s.draft, page);
    itemized_hub(send_past_less_common(&mut s, request));
    let P::SaveGiftCarryovers { value: mut page, appraisal_choices, .. } =
        open_itemized_topic(&mut s, TaxSection::GiftCarryovers)
    else {
        panic!("expected gifts")
    };
    let id = page.appraisals[0].id;
    assert!(id > 0);
    assert_eq!(page.appraisal_choices, appraisal_choices);
    assert_eq!(appraisal_choices[0].id, id);
    let mut gift = stock_gift(20200101, 800_000, 3_000_000);
    gift.kind = GiftKind::Other {
        category: PropertyCategory::Art,
        similar_kind: SimilarPropertyKind::Paintings,
        condition: "Good".into(),
        unrelated_use: false,
    };
    gift.value_method = ValueMethod::Appraisal;
    gift.appraisal = Some(id);
    page.gifts.push(gift.into());
    let request = save_gift_carryovers(&s.draft, page.clone());
    itemized_hub(send_past_less_common(&mut s, request));
    assert_eq!(
        value(&supported(&s.draft), "charity_capital_gain_thirty_current_deduction"),
        17_700
    );
    let before = s.draft.clone();
    let mut bad = page.clone();
    bad.gifts[0].appraisal = Some(id + 1000);
    let request = save_gift_carryovers(&s.draft, bad);
    rejected_with(send(&mut s, request), "GIFT_APPRAISAL_UNKNOWN");
    assert_eq!(s.draft, before);
    let mut again = restart(s.draft.clone());
    let P::SaveGiftCarryovers { value: reopened, .. } =
        open_itemized_topic(&mut again, TaxSection::GiftCarryovers)
    else {
        panic!("expected gifts")
    };
    assert_eq!(reopened, page);
    assert_eq!(supported(&again.draft), supported(&s.draft));
    let mut restored = reopened;
    restored.appraisals[0].effective_date = 20261116;
    let request = save_gift_carryovers(&again.draft, restored);
    rejected_with(send(&mut again, request), "GIFT_APPRAISAL_EFFECTIVE_DATE");
    assert_eq!(again.draft, s.draft);
}

#[test]
fn unit15_two_new_gifts_actual_client_save_reopen_and_restart() {
    let client = NativeInterviewCase("TAX_UNIT15_CLIENT");
    let mut s = at_the_gifts_page(DeductionRoute::Itemize, 0);
    let mut page = gifts(vec![], vec![], vec![]);
    page.appraisals.push(gift_appraisal(0, 20261115));
    page.appraisals[0].declaration_date = 20261201;
    let request = save_gift_carryovers(&s.draft, page);
    itemized_hub(send_past_less_common(&mut s, request));
    let seed = open_itemized_topic(&mut s, TaxSection::GiftCarryovers);
    client.write("SEED", &seed);
    let P::SaveGiftCarryovers { value: mut page, .. } = seed else { unreachable!() };
    let id = page.appraisals[0].id;
    for amount in [3_000_000, 1_000_000] {
        let mut gift = stock_gift(20200101, 800_000, amount);
        gift.kind = GiftKind::Other {
            category: PropertyCategory::Art,
            similar_kind: SimilarPropertyKind::Paintings,
            condition: "Good".into(),
            unrelated_use: false,
        };
        gift.value_method = ValueMethod::Appraisal;
        gift.appraisal = Some(id);
        page.gifts.push(gift.into());
    }
    let mut request = save_gift_carryovers(&s.draft, page.clone());
    client.write("ANSWERS", &request);
    if client.required_calculation(&mut s) {
        return;
    }
    request = client.input("PAYLOAD").unwrap_or(request);
    itemized_hub(send_past_less_common(&mut s, request));
    let ded = s.draft.deductions.as_ref().unwrap();
    assert_eq!(ded.property_gifts.len(), 2);
    assert!(ded.property_gifts.iter().all(|gift| gift.appraisal == Some(id)));
    assert_eq!(
        value(&supported(&s.draft), "charity_capital_gain_thirty_current_deduction"),
        17_700
    );
    assert_eq!(
        value(&supported(&s.draft), "charity_capital_gain_thirty_return_year_carryforward"),
        22_300
    );
    let mut restored = restart(s.draft.clone());
    let P::SaveGiftCarryovers { value: reopened, .. } =
        open_itemized_topic(&mut restored, TaxSection::GiftCarryovers)
    else {
        unreachable!()
    };
    assert_eq!(reopened, page);
    assert!(reopened.gifts.iter().all(|row| row.gift.appraisal.is_none()));
    assert_eq!(supported(&restored.draft), supported(&s.draft));
}

#[test]
fn unit15_original_map_carry_and_appraisal_errors_preserve_the_draft() {
    let client = NativeInterviewCase("TAX_UNIT15_CLIENT");
    let mut s = at_the_gifts_page(DeductionRoute::Itemize, 0);
    let before = s.draft.clone();
    let empty = || gifts(vec![], vec![], vec![]);
    let mut duplicate = empty();
    duplicate.carryovers =
        vec![prior_cash_charity(2025, 100).into(), prior_cash_charity(2025, 200).into()];
    let mut old = empty();
    old.carryovers = vec![prior_cash_charity(2020, 100).into()];
    let mut disaster = empty();
    disaster.carryovers = vec![
        CharityCarryover {
            research_parts: None,
            year: 2022,
            class: CharityClass::DisasterRelief,
            balance: 10_000,
            appreciation: None,
            same_filers: true,
        }
        .into(),
    ];
    let mut election = empty();
    election.basis_election = true;
    election.carryovers = vec![
        CharityCarryover {
            research_parts: None,
            year: 2025,
            class: CharityClass::CapitalGain30,
            balance: 10_000,
            appreciation: None,
            same_filers: true,
        }
        .into(),
    ];
    let mut condition = empty();
    let mut sofa = goods_gift(GoodsKind::Household, 200_000, 60_000);
    sofa.kind = GiftKind::Goods { kind: GoodsKind::Household, condition: GoodsCondition::NotGood };
    condition.gifts.push(sofa.into());
    let mut historic = stock_gift(20200101, 800_000, 3_000_000);
    historic.kind = GiftKind::Conservation {
        purpose: ConservationPurpose::OpenSpace,
        historic_building: Some(HistoricBuildingGift {
            rehabilitation_credits: 2_000_000,
            building_value: 100_000_000,
        }),
        nps_number: None,
        farmer_rancher: false,
    };
    let mut invalid_historic = empty();
    invalid_historic.gifts.push(historic.clone().into());
    let cases = [
        ("CHARITY_CARRY_TWICE", duplicate),
        ("CHARITY_CARRY_TOO_OLD", old),
        ("CHARITY_CARRY_DISASTER_YEAR", disaster),
        ("GIFT_ELECTION_NEEDS_CARRY_APPRECIATION", election),
        ("GIFT_ITEM_NEEDS_APPRAISAL", condition),
        ("GIFT_HISTORIC_CREDITS_PURPOSE", invalid_historic),
    ];
    for (key, page) in cases {
        let request = save_gift_carryovers(&s.draft, page);
        client.write_file("ERROR_PROBES", &alloc::format!("{key}.avro"), &request);
        rejected_with(send(&mut s, request), key);
        assert_eq!(s.draft, before, "{key}");
    }
    client.files("ERROR_PROBES", &[("manifest.json", br#"["CHARITY_CARRY_TWICE","CHARITY_CARRY_TOO_OLD","CHARITY_CARRY_DISASTER_YEAR","GIFT_ELECTION_NEEDS_CARRY_APPRECIATION","GIFT_ITEM_NEEDS_APPRAISAL","GIFT_HISTORIC_CREDITS_PURPOSE"]"#.to_vec())]);
    for (name, page) in [
        ("conservation_fifty", {
            let mut p = empty();
            p.carryovers.push(
                CharityCarryover {
                    research_parts: None,
                    year: 2011,
                    class: CharityClass::Conservation,
                    balance: 10_000,
                    appreciation: None,
                    same_filers: true,
                }
                .into(),
            );
            p
        }),
        ("conservation_hundred", {
            let mut p = empty();
            p.carryovers.push(
                CharityCarryover {
                    research_parts: None,
                    year: 2011,
                    class: CharityClass::Conservation100,
                    balance: 10_000,
                    appreciation: None,
                    same_filers: true,
                }
                .into(),
            );
            p
        }),
        ("disaster_2021", {
            let mut p = empty();
            p.carryovers.push(
                CharityCarryover {
                    research_parts: None,
                    year: 2021,
                    class: CharityClass::DisasterRelief,
                    balance: 10_000,
                    appreciation: None,
                    same_filers: true,
                }
                .into(),
            );
            p
        }),
        ("election_zero_appreciation", {
            let mut p = empty();
            p.basis_election = true;
            p.carryovers.push(
                CharityCarryover {
                    research_parts: None,
                    year: 2025,
                    class: CharityClass::CapitalGain30,
                    balance: 10_000,
                    appreciation: Some(0),
                    same_filers: true,
                }
                .into(),
            );
            p
        }),
        ("historic_structure", {
            let mut p = empty();
            if let GiftKind::Conservation { purpose, .. } = &mut historic.kind {
                *purpose = ConservationPurpose::HistoricStructure;
            }
            p.gifts.push(historic.into());
            p
        }),
        ("jewelry_condition", {
            let mut p = empty();
            let mut gift = goods_gift(GoodsKind::Jewelry, 200_000, 60_000);
            gift.kind =
                GiftKind::Goods { kind: GoodsKind::Jewelry, condition: GoodsCondition::NotGood };
            p.gifts.push(gift.into());
            p
        }),
    ] {
        let request = save_gift_carryovers(&before, page);
        client.write_file("ERROR_PROBES", &alloc::format!("valid_{name}.avro"), &request);
        let mut valid_session = Interview::new(TEST_USER, before.clone());
        itemized_hub(send_past_less_common(&mut valid_session, request));
    }
}

#[test]
fn unit15_two_new_appraisals_actual_client_allocate_and_reopen() {
    let client = NativeInterviewCase("TAX_UNIT15_APPRAISAL");
    let mut s = at_the_gifts_page(DeductionRoute::Itemize, 0);
    let seed = open_itemized_topic(&mut s, TaxSection::GiftCarryovers);
    client.write("SEED", &seed);
    let P::SaveGiftCarryovers { value: mut page, .. } = seed else { unreachable!() };
    for label in ["First independent appraiser", "Second independent appraiser"] {
        let mut appraisal = gift_appraisal(0, 20261115);
        appraisal.appraiser = label.into();
        page.appraisals.push(appraisal);
    }
    let mut request = save_gift_carryovers(&s.draft, page);
    client.write("ANSWERS", &request);
    request = client.input("PAYLOAD").unwrap_or(request);
    itemized_hub(send_past_less_common(&mut s, request));
    let appraisals = &s.draft.deductions.as_ref().unwrap().appraisals;
    assert_eq!(appraisals.len(), 2);
    assert!(appraisals[0].id > 0 && appraisals[1].id > 0 && appraisals[0].id != appraisals[1].id);
    assert_eq!(appraisals[0].appraiser, "First independent appraiser");
    assert_eq!(appraisals[1].appraiser, "Second independent appraiser");
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let P::SaveGiftCarryovers { value: reopened, appraisal_choices, .. } =
        open_itemized_topic(&mut again, TaxSection::GiftCarryovers)
    else {
        unreachable!()
    };
    assert_eq!(reopened.appraisals, *appraisals);
    assert_eq!(
        appraisal_choices.iter().map(|c| c.id).collect::<Vec<_>>(),
        appraisals.iter().map(|a| a.id).collect::<Vec<_>>()
    );
    assert_eq!(supported(&again.draft), supported(&saved));
    let mut duplicate = reopened.clone();
    duplicate.appraisals[1].id = duplicate.appraisals[0].id;
    let request = save_gift_carryovers(&again.draft, duplicate);
    client.write("DUPLICATE", &request);
    rejected_with(
        send(&mut again, request),
        "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
    );
    assert_eq!(again.draft, saved);
    let mut foreign = reopened;
    foreign.appraisals[0].id += 1000;
    let request = save_gift_carryovers(&again.draft, foreign);
    rejected_with(
        send(&mut again, request),
        "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
    );
    assert_eq!(again.draft, saved);
}

#[test]
fn unit4_elected_three_building_history_and_results_survive_save_restart_reopen() {
    let (mut d, _) = crate::tests::activity::rental_specials::unit4_three_buildings();
    d.income.truncate(1);
    let mut pages = Vec::new();
    for (label, rent, regular, amt) in [
        ("Building U", 15_000, 30_000, 24_000),
        ("Building V", 5_000, 10_000, 6_000),
        ("Building W", 20_000, 0, 0),
    ] {
        let mut page = rental_page(rent * 100);
        page.address = label.into();
        page.building.cost = 33_000_000;
        page.building.land = 5_500_000;
        page.building.in_service = 20200101;
        page.more.participation = RentalParticipation::Professional;
        page.more.suspended = SuspendedLoss {
            regular: regular * 100,
            amt: amt * 100,
            active_in_loss_year: regular == 0 && amt == 0,
        };
        pages.push(page);
    }
    let mut session = Interview::new(TEST_USER, d);
    accepted(&save_rentals(&mut session, pages.clone()));
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    assert_rentals_round_trip(&mut session, &pages);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 30_000);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 20_000);
    assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 0);
    let revision = session.draft.revision;
    let reopened = page_documents(&open_topic(&mut session, TaxSection::Rentals)).to_vec();
    let request = save_topic(&session.draft.clone(), TaxSection::Rentals, reopened);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, revision);
}

// Approved joint-taxpayer reconciliation: source owners and prior histories
// survive Save/reopen/restart, while the annual election covers both spouses.
#[test]
fn unit4_joint_election_combines_spouse_sources_after_restart() {
    let client = NativeInterviewCase("TAX_UNIT4_ROWS");
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        make_professional(&mut d, owner, false);
    }
    d.people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    d.people
        .as_mut()
        .unwrap()
        .spouse
        .as_mut()
        .unwrap()
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    let mut documents = Vec::new();
    for (owner, rent, prior) in [(Owner::Taxpayer, 20_000, 0), (Owner::Spouse, 10_000, 10_000)] {
        let mut page = rental_page(rent * 100);
        page.address = if owner == Owner::Taxpayer {
            "Synthetic taxpayer building"
        } else {
            "Synthetic spouse building"
        }
        .into();
        page.building.cost = 33_000_000;
        page.building.land = 5_500_000;
        page.building.in_service = 20200101;
        page.more.participation = RentalParticipation::Professional;
        page.more.suspended =
            SuspendedLoss { regular: prior * 100, amt: prior * 100, active_in_loss_year: false };
        documents.push(owned(owner, 0, IncomeDocument::Rental(page)));
    }
    let mut request = save_topic(&d, TaxSection::Rentals, documents);
    let mut session = Interview::new(TEST_USER, d);
    client.write_file("DIR", "rows-answers.avro", &request);
    client.write_file("DIR", "rows-seed.avro", &open_topic(&mut session, TaxSection::Rentals));
    if client.required_calculation(&mut session) {
        return;
    }
    request = client.input("PAYLOAD").unwrap_or(request);
    accepted(&send(&mut session, request));
    let saved = client.restart(&mut session);
    let reopened_request = open_topic(&mut session, TaxSection::Rentals);
    client.write_file("DIR", "rows-reopened.avro", &reopened_request);
    let reopened = page_documents(&reopened_request).to_vec();
    assert_eq!(
        reopened.iter().map(|entry| entry.owner).collect::<Vec<_>>(),
        vec![Owner::Taxpayer, Owner::Spouse]
    );
    let r = supported(&session.draft);
    // The combined $10,000 income absorbs the spouse's $10,000 prior loss.
    // The superseded owner-split 70,000/10,000/10,000 oracle remains in the audit.
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 0);
    assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 0);
    // Either spouse alone may independently qualify the joint taxpayer.
    // The other spouse's personal status stays absent, not inferred/persisted.
    for qualified in [Owner::Taxpayer, Owner::Spouse] {
        let mut one = saved.clone();
        let people = one.people.as_mut().unwrap();
        if qualified == Owner::Taxpayer {
            people.spouse.as_mut().unwrap().real_estate_professional = None;
        } else {
            people.taxpayer.real_estate_professional = None;
        }
        let result = supported(&one);
        assert_eq!(value(&result, "agi"), 60_000);
        assert_eq!(value(&result, "rental_suspended_loss"), 0);
        assert_eq!(value(&result, "amt_rental_suspended_loss"), 0);
        assert_eq!(value(&result, "schedule_e_line43_professional_rentals"), 0);
        // The election's materiality consistency test also crosses source owners.
        let FederalIncome::Rental { data } = &mut one.income[2].income else { unreachable!() };
        data.participation = RentalParticipation::NotActive;
        assert_eq!(
            crate::extended_rules::professional_election_rule(&one),
            Some("ISSUE_PROFESSIONAL_ELECTION_ALL_RENTALS")
        );
    }
    let request = save_topic(&session.draft.clone(), TaxSection::Rentals, reopened);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved);
}

#[test]
fn unit4_joint_election_has_one_editor_and_default_false_cannot_revoke_it() {
    use crate::federal_model::ProfessionalRentalElection::{SeparateActivities, SingleActivity};
    let d = complete(FederalStatus::Joint);
    let mut session = Interview::new(TEST_USER, d);
    // The spouse qualifies first and records the existing annual election.
    let mut spouse = filer(session.draft.people.as_ref().unwrap().spouse.as_ref().unwrap());
    spouse.real_estate_professional =
        Some(ProfessionalRentals { single_activity: true, over_500_hours: true }.into());
    let request = save_spouse(&session.draft.clone(), spouse);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.professional_elections(Owner::Taxpayer), vec![1]);
    assert_eq!(session.draft.professional_elections(Owner::Spouse), vec![0, 1]);
    // A fresh qualification on the other page supplies the normal false default.
    let mut you = filer(&session.draft.people.as_ref().unwrap().taxpayer);
    you.real_estate_professional =
        Some(ProfessionalRentals { single_activity: false, over_500_hours: false }.into());
    let request = save_you(&session.draft.clone(), you);
    let Reply::Edit(P::SaveSpouse { spouse, allowed_professional_elections, .. }) =
        send(&mut session, request)
    else {
        panic!("spouse editor");
    };
    assert_eq!(allowed_professional_elections, vec![0, 1]);
    assert_eq!(spouse.real_estate_professional.as_ref().unwrap().single_activity, SingleActivity);
    assert!(!professional_status(&session, Owner::Taxpayer).unwrap().single_activity);
    assert!(professional_status(&session, Owner::Spouse).unwrap().single_activity);
    let saved = session.draft.clone();
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let request = P::Back {
        user: TEST_USER,
        cookie: saved.cookie.clone(),
        revision: saved.revision,
        previous: PAGE_PEOPLE,
    };
    let Reply::Edit(P::SavePeople { you, allowed_professional_elections, .. }) =
        send(&mut session, request)
    else {
        panic!("taxpayer editor");
    };
    assert_eq!(allowed_professional_elections, vec![1]);
    assert_eq!(you.real_estate_professional.as_ref().unwrap().single_activity, SingleActivity);
    let request = save_you(&session.draft.clone(), you);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved); // Hidden projected choice is not persisted twice.
    // Only the authoritative existing answer can turn off the annual election.
    let mut spouse = Filer::from(spouse);
    spouse.real_estate_professional.as_mut().unwrap().single_activity = SeparateActivities;
    let request = save_spouse(&session.draft.clone(), spouse);
    accepted(&send(&mut session, request));
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        assert!(!session.draft.professional(owner).unwrap().single_activity);
    }
    assert!(professional_status(&session, Owner::Spouse).unwrap().over_500_hours);
    assert!(!professional_status(&session, Owner::Taxpayer).unwrap().over_500_hours);
}

#[test]
fn unit4_joint_election_transfers_editor_when_personal_qualification_is_removed() {
    let mut d = complete(FederalStatus::Joint);
    make_professional(&mut d, Owner::Taxpayer, true);
    make_professional(&mut d, Owner::Spouse, false);
    d.people
        .as_mut()
        .unwrap()
        .taxpayer
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    // Conflicting retained flags use the elected answer, not the other's false.
    let mut session = Interview::new(TEST_USER, d);
    let mut you = filer(&session.draft.people.as_ref().unwrap().taxpayer);
    you.real_estate_professional = None;
    let request = save_you(&session.draft.clone(), you);
    let Reply::Edit(P::SaveSpouse { allowed_professional_elections, spouse, .. }) =
        send(&mut session, request)
    else {
        panic!("remaining qualifier");
    };
    assert_eq!(allowed_professional_elections, vec![0, 1]);
    assert!(professional_status(&session, Owner::Taxpayer).is_none());
    assert!(professional_status(&session, Owner::Spouse).unwrap().single_activity);
    assert!(!professional_status(&session, Owner::Spouse).unwrap().over_500_hours);
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let request = save_spouse(&session.draft.clone(), spouse.into());
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved);
}

// The optional environment bridge exercises the actual Studio editor using
// complete normal protocol values. The ordinary test still controls native
// Save/reopen/restart and the independent combined former-passive amounts.
#[test]
fn unit4_new_qualification_actual_client_keeps_joint_election_after_restart() {
    let client = NativeInterviewCase("TAX_UNIT4_CLIENT");
    let mut d = complete(FederalStatus::Joint);
    make_professional(&mut d, Owner::Spouse, true);
    d.people
        .as_mut()
        .unwrap()
        .spouse
        .as_mut()
        .unwrap()
        .real_estate_professional
        .as_mut()
        .unwrap()
        .single_activity = true;
    for (owner, receipts, prior) in [(Owner::Taxpayer, 20_000, 0), (Owner::Spouse, 10_000, 10_000)]
    {
        let mut data = rental(receipts, 0);
        data.participation = RentalParticipation::Professional;
        data.history =
            RentalHistory { regular: prior * 100, amt: prior * 100, active_in_loss_year: false };
        add(&mut d, income(FederalIncome::Rental { data }));
        d.income.last_mut().unwrap().owner = owner;
    }
    let mut session = Interview::new(TEST_USER, d);
    let back = P::Back {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        previous: PAGE_PEOPLE,
    };
    let Reply::Edit(seed) = send(&mut session, back) else {
        panic!("people editor");
    };
    let mut first = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let back = P::Back {
        user: TEST_USER,
        cookie: first.draft.cookie.clone(),
        revision: first.draft.revision,
        previous: PAGE_PEOPLE,
    };
    let Reply::Edit(first_seed) = send(&mut first, back) else {
        panic!("first qualifier editor");
    };
    client.write("FIRST_SEED", &first_seed);
    client.write("SEED", &seed);
    let P::SavePeople { mut you, allowed_professional_elections, .. } = seed else {
        unreachable!()
    };
    assert_eq!(allowed_professional_elections, vec![1]);
    assert!(you.real_estate_professional.is_none());
    you.real_estate_professional =
        Some(ProfessionalRentals { single_activity: false, over_500_hours: false }.into());
    let mut request = save_you(&session.draft.clone(), you);
    client.write("ANSWERS", &request);
    if let Some(calc) = client.input("CALC_INPUT") {
        if matches!(calc, P::EstimateSpouse { .. }) {
            accepted(&send(&mut session, request.clone()));
        }
        let Reply::Response(response) = send(&mut session, calc) else {
            panic!("calculation response");
        };
        client.required_write("CALC_OUTPUT", &response);
        return;
    }
    request = client.input("PAYLOAD").unwrap_or(request);
    let Reply::Edit(spouse_editor @ P::SaveSpouse { .. }) = send(&mut session, request) else {
        panic!("joint spouse editor");
    };
    client.write("SPOUSE", &spouse_editor);
    let saved = session.draft.clone();
    assert!(!professional_status(&session, Owner::Taxpayer).unwrap().single_activity);
    assert!(!professional_status(&session, Owner::Taxpayer).unwrap().over_500_hours);
    assert!(professional_status(&session, Owner::Spouse).unwrap().single_activity);
    assert!(professional_status(&session, Owner::Spouse).unwrap().over_500_hours);
    client.restart(&mut session);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 100_000);
    assert_eq!(value(&result, "rental_suspended_loss"), 0);
    assert_eq!(value(&result, "amt_rental_suspended_loss"), 0);
    assert_eq!(value(&result, "schedule_e_line43_professional_rentals"), 0);
    let back = P::Back {
        user: TEST_USER,
        cookie: saved.cookie.clone(),
        revision: saved.revision,
        previous: PAGE_PEOPLE,
    };
    let Reply::Edit(reopened @ P::SavePeople { .. }) = send(&mut session, back) else {
        panic!("reopened people");
    };
    client.write("REOPENED", &reopened);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    if let Some(request) = client.input("REVOKE_PAYLOAD") {
        accepted(&send(&mut session, request));
        let session = restart(session.draft);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 110_000);
        assert_eq!(value(&result, "rental_suspended_loss"), 10_000);
        assert_eq!(value(&result, "schedule_e_line43_professional_rentals"), 10_000);
    }
}

#[test]
fn validation_audit_f01_hsa_null_excess_details_save_reopen_and_validate_present_values() {
    use libertas::AvroDecode;
    let mut session = at_the_adjustments_hub(FederalStatus::Joint, 0);
    session.draft.adjustments.as_mut().unwrap().hsa = vec![
        hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 437_500),
        hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 437_500),
    ];
    let page = open_hsa_page(&mut session);
    let P::SaveHsa { value: inputs, .. } = &page else {
        panic!("expected HSA editor");
    };
    assert!(inputs.accounts.iter().all(|account| account.excess_accounting.is_none()));
    let encoded = page.to_avro();
    accepted(&send(&mut session, P::avro_decode(&encoded, &mut 0).unwrap()));
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: inputs, .. } = &mut page else {
        panic!("expected reopened HSA editor");
    };
    assert!(inputs.accounts.iter().all(|account| account.excess_accounting.is_none()));
    inputs.accounts[0].excess_accounting = Some(HsaExcessAccounting {
        prior_year: None,
        employer_excess_in_wages: 0,
        year_end_value: 100,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
    });
    accepted(&send(&mut session, page));
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: inputs, .. } = &mut page else {
        panic!("expected reopened HSA editor");
    };
    let details = inputs.accounts[0].excess_accounting.as_mut().unwrap();
    assert_eq!(details.year_end_value, 100);
    details.later_return_year_deposits = 100;
    details.later_employer_deposits = 101;
    let before = session.draft.clone();
    rejected_with(send(&mut session, page), "HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER");
    assert_eq!(session.draft, before);
}

#[test]
fn validation_f10_continuing_risk_choices_follow_the_delivered_context() {
    for partnership in [false, true] {
        for history in [
            ScorpRiskStart::Unknown,
            ScorpRiskStart::Detailed { capacity: 50_000, profit: 0, loss: 0 },
        ] {
            let original = complete(FederalStatus::Single);
            let doc = if partnership {
                let mut page = k1_1065("Synthetic Partners");
                page.basis.start = k1_1065_continuing(50_000, history.clone());
                IncomeDocument::K1Partnership(page)
            } else {
                let mut page = k1_1120s("Synthetic Corp");
                page.basis.start = k1_1120s_continuing(50_000, 50_000);
                let K1ScorpStart::Continuing { risk_history, .. } = &mut page.basis.start else {
                    unreachable!()
                };
                *risk_history = history.clone();
                IncomeDocument::K1Scorp(page)
            };
            let mut session = Interview::new(TEST_USER, original.clone());
            let request = save_topic(&original, TaxSection::PassThrough, vec![document(0, doc)]);
            let P::SaveIncomes { allowed_continuing_risk, .. } = &request else { unreachable!() };
            assert_eq!(*allowed_continuing_risk, vec![1, 2]);
            accepted(&send(&mut session, request));
            let before = session.draft.clone();
            let mut reopened = open_topic(&mut session, TaxSection::PassThrough);
            let P::SaveIncomes { allowed_continuing_risk, documents, .. } = &mut reopened else {
                unreachable!()
            };
            assert_eq!(*allowed_continuing_risk, ScorpRiskStart::continuing_choices());
            let row = documents
                .documents
                .iter_mut()
                .find(|row| {
                    matches!(
                        row.document,
                        IncomeDocument::K1Scorp(_) | IncomeDocument::K1Partnership(_)
                    )
                })
                .unwrap();
            let restored = match &mut row.document {
                IncomeDocument::K1Scorp(page) => {
                    let K1ScorpStart::Continuing { risk_history, .. } = &mut page.basis.start
                    else {
                        unreachable!()
                    };
                    risk_history
                }
                IncomeDocument::K1Partnership(page) => {
                    let BasisStart::Continuing { risk_history, .. } = &mut page.basis.start else {
                        unreachable!()
                    };
                    risk_history
                }
                _ => unreachable!(),
            };
            assert_eq!(*restored, history);
            *restored = ScorpRiskStart::New;
            // An echoed catalog cannot authorize an invalid answer on the server.
            *allowed_continuing_risk = vec![0];
            rejected_with(
                send(&mut session, reopened),
                if partnership { "PARTNER_RISK_HISTORY_NEW" } else { "SCORP_RISK_HISTORY_NEW" },
            );
            assert_eq!(session.draft, before);
        }
    }
}

include!("../health/unit46_interview_registration.rs");
include!("../credits/unit16_interview_registration.rs");

#[test]
fn unit16_actual_fresh_entered_credit_and_history_rows_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_CREDIT");
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 11_610_000, 0);
    let profile = std::env::var("TAX_UNIT16_CREDIT_PROFILE").unwrap_or_default();
    let (entered_credit, farm, category) = match profile.as_str() {
        "work" | "work-farm" => (
            BusinessCredit::WorkOpportunity,
            profile.ends_with("-farm"),
            BusinessExpenseCategory::Wages,
        ),
        "pension" | "pension-farm" => (
            BusinessCredit::PensionStartup,
            profile.ends_with("-farm"),
            BusinessExpenseCategory::LegalAndProfessional,
        ),
        "auto" | "auto-farm" => (
            BusinessCredit::AutoEnrollment,
            profile.ends_with("-farm"),
            BusinessExpenseCategory::LegalAndProfessional,
        ),
        "military" | "military-farm" => (
            BusinessCredit::MilitarySpouse,
            profile.ends_with("-farm"),
            BusinessExpenseCategory::EmployeePension,
        ),
        "health" | "health-farm" => (
            BusinessCredit::SmallEmployerHealth,
            profile.ends_with("-farm"),
            BusinessExpenseCategory::EmployeeBenefits,
        ),
        "leave-premium" | "leave-premium-farm" => (
            BusinessCredit::PaidLeave,
            profile.ends_with("-farm"),
            BusinessExpenseCategory::EmployeeBenefits,
        ),
        "leave" | "leave-farm" => {
            (BusinessCredit::PaidLeave, profile.ends_with("-farm"), BusinessExpenseCategory::Wages)
        }
        "" => (BusinessCredit::DisabledAccess, false, BusinessExpenseCategory::Wages),
        _ => panic!("unknown completed employer source profile"),
    };
    let entered_amount = if profile.is_empty() { 500_000 } else { 50_000 };
    let mut activity = if farm {
        raised_farm(40_000, 0)
    } else {
        cash_business(if profile.is_empty() { 0 } else { 40_000 })
    };
    if !profile.is_empty() {
        let FederalIncome::Business { data } = &mut activity else { panic!() };
        data.payroll = BusinessPayroll::Yes {
            statements: vec![PayrollW2 {
                label: "Actual employer source W-2".into(),
                wages: 400_000,
                medicare: 400_000,
            }],
            attribution: PayrollQbiAttribution::EntireBusiness,
        };
        // Actual completed-form expenses, already reduced where the source
        // form requires it. Payroll remains its original gross4000; it is not
        // another deduction and there is no App-computed duplicate claim.
        data.expenses = vec![ordinary_business_expense(
            category.clone(),
            if entered_credit == BusinessCredit::AutoEnrollment { 400_000 } else { 350_000 },
        )];
        if category != BusinessExpenseCategory::Wages {
            data.expenses.push(ordinary_business_expense(BusinessExpenseCategory::Wages, 400_000));
        }
    }
    add(&mut d, income(activity));
    let baseline = supported(&d);
    let source = d.income.last().unwrap().id;
    d.credits.as_mut().unwrap().business_credits = Some(empty_business_credit_history());
    let mut session = Interview::new(TEST_USER, d);
    if client.required_calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::MoreCredits, "SEED");
    let mut answers = seed;
    let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
    let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
    history.credits.push(
        EnteredBusinessCredit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            credit: entered_credit,
            amount: entered_amount,
            source: if farm {
                CreditSource::Farm { farm: Some(source), reduced: true }
            } else {
                CreditSource::Business { business: Some(source), reduced: true }
            },
            election: CreditElection::None,
        }
        .into(),
    );
    history.carryforwards.push(
        BusinessCreditCarryforward {
            credit: BusinessCredit::PensionStartup,
            origin_year: 2024,
            amount: 100_000,
            passive: None,
            filer: CarryFiler::SameFilers,
            research_source: None,
        }
        .into(),
    );
    client.save(&mut session, answers);
    let r = supported(&session.draft);
    assert_eq!(super::value(&r, "general_business_credit"), entered_amount / 100 + 1000);
    assert_eq!(super::value(&r, "agi"), super::value(&baseline, "agi"));
    assert_eq!(super::value(&r, "taxable_income"), super::value(&baseline, "taxable_income"));
    assert_eq!(
        super::value(&r, &alloc::format!("form_3800_part_3_line_{}(e)", entered_credit.line(2026))),
        entered_amount / 100
    );
    let FederalIncome::Business { data } = &session.draft.income.last().unwrap().income else {
        panic!()
    };
    assert_eq!(data.payroll.qbi_wages(), if profile.is_empty() { 0 } else { 400_000 });
    assert!(
        data.work_opportunity_credit.is_none()
            && data.retirement_plan_credit.is_none()
            && data.small_employer_health_credit.is_none()
            && data.paid_leave_credit.is_none()
    );
    if !profile.is_empty() {
        std::eprintln!(
            "completed employer source profile={profile}, AGI={}, TI={}, entered={}, total_credit={}, gross_payroll={}",
            super::value(&r, "agi"),
            super::value(&r, "taxable_income"),
            entered_amount / 100,
            super::value(&r, "general_business_credit"),
            data.payroll.qbi_wages() / 100
        );
    }
    client.restart(&mut session);
    let reopened = client.editor(&mut session, TaxSection::MoreCredits, "REOPENED");
    let P::SaveMoreCredits { value, .. } = reopened else { panic!() };
    let MoreCredit::BusinessCredits(history) = &value.credits[0] else { panic!() };
    assert_eq!(history.credits.len(), 1);
    assert_eq!(history.credits[0].credit, entered_credit);
    assert_eq!(history.credits[0].amount, entered_amount);
    assert_eq!(history.carryforwards.len(), 1);
    assert_eq!(history.credits[0].allowed_elections, vec![0]);
    assert!(history.carryforwards[0].allowed_research_choices.is_empty());
    assert_eq!(history.carryforwards[0].research_source, None);
}

#[test]
fn unit16_actual_fresh_election_and_transfer_row_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_ELECTION");
    let profile = std::env::var("TAX_UNIT16_ELECTION_PROFILE").unwrap_or_else(|_| "full".into());
    let (wages, amount, election, bought, used, payment, carry) = match profile.as_str() {
        "full" => (
            21_610_000,
            5_000_000,
            CreditElection::ElectivePayment { amount: 5_000_000 },
            false,
            7838,
            42162,
            0,
        ),
        "partial" => (
            21_610_000,
            5_000_000,
            CreditElection::ElectivePayment { amount: 3_000_000 },
            false,
            7838,
            30000,
            12162,
        ),
        "sold" => {
            (11_610_000, 500_000, CreditElection::Sold { amount: 150_000 }, false, 3500, 0, 0)
        }
        "bought" => (11_610_000, 500_000, CreditElection::None, true, 0, 0, 5000),
        _ => panic!("unknown election/transfer profile"),
    };
    // The original F5 and transfer map controls, entered through their actual
    // source routes. Transfer cash is neither income nor a deduction.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
    add(&mut d, income(cash_business(0)));
    let source = d.income.last().unwrap().id;
    let baseline = supported(&d);
    d.credits.as_mut().unwrap().business_credits = Some(empty_business_credit_history());
    let mut session = Interview::new(TEST_USER, d);
    if client.calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::MoreCredits, "SEED");
    let mut answers = seed;
    let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
    let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
    let entry = EnteredBusinessCredit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        credit: BusinessCredit::AdvancedManufacturingProduction,
        amount,
        source: if bought {
            CreditSource::Bought
        } else {
            CreditSource::Business { business: Some(source), reduced: true }
        },
        election: election.clone(),
    };
    history.credits.push(entry.clone().into());
    client.save(&mut session, answers);
    assert_eq!(
        session.draft.credits.as_ref().unwrap().business_credits.as_ref().unwrap().credits,
        vec![entry]
    );
    let r = supported(&session.draft);
    assert_eq!(super::value(&r, "agi"), super::value(&baseline, "agi"));
    assert_eq!(super::value(&r, "taxable_income"), super::value(&baseline, "taxable_income"));
    assert_eq!(super::value(&r, "general_business_credit"), used);
    assert_eq!(
        if r.lines.iter().any(|v| v.id == "schedule_3_elective_payment") {
            super::value(&r, "schedule_3_elective_payment")
        } else {
            0
        },
        payment
    );
    for (column, expected) in [
        ('d', 0),
        ('e', if bought { 0 } else { amount / 100 }),
        (
            'f',
            if bought {
                amount / 100
            } else if profile == "sold" {
                -1500
            } else {
                0
            },
        ),
        (
            'g',
            if bought {
                0
            } else if profile == "sold" {
                3500
            } else {
                amount / 100
            },
        ),
        (
            'h',
            if profile == "full" {
                50000
            } else if profile == "partial" {
                30000
            } else {
                0
            },
        ),
        ('i', used),
        ('j', payment),
    ] {
        assert_eq!(
            super::value(&r, &alloc::format!("form_3800_part_3_line_1b({column})")),
            expected,
            "{profile}/{column}"
        );
    }
    assert_eq!(
        super::value(
            &r,
            if bought {
                "business_credit_advanced_manufacturing_production_current_year_passive_to_next_year"
            } else {
                "business_credit_advanced_manufacturing_production_current_year_to_next_year"
            }
        ),
        carry
    );
    let saved = client.restart(&mut session);
    let reopened = client.editor(&mut session, TaxSection::MoreCredits, "REOPENED");
    let P::SaveMoreCredits { value, .. } = &reopened else { panic!() };
    let MoreCredit::BusinessCredits(history) = &value.credits[0] else { panic!() };
    assert_eq!(history.credits.len(), 1);
    assert_eq!(history.credits[0].election, election);
    assert_eq!(history.credits[0].allowed_elections, vec![0, 1, 2]);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    std::eprintln!(
        "actual election/transfer {profile}: AGI={}, TI={}, used={used}, payment={payment}, carry={carry}",
        super::value(&r, "agi"),
        super::value(&r, "taxable_income")
    );
}

#[test]
fn unit16_actual_fresh_research_history_source_row_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_RESEARCH");
    let profile = std::env::var("TAX_UNIT16_RESEARCH_PROFILE").unwrap_or_else(|_| "forward".into());
    assert!(matches!(profile.as_str(), "forward" | "none" | "carryback"));
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_050_000, 0);
    let mut k1 = complete_ordinary_scorp(19500, true);
    for line in [BusinessCredit::Research, BusinessCredit::ResearchSmallBusiness] {
        k1.credits.push(K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            code: K1CreditCode::Research,
            line: Some(line),
            amount: 500_000,
        });
    }
    add(&mut d, income(FederalIncome::Scorp { data: k1 }));
    let entity = d.income.last().unwrap().id;
    d.credits.as_mut().unwrap().business_credits = Some(empty_business_credit_history());
    let mut session = Interview::new(TEST_USER, d);
    if client.calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::MoreCredits, "SEED");
    let mut answers = seed;
    let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
    let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
    if profile == "carryback" {
        history.carrybacks.push(
            BusinessCreditCarryback {
                credit: BusinessCredit::Research,
                year: 2025,
                amount: 50_000,
                research_source: Some(entity),
            }
            .into(),
        );
    } else {
        history.carryforwards.push(
            BusinessCreditCarryforward {
                credit: BusinessCredit::Research,
                origin_year: 2024,
                amount: 100_000,
                passive: None,
                filer: CarryFiler::SameFilers,
                research_source: (profile != "none").then_some(entity),
            }
            .into(),
        );
    }
    client.save(&mut session, answers);
    let r = supported(&session.draft);
    assert_eq!(super::value(&r, "agi"), 120000);
    assert_eq!(super::value(&r, "taxable_income"), 100000);
    assert_eq!(super::value(&r, "general_business_credit"), 2246);
    if profile == "carryback" {
        assert_eq!(
            r.lines
                .iter()
                .filter(|v| v.id.ends_with("carryback_2025"))
                .map(|v| super::value(&r, &v.id))
                .sum::<i64>(),
            500
        );
    } else {
        assert_eq!(
            super::value(&r, "form_3800_part_4_line_1c(g)"),
            if profile == "none" { 0 } else { 1000 }
        );
        assert_eq!(
            super::value(&r, "form_3800_part_4_line_1c(i)"),
            if profile == "none" { 1000 } else { 0 }
        );
    }
    let saved = client.restart(&mut session);
    assert_eq!(supported(&session.draft), r);
    let reopened = client.editor(&mut session, TaxSection::MoreCredits, "REOPENED");
    let P::SaveMoreCredits { value, .. } = &reopened else { panic!() };
    let MoreCredit::BusinessCredits(history) = &value.credits[0] else { panic!() };
    if profile == "carryback" {
        assert_eq!(
            history.carrybacks[0].research_source,
            Some(crate::business_credit_page::ResearchCreditBusiness::Business { source: entity })
        );
        assert_eq!(history.carrybacks[0].allowed_research_choices, vec![0]);
    } else {
        assert_eq!(
            history.carryforwards[0].research_source,
            (profile != "none").then_some(
                crate::business_credit_page::ResearchCreditBusiness::Business { source: entity }
            )
        );
        assert_eq!(history.carryforwards[0].allowed_research_choices, vec![0]);
    }
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    std::eprintln!(
        "actual research history {profile}: original AGI120000/TI100000/shared cap2246 preserved"
    );
}

#[test]
fn unit16_actual_fresh_retained_interest_origin_row_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_INTEREST");
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    add(&mut d, income(savings_income(10000, 0)));
    let mut corporation = complete_ordinary_scorp(0, true);
    corporation.interest = 2_000_000;
    add(&mut d, income(FederalIncome::Scorp { data: corporation }));
    let entity = d.income.last().unwrap().id;
    itemized(d.deductions.as_mut().unwrap()).real_estate_taxes = 1_000_000;
    // Hold the approved witness's itemized route fixed. Automatic otherwise
    // correctly compares the new research limit and can prefer standard.
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized { facts };
    d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
        credits: vec![],
        carrybacks: vec![],
        spouse_without_credits: false,
        carryforwards: vec![BusinessCreditCarryforward {
            credit: BusinessCredit::Research,
            origin_year: 2024,
            amount: 1_000_000,
            passive: None,
            filer: CarryFiler::SameFilers,
            research_source: Some(entity),
        }],
    });
    d.deductions.as_mut().unwrap().investment_interest = Some(InvestmentBorrowing {
        loans: vec![native_investment_loan("Existing portfolio source".into(), 0, 0, 0)],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
    let mut session = Interview::new(TEST_USER, d);
    if client.calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::OtherItemized, "SEED");
    let mut answers = seed;
    let P::SaveOtherItemized { value, .. } = &mut answers else { panic!() };
    value.investment_interest.as_mut().unwrap().loans.push(InvestmentLoanPage {
        paid_owners: None,
        carry_owners: None,
        label: "Prior actual entity interest".into(),
        source: 0,
        paid: 0,
        carry: 1_000_000,
        amt_carry: 1_000_000,
        private_activity: 0,
        allowed_origin_choices: vec![0],
        originating_k1: Some(crate::InvestmentK1Origin::Entity { source: entity }),
    });
    client.save(&mut session, answers);
    let r = supported(&session.draft);
    assert_eq!(super::value(&r, "investment_interest_deduction"), 10000);
    assert_eq!(super::value(&r, "investment_interest_carryforward"), 0);
    let selected =
        std::env::var("TAX_UNIT16_INTEREST_EXPECT_ORIGIN").ok().as_deref() != Some("none");
    assert_eq!(super::value(&r, "general_business_credit"), if selected { 1583 } else { 3166 });
    let borrowing =
        session.draft.deductions.as_ref().unwrap().investment_interest.as_ref().unwrap();
    assert_eq!(borrowing.loans[1].source, 0);
    assert_eq!(borrowing.loans[1].originating_k1, selected.then_some(entity));
    let saved = client.restart(&mut session);
    let reopened = client.editor(&mut session, TaxSection::OtherItemized, "REOPENED");
    let mut forged = reopened.clone();
    let P::SaveOtherItemized { value, .. } = &mut forged else { panic!() };
    value.investment_interest.as_mut().unwrap().loans[1].originating_k1 =
        Some(crate::InvestmentK1Origin::Entity { source: 999999 });
    rejected_with(
        current_source_refusal(
            "secondary-admission-3374",
            &mut session,
            forged,
            "INVESTMENT_BORROWING_RECORDS",
            Some("[value/investment_interest/loans/1/originating_k1/source]"),
            SourceScope::Secondary(SecondaryCorrection::InvestmentOrigin),
        ),
        "INVESTMENT_BORROWING_RECORDS",
    );
    assert_eq!(session.draft, saved);
    accepted(&send(&mut session, save_topic(&saved, TaxSection::PassThrough, vec![])));
    let borrowing =
        session.draft.deductions.as_ref().unwrap().investment_interest.as_ref().unwrap();
    assert_eq!(borrowing.loans[1].source, 0);
    assert_eq!(borrowing.loans[1].carry, 1_000_000);
    assert_eq!(borrowing.loans[1].originating_k1, None);
    assert_eq!(
        session.draft.credits.as_ref().unwrap().business_credits.as_ref().unwrap().carryforwards[0]
            .research_source,
        None
    );
}

#[test]
fn unit16_actual_fresh_k1_property_and_statement_rows_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_K1");
    let profile = std::env::var("TAX_UNIT16_K1_PROFILE").unwrap_or_else(|_| "own".into());
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 31_610_000, 0);
    let mut page = k1_1065("Synthetic actual Form3468 partnership");
    page.interest =
        PartnerInterest::General { participation: Participation::Over500Hours { hours: 600 } };
    page.basis.contributions = 10_000_000;
    let (mut session, reply) = save_1065s(&d, vec![page]);
    accepted(&reply);
    if client.calculation(&mut session) {
        return;
    }
    let seed = open_topic(&mut session, TaxSection::PassThrough);
    client.write("SEED", &seed);
    let mut answers = seed;
    let P::SaveIncomes { documents, .. } = &mut answers else { panic!() };
    let IncomeDocument::K1Partnership(page) = &mut documents.documents[0].document else {
        panic!()
    };
    page.more.credits.push(crate::K1CreditPage::from(K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code: K1CreditCode::Other,
        line: Some(BusinessCredit::DifferentialWage),
        amount: 100_000,
    }));
    page.more.investment_property.push(K1InvestmentProperty::Energy {
        id: 0,
        basis: 1_000_000,
        rate: EnergyRate {
            increased_rate: false,
            domestic_content: false,
            energy_community: false,
        },
        own_form: if matches!(profile.as_str(), "automatic" | "no") {
            None
        } else {
            Some(OwnInvestmentCredit { amount: 200_000, reduced: profile != "unreduced" })
        },
    });
    page.more.allowed_investment_confirmations =
        crate::k1_page::investment_choices(&page.more.investment_property);
    page.more.investment_confirmed = if matches!(profile.as_str(), "automatic" | "no") {
        Some(if profile == "no" {
            K1InvestmentConfirmation::No
        } else {
            K1InvestmentConfirmation::Yes
        })
    } else {
        None
    };
    client.save(&mut session, answers);
    let result = supported(&session.draft);
    assert_eq!(
        super::value(&result, "general_business_credit"),
        match profile.as_str() {
            "automatic" => 1600,
            "unreduced" | "no" => 1000,
            _ => 3000,
        }
    );
    let expected_reduction = match profile.as_str() {
        "automatic" | "no" => 30_000,
        "unreduced" => 0,
        _ => 100_000,
    };
    let FederalIncome::Partnership { data } = &session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .income
    else {
        panic!()
    };
    assert_eq!(
        crate::investment_credit::k1_basis_decrease(&data.investment_property),
        expected_reduction
    );
    assert_eq!(data.credits[0].code, K1CreditCode::Other);
    assert_eq!(data.credits[0].line, Some(BusinessCredit::DifferentialWage));
    if profile == "unreduced" {
        assert!(noted(&result, "WARN_ENTERED_BUSINESS_CREDIT_LEFT_OUT"));
    }
    let saved = client.restart(&mut session);
    assert_eq!(supported(&session.draft), result);
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    client.write("REOPENED", &reopened);
    let P::SaveIncomes { documents, .. } = &reopened else { panic!() };
    let IncomeDocument::K1Partnership(page) = &documents.documents[0].document else { panic!() };
    assert_eq!(
        page.more.allowed_investment_confirmations,
        if matches!(profile.as_str(), "automatic" | "no") { vec![0, 1] } else { vec![] }
    );
    assert_eq!(page.more.credits.len(), 1);
    assert_eq!(page.more.investment_property.len(), 1);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
}

#[test]
fn unit16_actual_fresh_retained_nol_row_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_NOL");
    let (d, entity) = crate::tests::credits::form_3800::retained_research_draft();
    let mut session = Interview::new(TEST_USER, d);
    if client.calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::SelfEmployment, "SEED");
    let mut answers = seed;
    let P::SaveIncomes { documents, .. } = &mut answers else { panic!() };
    documents.documents.push(document(
        0,
        IncomeDocument::NetOperatingLoss(LossCarryforward {
            regular: vec![NolYearPage {
                ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                injured_owners: None,
                year: 2024,
                balance: 1_000_000,
                allowed_research_parts: vec![0],
                research_parts: Some(ResearchLossPartsPage::Parts {
                    sources: vec![ResearchLossPartPage { source: entity, amount: 1_000_000 }],
                }),
            }],
            amt: vec![],
            source: LossSource::Business,
        }),
    ));
    client.save(&mut session, answers);
    let selected = std::env::var("TAX_UNIT16_NOL_EXPECT_ORIGIN").ok().as_deref() != Some("none");
    let r = supported(&session.draft);
    assert_eq!(super::value(&r, "general_business_credit"), if selected { 3673 } else { 5510 });
    assert_eq!(super::value(&r, "prior_nol_deduction"), 10000);
    assert_eq!(super::value(&r, "agi"), 200000);
    assert_eq!(super::value(&r, "taxable_income"), 183900);
    let saved = client.restart(&mut session);
    let reopened = client.editor(&mut session, TaxSection::SelfEmployment, "REOPENED");
    let P::SaveIncomes { documents, .. } = &reopened else { panic!() };
    let IncomeDocument::NetOperatingLoss(loss) = &documents.documents.last().unwrap().document
    else {
        panic!()
    };
    assert_eq!(loss.regular[0].allowed_research_parts, vec![0]);
    assert_eq!(loss.regular[0].research_parts.is_some(), selected);
    let mut forged = reopened.clone();
    let P::SaveIncomes { documents, .. } = &mut forged else { panic!() };
    let IncomeDocument::NetOperatingLoss(loss) =
        &mut documents.documents.last_mut().unwrap().document
    else {
        panic!()
    };
    loss.regular[0].research_parts = Some(ResearchLossPartsPage::Parts {
        sources: vec![ResearchLossPartPage { source: 999999, amount: 1_000_000 }],
    });
    rejected_with(
        current_income_refusal(
            "income-source-3509",
            &mut session,
            forged,
            "RETAINED_RESEARCH_SOURCE_UNAVAILABLE",
            "[documents/documents/0/document/regular/0/research_parts/sources/0/source]",
        ),
        "RETAINED_RESEARCH_SOURCE_UNAVAILABLE",
    );
    assert_eq!(session.draft, saved);
    accepted(&send(&mut session, save_topic(&saved, TaxSection::PassThrough, vec![])));
    let NolHistory::Records { years, .. } = &session.draft.screening.as_ref().unwrap().nol else {
        panic!()
    };
    assert_eq!(years[0].balance, 1_000_000);
    assert_eq!(years[0].research_parts, None);
}

#[test]
fn unit16_actual_fresh_rental_basis_row_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_RENTAL");
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d.credits.as_mut().unwrap().business_credits = Some(empty_business_credit_history());
    let mut session = Interview::new(TEST_USER, d);
    let seed = open_topic(&mut session, TaxSection::Rentals);
    let credit_stage = std::env::var("TAX_UNIT16_RENTAL_CREDIT_STAGE").is_ok();
    if std::env::var("TAX_UNIT16_RENTAL_CALC_INPUT").is_ok() && !credit_stage {
        assert!(client.required_calculation(&mut session));
        return;
    }
    client.write("SEED", &seed);
    let mut page = rental_page(2_400_000);
    page.address = "Synthetic own-form rental".into();
    page.building.cost = 2_000_000;
    page.building.land = 0;
    page.building.in_service = 20260115;
    page.building.credit_reduction = 400_000;
    let mut answers = seed;
    let P::SaveIncomes { documents, .. } = &mut answers else { unreachable!() };
    documents.documents = vec![document(0, IncomeDocument::Rental(page.clone()))];
    client.save(&mut session, answers);
    let saved = session.draft.clone();
    assert_eq!(value(&supported(&saved), "agi"), 123_442);
    // 2026-10-19, D-2026-10-19-01: $16,000×23/660=$557.5758;
    // exact source cents round once to55,758c, then dollars on the form.
    assert_eq!(line_amount(&supported(&saved), "rental_2_depreciation"), 55_758);
    assert_eq!(displayed_line_amount(&supported(&saved), "rental_2_depreciation"), 55_800);
    let source = saved.income.last().unwrap().id;
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    client.write("REOPENED", &reopened);
    let IncomeDocument::Rental(reopened_page) = &page_documents(&reopened)[0].document else {
        unreachable!()
    };
    assert_eq!(reopened_page, &page);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    if std::env::var("TAX_UNIT16_RENTAL_CALC_INPUT").is_ok() {
        assert!(client.required_calculation(&mut session));
        return;
    }
    let saved = session.draft.clone();
    let Reply::Edit(seed) = send(&mut session, navigate(&saved, TaxSection::MoreCredits)) else {
        panic!("more credits editor")
    };
    client.write("CREDIT_SEED", &seed);
    let mut answers = seed;
    let P::SaveMoreCredits { value: credits_page, .. } = &mut answers else { unreachable!() };
    let MoreCredit::BusinessCredits(history) = &mut credits_page.credits[0] else { unreachable!() };
    history.credits.push(
        EnteredBusinessCredit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            credit: BusinessCredit::Rehabilitation,
            amount: 80_000,
            source: CreditSource::Rental { rental: Some(source), reduced: true },
            election: CreditElection::None,
        }
        .into(),
    );
    NativeInterviewCase("TAX_UNIT16_RENTAL_CREDIT").save(&mut session, answers);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "general_business_credit"), 800);
    assert_eq!(line_amount(&result, "rental_2_depreciation"), 55_758);
    assert_eq!(displayed_line_amount(&result, "rental_2_depreciation"), 55_800);
    assert_eq!(value(&result, "agi"), 123_442);
    let saved = client.restart(&mut session);
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    let IncomeDocument::Rental(page) = &page_documents(&reopened)[0].document else {
        unreachable!()
    };
    assert_eq!(page.building.cost, 2_000_000);
    assert_eq!(page.building.credit_reduction, 400_000);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
}

#[test]
fn unit16_actual_fresh_retained_capital_row_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_CAPITAL");
    let (mut d, source) = crate::tests::credits::form_3800::retained_capital_draft();
    let mut record = d.screening.as_mut().unwrap().capital_history.remove(0);
    let none = std::env::var("TAX_UNIT16_CAPITAL_EXPECT_ORIGIN").is_ok_and(|v| v == "none");
    record.research_parts = (!none).then(|| ResearchCapitalParts::Parts {
        sources: vec![ResearchCapitalPart { source, short: 1_000_000, long: 0 }],
    });
    let mut session = Interview::new(TEST_USER, d);
    let seed = open_topic(&mut session, TaxSection::InvestmentSales);
    if client.required_calculation(&mut session) {
        return;
    }
    client.write("SEED", &seed);
    let mut page = PriorCapitalReturnPage::from(record.clone());
    page.allowed_research_parts = vec![0];
    let mut answers = seed;
    let P::SaveIncomes { documents, .. } = &mut answers else { unreachable!() };
    documents.documents = vec![document(0, IncomeDocument::CapitalLossCarryover(page))];
    client.save(&mut session, answers);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "general_business_credit"), if none { 5567 } else { 5011 });
    assert_eq!(value(&result, "agi"), 207_000);
    assert_eq!(value(&result, "taxable_income"), 190_900);
    assert_eq!(value(&result, "capital_loss_carryforward"), 7000);
    assert_eq!(session.draft.screening.as_ref().unwrap().capital_history[0], record);
    let saved = client.restart(&mut session);
    let reopened = open_topic(&mut session, TaxSection::InvestmentSales);
    client.write("REOPENED", &reopened);
    let IncomeDocument::CapitalLossCarryover(page) = &page_documents(&reopened)[0].document else {
        unreachable!()
    };
    assert_eq!(page.allowed_research_parts, vec![0]);
    assert_eq!(page.research_parts.is_none(), none);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    if !none {
        let Some(ResearchCapitalParts::Parts { sources }) =
            &mut session.draft.screening.as_mut().unwrap().capital_history[0].research_parts
        else {
            unreachable!()
        };
        sources[0].source = 987_654_321;
        assert!(!session.draft.valid_stored());
    }
}

#[test]
fn unit16_actual_fresh_retained_charity_row_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_CHARITY");
    let profile = std::env::var("TAX_UNIT16_CHARITY_PROFILE").unwrap_or_else(|_| "actual".into());
    let (mut d, source) = crate::tests::credits::form_3800::retained_charity_draft();
    let basis = profile.starts_with("basis-");
    if basis {
        d.deductions.as_mut().unwrap().basis_election = true;
    }
    let mut record = d.deductions.as_mut().unwrap().charity_carryovers.remove(0);
    if basis {
        record.class = CharityClass::CapitalGain30;
        record.appreciation = Some(400_000);
    }
    record.research_parts = match profile.as_str() {
        "personal" => None,
        _ => Some(ResearchCharityParts::Parts {
            sources: vec![ResearchCharityPart {
                origin: if profile == "other" {
                    ResearchCharityOrigin::OtherEntity(OtherCharityEntity {
                        name: "Synthetic retained donor entity".into(),
                        ein: "12-3456789".into(),
                    })
                } else {
                    ResearchCharityOrigin::CurrentK1 { source }
                },
                amount: if basis { 500_000 } else { 1_000_000 },
                original_appreciation: if basis {
                    Some(match profile.as_str() {
                        "basis-a" => 300_000,
                        "basis-b" => 100_000,
                        "basis-zero" => 0,
                        _ => panic!("unknown basis profile"),
                    })
                } else {
                    None
                },
            }],
        }),
    };
    let mut session = Interview::new(TEST_USER, d);
    let saved = session.draft.clone();
    let Reply::Edit(seed) = send(&mut session, navigate(&saved, TaxSection::GiftCarryovers)) else {
        panic!("gift carryover editor")
    };
    if client.required_calculation(&mut session) {
        return;
    }
    client.write("SEED", &seed);
    let mut page = CharityCarryoverPage::from(record.clone());
    page.allowed_research_parts = vec![0];
    page.allowed_source_appreciation = if basis { vec![0] } else { vec![] };
    let mut answers = seed;
    let P::SaveGiftCarryovers { value: gifts, .. } = &mut answers else { unreachable!() };
    gifts.carryovers = vec![page];
    client.save(&mut session, answers);
    let result = supported(&session.draft);
    assert_eq!(
        value(&result, "general_business_credit"),
        match profile.as_str() {
            "personal" => 5493,
            "other" => 5737,
            "basis-a" => 5362,
            "basis-b" => 5091,
            "basis-zero" => 4953,
            _ => 4026,
        }
    );
    assert_eq!(value(&result, "agi"), 210_000);
    assert_eq!(value(&result, "taxable_income"), if basis { 195_050 } else { 191_050 });
    assert_eq!(
        value(
            &result,
            if basis {
                "charity_carry_2024_capital_gain_thirty_deduction"
            } else {
                "charity_carry_2024_cash_sixty_deduction"
            }
        ),
        if basis { 4950 } else { 8950 }
    );
    assert_eq!(session.draft.deductions.as_ref().unwrap().charity_carryovers[0], record);
    let saved = client.restart(&mut session);
    let reopened = client.editor(&mut session, TaxSection::GiftCarryovers, "REOPENED");
    let P::SaveGiftCarryovers { value: gifts, .. } = &reopened else { unreachable!() };
    assert_eq!(gifts.carryovers[0].allowed_research_parts, vec![0]);
    assert_eq!(crate::CharityCarryover::from(gifts.carryovers[0].clone()), record);
    accepted(&send(&mut session, reopened.clone()));
    assert_eq!(session.draft, saved);
    if profile != "personal" {
        let mut invalid = reopened.clone();
        let P::SaveGiftCarryovers { value: gifts, .. } = &mut invalid else { unreachable!() };
        let Some(ResearchCharityPartsPage::Parts { sources }) =
            &mut gifts.carryovers[0].research_parts
        else {
            unreachable!()
        };
        sources[0].amount = record.balance + 100;
        rejected_with(send(&mut session, invalid), "RETAINED_RESEARCH_PARTS_ABOVE_BALANCE");
        assert_eq!(session.draft, saved);
    }
    if basis {
        let mut invalid = reopened;
        let P::SaveGiftCarryovers { value: gifts, .. } = &mut invalid else { unreachable!() };
        let Some(ResearchCharityPartsPage::Parts { sources }) =
            &mut gifts.carryovers[0].research_parts
        else {
            unreachable!()
        };
        sources[0].original_appreciation =
            Some(crate::CharitySourceAppreciation::Amount { amount: 400_001 });
        rejected_with(
            send(&mut session, invalid),
            "RETAINED_CHARITY_SOURCE_APPRECIATION_ABOVE_TOTAL",
        );
        assert_eq!(session.draft, saved);
    }
    let mut removed = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut removed else { unreachable!() };
    documents.documents.clear();
    accepted(&send(&mut session, removed));
    let carry = &session.draft.deductions.as_ref().unwrap().charity_carryovers[0];
    assert_eq!(carry.balance, 1_000_000);
    assert!(carry.research_parts.is_none());
    assert!(session.draft.valid_stored());
}

#[test]
fn unit16_actual_fresh_k1_credit_import_lines_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_IMPORT");
    let profile =
        std::env::var("TAX_UNIT16_IMPORT_PROFILE").unwrap_or_else(|_| "partnership".into());
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 31_610_000, 0);
    let document_value = if profile == "scorp" {
        let mut page = k1_1120s("Actual S corporation credit statement");
        page.interest = 1_000_000;
        page.participation = Participation::Over500Hours { hours: 600 };
        page.basis.additions = 10_000_000;
        IncomeDocument::K1Scorp(page)
    } else {
        assert_eq!(profile, "partnership");
        let mut page = k1_1065("Actual partnership credit statement");
        page.more.interest = 1_000_000;
        page.interest =
            PartnerInterest::General { participation: Participation::Over500Hours { hours: 600 } };
        page.basis.contributions = 10_000_000;
        IncomeDocument::K1Partnership(page)
    };
    let mut session = Interview::new(TEST_USER, d.clone());
    accepted(&send(
        &mut session,
        save_topic(&d, TaxSection::PassThrough, vec![document(0, document_value)]),
    ));
    if client.calculation(&mut session) {
        return;
    }
    let seed = open_topic(&mut session, TaxSection::PassThrough);
    client.write("SEED", &seed);
    let expected: Vec<K1Credit> = [
        (K1CreditCode::OtherRentalRealEstate, BusinessCredit::DisabledAccess),
        (K1CreditCode::OtherRental, BusinessCredit::DifferentialWage),
        (K1CreditCode::RenewableElectricity, BusinessCredit::RenewableElectricitySpecified),
        (K1CreditCode::Transferred, BusinessCredit::Energy),
        (K1CreditCode::Other, BusinessCredit::EmployerChildcare),
        (K1CreditCode::Research, BusinessCredit::ResearchSmallBusiness),
    ]
    .into_iter()
    .map(|(code, line)| K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        code,
        line: Some(line),
        amount: 10_000,
    })
    .collect();
    let mut answers = seed;
    let P::SaveIncomes { documents, .. } = &mut answers else { panic!() };
    let imported = expected.iter().cloned().map(crate::K1CreditPage::from).collect();
    match &mut documents.documents[0].document {
        IncomeDocument::K1Partnership(page) => page.more.credits = imported,
        IncomeDocument::K1Scorp(page) => page.more.credits = imported,
        _ => panic!(),
    }
    client.save(&mut session, answers);
    let result = supported(&session.draft);
    assert_eq!(super::value(&result, "agi"), 326_100);
    assert_eq!(super::value(&result, "taxable_income"), 310_000);
    // F/G and transferred BC remain passive with no passive income. The
    // statement's AB/ZZ and the separately answered M each allow100.
    assert_eq!(super::value(&result, "general_business_credit"), 300);
    let credits = match &session.draft.income.last().unwrap().income {
        FederalIncome::Partnership { data } => &data.credits,
        FederalIncome::Scorp { data } => &data.credits,
        _ => panic!(),
    };
    assert_eq!(credits, &expected);
    let saved = client.restart(&mut session);
    assert_eq!(supported(&session.draft), result);
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    client.write("REOPENED", &reopened);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
}

#[test]
fn unit16_actual_fresh_clean_vehicle_prior_magi_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_VEHICLE");
    let profile = std::env::var("TAX_UNIT16_VEHICLE_PROFILE").unwrap_or_else(|_| "prior".into());
    let (current, prior, credit, expected) = match profile.as_str() {
        "current" => (150000, None, BusinessCredit::CleanVehicle, 7500),
        "over" => (150001, None, BusinessCredit::CleanVehicle, 0),
        "prior" => (150001, Some(150000), BusinessCredit::CleanVehicle, 7500),
        "prior-over" => (150001, Some(150001), BusinessCredit::CleanVehicle, 0),
        "commercial" => (150001, Some(150001), BusinessCredit::CommercialCleanVehicle, 7500),
        _ => panic!("unknown original R19 profile"),
    };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, current * 100, 0);
    add(&mut d, income(cash_business(0)));
    let source = d.income.last().unwrap().id;
    let baseline = supported(&d);
    d.credits.as_mut().unwrap().business_credits = Some(empty_business_credit_history());
    let mut session = Interview::new(TEST_USER, d);
    if client.calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::MoreCredits, "SEED");
    let mut answers = seed;
    let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
    let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
    history.credits.push(
        EnteredBusinessCredit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            credit,
            amount: 750000,
            source: CreditSource::Business { business: Some(source), reduced: true },
            election: CreditElection::None,
        }
        .into(),
    );
    value.credits.push(MoreCredit::CleanVehiclesAndRefueling {
        clean_vehicles: Some(
            FederalCleanVehicleClaim {
                prior_year: prior.map(|magi| FederalCleanVehiclePrior {
                    status: FederalStatus::Single,
                    magi: magi * 100,
                }),
                current_territory_addbacks: 0,
                // This is the existing shared income-limit owner, not a second
                // personal vehicle claim or a fabricated vehicle row.
                vehicles: vec![],
            }
            .into(),
        ),
        personal_refueling: vec![],
    });
    client.save(&mut session, answers);
    let r = supported(&session.draft);
    assert_eq!(
        r.lines
            .iter()
            .find(|row| row.id == "general_business_credit")
            .map_or(0, |row| crate::kernel::ratio::dollars(row.amount)),
        expected
    );
    assert_eq!(super::value(&r, "agi"), super::value(&baseline, "agi"));
    assert_eq!(super::value(&r, "taxable_income"), super::value(&baseline, "taxable_income"));
    let claim = session.draft.credits.as_ref().unwrap().clean_vehicles.as_ref().unwrap();
    assert!(claim.vehicles.is_empty());
    assert_eq!(claim.prior_year.as_ref().map(|p| p.magi), prior.map(|magi| magi * 100));
    assert_eq!(claim.current_territory_addbacks, 0);
    assert!(session.draft.credits.as_ref().unwrap().personal_refueling.is_empty());
    let saved = client.restart(&mut session);
    assert_eq!(supported(&session.draft), r);
    let reopened = client.editor(&mut session, TaxSection::MoreCredits, "REOPENED");
    let P::SaveMoreCredits { value, .. } = &reopened else { panic!() };
    assert!(value.credits.iter().any(|row| matches!(row, MoreCredit::CleanVehiclesAndRefueling { clean_vehicles: Some(v), personal_refueling } if v.vehicles.is_empty() && personal_refueling.is_empty() && v.prior_year.as_ref().map(|p| p.magi) == prior.map(|magi| magi * 100))));
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    std::eprintln!(
        "actual original R19 {profile}: currentMAGI={current}, priorMAGI={prior:?}, GBC={expected}; no duplicate personal vehicle claim"
    );
}

#[test]
fn unit16_actual_fresh_shared_line_cap_and_section_b_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_CAP");
    let profile = std::env::var("TAX_UNIT16_CAP_PROFILE").unwrap_or_else(|_| "childcare".into());
    let carryback = profile == "carryback";
    let section_b = match profile.as_str() {
        "childcare" | "carryback" => false,
        "section-b" => true,
        _ => panic!("unknown shared-cap source profile"),
    };
    let mut d = complete(if carryback { FederalStatus::Joint } else { FederalStatus::Single });
    d.income[0] = wage(
        1,
        Owner::Taxpayer,
        if carryback {
            5_716_000
        } else if section_b {
            11_610_000
        } else {
            101_610_000
        },
        0,
    );
    add(&mut d, income(cash_business(0)));
    let source = d.income.last().unwrap().id;
    if !carryback {
        let mut k1 = ordinary_partner_source(0, true);
        k1.credits.push(K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            code: if section_b {
                K1CreditCode::EmpowermentZone
            } else {
                K1CreditCode::EmployerChildcare
            },
            line: None,
            amount: if section_b { 900_000 } else { 40_000_000 },
        });
        add(&mut d, income(FederalIncome::Partnership { data: k1 }));
    }
    let baseline = supported(&d);
    d.credits.as_mut().unwrap().business_credits = Some(empty_business_credit_history());
    let mut session = Interview::new(TEST_USER, d);
    if client.calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::MoreCredits, "SEED");
    let mut answers = seed;
    let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
    let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
    history.credits.push(
        EnteredBusinessCredit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            credit: if section_b || carryback {
                BusinessCredit::DifferentialWage
            } else {
                BusinessCredit::EmployerChildcare
            },
            amount: if carryback {
                600_000
            } else if section_b {
                2_000_000
            } else {
                30_000_000
            },
            source: CreditSource::Business { business: Some(source), reduced: true },
            election: CreditElection::None,
        }
        .into(),
    );
    if section_b {
        history.carryforwards.push(
            BusinessCreditCarryforward {
                credit: BusinessCredit::EmpowermentZone,
                origin_year: 2024,
                amount: 300_000,
                passive: None,
                filer: CarryFiler::SameFilers,
                research_source: None,
            }
            .into(),
        );
    }
    if carryback {
        history.carrybacks.push(
            BusinessCreditCarryback {
                credit: BusinessCredit::DifferentialWage,
                year: 2025,
                amount: 300_000,
                research_source: None,
            }
            .into(),
        );
    }
    client.save(&mut session, answers);
    let r = supported(&session.draft);
    assert_eq!(super::value(&r, "agi"), super::value(&baseline, "agi"));
    assert_eq!(super::value(&r, "taxable_income"), super::value(&baseline, "taxable_income"));
    if carryback {
        for (id, expected) in [
            ("agi", 57160),
            ("taxable_income", 24960),
            ("form_3800_line_16", 2501),
            ("general_business_credit", 2501),
            ("business_credit_differential_wage_current_year_to_next_year", 499),
        ] {
            assert_eq!(super::value(&r, id), expected, "{id}");
        }
    } else if section_b {
        // A genuine full-return source control supplements, without replacing,
        // the original explicit30000/20000 component-capacity fixture.
        for (id, expected) in [
            ("regular_income_tax", 16712),
            ("form_3800_line_14", 6760),
            ("form_3800_line_17", 9952),
            ("form_3800_line_18", 5070),
            ("form_3800_line_21", 1690),
            ("form_3800_line_26", 1690),
            ("general_business_credit", 11642),
            ("business_credit_empowerment_zone_2024_used", 1690),
            ("business_credit_empowerment_zone_2024_to_next_year", 1310),
            ("business_credit_empowerment_zone_current_year_to_next_year", 9000),
        ] {
            assert_eq!(super::value(&r, id), expected, "{id}");
        }
    } else {
        // Preserve the original own300000 + K1 400000 source facts. The
        // shared500000 ceiling precedes use; excess200000 is not carried.
        assert_eq!(super::value(&r, "form_3800_part_3_line_1k(e)"), 500000);
        assert_eq!(
            super::value(&r, "business_credit_employer_childcare_current_year_to_next_year"),
            500000 - super::value(&r, "general_business_credit")
        );
        assert!(
            r.review_notes.iter().any(|n| n.display.key == "WARN_EMPLOYER_CHILDCARE_LIMIT_ASSUMED")
        );
    }
    let saved = client.restart(&mut session);
    assert_eq!(supported(&session.draft), r);
    let reopened = client.editor(&mut session, TaxSection::MoreCredits, "REOPENED");
    if carryback {
        let mut forged = reopened.clone();
        let P::SaveMoreCredits { value, .. } = &mut forged else { panic!() };
        let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
        history.carrybacks[0].amount = 350_000;
        // A calculated carryback limit is an ordinary saved-answer correction:
        // the existing interview returns an Unsupported Review with the exact
        // issue, rather than rejecting a structurally valid source answer.
        let mut invalid_session = restart(saved.clone());
        let Reply::Response(review) = send(&mut invalid_session, forged) else { panic!() };
        client.write("LIMIT_REVIEW", &review);
        let P::Review { result, .. } = review else { panic!() };
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(
            result
                .issues
                .iter()
                .any(|issue| status_key(&issue.message).0 == "ISSUE_CREDIT_CARRYBACK_LIMIT")
        );
        assert!(result.tax.is_none());
        let invalid_saved = invalid_session.draft.clone();
        let Reply::Edit(mut correction) =
            send(&mut invalid_session, navigate(&invalid_saved, TaxSection::MoreCredits))
        else {
            panic!()
        };
        let P::SaveMoreCredits { value, .. } = &mut correction else { panic!() };
        let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
        history.carrybacks[0].amount = 300_000;
        accepted(&send(&mut invalid_session, correction));
        assert_eq!(
            super::value(&supported(&invalid_session.draft), "general_business_credit"),
            2501
        );
        assert_eq!(
            super::value(
                &supported(&invalid_session.draft),
                "business_credit_differential_wage_current_year_to_next_year"
            ),
            499
        );
        assert_eq!(session.draft, saved);
        let mut forged = reopened.clone();
        let P::SaveMoreCredits { value, .. } = &mut forged else { panic!() };
        let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
        history.carrybacks[0].year = 2024;
        rejected_with(send(&mut session, forged), "BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED");
        assert_eq!(session.draft, saved);
    }
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    std::eprintln!(
        "actual {profile} source route: AGI={}, TI={}, GBC={}",
        super::value(&r, "agi"),
        super::value(&r, "taxable_income"),
        super::value(&r, "general_business_credit")
    );
}

#[test]
fn unit16_actual_fresh_changed_filer_carry_save_reopen_restart() {
    extern crate std;
    let client = NativeInterviewCase("TAX_UNIT16_R4");
    let profile = std::env::var("TAX_UNIT16_R4_PROFILE").unwrap_or_else(|_| "you".into());
    let mut d = crate::tests::credits::form_3800::changed_filer_profile(&profile);
    assert!(d.injured_spouse.is_none());
    let wanted = d.credits.as_mut().unwrap().business_credits.take().unwrap();
    d.credits.as_mut().unwrap().business_credits = Some(empty_business_credit_history());
    let baseline = supported(&d);
    let mut session = Interview::new(TEST_USER, d);
    if client.calculation(&mut session) {
        return;
    }
    let seed = client.editor(&mut session, TaxSection::MoreCredits, "SEED");
    let mut answers = seed;
    let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
    let MoreCredit::BusinessCredits(history) = &mut value.credits[0] else { panic!() };
    *history = wanted.into();
    client.write("ANSWERS", &answers);
    if std::env::var("TAX_UNIT16_R4_SEED_ONLY").as_deref() == Ok("1") {
        return;
    }
    client.submit(&mut session, answers);
    assert!(session.draft.injured_spouse.is_none());
    let r = supported(&session.draft);
    assert_eq!(super::value(&r, "agi"), super::value(&baseline, "agi"));
    assert_eq!(super::value(&r, "taxable_income"), super::value(&baseline, "taxable_income"));
    let (expected, unused, filer) = match profile.as_str() {
        "you" => (4884, 116, CarryFiler::You),
        "deferred-you" => (4618, 5382, CarryFiler::You),
        "deferred-spouse" => (625, 9375, CarryFiler::Spouse),
        "spouse" => (479, 4521, CarryFiler::Spouse),
        "same" => (5000, 0, CarryFiler::SameFilers),
        "current" => (5363, 116, CarryFiler::You),
        "rmd" => (4904, 96, CarryFiler::You),
        "missing" => (1000, 5000, CarryFiler::You),
        "zero" => (0, 5000, CarryFiler::You),
        "short" | "qualified-election" | "capital-election" => (6080, 3920, CarryFiler::You),
        "long" => (4904, 5096, CarryFiler::You),
        "clean-28" => (6909, 3091, CarryFiler::You),
        "specified-28" => (43134, 6866, CarryFiler::You),
        "shared-election" => (1000, 10_000, CarryFiler::You),
        "qsbs" => (5503, 4497, CarryFiler::You),
        "hsa-you" => (4996, 5004, CarryFiler::You),
        "hsa-spouse" => (4713, 5287, CarryFiler::You),
        "clergy-source" => (4512, 5488, CarryFiler::You),
        "household-source" => (4507, 5493, CarryFiler::You),
        "cf-source" => (4881, 5119, CarryFiler::You),
        _ => unreachable!(),
    };
    assert_eq!(super::value(&r, "general_business_credit"), expected);
    let prefix = if filer == CarryFiler::SameFilers {
        alloc::string::String::from("business_credit_work_opportunity_2025")
    } else {
        alloc::format!(
            "business_credit_{}_source_none_filer_{}_2025",
            if profile == "clean-28" { "clean_electricity" } else { "work_opportunity" },
            filer as i32
        )
    };
    assert_eq!(super::value(&r, &alloc::format!("{prefix}_to_next_year")), unused);
    assert_eq!(
        noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"),
        matches!(profile.as_str(), "missing" | "zero" | "shared-election")
    );
    let saved = client.restart(&mut session);
    assert_eq!(supported(&session.draft), r);
    let reopened = client.editor(&mut session, TaxSection::MoreCredits, "REOPENED");
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert!(session.draft.injured_spouse.is_none());
}

mod qbi_aggregation;
include!("../activity/unit19/interview_registration.rs");

include!("interview/qbi_current_ptp/interview_registration.rs");

include!("../passthrough/k1res_d6/interview_registration.rs");

#[test]
fn federal_cap_widening_keeps_w2_schedule_one_overflow_locator() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut form = w2(1_000);
    form.wages = i64::MAX;
    form.answers.deferred_pension = i64::MAX;
    form.answers.inmate_pay = 1;
    let reply =
        send(&mut session, save_documents(&original, vec![document(0, IncomeDocument::W2(form))]));
    let Reply::Response(P::Problem { message, field }) = reply else { panic!("{reply:?}") };
    assert_eq!(message, libertas::libertas_formatted_text("W2_SCHEDULE_1_PAY_ABOVE_WAGES", &[]));
    assert_eq!(field.as_deref(), Some("[documents/documents/0/document/answers/inmate_pay]"));
    assert_eq!(session.draft, original);
}

#[test]
fn federal_cap_widening_keeps_1099g_original_relationship_priority() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut form = Form1099G {
        amount: i64::MAX,
        withholding: i64::MAX,
        more: Default::default(),
        state_details: Default::default(),
    };
    // The native source check follows this relationship check. Preserve that
    // order even for a forged negative source and an overflowing partial sum.
    form.more.rtaa = i64::MAX;
    form.more.family_leave = -i64::MAX;
    form.more.market_gain = -1;
    let reply = send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::Form1099G(form))]),
    );
    let Reply::Response(P::Problem { message, field }) = reply else { panic!("{reply:?}") };
    assert_eq!(
        message,
        libertas::libertas_formatted_text("FORM_1099G_WITHHOLDING_ABOVE_BENEFITS", &[])
    );
    assert_eq!(field.as_deref(), Some("[documents/documents/0/document/withholding]"));
    assert_eq!(session.draft, original);
}

#[test]
fn unit47_q1099_box_comparison_refuses_overflow_without_changing_the_draft() {
    let (mut session, _) = education_session();
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    let mut documents = page_documents(&page).to_vec();
    let index = documents.len();
    documents.push(document(
        0,
        IncomeDocument::Q1099(q1099_page(
            i64::MAX,
            EducationAccount::Qtp { earnings: i64::MAX, basis: 1 },
            plain_withdrawal(),
            Q1099Beneficiary::Recipient,
        )),
    ));
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let request = save_topic(&saved, TaxSection::OtherIncome, documents);
    let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
        panic!("inconsistent 1099-Q boxes must be refused")
    };
    assert_eq!(message, libertas::libertas_formatted_text("Q1099_BOXES_DO_NOT_ADD_UP", &[]));
    let expected = alloc::format!("[documents/documents/{index}/document/gross]");
    assert_eq!(field.as_deref(), Some(expected.as_str()));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
}
