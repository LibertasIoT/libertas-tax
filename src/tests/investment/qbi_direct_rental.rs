// i8995, Determining QBI: section1231 enters QBI only as ordinary character.
// Independent sale basis/rounding witness lives in the shared source builder.
use super::*;

#[test]
fn direct_rental_property_qbi_uses_native_character_once() {
    for (lookback, ordinary, capital, deduction) in
        [(0, 0, 20_000, 0), (5_000, 5_000, 15_000, 1_000), (20_000, 20_000, 0, 4_000)]
    {
        let draft = qualified_rental_sale(522_000, Some(lookback));
        let result = supported(&draft);
        assert_eq!(value(&result, "agi"), 80_000);
        assert_eq!(value(&result, "rental_income"), 0);
        assert_eq!(value(&result, "form4797_line13"), ordinary);
        assert_eq!(value(&result, "capital_gain"), capital);
        assert_eq!(value(&result, "qbi_deduction"), deduction);
        let mut scratch = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&draft, 6_000_000, &mut scratch, false);
        assert_eq!(pool.activities.len(), 1);
        assert_eq!(pool.qbi_source_ids, vec![draft.income[1].id]);
        assert_eq!(pool.activities[0].income, 0);
        assert_eq!(pool.activities[0].wages, 2_000);
        // Sold in August: this property is not held at year end for UBIA.
        assert_eq!(pool.activities[0].property, 0);
        assert_eq!(
            crate::section1231::rental_property_qbi(&draft, &pool).unwrap().current,
            if ordinary == 0 { vec![] } else { vec![(0, ordinary, None)] }
        );

        // Establishing QBI changes no recognition, recapture or NIIT amount.
        let mut unqualified = draft.clone();
        let FederalIncome::Rental { data } = &mut unqualified.income[1].income else {
            unreachable!()
        };
        data.qbi_business = RentalBusiness::No;
        let baseline = supported(&unqualified);
        for line in [
            "agi",
            "rental_income",
            "form4797_line8",
            "form4797_line13",
            "capital_gain",
            "net_investment_income_tax",
        ] {
            assert_eq!(value(&result, line), value(&baseline, line), "{line}");
        }
    }
}

#[test]
fn direct_rental_ordinary_loss_retains_qualified_loss_carry() {
    let result = supported(&qualified_rental_sale(492_000, Some(0)));
    assert_eq!(value(&result, "agi"), 50_000);
    assert_eq!(value(&result, "form4797_line8"), -10_000);
    assert_eq!(value(&result, "rental_income"), 0);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 10_000);
}

#[test]
fn direct_rental_unknown_lookback_is_not_known_empty_history() {
    let unknown = qualified_rental_sale(522_000, None);
    let result = estimate(&unknown);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.issues.iter().any(|issue| issue.display.key == "ISSUE_SECTION1231_ADD__BUSINESS_PROPERTY_LOSSES_FROM_PRIOR_FIVE_YEARS__ONCE_F5B687DD"), "{:?}", result.issues);
    assert_eq!(FederalEstimate::from(result).tax, None);
    let known_empty = supported(&qualified_rental_sale(522_000, Some(0)));
    assert_eq!(value(&known_empty, "capital_gain"), 20_000);
    assert_eq!(value(&known_empty, "qbi_deduction"), 0);
}

#[test]
fn direct_rental_qbi_uses_only_the_allowed_current_property_loss() {
    let mut draft = qualified_rental_sale(492_000, Some(0));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.at_risk = simplified_rental_risk(500_000, 0, 0, [0; 4]);
    let result = supported(&draft);
    // $10,000 current loss, $5,000 amount at risk: only $5,000 enters QBI.
    assert_eq!(value(&result, "form4797_line8"), -5_000);
    assert_eq!(value(&result, "agi"), 55_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 5_000);
}

#[test]
fn direct_rental_qbi_keeps_unknown_original_property_qualification_explicit() {
    let mut draft = qualified_rental_sale(522_000, Some(20_000));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.at_risk = simplified_rental_risk(60_000_000, 0, 0, [0, 0, 100_000, 100_000]);
    let before = draft.clone();
    let result = estimate(&draft);
    assert_eq!(draft, before);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "ACTIVITY_QBI_HISTORY_UNAVAILABLE")
    );
    assert_eq!(FederalEstimate::from(result).tax, None);
}

