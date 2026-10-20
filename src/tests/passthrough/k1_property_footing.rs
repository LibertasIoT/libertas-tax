use crate::tests::support::assert::k1_consolidated::k1_form4797_sum as form;
// Independent source-row and aggregate footings through the actual loss pool
// and reporting owners, with final-return and restart parity.
use super::*;
use crate::tests::passthrough::k1_sale_return::{draft, sources};
use alloc::format;

fn sale(gain: i64) -> K1Section179Sale {
    k1_reported_sale(
        "Synthetic business property",
        20230301,
        [(20_000 + gain) * 100, 2_000_000, 0, 0],
    )
}

fn property(net: i64, basis: i64, risk: i64, passive: i64) -> K1Section1231 {
    K1Section1231 {
        net: net * 100,
        unrecaptured: 0,
        basis_loss: basis * 100,
        risk_loss: risk * 100,
        passive_loss: passive * 100,
    }
}

fn mirror(source: &mut FederalIncome) {
    match source {
        FederalIncome::Partnership { data } => data.alternative = Some(data.regular.clone()),
        FederalIncome::Scorp { data } => {
            let amt = amt_records(data);
            for sale in &mut data.section179_sales {
                sale.alternative =
                    crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
            }
            data.amt = Some(amt);
        }
        _ => unreachable!(),
    }
}

fn configure(
    source: &mut FederalIncome,
    rows: Vec<K1Section179Sale>,
    original: Option<K1Section1231>,
    basis: i64,
    risk: i64,
    passive: bool,
) {
    let start = ScorpRiskStart::Detailed { capacity: risk * 100, profit: 0, loss: 0 };
    match source {
        FederalIncome::Partnership { data } => {
            data.section179_sales = rows;
            data.regular.section1231 = original;
            data.regular.basis.opening = basis * 100;
            data.regular.risk.start = start;
            let PartnerParticipation::General { facts } = &mut data.participation else {
                unreachable!()
            };
            if passive {
                facts.minutes = 0;
            }
        }
        FederalIncome::Scorp { data } => {
            data.section179_sales = rows;
            data.section1231 = original;
            data.opening = basis * 100;
            data.risk.start = start;
            if passive {
                data.participation.as_mut().unwrap().minutes = 0;
            }
        }
        _ => unreachable!(),
    }
    mirror(source);
}

fn report(d: &FederalDraft, amt: bool) -> FederalResult {
    let before = d.clone();
    let mut result = FederalResult::empty(d.revision);
    let pool = crate::extended_rules::rentals_for(d, 200_000, &mut result, amt);
    assert!(pool.inputs_complete, "{:?}", result.issues);
    crate::k1_sales::report(d, &pool, amt, &mut result);
    crate::section1231::report(d, &pool, amt, &mut result);
    assert!(result.issues.is_empty(), "{:?}", result.issues);
    assert_eq!(*d, before);
    result
}

fn check(d: &FederalDraft, id: i64, amt: bool, sales: &[i64], residual: i64, net: i64) {
    let result = report(d, amt);
    let prefix = if amt { "amt_" } else { "" };
    for (index, expected) in sales.iter().enumerate() {
        let key = format!("{prefix}k1_{id}_section179_sale_{}_section1231", index + 1);
        assert_eq!(value(&result, &key), *expected, "{key}");
        if amt {
            assert!(result.lines.iter().find(|v| v.id == key).unwrap().form_line.is_none());
        } else {
            let row = result.lines.iter().find(|v| v.id == key).unwrap();
            assert!(row.form_line.as_ref().is_some_and(
                |f| f.form == TaxForm::Form4797 && matches!(f.line.as_str(), "2" | "6")
            ));
        }
    }
    let kind = match &d.income.iter().find(|v| v.id == id).unwrap().income {
        FederalIncome::Partnership { .. } => "partnership",
        FederalIncome::Scorp { .. } => "scorp",
        _ => unreachable!(),
    };
    assert_eq!(value(&result, &format!("{prefix}{kind}_{id}_section1231")), residual);
    assert_eq!(sales.iter().sum::<i64>() + residual, net);
    if amt {
        assert!(result.lines.iter().all(|row| row.form_line.is_none()));
    } else {
        assert_eq!(
            ["2", "3", "4", "5", "6", "7"].iter().map(|line| form(&result, line)).sum::<i64>(),
            net
        );
        assert_eq!(form(&result, "8"), net);
        assert_eq!(form(&result, "12"), net.min(0));
    }
    let final_return = estimate(d);
    assert_eq!(
        final_return.state,
        ResultState::EstimatedForSupportedScope,
        "{:?}",
        final_return.issues
    );
    assert_eq!(estimate(&restart(d.clone()).draft), final_return);
}

