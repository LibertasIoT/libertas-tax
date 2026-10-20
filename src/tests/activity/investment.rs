use super::*;
use crate::tests::investment::investment::{
    add_vacation_home, bond_interest_entry, bond_record, bond_student, cents, rental_loss_return,
    rental_sale_return, sold_rental, unit40_return, unit40_roof,
};

#[test]
fn rental_profit_flows_to_schedule_e_agi_niit_and_qbi_once() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 21_000_000, 0)];
    let mut v = rental(30_000, 5000); // $15,000 net after $10,000 depreciation.
    // Above the QBI phase-in scope, use investment treatment for this NIIT fixture.
    v.qbi_business = RentalBusiness::No;
    add(&mut d, income(FederalIncome::Rental { data: v.clone() }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 15_000);
    assert_eq!(value(&r, "agi"), 225_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 570);
    assert_eq!(return_line(&r, "8"), 1_500_000);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    v.qbi_business = RentalBusiness::Yes { material_participation: false };
    d.income[1].income = FederalIncome::Rental { data: v };
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 3000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
}

#[test]
fn passive_rental_allowance_matches_irs_example_and_phaseout_boundaries() {
    for (wages, allowed, suspended) in
        [(100000, 25000, 6000), (120000, 15000, 16000), (150000, 0, 31000)]
    {
        let mut d = complete(FederalStatus::Single);
        d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
        add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
        let r = supported(&d);
        assert_eq!(value(&r, "rental_income"), -allowed);
        assert_eq!(value(&r, "rental_suspended_loss"), suspended);
        assert_eq!(value(&r, "agi"), wages - allowed);
    }
}

#[test]
fn rental_qbi_minimum_requires_material_participation() {
    for (material, expected) in [(true, 400), (false, 200)] {
        let mut d = complete(FederalStatus::Single);
        let mut v = rental(11000, 0);
        v.qbi_business = RentalBusiness::Yes { material_participation: material };
        add(&mut d, income(FederalIncome::Rental { data: v }));
        assert_eq!(value(&supported(&d), "qbi_deduction"), expected);
    }
}

#[test]
fn rental_allocations_reconcile_with_multiple_tiny_loss_properties() {
    let mut d = complete(FederalStatus::Single);
    for _ in 0..3 {
        let mut v = rental(9999, 0);
        v.participation = crate::RentalParticipation::NotActive;
        add(&mut d, income(FederalIncome::Rental { data: v }));
    }
    add(&mut d, income(FederalIncome::Rental { data: rental(10001, 0) }));
    let r = supported(&d);
    let per_property = r
        .lines
        .iter()
        .filter(|l| {
            l.id.starts_with("rental_")
                && l.id.ends_with("_suspended_loss")
                && l.id != "rental_suspended_loss"
        })
        .map(|l| l.amount)
        .sum::<i64>();
    assert_eq!(value(&r, "rental_suspended_loss"), 2);
    assert_eq!(per_property, 200);
}

#[test]
fn qbi_2026_phase_in_service_limits_and_loss_netting() {
    use crate::business_rules::{QbiActivity, qbi};
    let activity = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 100000,
        wages: 0,
        property: 0,
        active: false,
        service: Some(false),
        group: None,
    };
    let mut r = supported(&complete(FederalStatus::Single));
    for (taxable, expected) in [(201750, 20000), (239250, 10000), (276750, 0)] {
        assert_eq!(
            qbi(core::slice::from_ref(&activity), taxable, 0, FederalStatus::Single, &mut r),
            (expected, 0)
        );
    }
    let service = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        service: Some(true),
        ..activity.clone()
    };
    assert_eq!(
        qbi(core::slice::from_ref(&service), 239250, 0, FederalStatus::Single, &mut r),
        (5000, 0)
    );
    assert_eq!(qbi(&[service], 276750, 0, FederalStatus::Single, &mut r), (0, 0));
    let wages = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        wages: 10000,
        ..activity.clone()
    };
    assert_eq!(qbi(&[wages], 300000, 0, FederalStatus::Single, &mut r), (5000, 0));
    // An unknown classification is the conservative one: a specified service
    // for income above the threshold (Answers and unknowns).
    let unresolved = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        service: None,
        ..activity.clone()
    };
    let specified = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        service: Some(true),
        ..activity.clone()
    };
    let issues = r.issues.len();
    assert_eq!(
        qbi(&[unresolved], 300000, 0, FederalStatus::Single, &mut r),
        qbi(&[specified], 300000, 0, FederalStatus::Single, &mut r)
    );
    assert_eq!(r.issues.len(), issues);
    let grouped_service = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        service: Some(true),
        group: Some("Shops".into()),
        ..activity.clone()
    };
    qbi(&[grouped_service], 300000, 0, FederalStatus::Single, &mut r);
    assert!(r.issues.iter().any(|i| {
        {
            let m = &i.display;
            m.key == "ISSUE_QBI_SERVICE_BUSINESS_AGGREGATION"
        }
    }));
    let property = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        property: 400000,
        ..activity.clone()
    };
    assert_eq!(qbi(&[property], 300000, 0, FederalStatus::Single, &mut r), (10000, 0));
    let profit = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        income: 200000,
        wages: 50000,
        ..activity.clone()
    };
    let loss = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        income: -50000,
        ..activity.clone()
    };
    assert_eq!(qbi(&[profit, loss], 300000, 0, FederalStatus::Single, &mut r), (25000, 0));
    assert_eq!(
        qbi(
            &[QbiActivity {
                source_wages_cents: None,
                source_property_cents: None,
                source_income_cents: None,
                active: true,
                ..activity.clone()
            }],
            300000,
            0,
            FederalStatus::Single,
            &mut r
        ),
        (400, 0)
    );
    assert_eq!(
        qbi(
            &[QbiActivity {
                source_wages_cents: None,
                source_property_cents: None,
                source_income_cents: None,
                income: -3000,
                ..activity
            }],
            10000,
            0,
            FederalStatus::Single,
            &mut r
        ),
        (0, 3000)
    );
}

#[test]
fn professional_gambling_cannot_create_loss_and_applies_meal_limits_first() {
    let mut entry = professional_gambling(100000, 120000, 10000);
    if let FederalIncome::Business { data } = &mut entry {
        assert_eq!(data.expense_total(false), 10000000);
        match &mut data.activity {
            BusinessActivity::Ordinary { receipts, .. }
            | BusinessActivity::Gambling { receipts, .. } => *receipts = 9500000,
            _ => panic!("expected nonfarm test business"),
        };
        data.activity = BusinessActivity::Gambling {
            returns: 0,
            receipts: data.receipts(),
            accounting: BusinessAccounting::Cash,
            losses: 9000000,
            withholding: 0,
            eligible: true,
        };
        data.expenses[0].treatment = BusinessExpenseKind::Meals;
        assert_eq!(data.expense_total(false), 8550000); // 90% of 90,000 wagers + 5,000 allowable meals.
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(entry));
    supported(&d);
    add(&mut d, gambling(1000, 0));
    unsupported(&d, "Income");
}

#[test]
fn professional_gambling_qbi_uses_limited_profit_and_invalid_losses_are_rejected() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(professional_gambling(100000, 90000, 10000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 69293);
    assert_eq!(value(&r, "qbi_deduction"), 1859);
    assert_eq!(value(&r, "self_employment_tax"), 1413);
    if let FederalIncome::Business { data } = &mut d.income[1].income {
        data.activity = BusinessActivity::Gambling {
            returns: 0,
            receipts: data.receipts(),
            accounting: BusinessAccounting::Cash,
            losses: -1,
            withholding: 0,
            eligible: true,
        };
    }
    assert!(!d.valid_stored());
}

#[test]
fn rental_de_minimis_uses_shared_eligibility_and_amount_validation() {
    let mut d = complete(FederalStatus::Single);
    let mut property = rental(50000, 0);
    property.expenses.clear();
    property.expenses.push(crate::extended_model::RentalExpense {
        category: crate::extended_model::RentalExpenseCategory::Other {
            label: "Small rental appliances".into(),
        },
        amount: 100000,
        treatment: BusinessExpenseKind::DeMinimis {
            largest_cost: 100000,
            financial_statement: false,
            eligible: true,
        },
    });
    add(&mut d, income(FederalIncome::Rental { data: property }));
    assert_eq!(value(&supported(&d), "rental_2_expense_0_de_minimis"), 1000);
    let FederalIncome::Rental { data } = &mut d.income[1].income else { panic!() };
    let BusinessExpenseKind::DeMinimis { eligible, .. } = &mut data.expenses[0].treatment else {
        panic!()
    };
    *eligible = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_DE_MINIMIS_EXPENSE_LEFT_OUT"));
    assert!(r.lines.iter().all(|l| l.id != "rental_2_expense_0_de_minimis"));
    let FederalIncome::Rental { data } = &mut d.income[1].income else { panic!() };
    let BusinessExpenseKind::DeMinimis { largest_cost, .. } = &mut data.expenses[0].treatment
    else {
        panic!()
    };
    *largest_cost = -1;
    assert!(!d.valid_stored());
}

