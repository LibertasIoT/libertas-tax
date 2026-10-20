use super::*;
use crate::tests::cross::interview::{asset, business, cash_farm};

#[test]
fn unit17_farm_owned_vehicle_keeps_unsupported_and_gambling_source_boundaries() {
    let mut farm = cash_farm(2_000_000, 100_000);
    farm.vehicles = vec![owned_actual_vehicle(FirstYearDeduction::Section179 {
        description: "Farm delivery car".into(),
        elected: 200_000,
    })];
    let mut draft = empty_return(FederalStatus::Single);
    add(
        &mut draft,
        income(FederalIncome::Business { data: crate::farm_page::farm_record(&farm, None) }),
    );
    let source = draft.income[0].id;
    let draft = save_native_asset_identities(
        draft,
        source,
        IncomeDocument::Farm(farm),
        TaxSection::Farming,
    );
    let mut unsupported_vehicle = draft.clone();
    let FederalIncome::Business { data } = &mut unsupported_vehicle.income[0].income else {
        unreachable!()
    };
    let ListedProperty::Vehicle { passenger_automobile, .. } = &mut data.assets[0].listed else {
        unreachable!()
    };
    *passenger_automobile = false;
    let mut gambling = draft;
    let FederalIncome::Business { data } = &mut gambling.income[0].income else { unreachable!() };
    data.activity = BusinessActivity::Gambling {
        receipts: 2_000_000,
        returns: 0,
        accounting: BusinessAccounting::Cash,
        losses: 0,
        withholding: 0,
        eligible: true,
    };
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        assert_eq!(
            section179_direct_income_sources(&unsupported_vehicle, system),
            Err(Section179PreparedIssue::SourceBoundaryUnavailable(
                source,
                "ISSUE_CONFIRM_AN_OWNED_5_YEAR_VEHICLE_ON_SCHEDULE_C_WITH_ACTUAL_EXPENSES_93473DC7"
            )),
        );
        assert_eq!(
            section179_direct_income_sources(&gambling, system),
            Err(Section179PreparedIssue::SourceIncomeUnavailable(source)),
        );
    }
}

#[test]
fn unit17_direct_sources_keep_routed_receipts_on_their_actual_businesses() {
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, income(cash_business(10_000)));
    let business = draft.income[0].id;
    add(&mut draft, income(raised_farm(10_000, 3_000)));
    let farm = draft.income[1].id;
    add(&mut draft, income(cooperative_routed_income(business, 200_000, 50_000)));
    add(&mut draft, income(cooperative_routed_income(farm, 200_000, 50_000)));
    let mut grant = government_unemployment(0, &[], 0);
    let FederalIncome::Unemployment { data } = &mut grant.income else { unreachable!() };
    data.taxable_grants = 250_000;
    data.grant_use = Some(FederalGrantUse::Business { business: Some(business) });
    data.agriculture_payment = 400_000;
    data.agriculture_farm = Some(farm);
    add(&mut draft, grant);
    let raw = draft;
    let before = raw.clone();
    let mut notes = Vec::new();
    let mut draft = crate::unknowns::supported(&raw, &mut notes).unwrap_or_else(|| raw.clone());
    assert!(notes.is_empty(), "{notes:?}");
    let prepared = draft.clone();
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let sources = section179_direct_income_sources(&draft, system).unwrap();
        assert_eq!(sources.len(), 2);
        for (id, operating) in [(business, 1_400_000), (farm, 1_250_000)] {
            let source = sources.iter().find(|source| source.source == id).unwrap();
            assert_eq!(source.system, system);
            assert_eq!(source.operating_net_cents, operating);
            assert_eq!(source.recognized_property_gain_dollars, 0);
        }
    }
    draft.income.reverse();
    let sources =
        section179_direct_income_sources(&draft, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(
        sources.iter().find(|source| source.source == business).unwrap().operating_net_cents,
        1_400_000
    );
    draft.income.reverse();
    assert_eq!(draft, prepared);
    assert_eq!(raw, before);
}

#[test]
fn unit17_direct_sources_apply_selected_depreciation_once() {
    let draft = business_with_bonus(50_000, separate_amt_equipment());
    let mut page = business(5_000_000);
    let mut property = asset(1_000_000, 20170601, AssetClass::Ten);
    property.acquired = Some(20160101);
    property.history = AssetHistory::Continuing {
        prior_depreciation: 901_696,
        first_year_179: 0,
        first_year_bonus: 0,
        carryforward: 0,
        mid_quarter: false,
    };
    page.assets = vec![property];
    let source = draft.income[1].id;
    let draft = save_native_asset_identities(
        draft,
        source,
        IncomeDocument::Business(page),
        TaxSection::SelfEmployment,
    );
    let before = draft.clone();
    for (system, operating) in [
        (Section179IncomeSystem::Regular, 4_934_464),
        (Section179IncomeSystem::Alternative, 4_912_605),
    ] {
        let sources = section179_direct_income_sources(&draft, system).unwrap();
        assert_eq!(sources.len(), 1);
        let source = sources[0];
        assert_eq!(source.source, draft.income[1].id);
        assert_eq!(source.system, system);
        assert_eq!(source.operating_net_cents, operating);
        assert_eq!(source.recognized_property_gain_dollars, 0);
    }
    assert_eq!(draft, before);
}

