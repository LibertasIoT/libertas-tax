use super::*;

#[test]
fn return_of_capital_uses_basis_before_recognizing_gain() {
    for (distribution, basis, reduction, remaining, gain) in [
        (1000, 5000, 1000, 4000, 0),
        (5000, 5000, 5000, 0, 0),
        (8000, 5000, 5000, 0, 3000),
        (3000, 0, 0, 0, 3000),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(capital_return(distribution, basis, GainTerm::Long)));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), gain);
        assert_eq!(value(&r, "agi"), 60000 + gain);
        let id = d.income[1].id;
        for (suffix, expected) in
            [("basis_reduction", reduction), ("remaining_basis", remaining), ("capital_gain", gain)]
        {
            assert_eq!(value(&r, &alloc::format!("distribution_{id}_{suffix}")), expected);
        }
        assert_eq!(value(&r, "income_tax"), 5023);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn return_of_capital_does_not_pool_basis_between_lots() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(capital_return(3000, 0, GainTerm::Short)));
    add(&mut d, income(capital_return(1000, 9000, GainTerm::Long)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 3000);
    assert_eq!(value(&r, "income_tax"), 5383);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    let id = d.income[2].id;
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_remaining_basis")), 8000);
    let FederalIncome::ReturnOfCapital { special, .. } = &mut d.income[1].income else { panic!() };
    *special = true;
    assert!(noted(&supported(&d), "WARN_RETURN_OF_CAPITAL_LEFT_OUT"));
}
#[test]
fn section1244_basis_split_precedes_annual_limit_and_preserves_holding_period() {
    let mut d = complete(FederalStatus::Single);
    let mut stock = Section1244Stock {
        basis: StockLossBasis::Cash { cost: 10000000, added: 2000000 },
        acquired: 20200101,
        event: StockLossEvent::Worthless { confirmed: true },

        ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
    };
    add(&mut d, income(FederalIncome::SmallBusinessStock { data: stock.clone() }));
    let r = supported(&d);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 50000);
    assert_eq!(value(&r, "capital_long_carryforward"), 70000);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    stock.acquired = 20260101;
    stock.basis = StockLossBasis::Property {
        liabilities: 0,
        basis: 10000000,
        value: 2000000,
        acquired: 20200101,
    };
    assert_eq!(stock.term(), GainTerm::Long);
    d.income[1].income = FederalIncome::SmallBusinessStock { data: stock.clone() };
    let r = supported(&d);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 20000);
    assert_eq!(value(&r, "capital_long_carryforward"), 77000);
    stock.basis = StockLossBasis::Property {
        liabilities: 0,
        basis: 10000000,
        value: 2000000,
        acquired: 20260101,
    };
    assert_eq!(stock.term(), GainTerm::Short);
    d.income[1].income = FederalIncome::SmallBusinessStock { data: stock };
    assert_eq!(value(&supported(&d), "capital_short_carryforward"), 77000);
}
#[test]
fn section1244_basis_sources_reject_impossible_records_and_preserve_cents() {
    let mut stock = Section1244Stock {
        basis: StockLossBasis::Cash { cost: 1, added: 2 },
        acquired: 20260101,
        event: StockLossEvent::Worthless { confirmed: true },

        ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
    };
    assert!(stock.valid());
    assert_eq!(stock.ordinary_loss(), 1);
    assert_eq!(stock.loss(), 3);
    stock.basis = StockLossBasis::Cash { cost: 0, added: 0 };
    assert_eq!(stock.ordinary_loss(), 0);
    stock.basis = StockLossBasis::Cash { cost: 100000000000, added: 1 };
    assert!(!stock.valid());
    stock.basis =
        StockLossBasis::Property { liabilities: 0, basis: 100, value: 50, acquired: 20260102 };
    assert!(!stock.valid());
}

