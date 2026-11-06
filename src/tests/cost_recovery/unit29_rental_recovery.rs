// Ordinary 1987–May12,1993 nonresidential GDS source fitting. Source values,
// gain basis, original clocks and UBIA remain with their existing owners.
use super::*;
use crate::tests::cross::interview::{
    assert_rentals_round_trip, rental_page, save_rentals, stored_rentals,
};

fn commercial(placed: u32) -> FederalRental {
    let mut v = rental(48_000, 6_000);
    v.property_type = RentalPropertyType::Commercial;
    v.buildings[0].in_service = placed;
    v.qbi_business = RentalBusiness::Yes { material_participation: true };
    v
}

#[test]
fn unit29_rental_historical_315_annual_amt_and_gds_ubia_share_original_clock() {
    for placed in [19870101, 19900101, 19930512] {
        let v = commercial(placed);
        let b = &v.buildings[0];
        // $275000 building basis: all31.5years have expired by2026. The
        // original pre1999 AMT40year schedule still allows275000/40=$6875.
        assert_eq!(
            crate::extended_rules::depreciation_for_year(
                b,
                false,
                (&v.property_type, &crate::RentalLocation::UnitedStates),
                2026,
                v.sale.date(),
                crate::kernel::source_fact::SourceFact::NotReported
            ),
            Ok(0)
        );
        assert_eq!(
            crate::extended_rules::depreciation_for_year(
                b,
                true,
                (&v.property_type, &crate::RentalLocation::UnitedStates),
                2026,
                v.sale.date(),
                crate::kernel::source_fact::SourceFact::NotReported
            ),
            Ok(687_500)
        );
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Rental { data: v }));
        let mut result = FederalResult::empty(0);
        let pass = crate::extended_rules::rentals(&d, 60_000, &mut result);
        assert_eq!(pass.activities.len(), 1);
        assert_eq!(pass.activities[0].property, 0);
        assert_eq!(value(&supported(&d), "agi"), 102_000);
    }
    // An actual current improvement keeps its own clock and UBIA; it
    // cannot restart the exhausted original building or inherit its period.
    let mut mixed = commercial(19900101);
    mixed.buildings.push(purchased_rental_building(1_000_000, 0, 20200101));
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Rental { data: mixed }));
    let mut result = FederalResult::empty(0);
    assert_eq!(
        crate::extended_rules::rentals(&draft, 60_000, &mut result).activities[0].property,
        10_000
    );
    // Preserve current commercial and residential money controls unchanged.
    let v = commercial(20200101);
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &v.buildings[0],
            false,
            (&v.property_type, &crate::RentalLocation::UnitedStates),
            2026,
            v.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(705_100)
    );
    let v = rental(48_000, 6_000);
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &v.buildings[0],
            false,
            (&v.property_type, &crate::RentalLocation::UnitedStates),
            2026,
            v.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(1_000_000)
    );
}

#[test]
fn unit29_rental_315_accumulated_and_final_cents_never_exceed_original_basis() {
    for placed in [19870101, 19900101, 19930512] {
        let v = commercial(placed);
        let b = &v.buildings[0];
        let amounts: Vec<i64> = (placed / 10000..=2026)
            .map(|year| {
                crate::extended_rules::depreciation_for_year(
                    b,
                    false,
                    (&v.property_type, &crate::RentalLocation::UnitedStates),
                    year,
                    None,
                    crate::kernel::source_fact::SourceFact::NotReported,
                )
                .unwrap()
            })
            .collect();
        // Annual rounded source amounts close to the actual27500000c basis,
        // including the final-year remaining cents, then stay at known zero.
        assert_eq!(amounts.iter().sum::<i64>(), 27_500_000);
        assert_eq!(*amounts.last().unwrap(), 0);
        assert_eq!(
            crate::asset_sale::accumulated(b, false, &v.property_type, 20260915),
            Ok(27_500_000)
        );
    }
    let v = commercial(19900101);
    // Earlier rounded source allowances total27027120c. The final2021
    // fraction would round472884c, but actual remaining basis is472880c.
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &v.buildings[0],
            false,
            (&v.property_type, &crate::RentalLocation::UnitedStates),
            2021,
            None,
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(472_880)
    );
    // First year23halfmonths/756, then24/756; no annual rate override.
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &v.buildings[0],
            false,
            (&v.property_type, &crate::RentalLocation::UnitedStates),
            1990,
            None,
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(836_640)
    );
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &v.buildings[0],
            false,
            (&v.property_type, &crate::RentalLocation::UnitedStates),
            1991,
            None,
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(873_016)
    );
    assert_eq!(
        crate::asset_sale::accumulated(&v.buildings[0], true, &v.property_type, 20260915),
        Ok(25_208_333)
    );
}

