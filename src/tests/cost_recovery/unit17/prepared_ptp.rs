use super::*;
use alloc::collections::BTreeMap;

#[test]
fn unit17_prepared_ptp_retains_selected_allocation_payments_and_report_context() {
    let mut draft = ptp179_draft(20_000, 5_000);
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.section179_carryover = 200_000;
    data.guaranteed_services = 12_345;
    data.guaranteed_capital = 6_789;
    data.alternative.as_mut().unwrap().ordinary = 100_000;
    data.amt_section179_income_carry = Some(Section179IncomeCarryHistory {
        gross: 90_000,
        sold: 0,
        vintages: vec![Asset179Carryover { year: 2024, amount: 90_000 }],
    });
    let regular =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (20_000) * 100).unwrap();
    let alternative =
        selected_ptp179(&draft, Section179IncomeSystem::Alternative, 2, (1_000) * 100).unwrap();
    for (allocation, is_amt, income_cents, allowed, carry, year) in [
        (&regular, false, 2_000_000, 700_000, 200_000, None),
        (&alternative, true, 100_000, 590_000, 90_000, Some(2024)),
    ] {
        let source = allocation.ptp_section179_pal_input(2).unwrap();
        assert_eq!((source.k1, source.alternative, source.active), (2, is_amt, true));
        assert_eq!(source.owner_income_cents, income_cents);
        assert_eq!(
            (source.guaranteed_services_cents, source.guaranteed_capital_cents),
            (12_345, 6_789)
        );
        assert_eq!(source.allowed_cents, allowed);
        assert_eq!(source.business_income_carry_cents, 0);
        assert_eq!(
            source.components,
            vec![
                Ptp179ComponentAllocation {
                    component: Ptp179Component::Current,
                    available_cents: 500_000,
                    allowed_cents: 500_000
                },
                Ptp179ComponentAllocation {
                    component: Ptp179Component::PriorIncomeCarry { original_year: year },
                    available_cents: carry,
                    allowed_cents: carry
                },
            ]
        );
        let mut report = FederalResult::empty(draft.revision);
        allocation.ptp_report_context(&[]).unwrap().report(&mut report);
        assert_eq!(value(&report, "section179_prior_carryover_total"), carry / 100);
        assert_eq!(value(&report, "section179_deduction"), allowed / 100);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_SECTION179_PUBLICLY_TRADED")
        );
    }
    // A later edited source cannot alter the already-bound selected result.
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.amt_section179_income_carry = None;
    data.section179_carryover = 999_900;
    let mut report = FederalResult::empty(draft.revision);
    alternative.ptp_report_context(&[]).unwrap().report(&mut report);
    assert_eq!(value(&report, "section179_prior_carryover_total"), 900);
    assert_eq!(alternative.ptp_section179_pal_input(99), Err(Ptp179SourceIssue::UnknownSource));
}

#[test]
fn unit17_prepared_ptp_keeps_missing_selected_history_and_risk_unavailable() {
    let mut draft = ptp179_draft(20_000, 5_000);
    assert_eq!(
        selected_ptp179(&draft, Section179IncomeSystem::Alternative, 2, (20_000) * 100),
        Err(Section179PreparedIssue::K1Availability(
            2,
            K1Section179BasisIssue::AlternativeIncomeCarryUnavailable
        ))
    );
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    data.regular.risk.start = ScorpRiskStart::Unknown;
    data.alternative.as_mut().unwrap().risk.start = ScorpRiskStart::Unknown;
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        assert_eq!(
            selected_ptp179(&draft, system, 2, (20_000) * 100),
            Err(Section179PreparedIssue::K1Availability(
                2,
                K1Section179BasisIssue::SourceLedgerUnavailable(
                    "ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED"
                )
            ))
        );
    }
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.section179 = 0;
    let zero =
        selected_ptp179(&draft, Section179IncomeSystem::Alternative, 2, (20_000) * 100).unwrap();
    assert_eq!(zero.income.k1[&2].allowed_cents, 0);
    assert_eq!(zero.ptp_section179_pal_input(2), Err(Ptp179SourceIssue::RiskUnavailable));
}

