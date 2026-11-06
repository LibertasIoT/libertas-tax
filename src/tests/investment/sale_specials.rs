// Unit B9-5: the investment sale specials of the design map tax-forms/investment-sale-specials.md.
// Options (a written option is the new holding, an option bought is an ordinary lot, an exercised
// option changes the stock lot), the empty basis, the Form 1099-DA sale and boxes G to L, Form 8949
// rows with columns (a), d-(h) and the codes B, C, D, E, L, M, N, T, W, O, Q, Y and Z, the
// Schedule D columns (d), e and (g), qualified opportunity funds (Form 8997) and a fund's
// section 1202 gain. Fixtures F1 to F10 of the map, each amount worked by hand in the comment
// beside it. Amounts are cents; Schedule D lines are whole dollars. Every amount is synthetic.
extern crate std;
use super::*;
use crate::tests::cross::interview::{document, open_topic, page_documents, rejected_with};
use crate::tests::support::client::income_pages::{
    open_document_center, save_documents, save_each_topic,
};
use crate::{
    Form2439, FundQsbsExclusion, QofDeferral, QofEvent, QofGainCode, QofInvestment, Sale1099B,
};
use alloc::string::{String, ToString};

fn single() -> FederalDraft {
    complete(FederalStatus::Single)
}

// A 1099-B lot: reported to the IRS, an investment, no adjustment.
fn add_sale(d: &mut FederalDraft, sale: FederalSale) -> i64 {
    add(d, income(FederalIncome::Sale { data: sale_source(sale) }));
    d.income.last().unwrap().id
}

fn added(d: &mut FederalDraft, data: FederalIncome) -> i64 {
    add(d, income(data));
    d.income.last().unwrap().id
}

fn row_value(r: &FederalResult, id: i64, row: u32, column: &str) -> i64 {
    r.lines
        .iter()
        .find(|l| l.id == alloc::format!("form_8949_{id}_{row}_{column}"))
        .unwrap_or_else(|| panic!("no Form 8949 row {id}.{row} {column}"))
        .amount
}

fn row_count(r: &FederalResult) -> usize {
    r.lines.iter().filter(|l| l.id.starts_with("form_8949_") && l.id.ends_with("_gain")).count()
}

// The codes and the box a row's explanation names.
fn row_text(r: &FederalResult, id: i64, row: u32) -> (String, String) {
    let line =
        r.lines.iter().find(|l| l.id == alloc::format!("form_8949_{id}_{row}_gain")).unwrap();
    let text = |index: usize| match line.explanation.args.get(index) {
        Some(crate::MessageArg::Text(text)) => text.clone(),
        _ => String::new(),
    };
    (text(1), text(2))
}

fn qof(invested_on: u32, short: i64, long: i64, event: QofEvent) -> QofInvestment {
    QofInvestment {
        fund_ein: Default::default(),
        source_recipient: Default::default(),
        fund: "Example Opportunity Fund".into(),
        invested_on,
        short_term: Some(short),
        long_term: Some(long),
        gain_code: QofGainCode::None,
        event: Some(event),
        special: false,
    }
}

#[test]
fn prior_year_8997_saves_printed_facts_before_the_current_year_event() {
    use crate::tests::cross::interview::accepted;
    use libertas::SchemaPath;

    let mut session = Interview::new(TEST_USER, single());
    let original_count = session.draft.income.len();
    let native =
        qof(20190601, 5_000_000, 15_000_000, QofEvent::Held { fair_value: Some(18_000_000) });
    // Part IV of the 2025 paper prints deferred gains, never a 2026 event.
    let paper = crate::QofInvestmentPaper::from(&native);
    let row = named_document(
        &session.draft,
        -1,
        0,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(paper),
        },
    );
    let request = save_documents(&mut session, vec![row]);
    let before_revision = session.draft.revision;
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, before_revision + 1);
    assert_eq!(session.draft.income.len(), original_count + 1);
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::OpportunityFund { .. }))
        .unwrap();
    let id = entry.id;
    let FederalIncome::OpportunityFund { data } = &entry.income else { unreachable!() };
    let printed = crate::QofInvestmentPaper::from(data);
    assert_eq!(data.event, None);
    assert_eq!((data.short_term, data.long_term), (Some(5_000_000), Some(15_000_000)));
    let saved = session.draft.clone();
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let page = open_document_center(&mut session);
    let reopened = page_documents(&page).into_iter().find(|row| row.id == id).unwrap();
    assert_eq!(
        reopened.document,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(printed.clone()),
        }
    );
    accepted(&send(&mut session, page));
    assert_eq!(session.draft, saved);

    let result = estimate(&session.draft);
    assert!(result.tax.is_none(), "unanswered is not a zero or held investment");
    assert!(result.issues.iter().any(|issue| issue.display.key == "QOF_EVENT_NEEDED"));
    let key = alloc::format!("income/{id}/event");
    // The strict event question is asked on the page's `questions`.
    let path = SchemaPath::root()
        .field("questions")
        .key(key.as_str())
        .field("answer")
        .field("value")
        .to_string();
    let items = crate::interview::remaining_items(&session.draft, &result, TaxSection::Review);
    assert!(items.iter().any(|item| item.section == TaxSection::InvestmentSales
        && item.field.as_deref() == Some(path.as_str())));
    let mut request = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let answer = followups.iter_mut().find(|row| row.key == key).unwrap();
    assert_eq!(answer.answer, crate::DocumentAnswer::OpportunityFundEvent { value: None });
    answer.answer = crate::DocumentAnswer::OpportunityFundEvent { value: native.event.clone() };
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("qof_{id}_included_short")), 37_500);
    assert_eq!(value(&result, &alloc::format!("qof_{id}_included_long")), 112_500);
    assert_eq!(
        value(&result, "capital_gain"),
        150_000,
        "the existing owner includes the gain once"
    );
    let answered = session.draft.clone();
    session = restart(answered.clone());
    let page = open_topic(&mut session, TaxSection::InvestmentSales);
    let P::SaveIncomes { followups, .. } = &page else { unreachable!() };
    assert!(followups.iter().any(|row| row.key == key
        && row.answer
            == crate::DocumentAnswer::OpportunityFundEvent { value: native.event.clone() }));
    accepted(&send(&mut session, page));
    assert_eq!(session.draft, answered);
    assert_eq!(supported(&session.draft).tax, result.tax);
    let paper_page = open_document_center(&mut session);
    accepted(&send(&mut session, paper_page));
    assert_eq!(session.draft, answered, "the printed-only editor retains accepted event answers");

    // A blank deferred-gain column is distinct from a printed zero after Save and restart.
    for short_term in [None, Some(0)] {
        let mut paper = printed.clone();
        paper.short_term = short_term;
        let page = open_document_center(&mut session);
        let mut documents = page_documents(&page);
        documents.iter_mut().find(|row| row.id == id).unwrap().document =
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::OpportunityFund(paper.clone()),
            };
        let request = save_documents(&mut session, documents);
        accepted(&send(&mut session, request));
        session = restart(session.draft.clone());
        let page = open_document_center(&mut session);
        let row = page_documents(&page).into_iter().find(|row| row.id == id).unwrap();
        assert_eq!(
            row.document,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::OpportunityFund(paper),
            }
        );
        accepted(&send(&mut session, page));
    }
}

// ---- F1: a written call that expires (section 1234(b)(1)) ----

