// Independent regular/AMT source allocations. These verify the mapped sale
// exceptions and the residual code-loss guard, not full R9 return acceptance.
use super::*;
use crate::k1_items::K1Items;
use crate::k1_loss::{K1LossClass::OtherLoss, LossLedger};
use crate::tests::passthrough::k1_sale_return::{draft, sources};

fn refused_with(result: &FederalResult, key: &str) -> bool {
    result.issues.iter().any(|issue| issue.display.key == key)
}

fn sale(loss: i64) -> K1Section179Sale {
    k1_reported_sale(
        "Synthetic short-held machine",
        20260101,
        [(10_000 - loss) * 100, 1_000_000, 0, 0],
    )
}

fn start(amount: i64) -> ScorpRiskStart {
    ScorpRiskStart::Detailed { capacity: amount * 100, profit: 0, loss: 0 }
}

pub(in crate::tests) fn disjoint_view_counterpart(
    source: &mut FederalIncome,
    regular: Vec<K1Section179Sale>,
    amt: Vec<K1Section179Sale>,
    basis: [i64; 2],
    risk: [i64; 2],
) {
    // These old unkeyed vectors prove independent monetary worksheets, not
    // physical correspondence. This explicitly disjoint synthetic counterpart
    // preserves every original amount/order without pairing by any attribute.
    // Separate identity controls declare actual shared physical sources.
    let mut properties = regular;
    for sale in &mut properties {
        sale.alternative = crate::K1Section179SaleAmtSource::NotReported;
    }
    properties.extend(amt.into_iter().map(|mut sale| {
        sale.alternative = crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
        sale.regular = None;
        sale
    }));
    match source {
        FederalIncome::Partnership { data } => {
            data.section179_sales = properties;
            data.regular.basis.opening = basis[0] * 100;
            data.regular.risk.start = start(risk[0]);
            let mut alternative = data.regular.clone();
            alternative.basis.opening = basis[1] * 100;
            alternative.risk.start = start(risk[1]);
            data.alternative = Some(alternative);
        }
        FederalIncome::Scorp { data } => {
            data.section179_sales = properties;
            data.opening = basis[0] * 100;
            data.risk.start = start(risk[0]);
            let mut alternative = amt_records(data);
            alternative.stock = basis[1] * 100;
            alternative.risk.start = start(risk[1]);
            data.amt = Some(alternative);
        }
        _ => unreachable!(),
    }
}

fn code_loss(source: &mut FederalIncome, amount: i64) {
    match source {
        FederalIncome::Partnership { data } => data.income.push(PartnerIncome {
            code: PartnerIncomeCode::NonportfolioShort,
            amount: -amount * 100,
        }),
        FederalIncome::Scorp { data } => data
            .income
            .push(ScorpIncome { code: ScorpIncomeCode::NonportfolioShort, amount: -amount * 100 }),
        _ => unreachable!(),
    }
}

fn ledger(source: &FederalIncome, amt: bool) -> (LossLedger, K1Items) {
    match source {
        FederalIncome::Partnership { data } => {
            let source = if amt { data.alternative.as_ref().unwrap() } else { &data.regular };
            (
                data.ledger(source).unwrap(),
                source.source_k1_items(&data.items_for_record(source).unwrap()).unwrap(),
            )
        }
        FederalIncome::Scorp { data } => {
            let source: crate::scorp_amt::ScorpSource<'_> =
                if amt { data.amt.as_ref().unwrap().source(data).unwrap() } else { data.into() };
            (source.ledger().unwrap(), source.k1_items())
        }
        _ => unreachable!(),
    }
}

