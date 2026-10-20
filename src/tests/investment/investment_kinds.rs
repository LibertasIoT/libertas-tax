// Unit B7a: the investment kinds no page wrote before (employee stock, ESPP and NUA share sales,
// qualified small business stock, the section 1244 loss, worthless securities, nonbusiness bad
// debts, the 1099-DIV return of capital or liquidation, Form 2439, and the business and farm
// records for section 197 intangibles and the fuel tax credit). Fixtures F1 to F12 of the design
// map (tax-forms/investment-kinds.md), each amount worked by hand in the comment beside it from
// Pub. 525 Examples 10 to 12, Pub. 550 (section 1244 and return of capital examples) and the
// Schedule D instructions, with the 2026 amounts the App uses: $16,100 single standard deduction,
// 10% to $12,400, 12% to $50,400, 22% to $105,700, 0% capital gain rate through $49,450 of taxable
// income, Tax Table rows $50 wide taxed at their midpoint. The single filer `complete` has W-2
// wages of $60,000: AGI 60,000, taxable 43,900, tax 5,023 (Tax Table midpoint 43,925: 1,240 + 12%
// x 31,525). Every amount is synthetic.
use super::*;
use crate::tests::cross::interview::{
    accepted, business, cash_farm, document, farm_save, open_topic, page_documents, rejected_with,
    save_businesses, save_topic, stored_business,
};
use crate::{
    BusinessIntangible, CapitalReturn1099Div, EmployeeShareSale, EmployeeShareSaleMore,
    EsppShareSale, EsppShareSaleMore, Form2439, LiquidationProperty, NonbusinessBadDebt,
    NuaInheritance, NuaShareSale, NuaShareSaleMore, QsbStockLot, QsbsSale, Section1244Stock,
    StockLossBasis, StockLossEvent, WorthlessSecurity,
};
use alloc::{format, string::String};

fn single() -> Interview {
    Interview::new(TEST_USER, complete(FederalStatus::Single))
}
// Saves the Investment sales page holding exactly these documents.
fn investment(s: &mut Interview, documents: Vec<IncomeDocument>) -> Reply {
    save(s, TaxSection::InvestmentSales, documents)
}
fn save(s: &mut Interview, topic: TaxSection, documents: Vec<IncomeDocument>) -> Reply {
    let request = save_topic(
        &s.draft.clone(),
        topic,
        documents.into_iter().map(|d| document(0, d)).collect(),
    );
    send(s, request)
}
// The id of the session's only stored income of a kind.
fn id_of(s: &Interview, wanted: impl Fn(&FederalIncome) -> bool) -> i64 {
    s.draft.income.iter().find(|e| wanted(&e.income)).expect("a stored entry").id
}
// The page the session reopens the topic with, its rows.
fn reopened(s: &mut Interview, topic: TaxSection) -> Vec<IncomeDocument> {
    page_documents(&open_topic(s, topic)).iter().map(|item| item.document.clone()).collect()
}
// Saving the reopened page unchanged keeps the stored records.
fn round_trips(s: &mut Interview, topic: TaxSection) {
    let before = s.draft.income.clone();
    let rows = page_documents(&open_topic(s, topic)).to_vec();
    let request = save_topic(&s.draft.clone(), topic, rows);
    accepted(&send(s, request));
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
    accepted(&investment(&mut s, vec![IncomeDocument::EmployeeShareSale(rsu_sale(20260301))]));
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
        vec![IncomeDocument::EmployeeShareSale(rsu_sale(20260301))]
    );
    round_trips(&mut s, TaxSection::InvestmentSales);
    // Boundary: sold 2026-02-01, exactly one year after the 2025-02-01 vesting, is short-term. The
    // 5,000 is ordinary: taxable 48,900, Tax Table midpoint 48,925: 1,240 + 12% x 36,525 = 5,623.
    accepted(&investment(&mut s, vec![IncomeDocument::EmployeeShareSale(rsu_sale(20260201))]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_1b"), 5_000);
    assert_eq!(value(&r, "capital_gain_8b"), 0);
    assert_eq!(tax(&r), 562_300);
    // The special answer leaves the sale out with its notice.
    let mut left_out = rsu_sale(20260301);
    left_out.more.special = true;
    accepted(&investment(&mut s, vec![IncomeDocument::EmployeeShareSale(left_out.clone())]));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_EMPLOYEE_STOCK_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        vec![IncomeDocument::EmployeeShareSale(left_out)]
    );
}

#[test]
fn employee_share_sale_rules_hold_on_both_sides() {
    let mut s = single();
    let mut v = rsu_sale(20260301);
    v.taxed = 20260302;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::EmployeeShareSale(v)]),
        "EMPLOYEE_STOCK_SOLD_BEFORE_TAXED",
    );
    for sold in [20251231, 20270101] {
        rejected_with(
            investment(&mut s, vec![IncomeDocument::EmployeeShareSale(rsu_sale(sold))]),
            "EMPLOYEE_STOCK_SALE_NOT_IN_RETURN_YEAR",
        );
    }
    // The wash sale loss is at most the loss: basis max(value, paid) + fees = 20,000 + 100, proceeds
    // 19,000 gives a loss of 1,100.
    let mut v = rsu_sale(20260301);
    v.proceeds = 1_900_000;
    v.more.fees = 10_000;
    v.more.wash_sale = 110_000;
    accepted(&investment(&mut s, vec![IncomeDocument::EmployeeShareSale(v.clone())]));
    v.more.wash_sale = 110_001;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::EmployeeShareSale(v)]),
        "SALE_WASH_SALE_ABOVE_LOSS",
    );
}