#[test]
fn f1_a_written_call_that_expires_is_a_short_term_gain_on_line_1a() {
    // 2 XYZ calls written for $3.10 a share: 200 x 3.10 = 620.00, less 1.30 commission = 618.70
    // received. They expire unexercised: proceeds 618.70, basis 0.00, short-term, basis reported.
    // Exception 1: Schedule D line 1a (d) $619, e $0, no Form 8949 row; the short-term total is
    // +618.70 and the option income is investment income.
    let mut d = single();
    let mut call = native_sale(GainTerm::Short, 61_870, Some(0));
    call.use_kind = SaleUse::WrittenOption { wash_sale: 0 };
    add_sale(&mut d, call);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_1a"), 619);
    assert_eq!(value(&r, "capital_gain_1a_proceeds"), 619);
    assert_eq!(value(&r, "capital_gain_1a_basis"), 0);
    assert_eq!(value(&r, "capital_gain"), 619);
    assert_eq!(row_count(&r), 0, "an Exception 1 lot has no Form 8949 row");
    assert_eq!(
        printed_line(&r, "capital_gain_1a_proceeds"),
        (crate::TaxForm::ScheduleD, "1a(d)".into())
    );
    // Held 2 years 2 months and still short-term: the term is the entered Short; Long fails.
    let mut long = native_sale(GainTerm::Long, 61_870, Some(0));
    long.use_kind = SaleUse::WrittenOption { wash_sale: 0 };
    assert_eq!(
        crate::federal_rules::validate_sale(&long).unwrap_err().0,
        "SALE_WRITTEN_OPTION_SHORT_TERM"
    );
    let page = |sale: &FederalSale| IncomeSource::InvestmentSale {
        value: crate::InvestmentSaleDocument::Sale1099B(sale_document(SaleSource {
            form: Sale1099B {
                payer: Default::default(),
                recipient: Default::default(),
                state_details: Default::default(),
                description: "2 XYZ CALL 06/19/26 60 WRITTEN".into(),
                digital_asset: false,
                term: Some(sale.term),
                proceeds: sale.proceeds,
                basis: sale.basis,
                reporting: BasisReporting::ReportedToIrs,
                use_kind: sale.use_kind.clone(),
                collectible: false,
                market_discount: 0,
                several_sales: false,
                withholding: 0,
            },
            adjustment: SaleAdjustment::None,
            special: crate::SaleSpecial::NoneOfThese,
            amt: SaleAmt::SameAsRegular,
        })),
    };
    assert_eq!(
        crate::interview::document_rule(&page(&long)),
        Some("SALE_WRITTEN_OPTION_SHORT_TERM")
    );
    assert_eq!(crate::interview::document_rule(&page(&d_sale(&d))), None);
    // The holder's side of the same contracts, bought for 620.00 + 1.30 = 621.30 and expired:
    // an Investment lot of proceeds 0.00 and basis 621.30, a $621 loss on line 1a (d) $0, e $621.
    let mut d = single();
    add_sale(&mut d, native_sale(GainTerm::Short, 0, Some(62_130)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_1a_proceeds"), 0);
    assert_eq!(value(&r, "capital_gain_1a_basis"), 621);
    assert_eq!(value(&r, "capital_gain_1a"), -621);
    assert_eq!(value(&r, "capital_gain"), -621);
}

fn d_sale(d: &FederalDraft) -> FederalSale {
    let FederalIncome::Sale { data } = &d.income.last().unwrap().income else { panic!() };
    FederalSale::from(data)
}

// ---- F2: a put exercised (Pub. 550, the holder) ----

#[test]
fn f2_a_put_exercised_reduces_the_stock_sale_net_or_as_code_e() {
    // 100 ABC shares bought for 5,000.00 + 4.95 = 5,004.95 and held over a year; a $45 put bought
    // for 300.00 + 0.70 = 300.70 and exercised, selling the shares for 4,500.00 - 4.95 = 4,495.05.
    // The put's cost reduces the amount realized: 4,495.05 - 300.70 = 4,194.35; the loss is
    // 4,194.35 - 5,004.95 = (810.60), long-term.
    // (a) The broker reports net (box 6): 1d 4,194.35, 1e 5,004.95, box D, no code: line 8a.
    let mut d = single();
    add_sale(&mut d, native_sale(GainTerm::Long, 419_435, Some(500_495)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_8a_proceeds"), 4_194);
    assert_eq!(value(&r, "capital_gain_8a_basis"), 5_005);
    assert_eq!(value(&r, "capital_gain_8a"), -811);
    // (b) The broker reports gross: 1d 4,495.05, and code E with the put's cost as -300.70:
    // Part II box D row (d) 4,495.05, e 5,004.95, f E, g (300.70), h (810.60); Schedule D
    // line 8b (d) $4,495, e $5,005, g $(301), h $(811): the same long-term loss.
    let mut d = single();
    let mut gross = native_sale(GainTerm::Long, 449_505, Some(500_495));
    gross.adjustment = SaleAdjustment::OptionPremium { amount: -30_070 };
    let id = add_sale(&mut d, gross);
    let r = supported(&d);
    assert_eq!(row_value(&r, id, 1, "proceeds"), 449_505);
    assert_eq!(row_value(&r, id, 1, "basis"), 500_495);
    assert_eq!(row_value(&r, id, 1, "adjustment"), -30_070);
    assert_eq!(row_value(&r, id, 1, "gain"), -81_060);
    assert_eq!(row_text(&r, id, 1), ("D".into(), "E".into()));
    assert_eq!(value(&r, "capital_gain_8b_proceeds"), 4_495);
    assert_eq!(value(&r, "capital_gain_8b_basis"), 5_005);
    assert_eq!(value(&r, "capital_gain_8b_adjustment"), -301);
    assert_eq!(value(&r, "capital_gain_8b"), -811);
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    assert_eq!(
        printed_line(&r, "capital_gain_8b_adjustment"),
        (crate::TaxForm::ScheduleD, "8b(g)".into())
    );
    // A premium adjustment belongs on an investment, not on an option you wrote.
    let mut bad = gross_sale();
    bad.use_kind = SaleUse::WrittenOption { wash_sale: 0 };
    bad.term = GainTerm::Short;
    assert_eq!(
        crate::federal_rules::validate_sale(&bad).unwrap_err().0,
        "SALE_OPTION_PREMIUM_ON_STOCK"
    );
    // A call you wrote that was exercised: the premium is added to the stock's proceeds (code E
    // positive), here 620.00 added to 4,000.00 proceeds against a 4,200.00 basis: gain 420.00.
    let mut call = native_sale(GainTerm::Long, 400_000, Some(420_000));
    call.adjustment = SaleAdjustment::OptionPremium { amount: 62_000 };
    assert_eq!(call.gain(), 42_000);
    assert_eq!(call.column_g(), 62_000);
    assert_eq!(call.codes(), "E");
}

fn gross_sale() -> FederalSale {
    let mut gross = native_sale(GainTerm::Long, 449_505, Some(500_495));
    gross.adjustment = SaleAdjustment::OptionPremium { amount: -30_070 };
    gross
}

// ---- F3: a section 1256 contract on Form 6781 (no change) ----

#[test]
fn f3_section_1256_contracts_split_forty_sixty_and_stay_the_futures_entry() {
    // A Form 1099-B box 11 aggregate of 12,345.00 on index options: line 8 = 40% = 4,938.00
    // short-term, line 9 = 12,345 - 4,938 = 7,407 long-term; Schedule D line 4 4,938, line 11 7,407.
    let mut d = single();
    added(
        &mut d,
        FederalIncome::Section1256 {
            data: contracts_source(crate::federal_model::FederalSection1256 {
                aggregate: 1_234_500,
                withholding: 0,
                special_treatment: false,
            }),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_4"), 4_938);
    assert_eq!(value(&r, "capital_gain_11"), 7_407);
    assert_eq!(value(&r, "capital_gain"), 12_345);
    // A written option is never the futures entry: a written index option is this entry, so the
    // option page's text names the futures entry for section 1256 options.
    assert!(
        crate::federal_model::HUB_SOURCE
            .contains("section 1256 options use their separate worksheet")
    );
}

// ---- F4: a wash sale with box 1g and the basis carry ----

#[test]
fn f4_a_wash_sale_row_and_the_replacement_lot_carry_the_disallowed_loss() {
    // Lot 1: 100 shares bought for 1,000.00, sold for 600.00; 75 identical shares bought within 30
    // days. Disallowed = 75/100 x (1,000 - 600) = 300.00 (box 1g). Part I box A row: (d) 600.00,
    // (e) 1,000.00, f W, g 300.00, h (100.00), Schedule D line 1b. The replacement's basis is
    // 450 + 300 = 750.00; lot 2 sold for 900.00 shows 1e 750.00 and no 1g: Exception 1, line 1a
    // (d) $900, e $750, h $150. Short-term net = -100 + 150 = $50.
    let mut d = single();
    let mut first = native_sale(GainTerm::Short, 60_000, Some(100_000));
    first.use_kind = SaleUse::Investment { wash_sale: 30_000 };
    let id = add_sale(&mut d, first);
    add_sale(&mut d, native_sale(GainTerm::Short, 90_000, Some(75_000)));
    let r = supported(&d);
    assert_eq!(row_value(&r, id, 1, "proceeds"), 60_000);
    assert_eq!(row_value(&r, id, 1, "basis"), 100_000);
    assert_eq!(row_value(&r, id, 1, "adjustment"), 30_000);
    assert_eq!(row_value(&r, id, 1, "gain"), -10_000);
    assert_eq!(row_text(&r, id, 1), ("A".into(), "W".into()));
    assert_eq!(value(&r, "capital_gain_1b"), -100);
    assert_eq!(value(&r, "capital_gain_1b_adjustment"), 300);
    assert_eq!(value(&r, "capital_gain_1a"), 150);
    assert_eq!(value(&r, "capital_gain_1a_proceeds"), 900);
    assert_eq!(value(&r, "capital_gain_1a_basis"), 750);
    assert_eq!(value(&r, "capital_gain"), 50);
    // The disallowed loss cannot exceed the loss (the client rule and its server twin).
    let mut over = native_sale(GainTerm::Short, 60_000, Some(100_000));
    over.use_kind = SaleUse::Investment { wash_sale: 40_001 };
    assert!(crate::federal_rules::validate_sale(&over).is_err());
    over.use_kind = SaleUse::Investment { wash_sale: 40_000 };
    assert!(crate::federal_rules::validate_sale(&over).is_ok());
    // An empty basis counts as zero, so there is no loss for a wash sale to disallow.
    over.basis = None;
    assert!(crate::federal_rules::validate_sale(&over).is_err());
}

// ---- F5: a Form 1099-DA sale with basis not reported ----

#[test]
fn f5_a_digital_asset_sale_is_box_k_and_an_empty_basis_counts_as_zero() {
    // 0.5 BTC bought for 24,000.00 + 60.00 = 24,060.00, transferred in (noncovered), sold for
    // 41,250.00: Form 1099-DA 1f 41,250.00, 1g blank, box 6 long-term, box 9 checked. Entered
    // with the wallet basis: Part II box K row (d) 41,250.00, e 24,060.00, h 17,190.00, and
    // Schedule D line 9 (d) $41,250, e $24,060, h $17,190; never box E or F.
    let mut d = single();
    let mut coin = native_sale(GainTerm::Long, 4_125_000, Some(2_406_000));
    coin.basis_reporting = BasisReporting::NotReported;
    coin.digital_asset = true;
    let id = add_sale(&mut d, coin.clone());
    let r = supported(&d);
    assert_eq!(row_value(&r, id, 1, "gain"), 1_719_000);
    assert_eq!(row_text(&r, id, 1), ("K".into(), String::new()));
    assert_eq!(value(&r, "capital_gain_9_proceeds"), 41_250);
    assert_eq!(value(&r, "capital_gain_9_basis"), 24_060);
    assert_eq!(value(&r, "capital_gain_9"), 17_190);
    assert_eq!(value(&r, "capital_gain_10"), 0);
    // Left empty, the basis is zero: (e) 0, h $41,250 and one assumption note.
    let mut d = single();
    coin.basis = None;
    add_sale(&mut d, coin.clone());
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_9_basis"), 0);
    assert_eq!(value(&r, "capital_gain_9"), 41_250);
    assert!(noted(&r, "WARN_SALE_BASIS_ASSUMED_ZERO"));
    // The boxes of every holding period, reporting and digital asset combination.
    for (term, reporting, digital, letter, line) in [
        (GainTerm::Short, BasisReporting::NotReported, false, 'B', "2"),
        (GainTerm::Short, BasisReporting::NoForm1099B, false, 'C', "3"),
        (GainTerm::Long, BasisReporting::NotReported, false, 'E', "9"),
        (GainTerm::Long, BasisReporting::NoForm1099B, false, 'F', "10"),
        (GainTerm::Short, BasisReporting::ReportedToIrs, true, 'G', "1b"),
        (GainTerm::Short, BasisReporting::NotReported, true, 'H', "2"),
        (GainTerm::Short, BasisReporting::NoForm1099B, true, 'I', "3"),
        (GainTerm::Long, BasisReporting::ReportedToIrs, true, 'J', "8b"),
        (GainTerm::Long, BasisReporting::NotReported, true, 'K', "9"),
        (GainTerm::Long, BasisReporting::NoForm1099B, true, 'L', "10"),
    ] {
        let mut d = single();
        let mut sale = native_sale(term, 200_000, Some(150_000));
        sale.basis_reporting = reporting;
        sale.digital_asset = digital;
        sale.several_sales = true;
        let id = add_sale(&mut d, sale);
        let r = supported(&d);
        assert_eq!(row_text(&r, id, 1).0, alloc::format!("{letter}"));
        assert_eq!(value(&r, &alloc::format!("capital_gain_{line}")), 500, "{letter}");
        assert_eq!(value(&r, &alloc::format!("capital_gain_{line}_proceeds")), 2_000, "{letter}");
    }
    // A clean digital asset lot with its basis reported is Exception 1 (lines 1a and 8a).
    let mut d = single();
    let mut clean = native_sale(GainTerm::Long, 200_000, Some(150_000));
    clean.digital_asset = true;
    add_sale(&mut d, clean);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_8a"), 500);
    assert_eq!(row_count(&r), 0);
}

// ---- F6: opportunity fund inclusion on December 31, 2026 ----

#[test]
fn f6_an_opportunity_fund_investment_includes_its_deferred_gain() {
    // Invested 2019-06-01: Part IV short-term 50,000.00, long-term 150,000.00, no code, held on
    // December 31, 2026 worth 180,000.00. D = 200,000. Seven years were reached on 2026-06-01:
    // the basis increase is 15% x 200,000 = 30,000.00. Included = min(200,000, 180,000) - 30,000
    // = 150,000.00: short = 150,000 x 50,000 / 200,000 = 37,500.00, long = 112,500.00. Rows: Part I
    // box C code Y 37,500.00 (line 3) and Part II box F code Y 112,500.00 (line 10); the basis
    // after is 30,000 + 150,000 = 180,000.00.
    let held = |value| QofEvent::Held { fair_value: value };
    let mut d = single();
    let id = added(
        &mut d,
        FederalIncome::OpportunityFund {
            data: qof(20190601, 5_000_000, 15_000_000, held(Some(18_000_000))),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_basis_increase")), 30_000);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_included_short")), 37_500);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_included_long")), 112_500);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_basis_after")), 180_000);
    assert_eq!(row_value(&r, id, 1, "gain"), 3_750_000);
    assert_eq!(row_text(&r, id, 1), ("C".into(), "Y".into()));
    assert_eq!(row_value(&r, id, 2, "gain"), 11_250_000);
    assert_eq!(row_text(&r, id, 2), ("F".into(), "Y".into()));
    assert_eq!(value(&r, "capital_gain_3"), 37_500);
    assert_eq!(value(&r, "capital_gain_10"), 112_500);
    assert_eq!(value(&r, "capital_gain"), 150_000);
    // (a) Invested 2022-03-01: five years only on 2027-03-01, so no increase: included = min(D, V).
    let mut d = single();
    let id = added(
        &mut d,
        FederalIncome::OpportunityFund {
            data: qof(20220301, 5_000_000, 15_000_000, held(Some(18_000_000))),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_basis_increase")), 0);
    assert_eq!(value(&r, "capital_gain"), 180_000);
    // Five years only (invested 2021-12-31): 10% x 200,000 = 20,000.
    let mut d = single();
    let id = added(
        &mut d,
        FederalIncome::OpportunityFund {
            data: qof(20211231, 5_000_000, 15_000_000, held(Some(20_000_000))),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_basis_increase")), 20_000);
    assert_eq!(value(&r, "capital_gain"), 180_000);
    // (b) The value left empty: V = D, included = 200,000 - 30,000 = 170,000 with one note.
    let mut d = single();
    added(
        &mut d,
        FederalIncome::OpportunityFund { data: qof(20190601, 5_000_000, 15_000_000, held(None)) },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 170_000);
    assert!(noted(&r, "WARN_QOF_VALUE_ASSUMED"));
    // (c) Sold in full on 2026-09-30 for 250,000.00: included = min(200,000, 250,000) - 30,000 =
    // 170,000.00 (42,500 short, 127,500 long); the basis after is 200,000.00, so a separate 1099-B
    // sale of 250,000 against that basis is a $50,000 long-term gain; 170,000 + 50,000 = 220,000 =
    // 250,000 - 30,000.
    let mut d = single();
    let id = added(
        &mut d,
        FederalIncome::OpportunityFund {
            data: qof(
                20190601,
                5_000_000,
                15_000_000,
                QofEvent::Sold { sold_on: 20260930, amount: 25_000_000 },
            ),
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_included_short")), 42_500);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_included_long")), 127_500);
    assert_eq!(value(&r, &alloc::format!("qof_{id}_basis_after")), 200_000);
    add_sale(&mut d, native_sale(GainTerm::Long, 25_000_000, Some(20_000_000)));
    assert_eq!(value(&supported(&d), "capital_gain"), 220_000);
    // Sold before five years: invested 2022-03-01, sold 2026-09-30 is 4 years 7 months: no increase.
    let mut d = single();
    added(
        &mut d,
        FederalIncome::OpportunityFund {
            data: qof(
                20220301,
                0,
                10_000_000,
                QofEvent::Sold { sold_on: 20260930, amount: 4_000_000 },
            ),
        },
    );
    assert_eq!(value(&supported(&d), "capital_gain"), 40_000);
}

#[test]
fn an_opportunity_fund_section_1231_gain_is_the_o_y_pair_and_form_4797_part_i() {
    // Special gain code B: long-term only. The Form 8949 code Y row and the code O row net to zero
    // on Schedule D line 10; the gain is a Form 4797 Part I section 1231 source (net investment
    // income) and reaches Schedule D line 11 as capital gain when the year's net is a gain.
    let mut d = single();
    d.screening.as_mut().unwrap().section1231 =
        Some(crate::Section1231History { regular: vec![], amt: vec![] });
    let mut data = qof(20190601, 0, 10_000_000, QofEvent::Held { fair_value: Some(10_000_000) });
    data.gain_code = QofGainCode::Section1231;
    let id = added(&mut d, FederalIncome::OpportunityFund { data });
    let r = supported(&d);
    // 100,000 - 15% x 100,000 = 85,000.
    assert_eq!(value(&r, &alloc::format!("qof_{id}_included_property")), 85_000);
    assert_eq!(row_value(&r, id, 1, "gain"), 8_500_000);
    assert_eq!(row_text(&r, id, 1), ("F".into(), "Y".into()));
    assert_eq!(row_value(&r, id, 2, "gain"), -8_500_000);
    assert_eq!(row_text(&r, id, 2), ("F".into(), "O".into()));
    assert_eq!(value(&r, "capital_gain_10"), 0);
    assert_eq!(value(&r, "capital_gain_11"), 85_000);
    assert_eq!(value(&r, "capital_gain"), 85_000);
    // The gain is a Form 4797 line 2 source row, so lines 2-7 add to line 8 (F13, 4797-D4).
    assert_eq!(
        printed_line(&r, &alloc::format!("qof_{id}_included_property")),
        (TaxForm::Form4797, "2".into())
    );
    assert_eq!(value(&r, "form4797_line8"), 85_000);
    // Collectibles (special gain code D) is 28 percent rate gain.
    let mut d = single();
    let mut data = qof(20190601, 0, 10_000_000, QofEvent::Held { fair_value: Some(10_000_000) });
    data.gain_code = QofGainCode::Collectibles;
    added(&mut d, FederalIncome::OpportunityFund { data });
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 85_000);
    assert_eq!(value(&r, "rate_28_gain"), 85_000);
}

#[test]
fn an_opportunity_fund_treatment_the_app_cannot_compute_is_left_out_with_a_notice() {
    let unsupported = [
        QofGainCode::Section1256,
        QofGainCode::Straddle,
        QofGainCode::ExemptByTreaty,
        QofGainCode::NoninclusionTransfer,
    ];
    for (code, special) in
        unsupported.into_iter().map(|c| (c, false)).chain([(QofGainCode::None, true)])
    {
        let mut d = single();
        let mut data =
            qof(20190601, 0, 10_000_000, QofEvent::Held { fair_value: Some(10_000_000) });
        data.gain_code = code;
        data.special = special;
        added(&mut d, FederalIncome::OpportunityFund { data });
        let r = supported(&d);
        assert!(noted(&r, "WARN_QOF_INVESTMENT_LEFT_OUT"));
        assert_eq!(value(&r, "capital_gain"), 0);
    }
}

#[test]
fn each_printed_form_8997_gain_code_is_declared_and_treated_as_before() {
    let model = include_str!("../../investment/opportunity_fund.rs");
    let start = model.find("pub enum QofGainCode").unwrap();
    let body = &model[start..start + model[start..].find("\n}").unwrap()];
    for (code, name) in [
        ("A", "Section1256"),
        ("B", "Section1231"),
        ("C", "Straddle"),
        ("D", "Collectibles"),
        ("E", "ExemptByTreaty"),
        ("F", "NoninclusionTransfer"),
    ] {
        assert!(body.contains(&alloc::format!("#[libertas_form_code(\"{code}\")]\n    {name},")));
    }
    for gain_code in [
        QofGainCode::Section1256,
        QofGainCode::Straddle,
        QofGainCode::ExemptByTreaty,
        QofGainCode::NoninclusionTransfer,
    ] {
        let mut data =
            qof(20190601, 0, 10_000_000, QofEvent::Held { fair_value: Some(10_000_000) });
        data.gain_code = gain_code;
        assert!(data.left_out());
    }
}

#[test]
fn an_opportunity_fund_investment_page_rules_and_ranges() {
    let held = QofEvent::Held { fair_value: None };
    let page = |data: QofInvestment| IncomeSource::InvestmentSale {
        value: crate::InvestmentSaleDocument::OpportunityFund(qof_document(data)),
    };
    let mut data = qof(20190601, 0, 0, held.clone());
    assert_eq!(crate::interview::document_rule(&page(data.clone())), Some("QOF_NO_DEFERRED_GAIN"));
    data = qof(20190601, 100, 100, held.clone());
    data.gain_code = QofGainCode::Section1231;
    assert_eq!(
        crate::interview::document_rule(&page(data.clone())),
        Some("QOF_SECTION_1231_LONG_TERM")
    );
    data.gain_code = QofGainCode::Collectibles;
    assert_eq!(
        crate::interview::document_rule(&page(data.clone())),
        Some("QOF_COLLECTIBLES_LONG_TERM")
    );
    // Dates: the first investment date, and an investment in 2026 defers nothing past 2026.
    assert!(qof(20171222, 0, 100, held.clone()).valid());
    assert!(qof(20251231, 0, 100, held.clone()).valid());
    let sold = |on| QofEvent::Sold { sold_on: on, amount: 100 };
    assert!(qof(20190601, 0, 100, sold(20261231)).valid());
    let mut named = qof(20190601, 0, 100, sold(20261231));
    named.fund = "Full printed QOF description 🧾".repeat(20);
    assert!(named.valid());
    // Preserve this owner's original is_empty semantics; no trim rule added.
    named.fund = " ".into();
    assert!(named.valid());
}

// ---- F7: a nominee sale (code N) ----

#[test]
fn f7_a_nominee_sale_is_a_row_with_column_g_backing_out_the_gain() {
    // A 1099-B in the person's name for a parent's shares: 1d 10,000.00, parent's basis 6,000.00,
    // box B, short-term. Row Part I box B: (d) 10,000.00, e 6,000.00, f N, g (4,000.00),
    // (h) 0.00: line 2 (d) $10,000, e $6,000, g $(4,000), h $0. No income.
    let mut d = single();
    let mut nominee = native_sale(GainTerm::Short, 1_000_000, Some(600_000));
    nominee.basis_reporting = BasisReporting::NotReported;
    nominee.adjustment = SaleAdjustment::Nominee;
    let id = add_sale(&mut d, nominee);
    let r = supported(&d);
    assert_eq!(row_value(&r, id, 1, "adjustment"), -400_000);
    assert_eq!(row_value(&r, id, 1, "gain"), 0);
    assert_eq!(row_text(&r, id, 1), ("B".into(), "N".into()));
    assert_eq!(value(&r, "capital_gain_2_proceeds"), 10_000);
    assert_eq!(value(&r, "capital_gain_2_basis"), 6_000);
    assert_eq!(value(&r, "capital_gain_2_adjustment"), -4_000);
    assert_eq!(value(&r, "capital_gain_2"), 0);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "agi"), value(&supported(&single()), "agi"));
    // A nominee loss is backed out the same way.
    let mut loss = native_sale(GainTerm::Short, 100_000, Some(300_000));
    loss.adjustment = SaleAdjustment::Nominee;
    assert_eq!(loss.gain(), 0);
    assert_eq!(loss.column_g(), 200_000);
}

// ---- F8: market discount (code D) ----

#[test]
fn f8_accrued_market_discount_is_interest_up_to_the_gain() {
    // A $10,000 face bond bought for 9,400.00 and sold for 9,900.00; 1099-B 1e 9,400.00, 1f
    // 210.00, long-term. Worksheet: line 3 = 9,900 - 9,400 = 500, line 4 = 210, line 5 = 210:
    // interest +$210; row Part II box D (d) 9,900.00, e 9,400.00, f D, g (210.00), h
    // 290.00, Schedule D line 8b (not 8a).
    let mut d = single();
    let mut bond = native_sale(GainTerm::Long, 990_000, Some(940_000));
    bond.market_discount = 21_000;
    let id = add_sale(&mut d, bond.clone());
    let before = supported(&single());
    let r = supported(&d);
    assert_eq!(row_value(&r, id, 1, "adjustment"), -21_000);
    assert_eq!(row_value(&r, id, 1, "gain"), 29_000);
    assert_eq!(row_text(&r, id, 1), ("D".into(), "D".into()));
    assert_eq!(value(&r, "capital_gain_8b"), 290);
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    assert_eq!(value(&r, "capital_gain"), 290);
    assert_eq!(value(&r, &alloc::format!("sale_{id}_market_discount")), 210);
    assert_eq!(value(&r, "agi") - value(&before, "agi"), 500);
    // The gain below the discount: interest is the whole gain.
    let mut small = native_sale(GainTerm::Long, 950_000, Some(940_000));
    small.market_discount = 21_000;
    assert_eq!(small.market_discount_interest(), 10_000);
    assert_eq!(small.gain(), 0);
    // Loss variant: sold for 9,300.00, line 3 = 0, no interest, g 0, h (100.00).
    let mut d = single();
    let mut loss = bond.clone();
    loss.proceeds = 930_000;
    let id = add_sale(&mut d, loss);
    let r = supported(&d);
    assert_eq!(row_value(&r, id, 1, "adjustment"), 0);
    assert_eq!(row_value(&r, id, 1, "gain"), -10_000);
    assert_eq!(value(&r, "capital_gain_8b"), -100);
    assert!(!r.lines.iter().any(|l| l.id.ends_with("_market_discount")));
    // Market discount needs a basis and a bond held as an investment.
    bond.basis = None;
    assert_eq!(
        crate::federal_rules::validate_sale(&bond).unwrap_err().0,
        "SALE_MARKET_DISCOUNT_NEEDS_BASIS"
    );
    bond.basis = Some(940_000);
    bond.use_kind = SaleUse::Personal;
    assert_eq!(
        crate::federal_rules::validate_sale(&bond).unwrap_err().0,
        "SALE_MARKET_DISCOUNT_ON_A_BOND"
    );
}

// ---- F9: a written call bought back at a loss ----

#[test]
fn f9_a_written_option_closed_at_a_loss_has_negative_proceeds_or_a_basis() {
    // Written for $200.00 net, bought back for $650.00. Style 1: 1d (450.00), 1e 0.00. Style 2: 1d
    // 200.00, 1e 650.00. Both: (h) (450.00) short-term, line 1a.
    for (proceeds, basis) in [(-45_000, 0), (20_000, 65_000)] {
        let mut d = single();
        let mut call = native_sale(GainTerm::Short, proceeds, Some(basis));
        call.use_kind = SaleUse::WrittenOption { wash_sale: 0 };
        add_sale(&mut d, call);
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain_1a"), -450);
        assert_eq!(value(&r, "capital_gain"), -450);
    }
    // The same -450.00 on an Investment lot is refused: client rule key and server twin.
    let investment = native_sale(GainTerm::Short, -45_000, Some(0));
    assert_eq!(
        crate::federal_rules::validate_sale(&investment).unwrap_err().0,
        "SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY"
    );
    let page = IncomeSource::InvestmentSale {
        value: crate::InvestmentSaleDocument::Sale1099B(sale_document(SaleSource {
            form: Sale1099B {
                payer: Default::default(),
                recipient: Default::default(),
                state_details: Default::default(),
                description: String::new(),
                digital_asset: false,
                term: Some(GainTerm::Short),
                proceeds: -45_000,
                basis: Some(0),
                reporting: BasisReporting::ReportedToIrs,
                use_kind: SaleUse::Investment { wash_sale: 0 },
                collectible: false,
                market_discount: 0,
                several_sales: false,
                withholding: 0,
            },
            adjustment: SaleAdjustment::None,
            special: crate::SaleSpecial::NoneOfThese,
            amt: SaleAmt::SameAsRegular,
        })),
    };
    assert_eq!(
        crate::interview::document_rule(&page),
        Some("SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY")
    );
    // A written option's loss counts as an investment loss (Form 4952), like any option sold.
    let mut call = native_sale(GainTerm::Short, -45_000, Some(0));
    call.use_kind = SaleUse::WrittenOption { wash_sale: 0 };
    assert_eq!(call.gain(), -45_000);
    assert!(crate::federal_rules::validate_sale(&call).is_ok());
}

// ---- F10: a 2026 gain invested in a fund in 2027 (code Z) ----

#[test]
fn f10_a_gain_invested_in_a_fund_in_2027_is_deferred_by_code_z() {
    // A 2026-11-03 sale with a long-term gain of 80,000.00 (no form, box F, line 10); 60,000.00
    // invested in a fund on 2027-02-15, within 180 days, confirmed. Row Part II box F code Z (g)
    // (60,000.00) on line 10; the long-term total is 80,000 - 60,000 = 20,000.
    let deferral = |long: i64, within: bool, on: u32| QofDeferral {
        fund: "Example Opportunity Fund".into(),
        invested_on: on,
        short_term: 0,
        long_term: long,
        within_180_days: within,
        special: false,
    };
    let sale = || {
        let mut sale = native_sale(GainTerm::Long, 10_000_000, Some(2_000_000));
        sale.basis_reporting = BasisReporting::NoForm1099B;
        sale
    };
    let mut d = single();
    add_sale(&mut d, sale());
    let id = added(
        &mut d,
        FederalIncome::OpportunityFundDeferral { data: deferral(6_000_000, true, 20270215) },
    );
    let r = supported(&d);
    assert_eq!(row_value(&r, id, 1, "adjustment"), -6_000_000);
    assert_eq!(row_value(&r, id, 1, "gain"), -6_000_000);
    assert_eq!(row_text(&r, id, 1), ("F".into(), "Z".into()));
    assert_eq!(value(&r, &alloc::format!("qof_deferral_{id}_long")), 60_000);
    assert_eq!(value(&r, "capital_gain_10"), 20_000);
    assert_eq!(value(&r, "capital_gain_10_adjustment"), -60_000);
    assert_eq!(value(&r, "capital_gain"), 20_000);
    // A long-term deferral of 90,000 is above the 80,000 of long-term gains: a blocking issue.
    let mut d = single();
    add_sale(&mut d, sale());
    added(
        &mut d,
        FederalIncome::OpportunityFundDeferral { data: deferral(9_000_000, true, 20270215) },
    );
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_QOF_DEFERRAL_ABOVE_GAINS"),
        "{:?}",
        r.issues
    );
    // The 180-day answer No: no deferral and one notice.
    let mut d = single();
    add_sale(&mut d, sale());
    added(
        &mut d,
        FederalIncome::OpportunityFundDeferral { data: deferral(6_000_000, false, 20270215) },
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_QOF_DEFERRAL_NOT_ALLOWED"));
    assert_eq!(value(&r, "capital_gain"), 80_000);
    // A special treatment leaves it out with its own notice.
    let mut special = deferral(6_000_000, true, 20270215);
    special.special = true;
    let mut d = single();
    add_sale(&mut d, sale());
    added(&mut d, FederalIncome::OpportunityFundDeferral { data: special });
    let r = supported(&d);
    assert!(noted(&r, "WARN_QOF_DEFERRAL_LEFT_OUT"));
    assert_eq!(value(&r, "capital_gain"), 80_000);
    // An investment in 2026 defers nothing past December 31, 2026: outside the field range.
    assert!(!deferral(6_000_000, true, 20261220).valid());
    assert!(!deferral(6_000_000, true, 20270629).valid());
    assert!(deferral(6_000_000, true, 20270101).valid());
    assert!(deferral(6_000_000, true, 20270628).valid());
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::Record {
            value: crate::IncomeRecord::OpportunityFundDeferral(deferral(0, true, 20270215))
        }),
        Some("QOF_DEFERRAL_NO_GAIN")
    );
}

