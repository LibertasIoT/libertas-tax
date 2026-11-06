// Investment sources: employee stock, ESPP and NUA share sales, qualified small business stock,
// section 1244 losses, worthless securities, nonbusiness bad debts, 1099-DIV return of capital
// and liquidation, Form 2439, section 197 intangibles and the fuel tax credit. Each amount is
// worked by hand in the comment beside it from
// Pub. 525 Examples 10 to 12, Pub. 550 (section 1244 and return of capital examples) and the
// Schedule D instructions, with the 2026 amounts the App uses: $16,100 single standard deduction,
// 10% to $12,400, 12% to $50,400, 22% to $105,700, 0% capital gain rate through $49,450 of taxable
// income, Tax Table rows $50 wide taxed at their midpoint. The single filer `complete` has W-2
// wages of $60,000: AGI 60,000, taxable 43,900, tax 5,023 (Tax Table midpoint 43,925: 1,240 + 12%
// x 31,525). Every amount is synthetic.
use super::*;
use crate::tests::cross::interview::{
    accepted, business, cash_farm, document, farm_save, import_unselected, open_topic,
    page_documents, rejected_with, save_businesses, save_topic, stored_business,
};
use crate::tests::support::builders::capital_return_document;
use crate::tests::support::client::income_pages::{
    open_documents, qsbs_accepted_source, qsbs_bool_fact, save_documents, save_espp_lot, topic_for,
};
use crate::{
    CapitalReturn1099Div, EmployeeShareSale, EmployeeShareSaleMore, Form2439, LiquidationProperty,
    NonbusinessBadDebt, NuaInheritance, NuaShareSale, NuaShareSaleMore, QsbStockLot, QsbsSale,
    Section1244Stock, StockLossBasis, StockLossEvent, WorthlessSecurity,
};
use alloc::{format, string::String};

fn single() -> Interview {
    Interview::new(TEST_USER, complete(FederalStatus::Single))
}
// Saves source documents and topic records through their actual editors.
fn investment(s: &mut Interview, documents: Vec<IncomeSource>) -> Reply {
    save(s, TaxSection::InvestmentSales, documents)
}
fn save(s: &mut Interview, topic: TaxSection, documents: Vec<IncomeSource>) -> Reply {
    let (records, sources): (Vec<_>, Vec<_>) = documents
        .into_iter()
        .map(|d| document(0, d))
        .partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    let source_reply = if sources.is_empty() {
        None
    } else {
        let request = save_documents(s, sources);
        let reply =
            crate::tests::support::client::income_pages::save_source_fixture_request(s, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
        accepted(&reply);
        Some(reply)
    };
    if records.is_empty() {
        return source_reply.expect("at least one source or record");
    }
    let request = save_topic(s, topic, records);
    send(s, request)
}
// The id of the session's only stored income of a kind.
fn id_of(s: &Interview, wanted: impl Fn(&FederalIncome) -> bool) -> i64 {
    s.draft.income.iter().find(|e| wanted(&e.income)).expect("a stored entry").id
}
// Observe the topic's source documents and records through their separate editors.
fn reopened(s: &mut Interview, topic: TaxSection) -> Vec<IncomeSource> {
    let mut rows: Vec<_> = page_documents(&open_documents(s))
        .into_iter()
        .filter(|row| topic_for(&row.document) == topic)
        .collect();
    rows.extend(page_documents(&open_topic(s, topic)));
    rows.into_iter().map(|row| row.document).collect()
}
// Saving both current editors unchanged keeps the stored records.
fn round_trips(s: &mut Interview, topic: TaxSection) {
    let before = s.draft.income.clone();
    let sources = open_documents(s);
    if page_documents(&sources).iter().any(|row| topic_for(&row.document) == topic) {
        accepted(&send(s, sources));
    }
    let records = open_topic(s, topic);
    if !page_documents(&records).is_empty() {
        accepted(&send(s, records));
    }
    assert_eq!(s.draft.income, before);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}

fn tax(r: &FederalResult) -> i64 {
    r.tax.expect("a tax")
}

// ---- F1: employee stock (Pub. 525 Example 12) ----

// 100 shares bought at $10 ($1,000 paid), vested 2025-02-01 at $200 a share (value $20,000, $19,000
// in the 2025 W-2), sold 2026-03-01 for $25,000 net.
fn rsu_sale(sold: u32) -> EmployeeShareSale {
    EmployeeShareSale {
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
        sold,
        proceeds: 2_500_000,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        taxed: 20250201,
        value: 2_000_000,
        paid: 100_000,
        more: EmployeeShareSaleMore::default(),
    }
}

#[test]
fn employee_share_sale_matches_pub_525_example_12() {
    // Basis = 1,000 + (20,000 - 1,000) + 0 = 20,000; sold more than a year after 2025-02-01, so
    // long-term: gain 5,000. AGI 65,000; taxable 48,900; the 5,000 is inside the 0% band (48,900 is
    // at most 49,450): tax 5,023. Form 8949 box A with code B (the broker's box 1e omits the
    // compensation): Schedule D line 8b, not the summary line 8a.
    let mut s = single();
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(rsu_sale(20260301)),
        }],
    ));
    let id = id_of(&s, |i| matches!(i, FederalIncome::EmployeeStockSale { .. }));
    // An unnamed source stores no name; its display name is LABEL_ENTRY_*.
    assert!(s.draft.income.last().unwrap().label.is_empty());
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &format!("employee_stock_{id}_{suffix}"));
    assert_eq!(line("compensation"), 19_000);
    assert_eq!(line("basis"), 20_000);
    assert_eq!(line("capital_gain"), 5_000);
    assert_eq!(value(&r, "agi"), 65_000);
    assert_eq!(tax(&r), 502_300);
    assert_eq!(value(&r, "capital_gain_8b"), 5_000);
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    assert_eq!(value(&r, "capital_gain_1b"), 0);
    // It reopens as the same kind with the same facts.
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(rsu_sale(20260301))
            }]
        )
    );
    round_trips(&mut s, TaxSection::InvestmentSales);
    // Boundary: sold 2026-02-01, exactly one year after the 2025-02-01 vesting, is short-term. The
    // 5,000 is ordinary: taxable 48,900, Tax Table midpoint 48,925: 1,240 + 12% x 36,525 = 5,623.
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(rsu_sale(20260201)),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_1b"), 5_000);
    assert_eq!(value(&r, "capital_gain_8b"), 0);
    assert_eq!(tax(&r), 562_300);
    // The special answer leaves the sale out with its notice.
    let mut left_out = rsu_sale(20260301);
    left_out.more.special = true;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(left_out.clone()),
        }],
    ));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_EMPLOYEE_STOCK_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(left_out)
            }]
        )
    );
}

#[test]
fn employee_share_sale_rules_hold_on_both_sides() {
    let mut s = single();
    let mut v = rsu_sale(20260301);
    v.taxed = 20260302;
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(v),
            }],
        ),
        "EMPLOYEE_STOCK_SOLD_BEFORE_TAXED",
    );
    for sold in [20251231, 20270101] {
        rejected_with(
            investment(
                &mut s,
                vec![IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::EmployeeShareSale(rsu_sale(sold)),
                }],
            ),
            "EMPLOYEE_STOCK_SALE_NOT_IN_RETURN_YEAR",
        );
    }
    // The wash sale loss is at most the loss: basis max(value, paid) + fees = 20,000 + 100, proceeds
    // 19,000 gives a loss of 1,100.
    let mut v = rsu_sale(20260301);
    v.proceeds = 1_900_000;
    v.more.fees = 10_000;
    v.more.wash_sale = 110_000;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(v.clone()),
        }],
    ));
    v.more.wash_sale = 110_001;
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(v),
            }],
        ),
        "SALE_WASH_SALE_ABOVE_LOSS",
    );
}

// ---- F2 and F3: ESPP (Pub. 525 Examples 10 and 11) ----

fn espp_page(granted: u32, exercised: u32, sold: u32, in_wages: bool) -> EsppSale {
    let mut lot = ordinary_espp_lot();
    lot.granted = granted;
    lot.exercised = exercised;
    lot.sold = sold;
    lot.in_wages = in_wages;
    lot
}

#[test]
fn espp_qualifying_sale_matches_pub_525_example_10() {
    // Granted 2023-11-01 (box 3 $22, box 8 $20), exercised 2025-05-01 (box 4 $23, box 5 $20), sold
    // 2026-07-01 for $3,000: more than 2 years after the grant (2025-11-01) and 1 year after the
    // exercise (2026-05-01), so qualifying. Compensation = the lesser of 100 x (22 - 20) = 200 and
    // the gain 3,000 - 2,000 = 1,000: 200, on Schedule 1 line 8k (no W-2 includes it). Basis 2,200;
    // long-term gain 800. AGI 61,000; taxable 44,900; ordinary 44,100 (Tax Table midpoint 44,125:
    // 1,240 + 12% x 31,725 = 5,047) and the 800 at 0%: tax 5,047.
    let mut s = single();
    let page = espp_page(20231101, 20250501, 20260701, false);
    accepted(&save_espp_lot(&mut s, &page));
    let id = id_of(&s, |i| matches!(i, FederalIncome::EsppSaleSource { .. }));
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &format!("espp_sale_{id}_{suffix}"));
    assert_eq!(line("compensation"), 200);
    assert_eq!(line("basis"), 2_200);
    assert_eq!(line("capital_gain"), 800);
    assert_eq!(value(&r, "agi"), 61_000);
    assert_eq!(tax(&r), 504_700);
    // Code B in box A: line 8b.
    assert_eq!(value(&r, "capital_gain_8b"), 800);
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EsppShareSale(espp_source_paper(&page))
            }]
        )
    );
    round_trips(&mut s, TaxSection::InvestmentSales);
}

#[test]
fn espp_disqualifying_sale_matches_pub_525_example_11_and_reconciles_wages() {
    // Granted 2024-06-01, exercised 2025-12-01, sold 2026-06-01 for $3,000: disqualifying.
    // Compensation = 100 x (23 - 20) = 300 (not limited by the gain); basis 2,300; sold within a
    // year of the exercise, so short-term gain 700. The W-2 wages are 60,300 and include it
    // (in_wages): AGI 60,300 + 700 = 61,000; taxable 44,900 all ordinary, Tax Table midpoint
    // 44,925: 1,240 + 12% x 32,525 = 5,143.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_030_000, 0);
    let mut s = Interview::new(TEST_USER, d);
    let page = espp_page(20240601, 20251201, 20260601, true);
    accepted(&save_espp_lot(&mut s, &page));
    let id = id_of(&s, |i| matches!(i, FederalIncome::EsppSaleSource { .. }));
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &format!("espp_sale_{id}_{suffix}"));
    assert_eq!(line("compensation"), 300);
    assert_eq!(line("basis"), 2_300);
    assert_eq!(line("capital_gain"), 700);
    assert_eq!(value(&r, "agi"), 61_000);
    assert_eq!(tax(&r), 514_300);
    assert_eq!(value(&r, "capital_gain_1b"), 700);
    round_trips(&mut s, TaxSection::InvestmentSales);
    // Wages below the compensation marked as included block the estimate; the correction is the
    // sale's page or the W-2.
    let mut low = s.draft.clone();
    low.income[0] = wage(1, Owner::Taxpayer, 29_900, 0);
    let r = estimate(&low);
    let issue = r
        .issues
        .iter()
        .find(|i| {
            i.display.key
                == "ISSUE_COMBINED_STOCK_COMPENSATION_MARKED_AS_ALREADY_REPORTED_CANNOT_EXCEE_0E5F6C00"
        })
        .unwrap_or_else(|| panic!("{:?}", r.issues));
    assert!(matches!(
        &issue.context,
        IssueContext::Corrective { sections, .. }
            if *sections == [TaxSection::InvestmentSales, TaxSection::Wages]
    ));
    // With no W-2 including it, the 300 is Schedule 1 line 8k and the tax is the same.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save_espp_lot(&mut s, &espp_page(20240601, 20251201, 20260601, false)));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 61_000);
    assert_eq!(tax(&r), 514_300);
}

#[test]
fn espp_share_sale_rules_and_special_answer() {
    let mut s = single();
    let mut v = espp_page(20250601, 20250501, 20260701, false);
    rejected_with(save_espp_lot(&mut s, &v), "ESPP_EXERCISED_BEFORE_GRANTED");
    v = espp_page(20231101, 20260801, 20260701, false);
    rejected_with(save_espp_lot(&mut s, &v), "ESPP_SOLD_BEFORE_EXERCISED");
    rejected_with(
        save_espp_lot(&mut s, &espp_page(20231101, 20250501, 20270102, false)),
        "ESPP_SALE_NOT_IN_RETURN_YEAR",
    );
    // The wash sale bound is the server's (the basis needs the per-share products).
    let mut v = espp_page(20231101, 20250501, 20260701, false);
    v.proceeds = 100_000;
    v.wash_sale = 1_000_001;
    rejected_with(save_espp_lot(&mut s, &v), "SALE_WASH_SALE_ABOVE_LOSS");
    let mut s = single();
    let paper = espp_source_paper(&ordinary_espp_lot());
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EsppShareSale(paper),
        }],
    ));
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    for row in followups {
        if let DocumentAnswer::EsppAdjustments { value } = &mut row.answer {
            value.special = true;
        }
    }
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_ESPP_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
    let answer = s.draft.screening.as_ref().unwrap().espp_sale_answers.last().unwrap();
    assert!(answer.more.special);
    assert!(answer.acquisition.is_none());
    assert!(answer.wage_inclusion.is_none());
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    assert!(!followups.iter().any(|row| matches!(
        row.answer,
        DocumentAnswer::EsppAcquisition { .. }
            | DocumentAnswer::EsppFixedGrantPrice { .. }
            | DocumentAnswer::EsppWageInclusion { .. }
    )));
    for row in followups {
        if let DocumentAnswer::EsppAdjustments { value } = &mut row.answer {
            value.special = false;
        }
    }
    accepted(&send(&mut s, request));
    let P::SaveIncomes { followups, .. } = open_topic(&mut s, TaxSection::InvestmentSales) else {
        unreachable!()
    };
    assert!(
        followups.iter().any(|row| matches!(row.answer, DocumentAnswer::EsppAcquisition { .. }))
    );
}

// The ESPP paper: 100 shares granted 2024-01-01 (market value $30.00), bought 2024-06-30 (market
// value $35.00) for $25.50 a share, the price the plan fixed from the grant-date value; sold
// 2026-10-01 for $5,000 with box 1e $2,550 not reported to the IRS. Entered from the plan's
// records, with no Form 3922.
fn plan_record_espp_lot(grant_price: Option<i64>) -> EsppSale {
    EsppSale {
        granted: 20240101,
        exercised: 20240630,
        sold: 20261001,
        grant_market: 3_000,
        grant_price,
        exercise_price: 2_550,
        market_price: 3_500,
        proceeds: 500_000,
        reported_basis: Some(255_000),
        reporting: BasisReporting::NotReported,
        ..ordinary_espp_lot()
    }
}

