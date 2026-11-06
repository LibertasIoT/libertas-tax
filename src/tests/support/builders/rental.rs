use crate::tests::*;

pub(in crate::tests) fn rental_casualty_source(kind: crate::CasualtyKind) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut property = rental(10_000, 0);
    property.buildings = vec![purchased_rental_building(6_600_000, 0, 20200101)];
    property.buildings[0].casualty = Some(crate::RentalCasualty {
        casualty: crate::PropertyCasualty {
            event: "Synthetic December storm".into(),
            date: 20261215,
            declaration: crate::CasualtyDeclaration::Undeclared,
            kind,
            reimbursement: 1_200_000,
            received: 1_200_000,
            replacement_cost: 0,
        },
        recurring: false,
        continuing_records: Default::default(),
    });
    add(&mut draft, income(FederalIncome::Rental { data: property }));
    draft
}

// Actual full-property source; no injected computed loss or hidden carriers.
pub(in crate::tests) fn recurring_rental_casualty(qualified: bool) -> FederalDraft {
    let mut draft = rental_casualty_source(crate::CasualtyKind::Damaged {
        before: 10_000_000,
        after: 1_000_000,
        retired: Some(20261215),
    });
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.rent = 5_000_000;
    data.expenses[0].amount = 50_000;
    data.buildings[0].casualty.as_mut().unwrap().recurring = true;
    data.qbi_business = if qualified {
        RentalBusiness::Yes { material_participation: false }
    } else {
        RentalBusiness::No
    };
    draft
}

pub(in crate::tests) fn rental_casualty_basis(
    basis: i64,
    depreciation: i64,
    prior_depreciation: i64,
) -> crate::RentalCasualtyBasis {
    crate::RentalCasualtyBasis {
        basis: crate::SourceCents::Reported { amount: basis },
        depreciation: crate::SourceCents::Reported { amount: depreciation },
        prior_depreciation: crate::SourceCents::Reported { amount: prior_depreciation },
    }
}

pub(in crate::tests) fn kept_rental_casualty() -> FederalDraft {
    let mut draft = recurring_rental_casualty(false);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let source = data.buildings[0].casualty.as_mut().unwrap();
    source.casualty.kind =
        crate::CasualtyKind::Damaged { before: 2_500_000, after: 1_500_000, retired: None };
    source.casualty.reimbursement = 0;
    source.casualty.received = 0;
    *source.continuing_records = crate::RentalCasualtyRecords {
        regular: rental_casualty_basis(4_940_000, 1_660_000, 1_430_000),
        alternative: rental_casualty_basis(4_940_000, 1_660_000, 1_430_000),
    };
    draft
}

// Pub527 Worksheet5-1: actual12,000FMV decline; equal rental/personal days.
// The kept source ledger establishes annual post-loss recovery of
// 39,700*24/517 =1,842.94, before the rental-day split and income limit.
pub(in crate::tests) fn standard_home_casualty() -> FederalDraft {
    let mut draft = kept_rental_casualty();
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.rent = 200_000;
    data.expenses[0].amount = 100_000;
    data.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 60,
        overlap_days: 0,
        direct_expenses: 0,
    };
    data.buildings[0].casualty.as_mut().unwrap().casualty.kind =
        CasualtyKind::Damaged { before: 2_500_000, after: 1_300_000, retired: None };
    draft
}

// A rehabilitation record: measuring period start, adjusted basis at its
// start, expenditures in it, the earlier credit years and the confirmation.
pub(in crate::tests) fn rehabilitation(
    period_start: u32,
    building_basis: i64,
    period_expenditures: i64,
    earlier: Vec<CreditYear>,
    confirmed: bool,
) -> RehabilitationCredit {
    RehabilitationCredit {
        period_start,
        phased: false,
        building_basis: building_basis * 100,
        period_expenditures: period_expenditures * 100,
        earlier,
        freed: 0,
        confirmed,
    }
}

