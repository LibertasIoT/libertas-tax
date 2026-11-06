// Section 179 source items through the existing owner limits. Whole-return
// acceptance is exercised separately in k1_sale_final.rs.
use super::*;
use crate::k1_items::BusinessProperty;

fn sale(price: i64, short: bool) -> K1Section179Sale {
    k1_reported_sale(
        "Synthetic packaging line",
        if short { 20260301 } else { 20230301 },
        [price * 100, 50_000 * 100, 8_000 * 100, 20_000 * 100],
    )
}

fn capacity(value: i64) -> ScorpRiskStart {
    ScorpRiskStart::Detailed { capacity: value * 100, profit: 0, loss: 0 }
}

fn partner(ordinary: i64, opening: i64, risk: i64) -> PartnerK1 {
    let mut source = ordinary_partner_source(ordinary, true);
    source.regular.basis.opening = Some(opening * 100);
    source.regular.risk.start = capacity(risk);
    source.alternative = Some(source.regular.clone());
    source
}

fn scorp(ordinary: i64, opening: i64, risk: i64) -> ScorpK1 {
    let mut source = complete_ordinary_scorp(ordinary, true);
    source.loans.clear();
    source.opening = Some(opening * 100);
    source.risk.start = capacity(risk);
    source.risk.prior_loss = 0;
    source.risk.deducted = 0;
    source.qbi_statement = None;
    source.amt = None;
    source
}

#[test]
fn k1_sale_limits_business_property_preserves_gross_sources_and_original_box() {
    let original = K1Section1231 {
        net: -10_000 * 100,
        unrecaptured: 1_000 * 100,
        basis_loss: 2_000 * 100,
        risk_loss: 3_000 * 100,
        passive_loss: 4_000 * 100,
    };
    let unchanged = original.clone();
    let sales = K1Section179Sale::worksheets(&[sale(60_000, false)], 0, 0).unwrap();
    let property = BusinessProperty::of(Some(&original), &sales).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(property.gain), 10_000);
    assert_eq!(crate::kernel::ratio::dollars(property.loss), 10_000);
    assert_eq!(crate::kernel::ratio::dollars(property.net()), 0);
    assert_eq!(crate::kernel::ratio::dollars(property.unrecaptured), 1_000);
    assert_eq!(crate::kernel::ratio::dollars(property.basis_loss), 2_000);
    assert_eq!(crate::kernel::ratio::dollars(property.risk_loss), 3_000);
    assert_eq!(crate::kernel::ratio::dollars(property.passive_loss), 4_000);
    assert_eq!(original, unchanged);
    assert!(BusinessProperty::of(None, &[]).is_none());
}

fn long_sales() -> Vec<K1Section179Sale> {
    // First: gain 38,000 = recapture 28,000 + section 1231 gain 10,000.
    // Second: section 1231 loss 14,000. Ordinary box loss 30,000 below.
    vec![sale(60_000, false), sale(8_000, false)]
}

fn assert_long_ledger(ledger: &crate::k1_loss::LossLedger) {
    use K1LossClass::*;
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_allowed[Ordinary].get()), 30_000);
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_allowed[Section1231].get()), 14_000);
    // $38,000 of activity income funds $44,000 of basis-allowed losses.
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_allowed[Ordinary].get()), 25_909);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_allowed[Section1231].get()), 12_091);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_suspended[Ordinary].get()), 4_091);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_suspended[Section1231].get()), 1_909);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_allowed[OtherLoss].get()), 0);
}

