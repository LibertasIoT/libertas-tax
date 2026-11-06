use crate::tests::support::client::roundtrip_native_draft;
// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::support::client::unit50_interview_native::NativeInterviewCase;
use super::*;
use crate::interview::document_income;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, native_w2 as w2};
use super::support::client::income_pages::{
    open_documents, open_topic, page_documents, save_documents, save_each_topic, save_topic,
    topic_for,
};
use crate::tests::cross::interview::{
    PAGE_DEDUCTIONS, PAGE_ITEMIZED, deductions_value, form_3921, gambling_losses, go_to,
    go_to_from_review, hub_rows, import_unselected, interest, itemized_hub, navigate,
    no_rental_expenses, open_credits_topic, open_itemized_topic, refusal, review_result, row_of,
    send_past_less_common, w2g,
};

// The page's W-2 without the calculated follow-up choices the server adds.
fn uncalled(document: &IncomeSource) -> IncomeSource {
    document.clone()
}

// Each follow-up answer opens only when the W-2's boxes (or the return) call
// for it; a plain W-2 opens none.
#[test]
fn w2_follow_up_answers_open_only_when_the_boxes_call_for_them() {
    let mut d = complete(FederalStatus::Single);
    let calls = |d: &FederalDraft, w: &W2| {
        let mut draft = d.clone();
        let source =
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w.clone())) };
        draft.income[0].income = document_income(&source);
        draft.income[0].state_source = crate::IncomeStateSource::from_document(&source);
        let mut session = Interview::new(TEST_USER, draft);
        let P::SaveWagesAnswers { wages, .. } = open_topic(&mut session, TaxSection::Wages) else {
            panic!("the wage topic")
        };
        wages.into_iter().next().expect("accepted W-2 has its keyed answer row").answers.calls
    };
    assert_eq!(calls(&d, &w2(6_000_000)), W2Calls::default());
    let mut w = w2(6_000_000);
    w.more.allocated_tips = 1;
    let tipped = calls(&d, &w);
    assert_eq!((tipped.tips, tipped.occupation), (vec![0], vec![0, 1]));
    assert!(tipped.overtime.is_empty() && tipped.coverage.is_empty());
    let mut w = w2(6_000_000);
    w.more.box12 = vec![Box12Entry { code: W2Code::TT, amount: 1 }];
    w.more.statutory = true;
    w.state_details.nonqualified_plans = 1;
    let coded = calls(&d, &w);
    assert_eq!(
        (coded.overtime, coded.statutory_business, coded.deferred_pension),
        (vec![0, 1], vec![0], vec![0])
    );
    let mut church = w2(6_000_000);
    (church.social_wages, church.medicare_wages) = (0, 0);
    assert_eq!(calls(&d, &church).coverage, vec![1, 2, 3]);
    assert!(calls(&d, &w2(6_000_000)).disability.is_empty());
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    assert_eq!(calls(&d, &w2(6_000_000)).disability, vec![1]);
}

#[test]
fn wages_answers_save_reopen_and_restart_keep_the_printed_w2_separate() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Wage { data: WageSource::W2(source) } = &mut draft.income[0].income else {
        panic!("complete fixture has its wage source")
    };
    source.form.box12 = vec![crate::PrintedBox12Entry { code: W2Code::TT, amount: Some(100) }];
    let paper = source.form.clone();
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_topic(&mut session, TaxSection::Wages);
    let P::SaveWagesAnswers { wages, .. } = &mut page else { panic!("Wages editor") };
    wages[0].answers.overtime_flsa_required = Some(WageDeductionEligibility::Eligible);
    let submitted = wages[0].answers.clone();
    let before = session.draft.clone();
    let estimate = super::support::client::estimate_documents_from_save(&page);
    assert!(matches!(send(&mut session, estimate), Reply::Response(P::RunningEstimate { .. })));
    assert_eq!(session.draft, before, "a running estimate does not save answers");
    accepted(&send(&mut session, page));
    let mut session = restart(roundtrip_native_draft(session.draft));
    let P::SaveWagesAnswers { wages, .. } = open_topic(&mut session, TaxSection::Wages) else {
        panic!("reopened Wages editor")
    };
    assert_eq!(wages[0].answers, submitted);
    let FederalIncome::Wage { data: WageSource::W2(source) } = &session.draft.income[0].income
    else {
        panic!("accepted W-2")
    };
    assert_eq!(source.form, paper, "topic answers do not change printed boxes");
}

// The W-2 fold (box 8, box 12, box 13) maps onto FederalWage.
#[test]
fn w2_fold_maps_tips_box12_and_box13() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2_source(w2(6_000_000));
    w.more = W2More {
        payroll: W2Payroll::SocialSecurity,
        social_tips: 200_000,
        allocated_tips: 50_000,
        care_benefits: 0,
        box12: vec![
            Box12Entry { code: W2Code::A, amount: 10_000 },
            Box12Entry { code: W2Code::B, amount: 5_000 },
            Box12Entry { code: W2Code::TP, amount: 100_000 },
            Box12Entry { code: W2Code::TT, amount: 80_000 },
            Box12Entry { code: W2Code::D, amount: 300_000 },
        ],
        statutory: false,
        retirement_plan: true,
    };
    w.answers.tips_listed_occupation = Some(WageDeductionEligibility::Eligible);
    w.answers.overtime_flsa_required = Some(WageDeductionEligibility::Eligible);
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w.clone())) },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = &FederalWage::from(data);
    assert_eq!(data.social_tips, 200_000);
    assert_eq!(
        data.employee_tips,
        Some(EmployeeTips {
            cash: CashTipRecords::Allocated { amount: 50_000 },
            noncash: 0,
            coverage: TipCoverage::SocialAndMedicare,
        })
    );
    assert_eq!(
        data.uncollected_tax,
        Some(W2UncollectedTax {
            tip_social: 10_000,
            tip_medicare: 5_000,
            life_social: 0,
            life_medicare: 0,
        })
    );
    let facts = &data.work_deductions;
    assert_eq!(
        facts.tips_claim,
        TipsClaim::Yes {
            w2_tips: 100_000,
            // Form 4137 col. 1(c): the reported code TP amount plus the box 8
            // allocated tips (i1040gi).
            form4137_tips: 150_000,
            tips_eligible: Some(true),
        }
    );
    assert_eq!(
        facts.overtime_claim,
        OvertimeClaim::Yes { overtime: 80_000, overtime_eligible: Some(true) }
    );
    assert_eq!(data.other_box12, vec![W2CodeAmount { code: W2Code::D, amount: 300_000 }]);
    assert_eq!(data.care_benefits, 0);
    assert!(!data.statutory_employee);
    assert!(data.retirement_plan);
    // Reopening rebuilds the identical page, in the same box 12 order.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session)).to_vec();
    assert_eq!(
        uncalled(&documents[0].document),
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w)) }
        )
    );
}

// Box10 and statutory W-2 facts survive saving and restart; both now calculate.
// With no qualifying person, the received care benefits become taxable.
#[test]
fn statutory_w2_and_care_benefits_are_stored_and_calculated() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2(6_000_000);
    w.more = W2More {
        payroll: W2Payroll::SocialSecurity,
        care_benefits: 40_000,
        statutory: true,
        ..Default::default()
    };
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w.clone())) },
            )],
        );
        send(&mut session, request)
    };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = &FederalWage::from(data);
    assert!(data.statutory_employee);
    assert_eq!(data.care_benefits, 40_000);
    let mut page = open_credits_topic(&mut session, TaxSection::Care);
    if let P::SaveCare { value, .. } = &mut page {
        value.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    }
    accepted(&send(&mut session, page));
    let r = estimate(&session.draft);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert!(!r.issues.iter().any(|i| i.message.contains("W-2 box 13")));
    assert!(noted(&r, "NOTE_CARE_NO_PROVIDER"));
    assert_eq!(value(&r, "care_benefits_taxable"), 400);
    assert_eq!(
        value(&r, &alloc::format!("statutory_wage_{}_net_profit", session.draft.income[0].id)),
        60000
    );
    // Reopening retains the independently accepted source document.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session)).to_vec();
    assert_eq!(
        uncalled(&documents[0].document),
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w)) }
        )
    );
}

#[test]
fn unemployment_fold_boxes_refuse_or_leave_out_by_name() {
    use crate::DocumentAnswer as A;
    let original = complete(FederalStatus::Single);
    // The printed boxes, then the topic answers: the grant is the filer's own,
    // and whether box 10 has a medical part or a special treatment applies.
    let unemployment_more = |rtaa: i64,
                             taxable_grants: i64,
                             agriculture_payment: i64,
                             market_gain: i64,
                             family_leave: i64,
                             family_leave_medical: bool,
                             special: bool| {
        let form = Form1099G {
            amount: 1_000_000,
            withholding: 100_000,
            rtaa,
            taxable_grants,
            agriculture_payment,
            market_gain,
            family_leave,
            ..Default::default()
        };
        let answers = [
            A::GovernmentGrantUse { value: Some(GrantUse::You) },
            A::GovernmentMedicalLeave { value: family_leave_medical },
            A::GovernmentSpecial { value: special },
        ];
        (form, answers)
    };
    let save = |session: &mut Interview, (form, answers): (Form1099G, [A; 3])| {
        let _answers =
            crate::tests::support::client::income_pages::unemployment_answers(answers.to_vec());
        let request = save_documents(
            session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    refund_state: None,
                    refund_locality: Default::default(),
                    value: government_document(form),
                },
            )],
        );
        let reply = crate::tests::support::client::income_pages::save_source_fixture_request(
            session, request,
        );
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    };
    let refused_for = |source, needle: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        save(&mut session, source);
        let r = estimate(&session.draft);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        if needle == "box 7 agriculture payments" {
            missing_related_activity(
                &session.draft,
                &r,
                crate::related_forms::DocumentKind::Form1099GAgriculture,
                &[TaxSection::Farming],
            );
        } else {
            assert!(r.issues.iter().any(|i| i.message.contains(needle)), "{r:?}");
        }
    };
    // Boxes 5, 7, 8, 9 and box 10 with a medical-leave portion, plus the
    // special checkbox, are each refused by name; none is calculated yet.
    // Box 5 RTAA is not refused: it reaches Schedule 1 line 8z.
    let mut session = Interview::new(TEST_USER, original.clone());
    save(&mut session, unemployment_more(1_000, 0, 0, 0, 0, false, false));
    let r = supported(&session.draft);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(value(&r, "other_income_2_rtaa"), 10);
    refused_for(unemployment_more(0, 0, 0, 1, 0, false, false), "box 9 market gain");
    refused_for(unemployment_more(0, 0, 1, 0, 0, false, false), "box 7 agriculture payments");
    // A medical-leave portion is taxed in full and the special checkbox leaves
    // the statement out, each with a notice.
    for (source, key) in [
        (unemployment_more(0, 0, 0, 0, 1, true, false), "WARN_FAMILY_LEAVE_TAXED_IN_FULL"),
        (unemployment_more(0, 0, 0, 0, 0, false, true), "WARN_UNEMPLOYMENT_LEFT_OUT"),
    ] {
        let mut session = Interview::new(TEST_USER, original.clone());
        save(&mut session, source);
        assert!(noted(&supported(&session.draft), key), "{key}");
    }
    // Box 6 taxable grants is not refused: it reaches Schedule 1 line 8z.
    let mut session = Interview::new(TEST_USER, original.clone());
    save(&mut session, unemployment_more(0, 25_000, 0, 0, 0, false, false));
    let r = supported(&session.draft);
    assert_eq!(value(&r, "taxable_grants_1099g"), 250);
    let ids = session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::GovernmentPayment { data } => Some(data.component_ids.clone()),
            _ => None,
        })
        .unwrap();
    assert!(ids.grant > 0);
    // Reopening restores the printed form unchanged.
    let (expected, _) = unemployment_more(0, 25_000, 0, 0, 0, false, false);
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::GovernmentPayment { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        documents[0].document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: government_document(expected)
            }
        )
    );
}

