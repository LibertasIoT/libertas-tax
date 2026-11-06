use super::*;
use crate::tests::cross::interview::{
    accepted, business, document, reopened_businesses, save_businesses, save_topic, stored_business,
};

#[test]
fn unit17_r1_property_identity_survives_save_restart_reorder_and_rejects_rebinding() {
    let mut session = restart(empty_return(FederalStatus::Single));
    let mut page = business(10_000_000);
    page.assets = vec![
        crate::tests::cross::interview::asset(100_000, 20260301, AssetClass::Five),
        crate::tests::cross::interview::asset(200_000, 20260401, AssetClass::Seven),
    ];
    accepted(&save_businesses(&mut session, vec![page]));
    let (income_id, stored) = stored_business(&session);
    assert!(stored.assets.iter().all(|asset| asset.id > 0));
    assert_ne!(stored.assets[0].id, stored.assets[1].id);
    assert!(session.draft.valid_stored());
    let before = supported(&session.draft);
    let mut page = reopened_businesses(session.draft.clone()).remove(0);
    let ids: Vec<_> = page.assets.iter().map(|asset| asset.id).collect();
    page.assets.reverse();
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(
            income_id,
            IncomeSource::Record { value: crate::IncomeRecord::Business(page.clone()) },
        )],
    );
    accepted(&send(&mut session, request));
    let (_, reordered) = stored_business(&session);
    assert_eq!(
        reordered.assets.iter().map(|asset| asset.id).collect::<Vec<_>>(),
        vec![ids[1], ids[0]]
    );
    assert_eq!(value(&supported(&session.draft), "agi"), value(&before, "agi"));
    page.more.research_property_choices = crate::cohorts::property_choices(&reordered);
    assert_eq!(reopened_businesses(session.draft.clone()), vec![page.clone()]);
    // An existing property cannot be submitted as the child of another business.
    let new_business = business(1_000_000);
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![
            document(
                income_id,
                IncomeSource::Record { value: crate::IncomeRecord::Business(new_business) },
            ),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) }),
        ],
    );
    let original = session.draft.clone();
    crate::tests::cross::interview::rejected_with(
        send(&mut session, request),
        "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
    );
    assert_eq!(session.draft, original);
}

#[test]
fn unit17_depreciation_source_prepares_distinct_actual_amt_before_179_income() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { mut data } = cash_business(11_800) else { unreachable!() };
    let mut older_acquired = depreciable_asset(14_000, EquipmentClass::Five, 20260101, 0);
    older_acquired.acquired = 20170927;
    older_acquired.id = 3;
    let mut elected = depreciable_asset(10_000, EquipmentClass::Five, 20260101, 10_000);
    elected.id = 4;
    data.assets = vec![older_acquired, elected];
    add(&mut draft, income(FederalIncome::Business { data }));
    draft.next_id = 5;
    assert!(draft.valid_stored());
    let FederalIncome::Business { data } = &draft.income[1].income else { unreachable!() };
    assert_eq!(data.assets.iter().map(|asset| asset.id).collect::<Vec<_>>(), vec![3, 4]);
    let mid_quarter = crate::depreciation::mid_quarter(&draft);
    // First-use property acquired before bonus eligibility:5year HY regular
    //200%DB is20%, AMT150%DB is15%, giving2800/2100. The separate10000
    //section179 election reduces that asset's depreciation basis, not these
    //before179 operating-income amounts. IRS6251's9000/9700 income example.
    assert_eq!(data.assets[0].deduction(mid_quarter), 280_000);
    assert_eq!(data.assets[0].deduction_for(mid_quarter, true), 210_000);
    assert_eq!(data.assets[1].deduction(mid_quarter), 0);
    assert_eq!(data.assets[1].deduction_for(mid_quarter, true), 0);
    assert_eq!(
        data.receipts() - data.assets.iter().map(|asset| asset.deduction(mid_quarter)).sum::<i64>(),
        900_000
    );
    assert_eq!(
        data.receipts()
            - data.assets.iter().map(|asset| asset.deduction_for(mid_quarter, true)).sum::<i64>(),
        970_000
    );
    assert_eq!(crate::depreciation::business_amt_adjustment(data, mid_quarter), 70_000);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn unit17_depreciation_keeps_actual_regular_and_amt_history() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { mut data } = cash_business(20_000) else { unreachable!() };
    let mut asset = continuing_asset(
        1_000_000,
        20250101,
        EquipmentClass::Five,
        200_000,
        crate::AssetDisposal::No,
    );
    asset.acquired = 20170927; // Actual pre-bonus acquisition makes separate AMT depreciation applicable.
    asset.id = 3;
    if let EquipmentHistory::Continuing { amt, .. } = &mut asset.history {
        *amt = EquipmentAmtHistory::Separate {
            prior_depreciation: 150_000,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        };
    }
    data.assets = vec![asset];
    add(&mut draft, income(FederalIncome::Business { data }));
    draft.next_id = 4;
    let FederalIncome::Business { data } = &draft.income[1].income else { unreachable!() };
    let mid_quarter = crate::depreciation::mid_quarter(&draft);
    assert_eq!(data.assets[0].deduction(mid_quarter), 320_000);
    assert_eq!(data.assets[0].deduction_for(mid_quarter, true), 255_000);
    assert_eq!(crate::depreciation::business_amt_adjustment(data, mid_quarter), 65_000);
}

