use super::*;

#[test]
fn bonus_equipment_preserves_qbi_acquisition_basis_and_can_create_loss() {
    let mut d = business_with_bonus(100000, bonus_equipment(40000, 100));
    d.income[0] = wage(1, Owner::Taxpayer, 50000000, 0);
    assert_eq!(value(&supported(&d), "qbi_deduction"), 1000); // 2.5% of $40,000 UBIA survives full depreciation.
    let loss = business_with_bonus(1000, bonus_equipment(100000, 100));
    let r = supported(&loss);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 100000);
    assert_eq!(value(&r, "agi"), -39000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
}

#[test]
fn macrs_mid_quarter_uses_all_quarters_and_return_wide_prebonus_business_basis() {
    for (date, expected) in
        [(20260101, 350000), (20260401, 250000), (20260701, 150000), (20261001, 50000)]
    {
        let asset = macrs_equipment(10000, EquipmentClass::Five, EquipmentMethod::Regular, date);
        assert_eq!(asset.deduction(true), expected);
    }
    let mut d = business_with_bonus(
        100000,
        macrs_equipment(6000, EquipmentClass::Five, EquipmentMethod::Regular, 20260101),
    );
    let mut second = cash_business(100000);
    let FederalIncome::Business { data } = &mut second else { panic!() };
    // A different class may use bonus; exactly 40% in Q4 does not trigger MQ.
    data.assets.push(macrs_equipment(
        4000,
        EquipmentClass::Seven,
        EquipmentMethod::Regular,
        20261001,
    ));
    data.assets[0].history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::Apply,
        tables: false,
    };
    add(&mut d, income(second));
    assert!(!crate::depreciation::mid_quarter(&d));
    assert_eq!(value(&supported(&d), "business_2_asset_0_macrs"), 1200);
    let FederalIncome::Business { data } = &mut d.income[2].income else { panic!() };
    data.assets[0].cost += 1;
    assert!(crate::depreciation::mid_quarter(&d));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 2100);
    assert_eq!(value(&r, "business_2_macrs_mid_quarter"), 1);
    let FederalIncome::Business { data } = &mut d.income[2].income else { panic!() };
    data.assets[0].listed = ListedProperty::No { business_percent: 50 };
    assert!(!crate::depreciation::mid_quarter(&d));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn phased_bonus_acquisition_boundaries_and_remaining_basis_reconcile() {
    for (acquired, bonus, macrs, remaining) in
        [(20170928, 2000, 1600, 6400), (20250119, 2000, 1600, 6400), (20250120, 10000, 0, 0)]
    {
        let mut asset = bonus_equipment(10000, 100);
        asset.acquired = acquired;
        let d = business_with_bonus(100000, asset);
        let r = supported(&d);
        assert_eq!(value(&r, "business_2_asset_0_bonus"), bonus);
        assert_eq!(value(&r, "business_2_asset_0_macrs"), macrs);
        assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), remaining);
        assert_eq!(bonus + macrs + remaining, 10000);
        assert_eq!(restart(d.clone()).draft, d);
    }
    let mut asset = bonus_equipment(10000, 100);
    asset.acquired = 20170927;
    let d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 2000);
    assert_eq!(value(&r, "business_2_asset_0_amt_depreciation"), 1500);
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 8000);
    assert_eq!(value(&r, "business_2_asset_0_amt_remaining_basis"), 8500);
    assert_eq!(value(&r, "amt_business_depreciation_adjustment"), 500);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn phased_bonus_uses_remaining_basis_for_each_gds_method_and_business_share() {
    for (method, macrs) in [
        (EquipmentMethod::Regular, 800),
        (EquipmentMethod::Declining150, 600),
        (EquipmentMethod::StraightLine, 400),
    ] {
        let mut asset = bonus_equipment(10000, 50);
        asset.acquired = 20250119;
        asset.method = method;
        let d = business_with_bonus(100000, asset);
        let r = supported(&d);
        assert_eq!(value(&r, "business_2_asset_0_bonus"), 1000);
        assert_eq!(value(&r, "business_2_asset_0_macrs"), macrs);
        assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 4000 - macrs);
        assert_eq!(crate::interview::document_amount(&d.income[1].income, false), None);
        let mut ordinary = d.clone();
        let FederalIncome::Business { data } = &mut ordinary.income[1].income else { panic!() };
        data.assets.clear();
        data.expenses.push(ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Equivalent deduction".into() },
            (1000 + macrs) * 100,
        ));
        let expected = supported(&ordinary);
        for id in ["agi", "self_employment_tax", "qbi_deduction"] {
            assert_eq!(value(&r, id), value(&expected, id));
        }
        assert_eq!(r.tax, expected.tax);
    }
}

