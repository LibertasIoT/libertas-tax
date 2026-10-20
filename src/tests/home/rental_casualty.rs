use super::*;
use crate::casualty::rental::items;
use crate::kernel::source_fact::SourceFact;

#[test]
fn kept_recurring_damage_uses_actual_records_and_the_original_remaining_clock() {
    let draft = kept_rental_casualty();
    let id = draft.income.last().unwrap().id;
    let source = items(&draft).remove(0);
    let facts = established(&source.regular);
    assert!(source.kept);
    assert_eq!(facts.basis_cents, 4_940_000);
    assert_eq!(facts.raw_loss_cents, 1_000_000);
    assert_eq!(facts.continuing_basis_cents, SourceFact::Reported(3_940_000));
    let FederalIncome::Rental { data } = &draft.income.last().unwrap().income else {
        unreachable!()
    };
    // Pub946: original27.5year clock, 517halfmonths at Jan1,2026.
    // Revised opening basis3,940,000+1,660,000−1,430,000 =4,170,000.
    // Its annual24/517share is193,578cents; no casualty-month split.
    assert_eq!(
        crate::casualty::rental::current_depreciation(
            &data.buildings[0],
            data,
            false,
            2026,
            SourceFact::NotReported
        )
        .unwrap(),
        193_578
    );
    assert_eq!(crate::extended_rules::rental_net(data).unwrap(), 4_756_422);
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line27")), 1_000_000);
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_casualty_0_ordinary")),
        -1_000_000
    );
    let saved = save_reopen_checked_sources(
        draft,
        &[TaxSection::Rentals],
        &[TaxSection::Rentals],
        estimate,
    );
    assert_eq!(items(&saved).remove(0), source);
}

#[test]
fn kept_event_retains_independent_amt_records_after_regular_recovery_has_ended() {
    let mut draft = kept_rental_casualty();
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let building = &mut data.buildings[0];
    building.in_service = 19980101;
    let records = &mut building.casualty.as_mut().unwrap().continuing_records;
    records.regular = rental_casualty_basis(0, 6_600_000, 6_600_000);
    records.alternative = rental_casualty_basis(1_828_750, 4_771_250, 4_613_125);
    let building = &data.buildings[0];
    // Regular27.5years ended in2025; original pre1999AMT ADS40 continues.
    // AMT opening986,875 at24/289remaining halfmonths gives81,955cents.
    assert_eq!(
        crate::casualty::rental::current_depreciation(
            building,
            data,
            false,
            2026,
            SourceFact::NotReported
        )
        .unwrap(),
        0
    );
    assert_eq!(
        crate::casualty::rental::current_depreciation(
            building,
            data,
            true,
            2026,
            SourceFact::NotReported
        )
        .unwrap(),
        81_955
    );
    let sources = items(&draft);
    assert_eq!(established(&sources[0].regular).raw_loss_cents, 0);
    assert_eq!(established(&sources[0].alternative).raw_loss_cents, 1_000_000);
    assert!(estimate(&draft).tax.is_some());
}

#[test]
fn kept_commercial_damage_uses_the_original_39_year_clock_without_annual_table_rounding() {
    let mut draft = kept_rental_casualty();
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.property_type = RentalPropertyType::Commercial;
    let building = &mut data.buildings[0];
    building.cost = 7_800_000;
    let records = &mut building.casualty.as_mut().unwrap().continuing_records;
    records.regular = rental_casualty_basis(6_416_666, 1_383_334, 1_191_667);
    records.alternative = records.regular.clone();
    // Pub946: Jan2020 commercial property retains its original39-year
    // recovery period. Actual event ledger gives revised opening5,608,333
    // and793 remaining halfmonths; round24/793 once to169,735 cents.
    for amt in [false, true] {
        assert_eq!(
            crate::casualty::rental::current_depreciation(
                &data.buildings[0],
                data,
                amt,
                2026,
                SourceFact::NotReported
            )
            .unwrap(),
            169_735
        );
    }
    let sources = items(&draft);
    assert_eq!(established(&sources[0].regular).raw_loss_cents, 1_000_000);
    assert_eq!(sources[0].alternative, sources[0].regular);
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
}

