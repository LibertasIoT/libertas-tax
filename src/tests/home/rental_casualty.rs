use super::*;
use crate::kernel::source_fact::SourceFact;
use crate::tests::support::builders::accepted_rental_casualty_items as items;

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
            answer_separate_pages(&mut draft);
            let setup = draft.setup.as_mut().unwrap();
            setup.community_property = community_arizona();
            for entry in &mut draft.income {
                entry.community = CommunityShare::Community;
            }
            let entry = draft.income.last_mut().unwrap();
            entry.community = CommunityShare::Community;
            entry.owner = crate::DocumentOwner::from(owner);
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
    ] {
        let mut session = restart(kept_rental_casualty());
        let editor = open_topic(&mut session, TaxSection::Rentals);
        let mut documents = page_documents(&editor).to_vec();
        let IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) } =
            &mut documents[0].document
        else {
            unreachable!()
        };
        let records = &mut rental.building.casualty.as_mut().unwrap().continuing_records.regular;
        let value = crate::SourceCents::Reported { amount };
        match field {
            "prior_depreciation" => records.prior_depreciation = value,
            "basis" => records.basis = value,
            _ => records.depreciation = value,
        }
        let accepted = session.draft.clone();
        let request = save_topic(&mut session, TaxSection::Rentals, documents);
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
    {
        let (before, after, retired, received, key, field) = (
            2_500_000,
            1_500_000,
            None,
            1_200_001,
            "CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT",
            "received",
        );
        let mut session = restart(draft.clone());
        let editor = open_topic(&mut session, TaxSection::Rentals);
        let mut documents = page_documents(&editor).to_vec();
        let IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) } =
            &mut documents[0].document
        else {
            unreachable!()
        };
        let casualty = &mut rental.building.casualty.as_mut().unwrap().casualty;
        casualty.kind = CasualtyKind::Damaged { before, after, retired };
        casualty.received = received;
        let accepted = session.draft.clone();
        let request = save_topic(&mut session, TaxSection::Rentals, documents);
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
    let sources = crate::asset_sale::rental_sources(
        crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft),
        ),
        false,
        None,
    )
    .unwrap();
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
fn prior_home_carry_uses_income_after_current_casualty_once() {
    let mut draft = recurring_rental_casualty(false);
    // The checklist names last year's home-use worksheet carries.
    draft.checklist.carryovers |= crate::CarryoverTopics::HOME_USE_RENTAL.0;
    let id = draft.income.last().unwrap().id;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    // Pub527 carryover limit: 47,200 current operating income less the
    // separate 37,400 casualty leaves 9,800 for prior home-use expenses.
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .home_depreciation_carryover = crate::SourceCents::Reported { amount: 1_000_000 };
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: 2_000_000 };
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    for (prefix, carry) in [("", 20_000), ("amt_", 1_020_000)] {
        assert_eq!(
            line_amount(&result, &alloc::format!("{prefix}rental_{id}_home_third_tier_allowed")),
            980_000
        );
        assert_eq!(
            line_amount(
                &result,
                &alloc::format!("{prefix}rental_{id}_home_depreciation_carryover")
            ),
            carry
        );
    }
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_casualty_0_ordinary")),
        -3_740_000
    );
    // The shared return fixture includes $60,000 wages; the rental's
    // Schedule E profit and separate casualty now offset exactly.
    assert_eq!(line_amount(&result, "agi"), 6_000_000);
    let FederalIncome::Rental { data } = &draft.income.last().unwrap().income else {
        unreachable!()
    };
    assert_eq!(crate::extended_rules::rental_net(data).unwrap(), 3_740_000);
    save_reopen_checked_sources(draft, &[TaxSection::Rentals], &[TaxSection::Rentals], estimate);
}

#[test]
fn prior_home_carry_after_casualty_keeps_operating_priority_and_passive_balance() {
    let mut draft = recurring_rental_casualty(false);
    let id = draft.income.last().unwrap().id;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.participation = RentalParticipation::NotActive;
    data.history.regular = 100_000;
    data.history.amt = 200_000;
    data.home_operating_carryover = 500_000;
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .home_depreciation_carryover = crate::SourceCents::Reported { amount: 1_000_000 };
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: 1_000_000 };
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_home_operating_carryover")), 0);
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_home_third_tier_allowed")),
        480_000
    );
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_home_depreciation_carryover")),
        520_000
    );
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_suspended_loss")), 100_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_rental_{id}_suspended_loss")), 200_000);
}

