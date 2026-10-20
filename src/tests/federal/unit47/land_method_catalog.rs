use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::{
    assert::{catalog_unbounded, source_ids},
    builders::{cash_farm, farm_land as land, farm_only_return},
    client::save_farms,
};
use crate::tests::*;
use alloc::{collections::BTreeSet, format, vec::Vec};

// 2026-10-15 D13: remove former 100x400/100x10 capacity-fitting
// multiplicity. These multiple-parent source graphs still exceed the old
// derived 400/100 catalogs; all per-source figures and lifecycle checks remain.
#[test]
fn unit47_land_catalog_joins_two_saved_farms() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let farms = (0..2)
        .map(|i| {
            let mut farm = cash_farm(1_000_000, 0);
            farm.name = format!("Synthetic land source {i}");
            farm.more.land_basis = (0..201).map(|j| land(format!("Parcel {i}/{j}"))).collect();
            farm
        })
        .collect();
    save_farms(&mut session, farms);
    source_ids(&session);
    let saved = session.draft.clone();
    let actual = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes {
        basis_assets,
        cooperative_properties,
        documents,
        ..
    } = &actual
    else {
        unreachable!()
    };
    let source_land: Vec<_> = crate::farm::sources(&session.draft)
        .flat_map(|s| s.farm.land_basis.iter().map(|land| land.id))
        .collect();
    assert_eq!(source_land.len(), 402);
    assert!(source_land.iter().all(|id| *id > 0));
    assert_eq!(
        source_land.iter().copied().collect::<BTreeSet<_>>().len(),
        402
    );
    assert_eq!(
        basis_assets.iter().map(|c| c.id).collect::<Vec<_>>(),
        source_land
    );
    assert_eq!(
        cooperative_properties
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        source_land
    );
    assert_eq!(documents.documents.len(), 2);
    for row in &documents.documents {
        let IncomeDocument::Farm(farm) = &row.document else {
            unreachable!()
        };
        assert_eq!(farm.more.land_basis.len(), 201);
        assert!(
            farm.more
                .land_basis
                .iter()
                .all(|p| p.regular == 100_000 && p.amt == 100_000)
        );
    }
    accepted(&send(&mut session, actual.clone()));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, TaxSection::Farming);
    assert_eq!(
        crate::farm_specials::basis_choices(&cold.draft)
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        source_land
    );
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    catalog_unbounded(
        &["TaxInterviewProtocol", "SaveIncomes", "basis_assets"],
        "FarmChoice",
    );
    for request in [
        "SaveIncomes",
        "DocumentPropertyOptions",
        "PreviewDocument",
        "EstimateDocuments",
    ] {
        catalog_unbounded(
            &["TaxInterviewProtocol", request, "cooperative_properties"],
            "RecordChoice",
        );
    }
}

#[test]
fn unit47_method_catalog_joins_eleven_saved_farms() {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let farms = (0..11)
        .map(|i| {
            let mut farm = cash_farm(1_000_000, 0);
            farm.name = format!("Synthetic method source {i}");
            farm.more.method_changes = (0..10)
                .map(|_| MethodChange {
                    id: 0,
                    year: 2026,
                    consent: MethodChangeConsent::Automatic { dcn: 122 },
                    adjustment: Adjustment481::FourYears {
                        amount: 400_000,
                        rest_in_current_year: false,
                    },
                    amt: MethodChangeAmt::Same,
                })
                .collect();
            assert!(crate::method_change::list_issue(&farm.more.method_changes).is_none());
            farm
        })
        .collect();
    save_farms(&mut session, farms);
    source_ids(&session);
    let saved = session.draft.clone();
    let actual = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes {
        farm_method_changes,
        ..
    } = &actual
    else {
        unreachable!()
    };
    let methods: Vec<_> = crate::farm::sources(&session.draft)
        .flat_map(|s| {
            crate::method_change::records(&s.entry.income)
                .unwrap()
                .iter()
        })
        .collect();
    assert_eq!(methods.len(), 110);
    assert!(
        methods
            .iter()
            .all(|m| m.id > 0 && m.amount_current_year() == 100_000)
    );
    assert_eq!(
        methods.iter().map(|m| m.id).collect::<BTreeSet<_>>().len(),
        110
    );
    assert_eq!(
        farm_method_changes.iter().map(|c| c.id).collect::<Vec<_>>(),
        methods.iter().map(|m| m.id).collect::<Vec<_>>()
    );
    accepted(&send(&mut session, actual.clone()));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, TaxSection::Farming);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(crate::farm_specials::method_choices(&cold.draft).len(), 110);
    catalog_unbounded(
        &["TaxInterviewProtocol", "SaveIncomes", "farm_method_changes"],
        "FarmChoice",
    );
}
