use crate::k1_page::{K1Partnership, K1Scorp};
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
    all_documents, catalog_estimate_documents, catalog_preview_document, open_document_center,
    open_documents, open_topic, page_documents, save_documents, save_each_topic, save_topic,
    topic_for, topic_kinds,
};

#[test]
fn document_candidate_save_is_pure_for_the_callers_accepted_return() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    for _ in 0..2 {
        assert_eq!(
            crate::tests::support::client::income_pages::saved_document_amount(
                &mut session,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(6_200_000))) }
            ),
            Some(6_200_000)
        );
    }
    assert_eq!(session.draft, before);
}

#[test]
fn m1_documents_page_maps_onto_income_records() {
    let mut original = empty_return(FederalStatus::Joint);
    original.people.as_mut().unwrap().spouse.as_mut().unwrap().first_name = "Jordan".into();
    let mut session = Interview::new(TEST_USER, original.clone());
    let P::SaveDocuments { allowed_owners, documents, .. } = open_documents(&mut session) else {
        unreachable!()
    };
    assert_eq!(allowed_owners, vec![Owner::Taxpayer as i32, Owner::Spouse as i32]);
    assert!(documents.common.wages.documents.is_empty());
    let P::SaveWagesAnswers { wages, followups, records, allowed_owners, .. } =
        open_topic(&mut session, TaxSection::Wages)
    else {
        unreachable!()
    };
    assert_eq!(allowed_owners, vec![Owner::Taxpayer as i32, Owner::Spouse as i32]);
    assert!(wages.is_empty());
    assert!(followups.is_empty());
    assert!(records.documents.is_empty());
    let mut int = Int1099 {
        more: IntMore {
            early_penalty: 1_000,
            special: InterestSpecial::NoneOfThese,
            ..Default::default()
        },
        ..native_int1099("Synthetic Bank".into(), 30_000)
    };
    let mut div = native_div1099("".into(), 50_000, 40_000);
    let mut wages = w2(5_000_000);
    wages.employee = accepted_document_names(&original, Owner::Spouse).recipient;
    int.recipient = accepted_document_names(&original, Owner::Spouse).recipient;
    div.recipient = accepted_document_names(&original, Owner::Taxpayer).recipient;
    let request = save_documents(
        &mut session,
        vec![
            owned(
                Owner::Spouse,
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wages.clone())) },
            ),
            owned(
                Owner::Spouse,
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(int.clone())),
                },
            ),
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(div.clone())),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    let first = original.next_id;
    let expected = vec![
        FederalIncomeEntry {
            printed: Some(PrintedDocumentNames {
                issuer: Some(wages.employer.clone()),
                ..accepted_document_names(&original, Owner::Spouse)
            }),
            refund_management: None,
            state_source: Default::default(),
            id: first,
            label: "".into(),
            owner: crate::DocumentOwner::Spouses,
            income: FederalIncome::Wage {
                data: wage_source(FederalWage {
                    tips_without_w2: false,
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
                }),
            },
            community: CommunityShare::NotNeeded,
        },
        FederalIncomeEntry {
            printed: Some(PrintedDocumentNames {
                issuer: Some(int.payer.clone()),
                ..accepted_document_names(&original, Owner::Spouse)
            }),
            refund_management: None,
            state_source: Default::default(),
            id: first + 1,
            label: "Synthetic Bank".into(),
            owner: crate::DocumentOwner::Spouses,
            income: FederalIncome::Interest {
                data: interest_source(native_interest(30_000, 1_000)),
            },
            community: CommunityShare::NotNeeded,
        },
        FederalIncomeEntry {
            printed: Some(PrintedDocumentNames {
                issuer: Some(div.payer.clone()),
                ..accepted_document_names(&original, Owner::Taxpayer)
            }),
            refund_management: None,
            state_source: Default::default(),
            id: first + 2,
            label: "".into(),
            owner: crate::DocumentOwner::Yours,
            income: FederalIncome::Dividend {
                data: dividend_source(FederalDividend {
                    section199a: ReitDividends::None,
                    ..native_dividend(50_000, 40_000, 0)
                }),
            },
            community: CommunityShare::NotNeeded,
        },
    ];
    // Each stored paper prints the parties its accepted names record.
    let expected: Vec<_> = expected
        .into_iter()
        .map(|mut entry| {
            prepare_income_fixture(&mut entry, None);
            // Saving the page answers no W-2 question: coverage and disability stay unanswered.
            if let FederalIncome::Wage { data: crate::WageSource::W2(data) } = &mut entry.income {
                data.answers.coverage = None;
                data.answers.disability = None;
            }
            entry
        })
        .collect();
    assert_eq!(session.draft.income, expected);
    assert!(session.draft.finished_sections.contains(&TaxSection::Documents));
    assert_eq!(session.draft.revision, original.revision + 1);
    // Reopening rebuilds the same pages from the draft, and saving them
    // unchanged changes nothing.
    let mut session = restart(session.draft);
    let accepted_draft = session.draft.clone();
    int.more.calls = IntCalls { nominee: vec![0], ..Default::default() };

    let rebuilt: Vec<_> = all_documents(&mut session)
        .iter()
        .map(|item| (item.id, item.owner.owned(), item.document.clone()))
        .collect();
    assert_eq!(
        rebuilt,
        vec![
            (
                first,
                Owner::Spouse,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wages.clone())) }
            ),
            (
                first + 1,
                Owner::Spouse,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(int))
                }
            ),
            (
                first + 2,
                Owner::Taxpayer,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(div))
                }
            ),
        ]
    );
    let page = open_document_center(&mut session);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft, accepted_draft);
    assert!(!session.dirty);
    // A single return offers only the taxpayer as a document owner.
    let mut single = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let P::SaveDocuments { allowed_owners, .. } = open_documents(&mut single) else {
        unreachable!()
    };
    assert_eq!(allowed_owners, vec![Owner::Taxpayer as i32]);
}

#[test]
fn document_more_nothing_leaves_defaults() {
    let original = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    save_each_topic(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(4_000_000))) },
            ),
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(
                        native_int1099("".into(), 20_000),
                    )),
                },
            ),
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(
                        native_div1099("".into(), 10_000, 5_000),
                    )),
                },
            ),
        ],
    );
    let income = &session.draft.income;
    assert_eq!(income.len(), 3);
    let FederalIncome::Wage { data } = &income[0].income else { panic!("W-2 expected") };
    let data = FederalWage::from(data);
    assert_eq!(data.payroll, W2Payroll::SocialSecurity);
    assert_eq!(data.employee_tips, None);
    assert_eq!(data.work_deductions, FederalWorkDeductions::NONE);
    assert_eq!(data.uncollected_tax, None);
    assert_eq!(data.disability, DisabilityIncome::None);
    assert_eq!(data.social_tips, 0);
    assert_eq!(income[0].label, "");
    let FederalIncome::Interest { data } = &income[1].income else { panic!("1099-INT expected") };
    let data = FederalInterest::from(data);
    assert_eq!(
        (data.early_penalty, data.exempt, data.private_activity, data.withholding, data.special),
        (0, 0, 0, 0, InterestSpecial::NoneOfThese)
    );
    assert_eq!(income[1].label, "");
    let FederalIncome::Dividend { data } = &income[2].income else { panic!("1099-DIV expected") };
    let data = FederalDividend::from(data);
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
    if let FederalIncome::Wage { data: crate::WageSource::W2(data) } = &mut d.income[0].income {
        data.form.social_tips = Some(12_300);
    }
    let mut session = Interview::new(TEST_USER, d);
    let documents = all_documents(&mut session);
    let IncomeSource::Wage { value: crate::WageDocument::W2(w) } = &documents[0].document else {
        panic!("W-2 expected")
    };
    assert_eq!(w.social_tips, Some(12_300));
    let IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(paper) } =
        &documents[1].document
    else {
        panic!("1099-INT expected")
    };
    assert!(paper.payer.name.is_empty());
    assert_eq!(paper.investment_expenses, 0);
    assert!(paper.foreign_country_or_us_territory.is_empty());
    let FederalIncome::Interest { data: crate::InterestSource::Int1099(source) } =
        &session.draft.income[1].income
    else {
        panic!("1099-INT source expected")
    };
    let reopened = crate::Form1099IntSource { form: paper.clone(), ..source.clone() };
    assert_eq!(
        crate::IntBoxes::from(&reopened),
        crate::IntBoxes::from(&native_int1099("".into(), 20_000))
    );
    assert_eq!((paper.early_withdrawal_penalty, paper.federal_income_tax_withheld), (0, 0));
    // The topic, not the stored paper, calls for the nominee answer, and only that one.
    let id = session.draft.income[1].id;
    let topic = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &topic else { panic!("interest topic expected") };
    let keys: Vec<_> = followups
        .iter()
        .map(|row| row.key.as_str())
        .filter(|key| key.starts_with(&alloc::format!("income/{id}/")))
        .collect();
    assert!(keys.contains(&alloc::format!("income/{id}/nominee").as_str()));
    assert!(!keys.contains(&alloc::format!("income/{id}/after_death").as_str()));
}

#[test]
fn preview_document_calculation_fills_the_row() {
    let d = empty_return(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, d.clone());
    let preview = |session: &mut Interview, document: IncomeSource| {
        crate::tests::support::client::income_pages::saved_document_amount(session, document)
    };
    let wage = IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(4_000_000))) };
    let interest = IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
            "".into(),
            20_000,
        ))),
    };
    let dividend = IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Div1099(dividend_document(Div1099 {
            capital_distributions: 2_500,
            ..native_div1099("".into(), 10_000, 5_000)
        })),
    };
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
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
            )],
        );
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
    let reject = |div: Div1099, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            let request = save_documents(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::Div1099(dividend_document(div)),
                    },
                )],
            );
            send(&mut session, request)
        };
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
        assert_eq!(session.draft, original);
    };
    // Box 2b (unrecaptured section 1250 gain) cannot exceed box 2a.
    reject(
        Div1099 {
            capital_distributions: 10_000,
            unrecaptured_distributions: 10_001,
            ..native_div1099("".into(), 100_000, 0)
        },
        "DIV_2B_ABOVE_2A",
    );
    // Box 2d (collectibles) cannot exceed box 2a either, independently of 2b.
    reject(
        Div1099 {
            capital_distributions: 10_000,
            collectible_distributions: 10_001,
            ..native_div1099("".into(), 100_000, 0)
        },
        "DIV_2D_ABOVE_2A",
    );
    // Box 13 cannot exceed box 12.
    reject(
        Div1099 {
            exempt_dividends: 5_000,
            private_activity_dividends: 5_001,
            ..native_div1099("".into(), 100_000, 0)
        },
        "DIV_13_ABOVE_12",
    );
}

#[test]
fn refund_year_is_the_deduction_year() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let refund = refund_document(
        Refund1099G {
            payer: Default::default(),
            recipient: Default::default(),
            state_details: Default::default(),
            amount: 500_000,
            year: 2019,
            withholding: 0,
        },
        RefundDeduction::IncomeTax {
            income_paid: 800_000,
            salt_paid: 800_000,
            salt_deducted: 800_000,
            itemized: 1_500_000,
            standard: 1_220_000,
            sales_available: 0,
            property_refunded: 0,
        },
    );
    {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    refund_state: None,
                    refund_locality: Default::default(),
                    value: refund_paper(refund.clone()),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    let FederalIncome::GovernmentPayment { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(data.form.state_or_local_income_tax_refunds, 500_000);
    assert_eq!(data.form.federal_income_tax_withheld, 0);
    assert_eq!(
        data.history,
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
    let documents = page_documents(&open_document_center(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::GovernmentPayment { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        vec![(documents[0].document).clone()],
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            vec![IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: refund_paper(refund)
            }]
        )
    );
    // "Payments made over several years" is accepted and stored (not
    // rejected), then left out with a notice because the page cannot
    // detail the multi-year allocation the deduction needs.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    refund_state: None,
                    refund_locality: Default::default(),
                    value: refund_document(
                        Refund1099G {
                            payer: Default::default(),
                            recipient: Default::default(),
                            state_details: Default::default(),
                            amount: 500_000,
                            year: 2025,
                            withholding: 0,
                        },
                        RefundDeduction::SeveralYears,
                    ),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_TAX_REFUND_LEFT_OUT"), "{r:?}");
    // Reopening a several-years refund reopens as the same choice.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_document_center(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::GovernmentPayment { .. }))
        .collect::<Vec<_>>();
    let IncomeSource::GovernmentPayment { value: _reopened, .. } = &documents[0].document else {
        panic!("expected a refund document, got {:?}", documents[0].document)
    };
    let FederalIncome::GovernmentPayment { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert!(matches!(
        data.history,
        TaxRefundHistory::IncomeTaxMultiplePriorYears { allocation_complete: false, .. }
    ));
}

pub(in crate::tests) fn r1099(
    gross: i64,
    code: DistributionCode,
    ira: bool,
) -> crate::R1099Answered {
    crate::R1099Answered::new(
        crate::R1099Source {
            form: R1099 {
                payer: Default::default(),
                payer_telephone: alloc::string::String::new(),
                account_number: alloc::string::String::new(),
                recipient: Default::default(),
                gross,
                taxable: Some(gross),
                taxable_not_determined: false,
                total_distribution: false,
                employee_contributions: Some(0),
                excess_contribution_earnings: Some(0),
                state_details: Default::default(),
                withholding: Some(0),
                box7: box7_of(&code),
                ira,
                trump: false,
                box_3: Some(0),
                box_6: Some(0),
                box_8a: Some(0),
                box_8b_percent: None,
                box_9a_percent: None,
                box_9b: None,
                box_10: Some(0),
                first_roth_year: 0,
                fatca: false,
                payment_date: None,
            },
            code: Some(code),
            taxable_method: Some(crate::TaxableMethod::BoxTwoA),
            disability: None,
            box_10_basis: None,
            disaster: None,
            after_death: None,
            ira_basis: false,
        },
        Vec::new(),
    )
}
pub(in crate::tests) fn normal_kept() -> DistributionCode {
    DistributionCode::Normal { what_happened: Disposition::Kept }
}
pub(in crate::tests) fn early_kept() -> DistributionCode {
    DistributionCode::Early { what_happened: Disposition::Kept, exception: EarlyException::None }
}
// A 1099-R whose box 7a codes lead to Another code.
pub(in crate::tests) fn other_1099(gross: i64, printed: &str, ira: bool) -> crate::R1099Answered {
    let mut r = r1099(gross, DistributionCode::Nonqualified, ira);
    r.source_mut().code = Some(DistributionCode::Other);
    printed_as(r, printed)
}

#[test]
fn unsupported_code_is_refused_by_name() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r.clone())),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let FederalIncome::Retirement { data } = &session.draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(
        FederalRetirement::from(data).ordinary_distribution,
        Some(RetirementTreatment::Unsupported)
    );
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
    let documents = page_documents(&open_document_center(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        vec![(documents[0].document).clone()],
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            vec![IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(r))
            }]
        )
    );
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
        accepted(&{
            let request = save_documents(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Wage {
                        value: crate::WageDocument::W2(wage_document(form.clone())),
                    },
                )],
            );
            send(&mut session, request)
        });
        let page = open_documents(&mut session);
        let reopened = page_documents(&page)[0].document.clone();
        assert_eq!(
            vec![(reopened).clone()],
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) }]
            )
        );
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
        {
            let request = save_documents(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Wage {
                        value: crate::WageDocument::W2(wage_document(form.clone())),
                    },
                )],
            );
            send(&mut session, request)
        },
        "W2_SOCIAL_TAX_ABOVE_WAGES",
    );
    assert_eq!(session.draft, original);
    assert!(!session.dirty);

    form.social_tax = 62_620;
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
            )],
        );
        send(&mut session, request)
    });
    let before = session.draft.clone();
    let dirty = session.dirty;
    let mut reopened = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut reopened else {
        panic!("expected the center wages editor")
    };
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[0].document else {
        panic!("expected the W-2")
    };
    form.social_tax = Some(1_010_001);
    rejected_with(send(&mut session, reopened), "W2_SOCIAL_TAX_ABOVE_WAGES");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    let reopened = open_documents(&mut session);
    let IncomeSource::Wage { value: crate::WageDocument::W2(form) } =
        &page_documents(&reopened)[0].document
    else {
        unreachable!()
    };
    assert_eq!(form.social_tax, Some(62_620));
}

#[test]
fn income_array_saves_atomically() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let source = open_document_center(&mut session);
    accepted(&send(&mut session, source));
    let original = session.draft.clone();
    session.dirty = false;
    let records = page_documents(&open_documents(&mut session)).to_vec();
    let request = save_documents(&mut session, records.clone());
    send(&mut session, request);
    assert_eq!(session.draft, original);
    assert!(!session.dirty);
    let mut invalid = records.clone();
    invalid.push(records[0].clone());
    let request = save_documents(&mut session, invalid);
    error(send(&mut session, request));
    assert_eq!(session.draft, original);
    let mut changed = records;
    if let IncomeSource::Wage { value: crate::WageDocument::W2(w) } = &mut changed[0].document {
        w.wages = Some(6_100_000);
    }
    let save = save_documents(&mut session, changed);
    send(&mut session, save.clone());
    assert_eq!(session.draft.income.len(), 1);
    assert_eq!(session.draft.income[0].id, 1);
    assert_eq!(session.draft.revision, original.revision + 1);
    error(send(&mut session, save));
    let request = save_documents(&mut session, vec![]);
    send(&mut session, request);
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
    let mut fixture = complete(FederalStatus::Single);
    fixture.people.as_mut().unwrap().taxpayer.first_name = "Casey".into();
    fixture.people.as_mut().unwrap().taxpayer.last_name = "Sample".into();
    name_accepted_income_documents(&mut fixture);
    let mut s = Interview::new(TEST_USER, FederalDraft::empty(fixture.cookie.clone()));
    let mut reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    // Amended for M3-P13: one more section, the federal payments page;
    // M3-P15: and the other taxes page; M3-P18: and the opening checklist.
    for _ in 0..24 {
        let p = match crate::tests::support::client::answer_no_estimates(reply) {
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
                estimate: estimate.map(Into::into),
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
                estimate: estimate.map(Into::into),
                warnings: vec![],
                progress,
                you: filer(&fixture.people.as_ref().unwrap().taxpayer),
            },
            // Amended for M3-P1: the income hub's suggestion opens the Wages
            // page, which saves the fixture's W-2; once it is saved the
            // suggestion continues, which records the hub finished.
            Reply::Response(hub @ P::Hub { .. }) => match send(&mut s, go_on(&hub)) {
                Reply::Edit(editor @ P::SaveDocuments { .. }) => {
                    answer(Reply::Edit(editor), Filing::Single)
                }
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
                Reply::Response(
                    P::Review { .. }
                    | P::Hub { .. }
                    | P::BeginPayments { .. }
                    | P::BeginOftenMissed { .. },
                )
                | Reply::Edit(
                    P::SaveDeductions { .. }
                    | P::SavePayments { .. }
                    | P::SaveMoreAdjustments { .. }
                    | P::SaveMoreCredits { .. }
                    | P::SaveOtherTaxes { .. }
                    | P::SaveOftenMissed { .. },
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
                | P::SaveOtherTaxes { .. }
                | P::SaveOftenMissed { .. }),
            ) => editor,
            // New workflows compose the required answers before submitting.
            reply @ Reply::Response(P::BeginOftenMissed { .. }) => {
                crate::tests::support::client::answer_often_missed(
                    reply,
                    crate::OftenMissed::default(),
                )
            }
            // The dependents page is asked on every return.
            reply @ Reply::Response(P::BeginChildren { .. }) => answer(reply, Filing::Single),
            // Beyond your documents: nothing applies.
            reply @ (Reply::Edit(P::SaveIncomeWithoutForm { .. })
            | Reply::Response(P::BeginIncomeWithoutForm { .. })) => without_form_editor(reply),
            reply @ (Reply::Edit(P::SaveChecklist { .. })
            | Reply::Response(P::BeginChecklist { .. })) => tick(reply, Checklist::NONE),
            Reply::Edit(editor @ P::SaveDocuments { .. }) => {
                answer(Reply::Edit(editor), Filing::Single)
            }
            Reply::Edit(editor @ P::SaveIncomes { .. }) => {
                answer(Reply::Edit(editor), Filing::Single)
            }
            Reply::Edit(editor @ P::SaveWagesAnswers { .. }) => editor,
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
    error({
        let request = save_documents(&mut s, vec![]);
        send(&mut s, request)
    });
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
            estimate: None,
            warnings: vec![],
            progress: vec![],
            you: filer(&original.people.as_ref().unwrap().taxpayer),
        },
    );
    // The people page follows your page on every return (gate fixes batch A).
    assert!(matches!(reply, Reply::Response(P::BeginChildren { .. })), "{reply:?}");
    assert_eq!(session.draft, original);
    assert!(!session.dirty);
    assert_eq!(restart(session.draft).draft, original);
}

#[test]
fn changed_history_replaces_page_atomically_and_keeps_later_pages() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let source = open_document_center(&mut session);
    accepted(&send(&mut session, source));
    let original = session.draft.clone();
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
            estimate: None,
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
            estimate: None,
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
            | (
                "unfinished",
                Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. }),
            )
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
        estimate: None,
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
        estimate: None,
        warnings: vec![],
        progress: vec![],
        you,
    }
}
// The separate return's community-property step, answered on its editor.
pub(in crate::tests) fn save_community(d: &FederalDraft, value: CommunityAnswer) -> P {
    P::SaveCommunityProperty {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_PEOPLE,
        previous: PAGE_PEOPLE,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value,
    }
}
// The separate return's spouse boxes step, answered on its editor.
pub(in crate::tests) fn save_spouse_boxes(d: &FederalDraft, value: SpouseBoxes) -> P {
    P::SaveSpouseBoxes {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_DEDUCTIONS,
        previous: PAGE_DEDUCTIONS,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value,
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
        estimate: None,
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
        estimate: None,
        warnings: vec![],
        progress: vec![],
        from_documents: vec![],
        value: value.into(),
    }
}
// Beyond your documents with only these activities checked.
pub(in crate::tests) fn activities(activities: u32) -> Checklist {
    Checklist { activities, ..Checklist::NONE }
}
// Next on the checklist page a reply opened, with these ticks.
// The unsaved page (BeginChecklist) is answered as a client builds it.
pub(in crate::tests) fn tick(reply: Reply, value: Checklist) -> P {
    let (cookie, revision, page, previous, review, estimate, progress, from_documents) = match reply
    {
        Reply::Edit(P::SaveChecklist {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            from_documents,
            ..
        }) => (cookie, revision, page, previous, review, estimate, progress, from_documents),
        Reply::Response(P::BeginChecklist {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            from_documents,
        }) => (
            cookie,
            revision,
            page,
            previous,
            review,
            estimate.map(Into::into),
            progress,
            from_documents,
        ),
        reply => panic!("expected the checklist, got {reply:?}"),
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
        from_documents,
        value: value.into(),
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
    crate::tests::support::client::answer_no_estimates(send(s, request))
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
        identification: Identification::SocialSecurityNumber,
        details: FilerDetails {
            blind: false,
            claimable: Claimable::Nobody,
            more: MoreAboutYou::Nothing,
            real_estate_professional: None,
        },
        situations: 0,
    }
}

