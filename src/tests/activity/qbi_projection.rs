use super::*;
use crate::business_rules::{QbiActivity, QbiFiling, QbiOther, QbiResult, qbi_filed};

pub(in crate::tests) fn assert_form_8995_foots(r: &FederalResult) {
    let Some(form) = r.lines.iter().find_map(|row| {
        row.form_line
            .as_ref()
            .filter(|line| matches!(line.form, TaxForm::Form8995 | TaxForm::Form8995A))
            .map(|line| line.form)
    }) else {
        return;
    };
    let at = |number: &str| -> i64 {
        r.lines
            .iter()
            .filter(|row| {
                row.kind == LineKind::Money
                    && row.form_line.as_ref().is_some_and(|line| {
                        line.form == form
                            && line.kind == crate::FormLineKind::Line
                            && line.line == number
                    })
            })
            .map(|row| row.amount / 100)
            .sum()
    };
    let fraction = crate::kernel::ratio::fraction;
    if form == TaxForm::Form8995 {
        assert_eq!(at("1"), at("2"));
        assert_eq!((at("2") + at("3")).max(0), at("4"));
        assert_eq!(fraction(at("4"), 20, 100), at("5"));
        assert_eq!((at("6") + at("7")).max(0), at("8"));
        assert_eq!(fraction(at("8"), 20, 100), at("9"));
        assert_eq!(at("5") + at("9"), at("10"));
        assert_eq!((at("11") - at("12")).max(0), at("13"));
        assert_eq!(fraction(at("13"), 20, 100), at("14"));
        assert_eq!(at("10").min(at("14")), at("15"));
        assert_eq!(at("15").max(at("16")), at("17"));
    } else {
        assert_eq!(at("15"), at("16"));
        assert_eq!(at("16"), at("27"));
        assert_eq!((at("28") + at("29")).max(0), at("30"));
        assert_eq!(fraction(at("30"), 20, 100), at("31"));
        assert_eq!(at("27") + at("31"), at("32"));
        assert_eq!((at("33") - at("34")).max(0), at("35"));
        assert_eq!(fraction(at("35"), 20, 100), at("36"));
        assert_eq!(at("32").min(at("36")), at("37"));
        assert_eq!(at("37") + at("38"), at("39"));
        assert_eq!(at("39").max(at("40")), at("41"));
        let raw = |id: &str| r.lines.iter().find(|row| row.id == id).map(|row| row.amount);
        for row in r
            .lines
            .iter()
            .filter(|row| row.id.starts_with("qbi_column_") && row.id.ends_with("_line_26"))
        {
            let prefix = row.id.strip_suffix("26").unwrap();
            let cell = |number: &str| raw(&alloc::format!("{prefix}{number}")).unwrap() / 100;
            assert_eq!(cell("17") - cell("18"), cell("19"));
            assert_eq!(cell("20") - cell("21"), cell("22"));
            assert_eq!(fraction(cell("19"), cell("22"), cell("23")), cell("25"));
            assert_eq!(cell("17") - cell("25"), cell("26"));
            assert_eq!(cell("26"), cell("12"));
            assert_eq!(
                fraction(cell("22"), 10000, cell("23")),
                raw(&alloc::format!("{prefix}24")).unwrap()
            );
        }
        for row in r
            .lines
            .iter()
            .filter(|row| row.id.starts_with("qbi_aggregation_") && row.id.contains("_total_"))
        {
            let (prefix, axis) = row.id.split_once("_total_").unwrap();
            let member_prefix = alloc::format!("{prefix}_member_");
            let suffix = alloc::format!("_{axis}");
            let sum: i64 = r
                .lines
                .iter()
                .filter(|part| part.id.starts_with(&member_prefix) && part.id.ends_with(&suffix))
                .map(|part| part.amount)
                .sum();
            assert_eq!(sum, row.amount, "actual aggregation members {}", row.id);
        }
        if let Some(losses) = raw("qbi_loss_line_3") {
            let cells = |suffix: &str| -> Vec<i64> {
                r.lines
                    .iter()
                    .filter(|row| {
                        row.id.starts_with("qbi_loss_column_") && row.id.ends_with(suffix)
                    })
                    .map(|row| row.amount / 100)
                    .collect()
            };
            let income = cells("_income");
            let reductions = cells("_reduction");
            let adjusted = cells("_adjusted");
            assert_eq!(income.len(), reductions.len());
            assert_eq!(income.len(), adjusted.len());
            for ((income, loss), net) in income.iter().zip(&reductions).zip(&adjusted) {
                assert_eq!((income + loss).max(0), *net);
            }
            assert_eq!(
                income.iter().filter(|v| **v < 0).sum::<i64>() + at("Schedule C 2"),
                losses / 100
            );
            assert_eq!(income.iter().map(|v| (*v).max(0)).sum::<i64>(), at("Schedule C 4"));
            assert_eq!(-at("Schedule C 3").abs().min(at("Schedule C 4")), at("Schedule C 5"));
            assert_eq!(reductions.iter().sum::<i64>(), at("Schedule C 5"));
            assert_eq!((-at("Schedule C 3") + at("Schedule C 5")).max(0), at("Schedule C 6"));
            assert_eq!(adjusted.iter().sum::<i64>(), at("2"));
        }
    }
    let deduction = at(if form == TaxForm::Form8995 { "17" } else { "41" });
    assert_eq!(deduction * 100, return_line(r, "13b"));
}

