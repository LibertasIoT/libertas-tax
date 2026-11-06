use super::*;
use crate::tests::cross::interview::{
    assert_round_trip, business, no_expenses, open_topic, page_documents, save_businesses,
    save_topic, stored_business,
};

#[test]
fn passive_current_profit_retains_schedule_c_se_and_qbi_without_schedule_e_duplication() {
    let mut passive = complete(FederalStatus::Single);
    add(&mut passive, income(passive_business(20_000)));
    let id = passive.income.last().unwrap().id;
    let mut active = complete(FederalStatus::Single);
    add(&mut active, income(cash_business(20_000)));
    let result = supported(&passive);
    let expected = supported(&active);
    for key in ["agi", "self_employment_tax", "qbi_deduction", "schedule_1_business_income"] {
        assert_eq!(value(&result, key), value(&expected, key), "{key}");
    }
    assert_eq!(value(&result, "schedule_1_business_income"), 20_000);
    assert!(!result.lines.iter().any(|line| line.id == "schedule_1_rental_income"));
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_allowed")), 20_000);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_suspended")), 0);
    assert_eq!(value(&result, "form8582_line_2a"), 20_000);
    assert!(!noted(&result, "WARN_BUSINESS_LEFT_OUT"));
}

#[test]
fn passive_current_loss_is_suspended_before_se_qbi_and_excess_business_loss() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(passive_business(-400_000)));
    let id = draft.income.last().unwrap().id;
    let result = supported(&draft);
    for key in ["self_employment_tax", "qbi_loss_carryforward", "excess_business_loss"] {
        assert_eq!(value(&result, key), 0, "{key}");
    }
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_current")), -400_000);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_suspended")), 400_000);
    assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), 0);
    assert!(!result.lines.iter().any(|line| matches!(
        line.id.as_str(),
        "schedule_1_business_income" | "schedule_1_rental_income"
    )));
    assert_eq!(value(&result, &alloc::format!("business_{id}_total_expenses")), 400_000);
    assert_eq!(value(&result, &alloc::format!("business_{id}_tentative_profit")), -400_000);
}

#[test]
fn passive_current_cross_business_release_keeps_actual_cents_and_source_identity() {
    let mut draft = complete(FederalStatus::Single);
    let mut gain = passive_business(2000);
    let FederalIncome::Business { data } = &mut gain else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts = 200_049;
    add(&mut draft, income(gain));
    let gain_id = draft.income.last().unwrap().id;
    let mut loss = passive_business(-4000);
    let FederalIncome::Business { data } = &mut loss else { unreachable!() };
    data.expenses[0].amount = 400_099;
    add(&mut draft, income(loss));
    let loss_id = draft.income.last().unwrap().id;
    let accepted = draft.clone();
    let result = supported(&draft);
    for (id, suffix, cents) in [
        (gain_id, "passive_allowed", 200_049),
        (loss_id, "passive_allowed", -200_049),
        (loss_id, "passive_suspended", 200_050),
    ] {
        assert_eq!(
            result
                .lines
                .iter()
                .find(|line| { line.id == alloc::format!("business_{id}_{suffix}") })
                .unwrap()
                .amount,
            cents
        );
    }
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    assert!(!result.lines.iter().any(|line| line.id == "schedule_1_business_income"));
    assert!(!result.lines.iter().any(|line| line.id == "schedule_1_rental_income"));
    assert_eq!(draft, accepted);
}