#[test]
fn espp_records_answer_takes_the_grant_date_price_and_its_item_names_that_field() {
    // Qualifying (sold more than 2 years after the grant and 1 year after the purchase):
    // compensation the lesser of 100 x (30.00 - 25.50) = 450 and the gain 5,000 - 2,550 = 2,450:
    // 450 on Schedule 1, no W-2 including it. Basis 2,550 + 450 = 3,000, long-term gain 2,000.
    // AGI 60,000 + 450 + 2,000 = 62,450.
    let mut s = single();
    accepted(&save_espp_lot(&mut s, &plan_record_espp_lot(Some(2_550))));
    let id = id_of(&s, |i| matches!(i, FederalIncome::EsppSaleSource { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("espp_sale_{id}_compensation")), 450);
    assert_eq!(value(&r, &format!("espp_sale_{id}_basis")), 3_000);
    assert_eq!(value(&r, &format!("espp_sale_{id}_capital_gain")), 2_000);
    assert_eq!(value(&r, "agi"), 62_450);
    // A records answer without that price is refused at its own field.
    let mut other = single();
    let reply = save_espp_lot(&mut other, &plan_record_espp_lot(None));
    let Reply::Response(P::Problem { field: Some(field), .. }) = &reply else {
        panic!("{reply:?}")
    };
    assert!(field.contains("espp_acquisition") && field.contains("Records"), "{field}");
    assert!(field.trim_end_matches(']').ends_with("grant_price"), "{field}");
    rejected_with(reply, "ESPP_RECORDS_GRANT_PRICE_NEEDED");
    // A stored records answer without it: the estimate item names that field, and the page asks
    // no separate Form 3922 box 8 question.
    let answer = s.draft.screening.as_mut().unwrap().espp_sale_answers.last_mut().unwrap();
    let Some(crate::EsppAcquisitionChoice::Records { facts }) = &mut answer.acquisition else {
        unreachable!()
    };
    facts.grant_price = crate::SourceCents::Unavailable;
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    let item = remaining_items
        .iter()
        .find(|row| {
            crate::Message::decode(&row.item).args.first()
                == Some(&crate::MessageArg::Resource("ESPP_RECORDS_GRANT_PRICE_NEEDED".into()))
        })
        .unwrap_or_else(|| panic!("{remaining_items:?}"));
    assert_eq!(item.section, TaxSection::InvestmentSales);
    let field = item.field.as_deref().unwrap();
    assert!(field.contains(&format!("income/{id}/espp_acquisition")), "{field}");
    assert!(field.trim_end_matches(']').ends_with("Records/facts/grant_price"), "{field}");
    let mut request = open_topic(&mut s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    assert!(
        !followups
            .iter()
            .any(|row| matches!(row.answer, DocumentAnswer::EsppFixedGrantPrice { .. }))
    );
    // Answering it there clears the item.
    for row in followups.iter_mut() {
        if let DocumentAnswer::EsppAcquisition {
            value: Some(crate::EsppAcquisitionPage::Records { facts }),
        } = &mut row.answer
        {
            facts.grant_price = crate::SourceCents::Reported { amount: 2_550 };
        }
    }
    accepted(&send(&mut s, request));
    assert_eq!(value(&supported(&s.draft), "agi"), 62_450);
}

// The ESPP paper as the import returns it: the 1099-B boxes and the employer's plan record.
fn plan_record_espp_paper(grant_price: Option<i64>) -> crate::EsppShareSale {
    crate::EsppShareSale {
        plan_record: Some(crate::EsppPlanRecord {
            granted: 20240101,
            exercised: 20240630,
            grant_market: 3_000,
            market_price: 3_500,
            exercise_price: 2_550,
            grant_price,
        }),
        ..espp_source_paper(&plan_record_espp_lot(None))
    }
}

// Saves one document row through the Document center and answers each wage-inclusion question
// No, as no W-2 includes the compensation.
fn save_espp_paper(s: &mut Interview, paper: crate::EsppShareSale) -> i64 {
    let request = save_documents(
        s,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EsppShareSale(paper),
            },
        )],
    );
    accepted(&send(s, request));
    id_of(s, |i| matches!(i, FederalIncome::EsppSaleSource { .. }))
}
fn answer_followups(s: &mut Interview, answer: impl Fn(&mut DocumentAnswer)) {
    let mut request = open_topic(s, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    followups.iter_mut().for_each(|row| answer(&mut row.answer));
    accepted(&send(s, request));
}

#[test]
fn an_espp_plan_record_on_the_row_supplies_the_acquisition_facts() {
    // The same sale as above, its purchase facts read from the plan record printed with the
    // 1099-B: compensation 450, basis 3,000, long-term gain 2,000, AGI 62,450. Investment sales
    // asks no acquisition source.
    let mut s = single();
    let id = save_espp_paper(&mut s, plan_record_espp_paper(Some(2_550)));
    let P::SaveIncomes { followups, .. } = open_topic(&mut s, TaxSection::InvestmentSales) else {
        unreachable!()
    };
    assert!(!followups.iter().any(|row| matches!(
        row.answer,
        DocumentAnswer::EsppAcquisition { .. } | DocumentAnswer::EsppFixedGrantPrice { .. }
    )));
    answer_followups(&mut s, |answer| {
        if let DocumentAnswer::EsppWageInclusion { value } = answer {
            *value = Some(false);
        }
    });
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("espp_sale_{id}_compensation")), 450);
    assert_eq!(value(&r, &format!("espp_sale_{id}_basis")), 3_000);
    assert_eq!(value(&r, &format!("espp_sale_{id}_capital_gain")), 2_000);
    assert_eq!(value(&r, "agi"), 62_450);
    // A record without the grant-date price: only that price is asked, its estimate item names
    // that question, and the answer completes the sale.
    let mut s = single();
    let id = save_espp_paper(&mut s, plan_record_espp_paper(None));
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    let item = remaining_items
        .iter()
        .find(|row| {
            crate::Message::decode(&row.item).args.first()
                == Some(&crate::MessageArg::Resource("ESPP_RECORD_GRANT_PRICE_NEEDED".into()))
        })
        .unwrap_or_else(|| panic!("{remaining_items:?}"));
    assert_eq!(item.section, TaxSection::InvestmentSales);
    assert!(
        item.field.as_deref().unwrap().contains(&format!("income/{id}/espp_fixed_grant_price"))
    );
    let P::SaveIncomes { followups, .. } = open_topic(&mut s, TaxSection::InvestmentSales) else {
        unreachable!()
    };
    assert!(
        !followups.iter().any(|row| matches!(row.answer, DocumentAnswer::EsppAcquisition { .. }))
    );
    answer_followups(&mut s, |answer| {
        if let DocumentAnswer::EsppFixedGrantPrice { value } = answer {
            *value = crate::SourceCents::Reported { amount: 2_550 };
        }
    });
    answer_followups(&mut s, |answer| {
        if let DocumentAnswer::EsppWageInclusion { value } = answer {
            *value = Some(false);
        }
    });
    assert_eq!(value(&supported(&s.draft), "agi"), 62_450);
    round_trips(&mut s, TaxSection::InvestmentSales);
}

// ---- F4: NUA shares ----

// Distributed 2024-05-01 with a value of $50,000 and $30,000 of deferred NUA (basis $20,000), sold
// 2026-03-01 for $60,450.
fn nua_page(distributed: u32, proceeds: i64) -> NuaShareSale {
    NuaShareSale {
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
        sold: 20260301,
        proceeds,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        distributed,
        distribution_value: 5_000_000,
        deferred_nua: 3_000_000,
    }
}
// The sale's answers on Investment sales: own shares, no wash sale or special
// treatment.
fn no_nua_adjustments() -> NuaShareSaleMore {
    NuaShareSaleMore { wash_sale: 0, special: false }
}
fn answer_nua(s: &mut Interview, more: NuaShareSaleMore, ownership: crate::NuaOwnership) -> Reply {
    crate::tests::support::client::income_pages::answer_nua_sale(s, &more, &ownership)
}
fn own_nua_sale(s: &mut Interview, page: NuaShareSale) {
    accepted(&investment(
        s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::NuaShareSale(page),
        }],
    ));
    accepted(&answer_nua(s, no_nua_adjustments(), crate::NuaOwnership::Own));
}

#[test]
fn nua_share_sale_splits_the_nua_from_the_post_distribution_gain() {
    // The NUA part 30,000 is long-term whatever the holding period; the rest, 60,450 - 50,000 =
    // 10,450, is long-term too (held from the distribution more than a year). AGI 100,450; taxable
    // 84,350; ordinary 43,900 (5,023); 0% on 49,450 - 43,900 = 5,550; 15% on 40,450 - 5,550 =
    // 34,900 = 5,235: tax 10,258. Both parts are code B lots in box A: Schedule D line 8b.
    let mut s = single();
    own_nua_sale(&mut s, nua_page(20240501, 6_045_000));
    let id = id_of(&s, |i| matches!(i, FederalIncome::NuaSaleSource { .. }));
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &format!("nua_sale_{id}_{suffix}"));
    assert_eq!(line("basis"), 20_000);
    assert_eq!(line("realized_nua"), 30_000);
    assert_eq!(line("post_distribution_gain"), 10_450);
    assert_eq!(value(&r, "agi"), 100_450);
    assert_eq!(tax(&r), 1_025_800);
    // The remainder's proceeds go on top of the NUA part's 30,000 of proceeds and 50,000 of basis:
    // 30,000 + (60,450 - 50,000).
    assert_eq!(value(&r, "capital_gain_8b"), 40_450);
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    round_trips(&mut s, TaxSection::InvestmentSales);
    // Boundary: distributed 2025-09-01, the remainder is short-term (10,450 on line 1b) and the 30,000
    // NUA part stays long-term. Ordinary income 54,350 (Tax Table midpoint 54,375: 1,240 + 4,560 +
    // 22% x 3,975 = 6,674.50, rounded 6,675); the 30,000 is above the 0% band: 15% = 4,500. Tax
    // 11,175.
    own_nua_sale(&mut s, nua_page(20250901, 6_045_000));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_1b"), 10_450);
    assert_eq!(value(&r, "capital_gain_8b"), 30_000);
    assert_eq!(tax(&r), 1_117_500);
    // A loss has no realized NUA: proceeds 15,000 against the 20,000 basis is a (5,000) long-term
    // loss; AGI 55,000.
    own_nua_sale(&mut s, nua_page(20240501, 1_500_000));
    let id = id_of(&s, |i| matches!(i, FederalIncome::NuaSaleSource { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("nua_sale_{id}_realized_nua")), 0);
    // No NUA is realized, so the broker's plan cost is the actual basis: no code B, the summary
    // line 8a.
    assert_eq!(value(&r, "capital_gain_8a"), -5_000);
    assert_eq!(value(&r, "agi"), 57_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 2_000);
}

#[test]
fn nua_share_sale_inheritance_rules_and_special_answer() {
    let mut s = single();
    // The printed sale saves without its answers: nothing is pre-answered,
    // and the sale waits for them on Investment sales.
    let v = nua_page(20240501, 6_045_000);
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::NuaShareSale(v.clone()),
        }],
    ));
    let id = id_of(&s, |i| matches!(i, FederalIncome::NuaSaleSource { .. }));
    assert_eq!(crate::nua_sale::answers(&s.draft, id), None);
    assert!(noted(&supported(&s.draft), "NUA_SALE_ANSWERS_NEEDED"));
    // The inheritance dates are the ownership answer's rules.
    let inherited = |died| crate::NuaOwnership::Inherited {
        inheritance: NuaInheritance { died, value: 5_500_000, estate_tax: false },
    };
    rejected_with(
        answer_nua(&mut s, no_nua_adjustments(), inherited(20240401)),
        "NUA_DIED_BEFORE_DISTRIBUTED",
    );
    rejected_with(
        answer_nua(&mut s, no_nua_adjustments(), inherited(20260401)),
        "NUA_SOLD_BEFORE_DIED",
    );
    accepted(&answer_nua(&mut s, no_nua_adjustments(), inherited(20250101)));
    let answer = crate::nua_sale::answers(&s.draft, id).unwrap();
    assert_eq!(answer.ownership, Some(inherited(20250101)));
    // The printed sale reopens as printed.
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::NuaShareSale(v)
            }]
        )
    );
    let mut v = nua_page(20240501, 6_045_000);
    v.sold = 20240401;
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::NuaShareSale(v),
            }],
        ),
        "NUA_SOLD_BEFORE_DISTRIBUTED",
    );
    let mut v = nua_page(20240501, 6_045_000);
    v.sold = 20251231;
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::NuaShareSale(v),
            }],
        ),
        "NUA_SALE_NOT_IN_2026",
    );
    let mut v = nua_page(20240501, 6_045_000);
    v.deferred_nua = 5_000_001;
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::NuaShareSale(v),
            }],
        ),
        "NUA_ABOVE_DISTRIBUTION_VALUE",
    );
    // The wash sale bound is the adjustments answer's.
    let mut s = single();
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::NuaShareSale(nua_page(20240501, 1_000_000)),
        }],
    ));
    let reply = answer_nua(
        &mut s,
        NuaShareSaleMore { wash_sale: 1_000_001, ..no_nua_adjustments() },
        crate::NuaOwnership::Own,
    );
    rejected_with(reply, "SALE_WASH_SALE_ABOVE_LOSS");
    // A special treatment is left out with a notice, and its ownership is
    // not asked.
    let mut s = single();
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::NuaShareSale(nua_page(20240501, 6_045_000)),
        }],
    ));
    accepted(&answer_nua(
        &mut s,
        NuaShareSaleMore { special: true, ..no_nua_adjustments() },
        crate::NuaOwnership::Own,
    ));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_NUA_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
    // The ownership question is no longer asked for it.
    let P::SaveIncomes { followups, .. } = crate::tests::support::client::income_pages::open_topic(
        &mut s,
        TaxSection::InvestmentSales,
    ) else {
        unreachable!()
    };
    assert!(
        !followups
            .iter()
            .any(|row| matches!(row.answer, crate::DocumentAnswer::NuaOwnership { .. }))
    );
}

// ---- Form 8949 code B across the four kinds, and Schedule D footing ----

#[test]
fn stock_compensation_sales_are_code_b_lots_not_summary_lines() {
    // Compensation moves a lot off the summary lines 1a and 8a. A qualifying ISO sale, a restricted
    // stock sale with no compensation and an ESPP sale with none stay on them.
    let mut s = single();
    let mut zero = rsu_sale(20260301);
    zero.paid = 2_000_000;
    zero.value = 2_000_000;
    accepted(&investment(
        &mut s,
        vec![
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(rsu_sale(20260301)),
            },
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(zero),
            },
        ],
    ));
    let r = supported(&s.draft);
    // 5,000 with compensation: line 8b; the second lot (paid 20,000 = value, no compensation, sold
    // for 25,000): gain 5,000 on line 8a.
    assert_eq!(value(&r, "capital_gain_8b"), 5_000);
    assert_eq!(value(&r, "capital_gain_8a"), 5_000);
    // The stored sales carry the column (g) correction of code B: the compensation.
    let FederalIncome::EmployeeStockSale { data } = &s.draft.income[1].income else { panic!() };
    assert_eq!(
        data.sale().adjustment,
        crate::SaleAdjustment::BasisCorrection { reported_basis: 100_000 }
    );
    let FederalIncome::EmployeeStockSale { data } = &s.draft.income[2].income else { panic!() };
    assert_eq!(data.sale().adjustment, crate::SaleAdjustment::None);
}

#[test]
fn a_printed_box_1e_and_box_12_place_the_stock_compensation_lot() {
    // F3 (2026-10-09). The restricted-stock lot of the Pub. 525 example: actual basis 20,000
    // (1,000 paid + 19,000 compensation), sold for 25,000. Box 1e as printed (broker figure) is the
    // column (e) amount of the code B row; column (g) is the correction to the actual basis.
    let sale = |reported: Option<i64>, reporting: BasisReporting| {
        let mut page = rsu_sale(20260301);
        page.reported_basis = reported;
        page.reporting = reporting;
        let mut s = single();
        accepted(&investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(page),
            }],
        ));
        let FederalIncome::EmployeeStockSale { data } = &s.draft.income[1].income else { panic!() };
        (data.sale(), supported(&s.draft))
    };
    // The broker printed 1,000 (the paid amount, no compensation): code B, correction 19,000,
    // box A, Schedule D line 8b.
    let (lot, r) = sale(Some(100_000), BasisReporting::ReportedToIrs);
    assert_eq!(lot.adjustment, crate::SaleAdjustment::BasisCorrection { reported_basis: 100_000 });
    assert_eq!(value(&r, "capital_gain_8b"), 5_000);
    // The broker already included the compensation (box 1e = 20,000): nothing to correct, so
    // the lot is a clean summary lot on line 8a with the same gain.
    let (lot, r) = sale(Some(2_000_000), BasisReporting::ReportedToIrs);
    assert_eq!(lot.adjustment, crate::SaleAdjustment::None);
    assert_eq!(value(&r, "capital_gain_8a"), 5_000);
    assert_eq!(value(&r, "capital_gain_8b"), 0);
    // Basis not reported to the IRS (box B/E) or no 1099-B (box C/F): the person's own basis,
    // no code B, and a gain that does not change.
    for reporting in [BasisReporting::NotReported, BasisReporting::NoForm1099B] {
        let (lot, r) = sale(Some(100_000), reporting);
        assert_eq!(lot.adjustment, crate::SaleAdjustment::None);
        assert_eq!(lot.basis_reporting, reporting);
        assert_eq!(value(&r, "agi"), 65_000, "{reporting:?}");
    }
    // An unprinted box 1e keeps the model's figure (the cost without the compensation).
    let (lot, _) = sale(None, BasisReporting::ReportedToIrs);
    assert_eq!(lot.adjustment, crate::SaleAdjustment::BasisCorrection { reported_basis: 100_000 });
}

