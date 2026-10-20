use crate::tests::*;

// Exact source facts for legacy fixtures that identify a personal-purchase
// part. Actual qualification exclusions are documented zero in these cases.
pub(in crate::tests) fn cooperative_personal(data: &mut FederalCooperative, amount: i64) {
    if data.allocations.is_empty() {
        data.allocations.push(crate::cooperative::automatic(data, None));
    }
    data.allocations[0].personal.patronage = amount;
    data.allocations[0].personal_qualification = Some(CoopQualification::default());
}

pub(in crate::tests) fn cooperative_routed_income(
    activity: i64,
    patronage: i64,
    personal: i64,
) -> FederalIncome {
    let mut data = FederalCooperative { patronage, ..Default::default() };
    data.allocations.push(crate::cooperative::automatic(&data, Some(activity)));
    cooperative_personal(&mut data, personal);
    FederalIncome::Cooperative { data }
}

// Accepted synthetic equipment and actual cooperative allocations. Every
// relationship uses the same durable source IDs as the normal association view.
pub(in crate::tests) fn cooperative_asset_refunds(
    asset: BusinessAsset,
    refunds: &[[i64; 4]],
) -> (FederalDraft, i64, i64) {
    cooperative_asset_refunds_on(asset, refunds, cash_business(100_000))
}

pub(in crate::tests) fn cooperative_asset_refunds_on(
    mut asset: BusinessAsset,
    refunds: &[[i64; 4]],
    activity_source: FederalIncome,
) -> (FederalDraft, i64, i64) {
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, income(activity_source));
    let activity = draft.income[0].id;
    asset.id = draft.next_id;
    draft.next_id += 1;
    let property = asset.id;
    match &mut draft.income[0].income {
        FederalIncome::Business { data } => data.assets.push(asset),
        FederalIncome::FarmRental { data } => data.assets.push(asset),
        _ => unreachable!(),
    }
    for &[amount, qualified_payments, qualified_items, sstb_items] in refunds {
        let mut data = FederalCooperative {
            patronage: amount,
            qualified_payments,
            qualified_items,
            sstb_items,
            ..Default::default()
        };
        let mut allocation = crate::cooperative::automatic(&data, Some(activity));
        allocation.properties.push(CoopPropertyRefund {
            distribution: CoopDistributionBox::Patronage,
            property: Some(property),
            source: "Synthetic accepted equipment purchase invoice".into(),
            amount,
            qualified_payments,
            qualified_items,
            sstb_items,
            treatment: CoopPropertyTreatment::Purchase,
        });
        data.allocations.push(allocation);
        add(&mut draft, income(FederalIncome::Cooperative { data }));
    }
    (draft, activity, property)
}

// Existing accepted parcel supplies the native cross-kind identity control;
// no conservation election competes with the purchase-refund receipt case.
pub(in crate::tests) fn cooperative_equipment_identity_return() -> FederalDraft {
    let mut draft = conservation_return();
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    draft
}

// Existing community owner keeps C/F whole for operator SE and projects4835
// records; these are actual MFS setup facts, never an inferred asset ratio.
pub(in crate::tests) fn cooperative_community_mfs(draft: FederalDraft) -> FederalDraft {
    let mut separate = empty_return(FederalStatus::Separate);
    separate.income = draft.income;
    separate.next_id = draft.next_id;
    let setup = separate.setup.as_mut().unwrap();
    setup.community_property = community_arizona();
    setup.status = FilingChoice::Separate {
        section179: Section179Spouse::Records {
            cost: 0,
            allocation: Section179Share::Equal,
            eligible: true,
        },
    };
    for entry in &mut separate.income {
        entry.community = CommunityShare::Community;
    }
    separate
}

pub(in crate::tests) fn cooperative_partial_building() -> BusinessAsset {
    let mut asset = continuing_equipment(20240601, 2_000_000, false);
    asset.cost = 39_000_000;
    asset.class = EquipmentClass::ThirtyNine;
    asset.method = EquipmentMethod::StraightLine;
    asset.partial =
        Some(PartialDisposition { date: 20260610, cost_of_part: 3_900_000, proceeds: 0 });
    asset
}

// A 2026 return whose continuing equipment and refund both enter through
// normal editors. IDs come exclusively from accepted native Saves.
pub(in crate::tests) fn continuing_purchase_refund_session() -> (Interview, i64, i64) {
    use crate::tests::cross::interview::{asset, business, document, open_topic};
    let mut session = restart(complete(FederalStatus::Single));
    let mut request = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let mut page = business(1_000_000);
    let mut equipment = asset(100_000, 20240601, AssetClass::Five);
    equipment.history = AssetHistory::Continuing {
        prior_depreciation: 52_000,
        first_year_179: 0,
        first_year_bonus: 0,
        carryforward: 0,
        mid_quarter: false,
    };
    page.assets.push(equipment);
    documents.documents.push(document(0, IncomeDocument::Business(page)));
    accepted(&send(&mut session, request));
    let activity = session.draft.income.last().unwrap().id;
    let FederalIncome::Business { data } = &session.draft.income.last().unwrap().income else {
        unreachable!()
    };
    let property = data.assets[0].id;
    let mut request = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, cooperative_properties, .. } = &mut request else {
        unreachable!()
    };
    assert!(cooperative_properties.iter().any(|choice| choice.id == property));
    let mut refund = crate::tests::farm::cooperative::patr_page(30_000, 0, Some(activity));
    refund.withholding = 0;
    refund.more.allocations[0].properties.push(CoopPropertyRefundInput {
        distribution: CoopDistributionBox::Patronage,
        property: Some(property),
        source: "Synthetic continuing equipment purchase invoice".into(),
        amount: 30_000,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: CoopPropertyTreatment::Purchase,
    });
    documents.documents.push(document(0, IncomeDocument::Patr1099(refund)));
    accepted(&send(&mut session, request));
    (session, activity, property)
}