fn check(
    source: FederalIncome,
    allowed_class: [i64; 2],
    allowed_sales: [Vec<i64>; 2],
    refusal: bool,
) {
    let original = source.clone();
    let (regular, regular_items) = ledger(&source, false);
    let (amt, amt_items) = ledger(&source, true);
    assert_eq!(
        [
            crate::kernel::ratio::dollars(regular.basis_allowed[OtherLoss].get()),
            crate::kernel::ratio::dollars(amt.basis_allowed[OtherLoss].get())
        ],
        allowed_class
    );
    for (ledger, items, expected) in
        [(&regular, &regular_items, &allowed_sales[0]), (&amt, &amt_items, &allowed_sales[1])]
    {
        let basis = LossLedger { risk_allowed: ledger.basis_allowed, ..*ledger };
        assert_eq!(
            items.flows(&basis).sales.iter().map(|(_, _, cents)| cents / 100).collect::<Vec<_>>(),
            *expected
        );
    }
    assert_eq!(
        crate::k1_loss::amt_limit_differs(&regular, &amt, &regular_items, &amt_items),
        refusal
    );
    // Exercise the actual collector's source selection, not only its helper.
    let (d, _) = draft(source);
    let result = estimate(&d);
    assert_eq!(refused_with(&result, "ISSUE_K1_AMT_LIMIT_DIFFERS"), refusal);
    if refusal {
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(result.lines.is_empty());
    } else {
        assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
        assert!(result.issues.is_empty());
    }
    let guard = match &d.income[0].income {
        FederalIncome::Partnership { .. } => "ISSUE_PARTNER_K1_OTHER_INFORMATION",
        FederalIncome::Scorp { .. } => "ISSUE_SCORP_K1_OTHER_INFORMATION",
        _ => unreachable!(),
    };
    assert!(!refused_with(&result, guard));
    assert_eq!(d.income[0].income, original);
}

#[test]
fn k1_sale_amt_basis_difference_has_a_mapped_sale_destination() {
    for mut source in sources(0, 0, 0, 20_000) {
        let regular = sale(4_000);
        let mut amt = regular.clone();
        amt.regular.as_mut().unwrap().depreciation = 200_000; // AMT loss $2,000; regular loss $4,000.
        disjoint_view_counterpart(&mut source, vec![regular], vec![amt], [3_000; 2], [10_000; 2]);
        check(source, [3_000, 2_000], [vec![-3_000], vec![-2_000]], false);
    }
}

#[test]
fn k1_sale_amt_sale_only_in_one_tax_system_is_still_mapped() {
    for regular in [false, true] {
        for mut source in sources(0, 0, 0, 20_000) {
            let (rows, other, allowed, sales) = if regular {
                (vec![sale(2_000)], vec![], [2_000, 0], [vec![-2_000], vec![]])
            } else {
                (vec![], vec![sale(2_000)], [0, 2_000], [vec![], vec![-2_000]])
            };
            disjoint_view_counterpart(&mut source, rows, other, [10_000; 2], [10_000; 2]);
            check(source, allowed, sales, false);
        }
    }
}

#[test]
fn k1_sale_amt_independent_depreciation_can_change_a_loss_to_a_gain() {
    for mut source in sources(0, 0, 0, 20_000) {
        let regular = sale(4_000);
        let mut amt = regular.clone();
        amt.regular.as_mut().unwrap().depreciation = 600_000; // AMT basis $4,000; price $6,000.
        disjoint_view_counterpart(&mut source, vec![regular], vec![amt], [3_000; 2], [10_000; 2]);
        let (d, _) = draft(source.clone());
        let entry = &d.income[0];
        let facts = match &entry.income {
            FederalIncome::Partnership { data } => data.totals_facts(entry),
            FederalIncome::Scorp { data } => data.totals_facts(entry),
            _ => unreachable!(),
        };
        assert_eq!(facts.flows.ordinary, -300_000);
        assert_eq!(facts.amt_flows.ordinary, 200_000);
        check(source, [3_000, 0], [vec![-3_000], vec![2_000]], false);
    }
}

