use crate::tests::support::assert::k1_consolidated::{
    k1_form4797_sum as form, k1_supported_without_issues as supported,
};
// Approved whole-return sale oracles, independent of source-owner previews.
use super::*;
use crate::tests::passthrough::k1_sale_return::{draft, sources};

#[test]
fn k1_sale_final_f2_complete_return_keeps_approved_amounts() {
    for mut source in sources(15_000, 2_000, 2_000, 30_000) {
        k1_qbi_statement(&mut source, 15_000, false);
        let (d, id) = draft(source);
        let r = supported(&d);
        for (key, expected) in [
            ("agi", 21_000),
            ("qbi_deduction", 980),
            ("taxable_income", 3_920),
            ("self_employment_tax", 0),
            ("section179_prior_carryover_total", 0),
        ] {
            assert_eq!(value(&r, key), expected, "{key}");
        }
        for (line, expected) in [
            ("22", 30_000),
            ("23", 50_000),
            ("24", 26_000),
            ("25", 24_000),
            ("26", 6_000),
            ("27a", 26_000),
            ("27b", 6_000),
            ("33", 6_000),
            ("14", 6_000),
            ("19", 6_000),
            ("20b", 6_000),
            ("6", 0),
        ] {
            assert_eq!(form(&r, line), expected, "line {line}");
        }
        assert_eq!(value(&r, "schedule_1_other_gains"), 6_000);
        assert_eq!(value(&r, &alloc::format!("k1_{id}_section179_sale_1_recapture")), 6_000);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

#[test]
fn k1_sale_final_f2b_complete_return_keeps_loss_and_history() {
    for mut source in sources(15_000, 2_000, 2_000, 20_000) {
        k1_qbi_statement(&mut source, 15_000, false);
        let (d, id) = draft(source);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 11_000);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        // Section 199A(i): the 2026 active-business minimum applies even
        // when the ordinary taxable-income limit would produce zero.
        assert_eq!(value(&r, "qbi_total"), 11_000);
        assert_eq!(value(&r, "qbi_deduction"), 400);
        assert_eq!(value(&r, "section1231_current_loss_history"), 4_000);
        assert_eq!(value(&r, "section1231_2026_next_lookback"), 4_000);
        assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 4_000);
        assert_eq!(value(&r, "schedule_1_other_gains"), -4_000);
        for line in ["2", "8", "12", "19", "20b"] {
            assert_eq!(form(&r, line), -4_000, "line {line}");
        }
        assert_eq!(value(&r, &alloc::format!("k1_{id}_section179_sale_1_section1231")), -4_000);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

fn participation(source: &mut FederalIncome, active: bool, trading: bool) {
    let kind = if trading { ScorpTradeKind::FinancialTrading } else { ScorpTradeKind::Ordinary };
    match source {
        FederalIncome::Partnership { data } => {
            data.trade_kind = kind;
            let PartnerParticipation::General { facts } = &mut data.participation else {
                unreachable!()
            };
            facts.minutes = if active { 30_001 } else { 0 };
        }
        FederalIncome::Scorp { data } => {
            data.trade_kind = kind;
            data.participation.as_mut().unwrap().minutes = if active { 30_001 } else { 0 };
        }
        _ => unreachable!(),
    }
}

#[test]
fn k1_sale_final_niit_keeps_passive_and_trading_sales_and_excludes_active_business() {
    for (price, net) in [(30_000, 21_000), (60_000, 51_000), (20_000, 11_000)] {
        for (active, trading) in [(true, false), (false, false), (true, true), (false, true)] {
            for mut source in sources(15_000, 2_000, 2_000, price) {
                participation(&mut source, active, trading);
                let (mut d, _) = draft(source);
                let id = d.next_id;
                add(&mut d, wage(id, Owner::Taxpayer, 25_000_000, 0));
                let r = supported(&d);
                let expected = if active && !trading { 0 } else { net * 38 / 1000 };
                assert_eq!(
                    value(&r, "net_investment_income_tax"),
                    expected,
                    "price {price}, active {active}, trading {trading}"
                );
                assert_eq!(value(&r, "self_employment_tax"), 0);
                assert_eq!(value(&r, "agi"), 250_000 + net);
            }
        }
    }
}

#[test]
fn k1_sale_final_amt_depreciation_difference_reaches_line_two_k_once() {
    for mut source in sources(15_000, 2_000, 2_000, 30_000) {
        k1_qbi_statement(&mut source, 15_000, false);
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    600_000
            }
            FederalIncome::Scorp { data } => {
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    600_000
            }
            _ => unreachable!(),
        }
        let (d, id) = draft(source);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 21_000);
        assert_eq!(value(&r, "qbi_deduction"), 980);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), 18_020);
        assert_eq!(value(&r, "amt_disposition_adjustment"), -2_000);
        assert_eq!(
            printed_line(&r, "amt_disposition_adjustment"),
            (TaxForm::Form6251, "2k".into())
        );
        assert_eq!(value(&r, &alloc::format!("amt_k1_{id}_section179_sale_1_recapture")), 4_000);
        assert_eq!(form(&r, "27b"), 6_000);
        assert!(
            r.lines
                .iter()
                .filter(|v| v.id.starts_with(&alloc::format!("amt_k1_{id}_section179_sale_")))
                .all(|v| v.form_line.is_none())
        );
    }
}