pub(in crate::tests) fn rental(rent: i64, expense: i64) -> FederalRental {
    FederalRental {
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        location: crate::RentalLocation::UnitedStates,
        energy_buildings: Vec::new(),
        property_qbi_history: Vec::new(),
        related: Vec::new(),
        assets: alloc::vec::Vec::new(),
        rent: rent * 100,
        expenses: vec![crate::extended_model::RentalExpense {
            treatment: BusinessExpenseKind::Ordinary,
            category: crate::extended_model::RentalExpenseCategory::Other {
                label: "Ordinary expenses".into(),
            },
            amount: expense * 100,
        }],
        buildings: vec![purchased_rental_building(330_000 * 100, 55_000 * 100, 20200101)],
        participation: crate::RentalParticipation::Active,
        ordinary_rental: true,
        at_risk: Some(crate::RentalAtRisk::All),
        qbi_business: RentalBusiness::No,
        history: RentalHistory::NONE,
        method_changes: Vec::new(),
        payroll: BusinessPayroll::No,
        qbi_aggregation_group: None,
        personal_use: crate::extended_model::RentalPersonalUse::No,
        home_operating_carryover: 0,
        //2026-10-19: synthetic prior worksheets explicitly establish these independent records.
        prior_home_carry: None,
        property_type: RentalPropertyType::SingleFamily,
        sale: crate::RentalSale::No,
        statement: None,
        section179_business: None,
    }
}

pub(in crate::tests) fn royalty(receipts: i64, expenses: i64) -> FederalIncome {
    FederalIncome::Royalty {
        data: crate::FederalRoyalty {
            related: alloc::vec![],
            kind: crate::RoyaltyKind::Intangible,
            receipts: receipts * 100,
            advance: 0,
            expenses: if expenses == 0 {
                vec![]
            } else {
                vec![RentalExpense {
                    category: RentalExpenseCategory::LegalAndProfessional,
                    amount: expenses * 100,
                    treatment: BusinessExpenseKind::Ordinary,
                }]
            },
            depletion: crate::RoyaltyDepletion::No,
            ordinary: true,
        },
    }
}

pub(in crate::tests) fn personal_rent(receipts: i64, expenses: i64) -> FederalIncome {
    FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental {
            related: alloc::vec![],
            basis: 1000000,
            depreciable_basis: 1000000,
            assets: vec![],
            history: PersonalRentalHistory::NONE,
        },
        receipts: receipts * 100,
        expenses: expenses * 100,
        ordinary: true,
    }
}

pub(in crate::tests) fn personal_rent_with_history(
    receipts: i64,
    expenses: i64,
    regular: i64,
    amt: i64,
    investment: bool,
) -> FederalIncome {
    let mut rent = personal_rent(receipts, expenses);
    let FederalIncome::NonbusinessProperty { kind, .. } = &mut rent else { panic!() };
    *kind = NonbusinessPropertyKind::PersonalRental {
        related: alloc::vec![],
        basis: 1000000,
        depreciable_basis: if investment { 0 } else { 1000000 },
        assets: vec![],
        history: PersonalRentalHistory { regular: regular * 100, amt: amt * 100 },
    };
    rent
}

pub(in crate::tests) fn personal_rent_with_assets(
    receipts: i64,
    expenses: i64,
    assets: alloc::vec::Vec<BusinessAsset>,
) -> FederalIncome {
    let mut rent = personal_rent(receipts, expenses);
    let FederalIncome::NonbusinessProperty { kind, .. } = &mut rent else { unreachable!() };
    let basis = assets.iter().map(|a| a.cost).sum();
    *kind = NonbusinessPropertyKind::PersonalRental {
        related: alloc::vec![],
        basis,
        depreciable_basis: basis,
        assets,
        history: PersonalRentalHistory::NONE,
    };
    rent
}

pub(in crate::tests) fn rental_equipment(cost: i64, bonus: BonusElection) -> BusinessAsset {
    let mut asset = bonus_equipment(cost, 100);
    asset.history =
        EquipmentHistory::New { section179: Section179::Ineligible, bonus, tables: false };
    asset
}