#[test]
fn unit17_depreciation_source_reuses_the_identical_asset_owner_for_farm() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { mut data } = raised_farm(50_000, 0) else { unreachable!() };
    let mut asset = depreciable_asset(14_000, EquipmentClass::Five, 20260101, 0);
    asset.acquired = 20170927;
    asset.id = 3;
    data.assets = vec![asset];
    add(&mut draft, income(FederalIncome::Business { data }));
    draft.next_id = 4;
    assert!(draft.valid_stored());
    let FederalIncome::Business { data } = &draft.income[1].income else { unreachable!() };
    let mid_quarter = crate::depreciation::mid_quarter(&draft);
    assert_eq!(data.assets[0].id, 3);
    assert_eq!(data.assets[0].deduction(mid_quarter), 280_000);
    assert_eq!(data.assets[0].deduction_for(mid_quarter, true), 210_000);
    assert_eq!(crate::depreciation::business_amt_adjustment(data, mid_quarter), 70_000);
}

#[test]
fn unit17_direct_amt_carry_comes_from_actual_existing_annual_components() {
    let mut asset = continuing_asset(
        2_000_000,
        20250101,
        EquipmentClass::Five,
        1_100_000,
        crate::AssetDisposal::No,
    );
    asset.id = 3;
    asset.acquired = 20170927;
    let EquipmentHistory::Continuing { initial_section179, carryforward, amt, .. } =
        &mut asset.history
    else {
        unreachable!()
    };
    *initial_section179 = 1_000_000;
    *carryforward = 100_000;
    *amt = EquipmentAmtHistory::Separate {
        prior_depreciation: 1_120_000,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    asset.history_details = Some(direct_amt_carry_history());
    assert!(asset.valid());
    // Existing cumulative AMT11200 less actual OTHER AMT1500 identifies
    // AMT179 allowed9700 and carry300. Regular11000−other2000 gives9000
    // allowed and1000carry. No past income/MACRS figure is reconstructed.
    asset.amt_section179_income_carry = Some(30_001);
    assert!(!asset.valid());
    assert_eq!(asset.amt_section179_carry(), Err(()));
    asset.amt_section179_income_carry = Some(30_000);
    assert!(asset.valid());
    asset.amt_section179_income_carry = None; // Derivation needs no duplicate fact.
    assert_eq!(asset.amt_section179_carry(), Ok(30_000));
    assert_eq!(asset.available179(), 100_000);
    let alternative = asset.amt_projection_or_self();
    assert_eq!(alternative.available179(), 30_000);
    assert_eq!(asset.opening_basis(), 800_000);
    assert_eq!(alternative.opening_basis(), 850_000);
    let FederalIncome::Business { mut data } = cash_business(20_000) else { unreachable!() };
    data.assets = vec![asset.clone()];
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Business { data }));
    draft.next_id = 4;
    assert!(draft.valid_stored());
    assert_eq!(asset.deduction(false), 320_000);
    assert_eq!(asset.deduction_for(false, true), 255_000);
    assert_eq!(restart(draft.clone()).draft, draft);

    if let EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Separate { prior_depreciation, .. },
        ..
    } = &mut asset.history
    {
        *prior_depreciation = 1_150_000;
    }
    assert_eq!(asset.amt_section179_carry(), Ok(0)); // Known full10000 allowed.
    assert_eq!(asset.amt_projection_or_self().available179(), 0);
    if let EquipmentHistory::Continuing {
        prior_depreciation,
        carryforward,
        amt: EquipmentAmtHistory::Separate { prior_depreciation: alternative_prior, .. },
        ..
    } = &mut asset.history
    {
        *prior_depreciation = 1_200_000;
        *carryforward = 0;
        *alternative_prior = 1_120_000;
    }
    let records = asset.history_details.as_mut().unwrap();
    records.years[0].section179_allowed = 1_000_000;
    records.carryovers.clear();
    assert!(asset.valid());
    assert_eq!(asset.available179(), 0);
    assert_eq!(asset.amt_section179_carry(), Ok(30_000));
    assert_eq!(asset.amt_projection_or_self().available179(), 30_000);
    asset.history_details.as_mut().unwrap().years.clear();
    assert_eq!(asset.amt_section179_carry(), Err(())); // Unknown is not zero.
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.assets[0].history_details = None;
    assert_eq!(data.assets[0].amt_section179_carry(), Err(()));
}