#[test]
fn unemployment_1099g_with_only_box_7_imports_and_reaches_the_farm() {
    // Form 1099-G with only box 7 (agriculture payments) printed: AI import
    // must not demand box 1. Box 1 is importable-empty (default 0) and box 7
    // is an import target; box 4 may exceed box 1 when box 7 carries the
    // payment. Import mapping: box 7 $800 -> agriculture_payments, box 1
    // $0 -> unemployment_compensation. The payment adds to the farm's Schedule F program income:
    // $10,000 raised + $800 = $10,800 gross.
    let model = include_str!("../../payments/form_1099_g_model.rs");
    let page = &model[model.find("pub struct Form1099GPaper {").unwrap()..];
    let page = &page[..page.find("\n}").unwrap()];
    let page = page
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let printed_amount = |field: &str| {
        alloc::format!(
            "#[libertas_llm_doc_field(blank = 0)]\n    #[libertas_money(\"USD\", 2)]\n    #[libertas_number(min = 0, max = 100000000000)]\n    #[libertas_default(0)]\n    pub {field}: i64"
        )
    };
    assert!(page.contains(&printed_amount("unemployment_compensation")), "{page}");
    // A blank box 7 stays blank (null); a printed zero stays zero.
    assert!(
        page.contains(
            "#[libertas_llm_doc_field(blank = $null)]\n    #[libertas_money(\"USD\", 2)]\n    #[libertas_number(min = 0, max = 100000000000)]\n    #[libertas_default($null)]\n    pub agriculture_payments: Option<i64>"
        ),
        "{page}"
    );
    assert!(model.contains("box 7 agriculture payments"));
    assert!(model.contains("returning zero for each of these boxes that is clearly blank"));
    assert!(model.contains("returning null for each of these boxes that is clearly blank"));

    let original = complete(FederalStatus::Single);
    let doc = Form1099G { withholding: 8_000, agriculture_payment: 80_000, ..Default::default() };
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    refund_state: None,
                    refund_locality: Default::default(),
                    value: government_document(doc.clone()),
                },
            )],
        );
        send(&mut session, request)
    };
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::GovernmentPayment { data } => Some(data.clone()),
            _ => None,
        })
        .expect("stored 1099-G");
    assert_eq!(
        (
            stored.form.unemployment_compensation,
            stored.form.agriculture_payments,
            stored.form.federal_income_tax_withheld
        ),
        (0, Some(80_000), 8_000)
    );
    assert!(stored.component_ids.agriculture > 0);
    let mut with_farm = session.draft.clone();
    add(&mut with_farm, income(raised_farm(10_000, 3_000)));
    let farm_id = with_farm.income.last().unwrap().id;
    let r = supported(&with_farm);
    assert_eq!(value(&r, &alloc::format!("farm_{farm_id}_gross_income")), 10_800);
    // Reopening restores the document unchanged.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::GovernmentPayment { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        documents[0].document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: government_document(doc)
            }
        )
    );
    // Box 4 above every printed payment is still refused.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = {
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::GovernmentPayment {
                    refund_state: None,
                    refund_locality: Default::default(),
                    value: government_document(Form1099G {
                        withholding: 80_001,
                        agriculture_payment: 80_000,
                        ..Default::default()
                    }),
                },
            )],
        );
        send(&mut session, request)
    };
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

// The direct-to-charity answer on the Other income prize kind is the record's:
// only Yes excludes the prize. No includes it, and the page reopens with the answer given.
#[test]
fn prize_page_keeps_the_direct_to_charity_answer() {
    for (eligible, taxable) in [(true, 0), (false, 500)] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let prize = OtherIncomeItem {
            kind: OtherIncomeKind::Prize {
                noncash: 0,
                disposition: PrizeDisposition::DirectCharity { eligible },
            },
            amount: 50_000,
            withholding: 0,
        };
        accepted(&{
            let request = save_topic(
                &mut session,
                TaxSection::OtherIncome,
                vec![document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(prize.clone()) },
                )],
            );
            send(&mut session, request)
        });
        let FederalIncome::OtherIncome { data } = &session.draft.income[1].income else {
            panic!("expected the prize record")
        };
        assert_eq!(
            data.kind,
            OtherIncomeKind::Prize {
                noncash: 0,
                disposition: PrizeDisposition::DirectCharity { eligible },
            }
        );
        let r = supported(&session.draft);
        assert_eq!(value(&r, "taxable_prizes"), taxable);
        assert!(
            page_documents(&open_topic(
                &mut session,
                topic_for(&IncomeSource::Record {
                    value: crate::IncomeRecord::OtherIncome(prize.clone())
                })
            ))
            .iter()
            .any(|item| item.document
                == IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(prize.clone()) })
        );
    }
}

#[test]
fn prize_royalty_and_equipment_rent_reach_their_lines() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let prize = OtherIncomeItem {
        kind: OtherIncomeKind::Prize { noncash: 0, disposition: PrizeDisposition::Received },
        amount: 50_000,
        withholding: 0,
    };
    let royalty = RoyaltyProperty {
        related: crate::RelatedForms::default(),
        kind: crate::RoyaltyKind::Intangible,
        receipts: 200_000,
        advance: 0,
        expenses: RentalExpenses { legal: 30_000, ..no_rental_expenses() },
        depletion: crate::RoyaltyDepletion::No,
        special: false,
    };
    let equipment_rent = EquipmentRent {
        related: crate::RelatedForms::default(),
        receipts: 150_000,
        expenses: 20_000,
        basis: 500_000,
        depreciable_basis: 500_000,
        assets: vec![],
        history: PersonalRentalHistory::NONE,
    };
    {
        let request = save_topic(
            &mut session,
            TaxSection::OtherIncome,
            vec![
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(prize.clone()) },
                ),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::EquipmentRental(equipment_rent.clone()),
                    },
                ),
            ],
        );
        send(&mut session, request)
    };
    let req = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::Royalties(royalty.clone()) },
        )],
    );
    accepted(&send(&mut session, req));
    let records: Vec<_> = session.draft.income[1..].iter().map(|e| &e.income).collect();
    assert_eq!(records[0], &prize_income(50_000, 0, 0, PrizeDisposition::Received));
    assert_eq!(records[2], &super::royalty(2_000, 300));
    assert_eq!(
        records[1],
        &FederalIncome::NonbusinessProperty {
            kind: NonbusinessPropertyKind::PersonalRental {
                related: alloc::vec![],
                basis: 500_000,
                depreciable_basis: 500_000,
                assets: vec![],
                history: PersonalRentalHistory::NONE,
            },
            receipts: 150_000,
            expenses: 20_000,
            ordinary: true,
        }
    );
    // Schedule 1 line 8i ($500), the royalty net ($2,000 - $300 = $1,700) and
    // the personal-property rent gross ($1,500).
    let r = supported(&session.draft);
    assert_eq!(value(&r, "taxable_prizes"), 500);
    assert_eq!(value(&r, "nonbusiness_royalty_income"), 1_700);
    assert_eq!(value(&r, "nonbusiness_personal_rent"), 1_500);
    // Reopening rebuilds all three documents.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_topic(&mut session, TaxSection::OtherIncome)).to_vec();
    assert_eq!(
        documents[0].document,
        IncomeSource::Record { value: crate::IncomeRecord::OtherIncome(prize) }
    );
    assert_eq!(
        documents[1].document,
        IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(equipment_rent) }
    );
    let rentals = page_documents(&open_topic(&mut session, TaxSection::Rentals)).to_vec();
    assert_eq!(
        rentals[0].document,
        IncomeSource::Record { value: crate::IncomeRecord::Royalties(royalty) }
    );
}

#[test]
fn wages_save_keeps_the_interest_documents() {
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let dividend = document(
        0,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(native_div1099(
                "".into(),
                10_000,
                5_000,
            ))),
        },
    );
    save_each_topic(
        &mut s,
        vec![
            document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(4_000_000))) },
            ),
            interest(20_000),
            dividend,
        ],
    );
    let accepted = s.draft.clone();
    assert_eq!(accepted.income.len(), 3);
    // A changed W-2 replaces the W-2 in place; the 1099s keep their records
    // and identities.
    let mut wages = page_documents(&open_documents(&mut s))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Wage { .. }))
        .collect::<Vec<_>>();
    if let IncomeSource::Wage { value: crate::WageDocument::W2(w) } = &mut wages[0].document {
        w.wages = Some(4_100_000);
        w.social_wages = Some(4_100_000);
        w.medicare_wages = Some(4_100_000);
    }
    {
        let request = save_documents(&mut s, wages);
        send(&mut s, request)
    };
    assert_eq!(s.draft.revision, accepted.revision + 1);
    assert_eq!(s.draft.income[1..], accepted.income[1..]);
    assert_eq!(s.draft.income[0].id, accepted.income[0].id);
    assert_ne!(s.draft.income[0], accepted.income[0]);
    // Removing every W-2 removes only the W-2.
    let changed = s.draft.clone();
    {
        let request = save_documents(&mut s, vec![]);
        send(&mut s, request)
    };
    assert_eq!(s.draft.income, changed.income[1..]);
    // A row cannot take over another topic's record.
    let mut stolen = document(
        0,
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(1_000_000))) },
    );
    stolen.id = changed.income[1].id;
    let current = s.draft.clone();
    error({
        let request = save_documents(&mut s, vec![stolen]);
        send(&mut s, request)
    });
    assert_eq!(s.draft, current);
}

// W-2 and wave-7/other AI Import documents use the same server-supplied year.
#[test]
fn wages_editor_supplies_hidden_w2_document_year() {
    let draft = empty_return(FederalStatus::Single);
    let mut interview = Interview::new(TEST_USER, draft);
    let P::SaveDocuments { topic, document_facts, .. } = open_documents(&mut interview) else {
        panic!("expected the Document center")
    };
    assert_eq!(topic, TaxSection::Documents);
    assert_eq!(document_facts.year, 2026);
}

fn sold_and_option_cost(sold: f64, option_cost: i64) -> Form3921More {
    Form3921More { sold, recognition: IsoRecognition::Exercise, option_cost }
}

