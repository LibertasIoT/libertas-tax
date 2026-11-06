use super::*;
use crate::tests::cross::interview::{accepted, asset, business, document, open_topic, save_topic};
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

#[test]
fn unit17_direct_amt_carry_actual_editor_save_reopen_restart_and_clear() {
    extern crate std;
    use libertas::AvroEncode;
    let client = NativeInterviewCase("TAX_UNIT17_AMT_CARRY");
    let kind = std::env::var("TAX_UNIT17_AMT_CARRY_KIND").unwrap_or_else(|_| "business".into());
    let clear = std::env::var("TAX_UNIT17_AMT_CARRY_CLEAR").is_ok();
    let topic = if kind == "farm" { TaxSection::Farming } else { TaxSection::SelfEmployment };
    let mut initial = complete(FederalStatus::Single);
    initial.income.clear();
    if let Ok(file) = std::env::var("TAX_UNIT17_AMT_CARRY_PRIOR_DRAFT") {
        initial = read_native_draft(file);
    }
    let mut session = restart(initial);
    let seed = open_topic(&mut session, topic);
    if client.required_calculation(&mut session) {
        return;
    }
    let mut pages = Vec::new();
    for index in 0..3 {
        let mut row = asset(2_000_000, 20250101, AssetClass::Five);
        row.acquired = Some(20250101);
        row.gds_mut().unwrap().history = AssetHistory::Continuing {
            prior_depreciation: if index == 2 { 1_200_000 } else { 1_100_000 },
            first_year_179: 1_000_000,
            first_year_bonus: 0,
            carryforward: if index == 2 { 0 } else { 100_000 },
            mid_quarter: false,
        };
        row.amt_section179_income_carry =
            AssetAmt179CarryAnswer::from_amount(Some(if index == 1 { 0 } else { 30_000 }));
        row.allowed_amt_carry = vec![0];
        pages.push(row);
    }
    let value = if kind == "farm" {
        let mut page = crate::tests::cross::interview::cash_farm(10_000_000, 0);
        page.assets = pages;
        IncomeSource::Record { value: crate::IncomeRecord::Farm(page) }
    } else {
        let mut page = business(10_000_000);
        page.business_code = Some(541990);
        if kind == "vehicle" {
            page.vehicles = pages
                .into_iter()
                .map(|asset| Vehicle {
                    method: VehicleMethod::Actual {
                        macrs_source: None,
                        id: 0,
                        history_details: None,
                        cost: asset.cost,
                        credit_reduction: 0,
                        acquired: None,
                        placed: asset.placed,
                        total_miles: 10_000,
                        business_miles: 10_000,
                        operating_costs: 0,
                        loan_interest: 0,
                        kind: crate::VehicleKind::HeavyTruckVan,
                        earlier_business_percent: None,
                        history: asset.gds().unwrap().history.clone(),
                        disposal: AssetDisposal::No,
                        amt_section179_income_carry: asset
                            .amt_section179_income_carry
                            .map(alloc::boxed::Box::new),
                        allowed_amt_carry: vec![0],
                    },
                })
                .collect();
        } else {
            page.assets = pages;
        }
        IncomeSource::Record { value: crate::IncomeRecord::Business(page) }
    };
    let mut request = save_topic(&mut session, topic, vec![document(0, value)]);
    let write = |name: &str, value: &P| {
        if let Ok(directory) = std::env::var("TAX_UNIT17_AMT_CARRY_DIR") {
            let mut bytes = Vec::new();
            value.avro_encode(&mut bytes);
            std::fs::write(std::path::Path::new(&directory).join(name), bytes).unwrap();
        }
    };
    write("seed.avro", &seed);
    write("answers.avro", &request);
    if let Some(payload) = client.input("PAYLOAD") {
        request = payload;
    }
    accepted(&send(&mut session, request));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
        .unwrap();
    let FederalIncome::Business { data } = &entry.income else { unreachable!() };
    assert_eq!(data.assets.len(), 3);
    let ids = data.assets.iter().map(|asset| asset.id).collect::<alloc::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 3);
    assert!(ids.iter().all(|id| *id > 0));
    for (index, asset) in data.assets.iter().enumerate() {
        let actual =
            if clear && index == 0 { None } else { Some(if index == 1 { 0 } else { 30_000 }) };
        assert_eq!(asset.amt_section179_income_carry, actual);
        assert_eq!(asset.amt_section179_carry(), actual.ok_or(()));
        assert_eq!(asset.available179(), if index == 2 { 0 } else { 100_000 });
        assert_eq!(asset.deduction_for(false, false), 320_000);
        assert_eq!(asset.deduction_for(false, true), 320_000);
    }
    assert_eq!(data.assets.iter().map(|asset| asset.deduction(false)).sum::<i64>(), 960_000);
    assert_eq!(
        data.assets.iter().map(|asset| asset.deduction_for(false, true)).sum::<i64>(),
        960_000
    );
    let saved = session.draft.clone();
    let records = native_draft_records(&saved);
    if let Ok(directory) = std::env::var("TAX_UNIT17_AMT_CARRY_DIR") {
        write_native_draft_records(std::path::Path::new(&directory).join("draft.avro"), &records);
    }
    let draft = draft_from_native_records(saved.tax_year, &records);
    assert_eq!(native_draft_records(&draft), records);
    assert_eq!(draft, saved);
    let mut again = restart(draft);
    let reopened = open_topic(&mut again, topic);
    write("reopened.avro", &reopened);
    accepted(&send(&mut again, reopened));
    assert_eq!(again.draft, saved);
}