#[test]
fn continuing_macrs_rejects_bad_history_and_expired_unrecovered_basis() {
    unsupported(
        &business_with_bonus(100000, continuing_equipment(20200601, 99999, false)),
        "Income:",
    );
    unsupported(
        &business_with_bonus(100000, continuing_equipment(20260601, 20000, false)),
        "Income:",
    );
    assert!(!business_with_bonus(100000, continuing_equipment(20250601, -1, false)).valid_stored());
    assert!(
        !business_with_bonus(100000, continuing_equipment(20250601, 100001, false)).valid_stored()
    );
    let mut d = business_with_bonus(100000, continuing_equipment(20200601, 100000, false));
    d.income[0] = wage(1, Owner::Taxpayer, 50000000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 0);
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 0);
    // Zero adjusted basis does not erase QBI UBIA during the 10-year minimum period.
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[0].cost = 4000000;
    let EquipmentHistory::Continuing { amt: EquipmentAmtHistory::Same, prior_depreciation, .. } =
        &mut data.assets[0].history
    else {
        panic!()
    };
    *prior_depreciation = 4000000;
    assert_eq!(value(&supported(&d), "qbi_deduction"), 1000);
}

#[test]
fn continuing_macrs_business_allocation_does_not_reduce_recorded_prior_basis_twice() {
    let mut asset = continuing_equipment(20250601, 10000, false);
    asset.listed = ListedProperty::No { business_percent: 50 };
    let d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 160);
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 240);
    let mut ordinary = d.clone();
    let FederalIncome::Business { data } = &mut ordinary.income[1].income else { panic!() };
    data.assets.clear();
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Equivalent deduction".into() },
        16000,
    ));
    let expected = supported(&ordinary);
    for id in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(r.tax, expected.tax);
}

#[test]
fn macrs_table_continuation_uses_original_postbonus_basis_not_opening_basis() {
    let mut asset = continuing_equipment(20250601, 52000, false);
    asset.history = table_history(52000, 40000, false); // 40% bonus + 20% of remaining $600.
    let d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(line_amount(&r, "business_2_asset_0_macrs"), 19200); // $600 * 32%.
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 288);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn section197_prior_year_final_month_and_exhausted_basis_are_correct() {
    for (acquired, deduction, remaining) in
        [(20250701, 1200, 16200), (20110701, 600, 0), (20110101, 0, 0), (19930811, 0, 0)]
    {
        let r = supported(&intangible_business(acquired, 19900101));
        assert_eq!(value(&r, "business_2_intangible_0_amortization"), deduction);
        assert_eq!(value(&r, "business_2_intangible_0_remaining_basis"), remaining);
    }
}

#[test]
fn section197_flows_through_business_taxes_without_tangible_qbi_basis() {
    let d = intangible_business(20260701, 20200101);
    let r = supported(&d);
    let mut ordinary = d.clone();
    let FederalIncome::Business { data } = &mut ordinary.income[1].income else { panic!() };
    data.intangibles.clear();
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Equivalent deduction".into() },
        60000,
    ));
    let expected = supported(&ordinary);
    for id in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(r.tax, expected.tax);
    let mut high = d.clone();
    high.income[0] = wage(1, Owner::Taxpayer, 50000000, 0);
    assert_eq!(value(&supported(&high), "qbi_deduction"), 400); // Tangible UBIA would incorrectly give $450.
    let mut loss = d;
    let FederalIncome::Business { data } = &mut loss.income[1].income else { panic!() };
    match &mut data.activity {
        BusinessActivity::Ordinary { receipts, .. }
        | BusinessActivity::Gambling { receipts, .. } => *receipts = 0,
        _ => panic!("expected nonfarm test business"),
    };
    assert_eq!(value(&supported(&loss), "agi"), 59400);
}