#[test]
fn k1_sale_amt_residual_comparison_keeps_original_code_cents_unchanged() {
    for mut source in sources(0, 0, 0, 20_000) {
        code_loss(&mut source, 1);
        match &mut source {
            FederalIncome::Partnership { data } => data.income[0].amount = -149,
            FederalIncome::Scorp { data } => data.income[0].amount = -149,
            _ => unreachable!(),
        }
        disjoint_view_counterpart(&mut source, vec![sale(2)], vec![sale(4)], [2, 3], [10_000; 2]);
        // 2026-10-19: original code149c shares source capacity with each sale.
        // Regular200c over349c admits85c code/115c sale; AMT300c over549c
        // admits81c code/219c sale. The original source stays149c.
        let (regular, items) = ledger(&source, false);
        let (amt, amt_items) = ledger(&source, true);
        assert_eq!(items.flows(&regular).nonportfolio[0], -85);
        assert_eq!(amt_items.flows(&amt).nonportfolio[0], -81);
        // 2026-10-19 correction: i6251 (2025 final), line 2k, recomputes
        // Schedule D with AMT amounts. The existing capital owner carries
        // this four-cent difference before the form's dollar projection.
        let (d, _) = draft(source.clone());
        let entry = &d.income[0];
        let facts = match &entry.income {
            FederalIncome::Partnership { data } => data.totals_facts(entry),
            FederalIncome::Scorp { data } => data.totals_facts(entry),
            _ => unreachable!(),
        };
        let mut totals = crate::federal_rules::Totals::default();
        crate::k1_totals::apply(&facts, &mut totals, &mut FederalResult::empty(d.revision));
        assert_eq!(totals.amt_sale_adjustment, [4, 0]);
        check(source, [2, 3], [vec![-1], vec![-2]], false);
    }
}

#[test]
fn k1_sale_amt_equal_class_totals_keep_independent_capital_code_losses() {
    for mut source in sources(0, 0, 0, 20_000) {
        code_loss(&mut source, 4_000);
        disjoint_view_counterpart(
            &mut source,
            vec![sale(4_000)],
            vec![sale(2_000)],
            [3_000; 2],
            [10_000; 2],
        );
        // Regular: $1,500 sale / $1,500 code; AMT: $1,000 sale / $2,000 code.
        // 2026-10-19 correction: i6251 (2025 final), line 2k; the
        // existing capital adjustment owns the -$500 code difference.
        check(source, [3_000; 2], [vec![-1_500], vec![-1_000]], false);
    }
}

#[test]
fn k1_sale_amt_different_class_totals_with_equal_residual_codes_are_mapped() {
    for mut source in sources(0, 0, 0, 20_000) {
        code_loss(&mut source, 4_000);
        disjoint_view_counterpart(
            &mut source,
            vec![sale(4_000)],
            vec![sale(8_000)],
            [2_000, 3_000],
            [10_000; 2],
        );
        // Both systems allow $1,000 of code loss. Only the mapped sale share
        // differs: $1,000 regular, $2,000 AMT.
        check(source, [2_000, 3_000], [vec![-1_000], vec![-2_000]], false);
    }
}

#[test]
fn k1_sale_amt_positive_sale_keeps_independent_capital_code_limits() {
    for mut source in sources(0, 0, 0, 20_000) {
        code_loss(&mut source, 4_000);
        disjoint_view_counterpart(
            &mut source,
            vec![sale(-10_000)],
            vec![sale(-1_000)],
            [0; 2],
            [10_000; 2],
        );
        // 2026-10-19 correction: i6251 (2025 final), line 2k; the
        // mapped capital owner admits the +$3,000 code adjustment.
        check(source, [4_000, 1_000], [vec![10_000], vec![1_000]], false);
    }
}

#[test]
fn k1_sale_amt_two_properties_keep_the_basis_allocation_and_source_order() {
    for mut source in sources(0, 0, 0, 20_000) {
        disjoint_view_counterpart(
            &mut source,
            vec![sale(2_000), sale(6_000)],
            vec![sale(6_000), sale(2_000)],
            [3_000; 2],
            [10_000; 2],
        );
        check(source, [3_000; 2], [vec![-750, -2_250], vec![-2_250, -750]], false);
    }
}

