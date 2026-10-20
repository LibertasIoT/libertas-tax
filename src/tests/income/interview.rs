use crate::tests::support::client::roundtrip_native_draft;
// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::support::client::unit50_interview_native::NativeInterviewCase;
use super::*;
use crate::interview::document_income;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, native_w2 as w2};
use super::support::client::income_pages::{
    catalog_preview_document, open_documents, open_topic, page_documents, save_documents,
    save_each_topic, save_topic, topic_for,
};
use crate::tests::cross::interview::{
    PAGE_DEDUCTIONS, PAGE_INCOME, PAGE_ITEMIZED, deductions_value, form_3921, gambling_losses,
    go_to, go_to_from_review, hub_rows, import_unselected, interest, itemized_hub, navigate,
    no_rental_expenses, open_credits_topic, open_itemized_topic, refusal, review_result, row_of,
    send_past_less_common, w2g,
};

// Unit M3-P2: the W-2 page's fold (box 8, box 12, box 13) maps onto
// FederalWage the way the map describes.
#[test]
fn w2_fold_maps_tips_box12_and_box13() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2(6_000_000);
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
    w.answers.tips_listed_occupation = WageDeductionEligibility::Eligible;
    w.answers.overtime_flsa_required = WageDeductionEligibility::Eligible;
    let reply = send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(w.clone()))]),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
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
            tips_eligible: true,
        }
    );
    assert_eq!(
        facts.overtime_claim,
        OvertimeClaim::Yes { overtime: 80_000, overtime_eligible: true }
    );
    assert_eq!(data.other_box12, vec![W2CodeAmount { code: W2Code::D, amount: 300_000 }]);
    assert_eq!(data.care_benefits, 0);
    assert!(!data.statutory_employee);
    assert!(data.retirement_plan);
    // Reopening rebuilds the identical page, in the same box 12 order.
    let mut session = restart(session.draft);
    let documents = page_documents(&open_documents(&mut session)).to_vec();
    assert_eq!(documents[0].document, IncomeDocument::W2(w));
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
    let reply = send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(w.clone()))]),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
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
    assert_eq!(documents[0].document, IncomeDocument::W2(w));
}

#[test]
fn unemployment_fold_boxes_refuse_or_leave_out_by_name() {
    let original = complete(FederalStatus::Single);
    let unemployment_more = |rtaa: i64,
                             taxable_grants: i64,
                             agriculture_payment: i64,
                             market_gain: i64,
                             family_leave: i64,
                             family_leave_medical: bool,
                             special: bool| Form1099GMore {
        rtaa,
        taxable_grants,
        agriculture_payment,
        grant_use: Some(GrantUse::You),
        market_gain,
        family_leave,
        family_leave_medical,
        repayments: Vec::new(),
        special,
        agriculture_farm: None,
    };
    let doc = |more: Form1099GMore| Form1099G {
        state_details: Default::default(),
        amount: 1_000_000,
        withholding: 100_000,
        more,
    };
    let refused_for = |more: Form1099GMore, needle: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::Form1099G(doc(more)))]),
        );
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        let r = estimate(&session.draft);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        assert!(r.issues.iter().any(|i| i.message.contains(needle)), "{r:?}");
    };
    // Boxes 5, 7, 8, 9 and box 10 with a medical-leave portion, plus the
    // special checkbox, are each refused by name; none is calculated yet.
    // Box 5 RTAA is not refused: it reaches Schedule 1 line 8z.
    let mut session = Interview::new(TEST_USER, original.clone());
    send(
        &mut session,
        save_documents(
            &original,
            vec![document(
                0,
                IncomeDocument::Form1099G(doc(unemployment_more(1_000, 0, 0, 0, 0, false, false))),
            )],
        ),
    );
    let r = supported(&session.draft);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(value(&r, "other_income_2_rtaa"), 10);
    refused_for(unemployment_more(0, 0, 0, 1, 0, false, false), "box 9 market gain");
    refused_for(unemployment_more(0, 0, 1, 0, 0, false, false), "box 7 agriculture payments");
    // A medical-leave portion is taxed in full and the special checkbox leaves
    // the statement out, each with a notice.
    for (more, key) in [
        (unemployment_more(0, 0, 0, 0, 1, true, false), "WARN_FAMILY_LEAVE_TAXED_IN_FULL"),
        (unemployment_more(0, 0, 0, 0, 0, false, true), "WARN_UNEMPLOYMENT_LEFT_OUT"),
    ] {
        let mut session = Interview::new(TEST_USER, original.clone());
        send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::Form1099G(doc(more)))]),
        );
        assert!(noted(&supported(&session.draft), key), "{key}");
    }
    // Box 6 taxable grants is not refused: it reaches Schedule 1 line 8z.
    let mut session = Interview::new(TEST_USER, original.clone());
    send(
        &mut session,
        save_documents(
            &original,
            vec![document(
                0,
                IncomeDocument::Form1099G(doc(unemployment_more(0, 25_000, 0, 0, 0, false, false))),
            )],
        ),
    );
    let r = supported(&session.draft);
    assert_eq!(value(&r, "taxable_grants_1099g"), 250);
    // Reopening restores the fold.
    let mut session = restart(session.draft);
    let documents =
        page_documents(&open_topic(&mut session, TaxSection::GovernmentBenefits)).to_vec();
    assert_eq!(
        documents[0].document,
        IncomeDocument::Form1099G(doc(unemployment_more(0, 25_000, 0, 0, 0, false, false)))
    );
}