#[test]
fn section1244_assumed_mortgage_reduces_basis_before_builtin_loss_adjustment() {
    for (fair_value, liabilities, ordinary, capital) in [
        (15000, 2000, 13000, 5000),
        (15000, 18000, 0, 2000),
        (25000, 2000, 18000, 0),
        (15000, 20000, 0, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        let stock = Section1244Stock {
            basis: StockLossBasis::Property {
                basis: 2000000,
                value: fair_value * 100,
                liabilities: liabilities * 100,
                acquired: 20200101,
            },
            acquired: 20260101,
            event: StockLossEvent::Worthless { confirmed: true },

            ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
        };
        assert!(stock.valid());
        assert_eq!(stock.loss(), (20000 - liabilities) * 100);
        add(&mut d, income(FederalIncome::SmallBusinessStock { data: stock }));
        let r = supported(&d);
        assert_eq!(value(&r, "section_1244_ordinary_loss"), ordinary);
        assert_eq!(value(&r, "capital_gain"), -capital.min(3000));
        assert_eq!(value(&r, "agi"), 60000 - ordinary - capital.min(3000));
        assert_eq!(value(&r, "capital_long_carryforward"), (capital - 3000).max(0));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn section1244_capital_excess_and_basis_split_enter_investment_loss_pool_once() {
    for acquired in [20200101, 20260101] {
        let mut d = complete(FederalStatus::Single);
        let mut stock = section1244_loss(60000, 0, acquired);
        let FederalIncome::SmallBusinessStock { data } = &mut stock else { panic!() };
        data.basis = StockLossBasis::Cash { cost: 6000000, added: 1000000 };
        add(&mut d, income(stock));
        add(&mut d, income(collectible_sale(80000, 0, GainTerm::Short, false)));
        investment_borrowing(&mut d, 20000, 0, 0);
        let r = supported(&d);
        // 70,000 total loss: 50,000 ordinary, 10,000 basis capital loss,
        // and 10,000 capital excess over the ordinary allowance.
        assert_eq!(value(&r, "section_1244_ordinary_loss"), 50000);
        assert_eq!(value(&r, "capital_gain"), 60000);
        assert_eq!(value(&r, "investment_interest_deduction"), 10000);
        assert_eq!(value(&r, "investment_interest_carryforward"), 10000);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), 10000);
    }
}

#[test]
fn current_sale_separate_amt_basis_recomputes_loss_limit_carry_and_tax() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic unrelated ISO", 2000.0, 1000, 11000)];
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(60000, 10000, GainTerm::Long), 100000, 0)),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 50000);
    assert_eq!(value(&r, "agi"), 110000);
    assert_eq!(value(&r, "amt_capital_gain"), -3000);
    assert_eq!(value(&r, "amt_capital_adjustment"), -53000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 37000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 257000);
    // (257,000 AMTI - 90,100 exemption) * 26%; no AMT capital gain.
    assert_eq!(r.tax, Some(4339400));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn section1231_matches_publication_544_2025_lookback_example() {
    use crate::section1231::{Item, LossYear, calculate};
    // IRS Pub. 544: 2022 loss $2,500, of which $1,800 was recaptured in 2024.
    // The 2025 $2,000 gain has $700 ordinary and $1,300 capital character.
    let r = calculate(
        2025,
        &[Item { net: 2_000, unrecaptured: 0 }],
        &[LossYear { year: 2022, original: 2_500, remaining: 700 }],
    )
    .unwrap();
    assert_eq!((r.net, r.lookback, r.ordinary, r.long_term), (2_000, 700, 700, 1_300));
    assert_eq!(r.history[0].original, 2_500);
    assert_eq!(r.history[0].recaptured, 700);
    assert_eq!(r.history[0].carried, 0);
    assert!(r.next_history(0).unwrap().is_empty());
}
#[test]
fn section1231_matches_form_4797_2025_fifo_example_and_rolls_into_2026() {
    use crate::section1231::{Item, LossYear, calculate};
    // IRS 4797 example: $4,000/$6,000 losses in 2020/2021; a $3,000
    // 2024 gain has already reduced the first loss. Enter retained balances.
    let r = calculate(
        2025,
        &[Item { net: 2_000, unrecaptured: 0 }],
        &[
            LossYear { year: 2021, original: 6_000, remaining: 6_000 },
            LossYear { year: 2020, original: 4_000, remaining: 1_000 },
        ],
    )
    .unwrap();
    assert_eq!((r.lookback, r.ordinary, r.long_term), (7_000, 2_000, 0));
    assert_eq!(
        r.history.iter().map(|v| (v.year, v.recaptured)).collect::<alloc::vec::Vec<_>>(),
        vec![(2020, 1_000), (2021, 1_000)]
    );
    let next = r.next_history(0).unwrap();
    assert_eq!(next, vec![LossYear { year: 2021, original: 6_000, remaining: 5_000 }]);
    let r = calculate(2026, &[Item { net: 6_000, unrecaptured: 0 }], &next).unwrap();
    assert_eq!((r.ordinary, r.long_term), (5_000, 1_000));
    assert!(r.next_history(0).unwrap().is_empty());
}
#[test]
fn section1231_five_year_boundary_expires_only_after_current_year_use() {
    use crate::section1231::{Item, LossYear, calculate};
    let r = calculate(
        2026,
        &[Item { net: 200, unrecaptured: 0 }],
        &[
            LossYear { year: 2025, original: 2_000, remaining: 2_000 },
            LossYear { year: 2020, original: 900, remaining: 900 },
            LossYear { year: 2022, original: 1_000, remaining: 1_000 },
            LossYear { year: 2021, original: 500, remaining: 500 },
        ],
    )
    .unwrap();
    assert_eq!(r.lookback, 3_500);
    assert_eq!(r.ordinary, 200);
    assert_eq!(r.history[0].expired, 900);
    assert_eq!(r.history[0].recaptured, 0);
    assert_eq!(r.history[1].recaptured, 200);
    assert_eq!(r.history[1].expired, 300);
    for v in &r.history {
        assert_eq!(v.opening, v.recaptured + v.expired + v.carried);
    }
    let next = r.next_history(0).unwrap();
    assert_eq!(next.iter().map(|v| v.year).collect::<alloc::vec::Vec<_>>(), vec![2022, 2025]);
    let later = calculate(2027, &[], &next).unwrap();
    assert_eq!(later.history[0].expired, 1_000);
    assert_eq!(
        later.next_history(0).unwrap(),
        vec![LossYear { year: 2025, original: 2_000, remaining: 2_000 }]
    );
}
#[test]
fn section1231_nets_activities_before_lookback_and_1250_cap() {
    use crate::section1231::{Item, LossYear, calculate};
    let items =
        [Item { net: 20_000, unrecaptured: 10_000 }, Item { net: -12_000, unrecaptured: 0 }];
    let prior = [LossYear { year: 2024, original: 3_000, remaining: 3_000 }];
    let r = calculate(2026, &items, &prior).unwrap();
    assert_eq!((r.net, r.ordinary, r.long_term, r.unrecaptured), (8_000, 3_000, 5_000, 5_000));
    assert_eq!(r, calculate(2026, &[items[1], items[0]], &prior).unwrap());
    // A K-1 net can include offsetting property losses even when its
    // separately reported unrecaptured 1250 amount exceeds that net.
    assert_eq!(r, calculate(2026, &[Item { net: 8_000, unrecaptured: 10_000 }], &prior).unwrap());
}
#[test]
fn section1231_lookback_consumes_business_1250_before_preferential_gain() {
    use crate::section1231::{Item, LossYear, calculate};
    for (component, prior, capital, special) in [
        (7_000, 3_000, 7_000, 4_000),
        (2_000, 3_000, 7_000, 0),
        (20_000, 3_000, 7_000, 7_000),
        (7_000, 30_000, 0, 0),
        (7_000, 0, 10_000, 7_000),
    ] {
        let history = if prior == 0 {
            vec![]
        } else {
            vec![LossYear { year: 2025, original: prior, remaining: prior }]
        };
        let r =
            calculate(2026, &[Item { net: 10_000, unrecaptured: component }], &history).unwrap();
        assert_eq!((r.long_term, r.unrecaptured), (capital, special));
        assert_eq!(r.ordinary + r.long_term, 10_000);
    }
}
#[test]
fn section1231_current_loss_requires_final_deductibility_before_new_history() {
    use crate::section1231::{Error1231, Item, LossYear, calculate};
    let r = calculate(
        2026,
        &[Item { net: 7_000, unrecaptured: 5_000 }, Item { net: -9_000, unrecaptured: 0 }],
        &[LossYear { year: 2024, original: 3_000, remaining: 1_000 }],
    )
    .unwrap();
    assert_eq!((r.net, r.ordinary, r.long_term, r.unrecaptured), (-2_000, -2_000, 0, 0));
    assert_eq!(r.history[0].recaptured, 0);
    assert_eq!(r.retained.len(), 1);
    let history = r.next_history(2_000).unwrap();
    assert_eq!(history[1], LossYear { year: 2026, original: 2_000, remaining: 2_000 });
    assert_eq!(r.next_history(0).unwrap().len(), 1);
    assert_eq!(r.next_history(1_500).unwrap()[1].remaining, 1_500);
    assert_eq!(r.next_history(2_001), Err(Error1231::InvalidAmount));
    assert_eq!(r.next_history(-1), Err(Error1231::InvalidAmount));
    let zero = calculate(2026, &[Item { net: 0, unrecaptured: 5_000 }], &[]).unwrap();
    assert_eq!((zero.ordinary, zero.long_term, zero.unrecaptured), (0, 0, 0));
    assert_eq!(zero.next_history(1), Err(Error1231::InvalidAmount));
}
#[test]
fn section1231_regular_and_amt_classifications_use_independent_sources() {
    use crate::section1231::{Item, LossYear, calculate};
    let regular = calculate(
        2026,
        &[Item { net: 10_000, unrecaptured: 6_000 }],
        &[LossYear { year: 2023, original: 5_000, remaining: 3_000 }],
    )
    .unwrap();
    let amt = calculate(
        2026,
        &[Item { net: 8_000, unrecaptured: 7_000 }],
        &[LossYear { year: 2023, original: 4_000, remaining: 2_000 }],
    )
    .unwrap();
    assert_eq!((regular.ordinary, regular.long_term, regular.unrecaptured), (3_000, 7_000, 3_000));
    assert_eq!((amt.ordinary, amt.long_term, amt.unrecaptured), (2_000, 6_000, 5_000));
    assert_eq!(regular.history[0].original, 5_000);
    assert_eq!(amt.history[0].original, 4_000);
}
#[test]
fn section1231_rejects_invalid_history_and_unrepresentable_totals() {
    use crate::section1231::{Error1231, Item, LossYear, calculate};
    for row in [
        LossYear { year: 2026, original: 1, remaining: 1 },
        LossYear { year: 0, original: 1, remaining: 1 },
        LossYear { year: 2025, original: 0, remaining: 0 },
        LossYear { year: 2025, original: 1, remaining: 2 },
        LossYear { year: 2025, original: 1, remaining: -1 },
    ] {
        assert_eq!(calculate(2026, &[], &[row]), Err(Error1231::InvalidHistory));
    }
    let row = LossYear { year: 2025, original: 1, remaining: 1 };
    assert_eq!(calculate(2026, &[], &[row, row]), Err(Error1231::InvalidHistory));
    assert_eq!(calculate(5, &[], &[]), Err(Error1231::InvalidHistory));
    assert_eq!(
        calculate(2026, &[Item { net: 0, unrecaptured: -1 }], &[]),
        Err(Error1231::InvalidAmount)
    );
    assert_eq!(
        calculate(2026, &[Item { net: i64::MIN, unrecaptured: 0 }], &[]),
        Err(Error1231::Overflow)
    );
    assert_eq!(
        calculate(
            2026,
            &[Item { net: i64::MAX, unrecaptured: 0 }, Item { net: 1, unrecaptured: 0 }],
            &[]
        ),
        Err(Error1231::Overflow)
    );
    assert_eq!(
        calculate(
            2026,
            &[Item { net: 0, unrecaptured: i64::MAX }, Item { net: 0, unrecaptured: 1 }],
            &[]
        ),
        Err(Error1231::Overflow)
    );
    let rows = [LossYear { year: 2024, original: i64::MAX, remaining: i64::MAX }, row];
    assert_eq!(calculate(2026, &[], &rows), Err(Error1231::Overflow));
    let r = calculate(
        2026,
        &[
            Item { net: i64::MAX, unrecaptured: 0 },
            Item { net: 1, unrecaptured: 0 },
            Item { net: -1, unrecaptured: 0 },
        ],
        &[],
    )
    .unwrap();
    assert_eq!(r.long_term, i64::MAX);
}