#[test]
fn a_printed_box_1e_and_box_12_place_the_nua_remainder_lot() {
    // F3 (2026-10-09). The remainder lot of an NUA sale (plan cost 20,000 plus the realized NUA
    // 30,000 is its actual basis): the broker's printed box 1e is its column (e) amount.
    let sale = |reported: Option<i64>, reporting: BasisReporting| {
        let mut page = nua_page(20240501, 6_045_000);
        page.reported_basis = reported;
        page.reporting = reporting;
        let mut s = single();
        own_nua_sale(&mut s, page);
        let data = crate::nua_sale::resolve(&s.draft, &s.draft.income[1]).unwrap();
        (data.parts(), supported(&s.draft))
    };
    // Box 1e shows the plan cost alone: code B, column (g) the NUA added back.
    let (parts, r) = sale(Some(2_000_000), BasisReporting::ReportedToIrs);
    assert_eq!(
        parts[1].adjustment,
        crate::SaleAdjustment::BasisCorrection { reported_basis: 2_000_000 }
    );
    assert_eq!(value(&r, "capital_gain_8b"), 40_450);
    // The broker already included the NUA (box 1e = 50,000): nothing to correct on the remainder,
    // and the gain does not change.
    let (parts, r) = sale(Some(5_000_000), BasisReporting::ReportedToIrs);
    assert_eq!(parts[1].adjustment, crate::SaleAdjustment::None);
    assert_eq!(value(&r, "agi"), 100_450);
    // Basis not reported to the IRS (box B/E) or no 1099-B (box C/F): the person's own basis, no
    // code B on either part, and the same gain.
    for reporting in [BasisReporting::NotReported, BasisReporting::NoForm1099B] {
        let (parts, r) = sale(Some(2_000_000), reporting);
        assert!(parts.iter().all(|p| p.adjustment == crate::SaleAdjustment::None));
        assert!(parts.iter().all(|p| p.basis_reporting == reporting));
        assert_eq!(value(&r, "agi"), 100_450, "{reporting:?}");
    }
}

// Entries added directly to a return, with ids.
fn with_incomes(incomes: Vec<FederalIncome>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    for i in incomes {
        add(&mut d, income(i));
    }
    d
}

// The lines of Schedule D that carry gains: 1a to 4 are short-term, 8a to 11 long-term.
fn schedule_d_totals(r: &FederalResult) -> (i64, i64) {
    let sum = |ids: &[&str]| ids.iter().map(|id| value(r, id)).sum::<i64>();
    (
        sum(&[
            "capital_gain_1a",
            "capital_gain_1b",
            "capital_gain_2",
            "capital_gain_3",
            "capital_gain_4",
        ]),
        sum(&[
            "capital_gain_8a",
            "capital_gain_8b",
            "capital_gain_9",
            "capital_gain_10",
            "capital_gain_11",
        ]),
    )
}

#[test]
fn schedule_d_lines_foot_with_the_investment_kinds() {
    // Long-term: an employee lot (5,000 on 8b), an NUA lot (30,000 + 10,450 on 8b), QSBS issued in
    // 2008 with a 50% exclusion (gain 100,000, taxable 50,000 on box F line 10), the section 1244
    // capital part (a long-term loss of 500 on line 10), a worthless security (500 on line 10), a
    // return of capital (1,000 gain on line 10) and Form 2439 (5,000 on line 11). The source fixture adds a
    // basis-reported collectible (code C, 4,000 on 8b, not 8a), a basis-reported personal-use
    // loss (code L, 0 on 8b) and a Form 4684 casualty gain (5,000 on line 11). The sum of lines
    // 8a to 11 is the long-term net and the short-term lines add to zero, so with no carryover the
    // eight lines, 4 and 11 equal the return's capital gain.
    let mut d = with_incomes(vec![]);
    let s_page = |sold| rsu_sale(sold);
    for entry in [
        crate::interview::document_income(&IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(s_page(20260301)),
        }),
        // The NUA sale as its answers complete it: own shares, nothing special.
        FederalIncome::NuaSale {
            data: nua_page(20240501, 6_045_000).lot(&no_nua_adjustments(), None),
        },
        FederalIncome::QualifiedSmallBusinessSale {
            data: qsbs_page(20080602, 2_000_000, 12_000_000, BasisReporting::NoForm1099B),
        },
        crate::interview::document_income(&IncomeSource::Record {
            value: crate::IncomeRecord::SmallBusinessStockLoss(section_1244_cash(
                1_000_000, 200_000, 900_000,
            )),
        }),
        crate::interview::document_income(&IncomeSource::Record {
            value: crate::IncomeRecord::WorthlessSecurity(worthless(500_000, 20250615)),
        }),
        crate::interview::document_income(&IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Form2439(form_2439(
                500_000, 0, 0, 300_000, 105_000,
            )),
        }),
    ] {
        add(&mut d, income(entry));
    }
    add(
        &mut d,
        income(FederalIncome::ReturnOfCapital {
            kind: CapitalDistributionKind::Nondividend,
            amount: 300_000,
            property: vec![],
            basis: 200_000,
            term: GainTerm::Long,
            special: false,
        }),
    );
    let mut reported_collectible = collectible_sale(10_000, 6_000, GainTerm::Long, false);
    let mut reported_personal_loss = personal_sale(5_000, 6_000, GainTerm::Long);
    for lot in [&mut reported_collectible, &mut reported_personal_loss] {
        let FederalIncome::Sale { data } = lot else { unreachable!() };
        data.form.reporting = BasisReporting::ReportedToIrs;
    }
    add(&mut d, income(reported_collectible));
    add(&mut d, income(reported_personal_loss));
    let mut stolen = personal_disaster("Stolen car", 1_500_000);
    stolen.declaration = crate::CasualtyDeclaration::Undeclared;
    stolen.zip.clear();
    stolen.properties[0].before = 2_000_000;
    stolen.properties[0].reimbursement = 2_000_000;
    stolen.properties[0].received = 2_000_000;
    d.deductions.as_mut().unwrap().personal_casualties.push(stolen);
    let r = supported(&d);
    let (short, long) = schedule_d_totals(&r);
    assert_eq!(short, 0);
    assert_eq!(long, 5_000 + 40_450 + 50_000 - 500 - 5_000 + 1_000 + 5_000 + 4_000 + 5_000);
    assert_eq!(long, value(&r, "capital_gain"));
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    assert_eq!(value(&r, "capital_gain_11"), 10_000);
    // Short-term: an employee lot sold within a year of vesting, a section 1244 loss held under a
    // year, a worthless security bought on 2025-12-31 (short), a bad debt and a return of capital
    // gain, with a 1099-B short-term gain of 30,000 so the net is positive.
    let mut d = with_incomes(vec![]);
    let mut short_sale = rsu_sale(20260201);
    short_sale.taxed = 20250601;
    let mut loss = section_1244_cash(1_000_000, 200_000, 900_000);
    loss.acquired = 20260101;
    for entry in [
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::EmployeeShareSale(short_sale),
        },
        IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(loss) },
        IncomeSource::Record {
            value: crate::IncomeRecord::WorthlessSecurity(worthless(500_000, 20251231)),
        },
        IncomeSource::Record {
            value: crate::IncomeRecord::BadDebt(cash_loan_debt(1_000_000, 200_000)),
        },
    ] {
        add(&mut d, income(crate::interview::document_income(&entry)));
    }
    add(
        &mut d,
        income(FederalIncome::ReturnOfCapital {
            kind: CapitalDistributionKind::Nondividend,
            amount: 300_000,
            property: vec![],
            basis: 200_000,
            term: GainTerm::Short,
            special: false,
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Short, 4_000_000, Some(1_000_000))),
        }),
    );
    let r = supported(&d);
    let (short, long) = schedule_d_totals(&r);
    // 5,000 (employee, short) - 500 (1244 capital) - 5,000 (worthless) - 8,000 (bad debt) + 1,000
    // (return of capital) + 30,000.
    assert_eq!(short, 5_000 - 500 - 5_000 - 8_000 + 1_000 + 30_000);
    assert_eq!(long, 0);
    assert_eq!(short, value(&r, "capital_gain"));
    assert_eq!(value(&r, "capital_gain_1a"), 30_000);
    assert_eq!(value(&r, "capital_gain_1b"), 5_000);
    assert_eq!(value(&r, "capital_gain_3"), -500 - 5_000 - 8_000 + 1_000);
}

// ---- F5: qualified small business stock ----

fn qsbs_page(issued: u32, basis: i64, proceeds: i64, reporting: BasisReporting) -> QsbIssuerSale {
    QsbIssuerSale {
        issuer: "Synthetic Robotics Inc.".into(),
        lots: vec![QsbStockLot {
            issued,
            sold: 20260501,
            basis,
            proceeds,
            zone_gain_through_2018: None,
        }],
        reporting,
        withholding: 0,
        prior_eligible_gain: Some(0),
        issuer_qualified: Some(true),
        active_business: Some(true),
        no_disqualification: Some(true),
        zone_60_exclusion: None,
        complete_history: Some(true),
        special: Some(false),
    }
}

#[test]
fn qsbs_sale_excludes_all_of_a_100_percent_lot_and_the_50_percent_boundary_is_28_percent_gain() {
    // 100%: issued 2015-03-02, basis 100,000, sold 2026-05-01 for 1,100,000, no 1099-B (box F).
    // Gain 1,000,000; the limit is the larger of 10,000,000 and 10 x 100,000: excluded 1,000,000,
    // taxable 0, no 28% gain, no AMT preference. AGI 60,000, tax 5,023.
    let mut s = single();
    let page = qsbs_page(20150302, 10_000_000, 110_000_000, BasisReporting::NoForm1099B);
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(
                crate::tests::support::builders::qsbs_document(&(page.clone())),
            ),
        }],
    ));
    assert_eq!(s.draft.income.last().unwrap().label, "Synthetic Robotics Inc.");
    let id = id_of(&s, |i| matches!(i, FederalIncome::QualifiedSmallBusinessSale { .. }));
    let r = supported(&s.draft);
    let line = |suffix: &str| value(&r, &format!("qsbs_{id}_{suffix}"));
    assert_eq!(line("realized_gain"), 1_000_000);
    assert_eq!(line("exclusion_limit"), 10_000_000);
    assert_eq!(line("excluded_gain"), 1_000_000);
    assert_eq!(line("taxable_gain"), 0);
    assert_eq!(line("amt_preference"), 0);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(tax(&r), 502_300);
    // Form 8949 box F code Q: column (h) is 0 on line 10.
    assert_eq!(value(&r, "capital_gain_10"), 0);
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&(page))
                )
            }]
        )
    );
    round_trips(&mut s, TaxSection::InvestmentSales);
    // 50%: issued 2008-06-02, basis 20,000, proceeds 120,000: gain 100,000, excluded 50,000, the
    // included 50,000 is 28% rate gain (worksheet line 2), AMT preference 7% x 50,000 = 3,500 (Form
    // 6251 line 2h). AGI 110,000; taxable 93,900; Tax Table midpoint 93,925: 1,240 + 4,560 + 22% x
    // 43,525 = 15,375.50, rounded 15,376; the Schedule D Tax Worksheet (43,900 at 5,023 plus 50,000
    // at 28% = 19,023) is higher, so 15,376. AMT: 93,900 + 16,100 + 3,500 = 113,500 less the 90,100
    // exemption = 23,400; at most 26% x 23,400 = 6,084 is below the regular tax: no AMT.
    let page = qsbs_page(20080602, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(
                crate::tests::support::builders::qsbs_document(&(page)),
            ),
        }],
    ));
    let id = id_of(&s, |i| matches!(i, FederalIncome::QualifiedSmallBusinessSale { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("qsbs_{id}_excluded_gain")), 50_000);
    assert_eq!(value(&r, &format!("qsbs_{id}_rate_28_gain")), 50_000);
    assert_eq!(value(&r, &format!("qsbs_{id}_amt_preference")), 3_500);
    assert_eq!(value(&r, "qsbs_amt_preference"), 3_500);
    assert_eq!(printed_line(&r, "qsbs_amt_preference"), (TaxForm::Form6251, "2h".into()));
    assert_eq!(value(&r, "agi"), 110_000);
    assert_eq!(value(&r, "rate_28_gain"), 50_000);
    assert_eq!(tax(&r), 1_537_600);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    // The taxable half is on line 10 (box F).
    assert_eq!(value(&r, "capital_gain_10"), 50_000);
}

#[test]
fn qsbs_taxable_gain_lands_in_the_form_8949_box_the_page_names() {
    // Not sure: no exclusion, the whole 100,000 is long-term gain taxed, with a notice. Box D (basis
    // reported) is line 8b, E line 9 and F line 10; two entries of one issuer split the taxable gain
    // by their own gain.
    for (reporting, line) in [
        (BasisReporting::ReportedToIrs, "capital_gain_8b"),
        (BasisReporting::NotReported, "capital_gain_9"),
        (BasisReporting::NoForm1099B, "capital_gain_10"),
    ] {
        let mut s = single();
        let mut page = qsbs_page(20080602, 2_000_000, 12_000_000, reporting);
        page.issuer_qualified = Some(false);
        accepted(&investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&(page)),
                ),
            }],
        ));
        let r = supported(&s.draft);

        assert_eq!(value(&r, line), 100_000, "{line}");
        assert_eq!(value(&r, "agi"), 160_000);
    }
    // The same issuer on two entries in different boxes: gain 100,000 (box E) and 40,000 (box F),
    // excluded 50% of both (70,000), so taxable 70,000 split 50,000 / 20,000.
    let mut s = single();
    let one = qsbs_page(20080602, 2_000_000, 12_000_000, BasisReporting::NotReported);
    let two = qsbs_page(20080602, 1_000_000, 5_000_000, BasisReporting::NoForm1099B);
    accepted(&investment(
        &mut s,
        vec![
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&(one)),
                ),
            },
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&(two)),
                ),
            },
        ],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_9"), 50_000);
    assert_eq!(value(&r, "capital_gain_10"), 20_000);
}

#[test]
fn qsbs_lots_must_meet_the_original_section_1202_dates() {
    let mut s = single();
    // Issued after July 4, 2025: the three-year tier of P.L. 119-21 cannot be met in 2026.
    let later = qsbs_page(20250705, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&(later)),
                ),
            }],
        ),
        "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
    );
    // Issued before the section 1202 start, or not held more than five years.
    for (issued, sold) in [(19930810, 20260501), (20210501, 20260501), (20210502, 20260501)] {
        let mut page = qsbs_page(issued, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
        page.lots[0].sold = sold;
        rejected_with(
            investment(
                &mut s,
                vec![IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::QsbsSale(
                        crate::tests::support::builders::qsbs_document(&(page)),
                    ),
                }],
            ),
            "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
        );
    }
    // Issued on July 4, 2025 and sold in 2026 is still short of five years; issued 2021-04-30 and
    // sold 2026-05-01 is more than five years.
    let mut page = qsbs_page(20210430, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
    page.lots[0].sold = 20260501;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(
                crate::tests::support::builders::qsbs_document(&(page)),
            ),
        }],
    ));
    // The special answer leaves the sale out.
    let mut special = qsbs_page(20150302, 10_000_000, 110_000_000, BasisReporting::NoForm1099B);
    special.special = Some(true);
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(
                crate::tests::support::builders::qsbs_document(&(special)),
            ),
        }],
    ));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_QSBS_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
}