#[test]
fn unemployment_1099g_with_only_box_7_imports_and_reaches_the_farm() {
    // Form 1099-G with only box 7 (agriculture payments) printed: AI import
    // must not demand box 1. Box 1 is importable-empty (default 0) and box 7
    // is an import target; box 4 may exceed box 1 when box 7 carries the
    // payment. Import mapping: box 7 $800 -> more.agriculture_payment, box 1
    // $0 -> amount. The payment adds to the farm's Schedule F program income:
    // $10,000 raised + $800 = $10,800 gross.
    let model = include_str!("../../federal/federal_model.rs");
    let page = &model[model.find("pub struct Form1099G {").unwrap()..];
    let page = &page[..page.find("\n}").unwrap()];
    assert!(
        page.contains("/// Box 1 unemployment compensation\n    /// Leave zero on a form that shows only box 7 agriculture payments.\n    #[libertas_llm_doc_field]\n    #[libertas_money(\"USD\", 2)]"),
        "{page}"
    );
    assert!(page.contains("#[libertas_default(0)]\n    pub amount: i64"));
    assert!(!page.contains("allow_empty = false)]\n    #[libertas_money(\"USD\", 2)]\n    #[libertas_number(min = 0)]\n    pub amount"));
    let more = &model[model.find("pub struct Form1099GMore {").unwrap()..];
    let more = &more[..more.find("\n}").unwrap()];
    assert!(more.contains("#[libertas_llm_doc_field]\n    #[libertas_money(\"USD\", 2)]\n    #[libertas_number(min = 0)]\n    #[libertas_default(0)]\n    pub agriculture_payment: i64"));
    assert!(model.contains(
        "Read box 7 agriculture payments, returning an empty string when it is clearly blank."
    ));

    let original = complete(FederalStatus::Single);
    let doc = Form1099G {
        state_details: Default::default(),
        amount: 0,
        withholding: 8_000,
        more: Form1099GMore { agriculture_payment: 80_000, ..Default::default() },
    };
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::Form1099G(doc.clone()))]),
    );
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Unemployment { data } => Some(data.clone()),
            _ => None,
        })
        .expect("stored 1099-G");
    assert_eq!((stored.amount, stored.agriculture_payment, stored.withholding), (0, 80_000, 8_000));
    let mut with_farm = session.draft.clone();
    add(&mut with_farm, income(raised_farm(10_000, 3_000)));
    let farm_id = with_farm.income.last().unwrap().id;
    let r = supported(&with_farm);
    assert_eq!(value(&r, &alloc::format!("farm_{farm_id}_gross_income")), 10_800);
    // Reopening restores the document unchanged.
    let mut session = restart(session.draft);
    let documents =
        page_documents(&open_topic(&mut session, TaxSection::GovernmentBenefits)).to_vec();
    assert_eq!(documents[0].document, IncomeDocument::Form1099G(doc));
    // Box 4 above every printed payment is still refused.
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = send(
        &mut session,
        save_documents(
            &original,
            vec![document(
                0,
                IncomeDocument::Form1099G(Form1099G {
                    state_details: Default::default(),
                    amount: 0,
                    withholding: 80_001,
                    more: Form1099GMore { agriculture_payment: 80_000, ..Default::default() },
                }),
            )],
        ),
    );
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
        accepted(&send(
            &mut session,
            save_documents(
                &original,
                vec![document(0, IncomeDocument::OtherIncome(prize.clone()))],
            ),
        ));
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
                topic_for(&IncomeDocument::OtherIncome(prize.clone()))
            ))
            .iter()
            .any(|item| item.document == IncomeDocument::OtherIncome(prize.clone()))
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
        kind: crate::RoyaltyKind::Intangible,
        receipts: 200_000,
        advance: 0,
        expenses: RentalExpenses { legal: 30_000, ..no_rental_expenses() },
        depletion: crate::RoyaltyDepletion::No,
        special: false,
    };
    let equipment_rent = EquipmentRent {
        receipts: 150_000,
        expenses: 20_000,
        basis: 500_000,
        depreciable_basis: 500_000,
        assets: vec![],
        history: PersonalRentalHistory::NONE,
    };
    send(
        &mut session,
        save_documents(
            &original,
            vec![
                document(0, IncomeDocument::OtherIncome(prize.clone())),
                document(0, IncomeDocument::EquipmentRental(equipment_rent.clone())),
            ],
        ),
    );
    let req = save_topic(
        &session.draft,
        TaxSection::Rentals,
        vec![document(0, IncomeDocument::Royalties(royalty.clone()))],
    );
    accepted(&send(&mut session, req));
    let records: Vec<_> = session.draft.income[1..].iter().map(|e| &e.income).collect();
    assert_eq!(records[0], &prize_income(50_000, 0, 0, PrizeDisposition::Received));
    assert_eq!(records[2], &super::royalty(2_000, 300));
    assert_eq!(
        records[1],
        &FederalIncome::NonbusinessProperty {
            kind: NonbusinessPropertyKind::PersonalRental {
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
    assert_eq!(documents[0].document, IncomeDocument::OtherIncome(prize));
    assert_eq!(documents[1].document, IncomeDocument::EquipmentRental(equipment_rent));
    let rentals = page_documents(&open_topic(&mut session, TaxSection::Rentals)).to_vec();
    assert_eq!(rentals[0].document, IncomeDocument::Royalties(royalty));
}

#[test]
fn wages_save_keeps_the_interest_documents() {
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let dividend = document(0, IncomeDocument::Div1099(native_div1099("".into(), 10_000, 5_000)));
    save_each_topic(
        &mut s,
        vec![document(0, IncomeDocument::W2(w2(4_000_000))), interest(20_000), dividend],
    );
    let accepted = s.draft.clone();
    assert_eq!(accepted.income.len(), 3);
    // A changed W-2 replaces the W-2 in place; the 1099s keep their records
    // and identities.
    let mut wages = page_documents(&open_documents(&mut s)).to_vec();
    if let IncomeDocument::W2(w) = &mut wages[0].document {
        w.wages = 4_100_000;
        w.social_wages = 4_100_000;
        w.medicare_wages = 4_100_000;
    }
    send(&mut s, save_topic(&accepted, TaxSection::Wages, wages));
    assert_eq!(s.draft.revision, accepted.revision + 1);
    assert_eq!(s.draft.income[1..], accepted.income[1..]);
    assert_eq!(s.draft.income[0].id, accepted.income[0].id);
    assert_ne!(s.draft.income[0], accepted.income[0]);
    // Removing every W-2 removes only the W-2.
    let changed = s.draft.clone();
    send(&mut s, save_topic(&changed, TaxSection::Wages, vec![]));
    assert_eq!(s.draft.income, changed.income[1..]);
    // A row cannot take over another topic's record.
    let mut stolen = document(0, IncomeDocument::W2(w2(1_000_000)));
    stolen.id = changed.income[1].id;
    let current = s.draft.clone();
    error(send(&mut s, save_topic(&current, TaxSection::Wages, vec![stolen])));
    assert_eq!(s.draft, current);
}

// W-2 and wave-7/other AI Import documents use the same server-supplied year.
#[test]
fn wages_editor_supplies_hidden_w2_document_year() {
    for section in [
        TaxSection::Wages,
        TaxSection::Retirement,
        TaxSection::Gambling,
        TaxSection::InvestmentSales,
        TaxSection::CanceledDebt,
        TaxSection::InterestDividends,
        TaxSection::InvestmentSales,
        TaxSection::GovernmentBenefits,
        TaxSection::OtherIncome,
        TaxSection::Farming,
    ] {
        let draft = empty_return(FederalStatus::Single);
        let mut interview = Interview::new(TEST_USER, draft);
        let P::SaveIncomes { topic, document_facts, .. } = open_topic(&mut interview, section)
        else {
            panic!("expected the {section:?} editor")
        };
        assert_eq!(topic, section);
        assert_eq!(document_facts.year, 2026);
    }
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
    let form = form_3921(sold_and_option_cost(250.0, 40_000));
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::Form3921(form.clone()))],
    );
    accepted(&send(&mut s, request));
    let stored = &s.draft.screening.as_ref().unwrap().iso_exercises;
    assert_eq!(stored.len(), 1);
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
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    assert_eq!(
        page_documents(&reopened),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::Form3921(form.clone()))
        }]
    );
    let before = s.draft.screening.clone();
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        page_documents(&reopened).to_vec(),
    );
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.screening, before);
    // Removing the form removes the adjustment.
    let request = save_topic(&s.draft.clone(), TaxSection::InvestmentSales, vec![]);
    accepted(&send(&mut s, request));
    assert!(s.draft.screening.as_ref().unwrap().iso_exercises.is_empty());
    assert_eq!(value(&supported(&s.draft), "alternative_minimum_taxable_income"), 60_000);
}