#[test]
fn live_section1231_lookback_uses_retained_irs_example_balance() {
    // Pub. 544's $2,500 loss less $1,800 recaptured gives a $700 balance.
    // Keep that source history and apply a $2,000 gain in 2026.
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, 2_000, 0, 0), 0);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2022, 2_500, 700)], &[]));
    let computed = lookback_worksheet(&d, false).unwrap().unwrap();
    assert_eq!((computed.net, computed.ordinary, computed.long_term), (2_000, 700, 1_300));
    assert_eq!(computed.history[0].recaptured, 700);
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.starts_with("Section 1231 regular 2026 worksheet: net after activity limits $2000; prior five-year nonrecaptured losses $700; ordinary amount $700; long-term amount $1300;")));
    let worksheet = &r
        .review_notes
        .iter()
        .find(|note| note.starts_with("Section 1231 regular 2026 worksheet:"))
        .unwrap()
        .display;
    assert_eq!(worksheet.key, "NOTE_1231_REGULAR_WORKSHEET");
    assert_eq!(worksheet.args[0], MessageArg::Number(2026));
    assert_eq!(worksheet.args[1], MessageArg::usd_dollars(2_000));
    assert_eq!(worksheet.args[2], MessageArg::usd_dollars(700));
    // No box 17 code V statement: no property QBI records are needed.
    assert!(r.tax.is_some(), "{r:?}");
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn section1231_live_worksheet_nets_all_activities_before_lookback_and_1250_cap() {
    let mut gain = former_passive_property_scorp(0, 12_000, 0, 0);
    gain.section1231.as_mut().unwrap().unrecaptured = 1_000_000;
    gain.amt = Some(amt_records(&gain));
    let (mut d, _) = passive_property_pool(gain, 0);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: scorp_source(former_passive_property_scorp(0, -4_000, 0, 0)),
        }),
    );
    // Input order is deliberately not chronological.
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2024, 2_000, 2_000), (2021, 4_000, 3_000)], &[]));
    let c = lookback_worksheet(&d, false).unwrap().unwrap();
    assert_eq!(
        (c.net, c.lookback, c.ordinary, c.long_term, c.unrecaptured),
        (8_000, 5_000, 5_000, 3_000, 3_000)
    );
    assert_eq!(
        c.history.iter().map(|v| (v.year, v.recaptured, v.carried)).collect::<alloc::vec::Vec<_>>(),
        vec![(2021, 3_000, 0), (2024, 2_000, 0)]
    );
    let r = estimate(&d);
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.starts_with("Section 1231 regular 2026 worksheet:"))
            .count(),
        1
    );
}