#[test]
fn unit17_direct_amt_carry_same_macrs_uses_actual_source_without_regular_copy() {
    let mut asset = continuing_asset(
        2_000_000,
        20250101,
        EquipmentClass::Five,
        1_100_000,
        crate::AssetDisposal::No,
    );
    asset.id = 3;
    asset.acquired = 20250101;
    if let EquipmentHistory::Continuing { initial_section179, carryforward, .. } =
        &mut asset.history
    {
        *initial_section179 = 1_000_000;
        *carryforward = 100_000;
    }
    assert!(asset.valid());
    assert!(asset.qualifies());
    assert_eq!(asset.amt_section179_carry(), Err(()));
    // Missing is not the regular1000 and not knownzero. The ordinary stored
    // record may retain an unanswered source; the checked supplier refuses it.
    asset.amt_section179_income_carry = Some(30_000);
    assert!(asset.valid());
    assert!(asset.qualifies());
    assert_eq!(asset.amt_section179_carry(), Ok(30_000));
    let alternative = asset.amt_projection_or_self();
    assert_eq!(asset.available179(), 100_000);
    assert_eq!(alternative.available179(), 30_000);
    assert_eq!(asset.opening_basis(), 800_000);
    assert_eq!(alternative.opening_basis(), 800_000);
    assert_eq!(asset.deduction_for(false, false), 320_000);
    assert_eq!(asset.deduction_for(false, true), 320_000);
    let FederalIncome::Business { mut data } = cash_business(20_000) else { unreachable!() };
    data.assets = vec![asset.clone()];
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Business { data }));
    draft.next_id = 4;
    assert!(draft.valid_stored());
    assert_eq!(restart(draft.clone()).draft, draft);
    assert_eq!(asset.deduction(false), 320_000);
    assert_eq!(asset.deduction_for(false, true), 320_000);
    asset.amt_section179_income_carry = Some(0);
    assert_eq!(asset.amt_section179_carry(), Ok(0));
    assert_eq!(asset.amt_projection_or_self().available179(), 0);
    if let EquipmentHistory::Continuing { carryforward, prior_depreciation, .. } =
        &mut asset.history
    {
        *carryforward = 0;
        *prior_depreciation = 1_200_000;
    }
    asset.amt_section179_income_carry = Some(30_000);
    assert!(asset.valid());
    assert_eq!(asset.available179(), 0);
    assert_eq!(asset.amt_projection_or_self().available179(), 30_000);
    assert_eq!(asset.amt_projection_or_self().opening_basis(), 800_000);
    asset.amt_section179_income_carry = Some(1_000_001);
    assert!(!asset.valid());
    asset.amt_section179_income_carry = None;
    if let EquipmentHistory::Continuing { amt, .. } = &mut asset.history {
        *amt = EquipmentAmtHistory::Separate {
            prior_depreciation: 1_170_000,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        };
    }
    // The existing Separate option is specifically a150%DB refigure for
    // pre-bonus-eligible acquisition, not a truthful same-MACRS179 split.
    assert!(!asset.qualifies());
}