#[test]
fn filer_details_preserve_defaults_and_each_nondefault_through_person_and_spouse() {
    for fact in 0..7 {
        let mut you = adult("Alex", "Sample", 19850301);
        match fact {
            0 => {}
            1 => you.details.blind = true,
            2 => {
                you.details.claimable = Claimable::Possibly {
                    another_eitc_child: false,
                    dependency: FilerDependency::CanBeClaimed { claimed: true },
                    us_home_over_half_year: true,
                    full_time_student: false,
                    disabled: false,
                    incapable_self_care: false,
                    support_share: SupportShare::LessThanHalf,
                    parent_alive: true,
                };
            }
            3 => you.identification = Identification::Itin,
            4 => {
                you.details.more = MoreAboutYou::Died {
                    date: 20260630,
                    estate_medical: 123_456,
                    bonds: None,
                    ordinary: true,
                };
            }
            5 => {
                you.details.real_estate_professional = Some(ProfessionalRentalsPage {
                    single_activity: ProfessionalRentalElection::SeparateActivities,
                    over_500_hours: true,
                });
            }
            6 => you.situations = YourSituations::COMMUNITY_PROPERTY.0,
            _ => unreachable!(),
        }
        assert_eq!(filer(&crate::interview::federal_person(&you)), you, "fact {fact}");
        let spouse = SpouseAnswers::from(you.clone());
        assert_eq!(SpouseAnswers::from_avro(&spouse.to_avro()).unwrap(), spouse);
        assert_eq!(Filer::from(spouse), you, "fact {fact}");
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
            separate_spouse: SeparateSpouseBoxes::NotAnswered,
            label: crate::Message::new(
                "LABEL_RETURN",
                alloc::vec![crate::MessageArg::Number(2026)]
            )
            .encode(),
            basis: AmountBasis::Actual,
            status: FilingChoice::Joint,
            marital_state: MaritalState::Married { living: SpouseLiving::Together },
            paid_over_half_home: false,
            community_property: CommunityProperty::NotAnswered,
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
            whole_return: 0,
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
        details: FilerDetails {
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
            ..adult("Jordan", "Sample", 20030720).details
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
        let mut you = adult("Alex", "Sample", 19850301);
        you.identification = identification;
        let spouse = Filer {
            identification,
            details: FilerDetails {
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
                ..adult("Jordan", "Sample", 20030720).details
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
            data: interest_source(FederalInterest {
                taxable: 30_000,
                withholding: 10_000,
                special: InterestSpecial::NoneOfThese,
                ..Default::default()
            }),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                ordinary: 20_000,
                withholding: 5_000,
                ..Default::default()
            }),
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
    // Current selections retain their declared numeric values when another
    // situation becomes supported. Refusal checks cover names and destinations.
    for (situation, value) in RareSituation::ALL.into_iter().zip([0, 1, 3, 4, 6, 7, 8, 9, 10, 11]) {
        assert_eq!(situation.to_avro(), vec![2 * value]);
        assert_eq!(RareSituation::from_avro(&[2 * value]).unwrap(), situation);
    }
    // Farm rental is entered through the Farming topic.
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
        let explanation = if matches!(item, Unprepared::OtherAdditionalTaxes { .. }) {
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

// Saves About you, opened from the review, with these whole-return items.
pub(in crate::tests) fn save_your_situations(s: &mut Interview, bits: u32) -> Reply {
    let Reply::Edit(mut request @ P::SavePeople { .. }) = from_review(s, TaxSection::People) else {
        panic!("expected About you")
    };
    if let P::SavePeople { you, .. } = &mut request {
        you.situations = bits;
    }
    send(s, request)
}
// Saves the filers' pages, opened from the review, with these whole-return
// situations: the YourSituations items on About you, the others on the
// spouse page of a joint return. Returns the last page's reply.
pub(in crate::tests) fn save_whole_return(s: &mut Interview, bits: u32) -> Reply {
    let mine = YourSituations::DUAL_STATUS.0 | YourSituations::COMMUNITY_PROPERTY.0;
    let Reply::Edit(mut request @ P::SavePeople { .. }) =
        send(s, go_to(&s.draft, PAGE_REVIEW, TaxSection::People))
    else {
        panic!("expected About you")
    };
    if let P::SavePeople { you, .. } = &mut request {
        you.situations = bits & mine;
    }
    let reply = send(s, request);
    if matches!(reply, Reply::Response(P::Problem { .. }))
        || s.draft.people.as_ref().is_none_or(|p| p.spouse.is_none())
    {
        return reply;
    }
    let Reply::Edit(mut request @ P::SaveSpouse { .. }) =
        send(s, go_to(&s.draft, PAGE_REVIEW, TaxSection::Spouse))
    else {
        panic!("expected About your spouse")
    };
    if let P::SaveSpouse { spouse, .. } = &mut request {
        spouse.situations = bits & !mine;
    }
    send(s, request)
}
// The whole-return situations are asked on the filers' pages. Two items
// are refused by name under About you, two are notices that leave the tax
// unchanged, and each page takes only its own items.
#[test]
fn whole_return_situations_refuse_and_notice_on_the_filers_pages() {
    let save = save_whole_return;
    let single = complete(FederalStatus::Single);
    for (bit, form) in [
        (YourSituations::DUAL_STATUS.0, "Form 1040-NR"),
        (YourSituations::COMMUNITY_PROPERTY.0, "Form 8958"),
    ] {
        let mut s = Interview::new(TEST_USER, single.clone());
        save(&mut s, bit);
        assert_eq!(s.draft.whole_return(), bit);
        let r = estimate(&s.draft);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(
            r.issues.iter().any(|i| i.section == "About you"
                && i.message.contains(form)
                && i.message.ends_with("is not calculated yet.")),
            "{r:?}"
        );
        // Review names it once, on About you.
        let items = crate::interview::remaining_items(&s.draft, &r, TaxSection::Review);
        assert_eq!(items.iter().filter(|i| i.section == TaxSection::People).count(), 1);
        // Unchecking it restores the supported estimate.
        save(&mut s, 0);
        assert_eq!(supported(&s.draft).tax, supported(&single).tax);
    }
    // About you offers only its own items.
    let mut s = Interview::new(TEST_USER, single.clone());
    let reply = save_your_situations(&mut s, SpouseSituations::INJURED_SPOUSE.0);
    assert_eq!(refusal(reply), "Choose only the listed items.");
    assert_eq!(s.draft.whole_return(), 0);
    // On a joint return the nonresident spouse choice and innocent spouse
    // relief are notices on the spouse page; the tax is the ordinary joint
    // return's.
    let joint = complete(FederalStatus::Joint);
    let mut s = Interview::new(TEST_USER, joint.clone());
    save(&mut s, SpouseSituations::NONRESIDENT_SPOUSE.0 | SpouseSituations::INNOCENT_SPOUSE.0);
    let r = supported(&s.draft);
    assert_eq!(r.tax, supported(&joint).tax);
    for key in [
        "NOTE_NONRESIDENT_SPOUSE_TREATED_AS_A_RESIDENT",
        "NOTE_INNOCENT_SPOUSE_RELIEF_IS_A_SEPARATE_REQUEST",
    ] {
        assert!(
            r.review_notes.iter().any(|n| n.display.key == key
                && n.context == Some(crate::IssueContext::Section(TaxSection::Spouse))),
            "{key}"
        );
    }
    // Leaving the joint return removes the spouse page with its items.
    let mut d = s.draft.clone();
    d.setup = complete(FederalStatus::Single).setup;
    crate::interview::prune(&mut d);
    assert_eq!(d.whole_return(), 0);
}

// What carries a formatted line: the field itself, or an estimate's line.
pub(in crate::tests) trait Line {
    fn line(&self) -> &[u8];
}
impl Line for [u8] {
    fn line(&self) -> &[u8] {
        self
    }
}
impl Line for Vec<u8> {
    fn line(&self) -> &[u8] {
        self
    }
}
impl Line for crate::EstimateSoFar {
    fn line(&self) -> &[u8] {
        &self.line
    }
}
impl Line for crate::EstimateCarry {
    fn line(&self) -> &[u8] {
        &self.line
    }
}
impl<T: Line> Line for Option<T> {
    fn line(&self) -> &[u8] {
        self.as_ref().map_or(&[], Line::line)
    }
}

// The text a formatted-text field shows.
pub(in crate::tests) fn shown<T: Line + ?Sized>(message: &T) -> alloc::string::String {
    use libertas::AvroDecode;
    let message = message.line();
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
        Reply::Response(P::BeginCommunityProperty { progress, .. })
        | Reply::Edit(P::SaveCommunityProperty { progress, .. }) => {
            ("community property", progress)
        }
        Reply::Response(P::BeginSpouseItemizes { progress, .. })
        | Reply::Edit(P::SaveSpouseItemizes { progress, .. }) => ("spouse itemizing", progress),
        Reply::Response(P::BeginSpouseBoxes { progress, .. })
        | Reply::Edit(P::SaveSpouseBoxes { progress, .. }) => ("spouse boxes", progress),
        Reply::Response(P::BeginChildren { progress, .. }) => ("people", progress),
        Reply::Edit(P::SaveChildren { progress, .. }) => {
            ("people", progress.as_ref().expect("supplied progress"))
        }
        Reply::Response(P::BeginIncomeWithoutForm { progress, .. })
        | Reply::Edit(P::SaveIncomeWithoutForm { progress, .. }) => {
            ("income without a form", progress)
        }
        Reply::Edit(P::SaveChecklist { progress, .. })
        | Reply::Response(P::BeginChecklist { progress, .. }) => ("checklist", progress),
        Reply::Edit(P::SaveDocuments { progress, .. }) => {
            ("documents", progress.as_ref().expect("native supplied progress"))
        }
        Reply::Response(P::Hub { progress, page: PAGE_INCOME, .. }) => ("income", progress),
        Reply::Response(P::Hub { progress, page: PAGE_ITEMIZED, .. }) => ("itemized", progress),
        Reply::Response(P::Hub { progress, page: PAGE_CREDITS, .. }) => ("credits", progress),
        Reply::Edit(P::SaveDeductions { progress, .. }) => ("deductions", progress),
        Reply::Edit(P::SavePayments { progress, .. })
        | Reply::Response(P::BeginPayments { progress, .. }) => ("payments", progress),
        Reply::Edit(P::SaveOftenMissed { progress, .. })
        | Reply::Response(P::BeginOftenMissed { progress, .. }) => ("often missed", progress),
        Reply::Edit(P::SaveMoreAdjustments { progress, .. }) => ("more adjustments", progress),
        Reply::Edit(P::SaveMoreCredits { progress, .. }) => {
            ("more credits", progress.as_ref().expect("supplied progress"))
        }
        Reply::Edit(P::SaveOtherTaxes { progress, .. }) => {
            ("other taxes", progress.as_ref().expect("supplied progress"))
        }
        Reply::Response(P::Hub { progress, .. }) => ("adjustments", progress),
        Reply::Edit(P::SaveHsa { progress, .. }) => ("hsa", progress),
        Reply::Edit(P::SaveCare { progress, .. })
        | Reply::Response(P::BeginCare { progress, .. }) => ("care", progress),
        Reply::Edit(P::SaveWagesAnswers { progress, .. }) => {
            ("wages", progress.as_ref().expect("native supplied progress"))
        }
        Reply::Edit(P::SaveIncomes { progress, topic: TaxSection::Wages, .. }) => {
            ("wages", progress.as_ref().expect("native supplied progress"))
        }
        Reply::Edit(P::SaveIncomes { progress, topic: TaxSection::SelfEmployment, .. }) => {
            ("self-employment", progress.as_ref().expect("native supplied progress"))
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
// Walks on from a reply to the review: each less-common page passed with
// "None of these", each hub continued.
pub(in crate::tests) fn walk_on_to_review(s: &mut Interview, mut reply: Reply) {
    loop {
        reply = match crate::tests::support::client::answer_no_estimates(past_less_common(s, reply))
        {
            Reply::Response(hub @ P::Hub { .. }) => send(s, hub_action(&hub)),
            Reply::Response(P::Review { .. }) => return,
            Reply::Edit(P::SaveEducation { .. }) => {
                panic!("answer the education topic before continuing to Review")
            }
            reply @ (Reply::Edit(P::SaveCare { .. }) | Reply::Response(P::BeginCare { .. })) => {
                let mut page = crate::tests::cross::interview::care_editor(reply);
                fill_care_provider(&mut page);
                send(s, page)
            }
            Reply::Edit(
                page @ (P::SaveDeductions { .. }
                | P::SavePayments { .. }
                | P::SaveIncomes { .. }
                | P::SaveWagesAnswers { .. }
                | P::SaveIncomeWithoutForm { .. }),
            ) => send(s, page),
            begin @ Reply::Response(P::BeginIncomeWithoutForm { .. }) => {
                send(s, without_form_editor(begin))
            }
            other => panic!("unexpected {other:?}"),
        };
    }
}
// Passes the empty dependents page, unchanged documents and direct-income
// answers, returning the remaining checklist.
pub(in crate::tests) fn past_people(s: &mut Interview, reply: Reply) -> Reply {
    past_people_with_self_work(s, reply, false)
}
pub(in crate::tests) fn past_people_with_self_work(
    s: &mut Interview,
    mut reply: Reply,
    self_work: bool,
) -> Reply {
    loop {
        reply = match reply {
            Reply::Response(P::BeginChildren { .. }) => {
                let request = save_children(&s.draft, vec![]);
                send(s, request)
            }
            Reply::Edit(page @ P::SaveDocuments { .. }) => send(s, page),
            reply @ (Reply::Edit(P::SaveIncomeWithoutForm { .. })
            | Reply::Response(P::BeginIncomeWithoutForm { .. })) => {
                let mut page = without_form_editor(reply);
                let P::SaveIncomeWithoutForm { value, .. } = &mut page else { unreachable!() };
                value.self_work |= self_work;
                send(s, page)
            }
            other => return other,
        };
    }
}
// Save the direct income page using its actual editor metadata.
pub(in crate::tests) fn save_without_form(
    s: &mut Interview,
    answers: crate::IncomeWithoutFormAnswers,
) -> Reply {
    let mut page = without_form_editor(from_review(s, TaxSection::IncomeWithoutForm));
    let P::SaveIncomeWithoutForm { value, .. } = &mut page else { unreachable!() };
    *value = answers;
    send(s, page)
}
// The income without a form editor as a client holds it: a saved page as
// its EditRequest supplies it; the unsaved page's WorkflowRequest built with
// both questions answered No (the client starts them unanswered).
pub(in crate::tests) fn without_form_editor(reply: Reply) -> P {
    match reply {
        Reply::Edit(page @ P::SaveIncomeWithoutForm { .. }) => page,
        Reply::Response(P::BeginIncomeWithoutForm {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            navigation,
            suggested_destination,
        }) => P::SaveIncomeWithoutForm {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: estimate.map(Into::into),
            warnings: vec![],
            progress,
            value: crate::IncomeWithoutFormAnswers { self_work: false, tips: false, other: None },
            navigation,
            suggested_destination,
        },
        other => panic!("expected the income without a form editor, got {other:?}"),
    }
}
// Answers new less-common workflows with "None of these" and preserves the
// accepted answers when reopening an editor, returning the reply that follows.
pub(in crate::tests) fn past_less_common(s: &mut Interview, mut reply: Reply) -> Reply {
    loop {
        let request = match reply {
            begin @ Reply::Response(P::BeginOftenMissed { .. }) => {
                crate::tests::support::client::answer_often_missed(
                    begin,
                    crate::OftenMissed::default(),
                )
            }
            Reply::Edit(
                page @ (P::SaveMoreAdjustments { .. }
                | P::SaveMoreCredits { .. }
                | P::SaveOtherTaxes { .. }
                | P::SaveOftenMissed { .. }),
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
            estimate: estimate.map(Into::into),
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
            estimate: estimate.map(Into::into),
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
            estimate: estimate.map(Into::into),
            warnings: vec![],
            progress,
            spouse: adult("Jordan", "Sample", 19860101).into(),
        },
        // A separate return's steps: not a community-property return, and a
        // spouse with none of the boxes.
        Reply::Response(P::BeginCommunityProperty {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
        }) => P::SaveCommunityProperty {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: estimate.map(Into::into),
            warnings: vec![],
            progress,
            value: CommunityAnswer::No,
        },
        Reply::Response(P::BeginSpouseBoxes {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
        }) => P::SaveSpouseBoxes {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: estimate.map(Into::into),
            warnings: vec![],
            progress,
            value: SpouseBoxes::NoneOfThese,
        },
        Reply::Response(P::BeginSpouseItemizes {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
        }) => P::SaveSpouseItemizes {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: estimate.map(Into::into),
            warnings: vec![],
            progress,
            value: false,
        },
        Reply::Edit(
            page @ (P::SaveCommunityProperty { .. }
            | P::SaveSpouseBoxes { .. }
            | P::SaveSpouseItemizes { .. }),
        ) => page,
        Reply::Edit(mut request @ P::SaveDocuments { .. }) => {
            let P::SaveDocuments { documents, allowed_community, .. } = &mut request else {
                unreachable!()
            };
            if documents.common.wages.documents.is_empty() {
                documents.common.wages.documents = vec![crate::WageDocumentRow {
                    id: 0,
                    owner: crate::DocumentOwner::Yours,
                    document: crate::WageDocument::W2(wage_document(W2 {
                        employer: Default::default(),
                        employee: crate::PrintedParty::from_name("Casey Sample".into()),
                        ..w2(6_000_000)
                    })),
                    community: CommunityShare::NotNeeded,
                    refund_management: None,
                    allowed_community: allowed_community.clone(),
                    preview: None,
                    investment_caps: None,
                    refund_management_needed: None,
                }];
            }
            request
        }
        Reply::Edit(request @ (P::SaveIncomes { .. } | P::SaveWagesAnswers { .. })) => request,
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
            estimate: estimate.map(Into::into),
            warnings: vec![],
            progress: Some(progress),
            dependents: Dependents { people: vec![] },
            allowed_lived,
            days_alive,
            credits: Vec::new(),
        },
        reply @ (Reply::Edit(P::SaveCare { .. }) | Reply::Response(P::BeginCare { .. })) => {
            let mut page = crate::tests::cross::interview::care_editor(reply);
            fill_care_provider(&mut page);
            page
        }
        Reply::Response(hub @ P::Hub { .. }) => go_on(&hub),
        // Beyond your documents: nothing applies; the W-2 is entered from
        // the income hub.
        reply @ (Reply::Edit(P::SaveIncomeWithoutForm { .. })
        | Reply::Response(P::BeginIncomeWithoutForm { .. })) => without_form_editor(reply),
        reply @ (Reply::Edit(P::SaveChecklist { .. })
        | Reply::Response(P::BeginChecklist { .. })) => tick(reply, Checklist::NONE),
        // The deductions page opens complete: Next saves it unchanged.
        Reply::Edit(page @ P::SaveDeductions { .. }) => page,
        // Choose no estimated payments when this synthetic walk has none.
        begin @ Reply::Response(P::BeginPayments { .. }) => {
            let Reply::Edit(page) = crate::tests::support::client::answer_no_estimates(begin)
            else {
                panic!("complete next-year choice")
            };
            page
        }
        begin @ Reply::Response(P::BeginOftenMissed { .. }) => {
            crate::tests::support::client::answer_often_missed(begin, OftenMissed::default())
        }
        Reply::Edit(page @ P::SavePayments { .. }) => page,
        // Gate fixes batch A: each less-common page opens with "None of
        // these"; Next saves it unchanged.
        Reply::Edit(
            page @ (P::SaveMoreAdjustments { .. }
            | P::SaveMoreCredits { .. }
            | P::SaveOtherTaxes { .. }
            | P::SaveOftenMissed { .. }),
        ) => page,
        // The other taxes page opens empty (M3-P15).
        other => panic!("unexpected page {other:?}"),
    }
}

#[test]
fn m1_sequence_is_four_pages() {
    // Summary-only hubs continue automatically; Review retains their answers
    // and their explicit navigation destinations.
    let native = NativeInterviewCase("ZERO_INPUT_HUBS");
    let walked = ["wages", "often missed", "review"];
    for (filing, expected) in [
        (
            Filing::Single,
            ["setup", "you", "people", "documents", "income without a form", "checklist"]
                .into_iter()
                .chain(walked)
                .collect::<Vec<_>>(),
        ),
        (
            Filing::MarriedJointly,
            ["setup", "you", "spouse", "people", "documents", "income without a form", "checklist"]
                .into_iter()
                .chain(walked)
                .collect(),
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
            assert!(visited.len() < 20, "{visited:?}");
            let request = answer(reply, filing);
            native.write_file(
                "OUTPUT",
                &alloc::format!(
                    "{}-{:02}-request.avro",
                    if joint { "joint" } else { "single" },
                    visited.len()
                ),
                &request,
            );
            reply = send(&mut s, request);
        }
        assert_eq!(visited, expected);
        // Summary sections remain represented in the return's progress.
        let filers = if joint {
            vec![(1, 11), (2, 11), (2, 11), (3, 11)]
        } else {
            vec![(1, 11), (2, 11), (3, 11)]
        };
        let sections: Vec<_> =
            filers.into_iter().chain([(4, 11), (5, 11), (0, 0), (7, 11), (10, 11)]).collect();
        assert_eq!(progress.len(), sections.len());
        for (shown, (n, total)) in progress.iter().zip(&sections) {
            if *n == 0 {
                // Opened before any tick: the sections that always ask.
                assert!(
                    shown.contains(
                        "Your return: Income, Adjustments to income, Credits, Situations people often miss, Review"
                    ),
                    "{shown}"
                );
                continue;
            }
            let text = alloc::format!("2026 federal · section {n} of {total}");
            assert!(shown.contains(text.as_str()), "{shown}");
        }
        let at = |name: &str| visited.iter().position(|v| *v == name).unwrap();
        assert_eq!(progress[at("documents")], "2026 federal · section 4 of 11");
        // Their draft records hold the "nothing applies" answers, so the
        // return is complete and estimated.
        let d = &s.draft;
        assert!(d.dependents.is_empty());
        assert_eq!(
            d.finished_sections,
            vec![
                TaxSection::Children,
                TaxSection::Documents,
                TaxSection::IncomeWithoutForm,
                TaxSection::Checklist,
                TaxSection::Wages,
                TaxSection::Income,
                TaxSection::Adjustments,
                TaxSection::Deductions,
                TaxSection::Credits,
                TaxSection::OtherTaxes,
                TaxSection::Payments,
                TaxSection::OftenMissed
            ]
        );
        let nothing = complete(FederalStatus::Single);
        assert_eq!(d.adjustments, nothing.adjustments);
        assert_eq!(d.deductions, nothing.deductions);
        assert_eq!(d.screening, nothing.screening);
        assert!(d.credits.as_ref().is_some_and(|c| c.education.is_empty()));
        assert_eq!(estimate(d).state, ResultState::EstimatedForSupportedScope);
        let Reply::Response(P::Review { answers, .. }) = &reply else { unreachable!() };
        for section in [TaxSection::Income, TaxSection::Adjustments, TaxSection::Credits] {
            assert!(answers.iter().any(|row| row.section == section));
        }
        let saved = s.draft.clone();
        let mut resumed = restart(saved.clone());
        assert!(matches!(
            send(&mut resumed, P::OpenInterview { user: TEST_USER }),
            Reply::Response(P::Review { .. })
        ));
        for section in [TaxSection::Income, TaxSection::Adjustments, TaxSection::Credits] {
            assert!(matches!(from_review(&mut resumed, section), Reply::Response(P::Hub { .. })));
        }
        assert_eq!(resumed.draft, saved, "opening summaries preserves accepted answers");
        let Reply::Response(P::Hub { previous, .. }) =
            from_review(&mut resumed, TaxSection::Income)
        else {
            unreachable!()
        };
        let request = back(&resumed.draft, previous);
        let checklist = send(&mut resumed, request);
        assert_eq!(opened(&checklist).0, "checklist");
        let Reply::Edit(P::SaveChecklist { previous, .. }) = checklist else { unreachable!() };
        let request = back(&resumed.draft, previous);
        let income_without_form = send(&mut resumed, request);
        assert_eq!(opened(&income_without_form).0, "income without a form");
        let Reply::Edit(P::SaveIncomeWithoutForm { previous, .. }) = income_without_form else {
            unreachable!()
        };
        let request = back(&resumed.draft, previous);
        let documents = send(&mut resumed, request);
        assert_eq!(opened(&documents).0, "documents");
        assert_eq!(resumed.draft, saved, "Back preserves the accepted return");
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
    let documents = [
        document(
            0,
            IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(W2 {
                    employer: Default::default(),
                    employee: Default::default(),

                    state_details: Default::default(),
                    wages: box_("box1_wages"),
                    withholding: box_("box2_withheld"),
                    social_wages: box_("box3"),
                    social_tax: box_("box4"),
                    medicare_wages: box_("box5"),
                    medicare_tax: box_("box6"),
                    more: W2More::default(),
                })),
            },
        ),
        document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                    "".into(),
                    cents(&fixture["int1099"][0]["box1_interest"]),
                ))),
            },
        ),
    ];
    let refund = cents(&fixture["expected"]["lines"]["34"]);
    let expected = alloc::format!("${}.{:02}", refund / 100, refund % 100);
    assert_eq!(expected, "$141.00");
    let setup = ReturnSetup { filing: Filing::Single };
    let you = adult("Casey", "Sample", 19850315);
    // The source documents are accepted in the document center before the
    // remaining income facts and deduction pages.
    let wages = &documents[..1];
    let interest = &documents[1..];
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
    let center_progress =
        if let P::SaveDocuments { progress, .. } = &request { progress.clone() } else { None };
    match crate::tests::support::client::income_pages::candidate_save_review(s, request) {
        Reply::Response(
            P::RunningEstimate { estimate, progress, .. }
            | P::PaymentsEstimate { estimate, progress, .. }
            | P::IncomeRunningEstimate { estimate, progress, .. },
        ) => (shown(&estimate), shown(&progress)),
        Reply::Response(P::Review { estimate, .. }) => {
            (shown(&estimate), shown(center_progress.as_deref().unwrap_or(&[])))
        }
        other => panic!("expected the actual candidate estimate, got {other:?}"),
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
fn page_walk_estimate_documents(
    s: &mut Interview,
    _topic: TaxSection,
    documents: &[IncomeDocumentItem],
) -> P {
    // The same papers keep their accepted identities: a row saved with id 0
    // is a new paper, whose situations question is asked again.
    let rows = documents
        .iter()
        .cloned()
        .map(|mut row| {
            if row.id == 0
                && let Some(entry) = s.draft.income.iter().find(|entry| {
                    core::mem::discriminant(&entry.income)
                        == core::mem::discriminant(&crate::interview::document_income(
                            &row.document,
                        ))
                })
            {
                row.id = entry.id;
            }
            row
        })
        .collect();
    save_documents(s, rows)
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
    assert!(text.contains("not available yet"), "{text}");
    // M3-P19: progress counts the sections this return shows: setup, you,
    // people, the checklist, the four sections every return walks (gate
    // fixes batch A) and the review.
    assert!(progress.contains("section 1 of 11"), "{progress}");
    let request = save_setup(&s.draft, setup.clone());
    send(s, request);
    let (text, _) = page_walk_running(s, page_walk_estimate_you(you));
    assert!(text.contains("not available yet"), "{text}");
    let request = save_you(&s.draft, you.clone());
    send(s, request);
    let request = save_children(&s.draft, vec![]);
    let Reply::Edit(request @ P::SaveDocuments { .. }) = send(s, request) else {
        panic!("expected the document center")
    };
    let reply = send(s, request);
    let reply = past_people(s, reply);
    assert!(matches!(reply, Reply::Response(P::BeginChecklist { .. })));
    // Beyond your documents: nothing applies; its calculation replies with
    // the estimate and the planned path as its progress.
    let (_, progress) = page_walk_running(
        s,
        P::EstimateChecklist { user: TEST_USER, value: Checklist::NONE.into() },
    );
    assert!(
        progress
            .ends_with("Your return: Income, Adjustments to income, Credits, Situations people often miss, Review"),
        "{progress}"
    );
    let request = save_checklist(&s.draft, TaxSection::Documents as i32, Checklist::NONE);
    // Its Next passes the summaries and opens the last required question.
    assert!(matches!(
        send(s, request),
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
}

#[inline(never)]
fn page_walk_wages(s: &mut Interview, wages: &[IncomeDocumentItem]) {
    // A topic page estimates its unsaved documents merged into the accepted
    // return, as if the filer continued past the hub.
    // The W-2 is entered from the income hub's Wages row.
    open_topic(s, TaxSection::Wages);
    s.dirty = false; // persisted by the task
    let before = s.draft.clone();
    let request = page_walk_estimate_documents(s, TaxSection::Wages, wages);
    let (_, progress) = page_walk_running(s, request);
    assert_eq!(progress, "2026 federal · section 4 of 11");
    // A calculation is pure.
    assert_eq!(s.draft, before);
    assert!(!s.dirty);
    // Saving it opens the next planned topic (M3-P19).
    let navigation = go_to(&s.draft, PAGE_INCOME, TaxSection::Documents);
    let Reply::Edit(mut request @ P::SaveDocuments { .. }) = send(s, navigation) else {
        unreachable!()
    };
    put_walk_sources(&s.draft, &mut request, wages.to_vec());
    let reply = send(s, request);
    let reply = past_people(s, reply);
    assert!(matches!(reply, Reply::Edit(P::SaveChecklist { .. })));
}

#[inline(never)]
fn page_walk_interest(s: &mut Interview, interest: &[IncomeDocumentItem], expected: &str) {
    // The W-2's situations question is answered once ("None of these"),
    // as the walk asks it; until then the estimate waits for it.
    let wages = open_topic(s, TaxSection::Wages);
    accepted(&send(s, wages));
    let request = page_walk_estimate_documents(s, TaxSection::InterestDividends, interest);
    let (text, progress) = page_walk_running(s, request);
    assert!(text.contains(expected), "{text}");
    assert_eq!(progress, "2026 federal · section 4 of 11");
    let navigation = go_to(&s.draft, PAGE_INCOME, TaxSection::Documents);
    let Reply::Edit(mut request @ P::SaveDocuments { .. }) = send(s, navigation) else {
        unreachable!()
    };
    put_walk_sources(&s.draft, &mut request, interest.to_vec());
    let reply = send(s, request);
    let Reply::Edit(checklist @ P::SaveChecklist { .. }) = past_people(s, reply) else {
        panic!("source save returns to the checklist")
    };
    let reply = send_past_less_common(s, checklist);
    // The new 1099-INT's situations question is asked once, then the review.
    let Reply::Edit(topic @ P::SaveIncomes { topic: TaxSection::InterestDividends, .. }) = reply
    else {
        panic!("the interest topic asks its situations, got {reply:?}")
    };
    let reply = send_past_less_common(s, topic);
    assert!(matches!(reply, Reply::Response(P::Review { .. })));
    let Reply::Response(hub @ P::Hub { .. }) = from_review(s, TaxSection::Income) else {
        panic!("expected the income summary")
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
    assert!(shown(progress).contains("section 8 of 11"), "{}", shown(progress));
    let P::SaveHsa { followups, .. } = open_hsa_page(s) else { unreachable!() };
    let (text, progress) = page_walk_running(
        s,
        P::EstimateHsa {
            followups,
            questions: Vec::new(),
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
                contribution_sources: vec![],
                employer_statements: vec![],

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
        progress.contains("section 8 of 11 · Health and medical savings accounts"),
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
    assert!(shown(progress).contains("section 9 of 12"));
    let (text, progress) = page_walk_running(
        s,
        P::EstimateDeductions {
            user: TEST_USER,

            allowed_repayment: vec![0],
            home_sites: vec![],
            value: Deductions {
                car_refunds: Default::default(),
                itemize_override: None,
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
    assert!(progress.contains("section 9 of 12"), "{progress}");
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
    assert!(shown(progress).contains("section 9 of 11"));
    let other_taxes = from_review(s, TaxSection::OtherTaxes);
    let Reply::Edit(P::SaveOtherTaxes { estimate: text, progress, .. }) = &other_taxes else {
        panic!("other taxes page expected, got {other_taxes:?}")
    };
    assert!(shown(text.as_ref().expect("supplied estimate")).contains(expected));
    assert!(shown(progress.as_ref().expect("supplied progress")).contains("section 10 of 12"));
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
    assert!(shown(progress).contains("section 10 of 12"));
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
    page_walk_enqueue(&mut requests, || page_walk_estimate_setup(setup), "section 1 of 11");
    page_walk_enqueue(&mut requests, || page_walk_estimate_you(you), "section 2 of 11");
    page_walk_enqueue(
        &mut requests,
        || page_walk_estimate_documents(s, TaxSection::Wages, wages),
        "section 4 of 11",
    );
    page_walk_enqueue(
        &mut requests,
        || page_walk_estimate_documents(s, TaxSection::InterestDividends, interest),
        "section 4 of 11",
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
        "section 10 of 12",
    );
    page_walk_enqueue(
        &mut requests,
        || P::EstimateOtherTaxes {
            user: TEST_USER,
            allowed_owners: vec![0],
            mortgage_gift_owners: vec![],
            coverdell_corrections: vec![],
            value: OtherTaxes { trump_returns: Default::default(), taxes: vec![] },
        },
        "section 10 of 12",
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
    let P::SaveDocuments { estimate: text, .. } = open_documents(s) else { unreachable!() };
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
    assert!(text.contains("not available yet"), "{text}");
    // A refused answer gets the reply its save would send.
    error(crate::tests::support::client::income_pages::candidate_save_review(
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
    assert!(progress.contains("section 2 of 11"), "{progress}");
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
    let source = open_document_center(&mut s);
    accepted(&send(&mut s, source));
    let d = s.draft.clone();
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
            // The people page is asked on every return.
            ("Children and people you support", TaxSection::Children),
            ("Document center", TaxSection::Documents),
            ("Income without a form", TaxSection::IncomeWithoutForm),
            ("Beyond your documents", TaxSection::Checklist),
            ("Income", TaxSection::Income),
            ("Adjustments to income", TaxSection::Adjustments),
            ("Deductions", TaxSection::Deductions),
            // Amended for M3-P11.
            ("Credits", TaxSection::Credits),
            // Amended for M3-P15.
            ("Other taxes", TaxSection::OtherTaxes),
            // Amended for M3-P13.
            ("Federal payments", TaxSection::Payments),
            ("Situations people often miss", TaxSection::OftenMissed),
        ]
    );
    assert_eq!(status_text(&answers[3].status), "Nothing entered");
    assert_eq!(status_text(&answers[4].status), "Entered");
    assert_eq!(status_text(&answers[11].status), "Nothing entered");
    assert_eq!(status_text(&answers[12].status), "Nothing entered");
    assert_eq!(status_text(&answers[0].status), "Married filing jointly");
    assert_eq!(status_text(&answers[1].status), "Synthetic Filer");
    // A W-2 checks no box beyond the documents.
    assert_eq!(status_text(&answers[5].status), "None of these");
    assert_eq!(status_text(&answers[6].status), "None of these");
    // The income row summarizes its hub's topics.
    assert_eq!(status_text(&answers[7].status), "1 W-2");
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
                | P::SaveWagesAnswers { review: true, .. }
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
        estimate: None,
        warnings: vec![],
        progress: Some(vec![]),
        dependents: Dependents { people },
        allowed_lived: Vec::new(),
        days_alive: 365,
        credits: Vec::new(),
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
        care: Some(DependentCare::No),
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
        IncomeSource::Wage {
            value: crate::WageDocument::W2(wage_document(W2 {
                employer: Default::default(),
                employee: Default::default(),

                state_details: Default::default(),
                wages: 3_800_000,
                withholding: 160_000,
                social_wages: 3_800_000,
                social_tax: 235_600,
                medicare_wages: 3_800_000,
                medicare_tax: 55_100,
                more: W2More::default(),
            })),
        },
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
// Two children; care is paid for the younger (M2_CARE, on the care page).
pub(in crate::tests) fn m2_children() -> Vec<Dependent> {
    vec![row("Ava", "Sample", 20180214), row("Ben", "Sample", 20210930)]
}
pub(in crate::tests) const M2_CARE: [CareCosts; 2] =
    [CareCosts::No, CareCosts::Yes { expenses: 480_000, incapable_self_care: false }];
// The M2 household with its care entered on the care page.
pub(in crate::tests) fn m2_household() -> Interview {
    let mut s = household_return(m2_children());
    save_care_costs(&mut s, &M2_CARE);
    s.dirty = false; // persisted by the task
    s
}
// The care page's rows, in the people page's order, set to `care`.
pub(in crate::tests) fn put_care_costs(page: &mut P, care: &[CareCosts]) {
    let P::SaveCare { value, .. } = page else { panic!("Care page") };
    // Answered people first, then any new ones, as the page lists them.
    for (row, care) in
        value.qualifying_people.iter_mut().chain(value.new_people.iter_mut()).zip(care)
    {
        row.care = *care;
    }
    fill_care_provider(page);
}
// Saves `care` on the care page, opened from the credits hub.
pub(in crate::tests) fn save_care_costs(s: &mut Interview, care: &[CareCosts]) {
    let mut page = open_credits_topic(s, TaxSection::Care);
    put_care_costs(&mut page, care);
    accepted(&send(s, page));
}
// The deductions page's save as the sequence opened it (M3-P9); the echoed
// allowed routes are never trusted, so they are sent empty.
pub(in crate::tests) fn save_deductions(
    d: &FederalDraft,
    itemize_override: Option<ItemizeOverride>,
    gifts: i64,
) -> P {
    P::SaveDeductions {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_DEDUCTIONS,
        previous: PAGE_ADJUSTMENTS,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],

        allowed_repayment: vec![],
        home_sites: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value: Deductions {
            car_refunds: Default::default(),
            itemize_override,
            cash_gifts: gifts,
            fund_cash_gifts: 0,
            other_cash_gifts: 0,
            car_loans: d.deductions.as_ref().map_or_else(Vec::new, |deductions| {
                deductions.vehicle_loans.iter().map(Into::into).collect()
            }),
            casualties: vec![],
            repayment: RepaymentClaim::None,
        },
    }
}
// A head-of-household return entered page by page, saved through the review.
pub(in crate::tests) fn household_return(people: Vec<Dependent>) -> Interview {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m2-household".into()));
    let request = save_setup(&s.draft, head_of_household());
    accepted(&send(&mut s, request));
    let request = save_you(&s.draft, adult("Casey", "Sample", 19880610));
    accepted(&send(&mut s, request));
    let request = save_children(&s.draft, people);
    let Reply::Edit(mut request @ P::SaveDocuments { .. }) = send(&mut s, request) else {
        unreachable!()
    };
    let P::SaveDocuments { documents, allowed_community, .. } = &mut request else {
        unreachable!()
    };
    let mut source = m2_w2();
    source.document.set_printed_names(&accepted_document_names(&s.draft, source.owner.owned()));
    let IncomeSource::Wage { value } = source.document else { unreachable!() };
    documents.common.wages.documents.push(crate::WageDocumentRow {
        id: source.id,
        owner: source.owner,
        document: value,
        community: source.community,
        refund_management: source.refund_management,
        allowed_community: allowed_community.clone(),
        preview: source.preview,
        investment_caps: source.investment_caps,
        refund_management_needed: source.refund_management_needed,
    });
    let reply = send(&mut s, request);
    let reply = past_people(&mut s, reply);
    let request = tick(reply, Checklist::NONE);
    let reply = send(&mut s, request);
    walk_on_to_review(&mut s, reply);
    s.dirty = false; // persisted by the task
    s
}

#[test]
fn m2_sequence_is_five_pages_with_dependents() {
    // Required care answers remain in the walk after the empty summaries pass.
    for (filing, expected, sections) in [
        (
            head_of_household().filing,
            vec![
                "setup",
                "you",
                "people",
                "documents",
                "income without a form",
                "checklist",
                "wages",
                "care",
                "often missed",
                "review",
            ],
            vec![(1, 11), (2, 11), (3, 11), (4, 11), (5, 11), (6, 0), (7, 11), (9, 11), (10, 11)],
        ),
        (
            Filing::MarriedJointly,
            vec![
                "setup",
                "you",
                "spouse",
                "people",
                "documents",
                "income without a form",
                "checklist",
                "wages",
                "care",
                "often missed",
                "review",
            ],
            vec![
                (1, 11),
                (2, 11),
                (2, 11),
                (3, 11),
                (4, 11),
                (5, 11),
                (6, 0),
                (7, 11),
                (9, 11),
                (10, 11),
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
                        estimate: estimate.map(Into::into),
                        warnings: vec![],
                        progress: Some(progress),
                        dependents: Dependents { people: m2_children() },
                        allowed_lived,
                        days_alive,
                        credits: Vec::new(),
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
                // The younger child's care, paid: the box plans the care
                // page, which takes the costs.
                reply @ (Reply::Edit(P::SaveChecklist { .. })
                | Reply::Response(P::BeginChecklist { .. })) => {
                    tick(reply, Checklist { paid: PaidTopics::CARE.0, ..Checklist::NONE })
                }
                reply
                @ (Reply::Edit(P::SaveCare { .. }) | Reply::Response(P::BeginCare { .. })) => {
                    let mut page = crate::tests::cross::interview::care_editor(reply);
                    put_care_costs(&mut page, &M2_CARE);
                    page
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
        assert!(progress[0].contains("2026 federal · section 1 of 11"));
        for (shown, (n, total)) in progress.iter().zip(&sections).skip(1) {
            // The checklist's progress is its planned path (M3-P18).
            if shown.contains("Your return: ") {
                assert_eq!(*n, 6);
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
    let mut s = m2_household();
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
            ("Document center", "Entered", TaxSection::Documents),
            ("Income without a form", "None of these", TaxSection::IncomeWithoutForm),
            // The care page's entries check the care box.
            ("Beyond your documents", "1 topic", TaxSection::Checklist),
            ("Income", "1 W-2", TaxSection::Income),
            ("Adjustments to income", "Nothing entered", TaxSection::Adjustments),
            ("Deductions", "Compare with itemized deductions", TaxSection::Deductions),
            // Amended for M3-P11: the credits row counts the care topic.
            ("Credits", "1 topic", TaxSection::Credits),
            // Amended for M3-P15.
            ("Other taxes", "Nothing entered", TaxSection::OtherTaxes),
            ("Federal payments", "Nothing entered", TaxSection::Payments),
            ("Situations people often miss", "None of these", TaxSection::OftenMissed),
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
    // Remove the spouse's document in its source editor, then change the filing status.
    let mut joint = complete(FederalStatus::Joint);
    joint.dependents = vec![dependent_record(joint.next_id, "Ava", "Sample", 20180214)];
    joint.next_id += 1;
    let id = joint.next_id;
    add(&mut joint, wage(id, Owner::Spouse, 2_000_000, 200_000));
    let mut s = Interview::new(TEST_USER, joint.clone());
    let mut source = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut source else { unreachable!() };
    documents.common.wages.documents.retain(|row| row.owner != crate::DocumentOwner::Spouses);
    accepted(&send(&mut s, source));
    let request = reviewing(save_setup(&s.draft, ReturnSetup { filing: Filing::Single }));
    review(&mut s, request);
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
    let checklist = past_people_with_self_work(&mut s, reply, true);
    // A business of your own: the walk opens the planned Self-employment
    // page; its Back opens the hub.
    let request = tick(checklist, Checklist::NONE);
    let Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, previous, .. }) =
        send(&mut s, request)
    else {
        panic!("expected the Self-employment page")
    };
    let request = back(&s.draft, previous);
    let Reply::Response(hub @ P::Hub { .. }) = send(&mut s, request) else {
        panic!("expected the income hub")
    };
    (s, hub)
}
// Next opens the first unfinished page, even after a jump ahead: continuing
// a later hub reached with Go to does not skip the income step.
#[test]
fn continue_after_a_jump_ahead_opens_the_first_unfinished_page() {
    let (mut s, _) = at_the_income_hub();
    let request = navigate(&s.draft, TaxSection::Credits);
    let reply = send(&mut s, request);
    let Reply::Response(credits @ P::Hub { .. }) = reply else {
        panic!("expected the credits hub")
    };
    let reply = send(&mut s, hub_action(&credits));
    assert!(
        matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. })),
        "{reply:?}"
    );
    assert!(s.draft.finished_sections.contains(&TaxSection::Credits));
    // Resume opens the same page.
    let reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    assert!(matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. })));
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
// The rows of a hub with no uncommon-items page, each topic not started.
pub(in crate::tests) fn not_started_rows(not_started: &[TaxSection]) -> Vec<Row> {
    not_started.iter().map(|t| row_of(section_name(*t), "Not started", *t)).collect()
}
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
const ALL_INCOME_TOPICS: [TaxSection; 15] = [
    TaxSection::Wages,
    TaxSection::InterestDividends,
    TaxSection::InvestmentSales,
    TaxSection::Retirement,
    TaxSection::SelfEmployment,
    TaxSection::Rentals,
    TaxSection::GovernmentBenefits,
    TaxSection::OtherIncome,
    TaxSection::LongTermCareBenefits,
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
    document(
        0,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "".into(),
                taxable,
            ))),
        },
    )
}

#[test]
fn income_hub_suggests_the_planned_topic_then_continue() {
    let (mut s, hub) = at_the_income_hub();
    let P::Hub { page, previous, review, suggested, allowed_sections, progress, .. } = &hub else {
        unreachable!()
    };
    // The hub separates Actions from Go to, and each bound topic link
    // opens its section in one tap.
    // The previous section is Beyond your documents, which planned a
    // business: the hub suggests Self-employment because it is planned (a
    // return with nothing checked suggests Continue), and its first row is
    // what the page says.
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
            TaxSection::LongTermCareBenefits,
            TaxSection::Gambling,
            TaxSection::Farming,
            TaxSection::PassThrough,
            TaxSection::CanceledDebt,
            TaxSection::WorkAbroad,
            TaxSection::HomeOffices,
            TaxSection::Review,
        ]
        .map(|t| t as i32),
    );
    assert_eq!(*suggested, TaxSection::SelfEmployment);
    // Setup, you, people, the checklist, income, adjustments, credits,
    // other taxes and the review.
    assert!(shown(progress).contains("2026 federal · section 7 of 11"));
    assert_eq!(told_row(&hub), ("What you told us".into(), "None of these".into()));
    // The planned topic first, then every other topic.
    let planned = TaxSection::SelfEmployment;
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of("Self-employment", "Planned", planned)],
            not_started_rows(&income_topics_but(&[planned]))
        )
    );
    // OK opens the Self-employment page; saving a business walks on to Less
    // common income (gate fixes batch A), whose Next ends the walk and
    // returns to the hub, which now suggests continuing to the next section.
    let Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. }) =
        send(&mut s, go_on(&hub))
    else {
        panic!("expected the Self-employment page")
    };
    let request = save_topic(&mut s, planned, vec![business_document(5_000_000)]);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    let Reply::Response(hub) = from_review(&mut s, TaxSection::Income) else {
        panic!("expected the income summary")
    };
    let P::Hub { suggested, .. } = &hub else { panic!("expected the hub, got {hub:?}") };
    assert_eq!(*suggested, TaxSection::Adjustments);
    // Amended for M3-N1: every other topic is listed "Not started", and
    // Continue is the last row.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of("Self-employment", "1 business", planned)],
            not_started_rows(&income_topics_but(&[planned]))
        )
    );
    // A topic with documents shows its count and amount.
    accepted(&save_walk_sources(&mut s, vec![interest(20_000)]));
    let Reply::Response(hub) = from_review(&mut s, TaxSection::Income) else {
        panic!("expected the income summary")
    };
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![
                // Its situations question is still to be answered.
                row_of(
                    "Interest and dividends",
                    "Needs attention: Fix",
                    TaxSection::InterestDividends
                ),
                row_of("Self-employment", "1 business", planned),
            ],
            not_started_rows(&income_topics_but(&[TaxSection::InterestDividends, planned]))
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
            .contains("section 7 of 11 · Interest and dividends")
    );
    // Back opens the hub; the hub is a response and has no Back of its own.
    let request = back(&s.draft, *previous);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Hub { .. })));
    // Saving, empty or not, continues the walk (M3-P19): the planned
    // Self-employment page is still to visit, so it opens; with the walk
    // ended the save returns to the hub (revisit_from_the_hub_returns_to_it).
    let wages_next = |reply: Reply| {
        matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. }))
    };
    assert!(wages_next(send(&mut s, editor.clone())));
    let reply = save_walk_sources(&mut s, vec![interest(20_000)]);
    // The new 1099-INT's situations question is asked first, once.
    assert!(
        matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::InterestDividends, .. })),
        "{reply:?}"
    );
    // Printed forms reopen in their owning source editor; the topic only
    // shows their read-only projection.
    let reopened = open_document_center(&mut s);
    let rows: Vec<_> = page_documents(&reopened)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
        .collect();
    assert_eq!(rows.len(), 1);
    let readonly = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { documents, source_documents, .. } = &readonly else { unreachable!() };
    assert!(documents.documents.is_empty());
    assert!(
        matches!(source_documents, Some(crate::TopicDocumentViews::InterestDividends { documents }) if documents.len() == 1)
    );
    let reply = save_walk_sources(
        &mut s,
        vec![document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(1_000_000))) },
        )],
    );
    accepted(&reply);
    // The first Save answers the papers' situations question; a second,
    // unchanged Save changes nothing.
    let readonly = open_topic(&mut s, TaxSection::InterestDividends);
    accepted(&send(&mut s, readonly));
    let before = s.draft.clone();
    let readonly = open_topic(&mut s, TaxSection::InterestDividends);
    accepted(&send(&mut s, readonly));
    assert_eq!(s.draft, before);
}

