use crate::tests::support::assert::k1_consolidated::{
    k1_form4797_sum as form, k1_supported_without_issues as supported,
};
// Compound B9-7 owners: original business property, section179 sale and
// disposed rental share one lookback while retaining QBI/NII/AMT character.
use super::*;
use crate::tests::passthrough::k1_sale_return::{draft, sources};

fn property(net: i64, unrecaptured: i64) -> K1Section1231 {
    k1_section1231(net, unrecaptured)
}
fn compound(mut source: FederalIncome) -> (FederalDraft, i64) {
    k1_qbi_statement(&mut source, 15_000, false);
    let rental = K1RentalAmounts {
        real_estate: -200_000,
        real_estate_carryover: 900_000,
        real_estate_property: K1RentalProperty {
            net: 600_000,
            unrecaptured: 400_000,
            passive_loss: 0,
        },
        ..Default::default()
    };
    match &mut source {
        FederalIncome::Partnership { data } => {
            data.limited_for_se = Some(true);
            data.reported_se = Some(0);
            data.regular.section1231 = Some(property(12_000, 3_000));
            data.regular.rentals = rental;
            data.rental_facts.participation = Some(K1RentalParticipation::NotActive);
            data.rental_facts.disposed = vec![K1RentalActivity::RealEstate];
            data.qbi.as_mut().unwrap().section1231_qbi = 1_200_000;
            data.alternative = Some(data.regular.clone());
        }
        FederalIncome::Scorp { data } => {
            let mut source_edit = edit_scorp(data);
            let data = &mut *source_edit;

            data.section1231 = Some(property(12_000, 3_000));
            data.rentals = rental;
            data.rental_facts.participation = Some(K1RentalParticipation::NotActive);
            data.rental_facts.disposed = vec![K1RentalActivity::RealEstate];
            data.property_qbi = ScorpPropertyQbi {
                gain: 1_200_000,
                qualified_1250: Some(300_000),
                complete: true,
                ..Default::default()
            };
            data.amt = Some(amt_records(data));
            for sale in &mut data.section179_sales {
                sale.alternative =
                    crate::K1Section179SaleAmtSource::from_known_regular(&sale.regular);
            }
        }
        _ => unreachable!(),
    }
    let (mut d, id) = draft(source);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2024, 7_000, 7_000)], &[(2024, 7_000, 7_000)]));
    let next = d.next_id;
    add(&mut d, wage(next, Owner::Taxpayer, 8_000_000, 0));
    (d, id)
}
#[test]
fn k1_compound_original_sale_rental_and_lookback_keep_one_final_footing() {
    for source in sources(15_000, 0, 0, 60_000) {
        let (d, id) = compound(source);
        let r = supported(&d);
        // Business 15 + ordinary recapture28, property12+10+6=28,
        // and disposed rental ordinary -2-9=-11; wages80 => AGI140.
        for (key, expected) in [
            ("agi", 140_000),
            ("qbi_total", 46_000),
            ("qbi_deduction", 9_200),
            ("taxable_income", 114_700),
            ("self_employment_tax", 0),
            ("schedule_1_other_gains", 35_000),
        ] {
            assert_eq!(value(&r, key), expected, "{key}");
        }
        for (line, expected) in [
            ("2", 18_000),
            ("6", 10_000),
            ("8", 28_000),
            ("9", 7_000),
            ("10", 21_000),
            ("13", 7_000),
            ("14", 28_000),
            ("19", 35_000),
            ("20b", 35_000),
        ] {
            assert_eq!(form(&r, line), expected, "line {line}");
        }
        assert!(!r.lines.iter().any(|v| v.id == "section1231_2024_next_lookback"
            || v.id == "amt_section1231_2024_next_lookback"));
        let mut scratch = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 80_000, &mut scratch, false);
        assert_eq!(pool.k1_section1231.len(), 1);
        assert_eq!(pool.k1_section1231[0].id, id);
        assert_eq!(pool.k1_section1231[0].business().net, 22_000);
        assert_eq!(pool.k1_section1231[0].rentals[0].net, 6_000);
        let worksheet = crate::section1231::worksheet(&d, &pool, false).unwrap().unwrap();
        assert_eq!(worksheet.history[0].recaptured, 7_000);
        assert_eq!(worksheet.history[0].carried, 0);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}
#[test]
fn k1_compound_independent_amt_dispositions_and_rental_release_are_not_added_twice() {
    for source in sources(15_000, 0, 0, 60_000) {
        let (mut d, _) = compound(source);
        match &mut d.income[0].income {
            FederalIncome::Partnership { data } => {
                let amt = data.alternative.as_mut().unwrap();
                amt.section1231.as_mut().unwrap().net = 1_000_000;
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    600_000;
                amt.rentals.real_estate_property.net = 400_000;
                amt.rentals.real_estate = -100_000;
            }
            FederalIncome::Scorp { data } => {
                let mut source_edit = edit_scorp(data);
                let data = &mut *source_edit;

                let amt = data.amt.as_mut().unwrap();
                amt.section1231.as_mut().unwrap().net = 1_000_000;
                data.section179_sales[0].alternative.reported_amounts_mut().unwrap().depreciation =
                    600_000;
                amt.rentals.real_estate_property.net = 400_000;
                amt.rentals.real_estate = -100_000;
            }
            _ => unreachable!(),
        }
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 140_000);
        assert_eq!(value(&r, "qbi_deduction"), 9_200);
        assert_eq!(value(&r, "amt_disposition_adjustment"), -6_000);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), 125_800);
        assert_eq!(form(&r, "19"), 35_000);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}