#[test]
fn kept_records_do_not_certify_missing_history_unpaid_insurance_rental_scope_or_credit() {
    for case in 0..5 {
        let mut draft = kept_rental_casualty();
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        let source = data.buildings[0].casualty.as_mut().unwrap();
        match case {
            0 => source.continuing_records.regular.basis = crate::SourceCents::Unavailable,
            1 => source.casualty.reimbursement = 1_200_000,
            // The explicit nonordinary choice omits the whole source with a
            // notice. Mixed use instead exercises an unanswered basis scope.
            2 => {
                data.personal_use = RentalPersonalUse::Yes {
                    fair_rental_days: 300,
                    personal_days: 1,
                    overlap_days: 0,
                    direct_expenses: 0,
                }
            }
            3 => {
                source.continuing_records.regular.prior_depreciation =
                    crate::SourceCents::Unavailable
            }
            _ => data.buildings[0].credit_reduction = 100,
        }
        let result = estimate(&draft);
        assert!(result.tax.is_none(), "case{case}: {:?}", result.issues);
        assert!(result.issues.iter().any(|issue| issue.display.key
            == if case == 4 {
                "RENTAL_CASUALTY_CREDIT_UNAVAILABLE"
            } else {
                "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE"
            }));
        if case == 0 {
            let source = items(&draft).remove(0);
            assert_eq!(source.regular, SourceFact::Unavailable);
            assert_eq!(established(&source.alternative).raw_loss_cents, 1_000_000);
        } else if case == 3 {
            assert_eq!(established(&items(&draft)[0].regular).raw_loss_cents, 1_000_000);
        }
    }
}

#[test]
fn kept_community_ledger_cents_fit_the_actual_cost_minus_land_on_both_returns() {
    for (cost, land, through, expected) in [
        (6_600_000, 0, 1_659_999, [(2_470_001, 829_999), (2_470_000, 830_000)]),
        (6_600_002, 1, 1_660_000, [(2_470_000, 830_000), (2_470_001, 830_000)]),
    ] {
        let mut projected = Vec::new();
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = kept_rental_casualty();
            draft.setup = complete(FederalStatus::Separate).setup;
            let setup = draft.setup.as_mut().unwrap();
            setup.community_property = community_arizona();
            for entry in &mut draft.income {
                entry.community = CommunityShare::Community;
            }
            let entry = draft.income.last_mut().unwrap();
            entry.community = CommunityShare::Community;
            entry.owner = owner;
            let FederalIncome::Rental { data } = &mut entry.income else { unreachable!() };
            let building = &mut data.buildings[0];
            building.cost = cost;
            building.land = land;
            let records = &mut building.casualty.as_mut().unwrap().continuing_records;
            records.regular = rental_casualty_basis(4_940_001, through, 1_430_000);
            records.alternative = records.regular.clone();
            let source = items(&draft).remove(0);
            let facts = established(&source.regular);
            assert_eq!(source.alternative, source.regular);
            projected.push((facts.basis_cents, facts.depreciation_cents));
            let result = estimate(&draft);
            assert!(result.tax.is_some(), "owner{owner:?}: {:?}", result.issues);
        }
        assert_eq!(projected.as_slice(), &expected);
        assert_eq!(projected.iter().map(|(basis, _)| basis).sum::<i64>(), 4_940_001);
        assert_eq!(projected.iter().map(|(_, depreciation)| depreciation).sum::<i64>(), through);
    }
}

#[test]
fn native_kept_history_rejects_invalid_reported_amounts_without_changing_the_record() {
    use crate::tests::support::client::income_pages::{open_topic, page_documents, save_topic};
    for (field, amount, key) in [
        ("prior_depreciation", 1_660_001, "RENTAL_CASUALTY_DEPRECIATION_RECORDS"),
        ("basis", 5_000_000, "RENTAL_CASUALTY_DEPRECIATION_RECORDS"),
        ("depreciation", -1, "RENTAL_CASUALTY_AMOUNT"),
    ] {
        let mut session = restart(kept_rental_casualty());
        let editor = open_topic(&mut session, TaxSection::Rentals);
        let mut documents = page_documents(&editor).to_vec();
        let IncomeDocument::Rental(rental) = &mut documents[0].document else { unreachable!() };
        let records = &mut rental.building.casualty.as_mut().unwrap().continuing_records.regular;
        let value = crate::SourceCents::Reported { amount };
        match field {
            "prior_depreciation" => records.prior_depreciation = value,
            "basis" => records.basis = value,
            _ => records.depreciation = value,
        }
        let accepted = session.draft.clone();
        let request = save_topic(&session.draft, TaxSection::Rentals, documents);
        let reply = send(&mut session, request);
        let Reply::Response(P::Problem { field: path, .. }) = &reply else {
            panic!("expected refusal")
        };
        assert!(path.as_ref().is_some_and(|path| {
            path.contains(&alloc::format!("continuing_records/regular/{field}"))
        }));
        rejected_with(reply, key);
        assert_eq!(session.draft, accepted);
    }
}