// ---- every Form 8949 code's column (g) and (h) ----

#[test]
fn each_code_has_its_column_g_and_column_h_effect() {
    // B: the form reported a basis of 1,000.00 and the real basis is 1,500.00: (e) shows 1,000.00
    // and (g) (500.00) takes (h) to 3,000 - 1,500 = 1,500.00.
    let mut b = native_sale(GainTerm::Long, 300_000, Some(150_000));
    b.adjustment = SaleAdjustment::BasisCorrection { reported_basis: 100_000 };
    assert_eq!((b.column_e(), b.column_g(), b.gain()), (100_000, -50_000, 150_000));
    assert_eq!(b.codes(), "B");
    // C: a collectible changes no amount; it needs a row.
    let mut c = native_sale(GainTerm::Long, 300_000, Some(150_000));
    c.collectible = true;
    assert_eq!((c.column_g(), c.gain(), c.codes().as_str()), (0, 150_000, "C"));
    assert!(!c.summary());
    // D: see F8. E: selling expenses 50.00 are a negative adjustment.
    let mut e = native_sale(GainTerm::Long, 300_000, Some(150_000));
    e.adjustment = SaleAdjustment::SellingExpenses { amount: 5_000 };
    assert_eq!((e.column_g(), e.gain(), e.codes().as_str()), (-5_000, 145_000, "E"));
    // L: a loss on personal-use property is a positive adjustment up to the loss.
    let mut l = native_sale(GainTerm::Long, 100_000, Some(150_000));
    l.use_kind = SaleUse::Personal;
    assert_eq!((l.column_g(), l.gain(), l.codes().as_str()), (50_000, 0, "L"));
    // M: several sales on one row changes no amount; it needs a row.
    let mut m = native_sale(GainTerm::Long, 300_000, Some(150_000));
    m.several_sales = true;
    assert_eq!((m.column_g(), m.gain(), m.codes().as_str()), (0, 150_000, "M"));
    assert!(!m.summary());
    // N: the gain is backed out.
    let mut n = native_sale(GainTerm::Long, 300_000, Some(150_000));
    n.adjustment = SaleAdjustment::Nominee;
    assert_eq!((n.column_g(), n.gain(), n.codes().as_str()), (-150_000, 0, "N"));
    // T: the holding period changes no amount.
    let mut t = native_sale(GainTerm::Long, 300_000, Some(150_000));
    t.adjustment = SaleAdjustment::HoldingPeriodCorrection;
    assert_eq!((t.column_g(), t.gain(), t.codes().as_str()), (0, 150_000, "T"));
    // W: a wash sale loss disallowed is a positive adjustment.
    let mut w = native_sale(GainTerm::Short, 60_000, Some(100_000));
    w.use_kind = SaleUse::Investment { wash_sale: 30_000 };
    assert_eq!((w.column_g(), w.gain(), w.codes().as_str()), (30_000, -10_000, "W"));
    // Codes combine in alphabetical order.
    let mut all = native_sale(GainTerm::Long, 60_000, Some(100_000));
    all.use_kind = SaleUse::Investment { wash_sale: 20_000 };
    all.collectible = true;
    all.several_sales = true;
    all.adjustment = SaleAdjustment::SellingExpenses { amount: 1_000 };
    assert_eq!(all.codes(), "CEMW");
    // Column (h) is always (d) - (e) + (g).
    for sale in [&b, &c, &e, &l, &m, &n, &t, &w, &all] {
        assert_eq!(sale.proceeds - sale.column_e() + sale.column_g(), sale.gain());
    }
    // Y, Z, O and Q rows appear in F6, F10, the section 1231 test and the fund section 1202 test.
}

