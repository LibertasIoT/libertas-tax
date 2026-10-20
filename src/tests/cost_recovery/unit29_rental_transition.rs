// Documentary 1993 transition sources, shared recovery and normal lifecycle.
use super::*;
use crate::tests::cross::interview::{
    accepted, assert_rentals_round_trip, open_topic, rental_page, save_rentals, stored_rentals,
};

#[test]
fn unit29_rental_transition_actual_sources_select_one_original_clock_for_every_consumer() {
    for (source, annual, accumulated, ubia, agi) in [
        (Commercial1993Transition::Ordinary39Year, 705_100, 23_386_321, 275_000, 100_949),
        (transition_binding(), 0, 27_500_000, 0, 108_000),
        (transition_construction(), 0, 27_500_000, 0, 108_000),
    ] {
        let record = transition_source_record(source);
        assert_eq!(
            crate::extended_rules::depreciation_for_year(
                &record.buildings[0],
                false,
                &record.property_type,
                2026,
                record.sale.date(),
                crate::kernel::source_fact::SourceFact::NotReported
            ),
            Ok(annual)
        );
        assert_eq!(
            crate::asset_sale::accumulated(
                &record.buildings[0],
                false,
                &record.property_type,
                20260915,
            ),
            Ok(accumulated)
        );
        assert_eq!(
            crate::extended_rules::depreciation_for_year(
                &record.buildings[0],
                true,
                &record.property_type,
                2026,
                record.sale.date(),
                crate::kernel::source_fact::SourceFact::NotReported
            ),
            Ok(687_500)
        );
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(FederalIncome::Rental { data: record }));
        let mut result = FederalResult::empty(0);
        assert_eq!(
            crate::extended_rules::rentals(&draft, 60_000, &mut result).activities[0].property,
            ubia
        );
        assert_eq!(value(&supported(&draft), "agi"), agi);
    }
}

