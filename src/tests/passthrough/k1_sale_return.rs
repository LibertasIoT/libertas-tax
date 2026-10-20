use crate::tests::support::assert::k1_consolidated::k1_form4797_sum as form_sum;
// Reporting and Form 4562 owner oracles. Whole-return acceptance is exercised
// separately in k1_sale_final.rs.
use super::*;
use alloc::format;

fn sale(price: i64, short: bool) -> K1Section179Sale {
    k1_reported_sale(
        "Synthetic packaging line",
        if short { 20260301 } else { 20230301 },
        [price * 100, 50_000 * 100, 8_000 * 100, 20_000 * 100],
    )
}

fn risk(value: i64) -> ScorpRiskStart {
    ScorpRiskStart::Detailed { capacity: value * 100, profit: 0, loss: 0 }
}

pub(in crate::tests) fn sources(
    ordinary: i64,
    carry: i64,
    sold: i64,
    price: i64,
) -> [FederalIncome; 2] {
    let mut partner = ordinary_partner_source(ordinary, true);
    partner.regular.basis.opening = 0;
    partner.regular.risk.start = risk(0);
    partner.section179_carryover = carry * 100;
    partner.section179_carryover_sold = sold * 100;
    partner.section179_sales = vec![sale(price, false)];
    partner.alternative = Some(partner.regular.clone());
    let mut scorp = complete_ordinary_scorp(ordinary, true);
    scorp.loans.clear();
    scorp.opening = 0;
    scorp.risk.start = risk(0);
    scorp.risk.prior_loss = 0;
    scorp.risk.deducted = 0;
    scorp.section179_carryover = carry * 100;
    scorp.section179_carryover_sold = sold * 100;
    scorp.section179_sales = vec![sale(price, false)];
    scorp.qbi_statement = None;
    mirror_scorp(&mut scorp);
    [FederalIncome::Partnership { data: partner }, FederalIncome::Scorp { data: scorp }]
}

fn mirror_scorp(source: &mut ScorpK1) {
    source.amt = Some(amt_records(source));
    for sale in &mut source.section179_sales {
        sale.alternative = crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
    }
}

pub(in crate::tests) fn draft(source: FederalIncome) -> (FederalDraft, i64) {
    let mut draft = complete(FederalStatus::Single);
    draft.income.clear();
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let id = draft.next_id;
    add(&mut draft, income(source));
    assert!(draft.valid_stored());
    (draft, id)
}

fn reporting(draft: &FederalDraft) -> (FederalResult, i64) {
    let mut result = FederalResult::empty(draft.revision);
    let pool = crate::extended_rules::rentals_for(draft, 200_000, &mut result, false);
    assert!(pool.inputs_complete, "{:?}", result.issues);
    let ordinary = crate::k1_sales::report(draft, &pool, false, &mut result);
    crate::section1231::report(draft, &pool, false, &mut result);
    let amt = crate::extended_rules::rentals_for(draft, 200_000, &mut result, true);
    assert!(amt.inputs_complete, "{:?}", result.issues);
    crate::k1_sales::report(draft, &amt, true, &mut result);
    crate::section1231::report(draft, &amt, true, &mut result);
    crate::k1_sales::report_recapture(&mut result);
    crate::section179::report(draft, &mut result);
    assert!(result.issues.is_empty(), "{:?}", result.issues);
    (result, ordinary)
}

pub(in crate::tests) fn passive_short(mut source: FederalIncome) -> FederalIncome {
    match &mut source {
        FederalIncome::Partnership { data } => {
            let PartnerParticipation::General { facts } = &mut data.participation else {
                unreachable!()
            };
            facts.minutes = 0;
            data.regular.basis.opening = 100_000 * 100;
            data.regular.risk.start = risk(100_000);
            data.section179_sales[0].acquired = 20260301;
            data.alternative = Some(data.regular.clone());
        }
        FederalIncome::Scorp { data } => {
            data.participation.as_mut().unwrap().minutes = 0;
            data.opening = 100_000 * 100;
            data.risk.start = risk(100_000);
            data.section179_sales[0].acquired = 20260301;
            mirror_scorp(data);
        }
        _ => unreachable!(),
    }
    source
}