#[test]
fn topic_is_part_of_the_draft_identity() {
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    // Both topic editors share the accepted identity. Wages has its own
    // request variant and page identity; generic income topics carry a topic.
    let identity = |p: &P| match p {
        P::SaveWagesAnswers { cookie, revision, review, page, .. } => {
            assert_eq!(*page, TaxSection::Wages as i32);
            (cookie.clone(), *revision, *review, TaxSection::Wages)
        }
        P::SaveIncomes { cookie, revision, review, topic, .. } => {
            (cookie.clone(), *revision, *review, *topic)
        }
        other => panic!("expected a topic page, got {other:?}"),
    };
    let wage_request = open_topic(&mut s, TaxSection::Wages);
    let interest_request = open_topic(&mut s, TaxSection::InterestDividends);
    let wages = identity(&wage_request);
    let interest_page = identity(&interest_request);
    assert_eq!((&wages.0, wages.1, wages.2), (&interest_page.0, interest_page.1, interest_page.2));
    assert_eq!(wages.3, TaxSection::Wages);
    assert_eq!(interest_page.3, TaxSection::InterestDividends);
    let documents = vec![document(
        0,
        IncomeSource::Record {
            value: crate::IncomeRecord::SellerMortgageInterest(crate::SellerMortgageInterest {
                buyer: crate::SellerMortgageParty {
                    buyer: "Buyer".into(),
                    buyer_tin: "123456789".into(),
                    buyer_address: "100 Main Street".into(),
                },
                interest: 20_000,
            }),
        },
    )];
    let records: Vec<_> = documents
        .clone()
        .into_iter()
        .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
        .collect();
    // The Wages record type excludes a seller mortgage before a request can
    // be composed. Its own preview uses the actual server-supplied wage page.
    assert!(crate::WagesRecordItem::from_income(records[0].clone()).is_none());
    let P::SaveWagesAnswers {
        businesses,
        wage_scorporations,
        allowed_wage_scorporations,
        allowed_owners,
        allowed_community,
        allowed_refund_management,
        wages,
        followups,
        records: wage_records,
        ..
    } = wage_request.clone()
    else {
        unreachable!()
    };
    let before = s.draft.clone();
    assert!(matches!(
        send(
            &mut s,
            P::EstimateWagesAnswers {
                user: TEST_USER,
                businesses,
                wage_scorporations,
                allowed_wage_scorporations,
                allowed_owners,
                allowed_community,
                allowed_refund_management,
                wages,
                followups,
                records: wage_records,
                questions: Vec::new(),
                situations: crate::DocumentSituations::NoneOfThese,
                allowed_situations: Vec::new(),
                situation_answers: Vec::new(),
                situation_documents: Vec::new(),
                disaster_choices: Vec::new(),
                people: Vec::new(),
                inherited_iras: Vec::new(),
                inherited_roth_iras: Vec::new(),
            }
        ),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(s.draft, before);
    let P::SaveIncomes { followups, .. } = interest_request else { unreachable!() };
    let estimate_request = catalog_estimate_documents(
        TaxSection::InterestDividends,
        IncomeRecords { qbi_aggregations: Vec::new(), documents: records }.into(),
        followups,
    );
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut s,
            estimate_request
        ),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(s.draft, before);
    accepted(&send(&mut s, wage_request));
    assert!(s.draft.income.is_empty());
    // A section that is not an income topic is not a documents page.
    let before = s.draft.clone();
    error({
        let mut request = save_topic(&mut s, TaxSection::InterestDividends, documents.clone());
        let P::SaveIncomes { topic, .. } = &mut request else { unreachable!() };
        *topic = TaxSection::Review;
        send(&mut s, request)
    });
    assert_eq!(s.draft, before);
    let request = save_topic(&mut s, TaxSection::InterestDividends, documents);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income.len(), 1);
    assert_eq!(s.draft.revision, before.revision + 1);
}

// The AI Import forms of wave 7 (1099-R, SSA-1099, W-2G, Form 3921, ISO
// 1099-B, 1099-S, 1099-C, 1099-A) validate against the same hidden year.

#[test]
fn page_opened_from_the_review_returns_to_it() {
    let mut d = complete(FederalStatus::Single);
    name_accepted_income_documents(&mut d);
    let mut s = Interview::new(TEST_USER, d.clone());
    let source = open_document_center(&mut s);
    accepted(&send(&mut s, source));
    let d = s.draft.clone();
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
            P::SavePeople { review: true, .. } | P::SaveWagesAnswers { review: true, .. }
        ));
        assert!(matches!(send(&mut s, editor), Reply::Response(P::Review { .. })));
    }
    assert_eq!(s.draft, d);
    let Reply::Edit(mut editor) = send(&mut s, go_to(&d, page, TaxSection::Documents)) else {
        panic!("expected the document center")
    };
    if let P::SaveDocuments { documents, .. } = &mut editor {
        let mut row = documents.common.wages.documents[0].clone();
        row.id = 0;
        row.document = crate::WageDocument::W2(wage_document(W2 {
            employee: accepted_document_names(&d, Owner::Taxpayer).recipient,
            ..w2(1_000_000)
        }));
        documents.common.wages.documents.push(row);
    }
    assert!(matches!(send(&mut s, editor), Reply::Response(P::Review { .. })));
    assert_eq!(s.draft.income.len(), 2);
    // The income summary opened from Review keeps that origin. Its topic
    // editor uses the normal forward path, passing summary-only hubs.
    let d = s.draft.clone();
    let Reply::Response(_hub @ P::Hub { review: true, .. }) =
        send(&mut s, go_to(&d, page, TaxSection::Income))
    else {
        panic!("expected the hub")
    };
    let editor = open_topic(&mut s, TaxSection::Wages);
    assert!(matches!(editor, P::SaveWagesAnswers { review: false, .. }));
    assert!(matches!(
        send(&mut s, editor),
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
    let d = s.draft.clone();
    // The completed summary's action returns directly to Review.
    // The summary is read again: saving the wages page answered the new W-2's
    // situations, so its action now returns directly to Review.
    let Reply::Response(hub @ P::Hub { review: true, .. }) =
        send(&mut s, go_to(&d, page, TaxSection::Income))
    else {
        panic!("expected the hub")
    };
    assert!(matches!(send(&mut s, go_on(&hub)), Reply::Response(P::Review { .. })));
    // The checklist opened from the hub returns to that explicit origin.
    let Reply::Edit(editor) = send(&mut s, go_to(&d, PAGE_INCOME, TaxSection::Checklist)) else {
        panic!("expected the checklist")
    };
    assert!(matches!(editor, P::SaveChecklist { review: false, previous: PAGE_INCOME, .. }));
    assert!(matches!(send(&mut s, editor), Reply::Response(P::Hub { page: PAGE_INCOME, .. })));
    assert_eq!(s.draft, d);
}

#[test]
fn saving_the_last_income_topic_continues_to_the_next_question() {
    let (mut s, _) = at_the_income_hub();
    let request =
        save_topic(&mut s, TaxSection::SelfEmployment, vec![business_document(5_000_000)]);
    let before_revision = s.draft.revision;
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    let passed = vec![
        TaxSection::Children,
        TaxSection::Documents,
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
        TaxSection::SelfEmployment,
        TaxSection::Income,
    ];
    assert_eq!(s.draft.finished_sections, passed);
    assert_eq!(s.draft.revision, before_revision + 1);
    let mut resumed = restart(s.draft.clone());
    assert!(matches!(
        send(&mut resumed, P::OpenInterview { user: TEST_USER }),
        Reply::Edit(P::SaveSelfEmployedBenefits { .. })
    ));
    // Opening the completed summary and its other topics remains navigation.
    s.dirty = false;
    let before = s.draft.clone();
    for section in [TaxSection::Income, TaxSection::Checklist, TaxSection::InterestDividends] {
        accepted(&send(&mut s, go_to(&before, PAGE_INCOME, section)));
    }
    assert_eq!(s.draft, before);
    assert!(!s.dirty);
    let request = open_topic(&mut s, TaxSection::SelfEmployment);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    assert_eq!(s.draft, before, "an unchanged topic adds no continuation revision");
}

// Enter the two printed source categories used by the walk fixtures into
// the actual source editor, preserving its navigation envelope.
fn put_walk_sources(draft: &FederalDraft, request: &mut P, rows: Vec<IncomeDocumentItem>) {
    let P::SaveDocuments { documents, allowed_community, .. } = request else { unreachable!() };
    let mut wages = false;
    let mut interest = false;
    for mut row in rows {
        if let Some(mut names) = row.document.printed_names()
            && names.recipient.name.is_empty()
        {
            names.recipient.name = accepted_document_names(draft, row.owner.owned()).recipient.name;
            row.document.set_printed_names(&names);
        }
        let allowed = if row.allowed_community.is_empty() {
            allowed_community.clone()
        } else {
            row.allowed_community
        };
        match row.document {
            IncomeSource::Wage { value } => {
                if !wages {
                    documents.common.wages.documents.clear();
                    wages = true;
                }
                documents.common.wages.documents.push(crate::WageDocumentRow {
                    id: row.id,
                    owner: row.owner,
                    document: value,
                    community: row.community,
                    refund_management: row.refund_management,
                    preview: row.preview,
                    investment_caps: row.investment_caps,
                    allowed_community: allowed,
                    refund_management_needed: row.refund_management_needed,
                });
            }
            IncomeSource::InterestDividend { value } => {
                if !interest {
                    documents.common.interest_dividends.documents.clear();
                    interest = true;
                }
                documents.common.interest_dividends.documents.push(
                    crate::InterestDividendDocumentRow {
                        id: row.id,
                        owner: row.owner,
                        document: value,
                        community: row.community,
                        refund_management: row.refund_management,
                        preview: row.preview,
                        investment_caps: row.investment_caps,
                        allowed_community: allowed,
                        refund_management_needed: row.refund_management_needed,
                    },
                );
            }
            _ => panic!("walk fixtures enter wage or interest sources here"),
        }
    }
}

// These manual source edits exercise Go to while the planned income walk
// remains available; source and checklist submissions are separate requests.
fn save_walk_sources(s: &mut Interview, rows: Vec<IncomeDocumentItem>) -> Reply {
    let complete = s.draft.finished_sections.contains(&TaxSection::Income);
    let navigation = go_to(&s.draft, PAGE_INCOME, TaxSection::Documents);
    let Reply::Edit(mut request @ P::SaveDocuments { .. }) = send(s, navigation) else {
        unreachable!()
    };
    put_walk_sources(&s.draft, &mut request, rows);
    let reply = send(s, request);
    let reply = past_people(s, reply);
    if complete {
        accepted(&reply);
        let request = go_to(&s.draft, TaxSection::Documents as i32, TaxSection::Income);
        send(s, request)
    } else {
        let Reply::Edit(checklist @ P::SaveChecklist { .. }) = reply else {
            panic!("source Save continues to checklist: {reply:?}")
        };
        send(s, checklist)
    }
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
                IncomeSource::Wage {
                    value: crate::WageDocument::W2(wage_document(W2 {
                        employer: Default::default(),
                        employee: Default::default(),

                        state_details: Default::default(),
                        wages: cents(&w["box1_wages"]),
                        withholding: cents(&w["box2_withheld"]),
                        social_wages: cents(&w["box3"]),
                        social_tax: cents(&w["box4"]),
                        medicare_wages: cents(&w["box5"]),
                        medicare_tax: cents(&w["box6"]),
                        more: W2More::default(),
                    })),
                },
            )
        })
        .collect();
    let ints: Vec<_> = fixture["int1099"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| interest(cents(&v["box1_interest"])))
        .collect();
    let listed = fixture["dependents"].as_array().cloned().unwrap_or_default();
    let people: Vec<Dependent> = listed
        .iter()
        .map(|p| {
            row(
                p["first_name"].as_str().unwrap(),
                p["last_name"].as_str().unwrap(),
                date(&p["birth_date"]),
            )
        })
        .collect();
    let care: Vec<CareCosts> = listed
        .iter()
        .map(|p| match p.get("care_expenses") {
            Some(amount) => CareCosts::Yes { expenses: cents(amount), incapable_self_care: false },
            None => CareCosts::No,
        })
        .collect();
    let ticks = if care.iter().any(|c| *c != CareCosts::No) {
        Checklist { paid: PaidTopics::CARE.0, ..Checklist::NONE }
    } else {
        Checklist::NONE
    };
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
            Reply::Edit(mut page @ P::SaveDocuments { .. }) => {
                put_walk_sources(&s.draft, &mut page, w2s.iter().chain(&ints).cloned().collect());
                page
            }
            // The W-2s, and the 1099-INTs when the fixture has them, are
            // entered in the Document center; beyond them, only the care the
            // fixture paid, entered on the care page.
            reply @ (Reply::Edit(P::SaveIncomeWithoutForm { .. })
            | Reply::Response(P::BeginIncomeWithoutForm { .. })) => without_form_editor(reply),
            reply @ (Reply::Edit(P::SaveChecklist { .. })
            | Reply::Response(P::BeginChecklist { .. })) => tick(reply, ticks),
            // The summary's task-selected Continue action is tapped.
            Reply::Response(hub @ P::Hub { .. }) => hub_action(&hub),
            Reply::Edit(page @ P::SaveIncomes { .. }) => page,
            reply @ (Reply::Edit(P::SaveCare { .. }) | Reply::Response(P::BeginCare { .. })) => {
                let mut page = crate::tests::cross::interview::care_editor(reply);
                put_care_costs(&mut page, &care);
                page
            }
            // The deductions page opens complete: Next saves it unchanged.
            Reply::Edit(page @ P::SaveDeductions { .. }) => page,
            begin @ Reply::Response(P::BeginPayments { .. }) => {
                let Reply::Edit(page) = crate::tests::support::client::answer_no_estimates(begin)
                else {
                    panic!("complete next-year choice")
                };
                page
            }
            begin @ Reply::Response(P::BeginOftenMissed { .. }) => {
                crate::tests::support::client::answer_often_missed(begin, OftenMissed::default())
            }
            Reply::Edit(page @ P::SavePayments { .. }) => page,
            // A topic holding a paper answers its situations question once: None of these.
            Reply::Edit(page @ P::SaveWagesAnswers { .. }) => page,
            // Each section's less-common page opens with "None of these"
            // (gate fixes batch A): Next saves it unchanged.
            Reply::Edit(
                page @ (P::SaveMoreAdjustments { .. }
                | P::SaveMoreCredits { .. }
                | P::SaveOtherTaxes { .. }
                | P::SaveOftenMissed { .. }),
            ) => page,
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
    // Nothing beyond the documents is planned; each section walks its
    // less-common page with "None of these".
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Documents,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Wages,
            TaxSection::InterestDividends,
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
            TaxSection::OftenMissed
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
            TaxSection::Documents,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Wages,
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::Care,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
            TaxSection::OftenMissed
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
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        allowed_interest_limit_election: Vec::new(),
        related: Default::default(),
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
        tips_amount_kinds: alloc::vec![1],
        overtime_amount_kinds: alloc::vec![1],
        intangible_predecessor_choices: Vec::new(),
        research_property_choices: Vec::new(),
        de_minimis: None,
        depletion: None,
        overtime: BusinessOvertimeEntry::Manual { data: None },
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
        tips: BusinessTipsEntry::Manual { data: None },
        material_participation: crate::ParticipationFacts {
            hours: 501,
            test: crate::ParticipationTest::Over500,
        },
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
        farm_rental_use: None,
        allowed_farm_rental_use: alloc::vec::Vec::new(),
        macrs_source: None,
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost,
        credit_reduction: 0,
        placed,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class,
                history: AssetHistory::New { first_year: FirstYearDeduction::Bonus },
            },
        },
        business_percent: 100,
        acquired: None,
        disposal: crate::AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    }
}
pub(in crate::tests) fn with_first_year(mut asset: Asset, first_year: FirstYearDeduction) -> Asset {
    asset.gds_mut().unwrap().history = AssetHistory::New { first_year };
    asset
}
pub(in crate::tests) fn save_businesses(
    session: &mut Interview,
    businesses: Vec<Business>,
) -> Reply {
    let request = save_topic(
        session,
        TaxSection::SelfEmployment,
        businesses
            .into_iter()
            .map(|b| document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(b) }))
            .collect(),
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
                IncomeSource::Record { value: crate::IncomeRecord::Business(b) } => b.clone(),
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
        page.more.intangible_predecessor_choices =
            crate::amortization_records::predecessor_choices(data);
        let mut document = crate::IncomeRecord::Business(page.clone());
        crate::interest_limit::refresh(&mut document, data.interest_limit_election.is_some());
        let crate::IncomeRecord::Business(projected) = document else { unreachable!() };
        page.allowed_interest_limit_election = projected.allowed_interest_limit_election;
    }
    let reopened = reopened_businesses(session.draft.clone());
    assert_eq!(reopened, expected);
    let revision = session.draft.revision;
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::SelfEmployment)).to_vec();
    let request = save_topic(&mut again, TaxSection::SelfEmployment, items);
    accepted(&send(&mut again, request));
    // The first save of the applying topic records the visit (a revision).
    let visited = session.draft.finished_sections.contains(&TaxSection::SelfEmployment);
    assert_eq!(again.draft.revision, revision + i64::from(!visited));
    assert_eq!(again.dirty, !visited);
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
        casualty_property: None,
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
        casualty_property: None,
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
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        // The complete fixture explicitly answers No for its depreciable building.
        allowed_interest_limit_election: crate::interest_limit::choices(true),
        location: crate::RentalLocation::UnitedStates,
        related: Default::default(),
        assets: alloc::vec::Vec::new(),
        address: "".into(),
        kind: RentalPropertyType::SingleFamily,
        rent,
        expenses: no_rental_expenses(),
        building: fixture_building(),
        personal_use: PersonalUse::No,
        more: rental_details(),
        section179_business: None,
        prior_home_carry: None,
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
        qbi: RentalBusiness::No,
        qbi_group: "".into(),
        payroll: BusinessPayroll::No,
        at_risk: Some(crate::RentalAtRisk::All),
        known_at_risk: vec![crate::RentalAtRisk::All],
        special: false,
    }
}
pub(in crate::tests) fn save_rentals(session: &mut Interview, rentals: Vec<Rental>) -> Reply {
    let request = save_topic(
        session,
        TaxSection::Rentals,
        rentals
            .into_iter()
            .map(|r| document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) }))
            .collect(),
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
            IncomeSource::Record { value: crate::IncomeRecord::Rental(r) } => {
                (r.clone(), item.preview)
            }
            other => panic!("expected a rental, got {other:?}"),
        })
        .collect()
}
// Saving the reopened rental page again stores no new revision.
pub(in crate::tests) fn assert_rentals_round_trip(session: &mut Interview, page: &[Rental]) {
    let reopened: Vec<Rental> =
        reopened_rentals(session.draft.clone()).into_iter().map(|(r, _)| r).collect();
    let expected = page
        .iter()
        .cloned()
        .map(|page| {
            let mut document = crate::IncomeRecord::Rental(page);
            crate::interest_limit::refresh(&mut document, false);
            let crate::IncomeRecord::Rental(page) = document else { unreachable!() };
            page
        })
        .collect::<Vec<_>>();
    assert_eq!(reopened, expected);
    let revision = session.draft.revision;
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::Rentals)).to_vec();
    let request = save_topic(&mut again, TaxSection::Rentals, items);
    accepted(&send(&mut again, request));
    assert_eq!(again.draft.revision, revision);
    assert!(!again.dirty);
}
// The last page before the review, as the review opens it.
pub(in crate::tests) fn open_often_missed(session: &mut Interview) -> P {
    match send(session, navigate(&session.draft.clone(), TaxSection::OftenMissed)) {
        Reply::Edit(page @ P::SaveOftenMissed { .. }) => page,
        begin @ Reply::Response(P::BeginOftenMissed { .. }) => {
            crate::tests::support::client::answer_often_missed(begin, OftenMissed::default())
        }
        other => panic!("expected the often-missed page, got {other:?}"),
    }
}
// A save of `value` composed from the opened page.
pub(in crate::tests) fn save_often_missed(session: &mut Interview, value: OftenMissed) -> Reply {
    let mut page = open_often_missed(session);
    let P::SaveOftenMissed { value: shown, .. } = &mut page else { unreachable!() };
    *shown = value.into();
    send(session, page)
}
// A save of the situations the App does not prepare, keeping the other answers.
pub(in crate::tests) fn save_unprepared(
    session: &mut Interview,
    unprepared: Vec<Unprepared>,
) -> Reply {
    let mut page = open_often_missed(session);
    let P::SaveOftenMissed { value, .. } = &mut page else { unreachable!() };
    value.unprepared = (!unprepared.is_empty()).then_some(unprepared);
    send(session, page)
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
        estimate: None,
        warnings: vec![],
        progress: Some(vec![]),
        allowed_owners: allowed_owners.clone(),
        mortgage_gift_owners: mortgage_gift_owners.clone(),
        coverdell_corrections: coverdell_corrections.clone(),
        value: OtherTaxes { taxes, trump_returns: Vec::new() },
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
    let P::SaveIncomes { followups, .. } = open_topic(&mut session, TaxSection::Rentals) else {
        unreachable!()
    };
    let request = {
        let mut request = catalog_estimate_documents(
            TaxSection::Rentals,
            (IncomeRecords {
                qbi_aggregations: Vec::new(),
                documents: vec![document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Rental(r.clone()) },
                )]
                .into_iter()
                .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
                .collect(),
            })
            .into(),
            followups,
        );
        let P::EstimateDocuments { allowed_documents, allowed_owners, .. } = &mut request else {
            unreachable!()
        };
        *allowed_documents =
            vec![libertas_macros::variant_index!(crate::IncomeSourceKind::Rental) as i32];
        *allowed_owners = vec![0];
        request
    };
    rejected_with(
        crate::tests::support::client::income_pages::candidate_save_review(&mut session, request),
        "RENTAL_LAND_ABOVE_COST",
    );
    let request =
        catalog_preview_document(IncomeSource::Record { value: crate::IncomeRecord::Rental(r) });
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
    .then_some(HsaEligibilityPage::FullYear { coverage: HsaCoverage::None });
    // These preexisting synthetic fixtures designate the reported principal
    // for the current contribution year; make that source fact explicit.
    let mut employer_sources =
        crate::health_employer::page_rows_current_year(d, crate::health_employer::bindings(d))
            .unwrap();
    for source in &mut employer_sources {
        source.allocation = HealthEmployerYear::CurrentYear;
    }
    let mut session = Interview::new(TEST_USER, d.clone());
    let P::SaveHsa { followups, .. } = open_hsa_page(&mut session) else { unreachable!() };
    P::SaveHsa {
        followups,
        questions: Vec::new(),
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
        estimate: None,
        warnings: vec![],
        progress: vec![],
        other_coverage_allowed: false,
        allowed_owners: vec![],
        allowed_family_split: vec![],
        ira_sources: vec![],
        archer_income: vec![],
        value: HsaPage {
            contribution_sources: vec![],
            employer_statements: vec![],

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
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
        TaxSection::Income,
    ];
    let mut s = Interview::new(TEST_USER, d);
    let wage = if w == 0 { w2(6_000_000) } else { w2_with_box_12_w(6_000_000, w) };
    let wage = named_document(
        &s.draft,
        -1,
        0,
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wage)) },
    );
    let request = crate::tests::support::client::income_pages::save_documents(&mut s, vec![wage]);
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
            | P::SaveMoreAdjustments { previous, .. },
        ) if hubs.contains(previous) => *previous,
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
    // A W-2 amount whose year is open is asked first, as a required question
    // (employer_year_questions); this helper answers the current year.
    if let Some(begin) = hsa_employer_year_begin(s)
        && matches!(&begin, P::BeginDocumentQuestions { questions, .. }
            if questions.iter().all(|q| q.key.starts_with("employer/")))
    {
        let answers = (0..hsa_open_employer_count(&begin))
            .map(|_| crate::DocumentStrictAnswer::EmployerYear {
                value: crate::HealthEmployerYear::CurrentYear,
            })
            .collect();
        crate::tests::support::client::own_questions::answer_questions(s, &begin, answers);
    }
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::Hsa);
    match send(s, request) {
        Reply::Edit(page @ P::SaveHsa { .. }) => page,
        other => panic!("expected the HSA page, got {other:?}"),
    }
}
// The HSA page's questions page when a W-2 amount's year is open, as the endpoint opens it.
pub(in crate::tests) fn hsa_employer_year_begin(s: &mut Interview) -> Option<P> {
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::Hsa);
    let reply = send(s, request);
    match crate::tests::support::client::own_questions::wire_page(s, reply) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. })
            if hsa_open_employer_count(&begin) > 0 =>
        {
            Some(begin)
        }
        _ => None,
    }
}
// The HSA page's questions page when the other filer's eligibility is open.
pub(in crate::tests) fn hsa_other_begin(s: &mut Interview) -> Option<P> {
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::Hsa);
    let reply = send(s, request);
    match crate::tests::support::client::own_questions::wire_page(s, reply) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. })
            if matches!(&begin, P::BeginDocumentQuestions { questions, .. }
                if questions.iter().any(|q| q.key == "other_coverage")) =>
        {
            Some(begin)
        }
        _ => None,
    }
}
pub(in crate::tests) fn hsa_open_employer_count(begin: &P) -> usize {
    let P::BeginDocumentQuestions { questions, .. } = begin else { unreachable!() };
    questions.iter().filter(|q| q.key.starts_with("employer/")).count()
}
pub(in crate::tests) fn hsa_page_value(page: &P) -> (&HsaPage, &[i32]) {
    let P::SaveHsa { value, allowed_family_split, .. } = page else {
        panic!("expected the HSA page, got {page:?}")
    };
    (value, allowed_family_split)
}

#[test]
fn adjustments_hub_suggests_hsa_when_box_12_w_exists() {
    // Without box 12 W nothing in the section is planned or derived: its hub
    // suggests continuing.
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 0);
    assert!(s.draft.finished_sections.contains(&TaxSection::Adjustments));
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
    assert_eq!(*suggested, TaxSection::Credits);
    // The sections counted: setup, you, people, the checklist, income,
    // adjustments, credits, other taxes and the review.
    assert!(shown(progress).contains("2026 federal · section 8 of 11"));
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
    // Amended for M3-P16: the topic's first remaining item is its attention,
    // here the W-2 amount's contribution year, which the HSA page asks.
    // Amended for M3-N1: then every other topic; Continue is a separate action.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of(
                "Health and medical savings accounts",
                "Needs attention: Choose the actual contribution year for each entered W-2 medical-account employer amount before completing this estimate.",
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
    assert!(shown(progress).contains("section 8 of 11 · Health and medical savings accounts"));
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
    // Saving the account continues; its summary still counts the account.
    let request = save_hsa(
        &s.draft,
        vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 200_000)],
        FamilySplit::NotApplicable,
    );
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
    let hub = open_adjustments_hub(&mut s);
    let P::Hub { suggested, .. } = &hub else {
        panic!("expected the adjustments hub, got {hub:?}")
    };
    assert_eq!(*suggested, TaxSection::Credits);
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
    // Continuing records the section and walks on to the review. The
    // derived HSA topic, once saved, is recorded as visited.
    let reply = send(&mut s, hub_action(&hub));
    walk_on_to_review(&mut s, reply);
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Documents,
            TaxSection::Hsa,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
            TaxSection::OftenMissed,
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
    let wage_id = s.draft.income[0].id;
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut s,
        vec![document(
            wage_id,
            IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(w2_with_box_12_w(6_000_000, 150_000))),
            },
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
        hsa_account(owner, HsaCoverageYear::Changed { months: months.into() }, 100_000)
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
    // The benefits walkthrough starts after the required source-center visit
    // and confirms there is no additional income without a form.
    let sources = open_document_center(&mut s);
    accepted(&send(&mut s, sources));
    accepted(&save_without_form(&mut s, crate::IncomeWithoutFormAnswers::default()));
    s
}

// M3-P9: the deductions page and the itemized hub.