#[test]
fn unit29_rental_315_exchanged_basis_and_ubia_do_not_restart_exhausted_clock() {
    let mut v = commercial(19900101);
    v.buildings[0].cost = 27_500_000;
    v.buildings[0].land = 0;
    let x = RentalExchange {
        transferred: 20260901,
        identified: 20260915,
        received: 20260915,
        extended: None,
        replacement: ExchangeReplacement {
            address: "Synthetic replacement".into(),
            value: 40_000_000,
            land: 0,
            used: ReplacementUse::Rental {
                kind: RentalPropertyType::Commercial,
                in_service: 20260915,
            },
        },
        cash_received: 0,
        other_received: 0,
        debt_relieved: 0,
        debt_assumed: 0,
        cash_paid: 0,
        expenses: 0,
        land_value: 0,
        party: ExchangeParty::Unrelated,
        special: false,
    };
    let form = x.form(&v, false).unwrap();
    assert_eq!(form.depreciation, 275_000);
    assert_eq!(form.exchanged_basis, 0);
    assert_eq!(form.excess_basis, 0);
    assert_eq!(x.replacement_depreciation(&v, false).unwrap().total(), 0);
    assert_eq!(x.ubia(&v).unwrap(), 0);
}

#[test]
fn unit29_rental_historical_source_dates_and_land_keep_named_native_boundaries() {
    {
        let placed = 19861231;
        let v = commercial(placed);
        assert_eq!(
            crate::extended_rules::depreciation_for_year(
                &v.buildings[0],
                false,
                (&v.property_type, &crate::RentalLocation::UnitedStates),
                2026,
                v.sale.date(),
                crate::kernel::source_fact::SourceFact::NotReported
            )
            .unwrap_err()
            .0,
            "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS"
        );
    }
    let mut v = commercial(19900101);
    v.buildings[0].land = v.buildings[0].cost + 1;
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &v.buildings[0],
            false,
            (&v.property_type, &crate::RentalLocation::UnitedStates),
            2026,
            v.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        )
        .unwrap_err()
        .0,
        "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS"
    );
    assert_eq!(
        crate::historical_recovery::rental_recovery_window(
            crate::historical_recovery::RentalBuildingRecovery::Commercial315,
            19900001,
            2026
        ),
        0
    );
}

#[test]
fn unit29_rental_315_normal_save_reopen_resave_restart_and_reject_atomicity() {
    let mut page = rental_page(4_800_000);
    page.kind = RentalPropertyType::Commercial;
    page.building.cost = 33_000_000;
    page.building.land = 5_500_000;
    page.building.in_service = 19900101;
    page.more.qbi = RentalBusiness::Yes { material_participation: true };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    crate::tests::cross::interview::accepted(&save_rentals(&mut session, vec![page.clone()]));
    let rows = stored_rentals(&session);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].2.buildings[0].in_service, 19900101);
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &rows[0].2.buildings[0],
            false,
            (&rows[0].2.property_type, &crate::RentalLocation::UnitedStates),
            2026,
            rows[0].2.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(0)
    );
    assert_rentals_round_trip(&mut session, core::slice::from_ref(&page));
    let saved = session.draft.clone();
    page.building.land = page.building.cost + 1;
    crate::tests::cross::interview::rejected_with(
        save_rentals(&mut session, vec![page]),
        "RENTAL_LAND_ABOVE_COST",
    );
    assert_eq!(session.draft, saved);
    assert_eq!(restart(saved.clone()).draft, saved);
    // A genuine reopened source correction retains its accepted incomeID;
    // annual depreciation and AGI follow the corrected source date once.
    let mut corrected =
        crate::tests::cross::interview::open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut corrected else { unreachable!() };
    let crate::IncomeRecord::Rental(edited) = &mut documents.documents[0].document else {
        unreachable!()
    };
    edited.building.in_service = 20200101;
    crate::tests::cross::interview::accepted(&send(&mut session, corrected));
    let after = stored_rentals(&session);
    assert_eq!(after[0].0, rows[0].0);
    assert_eq!(after[0].2.buildings[0].in_service, 20200101);
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &after[0].2.buildings[0],
            false,
            (&after[0].2.property_type, &crate::RentalLocation::UnitedStates),
            2026,
            after[0].2.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(705_100)
    );
    assert_eq!(value(&supported(&session.draft), "agi"), 100_949);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