#[test]
fn form_3921_kind_reaches_form_6251_line_2i() {
    // Single, W-2 wages $60,000 (AGI), standard deduction $16,100, taxable
    // income $43,900. Regular tax, 2026 Tax Table row 43,900-43,950 at its
    // $43,925 midpoint: 10% x $12,400 = $1,240 plus 12% x $31,525 = $3,783,
    // $5,023. Form 3921: 250 of the 1,000 shares sold in 2026 (the exercise
    // year, no adjustment for them, i6251 line 2i); 750 held at year-end.
    // Option cost $400 for the whole exercise, $300 allocated to the held
    // shares. Line 2i: 750 x ($210 - $10) = $150,000 - $300 = $149,700.
    // Form 6251 line 4 AMTI: $43,900 + $16,100 (line 2a standard deduction)
    // + $149,700 = $209,700. Exemption $90,100 (single, AMTI below the
    // $500,000 phaseout start), line 6 $119,600, 26% = $31,096 tentative
    // minimum tax (below the 28% band). AMT = $31,096 - $5,023 = $26,073;
    // total tax $5,023 + $26,073 = $31,096.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let exercise_id = s.draft.next_id;
    let form = form_3921(sold_and_option_cost(250.0, 40_000));
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form.clone()),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    let stored = &s.draft.screening.as_ref().unwrap().iso_exercises;
    assert_eq!(stored.len(), 1);
    let stored =
        stored.iter().map(|source| IsoExercise::try_from(source).unwrap()).collect::<Vec<_>>();
    assert_eq!(stored[0].label, "Synthetic Employer Inc");
    assert_eq!(stored[0].granted, 20240315);
    assert_eq!(stored[0].shares, 1000.0);
    assert_eq!(stored[0].sold, 250.0);
    assert_eq!(stored[0].option_cost, 40_000);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "regular_income_tax"), 5_023);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 209_700);
    assert_eq!(value(&r, "alternative_minimum_tax"), 26_073);
    assert_eq!(r.tax, Some(3_109_600));
    // It reopens as the same kind and owner, and saving it unchanged keeps
    // the stored exercise.
    let reopened = open_documents(&mut s);
    assert_eq!(
        page_documents(&reopened)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
            .collect::<Vec<_>>(),
        &[IncomeDocumentItem {
            refund_management_needed: None,
            ..document(
                exercise_id,
                expected_named_source(
                    &s.draft,
                    Owner::Taxpayer,
                    IncomeSource::InvestmentSale {
                        value: crate::InvestmentSaleDocument::Form3921(form.clone())
                    }
                )
            )
        }]
    );
    let before = s.draft.screening.clone();
    let request = save_documents(
        &mut s,
        page_documents(&reopened)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
            .collect::<Vec<_>>(),
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    assert_eq!(s.draft.screening, before);
    // Removing the form removes the adjustment.
    let mut request = open_documents(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.investment_sales.documents.clear();
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    assert!(s.draft.screening.as_ref().unwrap().iso_exercises.is_empty());
    assert_eq!(value(&supported(&s.draft), "alternative_minimum_taxable_income"), 60_000);
}

#[test]
fn form_3921_nothing_fold_round_trips_and_uses_box_4() {
    // Nothing else: all 1,000 shares held, no option cost, exercise-date
    // value. Line 2i 1,000 x $200 = $200,000; AMTI $260,000; less $90,100 =
    // $169,900 x 26% = $44,174; AMT $44,174 - $5,023 = $39,151.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let exercise_id = s.draft.next_id;
    let form = form_3921(Form3921More::default());
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form.clone()),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 260_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 39_151);
    // The fold reopens as Nothing exactly when its answers are the defaults.
    assert_eq!(
        page_documents(&open_documents(&mut s))
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
            .collect::<Vec<_>>(),
        &[IncomeDocumentItem {
            refund_management_needed: None,
            ..document(
                exercise_id,
                expected_named_source(
                    &s.draft,
                    Owner::Taxpayer,
                    IncomeSource::InvestmentSale {
                        value: crate::InvestmentSaleDocument::Form3921(form)
                    }
                )
            )
        }]
    );
    let replacement_exercise_id = s.draft.next_id;
    let something = form_3921(sold_and_option_cost(0.0, 1));
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(something.clone()),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    assert_eq!(
        page_documents(&open_documents(&mut s))
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
            .collect::<Vec<_>>(),
        &[IncomeDocumentItem {
            refund_management_needed: None,
            ..document(
                replacement_exercise_id,
                expected_named_source(
                    &s.draft,
                    Owner::Taxpayer,
                    IncomeSource::InvestmentSale {
                        value: crate::InvestmentSaleDocument::Form3921(something)
                    }
                )
            )
        }]
    );
}

#[test]
fn form_3921_every_share_sold_in_2026_has_no_adjustment() {
    // Boundary: sold equals box 5. A disposition in the exercise year makes
    // the regular and AMT treatment the same (i6251 line 2i), so AMTI is the
    // $60,000 of the return without the form and there is no AMT.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form_3921(sold_and_option_cost(
                    1000.0, 40_000,
                ))),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    // One millionth of a share more than box 5 is refused by its key.
    let before = s.draft.clone();
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let row = followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::IsoExerciseFacts { .. }))
        .unwrap();
    row.answer = crate::DocumentAnswer::IsoExerciseFacts {
        value: Some(sold_and_option_cost(1000.000001, 0)),
    };
    rejected_with(send(&mut s, request), "ISO_SOLD_ABOVE_TRANSFERRED");
    assert_eq!(s.draft, before);
}

#[test]
fn form_3921_exercise_on_the_grant_date_is_accepted_and_earlier_is_refused() {
    // Boundary: box 2 equal to box 1 is accepted; one day earlier is refused.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut form = form_3921(Form3921More::default());
    form.granted = form.exercised;
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form.clone()),
            },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    form.granted = 20260311;
    let before = s.draft.clone();
    rejected_with(
        {
            let request = save_documents(
                &mut s,
                vec![document(
                    0,
                    IncomeSource::InvestmentSale {
                        value: crate::InvestmentSaleDocument::Form3921(form),
                    },
                )],
            );
            send(&mut s, request)
        },
        "ISO_EXERCISED_BEFORE_GRANTED",
    );
    assert_eq!(s.draft, before);
}

#[test]
fn form_3921_recognition_must_fall_in_2026() {
    // Save the printed exercise before answering when its shares enter AMT income.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut form = form_3921(Form3921More::default());
    form.exercised = 20251231;
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form.clone()),
            },
        )],
    );
    accepted(&send(&mut s, request));
    let stored = &s.draft.screening.as_ref().unwrap().iso_exercises[0];
    assert_eq!(stored.form.exercised, 20251231);
    assert!(stored.more.is_none());
    assert!(s.draft.valid_stored());
    s = Interview::new(TEST_USER, roundtrip_native_draft(s.draft.clone()));
    let pending = estimate(&s.draft);
    assert!(pending.tax.is_none());
    assert_eq!(
        crate::interview::remaining_items(&s.draft, &pending, TaxSection::Review)
            .iter()
            .filter(|row| row.section == TaxSection::InvestmentSales)
            .count(),
        1,
    );
    // On exercise recognizes the December 31 exercise on the 2025 return.
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let row = followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::IsoExerciseFacts { .. }))
        .unwrap();
    assert!(matches!(row.answer, crate::DocumentAnswer::IsoExerciseFacts { value: None }));
    row.answer = crate::DocumentAnswer::IsoExerciseFacts { value: Some(Form3921More::default()) };
    let before = s.draft.clone();
    let message = refusal(send(&mut s, request));
    assert!(message.contains("Form 3921"), "{message}");
    assert!(message.contains("Form 6251 line 2i"), "{message}");
    assert_eq!(s.draft, before);
    // January 1, 2026 is an accepted recognition date once the answer is given.
    form.exercised = 20260101;
    let id = s.draft.screening.as_ref().unwrap().iso_exercises[0].id;
    let request = save_documents(
        &mut s,
        vec![document(
            id,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form.clone()),
            },
        )],
    );
    accepted(&send(&mut s, request));
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::IsoExerciseFacts { .. }))
        .unwrap()
        .answer = crate::DocumentAnswer::IsoExerciseFacts { value: Some(Form3921More::default()) };
    accepted(&send(&mut s, request));
    // Restricted shares exercised in 2025 and vesting on June 1, 2026 enter
    // AMT income at their $310 vesting value: 1,000 x ($310 - $10) =
    // $300,000; AMTI $360,000; less $90,100 = $269,900; 26% of $244,500 =
    // $63,570 plus 28% of $25,400 = $7,112, $70,682; AMT $70,682 - $5,023 =
    // $65,659.
    form.exercised = 20251231;
    let request = save_documents(
        &mut s,
        vec![document(
            id,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form.clone()),
            },
        )],
    );
    accepted(&send(&mut s, request));
    // The retained exercise answer needs review after the printed date changes.
    assert!(s.draft.valid_stored());
    assert!(estimate(&s.draft).tax.is_none());
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::IsoExerciseFacts { .. }))
        .unwrap()
        .answer = crate::DocumentAnswer::IsoExerciseFacts {
        value: Some(Form3921More {
            sold: 0.0,
            recognition: IsoRecognition::Vesting { date: 20260601, market_price: 31_000 },
            option_cost: 0,
        }),
    };
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 360_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 65_659);
    // A vesting date before box 2 is refused by its key on the topic answer.
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let crate::DocumentAnswer::IsoExerciseFacts { value: Some(more) } = &mut followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::IsoExerciseFacts { .. }))
        .unwrap()
        .answer
    else {
        unreachable!()
    };
    more.recognition = IsoRecognition::Vesting { date: 20251230, market_price: 31_000 };
    let before = s.draft.clone();
    rejected_with(send(&mut s, request), "ISO_VESTED_BEFORE_EXERCISED");
    assert_eq!(s.draft, before);
}

// The T2-44f documents as the filer enters them on the page: two W-2Gs and
// the return's losses, entered once.
fn t2_44f_documents() -> Vec<IncomeDocumentItem> {
    vec![
        document(0, w2g(500_000, 50_000)),
        document(0, w2g(350_000, 30_000)),
        document(0, gambling_losses(0, 1_000_000)),
    ]
}

