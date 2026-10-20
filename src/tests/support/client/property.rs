use super::lifecycle::{restart, send};
use crate::{
    Asset, BusinessAsset, FederalDraft, FederalIncome, IncomeDocument, TaxSection, Vehicle,
    VehicleMethod,
};
use alloc::{vec, vec::Vec};

// Bind only new expected identities to the accepted native property records.
// The conversion stores ordinary assets before actual-expense vehicles; all
// other expected fields remain independently authored for full roundtrip checks.
pub(in crate::tests) fn bind_saved_property_ids(
    assets: &mut [Asset],
    vehicles: &mut [Vehicle],
    saved: &[BusinessAsset],
) {
    let actual_count = vehicles
        .iter()
        .filter(|vehicle| matches!(vehicle.method, VehicleMethod::Actual { .. }))
        .count();
    assert_eq!(saved.len(), assets.len() + actual_count);
    for (index, record) in saved.iter().enumerate() {
        assert!(record.id > 0);
        assert!(!saved[..index].iter().any(|earlier| earlier.id == record.id));
    }
    let bind = |expected: &mut i64, record: &BusinessAsset| {
        if *expected == 0 {
            *expected = record.id;
        } else {
            assert_eq!(*expected, record.id);
        }
    };
    let mut records = saved.iter();
    for asset in assets {
        let record = records.next().unwrap();
        assert!(!record.is_vehicle());
        bind(&mut asset.id, record);
    }
    for vehicle in vehicles {
        if let VehicleMethod::Actual { id, .. } = &mut vehicle.method {
            let record = records.next().unwrap();
            assert!(record.is_vehicle());
            bind(id, record);
        }
    }
    assert!(records.next().is_none());
}

// Save proves property identity only. Some native financial histories cannot
// roundtrip through an ordinary page; keep those original facts unchanged.
pub(in crate::tests) fn save_native_asset_identities(
    mut draft: FederalDraft,
    source: i64,
    page: IncomeDocument,
    topic: TaxSection,
) -> FederalDraft {
    let entry = draft.income.iter_mut().find(|entry| entry.id == source).unwrap();
    let owner = entry.owner;
    let community = entry.community;
    let refund_management = entry.refund_management;
    let FederalIncome::Business { data } = &mut entry.income else {
        panic!("a native business source")
    };
    let native = data.clone();
    data.assets.clear();
    let mut session = restart(draft);
    let mut document = crate::tests::cross::interview::document(source, page);
    document.owner = owner;
    document.community = community;
    document.refund_management = refund_management;
    let request = crate::tests::cross::interview::save_topic(&session.draft, topic, vec![document]);
    crate::tests::cross::interview::accepted(&send(&mut session, request));
    let entry = session.draft.income.iter_mut().find(|entry| entry.id == source).unwrap();
    assert_eq!(entry.owner, owner);
    assert_eq!(entry.community, community);
    assert_eq!(entry.refund_management, refund_management);
    let FederalIncome::Business { data } = &mut entry.income else {
        panic!("the same saved business source")
    };
    let ids: Vec<_> = data.assets.iter().map(|asset| asset.id).collect();
    assert_eq!(ids.len(), native.assets.len());
    for (index, id) in ids.iter().enumerate() {
        assert!(*id > 0 && !ids[..index].contains(id));
    }
    *data = native;
    for (asset, id) in data.assets.iter_mut().zip(ids) {
        assert!(asset.id == 0 || asset.id == id);
        asset.id = id;
    }
    assert!(session.draft.valid_stored());
    session.draft
}

// Match only the canonical mileage identities assigned by the native Save owner.
pub(in crate::tests) fn bind_saved_mileage_ids(
    vehicles: &mut [Vehicle],
    saved: &[crate::BusinessMileage],
) {
    let mut rows = saved.iter();
    for vehicle in vehicles {
        if let VehicleMethod::StandardMileage { id, .. } = &mut vehicle.method {
            let row = rows.next().unwrap();
            assert!(row.id > 0);
            if *id == 0 {
                *id = row.id;
            } else {
                assert_eq!(*id, row.id);
            }
        }
    }
    assert!(rows.next().is_none());
}