fn established(
    fact: &SourceFact<crate::casualty::rental::SectionBItem>,
) -> &crate::casualty::rental::SectionBItem {
    let SourceFact::Reported(item) = fact else { panic!("expected actual event basis") };
    item
}

#[test]
fn ceased_event_basis_and_reimbursement_follow_the_actual_rental_schedule() {
    let mut draft = rental_casualty_source(CasualtyKind::Damaged {
        before: 2_500_000,
        after: 1_500_000,
        retired: Some(20261215),
    });
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let mut improvement = data.buildings[0].clone();
    improvement.casualty.as_mut().unwrap().casualty.event = "Synthetic improvement damage".into();
    data.buildings.push(improvement);
    let sources = items(&draft);
    let source = &sources[0];
    let facts = established(&source.regular);
    // $66,000 / 27.5 = $2,400 yearly. Jan2020 and midDec2026
    // each admit $2,300; five intervening years admit $12,000.
    assert_eq!(facts.depreciation_cents, 1_660_000);
    assert_eq!(facts.basis_cents, 4_940_000);
    assert_eq!(facts.raw_loss_cents, 0);
    assert_eq!(facts.admitted_loss_cents, SourceFact::Reported(0));
    assert_eq!(facts.ordinary_recapture_cents, SourceFact::NotReported);
    // Approved F9: reimbursement reduces continuing basis even with no loss.
    assert_eq!(facts.continuing_basis_cents, SourceFact::Reported(3_740_000));
    assert_eq!(source.alternative, source.regular);
    assert_eq!(source.income, draft.income.last().unwrap().id);
    assert!(!source.kept);
    assert_eq!(source.long_term, SourceFact::Reported(true));
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    assert!(!result.return_lines.is_empty());
    // 2026-10-19: pure-business SectionB27 has no280A limitation; this
    // zero-loss cessation has a fully known original depreciation schedule.
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{}_casualty_0_line27", source.income)),
        0
    );
    assert_eq!(
        line_amount(
            &result,
            &alloc::format!("rental_{}_casualty_0_continuing_basis", source.income)
        ),
        3_740_000
    );
    // Reached native Save, reopen, unchanged Save and storage restart retain
    // the actual embedded source, including improvement records.
    let saved = save_reopen_checked_sources(
        draft,
        &[TaxSection::Rentals],
        &[TaxSection::Rentals],
        estimate,
    );
    assert_eq!(items(&saved), sources);
}

#[test]
fn regular_and_amt_basis_are_independent_for_known_retirement_damage() {
    // 2026-10-19 Pub547/Pub946: discovery does not establish retirement.
    // Actual event-date retirement and FMV decline establish this control.
    let mut draft = rental_casualty_source(CasualtyKind::Damaged {
        before: 10_000_000,
        after: 0,
        retired: Some(20261215),
    });
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let building = &mut data.buildings[0];
    building.in_service = 19870101;
    let source = building.casualty.as_mut().unwrap();
    source.casualty.date = 20260615;
    if let CasualtyKind::Damaged { retired, .. } = &mut source.casualty.kind {
        *retired = Some(source.casualty.date);
    }
    source.casualty.reimbursement = 0;
    source.casualty.received = 0;
    source.recurring = true;
    let sources = items(&draft);
    let regular = established(&sources[0].regular);
    let amt = established(&sources[0].alternative);
    assert_eq!(regular.basis_cents, 0);
    // AMT 40 years: $1,581.25 first year + 38*$1,650 + $756.25.
    assert_eq!(amt.basis_cents, 96_250);
    assert_eq!(amt.raw_loss_cents, 96_250);
    assert_eq!(amt.continuing_basis_cents, SourceFact::Reported(0));
    assert!(sources[0].recurring);
    assert!(!sources[0].kept);
}

