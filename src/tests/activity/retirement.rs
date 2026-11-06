use super::*;

#[test]
fn separate_rental_allowance_requires_living_apart_all_year() {
    let mut d = complete(FederalStatus::Separate);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 10000) }));
    assert_eq!(value(&supported(&d), "rental_income"), 0);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::AllYear };
    assert_eq!(value(&supported(&d), "rental_income"), -7500);
}

#[test]
fn prior_rental_losses_have_separate_regular_and_amt_balances() {
    let mut d = complete(FederalStatus::Single);
    let mut v = rental(30000, 5000); // Current profit: 15,000.
    v.history = RentalHistory { regular: 2_500_000, amt: 3_000_000, ..RentalHistory::NONE };
    add(&mut d, income(FederalIncome::Rental { data: v }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -10000);
    assert_eq!(value(&r, "amt_rental_income"), -15000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(value(&r, "amt_rental_adjustment"), -5000);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 10000);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 15000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qbi_carryforward_categories_remain_separate_and_do_not_change_agi() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(reit_income(10000, 10000, true)));
    d.screening.as_mut().unwrap().qbi_history =
        QbiHistory { business: 3000000, reit_ptp: 400000, injured_owners: None };
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(value(&r, "qbi_deduction"), 1200);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 30000);
    assert_eq!(value(&r, "reit_ptp_loss_carryforward"), 0);
    d.screening.as_mut().unwrap().qbi_history =
        QbiHistory { business: 0, reit_ptp: 1500000, injured_owners: None };
    add(&mut d, income(cash_business(50000)));
    let r = supported(&d);
    assert_eq!(value(&r, "reit_ptp_loss_carryforward"), 5000);
    let mut no_reit_history = d.clone();
    no_reit_history.income[1].income = reit_income(10000, 0, true);
    no_reit_history.screening.as_mut().unwrap().qbi_history = QbiHistory::NONE;
    assert_eq!(value(&r, "qbi_deduction"), value(&supported(&no_reit_history), "qbi_deduction"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_personal_rental_expenses_release_against_own_profit_with_separate_amt_balance() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent_with_history(12000, 2000, 8000, 12000, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 10000);
    assert_eq!(value(&r, "agi"), 62000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), 2000);
    assert_eq!(value(&r, "amt_rental_adjustment"), -2000);
    assert_eq!(value(&r, "rental_magi"), 60000);
    assert_eq!(value(&r, "amt_rental_magi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_personal_rental_losses_reduce_recharacterized_investment_income_separately_for_amt() {
    for (regular, amt, deduction, amt_deduction, agi) in
        [(8000, 12000, 2000, 0, 62000), (12000, 8000, 0, 2000, 60000)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(personal_rent_with_history(12000, 2000, regular, amt, true)));
        investment_borrowing(&mut d, 4000, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), deduction);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), amt_deduction);
        assert_eq!(value(&r, "investment_interest_carryforward"), 4000 - deduction);
        assert_eq!(value(&r, "amt_investment_interest_carryforward"), 4000 - amt_deduction);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "rental_magi"), agi);
        assert_eq!(value(&r, "amt_rental_magi"), agi);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn personal_rental_recharacterized_investment_income_is_net_of_depreciation() {
    let mut d = complete(FederalStatus::Single);
    let mut rent =
        personal_rent_with_assets(10000, 1000, vec![rental_equipment(1000, BonusElection::Apply)]);
    let FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental { basis, .. },
        ..
    } = &mut rent
    else {
        unreachable!()
    };
    *basis = 1000000; // Only 10% depreciable: positive net income is recharacterized.
    add(&mut d, income(rent));
    investment_borrowing(&mut d, 10000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rent_2_investment_income"), 8000);
    assert_eq!(value(&r, "investment_interest_deduction"), 8000);
    assert_eq!(value(&r, "agi"), 68000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_personal_rental_amt_depreciation_flows_through_passive_limits_once() {
    for (receipts, regular_closing, amt_closing, adjustment, agi) in
        [(500, 155, 374, 0, 60000), (1000, 0, 0, -219, 60345)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(personal_rent_with_assets(receipts, 0, vec![separate_amt_equipment()])));
        let r = supported(&d);
        assert_eq!(value(&r, "personal_rental_2_suspended_loss"), regular_closing);
        assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), amt_closing);
        assert_eq!(value(&r, "amt_rental_adjustment"), adjustment);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "amt_rental_magi"), 60000);
        assert_eq!(
            r.lines
                .iter()
                .find(|l| l.id == "personal_rental_2_asset_0_amt_remaining_basis")
                .unwrap()
                .amount,
            43697
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent_with_assets(500, 0, vec![separate_amt_equipment()])));
    add(&mut d, income(FederalIncome::Rental { data: rental(10200, 0) }));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), 174);
    assert_eq!(value(&r, "amt_rental_adjustment"), -45);
    assert_eq!(value(&r, "agi"), 60045);
}

