use super::*;
use crate::business_rules::{QbiActivity, qbi};
use crate::tests::business::business::{aggregation_activity, aggregation_draft};
use crate::tests::support::builders::survey::survey_cash_business_data;

#[test]
fn rental_depreciation_midmonth_land_and_final_year() {
    let depreciation = |b: &crate::RentalBuilding, amt: bool, kind: &RentalPropertyType| {
        crate::extended_rules::depreciation_for_year(
            b,
            amt,
            (kind, &crate::RentalLocation::UnitedStates),
            2026,
            None,
            crate::kernel::source_fact::SourceFact::NotReported,
        )
    };
    let kind = RentalPropertyType::SingleFamily;
    let mut b = rental(0, 0).buildings.remove(0);
    assert_eq!(depreciation(&b, false, &kind).unwrap(), 1_000_000);
    b.in_service = 20260701;
    assert_eq!(depreciation(&b, false, &kind).unwrap(), 458_333);
    b.in_service = 20261231;
    assert_eq!(depreciation(&b, false, &kind).unwrap(), 41_667);
    b.in_service = 19990101;
    assert_eq!(depreciation(&b, false, &kind).unwrap(), 541_667);
    b.in_service = 19980101;
    assert_eq!(depreciation(&b, false, &kind).unwrap(), 0);
    b.land = b.cost + 1;
    assert!(depreciation(&b, false, &kind).is_err());
}

#[test]
fn rental_inactive_losses_offset_rental_profit_but_not_wages() {
    let mut d = complete(FederalStatus::Single);
    let mut loss = rental(1000, 6000); // -15,000
    loss.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: loss }));
    add(&mut d, income(FederalIncome::Rental { data: rental(20000, 0) })); // +10,000
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 5000);
    assert_eq!(value(&r, "agi"), 60000);
}

#[test]
fn qbi_minimum_uses_active_business_aggregate_before_passive_loss_allocation() {
    use crate::business_rules::{QbiActivity, qbi};
    let mut r = supported(&complete(FederalStatus::Single));
    let active = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 1000,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    };
    let passive_loss = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        income: -2000,
        active: false,
        ..active.clone()
    };
    assert_eq!(
        qbi(&[active.clone(), passive_loss], 100000, 0, FederalStatus::Single, &mut r),
        (400, 1000)
    );
    let active_loss = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        income: -1,
        ..active.clone()
    };
    assert_eq!(qbi(&[active, active_loss], 100000, 0, FederalStatus::Single, &mut r), (200, 0));
}