#[test]
fn prior_home_carry_after_casualty_preserves_each_unavailable_tax_ledger() {
    for (regular, alternative) in [
        (crate::SourceCents::Unavailable, crate::SourceCents::NotReported),
        (crate::SourceCents::NotReported, crate::SourceCents::Unavailable),
    ] {
        let mut draft = recurring_rental_casualty(false);
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        let amt_unavailable = matches!(alternative, crate::SourceCents::Unavailable);
        data.prior_home_carry
            .get_or_insert(crate::RentalHomeCarries::NONE)
            .home_depreciation_carryover = regular;
        data.prior_home_carry
            .get_or_insert(crate::RentalHomeCarries::NONE)
            .amt_home_depreciation_carryover = alternative;
        for amt in [false, true] {
            let mut output = FederalResult::empty(0);
            let result = crate::extended_rules::rentals_for(&draft, 0, &mut output, amt);
            assert_eq!(result.source_issue.is_some(), amt == amt_unavailable);
        }
    }
    let mut draft = recurring_rental_casualty(false);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .home_depreciation_carryover = crate::SourceCents::Reported { amount: 1_000_000 };
    data.prior_home_carry.get_or_insert(crate::RentalHomeCarries::NONE).home_casualty_carryover =
        crate::SourceCents::Unavailable;
    let result = estimate(&draft);
    assert!(result.tax.is_none());
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.display.key == "RENTAL_HOME_PRIOR_CHARACTER_UNAVAILABLE")
        .unwrap();
    assert_eq!(
        issue.context.identity(),
        crate::federal_model::NoteIdentity::Income(draft.income.last().unwrap().id)
    );
}

#[test]
fn standard_home_current_casualty_uses_income_room_once_and_keeps_character() {
    let draft = standard_home_casualty();
    let id = draft.income.last().unwrap().id;
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // Whole source loss12,000; rental6,000. Rent2,000 minus operating500
    // admits1,500 casualty and no depreciation. Only the latter belongs onE.
    for (suffix, expected) in [
        ("line18_depreciation", 0),
        ("line20_total_expenses", 50_000),
        ("line21_income", 150_000),
        ("casualty_0_line27", 150_000),
        ("casualty_0_ordinary", -150_000),
        ("home_casualty_allowed", 150_000),
        ("home_casualty_carryover", 450_000),
        //2026-10-20: Pub527 Worksheet5-1 carries the printed921depreciation,
        // with4500casualty; source annual recovery stays1842.94.
        ("home_depreciation_carryover", 542_100),
    ] {
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_{suffix}")),
            expected,
            "{suffix}"
        );
    }
    let source = items(&draft).remove(0);
    let facts = established(&source.regular);
    assert_eq!(facts.raw_loss_cents, 600_000);
    assert_eq!(facts.admitted_loss_cents, SourceFact::Reported(150_000));
    assert_eq!(facts.continuing_basis_cents, SourceFact::Reported(3_740_000));
    assert_eq!(line_amount(&result, "agi"), 6_000_000);
    let saved = save_reopen_checked_sources(
        draft,
        &[TaxSection::Rentals],
        &[TaxSection::Rentals],
        estimate,
    );
    let saved = estimate(&saved);
    assert!(saved.tax.is_some());
    assert_eq!(line_amount(&saved, &alloc::format!("rental_{id}_casualty_0_line27")), 150_000);
}