#[test]
fn k1_property_footing_f2b_allowed_loss_has_one_source_row() {
    for source in sources(15_000, 2_000, 2_000, 20_000) {
        let (d, id) = draft(source);
        check(&d, id, false, &[-4_000], 0, -4_000);
        check(&d, id, true, &[-4_000], 0, -4_000);
    }
}

#[test]
fn k1_property_footing_two_properties_share_basis_without_netting() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(&mut source, vec![sale(-2_000), sale(-6_000)], None, 3_000, 10_000, false);
        let (d, id) = draft(source);
        check(&d, id, false, &[-750, -2_250], 0, -3_000);
    }
}

#[test]
fn k1_property_footing_original_and_prior_sources_retain_their_residual() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(
            &mut source,
            vec![sale(-2_000)],
            Some(property(-4_000, 6_000, 4_000, 2_000)),
            6_000,
            5_000,
            false,
        );
        let (d, id) = draft(source);
        // Basis: [4k,2k,6k] -> [2k,1k,3k]. Risk adds prior4k;
        // [2k,1k,3k,4k] -> [1k,500,1.5k,2k]. Prior PAL2k stays held.
        check(&d, id, false, &[-500], -4_500, -5_000);
        check(&d, id, true, &[-500], -4_500, -5_000);
    }
}

#[test]
fn k1_property_footing_passive_cut_preserves_gross_gain_and_old_loss() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(
            &mut source,
            vec![sale(10_000), sale(-14_000)],
            Some(property(0, 0, 0, 6_000)),
            20_000,
            20_000,
            true,
        );
        let (d, id) = draft(source);
        // 20k total losses, 10k income: suspend7k current and3k prior.
        check(&d, id, false, &[10_000, -7_000], -3_000, 0);
    }
}

#[test]
fn k1_property_footing_former_passive_never_suspends_current_active_loss() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(
            &mut source,
            vec![sale(-4_000)],
            Some(property(0, 0, 0, 6_000)),
            10_000,
            10_000,
            false,
        );
        let (d, id) = draft(source);
        check(&d, id, false, &[-4_000], 0, -4_000);
    }
}

#[test]
fn k1_property_footing_independent_amt_loss_does_not_duplicate_form() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(&mut source, vec![sale(-4_000)], None, 3_000, 10_000, false);
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    200_000
            }
            FederalIncome::Scorp { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    200_000
            }
            _ => unreachable!(),
        }
        let (d, id) = draft(source);
        check(&d, id, false, &[-3_000], 0, -3_000);
        check(&d, id, true, &[-2_000], 0, -2_000);
    }
}

#[test]
fn k1_property_footing_basis_rounding_follows_source_order() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(&mut source, vec![sale(-1), sale(-2)], None, 2, 2, false);
        let (d, id) = draft(source);
        check(&d, id, false, &[-1, -1], 0, -2);
    }
}

#[test]
fn k1_property_footing_passive_rounds_unallowed_instead_of_allowed_loss() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(&mut source, vec![sale(2), sale(-3)], Some(property(0, 0, 0, 1)), 10, 10, true);
        let (d, id) = draft(source);
        // 2026-10-19: held200c splits150c current/50c prior. Actual rows
        // +200c/-150c/-50c report +2/-2/-1; Form4797 sums those rows.
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 200_000, &mut result, false);
        let property = pool.k1_section1231.iter().find(|value| value.id == id).unwrap();
        assert_eq!(property.section179_sales_cents, vec![200, -150]);
        assert_eq!((property.net_cents, property.net), (0, -1));
        check(&d, id, false, &[2, -2], -1, -1);
    }
}