#[test]
fn direct_rental_mixed_building_does_not_invent_business_depreciation_history() {
    let mut draft = qualified_rental_sale(522_000, Some(20_000));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    let RentalSale::Sold { personal_share_percent, .. } = &mut data.sale else { unreachable!() };
    *personal_share_percent = 10;
    let before = draft.clone();
    let result = estimate(&draft);
    assert_eq!(draft, before);
    assert_eq!(result.state, ResultState::Unsupported);
    let issues: Vec<_> = result
        .issues
        .iter()
        .filter(|issue| issue.display.key == "RENTAL_BUILDING_SALE_HISTORY_UNAVAILABLE")
        .collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].context, crate::interview::income_issue_context(&draft.income[1]));
    assert_eq!(FederalEstimate::from(result).tax, None);
}

#[test]
fn missing_mixed_building_history_review_keeps_each_source_once_after_native_restart() {
    use crate::tests::support::client::income_pages::{open_topic, page_documents, save_topic};
    for count in [1, 2] {
        let mut draft = mixed_rental_with_actual_sale_records();
        let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
        data.buildings[0].sale_history = None;
        if count == 2 {
            let second = draft.income[1].clone();
            add(&mut draft, second);
        }
        let mut session = Interview::new(TEST_USER, draft);
        let page = open_topic(&mut session, TaxSection::Rentals);
        let items = page_documents(&page).to_vec();
        let request = save_topic(&mut session, TaxSection::Rentals, items.clone());
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        let saved = session.draft.clone();
        let reopened = open_topic(&mut session, TaxSection::Rentals);
        assert_eq!(page_documents(&reopened), items);
        let mut again = restart(saved.clone());
        let reopened = open_topic(&mut again, TaxSection::Rentals);
        assert_eq!(page_documents(&reopened), items);
        let result = estimate(&again.draft);
        assert_eq!(result.state, ResultState::Unsupported);
        assert_eq!(result.tax, None);
        assert!(result.lines.is_empty());
        let issues: Vec<_> = result
            .issues
            .iter()
            .filter(|issue| issue.display.key == "RENTAL_BUILDING_SALE_HISTORY_UNAVAILABLE")
            .collect();
        assert_eq!(issues.len(), count);
        let sources: Vec<_> = again
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
            .map(crate::interview::income_issue_context)
            .collect();
        assert_eq!(sources.len(), count);
        for source in sources {
            assert_eq!(issues.iter().filter(|issue| issue.context == source).count(), 1);
        }
        let Reply::Response(P::Review { remaining_items, result: reviewed, page, .. }) =
            send(&mut again, P::ReviewAccepted { user: TEST_USER })
        else {
            panic!("expected accepted-return Review")
        };
        assert_eq!(reviewed.tax, None);
        for issue in issues {
            let status = crate::interview::contextual_message(
                &issue.display,
                &crate::interview::resolved_issue_context(&saved, &issue.context),
            )
            .encode();
            let rows: Vec<_> =
                remaining_items.iter().filter(|item| item.status == status).collect();
            // The two records deliberately have the same display name. Their
            // identities still produce separate, identical-looking Review rows.
            assert_eq!(rows.len(), count);
            assert!(rows.iter().all(|row| row.section == TaxSection::Rentals));
        }
        let request = P::OpenSection {
            user: TEST_USER,
            cookie: again.draft.cookie.clone(),
            revision: again.draft.revision,
            from: page,
            section: TaxSection::Rentals,
            reveal_path: None,
        };
        let Reply::Edit(opened) = send(&mut again, request) else {
            panic!("the source issue must open its native rental editor")
        };
        assert_eq!(page_documents(&opened), items);
        assert_eq!(again.draft, saved);
    }
}

#[test]
fn known_capital_property_does_not_need_a_second_qbi_activity_target() {
    let mut draft = qualified_rental_sale(522_000, Some(0));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.history.regular = 100_000;
    data.history.amt = 100_000;
    data.sale = RentalSale::Sold {
        date: 20260815,
        price: 52_200_000,
        land_price: 0,
        selling_expenses: 0,
        payment: SalePayment::Installment {
            assumed_debt: 0,
            payments: 26_100_000,
            interest: NoteInterest::Adequate,
            special: false,
        },
        buyer: Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let result = supported(&draft);
    // Half of $20,000 capital gain and half of $1,000 suspended loss.
    assert_eq!(value(&result, "agi"), 69_500);
    assert_eq!(value(&result, "capital_gain"), 10_000);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 500);
}