#[test]
fn form_3921_nothing_fold_round_trips_and_uses_box_4() {
    // Nothing else: all 1,000 shares held, no option cost, exercise-date
    // value. Line 2i 1,000 x $200 = $200,000; AMTI $260,000; less $90,100 =
    // $169,900 x 26% = $44,174; AMT $44,174 - $5,023 = $39,151.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let form = form_3921(Form3921More::default());
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::Form3921(form.clone()))],
    );
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 260_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 39_151);
    // The fold reopens as Nothing exactly when its answers are the defaults.
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::InvestmentSales)),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::Form3921(form))
        }]
    );
    let something = form_3921(sold_and_option_cost(0.0, 1));
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::Form3921(something.clone()))],
    );
    accepted(&send(&mut s, request));
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::InvestmentSales)),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(0, IncomeDocument::Form3921(something))
        }]
    );
}

#[test]
fn form_3921_every_share_sold_in_2026_has_no_adjustment() {
    // Boundary: sold equals box 5. A disposition in the exercise year makes
    // the regular and AMT treatment the same (i6251 line 2i), so AMTI is the
    // $60,000 of the return without the form and there is no AMT.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeDocument::Form3921(form_3921(sold_and_option_cost(1000.0, 40_000))),
        )],
    );
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    // One millionth of a share more than box 5 is refused by its key.
    let before = s.draft.clone();
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![document(
                    0,
                    IncomeDocument::Form3921(form_3921(sold_and_option_cost(1000.000001, 0))),
                )],
            ),
        ),
        "ISO_SOLD_ABOVE_TRANSFERRED",
    );
    assert_eq!(s.draft, before);
}

#[test]
fn form_3921_exercise_on_the_grant_date_is_accepted_and_earlier_is_refused() {
    // Boundary: box 2 equal to box 1 is accepted; one day earlier is refused.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut form = form_3921(Form3921More::default());
    form.granted = form.exercised;
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::Form3921(form.clone()))],
    );
    accepted(&send(&mut s, request));
    form.granted = 20260311;
    let before = s.draft.clone();
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![document(0, IncomeDocument::Form3921(form))],
            ),
        ),
        "ISO_EXERCISED_BEFORE_GRANTED",
    );
    assert_eq!(s.draft, before);
}