// ---- F2 and F3: ESPP (Pub. 525 Examples 10 and 11) ----

fn espp_page(granted: u32, exercised: u32, sold: u32, in_wages: bool) -> EsppShareSale {
    EsppShareSale {
        description: "".into(),
        shares: 100.0,
        sold,
        proceeds: 300_000,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        granted,
        exercised,
        grant_market: 2_200,
        market_price: 2_300,
        exercise_price: 2_000,
        grant_price: 2_000,
        more: EsppShareSaleMore { in_wages, ..EsppShareSaleMore::default() },
    }
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
    accepted(&investment(&mut s, vec![IncomeDocument::EsppShareSale(page.clone())]));
    let id = id_of(&s, |i| matches!(i, FederalIncome::EsppSale { .. }));
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
        vec![IncomeDocument::EsppShareSale(page)]
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
    accepted(&investment(&mut s, vec![IncomeDocument::EsppShareSale(page.clone())]));
    let id = id_of(&s, |i| matches!(i, FederalIncome::EsppSale { .. }));
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
    accepted(&investment(
        &mut s,
        vec![IncomeDocument::EsppShareSale(espp_page(20240601, 20251201, 20260601, false))],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 61_000);
    assert_eq!(tax(&r), 514_300);
}

#[test]
fn espp_share_sale_rules_and_special_answer() {
    let mut s = single();
    let mut v = espp_page(20250601, 20250501, 20260701, false);
    rejected_with(
        investment(&mut s, vec![IncomeDocument::EsppShareSale(v.clone())]),
        "ESPP_EXERCISED_BEFORE_GRANTED",
    );
    v = espp_page(20231101, 20260801, 20260701, false);
    rejected_with(
        investment(&mut s, vec![IncomeDocument::EsppShareSale(v)]),
        "ESPP_SOLD_BEFORE_EXERCISED",
    );
    rejected_with(
        investment(
            &mut s,
            vec![IncomeDocument::EsppShareSale(espp_page(20231101, 20250501, 20270102, false))],
        ),
        "ESPP_SALE_NOT_IN_RETURN_YEAR",
    );
    // The wash sale bound is the server's (the basis needs the per-share products).
    let mut v = espp_page(20231101, 20250501, 20260701, false);
    v.proceeds = 100_000;
    v.more.wash_sale = 1_000_001;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::EsppShareSale(v)]),
        "SALE_WASH_SALE_ABOVE_LOSS",
    );
    let mut special = espp_page(20231101, 20250501, 20260701, true);
    special.more.special = true;
    accepted(&investment(&mut s, vec![IncomeDocument::EsppShareSale(special.clone())]));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_ESPP_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
    // A certificate of divestiture is a special treatment, not a stored field.
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales)
            .into_iter()
            .map(|d| match d {
                IncomeDocument::EsppShareSale(v) => v.more.special,
                other => panic!("{other:?}"),
            })
            .collect::<Vec<_>>(),
        vec![true]
    );
}

// ---- F4: NUA shares ----

// Distributed 2024-05-01 with a value of $50,000 and $30,000 of deferred NUA (basis $20,000), sold
// 2026-03-01 for $60,450.
fn nua_page(distributed: u32, proceeds: i64) -> NuaShareSale {
    NuaShareSale {
        description: "".into(),
        sold: 20260301,
        proceeds,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        distributed,
        distribution_value: 5_000_000,
        deferred_nua: 3_000_000,
        more: NuaShareSaleMore::default(),
    }
}

