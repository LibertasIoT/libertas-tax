// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn continuing_macrs_preserves_original_quarter_and_straight_line_remaining_period() {
    let asset = continuing_equipment(20241001, 43000, true); // $50 + $380 previously.
    assert_eq!(asset.macrs_amount(false), 22800);
    assert_eq!(asset.macrs_amount(true), 22800); // 2026 convention cannot replace original.
    let mut asset = continuing_equipment(20240101, 187500, true);
    asset.cost = 300000;
    asset.class = EquipmentClass::Three;
    asset.method = EquipmentMethod::StraightLine;
    let d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 1000); // 1.125 years remain.
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 125);
    assert!(crate::interview::document_amount(&d.income[1].income, false).is_some());
}

#[test]
fn continuing_macrs_excludes_old_assets_from_new_purchase_convention_and_elections() {
    let mut asset = continuing_equipment(20251001, 5000, true);
    asset.method = EquipmentMethod::StraightLine;
    let mut d = business_with_bonus(100000, asset);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(macrs_equipment(
        100,
        EquipmentClass::Five,
        EquipmentMethod::Regular,
        20260601,
    ));
    assert!(!crate::depreciation::mid_quarter(&d));
    supported(&d); // New class election can differ from prior year's election.
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[1].placed = 20261001;
    assert!(crate::depreciation::mid_quarter(&d));
    supported(&d);
}

#[test]
fn macrs_published_schedules_cover_all_methods_classes_and_conventions() {
    // Published A-1/A-2 and A-14 values, including table-specific rounding.
    let mut asset = bonus_equipment(1000, 100);
    asset.class = EquipmentClass::Seven;
    assert_eq!(asset.table_rates(false), &[14290, 24490, 17490, 12490, 8930, 8920, 8930, 4460]);
    asset.class = EquipmentClass::Twenty;
    asset.placed = 20260101;
    assert_eq!(asset.table_rates(true)[0], 6563);
    assert_eq!(asset.table_rates(true)[20], 565);
    asset.class = EquipmentClass::Five;
    asset.method = EquipmentMethod::Declining150;
    assert_eq!(asset.table_rates(false), &[15000, 25500, 17850, 16660, 16660, 8330]);
    asset.method = EquipmentMethod::StraightLine;
    asset.class = EquipmentClass::Seven;
    assert_eq!(asset.table_rates(false), &[7140, 14290, 14290, 14280, 14290, 14280, 14290, 7140]);
}

#[test]
fn section197_requires_actual_origin_and_valid_source_records() {
    // 2026-10-23: Reg1.197-2 ties recovery to actual source facts, with no
    // arbitrary dollar or addition-count ceiling; capacity is checked separately.
    // 2026-10-23: app-contract Answers and unknowns omits an unresolved benefit.
    assert!(noted(
        &supported(&intangible_business(19930810, 19900101)),
        "WARN_INTANGIBLE_BENEFIT_LEFT_OUT",
    ));
    let mut unknown = intangible_business(20260101, 20200101);
    let FederalIncome::Business { data } = &mut unknown.income[1].income else { panic!() };
    data.intangibles[0].source.first_existed = None;
    assert!(noted(&supported(&unknown), "WARN_INTANGIBLE_BENEFIT_LEFT_OUT"));
    {
        let cost = 0;
        let mut d = intangible_business(20260101, 20200101);
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        data.intangibles[0].cost = cost;
        assert!(!d.valid_stored());
    }
}

#[test]
fn section197_added_cost_records_reject_invalid_amounts_dates_and_future_costs() {
    {
        let (amount, date) = (100, 20191231);
        let mut d = intangible_business(20200101, 19900101);
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        data.intangibles[0].additions.push(IntangibleCost { amount, date });
        assert!(!d.valid_stored());
    }
    let mut d = intangible_business(20200101, 19900101);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.intangibles[0].additions.push(IntangibleCost { amount: 100, date: 20270101 });
    assert!(!d.valid_stored());
}