#[test]
fn form_3921_recognition_must_fall_in_2026() {
    // Boundary: a December 31, 2025 exercise entered AMT income on the 2025
    // return and is refused by name; a January 1, 2026 exercise is accepted.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut form = form_3921(Form3921More::default());
    form.exercised = 20251231;
    let before = s.draft.clone();
    let message = refusal(send(
        &mut s,
        save_topic(
            &before,
            TaxSection::InvestmentSales,
            vec![document(0, IncomeDocument::Form3921(form.clone()))],
        ),
    ));
    assert!(message.contains("Form 3921"), "{message}");
    assert!(message.contains("Form 6251 line 2i"), "{message}");
    assert_eq!(s.draft, before);
    form.exercised = 20260101;
    accepted(&send(
        &mut s,
        save_topic(
            &before,
            TaxSection::InvestmentSales,
            vec![document(0, IncomeDocument::Form3921(form.clone()))],
        ),
    ));
    // Restricted shares exercised in 2025 and vesting on June 1, 2026 enter
    // AMT income at their $310 vesting value: 1,000 x ($310 - $10) =
    // $300,000; AMTI $360,000; less $90,100 = $269,900; 26% of $244,500 =
    // $63,570 plus 28% of $25,400 = $7,112, $70,682; AMT $70,682 - $5,023 =
    // $65,659.
    form.exercised = 20251231;
    form.more = Form3921More {
        sold: 0.0,
        recognition: IsoRecognition::Vesting { date: 20260601, market_price: 31_000 },
        option_cost: 0,
    };
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::Form3921(form.clone()))],
    );
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 360_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 65_659);
    // A vesting date before box 2 is refused by its key.
    form.more = Form3921More {
        sold: 0.0,
        recognition: IsoRecognition::Vesting { date: 20251230, market_price: 31_000 },
        option_cost: 0,
    };
    let before = s.draft.clone();
    rejected_with(
        send(
            &mut s,
            save_topic(
                &before,
                TaxSection::InvestmentSales,
                vec![document(0, IncomeDocument::Form3921(form))],
            ),
        ),
        "ISO_VESTED_BEFORE_EXERCISED",
    );
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
    accepted(&send(&mut session, save_documents(&original, t2_44f_documents())));
    // No gambling record is named; the Form W-2G kind is told from the
    // records kept without one by the record itself, never by a label.
    let stored: Vec<(&str, &FederalIncome)> =
        session.draft.income[1..].iter().map(|e| (e.label.as_str(), &e.income)).collect();
    let record = |winnings, losses, withholding, own_records| FederalIncome::Gambling {
        data: FederalGambling { winnings, losses, withholding, own_records, ordinary: true },
    };
    assert_eq!(
        stored,
        vec![
            ("", &record(500_000, 0, 50_000, false)),
            ("", &record(350_000, 0, 30_000, false)),
            ("", &record(0, 1_000_000, 0, true)),
        ]
    );
    // Reopening rebuilds the same rows, each previewing its winnings, and
    // saving them unchanged stores the same records.
    let page = open_topic(&mut session, TaxSection::Gambling);
    let rows = page_documents(&page).to_vec();
    assert_eq!(
        rows.iter().map(|r| r.document.clone()).collect::<Vec<_>>(),
        t2_44f_documents().into_iter().map(|r| r.document).collect::<Vec<_>>()
    );
    assert_eq!(
        rows.iter().map(|r| r.preview).collect::<Vec<_>>(),
        vec![Some(500_000), Some(350_000), Some(0)]
    );
    let before = session.draft.income.clone();
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.income, before);
    // A ticked "Something else applies" is stored as not ordinary and
    // reopens ticked.
    let draft = session.draft.clone();
    let mut special = rows.clone();
    special[2].document = IncomeDocument::GamblingLosses(GamblingLosses {
        other_winnings: 20_000,
        losses: 1_000_000,
        special: true,
    });
    special[0].document = IncomeDocument::W2g(W2g {
        state_details: Default::default(),
        winnings: 500_000,
        withholding: 50_000,
        special: true,
    });
    accepted(&send(&mut session, save_topic(&draft, TaxSection::Gambling, special.clone())));
    let page = open_topic(&mut session, TaxSection::Gambling);
    assert_eq!(
        page_documents(&page).iter().map(|r| r.document.clone()).collect::<Vec<_>>(),
        special.into_iter().map(|r| r.document).collect::<Vec<_>>()
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
    accepted(&send(&mut session, save_documents(&original, t2_44f_documents())));
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
        let reply = send(&mut session, save_documents(&original, documents));
        (reply, session.draft)
    };
    // Box 4 equal to box 1 passes; one cent above is rejected with the
    // client rule's key and nothing is stored.
    accepted(&attempt(vec![document(0, w2g(100_000, 100_000))]).0);
    let (reply, draft) = attempt(vec![document(0, w2g(100_000, 100_001))]);
    rejected_with(reply, "W2G_WITHHOLDING_ABOVE_WINNINGS");
    assert_eq!(draft, original);
    let mut session = Interview::new(TEST_USER, original.clone());
    rejected_with(
        send(&mut session, catalog_preview_document(w2g(100_000, 100_001))),
        "W2G_WITHHOLDING_ABOVE_WINNINGS",
    );
}