#[test]
fn section1231_loss_output_expires_old_history_without_manufacturing_current_deductibility() {
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, -6_000, 0, 0), 0);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: scorp_source(former_passive_property_scorp(0, 2_000, 0, 0)),
        }),
    );
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2021, 2_000, 2_000), (2023, 3_000, 3_000)], &[]));
    let c = lookback_worksheet(&d, false).unwrap().unwrap();
    assert_eq!((c.net, c.ordinary, c.long_term, c.unrecaptured), (-4_000, -4_000, 0, 0));
    assert_eq!((c.history[0].expired, c.history[1].carried), (2_000, 3_000));
    let r = estimate(&d);
    let current_loss = &r
        .review_notes
        .iter()
        .find(|note| note.starts_with("Section 1231 regular current net loss"))
        .unwrap()
        .display;
    assert_eq!(current_loss.key, "NOTE_1231_REGULAR_CURRENT_LOSS");
    assert_eq!(current_loss.args, vec![MessageArg::usd_dollars(4_000)]);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.starts_with("Section 1231 regular current net loss $4000:"))
    );
    assert!(!r.review_notes.iter().any(|n| n.starts_with("Section 1231 regular loss year 2026:")));
    // No box 17 code V statement: no property QBI records are needed.
    assert!(r.tax.is_some(), "{r:?}");
}

