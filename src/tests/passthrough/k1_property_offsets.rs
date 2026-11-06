use crate::tests::support::assert::k1_consolidated::k1_supported as supported;
use crate::tests::support::builders::k1_amount;
// Opposite source characters use the gross loss's qualified fraction, not
// the combined net business/QBI fraction. Independent final-return oracles.
use super::*;

fn source(ordinary: i64, property: i64, qualified: i64, property_qualified: i64) -> PartnerK1 {
    let mut p = ordinary_partner_source(ordinary, true);
    p.regular.section1231 = Some(k1_section1231(property, 0));
    p.regular.basis.opening = Some(100_000_000);
    p.regular.risk.start = ScorpRiskStart::Detailed { capacity: 100_000_000, profit: 0, loss: 0 };
    p.alternative = Some(p.regular.clone());
    p.qbi = Some(PartnerQbiStatement {
        section1231_qbi: property_qualified * 100,
        ..partner_qbi_statement(qualified)
    });
    p
}
fn draft(p: PartnerK1) -> FederalDraft {
    let (mut d, _) =
        crate::tests::passthrough::k1_sale_return::draft(FederalIncome::Partnership { data: p });
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Taxpayer, 8_000_000, 0));
    d
}
fn carried(r: &FederalResult, key: &str) -> i64 {
    r.lines.iter().find(|v| v.id == key).map_or(0, |v| v.amount / 100)
}

#[test]
fn k1_property_offsets_partner_operating_profit_absorbs_property_loss() {
    let mut p = source(100_000, -400_000, 100_000, -200_000);
    p.alternative.as_mut().unwrap().section1231.as_mut().unwrap().net = -30_000_000;
    let d = draft(p);
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    // Defer44/400 of the qualified200 loss, not44/300 of net QBI100.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 78_000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 356_000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 300_000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 0);
    assert_eq!(value(&r, "agi"), -176_000);
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_property_gain_absorbs_operating_loss() {
    let mut p = source(-400_000, 100_000, -200_000, 100_000);
    p.alternative.as_mut().unwrap().section1231.as_mut().unwrap().net = 5_000_000;
    let d = draft(p);
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 178_000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 0);
    assert_eq!(carried(&r, "section1231_2026_next_lookback"), 0);
    assert_eq!(value(&r, "agi"), -176_000);
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 94_000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 270_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_recaptured_gain_does_not_change_loss_qualification() {
    let mut d = draft(source(-400_000, 100_000, -200_000, 100_000));
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2024, 20_000, 20_000)], &[(2024, 20_000, 20_000)]));
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    // Ordinary property gain20 plus qualified loss200, then defer22 of loss.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 158_000);
    assert_eq!(carried(&r, "section1231_2026_next_lookback"), 0);
    assert_eq!(value(&r, "agi"), -176_000);
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_retained_property_loss_uses_listing_qualification() {
    let mut p = source(100_000, -300_000, 100_000, -150_000);
    p.regular.section1231.as_mut().unwrap().basis_loss = 10_000_000;
    p.alternative = Some(p.regular.clone());
    let d = draft(p);
    let r = supported(&d);
    // Allowed property400: current qualified150 and listed retained100.
    // Deferred qualified44*250/400=27.5; remaining net qualified loss122.5.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 122_500);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 356_000);
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_two_loss_characters_remain_explicit() {
    let d = draft(source(-200_000, -200_000, -100_000, -50_000));
    unsupported(&d, "Property excess business loss");
}

fn retained_ordinary() -> PartnerK1 {
    let mut p = source(-300_000, 100_000, -150_000, 100_000);
    p.regular.basis.prior_loss = 10_000_000;
    p.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2025, 100_000, 50_000, 100_000, 50_000)],
        risk: vec![],
        passive: vec![],
    });
    p.alternative = Some(p.regular.clone());
    p
}