#[test]
fn unit17_direct_sources_keep_recognized_property_dollars_before_character() {
    let mut draft = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(10_000, 3_000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    let mut sale = farm_land_sale(20210615, 20260616);
    sale.payment = SalePayment::Installment {
        assumed_debt: 0,
        payments: 7_500_000,
        interest: NoteInterest::Adequate,
        special: false,
    };
    set_land_sales(farm, vec![sale], &mut draft.next_id);
    add(&mut draft, income(entry));
    let id = draft.income[0].id;
    let before = draft.clone();
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let sources = section179_direct_income_sources(&draft, system).unwrap();
        assert_eq!(sources.len(), 1);
        let source = sources[0];
        assert_eq!(source.source, id);
        assert_eq!(source.operating_net_cents, 700_000);
        // The same native sale recognizes20,000 now:16,000 ordinary and4,000
        // section1231. Neither raw proceeds nor the latter class is the input.
        assert_eq!(source.recognized_property_gain_dollars, 20_000);
        let prepared = source.prepared_input().unwrap();
        assert_eq!(prepared.source, id);
        assert_eq!(prepared.system, system);
        assert_eq!(prepared.operating_net_cents, Some(700_000));
        assert_eq!(prepared.section1231_net_cents, Some(2_000_000));
    }
    assert_eq!(draft, before);
}

#[test]
fn unit17_direct_sources_preserve_applicable_unknown_source_boundaries() {
    let mut gambling = complete(FederalStatus::Single);
    let mut business = cash_business(10_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.activity = BusinessActivity::Gambling {
        receipts: 1_000_000,
        returns: 0,
        accounting: BusinessAccounting::Cash,
        losses: 0,
        withholding: 0,
        eligible: true,
    };
    add(&mut gambling, income(business));
    let before = gambling.clone();
    assert_eq!(
        section179_direct_income_sources(&gambling, Section179IncomeSystem::Regular),
        Err(Section179PreparedIssue::SourceIncomeUnavailable(gambling.income[1].id))
    );
    assert_eq!(gambling, before);
    let mut supplies = farm_supply_return(farm_supplies(20_000), 10_000, 70_000);
    let source = supplies.income[0].id;
    let FederalIncome::Business { data } = &mut supplies.income[0].income else { unreachable!() };
    data.assets.push(elected_asset(10_000, 5_000, BonusElection::ElectOut));
    // This representable page establishes identity only; the shared helper
    // restores the native election and supplies facts before the source read.
    let mut page = cash_farm(7_000_000, 1_000_000);
    let mut property = asset(data.assets[0].cost, data.assets[0].placed, AssetClass::Five);
    property.acquired = Some(data.assets[0].acquired);
    property.history = AssetHistory::New { first_year: FirstYearDeduction::Regular };
    page.assets = vec![property];
    let supplies = save_native_asset_identities(
        supplies,
        source,
        IncomeDocument::Farm(page),
        TaxSection::Farming,
    );
    let mut conservation = conservation_return();
    let FederalIncome::Business { data } = &mut conservation.income[1].income else {
        unreachable!()
    };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    let FarmAccounting::Cash { receipts } = &mut farm.accounting else { unreachable!() };
    receipts.push(FarmReceipt::Other { amount: 100_000 });
    let mut land = farmland_installment_return();
    let mut invalid_land = land.clone();
    let unrelated = invalid_land.next_id;
    farmland_basis_mut(&mut invalid_land).id = unrelated;
    invalid_land.next_id += 1;
    let before = invalid_land.clone();
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        assert_eq!(
            section179_direct_income_sources(&invalid_land, system),
            Err(Section179PreparedIssue::InvalidSource(
                invalid_land.income[1].id,
                "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26"
            ))
        );
    }
    assert_eq!(invalid_land, before);
    let FederalIncome::Business { data } = &mut land.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    // This retained basis/ownership source is structurally valid, but its
    // parcel basis is unavailable; operating receipts are not the missing fact.
    farm.specials.limitations.push(7);
    assert!(crate::federal_rules::validate_income(&land.income[1]).is_ok());
    let mut clergy = complete(FederalStatus::Single);
    let mut ministry = cash_business(10_000);
    let FederalIncome::Business { data } = &mut ministry else { unreachable!() };
    data.ministerial = true;
    add(&mut clergy, income(ministry));
    clergy.screening.as_mut().unwrap().clergy = vec![ClergyClaim {
        owner: Owner::Taxpayer,
        role: ClergyRole::Minister {
            housing: MinisterHousing::Parsonage {
                fair_rental_value: 1_000_000,
                utility_allowance: 0,
                utilities_paid: 0,
            },
            form_4361: false,
            employee_expenses: 0,
        },
    }];
    let mut receipts = complete(FederalStatus::Single);
    add(&mut receipts, income(raised_farm(10_000, 0)));
    add(&mut receipts, income(raised_farm(10_000, 0)));
    let farm = receipts.income[1].id;
    let mut payment = government_unemployment(0, &[], 0);
    let FederalIncome::Unemployment { data } = &mut payment.income else { unreachable!() };
    data.agriculture_payment = 10_000;
    add(&mut receipts, payment);
    let government = receipts.clone();
    receipts.income.pop();
    let mut patronage = cooperative_routed_income(farm, 10_000, 0);
    let FederalIncome::Cooperative { data } = &mut patronage else { unreachable!() };
    data.allocations[0].activity = None;
    add(&mut receipts, income(patronage));
    let recipient = receipts.clone();
    let FederalIncome::Cooperative { data } = &mut receipts.income[3].income else {
        unreachable!()
    };
    data.allocations[0].activity = Some(farm);
    data.allocations[0].properties.push(CoopPropertyRefund {
        distribution: CoopDistributionBox::Patronage,
        property: None,
        source: "Original purchase record".into(),
        amount: 10_000,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: CoopPropertyTreatment::Purchase,
    });
    assert_eq!(data.allocations[0].ordinary(), 0);
    for (draft, index, key) in [
        (
            supplies,
            0,
            "ISSUE_THE_PREPAID_CEILING_STILL_NEEDS_COORDINATION_WHEN_SEVERAL_FARMS_COM_960C7361",
        ),
        (conservation, 1, "WARN_FARM_CONSERVATION_GROSS_SOURCE"),
        (land, 1, "ISSUE_FARM_LAND_LINKED_BASIS_NEEDED"),
        (clergy, 1, "CLERGY_BEFORE179_EXPENSE_SOURCE"),
        (government, 3, "ISSUE_FORM_1099_G_BOX_7_AGRICULTURE_PAYMENTS_NEED_A_CHOSEN_FARM"),
        (recipient, 3, "ISSUE_COOP_ACTIVITY_NEEDED"),
        (receipts, 3, "ISSUE_COOP_PROPERTY_HISTORY_NEEDED"),
    ] {
        let before = draft.clone();
        let id = draft.income[index].id;
        for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
            assert_eq!(
                section179_direct_income_sources(&draft, system),
                Err(Section179PreparedIssue::SourceBoundaryUnavailable(id, key))
            );
        }
        assert_eq!(draft, before);
    }
}