#[test]
fn validation_audit_f11_qsbs_supported_date_boundaries_save_and_reopen() {
    for (issued, sold) in [
        (19930811, 20260101),
        (20201231, 20260101),
        (20210101, 20260102),
        (20200229, 20260228),
        (20150302, 20261231),
    ] {
        let mut session = single();
        let mut page = qsbs_page(issued, 100_000, 200_000, BasisReporting::NoForm1099B);
        page.lots[0].sold = sold;
        assert!(page.zone_60_exclusion.is_none());
        assert!(page.lots[0].zone_gain_through_2018.is_none());
        assert!(page.lots[0].dates_valid());
        accepted(&investment(
            &mut session,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&(page.clone())),
                ),
            }],
        ));
        supported(&session.draft);
        assert_eq!(
            reopened(&mut session, TaxSection::InvestmentSales),
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::QsbsSale(
                        crate::tests::support::builders::qsbs_document(&(page))
                    )
                }]
            )
        );
        let revision = session.draft.revision;
        round_trips(&mut session, TaxSection::InvestmentSales);
        assert_eq!(session.draft.revision, revision);
    }
}

#[test]
fn validation_audit_f11_qsbs_outside_dates_rejects_fresh_and_retained_rows_unchanged() {
    for (issued, sold) in [
        (19930810, 20260501),
        (20250704, 20260501),
        (20250705, 20260501),
        (20150302, 20251231),
        (20150302, 20270101),
        (20210501, 20260501),
        (20210502, 20260501),
    ] {
        let mut session = single();
        let mut invalid = qsbs_page(issued, 100_000, 200_000, BasisReporting::NoForm1099B);
        invalid.lots[0].sold = sold;
        assert!(!invalid.lots[0].dates_valid());
        let before = session.draft.clone();
        let dirty = session.dirty;
        rejected_with(
            investment(
                &mut session,
                vec![IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::QsbsSale(
                        crate::tests::support::builders::qsbs_document(&(invalid)),
                    ),
                }],
            ),
            "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
        );
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);

        let expected = qsbs_page(20150302, 100_000, 200_000, BasisReporting::NoForm1099B);
        accepted(&investment(
            &mut session,
            vec![IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(&(expected.clone())),
                ),
            }],
        ));
        let mut page = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut page else {
            panic!("expected the investment sales page")
        };
        let crate::InvestmentSaleDocument::QsbsSale(sale) =
            &mut documents.common.investment_sales.documents[0].document
        else {
            panic!("expected the qualified small business stock sale")
        };
        sale.lots[0].issued = issued;
        sale.lots[0].sold = sold;
        let before = session.draft.clone();
        let dirty = session.dirty;
        rejected_with(send(&mut session, page), "QSBS_LOT_OUTSIDE_SECTION_1202_DATES");
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        assert_eq!(
            reopened(&mut session, TaxSection::InvestmentSales),
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::QsbsSale(
                        crate::tests::support::builders::qsbs_document(&(expected))
                    )
                }]
            )
        );
    }
}

#[test]
fn qsbs_lot_gain_facts_are_page_rules_the_person_can_fix() {
    // F3 (2026-10-09): a loss or break-even sale and a zone-gain figure that
    // does not fit the lot are named native source failures
    // instead of refusing the whole estimate with the generic "review the
    // document amounts".
    let mut s = single();
    for (basis, proceeds) in [(2_000_000, 2_000_000), (2_000_000, 1_500_000)] {
        let page = qsbs_page(20150302, basis, proceeds, BasisReporting::NoForm1099B);
        rejected_with(
            investment(
                &mut s,
                vec![IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::QsbsSale(
                        crate::tests::support::builders::qsbs_document(&(page)),
                    ),
                }],
            ),
            "QSBS_LOT_PROCEEDS_NOT_ABOVE_BASIS",
        );
    }
    // Zone gain: the lot gains 10,000,000; 10,000,001 is above it, and a lot
    // issued after the zone dates takes none.
    let zone = |issued: u32, zone_gain: i64| {
        let mut page = qsbs_page(issued, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
        page.lots[0].zone_gain_through_2018 =
            Some(crate::QsbsZoneValuation::Established { gain: zone_gain });
        page
    };
    for (issued, amount, native_key, public_key) in [
        (
            20050301,
            10_000_001,
            "QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN",
            "QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN",
        ),
        (20150302, 100, "QSBS_LOT_ZONE_GAIN_OUTSIDE_ZONE_DATES", "DOCUMENT_QUESTIONS_CHANGED"),
    ] {
        let bad = zone(issued, amount);
        assert_eq!(bad.lots[0].issue().unwrap().key, native_key);
        let mut session = single();
        let request =
            crate::tests::support::builders::qsbs_zone_topic_request(&mut session, &bad, 0);
        let before = session.draft.clone();
        rejected_with(send(&mut session, request), public_key);
        assert_eq!(session.draft, before);
    }
    let mut ok = zone(20050301, 10_000_000);
    ok.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(
                crate::tests::support::builders::qsbs_document(&(ok)),
            ),
        }],
    ));
}

// ---- F6: section 1244 ----

fn section_1244_cash(cost: i64, added: i64, proceeds: i64) -> Section1244Stock {
    Section1244Stock {
        basis: StockLossBasis::Cash { cost, added },
        acquired: 20220110,
        event: StockLossEvent::Sale { date: 20260401, proceeds },

        ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
    }
}

#[test]
fn section_1244_loss_matches_the_pub_550_contribution_example() {
    // $10,000 paid at issue, $2,000 contributed later, sold 2026-04-01 for $9,000: loss 3,000,
    // ordinary 3,000 x 10,000 / 12,000 = 2,500 (Form 4797 line 11), capital 500 long-term (Form
    // 8949 box F code S, line 10). AGI 60,000 - 2,500 - 500 = 57,000; taxable 40,900, Tax Table
    // midpoint 40,925: 1,240 + 12% x 28,525 = 4,663.
    let mut s = single();
    let page = section_1244_cash(1_000_000, 200_000, 900_000);
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::SmallBusinessStockLoss(page.clone()),
        }],
    ));
    // An unnamed source stores no name; its display name is LABEL_ENTRY_*.
    assert!(s.draft.income.last().unwrap().label.is_empty());
    let r = supported(&s.draft);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 2_500);
    assert_eq!(printed_line(&r, "section_1244_ordinary_loss"), (TaxForm::Form4797, "11".into()));
    assert_eq!(value(&r, "capital_gain_10"), -500);
    assert_eq!(value(&r, "agi"), 57_000);
    assert_eq!(tax(&r), 466_300);
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        vec![IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(page) }]
    );
    round_trips(&mut s, TaxSection::InvestmentSales);
}

#[test]
fn section_1244_limit_example_from_the_schedule_d_instructions() {
    // Basis 60,000 paid in cash, sold for 1,000 after three years: the loss 59,000 is ordinary up
    // to 50,000 (Form 4797 line 11) and 9,000 is the Form 8949 code S capital loss (column (h)
    // (9,000), line 10). AGI 60,000 - 50,000 - 3,000 = 7,000. Taxable income before the capital
    // loss deduction is 7,000 + 3,000 - 16,100 = -6,100, below zero, so the Capital Loss Carryover
    // Worksheet uses none of the $3,000 against taxable income: the whole 9,000 long-term loss is
    // carried (the design map's 6,000 forgot the standard deduction). No tax.
    let mut s = single();
    let mut page = section_1244_cash(6_000_000, 0, 100_000);
    page.acquired = 20230101;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::SmallBusinessStockLoss(page.clone()),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 50_000);
    assert_eq!(value(&r, "capital_gain_10"), -9_000);
    assert_eq!(value(&r, "agi"), 7_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 9_000);
    assert_eq!(tax(&r), 0);
    // A joint return limit is 100,000 even if only one spouse has the loss: the whole 59,000 is
    // ordinary and no capital part remains.
    let mut d = complete(FederalStatus::Joint);
    let (stored, _) = (
        crate::interview::document_income(&IncomeSource::Record {
            value: crate::IncomeRecord::SmallBusinessStockLoss(page),
        }),
        0,
    );
    add(&mut d, income(stored));
    let r = supported(&d);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 59_000);
    assert_eq!(value(&r, "capital_gain_10"), 0);
}

#[test]
fn section_1244_property_example_and_unconfirmed_qualification() {
    // Pub. 550: property with a basis of 1,000 and a value of 250 exchanged for stock, sold for 200:
    // loss 800, ordinary 50 (basis reduced to value 250, less proceeds 200), capital 750.
    let mut s = single();
    let property = Section1244Stock {
        basis: StockLossBasis::Property {
            basis: 100_000,
            value: 25_000,
            liabilities: 0,
            acquired: 20200101,
        },
        acquired: 20220110,
        event: StockLossEvent::Sale { date: 20260401, proceeds: 20_000 },

        ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
    };
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::SmallBusinessStockLoss(property.clone()),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 50);
    assert_eq!(value(&r, "capital_gain_10"), -750);
    // Not sure about the qualification: all capital, with a notice.
    let mut unsure = section_1244_cash(1_000_000, 200_000, 900_000);
    unsure.ownership = crate::StockLossOwnership::OriginalIndividual { qualified: false };
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(unsure) }],
    ));
    let r = supported(&s.draft);

    assert_eq!(value(&r, "section_1244_ordinary_loss"), 0);
    assert_eq!(value(&r, "capital_gain_10"), -3_000);
    // Worthlessness not established: no 2026 loss, with a notice.
    let worthless = Section1244Stock {
        event: StockLossEvent::Worthless { confirmed: false },
        ..section_1244_cash(1_000_000, 0, 0)
    };
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::SmallBusinessStockLoss(worthless),
        }],
    ));
    let r = supported(&s.draft);

    assert_eq!(value(&r, "agi"), 60_000);
    // The scope confirmation leaves the loss out.
    let mut left_out = section_1244_cash(1_000_000, 0, 0);
    left_out.ownership = crate::StockLossOwnership::OtherUnmodeledHistory;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(left_out) }],
    ));
    assert!(noted(&supported(&s.draft), "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT"));
}

#[test]
fn section_1244_page_rules_hold_on_both_sides() {
    let mut s = single();
    let check = |s: &mut Interview, edit: fn(&mut Section1244Stock), key: &str| {
        let mut page = section_1244_cash(1_000_000, 200_000, 900_000);
        edit(&mut page);
        rejected_with(
            investment(
                s,
                vec![IncomeSource::Record {
                    value: crate::IncomeRecord::SmallBusinessStockLoss(page),
                }],
            ),
            key,
        );
    };
    check(&mut s, |v| v.acquired = 20270101, "SECTION_1244_ISSUED_AFTER_RETURN_YEAR");
    check(
        &mut s,
        |v| v.event = StockLossEvent::Sale { date: 20220109, proceeds: 0 },
        "SECTION_1244_SOLD_BEFORE_ISSUED",
    );
    check(
        &mut s,
        |v| v.event = StockLossEvent::Sale { date: 20251231, proceeds: 0 },
        "SECTION_1244_SALE_NOT_IN_RETURN_YEAR",
    );
    check(
        &mut s,
        |v| v.event = StockLossEvent::Sale { date: 20260401, proceeds: 1_200_001 },
        "SECTION_1244_PROCEEDS_ABOVE_BASIS",
    );
    check(
        &mut s,
        |v| {
            v.event = StockLossEvent::Worthless { confirmed: true };
            v.basis = StockLossBasis::Property {
                basis: 100,
                value: 100,
                liabilities: 101,
                acquired: 20200101,
            }
        },
        "SECTION_1244_LIABILITIES_ABOVE_BASIS",
    );
    check(
        &mut s,
        |v| {
            v.event = StockLossEvent::Worthless { confirmed: true };
            v.basis = StockLossBasis::Property {
                basis: 100,
                value: 100,
                liabilities: 0,
                acquired: 20220111,
            }
        },
        "SECTION_1244_PROPERTY_AFTER_ISSUE",
    );
}

// ---- F7: worthless securities ----

fn worthless(basis: i64, bought: u32) -> WorthlessSecurity {
    WorthlessSecurity {
        basis,
        acquisition: SecurityAcquisition::Purchased { date: bought },
        worthless: true,
        ordinary: true,
    }
}

#[test]
fn worthless_security_is_sold_on_december_31_in_box_c_or_f() {
    // Bought 2025-06-15 for 5,000, worthless: deemed sold 2026-12-31 for nothing, long-term (5,000)
    // on line 10 (box F). AGI 57,000; long-term carryover 2,000; tax 4,663.
    let mut s = single();
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::WorthlessSecurity(worthless(500_000, 20250615)),
        }],
    ));
    // An unnamed source stores no name; its display name is LABEL_ENTRY_*.
    assert!(s.draft.income.last().unwrap().label.is_empty());
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_10"), -5_000);
    assert_eq!(value(&r, "agi"), 57_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 2_000);
    assert_eq!(tax(&r), 466_300);
    round_trips(&mut s, TaxSection::InvestmentSales);
    // Bought 2025-12-31 is held exactly a year at December 31, 2026: short-term, line 3.
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::WorthlessSecurity(worthless(500_000, 20251231)),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_3"), -5_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 2_000);
    // Not sure it is worthless: no 2026 loss, with a notice.
    let mut unsure = worthless(500_000, 20250615);
    unsure.worthless = false;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(unsure) }],
    ));
    let r = supported(&s.draft);

    assert_eq!(value(&r, "agi"), 60_000);
    // Acquired after 2026 is refused on both sides.
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::WorthlessSecurity(worthless(500_000, 20270101)),
            }],
        ),
        "WORTHLESS_ACQUIRED_AFTER_2026",
    );
    let mut left_out = worthless(500_000, 20250615);
    left_out.ordinary = false;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(left_out) }],
    ));
    assert!(noted(&supported(&s.draft), "WARN_WORTHLESS_SECURITY_LEFT_OUT"));
}

// ---- F8: nonbusiness bad debts ----

fn cash_loan_debt(advanced: i64, recovered: i64) -> NonbusinessBadDebt {
    NonbusinessBadDebt {
        source: BadDebtSource::CashLoan { advanced, recovered },
        investment: false,
        bona_fide: true,
        political_debtor: false,
        worthless: true,
    }
}

#[test]
fn nonbusiness_bad_debt_is_a_short_term_loss_on_box_c() {
    // A $10,000 loan of which $2,000 was repaid: a (8,000) short-term loss on line 3 (box C) whatever
    // the holding period. AGI 60,000 - 3,000 = 57,000; short-term carryover 5,000; tax 4,663.
    let mut s = single();
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::BadDebt(cash_loan_debt(1_000_000, 200_000)),
        }],
    ));
    // An unnamed source stores no name; its display name is LABEL_ENTRY_*.
    assert!(s.draft.income.last().unwrap().label.is_empty());
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_3"), -8_000);
    assert_eq!(value(&r, "agi"), 57_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 5_000);
    assert_eq!(tax(&r), 466_300);
    round_trips(&mut s, TaxSection::InvestmentSales);
    // Not sure it is worthless: no loss, with a notice.
    let mut unsure = cash_loan_debt(1_000_000, 200_000);
    unsure.worthless = false;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record { value: crate::IncomeRecord::BadDebt(unsure) }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 60_000);
    // More recovered than advanced is refused on both sides.
    rejected_with(
        investment(
            &mut s,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::BadDebt(cash_loan_debt(1_000_000, 1_000_001)),
            }],
        ),
        "BAD_DEBT_RECOVERED_ABOVE_ADVANCED",
    );
    // The amounts the stored debt requires are at least one cent.
    let source = crate::federal_model::HUB_SOURCE;
    let sources = &source[source.find("pub enum BadDebtSource {").unwrap()..];
    let sources = &sources[..sources.find("pub enum GuaranteeMotive").unwrap()];
    for field in ["advanced", "paid", "deposited", "face", "price_paid"] {
        let marks: Vec<&str> = sources
            .lines()
            .filter(|l| l.trim_start().starts_with(&format!("{field}: i64")))
            .collect();
        assert!(!marks.is_empty(), "{field}");
        let at = sources.find(marks[0]).unwrap();
        let before = &sources[..at];
        let attribute = before.lines().rev().find(|l| l.contains("libertas_number")).unwrap();
        // The one-cent minimum and exact-cent headroom are both explicit.
        assert_eq!(attribute.trim(), "#[libertas_number(min = 1, max = 100000000000)]", "{field}");
    }
}