#[test]
fn unit17_r2_original_service_year_caps_do_not_restart_in_2026() {
    use crate::HistoricalAutoKind::{Car, ElectricCar, TruckVan};
    use crate::auto_recovery::caps::original_auto_limit_cents as cap;
    // Pub. 946 (2006), pp.63–64. Electric limits have their own indexed
    // rounding; three times the ordinary $1,775 would be the wrong $5,325.
    assert_eq!(cap(20060101, 20060101, 2026, &Car, false), Some(177_500));
    assert_eq!(cap(20060101, 20060101, 2026, &ElectricCar, false), Some(522_500));
    assert_eq!(cap(20060101, 20060101, 2026, &TruckVan, false), Some(187_500));
    assert_eq!(cap(19970805, 19970805, 2026, &ElectricCar, false), None);
    assert_eq!(cap(19970806, 19970806, 2026, &ElectricCar, false), Some(542_500));
    assert_eq!(cap(20070101, 20070101, 2026, &ElectricCar, false), None);
    // Rev. Proc.2018-25 and2019-13: $18,000 in2018, $16,000 in2019,
    // and the original $5,760 fourth-and-later limit in2026.
    assert_eq!(cap(20180101, 20180101, 2018, &Car, true), Some(1_800_000));
    assert_eq!(cap(20180101, 20180101, 2019, &Car, true), Some(1_600_000));
    assert_eq!(cap(20180101, 20180101, 2026, &Car, true), Some(576_000));
    // The old-acquisition additional first-year limits phase down separately
    // from the original year's ordinary cap (Pub.463, chapter4 footnotes).
    assert_eq!(cap(20170927, 20180101, 2018, &Car, true), Some(1_640_000));
    assert_eq!(cap(20170927, 20190101, 2019, &Car, true), Some(1_490_000));
    assert_eq!(cap(20170927, 20200101, 2020, &Car, true), Some(1_010_000));
    assert_eq!(cap(20170928, 20200101, 2020, &Car, true), Some(1_810_000));
    // An absent original table has no2026 fallback, nor a future-service cap.
    // 2026-10-18: Pub463 (1997), printed p18, supplies the original1996 later cap.
    assert_eq!(cap(19960101, 19960101, 2026, &Car, false), Some(177_500));
    assert_eq!(cap(19860101, 19860101, 2026, &Car, false), None);
    assert_eq!(cap(20260101, 20260101, 2025, &Car, true), None);
}