// Reg. 1.199A-1(d)(3)(iii): qualified PTP income or loss joins qualified
// REIT dividends (Form 8995 line 6 / Form 8995-A line 28), never the
// wage/UBIA-limited ordinary QBI activities, and never the aggregated
// business pool (C13a). A net PTP loss carries forward on that combined
// line (line 8/19, line 30/42), not the ordinary business-loss carryforward.
#[test]
fn ptp_qbi_is_separate_and_carries_its_own_loss() {
    // Single, taxable income before QBI $400,000: above the 2026 phase-in
    // ceiling ($201,750 + $75,000 = $276,750), so the wage/UBIA limit binds
    // in full on ordinary QBI. Net capital gain $0; the 20% taxable-income
    // cap is 20% x $400,000 = $80,000 (does not bind below).
    //
    // Schedule C: QBI $100,000, W-2 wages $60,000, UBIA $0.
    //   wage limit = max(50% x 60,000 = 30,000, 25% x 60,000 + 2.5% x 0 = 15,000) = 30,000
    //   deduction = min(20% x 100,000 = 20,000, 30,000) = $20,000
    // PTP K-1: $30,000 qualified income, $0 wages, $0 UBIA.
    //   no wage/UBIA limit applies -> 20% x 30,000 = $6,000
    // Total = 20,000 + 6,000 = $26,000, under the $80,000 cap.
    let mut r = supported(&complete(FederalStatus::Single));
    let schedule_c = [aggregation_activity(100_000, 60_000, None)];

    let mut owner = ordinary_partner_source(30_000, false);
    owner.publicly_traded = true;
    let statement = partner_qbi_statement(30_000);
    let prepared = statement
        .calculate(
            &owner,
            crate::tests::support::builders::k1_amount(0),
            0,
            400_000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(prepared.ptp, crate::kernel::ratio::exact::whole(30_000 * 100));
    assert_eq!(prepared.historical, 0);
    assert_eq!(prepared.current.income, 0);
    assert_eq!(prepared.current.wages, 0);
    assert_eq!(prepared.current.property, 0);
    assert!(prepared.current.group.is_none());

    let profit = crate::business_rules::qbi_with_history(
        &schedule_c,
        400_000,
        0,
        FederalStatus::Single,
        crate::business_rules::QbiOther {
            business_loss: 0,
            reit: crate::kernel::ratio::exact::dollars(&prepared.ptp).unwrap(),
            reit_loss: 0,
        },
        &mut r,
    );
    assert_eq!(profit.deduction, 26_000);
    assert_eq!(profit.business_loss, 0);
    assert_eq!(profit.reit_loss, 0);

    // A $10,000 PTP loss (a later year's fact pattern) reduces only the
    // REIT/PTP component: the Schedule C business deduction is unaffected
    // (still $20,000), and the loss carries forward on its own line instead
    // of netting against ordinary QBI business income.
    let mut loss_owner = ordinary_partner_source(-10_000, false);
    loss_owner.publicly_traded = true;
    let loss_statement = partner_qbi_statement(-10_000);
    let loss_prepared = loss_statement
        .calculate(
            &loss_owner,
            crate::tests::support::builders::k1_amount(0),
            0,
            400_000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(loss_prepared.ptp, crate::kernel::ratio::exact::whole(-10_000 * 100));

    let loss = crate::business_rules::qbi_with_history(
        &schedule_c,
        400_000,
        0,
        FederalStatus::Single,
        crate::business_rules::QbiOther {
            business_loss: 0,
            reit: crate::kernel::ratio::exact::dollars(&loss_prepared.ptp).unwrap(),
            reit_loss: 0,
        },
        &mut r,
    );
    assert_eq!(loss.deduction, 20_000);
    assert_eq!(loss.business_loss, 0);
    assert_eq!(loss.reit_loss, 10_000);
}

// Chosen: ignored, not refused. `PartnerQbiStatement` has no aggregation
// group-label field at all (Coverage: partnership.rs, "K-1 activities are
// never grouped"), so a publicly traded partnership cannot be entered into a
// Schedule B group in the first place; there is no fact to reject.
#[test]
fn ptp_is_never_aggregated() {
    let mut owner = ordinary_partner_source(30_000, false);
    owner.publicly_traded = true;
    let statement = partner_qbi_statement(30_000);
    let prepared = statement
        .calculate(
            &owner,
            crate::tests::support::builders::k1_amount(0),
            0,
            400_000,
            FederalStatus::Single,
        )
        .unwrap();
    // The placeholder pushed into the shared activities pool never carries a
    // group, so `business_rules::aggregate` can never combine it with
    // anything, however the taxpayer answers other activities' labels.
    assert!(prepared.current.group.is_none());

    // Two Schedule C activities sharing a label still aggregate their own
    // wages/UBIA as usual (C13a); adding the PTP amount changes the total by
    // exactly its own uncapped 20% share, proving it never joins that group's
    // combined wage/UBIA limit.
    let mut r = supported(&complete(FederalStatus::Single));
    let grouped = [
        aggregation_activity(200_000, 0, Some("Shops")),
        aggregation_activity(50_000, 120_000, Some("Shops")),
    ];
    let without_ptp = crate::business_rules::qbi_with_history(
        &grouped,
        400_000,
        0,
        FederalStatus::Single,
        crate::business_rules::QbiOther::default(),
        &mut r,
    );
    assert_eq!(without_ptp.deduction, 50_000);
    let with_ptp = crate::business_rules::qbi_with_history(
        &grouped,
        400_000,
        0,
        FederalStatus::Single,
        crate::business_rules::QbiOther {
            business_loss: 0,
            reit: crate::kernel::ratio::exact::dollars(&prepared.ptp).unwrap(),
            reit_loss: 0,
        },
        &mut r,
    );
    assert_eq!(with_ptp.deduction - without_ptp.deduction, 6_000);
}

#[test]
fn ptp_income_reaches_the_return_through_the_same_reit_ptp_component() {
    // Confirms the return-level wiring: a PTP K-1's qualified income enters
    // `qbi_deduction` through the same REIT/PTP component as qualified REIT
    // dividends (Form 8995 line 6), not the wage/UBIA-limited activities
    // pool. Baseline (reit_dividends_deduct_without_business_income_or_wage_limit):
    // $6,000 qualified REIT dividends alone give qbi_deduction $1,200 (20% x
    // 6,000). Adding a $30,000-qualified-income PTP K-1 (no wages/UBIA, so
    // unaffected by the wage limit either way) adds another 20% x 30,000 =
    // $6,000; both stay well under the 20% taxable-income cap at this
    // income level.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(reit_income(10000, 6000, true)));
    assert_eq!(value(&supported(&d), "qbi_deduction"), 1200);
    let mut ptp = ordinary_partner_source(30_000, false);
    ptp.publicly_traded = true;
    ptp.qbi = Some(partner_qbi_statement(30_000));
    add(&mut d, income(FederalIncome::Partnership { data: ptp }));
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 7200);
    assert_eq!(restart(d.clone()).draft, d);
}

// C-fixture (2026-09-25, matrix row 13, form-8995.md): an independent
// full-return Form 8995 fixture below the 2026 single QBI threshold, with
// a sole proprietorship and REIT dividends. Every total below is computed
// by hand from Schedule C, Schedule SE, Form 8995 and Form 1040, never
// copied from the App.
#[test]
fn qbi_return_below_the_threshold_matches_hand_computation() {
    // Single filer born 1983 (turns 43 in 2026, not a senior). One
    // sole-proprietorship Schedule C business (material participation,
    // receipts $140,000, one Office expense $10,000, no other expenses)
    // and a 1099-DIV reporting $2,000 ordinary dividends, all of it
    // section 199A REIT dividends (box 5), holding-period requirements
    // met.
    //
    // Schedule C: receipts $140,000.00, Office expense $10,000.00.
    //   Line 28 total expenses: $10,000.00. Line 31 net profit:
    //   140,000.00 - 10,000.00 = $130,000.00.
    //
    // Schedule SE (ordinary nonfarm method, one business, no W-2 wages, no
    // church income):
    //   Line 2/3: $130,000. Line 4a-4c: 130,000 x 92.35% = $120,055.00
    //   exactly (130,000 x 9,235 = 1,200,550,000, evenly divisible by
    //   10,000). Line 6: $120,055 (no church employee income). Line 7:
    //   $184,500 (2026 wage base). Line 8a-8d: $0 (no W-2). Line 9:
    //   184,500 - 0 = $184,500.
    //   Line 10 (12.4% of the smaller of line 6 and line 9, $120,055):
    //   120,055 x 12.4% = $14,886.82 exactly (1,488,682 cents).
    //   Line 11 (2.9% of line 6, never capped by the wage base): 120,055 x
    //   2.9% = $3,481.595, truncated by the App's integer cents arithmetic
    //   to $3,481.59 (348,159 cents).
    //   Combined cents: 1,488,682 + 348,159 = 1,836,841; (1,836,841 + 50)
    //   / 100 = 1,836,891 / 100 = $18,368 (line 12 self-employment tax,
    //   Schedule 2 line 4).
    //   Line 13 half-SE deduction: (18,368 + 1) / 2 = 18,369 / 2 = $9,184
    //   (Schedule 1 line 15).
    //
    // Form 8995 (2026 single QBI threshold $201,750; taxable income before
    // QBI is far below it, so the phase-in percentage is zero and no
    // SSTB/wage-limit reduction applies -- Form 8995's simplified path is
    // the same formula evaluated at a zero phase-in percentage):
    //   Line 1i business QBI = net profit less its attributable half-SE
    //   share: 130,000 - 9,184 = $120,816. Lines 2/4: $120,816 (no losses
    //   to net). Line 5: 120,816 x 20% = $24,163.20, rounds to $24,163.
    //   Line 6 REIT dividends: $2,000 (eligible). Line 7: $0 (no prior
    //   REIT/PTP history). Line 8: $2,000. Line 9: 2,000 x 20% = $400.
    //   Line 10: 24,163 + 400 = $24,563.
    //   Line 11 taxable income before QBI: AGI $122,816 - standard
    //   deduction $16,100 = $106,716. Line 12 (qualified dividends + net
    //   capital gain): $0. Line 13: $106,716. Line 14: 106,716 x 20% =
    //   $21,343.20, rounds to $21,343.
    //   Line 15: smaller of line 10 ($24,563) and line 14 ($21,343) =
    //   $21,343 -- the taxable-income cap binds here, below the $24,563
    //   sum of the business and REIT components.
    //   Line 16 ($400 active-business minimum): aggregate active QBI
    //   $120,816 >= $1,000, so the floor is $400, below line 15.
    //   Line 17: larger of line 15 ($21,343) and line 16 ($400) = $21,343
    //   QBI deduction.
    //
    // Additional Medicare tax (Form 8959; single $200,000 threshold):
    //   Schedule SE line 6 ($120,055) is below $200,000, so no Additional
    //   Medicare tax applies. NIIT (Form 8960, $200,000 MAGI threshold):
    //   AGI $122,816 is below it, so no NIIT applies either.
    //
    // Form 1040:
    //   Line 8 additional income (Schedule 1 line 3/10, the business
    //   profit only): $130,000. The $2,000 dividends are Form 1040 line
    //   3b, not line 8.
    //   Line 9 total income: 2,000 (dividends) + 130,000 (business) =
    //   $132,000.
    //   Line 10 adjustments (half-SE deduction only): $9,184.
    //   Line 11 AGI: 132,000 - 9,184 = $122,816.
    //   Line 12e standard deduction (single, born 1983, not 65 in 2026):
    //   $16,100. Line 13b QBI deduction: $21,343.
    //   Line 15 taxable income: 122,816 - 16,100 - 21,343 = $85,373.
    //   Line 16 regular tax: $85,373 is below the $100,000 Tax Table
    //   ceiling, so the September 2026 draft Tax Table's $50-wide-bracket
    //   midpoint method applies (i1040gi): midpoint = 85,373 / 50 x 50 +
    //   25 = $85,375. Tax on $85,375 under the 2026 single rate schedule
    //   (22% bracket): 12,400 x 10% + (50,400-12,400) x 12% +
    //   (85,375-50,400) x 22% = 1,240.00 + 4,560.00 + 7,694.50 =
    //   $13,494.50, which the table rounds to $13,495.
    //   Line 23 additional taxes (self-employment tax only; no Additional
    //   Medicare or NIIT): $18,368.
    //   Line 24a total tax: 13,495 + 18,368 = $31,863.
    //   Line 25a withholding: $0 (no W-2). Line 33 total payments: $0 (no
    //   withholding, no estimated payments, no refundable credits).
    //   Line 37 balance due: 31,863 - 0 = $31,863.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19830615;
    d.income.clear();
    add(
        &mut d,
        income(FederalIncome::Business {
            data: FederalBusiness {
                interest_limit_election: None,
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 14_000_000,
                    accounting: BusinessAccounting::Cash,
                },
                expenses: vec![ordinary_business_expense(
                    BusinessExpenseCategory::Office,
                    1_000_000,
                )],
                ..survey_cash_business_data()
            },
        }),
    );
    add(&mut d, income(reit_income(2000, 2000, true)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 18_368);
    assert_eq!(value(&r, "adjustments"), 9_184);
    assert_eq!(value(&r, "agi"), 122_816);
    assert_eq!(value(&r, "qbi_deduction"), 21_343);
    assert_eq!(value(&r, "taxable_income"), 85_373);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);

    assert_eq!(return_line(&r, "8"), 13_000_000);
    assert_eq!(return_line(&r, "9"), 13_200_000);
    assert_eq!(return_line(&r, "10"), 918_400);
    assert_eq!(return_line(&r, "11a / 11b"), 12_281_600);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 2_134_300);
    assert_eq!(return_line(&r, "15"), 8_537_300);
    assert_eq!(return_line(&r, "16"), 1_349_500);
    assert_eq!(return_line(&r, "24a"), 3_186_300);
    assert_eq!(return_line(&r, "37"), 3_186_300);
    assert_eq!(r.tax, Some(3_186_300));
    assert_eq!(r.payments, Some(0));
    assert_eq!(r.balance, Some(3_186_300));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn professional_gambling_limits_expenses_before_self_employment_and_qbi() {
    let mut d = empty_return(FederalStatus::Single);
    let entry = professional_gambling(100000, 90000, 10000);
    if let FederalIncome::Business { data } = &entry {
        assert_eq!(data.expense_total(false), 9000000);
    }
    add(&mut d, income(entry));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 1413);
    assert_eq!(value(&r, "agi"), 9293);
    assert_eq!(return_line(&r, "8"), 1000000);
    assert_eq!(return_line(&r, "25d"), 24000);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn personal_rental_purchases_share_the_exact_midquarter_test_with_business_assets() {
    for (cost, quarter, business_depreciation, rental_depreciation) in
        [(4000, false, 120000, 80000), (4001, true, 150000, 20005)]
    {
        let mut d = business_with_bonus(50000, rental_equipment(6000, BonusElection::ElectOut));
        let mut asset = rental_equipment(cost, BonusElection::ElectOut);
        asset.placed = 20261001;
        add(&mut d, income(personal_rent_with_assets(10000, 0, vec![asset])));
        assert_eq!(crate::depreciation::mid_quarter(&d), quarter);
        let r = supported(&d);
        for (id, amount) in [
            ("business_2_asset_0_macrs", business_depreciation),
            ("personal_rental_3_asset_0_macrs", rental_depreciation),
        ] {
            assert_eq!(line_amount(&r, id), amount, "{id}");
        }
        d.income.swap(1, 2);
        let reordered = supported(&d);
        assert_eq!(value(&reordered, "agi"), value(&r, "agi"));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn personal_rental_assets_reject_section179_and_inconsistent_class_elections() {
    for section179 in [Section179::Eligible { elected: 0 }, Section179::Eligible { elected: 10000 }]
    {
        let mut asset = rental_equipment(1000, BonusElection::Apply);
        asset.history =
            EquipmentHistory::New { section179, bonus: BonusElection::Apply, tables: false };
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(personal_rent_with_assets(10000, 0, vec![asset])));
        unsupported(&d, "Income:");
    }
    let mut d = business_with_bonus(50000, rental_equipment(6000, BonusElection::ElectOut));
    add(
        &mut d,
        income(personal_rent_with_assets(
            10000,
            0,
            vec![rental_equipment(1000, BonusElection::Apply)],
        )),
    );
    unsupported(&d, "Income:");
    let FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental { assets, .. },
        ..
    } = &mut d.income[2].income
    else {
        unreachable!()
    };
    assets[0] = rental_equipment(1000, BonusElection::ElectOut);
    assets[0].method = EquipmentMethod::StraightLine;
    unsupported(&d, "Income:");
}