#[test]
fn k1_sale_amt_basis_exception_does_not_change_the_later_risk_limit() {
    for mut source in sources(0, 0, 0, 20_000) {
        disjoint_view_counterpart(
            &mut source,
            vec![sale(4_000)],
            vec![sale(2_000)],
            [3_000; 2],
            [1_000; 2],
        );
        let (regular, items) = ledger(&source, false);
        let (amt, amt_items) = ledger(&source, true);
        assert_eq!(
            [
                crate::kernel::ratio::dollars(regular.risk_allowed[OtherLoss].get()),
                crate::kernel::ratio::dollars(amt.risk_allowed[OtherLoss].get())
            ],
            [1_000; 2]
        );
        assert_eq!(items.flows(&regular).ordinary, -100_000);
        assert_eq!(amt_items.flows(&amt).ordinary, -100_000);
        check(source, [3_000, 2_000], [vec![-3_000], vec![-2_000]], false);
    }
}

#[test]
fn k1_sale_amt_independent_risk_losses_reach_the_existing_disposition_totals() {
    for mut source in sources(0, 0, 0, 20_000) {
        disjoint_view_counterpart(
            &mut source,
            vec![sale(4_000)],
            vec![sale(4_000)],
            [4_000; 2],
            [1_000, 3_000],
        );
        let (d, _) = draft(source.clone());
        let entry = &d.income[0];
        let facts = match &entry.income {
            FederalIncome::Partnership { data } => data.totals_facts(entry),
            FederalIncome::Scorp { data } => data.totals_facts(entry),
            _ => unreachable!(),
        };
        assert_eq!(facts.flows.ordinary, -100_000);
        assert_eq!(facts.amt_flows.ordinary, -300_000);
        let mut totals = crate::federal_rules::Totals::default();
        let mut result = FederalResult::empty(d.revision);
        crate::k1_totals::apply(&facts, &mut totals, &mut result);
        assert_eq!(totals.k1_ordinary, [-100_000, -300_000]);
        // This is the existing AMT ordinary disposition input: -$2,000.
        // It is not another deduction or a stored carryover rewrite.
        assert_eq!((totals.k1_ordinary[1] - totals.k1_ordinary[0]) / 100, -2_000);
        check(source, [4_000; 2], [vec![-4_000], vec![-4_000]], false);
    }
}

#[test]
fn k1_sale_amt_unavailable_is_not_an_absent_or_copied_monetary_view() {
    for mut source in sources(15_000, 0, 0, 30_000) {
        let sales = match &mut source {
            FederalIncome::Partnership { data } => &mut data.section179_sales,
            FederalIncome::Scorp { data } => &mut data.section179_sales,
            _ => unreachable!(),
        };
        sales[0].alternative = K1Section179SaleAmtSource::Unavailable;
        assert_eq!(sales[0].monetary_view(true), None);
        assert!(sales[0].monetary_view(false).unwrap().is_some());
        let (d, _) = draft(source);
        let entry = &d.income[0];
        let mut regular_result = FederalResult::empty(d.revision);
        let regular_pool =
            crate::extended_rules::rentals_for(&d, 200_000, &mut regular_result, false);
        let regular = crate::k1_sales::facts(
            entry,
            false,
            regular_pool
                .k1_ledgers
                .iter()
                .find(|(id, _)| *id == entry.id)
                .map(|(_, ledger)| ledger),
        )
        .unwrap()
        .unwrap();
        assert_eq!(regular.values[0].gain, 8_000);
        assert_eq!(
            crate::k1_sales::facts(entry, true, None).err().unwrap().0,
            "ISSUE_K1_SALE_AMT_UNAVAILABLE"
        );
        let result = estimate(&d);
        assert!(refused_with(&result, "ISSUE_K1_SALE_AMT_UNAVAILABLE"), "{:?}", result.issues);
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(result.lines.is_empty());
        let mut parent_missing = d.clone();
        match &mut parent_missing.income[0].income {
            FederalIncome::Partnership { data } => data.alternative = None,
            FederalIncome::Scorp { data } => data.amt = None,
            _ => unreachable!(),
        }
        assert!(parent_missing.valid_stored());
        assert_eq!(
            crate::k1_sales::facts(&parent_missing.income[0], true, None).err().unwrap().0,
            "ISSUE_K1_SALE_AMT_UNAVAILABLE"
        );
        let parent_result = estimate(&parent_missing);
        let parent_key = match &parent_missing.income[0].income {
            FederalIncome::Partnership { .. } => {
                "ISSUE_EXTENDED_RULES_COMPLETE_SEPARATE_AMT_PARTNERSHIP_ACTIVITY_RECORDS_ABC340B8"
            }
            FederalIncome::Scorp { .. } => "ISSUE_SCORP_AMT_BASIS_AND_LOSS_RECORDS",
            _ => unreachable!(),
        };
        // The activity owner rejects missing parent records before its sale
        // worksheet runs. Keep that existing, more general source boundary.
        assert_eq!(parent_result.state, ResultState::Unsupported);
        assert!(refused_with(&parent_result, parent_key), "{:?}", parent_result.issues);
        assert!(parent_result.lines.is_empty());
        let mut known = d.clone();
        let sales = match &mut known.income[0].income {
            FederalIncome::Partnership { data } => &mut data.section179_sales,
            FederalIncome::Scorp { data } => &mut data.section179_sales,
            _ => unreachable!(),
        };
        sales[0].alternative = K1Section179SaleAmtSource::NotReported;
        assert_eq!(sales[0].monetary_view(true), Some(None));
        assert!(crate::k1_sales::facts(&known.income[0], true, None).unwrap().is_none());
        assert!(!refused_with(&estimate(&known), "ISSUE_K1_SALE_AMT_UNAVAILABLE"));
    }
}

