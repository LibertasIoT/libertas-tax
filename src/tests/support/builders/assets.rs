use crate::tests::*;

pub(in crate::tests) fn bonus_equipment(cost: i64, percent: u32) -> BusinessAsset {
    BusinessAsset {
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        description: alloc::string::String::new(),
        cost: cost * 100,
        credit_reduction: 0,
        acquired: 20250120,
        placed: 20260601,
        class: EquipmentClass::Five,
        history: EquipmentHistory::New {
            section179: if percent > 50 {
                Section179::Eligible { elected: 0 }
            } else {
                Section179::Ineligible
            },
            bonus: BonusElection::Apply,
            tables: false,
        },
        method: EquipmentMethod::Regular,
        listed: ListedProperty::No { business_percent: percent },
        disposal: crate::AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    }
}
pub(in crate::tests) fn business_with_bonus(receipts: i64, asset: BusinessAsset) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(receipts);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.assets.push(asset);
    add(&mut d, income(business));
    d
}

pub(in crate::tests) fn de_minimis_business(
    largest_cost: i64,
    financial_statement: bool,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(100000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.expenses.push(FederalBusinessExpense {
        category: BusinessExpenseCategory::Other {
            label: "Documented small equipment purchases".into(),
        },
        amount: 2000000,
        treatment: BusinessExpenseKind::DeMinimis {
            largest_cost,
            financial_statement,
            eligible: true,
        },
    });
    add(&mut d, income(business));
    d
}

pub(in crate::tests) fn macrs_equipment(
    cost: i64,
    class: EquipmentClass,
    method: EquipmentMethod,
    placed: u32,
) -> BusinessAsset {
    let mut asset = bonus_equipment(cost, 100);
    asset.class = class;
    asset.method = method;
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    asset.placed = placed;
    asset
}

pub(in crate::tests) fn continuing_equipment(
    placed: u32,
    prior_depreciation: i64,
    mid_quarter: bool,
) -> BusinessAsset {
    let mut asset = bonus_equipment(1000, 100);
    asset.acquired = 20170928;
    asset.placed = placed;
    asset.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        prior_depreciation,
        mid_quarter,
        initial_section179: 0,
        carryforward: 0,
        schedule: EquipmentSchedule::Manual,
    };
    asset
}

pub(in crate::tests) fn table_history(
    prior: i64,
    bonus: i64,
    mid_quarter: bool,
) -> EquipmentHistory {
    EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        prior_depreciation: prior,
        mid_quarter,
        initial_section179: 0,
        carryforward: 0,
        schedule: EquipmentSchedule::Tables { initial_bonus: bonus },
    }
}

pub(in crate::tests) fn startup_business(
    costs: i64,
    started: u32,
    capitalize: bool,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut entry = cash_business(100000);
    let FederalIncome::Business { data } = &mut entry else { panic!() };
    data.startup = BusinessStartup::Costs { costs, started, capitalize, eligible: true };
    add(&mut d, income(entry));
    d
}

pub(in crate::tests) fn intangible_business(acquired: u32, business_started: u32) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(100000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.intangibles.push(BusinessIntangible {
        label: "Purchased business goodwill".into(),
        additions: vec![],
        cost: 1800000,
        acquired,
        business_started,
        eligible: true,
    });
    add(&mut d, income(business));
    d
}

pub(in crate::tests) fn elected_asset(
    cost: i64,
    elected: i64,
    bonus: BonusElection,
) -> BusinessAsset {
    let mut asset = bonus_equipment(cost, 100);
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: elected * 100 },
        bonus,
        tables: false,
    };
    asset
}

pub(in crate::tests) fn separate179(
    cost: i64,
    elected: i64,
    spouse_cost: i64,
    allocation: Section179Share,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().status = FilingChoice::Separate {
        section179: Section179Spouse::Records {
            cost: spouse_cost * 100,
            allocation,
            eligible: true,
        },
    };
    let mut e = income(cash_business(10000000));
    let FederalIncome::Business { data } = &mut e.income else { panic!() };
    data.assets.push(elected_asset(cost, elected, BonusElection::ElectOut));
    add(&mut d, e);
    d
}