#[test]
fn expected_money_does_not_fabricate_received_gain_and_replacement_retains_character_work() {
    let mut draft = rental_casualty_source(CasualtyKind::Damaged {
        before: 10_000_000,
        after: 0,
        retired: Some(20261215),
    });
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let source = &mut data.buildings[0].casualty.as_mut().unwrap().casualty;
    source.reimbursement = 6_000_000;
    source.received = 0;
    let pending = items(&draft);
    assert_eq!(established(&pending[0].regular).realized_gain_cents, 0);
    assert_eq!(established(&pending[0].regular).continuing_basis_cents, SourceFact::Unavailable);
    assert!(estimate(&draft).lines.iter().all(|line| !line.id.ends_with("_continuing_basis")));
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let source = &mut data.buildings[0].casualty.as_mut().unwrap().casualty;
    source.received = 6_000_000;
    source.replacement_cost = 5_800_000;
    let paid = items(&draft);
    let facts = established(&paid[0].regular);
    assert_eq!(facts.realized_gain_cents, 1_060_000);
    assert_eq!(facts.recognized_gain_cents, 200_000);
    assert_eq!(facts.postponed_gain_cents, 860_000);
    assert!(estimate(&draft).tax.is_none());
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings[0].acquired = RentalAcquisition::Converted { fair_market_value: 4_400_000 };
    let converted = items(&draft);
    let facts = established(&converted[0].regular);
    // $44,000 / 27.5 = $1,600; first and event years each $1,533.33,
    // plus five full years. Gain basis starts from actual $66,000 cost.
    assert_eq!(facts.basis_cents, 3_293_334);
    assert_eq!(facts.gain_basis_cents, 5_493_334);
    let result = estimate(&draft);
    let id = converted[0].income;
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_casualty_0_loss_basis")),
        3_293_334
    );
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_casualty_0_gain_basis")),
        5_493_334
    );
    assert!(result.lines.iter().all(|line| {
        line.form_line.as_ref().is_none_or(|form| {
            form.form != TaxForm::Form4684 || !matches!(form.line.as_str(), "20" | "22" | "26")
        })
    }));
}

#[test]
fn mixed_use_event_history_never_borrows_the_sale_history_or_current_day_fraction() {
    let mut draft = rental_casualty_source(CasualtyKind::Destroyed);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 60,
        overlap_days: 0,
        direct_expenses: 0,
    };
    data.buildings[0].sale_history = Some(rental_sale_history(6_600_000, 1_000_000, 800_000));
    let sources = items(&draft);
    assert_eq!(sources[0].regular, SourceFact::Unavailable);
    assert_eq!(sources[0].alternative, SourceFact::Unavailable);
    assert_eq!(sources[0].boundary, Some("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE"));
    let result = estimate(&draft);
    assert!(result.tax.is_none());
    assert!(!result.lines.iter().any(|line| line.id.contains("casualty_0_line20")));
}

#[test]
fn property_projection_uses_the_existing_community_property_share() {
    let mut draft = rental_casualty_source(CasualtyKind::Damaged {
        before: 2_500_000,
        after: 1_500_000,
        retired: Some(20261215),
    });
    let setup = draft.setup.as_mut().unwrap();
    setup.status = complete(FederalStatus::Separate).setup.unwrap().status;
    setup.community_property = community_arizona();
    draft.income.last_mut().unwrap().community = CommunityShare::Community;
    let sources = items(&draft);
    let facts = established(&sources[0].regular);
    assert_eq!(facts.basis_cents, 2_470_000);
    assert_eq!(facts.reimbursement_cents, 600_000);
    assert_eq!(facts.fair_values_cents, Some([1_250_000, 750_000]));
    assert_eq!(facts.continuing_basis_cents, SourceFact::Reported(1_870_000));
}