pub(in crate::tests) fn separate_amt_equipment() -> BusinessAsset {
    // Used equipment acquired before the TCJA used-property bonus expansion.
    // Retained manual ten-year HY records: regular 200%, AMT 150%.
    let mut asset = continuing_equipment(20170601, 901696, false);
    asset.cost = 1000000;
    asset.acquired = 20160101;
    asset.class = EquipmentClass::Ten;
    if let EquipmentHistory::Continuing { amt, .. } = &mut asset.history {
        *amt = EquipmentAmtHistory::Separate {
            prior_depreciation: 868908,
            schedule: EquipmentSchedule::Manual,
        };
    }
    asset
}

pub(in crate::tests) fn large_separate_amt_business(receipts: i64, scale: i64) -> FederalDraft {
    let mut asset = separate_amt_equipment();
    asset.cost *= scale;
    if let EquipmentHistory::Continuing { prior_depreciation, amt, .. } = &mut asset.history {
        *prior_depreciation *= scale;
        if let EquipmentAmtHistory::Separate { prior_depreciation, .. } = amt {
            *prior_depreciation *= scale;
        }
    }
    business_with_bonus(receipts, asset)
}

pub(in crate::tests) fn personal_rental_owner_fixture() -> FederalDraft {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    let mut asset = rental_equipment(5000, BonusElection::ElectOut);
    asset.description = "Equipment rented to others".into();
    add(&mut d, income(personal_rent_with_assets(11_000, 2000, vec![asset])));
    let mut second = income(personal_rent_with_history(20_000, 4000, 3000, 5000, false));
    second.owner = crate::DocumentOwner::Spouses;
    add(&mut d, second);
    d
}

// Existing simplified rental facts, with the retained typed operating and
// property carries. Inputs are source cents, never reconstructed tax dollars.
pub(in crate::tests) fn simplified_rental_risk(
    basis_start: i64,
    increases: i64,
    decreases: i64,
    prior: [i64; 4],
) -> Option<crate::RentalAtRisk> {
    Some(crate::RentalAtRisk::Limited {
        record: Some(alloc::boxed::Box::new(crate::ActivityRiskRecord {
            regular: crate::RiskMethod::Simplified {
                facts: crate::RiskSimplified { basis_start, increases, decreases },
            },
            amt: crate::RiskAmtMethod::SameFactsAsRegular,
            financing: vec![],
            recapture: None,
        })),
        prior_disallowed: prior[0],
        amt_prior_disallowed: prior[1],
        prior_section1231_loss: prior[2],
        amt_prior_section1231_loss: prior[3],
    })
}

pub(in crate::tests) fn form4835_risk_property_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut rental = farm_rental_data(0, 0);
    // Existing building fixture: the $39,000 partial loss is $28,582;
    // remaining annual depreciation is $9,457.96 (business partial test).
    rental.farm.accounting =
        FarmAccounting::Cash { receipts: vec![FarmReceipt::Raised { amount: 945_796 }] };
    let mut building = continuing_asset(
        39_000_000,
        20160115,
        EquipmentClass::ThirtyNine,
        9_959_430,
        AssetDisposal::No,
    );
    building.id = d.next_id;
    d.next_id += 1;
    building.partial =
        Some(PartialDisposition { date: 20260610, cost_of_part: 3_900_000, proceeds: 0 });
    let mut short = depreciable_asset(1_000, EquipmentClass::Five, 20260101, 0);
    // 2026-10-16: asset_record writes eligible, zero elected for this
    // five-year business property. Form4835 still allows no section179.
    if let EquipmentHistory::New { section179, .. } = &mut short.history {
        *section179 = Section179::Eligible { elected: 0 };
    }
    short.id = d.next_id;
    d.next_id += 1;
    short.disposal = crate::tests::business::business::sold(20260715, 0, 0);
    rental.assets = vec![building, short];
    let mut long_land = farm_land_sale(20200101, 20260715);
    long_land.basis = 1_000_000;
    long_land.proceeds = 500_000;
    long_land.selling = 0;
    long_land.conservation = 0;
    long_land.amt = FarmLandAmt::Different { basis: 800_000, conservation: 0 };
    let mut short_land = farm_land_sale(20260101, 20260715);
    short_land.basis = 250_000;
    short_land.proceeds = 50_000;
    short_land.selling = 0;
    short_land.conservation = 0;
    set_land_sales(&mut rental.farm, vec![long_land, short_land], &mut d.next_id);
    rental.at_risk = simplified_rental_risk(1_829_100, 0, 0, [0; 4]);
    let Some(RentalAtRisk::Limited { record, .. }) = &mut rental.at_risk else { unreachable!() };
    record.as_mut().unwrap().amt = RiskAmtMethod::Separate {
        facts: alloc::boxed::Box::new(RiskMethod::Simplified {
            facts: RiskSimplified { basis_start: 1_729_100, increases: 0, decreases: 0 },
        }),
    };
    rental.qbi_business = RentalBusiness::Yes { material_participation: false };
    add(&mut d, income(FederalIncome::FarmRental { data: rental }));
    d
}