#[test]
fn gambling_losses_twice_is_rejected_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&send(
        &mut session,
        save_documents(
            &original,
            vec![document(0, gambling_losses(10_000, 50_000)), document(0, w2g(100_000, 0))],
        ),
    ));
    let mut session = Interview::new(TEST_USER, original.clone());
    rejected_with(
        send(
            &mut session,
            save_documents(
                &original,
                vec![
                    document(0, gambling_losses(10_000, 50_000)),
                    document(0, gambling_losses(0, 1)),
                ],
            ),
        ),
        "GAMBLING_LOSSES_TWICE",
    );
    assert_eq!(session.draft, original);
}

#[test]
fn special_gambling_is_left_out_with_a_notice_under_the_page() {
    for kind in [
        IncomeDocument::W2g(W2g {
            state_details: Default::default(),
            winnings: 100_000,
            withholding: 0,
            special: true,
        }),
        IncomeDocument::GamblingLosses(GamblingLosses {
            other_winnings: 100_000,
            losses: 0,
            special: true,
        }),
    ] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        accepted(&send(&mut session, save_documents(&original, vec![document(0, kind)])));
        let r = supported(&session.draft);
        assert!(noted(&r, "WARN_GAMBLING_LEFT_OUT"), "{r:?}");
    }
}

#[test]
fn w2g_with_losses_keeps_the_page_closed() {
    // Only fixtures can store a W-2G label with losses: the page cannot show
    // it (C6), so the topic is closed with a message.
    let mut d = complete(FederalStatus::Single);
    d.income.push(FederalIncomeEntry {
        refund_management: None,
        state_source: Default::default(),
        id: d.next_id,
        label: "".into(),
        owner: Owner::Taxpayer,
        income: FederalIncome::Gambling {
            data: FederalGambling {
                winnings: 100_000,
                losses: 10_000,
                withholding: 0,
                own_records: false,
                ordinary: true,
            },
        },
        community: CommunityShare::NotNeeded,
    });
    d.next_id += 1;
    let mut s = Interview::new(TEST_USER, d.clone());
    let reply = send(&mut s, go_to(&d, PAGE_INCOME, TaxSection::Gambling));
    assert!(refusal(reply).contains("cannot show yet"), "expected the page closed");
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
        refund_management: None,
        state_source: Default::default(),
        id: d.next_id,
        label: "".into(),
        owner: Owner::Taxpayer,
        income: document_income(&gambling_losses(200_000, 300_000)),
        community: CommunityShare::NotNeeded,
    });
    d.next_id += 1;
    let mut s = Interview::new(TEST_USER, d);
    let page = match send(&mut s, P::OpenInterview { user: TEST_USER }) {
        Reply::Edit(page @ P::SaveDeductions { .. }) => page,
        other => panic!("expected the deductions page, got {other:?}"),
    };
    assert_eq!(deductions_value(&page).0.route, DeductionRoute::Compare);
    // Saved on Compare, the itemized hub shows the derived row, which opens
    // the income topic; its Back and save return to the hub.
    // The itemized walk asks More itemized deductions first (gate fixes
    // batch A).
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
fn iso_share_sale(more: IsoShareSaleMore) -> IsoShareSale {
    IsoShareSale {
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
        more,
    }
}
fn save_iso_sale(s: &mut Interview, documents: Vec<IncomeDocument>) -> Reply {
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::InvestmentSales,
        documents.into_iter().map(|d| document(0, d)).collect(),
    );
    send(s, request)
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
    let sale = iso_share_sale(IsoShareSaleMore::default());
    accepted(&save_iso_sale(
        &mut s,
        vec![
            IncomeDocument::Form3921(form_3921(sold_and_option_cost(250.0, 40_000))),
            IncomeDocument::IsoShareSale(sale.clone()),
        ],
    ));
    let id = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::IsoSale { .. }))
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
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    let rows = page_documents(&reopened).to_vec();
    assert!(rows.iter().any(|row| row.document == IncomeDocument::IsoShareSale(sale.clone())));
    let before = s.draft.income.clone();
    let request = save_topic(&s.draft.clone(), TaxSection::InvestmentSales, rows);
    accepted(&send(&mut s, request));
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
        more: IsoShareSaleMore::default(),
    };
    accepted(&save_iso_sale(&mut s, vec![IncomeDocument::IsoShareSale(sale.clone())]));
    let entry =
        s.draft.income.iter().find(|e| matches!(e.income, FederalIncome::IsoSale { .. })).unwrap();
    assert_eq!(entry.label, "");
    let FederalIncome::IsoSale { data } = &entry.income else { panic!() };
    assert!(data.in_wages);
    let id = entry.id;
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
    accepted(&save_iso_sale(&mut s, vec![IncomeDocument::IsoShareSale(later.clone())]));
    let entry =
        s.draft.income.iter().find(|e| matches!(e.income, FederalIncome::IsoSale { .. })).unwrap();
    let FederalIncome::IsoSale { data } = &entry.income else { panic!() };
    assert!(!data.in_wages);
    let id = entry.id;
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &alloc::format!("iso_sale_{id}_{suffix}"));
    assert_eq!(line("compensation"), 0);
    assert_eq!(line("capital_gain"), 4_000);
    assert_eq!(line("amt_capital_gain"), 2_000);
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    assert_eq!(page_documents(&reopened)[0].document, IncomeDocument::IsoShareSale(later));
}