#[test]
fn unit17_direct_sources_distinguish_context_known_zero_and_conversion_overflow() {
    let mut draft = complete(FederalStatus::Single);
    assert!(
        section179_direct_income_sources(&draft, Section179IncomeSystem::Regular)
            .unwrap()
            .is_empty()
    );
    add(&mut draft, income(cash_business(0)));
    let id = draft.income[1].id;
    let mut unrelated = farm_rental_data(0, 0);
    set_land_sales(
        &mut unrelated.farm,
        vec![farm_land_sale(20160616, 20260616)],
        &mut draft.next_id,
    );
    unrelated.farm.land_sales[0].situation = FarmLandSituation::RelatedParty;
    add(&mut draft, income(FederalIncome::FarmRental { data: unrelated }));
    let before = draft.clone();
    let sources =
        section179_direct_income_sources(&draft, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].source, id);
    assert_eq!(sources[0].operating_net_cents, 0);
    assert_eq!(sources[0].recognized_property_gain_dollars, 0);
    assert_eq!(draft, before);
    let overflow =
        Section179DirectIncomeSource { recognized_property_gain_dollars: i64::MAX, ..sources[0] };
    assert_eq!(overflow.prepared_input(), Err(Section179PreparedIssue::AmountOverflow(id)));
    let setup = draft.setup.clone();
    draft.income.push(draft.income[1].clone());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.notary_profit = -1;
    draft.setup = None;
    assert_eq!(
        section179_direct_income_sources(&draft, Section179IncomeSystem::Regular),
        Err(Section179PreparedIssue::MissingReturnContext)
    );
    draft.setup = setup;
    assert_eq!(
        section179_direct_income_sources(&draft, Section179IncomeSystem::Regular),
        Err(Section179PreparedIssue::UnknownSource(id))
    );
    draft.income.pop();
    assert_eq!(
        section179_direct_income_sources(&draft, Section179IncomeSystem::Regular),
        Err(Section179PreparedIssue::InvalidSource(
            id,
            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26"
        ))
    );
}