#[test]
fn controlled_entity_rental_gain_joins_ordinary_qbi_once() {
    let mut draft = qualified_rental_sale(522_000, Some(0));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    let RentalSale::Sold { buyer, .. } = &mut data.sale else { unreachable!() };
    *buyer = Buyer::ControlledEntity;
    let result = supported(&draft);
    // Section1239: all $20,000 depreciable-building gain is ordinary.
    assert_eq!(value(&result, "agi"), 80_000);
    assert_eq!(value(&result, "capital_gain"), 0);
    assert_eq!(value(&result, "rental_income"), 0);
    assert_eq!(value(&result, "qbi_deduction"), 4_000);
}

#[test]
fn direct_rental_equipment_recapture_is_ordinary_qbi_once() {
    let draft = qualified_rental_with_recapture();
    let result = supported(&draft);
    // Fully recovered $1,000 equipment sold for $1,000: all gain ordinary.
    // Separate $20,000 building gain remains capital; QBI is $1,000.
    assert_eq!(value(&result, "agi"), 81_000);
    assert_eq!(value(&result, "capital_gain"), 20_000);
    assert_eq!(value(&result, "rental_income"), 0);
    assert_eq!(value(&result, "qbi_deduction"), 200);
}

#[test]
fn kept_rental_equipment_recapture_reaches_tax_and_qbi_once() {
    let mut draft = qualified_rental_with_recapture();
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.sale = RentalSale::No;
    data.rent = 2_400_000;
    // Whole-year building recovery $24,000 equals rent; equipment's
    // $1,000 ordinary gain is the sole nonwage income, not another rent.
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 61_000);
    assert_eq!(value(&result, "capital_gain"), 0);
    assert_eq!(value(&result, "rental_income"), 0);
    assert_eq!(value(&result, "qbi_deduction"), 200);
}

#[test]
fn rental_risk_cut_preserves_positive_ordinary_and_limits_negative_property_class() {
    let mut draft = qualified_rental_with_recapture();
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    let RentalSale::Sold { price, .. } = &mut data.sale else { unreachable!() };
    *price = 49_200_000;
    data.at_risk = simplified_rental_risk(500_000, 0, 0, [0; 4]);
    let result = supported(&draft);
    // $1,000 positive ordinary gain remains. $5,000 opening risk plus
    // that gain permits $6,000 of the $10,000 section1231 loss.
    assert_eq!(value(&result, "agi"), 55_000);
    assert_eq!(value(&result, "form4797_line8"), -6_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 5_000);
    assert_eq!(value(&result, "rental_2_sale_building"), -6_000);
    assert_eq!(value(&result, "rental_2_asset_0_disposal_1245_recapture"), 1_000);
    assert!(!result.lines.iter().any(|line| line.id == "rental_2_sale_at_risk_adjustment"));
    let mut scratch = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(&draft, 6_000_000, &mut scratch, false);
    let sources = crate::asset_sale::allowed_rental_sources(
        crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft),
        ),
        &pool,
        false,
    )
    .unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(
        (sources[0].1.ordinary, sources[0].1.section1231, sources[0].1.gain),
        (1_000, -6_000, -5_000)
    );
}

#[test]
fn kept_rental_passive_cut_withholds_current_asset_loss_without_creating_qbi_carry() {
    let draft = qualified_rental_with_passive_asset_loss();
    let result = supported(&draft);
    // Purchased/placed/sold in2026: no MACRS/bonus; $1,000 basis minus
    // $500 proceeds is ordinary loss. No passive income/active allowance.
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "rental_income"), 0);
    assert_eq!(value(&result, "rental_2_suspended_loss"), 500);
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    let mut scratch = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(&draft, 6_000_000, &mut scratch, false);
    let sources = crate::asset_sale::allowed_rental_sources(
        crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft),
        ),
        &pool,
        false,
    )
    .unwrap();
    assert_eq!((sources[0].1.ordinary, sources[0].1.section1231, sources[0].1.gain), (0, 0, 0));
}