#[test]
fn a_row_is_numbered_by_entry_and_schedule_d_columns_foot_across_boxes() {
    // Three lots in three boxes: Schedule D (d) - (e) + (g) = h on every line.
    let mut d = single();
    let mut a = native_sale(GainTerm::Short, 60_000, Some(100_000));
    a.use_kind = SaleUse::Investment { wash_sale: 30_000 };
    let mut b = native_sale(GainTerm::Short, 500_000, Some(300_000));
    b.basis_reporting = BasisReporting::NotReported;
    b.adjustment = SaleAdjustment::SellingExpenses { amount: 7_000 };
    let mut c = native_sale(GainTerm::Long, 800_000, Some(300_000));
    c.basis_reporting = BasisReporting::NoForm1099B;
    c.several_sales = true;
    for sale in [a, b, c] {
        add_sale(&mut d, sale);
    }
    let r = supported(&d);
    assert_eq!(row_count(&r), 3);
    for (line, short) in [("1b", true), ("2", true), ("10", false)] {
        let part = |name: &str| value(&r, &alloc::format!("capital_gain_{line}_{name}"));
        assert_eq!(
            part("proceeds") - part("basis") + part("adjustment"),
            value(&r, &alloc::format!("capital_gain_{line}")),
            "{line}"
        );
        let _ = short;
    }
    assert_eq!(value(&r, "capital_gain_2"), 1_930);
    assert_eq!(value(&r, "capital_gain_10"), 5_000);
    assert_eq!(value(&r, "capital_gain"), -100 + 1_930 + 5_000);
}