#[test]
fn nonbusiness_rental_does_not_supply_section179_cost_or_active_business_income() {
    let mut asset = rental_equipment(100000, BonusElection::ElectOut);
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 9000000 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let mut d = business_with_bonus(10000, asset);
    let before = supported(&d);
    add(
        &mut d,
        income(personal_rent_with_assets(
            100000,
            1000,
            vec![rental_equipment(10000, BonusElection::ElectOut)],
        )),
    );
    let after = supported(&d);
    for id in [
        "section179_eligible_cost",
        "section179_business_income_limit",
        "business_2_asset_0_section179",
        "business_2_asset_0_section179_carryforward",
    ] {
        assert_eq!(value(&after, id), value(&before, id), "{id}");
    }
    assert_eq!(value(&after, "personal_rental_3_asset_0_macrs"), 2000);
    assert_eq!(value(&after, "agi") - value(&before, "agi"), 97000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn passive_pool_business_and_rental_share_income_but_not_rental_allowance() {
    use crate::tests::support::loss_limits::{PassiveActivity as A, passive_suspensions};
    let pool = [
        A { key: (), net: -30_000, rental_allowance: true, ineligible_prior: 0 },
        A { key: (), net: -20_000, rental_allowance: false, ineligible_prior: 0 },
        A { key: (), net: 10_000, rental_allowance: false, ineligible_prior: 0 },
    ];
    assert_eq!(passive_suspensions(&pool, 0, 25_000).unwrap(), vec![3_000, 12_000, 0]);
    assert_eq!(
        passive_suspensions(
            &[A { key: (), net: -30_000, rental_allowance: false, ineligible_prior: 0 }],
            0,
            25_000
        )
        .unwrap(),
        vec![30_000]
    );
}

#[test]
fn property_qbi_net_zero_retains_different_business_qualified_portions() {
    let d = property_qbi_character_draft(10_000, 10_000);
    let items = property_qbi_character_items(&d).unwrap();
    assert_eq!(
        items.iter().map(|v| (v.gain, v.loss)).collect::<alloc::vec::Vec<_>>(),
        vec![(5_000, 0), (0, 10_000)]
    );
    assert_eq!(items.iter().map(|v| v.gain - v.loss).sum::<i64>(), -5_000);
    assert_eq!(lookback_worksheet(&d, false).unwrap().unwrap().net, 0);
    let r = estimate(&d);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("potential QBI after capital exclusion: gain $5000; loss $0"))
    );
    // Both current qualified portions are supplied by their code V statements.
    assert!(!noted(&r, "WARN_K1_QBI_LOSS_NO_STATEMENT"), "{r:?}");
}