#[test]
fn iso_sale_one_year_after_exercise_is_disqualifying() {
    // Boundary: the sale must be MORE than 1 year after the exercise.
    // Granted 2023-01-10, exercised 2025-08-03 at $10 (box 4 $30), 100
    // shares sold 2026-08-03 for $5,000: compensation $2,000 (the spread,
    // below the $4,000 gain); a day later none.
    let sale = |sold: u32| IsoShareSale {
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
        more: IsoShareSaleMore::default(),
    };
    assert_eq!(sale(20260803).record().compensation(), 200_000);
    assert_eq!(sale(20260804).record().compensation(), 0);
}

#[test]
fn iso_sale_page_rules_and_special_answer() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // Exercised before granted, sold before exercised (both sides), and a
    // sale outside 2026 (server only), each refused by its key.
    let mut v = iso_share_sale(IsoShareSaleMore::default());
    v.granted = 20240602;
    rejected_with(
        save_iso_sale(&mut s, vec![IncomeDocument::IsoShareSale(v)]),
        "ISO_EXERCISED_BEFORE_GRANTED",
    );
    let mut v = iso_share_sale(IsoShareSaleMore::default());
    v.exercised = 20260804;
    rejected_with(
        save_iso_sale(&mut s, vec![IncomeDocument::IsoShareSale(v)]),
        "ISO_SOLD_BEFORE_EXERCISED",
    );
    let mut v = iso_share_sale(IsoShareSaleMore::default());
    v.sold = 20251231;
    rejected_with(
        save_iso_sale(&mut s, vec![IncomeDocument::IsoShareSale(v)]),
        "ISO_SALE_NOT_IN_RETURN_YEAR",
    );
    // The special answer is stored and refused by name in the calculation.
    let special = iso_share_sale(IsoShareSaleMore {
        in_wages: true,
        selling_fees: 0,
        acquisition_costs: 0,
        option_cost: 0,
        special: true,
    });
    accepted(&save_iso_sale(&mut s, vec![IncomeDocument::IsoShareSale(special.clone())]));
    // The sale is left out with a notice; the estimate proceeds.
    assert!(noted(&supported(&s.draft), "WARN_ISO_SALE_LEFT_OUT"));
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    assert_eq!(page_documents(&reopened)[0].document, IncomeDocument::IsoShareSale(special));
    // Fees and commissions reopen in the fold.
    let fees = iso_share_sale(IsoShareSaleMore {
        in_wages: true,
        selling_fees: 2_500,
        acquisition_costs: 1_000,
        option_cost: 0,
        special: false,
    });
    accepted(&save_iso_sale(&mut s, vec![IncomeDocument::IsoShareSale(fees.clone())]));
    let reopened = open_topic(&mut s, TaxSection::InvestmentSales);
    assert_eq!(page_documents(&reopened)[0].document, IncomeDocument::IsoShareSale(fees));
}

#[test]
fn w2_reported_tips_alone_need_no_form_4137_record() {
    // Box 12 code TP with every tip reported to the employer: no Form 4137,
    // so no employee-tip record is required and the estimate is available.
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2(6_000_000);
    w.more.box12 = vec![Box12Entry { code: W2Code::TP, amount: 100_000 }];
    w.answers.tips_listed_occupation = WageDeductionEligibility::Eligible;
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(w))]),
    ));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
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
    let mut w = w2(6_000_000);
    w.more.box12 = vec![Box12Entry { code: W2Code::TP, amount: 100_000 }];
    w.answers.tips_listed_occupation = WageDeductionEligibility::Eligible;
    // One month: $300 received, $100 of it reported; the TP $1,000 covers the
    // rest of the year's reported tips without a record.
    w.answers.tip_records = vec![CashTipMonth { month: 3, received: 30_000, reported: 10_000 }];
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(w))]),
    ));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    assert_eq!(data.work_deductions.w2_tips(), 100_000);
    assert!(data.work_deductions.form4137_tips() > data.work_deductions.w2_tips());
    let r = estimate(&session.draft);
    assert!(!r.issues.iter().any(|i| i.message.contains("Form 4137")), "{r:?}");
}