#[test]
fn k1_sale_limits_partner_long_gain_funds_losses_before_shared_risk_limit() {
    let mut source = partner(-30_000, 6_000, 0);
    source.section179_sales = long_sales();
    let unchanged = source.clone();
    let plan = source.regular.plan(&source.items()).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(plan.basis_income), 38_000);
    assert_eq!(crate::kernel::ratio::dollars(plan.risk_income), 38_000);
    assert_eq!(
        crate::kernel::ratio::dollars(plan.input.current[K1LossClass::Section1231].get()),
        14_000
    );
    let pool = source.pool(&source.regular).unwrap();
    assert_long_ledger(&pool.activity.ledger);
    assert_eq!(crate::kernel::ratio::dollars(pool.items.ordinary), 28_000);
    assert_eq!(pool.combined.nonpassive, 0);
    assert_eq!(
        pool.combined.characters.unwrap().nonpassive.map(crate::kernel::ratio::dollars),
        [2_091, -2_091]
    );
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 0);
    assert_eq!(pool.activity.risk_end, Some(0));
    assert_eq!(source, unchanged);
}

#[test]
fn k1_sale_limits_scorp_long_gain_funds_losses_before_shared_risk_limit() {
    let mut source = scorp(-30_000, 6_000, 0);
    source.section179_sales = long_sales();
    let unchanged = source.clone();
    let basis = source.basis_result().unwrap();
    let pool = source.pool_activity_result().unwrap();
    assert_long_ledger(&pool.ledger);
    assert_eq!(crate::kernel::ratio::dollars(source.section1231_gain()), 10_000);
    assert_eq!(crate::kernel::ratio::dollars(pool.items.ordinary), 28_000);
    assert_eq!(pool.combined.nonpassive, 0);
    assert_eq!(
        pool.combined.characters.unwrap().nonpassive.map(crate::kernel::ratio::dollars),
        [2_091, -2_091]
    );
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
    assert_eq!(source, unchanged);
}

fn short_sales() -> Vec<K1Section179Sale> {
    // Ordinary gain 10,000 and loss 14,000, kept as distinct OtherLoss
    // items. Basis 34,000 plus gain 10,000 allows all current losses;
    // at risk only the gain funds the 30,000 + 14,000 loss classes.
    vec![sale(32_000, true), sale(8_000, true)]
}

fn assert_short_ledger(ledger: &crate::k1_loss::LossLedger) {
    use K1LossClass::*;
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_allowed[Ordinary].get()), 30_000);
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_allowed[OtherLoss].get()), 14_000);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_allowed[Ordinary].get()), 6_818);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_allowed[OtherLoss].get()), 3_182);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_suspended[Ordinary].get()), 23_182);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_suspended[OtherLoss].get()), 10_818);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_allowed[Section1231].get()), 0);
}

#[test]
fn k1_sale_limits_short_properties_share_other_loss_and_keep_individual_routes() {
    let mut partner = partner(-30_000, 34_000, 0);
    partner.section179_sales = short_sales();
    let pool = partner.pool(&partner.regular).unwrap();
    assert_short_ledger(&pool.activity.ledger);
    let (flows, _) = partner.outcome(&partner.regular).unwrap();
    // 1,000,000c funds 4,400,000c; the ordinary share retains its cents.
    assert_eq!(flows.ordinary, 681_818);
    assert_eq!(flows.sales, vec![(1, false, 1_000_000), (2, false, -318_182)]);
    assert!(flows.rows.is_empty());
    assert_eq!(pool.combined.nonpassive, 0);

    let mut scorp = scorp(-30_000, 34_000, 0);
    scorp.section179_sales = short_sales();
    let pool = scorp.pool_activity_result().unwrap();
    assert_short_ledger(&pool.ledger);
    let (scorp_flows, _) = scorp.outcome();
    assert_eq!(scorp_flows, flows);
    assert_eq!(pool.combined.nonpassive, 0);
}