#[test]
fn w2g_and_gambling_losses_map_onto_gambling_records_and_back() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    save_each_topic(&mut session, t2_44f_documents());
    // No gambling record is named; the Form W-2G kind is told from the
    // records kept without one by the record itself, never by a label.
    // The accepted W-2G prints the filer's name as winner: the records below name nobody.
    let stored: Vec<(&str, FederalIncome)> = session.draft.income[1..]
        .iter()
        .map(|e| {
            let mut income = e.income.clone();
            if let FederalIncome::Gambling { data: crate::GamblingSource::W2g(w) } = &mut income {
                w.form.winner = Default::default();
            }
            (e.label.as_str(), income)
        })
        .collect();
    let record = |winnings, losses, withholding, own_records| FederalIncome::Gambling {
        data: gambling_source(FederalGambling {
            winnings,
            losses,
            withholding,
            own_records,
            ordinary: true,
        }),
    };
    assert_eq!(
        stored,
        vec![
            ("", record(500_000, 0, 50_000, false)),
            ("", record(350_000, 0, 30_000, false)),
            ("", record(0, 1_000_000, 0, true)),
        ]
    );
    // Reopening rebuilds the same rows, each previewing its winnings, and
    // saving them unchanged stores the same records.
    let mut rows = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Gambling(_)))
        .collect::<Vec<_>>();
    rows.extend(page_documents(&open_topic(&mut session, TaxSection::Gambling)));
    assert_eq!(
        rows.iter().map(|r| r.document.clone()).collect::<Vec<_>>(),
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            t2_44f_documents().into_iter().map(|r| r.document).collect::<Vec<_>>()
        )
    );
    assert_eq!(
        rows.iter().map(|r| r.preview).collect::<Vec<_>>(),
        vec![Some(500_000), Some(350_000), Some(0)]
    );
    let before = session.draft.income.clone();
    save_each_topic(&mut session, rows.clone());
    assert_eq!(session.draft.income, before);
    // A ticked "Something else applies" is stored as not ordinary and
    // reopens ticked.
    let mut special = rows.clone();
    special[2].document = IncomeSource::Record {
        value: crate::IncomeRecord::GamblingLosses(GamblingLosses {
            other_winnings: 20_000,
            losses: 1_000_000,
            special: true,
        }),
    };
    special[0].document = IncomeSource::Gambling(w2g_document(W2g {
        payer: Default::default(),
        winner: Default::default(),
        state_details: Default::default(),
        winnings: 500_000,
        withholding: 50_000,
        special: true,
    }));
    save_each_topic(&mut session, special.clone());
    let mut reopened = page_documents(&open_documents(&mut session))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Gambling(_)))
        .collect::<Vec<_>>();
    reopened.extend(page_documents(&open_topic(&mut session, TaxSection::Gambling)));
    assert_eq!(
        reopened.iter().map(|r| r.document.clone()).collect::<Vec<_>>(),
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            special.into_iter().map(|r| r.document).collect::<Vec<_>>()
        )
    );
}

#[test]
fn gambling_topic_reaches_the_t2_44f_totals() {
    // The T2-44f hand fixture (tests/taxes.rs
    // itemizing_filer_with_two_w2gs_losses_above_winnings_and_withholding)
    // entered through the page. By hand: winnings $5,000 + $3,500 =
    // $8,500 (Schedule 1 line 8b); AGI $60,000 + $8,500 = $68,500; the loss
    // deduction is min($8,500, 90% x $10,000 = $9,000) = $8,500 (Schedule
    // A line 16); with $8,000 of property tax the itemized total $16,500
    // beats the $16,100 standard deduction; taxable income $52,000, tax
    // $6,158 (Tax Table midpoint $52,025); withholding $6,500 + $500 +
    // $300 = $7,300 (line 25c $800); overpayment $1,142.
    let mut original = complete(FederalStatus::Single);
    itemized(original.deductions.as_mut().unwrap()).personal_property_taxes = 800_000;
    let mut session = Interview::new(TEST_USER, original.clone());
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    save_each_topic(&mut session, t2_44f_documents());
    let r = supported(&session.draft);
    assert_eq!(value(&r, "agi"), 68_500);
    assert_eq!(value(&r, "gambling_winnings"), 8_500);
    assert_eq!(value(&r, "gambling_loss_deduction"), 8_500);
    assert_eq!(value(&r, "taxable_income"), 52_000);
    assert_eq!(value(&r, "income_tax"), 6_158);
    assert_eq!(return_line(&r, "25c"), 80_000);
    assert_eq!(return_line(&r, "25d"), 730_000);
    assert_eq!(return_line(&r, "34"), 114_200);
    // The review lists Schedule 1 line 8b and Schedule A line 17a (the 2026 form puts gambling there).
    let rows = review_result(session.draft.clone()).return_lines;
    for (label, amount) in [
        ("Schedule 1 8b — Gambling winnings", 850_000),
        ("Schedule A 17a — Gambling loss deduction", 850_000),
    ] {
        assert!(
            rows.iter().any(|r| status_text(&r.label) == label && r.amount == amount),
            "{label} missing from {rows:?}"
        );
    }
    // The income hub's row counts the kinds and adds their winnings.
    let Reply::Response(hub @ P::Hub { .. }) = go_to_from_review(&mut session, TaxSection::Income)
    else {
        panic!("expected the income hub")
    };
    assert!(hub_rows(&hub).contains(&row_of(
        "Gambling winnings",
        "2 W-2Gs, 1 gambling losses entry · $8500.00",
        TaxSection::Gambling
    )));
}

#[test]
fn w2g_withholding_above_winnings_is_rejected_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let attempt = |documents: Vec<IncomeDocumentItem>| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            let request = save_documents(&mut session, documents);
            send(&mut session, request)
        };
        (reply, session.draft)
    };
    // Box 4 equal to box 1 passes; one cent above is rejected with the
    // client rule's key and nothing is stored.
    accepted(&attempt(vec![document(0, w2g(100_000, 100_000))]).0);
    let (reply, draft) = attempt(vec![document(0, w2g(100_000, 100_001))]);
    rejected_with(reply, "W2G_WITHHOLDING_ABOVE_WINNINGS");
    assert_eq!(draft, original);
    let mut session = Interview::new(TEST_USER, original.clone());
    let request = save_documents(&mut session, vec![document(0, w2g(100_000, 100_001))]);
    rejected_with(
        crate::tests::support::client::income_pages::candidate_save_review(&mut session, request),
        "W2G_WITHHOLDING_ABOVE_WINNINGS",
    );
}

#[test]
fn gambling_losses_twice_is_rejected_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    save_each_topic(
        &mut session,
        vec![document(0, gambling_losses(10_000, 50_000)), document(0, w2g(100_000, 0))],
    );
    let mut session = Interview::new(TEST_USER, original.clone());
    rejected_with(
        {
            let request = save_topic(
                &mut session,
                TaxSection::Gambling,
                vec![
                    document(0, gambling_losses(10_000, 50_000)),
                    document(0, gambling_losses(0, 1)),
                ],
            );
            send(&mut session, request)
        },
        "GAMBLING_LOSSES_TWICE",
    );
    assert_eq!(session.draft, original);
}

#[test]
fn special_gambling_is_left_out_with_a_notice_under_the_page() {
    for kind in [
        IncomeSource::Gambling(w2g_document(W2g {
            payer: Default::default(),
            winner: Default::default(),
            state_details: Default::default(),
            winnings: 100_000,
            withholding: 0,
            special: true,
        })),
        IncomeSource::Record {
            value: crate::IncomeRecord::GamblingLosses(GamblingLosses {
                other_winnings: 100_000,
                losses: 0,
                special: true,
            }),
        },
    ] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        save_each_topic(&mut session, vec![document(0, kind)]);
        let r = supported(&session.draft);
        assert!(noted(&r, "WARN_GAMBLING_LEFT_OUT"), "{r:?}");
    }
}

#[test]
fn w2g_with_losses_keeps_the_page_closed() {
    // The W-2G has no loss box. The same winnings and losses now reopen
    // through their printed document and separate gambling-loss record.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    save_each_topic(
        &mut session,
        vec![document(0, w2g(100_000, 0)), document(0, gambling_losses(0, 10_000))],
    );
    let stored = session.draft.clone();
    let mut session = restart(stored.clone());
    let request = open_documents(&mut session);
    let rows = page_documents(&request);
    assert_eq!(
        rows.iter().filter(|row| matches!(row.document, IncomeSource::Gambling(_))).count(),
        1,
    );
    accepted(&send(&mut session, request));
    let request = open_topic(&mut session, TaxSection::Gambling);
    let rows = page_documents(&request);
    assert!(rows.iter().any(|row| row.document == gambling_losses(0, 10_000)));
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, stored);
    assert!(!session.dirty);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "gambling_winnings"), 1_000);
    // The retained $100 loss does not displace the standard deduction.
    assert_eq!(value(&result, "gambling_loss_deduction"), 0);
    assert_eq!(value(&result, "agi"), 61_000);
}

#[test]
fn gambling_losses_show_on_the_itemized_hub_and_open_compare() {
    // Losses entered on the income topic open the deductions page on
    // Compare before it is first saved, as an itemizable tick does (C7).
    let mut d = complete(FederalStatus::Single);
    d.finished_sections = vec![
        TaxSection::Children,
        TaxSection::Checklist,
        TaxSection::Income,
        TaxSection::Adjustments,
    ];
    d.income.push(FederalIncomeEntry {
        printed: None,
        refund_management: None,
        state_source: Default::default(),
        id: d.next_id,
        label: "".into(),
        owner: crate::DocumentOwner::Yours,
        income: document_income(&gambling_losses(200_000, 300_000)),
        community: CommunityShare::NotNeeded,
    });
    d.next_id += 1;
    let mut s = Interview::new(TEST_USER, d);
    crate::tests::support::client::admit_native_fixture_facts(&mut s);
    let source = open_documents(&mut s);
    accepted(&send(&mut s, source));
    let navigation =
        go_to(&s.draft, crate::tests::cross::interview::PAGE_ADJUSTMENTS, TaxSection::Deductions);
    let page = match send(&mut s, navigation) {
        Reply::Edit(page @ P::SaveDeductions { .. }) => page,
        other => panic!("expected the deductions page, got {other:?}"),
    };
    assert_eq!(deductions_value(&page).itemize_override, None);
    // Saved on Compare, the itemized hub shows the derived row, which opens
    // the income topic; its Back and save return to the hub.
    // The itemized walk asks More itemized deductions first.
    let reply = send_past_less_common(&mut s, page);
    let hub = match reply {
        Reply::Response(hub @ P::Hub { .. }) => hub,
        Reply::Edit(_) => {
            let request = go_to(&s.draft, PAGE_DEDUCTIONS, TaxSection::Itemized);
            itemized_hub(send_past_less_common(&mut s, request))
        }
        other => panic!("expected the itemized hub, got {other:?}"),
    };
    assert!(
        hub_rows(&hub).contains(&row_of(
            "Gambling losses",
            "$3000.00, from Gambling winnings",
            TaxSection::Gambling
        )),
        "{:?}",
        hub_rows(&hub)
    );
    let P::Hub { topics, .. } = &hub else { unreachable!() };
    let gambling = topics.items.iter().find(|item| item.section == TaxSection::Gambling).unwrap();
    let mut offset = 0;
    let encoded = <crate::common::FormattedMessage as libertas::AvroDecode>::avro_decode(
        &gambling.status,
        &mut offset,
    )
    .unwrap();
    assert_eq!(offset, gambling.status.len());
    assert_eq!(encoded.0, "REVIEW_GAMBLING_LOSSES_FROM");
    assert_eq!(
        encoded.1,
        vec![libertas::LibertasMessageArgumentDecode::Currency {
            currency_code: "USD".into(),
            scale: 2,
            minor_units: 300_000,
        }]
    );
    let page = open_itemized_topic(&mut s, TaxSection::Gambling);
    let P::SaveIncomes { previous, topic, .. } = &page else {
        panic!("expected the Gambling winnings page, got {page:?}")
    };
    assert_eq!((*previous, *topic), (PAGE_ITEMIZED, TaxSection::Gambling));
    itemized_hub(send_past_less_common(&mut s, page));
    // Without losses the derived local row is gone; global Go to still
    // opens the income topic so the filer can enter winnings.
    let mut d = complete(FederalStatus::Single);
    itemized(d.deductions.as_mut().unwrap()).personal_property_taxes = 800_000;
    let mut s = Interview::new(TEST_USER, d.clone());
    let hub = itemized_hub(send_past_less_common(&mut s, navigate(&d, TaxSection::Itemized)));
    assert!(!hub_rows(&hub).iter().any(|r| r.2 == TaxSection::Gambling));
    assert!(matches!(
        send(&mut s, go_to(&d, PAGE_ITEMIZED, TaxSection::Gambling)),
        Reply::Edit(P::SaveIncomes { topic: TaxSection::Gambling, .. })
    ));
}