#[test]
fn k1_sale_amt_unmapped_section1256_difference_retains_its_refusal() {
    for mut source in sources(0, 0, 0, 20_000) {
        code_loss(&mut source, 4_000);
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.income[0].code = PartnerIncomeCode::Section1256;
            }
            FederalIncome::Scorp { data } => {
                data.income[0].code = ScorpIncomeCode::Section1256;
            }
            _ => unreachable!(),
        }
        disjoint_view_counterpart(
            &mut source,
            vec![sale(4_000)],
            vec![sale(2_000)],
            [3_000; 2],
            [10_000; 2],
        );
        // Unlike Q/S, the Form 6781 owner has no separate AMT flow here.
        // Regular allows $1,500 of code C; AMT allows $2,000.
        check(source, [3_000; 2], [vec![-1_500], vec![-1_000]], true);
    }
}

#[test]
fn k1_sale_amt_capital_codes_keep_each_term_and_system_after_basis_allocation() {
    for long in [false, true] {
        for preferred in [false, true] {
            for mut source in sources(0, 0, 0, 20_000) {
                if preferred && matches!(source, FederalIncome::Scorp { .. }) {
                    continue; // Code Q is a partnership code only.
                }
                code_loss(&mut source, 4_000);
                match &mut source {
                    FederalIncome::Partnership { data } => {
                        data.income[0].code = match (preferred, long) {
                            (true, false) => PartnerIncomeCode::PreferredStockShort,
                            (true, true) => PartnerIncomeCode::PreferredStockLong,
                            (false, false) => PartnerIncomeCode::NonportfolioShort,
                            (false, true) => PartnerIncomeCode::NonportfolioLong,
                        };
                    }
                    FederalIncome::Scorp { data } => {
                        data.income[0].code = if long {
                            ScorpIncomeCode::NonportfolioLong
                        } else {
                            ScorpIncomeCode::NonportfolioShort
                        };
                    }
                    _ => unreachable!(),
                }
                disjoint_view_counterpart(
                    &mut source,
                    vec![sale(4_000)],
                    vec![sale(2_000)],
                    [3_000; 2],
                    [10_000; 2],
                );
                let (regular, items) = ledger(&source, false);
                let (amt, amt_items) = ledger(&source, true);
                assert_eq!(
                    items.flows(&regular).capital(),
                    if long { [0, -150_000] } else { [-150_000, 0] }
                );
                assert_eq!(
                    amt_items.flows(&amt).capital(),
                    if long { [0, -200_000] } else { [-200_000, 0] }
                );
                check(source, [3_000; 2], [vec![-1_500], vec![-1_000]], false);
            }
        }
    }
}

