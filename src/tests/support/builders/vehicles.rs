use crate::tests::*;

pub(in crate::tests) fn owned_actual_vehicle(first_year: FirstYearDeduction) -> Vehicle {
    Vehicle {
        method: VehicleMethod::Actual {
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            allowed_amt_carry: Vec::new(),
            cost: 1_000_000,
            credit_reduction: 0,
            acquired: Some(20260310),
            placed: 20260315,
            total_miles: 10_000,
            business_miles: 7_500,
            operating_costs: 400_000,
            loan_interest: 200_000,
            passenger_automobile: true,
            earlier_business_percent: None,
            history: AssetHistory::New { first_year },
            disposal: AssetDisposal::No,
        },
    }
}

pub(in crate::tests) fn personal_refueling(
    cost: i64,
    placed_in_service: u32,
) -> FederalPersonalRefueling {
    FederalPersonalRefueling {
        label: "Synthetic home charging unit".into(),
        placed_in_service,
        cost,
        census_geoid: "12345678901".into(),
        eligible_tract: true,
        qualified_property: true,
        personal_only: true,
    }
}

pub(in crate::tests) fn clean_vehicle_details(
    vin: &str,
    buyer: Owner,
) -> FederalCleanVehicleDetails {
    FederalCleanVehicleDetails {
        vin: vin.into(),
        buyer,
        acquired: 20250930,
        placed_in_service: 20260115,
        accepted_seller_report: true,
        qualified_vehicle: true,
        owned_for_use: true,
        disposition: crate::CleanVehicleDisposition::Kept,
        dealer_transfer: None,
    }
}

pub(in crate::tests) fn new_clean_vehicle() -> FederalCleanVehicle {
    FederalCleanVehicle::New {
        details: clean_vehicle_details("1HGBH41JXMN109186", Owner::Taxpayer),
        seller_credit: 750_000,
        business_use: None,
    }
}

pub(in crate::tests) fn used_clean_vehicle() -> FederalCleanVehicle {
    FederalCleanVehicle::PreviouslyOwned {
        details: clean_vehicle_details("1HGBH41JXMN109187", Owner::Taxpayer),
        model_year: 2023,
        sales_price: 2_000_000,
        first_eligible_transfer: true,
        no_credit_in_three_years: true,
    }
}

pub(in crate::tests) fn clean_vehicle_claim(
    vehicles: Vec<FederalCleanVehicle>,
) -> FederalCleanVehicleClaim {
    FederalCleanVehicleClaim { prior_year: None, current_territory_addbacks: 0, vehicles }
}

// A qualifying Schedule 1-A vehicle loan (T3-deduct).
pub(in crate::tests) fn vehicle_loan(lender: &str, interest: i64) -> FederalVehicleInterest {
    FederalVehicleInterest {
        label: lender.into(),
        interest,
        business: 0,
        original_use: true,
        assembled_in_us: true,
    }
}

pub(in crate::tests) fn native_business_mileage(
    first_half: u32,
    second_half: u32,
    total: u32,
) -> BusinessMileage {
    BusinessMileage {
        id: 0,
        first_half,
        second_half,
        total,
        placed: None,
        personal_use_available: true,
        other_vehicle: true,
        evidence: VehicleEvidence::Written,
        commuting: 0,
        parking_tolls: 0,
        interest: 0,
        property_tax: 0,
        owned: true,
    }
}

// Approved NRG-G1 F1: independent net Schedule C $20,000 after $4,500
// ordinary first-year depreciation of $37,500 x 60% business basis.
pub(in crate::tests) fn clean_vehicle_business_return() -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 4_000_000, 0);
    let mut source = cash_business(24_500);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    let mut asset = depreciable_asset(45_000, EquipmentClass::Five, 20260120, 0);
    asset.acquired = 20250915;
    asset.credit_reduction = 750_000;
    asset.listed = ListedProperty::Vehicle {
        total_miles: 15_000,
        business_miles: 9_000,
        operating_costs: 0,
        loan_interest: 0,
        passenger_automobile: true,
    };
    data.assets.push(asset);
    add(&mut draft, income(source));
    let property = draft.next_id;
    draft.next_id += 1;
    let entry = draft.income.last_mut().unwrap();
    let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
    data.assets[0].id = property;
    let mut vehicle = new_clean_vehicle();
    let FederalCleanVehicle::New { details, business_use, .. } = &mut vehicle else {
        unreachable!()
    };
    details.acquired = 20250915;
    details.placed_in_service = 20260120;
    *business_use = Some(crate::clean_vehicle::CleanVehicleBusinessUse {
        activity: crate::clean_vehicle::CleanVehicleActivity::Business { business: entry.id },
        property,
        credit_use: crate::UseFraction { used: 3, total: 5 },
    });
    draft.credits.as_mut().unwrap().clean_vehicles = Some(clean_vehicle_claim(vec![vehicle]));
    draft
}