#[test]
fn continuing_section179_history_preserves_qbi_cost_and_final_basis() {
    let mut asset = continuing_equipment(20210601, 97000, false);
    asset.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        prior_depreciation: 97000,
        mid_quarter: false,
        initial_section179: 50000,
        carryforward: 0,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    assert_eq!(asset.macrs_amount(false), 3000); // Final year exhausts actual remaining cents.
    assert_eq!(asset.business_basis(), 100000); // UBIA is not reduced by expensing.
    let mut manual = asset.clone();
    manual.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        prior_depreciation: 97000,
        mid_quarter: false,
        initial_section179: 50000,
        carryforward: 0,
        schedule: EquipmentSchedule::Manual,
    };
    assert_eq!(manual.macrs_amount(false), 3000);
    supported(&business_with_bonus(100000, manual));
    supported(&business_with_bonus(100000, asset));
}

#[test]
fn section179_phaseout_counts_zero_elections_and_changes_midquarter_basis() {
    let mut d = business_with_bonus(10000000, elected_asset(4090000, 1, BonusElection::ElectOut));
    assert_eq!(crate::section179::calculate(&d).limit, 256000000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(elected_asset(100000, 0, BonusElection::ElectOut));
    assert_eq!(crate::section179::calculate(&d).limit, 246000000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(elected_asset(2460000, 0, BonusElection::ElectOut));
    assert_eq!(crate::section179::calculate(&d).limit, 0);
    unsupported(&d, "Section 179");
    let mut d = business_with_bonus(10000, elected_asset(1000, 0, BonusElection::ElectOut));
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let mut last = elected_asset(1000, 1000, BonusElection::ElectOut);
    last.placed = 20261201;
    data.assets.push(last);
    assert!(!crate::depreciation::mid_quarter(&d));
    supported(&d);
}

#[test]
fn section179_carryforward_does_not_reenter_macrs_basis() {
    let mut asset = continuing_equipment(20250601, 28000, false);
    asset.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        initial_section179: 60000,
        carryforward: 40000,
        prior_depreciation: 28000,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    assert!(asset.valid() && asset.qualifies());
    assert_eq!(asset.opening_basis(), 32000);
    assert_eq!(asset.macrs_amount(false), 12800);
    let d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 400);
    assert_eq!(value(&r, "section179_carryforward"), 0);
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 192);
    assert_eq!(restart(d.clone()).draft, d);
}