#[test]
fn nua_share_sale_splits_the_nua_from_the_post_distribution_gain() {
    // The NUA part 30,000 is long-term whatever the holding period; the rest, 60,450 - 50,000 =
    // 10,450, is long-term too (held from the distribution more than a year). AGI 100,450; taxable
    // 84,350; ordinary 43,900 (5,023); 0% on 49,450 - 43,900 = 5,550; 15% on 40,450 - 5,550 =
    // 34,900 = 5,235: tax 10,258. Both parts are code B lots in box A: Schedule D line 8b.
    let mut s = single();
    accepted(&investment(
        &mut s,
        vec![IncomeDocument::NuaShareSale(nua_page(20240501, 6_045_000))],
    ));
    let id = id_of(&s, |i| matches!(i, FederalIncome::NuaSale { .. }));
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
    accepted(&investment(
        &mut s,
        vec![IncomeDocument::NuaShareSale(nua_page(20250901, 6_045_000))],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_1b"), 10_450);
    assert_eq!(value(&r, "capital_gain_8b"), 30_000);
    assert_eq!(tax(&r), 1_117_500);
    // A loss has no realized NUA: proceeds 15,000 against the 20,000 basis is a (5,000) long-term
    // loss; AGI 55,000.
    accepted(&investment(
        &mut s,
        vec![IncomeDocument::NuaShareSale(nua_page(20240501, 1_500_000))],
    ));
    let id = id_of(&s, |i| matches!(i, FederalIncome::NuaSale { .. }));
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
    let mut v = nua_page(20240501, 6_045_000);
    v.more.inherited = Some(NuaInheritance { died: 20240401, value: 5_500_000, estate_tax: false });
    rejected_with(
        investment(&mut s, vec![IncomeDocument::NuaShareSale(v.clone())]),
        "NUA_DIED_BEFORE_DISTRIBUTED",
    );
    v.more.inherited.as_mut().unwrap().died = 20260401;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::NuaShareSale(v.clone())]),
        "NUA_SOLD_BEFORE_DIED",
    );
    v.more.inherited.as_mut().unwrap().died = 20250101;
    accepted(&investment(&mut s, vec![IncomeDocument::NuaShareSale(v.clone())]));
    assert_eq!(
        reopened(&mut s, TaxSection::InvestmentSales),
        vec![IncomeDocument::NuaShareSale(v)]
    );
    let mut v = nua_page(20240501, 6_045_000);
    v.sold = 20240401;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::NuaShareSale(v)]),
        "NUA_SOLD_BEFORE_DISTRIBUTED",
    );
    let mut v = nua_page(20240501, 6_045_000);
    v.sold = 20251231;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::NuaShareSale(v)]),
        "NUA_SALE_NOT_IN_2026",
    );
    let mut v = nua_page(20240501, 6_045_000);
    v.deferred_nua = 5_000_001;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::NuaShareSale(v)]),
        "NUA_ABOVE_DISTRIBUTION_VALUE",
    );
    let mut v = nua_page(20240501, 1_000_000);
    v.more.wash_sale = 1_000_001;
    rejected_with(
        investment(&mut s, vec![IncomeDocument::NuaShareSale(v)]),
        "SALE_WASH_SALE_ABOVE_LOSS",
    );
    let mut special = nua_page(20240501, 6_045_000);
    special.more.special = true;
    accepted(&investment(&mut s, vec![IncomeDocument::NuaShareSale(special)]));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_NUA_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
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
            IncomeDocument::EmployeeShareSale(rsu_sale(20260301)),
            IncomeDocument::EmployeeShareSale(zero),
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
        accepted(&investment(&mut s, vec![IncomeDocument::EmployeeShareSale(page)]));
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
        accepted(&investment(&mut s, vec![IncomeDocument::NuaShareSale(page)]));
        let FederalIncome::NuaSale { data } = &s.draft.income[1].income else { panic!() };
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
    // return of capital (1,000 gain on line 10) and Form 2439 (5,000 on line 11). Unit F4 adds a
    // basis-reported collectible (code C, 4,000 on 8b, not 8a), a basis-reported personal-use
    // loss (code L, 0 on 8b) and a Form 4684 casualty gain (5,000 on line 11). The sum of lines
    // 8a to 11 is the long-term net and the short-term lines add to zero, so with no carryover the
    // eight lines, 4 and 11 equal the return's capital gain.
    let mut d = with_incomes(vec![]);
    let s_page = |sold| rsu_sale(sold);
    for entry in [
        IncomeDocument::EmployeeShareSale(s_page(20260301)),
        IncomeDocument::NuaShareSale(nua_page(20240501, 6_045_000)),
        IncomeDocument::QsbsSale(qsbs_page(
            20080602,
            2_000_000,
            12_000_000,
            BasisReporting::NoForm1099B,
        )),
        IncomeDocument::SmallBusinessStockLoss(section_1244_cash(1_000_000, 200_000, 900_000)),
        IncomeDocument::WorthlessSecurity(worthless(500_000, 20250615)),
        IncomeDocument::CapitalReturn1099Div(capital_return(
            CapitalDistributionKind::Nondividend,
            300_000,
            200_000,
        )),
        IncomeDocument::Form2439(form_2439(500_000, 0, 0, 300_000, 105_000)),
    ] {
        add(&mut d, income(crate::interview::document_income(&entry)));
    }
    let mut reported_collectible = collectible_sale(10_000, 6_000, GainTerm::Long, false);
    let mut reported_personal_loss = personal_sale(5_000, 6_000, GainTerm::Long);
    for lot in [&mut reported_collectible, &mut reported_personal_loss] {
        let FederalIncome::Sale { data } = lot else { unreachable!() };
        data.basis_reporting = BasisReporting::ReportedToIrs;
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
        IncomeDocument::EmployeeShareSale(short_sale),
        IncomeDocument::SmallBusinessStockLoss(loss),
        IncomeDocument::WorthlessSecurity(worthless(500_000, 20251231)),
        IncomeDocument::BadDebt(cash_loan_debt(1_000_000, 200_000)),
        IncomeDocument::CapitalReturn1099Div(CapitalReturn1099Div {
            term: GainTerm::Short,
            ..capital_return(CapitalDistributionKind::Nondividend, 300_000, 200_000)
        }),
    ] {
        add(&mut d, income(crate::interview::document_income(&entry)));
    }
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: native_sale(GainTerm::Short, 4_000_000, Some(1_000_000)),
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

fn qsbs_page(issued: u32, basis: i64, proceeds: i64, reporting: BasisReporting) -> QsbsSale {
    QsbsSale {
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
        prior_eligible_gain: 0,
        issuer_qualified: true,
        active_business: true,
        no_disqualification: true,
        zone_60_exclusion: None,
        complete_history: true,
        special: false,
    }
}