// An ISO sale page: 1,000 shares exercised in 2024 at $10 (box 3) with a
// $30 box 4 value, granted 2023-01-10, sold 2026-08-03 for $50,550.
fn iso_share_sale() -> IsoShareSale {
    IsoShareSale {
        calendar_year: Default::default(),
        account_number: Default::default(),
        box_1b_acquired: Default::default(),
        form_8949_checkbox: Default::default(),
        box_2_short_term: Default::default(),
        box_2_long_term: Default::default(),
        box_2_ordinary: Default::default(),
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        description: "Synthetic Employer Inc".into(),
        shares: 1000.0,
        sold: 20260803,
        proceeds: 5_055_000,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        granted: 20230110,
        exercised: 20240601,
        exercise_price: 1_000,
        market_price: 3_000,
    }
}
// The sale's answers on Investment sales: no fees, no special treatment.
fn no_iso_adjustments() -> IsoShareSaleMore {
    IsoShareSaleMore { selling_fees: 0, acquisition_costs: 0, option_cost: 0, special: false }
}
fn answer_iso(s: &mut Interview, more: IsoShareSaleMore, in_wages: bool) {
    accepted(&crate::tests::support::client::income_pages::answer_iso_sale(s, &more, in_wages));
}
fn save_iso_sale(s: &mut Interview, documents: Vec<IncomeSource>) -> Reply {
    let rows: Vec<_> = documents.into_iter().map(|d| document(0, d)).collect();
    let request = save_documents(s, rows.clone());
    let reply = send(s, request);
    if !matches!(reply, Reply::Response(P::Problem { .. })) {
        // The Form 3921 answers follow the paper, as the client saves them.
        crate::tests::support::client::income_pages::save_information_return_answers(s, &rows);
    }
    reply
}

#[test]
fn iso_shares_exercised_earlier_use_their_amt_basis() {
    // Single, W-2 wages $60,000; a 2026 Form 3921 whose 750 held shares give
    // Form 6251 line 2i $149,700 (form_3921_kind_reaches_form_6251_line_2i);
    // and the ISO sale above, a qualifying disposition (sold more than 2
    // years after the grant and 1 year after the exercise).
    // Regular: basis 1,000 x $10 = $10,000, long-term gain $40,550; no
    // compensation. AGI $100,550, taxable $84,450 (ordinary $43,900).
    //   QDCG worksheet: line 5 $43,900; 0% up to $49,450: $5,550; 15% on
    //   $35,000 = $5,250; line 22 Tax Table on $43,900 = $5,023; total
    //   $10,273, below the Tax Table's $13,297 on $84,450. Regular tax
    //   $10,273.
    // AMT basis: 1,000 x $30 box 4 = $30,000 (the $10,000 regular basis plus
    //   the $20,000 adjustment of 2024); AMT gain $20,550; line 2k -$20,000.
    // Form 6251: line 1 $84,450 + 2a $16,100 + 2i $149,700 + 2k -$20,000 =
    //   AMTI $230,250; exemption $90,100; line 6 $140,150. Part III: line 13
    //   $20,550; line 18 $140,150 - $20,550 = $119,600 x 26% = $31,096;
    //   line 20 is the regular QDCG line 5 ($43,900, "as figured for the
    //   regular tax"), so line 21 $5,550 at 0%; line 24 $15,000 x 15% =
    //   $2,250; line 40 $33,346, below 26% x $140,150 = $36,439. Tentative
    //   minimum tax $33,346; AMT $33,346 - $10,273 = $23,073; total tax
    //   $33,346.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let sale = iso_share_sale();
    accepted(&save_iso_sale(
        &mut s,
        vec![
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(form_3921(sold_and_option_cost(
                    250.0, 40_000,
                ))),
            },
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(sale.clone()),
            },
        ],
    ));
    answer_iso(&mut s, no_iso_adjustments(), false);
    let id = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::IsoSaleSource { .. }))
        .unwrap()
        .id;
    assert_eq!(s.draft.income.iter().find(|e| e.id == id).unwrap().label, "Synthetic Employer Inc");
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &alloc::format!("iso_sale_{id}_{suffix}"));
    assert_eq!(line("compensation"), 0);
    assert_eq!(line("regular_basis"), 10_000);
    assert_eq!(line("amt_basis"), 30_000);
    assert_eq!(line("capital_gain"), 40_550);
    assert_eq!(line("amt_capital_gain"), 20_550);
    assert_eq!(value(&r, "agi"), 100_550);
    assert_eq!(value(&r, "regular_income_tax"), 10_273);
    assert_eq!(value(&r, "amt_capital_adjustment"), -20_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 230_250);
    assert_eq!(value(&r, "alternative_minimum_tax"), 23_073);
    assert_eq!(r.tax, Some(3_334_600));
    // It reopens as the same kind, and saving it unchanged keeps the record.
    let reopened = open_documents(&mut s);
    let rows = page_documents(&reopened)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
        .collect::<Vec<_>>();
    assert!(rows.iter().any(|row| row.document
        == expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(sale.clone())
            }
        )));
    let before = s.draft.income.clone();
    let request = save_documents(&mut s, rows);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut s, request,
    ));
    assert_eq!(s.draft.income, before);
}

#[test]
fn iso_sale_on_the_two_year_grant_anniversary_is_disqualifying() {
    // Boundary (i6251 / Pub. 525): the sale must be MORE than 2 years after
    // the grant. 100 shares granted 2024-08-01, exercised 2025-07-01 at $10
    // with a $30 box 4 value, sold 2026-08-01 for $5,000.
    // Compensation: the smaller of the spread 100 x ($30 - $10) = $2,000
    // and the gain $5,000 - $1,000 = $4,000: $2,000, in the W-2 (Nothing).
    // Regular basis $1,000 + $2,000 = $3,000, gain $2,000; AMT reverses the
    // $2,000 compensation (line 2k) and uses the $3,000 box 4 basis: AMT
    // gain $2,000.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let sale = IsoShareSale {
        calendar_year: Default::default(),
        account_number: Default::default(),
        box_1b_acquired: Default::default(),
        form_8949_checkbox: Default::default(),
        box_2_short_term: Default::default(),
        box_2_long_term: Default::default(),
        box_2_ordinary: Default::default(),
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        description: "".into(),
        shares: 100.0,
        sold: 20260801,
        proceeds: 500_000,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        granted: 20240801,
        exercised: 20250701,
        exercise_price: 1_000,
        market_price: 3_000,
    };
    accepted(&save_iso_sale(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::IsoShareSale(sale.clone()),
        }],
    ));
    answer_iso(&mut s, no_iso_adjustments(), true);
    let entry = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::IsoSaleSource { .. }))
        .unwrap();
    assert_eq!(entry.label, "");
    let id = entry.id;
    let answer = crate::iso_return::answers(&s.draft, id).unwrap();
    assert_eq!(answer.wage_inclusion, Some(true));
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &alloc::format!("iso_sale_{id}_{suffix}"));
    assert_eq!(line("compensation"), 2_000);
    assert_eq!(line("amt_compensation_adjustment"), -2_000);
    assert_eq!(line("regular_basis"), 3_000);
    assert_eq!(line("amt_basis"), 3_000);
    assert_eq!(line("capital_gain"), 2_000);
    assert_eq!(line("amt_capital_gain"), 2_000);
    // One day later it qualifies: no compensation, regular gain $4,000,
    // AMT gain $2,000, and the W-2 answer is not stored.
    let mut later = sale.clone();
    later.sold = 20260802;
    accepted(&save_iso_sale(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::IsoShareSale(later.clone()),
        }],
    ));
    answer_iso(&mut s, no_iso_adjustments(), true);
    let entry = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::IsoSaleSource { .. }))
        .unwrap();
    let id = entry.id;
    // A qualifying sale has no compensation, so no W-2 answer is asked or stored.
    assert_eq!(crate::iso_return::answers(&s.draft, id).unwrap().wage_inclusion, None);
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &alloc::format!("iso_sale_{id}_{suffix}"));
    assert_eq!(line("compensation"), 0);
    assert_eq!(line("capital_gain"), 4_000);
    assert_eq!(line("amt_capital_gain"), 2_000);
    let reopened = open_documents(&mut s);
    assert_eq!(
        page_documents(&reopened)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
            .collect::<Vec<_>>()[0]
            .document,
        expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(later)
            }
        )
    );
}

#[test]
fn iso_sale_one_year_after_exercise_is_disqualifying() {
    // Boundary: the sale must be MORE than 1 year after the exercise.
    // Granted 2023-01-10, exercised 2025-08-03 at $10 (box 4 $30), 100
    // shares sold 2026-08-03 for $5,000: compensation $2,000 (the spread,
    // below the $4,000 gain); a day later none.
    let sale = |sold: u32| IsoShareSale {
        calendar_year: Default::default(),
        account_number: Default::default(),
        box_1b_acquired: Default::default(),
        form_8949_checkbox: Default::default(),
        box_2_short_term: Default::default(),
        box_2_long_term: Default::default(),
        box_2_ordinary: Default::default(),
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        description: "".into(),
        shares: 100.0,
        sold,
        proceeds: 500_000,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        granted: 20230110,
        exercised: 20250803,
        exercise_price: 1_000,
        market_price: 3_000,
    };
    assert_eq!(sale(20260803).lot(&no_iso_adjustments()).compensation(), 200_000);
    assert_eq!(sale(20260804).lot(&no_iso_adjustments()).compensation(), 0);
}

#[test]
fn iso_sale_page_rules_and_special_answer() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // Exercised before granted, sold before exercised (both sides), and a
    // sale outside 2026 (server only), each refused by its key.
    let mut v = iso_share_sale();
    v.granted = 20240602;
    rejected_with(
        save_iso_sale(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(v),
            }],
        ),
        "ISO_EXERCISED_BEFORE_GRANTED",
    );
    let mut v = iso_share_sale();
    v.exercised = 20260804;
    rejected_with(
        save_iso_sale(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(v),
            }],
        ),
        "ISO_SOLD_BEFORE_EXERCISED",
    );
    let mut v = iso_share_sale();
    v.sold = 20251231;
    rejected_with(
        save_iso_sale(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(v),
            }],
        ),
        "ISO_SALE_NOT_IN_RETURN_YEAR",
    );
    // The printed sale saves without its answers; the special answer is
    // given on Investment sales, stored by the sale, and left out by name in
    // the calculation.
    let paper = iso_share_sale();
    accepted(&save_iso_sale(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::IsoShareSale(paper.clone()),
        }],
    ));
    let id = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::IsoSaleSource { .. }))
        .unwrap()
        .id;
    assert_eq!(crate::iso_return::answers(&s.draft, id), None, "nothing is pre-answered");
    let special = IsoShareSaleMore { special: true, ..no_iso_adjustments() };
    answer_iso(&mut s, special.clone(), true);
    // The sale is left out with a notice; the estimate proceeds, and no W-2
    // question is asked for it.
    assert!(noted(&supported(&s.draft), "WARN_ISO_SALE_LEFT_OUT"));
    let answer = crate::iso_return::answers(&s.draft, id).unwrap();
    assert_eq!((answer.more.clone(), answer.wage_inclusion), (Some(special), None));
    // The printed sale reopens as printed.
    let reopened = open_documents(&mut s);
    assert_eq!(
        page_documents(&reopened)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
            .collect::<Vec<_>>()[0]
            .document,
        expected_named_source(
            &s.draft,
            Owner::Taxpayer,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(paper)
            }
        )
    );
    // Fees and commissions reopen with their question.
    let fees =
        IsoShareSaleMore { selling_fees: 2_500, acquisition_costs: 1_000, ..no_iso_adjustments() };
    answer_iso(&mut s, fees.clone(), true);
    let P::SaveIncomes { followups, .. } = crate::tests::support::client::income_pages::open_topic(
        &mut s,
        TaxSection::InvestmentSales,
    ) else {
        unreachable!()
    };
    assert!(followups.iter().any(
        |row| row.answer == crate::DocumentAnswer::IsoAdjustments { value: Some(fees.clone()) }
    ));
}

