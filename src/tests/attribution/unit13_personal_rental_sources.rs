use super::*;

#[test]
fn unit13_personal_rental_gross_expenses_and_niit_keep_final_native_owners() {
    let mut d = personal_rental_owner_fixture();
    // First:11,000 gross -2,000 paid -1,000 actual five-year depreciation.
    // Second:20,000 gross -4,000 paid -3,000 released regular prior loss.
    // Gross31,000 is13b, actual10,000 expenses are14, net21,000 is NII.
    // Regular NIIT798 is304/494 on8,000/13,000 NII, not gross11/20 shares.
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        assert_eq!(value(&r, "personal_rental_3_asset_0_macrs"), 1000);
        assert_eq!(value(&r, "nonbusiness_personal_rent"), 31_000);
        assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 10_000);
        assert_eq!(value(&r, "agi"), 271_000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 21_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 798);
        let i = usize::from(injured == Owner::Spouse);
        for (line, parts) in [
            ("13b", [[11_000, 20_000], [20_000, 11_000]]),
            ("14", [[3000, 7000], [7000, 3000]]),
            ("18", [[304, 494], [494, 304]]),
        ] {
            assert_eq!([column(&r, line, "b"), column(&r, line, "c")], parts[i], "{line}");
        }
        assert!(!r.review_notes.iter().any(|n| matches!(
            n.display.key.as_str(),
            "WARN_8379_ADJUSTMENT_OWNER_UNAVAILABLE" | "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE"
        )));
        assert_nonallocation_ordinary_unchanged(&d, &r);
    }
    d.income.reverse();
    assert_eq!(value(&supported(&d), "agi"), 271_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unit13_personal_rental_suspended_expense_stays_with_its_actual_source() {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    add(&mut d, income(personal_rent(1000, 21_000)));
    let mut profit = income(personal_rent(10_000, 0));
    profit.owner = crate::DocumentOwner::Spouses;
    add(&mut d, profit);
    let r = supported(&d);
    // Passive profit10,000 admits10,000 of the first20,000 loss;
    // first expenses11,000, not21,000 or an owner ratio of gross receipts.
    assert_eq!(value(&r, "personal_rental_3_suspended_loss"), 10_000);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 11_000);
    assert_eq!(value(&r, "agi"), 250_000);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [1000, 10_000]);
    assert_eq!([column(&r, "14", "b"), column(&r, "14", "c")], [11_000, 0]);
    assert_nonallocation_ordinary_unchanged(&d, &r);
    let mut interest = income(savings_income(5000, 0));
    interest.owner = crate::DocumentOwner::Spouses;
    add(&mut d, interest);
    let signed = supported(&d);
    assert_eq!(value(&signed, "form_8960_net_investment_income"), 5000);
    assert_eq!(value(&signed, "net_investment_income_tax"), 190);
    assert_eq!([column(&signed, "14", "b"), column(&signed, "14", "c")], [11_000, 0]);
    assert!(
        signed
            .review_notes
            .iter()
            .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
    assert!(!signed.lines.iter().any(|l| l.id == "form_8379_injured_refund_estimate"));
    assert_nonallocation_ordinary_unchanged(&d, &signed);
}

#[test]
fn unit13_personal_rental_cents_round_gross_and_allowed_expenses_in_native_categories() {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut e = income(personal_rent(1000, 100));
        e.owner = crate::DocumentOwner::from(owner);
        let FederalIncome::NonbusinessProperty { receipts, expenses, .. } = &mut e.income else {
            panic!()
        };
        *receipts += 49;
        *expenses += 49;
        add(&mut d, e);
    }
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        // Each displayed native net is900, but gross and adjustment totals
        // independently round to2,001 and201. The selected spouse gets
        //1,000/100 and the residual spouse1,001/101, not a rounded net repair.
        assert_eq!(value(&r, "nonbusiness_personal_rent"), 2001);
        assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 201);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 1800);
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [1000, 1001]);
        assert_eq!([column(&r, "14", "b"), column(&r, "14", "c")], [100, 101]);
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [34, 34]);
        assert_nonallocation_ordinary_unchanged(&d, &r);
    }
}