#[test]
fn qsbs_sale_excludes_all_of_a_100_percent_lot_and_the_50_percent_boundary_is_28_percent_gain() {
    // 100%: issued 2015-03-02, basis 100,000, sold 2026-05-01 for 1,100,000, no 1099-B (box F).
    // Gain 1,000,000; the limit is the larger of 10,000,000 and 10 x 100,000: excluded 1,000,000,
    // taxable 0, no 28% gain, no AMT preference. AGI 60,000, tax 5,023.
    let mut s = single();
    let page = qsbs_page(20150302, 10_000_000, 110_000_000, BasisReporting::NoForm1099B);
    accepted(&investment(&mut s, vec![IncomeDocument::QsbsSale(page.clone())]));
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
    assert_eq!(reopened(&mut s, TaxSection::InvestmentSales), vec![IncomeDocument::QsbsSale(page)]);
    round_trips(&mut s, TaxSection::InvestmentSales);
    // 50%: issued 2008-06-02, basis 20,000, proceeds 120,000: gain 100,000, excluded 50,000, the
    // included 50,000 is 28% rate gain (worksheet line 2), AMT preference 7% x 50,000 = 3,500 (Form
    // 6251 line 2h). AGI 110,000; taxable 93,900; Tax Table midpoint 93,925: 1,240 + 4,560 + 22% x
    // 43,525 = 15,375.50, rounded 15,376; the Schedule D Tax Worksheet (43,900 at 5,023 plus 50,000
    // at 28% = 19,023) is higher, so 15,376. AMT: 93,900 + 16,100 + 3,500 = 113,500 less the 90,100
    // exemption = 23,400; at most 26% x 23,400 = 6,084 is below the regular tax: no AMT.
    let page = qsbs_page(20080602, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
    accepted(&investment(&mut s, vec![IncomeDocument::QsbsSale(page)]));
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
        page.issuer_qualified = false;
        accepted(&investment(&mut s, vec![IncomeDocument::QsbsSale(page)]));
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
        vec![IncomeDocument::QsbsSale(one), IncomeDocument::QsbsSale(two)],
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
        investment(&mut s, vec![IncomeDocument::QsbsSale(later)]),
        "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
    );
    // Issued before the section 1202 start, or not held more than five years.
    for (issued, sold) in [(19930810, 20260501), (20210501, 20260501), (20210502, 20260501)] {
        let mut page = qsbs_page(issued, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
        page.lots[0].sold = sold;
        rejected_with(
            investment(&mut s, vec![IncomeDocument::QsbsSale(page)]),
            "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
        );
    }
    // Issued on July 4, 2025 and sold in 2026 is still short of five years; issued 2021-04-30 and
    // sold 2026-05-01 is more than five years.
    let mut page = qsbs_page(20210430, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
    page.lots[0].sold = 20260501;
    accepted(&investment(&mut s, vec![IncomeDocument::QsbsSale(page)]));
    // The special answer leaves the sale out.
    let mut special = qsbs_page(20150302, 10_000_000, 110_000_000, BasisReporting::NoForm1099B);
    special.special = true;
    accepted(&investment(&mut s, vec![IncomeDocument::QsbsSale(special)]));
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
        accepted(&investment(&mut session, vec![IncomeDocument::QsbsSale(page.clone())]));
        supported(&session.draft);
        assert_eq!(
            reopened(&mut session, TaxSection::InvestmentSales),
            vec![IncomeDocument::QsbsSale(page)]
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
            investment(&mut session, vec![IncomeDocument::QsbsSale(invalid)]),
            "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
        );
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);

        let expected = qsbs_page(20150302, 100_000, 200_000, BasisReporting::NoForm1099B);
        accepted(&investment(&mut session, vec![IncomeDocument::QsbsSale(expected.clone())]));
        let mut page = open_topic(&mut session, TaxSection::InvestmentSales);
        let P::SaveIncomes { documents, .. } = &mut page else {
            panic!("expected the investment sales page")
        };
        let IncomeDocument::QsbsSale(sale) = &mut documents.documents[0].document else {
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
            vec![IncomeDocument::QsbsSale(expected)]
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
            investment(&mut s, vec![IncomeDocument::QsbsSale(page)]),
            "QSBS_LOT_PROCEEDS_NOT_ABOVE_BASIS",
        );
    }
    // Zone gain: the lot gains 10,000,000; 10,000,001 is above it, and a lot
    // issued after the zone dates takes none.
    let zone = |issued: u32, zone_gain: i64| {
        let mut page = qsbs_page(issued, 2_000_000, 12_000_000, BasisReporting::NoForm1099B);
        page.lots[0].zone_gain_through_2018 = Some(zone_gain);
        page
    };
    rejected_with(
        investment(&mut s, vec![IncomeDocument::QsbsSale(zone(20050301, 10_000_001))]),
        "QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN",
    );
    rejected_with(
        investment(&mut s, vec![IncomeDocument::QsbsSale(zone(20050301, -1))]),
        "QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN",
    );
    rejected_with(
        investment(&mut s, vec![IncomeDocument::QsbsSale(zone(20150302, 100))]),
        "QSBS_LOT_ZONE_GAIN_OUTSIDE_ZONE_DATES",
    );
    let mut ok = zone(20050301, 10_000_000);
    ok.zone_60_exclusion = Some(true);
    accepted(&investment(&mut s, vec![IncomeDocument::QsbsSale(ok)]));
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
    accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(page.clone())]));
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
        vec![IncomeDocument::SmallBusinessStockLoss(page)]
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
    accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(page.clone())]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 50_000);
    assert_eq!(value(&r, "capital_gain_10"), -9_000);
    assert_eq!(value(&r, "agi"), 7_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 9_000);
    assert_eq!(tax(&r), 0);
    // A joint return limit is 100,000 even if only one spouse has the loss: the whole 59,000 is
    // ordinary and no capital part remains.
    let mut d = complete(FederalStatus::Joint);
    let (stored, _) =
        (crate::interview::document_income(&IncomeDocument::SmallBusinessStockLoss(page)), 0);
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
    accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(property.clone())]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 50);
    assert_eq!(value(&r, "capital_gain_10"), -750);
    // Not sure about the qualification: all capital, with a notice.
    let mut unsure = section_1244_cash(1_000_000, 200_000, 900_000);
    unsure.ownership = crate::StockLossOwnership::OriginalIndividual { qualified: false };
    accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(unsure)]));
    let r = supported(&s.draft);

    assert_eq!(value(&r, "section_1244_ordinary_loss"), 0);
    assert_eq!(value(&r, "capital_gain_10"), -3_000);
    // Worthlessness not established: no 2026 loss, with a notice.
    let worthless = Section1244Stock {
        event: StockLossEvent::Worthless { confirmed: false },
        ..section_1244_cash(1_000_000, 0, 0)
    };
    accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(worthless)]));
    let r = supported(&s.draft);

    assert_eq!(value(&r, "agi"), 60_000);
    // The scope confirmation leaves the loss out.
    let mut left_out = section_1244_cash(1_000_000, 0, 0);
    left_out.ownership = crate::StockLossOwnership::OtherUnmodeledHistory;
    accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(left_out)]));
    assert!(noted(&supported(&s.draft), "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT"));
}