#[test]
fn valid_older_and_elected_future_events_save_but_never_borrow_current_year_basis() {
    for (date, declaration, placed, boundary) in [
        (
            20251215,
            CasualtyDeclaration::Undeclared,
            20200101,
            "RENTAL_CASUALTY_EVENT_YEAR_UNAVAILABLE",
        ),
        (
            20271215,
            CasualtyDeclaration::Federal { reference: "EM-3999".into() },
            20200101,
            "RENTAL_CASUALTY_EVENT_YEAR_UNAVAILABLE",
        ),
        // Ready-to-rent is not acquisition: accept the actual preparation
        // casualty but do not invent a pre-service adjusted basis.
        (
            20260615,
            CasualtyDeclaration::Undeclared,
            20260701,
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
    ] {
        let mut draft = rental_casualty_source(CasualtyKind::Destroyed);
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.buildings[0].in_service = placed;
        let event = &mut data.buildings[0].casualty.as_mut().unwrap().casualty;
        event.date = date;
        event.declaration = declaration;
        let saved = save_reopen_checked_sources(
            draft,
            &[TaxSection::Rentals],
            &[TaxSection::Rentals],
            estimate,
        );
        let source = items(&saved).remove(0);
        assert_eq!(source.boundary, Some(boundary));
        assert_eq!(source.regular, SourceFact::Unavailable);
        let result = estimate(&saved);
        assert_eq!(
            result.issues.iter().filter(|issue| issue.display.key == boundary).count(),
            1,
            "one source correction across regular, AMT and property projections: {:?}",
            result.issues
        );
        let mut two_sources = saved.clone();
        let second = two_sources.income.last().unwrap().clone();
        add(&mut two_sources, second);
        let result = estimate(&two_sources);
        let issues: Vec<_> =
            result.issues.iter().filter(|issue| issue.display.key == boundary).collect();
        assert_eq!(issues.len(), 2, "each source retains its correction: {issues:?}");
        assert_ne!(issues[0].context.identity(), issues[1].context.identity());
    }
}

#[test]
fn missing_event_basis_holding_and_salvage_remain_unavailable() {
    let sale_source = qualified_rental_sale(522_000, Some(19_800));
    let FederalIncome::Rental { data: sold } = &sale_source.income.last().unwrap().income else {
        unreachable!()
    };
    let damage = CasualtyKind::Damaged { before: 10_000_000, after: 0, retired: Some(20261215) };
    for (kind, acquired, placed, sale, ordinary, holding, boundary) in [
        (
            damage.clone(),
            RentalAcquisition::Purchased,
            20260601,
            RentalSale::No,
            true,
            SourceFact::Unavailable,
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            damage.clone(),
            RentalAcquisition::Gift,
            20200101,
            RentalSale::No,
            true,
            SourceFact::Unavailable,
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            CasualtyKind::Destroyed,
            RentalAcquisition::Purchased,
            20200101,
            RentalSale::No,
            true,
            SourceFact::Reported(true),
            "RENTAL_CASUALTY_SALVAGE_UNAVAILABLE",
        ),
        (
            CasualtyKind::Stolen,
            RentalAcquisition::Purchased,
            20200101,
            RentalSale::No,
            true,
            SourceFact::Unavailable,
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            CasualtyKind::Stolen,
            RentalAcquisition::Inherited,
            20200101,
            RentalSale::No,
            true,
            SourceFact::Reported(true),
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            CasualtyKind::Damaged { before: 2_500_000, after: 1_500_000, retired: None },
            RentalAcquisition::Purchased,
            20200101,
            RentalSale::No,
            true,
            SourceFact::Reported(true),
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            CasualtyKind::Damaged { before: 10_000_000, after: 0, retired: Some(20261101) },
            RentalAcquisition::Purchased,
            20200101,
            RentalSale::No,
            true,
            SourceFact::Unavailable,
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            CasualtyKind::Damaged { before: 10_000_000, after: 0, retired: Some(20261231) },
            RentalAcquisition::Purchased,
            20200101,
            RentalSale::No,
            true,
            SourceFact::Reported(true),
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            damage,
            RentalAcquisition::Purchased,
            20200101,
            sold.sale.clone(),
            true,
            SourceFact::Unavailable,
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
        (
            CasualtyKind::Damaged { before: 10_000_000, after: 0, retired: Some(20261215) },
            RentalAcquisition::Purchased,
            20200101,
            RentalSale::No,
            false,
            SourceFact::Reported(true),
            "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE",
        ),
    ] {
        let mut draft = rental_casualty_source(kind);
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.buildings[0].in_service = placed;
        data.buildings[0].acquired = acquired;
        data.sale = sale;
        data.ordinary_rental = ordinary;
        let source = items(&draft).remove(0);
        assert_eq!(source.long_term, holding);
        assert_eq!(source.regular, SourceFact::Unavailable);
        assert_eq!(source.alternative, SourceFact::Unavailable);
        assert_eq!(source.boundary, Some(boundary));
    }
}

#[test]
fn election_window_uses_the_actual_return_year_and_other_recovery_years_stay_unknown() {
    let mut draft = rental_casualty_source(CasualtyKind::Destroyed);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let event = &mut data.buildings[0].casualty.as_mut().unwrap().casualty;
    event.date = 20271215;
    assert_eq!(event.year_issue(2026).unwrap().key, "RENTAL_CASUALTY_ELECTION_DECLARATION");
    assert!(event.year_issue(2027).is_none());
    draft.tax_year = 2027;
    let source = items(&draft).remove(0);
    assert_eq!(source.boundary, Some("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE"));
    assert_eq!(source.regular, SourceFact::Unavailable);
}

#[test]
fn native_save_locates_an_invalid_casualty_and_preserves_accepted_records() {
    use crate::tests::support::client::income_pages::{open_topic, page_documents, save_topic};
    let draft = rental_casualty_source(CasualtyKind::Damaged {
        before: 2_500_000,
        after: 1_500_000,
        retired: None,
    });
    for (before, after, retired, received, key, field) in [
        (
            2_500_000,
            1_500_000,
            None,
            1_200_001,
            "CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT",
            "received",
        ),
        (-1, 1_500_000, None, 1_200_000, "RENTAL_CASUALTY_AMOUNT", "before"),
        (2_500_000, -1, None, 1_200_000, "RENTAL_CASUALTY_AMOUNT", "after"),
        (2_500_000, 1_500_000, Some(20260230), 1_200_000, "RENTAL_CASUALTY_DATE", "retired"),
    ] {
        let mut session = restart(draft.clone());
        let editor = open_topic(&mut session, TaxSection::Rentals);
        let mut documents = page_documents(&editor).to_vec();
        let IncomeDocument::Rental(rental) = &mut documents[0].document else { unreachable!() };
        let casualty = &mut rental.building.casualty.as_mut().unwrap().casualty;
        casualty.kind = CasualtyKind::Damaged { before, after, retired };
        casualty.received = received;
        let accepted = session.draft.clone();
        let request = save_topic(&session.draft, TaxSection::Rentals, documents);
        let reply = send(&mut session, request);
        let Reply::Response(P::Problem { field: error_field_path, .. }) = &reply else {
            panic!("expected refusal")
        };
        assert!(error_field_path.as_ref().is_some_and(|path| path.contains(field)));
        rejected_with(reply, key);
        assert_eq!(session.draft, accepted);
    }
}

#[test]
fn recurring_ceased_loss_flows_once_through_the_existing_property_activity() {
    let draft = recurring_rental_casualty(false);
    let id = draft.income.last().unwrap().id;
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // Purchased66,000, originalJan2020 clock: allowed16,600 through
    // midDecember, casualty basis49,400, received/expected12,000 =>37,400.
    // Current rent50,000 -500 operating -2,300 depreciation =47,200
    // ScheduleE. The37,400 casualty is Form4797 ordinary, once.
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line27")), 3_740_000);
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_casualty_0_ordinary")),
        -37_400 * 100
    );
    let sources = crate::asset_sale::rental_sources(&draft, false).unwrap();
    assert_eq!(sources[0].1.ordinary, -37_400);
    assert_eq!(sources[0].1.section1231, 0);
    let FederalIncome::Rental { data } = &draft.income.last().unwrap().income else {
        unreachable!()
    };
    assert_eq!(
        crate::casualty::rental::current_depreciation(
            &data.buildings[0],
            data,
            false,
            2026,
            SourceFact::NotReported
        )
        .unwrap(),
        230_000
    );
    assert_eq!(crate::extended_rules::rental_net(data).unwrap(), 4_720_000);
    save_reopen_checked_sources(draft, &[TaxSection::Rentals], &[TaxSection::Rentals], estimate);
}

#[test]
fn recurring_casualty_joins_the_native_passive_limit_and_qbi_activity() {
    let mut draft = recurring_rental_casualty(true);
    let id = draft.income.last().unwrap().id;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.rent = 1_000_000;
    data.participation = RentalParticipation::NotActive;
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // Passive income7,200 absorbs the same source's casualty37,400.
    // No special allowance; remaining30,200 stays suspended. The casualty
    // never also reduces ScheduleE's expense net, or acquires a second QBI.
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_casualty_0_ordinary")), -720_000);
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_suspended_loss")), 3_020_000);
    assert!(!result.issues.iter().any(|issue| issue.display.key.contains("CASUALTY")));
}

#[test]
fn neutral_damage_keeps_the_known_estimate_and_ceasing_stops_at_the_event() {
    let mut draft = rental_casualty_source(CasualtyKind::Damaged {
        before: 2_500_000,
        after: 2_500_000,
        retired: None,
    });
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let source = &mut data.buildings[0].casualty.as_mut().unwrap().casualty;
    source.reimbursement = 0;
    source.received = 0;
    let current = crate::casualty::rental::current_depreciation(
        &data.buildings[0],
        data,
        false,
        2026,
        SourceFact::NotReported,
    )
    .unwrap();
    assert_eq!(current, 240_000);
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings[0].casualty.as_mut().unwrap().casualty.kind =
        CasualtyKind::Damaged { before: 2_500_000, after: 2_500_000, retired: Some(20261215) };
    assert_eq!(
        crate::casualty::rental::current_depreciation(
            &data.buildings[0],
            data,
            false,
            2026,
            SourceFact::NotReported
        )
        .unwrap(),
        230_000
    );
    assert!(estimate(&draft).tax.is_some());
}

#[test]
fn exceptional_loss_and_gain_do_not_enter_the_recurring_passive_packet() {
    let mut draft = recurring_rental_casualty(false);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings[0].casualty.as_mut().unwrap().recurring = false;
    let result = estimate(&draft);
    assert!(result.tax.is_none());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "RENTAL_CASUALTY_NONRECURRING_UNAVAILABLE")
    );
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let source = &mut data.buildings[0].casualty.as_mut().unwrap().casualty;
    source.received = 6_000_000;
    source.reimbursement = 6_000_000;
    let result = estimate(&draft);
    assert!(result.tax.is_none());
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "RENTAL_CASUALTY_GAIN_UNAVAILABLE")
    );
    // Existing rehabilitation source fixture, now shared with its owner.
    let mut draft = recurring_rental_casualty(false);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings[0].in_service = 20250101;
    data.buildings[0].rehabilitation = Some(rehabilitation(20250101, 1_000, 66_000, vec![], true));
    assert!(matches!(items(&draft)[0].regular, SourceFact::Reported(_)));
    let result = estimate(&draft);
    assert!(result.tax.is_none());
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "RENTAL_CASUALTY_CREDIT_UNAVAILABLE")
    );
    // Unchanged actual FMV cannot extend the current owner's year coverage.
    let mut draft = rental_casualty_source(CasualtyKind::Damaged {
        before: 2_500_000,
        after: 2_500_000,
        retired: None,
    });
    draft.tax_year = 2027;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let source = &mut data.buildings[0].casualty.as_mut().unwrap().casualty;
    source.date = 20271215;
    source.reimbursement = 0;
    source.received = 0;
    let result = estimate(&draft);
    assert!(result.tax.is_none());
    assert!(!result.financial_source_scope_complete);
}

