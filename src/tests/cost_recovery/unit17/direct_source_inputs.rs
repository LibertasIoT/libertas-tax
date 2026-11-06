use super::*;
use crate::tests::business::business::{asset_sale_return, press, shop_building, sold};
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
        IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) },
        TaxSection::Farming,
    );
    let mut unsupported_vehicle = draft.clone();
    let FederalIncome::Business { data } = &mut unsupported_vehicle.income[0].income else {
        unreachable!()
    };
    let ListedProperty::Vehicle { kind, .. } = &mut data.assets[0].listed else { unreachable!() };
    *kind = VehicleKind::HeavyTruckVan;
    // 2026-10-18 approved R2: heavy vehicle design now supports section179.
    // The owned vehicle editor remains limited to five-year recovery; a
    // seven-year source still crosses the same named source boundary.
    data.assets[0].class = EquipmentClass::Seven;
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
            Err(Section179PreparedIssue::SourceBoundaryUnavailable(
                source,
                "ISSUE_SECTION179_WAGERING_OWN_DEDUCTION"
            )),
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
    add(&mut draft, income(cooperative_routed_income(200_000, 50_000)));
    claim_last_cooperative(&mut draft, business);
    add(&mut draft, income(cooperative_routed_income(200_000, 50_000)));
    claim_last_cooperative(&mut draft, farm);
    let mut grant = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut grant.income else { unreachable!() };
    data.form.taxable_grants = Some(250_000);
    data.grant_use = None;
    data.form.agriculture_payments = Some(400_000);

    add(&mut draft, grant);
    // The grant belongs to the business; both it and the farm are eligible receivers.
    let FederalIncome::GovernmentPayment { data } = &draft.income.last().unwrap().income else {
        unreachable!()
    };
    let grant_component = data.component_ids.grant;
    crate::tests::support::builders::claim_related_component(&mut draft, business, grant_component);
    let component = crate::tests::support::builders::farm_government_component(&draft);
    crate::tests::support::builders::relate_farm_component(&mut draft, farm, component);
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
    property.gds_mut().unwrap().history = AssetHistory::Continuing {
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
        IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
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
        let claims = crate::related_forms::effective_claimants(&draft);
        let parts = crate::section179::direct_income_parts_with_home_with_claims(
            &draft, system, None, None, &claims, None,
        )
        .unwrap();
        assert_eq!(parts[0].income, source);
        let parts = parts[0].property.as_ref().unwrap();
        let FederalIncome::Business { data } = &draft.income[0].income else { unreachable!() };
        let BusinessActivity::Farming { data: farm } = &data.activity else { unreachable!() };
        assert_eq!(parts.len(), 1);
        assert_eq!(
            parts[0].id,
            crate::asset_sale::DirectBusinessPropertyId::Land {
                sale: farm.land_sales[0].id,
                parcel: farm.land_sales[0].basis_property,
            }
        );
        assert_eq!(
            (
                parts[0].dollars.gain,
                parts[0].dollars.ordinary,
                parts[0].dollars.section1231,
                parts[0].dollars.unrecaptured
            ),
            (20_000, 16_000, 4_000, 0)
        );
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
    // 2026-10-23: a source-only professional wagering business now supplies
    // its owner-computed taxable income (Reg. 1.179-2(c)(6), IRC 165(d)).
    let sources =
        section179_direct_income_sources(&gambling, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(sources[0].source, gambling.income[1].id);
    assert_eq!(sources[0].operating_net_cents, 1_000_000);
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
    property.gds_mut().unwrap().history =
        AssetHistory::New { first_year: FirstYearDeduction::Regular };
    page.assets = vec![property];
    let supplies = save_native_asset_identities(
        supplies,
        source,
        IncomeSource::Record { value: crate::IncomeRecord::Farm(page) },
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
    let FederalIncome::GovernmentPayment { data } = &mut payment.income else { unreachable!() };
    data.form.agriculture_payments = Some(10_000);
    add(&mut receipts, payment);
    let government = receipts.clone();
    receipts.income.pop();
    let patronage = cooperative_routed_income(10_000, 0);
    add(&mut receipts, income(patronage));
    let recipient = receipts.clone();
    claim_last_cooperative(&mut receipts, farm);
    let FederalIncome::Cooperative { data } = &mut receipts.income[3].income else {
        unreachable!()
    };
    data.more.allocations[0].properties.push(CoopPropertyRefund {
        distribution: CoopDistributionBox::Patronage,
        property: None,
        source: "Original purchase record".into(),
        amount: 10_000,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: CoopPropertyTreatment::Purchase,
    });
    assert_eq!(data.more.allocations[0].ordinary(), 0);
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
}

#[test]
fn unit17_property_parts_keep_stable_identities_character_and_zero_sales() {
    use crate::asset_sale::DirectBusinessPropertyId as Id;
    let mut building = shop_building(AssetDisposal::No);
    building.partial =
        Some(PartialDisposition { date: 20260610, cost_of_part: 3_900_000, proceeds: 0 });
    let draft = asset_sale_return(vec![
        press(sold(20260715, 600_000, 20_000)),
        press(sold(20260715, 384_000, 0)),
        building,
        shop_building(sold(20260620, 42_000_000, 2_000_000)),
    ]);
    let source = draft.income[1].id;
    let mut page = business(15_000_000);
    page.assets = (0..4).map(|_| asset(1_000_000, 20260101, AssetClass::Five)).collect();
    let mut draft = save_native_asset_identities(
        draft,
        source,
        IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
        TaxSection::SelfEmployment,
    );
    let claims = crate::related_forms::effective_claimants(&draft);
    let FederalIncome::Business { data } = &draft.income[1].income else { unreachable!() };
    let ids: Vec<_> = data.assets.iter().map(|asset| asset.id).collect();
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let read = |draft: &FederalDraft| {
            crate::section179::direct_income_parts_with_home_with_claims(
                draft, system, None, None, &claims, None,
            )
            .unwrap()
            .remove(0)
        };
        let before = draft.clone();
        let actual = read(&draft);
        let parts = actual.property.unwrap();
        assert_eq!(
            parts.iter().map(|part| part.id).collect::<Vec<_>>(),
            vec![Id::Asset(ids[0]), Id::Asset(ids[1]), Id::Partial(ids[2]), Id::Asset(ids[3])]
        );
        assert_eq!(
            parts
                .iter()
                .map(|part| (
                    part.dollars.gain,
                    part.dollars.ordinary,
                    part.dollars.section1231,
                    part.dollars.unrecaptured
                ))
                .collect::<Vec<_>>(),
            vec![
                (1_960, 1_960, 0, 0),
                (0, 0, 0, 0),
                (-28_582, 0, -28_582, 0),
                (114_177, 0, 114_177, 104_177)
            ]
        );
        assert_eq!(actual.income.recognized_property_gain_dollars, 87_555);
        assert_eq!(draft, before);
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        data.assets.reverse();
        let mut reordered = read(&draft).property.unwrap();
        reordered.reverse();
        assert_eq!(parts, reordered);
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        data.assets.reverse();
    }
}

#[test]
fn unit17_property_parts_leave_unidentified_cost_dispositions_unavailable() {
    use crate::tests::support::builders::timber::{timber_business, timber_cost};
    let mut cost = timber_cost();
    cost.ended = Some(AmortizationEnd {
        date: 20261231,
        how: DisposalKind::Sold {
            proceeds: 3_000_000,
            selling_expenses: 0,
            payment: SalePayment::InFull,
            buyer: Buyer::Unrelated,
            livestock: crate::SoldLivestock::No,
        },
    });
    let draft = timber_business(cost);
    let claims = crate::related_forms::effective_claimants(&draft);
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let parts = crate::section179::direct_income_parts_with_home_with_claims(
            &draft, system, None, None, &claims, None,
        )
        .unwrap();
        assert_eq!(parts[0].property, Err("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED"));
        assert_eq!(
            section179_direct_income_sources(&draft, system).unwrap(),
            vec![parts[0].income]
        );
    }
}