#[test]
fn k1_sale_final_mixed_forms_foot_to_schedule_one_once() {
    for mut source in sources(15_000, 0, 0, 30_000) {
        let extra = if matches!(source, FederalIncome::Partnership { .. }) { 5_000 } else { 0 };
        k1_qbi_statement(&mut source, 15_000, false);
        match &mut source {
            FederalIncome::Partnership { data } => {
                let mut loss = data.section179_sales[0].clone();
                loss.regular.as_mut().unwrap().price = 1_800_000;
                let mut short = loss.clone();
                short.regular.as_mut().unwrap().price = 2_100_000;
                short.acquired = 20260101;
                data.section179_sales.extend([loss, short]);
                data.income.push(PartnerIncome {
                    code: PartnerIncomeCode::SpeciallyAllocatedOrdinary,
                    amount: 500_000,
                });
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                let mut loss = data.section179_sales[0].clone();
                loss.regular.as_mut().unwrap().price = 1_800_000;
                let mut short = loss.clone();
                short.regular.as_mut().unwrap().price = 2_100_000;
                short.acquired = 20260101;
                data.section179_sales.extend([loss, short]);

                let amt = amt_records(data);
                for sale in &mut data.section179_sales {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
                data.amt = Some(amt);
            }
            _ => unreachable!(),
        }
        let (d, _) = draft(source);
        let r = supported(&d);
        for (key, expected) in [
            ("agi", 18_000 + extra),
            ("qbi_total", 18_000),
            ("qbi_deduction", if extra == 0 { 400 } else { 1_380 }),
            ("taxable_income", if extra == 0 { 1_500 } else { 5_520 }),
            ("schedule_1_other_gains", 3_000 + extra),
            ("self_employment_tax", 0),
        ] {
            assert_eq!(value(&r, key), expected, "{key}");
        }
        for (line, expected) in [
            ("2", -4_000),
            ("8", -4_000),
            ("11", extra - 1_000),
            ("12", -4_000),
            ("14", 8_000),
            ("19", 3_000 + extra),
            ("20b", 3_000 + extra),
            ("33", 8_000),
        ] {
            assert_eq!(form(&r, line), expected, "line {line}");
        }
        assert_eq!(
            (11..=18).map(|line| form(&r, &alloc::format!("{line}"))).sum::<i64>(),
            3_000 + extra
        );
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

#[test]
fn k1_sale_final_sale_never_adds_to_a_general_partners_reported_se() {
    for price in [18_000, 30_000, 60_000] {
        let FederalIncome::Partnership { mut data } =
            sources(15_000, 0, 0, price).into_iter().next().unwrap()
        else {
            unreachable!()
        };
        data.limited_for_se = false;
        data.reported_se = Some(1_500_000);
        let mut baseline = data.clone();
        baseline.section179_sales.clear();
        baseline.alternative = Some(baseline.regular.clone());
        let (base, _) = draft(FederalIncome::Partnership { data: baseline });
        let before = supported(&base);
        let (d, _) = draft(FederalIncome::Partnership { data });
        let after = supported(&d);
        assert_eq!(value(&after, "self_employment_tax"), value(&before, "self_employment_tax"));
        let source = after
            .review_notes
            .iter()
            .find(|n| n.display.key == "NOTE_PARTNERSHIP_SE_SOURCE")
            .unwrap();
        assert_eq!(source.display.args[1], MessageArg::usd_dollars(15_000));
        let line_two = after
            .lines
            .iter()
            .filter(|v| {
                v.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleSe && f.line == "2")
            })
            .map(|v| v.amount / 100)
            .sum::<i64>();
        assert_eq!(line_two, 15_000);
    }
}

fn plain_sale(loss: i64) -> K1Section179Sale {
    k1_reported_sale("Synthetic equipment", 20230301, [(20_000 - loss) * 100, 2_000_000, 0, 0])
}

#[test]
fn k1_sale_final_amt_uses_allowed_property_losses_and_keeps_passive_bounds() {
    for (basis, risk, active, regular, amt) in [
        ([3_000; 2], [10_000; 2], true, 3_000, 2_000),
        ([10_000; 2], [1_000, 3_000], true, 1_000, 2_000),
        ([10_000; 2], [10_000; 2], false, 0, 0),
    ] {
        for mut source in sources(0, 0, 0, 20_000) {
            crate::tests::passthrough::k1_sale_amt::disjoint_view_counterpart(
                &mut source,
                vec![plain_sale(4_000)],
                vec![plain_sale(2_000)],
                basis,
                risk,
            );
            participation(&mut source, active, false);
            let (mut d, id) = draft(source);
            let wage_id = d.next_id;
            add(&mut d, wage(wage_id, Owner::Taxpayer, 2_000_000, 0));
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 20_000 - regular);
            assert_eq!(value(&r, "alternative_minimum_taxable_income"), 20_000 - amt);
            assert_eq!(value(&r, "amt_disposition_adjustment"), regular - amt);
            assert_eq!(
                value(&r, &alloc::format!("k1_{id}_section179_sale_1_section1231")),
                -regular
            );
            assert_eq!(
                value(&r, &alloc::format!("amt_k1_{id}_section179_sale_1_section1231")),
                -amt
            );
        }
    }
}

#[test]
fn k1_sale_final_amt_separates_passive_ordinary_and_disposition_differences() {
    for mut source in sources(5_000, 0, 0, 20_000) {
        crate::tests::passthrough::k1_sale_amt::disjoint_view_counterpart(
            &mut source,
            vec![plain_sale(4_000)],
            vec![plain_sale(4_000)],
            [10_000; 2],
            [10_000; 2],
        );
        participation(&mut source, false, false);
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.alternative.as_mut().unwrap().ordinary = 300_000
            }
            FederalIncome::Scorp { data } => data.amt.as_mut().unwrap().adjustment = -200_000,
            _ => unreachable!(),
        }
        let (mut d, _) = draft(source);
        let id = d.next_id;
        add(&mut d, wage(id, Owner::Taxpayer, 2_000_000, 0));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 21_000);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), 20_000);
        assert_eq!(value(&r, "amt_disposition_adjustment"), 1_000);
        assert_eq!(value(&r, "amt_rental_adjustment"), -1_000);
    }
}

