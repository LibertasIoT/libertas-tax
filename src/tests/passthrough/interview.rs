// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
use crate::interview::document_income;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, native_w2 as w2};
use super::support::client::income_pages::{open_topic, page_documents, save_topic};
use crate::tests::cross::interview::{
    PAGE_INCOME, adult, back, benefits_request, car_1099c, earlier_installment,
    estate_activity_page, estate_final_amounts, estate_final_source, estate_nol_original_period,
    estate_nol_page, estate_nol_weeks_history, form_8615_return, hub_rows, income_ticks, k1_1041,
    k1_1041_more_with, k1_1065, k1_1065_benefits, k1_1065_carried_zero, k1_1065_continuing,
    k1_1065_more, k1_1120s, k1_1120s_carried_zero, k1_1120s_continuing, k1_1120s_fixture,
    k1_1120s_more_mut, k1_rental_boxes, k1_sale_input_statement, main_estate_qbi, no_property_qbi,
    open_adjustments_hub, open_benefits, open_itemized_topic, partner_benefits_session,
    past_people, problem_text, refusal, reopened_k1s, row_of, save_1065s, save_benefits,
    save_deductions, save_k1_1120s, save_k1s, save_more_credits, save_setup, save_you,
    se_health_test_months, se_health_test_year, tick, told_row,
};
fn stored_k1_1041(session: &Interview) -> (alloc::string::String, FederalEstateTrustPortfolioK1) {
    session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::EstateTrustPortfolio { data } => Some((e.label.clone(), data.clone())),
            _ => None,
        })
        .expect("a stored Schedule K-1 (Form 1041)")
}

#[test]
fn k1_1041_signed_activities_get_distinct_keys_and_preserve_history_on_reopen() {
    let mut page = k1_1041("Synthetic multi-activity trust");
    let mut profit = estate_activity_page("Store", crate::EstateActivityKind::Business, 1_000_000);
    profit.carried = crate::EstatePassiveCarry { regular: 200_000, alternative: 300_000 };
    let mut loss =
        estate_activity_page("Rental", crate::EstateActivityKind::RentalRealEstate, -600_000);
    loss.depreciation = 100_000;
    page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: 1_000_000,
        box7: -600_000,
        box8: 0,
        activities: vec![profit, loss],
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let (_, stored) = stored_k1_1041(&session);
    let ids: Vec<_> = stored.activities.activities.iter().map(|a| a.id).collect();
    assert!(ids.iter().all(|id| *id > 0));
    assert_ne!(ids[0], ids[1]);
    assert_eq!(stored.activities.activities[0].carried.regular, 200_000);
    assert_eq!(stored.activities.activities[0].carried.alternative, 300_000);
    // 10,000 profit - 2,000 prior - 7,000 current rental loss = 1,000.
    let result = supported(&session.draft);
    assert_eq!(value(&result, "schedule_1_rental_income"), 1_000);
    assert_eq!(
        crate::interview::document_amount(
            &session
                .draft
                .income
                .iter()
                .find(|e| matches!(e.income, FederalIncome::EstateTrustPortfolio { .. }))
                .unwrap()
                .income,
            false,
        ),
        None
    );
    let before = session.draft.clone();
    let items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    assert_eq!(items.len(), 1);
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, items);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, before);
    let mut restarted = restart(session.draft.clone());
    assert_eq!(reopened_k1s(&mut restarted), reopened_k1s(&mut session));
    assert_eq!(estimate(&restarted.draft), result);
}

#[test]
fn k1_1041_client_server_reconcile_signed_boxes_and_reject_forged_activity_identity() {
    let mut page = k1_1041("Synthetic multi-activity trust");
    let activity =
        estate_activity_page("Rental", crate::EstateActivityKind::RentalRealEstate, -100_000);
    page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: 0,
        box7: -99_999,
        box8: 0,
        activities: vec![activity],
    });
    assert_eq!(page.rule(), Some("ESTATE_BOX7_RECONCILE"));
    rejected_with(save_k1s(vec![page.clone()]).1, "ESTATE_BOX7_RECONCILE");
    let K1EstateTrustBusiness::Amounts(a) = &mut page.business else { unreachable!() };
    a.box7 = -100_000;
    assert_eq!(page.rule(), None);
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(p) = &mut items[0].document else { unreachable!() };
    let K1EstateTrustBusiness::Amounts(a) = &mut p.business else { unreachable!() };
    a.activities[0].id = 987_654;
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, items);
    let reply = current_income_refusal(
        "income-source-2018",
        &mut session,
        request,
        "ESTATE_ACTIVITY_LINK_INVALID",
        "[documents/documents/0/document/business/activities/0]",
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn k1_1041_portfolio_rules_still_reject_cross_box_overstatement() {
    let mut page = k1_1041("Synthetic trust");
    page.ordinary_dividends = 100;
    page.qualified_dividends = 101;
    rejected_with(save_k1s(vec![page]).1, "K1_1041_QUALIFIED_ABOVE_2A");
    let mut page = k1_1041("Synthetic trust");
    page.long_gain = 100;
    page.rate_28_gain = 50;
    page.unrecaptured_gain = 51;
    rejected_with(save_k1s(vec![page]).1, "K1_1041_4B_4C_ABOVE_4A");
    let mut page = k1_1041("Synthetic trust");
    page.more.tax_exempt_interest = 100;
    page.more.private_activity_interest = 101;
    rejected_with(save_k1s(vec![page]).1, "K1_1041_PRIVATE_ABOVE_14A");
}

#[test]
fn k1_topic_is_listed_on_the_checklist_and_the_income_hub() {
    // Question C5: the checklist's Schedule K-1 tick plans the topic; the
    // walk opens its page offering the Form 1041 and 1120-S kinds, and the hub
    // lists the topic with the K-1's count and amount once one is entered.
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("m3-hub".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let request = save_you(&s.draft, adult("Casey", "Sample", 19850315));
    let reply = send(&mut s, request);
    let checklist = past_people(&mut s, reply);
    let request = tick(checklist, income_ticks(IncomeTopics::PASS_THROUGH.0));
    let Reply::Edit(P::SaveIncomes {
        topic: TaxSection::PassThrough,
        previous,
        allowed_documents,
        ..
    }) = send(&mut s, request)
    else {
        panic!("expected the Schedule K-1 page")
    };
    // Amended for T2-29p: the Form 1065 kind comes first.
    assert_eq!(
        allowed_documents,
        vec![
            libertas_macros::variant_index!(IncomeDocument::K1Partnership) as i32,
            libertas_macros::variant_index!(IncomeDocument::K1PartnershipActivities) as i32,
            libertas_macros::variant_index!(IncomeDocument::K1ScorpActivities) as i32,
            libertas_macros::variant_index!(IncomeDocument::K1EstateTrust) as i32,
            libertas_macros::variant_index!(IncomeDocument::K1Scorp) as i32,
        ]
    );
    let request = back(&s.draft, previous);
    let Reply::Response(hub @ P::Hub { .. }) = send(&mut s, request) else {
        panic!("expected the income hub")
    };
    assert_eq!(told_row(&hub), ("What you told us".into(), "Schedule K-1".into()));
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::PassThrough);
    let mut page = k1_1041("Synthetic Family Trust");
    page.interest = 100_000;
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::PassThrough,
        vec![document(0, IncomeDocument::K1EstateTrust(page))],
    );
    accepted(&send(&mut s, request));
    let request = back(&s.draft, PAGE_INCOME);
    let Reply::Response(hub @ P::Hub { .. }) = send(&mut s, request) else {
        panic!("expected the income hub")
    };
    assert!(
        hub_rows(&hub).contains(&(
            "Partnerships, S corporations, estates and trusts".into(),
            "1 Form 1041 K-1 · $1000.00".into(),
            TaxSection::PassThrough
        )),
        "{:?}",
        hub_rows(&hub)
    );
}

#[test]
fn shared_k1_page_types_map_onto_their_records() {
    // Participation (question C12): the shortcuts write answers that make
    // material() No for an investor with fewer than five of the last ten
    // years, Yes at five, and Yes above 500 hours; Other keeps any record.
    let investor = Participation::Investor { prior_years: 4 };
    let facts = investor.record();
    assert_eq!(facts.material(), crate::participation::Participation::No);
    assert_eq!(Participation::from_record(&facts), investor);
    let five = Participation::Investor { prior_years: 5 };
    assert_eq!(five.record().material(), crate::participation::Participation::Yes);
    let worked = Participation::Over500Hours { hours: 501 };
    assert_eq!(worked.record().minutes, 30_060);
    assert_eq!(worked.record().material(), crate::participation::Participation::Yes);
    assert_eq!(Participation::from_record(&worked.record()), worked);
    let mut other = facts.clone();
    other.minutes = 12_000;
    other.regular = true;
    let choice = Participation::from_record(&other);
    assert_eq!(choice, Participation::Other { facts: other.clone() });
    assert_eq!(choice.record(), other);

    // LimitedParticipation onto PartnerParticipation::Limited.
    for choice in [
        LimitedParticipation::Investor { prior_years: 2 },
        LimitedParticipation::Over500Hours { hours: 600 },
        LimitedParticipation::Other {
            minutes: 12_000,
            prior_ten: 1,
            personal_service: true,
            service_years: 2,
        },
    ] {
        let stored = choice.record();
        assert!(matches!(stored, PartnerParticipation::Limited { .. }));
        assert_eq!(LimitedParticipation::from_record(&stored), Some(choice));
    }
    assert_eq!(
        LimitedParticipation::Over500Hours { hours: 600 }.record().material(),
        crate::participation::Participation::Yes
    );
    assert_eq!(LimitedParticipation::from_record(&PartnerParticipation::General { facts }), None);

    // K1IncomeSource: only "Not needed" unless the return reads the source
    // (T2-cy); each choice reopens from the two stored fields.
    assert_eq!(K1IncomeSource::allowed(false), vec![0]);
    assert_eq!(K1IncomeSource::allowed(true), vec![1, 2, 3]);
    for choice in [
        K1IncomeSource::NotNeeded,
        K1IncomeSource::Services { gross: 500_000 },
        K1IncomeSource::Capital,
        K1IncomeSource::Mixed { value: WorkValue::Amount { compensation: 300_000 } },
        K1IncomeSource::Mixed { value: WorkValue::AtLeastThirtyPercent },
    ] {
        let (source, gross) = choice.record();
        assert_eq!(K1IncomeSource::from_record(source.as_ref(), gross), Some(choice));
    }

    // AMT records (question C10): the same-as-regular statement writes the
    // regular records as the AMT records and reopens from them.
    let scorp = complete_ordinary_scorp(20_000, true);
    let same = ScorpAmtRecords::SameAsRegular.record(&scorp);
    assert_eq!((same.stock, same.adjustment, same.complete), (scorp.opening, 0, true));
    assert_eq!(ScorpAmtRecords::from_record(&scorp, &same), ScorpAmtRecords::SameAsRegular);
    let mut different = same.clone();
    different.adjustment = -50_000;
    assert_eq!(
        ScorpAmtRecords::from_record(&scorp, &different),
        ScorpAmtRecords::Separate { record: different.clone() }
    );
}
fn stored_k1_1120s(session: &Interview) -> ScorpK1 {
    session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Scorp { data } => Some(data.clone()),
            _ => None,
        })
        .expect("a stored Schedule K-1 (Form 1120-S)")
}

#[test]
fn k1_1120s_page_maps_every_branch_onto_the_record_and_reopens_unchanged() {
    // The shortest page: a new shareholder with box 1 only.
    let mut short = k1_1120s("Synthetic Corp A");
    short.ordinary = 500_000;
    short.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 500_000,
        wages: 100_000,
        property: 0,
        service: true,
        current_loss: 0,
        property_qbi: no_property_qbi(),
    };
    // Every other branch: Other boxes, a continuing shareholder with every
    // carryover and a loan, amounts not at risk, participation facts, no
    // section 199A statement and separate AMT records.
    let mut full = k1_1120s("Synthetic Corp B");
    full.ordinary = -300_000;
    full.interest = 10_000;
    full.dividends = 20_000;
    full.qualified_dividends = 15_000;
    full.distributions = 50_000;
    full.qbi = K1ScorpQbi::NoStatement;
    full.participation = Participation::Other {
        facts: ScorpParticipation {
            ordinary_scope: true,
            minutes: 12_000,
            substantially_all: false,
            at_least_as_much: true,
            prior_ten: 2,
            personal_service: false,
            service_years: 0,
            regular: false,
            management_minutes: 0,
            paid_manager: false,
            other_manager_more: false,
            other_business_over_100: false,
        },
    };
    full.more = K1ScorpOtherBoxes {
        amt_section179_income_carry: None,
        allowed_investment_confirmations: Vec::new(),
        section179_carryover_sold: 0,
        section179_sales: alloc::vec::Vec::new(),
        deductions: Vec::new(),
        income: Vec::new(),
        foreign_taxes: 0,
        short: -10_000,
        long: 40_000,
        collectibles: 5_000,
        unrecaptured_partnership: 1_000,
        unrecaptured_fund: 2_000,
        unrecaptured_business: 3_000,
        section1231: 30_000,
        section179: 7_000,
        section179_carryover: 4_000,
        cash_charity: 6_000,
        withholding: 1_500,
        private_activity: 800,
        exempt_interest: 900,
        exempt: 700,
        nondeductible: 600,
        stock_term: K1StockTerm::ShortTerm,
        trading: true,
        credits: Vec::new(),
        unentered: vec![ScorpK1Item::Credits],
        investment_property: Vec::new(),
        investment_confirmed: None,
        housing_confirmed: None,
    };
    full.basis = K1ScorpBasis {
        start: K1ScorpStart::Continuing {
            opening: 2_000_000,
            risk_history: ScorpRiskStart::Unknown,
            carried: K1ScorpCarried {
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
                basis_nondeductible: 1_200,
                losses_first: true,
                basis_short: 1_300,
                basis_long: 1_400,
                basis_collectibles: 100,
                risk_short: 1_500,
                risk_long: 1_600,
                risk_collectibles: 200,
                section1231_basis_loss: 1_700,
                section1231_risk_loss: 1_800,
                section1231_passive_loss: 1_900,
                risk_loss: 0,
                risk_nondeductible: 2_200,
                deducted: 2_300,
                recaptured: 2_000,
                passive_loss: 2_500,
            },
        },
        additions: 40_000,
        loans: vec![ScorpLoan {
            kind: ScorpLoanKind::FormalNote,
            repayment_term: ScorpNoteHolding::NoRepayment,
            principal: 300_000,
            basis: 200_000,
            advances: 10_000,
            repayments: 0,
        }],
        more_at_risk: K1ScorpAtRisk {
            excluded_funding: 5_000,
            protected: 1_000,
            released: 500,
            excluded_returns: 400,
        },
    };
    full.special = vec![crate::ScorpSpecial::Farming];
    // Separate AMT records: the restated regular ones with a box 15 code A
    // adjustment.
    let mut amt = ScorpAmtRecords::SameAsRegular.record(&document_income_scorp(&full));
    amt.adjustment = -25_000;
    full.amt = ScorpAmtRecords::Separate { record: amt };
    // A continuing shareholder whose only Other box is a nonzero trading flag
    // keeps the short Continuing fields and the investor shortcut.
    let mut middle = k1_1120s("Synthetic Corp C");
    middle.ordinary = 1_000_000;
    middle.participation = Participation::Over500Hours { hours: 600 };
    middle.basis.start = k1_1120s_continuing(500_000, 500_000);
    middle.basis.loans = vec![ScorpLoan {
        kind: ScorpLoanKind::OpenAccount,
        repayment_term: ScorpNoteHolding::NoRepayment,
        principal: 100_000,
        basis: 100_000,
        advances: 0,
        repayments: 0,
    }];
    let pages = vec![short.clone(), full.clone(), middle.clone()];
    let (mut session, reply) = save_k1_1120s(pages.clone());
    accepted(&reply);
    // The stored records are what the page writes.
    let records: Vec<_> = session
        .draft
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Scorp { data } => Some((e.label.clone(), data.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(records.len(), 3);
    let (label, data) = &records[1];
    assert_eq!(label, "Synthetic Corp B");
    assert!(!data.basis_complete());
    assert!(!data.risk.complete);
    assert!(!data.passive.complete);
    assert_eq!(data.trade_kind, ScorpTradeKind::FinancialTrading);
    assert_eq!(data.stock_term, Some(GainTerm::Short));
    assert_eq!(data.section179_carryover, 4_000);
    assert_eq!(data.capital.as_ref().unwrap().risk_collectibles, 200);
    assert_eq!(data.section1231.as_ref().unwrap().unrecaptured, 3_000);
    assert_eq!(data.qbi_statement, None);
    let (_, data) = &records[0];
    assert!(data.basis_complete());
    assert_eq!(data.capital, None);
    assert_eq!(data.section1231, None);
    assert_eq!(data.risk.start, ScorpRiskStart::New);
    assert_eq!(data.qbi_statement.as_ref().map(|q| (q.service, q.complete)), Some((true, true)));
    assert_eq!(data.amt.as_ref().map(|a| (a.stock, a.complete)), Some((0, true)));
    // Reopening gives the same pages; saving them unchanged keeps the records.
    assert_eq!(
        reopened_k1s(&mut session),
        pages.into_iter().map(IncomeDocument::K1Scorp).collect::<Vec<_>>()
    );
    let before = session.draft.income.clone();
    let items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, items);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, before);
}
// The regular stored record of a page (its AMT records aside).
fn document_income_scorp(page: &K1Scorp) -> ScorpK1 {
    let FederalIncome::Scorp { mut data } = document_income(&IncomeDocument::K1Scorp(page.clone()))
    else {
        unreachable!()
    };
    data.amt = None;
    data
}

#[test]
fn k1_1120s_rules_reject_on_both_sides_at_their_boundaries() {
    // Each client rule's server twin: the boundary is accepted, one cent
    // past it is rejected with the rule's key.
    // The page's "Losses carried from 2025" branch, opened with every amount zero.
    fn continuing(page: &mut K1Scorp) -> &mut K1ScorpCarried {
        if matches!(page.basis.start, K1ScorpStart::New) {
            page.basis.start = k1_1120s_continuing(100_000, 100_000);
        }
        let K1ScorpStart::Continuing { carried, .. } = &mut page.basis.start else {
            unreachable!()
        };
        carried
    }
    type Edit = Box<dyn Fn(&mut K1Scorp, i64)>;
    let cases: Vec<(&str, Edit)> = vec![
        (
            "SCORP_QUALIFIED_ABOVE_DIVIDENDS",
            Box::new(|p, extra| {
                p.dividends = 100_000;
                p.qualified_dividends = 100_000 + extra;
            }),
        ),
        (
            "SCORP_QBI_ABOVE_BOX_1",
            Box::new(|p, extra| {
                p.ordinary = 100_000;
                p.qbi = K1ScorpQbi::Statement {
                    ptp_income: 0,
                    reit_dividends: 0,
                    income: 100_000 + extra,
                    wages: 0,
                    property: 0,
                    service: false,
                    current_loss: 0,
                    property_qbi: no_property_qbi(),
                };
            }),
        ),
        (
            "SCORP_PRIVATE_ACTIVITY_ABOVE_EXEMPT",
            Box::new(|p, extra| {
                let K1ScorpOtherBoxes { private_activity, exempt_interest, .. } =
                    k1_1120s_more_mut(p);
                *exempt_interest = 50_000;
                *private_activity = 50_000 + extra;
            }),
        ),
        (
            "SCORP_NONDEDUCTIBLE_CARRY_NEEDS_ELECTION",
            Box::new(|p, extra| {
                // A zero carryover needs no election; one cent does.
                {
                    let K1ScorpCarried { basis_nondeductible, .. } = continuing(p);
                    *basis_nondeductible = extra;
                }
            }),
        ),
        (
            "SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG",
            Box::new(|p, extra| {
                let K1ScorpCarried { basis_long, basis_collectibles, .. } = continuing(p);
                *basis_long = 20_000;
                *basis_collectibles = 20_000 + extra;
            }),
        ),
        (
            "SCORP_CARRIED_COLLECTIBLES_ABOVE_LONG",
            Box::new(|p, extra| {
                let K1ScorpCarried { risk_long, risk_collectibles, .. } = continuing(p);
                *risk_long = 20_000;
                *risk_collectibles = 20_000 + extra;
            }),
        ),
    ];
    for (key, edit) in cases {
        let mut page = k1_1120s("Synthetic Corp");
        edit(&mut page, 0);
        let (_, reply) = save_k1_1120s(vec![page]);
        accepted(&reply);
        let mut page = k1_1120s("Synthetic Corp");
        edit(&mut page, 1);
        let (_, reply) = save_k1_1120s(vec![page]);
        rejected_with(reply, key);
    }
    // A box 1 loss allows no positive QBI; with the election a carried
    // nondeductible expense is accepted.
    let mut page = k1_1120s("Synthetic Corp");
    page.ordinary = -100;
    page.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 1,
        wages: 0,
        property: 0,
        service: false,
        current_loss: 0,
        property_qbi: no_property_qbi(),
    };
    rejected_with(save_k1_1120s(vec![page]).1, "SCORP_QBI_ABOVE_BOX_1");
    let mut page = k1_1120s("Synthetic Corp");
    {
        let K1ScorpCarried { basis_nondeductible, losses_first, .. } = continuing(&mut page);
        *basis_nondeductible = 1;
        *losses_first = true;
    }
    accepted(&save_k1_1120s(vec![page]).1);
    // Each other box is listed once.
    let mut page = k1_1120s("Synthetic Corp");
    {
        let K1ScorpOtherBoxes { unentered, .. } = k1_1120s_more_mut(&mut page);
        *unentered = vec![ScorpK1Item::RentalDisposition, ScorpK1Item::Credits];
    }
    accepted(&save_k1_1120s(vec![page.clone()]).1);
    {
        let K1ScorpOtherBoxes { unentered, .. } = k1_1120s_more_mut(&mut page);
        *unentered = vec![ScorpK1Item::RentalDisposition, ScorpK1Item::RentalDisposition];
    }
    rejected_with(save_k1_1120s(vec![page]).1, "SCORP_BOX_TWICE");
}

#[test]
fn k1_1120s_qbi_loss_tracking_follows_the_values() {
    // With a section 199A statement the QBI loss tracking sits beside it and
    // is always present: zero and empty lists when nothing applies. On both
    // sides a current QBI loss portion fits the box 1 (or box 9) loss, a
    // carried loss needs its loss-year history, and section 1231 QBI amounts
    // need this year's section 1231 item (decided by the K-1's own boxes).
    let statement = |loss_qbi: ScorpLossQbi, property_qbi| K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        current_loss: loss_qbi.current,
        property_qbi,
    };
    // No loss: the stored sections are empty.
    let (session, reply) = save_k1_1120s(vec![k1_1120s("Synthetic Corp")]);
    accepted(&reply);
    let record = stored_k1_1120s(&session);
    assert!(record.loss_qbi.is_empty() && record.property_qbi.is_empty());
    // A box 1 loss: the QBI portion fits the loss; zero (no QBI loss) is
    // a value too.
    let mut page = k1_1120s("Synthetic Corp");
    page.ordinary = -100_000;
    accepted(&save_k1_1120s(vec![page.clone()]).1);
    let mut loss = ScorpLossQbi { current: 100_001, ..Default::default() };
    page.qbi = statement(loss.clone(), no_property_qbi());
    rejected_with(save_k1_1120s(vec![page.clone()]).1, "SCORP_QBI_LOSS_RECORDS_NEEDED");
    loss.current = 100_000;
    page.qbi = statement(loss.clone(), no_property_qbi());
    let (mut session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    assert_eq!(stored_k1_1120s(&session).loss_qbi, loss);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page.clone())]);
    // Without a statement the K-1 adds no QBI and stores empty sections.
    page.qbi = K1ScorpQbi::NoStatement;
    let (session, reply) = save_k1_1120s(vec![page]);
    accepted(&reply);
    assert!(stored_k1_1120s(&session).loss_qbi.is_empty());
    // A carried basis loss needs its loss-year history.
    let mut page = k1_1120s("Synthetic Corp");
    let mut carried = k1_1120s_carried_zero();
    {
        let K1ScorpCarried { basis_loss, .. } = &mut carried;
        *basis_loss = 300_000;
    }
    page.basis.start =
        K1ScorpStart::Continuing { opening: 0, risk_history: ScorpRiskStart::Unknown, carried };
    rejected_with(save_k1_1120s(vec![page.clone()]).1, "SCORP_QBI_LOSS_RECORDS_NEEDED");
    let loss = ScorpLossQbi {
        history: vec![loss_year(2024, 3_000, 3_000, 3_000, 3_000)],
        ..Default::default()
    };
    page.qbi_records.ordinary.history = loss.history.clone();
    page.qbi = statement(loss, no_property_qbi());
    accepted(&save_k1_1120s(vec![page]).1);
    // A box 9 gain: the statement amounts hold its QBI portion; zero (no
    // QBI portion) is a value too. Without a box 9 or box 8c business item
    // the amounts must stay zero.
    let items = K1ScorpPropertyQbi { gain: 50_000, qualified_1250: None, current: 0 };
    let mut page = k1_1120s("Synthetic Corp");
    page.ordinary = 100_000;
    {
        let K1ScorpOtherBoxes { section1231, .. } = k1_1120s_more_mut(&mut page);
        *section1231 = 50_000;
    }
    let (session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    assert_eq!(stored_k1_1120s(&session).property_qbi.gain, 0);
    page.qbi = statement(Default::default(), items.clone());
    let (session, reply) = save_k1_1120s(vec![page]);
    accepted(&reply);
    let stored = stored_k1_1120s(&session).property_qbi;
    assert_eq!((stored.gain, stored.complete), (50_000, true));
    let mut page = k1_1120s("Synthetic Corp");
    page.qbi = statement(Default::default(), items);
    rejected_with(save_k1_1120s(vec![page.clone()]).1, "SCORP_QBI_PROPERTY_RECORDS_NEEDED");
    page.qbi = statement(
        Default::default(),
        K1ScorpPropertyQbi { qualified_1250: Some(1), ..no_property_qbi() },
    );
    rejected_with(save_k1_1120s(vec![page]).1, "SCORP_QBI_PROPERTY_RECORDS_NEEDED");
    // A carried section 1231 loss alone keeps zero statement amounts and
    // needs only its loss-year history; the confirmation is not read.
    let mut page = k1_1120s_carried_section1231_loss();
    page.qbi = statement(Default::default(), no_property_qbi());
    rejected_with(save_k1_1120s(vec![page.clone()]).1, "SCORP_QBI_PROPERTY_RECORDS_NEEDED");
    page.qbi_records.property.history = vec![loss_year(2024, 2_000, 2_000, 2_000, 2_000)];
    let (mut session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    let stored = stored_k1_1120s(&session).property_qbi;
    assert_eq!(stored.losses.history.len(), 1);
    assert!(stored.complete);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page)]);
}

// A shareholder whose only section 1231 item is a $2,000 loss carried from
// 2025 because of basis.
fn k1_1120s_carried_section1231_loss() -> K1Scorp {
    let mut page = k1_1120s("Synthetic Corp");
    let mut carried = k1_1120s_carried_zero();
    {
        let K1ScorpCarried { section1231_basis_loss, .. } = &mut carried;
        *section1231_basis_loss = 200_000;
    }
    page.basis.start =
        K1ScorpStart::Continuing { opening: 0, risk_history: ScorpRiskStart::Unknown, carried };
    page
}

#[test]
fn k1_1120s_import_without_section1231_item_keeps_carried_histories() {
    // AI Import selects the statement's variant and amounts only (see
    // k1_ai_import_selects_the_printed_boxes_only): applying a result that
    // finds no section 1231 item changes the selected fields and leaves the
    // unselected carried-loss histories as typed.
    let mut page = k1_1120s_carried_section1231_loss();
    let typed = K1ScorpLossHistory {
        history: vec![loss_year(2024, 2_000, 2_000, 2_000, 2_000)],
        ..Default::default()
    };
    page.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        current_loss: 0,
        property_qbi: K1ScorpPropertyQbi { gain: 10_000, qualified_1250: None, current: 0 },
    };
    page.qbi_records.property = typed.clone();
    // The import result: income, wages, property, service,
    // current_loss and property_qbi (zero amounts, no section 1231 item)
    // merge into the same Statement variant.
    if let K1ScorpQbi::Statement { income, property_qbi, current_loss, .. } = &mut page.qbi {
        *income = 0;
        *current_loss = 0;
        *property_qbi = no_property_qbi();
    }
    let (session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    assert_eq!(page.qbi_records.property, typed);
    assert_eq!(stored_k1_1120s(&session).property_qbi.losses.history, typed.history);
}

#[test]
fn k1_1120s_import_no_statement_preserves_personal_records_after_save_and_reopen() {
    let mut page = k1_1120s_carried_section1231_loss();
    page.qbi_records.property.history = vec![loss_year(2024, 2_000, 1_000, 2_000, 1_000)];
    page.qbi_records.property_complete = false;
    let retained = page.qbi_records.clone();
    // Import changes the printed union without entering the separate personal records.
    page.qbi = K1ScorpQbi::NoStatement;
    let (mut session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    session.draft.screening.as_mut().unwrap().section1231 =
        Some(crate::Section1231History { regular: vec![], amt: vec![] });
    let result = supported(&session.draft);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert!(noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    let record = stored_k1_1120s(&session);
    assert!(record.property_tracking());
    assert!(!record.loss_tracking());
    let mut malformed = record.clone();
    malformed.property_qbi.losses.history[0].remaining = -1;
    assert!(!malformed.valid(), "retained records still enforce their structural validity");
    let reopened = reopened_k1s(&mut session);
    assert_eq!(reopened, vec![IncomeDocument::K1Scorp(page)]);
    let IncomeDocument::K1Scorp(page) = &reopened[0] else { unreachable!() };
    assert_eq!(page.qbi_records, retained);
}

#[test]
fn k1_1120s_refusals_are_stored_and_named() {
    // Something else applies and each other box with an amount are stored,
    // refused by name on the review, with no amount on the return.
    let mut base = k1_1120s("Synthetic Corp");
    base.ordinary = 1_000_000;
    base.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 1_000_000,
        wages: 0,
        property: 0,
        service: false,
        current_loss: 0,
        property_qbi: no_property_qbi(),
    };
    base.participation = Participation::Over500Hours { hours: 600 };
    let (session, reply) = save_k1_1120s(vec![base.clone()]);
    accepted(&reply);
    supported(&session.draft);
    assert_eq!(
        crate::interview::document_amount(
            &FederalIncome::Scorp { data: stored_k1_1120s(&session) },
            false
        ),
        Some(1_000_000)
    );
    let refused = |page: K1Scorp, expected: &str| {
        let (session, reply) = save_k1_1120s(vec![page]);
        accepted(&reply);
        let r = estimate(&session.draft);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        assert!(
            r.issues.iter().any(|i| i.message.contains(expected)),
            "{expected}: {:?}",
            r.issues
        );
        assert_eq!(
            crate::interview::document_amount(
                &FederalIncome::Scorp { data: stored_k1_1120s(&session) },
                false
            ),
            None
        );
    };
    // Something else applies: the K-1 is left out with a notice.
    let mut page = base.clone();
    page.special = vec![crate::ScorpSpecial::Farming];
    let (session, reply) = save_k1_1120s(vec![page]);
    accepted(&reply);
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_SCORP_K1_LEFT_OUT"), "{r:?}");
    for item in [
        ScorpK1Item::RentalDisposition,
        ScorpK1Item::OtherIncome,
        ScorpK1Item::CreditRecapture,
        ScorpK1Item::AmtItems,
        ScorpK1Item::OtherInformation,
        ScorpK1Item::MultipleActivities,
    ] {
        let mut page = base.clone();
        {
            let K1ScorpOtherBoxes { unentered, .. } = k1_1120s_more_mut(&mut page);
            *unentered = vec![item];
        }
        refused(page, item.refusal());
    }
    // No section 199A statement with the K-1: it adds no QBI, with a note.
    let mut page = base;
    page.qbi = K1ScorpQbi::NoStatement;
    let (session, _) = save_k1_1120s(vec![page]);
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_K1_QBI_NO_STATEMENT"), "{r:?}");
    assert_eq!(value(&r, "qbi_deduction"), 0);
}
fn k1_1120s_fixture_session(page: K1Scorp) -> Interview {
    let draft = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, draft.clone());
    let request = save_topic(
        &draft,
        TaxSection::PassThrough,
        vec![document(0, IncomeDocument::K1Scorp(page))],
    );
    accepted(&send(&mut session, request));
    session
}

