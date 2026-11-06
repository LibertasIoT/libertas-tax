use crate::tests::*;

// https://www.irs.gov/publications/p946, Table B-1 and ADS recovery periods.

// Office information-system equipment, rented to a governmental unit under
// a documented 49-month lease. Pub946 Table B-1 class00.12 has a five-year
// ADS period; the qualifying lease raises that period to61.25 months.
pub(in crate::tests) fn required_ads_equipment() -> BusinessAsset {
    let mut asset =
        macrs_equipment(10_000, EquipmentClass::Five, EquipmentMethod::Regular, 20260601);
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::Apply,
        tables: true,
    };
    asset.macrs_source = Some(crate::MacrsRecoverySource {
        election: None,
        restrictions: Some(vec![crate::AdsRestrictionPeriod {
            first_year: 2026,
            last_year: None,
            kind: crate::AdsRestriction::TaxExemptUse,
        }]),
        foreign_years: Some(vec![crate::ForeignUseYear {
            year: 2026,
            outside_us: crate::UseFraction { used: 0, total: 1 },
        }]),
        original_system: Some(crate::OriginalMacrsSystem::Ads { began_year: 2026 }),
        class_life: Some(crate::AdsClassLife { table_asset_class: "00.12".into(), months: 72 }),
        lease: Some(crate::AdsLease { term_months: 49, lessee: crate::AdsLessee::Government }),
        improvement: None,
    });
    asset
}

pub(in crate::tests) fn voluntary_ads_equipment() -> BusinessAsset {
    let mut asset = required_ads_equipment();
    let source = asset.macrs_source.as_mut().unwrap();
    source.election = Some(crate::AdsElection { tax_year: 2026 });
    source.restrictions = Some(vec![]);
    source.lease = None;
    asset
}

// The retained section263A election belongs to the farming source, not each asset.
pub(in crate::tests) fn farm_with_capitalization_election(asset: BusinessAsset) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut row = raised_farm(100_000, 0);
    let FederalIncome::Business { data } = &mut row else { unreachable!() };
    data.assets.push(asset);
    add(&mut draft, income(row));
    let cohort_id = draft.next_id;
    draft.next_id += 1;
    let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    farm.specials.production.push(FarmProductionCohort {
        id: cohort_id,
        description: "Retained tree-production election".into(),
        property: FarmProducedKind::Plant,
        began: 20250101,
        productive: None,
        normal_preproductive_months: 36,
        tax_treatment: FarmCapitalization::OriginalElectionOut {
            year: FarmElectionYear { year: 2025, statement: "Original timely election".into() },
        },
        costs: vec![],
        prior_basis: 0,
        prior_amt_basis: 0,
        destination: FarmBasisDestination::Inventory { released: 0, amt_released: 0 },
    });
    draft
}

pub(in crate::tests) fn income_with_ads_asset(
    asset: BusinessAsset,
    rental_activity: bool,
) -> FederalIncome {
    if rental_activity {
        let mut data = rental(48_000, 10_000);
        data.assets.push(asset);
        FederalIncome::Rental { data }
    } else {
        let mut row = cash_business(100_000);
        let FederalIncome::Business { data } = &mut row else { unreachable!() };
        data.assets.push(asset);
        row
    }
}

pub(in crate::tests) fn voluntary_ads_passenger_car() -> BusinessAsset {
    let mut asset = actual_vehicle(200_000, 10_000, 10_000);
    asset.macrs_source = voluntary_ads_equipment().macrs_source;
    asset.macrs_source.as_mut().unwrap().class_life = None;
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::ElectOut,
        tables: true,
    };
    asset
}

pub(in crate::tests) fn employee_with_inherited_ads_vehicle(packet_supplied: bool) -> FederalDraft {
    let mut equipment = voluntary_ads_equipment();
    equipment.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::ElectOut,
        tables: true,
    };
    let mut draft = business_with_bonus(100_000, equipment);
    let mut vehicle = voluntary_ads_passenger_car();
    if packet_supplied {
        let source = vehicle.macrs_source.as_mut().unwrap();
        source.election = None;
        source.original_system = None;
    } else {
        vehicle.macrs_source = None;
    }
    draft.adjustments.as_mut().unwrap().employee_expenses.push(EmployeeExpenseClaim {
        improvement_buildings: Vec::new(),
        owner: Owner::Taxpayer,
        occupation: "Fee-basis public official".into(),
        capacity: EmployeeCapacity::FeeBasisOfficial,
        employers: vec![1],
        local_transportation: 0,
        travel: 0,
        other: 0,
        meals: 0,
        reimbursements: 0,
        dot_hours_of_service: false,
        mileage: vec![],
        leased_vehicles: vec![],
        vehicle_assets: vec![vehicle],
        equipment_assets: vec![],
    });
    draft
}

// Both records begin as ordinary current-year equipment with complete source
// answers. An election can belong to the other joint filer's first record.
pub(in crate::tests) fn ads_question_correction_draft(
    joint: bool,
    election_owner: Option<Owner>,
) -> FederalDraft {
    let mut draft = complete(if joint { FederalStatus::Joint } else { FederalStatus::Single });
    for index in 0..2 {
        let mut asset = voluntary_ads_equipment();
        let source = asset.macrs_source.as_mut().unwrap();
        if index != 0 || election_owner.is_none() {
            source.election = None;
            source.original_system = None;
            source.class_life = None;
        }
        let mut entry = income(income_with_ads_asset(asset, false));
        entry.label =
            if index == 0 { "First equipment activity" } else { "Second equipment activity" }
                .into();
        if index == 0 {
            entry.owner = crate::DocumentOwner::from(election_owner.unwrap_or(Owner::Taxpayer));
        }
        add(&mut draft, entry);
    }
    draft
}