fn activity(income: i64, wages: i64, group: Option<&str>) -> QbiActivity {
    QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income,
        wages,
        property: 0,
        active: true,
        service: Some(false),
        group: group.map(Into::into),
    }
}

fn projected(activities: &[QbiActivity], prior: i64) -> (QbiResult, FederalResult) {
    let mut r = FederalResult::empty(1);
    let result = qbi_filed(
        activities,
        400000,
        0,
        FederalStatus::Single,
        QbiOther { business_loss: prior, ..Default::default() },
        &QbiFiling::default(),
        &mut r,
    );
    crate::business_rules::projection::emit(&mut r, TaxForm::Form8995A, &result.lines);
    (result, r)
}

#[test]
fn print_qbi_phase_in_columns_reach_the_actual_return_without_rounded_percentage_tax() {
    // W2 $55,350 + ordinary S-corp $200,000 - standard $16,100 gives
    // before-QBI taxable $239,250, exactly halfway through the $75,000 range.
    // 20%=$40,000; wage/property max=max(15,000,7,500+2,500)=$15,000.
    // Reduction=(40,000-15,000)/2=$12,500; deduction=$27,500.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 5535000, 0);
    let mut source = complete_ordinary_scorp(200000, true);
    let statement = source.qbi_statement.as_mut().unwrap();
    statement.wages = 3000000;
    statement.property = 10000000;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    let r = supported(&d);
    for (number, expected) in [
        ("4", 30000),
        ("5", 15000),
        ("6", 7500),
        ("7", 100000),
        ("8", 2500),
        ("9", 10000),
        ("10", 15000),
        ("11", 15000),
        ("12", 27500),
        ("13", 27500),
        ("16", 27500),
        ("17", 40000),
        ("18", 15000),
        ("19", 25000),
        ("20", 239250),
        ("21", 201750),
        ("22", 37500),
        ("23", 75000),
        ("25", 12500),
        ("26", 27500),
    ] {
        let id = if number == "16" {
            "qbi_columns_total".into()
        } else {
            alloc::format!("qbi_column_1_line_{number}")
        };
        assert_eq!(value(&r, &id), expected, "line{number}");
    }
    let percentage = r.lines.iter().find(|row| row.id == "qbi_column_1_line_24").unwrap();
    assert_eq!(percentage.kind, LineKind::Percentage);
    assert_eq!(percentage.amount, 5000);
    assert_eq!(value(&r, "qbi_deduction"), 27500);
    assert_eq!(return_line(&r, "13b"), 2750000);
    assert_eq!(value(&r, "taxable_income"), 211750);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn print_qbi_aggregation_preserves_members_before_the_actual_combined_limit() {
    let (computed, r) =
        projected(&[activity(200000, 0, Some("Shops")), activity(50000, 120000, Some("Shops"))], 0);
    assert_eq!(computed.deduction, 50000);
    assert_eq!(computed.lines.members, vec![vec![0, 1]]);
    for (id, expected) in [
        ("qbi_aggregation_1_member_1_income", 200000),
        ("qbi_aggregation_1_member_2_income", 50000),
        ("qbi_aggregation_1_member_2_wages", 120000),
        ("qbi_aggregation_1_total_income", 250000),
        ("qbi_aggregation_1_total_wages", 120000),
        ("qbi_column_1_line_2", 250000),
        ("qbi_column_1_line_3", 50000),
        ("qbi_column_1_line_5", 60000),
        ("qbi_column_1_line_11", 50000),
        ("qbi_column_1_line_13", 50000),
        ("qbi_column_1_line_15", 50000),
        ("qbi_columns_total", 50000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert!(!r.lines.iter().any(|row| row.id == "qbi_column_1_line_24"));
    let review = crate::draft_return(&r);
    assert!(review.iter().any(|row| row.form_line.as_ref().is_some_and(|line| line.kind
        == crate::FormLineKind::Subrow
        && line.line == "Schedule B 1 3(c)")
        && row.amount == 20000000));
}

#[test]
fn print_qbi_group_loss_then_cross_business_loss_keeps_each_original_member() {
    // The $50,000 group member loss nets inside Shops before the $30,000
    // carry is shared18/12 across its $150,000 and the other $100,000 profit.
    let (computed, r) = projected(
        &[
            activity(200000, 0, Some("Shops")),
            activity(-50000, 120000, Some("Shops")),
            activity(100000, 80000, None),
        ],
        30000,
    );
    assert_eq!((computed.deduction, computed.business_loss), (44000, 0));
    for (id, expected) in [
        ("qbi_aggregation_1_member_2_income", -50000),
        ("qbi_aggregation_1_total_income", 150000),
        ("qbi_loss_column_1_income", 150000),
        ("qbi_loss_column_1_reduction", -18000),
        ("qbi_loss_column_1_adjusted", 132000),
        ("qbi_loss_column_2_income", 100000),
        ("qbi_loss_column_2_reduction", -12000),
        ("qbi_loss_column_2_adjusted", 88000),
        ("qbi_loss_line_2", -30000),
        ("qbi_loss_line_3", -30000),
        ("qbi_loss_line_4", 250000),
        ("qbi_loss_line_5", -30000),
        ("qbi_column_1_line_15", 26400),
        ("qbi_column_2_line_15", 17600),
        ("qbi_columns_total", 44000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
}

#[test]
fn print_qbi_excess_loss_retains_zero_allowed_columns_and_original_carry_owner() {
    let (computed, r) = projected(&[activity(10000, 20000, None), activity(-30000, 0, None)], 5000);
    assert_eq!((computed.deduction, computed.business_loss), (0, 25000));
    for (id, expected) in [
        ("qbi_loss_line_2", -5000),
        ("qbi_loss_line_3", -35000),
        ("qbi_loss_line_4", 10000),
        ("qbi_loss_line_5", -10000),
        ("qbi_loss_column_1_reduction", -10000),
        ("qbi_loss_column_1_adjusted", 0),
        ("qbi_loss_column_2_income", -30000),
        ("qbi_columns_total", 0),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert!(
        !crate::draft_return(&r)
            .iter()
            .any(|row| row.form_line == Some(FormLine::new(TaxForm::Form8995A, "16")))
    );
}

#[test]
fn source_qbi_wages_and_property_apply_eligibility_before_reporting() {
    let mut source = activity(100, 1, None);
    source.property = 1;
    source.service = Some(true);
    source.source_income_cents = Some(10_000);
    source.source_wages_cents = Some(50);
    source.source_property_cents = Some(50);
    let eligible =
        crate::business_rules::eligible_qbi_activity(&source, 239_250, FederalStatus::Single)
            .unwrap();
    assert_eq!(
        (eligible.source_income_cents, eligible.source_wages_cents, eligible.source_property_cents),
        (Some(5_000), Some(25), Some(25))
    );
    assert_eq!((eligible.income, eligible.wages, eligible.property), (50, 0, 0));
}