#[test]
fn w2_reported_tips_alone_need_no_form_4137_record() {
    // Box 12 code TP with every tip reported to the employer: no Form 4137,
    // so no employee-tip record is required and the estimate is available.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2_source(w2(6_000_000));
    w.more.box12 = vec![Box12Entry { code: W2Code::TP, amount: 100_000 }];
    w.answers.tips_listed_occupation = Some(WageDeductionEligibility::Eligible);
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w)) },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = &FederalWage::from(data);
    assert!(data.employee_tips.is_none());
    assert_eq!(data.work_deductions.form4137_tips(), 0);
    assert_eq!(data.work_deductions.tips(), 100_000);
    let r = estimate(&session.draft);
    assert!(!r.issues.iter().any(|i| i.message.contains("Form 4137")), "{r:?}");
}

#[test]
fn w2_tip_records_need_cover_only_the_tips_beyond_code_tp() {
    // With unreported tips, Form 4137 column 1(c) is the TP-reported tips plus
    // the unreported ones; the records need show only the unreported part.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2_source(w2(6_000_000));
    w.more.box12 = vec![Box12Entry { code: W2Code::TP, amount: 100_000 }];
    w.answers.tips_listed_occupation = Some(WageDeductionEligibility::Eligible);
    // One month: $300 received, $100 of it reported; the TP $1,000 covers the
    // rest of the year's reported tips without a record.
    w.answers.tip_records = Some(DailyTipRecord::Months {
        months: vec![CashTipMonth { month: 3, received: 30_000, reported: 10_000 }],
    });
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w)) },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = &FederalWage::from(data);
    assert_eq!(data.work_deductions.w2_tips(), 100_000);
    assert!(data.work_deductions.form4137_tips() > data.work_deductions.w2_tips());
    let r = estimate(&session.draft);
    assert!(!r.issues.iter().any(|i| i.message.contains("Form 4137")), "{r:?}");
}

#[test]
fn w2_import_keeps_daily_tip_records_and_box_12_answers() {
    let model = crate::federal_model::HUB_SOURCE;
    assert!(import_unselected(model, "pub struct W2Answers {"));
    assert!(!model.contains("#[libertas_llm_doc_field]\n    pub answers: W2Answers"));
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2_source(w2(6_000_000));
    w.more.box12 = vec![
        Box12Entry { code: W2Code::TP, amount: 100_000 },
        Box12Entry { code: W2Code::TT, amount: 80_000 },
    ];
    w.answers = W2Answers {
        calls: W2Calls::default(),
        medicaid_waiver: 0,
        noncash_tips: None,
        statutory_business: None,
        paying_scorporation: None,
        tip_records: Some(DailyTipRecord::Months {
            months: vec![CashTipMonth { month: 3, received: 150_000, reported: 100_000 }],
        }),
        tips_listed_occupation: Some(WageDeductionEligibility::Eligible),
        overtime_flsa_required: Some(WageDeductionEligibility::Ineligible),
        coverage: Some(crate::W2Coverage::ChurchEmployee),
        disability: None,
        abroad: None,
        deferred_pension: None,
        inmate_pay: 0,
        box14: Vec::new(),
    };
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w)) },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    // The import: box 8 printed, box 12 replaced by the form's codes.
    let reopened = page_documents(&open_documents(&mut session)).to_vec();
    let IncomeSource::Wage { value: crate::WageDocument::W2(mut imported) } =
        reopened[0].document.clone()
    else {
        panic!("expected the W-2")
    };
    let answers = stored_w2_answers(&session.draft, 0);
    imported.allocated_tips = Some(40_000);
    imported.box12 = vec![
        crate::PrintedBox12Entry { code: W2Code::TP, amount: Some(110_000) },
        crate::PrintedBox12Entry { code: W2Code::TT, amount: Some(90_000) },
    ];
    let request = save_documents(
        &mut session,
        vec![document(
            reopened[0].id,
            IncomeSource::Wage { value: crate::WageDocument::W2(imported) },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = &FederalWage::from(data);
    // The daily record still replaces the box 8 allocation.
    assert!(matches!(
        &data.employee_tips,
        Some(EmployeeTips { cash: CashTipRecords::Actual { months }, .. }) if months.len() == 1
    ));
    let facts = &data.work_deductions;
    assert!(matches!(
        facts.tips_claim,
        TipsClaim::Yes { w2_tips: 110_000, tips_eligible: Some(true), .. }
    ));
    assert!(matches!(
        facts.overtime_claim,
        OvertimeClaim::Yes { overtime: 90_000, overtime_eligible: Some(false) }
    ));
    assert_eq!(data.coverage, W2Coverage::ChurchEmployee);
    let page = open_documents(&mut session);
    let IncomeSource::Wage { value: crate::WageDocument::W2(again) } =
        &page_documents(&page)[0].document
    else {
        panic!("expected the W-2")
    };
    assert_eq!(stored_w2_answers(&session.draft, 0), answers);
    // A TP or TT code with no answer is refused on the server too.
    let mut unanswered = W2Source { form: again.clone(), answers: answers.clone() };
    unanswered.answers.tips_listed_occupation = None;
    let request = save_documents(
        &mut session,
        vec![document(
            reopened[0].id,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(unanswered)) },
        )],
    );
    rejected_with(
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        ),
        "W2_TP_OCCUPATION_NEEDED",
    );
}

#[test]
fn gambling_kinds_are_told_apart_by_the_record_not_by_a_name() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    save_each_topic(&mut session, t2_44f_documents());
    // Whatever a record is called, it reopens as the kind it is.
    session.draft.income[1].label = "Gambling losses and other winnings".into();
    session.draft.income[3].label = "W-2G".into();
    let mut again = restart(session.draft.clone());
    let mut rows = page_documents(&open_documents(&mut again))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Gambling(_)))
        .collect::<Vec<_>>();
    rows.extend(page_documents(&open_topic(&mut again, TaxSection::Gambling)));
    let documents: Vec<IncomeSource> = rows.into_iter().map(|item| item.document).collect();
    assert_eq!(
        documents,
        expected_named_sources(
            &session.draft,
            Owner::Taxpayer,
            t2_44f_documents().into_iter().map(|item| item.document).collect::<Vec<_>>()
        )
    );
}

#[test]
fn wage_specials_w2_save_reopens_new_source_facts_and_personal_answers() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut form = w2_source(w2(2_600_000));
    form.more.allocated_tips = 300_000;
    form.answers.tips_listed_occupation = Some(WageDeductionEligibility::Eligible);
    form.answers.noncash_tips = W2Amount::of(20_000);
    form.answers.medicaid_waiver = 10_000;
    form.more.box12 = vec![
        Box12Entry { code: W2Code::Q, amount: 500_000 },
        Box12Entry { code: W2Code::II, amount: 100_000 },
        Box12Entry { code: W2Code::TA, amount: 100_000 },
    ];
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    let page = open_documents(&mut session);
    assert_eq!(
        uncalled(&page_documents(&page)[0].document),
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) }
        )
    );
    let saved = session.draft.clone();
    assert_eq!(roundtrip_native_draft(saved.clone()), saved);
    let id = page_documents(&page)[0].id;
    form.more.box12.push(Box12Entry { code: W2Code::TP, amount: 600_000 });
    form.more.social_tips = 600_000;
    form.social_tax = 198_400;
    form.medicare_wages = 3_200_000;
    form.medicare_tax = 46_400;
    let request = save_documents(
        &mut session,
        vec![document(
            id,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
        )],
    );
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    let data = &FederalWage::from(data);
    assert_eq!(data.work_deductions.tips(), 900_000);
    assert_eq!(data.medicaid_waiver, 10_000);
}

#[test]
fn wage_specials_w2_server_twins_reject_inconsistent_boxes() {
    for key in ["W2_TP_OCCUPATION_NEEDED", "W2_SCHEDULE_1_PAY_ABOVE_WAGES", "W2_TA_ABOVE_LIMIT"] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut form = w2_source(w2(1_800_000));
        match key {
            "W2_TP_OCCUPATION_NEEDED" => form.more.allocated_tips = 300_000,
            "W2_SCHEDULE_1_PAY_ABOVE_WAGES" => form.answers.medicaid_waiver = 2_000_000,
            _ => form.more.box12 = vec![Box12Entry { code: W2Code::TA, amount: 260_000 }],
        }
        let before = session.draft.clone();
        rejected_with(
            {
                let request = save_documents(
                    &mut session,
                    vec![document(
                        0,
                        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
                    )],
                );
                crate::tests::support::client::income_pages::save_source_fixture_request(
                    &mut session,
                    request,
                )
            },
            key,
        );
        assert_eq!(session.draft, before);
    }
}

#[test]
fn wage_specials_rrta_imported_boxes_round_trip_without_raw_box14_duplication() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut form = w2(26_000_000);
    form.social_wages = 0;
    form.social_tax = 0;
    form.medicare_wages = 0;
    form.medicare_tax = 0;
    form.more.payroll = W2Payroll::Railroad {
        compensation: 26_000_000,
        tier_1: 1_143_900,
        tier_2: 670_000,
        medicare: 377_000,
        additional_medicare: 54_000,
    };
    // The railroad payroll is the W-2's coverage answer, saved with the paper.
    let mut source = w2_source(form.clone());
    source.answers.coverage =
        crate::tests::support::builders::native_income::railroad_answer(&form.more.payroll);
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(source)) },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    assert!(
        estimate(&session.draft).lines.iter().any(|line| line.id == "additional_medicare_w2_wages")
    );
    let page = open_documents(&mut session);
    assert_eq!(
        uncalled(&page_documents(&page)[0].document),
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) }
        )
    );
    let model = crate::federal_model::HUB_SOURCE;
    // Railroad payroll is an answer beside the paper, never a printed box.
    assert!(!model.contains("pub payroll: PrintedW2Payroll"));
    assert!(import_unselected(model, "pub struct W2Answers {"));
}