pub(in crate::tests) fn compare_rental_risk_record() -> ActivityRiskRecord {
    ActivityRiskRecord {
        regular: RiskMethod::Compare {
            simplified: RiskSimplified { basis_start: 20_000, increases: 0, decreases: 0 },
            detailed: RiskDetailed {
                opening: RiskDetailedOpening::PriorDetailed {
                    prior: PriorRiskBalance { year: 2025, regular: 10_000, amt: 10_000 },
                },
                increases: 0,
                decreases: 40_000,
                depletion: vec![],
            },
        },
        amt: RiskAmtMethod::SameFactsAsRegular,
        financing: vec![],
        recapture: Some(RiskRecaptureHistory {
            regular: RiskRecaptureAmounts { deducted_post1978: 100_000, recaptured_before: 0 },
            amt: RiskRecaptureAmounts { deducted_post1978: 100_000, recaptured_before: 0 },
            floor_1978: 0,
        }),
    }
}

// An actual answered No without Form6198 method/history facts.
pub(in crate::tests) fn rental_risk_without_method() -> Option<RentalAtRisk> {
    Some(RentalAtRisk::Limited {
        record: None,
        prior_disallowed: 0,
        amt_prior_disallowed: 0,
        prior_section1231_loss: 0,
        amt_prior_section1231_loss: 0,
    })
}

pub(in crate::tests) fn form4835_risk_prior_return(
    capacity: i64,
    operating_prior: i64,
    property_prior: i64,
    current_operating_loss: bool,
) -> FederalDraft {
    let mut draft = form4835_risk_property_return();
    let FederalIncome::FarmRental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    if current_operating_loss {
        data.farm.accounting = FarmAccounting::Cash { receipts: vec![] };
    }
    data.at_risk = simplified_rental_risk(
        capacity,
        0,
        0,
        [operating_prior, operating_prior, property_prior, property_prior],
    );
    draft
}

pub(in crate::tests) fn purchased_rental_building(
    cost: i64,
    land: i64,
    in_service: u32,
) -> RentalBuilding {
    RentalBuilding {
        commercial_1993_transition: None,
        casualty: None,
        casualty_property: None,
        id: 0,
        energy: None,
        sale_history: None,
        cost,
        land,
        in_service,
        credit_reduction: 0,
        acquired: crate::RentalAcquisition::Purchased,
        rehabilitation: None,
        partial: None,
    }
}

// Independent sale controls: $660,000 / 27.5 = $24,000 annual recovery.
// Jan2020–midAug2026 recovery is $23,000 + 5*$24,000 + $15,000;
// adjusted basis $502,000, while $15,000 rent covers current depreciation.
pub(in crate::tests) fn qualified_rental_sale(price: i64, lookback: Option<i64>) -> FederalDraft {
    let mut property = rental(15_000, 0);
    property.buildings = vec![purchased_rental_building(66_000_000, 0, 20200101)];
    property.qbi_business = RentalBusiness::Yes { material_participation: false };
    property.payroll = BusinessPayroll::Yes {
        statements: vec![PayrollW2 {
            label: "Synthetic rental payroll".into(),
            wages: 200_000,
            medicare: 200_000,
        }],
        attribution: PayrollQbiAttribution::EntireBusiness,
    };
    property.sale = RentalSale::Sold {
        date: 20260815,
        price: price * 100,
        land_price: 0,
        selling_expenses: 0,
        payment: SalePayment::InFull,
        buyer: Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Rental { data: property }));
    draft.screening.as_mut().unwrap().section1231 = lookback.map(|loss| {
        if loss == 0 {
            section1231_history(&[], &[])
        } else {
            section1231_history(&[(2025, loss, loss)], &[(2025, loss, loss)])
        }
    });
    draft
}