#[test]
fn standard_home_current_depreciation_precedes_prior_casualty_with_independent_amt_basis() {
    let mut draft = standard_home_casualty();
    let id = draft.income.last().unwrap().id;
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.rent = 500_000;
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .home_depreciation_carryover = SourceCents::Reported { amount: 50_000 };
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_depreciation_carryover = SourceCents::Reported { amount: 50_000 };
    data.prior_home_carry.get_or_insert(crate::RentalHomeCarries::NONE).home_casualty_carryover =
        SourceCents::Reported { amount: 20_000 };
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_casualty_carryover = SourceCents::Reported { amount: 20_000 };
    data.buildings[0].casualty.as_mut().unwrap().continuing_records.alternative =
        rental_casualty_basis(600_000, 6_000_000, 5_900_000);
    let sources = items(&draft);
    assert_eq!(established(&sources[0].regular).admitted_loss_cents, SourceFact::Reported(450_000));
    assert_eq!(
        established(&sources[0].alternative).admitted_loss_cents,
        SourceFact::Reported(300_000)
    );
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_line18_depreciation")), 0);
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_home_depreciation_carryover")),
        292_100
    );
    assert_eq!(
        line_amount(&result, &alloc::format!("amt_rental_{id}_home_depreciation_carryover")),
        0
    );
    assert_eq!(
        line_amount(&result, &alloc::format!("amt_rental_{id}_home_casualty_allowed")),
        320_000
    );
    assert_eq!(line_amount(&result, &alloc::format!("amt_rental_{id}_casualty_0_line27")), 300_000);
}

#[test]
fn standard_home_casualty_requires_actual_deduction_and_source_context() {
    for case in 0..7 {
        let mut draft = standard_home_casualty();
        if case == 0 {
            let ded = draft.deductions.as_mut().unwrap();
            ded.itemize_override = Some(ItemizeOverride::Itemize);
        }
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        match case {
            1 => {
                data.buildings[0].casualty.as_mut().unwrap().casualty.declaration =
                    CasualtyDeclaration::MajorDisaster { reference: "DR-1234".into() }
            }
            2 => {
                data.buildings[0].casualty.as_mut().unwrap().continuing_records.regular.basis =
                    SourceCents::Unavailable
            }
            3 => {
                data.prior_home_carry
                    .get_or_insert(crate::RentalHomeCarries::NONE)
                    .home_depreciation_carryover = SourceCents::Reported { amount: 50_000 };
                data.prior_home_carry
                    .get_or_insert(crate::RentalHomeCarries::NONE)
                    .home_casualty_carryover = SourceCents::Unavailable;
            }
            4 => {
                data.qbi_business = RentalBusiness::Yes { material_participation: false };
                data.prior_home_carry
                    .get_or_insert(crate::RentalHomeCarries::NONE)
                    .home_depreciation_carryover = SourceCents::Reported { amount: 50_000 };
                data.prior_home_carry
                    .get_or_insert(crate::RentalHomeCarries::NONE)
                    .home_casualty_carryover = SourceCents::Reported { amount: 20_000 };
                data.rent = 1_000_000;
            }
            _ => {}
        }
        if case == 3 {
            // Unused prior character does not prevent the known current result.
            assert!(estimate(&draft).tax.is_some());
            let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income
            else {
                unreachable!()
            };
            data.rent = 1_000_000;
        }
        if case == 5 {
            let mut event = personal_disaster("Synthetic settled personal gain", 100_000);
            event.id = draft.next_id;
            event.properties[0].id = draft.next_id + 1;
            draft.next_id += 2;
            event.properties[0].reimbursement = 200_000;
            event.properties[0].received = 200_000;
            draft.deductions.as_mut().unwrap().personal_casualties.push(event);
        }
        if case == 6 {
            draft.deductions.as_mut().unwrap().itemize_override = None;
            // An undecided source caller may not presume the actual election.
            let claims = crate::related_forms::effective_claimants(&draft);
            let source = crate::casualty::rental::items(
                crate::related_forms::RelatedFormContext::original(&draft, &claims),
                None,
            )
            .remove(0);
            assert_eq!(established(&source.regular).admitted_loss_cents, SourceFact::Unavailable);
        }
        let result = estimate(&draft);
        if case != 6 {
            assert!(result.tax.is_none(), "case{case}: {:?}", result.issues);
        }
        let expected = match case {
            0 => Some(crate::federal_model::NoteIdentity::Income(draft.income.last().unwrap().id)),
            1 => Some(crate::federal_model::NoteIdentity::Income(draft.income.last().unwrap().id)),
            5 => Some(crate::federal_model::NoteIdentity::Income(draft.income.last().unwrap().id)),
            _ => None,
        };
        if let Some(expected) = expected {
            let key = match case {
                0 => "RENTAL_CASUALTY_PROPERTY_BINDING_UNAVAILABLE",
                1 => "RENTAL_CASUALTY_PROPERTY_BINDING_UNAVAILABLE",
                _ => "RENTAL_CASUALTY_PROPERTY_BINDING_UNAVAILABLE",
            };
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == key && issue.context.identity() == expected),
                "{key}: {:?}",
                result.issues
            );
        }
    }
}