#[test]
fn k1_sale_return_f2_removes_sold_carry_without_a_second_basis_deduction() {
    for source in sources(15_000, 2_000, 2_000, 30_000) {
        let (draft, id) = draft(source);
        let deduction = crate::section179::calculate(&draft);
        assert_eq!(deduction.available, 0);
        let owner = &deduction.pass_throughs[0];
        assert_eq!((owner.carryover, owner.sold_carryover, owner.income), (0, 200_000, 2_100_000));
        let (basis, prefix) = match &draft.income.iter().find(|e| e.id == id).unwrap().income {
            FederalIncome::Partnership { data } => {
                (data.pool(&data.regular).unwrap().activity.basis.ending, "partnership")
            }
            FederalIncome::Scorp { data } => (data.basis_result().unwrap().stock, "scorp"),
            _ => unreachable!(),
        };
        assert_eq!(crate::kernel::ratio::dollars(basis.get()), 21_000);
        let mut result = estimate(&complete(FederalStatus::Single));
        crate::section179::report(&draft, &mut result);
        assert!(result.issues.is_empty(), "{:?}", result.issues);
        assert_eq!(value(&result, "section179_prior_carryover_total"), 0);
        assert_eq!(
            printed_line(&result, "section179_prior_carryover_total"),
            (TaxForm::Form4562, "10".into())
        );
        assert_eq!(value(&result, &format!("{prefix}_{id}_section179_prior_carryover")), 0);
    }
}

#[test]
fn k1_sale_return_partial_sold_carry_retains_only_the_remaining_deduction() {
    for source in sources(15_000, 2_000, 500, 30_000) {
        let (draft, id) = draft(source);
        let deduction = crate::section179::calculate(&draft);
        assert_eq!(deduction.available, 1_500 * 100);
        assert_eq!(deduction.income, 22_500 * 100);
        assert_eq!(deduction.allowed, 1_500 * 100);
        assert_eq!(deduction.pass_through[&id], 1_500 * 100);
        let basis = match &draft.income.iter().find(|e| e.id == id).unwrap().income {
            FederalIncome::Partnership { data } => {
                data.pool(&data.regular).unwrap().activity.basis.ending
            }
            FederalIncome::Scorp { data } => data.basis_result().unwrap().stock,
            _ => unreachable!(),
        };
        assert_eq!(crate::kernel::ratio::dollars(basis.get()), 22_500);
        let mut result = estimate(&complete(FederalStatus::Single));
        crate::section179::report(&draft, &mut result);
        assert!(result.issues.is_empty(), "{:?}", result.issues);
        assert_eq!(value(&result, "section179_prior_carryover_total"), 1_500);
    }
}

#[test]
fn k1_sale_return_active_sale_income_funds_carry_and_includes_long_term_property_gain() {
    for (price, expected) in [(30_000, 6_000), (60_000, 36_000)] {
        for source in sources(0, 4_000, 2_000, price) {
            let (draft, id) = draft(source);
            let deduction = crate::section179::calculate(&draft);
            assert_eq!(deduction.income, expected * 100);
            assert_eq!(deduction.available, 2_000 * 100);
            assert_eq!(deduction.pass_through[&id], 2_000 * 100);
        }
    }
}