#[test]
fn wage_specials_statutory_override_lifecycle_and_forged_source() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(0)));
    let business_id = d.income[1].id;
    let mut session = Interview::new(TEST_USER, d.clone());
    let mut form = w2_source(w2(5_000_000));
    form.more.statutory = true;
    form.answers.statutory_business = Some(StatutoryBusiness::Business { source: business_id });
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(
                1,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) },
            )],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    let data = &FederalWage::from(data);
    assert_eq!(data.statutory_business, Some(business_id));
    let saved = session.draft.clone();
    assert_eq!(roundtrip_native_draft(saved.clone()), saved);
    let mut clear_link = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut clear_link else { unreachable!() };
    let crate::WageDocument::W2(wage) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    let mut answers = stored_w2_answers(&session.draft, 0);
    answers.statutory_business = None;
    wage_document(W2Source { form: wage.clone(), answers });
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        clear_link,
    ));
    accepted(&{
        let request = save_topic(&mut session, TaxSection::SelfEmployment, vec![]);
        send(&mut session, request)
    });
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    let data = &FederalWage::from(data);
    assert_eq!(data.statutory_business, None);
    assert_eq!(data.wages, 5_000_000);
    assert!(session.draft.income.iter().all(|e| e.id != business_id));
    let now = session.draft.clone();
    form.answers.statutory_business = Some(StatutoryBusiness::Business { source: 999 });
    // The refused answer has a paper of its own, so the harness's restoration of the
    // accepted W-2 does not replay it.
    form.state_details.control_number = "refused".into();
    rejected_with(
        {
            let request = save_documents(
                &mut session,
                vec![document(
                    1,
                    IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
                )],
            );
            current_income_refusal(
                "income-source-7638",
                &mut session,
                request,
                "ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME",
                "[documents/common/wages/documents/0/document/answers/statutory_business]",
            )
        },
        "ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME",
    );
    assert_eq!(session.draft, now);
}

#[test]
fn wage_specials_named_eligibility_answers_round_trip_and_classify_sources() {
    for eligibility in [WageDeductionEligibility::Ineligible, WageDeductionEligibility::Eligible] {
        let d = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, d.clone());
        let mut form = w2_source(w2(2_600_000));
        form.more.box12 = vec![
            Box12Entry { code: W2Code::TP, amount: 300_000 },
            Box12Entry { code: W2Code::TT, amount: 100_000 },
        ];
        form.answers.tips_listed_occupation = Some(eligibility);
        form.answers.overtime_flsa_required = Some(eligibility);
        accepted(&{
            let request = save_documents(
                &mut session,
                vec![document(
                    1,
                    IncomeSource::Wage {
                        value: crate::WageDocument::W2(wage_document(form.clone())),
                    },
                )],
            );
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        });
        assert_eq!(
            uncalled(&page_documents(&open_documents(&mut session))[0].document),
            expected_named_source(
                &session.draft,
                Owner::Taxpayer,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) }
            )
        );
        let mut no_amount = w2(2_600_000);
        no_amount.more.box12 = vec![Box12Entry { code: W2Code::TT, amount: 100_000 }];
        let now = session.draft.clone();
        rejected_with(
            {
                let request = save_documents(
                    &mut session,
                    vec![document(
                        1,
                        IncomeSource::Wage {
                            value: crate::WageDocument::W2(wage_document(no_amount)),
                        },
                    )],
                );
                crate::tests::support::client::income_pages::save_source_fixture_request(
                    &mut session,
                    request,
                )
            },
            "W2_TT_FLSA_NEEDED",
        );
        assert_eq!(session.draft, now);
    }
}

#[test]
fn unit16_w2_actual_scorporation_employer_survives_save_reopen_restart_and_deletion() {
    let client = NativeInterviewCase("TAX_UNIT16_W2");
    let mut d = complete(FederalStatus::Single);
    d.income.clear();
    add(
        &mut d,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(20_000, true)) }),
    );
    let source_id = d.income.last().unwrap().id;
    let mut session = Interview::new(TEST_USER, d.clone());
    if client.required_calculation(&mut session) {
        return;
    }
    let page = open_documents(&mut session);
    client.write("SEED", &page);
    let P::SaveDocuments { wage_scorporations, allowed_wage_scorporations, .. } = page else {
        panic!()
    };
    assert_eq!(wage_scorporations.iter().map(|row| row.id).collect::<Vec<_>>(), vec![source_id]);
    assert_eq!(allowed_wage_scorporations, vec![0]);
    let mut form = w2_source(w2(4_000_000));
    form.answers.paying_scorporation =
        Some(crate::WageScorpEmployer::Corporation { source: source_id });
    let answers = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) },
        )],
    );
    let supplied = page_documents(&answers);
    client.save(&mut session, answers);
    crate::tests::support::client::income_pages::save_information_return_answers(
        &mut session,
        &supplied,
    );
    let data = session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::Wage { data } => Some(data),
            _ => None,
        })
        .unwrap();
    assert_eq!(FederalWage::from(data).paying_scorporation, Some(source_id));
    session = restart(session.draft.clone());
    let reopened = open_documents(&mut session);
    client.write("REOPENED", &reopened);
    assert_eq!(
        uncalled(&page_documents(&reopened)[0].document),
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form.clone())) }
        )
    );
    let wage_id = page_documents(&reopened)
        .into_iter()
        .find(|row| matches!(row.document, IncomeSource::Wage { .. }))
        .unwrap()
        .id;
    let saved = session.draft.clone();
    form.answers.paying_scorporation =
        Some(crate::WageScorpEmployer::Corporation { source: 999_999 });
    // The refused answer has a paper of its own, so the harness's restoration of the
    // accepted W-2 does not replay it.
    form.state_details.control_number = "refused".into();
    rejected_with(
        {
            let request = save_documents(
                &mut session,
                vec![document(
                    wage_id,
                    IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
                )],
            );
            current_source_refusal(
                "health-context-15",
                &mut session,
                request,
                "W2_SCORP_EMPLOYER_UNAVAILABLE",
                Some(
                    "[documents/common/wages/documents/0/document/answers/paying_scorporation/source]",
                ),
                SourceScope::Health,
            )
        },
        "W2_SCORP_EMPLOYER_UNAVAILABLE",
    );
    assert_eq!(session.draft, saved);
    let mut remove_employer = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut remove_employer else { unreachable!() };
    documents.common.k1.documents.retain(|row| row.id != source_id);
    let crate::WageDocument::W2(wage) = &mut documents
        .common
        .wages
        .documents
        .iter_mut()
        .find(|row| row.id == wage_id)
        .unwrap()
        .document
    else {
        unreachable!()
    };
    let mut answers = stored_w2_answers(
        &session.draft,
        session
            .draft
            .income
            .iter()
            .position(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
            .unwrap(),
    );
    answers.paying_scorporation = None;
    wage_document(W2Source { form: wage.clone(), answers });
    accepted(&send(&mut session, remove_employer));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    let data = &FederalWage::from(data);
    assert_eq!(data.paying_scorporation, None);
    assert_eq!(data.wages, 4_000_000);
    let page = open_documents(&mut session);
    let P::SaveDocuments { wage_scorporations, allowed_wage_scorporations, .. } = page else {
        panic!()
    };
    assert!(wage_scorporations.is_empty());
    assert!(allowed_wage_scorporations.is_empty());
}

#[test]
fn iso_quantity_above_native_domain_is_refused_before_commit() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let mut page = iso_share_sale();
    page.shares = f64::MAX;
    rejected_with(
        save_iso_sale(
            &mut session,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::IsoShareSale(page),
            }],
        ),
        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
    );
    assert_eq!(session.draft, before);
}

// Form 4137 (2026 draft), lines 4–6: the $20 test is per employer and
// month, while all unreported cash tips remain wage income.
#[test]
fn cash_tips_without_w2_monthly_threshold_and_native_roundtrip() {
    use super::support::builders::native_income::cash_tips_without_w2;
    for eligible in [false, true] {
        let source = cash_tips_without_w2(
            vec![
                CashTipMonth { month: 1, received: 1_900, reported: 0 },
                CashTipMonth { month: 2, received: 2_000, reported: 0 },
                CashTipMonth { month: 3, received: 3_000, reported: 2_900 },
            ],
            TipCoverage::SocialAndMedicare,
            eligible,
        );
        let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
        let request = save_topic(&mut s, TaxSection::Wages, vec![document(0, source.clone())]);
        accepted(&send(&mut s, request));
        let result = supported(&s.draft);
        assert_eq!(value(&result, "employee_tip_income"), 40);
        assert_eq!(return_line(&result, "1c"), 4_000);
        assert_eq!(return_line(&result, "1a"), 0);
        assert_eq!(value(&result, "employee_tips_0_social_tax"), 1);
        assert_eq!(value(&result, "employee_tips_0_medicare_tax"), 0);
        assert_eq!(value(&result, "tips_deduction"), if eligible { 40 } else { 0 });
        let draft = roundtrip_native_draft(s.draft.clone());
        let mut s = restart(draft);
        let page = open_topic(&mut s, TaxSection::Wages);
        assert_eq!(page_documents(&page)[0].document, source);
        let saved_id = page_documents(&page)[0].id;
        assert!(matches!(
            &page,
            P::SaveWagesAnswers { wages, .. } if wages.is_empty()
        ));
        assert_eq!(supported(&s.draft), result);
        accepted(&send(&mut s, page));
        let center = open_documents(&mut s);
        assert!(page_documents(&center).is_empty());
        accepted(&send(&mut s, center));
        let page = open_topic(&mut s, TaxSection::Wages);
        assert_eq!(page_documents(&page)[0].id, saved_id);
        assert_eq!(page_documents(&page)[0].document, source);
        assert_eq!(supported(&s.draft).return_lines, result.return_lines);
    }
}

#[test]
fn cash_tips_without_w2_reuses_wage_base_and_medicare_owners() {
    use super::support::builders::native_income::cash_tips_without_w2;
    for (wages, coverage, social, medicare, additional) in [
        (18_449_000, TipCoverage::SocialAndMedicare, 1, 15, 0),
        (20_000_000, TipCoverage::SocialAndMedicare, 0, 15, 9),
        (20_000_000, TipCoverage::MedicareOnlyGovernment, 0, 15, 9),
        (20_000_000, TipCoverage::Railroad, 0, 0, 0),
    ] {
        let mut payroll = w2(wages);
        payroll.social_wages = wages.min(18_450_000);
        payroll.social_tax = payroll.social_wages * 62 / 1000;
        let source = cash_tips_without_w2(
            vec![CashTipMonth { month: 1, received: 100_000, reported: 0 }],
            coverage,
            false,
        );
        let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
        save_each_topic(
            &mut s,
            vec![
                document(0, IncomeSource::Wage { value: WageDocument::W2(wage_document(payroll)) }),
                document(0, source),
            ],
        );
        let result = supported(&s.draft);
        assert_eq!(value(&result, "employee_tips_0_social_tax"), social);
        assert_eq!(value(&result, "employee_tips_0_medicare_tax"), medicare);
        assert_eq!(value(&result, "additional_medicare_tax"), additional);
        assert_eq!(return_line(&result, "1c"), 100_000);
        assert_eq!(return_line(&result, "1a"), wages);
        assert_eq!(
            result
                .review_notes
                .iter()
                .any(|n| n.display.key == "NOTE_RAILROAD_TIPS_REPORT_TO_EMPLOYER"),
            medicare == 0,
        );
    }
}