#[test]
fn section1231_live_worksheet_has_independent_amt_history_and_1250_component() {
    let mut source = former_passive_property_scorp(0, 10_000, 0, 0);
    source.section1231.as_mut().unwrap().unrecaptured = 800_000;
    source.amt = Some(amt_records(&source));
    let amt = source.amt.as_mut().unwrap().section1231.as_mut().unwrap();
    amt.net = 600_000;
    amt.unrecaptured = 400_000;
    let (mut d, _) = passive_property_pool(source, 0);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2022, 4_000, 4_000)], &[(2023, 1_000, 1_000)]));
    let regular = lookback_worksheet(&d, false).unwrap().unwrap();
    let amt = lookback_worksheet(&d, true).unwrap().unwrap();
    assert_eq!(
        (regular.net, regular.ordinary, regular.long_term, regular.unrecaptured),
        (10_000, 4_000, 6_000, 4_000)
    );
    assert_eq!(
        (amt.net, amt.ordinary, amt.long_term, amt.unrecaptured),
        (6_000, 1_000, 5_000, 3_000)
    );
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.starts_with("Section 1231 AMT 2026 worksheet: net after activity limits $6000; prior five-year nonrecaptured losses $1000; ordinary amount $1000; long-term amount $5000; business-asset section 1250 component $3000.")));
    let amt_note = &r
        .review_notes
        .iter()
        .find(|note| note.starts_with("Section 1231 AMT 2026 worksheet:"))
        .unwrap()
        .display;
    assert_eq!(amt_note.key, "NOTE_1231_AMT_WORKSHEET");
    assert_eq!(amt_note.args[1], MessageArg::usd_dollars(6_000));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn section1231_missing_history_is_not_a_confirmed_zero_balance() {
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, 2_000, 0, 0), 0);
    assert!(lookback_worksheet(&d, false).is_err());
    unsupported(&d, "Section 1231 lookback");
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let c = lookback_worksheet(&d, false).unwrap().unwrap();
    assert_eq!((c.lookback, c.ordinary, c.long_term), (0, 0, 2_000));
}

