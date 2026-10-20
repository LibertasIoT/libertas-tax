// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn rental_qbi_property_limit_excludes_land_and_expired_recovery() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    let mut v = rental(100000, 5000);
    v.qbi_business = RentalBusiness::Yes { material_participation: false };
    add(&mut d, income(FederalIncome::Rental { data: v }));
    assert_eq!(value(&supported(&d), "qbi_deduction"), 6875);
    let FederalIncome::Rental { data: v } = &mut d.income[1].income else { panic!() };
    v.buildings[0].in_service = 19990101;
    assert_eq!(value(&supported(&d), "qbi_deduction"), 0);
}

#[test]
fn prior_qbi_loss_offsets_current_business_with_proportional_wage_limits() {
    use crate::business_rules::{QbiActivity, QbiOther, qbi_with_history};
    let activities = [
        QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: None,
            owner: None,
            income: 60000,
            wages: 20000,
            property: 0,
            active: false,
            service: Some(false),
            group: None,
        },
        QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: None,
            owner: None,
            income: 40000,
            wages: 0,
            property: 0,
            active: false,
            service: Some(false),
            group: None,
        },
    ];
    let mut r = supported(&complete(FederalStatus::Single));
    let result = qbi_with_history(
        &activities,
        300000,
        0,
        FederalStatus::Single,
        QbiOther { business_loss: 50000, reit: 10000, reit_loss: 0 },
        &mut r,
    );
    // Carry is split 30,000 / 20,000. Only the first activity passes the wage limit.
    assert_eq!(result.deduction, 8000);
    assert_eq!(result.business_loss, 0);
    let result = qbi_with_history(
        &activities,
        300000,
        0,
        FederalStatus::Single,
        QbiOther { business_loss: 120000, reit: 0, reit_loss: 3000 },
        &mut r,
    );
    assert_eq!(result.deduction, 0);
    assert_eq!(result.business_loss, 20000);
    assert_eq!(result.reit_loss, 3000);
}

#[test]
fn personal_rental_prior_and_current_losses_share_other_passive_profit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent_with_history(2000, 5000, 4000, 6000, false)));
    add(&mut d, income(personal_rent(3000, 1000)));
    let r = supported(&d);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 5000);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 5000);
    assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), 7000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "amt_rental_adjustment"), 0);
    assert_eq!(value(&r, "rental_income"), 0);
}

#[test]
fn personal_rental_prior_history_rejects_invalid_balances() {
    let mut d = complete(FederalStatus::Single);
    let rent = personal_rent_with_history(12000, 2000, 8000, 12000, false);
    add(&mut d, income(rent));
    let FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental { history, .. },
        ..
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *history = PersonalRentalHistory { regular: -1, amt: 0 };
    assert!(!d.valid_stored());
}

#[test]
fn former_passive_property_current_net_loss_stays_outside_prior_loss_pool() {
    let source = former_passive_property_scorp(-8_000, 3_000, 6_000, 4_000);
    let activity = source.pool_activity_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(activity.passive), -10_000);
    assert_eq!(crate::kernel::ratio::dollars(activity.nonpassive), -5_000);
    assert_eq!(activity.prior_released, 0);
    let (d, id) = passive_property_pool(source, 5_000);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert_eq!(pool.passive_loss, 5_000);
    assert_eq!(pool.income, -6_000); // -8k current, -3k old ordinary, +5k other.
    assert_eq!(pool.scorp_nonpassive, -8_000);
    assert_eq!(pool.scorp_niit_adjustment, 8_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, 1_000, 2_000, 3_000)]);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn ordinary_qbi_property_limits_preserve_historical_sstb_eligibility() {
    let mut source = former_passive_property_scorp(-5_000, 10_000, 6_000, 4_000);
    source.loss_qbi = sole_qbi_loss(5_000, 100_000, 5_000).loss_qbi;
    source.loss_qbi.passive = vec![loss_year(2022, 6_000, 3_000, 6_000, 3_000)];
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    source.qbi_statement.as_mut().unwrap().service = true;
    reconcile_qbi_tracking(&mut source);
    let statement = source.qbi_statement.as_ref().unwrap();
    let prepared = statement.prepare(&source, 276_750, FederalStatus::Single).unwrap();
    let items = statement
        .activities(&prepared, crate::tests::support::builders::k1_amount(5_000), 0)
        .unwrap();
    assert_eq!(items.iter().map(|v| v.income).collect::<alloc::vec::Vec<_>>(), vec![0, -500]);
}

#[test]
fn property_qbi_capital_exclusion_clears_current_and_historical_components() {
    let d = property_qbi_character_draft(20_000, 10_000);
    let items = property_qbi_character_items(&d).unwrap();
    assert!(items.iter().all(|v| v.current_loss == 0 && v.loss == 0));
}