// Accept the source documents, then open the optional deductions editor
// from the adjustments section with its ordinary forward-walk context.
pub(in crate::tests) fn at_the_deductions_page(status: FederalStatus) -> (Interview, P) {
    let mut d = complete(status);
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Adjustments,
    ];
    let mut s = Interview::new(TEST_USER, d);
    let sources = open_document_center(&mut s);
    accepted(&send(&mut s, sources));
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::Deductions);
    match send(&mut s, request) {
        Reply::Edit(page @ P::SaveDeductions { .. }) => (s, page),
        other => panic!("expected the deductions page, got {other:?}"),
    }
}
pub(in crate::tests) fn deductions_value(page: &P) -> &Deductions {
    let P::SaveDeductions { value, .. } = page else {
        panic!("expected the deductions page, got {page:?}")
    };
    value
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
        begin @ Reply::Response(P::BeginMortgages { .. }) => mortgage_editor(begin),
        other => panic!("expected the {topic:?} page, got {other:?}"),
    }
}
// The mortgage editor as a client holds it: a saved page as its EditRequest
// supplies it; the WorkflowRequest page built with the answered loans as
// copied and each new loan answered as an ordinary first lender's loan paid
// in full (the client starts those facts unanswered). Every row goes in
// `loans`; the save accepts either list.
pub(in crate::tests) fn mortgage_editor(reply: Reply) -> P {
    match reply {
        Reply::Edit(page @ P::SaveMortgages { .. }) => page,
        Reply::Response(P::BeginMortgages {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            document_facts,
            mortgage_loans,
            rental_mortgage_loans,
            value,
        }) => {
            let answered = value.loans.into_iter().map(|v| crate::MortgageDocumentView {
                id: v.id,
                value: v.value,
                property: v.property,
                lender: v.lender,
                allowed_refund_origins: v.allowed_refund_origins,
                refund_origin: v.refund_origin,
                details: v.details,
                paid_share: v.paid_share,
                insurance_contract: v.insurance_contract,
                allowed_points: v.allowed_points,
                points: v.points,
                debt_date: v.debt_date,
            });
            let new = value.new_loans.into_iter().map(|v| crate::MortgageDocumentView {
                id: v.id,
                points: if v.allowed_points == [0] {
                    crate::MortgagePointTreatment::None
                } else {
                    crate::MortgagePointTreatment::Purchase
                },
                value: v.value,
                property: v.property,
                lender: v.lender,
                allowed_refund_origins: v.allowed_refund_origins,
                refund_origin: v.refund_origin,
                details: crate::MortgageDetailsView::FirstLender(Default::default()),
                paid_share: crate::MortgagePaidShare::All,
                insurance_contract: crate::SourceDate::Unavailable,
                allowed_points: v.allowed_points,
                debt_date: v.debt_date,
            });
            P::SaveMortgages {
                user: TEST_USER,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate: estimate.map(Into::into),
                document_questions: Vec::new(),
                warnings: vec![],
                progress,
                document_facts,
                mortgage_loans,
                rental_mortgage_loans,
                value: crate::Mortgages { loans: answered.chain(new).collect(), new_loans: vec![] },
            }
        }
        other => panic!("expected the mortgage page, got {other:?}"),
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
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value,
    }
}
// The mortgage page's save as the itemized hub opened it (M3-P10).
pub(in crate::tests) fn save_mortgages(
    session: &mut Interview,
    mut value: crate::MortgageDocuments,
) -> P {
    for form in &mut value.documents {
        if form.form.borrower.name.is_empty() {
            form.form.borrower = fixture_document_names(&session.draft, -1).unwrap().recipient;
        }
    }
    let mut request = open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.mortgages = value;
    request
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
        estimate: None,
        warnings: vec![],
        progress: vec![],
        allowed_owners: vec![],
        appraisal_choices: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
pub(in crate::tests) fn stored_facts(s: &Interview) -> ItemizedExpenses {
    s.draft.deductions.as_ref().unwrap().itemized.clone()
}
pub(in crate::tests) use crate::tests::support::builders::empty_facts;

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
        let mut s = at_the_deductions_page(FederalStatus::Single).0;
        let before = s.draft.clone();
        let loan = first_lender_mortgage(interest, principal, average);
        let reply = save_linked_mortgage_rows(&mut s, vec![loan]);
        if let Some(key) = refusal {
            rejected_with(reply, key);
            assert_eq!(s.draft, before);
        } else {
            accepted(&reply);
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
    let mut s = at_the_deductions_page(FederalStatus::Single).0;
    let request = save_mortgages(
        &mut s,
        crate::MortgageDocuments {
            documents: vec![
                Mortgage1098 {
                    lender: Default::default(),
                    borrower: Default::default(),
                    id: 0,
                    interest: 1_000_000,
                    principal: 20_000_000,
                    originated: 20260201,
                    refund: 0,
                    insurance: 0,
                    points: 300_000,
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
                    lender: Default::default(),
                    borrower: Default::default(),
                    id: 0,
                    interest: 500_000,
                    principal: 10_000_000,
                    originated: 20240101,
                    refund: 0,
                    insurance: 0,
                    points: 240_000,
                    more: MortgageMore {
                        details: MortgageDetails::FirstLender(MortgageFirstLender {
                            home: MortgageHome::Second,
                            purpose: MortgageUse::Home,
                            average: AverageBalance::WithinLimit,
                        }),
                        ..MortgageMore::default()
                    },
                },
            ]
            .into_iter()
            .map(mortgage_document)
            .collect(),
        },
    );
    // The itemized walk has nothing left, so its hub was passed (G-45).
    let reply = send_past_less_common(&mut s, request);
    assert!(matches!(reply, Reply::Response(P::Review { .. })), "{reply:?}");
    let first = MortgageMore::default();
    let second = MortgageMore {
        details: MortgageDetails::FirstLender(MortgageFirstLender {
            home: MortgageHome::Second,
            purpose: MortgageUse::Home,
            average: AverageBalance::WithinLimit,
        }),
        ..MortgageMore::default()
    };
    accepted(&save_mortgage_answers(
        &mut s,
        &[
            (first, crate::MortgagePointTreatment::Purchase),
            (second, crate::MortgagePointTreatment::Amortized { term_months: 240, ended: false }),
        ],
    ));
    let loans = &s.draft.deductions.as_ref().unwrap().mortgage;
    assert_eq!(mortgage_loan(&loans[0]).points, MortgagePoints::Purchase { amount: 300_000 });
    assert_eq!(
        mortgage_loan(&loans[1]).points,
        MortgagePoints::Amortized { amount: 240_000, term_months: 240, ended: false }
    );
}

#[test]
fn carryovers_map_onto_prior_cash_charity() {
    // Each CashCarryover becomes a complete PriorCashCharityCarry entry: the
    // page always supplies both confirmations and its own record reference.
    let mut s = at_the_deductions_page(FederalStatus::Single).0;
    s.draft.checklist.carryovers |= CarryoverTopics::CHARITY.0;
    let request = save_gift_carryovers(
        &s.draft,
        GiftCarryovers {
            vehicles: vec![],
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
    // Changing the deduction preference preserves the carryover records.
    let retained = s.draft.deductions.as_ref().unwrap().charity_carryovers.clone();
    let request = save_deductions(&s.draft, None, 0);
    send(&mut s, request);
    // 2026-11-02: automatic selection retains accepted carryover facts.
    assert_eq!(s.draft.deductions.as_ref().unwrap().charity_carryovers, retained);
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
    let model = crate::federal_model::HUB_SOURCE;
    for name in [
        "SaltElection",
        "FamilySplit",
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
    assert!(type_is_marked(
        include_str!("../../home/mortgage_point_treatment.rs"),
        "MortgagePointTreatment"
    ));
    // Canonical HOME replaces the embedded HomeOffice choice.
    assert!(type_is_marked(include_str!("../../home/home_site_source.rs"), "OfficeMethod"));
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
    crate::tests::support::client::admit_native_fixture_facts(&mut s);
    let sources = open_document_center(&mut s);
    accepted(&send(&mut s, sources));
    let hub = credits_hub(from_review(&mut s, TaxSection::Credits));
    (s, hub)
}
pub(in crate::tests) fn open_credits_topic(s: &mut Interview, topic: TaxSection) -> P {
    let request = go_to(&s.draft, PAGE_CREDITS, topic);
    match send(s, request) {
        Reply::Edit(page) => page,
        begin @ Reply::Response(P::BeginCare { .. }) => care_editor(begin),
        other => panic!("expected the {topic:?} page, got {other:?}"),
    }
}
// The care editor as a client holds it: a saved page as its EditRequest
// supplies it; the unanswered page's WorkflowRequest built with the answered
// people as copied and each new person's care answered No (the client starts
// those rows unanswered).
pub(in crate::tests) fn care_editor(reply: Reply) -> P {
    match reply {
        Reply::Edit(page @ P::SaveCare { .. }) => page,
        Reply::Response(P::BeginCare {
            allowed_injured_source_facts,
            people,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            allowed_owners,
            care_filers,
            care_months,
            businesses,
            farms,
            partnerships,
            allowed_filing,
            value,
        }) => P::SaveCare {
            allowed_filing,
            allowed_injured_source_facts,
            people,
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: estimate.map(Into::into),
            warnings: vec![],
            progress,
            allowed_owners,
            care_filers,
            care_months,
            businesses,
            farms,
            partnerships,
            value: crate::CarePage {
                plan_maximum: value.plan_maximum,
                qualifying_people: value.qualifying_people,
                new_people: value
                    .new_people
                    .into_iter()
                    .map(|row| crate::CareQualifyingRow {
                        id: row.id,
                        label: row.label,
                        care: CareCosts::No,
                        under_13_child: row.under_13_child,
                    })
                    .collect(),
                providers: value.providers,
                benefits: value.benefits.into_iter().map(Into::into).collect(),
                grace: value.grace,
                own_business: value.own_business,
                filers: value.filers,
                filing: value.filing,
                unpaid_return_year_care: value.unpaid_return_year_care,
                prior_year: value.prior_year,
            },
        },
        other => panic!("expected the care page, got {other:?}"),
    }
}
pub(in crate::tests) fn education_value(page: &P) -> &EducationCredits {
    let P::SaveDocuments { documents, .. } = page else {
        panic!("expected the center education editor, got {page:?}")
    };
    &documents.common.education
}
pub(in crate::tests) fn save_education(session: &mut Interview, mut value: EducationCredits) -> P {
    for form in &mut value.students {
        if form.form.student.name.is_empty()
            && let Some(names) = fixture_document_names(&session.draft, form.student_id)
        {
            form.form.student.name = names.recipient.name;
        }
    }
    let mut request = open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.education = value;
    request
}
pub(in crate::tests) fn more_credits_value(page: &P) -> &MoreCredits {
    let P::SaveMoreCredits { value, .. } = page else {
        panic!("expected the more credits page, got {page:?}")
    };
    value
}
pub(in crate::tests) fn save_more_credits(d: &FederalDraft, value: MoreCredits) -> P {
    P::SaveMoreCredits {
        document_questions: Vec::new(),
        credit_income_sources: crate::tax_credit_bonds::income_sources(d),
        allowed_credit_owners: vec![Owner::Taxpayer as i32],
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
        estate_foreign_sources: vec![],
        capital_sales: vec![],
        foreign_situations: vec![],
        mortgage_loans: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value,
    }
}
pub(in crate::tests) fn student(student: i64, tuition: i64) -> crate::StudentCreditDocument {
    crate::StudentCreditDocument {
        id: 0,
        student_id: student,
        form: student_credit_paper(StudentCredit {
            filer: Default::default(),
            student: Default::default(),
            tuition,
            scholarships: 0,
            half_time: true,
            graduate: false,
            prior_tuition_adjustment: crate::SourceCents::NotReported,
            prior_scholarship_adjustment: crate::SourceCents::NotReported,
        }),
    }
}

pub(in crate::tests) fn save_education_answers(
    session: &mut Interview,
    students: Vec<EducationStudent>,
) -> Reply {
    let mut request = open_credits_topic(session, TaxSection::Education);
    let P::SaveEducation { value, .. } = &mut request else { unreachable!() };
    value.students = students.into_iter().map(Into::into).collect();
    send(session, request)
}

#[test]
fn two_tuition_statements_for_one_student_keep_sources_and_one_annual_answer() {
    let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Single));
    let names = accepted_document_names(&session.draft, Owner::Taxpayer);
    let mut page = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    documents.common.education.students = vec![
        {
            let base = student(-1, 300_000);
            crate::StudentCreditDocument {
                form: crate::Form1098TPaper {
                    filer: (Some(names.clone()))
                        .as_ref()
                        .and_then(|names| names.issuer.clone())
                        .unwrap_or_default(),
                    student: (Some(names.clone()))
                        .as_ref()
                        .map(|names| names.recipient.clone())
                        .unwrap_or_default(),
                    ..base.form
                },
                ..base
            }
        },
        {
            let base = student(-1, 200_000);
            crate::StudentCreditDocument {
                form: crate::Form1098TPaper {
                    filer: (Some(names.clone()))
                        .as_ref()
                        .and_then(|names| names.issuer.clone())
                        .unwrap_or_default(),
                    student: (Some(names.clone()))
                        .as_ref()
                        .map(|names| names.recipient.clone())
                        .unwrap_or_default(),
                    ..base.form
                },
                ..base
            }
        },
    ];
    accepted(&send(&mut session, page));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education.len(), 2);
    assert_eq!(credits.students.len(), 1);
    assert_ne!(credits.education[0].id, credits.education[1].id);
    assert_eq!(credits.students[0].method, None);
    let (_, annual) =
        native_education(-1, education_expenses(0), EducationChoice::AmericanOpportunity);
    accepted(&save_education_answers(&mut session, vec![annual]));
    let credits = session.draft.credits.as_ref().unwrap();
    let inputs = credits.education_inputs();
    assert_eq!(inputs[0].expenses.tuition, 500_000);
    assert_eq!(
        inputs[0].sources,
        credits.education.iter().map(|source| source.id).collect::<Vec<_>>()
    );
}

#[test]
fn validation_audit_f07_education_distinct_students_and_empty_list_save_and_reopen() {
    let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Joint));
    let mut page = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    documents.common.education.students = vec![student(0, 300_000), student(0, 200_000)];
    accepted(&send(&mut session, page));
    let credits = session.draft.credits.as_ref().unwrap();
    let ids: Vec<_> = credits.education.iter().map(|source| source.id).collect();
    assert!(credits.students.is_empty());
    assert!(credits.education.iter().all(|source| {
        source.student == 0 && source.assigned == crate::DocumentRecipient::Unknown
    }));
    assert_eq!(
        unknown_documents(&session.draft),
        ids.iter().map(|id| (*id, TaxSection::Documents)).collect::<Vec<_>>()
    );
    let page = open_credits_topic(&mut session, TaxSection::Education);
    let P::SaveEducation { value, .. } = page else { unreachable!() };
    assert!(value.documents.iter().all(|source| source.student.is_none()));
    assert!(value.students.is_empty());
    let request = added_documents_request(
        &mut session,
        TaxSection::Education,
        -1,
        &[ids[0]],
        crate::AddedOwnership::Yours,
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.credits.as_ref().unwrap().students[0].method, None);
    let (_, first) =
        native_education(-1, education_expenses(0), EducationChoice::AmericanOpportunity);
    accepted(&save_education_answers(&mut session, vec![first.clone()]));
    let request = added_documents_request(
        &mut session,
        TaxSection::Documents,
        -2,
        &[ids[1]],
        crate::AddedOwnership::Yours,
    );
    accepted(&send(&mut session, request));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.students.len(), 2);
    assert_eq!(credits.students[1].method, None);
    assert!(!credits.education_pending().is_empty());
    let mut request = open_credits_topic(&mut session, TaxSection::Education);
    let P::SaveEducation { value, .. } = &mut request else { unreachable!() };
    assert_eq!(
        value.documents.iter().map(|source| source.student).collect::<Vec<_>>(),
        vec![Some(-1), Some(-2)]
    );
    let mut owners = open_document_center(&mut session);
    let P::SaveDocuments { document_owners, .. } = &mut owners else { unreachable!() };
    document_owners.iter_mut().find(|person| person.person == -2).unwrap().documents.clear();
    accepted(&send(&mut session, owners));
    accepted(&save_education_answers(&mut session, vec![first.clone()]));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.students.len(), 1);
    assert_eq!(credits.education[1].student, 0);
    assert_eq!(credits.education[1].assigned, crate::DocumentRecipient::Unknown);
    let request = added_documents_request(
        &mut session,
        TaxSection::Education,
        -1,
        &[ids[1]],
        crate::AddedOwnership::Yours,
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.credits.as_ref().unwrap().students.len(), 1);
    let reopened = open_document_center(&mut session);
    let expected: Vec<_> = [(ids[0], 300_000), (ids[1], 200_000)]
        .into_iter()
        .map(|(id, tuition)| crate::StudentCreditDocument { id, ..student(-1, tuition) })
        .collect();
    assert_eq!(education_value(&reopened).students, expected);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { document_owners, .. } = &mut request else { unreachable!() };
    document_owners.iter_mut().find(|person| person.person == -1).unwrap().documents.clear();
    accepted(&send(&mut session, request));
    assert!(session.draft.credits.as_ref().unwrap().students.is_empty());
    let mut reopened = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut reopened else { unreachable!() };
    documents.common.education.students.clear();
    accepted(&send(&mut session, reopened));
    assert!(education_value(&open_document_center(&mut session)).students.is_empty());
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
    let mut draft = complete(FederalStatus::Joint);
    draft.people.as_mut().unwrap().spouse.as_mut().unwrap().first_name = "Jordan".into();
    name_accepted_income_documents(&mut draft);
    let (mut s, _) = at_the_credits_hub(draft);
    let page = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let people = education_value(&page).people.clone();
    let labels: Vec<_> = people.iter().map(|p| status_text(&p.label)).collect();
    assert_eq!(labels, vec!["You", "Your spouse"]);
    let graduate = {
        let base = student(-1, 400_000);
        crate::StudentCreditDocument {
            form: crate::Form1098TPaper {
                filer: (Some(accepted_document_names(&s.draft, Owner::Taxpayer)))
                    .as_ref()
                    .and_then(|names| names.issuer.clone())
                    .unwrap_or_default(),
                student: (Some(accepted_document_names(&s.draft, Owner::Taxpayer)))
                    .as_ref()
                    .map(|names| names.recipient.clone())
                    .unwrap_or_default(),
                box_9: true,
                ..base.form
            },
            ..base
        }
    };
    let part_time = {
        let base = student(-2, 300_000);
        crate::StudentCreditDocument {
            form: crate::Form1098TPaper {
                filer: (Some(accepted_document_names(&s.draft, Owner::Spouse)))
                    .as_ref()
                    .and_then(|names| names.issuer.clone())
                    .unwrap_or_default(),
                student: (Some(accepted_document_names(&s.draft, Owner::Spouse)))
                    .as_ref()
                    .map(|names| names.recipient.clone())
                    .unwrap_or_default(),
                box_8: false,
                ..base.form
            },
            ..base
        }
    };
    let request = save_education(
        &mut s,
        EducationCredits { students: vec![graduate, part_time.clone()], people: people.clone() },
    );
    accepted(&send(&mut s, request));
    let (_, mut first) =
        native_education(-1, education_expenses(0), EducationChoice::LifetimeLearning);
    first.more.books_from_school = 50_000;
    first.more.other_books = 30_000;
    let (_, second) =
        native_education(-2, education_expenses(0), EducationChoice::LifetimeLearning);
    let reply = save_education_answers(&mut s, vec![first, second]);
    accepted(&reply);
    walk_on_to_review(&mut s, reply);
    let r = continued(&mut s);
    assert_eq!(value_of(&r, "education_credit"), 1_500);
    assert_eq!(value_of(&r, "refundable_education_credit"), 0);
    for (used_up, expected) in
        [(true, EducationChoice::LifetimeLearning), (false, EducationChoice::AmericanOpportunity)]
    {
        let request = save_education(
            &mut s,
            EducationCredits {
                students: vec![{
                    let base = part_time.clone();
                    crate::StudentCreditDocument {
                        form: crate::Form1098TPaper { box_8: true, ..base.form },
                        ..base
                    }
                }],
                people: people.clone(),
            },
        );
        accepted(&send(&mut s, request));
        let (_, mut annual) = native_education(-2, education_expenses(0), expected);
        annual.more.american_opportunity_used_up = Some(used_up);
        let reply = save_education_answers(&mut s, vec![annual]);
        accepted(&reply);
        walk_on_to_review(&mut s, reply);
        assert_eq!(s.draft.credits.as_ref().unwrap().students[0].method, Some(expected));
    }
    // An unmatched printed student stays pending on Education, with no annual
    // answers or credit inferred from the document's tuition.
    let mut unmatched = student(0, 100_000);
    unmatched.form.student = crate::PrintedParty::from_name("Unmatched student".into());
    let request = save_education(
        &mut s,
        EducationCredits { students: vec![unmatched], people: people.clone() },
    );
    accepted(&send(&mut s, request));
    let credits = s.draft.credits.as_ref().unwrap();
    let id = credits.education[0].id;
    assert_eq!(credits.education[0].student, 0);
    assert_eq!(credits.education[0].assigned, crate::DocumentRecipient::Unknown);
    assert!(credits.students.is_empty());
    assert_eq!(unknown_documents(&s.draft), vec![(id, TaxSection::Documents)]);
    let result = estimate(&s.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert_eq!(result.tax, None);
    let request = open_credits_topic(&mut s, TaxSection::Education);
    let P::SaveEducation { value, .. } = request else { unreachable!() };
    assert_eq!(value.documents[0].student, None);
    assert!(value.students.is_empty());
    // A catalog from before a household change still requires reopening.
    let request =
        save_education(&mut s, EducationCredits { students: vec![], people: people[..1].to_vec() });
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
    // Household removal preserves source facts until the source editor
    // removes that student's document; the second student's identity remains.
    let s = m2_household();
    let (mut s, _) = at_the_credits_hub(s.draft.clone());
    let page = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let people = education_value(&page).people.clone();
    assert_eq!(people.len(), 3);
    let first = people[1].id;
    let second = people[2].id;
    let request = save_education(
        &mut s,
        EducationCredits {
            students: vec![student(first, 100_000), student(second, 200_000)],
            people,
        },
    );
    accepted(&send(&mut s, request));
    let students = [(first, 100_000), (second, 200_000)]
        .into_iter()
        .map(|(id, tuition)| {
            native_education(id, education_expenses(tuition), EducationChoice::AmericanOpportunity)
                .1
        })
        .collect();
    let reply = save_education_answers(&mut s, students);
    accepted(&reply);
    walk_on_to_review(&mut s, reply);
    let d = s.draft.clone();
    let mut shown = reopened_people(&mut s);
    shown.remove(0);
    let reply = send(&mut s, reviewing(save_children(&d, shown)));
    accepted(&reply);
    assert!(s.draft.dependents().iter().all(|person| person.id != first));
    assert_eq!(s.draft.credits.as_ref().unwrap().education, d.credits.as_ref().unwrap().education);
    assert!(s.draft.valid_stored());
    let incomplete = estimate(&s.draft);
    assert_eq!(incomplete.state, ResultState::Incomplete);
    assert_eq!(incomplete.tax, None);
    let removed_id =
        d.credits.as_ref().unwrap().education.iter().find(|row| row.student == first).unwrap().id;
    let request = open_document_center(&mut s);
    accepted(&send(&mut s, request));
    let credits = s.draft.credits.as_ref().unwrap();
    let removed = credits.education.iter().find(|row| row.id == removed_id).unwrap();
    let original =
        d.credits.as_ref().unwrap().education.iter().find(|row| row.id == removed_id).unwrap();
    assert_eq!(removed.form, original.form);
    assert_eq!(removed.student, 0);
    assert_eq!(removed.assigned, crate::DocumentRecipient::Unknown);
    assert_eq!(credits.students.len(), 1);
    assert_eq!(credits.students[0].student, second);
    assert_eq!(unknown_documents(&s.draft), vec![(removed_id, TaxSection::Documents)]);
    let request = open_credits_topic(&mut s, TaxSection::Education);
    let P::SaveEducation { value, .. } = request else { unreachable!() };
    assert_eq!(value.documents.iter().find(|row| row.id == removed_id).unwrap().student, None);
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.education.students.retain(|row| row.id != removed_id);
    accepted(&send(&mut s, request));
    let credits = s.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education.len(), 1);
    assert_eq!(credits.education[0].form.box_1, 200_000);
    assert_eq!(credits.education[0].student, second);
    let page = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let original =
        d.credits.as_ref().unwrap().education.iter().find(|row| row.student == second).unwrap();
    let mut expected = student(second, 200_000);
    expected.id = original.id;
    expected.form.student = original.form.student.clone();
    expected.form.filer = original.form.filer.clone();
    assert_eq!(education_value(&page).students, vec![expected]);
}

// M3-P13: the federal payments page.

// The page as a new return opens it: no payments, the IRS figures any
// penalty. The withholding shown is derived and ignored on save.
pub(in crate::tests) fn no_payments() -> Payments {
    Payments {
        more: crate::PaymentsMore {
            extension_payer: None,
            extension_refund_property: None,
            penalty: PenaltyChoice::IrsFigures,
            next_year: NextYearEstimate::NotNeeded,
        },
        withheld: Some(Withheld { w2: 0, forms_1099: 0, other: 0, total: 0 }),
        estimated: crate::EstimatedPaymentEntry::None,
        extension: 0,
    }
}
// Finish on the review with these refund choices.
pub(in crate::tests) fn finish_request(d: &FederalDraft, refund: RefundChoice) -> P {
    P::Finish {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: TaxSection::Review as i32,
        previous: TaxSection::Review as i32,
        review: true,
        progress: vec![],
        refund,
    }
}
// No refund applied to 2027.
pub(in crate::tests) fn whole_refund() -> RefundChoice {
    RefundChoice { credit_applied_next_year: 0, credit_recipient: None, credit_capacity: None }
}
// The refund choices the review's finish step opens with.
pub(in crate::tests) fn begin_finish(s: &mut Interview) -> RefundChoice {
    let request = P::StartFinish {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
    };
    let Reply::Response(P::BeginFinish { refund, .. }) = send(s, request) else {
        panic!("expected the finish step")
    };
    refund
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
    match crate::tests::support::client::answer_no_estimates(reply) {
        Reply::Edit(page @ P::SavePayments { .. }) => {
            // The last page before the review is passed with its answers kept.
            let reply = send(s, page);
            past_less_common(s, reply)
        }
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
        estimate: None,
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
// A saved return at its payments page, its box checked: every earlier
// section is finished, so opening the interview resumes there.
pub(in crate::tests) fn at_the_payments_page(mut d: FederalDraft) -> (Interview, P) {
    let accepted_plan =
        d.payments.as_ref().map(|pay| NextYearEstimate::from(pay.next_year.clone()));
    d.finished_sections.retain(|s| *s != TaxSection::Payments);
    d.checklist.paid |= PaidTopics::PAYMENTS.0;
    let mut s = Interview::new(TEST_USER, d);
    crate::tests::support::client::admit_native_fixture_facts(&mut s);
    let sources = open_document_center(&mut s);
    accepted(&send(&mut s, sources));
    let Reply::Edit(page @ P::SavePayments { .. }) =
        crate::tests::support::client::answer_payment_method(
            send(&mut s, P::OpenInterview { user: TEST_USER }),
            accepted_plan,
        )
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
    let Reply::Edit(page @ P::SavePayments { .. }) =
        crate::tests::support::client::answer_no_estimates(send(s, request))
    else {
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
        more: crate::PaymentsMore {
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
            ..no_payments().more
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
        let mut value = figure(this_return(), withholding, income);
        value.more.next_year = next_year.clone();
        rejected_with(send(&mut s, save_payments(&before, value)), "FARMER_EVEN_TIMING");
        assert_eq!(s.draft, before);
    }
    let mut value = figure(this_return(), WithholdingTiming::Evenly, IncomeTiming::Evenly);
    value.more.next_year = next_year;
    accepted(&submit(&mut s, value));
}

// M3-P15: every situation finds its hub.

// Opens a section through the review's Go to.
pub(in crate::tests) fn go_to_from_review(s: &mut Interview, section: TaxSection) -> Reply {
    let request = navigate(&s.draft, section);
    send(s, request)
}

#[test]
fn every_refused_kind_is_a_value_of_the_one_unprepared_type() {
    // No situation carries data: each encodes as its declared numeric value.
    for situation in RareSituation::ALL {
        assert_eq!(situation.to_avro().len(), 1, "{situation:?}");
    }
    // Every refused kind round-trips as a page item and names itself.
    for item in Unprepared::all() {
        assert_eq!(Unprepared::from_avro(&item.to_avro()).unwrap(), item, "{item:?}");
        assert!(
            item.refused().name_key.starts_with("SITUATION_NAME_")
                || item.refused().name_key.starts_with("NAME_RARE_")
        );
    }
}

#[test]
fn the_often_missed_page_saves_the_unprepared_situations_in_the_order_chosen() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // An answer of the same page, saved first, is kept.
    accepted(&save_often_missed(
        &mut s,
        OftenMissed { benefits: MissedBenefits::EDUCATOR.0, ..Default::default() },
    ));
    accepted(&save_unprepared(
        &mut s,
        vec![Unprepared::Section965, RareSituation::TaxTreaty.into(), Unprepared::OtherAdjustments],
    ));
    assert_eq!(
        s.draft.often_missed.unprepared,
        vec![Unprepared::Section965, RareSituation::TaxTreaty.into(), Unprepared::OtherAdjustments]
    );
    assert_eq!(s.draft.often_missed.benefits, MissedBenefits::EDUCATOR.0);
    // The review names each as not calculated, opening this page.
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    // A situation listed twice is refused where it was listed.
    let before = s.draft.clone();
    let mut page = open_often_missed(&mut s);
    let P::SaveOftenMissed { value, .. } = &mut page else { unreachable!() };
    value.unprepared = Some(vec![RareSituation::RemicResidual.into(); 2]);
    current_source_refusal(
        "secondary-situation-duplicate",
        &mut s,
        page,
        "MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE",
        Some("[value/unprepared/1]"),
        SourceScope::Secondary(SecondaryCorrection::SituationChoice),
    );
    assert_eq!(s.draft, before);
    // Clearing the list clears only the situations.
    accepted(&save_unprepared(&mut s, vec![]));
    assert!(s.draft.often_missed.unprepared.is_empty());
    assert_eq!(s.draft.often_missed.benefits, MissedBenefits::EDUCATOR.0);
}

// A Form 3921 (T2-iso): box 3 $10.00, box 4 $210.00, box 5 1,000 shares.
pub(in crate::tests) fn form_3921(more: Form3921More) -> Form3921Paper {
    iso_document(IsoExerciseSource {
        id: 0,
        assigned: crate::DocumentRecipient::Unknown,
        owner_source: None,
        joint_share: None,
        owner: Owner::Taxpayer,
        more: Some(more),
        form: Form3921Paper {
            transferor: crate::PrintedParty::from_name("Synthetic Employer Inc".into()),
            employee: Default::default(),
            granted: 20240315,
            exercised: 20260310,
            exercise_price: 1_000,
            market_price: 21_000,
            shares: 1000.0,
            corporation: crate::PrintedParty::from_name("".into()),
            ..Default::default()
        },
    })
}

#[test]
fn final_return_page_maps_the_stored_final_return_both_ways() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut you = filer(&s.draft.people.as_ref().unwrap().taxpayer);
    assert_eq!(you.details.more, MoreAboutYou::Nothing);
    you.details.more = MoreAboutYou::Died {
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
    assert!(s.draft.often_missed.unprepared.is_empty());
    // The return computes, with the notice that the death adjustments are left out.
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT"));
    // It reopens on the page, and "Nothing else" clears it.
    assert_eq!(
        filer(&restart(s.draft.clone()).draft.people.unwrap().taxpayer).details.more,
        you.details.more
    );
    you.details.more = MoreAboutYou::Nothing;
    let request = save_you(&s.draft.clone(), you);
    accepted(&send(&mut s, request));
    assert_eq!(supported(&s.draft).tax, supported(&complete(FederalStatus::Single)).tax);
}

#[test]
fn death_date_rules_are_checked_on_save_with_the_client_keys() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let birth = s.draft.people.as_ref().unwrap().taxpayer.birth_date;
    for (date, key) in
        [(20251231, "DEATH_DATE_OUTSIDE_2026"), (20270101, "DEATH_DATE_OUTSIDE_2026")]
    {
        let mut you = filer(&s.draft.people.as_ref().unwrap().taxpayer);
        you.details.more = died_more(date);
        let request = save_you(&s.draft.clone(), you);
        rejected_with(send(&mut s, request), key);
    }
    let mut you = filer(&s.draft.people.as_ref().unwrap().taxpayer);
    you.birth_date = 20260615;
    you.details.more = died_more(20260614);
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
fn refusals_name_the_page_they_are_chosen_on() {
    let d = complete(FederalStatus::Single);
    for situation in super::every_refusal_item() {
        let name = situation.refused().lines;
        let mut refused = d.clone();
        situation.choose(&mut refused);
        let r = estimate(&refused);
        // The issue names the one page every refused situation is chosen on.
        assert!(
            r.issues
                .iter()
                .any(|i| i.section == section_name(TaxSection::OftenMissed)
                    && i.message.contains(name)),
            "{situation:?}: {r:?}"
        );
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
        document_questions: Vec::new(),
        credit_income_sources: crate::tax_credit_bonds::income_sources(&d),
        allowed_credit_owners: vec![Owner::Taxpayer as i32],
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
        estate_foreign_sources: vec![],
        capital_sales: vec![],
        foreign_situations: vec![],
        mortgage_loans: vec![],
        document_facts: DocumentFacts { year: 2026 },
        value: MoreCredits {
            answers: Vec::new(),
            credits: value.credits.into_iter().chain([MoreCredit::OtherCredit]).collect(),

            documents: Vec::new(),
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
    // M1 walks its 1099-INT's topic once (G-47); M2 keeps its care-only path.
    let m2 = include_str!("../../../tests/fixtures/returns/m2-hoh-two-children.json");
    let fixture: serde_json::Value = serde_json::from_str(m2).unwrap();
    assert_eq!(fixture["fields_touched"], 20);
    for (name, raw, taps) in [
        // Each topic holding a paper asks its situations once (wages; interest on M1).
        ("m1-single-w2", include_str!("../../../tests/fixtures/returns/m1-single-w2.json"), 9),
        ("m2-hoh-two-children", m2, 9),
    ] {
        let (mut s, result, sent) = enter_through_the_hubs(name, raw);
        assert_fixture_lines(raw, &result);
        assert_eq!(sent.len(), taps, "{name}");
        for request in &sent {
            match request {
                P::SavePeople { you, .. } => assert_eq!(you.details.more, MoreAboutYou::Nothing),
                P::SaveChildren { dependents, .. } => {
                    assert!(dependents.people.iter().all(|p| p.facts == PersonFacts::NONE.0))
                }
                P::SaveDeductions { value, .. } => assert!(value.car_loans.is_empty()),
                // Only the care paid is beyond the documents.
                P::SaveChecklist { value, .. } => {
                    assert_eq!(value.paid & !PaidTopics::CARE.0, 0);
                    assert_eq!((value.activities, value.carryovers), (0, 0));
                }
                P::SaveOftenMissed { value, .. } => {
                    assert_eq!(
                        *value,
                        crate::federal_model::OftenMissedAnswers::from(OftenMissed::default())
                    );
                }
                _ => {}
            }
        }
        assert!(s.draft.often_missed.unprepared.is_empty());
        assert!(s.draft.finished_sections.contains(&TaxSection::OftenMissed));
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
fn with_interest(mut documents: Vec<IncomeDocumentItem>) -> Interview {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    for row in &mut documents {
        let person = if row.owner == crate::DocumentOwner::Spouses { -2 } else { -1 };
        row.document = named_document(&s.draft, person, row.id, row.document.clone()).document;
    }
    crate::tests::support::client::income_pages::save_each_topic(&mut s, documents);
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
    accepted(&{
        let request = save_documents(
            &mut s,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w)) },
            )],
        );
        send(&mut s, request)
    });
    let items = items_of(&mut s);
    assert!(
        has_item(&items, "Child and dependent care", TaxSection::Care, "Coming up"),
        "{items:?}"
    );
    // An unsupported 1099-R code, stored on the retirement page.
    let mut s = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    accepted(&{
        crate::tests::support::client::save_retirement_fixture(
            &mut s,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r)),
                },
            )],
        )
    });
    let items = items_of(&mut s);
    assert!(
        has_item(&items, "Document center", TaxSection::Documents, "Form 1099-R code 5"),
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
    let mut s = at_the_deductions_page(FederalStatus::Single).0;
    let request = save_mortgages(
        &mut s,
        crate::MortgageDocuments {
            documents: vec![first_lender_mortgage(
                2_000_000,
                50_000_000,
                AverageBalance::WithinLimit,
            )]
            .into_iter()
            .map(mortgage_document)
            .collect(),
        },
    );
    accepted(&send(&mut s, request));
    assert!(!items_of(&mut s).iter().any(|(_, _, section)| *section == TaxSection::Documents));
    let mut d = s.draft.clone();
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Separate);
    // 2026-10-30: a separate return's community-property and spouse boxes
    // steps come first in the walk; this return answers them, so the mortgage
    // item is the one open.
    d.setup.as_mut().unwrap().community_property = CommunityProperty::No;
    d.setup.as_mut().unwrap().separate_spouse = crate::SeparateSpouseBoxes::NoneOfThese;
    answer_separate_pages(&mut d);
    let items = items_of(&mut Interview::new(TEST_USER, d));
    // 2026-10-29: accepted paper values stay in Document center; the
    // average-balance answer belongs to the mortgage topic.
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
    d.often_missed.unprepared = vec![RareSituation::TaxTreaty.into(), Unprepared::Section965];
    d.set_situations(YourSituations::DUAL_STATUS.0, true);
    let items = items_of(&mut Interview::new(TEST_USER, d));
    // Each situation once, named by the page that holds it, in sequence
    // order; the estimate's own copy of the refusal is not repeated. The
    // filer's whole-return items are About you's.
    assert_eq!(
        items,
        vec![
            (
                "About you".into(),
                "Part-year residency, dual-status and U.S. territory returns (Form 1040 line 12c, Form 1040-NR, Forms 4563 and 8689) is not calculated yet".into(),
                TaxSection::People
            ),
            (
                "Situations people often miss".into(),
                alloc::format!(
                    "{} is not calculated yet",
                    Unprepared::from(RareSituation::TaxTreaty).refused().lines
                ),
                TaxSection::OftenMissed
            ),
            (
                "Situations people often miss".into(),
                alloc::format!("{} is not calculated yet", Unprepared::Section965.refused().lines),
                TaxSection::OftenMissed
            ),
        ]
    );
}

#[test]
fn hub_attention_and_review_item_share_one_text() {
    // $2,000 of interest from a 1099-INT saved without a payer name.
    let mut s = with_interest(vec![interest(200_000)]);
    let (items, ..) = review_state(&mut s);
    let item = items.iter().find(|i| i.section == TaxSection::Documents).unwrap();
    assert_eq!(status_text(&item.item), "Document center");
    let request = go_to(&s.draft.clone(), PAGE_REVIEW, TaxSection::Income);
    let Reply::Response(hub @ P::Hub { .. }) = send(&mut s, request) else {
        panic!("expected the income hub")
    };
    assert!(hub_rows(&hub).contains(&row_of(
        "Interest and dividends",
        "1 1099-INT · $2000.00",
        TaxSection::InterestDividends
    )));
    assert!(
        !hub_rows(&hub)
            .iter()
            .any(|(_, status, section)| *section == TaxSection::InterestDividends
                && status.starts_with("Needs attention"))
    );
    let request = go_to(&s.draft, PAGE_REVIEW, item.section);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveDocuments { .. })));
    // The adjustments hub's HSA row and its review item, too.
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 100_000);
    let hub = open_adjustments_hub(&mut s);
    let (items, ..) = review_state(&mut s);
    // The review lists the account page as ahead; the hub row keeps the issue.
    let item = items.iter().find(|i| i.section == TaxSection::Hsa).unwrap();
    assert_eq!(status_text(&item.status), "Coming up");
    assert!(hub_rows(&hub)[0].1.starts_with("Needs attention: "));
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
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "Synthetic Bank".into(),
                200_000,
            ))),
        },
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
        Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, review: false, .. })
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
        let source = open_document_center(&mut s);
        accepted(&send(&mut s, source));
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
    // An unsupported 1099-R code 5 and a whole-return situation: the source
    // correction opens Document center and is the first suggestion.
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    accepted(&{
        crate::tests::support::client::save_retirement_fixture(
            &mut s,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r)),
                },
            )],
        )
    });
    let mut d = s.draft.clone();
    d.often_missed.unprepared = vec![Unprepared::Section965];
    let mut s = Interview::new(TEST_USER, d);
    let (items, suggested, ..) = review_state(&mut s);
    assert_eq!(items[0].section, TaxSection::Documents);
    assert_eq!(items.last().unwrap().section, TaxSection::OftenMissed);
    assert_eq!(suggested, TaxSection::Documents);
    // Go to opens it: the suggestion is an allowed section.
    let request = go_to(&s.draft.clone(), PAGE_REVIEW, suggested);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveDocuments { .. })));
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
            section == TaxSection::Documents
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
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
                "Synthetic Bank".into(),
                150_100
            )))
        },
    )])));
    // Dividends follow their own $1,500 test: $1,501 of box 1a unnamed.
    let dividend =
        |ordinary| {
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(
                        native_div1099("".into(), ordinary, 0),
                    )),
                },
            )
        };
    assert!(!payer_item(&mut with_interest(vec![dividend(150_000)])));
    assert!(payer_item(&mut with_interest(vec![dividend(150_100)])));
}

// M3-P18: the opening checklist, "Beyond your documents".

// A new single return saved through your page: the reply is the checklist.
fn at_the_checklist() -> (Interview, Reply) {
    at_the_checklist_with_self_work(false)
}
fn at_the_checklist_with_self_work(self_work: bool) -> (Interview, Reply) {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m3-checklist".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let request = save_you(&s.draft, adult("Casey", "Sample", 19850315));
    let reply = send(&mut s, request);
    let reply = past_people_with_self_work(&mut s, reply, self_work);
    (s, reply)
}
pub(in crate::tests) fn checklist_of(reply: &Reply) -> Checklist {
    match reply {
        Reply::Edit(P::SaveChecklist { value, .. }) => Checklist {
            activities: value.activities,
            paid: value.paid,
            carryovers: value.carryovers,
        },
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
    d.finished_sections =
        vec![TaxSection::Children, TaxSection::IncomeWithoutForm, TaxSection::Checklist];
    d.checklist = planned;
    Interview::new(TEST_USER, d)
}

#[test]
fn checklist_follows_the_people_pages_and_opens_with_nothing_ticked() {
    let (s, reply) = at_the_checklist();
    // After the people page and Document center, the checklist opens
    // unanswered (BeginChecklist); Back returns to Document center.
    let Reply::Response(P::BeginChecklist { page, previous, review, progress, .. }) = &reply else {
        panic!("expected the checklist, got {reply:?}")
    };
    assert_eq!(
        (*page, *previous, *review),
        (PAGE_CHECKLIST, TaxSection::IncomeWithoutForm as i32, false)
    );
    assert!(shown(progress).contains("Your return: "), "{}", shown(progress));
    // Resume stops at the unanswered checklist; manual navigation can already
    // open income once the filers are known.
    let mut resumed = restart(s.draft.clone());
    assert!(matches!(
        send(&mut resumed, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::BeginChecklist { .. })
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
    walk_on_to_review(&mut s, reply);
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    assert_eq!(told_row(&hub).1, "None of these");
    // Amended for M3-N1: every topic and Continue are rows.
    assert_eq!(hub_rows(&hub), not_started_rows(&ALL_INCOME_TOPICS));
    assert_eq!(
        s.draft.finished_sections,
        vec![
            TaxSection::Children,
            TaxSection::Documents,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Adjustments,
            TaxSection::Deductions,
            TaxSection::Credits,
            TaxSection::OtherTaxes,
            TaxSection::Payments,
            TaxSection::OftenMissed,
        ]
    );
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    let (_, _, _, answers) = review_state(&mut s);
    let row = answers.iter().find(|r| r.section == TaxSection::Checklist).unwrap();
    assert_eq!(
        (status_text(&row.item).as_str(), status_text(&row.status).as_str()),
        ("Beyond your documents", "None of these")
    );
}

// A business record as the Self-employment page saves it.
// Saves the topic page a reply opened with these records.
fn save_opened_topic(s: &mut Interview, reply: &Reply, records: Vec<IncomeDocumentItem>) -> Reply {
    let Reply::Edit(P::SaveIncomes { topic, .. }) = reply else {
        panic!("expected an income topic page, got {reply:?}")
    };
    let request = save_topic(s, *topic, records);
    send(s, request)
}
fn business_document(receipts: i64) -> IncomeDocumentItem {
    document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(business(receipts)) })
}
#[test]
fn ticks_become_planned_topics() {
    let (mut s, reply) = at_the_checklist_with_self_work(true);
    let ticks = Checklist {
        activities: ActivityTopics::RENTALS.0,
        paid: PaidTopics::PAYMENTS.0,
        ..Checklist::NONE
    };
    // The walk opens the first planned topic; its Back opens the hub.
    let reply = send(&mut s, tick(reply, ticks));
    let hub = income_hub(to_the_hub(&mut s, reply));
    // Each box is stored as planned, exactly as checked.
    assert_eq!(crate::interview::planned(&s.draft), ticks);
    assert_eq!(told_row(&hub), ("What you told us".into(), "Rental property".into()));
    // The planned topics first, then every other topic.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![
                row_of("Self-employment", "Planned", TaxSection::SelfEmployment),
                row_of("Rental property", "Planned", TaxSection::Rentals),
            ],
            not_started_rows(&income_topics_but(&[
                TaxSection::SelfEmployment,
                TaxSection::Rentals
            ]))
        )
    );
    assert_eq!(suggestion(&hub), TaxSection::SelfEmployment);
    // Entering the business moves its box from planned to the entries.
    let request =
        save_topic(&mut s, TaxSection::SelfEmployment, vec![business_document(5_000_000)]);
    let reply = send(&mut s, request);
    let hub = income_hub(to_the_hub(&mut s, reply));
    assert_eq!(crate::interview::planned(&s.draft).activities, ActivityTopics::RENTALS.0);
    assert_eq!(crate::interview::planned(&s.draft).paid, PaidTopics::PAYMENTS.0);
    assert_eq!(suggestion(&hub), TaxSection::Rentals);
    // The "What you told us" row reopens the page from the hub, with the
    // business checked from its entry; unchecking the rental returns to the
    // hub, which now suggests continuing.
    let request = go_to(&s.draft, PAGE_INCOME, TaxSection::Checklist);
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Edit(P::SaveChecklist { previous: PAGE_INCOME, review: false, .. })
    ));
    assert_eq!(checklist_of(&reply), ticks);
    let fewer = Checklist { activities: 0, ..ticks };
    // A checklist opened from a hub returns there; it remains explicit navigation.
    let hub = income_hub(send(&mut s, tick(reply, fewer)));
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    assert_eq!(crate::interview::planned(&s.draft), Checklist { activities: 0, ..ticks });
    // A changed box erases nothing it does not own.
    assert_eq!(s.draft.income.len(), 1);
}