#[test]
fn saving_and_reopening_the_new_sale_and_fund_kinds_keeps_every_field() {
    let mut s = Interview::new(TEST_USER, single());
    let mut written = native_sale(GainTerm::Short, -45_000, Some(0));
    written.use_kind = SaleUse::WrittenOption { wash_sale: 0 };
    let mut coin = native_sale(GainTerm::Long, 4_125_000, None);
    coin.digital_asset = true;
    coin.basis_reporting = BasisReporting::NotReported;
    coin.several_sales = true;
    let mut bond = native_sale(GainTerm::Long, 990_000, Some(940_000));
    bond.market_discount = 21_000;
    let mut nominee = native_sale(GainTerm::Short, 1_000_000, Some(600_000));
    nominee.adjustment = SaleAdjustment::Nominee;
    let mut premium = native_sale(GainTerm::Long, 449_505, Some(500_495));
    premium.adjustment = SaleAdjustment::OptionPremium { amount: -30_070 };
    let page = |sale: &FederalSale, description: &str| IncomeSource::InvestmentSale {
        value: crate::InvestmentSaleDocument::Sale1099B(sale_document(SaleSource {
            form: Sale1099B {
                payer: Default::default(),
                recipient: Default::default(),
                state_details: Default::default(),
                description: description.into(),
                digital_asset: sale.digital_asset,
                term: Some(sale.term),
                proceeds: sale.proceeds,
                basis: sale.basis,
                reporting: sale.basis_reporting,
                use_kind: sale.use_kind.clone(),
                collectible: sale.collectible,
                market_discount: sale.market_discount,
                several_sales: sale.several_sales,
                withholding: sale.withholding,
            },
            adjustment: sale.adjustment.clone(),
            special: sale.special,
            amt: SaleAmt::SameAsRegular,
        })),
    };
    let mut documents = vec![
        page(&written, ""),
        page(&coin, ""),
        page(&bond, ""),
        page(&nominee, ""),
        page(&premium, ""),
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(qof_document(qof(
                20190601,
                5_000_000,
                15_000_000,
                QofEvent::Held { fair_value: Some(18_000_000) },
            ))),
        },
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(qof_document(qof(
                20190601,
                0,
                15_000_000,
                QofEvent::Sold { sold_on: 20260930, amount: 25_000_000 },
            ))),
        },
        IncomeSource::Record {
            value: crate::IncomeRecord::OpportunityFundDeferral(QofDeferral {
                fund: "Fund".into(),
                invested_on: 20270215,
                short_term: 100,
                long_term: 0,
                within_180_days: true,
                special: false,
            }),
        },
    ];
    // Both QOF carriers keep complete descriptions through the
    // existing normal Save/reopen/restart lifecycle; amounts remain unchanged.
    for document in &mut documents {
        let before = document.clone();
        match document {
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::OpportunityFund(data),
            } => data.fund = "Full printed QOF 🧾".repeat(30),
            IncomeSource::Record { value: crate::IncomeRecord::OpportunityFundDeferral(data) } => {
                data.fund = "Full printed QOF 🧾".repeat(30)
            }
            _ => {}
        }
        rebind_paper_answers(&before, document);
    }
    save_each_topic(&mut s, documents.drain(..).map(|d| document(0, d)).collect());
    let stored = s.draft.income.clone();
    assert_eq!(stored.len(), 1 + 8);
    // Reopening the page and saving it unchanged keeps every stored record.
    let mut rows: Vec<_> = page_documents(&open_document_center(&mut s))
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::InvestmentSale { .. }))
        .collect();
    rows.extend(page_documents(&open_topic(&mut s, TaxSection::InvestmentSales)));
    assert_eq!(rows.len(), 8);
    save_each_topic(&mut s, rows);
    for (a, b) in s.draft.income.iter().zip(&stored) {
        assert_eq!(a, b);
    }
    assert_eq!(s.draft.income, stored);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    // A blank term box imports as an empty term: the page asks for it and the save refuses a null.
    let mut blank_term = sale_document(SaleSource {
        form: Sale1099B {
            payer: Default::default(),
            recipient: Default::default(),
            state_details: Default::default(),
            description: String::new(),
            digital_asset: false,
            term: None,
            proceeds: 100,
            basis: None,
            reporting: BasisReporting::ReportedToIrs,
            use_kind: SaleUse::Investment { wash_sale: 0 },
            collectible: false,
            market_discount: 0,
            several_sales: false,
            withholding: 0,
        },
        adjustment: SaleAdjustment::None,
        special: crate::SaleSpecial::NoneOfThese,
        amt: SaleAmt::SameAsRegular,
    });
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Sale1099B(blank_term.clone())
        }),
        Some("SALE_TERM_NEEDED")
    );
    blank_term.term = Some(GainTerm::Long);
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Sale1099B(blank_term)
        }),
        None
    );
    // A page that breaks a rule is refused on the server with the client rule's key.
    let bad = save_documents(
        &mut s,
        vec![document(0, page(&native_sale(GainTerm::Short, -1, Some(0)), "Negative"))],
    );
    rejected_with(send(&mut s, bad), "SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY");
}

