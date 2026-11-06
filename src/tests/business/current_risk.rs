use super::*;
use crate::tests::cross::interview::{
    assert_round_trip, business, no_expenses, save_businesses, stored_business,
};
use crate::tests::support::assert::accepted;
use crate::tests::support::client::{TEST_USER, restart};

#[test]
fn current_ordinary_risk_fixture_c_and_nonbinding_capacity() {
    // Approved6198 map fixtureC: current operating loss, actual investment room,
    // allowed operating deduction and retained current carry, all independent.
    for (loss, capacity, allowed, held) in
        [(1600, 120_000, 1200, 400), (800, 0, 0, 800), (1600, 200_000, 1600, 0)]
    {
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(current_risk_business(-loss, capacity)));
        let id = draft.income.last().unwrap().id;
        let result = supported(&draft);
        assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), -allowed);
        assert_eq!(value(&result, &alloc::format!("business_{id}_at_risk_carryover")), held);
        assert_eq!(value(&result, &alloc::format!("business_{id}_form6198_line1")), -loss);
        assert_eq!(value(&result, "agi"), 60_000 - allowed);
        assert!(!noted(&result, "WARN_BUSINESS_LOSS_LEFT_OUT"));
    }
}

#[test]
fn current_ordinary_risk_uses_independent_amt_capacity() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = current_risk_business(-1600, 120_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    let RentalAtRisk::Limited { record: Some(record), .. } = &mut data.at_risk else {
        unreachable!()
    };
    record.amt = RiskAmtMethod::Separate {
        facts: alloc::boxed::Box::new(RiskMethod::Simplified {
            facts: RiskSimplified { basis_start: 70_000, increases: 0, decreases: 0 },
        }),
    };
    add(&mut draft, income(source));
    let id = draft.income.last().unwrap().id;
    let result = supported(&draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_at_risk_allowed_ordinary")), -1200);
    assert_eq!(value(&result, &alloc::format!("amt_business_{id}_at_risk_allowed_ordinary")), -700);
    assert_eq!(value(&result, &alloc::format!("amt_business_{id}_at_risk_carryover")), 900);
    assert_eq!(value(&result, "amt_business_depreciation_adjustment"), 500);
}

#[test]
fn current_ordinary_risk_profit_and_mixed_physical_se_use_same_source() {
    let mut actual = complete(FederalStatus::Single);
    add(&mut actual, income(cash_business(20_000)));
    add(&mut actual, income(current_risk_business(-1600, 120_000)));
    let mut expected = complete(FederalStatus::Single);
    add(&mut expected, income(cash_business(20_000)));
    add(&mut expected, income(cash_business(-1200)));
    let actual = supported(&actual);
    let expected = supported(&expected);
    for key in ["agi", "self_employment_tax", "qbi_deduction", "schedule_1_business_income"] {
        assert_eq!(value(&actual, key), value(&expected, key), "{key}");
    }
    let mut profit = complete(FederalStatus::Single);
    add(&mut profit, income(current_risk_business(20_000, 0)));
    let mut full = complete(FederalStatus::Single);
    add(&mut full, income(cash_business(20_000)));
    assert_eq!(supported(&profit).tax, supported(&full).tax);
}

#[test]
fn current_ordinary_risk_native_save_reopen_restart_and_invalid_are_atomic() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = business(0);
    page.expenses = BusinessExpenses {
        other: vec![OtherExpense { description: "Operating costs".into(), amount: 160_000 }],
        ..no_expenses()
    };
    page.more.at_risk = simplified_rental_risk(120_000, 0, 0, [0; 4]).unwrap();
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    assert_eq!(stored_business(&session).1.at_risk, page.more.at_risk);
    assert_round_trip(&mut session, &[page.clone()]);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn current_ordinary_risk_missing_history_stays_a_named_source_boundary() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = current_risk_business(-1600, 120_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    let RentalAtRisk::Limited { prior_disallowed, .. } = &mut data.at_risk else { unreachable!() };
    *prior_disallowed = 10_000;
    add(&mut draft, income(source));
    let result = estimate(&draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "BUSINESS_RISK_ORIGINAL_FORM_HISTORY_REQUIRED")
    );
}