#[test]
fn prior_home_carry_is_not_released_using_room_that_ignores_the_casualty() {
    let mut draft = recurring_rental_casualty(false);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    //2026-10-19: the synthetic prior worksheets retain the original equal tax-ledger amounts.
    data.home_depreciation_carryover = crate::SourceCents::Reported { amount: 1_000_000 };
    data.amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: 1_000_000 };
    let result = estimate(&draft);
    assert!(result.tax.is_none());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "RENTAL_CASUALTY_HOME_CARRY_UNAVAILABLE")
    );
}

#[test]
fn prior_home_carry_casualty_guard_reads_each_tax_ledger_and_preserves_unknown() {
    for (regular, alternative, allowed) in [
        (
            crate::SourceCents::Reported { amount: 1_000_000 },
            crate::SourceCents::NotReported,
            [false, true],
        ),
        (
            crate::SourceCents::NotReported,
            crate::SourceCents::Reported { amount: 1_000_000 },
            [true, false],
        ),
        (crate::SourceCents::Unavailable, crate::SourceCents::NotReported, [false, true]),
        (crate::SourceCents::NotReported, crate::SourceCents::Unavailable, [true, false]),
    ] {
        let mut draft = recurring_rental_casualty(false);
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.home_depreciation_carryover = regular;
        data.amt_home_depreciation_carryover = alternative;
        for (amt, allowed) in [false, true].into_iter().zip(allowed) {
            let loss = crate::casualty::rental::property_losses(data, amt);
            if allowed {
                assert!(loss.is_ok(), "{loss:?}");
            } else {
                assert_eq!(loss, Err("RENTAL_CASUALTY_HOME_CARRY_UNAVAILABLE"));
            }
        }
    }
}