#[test]
fn unit29_rental_transition_improvement_retains_its_own_source_and_lifecycle() {
    for (source, annual, agi, ubia) in [
        (Commercial1993Transition::Ordinary39Year, 25_640, 107_744, 10_000),
        (transition_binding(), 0, 108_000, 0),
        (transition_construction(), 0, 108_000, 0),
    ] {
        let mut page = transition_source_page(transition_binding());
        page.building.in_service = 19900101;
        page.building.commercial_1993_transition = None;
        page.building.allowed_commercial_1993_transition.clear();
        page.more.improvements.push(Improvement {
            casualty: None,
            id: 0,
            energy: None,
            sale_history: None,
            credit_reduction: 0,
            cost: 1_000_000,
            in_service: 19930701,
            commercial_1993_transition: Some(source.clone()),
            allowed_commercial_1993_transition: crate::rental_recovery::allowed(
                &page.kind, 19930701,
            ),
            rehabilitation: None,
            partial: None,
        });
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        accepted(&save_rentals(&mut session, vec![page.clone()]));
        let record = &stored_rentals(&session)[0].2;
        assert_eq!(record.buildings.len(), 2);
        assert_eq!(record.buildings[0].commercial_1993_transition, None);
        assert_eq!(record.buildings[1].commercial_1993_transition, Some(source));
        assert_eq!(
            crate::extended_rules::depreciation_for_year(
                &record.buildings[1],
                false,
                &record.property_type,
                2026,
                record.sale.date(),
                crate::kernel::source_fact::SourceFact::NotReported
            ),
            Ok(annual)
        );
        assert_eq!(value(&supported(&session.draft), "agi"), agi);
        let mut result = FederalResult::empty(0);
        assert_eq!(
            crate::extended_rules::rentals(&session.draft, 60_000, &mut result).activities[0]
                .property,
            ubia
        );
        assert_rentals_round_trip(&mut session, core::slice::from_ref(&page));
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}

#[test]
fn unit29_rental_transition_exchange_carries_actual_source_clock_and_existing_form_rounding() {
    let exchange = RentalExchange {
        transferred: 20260915,
        identified: 20261001,
        received: 20261015,
        extended: None,
        replacement: ExchangeReplacement {
            address: "Synthetic replacement".into(),
            value: 40_000_000,
            land: 0,
            used: ReplacementUse::Rental {
                kind: RentalPropertyType::Commercial,
                in_service: 20261015,
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
    for (source, depreciation, basis, current, ubia) in [
        (Commercial1993Transition::Ordinary39Year, 233_863, 41_137, 146_918, 27_500_000),
        (transition_binding(), 275_000, 0, 0, 0),
        (transition_construction(), 275_000, 0, 0, 0),
    ] {
        let mut record = transition_source_record(source);
        record.buildings[0].cost = 27_500_000;
        record.buildings[0].land = 0;
        let form = exchange.form(&record, false).unwrap();
        assert_eq!(form.depreciation, depreciation);
        assert_eq!(form.exchanged_basis, basis);
        assert_eq!(form.excess_basis, 0);
        // Ordinary39: existing form-round basis4113700c *5/140 remaining
        // halfmonths rounds146918c; a fully recovered315 source carries0.
        assert_eq!(exchange.replacement_depreciation(&record, false).unwrap().total(), current);
        assert_eq!(exchange.ubia(&record).unwrap(), ubia);
    }
    let mut unknown = transition_source_record(transition_binding());
    unknown.buildings[0].commercial_1993_transition = None;
    assert_eq!(exchange.ubia(&unknown).unwrap_err().0, "RENTAL_1993_RECOVERY_SOURCE_REQUIRED");
    assert_eq!(
        exchange.replacement_depreciation(&unknown, false).unwrap_err().0,
        "RENTAL_1993_RECOVERY_SOURCE_REQUIRED"
    );
    assert!(exchange.replacement_depreciation(&unknown, true).is_ok());
}

#[test]
fn unit29_rental_transition_projection_is_date_specific_and_preserves_improvement_order() {
    let mut input = RentalRecoveryInputs {
        kind: RentalPropertyType::Commercial,
        building: RentalRecoveryDate { in_service: 20200101 },
        more: RentalRecoveryMoreDates {
            improvements: vec![
                RentalRecoveryDate { in_service: 19930701 },
                RentalRecoveryDate { in_service: 19930512 },
            ],
        },
    };
    let output = crate::rental_recovery::choices(&input);
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let before = session.draft.clone();
    let Reply::Response(reply) =
        send(&mut session, P::RentalRecoveryOptions { user: TEST_USER, value: input.clone() })
    else {
        panic!("source choices response")
    };
    assert_eq!(
        reply,
        P::RentalRecoveryChoices { building: output.building.clone(), more: output.more.clone() }
    );
    assert_eq!(session.draft, before);

    assert!(output.building.allowed_commercial_1993_transition.is_empty());
    assert_eq!(output.more.improvements.len(), 2);
    assert_eq!(
        output.more.improvements[0].allowed_commercial_1993_transition,
        crate::rental_recovery::allowed(&input.kind, 19930701)
    );
    assert!(output.more.improvements[1].allowed_commercial_1993_transition.is_empty());
    // PL103-66 section13151(b): both the first affected service date and
    // December31 service qualify for the actual documentary source choice.
    for placed in [19930513, 19931231] {
        assert_eq!(
            crate::rental_recovery::allowed(&RentalPropertyType::Commercial, placed).len(),
            3
        );
        assert!(
            crate::rental_recovery::source_rule(
                &RentalPropertyType::Commercial,
                placed,
                Some(&transition_binding())
            )
            .is_none()
        );
    }
    input.kind = RentalPropertyType::SingleFamily;
    assert!(
        crate::rental_recovery::choices(&input)
            .more
            .improvements
            .iter()
            .all(|row| row.allowed_commercial_1993_transition.is_empty())
    );
    assert!(crate::rental_recovery::allowed(&RentalPropertyType::Commercial, 19930230).is_empty());
}

#[test]
fn unit29_rental_transition_unknown_and_invalid_documentary_sources_never_assume_39() {
    let kind = RentalPropertyType::Commercial;
    // 2026-10-19: the Source facts contract retains optional unknown input;
    // require() stops the calculation at its named boundary below.
    assert!(crate::rental_recovery::source_rule(&kind, 19930701, None).is_none());
    for date in [0, 19930230, 19930513, 19940101] {
        for source in [
            Commercial1993Transition::BindingContract(Commercial1993ContractSource {
                effective: date,
                source_reference: "Actual contract".into(),
            }),
            Commercial1993Transition::ConstructionBegun(Commercial1993ConstructionSource {
                began: date,
                source_reference: "Actual construction".into(),
            }),
        ] {
            assert_eq!(
                crate::rental_recovery::source_rule(&kind, 19930701, Some(&source)).unwrap().key,
                "RENTAL_1993_TRANSITION_SOURCE_DATE"
            );
        }
    }
    for date in [19930512, 19940101] {
        assert!(crate::rental_recovery::source_rule(&kind, date, None).is_none());
        assert_eq!(
            crate::rental_recovery::source_rule(&kind, date, Some(&transition_binding()))
                .unwrap()
                .key,
            "RENTAL_1993_TRANSITION_NOT_APPLICABLE"
        );
    }
    assert_eq!(
        crate::rental_recovery::source_rule(
            &RentalPropertyType::SingleFamily,
            19930701,
            Some(&transition_binding()),
        )
        .unwrap()
        .key,
        "RENTAL_1993_TRANSITION_NOT_APPLICABLE"
    );
    for (reference, valid) in [
        ("".into(), false),
        ("😀".repeat(120), true),
        // 2026-10-19: No count caps contract removes the old 240-unit source cap.
        ("😀".repeat(121), true),
        (" ".into(), false),
    ] {
        let source = Commercial1993Transition::BindingContract(Commercial1993ContractSource {
            effective: 19930512,
            source_reference: reference,
        });
        assert_eq!(
            crate::rental_recovery::source_rule(&kind, 19930701, Some(&source)).is_none(),
            valid
        );
    }
    let mut record = transition_source_record(transition_binding());
    record.buildings[0].commercial_1993_transition = None;
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &record.buildings[0],
            false,
            &record.property_type,
            2026,
            record.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        )
        .unwrap_err()
        .0,
        "RENTAL_1993_RECOVERY_SOURCE_REQUIRED"
    );
    assert_eq!(
        crate::extended_rules::depreciation_for_year(
            &record.buildings[0],
            true,
            &record.property_type,
            2026,
            record.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        ),
        Ok(687_500)
    );
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Rental { data: record }));
    assert_ne!(estimate(&draft).state, ResultState::EstimatedForSupportedScope);
    assert!(draft.valid_stored());
    for qbi in [false, true] {
        let mut known_amt = draft.clone();
        let data = known_amt
            .income
            .iter_mut()
            .find_map(|entry| match &mut entry.income {
                FederalIncome::Rental { data } => Some(data),
                _ => None,
            })
            .unwrap();
        data.qbi_business = if qbi {
            RentalBusiness::Yes { material_participation: true }
        } else {
            RentalBusiness::No
        };
        let mut result = FederalResult::empty(0);
        let pass = crate::extended_rules::rentals_for(&known_amt, 60_000, &mut result, true);
        assert_eq!(pass.income, 41_125);
        assert_eq!(pass.qbi, if qbi { 41_125 } else { 0 });
        assert!(pass.activities.is_empty());
        assert!(pass.inputs_complete);
        // 2026-10-19: an ordinary kept passive loss is excluded from MAGI. The supplied
        // $60000 pre-rental MAGI and intrinsic AMT40 schedule suffice here.
        let data = known_amt
            .income
            .iter_mut()
            .find_map(|entry| match &mut entry.income {
                FederalIncome::Rental { data } => Some(data),
                _ => None,
            })
            .unwrap();
        data.rent = 0;
        let mut result = FederalResult::empty(0);
        let pass = crate::extended_rules::rentals_for(&known_amt, 60_000, &mut result, true);
        assert!(pass.inputs_complete);
        assert_eq!(pass.income, -6_875);
        assert_eq!(pass.qbi, if qbi { -6_875 } else { 0 });
        assert!(pass.activities.is_empty());
        let entry = known_amt
            .income
            .iter_mut()
            .find(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
            .unwrap();
        let source_id = entry.id;
        let FederalIncome::Rental { data } = &mut entry.income else { unreachable!() };
        data.buildings[0].commercial_1993_transition =
            Some(Commercial1993Transition::Ordinary39Year);
        let mut result = FederalResult::empty(0);
        let pass = crate::extended_rules::rentals_for(&known_amt, 60_000, &mut result, true);
        assert!(pass.inputs_complete);
        assert_eq!(pass.income, -6_875);
        assert_eq!(pass.qbi, if qbi { -6_875 } else { 0 });
        assert_eq!(
            pass.business_losses_by_id.get(&source_id).copied().unwrap_or(0),
            if qbi { 6_875 } else { 0 }
        );
        assert!(pass.activities.is_empty());
        // A nonpassive short-stay business contributes its regular net to
        // MAGI, so that actual regular-source dependency remains unavailable.
        let entry = known_amt
            .income
            .iter_mut()
            .find(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
            .unwrap();
        let FederalIncome::Rental { data } = &mut entry.income else { unreachable!() };
        data.property_type = RentalPropertyType::VacationShortTerm {
            stays: ShortTermStays::SevenDaysOrLess { material_participation: true },
        };
        data.buildings[0].commercial_1993_transition = None;
        let mut result = FederalResult::empty(0);
        let pass = crate::extended_rules::rentals_for(&known_amt, 60_000, &mut result, true);
        assert!(!pass.inputs_complete);
        assert_eq!(
            pass.source_issue.as_ref().unwrap().issue.key(),
            "RENTAL_1993_RECOVERY_SOURCE_REQUIRED"
        );
    }

    let mut page = transition_source_page(transition_binding());
    page.building.commercial_1993_transition = None;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_rentals(&mut session, vec![page.clone()]));
    assert_rentals_round_trip(&mut session, core::slice::from_ref(&page));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    assert_ne!(estimate(&session.draft).state, ResultState::EstimatedForSupportedScope);
}

#[test]
fn unit29_rental_transition_normal_save_resave_restart_and_nonzero_locator_atomicity() {
    for source in
        [Commercial1993Transition::Ordinary39Year, transition_binding(), transition_construction()]
    {
        let page = transition_source_page(source.clone());
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        accepted(&save_rentals(&mut session, vec![page.clone()]));
        assert_eq!(
            stored_rentals(&session)[0].2.buildings[0].commercial_1993_transition,
            Some(source)
        );
        assert_rentals_round_trip(&mut session, core::slice::from_ref(&page));
        let saved = session.draft.clone();
        let mut echoed = open_topic(&mut session, TaxSection::Rentals);
        let P::SaveIncomes { documents, .. } = &mut echoed else { unreachable!() };
        let IncomeDocument::Rental(value) = &mut documents.documents[0].document else {
            unreachable!()
        };
        value.building.allowed_commercial_1993_transition.clear();
        accepted(&send(&mut session, echoed));
        assert_eq!(session.draft, saved);
        assert_rentals_round_trip(&mut session, core::slice::from_ref(&page));
        let before = session.draft.clone();
        let mut invalid = page.clone();
        let improvement = |source| Improvement {
            casualty: None,
            id: 0,
            energy: None,
            sale_history: None,
            credit_reduction: 0,
            cost: 1_000_000,
            in_service: 19930701,
            commercial_1993_transition: Some(source),
            allowed_commercial_1993_transition: crate::rental_recovery::allowed(
                &invalid.kind,
                19930701,
            ),
            rehabilitation: None,
            partial: None,
        };
        invalid.more.improvements = vec![
            improvement(Commercial1993Transition::Ordinary39Year),
            improvement(Commercial1993Transition::ConstructionBegun(
                Commercial1993ConstructionSource {
                    began: 19930513,
                    source_reference: "Actual construction".into(),
                },
            )),
        ];
        let reply = save_rentals(&mut session, vec![rental_page(0), invalid]);
        let Reply::Response(P::Problem { field, .. }) = &reply else { panic!("source refusal") };
        assert_eq!(
            field.as_deref(),
            Some(
                "[documents/documents/1/document/more/improvements/1/commercial_1993_transition/began]"
            )
        );
        crate::tests::cross::interview::rejected_with(reply, "RENTAL_1993_TRANSITION_SOURCE_DATE");
        assert_eq!(session.draft, before);
        assert_eq!(restart(before.clone()).draft, before);
        assert_rentals_round_trip(&mut session, core::slice::from_ref(&page));
    }
}

#[test]
fn unit29_rental_transition_genuine_source_correction_keeps_identity_and_recomputes_tax() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_rentals(&mut session, vec![transition_source_page(transition_binding())]));
    let id = stored_rentals(&session)[0].0;
    let mut request = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::Rental(page) = &mut documents.documents[0].document else { unreachable!() };
    page.building.commercial_1993_transition = Some(Commercial1993Transition::Ordinary39Year);
    accepted(&send(&mut session, request));
    assert_eq!(stored_rentals(&session)[0].0, id);
    assert_eq!(value(&supported(&session.draft), "agi"), 100_949);
    let before = session.draft.clone();
    let mut request = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::Rental(page) = &mut documents.documents[0].document else { unreachable!() };
    page.building.in_service = 20200101;
    crate::tests::cross::interview::rejected_with(
        send(&mut session, request),
        "RENTAL_1993_TRANSITION_NOT_APPLICABLE",
    );
    assert_eq!(session.draft, before);
    let mut request = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let IncomeDocument::Rental(page) = &mut documents.documents[0].document else { unreachable!() };
    page.building.in_service = 20200101;
    // Native candidate twin of an explicit clear with genuine rebuilt choices;
    // the actual client test separately proves normal ConstrainedBy resetting.
    page.building.commercial_1993_transition = None;
    page.building.allowed_commercial_1993_transition =
        crate::rental_recovery::allowed(&page.kind, 20200101);
    accepted(&send(&mut session, request));
    assert_eq!(stored_rentals(&session)[0].0, id);
    assert_eq!(stored_rentals(&session)[0].2.buildings[0].in_service, 20200101);
    assert_eq!(value(&supported(&session.draft), "agi"), 100_949);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