// Assigned PTP179/PAL seam: native source-owner controls. The existing
// whole-return PTP routing refusal remains until its PAL consumer is delivered.
#[test]
fn unit17_ptp179_keeps_current_and_actual_prior_income_carry_out_of_pal() {
    let mut draft = ptp179_draft(20_000, 5_000);
    if let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income {
        data.section179_carryover = 200_000;
        data.section179_carryover_sold = 0;
        // AMT comes from the actual selected producer, with an independently
        // reported carry/year; the regular aggregate does not establish it.
        data.amt_section179_income_carry = Some(Section179IncomeCarryHistory {
            gross: 200_000,
            sold: 0,
            vintages: vec![Asset179Carryover { year: 2023, amount: 200_000 }],
        });
    }
    for alternative in [false, true] {
        let delivered = if alternative {
            selected_ptp179(&draft, Section179IncomeSystem::Alternative, 2, (20_000) * 100)
                .unwrap()
                .ptp_section179_pal_input(2)
                .unwrap()
                .clone()
        } else {
            selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (20_000) * 100)
                .unwrap()
                .ptp_section179_pal_input(2)
                .unwrap()
                .clone()
        };
        assert_eq!(delivered.k1, 2);
        assert_eq!(delivered.alternative, alternative);
        assert_eq!(delivered.owner_income_cents, 2_000_000);
        assert!(delivered.active);
        assert_eq!(delivered.allowed_cents, 700_000);
        assert_eq!(delivered.business_income_carry_cents, 0);
        assert_eq!(
            delivered.components,
            vec![
                crate::Ptp179ComponentAllocation {
                    component: crate::Ptp179Component::Current,
                    available_cents: 500_000,
                    allowed_cents: 500_000
                },
                crate::Ptp179ComponentAllocation {
                    component: crate::Ptp179Component::PriorIncomeCarry {
                        original_year: alternative.then_some(2023)
                    },
                    available_cents: 200_000,
                    allowed_cents: 200_000
                },
            ]
        );
        // Complete accepted native storage retains the same source result;
        // neither PAL prior losses nor restored sold carry is introduced anew.
        assert_eq!(
            if alternative {
                selected_ptp179(
                    &restart(draft.clone()).draft,
                    Section179IncomeSystem::Alternative,
                    2,
                    (20_000) * 100,
                )
                .unwrap()
                .ptp_section179_pal_input(2)
                .unwrap()
                .clone()
            } else {
                selected_ptp179(
                    &restart(draft.clone()).draft,
                    Section179IncomeSystem::Regular,
                    2,
                    (20_000) * 100,
                )
                .unwrap()
                .ptp_section179_pal_input(2)
                .unwrap()
                .clone()
            },
            delivered
        );
    }
}
#[test]
fn unit17_ptp179_income_limit_carry_never_enters_the_current_pal_amount() {
    let mut draft = ptp179_draft(5_000, 70_000);
    if let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income {
        data.section179_carryover = 200_000;
        data.section179_carryover_sold = 0;
    }
    let delivered = selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (5_000) * 100)
        .unwrap()
        .ptp_section179_pal_input(2)
        .cloned()
        .unwrap();
    // Existing wages60000 + active K1 ordinary5000, no other deduction:
    // current allowed65000, current carry5000 + retained carry2000.
    assert_eq!(delivered.allowed_cents, 6_500_000);
    assert_eq!(delivered.business_income_carry_cents, 700_000);
    assert_eq!(delivered.components[0].allowed_cents, 6_500_000);
    assert_eq!(delivered.components[1].allowed_cents, 0);
}
#[test]
fn unit17_ptp179_preserves_passive_source_for_its_own_pal_consumer() {
    let mut draft = ptp179_draft(20_000, 5_000);
    if let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income {
        data.participation = PartnerParticipation::General { facts: participation_facts() };
    }
    let delivered = selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, 0)
        .unwrap()
        .ptp_section179_pal_input(2)
        .cloned()
        .unwrap();
    assert!(!delivered.active);
    assert_eq!(delivered.owner_income_cents, 0);
    assert_eq!(delivered.allowed_cents, 500_000); // wages support179; PAL is later.
    assert_eq!(delivered.business_income_carry_cents, 0);
    // 2026-10-16 D6 source coordination: PTP routing is delivered, but this
    // original fixture supplies no AMT income-carry history (not even zero).
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.issues.iter().any(|issue| {
        issue.display.key == "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"
            && matches!(&issue.context, crate::IssueContext::Corrective { source, sections }
                if sections.is_empty() && matches!(source.as_ref(), crate::IssueContext::Income { id: 2, .. }))
    }));
}
#[test]
fn unit17_ptp179_preserves_alternative_basis_and_existing_owner_boundaries() {
    use crate::Ptp179SourceIssue as E;
    let mut draft = ptp179_draft(20_000, 5_000);
    if let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income {
        let alternative = data.alternative.as_mut().unwrap();
        alternative.ordinary = 0;
        alternative.basis.opening = 100_000;
        data.amt_section179_income_carry =
            Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    }
    let selected = selected_ptp179(&draft, Section179IncomeSystem::Alternative, 2, 0).unwrap();
    assert_eq!(selected.ptp_section179_pal_input(2).unwrap().allowed_cents, 100_000);
    assert_eq!(selected.income.k1[&2].components[0].available_cents, 100_000);
    let mut draft = ptp179_draft(5_000, 70_000);
    if let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income {
        data.guaranteed_capital = 100_000;
    }
    let Err(E::OwnerIssues(issues)) =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (5_000) * 100)
            .unwrap()
            .ptp_section179_pal_input(2)
            .cloned()
    else {
        panic!("must retain compensation source boundary")
    };
    assert!(issues.iter().any(|issue| issue.display.key.contains("GUARANTEED_PAYMENTS")));
    let selected =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (5_000) * 100).unwrap();
    assert_eq!(selected.ptp_section179_pal_input(2), Err(E::OwnerIssues(issues)));
    assert_eq!(selected.income.business_income_limit_cents, 6_500_000);
    assert_eq!(selected.income.k1[&2].allowed_cents, 6_500_000);
    let mut report = FederalResult::empty(draft.revision);
    selected.ptp_report_context(&[]).unwrap().report(&mut report);
    assert!(report.issues.iter().any(|issue| issue.display.key.contains("GUARANTEED_PAYMENTS")));
    assert_eq!(value(&report, "section179_business_income_limit"), 65_000);
    assert_eq!(value(&report, "section179_deduction"), 65_000);
}

