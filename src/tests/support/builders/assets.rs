use crate::tests::*;

pub(in crate::tests) fn bonus_equipment(cost: i64, percent: u32) -> BusinessAsset {
    BusinessAsset {
        farm_rental_use: None,
        id: 0,
        macrs_source: None,
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
    let prior = match acquired.max(business_started) {
        20250701 => 60_000,
        20110701 => 1_740_000,
        20110101 | 19930811 | 20100101 => 1_800_000,
        20160101 => 1_200_000,
        20200101 => 720_000,
        _ => 0,
    };
    data.intangibles.push(super::ordinary_intangible(1_800_000, acquired, business_started, prior));
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
        section179: Section179Spouse::Records { cost: spouse_cost * 100, allocation },
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

// The MACRS method is unchanged; only the independently income-limited
// section179 history differs. AMT's prior claimed amount is projected from
// the reported carry, without inventing a separate MACRS method history.
pub(in crate::tests) fn claimed_regular_asset_with_amt_carry(
    year: u32,
    cost: i64,
    amt_carry: i64,
) -> BusinessAsset {
    let mut asset = carried_asset(year, cost);
    let EquipmentHistory::Continuing { carryforward, prior_depreciation, .. } = &mut asset.history
    else {
        unreachable!()
    };
    *carryforward = 0;
    *prior_depreciation = cost * 100;
    asset.amt_section179_income_carry = Some(amt_carry * 100);
    asset
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
        kind: crate::VehicleKind::PassengerCar,
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
                // Synthetic AMT completion; the printed IRS oracle is regular tax.
                amt_section179_allowed: crate::SourceCents::Reported {
                    amount: if year == placed_year { elected } else { 0 },
                },
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

pub(in crate::tests) fn partial_bonus_auto() -> BusinessAsset {
    let mut asset = documented_auto(
        &[(2023, 20_200), (2024, 3_840), (2025, 2_304)],
        0,
        2_020_000,
        false,
        3_020_000,
    );
    asset.history_details.as_mut().unwrap().auto_history.as_mut().unwrap().first_year_bonus =
        OriginalBonusTreatment::Partial;
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
            amt_section179_allowed: crate::SourceCents::Unavailable,
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
        kind: crate::VehicleKind::PassengerCar,
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

// Derived carry control:2025 Rev.Proc.2025-16 caps, no bonus, actual source
// records. The return year changes no original schedule dates or cap table.
pub(in crate::tests) fn mixed_auto_179(
    receipts: i64,
    amt_allowed: crate::SourceCents,
) -> FederalDraft {
    let mut asset = documented_auto(&[(2025, 12_200)], 1_000_000, 0, false, 0);
    asset.id = 3;
    asset.cost = 10_000_000;
    asset.acquired = 20250615;
    asset.placed = 20250615;
    if let EquipmentHistory::Continuing { carryforward, .. } = &mut asset.history {
        *carryforward = 500_000;
    }
    let records = asset.history_details.as_mut().unwrap();
    records.original_business_basis = 10_000_000;
    records.auto_history.as_mut().unwrap().first_year_bonus = OriginalBonusTreatment::ElectedOut;
    records.carryovers = vec![Asset179Carryover { year: 2025, amount: 500_000 }];
    let row = &mut records.years[0];
    row.section179_allowed = 500_000;
    row.regular = 720_000;
    row.amt = match &amt_allowed {
        crate::SourceCents::Reported { amount } => 1_220_000 - amount,
        _ => 720_000,
    };
    row.amt_section179_allowed = amt_allowed;
    let mut draft = business_with_bonus(receipts, asset);
    draft.income.retain(|entry| matches!(entry.income, FederalIncome::Business { .. }));
    draft.next_id = 4;
    draft
}

pub(in crate::tests) fn uncapped_auto_179() -> FederalDraft {
    let mut draft = mixed_auto_179(9_400, crate::SourceCents::Reported { amount: 500_000 });
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    let asset = &mut data.assets[0];
    asset.cost = 3_000_000;
    if let EquipmentHistory::Continuing { prior_depreciation, .. } = &mut asset.history {
        *prior_depreciation = 900_000;
    }
    let records = asset.history_details.as_mut().unwrap();
    records.original_business_basis = 3_000_000;
    records.auto_history.as_mut().unwrap().first_year_capped_deduction = 900_000;
    records.years[0].regular = 400_000;
    records.years[0].amt = 400_000;
    draft
}

pub(in crate::tests) fn auto_with_retained_179_release() -> BusinessAsset {
    let mut draft = mixed_auto_179(100_000, crate::SourceCents::Reported { amount: 500_000 });
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    let mut asset = data.assets.remove(0);
    asset.acquired = 20230615;
    asset.placed = 20230615;
    if let EquipmentHistory::Continuing { carryforward, prior_depreciation, .. } =
        &mut asset.history
    {
        *carryforward = 300_000;
        *prior_depreciation = 4_340_000;
    }
    let records = asset.history_details.as_mut().unwrap();
    records.years[0].year = 2023;
    let mut second = records.years[0].clone();
    second.year = 2024;
    second.section179_allowed = 200_000;
    second.amt_section179_allowed = crate::SourceCents::Reported { amount: 200_000 };
    second.regular = 1_750_000;
    second.amt = 1_750_000;
    let mut third = second.clone();
    third.year = 2025;
    third.section179_allowed = 0;
    third.amt_section179_allowed = crate::SourceCents::NotReported;
    third.regular = 1_170_000;
    third.amt = 1_170_000;
    records.years.extend([second, third]);
    records.carryovers[0] = Asset179Carryover { year: 2023, amount: 300_000 };
    asset
}

pub(in crate::tests) fn bonus_auto_179() -> FederalDraft {
    let mut asset = documented_auto(
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
        1_000_000,
        1_300_000,
        false,
        3_700_000,
    );
    asset.id = 3;
    if let EquipmentHistory::Continuing { carryforward, .. } = &mut asset.history {
        *carryforward = 500_000;
    }
    let records = asset.history_details.as_mut().unwrap();
    records.carryovers = vec![Asset179Carryover { year: 2018, amount: 500_000 }];
    let first = &mut records.years[0];
    first.section179_allowed = 500_000;
    first.amt_section179_allowed = crate::SourceCents::Reported { amount: 300_000 };
    first.regular = 0;
    first.amt = 0;
    let mut draft = business_with_bonus(100_000, asset);
    draft.income.retain(|entry| matches!(entry.income, FederalIncome::Business { .. }));
    draft.next_id = 4;
    draft
}

pub(in crate::tests) fn community_auto_179() -> FederalDraft {
    let source = mixed_auto_179(100_000, crate::SourceCents::Reported { amount: 500_000 });
    let mut draft = complete(FederalStatus::Separate);
    draft.setup.as_mut().unwrap().status = FilingChoice::Separate {
        section179: Section179Spouse::Records { cost: 0, allocation: Section179Share::Equal },
    };
    draft.setup.as_mut().unwrap().community_property =
        crate::tests::support::builders::community_arizona();
    draft.income = source.income;
    draft.income[0].community = CommunityShare::Community;
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts += 1;
    data.assets[0].cost += 1;
    data.assets[0].history_details.as_mut().unwrap().original_business_basis += 1;
    draft.next_id = 4;
    draft
}

// Actual-expense vehicles (Form 4562 Part V, Schedule C lines 9 and 13).
pub(in crate::tests) fn actual_vehicle(
    cost: i64,
    total_miles: u32,
    business_miles: u32,
) -> BusinessAsset {
    BusinessAsset {
        cost: cost * 100,
        acquired: 20260310,
        placed: 20260315,
        history: EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::ElectOut,
            tables: false,
        },
        listed: ListedProperty::Vehicle {
            total_miles,
            business_miles,
            operating_costs: 400_000,
            loan_interest: 0,
            kind: crate::VehicleKind::PassengerCar,
        },
        ..bonus_equipment(0, 100)
    }
}

// New-year SUV source; money arguments are whole dollars.
pub(in crate::tests) fn suv_asset(elected: i64, exception: SuvException) -> BusinessAsset {
    let mut asset = actual_vehicle(70_000, 10_000, 10_000);
    asset.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 10_000,
        operating_costs: 0,
        loan_interest: 0,
        kind: VehicleKind::HeavySuv { exception },
    };
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: elected * 100 },
        bonus: BonusElection::Apply,
        tables: true,
    };
    asset
}