#[test]
fn passive_current_risk_precedes_pal_and_amt_uses_its_own_capacity() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(passive_business(2000)));
    let mut loss = passive_business(-5000);
    let FederalIncome::Business { data } = &mut loss else { unreachable!() };
    data.at_risk = simplified_rental_risk(300_000, 0, 0, [0; 4]).unwrap();
    let RentalAtRisk::Limited { record: Some(record), .. } = &mut data.at_risk else {
        unreachable!()
    };
    record.amt = RiskAmtMethod::Separate {
        facts: alloc::boxed::Box::new(RiskMethod::Simplified {
            facts: RiskSimplified { basis_start: 100_000, increases: 0, decreases: 0 },
        }),
    };
    add(&mut draft, income(loss));
    let id = draft.income.last().unwrap().id;
    let result = supported(&draft);
    for (prefix, risk, risk_carry, allowed, passive_carry) in
        [("", -3000, 2000, -2000, 1000), ("amt_", -1000, 4000, -1000, 0)]
    {
        for (suffix, expected) in [
            ("at_risk_allowed_ordinary", risk),
            ("at_risk_carryover", risk_carry),
            ("passive_current", risk),
            ("passive_allowed", allowed),
            ("passive_suspended", passive_carry),
        ] {
            assert_eq!(value(&result, &alloc::format!("{prefix}business_{id}_{suffix}")), expected);
        }
    }
    assert_eq!(value(&result, "amt_business_depreciation_adjustment"), 1000);
    assert_eq!(value(&result, "agi"), 60_000);
    assert!(!noted(&result, "WARN_BUSINESS_LOSS_LEFT_OUT"));
}

#[test]
fn passive_current_profit_releases_native_rental_loss_without_moving_c_to_e() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(passive_business(2000)));
    // The native building's $10,000 depreciation plus $5,000 operating
    // expenses against $10,000 rent establishes the current $5,000 loss.
    let mut rental = rental(10_000, 5000);
    rental.participation = RentalParticipation::NotActive;
    add(&mut draft, income(FederalIncome::Rental { data: rental }));
    let id = draft.income.last().unwrap().id;
    let result = supported(&draft);
    assert_eq!(value(&result, "schedule_1_business_income"), 2000);
    assert_eq!(value(&result, "schedule_1_rental_income"), -2000);
    assert_eq!(value(&result, &alloc::format!("rental_{id}_suspended_loss")), 3000);
    assert_eq!(value(&result, "self_employment_tax"), 283);
    assert_eq!(value(&result, "agi"), 59_858);
}

#[test]
fn passive_current_allowed_source_feeds_native_health_and_retirement_owners() {
    let mut passive = complete(FederalStatus::Joint);
    let mut plan = sep_business(20_000, 3000);
    let FederalIncome::Business { data } = &mut plan else { unreachable!() };
    let FederalIncome::Business { data: passive_facts } = passive_business(0) else {
        unreachable!()
    };
    data.material_participation = passive_facts.material_participation;
    add(&mut passive, income(plan));
    let mut loss = income(passive_business(-30_000));
    loss.owner = crate::DocumentOwner::Spouses;
    add(&mut passive, loss);
    let mut expected = complete(FederalStatus::Joint);
    add(&mut expected, income(sep_business(20_000, 3000)));
    let mut loss = income(cash_business(-20_000));
    loss.owner = crate::DocumentOwner::Spouses;
    add(&mut expected, loss);
    let result = supported(&passive);
    let control = supported(&expected);
    for key in [
        "self_employment_tax",
        "schedule_1_half_self_employment_tax",
        "schedule_1_self_employed_retirement",
        "agi",
        "qbi_deduction",
    ] {
        assert_eq!(value(&result, key), value(&control, key), "{key}");
    }
    let mut insured = complete(FederalStatus::Single);
    let mut health = health_business(20_000, 3000, false);
    let FederalIncome::Business { data } = &mut health else { unreachable!() };
    let FederalIncome::Business { data: passive_facts } = passive_business(0) else {
        unreachable!()
    };
    data.material_participation = passive_facts.material_participation;
    add(&mut insured, income(health));
    add(&mut insured, income(passive_business(-30_000)));
    let result = supported(&insured);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 3000);
    assert_eq!(value(&result, "agi"), 57_000);
}