#[test]
fn unit17_prepared_ptp_distinguishes_non_ptp_and_preserves_other_owner_issues() {
    let mut draft = ptp179_draft(20_000, 5_000);
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.publicly_traded = false;
    let allocation =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (20_000) * 100).unwrap();
    assert_eq!(allocation.ptp_section179_pal_input(2), Err(Ptp179SourceIssue::NotPtp));
    // A real owner deduction keeps its existing coordination refusal; the
    // source cache only excludes the exact receiving-owner PTP/PAL notices.
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.publicly_traded = true;
    data.pensions = 100;
    let allocation =
        selected_ptp179(&draft, Section179IncomeSystem::Regular, 2, (20_000) * 100).unwrap();
    let Err(Ptp179SourceIssue::OwnerIssues(issues)) = allocation.ptp_section179_pal_input(2) else {
        panic!("owner refusal must survive")
    };
    assert!(issues.iter().any(|issue| issue.display.key == "ISSUE_SECTION179_OWNER_DEDUCTIONS"));
    // A real unresolved partner deduction affects the same global allowance
    // read by an otherwise known direct business. Keep the actual owner issues,
    // not a successful zero share or a newly calculated regular allocation.
    add(&mut draft, income(cash_business(10_000)));
    let business = draft.income.last().unwrap().id;
    let system = Section179IncomeSystem::Regular;
    let direct: Vec<_> = section179_direct_income_sources(&draft, system)
        .unwrap()
        .iter()
        .map(|source| source.prepared_input().unwrap())
        .collect();
    let allocation =
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            system,
            &direct,
            &[Section179BusinessIncomeSource {
                source: 2,
                system,
                ordinary: Section179IncomeClass {
                    current_net_cents: 2_000_000,
                    prior_allowed_loss_cents: 0,
                },
                section1231: Section179IncomeClass {
                    current_net_cents: 0,
                    prior_allowed_loss_cents: 0,
                },
            }],
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
        )
        .unwrap();
    let Err(Ptp179SourceIssue::OwnerIssues(issues)) = allocation.ptp_section179_pal_input(2) else {
        panic!("the same actual owner refusal must survive direct reader binding")
    };
    assert!(issues.iter().any(|issue| issue.display.key == "ISSUE_SECTION179_OWNER_DEDUCTIONS"));
    for reader in [
        Section179PreparedAllocation::operating,
        Section179PreparedAllocation::allowed,
        Section179PreparedAllocation::expenses,
        Section179PreparedAllocation::tentative_profit,
        Section179PreparedAllocation::office,
    ] {
        assert_eq!(
            reader(&allocation, business),
            Err(Section179PreparedIssue::OwnerIssues(issues.clone()))
        );
        assert_eq!(reader(&allocation, 0), Err(Section179PreparedIssue::UnknownSource(0)));
        assert_eq!(reader(&allocation, 2), Err(Section179PreparedIssue::WrongSourceKind(2)));
    }
}

#[test]
fn unit17_prepared_ptp_regular_direct_assets_share_the_existing_allocator() {
    let mut draft = ptp179_draft(20_000, 5_000);
    let mut business = cash_business(10_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.assets.push(elected_asset(10_000, 3_000, BonusElection::ElectOut));
    add(&mut draft, income(business));
    // Accepted source identity follows the parent, as in the normal Save path.
    let FederalIncome::Business { data } = &mut draft.income[2].income else { unreachable!() };
    data.assets[0].id = draft.next_id;
    draft.next_id += 1;
    let FederalIncome::Business { data } = &draft.income[2].income else { unreachable!() };
    let operating = data.receipts()
        - data.operating_cost_with_adjustment(
            crate::depreciation::mid_quarter(&draft),
            crate::farm_conservation::expense_adjustment(&draft, 3, data),
        );
    assert_eq!(operating, 860_000); // $10,000 less actual pre179 MACRS $1,400.
    let system = Section179IncomeSystem::Regular;
    let allocation =
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            system,
            &[Section179DirectBusinessIncomeSource {
                source: 3,
                system,
                operating_net_cents: Some(operating),
                section1231_net_cents: Some(0),
            }],
            &[Section179BusinessIncomeSource {
                source: 2,
                system,
                ordinary: Section179IncomeClass {
                    current_net_cents: 2_000_000,
                    prior_allowed_loss_cents: 0,
                },
                section1231: Section179IncomeClass {
                    current_net_cents: 0,
                    prior_allowed_loss_cents: 0,
                },
            }],
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
        )
        .unwrap();
    assert_eq!(allocation.income.total_allowed_cents, 800_000);
    assert_eq!(allocation.assets, BTreeMap::from([((3, 4), 300_000)]));
    assert_eq!(allocation.ptp_section179_pal_input(2).unwrap().allowed_cents, 500_000);
    let mut selected = FederalResult::empty(draft.revision);
    allocation.ptp_report_context(&[]).unwrap().report(&mut selected);
    let mut native = FederalResult::empty(draft.revision);
    crate::section179::report(&draft, &mut native);
    assert_eq!(selected.lines, native.lines);
    assert_eq!(selected.issues, native.issues);
    assert_eq!(value(&selected, "business_3_asset_0_section179"), 3_000);
}