#[test]
fn topics_with_entries_open_ticked() {
    // A business, a household employee and an extension payment: each is
    // shown checked from its entry, and nothing is planned.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(2_000_000)));
    d.screening.as_mut().unwrap().household_employment = vec![household_employer(3_500)];
    d.payments.as_mut().unwrap().extension = 50_000;
    // As every save leaves it: each topic with entries checked.
    let d = crate::interview::with_ticks(d);
    let mut s = Interview::new(TEST_USER, d.clone());
    crate::tests::support::client::admit_native_fixture_facts(&mut s);
    let source = open_document_center(&mut s);
    accepted(&send(&mut s, source));
    let d = s.draft.clone();
    let reply = send(&mut s, navigate(&d, TaxSection::Checklist));
    assert!(matches!(reply, Reply::Edit(P::SaveChecklist { review: true, .. })));
    let opened = Checklist {
        activities: ActivityTopics::HOUSEHOLD_EMPLOYEE.0,
        paid: PaidTopics::PAYMENTS.0,
        ..Checklist::NONE
    };
    assert_eq!(checklist_of(&reply), opened);
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    // Saving it unchanged stores nothing twice: the review follows, unchanged.
    assert!(matches!(send(&mut s, tick(reply, opened)), Reply::Response(P::Review { .. })));
    assert_eq!(s.draft, d);
    let (_, _, _, answers) = review_state(&mut s);
    assert!(
        answers
            .iter()
            .any(|r| r.section == TaxSection::Checklist && status_text(&r.status) == "3 topics")
    );
}

// Self-employment documents keep their topic on the path without adding
// a business option or note to the remaining checklist.
#[test]
fn self_employment_evidence_stays_outside_the_remaining_checklist() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Nec1099 {
            data: nec_source(FederalForm1099Nec {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                overtime: 0,
                compensation: 120_000,
                withholding: 0,
                golden_parachute: 0,
                belongs: Some(IncomeUse::NotForProfit),
            }),
        }),
    );
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, navigate(&d, TaxSection::Checklist));
    let Reply::Edit(P::SaveChecklist { value, from_documents, .. }) = &reply else {
        panic!("expected Beyond your documents, got {reply:?}")
    };
    assert_eq!(*value, Checklist::NONE.into());
    assert!(from_documents.is_empty());
    let reply = from_review(&mut s, TaxSection::Checklist);
    accepted(&send(&mut s, tick(reply, Checklist::NONE)));
    assert_eq!(s.draft.checklist, Checklist::NONE);
    assert_eq!(s.draft.income, d.income);
    let reopened = from_review(&mut s, TaxSection::Checklist);
    assert_eq!(checklist_of(&reopened), Checklist::NONE);
    // Without the document, the box opens unchecked.
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.self_employment.documents.clear();
    accepted(&send(&mut s, request));
    assert_eq!(checklist_of(&from_review(&mut s, TaxSection::Checklist)), Checklist::NONE);
}

// Where each of the 40 options of the former opening checklist ("Your 2026
// at a glance") is answered now: a document brings its topic, reached from
// its hub; a box beyond the documents plans its page; an answer on
// "Situations people often miss" plans its page; a derived topic follows the
// facts; a whole-return situation is on About you or the spouse page.
#[derive(Clone, Debug)]
enum Home {
    Document(TaxSection),
    Beyond(Checklist, TaxSection),
    WithoutForm(TaxSection),
    Missed(OftenMissed, TaxSection),
    Derived(TaxSection),
    Filer(u32),
}
#[test]
fn every_option_of_the_old_checklist_has_its_home() {
    use ActivityTopics as A;
    use CarryoverTopics as C;
    use Home::*;
    use PaidTopics as Paid;
    use TaxSection as T;
    let boxes = |activities, paid, carryovers| Checklist { activities, paid, carryovers };
    let (activity, paid) = (|bit| boxes(bit, 0, 0), |bit| boxes(0, bit, 0));
    let missed = |bit: MissedIncome| OftenMissed { income: bit.0, ..OftenMissed::default() };
    let benefits = |bit: MissedBenefits| OftenMissed { benefits: bit.0, ..OftenMissed::default() };
    let options: [(&str, Home); 40] = [
        ("W-2 wages or employee pay reported on a 1099 (Form 8919)", Document(T::Wages)),
        ("Interest or dividends, a child's statements (Form 8814)", Document(T::InterestDividends)),
        ("Sales of investments or your home", Document(T::InvestmentSales)),
        ("Retirement plans, IRAs or Social Security", Document(T::Retirement)),
        ("Self-employment, gig work, 1099-K", WithoutForm(T::SelfEmployment)),
        ("Rental property", Beyond(activity(A::RENTALS.0), T::Rentals)),
        ("Unemployment or a state tax refund", Document(T::GovernmentBenefits)),
        ("Prizes, 1099-MISC items, 529 and Coverdell withdrawals", Document(T::OtherIncome)),
        ("Gambling winnings", Document(T::Gambling)),
        ("Farming or crop-share rent", Beyond(activity(A::FARM.0), T::Farming)),
        ("Schedule K-1", Document(T::PassThrough)),
        ("Canceled debt or a foreclosure", Document(T::CanceledDebt)),
        ("Work abroad", Beyond(activity(A::ABROAD.0), T::WorkAbroad)),
        ("HSA or Archer MSA", Beyond(paid(Paid::HSA.0), T::Hsa)),
        ("Health insurance or a SEP plan for your business", Derived(T::SelfEmployedBenefits)),
        ("IRA contributions", Beyond(paid(Paid::IRA_CONTRIBUTIONS.0), T::IraContributions)),
        ("K-12 educator expenses", Missed(benefits(MissedBenefits::EDUCATOR), T::Educator)),
        ("Student loan interest", Document(T::StudentLoan)),
        (
            "Reservist, performing artist or fee-basis official",
            Missed(benefits(MissedBenefits::EMPLOYEE_EXPENSES), T::EmployeeExpenses),
        ),
        ("Medical and dental expenses", Missed(benefits(MissedBenefits::MEDICAL), T::Medical)),
        ("State, local or property taxes", Beyond(paid(Paid::TAXES.0), T::Taxes)),
        ("Mortgage interest", Document(T::Mortgage)),
        ("Gifts to charity", Beyond(paid(Paid::GIFTS.0), T::Deductions)),
        (
            "Gifts of property, charity carried over",
            Beyond(boxes(0, Paid::GIFTS.0, C::CHARITY.0), T::Deductions),
        ),
        (
            "Investment interest",
            Missed(benefits(MissedBenefits::INVESTMENT_INTEREST), T::OtherItemized),
        ),
        ("Interest on a new personal vehicle loan", Document(T::Deductions)),
        ("College tuition", Document(T::Education)),
        ("Marketplace health insurance", Missed(missed(MissedIncome::MARKETPLACE), T::Marketplace)),
        ("A household employee", Beyond(activity(A::HOUSEHOLD_EMPLOYEE.0), T::OtherTaxes)),
        ("Incentive stock options", Document(T::InvestmentSales)),
        (
            "Excess contributions and account withdrawals",
            Missed(missed(MissedIncome::ACCOUNT_TAXES), T::OtherTaxes),
        ),
        ("Estimated tax payments", Beyond(paid(Paid::PAYMENTS.0), T::Payments)),
        ("A payment with an extension request", Beyond(paid(Paid::PAYMENTS.0), T::Payments)),
        ("Dual-status or U.S. territory return", Filer(YourSituations::DUAL_STATUS.0)),
        ("Community property with a domestic partner", Filer(YourSituations::COMMUNITY_PROPERTY.0)),
        ("Nonresident spouse treated as a resident", Filer(SpouseSituations::NONRESIDENT_SPOUSE.0)),
        ("Injured spouse allocation", Filer(SpouseSituations::INJURED_SPOUSE.0)),
        ("Innocent spouse relief", Filer(SpouseSituations::INNOCENT_SPOUSE.0)),
        (
            "Joint election for the net investment income tax",
            Filer(SpouseSituations::NONRESIDENT_SPOUSE.0 | SpouseSituations::NIIT_JOINT_ELECTION.0),
        ),
        (
            "You were the nonresident spouse",
            Filer(
                SpouseSituations::NONRESIDENT_SPOUSE.0
                    | SpouseSituations::YOU_WERE_THE_NONRESIDENT.0,
            ),
        ),
    ];
    let names: alloc::collections::BTreeSet<_> = options.iter().map(|(name, _)| *name).collect();
    assert_eq!(names.len(), 40);
    // A joint return that itemizes reaches every topic page.
    let mut itemizing = complete(FederalStatus::Joint);
    {
        itemizing.deductions.as_mut().unwrap().itemized = empty_facts();
        itemizing.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    for (name, home) in options {
        match home {
            Document(page) => {
                let mut s = Interview::new(TEST_USER, itemizing.clone());
                let reply = from_review(&mut s, page);
                assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{name}: {reply:?}");
            }
            Missed(answer, page) => {
                let mut d = itemizing.clone();
                d.often_missed = answer;
                assert!(crate::interview::planned_sections(&d).contains(&page), "{name}");
            }
            WithoutForm(page) => {
                let mut s = Interview::new(TEST_USER, itemizing.clone());
                accepted(&save_without_form(
                    &mut s,
                    crate::IncomeWithoutFormAnswers { self_work: true, ..Default::default() },
                ));
                assert!(crate::interview::planned_sections(&s.draft).contains(&page), "{name}");
            }
            Beyond(value, page) => {
                let mut d = complete(FederalStatus::Single);
                d.checklist = value;
                assert!(crate::interview::planned_sections(&d).contains(&page), "{name}");
            }
            Derived(page) => {
                let mut d = complete(FederalStatus::Single);
                add(&mut d, income(cash_business(2_000_000)));
                assert!(crate::interview::derived_topics(&d, T::Adjustments).contains(&page));
            }
            Filer(bits) => {
                let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
                accepted(&save_whole_return(&mut s, bits));
                assert_eq!(s.draft.whole_return(), bits, "{name}");
            }
        }
    }
}

#[test]
fn undeclared_checklist_ticks_are_refused() {
    // Unchecking a box with entries removes them (walk-owned data,
    // src/tests/cross/change_audit.rs); a box the questions do not declare is refused.
    let d = crate::interview::with_ticks(complete(FederalStatus::Single));
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, navigate(&d, TaxSection::Checklist));
    let undeclared = activities(ActivityTopics::BUSINESS.0 | 1 << 20);
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
    // planned; the W-2 shows its box checked.
    let mut s = at_the_adjustments_hub(FederalStatus::Single, 100_000);
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(suggestion(&hub), TaxSection::Hsa);
    assert_eq!(told_row(&hub).1, "Health and medical savings accounts");
    // A full-time student can enter education documents in Document center;
    // the fixture has already answered its less-common credit facts.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    let (mut s, hub) = at_the_credits_hub(d);
    assert_eq!(suggestion(&hub), TaxSection::OftenMissed);
    let source = open_document_center(&mut s);
    assert!(education_value(&source).people.iter().any(|person| person.id == -1));
    assert_eq!(told_row(&hub).1, "None of these");
    // Without a box or a derived need, an empty topic is never suggested:
    // Continue goes to the next shown section, the last page before the
    // review (federal payments only with its box).
    let (_, hub) = at_the_credits_hub(complete(FederalStatus::Single));
    assert_eq!(suggestion(&hub), TaxSection::OftenMissed);
}

#[test]
fn hub_suggests_attention_then_planned_then_continue() {
    // Self-employment is planned. The 1099-R needs correction in Document
    // center; the income hub keeps its planned topic suggestion.
    let mut s = planning(activities(ActivityTopics::BUSINESS.0));
    let r = other_1099(1_000_000, "5", false);
    let reply = crate::tests::support::client::save_retirement_fixture(
        &mut s,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(r)),
            },
        )],
    );
    let hub = income_hub(to_the_hub(&mut s, reply));
    assert_eq!(suggestion(&hub), TaxSection::SelfEmployment);
    let rows = hub_rows(&hub);
    assert!(rows.contains(&row_of("Self-employment", "Planned", TaxSection::SelfEmployment)));
    assert!(rows.contains(&row_of(
        "Retirement and Social Security",
        "1 1099-R · $10000.00",
        TaxSection::Retirement
    )));
    // Return to the summary while leaving the business plan unchanged.
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    // Review diagnoses the source after the mandatory sections are accepted.
    let reply = send(&mut s, hub_action(&hub));
    walk_on_to_review(&mut s, reply);
    let (items, ..) = review_state(&mut s);
    assert!(
        items.iter().any(|item| item.section == TaxSection::Documents),
        "review={items:?}; issues={:?}",
        estimate(&s.draft).issues
    );
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::Documents);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveDocuments { .. })));
    // Without the attention, the planned topic.
    let mut request = open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.retirement.documents.clear();
    accepted(&send(&mut s, request));
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    assert_eq!(suggestion(&hub), TaxSection::SelfEmployment);
    // Entering the business completes its plan; the completed income walk
    // suggests continuing to the adjustments section.
    let request =
        save_topic(&mut s, TaxSection::SelfEmployment, vec![business_document(5_000_000)]);
    let reply = send(&mut s, request);
    accepted(&reply);
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
}

#[test]
fn itemizable_ticks_open_deductions_on_compare() {
    let opened = |planned: Checklist| {
        let mut d = complete(FederalStatus::Single);
        d.finished_sections = vec![
            TaxSection::Children,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Income,
            TaxSection::Adjustments,
        ];
        d.checklist = planned;
        let mut s = Interview::new(TEST_USER, d);
        let sources = open_document_center(&mut s);
        accepted(&send(&mut s, sources));
        let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::Deductions);
        match send(&mut s, request) {
            Reply::Edit(page @ P::SaveDeductions { .. }) => (s, page),
            other => panic!("expected the deductions page, got {other:?}"),
        }
    };
    let route = |page: &P| deductions_value(page).itemize_override;
    // Gifts to charity only: the standard deduction, M1's default (cash
    // gifts are deductible without itemizing).
    let (_, page) = opened(Checklist { paid: PaidTopics::GIFTS.0, ..Checklist::NONE });
    assert_eq!(route(&page), None);
    // An itemizable box answers the route: Compare.
    let taxes = Checklist { paid: PaidTopics::TAXES.0, ..Checklist::NONE };
    let (mut s, page) = opened(taxes);
    assert_eq!(route(&page), None);
    // The planned taxes are no item needing review.
    assert!(items_of(&mut s).iter().all(|(_, _, section)| *section == TaxSection::Deductions));
    // Once the page is saved its stored route is shown, even Standard.
    let request = save_deductions(&s.draft, None, 0);
    accepted(&send(&mut s, request));
    let request = navigate(&s.draft, TaxSection::Deductions);
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the deductions page") };
    assert_eq!(route(&page), None);
    // Saved as it opened, Compare enters the itemized hub, whose walk opens
    // the planned taxes page directly (M3-P19); its Back opens the hub.
    let (mut s, page) = opened(taxes);
    let reply = send(&mut s, page);
    assert!(matches!(reply, Reply::Edit(P::SaveTaxes { .. })), "{reply:?}");
    let hub = itemized_hub(to_the_hub(&mut s, reply));
    assert_eq!(suggestion(&hub), TaxSection::Taxes);
    assert_eq!(
        told_row(&hub),
        ("What you told us".into(), "State, local or property taxes".into())
    );
    // Only the review's finishing row remains.
    let listed = items_of(&mut s);
    assert_eq!(listed.len(), 1, "{listed:?}");
}

#[test]
fn checklist_estimate_lists_the_planned_sections() {
    let (mut s, _) = at_the_checklist();
    s.dirty = false; // persisted by the task
    let before = s.draft.clone();
    let path = |s: &mut Interview, value: Checklist| match send(
        s,
        P::EstimateChecklist { user: TEST_USER, value: value.into() },
    ) {
        Reply::Response(P::RunningEstimate { progress, .. }) => {
            let text = shown(&progress);
            let start = text.find("Your return: ").unwrap();
            alloc::string::String::from(text[start..].trim_end_matches('\0'))
        }
        other => panic!("expected the running estimate, got {other:?}"),
    };
    // Income, adjustments, credits and the last page before the review are
    // always shown; the payments page only with its box.
    assert_eq!(
        path(&mut s, Checklist::NONE),
        "Your return: Income, Adjustments to income, Credits, Situations people often miss, Review"
    );
    assert_eq!(
        path(
            &mut s,
            Checklist { paid: PaidTopics::TAXES.0 | PaidTopics::PAYMENTS.0, ..Checklist::NONE }
        ),
        "Your return: Income, Adjustments to income, Deductions, Credits, Federal payments, Situations people often miss, Review"
    );
    // A carryover box opens its own page; charity carried over the
    // deductions.
    assert_eq!(
        path(
            &mut s,
            Checklist {
                paid: PaidTopics::IRA_CONTRIBUTIONS.0,
                carryovers: CarryoverTopics::MINIMUM_TAX_CREDIT.0 | CarryoverTopics::CHARITY.0,
                ..Checklist::NONE
            }
        ),
        "Your return: Income, Adjustments to income, Deductions, Credits, Situations people often miss, Review"
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
    d.checklist = activities(ActivityTopics::BUSINESS.0);
    let mut s = Interview::new(TEST_USER, d.clone());
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
    let hub = hub_of(from_review(&mut s, TaxSection::Income));
    assert!(hub_rows(&hub).contains(&row_of(
        "Self-employment",
        "Planned",
        TaxSection::SelfEmployment
    )));
    // Entering it: the row shows the entry and the box stays checked.
    let request =
        save_topic(&mut s, TaxSection::SelfEmployment, vec![business_document(5_000_000)]);
    accepted(&send(&mut s, request));
    assert_eq!(crate::interview::planned(&s.draft), Checklist::NONE);
    assert_ne!(s.draft.checklist.activities & ActivityTopics::BUSINESS.0, 0);
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
            P::SaveIncomeWithoutForm { .. } => "income without a form",
            P::SaveChecklist { .. } => "checklist",
            P::SaveDocuments { .. } => "documents",
            P::SaveIncomes { topic: TaxSection::Wages, .. } | P::SaveWagesAnswers { .. } => "wages",
            P::SaveIncomes { topic: TaxSection::InterestDividends, .. } => "interest",
            P::ContinueIncome { .. } => "income summary Continue",
            P::SaveMoreAdjustments { .. } => "more adjustments",
            P::ContinueAdjustments { .. } => "adjustments summary Continue",
            P::SaveCare { .. } => "care",
            P::SaveMoreCredits { .. } => "more credits",
            P::ContinueCredits { .. } => "credits summary Continue",
            P::SaveOtherTaxes { .. } => "other taxes",
            P::SavePayments { .. } => "payments",
            P::SaveOftenMissed { .. } => "often missed",
            P::Navigate { .. } => "hub row",
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
        Reply::Edit(
            P::SaveIncomes { progress, .. }
            | P::SaveDocuments { progress, .. }
            | P::SaveSelfEmployedBenefits { progress, .. },
        ) => shown(progress.as_ref().expect("native supplied progress")),
        Reply::Response(
            P::Hub { progress, .. }
            | P::BeginChildren { progress, .. }
            | P::BeginPayments { progress, .. }
            | P::BeginIncomeWithoutForm { progress, .. }
            | P::BeginChecklist { progress, .. }
            | P::BeginCare { progress, .. }
            | P::BeginOftenMissed { progress, .. },
        )
        | Reply::Edit(
            P::SavePayments { progress, .. }
            | P::SaveHsa { progress, .. }
            | P::SaveEducation { progress, .. }
            | P::SaveIncomeWithoutForm { progress, .. }
            | P::SaveChecklist { progress, .. },
        ) => shown(progress),
        other => panic!("unexpected page {other:?}"),
    }
}
// A new single return at the checklist, ticked with `ticks`: the reply to
// its Next.
fn walked_from_the_checklist(ticks: Checklist) -> (Interview, Reply) {
    let (mut s, reply) =
        at_the_checklist_with_self_work(ticks.activities & ActivityTopics::BUSINESS.0 != 0);
    let reply = send(&mut s, tick(reply, ticks));
    (s, reply)
}

#[test]
fn m1_path_is_checklist_wages_interest_summary_review() {
    // The actual source editors and required questions lead directly to Review.
    let raw = include_str!("../../../tests/fixtures/returns/m1-single-w2.json");
    let (mut s, result, sent) = enter_through_the_hubs("m1-single-w2", raw);
    assert_fixture_lines(raw, &result);
    assert_eq!(
        path_of(&sent),
        [
            "setup",
            "you",
            "people",
            "documents",
            "income without a form",
            "checklist",
            "wages",
            "interest",
            "often missed"
        ]
    );
    assert!(!sent.iter().any(|p| matches!(
        p,
        P::ContinueIncome { .. } | P::ContinueAdjustments { .. } | P::ContinueCredits { .. }
    )));
    assert!(!sent.iter().any(|p| matches!(p, P::Navigate { .. })));
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
}

#[test]
fn m2_path_completes_the_care_editor_before_credits_summary() {
    // The actual source editors and required questions lead directly to Review.
    let raw = include_str!("../../../tests/fixtures/returns/m2-hoh-two-children.json");
    let (mut s, result, sent) = enter_through_the_hubs("m2-hoh-two-children", raw);
    assert_fixture_lines(raw, &result);
    assert_eq!(
        path_of(&sent),
        [
            "setup",
            "you",
            "people",
            "documents",
            "income without a form",
            "checklist",
            "wages",
            "care",
            "often missed"
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
    assert_eq!(suggestion(&hub), TaxSection::OftenMissed);
}

#[test]
fn walk_follows_planned_topics_in_hub_order() {
    // Checked in any order, the planned income topics open in the hub's
    // order: Self-employment, Farming, then Work abroad; the unplanned topics
    // between them are passed.
    let (mut s, mut reply) = walked_from_the_checklist(activities(
        ActivityTopics::ABROAD.0 | ActivityTopics::FARM.0 | ActivityTopics::BUSINESS.0,
    ));
    let mut walked = Vec::new();
    loop {
        match &reply {
            Reply::Edit(P::SaveIncomes { topic, .. }) => {
                walked.push(*topic);
                // A business on Self-employment; the other two are saved
                // with nothing.
                let documents = if *topic == TaxSection::SelfEmployment {
                    vec![business_document(5_000_000)]
                } else {
                    vec![]
                };
                reply = if documents.is_empty() {
                    send(
                        &mut s,
                        match reply {
                            Reply::Edit(page @ P::SaveIncomes { .. }) => page,
                            _ => unreachable!(),
                        },
                    )
                } else {
                    save_opened_topic(&mut s, &reply, documents)
                };
            }
            _ => {
                assert!(matches!(reply, Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
                break;
            }
        }
    }
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    assert_eq!(walked, [TaxSection::SelfEmployment, TaxSection::Farming, TaxSection::WorkAbroad]);
    // The walk ends, after Less common income, at the hub, shown as the
    // section summary: it continues to the next shown section, adjustments
    // (gate fixes batch A).
    let P::Hub { progress, .. } = &hub else { unreachable!() };
    assert!(shown(progress).contains("· You're done with income"), "{}", shown(progress));
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    // A planned topic saved with nothing is visited: it stays planned (a
    // remaining item), listed "Not started", and the walk passes it.
    assert_eq!(
        crate::interview::planned(&s.draft).activities,
        ActivityTopics::FARM.0 | ActivityTopics::ABROAD.0
    );
    assert!(s.draft.finished_sections.contains(&TaxSection::Farming));
    assert!(s.draft.finished_sections.contains(&TaxSection::WorkAbroad));
    assert!(hub_rows(&hub).contains(&row_of("Farming", "Planned", TaxSection::Farming)));
    let mut resumed = restart(s.draft.clone());
    assert!(matches!(
        send(&mut resumed, P::OpenInterview { user: TEST_USER }),
        Reply::Edit(P::SaveSelfEmployedBenefits { .. })
    ));
}

#[test]
fn empty_sections_are_skipped_and_listed_on_the_review() {
    let raw = include_str!("../../../tests/fixtures/returns/m1-single-w2.json");
    let (mut s, _, sent) = enter_through_the_hubs("m1-single-w2", raw);
    // Deductions with nothing is skipped, and so is Federal payments without
    // its box.
    assert!(!sent.iter().any(|p| matches!(p, P::SaveDeductions { .. } | P::SavePayments { .. })));
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
    assert_eq!(status(TaxSection::Deductions), "Compare with itemized deductions");
    // Each row opens its page from the review.
    let hub = hub_of(from_review(&mut s, TaxSection::Adjustments));
    assert!(matches!(hub, P::Hub { page: PAGE_ADJUSTMENTS, review: true, .. }));
    assert!(matches!(
        from_review(&mut s, TaxSection::Payments),
        Reply::Edit(P::SavePayments { .. })
    ));
}

#[test]
fn completed_topic_attention_remains_available_after_automatic_continuation() {
    // The HSA page was visited and saved with no account, so the HSA is no
    // longer a derived topic; a W-2 with box 12 W and no account reopens it.
    let after_wages = |w: i64| {
        let mut d = complete(FederalStatus::Single);
        d.finished_sections = vec![
            TaxSection::Children,
            TaxSection::IncomeWithoutForm,
            TaxSection::Checklist,
            TaxSection::Hsa,
            TaxSection::MoreAdjustments,
            TaxSection::Income,
        ];
        let mut s = Interview::new(TEST_USER, d);
        let wage = if w == 0 { w2(6_000_000) } else { w2_with_box_12_w(6_000_000, w) };
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut s,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wage)) },
            )],
        );
        accepted(&send(&mut s, request));
        s
    };
    // The W-2 amount's contribution year and its owner's account are asked on
    // the HSA page, which refuses to save them open: the walk returns to it
    // instead of leaving a Review item.
    let mut s = after_wages(100_000);
    assert!(!s.draft.finished_sections.contains(&TaxSection::Adjustments));
    // The hub still names the topic that needs attention.
    let hub = hub_of(from_review(&mut s, TaxSection::Adjustments));
    assert!(matches!(hub, P::Hub { page: PAGE_ADJUSTMENTS, .. }));
    assert_eq!(suggestion(&hub), TaxSection::Hsa);
    assert!(hub_rows(&hub).iter().any(|(_, status, section)| {
        *section == TaxSection::Hsa && status.starts_with("Needs attention")
    }));
    // Without box 12 W the section is still shown, as every adjustments
    // section asks More adjustments (gate fixes batch A), with nothing
    // needing attention.
    let mut s = after_wages(0);
    assert!(s.draft.finished_sections.contains(&TaxSection::Adjustments));
    let hub = hub_of(from_review(&mut s, TaxSection::Adjustments));
    assert!(hub_rows(&hub).iter().all(|(_, status, _)| !status.starts_with("Needs attention")));
}

#[test]
fn revisiting_a_completed_hub_keeps_navigation_and_unchanged_saves() {
    // The walk has ended (Self-employment, the one planned topic, is saved): a topic
    // opened from the summary, planned or not, returns to the summary.
    let (mut s, reply) = walked_from_the_checklist(activities(ActivityTopics::BUSINESS.0));
    let reply = save_opened_topic(&mut s, &reply, vec![business_document(5_000_000)]);
    assert!(matches!(reply, Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    // An unplanned topic, with a document.
    let request = go_to(&s.draft, PAGE_INCOME, TaxSection::InterestDividends);
    let _page = send(&mut s, request);
    let reply = save_walk_sources(&mut s, vec![interest(20_000)]);
    accepted(&reply);
    let page = open_topic(&mut s, TaxSection::InterestDividends);
    accepted(&send(&mut s, page));
    let hub_after = income_hub(from_review(&mut s, TaxSection::Income));
    // Amended for M3-N1: then every other topic; Continue is a separate action.
    assert_eq!(
        hub_rows(&hub_after),
        with_tail(
            vec![
                row_of(
                    "Interest and dividends",
                    "1 1099-INT · $200.00",
                    TaxSection::InterestDividends
                ),
                row_of("Self-employment", "1 business", TaxSection::SelfEmployment),
            ],
            not_started_rows(&income_topics_but(&[
                TaxSection::InterestDividends,
                TaxSection::SelfEmployment
            ]))
        )
    );
    assert_eq!(suggestion(&hub_after), suggestion(&hub));
    // The planned topic again, saved unchanged: the hub, and no revision.
    let revision = s.draft.revision;
    let request = go_to(&s.draft, PAGE_INCOME, TaxSection::SelfEmployment);
    let Reply::Edit(page) = send(&mut s, request) else {
        panic!("expected the Self-employment page")
    };
    assert!(matches!(send(&mut s, page), Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    assert_eq!(s.draft.revision, revision);
}

// A new single return that plans W-2 wages and estimated payments, at the
// Wages page the walk opened.
fn business_and_estimated_payments() -> (Interview, Reply) {
    walked_from_the_checklist(Checklist {
        activities: ActivityTopics::BUSINESS.0,
        paid: PaidTopics::PAYMENTS.0,
        ..Checklist::NONE
    })
}

#[test]
fn progress_counts_only_shown_sections() {
    // Before any box: setup, you, people, the checklist, the income,
    // adjustments and credits sections, the last page before the review and
    // the review.
    let (_, reply) = at_the_checklist();
    assert!(progress_of(&reply).contains(
        "Your return: Income, Adjustments to income, Credits, Situations people often miss, Review"
    ));
    // Planned: federal payments joins the count, 10 sections.
    let (mut s, reply) = business_and_estimated_payments();
    assert!(
        progress_of(&reply).contains("2026 federal · section 7 of 12 · Self-employment"),
        "{}",
        progress_of(&reply)
    );
    let reply = save_opened_topic(&mut s, &reply, vec![business_document(5_000_000)]);
    assert!(matches!(reply, Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    assert!(
        progress_of(&reply)
            .contains("section 8 of 12 · Self-employed health insurance and retirement plan")
    );
    // A skipped section opened from the review counts itself: deductions,
    // with nothing for this return.
    let reply = from_review(&mut s, TaxSection::Deductions);
    let Reply::Edit(P::SaveDeductions { progress, .. }) = &reply else {
        panic!("expected the deductions page, got {reply:?}")
    };
    assert!(shown(progress).contains("section 9 of 13"), "{}", shown(progress));
}

#[test]
fn back_passes_skipped_sections() {
    let (mut s, reply) = business_and_estimated_payments();
    let reply = save_opened_topic(&mut s, &reply, vec![business_document(5_000_000)]);
    assert!(matches!(reply, Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    let hub = income_hub(from_review(&mut s, TaxSection::Income));
    // The summary continues to the next shown section: adjustments, which
    // every return walks for More adjustments (gate fixes batch A).
    assert_eq!(suggestion(&hub), TaxSection::Adjustments);
    // The payments page's Back opens the previous shown section, the credits
    // hub; other taxes is shown only when planned or needed.
    let Reply::Edit(P::SavePayments { previous, .. }) = from_review(&mut s, TaxSection::Payments)
    else {
        panic!("expected the payments page")
    };
    assert_eq!(previous, PAGE_CREDITS);
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
    // A rental and work abroad planned.
    let (mut s, reply) =
        walked_from_the_checklist(activities(ActivityTopics::RENTALS.0 | ActivityTopics::ABROAD.0));
    let resumed = |s: &Interview| {
        let mut resumed = restart(s.draft.clone());
        send(&mut resumed, P::OpenInterview { user: TEST_USER })
    };
    // Mid-walk, resume opens the next planned topic of the current hub.
    let reply = save_opened_topic(&mut s, &reply, vec![]);
    assert!(matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::WorkAbroad, .. })));
    assert!(matches!(
        resumed(&s),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::WorkAbroad, .. })
    ));
    // With the walk ended, its summaries pass without another request.
    let reply = save_opened_topic(&mut s, &reply, vec![]);
    assert!(matches!(
        reply,
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
    assert!(matches!(
        resumed(&s),
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
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
        TaxSection::OftenMissed,
        TaxSection::Review,
        TaxSection::Care,
        TaxSection::HomeOffices,
        TaxSection::Documents,
        TaxSection::IncomeWithoutForm,
        TaxSection::LongTermCareBenefits,
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
            .chain(TaxSection::Rentals as i32..=TaxSection::OtherIncome as i32)
            .chain([TaxSection::LongTermCareBenefits as i32])
            .chain(TaxSection::Gambling as i32..=TaxSection::WorkAbroad as i32)
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
    let request = save_deductions(&s.draft, Some(ItemizeOverride::Itemize), 0);
    let itemized = itemized_hub(send_past_less_common(&mut s, request));
    let P::Hub { navigation, .. } = itemized else { unreachable!() };
    assert_eq!(
        navigation.deductions,
        (TaxSection::Deductions as i32..=TaxSection::OtherItemized as i32).collect::<Vec<_>>()
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
    P::OpenSection {
        user: TEST_USER,
        cookie,
        revision,
        from: page,
        section: row.section,
        reveal_path: row.field,
    }
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
    P::OpenSection { user, cookie, revision, from, section: suggested, reveal_path: None }
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
                data: crate::GamblingSource::OwnRecords(crate::GamblingLosses {
                    other_winnings: 0,
                    losses: 100,
                    special: false,
                }),
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
    assert_eq!(crate::interview::issue_page(&d, &wage, None), Some(TaxSection::Documents));
    assert_eq!(crate::interview::issue_page(&d, &bank, None), Some(TaxSection::Documents));
    d.income[1].label = "Renamed bank".into();
    assert_eq!(crate::interview::issue_page(&d, &bank, None), Some(TaxSection::Documents));
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
    assert_eq!(remaining.iter().filter(|item| item.section == TaxSection::Documents && matches!(status_message(&item.status).args.get(1), Some(MessageArg::Nested(issue)) if issue.key == "ISSUE_EMPLOYEE_WAGES_RECORDS")).count(), 2, "{remaining:?}");
}

#[test]
fn hub_row_binds_its_topic() {
    // Schema: every hub and review row is a ReviewItem, whose type links to
    // OpenSection; OpenSection has no input (every field hidden), reads the
    // tapped row's section and falls back to the suggestion; the hub
    // declares separate Actions and Go to controls; every
    // Response carrying rows lists OpenSection and keeps Navigate.
    let navigation = include_str!("../../core/shared/navigation.rs");
    assert!(navigation.contains("#[libertas_link(\"OpenSection\")]\npub struct ReviewItem {"));
    let protocol = include_str!("../../core/protocol.rs");
    let open = declaration(protocol, "    OpenSection {");
    assert!(open.contains("#[libertas_copy_from(\"^<ReviewItem>.section,$.suggested\")]"));
    assert!(open.contains("#[libertas_reveal_path]\n        #[libertas_copy_from(\"^<ReviewItem>.field\")]\n        reveal_path: Option<String>"));
    let fields = open.matches(": ").count();
    assert_eq!(open.matches("#[libertas_hidden]").count(), fields, "{open}");
    let before_hub = &protocol[..protocol.find("    Hub {").unwrap()];
    let hub_attrs = &before_hub[before_hub.rfind("/// Topics").unwrap()..];
    assert!(hub_attrs.contains("ContinueIncome,ContinueAdjustments,ContinueItemized,ContinueCredits,ReviewAccepted,Navigate,OpenSection"));
    assert!(hub_attrs.contains("libertas_constrained_by"));
    assert!(declaration(protocol, "    Hub {").contains("allowed_actions"));
    assert!(
        protocol.contains(
            "#[libertas_next_request(\n        \"Navigate,OpenSection,StartFinish,OpenInterview,ViewReviewedRelatedForm\"\n    )]"
        )
    );
    // Server: the income hub suggests the planned Self-employment; a tap on another
    // row opens that topic at once, and saving it returns to the hub.
    let (mut s, hub) = at_the_income_hub();
    assert_eq!(suggestion(&hub), TaxSection::SelfEmployment);
    let request = tap_row(&hub, row_for(&hub, TaxSection::InterestDividends));
    let Reply::Edit(page) = send(&mut s, request) else { panic!("expected the interest page") };
    let P::SaveIncomes { topic, .. } = &page else {
        panic!("expected the interest page, got {page:?}")
    };
    assert_eq!(*topic, TaxSection::InterestDividends);
    // "What you told us" opens the checklist in one tap.
    let request = tap_row(&hub, row_for(&hub, TaxSection::Checklist));
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SaveChecklist { .. })));
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
    assert_eq!(suggestion(&hub), TaxSection::SelfEmployment);
    let by_fallback = send(&mut s, open_without_bound_row(&hub));
    let by_popup = send(&mut s, go_on(&hub));
    for reply in [by_fallback, by_popup] {
        assert!(
            matches!(reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. })),
            "{reply:?}"
        );
    }
    // The popup offers the hub's destinations, the suggestion among them.
    let P::Navigate { section, navigation, .. } = choose_suggestion(&hub) else { unreachable!() };
    assert!(navigation.groups.contains(&section.group_index()));
    assert_eq!(section.section(), suggestion(&hub));
}

// A printed source can be accepted before its nonprinted answer is known.
// Leaving income keeps that question pending; its review link opens the whole topic.
fn document_answer_survives_skip_and_blocks_review(
    source: IncomeSource,
    topic: TaxSection,
    field: &str,
    question: &str,
    answer: fn(&mut crate::DocumentAnswer),
) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    if topic == TaxSection::Retirement {
        accepted(&crate::tests::support::client::save_retirement_fixture(
            &mut session,
            vec![document(0, source)],
        ));
    } else {
        let request = save_documents(&mut session, vec![document(0, source)]);
        accepted(&send(&mut session, request));
    }
    let source_id = session
        .draft
        .income
        .iter()
        .find(|entry| {
            matches!(
                (&entry.income, topic),
                (FederalIncome::Wage { .. }, TaxSection::Wages)
                    | (FederalIncome::Retirement { .. }, TaxSection::Retirement)
            )
        })
        .unwrap()
        .id;
    let key = alloc::format!("income/{source_id}/{field}");
    let page = open_topic(&mut session, topic);
    let (unanswered, document) = match &page {
        P::SaveWagesAnswers { wages, .. } => {
            assert_eq!(topic, TaxSection::Wages);
            let row = wages.iter().find(|row| row.key == key).unwrap();
            assert!(!row.answers.calls.occupation.is_empty());
            assert!(row.answers.tips_listed_occupation.is_none());
            (
                crate::DocumentAnswer::Wage { answers: row.answers.clone() },
                status_message(&row.document),
            )
        }
        P::SaveIncomes { topic: opened, followups, .. } => {
            assert_eq!(*opened, topic);
            let row = followups.iter().find(|row| row.key == key).unwrap();
            let crate::DocumentAnswer::RetirementBasis { basis } = &row.answer else {
                panic!("unexpected pending answer: {:?}", row.answer)
            };
            assert!(basis.is_none());
            assert_eq!(row.allowed_answers.len(), 1);
            let prompt = status_message(&row.prompt);
            assert_eq!(prompt.key, "DOCUMENT_TOPIC_SOURCE_ITEM");
            let [MessageArg::Resource(prompt_question), MessageArg::Nested(document)] =
                prompt.args.as_slice()
            else {
                panic!("the combined prompt retains the question and its physical document")
            };
            assert_eq!(prompt_question, question);
            let prompt_text = shown(&row.prompt);
            let document_text = document.english();
            assert!(!document_text.is_empty());
            assert!(prompt_text.contains(&document_text));
            assert!(!prompt_text.contains(&alloc::format!("income/{source_id}")));
            (row.answer.clone(), document.as_ref().clone())
        }
        other => panic!("expected the document's answer editor, got {other:?}"),
    };
    assert!(matches!(
        document.key.as_str(),
        "DOCUMENT_CHOICE" | "DOCUMENT_FROM" | "DOCUMENT_PRINTED_FOR"
    ));
    let [MessageArg::Nested(kind), ..] = document.args.as_slice() else {
        panic!("the document label retains its physical kind")
    };
    assert_eq!(
        kind.key,
        if topic == TaxSection::Wages { "SAVING_KIND_W2_ONE" } else { "SAVING_KIND_1099_R_ONE" }
    );
    assert!(kind.args.is_empty());
    let document_text = document.english();
    assert!(!document_text.is_empty());
    assert!(!document_text.contains(&alloc::format!("income/{source_id}")));
    let calculation_state = estimate(&session.draft).state;

    // Continue is the actual walkthrough action for leaving the income section.
    if !session.draft.finished_sections.contains(&TaxSection::Income) {
        let request = continue_hub(&session.draft, PAGE_INCOME);
        accepted(&send(&mut session, request));
    }
    assert!(session.draft.finished_sections.contains(&TaxSection::Income));
    if topic == TaxSection::Wages {
        assert!(!session.draft.finished_sections.contains(&topic));
    }
    let Reply::Response(review @ P::Review { .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected review with the unanswered document question")
    };
    let P::Review { remaining_items, allowed_actions, result, .. } = &review else {
        unreachable!()
    };
    assert_eq!(result.state, calculation_state);
    assert!(!allowed_actions.contains(&FINISH));
    let expected = Message::new(
        "DOCUMENT_PENDING_ANSWER",
        vec![MessageArg::Resource(question.into()), MessageArg::Encoded(document.encode())],
    )
    .encode();
    let pending = remaining_items
        .iter()
        .find(|item| item.section == topic && item.item == expected)
        .expect("pending row retains both the question and its physical document")
        .clone();
    // A box question is answered in the page's `questions`.
    let locator: alloc::string::String = if topic == TaxSection::Wages {
        libertas::SchemaPath::root()
            .field("questions")
            .key(alloc::format!("{key}/occupation"))
            .field("answer")
            .into()
    } else {
        libertas::SchemaPath::root().field("questions").key(key.as_str()).field("answer").into()
    };
    assert_eq!(pending.field.as_deref(), Some(locator.as_str()));
    let navigation = tap_row(&review, pending);
    let P::OpenSection { reveal_path, .. } = &navigation else { unreachable!() };
    assert_eq!(reveal_path.as_deref(), Some(locator.as_str()));
    let Reply::Edit(mut request) = send(&mut session, navigation) else {
        panic!("the pending link must open the whole income topic")
    };
    let mut answered = unanswered.clone();
    answer(&mut answered);
    match &mut request {
        P::SaveWagesAnswers { wages, .. } => {
            assert_eq!(topic, TaxSection::Wages);
            let target = wages.iter_mut().find(|row| row.key == key).unwrap();
            assert_eq!(target.document, document.encode());
            let crate::DocumentAnswer::Wage { answers: expected } = &unanswered else {
                unreachable!()
            };
            assert_eq!(&target.answers, expected);
            let crate::DocumentAnswer::Wage { answers } = &answered else { unreachable!() };
            target.answers = answers.clone();
        }
        P::SaveIncomes { topic: opened, followups, .. } => {
            assert_eq!(*opened, topic);
            let target = followups.iter_mut().find(|row| row.key == key).unwrap();
            let P::SaveIncomes { followups: original, .. } = &page else { unreachable!() };
            assert_eq!(&*target, original.iter().find(|row| row.key == key).unwrap());
            target.answer = answered.clone();
        }
        other => panic!("expected the pending topic's editor, got {other:?}"),
    }
    let revision = session.draft.revision;
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, revision + 1);
    supported(&session.draft);
    let (remaining, _, actions, _) = review_state(&mut session);
    assert!(!remaining.iter().any(|item| item.section == topic && item.item == expected));
    assert!(remaining.is_empty(), "{remaining:?}");
    assert!(actions.contains(&FINISH));
    let reopened = open_topic(&mut session, topic);
    match &reopened {
        P::SaveWagesAnswers { wages, .. } => {
            let row = wages.iter().find(|row| row.key == key).unwrap();
            assert_eq!(row.document, document.encode());
            assert_eq!(crate::DocumentAnswer::Wage { answers: row.answers.clone() }, answered);
        }
        P::SaveIncomes { followups, .. } => {
            let row = followups.iter().find(|row| row.key == key).unwrap();
            assert_eq!(row.answer, answered);
        }
        other => panic!("expected the saved topic's editor, got {other:?}"),
    }
    let before = session.draft.clone();
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, before);
}

#[test]
fn w2_box12_question_can_be_skipped_then_answered_from_review() {
    let mut form = w2(6_000_000);
    form.more.box12 = vec![Box12Entry { code: W2Code::TP, amount: 100_000 }];
    document_answer_survives_skip_and_blocks_review(
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
        TaxSection::Wages,
        "wage",
        "W2_TP_OCCUPATION_NEEDED",
        |answer| {
            let crate::DocumentAnswer::Wage { answers } = answer else { unreachable!() };
            answers.tips_listed_occupation = Some(WageDeductionEligibility::Eligible);
        },
    );
}

#[test]
fn retirement_box10_basis_can_be_skipped_then_answered_from_review() {
    let mut form = printed_as(r1099(1_000_000, normal_kept(), false), "7B");
    form.source_mut().form.box_10 = Some(100_000);
    document_answer_survives_skip_and_blocks_review(
        IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(form)),
        },
        TaxSection::Retirement,
        "basis",
        "ISSUE_BOX_10_BASIS_UNKNOWN",
        |answer| {
            let crate::DocumentAnswer::RetirementBasis { basis } = answer else { unreachable!() };
            *basis = Some(crate::InPlanRothBasis::Known { amount: 25_000 });
        },
    );
}