#[test]
fn standard_home_unbound_casualty_leaves_depreciation_on_schedule_e() {
    for (edge, rent, reported_loss_cents, reimbursement) in [
        (false, 1_000_000, 600_000, 0),
        (true, 1_000_000, 600_100, 0),
        (true, 650_049, 600_000, 0),
        (true, 1_000_000, 100, 1_200_002),
        (true, 50_100, 100, 1_200_002),
    ] {
        let mut draft = standard_home_casualty();
        let id = draft.income.last().unwrap().id;
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.rent = rent;
        if edge {
            data.buildings[0].casualty.as_mut().unwrap().casualty.kind =
                CasualtyKind::Damaged { before: 2_500_101, after: 1_300_099, retired: None };
        }
        if rent == 50_100 {
            // Revised opening43.08 *24/517 gives2.00 annual recovery;
            // half is1.00. The printed casualty uses the1.00room first.
            *data.buildings[0].casualty.as_mut().unwrap().continuing_records =
                crate::RentalCasualtyRecords {
                    regular: rental_casualty_basis(1_204_310, 5_395_690, 5_395_690),
                    alternative: rental_casualty_basis(1_204_310, 5_395_690, 5_395_690),
                };
        }
        data.buildings[0].casualty.as_mut().unwrap().casualty.reimbursement = reimbursement;
        data.buildings[0].casualty.as_mut().unwrap().casualty.received = reimbursement;
        let source = items(&draft).remove(0);
        let facts = established(&source.regular);
        assert_eq!(
            facts.admitted_loss_cents,
            SourceFact::Reported(if reimbursement != 0 {
                0
            } else if edge {
                600_001
            } else {
                600_000
            })
        );
        let result = estimate(&draft);
        assert!(result.tax.is_some(), "{:?}", result.issues);
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_line18_depreciation")),
            if rent == 650_049 || rent == 50_100 { 0 } else { 92_100 }
        );
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_line21_income")),
            if rent == 50_100 {
                100
            } else if rent == 650_049 {
                600_000
            } else {
                857_900
            }
        );
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line27")),
            reported_loss_cents
        );
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_casualty_0_ordinary")),
            -reported_loss_cents
        );
        // Printed6001loss less6000room leaves1casualty carry, even
        // though the original6000.01source loss fitted6000.49native room.
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_home_casualty_carryover")),
            if rent == 650_049 { 100 } else { 0 }
        );
        if rent == 50_100 {
            assert_eq!(
                line_amount(&result, &alloc::format!("rental_{id}_home_depreciation_carryover")),
                100
            );
        }
        let printed =
            |line| line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line{line}"));
        assert_eq!(printed(23) - printed(24), printed(25));
        assert_eq!(printed(26), printed(20).min(printed(25)));
        assert!(printed(27) <= (printed(26) - printed(21)).max(0));
    }
    // Valid native sources fit exactly; rounded worksheet inputs can exceed
    // the closing ledger's integer range even though each operand still fits.
    use crate::casualty::home::{ThirdTierCarry, third_tier};
    assert!(
        third_tier(
            0,
            0,
            50,
            ThirdTierCarry { total_cents: i64::MAX - 50, casualty_cents: Some(0) },
            0
        )
        .is_ok()
    );
    assert_eq!(
        third_tier(
            0,
            100,
            100,
            ThirdTierCarry { total_cents: i64::MAX - 7, casualty_cents: Some(0) },
            0
        ),
        Err(crate::kernel::carry::LedgerError::InvalidAmount)
    );
    assert_eq!(
        third_tier(i64::MAX, 1, 0, ThirdTierCarry { total_cents: 0, casualty_cents: Some(0) }, 1),
        Err(crate::kernel::carry::LedgerError::InvalidAmount)
    );
    assert!(
        third_tier(i64::MAX, 0, 1, ThirdTierCarry { total_cents: 0, casualty_cents: Some(0) }, 1)
            .is_ok()
    );
}