#[test]
fn k1_sale_return_active_property_loss_reduces_section179_business_income() {
    for mut source in sources(15_000, 2_000, 2_000, 20_000) {
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.section179 = 12_000 * 100;
                data.regular.basis.opening = 100_000 * 100;
                data.regular.risk.start = risk(100_000);
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                data.section179 = 12_000 * 100;
                data.opening = 100_000 * 100;
                data.risk.start = risk(100_000);
                mirror_scorp(data);
            }
            _ => unreachable!(),
        }
        let (draft, id) = draft(source);
        let deduction = crate::section179::calculate(&draft);
        assert_eq!(deduction.available, 12_000 * 100);
        assert_eq!(deduction.income, 11_000 * 100);
        assert_eq!(deduction.pass_through[&id], 11_000 * 100);
        assert_eq!(deduction.available - deduction.allowed, 1_000 * 100);
    }
}

#[test]
fn k1_sale_return_passive_cut_preserves_gains_and_reconciles_all_ordinary_sources() {
    use crate::k1_items::{K1ItemFlows, K1ItemKind};
    let flows = K1ItemFlows {
        ordinary: -10_000 * 100,
        rows: vec![("11R", K1ItemKind::OrdinaryActivity, -6_000 * 100)],
        sales: vec![(1, false, 10_000 * 100), (2, false, -14_000 * 100)],
        ..Default::default()
    };
    let allowed = flows.after_passive(10_000 * 100).unwrap();
    assert_eq!(allowed.ordinary, 0);
    assert_eq!(allowed.rows[0].2, -3_000 * 100);
    assert_eq!(allowed.sales, vec![(1, false, 10_000 * 100), (2, false, -7_000 * 100)]);
    assert_eq!(
        allowed.rows.iter().map(|r| r.2).sum::<i64>()
            + allowed.sales.iter().map(|r| r.2).sum::<i64>(),
        allowed.ordinary
    );
    assert!(flows.after_passive(-1).is_none());
    assert!(flows.after_passive(10_001 * 100).is_none());
    assert!(flows.after_passive(20_001 * 100).is_none());
    assert_eq!(flows.ordinary, -1_000_000);
}

#[test]
fn k1_sale_return_f2_form_source_and_recapture_transfers_appear_once() {
    for source in sources(15_000, 2_000, 2_000, 30_000) {
        let (draft, id) = draft(source);
        let final_return = estimate(&draft);
        assert_eq!(
            final_return.state,
            ResultState::EstimatedForSupportedScope,
            "{:?}",
            final_return.issues
        );
        assert_eq!(value(&final_return, "agi"), 21_000);
        let (result, ordinary) = reporting(&draft);
        assert_eq!(ordinary, 6_000);
        let prefix = format!("k1_{id}_section179_sale_1");
        for (suffix, line, amount) in [
            ("price", "22", 30_000),
            ("cost", "23", 50_000),
            ("depreciation", "24", 26_000),
            ("basis", "25", 24_000),
            ("gain", "26", 6_000),
            ("depreciation1245", "27a", 26_000),
            ("recapture", "27b", 6_000),
            ("partthree", "34", 0),
            ("section1231", "6", 0),
        ] {
            let row = format!("{prefix}_{suffix}");
            assert_eq!(value(&result, &row), amount);
            assert_eq!(printed_line(&result, &row), (TaxForm::Form4797, line.into()));
        }
        assert_eq!(form_sum(&result, "27b"), 6_000);
        assert_eq!(form_sum(&result, "33"), 6_000);
        assert_eq!(form_sum(&result, "14"), 6_000);
        assert_eq!(value(&result, "section179_prior_carryover_total"), 0);
        // This source-owner helper does not invoke the final return footer.
        assert!(!result.lines.iter().any(|row| row.id == "form4797_line19"));
    }
}