#[test]
fn passive_current_native_save_reopen_restart_correction_and_deletion_recompute() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = business(0);
    page.more.material_participation =
        ParticipationFacts { hours: 0, test: ParticipationTest::None };
    page.expenses = BusinessExpenses {
        other: vec![OtherExpense { description: "Operating costs".into(), amount: 160_000 }],
        ..no_expenses()
    };
    page.more.at_risk = simplified_rental_risk(120_000, 0, 0, [0; 4]).unwrap();
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let id = stored_business(&session).0;
    assert_round_trip(&mut session, &[page]);
    session = restart(session.draft.clone());
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_suspended")), 1200);
    assert_eq!(value(&result, "agi"), 60_000);
    let mut documents =
        page_documents(&open_topic(&mut session, TaxSection::SelfEmployment)).to_vec();
    let IncomeSource::Record { value: crate::IncomeRecord::Business(page) } =
        &mut documents[0].document
    else {
        unreachable!()
    };
    page.receipts = 360_000;
    let request = save_topic(&mut session, TaxSection::SelfEmployment, documents);
    accepted(&send(&mut session, request));
    assert_eq!(stored_business(&session).0, id);
    session = restart(session.draft.clone());
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_allowed")), 2000);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_suspended")), 0);
    assert_eq!(value(&result, "self_employment_tax"), 283);
    let request = save_topic(&mut session, TaxSection::SelfEmployment, vec![]);
    accepted(&send(&mut session, request));
    session = restart(session.draft.clone());
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 60_000);
    assert!(
        !result.lines.iter().any(|line| line.id.starts_with(&alloc::format!("business_{id}_")))
    );
}

#[test]
fn passive_current_retains_prior_property_and_binding_179_source_boundaries() {
    for (kind, key) in [
        (0, "BUSINESS_RISK_ORIGINAL_FORM_HISTORY_REQUIRED"),
        (1, "BUSINESS_RISK_PROPERTY_SOURCE_REQUIRED"),
        // 2026-10-24: retain the native current-conduct source refusal.
        (2, "ISSUE_SECTION179_DIRECT_CURRENT_WORK"),
    ] {
        let mut draft = complete(FederalStatus::Single);
        let mut source = passive_business(-1600);
        let FederalIncome::Business { data } = &mut source else { unreachable!() };
        data.at_risk = simplified_rental_risk(120_000, 0, 0, [0; 4]).unwrap();
        match kind {
            0 => {
                let RentalAtRisk::Limited { prior_disallowed, .. } = &mut data.at_risk else {
                    unreachable!()
                };
                *prior_disallowed = 10_000;
            }
            1 => {
                data.sale = Some(BusinessSale {
                    date: 20260601,
                    inventory: 0,
                    goodwill: 100,
                    covenant: 0,
                    special: false,
                })
            }
            _ => {
                let mut asset = depreciable_asset(2000, EquipmentClass::Five, 20260101, 2000);
                asset.id = draft.next_id + 1;
                data.assets.push(asset);
            }
        }
        add(&mut draft, income(source));
        if kind == 2 {
            draft.next_id += 1;
        }
        let result = estimate(&draft);
        assert!(result.issues.iter().any(|issue| issue.display.key == key), "{:?}", result.issues);
        assert!(result.tax.is_none());
    }
}

#[test]
fn passive_current_scope_no_still_omits_only_that_business_without_changing_answers() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = passive_business(20_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.special_treatment = true;
    add(&mut draft, income(source));
    let accepted = draft.clone();
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 60_000);
    assert!(noted(&result, "WARN_BUSINESS_LEFT_OUT"));
    assert_eq!(draft, accepted);
}

#[test]
fn passive_current_se_income_stays_out_of_niit_above_its_threshold() {
    // Section1411(c)(6) excludes income taken into account for1401 even when
    // the trade is passive under469;1.469-1T(d)(3) still limits its losses.
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 26_000_000, 0);
    add(&mut draft, income(passive_business(20_000)));
    let result = supported(&draft);
    assert!(value(&result, "self_employment_tax") > 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 0);
    assert_eq!(value(&result, "net_investment_income_tax"), 0);
}

#[test]
fn passive_current_zero_risk_is_known_without_becoming_a_passive_loss() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(passive_business(2000)));
    let mut source = passive_business(-1600);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.at_risk = simplified_rental_risk(0, 0, 0, [0; 4]).unwrap();
    add(&mut draft, income(source));
    let id = draft.income.last().unwrap().id;
    let result = supported(&draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_at_risk_carryover")), 1600);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_current")), 0);
    assert_eq!(value(&result, &alloc::format!("business_{id}_passive_suspended")), 0);
    assert_eq!(value(&result, "schedule_1_business_income"), 2000);
    assert_eq!(value(&result, "self_employment_tax"), 283);
}

#[test]
fn passive_current_notary_exclusion_retains_niit_character() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 26_000_000, 0);
    let mut source = passive_business(20_000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.notary_profit = 2_000_000;
    add(&mut draft, income(source));
    let result = supported(&draft);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 20_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 760);
}