// ---- F9: return of capital or liquidation ----

fn capital_return(kind: CapitalDistributionKind, amount: i64, basis: i64) -> CapitalReturn1099Div {
    CapitalReturn1099Div {
        kind,
        amount,
        property: vec![],
        basis,
        term: GainTerm::Long,
        special: false,
    }
}

#[test]
fn dividend_printed_box_identity_normalizes_lots_without_guessing_finality() {
    let mut source = crate::DividendSource::printed(crate::Form1099DivPaper {
        cash_liquidation: Some(360_000),
        ..Default::default()
    });
    source.capital_returns = vec![
        capital_return(CapitalDistributionKind::Nondividend, 120_000, 200_000),
        capital_return(CapitalDistributionKind::InterimLiquidation, 120_000, 200_000),
        capital_return(CapitalDistributionKind::FinalLiquidation, 120_000, 200_000),
    ];
    // The native view follows box identity even before a stored answer is normalized.
    let native = crate::FederalDividend::from(&source);
    assert_eq!(native.capital_returns[0].kind, CapitalDistributionKind::Liquidation);
    source.normalize_capital_return_kinds();
    assert_eq!(source.capital_returns[1].kind, CapitalDistributionKind::InterimLiquidation);
    assert_eq!(source.capital_returns[2].kind, CapitalDistributionKind::FinalLiquidation);
    source.form.cash_liquidation = Some(0);
    source.form.noncash_liquidation = Some(360_000);
    assert_eq!(
        source.form.capital_return_kind(&source.capital_returns[0]),
        CapitalDistributionKind::Liquidation
    );
    let mut correction = crate::DividendSource::printed(crate::Form1099DivPaper {
        nondividend_distributions: Some(360_000),
        ..Default::default()
    });
    correction.keep_answers(&source);
    assert!(
        correction
            .capital_returns
            .iter()
            .all(|lot| lot.kind == CapitalDistributionKind::Nondividend)
    );
    // A form reporting both classes does not identify the class of any given lot.
    source.form.nondividend_distributions = Some(120_000);
    source.form.cash_liquidation = Some(240_000);
    source.capital_returns[0].kind = CapitalDistributionKind::Nondividend;
    let lots = source.capital_returns.clone();
    source.normalize_capital_return_kinds();
    assert_eq!(source.capital_returns, lots);
}

#[test]
fn liquidation_box_identity_preserves_unresolved_final_payment_status() {
    let mut value = capital_return(CapitalDistributionKind::Liquidation, 120_000, 200_000);
    value.property.push(LiquidationProperty { label: "Distributed shares".into(), value: 40_000 });
    assert!(value.issue().is_none());
    assert!(value.needs_liquidation_status());
    assert!(matches!(
        FederalIncome::ReturnOfCapital {
            kind: value.kind,
            amount: value.amount,
            property: value.property.clone(),
            basis: value.basis,
            term: value.term,
            special: value.special,
        },
        FederalIncome::ReturnOfCapital { kind: CapitalDistributionKind::Liquidation, .. }
    ));
    value.amount = 160_000;
    assert!(value.needs_liquidation_status(), "a final zero-gain row still needs finality");
    value.amount = 160_001;
    assert!(
        !value.needs_liquidation_status(),
        "gain and zero remaining basis agree for either stage"
    );
    value.kind = CapitalDistributionKind::FinalLiquidation;
    value.amount = 120_000;
    assert!(!value.needs_liquidation_status());
}

#[test]
fn return_of_capital_matches_pub_550_example_1() {
    // Pub. 550 Example 1 with amounts in hundreds: cost left 2,000, long-term; box 3 of 3,000: the
    // basis falls to zero and 1,000 is long-term gain on line 10. AGI 61,000; taxable 44,900; the
    // gain is in the 0% band: tax 5,023.
    let mut s = single();
    let page = capital_return(CapitalDistributionKind::Nondividend, 300_000, 200_000);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(capital_return_document(
                Default::default(),
                Default::default(),
                page.clone(),
            )),
        }],
    ));
    // An unnamed source stores no name; its display name is LABEL_ENTRY_*.
    assert!(s.draft.income.last().unwrap().label.is_empty());
    let id = id_of(&s, |i| matches!(i, FederalIncome::Dividend { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("distribution_{id}_lot_0_basis_reduction")), 2_000);
    assert_eq!(value(&r, &format!("distribution_{id}_lot_0_remaining_basis")), 0);
    assert_eq!(value(&r, &format!("distribution_{id}_lot_0_capital_gain")), 1_000);
    assert_eq!(value(&r, "capital_gain_10"), 1_000);
    assert_eq!(value(&r, "agi"), 61_000);
    assert_eq!(tax(&r), 502_300);
    assert_eq!(
        reopened(&mut s, TaxSection::InterestDividends),
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(capital_return_document(
                    Default::default(),
                    Default::default(),
                    page,
                ))
            }]
        )
    );
    round_trips(&mut s, TaxSection::InterestDividends);
    // Within cost: box 3 of 1,500 is no gain, 500 of cost remains.
    let within = capital_return(CapitalDistributionKind::Nondividend, 150_000, 200_000);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(capital_return_document(
                Default::default(),
                Default::default(),
                within,
            )),
        }],
    ));
    let id = id_of(&s, |i| matches!(i, FederalIncome::Dividend { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("distribution_{id}_lot_0_remaining_basis")), 500);
    assert_eq!(value(&r, "agi"), 60_000);
    // A final liquidation payment of 1,200 against a cost of 2,000 is a (800) long-term loss; Not
    // sure is taken as not final: no loss and a notice.
    let liquidation = capital_return(CapitalDistributionKind::FinalLiquidation, 120_000, 200_000);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(capital_return_document(
                Default::default(),
                Default::default(),
                liquidation.clone(),
            )),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_10"), -800);
}

#[test]
fn liquidation_property_belongs_to_a_liquidation_and_the_1099_div_keeps_computing() {
    let mut s = single();
    // A noncash-only printed form needs no fabricated cash or personal lot answer.
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(crate::Form1099DivPaper {
                    noncash_liquidation: Some(25_000),
                    ..Default::default()
                }),
            },
        )],
    );
    accepted(&send(&mut s, request));
    s = restart(s.draft);
    let printed = reopened(&mut s, TaxSection::InterestDividends);
    assert!(printed.iter().any(|source| matches!(source,
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(form) }
        if form.noncash_liquidation == Some(25_000)
            && form.nondividend_distributions.unwrap_or(0) == 0
            && form.cash_liquidation.unwrap_or(0) == 0
    )));
    let mut s = single();
    let mut page = capital_return(CapitalDistributionKind::Nondividend, 0, 200_000);
    page.property = vec![LiquidationProperty { label: "Box 10".into(), value: 100_000 }];
    // Mixed printed boxes leave attribution to the actual lot answer. The
    // property relationship remains invalid without testing a static bound.
    let request = save_documents(
        &mut s,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(crate::Form1099DivPaper {
                    nondividend_distributions: Some(1),
                    noncash_liquidation: Some(100_000),
                    ..Default::default()
                }),
            },
        )],
    );
    accepted(&send(&mut s, request));
    let id = id_of(&s, |income| matches!(income, FederalIncome::Dividend { .. }));
    let mut request = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let row =
        followups.iter_mut().find(|row| row.key == format!("income/{id}/capital_returns")).unwrap();
    row.answer = DocumentAnswer::DividendCapitalReturns { value: vec![page.clone()] };
    rejected_with(send(&mut s, request), "CAPITAL_RETURN_PROPERTY_NOT_LIQUIDATION");
    page.kind = CapitalDistributionKind::FinalLiquidation;
    // Cash of 500 and property of 1,000 against a cost of 2,000: a (500) long-term loss.
    page.amount = 50_000;
    let div = dividend_source(FederalDividend {
        ordinary: 50_000,
        cash_liquidation: 50_000,
        noncash_liquidation: 100_000,
        capital_returns: vec![page],
        ..Default::default()
    });
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(div)),
        }],
    ));
    let r = supported(&s.draft);
    // The 1099-DIV box 1a of 500 is no longer left out for a liquidation.
    assert_eq!(value(&r, "capital_gain_10"), -500);
    assert_eq!(value(&r, "agi"), 60_000 + 500 - 500);
    let mut special = capital_return(CapitalDistributionKind::Nondividend, 300_000, 200_000);
    special.special = true;
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(capital_return_document(
                Default::default(),
                Default::default(),
                special,
            )),
        }],
    ));
    assert!(noted(&supported(&s.draft), "WARN_RETURN_OF_CAPITAL_LEFT_OUT"));
}

fn div_1099(ordinary: i64) -> crate::Div1099 {
    native_div1099("Synthetic Fund".into(), ordinary, 0)
}

// ---- F10: Form 2439 ----

fn form_2439(
    gain: i64,
    unrecaptured: i64,
    small_business: i64,
    collectibles: i64,
    tax_paid: i64,
) -> crate::Form2439Paper {
    form2439_paper(Form2439 {
        fund: crate::PrintedParty::from_name("".into()),
        shareholder: Default::default(),
        gain,
        unrecaptured,
        small_business,
        collectibles,
        tax_paid,
    })
}

// The paper on the Document center, then the fund's topic answers the guard supplies.
fn save_fund_with_answers(s: &mut Interview, form: crate::Form2439Paper) -> Reply {
    let request = save_documents(
        s,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Form2439(form),
            },
        )],
    );
    crate::tests::support::client::income_pages::save_source_fixture_request(s, request)
}

#[test]
fn form_2439_reaches_schedule_d_line_11_and_schedule_3_line_13a() {
    // Box 1a 5,000 (long-term, Schedule D line 11), box 1d 3,000, box 2 1,050 (Schedule 3 line 13a).
    // The basis rises by 5,000 - 1,050 = 3,950. AGI 65,000; taxable 48,900: 2,000 of the gain at 0%
    // (up to 49,450 less ordinary income 43,900 is 5,550, but the 28% pool of 3,000 is taxed at the
    // ordinary 12%: 360). Tax 5,023 + 360 = 5,383; the 1,050 is a payment.
    let mut s = single();
    let mut page = form_2439(500_000, 0, 0, 300_000, 105_000);
    page.calendar_year = 0;
    page.other_tax_year_beginning = Some(20250701);
    page.other_tax_year_ending = Some(20260630);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Form2439(page.clone()),
        }],
    ));
    // An unnamed source stores no name; its display name is LABEL_ENTRY_*.
    assert!(s.draft.income.last().unwrap().label.is_empty());
    let id = id_of(&s, |i| matches!(i, FederalIncome::UndistributedGain { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_11"), 5_000);
    assert_eq!(value(&r, "rate_28_gain"), 3_000);
    assert_eq!(value(&r, "undistributed_gain_tax_credit"), 1_050);
    assert_eq!(
        printed_line(&r, "undistributed_gain_tax_credit"),
        (TaxForm::Schedule3, "13a".into())
    );
    assert_eq!(value(&r, &format!("fund_{id}_basis_increase")), 3_950);
    assert_eq!(value(&r, "agi"), 65_000);
    assert_eq!(tax(&r), 538_300);
    assert_eq!(
        reopened(&mut s, TaxSection::InterestDividends),
        expected_named_sources(
            &s.draft,
            Owner::Taxpayer,
            vec![IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Form2439(page)
            }]
        )
    );
    round_trips(&mut s, TaxSection::InterestDividends);
    // Box 1c section 1202 gain is calculated through its exclusion choice (B9-5): the fund's whole
    // 5,000.00 stays on line 11 and the credit is unchanged; a code Q row excludes the 1,000.00.
    let section_1202 = form_2439(500_000, 0, 100_000, 300_000, 105_000);
    let _answers = crate::tests::support::client::income_pages::source_answers(vec![
        crate::DocumentAnswer::DividendFundExclusion {
            value: Some(crate::FundQsbsExclusion::Hundred),
        },
    ]);
    accepted(&save_fund_with_answers(&mut s, section_1202));
    let r = supported(&s.draft);
    assert!(!noted(&r, "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT"));
    assert_eq!(value(&r, "capital_gain_11"), 5_000);
    assert_eq!(value(&r, "undistributed_gain_tax_credit"), 1_050);
    assert_eq!(value(&r, "agi"), 64_000);
    // The ownership answer leaves it out with its own notice.
    let special = form_2439(500_000, 0, 0, 0, 105_000);
    drop(_answers);
    let _answers = crate::tests::support::client::income_pages::source_answers(vec![
        crate::DocumentAnswer::UndistributedGainSpecial { value: true },
    ]);
    accepted(&save_fund_with_answers(&mut s, special));
    assert!(noted(&supported(&s.draft), "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT"));
    // The page rules.
    rejected_with(
        save(
            &mut s,
            TaxSection::InterestDividends,
            vec![IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Form2439(form_2439(
                    500_000, 300_000, 100_000, 100_001, 0,
                )),
            }],
        ),
        "FORM_2439_PARTS_ABOVE_1A",
    );
    rejected_with(
        save(
            &mut s,
            TaxSection::InterestDividends,
            vec![IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Form2439(form_2439(
                    500_000, 0, 0, 0, 500_001,
                )),
            }],
        ),
        "FORM_2439_TAX_ABOVE_1A",
    );
}

// ---- F11 and F12: the business and farm records ----

#[test]
fn business_page_amortizes_a_section_197_intangible() {
    // Goodwill of 150,000 acquired with the business, which started on 2024-07-01: 180 months from
    // July 2024. Months through 2025: 18 (July 2024 to December 2025); through 2026: 30. The 2026
    // amortization is 150,000 x 12 / 180 = 10,000; 150,000 - 150,000 x 18 / 180 = 135,000 remained at
    // the start of 2026, so 125,000 remain after it.
    let mut s = single();
    let mut b = business(5_000_000);
    b.more.intangibles = vec![
        crate::tests::support::builders::ordinary_intangible(
            15_000_000, 20240701, 20240701, 1_500_000,
        )
        .into(),
    ];
    accepted(&save_businesses(&mut s, vec![b.clone()]));
    let (id, stored) = stored_business(&s);
    b.more.intangibles = stored.intangibles.iter().cloned().map(Into::into).collect();
    b.more.intangible_predecessor_choices =
        crate::amortization_records::predecessor_choices(&stored);
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("business_{id}_intangible_0_amortization")), 10_000);
    assert_eq!(value(&r, &format!("business_{id}_intangible_0_remaining_basis")), 125_000);
    // Profit 50,000 - 10,000 amortization = 40,000 on top of the wages.
    let mut without = b.clone();
    without.more.intangibles = vec![];
    let mut plain = single();
    accepted(&save_businesses(&mut plain, vec![without]));
    let baseline = supported(&plain.draft);
    // AGI falls by the 10,000 less the half of the self-employment tax the smaller profit saves:
    // 10,000 x 92.35% x 15.3% = 1,413 of tax, half 706.50, so 10,000 - 707 = 9,293.
    assert_eq!(value(&baseline, "agi") - value(&r, "agi"), 9_293);
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::SelfEmployment))[0].document,
        IncomeSource::Record { value: crate::IncomeRecord::Business(b.clone()) }
    );
    // 2026-10-23: app-contract Answers and unknowns keeps the activity and
    // omits only the unresolved benefit; the accepted origin remains unknown.
    let mut unsure = b.clone();
    unsure.more.intangibles[0].source.first_existed = None;
    accepted(&crate::tests::support::client::cohorts::save_existing_business(&mut s, id, unsure));
    let unresolved = supported(&s.draft);
    assert!(noted(&unresolved, "WARN_INTANGIBLE_BENEFIT_LEFT_OUT"));
    assert_eq!(value(&unresolved, "agi"), value(&baseline, "agi"));
}