#[test]
fn property_qbi_net_loss_preserves_positive_business_items() {
    let d = property_qbi_character_draft(10_000, 20_000);
    let items = property_qbi_character_items(&d).unwrap();
    assert_eq!(
        items.iter().map(|v| (v.gain, v.loss)).collect::<alloc::vec::Vec<_>>(),
        vec![(5_000, 0), (0, 20_000)]
    );
}

#[test]
fn combined_property_qbi_counts_one_business_wage_limit() {
    let mut source = former_passive_property_scorp(10_000, 20_000, 0, 0);
    source.property_qbi = property_qbi(0, 20_000);
    source.qbi_statement = complete_ordinary_scorp(10_000, true).qbi_statement;
    source.qbi_statement.as_mut().unwrap().wages = 1_000_000;
    let statement = source.qbi_statement.as_ref().unwrap();
    let items = statement
        .combined_activities(
            &source,
            [crate::kernel::carry::Amount::default(); 2],
            crate::section1231::PropertyCharacter::Ordinary,
            Some(400_000),
            FederalStatus::Single,
            0,
            0,
        )
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!((items[0].income, items[0].wages), (30_000, 10_000));
    let mut r = estimate(&complete(FederalStatus::Single));
    assert_eq!(
        crate::business_rules::qbi(&items, 400_000, 0, FederalStatus::Single, &mut r),
        (5_000, 0)
    );
    assert_eq!(
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
            .unwrap()[0]
            .income,
        30_000
    );
}