#[test]
fn k1_sale_limits_f2c_refuses_ambiguous_carryover_before_any_owner_result() {
    let mut partner = partner(15_000, 0, 0);
    partner.section179_sales = long_sales();
    partner.section179_carryover = 2_000 * 100;
    partner.section179_carryover_sold = 2_000 * 100;
    assert_eq!(
        partner.regular.plan(&partner.items()).unwrap_err().0,
        "ISSUE_K1_SALE_CARRYOVER_SPLIT"
    );
    assert_eq!(partner.pool(&partner.regular).unwrap_err().0, "ISSUE_K1_SALE_CARRYOVER_SPLIT");
    let mut scorp = scorp(15_000, 0, 0);
    scorp.section179_sales = long_sales();
    scorp.section179_carryover = 2_000 * 100;
    scorp.section179_carryover_sold = 2_000 * 100;
    assert_eq!(scorp.basis_result().unwrap_err().0, "ISSUE_K1_SALE_CARRYOVER_SPLIT");
    assert_eq!(scorp.pool_activity_result().unwrap_err().0, "ISSUE_K1_SALE_CARRYOVER_SPLIT");
}

#[test]
fn k1_sale_limits_regular_and_amt_use_independent_sale_depreciation() {
    let mut partner = partner(15_000, 0, 0);
    partner.section179_carryover = 2_000 * 100;
    partner.section179_carryover_sold = 2_000 * 100;
    partner.section179_sales = vec![sale(30_000, false)];
    partner.alternative = Some(partner.regular.clone());
    partner.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
        2_000 * 100;
    let regular = partner.outcome(&partner.regular).unwrap().0;
    let amt = partner.outcome(partner.alternative.as_ref().unwrap()).unwrap().0;
    assert_eq!(regular.sales, vec![(1, true, 6_000 * 100)]);
    assert_eq!(amt.ordinary, 0);
    assert!(amt.sales.is_empty());
    assert_eq!(partner.ledger(&partner.regular).unwrap().risk_allowed, Default::default());

    let mut scorp = scorp(15_000, 0, 0);
    scorp.section179_carryover = 2_000 * 100;
    scorp.section179_carryover_sold = 2_000 * 100;
    scorp.section179_sales = vec![sale(30_000, false)];
    scorp.amt = Some(amt_records(&scorp));
    for sale in &mut scorp.section179_sales {
        sale.alternative = crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
    }
    scorp.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
        2_000 * 100;
    assert_eq!(scorp.outcome().0, regular);
    assert_eq!(scorp.amt.as_ref().unwrap().source(&scorp).unwrap().outcome().0, amt);
}

#[test]
fn k1_sale_limits_f2b_loss_creates_business_property_without_an_entered_box() {
    let mut partner = partner(15_000, 0, 0);
    partner.section179_carryover = 2_000 * 100;
    partner.section179_carryover_sold = 2_000 * 100;
    partner.section179_sales = vec![sale(20_000, false)];
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.risk_allowed[K1LossClass::Section1231].get()
        ),
        4_000
    );
    assert_eq!(
        pool.combined.characters.unwrap().nonpassive.map(crate::kernel::ratio::dollars),
        [15_000, -4_000]
    );
    assert!(partner.regular.section1231.is_none());
    let mut scorp = scorp(15_000, 0, 0);
    scorp.section179_carryover = 2_000 * 100;
    scorp.section179_carryover_sold = 2_000 * 100;
    scorp.section179_sales = partner.section179_sales;
    let pool = scorp.pool_activity_result().unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(pool.ledger.risk_allowed[K1LossClass::Section1231].get()),
        4_000
    );
    assert_eq!(
        pool.combined.characters.unwrap().nonpassive.map(crate::kernel::ratio::dollars),
        [15_000, -4_000]
    );
    assert!(scorp.section1231.is_none());
}