#[test]
fn k1_1120s_page_reproduces_the_shareholder_fixtures() {
    // T2-31: box 1 $20,000, box 4 $1,200, box 5a $2,000 (5b $1,500), box 12
    // code A $1,000, box 13 code O $240, box 16 code A $400, box 16 code D
    // $35,000 on shares held more than one year.
    //   Form 7203: $10,000 + $20,000 + $1,200 + $2,000 + $400 = $33,600
    //   before distributions; $35,000 leaves $0 and a $1,400 long-term
    //   gain; the $1,000 gift passes through the note's debt basis.
    //   AGI $60,000 + $1,200 + $2,000 + $1,400 + $20,000 = $84,600; less
    //   $16,100 standard deduction, $1,000 line 12f and QBI 20% x $20,000
    //   = $4,000 gives $63,500 taxable; tax $8,050 on $60,600 ordinary
    //   (Tax Table midpoint $60,625) plus 15% x $2,900 = $8,485; balance
    //   $8,485 - $6,500 - $240 = $1,745. The row shows Schedule E $20,000.
    let mut page = k1_1120s_fixture(20_000);
    page.interest = 120_000;
    page.dividends = 200_000;
    page.qualified_dividends = 150_000;
    page.distributions = 3_500_000;
    {
        let K1ScorpOtherBoxes { cash_charity, withholding, exempt_interest, stock_term, .. } =
            k1_1120s_more_mut(&mut page);
        *cash_charity = 100_000;
        *withholding = 24_000;
        *exempt_interest = 40_000;
        *stock_term = K1StockTerm::LongTerm;
    }
    let mut session = k1_1120s_fixture_session(page);
    let r = supported(&session.draft);
    assert_eq!(return_line(&r, "2a"), 40_000);
    assert_eq!(return_line(&r, "2b"), 120_000);
    assert_eq!(return_line(&r, "3a"), 150_000);
    assert_eq!(return_line(&r, "3b"), 200_000);
    assert_eq!(return_line(&r, "7a"), 140_000);
    assert_eq!(return_line(&r, "8"), 2_000_000);
    assert_eq!(return_line(&r, "11a / 11b"), 8_460_000);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "12f"), 100_000);
    assert_eq!(return_line(&r, "13b"), 400_000);
    assert_eq!(return_line(&r, "15"), 6_350_000);
    assert_eq!(return_line(&r, "16"), 848_500);
    assert_eq!(return_line(&r, "25c"), 24_000);
    assert_eq!(r.balance, Some(174_500));
    let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].preview, Some(2_000_000));
    // T2-29b: box 1 $20,000 and box 11 $5,000.
    //   Form 4562: line 6/8 $5,000, line 11 $60,000 + $20,000 = $80,000,
    //   line 12 $5,000, line 13 $0. Schedule E $20,000 - $5,000 = $15,000;
    //   AGI $75,000; QBI 20% x $15,000 = $3,000; taxable $75,000 - $16,100 -
    //   $3,000 = $55,900; tax $7,016 (midpoint $55,925); balance $516.
    //   The row is blank: the section 179 share depends on Form 4562's
    //   return-wide limits.
    let mut page = k1_1120s_fixture(20_000);
    {
        let K1ScorpOtherBoxes { section179, .. } = k1_1120s_more_mut(&mut page);
        *section179 = 500_000;
    }
    let mut session = k1_1120s_fixture_session(page);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
    assert_eq!(value(&r, "scorp_2_section179"), 5_000);
    assert_eq!(value(&r, "scorp_2_section179_carryforward"), 0);
    assert_eq!(return_line(&r, "8"), 1_500_000);
    assert_eq!(return_line(&r, "11a / 11b"), 7_500_000);
    assert_eq!(return_line(&r, "13b"), 300_000);
    assert_eq!(return_line(&r, "15"), 5_590_000);
    assert_eq!(return_line(&r, "16"), 701_600);
    assert_eq!(r.balance, Some(51_600));
    let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    assert_eq!(rows[0].preview, None);
}

// T2-31r question C9: the same T2-31 profit fixture with the at-risk
// history left missing (the page's default). No loss enters Form 6198, so
// the totals are unchanged: AGI $84,600, taxable $63,500, tax $8,485,
// balance $1,745 and the row's Schedule E $20,000. Reopening shows the same
// page. A box 11 amount still needs the history.
#[test]
fn k1_1120s_page_without_at_risk_history_reproduces_the_profit_fixture() {
    let missing = |start: &mut K1ScorpStart| {
        if let K1ScorpStart::Continuing { risk_history, .. } = start {
            *risk_history = ScorpRiskStart::Unknown;
        }
    };
    let mut page = k1_1120s_fixture(20_000);
    missing(&mut page.basis.start);
    page.interest = 120_000;
    page.dividends = 200_000;
    page.qualified_dividends = 150_000;
    page.distributions = 3_500_000;
    {
        let K1ScorpOtherBoxes { cash_charity, withholding, exempt_interest, stock_term, .. } =
            k1_1120s_more_mut(&mut page);
        *cash_charity = 100_000;
        *withholding = 24_000;
        *exempt_interest = 40_000;
        *stock_term = K1StockTerm::LongTerm;
    }
    let mut session = k1_1120s_fixture_session(page.clone());
    assert_eq!(stored_k1_1120s(&session).risk.start, ScorpRiskStart::Unknown);
    let r = supported(&session.draft);
    assert_eq!(return_line(&r, "7a"), 140_000);
    assert_eq!(return_line(&r, "8"), 2_000_000);
    assert_eq!(return_line(&r, "11a / 11b"), 8_460_000);
    assert_eq!(return_line(&r, "15"), 6_350_000);
    assert_eq!(return_line(&r, "16"), 848_500);
    assert_eq!(r.balance, Some(174_500));
    let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    assert_eq!(rows[0].preview, Some(2_000_000));
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page)]);
    let mut page = k1_1120s_fixture(20_000);
    missing(&mut page.basis.start);
    {
        let K1ScorpOtherBoxes { section179, .. } = k1_1120s_more_mut(&mut page);
        *section179 = 500_000;
    }
    let session = k1_1120s_fixture_session(page);
    unsupported(&session.draft, "S-corporation");
}

#[test]
fn k1_1065_import_no_statement_preserves_personal_records_after_save_and_reopen() {
    let mut page = k1_1065("Synthetic Partners");
    page.se_gross = SeGross::Nonfarm { amount: 100_000 };
    page.basis.start = BasisStart::Continuing {
        opening: 100_000,
        risk_history: ScorpRiskStart::Detailed { capacity: 100_000, profit: 0, loss: 0 },
        carried: k1_1065_carried_zero(),
    };
    if let BasisStart::Continuing { carried: PartnerCarried { basis_loss, .. }, .. } =
        &mut page.basis.start
    {
        *basis_loss = 30_000;
    }
    page.qbi_records = PartnerQbiRecords {
        qualified_se_gross: 50_000,
        history: Some(PartnerQbiHistory {
            basis: vec![loss_year(2024, 300, 150, 300, 150)],
            risk: vec![],
            passive: vec![],
        }),
    };
    let retained = page.qbi_records.clone();
    page.qbi = PartnerQbi::NoStatement;
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    // This K-1 has no current loss; its retained qualified loss is counted separately.
    assert!(noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    let reopened = reopened_k1s(&mut session);
    assert_eq!(reopened, vec![IncomeDocument::K1Partnership(page)]);
    let IncomeDocument::K1Partnership(page) = &reopened[0] else { unreachable!() };
    assert_eq!(page.qbi_records, retained);
}
fn stored_1065s(session: &Interview) -> Vec<PartnerK1> {
    session
        .draft
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Partnership { data } => Some(data.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn k1_1065_page_maps_every_branch_onto_the_record_and_reopens_unchanged() {
    let mut full = k1_1065("Synthetic Partners LP");
    full.interest = PartnerInterest::LimitedPartner {
        participation: LimitedParticipation::Over500Hours { hours: 600 },
    };
    full.liabilities_start = 5_000_000;
    full.liabilities_end = 6_000_000;
    full.ordinary = 1_500_000;
    full.services = 200_000;
    full.capital = 100_000;
    full.se = 200_000;
    full.se_gross = SeGross::Nonfarm { amount: 1_700_000 };
    full.distributions = 300_000;
    full.qbi = PartnerQbi::Statement {
        section1231_qbi: 0,
        ptp_income: 0,
        reit_dividends: 0,
        income: 1_500_000,
        wages: 400_000,
        property: 900_000,
        service: true,
    };
    full.qbi_records = PartnerQbiRecords {
        qualified_se_gross: 0,
        history: Some(PartnerQbiHistory { basis: vec![], risk: vec![], passive: vec![] }),
    };
    full.more = K1PartnershipOtherBoxes {
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
        interest: 10_000,
        dividends: 20_000,
        qualified_dividends: 15_000,
        royalties: 0,
        short: -5_000,
        long: 30_000,
        collectibles: 1_000,
        unrecaptured_partnership: 2_000,
        unrecaptured_fund: 3_000,
        section179: 50_000,
        section179_carryover: 20_000,
        cash_charity: 7_000,
        medical_insurance: 150_000,
        pensions: 9_000,
        backup_withholding: 1_000,
        exempt_interest: 4_000,
        exempt_other: 6_000,
        nondeductible: 8_000,
        fuel_credit: None,
        prior_fuel_credit: Some(PartnerPriorFuelCredit {
            claimed_credit: 12_000,
            filed_in_return_year: true,
            cash_method: true,
            full_prior_tax_benefit: true,
            partial_prior_year_benefit: None,
            prior_niit_recovery_excluded: true,
            not_reported_elsewhere: true,
        }),
        trading: true,
        credits: Vec::new(),
        unentered: vec![PartnerK1Item::RentalDisposition, PartnerK1Item::OtherIncome],
        investment_property: Vec::new(),
        investment_confirmed: None,
        housing_confirmed: None,
    };
    full.basis = PartnerBasis {
        start: BasisStart::Continuing {
            opening: 2_000_000,
            risk_history: ScorpRiskStart::Detailed { capacity: 2_000_000, profit: 0, loss: 0 },
            carried: PartnerCarried {
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
                basis_loss: 10_000,
                basis_nondeductible: 2_000,
                risk_loss: 3_000,
                risk_nondeductible: 1_000,
                deducted: 50_000,
                recaptured: 5_000,
                passive_loss: 4_000,
                se_basis: 1_000,
                se_risk: 500,
                se_passive: 0,
            },
        },
        contributions: 100_000,
        debt_start: 2_000_000,
        debt_end: 3_000_000,
        more_at_risk: PartnerAtRisk {
            excluded_contributions: 10_000,
            excluded_distributions: 20_000,
            protected: 30_000,
            released: 40_000,
        },
    };
    full.income_source =
        K1IncomeSource::Mixed { value: WorkValue::Amount { compensation: 900_000 } };
    let mut amt = full.record().regular;
    amt.ordinary = 1_400_000;
    full.amt = PartnerAmtRecords::Separate { record: amt.clone() };
    full.special = vec![crate::PartnerSpecial::RetiringPayments];

    let mut member = k1_1065("Synthetic Holdings LLC");
    member.publicly_traded = true;
    member.interest = PartnerInterest::LlcMember {
        box1_in_se: false,
        participation: Participation::Investor { prior_years: 3 },
    };
    member.ordinary = 800_000;
    member.se_gross = SeGross::Farming { amount: 50_000 };
    member.qbi = PartnerQbi::NoStatement;
    member.income_source = K1IncomeSource::Services { gross: 900_000 };

    let mut general = k1_1065("Synthetic Operating Partners");
    let mut facts = Participation::Investor { prior_years: 0 }.record();
    facts.minutes = 12_000;
    facts.regular = true;
    general.interest = PartnerInterest::General { participation: Participation::Other { facts } };
    general.ordinary = 300_000;
    general.se = 300_000;
    general.basis.start = k1_1065_continuing(100_000, ScorpRiskStart::Unknown);
    general.income_source = K1IncomeSource::Capital;

    let pages = vec![full.clone(), member.clone(), general.clone()];
    let (mut session, reply) =
        save_1065s(&form_8615_return(complete(FederalStatus::Single)), pages.clone());
    accepted(&reply);
    let stored = stored_1065s(&session);
    // The Limited partner branch: limited_for_se Yes and the Limited record.
    assert!(stored[0].limited_for_se);
    assert!(matches!(
        stored[0].participation,
        PartnerParticipation::Limited { minutes: 36_000, .. }
    ));
    assert_eq!(stored[0].trade_kind, ScorpTradeKind::FinancialTrading);
    assert_eq!((stored[0].regular.exempt, stored[0].regular.nondeductible), (6_000, 8_000));
    assert_eq!(stored[0].se_history, PartnerSeHistory { basis: 1_000, risk: 500, passive: 0 });
    // Something else applies writes every completeness answer No.
    assert_eq!(
        (stored[0].complete(), stored[0].regular.basis.complete, stored[0].regular.risk.complete),
        (false, false, false)
    );
    assert_eq!(stored[0].section179_carryover, 20_000);
    assert_eq!(stored[0].alternative, Some(amt));
    assert_eq!(stored[0].separate.as_ref().map(|s| s.qualified_dividends), Some(15_000));
    assert_eq!(
        stored[0].income_source,
        Some(BusinessIncomeSource::Mixed { compensation: 900_000 })
    );
    // An LLC member whose box 14 code A leaves out box 1 is limited for
    // self-employment (question C11); AMT same as regular; no statement.
    assert!(stored[1].limited_for_se);
    assert!(matches!(stored[1].participation, PartnerParticipation::General { .. }));
    assert_eq!(stored[1].alternative.as_ref(), Some(&stored[1].regular));
    assert_eq!(stored[1].qbi, None);
    assert_eq!(stored[1].separate, None);
    assert_eq!(stored[1].gross_personal_services, Some(900_000));
    assert_eq!(stored[1].regular.risk.start, ScorpRiskStart::New);
    assert!(!stored[2].limited_for_se);
    assert_eq!(stored[2].reported_se, Some(300_000));
    assert_eq!(stored[2].se_history, PartnerSeHistory { basis: 0, risk: 0, passive: 0 });
    assert_eq!(session.draft.income[1].label, "Synthetic Partners LP");
    assert_eq!(
        reopened_k1s(&mut session),
        pages.into_iter().map(IncomeDocument::K1Partnership).collect::<Vec<_>>()
    );
    // The partner's plan choice belongs to the self-employed benefits page
    // (T2-43p): saving the K-1 page keeps it.
    let sep = PartnerRetirement::Sep { rate: 2500 };
    for e in &mut session.draft.income {
        if let FederalIncome::Partnership { data } = &mut e.income {
            data.retirement = sep.clone();
        }
    }
    let before = session.draft.income.clone();
    let items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, items);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, before);
    assert!(stored_1065s(&session).iter().all(|p| p.retirement == sep));
    // An LLC member whose box 14 code A includes box 1 writes a general
    // partner's record, and reopens as one.
    let mut ticked = k1_1065("Synthetic LLC");
    ticked.interest = PartnerInterest::LlcMember {
        box1_in_se: true,
        participation: Participation::Investor { prior_years: 0 },
    };
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![ticked]);
    accepted(&reply);
    assert!(!stored_1065s(&session)[0].limited_for_se);
    assert_eq!(
        reopened_k1s(&mut session),
        vec![IncomeDocument::K1Partnership(k1_1065("Synthetic LLC"))]
    );
}

#[test]
fn k1_1065_rules_reject_on_both_sides_at_their_boundaries() {
    type Edit = Box<dyn Fn(&mut K1Partnership, i64)>;
    fn carried(page: &mut K1Partnership, field: usize, extra: i64) {
        let mut start = k1_1065_carried_zero();
        {
            let PartnerCarried {
                basis_loss,
                risk_loss,
                passive_loss,
                se_basis,
                se_risk,
                se_passive,
                ..
            } = &mut start;
            *basis_loss = 10_000;
            *risk_loss = 10_000;
            *passive_loss = 10_000;
            *[se_basis, se_risk, se_passive][field] = 10_000 + extra;
        }
        page.basis.start = BasisStart::Continuing {
            opening: 100_000,
            risk_history: ScorpRiskStart::Unknown,
            carried: start,
        };
    }
    let cases: Vec<(&str, Edit)> = vec![
        (
            "PARTNER_DEBT_ABOVE_LIABILITIES",
            Box::new(|p, extra| {
                p.liabilities_start = 50_000;
                p.basis.debt_start = 50_000 + extra;
            }),
        ),
        (
            "PARTNER_DEBT_ABOVE_LIABILITIES",
            Box::new(|p, extra| {
                p.liabilities_end = 50_000;
                p.basis.debt_end = 50_000 + extra;
            }),
        ),
        ("PARTNER_SE_CARRY_ABOVE_LOSS", Box::new(|p, extra| carried(p, 0, extra))),
        ("PARTNER_SE_CARRY_ABOVE_LOSS", Box::new(|p, extra| carried(p, 1, extra))),
        ("PARTNER_SE_CARRY_ABOVE_LOSS", Box::new(|p, extra| carried(p, 2, extra))),
        (
            "PARTNER_QUALIFIED_ABOVE_DIVIDENDS",
            Box::new(|p, extra| {
                p.more = k1_1065_more();
                {
                    let K1PartnershipOtherBoxes { dividends, qualified_dividends, .. } =
                        &mut p.more;
                    *dividends = 30_000;
                    *qualified_dividends = 30_000 + extra;
                }
            }),
        ),
    ];
    for (key, edit) in cases {
        let mut page = k1_1065("Synthetic Partners");
        edit(&mut page, 0);
        accepted(&save_1065s(&complete(FederalStatus::Single), vec![page]).1);
        let mut page = k1_1065("Synthetic Partners");
        edit(&mut page, 1);
        rejected_with(save_1065s(&complete(FederalStatus::Single), vec![page]).1, key);
    }
    // Each other box once; a partner before 2026 has no New at-risk start
    // (server only).
    let mut page = k1_1065("Synthetic Partners");
    page.more = k1_1065_more();
    {
        let K1PartnershipOtherBoxes { unentered, .. } = &mut page.more;
        *unentered = vec![PartnerK1Item::Credits, PartnerK1Item::AmtItems];
    }
    accepted(&save_1065s(&complete(FederalStatus::Single), vec![page.clone()]).1);
    {
        let K1PartnershipOtherBoxes { unentered, .. } = &mut page.more;
        *unentered = vec![PartnerK1Item::Credits, PartnerK1Item::Credits];
    }
    rejected_with(save_1065s(&complete(FederalStatus::Single), vec![page]).1, "PARTNER_BOX_TWICE");
    let mut page = k1_1065("Synthetic Partners");
    page.basis.start = k1_1065_continuing(100_000, ScorpRiskStart::New);
    rejected_with(
        save_1065s(&complete(FederalStatus::Single), vec![page]).1,
        "PARTNER_RISK_HISTORY_NEW",
    );
}

#[test]
fn k1_1065_refusals_are_stored_and_named() {
    let refused = |page: K1Partnership, section: &str, expected: &str| {
        let (session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page]);
        accepted(&reply);
        let r = estimate(&session.draft);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        assert!(
            r.issues.iter().any(|i| i.section == section && i.message.contains(expected)),
            "{expected}: {:?}",
            r.issues
        );
        let data = stored_1065s(&session).remove(0);
        assert_eq!(
            crate::interview::document_amount(&FederalIncome::Partnership { data }, false),
            None
        );
    };
    let profit = || {
        let mut page = k1_1065("Synthetic Partners");
        page.ordinary = 1_000_000;
        page.qbi = PartnerQbi::Statement {
            section1231_qbi: 0,
            ptp_income: 0,
            reit_dividends: 0,
            income: 1_000_000,
            wages: 0,
            property: 0,
            service: false,
        };
        page.qbi_records = PartnerQbiRecords { qualified_se_gross: 0, history: None };
        page.interest = PartnerInterest::LlcMember {
            box1_in_se: false,
            participation: Participation::Over500Hours { hours: 600 },
        };
        page
    };
    // The profitable page itself is supported, with its row amount.
    let (session, _) = save_1065s(&complete(FederalStatus::Single), vec![profit()]);
    supported(&session.draft);
    assert_eq!(
        crate::interview::document_amount(
            &FederalIncome::Partnership { data: stored_1065s(&session).remove(0) },
            false
        ),
        Some(1_000_000)
    );
    // Something else applies: the K-1 is left out with a notice.
    let mut page = profit();
    page.special = vec![crate::PartnerSpecial::PropertyContributed];
    let (session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page]);
    accepted(&reply);
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_PARTNERSHIP_K1_LEFT_OUT"), "{r:?}");
    // No section 199A statement with the K-1: it adds no QBI, with a note.
    let mut page = profit();
    page.qbi = PartnerQbi::NoStatement;
    let (session, _) = save_1065s(&complete(FederalStatus::Single), vec![page]);
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_K1_QBI_NO_STATEMENT"), "{r:?}");
    for item in [
        PartnerK1Item::RentalDisposition,
        PartnerK1Item::OtherIncome,
        PartnerK1Item::OtherDeductions,
        PartnerK1Item::CreditRecapture,
        PartnerK1Item::AmtItems,
        PartnerK1Item::PropertyDistribution,
        PartnerK1Item::OtherInformation,
        PartnerK1Item::MultipleActivities,
    ] {
        let mut page = profit();
        page.more = k1_1065_more();
        {
            let K1PartnershipOtherBoxes { unentered, .. } = &mut page.more;
            *unentered = vec![item];
        }
        refused(page, "Income: Synthetic Partners", item.refusal());
    }
    // A loss with no 2025 at-risk history (question C9).
    let mut page = profit();
    page.ordinary = -100_000;
    page.qbi = PartnerQbi::Statement {
        section1231_qbi: 0,
        ptp_income: 0,
        reit_dividends: 0,
        income: -100_000,
        wages: 0,
        property: 0,
        service: false,
    };
    page.qbi_records = PartnerQbiRecords { qualified_se_gross: 0, history: None };
    page.basis.start = k1_1065_continuing(1_000_000, ScorpRiskStart::Unknown);
    refused(page, "Partnership activity", "2025 Form 6198 figures");
    // Refusals open the Schedule K-1 topic.
    let mut page = profit();
    page.more = k1_1065_more();
    {
        let K1PartnershipOtherBoxes { unentered, .. } = &mut page.more;
        *unentered = vec![PartnerK1Item::RentalDisposition];
    }
    let (session, _) = save_1065s(&complete(FederalStatus::Single), vec![page]);
    let r = estimate(&session.draft);
    for issue in r
        .issues
        .iter()
        .filter(|i| i.section.starts_with("Partnership") || i.section.starts_with("Income: "))
    {
        assert_eq!(
            crate::interview::issue_page(
                &session.draft,
                &issue.context,
                Some({
                    let m = &issue.display;
                    m.key.as_str()
                })
            ),
            Some(TaxSection::PassThrough),
            "{issue:?}"
        );
    }
    assert!(r.issues.iter().any(|i| i.section.starts_with("Income: ")));
    // A return that reads the source (T2-cy: Form 8615 applies) answers
    // what produces the income: Not needed is refused on save by name.
    let young = form_8615_return(complete(FederalStatus::Single));
    let Reply::Response(P::Problem { message, .. }) = ({
        let mut source = Interview::new(TEST_USER, young.clone());
        current_document_refusal(
            "income-source-2937",
            &mut source,
            vec![document(0, IncomeDocument::K1Partnership(profit()))],
            "SOURCE_PARTNER_NEEDED",
            "[documents/documents/0/document/income_source]",
        )
    }) else {
        panic!("expected the young-filer refusal")
    };
    assert!(status_text(&message).contains("Standard Deduction Worksheet"), "{message:?}");
    let mut page = profit();
    page.income_source = K1IncomeSource::Capital;
    accepted(&save_1065s(&young, vec![page]).1);
}

#[test]
fn k1_1065_page_reproduces_the_t2_29_and_t2_29b_fixtures() {
    // The T2-29 hand fixture (tests/passthrough.rs
    // partner_separately_stated_items_reach_the_return_by_hand) entered
    // through the Schedule K-1 page: single, W-2 $60,000 ($6,500 withheld);
    // an LLC member (box 14 code A leaves out box 1, so limited for
    // self-employment) working over 500 hours; box 1 $20,000, box 14A $0;
    // QBI statement $20,000; box 5 $1,200, 6a $2,000, 6b $1,500, 8 ($500),
    // 9a $3,000, 13A $1,000, 15O $240, 18A $400; box 19A $5,000; opening
    // basis and 2025 Form 6198 capacity $10,000.
    //   Basis: $10,000 + $26,600 - $5,000 = $31,600; Part II $500 short loss
    //   and $1,000 gift fit. Lines 2a $400, 2b $1,200, 3a $1,500, 3b $2,000;
    //   Schedule D ($500) + $3,000 = 7a $2,500; Schedule 1 line 5 $20,000;
    //   AGI $60,000 + $1,200 + $2,000 + $2,500 + $20,000 = $85,700; 12e
    //   $16,100; 12f $1,000; QBI 20% x $20,000 = $4,000; taxable $64,600;
    //   tax $8,050 on $60,600 ordinary + 15% x $4,000 = $8,650; payments
    //   $6,500 + $240; balance due $1,910. The row's amount is Schedule E
    //   box 1 after limits, $20,000.
    // Question C9: with no 2025 Form 6198 figures ("Missing history") the
    // K-1 has no loss beyond its income and no at-risk carryover, so the
    // same totals follow.
    let page = |risk_history: ScorpRiskStart| {
        let mut page = k1_1065("Synthetic Partners");
        page.interest = PartnerInterest::LlcMember {
            box1_in_se: false,
            participation: Participation::Over500Hours { hours: 501 },
        };
        page.ordinary = 2_000_000;
        page.qbi = PartnerQbi::Statement {
            section1231_qbi: 0,
            ptp_income: 0,
            reit_dividends: 0,
            income: 2_000_000,
            wages: 0,
            property: 0,
            service: false,
        };
        page.qbi_records = PartnerQbiRecords { qualified_se_gross: 0, history: None };
        page.distributions = 500_000;
        page.more = k1_1065_more();
        {
            let K1PartnershipOtherBoxes {
                interest,
                dividends,
                qualified_dividends,
                short,
                long,
                cash_charity,
                backup_withholding,
                exempt_interest,
                ..
            } = &mut page.more;
            *interest = 120_000;
            *dividends = 200_000;
            *qualified_dividends = 150_000;
            *short = -50_000;
            *long = 300_000;
            *cash_charity = 100_000;
            *backup_withholding = 24_000;
            *exempt_interest = 40_000;
        }
        page.basis.start = k1_1065_continuing(1_000_000, risk_history);
        page
    };
    for history in [
        ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 },
        ScorpRiskStart::Unknown,
    ] {
        let (mut session, reply) =
            save_1065s(&complete(FederalStatus::Single), vec![page(history)]);
        accepted(&reply);
        let result = supported(&session.draft);
        for (line, amount) in [
            ("2a", 40_000),
            ("2b", 120_000),
            ("3a", 150_000),
            ("3b", 200_000),
            ("7a", 250_000),
            ("8", 2_000_000),
            ("11a / 11b", 8_570_000),
            ("12e", 1_610_000),
            ("12f", 100_000),
            ("13b", 400_000),
            ("15", 6_460_000),
            ("16", 865_000),
            ("25c", 24_000),
        ] {
            assert_eq!(return_line(&result, line), amount, "line {line}");
        }
        assert_eq!(result.balance, Some(191_000));
        let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        assert_eq!(rows[0].preview, Some(2_000_000));
    }

    // The T2-29b hand fixture
    // (partner_section179_reaches_form_4562_schedule_e_basis_and_qbi_by_hand):
    // box 1 $20,000, box 12 $5,000, statement $20,000, opening basis and
    // at-risk $10,000. Form 4562: line 6/8/9 $5,000; line 11 $60,000 +
    // $20,000 = $80,000; line 12 $5,000; line 13 $0. Schedule E $20,000 -
    // $5,000 = $15,000; AGI $75,000; QBI 20% x $15,000 = $3,000; taxable
    // $75,000 - $16,100 - $3,000 = $55,900; tax (midpoint $55,925) $1,240 +
    // $4,560 + 22% x $5,525 = $7,015.50, rounded $7,016; balance $516. The
    // row shows no amount: Form 4562 decides the section 179 part.
    let section179 = |risk_history: ScorpRiskStart| {
        let mut page = k1_1065("Synthetic Partners");
        page.interest = PartnerInterest::LlcMember {
            box1_in_se: false,
            participation: Participation::Over500Hours { hours: 501 },
        };
        page.ordinary = 2_000_000;
        page.qbi = PartnerQbi::Statement {
            section1231_qbi: 0,
            ptp_income: 0,
            reit_dividends: 0,
            income: 2_000_000,
            wages: 0,
            property: 0,
            service: false,
        };
        page.qbi_records = PartnerQbiRecords { qualified_se_gross: 0, history: None };
        page.more = k1_1065_more();
        {
            let K1PartnershipOtherBoxes { section179, .. } = &mut page.more;
            *section179 = 500_000;
        }
        page.basis.start = k1_1065_continuing(1_000_000, risk_history);
        page
    };
    let (mut session, reply) = save_1065s(
        &complete(FederalStatus::Single),
        vec![section179(ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 })],
    );
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "section179_business_income_limit"), 80_000);
    assert_eq!(value(&result, "section179_deduction"), 5_000);
    assert_eq!(value(&result, "partnership_2_section179"), 5_000);
    assert_eq!(value(&result, "partnership_2_section179_carryforward"), 0);
    assert_eq!(return_line(&result, "8"), 1_500_000);
    assert_eq!(return_line(&result, "11a / 11b"), 7_500_000);
    assert_eq!(return_line(&result, "13b"), 300_000);
    assert_eq!(return_line(&result, "15"), 5_590_000);
    assert_eq!(return_line(&result, "16"), 701_600);
    assert_eq!(result.balance, Some(51_600));
    let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    assert_eq!(rows[0].preview, None);
    // A section 179 amount needs the at-risk history (question C9).
    let (session, _) =
        save_1065s(&complete(FederalStatus::Single), vec![section179(ScorpRiskStart::Unknown)]);
    unsupported(&session.draft, "Partnership activity");
}