#[test]
fn farm_page_takes_the_fuel_credit_and_a_cash_fuel_recovery() {
    // Farm diesel, 2,000 gallons at 24.3 cents: a Form 4136 credit of 486 on Schedule 3 line 12. A
    // cash-method farm includes an earlier year's recovery (400) in Schedule F other income.
    let mut s = single();
    let mut farm = cash_farm(2_000_000, 500_000);
    farm.more.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            diesel_farm: Some(fuel_use(2_000_000, 480_000)),
            ..FederalFuelLines::default()
        }
        .into(),
        expense_treatment: FederalFuelExpenseTreatment::FullCostDeducted,
        records_complete: true,
    });
    farm.more.cash_fuel_tax_recovery = Some(cash_fuel_recovery(40_000));
    let request = farm_save(
        &mut s,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm.clone()) })],
    );
    accepted(&send(&mut s, request));
    let (id, data) = s
        .draft
        .income
        .iter()
        .find_map(|e| match &e.income {
            FederalIncome::Business { data } => Some((e.id, data.clone())),
            _ => None,
        })
        .unwrap();
    assert_eq!(data.fuel_tax_credit, farm.more.fuel_tax_credit);
    assert_eq!(data.cash_fuel_tax_recovery, farm.more.cash_fuel_tax_recovery);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "fuel_tax_credit"), 486);
    assert_eq!(value(&r, &format!("business_{id}_cash_fuel_tax_recovery")), 400);
    let documents = reopened(&mut s, TaxSection::Farming);
    assert_eq!(documents, vec![IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }]);
}

// ---- Search, offered kinds and import mappings ----

#[test]
fn investment_sales_and_interest_dividends_offer_the_new_kinds_and_search_names_them() {
    let mut s = single();
    let sales = page_kinds(&mut s, TaxSection::InvestmentSales);
    for kind in [
        libertas_macros::variant_index!(crate::IncomeSourceKind::EmployeeShareSale),
        libertas_macros::variant_index!(crate::IncomeSourceKind::EsppShareSale),
        libertas_macros::variant_index!(crate::IncomeSourceKind::NuaShareSale),
        libertas_macros::variant_index!(crate::IncomeSourceKind::QsbsSale),
        libertas_macros::variant_index!(crate::IncomeSourceKind::SmallBusinessStockLoss),
        libertas_macros::variant_index!(crate::IncomeSourceKind::WorthlessSecurity),
        libertas_macros::variant_index!(crate::IncomeSourceKind::BadDebt),
    ] {
        assert!(sales.contains(&(kind as i32)), "{kind}");
    }
    let dividends = page_kinds(&mut s, TaxSection::InterestDividends);
    for kind in [
        libertas_macros::variant_index!(crate::IncomeSourceKind::Div1099),
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form2439),
    ] {
        assert!(dividends.contains(&(kind as i32)), "{kind}");
    }
    let sources = include_str!("../../federal/income_editor.rs");
    assert!(!sources.contains("CapitalReturn1099Div"));
    let protocol = include_str!("../../core/shared/navigation.rs");
    for text in [
        "Employee stock: shares from nonqualified stock options, restricted stock and RSUs (Form W-2 box 12 code V)",
        "employee stock purchase plan (ESPP) shares (Form 3922, qualifying and disqualifying dispositions)",
        "net unrealized appreciation, NUA, Form 1099-R box 6",
        "Qualified small business stock (QSBS, section 1202, Form 8949 code Q)",
        "Small business stock losses (section 1244, Form 4797)",
        "Worthless securities and nonbusiness bad debts",
        "Nondividend distributions and return of capital (Form 1099-DIV box 3), liquidating distributions (boxes 9 and 10) and undistributed long-term capital gains (Form 2439",
        "Amortization of goodwill and other section 197 intangibles bought with a business (Form 4562-B), and the federal fuel tax credit (Form 4136)",
        "The federal fuel tax credit for off-highway farm fuel (Form 4136)",
    ] {
        let wanted =
            text.replace("net unrealized appreciation, NUA", "net unrealized appreciation (NUA");
        assert!(protocol.contains(&wanted) || protocol.contains(text), "{text}");
    }
}

fn page_kinds(s: &mut Interview, topic: TaxSection) -> Vec<i32> {
    let P::SaveDocuments { allowed_documents: mut kinds, .. } = open_documents(s) else {
        unreachable!()
    };
    let P::SaveIncomes { allowed_documents, .. } = open_topic(s, topic) else { unreachable!() };
    kinds.extend(allowed_documents);
    kinds
}

#[test]
fn the_stock_compensation_notice_and_help_name_the_four_kinds() {
    let notice = crate::interview::resource_text("WARN_SALE_STOCK_COMPENSATION_LEFT_OUT");
    for kind in
        ["Employee stock shares sold", "ESPP shares sold", "ISO shares sold", "NUA shares sold"]
    {
        assert!(notice.contains(kind), "{kind}");
    }
    let model = include_str!("../../investment/capital_sale_model.rs");
    assert!(model.contains("Wages or stock awards whose compensation isn't in a W-2 or isn't resolved. Ordinary employee shares have their own entries on this page."));
}

// The names of a page type's import targets: its fields marked for the AI import.
fn import_targets(source: &str, page: &str) -> Vec<String> {
    let at = source.find(&format!("pub struct {page} {{")).unwrap_or_else(|| panic!("{page}"));
    let body = &source[at..];
    let body = &body[..body.find("\n}").unwrap()];
    body.lines()
        .scan(false, |marked, line| {
            let text = line.trim();
            if text.starts_with("#[libertas_llm_doc_field") {
                *marked = true;
                return Some(None);
            }
            if *marked && !text.starts_with("#[") && !text.starts_with("//") {
                *marked = false;
                return Some(
                    text.trim_start_matches("pub ").split_once(':').map(|(n, _)| String::from(n)),
                );
            }
            Some(None)
        })
        .flatten()
        .collect()
}
// The instruction of the page's import action.
fn import_action<'a>(source: &'a str, page: &str) -> &'a str {
    let head = &source[..source.find(&format!("pub struct {page} {{")).unwrap()];
    &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..]
}

#[test]
fn share_sale_imports_read_the_1099_b_and_its_plan_record() {
    // Corrected 2026-10-31: the employer's stock plan record printed with the 1099-B shows the
    // date and value taxed as wages, the Form 3921 boxes 1 to 4 and the distribution value and
    // net unrealized appreciation, and the ESPP purchase facts, so the import reads them from
    // that record (Pub. 525, Stock Options and Employee Stock Purchase Plan; Form 3921
    // instructions; Pub. 575, Net unrealized appreciation).
    let employee = include_str!("../../income/employee_stock.rs");
    let action = import_action(employee, "EmployeeShareSale");
    assert!(action.contains("Read the 1099-B sale of these employee stock shares exactly"));
    // Box 1e and box 12 are printed facts of the 1099-B: the broker's basis
    // and whether it reported it to the IRS decide the Form 8949 box and code B.
    assert!(action.contains("Return box 1e as reported_basis exactly as printed"));
    assert!(action.contains("return box 12 as reporting."));
    assert!(action.contains("Read taxed, value and paid from the employer's stock plan record"));
    assert!(action.contains("When no such record is supplied, report an error"));
    assert!(action.contains("validation = \"$.document_facts\""));
    let sale = [
        "calendar_year",
        "source_issuer",
        "source_recipient",
        "account_number",
        "description",
        "box_1b_acquired",
        "sold",
        "proceeds",
        "reported_basis",
        "reporting",
        "form_8949_checkbox",
        "box_2_short_term",
        "box_2_long_term",
        "box_2_ordinary",
        "withholding",
    ];
    assert_eq!(
        import_targets(employee, "EmployeeShareSale"),
        [&sale[..], &["taxed", "value", "paid"]].concat()
    );
    let iso = include_str!("../../income/iso_sale.rs");
    let action = import_action(iso, "IsoShareSale");
    assert!(action.contains("Read granted, exercised, exercise_price and market_price"));
    assert_eq!(
        import_targets(iso, "IsoShareSale"),
        [
            "calendar_year",
            "source_issuer",
            "source_recipient",
            "account_number",
            "description",
            "box_1b_acquired",
            "shares",
            "sold",
            "proceeds",
            "reported_basis",
            "reporting",
            "form_8949_checkbox",
            "box_2_short_term",
            "box_2_long_term",
            "box_2_ordinary",
            "withholding",
            "granted",
            "exercised",
            "exercise_price",
            "market_price"
        ]
    );
    let espp = include_str!("../../income/espp_sale.rs");
    let action = import_action(espp, "EsppShareSale");
    assert!(action.contains("Do not return Form 3922 values even when a Form 3922 is supplied"));
    assert_eq!(
        import_targets(espp, "EsppShareSale"),
        [
            "calendar_year",
            "source_issuer",
            "source_recipient",
            "account_number",
            "description",
            "box_1b_acquired",
            "shares",
            "sold",
            "proceeds",
            "reported_basis",
            "reporting",
            "form_8949_checkbox",
            "box_2_short_term",
            "box_2_long_term",
            "box_2_ordinary",
            "withholding",
            "plan_record"
        ]
    );
    assert!(action.contains("Return null for plan_record when no such record is supplied"));
    assert_eq!(
        import_targets(include_str!("../../income/espp_source.rs"), "EsppPlanRecord"),
        ["granted", "exercised", "grant_market", "market_price", "exercise_price", "grant_price"]
    );
    let nua = include_str!("../../retirement/nua_sale.rs");
    let action = import_action(nua, "NuaShareSale");
    assert!(
        action.contains("do not return Form 1099-R values even when a Form 1099-R is supplied")
    );
    assert!(action.contains("Read distributed, distribution_value and deferred_nua"));
    assert_eq!(
        import_targets(nua, "NuaShareSale"),
        [&sale[..], &["distributed", "distribution_value", "deferred_nua"]].concat()
    );
    // The answers no paper prints stay with the person when an import replaces the targets.
    let mut page = rsu_sale(20260301);
    page.more.special = true;
    let kept = page.more.clone();
    (page.sold, page.proceeds, page.taxed, page.value) = (20260315, 2_600_000, 20250301, 1_900_000);
    assert_eq!(page.more, kept);
}

// Synthetic stock-plan papers: page 1 of each is a Form 1099-B of one sale of 100 shares for
// $5,000 on 2026-10-01; page 2 is the employer's or plan's record of the same shares, quoted here
// as printed, with the fields its lines fill.
// A printed record line and the import target it fills.
type RecordLine = (&'static str, &'static str);
const STOCK_PLAN_RECORDS: [(&str, &str, &[RecordLine]); 4] = [
    (
        "EmployeeShareSale",
        "Employee stock plan transaction record — EMPLOYEE",
        &[
            ("Award vested / taxed as wages: 2025-03-15", "taxed"),
            ("Value of these 100 shares taxed as wages: 4000.00", "value"),
            ("Amount employee paid: 0.00", "paid"),
        ],
    ),
    (
        "IsoShareSale",
        "Employee stock plan sale lot record — ISO",
        &[
            ("Shares in this sold lot: 100.000000", "shares"),
            ("Option grant date: 2022-06-15", "granted"),
            ("Option exercise date: 2026-05-15", "exercised"),
            ("Exercise price per share: 12.50", "exercise_price"),
            ("Exercise-date market value per share: 24.75", "market_price"),
        ],
    ),
    (
        "EsppPlanRecord",
        "Employee stock plan transaction record — ESPP",
        &[
            ("Option grant date: 2024-01-01", "granted"),
            ("Purchase / exercise date: 2024-06-30", "exercised"),
            ("Grant-date market value per share: 30.00", "grant_market"),
            ("Exercise-date market value per share: 35.00", "market_price"),
            ("Price paid per share: 25.50", "exercise_price"),
            ("Grant-date purchase price per share: 25.50", "grant_price"),
        ],
    ),
    (
        "NuaShareSale",
        "Employee stock plan transaction record — NUA",
        &[
            ("Plan distribution date: 2026-06-01", "distributed"),
            ("Distribution-date value of these shares: 4000.00", "distribution_value"),
            ("Net unrealized appreciation attributable to these shares: 3000.00", "deferred_nua"),
        ],
    ),
];

#[test]
fn a_1099_b_with_its_stock_plan_record_is_one_document_of_the_stock_plan_kind() {
    // The classifier keeps both pages in one document of the stock-plan kind and never adds the
    // 1099-B page as a plain Form 1099-B, so each $5,000 sale is entered once.
    let center = include_str!("../../federal/document_center.rs");
    let head = &center[..center.find("pub struct InvestmentSaleDocuments {").unwrap()];
    let classifier = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    for rule in [
        "A Form 1099-B together with the employer's or plan's stock-plan record for the same shares",
        "is one document of that specialized stock-sale kind",
        "list the 1099-B pages and the record pages together as its parts",
        "never list that 1099-B again as Form1099B or the record as a document of its own",
        "Identify every other actual Form 1099-B",
        // A lone 1099-B page, with no plan record, stays a plain Form 1099-B; Investment sales
        // asks about stock compensation missing from its basis.
        "A Form 1099-B with no such record is a plain Form1099B, even when its description names employee, option or plan shares.",
    ] {
        assert!(classifier.contains(rule), "{rule}");
    }
    // Each kind's name tells the model the record belongs to it.
    let kinds = include_str!("../../federal/income_editor.rs");
    for kind in [
        "sold in 2026, with their 1099-B and the employer's stock plan record, including",
        "a stock award, sold in 2026, with their 1099-B and the employer's stock plan record.",
        "sold in 2026, with their 1099-B and the employer's stock plan record or Form 3922.",
        "sold in 2026, with their 1099-B and the plan's record of the distributed shares.",
    ] {
        assert!(kinds.contains(kind), "{kind}");
    }
    // Every value the record prints fills an import target of its kind.
    for (kind, title, lines) in STOCK_PLAN_RECORDS {
        let source = match kind {
            "EmployeeShareSale" => include_str!("../../income/employee_stock.rs"),
            "IsoShareSale" => include_str!("../../income/iso_sale.rs"),
            "EsppPlanRecord" => include_str!("../../income/espp_source.rs"),
            _ => include_str!("../../retirement/nua_sale.rs"),
        };
        let targets = import_targets(source, kind);
        for (line, field) in lines {
            assert!(targets.iter().any(|t| t == field), "{title}: {line} -> {field}");
        }
    }
}