#[test]
fn w2_import_keeps_daily_tip_records_and_box_12_answers() {
    let model = include_str!("../../federal/federal_model.rs");
    assert!(import_unselected(model, "pub struct W2Answers {"));
    assert!(!model.contains("#[libertas_llm_doc_field]\n    pub answers: W2Answers"));
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut w = w2(6_000_000);
    w.more.box12 = vec![
        Box12Entry { code: W2Code::TP, amount: 100_000 },
        Box12Entry { code: W2Code::TT, amount: 80_000 },
    ];
    w.answers = W2Answers {
        medicaid_waiver: 0,
        noncash_tips: 0,
        statutory_business: None,
        paying_scorporation: None,
        tip_records: vec![CashTipMonth { month: 3, received: 150_000, reported: 100_000 }],
        tips_listed_occupation: WageDeductionEligibility::Eligible,
        overtime_flsa_required: WageDeductionEligibility::Ineligible,
        coverage: crate::W2Coverage::ChurchEmployee,
        disability: DisabilityIncome::None,
        abroad: 0,
        deferred_pension: 0,
        inmate_pay: 0,
    };
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(w))]),
    ));
    // The import: box 8 printed, box 12 replaced by the form's codes.
    let reopened = page_documents(&open_documents(&mut session)).to_vec();
    let IncomeDocument::W2(mut imported) = reopened[0].document.clone() else {
        panic!("expected the W-2")
    };
    let answers = imported.answers.clone();
    imported.more.allocated_tips = 40_000;
    imported.more.box12 = vec![
        Box12Entry { code: W2Code::TP, amount: 110_000 },
        Box12Entry { code: W2Code::TT, amount: 90_000 },
    ];
    let request = save_documents(
        &session.draft.clone(),
        vec![document(reopened[0].id, IncomeDocument::W2(imported))],
    );
    accepted(&send(&mut session, request));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { unreachable!() };
    // The daily record still replaces the box 8 allocation.
    assert!(matches!(
        &data.employee_tips,
        Some(EmployeeTips { cash: CashTipRecords::Actual { months }, .. }) if months.len() == 1
    ));
    let facts = &data.work_deductions;
    assert!(matches!(
        facts.tips_claim,
        TipsClaim::Yes { w2_tips: 110_000, tips_eligible: true, .. }
    ));
    assert!(matches!(
        facts.overtime_claim,
        OvertimeClaim::Yes { overtime: 90_000, overtime_eligible: false }
    ));
    assert_eq!(data.coverage, W2Coverage::ChurchEmployee);
    let page = open_documents(&mut session);
    let IncomeDocument::W2(again) = &page_documents(&page)[0].document else {
        panic!("expected the W-2")
    };
    assert_eq!(again.answers, answers);
    // A TP or TT code with no answer is refused on the server too.
    let mut unanswered = again.clone();
    unanswered.answers.tips_listed_occupation = WageDeductionEligibility::NoAmount;
    let request = save_documents(
        &session.draft.clone(),
        vec![document(reopened[0].id, IncomeDocument::W2(unanswered))],
    );
    rejected_with(send(&mut session, request), "W2_TP_OCCUPATION_NEEDED");
}

#[test]
fn gambling_kinds_are_told_apart_by_the_record_not_by_a_name() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&send(&mut session, save_documents(&original, t2_44f_documents())));
    // Whatever a record is called, it reopens as the kind it is.
    session.draft.income[1].label = "Gambling losses and other winnings".into();
    session.draft.income[3].label = "W-2G".into();
    let mut again = restart(session.draft.clone());
    let documents: Vec<IncomeDocument> =
        page_documents(&open_topic(&mut again, TaxSection::Gambling))
            .iter()
            .map(|item| item.document.clone())
            .collect();
    assert_eq!(
        documents,
        t2_44f_documents().into_iter().map(|item| item.document).collect::<Vec<_>>()
    );
}

#[test]
fn wage_specials_w2_save_reopens_new_source_facts_and_personal_answers() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut form = w2(2_600_000);
    form.more.allocated_tips = 300_000;
    form.answers.tips_listed_occupation = WageDeductionEligibility::Eligible;
    form.answers.noncash_tips = 20_000;
    form.answers.medicaid_waiver = 10_000;
    form.more.box12 = vec![
        Box12Entry { code: W2Code::Q, amount: 500_000 },
        Box12Entry { code: W2Code::II, amount: 100_000 },
        Box12Entry { code: W2Code::TA, amount: 100_000 },
    ];
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(form.clone()))]),
    ));
    let page = open_documents(&mut session);
    assert_eq!(page_documents(&page)[0].document, IncomeDocument::W2(form.clone()));
    let saved = session.draft.clone();
    assert_eq!(roundtrip_native_draft(saved.clone()), saved);
    let id = page_documents(&page)[0].id;
    form.more.box12.push(Box12Entry { code: W2Code::TP, amount: 600_000 });
    form.more.social_tips = 600_000;
    form.social_tax = 198_400;
    form.medicare_wages = 3_200_000;
    form.medicare_tax = 46_400;
    let request =
        save_documents(&session.draft.clone(), vec![document(id, IncomeDocument::W2(form))]);
    accepted(&send(&mut session, request));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    assert_eq!(data.work_deductions.tips(), 900_000);
    assert_eq!(data.medicaid_waiver, 10_000);
}

#[test]
fn wage_specials_w2_server_twins_reject_inconsistent_boxes() {
    for key in [
        "W2_TP_OCCUPATION_NEEDED",
        "W2_SCHEDULE_1_PAY_ABOVE_WAGES",
        "W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES",
        "W2_TA_ABOVE_LIMIT",
    ] {
        let original = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut form = w2(1_800_000);
        match key {
            "W2_TP_OCCUPATION_NEEDED" => form.more.allocated_tips = 300_000,
            "W2_SCHEDULE_1_PAY_ABOVE_WAGES" => form.answers.medicaid_waiver = 2_000_000,
            "W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES" => {
                form.more.payroll = W2Payroll::Railroad {
                    compensation: 1_800_000,
                    tier_1: 111_600,
                    tier_2: 0,
                    medicare: 26_100,
                    additional_medicare: 0,
                }
            }
            _ => form.more.box12 = vec![Box12Entry { code: W2Code::TA, amount: 260_000 }],
        }
        let before = session.draft.clone();
        rejected_with(
            send(
                &mut session,
                save_documents(&original, vec![document(0, IncomeDocument::W2(form))]),
            ),
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
    accepted(&send(
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::W2(form.clone()))]),
    ));
    let page = open_documents(&mut session);
    assert_eq!(page_documents(&page)[0].document, IncomeDocument::W2(form));
    let model = include_str!("../../federal/federal_model.rs");
    assert!(model.contains("#[libertas_llm_doc_field(blank = SocialSecurity)]\n    #[libertas_default(SocialSecurity)]\n    pub payroll: W2Payroll"));
    assert!(import_unselected(model, "pub struct W2Answers {"));
}