#[test]
fn k1_1120s_carried_losses_fold_round_trips_none_and_some() {
    // None: a continuing shareholder with only the opening basis writes no
    // carryover anywhere and reopens with the fold closed.
    let mut none = k1_1120s("Synthetic Corp N");
    none.ordinary = 1_000_000;
    none.basis.start = k1_1120s_continuing(500_000, 500_000);
    // Some: an ordinary basis loss of $500, an at-risk loss of $100 and a
    // passive loss of $200 carried from 2025.
    let mut some = k1_1120s("Synthetic Corp S");
    some.ordinary = 1_000_000;
    some.basis.start = K1ScorpStart::Continuing {
        opening: 500_000,
        risk_history: ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 },
        carried: k1_1120s_carried_zero(),
    };
    if let K1ScorpStart::Continuing {
        carried: K1ScorpCarried { basis_loss, risk_loss, passive_loss, .. },
        ..
    } = &mut some.basis.start
    {
        *basis_loss = 50_000;
        *risk_loss = 10_000;
        *passive_loss = 20_000;
    }
    // Without a section 199A statement no QBI loss tracking is needed.
    some.qbi = K1ScorpQbi::NoStatement;
    let pages = vec![none.clone(), some.clone()];
    let (mut session, reply) = save_k1_1120s(pages.clone());
    accepted(&reply);
    let records: Vec<ScorpK1> = session
        .draft
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Scorp { data } => Some(data.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(records[0].opening, 500_000);
    assert_eq!(records[0].prior_loss, 0);
    assert_eq!(records[0].risk.prior_loss, 0);
    assert_eq!(records[0].passive.loss, 0);
    assert_eq!(records[0].capital, None);
    assert_eq!(records[0].section1231, None);
    assert_eq!(
        records[0].risk.start,
        ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 }
    );
    assert_eq!(records[1].opening, 500_000);
    assert_eq!(records[1].prior_loss, 50_000);
    assert_eq!(records[1].risk.prior_loss, 10_000);
    assert_eq!(records[1].passive.loss, 20_000);
    assert_eq!(
        reopened_k1s(&mut session),
        pages.into_iter().map(IncomeDocument::K1Scorp).collect::<Vec<_>>()
    );
    // The Some branch with every amount zero writes the None record, so it
    // reopens as None.
    let mut zero = none.clone();
    zero.basis.start = K1ScorpStart::Continuing {
        opening: 500_000,
        risk_history: ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 },
        carried: k1_1120s_carried_zero(),
    };
    let (mut session, reply) = save_k1_1120s(vec![zero]);
    accepted(&reply);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(none)]);
    // The election alone (no amount) is carried data: it reopens as Some.
    let mut election = some.clone();
    election.basis.start = K1ScorpStart::Continuing {
        opening: 500_000,
        risk_history: ScorpRiskStart::Unknown,
        carried: k1_1120s_carried_zero(),
    };
    if let K1ScorpStart::Continuing { carried: K1ScorpCarried { losses_first, .. }, .. } =
        &mut election.basis.start
    {
        *losses_first = true;
    }
    let (mut session, reply) = save_k1_1120s(vec![election.clone()]);
    accepted(&reply);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(election)]);
}

#[test]
fn k1_1065_carried_losses_fold_round_trips_none_and_some() {
    let draft = complete(FederalStatus::Single);
    // None: only the opening basis; no SE history is written.
    let mut none = k1_1065("Synthetic Partners N");
    none.ordinary = 300_000;
    none.basis.start = k1_1065_continuing(100_000, ScorpRiskStart::Unknown);
    // Some: a $300 basis loss carried from 2025, $100 of it self-employment
    // earnings (se_basis 10_000 <= basis_loss 30_000).
    let mut some = k1_1065("Synthetic Partners S");
    some.ordinary = 300_000;
    some.basis.start = BasisStart::Continuing {
        opening: 100_000,
        risk_history: ScorpRiskStart::Unknown,
        carried: k1_1065_carried_zero(),
    };
    if let BasisStart::Continuing {
        carried:
            PartnerCarried {
                section1231_basis_loss: 0,
                section1231_risk_loss: 0,
                section1231_passive_loss: 0,
                basis_loss,
                se_basis,
                ..
            },
        ..
    } = &mut some.basis.start
    {
        *basis_loss = 30_000;
        *se_basis = 10_000;
    }
    let pages = vec![none.clone(), some.clone()];
    let (mut session, reply) = save_1065s(&draft, pages.clone());
    accepted(&reply);
    let stored = stored_1065s(&session);
    assert_eq!(stored[0].regular.basis.opening, 100_000);
    assert_eq!(stored[0].regular.basis.prior_loss, 0);
    assert_eq!(stored[0].se_history, PartnerSeHistory { basis: 0, risk: 0, passive: 0 });
    assert_eq!(stored[1].regular.basis.prior_loss, 30_000);
    assert_eq!(stored[1].se_history.basis, 10_000);
    assert_eq!(
        reopened_k1s(&mut session),
        pages.into_iter().map(IncomeDocument::K1Partnership).collect::<Vec<_>>()
    );
    // The Some branch with every amount zero reopens as None.
    let mut zero = none.clone();
    zero.basis.start = BasisStart::Continuing {
        opening: 100_000,
        risk_history: ScorpRiskStart::Unknown,
        carried: k1_1065_carried_zero(),
    };
    let (mut session, reply) = save_1065s(&draft, vec![zero]);
    accepted(&reply);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Partnership(none)]);
}

// Unit B8: the basis and at-risk carries of the loss classes write the stored
// record, reopen as the same page, and give the same ledger after a second
// save of the reopened page.
#[test]
fn k1_1065_loss_class_carries_round_trip_and_the_ledger_survives_reopen() {
    let draft = complete(FederalStatus::Single);
    let mut page = k1_1065("Synthetic Partners C");
    page.ordinary = 1_000_000;
    page.basis.start = BasisStart::Continuing {
        opening: 10_000_000,
        risk_history: ScorpRiskStart::Detailed { capacity: 10_000_000, profit: 0, loss: 0 },
        carried: k1_1065_carried_zero(),
    };
    if let BasisStart::Continuing { carried, .. } = &mut page.basis.start {
        carried.basis_short = 11_100;
        carried.basis_long = 22_200;
        carried.risk_short = 33_300;
        carried.risk_long = 44_400;
        carried.rental_real_estate_basis_loss = 55_500;
        carried.rental_real_estate_risk_loss = 66_600;
        carried.other_rental_basis_loss = 77_700;
        carried.other_rental_risk_loss = 88_800;
        carried.basis_section179 = 99_900;
        carried.basis_charity = 12_300;
        carried.basis_foreign_taxes = 45_600;
    }
    let (mut session, reply) = save_1065s(&draft, vec![page.clone()]);
    accepted(&reply);
    let stored = stored_1065s(&session);
    let regular = &stored[0].regular;
    let carry = |basis, risk| crate::K1LossCarry { basis, risk };
    assert_eq!(regular.capital_carried.short, carry(11_100, 33_300));
    assert_eq!(regular.capital_carried.long, carry(22_200, 44_400));
    assert_eq!(regular.carried.real_estate, carry(55_500, 66_600));
    assert_eq!(regular.carried.other_rental, carry(77_700, 88_800));
    assert_eq!(regular.carried.section179, 99_900);
    assert_eq!(regular.carried.charity, 12_300);
    assert_eq!(regular.carried.foreign_taxes, 45_600);
    // The AMT record "same as regular" carries the same amounts.
    assert_eq!(stored[0].alternative.as_ref().unwrap(), regular);
    let ledger = stored[0].ledger(&stored[0].regular).unwrap();
    assert!(!ledger.limited());
    // Reopen, save the reopened page again: the same record, the same ledger.
    let reopened = reopened_k1s(&mut session);
    assert_eq!(reopened, vec![IncomeDocument::K1Partnership(page.clone())]);
    let (second, reply) = save_1065s(&draft, vec![page.clone()]);
    accepted(&reply);
    let again = stored_1065s(&second);
    assert_eq!(again[0].regular, stored[0].regular);
    assert_eq!(again[0].ledger(&again[0].regular).unwrap(), ledger);
    // A new partner (from 2026) writes every carry zero.
    let mut fresh = page.clone();
    fresh.basis.start = BasisStart::New;
    let (session, reply) = save_1065s(&draft, vec![fresh]);
    accepted(&reply);
    let fresh = stored_1065s(&session);
    assert_eq!(fresh[0].regular.carried, crate::K1LossCarryforwards::default());
    assert_eq!(fresh[0].regular.capital_carried, crate::PartnerCapitalCarry::default());
}

#[test]
fn k1_1120s_loss_class_carries_round_trip_and_the_ledger_survives_reopen() {
    let mut page = k1_1120s("Synthetic Corp C");
    page.ordinary = 1_000_000;
    page.qbi = K1ScorpQbi::NoStatement;
    page.basis.start = k1_1120s_continuing(10_000_000, 10_000_000);
    if let K1ScorpStart::Continuing { carried, .. } = &mut page.basis.start {
        carried.rental_real_estate_basis_loss = 55_500;
        carried.rental_real_estate_risk_loss = 66_600;
        carried.other_rental_basis_loss = 77_700;
        carried.other_rental_risk_loss = 88_800;
        carried.basis_section179 = 99_900;
        carried.basis_charity = 12_300;
        carried.basis_foreign_taxes = 45_600;
    }
    let (mut session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    let stored = |session: &Interview| -> ScorpK1 {
        session
            .draft
            .income
            .iter()
            .find_map(|e| match &e.income {
                FederalIncome::Scorp { data } => Some(data.clone()),
                _ => None,
            })
            .unwrap()
    };
    let record = stored(&session);
    let carry = |basis, risk| crate::K1LossCarry { basis, risk };
    assert_eq!(record.carried.real_estate, carry(55_500, 66_600));
    assert_eq!(record.carried.other_rental, carry(77_700, 88_800));
    assert_eq!(record.carried.section179, 99_900);
    assert_eq!(record.carried.charity, 12_300);
    assert_eq!(record.carried.foreign_taxes, 45_600);
    // The AMT record "same as regular" carries the same amounts.
    assert_eq!(record.amt.as_ref().unwrap().carried, record.carried);
    let ledger = record.ledger().unwrap();
    assert!(!ledger.limited());
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page.clone())]);
    let (second, reply) = save_k1_1120s(vec![page]);
    accepted(&reply);
    assert_eq!(stored(&second).carried, record.carried);
    assert_eq!(stored(&second).ledger().unwrap(), ledger);
}

#[test]
fn partner_benefits_rows_map_onto_the_record_and_reopen_unchanged() {
    // Box 13 code M $6,000 inside box 4a $6,000, box 13 code R $3,000,
    // box 19 code A $5,000; and an investor's K-1 with no printed benefit.
    let mut s = partner_benefits_session(vec![
        k1_1065_benefits("Synthetic Partners", 600_000, 300_000, 500_000),
        k1_1065_benefits("Synthetic Fund LP", 0, 0, 0),
    ]);
    let stored = stored_1065s(&s);
    assert_eq!((stored[0].health_guaranteed, stored[0].pensions), (600_000, 300_000));
    assert_eq!(stored[0].retirement, PartnerRetirement::None);
    // A K-1 with box 13 code M or R makes the topic necessary (question Q7),
    // with no Schedule C business (question Q3: a row for every K-1).
    let hub = open_adjustments_hub(&mut s);
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::SelfEmployedBenefits);
    // Box 13 code R without its plan is the row's attention item.
    let rows = hub_rows(&hub);
    assert_eq!(rows[0].2, TaxSection::SelfEmployedBenefits, "{rows:?}");
    assert!(
        rows[0].1.starts_with("Needs attention") && rows[0].1.contains("box 13 code R"),
        "{rows:?}"
    );
    let value = open_benefits(&mut s);
    assert!(value.businesses.is_empty());
    assert_eq!(value.partnerships.len(), 2);
    let row = &value.partnerships[0];
    assert_eq!(status_text(&row.name), "Synthetic Partners");
    assert_eq!(
        (row.medical_insurance, row.pensions, row.cash_distributions),
        (600_000, 300_000, 500_000)
    );
    assert_eq!((&row.health, row.from_distributions), (&SeHealth::None, 0));
    assert_eq!(row.retirement, PartnerPlan::None);
    // All year $8,000 including $2,000 charged against distributions,
    // something else applies, a 20% partnership SEP; some months on the
    // investor's partnership-named policy.
    let mut sent = value.clone();
    sent.partnerships[0].health = se_health_test_year(800_000, true);
    sent.partnerships[0].from_distributions = 200_000;
    se_health_test_unreimbursed(&mut sent.partnerships[0].health);
    sent.partnerships[0].retirement = PartnerPlan::Sep { rate: 2000 };
    sent.partnerships[1].health = se_health_test_months(
        vec![
            BusinessHealthMonth {
                child_increase: 0,
                employer_eligible: false,
                month: 3,
                premiums: 40_000,
                reimbursed: None,
            },
            BusinessHealthMonth {
                child_increase: 0,
                employer_eligible: false,
                month: 4,
                premiums: 45_000,
                reimbursed: None,
            },
        ],
        true,
    );
    accepted(&save_benefits(&mut s, sent.clone()));
    let stored = stored_1065s(&s);
    let BusinessHealthInsurance::Plan { months, long_term_care, establishment } =
        &stored[0].health_insurance
    else {
        panic!("expected a plan")
    };
    assert!(!establishment.included());
    assert!(long_term_care.is_empty());
    assert_eq!(months.len(), 12);
    assert_eq!(months.iter().map(|m| m.premiums).sum::<i64>(), 800_000);
    assert!(months.iter().all(|m| !m.employer_eligible && m.child_increase == 0));
    assert_eq!(stored[0].health_distributions, 200_000);
    assert_eq!(stored[0].retirement, PartnerRetirement::Sep { rate: 2000 });
    // The K-1 boxes stay the K-1 topic's.
    assert_eq!((stored[0].health_guaranteed, stored[0].pensions), (600_000, 300_000));
    let BusinessHealthInsurance::Plan { months, establishment, .. } = &stored[1].health_insurance
    else {
        panic!("expected a plan")
    };
    assert!(establishment.included());
    assert_eq!(
        months.iter().map(|m| (m.month, m.premiums)).collect::<Vec<_>>(),
        vec![(3, 40_000), (4, 45_000)]
    );
    // Reopening gives the page that was saved; so does a restart.
    assert_eq!(open_benefits(&mut s), sent);
    let mut restarted = restart(s.draft.clone());
    assert_eq!(open_benefits(&mut restarted), sent);
    // Saving the Schedule K-1 topic unchanged keeps the page's answers, and
    // the K-1 reopens unchanged with its boxes 13 code M and R.
    let before = s.draft.income.clone();
    let items = page_documents(&open_topic(&mut s, TaxSection::PassThrough)).to_vec();
    assert_eq!(
        items[0].document,
        IncomeDocument::K1Partnership(k1_1065_benefits(
            "Synthetic Partners",
            600_000,
            300_000,
            500_000
        ))
    );
    let request = save_topic(&s.draft.clone(), TaxSection::PassThrough, items);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income, before);
    // An investor-only K-1 offers the page without suggesting it.
    let mut s = partner_benefits_session(vec![k1_1065_benefits("Synthetic Fund LP", 0, 0, 0)]);
    let hub = open_adjustments_hub(&mut s);
    let P::Hub { suggested, allowed_sections, .. } = &hub else { unreachable!() };
    assert_ne!(*suggested, TaxSection::SelfEmployedBenefits);
    assert!(allowed_sections.contains(&(TaxSection::SelfEmployedBenefits as i32)));
    assert_eq!(open_benefits(&mut s).partnerships.len(), 1);
}

#[test]
fn partner_benefits_rules_reject_on_both_sides_at_their_boundaries() {
    // The K-1 page: box 13 code M not above box 4a.
    let mut page = k1_1065_benefits("Synthetic Partners", 600_000, 0, 0);
    accepted(&save_1065s(&complete(FederalStatus::Single), vec![page.clone()]).1);
    page.services = 599_999;
    rejected_with(
        save_1065s(&complete(FederalStatus::Single), vec![page]).1,
        "PARTNER_MEDICAL_ABOVE_4A",
    );
    // The benefits row, against box 13 code M $6,000, box 13 code R $3,000
    // and box 19 code A $5,000 as stored (the row's copies are ignored).
    type Edit = Box<dyn Fn(&mut PartnerBenefits, i64)>;
    let all_year = |premiums: i64| se_health_test_year(premiums, true);
    let cases: Vec<(&str, Edit)> = vec![
        (
            "PARTNER_PREMIUMS_BELOW_K1",
            Box::new(move |row, less| {
                row.health = all_year(700_000 - less);
                row.from_distributions = 100_000;
            }),
        ),
        (
            "PARTNER_PREMIUMS_BELOW_K1",
            Box::new(|row, less| {
                // Server only: the rule language cannot sum the months.
                row.health = se_health_test_months(
                    vec![
                        BusinessHealthMonth {
                            child_increase: 0,
                            employer_eligible: false,
                            month: 1,
                            premiums: 400_000,
                            reimbursed: None,
                        },
                        BusinessHealthMonth {
                            child_increase: 0,
                            employer_eligible: false,
                            month: 2,
                            premiums: 300_000 - less,
                            reimbursed: None,
                        },
                    ],
                    true,
                );
                row.from_distributions = 100_000;
            }),
        ),
        (
            "PARTNER_DISTRIBUTION_PREMIUMS_ABOVE_19A",
            Box::new(move |row, more| {
                row.health = all_year(2_000_000);
                row.from_distributions = 500_000 + more;
            }),
        ),
        (
            "PARTNER_BOX_13R_NEEDS_PLAN",
            Box::new(move |row, none| {
                row.health = all_year(600_000);
                row.retirement =
                    if none == 1 { PartnerPlan::None } else { PartnerPlan::Sep { rate: 2500 } };
            }),
        ),
    ];
    for (key, edit) in cases {
        for (extra, ok) in [(0, true), (1, false)] {
            let mut s = partner_benefits_session(vec![k1_1065_benefits(
                "Synthetic Partners",
                600_000,
                300_000,
                500_000,
            )]);
            let mut value = open_benefits(&mut s);
            edit(&mut value.partnerships[0], extra);
            value.partnerships[0].medical_insurance = 0;
            value.partnerships[0].cash_distributions = 100_000_000;
            if key != "PARTNER_BOX_13R_NEEDS_PLAN" {
                value.partnerships[0].retirement = PartnerPlan::Other;
            }
            let reply = save_benefits(&mut s, value);
            if ok {
                accepted(&reply);
            } else {
                rejected_with(reply, key);
            }
        }
    }
    // None with box 13 code M: the premiums entered are zero.
    let mut s = partner_benefits_session(vec![k1_1065_benefits("Synthetic Partners", 1, 0, 0)]);
    let value = open_benefits(&mut s);
    rejected_with(save_benefits(&mut s, value), "PARTNER_PREMIUMS_BELOW_K1");
    // A plan without box 13 code R, either plan choice; None is accepted.
    for (plan, ok) in [
        (PartnerPlan::None, true),
        (PartnerPlan::Sep { rate: 2500 }, false),
        (PartnerPlan::Other, false),
        (PartnerPlan::DefinedBenefit { contribution: 100 }, false),
    ] {
        let mut s = partner_benefits_session(vec![k1_1065_benefits("Synthetic Partners", 0, 0, 0)]);
        let mut value = open_benefits(&mut s);
        value.partnerships[0].retirement = plan;
        value.partnerships[0].pensions = 100;
        let reply = save_benefits(&mut s, value);
        if ok {
            accepted(&reply);
        } else {
            rejected_with(reply, "PARTNER_PLAN_WITHOUT_BOX_13R");
        }
    }
    // Membership is read-only: a row dropped since the page was built.
    let mut s = partner_benefits_session(vec![
        k1_1065_benefits("Synthetic Partners", 0, 0, 0),
        k1_1065_benefits("Synthetic Fund LP", 0, 0, 0),
    ]);
    let mut value = open_benefits(&mut s);
    value.partnerships.pop();
    let request = benefits_request(&s.draft, value);
    assert!(
        refusal(current_source_refusal(
            "health-context-14",
            &mut s,
            request,
            "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN",
            None,
            SourceScope::Health
        ))
        .contains("businesses changed")
    );
}

#[test]
fn partner_benefit_refusals_are_named_and_open_the_page() {
    let issues = |s: &Interview| estimate(&s.draft).issues;
    // Something else applies: the stored eligibility false; the plan is
    // left out with a notice (Form 7206 line 1).
    let mut s = partner_benefits_session(vec![k1_1065_benefits("Synthetic Partners", 0, 0, 0)]);
    let mut value = open_benefits(&mut s);
    value.partnerships[0].health = se_health_test_year(100_000, true);
    se_health_test_unreimbursed(&mut value.partnerships[0].health);
    accepted(&save_benefits(&mut s, value));
    let r = estimate(&s.draft);
    assert!(noted(&r, "WARN_SE_HEALTH_LEFT_OUT"));
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.english.contains("entity payment")
                && n.english.contains("medical premiums"))
    );
    // Another plan type on box 13 code R.
    let mut s =
        partner_benefits_session(vec![k1_1065_benefits("Synthetic Partners", 0, 300_000, 0)]);
    let mut value = open_benefits(&mut s);
    value.partnerships[0].retirement = PartnerPlan::Other;
    accepted(&save_benefits(&mut s, value));
    assert_eq!(stored_1065s(&s)[0].retirement, PartnerRetirement::Other);
    assert_eq!(open_benefits(&mut s).partnerships[0].retirement, PartnerPlan::Other);
    let found = issues(&s);
    let other =
        found.iter().find(|i| i.section == "Partner retirement").expect("the other-plan refusal");
    assert!(
        other.message.contains("IRA payment")
            && other.message.contains("Schedule 1 line 16 or 20")
            && other.message.contains("Form 7206 line 9"),
        "{other:?}"
    );
    // Box 13 code R entered on the K-1 before its plan is chosen, and box 13
    // code M before the coverage: refused by name on the review, and both
    // open the self-employed benefits page.
    let s =
        partner_benefits_session(vec![k1_1065_benefits("Synthetic Partners", 600_000, 300_000, 0)]);
    let found = issues(&s);
    assert!(
        found
            .iter()
            .any(|i| i.section == "Partner retirement" && i.message.contains("box 13 code R")),
        "{found:?}"
    );
    assert!(found.iter().any(|i| i.section == "Partner health insurance"));
    for issue in found.iter().filter(|i| i.section.starts_with("Partner ")) {
        assert_eq!(
            crate::interview::issue_page(
                &s.draft,
                &issue.context,
                Some({
                    let m = &issue.display;
                    m.key.as_str()
                })
            ),
            Some(TaxSection::SelfEmployedBenefits),
            "{issue:?}"
        );
    }
    // Box 13 retains code O: royalty code I and the benefit codes are
    // calculated or left out with a notice, not refused.
    assert!(PartnerK1Item::OtherDeductions.refusal().contains("code O dependent care benefits"));
}

#[test]
fn partner_benefits_page_saves_and_reopens_a_solo_401k_plan() {
    // Pub. 560 chapter 5 / K-1 (1065) box 13 code R: the plan choice survives save, reopen and restart.
    let plan = PartnerPlan::Qualified {
        rate: 0,
        contribution: 0,
        deferral: 500_000,
        catch_up: 0,
        roth: 0,
    };
    let mut s =
        partner_benefits_session(vec![k1_1065_benefits("Synthetic Partners", 0, 300_000, 0)]);
    let mut value = open_benefits(&mut s);
    value.partnerships[0].retirement = plan.clone();
    accepted(&save_benefits(&mut s, value));
    assert!(matches!(
        stored_1065s(&s)[0].retirement,
        PartnerRetirement::Plan(BusinessRetirement::Qualified { deferral: 500_000, .. })
    ));
    assert_eq!(open_benefits(&mut s).partnerships[0].retirement, plan);
    assert_eq!(open_benefits(&mut restart(s.draft.clone())).partnerships[0].retirement, plan);
}

#[test]
fn partner_benefits_page_reproduces_the_partner_health_fixtures() {
    // Every return: single, W-2 $60,000 ($6,500 withheld); one K-1 entered
    // on the Schedule K-1 topic (box 1 $20,000, statement QBI $20,000,
    // opening basis and 2025 Form 6198 capacity $10,000, over 500 hours).
    let partner = |interest: PartnerInterest, services: i64, se: i64, qualified_se: i64| {
        let mut page = k1_1065("Synthetic Partners");
        page.interest = interest;
        page.ordinary = 2_000_000;
        page.services = services;
        page.se = se;
        page.se_gross = SeGross::Nonfarm { amount: se };
        page.qbi = PartnerQbi::Statement {
            section1231_qbi: 0,
            ptp_income: 0,
            reit_dividends: 0,
            income: 2_000_000,
            wages: 0,
            property: 0,
            service: false,
        };
        page.qbi_records = PartnerQbiRecords { qualified_se_gross: qualified_se, history: None };
        page.basis.start = k1_1065_continuing(
            1_000_000,
            ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 },
        );
        page
    };
    let general =
        || PartnerInterest::General { participation: Participation::Over500Hours { hours: 501 } };
    let with_boxes = |mut page: K1Partnership, medical: i64, pensions: i64| {
        page.more = k1_1065_more();
        {
            let K1PartnershipOtherBoxes { medical_insurance, pensions: box_13r, .. } =
                &mut page.more;
            *medical_insurance = medical;
            *box_13r = pensions;
        }
        page
    };
    let run = |page: K1Partnership, plan: PartnerPlan| {
        let mut s = partner_benefits_session(vec![page]);
        let mut value = open_benefits(&mut s);
        value.partnerships[0].health = se_health_test_year(600_000, true);
        value.partnerships[0].retirement = plan;
        accepted(&save_benefits(&mut s, value));
        let r = supported(&s.draft);
        // The hub names the row with something entered.
        let hub = open_adjustments_hub(&mut s);
        assert!(
            hub_rows(&hub).contains(&row_of(
                "Self-employed health insurance and retirement plan",
                "Synthetic Partners",
                TaxSection::SelfEmployedBenefits
            )),
            "{:?}",
            hub_rows(&hub)
        );
        r
    };

    // 1. partner_paid_health_enters_income_once_and_reuses_shared_deduction
    // (and the reconciled record of
    // partner_paid_health_reconciles_taxable_subset_before_deduction): the
    // partnership paid $6,000 of premiums as box 4a guaranteed payments,
    // box 13 code M $6,000; box 14A $26,000; qualified SE gross $20,000.
    //   Schedule E $20,000 + $6,000 = $26,000. SE $26,000 x 0.9235 =
    //   $24,011.00; tax x 15.3% = $3,673.68, $3,674; half $1,837.
    //   Form 7206: line 1/3 $6,000; line 4 $26,000; line 5-8 $1,837; line 13
    //   $24,163; line 14 $6,000 (Schedule 1 line 17). AGI $60,000 + $26,000
    //   - $1,837 - $6,000 = $78,163. QBI $20,000 - $1,837 x 20/26 ($1,413)
    //   - $6,000 x 20/26 ($4,615) = $13,972, deduction $2,794. Taxable
    //   $78,163 - $16,100 - $2,794 = $59,269; tax (midpoint $59,275) $1,240
    //   + $4,560 + 22% x $8,875 = $7,752.50, $7,753; plus SE $3,674 =
    //   $11,427; balance $11,427 - $6,500 = $4,927.
    let r = run(
        with_boxes(partner(general(), 600_000, 2_600_000, 2_000_000), 600_000, 0),
        PartnerPlan::None,
    );
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 6_000);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 1_837);
    assert_eq!(return_line(&r, "11a / 11b"), 7_816_300);
    assert_eq!(return_line(&r, "15"), 5_926_900);
    assert_eq!(return_line(&r, "16"), 775_300);
    assert_eq!(r.balance, Some(492_700));

    // 2. partner_paid_health_limited_partner_uses_service_earnings_cap
    // (question Q3: a limited partner's row): box 4a $6,000 of premiums,
    // box 13 code M $6,000, box 14A $6,000 (services only), qualified SE
    // gross $0.
    //   Schedule E $26,000. SE $6,000 x 0.9235 = $5,541; tax $847.77, $848;
    //   half $423.89, $424. Form 7206 line 4 $6,000, line 13 $5,576, line 14
    //   $5,576; the other $424 is a medical expense (standard deduction).
    //   AGI $60,000 + $26,000 - $424 - $5,576 = $80,000. QBI $20,000 (no
    //   qualified SE gross), deduction $4,000. Taxable $59,900; tax
    //   (midpoint $59,925) $5,800 + 22% x $9,525 = $7,895.50, $7,896; plus
    //   $848 = $8,744; balance $2,244.
    let limited = PartnerInterest::LimitedPartner {
        participation: LimitedParticipation::Over500Hours { hours: 600 },
    };
    let r = run(with_boxes(partner(limited, 600_000, 600_000, 0), 600_000, 0), PartnerPlan::None);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 5_576);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 424);
    assert_eq!(return_line(&r, "11a / 11b"), 8_000_000);
    assert_eq!(return_line(&r, "15"), 5_990_000);
    assert_eq!(return_line(&r, "16"), 789_600);
    assert_eq!(r.balance, Some(224_400));

    // 3. partner_sep_coordinates_health_qbi_and_keeps_se_tax_unchanged:
    // $6,000 of premiums paid personally on a partnership-named policy (no
    // box 13 code M), box 13 code R $3,000 to a 25% partnership SEP; box 14A
    // $20,000.
    //   SE $20,000 x 0.9235 = $18,470; tax $2,825.91, $2,826; half $1,413.
    //   SEP limit ($20,000 - $1,413) x 0.25 / 1.25 = $3,717.40 >= $3,000:
    //   Schedule 1 line 16 $3,000 (Form 7206 line 9). Form 7206 line 13
    //   $20,000 - $1,413 - $3,000 = $15,587; line 14 $6,000. AGI $60,000 +
    //   $20,000 - $1,413 - $3,000 - $6,000 = $69,587. QBI $20,000 - $1,413 -
    //   $6,000 - $3,000 = $9,587, deduction $1,917. Taxable $69,587 -
    //   $16,100 - $1,917 = $51,570; tax (midpoint $51,575) $5,800 + 22% x
    //   $1,175 = $6,058.50, $6,059; plus $2,826 = $8,885; balance $2,385.
    let r = run(
        with_boxes(partner(general(), 0, 2_000_000, 2_000_000), 0, 300_000),
        PartnerPlan::Sep { rate: 2500 },
    );
    assert_eq!(value(&r, "schedule_1_self_employed_retirement"), 3_000);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 6_000);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 1_413);
    assert_eq!(return_line(&r, "11a / 11b"), 6_958_700);
    assert_eq!(return_line(&r, "15"), 5_157_000);
    assert_eq!(return_line(&r, "16"), 605_900);
    assert_eq!(r.balance, Some(238_500));
}