#[test]
fn personal_rental_passive_profit_is_excluded_from_special_allowance_magi() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12000000, 0);
    add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
    add(&mut d, income(personal_rent(12000, 2000)));
    let r = supported(&d);
    // 31,000 loss less 10,000 passive profit and 15,000 special allowance.
    assert_eq!(value(&r, "rental_magi"), 120000);
    assert_eq!(value(&r, "rental_income"), -25000);
    assert_eq!(value(&r, "rental_suspended_loss"), 6000);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 6000);
    assert_eq!(value(&r, "agi"), 105000);
}

#[test]
fn passive_special_allowance_precedes_suspended_loss_allocation() {
    let mut d = complete(FederalStatus::Single);
    let active = rental(0, 10000); // 20,000 net loss including depreciation.
    let mut other = active.clone();
    other.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: active }));
    add(&mut d, income(FederalIncome::Rental { data: other }));
    add(&mut d, income(personal_rent(10000, 0)));
    let r = supported(&d);
    // Lines 1d=-20,000; 2d=-10,000; 3=-30,000; 9=20,000; 11=30,000.
    assert_eq!(value(&r, "rental_income"), -30000);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "rental_3_suspended_loss"), 10000);
    assert_eq!(value(&r, "rental_suspended_loss"), 10000);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 10000);
    assert_eq!(value(&r, "agi"), 40000);
    d.income.swap(1, 2);
    let reordered = supported(&d);
    assert_eq!(value(&reordered, "rental_2_suspended_loss"), 0);
    assert_eq!(value(&reordered, "rental_3_suspended_loss"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn personal_rental_losses_share_profit_but_never_active_special_allowance() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent(1000, 21000)));
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 10000) }));
    add(&mut d, income(personal_rent(10000, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -20000);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 10000);
    assert_eq!(value(&r, "rental_3_suspended_loss"), 0);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 11000);
    assert_eq!(value(&r, "agi"), 40000);
}

#[test]
fn personal_rental_expense_report_reconciles_opening_allowed_and_zero_closing_balances() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent_with_history(12000, 2000, 8000, 12000, true)));
    let r = supported(&d);
    for (prefix, opening, allowed, closing) in [("", 8000, 10000, 0), ("amt_", 12000, 12000, 2000)]
    {
        for (suffix, expected) in [
            ("prior_suspended_expenses", opening),
            ("allowed_expenses", allowed),
            ("suspended_loss", closing),
        ] {
            let id = alloc::format!("{prefix}personal_rental_2_{suffix}");
            assert_eq!(r.lines.iter().filter(|v| v.id == id).count(), 1);
            assert_eq!(value(&r, &id), expected);
        }
        assert_eq!(2000 + opening, allowed + closing);
    }
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn passive_pool_allowance_uses_active_group_net_and_remaining_total_loss() {
    use crate::tests::support::loss_limits::{PassiveActivity as A, passive_suspensions};
    let pool = [
        A { key: (), net: -30_000, rental_allowance: true, ineligible_prior: 0 },
        A { key: (), net: 20_000, rental_allowance: true, ineligible_prior: 0 },
        A { key: (), net: -50_000, rental_allowance: false, ineligible_prior: 0 },
    ];
    let r = passive_suspensions(&pool, 0, 25_000).unwrap();
    assert_eq!(r.iter().sum::<i64>(), 50_000); // only $10k special allowance
    assert_eq!(r[1], 0);
    assert_eq!(passive_suspensions(&pool, 70_000, 25_000).unwrap(), vec![0, 0, 0]);
}

#[test]
fn live_passive_qbi_uses_old_loss_character_and_preserves_three_carryforwards() {
    let source = passive_qbi_source();
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    add(&mut d, income(FederalIncome::Rental { data: rental(16_000, 0) }));
    let r = estimate(&d);
    scorp_note(&r, id, "QBI loss passing basis limits", 6_500);
    scorp_note(&r, id, "QBI loss passing at-risk limits", 4_000);
    scorp_note(&r, id, "QBI loss passing passive limits", 2_000);
    let message = r
        .review_notes
        .iter()
        .map(|note| &note.display)
        .find(|message| message.key == "NOTE_SCORP_QBI_PASSIVE_RELEASE")
        .expect("localized passive QBI release");
    assert_eq!(
        message.args,
        vec![
            crate::MessageArg::Number(id),
            crate::MessageArg::Currency {
                currency_code: "USD".into(),
                scale: 2,
                minor_units: 200_000,
            },
        ]
    );
    for expected in [
        "2026 basis-loss carryforward: $2000.00; QBI portion: $1000.00. Original suspended loss: $2000.00; original QBI portion: $1000.00.",
        "2026 at-risk-loss carryforward: $5000.00; QBI portion: $2500.00. Original suspended loss: $5000.00; original QBI portion: $2500.00.",
        "2026 passive-loss carryforward: $4000.00; QBI portion: $2000.00. Original suspended loss: $4000.00; original QBI portion: $2000.00.",
    ] {
        assert!(r.review_notes.iter().any(|v| v.contains(expected)), "{expected}");
    }
    assert_eq!(restart(d.clone()).draft, d);
    assert!(r.tax.is_none() && r.refund.is_none());
}

#[test]
fn former_passive_qbi_never_resuspends_current_nonpassive_loss() {
    let mut source = passive_qbi_source();
    source.participation.as_mut().unwrap().minutes = 30_001;
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: source.clone() }));
    add(&mut d, income(FederalIncome::Rental { data: rental(11_000, 0) }));
    let r = estimate(&d);
    scorp_note(&r, id, "QBI loss passing passive limits", 4_000);
    let release = source
        .loss_qbi
        .after_passive(&source, crate::tests::support::builders::k1_amount(1_000))
        .unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (9_000, 4_000)
    );
    assert_eq!(release.remaining.len(), 1);
    assert_eq!(
        (
            release.remaining[0].year,
            crate::kernel::ratio::dollars(release.remaining[0].remaining.get())
        ),
        (2017, 1_000)
    );
    assert!(
        source
            .loss_qbi
            .after_passive(&source, crate::tests::support::builders::k1_amount(2_001))
            .is_err()
    );
}

#[test]
fn former_passive_own_income_loss_release_does_not_reduce_unrelated_investment_income() {
    let (d, _) = investment_tax_draft(nonpassive_scorp(10_000, 4_000), 10_000);
    investment_tax_note(&estimate(&d), 10_000, 380);
    let (mut d, _) = investment_tax_draft(nonpassive_scorp(10_000, 15_000), 10_000);
    add(&mut d, income(FederalIncome::Rental { data: rental(13_000, 0) }));
    investment_tax_note(&estimate(&d), 10_000, 380); // only $3k older loss offsets $3k passive rent
}

#[test]
fn former_passive_property_zero_net_current_income_does_not_release_history_twice() {
    let source = former_passive_property_scorp(-5_000, 5_000, 6_000, 4_000);
    let activity = source.pool_activity_result().unwrap();
    assert_eq!(activity.prior_released, 0);
    assert_eq!(activity.nonpassive, 0);
    let (d, id) = passive_property_pool(source, 5_000);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert_eq!(pool.passive_loss, 5_000);
    assert_eq!(pool.income, -3_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, 3_000, 2_000, 5_000)]);
    // Form 8582 Part IX (F3, 2026-10-09): the activity's passive losses are
    // 6,000 ordinary and 4,000 section 1231 against 5,000 of other passive
    // income, so 5,000 is unallowed, split 3,000 / 2,000 in proportion to the
    // losses; the rest is allowed.
    for (suffix, column, expected) in [
        ("ordinary_unallowed", "IX (d)", 3_000),
        ("ordinary_allowed", "IX (e)", 3_000),
        ("section1231_unallowed", "IX (d)", 2_000),
        ("section1231_allowed", "IX (e)", 2_000),
    ] {
        let line = alloc::format!("form8582_part_ix_{id}_{suffix}");
        assert_eq!(value(&r, &line), expected, "{line}");
        assert_eq!(
            printed_line(&r, &line),
            (TaxForm::Form8582, alloc::string::ToString::to_string(column))
        );
    }
}

#[test]
fn former_passive_full_release_keeps_both_loss_characters_without_limiting_each_to_its_income() {
    for ordinary in [0, 5_000] {
        let source = former_passive_property_scorp(ordinary, 10_000, 6_000, 4_000);
        let activity = source.pool_activity_result().unwrap();
        assert_eq!(activity.passive, 0);
        assert_eq!(crate::kernel::ratio::dollars(activity.prior_released), 10_000);
        assert_eq!(
            activity.characters.unwrap().prior_released.map(crate::kernel::ratio::dollars),
            [6_000, 4_000]
        );
        let (d, id) = passive_property_pool(source, 0);
        let mut r = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
        assert_eq!(pool.passive_loss, 0);
        assert_eq!(pool.income, ordinary - 6_000);
        assert_eq!(pool.k1_section1231, vec![property_result(id, 6_000, 0, 6_000)]);
    }
}