#[test]
fn the_stock_plan_papers_import_once_each() {
    // The employee, ISO and NUA papers as the import returns them, saved as one row each. Single,
    // W-2 wages $60,000.
    // Employee (box E, basis not reported): basis 0 paid + 4,000 taxed as wages = 4,000; held from
    //   2025-03-15, long-term gain 1,000.
    // ISO (box B): exercised 2026-05-15 and sold within a year, a disqualifying disposition;
    //   compensation min(100 x (24.75 - 12.50), 5,000 - 1,250) = 1,225, already in the W-2; basis
    //   1,250 + 1,225 = 2,475, short-term gain 2,525.
    // NUA (box B): the 3,000 NUA is long-term; basis 4,000 - 3,000 = 1,000; the 1,000 gain after
    //   the 2026-06-01 distribution is short-term.
    // AGI 60,000 + 1,000 + 2,525 + 3,000 + 1,000 = 67,525: each sale's $5,000 counts once.
    let employee = EmployeeShareSale {
        calendar_year: Default::default(),
        account_number: Default::default(),
        box_1b_acquired: Default::default(),
        form_8949_checkbox: Default::default(),
        box_2_short_term: Default::default(),
        box_2_long_term: Default::default(),
        box_2_ordinary: Default::default(),
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        description: "100 sh. Example Orchard Design restricted stock units".into(),
        sold: 20261001,
        proceeds: 500_000,
        withholding: 0,
        reported_basis: Some(0),
        reporting: BasisReporting::NotReported,
        taxed: 20250315,
        value: 400_000,
        paid: 0,
        more: EmployeeShareSaleMore::default(),
    };
    let iso = crate::IsoShareSale {
        calendar_year: Default::default(),
        account_number: Default::default(),
        box_1b_acquired: Default::default(),
        form_8949_checkbox: Default::default(),
        box_2_short_term: Default::default(),
        box_2_long_term: Default::default(),
        box_2_ordinary: Default::default(),
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        description: "100 sh. Example Orchard Systems Inc ISO common".into(),
        shares: 100.0,
        sold: 20261001,
        proceeds: 500_000,
        withholding: 0,
        reported_basis: Some(125_000),
        reporting: BasisReporting::NotReported,
        granted: 20220615,
        exercised: 20260515,
        exercise_price: 1_250,
        market_price: 2_475,
    };
    let nua = NuaShareSale {
        calendar_year: Default::default(),
        account_number: Default::default(),
        box_1b_acquired: Default::default(),
        form_8949_checkbox: Default::default(),
        box_2_short_term: Default::default(),
        box_2_long_term: Default::default(),
        box_2_ordinary: Default::default(),
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        description: "100 sh. Example Orchard Design employer plan shares".into(),
        sold: 20261001,
        proceeds: 500_000,
        withholding: 0,
        reported_basis: Some(100_000),
        reporting: BasisReporting::NotReported,
        distributed: 20260601,
        distribution_value: 400_000,
        deferred_nua: 300_000,
    };
    let mut s = single();
    accepted(&investment(
        &mut s,
        [
            crate::InvestmentSaleDocument::EmployeeShareSale(employee),
            crate::InvestmentSaleDocument::IsoShareSale(iso),
            crate::InvestmentSaleDocument::NuaShareSale(nua),
        ]
        .into_iter()
        .map(|value| IncomeSource::InvestmentSale { value })
        .collect(),
    ));
    // The plan sales' answers are given on Investment sales: the W-2
    // includes the ISO compensation; the NUA shares are the person's own.
    accepted(&crate::tests::support::client::income_pages::answer_iso_sale(
        &mut s,
        &crate::IsoShareSaleMore {
            selling_fees: 0,
            acquisition_costs: 0,
            option_cost: 0,
            special: false,
        },
        true,
    ));
    accepted(&answer_nua(&mut s, no_nua_adjustments(), crate::NuaOwnership::Own));
    let sales = s.draft.income.iter().filter(|e| {
        matches!(
            e.income,
            FederalIncome::EmployeeStockSale { .. }
                | FederalIncome::IsoSaleSource { .. }
                | FederalIncome::NuaSaleSource { .. }
                | FederalIncome::Broker { .. }
        )
    });
    assert_eq!(sales.count(), 3, "one entry for each paper");
    let r = supported(&s.draft);
    let id = |wanted: fn(&FederalIncome) -> bool| id_of(&s, wanted);
    let employee = id(|i| matches!(i, FederalIncome::EmployeeStockSale { .. }));
    assert_eq!(value(&r, &format!("employee_stock_{employee}_basis")), 4_000);
    assert_eq!(value(&r, &format!("employee_stock_{employee}_capital_gain")), 1_000);
    let iso = id(|i| matches!(i, FederalIncome::IsoSaleSource { .. }));
    assert_eq!(value(&r, &format!("iso_sale_{iso}_compensation")), 1_225);
    assert_eq!(value(&r, &format!("iso_sale_{iso}_regular_basis")), 2_475);
    assert_eq!(value(&r, &format!("iso_sale_{iso}_capital_gain")), 2_525);
    let nua = id(|i| matches!(i, FederalIncome::NuaSaleSource { .. }));
    assert_eq!(value(&r, &format!("nua_sale_{nua}_realized_nua")), 3_000);
    assert_eq!(value(&r, &format!("nua_sale_{nua}_post_distribution_gain")), 1_000);
    assert_eq!(value(&r, "agi"), 67_525);
}

#[test]
fn qsbs_import_reads_the_sale_rows_and_leaves_the_issuer_answers() {
    let source = include_str!("../../investment/qsbs.rs");
    let action = import_action(source, "QsbsSale");
    assert!(action.contains("Return one lots row for each sale of that corporation's shares"));
    assert!(action.contains("If the document shows sales of more than one corporation's stock"));
    assert!(action.contains("do not decide whether the stock qualifies"));
    assert_eq!(
        import_targets(source, "QsbsSale"),
        [
            "calendar_year",
            "source_issuer",
            "source_recipient",
            "issuer",
            "lots",
            "reporting",
            "withholding"
        ]
    );
    assert_eq!(
        import_targets(source, "QsbsPaperLot"),
        ["lot_identifier", "shares", "issued", "sold", "basis", "proceeds"]
    );
}

#[test]
fn return_of_capital_and_form_2439_imports_read_the_printed_boxes() {
    let model = crate::federal_model::HUB_SOURCE;
    let lot_source = include_str!("../../investment/dividend_source.rs");
    assert!(import_unselected(lot_source, "pub struct CapitalReturn1099Div {"));
    let action = import_action(model, "Form2439");
    assert!(action.contains("box 1a total undistributed long-term capital gains"));
    assert!(action.contains("box 2 tax paid"));
    assert_eq!(
        import_targets(model, "Form2439"),
        [
            "fund",
            "shareholder",
            "gain",
            "unrecaptured",
            "small_business",
            "collectibles",
            "tax_paid"
        ]
    );
    // The normal1099-DIV import now retains boxes3/9/10 as printed totals,
    // while its personal lot facts are not an import target.
    let div = import_action(model, "Form1099DivPaper");
    assert!(div.contains("3 nondividend distributions"));
    assert!(div.contains("9 cash liquidation distributions"));
    assert!(div.contains("10 noncash liquidation distributions"));
    assert!(div.contains("Return null for each amount box that is clearly blank"));
    assert!(
        import_targets(model, "Form1099DivPaper").iter().all(|field| field != "capital_returns")
    );
    assert!(div.contains(
        "Do not infer share-lot basis, holding period, liquidation stage or property identity"
    ));
    assert!(div.contains("2c section 1202 gain into section_1202_gain"));
}

// ---- Year constants ----

#[test]
fn small_business_stock_limits_and_dates_are_year_constants() {
    use crate::year_constants as yc;
    assert_eq!(yc::SECTION_1244_ORDINARY_LOSS_LIMIT_CENTS, 5_000_000);
    assert_eq!(yc::SECTION_1244_ORDINARY_LOSS_LIMIT_JOINT_CENTS, 10_000_000);
    assert_eq!(yc::SECTION_1202_ISSUER_LIMIT_CENTS, 1_000_000_000);
    assert_eq!(yc::SECTION_1202_ISSUER_LIMIT_SEPARATE_CENTS, 500_000_000);
    assert_eq!(yc::SECTION_1202_FIRST_ISSUE_DATE, 19930811);
    assert_eq!(yc::SECTION_1202_LAST_ORIGINAL_TIER_ISSUE_DATE, 20250704);
    assert_eq!(yc::SECTION_1202_FIFTY_PERCENT_LAST_DATE, 20090217);
    assert_eq!(yc::SECTION_1202_SEVENTY_FIVE_PERCENT_LAST_DATE, 20100927);
    assert_eq!(yc::SECTION_1202_AMT_PREFERENCE_PERCENT, 7);
    // The call sites hold none of the literals.
    for (name, source) in [
        ("qsbs.rs", include_str!("../../investment/qsbs.rs")),
        ("stock_loss.rs", include_str!("../../investment/stock_loss.rs")),
    ] {
        let code: String = source
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for literal in
            ["19930811", "20250704", "20090217", "20100927", "500_000_000", "1_000_000_000"]
        {
            assert!(!code.contains(literal), "{name} holds {literal}");
        }
    }
    // The $15,000,000 cap and the three-year tier cannot apply to a 2026 sale: the earliest sale of
    // stock issued after July 4, 2025 is July 5, 2028.
}

// ---- Previews, and the ISO sale date's client twin ----

fn preview(s: &mut Interview, document: IncomeSource) -> Option<i64> {
    crate::tests::support::client::income_pages::saved_document_amount(s, document)
}

#[test]
fn amount_on_the_return_previews_the_supported_projection_of_each_new_kind() {
    let mut s = single();
    // The sale's gain, not its wage compensation.
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(rsu_sale(20260301))
            }
        ),
        Some(500_000)
    );
    // A QSBS issuer's amount depends on every entry of the issuer.
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::QsbsSale(
                    crate::tests::support::builders::qsbs_document(
                        &(qsbs_page(20080602, 2_000_000, 12_000_000, BasisReporting::NoForm1099B))
                    )
                )
            }
        ),
        None
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::Record {
                value: crate::IncomeRecord::SmallBusinessStockLoss(section_1244_cash(
                    1_000_000, 200_000, 900_000
                ))
            }
        ),
        Some(-300_000)
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::Record {
                value: crate::IncomeRecord::WorthlessSecurity(worthless(500_000, 20250615))
            }
        ),
        Some(-500_000)
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::Record {
                value: crate::IncomeRecord::BadDebt(cash_loan_debt(1_000_000, 200_000))
            }
        ),
        Some(-800_000)
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(capital_return_document(
                    Default::default(),
                    Default::default(),
                    capital_return(CapitalDistributionKind::Nondividend, 300_000, 200_000),
                ))
            }
        ),
        Some(100_000)
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Form2439(form_2439(
                    500_000, 0, 0, 300_000, 105_000
                ))
            }
        ),
        Some(500_000)
    );
    // A document left out of the estimate previews zero.
    let mut left_out = rsu_sale(20260301);
    left_out.more.special = true;
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::EmployeeShareSale(left_out)
            }
        ),
        Some(0)
    );
    assert_eq!(
        {
            let _answers = crate::tests::support::client::income_pages::source_answers(vec![
                crate::DocumentAnswer::DividendFundExclusion {
                    value: Some(crate::FundQsbsExclusion::Hundred),
                },
            ]);
            preview(
                &mut s,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Form2439(form_2439(
                        500_000, 0, 100_000, 0, 105_000,
                    )),
                },
            )
        },
        Some(400_000)
    );
    let mut unsure = worthless(500_000, 20250615);
    unsure.worthless = false;
    assert_eq!(
        preview(
            &mut s,
            IncomeSource::Record { value: crate::IncomeRecord::WorthlessSecurity(unsure) }
        ),
        Some(0)
    );
}

#[test]
fn unnamed_investment_entries_are_named_by_their_kind_and_identity() {
    for (key, text) in [
        ("LABEL_ENTRY_EMPLOYEE_SALE", "Employee shares sold {0,number,0}"),
        ("LABEL_ENTRY_ESPP_SALE", "ESPP shares sold {0,number,0}"),
        ("LABEL_ENTRY_NUA_SALE", "NUA shares sold {0,number,0}"),
        ("LABEL_ENTRY_SECTION_1244_LOSS", "Section 1244 stock loss {0,number,0}"),
        ("LABEL_ENTRY_WORTHLESS_SECURITY", "Worthless security {0,number,0}"),
        ("LABEL_ENTRY_BAD_DEBT", "Bad debt {0,number,0}"),
        ("LABEL_ENTRY_FORM_2439", "Form 2439 {0,number,0}"),
    ] {
        assert_eq!(crate::interview::resource_text(key), text);
    }
}

#[test]
fn a_1099_div_with_foreign_tax_round_trips_beside_a_return_of_capital() {
    let mut s = single();
    let mut div = div_1099(50_000);
    div.foreign_tax = 1_200;
    let capital = IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Div1099(capital_return_document(
            Default::default(),
            Default::default(),
            capital_return(CapitalDistributionKind::Nondividend, 300_000, 200_000),
        )),
    };
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(dividend_document(div.clone())),
            },
            capital.clone(),
        ],
    ));

    let rows = reopened(&mut s, TaxSection::InterestDividends);
    let expected = expected_named_sources(
        &s.draft,
        Owner::Taxpayer,
        vec![
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Div1099(dividend_document(div)),
            },
            capital,
        ],
    );
    for document in expected {
        assert!(rows.contains(&document));
    }
    round_trips(&mut s, TaxSection::InterestDividends);
}

#[test]
fn section_1202_and_1244_issues_point_at_investment_sales() {
    // F3 (2026-10-09): every fact these issues depend on is on the Investment
    // sales page, so their correction link is that page.
    let target = IssueContext::Section(TaxSection::InvestmentSales);
    let issue_context = |r: &FederalResult, section: &str| {
        r.issues
            .iter()
            .find(|issue| issue.section.contains(section))
            .unwrap_or_else(|| panic!("{section}: {:?}", r.issues))
            .context
            .clone()
    };
    // Section 1244: a short-term and a long-term loss together above the
    // $50,000 ordinary allowance need an allocation the App does not make.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(section1244_loss(40_000, 0, 20200101)));
    add(&mut d, income(section1244_loss(40_000, 0, 20260301)));
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{:?}", r.issues);
    assert_eq!(issue_context(&r, "Section 1244"), target);
    // Section 1202: two documents for one issuer with different prior
    // eligible gains.
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000_000, 710_000_000, 0));
    let mut spouse = qsb_income("Synthetic A Inc.", 10_000_000, 710_000_000, 100_000_000);
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = estimate(&d);
    assert_eq!(issue_context(&r, "Section 1202 issuer history"), target);
}

#[test]
fn a_2026_section_197_intangible_does_not_make_the_business_file_form_4562() {
    // F3 (2026-10-09): the 2026 Form 4562 has no Part VI; amortization moves
    // to Form 4562-B, so a section 197 intangible acquired in 2026 does not by
    // itself bring the business's other depreciation rows onto Form 4562.
    let mut s = single();
    let mut b = business(5_000_000);
    b.more.intangibles = vec![
        crate::tests::support::builders::ordinary_intangible(15_000_000, 20260201, 20260201, 0)
            .into(),
    ];
    accepted(&save_businesses(&mut s, vec![b]));
    let (_, stored) = stored_business(&s);
    assert_eq!(stored.intangibles.len(), 1);
    assert!(!crate::depreciation::assets_need_form_4562(&stored.assets));
    supported(&s.draft);
}

// §165 investment loss retains its actual holding period when only §1244 character is unavailable.
// Derived integration probe: $10,000 documented basis less $2,000 proceeds = $8,000 long-term
// loss; ordinary wages $60,000, annual capital-loss deduction $3,000, AGI $57,000.
#[test]
fn documented_nonoriginal_stock_loss_keeps_capital_character_on_public_page() {
    let mut s = single();
    let mut stock = section_1244_cash(1_000_000, 0, 200_000);
    stock.ownership = crate::StockLossOwnership::CapitalInvestment;
    accepted(&investment(
        &mut s,
        vec![IncomeSource::Record { value: crate::IncomeRecord::SmallBusinessStockLoss(stock) }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 0);
    assert_eq!(value(&r, "capital_gain"), -3_000);
    assert_eq!(value(&r, "agi"), 57_000);
    assert!(!noted(&r, "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT"));
    round_trips(&mut s, TaxSection::InvestmentSales);
}

// Explicit full §267 disallowance is zero. Missing/unmodeled basis or restriction history leaves
// only this loss unclaimed with a source notice; neither affects independently known wage income.
#[test]
fn stock_loss_restrictions_preserve_known_tax_and_accepted_history() {
    for ownership in [
        crate::StockLossOwnership::MissingBasis,
        crate::StockLossOwnership::WashSaleRestriction,
        crate::StockLossOwnership::OtherUnmodeledHistory,
        crate::StockLossOwnership::RelatedPartyRestriction,
    ] {
        let mut s = single();
        let mut stock = section_1244_cash(1_000_000, 0, 200_000);
        stock.ownership = ownership;
        accepted(&investment(
            &mut s,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::SmallBusinessStockLoss(stock),
            }],
        ));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "agi"), 60_000);
        assert_eq!(value(&r, "capital_gain"), 0);
        assert_eq!(
            noted(&r, "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT"),
            ownership != crate::StockLossOwnership::RelatedPartyRestriction
        );
        round_trips(&mut s, TaxSection::InvestmentSales);
    }
}