#[test]
fn standard_home_casualty_source_cents_and_community_orientation_share_one_admission() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut draft = standard_home_casualty();
        draft.setup = community_w2_draft(0, 0, 0, 0).setup;
        draft.deductions.as_mut().unwrap().spouse_itemizes = Some(false);
        for entry in &mut draft.income {
            entry.community = CommunityShare::Community;
        }
        let entry = draft.income.last_mut().unwrap();
        entry.owner = crate::DocumentOwner::from(owner);
        entry.community = CommunityShare::Community;
        let id = entry.id;
        let FederalIncome::Rental { data } = &mut entry.income else { unreachable!() };
        data.rent += 99;
        data.expenses[0].amount += 99;
        let CasualtyKind::Damaged { before, after, .. } =
            &mut data.buildings[0].casualty.as_mut().unwrap().casualty.kind
        else {
            unreachable!()
        };
        *before += 200;
        *after += 101;
        // Income and property allocation each divide the source once. Then
        // rental days divide once:1000.50income minus250.25operating.
        let result = estimate(&draft);
        assert!(result.tax.is_some(), "{owner:?}: {:?}", result.issues);
        let source = items(&draft).remove(0);
        // The document owner keeps the odd source cent; the other spouse
        // receives the residual before the rental-day allocation.
        assert_eq!(
            established(&source.regular).admitted_loss_cents,
            SourceFact::Reported(if owner == Owner::Taxpayer { 75_025 } else { 75_024 })
        );
        // Pub527 Worksheet5-1 uses printed rent:1000.50 rounds1001,
        // while the other spouse's1000.49 rounds1000; operating rounds250.
        let financial = if owner == Owner::Taxpayer { 75_100 } else { 75_000 };
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line27")),
            financial
        );
        assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_line21_income")), financial);
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_home_casualty_carryover")),
            if owner == Owner::Taxpayer { 225_000 } else { 225_100 }
        );
        let printed =
            |line| line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line{line}"));
        assert_eq!(printed(23) - printed(24), printed(25));
        assert_eq!(printed(26), printed(20).min(printed(25)));
        assert!(printed(27) <= (printed(26) - printed(21)).max(0));
    }
}

// A home-use dwelling is one saved personal casualty property only through its
// rental record's saved property identity, which survives save, reopen,
// restart and reordering; the event graph then holds that one property once.
#[test]
fn shared_casualty_property_identity_survives_save_reopen_restart_and_reordering() {
    use crate::tests::cross::interview::{accepted, from_review};
    let (draft, rental, cost, event, property) = shared_casualty_return();
    // Re-saving the personal casualty page keeps the event and property IDs.
    let mut session = restart(draft);
    let Reply::Edit(page @ P::SaveDeductions { .. }) =
        from_review(&mut session, TaxSection::Deductions)
    else {
        panic!("expected the deductions editor")
    };
    accepted(&send(&mut session, page));
    let mut saved = save_reopen_checked_sources(
        session.draft,
        &[TaxSection::Rentals],
        &[TaxSection::Rentals],
        estimate,
    );
    let mut session = restart(saved.clone());
    let request = crate::tests::cross::interview::open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { casualty_properties, .. } = &request else { panic!() };
    assert_eq!(casualty_properties.iter().map(|choice| choice.id).collect::<Vec<_>>(), [property]);
    let events = &mut saved.deductions.as_mut().unwrap().personal_casualties;
    let mut other = events[0].clone();
    other.id = saved.next_id;
    other.properties[0].id = saved.next_id + 1;
    events[0]
        .properties
        .insert(0, CasualtyProperty { id: saved.next_id + 2, ..other.properties[0].clone() });
    events.insert(0, other);
    saved.next_id += 3;
    let sources = crate::source_associations::SourceAssociations::new(&saved);
    let (bound_event, bound) = sources.rental_casualty_property(rental, cost).unwrap().unwrap();
    assert_eq!((bound_event.id, bound.id), (event, property));
    let events = &saved.deductions.as_ref().unwrap().personal_casualties;
    let rows = crate::casualty::all_personal_allowances(events, 0).unwrap();
    assert_eq!(rows.iter().filter(|row| row.property == property && row.event == event).count(), 1);
}