#[test]
fn unit17_d3_original_electric_class_changes_cap_without_changing_election_basis() {
    let mut asset = bonus_equipment(2_500, 100);
    asset.acquired = 20060101;
    asset.placed = 20060101;
    asset.history = EquipmentHistory::Continuing {
        initial_section179: 250_000,
        carryforward: 250_000,
        prior_depreciation: 0,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    asset.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 10_000,
        operating_costs: 0,
        loan_interest: 0,
        kind: crate::VehicleKind::PassengerCar,
    };
    asset.history_details = Some(HistoricalAssetRecords {
        original_business_basis: 250_000,
        original_income_use: UseFraction { used: 1, total: 1 },
        original_qualified_use: UseFraction { used: 1, total: 1 },
        original_section179_elected: 250_000,
        original_bonus: 0,
        years: (2006..2026)
            .map(|year| AssetYearHistory {
                year,
                income_use: UseFraction { used: 1, total: 1 },
                qualified_use: UseFraction { used: 1, total: 1 },
                section179_allowed: 0,
                amt_section179_allowed: crate::SourceCents::NotReported,
                bonus: 0,
                regular: 0,
                amt: 0,
                basis_increase: 0,
                basis_decrease: 0,
                recaptured: 0,
            })
            .collect(),
        carryovers: vec![Asset179Carryover { year: 2006, amount: 250_000 }],
        convention: AssetConvention::HalfYear,
        original_method: OriginalAssetMethod::Declining200,
        auto_history: Some(AutoRecoveryHistory {
            first_year_bonus: OriginalBonusTreatment::Ineligible,
            safe_harbor_adopted_year: None,
            car_or_truck: HistoricalAutoKind::Car,
            first_year_capped_deduction: 0,
            post_recovery_basis: 0,
        }),
    });
    assert_eq!(asset.automobile_limit(), Some(177_500));
    assert_eq!(asset.available179(), 250_000);
    assert_eq!(asset.opening_basis(), 0);
    asset.history_details.as_mut().unwrap().auto_history.as_mut().unwrap().car_or_truck =
        HistoricalAutoKind::ElectricCar;
    assert_eq!(asset.automobile_limit(), Some(522_500));
    assert_eq!(asset.available179(), 250_000);
    assert_eq!(asset.opening_basis(), 0);
    for (kind, allowed, carry) in
        [(HistoricalAutoKind::Car, 177_500, 72_500), (HistoricalAutoKind::ElectricCar, 250_000, 0)]
    {
        let mut claimed = asset.clone();
        claimed.history_details.as_mut().unwrap().auto_history.as_mut().unwrap().car_or_truck =
            kind;
        let mut d = complete(FederalStatus::Single);
        let mut entry = income(cash_business(10_000));
        let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
        data.assets.push(claimed);
        let id = d.next_id;
        add(&mut d, entry);
        let annual = crate::section179::calculate(&d);
        assert_eq!(annual.available, 250_000);
        assert_eq!(annual.allowed, allowed);
        assert_eq!(annual.assets.get(&(id, 0)), Some(&allowed));
        let result = supported(&d);
        assert_eq!(
            value(&result, &alloc::format!("business_{id}_asset_0_section179")),
            allowed / 100
        );
        assert_eq!(
            value(&result, &alloc::format!("business_{id}_asset_0_section179_carryforward")),
            carry / 100
        );
        assert_eq!(estimate(&restart(d.clone()).draft), result);
        let FederalIncome::Business { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.assets[0].history_details.as_mut().unwrap().years.pop();
        assert!(!data.assets[0].qualifies());
    }
    // The annual cap releases an existing income-limit balance without
    // relabeling it as bonus/MACRS basis. Other historical routes remain due.
}

#[test]
fn unit17_a_b_original_auto_schedules_and_post_recovery_basis() {
    // Rev.Proc.2019-13 Example1 retains its published whole-dollar annual
    // amounts and$51,599 total through2023, leaving$8,401, not$8,400.
    let a = documented_auto(
        &[
            (2018, 18_000),
            (2019, 13_440),
            (2020, 8_064),
            (2021, 4_838),
            (2022, 4_838),
            (2023, 2_419),
            (2024, 5_760),
            (2025, 2_641),
        ],
        0,
        1_800_000,
        true,
        840_100,
    );
    assert!(a.valid() && a.qualifies() && a.vehicle_supported());
    for (year, expected) in [
        (2018, 18_000),
        (2019, 13_440),
        (2020, 8_064),
        (2021, 4_838),
        (2022, 4_838),
        (2023, 2_419),
        (2024, 5_760),
        (2025, 2_641),
        (2026, 0),
    ] {
        assert_eq!(
            crate::kernel::ratio::dollars(crate::auto_recovery::amount(&a, year).unwrap()),
            expected,
            "A {year}"
        );
    }
    assert_eq!(a.deduction(false), 0);
    // Original179 election excludes the safe harbor. The$42,000 excess
    // starts after the original five-year recovery, not a new2026 clock.
    let b = documented_auto(
        &[
            (2018, 18_000),
            (2019, 0),
            (2020, 0),
            (2021, 0),
            (2022, 0),
            (2023, 0),
            (2024, 5_760),
            (2025, 5_760),
        ],
        1_800_000,
        0,
        false,
        4_200_000,
    );
    assert!(b.valid() && b.qualifies() && b.vehicle_supported());
    assert_eq!(b.opening_basis(), 3_048_000);
    assert_eq!(b.deduction(false), 576_000);
    assert_eq!(b.deduction_for(false, true), 576_000);
    let result = supported(&business_with_bonus(100_000, b.clone()));
    assert_eq!(value(&result, "business_2_asset_0_macrs"), 5_760);
    assert_eq!(value(&result, "business_2_asset_0_remaining_basis"), 24_720);
    assert_eq!(value(&result, "business_2_net_profit"), 94_240);
    let mut wrong = a.clone();
    wrong.history_details.as_mut().unwrap().auto_history.as_mut().unwrap().post_recovery_basis =
        840_000;
    assert!(!wrong.valid());
    let mut wrong = b.clone();
    wrong
        .history_details
        .as_mut()
        .unwrap()
        .auto_history
        .as_mut()
        .unwrap()
        .safe_harbor_adopted_year = Some(2019);
    assert!(!wrong.valid());
    let mut reordered = b.clone();
    reordered.history_details.as_mut().unwrap().years.reverse();
    assert_eq!(reordered.deduction(false), b.deduction(false));
    reordered.history_details.as_mut().unwrap().years.pop();
    assert!(!reordered.valid());
}

#[test]
fn unit17_partial_bonus_uses_nominal_macrs_basis_and_defers_cap_excess() {
    let partial = crate::tests::support::builders::partial_bonus_auto();
    assert!(partial.valid() && partial.qualifies() && partial.vehicle_supported());
    //80% of$60,000 is nominal bonus$48,000, capped at$20,200. Annual
    // MACRS uses the residual$12,000, not$39,800 after the actual capped bonus.
    assert_eq!(partial.opening_basis(), 3_365_600);
    assert_eq!(partial.deduction(false), 138_240);
    let result = supported(&business_with_bonus(100_000, partial.clone()));
    assert_eq!(value(&result, "business_2_asset_0_macrs"), 1_382);
    // A restored100%2025 allowance is calculable without promising the
    //2019 safe harbor's extension: its cap excess waits until2031.
    let mut current = documented_auto(&[(2025, 20_200)], 0, 2_020_000, false, 3_980_000);
    current.acquired = 20250120;
    current.placed = 20250201;
    assert!(current.valid() && current.qualifies());
    assert_eq!(current.deduction(false), 0);
    let mut invented_adoption = current;
    invented_adoption
        .history_details
        .as_mut()
        .unwrap()
        .auto_history
        .as_mut()
        .unwrap()
        .safe_harbor_adopted_year = Some(2026);
    invented_adoption.history_details.as_mut().unwrap().original_method =
        OriginalAssetMethod::SafeHarbor2019;
    assert!(!invented_adoption.valid());
}

#[test]
fn unit17_original_auto_editor_preserves_acquisition_and_method_on_save_restart() {
    for partial80 in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        let FederalIncome::Business { mut data } = cash_business(100_000) else { unreachable!() };
        data.assets = crate::tests::support::builders::original_auto_editor_sources();
        if partial80 {
            data.assets[1] = crate::tests::support::builders::partial_bonus_auto();
        }
        for (index, asset) in data.assets.iter_mut().enumerate() {
            asset.id = 3 + index as i64;
            assert!(asset.valid() && asset.qualifies() && asset.vehicle_supported());
        }
        let expected = data.assets.clone();
        add(&mut draft, income(FederalIncome::Business { data }));
        draft.next_id = 7;
        assert!(draft.valid_stored());
        let mut session = restart(draft);
        let page = reopened_businesses(session.draft.clone()).remove(0);
        for (vehicle, source) in page.vehicles.iter().zip(&expected) {
            let VehicleMethod::Actual { acquired, history_details, .. } = &vehicle.method else {
                unreachable!()
            };
            assert_eq!(*acquired, (source.acquired != source.placed).then_some(source.acquired));
            assert_eq!(history_details.as_deref(), source.history_details.as_ref());
        }
        let (id, _) = stored_business(&session);
        let save = |session: &mut Interview, page| {
            let request = save_topic(
                session,
                TaxSection::SelfEmployment,
                vec![document(
                    id,
                    IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
                )],
            );
            accepted(&send(session, request));
        };
        save(&mut session, page);
        let (_, saved) = stored_business(&session);
        assert_eq!(saved.assets, expected);
        // Original acquisition controls bonus eligibility; the 2018 methods
        // retain the original recovery schedule (Rev. Proc. 2019-13).
        assert_eq!(
            saved.assets.iter().map(|asset| asset.deduction(false)).collect::<Vec<_>>(),
            vec![280_000, if partial80 { 138_240 } else { 1_152_000 }, 576_000, 576_000]
        );
        let accepted_draft = session.draft.clone();
        let result = supported(&accepted_draft);
        // 2026-10-20: i4562 line 26(e) subtracts original nominal bonus and
        // elected section 179, not actual prior capped depreciation.
        for (index, basis) in
            [14_000, if partial80 { 12_000 } else { 36_000 }, 0, 0].into_iter().enumerate()
        {
            assert_eq!(
                value(
                    &result,
                    &alloc::format!("business_{id}_asset_{index}_listed_depreciation_basis")
                ),
                basis
            );
            assert_eq!(
                value(&result, &alloc::format!("business_{id}_asset_{index}_listed_section179")),
                0
            );
        }
        session = restart(accepted_draft.clone());
        let page = reopened_businesses(session.draft.clone()).remove(0);
        save(&mut session, page.clone());
        assert_eq!(session.draft, accepted_draft);
        assert_eq!(supported(&session.draft), result);
        let mut invalid = page;
        let VehicleMethod::Actual { acquired, placed, .. } = &mut invalid.vehicles[0].method else {
            unreachable!()
        };
        *acquired = Some(*placed + 1);
        let request = save_topic(
            &mut session,
            TaxSection::SelfEmployment,
            vec![document(
                id,
                IncomeSource::Record { value: crate::IncomeRecord::Business(invalid) },
            )],
        );
        crate::tests::cross::interview::rejected_with(
            send(&mut session, request),
            "ASSET_ACQUIRED_AFTER_PLACED",
        );
        assert_eq!(session.draft, accepted_draft);
    }
}