#[test]
fn former_passive_property_amt_uses_own_history_and_regular_magi() {
    let mut source = former_passive_property_scorp(5_000, 5_000, 8_000, 0);
    source.amt.as_mut().unwrap().adjustment = -1_000_000;
    let (d, id) = passive_property_pool(source, 4_000);
    for amt in [false, true] {
        let mut r = estimate(&complete(FederalStatus::Single));
        r.lines.clear();
        let pool = crate::extended_rules::rentals_for(&d, 200_000, &mut r, amt);
        assert_eq!(value(&r, if amt { "amt_rental_magi" } else { "rental_magi" }), 202_000);
        assert_eq!(pool.passive_loss, if amt { 4_000 } else { 0 });
        assert_eq!(pool.income, if amt { -5_000 } else { 1_000 });
        assert_eq!(pool.k1_section1231, vec![property_result(id, 5_000, 0, 5_000)]);
    }
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn former_passive_partial_release_nets_current_loss_before_old_history() {
    let source = former_passive_property_scorp(-5_000, 10_000, 6_000, 4_000);
    let activity = source.pool_activity_result().unwrap();
    let c = activity.characters.unwrap();
    assert_eq!(c.prior_released.map(crate::kernel::ratio::dollars), [1_000, 4_000]);
    assert_eq!(c.passive.map(crate::kernel::ratio::dollars), [-5_000, 0]);
    assert_eq!(c.nonpassive.map(crate::kernel::ratio::dollars), [-6_000, 6_000]);
    let (d, id) = passive_property_pool(source, 2_000);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert!(pool.inputs_complete);
    assert_eq!(pool.passive_loss, 3_000);
    assert_eq!(pool.income, -6_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, 6_000, 0, 6_000)]);
    unsupported(&d, "Section 1231 lookback");
}

#[test]
fn property_qbi_former_passive_current_losses_are_not_resuspended() {
    let mut source = former_passive_property_scorp(10_000, -5_000, 0, 10_000);
    let mut facts = property_qbi(5_000, 0);
    facts.losses.passive = vec![loss_year(2022, 10_000, 4_000, 10_000, 4_000)];
    source.property_qbi = facts;
    source.amt = Some(amt_records(&source));
    // Own net income releases $5k prior loss; other passive profit releases
    // another $2k. Current $5k property loss is nonpassive and allowed separately.
    let qbi = source
        .property_qbi
        .after_passive(&source, crate::tests::support::builders::k1_amount(3_000))
        .unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(qbi.amount.get()),
            crate::kernel::ratio::dollars(qbi.qbi.get())
        ),
        (12_000, 7_800)
    );
    assert_eq!(qbi.remaining.len(), 1);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(qbi.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(qbi.remaining[0].remaining_qbi.get())
        ),
        (3_000, 1_200)
    );
    assert!(
        source
            .property_qbi
            .after_passive(&source, crate::tests::support::builders::k1_amount(5_001))
            .is_err()
    );
}

#[test]
fn property_qbi_is_regular_only_even_with_different_amt_property_values() {
    let mut source = former_passive_property_scorp(0, 10_000, 0, 0);
    source.property_qbi = property_qbi(0, 5_000);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    source.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 200_000;
    assert!(source.valid());
    let alternate = source.amt.as_ref().unwrap().source(&source).unwrap();
    assert!(alternate.property_qbi.is_empty());
    let (d, id) = passive_property_pool(source, 0);
    let r = estimate(&d);
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.starts_with(&alloc::format!(
                "S-corporation document {id}, potential section 1231 QBI gain $5000."
            )))
            .count(),
        1
    );
    assert!(r.tax.is_none());
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn property_qbi_source_cannot_borrow_ordinary_histories_or_invent_qualification() {
    let mut source = passive_property_scorp(0, -6_000);
    source.property_qbi = property_qbi(3_000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, false).qbi_statement;
    assert!(source.valid());
    for case in 0..6 {
        let mut bad = source.clone();
        let qbi = &mut bad.property_qbi;
        match case {
            0 => qbi.losses.current = 600_001,
            1 => qbi.gain = 100,
            2 => qbi.losses.history = vec![loss_year(2024, 1_000, 1_000, 1_000, 1_000)],
            3 => qbi.losses.at_risk = vec![loss_year(2024, 1_000, 1_000, 1_000, 1_000)],
            4 => qbi.losses.passive = vec![loss_year(2017, 1_000, 100, 1_000, 100)],
            _ => bad.section1231 = None,
        }
        assert!(!bad.valid());
    }
    source.property_qbi.complete = false;
    assert!(source.valid());
    assert!(source.property_qbi.calculate(&source, &source.basis_result().unwrap()).is_err());
}

#[test]
fn property_qbi_confirmed_empty_stage_histories_trace_the_current_loss() {
    let mut source = passive_property_scorp(0, -6_000);
    source.property_qbi = property_qbi(3_000, 0);
    source.property_qbi.losses.at_risk = vec![];
    source.property_qbi.losses.passive = vec![];
    assert!(source.valid());
    let basis = source.basis_result().unwrap();
    assert!(source.property_qbi.calculate(&source, &basis).is_ok());
    assert!(source.property_qbi.after_at_risk(&source, &basis).is_ok());
    assert!(
        source
            .property_qbi
            .after_passive(&source, crate::tests::support::builders::k1_amount(6_000))
            .is_ok()
    );
}

#[test]
fn property_qbi_net_gain_excludes_constituent_losses_too() {
    let d = property_qbi_character_draft(20_000, 10_000);
    let items = property_qbi_character_items(&d).unwrap();
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|v| v.gain == 0 && v.loss == 0));
}

#[test]
fn property_qbi_recapture_and_missing_history_remain_explicit() {
    let mut d = property_qbi_character_draft(20_000, 10_000);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 2_000, 2_000)], &[]));
    assert!(property_qbi_character_items(&d).unwrap_err().contains("lookback recapture"));
    d.screening.as_mut().unwrap().section1231 = None;
    assert!(property_qbi_character_items(&d).is_err());
}

#[test]
fn property_qbi_ordinary_character_requires_source_and_complete_pool() {
    let mut d = property_qbi_character_draft(10_000, 20_000);
    let entry =
        d.income.iter_mut().find(|e| matches!(e.income, FederalIncome::Scorp { .. })).unwrap();
    if let FederalIncome::Scorp { data } = &mut entry.income {
        data.property_qbi = Default::default();
        data.qbi_statement = None;
    }
    // No statement and no property records: that K-1 adds no property QBI.
    assert_eq!(property_qbi_character_items(&d).unwrap().len(), 1);
    // With a statement, zero records are a value: that K-1's section 1231
    // gain has no qualified portion.
    let entry =
        d.income.iter_mut().find(|e| matches!(e.income, FederalIncome::Scorp { .. })).unwrap();
    if let FederalIncome::Scorp { data } = &mut entry.income {
        data.qbi_statement = Some(ScorpQbiStatement {
            ptp_income: 0,
            reit_dividends: 0,
            income: 0,
            wages: 0,
            property: 0,
            service: false,
            complete: true,
        });
        data.property_qbi.complete = true;
    }
    let items = property_qbi_character_items(&d).unwrap();
    assert_eq!(
        items.iter().map(|i| (i.id, i.gain, i.loss)).collect::<Vec<_>>(),
        vec![(2, 0, 0), (3, 0, 20_000)]
    );
    let mut d = property_qbi_character_draft(10_000, 20_000);
    let entry =
        d.income.iter_mut().find(|e| matches!(e.income, FederalIncome::Scorp { .. })).unwrap();
    if let FederalIncome::Scorp { data } = &mut entry.income {
        data.participation = None;
    }
    assert!(property_qbi_character_items(&d).is_err());
}

#[test]
fn property_qbi_character_uses_allowed_loss_and_retains_original_eligibility() {
    let mut source = former_passive_property_scorp(0, -2_000, 0, 0);
    source.section1231.as_mut().unwrap().basis_loss = 300_000;
    source.property_qbi = property_qbi(2_000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    source.property_qbi.losses.history = vec![loss_year(2017, 3_000, 0, 3_000, 0)];
    source.amt = Some(amt_records(&source));
    let (mut d, _) = passive_property_pool(source, 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    assert_eq!(lookback_worksheet(&d, false).unwrap().unwrap().net, -5_000);
    let items = property_qbi_character_items(&d).unwrap();
    assert_eq!((items[0].gain, items[0].loss), (0, 2_000));

    let mut source = passive_property_scorp(0, -10_000);
    source.property_qbi = property_qbi(10_000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, false).qbi_statement;
    let (mut d, _) = passive_property_pool(source, 4_000);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    assert_eq!(lookback_worksheet(&d, false).unwrap().unwrap().net, -4_000);
    let items = property_qbi_character_items(&d).unwrap();
    assert_eq!((items[0].gain, items[0].loss), (0, 4_000));
}

#[test]
fn former_passive_mixed_sign_qbi_release_keeps_current_loss_nonpassive() {
    let mut source = former_passive_property_scorp(10_000, -5_000, 4_000, 6_000);
    let mut qbi = property_qbi(5_000, 0);
    qbi.losses.passive = vec![loss_year(2023, 6_000, 3_000, 6_000, 3_000)];
    source.property_qbi = qbi;
    let qbi = &source.property_qbi;
    // $1k of property history is released by own net income. Current $5k
    // remains deductible even though $5k of prior property loss is suspended.
    let release =
        qbi.after_passive(&source, crate::tests::support::builders::k1_amount(5_000)).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (6_000, 5_500)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(release.remaining[0].remaining_qbi.get())
        ),
        (5_000, 2_500)
    );
    let (d, id) = passive_property_pool(source, 2_000);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert_eq!(pool.k1_section1231, vec![property_result(id, -8_000, 3_000, -6_000)]);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn ordinary_qbi_property_gain_releases_old_loss_without_becoming_qbi_income() {
    let mut source = former_passive_property_scorp(-5_000, 10_000, 6_000, 4_000);
    source.loss_qbi = sole_qbi_loss(5_000, 100_000, 5_000).loss_qbi;
    source.loss_qbi.passive = vec![loss_year(2022, 6_000, 3_000, 6_000, 3_000)];
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    reconcile_qbi_tracking(&mut source);
    let items = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::tests::support::builders::k1_amount(5_000), 0)
        .unwrap();
    assert_eq!(items.iter().map(|v| v.income).collect::<alloc::vec::Vec<_>>(), vec![-5_000, -500]);
    assert!(!items[1].active);
    assert_eq!(items[1].service, Some(false));
}