// All disposal variants share the actual rental/asset identity setup.
fn qualified_rental_with_asset(mut asset: BusinessAsset) -> FederalDraft {
    let mut draft = qualified_rental_sale(522_000, Some(0));
    asset.id = draft.next_id;
    draft.next_id += 1;
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.assets.push(asset);
    draft
}

// Disposal of an actual fully recovered asset, plus the independent building.
pub(in crate::tests) fn qualified_rental_with_recapture() -> FederalDraft {
    let mut asset = continuing_equipment(20160101, 100_000, false);
    asset.acquired = 20160101;
    asset.description = "Synthetic fully recovered rental equipment".into();
    asset.disposal = crate::tests::business::business::sold(20260815, 100_000, 0);
    qualified_rental_with_asset(asset)
}

pub(in crate::tests) fn qualified_rental_with_amt_recapture() -> FederalDraft {
    let mut asset = separate_amt_equipment();
    asset.description = "Synthetic rental equipment with separate AMT history".into();
    asset.disposal = crate::tests::business::business::sold(20260815, 100_000, 0);
    qualified_rental_with_asset(asset)
}

pub(in crate::tests) fn qualified_rental_with_passive_asset_loss() -> FederalDraft {
    let mut asset = bonus_equipment(1_000, 100);
    asset.acquired = 20260101;
    asset.placed = 20260101;
    asset.description = "Synthetic short-held rental equipment".into();
    asset.disposal = crate::tests::business::business::sold(20260815, 50_000, 0);
    let mut draft = qualified_rental_with_asset(asset);
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.sale = RentalSale::No;
    data.rent = 2_400_000;
    data.participation = RentalParticipation::NotActive;
    draft
}

pub(in crate::tests) fn qualified_rental_with_personal_asset_gain() -> FederalDraft {
    let mut asset = continuing_equipment(20160101, 50_000, false);
    asset.acquired = 20160101;
    asset.description = "Synthetic equipment used half for rental and half personally".into();
    asset.listed = ListedProperty::No { business_percent: 50 };
    // The source asks only about a fall from above50%; this asset was
    // always50% business, so that separate change question is null.
    asset.earlier_business_percent = None;
    asset.disposal = crate::tests::business::business::sold(20260815, 200_000, 0);
    qualified_rental_with_asset(asset)
}

// Section1033 award2,000 less replacement1,500 recognizes500; the
// short-held building's realized gain also includes allowed depreciation.
pub(in crate::tests) fn qualified_condemned_short_rental() -> FederalDraft {
    let mut draft = qualified_rental_sale(0, Some(0));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.buildings = vec![purchased_rental_building(100_000, 0, 20260101)];
    data.rent = 0;
    data.sale = RentalSale::Condemned {
        date: 20260815,
        award: 200_000,
        land_award: 0,
        expenses: 0,
        replacement_cost: 150_000,
    };
    draft
}

// Prior-year five-year equipment was fully expensed; personal conversion
// uses20% prior MACRS plus half of32% current MACRS on that deduction.
pub(in crate::tests) fn qualified_rental_with_section179_conversion() -> FederalDraft {
    let mut asset = continuing_equipment(20250101, 100_000, false);
    asset.acquired = 20250101;
    asset.description = "Synthetic expensed rental equipment converted personally".into();
    if let EquipmentHistory::Continuing { initial_section179, .. } = &mut asset.history {
        *initial_section179 = 100_000;
    }
    asset.disposal =
        crate::AssetDisposal::Disposed { date: 20260715, how: crate::DisposalKind::PersonalUse };
    let mut draft = qualified_rental_with_asset(asset);
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.sale = RentalSale::No;
    data.rent = 2_400_000;
    draft
}