// ---- section 1202 gain through a fund ----

fn dividend_with_1202(
    distributions: i64,
    gain: i64,
    exclusion: Option<FundQsbsExclusion>,
) -> FederalIncome {
    let data = dividend_source(FederalDividend {
        capital_distributions: distributions,
        section_1202_gain: gain,
        section_1202_exclusion: exclusion,
        ..Default::default()
    });
    FederalIncome::Dividend { data }
}

#[test]
fn a_funds_section_1202_gain_in_box_2c_is_excluded_by_the_chosen_percentage() {
    // Box 2a 100,000.00 of which box 2c 40,000.00: the full gain stays on Schedule D line 13 (long
    // term) and the exclusion is a Form 8949 Part II box F row, code Q, with a negative column (g).
    // 50%: 20,000.00 excluded, the other 20,000.00 is 28% rate gain, AMT preference 7% of 20,000 =
    // 1,400.00. 75%: 30,000.00 excluded, 10,000.00 28% rate gain, preference 2,100.00. 100%:
    // 40,000.00 excluded, no 28% gain, no preference. None: nothing excluded, no 28% gain.
    for (choice, excluded, rate_28, preference) in [
        (FundQsbsExclusion::Fifty, 20_000, 20_000, 1_400),
        (FundQsbsExclusion::SeventyFive, 30_000, 10_000, 2_100),
        (FundQsbsExclusion::Hundred, 40_000, 0, 0),
        (FundQsbsExclusion::None, 0, 0, 0),
    ] {
        let mut d = single();
        let id = added(&mut d, dividend_with_1202(10_000_000, 4_000_000, Some(choice)));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 100_000 - excluded, "{choice:?}");
        assert_eq!(value(&r, "rate_28_gain"), rate_28, "{choice:?}");
        assert_eq!(value(&r, "qsbs_amt_preference"), preference, "{choice:?}");
        if excluded > 0 {
            assert_eq!(row_value(&r, id, 1, "adjustment"), -excluded * 100);
            assert_eq!(row_text(&r, id, 1), ("F".into(), "Q".into()));
            assert_eq!(value(&r, "capital_gain_10_adjustment"), -excluded);
        } else {
            assert_eq!(row_count(&r), 0);
        }
    }
    // Box 2c is part of box 2a, and its exclusion must be chosen: client rules and server twins.
    let page = |gain, exclusion| FederalIncome::Dividend {
        data: dividend_source(FederalDividend {
            capital_distributions: 1_000,
            section_1202_gain: gain,
            section_1202_exclusion: exclusion,
            ..Default::default()
        }),
    };
    assert_eq!(
        crate::federal_rules::validate_income_amount(&page(1_001, Some(FundQsbsExclusion::Fifty)))
            .err()
            .map(|issue| issue.0),
        Some("DIV_SECTION_1202_ABOVE_2A")
    );
    assert_eq!(
        crate::federal_rules::validate_income_amount(&page(1_000, None)).err().map(|issue| issue.0),
        Some("DIV_SECTION_1202_NEEDS_EXCLUSION")
    );
    assert_eq!(
        crate::federal_rules::validate_income_amount(&page(1_000, Some(FundQsbsExclusion::None)))
            .err()
            .map(|issue| issue.0),
        None
    );
    assert_eq!(
        crate::federal_rules::validate_income_amount(&page(0, None)).err().map(|issue| issue.0),
        None
    );
}