#[test]
fn k1_property_offsets_partner_retained_ordinary_loss_keeps_original_qualification() {
    let d = draft(retained_ordinary());
    let r = supported(&d);
    // Current qualified150 plus retained50, gross allowed loss400.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 178_000);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_no_statement_still_retains_historical_qualification() {
    let mut p = retained_ordinary();
    p.qbi = None;
    let d = draft(p);
    let r = supported(&d);
    // Current loss contributes no QBI. Original qualified50 defers5.5.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 44_500);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert!(noted(&r, "WARN_K1_QBI_LOSS_NO_STATEMENT"));
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_no_statement_has_no_current_qualified_deferral() {
    let mut p = source(100_000, -400_000, 100_000, -200_000);
    p.qbi = None;
    let d = draft(p);
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 356_000);
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_current_sstb_phase_does_not_rephase_retained_property() {
    let mut p = source(100_000, -300_000, 100_000, -150_000);
    p.regular.section1231.as_mut().unwrap().basis_loss = 10_000_000;
    p.qbi.as_mut().unwrap().service = true;
    p.alternative = Some(p.regular.clone());
    let mut d = draft(p);
    let entry =
        d.income.iter_mut().find(|v| matches!(v.income, FederalIncome::Wage { .. })).unwrap();
    *entry = wage(entry.id, entry.owner.owned(), 90_000_000, 0);
    let r = supported(&d);
    // Current income/loss is phased out. Listed retained property100 stays
    // qualified and defers44/400 of that loss, leaving historical QBI89.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 89_000);
    assert_eq!(value(&r, "excess_business_loss"), 44_000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 356_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_partner_gain_side_histories_refuse_in_each_tax_system() {
    for amt in [false, true] {
        for stage in 0..3 {
            let mut p = source(-400_000, 100_000, -200_000, 100_000);
            let record = if amt { p.alternative.as_mut().unwrap() } else { &mut p.regular };
            let property = record.section1231.as_mut().unwrap();
            match stage {
                0 => property.basis_loss = 1_000_000,
                1 => property.risk_loss = 1_000_000,
                _ => property.passive_loss = 1_000_000,
            }
            unsupported(&draft(p), "Property excess business loss");
        }
    }
}

#[test]
fn k1_property_offsets_partner_joint_return_has_one_512k_threshold() {
    let mut d = complete(FederalStatus::Joint);
    d.income.clear();
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    add(
        &mut d,
        income(FederalIncome::Partnership { data: source(-700_000, 100_000, -350_000, 100_000) }),
    );
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Taxpayer, 8_000_000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 88_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 306_000);
    assert_eq!(value(&r, "agi"), -432_000);
    assert_eq!(value(&r, "nol_carryforward"), 520_000);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_rejected_candidate_does_not_mutate_activities() {
    let d = draft(source(-400_000, 100_000, -200_000, 100_000));
    let mut result = FederalResult::empty(d.revision);
    let pool = crate::extended_rules::rentals_for(&d, 0, &mut result, false);
    let mut activities = vec![crate::business_rules::QbiActivity {
        source_wages_cents: Some(0),
        source_property_cents: Some(0),
        source_income_cents: Some(-199_999 * 100),
        owner: None,
        income: -199_999,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    }];
    let original = activities.clone();
    assert!(
        crate::partnership_qbi::excess_with_property(
            &d,
            &pool,
            &mut activities,
            44_000,
            0,
            FederalStatus::Single
        )
        .is_err()
    );
    assert!(activities == original);
}

#[test]
fn k1_property_offsets_shared_rounding_is_current_first_in_either_storage_order() {
    use crate::business_rules::QbiActivity;
    let current = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: Some(1_000),
        owner: None,
        income: 10,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    };
    let historical = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: Some(-100),
        income: -1,
        active: false,
        ..current.clone()
    };
    let mut scorp = vec![current.clone(), historical.clone()];
    let mut partner = vec![historical, current];
    // One qualified dollar in each vintage: one dollar deferred allocates 50c
    // to each source before their reported rows are projected.
    assert_eq!(
        crate::scorp_qbi::defer_source_qbi(
            &mut scorp,
            Some(0),
            2,
            1,
            k1_amount(2),
            k1_amount(1),
            k1_amount(2)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        crate::scorp_qbi::defer_source_qbi(
            &mut partner,
            Some(1),
            2,
            1,
            k1_amount(2),
            k1_amount(1),
            k1_amount(2)
        )
        .unwrap(),
        1
    );
    assert_eq!((scorp[0].income, scorp[1].income), (11, -1));
    assert!(partner[1] == scorp[0] && partner[0] == scorp[1]);
    assert_eq!(
        (scorp[0].source_income_cents, scorp[1].source_income_cents),
        (Some(1_050), Some(-50))
    );
    let mut rows = [
        QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: Some(-50),
            income: -1,
            ..scorp[0].clone()
        },
        QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: Some(-50),
            income: -1,
            ..scorp[0].clone()
        },
    ];
    // Two distinct 50c reported rows give Form461 $2 gross loss, while the
    // qualified source loss remains 100c. A full reported fraction defers 100c.
    assert_eq!(
        crate::scorp_qbi::defer_source_qbi(
            &mut rows,
            None,
            2,
            2,
            k1_amount(1),
            k1_amount(0),
            k1_amount(1)
        )
        .unwrap(),
        1
    );
    assert!(rows.iter().all(|row| row.source_income_cents == Some(0) && row.income == 0));
    // A 50c qualified loss within $50 gross loss, with $49 deferred,
    // receives 49c. Project -1c once, rather than adding rounded operands.
    let mut endpoint = [QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: Some(-50),
        income: -1,
        ..scorp[0].clone()
    }];
    let cents = crate::kernel::carry::Amount::new(50).unwrap();
    assert_eq!(
        crate::scorp_qbi::defer_source_qbi(
            &mut endpoint,
            Some(0),
            50,
            49,
            cents,
            cents,
            k1_amount(50)
        )
        .unwrap(),
        0
    );
    assert_eq!((endpoint[0].source_income_cents, endpoint[0].income), (Some(-1), 0));
}