// Actual records for a single building, excluding land and disposed components.
pub(in crate::tests) fn rental_sale_history(
    basis: i64,
    regular: i64,
    amt: i64,
) -> crate::RentalBuildingSaleHistory {
    crate::RentalBuildingSaleHistory {
        basis: crate::SourceCents::Reported { amount: basis },
        depreciation: crate::SourceCents::Reported { amount: regular },
        amt_depreciation: crate::SourceCents::Reported { amount: amt },
        additional: crate::SourceCents::NotReported,
        amt_additional: crate::SourceCents::NotReported,
    }
}

pub(in crate::tests) fn rental_property_loss_year(
    year: u32,
    original: i64,
    qualified: i64,
    remaining: i64,
    remaining_qualified: i64,
) -> crate::RentalPropertyLossYear {
    crate::RentalPropertyLossYear {
        year,
        original,
        remaining,
        qualified: crate::SourceCents::Reported { amount: qualified },
        remaining_qualified: crate::SourceCents::Reported { amount: remaining_qualified },
    }
}

pub(in crate::tests) fn rental_with_prior_property_history(qualified_now: bool) -> FederalDraft {
    let mut draft = qualified_rental_sale(522_000, Some(0));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.sale = RentalSale::No;
    data.rent = 2_400_000;
    data.qbi_business = if qualified_now {
        RentalBusiness::Yes { material_participation: false }
    } else {
        RentalBusiness::No
    };
    data.at_risk = simplified_rental_risk(500_000, 0, 0, [0, 0, 1_000_000, 800_000]);
    // Actual opening worksheet: pre-2018 $3k; later $7k with original60%QBI.
    // Reversed entry order tests chronological FIFO, not list order.
    data.property_qbi_history = vec![
        rental_property_loss_year(2019, 1_000_000, 600_000, 700_000, 420_000),
        rental_property_loss_year(2017, 300_000, 0, 300_000, 0),
    ];
    draft
}

pub(in crate::tests) fn mixed_rental_with_actual_sale_records() -> FederalDraft {
    let mut draft = qualified_rental_sale(522_000, Some(19_800));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    let RentalSale::Sold { personal_share_percent, buyer, .. } = &mut data.sale else {
        unreachable!()
    };
    *personal_share_percent = 10;
    *buyer = Buyer::Unrelated;
    // Actual business cost550k, regular depreciation100k, AMT80k.
    // These differ from today's90%cost and a full-use schedule.
    data.buildings[0].sale_history = Some(rental_sale_history(55_000_000, 10_000_000, 8_000_000));
    draft
}

pub(in crate::tests) fn sold_rental_with_property_loss_history(
    year: u32,
    qualified: i64,
    lookback: i64,
) -> FederalDraft {
    let mut draft = qualified_rental_sale(522_000, Some(lookback));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.at_risk = simplified_rental_risk(60_000_000, 0, 0, [0, 0, 100_000, 100_000]);
    data.property_qbi_history =
        vec![rental_property_loss_year(year, 100_000, qualified, 100_000, qualified)];
    draft
}

pub(in crate::tests) fn community_rental_property_history(owner: Owner) -> FederalDraft {
    let mut draft = if owner == Owner::Taxpayer {
        community_w2_draft(6_000_000, 0, 0, 0)
    } else {
        community_w2_draft(0, 6_000_000, 0, 0)
    };
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut source = rental(0, 0);
    source.buildings[0].land = source.buildings[0].cost;
    source.at_risk = simplified_rental_risk(100_000, 0, 0, [0, 0, 2, 2]);
    source.property_qbi_history = vec![
        rental_property_loss_year(2020, 1, 1, 1, 1),
        rental_property_loss_year(2021, 1, 0, 1, 0),
    ];
    let mut entry = income(FederalIncome::Rental { data: source });
    entry.owner = crate::DocumentOwner::from(owner);
    entry.community = CommunityShare::Community;
    add(&mut draft, entry);
    draft
}

pub(in crate::tests) fn transition_binding() -> Commercial1993Transition {
    Commercial1993Transition::BindingContract(Commercial1993ContractSource {
        effective: 19930512,
        source_reference: "Actual binding purchase contract".into(),
    })
}

pub(in crate::tests) fn transition_construction() -> Commercial1993Transition {
    Commercial1993Transition::ConstructionBegun(Commercial1993ConstructionSource {
        began: 19930512,
        source_reference: "Actual construction commencement record".into(),
    })
}