#[test]
fn current_ordinary_risk_positive_sep_keeps_its_native_owner_income() {
    let mut actual = complete(FederalStatus::Joint);
    let mut source = sep_business(20_000, 3_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.at_risk = simplified_rental_risk(0, 0, 0, [0; 4]).unwrap();
    add(&mut actual, income(source));
    let mut spouse_loss = income(current_risk_business(-1600, 120_000));
    spouse_loss.owner = crate::DocumentOwner::Spouses;
    add(&mut actual, spouse_loss);
    let mut expected = complete(FederalStatus::Joint);
    add(&mut expected, income(sep_business(20_000, 3_000)));
    let mut spouse_loss = income(cash_business(-1200));
    spouse_loss.owner = crate::DocumentOwner::Spouses;
    add(&mut expected, spouse_loss);
    let actual = supported(&actual);
    let expected = supported(&expected);
    for key in [
        "self_employment_tax",
        "schedule_1_self_employed_retirement",
        "schedule_1_half_self_employment_tax",
        "qbi_deduction",
        "agi",
    ] {
        assert_eq!(value(&actual, key), value(&expected, key), "{key}");
    }
}

#[test]
fn current_ordinary_risk_retains_cents_before_annual_rounding() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = current_risk_business(-1600, 120_049);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.expenses[0].amount = 160_099;
    add(&mut draft, income(source));
    let id = draft.income.last().unwrap().id;
    let result = supported(&draft);
    let row = result
        .lines
        .iter()
        .find(|line| line.id == alloc::format!("business_{id}_at_risk_carryover"))
        .unwrap();
    assert_eq!(row.amount, 40_050);
    assert_eq!(
        result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("business_{id}_net_profit"))
            .unwrap()
            .amount,
        -120_049
    );
    assert_eq!(value(&result, "schedule_1_business_income"), -1200);
}

#[test]
fn current_ordinary_risk_property_character_remains_a_named_dependency() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = current_risk_business(-1600, 120_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.sale = Some(BusinessSale {
        date: 20260601,
        inventory: 0,
        goodwill: 100,
        covenant: 0,
        special: false,
    });
    add(&mut draft, income(source));
    let result = estimate(&draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "BUSINESS_RISK_PROPERTY_SOURCE_REQUIRED")
    );
}

#[test]
fn current_ordinary_risk_recapture_retains_its_original_character_requirement() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = current_risk_business(-800, 0);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    let RentalAtRisk::Limited { record, .. } = &mut data.at_risk else { unreachable!() };
    *record = Some(alloc::boxed::Box::new(compare_rental_risk_record()));
    add(&mut draft, income(source));
    let result = estimate(&draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "BUSINESS_RISK_RECAPTURE_CHARACTER_REQUIRED")
    );
}

#[test]
fn current_ordinary_risk_binding_section179_retains_its_source_coupling() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = current_risk_business(-1600, 120_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    let mut asset = depreciable_asset(2000, EquipmentClass::Five, 20260101, 2000);
    asset.id = draft.next_id + 1;
    data.assets.push(asset);
    add(&mut draft, income(source));
    draft.next_id += 1; // Reserve the asset's distinct accepted identity.
    let result = estimate(&draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "BUSINESS_RISK_SECTION179_SOURCE_REQUIRED"),
        "{:?}",
        result.issues
    );
}

#[test]
fn current_ordinary_risk_divided_source_keeps_its_counterpart_prerequisite() {
    let mut draft = community_w2_draft(6_000_000, 0, 0, 0);
    add(&mut draft, income(current_risk_business(-1600, 120_000)));
    let result = estimate(&draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "BUSINESS_RISK_DIVIDED_SOURCE_REQUIRED")
    );
    assert!(result.tax.is_none());
    assert!(result.return_lines.is_empty());
}