#[test]
fn passive_current_statutory_wages_keep_wage_character_and_cannot_release_pal() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(passive_business(0)));
    let id = draft.income.last().unwrap().id;
    source_statutory_wage(&mut draft, Owner::Taxpayer, Some(id));
    add(&mut draft, income(passive_business(-5000)));
    let loss = draft.income.last().unwrap().id;
    let result = supported(&draft);
    assert_eq!(value(&result, &alloc::format!("business_{loss}_passive_suspended")), 5000);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "agi"), 60_000);
    assert!(
        !result
            .lines
            .iter()
            .any(|line| line.form_line.as_ref().is_some_and(|form| form.form == TaxForm::Form8960))
    );
    assert!(
        !result.lines.iter().any(|line| line.id == alloc::format!("business_{id}_passive_allowed"))
    );
}

#[test]
fn passive_ministry_keeps_native_housing_correction_out_of_se_and_niit() {
    let mut passive = ministerial_housing_business_return(true, false);
    let mut active = ministerial_housing_business_return(false, false);
    let id = passive.income.last().unwrap().id;
    // Exercise the actual benefit owners as well as ordinary source totals.
    let FederalIncome::Business { data: plan } = health_business(20_000, 1000, false) else {
        unreachable!()
    };
    let FederalIncome::Business { data: retirement } = sep_business(20_000, 1000) else {
        unreachable!()
    };
    for draft in [&mut passive, &mut active] {
        let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.health_insurance = plan.health_insurance.clone();
        data.retirement = retirement.retirement.clone();
    }
    let result = supported(&passive);
    let control = supported(&active);
    assert_eq!(
        result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("business_{id}_passive_current"))
            .unwrap()
            .amount,
        1_333_333
    );
    for key in [
        "agi",
        "self_employment_tax",
        "qbi_deduction",
        "schedule_1_business_income",
        "schedule_1_self_employed_health_insurance",
        "schedule_1_self_employed_retirement",
    ] {
        assert_eq!(value(&result, key), value(&control, key), "{key}");
    }
    assert!(value(&result, "self_employment_tax") > 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 0);
    assert_eq!(value(&result, "net_investment_income_tax"), 0);
}

#[test]
fn passive_exempt_ministry_retains_complete_native_net_in_niit() {
    let passive = ministerial_housing_business_return(true, true);
    let active = ministerial_housing_business_return(false, true);
    let result = supported(&passive);
    let control = supported(&active);
    for key in ["agi", "self_employment_tax", "qbi_deduction", "schedule_1_business_income"] {
        assert_eq!(value(&result, key), value(&control, key), "{key}");
    }
    assert_eq!(value(&result, "schedule_1_business_income"), 13_333);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 13_333);
    assert_eq!(value(&result, "net_investment_income_tax"), 507);
    assert_eq!(value(&control, "net_investment_income_tax"), 0);
}

#[test]
fn passive_ministry_housing_correction_precedes_native_at_risk_limit() {
    let mut passive = ministerial_housing_business_return(true, false);
    let FederalIncome::Business { data } = &mut passive.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.at_risk = simplified_rental_risk(0, 0, 0, [0; 4]).unwrap();
    let id = passive.income.last().unwrap().id;
    let result = supported(&passive);
    let native = supported(&ministerial_housing_business_return(false, false));
    for suffix in ["form6198_line1", "passive_current", "passive_allowed", "net_profit"] {
        let line = result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("business_{id}_{suffix}"))
            .unwrap();
        assert_eq!(line.amount, 1_333_333, "{suffix}");
    }
    assert_eq!(value(&result, &alloc::format!("business_{id}_at_risk_carryover")), 0);
    for key in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&result, key), value(&native, key), "{key}");
    }
}

