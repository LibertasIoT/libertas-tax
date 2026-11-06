// schedule_c items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn equipment_class(class: AssetClass) -> EquipmentClass {
    match class {
        AssetClass::Three => EquipmentClass::Three,
        AssetClass::Five => EquipmentClass::Five,
        AssetClass::Seven => EquipmentClass::Seven,
        AssetClass::Ten => EquipmentClass::Ten,
        AssetClass::Fifteen => EquipmentClass::Fifteen,
        AssetClass::Twenty => EquipmentClass::Twenty,
        AssetClass::ThirtyNine => EquipmentClass::ThirtyNine,
    }
}
pub(super) fn asset_class(class: EquipmentClass) -> AssetClass {
    match class {
        EquipmentClass::Three => AssetClass::Three,
        EquipmentClass::Five => AssetClass::Five,
        EquipmentClass::Seven => AssetClass::Seven,
        EquipmentClass::Ten => AssetClass::Ten,
        EquipmentClass::Fifteen => AssetClass::Fifteen,
        EquipmentClass::Twenty => AssetClass::Twenty,
        EquipmentClass::ThirtyNine => AssetClass::ThirtyNine,
        EquipmentClass::NonMacrs { .. } => unreachable!("actual route handled by asset_page"),
    }
}
// The first-year choice onto the section 179 and bonus elections: Section179
// elects its amount; otherwise property that can be section 179 property
// (over 50% business use, not a building) is Eligible with nothing elected, so
// it still counts toward the annual phase-out, and the rest is Ineligible. Only
// Regular elects out of bonus depreciation, for the whole class.
pub(super) fn equipment_history(
    history: &AssetHistory,
    section179_property: bool,
) -> EquipmentHistory {
    match history {
        AssetHistory::New { first_year } => EquipmentHistory::New {
            section179: match first_year {
                FirstYearDeduction::Section179 { elected, .. } => {
                    Section179::Eligible { elected: *elected }
                }
                _ if section179_property => Section179::Eligible { elected: 0 },
                _ => Section179::Ineligible,
            },
            bonus: if *first_year == FirstYearDeduction::Regular {
                BonusElection::ElectOut
            } else {
                BonusElection::Apply
            },
            tables: true,
        },
        AssetHistory::Continuing {
            prior_depreciation,
            first_year_179,
            first_year_bonus,
            carryforward,
            mid_quarter,
        } => EquipmentHistory::Continuing {
            initial_section179: *first_year_179,
            carryforward: *carryforward,
            prior_depreciation: *prior_depreciation,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: *mid_quarter,
            schedule: EquipmentSchedule::Tables { initial_bonus: *first_year_bonus },
        },
    }
}
// The reverse of equipment_history. A Section179 choice is told from a
// Bonus one by its typed description, which every other asset leaves empty.
pub(super) fn asset_history(record: &BusinessAsset) -> AssetHistory {
    match record.history {
        EquipmentHistory::New { section179, bonus, .. } => AssetHistory::New {
            first_year: match (section179, bonus) {
                (_, BonusElection::ElectOut) => FirstYearDeduction::Regular,
                (Section179::Eligible { elected }, _)
                    if elected != 0 || !record.description.is_empty() =>
                {
                    FirstYearDeduction::Section179 {
                        description: record.description.clone(),
                        elected,
                    }
                }
                _ => FirstYearDeduction::Bonus,
            },
        },
        EquipmentHistory::Continuing {
            initial_section179,
            carryforward,
            prior_depreciation,
            mid_quarter,
            schedule,
            ..
        } => AssetHistory::Continuing {
            prior_depreciation,
            first_year_179: initial_section179,
            first_year_bonus: match schedule {
                EquipmentSchedule::Tables { initial_bonus } => initial_bonus,
                EquipmentSchedule::Manual => 0,
            },
            carryforward,
            mid_quarter,
        },
    }
}
// The description typed with a section 179 election; no other asset has one.
pub(super) fn asset_description(asset_history: &AssetHistory) -> alloc::string::String {
    match asset_history {
        AssetHistory::New { first_year: FirstYearDeduction::Section179 { description, .. } } => {
            description.clone()
        }
        _ => alloc::string::String::new(),
    }
}
// One page asset onto BusinessAsset: acquired defaults to the service date,
// the ordinary GDS method from the IRS tables, confirmed by the page.
// `business` is false for equipment rented out outside a business, which is
// never section 179 property.
pub(super) fn asset_record(asset: &Asset, business: bool) -> BusinessAsset {
    if let crate::AssetRecoveryPage::NonMacrs { facts } = &asset.recovery {
        return facts.record(asset);
    }
    let crate::AssetRecoveryPage::Gds { facts } = &asset.recovery else {
        unreachable!("actual route handled above")
    };
    let class = equipment_class(facts.class);
    BusinessAsset {
        id: asset.id,
        macrs_source: facts.source.clone().map(Into::into).or_else(|| asset.macrs_source.clone()),
        farm_rental_use: asset.farm_rental_use.clone(),
        history_details: asset.history_details.clone(),
        amt_section179_income_carry: crate::AssetAmt179CarryAnswer::amount(
            &asset.amt_section179_income_carry,
        ),
        description: asset_description(&facts.history),
        cost: asset.cost,
        credit_reduction: asset.credit_reduction,
        acquired: asset.acquired.unwrap_or(asset.placed),
        placed: asset.placed,
        class,
        history: equipment_history(
            &facts.history,
            business
                && asset.business_percent > 50
                && (class != EquipmentClass::ThirtyNine
                    || facts.source.as_ref().and_then(|s| s.improvement.as_ref()).is_some_and(
                        |row| {
                            matches!(
                                row.kind,
                                crate::RealImprovementKind::Interior1250 { made_by_taxpayer: true }
                                    | crate::RealImprovementKind::Roof
                                    | crate::RealImprovementKind::Hvac
                                    | crate::RealImprovementKind::FireProtectionOrAlarm
                                    | crate::RealImprovementKind::Security
                            )
                        },
                    )),
        ),
        method: EquipmentMethod::Regular,
        listed: ListedProperty::No { business_percent: asset.business_percent },
        disposal: asset.disposal.clone(),
        investment_credit: asset.investment_credit.clone(),
        earlier_business_percent: asset.earlier_business_percent,
        partial: asset.partial.clone(),
    }
}
pub(super) fn asset_item(record: &BusinessAsset) -> Asset {
    if let Some(system) = record.nonmacrs_route() {
        return crate::nonmacrs::asset_page(record, *system);
    }
    let history = asset_history(record);
    Asset {
        macrs_source: None,
        id: record.id,
        farm_rental_use: record.farm_rental_use.clone(),
        allowed_farm_rental_use: Vec::new(),
        history_details: record.history_details.clone(),
        amt_section179_income_carry: crate::AssetAmt179CarryAnswer::from_amount(
            record.amt_section179_income_carry,
        ),
        allowed_amt_carry: crate::AssetAmt179CarryAnswer::page_choices(&history),
        cost: record.cost,
        credit_reduction: record.credit_reduction,
        placed: record.placed,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: record.macrs_source.clone().map(|source| {
                    crate::MacrsRecoverySourcePage::from_source(
                        source,
                        matches!(record.history, EquipmentHistory::Continuing { .. }),
                        record.class == EquipmentClass::ThirtyNine,
                    )
                }),
                class: asset_class(record.class),
                history,
            },
        },
        business_percent: match record.listed {
            ListedProperty::No { business_percent } => business_percent,
            ListedProperty::Vehicle { .. } => {
                unreachable!("vehicle assets use their mileage editor")
            }
        },
        acquired: (record.acquired != record.placed).then_some(record.acquired),
        disposal: record.disposal.clone(),
        investment_credit: record.investment_credit.clone(),
        earlier_business_percent: record.earlier_business_percent,
        partial: record.partial.clone(),
    }
}
// Owner-dependent choices are supplied when the editor opens and rebuilt on Save.
// The calculation uses only row answers and this immutable cohort evidence.
pub(super) fn refresh_macrs_questions(
    source: &mut crate::MacrsRecoverySourcePage,
    class: EquipmentClass,
    continuing: bool,
    context: Option<&crate::MacrsSourceQuestionContext>,
) -> bool {
    let native: crate::MacrsRecoverySource = source.clone().into();
    let invocation = context.map_or_else(crate::macrs_source::AdsInvocation::default, |context| {
        context.invocation(class)
    });
    let (original, life) =
        native.page_choices_in_context(continuing, class == EquipmentClass::ThirtyNine, invocation);
    // A correction asks only for a newly applicable source record. Withdrawing
    // choices or retaining a supplied answer does not require another Save.
    let newly_asked = (source.original_system.is_none()
        && original.iter().any(|choice| !source.allowed_original_system.contains(choice)))
        || (source.class_life.is_none()
            && life.iter().any(|choice| !source.allowed_class_life.contains(choice)));
    source.allowed_original_system = original;
    source.allowed_class_life = life;
    newly_asked
}
pub(super) fn macrs_question_context(
    view: &crate::macrs_source::MacrsElectionView,
    owner: Owner,
    activity: Option<i64>,
    placed: u32,
    farming: bool,
    source: Option<&crate::MacrsRecoverySourcePage>,
) -> Option<alloc::boxed::Box<crate::MacrsSourceQuestionContext>> {
    let mut context = view.question_context(owner, activity, placed);
    context.farming = farming;
    context.retained_original_system =
        source.is_some_and(|source| source.original_system.is_some());
    context.retained_class_life = source.is_some_and(|source| source.class_life.is_some());
    (!context.elections.is_empty()
        || context.interest_limit_election.is_some()
        || !context.capitalization_years.is_empty()
        || context.retained_original_system
        || context.retained_class_life)
        .then(|| alloc::boxed::Box::new(context))
}
pub(super) fn refresh_asset_macrs_questions(
    asset: &mut Asset,
    owner: Owner,
    activity: Option<i64>,
    farming: bool,
    view: &crate::macrs_source::MacrsElectionView,
) -> bool {
    let crate::AssetRecoveryPage::Gds { facts } = &mut asset.recovery else {
        return false;
    };
    facts.macrs_question_context =
        macrs_question_context(view, owner, activity, asset.placed, farming, facts.source.as_ref());
    let Some(source) = &mut facts.source else {
        return false;
    };
    refresh_macrs_questions(
        source,
        equipment_class(facts.class),
        matches!(facts.history, AssetHistory::Continuing { .. }),
        facts.macrs_question_context.as_deref(),
    )
}
pub(super) fn refresh_record_macrs_questions(
    document: &mut crate::IncomeRecord,
    owner: Owner,
    activity: i64,
    view: &crate::macrs_source::MacrsElectionView,
) -> Vec<usize> {
    let (assets, farming) = match document {
        crate::IncomeRecord::Business(row) => (Some(&mut row.assets), false),
        crate::IncomeRecord::Farm(row) => (Some(&mut row.assets), true),
        crate::IncomeRecord::FarmRental(row) => (Some(&mut row.assets), false),
        crate::IncomeRecord::EquipmentRental(row) => (Some(&mut row.assets), false),
        _ => (None, false),
    };
    if let Some(assets) = assets {
        return assets
            .iter_mut()
            .enumerate()
            .filter_map(|(index, asset)| {
                refresh_asset_macrs_questions(asset, owner, Some(activity), farming, view)
                    .then_some(index)
            })
            .collect();
    }
    if let crate::IncomeRecord::Rental(row) = document {
        return row
            .assets
            .iter_mut()
            .enumerate()
            .filter_map(|(index, asset)| {
                let crate::CanonicalAssetPage::Gds { facts } = asset else {
                    return None;
                };
                let facts = &mut facts.financial;
                facts.macrs_question_context = macrs_question_context(
                    view,
                    owner,
                    Some(activity),
                    facts.placed,
                    false,
                    facts.source.as_deref(),
                );
                let source = facts.source.as_deref_mut()?;
                refresh_macrs_questions(
                    source,
                    facts.class.into(),
                    matches!(facts.history, EquipmentHistory::Continuing { .. }),
                    facts.macrs_question_context.as_deref(),
                )
                .then_some(index)
            })
            .collect();
    }
    Vec::new()
}
pub(super) fn refresh_building_recovery_questions(
    document: &mut crate::IncomeRecord,
    view: &crate::macrs_source::MacrsElectionView,
) -> Vec<usize> {
    let crate::IncomeRecord::Rental(page) = document else { return Vec::new() };
    page.more
        .energy_buildings
        .iter_mut()
        .enumerate()
        .filter_map(|(index, row)| {
            let offered = row.recovery.is_some() || view.building_recovery_needed(row.id);
            let allowed = if offered {
                alloc::vec![
                    libertas_macros::variant_index!(crate::BuildingRecoveryRecord::Record) as i32
                ]
            } else {
                Vec::new()
            };
            let changed = row.recovery.is_none()
                && allowed.iter().any(|choice| !row.allowed_recovery.contains(choice));
            row.allowed_recovery = allowed;
            changed.then_some(index)
        })
        .collect()
}
pub(super) fn macrs_question_warning(
    employee: bool,
    row: usize,
    asset: usize,
) -> crate::SaveWarning {
    crate::SaveWarning {
        message: crate::Message::new(
            if employee {
                "EMPLOYEE_MACRS_SOURCE_QUESTIONS_UPDATED"
            } else {
                "ASSET_MACRS_SOURCE_QUESTIONS_UPDATED"
            },
            alloc::vec![
                crate::MessageArg::Number(row as i64 + 1),
                crate::MessageArg::Number(asset as i64 + 1)
            ],
        )
        .encode(),
    }
}
// The page's vehicles in stored order: the standard-mileage vehicles first,
// then the actual-expense ones, each numbered "Vehicle n" in that order.
pub(super) fn vehicles_in_stored_order(
    vehicles: &[Vehicle],
) -> impl Iterator<Item = &VehicleMethod> {
    let mileage =
        vehicles.iter().filter(|v| matches!(v.method, VehicleMethod::StandardMileage { .. }));
    let actual = vehicles.iter().filter(|v| matches!(v.method, VehicleMethod::Actual { .. }));
    mileage.chain(actual).map(|v| &v.method)
}
// A standard-mileage vehicle onto BusinessMileage.
pub(super) fn mileage_record(
    id: i64,
    first_half: u32,
    second_half: u32,
    total: u32,
    more: &MileageMore,
) -> BusinessMileage {
    BusinessMileage {
        id,
        first_half,
        second_half,
        total,
        commuting: more.commuting,
        parking_tolls: more.parking_tolls,
        interest: more.interest,
        property_tax: more.property_tax,
        owned: !more.leased,
        placed: more.placed,
        personal_use_available: more.personal_use_available,
        other_vehicle: more.other_vehicle,
        evidence: more.evidence,
    }
}
pub(super) fn mileage_item(record: &BusinessMileage) -> Vehicle {
    Vehicle {
        method: VehicleMethod::StandardMileage {
            id: record.id,
            first_half: record.first_half,
            second_half: record.second_half,
            total: record.total,
            more: MileageMore {
                commuting: record.commuting,
                parking_tolls: record.parking_tolls,
                interest: record.interest,
                property_tax: record.property_tax,
                leased: !record.owned,
                placed: record.placed,
                personal_use_available: record.personal_use_available,
                other_vehicle: record.other_vehicle,
                evidence: record.evidence,
            },
        },
    }
}
// An actual-expense vehicle onto a 5-year BusinessAsset with its listed
// property facts; its business use remains the exact business miles over all miles.
#[allow(clippy::too_many_arguments)]
pub(super) fn vehicle_asset(
    id: i64,
    macrs_source: &Option<alloc::boxed::Box<crate::MacrsRecoverySource>>,
    history_details: &Option<alloc::boxed::Box<crate::HistoricalAssetRecords>>,
    amt_section179_income_carry: &Option<alloc::boxed::Box<crate::AssetAmt179CarryAnswer>>,
    cost: i64,
    credit_reduction: i64,
    acquired: Option<u32>,
    placed: u32,
    total_miles: u32,
    business_miles: u32,
    operating_costs: i64,
    loan_interest: i64,
    kind: crate::VehicleKind,
    earlier_business_percent: Option<u32>,
    history: &AssetHistory,
    disposal: &crate::AssetDisposal,
) -> BusinessAsset {
    BusinessAsset {
        farm_rental_use: None,
        macrs_source: macrs_source.as_deref().cloned(),
        id,
        history_details: history_details.as_deref().cloned(),
        amt_section179_income_carry: amt_section179_income_carry
            .as_deref()
            .map(crate::AssetAmt179CarryAnswer::value),
        description: asset_description(history),
        cost,
        credit_reduction,
        acquired: acquired.unwrap_or(placed),
        placed,
        class: EquipmentClass::Five,
        history: equipment_history(history, u64::from(business_miles) * 2 > u64::from(total_miles)),
        method: match history_details.as_ref().map(|records| &records.original_method) {
            Some(crate::OriginalAssetMethod::Declining150) => EquipmentMethod::Declining150,
            Some(crate::OriginalAssetMethod::StraightLine) => EquipmentMethod::StraightLine,
            _ => EquipmentMethod::Regular,
        },
        listed: ListedProperty::Vehicle {
            total_miles,
            business_miles,
            operating_costs,
            loan_interest,
            kind,
        },
        disposal: disposal.clone(),
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent,
        partial: None,
    }
}
pub(super) fn vehicle_item(record: &BusinessAsset) -> Option<Vehicle> {
    let ListedProperty::Vehicle {
        total_miles,
        business_miles,
        operating_costs,
        loan_interest,
        kind,
    } = record.listed
    else {
        return None;
    };
    let earlier_business_percent = record.earlier_business_percent;
    Some(Vehicle {
        method: VehicleMethod::Actual {
            macrs_source: record.macrs_source.clone().map(alloc::boxed::Box::new),
            id: record.id,
            history_details: record.history_details.clone().map(alloc::boxed::Box::new),
            amt_section179_income_carry: crate::AssetAmt179CarryAnswer::from_amount(
                record.amt_section179_income_carry,
            )
            .map(alloc::boxed::Box::new),
            allowed_amt_carry: crate::AssetAmt179CarryAnswer::page_choices(&asset_history(record)),
            cost: record.cost,
            credit_reduction: record.credit_reduction,
            acquired: (record.acquired != record.placed).then_some(record.acquired),
            placed: record.placed,
            total_miles,
            business_miles,
            operating_costs,
            loan_interest,
            kind,
            earlier_business_percent,
            history: asset_history(record),
            disposal: record.disposal.clone(),
        },
    })
}
// Part II lines in printed order, each with its stored category.
pub(super) fn expense_lines(e: &BusinessExpenses) -> [(BusinessExpenseCategory, i64); 18] {
    use BusinessExpenseCategory as C;
    [
        (C::Advertising, e.advertising),
        (C::Commissions, e.commissions),
        (C::ContractLabor, e.contract_labor),
        (C::EmployeeBenefits, e.employee_benefits),
        (C::Insurance, e.insurance),
        (C::MortgageInterest, e.mortgage_interest),
        (C::OtherInterest, e.other_interest),
        (C::LegalAndProfessional, e.legal_and_professional),
        (C::Office, e.office),
        (C::EmployeePension, e.pension_plans),
        (C::VehicleAndEquipmentRent, e.vehicle_and_equipment_rent),
        (C::OtherRent, e.other_rent),
        (C::Repairs, e.repairs),
        (C::Supplies, e.supplies),
        (C::TaxesAndLicenses, e.taxes_and_licenses),
        (C::Travel, e.travel),
        (C::Utilities, e.utilities),
        (C::Wages, e.wages),
    ]
}
// Each nonzero line is one FederalBusinessExpense; meals take the 50% or the
// DOT 80% treatment, every other line is ordinary.
pub(super) fn expense_records(e: &BusinessExpenses) -> Vec<FederalBusinessExpense> {
    let mut records: Vec<_> = expense_lines(e)
        .into_iter()
        .filter(|(_, amount)| *amount != 0)
        .map(|(category, amount)| FederalBusinessExpense {
            treatment: BusinessExpenseKind::Ordinary,
            category,
            amount,
        })
        .chain(e.meals.iter().map(|meal| FederalBusinessExpense {
            category: BusinessExpenseCategory::Meals,
            amount: meal.amount,
            treatment: match meal.kind {
                crate::MealKind::Business => BusinessExpenseKind::Meals,
                crate::MealKind::Transportation => BusinessExpenseKind::TransportationMeals,
                crate::MealKind::FullyDeductible => BusinessExpenseKind::Ordinary,
            },
        }))
        .chain(e.other.iter().map(|other| FederalBusinessExpense {
            category: BusinessExpenseCategory::Other { label: other.description.clone() },
            amount: other.amount,
            treatment: BusinessExpenseKind::Ordinary,
        }))
        .collect();
    records.sort_by_key(|record| record.category.index());
    records
}
pub(super) fn expense_items(records: &[FederalBusinessExpense]) -> BusinessExpenses {
    let mut e = BusinessExpenses {
        advertising: 0,
        commissions: 0,
        contract_labor: 0,
        employee_benefits: 0,
        insurance: 0,
        mortgage_interest: 0,
        other_interest: 0,
        legal_and_professional: 0,
        office: 0,
        vehicle_and_equipment_rent: 0,
        other_rent: 0,
        repairs: 0,
        supplies: 0,
        taxes_and_licenses: 0,
        travel: 0,
        meals: Vec::new(),
        pension_plans: 0,
        utilities: 0,
        wages: 0,
        other: Vec::new(),
    };
    for record in records {
        use BusinessExpenseCategory as C;
        let line = match &record.category {
            C::Advertising => &mut e.advertising,
            C::Commissions => &mut e.commissions,
            C::ContractLabor => &mut e.contract_labor,
            C::EmployeeBenefits => &mut e.employee_benefits,
            C::Insurance => &mut e.insurance,
            C::MortgageInterest => &mut e.mortgage_interest,
            C::OtherInterest => &mut e.other_interest,
            C::LegalAndProfessional => &mut e.legal_and_professional,
            C::Office => &mut e.office,
            C::VehicleAndEquipmentRent => &mut e.vehicle_and_equipment_rent,
            C::OtherRent => &mut e.other_rent,
            C::Repairs => &mut e.repairs,
            C::Supplies => &mut e.supplies,
            C::TaxesAndLicenses => &mut e.taxes_and_licenses,
            C::Travel => &mut e.travel,
            C::EmployeePension => &mut e.pension_plans,
            C::Meals => {
                let kind = match record.treatment {
                    BusinessExpenseKind::Meals => crate::MealKind::Business,
                    BusinessExpenseKind::TransportationMeals => crate::MealKind::Transportation,
                    _ => crate::MealKind::FullyDeductible,
                };
                if let Some(meal) = e.meals.iter_mut().find(|m| m.kind == kind) {
                    meal.amount += record.amount;
                } else {
                    e.meals.push(crate::MealExpense { kind, amount: record.amount });
                }
                continue;
            }
            C::Utilities => &mut e.utilities,
            C::Wages => &mut e.wages,
            C::Other { label } => {
                e.other.push(OtherExpense { description: label.clone(), amount: record.amount });
                continue;
            }
            // A farm-only line has no Schedule C field; the reopen check then
            // keeps the page closed (and the return refuses it by name).
            _ => continue,
        };
        *line = record.amount;
    }
    e
}
// The page's vehicles and property onto the stored mileage and asset
// records: mileage vehicles, then the property, then the actual-expense
// vehicles (shared by the Business and Farm kinds).
pub(crate) fn vehicle_and_asset_records(
    vehicles_page: &[Vehicle],
    assets_page: &[Asset],
) -> (Vec<BusinessMileage>, Vec<BusinessAsset>) {
    let mut vehicles = Vec::new();
    let mut vehicle_assets = Vec::new();
    for method in vehicles_in_stored_order(vehicles_page) {
        match method {
            VehicleMethod::StandardMileage { id, first_half, second_half, total, more } => {
                vehicles.push(mileage_record(*id, *first_half, *second_half, *total, more))
            }
            VehicleMethod::Leased { .. } => {}
            VehicleMethod::Actual {
                macrs_source,
                id,
                history_details,
                amt_section179_income_carry,
                allowed_amt_carry: _,
                cost,
                credit_reduction,
                acquired,
                placed,
                total_miles,
                business_miles,
                operating_costs,
                loan_interest,
                kind,
                earlier_business_percent,
                history,
                disposal,
            } => vehicle_assets.push(vehicle_asset(
                *id,
                macrs_source,
                history_details,
                amt_section179_income_carry,
                *cost,
                *credit_reduction,
                *acquired,
                *placed,
                *total_miles,
                *business_miles,
                *operating_costs,
                *loan_interest,
                *kind,
                *earlier_business_percent,
                history,
                disposal,
            )),
        }
    }
    let mut assets: Vec<BusinessAsset> =
        assets_page.iter().map(|asset| asset_record(asset, true)).collect();
    assets.extend(vehicle_assets);
    (vehicles, assets)
}
// Property assets precede actual-expense vehicle assets in the conversion above.
// Each descriptor is an actual editable Page row, never a stored ordinal used as a Page index.
pub(super) fn document_asset_rows(document: &IncomeSource) -> Vec<ValidationIssue> {
    let key = "ASSET_BONUS_ELECTION_BY_CLASS";
    let rows = |count: usize| {
        (0..count)
            .map(|index| ValidationIssue::new(key).at_list_item("assets", index))
            .collect::<Vec<_>>()
    };
    let (mut result, vehicles) = match document {
        IncomeSource::Record { value: crate::IncomeRecord::Business(v) } => {
            (rows(v.assets.len()), Some(v.vehicles.as_slice()))
        }
        IncomeSource::Record { value: crate::IncomeRecord::Farm(v) } => {
            (rows(v.assets.len()), Some(v.vehicles.as_slice()))
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(v) } => {
            (rows(v.assets.len()), Some(v.vehicles.as_slice()))
        }
        IncomeSource::Record { value: crate::IncomeRecord::Rental(v) } => {
            (rows(v.assets.len()), None)
        }
        IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(v) } => {
            (rows(v.assets.len()), None)
        }
        _ => (Vec::new(), None),
    };
    if let Some(vehicles) = vehicles {
        result.extend(
            vehicles
                .iter()
                .enumerate()
                .filter(|(_, vehicle)| matches!(vehicle.method, VehicleMethod::Actual { .. }))
                .map(|(index, _)| ValidationIssue::new(key).at_list_item("vehicles", index)),
        );
    }
    result
}
// The reverse of vehicle_and_asset_records; None for a stored vehicle
// asset the page cannot show.
pub(crate) fn vehicle_and_asset_items(
    mileage: &[BusinessMileage],
    stored: &[BusinessAsset],
) -> Option<(Vec<Vehicle>, Vec<Asset>)> {
    let mut vehicles: Vec<Vehicle> = mileage.iter().map(mileage_item).collect();
    let mut assets = Vec::new();
    for record in stored {
        if record.is_vehicle() {
            vehicles.push(vehicle_item(record)?);
        } else {
            assets.push(asset_item(record));
        }
    }
    Some((vehicles, assets))
}
// The page's business onto FederalBusiness. Owner health insurance and the
// retirement plan belong to the adjustments page and are kept from the
// accepted record with the same identity. Every other answer, including the
// always-present "More about this business" record, is the person's own.
pub(super) fn business_record(b: &Business, kept: Option<&FederalBusiness>) -> FederalBusiness {
    let (vehicles, assets) = vehicle_and_asset_records(&b.vehicles, &b.assets);
    let more = &b.more;
    FederalBusiness {
        improvement_buildings: b.improvement_buildings.clone(),
        interest_limit_election: b.interest_limit_election.clone().map(Into::into),
        related: b.related.forms.iter().map(|row| row.key).collect(),
        principal_business: b.principal_business.clone(),
        business_code: b.business_code,
        ein: b.ein.clone(),
        address: b.address.clone(),
        started_current_year: b.started_current_year,
        information_returns: b.information_returns,
        other_income: b.other_income.clone(),
        de_minimis: more.de_minimis.clone(),
        depletion: more.depletion.clone(),
        leased_vehicles: b
            .vehicles
            .iter()
            .filter_map(|v| {
                if let VehicleMethod::Leased { vehicle } = &v.method {
                    Some(vehicle.clone())
                } else {
                    None
                }
            })
            .collect(),
        ministerial: more.ministerial,
        notary_profit: more.notary_profit,
        notary_gross: more.notary_gross,
        source: more.source.clone(),
        income_source: more.income_source.record(),
        activity: match &more.gambling {
            None => BusinessActivity::Ordinary {
                returns: b.returns,
                receipts: b.receipts,
                accounting: b.accounting,
            },
            Some(g) => BusinessActivity::Gambling {
                returns: 0,
                receipts: b.receipts,
                accounting: b.accounting,
                losses: g.losses,
                withholding: g.withholding,
                eligible: g.eligible,
            },
        },
        inventory: more.inventory.clone(),
        startup: more.startup.clone(),
        method_changes: more.method_changes.clone(),
        intangibles: more
            .intangibles
            .iter()
            .map(|row| {
                row.record(kept.and_then(|data| {
                    data.intangibles.iter().find(|accepted| accepted.id == row.id && row.id > 0)
                }))
            })
            .collect(),
        timber: more.timber.clone(),
        other_amortization: more.other_amortization.iter().cloned().map(Into::into).collect(),
        health_insurance: kept
            .map_or(BusinessHealthInsurance::None, |k| k.health_insurance.clone()),
        retirement: kept.map_or(BusinessRetirement::None, |k| k.retirement.clone()),
        vehicles,
        assets,
        expenses: expense_records(&b.expenses),
        tips: more.tips.clone().into_record(),
        overtime: more.overtime.clone().into_record(),
        fuel_tax_credit: more.fuel_tax_credit.clone(),
        cash_fuel_tax_recovery: more.cash_fuel_tax_recovery.clone(),
        employer_tip_fica_credit: more.tip_fica.as_ref().map(Into::into),
        work_opportunity_credit: more.work_opportunity.as_ref().map(Into::into),
        retirement_plan_credit: more.retirement_plan.clone(),
        small_employer_health_credit: more.small_employer_health.as_ref().map(Into::into),
        paid_leave_credit: more.paid_leave.as_ref().map(Into::into),
        material_participation: more.material_participation.clone(),
        specified_service: b.specified_service,
        at_risk: more.at_risk.clone(),
        payroll: more.payroll.clone(),
        special_treatment: more.special,
        qbi_aggregation_group: crate::common::optional_text(&more.qbi_group),
        withholding: 0,
        commercial_fishing: more.commercial_fishing,
        sale: more.sale.clone(),
    }
}
// The reverse of business_record; None for a stored business the page's
// answers would not reproduce.
pub(super) fn business_item(entry_label: &str, data: &FederalBusiness) -> Option<Business> {
    let (receipts, accounting, gambling) = match data.activity {
        BusinessActivity::Ordinary { receipts, accounting, .. } => (receipts, accounting, None),
        BusinessActivity::Gambling {
            receipts, accounting, losses, withholding, eligible, ..
        } => (
            receipts,
            accounting,
            Some(crate::ProfessionalGambling { losses, withholding, eligible }),
        ),
        BusinessActivity::Farming { .. } => return None,
    };
    let (mut vehicles, assets) = vehicle_and_asset_items(&data.vehicles, &data.assets)?;
    vehicles.extend(
        data.leased_vehicles
            .iter()
            .cloned()
            .map(|vehicle| Vehicle { method: VehicleMethod::Leased { vehicle } }),
    );
    let item = Business {
        improvement_buildings: data.improvement_buildings.clone(),
        interest_limit_election: data.interest_limit_election.clone().map(Into::into),
        allowed_interest_limit_election: crate::interest_limit::choices(
            data.interest_limit_election.is_some(),
        ),
        related: crate::RelatedForms {
            forms: data
                .related
                .iter()
                .map(|&key| crate::RelatedFormView { key, summary: None })
                .collect(),
            ..Default::default()
        },
        principal_business: data.principal_business.clone(),
        business_code: data.business_code,
        ein: data.ein.clone(),
        address: data.address.clone(),
        started_current_year: data.started_current_year,
        information_returns: data.information_returns,
        returns: match data.activity {
            BusinessActivity::Ordinary { returns, .. }
            | BusinessActivity::Gambling { returns, .. } => returns,
            _ => 0,
        },
        other_income: data.other_income.clone(),
        name: entry_label.into(),
        receipts,
        accounting,
        expenses: expense_items(&data.expenses),
        vehicles,
        assets,
        specified_service: data.specified_service,
        more: BusinessDetails {
            tips_amount_kinds: alloc::vec![1],
            overtime_amount_kinds: alloc::vec![1],
            intangible_predecessor_choices: crate::amortization_records::predecessor_choices(data),
            research_property_choices: crate::cohorts::property_choices(data),
            de_minimis: data.de_minimis.clone(),
            overtime: data.overtime.clone().into(),
            depletion: data.depletion.clone(),
            ministerial: data.ministerial,
            notary_profit: data.notary_profit,
            notary_gross: data.notary_gross,
            inventory: data.inventory.clone(),
            startup: data.startup.clone(),
            method_changes: data.method_changes.clone(),
            intangibles: data.intangibles.iter().cloned().map(Into::into).collect(),
            timber: data.timber.clone(),
            other_amortization: data.other_amortization.iter().cloned().map(Into::into).collect(),
            payroll: data.payroll.clone(),
            work_opportunity: data.work_opportunity_credit.as_ref().map(Into::into),
            retirement_plan: data.retirement_plan_credit.clone(),
            tip_fica: data.employer_tip_fica_credit.as_ref().map(Into::into),
            small_employer_health: data.small_employer_health_credit.as_ref().map(Into::into),
            paid_leave: data.paid_leave_credit.as_ref().map(Into::into),
            fuel_tax_credit: data.fuel_tax_credit.clone(),
            cash_fuel_tax_recovery: data.cash_fuel_tax_recovery.clone(),
            tips: data.tips.clone().into(),
            material_participation: data.material_participation.clone(),
            at_risk: data.at_risk.clone(),
            qbi_group: data.qbi_aggregation_group.clone().unwrap_or_default(),
            income_source: crate::BusinessIncomeChoice::from_record(&data.income_source),
            gambling,
            source: data.source.clone(),
            special: data.special_treatment,
            commercial_fishing: data.commercial_fishing,
            sale: data.sale.clone(),
        },
    };
    (business_record(&item, Some(data)) == *data).then_some(item)
}
pub(super) fn asset_partial_issue(
    class: AssetClass,
    cost: i64,
    placed: u32,
    part: &Option<crate::PartialDisposition>,
) -> Option<ValidationIssue> {
    let issue = |key, field| ValidationIssue::new(key).at_field(field);
    if let Some(part) = part {
        if class != AssetClass::ThirtyNine {
            return Some(issue("PARTIAL_DISPOSITION_NEEDS_A_BUILDING_ROW", "partial"));
        }
        if part.cost_of_part >= cost {
            return Some(
                issue("PARTIAL_DISPOSITION_PART_ABOVE_COST", "cost_of_part").in_field("partial"),
            );
        }
        if part.date <= placed {
            return Some(issue("PARTIAL_DISPOSITION_BEFORE_SERVICE", "date").in_field("partial"));
        }
    }
    None
}
// Each source predicate chooses its local field before outer walkers add indices.
pub(super) fn asset_issue(asset: &Asset) -> Option<ValidationIssue> {
    if let crate::AssetRecoveryPage::NonMacrs { facts } = &asset.recovery {
        if let Some(issue) = disposal_issue(&asset.disposal, asset.placed) {
            return Some(issue.in_field("disposal"));
        }
        let record = facts.record(asset);
        return record
            .nonmacrs_fact_issue()
            .or_else(|| (!record.valid()).then_some("ASSET_NONMACRS_SOURCE_FACTS"))
            .map(|key| ValidationIssue::new(key).at_field("recovery"));
    }
    let crate::AssetRecoveryPage::Gds { facts } = &asset.recovery else {
        return None;
    };
    if let Some(issue) = asset_record(asset, true).macrs_source_fact_issue() {
        return Some(
            issue.in_field("source").in_field("facts").in_field("Gds").in_field("recovery"),
        );
    }
    let issue = |key, field| ValidationIssue::new(key).at_field(field);
    if asset.acquired.is_some_and(|acquired| acquired > asset.placed) {
        return Some(issue("ASSET_ACQUIRED_AFTER_PLACED", "acquired"));
    }
    if asset.credit_reduction > asset.cost {
        return Some(issue("ASSET_CREDIT_ABOVE_COST", "credit_reduction"));
    }
    if let Some(issue) = disposal_issue(&asset.disposal, asset.placed) {
        return Some(issue.in_field("disposal"));
    }
    if asset.earlier_business_percent.is_some()
        && (!matches!(facts.history, AssetHistory::Continuing { .. })
            || asset.business_percent > 50)
    {
        return Some(issue("ASSET_EARLIER_USE_ONLY_AFTER_USE_FELL", "earlier_business_percent"));
    }
    if let Some(issue) = asset_partial_issue(facts.class, asset.cost, asset.placed, &asset.partial)
    {
        return Some(issue);
    }
    if let Some(issue) = history_issue(&facts.history, asset.placed) {
        return Some(issue);
    }
    if let AssetHistory::New { first_year: FirstYearDeduction::Section179 { elected, .. } } =
        &facts.history
        && i128::from(*elected) * 100
            > i128::from(asset.cost - asset.credit_reduction) * i128::from(asset.business_percent)
    {
        return Some(
            issue("ASSET_179_ABOVE_BUSINESS_COST", "elected")
                .in_field("first_year")
                .in_field("history"),
        );
    }
    investment_issue(asset)
}
pub(super) fn investment_issue(asset: &Asset) -> Option<ValidationIssue> {
    use crate::InvestmentCredit;
    let crate::AssetRecoveryPage::Gds { facts } = &asset.recovery else {
        return None;
    };
    let issue = |key, field| ValidationIssue::new(key).at_field(field);
    let new = matches!(facts.history, AssetHistory::New { .. });
    match &asset.investment_credit {
        InvestmentCredit::No => None,
        InvestmentCredit::Rehabilitation(claim) => {
            if facts.class != AssetClass::ThirtyNine
                || asset.placed
                    < crate::year_constants::REHABILITATION_FIRST_PLACED_YEAR * 10000 + 101
            {
                return Some(issue(
                    "REHABILITATION_NEEDS_A_BUILDING_ROW",
                    if facts.class != AssetClass::ThirtyNine { "class" } else { "placed" },
                ));
            }
            if claim.period_start >= asset.placed {
                return Some(
                    issue("REHABILITATION_PERIOD_AFTER_PLACED", "period_start")
                        .in_field("investment_credit"),
                );
            }
            let qre =
                crate::kernel::ratio::fraction(asset.cost, i64::from(asset.business_percent), 100);
            claim.issue(asset.placed, qre).map(|issue| issue.in_field("investment_credit"))
        }
        InvestmentCredit::NewEnergy { construction_began, .. } => {
            if facts.class != AssetClass::Five || !new {
                Some(issue(
                    "ENERGY_CREDIT_NEEDS_NEW_5_YEAR_PROPERTY",
                    if facts.class != AssetClass::Five { "class" } else { "history" },
                ))
            } else if *construction_began > asset.placed {
                Some(
                    issue("ENERGY_CONSTRUCTION_AFTER_PLACED", "construction_began")
                        .in_field("investment_credit"),
                )
            } else {
                None
            }
        }
        InvestmentCredit::EarlierEnergy { credit, used, carried, .. } => {
            if facts.class != AssetClass::Five || new {
                Some(issue(
                    "EARLIER_ENERGY_NEEDS_CONTINUING_5_YEAR_PROPERTY",
                    if facts.class != AssetClass::Five { "class" } else { "history" },
                ))
            } else if i128::from(*used) + i128::from(*carried) > i128::from(*credit) {
                Some(
                    issue("INVESTMENT_CREDIT_HISTORY_ABOVE_CREDIT", "carried")
                        .in_field("investment_credit"),
                )
            } else {
                None
            }
        }
    }
}
pub(super) fn disposal_issue(
    disposal: &crate::AssetDisposal,
    placed: u32,
) -> Option<ValidationIssue> {
    if let crate::AssetDisposal::Disposed {
        how: crate::DisposalKind::Sold { proceeds, payment, buyer, .. },
        ..
    } = disposal
    {
        if let Some(issue) = payment.issue(*proceeds) {
            return Some(issue.in_field("payment").in_field("how"));
        }
        if *buyer != crate::Buyer::Unrelated && payment.installment() {
            return Some(
                ValidationIssue::new("INSTALLMENT_RELATED_BUYER").at_field("buyer").in_field("how"),
            );
        }
    }
    disposal
        .date()
        .filter(|date| *date <= placed)
        .map(|_| ValidationIssue::new("ASSET_DISPOSED_BEFORE_PLACED").at_field("date"))
}
// The date belongs to the containing asset or actual-expense vehicle, not history.
pub(super) fn history_issue(history: &AssetHistory, placed: u32) -> Option<ValidationIssue> {
    let key = match history {
        AssetHistory::New { .. } if placed / 10000 != 2026 => "ASSET_NEW_PLACED_IN_2026",
        AssetHistory::Continuing { .. } if placed >= YEAR_FIRST_DAY => {
            "ASSET_OLD_PLACED_BEFORE_2026"
        }
        _ => return None,
    };
    Some(ValidationIssue::new(key).at_field("placed"))
}
pub(super) fn business_issue(b: &Business) -> Option<ValidationIssue> {
    let more = &b.more;
    if let Some(issue) =
        more.at_risk.issue_for_method(&crate::RiskEffectiveCategory::OtherTradeBusiness, true)
    {
        return Some(issue.in_field("at_risk").in_field("more"));
    }
    if let Some(issue) = more
        .fuel_tax_credit
        .as_ref()
        .and_then(|claim| claim.fuels.vendor_blocked_pump_kerosene.as_ref())
        .and_then(|claim| claim.buyer_issue())
    {
        return Some(
            issue
                .in_field("vendor_blocked_pump_kerosene")
                .in_field("fuels")
                .in_field("fuel_tax_credit")
                .in_field("more"),
        );
    }

    if let Some(issue) = more.payroll.issue() {
        return Some(issue.in_field("payroll").in_field("more"));
    }
    for (index, meal) in b.expenses.meals.iter().enumerate() {
        if b.expenses.meals[..index].iter().any(|prior| prior.kind == meal.kind) {
            return Some(
                ValidationIssue::new("BUSINESS_MEAL_KIND_ONCE")
                    .at_field("kind")
                    .in_list("meals", index)
                    .in_field("expenses"),
            );
        }
    }
    if let Some(issue) = more.de_minimis.as_ref().and_then(|value| value.issue()) {
        return Some(issue.in_field("de_minimis").in_field("more"));
    }
    if let Some(issue) = more.depletion.as_ref().and_then(|value| value.issue()) {
        return Some(issue.in_field("depletion").in_field("more"));
    }
    if let Some(issue) = crate::method_change::list_issue(&more.method_changes) {
        return Some(issue.in_field("more"));
    }
    if (more.work_opportunity.is_some()
        || more.retirement_plan.is_some()
        || more.tip_fica.is_some()
        || more.small_employer_health.is_some()
        || more.paid_leave.is_some())
        && more.payroll == BusinessPayroll::No
    {
        return Some(
            ValidationIssue::new("EMPLOYER_CREDITS_NEED_PAYROLL")
                .at_field("payroll")
                .in_field("more"),
        );
    }
    if more.gambling.as_ref().is_some_and(|gambling| gambling.withholding > b.receipts) {
        return Some(
            ValidationIssue::new("GAMBLING_BUSINESS_WITHHOLDING_ABOVE_WINNINGS")
                .at_field("withholding")
                .in_field("gambling")
                .in_field("more"),
        );
    }
    if let Some(issue) = more
        .work_opportunity
        .as_ref()
        .map(crate::WorkOpportunityCredit::from)
        .and_then(|claim| claim.issue())
    {
        return Some(issue.in_field("work_opportunity").in_field("more"));
    }
    if let Some(issue) = more
        .small_employer_health
        .as_ref()
        .map(crate::SmallEmployerHealthCredit::from)
        .and_then(|claim| claim.issue())
    {
        return Some(issue.in_field("small_employer_health").in_field("more"));
    }
    if let Some(issue) =
        more.paid_leave.as_ref().map(crate::PaidLeaveCredit::from).and_then(|claim| claim.issue())
    {
        return Some(issue.in_field("paid_leave").in_field("more"));
    }
    if let BusinessPayroll::Yes { statements, .. } = &more.payroll {
        let listed = |label: &str| statements.iter().any(|w| w.label == label);
        for (field, labels) in [
            (
                "work_opportunity",
                more.work_opportunity
                    .iter()
                    .flat_map(|claim| claim.employees.iter().map(|e| e.label.as_str()))
                    .collect::<Vec<_>>(),
            ),
            (
                "tip_fica",
                more.tip_fica
                    .iter()
                    .flat_map(|claim| claim.employees.iter().map(|e| e.label.as_str()))
                    .collect::<Vec<_>>(),
            ),
            (
                "small_employer_health",
                more.small_employer_health
                    .iter()
                    .flat_map(|claim| claim.employees.iter().map(|e| e.label.as_str()))
                    .collect::<Vec<_>>(),
            ),
            (
                "paid_leave",
                more.paid_leave
                    .iter()
                    .flat_map(|claim| claim.employees.iter().map(|e| e.label.as_str()))
                    .collect::<Vec<_>>(),
            ),
        ] {
            for (index, label) in labels.iter().enumerate() {
                if !listed(label) {
                    return Some(
                        ValidationIssue::new("EMPLOYER_CREDIT_EMPLOYEE_NOT_ON_PAYROLL")
                            .at_field("label")
                            .in_list("employees", index)
                            .in_field(field)
                            .in_field("more"),
                    );
                }
            }
        }
    }
    shared_business_issue(&b.vehicles, &b.assets)
}
// Preserve the existing group-wide order: office, ordinary vehicles, leases, assets.
pub(crate) fn shared_business_issue(
    vehicles: &[Vehicle],
    assets: &[Asset],
) -> Option<ValidationIssue> {
    for (index, vehicle) in vehicles.iter().enumerate() {
        let issue = |key, field| ValidationIssue::new(key).at_field(field);
        let local = match &vehicle.method {
            VehicleMethod::StandardMileage { first_half, second_half, total, more, .. }
                if u64::from(*first_half) + u64::from(*second_half) + u64::from(more.commuting)
                    > u64::from(*total) =>
            {
                Some(if first_half > total {
                    issue("MILEAGE_ABOVE_TOTAL_MILES", "first_half")
                } else if u64::from(*first_half) + u64::from(*second_half) > u64::from(*total) {
                    issue("MILEAGE_ABOVE_TOTAL_MILES", "second_half")
                } else {
                    issue("MILEAGE_ABOVE_TOTAL_MILES", "commuting").in_field("more")
                })
            }
            VehicleMethod::Actual { total_miles, business_miles, .. }
                if business_miles > total_miles =>
            {
                Some(issue("VEHICLE_BUSINESS_MILES_ABOVE_TOTAL", "business_miles"))
            }
            VehicleMethod::Actual { cost, credit_reduction, .. } if credit_reduction > cost => {
                Some(issue("ASSET_CREDIT_ABOVE_COST", "credit_reduction"))
            }
            VehicleMethod::Actual {
                earlier_business_percent: Some(_),
                total_miles,
                business_miles,
                history,
                ..
            } if !matches!(history, AssetHistory::Continuing { .. })
                || u64::from(*business_miles) * 2 > u64::from(*total_miles) =>
            {
                Some(issue("VEHICLE_EARLIER_USE_ONLY_AFTER_USE_FELL", "earlier_business_percent"))
            }
            VehicleMethod::Actual {
                kind,
                history:
                    AssetHistory::New { first_year: FirstYearDeduction::Section179 { elected, .. } },
                ..
            } if kind.section179_limit().is_some_and(|limit| *elected > limit) => Some(
                issue("ASSET_179_ABOVE_SUV_LIMIT", "elected")
                    .in_field("Section179")
                    .in_field("first_year")
                    .in_field("New")
                    .in_field("history")
                    .in_field("Actual"),
            ),
            VehicleMethod::Actual { acquired: Some(acquired), placed, .. } if acquired > placed => {
                Some(issue("ASSET_ACQUIRED_AFTER_PLACED", "acquired"))
            }
            VehicleMethod::Actual { placed, history, disposal, .. } => {
                history_issue(history, *placed).or_else(|| {
                    disposal_issue(disposal, *placed).map(|issue| issue.in_field("disposal"))
                })
            }
            _ => None,
        };
        if let Some(issue) = local {
            return Some(issue.in_field("method").in_list("vehicles", index));
        }
        let (_, records) = vehicle_and_asset_records(core::slice::from_ref(vehicle), &[]);
        if let Some(issue) = records.first().and_then(crate::auto_recovery::history_issue) {
            return Some(issue.in_field("method").in_list("vehicles", index));
        }
    }
    for (index, vehicle) in vehicles.iter().enumerate() {
        if let VehicleMethod::Leased { vehicle } = &vehicle.method
            && let Some(issue) = vehicle.issue()
        {
            return Some(issue.in_field("vehicle").in_field("method").in_list("vehicles", index));
        }
    }
    assets
        .iter()
        .enumerate()
        .find_map(|(index, asset)| asset_issue(asset).map(|issue| issue.in_list("assets", index)))
}
// Equipment rented out outside a business is 100% income-producing
// nonbusiness property: section 179 is not available to it.
pub(super) fn equipment_rent_issue(v: &EquipmentRent) -> Option<ValidationIssue> {
    v.assets.iter().enumerate().find_map(|(index, asset)| {
        asset_issue(asset)
            .or_else(|| {
                asset.investment_credit.is_some().then_some(
                    ValidationIssue::new("EQUIPMENT_RENT_NO_INVESTMENT_CREDIT")
                        .at_field("investment_credit"),
                )
            })
            .or_else(|| {
                match &asset.recovery {
                    crate::AssetRecoveryPage::NonMacrs { facts } => {
                        facts.record(asset).available179() != 0
                    }
                    crate::AssetRecoveryPage::Gds { facts } => matches!(
                        facts.history,
                        AssetHistory::New { first_year: FirstYearDeduction::Section179 { .. } }
                    ),
                }
                .then_some(
                    ValidationIssue::new("EQUIPMENT_RENT_NO_SECTION_179")
                        .at_field("first_year")
                        .in_field("history"),
                )
            })
            .map(|issue| issue.in_list("assets", index))
    })
}

// Whether the return holds a self-employment business, which the
// self-employed benefits topic needs (M3-P8: offered only when one exists).
pub(super) fn has_ordinary_business(d: &FederalDraft) -> bool {
    d.income.iter().any(|e| topic_of(d, e) == Some(TaxSection::SelfEmployment))
}