#[test]
fn unit17_prepared_ptp_does_not_copy_regular_prior_asset_carry_into_amt() {
    let mut draft = ptp179_draft(20_000, 5_000);
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    let mut business = cash_business(10_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.assets.push(carried_asset(2025, 3_000));
    add(&mut draft, income(business));
    let FederalIncome::Business { data } = &mut draft.income[2].income else { unreachable!() };
    let asset_id = draft.next_id;
    data.assets[0].id = asset_id;
    draft.next_id += 1;
    let system = Section179IncomeSystem::Alternative;
    let input = [Section179BusinessIncomeSource {
        source: 2,
        system,
        ordinary: Section179IncomeClass {
            current_net_cents: 2_000_000,
            prior_allowed_loss_cents: 0,
        },
        section1231: Section179IncomeClass { current_net_cents: 0, prior_allowed_loss_cents: 0 },
    }];
    let businesses = [Section179DirectBusinessIncomeSource {
        source: 3,
        system,
        operating_net_cents: Some(1_000_000),
        section1231_net_cents: Some(0),
    }];
    assert_eq!(
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            system,
            &businesses,
            &input,
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 })
        ),
        Err(Section179PreparedIssue::AlternativeAssetCarryUnavailable(3, 0))
    );
    // An actual separately retained AMT balance admits this same prior asset;
    // the regular $3,000 carry is not relabeled as the AMT $900 balance.
    let mut known = draft.clone();
    let FederalIncome::Business { data } = &mut known.income[2].income else { unreachable!() };
    data.assets[0].amt_section179_income_carry = Some(90_000);
    let allocation =
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &known,
            system,
            &businesses,
            &input,
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
        )
        .unwrap();
    assert_eq!(allocation.income.total_allowed_cents, 590_000);
    assert_eq!(allocation.assets, BTreeMap::from([((3, asset_id), 90_000)]));
    assert_eq!(allocation.ptp_section179_pal_input(2).unwrap().allowed_cents, 500_000);
    let mut selected = FederalResult::empty(known.revision);
    allocation.ptp_report_context(&[]).unwrap().report(&mut selected);
    assert_eq!(value(&selected, "business_3_asset_0_section179"), 900);
    let FederalIncome::Business { data } = &known.income[2].income else { unreachable!() };
    assert_eq!(data.assets[0].available179(), 300_000);
    assert_eq!(data.assets[0].amt_section179_carry(), Ok(90_000));
    // Actual continuing history with zero original179 supplies known zero;
    // ordinary selected depreciation still follows the existing projection.
    let FederalIncome::Business { data } = &mut draft.income[2].income else { unreachable!() };
    data.assets[0] = continuing_equipment(20250601, 20_000, false);
    data.assets[0].id = asset_id;
    assert!(
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            system,
            &businesses,
            &input,
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 })
        )
        .is_ok()
    );
}

#[test]
fn unit17_prepared_ptp_checked_income_is_not_clamped_or_replaced_by_zero() {
    use K1Section179BasisIssue as B;
    use Section179PreparedIssue as E;

    let draft = ptp179_draft(20_000, 5_000);
    let system = Section179IncomeSystem::Regular;
    let prepare = |sources: &[Section179BusinessIncomeSource]| {
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &draft,
            system,
            &[],
            sources,
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
        )
    };
    let source = section179_income_source(2, (20_000) * 100, 0, 0, 0);
    assert_eq!(prepare(&[source, source]), Err(E::DuplicateSource(2)));
    for (invalid, issue) in [
        (
            section179_income_source(999, (20_000) * 100, 0, 0, 0),
            E::K1Availability(999, B::UnknownSource),
        ),
        (section179_income_source(1, (20_000) * 100, 0, 0, 0), E::K1Availability(1, B::NotK1)),
        (section179_income_source(2, (20_000) * 100, -100, 0, 0), E::NegativePriorLoss(2)),
        (section179_income_source(2, (20_000) * 100, 0, 0, -100), E::NegativePriorLoss(2)),
        // Each class is representable; adding the second actual-cent class overflows.
        (section179_income_source(2, 100, 0, i64::MAX, 0), E::AmountOverflow(2)),
        (
            Section179BusinessIncomeSource {
                system: Section179IncomeSystem::Alternative,
                ..source
            },
            E::WrongSystem(2),
        ),
    ] {
        assert_eq!(prepare(&[invalid]), Err(issue));
    }
    let mut invalid = draft.clone();
    let FederalIncome::Partnership { data } = &mut invalid.income[1].income else { unreachable!() };
    data.regular.basis.opening = -1;
    // Preserve the original unavailable-source family without assuming which native
    // validation/ledger stage reports the invalid basis record first.
    assert!(matches!(
        crate::section179::section179_with_native_prepared_income_and_operating_context(
            &invalid,
            system,
            &[],
            &[source],
            |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
        ),
        Err(E::K1Availability(2, B::InvalidSource(_) | B::SourceLedgerUnavailable(_)))
    ));
}