#[test]
fn current_ordinary_risk_without_section179_keeps_unrelated_gambling_source_independent() {
    let mut actual = complete(FederalStatus::Single);
    add(&mut actual, income(current_risk_business(-1600, 120_000)));
    add(&mut actual, income(professional_gambling(5000, 2000, 500)));
    let mut expected = complete(FederalStatus::Single);
    add(&mut expected, income(cash_business(-1200)));
    add(&mut expected, income(professional_gambling(5000, 2000, 500)));
    let actual = supported(&actual);
    let expected = supported(&expected);
    for key in ["self_employment_tax", "schedule_1_business_income", "agi", "qbi_deduction"] {
        assert_eq!(value(&actual, key), value(&expected, key), "{key}");
    }
    assert_eq!(actual.tax, expected.tax);
    assert!(!actual.lines.iter().any(|line| {
        line.form_line
            .as_ref()
            .is_some_and(|line| line.form == TaxForm::Form4562 && line.line == "11")
    }));
}

#[test]
fn current_ordinary_risk_keeps_section179_coupling_when_raw_allowance_is_zero() {
    // Risk limiting the separate loss could change the income available for
    // the elected179. A raw zero allowance does not prove independence.
    let mut draft = empty_return(FederalStatus::Single);
    let mut profitable = cash_business(2000);
    let FederalIncome::Business { data } = &mut profitable else { unreachable!() };
    let mut asset = depreciable_asset(1000, EquipmentClass::Five, 20260101, 1000);
    asset.id = draft.next_id + 1;
    data.assets.push(asset);
    add(&mut draft, income(profitable));
    draft.next_id += 1;
    add(&mut draft, income(current_risk_business(-3000, 50_000)));
    let result = estimate(&draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "BUSINESS_RISK_SECTION179_SOURCE_REQUIRED"),
        "{:?}",
        result.issues
    );
    assert!(result.tax.is_none());
}

#[test]
fn current_ordinary_risk_uses_actual_zero_k1_basis_allowance_without_gambling_dependency() {
    let mut actual = complete(FederalStatus::Single);
    add(&mut actual, income(current_risk_business(-1600, 120_000)));
    add(&mut actual, income(professional_gambling(5000, 2000, 500)));
    add(
        &mut actual,
        income(FederalIncome::Scorp { data: scorp_source(basis_blocked_section179_scorp()) }),
    );
    let mut expected = complete(FederalStatus::Single);
    add(&mut expected, income(cash_business(-1200)));
    add(&mut expected, income(professional_gambling(5000, 2000, 500)));
    add(
        &mut expected,
        income(FederalIncome::Scorp { data: scorp_source(basis_blocked_section179_scorp()) }),
    );
    let actual = supported(&actual);
    let expected = supported(&expected);
    assert_eq!(actual.tax, expected.tax);
    for key in ["self_employment_tax", "agi", "qbi_deduction", "schedule_1_business_income"] {
        assert_eq!(value(&actual, key), value(&expected, key), "{key}");
    }
}

#[test]
fn current_ordinary_risk_historical_participation_without_section179_keeps_form6198() {
    let mut draft = complete(FederalStatus::Single);
    let mut business = current_risk_business(-1600, 120_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.material_participation = ParticipationFacts {
        hours: 0,
        test: ParticipationTest::FiveOfTen { years: vec![2021, 2022, 2023, 2024, 2025] },
    };
    add(&mut draft, income(business));
    let source = draft.income.last().unwrap().id;
    let result = supported(&draft);
    assert_eq!(value(&result, &alloc::format!("business_{source}_net_profit")), -1200);
    assert_eq!(value(&result, &alloc::format!("business_{source}_at_risk_carryover")), 400);
    assert!(!result.lines.iter().any(|line| {
        line.form_line
            .as_ref()
            .is_some_and(|line| line.form == TaxForm::Form4562 && line.line == "11")
    }));
}