#[test]
fn section_1244_page_rules_hold_on_both_sides() {
    let mut s = single();
    let check = |s: &mut Interview, edit: fn(&mut Section1244Stock), key: &str| {
        let mut page = section_1244_cash(1_000_000, 200_000, 900_000);
        edit(&mut page);
        rejected_with(investment(s, vec![IncomeDocument::SmallBusinessStockLoss(page)]), key);
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
        vec![IncomeDocument::WorthlessSecurity(worthless(500_000, 20250615))],
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
        vec![IncomeDocument::WorthlessSecurity(worthless(500_000, 20251231))],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_3"), -5_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 2_000);
    // Not sure it is worthless: no 2026 loss, with a notice.
    let mut unsure = worthless(500_000, 20250615);
    unsure.worthless = false;
    accepted(&investment(&mut s, vec![IncomeDocument::WorthlessSecurity(unsure)]));
    let r = supported(&s.draft);

    assert_eq!(value(&r, "agi"), 60_000);
    // Acquired after 2026 is refused on both sides.
    rejected_with(
        investment(&mut s, vec![IncomeDocument::WorthlessSecurity(worthless(500_000, 20270101))]),
        "WORTHLESS_ACQUIRED_AFTER_2026",
    );
    let mut left_out = worthless(500_000, 20250615);
    left_out.ordinary = false;
    accepted(&investment(&mut s, vec![IncomeDocument::WorthlessSecurity(left_out)]));
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
        vec![IncomeDocument::BadDebt(cash_loan_debt(1_000_000, 200_000))],
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
    accepted(&investment(&mut s, vec![IncomeDocument::BadDebt(unsure)]));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "agi"), 60_000);
    // More recovered than advanced is refused on both sides.
    rejected_with(
        investment(&mut s, vec![IncomeDocument::BadDebt(cash_loan_debt(1_000_000, 1_000_001))]),
        "BAD_DEBT_RECOVERED_ABOVE_ADVANCED",
    );
    // The amounts the stored debt requires are at least one cent.
    let source = include_str!("../../federal/federal_model.rs");
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
        // 2026-10-16: the one-cent minimum remains after the redundant schema maximum is removed.
        assert_eq!(attribute.trim(), "#[libertas_number(min = 1)]", "{field}");
    }
}

// ---- F9: return of capital or liquidation ----

fn capital_return(kind: CapitalDistributionKind, amount: i64, basis: i64) -> CapitalReturn1099Div {
    CapitalReturn1099Div {
        payer: "".into(),
        kind,
        amount,
        property: vec![],
        basis,
        term: GainTerm::Long,
        special: false,
    }
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
        vec![IncomeDocument::CapitalReturn1099Div(page.clone())],
    ));
    // An unnamed source stores no name; its display name is LABEL_ENTRY_*.
    assert!(s.draft.income.last().unwrap().label.is_empty());
    let id = id_of(&s, |i| matches!(i, FederalIncome::ReturnOfCapital { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("distribution_{id}_basis_reduction")), 2_000);
    assert_eq!(value(&r, &format!("distribution_{id}_remaining_basis")), 0);
    assert_eq!(value(&r, &format!("distribution_{id}_capital_gain")), 1_000);
    assert_eq!(value(&r, "capital_gain_10"), 1_000);
    assert_eq!(value(&r, "agi"), 61_000);
    assert_eq!(tax(&r), 502_300);
    assert_eq!(
        reopened(&mut s, TaxSection::InterestDividends),
        vec![IncomeDocument::CapitalReturn1099Div(page)]
    );
    round_trips(&mut s, TaxSection::InterestDividends);
    // Within cost: box 3 of 1,500 is no gain, 500 of cost remains.
    let within = capital_return(CapitalDistributionKind::Nondividend, 150_000, 200_000);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeDocument::CapitalReturn1099Div(within)],
    ));
    let id = id_of(&s, |i| matches!(i, FederalIncome::ReturnOfCapital { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, &format!("distribution_{id}_remaining_basis")), 500);
    assert_eq!(value(&r, "agi"), 60_000);
    // A final liquidation payment of 1,200 against a cost of 2,000 is a (800) long-term loss; Not
    // sure is taken as not final: no loss and a notice.
    let liquidation = capital_return(CapitalDistributionKind::FinalLiquidation, 120_000, 200_000);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeDocument::CapitalReturn1099Div(liquidation.clone())],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain_10"), -800);
}