pub(in crate::tests) fn transition_source_page(source: Commercial1993Transition) -> Rental {
    let mut page = crate::tests::cross::interview::rental_page(4_800_000);
    page.kind = RentalPropertyType::Commercial;
    page.building.cost = 33_000_000;
    page.building.land = 5_500_000;
    page.building.in_service = 19930701;
    page.building.commercial_1993_transition = Some(source);
    page.building.allowed_commercial_1993_transition = crate::rental_recovery::allowed(
        &page.kind,
        &crate::RentalLocation::UnitedStates,
        page.building.in_service,
    );
    page.more.qbi = RentalBusiness::Yes { material_participation: true };
    page
}

pub(in crate::tests) fn transition_source_record(
    source: Commercial1993Transition,
) -> FederalRental {
    let FederalIncome::Rental { data } = crate::interview::document_income(&IncomeSource::Record {
        value: crate::IncomeRecord::Rental(transition_source_page(source)),
    }) else {
        unreachable!()
    };
    data
}

pub(in crate::tests) fn rental_energy_building() -> crate::RentalEnergyBuildingFacts {
    crate::RentalEnergyBuildingFacts {
        recovery: None,
        id: 0,
        building_source: "Certification EB-01, building A".into(),
        square_feet: 10000.0,
        ownership_numerator: 1,
        ownership_denominator: 1,
        deductions: (2023..=2026)
            .map(|tax_year| crate::RentalEnergyDeductionSource {
                tax_year,
                claimant_source: alloc::format!("Building A all-party records {tax_year}"),
                amount: SourceCents::NotReported,
            })
            .collect(),
    }
}
pub(in crate::tests) fn rental_energy_claim(physical_building: i64) -> crate::RentalEnergyClaim {
    crate::RentalEnergyClaim {
        physical_building,
        project_source: "EB-01 envelope invoice group".into(),
        system: crate::RentalEnergySystem::BuildingEnvelope,
        construction_began: 20260101,
        installation_began: 20260201,
        qualifying_cost: 2_000_000,
        project: crate::RentalEnergyProject::Eecbp {
            certification: Some(crate::RentalEnergyCertification {
                source_reference: "EB-01 signed certification".into(),
                certifier: "Synthetic Engineer".into(),
                certified: 20261215,
                license_source: "Synthetic State PE-101".into(),
                unrelated_to_claimant: true,
                calculation_source: "90.1-2007; qualified-software declaration EB-01".into(),
                savings_percent: 35.0,
            }),
        },
        increased: crate::RentalEnergyIncreasedAmount::Ordinary,
    }
}
pub(in crate::tests) fn rental_energy_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut property = rental(20_000, 2000);
    property.property_type = RentalPropertyType::Commercial;
    property.buildings = vec![purchased_rental_building(10_000_000, 0, 20261201)];
    property.energy_buildings = vec![rental_energy_building()];
    property.qbi_business = RentalBusiness::Yes { material_participation: true };
    add(&mut d, income(FederalIncome::Rental { data: property }));
    let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings[0].energy = Some(rental_energy_claim(data.energy_buildings[0].id));
    d
}

pub(in crate::tests) fn foreign_rental(placed: u32, kind: RentalPropertyType) -> FederalRental {
    let mut source = rental(48_000, 6_000);
    source.location = crate::RentalLocation::Foreign { country: "FR".into() };
    source.property_type = kind;
    source.buildings = vec![purchased_rental_building(36_000_000, 0, placed)];
    source
}

pub(in crate::tests) fn rental_source_view(
    draft: &FederalDraft,
    magi: i64,
    alternative: bool,
) -> crate::extended_rules::RentalResult {
    crate::extended_rules::rentals_for_home_with_care(
        draft,
        magi,
        &mut FederalResult::empty(draft.revision),
        alternative,
        [magi, magi],
        &crate::care::CareDeductions::default(),
    )
}