// Compare every submitted answer and request identity while excluding only
// the server-owned question metadata and the correction's warning output.
pub(in crate::tests) fn ads_editor_answers(mut request: P) -> P {
    let P::SaveIncomes { documents, warnings, .. } = &mut request else { unreachable!() };
    warnings.clear();
    for item in &mut documents.documents {
        let crate::IncomeRecord::Business(business) = &mut item.document else { unreachable!() };
        business.allowed_interest_limit_election.clear();
        business.more.research_property_choices.clear();
        business.more.intangible_predecessor_choices.clear();
        for asset in &mut business.assets {
            let crate::AssetRecoveryPage::Gds { facts } = &mut asset.recovery else {
                unreachable!()
            };
            facts.macrs_question_context = None;
            if let Some(source) = &mut facts.source {
                source.allowed_original_system.clear();
                source.allowed_class_life.clear();
            }
        }
    }
    request
}

// Pub946 Table B-1 class00.3: GDS15 years, class life and ADS20 years.
pub(in crate::tests) fn interest_land_improvement() -> BusinessAsset {
    let mut asset = voluntary_ads_equipment();
    asset.class = EquipmentClass::Fifteen;
    asset.macrs_source.as_mut().unwrap().class_life =
        Some(crate::AdsClassLife { table_asset_class: "00.3".into(), months: 240 });
    asset
}

pub(in crate::tests) fn interest_activity_draft(
    family: &str,
    election: Option<crate::InterestLimitElection>,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut asset = interest_land_improvement();
    if family == "Business" || family == "Rental" {
        asset.class = EquipmentClass::ThirtyNine;
        asset.history = EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::Apply,
            tables: true,
        };
        asset.macrs_source.as_mut().unwrap().class_life = None;
    }
    let row = match family {
        "Farm" => {
            let mut row = raised_farm(100_000, 0);
            let FederalIncome::Business { data } = &mut row else { unreachable!() };
            data.assets.push(asset);
            data.interest_limit_election = election;
            row
        }
        "FarmRental" => {
            let mut data = farm_rental_data(10_000_000, 0);
            data.assets.push(asset);
            data.interest_limit_election = election;
            FederalIncome::FarmRental { data }
        }
        "Rental" => {
            let mut data = rental(48_000, 10_000);
            data.assets.push(asset);
            data.interest_limit_election = election;
            FederalIncome::Rental { data }
        }
        "Business" => {
            let mut row = cash_business(100_000);
            let FederalIncome::Business { data } = &mut row else { unreachable!() };
            data.assets.push(asset);
            data.interest_limit_election = election;
            row
        }
        _ => panic!("unknown fixture activity"),
    };
    add(&mut draft, income(row));
    draft
}

pub(in crate::tests) fn qip_building(id: i64) -> crate::ImprovementBuildingFacts {
    crate::ImprovementBuildingFacts {
        id,
        description: "Market Street office building".into(),
        facts: crate::BuildingUseSource {
            first_service: Some(crate::BuildingFirstService::ExactDate { date: 19980601 }),
            kind: crate::RecoveryBuildingKind::Nonresidential,
        },
    }
}

pub(in crate::tests) fn qip_asset(building_id: i64) -> BusinessAsset {
    let mut asset = voluntary_ads_equipment();
    asset.id = 31;
    asset.class = EquipmentClass::ThirtyNine;
    asset.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::ElectOut,
        tables: true,
    };
    let source = asset.macrs_source.as_mut().unwrap();
    source.election = None;
    source.original_system = None;
    source.class_life = None;
    source.improvement = Some(crate::RealImprovement {
        building_id,
        kind: crate::RealImprovementKind::Interior1250 { made_by_taxpayer: true },
    });
    asset
}

pub(in crate::tests) fn qip_draft(asset: BusinessAsset, family: &str) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut row = match family {
        "Farm" => raised_farm(100_000, 0),
        "Rental" => FederalIncome::Rental { data: rental(48_000, 10_000) },
        "FarmRental" => FederalIncome::FarmRental { data: farm_rental_data(10_000_000, 0) },
        _ => cash_business(100_000),
    };
    let (assets, buildings) = match &mut row {
        FederalIncome::Business { data } => (&mut data.assets, &mut data.improvement_buildings),
        FederalIncome::Rental { data } => (&mut data.assets, &mut data.improvement_buildings),
        FederalIncome::FarmRental { data } => (&mut data.assets, &mut data.improvement_buildings),
        _ => unreachable!(),
    };
    assets.push(asset);
    buildings.push(qip_building(30));
    add(&mut draft, income(row));
    draft.next_id = 32;
    draft
}

pub(in crate::tests) fn employee_qip_draft() -> FederalDraft {
    let mut draft = employee_with_inherited_ads_vehicle(true);
    let claim = &mut draft.adjustments.as_mut().unwrap().employee_expenses[0];
    claim.vehicle_assets.clear();
    claim.equipment_assets.push(qip_asset(30));
    claim.improvement_buildings.push(qip_building(30));
    draft.next_id = 32;
    draft
}