#[test]
fn separate_rental_amt_depreciation_refigures_investment_interest_income() {
    let mut d = complete(FederalStatus::Single);
    let mut rent = personal_rent_with_assets(1000, 0, vec![separate_amt_equipment()]);
    let FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental { basis, .. },
        ..
    } = &mut rent
    else {
        unreachable!()
    };
    *basis = 10000000;
    add(&mut d, income(rent));
    investment_borrowing(&mut d, 1000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 345);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 126);
    assert_eq!(value(&r, "amt_rental_adjustment"), -219);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn passive_qbi_full_suspension_does_not_claim_allowed_loss() {
    let source = passive_qbi_source();
    let qbi = &source.loss_qbi;
    let release =
        qbi.after_passive(&source, crate::tests::support::builders::k1_amount(10_000)).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (0, 0)
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            release.remaining.iter().map(|v| v.remaining.get()).sum::<i64>()
        ),
        10_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            release.remaining.iter().map(|v| v.remaining_qbi.get()).sum::<i64>()
        ),
        4_000
    );
    assert!(
        qbi.after_passive(&source, crate::tests::support::builders::k1_amount(10_001)).is_err()
    );
}

#[test]
fn property_qbi_fifo_preserves_pre2018_and_each_suspension_stage() {
    let mut source = passive_property_scorp(0, -6_000);
    source.opening = Some(500_000);
    source.risk.start = ScorpRiskStart::Detailed { capacity: 300_000, profit: 0, loss: 0 };
    let property = source.section1231.as_mut().unwrap();
    property.basis_loss = 200_000;
    property.risk_loss = 100_000;
    property.passive_loss = 200_000;
    let mut facts = property_qbi(3_000, 0);
    facts.losses.history = vec![loss_year(2017, 2_000, 0, 2_000, 0)];
    facts.losses.at_risk = vec![loss_year(2020, 1_000, 500, 1_000, 500)];
    facts.losses.passive = vec![loss_year(2022, 2_000, 500, 2_000, 500)];
    source.property_qbi = facts;
    source.amt = Some(amt_records(&source));
    let qbi = &source.property_qbi;
    let basis = source.basis_result().unwrap();
    let first = qbi.calculate(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(first.amount.get()),
            crate::kernel::ratio::dollars(first.qbi.get())
        ),
        (5_000, 1_500)
    );
    assert_eq!(crate::kernel::ratio::dollars(first.remaining[0].remaining.get()), 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(first.remaining[1].original.get()),
            crate::kernel::ratio::dollars(first.remaining[1].original_qbi.get())
        ),
        (3_000, 1_500)
    );
    let risk = qbi.after_at_risk(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.amount.get()),
            crate::kernel::ratio::dollars(risk.qbi.get())
        ),
        (3_000, 1_100)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.remaining[1].original.get()),
            crate::kernel::ratio::dollars(risk.remaining[1].original_qbi.get())
        ),
        (3_000, 900)
    );
    let passive =
        qbi.after_passive(&source, crate::tests::support::builders::k1_amount(4_000)).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(passive.amount.get()),
            crate::kernel::ratio::dollars(passive.qbi.get())
        ),
        (1_000, 250)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(passive.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(passive.remaining[0].remaining_qbi.get())
        ),
        (1_000, 250)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(passive.remaining[1].original.get()),
            crate::kernel::ratio::dollars(passive.remaining[1].original_qbi.get())
        ),
        (3_000, 1_100)
    );
}

#[test]
fn ordinary_qbi_with_property_uses_shared_limits_but_ordinary_suspension() {
    let mut source = passive_property_scorp(-30_000, -40_000);
    source.opening = Some(3_500_000);
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_400_000, profit: 0, loss: 0 };
    source.loss_qbi = sole_qbi_loss(30_000, 35_000, 15_000).loss_qbi;
    source.qbi_statement = complete_ordinary_scorp(0, false).qbi_statement;
    reconcile_qbi_tracking(&mut source);
    source.amt = Some(amt_records(&source));
    let qbi = &source.loss_qbi;
    let released =
        qbi.after_passive(&source, crate::tests::support::builders::k1_amount(3_000)).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(released.amount.get()),
            crate::kernel::ratio::dollars(released.qbi.get())
        ),
        (3_000, 1_500)
    );
    let items = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::tests::support::builders::k1_amount(3_000), 0)
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].income, -1_500);
    // $7k is the combined suspension, above this ordinary character's $6k.
    assert!(qbi.after_passive(&source, crate::tests::support::builders::k1_amount(7_000)).is_err());
    let (d, _) = passive_property_pool(source, 7_000);
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("ordinary QBI source after shared activity limits: current $-1500; released historical qualified loss $0")));
    assert_eq!(r.tax, None);
}