#[test]
fn wage_specials_statutory_override_lifecycle_and_forged_source() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(0)));
    let business_id = d.income[1].id;
    let mut session = Interview::new(TEST_USER, d.clone());
    let mut form = w2(5_000_000);
    form.more.statutory = true;
    form.answers.statutory_business = Some(business_id);
    accepted(&send(
        &mut session,
        save_documents(&d, vec![document(1, IncomeDocument::W2(form.clone()))]),
    ));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    assert_eq!(data.statutory_business, Some(business_id));
    let saved = session.draft.clone();
    assert_eq!(roundtrip_native_draft(saved.clone()), saved);
    accepted(&send(&mut session, save_topic(&saved, TaxSection::SelfEmployment, vec![])));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    assert_eq!(data.statutory_business, None);
    assert_eq!(data.wages, 5_000_000);
    assert!(session.draft.income.iter().all(|e| e.id != business_id));
    let now = session.draft.clone();
    form.answers.statutory_business = Some(999);
    rejected_with(
        current_income_refusal(
            "income-source-7638",
            &mut session,
            save_documents(&now, vec![document(1, IncomeDocument::W2(form))]),
            "ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME",
            "[documents/documents/0/document/answers/statutory_business]",
        ),
        "ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME",
    );
    assert_eq!(session.draft, now);
}

#[test]
fn wage_specials_named_eligibility_answers_round_trip_and_classify_sources() {
    for eligibility in [WageDeductionEligibility::Ineligible, WageDeductionEligibility::Eligible] {
        let d = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, d.clone());
        let mut form = w2(2_600_000);
        form.more.box12 = vec![
            Box12Entry { code: W2Code::TP, amount: 300_000 },
            Box12Entry { code: W2Code::TT, amount: 100_000 },
        ];
        form.answers.tips_listed_occupation = eligibility;
        form.answers.overtime_flsa_required = eligibility;
        accepted(&send(
            &mut session,
            save_documents(&d, vec![document(1, IncomeDocument::W2(form.clone()))]),
        ));
        assert_eq!(
            page_documents(&open_documents(&mut session))[0].document,
            IncomeDocument::W2(form)
        );
        let mut no_amount = w2(2_600_000);
        no_amount.more.box12 = vec![Box12Entry { code: W2Code::TT, amount: 100_000 }];
        let now = session.draft.clone();
        rejected_with(
            send(
                &mut session,
                save_documents(&now, vec![document(1, IncomeDocument::W2(no_amount))]),
            ),
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
    add(&mut d, income(FederalIncome::Scorp { data: complete_ordinary_scorp(20_000, true) }));
    let source_id = d.income.last().unwrap().id;
    let mut session = Interview::new(TEST_USER, d.clone());
    if client.required_calculation(&mut session) {
        return;
    }
    let page = open_documents(&mut session);
    client.write("SEED", &page);
    let P::SaveIncomes { wage_scorporations, allowed_wage_scorporations, .. } = page else {
        panic!()
    };
    assert_eq!(wage_scorporations.iter().map(|row| row.id).collect::<Vec<_>>(), vec![source_id]);
    assert_eq!(allowed_wage_scorporations, vec![0]);
    let mut form = w2(4_000_000);
    form.answers.paying_scorporation =
        Some(crate::WageScorpEmployer::Corporation { source: source_id });
    let answers = save_documents(&d, vec![document(0, IncomeDocument::W2(form.clone()))]);
    client.save(&mut session, answers);
    let data = session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::Wage { data } => Some(data),
            _ => None,
        })
        .unwrap();
    assert_eq!(data.paying_scorporation, Some(source_id));
    session = restart(session.draft.clone());
    let reopened = open_documents(&mut session);
    client.write("REOPENED", &reopened);
    assert_eq!(page_documents(&reopened)[0].document, IncomeDocument::W2(form.clone()));
    let saved = session.draft.clone();
    form.answers.paying_scorporation =
        Some(crate::WageScorpEmployer::Corporation { source: 999_999 });
    rejected_with(
        current_source_refusal(
            "health-context-15",
            &mut session,
            save_documents(&saved, vec![document(1, IncomeDocument::W2(form))]),
            "W2_SCORP_EMPLOYER_UNAVAILABLE",
            Some("[documents/documents/0/document/answers/paying_scorporation/source]"),
            SourceScope::Health,
        ),
        "W2_SCORP_EMPLOYER_UNAVAILABLE",
    );
    assert_eq!(session.draft, saved);
    accepted(&send(&mut session, save_topic(&saved, TaxSection::PassThrough, vec![])));
    let FederalIncome::Wage { data } = &session.draft.income[0].income else { panic!() };
    assert_eq!(data.paying_scorporation, None);
    assert_eq!(data.wages, 4_000_000);
    let page = open_documents(&mut session);
    let P::SaveIncomes { wage_scorporations, allowed_wage_scorporations, .. } = page else {
        panic!()
    };
    assert!(wage_scorporations.is_empty());
    assert!(allowed_wage_scorporations.is_empty());
}

#[test]
fn iso_quantity_above_native_domain_is_refused_before_commit() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let mut page = iso_share_sale(IsoShareSaleMore::default());
    page.shares = f64::MAX;
    rejected_with(
        save_iso_sale(&mut session, vec![IncomeDocument::IsoShareSale(page)]),
        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
    );
    assert_eq!(session.draft, before);
}