#[test]
fn rental_asset_regular_and_amt_disposals_keep_actual_prior_basis_and_rounding() {
    let draft = qualified_rental_with_amt_recapture();
    let result = supported(&draft);
    // Pub946 manual SL switch/halfyear: unrecovered regular98,304c /
    // remaining1.5years /2 =32,768c. AMT131,092c /1.5 /2 =43,697.33c.
    // Sale100,000c yields regular34,464c→345 and AMT12,605.33c→126.
    assert_eq!(value(&result, "agi"), 80_017);
    // 2026-10-19, D-2026-10-19-01: exact cents remain inside the owner.
    assert_eq!(line_amount(&result, "rental_2_line18_depreciation"), 1_532_768);
    assert_eq!(displayed_line_amount(&result, "rental_2_line18_depreciation"), 1_532_800);
    assert_eq!(value(&result, "capital_gain"), 20_000);
    assert_eq!(value(&result, "qbi_deduction"), 3);
    assert_eq!(value(&result, "amt_disposition_adjustment"), -219);
    for (amt, ordinary) in [(false, 345), (true, 126)] {
        let mut scratch = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&draft, 6_000_000, &mut scratch, amt);
        let sources = crate::asset_sale::allowed_rental_sources(
            crate::related_forms::RelatedFormContext::original(
                &draft,
                &crate::related_forms::effective_claimants(&draft),
            ),
            &pool,
            amt,
        )
        .unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!((sources[0].1.ordinary, sources[0].1.section1231), (ordinary, 20_000));
    }
}

#[test]
fn rental_asset_personal_gain_stays_capital_outside_business_qbi() {
    let draft = qualified_rental_with_personal_asset_gain();
    let result = supported(&draft);
    // $2,000 price: business proceeds1,000/basis0 gives500recapture +
    // 500section1231; personal proceeds1,000 minus500basis gives500longgain.
    // The separate building's20,000gain remains capital.
    assert_eq!(value(&result, "agi"), 81_500);
    assert_eq!(value(&result, "capital_gain"), 21_000);
    assert_eq!(value(&result, "rental_income"), 0);
    assert_eq!(value(&result, "qbi_deduction"), 100);
}

#[test]
fn rental_property_prior_half_dollar_stays_exact_until_joint_projection() {
    let mut draft = qualified_rental_sale(502_100, Some(0));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.at_risk = simplified_rental_risk(60_000_000, 0, 0, [0, 0, 50, 50]);
    let mut scratch = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(&draft, 6_000_000, &mut scratch, false);
    let sources = crate::asset_sale::allowed_rental_sources(
        crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft),
        ),
        &pool,
        false,
    )
    .unwrap();
    // Current gain100d less actual prior50c =99.50d, reported jointly100.
    // Rounding the prior to1d first would invent a missing dollar/class cut.
    assert_eq!(sources.len(), 1);
    assert_eq!((sources[0].1.ordinary, sources[0].1.section1231, sources[0].1.gain), (0, 100, 100));
    // The report uses the same native joint: no separately rounded dollar
    // may appear as a fictitious prior-loss adjustment.
    let mut result = FederalResult::empty(0);
    let FederalIncome::Rental { data } = &draft.income[1].income else { unreachable!() };
    crate::asset_sale::report_rental(
        &draft,
        &mut result,
        draft.income[1].id,
        "Synthetic rental",
        data,
    );
    crate::asset_sale::report_rental_partial_limits(
        crate::related_forms::RelatedFormContext::original(
            &draft,
            &crate::related_forms::effective_claimants(&draft),
        ),
        &pool,
        &mut result,
    );
    assert_eq!(value(&result, "rental_2_sale_building_gain"), 100);
    assert!(!result.lines.iter().any(|line| line.id == "rental_2_sale_at_risk_adjustment"));
    // This native report probe does not claim a complete return while the
    // independently required original QBI history remains unanswered.
    assert!(crate::section1231::rental_property_qbi(&draft, &pool).is_err());
}