#[test]
fn accepted_native_loan_topics_edit_answers_and_ignore_source_summary_echoes() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let names = fixture_document_names(&session.draft, -1);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let mut mortgage = first_lender_mortgage(600_000, 10_000_000, AverageBalance::WithinLimit);
    mortgage.borrower = names.as_ref().unwrap().recipient.clone();
    mortgage.lender = names.as_ref().unwrap().issuer.clone().unwrap_or_default();
    mortgage.points = 100_000;
    documents.common.mortgages.documents.push(mortgage_document(mortgage));
    documents.common.student_loans.documents.push(crate::StudentLoanDocument {
        id: 0,
        form: student_loan_paper(Form1098E {
            lender: (names.clone())
                .as_ref()
                .and_then(|names| names.issuer.clone())
                .unwrap_or_default(),
            borrower: (names.clone())
                .as_ref()
                .map(|names| names.recipient.clone())
                .unwrap_or_default(),
            interest: 50_000,
            box_2: true,
        }),
    });
    let mut vehicle = vehicle_loan("Synthetic vehicle lender", 100_000);
    vehicle.document.payer_of_record = names.unwrap().recipient;
    vehicle.document.box_6 = true;
    vehicle.document.box_7 = false;
    documents
        .other
        .vehicle_loans
        .documents
        .push(crate::Form1098VliDocument { id: vehicle.id, form: vehicle.document });
    accepted(&send(&mut session, request));

    let request = navigate(&session.draft, TaxSection::StudentLoan);
    let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(&mut session, request) else {
        panic!("expected the student loan topic")
    };
    let mut bad = request.clone();
    let P::SaveStudentLoan { value, .. } = &mut bad else { unreachable!() };
    value.forms[0].id += 10_000;
    let before = session.draft.clone();
    rejected_with(send(&mut session, bad), "DOCUMENT_QUESTIONS_CHANGED");
    assert_eq!(session.draft, before);
    let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
    assert_eq!(value.forms[0].allowed_answers, vec![1]);
    value.forms[0].additional_interest =
        crate::StudentLoanAdditionalInterest::Checked { additional_interest: 25_000 };
    value.forms[0].value.box_1 = 0;
    value.forms[0].value.box_2 = false;
    accepted(&send(&mut session, request));
    let form = &session.draft.adjustments.as_ref().unwrap().student_loan.forms[0];
    assert_eq!(form.document.box_1, 50_000);
    assert!(form.document.box_2);
    assert_eq!(form.additional_interest, 25_000);

    let mut request = linked_mortgage_editor(&mut session);
    let mut bad = request.clone();
    let P::SaveMortgages { value, .. } = &mut bad else { unreachable!() };
    value.loans.clear();
    let before = session.draft.clone();
    rejected_with(send(&mut session, bad), "DOCUMENT_QUESTIONS_CHANGED");
    assert_eq!(session.draft, before);
    let P::SaveMortgages { value, .. } = &mut request else { unreachable!() };
    value.loans[0].property = "Synthetic main home".into();
    value.loans[0].points =
        crate::MortgagePointTreatment::Amortized { term_months: 360, ended: true };
    value.loans[0].value.box_1 = 0;
    value.loans[0].value.box_6 = 1;
    accepted(&send(&mut session, request));
    let loan = mortgage_loan(&session.draft.deductions.as_ref().unwrap().mortgage[0]);
    assert_eq!(loan.interest, 600_000);
    assert_eq!(loan.property, "Synthetic main home");
    assert_eq!(
        loan.points,
        MortgagePoints::Amortized { amount: 100_000, term_months: 360, ended: true }
    );

    let request = navigate(&session.draft, TaxSection::Deductions);
    let Reply::Edit(mut request @ P::SaveDeductions { .. }) = send(&mut session, request) else {
        panic!("expected the deductions topic")
    };
    let mut bad = request.clone();
    let P::SaveDeductions { value, .. } = &mut bad else { unreachable!() };
    value.car_loans.push(value.car_loans[0].clone());
    let before = session.draft.clone();
    rejected_with(send(&mut session, bad), "DOCUMENT_QUESTIONS_CHANGED");
    assert_eq!(session.draft, before);
    let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
    assert!(value.car_loans[0].value.box_6);
    assert!(!value.car_loans[0].value.box_7);
    value.car_loans[0].business = 20_000;
    // IRS 1098-VLI December 2026: boxes 6 and 7 are printed source checkboxes.
    value.car_loans[0].value.box_6 = false;
    value.car_loans[0].value.box_7 = true;
    value.car_loans[0].value.box_1 = 0;
    value.car_loans[0].value.lender.name = "Forged summary lender".into();
    accepted(&send(&mut session, request));
    let loan = &session.draft.deductions.as_ref().unwrap().vehicle_loans[0];
    assert_eq!(loan.document.lender.name, "Synthetic vehicle lender");
    assert_eq!((loan.document.box_1, loan.business), (100_000, 20_000));
    assert_eq!((loan.document.box_6, loan.document.box_7), (true, false));
    supported(&session.draft);
}

fn pending_loan_review(session: &mut Interview, topic: TaxSection, key: &str) -> (P, ReviewItem) {
    let Reply::Response(review @ P::Review { .. }) =
        send(session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected review with the retained loan conflict")
    };
    let P::Review { remaining_items, allowed_actions, .. } = &review else { unreachable!() };
    assert!(!allowed_actions.contains(&FINISH));
    let row = remaining_items.iter().find(|row| {
        let message = status_message(&row.item);
        row.section == topic && message.key == "DOCUMENT_PENDING_ANSWER"
            && matches!(message.args.first(), Some(MessageArg::Resource(question)) if question == key)
    }).expect("the retained loan conflict links to its editable topic").clone();
    (review, row)
}

#[test]
fn vehicle_printed_interest_correction_retains_business_answer_until_topic_fix() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut source = vehicle_loan("Synthetic correction lender", 100_000);
    source.document.payer_of_record = fixture_document_names(&session.draft, -1).unwrap().recipient;
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents
        .other
        .vehicle_loans
        .documents
        .push(crate::Form1098VliDocument { id: source.id, form: source.document });
    accepted(&send(&mut session, request));
    let navigation = navigate(&session.draft, TaxSection::Deductions);
    let Reply::Edit(mut request @ P::SaveDeductions { .. }) = send(&mut session, navigation) else {
        unreachable!()
    };
    let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
    value.car_loans[0].business = 60_000;
    accepted(&send(&mut session, request));
    assert_eq!(super::value(&supported(&session.draft), "vehicle_interest_deduction"), 400);
    let id = session.draft.deductions.as_ref().unwrap().vehicle_loans[0].id;
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.other.vehicle_loans.documents[0].form.box_1 = 50_000;
    accepted(&send(&mut session, request));
    let loan = &session.draft.deductions.as_ref().unwrap().vehicle_loans[0];
    assert_eq!((loan.id, loan.document.box_1, loan.business), (id, 50_000, 60_000));
    assert!(session.draft.valid_stored());
    assert!(
        estimate(&session.draft)
            .issues
            .iter()
            .any(|issue| issue.message.contains("Schedule C, E or F"))
    );
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let (review, row) = pending_loan_review(
        &mut session,
        TaxSection::Deductions,
        "VEHICLE_BUSINESS_ABOVE_INTEREST",
    );
    assert!(shown(&row.item).contains("Synthetic correction lender"));
    let Reply::Edit(mut request @ P::SaveDeductions { .. }) =
        send(&mut session, tap_row(&review, row))
    else {
        unreachable!()
    };
    let before = session.draft.clone();
    rejected_with(send(&mut session, request.clone()), "VEHICLE_BUSINESS_ABOVE_INTEREST");
    assert_eq!(session.draft, before);
    let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
    value.car_loans[0].business = 40_000;
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(super::value(&result, "vehicle_interest_deduction"), 100);
    let (remaining, _, actions, _) = review_state(&mut session);
    assert!(remaining.is_empty(), "{remaining:?}");
    assert!(actions.contains(&FINISH));
    assert_eq!(supported(&restart(session.draft.clone()).draft), result);
}

#[test]
fn student_loan_printed_correction_retains_employer_answer_until_topic_fix() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut source = box_1(100_000);
    source.document.borrower = fixture_document_names(&session.draft, -1).unwrap().recipient;
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents
        .common
        .student_loans
        .documents
        .push(crate::StudentLoanDocument { id: source.id, form: source.document });
    accepted(&send(&mut session, request));
    let navigation = go_to(&session.draft, PAGE_REVIEW, TaxSection::StudentLoan);
    let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(&mut session, navigation)
    else {
        unreachable!()
    };
    let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
    value.employer_paid_interest = 80_000;
    accepted(&send(&mut session, request));
    assert_eq!(return_line(&supported(&session.draft), "10"), 20_000);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.student_loans.documents[0].form.box_1 = 50_000;
    accepted(&send(&mut session, request));
    let claim = &session.draft.adjustments.as_ref().unwrap().student_loan;
    assert_eq!((claim.forms[0].document.box_1, claim.employer_paid_interest), (50_000, 80_000));
    assert!(session.draft.valid_stored());
    assert!(
        estimate(&session.draft).issues.iter().any(|issue| issue.message.contains("employer paid"))
    );
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let (review, row) = pending_loan_review(
        &mut session,
        TaxSection::StudentLoan,
        "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST",
    );
    let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) =
        send(&mut session, tap_row(&review, row))
    else {
        unreachable!()
    };
    let before = session.draft.clone();
    rejected_with(send(&mut session, request.clone()), "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST");
    assert_eq!(session.draft, before);
    let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
    value.employer_paid_interest = 40_000;
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "10"), 10_000);
    let (remaining, _, actions, _) = review_state(&mut session);
    assert!(remaining.is_empty(), "{remaining:?}");
    assert!(actions.contains(&FINISH));
    assert_eq!(supported(&restart(session.draft.clone()).draft), result);
}

#[test]
fn review_item_row_opens_its_page() {
    // An unsupported 1099-R code 5 is a remaining source item: its row opens Document center
    // in one tap. An answers row opens its section. With the interview walk
    // complete, only Navigate and link-only OpenSection remain allowed;
    // Finish joins only with nothing remaining.
    let original = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, original.clone());
    let r = other_1099(1_000_000, "5", false);
    accepted(&{
        crate::tests::support::client::save_retirement_fixture(
            &mut s,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r)),
                },
            )],
        )
    });
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
    assert_eq!(item.section, TaxSection::Documents);
    assert!(matches!(
        send(&mut s, tap_row(&review, item)),
        Reply::Edit(P::SaveDocuments { review: true, .. })
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
    assert_eq!(topics.items.last().unwrap().section, TaxSection::HomeOffices);
    assert_eq!(previous_section.section, TaxSection::Checklist);
    assert_eq!(status_text(&previous_section.item), "Previous step");
    assert!(allowed_actions.contains(&(libertas_macros::variant_index!(P::ContinueIncome) as i32)));
    assert!(!topics.items.iter().any(|row| row.section == TaxSection::Review));
    assert!(!s.draft.finished_sections.contains(&TaxSection::Income));
    // Explicit Continue passes every summary with no question remaining.
    let reply = send(&mut s, hub_action(&hub));
    assert!(
        matches!(
            reply,
            Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
        ),
        "{reply:?}"
    );
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
            TaxSection::Income => draft.checklist.paid = PaidTopics::IRA_CONTRIBUTIONS.0,
            TaxSection::Adjustments => draft.checklist.paid = PaidTopics::GIFTS.0,
            TaxSection::Itemized => {
                {
                    draft.deductions.as_mut().unwrap().itemized = empty_facts();
                    draft.deductions.as_mut().unwrap().itemize_override =
                        Some(ItemizeOverride::Itemize);
                };
                draft.checklist.paid = PaidTopics::CARE.0;
                draft.finished_sections.retain(|s| *s != TaxSection::Care);
            }
            TaxSection::Credits => draft.checklist.paid = PaidTopics::PAYMENTS.0,
            _ => unreachable!(),
        }
        let destination = match section {
            TaxSection::Income => TaxSection::Adjustments,
            TaxSection::Adjustments => TaxSection::Deductions,
            TaxSection::Itemized => TaxSection::Credits,
            TaxSection::Credits => TaxSection::Payments,
            _ => unreachable!(),
        };
        draft.finished_sections.retain(|s| *s != destination);
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
                TaxSection::Income => {
                    matches!(reply, Reply::Edit(P::SaveIraContributions { .. }))
                }
                TaxSection::Adjustments => matches!(reply, Reply::Edit(P::SaveDeductions { .. })),
                TaxSection::Itemized => matches!(reply, Reply::Edit(P::SaveCare { .. })),
                // Other taxes is shown only when planned or needed.
                TaxSection::Credits => matches!(reply, Reply::Response(P::BeginPayments { .. })),
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
        TaxSection::Adjustments => Some(TaxSection::MoreAdjustments),
        TaxSection::Credits => Some(TaxSection::MoreCredits),
        _ => None,
    }
}
// Start at an unfinished hub, with every later section unfinished too. Each
// destination is planned independently of the production navigation helpers.
fn continuation_draft(section: TaxSection, destination: TaxSection) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft
        .finished_sections
        .retain(|s| *s == TaxSection::IncomeWithoutForm || (*s as i32) < section as i32);
    if section == TaxSection::Itemized {
        {
            draft.deductions.as_mut().unwrap().itemized = empty_facts();
            draft.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
        };
    }
    match destination {
        TaxSection::Adjustments => draft.checklist.paid |= PaidTopics::IRA_CONTRIBUTIONS.0,
        TaxSection::Deductions => draft.checklist.paid |= PaidTopics::GIFTS.0,
        TaxSection::Credits => draft.checklist.paid |= PaidTopics::CARE.0,
        TaxSection::OtherTaxes => {
            draft.checklist.activities |= ActivityTopics::HOUSEHOLD_EMPLOYEE.0;
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
            // Continue may advance only after the earlier source-center step.
            let sources = open_document_center(&mut session);
            accepted(&send(&mut session, sources));
            let request = back(&session.draft, section as i32);
            let hub = hub_of(send(&mut session, request));
            let reply = send(&mut session, hub_action(&hub));
            assert!(
                match destination {
                    Adjustments => matches!(reply, Reply::Edit(P::SaveIraContributions { .. })),
                    Deductions => matches!(reply, Reply::Edit(P::SaveDeductions { .. })),
                    Credits => matches!(reply, Reply::Edit(P::SaveCare { .. })),
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
fn hub_continue_passes_a_visited_topic_without_an_extra_summary() {
    for (section, destination, topic) in [
        (TaxSection::Income, TaxSection::Adjustments, TaxSection::IraContributions),
        (TaxSection::Adjustments, TaxSection::Credits, TaxSection::Care),
        (TaxSection::Itemized, TaxSection::Credits, TaxSection::Care),
    ] {
        let mut draft = continuation_draft(section, destination);
        // Saving a planned topic with nothing entered visits it; the tick
        // remains, but the walkthrough must not open that editor again. The
        // destination's less-common page was passed too (gate fixes batch A).
        draft.finished_sections.push(topic);
        draft.finished_sections.extend(hub_topics_last(destination));
        let mut session = Interview::new(TEST_USER, draft);
        let sources = open_document_center(&mut session);
        accepted(&send(&mut session, sources));
        let request = back(&session.draft, section as i32);
        let hub = hub_of(send(&mut session, request));
        let reply = send(&mut session, hub_action(&hub));
        assert!(
            matches!(
                reply,
                Reply::Response(P::BeginOftenMissed { .. })
                    | Reply::Edit(P::SaveOftenMissed { .. })
            ),
            "visited {destination:?}: {reply:?}"
        );
        assert!(session.draft.finished_sections.contains(&destination));
        let summary = hub_of(from_review(&mut session, destination));
        assert!(matches!(summary, P::Hub { page, .. } if page == destination as i32));
    }
}

#[test]
fn hub_continue_leaves_the_section_even_when_a_topic_is_suggested() {
    for (section, topic) in [
        (TaxSection::Income, TaxSection::SelfEmployment),
        (TaxSection::Adjustments, TaxSection::IraContributions),
        (TaxSection::Itemized, TaxSection::GiftCarryovers),
        (TaxSection::Credits, TaxSection::Care),
    ] {
        // Each continues to the next shown section (gate fixes batch A).
        let next = match section {
            TaxSection::Income => TaxSection::Adjustments,
            TaxSection::Credits => TaxSection::OtherTaxes,
            _ => TaxSection::Credits,
        };
        let mut draft = continuation_draft(section, next);
        match section {
            TaxSection::Income => draft.checklist.activities |= ActivityTopics::BUSINESS.0,
            TaxSection::Adjustments => draft.checklist.paid |= PaidTopics::IRA_CONTRIBUTIONS.0,
            TaxSection::Itemized => draft.checklist.carryovers |= CarryoverTopics::CHARITY.0,
            TaxSection::Credits => draft.checklist.paid |= PaidTopics::CARE.0,
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
fn automatic_continuation_enters_derived_topics_without_checklist_ticks() {
    for education in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        draft.finished_sections.retain(|s| {
            *s == TaxSection::IncomeWithoutForm || (*s as i32) < TaxSection::Income as i32
        });
        draft.people.as_mut().unwrap().taxpayer.full_time_student = education;
        let mut session = Interview::new(TEST_USER, draft);
        let mut wages =
            if education { w2(6_000_000) } else { w2_with_box_12_w(6_000_000, 100_000) };
        wages.employee = accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
        let wage_id = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
            .unwrap()
            .id;
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                wage_id,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wages)) },
            )],
        );
        let reply = send_past_less_common(&mut session, request);
        assert_eq!(session.draft.checklist.paid, 0);
        assert!(
            if education {
                matches!(reply, Reply::Response(P::Review { .. }))
            } else {
                matches!(reply, Reply::Edit(P::SaveHsa { .. }))
            },
            "education={education}: {reply:?}"
        );
        if education {
            let request = go_to(&session.draft, PAGE_CREDITS, TaxSection::Documents);
            let Reply::Edit(P::SaveDocuments { documents, .. }) = send(&mut session, request)
            else {
                panic!("education source editor")
            };
            assert!(documents.common.education.people.iter().any(|person| person.id == -1));
        }
    }
}

#[test]
fn hub_continue_walks_a_new_return_through_saved_pages() {
    // Start from actual saved setup, filer and checklist pages. No later
    // completion marks are supplied by a pre-completed fixture.
    let (mut session, reply) = walked_from_the_checklist(Checklist {
        activities: ActivityTopics::RENTALS.0,
        paid: PaidTopics::IRA_CONTRIBUTIONS.0 | PaidTopics::PAYMENTS.0,
        ..Checklist::NONE
    });
    let reply = save_opened_topic(&mut session, &reply, vec![]);
    let Reply::Edit(iras @ P::SaveIraContributions { .. }) = reply else {
        panic!("finishing income opens the planned IRA contributions page")
    };
    let Reply::Edit(payments @ P::SavePayments { .. }) =
        crate::tests::support::client::answer_no_estimates(send(&mut session, iras))
    else {
        panic!("finishing adjustments passes the empty credits summary and opens Payments")
    };
    // The last page before the review is passed with its answers kept.
    let often_missed = crate::tests::support::client::answer_often_missed(
        send(&mut session, payments),
        OftenMissed::default(),
    );
    assert!(matches!(send(&mut session, often_missed), Reply::Response(P::Review { .. })));
}

#[test]
fn hub_continue_from_an_earlier_section_passes_completed_summaries() {
    let mut draft = complete(FederalStatus::Single);
    draft.finished_sections.retain(|s| *s != TaxSection::Payments);
    draft.finished_sections.push(TaxSection::Marketplace);
    draft.checklist.paid |= PaidTopics::PAYMENTS.0;
    let mut session = Interview::new(TEST_USER, draft);
    let sources = open_document_center(&mut session);
    accepted(&send(&mut session, sources));
    assert!(matches!(
        send(&mut session, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::BeginPayments { .. })
    ));
    let request = back(&session.draft, PAGE_INCOME);
    let income = income_hub(send(&mut session, request));
    let unchanged = session.draft.clone();
    let reply = send(&mut session, hub_action(&income));
    assert!(
        matches!(reply, Reply::Response(P::BeginPayments { .. })),
        "Income Continue passes completed summaries to Payments: {reply:?}"
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
    value.paid |= PaidTopics::IRA_CONTRIBUTIONS.0;
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
        reply = match crate::tests::support::client::answer_no_estimates(past_less_common(
            &mut session,
            reply,
        )) {
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
    // 2026-10-30: OpenHub also declares the stored-generation refusal; normal
    // refreshes still reload the hub without writing the return.
    assert!(attrs.contains("#[libertas_next_response(\"Hub,Problem,StoredReturnUnavailable\")]"));
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
pub(in crate::tests) fn at_the_gifts_page(
    itemize_override: Option<ItemizeOverride>,
    gifts: i64,
) -> Interview {
    let mut s = at_the_deductions_page(FederalStatus::Single).0;
    if itemize_override.is_none() {
        s.draft.checklist.paid |= PaidTopics::GIFTS.0;
    }
    let request = save_deductions(&s.draft, itemize_override, gifts);
    crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    s
}
pub(in crate::tests) fn gifts(
    goods: Vec<PropertyGift>,
    stock: Vec<PropertyGift>,
    vehicles: Vec<PropertyGift>,
) -> GiftCarryovers {
    // Vehicles are Form 1098-C gifts on their own list.
    GiftCarryovers {
        vehicles: vehicles.into_iter().map(vehicle_gift_page).collect(),
        gifts: goods.into_iter().chain(stock).map(Into::into).collect(),
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
    use crate::tests::support::client::charity::save_gift_source_fixtures;
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
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 100_000);
    let mut page = gifts(
        vec![
            goods_gift(GoodsKind::Clothing, 120_000, 30_000),
            goods_gift(GoodsKind::Household, 400_000, 170_000),
        ],
        vec![stock_gift(20200302, 200_000, 1_500_000)],
        vec![vehicle_gift(400_000, sold(260_000))],
    );
    itemized_hub(save_gift_source_fixtures(&mut s, &mut page));
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
    use crate::tests::support::client::charity::save_gift_source_fixtures;
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
    let mut value = gifts(
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
    itemized_hub(save_gift_source_fixtures(&mut s, &mut value));
    let ded = s.draft.deductions.as_ref().unwrap();
    // The stored gifts, then the Form 1098-C gifts, as the calculation reads them.
    let gifts = ded.gifts().expect("accepted property gifts");
    assert_eq!(gifts.len(), 5);
    let book = &gifts[0];
    assert!(matches!(book.kind, GiftKind::Goods { kind: GoodsKind::Books, .. }));
    assert_eq!((book.basis, book.value), (50_000, 20_000));
    assert!(book.appraisal.is_none());
    assert_eq!(
        gifts[1..3].iter().map(|g| g.claim(false)).collect::<Vec<_>>(),
        vec![(CharityClass::CapitalGain30, 1_500_000), (CharityClass::Property50, 100_000),]
    );
    assert!(matches!(gifts[3].kind, GiftKind::Vehicle { treatment: VehicleUse::NeedyPerson, .. }));
    assert_eq!(gifts[4].claim(false).1, 150_000);
    // Reopened, the page shows exactly what was saved.
    let P::SaveGiftCarryovers { value: reopened, .. } =
        open_itemized_topic(&mut s, TaxSection::GiftCarryovers)
    else {
        panic!("expected the gifts page")
    };
    assert_eq!(reopened, value);
    // 2026-11-02: automatic selection preserves all accepted gift records.
    let saved_gifts = s.draft.deductions.as_ref().unwrap().clone();
    let request = save_deductions(&s.draft, None, 0);
    send(&mut s, request);
    let ded = s.draft.deductions.as_ref().unwrap();
    assert_eq!(ded.property_gifts, saved_gifts.property_gifts);
    assert_eq!(ded.vehicle_gifts, saved_gifts.vehicle_gifts);
    assert_eq!(ded.vehicle_gift_documents.len(), 2);
    assert_eq!(ded.gifts().expect("accepted gift facts"), saved_gifts.gifts().unwrap());
    let result = supported(&s.draft);
    // 2026-11-02: retained gifts20,700 less the300 floor yield20,400.
    assert_eq!(crate::tests::value(&result, "deduction"), 20_400);
    assert_eq!(crate::tests::value(&result, "taxable_income"), 39_600);
    assert_eq!(result.tax, Some(450_700));
    assert_eq!(result.refund, Some(199_300));
}

#[test]
fn gift_page_rules_reject_on_both_sides() {
    use crate::tests::support::client::charity::save_gift_source_fixtures;
    let mut s = at_the_gifts_page(None, 0);
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
    let mut page = gifts(vec![], vec![], vec![vehicle]);
    rejected_with(save_gift_source_fixtures(&mut s, &mut page), "GIFT_BENEFIT_ABOVE_VALUE");
    let mut late = stock_gift(20261116, 100_000, 200_000);
    late.given = 20261115;
    let request = save_gift_carryovers(&s.draft, gifts(vec![], vec![late], vec![]));
    rejected_with(send(&mut s, request), "GIFT_BOUGHT_AFTER_GIVEN");
    let early = VehicleUse::Sold { sold: 20260531, proceeds: 100_000 };
    let mut page = gifts(vec![], vec![], vec![vehicle_gift(300_000, early)]);
    rejected_with(save_gift_source_fixtures(&mut s, &mut page), "VEHICLE_SOLD_BEFORE_GIVEN");
    assert_eq!(s.draft, before);
}

#[test]
fn similar_goods_above_5000_save_with_actual_section_b_requirements() {
    // Form 8283 Section B starts above $5,000 per similar-property group
    // across all gifts: two clothing gifts totalling exactly $5,000 stay in
    // Section A; one more cent needs a qualified appraisal; missing substantiation omits the gifts.
    // A different group does not add to the clothing total.
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
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
        farm_rental_use: None,
        allowed_farm_rental_use: alloc::vec::Vec::new(),
        macrs_source: None,
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost: 1_000_000,
        credit_reduction: 0,
        placed: 20240301,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class: AssetClass::Five,
                history: AssetHistory::Continuing {
                    prior_depreciation: 520_000,
                    first_year_179: 0,
                    first_year_bonus: 0,
                    carryforward: 0,
                    mid_quarter: false,
                },
            },
        },
        business_percent: 100,
        acquired: None,
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
            livestock: crate::SoldLivestock::No,
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
            macrs_source: None,
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
            kind: crate::VehicleKind::PassengerCar,
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
    let mut a = crate::cooperative::automatic(&data);
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
    assert_eq!(payments_value(&page).0.more.next_year, NextYearEstimate::NotNeeded);
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
    let entered = Payments {
        more: crate::PaymentsMore { next_year: changes.clone(), ..no_payments().more },
        ..no_payments()
    };
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
    assert_eq!(payments_value(&open_payments(&mut s)).0.more.next_year, changes);
    // T2-49b: the 2027 farmer answer belongs to the worksheet; the 2025
    // answer stays No (the IRS figures the penalty).
    assert!(!s.draft.payments.as_ref().unwrap().farmer_fisherman_prior_year);
    // Not needed again clears it and the review shows no Form 1040-ES line.
    accepted(&submit(&mut s, no_payments()));
    assert_eq!(s.draft.payments.as_ref().unwrap().next_year, NextYearEstimateRecord::NotNeeded);
    assert!(!estimate(&s.draft).lines.iter().any(|l| l.id.starts_with("form_1040_es_line_")));
}

// Canonical Form1095-A source lifecycle through actual public pages.
pub(in crate::tests) fn marketplace_page_of(page: &P) -> &MarketplaceFactsPage {
    let P::SaveMarketplace { value, page: at, previous, .. } = page else {
        panic!("expected Marketplace editor: {page:?}")
    };
    assert_eq!((*at, *previous), (PAGE_CREDITS, PAGE_CREDITS));
    value
}
pub(in crate::tests) fn save_marketplace(
    session: &mut Interview,
    mut value: MarketplaceFactsPage,
) -> P {
    let mut request = open_credits_topic(session, TaxSection::Marketplace);
    let P::SaveMarketplace { value: page, .. } = &mut request else { unreachable!() };
    if value.forms.is_empty() {
        value.forms = page.forms.clone();
    } else {
        assert_eq!(value.forms.len(), page.forms.len());
        for (form, accepted) in value.forms.iter_mut().zip(&page.forms) {
            form.id = accepted.id;
            form.value = accepted.value.clone();
        }
    }
    *page = value;
    request
}

pub(in crate::tests) fn documents_of(
    forms: &[MarketplaceDocumentView],
) -> Vec<crate::Form1095ADocument> {
    forms
        .iter()
        .map(|form| crate::Form1095ADocument { id: form.id, value: form.value.clone() })
        .collect()
}
pub(in crate::tests) fn save_marketplace_sources(
    session: &mut Interview,
    forms: Vec<crate::Form1095ADocument>,
) -> P {
    let mut request = open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.marketplace.documents = forms;
    request
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
) -> MarketplaceDocumentView {
    MarketplacePolicyRecord {
        printed: None,
        id: 0,
        form: MarketplacePolicyRecordPage {
            recipient: Default::default(),
            id: 0,
            issuer: issuer.into(),
            policy_number: "000123".into(),
            amounts: PolicyAmounts::AllYear { premium, benchmark: Some(benchmark), advance },
            status: MarketplaceStatementStatus::Original,
        }
        .into(),
        lawful_premiums: vec![],
        enrolling: MarketplaceFamilyRef::ThisReturn,
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
    forms: Vec<MarketplaceDocumentView>,
) -> MarketplaceFactsPage {
    // Explicit neutral completion for arithmetic fixtures: taxpayer-only
    // ordinary coverage and known eligible months, never a production default.
    MarketplaceFactsPage {
        eligibility: (1..=12)
            .filter(|month| {
                forms.iter().any(|p| {
                    crate::marketplace_sources::policy_month(&p.amounts(), *month).is_some()
                })
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
// Execute source preparation and topic facts through their separate actual editors.
pub(in crate::tests) fn save_marketplace_and_reopen(
    session: &mut Interview,
    value: MarketplaceFactsPage,
) -> MarketplaceFactsPage {
    let forms = documents_of(&value.forms);
    let request = save_marketplace_sources(session, forms);
    accepted(&send(session, request));
    let request = save_marketplace(session, value);
    accepted(&send(session, request));
    let request = open_credits_topic(session, TaxSection::Marketplace);
    marketplace_page_of(&request).clone()
}

// T2-44p: the W-2G and gambling losses kinds on the Gambling winnings topic.
pub(in crate::tests) fn w2g(winnings: i64, withholding: i64) -> IncomeSource {
    IncomeSource::Gambling(w2g_document(W2g {
        payer: Default::default(),
        winner: Default::default(),
        state_details: Default::default(),
        winnings,
        withholding,
        special: false,
    }))
}
pub(in crate::tests) fn gambling_losses(other_winnings: i64, losses: i64) -> IncomeSource {
    IncomeSource::Record {
        value: crate::IncomeRecord::GamblingLosses(GamblingLosses {
            other_winnings,
            losses,
            special: false,
        }),
    }
}

// T2-40p: the Schedule K-1 (Form 1041) kind on the Schedule K-1 topic, and
// the shared K-1 page types.
pub(in crate::tests) fn k1_1041(name: &str) -> K1EstateTrust {
    K1EstateTrust {
        estate_trust: crate::PrintedParty {
            name: name.into(),
            federal_id: "012345678".into(),
            address: Default::default(),
        },
        fiduciary: Default::default(),
        beneficiary: Default::default(),
        entity_period: None,
        final_character: None,
        successor: None,
        final_mortgages: vec![],
        amt_reconciliation: None,
        investment_income: None,
        reported_credits: vec![],
        foreign_statement: None,
        investment_property: vec![],
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
        section199a_eligible: Some(true),
        unentered,
    }
}
// Saves the K-1s in the document center of a fresh single return.
pub(in crate::tests) fn save_k1s(pages: Vec<K1EstateTrust>) -> (Interview, Reply) {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = {
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            pages
                .into_iter()
                .map(|p| {
                    document(
                        0,
                        IncomeSource::K1 {
                            value: crate::K1Document::K1EstateTrust(estate_document(p)),
                        },
                    )
                })
                .collect(),
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    (session, reply)
}
pub(in crate::tests) fn reopened_k1s(session: &mut Interview) -> Vec<IncomeSource> {
    page_documents(&open_document_center(session))
        .into_iter()
        .filter_map(|item| {
            matches!(item.document, IncomeSource::K1 { .. }).then_some(item.document)
        })
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
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        allowed_interest_limit_election: Vec::new(),
        related: Default::default(),
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
        material_participation: crate::ParticipationFacts {
            hours: 501,
            test: crate::ParticipationTest::Over500,
        },
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
pub(in crate::tests) fn farm_save(
    session: &mut Interview,
    documents: Vec<IncomeDocumentItem>,
) -> P {
    save_topic(session, TaxSection::Farming, documents)
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
pub(in crate::tests) fn reopened_farming(session: &mut Interview) -> (Vec<i32>, Vec<IncomeSource>) {
    let P::SaveIncomes { allowed_documents, documents, .. } =
        open_topic(session, TaxSection::Farming)
    else {
        unreachable!()
    };
    (
        allowed_documents,
        documents
            .documents
            .into_iter()
            .map(|item| IncomeSource::Record { value: item.document })
            .collect(),
    )
}

#[test]
fn young_filer_answers_what_produces_the_farms_income() {
    // F15 as T2-cy: on a return Form 8615 applies to, Not needed is refused
    // on save once the More fold is open (boundaries in
    // income_source_sets_follow_the_calculation).
    let mut original = form_8615_return(farm_only_return());
    let attempt = |original: &FederalDraft, source: FarmIncomeSource| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            let request = farm_save(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Farm(with_more(
                            cash_farm(1_000_000, 0),
                            |more| {
                                more.income_source = source;
                            },
                        )),
                    },
                )],
            );
            send(&mut session, request)
        };
        (reply, session)
    };
    let reply = {
        let mut source = Interview::new(TEST_USER, original.clone());
        current_document_refusal(
            "income-source-7609",
            &mut source,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Farm(with_more(cash_farm(1_000_000, 0), |more| {
                        more.income_source = FarmIncomeSource::NotNeeded;
                    })),
                },
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
    let IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } = &reopened[0] else {
        unreachable!()
    };
    assert_eq!(farm.more.income_source, FarmIncomeSource::Capital);
    // No parent alive: Form 8615 does not apply, Not needed passes.
    original.people.as_mut().unwrap().taxpayer.parent_alive = false;
    accepted(&attempt(&original, FarmIncomeSource::NotNeeded).0);
}

// T2-31p: the Schedule K-1 (Form 1120-S) kind on the Schedule K-1 topic.
pub(in crate::tests) fn k1_1120s(name: &str) -> K1Scorp {
    K1Scorp {
        exact: Default::default(),
        corporation: crate::PrintedParty::from_name(name.into()),
        shareholder: Default::default(),
        reporting_period: None,
        qbi_group: alloc::string::String::new(),
        disposition: Some(ScorpDisposition::Kept),
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
            participation: None,
            real_estate_qbi: None,
            other_qbi: None,
            carryover_active: true,
        },
        ordinary: 0,
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        distributions: 0,
        qbi_records: Some(K1ScorpQbiRecords {
            ordinary: Default::default(),
            property: Default::default(),
            property_complete: true,
        }),
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
        participation: Some(Participation::Investor { prior_years: 0 }),
        more: k1_1120s_more(),
        basis: K1ScorpBasis {
            start: K1ScorpStart::New,
            additions: 0,
            loans: vec![],
            more_at_risk: k1_1120s_all_at_risk(),
        },
        amt: Some(ScorpAmtRecords::SameAsRegular),
        special: Vec::new(),
    }
}
fn k1_1120s_more() -> K1ScorpOtherBoxes {
    K1ScorpOtherBoxes {
        calls: crate::K1ScorpCalls::default(),
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
        opening: Some(opening),
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
    let reply = crate::tests::support::client::income_pages::save_k1_sources_and_answers(
        &mut session,
        pages
            .into_iter()
            .map(|p| {
                document(0, IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) })
            })
            .collect(),
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
    page.participation = Some(Participation::Over500Hours { hours: 501 });
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
        printed: None,
        name: name.into(),
        reporting_period: None,
        qbi_group: alloc::string::String::new(),
        disposition: Some(PartnerDisposition::Kept),
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
            participation: None,
            real_estate_qbi: None,
            other_qbi: None,
            carryover_active: true,
        },
        publicly_traded: false,
        participation: Some(crate::PartnerParticipation::General {
            facts: Participation::Investor { prior_years: 0 }.record(),
        }),
        limited_for_se: Some(false),
        liabilities_start: 0,
        liabilities_end: 0,
        ordinary: 0,
        services: 0,
        capital: 0,
        se: 0,
        se_gross: SeGross::None,
        distributions: 0,
        qbi_records: Some(PartnerQbiRecords { qualified_se_gross: 0, history: None }),
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
        amt: Some(PartnerAmtRecords::SameAsRegular),
        special: Vec::new(),
    }
}
pub(in crate::tests) fn k1_1065_more() -> K1PartnershipOtherBoxes {
    K1PartnershipOtherBoxes {
        calls: crate::K1PartnershipCalls::default(),
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
    BasisStart::Continuing { opening: Some(opening), risk_history, carried: k1_1065_carried_zero() }
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
    let reply = crate::tests::support::client::income_pages::save_k1_sources_and_answers(
        &mut session,
        pages
            .into_iter()
            .map(|p| {
                document(
                    0,
                    IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) },
                )
            })
            .collect(),
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
    accepted(&{
        let request = save_topic(
            session,
            TaxSection::Retirement,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Form8606(Form8606 {
                        adjustment: 0,
                        reservist_repayments: vec![],
                        basis: 100_000,
                        year_end,
                    }),
                },
            )],
        );
        send(session, request)
    });
}