#[test]
fn k1_compound_niit_keeps_rental_release_separate_from_active_and_trading_business() {
    for source in sources(15_000, 0, 0, 60_000) {
        for trading in [false, true] {
            let (mut d, _) = compound(source.clone());
            let wage_id = d.income[1].id;
            d.income[1] = wage(wage_id, Owner::Taxpayer, 25_000_000, 0);
            match &mut d.income[0].income {
                FederalIncome::Partnership { data } => {
                    data.trade_kind = if trading {
                        ScorpTradeKind::FinancialTrading
                    } else {
                        ScorpTradeKind::Ordinary
                    }
                }
                FederalIncome::Scorp { data } => {
                    let mut source_edit = edit_scorp(data);
                    let data = &mut *source_edit;

                    data.trade_kind = if trading {
                        ScorpTradeKind::FinancialTrading
                    } else {
                        ScorpTradeKind::Ordinary
                    }
                }
                _ => unreachable!(),
            }
            add(&mut d, income(savings_income(20_000, 0)));
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 330_000);
            // Active ordinary excludes business65; disposed rental -5
            // remains a deduction against interest20. Trading includes65.
            assert_eq!(value(&r, "net_investment_income_tax"), if trading { 3_040 } else { 570 });
            assert_eq!(value(&r, "self_employment_tax"), 0);
        }
    }
}

#[test]
fn k1_compound_two_entity_sources_share_one_lookback_without_repeating_property_rows() {
    let [partner, scorp] = sources(15_000, 0, 0, 60_000);
    let (mut d, first) = compound(partner);
    let (other, _) = compound(scorp);
    let second = d.next_id;
    add(&mut d, income(other.income[0].income.clone()));
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2024, 14_000, 14_000)], &[(2024, 14_000, 14_000)]));
    for reverse in [false, true] {
        if reverse {
            d.income.reverse();
        }
        let r = supported(&d);
        for (key, expected) in [
            ("agi", 200_000),
            ("qbi_total", 92_000),
            ("qbi_deduction", 18_400),
            ("taxable_income", 165_500),
            ("schedule_1_other_gains", 70_000),
        ] {
            assert_eq!(value(&r, key), expected, "{key}");
        }
        for (line, expected) in [
            ("2", 36_000),
            ("6", 20_000),
            ("8", 56_000),
            ("9", 14_000),
            ("10", 42_000),
            ("14", 56_000),
            ("19", 70_000),
        ] {
            assert_eq!(form(&r, line), expected, "line {line}");
        }
        let mut scratch = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 80_000, &mut scratch, false);
        assert_eq!(pool.k1_section1231.len(), 2);
        for id in [first, second] {
            assert_eq!(pool.k1_section1231.iter().find(|v| v.id == id).unwrap().net, 28_000);
        }
        let ids: alloc::collections::BTreeSet<_> = r.lines.iter().map(|v| &v.id).collect();
        assert_eq!(ids.len(), r.lines.len());
    }
}
#[test]
fn k1_compound_capital_loss_netting_changes_amt_income_not_the_property_lookback() {
    for source in sources(15_000, 0, 0, 60_000) {
        let (mut d, _) = compound(source);
        d.screening.as_mut().unwrap().capital_history = vec![PriorCapitalReturn {
            injured_niit_owners: None,
            injured_owners: None,

            owners: PriorReturnOwners::Taxpayer,
            status: FederalStatus::Single,
            agi: 10_000_000,
            deductions: 1_000_000,
            senior_deduction: 0,
            short: 0,
            long: -3_300_000,
            deducted_loss: 300_000,
            section_1256: 0,
            amt: Some(PriorAmtCapitalReturn {
                taxable_income: 9_000_000,
                short: 0,
                long: 0,
                deducted_loss: 0,
            }),
            origins: vec![],
            ordinary_history: true,
            investment_parts: Some(InvestmentCapitalParts {
                short: 0,
                long: 3000000,
                amt: Some(InvestmentAmtCapitalParts { short: 0, long: 0 }),
            }),
            niit_difference: Some(0),
            research_parts: None,
        }];
        let r = supported(&d);
        // Regular carry30 offsets property capital21 => loss9, deduction3,
        // remainder6. AMT has no prior carry and includes capital21.
        assert_eq!(value(&r, "agi"), 116_000);
        assert_eq!(value(&r, "qbi_deduction"), 9_200);
        assert_eq!(form(&r, "8"), 28_000);
        assert_eq!(form(&r, "13"), 7_000);
        assert_eq!(value(&r, "amt_disposition_adjustment"), 24_000);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), 130_800);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}
#[test]
fn k1_compound_partial_high_rate_lookback_preserves_named_allocation_refusal() {
    for source in sources(15_000, 0, 0, 60_000) {
        let (mut d, _) = compound(source);
        d.screening.as_mut().unwrap().section1231 =
            Some(section1231_history(&[(2024, 5_000, 5_000)], &[(2024, 5_000, 5_000)]));
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(r.issues.iter().any(|i| i.display.key
            == "ISSUE_SECTION1231_SECTION_1231_LOOKBACK_RECAPTURE_NEEDS_BUSINESS_LEVEL_F77D46EA"));
        assert!(r.lines.is_empty());
    }
}