#[test]
fn condemned_short_rental_reports_only_native_recognized_ordinary_gain() {
    let draft = qualified_condemned_short_rental();
    let FederalIncome::Rental { data } = &draft.income[1].income else { unreachable!() };
    let native = crate::asset_sale::rental_sale(data, false).unwrap().unwrap();
    // Section1033 recognizes award2,000 minus replacement1,500, not
    // the entire realized gain on the1,000-cost depreciated building.
    assert!(native.short_rows[0].gain > 500);
    assert_eq!(native.recognized, 500);
    let result = supported(&draft);
    assert_eq!(value(&result, "rental_2_sale_row_0_ordinary"), 500);
    assert_eq!(value(&result, "capital_gain"), 0);
}

#[test]
fn rental_section179_conversion_reaches_operating_income_and_qbi_once() {
    // Form4797 instructions PartIV: recapture is other income on the
    // original deduction's schedule. See https://www.irs.gov/instructions/i4797.
    let draft = qualified_rental_with_section179_conversion();
    let result = supported(&draft);
    //1,000 expensed minus(200 prior +160 conversion-year) =640.
    // Kept building's24,000 rent and24,000 depreciation cancel.
    assert_eq!(value(&result, "rental_2_asset_0_section179_prior"), 1_000);
    assert_eq!(value(&result, "rental_2_asset_0_section179_recomputed"), 360);
    assert_eq!(value(&result, "rental_2_asset_0_section179_recapture"), 640);
    assert_eq!(value(&result, "rental_2_line3_rents"), 24_640);
    assert_eq!(value(&result, "rental_2_line20_total_expenses"), 24_000);
    assert_eq!(value(&result, "rental_income"), 640);
    assert_eq!(value(&result, "agi"), 60_640);
    assert_eq!(value(&result, "qbi_deduction"), 128);
    assert_eq!(value(&result, "capital_gain"), 0);
}

#[test]
fn rental_prior_property_fifo_keeps_original_qualification_after_current_business_changes() {
    // i8995 QBI loss-tracking: $5k released, first3kpre2018 then2k*60%=1.2k.
    for qualified_now in [true, false] {
        let draft = rental_with_prior_property_history(qualified_now);
        let result = supported(&draft);
        assert_eq!(value(&result, "agi"), 55_000);
        assert_eq!(value(&result, "form4797_line8"), -5_000);
        assert_eq!(value(&result, "qbi_loss_carryforward"), 1_200);
    }
}

#[test]
fn rental_current_and_historical_property_losses_keep_separate_qualified_parts() {
    let mut draft = rental_with_prior_property_history(true);
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.sale = RentalSale::Sold {
        date: 20260815,
        price: 49_200_000,
        land_price: 0,
        selling_expenses: 0,
        payment: SalePayment::InFull,
        buyer: Buyer::Unrelated,
        personal_share_percent: 0,
    };
    data.rent = 1_500_000;
    data.at_risk = simplified_rental_risk(1_000_000, 0, 0, [0, 0, 1_000_000, 800_000]);
    let result = supported(&draft);
    // $10k current and$10k prior share$10k risk capacity:5k each.
    // Current qualified5k plus FIFOhistorical1.2k; wages/UBIA are not duplicated.
    assert_eq!(value(&result, "agi"), 50_000);
    assert_eq!(value(&result, "rental_2_sale_building"), -5_000);
    assert_eq!(value(&result, "rental_2_sale_at_risk_adjustment"), -5_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 6_200);
}

#[test]
fn rental_property_history_releases_at_risk_before_partial_passive_allowance() {
    let mut draft = rental_with_prior_property_history(true);
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.participation = RentalParticipation::NotActive;
    let mut other = rental(12_500, 0);
    other.buildings = vec![purchased_rental_building(27_500_000, 0, 20200101)];
    other.participation = RentalParticipation::NotActive;
    add(&mut draft, income(FederalIncome::Rental { data: other }));
    let result = supported(&draft);
    // Other rental profit2500 allows half the risk-released5000. Historical
    // QBI1200 is reduced proportionately at PAL to600, not FIFO-released again.
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "form4797_line8"), -2_500);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 600);
}

#[test]
fn rental_property_withheld_losses_do_not_require_unused_qbi_history() {
    for withheld_by_risk in [true, false] {
        let mut draft = rental_with_prior_property_history(true);
        let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
        data.property_qbi_history.clear();
        data.participation = RentalParticipation::NotActive;
        if withheld_by_risk {
            data.at_risk = simplified_rental_risk(0, 0, 0, [0, 0, 1_000_000, 800_000]);
        }
        let result = supported(&draft);
        assert_eq!(value(&result, "agi"), 60_000);
        assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
    }
}