const AGGREGATION_NOTE: &str =
    "Aggregation elected under Reg. 1.199A-4 for group Shops; attach Schedule B (Form 8995-A)";

#[test]
fn qbi_aggregation_combines_wages_and_ubia_before_the_limit() {
    // Single, taxable income before QBI $400,000: above the 2026 phase-in
    // ceiling ($201,750 + $75,000 = $276,750), so the wage/UBIA limit applies
    // in full. Net capital gain $0; income cap 20% x $400,000 = $80,000.
    // A: QBI $200,000, W-2 wages $0, UBIA $0. B: QBI $50,000, wages $120,000.
    let mut r = supported(&complete(FederalStatus::Single));
    let ungrouped =
        [aggregation_activity(200_000, 0, None), aggregation_activity(50_000, 120_000, None)];
    // Ungrouped (Form 8995-A lines 3-11 per business):
    //   A: min(20% x 200,000 = 40,000, max(50% x 0, 25% x 0 + 2.5% x 0) = 0) = 0.
    //   B: min(20% x 50,000 = 10,000, max(50% x 120,000 = 60,000, 30,000)) = 10,000.
    //   Deduction 0 + 10,000 = $10,000 (below the $80,000 income cap).
    assert_eq!(qbi(&ungrouped, 400_000, 0, FederalStatus::Single, &mut r), (10_000, 0));
    assert!(!r.review_notes.iter().any(|n| n.contains("Aggregation")));
    // Grouped (Schedule B): one trade with QBI $250,000 and wages $120,000:
    //   min(20% x 250,000 = 50,000, max(60,000, 30,000) = 60,000) = $50,000.
    let grouped = ungrouped.clone().map(|v| QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        group: Some("Shops".into()),
        ..v
    });
    assert_eq!(qbi(&grouped, 400_000, 0, FederalStatus::Single, &mut r), (50_000, 0));
    assert!(r.review_notes.iter().any(|n| n.contains(AGGREGATION_NOTE)));
    let note = r
        .review_notes
        .iter()
        .find(|note| {
            let message = &note.display;
            message.key == "NOTE_QBI_AGGREGATION_ELECTION"
        })
        .unwrap();
    let encoded = note.display.encode();
    let mut offset = 0;
    let decoded = <crate::common::FormattedMessage as libertas::AvroDecode>::avro_decode(
        &encoded,
        &mut offset,
    )
    .unwrap();
    assert_eq!(offset, encoded.len());
    assert!(matches!(
        &decoded.1[..],
        [
            libertas::LibertasMessageArgumentDecode::LiteralText(group),
            libertas::LibertasMessageArgumentDecode::Currency { currency_code: a, scale: 0, minor_units: 250_000 },
            libertas::LibertasMessageArgumentDecode::Currency { currency_code: b, scale: 0, minor_units: 120_000 },
            libertas::LibertasMessageArgumentDecode::Currency { currency_code: c, scale: 0, minor_units: 0 }
        ] if group == "Shops" && a == "USD" && b == "USD" && c == "USD"
    ));

    // The label on the Schedule C records reaches the same engine: W-2 wages
    // $400,000 and businesses netting about $200,000 and $50,000 after payroll.
    let plain = supported(&aggregation_draft(
        40_000_000,
        [(20_000_000, 0), (17_000_000, 12_000_000)],
        None,
    ));
    let labeled = supported(&aggregation_draft(
        40_000_000,
        [(20_000_000, 0), (17_000_000, 12_000_000)],
        Some("Shops"),
    ));
    assert!(!plain.review_notes.iter().any(|n| n.contains("Aggregation")));
    assert!(labeled.review_notes.iter().any(|n| n.contains(AGGREGATION_NOTE)));
    assert!(value(&labeled, "qbi_deduction") > value(&plain, "qbi_deduction"));
}