#[test]
fn property_qbi_current_source_survives_new_stage_vintages_without_absorbing_old_loss() {
    let mut source = former_passive_property_scorp(0, -6_000, 0, 2_000);
    source.opening = 500_000;
    source.section1231.as_mut().unwrap().basis_loss = 200_000;
    source.section1231.as_mut().unwrap().risk_loss = 100_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 300_000, profit: 0, loss: 0 };
    let mut qbi = property_qbi(3_000, 0);
    qbi.losses.history = vec![loss_year(2017, 2_000, 0, 2_000, 0)];
    qbi.losses.at_risk = vec![loss_year(2022, 1_000, 500, 1_000, 500)];
    qbi.losses.passive = vec![loss_year(2023, 2_000, 500, 2_000, 500)];
    source.property_qbi = qbi;
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    source.amt = Some(amt_records(&source));
    let qbi = &source.property_qbi;
    // Basis releases $1.5k current QBI; at-risk admits $600 current
    // plus $500 historical. Current nonpassive losses bypass passive limits.
    assert_eq!(
        crate::kernel::ratio::dollars(
            qbi.current_allowed(&source, crate::tests::support::builders::k1_amount(2_000))
                .unwrap()
        ),
        600
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            qbi.after_passive(&source, crate::tests::support::builders::k1_amount(2_000))
                .unwrap()
                .qbi
                .get()
        ),
        1_100
    );
    let (mut d, _) = passive_property_pool(source, 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let items = property_qbi_character_items(&d).unwrap();
    assert_eq!((items[0].loss, items[0].current_loss), (1_100, 600));
}

#[test]
fn property_qbi_passive_current_source_uses_shared_stage_qualified_fraction() {
    let mut source = passive_property_scorp(0, -6_000);
    source.opening = 500_000;
    source.section1231.as_mut().unwrap().basis_loss = 200_000;
    source.section1231.as_mut().unwrap().risk_loss = 100_000;
    source.section1231.as_mut().unwrap().passive_loss = 200_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 300_000, profit: 0, loss: 0 };
    let mut qbi = property_qbi(3_000, 0);
    qbi.losses.history = vec![loss_year(2017, 2_000, 0, 2_000, 0)];
    qbi.losses.at_risk = vec![loss_year(2022, 1_000, 500, 1_000, 500)];
    qbi.losses.passive = vec![loss_year(2023, 2_000, 500, 2_000, 500)];
    source.property_qbi = qbi;
    let qbi = &source.property_qbi;
    // Other passive income permits $3.5k: $2k old passive history and
    // $1.5k of the $3k entering this stage. Qualified new-stage release
    // is $550, of which $300 traces to current source-year QBI.
    assert_eq!(
        crate::kernel::ratio::dollars(
            qbi.after_passive(&source, crate::tests::support::builders::k1_amount(1_500))
                .unwrap()
                .qbi
                .get()
        ),
        1_050
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            qbi.current_allowed(&source, crate::tests::support::builders::k1_amount(1_500))
                .unwrap()
        ),
        300
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            qbi.current_allowed(&source, crate::tests::support::builders::k1_amount(5_000))
                .unwrap()
        ),
        0
    );
}

#[test]
fn combined_property_qbi_phases_current_gains_losses_and_limits_exactly_once() {
    let source = combined_property_sstb_source();
    let statement = source.qbi_statement.as_ref().unwrap();
    for (taxable, current, wages, basis) in
        [(201_750, 20_000, 20_000, 100_000), (239_250, 10_000, 10_000, 50_000), (276_750, 0, 0, 0)]
    {
        let items = statement
            .combined_activities(
                &source,
                [crate::kernel::carry::Amount::default(); 2],
                crate::section1231::PropertyCharacter::Ordinary,
                Some(taxable),
                FederalStatus::Single,
                0,
                0,
            )
            .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!((items[0].income, items[0].wages, items[0].property), (current, wages, basis));
        assert_eq!(items[1].income, -2_000);
        assert_eq!(items[1].service, Some(false));
        assert!(!items[1].active);
        assert!(items.iter().all(|v| v.service == Some(false)));
    }
}

#[test]
fn combined_property_qbi_rejects_unknown_tax_base_and_excludes_capital_items() {
    let source = combined_property_sstb_source();
    let statement = source.qbi_statement.as_ref().unwrap();
    assert!(
        statement
            .combined_activities(
                &source,
                [crate::kernel::carry::Amount::default(); 2],
                crate::section1231::PropertyCharacter::Ordinary,
                None,
                FederalStatus::Single,
                0,
                0,
            )
            .is_err()
    );
    let items = statement
        .combined_activities(
            &source,
            [crate::kernel::carry::Amount::default(); 2],
            crate::section1231::PropertyCharacter::Capital,
            Some(239_250),
            FederalStatus::Single,
            0,
            0,
        )
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].income, -5_000);
    assert_eq!(items[0].wages, 10_000);
    assert!(
        statement
            .combined_activities(
                &source,
                [
                    crate::tests::support::builders::k1_amount(1),
                    crate::kernel::carry::Amount::default()
                ],
                crate::section1231::PropertyCharacter::Ordinary,
                Some(239_250),
                FederalStatus::Single,
                0,
                0,
            )
            .is_err()
    );
}

#[test]
fn combined_property_qbi_produces_reconciled_final_totals() {
    let mut source = former_passive_property_scorp(10_000, -20_000, 0, 0);
    source.qbi_statement = complete_ordinary_scorp(10_000, true).qbi_statement;
    source.qbi_statement.as_mut().unwrap().wages = 1_000_000;
    source.property_qbi = property_qbi(20_000, 0);
    let (mut d, _) = passive_property_pool(source, 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("combined business QBI after eligibility: current $-10000; historical qualified losses $0; allocated wages $10000")));
    assert_eq!(r.tax, Some(382300));
    assert_eq!(value(&r, "agi"), 50000);
    assert_eq!(value(&r, "regular_income_tax"), 3823);
}

#[test]
fn rental_magi_property_projection_coordinates_portfolio_losses_and_lookback() {
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, 5_000, 0, 0), 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let portfolio = crate::capital::CapitalAmounts { long: -10_000, ..Default::default() };
    assert_eq!(
        crate::section1231::rental_magi_adjustment(&d, portfolio, [0, 0], false).unwrap(),
        -5_000
    );
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 2_000, 2_000)], &[]));
    assert_eq!(
        crate::section1231::rental_magi_adjustment(&d, portfolio, [0, 0], false).unwrap(),
        -3_000
    );
    assert_eq!(
        crate::section1231::rental_magi_adjustment(&d, Default::default(), [0, 0], false).unwrap(),
        0
    );
}

#[test]
fn rental_magi_property_projection_excludes_passive_sources_and_uses_regular_history() {
    let mut source = former_passive_property_scorp(0, 5_000, 0, 0);
    source.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 2_000_000;
    let (mut d, _) = passive_property_pool(source, 0);
    add(&mut d, income(FederalIncome::Scorp { data: passive_property_scorp(0, -20_000) }));
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[], &[(2025, 20_000, 20_000)]));
    let portfolio = crate::capital::CapitalAmounts { long: -10_000, ..Default::default() };
    // The passive loss cannot turn this projected nonpassive gain ordinary.
    assert_eq!(
        crate::section1231::rental_magi_adjustment(&d, portfolio, [0, 0], false).unwrap(),
        -5_000
    );
}

#[test]
fn rental_magi_property_projection_requires_history_only_for_net_gain() {
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, 5_000, 0, 0), 0);
    assert!(
        crate::section1231::rental_magi_adjustment(&d, Default::default(), [0, 0], false).is_err()
    );
    add(
        &mut d,
        income(FederalIncome::Scorp { data: former_passive_property_scorp(0, -6_000, 0, 0) }),
    );
    assert_eq!(
        crate::section1231::rental_magi_adjustment(&d, Default::default(), [0, 0], false).unwrap(),
        0
    );
}

#[test]
fn rental_magi_projection_handles_capital_carryover_and_nol_measurement() {
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, 5_000, 0, 0), 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    // Without property, $4k carryover gives a $3k deduction; with it, $1k income.
    assert_eq!(
        crate::section1231::rental_magi_adjustment(&d, Default::default(), [0, 4_000], false)
            .unwrap(),
        -1_000
    );
    // NOL measurement excludes that $3k deduction, so the income increase is $1k.
    assert_eq!(
        crate::section1231::rental_magi_adjustment(&d, Default::default(), [0, 4_000], true)
            .unwrap(),
        -4_000
    );
}