#[test]
fn rental_history_unknown_qualified_amount_does_not_become_zero() {
    let mut draft = rental_with_prior_property_history(false);
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.property_qbi_history[0].qualified = crate::SourceCents::Unavailable;
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "ACTIVITY_QBI_HISTORY_UNAVAILABLE")
    );
    assert_eq!(FederalEstimate::from(result).tax, None);
}

#[test]
fn mixed_rental_building_uses_actual_business_basis_and_regular_amt_depreciation() {
    let draft = mixed_rental_with_actual_sale_records();
    let result = supported(&draft);
    // Business proceeds469800 less(550000-100000)=19800; the recorded
    // section1231 lookback makes that gain ordinary. The unrelated buyer
    // preserves the separately computed AMT loss (Pub544, related parties).
    // Personal proceeds52200 minus actual residual cost110000 is nondeductible.
    assert_eq!(value(&result, "agi"), 79_800);
    assert_eq!(value(&result, "capital_gain"), 0);
    assert_eq!(value(&result, "qbi_deduction"), 3_960);
    assert_eq!(value(&result, "rental_2_sale_building_gain"), 19_800);
    assert_eq!(value(&result, "amt_disposition_adjustment"), -20_000);
    let FederalIncome::Rental { data } = &draft.income[1].income else { unreachable!() };
    for (amt, gain, depreciation) in [(false, 19_800, 100_000), (true, -200, 80_000)] {
        let native = crate::asset_sale::rental_sale(data, amt).unwrap().unwrap();
        assert_eq!(
            (native.recognized, native.basis, native.depreciation, native.personal_basis),
            (gain, 550_000, depreciation, 110_000)
        );
    }
}

#[test]
fn mixed_building_unknown_amt_history_is_not_copied_from_regular() {
    let mut draft = mixed_rental_with_actual_sale_records();
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.buildings[0].sale_history.as_mut().unwrap().amt_depreciation =
        crate::SourceCents::Unavailable;
    assert!(crate::asset_sale::rental_sale(data, false).unwrap().is_some());
    assert!(crate::asset_sale::rental_sale(data, true).is_err());
    assert_eq!(FederalEstimate::from(estimate(&draft)).tax, None);
}

#[test]
fn rental_original_property_history_is_bound_to_its_actual_source() {
    let mut draft = rental_with_prior_property_history(true);
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    let other = data.clone();
    data.property_qbi_history.clear();
    add(&mut draft, income(FederalIncome::Rental { data: other }));
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "ACTIVITY_QBI_HISTORY_UNAVAILABLE")
    );
}

#[test]
fn rental_property_history_cents_reconcile_against_the_native_prior_row() {
    let mut draft = qualified_rental_sale(502_100, Some(100));
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.at_risk = simplified_rental_risk(60_000_000, 0, 0, [0, 0, 50, 50]);
    data.property_qbi_history = vec![rental_property_loss_year(2025, 50, 50, 50, 50)];
    let result = supported(&draft);
    // Current100d less actual50c remains native100d. Its prior native row is
    // zero, so history creates neither a spurious deduction nor suspension.
    assert_eq!(value(&result, "agi"), 60_100);
    assert_eq!(value(&result, "qbi_deduction"), 20);
    assert!(!result.lines.iter().any(|line| line.id == "rental_2_sale_at_risk_adjustment"));
}