// Unknown, misdirected or conflicting identities stay unavailable; a matching
// date or label never supplies the link.
#[test]
fn shared_casualty_property_is_never_inferred_and_conflicts_stay_unavailable() {
    use crate::source_associations::{SourceAssociationIssue, SourceAssociations};
    let (mut draft, rental, cost, event, property) = shared_casualty_return();
    fn link(draft: &mut FederalDraft) -> &mut Option<i64> {
        let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        &mut data.buildings[0].casualty_property
    }
    let resolve = |draft: &FederalDraft| {
        SourceAssociations::new(draft)
            .rental_casualty_property(rental, cost)
            .map(|found| found.map(|(event, property)| (event.id, property.id)))
    };
    *link(&mut draft) = None;
    assert_eq!(resolve(&draft), Ok(None));
    *link(&mut draft) = Some(property + 100);
    assert_eq!(resolve(&draft), Err(SourceAssociationIssue::UnknownSource(property + 100)));
    *link(&mut draft) = Some(event);
    assert_eq!(resolve(&draft), Err(SourceAssociationIssue::WrongSourceKind(event)));
    *link(&mut draft) = Some(property);
    draft.deductions.as_mut().unwrap().personal_casualties[0].declaration =
        CasualtyDeclaration::Federal { reference: "EM-1234".into() };
    assert_eq!(
        resolve(&draft),
        Err(SourceAssociationIssue::ParentMismatch { source: cost, parent: event })
    );
}