#[test]
fn k1_sale_limits_code_r_and_each_short_sale_keep_distinct_allowed_amounts() {
    let mut partner = partner(0, 0, 0);
    partner.section179_sales = short_sales();
    partner.income = vec![PartnerIncomeTotal {
        code: PartnerIncomeCode::SpeciallyAllocatedOrdinary,
        amount: -6_000 * 100,
    }];
    let (flows, _) = partner.outcome(&partner.regular).unwrap();
    // The $10,000 gain funds half of the $20,000 OtherLoss class.
    assert_eq!(flows.ordinary, 0);
    assert_eq!(
        flows.rows,
        vec![("11R", crate::k1_items::K1ItemKind::OrdinaryActivity, -3_000 * 100)]
    );
    assert_eq!(flows.sales, vec![(1, false, 10_000 * 100), (2, false, -7_000 * 100)]);
    assert_eq!(partner.k1_items().box_one(), 0);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_suspended[K1LossClass::OtherLoss].get()
        ),
        10_000
    );
    assert_eq!(pool.combined.nonpassive, 0);
}

#[test]
fn k1_sale_limits_passive_property_without_box_uses_one_shared_activity_and_class() {
    let mut partner = partner(-30_000, 100_000, 100_000);
    let Some(PartnerParticipation::General { facts }) = &mut partner.participation else {
        panic!()
    };
    facts.minutes = 0;
    partner.section179_sales = long_sales();
    partner.alternative = Some(partner.regular.clone());
    let mut scorp = scorp(-30_000, 100_000, 100_000);
    scorp.participation.as_mut().unwrap().minutes = 0;
    scorp.section179_sales = long_sales();
    for source in [
        FederalIncome::Partnership { data: partner },
        FederalIncome::Scorp { data: crate::tests::support::builders::scorp_source(scorp) },
    ] {
        let mut draft = complete(FederalStatus::Single);
        draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        let id = draft.next_id;
        add(&mut draft, income(source));
        assert!(draft.valid_stored());
        let mut result = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&draft, 200_000, &mut result, false);
        assert!(pool.inputs_complete, "{:?}", result.issues);
        let properties: Vec<_> = pool.k1_section1231.iter().filter(|p| p.id == id).collect();
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].net, 0);
        assert_eq!(properties[0].suspended, 4_000);
        let ledger = &pool.k1_ledgers.iter().find(|(key, _)| *key == id).unwrap().1;
        assert_eq!(
            crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Ordinary].get()),
            2_000
        );
        assert_eq!(
            crate::kernel::ratio::dollars(ledger.passive_suspended[K1LossClass::Section1231].get()),
            4_000
        );
        assert_eq!(pool.income, -28_000);
        let worksheet = crate::section1231::worksheet(&draft, &pool, false).unwrap().unwrap();
        assert_eq!(worksheet.net, 0);
        assert_eq!(worksheet.ordinary, 0);
        assert_eq!(worksheet.long_term, 0);
    }
}

#[test]
fn k1_sale_limits_sale_gain_enters_the_existing_return_wide_lookback_once() {
    let mut partner = partner(0, 0, 0);
    partner.section179_sales = vec![sale(60_000, false)];
    partner.alternative = Some(partner.regular.clone());
    let mut scorp = scorp(0, 0, 0);
    scorp.section179_sales = vec![sale(60_000, false)];
    for source in [
        FederalIncome::Partnership { data: partner },
        FederalIncome::Scorp { data: crate::tests::support::builders::scorp_source(scorp) },
    ] {
        let mut draft = complete(FederalStatus::Single);
        draft.screening.as_mut().unwrap().section1231 =
            Some(section1231_history(&[(2024, 5_000, 5_000)], &[]));
        let id = draft.next_id;
        add(&mut draft, income(source));
        let mut result = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&draft, 200_000, &mut result, false);
        assert!(pool.inputs_complete, "{:?}", result.issues);
        assert_eq!(pool.k1_section1231.iter().filter(|p| p.id == id).count(), 1);
        let worksheet = crate::section1231::worksheet(&draft, &pool, false).unwrap().unwrap();
        assert_eq!(worksheet.net, 10_000);
        assert_eq!(worksheet.ordinary, 5_000);
        assert_eq!(worksheet.long_term, 5_000);
        assert_eq!(pool.income, 0);
    }
}