// Fully suspended nonbusiness rental expenses cannot offset nonpassive wages.
#[test]
fn unit13_personal_rental_paired_halfcent_native_rounding_witness() {
    extern crate std;
    let mut observed = vec![];
    for cents in [49, 50, 51, 149, 150] {
        let mut d = complete(FederalStatus::Joint);
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut e = income(personal_rent(0, 0));
            e.owner = crate::DocumentOwner::from(owner);
            let FederalIncome::NonbusinessProperty { expenses, .. } = &mut e.income else {
                panic!()
            };
            *expenses = cents;
            add(&mut d, e);
        }
        let r = supported(&d);
        observed.push((cents, value(&r, "nonbusiness_personal_rent_expenses"), value(&r, "agi")));
    }
    std::println!(
        "Native paired expense cents / allowed whole-dollar expenses / AGI: {observed:?}"
    );
    use crate::tests::cross::interview::{accepted, document, save_topic};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let documents = [Owner::Taxpayer, Owner::Spouse].map(|owner| {
        let mut row = document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::EquipmentRental(EquipmentRent {
                    related: crate::RelatedForms::default(),
                    receipts: 0,
                    expenses: 50,
                    basis: 1_000_000,
                    depreciable_basis: 1_000_000,
                    assets: vec![],
                    history: PersonalRentalHistory::NONE,
                }),
            },
        );
        row.owner = crate::DocumentOwner::from(owner);
        row
    });
    let request = save_topic(&mut session, TaxSection::OtherIncome, documents.to_vec());
    accepted(&send(&mut session, request));
    let saved = supported(&session.draft);
    std::println!(
        "Accepted publicSave 50-cent pair: expense={}, AGI={}",
        value(&saved, "nonbusiness_personal_rent_expenses"),
        value(&saved, "agi")
    );

    assert_eq!(
        observed,
        vec![
            (49, 0, 100_000),
            (50, 0, 100_000),
            (51, 0, 100_000),
            (149, 0, 100_000),
            (150, 0, 100_000)
        ],
        "Fully suspended nonbusiness rental costs cannot create a negative expense or a deduction against wages"
    );
}

#[test]
fn unit13_personal_rental_released_fractional_prior_rounds_combined_allowed_cost() {
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(personal_rent(1000, 0));
        entry.owner = crate::DocumentOwner::from(owner);
        let FederalIncome::NonbusinessProperty { kind, .. } = &mut entry.income else { panic!() };
        let NonbusinessPropertyKind::PersonalRental { history, .. } = kind;
        history.regular = 49;
        history.amt = 149;
        add(&mut d, entry);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 1);
    assert_eq!(value(&r, "agi"), 101_999);
    for alternative in [false, true] {
        let mut output = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 100_000, &mut output, alternative);
        assert!(output.issues.is_empty());
        let allowed = pool.personal_expenses_cents().unwrap();
        assert_eq!(allowed.values().sum::<i64>(), if alternative { 298 } else { 98 });
        assert_eq!(pool.personal_expense_adjustment, if alternative { -3 } else { -1 });
    }
}

#[test]
fn unit13_personal_rental_final_cost_uses_actual_amt_depreciation_and_history() {
    let mut d = complete(FederalStatus::Joint);
    let mut entry = income(personal_rent_with_assets(0, 0, vec![separate_amt_equipment()]));
    let FederalIncome::NonbusinessProperty { expenses, kind, .. } = &mut entry.income else {
        panic!()
    };
    *expenses = 49;
    let NonbusinessPropertyKind::PersonalRental { history, .. } = kind;
    history.regular = 149;
    history.amt = 249;
    add(&mut d, entry);
    let mut profit = income(personal_rent(1000, 0));
    profit.owner = crate::DocumentOwner::Spouses;
    add(&mut d, profit);
    for alternative in [false, true] {
        let mut output = FederalResult::empty(d.revision);
        let mut pool = crate::extended_rules::rentals_for(&d, 100_000, &mut output, alternative);
        assert!(output.issues.is_empty());
        assert_eq!(pool.post_pal.as_ref().unwrap().alternative, alternative);
        let allowed = pool.personal_expenses_cents().unwrap();
        assert_eq!(allowed[&2], if alternative { 87_693 } else { 65_734 });
        assert_eq!(allowed[&3], 0);
        assert_eq!(pool.personal_expense_adjustment, if alternative { -221 } else { -1 });
        assert_eq!(
            value(
                &output,
                &alloc::format!(
                    "{}personal_rental_2_allowed_expenses",
                    if alternative { "amt_" } else { "" }
                )
            ),
            if alternative { 877 } else { 657 }
        );
        pool.post_pal = None;
        assert!(pool.personal_expenses_cents().is_none());
    }
    assert_eq!(value(&supported(&d), "agi"), 100_343);
    d.income.reverse();
    assert_eq!(value(&supported(&d), "agi"), 100_343);
}