#[test]
fn rental_property_magi_projection_is_used_by_the_return_pipeline() {
    let (mut d, _) = passive_property_pool(former_passive_property_scorp(0, 5_000, 0, 0), 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(0, 10_000, GainTerm::Long), 10_000, 0)),
    );
    let r = estimate(&d);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.starts_with("Rental-allowance property-income adjustment: $-5000."))
    );
    assert!(!r.issues.iter().any(|i| i.section == "Property income for rental MAGI"));
    // The K-1 has no box 17 code V statement: it adds no QBI.
    assert!(noted(&r, "WARN_K1_QBI_NO_STATEMENT"), "{r:?}");
}

#[test]
fn completed_passive_property_loss_history_excludes_suspended_balance() {
    let mut source = passive_property_scorp(0, -20000);
    source.property_qbi = property_qbi(20000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, false).qbi_statement;
    let (mut draft, _) = passive_property_pool(source, 10000);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&draft);
    // Only $10,000 property loss is released by the other passive income.
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(return_line(&r, "8"), 0);
    assert_eq!(value(&r, "regular_income_tax"), 5023);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 10000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 10000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
    assert_eq!(value(&r, "eic_investment_income"), 0);
}

#[test]
fn unreleased_passive_history_does_not_block_trading_interest_return() {
    let draft = trading_with_unreleased_histories();
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 50000);
    assert_eq!(value(&r, "investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 15000);
    assert_eq!(value(&r, "regular_income_tax"), 4555);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 10000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 20000);
    assert_eq!(value(&r, "amt_section1231_current_loss_history"), 30000);
    assert_eq!(return_line(&r, "8"), -1000000);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn released_former_passive_expenses_still_require_investment_attribution() {
    let mut draft = trading_with_unreleased_histories();
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        data.section1231.as_mut().unwrap().net = -500000;
        data.property_qbi.losses.current = 500000;
        data.amt = Some(amt_records(data));
    }
    unsupported(&draft, "K-1 investment interest");
    unsupported(&draft, "Property investment interest");
}

#[test]
fn commercial_rental_is_no_longer_refused_and_residential_is_unchanged() {
    // Unit C19b: the rental's property type (Schedule E line 1b) now selects
    // 27.5-year (single-family/multi-family) or 39-year (commercial)
    // depreciation directly; the `RareSituation::NonresidentialRental`
    // catch-all narrows to special situations, not the ordinary case.
    //
    // rental()'s building: $330,000 cost, $55,000 land ($275,000
    // depreciable), in service January 2020, so 2026 is a full middle
    // recovery year for either period.
    let mut single_family = complete(FederalStatus::Single);
    add(&mut single_family, income(FederalIncome::Rental { data: rental(30_000, 5_000) }));
    // Type 1 (single-family, the default): unchanged 27.5-year GDS.
    // $275,000 x 24/660 = $10,000.00 (as before this unit).
    let r = supported(&single_family);
    assert_eq!(value(&r, "rental_2_depreciation"), 10_000);
    // A 2020 building needs no Form 4562 (i4562 "Who Must File"): its
    // depreciation is on Schedule E line 18 alone.
    assert!(
        r.lines.iter().all(|l| l.form_line.as_ref().is_none_or(|f| f.form != TaxForm::Form4562)),
        "{r:?}"
    );

    // Type 4 (commercial): now computed, no longer refused. 2026 is not the
    // first or closing year, so the flat years-2-39 rate applies:
    // $275,000 x 2.564% = $7,051.00.
    let mut commercial = single_family.clone();
    let FederalIncome::Rental { data } = &mut commercial.income[1].income else {
        panic!("expected the rental entry at index 1");
    };
    data.property_type = RentalPropertyType::Commercial;
    let r = supported(&commercial);
    assert_eq!(value(&r, "rental_2_depreciation"), 7_051);
    assert!(
        r.lines.iter().all(|l| l.form_line.as_ref().is_none_or(|f| f.form != TaxForm::Form4562)),
        "{r:?}"
    );

    for kind in [
        RentalPropertyType::VacationShortTerm { stays: crate::ShortTermStays::OverSevenDays },
        RentalPropertyType::Land,
        RentalPropertyType::SelfRental,
        RentalPropertyType::Other { description: "Storage building".into() },
    ] {
        let mut d = single_family.clone();
        let FederalIncome::Rental { data } = &mut d.income[1].income else { panic!() };
        data.property_type = kind;
        supported(&d);
    }
}

#[test]
fn rental_under_15_days_is_excluded() {
    // 14 fair rental days and 30 personal days: a home rented fewer than
    // 15 days. The $6,000 rent is not income and nothing is deducted; the
    // prior $300 operating carryover waits for a later year.
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 6000, 14, 30, [300, 0]);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_2_depreciation"), 0);
    assert_eq!(cents(&r, "rental_2_home_operating_carryover"), 30_000);
    assert_eq!(value(&r, "agi"), 60_000);

    // 15 fair rental days is a rental: rent 1,500 is reported. Share 15/45:
    // line 2 = 1,400; line 3 = 100; operating 800 + 300 = 1,100, allowed
    // 100, line 7a = 1,000; depreciation 1,333.33 all carried (line 7b).
    let mut d = complete(FederalStatus::Single);
    add_vacation_home(&mut d, 1500, 15, 30, [300, 0]);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_2_depreciation"), 0);
    assert_eq!(cents(&r, "rental_2_home_operating_carryover"), 100_000);
    assert_eq!(cents(&r, "rental_2_home_depreciation_carryover"), 133_333);
}

#[test]
fn rental_without_personal_use_is_unchanged() {
    // Same property as the passive-allowance test: rent 9,000, expenses
    // 30,000, depreciation 10,000 -> loss 31,000; $25,000 allowed at
    // $100,000 MAGI, 6,000 suspended. Zero personal days leave it unchanged.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 10_000_000, 0)];
    add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -25_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 6000);
    let FederalIncome::Rental { data } = &mut d.income[1].income else { unreachable!() };
    data.personal_use = crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days: 365,
        personal_days: 0,
        overlap_days: 0,
        direct_expenses: 0,
    };
    assert_eq!(supported(&d), r);
}

#[test]
fn real_estate_professional_rental_loss_is_nonpassive() {
    // Rent 0, expenses 30,000, plus the shared fixture's depreciation
    // (330,000 cost, 55,000 land, in service 2020: 275,000 x 24/660 =
    // 10,000/year) gives a $40,000 loss. Wages of $200,000 make the
    // rental-allowance MAGI $200,000 for this wage-only return.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 30_000) }));
    // Passive investor (Pub 925 special allowance): $25,000 - ($200,000 -
    // $100,000) / 2 = $0 allowance; the whole $40,000 loss is suspended and
    // AGI is unaffected by the rental.
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 40_000);
    assert_eq!(value(&r, "agi"), 200_000);

    // Qualified real-estate professional who materially participates
    // (confirmed hours): Sec. 469(c)(7) makes the loss nonpassive, with no
    // Form 8582 allowance or phaseout; it offsets the wages in full.
    make_professional(&mut d, Owner::Taxpayer, false);
    let FederalIncome::Rental { data } = &mut d.income[1].income else { unreachable!() };
    data.participation = crate::RentalParticipation::Professional;
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -40_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(value(&r, "agi"), 160_000);
}

#[test]
fn a_professional_rental_needs_the_owners_recorded_status() {
    let mut d = complete(FederalStatus::Single);
    let mut data = rental(9000, 30_000);
    data.participation = crate::RentalParticipation::Professional;
    add(&mut d, income(FederalIncome::Rental { data }));
    // Without the People page's status the rental is refused by name, with
    // a correction link to the owner's page.
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_RENTAL_PROFESSIONAL_NEEDS_STATUS"),
        "{r:?}"
    );
    make_professional(&mut d, Owner::Taxpayer, false);
    supported(&d);
}

#[test]
fn rental_days_cannot_exceed_365() {
    // Fair rental days plus personal-use days above 365 is a structural
    // bound (like a negative amount or an out-of-range building date), so it
    // fails the same accepted-data validity gate as those, not a named
    // calculation issue; matches the invalid-record precedent above.
    let mut d = complete(FederalStatus::Single);
    let mut v = rental(9000, 30_000);
    v.personal_use = crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days: 200,
        personal_days: 200,
        overlap_days: 0,
        direct_expenses: 0,
    };
    add(&mut d, income(FederalIncome::Rental { data: v }));
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Incomplete);
    assert!(r.issues.iter().any(|issue| issue.section == "Saved return"));
}