#[test]
fn partner_income_source_is_checked_against_its_allowed_set_on_save() {
    // Not read (an adult): anything but Not needed is refused by name.
    let adult = complete(FederalStatus::Single);
    let mut page = k1_1065("Synthetic Partners");
    page.income_source = K1IncomeSource::Capital;
    let message = problem_text({
        let mut source = Interview::new(TEST_USER, adult.clone());
        current_document_refusal(
            "income-source-2728",
            &mut source,
            vec![document(0, IncomeDocument::K1Partnership(page.clone()))],
            "SOURCE_NOT_NEEDED",
            "[documents/documents/0/document/income_source]",
        )
    });
    assert!(message.contains("choose Not needed"), "{message}");
    // Read through the dependent worksheet (spouse claimable, joint).
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    accepted(&save_1065s(&d, vec![page]).1);
    let message = problem_text({
        let mut source = Interview::new(TEST_USER, d.clone());
        current_document_refusal(
            "income-source-2832",
            &mut source,
            vec![document(0, IncomeDocument::K1Partnership(k1_1065("Synthetic Partners")))],
            "SOURCE_PARTNER_NEEDED",
            "[documents/documents/0/document/income_source]",
        )
    });
    assert!(message.contains("Standard Deduction Worksheet"), "{message}");
}

#[test]
fn k1_1120s_loan_repayment_term_matches_the_loan_on_save() {
    // The save twin of the client rule SCORP_LOAN_REPAYMENT_TERM: a written
    // note with principal repaid states its holding period, and an open
    // account never has one.
    let mut page = k1_1120s_fixture(10_000);
    page.basis.loans = vec![ScorpLoan {
        kind: ScorpLoanKind::FormalNote,
        repayment_term: ScorpNoteHolding::NoRepayment,
        principal: 300_000,
        basis: 300_000,
        advances: 0,
        repayments: 100_000,
    }];
    rejected_with(save_k1_1120s(vec![page.clone()]).1, "SCORP_LOAN_REPAYMENT_TERM");
    page.basis.loans[0].repayment_term = ScorpNoteHolding::Single { term: GainTerm::Long };
    accepted(&save_k1_1120s(vec![page.clone()]).1);
    page.basis.loans[0].kind = ScorpLoanKind::OpenAccount;
    rejected_with(save_k1_1120s(vec![page.clone()]).1, "SCORP_LOAN_REPAYMENT_TERM");
    page.basis.loans[0].repayment_term = ScorpNoteHolding::NoRepayment;
    accepted(&save_k1_1120s(vec![page]).1);
}

#[test]
fn k1_1120s_continuing_shareholder_with_new_at_risk_start_is_refused_by_name() {
    // The save twin of PARTNER_RISK_HISTORY_NEW: a shareholder before 2026
    // has a 2025 at-risk history (or a missing one), never a New start.
    let mut page = k1_1120s("Synthetic Corp");
    page.ordinary = 1_000_000;
    page.basis.start = k1_1120s_continuing(500_000, 500_000);
    accepted(&save_k1_1120s(vec![page.clone()]).1);
    if let K1ScorpStart::Continuing { risk_history, .. } = &mut page.basis.start {
        *risk_history = ScorpRiskStart::Unknown;
    }
    accepted(&save_k1_1120s(vec![page.clone()]).1);
    if let K1ScorpStart::Continuing { risk_history, .. } = &mut page.basis.start {
        *risk_history = ScorpRiskStart::New;
    }
    rejected_with(save_k1_1120s(vec![page.clone()]).1, "SCORP_RISK_HISTORY_NEW");
    // A shareholder from 2026 (the New start) is accepted.
    page.basis.start = K1ScorpStart::New;
    accepted(&save_k1_1120s(vec![page]).1);
}

#[test]
fn k1_1120s_credit_codes_reach_form_3800_and_reopen() {
    // A materially participating shareholder (over 500 hours) with box 1 $0
    // and box 13 code J $1,000: tax $5,023 on the W-2, the nonpassive
    // Part III line 4b credit is used in full.
    let mut page = k1_1120s_fixture(0);
    {
        let K1ScorpOtherBoxes { credits, .. } = k1_1120s_more_mut(&mut page);
        *credits = vec![
            K1Credit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                line: None,
                code: K1CreditCode::WorkOpportunity,
                amount: 100_000,
            }
            .into(),
        ];
    }
    let mut session = k1_1120s_fixture_session(page.clone());
    let r = supported(&session.draft);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year_used"), 1_000);
    assert_eq!(value(&r, "general_business_credit"), 1_000);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page.clone())]);
    // The same code twice is refused on save (K1_CREDIT_CODE_TWICE).
    {
        let K1ScorpOtherBoxes { credits, .. } = k1_1120s_more_mut(&mut page);
        credits.push(
            K1Credit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                line: None,
                code: K1CreditCode::WorkOpportunity,
                amount: 100,
            }
            .into(),
        );
    }
    let (_, reply) = save_k1_1120s(vec![page]);
    rejected_with(reply, "K1_CREDIT_CODE_TWICE");
}

// Form 8586 map (unit form-8586): the K-1 (1065) and K-1 (1120-S) pages take
// box 15 / box 13 codes C and D, the stored records carry them to Form 3800
// line 4d through Form 8582-CR, and the import instructions name them.
// Single, W-2 $60,000: TI 43,900, tax 5,023; the allowance tax is 5,023 - 2,023
// = 3,000, so code C $1,000 and code D $1,500 are allowed in full ($2,500).
#[test]
fn k1_codes_c_and_d_reach_form_3800_line_4d_and_reopen() {
    let mut page = k1_1065("Synthetic Housing Fund LP");
    page.interest = PartnerInterest::LimitedPartner {
        participation: LimitedParticipation::Investor { prior_years: 0 },
    };
    page.more.credits = vec![
        K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            line: None,
            code: K1CreditCode::LowIncomeHousingJ5,
            amount: 100_000,
        }
        .into(),
        K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            line: None,
            code: K1CreditCode::LowIncomeHousingOther,
            amount: 150_000,
        }
        .into(),
    ];
    // A housing credit needs the typed pre-1990 confirmation (F3).
    let (_, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    rejected_with(reply, "K1_HOUSING_NEEDS_CONFIRMATION");
    page.more.housing_confirmed = Some(true);
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "form8582cr_line_3a"), 2_500);
    assert_eq!(value(&r, "form8582cr_line_37"), 2_500);
    assert_eq!(value(&r, "business_credit_low_income_housing_current_year_passive"), 2_500);
    assert_eq!(value(&r, "form_3800_line_33"), 2_500);
    assert_eq!(value(&r, "general_business_credit"), 2_500);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Partnership(page.clone())]);
    // The same code twice is refused on save.
    page.more.credits.push(
        K1Credit {
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
            line: None,
            code: K1CreditCode::LowIncomeHousingOther,
            amount: 100,
        }
        .into(),
    );
    let (_, reply) = save_1065s(&complete(FederalStatus::Single), vec![page]);
    rejected_with(reply, "K1_CREDIT_CODE_TWICE");
    // The 1120-S page takes them too (box 13).
    let mut shareholder = k1_1120s_fixture(0);
    {
        let K1ScorpOtherBoxes { credits, housing_confirmed, .. } =
            k1_1120s_more_mut(&mut shareholder);
        *credits = vec![
            K1Credit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                line: None,
                code: K1CreditCode::LowIncomeHousingJ5,
                amount: 100_000,
            }
            .into(),
        ];
        *housing_confirmed = Some(true);
    }
    let session = k1_1120s_fixture_session(shareholder);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "form8582cr_line_3a"), 1_000);
    assert_eq!(value(&r, "business_credit_low_income_housing_current_year_passive_allowed"), 1_000);
    assert_eq!(value(&r, "general_business_credit"), 1_000);
}

#[test]
fn k1_import_instructions_name_codes_c_and_d_and_the_pre_1990_case() {
    let source = include_str!("../../passthrough/k1_page.rs");
    // Each of the two imports (1120-S box 13, 1065 box 15) lists the codes it
    // reads and the categories it leaves unentered.
    for box_number in ["13", "15"] {
        assert!(
            source
                .contains(&alloc::format!("`credits` lists each supported box {box_number} code"))
        );
        for code in ["C (LowIncomeHousingJ5)", "D (LowIncomeHousingOther)", "J (WorkOpportunity)"] {
            assert_eq!(source.matches(code).count(), 2);
        }
        assert!(source.contains(&alloc::format!("Credits for box {box_number} codes H and AP through AU; CreditRecapture for box {box_number} code AX;")));
    }
    // The pre-1990 case is a typed confirmation the import never reads (F3,
    // 2026-10-09; Form 8582-CR Part III), no longer a clause of "Something
    // else applies".
    assert_eq!(
        source.matches("Never read `investment_confirmed` or `housing_confirmed`").count(),
        2
    );
    assert!(!source.contains("when you acquired your interest before 1990"));
    // The model's code list is the three pieces the instructions name.
    assert_eq!(K1CreditCode::LowIncomeHousingJ5.credit(), BusinessCredit::LowIncomeHousing);
}

#[test]
fn k1_1065_rentals_map_onto_the_record_and_reopen_unchanged() {
    let mut page = k1_1065("Synthetic Rentals LP");
    page.rentals = k1_rental_boxes(-3_000_000, 400_000);
    page.basis.start = k1_1065_continuing(5_000_000, ScorpRiskStart::Unknown);
    if let BasisStart::Continuing { carried, .. } = &mut page.basis.start {
        carried.rental_real_estate_passive_loss = 250_000;
        carried.other_rental_passive_loss = 75_000;
    }
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    let stored = stored_1065s(&session).remove(0);
    let amounts = K1RentalAmounts {
        real_estate_property: Default::default(),
        other_property: Default::default(),
        real_estate: -3_000_000,
        real_estate_carryover: 250_000,
        other: 400_000,
        other_carryover: 75_000,
    };
    assert_eq!(stored.regular.rentals, amounts);
    // Same as regular for AMT: the AMT record carries the same amounts.
    assert_eq!(stored.alternative.as_ref().unwrap().rentals, amounts);
    assert_eq!(
        stored.rental_facts,
        K1RentalFacts {
            disposed: Vec::new(),
            participation: K1RentalParticipation::Active,
            real_estate_qbi: Some(K1RentalQbi { wages: 120_000, property: 5_000_000 }),
            other_qbi: None,
            carryover_active: true,
        }
    );
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Partnership(page)]);
}

#[test]
fn k1_1065_limited_partner_cannot_actively_participate_on_both_sides() {
    let save = |page: K1Partnership| save_1065s(&complete(FederalStatus::Single), vec![page]).1;
    let mut page = k1_1065("Synthetic Rentals LP");
    page.rentals = k1_rental_boxes(-3_000_000, 0);
    // A general partner may answer Active.
    accepted(&save(page.clone()));
    page.interest = PartnerInterest::LimitedPartner {
        participation: LimitedParticipation::Investor { prior_years: 0 },
    };
    assert_eq!(page.rule(), Some("PARTNER_LIMITED_NOT_ACTIVE"));
    rejected_with(save(page.clone()), "PARTNER_LIMITED_NOT_ACTIVE");
    // The stored combination is not valid either.
    let mut record = page.record();
    assert!(!record.valid());
    for participation in [K1RentalParticipation::NotActive, K1RentalParticipation::Professional] {
        page.rentals.participation = participation;
        accepted(&save(page.clone()));
        record.rental_facts.participation = participation;
        assert!(record.valid());
    }
}

#[test]
fn k1_1120s_rentals_map_onto_the_record_and_reopen_unchanged() {
    let mut page = k1_1120s("Synthetic Rentals Inc");
    page.rentals = k1_rental_boxes(-2_000_000, 0);
    page.basis.start = k1_1120s_continuing(5_000_000, 5_000_000);
    if let K1ScorpStart::Continuing { carried, .. } = &mut page.basis.start {
        carried.rental_real_estate_passive_loss = 125_000;
    }
    let (mut session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Scorp { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    let amounts = K1RentalAmounts {
        real_estate: -2_000_000,
        real_estate_carryover: 125_000,
        ..Default::default()
    };
    assert_eq!(stored.rentals, amounts);
    assert_eq!(stored.amt.as_ref().unwrap().rentals, amounts);
    assert_eq!(stored.rental_facts.participation, K1RentalParticipation::Active);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page)]);
}

// Unit k1-rentals: a Professional box 2 answer needs the filer's status from
// the People page; saving the status turns the K-1's refusal into a
// calculation.
#[test]
fn a_k1_box_2_professional_answer_needs_the_status_recorded_on_the_people_page() {
    let mut page = k1_1065("Synthetic Rentals LP");
    page.rentals = K1RentalBoxes {
        real_estate_section1231: 0,
        real_estate_unrecaptured: 0,
        other_section1231: 0,
        other_unrecaptured: 0,
        disposed: Vec::new(),
        real_estate: -1_000_000,
        other: 0,
        participation: K1RentalParticipation::Professional,
        real_estate_qbi: None,
        other_qbi: None,
        carryover_active: true,
    };
    page.basis.start = k1_1065_continuing(
        5_000_000,
        ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 },
    );
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page]);
    accepted(&reply);
    let r = estimate(&session.draft);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_K1_RENTAL_PROFESSIONAL_STATUS"),
        "{r:?}"
    );
    let mut you = crate::interview::filer(&session.draft.people.as_ref().unwrap().taxpayer);
    you.real_estate_professional =
        Some(ProfessionalRentals { single_activity: false, over_500_hours: false }.into());
    let request = save_you(&session.draft.clone(), you);
    accepted(&send(&mut session, request));
    let r = supported(&session.draft);
    let id = session.draft.income[1].id;
    assert_eq!(value(&r, &alloc::format!("partnership_{id}_rental_real_estate_net")), -10_000);
}

#[test]
fn k3_on_a_1065_k1_maps_onto_the_record_and_reopens_unchanged() {
    let mut page = k1_1065("Synthetic Global Fund LP");
    page.basis.start = k1_1065_continuing(5_000_000, ScorpRiskStart::Unknown);
    page.more.foreign_taxes = 90_000;
    page.schedule_k3 = crate::K3Attached::Attached(k3_every_field());
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    let stored = stored_1065s(&session).remove(0);
    assert_eq!(stored.schedule_k3, Some(k3_every_field()));
    assert_eq!(stored.separate.as_ref().unwrap().foreign_taxes, 90_000);
    assert!(stored.valid());
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Partnership(page.clone())]);
    // Not attached: no stored K-3, and the box 21 amount alone still reopens.
    page.schedule_k3 = crate::K3Attached::NotAttached;
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    assert_eq!(stored_1065s(&session).remove(0).schedule_k3, None);
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Partnership(page)]);
}

#[test]
fn k3_on_a_1120s_k1_maps_onto_the_record_and_reopens_unchanged() {
    let mut page = k1_1120s("Synthetic Consulting Inc");
    page.basis.start = k1_1120s_continuing(5_000_000, 5_000_000);
    page.more.foreign_taxes = 300_000;
    page.schedule_k3 = crate::K3Attached::Attached(k3_every_field());
    let (mut session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    let stored = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Scorp { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(stored.schedule_k3, Some(k3_every_field()));
    assert_eq!(stored.foreign_taxes, 300_000);
    assert!(stored.valid());
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page)]);
}

#[test]
fn k3_rules_reject_on_both_sides() {
    use crate::tests::foreign::foreign::{k3_blank, k3_row};
    let twice = |item: crate::K3Item| {
        let mut k3 = k3_blank();
        k3.items = vec![item, item];
        k3
    };
    let above = || {
        let mut k3 = k3_blank();
        k3.countries = vec![k3_row("JA", crate::K3Category::General, 1_000, 1_001, 0)];
        k3
    };
    for (k3, key) in [
        (twice(crate::K3Item::Loans), "K3_ITEM_TWICE"),
        (above(), "FOREIGN_QUALIFIED_ABOVE_INCOME"),
    ] {
        let mut page = k1_1065("Synthetic Global Fund LP");
        page.schedule_k3 = crate::K3Attached::Attached(k3.clone());
        assert_eq!(page.rule(), Some(key));
        rejected_with(save_1065s(&complete(FederalStatus::Single), vec![page]).1, key);
        let mut page = k1_1120s("Synthetic Consulting Inc");
        page.schedule_k3 = crate::K3Attached::Attached(k3);
        assert_eq!(page.rule(), Some(key));
        rejected_with(save_k1_1120s(vec![page]).1, key);
    }
    // The stored record holds at most 40 country rows and two-letter codes.
    for (rows, code, valid) in [(40, "JA", true), (41, "JA", false), (1, "JAP", false)] {
        let mut k3 = k3_blank();
        k3.countries =
            (0..rows).map(|_| k3_row(code, crate::K3Category::General, 1, 0, 0)).collect();
        let mut page = k1_1065("Synthetic Global Fund LP");
        page.schedule_k3 = crate::K3Attached::Attached(k3);
        assert_eq!(page.record().valid(), valid, "{rows} rows {code}");
    }
    // Equal income and qualified dividends are the boundary: accepted.
    let mut k3 = k3_blank();
    k3.countries = vec![k3_row("JA", crate::K3Category::General, 1_000, 1_000, 0)];
    let mut page = k1_1065("Synthetic Global Fund LP");
    page.schedule_k3 = crate::K3Attached::Attached(k3);
    assert_eq!(page.rule(), None);
    accepted(&save_1065s(&complete(FederalStatus::Single), vec![page]).1);
}

// A K-1 (1065) with investment credit property: box 20 code D rehabilitation
// expenditures of $500,000 for a building placed in service in 2025, a
// general partner who materially participates: the credit 4% x 500,000 =
// 20,000, a specified credit (Form 3800 line 4k) used against the tax of
// $5,023 on taxable income $43,900 and the rest carried.
#[test]
fn k1_investment_property_reaches_form_3468_and_reopens() {
    let mut page = k1_1065("Synthetic Historic Fund LP");
    page.interest =
        PartnerInterest::General { participation: Participation::Over500Hours { hours: 600 } };
    page.more.investment_property = vec![crate::K1InvestmentProperty::RehabilitationOther {
        id: 0,
        own_form: None,
        expenditures: 50_000_000,
        placed_year: 2025,
    }];
    page.more.allowed_investment_confirmations = vec![0, 1];
    // The personal confirmation is required with the list (needed only then).
    let draft = complete(FederalStatus::Single);
    let (_, reply) = save_1065s(&draft, vec![page.clone()]);
    rejected_with(reply, "K1_INVESTMENT_NEEDS_CONFIRMATION");
    page.more.investment_confirmed = Some(K1InvestmentConfirmation::Yes);
    let (mut session, reply) = save_1065s(&draft, vec![page.clone()]);
    accepted(&reply);
    let r = supported(&session.draft);
    let tax = value(&r, "regular_income_tax");
    assert_eq!(tax, 5_023);
    assert_eq!(value(&r, "partner_2_investment_0_form3468_basis"), 500_000);
    assert_eq!(value(&r, "partner_2_investment_0_form3468_credit"), 20_000);
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year"), 20_000);
    assert_eq!(value(&r, "form_3800_line_30"), 20_000);
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year_used"), tax);
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year_to_next_year"), 20_000 - tax);
    // Placed in service in 2025: no basis reduction in 2026.
    assert_eq!(value(&r, "partner_2_investment_0_basis_reduction"), 0);
    let mut accepted_page = page.clone();
    *accepted_page.more.investment_property[0].id_mut() = 3;
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Partnership(accepted_page)]);
    // Not confirmed: left out with a notice (the list is not an answer the
    // estimate can rely on).
    page.more.investment_confirmed = Some(K1InvestmentConfirmation::No);
    let (session, reply) = save_1065s(&draft, vec![page]);
    accepted(&reply);
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_K1_INVESTMENT_CREDIT_LEFT_OUT"));
    assert!(r.lines.iter().all(|l| l.id != "business_credit_rehabilitation_current_year"));
}

#[test]
fn k1_1120s_investment_property_reaches_form_3468_and_reopens() {
    let mut page = k1_1120s_fixture(0);
    {
        let more = k1_1120s_more_mut(&mut page);
        more.investment_property = vec![crate::K1InvestmentProperty::CleanElectricity {
            id: 0,
            own_form: None,
            basis: 5_000_000,
            storage: true,
            rate: crate::EnergyRate {
                increased_rate: true,
                domestic_content: false,
                energy_community: true,
            },
        }];
    }
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let draft = session.draft.clone();
    let request = save_topic(
        &draft,
        TaxSection::PassThrough,
        vec![document(0, IncomeDocument::K1Scorp(page.clone()))],
    );
    rejected_with(send(&mut session, request), "K1_INVESTMENT_NEEDS_CONFIRMATION");
    k1_1120s_more_mut(&mut page).investment_confirmed = Some(K1InvestmentConfirmation::Yes);
    let session = k1_1120s_fixture_session(page.clone());
    let r = supported(&session.draft);
    // 30% + 10% of $50,000 = 20,000, storage technology (Part V line 3a);
    // half of it reduces the shareholder's stock basis.
    assert_eq!(value(&r, "scorp_2_investment_0_form3468_credit"), 20_000);
    assert_eq!(value(&r, "scorp_2_investment_0_basis_reduction"), 10_000);
    assert_eq!(printed_line(&r, "scorp_2_investment_0_form3468_basis").1, "3a");
    assert_eq!(value(&r, "business_credit_clean_electricity_current_year"), 20_000);
}

// Unit B9-1: the box 11 (10) and box 13 (12) lists, what happened to the
// interest, the holding period and the situations write the stored records,
// reopen as the same pages and give the same estimate; the page rules and
// their save-time twins reject the same combinations.
#[test]
fn k1_1065_box_items_disposition_and_situations_round_trip_through_save_and_reopen() {
    let draft = complete(FederalStatus::Single);
    let mut page = k1_1065("Synthetic Partners D");
    page.final_k1 = true;
    page.basis.start = BasisStart::Continuing {
        opening: 10_000_000,
        risk_history: ScorpRiskStart::Detailed { capacity: 10_000_000, profit: 0, loss: 0 },
        carried: k1_1065_carried_zero(),
    };
    {
        let more = &mut page.more;
        more.income = vec![
            crate::PartnerIncome {
                code: crate::PartnerIncomeCode::PortfolioIncome,
                amount: 200_000,
            },
            crate::PartnerIncome {
                code: crate::PartnerIncomeCode::NonportfolioLong,
                amount: -50_000,
            },
        ];
        more.deductions = vec![crate::PartnerDeduction {
            code: crate::PartnerDeductionCode::NoncashFifty,
            amount: 300_000,
        }];
        more.deferred_compensation_tax = 7_700;
        more.crp_payments = 3_300;
        more.sale_ordinary = 800_000;
        more.sale_collectibles = 100_000;
        more.sale_unrecaptured = 200_000;
        more.term = K1StockTerm::LongTerm;
    }
    page.disposition = PartnerDisposition::Sold { proceeds: 9_000_000 };
    page.special = vec![crate::PartnerSpecial::UnreimbursedExpenses];
    page.qbi = PartnerQbi::Statement {
        section1231_qbi: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        reit_dividends: 12_300,
        ptp_income: -4_500,
    };
    assert_eq!(page.rule(), None);
    let (mut session, reply) = save_1065s(&draft, vec![page.clone()]);
    accepted(&reply);
    let stored = stored_1065s(&session);
    let record = &stored[0];
    assert!(record.final_k1);
    assert_eq!(record.income.len(), 2);
    assert_eq!(record.deductions[0].amount, 300_000);
    assert_eq!(record.deferred_compensation_tax, 7_700);
    assert_eq!(record.crp_payments, 3_300);
    assert_eq!(record.sale.ordinary, 800_000);
    assert_eq!(record.interest_term, Some(GainTerm::Long));
    assert_eq!(record.disposition, crate::K1Disposition::Sold { proceeds: 9_000_000 });
    assert_eq!(record.special, vec![crate::PartnerSpecial::UnreimbursedExpenses]);
    assert!(record.complete());
    let qbi = record.qbi.as_ref().unwrap();
    assert_eq!((qbi.reit_dividends, qbi.ptp_income), (12_300, -4_500));
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Partnership(page.clone())]);
    // A second save of the reopened page writes the same record.
    let (second, reply) = save_1065s(&draft, vec![page.clone()]);
    accepted(&reply);
    assert_eq!(stored_1065s(&second)[0], *record);
    // The page rules and their twins.
    let mut bad = page.clone();
    bad.more.income.push(bad.more.income[0].clone());
    assert_eq!(bad.rule(), Some("K1_INCOME_CODE_TWICE"));
    rejected_with(save_1065s(&draft, vec![bad]).1, "K1_INCOME_CODE_TWICE");
    let mut bad = page.clone();
    bad.more.income[0].amount = -100;
    assert_eq!(bad.rule(), Some("K1_INCOME_CODE_SIGN"));
    rejected_with(save_1065s(&draft, vec![bad]).1, "K1_INCOME_CODE_SIGN");
    let mut bad = page.clone();
    bad.more.deductions.push(bad.more.deductions[0].clone());
    assert_eq!(bad.rule(), Some("K1_DEDUCTION_CODE_TWICE"));
    let mut bad = page.clone();
    bad.special = vec![crate::PartnerSpecial::RetiringPayments; 2];
    assert_eq!(bad.rule(), Some("K1_SITUATION_TWICE"));
    // F4c: a final K-1 with the interest still owned.
    let mut bad = page.clone();
    bad.disposition = PartnerDisposition::Kept;
    bad.more.sale_ordinary = 0;
    bad.more.sale_collectibles = 0;
    bad.more.sale_unrecaptured = 0;
    assert_eq!(bad.rule(), Some("K1_FINAL_NEEDS_DISPOSITION"));
    rejected_with(save_1065s(&draft, vec![bad]).1, "K1_FINAL_NEEDS_DISPOSITION");
    // F4d: the sale codes with the interest still owned.
    let mut bad = page.clone();
    bad.final_k1 = false;
    bad.disposition = PartnerDisposition::Kept;
    assert_eq!(bad.rule(), Some("K1_SALE_CODES_NEED_SALE"));
    // A sale or a liquidation needs the holding period; a liquidation the
    // final K-1 box.
    let mut bad = page.clone();
    bad.more.term = K1StockTerm::NotNeeded;
    assert_eq!(bad.rule(), Some("K1_SALE_NEEDS_HOLDING_PERIOD"));
    let mut bad = page.clone();
    bad.disposition = PartnerDisposition::LiquidatedForCash;
    bad.more.sale_ordinary = 0;
    bad.more.sale_collectibles = 0;
    bad.more.sale_unrecaptured = 0;
    bad.final_k1 = false;
    assert_eq!(bad.rule(), Some("K1_LIQUIDATION_NEEDS_FINAL"));
}

#[test]
fn k1_1120s_box_items_disposition_and_situations_round_trip_through_save_and_reopen() {
    let mut page = k1_1120s("Synthetic Corp D");
    page.ordinary = 1_000_000;
    page.qbi = K1ScorpQbi::NoStatement;
    page.basis.start = k1_1120s_continuing(10_000_000, 10_000_000);
    page.final_k1 = true;
    page.more.income = vec![
        crate::ScorpIncome { code: crate::ScorpIncomeCode::PortfolioIncome, amount: 200_000 },
        crate::ScorpIncome { code: crate::ScorpIncomeCode::Section1256, amount: -50_000 },
    ];
    page.more.deductions = vec![crate::ScorpDeduction {
        code: crate::ScorpDeductionCode::CashThirty,
        amount: 100_000,
    }];
    page.more.stock_term = K1StockTerm::LongTerm;
    page.disposition = ScorpDisposition::Sold { proceeds: 5_000_000 };
    page.special = vec![crate::ScorpSpecial::HousingBefore1990];
    assert_eq!(page.rule(), None);
    let (mut session, reply) = save_k1_1120s(vec![page.clone()]);
    accepted(&reply);
    let record = session
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Scorp { data } => Some(data.clone()),
            _ => None,
        })
        .unwrap();
    assert!(record.final_k1 && record.basis_complete());
    assert_eq!(record.income.len(), 2);
    assert_eq!(record.disposition, crate::K1Disposition::Sold { proceeds: 5_000_000 });
    assert_eq!(reopened_k1s(&mut session), vec![IncomeDocument::K1Scorp(page.clone())]);
    let mut bad = page.clone();
    bad.more.income[1].amount = 0;
    assert_eq!(bad.rule(), Some("K1_INCOME_CODE_SIGN"));
    rejected_with(save_k1_1120s(vec![bad]).1, "K1_INCOME_CODE_SIGN");
    let mut bad = page.clone();
    bad.disposition = ScorpDisposition::Kept;
    assert_eq!(bad.rule(), Some("K1_FINAL_NEEDS_DISPOSITION"));
    let mut bad = page.clone();
    bad.more.stock_term = K1StockTerm::NotNeeded;
    assert_eq!(bad.rule(), Some("K1_SALE_NEEDS_HOLDING_PERIOD"));
    let mut bad = page.clone();
    bad.special = vec![crate::ScorpSpecial::Farming; 2];
    assert_eq!(bad.rule(), Some("K1_SITUATION_TWICE"));
}

#[test]
fn estate_ein_leading_zero_survives_save_restart_and_failed_edit() {
    let mut page = k1_1041("Synthetic leading-zero trust");
    page.ein = "012345678".into();
    page.interest = 60_000;
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    assert_eq!(stored_k1_1041(&session).1.ein, "012345678");
    let accepted_draft = session.draft.clone();
    session.dirty = false;
    for invalid in ["12-3456789", "12345678", "12345678x", "１２３４５６７８９"] {
        let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(p) = &mut items[0].document else {
            panic!("estate page")
        };
        p.ein = invalid.into();
        let request = save_topic(&accepted_draft, TaxSection::PassThrough, items);
        rejected_with(send(&mut session, request), "ESTATE_EIN_INVALID");
        assert_eq!(session.draft, accepted_draft);
        assert!(!session.dirty);
    }
    let mut again = restart(accepted_draft.clone());
    let reopened = reopened_k1s(&mut again);
    let IncomeDocument::K1EstateTrust(page) = &reopened[0] else { panic!("estate page") };
    assert_eq!(page.ein, "012345678");
    assert_eq!(value(&estimate(&again.draft), "estate_trust_2_1_interest"), 600);
}