#[test]
fn unit13_personal_rental_recharacterized_profit_keeps_own_prior_and_no_pool_offset() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(personal_rent_with_history(1000, 100, 200, 400, true)));
    let mut loss = income(personal_rent(0, 500));
    loss.owner = crate::DocumentOwner::Spouses;
    add(&mut d, loss);
    for alternative in [false, true] {
        let mut output = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 100_000, &mut output, alternative);
        assert!(output.issues.is_empty());
        let allowed = pool.personal_expenses_cents().unwrap();
        assert_eq!(allowed[&2], if alternative { 50_000 } else { 30_000 });
        assert_eq!(allowed[&3], 0);
        let source =
            pool.post_pal.as_ref().unwrap().sources.iter().find(|s| s.income == 2).unwrap();
        assert!(!source.common_pool);
        assert_eq!(source.suspended_cents, 0);
    }
    assert_eq!(value(&supported(&d), "agi"), 100_700);
}

#[test]
fn unit13_personal_rental_consumes_same_mixed_real_estate_pool_and_allowance() {
    for profit in [true, false] {
        let mut d = complete(FederalStatus::Joint);
        add(&mut d, income(personal_rent(0, 100)));
        let mut real_estate =
            income(FederalIncome::Rental { data: rental(if profit { 10_001 } else { 9900 }, 0) });
        real_estate.owner = crate::DocumentOwner::Spouses;
        add(&mut d, real_estate);
        for alternative in [false, true] {
            let mut output = FederalResult::empty(d.revision);
            let pool = crate::extended_rules::rentals_for(&d, 100_000, &mut output, alternative);
            assert!(output.issues.is_empty());
            assert_eq!(pool.income, if profit { 1 } else { -100 });
            assert_eq!(pool.personal_expenses_cents().unwrap()[&2], if profit { 100 } else { 0 });
            let packet = pool.post_pal.as_ref().unwrap();
            assert_eq!(packet.alternative, alternative);
            assert_eq!(packet.special_allowance_cents, if profit { 0 } else { 10_000 });
            assert_eq!(
                packet.sources.iter().find(|v| v.income == 2).unwrap().suspended_cents,
                if profit { 9900 } else { 10_000 }
            );
        }
        assert_eq!(value(&supported(&d), "agi"), if profit { 100_000 } else { 99_900 });
    }
}

#[test]
fn unit13_personal_rental_display_retains_annual_held_residual_and_raw_costs() {
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(personal_rent(0, 0));
        entry.owner = crate::DocumentOwner::from(owner);
        let FederalIncome::NonbusinessProperty { expenses, .. } = &mut entry.income else {
            panic!()
        };
        *expenses = 50;
        add(&mut d, entry);
    }
    for alternative in [false, true] {
        let mut output = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 100_000, &mut output, alternative);
        assert!(output.issues.is_empty());
        assert_eq!(pool.personal_expenses_cents().unwrap().values().sum::<i64>(), 0);
        assert_eq!(
            pool.post_pal.as_ref().unwrap().sources.iter().map(|s| s.suspended_cents).sum::<i64>(),
            100
        );
        let prefix = if alternative { "amt_" } else { "" };
        assert_eq!(
            value(&output, &alloc::format!("{prefix}personal_rental_2_suspended_loss"))
                + value(&output, &alloc::format!("{prefix}personal_rental_3_suspended_loss")),
            1
        );
    }
    assert_eq!(value(&supported(&d), "agi"), 100_000);
}