// Unit: matrix row 17, Schedule E page 1 typical case. One MFJ return, one
// W-2, one residential rental with no personal use, active participation and
// no real-estate-professional election: the form's ordinary path through
// Schedule E, Form 8582 and Form 1040. The building (in service March 2021)
// is reused unchanged by the $150,000-MAGI boundary test below.
fn schedule_e_typical_rental() -> FederalRental {
    use crate::extended_model::{RentalExpense, RentalExpenseCategory};
    let expense = |category: RentalExpenseCategory, amount: i64| RentalExpense {
        category,
        amount: amount * 100,
        treatment: BusinessExpenseKind::Ordinary,
    };
    FederalRental {
        // $220,000 depreciable basis (land excluded), in service March 2021:
        // by 2026 this is the building's 6th calendar year (2021-2026), so
        // `depreciation()`'s mid-month half-month count for 27.5-year GDS
        // clamps to the full 24 half-months (a full year's rate).
        // 220,000 x 24/660 = $8,000.
        rent: 24_000 * 100,
        expenses: vec![
            expense(RentalExpenseCategory::Insurance, 1_400),
            expense(RentalExpenseCategory::ManagementFees, 2_400),
            expense(RentalExpenseCategory::MortgageInterest, 9_600),
            expense(RentalExpenseCategory::Repairs, 1_800),
            expense(RentalExpenseCategory::TaxesAndLicenses, 3_200),
            expense(RentalExpenseCategory::Utilities, 600),
        ],
        buildings: vec![purchased_rental_building(220_000 * 100, 0, 20210301)],
        ..rental(0, 0)
    }
}

#[test]
fn residential_rental_return_matches_hand_computation() {
    // MFJ, spouses born 1976 and 1978 (well under 65; no senior deduction or
    // age-related threshold applies to either). One W-2: box 1 $128,000, box
    // 2 $17,000; `wage()` derives boxes 3-6 consistently (box 3/5 wages
    // $128,000, box 4 $7,936, box 6 $1,856 — not separately asserted, they do
    // not affect this return).
    //
    // Schedule E Part I, by hand from i1040se's 2026 draft line list:
    //   Line 3  Rents received: $24,000.
    //   Line 9  Insurance: $1,400.
    //   Line 11 Management fees: $2,400.
    //   Line 12 Mortgage interest paid to banks, etc.: $9,600.
    //   Line 14 Repairs: $1,800.
    //   Line 16 Taxes: $3,200.
    //   Line 17 Utilities: $600.
    //   Lines 5,6,7,8,10,13a,13b,15,19: not entered ($0).
    //   Line 18 Depreciation (Pub 527, 27.5-year SL, mid-month, full 2026
    //     year): $220,000 / 27.5 = $8,000.
    //   Line 20 Total expenses: 1,400+2,400+9,600+1,800+3,200+600+8,000 =
    //     $27,000.
    //   Line 21 Income or (loss): $24,000 - $27,000 = -$3,000.
    //   Line 22 Deductible rental real estate loss after Form 8582: Form
    //     8582's Worksheet 1 special allowance is active-participation
    //     $25,000, phased out 50 cents per dollar of MAGI over $100,000
    //     (i8582): MAGI here is AGI before the rental loss itself, i.e. the
    //     $128,000 of wages (no other income or adjustments), which is
    //     inside the $100,000-$150,000 phase-out band:
    //       $25,000 - 50% x ($128,000 - $100,000) = $25,000 - $14,000 =
    //       $11,000 allowance.
    //     The $3,000 loss is smaller than the $11,000 allowance, so the
    //     entire loss is currently deductible; nothing is suspended.
    //   Lines 23a-23e (cross-property aggregates by printed line) and the
    //     24/25 income-only/losses-only split: this return has one property
    //     and a loss, so line 24 (income) is $0 and line 25 (losses) is
    //     $3,000; both collapse into line 21/22's single signed net, which is
    //     all the App tracks (see the Schedule E map's "not line-preserving"
    //     scope note).
    //   Line 26 Total rental real estate and royalty income or (loss):
    //     -$3,000 (no royalties).
    //
    // Schedule 1 line 5 (rents, royalties...): -$3,000, carried to Form 1040
    // line 8 (the only Schedule 1 additional-income item on this return).
    //
    // Form 1040:
    //   Line 8  Additional income (Schedule 1): -$3,000.
    //   Line 9  Total income: $128,000 - $3,000 = $125,000.
    //   Line 11 Adjusted gross income: $125,000 (no Schedule 1 adjustments).
    //   Line 12e Standard deduction (MFJ, neither 65+/blind, not a
    //     dependent): $32,200.
    //   Line 15 Taxable income: $125,000 - $32,200 = $92,800.
    //   Line 16 Tax: under $100,000, so the September 2026 draft Tax Table
    //     row "at least $92,800 but less than $92,850" applies, midpoint
    //     $92,825. 2026 MFJ brackets: 10% to $24,800, 12% to $100,800:
    //       10% x $24,800 + 12% x ($92,825 - $24,800)
    //       = $2,480 + 12% x $68,025 = $2,480 + $8,163 = $10,643.
    //   Line 24a Total tax: $10,643 (no credits or additional taxes apply).
    //   Line 25a Federal income tax withheld (W-2): $17,000.
    //   Line 33 Total payments: $17,000.
    //   Line 34 Overpayment (refund): $17,000 - $10,643 = $6,357.
    //   Line 37 Balance due: $0 (an overpayment, not a balance due).
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19760610;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19780815;
    d.income = vec![wage(1, Owner::Taxpayer, 12_800_000, 1_700_000)];
    add(&mut d, income(FederalIncome::Rental { data: schedule_e_typical_rental() }));
    let r = supported(&d);

    assert_eq!(value(&r, "rental_2_expense_line9_insurance"), 1_400);
    assert_eq!(value(&r, "rental_2_expense_line11_management_fees"), 2_400);
    assert_eq!(value(&r, "rental_2_expense_line12_mortgage_interest"), 9_600);
    assert_eq!(value(&r, "rental_2_expense_line14_repairs"), 1_800);
    assert_eq!(value(&r, "rental_2_expense_line16_taxes"), 3_200);
    assert_eq!(value(&r, "rental_2_expense_line17_utilities"), 600);
    assert_eq!(value(&r, "rental_2_depreciation"), 8_000);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "rental_income"), -3_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(value(&r, "agi"), 125_000);

    assert_eq!(return_line(&r, "8"), -300_000);
    assert_eq!(return_line(&r, "9"), 12_500_000);
    assert_eq!(return_line(&r, "11a / 11b"), 12_500_000);
    assert_eq!(return_line(&r, "12e"), 3_220_000);
    assert_eq!(return_line(&r, "15"), 9_280_000);
    assert_eq!(return_line(&r, "16"), 1_064_300);
    assert_eq!(return_line(&r, "24a"), 1_064_300);
    assert_eq!(return_line(&r, "25a"), 1_700_000);
    assert_eq!(return_line(&r, "33"), 1_700_000);
    assert_eq!(return_line(&r, "34"), 635_700);
    assert_eq!(return_line(&r, "37"), 0);
}

#[test]
fn passive_allowance_is_zero_at_150000_agi() {
    // Same property as the typical-case fixture above (a $3,000 loss before
    // Form 8582), on a joint return with wages equal to the $150,000 MAGI
    // ceiling named in i8582's phase-out (Pub 925): allowance =
    // $25,000 - 50% x ($150,000 - $100,000) = $25,000 - $25,000 = $0. With no
    // allowance, none of the $3,000 loss is currently deductible; the whole
    // amount is suspended and AGI is unaffected by the rental.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 15_000_000, 0)];
    add(&mut d, income(FederalIncome::Rental { data: schedule_e_typical_rental() }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 3_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 3_000);
    assert_eq!(value(&r, "agi"), 150_000);
}

#[test]
fn rental_profit_is_reported_without_form_8582() {
    // Form 8582's special allowance and MAGI phase-out (Pub 925) limit a
    // passive loss; they have no effect on a passive activity's net profit,
    // which is simply Schedule E/Schedule 1 line 5 income. Wages of $128,000
    // put this return's MAGI in the same $100,000-$150,000 phase-out band as
    // the two tests above, but since this property is profitable there is no
    // allowance to compute and no suspended loss.
    //
    // Net profit, using the shared `rental()` fixture building (330,000 cost,
    // 55,000 land, in service 2020, a middle recovery year: (330,000-55,000)
    // x 24/660 = $10,000 depreciation): rent $30,000 - expenses $5,000 -
    // depreciation $10,000 = $15,000.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 12_800_000, 0)];
    add(&mut d, income(FederalIncome::Rental { data: rental(30_000, 5_000) }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 15_000);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "agi"), 143_000);
}