pub(in crate::tests) fn carried_asset(year: u32, amount: i64) -> BusinessAsset {
    let mut a = bonus_equipment(amount, 100);
    a.acquired = 20170928;
    a.placed = year * 10000 + 601;
    a.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        initial_section179: amount * 100,
        carryforward: amount * 100,
        prior_depreciation: 0,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    a
}

pub(in crate::tests) fn depreciable_asset(
    cost: i64,
    class: EquipmentClass,
    placed: u32,
    elected: i64,
) -> BusinessAsset {
    BusinessAsset {
        cost: cost * 100,
        acquired: placed,
        placed,
        class,
        history: EquipmentHistory::New {
            section179: if elected > 0 {
                Section179::Eligible { elected: elected * 100 }
            } else {
                Section179::Ineligible
            },
            bonus: BonusElection::ElectOut,
            tables: true,
        },
        ..bonus_equipment(0, 100)
    }
}

pub(in crate::tests) fn continuing_asset(
    cost: i64,
    placed: u32,
    class: EquipmentClass,
    prior_depreciation: i64,
    disposal: crate::AssetDisposal,
) -> BusinessAsset {
    BusinessAsset {
        cost,
        acquired: placed,
        placed,
        class,
        history: EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        disposal,
        ..bonus_equipment(0, 100)
    }
}

pub(in crate::tests) fn documented_auto(
    rows: &[(u32, i64)],
    elected: i64,
    bonus: i64,
    safe: bool,
    post_basis: i64,
) -> BusinessAsset {
    let placed_year = rows[0].0;
    let mut asset = bonus_equipment(60_000, 100);
    asset.acquired = placed_year * 10000 + 101;
    asset.placed = asset.acquired;
    asset.class = EquipmentClass::Five;
    asset.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 10_000,
        operating_costs: 0,
        loan_interest: 0,
        passenger_automobile: true,
    };
    let prior = rows.iter().map(|(_, amount)| amount * 100).sum();
    asset.history = EquipmentHistory::Continuing {
        initial_section179: elected,
        carryforward: 0,
        prior_depreciation: prior,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: bonus },
    };
    asset.history_details = Some(HistoricalAssetRecords {
        original_business_basis: 6_000_000,
        original_income_use: UseFraction { used: 1, total: 1 },
        original_qualified_use: UseFraction { used: 1, total: 1 },
        original_section179_elected: elected,
        original_bonus: bonus,
        years: rows
            .iter()
            .map(|&(year, amount)| AssetYearHistory {
                year,
                income_use: UseFraction { used: 1, total: 1 },
                qualified_use: UseFraction { used: 1, total: 1 },
                section179_allowed: if year == placed_year { elected } else { 0 },
                bonus: if year == placed_year { bonus } else { 0 },
                regular: amount * 100 - if year == placed_year { elected + bonus } else { 0 },
                amt: amount * 100 - if year == placed_year { elected + bonus } else { 0 },
                basis_increase: 0,
                basis_decrease: 0,
                recaptured: 0,
            })
            .collect(),
        carryovers: vec![],
        convention: AssetConvention::HalfYear,
        original_method: if safe {
            OriginalAssetMethod::SafeHarbor2019
        } else {
            OriginalAssetMethod::Declining200
        },
        auto_history: Some(AutoRecoveryHistory {
            first_year_bonus: OriginalBonusTreatment::HundredPercent,
            safe_harbor_adopted_year: safe.then_some(placed_year + 1),
            car_or_truck: HistoricalAutoKind::Car,
            first_year_capped_deduction: rows[0].1 * 100,
            post_recovery_basis: post_basis,
        }),
    });
    asset
}

