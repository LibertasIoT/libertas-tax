use super::*;

#[test]
fn macrs_first_year_manual_rates_cover_every_equipment_class_and_method() {
    for (class, regular, slower, straight) in [
        (EquipmentClass::Three, 400000, 300000, 200000),
        (EquipmentClass::Five, 240000, 180000, 120000),
        (EquipmentClass::Seven, 171429, 128571, 85714),
        (EquipmentClass::Ten, 120000, 90000, 60000),
        (EquipmentClass::Fifteen, 60000, 60000, 40000),
        (EquipmentClass::Twenty, 45000, 45000, 30000),
    ] {
        for (method, expected) in [
            (EquipmentMethod::Regular, regular),
            (EquipmentMethod::Declining150, slower),
            (EquipmentMethod::StraightLine, straight),
        ] {
            let asset = macrs_equipment(12000, class, method, 20260601);
            assert_eq!(asset.deduction(false), expected, "{class:?} {method:?}");
            let d = business_with_bonus(100000, asset);
            let r = supported(&d);
            let actual = r.lines.iter().find(|v| v.id == "business_2_asset_0_macrs").unwrap();
            assert_eq!(actual.amount, expected);
            assert_eq!(value(&r, "business_2_macrs_mid_quarter"), 0);
        }
    }
}

#[test]
fn phased_bonus_mid_quarter_and_class_election_out_are_separate() {
    let mut asset = bonus_equipment(10000, 100);
    asset.acquired = 20250119;
    asset.placed = 20261001;
    let mut d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_macrs_mid_quarter"), 1);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 2000);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 400);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[0].history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 500);
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 9500);
    // Both acquisition cohorts can claim bonus in one recovery class.
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[0].history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::Apply,
        tables: false,
    };
    data.assets.push(bonus_equipment(2000, 100));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_1_bonus"), 2000);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 400);
}

#[test]
fn section179_joint_income_shares_limits_and_separate_needs_spouse_records() {
    let mut d = complete(FederalStatus::Joint);
    for (owner, cost) in [(Owner::Taxpayer, 120000), (Owner::Spouse, 80000)] {
        let mut e = income(cash_business(0));
        e.owner = owner;
        let FederalIncome::Business { data } = &mut e.income else { panic!() };
        data.assets.push(elected_asset(cost, cost, BonusElection::ElectOut));
        add(&mut d, e);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 100000);
    assert_eq!(value(&r, "section179_carryforward"), 100000);
    let mut separate = complete(FederalStatus::Separate);
    let mut e = income(cash_business(10000));
    let FederalIncome::Business { data } = &mut e.income else { panic!() };
    data.assets.push(elected_asset(1000, 1000, BonusElection::ElectOut));
    add(&mut separate, e);
    // Spouse records not entered: no section 179 on this separate return,
    // the election depreciated normally, with a warning.
    let r = supported(&separate);
    assert!(noted(&r, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"), "{r:?}");
}

#[test]
fn separate_section179_combines_purchases_before_sharing_limit() {
    for (share, expected) in [
        (Section179Share::Equal, 1175000),
        (Section179Share::Elected { yours: 9500, spouse: 500 }, 2232500),
        (Section179Share::Elected { yours: 0, spouse: 10000 }, 0),
        (Section179Share::Elected { yours: 10000, spouse: 0 }, 2350000),
        (Section179Share::Elected { yours: 3333, spouse: 6667 }, 783255),
        (Section179Share::Elected { yours: 9500, spouse: 1000 }, 1175000),
    ] {
        let d = separate179(4000000, expected, 300000, share);
        let calc = crate::section179::calculate(&d);
        assert_eq!(calc.shared_limit, 235000000);
        assert_eq!(calc.limit, expected * 100);
        if expected > 0 {
            let r = supported(&d);
            assert_eq!(value(&r, "section179_spouse_eligible_cost"), 300000);
            assert_eq!(value(&r, "section179_deduction"), expected);
        }
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn separate_section179_rejects_excess_election_and_invalid_shares() {
    unsupported(&separate179(4000000, 1175001, 300000, Section179Share::Equal), "Section 179");
    unsupported(&separate179(4000000, 1, 2650000, Section179Share::Equal), "Section 179");
    assert!(!separate179(1000, 1000, -1, Section179Share::Equal).valid_stored());
    assert!(
        !separate179(1000, 1000, 0, Section179Share::Elected { yours: 10001, spouse: 0 })
            .valid_stored()
    );
}

#[test]
fn unconfirmed_separate_section179_records_depreciate_normally() {
    let unconfirmed = |d: &mut FederalDraft| {
        let FilingChoice::Separate { section179, .. } = &mut d.setup.as_mut().unwrap().status
        else {
            panic!()
        };
        let Section179Spouse::Records { eligible, .. } = section179 else { panic!() };
        *eligible = false;
    };
    // No takes no section 179: the same asset is depreciated as if it
    // elected zero, with the notice.
    let mut d = separate179(1000, 1000, 0, Section179Share::Equal);
    unconfirmed(&mut d);
    let r = supported(&d);
    assert!(noted(&r, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"), "{r:?}");
    assert!(r.lines.iter().all(|line| line.id != "section179_deduction" || line.amount == 0));
    let expected = supported(&separate179(1000, 0, 0, Section179Share::Equal));
    assert_eq!(r.tax, expected.tax);
    assert!(!noted(&expected, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"));
    // An earlier carryforward is not deducted and waits for 2027.
    let mut d = separate179(1000, 0, 0, Section179Share::Equal);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(carried_asset(2025, 30000));
    unconfirmed(&mut d);
    let r = supported(&d);
    assert!(noted(&r, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "section179_deduction"), 0);
    assert_eq!(value(&r, "section179_carryforward"), 30000);
}

#[test]
fn separate_section179_zero_share_preserves_carry_without_spouse_income() {
    let mut d = separate179(1000, 0, 0, Section179Share::Elected { yours: 0, spouse: 10000 });
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(carried_asset(2025, 30000));
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 0);
    assert_eq!(value(&r, "section179_carryforward"), 30000);
    let original_income = crate::section179::calculate(&d).income;
    let FilingChoice::Separate { section179, .. } = &mut d.setup.as_mut().unwrap().status else {
        panic!()
    };
    *section179 = Section179Spouse::Records {
        cost: 5000000,
        allocation: Section179Share::Equal,
        eligible: true,
    };
    assert_eq!(crate::section179::calculate(&d).income, original_income);
}