#[test]
fn a_funds_form_2439_box_1c_gain_is_excluded_by_the_chosen_percentage() {
    // Form 2439 box 1a 50,000.00 (line 11), box 1c 20,000.00, 75% exclusion: 15,000.00 excluded;
    // the fund's whole 1a gain stays on Schedule D line 11 and the exclusion is a code Q row.
    let page = |small_business| {
        form2439_paper(Form2439 {
            fund: crate::PrintedParty::from_name(String::new()),
            shareholder: Default::default(),
            gain: 5_000_000,
            small_business,
            ..Default::default()
        })
    };
    let mut d = single();
    let id = added(
        &mut d,
        FederalIncome::UndistributedGain {
            data: crate::UndistributedGainSource {
                section_1202_exclusion: Some(FundQsbsExclusion::SeventyFive),
                ..crate::UndistributedGainSource::printed(page(2_000_000))
            },
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_11"), 50_000);
    assert_eq!(row_value(&r, id, 1, "adjustment"), -1_500_000);
    assert_eq!(row_text(&r, id, 1), ("F".into(), "Q".into()));
    assert_eq!(value(&r, "capital_gain"), 35_000);
    assert_eq!(value(&r, "rate_28_gain"), 5_000);
    // Box 1c with no chosen exclusion is refused on the topic page with the client rule's key;
    // the printed paper alone is accepted on the Document center.
    let unanswered = FederalIncomeEntry {
        income: FederalIncome::UndistributedGain {
            data: crate::UndistributedGainSource::printed(page(2_000_000)),
        },
        ..d.income[1].clone()
    };
    assert_eq!(
        crate::federal_rules::validate_income_issue_for_year_with_mode(&unanswered, 2026, true)
            .map_err(|(issue, _)| issue.key),
        Err("FORM_2439_SECTION_1202_NEEDS_EXCLUSION")
    );
    assert!(
        crate::federal_rules::validate_income_issue_for_year_with_mode(&unanswered, 2026, false)
            .is_ok()
    );
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Form2439(page(2_000_000))
        }),
        None
    );
    // Over the per-issuer limit: the fund's gain is one issuer's, capped at $10,000,000
    // ($5,000,000 married filing separately).
    let capped = crate::qsbs::fund_gain(1_500_000_000, Some(FundQsbsExclusion::Hundred), false);
    assert_eq!(capped.excluded, 1_000_000_000);
    let separate = crate::qsbs::fund_gain(1_500_000_000, Some(FundQsbsExclusion::Hundred), true);
    assert_eq!(separate.excluded, 500_000_000);
}