// M3-F1 boundary (3 of 3): the Form 8582 active-participation allowance's
// $25,000 edge (schedule-e.md map; i8582 Worksheet 1) -- exactly at $100,000
// MAGI the allowance is still the full, unphased $25,000, one dollar under
// where the 50-cents-per-dollar phase-out begins.
#[test]
fn passive_allowance_is_full_at_exactly_100000_magi() {
    // MAGI here is AGI before the rental loss, i.e. $100,000 of wages with
    // no other income or adjustments: positive(100,000-100,000) = $0 excess,
    // so the allowance is the full, unreduced $25,000.
    // rental() building: (330,000-55,000)/27.5 = $10,000 depreciation (2020
    // in-service, a full mid-recovery year by 2026). Rent $0, other expense
    // $16,000: total expenses 16,000+10,000 = $26,000, a $26,000 loss. The
    // $25,000 allowance leaves exactly $1,000 suspended and a $25,000
    // allowed loss.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 10_000_000, 0)];
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 16_000) }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -25_000);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 1_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 1_000);
}

#[test]
fn passive_loss_carryover_is_reduced_after_basis() {
    // $1,500 excluded (insolvent by $2,900). Basis has no room ($5,000 basis
    // and cash after < $6,000 liabilities after), so the listed rental
    // building keeps its basis and the $5,000 suspended rental loss takes all
    // $1,500 (line 12), leaving $3,500.
    let mut d = complete(FederalStatus::Single);
    let mut loss = rental(1000, 6000);
    loss.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: loss }));
    add(&mut d, income(FederalIncome::Rental { data: rental(20000, 0) }));
    let mut debt = insolvency_debt(1500, 7500, 4600, 5000);
    let FederalIncome::PersonalDebt { data } = &mut debt.income else { panic!() };
    let mut data = edit_debt(data);
    let Some(PersonalDebtTreatment::Insolvent { data: facts }) = &mut data.treatment else {
        panic!()
    };
    facts.property.push(property(
        "Rental house",
        DebtPropertyKind::Building { secured: false },
        90000,
    ));
    drop(data);
    add(&mut d, debt);
    let r = supported(&d);
    assert_eq!(value(&r, "debt_4_form982_line12"), 1500);
    assert_eq!(value(&r, "debt_4_passive_after"), 3500);
    assert_eq!(value(&r, "debt_4_basis_reduction"), 0);
    // Unlisted rental property is refused by name once the debt reaches basis.
    let FederalIncome::PersonalDebt { data } = &mut d.income[3].income else { panic!() };
    let mut data = edit_debt(data);
    let Some(PersonalDebtTreatment::Insolvent { data: facts }) = &mut data.treatment else {
        panic!()
    };
    facts.property.pop();
    drop(data);
    unsupported(&d, "Insolvency tax attributes");
}