// Env-gated Avro bridge for the genuine normal editor; never changes tax logic.
#[test]
fn unit29_rental_315_actual_client_save_reopen_resave_restart() {
    extern crate std;
    use crate::tests::cross::interview::{document, open_topic, page_documents};
    let placed = match std::env::var("TAX_UNIT29_RENTAL_CASE").as_deref() {
        Ok("1987") => 19870101,
        Ok("1993") => 19930512,
        _ => 19900101,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let seed = open_topic(&mut session, TaxSection::Rentals);
    assert!(page_documents(&seed).is_empty());
    if let Ok(path) = std::env::var("TAX_UNIT29_RENTAL_CALC_INPUT") {
        let calculation = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(
            calculation,
            P::EstimateDocuments { .. }
                | P::PreviewDocument { .. }
                | P::DocumentPropertyOptions { .. }
        ));
        let before = session.draft.clone();
        let Reply::Response(reply) = send(&mut session, calculation) else {
            panic!("calculation response")
        };
        assert!(matches!(
            reply,
            P::RunningEstimate { .. }
                | P::IncomePreview { .. }
                | P::DocumentPropertyChoices { .. }
                | P::Problem { .. }
        ));
        assert_eq!(session.draft, before);
        std::fs::write(std::env::var("TAX_UNIT29_RENTAL_CALC_OUTPUT").unwrap(), reply.to_avro())
            .unwrap();
        return;
    }
    let mut rental = rental_page(4_800_000);
    rental.kind = RentalPropertyType::Commercial;
    rental.building.cost = 33_000_000;
    rental.building.land = 5_500_000;
    rental.building.in_service = placed;
    let mut page = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    documents.documents = vec![document(
        0,
        IncomeSource::Record { value: crate::IncomeRecord::Rental(rental.clone()) },
    )]
    .into_iter()
    .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
    .collect();
    for (key, request) in [("TAX_UNIT29_RENTAL_SEED", &seed), ("TAX_UNIT29_RENTAL_ANSWERS", &page)]
    {
        if let Ok(path) = std::env::var(key) {
            std::fs::write(path, request.to_avro()).unwrap();
        }
    }
    if let Ok(path) = std::env::var("TAX_UNIT29_RENTAL_PAYLOAD") {
        page = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(page, P::SaveIncomes { .. }));
    }
    let before = session.draft.clone();
    let reply = send(&mut session, page);
    if let Ok(key) = std::env::var("TAX_UNIT29_RENTAL_EXPECT_ERROR") {
        crate::tests::cross::interview::rejected_with(reply, &key);
        assert_eq!(session.draft, before);
        assert_eq!(restart(before.clone()).draft, before);
        let reopened = open_topic(&mut session, TaxSection::Rentals);
        assert!(page_documents(&reopened).is_empty());
        if let Ok(path) = std::env::var("TAX_UNIT29_RENTAL_REOPEN") {
            std::fs::write(path, reopened.to_avro()).unwrap();
        }
        return;
    }
    crate::tests::cross::interview::accepted(&reply);
    let stored = session.draft.clone();
    let rows = stored_rentals(&session);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].2.buildings[0].in_service, placed);
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &rows[0].2.buildings[0],
            false,
            (&rows[0].2.property_type, &crate::RentalLocation::UnitedStates),
            2026,
            rows[0].2.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(0)
    );
    assert_eq!(value(&supported(&stored), "agi"), 108_000);
    assert_rentals_round_trip(&mut session, &[rental]);
    assert_eq!(restart(stored.clone()).draft, stored);
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    if let Ok(path) = std::env::var("TAX_UNIT29_RENTAL_REOPEN") {
        std::fs::write(path, reopened.to_avro()).unwrap();
    }
    if let Ok(path) = std::env::var("TAX_UNIT29_RENTAL_RESAVE") {
        let request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(request, P::SaveIncomes { .. }));
        crate::tests::cross::interview::accepted(&send(&mut session, request));
        assert_eq!(session.draft, stored);
        assert_eq!(restart(session.draft.clone()).draft, stored);
    }
}

#[test]
fn unit29_rental_shared_recovery_propagates_canonical_operand_errors_without_panic() {
    let mut v = commercial(20200101);
    v.buildings[0].land = 0;
    v.buildings[0].acquired = RentalAcquisition::Converted { fair_market_value: 0 };
    v.buildings[0].rehabilitation = Some(RehabilitationCredit {
        period_start: 20180101,
        phased: false,
        building_basis: 0,
        period_expenditures: 33_000_000,
        earlier: vec![],
        freed: 0,
        confirmed: true,
    });
    // Preserve the canonical owner's operand error as a named source
    // boundary; no FMV/rehabilitation qualification rule or zero basis is
    // invented here. The ordinary Page writes Purchased on improvements.
    assert_eq!(
        crate::historical_recovery::rental_building_amount(
            crate::historical_recovery::RentalBuildingRecovery::Commercial39,
            -6_600_000,
            20200101,
            2026,
            24,
            None
        )
        .unwrap_err(),
        OfficeRecoveryIssue::InvalidBasis
    );
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &v.buildings[0],
            false,
            (&v.property_type, &crate::RentalLocation::UnitedStates),
            2026,
            v.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        )
        .unwrap_err()
        .0,
        "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS"
    );
    v.buildings[0].rehabilitation = None;
    for year in [0, 10_000, u32::MAX] {
        assert_eq!(
            crate::historical_recovery::rental_building_amount(
                crate::historical_recovery::RentalBuildingRecovery::Commercial39,
                0,
                20200101,
                year,
                0,
                None
            )
            .unwrap_err(),
            OfficeRecoveryIssue::InvalidDate
        );
        assert_eq!(
            crate::extended_rules::depreciation_for_year(
                &v.buildings[0],
                false,
                (&v.property_type, &crate::RentalLocation::UnitedStates),
                year,
                None,
                crate::kernel::source_fact::SourceFact::NotReported
            )
            .unwrap_err()
            .0,
            "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS"
        );
    }
}