// ---- the removed refusals, kinds and notices ----

#[test]
fn no_fixture_or_resource_still_names_a_removed_refusal() {
    use std::fs;
    let removed = [
        "WARN_SALE_OPTION_LEFT_OUT",
        "WARN_SALE_UNKNOWN_BASIS_LEFT_OUT",
        "ISSUE_A_NOMINEE_SALE_FORM_8949_CODE_N_IS_NOT_SUPPORTED_47BCAE39",
        "ISSUE_SITUATION_OPPORTUNITY_FUND",
        "SITUATION_NAME_OPPORTUNITY_FUND",
        "WARN_DIVIDEND_SECTION_1202_LEFT_OUT",
        "WARN_UNDISTRIBUTED_GAIN_SECTION_1202_LEFT_OUT",
        "SaleSpecial::UnknownBasis",
        "SaleSpecial::OptionContract",
        "RareSituation::OpportunityFund",
        "DividendSpecial::Section1202",
    ];
    let mut dirs = vec![std::path::PathBuf::from("src")];
    let mut files = 0;
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                && !path.ends_with("sale_specials.rs")
            {
                let text = fs::read_to_string(&path).unwrap();
                files += 1;
                for name in removed {
                    assert!(!text.contains(name), "{} names {name}", path.display());
                }
            }
        }
    }
    assert!(files > 80, "{files}");
    for (key, _) in APP_STRINGS {
        assert!(!removed.contains(key), "{key}");
    }
    // The situations list no longer offers an opportunity fund and the sale special only the
    // treatments the page still leaves out.
    assert_eq!(RareSituation::ALL.len(), 10);
    for special in [
        crate::SaleSpecial::StockCompensation,
        crate::SaleSpecial::BusinessProperty,
        crate::SaleSpecial::Home,
        crate::SaleSpecial::Another,
    ] {
        assert!(special.notice().is_some());
    }
    assert!(crate::SaleSpecial::NoneOfThese.notice().is_none());
}

#[test]
fn an_empty_basis_is_zero_with_one_note_and_other_specials_still_leave_a_sale_out() {
    let mut d = single();
    add_sale(&mut d, native_sale(GainTerm::Long, 300_000, None));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 3_000);
    assert!(noted(&r, "WARN_SALE_BASIS_ASSUMED_ZERO"));
    // A personal-use lot with no basis has no loss and a gain is taxable.
    let mut personal = native_sale(GainTerm::Long, 300_000, None);
    personal.use_kind = SaleUse::Personal;
    assert_eq!(personal.personal_loss(), 0);
    assert_eq!(personal.gain(), 300_000);
    // Another treatment still leaves the sale out with its notice.
    let mut d = single();
    let mut other = native_sale(GainTerm::Long, 300_000, Some(100_000));
    other.special = crate::SaleSpecial::Another;
    add_sale(&mut d, other);
    let r = supported(&d);
    assert!(noted(&r, "WARN_SALE_OTHER_LEFT_OUT"));
    assert_eq!(value(&r, "capital_gain"), 0);
}

// ---- Unit B11 4797: the removed disposition refusals ----

#[test]
fn the_disposition_refusals_that_are_computed_now_are_gone() {
    // Each of these stood in front of one short rule of Pub. 544, Pub. 946 or the
    // Form 4797 instructions (map business-disposition-specials.md, section 2.1).
    for key in [
        "ISSUE_ASSET_SAME_YEAR_DISPOSAL",
        "ISSUE_ASSET_DISPOSAL_WITH_179_CARRYFORWARD",
        "ISSUE_ASSET_FINAL_MID_QUARTER_DISPOSAL",
        "ISSUE_ASSET_PERSONAL_SALE_GAIN",
        "ISSUE_ASSET_MIXED_USE_INSTALLMENT",
        "ISSUE_ASSET_SHORT_HOLDING_INSTALLMENT",
        "ISSUE_ASSET_VEHICLE_PERSONAL_USE_RECAPTURE",
        "ISSUE_ASSET_PERSONAL_USE_179_RECAPTURE",
        "ISSUE_RENTAL_BUILDING_SHORT_HOLDING_PERIOD",
        "ISSUE_RENTAL_SALE_WITH_PERSONAL_USE",
        "ISSUE_RENTAL_SALE_NIIT_SAFE_HARBOR",
        "ISSUE_RENTAL_INSTALLMENT_MIXED_GAIN_LOSS",
        "ISSUE_RENTAL_INSTALLMENT_PASSIVE_LOSS",
        "ISSUE_AN_INSTALLMENT_SALE_OF_A_RENTAL_WITH_A_2026_LOSS_RELEASES_THE_LOSS_5C2C575E",
        "ISSUE_EMPLOYEE_VEHICLE_DISPOSAL_NOT_CALCULATED",
        "ISSUE_SELLING_OR_STOPPING_THE_USE_OF_EQUIPMENT_RENTED_OUT_OUTSIDE_A_BUSIN_3CE6FF30",
        "ISSUE_BUSINESS_PROPERTY_LOSSES_FROM_2021_2025_MAKE_PART_OF_THIS_YEAR_S_SE_6C81AB34",
        "ISSUE_SECTION1231_BUSINESS_PROPERTY_SOLD_BY_A_BUSINESS_D968ED55",
    ] {
        assert!(APP_STRINGS.iter().all(|(k, _)| *k != key), "{key} is still declared");
    }
    // The 1099-B sale entry of business property says the asset's own record owns it.
    let text =
        APP_STRINGS.iter().find(|(k, _)| *k == "WARN_SALE_BUSINESS_PROPERTY_LEFT_OUT").unwrap().1;
    assert!(text.contains("own record"));
}

#[test]
fn equipment_rented_out_outside_a_business_is_sold_with_recapture_and_a_capital_part() {
    // The $10,000 equipment (placed 2024-03-01, $5,200 allowed through 2025 plus the
    // $960 half-year share of 2026), sold 2026-07-15 for $12,000: gain $8,160, of
    // which $6,160 (the depreciation allowed) is section 1245 ordinary income on
    // Part III line 27b, and $2,000 a long-term capital gain on Form 8949.
    let mut d = complete(FederalStatus::Single);
    let asset = BusinessAsset {
        cost: 1_000_000,
        acquired: 20240301,
        placed: 20240301,
        history: EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation: 520_000,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        disposal: crate::AssetDisposal::Disposed {
            date: 20260715,
            how: crate::DisposalKind::Sold {
                livestock: crate::SoldLivestock::No,
                proceeds: 1_200_000,
                selling_expenses: 0,
                payment: crate::SalePayment::InFull,
                buyer: crate::Buyer::Unrelated,
            },
        },
        ..bonus_equipment(0, 100)
    };
    add(
        &mut d,
        income(FederalIncome::NonbusinessProperty {
            kind: crate::NonbusinessPropertyKind::PersonalRental {
                related: alloc::vec![],
                basis: 1_000_000,
                depreciable_basis: 1_000_000,
                assets: vec![asset],
                history: crate::PersonalRentalHistory::NONE,
            },
            receipts: 0,
            expenses: 0,
            ordinary: true,
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_2_asset_0_disposal_gain"), 8_160);
    assert_eq!(value(&r, "personal_rental_2_asset_0_disposal_1245_recapture"), 6_160);
    assert_eq!(value(&r, "form_8949_2_1_gain"), 2_000);
}