#[test]
fn k1_1041_pub559_dni_keeps_each_beneficiarys_rental_and_interest_character() {
    // Pub.559 (2025), printed p.32: DNI $3,000 consists of $1,800
    // rental income and $1,200 interest. Each equal beneficiary reports
    // $900 rental and $600 interest on their separate return.
    for beneficiary in ["Synthetic beneficiary A trust", "Synthetic beneficiary B trust"] {
        let mut page = k1_1041(beneficiary);
        page.interest = 60_000;
        page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
            box6: 0,
            box7: 90_000,
            box8: 0,
            activities: vec![estate_activity_page(
                "Allocated rental",
                crate::EstateActivityKind::RentalRealEstate,
                90_000,
            )],
        });
        let (session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "estate_trust_2_1_interest"), 600);
        assert_eq!(value(&result, "estate_trust_2_activity_3_33d"), 900);
        assert_eq!(value(&result, "estate_trust_2_activity_3_suspended"), 0);
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
}

#[test]
fn k1_1041_pub559_governing_instrument_preserves_independent_source_allocations() {
    // Pub.559 printed p.32 alternative: Harper receives $1,200 interest;
    // Drew receives $1,800 rental. These are separate beneficiary returns.
    let mut interest = k1_1041("Synthetic Harper source");
    interest.interest = 120_000;
    let (session, reply) = save_k1s(vec![interest]);
    accepted(&reply);
    assert_eq!(value(&supported(&session.draft), "estate_trust_2_1_interest"), 1_200);
    let mut rental = k1_1041("Synthetic Drew source");
    rental.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: 0,
        box7: 180_000,
        box8: 0,
        activities: vec![estate_activity_page(
            "Allocated rental",
            crate::EstateActivityKind::RentalRealEstate,
            180_000,
        )],
    });
    let (session, reply) = save_k1s(vec![rental]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "estate_trust_2_1_interest"), 0);
    assert_eq!(value(&result, "estate_trust_2_activity_3_33d"), 1_800);
}

#[test]
fn k1_1041_td9918_separate_beneficiaries_keep_their_67e_excess_deductions() {
    // TD9918 / Reg.1.642(h)-5 Example2: excess $11,000, shares
    // E75%=$8,250, F25%=$2,750. Uses the published source amounts;
    // ordinary 2026 beneficiary return is a derived application.
    let baseline = supported(&complete(FederalStatus::Single));
    for amount in [825_000, 275_000] {
        let mut page = k1_1041("Synthetic terminated fiduciary source");
        page.final_character = Some(estate_final_source(0, false));
        page.more.final_year = estate_final_amounts(amount);
        let (session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "estate_trust_67e_deduction"), amount / 100);
        assert_eq!(value(&result, "agi"), value(&baseline, "agi") - amount / 100);
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
}

#[test]
fn k1_1041_td9847_negative_qbi_is_independent_of_tax_exempt_dni() {
    // TD9847 Reg.1.199A-6(d)(3)(viii), actual2018 B: apportioned
    // QBI -$11,750 despite tax-exempt DNI. The loss source is historical;
    // the 2026 carry projection below does not reuse 2018 tax thresholds.
    let mut page = k1_1041("Synthetic independent QBI source");
    page.more.tax_exempt_interest = 50_000;
    page.qbi.statements.push(crate::EstateQbiStatementPage {
        allocations: Vec::new(),
        name: "DNI-apportioned business".into(),
        income: -1_175_000,
        wages: 1_875_000,
        property: 3_125_000,
        service: false,
        kind: crate::EstateQbiKind::Business,
        activities: vec![],
        passive_history: vec![],
    });
    let (session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 11_750);
    assert_eq!(value(&result, "estate_trust_2_14a_tax_exempt_interest"), 500);
}

#[test]
fn k1_1041_signed_activity_amt_adjustment_enters_the_passive_owner_once() {
    // Derived i1041sk1/6251/8582 fixture: regular1000-1500=-500;
    // AMT1000-1200=-200. Other passive profit200 leaves regular300
    // suspended, AMT0. The300 difference is already included in A.
    let mut page = k1_1041("Synthetic regular and AMT activity source");
    let mut loss =
        estate_activity_page("Loss activity", crate::EstateActivityKind::Business, 100_000);
    loss.depreciation = 150_000;
    page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: 120_000,
        box7: 0,
        box8: 0,
        activities: vec![
            loss,
            estate_activity_page("Profit activity", crate::EstateActivityKind::Business, 20_000),
        ],
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
    let K1EstateTrustBusiness::Amounts(source) = &mut page.business else { unreachable!() };
    let loss_id = source.activities[0].id;
    source.activities[0].amt_difference = 30_000;
    page.more.amt = K1EstateTrustAmt::CodeAOnly { a: 30_000 };
    page.amt_reconciliation = Some(crate::EstateAmtReconciliationPage {
        other_in_a: 30_000,
        adjustments: vec![crate::EstateAmtAdjustmentPage {
            code: crate::EstateAmtCode::DepreciationG,
            amount: 30_000,
            treatment: crate::EstateAmtTreatmentPage::Activity {
                included_in_a: true,
                activity: loss_id,
            },
        }],
    });
    let before = session.draft.clone();
    let reply = send(&mut session, save_topic(&before, TaxSection::PassThrough, items));
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "estate_trust_2_activity_3_suspended"), 300);
    assert_eq!(value(&result, "amt_estate_trust_2_activity_3_suspended"), 0);
    // Column (c) is the allowed deduction, not the net loss:
    // $1,500 depreciation less $300 suspended = $1,200.
    assert_eq!(value(&result, "estate_trust_2_activity_3_33c"), 1_200);
    assert_eq!(value(&result, "estate_trust_2_activity_3_33d"), 1_000);
    assert_eq!(value(&result, "estate_trust_2_activity_4_33d"), 200);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
}

// Main's boxes 6–8 source fixture, expressed through the per-activity page.
fn main_estate_activities(
    incomes: [i64; 3],
    deductions: [i64; 3],
    material: bool,
) -> K1EstateTrustBusiness {
    let kinds = [
        crate::EstateActivityKind::Business,
        crate::EstateActivityKind::RentalRealEstate,
        crate::EstateActivityKind::OtherRental,
    ];
    let mut rows = Vec::new();
    for (i, kind) in kinds.into_iter().enumerate() {
        if incomes[i] == 0 && deductions[i] == 0 {
            continue;
        }
        let mut row =
            estate_activity_page(["Business", "Real estate", "Other rental"][i], kind, incomes[i]);
        row.depreciation = deductions[i];
        if i == 0 && material {
            row.participation = crate::EstateParticipation::Material(ScorpParticipation {
                minutes: 30_001,
                ..participation_facts()
            });
        }
        rows.push(row);
    }
    K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: incomes[0],
        box7: incomes[1],
        box8: incomes[2],
        activities: rows,
    })
}
#[test]
fn k1_1041_page_maps_every_branch_onto_the_record_and_reopens_unchanged() {
    let mut full = k1_1041("Synthetic Family Trust");
    full.interest = 120_000;
    full.ordinary_dividends = 90_000;
    full.qualified_dividends = 50_000;
    full.short_gain = 10_000;
    full.long_gain = 80_000;
    full.rate_28_gain = 5_000;
    full.unrecaptured_gain = 7_000;
    full.royalties = K1EstateTrustRoyalties::Royalties {
        amount: 300_000,
        deduction: K1RoyaltyDeduction::Depreciation { amount: 40_000 },
    };
    full.business = main_estate_activities([500_000, 200_000, 0], [0, 20_000, 0], true);
    full.qbi = main_estate_qbi(600_000, 10_000);
    full.gross_farming_fishing_income = 100_000;
    full.more = K1EstateTrustOtherBoxes {
        ird_estate_tax: 3_000,
        final_year: K1FinalYear::Final {
            section_67e: 4_000,
            personal_property_tax: 0,
            real_estate_tax: 6_000,
            state_income_tax: 0,
            state_sales_tax: 0,
            short_carry: 0,
            long_carry: 9_000,
            nol: None,
        },
        amt: K1EstateTrustAmt::Components { a: 12_000, b: 2_000, c: 0, d: 10_000, e: 3_000, f: 0 },
        minimum_tax_exclusion: None,
        estimated_tax: K1EstateTrustEstimatedTax::Allocated { amount: 25_000, filed_1041t: true },
        backup_withholding: 1_500,
        tax_exempt_interest: 8_000,
        private_activity_interest: 2_000,
        niit: Some(EstateTrustNiitAdjustment { line_7: 500, modified_agi: 500 }),
        section199a_dividends: 30_000,
        unentered: vec![EstateTrustK1Item::Credits],
    };
    let (session, reply) = save_k1s(vec![full.clone()]);
    accepted(&reply);
    let (label, r) = stored_k1_1041(&session);
    assert_eq!(label, "Synthetic Family Trust");
    assert_eq!((r.royalty_income, r.box_5_royalty_only), (300_000, true));
    assert_eq!(
        (
            r.box_9_royalty_depreciation,
            r.box_9_royalty_sound_recording_only,
            r.box_9_royalty_cost_depletion_only,
            r.box_9_royalty_amortization_only
        ),
        (40_000, true, false, false)
    );
    assert_eq!((r.activities.box6, r.activities.box7), (500_000, 200_000));
    assert_eq!((r.box_10_interest_ird_estate_tax, r.box_10_interest_ird_only), (3_000, true));
    assert_eq!(
        (
            r.final_year_successor,
            r.final_67e_no_niit_allocation,
            r.final_taxes_only,
            r.final_carry_same_tax_character,
            r.final_long_carry
        ),
        (false, true, true, true, 9_000)
    );
    assert_eq!(
        (
            r.amt_box12a_standalone,
            r.amt_box12_components_confirmed,
            r.amt_box12e_included_in_d,
            r.amt_box12f_included_in_d
        ),
        (false, true, true, false)
    );
    assert_eq!((r.allocated_estimated_tax, r.form_1041t_timely), (25_000, true));
    assert_eq!(r.section199a_dividends, ReitDividends::Reported { amount: 30_000, eligible: true });
    assert_eq!(r.unentered, vec![EstateTrustK1Item::Credits]);
    let mut other = full.clone();
    other.name = "Synthetic Estate".into();
    other.royalties = K1EstateTrustRoyalties::Royalties {
        amount: 300_000,
        deduction: K1RoyaltyDeduction::Depletion { amount: 10_000 },
    };
    let mut amortized = k1_1041("Synthetic Royalty Trust");
    amortized.royalties = K1EstateTrustRoyalties::Royalties {
        amount: 50_000,
        deduction: K1RoyaltyDeduction::Amortization { amount: 5_000 },
    };
    amortized.more = k1_1041_more_with(
        K1EstateTrustAmt::CodeAOnly { a: -4_000 },
        K1EstateTrustEstimatedTax::Allocated { amount: 1_000, filed_1041t: false },
        vec![],
    );
    let mut empty_more = k1_1041("Synthetic Trust B");
    empty_more.business = main_estate_activities([0, 0, 100_000], [0; 3], false);
    let pages = vec![full, other, amortized, empty_more, k1_1041("Synthetic Trust C")];
    let (mut session, reply) = save_k1s(pages);
    accepted(&reply);
    // Normal Save allocates activity identities. Reopen and resave preserve them.
    let before = session.draft.clone();
    let reopened = reopened_k1s(&mut session);
    let items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, items);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, before);
    assert_eq!(reopened_k1s(&mut restart(before)), reopened);
}
#[test]
fn k1_1041_rules_reject_on_both_sides_at_their_boundaries() {
    type Edit = Box<dyn Fn(&mut K1EstateTrust, i64)>;
    let cases: Vec<(&str, Edit)> = vec![
        (
            "K1_1041_QUALIFIED_ABOVE_2A",
            Box::new(|p, extra| {
                p.ordinary_dividends = 100_000;
                p.qualified_dividends = 60_000;
                p.more.section199a_dividends = 40_000 + extra;
            }),
        ),
        (
            "K1_1041_4B_4C_ABOVE_4A",
            Box::new(|p, extra| {
                p.long_gain = 100_000;
                p.rate_28_gain = 30_000;
                p.unrecaptured_gain = 70_000 + extra;
            }),
        ),
        (
            "K1_1041_PRIVATE_ABOVE_14A",
            Box::new(|p, extra| {
                p.more.tax_exempt_interest = 50_000;
                p.more.private_activity_interest = 50_000 + extra;
            }),
        ),
        (
            "K1_1041_ROYALTY_DEDUCTION_ABOVE_BOX_5",
            Box::new(|p, extra| {
                p.royalties = K1EstateTrustRoyalties::Royalties {
                    amount: 20_000,
                    deduction: K1RoyaltyDeduction::Depletion { amount: 20_000 + extra },
                };
            }),
        ),
    ];
    for (key, edit) in cases {
        let mut page = k1_1041("Synthetic Trust");
        edit(&mut page, 0);
        accepted(&save_k1s(vec![page]).1);
        let mut page = k1_1041("Synthetic Trust");
        edit(&mut page, 1);
        assert_eq!(page.rule(), Some(key));
        rejected_with(save_k1s(vec![page]).1, key);
    }
    // Approved signed source design permits directly apportioned losses and
    // independent QBI; these supersede the old positive-only source caps.
    for kind in 0..3 {
        let mut page = k1_1041("Synthetic deduction boundary");
        let mut deductions = [0; 3];
        deductions[kind] = 100_001;
        page.business = main_estate_activities([100_000; 3], deductions, false);
        page.qbi = main_estate_qbi(300_001, 0);
        assert_eq!(page.rule(), None);
        accepted(&save_k1s(vec![page]).1);
    }
    let mut page = k1_1041("Synthetic duplicate box");
    page.more.unentered = vec![EstateTrustK1Item::AmtItems, EstateTrustK1Item::Credits];
    accepted(&save_k1s(vec![page.clone()]).1);
    page.more.unentered = vec![EstateTrustK1Item::AmtItems; 2];
    rejected_with(save_k1s(vec![page]).1, "K1_1041_BOX_TWICE");
}
#[test]
fn k1_1041_refusals_are_stored_and_named() {
    let mut page = k1_1041("Synthetic Trust");
    page.interest = 100_000;
    page.special = true;
    let (session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    assert!(
        estimate(&session.draft)
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_ESTATE_OTHER_SOURCE_REQUIRED")
    );
    for item in [
        EstateTrustK1Item::AmtItems,
        EstateTrustK1Item::Credits,
        EstateTrustK1Item::OtherInformation,
    ] {
        let mut page = k1_1041("Synthetic Trust");
        page.interest = 100_000;
        page.more.unentered = vec![item];
        let (session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = estimate(&session.draft);
        if item == EstateTrustK1Item::Credits {
            assert!(noted(&result, "WARN_ESTATE_UNENTERED_CREDIT_LEFT_OUT"));
        } else if let Some(key) = item.refusal_key() {
            assert!(result.issues.iter().any(|i| i.display.key == key));
        } else {
            assert!(!result.review_notes.is_empty());
        }
        assert_eq!(crate::interview::document_amount(&session.draft.income[1].income, false), None);
    }
}
#[test]
fn k1_1041_page_reproduces_the_boxes_6_to_8_fixture() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 650_000);
    let mut session = Interview::new(TEST_USER, draft.clone());
    let mut page = k1_1041("Synthetic Family Trust");
    page.interest = 100_000;
    page.business = main_estate_activities([4_000_000, 1_200_000, 300_000], [0, 200_000, 0], true);
    page.qbi = main_estate_qbi(3_800_000, 1_000_000);
    let request = save_topic(
        &draft,
        TaxSection::PassThrough,
        vec![document(0, IncomeDocument::K1EstateTrust(page))],
    );
    accepted(&send(&mut session, request));
    let r = supported(&session.draft);
    assert_eq!(value(&r, "schedule_1_rental_income"), 53_000);
    assert_eq!(return_line(&r, "11a / 11b"), 11_400_000);
    assert_eq!(value(&r, "qbi_deduction"), 7_600);
    assert_eq!(return_line(&r, "15"), 9_030_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    // A source-only preview cannot promise a post-passive-loss return amount.
    assert_eq!(page_documents(&open_topic(&mut session, TaxSection::PassThrough))[0].preview, None);
}

#[test]
fn k1_1041_multiple_successor_origins_and_personal_losses_use_one_year_pool() {
    // Derived 2026 application of 642(h)/172: W-2 60,000; standard
    // deduction16,100 =>43,900. Ordinary post2017 NOLs70,000 =>35,120
    // deduction(80%). AMTI60,000 =>54,000 ATNOLD(90%). Original2020
    // uses first; the two actual2025 fiduciary sources remain separate inputs.
    let a = estate_nol_page("Synthetic successor A", 2025, 20_000, 10_000, None);
    let b = estate_nol_page("Synthetic successor B", 2025, 10_000, 20_000, None);
    for pages in [vec![a.clone(), b.clone()], vec![b.clone(), a.clone()]] {
        let (mut session, reply) = save_k1s(pages);
        accepted(&reply);
        let before = session.draft.clone();
        accepted(&send(
            &mut session,
            save_topic(
                &before,
                TaxSection::SelfEmployment,
                vec![document(
                    0,
                    IncomeDocument::NetOperatingLoss(LossCarryforward {
                        regular: vec![
                            NolYear {
                                ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                                year: 2020,
                                balance: 4_000_000,
                                research_parts: None,
                                injured_owners: None,
                            }
                            .into(),
                        ],
                        amt: vec![AtnolYear {
                            year: 2020,
                            balance: 4_000_000,
                            special_exception: false,
                        }],
                        source: LossSource::Business,
                    }),
                )],
            ),
        ));
        let accepted_draft = session.draft.clone();
        let result = supported(&accepted_draft);
        assert_eq!(value(&result, "prior_nol_deduction"), 35_120);
        assert_eq!(value(&result, "nol_2020_used"), 35_120);
        assert_eq!(value(&result, "nol_2020_carryforward"), 4_880);
        assert_eq!(value(&result, "nol_2025_carryforward"), 30_000);
        assert_eq!(value(&result, "regular_total_nol_carryforward"), 34_880);
        assert_eq!(value(&result, "prior_atnol_deduction"), 54_000);
        assert_eq!(value(&result, "atnol_2020_used"), 40_000);
        assert_eq!(value(&result, "atnol_2025_used"), 14_000);
        assert_eq!(value(&result, "amt_total_nol_carryforward"), 16_000);
        assert_eq!(estimate(&restart(accepted_draft.clone()).draft), result);
        assert_eq!(session.draft, accepted_draft);
        let reopened = open_topic(&mut session, TaxSection::PassThrough);
        assert_eq!(page_documents(&reopened).len(), 2);
    }
}

#[test]
fn k1_1041_actual_finite_successor_period_expires_without_shifting_original_year() {
    // 26CFR1.642(h)-1(b): for calendar-year loss2007, estate2008–2026
    // consumes19 carry years; beneficiary2026 is20th(last). Loss2008 has
    // two beneficiary years left. Source statement supplies1/2 explicitly.
    // Regular absorption43,900 of100,000; AMT absorption54,000 of80,000.
    for (year, remaining, regular_carry, amt_carry) in [(2007, 1, 0, 0), (2008, 2, 56_100, 26_000)]
    {
        let mut page = estate_nol_page(
            "Synthetic finite-period fiduciary",
            year,
            100_000,
            80_000,
            Some(remaining),
        );
        estate_nol_original_period(&mut page, year * 10000 + 101, year * 10000 + 1231);
        let (session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "prior_nol_deduction"), 100_000);
        assert_eq!(value(&result, "prior_atnol_deduction"), 54_000);
        assert_eq!(value(&result, &alloc::format!("estate_trust_2_nol_{year}_used")), 43_900);
        assert_eq!(
            value(&result, &alloc::format!("estate_trust_2_nol_{year}_expired")),
            56_100 - regular_carry
        );
        assert_eq!(value(&result, "regular_total_nol_carryforward"), regular_carry);
        assert_eq!(value(&result, "amt_total_nol_carryforward"), amt_carry);
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
}

#[test]
fn k1_1041_missing_older_period_omits_only_that_loss_year_and_preserves_source() {
    let mut page = estate_nol_page("Synthetic incomplete period", 2007, 80_000, 80_000, None);
    page.interest = 100_000;
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.years.push(FinalEstateTrustNolYear {
        original_period: None,
        year: 2025,
        remaining_years: None,
        regular_balance: 1_000_000,
        alternative_balance: 1_000_000,
    });
    let (session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let original = session.draft.clone();
    let result = supported(&original);
    assert!(noted(&result, "WARN_ESTATE_NOL_PERIOD_LEFT_OUT"));
    assert_eq!(value(&result, "prior_nol_deduction"), 10_000);
    assert_eq!(value(&result, "agi"), 51_000);
    let (_, record) = stored_k1_1041(&session);
    assert_eq!(record.final_nol.unwrap().years.len(), 2);
    assert_eq!(estimate(&restart(original.clone()).draft), result);
    assert_eq!(session.draft, original);
}

#[test]
fn k1_1041_incompatible_same_original_year_periods_do_not_get_source_priority() {
    let mut a = estate_nol_page("Synthetic period A", 2008, 20_000, 20_000, Some(1));
    let mut b = estate_nol_page("Synthetic period B", 2008, 30_000, 30_000, Some(2));
    estate_nol_original_period(&mut a, 20080101, 20081231);
    estate_nol_split_year(&mut a, 2011, 2);
    estate_nol_original_period(&mut b, 20080101, 20081231);
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut a.more.final_year else { unreachable!() };
    nol.years.push(FinalEstateTrustNolYear {
        original_period: Some(FinalEstateTrustLossPeriod { began: 20250101, ended: 20251231 }),
        year: 2025,
        remaining_years: None,
        regular_balance: 1_000_000,
        alternative_balance: 1_000_000,
    });
    for pages in [vec![a.clone(), b.clone()], vec![b.clone(), a.clone()]] {
        let (session, reply) = save_k1s(pages);
        accepted(&reply);
        let result = supported(&session.draft);
        assert!(noted(&result, "WARN_ESTATE_NOL_POOL_LEFT_OUT"));
        assert_eq!(value(&result, "prior_nol_deduction"), 10_000);
        assert_eq!(value(&result, "prior_atnol_deduction"), 10_000);
    }
}

fn estate_nol_split_year(page: &mut K1EstateTrust, year: u32, parts: u32) {
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.taxable_periods.retain(|p| p.period.began / 10000 != year);
    let months = 12 / parts;
    for index in 0..parts {
        nol.taxable_periods.push(FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod {
                began: year * 10000 + (index * months + 1) * 100 + 1,
                ended: crate::dates::last_day_of_month(year, (index + 1) * months),
            },
            accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
        });
    }
}

#[test]
fn k1_1041_original_short_years_control_absorption_not_source_order_or_expiry() {
    // Derived 642(h)/172 chronology: these are different fiduciaries, so
    // intervening short years may give the later original loss an earlier
    // expiry. Earlier Jan-Mar2010 loss30,000 has three beneficiary carry years;
    // later Apr-Dec2010 loss30,000 expires now. TI43,900 uses the earlier
    // 30,000 first, then13,900; remaining16,100 expires, not carried. AMTI
    // 60,000 absorbs54,000, with the later6,000 expiring. Sorting by earliest
    // expiry would incorrectly carry16,100 regular/6,000 AMT.
    let mut earlier =
        estate_nol_page("Synthetic earlier original period", 2010, 30_000, 30_000, Some(3));
    estate_nol_original_period(&mut earlier, 20100101, 20100331);
    let mut later =
        estate_nol_page("Synthetic later original period", 2010, 30_000, 30_000, Some(1));
    estate_nol_original_period(&mut later, 20100401, 20101231);
    estate_nol_split_year(&mut later, 2011, 4);
    for pages in [vec![earlier.clone(), later.clone()], vec![later.clone(), earlier.clone()]] {
        let (session, reply) = save_k1s(pages);
        accepted(&reply);
        let original = session.draft.clone();
        let result = supported(&original);
        assert!(!noted(&result, "WARN_ESTATE_NOL_POOL_LEFT_OUT"));
        assert_eq!(value(&result, "prior_nol_deduction"), 60_000);
        assert_eq!(value(&result, "nol_2010_used"), 43_900);
        assert_eq!(value(&result, "nol_2010_expired"), 16_100);
        assert_eq!(value(&result, "regular_total_nol_carryforward"), 0);
        assert_eq!(value(&result, "atnol_2010_used"), 54_000);
        assert_eq!(value(&result, "atnol_2010_expired"), 6_000);
        assert_eq!(value(&result, "amt_total_nol_carryforward"), 0);
        assert_eq!(estimate(&restart(original.clone()).draft), result);
        assert_eq!(session.draft, original);
    }
}

#[test]
fn k1_1041_distinct_short_original_years_on_one_statement_keep_their_periods() {
    let mut page =
        estate_nol_page("Synthetic successive short years", 2010, 30_000, 30_000, Some(1));
    estate_nol_original_period(&mut page, 20100101, 20100331);
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.years.push(FinalEstateTrustNolYear {
        year: 2010,
        original_period: Some(FinalEstateTrustLossPeriod { began: 20100401, ended: 20101231 }),
        remaining_years: Some(2),
        regular_balance: 3_000_000,
        alternative_balance: 3_000_000,
    });
    estate_nol_split_year(&mut page, 2011, 3);
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert!(!noted(&result, "WARN_ESTATE_NOL_POOL_LEFT_OUT"));
    assert_eq!(value(&result, "prior_nol_deduction"), 60_000);
    assert_eq!(value(&result, "regular_total_nol_carryforward"), 16_100);
    assert_eq!(value(&result, "amt_total_nol_carryforward"), 6_000);
    let (_, stored) = stored_k1_1041(&session);
    assert_eq!(stored.final_nol.as_ref().unwrap().years.len(), 2);
    let before = session.draft.clone();
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(bad) = &mut items[0].document else { unreachable!() };
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut bad.more.final_year else {
        unreachable!()
    };
    nol.years[1].original_period.as_mut().unwrap().began = 20070101;
    let reply = send(&mut session, save_topic(&before, TaxSection::PassThrough, items));
    rejected_with(reply, "ESTATE_NOL_PERIOD_INVALID");
    assert_eq!(session.draft, before);
    assert_eq!(estimate(&restart(before).draft), result);

    // D-2026-10-15-13: retain actual short-period identity and chronology
    // above the old 100-loss / 1,000-period ceilings. Each post-2017 loss
    // is $1: all $101 is below both current regular and AMT income limits.
    let mut page = estate_nol_page("Synthetic many short periods", 2018, 1, 1, None);
    let K1FinalYear::Final { nol: Some(history), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    retained_estate_nol_periods(history);
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let saved = session.draft.clone();
    let result = supported(&saved);
    assert_eq!(value(&result, "prior_nol_deduction"), 101);
    assert_eq!(value(&result, "prior_atnol_deduction"), 101);
    assert_eq!(value(&result, "regular_total_nol_carryforward"), 0);
    assert_eq!(value(&result, "amt_total_nol_carryforward"), 0);
    let (_, stored) = stored_k1_1041(&session);
    let history = stored.final_nol.unwrap();
    assert_eq!(history.years.len(), 101);
    assert_eq!(history.taxable_periods.len(), 1007);
    let correct = open_topic(&mut session, TaxSection::PassThrough);
    let dirty = session.dirty;
    for period_overlap in [false, true] {
        let mut bad = correct.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
        let IncomeDocument::K1EstateTrust(page) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let K1FinalYear::Final { nol: Some(history), .. } = &mut page.more.final_year else {
            unreachable!()
        };
        let (key, path) = if period_overlap {
            history.taxable_periods[1000].period.began = history.taxable_periods[999].period.ended;
            (
                "ESTATE_NOL_PERIOD_INVALID",
                "[documents/documents/0/document/more/final_year/nol/taxable_periods/1000/period/began]",
            )
        } else {
            history.years[100].original_period = history.years[99].original_period.clone();
            (
                "NOL_YEAR_TWICE",
                "[documents/documents/0/document/more/final_year/nol/years/100/year]",
            )
        };
        for request in [super::support::client::estimate_documents_from_save(&bad), bad] {
            let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
                panic!("invalid retained estate source")
            };
            assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
            assert_eq!(field.as_deref(), Some(path));
            assert_eq!(session.draft, saved);
            assert_eq!(session.dirty, dirty);
        }
    }
    assert!(matches!(
        send(&mut session, super::support::client::estimate_documents_from_save(&correct)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, correct));
    let reopened = super::support::client::save_reopen_native_sources(
        session.draft,
        &[TaxSection::PassThrough],
        &[TaxSection::PassThrough],
    );
    assert_eq!(reopened, saved);
    assert_eq!(supported(&reopened), result);
}

#[test]
fn k1_1041_debt_attribute_reduction_keeps_original_period_order_before_year_totals() {
    // Derived 108(b)(4)(B)/172 endpoint: earlier original loss80,000
    // carries36,100 regular/26,000 AMT; later30,000 expires. Excluded
    // debt10,000 reduces the earlier surviving loss first, not the later
    // expiring balance. Annual display aggregation occurs after reduction.
    let mut earlier =
        estate_nol_page("Synthetic earlier surviving period", 2010, 80_000, 80_000, Some(3));
    estate_nol_original_period(&mut earlier, 20100101, 20100331);
    let mut later =
        estate_nol_page("Synthetic later expiring period", 2010, 30_000, 30_000, Some(1));
    estate_nol_original_period(&mut later, 20100401, 20101231);
    estate_nol_split_year(&mut later, 2011, 4);
    let (mut session, reply) = save_k1s(vec![later, earlier]);
    accepted(&reply);
    let before = session.draft.clone();
    let cancellation = car_1099c(
        1_000_000,
        CanceledDebtChoice::Insolvent {
            liabilities: 1_000_000,
            assets: 0,
            basis_and_cash_after: 0,
            amt_basis_and_cash_after: None,
            business_debt: false,
            property: vec![],
            depreciable_first: 0,
        },
    );
    let request = save_topic(&before, TaxSection::CanceledDebt, vec![document(0, cancellation)]);
    accepted(&send(&mut session, request));
    let original = session.draft.clone();
    let result = supported(&original);
    assert_eq!(value(&result, "nol_2010_debt_reduction"), 10_000);
    assert_eq!(value(&result, "nol_2010_expired"), 30_000);
    assert_eq!(value(&result, "regular_total_nol_carryforward"), 26_100);
    assert_eq!(value(&result, "atnol_2010_debt_reduction"), 10_000);
    assert_eq!(value(&result, "atnol_2010_expired"), 30_000);
    assert_eq!(value(&result, "amt_total_nol_carryforward"), 16_000);
    assert_eq!(estimate(&restart(original.clone()).draft), result);
    assert_eq!(session.draft, original);
}

#[test]
fn k1_1041_original_period_is_bounded_by_actual_fiduciary_termination() {
    let mut valid = estate_nol_page("Synthetic actual termination", 2026, 10_000, 10_000, None);
    estate_nol_original_period(&mut valid, 20260101, 20260930);
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut valid.more.final_year else {
        unreachable!()
    };
    nol.terminated_on = Some(20260930);
    nol.taxable_periods.truncate(1);
    let (mut session, reply) = save_k1s(vec![valid]);
    accepted(&reply);
    let before = session.draft.clone();
    let result = supported(&before);
    assert_eq!(value(&result, "prior_nol_deduction"), 10_000);
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    // Calendar-year end would incorrectly admit this loss period.
    nol.years[0].original_period.as_mut().unwrap().ended = 20261001;
    let request = save_topic(&before, TaxSection::PassThrough, items);
    rejected_with(send(&mut session, request), "ESTATE_NOL_PERIOD_INVALID");
    assert_eq!(session.draft, before);
    assert_eq!(estimate(&restart(before).draft), result);
}

#[test]
fn k1_1041_royalty_gross_and_apportioned_deductions_foot_part_three() {
    // Derived from 2026 draft f1041sk1 printed p.2 reporting table and
    // f1040se printed p.2: box 5 belongs in 33(f), box 9 in 33(e).
    // Passive income 1,200 allows 1,200 of 1,500 activity deductions;
    // royalty gross 1,000 less 200 stays separate. Part III = 800.
    for deduction in [
        crate::K1RoyaltyDeduction::Depreciation { amount: 20_000 },
        crate::K1RoyaltyDeduction::Depletion { amount: 20_000 },
        crate::K1RoyaltyDeduction::Amortization { amount: 20_000 },
    ] {
        let mut page = k1_1041("Synthetic royalty and passive activity source");
        page.royalties = K1EstateTrustRoyalties::Royalties { amount: 100_000, deduction };
        let mut loss =
            estate_activity_page("Passive loss", crate::EstateActivityKind::Business, 100_000);
        loss.depreciation = 150_000;
        page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
            box6: 120_000,
            box7: 0,
            box8: 0,
            activities: vec![
                loss,
                estate_activity_page("Passive profit", crate::EstateActivityKind::Business, 20_000),
            ],
        });
        let (session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = supported(&session.draft);
        for (id, expected, printed) in [
            ("estate_trust_2_line33f_royalties", 1_000, "33f"),
            ("estate_trust_2_line33e_royalty_deductions", 200, "33e"),
            ("schedule_e_line34a_passive_income", 1_200, "34a(d)"),
            ("schedule_e_line34a_other_income", 1_000, "34a(f)"),
            ("schedule_e_line34b_passive_losses", 1_200, "34b(c)"),
            ("schedule_e_line34b_other_losses", 200, "34b(e)"),
            ("schedule_e_line35_estate_income", 2_200, "35"),
            ("schedule_e_line36_estate_losses", -1_400, "36"),
            ("schedule_e_line37_estate_net", 800, "37"),
        ] {
            assert_eq!(value(&result, id), expected, "{id}");
            assert_eq!(
                result.lines.iter().find(|line| line.id == id).unwrap().form_line,
                Some(crate::FormLine::new(crate::TaxForm::ScheduleE, printed))
            );
        }
        assert_eq!(value(&result, "estate_trust_2_activity_3_suspended"), 300);
        assert_eq!(value(&result, "schedule_1_rental_income"), 800);
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
}

#[test]
fn k1_1041_credit_letters_use_estate_destinations_and_actual_activity_links() {
    // 2026 i1041sk1 printed pp.3-4 and f1041sk1 p.2 table: C is
    // low-income housing, F work opportunity, G small-employer health.
    // These are estate code identities, not the shareholder code table.
    for (code, key, housing) in [
        (crate::EstateCreditCode::C, "low_income_housing", true),
        (crate::EstateCreditCode::F, "work_opportunity", false),
        (crate::EstateCreditCode::G, "small_employer_health", false),
    ] {
        let mut page = k1_1041("Synthetic estate allocated credit");
        let mut activity = estate_activity_page(
            "Credit source",
            if housing {
                crate::EstateActivityKind::RentalRealEstate
            } else {
                crate::EstateActivityKind::Business
            },
            6_000_000,
        );
        if !housing {
            activity.participation = crate::EstateParticipation::Material(ScorpParticipation {
                minutes: 30_060,
                ..participation_facts()
            });
        }
        page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
            box6: if housing { 0 } else { 6_000_000 },
            box7: if housing { 6_000_000 } else { 0 },
            box8: 0,
            activities: vec![activity],
        });
        let (mut session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(page) = &mut rows[0].document else { unreachable!() };
        let K1EstateTrustBusiness::Amounts(source) = &page.business else { unreachable!() };
        page.reported_credits = vec![crate::EstateReportedCreditPage {
            code: code.clone(),
            amount: 100_000,
            activities: vec![source.activities[0].id],
        }];
        let before = session.draft.clone();
        accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, rows)));
        let result = supported(&session.draft);
        let suffix = "";
        assert_eq!(
            value(&result, &alloc::format!("business_credit_{key}_current_year{suffix}_used")),
            1_000
        );
        let source_suffix = if housing { "_passive" } else { "" };
        assert_eq!(
            printed_line(
                &result,
                &alloc::format!("business_credit_{key}_current_year{source_suffix}")
            ),
            (
                crate::TaxForm::Form3800,
                match code {
                    crate::EstateCreditCode::C => "4d",
                    crate::EstateCreditCode::F => "4b",
                    crate::EstateCreditCode::G => "4h",
                    _ => unreachable!(),
                }
                .into()
            )
        );
        assert_eq!(value(&result, "general_business_credit"), 1_000);
        let accepted_draft = session.draft.clone();
        let mut restarted = restart(accepted_draft.clone());
        let rows = page_documents(&open_topic(&mut restarted, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(page) = &rows[0].document else { unreachable!() };
        assert_eq!(page.reported_credits[0].code, code);
        assert_eq!(page.reported_credits[0].activities, vec![3]);
        assert_eq!(estimate(&restarted.draft), result);
        let mut forged = rows;
        let IncomeDocument::K1EstateTrust(page) = &mut forged[0].document else { unreachable!() };
        page.reported_credits[0].activities = vec![999999];
        rejected_with(
            send(&mut restarted, save_topic(&accepted_draft, TaxSection::PassThrough, forged)),
            "ESTATE_ACTIVITY_LINK_INVALID",
        );
        assert_eq!(restarted.draft, accepted_draft);
    }
}