// A kept rental casualty whose building is a saved personal casualty property
// of the same storm: (return, rental ID, building cost ID, event ID, property ID).
pub(in crate::tests) fn shared_casualty_return() -> (FederalDraft, i64, i64, i64, i64) {
    let mut draft = kept_rental_casualty();
    let mut event = personal_disaster("Synthetic December storm", 500_000);
    event.declaration = CasualtyDeclaration::Undeclared;
    event.zip = "".into();
    event.date = 20261215;
    event.properties[0].label = "Synthetic vacation home".into();
    event.id = draft.next_id;
    event.properties[0].id = draft.next_id + 1;
    draft.next_id += 2;
    let ids = (event.id, event.properties[0].id);
    draft.deductions.as_mut().unwrap().personal_casualties.push(event);
    let rental = draft.income.last().unwrap().id;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let building = &mut data.buildings[0];
    if building.id == 0 {
        building.id = draft.next_id;
        draft.next_id += 1;
    }
    let source = &building.casualty.as_ref().unwrap().casualty;
    assert_eq!((source.date, &source.declaration), (20261215, &CasualtyDeclaration::Undeclared));
    building.casualty_property = Some(ids.1);
    let cost = building.id;
    (draft, rental, cost, ids.0, ids.1)
}

// One canonical whole dwelling and its actual damaged cost source. The
// ordinary event loses $12,000 before one $100 floor; the qualified event
// instead uses one $500 floor. Both have equal personal/rental days.
pub(in crate::tests) fn bound_home_casualty(qualified: bool) -> (FederalDraft, i64, i64) {
    let mut draft = standard_home_casualty();
    let mut event = personal_disaster("Synthetic December storm", 4_940_000);
    event.id = draft.next_id;
    event.properties[0].id = draft.next_id + 1;
    draft.next_id += 2;
    event.date = 20261215;
    event.declaration = if qualified {
        CasualtyDeclaration::MajorDisaster { reference: "DR-1234".into() }
    } else {
        CasualtyDeclaration::Federal { reference: "EM-1234".into() }
    };
    event.properties[0].before = 2_500_000;
    event.properties[0].after = 1_300_000;
    let property = event.properties[0].id;
    let rental = draft.income.last().unwrap().id;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    // Pub527 Worksheet5-1 line2d: this $500 is directly rental-only, so
    // line2e may exceed rent. Ordinary tier4 expenses would instead carry.
    data.expenses[0].amount = 0;
    let RentalPersonalUse::Yes { direct_expenses, .. } = &mut data.personal_use else {
        unreachable!()
    };
    *direct_expenses = 50_000;
    let cost = &mut data.buildings[0];
    if cost.id == 0 {
        cost.id = draft.next_id;
        draft.next_id += 1;
    }
    cost.casualty_property = Some(property);
    cost.casualty.as_mut().unwrap().casualty.declaration = event.declaration.clone();
    draft.deductions.as_mut().unwrap().personal_casualties.push(event);
    (draft, rental, property)
}

pub(in crate::tests) fn duplicate_bound_home_casualty_cost(draft: &mut FederalDraft) {
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let mut second = data.buildings[0].clone();
    second.id = draft.next_id;
    draft.next_id += 1;
    data.buildings.push(second);
}

pub(in crate::tests) fn split_bound_home_casualty_cost(draft: &mut FederalDraft) {
    duplicate_bound_home_casualty_cost(draft);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    for cost in &mut data.buildings {
        cost.cost /= 2;
        let records = cost.casualty.as_mut().unwrap();
        records.casualty.kind =
            CasualtyKind::Damaged { before: 1_250_000, after: 650_000, retired: None };
        records.continuing_records.regular = rental_casualty_basis(2_470_000, 830_000, 715_000);
        records.continuing_records.alternative = records.continuing_records.regular.clone();
    }
}

// Test fixtures resolve the complete accepted return before inspecting casualty sources.
pub(in crate::tests) fn accepted_rental_casualty_items(
    draft: &FederalDraft,
) -> Vec<crate::casualty::rental::LocatedRentalCasualty> {
    let claims = crate::related_forms::effective_claimants(draft);
    let result = estimate(draft);
    crate::casualty::rental::items(
        crate::related_forms::RelatedFormContext::original(draft, &claims),
        result.rental_home_context.as_ref(),
    )
}