#[test]
fn liquidation_property_belongs_to_a_liquidation_and_the_1099_div_keeps_computing() {
    let mut s = single();
    let mut page = capital_return(CapitalDistributionKind::Nondividend, 0, 200_000);
    page.property = vec![LiquidationProperty { label: "Box 10".into(), value: 100_000 }];
    rejected_with(
        save(
            &mut s,
            TaxSection::InterestDividends,
            vec![IncomeDocument::CapitalReturn1099Div(page.clone())],
        ),
        "CAPITAL_RETURN_PROPERTY_NOT_LIQUIDATION",
    );
    page.kind = CapitalDistributionKind::FinalLiquidation;
    // Cash of 500 and property of 1,000 against a cost of 2,000: a (500) long-term loss.
    page.amount = 50_000;
    let div = div_1099(50_000);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeDocument::Div1099(div), IncomeDocument::CapitalReturn1099Div(page)],
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
        vec![IncomeDocument::CapitalReturn1099Div(special)],
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
) -> Form2439 {
    Form2439 {
        section_1202_exclusion: None,
        fund: "".into(),
        gain,
        unrecaptured,
        small_business,
        collectibles,
        tax_paid,
        special: false,
    }
}

#[test]
fn form_2439_reaches_schedule_d_line_11_and_schedule_3_line_13a() {
    // Box 1a 5,000 (long-term, Schedule D line 11), box 1d 3,000, box 2 1,050 (Schedule 3 line 13a).
    // The basis rises by 5,000 - 1,050 = 3,950. AGI 65,000; taxable 48,900: 2,000 of the gain at 0%
    // (up to 49,450 less ordinary income 43,900 is 5,550, but the 28% pool of 3,000 is taxed at the
    // ordinary 12%: 360). Tax 5,023 + 360 = 5,383; the 1,050 is a payment.
    let mut s = single();
    let page = form_2439(500_000, 0, 0, 300_000, 105_000);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeDocument::Form2439(page.clone())],
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
        vec![IncomeDocument::Form2439(page)]
    );
    round_trips(&mut s, TaxSection::InterestDividends);
    // Box 1c section 1202 gain is calculated through its exclusion choice (B9-5): the fund's whole
    // 5,000.00 stays on line 11 and the credit is unchanged; a code Q row excludes the 1,000.00.
    let mut section_1202 = form_2439(500_000, 0, 100_000, 300_000, 105_000);
    section_1202.section_1202_exclusion = Some(crate::FundQsbsExclusion::Hundred);
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeDocument::Form2439(section_1202)],
    ));
    let r = supported(&s.draft);
    assert!(!noted(&r, "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT"));
    assert_eq!(value(&r, "capital_gain_11"), 5_000);
    assert_eq!(value(&r, "undistributed_gain_tax_credit"), 1_050);
    assert_eq!(value(&r, "agi"), 64_000);
    // The ownership answer leaves it out with its own notice.
    let mut special = form_2439(500_000, 0, 0, 0, 105_000);
    special.special = true;
    accepted(&save(&mut s, TaxSection::InterestDividends, vec![IncomeDocument::Form2439(special)]));
    assert!(noted(&supported(&s.draft), "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT"));
    // The page rules.
    rejected_with(
        save(
            &mut s,
            TaxSection::InterestDividends,
            vec![IncomeDocument::Form2439(form_2439(500_000, 300_000, 100_000, 100_001, 0))],
        ),
        "FORM_2439_PARTS_ABOVE_1A",
    );
    rejected_with(
        save(
            &mut s,
            TaxSection::InterestDividends,
            vec![IncomeDocument::Form2439(form_2439(500_000, 0, 0, 0, 500_001))],
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
    b.more.intangibles = vec![BusinessIntangible {
        label: "Purchased business goodwill".into(),
        additions: vec![],
        cost: 15_000_000,
        acquired: 20240701,
        business_started: 20240701,
        eligible: true,
    }];
    accepted(&save_businesses(&mut s, vec![b.clone()]));
    let (id, stored) = stored_business(&s);
    assert_eq!(stored.intangibles, b.more.intangibles);
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
        IncomeDocument::Business(b.clone())
    );
    // Not sure it qualifies: no deduction, with a notice; No leaves it out.
    let mut unsure = b.clone();
    unsure.more.intangibles[0].eligible = false;
    accepted(&save_businesses(&mut s, vec![unsure]));
    let r = supported(&s.draft);

    assert!(!r.lines.iter().any(|l| l.id == format!("business_{id}_intangible_0_amortization")));
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
        },
        expense_treatment: FederalFuelExpenseTreatment::FullCostDeducted,
        records_complete: true,
    });
    farm.more.cash_fuel_tax_recovery = Some(cash_fuel_recovery(40_000));
    let request =
        farm_save(&s.draft.clone(), vec![document(0, IncomeDocument::Farm(farm.clone()))]);
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
    assert_eq!(documents, vec![IncomeDocument::Farm(farm)]);
}

// ---- Search, offered kinds and import mappings ----