#[test]
fn passive_ministry_amt_retains_its_native_depreciation_and_expense_correction() {
    let mut draft = ministerial_housing_business_return(true, true);
    let mut asset = depreciable_asset(10_000, EquipmentClass::Five, 20260101, 0);
    // Pre-2017 acquisition first used in2026: regular200% versus AMT150%,
    // with no bonus; the owner's current housing expense correction differs.
    asset.acquired = 20100101;
    asset.id = draft.next_id;
    draft.next_id += 1;
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.assets.push(asset);
    let id = draft.income.last().unwrap().id;
    let result = supported(&draft);
    for (prefix, cents) in [("", 1_200_000), ("amt_", 1_233_333)] {
        let row = result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("{prefix}business_{id}_passive_current"))
            .unwrap();
        assert_eq!(row.amount, cents);
    }
    assert_eq!(value(&result, "amt_business_depreciation_adjustment"), 333);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 12_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 456);
}

#[test]
fn current_property_risk_and_pal_preserve_original_ordinary_loss_and_operating_earnings() {
    // $3,000 C loss plus $1,000 native short-held property loss. The $2,000
    // risk capacity permits $1,500/$500; $1,000 other passive profit permits
    // $750/$250 after PAL. Only C earnings enter Schedule SE.
    let mut draft = current_property_business_return(
        true,
        -3000,
        Some(200_000),
        vec![same_year_property_sale(200_049, 100_049)],
    );
    let id = draft.income[1].id;
    add(&mut draft, income(passive_business(1000)));
    let accepted = draft.clone();
    let result = supported(&draft);
    for (suffix, expected) in [
        ("form6198_line1", -3000),
        ("form6198_line2b", -1000),
        ("form6198_line5", -4000),
        ("at_risk_carryover", 2000),
        ("at_risk_allowed_ordinary", -1500),
        ("at_risk_allowed_ordinary_property", -500),
        ("passive_suspended", 1000),
        ("net_profit", -750),
        ("asset_0_disposal_ordinary", -250),
    ] {
        assert_eq!(value(&result, &alloc::format!("business_{id}_{suffix}")), expected, "{suffix}");
    }
    assert_eq!(value(&result, "schedule_1_business_income"), 250);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    assert!(!result.lines.iter().any(|line| line.id == "schedule_1_rental_income"));
    assert_eq!(draft, accepted);
}

#[test]
fn current_property_positive_1245_subset_supplies_risk_room_without_becoming_se_income() {
    // Fully recovered $5,000 equipment yields $3,100 known section1245 income.
    // I6198's gain-offset example permits $3,700 of the $4,600 C loss with
    // $600 risk capacity. PAL then holds $600 more: C -$3,100, 4797 +$3,100.
    let mut asset = continuing_asset(
        500_000,
        20180101,
        EquipmentClass::Five,
        500_000,
        same_year_property_sale(1, 310_000).disposal,
    );
    asset.method = EquipmentMethod::Regular;
    let draft = current_property_business_return(true, -4600, Some(60_000), vec![asset]);
    let id = draft.income[1].id;
    let result = supported(&draft);
    for (suffix, expected) in [
        ("at_risk_allowed_ordinary", -3700),
        ("at_risk_carryover", 900),
        ("passive_suspended", 600),
        ("net_profit", -3100),
        ("asset_0_disposal_1245_recapture", 3100),
    ] {
        assert_eq!(value(&result, &alloc::format!("business_{id}_{suffix}")), expected, "{suffix}");
    }
    assert_eq!(value(&result, "form8582_line_2a"), 3100);
    assert_eq!(value(&result, "form8582_line_2b"), -3700);
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    assert!(!noted(&result, "BUSINESS_RISK_RECAPTURE_CHARACTER_REQUIRED"));
}

#[test]
fn current_property_original_ids_survive_reorder_and_each_loss_row_keeps_its_allowance() {
    let mut draft = current_property_business_return(
        true,
        0,
        Some(200_000),
        vec![same_year_property_sale(200_000, 100_000), same_year_property_sale(400_000, 100_000)],
    );
    let id = draft.income[1].id;
    add(&mut draft, income(passive_business(1000)));
    let first = supported(&draft);
    assert_eq!(value(&first, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), -250);
    assert_eq!(value(&first, &alloc::format!("business_{id}_asset_1_disposal_ordinary")), -750);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let ids: Vec<_> = data.assets.iter().map(|asset| asset.id).collect();
    data.assets.reverse();
    assert_eq!(data.assets.iter().map(|asset| asset.id).collect::<Vec<_>>(), vec![ids[1], ids[0]]);
    let after = supported(&draft);
    assert_eq!(value(&after, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), -750);
    assert_eq!(value(&after, &alloc::format!("business_{id}_asset_1_disposal_ordinary")), -250);
    assert_eq!(value(&first, "agi"), value(&after, "agi"));
    assert_eq!(value(&first, "qbi_deduction"), value(&after, "qbi_deduction"));
}