#[test]
fn rental_source_history_survives_native_save_reopen_restart_and_rejects_invalid_correction() {
    use crate::tests::support::assert::rejected_with;
    use crate::tests::support::client::income_pages::{open_topic, page_documents, save_topic};
    for draft in [rental_with_prior_property_history(true), mixed_rental_with_actual_sale_records()]
    {
        let mut session = Interview::new(TEST_USER, draft.clone());
        let page = open_topic(&mut session, TaxSection::Rentals);
        let items = page_documents(&page).to_vec();
        let request = save_topic(&mut session, TaxSection::Rentals, items.clone());
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(session.draft.income[1], draft.income[1]);
        let mut again = restart(session.draft.clone());
        let reopened = open_topic(&mut again, TaxSection::Rentals);
        assert_eq!(page_documents(&reopened), items);
        let mut invalid = items;
        let IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) } =
            &mut invalid[0].document
        else {
            unreachable!()
        };
        if rental.building.sale_history.is_some() {
            continue;
        }
        rental.more.property_qbi_history[0].remaining += 1;
        let before = again.draft.clone();
        let reply = {
            let request = save_topic(&mut again, TaxSection::Rentals, invalid);
            send(&mut again, request)
        };
        rejected_with(reply, "ACTIVITY_QBI_HISTORY_UNAVAILABLE");
        assert_eq!(again.draft, before);
    }
}

#[test]
fn ordinary_sale_keeps_current_gain_and_original_qualified_prior_loss_distinct() {
    for (year, qualified, lookback, deduction) in
        [(2017, 0, 20_000, 4_000), (2025, 100_000, 20_000, 3_800), (2025, 100_000, 0, 0)]
    {
        let draft = sold_rental_with_property_loss_history(year, qualified, lookback);
        let result = supported(&draft);
        // $20k current gain less$1k released loss: joint gain19k. Original
        // nonqualified loss stays excluded from QBI; qualified history enters
        // separately. Fully capital character excludes both property items.
        assert_eq!(value(&result, "agi"), 79_000);
        assert_eq!(value(&result, "capital_gain"), if lookback == 0 { 19_000 } else { 0 });
        assert_eq!(value(&result, "qbi_deduction"), deduction);
    }
}

#[test]
fn mixed_building_history_stays_with_its_actual_improvement() {
    let mut draft = mixed_rental_with_actual_sale_records();
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.buildings.push(purchased_rental_building(10_000_000, 0, 20210101));
    assert!(crate::asset_sale::rental_sale(data, false).is_err());
    data.buildings[1].sale_history = Some(rental_sale_history(8_000_000, 1_000_000, 800_000));
    // Business basis550k+80k and depreciation100k+10k against469800.
    assert_eq!(crate::asset_sale::rental_sale(data, false).unwrap().unwrap().recognized, -50_200);
    assert_eq!(crate::asset_sale::rental_sale(data, true).unwrap().unwrap().recognized, -72_200);
}

#[test]
fn whole_mixed_building_history_does_not_establish_a_separately_disposed_component() {
    let mut draft = mixed_rental_with_actual_sale_records();
    let FederalIncome::Rental { data } = &mut draft.income[1].income else { unreachable!() };
    data.buildings[0].partial =
        Some(PartialDisposition { date: 20260601, cost_of_part: 1_000_000, proceeds: 0 });
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "RENTAL_MIXED_COMPONENT_HISTORY_UNAVAILABLE")
    );
    assert_eq!(FederalEstimate::from(result).tax, None);
}

#[test]
fn community_odd_cent_property_vintages_conserve_source_rows_and_native_carry() {
    let mut projected = Vec::new();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let draft = community_rental_property_history(owner);
        let entries = crate::community::property_entries(&draft);
        let source = entries
            .iter()
            .find_map(|entry| match &entry.income {
                FederalIncome::Rental { data } => Some(data),
                _ => None,
            })
            .unwrap();
        // Two one-cent vintages must join the single one-cent divided carry,
        // rather than independently rounding to two cents for one spouse.
        assert_eq!(source.at_risk.as_ref().unwrap().prior_sale(false), 1);
        assert_eq!(source.property_qbi_history.iter().map(|row| row.remaining).sum::<i64>(), 1);
        assert!(
            crate::rental_property_history::history_issue(&source.property_qbi_history, 1)
                .is_none()
        );
        projected.push(source.property_qbi_history.clone());
        assert_eq!(value(&supported(&draft), "agi"), 30_000);
    }
    assert_eq!(projected[0][0].year, 2020);
    assert_eq!(projected[0][0].remaining, 1);
    assert_eq!(projected[0][0].remaining_qualified, crate::SourceCents::Reported { amount: 1 });
    assert_eq!(projected[1][0].year, 2021);
    assert_eq!(projected[1][0].remaining, 1);
    assert_eq!(projected[1][0].remaining_qualified, crate::SourceCents::Reported { amount: 0 });
}