// C19, C21: defaults and inline choices declared on the page types.
#[test]
fn form_8862_groups_default_to_never_and_retirement_plans_are_inline() {
    let form_8862 = include_str!("../../forms/disallowance_source.rs");
    for field in ["earned_income", "child_credits", "education_credit"] {
        assert!(
            form_8862.contains(&alloc::format!("#[libertas_default(Never)]\n    pub {field}:")),
            "{field}"
        );
    }
    let model = crate::federal_model::HUB_SOURCE;
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
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
        TaxSection::Income,
    ];
    let mut s = Interview::new(TEST_USER, d.clone());
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut s,
        pages
            .into_iter()
            .map(|p| {
                document(
                    0,
                    IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) },
                )
            })
            .collect(),
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
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
        estimate: None,
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
        (crate::federal_model::HUB_SOURCE, "farmer: Farmer,", "allowed_farmer"),
        (
            crate::federal_model::HUB_SOURCE,
            "pub income_source: crate::BusinessIncomeChoice,",
            "allowed_business_source",
        ),
        (
            include_str!("../../farm/farm_page.rs"),
            "pub income_source: FarmIncomeSource,",
            "allowed_farm_source",
        ),
        (
            include_str!("../../federal/document_center.rs"),
            "income_source: crate::K1IncomeSource,",
            "^<DocumentQuestion>.k1.source_choices",
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
fn disaster_r1099(amount: i64, earlier: i64, spread: DisasterSpread) -> crate::R1099Answered {
    let mut source = r1099(
        3_000_000,
        DistributionCode::Early {
            what_happened: Disposition::Kept,
            exception: EarlyException::None,
        },
        false,
    );
    // Fixture answers are kept per printed paper: each designation prints its own payer.
    source.source_mut().form.payer = crate::PrintedParty::from_name(alloc::format!(
        "Disaster plan {amount} {earlier} {spread:?}"
    ));
    source.source_mut().disaster = Some(disaster_fold(amount, earlier, spread));
    source
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
    {
        crate::tests::support::client::income_pages::save_each_topic(
            &mut session,
            vec![
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(fold.clone())),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Form8915(earlier.clone()) },
                ),
            ],
        );
    };
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Retirement { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(&FederalRetirement::from(&stored).disaster, fold.disaster());
    assert_eq!(
        session.draft.screening.as_ref().unwrap().disaster_carries,
        vec![DisasterCarry { owner: Owner::Taxpayer, form: earlier.clone() }]
    );
    let mut restarted = restart(session.draft.clone());
    let page = open_topic(&mut restarted, TaxSection::Retirement);
    let documents: Vec<_> =
        page_documents(&page).iter().map(|item| item.document.clone()).collect();
    let source_page = open_document_center(&mut restarted);
    let expected = expected_named_sources(
        &restarted.draft,
        Owner::Taxpayer,
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(fold.clone())),
        }],
    );
    assert!(page_documents(&source_page).iter().any(|row| expected.contains(&row.document)));
    assert!(
        documents.contains(&IncomeSource::Record { value: crate::IncomeRecord::Form8915(earlier) })
    );
}

#[test]
fn qualified_disaster_rules_reject_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let reject = |documents: Vec<IncomeSource>, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = crate::tests::support::client::save_retirement_fixture(
            &mut session,
            documents.into_iter().map(|d| document(0, d)).collect(),
        );
        rejected_with(reply, key);
        assert_eq!(session.draft, original);
    };
    // Client rules of one 1099-R, checked again on save.
    let mut over_gross = disaster_r1099(2_000_000, 0, DisasterSpread::OverThreeYears);
    over_gross.source_mut().form.gross = 1_000_000;
    over_gross.source_mut().form.taxable = Some(1_000_000);
    reject(
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(over_gross)),
        }],
        "R_DISASTER_ABOVE_GROSS",
    );
    reject(
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
                2_000_000,
                200_001,
                DisasterSpread::OverThreeYears,
            ))),
        }],
        "DISASTER_ABOVE_LIMIT",
    );
    let mut over_repaid = disaster_r1099(1_000_000, 0, DisasterSpread::OverThreeYears);
    if let DisasterDistribution::Qualified { repaid, .. } =
        over_repaid.source_mut().disaster.as_mut().unwrap()
    {
        *repaid = crate::DisasterRepayment::Yes { total: 1_000_001, in_return_year: 0 };
    }
    reject(
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(over_repaid)),
        }],
        "DISASTER_REPAID_ABOVE_AMOUNT",
    );
    // Across the page's 1099-Rs (server only): one limit per disaster, one
    // line 1b amount and one election per disaster year.
    reject(
        vec![
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
                    2_000_000,
                    0,
                    DisasterSpread::OverThreeYears,
                ))),
            },
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
                    200_001,
                    0,
                    DisasterSpread::OverThreeYears,
                ))),
            },
        ],
        "DISASTER_ABOVE_LIMIT",
    );
    reject(
        vec![
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
                    100_000,
                    0,
                    DisasterSpread::OverThreeYears,
                ))),
            },
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
                    100_000,
                    0,
                    DisasterSpread::AllInReturnYear,
                ))),
            },
        ],
        "DISASTER_ELECTION_DIFFERS",
    );
    reject(
        vec![
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
                    100_000,
                    0,
                    DisasterSpread::OverThreeYears,
                ))),
            },
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disaster_r1099(
                    100_000,
                    5_000,
                    DisasterSpread::OverThreeYears,
                ))),
            },
        ],
        "DISASTER_EARLIER_DIFFERS",
    );
    // Earlier Form 8915-F rules.
    reject(
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Form8915(earlier_8915(Form8915Year::Filed2024, 2025)),
        }],
        "FORM_8915_DISASTER_AFTER_FORM",
    );
    let mut carried = earlier_8915(
        Form8915Year::Filed2025 { line_13: 200_000, line_14: 500_000, line_24: 0, line_25: 0 },
        2025,
    );
    carried.repayments = Form8915Repayments { plan_carried_back: 300_001, ..Default::default() };
    reject(
        vec![IncomeSource::Record { value: crate::IncomeRecord::Form8915(carried) }],
        "FORM_8915_CARRIED_BACK_ABOVE_EXCESS",
    );
    let twice = earlier_8915(Form8915Year::Filed2024, 2024);
    reject(
        vec![
            IncomeSource::Record { value: crate::IncomeRecord::Form8915(twice.clone()) },
            IncomeSource::Record { value: crate::IncomeRecord::Form8915(twice) },
        ],
        "FORM_8915_TWICE",
    );
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
        livestock: None,
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
    let work_opportunity = WorkOpportunityCredit {
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
    };
    b.more.work_opportunity = Some((&work_opportunity).into());
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
    let small_employer_health = SmallEmployerHealthCredit {
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
    };
    b.more.small_employer_health = Some((&small_employer_health).into());
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
        leave_hours: f64::from(hours) / 100.0,
        leave_wages: wages,
        rate: f64::from(rate) / 100.0,
    };
    let paid_leave = PaidLeaveCredit {
        employees: vec![
            employee("A", 16_000, 400_000, 10_000),
            employee("B", 8_000, 150_000, 7_500),
        ],
        insurance: Some(PaidLeaveInsurance { premiums: 600_000, rate: 60.0 }),
        confirmed: true,
    };
    b.more.paid_leave = Some((&paid_leave).into());
    b
}

#[test]
fn paid_leave_page_rules_are_checked_on_save() {
    // Exercise the actual native Save owner; declarations are server-only.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let edited = |edit: &dyn Fn(&mut crate::PaidLeaveCreditPage)| {
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
        save_businesses(&mut session, vec![edited(&|c| c.employees[0].leave_hours = 480.01)]),
        "PAID_LEAVE_ABOVE_12_WEEKS",
    );
    accepted(&save_businesses(&mut session, vec![edited(&|c| c.employees[0].leave_hours = 480.0)]));
    // Premiums only are accepted.
    accepted(&save_businesses(&mut session, vec![edited(&|c| c.employees.clear())]));
    // The credit needs the payroll.
    let mut b = paid_leave_business();
    b.more.payroll = BusinessPayroll::No;
    rejected_with(save_businesses(&mut session, vec![b]), "EMPLOYER_CREDITS_NEED_PAYROLL");
}

// T3-982: the Canceled debt topic, its Form 1099-C and Form 1099-A kinds.
pub(in crate::tests) fn car_1099c(amount: i64, treatment: CanceledDebtChoice) -> IncomeSource {
    IncomeSource::CanceledDebt {
        value: crate::CanceledDebtDocument::Cancel1099C(debt_document(personal_debt_source(
            Cancel1099C {
                creditor: crate::PrintedParty::from_name("Synthetic Auto Credit".into()),
                debtor: Default::default(),
                interest: None,
                debt_description: alloc::string::String::new(),
                event_code: alloc::string::String::new(),
                date: 20260315,
                amount,
                liable: true,
                property: PropertyTaken::None,
                treatment: Some(treatment),
            },
        ))),
    }
}
fn canceled_debt_documents() -> Vec<IncomeDocumentItem> {
    vec![
        document(
            0,
            IncomeSource::CanceledDebt {
                value: crate::CanceledDebtDocument::Cancel1099C(debt_document(
                    personal_debt_source(Cancel1099C {
                        creditor: crate::PrintedParty::from_name("Synthetic Auto Credit".into()),
                        debtor: Default::default(),
                        interest: None,
                        debt_description: alloc::string::String::new(),
                        event_code: alloc::string::String::new(),
                        date: 20260315,
                        amount: 150_000,
                        liable: true,
                        property: PropertyTaken::Taken {
                            value: 700_000,
                            property: Some(TransferredProperty::PersonalUse {
                                basis: 1_200_000,
                                term: GainTerm::Long,
                                collectible: false,
                            }),
                        },
                        treatment: Some(CanceledDebtChoice::Taxable),
                    }),
                )),
            },
        ),
        document(
            0,
            IncomeSource::CanceledDebt {
                value: crate::CanceledDebtDocument::Cancel1099C(debt_document(
                    personal_debt_source(Cancel1099C {
                        creditor: crate::PrintedParty::from_name("Synthetic Farm Credit".into()),
                        debtor: Default::default(),
                        interest: None,
                        debt_description: alloc::string::String::new(),
                        event_code: alloc::string::String::new(),
                        date: 20260601,
                        amount: 100_000,
                        liable: true,
                        property: PropertyTaken::None,
                        treatment: Some(CanceledDebtChoice::FarmDebt {
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
                        }),
                    }),
                )),
            },
        ),
        document(
            0,
            IncomeSource::CanceledDebt {
                value: crate::CanceledDebtDocument::Acquisition1099A(acquisition_document(
                    property_acquisition_source(Acquisition1099A {
                        lender: crate::PrintedParty::from_name("Synthetic Land Bank".into()),
                        borrower: Default::default(),
                        description: alloc::string::String::new(),
                        date: 20260801,
                        balance: 5_000_000,
                        value: 3_000_000,
                        liable: false,
                        property: Some(AcquiredProperty::Property {
                            kind: TransferredProperty::Investment {
                                basis: 4_000_000,
                                term: GainTerm::Long,
                                collectible: false,
                            },
                        }),
                    }),
                )),
            },
        ),
    ]
}

#[test]
fn canceled_debt_kinds_map_onto_records_and_reopen_unchanged() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(
        &({
            let request = save_documents(&mut session, canceled_debt_documents());
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        }),
    );
    let stored = &session.draft.income[1..];
    assert_eq!(
        stored.iter().map(|e| e.label.as_str()).collect::<Vec<_>>(),
        vec!["Synthetic Auto Credit", "Synthetic Farm Credit", "Synthetic Land Bank"]
    );
    // The printed value and the supplied transfer answer have separate homes.
    let FederalIncome::PersonalDebt { data: car } = &stored[0].income else { panic!() };
    assert_eq!(
        (
            PersonalDebtCancellation::from(car).personally_liable,
            PersonalDebtCancellation::from(car).box7_fmv
        ),
        (true, Some(700_000))
    );
    let FederalIncome::PersonalDebt { data: farm } = &stored[1].income else { panic!() };
    let Some(PersonalDebtTreatment::FarmDebt { data: facts }) =
        &PersonalDebtCancellation::from(farm).treatment
    else {
        panic!()
    };
    assert_eq!(facts.insolvency.as_ref().map(|i| i.liabilities), Some(1_100_000));
    let FederalIncome::PropertyAcquisition { data: land } = &stored[2].income else { panic!() };
    assert_eq!(
        (
            PropertyAcquisition::from(land).realized(),
            PropertyAcquisition::from(land).personally_liable
        ),
        (5_000_000, false)
    );
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
    let page = open_document_center(&mut session);
    assert_eq!(
        page_documents(&page)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::CanceledDebt { .. }))
            .collect::<Vec<_>>()
            .iter()
            .map(|r| r.document.clone())
            .collect::<Vec<_>>(),
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            canceled_debt_documents().into_iter().map(|r| r.document).collect::<Vec<_>>()
        )
    );
    let before = session.draft.income.clone();
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.income, before);
}

#[test]
fn canceled_debt_rules_reject_on_both_sides_at_their_boundaries() {
    let original = complete(FederalStatus::Single);
    let attempt = |document: IncomeSource| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            let request = save_documents(&mut session, vec![self::document(0, document)]);
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        };
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
        let request = save_documents(&mut session, vec![document(0, car_1099c(150_000, bad))]);
        rejected_with(
            {
                let desired = page_documents(&request);
                let mut candidate = restart(session.draft.clone());
                accepted(&send(&mut candidate, request));
                let request =
                    crate::tests::support::client::income_pages::information_return_answers_for_topic(
                        &mut candidate,
                        &desired,
                        TaxSection::CanceledDebt,
                    ).unwrap();
                send(
                    &mut candidate,
                    crate::tests::support::client::estimate_documents_from_save(&request),
                )
            },
            key,
        );
    }
}

#[test]
fn canceled_debt_refusals_are_stored_named_and_open_the_page() {
    // Something else is stored as the named situation and refused with its
    // form line; the Form 982 refusals open the Canceled debt topic, which
    // the checklist lists and the Less common income list no longer offers.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                car_1099c(
                    150_000,
                    CanceledDebtChoice::SomethingElse {
                        situation: UnsupportedDebt::PrincipalResidence,
                    },
                ),
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    let FederalIncome::PersonalDebt { data } = &session.draft.income[1].income else { panic!() };
    assert_eq!(
        PersonalDebtCancellation::from(data).treatment,
        Some(PersonalDebtTreatment::Unsupported { situation: UnsupportedDebt::PrincipalResidence })
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
        Some(TaxSection::Documents)
    );
    assert_eq!(
        crate::interview::issue_page(
            &session.draft,
            &crate::IssueContext::Section(TaxSection::CanceledDebt),
            None
        ),
        Some(TaxSection::CanceledDebt)
    );
    assert!(!Unprepared::all().iter().any(|item| item.refused().lines.contains("Canceled")));
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
        let (key, arguments) = status_key(&more.status);
        assert!(
            ["REVIEW_NONE", "REVIEW_NOT_STARTED"].contains(&key.as_str()) && arguments == 0,
            "{more:?}"
        );
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
    // A topic not reached yet reads as not started, never as needing attention.
    let coming = attention_status(&formatted("REVIEW_COMING_UP", &[]));
    assert_eq!(status_text(&coming), "Not started");
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
    ticked.checklist.paid = PaidTopics::IRA_CONTRIBUTIONS.0 | PaidTopics::CARE.0;
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
    let income = ALL_INCOME_TOPICS.to_vec();
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
    assert_eq!(s.draft.checklist.paid, 0);
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

// The IRA contributions page opened from the adjustments hub's row, saved
// with `people`; the hub it returns to.
fn iras_from_the_hub(s: &mut Interview, people: Vec<IraContributor>) -> P {
    let hub = open_adjustments_hub(s);
    let row = row_for(&hub, TaxSection::IraContributions);
    let Reply::Edit(mut request @ P::SaveIraContributions { .. }) = send(s, tap_row(&hub, row))
    else {
        panic!("expected the IRA contributions page")
    };
    let P::SaveIraContributions { value, .. } = &mut request else { unreachable!() };
    value.people = people;
    accepted(&send(s, request));
    open_adjustments_hub(s)
}
fn ira_row_status(hub: &P) -> alloc::string::String {
    status_text(&row_for(hub, TaxSection::IraContributions).status)
}
fn iras_ticked(s: &Interview) -> bool {
    s.draft.checklist.paid & PaidTopics::IRA_CONTRIBUTIONS.0 != 0
}

#[test]
fn an_entry_without_its_tick_sets_the_tick() {
    // Unchecked, the topic is still listed; a contribution saved through its
    // row checks the box, the row shows the entry, and the review has no item
    // for it.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert!(!iras_ticked(&s));
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(ira_row_status(&hub), "Not started");
    let hub = iras_from_the_hub(&mut s, vec![ira_row(Owner::Taxpayer, 300_000, None)]);
    assert!(iras_ticked(&s));
    assert_ne!(ira_row_status(&hub), "Not started");
    let (items, _, actions, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    assert!(actions.contains(&FINISH));
}

#[test]
fn a_tick_follows_its_entries_after_the_first_walk() {
    // Checked with nothing entered: "Planned", a soft reminder, never an
    // item needing review. An entry replaces it; removing the last entry
    // clears the box and the row reads "Not started".
    let mut d = complete(FederalStatus::Single);
    d.checklist.paid = PaidTopics::IRA_CONTRIBUTIONS.0;
    let mut s = Interview::new(TEST_USER, d);
    let hub = open_adjustments_hub(&mut s);
    assert_eq!(ira_row_status(&hub), "Planned");
    let (items, _, _, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
    let hub = iras_from_the_hub(&mut s, vec![ira_row(Owner::Taxpayer, 300_000, None)]);
    assert_ne!(ira_row_status(&hub), "Planned");
    assert!(iras_ticked(&s));
    let hub = iras_from_the_hub(&mut s, vec![]);
    assert!(!iras_ticked(&s));
    assert_eq!(ira_row_status(&hub), "Not started");
    let (items, _, _, _) = review_state(&mut s);
    assert!(items.is_empty(), "{items:?}");
}

#[test]
fn a_plan_navigates_the_first_walk_only() {
    // The first walk visits the planned topic; afterwards Next (resume)
    // never re-enters it for its box, though it stays planned. A box newly
    // checked is walked like a first-walk plan.
    let mut d = complete(FederalStatus::Single);
    d.finished_sections
        .retain(|s| !matches!(s, TaxSection::Adjustments | TaxSection::IraContributions));
    d.checklist.paid = PaidTopics::IRA_CONTRIBUTIONS.0;
    let mut s = Interview::new(TEST_USER, d);
    let Reply::Edit(request @ P::SaveIraContributions { .. }) =
        send(&mut s, P::OpenInterview { user: TEST_USER })
    else {
        panic!("the first walk opens the planned topic")
    };
    // Next with nothing entered: the walk passes the topic.
    accepted(&send(&mut s, request));
    assert!(s.draft.finished_sections.contains(&TaxSection::IraContributions));
    let draft = s.draft.clone();
    let reply = send(&mut s, go_to(&draft, PAGE_ADJUSTMENTS, TaxSection::Review));
    accepted(&reply);
    assert!(iras_ticked(&s));
    let reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    assert!(!matches!(reply, Reply::Edit(P::SaveIraContributions { .. })), "{reply:?}");
    // Unchecked and checked again: walked again.
    let draft = s.draft.clone();
    let reply = send(&mut s, navigate(&draft, TaxSection::Checklist));
    let value = checklist_of(&reply);
    let reply = send(&mut s, tick(reply, Checklist { paid: 0, ..value }));
    accepted(&reply);
    assert!(!iras_ticked(&s));
    let draft = s.draft.clone();
    let reply = send(&mut s, navigate(&draft, TaxSection::Checklist));
    let value = checklist_of(&reply);
    accepted(&send(
        &mut s,
        tick(reply, Checklist { paid: PaidTopics::IRA_CONTRIBUTIONS.0, ..value }),
    ));
    assert!(!s.draft.finished_sections.contains(&TaxSection::IraContributions));
    let reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    assert!(matches!(reply, Reply::Edit(P::SaveIraContributions { .. })), "{reply:?}");
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
        // An accepted tax-payment plan makes the itemized summary eligible
        // while automatic deductions and the unfinished income stay intact.
        let Reply::Edit(mut checklist @ P::SaveChecklist { .. }) =
            from_review(&mut session, TaxSection::Checklist)
        else {
            panic!("expected the checklist editor")
        };
        let P::SaveChecklist { value, .. } = &mut checklist else { unreachable!() };
        value.paid |= PaidTopics::TAXES.0;
        accepted(&send(&mut session, checklist));
        let d = session.draft.clone();
        let Reply::Edit(editor @ P::SaveDeductions { .. }) =
            send(&mut session, go_to(&d, PAGE_INCOME, TaxSection::Deductions))
        else {
            panic!("deductions must open before finishing income")
        };
        let mut editor = editor;
        if let P::SaveDeductions { value, .. } = &mut editor {
            value.itemize_override = None;
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
        let navigation = include_str!("../../core/shared/navigation.rs");
        let definition = &navigation[navigation.find("pub enum TaxSection").unwrap()
            ..navigation.find("pub enum NavigationDestination").unwrap()];
        assert!(
            definition.contains("Section 199A (199a), Form 8995 (8995) or Form 8995-A (8995-A)")
        );
        for description in definition.split("\n    /// ").skip(1) {
            assert!(!description.trim().is_empty());
        }
    }

    // The client refuses a search whose serialized context passes 64 KiB. The Review is the
    // largest offered set: a Go to leaf and an answers link for every section, each its own
    // candidate id. Under protocols.md "Candidate discovery and assembly before AI" and
    // "Navigation context supplied to AI" the context carries each destination's name and
    // description once, shared by the leaf and the link that open it; each candidate adds a
    // label, status, request description and JSON framing, allowed 200 bytes. Today's Studio
    // client still sends the description with each candidate (about twice), a conformance gap
    // tracked in the platform client-conformance backlog. Rows repeated under remaining items
    // or notes add ids only.
    #[test]
    fn the_review_search_context_stays_under_the_client_limit() {
        let source = include_str!("../../core/shared/navigation.rs");
        let body = &source[source.find("pub enum TaxSection").unwrap()
            ..source.find("pub enum NavigationDestination").unwrap()];
        let body = &body[..body.find("\n}").unwrap()];
        let (mut names, mut descriptions, mut count) = (0, 0, 0);
        let mut docs: Vec<&str> = Vec::new();
        for line in body.lines().map(str::trim) {
            if let Some(text) = line.strip_prefix("///") {
                docs.push(text.trim());
            } else if line.ends_with(',') && line.chars().next().is_some_and(|c| c.is_uppercase()) {
                names += docs[0].len();
                descriptions += docs[1..].join(" ").len();
                count += 1;
                docs.clear();
            }
        }
        assert!(count >= 40, "{count}");
        let prompt = crate::interview::resource_text("TAX_NAVIGATION_PROMPT").len();
        let candidates = 2 * count;
        let parts = [
            ("the navigation prompt", prompt),
            ("the section descriptions, once each", descriptions),
            ("the section names, once each", names),
            ("the per-candidate overhead", 200 * candidates),
        ];
        let total: usize = parts.iter().map(|(_, bytes)| bytes).sum();
        let largest = parts.iter().max_by_key(|(_, bytes)| bytes).unwrap();
        assert!(
            total < 64 * 1024,
            "the Review search context is about {total} bytes for {candidates} candidates; \
             the largest part is {} at {} bytes; parts: {parts:?}",
            largest.0,
            largest.1
        );
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
    let model = crate::federal_model::HUB_SOURCE;
    assert!(import_unselected(model, "pub struct DividendSource {"));
    assert!(
        !model.contains(
            "#[libertas_llm_doc_field]\n    pub reit_eligible: Option<DividendEligibility>"
        )
    );
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let dividend = {
        let mut source = crate::DividendSource::printed(dividend_document(Div1099 {
            section199a: 40_000,
            ..native_div1099("".into(), 100_000, 0)
        }));
        source.reit_eligible = Some(true.into());
        dividend_document(source)
    };
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend),
                },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    // The import: box 5 printed with a new amount, the answer untouched.
    let reopened = page_documents(&open_document_center(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
        .collect::<Vec<_>>();
    let IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Div1099(mut imported),
    } = reopened[0].document.clone()
    else {
        panic!("expected the 1099-DIV")
    };
    imported.ordinary = Some(120_000);
    imported.section199a = Some(50_000);
    let request = save_documents(
        &mut session,
        vec![document(
            reopened[0].id,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(imported),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let data = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Dividend { data } => Some(data),
            _ => None,
        })
        .expect("the dividend");
    assert_eq!(
        FederalDividend::from(data).section199a,
        ReitDividends::Reported { amount: 50_000, eligible: Some(true) }
    );
    let page = open_document_center(&mut session);
    let rows: Vec<_> = page_documents(&page)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
        .collect();
    let IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(again) } =
        &rows[0].document
    else {
        panic!("expected the 1099-DIV")
    };
    assert_eq!(
        session.draft.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::Dividend { data } => data.reit_eligible,
            _ => None,
        }),
        Some(true.into())
    );
    // Box 5 with no answer is refused on the server too.
    let unanswered = dividend_document(crate::DividendSource::printed(again.clone()));
    let request = save_documents(
        &mut session,
        vec![document(
            reopened[0].id,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(unanswered),
            },
        )],
    );
    let desired = page_documents(&request);
    accepted(&send(&mut session, request));
    let topic = crate::tests::support::client::income_pages::information_return_answers_for_topic(
        &mut session,
        &desired,
        TaxSection::InterestDividends,
    )
    .unwrap();
    rejected_with(send(&mut session, topic), "DIV_REIT_ELIGIBILITY_NEEDED");
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
        estimated: crate::EstimatedPaymentEntry::Sources {
            payments: vec![page_paid(20260415, 100_000)],
        },
        more: crate::PaymentsMore { penalty: choice, ..no_payments().more },
        ..no_payments()
    };
    accepted(&submit(&mut s, blank.clone()));
    let pay = s.draft.payments.clone().unwrap();
    assert_eq!(pay.prior_year.as_ref().map(|p| p.agi), Some(None));
    let page = open_payments(&mut s);
    assert_eq!(payments_value(&page).0.more.penalty, blank.more.penalty);
}

// Form 8815 on the Interest and dividends page (F7). The savings-bond record
// names each person by identity, the key of the page's people catalog, so
// reopening the page returns the same rows whatever the catalog's order.
#[test]
fn savings_bonds_page_round_trips_people_by_identity_and_checks_its_rules() {
    use crate::tests::investment::investment::bond_return;
    let mut d = bond_return(FederalStatus::Joint, 100_000, 3_000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().first_name = "Jordan".into();
    name_accepted_income_documents(&mut d);
    let child = d.dependents[0].id;
    // The page's 1099-INT replaces the fixture's statement.
    d.income.pop();
    let mut session = Interview::new(TEST_USER, d);
    let page = open_document_center(&mut session);
    let P::SaveDocuments { people, allowed_documents, .. } = &page else { unreachable!() };
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
        source_issuer: Default::default(),
        source_recipient: Default::default(),
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
    let interest_document = named_document(
        &session.draft,
        -2,
        0,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(int)),
        },
    );
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![interest_document],
    );
    accepted(&send(&mut session, request));
    let interest_row = page_documents(&open_document_center(&mut session))
        .iter()
        .find(|r| {
            matches!(
                r.document,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(_)
                }
            )
        })
        .unwrap()
        .clone();
    bonds.interest_statement = interest_row.id;
    bonds.source_recipient = accepted_document_names(&session.draft, Owner::Spouse).recipient;
    let documents = vec![
        interest_row,
        owned(
            crate::DocumentOwner::Spouses,
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
                    savings_bonds_source(bonds.clone()),
                )),
            },
        ),
    ];
    let request = save_documents(&mut session, documents);
    let reply = crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    // Stored by identity, in the page's order, owned by the spouse.
    let entry = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::SavingsBonds { .. }))
        .expect("the savings bond record");
    assert_eq!(entry.owner, crate::DocumentOwner::Spouses);
    assert_eq!(entry.label, "");
    let FederalIncome::SavingsBonds { data } = &entry.income else { unreachable!() };
    let data = data.natural().expect("supplied bond entry facts and answers");
    assert_eq!(data.students.iter().map(|s| s.person).collect::<Vec<_>>(), vec![child, -2, -1]);
    assert_eq!(data.proceeds, 1_200_000);
    assert_eq!(data.earliest_issue_date, 20100601);
    // The stored return reopens as the same page row.
    session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
    let page = open_document_center(&mut session);
    let reopened: Vec<_> = page_documents(&page)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
        .collect::<Vec<_>>()
        .iter()
        .filter_map(|item| match &item.document {
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::SavingsBonds(v),
            } => Some((item.owner.owned(), v.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        reopened,
        vec![(Owner::Spouse, savings_bonds_document(savings_bonds_source(bonds.clone())))]
    );
    // No amount is previewed: the exclusion depends on the whole return.
    assert!(
        page_documents(&page)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
            .collect::<Vec<_>>()
            .iter()
            .filter(|item| matches!(
                item.document,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::SavingsBonds(_)
                }
            ))
            .all(|item| item.preview.is_none())
    );

    // The page rules, on the server with their client keys.
    let reject = |bonds: SavingsBonds, twice: bool, key: &str| {
        let mut session = Interview::new(TEST_USER, session.draft.clone());
        let mut documents = vec![owned(
            crate::DocumentOwner::Yours,
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
                    savings_bonds_source(bonds.clone()),
                )),
            },
        )];
        if twice {
            documents.push(owned(
                crate::DocumentOwner::Yours,
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
                        savings_bonds_source(bonds),
                    )),
                },
            ));
        }
        let before = session.draft.clone();
        let reply = {
            let request = save_documents(&mut session, documents);
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        };
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
}
fn document_index_of_savings_bonds() -> i32 {
    libertas_macros::variant_index!(crate::IncomeSourceKind::SavingsBonds) as i32
}