#[test]
fn unit17_pre2018_original_car_truck_electric_recovery_reuses_original_caps() {
    for (kind, current, opening, closing) in [
        (HistoricalAutoKind::Car, 1_775, 139_215, 137_440),
        (HistoricalAutoKind::TruckVan, 1_875, 136_515, 134_640),
        (HistoricalAutoKind::ElectricCar, 5_225, 59_145, 53_920),
    ] {
        let asset = ordinary_original_auto(kind);
        for (method, original) in [
            (EquipmentMethod::Regular, OriginalAssetMethod::Declining200),
            (EquipmentMethod::Declining150, OriginalAssetMethod::Declining150),
            (EquipmentMethod::StraightLine, OriginalAssetMethod::StraightLine),
        ] {
            for mid_quarter in [false, true] {
                let mut asset = asset.clone();
                asset.method = method;
                let records = asset.history_details.as_mut().unwrap();
                records.original_method = original.clone();
                records.convention = if mid_quarter {
                    AssetConvention::MidQuarter
                } else {
                    AssetConvention::HalfYear
                };
                let EquipmentHistory::Continuing { mid_quarter: saved, .. } = &mut asset.history
                else {
                    unreachable!()
                };
                *saved = mid_quarter;
                assert!(asset.valid() && asset.qualifies() && asset.vehicle_supported());
                assert_eq!(asset.opening_basis(), opening * 100);
                assert_eq!(asset.deduction(false), current * 100);
            }
        }
        let result = supported(&business_with_bonus(100_000, asset));
        assert_eq!(value(&result, "business_2_asset_0_macrs"), current);
        assert_eq!(value(&result, "business_2_asset_0_remaining_basis"), closing);
        assert_eq!(value(&result, "business_2_net_profit"), 100_000 - current);
    }
}

