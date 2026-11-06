use super::*;
use crate::income_tax::{AmtTaxResult, AmtTaxWorksheet, Preference};

pub(in crate::tests) fn assert_form_6251_rates_foot(r: &FederalResult) {
    let amounts: alloc::collections::BTreeMap<_, _> = r
        .lines
        .iter()
        .filter_map(|row| {
            row.id
                .strip_prefix("amt_capital_line_")
                .map(|line| (line.parse::<u8>().unwrap(), row.amount))
        })
        .collect();
    let at = |line| *amounts.get(&line).expect("complete AMT capital trace");
    let whole = |cents: i64| crate::kernel::ratio::dollars(cents) * 100;
    let tax = |income: i64| {
        let status = r
            .lines
            .iter()
            .find(|row| row.id == "amt_capital_line_25")
            .map(|row| row.amount)
            .unwrap();
        let bracket = if status == 30685000 { 12225000 } else { 24450000 };
        // D-2026-10-19-12: Part III prints each tax line in whole dollars.
        whole((income.min(bracket) * 26 + (income - bracket).max(0) * 28) / 100)
    };
    if !amounts.is_empty() {
        for line in 12..=32 {
            assert!(amounts.contains_key(&line), "missing AMT line{line}");
        }
        for row in r.lines.iter().filter(|row| row.id.starts_with("amt_capital_line_")) {
            let number = row.id.strip_prefix("amt_capital_line_").unwrap();
            assert_eq!(row.kind, LineKind::Money);
            assert_eq!(row.form_line.as_ref().unwrap().form, TaxForm::Form6251);
            assert_eq!(row.form_line.as_ref().unwrap().line, number);
        }
        assert_eq!(at(16), at(12).min(at(15)));
        assert_eq!(at(17), at(12) - at(16));
        assert_eq!(at(18), tax(at(17)));
        assert_eq!(at(21), (at(19) - at(20)).max(0));
        assert_eq!(at(22), at(12).min(at(13).max(0)));
        assert_eq!(at(23), at(21).min(at(22)));
        assert_eq!(at(24), at(22) - at(23));
        assert_eq!(at(26), at(21));
        assert_eq!(at(28), at(26) + at(27));
        assert_eq!(at(29), (at(25) - at(28)).max(0));
        assert_eq!(at(30), at(24).min(at(29)));
        assert_eq!(at(31), whole(at(30) * 15 / 100));
        assert_eq!(at(32), at(23) + at(30));
        if at(32) == at(12) {
            for line in 33..=37 {
                assert!(!amounts.contains_key(&line));
            }
        } else {
            assert_eq!(at(33), at(22) - at(32));
            assert_eq!(at(34), whole(at(33) * 20 / 100));
            if at(14) == 0 {
                for line in 35..=37 {
                    assert!(!amounts.contains_key(&line));
                }
            } else {
                assert_eq!(at(35), at(17) + at(32) + at(33));
                assert_eq!(at(36), at(12) - at(35));
                assert_eq!(at(37), whole(at(36) * 25 / 100));
            }
        }
        assert_eq!(
            at(38),
            at(18) + at(31) + amounts.get(&34).unwrap_or(&0) + amounts.get(&37).unwrap_or(&0)
        );
        assert_eq!(at(39), tax(at(12)));
        assert_eq!(at(40), at(38).min(at(39)));
    }
    let raw = |id: &str| r.lines.iter().find(|row| row.id == id).map(|row| row.amount);
    if let Some(foreign) = raw("amt_foreign_worksheet_line_6") {
        let line = |number| raw(&alloc::format!("amt_foreign_worksheet_line_{number}")).unwrap();
        assert_eq!(line("3"), line("1") + line("2c"));
        assert_eq!(line("6"), (line("4") - line("5")).max(0));
        if !amounts.is_empty() {
            assert_eq!(at(12), line("3"));
            assert_eq!(at(40), line("4"));
        }
        if let Some(tax) = raw("amt_tax_before_foreign_credit") {
            assert_eq!(tax, foreign);
            assert_eq!(raw("amt_taxable_base"), Some(line("1")));
        }
    } else if !amounts.is_empty()
        && let Some(tax) = raw("amt_tax_before_foreign_credit")
    {
        assert_eq!(tax, at(40));
        assert_eq!(raw("amt_taxable_base"), Some(at(12)));
    }
}

fn compute(
    base: i64,
    preferential: i64,
    ordinary: i64,
    special: [i64; 2],
    excluded: i64,
    foreign: Option<Preference>,
) -> AmtTaxResult {
    AmtTaxWorksheet {
        year: 2026,
        status: FederalStatus::Single,
        base,
        preferential,
        regular_ordinary: ordinary,
        excluded,
        special,
        foreign,
    }
    .calculate()
}

fn projected(computation: &AmtTaxResult) -> FederalResult {
    let mut r = FederalResult::empty(1);
    crate::income_tax::projection::emit(&mut r, computation, true);
    assert_form_6251_rates_foot(&r);
    r
}