#[test]
fn k1_1041_foreign_statement_election_survives_other_income_topic_save() {
    // Derived section904(j) qualified-payee endpoint: actual tax200 is
    // below the single-filer300 limit, after the return-owned Yes election.
    let mut page = k1_1041("Synthetic foreign-tax qualified payee statement");
    page.interest = 100_000;
    page.foreign_statement = Some(crate::EstateForeignStatement {
        tax: 20_000,
        treatment: crate::EstateForeignTreatment::QualifiedPassivePayeeStatement,
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    assert_eq!(crate::extended_rules::payee_statement_taxes(&session.draft), 20_000);
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            credit_income_sources: vec![],
            credits: vec![MoreCredit::ForeignTax(ForeignTaxPage::Statements { eligible: true })],
        },
    );
    accepted(&send(&mut session, request));
    assert_eq!(value(&supported(&session.draft), "foreign_tax_credit"), 200);
    let request = open_topic(&mut session, TaxSection::InterestDividends);
    accepted(&send(&mut session, request));
    assert_eq!(
        session.draft.credits.as_ref().unwrap().foreign_tax,
        ForeignTaxClaim::Statements { eligible: Some(true) }
    );
    assert_eq!(value(&supported(&session.draft), "foreign_tax_credit"), 200);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), estimate(&session.draft));
}

#[test]
fn k1_royalty_pages_save_reopen_restart_and_correct_the_same_document() {
    let mut partnership = k1_1065("Synthetic royalty partnership");
    partnership.more.royalties = 800_000;
    partnership.more.deductions = vec![crate::PartnerDeduction {
        code: crate::PartnerDeductionCode::RoyaltyDeductions,
        amount: 120_000,
    }];
    partnership.qbi = PartnerQbi::NoStatement;
    let original = complete(FederalStatus::Single);
    let (mut session, reply) = save_1065s(&original, vec![partnership.clone()]);
    accepted(&reply);
    let stored = &stored_1065s(&session)[0];
    assert_eq!(stored.separate.as_ref().unwrap().royalties, 800_000);
    assert_eq!(K1Partnership::from_record(&partnership.name, stored), Some(partnership.clone()));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "schedule_e_rental_royalty_total"), 6_800);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    let mut corrected = partnership.clone();
    corrected.more.royalties = 900_000;
    let mut source = stored.clone();
    source.separate.as_mut().unwrap().royalties = 900_000;
    assert_eq!(corrected.record(), source);
    assert_eq!(K1Partnership::from_record(&corrected.name, &source), Some(corrected.clone()));

    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .id;
    let revision = session.draft.revision;
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Partnership(corrected))],
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, revision + 1);
    assert_eq!(session.draft.income.iter().filter(|e| e.id == id).count(), 1);
    assert_eq!(stored_1065s(&session)[0].separate.as_ref().unwrap().royalties, 900_000);
    assert_eq!(value(&supported(&session.draft), "schedule_e_rental_royalty_total"), 7_800);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), supported(&session.draft));

    let mut shareholder = k1_1120s("Synthetic royalty corporation");
    shareholder.royalties = 800_000;
    shareholder.qbi = K1ScorpQbi::NoStatement;
    shareholder.more.deductions = vec![crate::ScorpDeduction {
        code: crate::ScorpDeductionCode::RoyaltyDeductions,
        amount: 120_000,
    }];
    let (mut session, reply) = save_k1_1120s(vec![shareholder.clone()]);
    accepted(&reply);
    let stored = stored_k1_1120s(&session);
    assert_eq!(stored.royalties, 800_000);
    assert_eq!(K1Scorp::from_record(&shareholder.name, &stored), Some(shareholder.clone()));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "schedule_e_rental_royalty_total"), 6_800);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    shareholder.royalties = 900_000;
    let mut source = stored;
    source.royalties = 900_000;
    assert_eq!(shareholder.record(), source);
    assert_eq!(K1Scorp::from_record(&shareholder.name, &source), Some(shareholder.clone()));
    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Scorp { .. }))
        .unwrap()
        .id;
    let revision = session.draft.revision;
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Scorp(shareholder))],
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, revision + 1);
    assert_eq!(session.draft.income.iter().filter(|e| e.id == id).count(), 1);
    assert_eq!(stored_k1_1120s(&session).royalties, 900_000);
    assert_eq!(value(&supported(&session.draft), "schedule_e_rental_royalty_total"), 7_800);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), supported(&session.draft));
}

#[test]
fn k1_1041_signed_independent_amt_lines_do_not_duplicate_code_a() {
    // Derived 2026 draft beneficiary source/2025 final i6251 printed p.8:
    // G pre1987 depreciation is line3; H depletion is2d. A includes both.
    let mut page = k1_1041("Synthetic signed independent AMT statement");
    page.interest = 100_000;
    page.more.amt = K1EstateTrustAmt::CodeAOnly { a: 5_000 };
    page.amt_reconciliation = Some(crate::EstateAmtReconciliationPage {
        other_in_a: 5_000,
        adjustments: vec![
            crate::EstateAmtAdjustmentPage {
                code: crate::EstateAmtCode::DepreciationG,
                amount: 10_000,
                treatment: crate::EstateAmtTreatmentPage::Independent {
                    line: crate::EstateAmtLine::Pre1987Depreciation3,
                    included_in_a: true,
                },
            },
            crate::EstateAmtAdjustmentPage {
                code: crate::EstateAmtCode::DepletionH,
                amount: -5_000,
                treatment: crate::EstateAmtTreatmentPage::Independent {
                    line: crate::EstateAmtLine::Depletion2d,
                    included_in_a: true,
                },
            },
        ],
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "estate_trust_2_amt_independent_0"), 100);
    assert_eq!(value(&result, "estate_trust_2_amt_independent_1"), -50);
    assert_eq!(value(&result, "alternative_minimum_taxable_income"), 61_050);
    let before = session.draft.clone();
    let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut rows[0].document else { unreachable!() };
    page.amt_reconciliation.as_mut().unwrap().adjustments[0].code =
        crate::EstateAmtCode::AmortizationI;
    rejected_with(
        send(&mut session, save_topic(&before, TaxSection::PassThrough, rows)),
        "ESTATE_AMT_LINE_INVALID",
    );
    assert_eq!(session.draft, before);
    assert_eq!(estimate(&restart(before).draft), result);
}

#[test]
fn k1_1041_unentered_box14_benefits_preserve_income_and_restart() {
    // 2026 draft i1041sk1 printed pp.3-5 and 09/2017 i8873:
    // investment/foreign credits, Form4952 deduction and an ETI exclusion
    // are benefits. Their absence does not erase the separate box1 income.
    let mut ordinary = k1_1041("Synthetic unentered benefit statement");
    ordinary.interest = 100_000;
    let (baseline, reply) = save_k1s(vec![ordinary.clone()]);
    accepted(&reply);
    let baseline_result = supported(&baseline.draft);
    for (item, key) in [
        (EstateTrustK1Item::ForeignTaxes, "WARN_ESTATE_BOX14_B_LEFT_OUT"),
        (EstateTrustK1Item::Rehabilitation, "WARN_ESTATE_BOX14_C_LEFT_OUT"),
        (EstateTrustK1Item::EnergyProperty, "WARN_ESTATE_BOX14_D_LEFT_OUT"),
        (EstateTrustK1Item::InvestmentIncome, "WARN_ESTATE_BOX14_E_LEFT_OUT"),
        (EstateTrustK1Item::ForeignTradingGrossReceipts, "WARN_ESTATE_BOX14_G_LEFT_OUT"),
        (EstateTrustK1Item::AdvancedCoalGasification, "WARN_ESTATE_BOX14_J_LEFT_OUT"),
        (EstateTrustK1Item::AdvancedEnergy, "WARN_ESTATE_BOX14_K_LEFT_OUT"),
        (EstateTrustK1Item::AdvancedManufacturing, "WARN_ESTATE_BOX14_L_LEFT_OUT"),
        (EstateTrustK1Item::CleanElectricity, "WARN_ESTATE_BOX14_M_LEFT_OUT"),
    ] {
        let mut page = ordinary.clone();
        page.more.unentered = vec![item];
        let (mut session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = supported(&session.draft);
        assert_eq!(return_line(&result, "2b"), 100_000);
        assert_eq!(result.tax, baseline_result.tax);
        assert!(noted(&result, key), "{item:?}");
        let before = session.draft.clone();
        let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(reopened) = &mut rows[0].document else { unreachable!() };
        assert_eq!(reopened.more.unentered, vec![item]);
        let request = save_topic(&before, TaxSection::PassThrough, rows.clone());
        accepted(&send(&mut session, request));
        assert_eq!(session.draft, before);
        assert_eq!(estimate(&restart(before.clone()).draft), result);
        let IncomeDocument::K1EstateTrust(reopened) = &mut rows[0].document else { unreachable!() };
        reopened.more.unentered.push(item);
        rejected_with(
            send(&mut session, save_topic(&before, TaxSection::PassThrough, rows)),
            "K1_1041_BOX_TWICE",
        );
        assert_eq!(session.draft, before);
    }
}

#[test]
fn k1_1041_unentered_added_tax_and_unclassified_information_stay_incomplete() {
    for (item, key) in [
        (EstateTrustK1Item::AmtItems, "ISSUE_ESTATE_K1_AMT_ITEMS"),
        (EstateTrustK1Item::CreditRecapture, "ISSUE_ESTATE_K1_CREDIT_RECAPTURE"),
        (EstateTrustK1Item::OtherInformation, "ISSUE_ESTATE_K1_OTHER_INFORMATION"),
    ] {
        let mut page = k1_1041("Synthetic unresolved mandatory statement");
        page.interest = 100_000;
        page.more.unentered = vec![item];
        let (session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = estimate(&session.draft);
        // The ordinary source stays accepted. A mandatory unknown has no
        // complete estimate or printed return rows by the existing contract.
        assert_eq!(stored_k1_1041(&session).1.interest, 100_000);
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(!result.issues.is_empty());
        assert!(result.issues.iter().any(|issue| issue.display.key == key), "{result:?}");
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
}

#[test]
fn k1_benefit_notices_save_reopen_restart_and_correct_without_losing_document_facts() {
    let mut partnership = k1_1065("Synthetic benefit partnership");
    partnership.more.royalties = 800_000;
    partnership.more.deductions = vec![crate::PartnerDeduction {
        code: crate::PartnerDeductionCode::RoyaltyDeductions,
        amount: 120_000,
    }];
    partnership.more.unentered = vec![PartnerK1Item::Credits, PartnerK1Item::Depletion];
    let (mut session, reply) =
        save_1065s(&complete(FederalStatus::Single), vec![partnership.clone()]);
    accepted(&reply);
    let accepted_draft = session.draft.clone();
    let result = supported(&session.draft);
    assert_eq!(value(&result, "schedule_e_rental_royalty_total"), 6_800);
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(
        K1Partnership::from_record(&partnership.name, &stored_1065s(&session)[0]),
        Some(partnership.clone())
    );
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .id;
    let revision = session.draft.revision;
    partnership.more.unentered.clear();
    partnership.more.royalties = 900_000;
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Partnership(partnership))],
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, revision + 1);
    assert_eq!(session.draft.income.iter().filter(|e| e.id == id).count(), 1);
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "schedule_e_rental_royalty_total"), 7_800);
    assert!(!noted(&corrected, "WARN_K1_ITEM_LEFT_OUT"));
    assert_eq!(estimate(&restart(session.draft.clone()).draft), corrected);

    let mut shareholder = k1_1120s("Synthetic benefit corporation");
    shareholder.royalties = 800_000;
    shareholder.more.deductions = vec![crate::ScorpDeduction {
        code: crate::ScorpDeductionCode::RoyaltyDeductions,
        amount: 120_000,
    }];
    shareholder.more.unentered = vec![ScorpK1Item::Credits, ScorpK1Item::Depletion];
    let (mut session, reply) = save_k1_1120s(vec![shareholder.clone()]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "schedule_e_rental_royalty_total"), 6_800);
    assert_eq!(
        K1Scorp::from_record(&shareholder.name, &stored_k1_1120s(&session)),
        Some(shareholder.clone())
    );
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Scorp { .. }))
        .unwrap()
        .id;
    shareholder.more.unentered.clear();
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Scorp(shareholder))],
    );
    accepted(&send(&mut session, request));
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "schedule_e_rental_royalty_total"), 6_800);
    assert!(!noted(&corrected, "WARN_K1_ITEM_LEFT_OUT"));
    assert_eq!(estimate(&restart(session.draft.clone()).draft), corrected);
}

#[test]
fn k1_partner_property_pages_save_reopen_restart_and_correct_the_same_document() {
    let mut page = k1_1065("Synthetic property partnership");
    page.interest = PartnerInterest::LlcMember {
        box1_in_se: false,
        participation: Participation::Over500Hours { hours: 501 },
    };
    page.ordinary = 1_000_000;
    page.more.section1231 = 1_200_000;
    page.more.unrecaptured_business = 300_000;
    page.basis.contributions = 10_000_000;
    page.qbi = PartnerQbi::Statement {
        income: 1_000_000,
        wages: 5_000_000,
        property: 0,
        service: false,
        section1231_qbi: 1_200_000,
        reit_dividends: 0,
        ptp_income: 0,
    };
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2024, 5_000, 5_000)], &[(2024, 5_000, 5_000)]));
    let (mut session, reply) = save_1065s(&d, vec![page.clone()]);
    accepted(&reply);
    let accepted_draft = session.draft.clone();
    let source = &stored_1065s(&session)[0];
    assert_eq!(source.regular.section1231.as_ref().unwrap().net, 1_200_000);
    assert_eq!(source.alternative.as_ref().unwrap(), &source.regular);
    assert_eq!(K1Partnership::from_record(&page.name, source), Some(page.clone()));
    let r = supported(&session.draft);
    assert_eq!(value(&r, "form4797_line8"), 12_000);
    assert_eq!(value(&r, "form4797_line13"), 5_000);
    assert_eq!(value(&r, "qbi_deduction"), 3_000);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), r);
    assert_eq!(session.draft, accepted_draft);
    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .id;
    let revision = session.draft.revision;
    let mut bad = page.clone();
    let PartnerQbi::Statement { section1231_qbi, .. } = &mut bad.qbi else { panic!() };
    *section1231_qbi = 1_500_000;
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Partnership(bad))],
    );
    let reply = send(&mut session, request);
    rejected_with(reply, "K1_SECTION1231_QBI_ABOVE_BOX");
    assert_eq!(session.draft, accepted_draft);
    page.more.section1231 = -400_000;
    page.more.unrecaptured_business = 0;
    let PartnerQbi::Statement { section1231_qbi, .. } = &mut page.qbi else { panic!() };
    *section1231_qbi = -400_000;
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Partnership(page.clone()))],
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, revision + 1);
    assert_eq!(session.draft.income.iter().filter(|e| e.id == id).count(), 1);
    let source = &stored_1065s(&session)[0];
    assert_eq!(K1Partnership::from_record(&page.name, source), Some(page));
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "form4797_line12"), -4_000);
    assert_eq!(value(&corrected, "qbi_deduction"), 1_200);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), corrected);
}

#[test]
fn k1_rental_property_pages_save_reopen_restart_reject_duplicates_and_correct() {
    for scorp in [false, true] {
        let (mut d, _) = crate::tests::passthrough::k1_rental::rental_property_sale(true, scorp);
        let source = d.income.pop().unwrap();
        let page = match &source.income {
            FederalIncome::Partnership { data } => IncomeDocument::K1Partnership(
                K1Partnership::from_record("Synthetic rental partnership", data).unwrap(),
            ),
            FederalIncome::Scorp { data } => IncomeDocument::K1Scorp(
                K1Scorp::from_record("Synthetic rental corporation", data).unwrap(),
            ),
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, d);
        let request =
            save_topic(&session.draft, TaxSection::PassThrough, vec![document(0, page.clone())]);
        accepted(&send(&mut session, request));
        assert_eq!(reopened_k1s(&mut session), vec![page.clone()]);
        let accepted_draft = session.draft.clone();
        let r = supported(&accepted_draft);
        assert_eq!(return_line(&r, "11a / 11b"), 6_500_000);
        assert_eq!(estimate(&restart(accepted_draft.clone()).draft), r);
        let id = session.draft.income.last().unwrap().id;
        let mut bad = page.clone();
        let rentals = match &mut bad {
            IncomeDocument::K1Partnership(page) => &mut page.rentals,
            IncomeDocument::K1Scorp(page) => &mut page.rentals,
            _ => unreachable!(),
        };
        rentals.disposed.push(K1RentalActivity::RealEstate);
        let request = save_topic(&session.draft, TaxSection::PassThrough, vec![document(id, bad)]);
        rejected_with(send(&mut session, request), "K1_RENTAL_DISPOSED_TWICE");
        assert_eq!(session.draft, accepted_draft);
        let mut corrected = page;
        let rentals = match &mut corrected {
            IncomeDocument::K1Partnership(page) => &mut page.rentals,
            IncomeDocument::K1Scorp(page) => &mut page.rentals,
            _ => unreachable!(),
        };
        rentals.disposed.clear();
        let request = save_topic(
            &session.draft,
            TaxSection::PassThrough,
            vec![document(id, corrected.clone())],
        );
        accepted(&send(&mut session, request));
        assert_eq!(session.draft.revision, accepted_draft.revision + 1);
        assert_eq!(session.draft.income.iter().filter(|entry| entry.id == id).count(), 1);
        assert_eq!(reopened_k1s(&mut session), vec![corrected]);
        let result = supported(&session.draft);
        assert_eq!(return_line(&result, "11a / 11b"), 7_000_000);
        assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    }
}

#[test]
fn k1_1041_unentered_investment_income_omits_only_dependent_deduction() {
    let mut page = k1_1041("Synthetic missing Form4952 source");
    page.interest = 100_000;
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let request = save_deductions(&session.draft, DeductionRoute::Itemize, 0);
    accepted(&send(&mut session, request));
    let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value: expenses, .. } = &mut request else { unreachable!() };
    expenses.investment_interest = Some(estate_investment_loan(50_000));
    accepted(&send(&mut session, request));
    let baseline = supported(&session.draft);
    assert_eq!(value(&baseline, "investment_interest_deduction"), 500);
    let before = session.draft.clone();
    let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut rows[0].document else { unreachable!() };
    page.more.unentered = vec![EstateTrustK1Item::InvestmentIncome];
    accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, rows)));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "2b"), 100_000);
    assert!(
        !result
            .lines
            .iter()
            .any(|line| line.id == "investment_interest_deduction" && line.amount != 0)
    );
    assert!(noted(&result, "WARN_ESTATE_INVESTMENT_INTEREST_LEFT_OUT"));
    // Working-copy omission must preserve the actual accepted expense for
    // correction when the missing fiduciary statement is supplied later.
    assert_eq!(
        session.draft.deductions.as_ref().unwrap().investment_interest().unwrap().paid,
        50_000
    );
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
}

#[test]
fn k1_1041_final_67e_niit_deduction_uses_part_two_unless_already_in_h() {
    // Current Reg.1.1411-4(g)(4)(iii) and final2025 i8960 Line10;
    // historical2017 i8960 printedp17 explicitly identifies642(h)(2).
    // Derived2026: 200,000 investment interest +60,000 wages -8,250
    // AGI deduction. Only1,250 is properly allocable to NII. A separate
    // deduction leaves grossLine8 unchanged; actual14H belongs on7.
    for in_h in [false, true] {
        let mut page = k1_1041("Synthetic final NII deduction source");
        page.interest = 20_000_000;
        page.more.final_year = estate_final_amounts(825_000);
        page.final_character = Some(estate_final_source(125_000, in_h));
        if in_h {
            page.more.niit = Some(EstateTrustNiitAdjustment { line_7: -125_000, modified_agi: 0 });
        }
        let (mut session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 251_750);
        assert_eq!(value(&result, "estate_trust_67e_deduction"), 8_250);
        assert_eq!(value(&result, "form_8960_other_modifications"), if in_h { -1_250 } else { 0 });
        assert_eq!(
            value(&result, "form_8960_investment_income"),
            if in_h { 198_750 } else { 200_000 }
        );
        assert_eq!(
            value(&result, "form_8960_additional_modifications"),
            if in_h { 0 } else { 1_250 }
        );
        assert_eq!(value(&result, "form_8960_net_investment_income"), 198_750);
        assert_eq!(value(&result, "net_investment_income_tax"), 1_967);
        let before = session.draft.clone();
        let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, rows)));
        assert_eq!(session.draft, before);
        assert_eq!(estimate(&restart(before).draft), result);
    }
}

#[test]
fn k1_1041_ordinary_investment_source_replaces_regular_and_amt_subtotal_once() {
    // Derived from Form4952 line4a: fiduciary ordinary NII500/AMT250
    // replaces, not adds to, box1 interest1000. Borrowing expense800
    // gives allowed500/250 and carry300/550 in the existing owner.
    let mut page = k1_1041("Synthetic ordinary Form4952 statement");
    page.interest = 100_000;
    page.investment_income = Some(crate::EstateInvestmentIncome {
        regular: 50_000,
        alternative: 25_000,
        character: crate::EstateInvestmentCharacter::Ordinary,
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let request = save_deductions(&session.draft, DeductionRoute::Itemize, 0);
    accepted(&send(&mut session, request));
    let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value: expenses, .. } = &mut request else { unreachable!() };
    expenses.investment_interest = Some(estate_investment_loan(80_000));
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "2b"), 100_000);
    assert_eq!(value(&result, "agi"), 61_000);
    assert_eq!(value(&result, "investment_interest_deduction"), 500);
    assert_eq!(value(&result, "amt_investment_interest_deduction"), 250);
    assert_eq!(value(&result, "investment_interest_carryforward"), 300);
    assert_eq!(value(&result, "amt_investment_interest_carryforward"), 550);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
}

#[test]
fn k1_1041_unknown_final_capital_niit_character_leaves_only_its_investment_tax_benefit_out() {
    // Derived2026, i8960 Net Gains Worksheet2(h)-(j), Reg1.642(h)-1(b).
    // $300,000 wages +$10,000 interest -$3,000 regular capital loss
    // gives AGI307,000. Without the fiduciary's NIIT carry balance, omit
    // only the investment-tax loss benefit: NII10,000, NIIT380. The
    // regular/AMT carryover is still5000 and must not be thrown away.
    let mut page = k1_1041("Synthetic final capital character missing");
    page.interest = 1_000_000;
    page.more.final_year = estate_final_amounts(0);
    let K1FinalYear::Final { short_carry, .. } = &mut page.more.final_year else { unreachable!() };
    *short_carry = 500_000;
    let mut finals = estate_final_source(0, false);
    finals.capital.alternative_short = 500_000;
    page.final_character = Some(finals);
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let before = session.draft.clone();
    let mut wages = w2(30_000_000);
    // Actual synthetic source is exempt from Social Security withholding.
    wages.social_wages = 0;
    wages.social_tax = 0;
    accepted(&send(
        &mut session,
        save_topic(&before, TaxSection::Wages, vec![document(1, IncomeDocument::W2(wages))]),
    ));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 307_000);
    assert_eq!(return_line(&result, "7a"), -300_000);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 10_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 380);
    assert!(noted(&result, "WARN_ESTATE_CAPITAL_NIIT_LEFT_OUT"));
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    assert_eq!(stored_k1_1041(&session).1.final_short_carry, 500_000);
}

#[test]
fn k1_1041_final_capital_niit_difference_uses_the_existing_net_gains_worksheet_once() {
    // Source-derived2026 worksheet application: regular carry5000/current
    // deduction3000 leaves2000. With excluded differences0/2500/5000,
    // current additions are0/500/3000 and next differences0/2000/2000.
    for (difference, current, next, nii, tax) in [
        (0, 0, 0, 7_000, 266),
        (250_000, 500, 2_000, 7_500, 285),
        (500_000, 3_000, 2_000, 10_000, 380),
    ] {
        let mut page = k1_1041("Synthetic retained fiduciary capital NIIT worksheet");
        page.interest = 1_000_000;
        page.more.final_year = estate_final_amounts(0);
        let K1FinalYear::Final { short_carry, .. } = &mut page.more.final_year else {
            unreachable!()
        };
        *short_carry = 500_000;
        let mut finals = estate_final_source(0, false);
        finals.capital.alternative_short = 500_000;
        finals.capital.niit_difference = Some(difference);
        page.final_character = Some(finals);
        let (mut session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let before = session.draft.clone();
        let mut wages = w2(30_000_000);
        wages.social_wages = 0;
        wages.social_tax = 0;
        accepted(&send(
            &mut session,
            save_topic(&before, TaxSection::Wages, vec![document(1, IncomeDocument::W2(wages))]),
        ));
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 307_000);
        assert_eq!(value(&result, "form_8960_excluded_gains"), current);
        assert_eq!(value(&result, "niit_capital_carry_adjustment"), next);
        assert_eq!(value(&result, "form_8960_net_investment_income"), nii);
        assert_eq!(value(&result, "net_investment_income_tax"), tax);
        assert!(!noted(&result, "WARN_ESTATE_CAPITAL_NIIT_LEFT_OUT"));
        let before = session.draft.clone();
        let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, rows.clone())));
        assert_eq!(session.draft, before);
        assert_eq!(estimate(&restart(before.clone()).draft), result);
        let mut bad = rows;
        let IncomeDocument::K1EstateTrust(page) = &mut bad[0].document else { unreachable!() };
        page.final_character.as_mut().unwrap().capital.niit_difference = Some(500_001);
        rejected_with(
            send(&mut session, save_topic(&before, TaxSection::PassThrough, bad)),
            "ESTATE_FINAL_CHARACTER_INVALID",
        );
        assert_eq!(session.draft, before);
    }
}