#[test]
fn passive_partial_building_loss_returns_to_4797_after_pal_without_depreciating_twice() {
    // $39,000 component: $10,418 native recovery leaves $28,582 loss.
    // Remaining-building depreciation is $9,457.96; both losses are passive.
    let mut building = continuing_asset(
        39_000_000,
        20160115,
        EquipmentClass::ThirtyNine,
        9_959_430,
        AssetDisposal::No,
    );
    building.partial =
        Some(PartialDisposition { date: 20260610, cost_of_part: 3_900_000, proceeds: 0 });
    let draft = current_property_business_return(true, 0, None, vec![building]);
    let id = draft.income[1].id;
    let result = supported(&draft);
    assert_eq!(
        value(&result, &alloc::format!("business_{id}_asset_0_partial_depreciation")),
        10_418
    );
    assert_eq!(value(&result, &alloc::format!("business_{id}_asset_0_partial_gain")), 0);
    // 2026-10-24: this work row retains Cents; value() truncates Dollars.
    let depreciation = result
        .lines
        .iter()
        .find(|row| row.id == alloc::format!("business_{id}_asset_0_macrs"))
        .unwrap();
    assert_eq!(depreciation.amount, 945_796);
    let suspended = result
        .lines
        .iter()
        .find(|row| row.id == alloc::format!("business_{id}_passive_suspended"))
        .unwrap();
    assert_eq!(suspended.amount, 3_803_996);
    assert_eq!(value(&result, "agi"), 60_000);
    assert!(!result.lines.iter().any(|line| line.id == "form4797_line1c"));
    let FederalIncome::Business { data } = &draft.income[1].income else { unreachable!() };
    assert_eq!(data.assets[0].cost, 39_000_000);
    assert_eq!(data.assets[0].partial.as_ref().unwrap().cost_of_part, 3_900_000);
}

#[test]
fn passive_property_gain_retains_niit_while_operating_profit_retains_se_character() {
    let mut draft = current_property_business_return(
        true,
        20_000,
        None,
        vec![same_year_property_sale(100_000, 1_100_000)],
    );
    draft.income[0] = wage(1, Owner::Taxpayer, 26_000_000, 0);
    // No section1231 source exists, so this does not require a lookback record.
    draft.screening.as_mut().unwrap().section1231 = None;
    let result = supported(&draft);
    assert_eq!(value(&result, "schedule_1_business_income"), 20_000);
    assert!(value(&result, "self_employment_tax") > 0);
    assert_eq!(value(&result, "form_8960_net_investment_income"), 10_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 380);
}

#[test]
fn current_property_native_save_reopen_restart_correction_and_deletion_recompute() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = business(0);
    page.more.material_participation =
        ParticipationFacts { hours: 0, test: ParticipationTest::None };
    page.more.at_risk = simplified_rental_risk(100_000, 0, 0, [0; 4]).unwrap();
    let mut asset = crate::tests::cross::interview::with_first_year(
        crate::tests::cross::interview::asset(200_000, 20260101, AssetClass::Five),
        FirstYearDeduction::Regular,
    );
    asset.disposal = same_year_property_sale(200_000, 100_000).disposal;
    page.assets = vec![asset];
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let id = stored_business(&session).0;
    let property_id = stored_business(&session).1.assets[0].id;
    assert!(property_id > 0);
    assert_round_trip(&mut session, &[page]);
    session = restart(session.draft.clone());
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), 0);
    let mut documents =
        page_documents(&open_topic(&mut session, TaxSection::SelfEmployment)).to_vec();
    let IncomeSource::Record { value: crate::IncomeRecord::Business(page) } =
        &mut documents[0].document
    else {
        unreachable!()
    };
    page.assets[0].disposal = same_year_property_sale(200_000, 300_000).disposal;
    let request = save_topic(&mut session, TaxSection::SelfEmployment, documents);
    accepted(&send(&mut session, request));
    session = restart(session.draft.clone());
    assert_eq!(stored_business(&session).1.assets[0].id, property_id);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), 1000);
    let request = save_topic(&mut session, TaxSection::SelfEmployment, vec![]);
    accepted(&send(&mut session, request));
    session = restart(session.draft.clone());
    assert_eq!(value(&supported(&session.draft), "agi"), 60_000);
}