#[test]
fn section1231_lookback_does_not_use_partial_activity_pool_or_missing_amt() {
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, 2_000, 0, 0), 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut other = complete_ordinary_scorp(1_000, false);
    other.passive.complete = false;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(other) }));
    assert!(lookback_worksheet(&d, false).is_err());
    // The estimate leaves the unsupported passive history out.
    assert!(noted(&estimate(&d), "WARN_SCORP_PASSIVE_HISTORY_LEFT_OUT"));
    if let FederalIncome::Scorp { data } = &mut d.income.last_mut().unwrap().income {
        let mut data = edit_scorp(data);
        data.passive.complete = true;
        data.amt = None;
    }
    assert!(lookback_worksheet(&d, false).is_ok());
    assert!(lookback_worksheet(&d, true).is_err());
}

#[test]
fn section1231_history_validates_years_balances_and_duplicates_per_tax_system() {
    let good = section1231_history(&[(2021, 1_000, 0), (2025, 1_000, 700)], &[(2025, 900, 800)]);
    assert!(good.valid());
    for case in [4, 5] {
        let mut bad = good.clone();
        match case {
            4 => bad.regular[1].remaining = 100_001,
            _ => bad.regular.push(bad.regular[1].clone()),
        }
        assert!(!bad.valid());
        let mut d = complete(FederalStatus::Single);
        d.screening.as_mut().unwrap().section1231 = Some(bad);
        assert!(!d.valid_stored());
    }
}