#[test]
fn two_rentals_with_prior_unallowed_losses_split_the_phased_out_allowance() {
    // Form 8582 hand fixture, MFJ. One W-2: box 1 $130,000, withholding
    // $18,000 (round numbers; the W-2's derived boxes 3-6 do not affect this
    // return). Two active-participation residential rentals in the shared
    // Form 8582 pool, each carrying a prior-year Form 8582 regular-tax
    // unallowed loss into this year's Part I, on a return whose MAGI sits
    // inside the $100,000-$150,000 phase-out band (i8582 Worksheet 1; Pub
    // 925).
    //
    // Both rentals share the `rental()` fixture building (Pub 527, 27.5-year
    // SL, mid-month convention, a middle recovery year already used by the
    // neighbouring tests above: (330,000-55,000) land-excluded basis x
    // 24/660 half-months = $10,000 depreciation each).
    //
    // Rental A (id 2): rent $5,000, one $5,000 "Other" expense, $10,000
    //   depreciation: current-year net = 5,000 - 5,000 - 10,000 = -$10,000.
    //   Prior Form 8582 regular-tax unallowed loss carried into Part I:
    //   $5,000. Activity's total current+prior loss: $10,000 + $5,000 =
    //   $15,000.
    // Rental B (id 3): rent $9,000, one $3,000 "Other" expense, $10,000
    //   depreciation: current-year net = 9,000 - 3,000 - 10,000 = -$4,000.
    //   Prior Form 8582 regular-tax unallowed loss carried into Part I:
    //   $5,000. Activity's total current+prior loss: $4,000 + $5,000 =
    //   $9,000.
    // Combined Part I loss: $15,000 + $9,000 = $24,000. Part II line 3 (other
    //   passive income) is $0, so none of the loss is offset before the
    //   allowance.
    //
    // MAGI (Worksheet 1; excludes the rental loss itself, per the App's
    //   `rental_magi` line) = $130,000 wages, inside the phase-out:
    //   allowance = $25,000 - 50% x ($130,000 - $100,000)
    //             = $25,000 - $15,000 = $10,000.
    // The $24,000 total loss exceeds the $10,000 allowance, so Worksheet 5
    //   (Part VIII) allocates the allowance in proportion to each activity's
    //   own loss out of the $24,000 total:
    //     A: $10,000 x $15,000/$24,000 = $6,250.
    //     B: $10,000 x $9,000/$24,000 = $3,750 (= $10,000 - $6,250).
    //   Worksheet 7 (Part IX) then allocates the $14,000 left over
    //   ($24,000 - $10,000) in proportion to each activity's own loss after
    //   its Worksheet 5 share; since nothing else competes for that $14,000,
    //   each activity's own remainder is exactly its unallowed loss:
    //     A: $15,000 - $6,250 = $8,750 suspended (A's allowed loss this year
    //       is $15,000 - $8,750 = $6,250, equal to its Worksheet 5 share).
    //     B: $9,000 - $3,750 = $5,250 suspended (allowed $9,000 - $5,250 =
    //       $3,750, equal to its Worksheet 5 share).
    // Total allowed loss: $6,250 + $3,750 = $10,000 (equal to the allowance,
    //   as it must be whenever the pool's total loss exceeds it and there is
    //   no other passive income). Total suspended: $8,750 + $5,250 =
    //   $14,000.
    //
    // Schedule 1 line 5 / Form 1040 line 8: -$10,000 (rents/royalties, the
    //   allowed loss).
    // Form 1040:
    //   Line 9  Total income: $130,000 - $10,000 = $120,000.
    //   Line 11 AGI: $120,000 (no Schedule 1 adjustments).
    //   Line 12e Standard deduction (MFJ, neither 65+/blind, not a
    //     dependent): $32,200.
    //   Line 15 Taxable income: $120,000 - $32,200 = $87,800.
    //   Line 16 Tax: under $100,000, so the September 2026 draft Tax Table
    //     row "at least $87,800 but less than $87,850" applies, midpoint
    //     $87,825. 2026 MFJ brackets: 10% to $24,800, 12% to $100,800:
    //       10% x $24,800 + 12% x ($87,825 - $24,800)
    //       = $2,480 + 12% x $63,025 = $2,480 + $7,563 = $10,043.
    //   Line 24a Total tax: $10,043 (no other credits or additional taxes).
    //   Line 25a Federal income tax withheld: $18,000.
    //   Line 33 Total payments: $18,000.
    //   Line 34 Overpayment (refund): $18,000 - $10,043 = $7,957.
    //   Line 37 Balance due: $0.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 13_000_000, 1_800_000)];
    let mut a = rental(5_000, 5_000);
    a.history = RentalHistory { regular: 500_000, amt: 500_000, ..RentalHistory::NONE };
    add(&mut d, income(FederalIncome::Rental { data: a }));
    let mut b = rental(9_000, 3_000);
    b.history = RentalHistory { regular: 500_000, amt: 500_000, ..RentalHistory::NONE };
    add(&mut d, income(FederalIncome::Rental { data: b }));
    let r = supported(&d);

    assert_eq!(value(&r, "rental_magi"), 130_000);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 8_750);
    assert_eq!(value(&r, "rental_3_suspended_loss"), 5_250);
    assert_eq!(value(&r, "rental_income"), -10_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 14_000);
    assert_eq!(value(&r, "agi"), 120_000);

    assert_eq!(return_line(&r, "8"), -1_000_000);
    assert_eq!(return_line(&r, "9"), 12_000_000);
    assert_eq!(return_line(&r, "11a / 11b"), 12_000_000);
    assert_eq!(return_line(&r, "12e"), 3_220_000);
    assert_eq!(return_line(&r, "15"), 8_780_000);
    assert_eq!(return_line(&r, "16"), 1_004_300);
    assert_eq!(return_line(&r, "24a"), 1_004_300);
    assert_eq!(return_line(&r, "25a"), 1_800_000);
    assert_eq!(return_line(&r, "33"), 1_800_000);
    assert_eq!(return_line(&r, "34"), 795_700);
    assert_eq!(return_line(&r, "37"), 0);
}

#[test]
fn rental_loss_without_active_participation_is_fully_suspended() {
    // Sec. 469(i)/i8582: the special allowance applies only to a loss
    // activity in which the taxpayer actively participates (Pub 925). An
    // active-participation loss can be allowed by the allowance even with no
    // other passive income; without active participation the loss can still
    // be offset only by other passive income, and with none here ($0 of
    // Part II line 3) the entire loss is suspended even though $50,000 of
    // wages sits far below the $100,000 floor of the single-filer phase-out
    // band that would otherwise allow the full $25,000. Same $31,000-loss
    // activity (`rental(9000, 30000)`, no prior history) as the tests above,
    // with `active_participation` answered No instead of the fixture's
    // default Yes.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 5_000_000, 0)];
    let mut v = rental(9_000, 30_000);
    v.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: v }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 31_000);
    assert_eq!(value(&r, "agi"), 50_000);
}

#[test]
fn rental_sale_overall_loss_is_nonpassive_at_the_469g_boundary() {
    let mut other = rental(6_000, 14_000);
    other.participation = crate::RentalParticipation::NotActive;
    // Prior loss $113,583: $1,750 - $113,583 + $111,833 = $0, an overall
    // gain of zero, still passive: nothing offsets the other rental's
    // $18,000 loss, which stays suspended.
    let r = supported(&rental_sale_return(vec![sold_rental(113_583), other.clone()]));
    // Line 22 holds the whole $113,583 prior loss (the $1,750 of line 21 income
    // uses part of it); lines 21 and 22 net to $111,833.
    assert_eq!(value(&r, "rental_2_line22_allowed_loss"), -113_583);
    assert_eq!(value(&r, "rental_3_suspended_loss"), 18_000);
    // Prior loss $113,584: an overall loss of $1, nonpassive and allowed in
    // full; the other rental's loss still has no passive income.
    let r = supported(&rental_sale_return(vec![sold_rental(113_584), other]));
    assert_eq!(value(&r, "rental_2_line22_allowed_loss"), -113_584);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "rental_3_suspended_loss"), 18_000);
    // AGI: $60,000 wages - $111,834 + $111,833 = $59,999.
    assert_eq!(value(&r, "agi"), 59_999);
}

#[test]
fn rental_unrecaptured_1250_gain_is_the_smaller_of_gain_and_depreciation() {
    // Sold for $300,000 ($80,000 land), no selling expenses: building line
    // 24 $220,000 - $209,167 = $10,833, below its $65,833 depreciation.
    let mut property = sold_rental(0);
    property.sale = crate::RentalSale::Sold {
        date: 20260815,
        price: 300_000 * 100,
        land_price: 80_000 * 100,
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let r = supported(&rental_sale_return(vec![property]));
    assert_eq!(value(&r, "rental_2_sale_building_gain"), 10_833);
    assert_eq!(value(&r, "rental_2_sale_unrecaptured_1250"), 10_833);
    assert_eq!(value(&r, "rental_2_sale_land"), 25_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 10_833);
    assert_eq!(value(&r, "form4797_line8"), 35_833);
}

#[test]
fn rental_sales_outside_the_calculation_are_refused_by_name() {
    // A part replaced after the whole property was sold (or on an exchanged
    // rental) is not a part of anything: refused by name. A year-old improvement
    // and personal use are calculated now (unit B11 4797).
    let mut property = sold_rental(0);
    property.buildings[0].partial =
        Some(crate::PartialDisposition { date: 20261001, cost_of_part: 1_000_000, proceeds: 0 });
    let r = estimate(&rental_sale_return(vec![property]));
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_RENTAL_PARTIAL_WITH_DISPOSITION"));
    let mut property = sold_rental(0);
    property.buildings.push(purchased_rental_building(1_000_000, 0, 20250901));
    supported(&rental_sale_return(vec![property]));
}

// IRC 469(i)(3)(E)(ii), 86(b)(2)(A) and 219(g)(3)(A)(ii) read AGI "without
// regard to" section 135, so the passive-loss allowance, taxable Social
// Security and the IRA deduction all see the savings bond interest gross, as
// the i8815 line 9 worksheet does (Schedule B line 2 before the exclusion).
// Only Form 8815's own AGI and later consumers see the net. Pins that split.
//   Single; wages $100,000; box 3 $3,000 (all excludable: her own tuition
//   $10,000 exceeds the $3,000 proceeds); a rental with a $26,000 loss.
//   Allowance MAGI = 100,000 + 3,000 gross = 103,000 (the loss, the Social
//   Security and the IRA deduction excluded): 25,000 - 50% x 3,000 = 23,500,
//   so 2,500 is suspended (net interest would have allowed all 25,000).
//   Form 8815 line 9 = AGI before the exclusion = 100,000 + 3,000 - 23,500 =
//   79,500, under 101,800: the exclusion is 3,000. AGI = 100,000 + 3,000 -
//   3,000 - 23,500 = 76,500.
#[test]
fn passive_allowance_reads_the_savings_bond_interest_gross() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19750101;
    add(&mut d, bond_interest_entry(3_000));
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 16_000) }));
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 3_000, 3_000, 20050101, vec![bond_student(-1, 10_000, 0)]),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -23_500);
    assert_eq!(value(&r, "rental_suspended_loss"), 2_500);
    assert_eq!(value(&r, "form_8815_modified_agi"), 79_500);
    assert_eq!(value(&r, "savings_bond_exclusion"), 3_000);
    assert_eq!(value(&r, "agi"), 76_500);
}