#[test]
fn investment_sales_and_interest_dividends_offer_the_new_kinds_and_search_names_them() {
    let mut s = single();
    let sales = page_kinds(&mut s, TaxSection::InvestmentSales);
    for kind in [
        libertas_macros::variant_index!(IncomeDocument::EmployeeShareSale),
        libertas_macros::variant_index!(IncomeDocument::EsppShareSale),
        libertas_macros::variant_index!(IncomeDocument::NuaShareSale),
        libertas_macros::variant_index!(IncomeDocument::QsbsSale),
        libertas_macros::variant_index!(IncomeDocument::SmallBusinessStockLoss),
        libertas_macros::variant_index!(IncomeDocument::WorthlessSecurity),
        libertas_macros::variant_index!(IncomeDocument::BadDebt),
    ] {
        assert!(sales.contains(&(kind as i32)), "{kind}");
    }
    let dividends = page_kinds(&mut s, TaxSection::InterestDividends);
    for kind in [
        libertas_macros::variant_index!(IncomeDocument::CapitalReturn1099Div),
        libertas_macros::variant_index!(IncomeDocument::Form2439),
    ] {
        assert!(dividends.contains(&(kind as i32)), "{kind}");
    }
    let protocol = include_str!("../../core/protocol.rs");
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
    let prompt = crate::interview::resource_text("TAX_NAVIGATION_PROMPT");
    assert!(prompt.contains("ESPP, Form 3922, RSUs, nonqualified stock options, NUA shares, section 1202 and section 1244 stock, worthless securities and bad debts belong to Investment sales"));
    assert!(prompt.contains("return of capital, 1099-DIV box 3, liquidating distributions and Form 2439 belong to Interest and dividends"));
}