#[test]
fn unit17_original_auto_missing_duplicate_use_change_adjusted_history_stays_unavailable() {
    for original in [
        ordinary_original_auto(HistoricalAutoKind::ElectricCar),
        historical_original_auto(1987, [2560, 4100, 2450, 1475], HistoricalAutoKind::Car),
    ] {
        for change in 0..6 {
            let mut asset = original.clone();
            let records = asset.history_details.as_mut().unwrap();
            match change {
                0 => {
                    records.years.pop();
                }
                1 => records.years[0].year = records.years[1].year,
                2 => records.years[0].income_use.used = 0,
                3 => records.years[0].qualified_use.used = 0,
                4 => records.years[0].basis_increase = 1,
                _ => records.years[0].recaptured = 1,
            }
            assert_eq!(crate::auto_recovery::amount(&asset, 2026), None);
            assert!(!asset.valid());
        }
    }
}

#[test]
fn unit17_original_1987_1996_caps_use_each_published_annual_column() {
    use crate::HistoricalAutoKind::{Car, ElectricCar, TruckVan};
    use crate::auto_recovery::caps::original_auto_limit_cents as cap;
    // Pub917 (1994), p10 Table2; Pub463 (1997), p18. Dollars as published.
    for (year, columns) in [
        (1987, [2560, 4100, 2450, 1475]),
        (1988, [2560, 4100, 2450, 1475]),
        (1989, [2660, 4200, 2550, 1475]),
        (1990, [2660, 4200, 2550, 1475]),
        (1991, [2660, 4300, 2550, 1575]),
        (1992, [2760, 4400, 2650, 1575]),
        (1993, [2860, 4600, 2750, 1675]),
        (1994, [2960, 4700, 2850, 1675]),
        (1995, [3060, 4900, 2950, 1775]),
        (1996, [3060, 4900, 2950, 1775]),
    ] {
        let placed = year * 10000 + 701;
        for kind in [Car, TruckVan] {
            for (offset, dollars) in columns.into_iter().enumerate() {
                assert_eq!(
                    cap(placed, placed, year + offset as u32, &kind, false),
                    Some(dollars * 100)
                );
            }
            assert_eq!(cap(placed, placed, 2026, &kind, false), Some(columns[3] * 100));
            assert_eq!(cap(placed, placed, year, &kind, true), Some(columns[0] * 100));
            assert_eq!(cap(placed, placed, year - 1, &kind, false), None);
        }
        assert_eq!(cap(placed, placed, 2026, &ElectricCar, false), None);
    }
}