#[test]
fn k1_property_offsets_portfolio_capital_carry_limits_the_business_gain_offset() {
    let mut d = draft(source(-400_000, 100_000, -200_000, 100_000));
    d.screening.as_mut().unwrap().capital_history = vec![PriorCapitalReturn {
        injured_niit_owners: None,
        injured_owners: None,

        owners: PriorReturnOwners::Taxpayer,
        status: FederalStatus::Single,
        agi: 10_000_000,
        deductions: 1_000_000,
        senior_deduction: 0,
        short: 0,
        long: -20_300_000,
        deducted_loss: 300_000,
        section_1256: 0,
        amt: Some(PriorAmtCapitalReturn {
            taxable_income: 9_000_000,
            short: 0,
            long: 0,
            deducted_loss: 0,
        }),
        origins: vec![],
        ordinary_history: true,
        investment_parts: Some(InvestmentCapitalParts {
            short: 0,
            long: 20000000,
            amt: Some(InvestmentAmtCapitalParts { short: 0, long: 0 }),
        }),
        niit_difference: Some(0),
        research_parts: None,
    }];
    let r = supported(&d);
    // Regular capital gain100 is absorbed by portfolio carry200; it cannot
    // absorb operating loss for Form461. AMT has no portfolio carry.
    assert_eq!(value(&r, "excess_business_loss"), 144_000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 44_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
    assert_eq!(value(&r, "agi"), -179_000);
    // Form172 excludes the portfolio loss offset against business gain100
    // and the capital deduction3: current NOL76 plus EBL144 is total220.
    assert_eq!(value(&r, "nol_carryforward"), 220_000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 220_000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 0);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_zero_property_keeps_ordinary_only_allocation() {
    let d = draft(source(-400_000, 0, -200_000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 144_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
    assert_eq!(value(&r, "agi"), -176_000);
    assert_eq!(value(&r, "nol_carryforward"), 320_000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 0);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn k1_property_offsets_pya_guard_uses_actual_source_and_existing_report_endpoint() {
    for partner in [true, false] {
        let income = if partner {
            let mut owner = source(0, 0, 0, 0);
            owner.regular.ordinary = -50;
            owner.regular.basis.prior_loss = 50;
            owner.regular.section1231.as_mut().unwrap().net = 50;
            owner.qbi = None;
            owner.alternative = Some(owner.regular.clone());
            FederalIncome::Partnership { data: owner }
        } else {
            let mut owner = complete_ordinary_scorp(0, true);
            owner.ordinary = -50;
            owner.opening = Some(100_000_000);
            owner.prior_loss = 50;
            owner.risk.start =
                ScorpRiskStart::Detailed { capacity: 100_000_000, profit: 0, loss: 0 };
            owner.section1231 = Some(k1_section1231(0, 0));
            owner.section1231.as_mut().unwrap().net = 50;
            owner.qbi_statement = None;
            owner.amt = Some(amt_records(&owner));
            FederalIncome::Scorp { data: crate::tests::support::builders::scorp_source(owner) }
        };
        let (mut d, id) = crate::tests::passthrough::k1_sale_return::draft(income);
        d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        let mut report = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 0, &mut report, false);
        assert!(pool.inputs_complete, "{:?}", report.issues);
        // Preserve the existing reported endpoint. This cohort does not add
        // separately rendered PYA rows or substitute their hypothetical total.
        assert_eq!(pool.ordinary_pass_through_business(), -1);
        assert_eq!(
            pool.k1_amounts.iter().find(|source| source.id == id).unwrap().source_net_cents,
            Some(-100)
        );
        let character = crate::section1231::worksheet(&d, &pool, false).unwrap().unwrap();
        assert_eq!(
            crate::section1231::property_excess_allocation(&d, &pool, &character, 1, false, 0, &[])
                .unwrap(),
            0
        );
        let mut missing = pool.clone();
        missing.k1_amounts.iter_mut().find(|source| source.id == id).unwrap().source_net_cents =
            None;
        assert!(
            crate::section1231::property_excess_allocation(
                &d,
                &missing,
                &character,
                1,
                false,
                0,
                &[]
            )
            .is_err()
        );
    }
}