#[test]
fn k1_1041_fiduciary_period_count_rejects_impossible_remaining_years_without_mutation() {
    // The superseded synthetic oracle claimed two beneficiary years after
    // Jan–Mar2008. Actual Apr–Dec2008 plus annual2009–2026 already consumes
    // nineteen carry years, so only beneficiary2026 (one) remains.
    let mut page = estate_nol_page("Synthetic actual chain", 2008, 80_000, 80_000, Some(1));
    estate_nol_original_period(&mut page, 20080101, 20080331);
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let before = session.draft.clone();
    let expected = supported(&before);
    assert_eq!(value(&expected, "regular_total_nol_carryforward"), 0);
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.years[0].remaining_years = Some(2);
    rejected_with(
        send(&mut session, save_topic(&before, TaxSection::PassThrough, items)),
        "ESTATE_NOL_PERIOD_INVALID",
    );
    assert_eq!(session.draft, before);
    assert_eq!(estimate(&restart(before).draft), expected);
}

#[test]
fn k1_1041_52_53_week_effective_year_controls_pre2018_character_and_count() {
    // Derived2026 source shape, not a published IRS dollar example.
    // Same100,000 regular/AMT loss: deemed2017 remains pre2018 (eleven
    // beneficiary years including2026 after nine actual fiduciary carry
    // years). Deemed2018 is indefinite/80%-limited regular, even though
    // its actual period starts2017-12-30. TI43,900 gives35,120 at80%.
    for (year, remaining, deduction, carry) in
        [(2016, Some(11), 100_000, 56_100), (2017, None, 35_120, 64_880)]
    {
        let mut page =
            estate_nol_page("Synthetic52/53-week source", year, 100_000, 100_000, remaining);
        estate_nol_weeks_history(&mut page, year == 2016);
        let (mut session, reply) = save_k1s(vec![page]);
        accepted(&reply);
        let before = session.draft.clone();
        let result = supported(&before);
        assert!(!noted(&result, "WARN_ESTATE_NOL_PERIOD_LEFT_OUT"));
        assert_eq!(value(&result, "prior_nol_deduction"), deduction);
        assert_eq!(value(&result, "regular_total_nol_carryforward"), carry);
        assert_eq!(value(&result, "prior_atnol_deduction"), 54_000);
        assert_eq!(value(&result, "amt_total_nol_carryforward"), 46_000);
        let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
        let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
            unreachable!()
        };
        nol.taxable_periods.reverse();
        accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, items)));
        let mut expected_after_save = result.clone();
        expected_after_save.revision += 1;
        assert_eq!(supported(&session.draft), expected_after_save);
        assert_eq!(estimate(&restart(session.draft.clone()).draft), expected_after_save);
        let accepted_draft = session.draft.clone();
        let mut bad = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(page) = &mut bad[0].document else { unreachable!() };
        let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
            unreachable!()
        };
        // The actual original year stays unchanged; an applicability year
        // must not replace the fiduciary's actual beginning year.
        nol.years[0].year = year + 1;
        rejected_with(
            send(&mut session, save_topic(&accepted_draft, TaxSection::PassThrough, bad)),
            "ESTATE_NOL_PERIOD_INVALID",
        );
        assert_eq!(session.draft, accepted_draft);
    }
}

#[test]
fn k1_1041_appended_52_53_change_stub_is_one_actual_taxable_period() {
    // Derived uniform22-year shift of Reg1.441-2(b)(3)Example2 dates:
    // May1–2,2024 is appended to normalMay3,2024–May1,2025. This is one
    // actual period, not two years; its actual inclusive length is366days.
    let mut page =
        estate_nol_page("Synthetic appended two-day change", 2024, 100_000, 100_000, None);
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.terminated_on = Some(20261231);
    nol.years[0].original_period =
        Some(FinalEstateTrustLossPeriod { began: 20240501, ended: 20250501 });
    nol.taxable_periods = vec![
        FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod { began: 20240501, ended: 20250501 },
            accounting: FinalEstateTrustAccountingYear::Weeks52Or53 {
                annual_began: 20240503,
                annual_ended: 20250501,
                reference_ended: 20250430,
                closing: FinalEstateTrustWeekClosing::Nearest,
            },
        },
        FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod { began: 20250502, ended: 20260501 },
            accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
        },
        FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod { began: 20260502, ended: 20261231 },
            accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
        },
    ];
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let before = session.draft.clone();
    let expected = supported(&before);
    assert_eq!(value(&expected, "prior_nol_deduction"), 35_120);
    let (_, record) = stored_k1_1041(&session);
    assert_eq!(record.final_nol.unwrap().taxable_periods.len(), 3);
    assert_eq!(estimate(&restart(before.clone()).draft), expected);
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    // An unappended ordinary year cannot exceed12calendar months.
    nol.taxable_periods[0].accounting = FinalEstateTrustAccountingYear::CalendarFiscalOrShort;
    rejected_with(
        send(&mut session, save_topic(&before, TaxSection::PassThrough, items)),
        "ESTATE_NOL_PERIOD_INVALID",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn k1_1041_final_short_52_53_year_keeps_actual2026_origin_and_termination() {
    // Derived from the general1.441-2(c)(1)/642(h)(1) rules: actual final
    // periodDec26–31,2026 is a shortened elected52/53 period whose normal
    // annual close isFridayDec31,2027. Applicability is deemedJan2027,
    // but source origin and first beneficiary year remain actual2026.
    let mut page =
        estate_nol_page("Synthetic final short52/53 source", 2026, 100_000, 100_000, None);
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.terminated_on = Some(20261231);
    nol.years[0].original_period =
        Some(FinalEstateTrustLossPeriod { began: 20261226, ended: 20261231 });
    nol.taxable_periods = vec![FinalEstateTrustTaxPeriod {
        period: FinalEstateTrustLossPeriod { began: 20261226, ended: 20261231 },
        accounting: FinalEstateTrustAccountingYear::Weeks52Or53 {
            annual_began: 20261226,
            annual_ended: 20271231,
            reference_ended: 20271231,
            closing: FinalEstateTrustWeekClosing::Last,
        },
    }];
    let (session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "prior_nol_deduction"), 35_120);
    assert_eq!(value(&result, "regular_total_nol_carryforward"), 64_880);
    let (_, record) = stored_k1_1041(&session);
    assert_eq!(record.final_nol.unwrap().years[0].year, 2026);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
}

#[test]
fn k1_1041_known_remaining_count_without_actual_history_leaves_only_old_loss_benefit_out() {
    let page =
        estate_nol_page("Synthetic count without actual history", 2008, 100_000, 80_000, Some(2));
    let (session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let before = session.draft.clone();
    let result = supported(&before);
    assert!(noted(&result, "WARN_ESTATE_NOL_PERIOD_LEFT_OUT"));
    assert_eq!(value(&result, "agi"), 60_000);
    assert!(
        !result
            .lines
            .iter()
            .any(|line| line.id == "prior_nol_deduction" || line.id == "prior_atnol_deduction")
    );
    let (_, record) = stored_k1_1041(&session);
    let original = record.final_nol.unwrap();
    assert_eq!(original.years[0].remaining_years, Some(2));
    assert!(original.taxable_periods.is_empty());
    assert_eq!(estimate(&restart(before.clone()).draft), result);
    assert_eq!(session.draft, before);
}

fn estate_capital_parts(short: i64, long: i64) -> crate::EstateCapitalLossParts {
    crate::EstateCapitalLossParts { short: short * 100, long: long * 100 }
}
fn estate_mixed_capital_source() -> K1EstateTrust {
    let mut page = k1_1041("Synthetic mixed capital source");
    page.more.final_year = estate_final_amounts(0);
    let K1FinalYear::Final { short_carry, .. } = &mut page.more.final_year else { unreachable!() };
    *short_carry = 800_000;
    let mut f = estate_final_source(0, false);
    f.capital.alternative_short = 500_000;
    f.capital.niit_difference = Some(0);
    f.capital.regular_investment = Some(estate_capital_parts(3000, 0));
    f.capital.alternative_investment = Some(estate_capital_parts(2000, 0));
    f.capital.regular_business = Some(estate_capital_parts(5000, 0));
    f.capital.alternative_business = Some(estate_capital_parts(1000, 0));
    page.final_character = Some(f);
    page
}

#[test]
fn k1_1041_mixed_capital_investment_parts_feed_regular_and_amt_4952_once() {
    // Derived2026 Form4952 printedp3 line4d and Reg1.642(h)-1(b).
    // Interest2000 +STgain10000 minus actual investment carry3000/2000
    // gives ordinary investment income9000/10000. Longgain6000 less
    // investment LTcarry1000/500 is excluded without an election. Interest9500
    // allows9000/9500 and carries500/0. Whole STcarry8000/5000 and
    // LTcarry4000/2000 remain on ScheduleD: total regular gain4000.
    let mut page = estate_mixed_capital_source();
    page.interest = 200_000;
    page.short_gain = 1_000_000;
    page.long_gain = 600_000;
    let K1FinalYear::Final { long_carry, .. } = &mut page.more.final_year else { unreachable!() };
    *long_carry = 400_000;
    let c = &mut page.final_character.as_mut().unwrap().capital;
    c.alternative_long = 200_000;
    c.regular_investment.as_mut().unwrap().long = 100_000;
    c.alternative_investment.as_mut().unwrap().long = 50_000;
    c.regular_business.as_mut().unwrap().long = 200_000;
    c.alternative_business.as_mut().unwrap().long = 50_000;
    let (mut s, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let request = save_deductions(&s.draft, DeductionRoute::Itemize, 0);
    accepted(&send(&mut s, request));
    let mut request = open_itemized_topic(&mut s, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value: expenses, .. } = &mut request else { unreachable!() };
    expenses.investment_interest = Some(estate_investment_loan(950_000));
    accepted(&send(&mut s, request));
    let result = supported(&s.draft);
    assert_eq!(value(&result, "agi"), 66_000);
    assert_eq!(value(&result, "investment_interest_net_gain"), 12_000);
    assert_eq!(value(&result, "investment_interest_deduction"), 9_000);
    assert_eq!(value(&result, "amt_investment_interest_deduction"), 9_500);
    assert_eq!(value(&result, "investment_interest_carryforward"), 500);
    assert_eq!(value(&result, "amt_investment_interest_carryforward"), 0);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), result);
    let before = s.draft.clone();
    let rows = page_documents(&open_topic(&mut s, TaxSection::PassThrough)).to_vec();
    accepted(&send(&mut s, save_topic(&before, TaxSection::PassThrough, rows.clone())));
    assert_eq!(s.draft, before);
    // Independent known-zero investment classification permits the full
    // deduction. Business character and NIIT difference are not rewritten.
    for (investment, allowed, amt_allowed) in [(0, 9500, 9500), (8000, 4000, 7000)] {
        let mut rows = rows.clone();
        let IncomeDocument::K1EstateTrust(p) = &mut rows[0].document else { unreachable!() };
        let c = &mut p.final_character.as_mut().unwrap().capital;
        c.regular_investment =
            Some(estate_capital_parts(investment, if investment == 0 { 0 } else { 4000 }));
        c.alternative_investment = Some(estate_capital_parts(
            if investment == 0 { 0 } else { 5000 },
            if investment == 0 { 0 } else { 2000 },
        ));
        let before = s.draft.clone();
        accepted(&send(&mut s, save_topic(&before, TaxSection::PassThrough, rows)));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "investment_interest_deduction"), allowed);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), amt_allowed);
        assert_eq!(value(&r, "agi"), 66_000);
    }
    // Missing either tax-system investment source omits only the dependent
    // joint Form4952 claim; original borrowed-interest facts remain accepted.
    for alternative in [false, true] {
        let mut rows = rows.clone();
        let IncomeDocument::K1EstateTrust(p) = &mut rows[0].document else { unreachable!() };
        let c = &mut p.final_character.as_mut().unwrap().capital;
        if alternative {
            c.alternative_investment = None;
        } else {
            c.regular_investment = None;
        }
        let before = s.draft.clone();
        accepted(&send(&mut s, save_topic(&before, TaxSection::PassThrough, rows)));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "agi"), 66_000);
        assert!(noted(&r, "WARN_ESTATE_INVESTMENT_INTEREST_LEFT_OUT"));
        assert!(!r.lines.iter().any(|l| l.id == "investment_interest_deduction" && l.amount != 0));
        assert!(s.draft.deductions.as_ref().unwrap().investment_interest().is_some());
        assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
    }
    // Each purpose/tax-system/term has a page bound and authoritative Save twin.
    let before = s.draft.clone();
    s.dirty = false;
    for field in 0..8 {
        let mut rows = rows.clone();
        let IncomeDocument::K1EstateTrust(p) = &mut rows[0].document else { unreachable!() };
        let c = &mut p.final_character.as_mut().unwrap().capital;
        let parts = match field / 2 {
            0 => c.regular_investment.as_mut(),
            1 => c.alternative_investment.as_mut(),
            2 => c.regular_business.as_mut(),
            _ => c.alternative_business.as_mut(),
        }
        .unwrap();
        if field % 2 == 0 {
            parts.short = if field / 2 == 0 || field / 2 == 2 { 800_001 } else { 500_001 };
        } else {
            parts.long = if field / 2 == 0 || field / 2 == 2 { 400_001 } else { 200_001 };
        }
        rejected_with(
            send(&mut s, save_topic(&before, TaxSection::PassThrough, rows)),
            "ESTATE_FINAL_CHARACTER_INVALID",
        );
        assert_eq!(s.draft, before);
        assert!(!s.dirty);
    }
}

#[test]
fn k1_1041_mixed_capital_business_parts_feed_current_regular_and_amt_nol_once() {
    // Derived Form172 lines2–22: wages60000, businessloss100000,
    // actual business propertygain10000, total carry8000regular/5000AMT.
    // Nonbusiness carry3000/4000 cannot offset businessgain; business
    // carry5000/1000 can. Current NOL is35000/31000, while regular AGI-38000.
    let mut p = estate_mixed_capital_source();
    let mut activity = estate_activity_page(
        "Material operating loss",
        crate::EstateActivityKind::Business,
        -10_000_000,
    );
    activity.participation = crate::EstateParticipation::Material(ScorpParticipation {
        minutes: 30_060,
        ..participation_facts()
    });
    p.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: -10_000_000,
        box7: 0,
        box8: 0,
        activities: vec![activity],
    });
    let (mut s, reply) = save_k1s(vec![p]);
    accepted(&reply);
    let sale = earlier_installment(
        crate::PropertyCharacter::BusinessProperty {
            used: crate::PropertyUse::Business,
            unrecaptured: 0,
        },
        0,
        2_500_000,
    );
    let before = s.draft.clone();
    accepted(&send(
        &mut s,
        save_topic(
            &before,
            TaxSection::InvestmentSales,
            vec![
                document(0, IncomeDocument::InstallmentPayments(sale.page())),
                document(
                    0,
                    IncomeDocument::BusinessPropertyLosses(crate::BusinessPropertyLosses {
                        regular: vec![],
                        amt: vec![],
                    }),
                ),
            ],
        ),
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), -38_000);
    assert_eq!(value(&r, "current_net_operating_loss"), 35_000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 31_000);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
    let rows = page_documents(&open_topic(&mut s, TaxSection::PassThrough)).to_vec();
    // Missing regular business facts leave only regular current NOL out;
    // separately known AMT and ordinary capital amounts still compute.
    for alternative in [false, true] {
        let mut rows = rows.clone();
        let IncomeDocument::K1EstateTrust(p) = &mut rows[0].document else { unreachable!() };
        let c = &mut p.final_character.as_mut().unwrap().capital;
        if alternative {
            c.alternative_business = None;
        } else {
            c.regular_business = None;
        }
        let before = s.draft.clone();
        accepted(&send(&mut s, save_topic(&before, TaxSection::PassThrough, rows)));
        let result = supported(&s.draft);
        assert_eq!(value(&result, "agi"), -38_000);
        assert_eq!(
            value(&result, "current_net_operating_loss"),
            if alternative { 35000 } else { 0 }
        );
        assert_eq!(
            value(&result, "amt_current_net_operating_loss"),
            if alternative { 0 } else { 31000 }
        );
        assert!(noted(
            &result,
            if alternative {
                "WARN_ESTATE_CAPITAL_ATNOL_LEFT_OUT"
            } else {
                "WARN_ESTATE_CAPITAL_NOL_LEFT_OUT"
            }
        ));
        assert_eq!(estimate(&restart(s.draft.clone()).draft), result);
    }
}

#[test]
fn k1_1041_actual_current_entity_year_keeps_the_december_52_53_exception() {
    // Derived2026 Reg1.441-2(e)(1),(3) application, not a shifted copy of
    // its Wednesday2001 published example: nearest-Sunday election means
    // prior Dec28,2025 closing, Dec29,2025 beginning, Jan3,2027 closing.
    // These 371 actual days belong to the calendar beneficiary's2026.
    let elected = FinalEstateTrustTaxPeriod {
        period: FinalEstateTrustLossPeriod { began: 20251229, ended: 20270103 },
        accounting: FinalEstateTrustAccountingYear::Weeks52Or53 {
            annual_began: 20251229,
            annual_ended: 20270103,
            reference_ended: 20261231,
            closing: FinalEstateTrustWeekClosing::Nearest,
        },
    };
    let mut page = k1_1041("Synthetic actual December-reference source");
    page.entity_period = Some(crate::EstateCurrentPeriodPage::from_record(&elected));
    page.interest = 1_000_000;
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    assert_eq!(stored_k1_1041(&session).1.entity_period, Some(elected.clone()));
    let expected = supported(&session.draft);
    assert_eq!(value(&expected, "agi"), 70_000);
    assert_eq!(value(&expected, "estate_trust_2_1_interest"), 10_000);
    let before = session.draft.clone();
    session.dirty = false;
    let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, rows)));
    let mut again = restart(before.clone());
    let docs = reopened_k1s(&mut again);
    let IncomeDocument::K1EstateTrust(p) = &docs[0] else { unreachable!() };
    assert_eq!(p.entity_period.as_ref().map(crate::EstateCurrentPeriodPage::record), Some(elected));
    assert_eq!(estimate(&again.draft), expected);

    // The identical Jan2027 actual ending is not a2026 ordinary fiscal
    // source. Neither a different beneficiary year nor invented election
    // boundaries can change the accepted amount/draft/dirty state.
    for (period, key) in [
        (
            FinalEstateTrustTaxPeriod {
                period: FinalEstateTrustLossPeriod { began: 20260105, ended: 20270103 },
                accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
            },
            "ESTATE_SOURCE_PERIOD_INVALID",
        ),
        (
            FinalEstateTrustTaxPeriod {
                period: FinalEstateTrustLossPeriod { began: 20241230, ended: 20251228 },
                accounting: FinalEstateTrustAccountingYear::Weeks52Or53 {
                    annual_began: 20241230,
                    annual_ended: 20251228,
                    reference_ended: 20251231,
                    closing: FinalEstateTrustWeekClosing::Nearest,
                },
            },
            "ESTATE_SOURCE_PERIOD_INVALID",
        ),
        (
            FinalEstateTrustTaxPeriod {
                period: FinalEstateTrustLossPeriod { began: 20241229, ended: 20260103 },
                accounting: FinalEstateTrustAccountingYear::Weeks52Or53 {
                    annual_began: 20241229,
                    annual_ended: 20260103,
                    reference_ended: 20251231,
                    closing: FinalEstateTrustWeekClosing::Nearest,
                },
            },
            "ESTATE_SOURCE_PERIOD_INVALID",
        ),
        (
            FinalEstateTrustTaxPeriod {
                period: FinalEstateTrustLossPeriod { began: 20260105, ended: 20270103 },
                accounting: FinalEstateTrustAccountingYear::Weeks52Or53 {
                    annual_began: 20260105,
                    annual_ended: 20270103,
                    reference_ended: 20261231,
                    closing: FinalEstateTrustWeekClosing::Nearest,
                },
            },
            "ESTATE_SOURCE_PERIOD_INVALID",
        ),
    ] {
        let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(p) = &mut rows[0].document else { unreachable!() };
        p.entity_period = Some(crate::EstateCurrentPeriodPage::from_record(&period));
        session.dirty = false;
        rejected_with(send(&mut session, save_topic(&before, TaxSection::PassThrough, rows)), key);
        assert_eq!(session.draft, before);
        assert!(!session.dirty);
    }
    // An ordinary fiscal and a genuine ordinary short2026 period remain
    // current sources; the optional period never changes income character.
    for (began, ended) in [(20250701, 20260630), (20261001, 20261215)] {
        let mut p = k1_1041("Synthetic ordinary actual source");
        p.interest = 1_000_000;
        p.entity_period =
            Some(crate::EstateCurrentPeriodPage::from_record(&FinalEstateTrustTaxPeriod {
                period: FinalEstateTrustLossPeriod { began, ended },
                accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
            }));
        let (s, reply) = save_k1s(vec![p]);
        accepted(&reply);
        assert_eq!(value(&supported(&s.draft), "agi"), 70_000);
    }
}

#[test]
fn k1_1041_current_entity_period_does_not_broaden_final_nol_history() {
    let mut page =
        estate_nol_page("Synthetic history remains separate", 2025, 20_000, 20_000, None);
    let (mut s, reply) = save_k1s(vec![page.clone()]);
    accepted(&reply);
    let before = s.draft.clone();
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.years[0].original_period =
        Some(FinalEstateTrustLossPeriod { began: 20251229, ended: 20270103 });
    let mut rows = page_documents(&open_topic(&mut s, TaxSection::PassThrough)).to_vec();
    rows[0].document = IncomeDocument::K1EstateTrust(page);
    s.dirty = false;
    rejected_with(
        send(&mut s, save_topic(&before, TaxSection::PassThrough, rows)),
        "ESTATE_NOL_PERIOD_INVALID",
    );
    assert_eq!(s.draft, before);
    assert!(!s.dirty);
}

#[test]
fn k1_1041_code12j_retains_signed_next_year_source_without_adding_current_amt() {
    //2026draft beneficiary instructions printedp3:12J helps2027 Form8801.
    //2025final fiduciary i1041 printedp47:it is the exclusion-item portion
    //of12A, not a second2026 adjustment. Opposite-signed deferral/exclusion
    //components need not have absolute values bounded by their net12A.
    for exclusion in [None, Some(-200_000), Some(0), Some(400_000)] {
        let mut p = k1_1041("Synthetic retained exclusion statement");
        p.more.amt = K1EstateTrustAmt::CodeAOnly { a: 100_000 };
        p.more.minimum_tax_exclusion = exclusion;
        let (mut s, reply) = save_k1s(vec![p]);
        accepted(&reply);
        assert_eq!(stored_k1_1041(&s).1.minimum_tax_exclusion, exclusion);
        let expected = supported(&s.draft);
        assert_eq!(value(&expected, "agi"), 60_000);
        assert_eq!(value(&expected, "alternative_minimum_taxable_income"), 61_000);
        let before = s.draft.clone();
        let rows = page_documents(&open_topic(&mut s, TaxSection::PassThrough)).to_vec();
        accepted(&send(&mut s, save_topic(&before, TaxSection::PassThrough, rows)));
        let mut again = restart(before.clone());
        let docs = reopened_k1s(&mut again);
        let IncomeDocument::K1EstateTrust(p) = &docs[0] else { unreachable!() };
        assert_eq!(p.more.minimum_tax_exclusion, exclusion);
        assert_eq!(estimate(&again.draft), expected);
        // Normal server validity also rejects malformed restored/source
        // amounts; no truncation or alternate current-tax interpretation.
        let mut rows = page_documents(&open_topic(&mut s, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(p) = &mut rows[0].document else { unreachable!() };
        p.more.minimum_tax_exclusion = Some(100_000_000_001);
        s.dirty = false;
        rejected_with(
            send(&mut s, save_topic(&before, TaxSection::PassThrough, rows)),
            "ESTATE_MINIMUM_TAX_EXCLUSION_INVALID",
        );
        assert_eq!(s.draft, before);
        assert!(!s.dirty);
    }
}

#[test]
fn k1_1041_change_from_52_53_keeps_the_stub_in_one_ordinary_loss_period() {
    // Derived from TD8996 p30, Reg1.441-2(b)(2)(ii), not a published
    // numerical example. An approved change after FridayDec29,2017 to
    // calendar2018 appends Dec30–31 to Jan1–Dec31,2018. This ordinary
    // period actually begins in2017; elected-year nearest-month treatment
    // cannot relabel it2018. Eight following fiduciary periods leave12.
    let mut page = estate_nol_page(
        "Synthetic ordinary year after elected year",
        2017,
        100_000,
        100_000,
        Some(12),
    );
    let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
        unreachable!()
    };
    nol.terminated_on = Some(20261231);
    let original = FinalEstateTrustLossPeriod { began: 20171230, ended: 20181231 };
    nol.years[0].original_period = Some(original.clone());
    nol.taxable_periods.push(FinalEstateTrustTaxPeriod {
        period: original,
        accounting: FinalEstateTrustAccountingYear::CalendarFiscalAfterWeeks52Or53 {
            annual_began: 20180101,
        },
    });
    for year in 2019..=2026 {
        nol.taxable_periods.push(FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod {
                began: year * 10000 + 101,
                ended: year * 10000 + 1231,
            },
            accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
        });
    }
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let before = session.draft.clone();
    let expected = supported(&before);
    // $60,000 wages less $16,100 standard deduction; this pre2018
    // ordinary-period loss deducts its full $100,000 balance and absorbs
    // $43,900 for carryover purposes, not the post2017 80% ($35,120).
    assert_eq!(value(&expected, "prior_nol_deduction"), 100_000);
    assert_eq!(value(&expected, "estate_trust_2_nol_2017_used"), 43_900);
    assert_eq!(value(&expected, "regular_total_nol_carryforward"), 56_100);
    assert_eq!(estimate(&restart(before.clone()).draft), expected);
    let (_, record) = stored_k1_1041(&session);
    let nol = record.final_nol.unwrap();
    assert_eq!(nol.taxable_periods.len(), 9);
    assert_eq!(nol.years[0].year, 2017);
    assert_eq!(nol.years[0].remaining_years, Some(12));
    for bad_begin in [20171231, 20180102, 20180107] {
        let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
        let K1FinalYear::Final { nol: Some(nol), .. } = &mut page.more.final_year else {
            unreachable!()
        };
        nol.taxable_periods[0].accounting =
            FinalEstateTrustAccountingYear::CalendarFiscalAfterWeeks52Or53 {
                annual_began: bad_begin,
            };
        let dirty = session.dirty;
        rejected_with(
            send(&mut session, save_topic(&before, TaxSection::PassThrough, items)),
            "ESTATE_NOL_PERIOD_INVALID",
        );
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
    }
    let items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, items)));
    assert_eq!(supported(&session.draft), expected);
}

#[test]
fn k1_1041_current_ordinary_appended_stub_preserves_strict_page_source_dates() {
    let mut page = k1_1041("Synthetic current ordinary combined period");
    page.interest = 1_000_000;
    page.entity_period = Some(crate::EstateCurrentPeriodPage {
        began: 20251230,
        ended: 20261231,
        accounting: crate::EstateCurrentAccountingPage::CalendarFiscalAfterWeeks52Or53 {
            annual_began: 20260101,
        },
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let before = session.draft.clone();
    let expected = supported(&before);
    assert_eq!(value(&expected, "agi"), 70_000);
    assert_eq!(estimate(&restart(before.clone()).draft), expected);
    let items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &items[0].document else { unreachable!() };
    assert_eq!(page.entity_period.as_ref().unwrap().began, 20251230);
    assert_eq!(page.entity_period.as_ref().unwrap().ended, 20261231);
    assert!(matches!(
        page.entity_period.as_ref().unwrap().accounting,
        crate::EstateCurrentAccountingPage::CalendarFiscalAfterWeeks52Or53 {
            annual_began: 20260101
        }
    ));
    accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, items)));
    assert_eq!(supported(&session.draft), expected);
}

#[test]
fn k1_1041_one_qbi_statement_keeps_fully_allowed_material_profit_and_loss() {
    // Derived Reg.1.199A-3(b)(2)(i)(B) source endpoint: both actual
    // activities are fully allowed. Fiduciary QBI5000 is not allocated
    // across its10000 profit and5000 loss or reduced a second time.
    let mut page = k1_1041("Synthetic fully allowed aggregate QBI source");
    let material = crate::EstateParticipation::Material(ScorpParticipation {
        minutes: 30_060,
        ..participation_facts()
    });
    let mut profit =
        estate_activity_page("Material profit", crate::EstateActivityKind::Business, 1_000_000);
    profit.participation = material.clone();
    let mut loss =
        estate_activity_page("Material loss", crate::EstateActivityKind::Business, -500_000);
    loss.participation = material;
    page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: 500_000,
        box7: 0,
        box8: 0,
        activities: vec![profit, loss],
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
    let K1EstateTrustBusiness::Amounts(activities) = &page.business else { unreachable!() };
    let ids: Vec<_> = activities.activities.iter().map(|a| a.id).collect();
    page.qbi.statements.push(crate::EstateQbiStatementPage {
        allocations: Vec::new(),
        name: "Actual fiduciary statement".into(),
        income: 500_000,
        wages: 0,
        property: 0,
        service: false,
        kind: crate::EstateQbiKind::Business,
        activities: ids,
        passive_history: vec![],
    });
    let before = session.draft.clone();
    accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, items)));
    let expected = supported(&session.draft);
    assert_eq!(value(&expected, "agi"), 65_000);
    assert_eq!(value(&expected, "qbi_deduction"), 1_000);
    assert!(!noted(&expected, "WARN_ESTATE_QBI_SOURCE_LEFT_OUT"));
    assert_eq!(estimate(&restart(session.draft.clone()).draft), expected);
    let before = session.draft.clone();
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut items[0].document else { unreachable!() };
    assert_eq!(page.qbi.statements[0].income, 500_000);
    assert_eq!(page.qbi.statements[0].activities.len(), 2);
    // A source with a suspended linked loss is different: preserve its
    // printed statement, omit only the undetermined QBI benefit.
    let K1EstateTrustBusiness::Amounts(activities) = &mut page.business else { unreachable!() };
    activities.activities[1].participation = crate::EstateParticipation::Passive;
    accepted(&send(&mut session, save_topic(&before, TaxSection::PassThrough, items)));
    let limited = supported(&session.draft);
    assert_eq!(value(&limited, "agi"), 70_000);
    assert_eq!(value(&limited, "qbi_deduction"), 0);
    assert_eq!(value(&limited, "estate_trust_2_activity_4_suspended"), 5_000);
    assert!(noted(&limited, "WARN_ESTATE_QBI_SOURCE_LEFT_OUT"));
    assert_eq!(estimate(&restart(session.draft.clone()).draft), limited);
}