#[test]
fn itemized_home_casualty_uses_one_whole_property_and_excludes_settled_income_from_agi() {
    let (mut draft, id, _) = bound_home_casualty(false);
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.itemize_override = Some(ItemizeOverride::Itemize);
    crate::extended_rules::HOME_USE_SEEN.with(|seen| seen.borrow_mut().clear());
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // $12,000 less one $100 floor and $6,000 AGI floor gives $5,900
    // whole-property allowance; half is rental. Its $2,950 is not income
    // room for depreciation or excess casualty, nor a Schedule E expense.
    for (suffix, amount) in [
        ("home_casualty_otherwise", 295_000),
        ("casualty_0_line27", 295_000),
        ("casualty_0_ordinary", -295_000),
        ("home_casualty_allowed", 295_000),
        ("home_casualty_carryover", 305_000),
        ("line18_depreciation", 0),
        ("line21_income", 150_000),
    ] {
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    assert_eq!(line_amount(&result, "agi"), 5_855_000);
    let context = crate::extended_rules::HOME_USE_SEEN
        .with(|seen| seen.borrow().iter().rfind(|(source, _)| *source == id).unwrap().1.clone());
    assert_eq!(context.dwelling_agi_cents(id), Some(6_000_000));
    // The personal form removes the rental operands once, while the
    // accepted canonical record remains the complete dwelling.
    let personal = crate::home_sites::personal_casualties(&draft).unwrap();
    assert_eq!(personal[0].properties[0].before - personal[0].properties[0].after, 600_000);
    assert_eq!(
        draft.deductions.as_ref().unwrap().personal_casualties[0].properties[0].before,
        2_500_000
    );
    assert!(result.rental_home_context.as_ref().unwrap().itemizes.unwrap());
}

#[test]
fn qualified_home_casualty_standard_and_automatic_routes_keep_independent_amt_capacity() {
    let (mut draft, id, _) = bound_home_casualty(true);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings[0].casualty.as_mut().unwrap().continuing_records.alternative =
        rental_casualty_basis(600_000, 6_000_000, 5_900_000);
    let standard = estimate(&draft);
    assert!(standard.tax.is_some(), "{:?}", standard.issues);
    // Whole $12,000 less the single $500 floor; rental half $5,750.
    // AMT's actual business loss is $3,000, so its independent capacity
    // limits that allowance without changing the shared regular AGI floor.
    assert_eq!(
        line_amount(&standard, &alloc::format!("rental_{id}_home_casualty_otherwise")),
        575_000
    );
    assert_eq!(line_amount(&standard, &alloc::format!("rental_{id}_casualty_0_line27")), 575_000);
    assert_eq!(
        line_amount(&standard, &alloc::format!("amt_rental_{id}_home_casualty_otherwise")),
        300_000
    );
    assert_eq!(
        line_amount(&standard, &alloc::format!("amt_rental_{id}_casualty_0_line27")),
        300_000
    );
    assert_eq!(line_amount(&standard, "agi"), 5_575_000);
    assert_eq!(
        standard.rental_home_context.as_ref().unwrap().dwelling_agi_cents(id),
        Some(6_000_000)
    );
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.itemize_override = Some(ItemizeOverride::Itemize);
    let itemized_result = estimate(&draft);
    assert!(itemized_result.tax.is_some(), "{:?}", itemized_result.issues);
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.itemize_override = None;
    let automatic = estimate(&draft);
    assert!(automatic.tax.is_some(), "{:?}", automatic.issues);
    assert_eq!(
        automatic.tax.unwrap() - automatic.refundable_credits.unwrap(),
        (standard.tax.unwrap() - standard.refundable_credits.unwrap())
            .min(itemized_result.tax.unwrap() - itemized_result.refundable_credits.unwrap())
    );
    assert!(automatic.rental_home_context.as_ref().unwrap().itemizes.is_some());
    assert_eq!(line_amount(&automatic, &alloc::format!("rental_{id}_casualty_0_line27")), 575_000);
}

#[test]
fn shared_home_casualty_costs_conserve_one_property_allowance_through_native_lifecycle() {
    let (mut draft, id, property) = bound_home_casualty(false);
    split_bound_home_casualty_cost(&mut draft);
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.itemize_override = Some(ItemizeOverride::Itemize);
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    for index in 0..2 {
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_casualty_{index}_line27")),
            147_500
        );
    }
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_home_casualty_otherwise")),
        295_000
    );
    use crate::tests::cross::interview::{accepted, from_review};
    let mut session = restart(draft);
    let Reply::Edit(mut page @ P::SaveDeductions { .. }) =
        from_review(&mut session, TaxSection::Deductions)
    else {
        panic!("expected the actual Deductions editor")
    };
    let P::SaveDeductions { value, .. } = &mut page else { unreachable!() };
    value.casualties[0].properties[0].label = "Edited synthetic shared dwelling".into();
    accepted(&send(&mut session, roundtrip_native_request(page)));
    let saved = save_reopen_checked_sources(
        session.draft,
        &[TaxSection::Rentals],
        &[TaxSection::Rentals],
        estimate,
    );
    let mut reopened = restart(saved.clone());
    let Reply::Edit(P::SaveDeductions { value, .. }) =
        from_review(&mut reopened, TaxSection::Deductions)
    else {
        panic!("expected the saved Deductions editor")
    };
    assert_eq!(value.casualties[0].properties[0].id, property);
    assert_eq!(value.casualties[0].properties[0].label, "Edited synthetic shared dwelling");
    let FederalIncome::Rental { data } = &saved.income.last().unwrap().income else {
        unreachable!()
    };
    assert_eq!(
        data.buildings.iter().map(|cost| cost.casualty_property).collect::<Vec<_>>(),
        [Some(property), Some(property)]
    );
    let mut saved = restart(saved).draft;
    let FederalIncome::Rental { data } = &mut saved.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.buildings.swap(0, 1);
    let result = estimate(&saved);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    assert_eq!(
        line_amount(&result, &alloc::format!("rental_{id}_home_casualty_otherwise")),
        295_000
    );
    assert_eq!(line_amount(&result, "agi"), 5_855_000);
}