#[test]
fn material_current_property_loss_after_risk_reaches_461_and_qbi_once() {
    // $500,000 ordinary property loss is limited to $400,000 at risk.
    // Section461's $256,000 single-filer limit then defers $144,000.
    let draft = current_property_business_return(
        false,
        0,
        Some(40_000_000),
        vec![same_year_property_sale(60_000_000, 10_000_000)],
    );
    let id = draft.income[1].id;
    let result = supported(&draft);
    assert_eq!(
        value(&result, &alloc::format!("business_{id}_asset_0_disposal_ordinary")),
        -400_000
    );
    assert_eq!(value(&result, &alloc::format!("business_{id}_at_risk_carryover")), 100_000);
    assert_eq!(value(&result, "excess_business_loss"), 144_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 256_000);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "agi"), -196_000);
}

#[test]
fn current_property_regular_and_amt_native_columns_keep_independent_risk_outcomes() {
    // Five-year equipment: regular table recovery through2025 $5,200 and
    // disposal-year $960; AMT150% table recovery $4,050 plus $892.50.
    // Native4797 recovery is $6,160/$4,943, so a $2,000 sale loses
    // $1,840/$3,057. Zero risk lets positive operating income absorb only
    // $1,040/$1,107.50 of those respective property losses.
    let mut asset = continuing_asset(
        1_000_000,
        20240301,
        EquipmentClass::Five,
        520_000,
        same_year_property_sale(1, 200_000).disposal,
    );
    asset.acquired = 20160101;
    let EquipmentHistory::Continuing { amt, .. } = &mut asset.history else { unreachable!() };
    *amt = EquipmentAmtHistory::Separate {
        prior_depreciation: 405_000,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    let draft = current_property_business_return(false, 2000, Some(0), vec![asset]);
    let id = draft.income[1].id;
    let result = supported(&draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_form6198_line2b")), -1840);
    assert_eq!(value(&result, &alloc::format!("business_{id}_at_risk_carryover")), 800);
    assert_eq!(
        value(&result, &alloc::format!("business_{id}_asset_0_disposal_section1231")),
        -1040
    );
    let row = result
        .lines
        .iter()
        .find(|row| row.id == alloc::format!("amt_business_{id}_at_risk_carryover"))
        .unwrap();
    assert_eq!(row.amount, 194_950);
    let row = result
        .lines
        .iter()
        .find(|row| row.id == alloc::format!("amt_business_{id}_at_risk_allowed_section1231"))
        .unwrap();
    assert_eq!(row.amount, -110_750);
    assert_eq!(value(&result, "amt_business_depreciation_adjustment"), 68);
    // The actual $1,040 REG allowed property loss offsets its $1,040
    // operating source in rental MAGI; AMT retains its own $1,107.50 risk
    // result above while using the regular rental-MAGI invocation base.
    assert_eq!(value(&result, "rental_magi"), 60_000);
    assert_eq!(value(&result, "amt_rental_magi"), 60_000);
}

#[test]
fn passive_property_1250_subset_and_capital_character_return_to_the_original_source_once() {
    let mut disposal = same_year_property_sale(1, 40_000_000).disposal;
    let AssetDisposal::Disposed { date, .. } = &mut disposal else { unreachable!() };
    *date = 20260620;
    // Native building columns: cost$390,000, recovery$104,177, price$400,000;
    // $114,177 section1231 gain includes $104,177 unrecaptured1250.
    let building =
        continuing_asset(39_000_000, 20160115, EquipmentClass::ThirtyNine, 9_959_430, disposal);
    let mut draft = current_property_business_return(true, 0, None, vec![building]);
    add(&mut draft, income(passive_business(-10_000)));
    let id = draft.income[1].id;
    let result = supported(&draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_asset_0_disposal_gain")), 114_177);
    assert_eq!(value(&result, "form4797_line10"), 114_177);
    assert_eq!(value(&result, "unrecaptured_section_1250_gain"), 104_177);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 14_583);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    assert_eq!(value(&result, "agi"), 159_594);
    // 2026-10-24: i8960 Who Must File requires MAGI above $200,000.
    // The $114,177 property gain is unexcluded; this return owes no NIIT and
    // the native form owner omits Form8960 rather than emitting line12.
    assert_eq!(value(&result, "niit_disposition_adjustment"), 0);
    assert_eq!(value(&result, "niit_income_tax_deduction"), 0);
    assert_eq!(value(&result, "net_investment_modified_agi"), 159_594);
    assert_eq!(value(&result, "net_investment_income_tax"), 0);
    assert!(
        !result
            .lines
            .iter()
            .any(|row| row.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form8960))
    );
    assert!(!result.lines.iter().any(|line| line.id == "schedule_1_rental_income"));
}

#[test]
fn native_property_category_rounds_once_across_two_half_dollar_loss_rows() {
    for passive in [false, true] {
        // Risk-only: $2 current loss, $1 capacity. PAL-only: $2 current loss,
        // $1 other passive profit. Both leave exactly50c allowed per property.
        let mut draft = current_property_business_return(
            passive,
            0,
            (!passive).then_some(100),
            vec![same_year_property_sale(200, 100), same_year_property_sale(200, 100)],
        );
        let id = draft.income[1].id;
        if passive {
            add(&mut draft, income(passive_business(1)));
        }
        let before = supported(&draft);
        assert_eq!(value(&before, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), -1);
        assert_eq!(value(&before, &alloc::format!("business_{id}_asset_1_disposal_ordinary")), 0);
        assert_eq!(value(&before, "agi"), if passive { 60_000 } else { 59_999 });
        assert_eq!(value(&before, "qbi_loss_carryforward"), if passive { 0 } else { 1 });
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        data.assets.reverse();
        let after = supported(&draft);
        assert_eq!(value(&after, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), 0);
        assert_eq!(value(&after, &alloc::format!("business_{id}_asset_1_disposal_ordinary")), -1);
        assert_eq!(value(&before, "agi"), value(&after, "agi"));
        // The same already-limited source remains whole on a community
        // separate return. Native annual report residuals stay on their IDs;
        // this does not re-recognize the original $2 loss or divide it again.
        let mut community = draft.clone();
        community.setup = complete(FederalStatus::Separate).setup;
        answer_separate_pages(&mut community);
        community.setup.as_mut().unwrap().community_property = community_arizona();
        for entry in &mut community.income {
            entry.community = if matches!(entry.income, FederalIncome::Wage { .. }) {
                CommunityShare::Community
            } else {
                CommunityShare::Separate
            };
        }
        let result = supported(&community);
        assert_eq!(value(&result, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), 0);
        assert_eq!(value(&result, &alloc::format!("business_{id}_asset_1_disposal_ordinary")), -1);
        assert_eq!(line_amount(&result, &alloc::format!("form8958_gain_{id}_this_return")), -100);
        assert_eq!(value(&result, "agi"), if passive { 30_000 } else { 29_999 });
    }
}

#[test]
fn native_signed_property_class_preserves_positive_dollar_and_half_dollar_loss_residual() {
    for passive in [false, true] {
        // C -$1, property -$1 and+$1. Zero risk capacity or zero external
        // PAL income holds50c each from C/property. Native C is-$1; the
        // signed property category is+50c→+$1, so its negative row is$0.
        let draft = current_property_business_return(
            passive,
            -1,
            (!passive).then_some(0),
            vec![same_year_property_sale(200, 100), same_year_property_sale(100, 200)],
        );
        let id = draft.income[1].id;
        let result = supported(&draft);
        assert_eq!(
            result
                .lines
                .iter()
                .find(|row| row.id == alloc::format!("business_{id}_net_profit"))
                .unwrap()
                .amount,
            -50
        );
        assert_eq!(value(&result, &alloc::format!("business_{id}_asset_0_disposal_ordinary")), 0);
        assert_eq!(value(&result, &alloc::format!("business_{id}_asset_1_disposal_ordinary")), 1);
        assert_eq!(value(&result, "agi"), 60_000);
        assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
        assert_eq!(value(&result, "self_employment_tax"), 0);
    }
}