fn estate_investment_loan(paid: i64) -> crate::InvestmentBorrowingPage {
    crate::InvestmentBorrowingPage {
        loans: vec![crate::InvestmentLoanPage {
            paid_owners: None,
            carry_owners: None,
            originating_k1: None,
            allowed_origin_choices: vec![],
            label: "Estate investment borrowing".into(),
            source: 0,
            paid,
            carry: 0,
            amt_carry: 0,
            private_activity: 0,
        }],
        election: crate::InvestmentElection::None,
        income_use: crate::InvestmentInterestUse::Ordinary,
        limited_allocation: crate::InvestmentLimitedAllocation::SourceDetermined,
    }
}

#[test]
fn k1_1041_shared_multi_activity_qbi_uses_actual_limited_current_and_history_sources() {
    // Derived from1.199A-3(b)(1)(iv): passive activity has current loss10,000
    // plus prior2,000; a separate passive profit6,000 releases prior2,000
    // then current4,000. Two QBI statements share qualified current loss
    // 6,000+2,000 and retained qualified loss750+250, without duplicating
    // either source. Material activity contributes8,000 current QBI:
    // netQBI8,000-40%*8,000-1,000=3,800; deduction20%=760.
    let mut page = k1_1041("Synthetic shared fiduciary QBI sources");
    let mut loss =
        estate_activity_page("Limited business", crate::EstateActivityKind::Business, -1_000_000);
    loss.carried.regular = 200_000;
    loss.carried.alternative = 200_000;
    let mut material =
        estate_activity_page("Material business", crate::EstateActivityKind::Business, 1_000_000);
    material.participation = crate::EstateParticipation::Material(ScorpParticipation {
        minutes: 30_001,
        ..participation_facts()
    });
    page.business = K1EstateTrustBusiness::Amounts(crate::EstateActivitiesPage {
        box6: 0,
        box7: 600_000,
        box8: 0,
        activities: vec![
            loss,
            material,
            estate_activity_page(
                "Passive profit",
                crate::EstateActivityKind::RentalRealEstate,
                600_000,
            ),
        ],
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1EstateTrust(page) = &mut rows[0].document else { unreachable!() };
    let K1EstateTrustBusiness::Amounts(activities) = &page.business else { unreachable!() };
    let loss_id = activities.activities[0].id;
    let material_id = activities.activities[1].id;
    page.qbi.statements = [("First", -600_000, 75_000), ("Second", -200_000, 25_000)]
        .into_iter()
        .map(|(name, loss, qualified)| crate::EstateQbiStatementPage {
            name: name.into(),
            income: loss + 400_000,
            wages: 0,
            property: 0,
            service: false,
            kind: crate::EstateQbiKind::Business,
            activities: vec![loss_id, material_id],
            passive_history: vec![],
            allocations: vec![
                crate::EstateQbiAllocationPage {
                    activity: loss_id,
                    income: loss,
                    passive_history: vec![crate::ScorpLossYear {
                        year: 2025,
                        original: 200_000,
                        qualified,
                        remaining: 200_000,
                        remaining_qualified: qualified,
                    }],
                },
                crate::EstateQbiAllocationPage {
                    activity: material_id,
                    income: 400_000,
                    passive_history: vec![],
                },
            ],
        })
        .collect();
    assert_eq!(page.rule(), None);
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, rows);
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "qbi_deduction"), 760);
    assert!(!noted(&result, "WARN_ESTATE_QBI_SOURCE_LEFT_OUT"));
    assert!(!noted(&result, "WARN_ESTATE_QBI_HISTORY_LEFT_OUT"));
    let before = session.draft.clone();
    let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, rows);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, before);
    assert_eq!(estimate(&restart(before.clone()).draft), result);
    for duplicate in [false, true] {
        let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let IncomeDocument::K1EstateTrust(page) = &mut rows[0].document else { unreachable!() };
        if duplicate {
            page.qbi.statements[1].allocations[0].passive_history[0].qualified = 200_000;
            page.qbi.statements[1].allocations[0].passive_history[0].remaining_qualified = 200_000;
        } else {
            page.qbi.statements[0].allocations[0].income += 1;
        }
        let key = if duplicate {
            "ESTATE_QBI_HISTORY_RECONCILE"
        } else {
            "ESTATE_QBI_ALLOCATION_RECONCILE"
        };
        assert_eq!(page.rule(), Some(key));
        let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, rows);
        rejected_with(send(&mut session, request), key);
        assert_eq!(session.draft, before);
    }
}
fn se_health_test_unreimbursed(health: &mut SeHealth) {
    let SeHealth::Plan { establishment, .. } = health else { panic!("plan required") };
    *establishment = HealthEstablishmentPage::Partnership {
        payment: EntityPremiumPayment::IndividualUnreimbursed,
    };
}

/// Derived Form7206 line11/14 case, 2026 source amounts. IRS instructions7206
/// (2025 final) and Notice2008-1 Examples2–4: an entity-established plan uses
/// that corporation's actual Medicare wages, without adding wage income.
/// Synthetic completion: single adult, one $10000 QBI K-1, no other amounts;
/// W-2 box1$11000 already includes $6000 premiums, box5$5000 excludes them.
/// https://www.irs.gov/instructions/i7206
/// https://www.irs.gov/irb/2008-02_IRB
#[test]
fn scorp_health_plan_uses_actual_wage_limit_and_survives_k1_resave() {
    let mut k1 = k1_1120s("Synthetic Health Corp");
    k1.ordinary = 1_000_000;
    k1.participation = Participation::Over500Hours { hours: 600 };
    let K1ScorpQbi::Statement { income, .. } = &mut k1.qbi else { panic!() };
    *income = 1_000_000;
    let (mut s, reply) = save_k1_1120s(vec![k1]);
    accepted(&reply);
    let mut wage = w2(1_100_000);
    wage.social_wages = 500_000;
    wage.social_tax = 31_000;
    wage.medicare_wages = 500_000;
    wage.medicare_tax = 7_250;
    let request =
        save_topic(&s.draft, TaxSection::Wages, vec![document(0, IncomeDocument::W2(wage))]);
    accepted(&send(&mut s, request));
    let wage_id =
        s.draft.income.iter().find(|e| matches!(e.income, FederalIncome::Wage { .. })).unwrap().id;
    let mut benefits = open_benefits(&mut s);
    assert_eq!(benefits.scorporations.len(), 1);
    benefits.scorporations[0].health = SeHealth::Plan {
        establishment: HealthEstablishmentPage::Scorporation {
            payment: EntityPremiumPayment::EntityPaid,
            wage: wage_id,
        },
        months: vec![BusinessHealthMonth {
            month: 1,
            premiums: 600_000,
            child_increase: 0,
            employer_eligible: false,
            reimbursed: None,
        }],
        long_term_care: vec![],
    };
    accepted(&save_benefits(&mut s, benefits.clone()));
    assert_eq!(open_benefits(&mut s), benefits);
    let r = supported(&s.draft);
    let id = benefits.scorporations[0].activity;
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 5000);
    assert_eq!(value(&r, &alloc::format!("business_{id}_medical_premiums")), 1000);
    assert_eq!(value(&r, "agi"), 16000);
    assert!(
        r.review_notes.iter().any(|n| {
            n.english.contains("QBI income less released historical QBI losses: $5000")
        })
    );
    let items = page_documents(&open_topic(&mut s, TaxSection::PassThrough)).to_vec();
    let request = save_topic(&s.draft, TaxSection::PassThrough, items);
    accepted(&send(&mut s, request));
    assert_eq!(open_benefits(&mut s), benefits);
    let mut restarted = restart(s.draft.clone());
    assert_eq!(open_benefits(&mut restarted), benefits);
    let before = s.draft.clone();
    let SeHealth::Plan { establishment, .. } = &mut benefits.scorporations[0].health else {
        panic!()
    };
    *establishment = HealthEstablishmentPage::Scorporation {
        payment: EntityPremiumPayment::EntityPaid,
        wage: 999_999,
    };
    error(current_source_refusal(
        "health-context-16",
        &mut s,
        benefits_request(&before, benefits),
        "PROBLEM_INVALID_HEALTH_PLAN_SOURCE",
        Some("[value/scorporations/0/health/establishment/wage]"),
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before);
    let request = save_topic(&s.draft, TaxSection::Wages, vec![]);
    error(current_source_refusal(
        "health-context-17",
        &mut s,
        request,
        "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
        None,
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before);
}

/// Derived partnership-named policy source case from instructions7206,
/// General Instructions, plan establishment (2025 final /2026draft).
/// https://www.irs.gov/instructions/i7206
/// Neutral completion: active general partner, ordinary/SE earnings20000,
/// no guaranteed payment, insurance box13M or distribution funding; partner
/// directly pays1000 on the partnership's own policy. The existing wage
/// fixture and all other source amounts stay unchanged.
#[test]
fn partnership_named_policy_you_paid_needs_no_fabricated_reimbursement_income() {
    let mut page = k1_1065("Partnership named policy");
    page.interest =
        PartnerInterest::General { participation: Participation::Over500Hours { hours: 600 } };
    page.ordinary = 2_000_000;
    page.se = 2_000_000;
    page.se_gross = SeGross::Nonfarm { amount: 2_000_000 };
    page.qbi = PartnerQbi::Statement {
        reit_dividends: 0,
        ptp_income: 0,
        income: 2_000_000,
        wages: 0,
        property: 0,
        service: false,
        section1231_qbi: 0,
    };
    page.qbi_records.qualified_se_gross = 2_000_000;
    let mut s = partner_benefits_session(vec![page]);
    let before = supported(&s.draft);
    let mut benefits = open_benefits(&mut s);
    let mut health = se_health_test_year(100_000, true);
    let SeHealth::Plan { establishment, .. } = &mut health else { panic!() };
    *establishment = HealthEstablishmentPage::Partnership {
        payment: EntityPremiumPayment::PartnershipPolicyYouPaid,
    };
    benefits.partnerships[0].health = health;
    accepted(&save_benefits(&mut s, benefits.clone()));
    assert_eq!(open_benefits(&mut s), benefits);
    let result = supported(&s.draft);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 1000);
    assert_eq!(value(&result, "agi"), value(&before, "agi") - 1000);
    assert_eq!(value(&result, "self_employment_tax"), value(&before, "self_employment_tax"));
    let FederalIncome::Partnership { data } = &s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .income
    else {
        panic!()
    };
    assert_eq!(data.guaranteed_services, 0);
}

#[test]
fn k1_box14_pages_save_reopen_restart_and_correct_the_same_document() {
    // Approved F3: independent box 14 earnings beside wages near the 2026
    // Social Security wage base. Then correct the category and code A.
    let mut page = k1_1065("Synthetic box 14 partnership");
    page.interest =
        PartnerInterest::General { participation: Participation::Over500Hours { hours: 501 } };
    page.ordinary = 1_500_000;
    page.se = 2_000_000;
    page.se_gross = SeGross::Nonfarm { amount: 3_000_000 };
    page.qbi_records.qualified_se_gross = 3_000_000;
    page.basis.contributions = 10_000_000;
    page.qbi = PartnerQbi::Statement {
        income: 1_500_000,
        wages: 0,
        property: 0,
        service: false,
        section1231_qbi: 0,
        reit_dividends: 0,
        ptp_income: 0,
    };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 18_000_000, 1_800_000);
    let (mut session, reply) = save_1065s(&d, vec![page.clone()]);
    accepted(&reply);
    let before = session.draft.clone();
    let r = supported(&before);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 1_094);
    assert_eq!(value(&r, "qbi_deduction"), 2_891);
    assert_eq!(estimate(&restart(before.clone()).draft), r);
    assert_eq!(session.draft, before);
    assert_eq!(
        K1Partnership::from_record(&page.name, &stored_1065s(&session)[0]),
        Some(page.clone())
    );
    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .id;
    let mut revision = session.draft.revision;
    page.se_gross = SeGross::Farming { amount: 3_000_000 };
    for se in [2_000_000, 1_500_000] {
        page.se = se;
        let request = save_topic(
            &session.draft,
            TaxSection::PassThrough,
            vec![document(id, IncomeDocument::K1Partnership(page.clone()))],
        );
        accepted(&send(&mut session, request));
        revision += 1;
        assert_eq!(session.draft.revision, revision);
        assert_eq!(session.draft.income.iter().filter(|e| e.id == id).count(), 1);
        assert_eq!(
            K1Partnership::from_record(&page.name, &stored_1065s(&session)[0]),
            Some(page.clone())
        );
        let corrected = supported(&session.draft);
        assert_eq!(value(&corrected, "taxpayer_se_farm_profit"), se / 100);
        assert_eq!(value(&corrected, "taxpayer_se_nonfarm_profit"), 0);
        assert_eq!(
            value(&corrected, "taxpayer_self_employment_tax"),
            if se == 2_000_000 { 1_094 } else { 960 }
        );
        assert_eq!(estimate(&restart(session.draft.clone()).draft), corrected);
    }
}

#[test]
fn k1_1041_preferential_investment_statement_reuses_one_regular_and_amt_election() {
    let mut page = k1_1041("Synthetic preferential investment trust");
    page.interest = 100_000;
    page.ordinary_dividends = 40_000;
    page.qualified_dividends = 30_000;
    page.short_gain = 20_000;
    page.long_gain = 100_000;
    page.more.amt =
        K1EstateTrustAmt::Components { a: -35_000, b: -10_000, c: -5_000, d: -20_000, e: 0, f: 0 };
    page.investment_income = Some(crate::EstateInvestmentIncome {
        regular: 260_000,
        alternative: 225_000,
        character: crate::EstateInvestmentCharacter::Preferential {
            qualified: 30_000,
            gains: 120_000,
            amt_qualified: 20_000,
            amt_gains: 95_000,
        },
    });
    let (mut session, reply) = save_k1s(vec![page]);
    accepted(&reply);
    let request = save_deductions(&session.draft, DeductionRoute::Itemize, 0);
    accepted(&send(&mut session, request));
    let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value: expenses, .. } = &mut request else { unreachable!() };
    let mut loan = estate_investment_loan(200_000);
    loan.election = crate::InvestmentElection::Allocated { capital: 50_000, dividends: 20_000 };
    expenses.investment_interest = Some(loan);
    accepted(&send(&mut session, request));
    // Ordinary1,100 + netST200 + elected700 =2,000 regular. AMT keeps
    // ordinary1,100 + netST150 + same elected700 =1,950; separatecarry50.
    let result = supported(&session.draft);
    assert_eq!(value(&result, "investment_interest_deduction"), 2_000);
    assert_eq!(value(&result, "amt_investment_interest_deduction"), 1_950);
    assert_eq!(value(&result, "investment_interest_carryforward"), 0);
    assert_eq!(value(&result, "amt_investment_interest_carryforward"), 50);
    assert_eq!(return_line(&result, "2b"), 100_000);
    assert_eq!(return_line(&result, "3b"), 40_000);
    assert_eq!(estimate(&restart(session.draft.clone()).draft), result);
    // Actual AMT qualified dividends cannot fund a larger regular election.
    let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
    let P::SaveOtherItemized { value: expenses, .. } = &mut request else { unreachable!() };
    expenses.investment_interest.as_mut().unwrap().election =
        crate::InvestmentElection::Allocated { capital: 50_000, dividends: 25_000 };
    accepted(&send(&mut session, request));
    assert!(estimate(&session.draft).issues.iter().any(|i| i.display.key
        == "ISSUE_THE_ELECTED_CAPITAL_GAIN_EXCEEDS_THE_AVAILABLE_AMT_INVESTMENT_CAPIT_6CBAA2D7"));
}

#[test]
fn k1_sale_inputs_partner_save_reopen_restart_and_independent_amt() {
    for separate in [false, true] {
        let mut page = k1_1065("Synthetic packaging partnership");
        page.more.section179_sales = vec![k1_sale_input_statement()];
        page.more.section179_carryover = 200_000;
        page.more.section179_carryover_sold = 200_000;
        if separate {
            let record = page.record().regular;
            let mut amounts = page.more.section179_sales[0].regular.clone().unwrap();
            amounts.depreciation = 600_000;
            page.more.section179_sales[0].alternative =
                crate::K1Section179SaleAmtSource::Reported { amounts };
            page.amt = PartnerAmtRecords::Separate { record };
        }
        let (session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
        accepted(&reply);
        let stored = &stored_1065s(&session)[0];
        assert!(stored.section179_sales[0].id > 0);
        page.more.section179_sales[0].id = stored.section179_sales[0].id;
        assert_eq!(stored.section179_sales, page.record().section179_sales);
        assert_eq!(stored.section179_carryover_sold, 200_000);
        assert_eq!(stored.section179_carryover, 200_000);
        assert_eq!(
            stored.section179_sales[0].alternative.reported_amounts().unwrap().depreciation,
            if separate { 600_000 } else { 800_000 }
        );
        assert_eq!(K1Partnership::from_record(&page.name, stored), Some(page.clone()));
        let mut restored = restart(session.draft.clone());
        assert_eq!(restored.draft.income, session.draft.income);
        assert_eq!(reopened_k1s(&mut restored), vec![IncomeDocument::K1Partnership(page)]);
        // The saved fully taxable statement now reaches final calculation.
        let result = estimate(&restored.draft);
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
    }
}

#[test]
fn k1_sale_inputs_scorp_save_reopen_restart_and_independent_amt() {
    for separate in [false, true] {
        let mut page = k1_1120s("Synthetic packaging corporation");
        page.more.section179_sales = vec![k1_sale_input_statement()];
        page.more.section179_carryover = 200_000;
        page.more.section179_carryover_sold = 200_000;
        if separate {
            let record = ScorpAmtRecords::same_as(&page.record());
            let mut amounts = page.more.section179_sales[0].regular.clone().unwrap();
            amounts.depreciation = 600_000;
            page.more.section179_sales[0].alternative =
                crate::K1Section179SaleAmtSource::Reported { amounts };
            page.amt = ScorpAmtRecords::Separate { record };
        }
        let (session, reply) = save_k1_1120s(vec![page.clone()]);
        accepted(&reply);
        let stored = stored_k1_1120s(&session);
        assert!(stored.section179_sales[0].id > 0);
        page.more.section179_sales[0].id = stored.section179_sales[0].id;
        assert_eq!(stored.section179_sales, page.record().section179_sales);
        assert_eq!(stored.section179_carryover_sold, 200_000);
        let projected = stored.amt.as_ref().unwrap().source(&stored).unwrap();
        assert_eq!(
            projected.section179_sales[0].regular.as_ref().unwrap().depreciation,
            if separate { 600_000 } else { 800_000 }
        );
        assert_eq!(K1Scorp::from_record(&page.name, &stored), Some(page.clone()));
        let mut restored = restart(session.draft.clone());
        assert_eq!(restored.draft.income, session.draft.income);
        assert_eq!(reopened_k1s(&mut restored), vec![IncomeDocument::K1Scorp(page)]);
        let result = estimate(&restored.draft);
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
    }
}

#[test]
fn k1_sale_inputs_root_relationships_reject_on_save_with_exact_keys() {
    for (carryover, sold, section179, expected) in [
        (200_000, 250_000, 2_000_000, "K1_SALE_CARRYOVER_ABOVE_CARRYOVER"),
        (300_000, 250_000, 200_000, "K1_SALE_CARRYOVER_ABOVE_SHARE"),
    ] {
        let mut partner = k1_1065("Synthetic partnership");
        partner.more.section179_sales = vec![k1_sale_input_statement()];
        partner.more.section179_sales[0].regular.as_mut().unwrap().section179 = section179;
        partner.more.section179_carryover = carryover;
        partner.more.section179_carryover_sold = sold;
        assert_eq!(partner.rule(), Some(expected));
        assert!(!partner.record().valid());
        rejected_with(save_1065s(&complete(FederalStatus::Single), vec![partner]).1, expected);
        let mut scorp = k1_1120s("Synthetic corporation");
        scorp.more.section179_sales = vec![k1_sale_input_statement()];
        scorp.more.section179_sales[0].regular.as_mut().unwrap().section179 = section179;
        scorp.more.section179_carryover = carryover;
        scorp.more.section179_carryover_sold = sold;
        assert_eq!(scorp.rule(), Some(expected));
        assert!(!scorp.record().valid());
        rejected_with(save_k1_1120s(vec![scorp]).1, expected);
    }
}

#[test]
fn k1_sale_inputs_nested_regular_and_amt_rules_use_the_same_save_keys() {
    for (case, expected) in [
        (0, "K1_SALE_DATES"),
        (1, "K1_SALE_BASIS_BELOW_ZERO"),
        (2, "K1_SALE_CARRYOVER_ABOVE_SHARE"),
    ] {
        for amt in [false, true] {
            let mut sale = k1_sale_input_statement();
            match case {
                0 => sale.acquired = 20260701,
                1 => sale.regular.as_mut().unwrap().depreciation = 5_000_000,
                _ => sale.regular.as_mut().unwrap().section179 = 100_000,
            }
            let mut partner = k1_1065("Synthetic partnership");
            partner.more.section179_sales = vec![k1_sale_input_statement()];
            partner.more.section179_carryover = 200_000;
            partner.more.section179_carryover_sold = 200_000;
            if amt {
                let record = partner.record().regular;
                if case == 0 {
                    let mut amt_sale = sale.clone();
                    amt_sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&amt_sale.regular);
                    amt_sale.regular = None;
                    partner.more.section179_sales.push(amt_sale);
                } else {
                    partner.more.section179_sales[0].alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
                partner.amt = PartnerAmtRecords::Separate { record };
            } else {
                partner.more.section179_sales = vec![sale.clone()];
            }
            assert_eq!(partner.rule(), Some(expected));
            assert!(!partner.record().valid());
            rejected_with(save_1065s(&complete(FederalStatus::Single), vec![partner]).1, expected);
            let mut scorp = k1_1120s("Synthetic corporation");
            scorp.more.section179_sales = vec![k1_sale_input_statement()];
            scorp.more.section179_carryover = 200_000;
            scorp.more.section179_carryover_sold = 200_000;
            if amt {
                let record = ScorpAmtRecords::same_as(&scorp.record());
                if case == 0 {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                    sale.regular = None;
                    scorp.more.section179_sales.push(sale);
                } else {
                    scorp.more.section179_sales[0].alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
                scorp.amt = ScorpAmtRecords::Separate { record };
            } else {
                scorp.more.section179_sales = vec![sale];
            }
            assert_eq!(scorp.rule(), Some(expected));
            assert!(!scorp.record().valid());
            rejected_with(save_k1_1120s(vec![scorp]).1, expected);
        }
    }
}

#[test]
fn k1_sale_inputs_ambiguous_personal_split_is_saved_without_guessing() {
    let mut page = k1_1065("Synthetic partnership");
    page.more.section179_sales = vec![k1_sale_input_statement(), k1_sale_input_statement()];
    page.more.section179_sales[1].description = "Second packaging line".into();
    page.more.section179_carryover = 200_000;
    page.more.section179_carryover_sold = 200_000;
    assert_eq!(page.rule(), None);
    assert!(page.record().valid());
    let (session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    assert_eq!(
        K1Partnership::from_record(&page.name, &stored_1065s(&session)[0]),
        Some({
            for (row, accepted) in page
                .more
                .section179_sales
                .iter_mut()
                .zip(&stored_1065s(&session)[0].section179_sales)
            {
                assert!(accepted.id > 0);
                row.id = accepted.id;
            }
            page
        })
    );
    let p = &stored_1065s(&session)[0];
    assert_eq!(
        crate::K1Section179Sale::worksheets(
            &p.section179_sales,
            p.section179_carryover,
            p.section179_carryover_sold
        )
        .unwrap_err()
        .0,
        "ISSUE_K1_SALE_CARRYOVER_SPLIT"
    );
}

#[test]
fn k1_sale_inputs_printed_list_replacement_preserves_personal_carryover_and_id() {
    let mut page = k1_1065("Synthetic partnership");
    page.more.section179_sales = vec![k1_sale_input_statement()];
    page.more.section179_carryover = 200_000;
    page.more.section179_carryover_sold = 200_000;
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    let id = session
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .unwrap()
        .id;
    let revision = session.draft.revision;
    // Simulate the selected printed fields returned by document import;
    // the separate metadata regression checks its actual selection mask.
    page.more.section179_sales[0].regular.as_mut().unwrap().price = 3_500_000;
    let request = save_topic(
        &session.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Partnership(page.clone()))],
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, revision + 1);
    assert_eq!(session.draft.income.iter().filter(|e| e.id == id).count(), 1);
    let child = stored_1065s(&session)[0].section179_sales[0].id;
    assert!(child > 0);
    page.more.section179_sales[0].id = child;
    let mut restored = restart(session.draft.clone());
    assert_eq!(reopened_k1s(&mut restored), vec![IncomeDocument::K1Partnership(page)]);
    assert_eq!(stored_1065s(&restored)[0].section179_carryover_sold, 200_000);
    let before = restored.draft.clone();
    let mut invalid = k1_1065("Synthetic partnership");
    invalid.more.section179_carryover = 200_000;
    invalid.more.section179_carryover_sold = 200_000;
    let request = save_topic(
        &restored.draft,
        TaxSection::PassThrough,
        vec![document(id, IncomeDocument::K1Partnership(invalid))],
    );
    rejected_with(send(&mut restored, request), "K1_SALE_CARRYOVER_ABOVE_SHARE");
    assert_eq!(restored.draft, before);
}

#[test]
fn k1_sale_inputs_raw_records_check_every_statement_and_both_tax_systems() {
    for amt in [false, true] {
        for bad in 0..5 {
            let mut sales = vec![k1_sale_input_statement()];
            match bad {
                0 => sales[0].regular.as_mut().unwrap().price = -1,
                1 => sales[0].sold = 20260230,
                2 => sales[0].description = "".into(),
                3 => sales[0].regular.as_mut().unwrap().cost = 100_000_000_001,
                _ => sales.resize(21, k1_sale_input_statement()),
            }
            let mut partner = k1_1065("Synthetic partnership").record();
            if amt {
                for sale in &mut sales {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                    sale.regular = None;
                }
            }
            partner.section179_sales = sales.clone();
            assert!(!partner.valid(), "partner AMT={amt}, bad={bad}");
            let mut scorp = k1_1120s("Synthetic corporation").record();
            scorp.section179_sales = sales;
            assert!(!scorp.valid(), "corporation AMT={amt}, bad={bad}");
        }
    }
}

#[test]
fn k1_sale_final_public_f2_correction_restart_and_invalid_save_keep_identity() {
    for mut source in
        crate::tests::passthrough::k1_sale_return::sources(15_000, 2_000, 2_000, 30_000)
    {
        k1_qbi_statement(&mut source, 15_000, false);
        let page = match &source {
            FederalIncome::Partnership { data } => IncomeDocument::K1Partnership(
                K1Partnership::from_record("Synthetic F2 partnership", data).unwrap(),
            ),
            FederalIncome::Scorp { data } => IncomeDocument::K1Scorp(
                K1Scorp::from_record("Synthetic F2 corporation", data).unwrap(),
            ),
            _ => unreachable!(),
        };
        let (mut base, _) = crate::tests::passthrough::k1_sale_return::draft(source);
        base.income.clear();
        let mut session = Interview::new(TEST_USER, base.clone());
        let reply = send(
            &mut session,
            save_topic(&base, TaxSection::PassThrough, vec![document(0, page.clone())]),
        );
        accepted(&reply);
        let result = estimate(&session.draft);
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
        assert_eq!(value(&result, "agi"), 21_000);
        assert_eq!(value(&result, "qbi_deduction"), 980);
        assert_eq!(value(&result, "taxable_income"), 3_920);
        let id = session.draft.income[0].id;
        let revision = session.draft.revision;
        // Correct the actual accepted editor so hidden source IDs survive.
        let mut corrected = reopened_k1s(&mut session).remove(0);
        match &mut corrected {
            IncomeDocument::K1Partnership(page) => {
                page.more.section179_sales[0].regular.as_mut().unwrap().price = 2_000_000
            }
            IncomeDocument::K1Scorp(page) => {
                page.more.section179_sales[0].regular.as_mut().unwrap().price = 2_000_000
            }
            _ => unreachable!(),
        }
        let current = session.draft.clone();
        let reply = send(
            &mut session,
            save_topic(&current, TaxSection::PassThrough, vec![document(id, corrected.clone())]),
        );
        accepted(&reply);
        assert_eq!(session.draft.income[0].id, id);
        assert_eq!(session.draft.revision, revision + 1);
        let result = estimate(&session.draft);
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
        assert_eq!(value(&result, "agi"), 11_000);
        assert_eq!(value(&result, "section1231_2026_next_lookback"), 4_000);
        let mut restored = restart(session.draft.clone());
        assert_eq!(estimate(&restored.draft), result);
        assert_eq!(reopened_k1s(&mut restored), vec![corrected.clone()]);
        match &mut corrected {
            IncomeDocument::K1Partnership(page) => page.more.section179_carryover_sold = 250_000,
            IncomeDocument::K1Scorp(page) => page.more.section179_carryover_sold = 250_000,
            _ => unreachable!(),
        }
        let accepted_draft = restored.draft.clone();
        let reply = send(
            &mut restored,
            save_topic(&accepted_draft, TaxSection::PassThrough, vec![document(id, corrected)]),
        );
        rejected_with(reply, "K1_SALE_CARRYOVER_ABOVE_CARRYOVER");
        assert_eq!(restored.draft, accepted_draft);
    }
}

#[test]
fn partial_partner_fuel_recovery_native_save_reopens_restarts_and_rejects_invalid_atomically() {
    let mut page = k1_1065("Synthetic fuel partnership");
    page.ordinary = 2_000_000;
    page.interest = PartnerInterest::LlcMember {
        box1_in_se: false,
        participation: Participation::Over500Hours { hours: 600 },
    };
    page.more.prior_fuel_credit = Some(partial_partner_fuel_recovery());
    let (mut session, reply) = save_1065s(&complete(FederalStatus::Single), vec![page.clone()]);
    accepted(&reply);
    let report = supported(&session.draft);
    assert_eq!(value(&report, "partnership_2_prior_fuel_credit_recovery"), 718);
    let before = session.draft.clone();
    let reopened = reopened_k1s(&mut session);
    assert_eq!(reopened, vec![IncomeDocument::K1Partnership(page)]);
    let mut restored = restart(before.clone());
    assert_eq!(reopened_k1s(&mut restored), reopened);
    assert_eq!(supported(&restored.draft), report);
    let mut items = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
    let IncomeDocument::K1Partnership(page) = &mut items[0].document else { unreachable!() };
    let prior = page.more.prior_fuel_credit.as_mut().unwrap();
    // Even an inactive comparison is validated before the Save commits.
    prior.full_prior_tax_benefit = true;
    prior.partial_prior_year_benefit.as_mut().unwrap().worksheet_reference.clear();
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, items);
    let dirty = session.dirty;
    current_income_refusal(
        "partial-partner-fuel-invalid-worksheet",
        &mut session,
        request,
        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
        "[documents/documents/0/document/more/prior_fuel_credit/partial_prior_year_benefit]",
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    assert_eq!(supported(&restart(session.draft.clone()).draft), report);
}