// Form 8818 import: the AI import reads the three printed facts (line 6, line
// 8 and the earliest column (b) issue date); the owner, the earlier-return
// source link and the people stay the filer's.
#[test]
fn savings_bonds_import_reads_form_8818_and_leaves_personal_records() {
    let model = crate::federal_model::HUB_SOURCE;
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
    // 2026-10-23: printed identity is imported alongside the source boxes.
    assert_eq!(
        imported,
        vec![
            "source_issuer",
            "source_recipient",
            "proceeds",
            "redemption_interest",
            "earliest_issue_date"
        ]
    );
    assert_eq!(page.matches("(allow_empty = false)").count(), 3);
    // An import replaces exactly those fields of a row and keeps the rest.
    let mut row = SavingsBonds {
        source_issuer: Default::default(),
        source_recipient: Default::default(),
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
        payer: crate::PrintedParty::from_name(broker.into()),
        recipient: Default::default(),
        aggregate,
        withholding: 12_345,
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
        states: vec![crate::StateTaxWithholding {
            state: crate::TaxState::CA,
            payer_state_id: "0000123".into(),
            withholding: Some(777),
        }],
    };
    let blank = contracts_page("", 400_000);
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut s,
        vec![
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Contracts1099B(named.clone()),
                },
            ),
            owned(
                Owner::Taxpayer,
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Contracts1099B(blank.clone()),
                },
            ),
        ],
    );
    accepted(&send(&mut s, request));
    let stored = &s.draft.income;
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].label, "Example Futures Broker");
    assert_eq!(stored[1].label, "");
    // The stored source is the paper as printed (its parties and state rows
    // included) with the unanswered topic choice.
    let IncomeSource::InvestmentSale {
        value: crate::InvestmentSaleDocument::Contracts1099B(paper),
    } = expected_named_source(
        &s.draft,
        Owner::Taxpayer,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(named.clone()),
        },
    )
    else {
        unreachable!()
    };
    assert_eq!(
        stored[0].income,
        FederalIncome::Section1256 { data: crate::ContractsSource::printed(paper) }
    );
    assert_eq!(
        crate::FederalSection1256::from(match &stored[0].income {
            FederalIncome::Section1256 { data } => data,
            _ => unreachable!(),
        }),
        crate::FederalSection1256 {
            aggregate: -250_000,
            withholding: 12_345,
            special_treatment: false
        }
    );
    assert_eq!(
        stored[0].state_source,
        Some(crate::IncomeStateSource::Withholding(named.state_details.clone()))
    );
    assert_eq!(stored[1].state_source, None);
    // Reopened, the page reproduces both rows and saving them unchanged
    // reproduces the same records.
    let reopened = open_document_center(&mut s);
    let rows: Vec<_> = page_documents(&reopened)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
        .collect::<Vec<_>>()
        .iter()
        .map(|item| item.document.clone())
        .collect();
    assert_eq!(
        rows,
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Contracts1099B(named)
                },
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Contracts1099B(blank)
                }
            ]
        )
    );
    let before = s.draft.income.clone();
    let request = save_documents(
        &mut s,
        page_documents(&reopened)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
            .collect::<Vec<_>>()
            .to_vec(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income, before);
    // The preview is the Form 6781 line 1 amount; the carryback kind shows
    // no amount.
    let preview = |s: &mut Interview, document: IncomeSource| {
        crate::tests::support::client::income_pages::saved_document_amount(s, document)
    };
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Contracts1099B(contracts_page("", -250_000))
            }
        ),
        Some(-250_000)
    );
    // The printed amount is retained; its separately supplied topic answer
    // leaves this statement out of the supported estimate.
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Contracts1099B(contracts_document(
                    crate::ContractsSource { form: contracts_page("", 400_000), special: true },
                )),
            },
        ),
        Some(0)
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::Record {
                value: crate::IncomeRecord::Section1256Carryback(crate::Section1256Carryback {
                    amount: 1,
                    injured_owners: None
                })
            }
        ),
        None
    );
}

#[test]
fn contracts_1099b_import_reads_box_11_box_4_and_the_state_boxes_only() {
    // A printed Form 1099-B, contracts included, is imported as the complete
    // paper (broker_document.rs); the contracts and sale kinds that remain in
    // form_1099_b_model.rs are the filer's own records and import nothing.
    let records = include_str!("../../investment/form_1099_b_model.rs");
    for name in ["pub struct Contracts1099B {", "pub struct Sale1099B {"] {
        let head = &records[..records.find(name).unwrap()];
        let head = &head[head.rfind("\n}").unwrap_or(0)..];
        assert!(!head.contains("#[libertas_llm_doc_trans("), "{name} imports nothing");
    }
    let model = include_str!("../../investment/broker_document.rs");
    let start = model.find("pub struct Form1099B {").unwrap();
    let page = &model[start..];
    let page = &page[..page.find("\n}").unwrap()];
    let head = &model[..start];
    let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    assert!(
        action.contains("Boxes 8 through 11 and box 13 are independent actual printed amounts")
    );
    assert!(action.contains("Read box 11 exactly; never manufacture it from boxes 8 through 10"));
    assert!(action.contains("Return null for clearly blank withholding"));
    assert!(action.contains("Copy each populated state row once"));
    assert!(action.contains("Never infer an owner from the household"));
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
    // The printed identity, box 11 aggregate, box 4 withholding and the state
    // boxes are imported; the filer's hedge, election and dealer answer is
    // not on the paper.
    for field in ["payer", "recipient", "box_11_aggregate", "box_4_withholding", "state_details"] {
        assert!(imported.contains(&field), "{field} is imported");
    }
    assert!(!page.contains("special"));
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

pub(in crate::tests) fn roth_item_for_tests() -> crate::R1099Answered {
    let mut source = printed_as(r1099(1_000_000, DistributionCode::Nonqualified, false), "Q");
    source.source_mut().code = Some(DistributionCode::RothIra {
        sequence: None,
        rolled_over: 0,
        roth: RothIraCode::Qualified,
    });
    source
}

// Form 1099-Q and the 529 and Coverdell expenses record on the Prizes,
// royalties, alimony and 529 plans page. The page and the stored
// record name a student by identity, the key of the page's people catalog.
pub(in crate::tests) fn education_session() -> (Interview, i64) {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    let child = d.dependents[0].id;
    (Interview::new(TEST_USER, d), child)
}
pub(in crate::tests) fn q1099_page(
    gross: i64,
    account: EducationAccount,
    transfer: Q1099Transfer,
    beneficiary: Q1099Beneficiary,
) -> crate::EducationDistributionSource {
    crate::EducationDistributionSource {
        form: q1099_paper(Q1099 {
            payer_trustee: Default::default(),
            recipient: Default::default(),
            gross,
            account,
            transfer,
            recipient_not_beneficiary: !matches!(&beneficiary, Q1099Beneficiary::Recipient),
        }),
        beneficiary: Some((&beneficiary).into()),
        coverdell_basis_start: None,
        coverdell_contributions: 0,
        redeposited: 0,
        taxable_transfer_part: 0,
        roth_rollover: None,
        late_rollovers: None,
    }
}
pub(in crate::tests) fn plain_withdrawal() -> Q1099Transfer {
    Q1099Transfer::Withdrawal { reason: DistributionReason::Withdrawal }
}

// Form 1099-Q import: the AI import reads the printed boxes (1, 2, 3, 4a/4b,
// 5, 6, 7 and its code) and leaves the filer's records alone.
#[test]
fn q1099_import_reads_the_printed_boxes_and_leaves_personal_records() {
    let source = include_str!("../../family/education_account_source.rs");
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
    assert!(action.contains("Read box 6 into recipient_not_beneficiary exactly as checked"));
    assert!(action.contains("validation = \"$.document_facts\""));
    // Everything up to the college-cost facts, whose rows are never imported.
    let page = &page[..page.find("/// College tuition and required books").unwrap()];
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
            "payer_trustee",
            "recipient",
            "gross",
            "account",
            "transfer",
            "recipient_not_beneficiary",
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
        participation: Some(K1RentalParticipation::Active),
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
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![
                document(
                    0,
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::Int1099(interest_document(bank(
                            10_000,
                        ))),
                    },
                ),
                document(
                    0,
                    IncomeSource::InterestDividend {
                        value: crate::InterestDividendDocument::Int1099(interest_document(bank(
                            5_000,
                        ))),
                    },
                ),
            ],
        );
        send(&mut session, request)
    });
    // The amounts stay on the documents, so equal payers keep their own box 6;
    // the payee-statement taxes sum them.
    assert_eq!(crate::extended_rules::payee_statement_taxes(&session.draft), 15_000);
    let reopened: Vec<i64> =
        page_documents(&open_document_center(&mut restart(session.draft.clone())))
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InterestDividend { .. }))
            .map(|item| match &item.document {
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(v),
                } => v.foreign_tax_paid,
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
        TaxSection::IncomeWithoutForm,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Adjustments,
    ];
    let first = d.next_id;
    d.dependents.push(dependent_record(first, "Twin", "Sample", 20180101));
    d.dependents.push(dependent_record(first + 1, "Twin", "Sample", 20180101));
    d.next_id += 2;
    let mut s = Interview::new(TEST_USER, d);
    let sources = open_document_center(&mut s);
    accepted(&send(&mut s, sources));
    let mut planned = s.draft.often_missed.clone();
    planned.benefits |= MissedBenefits::MEDICAL.0;
    accepted(&save_often_missed(&mut s, planned));
    let page = from_review(&mut s, TaxSection::Deductions);
    assert!(matches!(page, Reply::Edit(P::SaveDeductions { .. })), "{page:?}");
    let request = save_deductions(&s.draft, None, 0);
    crate::tests::support::client::deductions::submit_to_itemized_summary(&mut s, request);
    let mut request = open_itemized_topic(&mut s, TaxSection::Medical);
    let P::SaveMedical { value, .. } = &mut request else { unreachable!() };
    assert_eq!(
        value.people.iter().map(|p| status_text(&p.label)).collect::<Vec<_>>(),
        vec![
            alloc::string::String::from("You"),
            alloc::string::String::from("Twin Sample"),
            alloc::string::String::from("Twin Sample")
        ]
    );
    value.expenses = 0;
    value.long_term_care = vec![CarePremium {
        qualified: CareContractQualification::Qualified,
        person: first + 1,
        premiums: 60_000,
    }];
    itemized_hub(send_past_less_common(&mut s, request));
    let stored = stored_facts(&s).long_term_care;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].person, first + 1);
    assert_eq!(stored[0].premiums, 60_000);
    // Reopening the page puts the premium on the second twin, not the first.
    let P::SaveMedical { value, .. } = open_itemized_topic(&mut s, TaxSection::Medical) else {
        panic!("expected the medical page")
    };
    assert_eq!(value.long_term_care.len(), 1);
    assert_eq!(value.long_term_care[0].person, first + 1);
    assert_eq!(value.long_term_care[0].premiums, 60_000);
}

// Entry display names: an unnamed income entry reads as its kind's
// capitalized noun over its identity, built in one owner.
#[test]
fn unnamed_income_entries_read_as_their_kind_noun_over_the_identity() {
    let mut d = complete(FederalStatus::Single);
    d.income[0].label = "".into();
    d.income.push(FederalIncomeEntry {
        printed: None,
        refund_management: None,
        state_source: Default::default(),
        id: 5,
        label: "".into(),
        owner: crate::DocumentOwner::Yours,
        income: cash_business(0),
        community: CommunityShare::NotNeeded,
    });
    d.income.push(FederalIncomeEntry {
        printed: None,
        refund_management: None,
        state_source: Default::default(),
        id: 6,
        label: "".into(),
        owner: crate::DocumentOwner::Yours,
        income: FederalIncome::Section1256 {
            data: contracts_source(crate::FederalSection1256 {
                aggregate: 100_000,
                withholding: 0,
                special_treatment: false,
            }),
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
    let request = save_topic(
        &mut s,
        TaxSection::InterestDividends,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Form8814(election_page(
                    child,
                    vec![ChildStatement::Int1099(election_interest(300_000).interest_source())],
                )),
            },
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
                answers: Vec::new(),
                credits: vec![MoreCredit::ForeignTax(claim.clone().try_into().unwrap())],

                documents: Vec::new(),
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
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(bank.clone())),
            },
        )],
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.credits.as_ref().unwrap().foreign_tax, ForeignTaxClaim::No);
    let request = save_more_credits(
        &s.draft.clone(),
        MoreCredits {
            answers: Vec::new(),
            documents: Vec::new(),
            credits: vec![MoreCredit::ForeignTax(claim.clone().try_into().unwrap())],
        },
    );
    accepted(&send(&mut s, request));
    // A 1099 with a foreign tax keeps the confirmed answer through a save.
    let mut bank = bank;
    bank.more.foreign_tax = 5_000;
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(bank)),
            },
        )],
    );
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
                answers: Vec::new(),
                credits: vec![MoreCredit::BusinessCredits(history.into())],

                documents: Vec::new(),
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
    save_whole_return(&mut s, SpouseSituations::NONRESIDENT_SPOUSE.0);
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
    let both = SpouseSituations::NONRESIDENT_SPOUSE.0 | YourSituations::DUAL_STATUS.0;
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
        reply @ (Reply::Edit(P::SaveMortgages { .. })
        | Reply::Response(P::BeginMortgages { .. })) => mortgage_editor(reply),
        other => panic!("expected mortgage editor, got {other:?}"),
    }
}
// Apply the fixture's explicit nonprinted answers through the accepted topic rows.
pub(in crate::tests) fn save_mortgage_answers(
    s: &mut Interview,
    answers: &[(MortgageMore, crate::MortgagePointTreatment)],
) -> Reply {
    let mut request = linked_mortgage_editor(s);
    let P::SaveMortgages { value, .. } = &mut request else { unreachable!() };
    assert_eq!(value.loans.len(), answers.len());
    for (row, (more, points)) in value.loans.iter_mut().zip(answers) {
        row.property = more.property.clone();
        // The origin is answered on the questions page and echoed read-only.
        if let Some(source) = s
            .draft
            .deductions
            .as_mut()
            .unwrap()
            .mortgage
            .iter_mut()
            .find(|source| source.id == row.id)
        {
            source.refund_origin = more.refund_origin.clone();
        }
        row.refund_origin = more.refund_origin.clone();
        row.details = more.details.clone().into();
        row.points = points.clone();
    }
    send(s, request)
}
pub(in crate::tests) fn save_linked_mortgage_rows(
    s: &mut Interview,
    loans: Vec<Mortgage1098>,
) -> Reply {
    fn apply(s: &mut Interview, loans: Vec<Mortgage1098>) -> Reply {
        let answers = loans
            .iter()
            .map(|loan| {
                let points = if loan.points == 0 {
                    crate::MortgagePointTreatment::None
                } else {
                    let accepted = s
                        .draft
                        .deductions
                        .as_ref()
                        .unwrap()
                        .mortgage
                        .iter()
                        .find(|accepted| accepted.id == loan.id);
                    match accepted.map(|accepted| mortgage_loan(accepted).points) {
                        Some(MortgagePoints::Amortized { term_months, ended, .. }) => {
                            crate::MortgagePointTreatment::Amortized { term_months, ended }
                        }
                        Some(MortgagePoints::Improvement { term_months, .. }) => {
                            crate::MortgagePointTreatment::Improvement { term_months }
                        }
                        _ => crate::MortgagePointTreatment::Purchase,
                    }
                };
                let mut more = loan.more.clone();
                if loan.refund == 0 {
                    more.refund_origin = None;
                }
                (more, points)
            })
            .collect::<Vec<_>>();
        let request = save_mortgages(
            s,
            crate::MortgageDocuments {
                documents: loans.into_iter().map(mortgage_document).collect(),
            },
        );
        let reply = send(s, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
        save_mortgage_answers(s, &answers)
    }
    // Preflight the two ordinary saves in a cold session so the fixture's
    // rejection keeps the caller's accepted source graph unchanged.
    let mut candidate = restart(s.draft.clone());
    let reply = apply(&mut candidate, loans.clone());
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return reply;
    }
    apply(s, loans)
}
pub(in crate::tests) fn linked_mortgage_rows(s: &mut Interview) -> Vec<Mortgage1098> {
    let P::SaveDocuments { documents, .. } = open_document_center(s) else { unreachable!() };
    documents
        .common
        .mortgages
        .documents
        .into_iter()
        .map(|document| {
            let source = s
                .draft
                .deductions
                .as_ref()
                .unwrap()
                .mortgage
                .iter()
                .find(|source| source.id == document.id)
                .expect("accepted mortgage source");
            mortgage_snapshot(source)
        })
        .collect()
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
        farm_rental_use: None,
        allowed_farm_rental_use: alloc::vec::Vec::new(),
        macrs_source: None,
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        // A prior election offers Records; the actual AMT balance stays unknown.
        allowed_amt_carry: vec![0],
        cost: 1_400_000,
        credit_reduction: 0,
        placed: 20230601,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class: AssetClass::Seven,
                history: AssetHistory::Continuing {
                    prior_depreciation: 1_400_000,
                    first_year_179: 1_400_000,
                    first_year_bonus: 0,
                    carryforward: 0,
                    mid_quarter: false,
                },
            },
        },
        business_percent: 40,
        earlier_business_percent: Some(100),
        acquired: None,
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
            id: 0,
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
    page.successor = Some(true);
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
    k1.participation = Some(Participation::Over500Hours { hours: 600 });
    let (mut s, reply) = save_k1_1120s(vec![k1]);
    accepted(&reply);
    s.draft.people.as_mut().unwrap().taxpayer.birth_date = 19810102;
    let mut wage = w2(1_100_000);
    wage.social_wages = 500_000;
    wage.social_tax = 31_000;
    wage.medicare_wages = 500_000;
    wage.medicare_tax = 7_250;
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wage)) },
        )],
    );
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

fn care_charge_document(printed: &str, reduction: i64) -> crate::R1099Answered {
    let mut source =
        printed_as(r1099(1_000_000, DistributionCode::QualifiedCareCharge, false), printed);
    source.source_mut().form.taxable = Some(0);
    source.source_mut().form.box_8a = Some(reduction);
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
            &mut session,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(source.clone())),
                },
            )],
        );
        accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        ));
        let entry = session
            .draft
            .income
            .iter()
            .find(|e| matches!(e.income, FederalIncome::Retirement { .. }))
            .unwrap();
        let id = entry.id;
        let FederalIncome::Retirement { data } = &entry.income else { unreachable!() };
        let data = FederalRetirement::from(data);
        assert_eq!(entry.owner, crate::DocumentOwner::Yours);
        assert_eq!(data.gross, source.gross);
        assert_eq!(data.kind, RetirementKind::Nonqualified);
        assert_eq!(
            (data.ordinary_distribution.clone(), data.box_8a),
            (Some(RetirementTreatment::QualifiedCareCharge), reduction)
        );
        assert!(!crate::care_benefit_return::annuity_source(
            &session.draft,
            id,
            Owner::Taxpayer,
            1
        ));
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
        let page = open_document_center(&mut session);
        let rows: Vec<_> = page_documents(&page)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
            .collect();
        assert_eq!(rows.len(), 1);
        let got = &rows[0];
        assert_eq!(got.id, id);
        assert_eq!(
            vec![(got.document).clone()],
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(source))
                }]
            )
        );
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
    v.source_mut().form.taxable = Some(1);
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.source_mut().form.ira = true;
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.source_mut().form.trump = true;
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.source_mut().form.box_3 = Some(1);
    cases.push(v.clone());
    v = care_charge_document("W", 0);
    v.source_mut().code = Some(DistributionCode::Nonqualified);
    cases.push(v.clone());
    v = care_charge_document("7W", 0);
    cases.push(v);
    for source in cases {
        assert!(DistributionCodes::treatment_rule(&source).is_some());
        let mut session = Interview::new(TEST_USER, original.clone());
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(source)),
                },
            )],
        );
        assert!(matches!(
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request
            ),
            Reply::Response(P::Problem { .. })
        ));
        assert_eq!(session.draft, original);
    }
    let mut session = Interview::new(TEST_USER, original);
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(care_charge_document(
                    "W", 70_000,
                ))),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
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
        let crate::RetirementSource::R1099(data) = data else { unreachable!() };
        match change {
            0 => {
                data.form.taxable = Some(1);
                data.taxable_method = Some(TaxableMethod::BoxTwoA);
            }
            1 => data.form.ira = true,
            2 => data.code = Some(normal_kept()),
            _ => {
                data.form.box_8a = Some(data.form.gross + 1);
                data.code = Some(DistributionCode::QualifiedCareCharge)
            }
        }
        if change == 2 {
            // A conflicting personal election is retained until the topic fixes it.
            assert!(forged.valid_stored());
            assert!(
                estimate(&forged)
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "R_CARE_CHARGE_SOURCE_INVALID")
            );
        } else {
            assert!(!forged.valid_stored(), "{change}");
        }
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
    withdrawal.source_mut().form.taxable_not_determined = true;
    withdrawal.source_mut().taxable_method = Some(TaxableMethod::AnnuityWithdrawal {
        data: crate::AnnuityDistribution {
            loan_pledge: false,
            related: vec![],
            value: 1_000_000,
            investment: 600_000,
            eligible: true,
        },
    });
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(withdrawal.clone())),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let baseline = supported(&session.draft);
    assert_eq!(return_line(&baseline, "5b"), 400_000);
    let mut documents = page_documents(&open_document_center(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect::<Vec<_>>();
    documents.push(document(
        0,
        IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(care_charge_document(
                "W", 70_000,
            ))),
        },
    ));
    let request = save_documents(&mut session, documents);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "5b"), 400_000);
    assert_eq!(result.tax, baseline.tax);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
}

#[test]
fn code_w_candidate_save_uses_the_same_actual_source_guards() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    assert_eq!(
        crate::tests::support::client::income_pages::saved_document_amount(
            &mut session,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(care_charge_document(
                    "6W", 70_000
                )))
            }
        ),
        Some(0)
    );
    let mut invalid = care_charge_document("W", 70_000);
    invalid.source_mut().form.taxable = Some(1);
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(invalid)),
            },
        )],
    );
    rejected_with(
        crate::tests::support::client::save_retirement_fixture(
            &mut session,
            page_documents(&request),
        ),
        "R_CARE_CHARGE_SOURCE_INVALID",
    );
    assert_eq!(session.draft, before);
    assert!(!session.dirty);
}

pub(in crate::tests) fn fill_care_provider(page: &mut P) {
    let P::SaveCare { value, .. } = page else { panic!("Care page") };
    let paid = value
        .qualifying_people
        .iter()
        .chain(value.new_people.iter())
        .map(|p| match p.care {
            CareCosts::Yes { expenses, .. } => expenses,
            CareCosts::No => 0,
        })
        .sum::<i64>();
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

include!("../passthrough/unit22_interview_registration.rs");
include!("../federal/unit47/interview_registration.rs");

#[test]
fn unit15_appraisal_catalog_normal_save_reopen_error_and_restart() {
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
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
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
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
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
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
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
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
    let request = save_topic(&mut session, TaxSection::Rentals, reopened);
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
        documents.push(owned(
            owner,
            0,
            IncomeSource::Record { value: crate::IncomeRecord::Rental(page) },
        ));
    }
    let mut session = Interview::new(TEST_USER, d);
    let mut request = save_topic(&mut session, TaxSection::Rentals, documents);
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
        reopened.iter().map(|entry| entry.owner.owned()).collect::<Vec<_>>(),
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
    let request = save_topic(&mut session, TaxSection::Rentals, reopened);
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
    spouse.details.real_estate_professional =
        Some(ProfessionalRentals { single_activity: true, over_500_hours: true }.into());
    let request = save_spouse(&session.draft.clone(), spouse);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.professional_elections(Owner::Taxpayer), vec![1]);
    assert_eq!(session.draft.professional_elections(Owner::Spouse), vec![0, 1]);
    // A fresh qualification on the other page supplies the normal false default.
    let mut you = filer(&session.draft.people.as_ref().unwrap().taxpayer);
    you.details.real_estate_professional =
        Some(ProfessionalRentals { single_activity: false, over_500_hours: false }.into());
    let request = save_you(&session.draft.clone(), you);
    let Reply::Edit(P::SaveSpouse { spouse, allowed_professional_elections, .. }) =
        send(&mut session, request)
    else {
        panic!("spouse editor");
    };
    assert_eq!(allowed_professional_elections, vec![0, 1]);
    assert_eq!(
        spouse.details.real_estate_professional.as_ref().unwrap().single_activity,
        SingleActivity
    );
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
    assert_eq!(
        you.details.real_estate_professional.as_ref().unwrap().single_activity,
        SingleActivity
    );
    let request = save_you(&session.draft.clone(), you);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved); // Hidden projected choice is not persisted twice.
    // Only the authoritative existing answer can turn off the annual election.
    let mut spouse = Filer::from(spouse);
    spouse.details.real_estate_professional.as_mut().unwrap().single_activity = SeparateActivities;
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
    you.details.real_estate_professional = None;
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
        d.income.last_mut().unwrap().owner = crate::DocumentOwner::from(owner);
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
    assert!(you.details.real_estate_professional.is_none());
    you.details.real_estate_professional =
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
                crate::K1Document::K1Partnership((page).into())
            } else {
                let mut page = k1_1120s("Synthetic Corp");
                page.basis.start = k1_1120s_continuing(50_000, 50_000);
                let K1ScorpStart::Continuing { risk_history, .. } = &mut page.basis.start else {
                    unreachable!()
                };
                *risk_history = history.clone();
                crate::K1Document::K1Scorp((page).into())
            };
            let mut session = Interview::new(TEST_USER, original.clone());
            let request =
                save_documents(&mut session, vec![document(0, IncomeSource::K1 { value: doc })]);
            let P::SaveDocuments { allowed_continuing_risk, .. } = &request else { unreachable!() };
            assert_eq!(*allowed_continuing_risk, vec![1, 2]);
            accepted(&if matches!(request, P::SaveDocuments { .. }) {
                crate::tests::support::client::income_pages::save_source_fixture_request(
                    &mut session,
                    request,
                )
            } else {
                send(&mut session, request)
            });
            let before = session.draft.clone();
            let mut reopened = open_topic(&mut session, TaxSection::PassThrough);
            let P::SaveIncomes { allowed_continuing_risk, followups, .. } = &mut reopened else {
                unreachable!()
            };
            assert_eq!(*allowed_continuing_risk, ScorpRiskStart::continuing_choices());
            let index = followups
                .iter()
                .position(|row| match &row.answer {
                    crate::DocumentAnswer::K1PartnerBasisStart { .. } => partnership,
                    crate::DocumentAnswer::K1ScorpBasisStart { .. } => !partnership,
                    _ => false,
                })
                .unwrap();
            // The basis start is its own required strict question, with the
            // continuing risk histories as its allowed values.
            assert_eq!(followups[index].allowed_values, ScorpRiskStart::continuing_choices());
            let restored = match &mut followups[index].answer {
                crate::DocumentAnswer::K1ScorpBasisStart {
                    value: Some(K1ScorpStart::Continuing { risk_history, .. }),
                } => risk_history,
                crate::DocumentAnswer::K1PartnerBasisStart {
                    value: Some(BasisStart::Continuing { risk_history, .. }),
                } => risk_history,
                _ => unreachable!(),
            };
            assert_eq!(*restored, history);
            *restored = ScorpRiskStart::New;
            // An echoed catalog cannot authorize an invalid answer on the server.
            *allowed_continuing_risk = vec![0];
            let path: alloc::string::String = libertas::SchemaPath::root()
                .field("followups")
                .key(followups[index].key.as_str())
                .field("answer")
                .field("value")
                .field("risk_history")
                .into();
            crate::tests::support::assert::k1res::assert_located_rejection(
                &mut session,
                reopened,
                if partnership { "PARTNER_RISK_HISTORY_NEW" } else { "SCORP_RISK_HISTORY_NEW" },
                &path,
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
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(k1) }));
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
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(corporation) }));
    let entity = d.income.last().unwrap().id;
    itemized(d.deductions.as_mut().unwrap()).real_estate_taxes = 1_000_000;
    // Hold the approved witness's itemized route fixed. Automatic otherwise
    // correctly compares the new research limit and can prefer standard.
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
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
    accepted(&{
        let mut request = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        documents.common.k1.documents.clear();
        send(&mut session, request)
    });
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
    name_accepted_income_documents(&mut d);
    let mut page = k1_1065("Synthetic actual Form3468 partnership");
    (page.participation, page.limited_for_se) = (
        Some(crate::PartnerParticipation::General {
            facts: (Participation::Over500Hours { hours: 600 }).record(),
        }),
        Some(false),
    );
    page.basis.contributions = 10_000_000;
    let (mut session, reply) = save_1065s(&d, vec![page]);
    accepted(&reply);
    if client.calculation(&mut session) {
        return;
    }
    let seed = open_document_center(&mut session);
    client.write("SEED", &seed);
    let mut answers = seed;
    let P::SaveDocuments { documents, .. } = &mut answers else { panic!() };
    let crate::K1Document::K1Partnership(page) = &mut documents.common.k1.documents[0].document
    else {
        panic!()
    };
    crate::tests::support::builders::push_partnership_credit(
        page,
        K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            code: K1CreditCode::Other,
            line: Some(BusinessCredit::DifferentialWage),
            amount: 100_000,
        },
    );
    crate::tests::support::builders::push_partnership_property(
        page,
        K1InvestmentProperty::Energy {
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
        },
    );
    page.personal.as_mut().unwrap().investment_confirmed =
        if matches!(profile.as_str(), "automatic" | "no") {
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
    let data = data.clone();
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
    let reopened = open_document_center(&mut session);
    client.write("REOPENED", &reopened);
    let P::SaveDocuments { documents, .. } = &reopened else { panic!() };
    let crate::K1Document::K1Partnership(page) = &documents.common.k1.documents[0].document else {
        panic!()
    };
    assert_eq!(
        page.calculation_page().more.allowed_investment_confirmations,
        if matches!(profile.as_str(), "automatic" | "no") { vec![0, 1] } else { vec![] }
    );
    assert_eq!(page.statements.credits.len(), 1);
    assert_eq!(page.statements.investment_property.len(), 1);
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
    documents.documents.push(
        crate::IncomeRecordItem::from_income(document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::NetOperatingLoss(LossCarryforward {
                    regular: vec![NolYearPage {
                        ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                        injured_owners: None,
                        year: 2024,
                        balance: 1_000_000,
                        allowed_research_parts: vec![0],
                        research_parts: Some(ResearchLossPartsPage::Parts {
                            sources: vec![ResearchLossPartPage {
                                source: entity,
                                amount: 1_000_000,
                            }],
                        }),
                    }],
                    amt: vec![],
                    source: LossSource::Business,
                }),
            },
        ))
        .expect("record fixture"),
    );
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
    let crate::IncomeRecord::NetOperatingLoss(loss) = &documents.documents.last().unwrap().document
    else {
        panic!()
    };
    assert_eq!(loss.regular[0].allowed_research_parts, vec![0]);
    assert_eq!(loss.regular[0].research_parts.is_some(), selected);
    let mut forged = reopened.clone();
    let P::SaveIncomes { documents, .. } = &mut forged else { panic!() };
    let crate::IncomeRecord::NetOperatingLoss(loss) =
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
    accepted(&{
        let mut request = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        documents.common.k1.documents.clear();
        send(&mut session, request)
    });
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
    documents.documents = vec![document(
        0,
        IncomeSource::Record { value: crate::IncomeRecord::Rental(page.clone()) },
    )]
    .into_iter()
    .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
    .collect();
    client.save(&mut session, answers);
    let saved = session.draft.clone();
    assert_eq!(value(&supported(&saved), "agi"), 123_442);
    // $16,000×23/660=$557.5758;
    // exact source cents round once to55,758c, then dollars on the form.
    assert_eq!(line_amount(&supported(&saved), "rental_2_depreciation"), 55_758);
    assert_eq!(displayed_line_amount(&supported(&saved), "rental_2_depreciation"), 55_800);
    let source = saved.income.last().unwrap().id;
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    client.write("REOPENED", &reopened);
    let IncomeSource::Record { value: crate::IncomeRecord::Rental(reopened_page) } =
        &page_documents(&reopened)[0].document
    else {
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
    let IncomeSource::Record { value: crate::IncomeRecord::Rental(page) } =
        &page_documents(&reopened)[0].document
    else {
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
    documents.documents = vec![document(
        0,
        IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(page) },
    )]
    .into_iter()
    .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
    .collect();
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
    let IncomeSource::Record { value: crate::IncomeRecord::CapitalLossCarryover(page) } =
        &page_documents(&reopened)[0].document
    else {
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
    let mut removed = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut removed else { unreachable!() };
    documents.common.k1.documents.clear();
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
    name_accepted_income_documents(&mut d);
    let document_value = if profile == "scorp" {
        let mut page = k1_1120s("Actual S corporation credit statement");
        page.interest = 1_000_000;
        page.participation = Some(Participation::Over500Hours { hours: 600 });
        page.basis.additions = 10_000_000;
        IncomeSource::K1 { value: crate::K1Document::K1Scorp((page).into()) }
    } else {
        assert_eq!(profile, "partnership");
        let mut page = k1_1065("Actual partnership credit statement");
        page.more.interest = 1_000_000;
        (page.participation, page.limited_for_se) = (
            Some(crate::PartnerParticipation::General {
                facts: (Participation::Over500Hours { hours: 600 }).record(),
            }),
            Some(false),
        );
        page.basis.contributions = 10_000_000;
        IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }
    };
    let mut session = Interview::new(TEST_USER, d.clone());
    accepted(&{
        let request = save_documents(&mut session, vec![document(0, document_value)]);
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    if client.calculation(&mut session) {
        return;
    }
    let seed = open_document_center(&mut session);
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
    let P::SaveDocuments { documents, .. } = &mut answers else { panic!() };
    match &mut documents.common.k1.documents[0].document {
        // The rental credits make participation a required answer.
        crate::K1Document::K1Partnership(page) => {
            crate::tests::support::builders::set_partnership_credits(page, &expected);
            page.personal.as_mut().unwrap().rental_participation =
                Some(crate::K1RentalParticipation::NotActive);
        }
        crate::K1Document::K1Scorp(page) => {
            crate::tests::support::builders::set_scorp_credits(page, &expected);
            let mut personal = crate::tests::support::builders::scorp_fixture_source(page).personal;
            personal.rental_participation = Some(crate::K1RentalParticipation::NotActive);
            *page =
                crate::tests::support::builders::remember_scorp_document(page.clone(), personal);
        }
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
        FederalIncome::Partnership { data } => data.clone().credits,
        FederalIncome::Scorp { data } => ScorpK1::from(data).credits,
        _ => panic!(),
    };
    assert_eq!(credits, expected);
    let saved = client.restart(&mut session);
    assert_eq!(supported(&session.draft), result);
    let reopened = open_document_center(&mut session);
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
        let P::Review { result, remaining_items, .. } = review else { panic!() };
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(remaining_items.iter().any(|item| matches!(
            status_message(&item.status).args.get(1),
            Some(MessageArg::Nested(issue)) if issue.key == "ISSUE_CREDIT_CARRYBACK_LIMIT"
        )));
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

#[path = "../federal/document_center.rs"]
mod document_center;

#[test]
fn no_form_income_records_have_their_own_summary_and_correction_labels() {
    use crate::tests::support::builders::native_income::cash_tips_without_w2;
    for (source, topic, summary, label) in [
        (
            cash_tips_without_w2(
                vec![CashTipMonth { month: 1, received: 100_000, reported: 0 }],
                TipCoverage::SocialAndMedicare,
                true,
            ),
            TaxSection::Wages,
            "1 cash tip record · $1000.00",
            "Cash tip record",
        ),
        (
            IncomeSource::Record {
                value: IncomeRecord::PrivateLoanInterest(PrivateLoanInterest {
                    borrower: "Synthetic Borrower".into(),
                    interest: 100_000,
                    seller_mortgage: None,
                }),
            },
            TaxSection::InterestDividends,
            "1 private loan interest record · $1000.00",
            "Private loan interest",
        ),
    ] {
        let mut session = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
        let request = save_topic(&mut session, topic, vec![document(0, source)]);
        accepted(&send(&mut session, request));
        let entry = &session.draft.income[0];
        assert_eq!(
            status_text(&crate::interview::topic_summary(&[entry], false).encode()),
            summary
        );
        // An unnamed source's correction still names its own record type.
        let mut unnamed = entry.clone();
        unnamed.label.clear();
        let crate::IssueContext::Income { id, label: actual } =
            crate::interview::income_issue_context(&unnamed)
        else {
            unreachable!()
        };
        assert_eq!(id, entry.id);
        assert_eq!(status_text(&actual.encode()), alloc::format!("{label} {}", entry.id));
    }
}