fn page_kinds(s: &mut Interview, topic: TaxSection) -> Vec<i32> {
    match open_topic(s, topic) {
        P::SaveIncomes { allowed_documents, .. } => allowed_documents,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_stock_compensation_notice_and_help_name_the_four_kinds() {
    let notice = crate::interview::resource_text("WARN_SALE_STOCK_COMPENSATION_LEFT_OUT");
    for kind in
        ["Employee stock shares sold", "ESPP shares sold", "ISO shares sold", "NUA shares sold"]
    {
        assert!(notice.contains(kind), "{kind}");
    }
    let model = include_str!("../../federal/federal_model.rs");
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
fn share_sale_imports_read_the_1099_b_boxes_and_leave_the_plan_records() {
    let employee = include_str!("../../income/employee_stock.rs");
    let action = import_action(employee, "EmployeeShareSale");
    assert!(action.contains("Read the 1099-B sale of these employee stock shares exactly"));
    // Box 1e and box 12 are printed facts of the 1099-B (F3, 2026-10-09): the broker's basis
    // and whether it reported it to the IRS decide the Form 8949 box and code B.
    assert!(action.contains("Return box 1e as reported_basis exactly as printed"));
    assert!(action.contains("return box 12 as reporting; do not return box 1b"));
    assert!(action.contains("validation = \"$.document_facts\""));
    assert_eq!(
        import_targets(employee, "EmployeeShareSale"),
        ["description", "sold", "proceeds", "reported_basis", "reporting", "withholding"]
    );
    let espp = include_str!("../../income/espp_sale.rs");
    let action = import_action(espp, "EsppShareSale");
    assert!(action.contains("Do not return Form 3922 values even when a Form 3922 is supplied"));
    assert_eq!(
        import_targets(espp, "EsppShareSale"),
        ["description", "shares", "sold", "proceeds", "reported_basis", "reporting", "withholding"]
    );
    let nua = include_str!("../../retirement/nua_sale.rs");
    let action = import_action(nua, "NuaShareSale");
    assert!(
        action.contains("do not return Form 1099-R values even when a Form 1099-R is supplied")
    );
    assert_eq!(
        import_targets(nua, "NuaShareSale"),
        ["description", "sold", "proceeds", "reported_basis", "reporting", "withholding"]
    );
    // The page values a document cannot supply stay with the person when an import replaces the
    // targets.
    let mut page = rsu_sale(20260301);
    page.more.special = true;
    let kept = (page.taxed, page.value, page.paid, page.more.clone());
    (page.sold, page.proceeds, page.withholding) = (20260315, 2_600_000, 5_000);
    assert_eq!((page.taxed, page.value, page.paid, page.more.clone()), kept);
}

#[test]
fn qsbs_import_reads_the_sale_rows_and_leaves_the_issuer_answers() {
    let source = include_str!("../../investment/qsbs.rs");
    let action = import_action(source, "QsbsSale");
    assert!(action.contains("Return one lots row for each sale of that corporation's shares"));
    assert!(action.contains("If the document shows sales of more than one corporation's stock"));
    assert!(action.contains("do not decide whether the stock qualifies"));
    assert_eq!(import_targets(source, "QsbsSale"), ["issuer", "lots", "reporting", "withholding"]);
    assert_eq!(import_targets(source, "QsbStockLot"), ["issued", "sold", "basis", "proceeds"]);
}

#[test]
fn return_of_capital_and_form_2439_imports_read_the_printed_boxes() {
    let model = include_str!("../../federal/federal_model.rs");
    let action = import_action(model, "CapitalReturn1099Div");
    assert!(action.contains(
        "Read box 3 nondividend distributions, or box 9 cash liquidation distributions, into amount"
    ));
    assert!(action.contains("Read box 10 noncash liquidation distributions into one property row"));
    assert!(action.contains("Do not read boxes 1a through 2f, 4 through 8 or 11 through 16"));
    assert_eq!(import_targets(model, "CapitalReturn1099Div"), ["payer", "amount", "property"]);
    let action = import_action(model, "Form2439");
    assert!(action.contains("box 1a total undistributed long-term capital gains"));
    assert!(action.contains("box 2 tax paid"));
    assert_eq!(
        import_targets(model, "Form2439"),
        ["fund", "gain", "unrecaptured", "small_business", "collectibles", "tax_paid"]
    );
    // The normal1099-DIV import now retains boxes3/9/10 as printed totals,
    // while its personal lot facts are not an import target.
    let div = import_action(model, "Div1099");
    assert!(div.contains("Read box 3 nondividend distributions, box 9 cash liquidation distributions, and box 10 noncash liquidation distributions"));
    assert!(div.contains("preserve the personal capital_returns list"));
    assert!(div.contains("box 2c section 1202 gain into section_1202_gain"));
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

fn preview(s: &mut Interview, document: IncomeDocument) -> Option<i64> {
    let mut request = crate::tests::support::client::survey::survey_preview_document(document);
    let P::PreviewDocument { foreign_countries, .. } = &mut request else { unreachable!() };
    *foreign_countries = crate::foreign_accounts::countries();
    match send(s, request) {
        Reply::Response(P::IncomePreview { preview, .. }) => preview,
        other => panic!("{other:?}"),
    }
}

#[test]
fn amount_on_the_return_previews_the_supported_projection_of_each_new_kind() {
    let mut s = single();
    // The sale's gain, not its wage compensation.
    assert_eq!(
        preview(&mut s, IncomeDocument::EmployeeShareSale(rsu_sale(20260301))),
        Some(500_000)
    );
    // A QSBS issuer's amount depends on every entry of the issuer.
    assert_eq!(
        preview(
            &mut s,
            IncomeDocument::QsbsSale(qsbs_page(
                20080602,
                2_000_000,
                12_000_000,
                BasisReporting::NoForm1099B
            ))
        ),
        None
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeDocument::SmallBusinessStockLoss(section_1244_cash(1_000_000, 200_000, 900_000))
        ),
        Some(-300_000)
    );
    assert_eq!(
        preview(&mut s, IncomeDocument::WorthlessSecurity(worthless(500_000, 20250615))),
        Some(-500_000)
    );
    assert_eq!(
        preview(&mut s, IncomeDocument::BadDebt(cash_loan_debt(1_000_000, 200_000))),
        Some(-800_000)
    );
    assert_eq!(
        preview(
            &mut s,
            IncomeDocument::CapitalReturn1099Div(capital_return(
                CapitalDistributionKind::Nondividend,
                300_000,
                200_000
            ))
        ),
        Some(100_000)
    );
    assert_eq!(
        preview(&mut s, IncomeDocument::Form2439(form_2439(500_000, 0, 0, 300_000, 105_000))),
        Some(500_000)
    );
    // A document left out of the estimate previews zero.
    let mut left_out = rsu_sale(20260301);
    left_out.more.special = true;
    assert_eq!(preview(&mut s, IncomeDocument::EmployeeShareSale(left_out)), Some(0));
    assert_eq!(
        preview(
            &mut s,
            IncomeDocument::Form2439({
                let mut page = form_2439(500_000, 0, 100_000, 0, 105_000);
                page.section_1202_exclusion = Some(crate::FundQsbsExclusion::Hundred);
                page
            })
        ),
        Some(400_000)
    );
    let mut unsure = worthless(500_000, 20250615);
    unsure.worthless = false;
    assert_eq!(preview(&mut s, IncomeDocument::WorthlessSecurity(unsure)), Some(0));
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
        ("LABEL_ENTRY_CAPITAL_RETURN", "1099-DIV return of capital {0,number,0}"),
        ("LABEL_ENTRY_FORM_2439", "Form 2439 {0,number,0}"),
    ] {
        assert_eq!(crate::interview::resource_text(key), text);
    }
}

#[test]
fn a_1099_div_with_foreign_tax_round_trips_beside_a_return_of_capital() {
    let mut s = single();
    let mut div = div_1099(50_000);
    div.more.foreign_tax = 1_200;
    let capital = IncomeDocument::CapitalReturn1099Div(capital_return(
        CapitalDistributionKind::Nondividend,
        300_000,
        200_000,
    ));
    accepted(&save(
        &mut s,
        TaxSection::InterestDividends,
        vec![IncomeDocument::Div1099(div.clone()), capital.clone()],
    ));
    let rows = reopened(&mut s, TaxSection::InterestDividends);
    assert!(rows.contains(&IncomeDocument::Div1099(div)));
    assert!(rows.contains(&capital));
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
    spouse.owner = Owner::Spouse;
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
    b.more.intangibles = vec![BusinessIntangible {
        label: "Purchased business goodwill".into(),
        additions: vec![],
        cost: 15_000_000,
        acquired: 20260201,
        business_started: 20260201,
        eligible: true,
    }];
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
    accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(stock)]));
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
        accepted(&investment(&mut s, vec![IncomeDocument::SmallBusinessStockLoss(stock)]));
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
fn federal_cap_widening_keeps_form_2439_overflow_parts_priority() {
    let form = form_2439(i64::MAX, i64::MAX, 1, 0, i64::MAX);
    let issue = form.issue().unwrap();
    assert_eq!(issue.key, "FORM_2439_PARTS_ABOVE_1A");
    assert_eq!(issue.field.unwrap().square(), "[gain]");
}

#[test]
fn employee_stock_preflight_compares_basis_beyond_i64() {
    let mut page = rsu_sale(20260301);
    page.value = i64::MAX;
    page.more.fees = i64::MAX;
    page.more.wash_sale = 1;
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::EmployeeShareSale(page.clone())),
        None
    );
    assert!(!page.record().valid());
}

#[test]
fn espp_quantity_above_native_domain_is_refused_before_commit() {
    let mut session = single();
    let before = session.draft.clone();
    let mut page = espp_page(20231101, 20250501, 20260701, true);
    page.shares = f64::MAX;
    rejected_with(
        investment(&mut session, vec![IncomeDocument::EsppShareSale(page)]),
        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn section1244_preflight_compares_cash_basis_beyond_i64() {
    let page = section_1244_cash(i64::MAX, i64::MAX, i64::MAX);
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::SmallBusinessStockLoss(page)),
        None
    );
}