#[test]
fn k1_sale_final_f2c_multiple_properties_keep_the_exact_allocation_refusal() {
    for mut source in sources(15_000, 2_000, 2_000, 30_000) {
        match &mut source {
            FederalIncome::Partnership { data } => {
                data.section179_sales.push(data.section179_sales[0].clone());
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                data.section179_sales.push(data.section179_sales[0].clone());
                let amt = amt_records(data);
                for sale in &mut data.section179_sales {
                    sale.alternative =
                        crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
                }
                data.amt = Some(amt);
            }
            _ => unreachable!(),
        }
        let (d, _) = draft(source);
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(r.lines.is_empty());
        assert!(r.issues.iter().any(|v| v.display.key == "ISSUE_K1_SALE_CARRYOVER_SPLIT"));
    }
}

#[test]
fn k1_sale_final_explicit_unsupported_other_information_is_still_refused() {
    for mut source in sources(15_000, 2_000, 2_000, 30_000) {
        let key = match &mut source {
            FederalIncome::Partnership { data } => {
                data.unentered.push(PartnerK1Item::OtherInformation);
                "ISSUE_PARTNER_K1_OTHER_INFORMATION"
            }
            FederalIncome::Scorp { data } => {
                data.unentered.push(ScorpK1Item::OtherInformation);
                "ISSUE_SCORP_K1_OTHER_INFORMATION"
            }
            _ => unreachable!(),
        };
        let (d, _) = draft(source);
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(r.lines.is_empty());
        assert!(r.issues.iter().any(|v| v.display.key == key));
    }
}