#[test]
fn unit17_income_source_nets_actual_classes_once_and_supplied_zero_wins() {
    let draft = ptp179_draft(5_000, 70_000);
    // Wages60000 + ordinary(8000-2000) +1231(-3000-1000) =62000.
    let supplied = crate::section179::section179_with_native_prepared_income_and_operating_context(
        &draft,
        Section179IncomeSystem::Regular,
        &[],
        &[section179_income_source(2, (8_000) * 100, (2_000) * 100, (-3_000) * 100, (1_000) * 100)],
        |_, _, _| {
            Some(Section179OtherLimitedDeduction {
                system: Section179IncomeSystem::Regular,
                amount_cents: 0,
            })
        },
    )
    .unwrap()
    .income;
    assert_eq!(supplied.business_income_limit_cents, 6_200_000);
    assert_eq!(supplied.total_allowed_cents, 6_200_000);
    assert_eq!(supplied.k1[&2].components[0].available_cents, 7_000_000);
    assert_eq!(supplied.k1[&2].components[0].allowed_cents, 6_200_000);
    assert_eq!(supplied.k1[&2].business_income_carry_cents, 800_000);
    let zero = crate::section179::section179_with_native_prepared_income_and_operating_context(
        &draft,
        Section179IncomeSystem::Regular,
        &[],
        &[section179_income_source(2, 0, 0, 0, 0)],
        |_, _, _| {
            Some(Section179OtherLimitedDeduction {
                system: Section179IncomeSystem::Regular,
                amount_cents: 0,
            })
        },
    )
    .unwrap()
    .income;
    assert_eq!(zero.business_income_limit_cents, 6_000_000);
    assert_eq!(zero.k1[&2].business_income_carry_cents, 1_000_000);
    // A signed negative contribution offsets other active income before the
    // existing positive return-income limit; it is not floored per source.
    let negative = crate::section179::section179_with_native_prepared_income_and_operating_context(
        &draft,
        Section179IncomeSystem::Regular,
        &[],
        &[section179_income_source(2, (-70_000) * 100, 0, 0, 0)],
        |_, _, _| {
            Some(Section179OtherLimitedDeduction {
                system: Section179IncomeSystem::Regular,
                amount_cents: 0,
            })
        },
    )
    .unwrap()
    .income;
    assert_eq!(negative.business_income_limit_cents, 0);
    assert_eq!(negative.k1[&2].allowed_cents, 0);
    assert_eq!(negative.k1[&2].business_income_carry_cents, 7_000_000);
}

#[test]
fn unit17_income_source_checks_combined_inputs_without_changing_native_arithmetic() {
    let draft = ptp179_draft(5_000, 70_000);
    // This source-cent amount is representable, but cannot be added to the
    // untouched60000 wages in i64. Reject before allocation instead of panic
    // or wraparound. This is a transient API amount, not a new stored range.
    let huge = section179_income_source(2, i64::MAX - 1, 0, 0, 0);
    assert_eq!(
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            Section179IncomeSystem::Regular,
            &[],
            &[huge],
            |_, _, _| Some(Section179OtherLimitedDeduction {
                system: Section179IncomeSystem::Regular,
                amount_cents: 0
            })
        ),
        Err(crate::Section179PreparedIssue::AggregateAmountOverflow)
    );
    assert_eq!(crate::section179::calculate(&draft).income, 6_500_000);
}