#[test]
fn section1231_retained_history_without_current_items_is_not_a_deduction() {
    let baseline = complete(FederalStatus::Single);
    let expected = supported(&baseline);
    let mut d = baseline;
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2021, 1_000, 700), (2024, 500, 200)], &[]));
    let r = supported(&d);
    assert_eq!(r.return_lines, expected.return_lines);
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "Section 1231 regular prior-loss balances remaining eligible for 2027: $200."
    )));
    assert!(r.review_notes.iter().any(|n| {
        n.starts_with("Section 1231 AMT prior-loss balances remaining eligible for 2027: $0.")
    }));
}

#[test]
fn restricted_stock_sale_preserves_valid_83b_transfer_basis() {
    // Rev. Proc. 2012-29 examples 1 and 4: transfer FMV $25,000,
    // later vesting FMV $40,000, sale $60,000. Shift sale to 2026.
    // Neither later vesting value nor previously taxed compensation is added again.
    for paid in [0, 2500000] {
        for acquired in [20230401, 20260401] {
            let mut d = complete(FederalStatus::Single);
            let lot = EmployeeStockSale {
                acquired,
                sold: 20261001,
                paid,
                market_value: 2500000,
                fees: 0,
                proceeds: 6000000,
                wash_sale: 0,
                withholding: 0,
                reported_basis: None,
                reporting: crate::BasisReporting::ReportedToIrs,
                special: false,
            };
            assert_eq!(lot.compensation(), 2500000 - paid);
            assert_eq!(lot.sale().basis_amount(), 2500000);
            assert_eq!(
                lot.sale().term,
                if acquired < 20260000 { GainTerm::Long } else { GainTerm::Short }
            );
            add(&mut d, income(FederalIncome::EmployeeStockSale { data: lot }));
            let r = supported(&d);
            assert_eq!(value(&r, "capital_gain"), 35000);
            assert_eq!(value(&r, "agi"), 95000);
            assert_eq!(supported(&restart(d.clone()).draft), r);
            let FederalIncome::EmployeeStockSale { data } = &mut d.income[1].income else {
                unreachable!()
            };
            // An unconfirmed election/history is left out with a notice.
            data.special = true;
            assert!(noted(&supported(&d), "WARN_EMPLOYEE_STOCK_SALE_LEFT_OUT"));
        }
    }
}

#[test]
fn young_capital_partner_is_classified_for_investment_tax_readiness() {
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.parent_alive = true;
    let mut data = ordinary_partner_source(10000, true);
    data.income_source = Some(BusinessIncomeSource::Capital);
    data.qbi = Some(partner_qbi_statement(0));
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert!(
        !r.issues.iter().any(|i| i.section == "Investment tax"
            && i.message.contains("capital or mixed-source business")),
        "{:?}",
        r.issues
    );
    assert!(r.review_notes.iter().any(|n| n.contains("young-filer earned income: $0")));
    let FederalIncome::Partnership { data } = &mut d.income[0].income else { unreachable!() };
    // Not known: taken as capital, the same result with a warning.
    data.income_source = None;
    let unknown = estimate(&d);
    assert!(
        !unknown
            .issues
            .iter()
            .any(|i| i.section == "Young-partner earned income" || i.section == "Investment tax"),
        "{:?}",
        unknown.issues
    );
    assert!(noted(&unknown, "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL"));
    assert_eq!(unknown.tax, r.tax);
}

#[test]
fn qsbs_ten_times_basis_and_prior_issuer_history_are_applied_once() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, qsb_income("Synthetic A Inc.", 200_000_000, 2_700_000_000, 0));
    let result = supported(&d);
    assert_eq!(value(&result, "qsbs_2_exclusion_limit"), 20_000_000);
    assert_eq!(value(&result, "qsbs_2_taxable_gain"), 5_000_000);

    let mut with_history = complete(FederalStatus::Single);
    add(&mut with_history, qsb_income("Synthetic B Inc.", 10_000_000, 510_000_000, 800_000_000));
    let result = supported(&with_history);
    assert_eq!(value(&result, "qsbs_2_exclusion_limit"), 2_000_000);
    assert_eq!(value(&result, "qsbs_2_excluded_gain"), 2_000_000);
    assert_eq!(value(&result, "qsbs_2_taxable_gain"), 3_000_000);
}
