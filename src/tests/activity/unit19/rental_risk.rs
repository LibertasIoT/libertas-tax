use super::*;

#[test]
fn form4835_property_rows_and_qbi_use_the_same_regular_and_amt_risk_source() {
    let draft = form4835_risk_property_return();
    let entry =
        draft.income.iter().find(|e| matches!(e.income, FederalIncome::FarmRental { .. })).unwrap();
    let id = entry.id;
    // Operating income exactly pays native depreciation. Ordinary property
    // losses $1,000+$2,000 and section1231 losses $28,582+$5,000 are each
    // allowed half by $18,291 risk. AMT's land loss is $3,000, risk $17,291.
    // The partial's own half is $14,291 in BOTH systems; it cannot absorb the
    // other land/asset losses' entire cut. Qualified loss is $18,291 total.
    let result = supported(&draft);
    for (line, expected) in [
        (alloc::format!("rental_{id}_at_risk_allowed_ordinary_property"), -1_500),
        (alloc::format!("rental_{id}_at_risk_allowed_section1231"), -16_791),
        (alloc::format!("amt_rental_{id}_at_risk_allowed_ordinary_property"), -1_500),
        (alloc::format!("amt_rental_{id}_at_risk_allowed_section1231"), -15_791),
        (alloc::format!("business_{id}_asset_0_partial_gain"), -14_291),
        (alloc::format!("business_{id}_asset_1_disposal_ordinary"), -500),
        (alloc::format!("farm_{id}_land_0_section1231"), -2_500),
        (alloc::format!("farm_{id}_land_1_ordinary"), -1_000),
        ("form4797_line1c".into(), 14_291),
        ("form4797_line8".into(), -16_791),
        ("qbi_loss_carryforward".into(), 18_291),
        ("agi".into(), 181_709),
    ] {
        assert_eq!(value(&result, &line), expected, "{line}");
    }
    // Actual final character rows foot to the native source class totals.
    for (line, expected) in [("2", -16_791), ("11", -1_500)] {
        let detail: i64 = result
            .lines
            .iter()
            .filter(|row| {
                row.form_line
                    .as_ref()
                    .is_some_and(|form| form.form == TaxForm::Form4797 && form.line == line)
            })
            .map(|row| row.amount)
            .sum();
        assert_eq!(detail, expected * 100, "Form4797 line{line}");
    }
    for (amt, ordinary, property) in [(false, -1_500, -16_791), (true, -1_500, -15_791)] {
        let mut r = FederalResult::empty(draft.revision);
        let pool = crate::extended_rules::rentals_for(&draft, 200_000, &mut r, amt);
        assert!(r.issues.is_empty(), "{:?}", r.issues);
        assert_eq!(pool.farm_rental_property[&id].ordinary, ordinary);
        assert_eq!(pool.farm_rental_property[&id].section1231, property);
        assert_eq!(pool.farm_rental_property_rows[&id].partial_gains, vec![(0, -14_291)]);
    }
    let accepted =
        save_reopen_native_sources(draft.clone(), &[TaxSection::Farming], &[TaxSection::Farming]);
    assert_eq!(accepted.income, draft.income);
    assert_eq!(supported(&accepted).return_lines, result.return_lines);
}

#[test]
fn compare_method_keeps_the_actual_detailed_closing_balance_and_history_boundary() {
    use crate::activity_risk::{Part1Line, ProjectionIssue, project_record};
    let mut record = compare_rental_risk_record();
    let input = [(Part1Line::Operating, -50_000), (Part1Line::Property, 20_000)];
    // Compare permits $200 rather than detailed capacity zero. Actual detailed
    // investment is still -$300; the allowed net -$200 leaves a $500 deficit.
    for amt in [false, true] {
        let projection =
            project_record(&input, &record, &RiskEffectiveCategory::HoldingRealProperty, amt)
                .unwrap();
        assert_eq!(projection.components.allowed, vec![-40_000, 20_000]);
        assert_eq!(projection.components.suspended, vec![10_000, 0]);
        assert_eq!(projection.recapture, Ok(50_000));
        assert!(projection.lines.contains(&("19a", -30_000)));
        assert!(projection.lines.contains(&("20", 20_000)));
    }
    record.recapture = None;
    let unknown =
        project_record(&input, &record, &RiskEffectiveCategory::HoldingRealProperty, false)
            .unwrap();
    assert_eq!(unknown.recapture, Err(ProjectionIssue::HistoryRequired));
    let profit = project_record(
        &[(Part1Line::Operating, 40_000)],
        &record,
        &RiskEffectiveCategory::HoldingRealProperty,
        false,
    )
    .unwrap();
    assert_eq!(
        profit.recapture,
        Ok(0),
        "known profit cures the actual deficit without guessed history"
    );
}

#[test]
fn unused_prior6198_is_known_but_released_or_ambiguous_qualification_stays_unavailable() {
    for (operating, property) in [(100_000, 0), (0, 100_000)] {
        let unused = form4835_risk_prior_return(0, operating, property, false);
        let known = supported(&unused);
        assert_eq!(value(&known, "agi"), 200_000);
        assert_eq!(value(&known, "qbi_loss_carryforward"), 0);
        let released = form4835_risk_prior_return(5_000_000, operating, property, false);
        let before = released.clone();
        let r = estimate(&released);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(
            r.issues.iter().any(|issue| issue.display.key == "ACTIVITY_QBI_HISTORY_UNAVAILABLE")
        );
        assert!(r.tax.is_none());
        assert!(r.return_lines.is_empty());
        assert_eq!(released, before);
    }
    // Current native operating loss $9,457.96 and prior $1,000 share an
    // actual partial risk allowance. Their original qualified split is not
    // supplied and is not recovered from final operating/PAL totals.
    let ambiguous = form4835_risk_prior_return(1_000_000, 100_000, 0, true);
    let r = estimate(&ambiguous);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|issue| issue.display.key == "ACTIVITY_QBI_HISTORY_UNAVAILABLE"));
    assert!(r.tax.is_none());
}