#[test]
fn planned_cash_tips_need_record_save_then_calculate_without_a_stale_refusal() {
    use super::support::builders::native_income::cash_tips_without_w2;
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let reply = crate::tests::cross::interview::save_without_form(
        &mut s,
        IncomeWithoutFormAnswers { tips: true, ..Default::default() },
    );
    assert!(matches!(reply, Reply::Edit(P::SaveWagesAnswers { .. })), "{reply:?}");
    let result = estimate(&s.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.tax.is_none());
    assert!(
        crate::interview::remaining_items(&s.draft, &result, TaxSection::Wages)
            .iter()
            .any(|row| row.section == TaxSection::Wages)
    );
    let source = cash_tips_without_w2(
        vec![CashTipMonth { month: 1, received: 100_000, reported: 0 }],
        TipCoverage::SocialAndMedicare,
        true,
    );
    let request = save_topic(&mut s, TaxSection::Wages, vec![document(0, source)]);
    let reply = send(&mut s, request);
    accepted(&reply);
    assert!(matches!(reply, Reply::Edit(P::SaveOftenMissed { .. })), "{reply:?}");
    assert!(s.draft.finished_sections.contains(&TaxSection::Income));
    let result = supported(&s.draft);
    assert_eq!(value(&result, "employee_tip_income"), 1_000);
    assert!(s.draft.without_form.as_ref().unwrap().tips);
    assert!(result.issues.is_empty());
}

#[test]
fn cash_tips_without_w2_rejects_duplicate_months_atomically() {
    use super::support::builders::native_income::cash_tips_without_w2;
    let source = cash_tips_without_w2(
        vec![CashTipMonth { month: 1, received: 2_000, reported: 0 }; 2],
        TipCoverage::SocialAndMedicare,
        false,
    );
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let before = s.draft.clone();
    let request = save_topic(&mut s, TaxSection::Wages, vec![document(0, source)]);
    rejected_with(send(&mut s, request), "ISSUE_EMPLOYEE_TIPS_RECORDS");
    assert_eq!(s.draft, before);
}

#[test]
fn w2_informational_codes_and_control_number_survive_correction_and_restart_without_figures() {
    use crate::tests::support::client::income_pages::{reopened_wage_fixture, save_wage_fixture};
    let paper = informational_wage_document();
    let mut plain = paper.clone();
    plain.state_details.control_number.clear();
    plain.more.box12.clear();
    let mut session = restart(complete(FederalStatus::Single));
    accepted(&save_wage_fixture(&mut session, 0, plain));
    let mut expected = supported(&session.draft);
    let id = session.draft.income[0].id;
    accepted(&save_wage_fixture(&mut session, id, paper.clone()));
    expected.revision = session.draft.revision;
    assert_eq!(supported(&session.draft), expected);
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    let data = &FederalWage::from(data);
    assert_eq!(
        data.other_box12,
        paper
            .more
            .box12
            .iter()
            .map(|row| W2CodeAmount { code: row.code, amount: row.amount })
            .collect::<Vec<_>>()
    );
    let saved = session.draft.clone();
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let (reopened_id, mut corrected) = reopened_wage_fixture(&mut session);
    assert_eq!(reopened_id, id);
    assert_eq!(corrected.state_details.control_number, "00-SYNTHETIC/7");
    assert_eq!(corrected.more.box12, paper.more.box12);
    assert_eq!(stored_w2_answers(&session.draft, 0).calls, W2Calls::default());
    assert_eq!(supported(&session.draft), expected);
    // Correct printed facts on the same source without changing wages or answers.
    corrected.state_details.control_number = "0008-A".into();
    corrected.more.box12.retain(|row| matches!(row.code, W2Code::V | W2Code::HH));
    corrected.more.box12[0].amount = 99_999;
    accepted(&save_wage_fixture(&mut session, id, corrected.clone()));
    expected.revision = session.draft.revision;
    assert_eq!(supported(&session.draft), expected);
    session = restart(session.draft);
    let (corrected_id, mut removed) = reopened_wage_fixture(&mut session);
    assert_eq!(corrected_id, id);
    assert_eq!(removed.state_details.control_number, corrected.state_details.control_number);
    assert_eq!(removed.more.box12, corrected.more.box12);
    removed.state_details.control_number.clear();
    removed.more.box12.clear();
    accepted(&save_wage_fixture(&mut session, id, removed));
    expected.revision = session.draft.revision;
    assert_eq!(supported(&session.draft), expected);
    session = restart(session.draft);
    let (removed_id, removed) = reopened_wage_fixture(&mut session);
    assert_eq!(removed_id, id);
    assert!(removed.state_details.control_number.is_empty());
    assert!(removed.more.box12.is_empty());
    assert_eq!(supported(&session.draft), expected);
}

#[test]
fn w2_entry_collects_payroll_once_and_converts_without_classifying_labels() {
    let mut paper = wage_document(w2(1_800_000));
    (paper.social_wages, paper.social_tax, paper.medicare_wages, paper.medicare_tax) =
        (Some(0), Some(0), Some(0), Some(0));
    let payroll = W2Payroll::Railroad {
        compensation: 2_000_000,
        tier_1: 124_000,
        tier_2: 98_000,
        medicare: 29_000,
        additional_medicare: 0,
    };
    paper.box14a = vec![W2OtherItem {
        label: "Employer's railroad compensation label".into(),
        value: W2OtherValue::Money { amount: 2_000_000 },
    }];
    let mut source = W2Source::printed(paper.clone());
    // The paper alone is Social Security payroll; the railroad answer beside it decides.
    assert_eq!(W2::from(&paper).more.payroll, W2Payroll::SocialSecurity);
    source.answers.coverage =
        crate::tests::support::builders::native_income::railroad_answer(&payroll);
    assert_eq!(FederalWage::from(&source).payroll, payroll);
    // Correcting the paper keeps the answers; printed box 3 wages end the railroad answer's effect.
    source.answers.medicaid_waiver = 1_000;
    let mut corrected = WageSource::W2(source.clone());
    let WageSource::W2(value) = &mut corrected else { unreachable!() };
    value.form.social_wages = Some(1_800_000);
    value.answers = Default::default();
    corrected.keep_answers(&WageSource::W2(source));
    let WageSource::W2(value) = corrected else { unreachable!() };
    assert_eq!(value.answers.medicaid_waiver, 1_000);
    assert_eq!(value.native().more.payroll, W2Payroll::SocialSecurity);
}

fn box14_source(items: &[&str], states: &[TaxState]) -> W2Source {
    let mut paper = wage_document(w2(6_000_000));
    paper.box14a = items
        .iter()
        .map(|label| W2OtherItem {
            label: (*label).into(),
            value: W2OtherValue::Money { amount: 123_456 },
        })
        .collect();
    paper.states = states
        .iter()
        .map(|state| StateIncomeTax {
            state: *state,
            payer_state_id: alloc::string::String::new(),
            income: None,
            withholding: None,
        })
        .collect();
    let mut source = W2Source::printed(paper);
    source.classify_box14();
    source
}

#[test]
fn w2_box14_labels_settle_their_kind_and_state_once() {
    use W2Box14Kind::*;
    for (label, printed, kind, state) in [
        ("CA SDI", &[][..], StateDisabilityOrLeave, Some(TaxState::CA)),
        ("CASDI", &[], StateDisabilityOrLeave, Some(TaxState::CA)),
        ("vpdi - california", &[], StateDisabilityOrLeave, Some(TaxState::CA)),
        ("NJ UI/WF/SWF", &[], StateDisabilityOrLeave, Some(TaxState::NJ)),
        ("FLI", &[TaxState::NJ], StateDisabilityOrLeave, Some(TaxState::NJ)),
        ("N.Y. PFL", &[], StateDisabilityOrLeave, Some(TaxState::NY)),
        ("NY PFL", &[], StateDisabilityOrLeave, Some(TaxState::NY)),
        ("TDI", &[TaxState::RI, TaxState::RI], StateDisabilityOrLeave, Some(TaxState::RI)),
        ("MAPFML", &[TaxState::MA, TaxState::NY], StateDisabilityOrLeave, Some(TaxState::MA)),
        ("WA PFML", &[], StateDisabilityOrLeave, Some(TaxState::WA)),
        ("414(h)", &[TaxState::NY], StatePickup, Some(TaxState::NY)),
        ("STPICKUP", &[], StatePickup, None),
        ("RRTA Tier 1", &[], Railroad, None),
        ("Union Dues", &[], UnionDues, None),
        ("Dental", &[TaxState::CA], PretaxBenefit, None),
        ("Uniform", &[], Informational, None),
    ] {
        let source = box14_source(&[label], printed);
        let meaning = &source.answers.box14[0];
        assert_eq!((meaning.kind, meaning.state), (Some(kind), state), "{label}");
    }
    // An unknown label asks once, with its printed label.
    let source = box14_source(&["Employer widget"], &[TaxState::CA]);
    assert_eq!(source.answers.box14, vec![W2Box14Meaning { item: 0, state: None, kind: None }]);
    let asks = source.box14_asks();
    assert_eq!((asks.len(), asks[0].label.as_str()), (1, "Employer widget"));
    // A kind that needs a state and finds none asks for the state only.
    let source = box14_source(&["SDI"], &[]);
    assert_eq!(source.box14_asks()[0].kind, Some(StateDisabilityOrLeave));
    assert!(box14_source(&["SDI"], &[TaxState::CA]).box14_asks().is_empty());
}

#[test]
fn w2_box14_recorded_answer_survives_paper_correction_and_restart() {
    use crate::tests::support::client::income_pages::{reopened_wage_fixture, save_wage_fixture};
    let item = |label: &str| W2OtherItem {
        label: label.into(),
        value: W2OtherValue::Money { amount: 5_000 },
    };
    let mut paper = wage_document(w2(6_000_000));
    paper.box14a = vec![item("Union Dues"), item("CA SDI")];
    let mut session = restart(complete(FederalStatus::Single));
    accepted(&save_wage_fixture(&mut session, 0, W2::from(&paper)));
    let id = session.draft.income[0].id;
    let box14 = stored_w2_answers(&session.draft, 0).box14;
    assert_eq!(box14[0].kind, Some(W2Box14Kind::UnionDues));
    assert_eq!(box14[1].kind, Some(W2Box14Kind::StateDisabilityOrLeave));
    // The person's answer is recorded on the wrapper.
    let FederalIncome::Wage { data: WageSource::W2(source) } = &mut session.draft.income[0].income
    else {
        unreachable!()
    };
    source.answers.box14[0].kind = Some(W2Box14Kind::PretaxBenefit);
    source.answers.box14[1].kind = Some(W2Box14Kind::Other);
    let recorded = source.answers.box14.clone();
    // A restart and a corrected paper keep both, whatever the new labels say.
    session = restart(session.draft);
    let (reopened, mut corrected) = reopened_wage_fixture(&mut session);
    assert_eq!(reopened, id);
    corrected.state_details.box14a = vec![item("Employer widget"), item("Dues")];
    accepted(&save_wage_fixture(&mut session, id, corrected));
    assert_eq!(stored_w2_answers(&session.draft, 0).box14, recorded);
}