#[test]
fn unit13_qbi_source_owners_survive_aggregation_and_the_joint_minimum() {
    use crate::business_rules::{QbiFiling, QbiOther, qbi_filed};
    let activities = [
        QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: None,
            owner: Some(Owner::Taxpayer),
            income: 10_000,
            wages: 100_000,
            property: 0,
            active: true,
            service: Some(false),
            group: Some("Actual aggregation".into()),
        },
        QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: None,
            owner: Some(Owner::Spouse),
            income: 20_000,
            wages: 100_000,
            property: 0,
            active: true,
            service: Some(false),
            group: Some("Actual aggregation".into()),
        },
    ];
    let mut r = FederalResult::empty(0);
    let grouped = qbi_filed(
        &activities,
        600_000,
        0,
        FederalStatus::Joint,
        QbiOther::default(),
        &QbiFiling::default(),
        &mut r,
    );
    assert_eq!(grouped.deduction, 6_000);
    assert!(grouped.lines.grouped);
    assert_eq!(grouped.owner_income, Some([10_000, 20_000]));
    assert_eq!(grouped.owner_active, Some([10_000, 20_000]));
    let minimum = qbi_filed(
        &activities,
        100_000,
        0,
        FederalStatus::Joint,
        QbiOther { business_loss: 40_000, ..Default::default() },
        &QbiFiling::default(),
        &mut r,
    );
    assert_eq!(minimum.deduction, 400);
    assert_eq!(minimum.owner_active, Some([10_000, 20_000]));
    let mut unknown = activities.clone();
    unknown[1].owner = None;
    let unknown = qbi_filed(
        &unknown,
        600_000,
        0,
        FederalStatus::Joint,
        QbiOther::default(),
        &QbiFiling::default(),
        &mut r,
    );
    assert_eq!(unknown.deduction, grouped.deduction);
    assert_eq!(unknown.owner_income, None);
}