#[test]
fn professional_participation_is_chosen_for_each_rental() {
    // $200,000 wages leave no special allowance. The status is recorded, but
    // only the rental marked Professional is nonpassive: its -20,000 offsets
    // the wages; the Active rental's -20,000 is wholly suspended.
    let mut d = rental_loss_return(200_000);
    make_professional(&mut d, Owner::Taxpayer, false);
    let mut professional = rental(0, 10_000);
    professional.participation = crate::RentalParticipation::Professional;
    add(&mut d, income(FederalIncome::Rental { data: professional }));
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 10_000) }));
    let r = supported(&d);
    let active_id = d.income[2].id;
    assert_eq!(value(&r, &alloc::format!("rental_{active_id}_suspended_loss")), 20_000);
    assert_eq!(value(&r, "agi"), 200_000 - 20_000);
}

#[test]
fn an_allowed_professional_rental_loss_is_left_out_of_the_form_8582_magi() {
    // Wages 140,000. A professional rental loses 30,000 (rent 0, expenses
    // 20,000, depreciation 10,000) and is allowed in full; i8582 line 6 and
    // Pub 925 leave that allowed loss out, so MAGI stays 140,000 and the
    // allowance for the active rental is 25,000 - (140,000 - 100,000) / 2 =
    // 5,000. The active rental loses 20,000: 5,000 is allowed and 15,000 is
    // suspended. (The loss included would give MAGI 110,000, an allowance of
    // 20,000 and nothing suspended.)
    let mut d = rental_loss_return(140_000);
    make_professional(&mut d, Owner::Taxpayer, false);
    let mut professional = rental(0, 20_000);
    professional.participation = crate::RentalParticipation::Professional;
    add(&mut d, income(FederalIncome::Rental { data: professional }));
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 10_000) }));
    let r = supported(&d);
    let active_id = d.income[2].id;
    assert_eq!(value(&r, "rental_magi"), 140_000);
    assert_eq!(value(&r, &alloc::format!("rental_{active_id}_suspended_loss")), 15_000);
    assert_eq!(value(&r, "agi"), 140_000 - 30_000 - 5_000);
}

#[test]
fn form_8582_part_i_columns_split_active_and_other_activities() {
    // F3 (2026-10-09): lines 1a-1c and 2a-2c carry the Part IV and V columns.
    // Rental A is active now: current loss -20,000 (expenses 10,000 and
    // depreciation 10,000) with a 30,000 prior loss from a year without
    // active participation, which goes to Part V, not Part IV. Rental B is
    // not active: current income 5,000 (rent 25,000) with a 2,000 prior
    // loss.
    let mut d = rental_loss_return(60_000);
    let mut a = rental(0, 10_000);
    a.history = RentalHistory { regular: 3_000_000, amt: 3_000_000, active_in_loss_year: false };
    add(&mut d, income(FederalIncome::Rental { data: a }));
    let mut b = rental(25_000, 10_000);
    b.participation = crate::RentalParticipation::NotActive;
    b.history = RentalHistory { regular: 200_000, amt: 200_000, ..RentalHistory::NONE };
    add(&mut d, income(FederalIncome::Rental { data: b }));
    let r = supported(&d);
    for (line, expected) in [
        ("1a", 0),
        ("1b", -20_000),
        ("1c", 0),
        ("1d", -20_000),
        ("2a", 5_000),
        ("2b", 0),
        ("2c", -32_000),
        ("2d", -27_000),
        ("3", -47_000),
    ] {
        let id = alloc::format!("form8582_line_{line}");
        assert_eq!(value(&r, &id), expected, "{id}");
        assert_eq!(
            printed_line(&r, &id),
            (TaxForm::Form8582, alloc::string::ToString::to_string(line))
        );
    }
}

#[test]
fn f8_condemned_rental_recognizes_gain_above_the_replacement_cost() {
    // Building $300,000 (land $60,000) in service 2017-01-01, 27.5-year mid-month:
    // 23 + 8 x 24 + 13 half-months through July 15, 2026 = 228 / 660 x $240,000 =
    // $82,909 allowed. Award $400,000 ($90,000 for the land), expenses $5,000 by award
    // share ($3,875 building, $1,125 land). Building gain 310,000 - 3,875 - (240,000 -
    // 82,909) = $149,034; land 90,000 - 1,125 - 60,000 = $28,875; total $177,909. Amount
    // realized $395,000 less $380,000 spent = $15,000 recognized, $162,909 postponed,
    // divided by gain: building $12,565, land $2,435.
    let mut property = rental(20_000, 5_000);
    property.buildings[0].cost = 300_000 * 100;
    property.buildings[0].land = 60_000 * 100;
    property.buildings[0].in_service = 20170101;
    property.sale = crate::RentalSale::Condemned {
        date: 20260715,
        award: 400_000 * 100,
        land_award: 90_000 * 100,
        expenses: 5_000 * 100,
        replacement_cost: 380_000 * 100,
    };
    let r = supported(&rental_sale_return(vec![property]));
    for (suffix, amount) in [
        ("building_depreciation", 82_909),
        ("building_cost", 243_875),
        ("building_gain", 149_034),
        ("land", 2_435),
        ("postponed", 162_909),
        ("unrecaptured_1250", 12_565),
    ] {
        assert_eq!(value(&r, &alloc::format!("rental_2_sale_{suffix}")), amount, "{suffix}");
    }
    assert_eq!(value(&r, "form4797_line8"), 15_000);
    assert!(r.review_notes.iter().any(|n| n.display.key == "NOTE_CONDEMNATION_RECAPTURE_LIMITED"));
}

#[test]
fn unit40_professional_takes_the_partial_loss_in_full() {
    // Materially participating professional: $9,717 + $7,100 is all
    // nonpassive loss despite $200,000 wages. AGI $200,000-$16,817=$183,183.
    let mut property = unit40_roof(0, 0, 0);
    property.participation = crate::RentalParticipation::Professional;
    let mut d = unit40_return(200_000, property);
    make_professional(&mut d, Owner::Taxpayer, false);
    let r = supported(&d);
    for (id, expected) in [
        ("rental_income", -9_717),
        ("rental_2_line22_allowed_loss", -9_717),
        ("rental_2_building_0_partial_gain", -7_100),
        ("form4797_line1c", 7_100),
        ("form4797_line8", -7_100),
        ("agi", 183_183),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert!(!r.lines.iter().any(|row| row.id == "form8582_part_ix_2_section1231_unallowed"));
}

#[test]
fn unit40_at_risk_limit_precedes_partial_disposition_passive_allocation() {
    // Combined loss $9,717 + $7,100 = $16,817. At-risk capacity $5,000
    // admits $2,889 ordinary and $2,111 property; $11,817 stays at risk.
    // MAGI $140,000 admits that entire $5,000 through the special allowance,
    // leaving no PAL carry and AGI $135,000. Form4797 1c includes only $2,111.
    let mut property = unit40_roof(0, 0, 0);
    property.at_risk = simplified_rental_risk(5_000 * 100, 0, 0, [0, 0, 0, 0]);
    let r = supported(&unit40_return(140_000, property));
    for (id, expected) in [
        ("rental_2_at_risk_carryover", 11_817),
        ("rental_2_at_risk_allowed_ordinary", -2_889),
        ("rental_2_at_risk_allowed_section1231", -2_111),
        ("rental_2_suspended_loss", 0),
        ("rental_income", -2_889),
        ("rental_2_building_0_partial_gain", -2_111),
        ("form4797_line1c", 2_111),
        ("agi", 135_000),
    ] {
        let actual = if id.contains("at_risk_") {
            // At-risk source rows retain cents; round their displayed dollars
            // rather than truncating the $11,816.67 retained source balance.
            crate::kernel::ratio::dollars(line_amount(&r, id))
        } else {
            value(&r, id)
        };
        assert_eq!(actual, expected, "{id}");
    }
}