pub(in crate::tests) fn direct_amt_carry_history() -> HistoricalAssetRecords {
    let fraction = UseFraction { used: 100, total: 100 };
    HistoricalAssetRecords {
        original_business_basis: 2_000_000,
        original_income_use: fraction,
        original_qualified_use: fraction,
        original_section179_elected: 1_000_000,
        original_bonus: 0,
        years: vec![AssetYearHistory {
            year: 2025,
            income_use: fraction,
            qualified_use: fraction,
            section179_allowed: 900_000,
            bonus: 0,
            regular: 200_000,
            amt: 150_000,
            basis_increase: 0,
            basis_decrease: 0,
            recaptured: 0,
        }],
        carryovers: vec![Asset179Carryover { year: 2025, amount: 100_000 }],
        convention: AssetConvention::HalfYear,
        original_method: OriginalAssetMethod::Declining200,
        auto_history: None,
    }
}

// Approved original-auto schedules and actual acquisition/method source facts.
pub(in crate::tests) fn original_auto_editor_sources() -> Vec<BusinessAsset> {
    let mut before_bonus = bonus_equipment(14_000, 100);
    before_bonus.acquired = 20170927;
    before_bonus.placed = 20260601;
    before_bonus.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::Apply,
        tables: true,
    };
    before_bonus.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 10_000,
        operating_costs: 0,
        loan_interest: 0,
        passenger_automobile: true,
    };
    let mut before_restored = documented_auto(&[(2025, 20_200)], 0, 2_020_000, false, 3_980_000);
    before_restored.acquired = 20250119;
    before_restored.placed = 20250201;
    before_restored
        .history_details
        .as_mut()
        .unwrap()
        .auto_history
        .as_mut()
        .unwrap()
        .first_year_bonus = OriginalBonusTreatment::Partial;
    let mut declining = documented_auto(
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
    declining.method = EquipmentMethod::Declining150;
    declining.history_details.as_mut().unwrap().original_method = OriginalAssetMethod::Declining150;
    let mut straight = declining.clone();
    straight.method = EquipmentMethod::StraightLine;
    straight.history_details.as_mut().unwrap().original_method = OriginalAssetMethod::StraightLine;
    vec![before_bonus, before_restored, declining, straight]
}

// Pub.946 original2006 car/truck/electric annual caps. This $180,000
// unadjusted schedule remains capped throughout its original recovery period.
pub(in crate::tests) fn ordinary_original_auto(kind: HistoricalAutoKind) -> BusinessAsset {
    let (first, second, third, later, post_basis) = match kind {
        HistoricalAutoKind::Car => (2_960, 4_800, 2_850, 1_775, 16_406_500),
        HistoricalAutoKind::TruckVan => (3_260, 5_200, 3_150, 1_875, 16_276_500),
        HistoricalAutoKind::ElectricCar => (8_980, 14_400, 8_650, 5_225, 13_229_500),
    };
    let asset = historical_original_auto(2006, [first, second, third, later], kind);
    assert_eq!(
        asset.history_details.as_ref().unwrap().auto_history.as_ref().unwrap().post_recovery_basis,
        post_basis
    );
    asset
}

// Actual retained annual deductions for a high-basis auto whose published caps bind
// through the six original MACRS years and every later unrecovered-basis year.
pub(in crate::tests) fn historical_original_auto(
    placed_year: u32,
    [first, second, third, later]: [i64; 4],
    kind: HistoricalAutoKind,
) -> BusinessAsset {
    let rows: Vec<_> = (placed_year..2026)
        .map(|year| {
            (
                year,
                match year - placed_year {
                    0 => first,
                    1 => second,
                    2 => third,
                    _ => later,
                },
            )
        })
        .collect();
    let post_basis = 18_000_000 - (first + second + third + 3 * later) * 100;
    let mut asset = documented_auto(&rows, 0, 0, false, post_basis);
    asset.cost = 18_000_000;
    asset.placed = placed_year * 10000 + 701;
    let records = asset.history_details.as_mut().unwrap();
    records.original_business_basis = asset.cost;
    let auto = records.auto_history.as_mut().unwrap();
    auto.first_year_bonus = OriginalBonusTreatment::ElectedOut;
    auto.car_or_truck = kind;
    asset
}