// Current sold equipment has no first-year recovery when placed and disposed
// in the same year (Reg.1.168(d)-1); its native short-held loss is ordinary.
pub(in crate::tests) fn same_year_property_sale(
    cost_cents: i64,
    proceeds_cents: i64,
) -> BusinessAsset {
    let mut asset = depreciable_asset(1, EquipmentClass::Five, 20260101, 0);
    asset.cost = cost_cents;
    asset.disposal = crate::AssetDisposal::Disposed {
        date: 20260715,
        how: crate::DisposalKind::Sold {
            proceeds: proceeds_cents,
            selling_expenses: 0,
            payment: crate::SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
            livestock: crate::Livestock::No.into(),
        },
    };
    asset
}

pub(in crate::tests) fn current_property_business_return(
    passive: bool,
    operating: i64,
    risk_cents: Option<i64>,
    assets: Vec<BusinessAsset>,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut source = if passive { passive_business(operating) } else { cash_business(operating) };
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    let asset_count = assets.len();
    data.assets = assets;
    if let Some(capacity) = risk_cents {
        data.at_risk = simplified_rental_risk(capacity, 0, 0, [0; 4]).unwrap();
    }
    add(&mut draft, income(source));
    let source = draft.income.last().unwrap().id;
    let mut page = crate::tests::cross::interview::business(0);
    page.assets = (0..asset_count)
        .map(|_| crate::tests::cross::interview::asset(100_000, 20260101, AssetClass::Five))
        .collect();
    // Native Save supplies only canonical IDs; the shared helper restores all
    // authored source amounts, histories and participation facts unchanged.
    let mut draft = save_native_asset_identities(
        draft,
        source,
        IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
        TaxSection::SelfEmployment,
    );
    // This fixture explicitly supplies no prior1231 losses after its identity
    // Save; placeholder property must not prune that authored source answer.
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    draft
}