#[test]
fn home_casualty_shared_event_reduction_and_unavailable_sources_use_canonical_records() {
    let (mut draft, id, _) = bound_home_casualty(false);
    let event = &mut draft.deductions.as_mut().unwrap().personal_casualties[0];
    let mut other = personal_disaster("Other actual property", 1_200_000).properties.remove(0);
    other.id = draft.next_id;
    draft.next_id += 1;
    event.properties.push(other);
    let FederalIncome::Rental { data } = &draft.income.last().unwrap().income else {
        unreachable!()
    };
    let losses = crate::casualty::rental::property_losses(data, false).unwrap();
    // Two equal $12,000 losses share one $100 event floor and the $6,000
    // AGI floor: $8,950 per property, of which half this dwelling is rental.
    assert_eq!(
        crate::casualty::rental_home::allowances(&draft, id, data, &losses, false, true, 6_000_000)
            .unwrap(),
        [447_500]
    );
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.itemize_override = Some(ItemizeOverride::Itemize);
    for incomplete in [true, false] {
        let mut trial = draft.clone();
        if incomplete {
            trial.deductions.as_mut().unwrap().personal_casualties[0].source_complete = false;
        } else {
            duplicate_bound_home_casualty_cost(&mut trial);
        }
        let result = estimate(&trial);
        assert!(result.tax.is_none(), "{incomplete}: {:?}", result.issues);
        let key = if incomplete {
            "RENTAL_CASUALTY_PERSONAL_ALLOWANCE_UNAVAILABLE"
        } else {
            "RENTAL_CASUALTY_PROPERTY_BINDING_UNAVAILABLE"
        };
        assert!(result.issues.iter().any(|issue| issue.display.key == key), "{:?}", result.issues);
    }
}

#[test]
fn linked_home_casualty_ownership_precedes_shared_floors_and_personal_split() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let (mut draft, id, _) = bound_home_casualty(true);
        draft.setup = community_w2_draft(0, 0, 0, 0).setup;
        draft.deductions.as_mut().unwrap().spouse_itemizes = Some(false);
        for entry in &mut draft.income {
            entry.community = CommunityShare::Community;
        }
        draft.income.last_mut().unwrap().owner = crate::DocumentOwner::from(owner);
        let result = estimate(&draft);
        assert!(result.tax.is_some(), "{owner:?}: {:?}", result.issues);
        // Actual filer ownership halves the $12,000 source loss BEFORE its
        // one $500 floor: $5,500 whole-filer hypothetical, half rental $2,750.
        // Applying ownership after the floor would wrongly produce $2,875.
        assert_eq!(
            line_amount(&result, &alloc::format!("rental_{id}_home_casualty_otherwise")),
            275_000
        );
        assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line27")), 275_000);
        assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_line21_income")), 75_000);
        assert_eq!(line_amount(&result, "agi"), 2_800_000);
        let personal = crate::home_sites::personal_casualties(&draft).unwrap();
        assert_eq!(personal[0].properties[0].before - personal[0].properties[0].after, 300_000);
        assert_eq!(crate::casualty::section_a(&personal, 28_000).qualified_net, 2_500);
        assert_eq!(
            result.rental_home_context.as_ref().unwrap().dwelling_agi_cents(id),
            Some(3_000_000)
        );
    }
}

#[test]
fn settled_current_and_prior_home_casualties_leave_the_same_hypothetical_agi() {
    let (mut draft, id, _) = bound_home_casualty(true);
    let FederalIncome::Rental { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.rent = 1_000_000;
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .home_depreciation_carryover = SourceCents::Reported { amount: 50_000 };
    data.prior_home_carry.get_or_insert(crate::RentalHomeCarries::NONE).home_casualty_carryover =
        SourceCents::Reported { amount: 20_000 };
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_depreciation_carryover = SourceCents::Reported { amount: 30_000 };
    data.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_casualty_carryover = SourceCents::Reported { amount: 10_000 };
    let result = estimate(&draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // Line2c $5,750 plus current excess $250, depreciation $921, then
    // actual prior $200 casualty/$300 depreciation; no combined carry is
    // relabeled as depreciation or counted in the hypothetical AGI.
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_casualty_0_line27")), 600_000);
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_home_casualty_allowed")), 620_000);
    assert_eq!(
        line_amount(&result, &alloc::format!("amt_rental_{id}_home_casualty_allowed")),
        610_000
    );
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_line18_depreciation")), 122_100);
    assert_eq!(line_amount(&result, &alloc::format!("rental_{id}_line21_income")), 827_900);
    assert_eq!(line_amount(&result, "agi"), 6_207_900);
    assert_eq!(
        result.rental_home_context.as_ref().unwrap().dwelling_agi_cents(id),
        Some(6_000_000)
    );
}