#[test]
fn k1_sale_amt_capital_code_native_save_reopen_restart_and_correction() {
    use crate::tests::support::assert::accepted;
    use crate::tests::support::client::income_pages::open_topic;
    use crate::tests::support::client::{
        restart, roundtrip_native_draft, roundtrip_native_request, save_reopen_native_sources, send,
    };

    for mut source in sources(0, 0, 0, 20_000) {
        code_loss(&mut source, 4_000);
        disjoint_view_counterpart(
            &mut source,
            vec![sale(4_000)],
            vec![sale(2_000)],
            [3_000; 2],
            [10_000; 2],
        );
        let (original, id) = draft(source);
        let saved = save_reopen_native_sources(
            original,
            &[TaxSection::PassThrough],
            &[TaxSection::PassThrough],
        );
        assert_eq!(saved.income[0].id, id);
        let (regular, items) = ledger(&saved.income[0].income, false);
        let (amt, amt_items) = ledger(&saved.income[0].income, true);
        assert_eq!(items.flows(&regular).capital(), [-150_000, 0]);
        assert_eq!(amt_items.flows(&amt).capital(), [-200_000, 0]);
        let mut session = restart(saved);
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let document = &mut documents.documents[0];
        assert_eq!(document.id, id);
        let sales_before = match &mut document.document {
            IncomeDocument::K1Partnership(page) => {
                page.more.income[0].amount = -200_000;
                page.more.section179_sales.clone()
            }
            IncomeDocument::K1Scorp(page) => {
                page.more.income[0].amount = -200_000;
                page.more.section179_sales.clone()
            }
            _ => unreachable!(),
        };
        assert!(sales_before.iter().all(|sale| sale.id > 0));
        let expected_document = document.document.clone();
        accepted(&send(&mut session, roundtrip_native_request(request)));
        assert_eq!(session.draft.income[0].id, id);
        let (regular, items) = ledger(&session.draft.income[0].income, false);
        let (amt, amt_items) = ledger(&session.draft.income[0].income, true);
        assert_eq!(items.flows(&regular).capital(), [-100_000, 0]);
        assert_eq!(amt_items.flows(&amt).capital(), [-150_000, 0]);
        let corrected = supported(&session.draft);
        let mut cold = restart(roundtrip_native_draft(session.draft));
        assert_eq!(estimate(&cold.draft), corrected);
        let reopened = open_topic(&mut cold, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = reopened else { unreachable!() };
        assert_eq!(documents.documents[0].id, id);
        assert_eq!(documents.documents[0].document, expected_document);
    }
}

#[test]
fn k1_sale_amt_capital_code_losses_do_not_create_foreign_credit_gross_gains() {
    for long in [false, true] {
        for preferred in [false, true] {
            for mut source in sources(0, 0, 0, 20_000) {
                if preferred && matches!(source, FederalIncome::Scorp { .. }) {
                    continue; // Code Q is a partnership code only.
                }
                code_loss(&mut source, 100);
                match &mut source {
                    FederalIncome::Partnership { data } => {
                        data.income[0].code = match (preferred, long) {
                            (true, false) => PartnerIncomeCode::PreferredStockShort,
                            (true, true) => PartnerIncomeCode::PreferredStockLong,
                            (false, false) => PartnerIncomeCode::NonportfolioShort,
                            (false, true) => PartnerIncomeCode::NonportfolioLong,
                        };
                    }
                    FederalIncome::Scorp { data } => {
                        data.income[0].code = if long {
                            ScorpIncomeCode::NonportfolioLong
                        } else {
                            ScorpIncomeCode::NonportfolioShort
                        };
                    }
                    _ => unreachable!(),
                }
                disjoint_view_counterpart(&mut source, vec![], vec![], [100, 50], [1_000; 2]);
                let (regular, items) = ledger(&source, false);
                let (amt, amt_items) = ledger(&source, true);
                assert_eq!(
                    items.flows(&regular).capital(),
                    if long { [0, -10_000] } else { [-10_000, 0] }
                );
                assert_eq!(
                    amt_items.flows(&amt).capital(),
                    if long { [0, -5_000] } else { [-5_000, 0] }
                );
                let mut d = crate::tests::foreign::foreign::passive_fund_return(60_000);
                add(&mut d, income(source));
                let result = supported(&d);
                assert_eq!(value(&result, "agi"), 69_900);
                // 2026-10-19 correction: i1116 (2025 final), lines 3d/3e,
                // counts gains before losses; i6251 line 8 Step 2 uses the
                // AMT's own income. REG -$100 / AMT -$50 produces no gross
                // gain in either system. Wages $60,000 + fund $10,000 remain.
                // https://www.irs.gov/instructions/i1116
                // https://www.irs.gov/instructions/i6251
                for amt in [false, true] {
                    assert_eq!(
                        form_1116_any(&result, amt, "_gross_income_all_sources"),
                        70_000,
                        "long={long}, preferred={preferred}, amt={amt}"
                    );
                }
            }
        }
    }
}

#[test]
fn k1_sale_amt_k3_capital_code_losses_keep_the_entity_gross_owner() {
    for mut source in sources(0, 0, 0, 20_000) {
        code_loss(&mut source, 100);
        disjoint_view_counterpart(&mut source, vec![], vec![], [100, 50], [1_000; 2]);
        let mut k3 = crate::tests::foreign::foreign::k3_blank();
        k3.us_gross = 100_000;
        match &mut source {
            FederalIncome::Partnership { data } => data.schedule_k3 = Some(k3),
            FederalIncome::Scorp { data } => data.schedule_k3 = Some(k3),
            _ => unreachable!(),
        }
        let mut d = crate::tests::foreign::foreign::passive_fund_return(60_000);
        add(&mut d, income(source));
        let result = supported(&d);
        // The K-3 supplies $1,000 gross in place of the entity's code net.
        // Its independent -$100/-$50 losses supply no additional gross gain.
        assert_eq!(
            [false, true].map(|amt| form_1116_any(&result, amt, "_gross_income_all_sources")),
            [71_000; 2]
        );
    }
}

#[test]
fn k1_sale_amt_k3_own_distribution_gains_stay_outside_entity_gross() {
    for mut source in k1_distribution_counterparts(30_000, [10_000, 20_000]) {
        let k3 = crate::tests::foreign::foreign::k3_blank();
        match &mut source {
            FederalIncome::Partnership { data } => data.schedule_k3 = Some(k3),
            FederalIncome::Scorp { data } => data.schedule_k3 = Some(k3),
            _ => unreachable!(),
        }
        let mut d = crate::tests::foreign::foreign::passive_fund_return(60_000);
        add(&mut d, income(source));
        let result = supported(&d);
        // The entity reports no gross. The owner's $300 cash distribution
        // exceeds independent $200/$100 bases by $100/$200, outside K-3.
        assert_eq!(
            [false, true].map(|amt| form_1116_any(&result, amt, "_gross_income_all_sources")),
            [70_100, 70_200]
        );
    }
}

#[test]
fn k1_sale_amt_own_gross_cents_round_after_replacing_the_regular_subtotal() {
    for source in k1_distribution_counterparts(100, [49, 51]) {
        let mut d = crate::tests::foreign::foreign::passive_fund_return(60_000);
        let id = d.next_id;
        add(&mut d, income(source));
        let entry = d.income.iter().find(|entry| entry.id == id).unwrap();
        let facts = match &entry.income {
            FederalIncome::Partnership { data } => data.totals_facts(entry),
            FederalIncome::Scorp { data } => data.totals_facts(entry),
            _ => unreachable!(),
        };
        assert_eq!(
            [facts.disposal.distribution_gain, facts.amt_disposal.distribution_gain],
            [49, 51]
        );
        let result = supported(&d);
        // $1 cash less 51c/49c basis gives 49c/51c gross. Round the AMT
        // subtotal once: 51c -> $1, rather than $0 + rounded 2c -> $0.
        assert_eq!(
            [false, true].map(|amt| form_1116_any(&result, amt, "_gross_income_all_sources")),
            [70_000, 70_001]
        );
    }
}

#[test]
fn k1_sale_amt_gross_operands_preserve_integral_cancellation() {
    for edge in [i64::MIN, i64::MAX] {
        let mut totals = crate::federal_rules::Totals::default();
        totals.capital_gross = i64::MAX;
        totals.amt_k1_capital_gross_adjustment = -1;
        totals.amt_sale_adjustment = [edge, 10];
        totals.amt_k1_capital_net_adjustment = edge;
        let mut result = FederalResult::empty(0);
        assert_eq!(totals.amt_capital_operands(&mut result), Some([i64::MAX - 1, 10]));
        assert!(result.issues.is_empty());
    }
}

#[test]
fn k1_sale_amt_gross_operands_refuse_unrepresentable_cents_at_foreign_credit() {
    for gross in [false, true] {
        let mut totals = crate::federal_rules::Totals::default();
        totals.capital_gross = if gross { i64::MAX } else { 0 };
        totals.amt_k1_capital_gross_adjustment = i64::from(gross);
        totals.amt_sale_adjustment = if gross { [0, 0] } else { [i64::MAX, 1] };
        let mut result = FederalResult::empty(0);
        assert_eq!(totals.amt_capital_operands(&mut result), None);
        assert_eq!(result.state, ResultState::Incomplete);
        assert_eq!(result.tax, None);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].display.key, crate::k1_activity::ordinary::RANGE_KEY);
        assert_eq!(result.issues[0].context, crate::IssueContext::Section(TaxSection::MoreCredits));
    }
}