#[test]
fn liquidation_stage_pending_resolves_through_the_existing_topic_and_restart() {
    // An accepted native distribution has no separate paper kind. Its first
    // unchanged center save preserves the lot until the normal topic resolves it.
    let mut draft = complete(FederalStatus::Single);
    add(
        &mut draft,
        income(FederalIncome::ReturnOfCapital {
            kind: CapitalDistributionKind::Liquidation,
            amount: 120_000,
            property: vec![],
            basis: 200_000,
            term: GainTerm::Long,
            special: false,
        }),
    );
    let id = draft.income.last().unwrap().id;
    let mut session = Interview::new(TEST_USER, draft);
    let request = open_documents(&mut session);
    let rows = page_documents(&request);
    assert_eq!(rows.iter().filter(|row| row.id == id).count(), 1);
    let row = rows.iter().find(|row| row.id == id).unwrap();
    let IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(form) } =
        &row.document
    else {
        panic!("the native distribution reopens as its actual DIV")
    };
    assert_eq!(
        (
            form.nondividend_distributions.unwrap_or(0),
            form.cash_liquidation.unwrap_or(0),
            form.noncash_liquidation.unwrap_or(0),
            form.withholding.unwrap_or(0)
        ),
        (0, 120_000, 0, 0)
    );
    accepted(&send(&mut session, request));
    let result = estimate(&session.draft);
    assert!(result.tax.is_none());
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "DIVIDEND_LIQUIDATION_STAGE_NEEDED")
    );
    session = restart(session.draft);
    let mut request = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let row = followups
        .iter_mut()
        .find(|row| row.key == format!("income/{id}/capital_returns"))
        .expect("the unresolved stage links to its normal topic");
    let DocumentAnswer::DividendCapitalReturns { value: lots } = &mut row.answer else {
        unreachable!()
    };
    assert_eq!(lots, &vec![capital_return(CapitalDistributionKind::Liquidation, 120_000, 200_000)]);
    lots[0].kind = CapitalDistributionKind::FinalLiquidation;
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "capital_gain"), -800);
    session = restart(session.draft);
    assert_eq!(supported(&session.draft), result);
    assert!(
        !result.issues.iter().any(|issue| issue.display.key == "DIVIDEND_LIQUIDATION_STAGE_NEEDED"),
        "accepted lot facts remain editable, but no finality answer is owed"
    );
    let mut request = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row =
        documents.common.interest_dividends.documents.iter_mut().find(|row| row.id == id).unwrap();
    let crate::InterestDividendDocument::Div1099(source) = &mut row.document else {
        unreachable!()
    };
    source.cash_liquidation = Some(210_000);
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let row = followups
        .iter_mut()
        .find(|row| row.key == format!("income/{id}/capital_returns"))
        .expect("the changed paper reopens capital-lot reconciliation");
    let DocumentAnswer::DividendCapitalReturns { value: lots } = &mut row.answer else {
        unreachable!()
    };
    lots[0].kind = CapitalDistributionKind::Liquidation;
    lots[0].amount = 210_000;
    accepted(&send(&mut session, request));
    assert_eq!(value(&supported(&session.draft), "capital_gain"), 100);
    let result = supported(&session.draft);
    assert!(
        !result.issues.iter().any(|issue| issue.display.key == "DIVIDEND_LIQUIDATION_STAGE_NEEDED"),
        "a stage that cannot affect this return is not a required question"
    );
}

#[test]
fn smith_qsbs_closing_statement_classifies_as_qsbs_sale_without_printed_eligibility() {
    // A synthetic paying agent's "Stock sale cash closing statement", marked "not Form 1099-B",
    // prints the issuing corporation, lot, original issue
    // date, sale date, proceeds and basis but never "qualified small business stock" or
    // "section 1202". The classifier and the kind take it as QsbsSale on its printed sale alone.
    let center = include_str!("../../federal/document_center.rs");
    assert!(center.contains("Classify a paying agent's or closing statement of a sale of one corporation's stock that is not a Form 1099-B, such as a stock sale cash closing statement, as QsbsSale whether or not it names qualified small business stock or section 1202"));
    assert!(!center.contains("explicitly identifying qualified small business stock"));
    let kinds = include_str!("../../federal/income_editor.rs");
    let kind = &kinds[..kinds.find("    QsbsSale(crate::QsbsSale),").unwrap()];
    let kind = &kind[kind.rfind("    /// Stock sale closing statement").unwrap()..];
    assert!(
        kind.contains(
            "that is not a Form 1099-B, whether or not it names qualified small business stock"
        ),
        "{kind}"
    );
    let model = include_str!("../../investment/qsbs.rs");
    let action = import_action(model, "QsbsSale");
    assert!(action.contains("from a Form 1099-B or a paying agent's or closing statement exactly"));
    assert!(action.contains("NoForm1099B when the document is not a Form 1099-B"));

    // The paper's printed values as AI Import fills them, then the person's topic answers.
    let mut page = qsbs_page(20160301, 200_000, 1_200_000, BasisReporting::NoForm1099B);
    let source_issuer = crate::PrintedParty {
        name: "Example Seed Paying Agent".into(),
        federal_id: "00-0000608".into(),
        address: "608 Example Closing Lane, Example Grove VA 00000".into(),
    };

    page.issuer = "Example Seed Instruments Corporation".into();
    page.lots[0].sold = 20260901;
    let mut s = single();
    // The household's taxpayer stands in for the printed recipient John Smith.
    let source_recipient = crate::PrintedParty {
        name: crate::tests::support::builders::fixture_document_names(&s.draft, -1)
            .unwrap()
            .recipient
            .name,
        ..Default::default()
    };
    let mut paper = QsbsSale::from_record(&page);
    paper.source_issuer = source_issuer;
    paper.source_recipient = source_recipient;
    let paper = crate::tests::support::builders::remember_qsbs_answers(&page, paper);
    accepted(&investment(
        &mut s,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(paper),
        }],
    ));
    // Issued 2016-03-01 (after September 27, 2010), held over five years: 100% of the $10,000
    // gain is excluded.
    let id = id_of(&s, |i| matches!(i, FederalIncome::QualifiedSmallBusinessSale { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("qsbs_{id}_realized_gain")), 10_000);
    assert_eq!(value(&r, &format!("qsbs_{id}_excluded_gain")), 10_000);
    assert_eq!(value(&r, &format!("qsbs_{id}_taxable_gain")), 0);
}

#[test]
fn qsbs_strict_facts_follow_paper_and_survive_save_restart_and_corrections() {
    let mut source = qsbs_page(20080602, 100_000, 300_000, BasisReporting::NoForm1099B);
    source.lots.push(QsbStockLot {
        issued: 20150302,
        sold: 20260601,
        basis: 200_000,
        proceeds: 700_000,
        zone_gain_through_2018: None,
    });
    let mut session = single();
    // A raw printed paper has no registered personal fixture answers.
    let paper = QsbsSale::from_record(&source);
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(paper) },
        )],
    );
    let request = fixture_owner_inputs(&session.draft, request);
    accepted(&send(&mut session, request));
    let stored = qsbs_accepted_source(&session);
    assert_eq!(
        (stored.special, stored.issuer_qualified, stored.prior_eligible_gain),
        (None, None, None)
    );
    let mut result = FederalResult::empty(session.draft.revision);
    let native = crate::qsbs_return::calculate(&session.draft, &mut result);
    assert_eq!((native.taxable_gain, native.excluded_gain), (700_000, 0));
    assert!(crate::qsbs_followups::needed(&session.draft));
    let before = session.draft.clone();
    let unanswered = open_topic(&mut session, TaxSection::InvestmentSales);
    rejected_with(send(&mut session, unanswered), "QSBS_ANSWER_NEEDED");
    assert_eq!(session.draft, before);
    qsbs_bool_fact(&mut session, "qsbs_special", false);
    qsbs_bool_fact(&mut session, "qsbs_issuer_qualified", false);
    let stored = qsbs_accepted_source(&session);
    assert_eq!((stored.active_business, stored.prior_eligible_gain), (None, None));
    assert!(
        !crate::qsbs_followups::needed(&session.draft),
        "a disqualifying No ends the exclusion questions"
    );
    let mut result = FederalResult::empty(session.draft.revision);
    let native = crate::qsbs_return::calculate(&session.draft, &mut result);
    assert_eq!((native.taxable_gain, native.excluded_gain), (700_000, 0));
    qsbs_bool_fact(&mut session, "qsbs_issuer_qualified", true);
    for field in ["qsbs_active_business", "qsbs_no_disqualification", "qsbs_complete_history"] {
        qsbs_bool_fact(&mut session, field, true);
    }
    let mut host = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut host else { panic!() };
    let row = followups.iter_mut().find(|row| row.key.ends_with("qsbs_prior_gain")).unwrap();
    let crate::DocumentAnswer::QsbsPriorGain { value: target } = &mut row.answer else { panic!() };
    assert!(target.known.is_empty());
    let Reply::Response(P::QsbsPriorGainAnswered { value, known }) = send(
        &mut session,
        P::AnswerQsbsPriorGain { user: TEST_USER, prompt: row.prompt.clone(), value: 0 },
    ) else {
        panic!()
    };
    target.value = Some(value);
    target.known = known;
    // The separate zone record question is also initially unanswered.
    let zone = followups.iter_mut().find(|row| row.key.ends_with("qsbs_zone")).unwrap();
    let crate::DocumentAnswer::QsbsZone { value } = &mut zone.answer else { panic!() };
    assert!(value.known.is_empty());
    let Reply::Response(P::QsbsZoneAnswered { value: answer, known }) = send(
        &mut session,
        P::AnswerQsbsZone {
            user: TEST_USER,
            prompt: zone.prompt.clone(),
            value: crate::QsbsZoneCertification::NoRecords,
        },
    ) else {
        panic!()
    };
    value.value = Some(answer);
    value.known = known;
    accepted(&send(&mut session, host));
    assert!(!crate::qsbs_followups::needed(&session.draft));
    session = restart(session.draft.clone());
    let mut host = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut host else { panic!() };
    let zone = followups.iter_mut().find(|row| row.key.ends_with("qsbs_zone")).unwrap();
    let crate::DocumentAnswer::QsbsZone { value } = &mut zone.answer else { panic!() };
    assert_eq!(value.known, vec![crate::QsbsZoneCertification::NoRecords]);
    value.value = Some(crate::QsbsZoneCertification::Yes);
    value.known = vec![crate::QsbsZoneCertification::Yes];
    accepted(&send(&mut session, host));
    let mut host = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut host else { panic!() };
    let lot = followups.iter_mut().find(|row| row.key.ends_with("qsbs_zone_gain/0")).unwrap();
    let crate::DocumentAnswer::QsbsZoneGain { value } = &mut lot.answer else { panic!() };
    assert!(value.known.is_empty());
    let Reply::Response(P::QsbsValuationAnswered { value: answer, known }) = send(
        &mut session,
        P::AnswerQsbsValuation {
            lot: value.lot.clone(),
            user: TEST_USER,
            prompt: lot.prompt.clone(),
            value: crate::QsbsZoneValuation::NoRecords,
        },
    ) else {
        panic!()
    };
    value.value = Some(answer);
    value.known = known;
    accepted(&send(&mut session, host));
    let mut result = FederalResult::empty(session.draft.revision);
    let native = crate::qsbs_return::calculate(&session.draft, &mut result);
    assert_eq!((native.taxable_gain, native.excluded_gain), (100_000, 600_000));
    assert!(!crate::qsbs_followups::needed(&session.draft));
    // Economic corrections retain issuer qualification and unavailable records.
    let mut paper = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut paper else { panic!() };
    let crate::InvestmentSaleDocument::QsbsSale(value) =
        &mut documents.common.investment_sales.documents[0].document
    else {
        panic!()
    };
    value.lots[0].proceeds = 400_000;
    value.lots.reverse();
    value.issuer = "  SYNTHETIC ROBOTICS INC.  ".into();
    accepted(&send(&mut session, paper));
    let stored = qsbs_accepted_source(&session);
    assert_eq!(
        (stored.issuer_qualified, stored.active_business, stored.no_disqualification),
        (Some(true), Some(true), Some(true))
    );
    assert_eq!(stored.lots[1].zone_gain_through_2018, Some(crate::QsbsZoneValuation::NoRecords));
    assert!(!crate::qsbs_followups::needed(&session.draft));
    session = restart(session.draft.clone());
    let mut result = FederalResult::empty(session.draft.revision);
    let native = crate::qsbs_return::calculate(&session.draft, &mut result);
    assert_eq!((native.taxable_gain, native.excluded_gain), (150_000, 650_000));
    let mut paper = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut paper else { panic!() };
    let crate::InvestmentSaleDocument::QsbsSale(value) =
        &mut documents.common.investment_sales.documents[0].document
    else {
        panic!()
    };
    value.lots[1].issued = 20100927;
    accepted(&send(&mut session, paper));
    let stored = qsbs_accepted_source(&session);
    assert_eq!(
        (stored.issuer_qualified, stored.active_business, stored.no_disqualification),
        (None, None, None)
    );
    assert_eq!(
        (stored.special, stored.complete_history, stored.prior_eligible_gain),
        (Some(false), Some(true), Some(0))
    );
    assert!(stored.lots[1].zone_gain_through_2018.is_none());
    assert!(crate::qsbs_followups::needed(&session.draft));
    let mut result = FederalResult::empty(session.draft.revision);
    let native = crate::qsbs_return::calculate(&session.draft, &mut result);
    assert_eq!((native.taxable_gain, native.excluded_gain), (800_000, 0));
}

#[test]
fn qsbs_101_printed_lots_survive_public_save_reopen_restart() {
    let mut source = qsbs_page(20150302, 100_000, 300_000, BasisReporting::NoForm1099B);
    source.lots = vec![source.lots[0].clone(); 101];
    assert!(source.valid(), "101 printed lots have no tax-law or schema exclusion");
    let mut session = single();
    accepted(&investment(
        &mut session,
        vec![IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::QsbsSale(
                crate::tests::support::builders::qsbs_document(&source),
            ),
        }],
    ));
    assert_eq!(qsbs_accepted_source(&session).lots.len(), 101);
    let mut result = FederalResult::empty(session.draft.revision);
    let native = crate::qsbs_return::calculate(&session.draft, &mut result);
    // Each post-2010 lot gains $2,000, wholly excluded under the existing cap.
    assert_eq!((native.excluded_gain, native.taxable_gain), (20_200_000, 0));
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let paper = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &paper else { panic!() };
    let crate::InvestmentSaleDocument::QsbsSale(paper_sale) =
        &documents.common.investment_sales.documents[0].document
    else {
        panic!()
    };
    assert_eq!(paper_sale.lots.len(), 101);
    accepted(&send(&mut session, paper));
    assert_eq!(session.draft.income, saved.income);
    let mut result = FederalResult::empty(session.draft.revision);
    let reopened = crate::qsbs_return::calculate(&session.draft, &mut result);
    assert_eq!((reopened.excluded_gain, reopened.taxable_gain), (20_200_000, 0));
}