#[test]
fn k1_property_footing_active_rows_exclude_the_same_reported_niit_amount() {
    for mut source in sources(0, 0, 0, 20_000) {
        // The sale worksheet owns reported +2/-3 dollars. Its 200c gain
        // funds 400c current property losses at basis: original100c/sale300c
        // become allowed50c/150c. Risk admits that same200c in full.
        configure(&mut source, vec![sale(2), sale(-3)], Some(property(-1, 0, 0, 0)), 0, 0, false);
        let (d, id) = draft(source);
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 200_000, &mut result, false);
        assert!(pool.inputs_complete, "{:?}", result.issues);
        let ledger = &pool.k1_ledgers.iter().find(|(key, _)| *key == id).unwrap().1;
        assert_eq!(ledger.basis_allowed[K1LossClass::Section1231].get(), 200);
        assert_eq!(ledger.basis_suspended[K1LossClass::Section1231].get(), 200);
        assert_eq!(ledger.risk_allowed[K1LossClass::Section1231].get(), 200);
        let property = pool.k1_section1231.iter().find(|value| value.id == id).unwrap();
        assert_eq!(property.section179_sales_cents, vec![200, -150]);
        assert_eq!((property.net_cents, property.nonpassive_cents, property.net), (0, 0, -1));
        let character = crate::section1231::worksheet(&d, &pool, false).unwrap().unwrap();
        assert_eq!((character.net, character.ordinary), (-1, -1));
        let niit = crate::section1231::property_niit(&d, &pool, &character, 0, 0, 0, 0).unwrap();
        // Actual current participation and no prior passive/rental contribution
        // prove all three rows excluded: reported -1 minus that same -1 is zero.
        assert_eq!((niit.adjustment, niit.next_capital_adjustment), (0, 0));
        check(&d, id, false, &[2, -2], -1, -1);
    }
}

#[test]
fn k1_property_footing_original_cents_are_preserved_and_rounded_once() {
    for mut source in sources(0, 0, 0, 20_000) {
        let mut original = property(0, 0, 0, 0);
        original.net = -149;
        configure(&mut source, vec![sale(-2)], Some(original), 2, 2, false);
        let (d, id) = draft(source);
        // Whole-dollar weights [1,2] share basis2 as [1,1].
        check(&d, id, false, &[-1], -1, -2);
        let cents = match &d.income[0].income {
            FederalIncome::Partnership { data } => data.regular.section1231.as_ref().unwrap().net,
            FederalIncome::Scorp { data } => data.section1231.as_ref().unwrap().net,
            _ => unreachable!(),
        };
        assert_eq!(cents, -149);
    }
}

#[test]
fn k1_property_footing_rental_component_stays_in_original_k1_row() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(
            &mut source,
            vec![sale(-2_000)],
            Some(property(5_000, 0, 0, 0)),
            10_000,
            10_000,
            false,
        );
        let rental = K1RentalProperty { net: 300_000, ..Default::default() };
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.regular.rentals.real_estate_property = rental
            }
            FederalIncome::Scorp { data } => data.rentals.real_estate_property = rental,
            _ => unreachable!(),
        }
        mirror(&mut source);
        let (d, id) = draft(source);
        check(&d, id, false, &[-2_000], 8_000, 6_000);
    }
}

#[test]
fn k1_property_footing_bad_pool_fails_with_named_issue() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(&mut source, vec![sale(-4_000)], None, 10_000, 10_000, false);
        let (d, _) = draft(source);
        let mut result = FederalResult::empty(d.revision);
        let mut pool = crate::extended_rules::rentals_for(&d, 200_000, &mut result, false);
        assert!(pool.inputs_complete);
        pool.k1_section1231[0].net += 1;
        let prior_lines = result.lines.clone();
        crate::k1_sales::report(&d, &pool, false, &mut result);
        assert!(result.issues.iter().any(|v| v.display.key == "ISSUE_K1_SALE_PASSIVE_ALLOCATION"));
        assert_eq!(result.lines, prior_lines);
    }
}