#[test]
fn k1_sale_return_independent_amt_sale_rows_do_not_duplicate_printed_form() {
    for mut source in sources(15_000, 2_000, 2_000, 30_000) {
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    6_000 * 100
            }
            FederalIncome::Scorp { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    6_000 * 100
            }
            _ => unreachable!(),
        }
        let (draft, id) = draft(source);
        let (result, _) = reporting(&draft);
        assert_eq!(value(&result, &format!("k1_{id}_section179_sale_1_ordinary")), 6_000);
        assert_eq!(value(&result, &format!("amt_k1_{id}_section179_sale_1_ordinary")), 4_000);
        assert!(
            result
                .lines
                .iter()
                .filter(|row| row.id.starts_with(&format!("amt_k1_{id}_section179_sale_")))
                .all(|row| row.form_line.is_none())
        );
        assert_eq!(form_sum(&result, "27b"), 6_000);
        assert_eq!(form_sum(&result, "14"), 6_000);
    }
}

#[test]
fn k1_sale_return_property_gain_separates_original_box_from_sale_line_six() {
    for mut source in sources(0, 2_000, 2_000, 60_000) {
        let property = Some(k1_section1231(5_000, 0));
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.regular.section1231 = property;
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                data.section1231 = property;
                mirror_scorp(data);
            }
            _ => unreachable!(),
        }
        let (draft, id) = draft(source);
        let (result, _) = reporting(&draft);
        let name = match draft.income.iter().find(|e| e.id == id).unwrap().income {
            FederalIncome::Partnership { .. } => "partnership",
            _ => "scorp",
        };
        assert_eq!(value(&result, &format!("{name}_{id}_section1231")), 5_000);
        assert_eq!(form_sum(&result, "6"), 10_000);
        assert_eq!(form_sum(&result, "2"), 5_000);
        assert_eq!(value(&result, "form4797_line8"), 15_000);
    }
}

#[test]
fn k1_sale_return_short_loss_uses_its_own_passive_activity_allocation() {
    for source in sources(0, 0, 0, 20_000) {
        let (draft, id) = draft(passive_short(source));
        let mut result = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&draft, 200_000, &mut result, false);
        assert!(pool.inputs_complete, "{:?}", result.issues);
        assert_eq!(
            pool.k1_cuts
                .ordinary_by_id
                .iter()
                .map(|(id, cents)| (*id, crate::kernel::ratio::dollars(cents.get())))
                .collect::<Vec<_>>(),
            vec![(id, 2_000)]
        );
        assert_eq!(crate::k1_sales::report(&draft, &pool, false, &mut result), 0);
        let row = format!("k1_{id}_section179_sale_1_ordinary");
        assert_eq!(value(&result, &row), 0);
        assert_eq!(printed_line(&result, &row), (TaxForm::Form4797, "11".into()));
    }
}

#[test]
fn k1_sale_return_shared_passive_pool_keeps_each_property_with_its_owner() {
    for (loss, gain) in sources(0, 0, 0, 20_000).into_iter().zip(sources(0, 0, 0, 23_000)) {
        let (mut draft, loss_id) = draft(passive_short(loss));
        let gain_id = draft.next_id;
        add(&mut draft, income(passive_short(gain)));
        assert!(draft.valid_stored());
        let mut result = FederalResult::empty(draft.revision);
        let pool = crate::extended_rules::rentals_for(&draft, 200_000, &mut result, false);
        assert!(pool.inputs_complete, "{:?}", result.issues);
        assert_eq!(crate::kernel::ratio::dollars(pool.k1_cuts.ordinary.get()), 1_000);
        assert_eq!(
            pool.k1_cuts
                .ordinary_by_id
                .iter()
                .map(|(id, cents)| (*id, crate::kernel::ratio::dollars(cents.get())))
                .collect::<Vec<_>>(),
            vec![(loss_id, 1_000), (gain_id, 0)]
        );
        assert_eq!(crate::k1_sales::report(&draft, &pool, false, &mut result), 0);
        assert_eq!(value(&result, &format!("k1_{loss_id}_section179_sale_1_ordinary")), -1_000);
        assert_eq!(value(&result, &format!("k1_{gain_id}_section179_sale_1_ordinary")), 1_000);
        assert_eq!(form_sum(&result, "11"), 0);
    }
}