#[test]
fn unit17_direct_amt_carry_question_uses_actual_missing_source_only() {
    let new = bonus_equipment(20_000, 100);
    assert!(new.amt_carry_choices().is_empty());
    let mut asset = new.clone();
    asset.acquired = 20170927;
    asset.placed = 20250101;
    asset.history = EquipmentHistory::Continuing {
        initial_section179: 1_000_000,
        carryforward: 100_000,
        prior_depreciation: 1_100_000,
        amt: EquipmentAmtHistory::Separate {
            prior_depreciation: 1_120_000,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    assert_eq!(asset.amt_carry_choices(), vec![0]);
    let unknown: CanonicalAssetPage = asset.clone().into();
    assert_eq!(unknown.macrs().unwrap().amt_section179_income_carry, None);
    let unchanged: BusinessAsset = unknown.into();
    assert_eq!(unchanged, asset);
    asset.history_details = Some(direct_amt_carry_history());
    assert!(asset.valid());
    assert_eq!(asset.amt_section179_carry(), Ok(30_000));
    assert!(asset.amt_carry_choices().is_empty());
    let page: CanonicalAssetPage = asset.clone().into();
    assert!(page.macrs().unwrap().allowed_amt_carry.is_empty());
    assert_eq!(page.macrs().unwrap().amt_section179_income_carry, None);
    let unchanged: BusinessAsset = page.into();
    assert_eq!(unchanged, asset);
    asset.amt_section179_income_carry = Some(30_000);
    let page: CanonicalAssetPage = asset.clone().into();
    assert!(page.macrs().unwrap().allowed_amt_carry.is_empty());
    assert_eq!(page.macrs().unwrap().amt_section179_income_carry, None);
    let unchanged: BusinessAsset = page.into();
    assert_eq!(unchanged, asset);
    // An authoritative correction to an inapplicable history withdraws only
    // the separate carry answer; it does not invent a year or zero balance.
    let mut page: CanonicalAssetPage = asset.into();
    page.macrs_mut().unwrap().financial.history = new.history;
    page.macrs_mut().unwrap().financial.history_details = None;
    let corrected: BusinessAsset = page.into();
    assert_eq!(corrected.amt_section179_income_carry, None);

    // Correct an independently supplied balance through the ordinary rental
    // page when complete annual records make that separate answer inapplicable.
    use crate::tests::cross::interview::{
        assert_rentals_round_trip, rental_page, reopened_rentals, save_rentals, stored_rentals,
    };
    let mut manual = unchanged;
    manual.history_details = None;
    // This rental has exhausted its regular election: actual1000000 allowed
    // plus200000 other depreciation. Its independently limited AMT carry differs.
    let EquipmentHistory::Continuing { prior_depreciation, carryforward, .. } = &mut manual.history
    else {
        unreachable!()
    };
    *prior_depreciation = 1_200_000;
    *carryforward = 0;
    assert!(manual.valid());
    assert_eq!(manual.available179(), 0);
    assert_eq!(manual.amt_section179_carry(), Ok(30_000));
    let mut rental = rental_page(1_000_000);
    rental.assets = vec![manual.into()];
    let mut session = restart(empty_return(FederalStatus::Single));
    accepted(&save_rentals(&mut session, vec![rental]));
    let (income_id, _, stored) = stored_rentals(&session).remove(0);
    let asset_id = stored.assets[0].id;
    assert!(asset_id > 0);
    assert_eq!(stored.assets[0].amt_section179_income_carry, Some(30_000));
    let mut rental = reopened_rentals(session.draft.clone()).remove(0).0;
    let page = &mut rental.assets[0];
    let mut records = direct_amt_carry_history();
    // Actual cumulative AMT1120000 minus other AMT160000 identifies
    // AMT179960000 and carry40000, without changing regular depreciation.
    records.years[0].section179_allowed = 1_000_000;
    records.carryovers[0].amount = 0;
    records.years[0].amt = 160_000;
    page.macrs_mut().unwrap().financial.history_details = Some(records.clone());
    let rental_energy_buildings =
        crate::source_associations::SourceAssociations::new(&session.draft)
            .physical_building_choices();
    let Reply::Response(P::CanonicalAssetAmtCarryChoices { allowed }) = send(
        &mut session,
        P::CanonicalAssetAmtCarryOptions {
            rental_energy_buildings,
            user: TEST_USER,
            financial: page.macrs().unwrap().financial.clone(),
        },
    ) else {
        panic!("ordinary source calculation")
    };
    assert!(allowed.is_empty());
    page.macrs_mut().unwrap().allowed_amt_carry = allowed;
    page.macrs_mut().unwrap().amt_section179_income_carry = None;
    let corrected: BusinessAsset = page.clone().into();
    assert!(corrected.valid());
    assert_eq!(corrected.amt_section179_income_carry, None);
    assert_eq!(corrected.amt_section179_carry(), Ok(40_000));
    assert_eq!(corrected.history_details.as_ref(), Some(&records));
    let mut forged = page.clone();
    forged.macrs_mut().unwrap().amt_section179_income_carry =
        AssetAmt179CarryAnswer::from_amount(Some(30_000));
    let forged: BusinessAsset = forged.into();
    assert!(!forged.valid());
    assert_eq!(forged.amt_section179_carry(), Err(()));
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(
            income_id,
            IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) },
        )],
    );
    accepted(&send(&mut session, request));
    let (_, _, stored) = stored_rentals(&session).remove(0);
    assert_eq!(stored.assets[0].id, asset_id);
    assert_eq!(stored.assets[0].amt_section179_income_carry, None);
    assert_eq!(stored.assets[0].amt_section179_carry(), Ok(40_000));
    assert_eq!(stored.assets[0].history_details.as_ref(), Some(&records));
    let reopened = reopened_rentals(session.draft.clone()).remove(0).0;
    assert_eq!(reopened.assets[0].macrs().unwrap().amt_section179_income_carry, None);
    assert_eq!(reopened.assets[0].macrs().unwrap().financial.amt_section179_carry(), Ok(40_000));
    assert_rentals_round_trip(&mut session, &[reopened]);
}