#[test]
fn k1_property_footing_whole_interest_sale_releases_only_its_own_prior_loss() {
    for mut source in sources(0, 0, 0, 20_000) {
        configure(
            &mut source,
            vec![sale(-4_000)],
            Some(property(0, 0, 0, 6_000)),
            10_000,
            10_000,
            true,
        );
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.final_k1 = true;
                data.disposition = K1Disposition::Sold { proceeds: 1_000_000 };
                data.interest_term = Some(GainTerm::Long);
            }
            FederalIncome::Scorp { data } => {
                data.final_k1 = true;
                data.disposition = K1Disposition::Sold { proceeds: 1_000_000 };
                data.stock_term = Some(GainTerm::Long);
            }
            _ => unreachable!(),
        }
        mirror(&mut source);
        let (d, id) = draft(source);
        check(&d, id, false, &[-4_000], -6_000, -10_000);
    }
}

#[test]
fn k1_property_footing_shared_passive_income_preserves_activity_ids() {
    for (mut loss, mut gain) in sources(0, 0, 0, 20_000).into_iter().zip(sources(0, 0, 0, 20_000)) {
        configure(&mut loss, vec![sale(-2_000)], None, 10_000, 10_000, true);
        configure(&mut gain, vec![sale(1_000)], None, 10_000, 10_000, true);
        let (mut d, loss_id) = draft(loss);
        let gain_id = d.next_id;
        add(&mut d, income(gain));
        let result = report(&d, false);
        assert_eq!(value(&result, &format!("k1_{loss_id}_section179_sale_1_section1231")), -1_000);
        assert_eq!(value(&result, &format!("k1_{gain_id}_section179_sale_1_section1231")), 1_000);
        assert_eq!(form(&result, "2"), -1_000);
        assert_eq!(form(&result, "6"), 1_000);
        assert_eq!(form(&result, "8"), 0);
    }
}

#[test]
fn k1_property_footing_mixed_sales_foot_both_parts_and_recapture_once() {
    for mut source in sources(0, 0, 0, 30_000) {
        let mut recapture = sale(0);
        recapture.regular.as_mut().unwrap().price = 3_000_000;
        recapture.regular.as_mut().unwrap().cost = 5_000_000;
        recapture.regular.as_mut().unwrap().depreciation = 800_000;
        recapture.regular.as_mut().unwrap().section179 = 2_000_000;
        let mut short = sale(1_000);
        short.acquired = 20260101;
        configure(&mut source, vec![sale(-4_000), short, recapture], None, 100_000, 100_000, false);
        // With no sold carry: adjusted basis22k, recapture8k.
        let (d, id) = draft(source);
        let mut result = report(&d, false);
        crate::k1_sales::report_recapture(&mut result);
        assert_eq!(value(&result, &format!("k1_{id}_section179_sale_1_section1231")), -4_000);
        assert_eq!(form(&result, "2"), -4_000);
        assert_eq!(form(&result, "8"), -4_000);
        assert_eq!(form(&result, "12"), -4_000);
        assert_eq!(form(&result, "11"), 1_000);
        assert_eq!(form(&result, "27b"), 8_000);
        assert_eq!(form(&result, "33"), 8_000);
        assert_eq!(form(&result, "14"), 8_000);
        let ordinary = (11..=18).map(|line| form(&result, &format!("{line}"))).sum::<i64>();
        assert_eq!(ordinary, 5_000);
        crate::k1_sales::report_ordinary_total(&mut result, ordinary);
        assert_eq!(form(&result, "19"), 5_000);
        assert_eq!(form(&result, "20b"), 5_000);
    }
}

#[test]
fn k1_property_footing_zero_and_short_loss_keep_their_form_parts() {
    for mut source in sources(0, 0, 0, 20_000) {
        let mut short = sale(-1_000);
        short.acquired = 20260101;
        configure(&mut source, vec![sale(0), short], None, 10_000, 10_000, false);
        let (d, id) = draft(source);
        let result = report(&d, false);
        let zero = format!("k1_{id}_section179_sale_1_section1231");
        assert_eq!(value(&result, &zero), 0);
        assert_eq!(printed_line(&result, &zero), (TaxForm::Form4797, "2".into()));
        let short = format!("k1_{id}_section179_sale_2_section1231");
        assert_eq!(value(&result, &short), 0);
        assert!(result.lines.iter().find(|v| v.id == short).unwrap().form_line.is_none());
        assert_eq!(form(&result, "11"), -1_000);
    }
}