#[test]
fn print_amt_all_capital_bands_keep_the_actual_regular_occupancy() {
    // AMTI800000 exhausts the exemption. Gain650000, ordinary150000;
    // regular ordinary30000 fills the zero-rate band first: free19450,
    // 15%496050,20%134500. Tax39000+74407.50+26900=140307.50 ->140308.
    let c = compute(800000, 650000, 30000, [0, 0], 0, None);
    assert_eq!(c.tax, 140308);
    let r = projected(&c);
    for (line, cents) in [
        (12, 80000000),
        (17, 15000000),
        (18, 3900000),
        (20, 3000000),
        (23, 1945000),
        (30, 49605000),
        (31, 7440800),
        (33, 13450000),
        (34, 2690000),
        (38, 14030800),
        (39, 21911000),
        (40, 14030800),
    ] {
        assert_eq!(line_amount(&r, &alloc::format!("amt_capital_line_{line}")), cents);
    }
}

#[test]
fn print_amt_unrecaptured_gain_retains_the_actual_twenty_five_percent_component() {
    // Taxable AMT309900, normal gain100000,1250 pool50000. Ordinary159900
    // at26%=41574; normal gain15000 and1250 gain12500 ->69074.
    let c = compute(400000, 100000, 200000, [20000, 50000], 0, None);
    assert_eq!(c.tax, 69074);
    let r = projected(&c);
    assert_eq!(value(&r, "amt_capital_line_17"), 159900);
    assert_eq!(value(&r, "amt_capital_line_36"), 50000);
    assert_eq!(value(&r, "amt_capital_line_37"), 12500);
    assert_eq!(value(&r, "amt_capital_line_40"), 69074);
}

#[test]
fn print_amt_rate_cents_round_once_and_zero_rate_skips_unused_lines() {
    let c = compute(90104, 3, 60000, [0, 0], 0, None);
    // 2026-10-19 (D-2026-10-19-12): the tax lines print whole dollars and line 40 is the smaller
    // of the printed lines 38 and 39. 26 cents ordinary plus 45 cents gain print 0 + 0, so the
    // tax is 0 (it was one rounding of 71 cents, 1 dollar).
    assert_eq!(c.tax, 0);
    let r = projected(&c);
    for (line, cents) in [(18, 0), (31, 0), (38, 0), (39, 100), (40, 0)] {
        assert_eq!(line_amount(&r, &alloc::format!("amt_capital_line_{line}")), cents);
    }
    let free = compute(90104, 4, 0, [0, 0], 0, None);
    assert_eq!(free.tax, 0);
    assert!(!projected(&free).lines.iter().any(|row| row.id == "amt_capital_line_33"));
    let mut withheld = FederalResult::empty(1);
    crate::income_tax::projection::emit(&mut withheld, &free, false);
    assert!(withheld.lines.is_empty());
}

#[test]
fn print_amt_foreign_stack_preserves_the_capped_source_gain_and_final_subtraction() {
    //AMT taxable109900; excluded50000 adds to159900. Gain20000 at15%,
    // ordinary139900 at26%:39374, less13000 excluded tax ->26374.
    let c = compute(
        200000,
        20000,
        80000,
        [0, 0],
        50000,
        Some(Preference { dividends: 0, gain: 20000, line18: 0, line19: 0 }),
    );
    assert_eq!(c.tax, 26374);
    assert_eq!(c.foreign, Some([10990000, 5000000, 15990000, 3937400, 1300000, 2637400]));
    projected(&c);
    //AMT capital excess clips the source gain to109900 before stacking.
    let excess = compute(
        200000,
        200000,
        0,
        [0, 0],
        50000,
        Some(Preference { dividends: 0, gain: 200000, line18: 0, line19: 0 }),
    );
    let r = projected(&excess);
    assert_eq!(value(&r, "amt_capital_line_13"), 109900);
    assert_eq!(value(&r, "amt_capital_line_12"), 159900);
}

#[test]
fn print_amt_actual_return_rate_rows_match_save_restart_and_final_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 20000000, 0));
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(native_dividend(2000000, 2000000, 0)),
        }),
    );
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic rate trace ISO", 1000.0, 10000, 20000)];
    let r = supported(&d);
    //AMTI320000-exemption90100=229900. Ordinary209900*26%=54574,
    // dividends20000*15%=3000 ->57574; regular39734 givesAMT17840.
    assert_eq!(value(&r, "amt_capital_line_12"), 229900);
    assert_eq!(value(&r, "amt_capital_line_18"), 54574);
    assert_eq!(value(&r, "amt_capital_line_31"), 3000);
    assert_eq!(value(&r, "amt_capital_line_40"), 57574);
    assert_eq!(value(&r, "regular_income_tax"), 39734);
    assert_eq!(value(&r, "alternative_minimum_tax"), 17840);
    assert_eq!(restart(d.clone()).draft, d);
}

// D-2026-10-19-12: Part III foots on its printed lines. Ordinary AMT income 2 taxes at 26% to
// 0.52 (prints 1) and 10 of gain at 15% to 1.50 (prints 2): line 38 is 3 and the tax 3, where
// the exact sum 2.02 used to print and carry 2.
#[test]
fn amt_part_three_adds_the_printed_tax_lines() {
    let c = compute(90112, 10, 60000, [0, 0], 0, None);
    let r = projected(&c);
    let at = |n: u8| line_amount(&r, &alloc::format!("amt_capital_line_{n}"));
    assert_eq!((at(18), at(31)), (100, 200));
    assert_eq!(at(38), at(18) + at(31));
    assert_eq!(at(40), at(38).min(at(39)));
    assert_eq!(c.tax, 3);
}