#[test]
fn unit17_original_1987_1996_actual_histories_reach_native_deduction() {
    // Derived stable-use histories from the published cap rows, not IRS worked examples.
    for (year, columns, opening, closing) in [
        (1987, [2560, 4100, 2450, 1475], 117_790, 116_315),
        (1991, [2660, 4300, 2550, 1575], 120_090, 118_515),
        (1996, [3060, 4900, 2950, 1775], 121_165, 119_390),
    ] {
        let asset = historical_original_auto(year, columns, HistoricalAutoKind::Car);
        assert!(asset.valid() && asset.qualifies() && asset.vehicle_supported());
        assert_eq!(asset.opening_basis(), opening * 100);
        assert_eq!(asset.deduction(false), columns[3] * 100);
        let result = supported(&business_with_bonus(100_000, asset));
        assert_eq!(value(&result, "business_2_asset_0_macrs"), columns[3]);
        assert_eq!(value(&result, "business_2_asset_0_remaining_basis"), closing);
        assert_eq!(value(&result, "business_2_net_profit"), 100_000 - columns[3]);
    }
}

#[test]
fn unit17_inconsistent_original_auto_history_is_located_and_save_is_atomic() {
    let source = historical_original_auto(1987, [2560, 4100, 2450, 1475], HistoricalAutoKind::Car);
    let mut page = business(10_000_000);
    page.vehicles = crate::interview::vehicle_and_asset_items(&[], &[source]).unwrap().0;
    let mut session = restart(empty_return(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![page]));
    let accepted_draft = session.draft.clone();
    let result = supported(&accepted_draft);
    session = restart(accepted_draft.clone());
    let (id, _) = stored_business(&session);
    let page = reopened_businesses(session.draft.clone()).remove(0);
    let save = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(
            id,
            IncomeSource::Record { value: crate::IncomeRecord::Business(page.clone()) },
        )],
    );
    accepted(&send(&mut session, save));
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(supported(&session.draft), result);
    for missing in [false, true] {
        let mut invalid = page.clone();
        let VehicleMethod::Actual { history_details: Some(history), .. } =
            &mut invalid.vehicles[0].method
        else {
            unreachable!()
        };
        if missing {
            assert_eq!(history.years.pop().unwrap().year, 2025);
        } else {
            history.years[0].regular = 100;
        }
        let request = save_topic(
            &mut session,
            TaxSection::SelfEmployment,
            vec![document(
                id,
                IncomeSource::Record { value: crate::IncomeRecord::Business(invalid) },
            )],
        );
        crate::tests::support::assert::k1res::assert_located_rejection(
            &mut session,
            request,
            "AUTO_RECOVERY_HISTORY_INCONSISTENT",
            "[documents/documents/0/document/vehicles/0/method/history_details/years]",
        );
    }
}