#[test]
fn k1_sale_amt_gross_producer_preserves_cross_term_cancellation() {
    let (d, _) = draft(k1_distribution_counterparts(0, [0, 0])[0].clone());
    let entry = &d.income[0];
    let FederalIncome::Partnership { data } = &entry.income else { unreachable!() };
    let mut facts = data.totals_facts(entry);
    // Synthetic admitted owner inputs isolate its arithmetic boundary:
    // the regular +10/-10 cents net to zero, with 10 cents gross.
    facts.flows.preferred = [10, -10];
    let mut totals = crate::federal_rules::Totals::default();
    totals.amt_k1_capital_net_adjustment = i64::MIN;
    totals.amt_k1_capital_gross_adjustment = 10;
    let mut result = FederalResult::empty(d.revision);
    crate::k1_totals::apply(&facts, &mut totals, &mut result);
    assert!(result.issues.is_empty());
    assert_eq!(
        [totals.amt_k1_capital_net_adjustment, totals.amt_k1_capital_gross_adjustment],
        [i64::MIN, 0]
    );
}

#[test]
fn k1_sale_amt_gross_producer_refuses_atomically_at_its_income_source() {
    let (d, _) = draft(k1_distribution_counterparts(100, [49, 51])[0].clone());
    let entry = &d.income[0];
    let FederalIncome::Partnership { data } = &entry.income else { unreachable!() };
    let facts = data.totals_facts(entry);
    for gross in [false, true] {
        let opening = [if gross { 0 } else { i64::MAX }, if gross { i64::MAX } else { 0 }];
        let mut totals = crate::federal_rules::Totals::default();
        totals.amt_k1_capital_net_adjustment = opening[0];
        totals.amt_k1_capital_gross_adjustment = opening[1];
        let mut result = FederalResult::empty(d.revision);
        crate::k1_totals::apply(&facts, &mut totals, &mut result);
        assert_eq!(
            [totals.amt_k1_capital_net_adjustment, totals.amt_k1_capital_gross_adjustment],
            opening
        );
        assert_eq!(totals.capital_gross, 0);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].display.key, crate::k1_activity::ordinary::RANGE_KEY);
        assert_eq!(result.issues[0].context, crate::interview::income_issue_context(entry));
    }
}
