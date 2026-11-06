use super::*;

#[test]
fn capital_loss_limit_and_carryforward() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(FederalSale {
                use_kind: SaleUse::Investment { wash_sale: 100_000 },
                ..native_sale(GainTerm::Long, 100_000, Some(900_000))
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 4000);
    answer_separate_pages(&mut d);
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Separate);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    assert_eq!(value(&supported(&d), "capital_gain"), -1500);
}

#[test]
fn private_activity_interest_is_exempt_for_regular_tax_but_enters_amt() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(20_000_000, 20_000_000)),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(r.tax, Some(4_417_400));
    assert_eq!(return_line(&r, "2a"), 20_000_000);
}

#[test]
fn oid_document_flows_to_schedule_b_with_premium_and_market_discount() {
    // One 1099-OID payer: box 1 $1,200, box 2 $80, box 4 $110, box 5 $60,
    // box 6 $150, box 10 $40, box 11 (tax-exempt OID) $500, of which $200 is
    // the private-activity part (from the payer's supplemental statement,
    // not a numbered box). Box 8 (Treasury OID) is $0 here.
    //
    // Taxable OID (box 1 + box 2 + box 8 - box 6 - box 10, floored at zero,
    // per payer): 1200 + 80 + 0 - 150 - 40 = 1090 (positive; no floor
    // applies). Box 5 market discount joins taxable interest the same way
    // 1099-INT box 10 does: 1090 + 60 = 1150 total taxable interest.
    //
    // `complete(FederalStatus::Single)` supplies the only other income, one
    // $60,000 W-2 with no interest/dividends of its own, so:
    //   AGI = 60,000 + 1,150 = 61,150.
    //   Line 2a (tax-exempt OID, box 11) = 500.
    //   Line 2b (taxable interest, includes the OID and market discount) = 1,150.
    //   Line 25b (1099 withholding, box 4) = 110.
    //
    // AMT preference: the $200 private-activity part of the box 11
    // tax-exempt OID enters Form 6251 AMTI dollar for dollar, the same as
    // 1099-INT box 9. In `federal_rules.rs`'s `amt_base` formula, a
    // standard-deduction filer's `+ standard` addback exactly cancels the
    // `- unlimited_deduction` term (both equal the standard deduction), and
    // every other term (NOL, loan gain, capital, refund, QBI, ISO, business,
    // rental, itemized deductions) is zero for this one-wage, one-1099-OID
    // return, so AMTI = AGI + private-activity = 61,150 + 200 = 61,350.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Oid {
            data: oid_source(FederalOid {
                oid: 120_000,
                other_periodic_interest: 8_000,
                withholding: 11_000,
                market_discount: 6_000,
                acquisition_premium: 15_000,
                exempt: 50_000,
                bond_premium: 4_000,
                private_activity: 20_000,
                special: crate::OidSpecial::NoneOfThese,
                ..Default::default()
            }),
        }),
    );
    d.screening.as_mut().unwrap().taxable_premium_election = Some(true);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 61150);
    assert_eq!(return_line(&r, "2a"), 50_000);
    assert_eq!(return_line(&r, "2b"), 115_000);
    assert_eq!(return_line(&r, "25b"), 11_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 61350);
}

#[test]
fn treasury_oid_joins_taxable_oid_and_may_be_negative() {
    // Payer A: box 1 $300, box 8 (Treasury OID) -$120. Box 8 may be
    // negative, such as a TIPS deflation adjustment, and joins taxable OID
    // before the per-payer floor: 300 + (-120) = 180 (positive; no floor
    // applies).
    // Payer B: box 1 $300, box 8 -$500. The negative box 8 exceeds this
    // payer's own box 1 OID, so this payer's own contribution floors at
    // zero rather than contributing -$200, and the shortfall never borrows
    // from payer A's $180: positive(300 + (-500)) = 0.
    // Line 2b (taxable interest, both payers) = 180 + 0 = 180.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Oid {
            data: oid_source(FederalOid {
                oid: 30_000,
                treasury_oid: -12_000,
                ..Default::default()
            }),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Oid {
            data: oid_source(FederalOid {
                oid: 30_000,
                treasury_oid: -50_000,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 18_000);
    assert_eq!(value(&r, "agi"), 60180);
}

#[test]
fn oid_special_treatment_is_left_out_and_negative_net_is_refused() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Oid {
            data: oid_source(FederalOid {
                oid: 100_000,
                special: crate::OidSpecial::Another,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_OID_OTHER_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60000);

    // Box 6 acquisition premium ($200) exceeds box 1 OID ($100): the taxable
    // amount floors at zero instead of contributing -$100 of interest.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Oid {
            data: oid_source(FederalOid {
                oid: 10_000,
                acquisition_premium: 20_000,
                special: crate::OidSpecial::NoneOfThese,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 0);
    assert_eq!(value(&r, "agi"), 60000);
}

#[test]
fn bond_premium_and_accrued_interest_reduce_schedule_b_per_payer() {
    // Payer A: box 1 $900, box 11 bond premium $120, plus $200 of accrued
    // interest paid on purchase. Box 11 and the accrued interest each
    // reduce this payer's own box 1 interest, not below zero:
    //   900 - 120 - 200 = 580.
    // Payer B: box 1 $300, box 12 bond premium on a Treasury obligation
    // $350. The premium exceeds this payer's own interest, so it floors at
    // zero rather than borrowing the $50 shortfall from payer A:
    //   positive(300 - 350) = 0.
    // Line 2b (taxable interest, all payers) = 580 + 0 = 580. Flooring the
    // combined $1,200 box 1 total by the combined $670 of reductions before
    // any per-payer floor would instead give 1,200 - 670 = 530, so this
    // assertion also distinguishes the required per-payer floor from an
    // incorrect combined one.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest {
                taxable: 90_000,
                bond_premium: 12_000,
                accrued_interest: 20_000,
                ..Default::default()
            }),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest {
                taxable: 30_000,
                treasury_bond_premium: 35_000,
                ..Default::default()
            }),
        }),
    );
    d.screening.as_mut().unwrap().taxable_premium_election = Some(true);
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 58_000);
}

#[test]
fn tax_exempt_premium_and_exempt_interest_dividends_adjust_line_2a_and_amt() {
    // One 1099-INT: box 8 tax-exempt interest $1,000, box 13 tax-exempt
    // bond premium $150. Box 13 reduces this payer's own tax-exempt
    // interest, not below zero: 1,000 - 150 = 850.
    //
    // One 1099-DIV: box 12 exempt-interest dividends $400 (not part of box
    // 1a ordinary dividends), box 13 (the private-activity subset of box
    // 12, validated <= box 12) $100. Box 12 joins Form 1040 line 2a
    // directly, the same way 1099-INT box 8 does; box 13 joins the Form
    // 6251 AMT preference, the same way 1099-INT box 9 does.
    //
    // Line 2a = 850 (the 1099-INT, net of its own box 13) + 400 (the
    // 1099-DIV box 12) = 1,250.
    //
    // AMTI: `complete(FederalStatus::Single)` supplies the only other
    // income, a $60,000 W-2; both documents' taxable/ordinary boxes are
    // zero, so AGI is unaffected and stays $60,000.
    // `oid_document_flows_to_schedule_b_with_premium_and_market_discount`
    // (above) establishes that for a standard-deduction single filer with
    // no other adjustment, `amt_base`'s (`federal_rules.rs`) `+ standard`
    // addback exactly cancels its `- unlimited_deduction` term, leaving
    // AMTI = AGI + the private-activity preference = 60,000 + 100 (the
    // 1099-DIV box 13 amount) = 60,100.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest {
                exempt: 100_000,
                tax_exempt_bond_premium: 15_000,
                ..Default::default()
            }),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                exempt_dividends: 40_000,
                private_activity_dividends: 10_000,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(return_line(&r, "2a"), 125_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60100);

    // Box 13 above box 12 is refused.
    let mut over_dividend = d.clone();
    if let FederalIncome::Dividend { data } = &mut over_dividend.income[2].income {
        let mut data = edit_dividend(data);
        data.private_activity_dividends = data.exempt_dividends + 1;
    } else {
        panic!("expected the dividend entry at index 2: {:?}", over_dividend.income);
    }
    assert!(!over_dividend.valid_stored());
}

#[test]
fn capital_carryover_preserves_character_and_unused_below_zero_deduction() {
    use crate::extended_rules::capital_carryover;
    // Schedule D worksheet: use short losses first; gains of the opposite
    // character offset losses before any carryover can remain.
    for (short, long, deduction, taxable, expected) in [
        (-5000, -10000, 3000, 40000, [2000, 10000]),
        (-1000, -10000, 3000, 40000, [0, 8000]),
        (-10000, 4000, 3000, 40000, [3000, 0]),
        (4000, -10000, 3000, 40000, [0, 3000]),
        (-5000, -10000, 3000, -3000, [5000, 10000]),
        (-5000, -10000, 3000, -2000, [4000, 10000]),
        (-1000, -10000, 3000, -1000, [0, 9000]),
        (-1000, -1000, 2000, -5000, [1000, 1000]),
        (-5000, -10000, 1500, 40000, [3500, 10000]),
        (5000, 10000, 0, 40000, [0, 0]),
    ] {
        assert_eq!(capital_carryover(short, long, deduction, taxable), expected);
    }
}

#[test]
fn amt_rental_uses_regular_magi_and_keeps_regular_qbi_deduction() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 0);
    let mut v = rental(10000, 31000); // 31,000 loss after 10,000 depreciation
    v.qbi_business = RentalBusiness::Yes { material_participation: true };
    add(&mut d, income(FederalIncome::Rental { data: v }));
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), -15000);
    assert_eq!(value(&r, "amt_rental_magi"), 120000);
    assert_eq!(value(&r, "amt_rental_income"), -15000);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 16000);
    assert_eq!(value(&r, "amt_rental_adjustment"), 0);
    // Switch to positive QBI: the same regular deduction reduces AMTI.
    let FederalIncome::Rental { data: v } = &mut d.income[1].income else { panic!() };
    v.rent = 7_000_000;
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 5800);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 343200);
}

#[test]
fn signed_fraction_rounding_preserves_losses() {
    use crate::kernel::ratio::fraction;
    assert_eq!(fraction(-3000, 75000, 75000), -3000);
    assert_eq!(fraction(-3, 1, 2), -2);
    assert_eq!(fraction(3, 1, 2), 2);
}
#[test]
fn casual_gambling_uses_2026_loss_limit_and_preserves_gross_income() {
    for (sources, expected) in [
        (vec![(30000, 30000)], 27000),
        (vec![(30000, 40000)], 30000),
        (vec![(30000, 0), (0, 40000)], 30000),
    ] {
        let mut d = complete(FederalStatus::Single);
        for (winnings, losses) in &sources {
            add(&mut d, gambling(*winnings, *losses));
        }
        let r = supported(&d);
        assert_eq!(value(&r, "gambling_loss_deduction"), expected);
        if sources == [(30000, 30000)] {
            assert_eq!(value(&r, "agi"), 90000);
            assert_eq!(value(&r, "taxable_income"), 63000);
            assert_eq!(return_line(&r, "8"), 3000000);
            assert_eq!(value(&r, "self_employment_tax"), 0);
            assert_eq!(restart(d.clone()).draft, d);
        } else if sources.len() == 1 {
            // 2026-11-02: automatic selection keeps the retained eligible itemized claim.
            d.deductions.as_mut().unwrap().itemize_override = None;
            assert_eq!(value(&supported(&d), "gambling_loss_deduction"), 30000);
        }
    }
}

#[test]
fn personal_sale_loss_cannot_offset_another_items_capital_gain() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_sale(1000, 5000, GainTerm::Long)));
    add(&mut d, income(personal_sale(4000, 1000, GainTerm::Long)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 3000);
    assert_eq!(value(&r, "personal_sale_disallowed_loss"), 4000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
    assert_eq!(value(&r, "agi"), 63000);
    assert_eq!(value(&r, "income_tax"), 5023);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Sale { data } = &mut d.income[1].income else { panic!() };
    data.form.use_kind = SaleUse::Investment { wash_sale: 0 };
    assert_eq!(value(&supported(&d), "capital_gain"), -1000);
}

#[test]
fn collectible_sales_net_losses_and_preserve_personal_loss_disallowance() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(collectible_sale(4000, 1000, GainTerm::Long, false)));
    add(&mut d, income(collectible_sale(1000, 5000, GainTerm::Long, true)));
    let r = supported(&d);
    assert_eq!(value(&r, "rate_28_gain"), 3000);
    assert_eq!(value(&r, "income_tax"), 5383);
    assert_eq!(value(&r, "personal_sale_disallowed_loss"), 4000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[2].income = collectible_sale(1000, 1500, GainTerm::Short, false);
    assert_eq!(value(&supported(&d), "rate_28_gain"), 2500);
    d.income[2].income = collectible_sale(1000, 5000, GainTerm::Long, false);
    let r = supported(&d);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!(value(&r, "capital_gain"), -1000);
}

#[test]
fn collectible_mixed_gain_bands_follow_distinct_regular_and_amt_worksheet_bases() {
    use crate::federal_rules::preferential_tax_with_collectibles;
    // Worksheet ordinary base 201,775, capped gain 398,225 at 28%,
    // and 100,000 of other gain at 15% (not 20% despite total income).
    assert_eq!(
        preferential_tax_with_collectibles(700000, 600000, 500000, FederalStatus::Single),
        167527
    );
    assert_eq!(
        crate::extended_rules::amt_tax_with_collectibles(
            700000,
            100000,
            600000,
            FederalStatus::Single,
            0,
            500000
        ),
        178110
    );
}

#[test]
fn dividend_collectible_distribution_is_a_component_not_additional_income() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                capital_distributions: 500000,
                collectible_distributions: 300000,
                withholding: 10000,
                special: crate::DividendSpecial::NoneOfThese,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 65000);
    assert_eq!(value(&r, "capital_gain"), 5000);
    assert_eq!(value(&r, "rate_28_gain"), 3000);
    assert_eq!(value(&r, "income_tax"), 5383);
    assert_eq!(r.payments, Some(660000));
    assert_eq!(restart(d.clone()).draft, d);
    add(&mut d, income(collectible_sale(1000, 2000, GainTerm::Short, false)));
    assert_eq!(value(&supported(&d), "rate_28_gain"), 2000);
    let FederalIncome::Dividend { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_dividend(data);
    data.collectible_distributions = 500001;
    drop(data);
    assert!(!d.valid_stored());
}

#[test]
fn section_1250_and_collectible_distributions_share_loss_netting_in_order() {
    for (loss, term, collectible, expected_28, expected_25) in [
        (1000, GainTerm::Short, false, 2000, 5000),
        (4000, GainTerm::Short, false, 0, 4000),
        (9000, GainTerm::Short, false, 0, 0),
        (4000, GainTerm::Long, false, 1000, 5000),
        (4000, GainTerm::Long, true, 0, 4000),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(mixed_gain_distribution()));
        let mut sale = collectible_sale(0, loss, term, false);
        let FederalIncome::Sale { data } = &mut sale else { panic!() };
        data.form.collectible = collectible;
        add(&mut d, income(sale));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 10000 - loss);
        assert_eq!(value(&r, "rate_28_gain"), expected_28);
        assert_eq!(value(&r, "unrecaptured_section_1250_gain"), expected_25);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn cash_liquidation_recognizes_loss_only_on_final_cancellation() {
    let mut d = complete(FederalStatus::Single);
    let mut distribution = capital_return(2000, 10000, GainTerm::Long);
    let FederalIncome::ReturnOfCapital { kind, .. } = &mut distribution else { panic!() };
    *kind = CapitalDistributionKind::InterimLiquidation;
    add(&mut d, income(distribution));
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_remaining_basis")), 8000);
    let FederalIncome::ReturnOfCapital { kind, .. } = &mut d.income[1].income else { panic!() };
    *kind = CapitalDistributionKind::FinalLiquidation;
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_capital_gain")), -8000);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_remaining_basis")), 0);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 5000);
    assert_eq!(crate::interview::document_amount(&d.income[1].income, false), Some(-800000));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn official_duty_suspends_home_sale_test_window_and_preserves_source_facts() {
    for (status, taxable) in [(FederalStatus::Single, 300000), (FederalStatus::Joint, 50000)] {
        let mut d = complete(status);
        add(
            &mut d,
            income(FederalIncome::HomeSale {
                data: official_duty_sale_source(status == FederalStatus::Joint),
            }),
        );
        assert_eq!(value(&supported(&d), "capital_gain"), taxable);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn official_duty_requires_qualified_orders_property_election_and_each_residence_test() {
    for case in [0, 1, 2, 3, 4, 6] {
        let mut d = complete(FederalStatus::Single);
        let mut home = official_duty_sale_source(false);
        let service = home.facts_mut().service.as_mut().unwrap();
        match case {
            0 => {
                service.election.service = HomeDutyKind::Uniformed {
                    duty: QualifiedHomeDuty { extended_orders: false, remote_or_quarters: true },
                }
            }
            1 => service.election.one_property = false,
            2 => {
                service.election.service = HomeDutyKind::Uniformed {
                    duty: QualifiedHomeDuty { extended_orders: true, remote_or_quarters: false },
                }
            }
            3 => service.election.service = HomeDutyKind::Other,
            4 => service.added_lived = 729,
            6 => service.election.suspended_days = 1,
            _ => unreachable!(),
        }
        add(&mut d, income(FederalIncome::HomeSale { data: home }));
        if case >= 5 {
            assert_eq!(estimate(&d).state, ResultState::Incomplete);
        } else {
            unsupported(&d, "Income:");
        }
    }
}
#[test]
fn purchased_nonsecurity_debt_uses_cost_not_face_for_short_term_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(purchased_bad_debt(10000, 4000, 200)));
    let r = supported(&d);
    assert_eq!(value(&r, "bad_debt_2_short_term_loss"), -4200);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 1200);
    assert_eq!(restart(d.clone()).draft, d);
    add(&mut d, income(collectible_sale(5000, 1000, GainTerm::Short, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -200);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
}
#[test]
fn bad_debt_requires_complete_current_year_worthlessness_and_real_debt() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(bad_debt(10000, 0)));
    let FederalIncome::NonbusinessBadDebt { data: crate::NonbusinessBadDebt { worthless, .. } } =
        &mut d.income[1].income
    else {
        panic!()
    };
    // Not worthless in 2026: no loss this year.
    *worthless = false;
    let r = supported(&d);
    assert!(!r.lines.iter().any(|line| line.id.starts_with("bad_debt_")));
    let FederalIncome::NonbusinessBadDebt {
        data: crate::NonbusinessBadDebt { worthless, bona_fide, .. },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *worthless = true;
    *bona_fide = false;
    let r = supported(&d);
    assert!(!r.lines.iter().any(|line| line.id.starts_with("bad_debt_")));
    let FederalIncome::NonbusinessBadDebt {
        data: crate::NonbusinessBadDebt { source: BadDebtSource::CashLoan { recovered, .. }, .. },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *recovered = 1000001;
    assert!(!d.valid_stored());
}
#[test]
fn section1244_return_limit_preserves_capital_excess_and_does_not_net_gains() {
    for (status, limit, original_wages) in
        [(FederalStatus::Single, 50000, 60000), (FederalStatus::Joint, 100000, 100000)]
    {
        let mut d = complete(status);
        add(&mut d, income(section1244_loss(limit + 10000, 0, 20200101)));
        let r = supported(&d);
        assert_eq!(value(&r, "section_1244_ordinary_loss"), limit);
        assert_eq!(value(&r, "agi"), original_wages - limit - 3000);
        assert_eq!(return_line(&r, "8"), -limit * 100);
        assert_eq!(value(&r, "capital_long_carryforward"), 10000);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(section1244_loss(50000, 0, 20200101)));
    let mut gain = collectible_sale(30000, 10000, GainTerm::Long, false);
    if let FederalIncome::Sale { data } = &mut gain {
        data.form.collectible = false;
    }
    add(&mut d, income(gain));
    let r = supported(&d);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 50000);
    assert_eq!(value(&r, "capital_gain"), 20000);
    assert_eq!(value(&r, "agi"), 30000);
}
#[test]
fn section1244_loss_event_and_source_qualification_are_validated() {
    let mut d = complete(FederalStatus::Single);
    let mut stock = Section1244Stock {
        basis: StockLossBasis::Cash { cost: 6000000, added: 0 },
        acquired: 20251231,
        event: StockLossEvent::Worthless { confirmed: true },

        ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
    };
    assert_eq!(stock.term(), GainTerm::Short);
    stock.acquired = 20251230;
    assert_eq!(stock.term(), GainTerm::Long);
    stock.acquired = 20260101;
    add(&mut d, income(FederalIncome::SmallBusinessStock { data: stock.clone() }));
    assert_eq!(value(&supported(&d), "capital_short_carryforward"), 10000);
    stock.ownership = crate::StockLossOwnership::OriginalIndividual { qualified: false };
    d.income[1].income = FederalIncome::SmallBusinessStock { data: stock.clone() };
    // Unconfirmed qualification: the loss is capital, with a warning.
    let r = supported(&d);

    assert_eq!(value(&r, "section_1244_ordinary_loss"), 0);
    stock.ownership = crate::StockLossOwnership::OriginalIndividual { qualified: true };
    stock.ownership = crate::StockLossOwnership::OtherUnmodeledHistory;
    d.income[1].income = FederalIncome::SmallBusinessStock { data: stock.clone() };
    assert!(noted(&supported(&d), "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT"));
    stock.ownership = crate::StockLossOwnership::OriginalIndividual { qualified: true };
    stock.event = StockLossEvent::Worthless { confirmed: false };
    d.income[1].income = FederalIncome::SmallBusinessStock { data: stock.clone() };
    // Not worthless this year: no loss, and no note for a No.
    let r = supported(&d);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 0);
    assert!(!noted(&r, "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT"));

    stock.event = StockLossEvent::Sale { date: 20250101, proceeds: 0 };
    assert!(!stock.valid());
    stock.event = StockLossEvent::Sale { date: 20261231, proceeds: 6000001 };
    assert!(!stock.valid());
}

#[test]
fn section1244_property_and_later_contributions_match_irs_examples() {
    for (basis, proceeds, ordinary, capital) in [
        (StockLossBasis::Cash { cost: 1000000, added: 200000 }, 900000, 2500, 500),
        (
            StockLossBasis::Property {
                liabilities: 0,
                basis: 100000,
                value: 25000,
                acquired: 20200101,
            },
            20000,
            50,
            750,
        ),
        (
            StockLossBasis::Property {
                liabilities: 0,
                basis: 100000,
                value: 25000,
                acquired: 20200101,
            },
            50000,
            0,
            500,
        ),
        (
            StockLossBasis::Property {
                liabilities: 0,
                basis: 100000,
                value: 150000,
                acquired: 20200101,
            },
            20000,
            800,
            0,
        ),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::SmallBusinessStock {
                data: Section1244Stock {
                    basis,
                    acquired: 20260101,
                    event: StockLossEvent::Sale { date: 20261231, proceeds },

                    ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
                },
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "section_1244_ordinary_loss"), ordinary);
        assert_eq!(value(&r, "capital_gain"), -capital);
        assert_eq!(value(&r, "agi"), 60000 - ordinary - capital);
        assert_eq!(return_line(&r, "8"), -ordinary * 100);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn royalty_special_treatments_remain_explicit_and_invalid_costs_are_rejected() {
    let mut d = complete(FederalStatus::Single);
    let mut value = royalty(10000, 1000);
    if let FederalIncome::Royalty { data } = &mut value {
        data.ordinary = false;
    }
    add(&mut d, income(value));
    assert!(noted(&supported(&d), "WARN_NONBUSINESS_PROPERTY_LEFT_OUT"));
}

#[test]
fn investment_property_taxes_are_uncapped_and_coordinate_niit_and_amt() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    add(&mut d, income(savings_income(100000, 0)));
    itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 5000000;
    elect_itemized(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_property_tax_deduction"), 50000);
    assert_eq!(value(&r, "deduction"), 50000); // Not personal SALT's $40,400 cap.
    assert_eq!(value(&r, "net_investment_income_tax"), 1900);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 400000);
    assert_eq!(restart(d.clone()).draft, d);
    // 2026-11-02: automatic selection keeps the retained eligible itemized claim.
    d.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&d);
    assert_eq!(value(&r, "investment_property_tax_deduction"), 50000);
    assert_eq!(value(&r, "net_investment_income_tax"), 1900);
}

#[test]
fn investment_property_tax_respects_section_68_and_saved_amount_bounds() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 80000000, 0);
    add(&mut d, income(savings_income(100000, 0)));
    itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 10000000;
    let r = supported(&d);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 5405);
    assert_eq!(value(&r, "investment_property_tax_deduction"), 94595);
    assert_eq!(value(&r, "net_investment_income_tax"), 205);
}

#[test]
fn niit_sales_and_personal_property_taxes_are_not_investment_expenses() {
    let mut d = niit_tax_source_return(PersonalTaxElection::Sales { paid: 1000000 });
    itemized(d.deductions.as_mut().unwrap()).personal_property_taxes = 1000000;
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 70000);
    assert_eq!(value(&r, "niit_income_tax_deduction"), 0);
    assert_eq!(value(&r, "net_investment_income_tax"), 1900);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn niit_income_tax_cap_is_shared_with_personal_property_taxes() {
    let mut d = niit_tax_source_return(PersonalTaxElection::Income {
        paid: 4000000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    });
    itemized(d.deductions.as_mut().unwrap()).personal_property_taxes = 4000000;
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 90400);
    // $40,400 personal cap * half income tax * .25 income ratio.
    assert_eq!(value(&r, "niit_income_tax_deduction"), 5050);
    assert_eq!(value(&r, "net_investment_income_tax"), 1708);
}

#[test]
fn investment_interest_expenses_reduce_regular_limit_but_not_amt() {
    for (wages, interest, property_taxes, income_taxes, expected, restart_case) in [
        (
            30000000,
            80000,
            4000000,
            0,
            &[
                ("investment_interest_deduction", 60000),
                ("investment_interest_carryforward", 20000),
                ("amt_investment_interest_deduction", 80000),
                ("deduction", 100000),
                ("alternative_minimum_taxable_income", 320000),
                ("net_investment_income_tax", 0),
            ][..],
            false,
        ),
        (
            30000000,
            70000,
            0,
            1000000,
            &[
                ("investment_interest_deduction", 70000),
                ("investment_interest_carryforward", 0),
                ("niit_income_tax_deduction", 2500),
                ("net_investment_income_tax", 1045),
                ("alternative_minimum_taxable_income", 330000),
            ][..],
            true,
        ),
        (
            80000000,
            100000,
            0,
            0,
            &[
                ("investment_interest_deduction", 100000),
                ("investment_interest_carryforward", 0),
                ("itemized_deduction_reduction", 5405),
                ("deduction", 94595),
                ("niit_itemized_deductions", 94595),
                ("net_investment_income_tax", 205),
                ("alternative_minimum_taxable_income", 800000),
            ][..],
            true,
        ),
        // The entire allowed $40,400 SALT deduction still leaves $59,600.
        (
            30000000,
            59600,
            0,
            10000000,
            &[
                ("investment_interest_deduction", 59600),
                ("investment_interest_carryforward", 0),
                ("niit_income_tax_deduction", 10100),
            ][..],
            false,
        ),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        add(&mut d, income(savings_income(100000, 0)));
        investment_borrowing(&mut d, interest, 0, 0);
        let deductions = itemized(d.deductions.as_mut().unwrap());
        deductions.investment_property_taxes = property_taxes;
        if income_taxes != 0 {
            deductions.personal_tax = PersonalTaxElection::Income {
                paid: income_taxes,
                allocation: TaxIncomeAllocation::CurrentReturn,
            };
        }
        let r = supported(&d);
        for &(key, amount) in expected {
            assert_eq!(value(&r, key), amount);
        }
        if restart_case {
            assert_eq!(restart(d.clone()).draft, d);
        }
        if income_taxes == 10000000 {
            d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans[0].paid +=
                100;
            let r = supported(&d);
            assert_eq!(value(&r, "investment_interest_deduction"), 59600);
            assert_eq!(value(&r, "investment_interest_carryforward"), 1);
            assert!(noted(&r, "NOTE_INVESTMENT_TAX_ALLOCATION"));
            assert_eq!(value(&r, "agi"), 400000);
        }
    }
}

#[test]
fn investment_interest_elections_and_negative_stored_amounts_are_not_silent() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(10000, 0)));
    investment_borrowing(&mut d, 5000, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 100 };
    unsupported(&d, "Investment interest");
}

#[test]
fn investment_interest_excludes_qualified_and_long_term_but_includes_net_short_gain() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(native_dividend(1000000, 800000, 1000000)),
        }),
    );
    add(&mut d, income(collectible_sale(5000, 1000, GainTerm::Short, false)));
    investment_borrowing(&mut d, 20000, 0, 0);
    let r = supported(&d);
    // 2,000 nonqualified dividends plus 4,000 net short-term gain.
    assert_eq!(value(&r, "investment_interest_deduction"), 6000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 14000);
    add(&mut d, income(collectible_sale(0, 12000, GainTerm::Long, false)));
    // Long-term loss offsets the short-term gain: 4,000 - 2,000.
    assert_eq!(value(&supported(&d), "investment_interest_deduction"), 4000);
}

#[test]
fn standard_route_does_not_carry_allowed_interest_or_deduct_amt_interest() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(10000, 0)));
    investment_borrowing(&mut d, 12000, 0, 0);
    let deductions = d.deductions.as_mut().unwrap();
    let mut facts = itemized(deductions).clone();
    facts.investment_property_taxes = 400000;
    {
        deductions.itemized = facts;
        deductions.itemize_override = None;
    };
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 16100);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 0);
    // No itemized property expense reduces the standard route's income limit.
    assert_eq!(value(&r, "investment_interest_carryforward"), 2000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 70000);
}

#[test]
fn investment_interest_keeps_personal_and_home_gains_out_of_portfolio_limit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(10000, 0)));
    add(&mut d, income(collectible_sale(5000, 1000, GainTerm::Short, false)));
    add(&mut d, income(collectible_sale(0, 6000, GainTerm::Long, false)));
    add(&mut d, income(personal_sale(10000, 0, GainTerm::Short)));
    add(&mut d, income(FederalIncome::HomeSale { data: home_sale_source(false) }));
    investment_borrowing(&mut d, 15000, 0, 0);
    let r = supported(&d);
    // Portfolio net loss contributes no income; taxable personal/home gains
    // cannot absorb that loss for purposes of increasing Form 4952's limit.
    assert_eq!(value(&r, "investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 5000);
    assert_eq!(value(&r, "capital_gain"), 308000);
    assert_eq!(value(&r, "investment_interest_net_gain"), 0);
    assert_eq!(value(&r, "investment_interest_ordinary_income"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn investment_interest_nets_worthless_investment_securities_by_holding_period() {
    for acquisition in
        [SecurityAcquisition::Inherited, SecurityAcquisition::Purchased { date: 20260101 }]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(collectible_sale(5000, 0, GainTerm::Short, false)));
        add(
            &mut d,
            income(FederalIncome::WorthlessSecurity {
                data: crate::WorthlessSecurity {
                    basis: 300000,
                    acquisition,
                    worthless: true,
                    ordinary: true,
                },
            }),
        );
        investment_borrowing(&mut d, 5000, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), 2000);
        assert_eq!(value(&r, "investment_interest_carryforward"), 3000);
    }
}

#[test]
fn investment_interest_uses_confirmed_investment_capital_carryovers() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(collectible_sale(20000, 0, GainTerm::Short, false)));
    d.screening.as_mut().unwrap().capital_history = vec![capital_history()];
    investment_borrowing(&mut d, 10000, 0, 0);
    let r = supported(&d);
    // Calculated carry-in: 4,000 short and 10,000 long, not entered totals.
    assert_eq!(value(&r, "investment_interest_deduction"), 6000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 4000);
    assert_eq!(restart(d.clone()).draft, d);
    d.screening.as_mut().unwrap().capital_history[0].investment_parts = None;
    // This case leaves Form 4952 attribution unresolved but supplies its
    // separate retained Form 8960 worksheet fact.
    d.screening.as_mut().unwrap().capital_history[0].niit_difference = Some(0);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 6000);
    assert_eq!(value(&r, "investment_interest_deduction"), 6000);
    assert!(noted(&r, "NOTE_INVESTMENT_CAPITAL_PARTS"));
    d.deductions.as_mut().unwrap().itemize_override = None;
    // Unresolved source portions preserve the ordinary Schedule D result.
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 6000);
}

#[test]
fn investment_interest_liquidation_loss_requires_a_final_distribution() {
    for (kind, allowed) in [
        (CapitalDistributionKind::InterimLiquidation, 5000),
        (CapitalDistributionKind::FinalLiquidation, 2000),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(collectible_sale(5000, 0, GainTerm::Short, false)));
        add(
            &mut d,
            income(FederalIncome::ReturnOfCapital {
                kind,
                amount: 200000,
                property: vec![],
                basis: 500000,
                term: GainTerm::Long,
                special: false,
            }),
        );
        investment_borrowing(&mut d, 6000, 0, 0);
        assert_eq!(value(&supported(&d), "investment_interest_deduction"), allowed);
    }
}

#[test]
fn investment_interest_empty_income_pool_does_not_need_tax_attribution() {
    let mut d = complete(FederalStatus::Single);
    investment_borrowing(&mut d, 10000, 0, 0);
    itemized(d.deductions.as_mut().unwrap()).personal_tax =
        PersonalTaxElection::Income { paid: 1000000, allocation: TaxIncomeAllocation::Unresolved };
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "investment_interest_carryforward"), 10000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
}

#[test]
fn investment_election_uses_capital_first_and_preserves_reported_dividends() {
    let d = elected_portfolio();
    let r = supported(&d);
    assert_eq!(value(&r, "investment_elected_capital_gain"), 10000);
    assert_eq!(value(&r, "investment_elected_qualified_dividends"), 5000);
    assert_eq!(value(&r, "investment_interest_deduction"), 15000);
    assert_eq!(return_line(&r, "3a"), 2000000);
    // $60,000 ordinary income uses the $60,000–60,050 Tax Table row;
    // the remaining $15,000 is taxed at 15%.
    assert_eq!(value(&r, "regular_income_tax"), 10168);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn investment_election_coordinates_positive_amt_without_electing_exempt_income() {
    let mut d = elected_portfolio();
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(20000000, 20000000)),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 275000);
    // AMT taxable $184,900: $169,900 at 26%, $15,000 at 15%; subtract
    // $10,168 regular tax after the Tax Table lookup.
    assert_eq!(value(&r, "alternative_minimum_tax"), 36256);
}

#[test]
fn investment_election_does_not_shrink_qbi_net_capital_gain_input() {
    let mut d = elected_portfolio();
    d.income.remove(0);
    add(&mut d, income(cash_business(40000)));
    let r = supported(&d);
    // AGI 67,174, deduction 15,000; QBI limit still subtracts all 30,000
    // original qualified dividends/net capital gain: 20% of 22,174.
    assert_eq!(value(&r, "qbi_deduction"), 4435);
    assert_eq!(value(&r, "regular_income_tax"), 3679);
}

#[test]
fn investment_election_allocation_changes_special_rate_gains() {
    let mut d = elected_portfolio();
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    let FederalIncome::Dividend { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_dividend(data);
    data.ordinary = 1000000;
    data.qualified = 1000000;
    data.capital_distributions = 3000000;
    data.collectible_distributions = 1000000;
    data.unrecaptured_distributions = 2000000;
    drop(data);
    let r = supported(&d);
    // Ordinary tax on 300,000 is 73,769.25, plus 15,000 at 25% and 10,000 at 15%.
    assert_eq!(value(&r, "regular_income_tax"), 79019);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: 500000, dividends: 1000000 };
    let r = supported(&d);
    // Same total election leaves 20,000 at 25% and 5,000 at 28% instead.
    assert_eq!(value(&r, "regular_income_tax"), 80169);
    assert_eq!(value(&r, "investment_elected_capital_gain"), 5000);
    assert_eq!(value(&r, "investment_elected_qualified_dividends"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn investment_election_remains_effective_when_standard_deduction_wins() {
    let mut d = elected_portfolio();
    let deductions = d.deductions.as_mut().unwrap();
    let facts = itemized(deductions).clone();
    deductions.investment_interest.as_mut().unwrap().loans[0].paid = 500000;
    {
        deductions.itemized = facts;
        deductions.itemize_override = None;
    };
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 16100);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "investment_interest_carryforward"), 0);
    assert_eq!(value(&r, "investment_elected_capital_gain"), 10000);
    assert_eq!(value(&r, "investment_elected_qualified_dividends"), 5000);
    // The taxpayer's election is not silently canceled by deduction selection.
    assert_eq!(value(&r, "regular_income_tax"), 9926);
}

#[test]
fn elected_investment_interest_is_subject_to_2026_itemized_limit() {
    let mut d = elected_portfolio();
    d.income[0] = wage(1, Owner::Taxpayer, 80000000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 15000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 0);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 811);
    assert_eq!(value(&r, "niit_itemized_deductions"), 14189);
    assert_eq!(value(&r, "net_investment_income_tax"), 601);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 815000);
}

#[test]
fn surrender_loss_does_not_reduce_ordinary_income_or_create_capital_loss() {
    let loss = contract_discharge(RetirementKind::Nonqualified, 8000, 10000);
    let mut comparison = loss.clone();
    let FederalIncome::Retirement { data } = &mut comparison.income[1].income else { panic!() };
    let mut data = edit_retirement(data);
    data.box_2a = 0;
    data.taxable = Some(RetirementTaxable::Payer);
    drop(data);
    let r = supported(&loss);
    let expected = supported(&comparison);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(value(&r, "agi"), value(&expected, "agi"));
    assert_eq!(return_line(&r, "7a"), return_line(&expected, "7a"));
}

#[test]
fn full_contract_discharge_taxes_unconfirmed_and_rejects_wrong_account_facts() {
    unsupported(&contract_discharge(RetirementKind::Ira, 15000, 10000), "Income:");
    // No: no investment is recovered; the whole payment is taxed, noted.
    let mut d = contract_discharge(RetirementKind::Nonqualified, 15000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_retirement(data);
    let Some(RetirementTaxable::FullDischarge { data: crate::ContractDischarge { eligible, .. } }) =
        &mut data.taxable
    else {
        panic!()
    };
    *eligible = false;
    drop(data);
    let r = supported(&d);
    assert!(noted(&r, "WARN_RETIREMENT_BASIS_LEFT_OUT"), "{r:?}");
    assert!(r.tax > supported(&contract_discharge(RetirementKind::Nonqualified, 15000, 10000)).tax);
}

#[test]
fn bonus_equipment_requires_eligible_dates_property_and_source_bounds() {
    for (acquired, placed) in [(20250120, 20251231), (20260602, 20260601)] {
        let mut asset = bonus_equipment(10000, 100);
        asset.acquired = acquired;
        asset.placed = placed;
        unsupported(&business_with_bonus(100000, asset), "Income:");
    }
}
#[test]
fn personal_canceled_debt_enters_schedule_one_and_agi_without_employment_tax() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, canceled_personal_debt(5000, PersonalDebtTreatment::Taxable));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_debt_cancellation"), 5000);
    assert_eq!(value(&r, "agi"), value(&before, "agi") + 5000);
    assert_eq!(return_line(&r, "8"), return_line(&before, "8") + 500000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn personal_debt_gift_exception_has_no_income_or_attribute_reduction() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, canceled_personal_debt(5000, PersonalDebtTreatment::GiftOrBequest));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_debt_cancellation"), 0);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(r.tax, before.tax);
}
#[test]
fn personal_debt_is_not_net_investment_income_above_niit_threshold() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 25000000, 0);
    add(&mut d, canceled_personal_debt(50000, PersonalDebtTreatment::Taxable));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 300000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
}
#[test]
fn bankruptcy_unconfirmed_court_discharge_or_other_event_is_not_assumed_supported() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, bankruptcy_debt(1500, 6100, 6000));
    add(&mut d, insolvency_debt(1000, 3000, 1000, 4000));
    unsupported(&d, "Bankruptcy tax attributes");
}
#[test]
fn bankruptcy_reduces_the_calculated_capital_loss_carryover() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, bankruptcy_debt(1500, 6100, 6000));
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Long, 0, Some(1000000))),
        }),
    );
    // $10,000 long-term loss: $3,000 deducted, $7,000 carried. The whole
    // $1,500 chapter 13 discharge is excluded (line 2) and, with no NOL or
    // credits, reduces the long-term carryover to $5,500 in both tax systems.
    let r = supported(&d);
    assert_eq!(value(&r, "debt_2_form982_line1a"), 1500);
    assert_eq!(value(&r, "debt_2_form982_line9"), 1500);
    assert_eq!(value(&r, "debt_2_capital_after"), 5500);
    assert_eq!(value(&r, "debt_2_amt_capital_after"), 5500);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
}
#[test]
fn service_discharge_requires_eligible_school_broad_service_terms_and_no_prohibited_work() {
    // Any failed condition leaves the whole cancellation taxable.
    for (eligible_school, service_terms, prohibited_services) in
        [(false, true, false), (true, false, false), (true, true, true)]
    {
        let mut d = complete(FederalStatus::Single);
        let before = supported(&d);
        add(
            &mut d,
            canceled_personal_debt(
                12000,
                PersonalDebtTreatment::StudentDischarge {
                    reason: StudentDischargeReason::Service {
                        lender: StudentServiceLender::SchoolProgram,
                        eligible_school,
                        service_terms,
                        prohibited_services,
                    },
                },
            ),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), value(&before, "agi") + 12000);
        assert_eq!(value(&r, "personal_debt_cancellation"), 12000);
    }
}
#[test]
fn royalty_expenses_reduce_other_portfolio_income_without_double_deduction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(royalty(1000, 4000)));
    add(&mut d, income(savings_income(5000, 0)));
    investment_borrowing(&mut d, 6000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 62000);
    assert_eq!(value(&r, "investment_interest_deduction"), 2000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 4000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 2000);
}

#[test]
fn section1244_ordinary_loss_nets_dispositions_without_erasing_interest_income() {
    for (gain, expected) in [(0, 2000), (4000, 2000), (15000, 7000)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(section1244_loss(10000, 0, 20200101)));
        add(&mut d, income(collectible_sale(gain, 0, GainTerm::Short, false)));
        add(&mut d, income(savings_income(2000, 0)));
        investment_borrowing(&mut d, 20000, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), expected);
        assert_eq!(value(&r, "investment_interest_carryforward"), 20000 - expected);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), expected);
        assert_eq!(value(&r, "section_1244_ordinary_loss"), 10000);
        assert_eq!(value(&r, "capital_gain"), gain);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn section1244_ordinary_loss_caps_investment_capital_gain_election() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(section1244_loss(10000, 0, 20200101)));
    add(&mut d, income(collectible_sale(15000, 0, GainTerm::Long, false)));
    investment_borrowing(&mut d, 20000, 0, 0);
    assert_eq!(value(&supported(&d), "investment_interest_deduction"), 0);
    for (amount, valid) in [(5000, true), (5001, false)] {
        d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
            InvestmentElection::CapitalFirst { amount: amount * 100 };
        if valid {
            let r = supported(&d);
            assert_eq!(value(&r, "investment_interest_deduction"), 5000);
            assert_eq!(value(&r, "investment_elected_capital_gain"), 5000);
            assert_eq!(value(&r, "capital_gain"), 15000);
        } else {
            unsupported(&d, "Investment interest");
        }
    }
}

#[test]
fn bad_debt_investment_purpose_controls_form4952_not_schedule_d_loss() {
    for (purpose, expected) in [(true, 4000), (false, 12000)] {
        let mut d = complete(FederalStatus::Single);
        let mut debt = bad_debt(10000, 2000);
        let FederalIncome::NonbusinessBadDebt {
            data: crate::NonbusinessBadDebt { investment, .. },
        } = &mut debt
        else {
            panic!()
        };
        *investment = purpose;
        add(&mut d, income(debt));
        add(&mut d, income(collectible_sale(10000, 0, GainTerm::Short, false)));
        add(&mut d, income(savings_income(2000, 0)));
        investment_borrowing(&mut d, 15000, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 2000);
        assert_eq!(value(&r, "investment_interest_deduction"), expected);
        assert_eq!(value(&r, "investment_interest_carryforward"), 15000 - expected);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), expected);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn investment_bad_debt_loss_does_not_reduce_unrelated_interest_below_disposition_floor() {
    let mut d = complete(FederalStatus::Single);
    let mut debt = bad_debt(10000, 2000);
    let FederalIncome::NonbusinessBadDebt { data: crate::NonbusinessBadDebt { investment, .. } } =
        &mut debt
    else {
        panic!()
    };
    *investment = true;
    add(&mut d, income(debt));
    add(&mut d, income(savings_income(2000, 0)));
    investment_borrowing(&mut d, 10000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 2000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 8000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 5000);
}

#[test]
fn active_group_profits_cap_special_allowance_before_other_loss_allocation() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 10000) })); // -20,000
    add(&mut d, income(FederalIncome::Rental { data: rental(20000, 0) })); // +10,000
    let mut other = rental(0, 10000);
    other.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: other }));
    let r = supported(&d);
    // Active group's 10,000 net loss limits the special allowance. After
    // its allocation, remaining weights are 10,000 active and 20,000 other.
    assert_eq!(value(&r, "rental_income"), -10000);
    assert_eq!(value(&r, "rental_suspended_loss"), 20000);
    assert_eq!(value(&r, "rental_2_suspended_loss") + value(&r, "rental_4_suspended_loss"), 20000);
    assert!((6666..=6667).contains(&value(&r, "rental_2_suspended_loss")));
    assert_eq!(value(&r, "agi"), 50000);
}

#[test]
fn alternative_nol_can_arise_without_a_regular_nol_and_is_visible() {
    let d = large_separate_amt_business(10000, 100);
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 17395);
    assert_eq!(value(&r, "amt_nol_carryforward"), 17395);
    assert!(breakdown(&r).iter().any(|v| v.label == "AMT NOL carryforward" && v.amount == 1739500));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn alternative_nol_uses_amt_investment_interest_and_not_the_standard_deduction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-80000)));
    add(&mut d, income(savings_income(10000, 0)));
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(1000000, 1000000)),
        }),
    );
    investment_borrowing(&mut d, 20000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 20000);
    assert_eq!(value(&r, "current_net_operating_loss"), 20000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 20000);
    d.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&d);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn alternative_only_nol_reaches_debt_attribute_and_final_return_checks() {
    let mut d = large_separate_amt_business(655360, 1000);
    // Regular business income is exactly zero, so no regular NOL or QBI
    // loss can accidentally make either guard pass this regression.
    let r = supported(&d);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
    assert_eq!(value(&r, "amt_nol_carryforward"), 158587);
    add(&mut d, bankruptcy_debt(1000, 10000, 0));
    unsupported(&d, "Bankruptcy tax attributes");
    d.income.pop();
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20261231, ordinary: true, estate_medical: 0 });
    // Pub. 559: the decedent's unused carryovers end with the final return;
    // the return is computed, with the note.
    assert!(noted(&supported(&d), "NOTE_FINAL_RETURN_CARRYOVERS_END"));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn direct_and_carried_cash_exhaust_stock_room_without_reaging_stock_excess() {
    // $35,000 current + $20,000 old cash exceed the $50,000 A ceiling;
    // neither dollar of the $10,000 stock gift is currently allowable.
    let mut draft =
        current_and_prior_cash_with_appreciated_stock(10_000_000, 3_500_000, 20_000, 1_000_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 35_000);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 19_500);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 500);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 0);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 10_000);
    assert_eq!(value(&result, "charity_cash_sixty_return_year_carryforward"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 10_000);
    assert_eq!(return_line(&result, "12e"), 5_450_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_capital_gain_thirty_return_year_carryforward"), 10_000);
    assert_eq!(value(&automatic, "charity_return_year_carryforward"), 10_000);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn current_cash_takes_sixty_percent_before_old_cash_and_stock_carry() {
    // $50,000 current cash leaves $10,000 G room for a $20,000 old balance.
    // All $40,000 stock is a distinct 2026 C carry. The unused old $10,000
    // remains a 2025 G balance, and the $500 floor is absorbed by old cash.
    let draft =
        current_and_prior_cash_with_appreciated_stock(10_000_000, 5_000_000, 20_000, 4_000_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 50_000);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 9_500);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 500);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 40_000);
    assert_eq!(value(&result, "charity_cash_sixty_return_year_carryforward"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 40_000);
    assert_eq!(return_line(&result, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn small_appreciated_stock_charges_floor_before_older_cash() {
    let draft = cash_carry_with_appreciated_stock(10_000_000, 10_000, 10_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 0);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 9_600);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 400);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 960_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);
}

#[test]
fn atnol_absorption_adds_back_capital_loss_without_increasing_deduction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 100000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 57000);
    assert_eq!(value(&r, "prior_atnol_deduction"), 51300);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 5700);
    assert_eq!(value(&r, "atnol_2025_used"), 54000);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 46000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn atnol_absorption_excludes_qbi_but_deduction_keeps_regular_qbi() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(reit_income(10000, 10000, true)));
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 100000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 2000);
    assert_eq!(value(&r, "prior_atnol_deduction"), 61200);
    assert_eq!(value(&r, "atnol_2025_used"), 63000);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 37000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 6800);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn atnol_can_be_consumed_when_capital_loss_eliminates_actual_amti() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100000, 0);
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 10000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_atnol_deduction"), 0);
    assert_eq!(value(&r, "atnol_2025_used"), 900);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 9100);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 0);
}

#[test]
fn capital_loss_origin_records_reconcile_without_changing_current_tax() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![capital_history()];
    let before = supported(&d);
    let history = &mut d.screening.as_mut().unwrap().capital_history[0];
    history.origins = vec![
        CapitalLossOrigin { year: 2022, short: 200000, long: 0 },
        CapitalLossOrigin { year: 2025, short: 200000, long: 1000000 },
    ];
    // D-2026-10-15-13: one distinct year per source, without a record cap.
    // Reassign 99 cents of the same $4,000 short carry to 99 earlier years.
    history.origins[0].short -= 99;
    history.origins.extend((1900..1999).map(|year| CapitalLossOrigin { year, short: 1, long: 0 }));
    let entered = history.clone();
    let mut session = Interview::new(TEST_USER, d.clone());
    let request = crate::tests::cross::interview::save_topic(
        &mut session,
        TaxSection::InvestmentSales,
        vec![crate::tests::cross::interview::document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::CapitalLossCarryover(entered.into()),
            },
        )],
    );
    let before_estimate = session.draft.clone();
    let dirty = session.dirty;
    let mut invalid = request.clone();
    let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
    let crate::IncomeRecord::CapitalLossCarryover(history) = &mut documents.documents[0].document
    else {
        unreachable!()
    };
    history.origins[100].year = history.origins[99].year;
    // The calculation returns the native invalid-data estimate; only Save
    // performs stored-record admission and returns the page Problem.
    let Reply::Response(P::IncomeRunningEstimate { estimate: Some(estimate), .. }) =
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            super::support::client::estimate_documents_from_save(&invalid),
        )
    else {
        panic!("expected the capital-source running estimate")
    };
    // The refusal is a row of the list, never repeated in the line.
    assert!(crate::tests::cross::interview::shown(&estimate.line).contains("not available yet"));
    // 2026-10-29: a page lists only its own fixes and counts other pages.
    // The invalid accepted-data refusal must still appear once on Review.
    let mut invalid_review = before_estimate.clone();
    let origins = &mut invalid_review.screening.as_mut().unwrap().capital_history[0].origins;
    origins[100].year = origins[99].year;
    // Corrupt accepted data is blocked at Interview admission, before Review.
    // Exercise the same Review-list producer for this invalid preview candidate.
    let invalid_result = crate::federal_rules::estimate(&invalid_review);
    let review =
        crate::interview::remaining_items(&invalid_review, &invalid_result, TaxSection::Review);
    assert_eq!(
        review
            .iter()
            .filter(|row| crate::tests::cross::interview::shown(&row.status)
                .contains("Accepted data is invalid; no calculation was performed."))
            .count(),
        1,
        "{review:?}"
    );
    let [later] = estimate.remaining_items.as_slice() else {
        panic!("expected one later-pages row: {:?}", estimate.remaining_items)
    };
    assert_eq!(later.section, TaxSection::Review);
    assert_eq!(
        crate::tests::support::client::status_message(&later.status),
        crate::Message::new("ESTIMATE_LATER_ITEMS", vec![crate::MessageArg::Count(1)])
    );
    assert_eq!(session.draft, before_estimate);
    assert_eq!(session.dirty, dirty);
    let Reply::Response(P::Problem { message, field }) = send(&mut session, invalid) else {
        panic!("duplicate capital origin year")
    };
    assert_eq!(
        message,
        libertas::libertas_formatted_text(
            "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
            &[]
        )
    );
    assert_eq!(field, None);
    assert_eq!(session.draft, before_estimate);
    assert_eq!(session.dirty, dirty);
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            super::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, before_estimate);
    assert_eq!(session.dirty, dirty);
    crate::tests::cross::interview::accepted(&send(&mut session, request));
    let r = supported(&session.draft);
    assert_eq!(r.tax, before.tax);
    assert_eq!(value(&r, "capital_short_carryforward"), 1000);
    assert_eq!(value(&r, "capital_long_carryforward"), 10000);
    d = super::support::client::save_reopen_native_sources(
        session.draft,
        &[TaxSection::InvestmentSales],
        &[TaxSection::InvestmentSales],
    );
    // The explicit native checklist save advances the return revision; every
    // computed field must still match the pre-restart result exactly.
    let mut expected = r;
    expected.revision = d.revision;
    assert_eq!(supported(&d), expected);
    d.screening.as_mut().unwrap().capital_history[0].origins[0].short -= 100;
    unsupported(&d, "Capital loss history");
}

#[test]
fn capital_loss_origins_reject_duplicate_years_and_invalid_amounts() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![capital_history()];
    d.screening.as_mut().unwrap().capital_history[0].origins = vec![
        CapitalLossOrigin { year: 2025, short: 400000, long: 0 },
        CapitalLossOrigin { year: 2025, short: 0, long: 1000000 },
    ];
    assert!(!d.valid_stored());
}

#[test]
fn debt_nol_reduction_uses_oldest_remaining_year_and_preserves_current_tax() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 10000), (2017, 45000), (2020, 2000)]);
    let before = supported(&d);
    add(&mut d, insolvency_debt(1500, 7500, 4600, 10000));
    let r = supported(&d);
    assert_eq!(value(&r, "nol_2017_used"), 43900);
    assert_eq!(value(&r, "nol_2017_debt_reduction"), 1100);
    assert_eq!(value(&r, "nol_2017_carryforward"), 0);
    assert_eq!(value(&r, "nol_2020_debt_reduction"), 400);
    assert_eq!(value(&r, "nol_2020_carryforward"), 1600);
    assert_eq!(value(&r, "nol_2025_carryforward"), 10000);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 11600);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(r.tax, before.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn debt_nol_reduction_preserves_cents_without_changing_tax_rounding() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let mut entry = bankruptcy_debt(1500, 10000, 6000);
    let FederalIncome::PersonalDebt { data } = &mut entry.income else { panic!() };
    data.form.amount += 25;
    add(&mut d, entry);
    let r = supported(&d);
    let amount = |id: &str| line_amount(&r, id);
    assert_eq!(amount("nol_2025_debt_reduction"), 150025);
    assert_eq!(amount("nol_2025_carryforward"), 6337975);
    assert_eq!(amount("regular_total_nol_carryforward"), 6337975);
    assert_eq!(value(&r, "prior_nol_deduction"), 35120);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
}

#[test]
fn capital_carryover_adds_back_enhanced_senior_but_not_age_standard_deduction() {
    for (wages, remaining) in [(25000, 7000), (20000, 8150), (17000, 10000)] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19560615;
        d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
        add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
        let r = supported(&d);
        assert_eq!(value(&r, "senior_deduction"), 6000);
        assert_eq!(value(&r, "deduction"), 18150);
        assert_eq!(value(&r, "taxable_income"), 0);
        assert_eq!(value(&r, "capital_short_carryforward"), remaining);
        assert_eq!(value(&r, "amt_capital_short_carryforward"), 7000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn senior_capital_carryover_preserves_short_first_character() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19560615;
    d.income = vec![wage(1, Owner::Taxpayer, 2500000, 0)];
    add(&mut d, income(collectible_sale(0, 1000, GainTerm::Short, false)));
    add(&mut d, income(collectible_sale(0, 9000, GainTerm::Long, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    assert_eq!(value(&r, "capital_long_carryforward"), 7000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 0);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 7000);
}

#[test]
fn amt_only_current_loss_preserves_capital_loss_not_used_by_amt_income() {
    let mut d = large_separate_amt_business(655360, 1000);
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Long, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_long_carryforward"), 7000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 10000);
    assert_eq!(value(&r, "amt_capital_loss_carryforward"), 10000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 158587);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn amt_only_capital_loss_coordinates_atnol_deduction_and_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![amt_only_capital_history()];
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 100000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "prior_atnol_deduction"), 51300);
    assert_eq!(value(&r, "atnol_2025_used"), 54000);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 46000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 5700);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 7000);
}

#[test]
fn amt_capital_history_recomputes_collectible_and_unrecaptured_pools() {
    let mut d = complete(FederalStatus::Single);
    let mut history = amt_only_capital_history();
    history.short = -1300000;
    history.deducted_loss = 300000;
    history.amt = Some(PriorAmtCapitalReturn {
        taxable_income: 10000000,
        short: 0,
        long: 0,
        deducted_loss: 0,
    });
    history.investment_parts = Some(InvestmentCapitalParts {
        short: 1_000_000,
        long: 0,
        amt: Some(InvestmentAmtCapitalParts { short: 0, long: 0 }),
    });
    d.screening.as_mut().unwrap().capital_history = vec![history];
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    add(&mut d, income(collectible_sale(8000, 0, GainTerm::Long, false)));
    let mut fund = undistributed_gain();
    let FederalIncome::UndistributedGain { data } = &mut fund else { panic!() };
    data.form.unrecaptured_section_1250_gain = 200000;
    add(&mut d, income(fund));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 3000);
    assert_eq!(value(&r, "rate_28_gain"), 1000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 2000);
    assert_eq!(value(&r, "amt_capital_gain"), 13000);
    assert_eq!(value(&r, "amt_rate_28_gain"), 11000);
    assert_eq!(value(&r, "amt_unrecaptured_section_1250_gain"), 2000);
    assert_eq!(r.tax, Some(4753400));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn differing_amt_pools_do_not_silently_reallocate_an_explicit_investment_election() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![amt_only_capital_history()];
    add(&mut d, income(ordinary_investment_sale(12000, 0, GainTerm::Long)));
    investment_borrowing(&mut d, 5000, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: 1000000, dividends: 0 };
    unsupported(&d, "AMT investment election");
}

#[test]
fn allocated_investment_election_survives_different_sufficient_amt_capital_pools() {
    // The AMT pool may be smaller, exactly the elected capital, or larger
    // than the regular pool. Neither case changes this valid allocation.
    for (regular_short, amt_short, regular_gain, amt_gain) in [
        (0, -800000, 10000, 5000),
        (0, -1000000, 10000, 3000),
        (0, 0, 10000, 10000),
        (-800000, 0, 5000, 10000),
    ] {
        let mut d = elected_portfolio();
        let mut history = amt_only_capital_history();
        history.short = regular_short;
        history.deducted_loss = if regular_short < 0 { 300000 } else { 0 };
        let amt = history.amt.as_mut().unwrap();
        amt.short = amt_short;
        amt.deducted_loss = if amt_short < 0 { 300000 } else { 0 };
        // Each supplied negative ST balance uses $3,000 in the prior return.
        history.investment_parts = Some(InvestmentCapitalParts {
            short: (-regular_short - if regular_short < 0 { 300000 } else { 0 }).max(0),
            long: 0,
            amt: Some(InvestmentAmtCapitalParts {
                short: (-amt_short - if amt_short < 0 { 300000 } else { 0 }).max(0),
                long: 0,
            }),
        });
        d.screening.as_mut().unwrap().capital_history = vec![history];
        d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
            InvestmentElection::Allocated { capital: 300000, dividends: 1200000 };
        let r = supported(&d);
        assert_eq!(value(&r, "investment_elected_capital_gain"), 3000);
        assert_eq!(value(&r, "investment_elected_qualified_dividends"), 12000);
        assert_eq!(value(&r, "amt_investment_elected_capital_gain"), 3000);
        assert_eq!(value(&r, "amt_investment_elected_qualified_dividends"), 12000);
        assert_eq!(value(&r, "investment_interest_deduction"), 15000);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), 15000);
        assert_eq!(value(&r, "capital_gain"), regular_gain);
        assert_eq!(value(&r, "amt_capital_gain"), amt_gain);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn dividend_only_investment_election_survives_zero_amt_capital_pool() {
    let mut d = elected_portfolio();
    d.screening.as_mut().unwrap().capital_history = vec![amt_only_capital_history()];
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: 0, dividends: 1500000 };
    let r = supported(&d);
    assert_eq!(value(&r, "amt_capital_gain"), 0);
    assert_eq!(value(&r, "amt_investment_elected_capital_gain"), 0);
    assert_eq!(value(&r, "amt_investment_elected_qualified_dividends"), 15000);
    assert_eq!(value(&r, "investment_interest_deduction"), 15000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn amt_personal_sale_losses_are_disallowed_item_by_item() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(sale_with_amt(personal_sale(1000, 5000, GainTerm::Long), 500, 0)));
    add(&mut d, income(sale_with_amt(personal_sale(4000, 1000, GainTerm::Long), 9000, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 3000);
    assert_eq!(value(&r, "personal_sale_disallowed_loss"), 4000);
    assert_eq!(value(&r, "amt_capital_gain"), 500);
    assert_eq!(value(&r, "amt_personal_sale_disallowed_loss"), 5000);
    assert_eq!(value(&r, "amt_capital_loss_carryforward"), 0);
}

#[test]
fn current_amt_sale_loss_coordinates_atnol_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 100000, false)]);
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(1000, 1000, GainTerm::Long), 11000, 0)),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "prior_atnol_deduction"), 51300);
    // Loss absorption removes the 3,000 capital deduction before its 90% cap.
    assert_eq!(value(&r, "atnol_2025_used"), 54000);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 46000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 5700);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 7000);
}

#[test]
fn mortgage_certificate_allocates_whole_loan_interest_from_source_facts() {
    let mut d = complete(FederalStatus::Single);
    let deductions = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(deductions, 2_000_000);
    let mut credit = mortgage_certificate_credit(2000, 10000, 0);
    credit.current.as_mut().unwrap().interest = MortgageInterestSource::WholeLoan {
        paid_interest: 750_000,
        certified_debt: 10_000_000,
        original_loan_amount: 12_500_000,
    };
    d.credits.as_mut().unwrap().mortgage_credit = Some(credit);
    let r = supported(&d);
    // Pub. 530's $100,000 / $125,000 allocation: $7,500 × 80% = $6,000.
    assert_eq!(value(&r, "mortgage_certified_interest"), 6000);
    assert_eq!(value(&r, "mortgage_credit_current_potential"), 1200);
    assert_eq!(value(&r, "deduction"), 18800);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageInterestSource::WholeLoan { certified_debt, .. } = &mut certificate.interest {
        *certified_debt = 13_000_000;
    }
    let full_interest = supported(&d);
    assert_eq!(value(&full_interest, "mortgage_certified_interest"), 7500);
    assert_eq!(value(&full_interest, "mortgage_credit_current_potential"), 1500);
}

#[test]
fn completed_on_site_casualty_replacement_avoids_form_8828_recapture() {
    let mut d = complete(FederalStatus::Single);
    let ordinary_return = supported(&d);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::ExemptReplacedCasualty {
            destroyed_on: 20260401,
            replaced_on: 20270901,
            qualified_replacement_and_casualty_tax_resolved: true,
        },
    );
    let replaced = supported(&d);
    assert_eq!(value(&replaced, "mortgage_subsidy_1_recapture_tax"), 0);
    assert_eq!(value(&replaced, "mortgage_subsidy_recapture_tax"), 0);
    assert_eq!(replaced.tax, ordinary_return.tax);
    assert_eq!(supported(&restart(d.clone()).draft), replaced);

    if let MortgageSubsidyDisposition::ExemptReplacedCasualty {
        qualified_replacement_and_casualty_tax_resolved,
        ..
    } = mortgage_disposition(&mut d)
    {
        *qualified_replacement_and_casualty_tax_resolved = false;
    }
    unsupported(&d, "Mortgage subsidy recapture");
    if let MortgageSubsidyDisposition::ExemptReplacedCasualty {
        qualified_replacement_and_casualty_tax_resolved,
        replaced_on,
        ..
    } = mortgage_disposition(&mut d)
    {
        *qualified_replacement_and_casualty_tax_resolved = true;
        *replaced_on = 20290101;
    }
    assert!(!d.valid_stored());
}

#[test]
fn live_current_sstb_loss_reports_only_income_adjusted_carryforwards() {
    for (wages, qualified) in [(6_000_000, 4_000), (40_000_000, 0)] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        let id = d.next_id;
        add(
            &mut d,
            income(FederalIncome::Scorp { data: scorp_source(scorp_current_loss_statement()) }),
        );
        let r = estimate(&d);
        scorp_note(&r, id, "QBI loss passing passive limits", qualified);
        scorp_note(&r, id, "QBI income less released historical QBI losses", -qualified);
        assert!(!r.issues.iter().any(|i| i.section == "S-corporation QBI statement"));
        if qualified == 0 {
            assert!(
                !r.review_notes.iter().any(|n| n.contains("QBI loss passing basis limits: $6500."))
            );
        }
        assert!(r.tax.is_none());
    }
}

#[test]
fn nonparticipating_trading_loss_flows_through_regular_amt_and_niit() {
    let mut source = nonpassive_scorp(-15_000, 0);
    source.trade_kind = ScorpTradeKind::FinancialTrading;
    source.participation = Some(participation_facts());
    source.amt = Some(amt_records(&source));
    let (d, id) = investment_tax_draft(source, 10_000);
    let r = estimate(&d);
    passive_note(&r, id, 0, 0);
    amt_k1_note(&r, -8_000, -8_000);
    investment_tax_note(&r, 2_000, 76);
    k1_eic_investment_note(&r, 10_000);
}

#[test]
fn nonparticipating_trading_profit_cannot_release_unrelated_passive_loss() {
    let mut source = nonpassive_scorp(10_000, 0);
    source.trade_kind = ScorpTradeKind::FinancialTrading;
    source.participation = Some(participation_facts());
    let (mut d, _) = investment_tax_draft(source, 0);
    let rental_id = d.next_id;
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 0) }));
    let r = estimate(&d);
    passive_note(&r, rental_id, 0, 10_000); // high MAGI prevents special allowance
    investment_tax_note(&r, 10_000, 380);
    k1_eic_investment_note(&r, 0);
}

#[test]
fn trading_nonpassive_classification_preserves_qbi_loss_character_without_active_status() {
    let mut source = passive_qbi_source();
    source.trade_kind = ScorpTradeKind::FinancialTrading;
    let release = source
        .loss_qbi
        .after_passive(&source, crate::tests::support::builders::k1_amount(2_000))
        .unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (8_000, 4_000)
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            release.remaining.iter().map(|v| v.remaining.get()).sum::<i64>()
        ),
        2_000
    );
    let statement = ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: true,
        complete: true,
    };
    source.qbi_statement = Some(statement.clone());
    let prepared = statement.prepare(&source, 100_000, FederalStatus::Single).unwrap();
    let activities = statement
        .activities(&prepared, crate::tests::support::builders::k1_amount(2_000), 0)
        .unwrap();
    assert_eq!(activities[0].income, -4_000);
    assert!(!activities[0].active);
}
#[test]
fn nonparticipating_trading_loss_reduces_interest_capacity_after_loss_limits() {
    let d = trading_interest_draft(-15_000, false, 10_000);
    k1_interest_note(&estimate(&d), 2_000, 5_000); // regular at-risk loss 8k; AMT loss 5k
}

#[test]
fn ordinary_loan_gain_is_not_preferential_capital_after_short_term_loss_netting() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: scorp_source(scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount)),
        }),
    );
    for (term, proceeds, basis) in
        [(GainTerm::Long, 1_100_000, 100_000), (GainTerm::Short, 100_000, 900_000)]
    {
        add(
            &mut d,
            income(FederalIncome::Sale {
                data: sale_source(native_sale(term, proceeds, Some(basis))),
            }),
        );
    }
    investment_borrowing(&mut d, 5_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 2_000);
    assert_eq!(value(&r, "investment_interest_deduction"), 2_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 3_000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 2_000);
}
#[test]
fn amt_only_written_note_gain_uses_the_required_holding_period() {
    // A written note with principal repaid always carries its holding
    // period, even when only the AMT basis produces a gain.
    let mut source = repaid_written_note(GainTerm::Long);
    source.loans[0].basis = 1_000_000;
    source.loans[0].repayment_term = ScorpNoteHolding::NoRepayment;
    assert!(!source.valid());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source.clone()) }));
    source.loans[0].repayment_term = ScorpNoteHolding::Single { term: GainTerm::Long };
    d.income[1].income = FederalIncome::Scorp { data: scorp_source(source) };
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "amt_capital_gain"), 2_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 62_000);
    assert_eq!(value(&r, "scorp_amt_loan_gain_adjustment"), 0);
}
#[test]
fn written_note_long_term_gain_is_not_investment_interest_income_without_election() {
    for (term, deduction) in [(GainTerm::Short, 2_000), (GainTerm::Long, 0)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Scorp { data: scorp_source(repaid_written_note(term)) }));
        investment_borrowing(&mut d, 3_000, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), deduction);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), deduction);
        assert_eq!(value(&r, "investment_interest_carryforward"), 3_000 - deduction);
    }
}
#[test]
fn written_note_gain_nets_with_existing_capital_losses_once() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp { data: scorp_source(repaid_written_note(GainTerm::Long)) }),
    );
    add(&mut d, income(investment_sale_gain(GainTerm::Short, -3_000, -3_000)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -1_000);
    assert_eq!(value(&r, "amt_capital_gain"), -1_000);
    assert_eq!(value(&r, "agi"), 59_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    k1_eic_investment_note(&r, 0);
}

#[test]
fn nonpassive_property_and_ordinary_current_amounts_do_not_fund_other_passive_losses() {
    for (ordinary, property) in [(-5_000, 10_000), (10_000, -5_000)] {
        let source = former_passive_property_scorp(ordinary, property, 0, 0);
        let activity = source.pool_activity_result().unwrap();
        assert_eq!(activity.passive, 0);
        // 2026-10-19: the source activity is cents; the annual pool below is dollars.
        assert_eq!(activity.nonpassive, 500_000);
        assert_eq!(activity.characters.unwrap().nonpassive, [ordinary * 100, property * 100]);
        let (d, id) = passive_property_pool(source, -2_000);
        let mut r = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
        assert_eq!(pool.passive_loss, 2_000);
        assert_eq!(pool.income, ordinary);
        assert_eq!(pool.scorp_business, ordinary);
        assert_eq!(pool.scorp_nonpassive, ordinary);
        assert_eq!(pool.scorp_niit_adjustment, -ordinary);
        assert_eq!(pool.k1_section1231, vec![property_result(id, property, 0, property)]);
        unsupported(&d, "Section 1231 lookback");
    }
}

#[test]
fn current_sstb_preparation_applies_to_both_loss_characters_but_not_old_history() {
    let mut source = sole_qbi_loss(10_000, 100_000, 10_000);
    source.section1231 = Some(K1Section1231 { basis_loss: 400_000, ..k1_section1231(-20_000, 0) });
    let mut facts = property_qbi(10_000, 0);
    facts.losses.history = vec![loss_year(2023, 4_000, 4_000, 4_000, 4_000)];
    source.property_qbi = facts;
    source.qbi_statement.as_mut().unwrap().service = true;
    source.amt = Some(amt_records(&source));
    let statement = source.qbi_statement.as_ref().unwrap();
    for (taxable, current) in [(201_750, 10_000), (239_250, 5_000), (276_750, 0)] {
        let prepared = statement.prepare(&source, taxable, FederalStatus::Single).unwrap();
        assert_eq!(prepared.loss_qbi.current, current * 100);
        let qbi = &prepared.property_qbi;
        assert_eq!(qbi.losses.current, current * 100);
        assert_eq!(qbi.losses.history[0].remaining_qualified, 400_000);
        assert_eq!(
            crate::kernel::ratio::dollars(
                qbi.calculate(&prepared, &prepared.basis_result().unwrap()).unwrap().qbi.get()
            ),
            4_000 + current
        );
    }
    let (d, _) = passive_property_pool(source, 0);
    unsupported(&d, "Section 1231 lookback");
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains(
        "property QBI loss tracking: establish current-year service-business eligibility"
    )));
    assert!(!r.review_notes.iter().any(|n| n.contains("property loss passing basis limits")));
}

#[test]
fn shared_capital_stage_applies_property_and_portfolio_loss_offsets() {
    let input = crate::capital::CapitalAmounts {
        short: -2_000,
        long: 15_000,
        collectibles: 3_000,
        unrecaptured: 8_000,
    };
    let r = input.calculate([0, 4_000], false, false);
    assert_eq!((r.short, r.long, r.income, r.net_gain), (-2_000, 11_000, 9_000, 9_000));
    assert_eq!((r.line18, r.line19, r.collectible_gain, r.unrecaptured_gain), (0, 5_000, 0, 5_000));
    // Independent AMT history/amounts are evaluated without mutating regular.
    let a = crate::capital::CapitalAmounts { unrecaptured: 6_000, ..input }.calculate(
        [1_000, 0],
        false,
        false,
    );
    assert_eq!((a.income, a.unrecaptured_gain), (12_000, 6_000));
    assert_eq!(r.income, 9_000);
}

#[test]
fn shared_capital_stage_limits_both_special_rate_pools_to_total_gain() {
    let r = crate::capital::CapitalAmounts {
        short: 0,
        long: 7_000,
        collectibles: 6_000,
        unrecaptured: 5_000,
    }
    .calculate([0, 0], false, false);
    assert_eq!((r.net_gain, r.unrecaptured_gain, r.collectible_gain), (7_000, 5_000, 2_000));
}

#[test]
fn property_return_stage_coordinates_regular_and_amt_capital_with_portfolio_losses() {
    let mut source = former_passive_property_scorp(0, 20_000, 0, 0);
    source.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 1_000_000;
    let (mut d, _) = passive_property_pool(source, 0);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 8_000, 8_000)], &[]));
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(0, 10_000, GainTerm::Long), 10_000, 0)),
    );
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.starts_with("Property return-income stage: regular ordinary $8000; regular capital income after all netting $2000; AMT ordinary $0; AMT capital income $0.")));
    // No box 17 code V statement: the K-1 adds no QBI, so its ordinary
    // section 1231 item needs no property QBI records and the return computes.
    assert!(r.tax.is_some(), "{r:?}");
}

#[test]
fn property_return_stage_uses_passive_allowed_amount_not_gross_loss() {
    let mut source = passive_property_scorp(0, -20_000);
    source.property_qbi = property_qbi(20_000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, false).qbi_statement;
    let (mut d, _) = passive_property_pool(source, 10_000);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert!(r.review_notes.iter().any(|n| n.starts_with("Property return-income stage: regular ordinary $-10000; regular capital income after all netting $0;")));
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("combined business QBI after eligibility: current $-10000;"))
    );
    assert_eq!(r.tax, Some(502300));
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "regular_income_tax"), 5023);
}

#[test]
fn property_niit_distinguishes_passive_nonpassive_and_trading_gains() {
    for (material, trading, expected) in
        [(false, false, 8000), (true, false, -12000), (true, true, 8000)]
    {
        let mut source = if material {
            former_passive_property_scorp(0, 20000, 0, 0)
        } else {
            passive_property_scorp(0, 20000)
        };
        if trading {
            source.trade_kind = ScorpTradeKind::FinancialTrading;
        }
        source.amt = Some(amt_records(&source));
        let (mut draft, _) = passive_property_pool(source, 0);
        draft.screening.as_mut().unwrap().section1231 =
            Some(section1231_history(&[(2025, 8000, 8000)], &[]));
        let value = property_niit_worksheet(&draft, 12000, 12000).unwrap();
        assert_eq!(value.adjustment, expected);
        assert_eq!(value.next_capital_adjustment, 0);
        // Line 5a already contains $12,000 capital. Ordinary recapture is
        // included for passive/trading businesses; an active ordinary business
        // contributes neither its ordinary nor its capital property gain.
        assert_eq!(12000 + value.adjustment, if material && !trading { 0 } else { 20000 });
    }
}

#[test]
fn property_niit_ordinary_losses_use_allowed_activity_amounts() {
    for (source, other, expected) in [
        (former_passive_property_scorp(0, -20000, 0, 0), 0, 0),
        (passive_property_scorp(0, -20000), 10000, -10000),
    ] {
        let (mut draft, _) = passive_property_pool(source, other);
        draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        let value = property_niit_worksheet(&draft, 0, 0).unwrap();
        assert_eq!(value.adjustment, expected);
        assert_eq!(value.next_capital_adjustment, 0);
    }
}

#[test]
fn property_niit_excluded_capital_losses_keep_carryforward_difference() {
    let (mut draft, _) = passive_property_pool(former_passive_property_scorp(0, -10000, 0, 0), 0);
    add(
        &mut draft,
        income(FederalIncome::Scorp { data: scorp_source(passive_property_scorp(0, 20000)) }),
    );
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    // Net section 1231 gain $10,000, portfolio loss $20,000. Schedule D
    // deducts $3,000 and carries $7,000; NII investment gains net to zero.
    let value = property_niit_worksheet(&draft, -10000, -3000).unwrap();
    assert_eq!(value.adjustment, 3000);
    assert_eq!(value.next_capital_adjustment, 7000);
    assert_eq!(-3000 + value.adjustment, 0);
    // With only $5,000 portfolio loss no capital loss is deferred.
    let value = property_niit_worksheet(&draft, 5000, 5000).unwrap();
    assert_eq!(value.adjustment, 10000);
    assert_eq!(value.next_capital_adjustment, 0);
    assert_eq!(5000 + value.adjustment, 15000);
}

#[test]
fn property_lookback_does_not_guess_partial_qualified_gain_allocation() {
    let mut source = former_passive_property_scorp(0, 20000, 0, 0);
    source.property_qbi = property_qbi(0, 10000);
    let facts = &source.property_qbi;
    assert!(
        facts.qualified_recapture(&source, 8000).unwrap_err().1.contains("source-gain allocation")
    );
    // Full recapture determines the entire qualified portion even without
    // individual asset allocation. No-recapture has no ordinary QBI gain.
    assert_eq!(facts.qualified_recapture(&source, 20000).unwrap(), 10000);
    assert_eq!(facts.qualified_recapture(&source, 0).unwrap(), 0);
    assert!(facts.qualified_recapture(&source, 20001).is_err());
}

#[test]
fn property_gain_return_has_reconciled_1040_without_lookback() {
    for material in [false, true] {
        let draft = complete_property_gain_draft(material, 0);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 80000);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(value(&r, "taxable_income"), 63900);
        // Wages less standard deduction: $43,900 ordinary taxable income.
        // $5,550 capital at 0%, remaining $14,450 at 15% = $2,168 rounded.
        // Ordinary tax: $1,240 + $31,500 * 12% = $5,020.
        assert_eq!(value(&r, "regular_income_tax"), 7191);
        assert_eq!(return_line(&r, "7a"), 2000000);
        assert_eq!(return_line(&r, "8"), 0);
        assert_eq!(return_line(&r, "9"), 8000000);
        assert_eq!(value(&r, "eic_investment_income"), if material { 0 } else { 20000 });
        assert_eq!(value(&r, "net_investment_income_tax"), 0);
        assert!(r.tax.is_some());
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn property_lookback_return_has_ordinary_qbi_and_remaining_capital() {
    let r = supported(&complete_property_gain_draft(true, 8000));
    assert_eq!(value(&r, "agi"), 80000);
    assert_eq!(value(&r, "qbi_deduction"), 1600);
    assert_eq!(value(&r, "taxable_income"), 62300);
    assert_eq!(return_line(&r, "7a"), 1200000);
    assert_eq!(return_line(&r, "8"), 800000);
    assert_eq!(return_line(&r, "9"), 8000000);
    // Ordinary $50,300: $1,240 + $37,900 * 12% = $5,788;
    // all $12,000 long-term gain at 15% = $1,800.
    assert_eq!(value(&r, "regular_income_tax"), 7591);
    assert_eq!(value(&r, "eic_investment_income"), 0);
    assert!(r.tax.is_some());
}

#[test]
fn completed_property_gain_return_distinguishes_niit_and_amt_sources() {
    for material in [false, true] {
        let mut draft = complete_property_gain_draft(material, 0);
        if let FederalIncome::Wage { data } = &mut draft.income[0].income {
            let mut data = edit_wage(data);
            data.wages = 30000000;
        }
        if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
            let mut data = edit_scorp(data);
            data.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 1000000;
        }
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 320000);
        assert_eq!(value(&r, "amt_capital_gain"), 10000);
        assert_eq!(return_line(&r, "7a"), 2000000);
        // NIIT uses regular $20,000, never the different AMT gain.
        assert_eq!(value(&r, "net_investment_income_tax"), if material { 0 } else { 760 });
        assert!(r.tax.is_some());
    }
}

#[test]
fn completed_property_gain_history_retains_years_after_recapture_and_expiry() {
    let mut draft = complete_property_gain_draft(true, 0);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(
        &[(2021, 30000, 25000), (2025, 10000, 10000)],
        &[(2025, 5000, 5000)],
    ));
    let r = supported(&draft);
    // $20,000 regular recapture uses the oldest year; its remaining $5,000
    // expires. AMT recaptures only its independent $5,000 history.
    assert_eq!(value(&r, "section1231_current_loss_history"), 0);
    assert_eq!(value(&r, "section1231_2025_next_lookback"), 10000);
    assert!(!r.lines.iter().any(|v| v.id == "section1231_2021_next_lookback"));
    assert!(!r.lines.iter().any(|v| v.id == "amt_section1231_2025_next_lookback"));
    assert_eq!(value(&r, "amt_section1231_current_loss_history"), 0);
}

#[test]
fn property_loss_history_is_not_limited_to_current_nol() {
    let mut source = former_passive_property_scorp(0, -100000, 0, 0);
    source.property_qbi = property_qbi(100000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    let (mut draft, _) = passive_property_pool(source, 0);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), -40000);
    assert_eq!(value(&r, "nol_carryforward"), 40000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 100000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 100000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 100000);
    assert_eq!(return_line(&r, "8"), -10000000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn property_capital_needs_election_and_material_trading_is_excluded_from4952() {
    let mut draft = complete_trading_property_interest(10000, 20000);
    let r = supported(&draft);
    assert_eq!(value(&r, "investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 15000);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.participation.as_mut().unwrap().minutes = 30001;
    }
    let r = supported(&draft);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 0);
    assert_eq!(value(&r, "investment_interest_carryforward"), 25000);
}

#[test]
fn ordinary_trading_property_loss_nets_with_disposition_gains_before_election() {
    let mut draft = complete_trading_property_interest(10000, -20000);
    add(
        &mut draft,
        income(sale_with_amt(ordinary_investment_sale(30000, 0, GainTerm::Long), 0, 0)),
    );
    draft.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 500000 };
    let r = supported(&draft);
    // Net disposition gain $10k is all net capital gain for line 4e;
    // only the elected $5k enters alongside $10k ordinary trading income.
    assert_eq!(value(&r, "investment_interest_deduction"), 15000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 15000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 10000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 20000);
    assert_eq!(value(&r, "agi"), 80000);
    assert_eq!(return_line(&r, "7a"), 3000000);
    assert_eq!(return_line(&r, "8"), -1000000);
}

#[test]
fn property_excess_loss_does_not_invent_mixed_source_allocations() {
    let mut draft = property_excess_draft(400000, 300000, 500000, 400000, FederalStatus::Single);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.ordinary = -100000;
    drop(data);
    unsupported(&draft, "Property excess business loss");
    let mut draft = property_excess_draft(400000, 300000, 500000, 400000, FederalStatus::Single);
    add(
        &mut draft,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(10000, true)) }),
    );
    unsupported(&draft, "Property excess business loss");
    // Regular allocation can be resolved while independent AMT source cannot.
    let mut draft = property_excess_draft(400000, 300000, 500000, 400000, FederalStatus::Single);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.amt.as_mut().unwrap().adjustment = -100000;
    drop(data);
    unsupported(&draft, "Property excess business loss");
}

#[test]
fn multiple_property_excess_losses_reconcile_different_qualified_proportions() {
    for (first, second, current, deferred) in
        [(200000, 100000, 192000, 108000), (0, 200000, 128000, 72000), (0, 0, 0, 0)]
    {
        let mut draft = property_excess_draft(200000, 150000, 300000, first, FederalStatus::Single);
        let mut other =
            property_excess_draft(200000, 100000, 300000, second, FederalStatus::Single);
        add(&mut draft, other.income.remove(1));
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), -196000);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 144000);
        assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 0);
        assert_eq!(value(&r, "qbi_loss_carryforward"), current);
        assert_eq!(value(&r, "nol_carryforward"), 340000);
        assert_eq!(value(&r, "amt_nol_carryforward"), 190000);
        assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
        assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 250000);
        assert_eq!(return_line(&r, "8"), -25600000);
        assert_eq!(r.tax, Some(0));
        assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "K-1 excess-loss character: NOL carryforward $144000; qualified portion ${deferred}."
        ))));
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn operating_loss_property_gain_defers_operating_qbi_without_reducing_recapture() {
    for (qualified, recapture, gain, current, deferred) in [
        (400000, 0, 100000, 356000, 44000),
        (400000, 20000, 100000, 336000, 44000),
        (200000, 20000, 100000, 158000, 22000),
        (400000, 20000, 0, 356000, 44000),
    ] {
        let draft = operating_loss_property_gain_draft(qualified, recapture, gain);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), -196000);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 44000);
        assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 0);
        assert_eq!(value(&r, "qbi_loss_carryforward"), current);
        assert_eq!(value(&r, "nol_carryforward"), 240000);
        assert_eq!(value(&r, "amt_nol_carryforward"), 190000);
        assert_eq!(value(&r, "section1231_current_loss_history"), 0);
        assert_eq!(value(&r, "amt_section1231_current_loss_history"), 0);
        assert_eq!(return_line(&r, "7a"), (100000 - recapture) * 100);
        assert_eq!(return_line(&r, "8"), (-356000 + recapture) * 100);
        assert_eq!(r.tax, Some(0));
        assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "K-1 excess-loss character: NOL carryforward $44000; qualified portion ${deferred}."
        ))));
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn operating_loss_property_gain_uses_allowed_loss_and_current_sstb_eligibility() {
    let mut draft = operating_loss_property_gain_draft(200000, 20000, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.opening = Some(26000000); // Gain raises basis to $360k; $40k stays suspended.
    drop(data);
    let r = supported(&draft);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 4000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 158000);
    assert_eq!(value(&r, "nol_carryforward"), 200000);
    assert_eq!(r.tax, Some(0));
    let mut draft = operating_loss_property_gain_draft(400000, 20000, 100000);
    draft.income[0] = wage(1, Owner::Taxpayer, 51135000, 0);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.qbi_statement.as_mut().unwrap().service = true;
    drop(data);
    let r = supported(&draft);
    // SSTB 50%: loss $200k, qualified recapture $10k, deferral $22k.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 168000);
    assert_eq!(value(&r, "taxable_income"), 239250);
    // $159,250 ordinary taxed at $30,818; $80k capital taxed at 15%.
    assert_eq!(value(&r, "regular_income_tax"), 42818);
    assert_eq!(return_line(&r, "16"), 4281800);
    assert_eq!(r.tax, Some(4562000));
}

#[test]
fn property_loss_operating_profit_defers_property_qbi_and_keeps_profit() {
    for (qualified_loss, qualified_profit, current, deferred) in [
        (400000, 100000, 256000, 44000),
        (200000, 100000, 78000, 22000),
        (400000, 0, 356000, 44000),
        (0, 100000, 0, 0),
    ] {
        let draft = property_loss_operating_profit_draft(qualified_loss, qualified_profit);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), -196000);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 44000);
        assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 0);
        assert_eq!(value(&r, "qbi_loss_carryforward"), current);
        assert_eq!(value(&r, "nol_carryforward"), 240000);
        assert_eq!(value(&r, "amt_nol_carryforward"), 140000);
        // $400k property loss less $44k deferred. The $100k operating
        // profit absorbs loss for Form 461 but is not section 1231 gain.
        assert_eq!(value(&r, "section1231_2026_next_lookback"), 356000);
        assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 300000);
        assert_eq!(return_line(&r, "8"), -25600000);
        assert_eq!(r.tax, Some(0));
        assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "K-1 excess-loss character: NOL carryforward $44000; qualified portion ${deferred}."
        ))));
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn multiple_full_recapture_allocates_trading_investment_interest() {
    let mut draft = multiple_fully_recaptured_gains(30000);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.trade_kind = ScorpTradeKind::FinancialTrading;
        data.qbi_statement.as_mut().unwrap().service = true;
    }
    investment_borrowing(&mut draft, 25000, 0, 0);
    let r = supported(&draft);
    assert_eq!(value(&r, "investment_interest_deduction"), 20000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 5000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 15000);
    assert_eq!(value(&r, "agi"), 110000);
    assert_eq!(return_line(&r, "7a"), 0);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert!(r.tax.is_some());
}

#[test]
fn partial_recapture_uses_source_rate_classes_before_general_gains() {
    for (recapture, deduction, tax) in
        [(8000, 1600, 12924), (20000, 4000, 12396), (35000, 7000, 12786)]
    {
        let mut draft = rate_ordered_recapture(recapture, 5000);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 110000);
        assert_eq!(value(&r, "qbi_deduction"), deduction);
        assert_eq!(value(&r, "regular_income_tax"), tax);
        assert_eq!(return_line(&r, "7a"), (50000 - recapture) * 100);
        assert_eq!(return_line(&r, "8"), recapture * 100);
        assert_eq!(value(&r, "amt_capital_gain"), 25000);
        assert_eq!(restart(draft.clone()).draft, draft);
        draft.income.swap(1, 2);
        let reordered = supported(&draft);
        assert_eq!(reordered.tax, r.tax);
        assert_eq!(value(&reordered, "qbi_deduction"), deduction);
    }
}

#[test]
fn rate_ordered_recapture_preserves_niit_and_amt_interest_sources() {
    let mut draft = rate_ordered_recapture(8000, 5000);
    draft.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    let r = supported(&draft);
    assert_eq!(value(&r, "regular_income_tax"), 77874);
    assert_eq!(value(&r, "net_investment_income_tax"), 760);
    assert_eq!(r.tax, Some(7953400));
    draft.income[0] = wage(1, Owner::Taxpayer, 6000000, 0);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.trade_kind = ScorpTradeKind::FinancialTrading;
        data.qbi_statement.as_mut().unwrap().service = true;
    }
    investment_borrowing(&mut draft, 25000, 0, 0);
    let r = supported(&draft);
    assert_eq!(value(&r, "investment_interest_deduction"), 8000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 5000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 17000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 20000);
}

#[test]
fn qualified_rate_facts_resolve_mixed_property_gain_recapture() {
    for (qualified, high, recapture, deduction, tax) in [
        (8000, 0, 8000, 0, 8216),
        (12000, 12000, 8000, 1600, 7864),
        (8000, 0, 16000, 800, 8320),
        (12000, 12000, 16000, 2400, 7968),
        (10000, 2000, 12000, 400, 8128),
        (10000, 2000, 16000, 1200, 8232),
    ] {
        let draft = qualified_rate_gain(qualified, Some(high), recapture);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 80000);
        assert_eq!(value(&r, "qbi_deduction"), deduction);
        assert_eq!(value(&r, "regular_income_tax"), tax);
        assert_eq!(return_line(&r, "7a"), (20000 - recapture) * 100);
        assert_eq!(return_line(&r, "8"), recapture * 100);
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn qualified_rate_facts_do_not_invent_within_class_or_loss_allocation() {
    let unknown = qualified_rate_gain(8000, None, 8000);
    let result = supported(&unknown);
    assert!(
        result
            .review_notes
            .iter()
            .any(|note| note.display.key == "WARN_SCORP_QBI_ALLOCATION_LEFT_OUT")
    );
    unsupported(&qualified_rate_gain(10000, Some(2000), 8000), "Section 1231 QBI character");
    let draft = qualified_rate_gain(8000, Some(0), 8000);
    let FederalIncome::Scorp { data } = &draft.income[1].income else { panic!() };
    let mut source = ScorpK1::from(data);
    source.section1231.as_mut().unwrap().passive_loss = 10000;
    assert!(source.property_qbi.qualified_recapture(&source, 8000).is_err());
}

#[test]
fn property_history_released_by_other_passive_income_remains_guarded() {
    let mut draft = trading_with_unreleased_histories();
    add(
        &mut draft,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(2000, false)) }),
    );
    unsupported(&draft, "Property investment interest");
}

#[test]
fn base_year_capital_gain_bands_do_not_use_current_year_limits() {
    use crate::income_tax::Rates;
    use FederalStatus::*;
    // Whole-dollar tax on all-preferential income $10,000 above the 15% ceiling.
    let fixtures = [
        (2023, Single, 44625, 502300, 69151),
        (2023, Joint, 89250, 563850, 71690),
        (2023, Head, 59750, 533050, 71495),
        (2023, Separate, 44625, 286900, 36841),
        (2024, Single, 47025, 528900, 72781),
        (2024, Joint, 94050, 593750, 75455),
        (2024, Head, 63000, 561350, 75253),
        (2024, Separate, 47025, 301850, 38724),
        (2025, Single, 48350, 543400, 74758),
        (2025, Joint, 96700, 610050, 77503),
        (2025, Head, 64750, 576700, 77293),
        (2025, Separate, 48350, 310000, 39748),
    ];
    for (year, status, zero, income, tax) in fixtures {
        let rates = Rates::for_year(year, status).unwrap();
        assert_eq!(rates.schedule_d(zero, zero, 0, 0, false), 0);
        assert_eq!(rates.schedule_d(zero + 10000, zero + 10000, 0, 0, false), 1500);
        assert_eq!(rates.schedule_d(income, income, 0, 0, false), tax, "{year} {status:?}");
        if status == Joint {
            assert_eq!(
                Rates::for_year(year, Surviving).unwrap().schedule_d(income, income, 0, 0, false),
                tax
            );
        }
    }
}

#[test]
fn base_year_special_gains_cross_the_correct_24_percent_ceiling() {
    use crate::income_tax::Rates;
    for (year, income, at_25, at_28) in
        [(2023, 183100, 37354, 37384), (2024, 192950, 39361, 39391), (2025, 198300, 40449, 40479)]
    {
        let rates = Rates::for_year(year, FederalStatus::Single).unwrap();
        // $1,000 of the $2,000 special gain remains in the 24% band;
        // only the last $1,000 reaches the special rate ceiling.
        assert_eq!(rates.schedule_d(income, 2000, 2000, 2000, false), at_25);
        assert_eq!(rates.schedule_d(income, 2000, 2000, 0, true), at_28);
    }
}

#[test]
fn mixed_source_excess_loss_has_one_loss_character() {
    for foreign_loss in [true, false] {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, wage(0, Owner::Taxpayer, 30000000, 0));
        add(&mut d, income(cash_business(if foreign_loss { 20000 } else { -300000 })));
        add(&mut d, income(foreign_service_business(if foreign_loss { -300000 } else { 20000 })));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 44000);
        assert_eq!(r.tax, Some(if foreign_loss { 352300 } else { 400300 }));
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "excess_business_loss"), 24000);
        assert_eq!(value(&r, "nol_carryforward"), 24000);
        assert_eq!(value(&r, "current_net_operating_loss"), 0);
        assert_eq!(value(&r, "qbi_deduction"), if foreign_loss { 4000 } else { 0 });
        assert_eq!(value(&r, "qbi_loss_carryforward"), if foreign_loss { 0 } else { 276000 });
        assert_eq!(
            value(
                &r,
                if foreign_loss {
                    "business_nonqualified_excess_loss"
                } else {
                    "business_qualified_excess_loss"
                }
            ),
            24000
        );
        let mut service = d.clone();
        let FederalIncome::Business { data } = &mut service.income[1].income else {
            unreachable!()
        };
        data.specified_service = true;
        let service_result = supported(&service);
        assert_eq!(service_result.tax, r.tax);
        assert_eq!(
            value(&service_result, "qbi_loss_carryforward"),
            value(&r, "qbi_loss_carryforward")
        );
        assert_eq!(value(&service_result, "qbi_deduction"), value(&r, "qbi_deduction"));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income.reverse();
        assert_eq!(supported(&d).tax, r.tax);
        add(
            &mut d,
            income(if foreign_loss {
                cash_business(-10000)
            } else {
                foreign_service_business(-10000)
            }),
        );
        unsupported(&d, "Foreign business loss allocation");
    }
}

#[test]
fn housing_relief_does_not_inflate_net_investment_magi() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 50000, ForeignHousingLocation::General);
    add(&mut d, income(savings_income(20000, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "net_investment_modified_agi"), 201394);
    assert_eq!(value(&r, "net_investment_income_tax"), 53);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn general_rule_delayed_installments_preserve_original_ratio_and_cost_cap() {
    let mut a = fixed_annuity();
    a.started = 20250101;
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 0 };
    a.cost = 1080000;
    a.periodic = 10000;
    a.received = 15; // IRS twelve current installments plus three from a prior year.
    a.recovered = 40500; // Nine prior installments at $45 each.
    assert_eq!(a.ratio(), 450);
    assert!(a.valid(150000));
    assert_eq!(a.excluded(), 67500);
    for (recovered, exclusion) in [(40500, 67500), (1050000, 30000)] {
        a.recovered = recovered;
        a.started = if recovered == 40500 { 20250101 } else { 20000101 };
        let mut d = complete(FederalStatus::Single);
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            1500,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data } = &mut entry.income {
            let mut data = edit_retirement(data);
            data.taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
        }
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), 150000);
        assert_eq!(return_line(&r, "5b"), 150000 - exclusion);
        assert_eq!(restart(d.clone()).draft, d);
    }
    a.started = 20251201;
    a.received = 14;
    assert!(!a.valid(140000)); // Only 13 monthly periods exist by year-end.
    a.received = 13;
    assert!(a.valid(130000));
    a.started = 20260101;
    assert!(!a.valid(130000));
    a.started = 20270101;
    assert!(!a.valid(130000)); // Invalid dates short-circuit the month arithmetic.
}

#[test]
fn guaranteed_total_reproduces_irs_fractional_installment_example() {
    let mut a = fixed_annuity();
    a.cost = 2105300;
    a.periodic = 10000;
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 2105300 };
    // Pub. 939: $21,053 / $1,200 = 17.544... years -> Table VII 18 years.
    assert!(a.valid(120000));
    assert_eq!(a.refund_adjustment(), 315800);
    assert_eq!(a.cost - a.refund_adjustment(), 1789500);
    assert_eq!(a.ratio(), 746);
    assert_eq!(a.excluded(), 89520);
    let mut d = complete(FederalStatus::Single);
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1200,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        let mut data = edit_retirement(data);
        data.taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 30500);
    assert_eq!(value(&r, "annuity_2_refund_adjustment"), 3158);
    assert_eq!(restart(d.clone()).draft, d);
    a.partial = 1;
    assert!(!a.valid(120001)); // A fractional original first installment needs separate timing.
    a.partial = 0;
    a.started = 20000101;
    // The $53 final guaranteed installment is still due at period 211.
    assert!(!a.ends_at_death(20170731));
    assert!(a.ends_at_death(20170801));
}

// P.L. 119-108 section 2 (Sept. 11, 2026) ended P.L. 119-21 section 70438's
// window: a presidential major disaster whose incident period began on or
// after December 28, 2019 and before January 1, 2027 is a qualified disaster
// area, so a 2026 DR- event is a qualified disaster loss. An emergency (EM-)
// declaration is a federally declared disaster but not a qualified one.
#[test]
fn a_2026_major_disaster_is_a_qualified_disaster_loss() {
    let mut d = complete(FederalStatus::Single);
    let mut event = major_disaster("Major storm", 3_000_000);
    assert!(event.qualified());
    d.deductions.as_mut().unwrap().personal_casualties.push(event.clone());
    assert!(d.valid_stored());
    // An emergency declaration is not a major disaster, and a DR- number is
    // not an emergency declaration.
    event.declaration = CasualtyDeclaration::Federal { reference: "EM-3999".into() };
    assert!(!event.qualified());
    d.deductions.as_mut().unwrap().personal_casualties = vec![event.clone()];
    assert!(d.valid_stored());
    event.declaration = CasualtyDeclaration::Federal { reference: "DR-1234".into() };
    d.deductions.as_mut().unwrap().personal_casualties = vec![event.clone()];
    assert!(!d.valid_stored());
    event.declaration = CasualtyDeclaration::MajorDisaster { reference: "EM-3999".into() };
    d.deductions.as_mut().unwrap().personal_casualties = vec![event];
    assert!(!d.valid_stored());
}

#[test]
fn used_goods_report_section_a_and_b_values_by_similar_property_group() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut clothing = used_goods(500_001);
    clothing.basis = 1_000_000;
    clothing.value_method = ValueMethod::Appraisal;
    clothing.appraisal = Some(2);
    ded.appraisals.push(gift_appraisal(2, 20261115));
    d.next_id = 3;
    let mut household = used_goods(500_000);
    household.description = "Used table".into();
    household.kind =
        GiftKind::Goods { kind: GoodsKind::Household, condition: GoodsCondition::Good };
    household.basis = 1_000_000;
    ded.property_gifts.extend([clothing, household]);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&d);
    assert_eq!(value(&result, "form_8283_section_a_value"), 5_000);
    assert_eq!(value(&result, "form_8283_section_b_value"), 5_000);
    assert_eq!(line_amount(&result, "form_8283_section_b_value"), 500_000);
    assert_eq!(d.deductions.as_ref().unwrap().property_gifts[0].value, 500_001);
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn whaling_cap_and_mixed_use_boat_do_not_enter_direct_cash_or_property_gifts() {
    let mut draft = complete(FederalStatus::Single);
    let ded = draft.deductions.as_mut().unwrap();
    ded.whaling_captains.push(whaling_claim(vec![
        whaling_expense(
            WhalingExpenseKind::Boat { use_mode: WhalingBoatUse::Mixed { sanctioned_days: 100 } },
            365_000,
        ),
        whaling_expense(WhalingExpenseKind::WeaponsAndGear, 6_000_000),
    ]));
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_whaling_eligible_expenses"), 61_000);
    assert_eq!(value(&result, "charity_whaling_statutory_claim"), 50_000);
    assert_eq!(value(&result, "charity_whaling_deduction"), 17_700);
    assert_eq!(value(&result, "charity_whaling_return_year_carryforward"), 32_300);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 0);
    assert!(result.lines.iter().all(|line| !line.id.starts_with("form_8283")));
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_whaling_deduction"), 17_700);
    assert_eq!(value(&automatic, "charity_whaling_return_year_carryforward"), 32_300);
}

#[test]
fn current_charity_uses_its_limits_before_2025_stock_carry_without_duplication() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.charity_carryovers.push(prior_public_stock_carry());
    ded.set_cash_charity(500_000);
    let mut ordinary_stock = public_stock_gift();
    ordinary_stock.acquired = 20260501;
    ordinary_stock.value = 300_000;
    let mut appreciated_stock = public_stock_gift();
    appreciated_stock.value = 800_000;
    ded.property_gifts.extend([ordinary_stock, appreciated_stock]);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "charity_cash_sixty_current_deduction"), 5_000);
    assert_eq!(value(&mixed, "charity_property_fifty_current_deduction"), 3_000);
    assert_eq!(value(&mixed, "charity_capital_gain_thirty_current_deduction"), 8_000);
    assert_eq!(value(&mixed, "charity_carry_2025_capital_gain_thirty_used"), 10_000);
    assert_eq!(value(&mixed, "charity_carry_2025_capital_gain_thirty_deduction"), 9_700);
    assert_eq!(value(&mixed, "charity_carry_2025_capital_gain_thirty_floor_not_carried"), 300);
    assert_eq!(value(&mixed, "charity_carry_2025_capital_gain_thirty_remaining"), 0);
    assert_eq!(value(&mixed, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&mixed, "12e"), 2_570_000);
    assert_eq!(estimate(&restart(d.clone()).draft), mixed);

    d.deductions.as_mut().unwrap().property_gifts[1].value = 1_500_000;
    let partial = supported(&d);
    assert_eq!(value(&partial, "charity_capital_gain_thirty_current_deduction"), 15_000);
    assert_eq!(value(&partial, "charity_carry_2025_capital_gain_thirty_used"), 3_000);
    assert_eq!(value(&partial, "charity_carry_2025_capital_gain_thirty_deduction"), 2_700);
    assert_eq!(value(&partial, "charity_carry_2025_capital_gain_thirty_floor_not_carried"), 300);
    assert_eq!(value(&partial, "charity_carry_2025_capital_gain_thirty_remaining"), 7_000);
    assert_eq!(value(&partial, "charity_return_year_carryforward"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), partial);

    d.deductions.as_mut().unwrap().property_gifts[1].value = 2_000_000;
    let current_first = supported(&d);
    assert_eq!(value(&current_first, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(
        value(&current_first, "charity_capital_gain_thirty_return_year_carryforward"),
        2_300
    );
    assert_eq!(value(&current_first, "charity_carry_2025_capital_gain_thirty_used"), 0);
    assert_eq!(
        value(&current_first, "charity_carry_2025_capital_gain_thirty_floor_not_carried"),
        0
    );
    assert_eq!(value(&current_first, "charity_carry_2025_capital_gain_thirty_remaining"), 10_000);
    assert_eq!(value(&current_first, "charity_return_year_carryforward"), 2_300);
    assert_eq!(estimate(&restart(d.clone()).draft), current_first);
}

#[test]
fn short_held_and_loss_value_public_stock_use_fifty_percent_property_class() {
    let mut d = complete(FederalStatus::Single);
    let mut short_held = public_stock_gift();
    short_held.acquired = 20260115;
    short_held.value = 100_000;
    short_held.basis = 80_000;
    let mut loss_value = public_stock_gift();
    loss_value.description = "Synthetic second public stock".into();
    loss_value.value = 100_000;
    loss_value.basis = 150_000;
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.extend([short_held, loss_value]);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let ordinary = supported(&d);
    assert_eq!(value(&ordinary, "charity_property_1_claim"), 800);
    assert_eq!(value(&ordinary, "charity_property_2_claim"), 1_000);
    assert_eq!(value(&ordinary, "charity_property_fifty_current_deduction"), 1_500);
    assert_eq!(value(&ordinary, "form_8283_section_a_value"), 1_800);
    assert_eq!(value(&ordinary, "charity_return_year_carryforward"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), ordinary);

    d.deductions.as_mut().unwrap().property_gifts.push(public_stock_gift());
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "charity_property_fifty_current_deduction"), 1_800);
    assert_eq!(value(&mixed, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(value(&mixed, "form_8283_section_a_value"), 21_800);
    assert_eq!(value(&mixed, "charity_return_year_carryforward"), 2_300);

    // A single page election applies to every eligible long-term public-charity gift.
    d.deductions.as_mut().unwrap().basis_election = true;
    let elected = supported(&d);
    assert_eq!(value(&elected, "charity_property_fifty_current_deduction"), 6_500);
    assert_eq!(value(&elected, "charity_capital_gain_thirty_current_deduction"), 0);
}

#[test]
fn original_issue_qsbs_excludes_eligible_gain_without_amt_or_niit() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        qsb_income(&"Synthetic Software legal issuer 🧾".repeat(20), 10_000_000, 110_000_000, 0),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "qsbs_2_realized_gain"), 1_000_000);
    assert_eq!(value(&r, "qsbs_2_excluded_gain"), 1_000_000);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "amt_capital_gain"), 0);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn qsbs_dollar_cap_excess_has_ordinary_long_term_character() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = qsb_income("Synthetic Software Inc.", 10_000_000, 1_310_000_000, 0);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut entry.income {
        data.withholding = 120_000;
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(value(&r, "qsbs_2_exclusion_limit"), 10_000_000);
    assert_eq!(value(&r, "qsbs_2_excluded_gain"), 10_000_000);
    assert_eq!(value(&r, "qsbs_2_taxable_gain"), 3_000_000);
    assert_eq!(value(&r, "capital_gain"), 3_000_000);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    // Form 8960 limits the $3m investment gain by MAGI above the $200k threshold.
    assert_eq!(value(&r, "net_investment_income_tax"), 108_680);
    assert_eq!(return_line(&r, "25b"), 120_000);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn older_qsbs_uses_fifty_or_seventy_five_percent_and_amt_preference() {
    for (issued, excluded, taxable, amt_preference) in
        [(20000101, 500_000, 500_000, 35_000), (20090601, 750_000, 250_000, 52_500)]
    {
        let mut d = complete(FederalStatus::Single);
        let mut entry = qsb_income("Synthetic A Inc.", 10_000_000, 110_000_000, 0);
        if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut entry.income {
            data.lots[0].issued = issued;
        }
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(value(&r, "qsbs_2_excluded_gain"), excluded);
        assert_eq!(value(&r, "qsbs_2_taxable_gain"), taxable);
        assert_eq!(value(&r, "qsbs_2_rate_28_gain"), taxable);
        assert_eq!(value(&r, "rate_28_gain"), taxable);
        assert_eq!(value(&r, "qsbs_2_amt_preference"), amt_preference);
        assert_eq!(value(&r, "capital_gain"), taxable);
        assert_eq!(estimate(&restart(d.clone()).draft), r);
    }
}

#[test]
fn partial_qsbs_exclusion_does_not_put_issuer_cap_excess_in_28_percent_pool() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = qsb_income("Synthetic A Inc.", 10_000_000, 1_310_000_000, 0);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut entry.income {
        data.lots[0].issued = 20090601;
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(value(&r, "qsbs_2_excluded_gain"), 7_500_000);
    assert_eq!(value(&r, "qsbs_2_taxable_gain"), 5_500_000);
    assert_eq!(value(&r, "qsbs_2_rate_28_gain"), 2_500_000);
    assert_eq!(value(&r, "rate_28_gain"), 2_500_000);
    assert_eq!(value(&r, "qsbs_2_amt_preference"), 525_000);
}

#[test]
fn older_qsbs_unavailable_zone_valuation_and_unbound_mixed_cohorts() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = qsb_income("Synthetic A Inc.", 10_000_000, 110_000_000, 0);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut entry.income {
        data.lots[0].issued = 20050101;
    }
    add(&mut d, entry);
    // Absent for a lot in the zone interval: assumed No, with a warning.
    let absent = supported(&d);
    assert!(noted(&absent, "WARN_QSBS_ZONE_ASSUMED_NO"), "{absent:?}");
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.zone_60_exclusion = Some(crate::QsbsZoneCertification::No);
    }
    let without_zone = supported(&d);
    assert!(!noted(&without_zone, "WARN_QSBS_ZONE_ASSUMED_NO"));
    assert_eq!(absent.tax, without_zone.tax);
    // Yes with no zone gain entered: no zone increase, with the warning.
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_QSBS_ZONE_ASSUMED_NO"), "{r:?}");
    assert_eq!(r.tax, without_zone.tax);
    // An incomplete issuer history: no exclusion, the whole gain is taxed.
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.zone_60_exclusion = Some(crate::QsbsZoneCertification::No);
        data.complete_history = Some(false);
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_QSBS_EXCLUSION_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "qsbs_2_taxable_gain"), 1_000_000);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.complete_history = Some(true);
        data.lots.push(QsbStockLot {
            issued: 20150101,
            sold: 20260601,
            basis: 10_000_000,
            proceeds: 110_000_000,
            zone_gain_through_2018: None,
        });
    }
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "qsbs_2_excluded_gain"), 1_500_000);
    assert_eq!(value(&mixed, "qsbs_2_taxable_gain"), 500_000);
    assert_eq!(value(&mixed, "qsbs_2_rate_28_gain"), 500_000);
    assert_eq!(value(&mixed, "qsbs_2_amt_preference"), 35_000);

    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.lots.push(QsbStockLot {
            issued: 20090601,
            sold: 20260601,
            basis: 10_000_000,
            proceeds: 110_000_000,
            zone_gain_through_2018: None,
        });
    }
    let all_three = supported(&d);
    assert_eq!(value(&all_three, "qsbs_2_excluded_gain"), 2_250_000);
    assert_eq!(value(&all_three, "qsbs_2_taxable_gain"), 750_000);
    assert_eq!(value(&all_three, "qsbs_2_rate_28_gain"), 750_000);
    assert_eq!(value(&all_three, "qsbs_2_amt_preference"), 87_500);
    assert_eq!(estimate(&restart(d.clone()).draft), all_three);

    let mut split = d.clone();
    let seventy_five =
        if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut split.income[1].income {
            data.lots.pop().unwrap()
        } else {
            unreachable!()
        };
    let mut second = qsb_income(" synthetic a inc. ", seventy_five.basis, seventy_five.proceeds, 0);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut second.income {
        data.lots = vec![seventy_five];
    }
    add(&mut split, second);
    let split_result = supported(&split);
    for id in [
        "qsbs_2_excluded_gain",
        "qsbs_2_taxable_gain",
        "qsbs_2_amt_preference",
        "rate_28_gain",
        "capital_gain",
    ] {
        assert_eq!(value(&split_result, id), value(&all_three, id), "{id}");
    }

    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
        data.lots[0].zone_gain_through_2018 =
            Some(crate::QsbsZoneValuation::Established { gain: 40_000_000 });
    }
    let zone_mixed = supported(&d);
    assert_eq!(value(&zone_mixed, "qsbs_2_excluded_gain"), 2_290_000);
    assert_eq!(value(&zone_mixed, "qsbs_2_taxable_gain"), 710_000);
    assert_eq!(value(&zone_mixed, "qsbs_2_amt_preference"), 90_300);

    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.zone_60_exclusion = Some(crate::QsbsZoneCertification::No);
        data.lots[0].zone_gain_through_2018 = None;
        data.lots[1].proceeds = 1_310_000_000;
    }
    unsupported(&d, "Section 1202 mixed acquisition cohorts");
}

#[test]
fn empowerment_zone_qsbs_excludes_extra_ten_percent_of_pre_2019_gain() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = qsb_income("Synthetic Zone Inc.", 10_000_000, 110_000_000, 0);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut entry.income {
        data.lots[0].issued = 20050101;
        data.lots[0].zone_gain_through_2018 =
            Some(crate::QsbsZoneValuation::Established { gain: 60_000_000 });
        data.zone_60_exclusion = Some(crate::QsbsZoneCertification::Yes);
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(value(&r, "qsbs_2_zone_gain_through_2018"), 600_000);
    assert_eq!(value(&r, "qsbs_2_excluded_gain"), 560_000);
    assert_eq!(value(&r, "qsbs_2_taxable_gain"), 440_000);
    assert_eq!(value(&r, "rate_28_gain"), 440_000);
    assert_eq!(value(&r, "qsbs_2_amt_preference"), 39_200);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.lots[0].proceeds = 1_310_000_000;
        data.lots[0].zone_gain_through_2018 =
            Some(crate::QsbsZoneValuation::Established { gain: 600_000_000 });
    }
    unsupported(&d, "Section 1202 zone issuer limit");

    // The entire $13 million gain has one 60% rate, so the $10 million
    // issuer limit does not require an allocation between different rates.
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.lots[0].zone_gain_through_2018 =
            Some(crate::QsbsZoneValuation::Established { gain: 1_300_000_000 });
    }
    let capped = supported(&d);
    assert_eq!(value(&capped, "qsbs_2_exclusion_limit"), 10_000_000);
    assert_eq!(value(&capped, "qsbs_2_excluded_gain"), 6_000_000);
    assert_eq!(value(&capped, "qsbs_2_taxable_gain"), 7_000_000);
    assert_eq!(value(&capped, "qsbs_2_rate_28_gain"), 4_000_000);
    assert_eq!(value(&capped, "qsbs_2_amt_preference"), 420_000);
    assert_eq!(estimate(&restart(d.clone()).draft), capped);
}

#[test]
fn qsbs_requires_verified_qualification_and_more_than_five_years() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000_000, 110_000_000, 0));
    let excluded = supported(&d);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.active_business = Some(false);
    }
    // Unconfirmed qualification: no exclusion, the whole gain is taxed.
    let r = supported(&d);

    assert!(value(&r, "agi") > value(&excluded, "agi"));
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.active_business = Some(true);
        data.special = Some(true);
    }
    // Not original issue: the sale is left out with a notice.
    let r = supported(&d);
    assert!(noted(&r, "WARN_QSBS_SALE_LEFT_OUT"), "{r:?}");
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.special = Some(false);
    }

    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.active_business = Some(true);
        data.lots[0].issued = 20210601;
    }
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
}

#[test]
fn qsbs_exclusion_reduces_current_year_nol_as_form_172_requires() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 10_000_000, 0));
    add(&mut d, income(cash_business(-300_000)));
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000_000, 110_000_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "current_net_operating_loss"), 0);
    assert!(value(&before, "current_net_operating_loss") > 0);
}

#[test]
fn form_172_keeps_qsbs_exclusion_when_other_sales_create_a_capital_loss() {
    use crate::business_rules::current_nol;
    // $1m fully excluded QSBS gain, $1.5m other nonbusiness capital loss,
    // and a $2m ordinary business loss. Schedule D deducts only $3,000.
    // Form 172 lines 3, 17 and 16–22 still add back the $1m exclusion.
    assert_eq!(
        current_nol(-2_003_000, nol_capital(-3_000, -500_000, 0, 1_000_000, -1_500_000), 0, 0,),
        1_000_000
    );
    // Personal deductions use the full pre-exclusion capital-gain room.
    assert_eq!(
        current_nol(-2_500_000, nol_capital(0, 1_000_000, 0, 1_000_000, 0), 500_000, 0,),
        1_500_000
    );
}

#[test]
fn qsbs_with_other_investment_loss_keeps_its_form_172_addback_on_the_return() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 10_000_000, 0));
    add(&mut d, income(cash_business(-300_000)));
    let mut loss = personal_sale(0, 150_000, GainTerm::Long);
    if let FederalIncome::Sale { data } = &mut loss {
        data.form.use_kind = SaleUse::Investment { wash_sale: 0 };
    }
    add(&mut d, income(loss));
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic A Inc.", 1_000_000, 11_000_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "capital_gain"), -3_000);
    assert_eq!(
        value(&before, "current_net_operating_loss") - value(&after, "current_net_operating_loss"),
        100_000
    );
}

#[test]
fn itemized_personal_casualty_creates_current_nol_under_section_172() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    let ded = d.deductions.as_mut().unwrap();
    ded.personal_casualties.push(personal_disaster("Synthetic federal storm", 4_000_000));
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let federal = supported(&d);
    assert_eq!(value(&federal, "personal_casualty_deduction"), 37_900);
    assert_eq!(value(&federal, "current_net_operating_loss"), 17_900);
    assert_eq!(value(&federal, "amt_current_net_operating_loss"), 17_900);
    assert_eq!(estimate(&restart(d.clone()).draft), federal);

    d.deductions.as_mut().unwrap().personal_casualties[0].declaration =
        CasualtyDeclaration::State { reference: "Synthetic state declaration".into() };
    let state = supported(&d);
    // Section 172(d)(4)(C) covers section 165(c)(3) casualties; the IRS's
    // older federal-only wording predates the 2026 state-disaster expansion.
    assert_eq!(value(&state, "current_net_operating_loss"), 17_900);
    assert_eq!(value(&state, "amt_current_net_operating_loss"), 17_900);

    d.deductions
        .as_mut()
        .unwrap()
        .personal_casualties
        .push(personal_disaster("Another federal storm", 1_000_000));
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "personal_casualty_deduction"), 47_800);
    assert_eq!(value(&mixed, "current_net_operating_loss"), 27_800);
    d.deductions.as_mut().unwrap().personal_casualties.pop();
}

#[test]
fn section_68_reduces_itemized_casualty_source_for_current_nol() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 70_000_000, 0);
    let ded = d.deductions.as_mut().unwrap();
    ded.personal_casualties.push(personal_disaster("Synthetic federal storm", 100_000_000));
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let r = supported(&d);
    assert_eq!(value(&r, "personal_casualty_deduction"), 929_900);
    // Section 68(a)(2) measures taxable income without flooring it at zero:
    // 700,000 - 929,900 + 929,900 = 700,000, over the 37% start of 640,600, so
    // 2/37 x 59,400 = 3,211 (the old floor at zero gave 2/37 x 289,300 = 15,638).
    assert_eq!(value(&r, "itemized_deduction_reduction"), 3_211);
    assert_eq!(value(&r, "current_net_operating_loss"), 226_689);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 229_900);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn retained_card_issuer_active_income_does_not_add_investment_income() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 4_000_000);
    add(
        &mut draft,
        income(FederalIncome::Interest { data: interest_source(native_interest(10_000, 0)) }),
    );
    let mut without_credit = draft.clone();
    let mut baseline_business = retained_card_issuer_business();
    baseline_business.fuel_tax_credit = None;
    add(&mut without_credit, income(FederalIncome::Business { data: baseline_business }));
    add(&mut draft, income(FederalIncome::Business { data: retained_card_issuer_business() }));
    let baseline = supported(&without_credit);
    let report = supported(&draft);
    assert_eq!(value(&baseline, "net_investment_income_tax"), 4);
    assert_eq!(
        value(&report, "net_investment_income_tax"),
        value(&baseline, "net_investment_income_tax")
    );
    assert!(value(&report, "agi") > value(&baseline, "agi"));
}

#[test]
fn form_8912_requires_direct_pre_2018_source_and_unreported_interest() {
    let good = direct_tax_credit_bond(
        "GUARD12345",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        100_000,
    );
    for case in 0..9 {
        let mut draft = complete(FederalStatus::Single);
        let mut bond = good.clone();
        match case {
            0 => bond.answers.issued_on = Some(20180101),
            1 => bond.form.bond_type = None,
            2 => bond.direct_scope_mut().unwrap().directly_held_full_year = false,
            3 => bond.direct_scope_mut().unwrap().direct_issuer_report = false,
            4 => bond.direct_scope_mut().unwrap().no_direct_pay_election = false,
            5 => bond.answers.prior_carry_dollars = 1_000,
            6 => bond.direct_scope_mut().unwrap().deemed_interest_not_elsewhere = false,
            7 => bond.direct_scope_mut().unwrap().separate_interest_entered = false,
            8 => bond.direct_scope_mut().unwrap().ordinary_investment_interest = false,
            _ => unreachable!(),
        }
        if case == 5 {
            bond.answers.prior_carry_confirmed = Some(false);
        }
        draft.credits.as_mut().unwrap().tax_credit_bonds = vec![bond];
        match case {
            // An issue date after 2017 is not a confirmation: still refused.
            0 => unsupported(&draft, "Tax credit bond"),
            // An undocumented prior carry is simply not claimed.
            5 => {
                let r = supported(&draft);
                assert!(value(&r, "bond_credit_deemed_interest") > 0);
                assert!(!noted(&r, "WARN_TAX_CREDIT_BOND_LEFT_OUT"));
            }
            // Missing actual required-income facts do not establish ordinary tax.
            6 | 8 => unsupported(&draft, "Tax-credit bond income"),
            // Other scope limits omit only credit; the known deemed interest remains.
            _ => {
                let r = supported(&draft);
                assert!(noted(&r, "WARN_TAX_CREDIT_BOND_LEFT_OUT"), "{case}");
                assert_eq!(value(&r, "bond_credit_deemed_interest"), 1000);
                assert_eq!(value(&r, "tax_credit_bond_credit"), 0);
            }
        }
    }
    let mut draft = complete(FederalStatus::Single);
    let mut same = good.clone();
    same.form.box1_credit = Some(200_000);
    same.form.issuer.federal_id = "123456789".into();
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![good, same];
    assert!(!draft.valid_stored());

    let mut other_issuer = draft.credits.as_ref().unwrap().tax_credit_bonds[1].clone();
    other_issuer.form.issuer.name = "Another synthetic issuer".into();
    other_issuer.form.issuer.federal_id = "98-7654321".into();
    draft.credits.as_mut().unwrap().tax_credit_bonds[1] = other_issuer;
    assert!(draft.valid_stored());
    let report = supported(&draft);
    assert_eq!(value(&report, "form_8912_line1"), 3000);
    assert_eq!(value(&report, "tax_credit_bond_credit"), 3000);
}

#[test]
fn part_iii_lines_19i_and_19j_are_emitted() {
    // Form 4562 Part III on the 2026 draft: residential rental property
    // placed in service in 2026 is line 19i, nonresidential (commercial)
    // rental property line 19j; each property's total is line 22.
    let mut d = complete(FederalStatus::Single);
    let mut residential = rental(30_000, 5_000);
    residential.buildings[0].in_service = 20260115;
    add(&mut d, income(FederalIncome::Rental { data: residential }));
    let mut commercial = rental(40_000, 6_000);
    commercial.property_type = RentalPropertyType::Commercial;
    commercial.buildings[0].in_service = 20260115;
    add(&mut d, income(FederalIncome::Rental { data: commercial }));
    let r = supported(&d);
    let tagged = |id: &str, line: &str| {
        r.lines
            .iter()
            .any(|l| l.id == id && l.form_line == Some(FormLine::new(TaxForm::Form4562, line)))
    };
    assert!(tagged("rental_2_building_0_depreciation", "19i"), "{r:?}");
    assert!(tagged("rental_3_building_0_depreciation", "19j"), "{r:?}");
    assert!(tagged("rental_2_depreciation", "22"), "{r:?}");
    assert!(tagged("rental_3_depreciation", "22"), "{r:?}");
}

#[test]
fn schedule_b_threshold_at_exactly_1500() {
    // Schedule B is filed only "over $1,500" of interest or of dividends
    // (SCHEDULE_B_THRESHOLD in federal_rules.rs). The gate compares
    // `dollars(t.interest)`/`dollars(t.dividends)`, the same
    // round-half-up-to-the-nearest-dollar conversion the rest of the return
    // uses, not the raw cents total. Exactly $1,500.00 rounds to $1,500 and
    // stays at the threshold (not shown); the smallest cents amount that
    // rounds up to $1,501 is $1,500.50, not $1,500.01 (which still rounds
    // down to $1,500).
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest {
                private_loan: false,
                foreign_tax: 0,
                us_obligations: 0,
                taxable: 150_000, // $1,500.00
                early_penalty: 0,
                exempt: 0,
                private_activity: 0,
                withholding: 0,
                special: InterestSpecial::NoneOfThese,
                bond_premium: 0,
                treasury_bond_premium: 0,
                tax_exempt_bond_premium: 0,
                accrued_interest: 0,
                after_death: None,

                investment_expenses: 0,
                market_discount: 0,
                foreign_country: alloc::string::String::new(),

                nominee: None,
                adjustments: None,

                seller_mortgage: None,
                ..Default::default()
            }),
        }),
    );
    assert!(!form_coverage(&supported(&d)).contains_key(&TaxForm::ScheduleB));

    if let FederalIncome::Interest { data } = &mut d.income[1].income {
        let mut data = edit_interest(data);
        data.taxable = 150_050; // $1,500.50: the smallest amount over the threshold.
    } else {
        panic!("expected the interest entry at index 1: {:?}", d.income);
    }
    let r = supported(&d);
    assert!(form_coverage(&r).get(&TaxForm::ScheduleB).is_some_and(|lines| lines.contains("4")));
    assert_eq!(value(&r, "schedule_b_taxable_interest"), 1501);

    // Ordinary dividends follow the identical whole-dollar gate.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section_1202_gain: 0,
                section_1202_exclusion: None,
                foreign_tax: 0,
                ordinary: 150_000, // $1,500.00
                qualified: 0,
                section199a: ReitDividends::None,
                capital_distributions: 0,
                collectible_distributions: 0,
                unrecaptured_distributions: 0,
                withholding: 0,
                special: crate::DividendSpecial::NoneOfThese,
                exempt_dividends: 0,
                private_activity_dividends: 0,
                after_death: None,

                section_897_ordinary: 0,
                section_897_capital: 0,
                foreign_country: alloc::string::String::new(),

                nominee: None,

                nondividend_distributions: 0,
                cash_liquidation: 0,
                noncash_liquidation: 0,
                capital_returns: Vec::new(),
            }),
        }),
    );
    assert!(!form_coverage(&supported(&d)).contains_key(&TaxForm::ScheduleB));

    if let FederalIncome::Dividend { data } = &mut d.income[1].income {
        let mut data = edit_dividend(data);
        data.ordinary = 150_050; // $1,500.50: the smallest amount over the threshold.
    } else {
        panic!("expected the dividend entry at index 1: {:?}", d.income);
    }
    let r = supported(&d);
    assert!(form_coverage(&r).get(&TaxForm::ScheduleB).is_some_and(|lines| lines.contains("6")));
    assert_eq!(value(&r, "schedule_b_ordinary_dividends"), 1501);
}

#[test]
fn interest_and_dividend_subset_boxes_are_validated() {
    // 1099-INT box 9 (specified private-activity bond interest) cannot
    // exceed box 8 (total tax-exempt interest): federal_rules.rs
    // validate_income_amount, `v.private_activity <= v.exempt`.
    let mut over_private_activity = complete(FederalStatus::Single);
    add(
        &mut over_private_activity,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(50_000, 50_001)),
        }),
    );
    assert!(!over_private_activity.valid_stored());

    // 1099-DIV box 1b (qualified dividends) cannot exceed box 1a (ordinary
    // dividends).
    let mut over_qualified = complete(FederalStatus::Single);
    add(
        &mut over_qualified,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(100_000, 100_001, 0)
            }),
        }),
    );
    assert!(!over_qualified.valid_stored());

    // 1099-DIV box 2b (unrecaptured section 1250 gain) plus box 2d
    // (collectibles gain) cannot exceed box 2a (total capital gain
    // distributions).
    let mut over_subset = complete(FederalStatus::Single);
    add(
        &mut over_subset,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                collectible_distributions: 60_000,
                unrecaptured_distributions: 40_001,
                ..native_dividend(0, 0, 100_000)
            }),
        }),
    );
    assert!(!over_subset.valid_stored());
}

#[test]
fn community_property_return_divides_premium_accrued_and_exempt_dividend_fields() {
    // A statement's bond premium, accrued interest and exempt-interest dividends
    // are part of its community item: each is netted on the statement and the
    // result is divided like any other community income (Form 8958).
    let mut base =
        community_joint_portfolio_draft(6_000_000, 10_000_000, 200_000, 400_000, 200_000);
    // The premium scenarios below expressly elect taxable-bond amortization.
    base.screening.as_mut().unwrap().taxable_premium_election = Some(true);
    assert_eq!(return_line(&supported(&base), "2b"), 100_000);
    for (mutate, expected) in [
        ((|v: &mut FederalInterest| v.bond_premium = 20_000) as fn(&mut FederalInterest), 90_000),
        (|v: &mut FederalInterest| v.treasury_bond_premium = 20_000, 90_000),
        (|v: &mut FederalInterest| v.accrued_interest = 20_000, 90_000),
    ] {
        let mut d = base.clone();
        let FederalIncome::Interest { data } = &mut d.income[2].income else { unreachable!() };
        let mut data = edit_interest(data);
        mutate(&mut data);
        drop(data);
        assert_eq!(return_line(&supported(&d), "2b"), expected);
    }
    let mut d = base.clone();
    let FederalIncome::Dividend { data } = &mut d.income[3].income else { unreachable!() };
    let mut data = edit_dividend(data);
    data.exempt_dividends = 200_000;
    drop(data);
    assert_eq!(return_line(&supported(&d), "2a"), 100_000);
}

#[test]
fn schedule_b_part_iii_answers_produce_review_notes() {
    // Schedule B Part III screening answers (foreign accounts and trusts):
    // Yes on either adds a review note "FinCEN 114 / Form 3520 may be required" and leaves the estimate supported.
    // No adds nothing.
    // NotSure refuses naming the line.
    let base = complete(FederalStatus::Single);
    assert_eq!(supported(&base).state, ResultState::EstimatedForSupportedScope);
    assert!(!supported(&base).review_notes.iter().any(|n| n.contains("FinCEN 114")));

    // Yes on 7a (foreign financial account) with country name adds the review note
    // and leaves the return in estimated state.
    let mut d = base.clone();
    d.screening.as_mut().unwrap().foreign_account = true;
    d.screening.as_mut().unwrap().fbar_required = Some(true);
    d.screening.as_mut().unwrap().fbar_countries = vec![0x4652];
    let r = supported(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("FinCEN 114")));

    // Yes on 8 (foreign trust) adds the review note and leaves the return in estimated state.
    let mut d = base.clone();
    d.screening.as_mut().unwrap().foreign_trust = true;
    let r = supported(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("FinCEN 114")));

    // Yes on both 7a and 8 adds the review note once.
    let mut d = base.clone();
    d.screening.as_mut().unwrap().foreign_account = true;
    d.screening.as_mut().unwrap().fbar_required = Some(true);
    d.screening.as_mut().unwrap().fbar_countries = vec![0x474d];
    d.screening.as_mut().unwrap().foreign_trust = true;
    let r = supported(&d);
    let note_count = r.review_notes.iter().filter(|n| n.contains("FinCEN 114")).count();
    assert_eq!(note_count, 1);

    // No on both 7a and 8 adds no review note.
    let mut d = base.clone();
    d.screening.as_mut().unwrap().foreign_account = false;
    d.screening.as_mut().unwrap().foreign_trust = false;
    let r = supported(&d);
    assert!(!r.review_notes.iter().any(|n| n.contains("FinCEN 114")));
}

#[test]
fn schedule_b_parts_are_required_independently() {
    // Schedule B's over-$1,500 filing requirement (SCHEDULE_B_THRESHOLD in
    // federal_rules.rs) applies to Part I (interest) and Part II (dividends)
    // separately: crossing the threshold on one side does not add the
    // other side's line, and staying under it on one side does not remove
    // the other side's line.
    //
    // Interest $1,600.00 (over $1,500) with dividends $200.00 (under
    // $1,500): only Schedule B line 4 (taxable interest) is emitted.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(160_000, 0)) }),
    );
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(20_000, 0, 0)
            }),
        }),
    );
    let coverage = form_coverage(&supported(&d));
    let schedule_b = coverage
        .get(&TaxForm::ScheduleB)
        .expect("Schedule B line 4 is required by the interest side alone");
    assert!(schedule_b.contains("4"));
    assert!(!schedule_b.contains("6"));

    // The reverse: dividends $1,600.00 (over $1,500) with interest $200.00
    // (under $1,500) emits only Schedule B line 6 (ordinary dividends).
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(20_000, 0)) }),
    );
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(160_000, 0, 0)
            }),
        }),
    );
    let coverage = form_coverage(&supported(&d));
    let schedule_b = coverage
        .get(&TaxForm::ScheduleB)
        .expect("Schedule B line 6 is required by the dividend side alone");
    assert!(schedule_b.contains("6"));
    assert!(!schedule_b.contains("4"));
}

#[test]
fn schedule_b_bond_premium_floor_is_per_payer() {
    // Payer A: box1 $300, box11 bond premium $800. The premium exceeds this
    // payer's own interest, so it floors at zero rather than borrowing the
    // $500 shortfall from payer B: positive(300 - 800) = 0.
    // Payer B (clean, no premium): box1 $2,000.
    //
    // Correct per-payer total: 0 + 2,000 = 2,000, which is over the $1,500
    // Schedule B threshold. An incorrect combined-total floor would instead
    // sum both payers' box 1 amounts first (300 + 2,000 = 2,300) and
    // subtract the combined premium once (positive(2,300 - 800) = 1,500),
    // which is not over $1,500 and would wrongly leave Schedule B off the
    // return entirely. Asserting the $2,000 total (and that Schedule B is
    // shown) distinguishes the required per-payer floor from that
    // incorrect combined one.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest {
                taxable: 30_000,
                bond_premium: 80_000,
                ..Default::default()
            }),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(200_000, 0)) }),
    );
    d.screening.as_mut().unwrap().taxable_premium_election = Some(true);
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 200_000);
    assert_eq!(value(&r, "schedule_b_taxable_interest"), 2000);
    let schedule_b = form_coverage(&r)
        .get(&TaxForm::ScheduleB)
        .cloned()
        .expect("the per-payer $2,000 total is over the $1,500 threshold");
    assert!(schedule_b.contains("4"));
}

#[test]
fn tax_exempt_interest_stays_off_schedule_b() {
    // $5,000 of tax-exempt interest (1099-INT box 8) and no taxable
    // interest: tax-exempt interest is never a Schedule B Part I amount
    // (lines 1, 2 or 4 all describe only taxable interest), so with no
    // taxable interest and no dividends, Schedule B is not shown at all,
    // even though $5,000 is well over the $1,500 threshold that would
    // apply to taxable interest. The $5,000 still reaches Form 1040 line
    // 2a directly.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest {
                exempt: 500_000,
                special: InterestSpecial::NoneOfThese,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert!(!form_coverage(&r).contains_key(&TaxForm::ScheduleB));
    assert_eq!(return_line(&r, "2a"), 500_000);
    assert_eq!(return_line(&r, "2b"), 0);
    // complete(FederalStatus::Single)'s only other income is the $60,000
    // W-2; tax-exempt interest is excluded from total income, so AGI is
    // unaffected by the $5,000.
    assert_eq!(value(&r, "agi"), 60000);

    // The same payer also has $2,000 of taxable interest (over the $1,500
    // threshold): Schedule B line 4 now shows only the $2,000 taxable
    // amount, never the $5,000 tax-exempt amount, while line 2a keeps
    // showing the $5,000 tax-exempt amount unchanged.
    if let FederalIncome::Interest { data } = &mut d.income[1].income {
        let mut data = edit_interest(data);
        data.taxable = 200_000;
    } else {
        panic!("expected the interest entry at index 1: {:?}", d.income);
    }
    let r = supported(&d);
    let schedule_b = form_coverage(&r)
        .get(&TaxForm::ScheduleB)
        .cloned()
        .expect("the $2,000 taxable amount is over the $1,500 threshold");
    assert!(schedule_b.contains("4"));
    assert!(!schedule_b.contains("6"));
    assert_eq!(value(&r, "schedule_b_taxable_interest"), 2000);
    assert_eq!(return_line(&r, "2a"), 500_000);
    assert_eq!(return_line(&r, "2b"), 200_000);
}

// Matrix row 11 (Schedule D): a typical single filer with a capital-gains
// return, hand-computed from the fetched 2026 draft Schedule D
// (irs.gov/pub/irs-dft/f1040sd--dft.pdf) and its instructions
// (irs.gov/pub/irs-dft/i1040sd--dft.pdf, labeled "(2025)" but structurally
// the same worksheets, per the schedule-d.md map's finding that this is the
// only fetched i1040sd), the Qualified Dividends and Capital Gain Tax
// Worksheet in i1040gi, and the 2026 single 0%/15% breakpoint recorded from
// Rev. Proc. 2025-32 as the App's own constant (income_tax.rs
// Rates::for_year(2026, Single)). No expected number below is copied from
// the App. The card names only the loss amounts for the short- and
// long-term losses, not their proceeds/basis; proceeds and basis below are
// chosen to produce exactly those stated losses.
#[test]
fn capital_gains_return_matches_hand_computation() {
    // W-2 (box 1 $95,000, box 2 $14,000; boxes 3-6 consistent via the
    // wage() fixture helper, both under the $184,500 social security wage
    // base): line 1a/1z wages $95,000.
    //
    // Four sales, basis reported to the IRS, no adjustments:
    //   Short-term gain: proceeds $14,000, basis $10,000 -> +$4,000.
    //   Short-term loss: proceeds $1,000, basis $2,500 -> -$1,500.
    //   Long-term gain: proceeds $30,000, basis $18,000 -> +$12,000.
    //   Long-term loss: proceeds $3,000, basis $5,000 -> -$2,000.
    // 1099-DIV: box 1a $1,000 ordinary dividends (line 3b), box 1b $800
    //   qualified dividends (line 3a, a subset of 1a), box 2a $600 capital
    //   gain distributions (Schedule D line 13).
    //
    // Schedule D:
    //   Line 1b (short-term, basis reported): $4,000 - $1,500 = $2,500.
    //   Line 7: $2,500 (no carryover, no lines 4/5/6 items).
    //   Line 8b (long-term, basis reported): $12,000 - $2,000 = $10,000.
    //   Line 13: $600 (1099-DIV box 2a).
    //   Line 15: $10,000 + $600 = $10,600.
    //   Line 16: $2,500 + $10,600 = $13,100 (a gain; the annual loss limit
    //     doesn't apply here) -> Form 1040 line 7a.
    //   Line 17: both line 15 and line 16 are gains -> yes.
    //   Line 18 (28% Rate Gain Worksheet): $0 (no collectibles/section 1202).
    //   Line 19 (Unrecaptured Section 1250 Gain Worksheet): $0 (no section
    //     1250 property).
    //   Line 20: lines 18 and 19 are both zero, so the Qualified Dividends
    //     and Capital Gain Tax Worksheet applies (not the fuller Schedule D
    //     Tax Worksheet).
    //   Line 21: not applicable; line 16 is a gain, not a loss.
    //   Line 22: there are qualified dividends, so a worksheet (not a plain
    //     Form 1040 line 16 table lookup) figures the tax.
    //
    // Form 1040:
    //   Line 1a/1z: $95,000. Line 3a: $800. Line 3b: $1,000. Line 7a:
    //     $13,100.
    //   Line 9 (total income): 95,000 + 1,000 + 13,100 = $109,100.
    //   Line 10 (adjustments): $0. Line 11 (AGI): $109,100.
    //   Line 12e (2026 single standard deduction, not 65/blind): $16,100.
    //   Line 13b (QBI deduction): $0 (no QBI income). Line 14: $16,100.
    //   Line 15 (taxable income): 109,100 - 16,100 = $93,000.
    //
    //   Line 16, Qualified Dividends and Capital Gain Tax Worksheet
    //   (i1040gi):
    //     1. Line 15 taxable income: $93,000.
    //     2. Line 3a qualified dividends: $800.
    //     3. Smaller of Schedule D line 15 ($10,600) or line 16 ($13,100):
    //        $10,600.
    //     4. Line 2 + line 3: $800 + $10,600 = $11,400.
    //     5. Line 1 - line 4: $93,000 - $11,400 = $81,600.
    //     6. 2026 single 0%/15% breakpoint: $49,450.
    //     7. Smaller of line 1/6: $49,450.
    //     8. Smaller of line 5 ($81,600)/line 7 ($49,450): $49,450.
    //     9. Line 7 - line 8: $0 (taxed at 0%).
    //    10. Smaller of line 1/4: $11,400.
    //    11. Line 9: $0.
    //    12. Line 10 - line 11: $11,400.
    //    13. 2026 single 15%/20% breakpoint: $545,500.
    //    14. Smaller of line 1/13: $93,000.
    //    15. Line 5 + line 9: $81,600.
    //    16. Line 14 - line 15: $93,000 - $81,600 = $11,400.
    //    17. Smaller of line 12/16: $11,400.
    //    18. Line 17 x 15%: $1,710.
    //    19. Line 9 + line 17: $11,400.
    //    20. Line 10 - line 19: $0.
    //    21. Line 20 x 20%: $0.
    //    22. Tax on line 5 ($81,600): the 2026 draft Tax Table's $50 bracket
    //        $81,600-$81,650, midpoint $81,625, using the 2026 single rate
    //        schedule: 12,400x10% + (50,400-12,400)x12% +
    //        (81,625-50,400)x22% = 1,240 + 4,560 + 6,869.50 = 12,669.50,
    //        rounds to $12,670.
    //    23. Lines 18+21+22: 1,710 + 0 + 12,670 = $14,380.
    //    24. Tax on line 1 ($93,000): bracket $93,000-$93,050, midpoint
    //        $93,025: 1,240 + 4,560 + (93,025-50,400)x22% = 1,240 + 4,560 +
    //        9,377.50 = 15,177.50, rounds to $15,178.
    //    25. Smaller of line 23/24: $14,380 -> Form 1040 line 16.
    //
    //   Line 17 (Schedule 2 Part I, AMT and repayments): $0. Form 6251:
    //     AMTI = taxable income $93,000 + standard deduction addback
    //     $16,100 = $109,100 (no other preferences); 2026 single AMT
    //     exemption $90,100, phaseout starting at $500,000 (Rev. Proc.
    //     2025-32, App constant income_tax.rs AmtRates::for_year), so the
    //     full exemption applies: $109,100 - $90,100 = $19,000 AMT base.
    //     The $11,400 preferential amount (same qualified dividends + net
    //     LTCG) stacks above the $49,450 breakpoint and is taxed at 15%
    //     ($1,710); the remaining $19,000 - $11,400 = $7,600 ordinary AMT
    //     amount is taxed at 26% ($1,976). Tentative minimum tax $1,976 +
    //     $1,710 = $3,686 is less than the $14,380 regular tax, so Form 6251
    //     line 11 AMT is $0.
    //   Line 18 (income tax before credits): $14,380 + $0 = $14,380.
    //   Lines 19-21 (credits): $0.
    //   Line 22: $14,380.
    //   Line 23 (Schedule 2 Part II, additional taxes): $0. Net investment
    //     income tax (Form 8960) doesn't apply: AGI $109,100 is under the
    //     $200,000 single MAGI threshold.
    //   Line 24a (total tax): $14,380.
    //   Line 25a (W-2 withholding): $14,000. Line 25d: $14,000.
    //   Line 33 (total payments): $14,000 (no estimated payments, no
    //     refundable credits).
    //   Line 34/37: payments ($14,000) are less than tax ($14,380), so
    //     there's no overpayment (line 34: $0) and a $380 balance due
    //     (line 37): $14,380 - $14,000 = $380.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19780615;
    d.income = vec![wage(1, Owner::Taxpayer, 9_500_000, 1_400_000)];
    d.next_id = 2;
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Short, 1_400_000, Some(1_000_000))),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Short, 100_000, Some(250_000))),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Long, 3_000_000, Some(1_800_000))),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Long, 300_000, Some(500_000))),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(native_dividend(100_000, 80_000, 60_000)),
        }),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "3a"), 80_000);
    assert_eq!(return_line(&r, "3b"), 100_000);
    assert_eq!(return_line(&r, "7a"), 1_310_000);
    assert_eq!(value(&r, "capital_gain"), 13_100);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 0);
    assert_eq!(return_line(&r, "9"), 10_910_000);
    assert_eq!(return_line(&r, "11a / 11b"), 10_910_000);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "15"), 9_300_000);
    assert_eq!(return_line(&r, "16"), 1_438_000);
    assert_eq!(value(&r, "income_tax"), 14_380);
    assert_eq!(return_line(&r, "24a"), 1_438_000);
    assert_eq!(return_line(&r, "25a"), 1_400_000);
    assert_eq!(return_line(&r, "33"), 1_400_000);
    assert_eq!(return_line(&r, "34"), 0);
    assert_eq!(return_line(&r, "37"), 38_000);
    assert_eq!(r.tax, Some(1_438_000));
    assert_eq!(r.payments, Some(1_400_000));
    assert_eq!(r.balance, Some(38_000));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (1): a combined short- and long-term loss deducts only the
// annual $3,000 limit ($1,500 married filing separately), applied to the
// short-term loss first (i1040sd Schedule D line 21 instructions and the
// Capital Loss Carryover Worksheet). No expected number below is copied
// from the App.
#[test]
fn net_capital_loss_deducts_3000_and_carries_forward() {
    // $5,000 short-term loss and $4,000 long-term loss, no prior-year
    // carryover: combined $9,000 loss (Schedule D line 16). The annual
    // limit deducts $3,000 (line 21; single filer), applied against the
    // short-term loss first: $5,000 - $3,000 = $2,000 short-term carryover
    // (2027 Schedule D line 6); the $4,000 long-term loss is untouched
    // (2027 Schedule D line 14); $6,000 total carryover.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Short, 0, Some(500_000))),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Long, 0, Some(400_000))),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_short_carryforward"), 2000);
    assert_eq!(value(&r, "capital_long_carryforward"), 4000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 6000);

    // Married filing separately: the annual limit is $1,500 instead, still
    // applied to the short-term loss first: $5,000 - $1,500 = $3,500 short
    // carryover; the $4,000 long-term loss stays untouched; $7,500 total.
    answer_separate_pages(&mut d);
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Separate);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -1500);
    assert_eq!(value(&r, "capital_short_carryforward"), 3500);
    assert_eq!(value(&r, "capital_long_carryforward"), 4000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 7500);
}

// Boundary (2): a collectibles gain is capped at a 28% rate through the
// Schedule D Tax Worksheet's 28% Rate Gain Worksheet (line 18), not the
// filer's higher ordinary marginal rate. No expected number below is
// copied from the App.
#[test]
fn collectibles_gain_uses_the_28_percent_worksheet() {
    // $221,100 in wages (well into the 2026 single 32% bracket after the
    // standard deduction) and one $8,000 long-term collectible gain
    // (proceeds $20,000, basis $12,000): AGI $229,100, less the $16,100
    // single standard deduction, taxable income $213,000. Schedule D lines
    // 15, 16 and 18 are all $8,000 (all the gain is collectible; there is
    // no other sale).
    //
    // Schedule D Tax Worksheet (i1040sd, used because line 18 > 0):
    //   1. Taxable income: $213,000.
    //   2-6. No qualified dividends, no Form 4952: all $0.
    //   7. Smaller of Schedule D line 15/16: $8,000.
    //   8-9. No Form 4952: line 9 = line 7 = $8,000.
    //  10. Line 6 + line 9: $8,000.
    //  11. Schedule D lines 18 + 19: $8,000 + $0 = $8,000.
    //  12. Smaller of line 9/11: $8,000.
    //  13. Line 10 - line 12: $0 (none of the gain is 0%/15%/20%-eligible;
    //      all of it is the 28% collectible pool).
    //  14. Line 1 - line 13: $213,000.
    //  15. 2026 single 0%/15% breakpoint: $49,450.
    //  16. Smaller of line 1/15: $49,450. 17. Smaller of line 14/16:
    //      $49,450.
    //  18. Line 1 - line 10: $213,000 - $8,000 = $205,000.
    //  19. Smaller of line 1 or $197,300 (single): $197,300.
    //  20. Smaller of line 14/19: $197,300. 21. Larger of line 18/20:
    //      $205,000.
    //  22. Line 16 - line 17: $0 (0% amount).
    //  23. Smaller of line 1/13: $0. 24. Line 22: $0. 25. Line 23-24: $0.
    //  26. 2026 single 15%/20% breakpoint: $545,500. 27. Smaller of
    //      line 1/26: $213,000. 28. Line 21+22: $205,000. 29. Line 27-28:
    //      $8,000. 30. Smaller of line 25/29: $0. 31. 15% amount: $0.
    //  32. Line 24+30: $0. 33. Line 23-32: $0. 34. 20% amount: $0.
    //  Schedule D line 19 is $0, so lines 35-40 are skipped (treated as $0)
    //  and the worksheet goes to line 41.
    //  Schedule D line 18 is $8,000 (not zero), so lines 41-43 are
    //  computed:
    //  41. Lines 21+22+30+33+39: $205,000 + $0 + $0 + $0 + $0 = $205,000.
    //  42. Line 1 - line 41: $213,000 - $205,000 = $8,000 (the 28% amount).
    //  43. Line 42 x 28%: $8,000 x 0.28 = $2,240.
    //  44. Tax on line 21 ($205,000), 2026 single rate schedule
    //      (>= $100,000, no midpoint rounding): 12,400x10% + 38,000x12% +
    //      55,300x22% + 96,075x24% + 3,225x32% = 1,240 + 4,560 + 12,166 +
    //      23,058 + 1,032 = $42,056.
    //  45. Lines 31+34+40+43+44: $0+$0+$0+$2,240+$42,056 = $44,296.
    //  46. Tax on line 1 ($213,000): 1,240+4,560+12,166+23,058 (through
    //      $201,775) + (213,000-201,775)x32% = 41,024 + 3,592 = $44,616.
    //  47. Smaller of line 45/46: $44,296 -> Form 1040 line 16. This is
    //      $320 less than treating the $8,000 as ordinary income at the
    //      32% bracket ($44,616): $8,000 x (32% - 28%) = $320.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 22_110_000, 0)];
    d.next_id = 2;
    add(&mut d, income(collectible_sale(20_000, 12_000, GainTerm::Long, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "rate_28_gain"), 8_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 0);
    assert_eq!(value(&r, "capital_gain"), 8_000);
    assert_eq!(value(&r, "taxable_income"), 213_000);
    assert_eq!(return_line(&r, "16"), 4_429_600);
}

// Boundary (3): unrecaptured section 1250 gain reaches Schedule D line 19
// and is capped at a 25% rate, not the filer's higher ordinary marginal
// rate. No expected number below is copied from the App.
#[test]
fn unrecaptured_1250_gain_is_capped_at_25_percent() {
    // Same $221,100 wages and $213,000 taxable income as the 28% boundary
    // above, but the $8,000 long-term gain is unrecaptured section 1250
    // gain (a 1099-DIV box 2a/2c capital gain distribution) instead of a
    // collectible: Schedule D line 19 is $8,000, line 18 is $0.
    //
    // Schedule D Tax Worksheet: lines 1-34 are identical to the 28%
    // boundary above (line 11 = Schedule D lines 18+19 = $0+$8,000 =
    // $8,000, the same total), so line 21 = $205,000 and lines
    // 22/30/31/33/34 are all $0.
    // Schedule D line 19 is $8,000 (not zero), so lines 35-40 are computed:
    //  35. Smaller of line 9 ($8,000) or Schedule D line 19 ($8,000):
    //      $8,000.
    //  36. Line 10 + line 21: $8,000 + $205,000 = $213,000.
    //  37. Line 1: $213,000.
    //  38. Line 36 - line 37: $0.
    //  39. Line 35 - line 38: $8,000 (the 25% amount).
    //  40. Line 39 x 25%: $8,000 x 0.25 = $2,000.
    //  Schedule D line 18 is $0, so lines 41-43 are skipped (treated as $0)
    //  and the worksheet goes to line 44.
    //  44. Tax on line 21 ($205,000): $42,056 (same as the 28% boundary).
    //  45. Lines 31+34+40+43+44: $0+$0+$2,000+$0+$42,056 = $44,056.
    //  46. Tax on line 1 ($213,000): $44,616 (same as the 28% boundary).
    //  47. Smaller of line 45/46: $44,056 -> Form 1040 line 16. This is
    //      $560 less than treating the $8,000 as ordinary income at the
    //      32% bracket: $8,000 x (32% - 25%) = $560.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 22_110_000, 0)];
    d.next_id = 2;
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                unrecaptured_distributions: 800_000,
                ..native_dividend(0, 0, 800_000)
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 8_000);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!(value(&r, "capital_gain"), 8_000);
    assert_eq!(value(&r, "taxable_income"), 213_000);
    assert_eq!(return_line(&r, "16"), 4_405_600);
}

// Boundary (4): a prior-year capital loss carryover declared in the
// screening history enters Schedule D lines 6 (short-term) and 14
// (long-term) and nets against the current year's gains. No expected
// number below is copied from the App.
#[test]
fn prior_year_carryover_nets_on_lines_6_and_14() {
    // Capital Loss Carryover Worksheet-Lines 6 and 14 (i1040sd
    // instructions, p.9), using the 2025 return's facts:
    //   1. 2025 Form 1040 line 15 taxable income: $50,000.
    //   2. 2025 Schedule D line 21 loss (positive): the smaller of the
    //      $6,000 total loss (line 16: -$4,000 - $2,000) or the $3,000
    //      single limit: $3,000.
    //   3. Line 1 + line 2: $53,000.
    //   4. Smaller of line 2/3: $3,000.
    //   5. 2025 Schedule D line 7 loss (positive): $4,000.
    //   6. 2025 Schedule D line 15 gain (0, it's a loss): $0.
    //   7. Line 4 + line 6: $3,000.
    //   8. Line 5 - line 7: $4,000 - $3,000 = $1,000 -> 2026 Schedule D
    //      line 6 short-term carryover.
    //   9. 2025 Schedule D line 15 loss (positive): $2,000.
    //  10. 2025 Schedule D line 7 gain (0, it's a loss): $0.
    //  11. Line 4 - line 5: $3,000 - $4,000 = -$1,000, floored to $0.
    //  12. Line 10 + line 11: $0.
    //  13. Line 9 - line 12: $2,000 - $0 = $2,000 -> 2026 Schedule D
    //      line 14 long-term carryover.
    //
    // 2026 Schedule D, with a $5,000 current-year short-term gain
    // (proceeds $15,000, basis $10,000) and a $10,000 current-year
    // long-term gain (proceeds $40,000, basis $30,000):
    //   Line 6: -$1,000. Line 7: $5,000 - $1,000 = $4,000.
    //   Line 14: -$2,000. Line 15: $10,000 - $2,000 = $8,000.
    //   Line 16: $4,000 + $8,000 = $12,000 (capital_gain). The current
    //   year's gain more than uses up both carryovers, so nothing carries
    //   forward again into 2027.
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![PriorCapitalReturn {
        injured_niit_owners: None,
        injured_owners: None,

        owners: PriorReturnOwners::Taxpayer,
        status: FederalStatus::Single,
        agi: 6_200_000,
        deductions: 1_200_000,
        senior_deduction: 0,
        short: -400_000,
        long: -200_000,
        deducted_loss: 300_000,
        section_1256: 0,
        amt: None,
        origins: vec![],
        ordinary_history: true,
        investment_parts: Some(InvestmentCapitalParts { short: 100_000, long: 200_000, amt: None }),
        niit_difference: Some(0),
        research_parts: None,
    }];
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Short, 1_500_000, Some(1_000_000))),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Long, 4_000_000, Some(3_000_000))),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 12_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    assert_eq!(value(&r, "capital_long_carryforward"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

// Matrix row 12 (Form 1099-B / 8949 map, i8949 column (f)/(g) codes).
// Three lots, one per adjustment code, each computed by hand from the
// instructions, not copied from the App's output:
//   Code B (basis correction): the broker's 1099-B reported a $8,000 basis,
//     but the actual/corrected basis is $9,500. `FederalSale.basis` carries
//     the corrected $9,500 (the App always nets gain from the true basis);
//     `SaleAdjustment::BasisCorrection.reported_basis` carries the broker's
//     wrong $8,000 for column (e)/(g) display, an $8,000 - $9,500 = -$1,500
//     column (g) adjustment. Gain = $12,000 - $9,500 = $2,500 (short-term).
//   Code T (holding-period correction): the broker reported this lot
//     short-term, but it was actually held more than a year. The corrected
//     `term` is `Long`; `SaleAdjustment::HoldingPeriodCorrection` records
//     the code. Gain = $5,000 - $3,000 = $2,000 (long-term, not short).
//   Code E (selling expenses not on the 1099-B): $250 of expenses the
//     broker never saw reduce the gain directly. Gain =
//     $6,000 - $4,000 - $250 = $1,750 (short-term).
// Schedule D totals: short $2,500 + $1,750 = $4,250; long $2,000; combined
// capital gain $4,250 + $2,000 = $6,250. No loss limit or carryover applies
// (all three lots are gains), so Schedule D nets them exactly as any other
// three lots would.
#[test]
fn form_8949_codes_b_t_e_adjust_gain_per_lot() {
    let code_b = FederalSale {
        adjustment: SaleAdjustment::BasisCorrection { reported_basis: 800_000 },
        ..native_sale(GainTerm::Short, 1_200_000, Some(950_000))
    };
    assert_eq!(code_b.gain(), 250_000);

    let code_t = FederalSale {
        adjustment: SaleAdjustment::HoldingPeriodCorrection,
        ..native_sale(GainTerm::Long, 500_000, Some(300_000))
    };
    assert_eq!(code_t.gain(), 200_000);
    assert_eq!(code_t.term, GainTerm::Long);

    let code_e = FederalSale {
        adjustment: SaleAdjustment::SellingExpenses { amount: 25_000 },
        ..native_sale(GainTerm::Short, 600_000, Some(400_000))
    };
    assert_eq!(code_e.gain(), 175_000);

    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Sale { data: sale_source(code_b) }));
    add(&mut d, income(FederalIncome::Sale { data: sale_source(code_t) }));
    add(&mut d, income(FederalIncome::Sale { data: sale_source(code_e) }));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 6_250);
    assert_eq!(restart(d.clone()).draft, d);
}

// A nominee sale (Form 8949 code N) reports a transaction on behalf of the
// actual owner: the row backs the gain out in column (g), so it adds nothing
// to the return (B9-5; the sale_specials fixture F7 has the arithmetic).
#[test]
fn nominee_sale_is_calculated_by_code_n() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(FederalSale {
                adjustment: SaleAdjustment::Nominee,
                ..native_sale(GainTerm::Long, 500_000, Some(300_000))
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "capital_gain_8b"), 0);
    assert_eq!(value(&r, "capital_gain_8b_adjustment"), -2_000);
}

// Matrix row 12 (Form 1099-B / 8949 map): each `FederalSale` lot's
// `BasisReporting`, combined with its holding period and whether it carries
// a Form 8949 adjustment code or a wash-sale disallowance, selects one
// Schedule D line (i8949 boxes A-F; i1040sd lines 1a/1b/2/3 short-term,
// 8a/8b/9/10 long-term). Six lots, one per box A-F; a seventh lot is also
// box A (covered, short-term) but carries a code E selling-expense
// adjustment, so it moves off the summary line 1a onto line 1b with the
// other box A lots that need Form 8949.
//
// Every gain computed by hand from the instructions, not the App's output:
//   Line 1a (box A, no adjustment): $10,000 - $7,000 = $3,000 gain.
//   Line 1b (box A, code E $200 selling expense): $5,000 - $3,000 - $200 =
//     $1,800 gain.
//   Line 2 (box B): $8,000 - $6,500 = $1,500 gain.
//   Line 3 (box C): $4,000 - $4,800 = $800 loss.
//   Line 7 (short-term total, lines 1a-3 combined): $3,000 + $1,800 +
//     $1,500 - $800 = $5,500.
//   Line 8a (box D, no adjustment): $20,000 - $12,000 = $8,000 gain.
//   Line 9 (box E): $15,000 - $9,000 = $6,000 gain.
//   Line 10 (box F): $9,000 - $10,500 = $1,500 loss.
//   Line 15 (long-term total, lines 8a-10 combined): $8,000 + $6,000 -
//     $1,500 = $12,500.
// The combined $18,000 is a gain, so no loss limit or carryover applies;
// Schedule D nets these seven lots exactly as any other seven lots would.
#[test]
fn schedule_d_lines_group_lots_by_form_8949_box() {
    fn sale(
        term: GainTerm,
        basis_reporting: BasisReporting,
        adjustment: SaleAdjustment,
        proceeds: i64,
        basis: i64,
    ) -> FederalSale {
        FederalSale { adjustment, basis_reporting, ..native_sale(term, proceeds, Some(basis)) }
    }
    let mut d = complete(FederalStatus::Single);
    for data in [
        // Box A, line 1a: covered, no adjustment.
        sale(
            GainTerm::Short,
            BasisReporting::ReportedToIrs,
            SaleAdjustment::None,
            1_000_000,
            700_000,
        ),
        // Box A, line 1b: covered, but code E moves it off the summary line.
        sale(
            GainTerm::Short,
            BasisReporting::ReportedToIrs,
            SaleAdjustment::SellingExpenses { amount: 20_000 },
            500_000,
            300_000,
        ),
        // Box B, line 2: basis not reported to the IRS.
        sale(GainTerm::Short, BasisReporting::NotReported, SaleAdjustment::None, 800_000, 650_000),
        // Box C, line 3: no Form 1099-B.
        sale(GainTerm::Short, BasisReporting::NoForm1099B, SaleAdjustment::None, 400_000, 480_000),
        // Box D, line 8a: covered, no adjustment.
        sale(
            GainTerm::Long,
            BasisReporting::ReportedToIrs,
            SaleAdjustment::None,
            2_000_000,
            1_200_000,
        ),
        // Box E, line 9: basis not reported to the IRS.
        sale(GainTerm::Long, BasisReporting::NotReported, SaleAdjustment::None, 1_500_000, 900_000),
        // Box F, line 10: no Form 1099-B.
        sale(GainTerm::Long, BasisReporting::NoForm1099B, SaleAdjustment::None, 900_000, 1_050_000),
    ] {
        add(&mut d, income(FederalIncome::Sale { data: sale_source(data) }));
    }
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_1a"), 3_000);
    assert_eq!(value(&r, "capital_gain_1b"), 1_800);
    assert_eq!(value(&r, "capital_gain_2"), 1_500);
    assert_eq!(value(&r, "capital_gain_3"), -800);
    assert_eq!(value(&r, "capital_gain_8a"), 8_000);
    assert_eq!(value(&r, "capital_gain_8b"), 0);
    assert_eq!(value(&r, "capital_gain_9"), 6_000);
    assert_eq!(value(&r, "capital_gain_10"), -1_500);
    let line7 = value(&r, "capital_gain_1a")
        + value(&r, "capital_gain_1b")
        + value(&r, "capital_gain_2")
        + value(&r, "capital_gain_3");
    let line15 = value(&r, "capital_gain_8a")
        + value(&r, "capital_gain_8b")
        + value(&r, "capital_gain_9")
        + value(&r, "capital_gain_10");
    assert_eq!(line7, 5_500);
    assert_eq!(line15, 12_500);
    assert_eq!(value(&r, "capital_gain"), line7 + line15);
    assert_eq!(value(&r, "capital_gain"), 18_000);
    assert_eq!(restart(d.clone()).draft, d);
}

// Matrix row 12 (Form 1099-B / 8949 map): a typical married-filing-jointly
// brokerage return, hand-computed from the fetched final Schedule D (Form
// 1040) 2025 (irs.gov/pub/irs-pdf/f1040sd.pdf) and its instructions
// (irs.gov/pub/irs-pdf/i1040sd.pdf — the 2026 versions are not yet
// published; box/code definitions and the Schedule D Tax Worksheet
// structure are unchanged from recent years), the 2026 rate schedule and
// capital-gain breakpoints already audited in `income_tax.rs`
// (`Rates::for_year(2026, ...)`, sourced from the IRS 2026 rate-schedule
// revenue procedure), and the 2026 standard deduction in
// `federal_rules.rs` `standard_base`. No expected number below is copied
// from the App.
//
// One W-2 (box 1 $120,000, box 2 $16,000; boxes 3-6 consistent via the
// `wage()` fixture helper, under the $184,500 social security wage base)
// and five 1099-B lots, none collectible except (e):
//   (a) covered short-term: proceeds $9,000, basis $7,500 -> +$1,500 gain.
//       Reported to the IRS (box A), no code, no wash sale: a "clean" lot
//       (`federal_rules.rs` `collect_sale`), so it is a Form 8949 box A
//       summary entry -> Schedule D line 1a.
//   (b) covered short-term, $600 disallowed wash-sale loss (code W):
//       proceeds $4,000, basis $5,000 -> $1,000 raw loss, added back by
//       $600 -> -$400 net loss. Still box A (reported to the IRS), but the
//       wash-sale adjustment means it is not "clean," so Form 8949 needs
//       its own row and Schedule D line 1b, not the line 1a summary.
//   (c) covered long-term: proceeds $25,000, basis $15,000 -> +$10,000
//       gain. Box D (reported to the IRS, long-term), clean -> line 8a.
//   (d) noncovered long-term (basis received on a 1099-B but not reported
//       to the IRS): proceeds $8,000, basis $6,500 -> +$1,500 gain. Box E
//       -> line 9.
//   (e) long-term collectible (gold coins), no Form 1099-B: proceeds
//       $5,000, basis $3,000 -> +$2,000 gain. Box F -> line 10. Per the
//       form-8949-1099-b.md map's own code table, a no-1099-B collectible
//       sale carries Form 8949 code C; the App models this lot directly
//       through `FederalSale.collectible: bool` rather than a stored code
//       letter (the map's gap note: no column (f)/(g) code is stored for
//       C).
//
// Schedule D (i1040sd 2025 final; lines match the card's line list
// exactly, since these five lots hit only 1a, 1b, 8a, 9 and 10):
//   Line 1a (box A, clean): $1,500.
//   Line 1b (box A, code W): -$400.
//   Line 7 (short-term total, lines 1a-3): $1,500 - $400 = $1,100.
//   Line 8a (box D, clean): $10,000.
//   Line 9 (box E): $1,500.
//   Line 10 (box F): $2,000.
//   Line 15 (long-term total, lines 8a-10): $10,000 + $1,500 + $2,000 =
//     $13,500.
//   Line 16: $1,100 + $13,500 = $14,600, a gain -> Form 1040 line 7a. Per
//     Schedule D Part III: "If line 16 is a gain ... go to line 17 below."
//   Line 17: are lines 15 ($13,500) and 16 ($14,600) both gains? Yes -> go
//     to line 18.
//   Line 18 (28% Rate Gain Worksheet): only lot (e) is a collectible, and
//     the short-term total (line 7) is a gain, not a loss, so nothing
//     reduces the collectible pool: $2,000.
//   Line 19 (Unrecaptured Section 1250 Gain Worksheet): $0 (no section
//     1250 property).
//   Line 20: are lines 18 and 19 both zero/blank (and no Form 4952)? No
//     (line 18 is $2,000) -> "Complete the Schedule D Tax Worksheet ...
//     Don't complete lines 21 and 22 below."
//   Line 21: not completed (Schedule D's own line 20 instruction skips it
//     whenever line 17 answered "yes"; it is also inapplicable on its own
//     terms, since it only ever holds a loss, and line 16 is a gain).
//   Line 22: likewise not completed for the same reason (line 20 already
//     routed the return to the Schedule D Tax Worksheet).
//
// Form 1040 line 7-15:
//   Line 1a/1z: $120,000. Line 7a: $14,600.
//   Line 9 (total income): 120,000 + 14,600 = $134,600.
//   Line 10 (adjustments): $0. Line 11 (AGI): $134,600.
//   Line 12e (2026 joint standard deduction, neither spouse 65/blind):
//     $32,200. Line 14: $32,200.
//   Line 15 (taxable income): 134,600 - 32,200 = $102,400.
//
// Line 16, Schedule D Tax Worksheet (i1040sd; used because line 18 > 0),
// numbered exactly as the fetched worksheet:
//    1. Taxable income: $102,400.
//    2-6. No qualified dividends, no Form 4952: all $0.
//    7. Smaller of Schedule D line 15 ($13,500)/line 16 ($14,600): $13,500.
//    8-9. No Form 4952: line 9 = line 7 = $13,500.
//   10. Line 6 + line 9: $13,500.
//   11. Schedule D lines 18 + 19: $2,000 + $0 = $2,000.
//   12. Smaller of line 9/11: $2,000.
//   13. Line 10 - line 12: $13,500 - $2,000 = $11,500.
//   14. Line 1 - line 13: $102,400 - $11,500 = $90,900.
//   15. 2026 joint 0%/15% breakpoint: $98,900.
//   16. Smaller of line 1/15: $98,900. 17. Smaller of line 14/16: $90,900.
//   18. Line 1 - line 10: $102,400 - $13,500 = $88,900.
//   19. Smaller of line 1 or the joint 24%-bracket ceiling (well above
//       $102,400 either way; the fetched (2025) worksheet prints $394,600
//       here, and the 2026 rate schedule's own ceiling is $403,550 -- both
//       exceed line 1, so the choice does not change this return): $102,400.
//   20. Smaller of line 14/19: $90,900. 21. Larger of line 18/20: $90,900.
//   22. Line 16 - line 17: $98,900 - $90,900 = $8,000 (0% amount).
//   Line 1 ($102,400) and line 16 ($98,900) differ, so continue to line 23.
//   23. Smaller of line 1/13: $11,500. 24. Line 22: $8,000. 25. Line 23-24:
//       $3,500.
//   26. 2026 joint 15%/20% breakpoint: $613,700. 27. Smaller of line
//       1/26: $102,400. 28. Line 21+22: $98,900. 29. Line 27-28: $3,500.
//   30. Smaller of line 25/29: $3,500. 31. Line 30 x 15%: $525.
//   32. Line 24+30: $11,500. Line 1 ($102,400) and line 32 ($11,500)
//       differ, so continue to line 33.
//   33. Line 23-32: $0. 34. Line 33 x 20%: $0.
//   Schedule D line 19 is $0, so lines 35-40 are skipped (treated as $0)
//   and the worksheet goes to line 41.
//   Schedule D line 18 is $2,000 (not zero), so lines 41-43 are computed:
//   41. Lines 21+22+30+33+39: $90,900+$8,000+$3,500+$0+$0 = $102,400.
//   42. Line 1 - line 41: $102,400 - $102,400 = $0 (the 28% amount: none of
//       the collectible gain is left over once the 0%/15% brackets have
//       absorbed all $13,500 of preferential income; 28% is a ceiling on
//       collectibles, not a floor).
//   43. Line 42 x 28%: $0.
//   44. Tax on line 21 ($90,900), 2026 joint rate schedule, Tax Table
//       ($90,900 < $100,000): the $50 bracket $90,900-$90,950, midpoint
//       $90,925, entirely inside the 12% bracket ($24,800-$100,800):
//       24,800x10% + (90,925-24,800)x12% = 2,480 + 7,935 = $10,415.
//   45. Lines 31+34+40+43+44: $525+$0+$0+$0+$10,415 = $10,940.
//   46. Tax on line 1 ($102,400, >= $100,000, no midpoint rounding), in
//       the 22% bracket ($100,800-$211,400): 24,800x10% +
//       (100,800-24,800)x12% + (102,400-100,800)x22% = 2,480 + 9,120 + 352
//       = $11,952.
//   47. Smaller of line 45/46: $10,940 -> Form 1040 line 16.
//
// Form 1040 line 17-37:
//   Line 17 (AMT and Schedule 2 repayments): $0. AMTI = taxable income
//     $102,400 + standard deduction addback $32,200 = $134,600, under the
//     2026 joint AMT exemption $140,200 (`income_tax.rs`
//     `AmtRates::for_year(2026, Joint)`), so tentative minimum tax is $0.
//   Line 18 (income tax before credits): $10,940 + $0 = $10,940.
//   Lines 19-21 (credits): $0 (none claimed).
//   Line 22: $10,940.
//   Line 23 (additional taxes): $0. Net investment income tax doesn't
//     apply: AGI $134,600 is under the $250,000 joint MAGI threshold; no
//     self-employment income.
//   Line 24a (total tax): $10,940.
//   Line 25a (W-2 withholding): $16,000. Line 25d: $16,000 (the sales carry
//     no 1099-B withholding here).
//   Line 33 (total payments): $16,000 (no estimated payments, no
//     refundable credits).
//   Line 34/37: payments ($16,000) exceed tax ($10,940), so there's a
//     $5,060 overpayment (line 34): 16,000 - 10,940 = $5,060, and no
//     balance due (line 37): $0.
#[test]
fn brokerage_lots_return_matches_hand_computation() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19750314;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19770920;
    d.income = vec![wage(1, Owner::Taxpayer, 12_000_000, 1_600_000)];
    d.next_id = 2;
    // (a) covered short-term, clean -> Form 8949 box A, Schedule D line 1a.
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Short, 900_000, Some(750_000))),
        }),
    );
    // (b) covered short-term, code W wash-sale disallowance -> box A but
    // not clean, Schedule D line 1b.
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(FederalSale {
                use_kind: SaleUse::Investment { wash_sale: 60_000 },
                ..native_sale(GainTerm::Short, 400_000, Some(500_000))
            }),
        }),
    );
    // (c) covered long-term, clean -> box D, line 8a.
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Long, 2_500_000, Some(1_500_000))),
        }),
    );
    // (d) noncovered long-term (basis not reported to the IRS) -> box E,
    // line 9.
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(FederalSale {
                basis_reporting: BasisReporting::NotReported,
                ..native_sale(GainTerm::Long, 800_000, Some(650_000))
            }),
        }),
    );
    // (e) long-term collectible (gold coins), no Form 1099-B -> box F, line
    // 10, and the 28% Rate Gain Worksheet.
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(FederalSale {
                collectible: true,
                basis_reporting: BasisReporting::NoForm1099B,
                ..native_sale(GainTerm::Long, 500_000, Some(300_000))
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_1a"), 1_500);
    assert_eq!(value(&r, "capital_gain_1b"), -400);
    assert_eq!(value(&r, "capital_gain_8a"), 10_000);
    assert_eq!(value(&r, "capital_gain_9"), 1_500);
    assert_eq!(value(&r, "capital_gain_10"), 2_000);
    let line7 = value(&r, "capital_gain_1a") + value(&r, "capital_gain_1b");
    let line15 =
        value(&r, "capital_gain_8a") + value(&r, "capital_gain_9") + value(&r, "capital_gain_10");
    assert_eq!(line7, 1_100);
    assert_eq!(line15, 13_500);
    assert_eq!(value(&r, "capital_gain"), line7 + line15);
    assert_eq!(value(&r, "capital_gain"), 14_600);
    assert_eq!(value(&r, "rate_28_gain"), 2_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 0);
    assert_eq!(return_line(&r, "7a"), 1_460_000);
    assert_eq!(return_line(&r, "9"), 13_460_000);
    assert_eq!(return_line(&r, "11a / 11b"), 13_460_000);
    assert_eq!(return_line(&r, "12e"), 3_220_000);
    assert_eq!(return_line(&r, "15"), 10_240_000);
    assert_eq!(return_line(&r, "16"), 1_094_000);
    assert_eq!(return_line(&r, "24a"), 1_094_000);
    assert_eq!(return_line(&r, "25a"), 1_600_000);
    assert_eq!(return_line(&r, "33"), 1_600_000);
    assert_eq!(return_line(&r, "34"), 506_000);
    assert_eq!(return_line(&r, "37"), 0);
    assert_eq!(r.tax, Some(1_094_000));
    assert_eq!(r.payments, Some(1_600_000));
    assert_eq!(r.refund, Some(506_000));
    assert_eq!(r.balance, Some(0));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (1): `validate_sale` (federal_rules.rs) refuses a disallowed
// wash-sale loss (Form 8949 code W) that exceeds the lot's own loss
// (proceeds less basis). A $1,000 loss (proceeds $4,000, basis $5,000) can
// disallow at most $1,000; that boundary amount is accepted, one cent more
// is refused. No expected number below is copied from the App.
#[test]
fn wash_sale_disallowance_cannot_exceed_the_loss() {
    fn lot(wash_sale: i64) -> FederalSale {
        FederalSale {
            use_kind: SaleUse::Investment { wash_sale },
            ..native_sale(GainTerm::Short, 400_000, Some(500_000))
        }
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Sale { data: sale_source(lot(100_000)) }));
    assert!(d.valid_stored());
    d.income.last_mut().unwrap().income = FederalIncome::Sale { data: sale_source(lot(100_001)) };
    assert!(!d.valid_stored());
}

// Boundary (2): when the wash-sale disallowance equals the lot's entire
// loss, `gain()` (`proceeds - basis + wash_sale()`) floors at exactly $0.
// It can never become positive, because `validate_sale` never allows
// `wash_sale()` to exceed the loss in the first place. No expected number
// below is copied from the App.
#[test]
fn wash_sale_add_back_never_creates_a_gain() {
    let lot = FederalSale {
        use_kind: SaleUse::Investment { wash_sale: 100_000 },
        ..native_sale(GainTerm::Short, 400_000, Some(500_000))
    };
    assert_eq!(lot.gain(), 0);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Sale { data: sale_source(lot) }));
    let r = supported(&d);
    // The wash sale keeps this lot off the line 1a summary (`collect_sale`'s
    // "clean" test requires `wash_sale() == 0`), so it lands on line 1b.
    assert_eq!(value(&r, "capital_gain_8b"), 0);
    assert_eq!(value(&r, "capital_gain"), 0);
}

// Boundary (3): Form 8949 code E (selling expenses not reflected on the
// payer statement) subtracts the expense in `gain()`'s column (g) term.
// Entering the same economic lot with the expense already netted out of
// proceeds (the field's ordinary "net of selling fees" convention, no
// code) gives the identical gain as entering the gross proceeds with a
// code E adjustment for the same expense. No expected number below is
// copied from the App.
#[test]
fn code_e_selling_expenses_match_net_proceeds() {
    // Gross proceeds $10,000, basis $6,000, $500 selling expense not
    // reflected on the 1099-B: net gain either way is
    // 10,000 - 6,000 - 500 = $3,500.
    let net_convention = native_sale(GainTerm::Short, 950_000, Some(600_000));
    let code_e = FederalSale {
        adjustment: SaleAdjustment::SellingExpenses { amount: 50_000 },
        ..native_sale(GainTerm::Short, 1_000_000, Some(600_000))
    };
    assert_eq!(net_convention.gain(), 350_000);
    assert_eq!(code_e.gain(), 350_000);
    assert_eq!(net_convention.gain(), code_e.gain());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Sale { data: sale_source(code_e) }));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 3_500);
}

// Boundary (4): `FederalSale` has no acquisition/sale date fields (matrix
// row 12 map: box 1b/1c "derived only as `FederalSale.term: GainTerm`
// (`Short`/`Long`), not a date"). The App takes an already-determined
// holding-period term flag directly rather than computing "held one year
// or less" from dates, so there is no one-year/one-day boundary to
// exercise inside the App; this test instead confirms the two term values
// it does take are recorded exactly as given, placing an otherwise
// identical lot on Schedule D's short-term total (line 7) when
// `term: GainTerm::Short` and the long-term total (line 15) when
// `term: GainTerm::Long`. No expected number below is copied from the App.
#[test]
fn holding_period_term_is_recorded_as_the_model_takes_it() {
    fn lot(term: GainTerm) -> FederalSale {
        native_sale(term, 900_000, Some(700_000))
    }
    let mut short = complete(FederalStatus::Single);
    add(&mut short, income(FederalIncome::Sale { data: sale_source(lot(GainTerm::Short)) }));
    let r = supported(&short);
    assert_eq!(value(&r, "capital_gain_1a"), 2_000);
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    assert_eq!(return_line(&r, "7a"), 200_000);

    let mut long = complete(FederalStatus::Single);
    add(&mut long, income(FederalIncome::Sale { data: sale_source(lot(GainTerm::Long)) }));
    let r = supported(&long);
    assert_eq!(value(&r, "capital_gain_1a"), 0);
    assert_eq!(value(&r, "capital_gain_8a"), 2_000);
    assert_eq!(return_line(&r, "7a"), 200_000);
}

// Vacation-home fixture shared by the personal-use tests: rent $6,000;
// mortgage interest $3,000, real estate taxes $1,200 and other operating
// expenses $2,400 for the whole unit; one building with a $110,000
// depreciable basis in service since January 2020, so a full year of
// 27.5-year straight-line depreciation is 110,000 x 24/660 = $4,000.
pub(in crate::tests) fn add_vacation_home(
    d: &mut FederalDraft,
    rent: i64,
    fair: u32,
    personal: u32,
    prior: [i64; 2],
) {
    let mut entry = vacation_home(rent, fair, personal, prior);
    if let FederalIncome::Rental { data } = &mut entry.income {
        link_home_fixture(d, data);
    }
    add(d, entry);
}
fn vacation_home(rent: i64, fair: u32, personal: u32, prior: [i64; 2]) -> FederalIncomeEntry {
    let expense = |category, amount: i64| crate::extended_model::RentalExpense {
        treatment: BusinessExpenseKind::Ordinary,
        category,
        amount: amount * 100,
    };
    let mut v = rental(rent, 0);
    v.expenses = vec![
        expense(crate::extended_model::RentalExpenseCategory::MortgageInterest, 3000),
        expense(crate::extended_model::RentalExpenseCategory::TaxesAndLicenses, 1200),
        expense(crate::extended_model::RentalExpenseCategory::Utilities, 2400),
    ];
    v.buildings[0].cost = 130_000 * 100;
    v.buildings[0].land = 20_000 * 100;
    v.personal_use = crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days: fair,
        personal_days: personal,
        overlap_days: 0,
        direct_expenses: 0,
    };
    v.home_operating_carryover = prior[0] * 100;
    //2026-10-19: both synthetic tax worksheets explicitly carry depreciation only.
    v.prior_home_carry.get_or_insert(crate::RentalHomeCarries::NONE).home_depreciation_carryover =
        crate::SourceCents::Reported { amount: prior[1] * 100 };
    v.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: prior[1] * 100 };
    income(FederalIncome::Rental { data: v })
}

pub(in crate::tests) fn cents(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("missing {id}: {r:?}")).amount
}

// Unit C17c: Schedule E lines 5-19 by printed line and property. Both
// properties are kept profitable (no loss) so the shared Form 8582 pool
// neither suspends nor reallocates any amount, isolating the per-category
// lines and each property's hand-computed line 20 total.
#[test]
fn schedule_e_expense_lines_are_reported_by_category_and_property() {
    use crate::extended_model::{RentalExpense, RentalExpenseCategory};
    let expense = |category: RentalExpenseCategory, amount: i64| RentalExpense {
        category,
        amount: amount * 100,
        treatment: BusinessExpenseKind::Ordinary,
    };
    fn property(rent: i64, expenses: Vec<RentalExpense>, cost: i64) -> FederalRental {
        FederalRental {
            interest_limit_election: None,
            location: crate::RentalLocation::UnitedStates,
            property_qbi_history: Vec::new(),
            rent: rent * 100,
            expenses,
            buildings: vec![purchased_rental_building(cost * 100, 0, 20200101)],
            ..rental(0, 0)
        }
    }
    // Property 1 (id 2): one expense on every enterable line. Sum of lines
    // 5-17 and 19: 100+200+150+300+400+250+500+3000+600+700+800+90+1200+350+120
    // = 8,760. Building: $110,000 - $0 land, in service Jan 2020, a middle
    // recovery year: 110,000 x 24/660 = $4,000 (same building as the
    // vacation-home fixture). Line 20 = 8,760 + 4,000 = 12,760.
    // Net = rent 20,000 - line 20 12,760 = $7,240 profit.
    let property_one = property(
        20_000,
        vec![
            expense(RentalExpenseCategory::Advertising, 100),
            expense(RentalExpenseCategory::AutoAndTravel, 200),
            expense(RentalExpenseCategory::CleaningAndMaintenance, 150),
            expense(RentalExpenseCategory::Commissions, 300),
            expense(RentalExpenseCategory::Insurance, 400),
            expense(RentalExpenseCategory::LegalAndProfessional, 250),
            expense(RentalExpenseCategory::ManagementFees, 500),
            expense(RentalExpenseCategory::MortgageInterest, 3000),
            expense(RentalExpenseCategory::VehicleLoanInterest, 600),
            expense(RentalExpenseCategory::OtherInterest, 700),
            expense(RentalExpenseCategory::Repairs, 800),
            expense(RentalExpenseCategory::Supplies, 90),
            expense(RentalExpenseCategory::TaxesAndLicenses, 1200),
            expense(RentalExpenseCategory::Utilities, 350),
            expense(RentalExpenseCategory::Other { label: "Pest control".into() }, 120),
        ],
        110_000,
    );

    // Property 2 (id 3): three of the fifteen lines. Sum: 500 + 700 + 300 =
    // 1,500. Building: $55,000 - $0 land, same in-service date: 55,000 x
    // 24/660 = $2,000. Line 20 = 1,500 + 2,000 = 3,500.
    // Net = rent 10,000 - line 20 3,500 = $6,500 profit.
    let property_two = property(
        10_000,
        vec![
            expense(RentalExpenseCategory::Advertising, 500),
            expense(RentalExpenseCategory::Repairs, 700),
            expense(RentalExpenseCategory::Other { label: "Snow removal".into() }, 300),
        ],
        55_000,
    );

    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Rental { data: property_one }));
    add(&mut d, income(FederalIncome::Rental { data: property_two }));
    let r = supported(&d);

    // Property 1 (id 2): every entered line, by hand.
    for (id, expected) in [
        ("rental_2_expense_line5_advertising", 100),
        ("rental_2_expense_line6_auto_and_travel", 200),
        ("rental_2_expense_line7_cleaning_and_maintenance", 150),
        ("rental_2_expense_line8_commissions", 300),
        ("rental_2_expense_line9_insurance", 400),
        ("rental_2_expense_line10_legal_and_professional", 250),
        ("rental_2_expense_line11_management_fees", 500),
        ("rental_2_expense_line12_mortgage_interest", 3000),
        ("rental_2_expense_line13a_vehicle_loan_interest", 600),
        ("rental_2_expense_line13b_other_interest", 700),
        ("rental_2_expense_line14_repairs", 800),
        ("rental_2_expense_line15_supplies", 90),
        ("rental_2_expense_line16_taxes", 1200),
        ("rental_2_expense_line17_utilities", 350),
        ("rental_2_expense_line19_other", 120),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert_eq!(value(&r, "rental_2_depreciation"), 4000);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 0);

    // Property 2 (id 3): only the three entered lines are present; the other
    // twelve enterable categories emit no line at all.
    for (id, expected) in [
        ("rental_3_expense_line5_advertising", 500),
        ("rental_3_expense_line14_repairs", 700),
        ("rental_3_expense_line19_other", 300),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    for id in [
        "rental_3_expense_line6_auto_and_travel",
        "rental_3_expense_line7_cleaning_and_maintenance",
        "rental_3_expense_line8_commissions",
        "rental_3_expense_line9_insurance",
        "rental_3_expense_line10_legal_and_professional",
        "rental_3_expense_line11_management_fees",
        "rental_3_expense_line12_mortgage_interest",
        "rental_3_expense_line13a_vehicle_loan_interest",
        "rental_3_expense_line13b_other_interest",
        "rental_3_expense_line15_supplies",
        "rental_3_expense_line16_taxes",
        "rental_3_expense_line17_utilities",
    ] {
        assert!(!r.lines.iter().any(|l| l.id == id), "{id}");
    }
    assert_eq!(value(&r, "rental_3_depreciation"), 2000);
    assert_eq!(value(&r, "rental_3_suspended_loss"), 0);

    // Each property's line 20 total (lines 5-19, including line 18
    // depreciation) reconciles with its own rent, and neither property's
    // loss allowance or suspension changed either total: 20,000 - 12,760 =
    // 7,240 and 10,000 - 3,500 = 6,500, so the combined Schedule E net is
    // 7,240 + 6,500 = $13,740.
    let line_20_one = 8_760 + 4_000;
    let line_20_two = 1_500 + 2_000;
    assert_eq!(20_000 - line_20_one, 7_240);
    assert_eq!(10_000 - line_20_two, 6_500);
    assert_eq!(value(&r, "rental_income"), 7_240 + 6_500);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn building_placed_in_service_in_october_uses_mid_month() {
    // Pub 527's mid-month convention treats an October in-service date as
    // the 15th of the month, leaving 2.5 months (5 half-months) of the
    // 27.5-year (660-half-month) GDS recovery period for 2026:
    //   $132,000 depreciable basis x 5/660 half-months = $1,000.00.
    // Rent $5,000, no other expenses: net profit $5,000 - $1,000 = $4,000
    // (below any passive-loss question, since it is not a loss).
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Rental {
            data: FederalRental {
                interest_limit_election: None,
                location: crate::RentalLocation::UnitedStates,
                property_qbi_history: Vec::new(),
                rent: 5_000 * 100,
                expenses: vec![],
                buildings: vec![purchased_rental_building(132_000 * 100, 0, 20261015)],
                ..rental(0, 0)
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "rental_2_depreciation"), 1_000);
    assert_eq!(value(&r, "rental_income"), 4_000);
}

#[test]
fn separate_return_living_apart_gets_half_allowance_and_phaseout() {
    // i8582 Worksheet 1: a Married Filing Separately return with spouses
    // living apart the entire year gets the halved special allowance
    // ($12,500) and a halved phase-out band ($50,000-$75,000 of MAGI),
    // exactly mirroring the joint/single $25,000 allowance and
    // $100,000-$150,000 band already tested in
    // `passive_rental_allowance_matches_irs_example_and_phaseout_boundaries`
    // above. Same activity as that test (`rental(9000, 30000)`, the
    // $330,000/$55,000 building placed in service 2020, a middle recovery
    // year, so $10,000 depreciation): current loss = 9,000 - 30,000 -
    // 10,000 = -$31,000, no prior history.
    //   $50,000 MAGI (at the floor): allowance = $12,500 - 50% x $0 =
    //     $12,500 allowed; $31,000 - $12,500 = $18,500 suspended.
    //   $60,000 MAGI (mid-phaseout): allowance = $12,500 - 50% x $10,000 =
    //     $7,500 allowed; $31,000 - $7,500 = $23,500 suspended.
    //   $75,000 MAGI (at the ceiling): allowance = $12,500 - 50% x $25,000 =
    //     $0 allowed; the full $31,000 is suspended.
    for (wages, allowed, suspended) in
        [(50000, 12500, 18500), (60000, 7500, 23500), (75000, 0, 31000)]
    {
        let mut d = complete(FederalStatus::Separate);
        d.setup.as_mut().unwrap().marital_state =
            MaritalState::Married { living: SpouseLiving::AllYear };
        d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
        add(&mut d, income(FederalIncome::Rental { data: rental(9_000, 30_000) }));
        let r = supported(&d);
        assert_eq!(value(&r, "rental_income"), -allowed);
        assert_eq!(value(&r, "rental_suspended_loss"), suspended);
        assert_eq!(value(&r, "agi"), wages - allowed);
    }
}

#[test]
fn separate_return_living_together_gets_no_special_allowance() {
    // i8582 Worksheet 1: a Married Filing Separately return where the
    // spouses lived together at any time during the year gets no special
    // allowance at all ($0), regardless of MAGI — unlike the halved-but-
    // still-phasing-out amount tested above for spouses living apart all
    // year. `complete()` already sets Married Filing Separately with
    // spouses living together (Pub 925's default). Wages of $10,000 are far
    // below the $50,000 floor of the halved phase-out band that would
    // otherwise apply for a living-apart return, showing the $0 here comes
    // from the living-together rule and not from an ordinary phase-out.
    // Same $31,000-loss activity as the test above (`rental(9000, 30000)`,
    // no prior history): with no allowance, the entire $31,000 loss is
    // suspended and AGI is unaffected by the rental.
    let mut d = complete(FederalStatus::Separate);
    d.income = vec![wage(1, Owner::Taxpayer, 1_000_000, 0)];
    add(&mut d, income(FederalIncome::Rental { data: rental(9_000, 30_000) }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 31_000);
    assert_eq!(value(&r, "agi"), 10_000);
}

// ---- T2-35 Form 4797: a rental property sold ----

pub(in crate::tests) fn sold_rental(prior_suspended: i64) -> FederalRental {
    let mut property = rental(12_000, 4_000);
    property.history = if prior_suspended > 0 {
        RentalHistory {
            regular: prior_suspended * 100,
            amt: prior_suspended * 100,
            ..RentalHistory::NONE
        }
    } else {
        RentalHistory::NONE
    };
    property.sale = crate::RentalSale::Sold {
        date: 20260815,
        price: 400_000 * 100,
        land_price: 80_000 * 100,
        selling_expenses: 24_000 * 100,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    property
}
pub(in crate::tests) fn rental_sale_return(properties: Vec<FederalRental>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    for property in properties {
        add(&mut d, income(FederalIncome::Rental { data: property }));
    }
    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![], amt: vec![] });
    d
}

#[test]
fn form_4797_rental_sale_matches_hand_computation() {
    // Single filer, $60,000 W-2. A single-family rental (cost $330,000, land
    // $55,000, in service January 1, 2020) sold August 15, 2026 for $400,000
    // ($80,000 for the land) with $24,000 selling expenses, $5,000 of prior
    // suspended passive losses; a second rental without active
    // participation lost $18,000 (Form 4797, Pub 527 ch. 5, Pub 925).
    let mut other = rental(6_000, 14_000);
    other.participation = crate::RentalParticipation::NotActive;
    let d = rental_sale_return(vec![sold_rental(5_000), other]);
    let r = supported(&d);
    // Depreciation, 27.5-year mid-month on $275,000: 2020 23 half-months
    // ($9,583.33), 2021-2025 $10,000 each, 2026 through mid-August 15
    // half-months ($6,250): $65,833.33.
    assert_eq!(value(&r, "rental_2_line18_depreciation"), 6_250);
    // Selling expenses by price: $24,000 x 320/400 = $19,200 building,
    // $4,800 land. Part III: line 22 $320,000 (the building's price);
    // 23 $275,000 + $19,200 = $294,200; 24 $65,833; 25 $228,367; 26 $91,633. Part I line 2 land: $80,000 - $4,800 -
    // $55,000 = $20,200. Line 8 $111,833.
    for (suffix, amount) in [
        ("building_sale_price", 320_000),
        ("building_cost", 294_200),
        ("building_depreciation", 65_833),
        ("building_adjusted_basis", 228_367),
        ("building_gain", 91_633),
        ("land", 20_200),
        ("unrecaptured_1250", 65_833),
    ] {
        assert_eq!(value(&r, &alloc::format!("rental_2_sale_{suffix}")), amount, "{suffix}");
    }
    assert_eq!(value(&r, "form4797_line8"), 111_833);
    assert_eq!(value(&r, "form4797_line10"), 111_833);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 65_833);
    // Section 469(g): the sold rental's Schedule E $12,000 - $4,000 - $6,250
    // = $1,750 less the $5,000 prior loss plus the $111,833 gain is an
    // overall gain, so its own $3,250 loss is allowed and $108,583 of
    // passive income frees the other rental's $18,000 loss in full.
    // Line 22 is Form 8582 Part VIII column (c): the whole $5,000 prior-year
    // loss, part of it used against the property's own $1,750 income (line 21),
    // so lines 21 and 22 net to the $3,250 loss.
    assert_eq!(value(&r, "rental_2_line21_income"), 1_750);
    assert_eq!(value(&r, "rental_2_line22_allowed_loss"), -5_000);
    assert_eq!(value(&r, "rental_3_line22_allowed_loss"), -18_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    // AGI: $60,000 - $21,250 + $111,833 = $150,583.
    assert_eq!(value(&r, "agi"), 150_583);
}

// The Form 8938 note of a return with a foreign account and this highest
// value (dollars); None when there is no note.
fn form_8938_note(mut d: FederalDraft, highest: Option<i64>) -> Option<alloc::string::String> {
    let screening = d.screening.as_mut().unwrap();
    screening.foreign_account = true;
    screening.fbar_required = Some(true);
    screening.fbar_countries = vec![0x4652];
    screening.foreign_asset_max = highest.map(|v| v * 100);
    estimate(&d).review_notes.into_iter().find(|n| n.contains("Form 8938")).map(|n| n.english)
}

#[test]
fn form_8938_note_uses_complete_resources_and_exact_values() {
    let message = |mut draft: FederalDraft, cents: Option<i64>| {
        let screening = draft.screening.as_mut().unwrap();
        screening.foreign_account = true;
        screening.fbar_required = Some(true);
        screening.fbar_countries = vec![0x4652];
        screening.foreign_asset_max = cents;
        estimate(&draft).review_notes.into_iter().find(|n| n.contains("Form 8938")).unwrap().display
    };
    let unknown = message(complete(FederalStatus::Single), None);
    assert_eq!(unknown.key, "NOTE_8938_US_UNKNOWN");
    assert_eq!(
        unknown.args,
        vec![MessageArg::usd_dollars(50_000), MessageArg::usd_dollars(75_000)]
    );
    let conditional = message(complete(FederalStatus::Single), Some(5_000_137));
    assert_eq!(conditional.key, "NOTE_8938_US_CONDITIONAL");
    assert_eq!(conditional.args[1], MessageArg::usd_cents(5_000_137));
    let mut abroad = empty_return(FederalStatus::Single);
    add(&mut abroad, income(foreign_pay(80_000)));
    set_claim(&mut abroad, foreign_claim(Owner::Taxpayer));
    let high = message(abroad, Some(30_000_137));
    assert_eq!(high.key, "NOTE_8938_ABROAD_HIGH");
    assert_eq!(high.args[0], MessageArg::usd_cents(30_000_137));
}

#[test]
fn form_8938_check_uses_the_united_states_thresholds() {
    // i8938: living in the United States, unmarried: more than $50,000 on
    // the last day or $75,000 at any time; joint: $100,000 / $150,000.
    let single = complete(FederalStatus::Single);
    // At $50,000 neither test can be met (the year-end value is at most
    // the highest): no note.
    assert_eq!(form_8938_note(single.clone(), Some(50_000)), None);
    // $50,001 to $75,000: required only if the December 31 value was above
    // $50,000.
    let note = form_8938_note(single.clone(), Some(50_001)).unwrap();
    assert!(note.starts_with("Form 8938 is required if"), "{note}");
    assert!(note.contains("$50000 on December 31, 2026"), "{note}");
    let note = form_8938_note(single.clone(), Some(75_000)).unwrap();
    assert!(note.starts_with("Form 8938 is required if"), "{note}");
    // Above $75,000: likely required, and not prepared by the App.
    let note = form_8938_note(single.clone(), Some(75_001)).unwrap();
    assert!(note.starts_with("Form 8938 is likely required"), "{note}");
    assert!(note.contains("$75000 any-time threshold"), "{note}");
    assert!(note.contains("does not prepare Form 8938"), "{note}");
    // Joint: $150,000 is at the any-time threshold, $150,001 above it.
    let joint = complete(FederalStatus::Joint);
    let note = form_8938_note(joint.clone(), Some(150_000)).unwrap();
    assert!(note.contains("$100000 on December 31"), "{note}");
    let note = form_8938_note(joint.clone(), Some(150_001)).unwrap();
    assert!(note.contains("filing jointly"), "{note}");
    assert!(note.starts_with("Form 8938 is likely required"), "{note}");
    assert_eq!(form_8938_note(joint, Some(100_000)), None);
    // Married filing separately uses the unmarried amounts.
    let separate = complete(FederalStatus::Separate);
    assert!(form_8938_note(separate, Some(75_001)).unwrap().contains("$75000 any-time"));
}

#[test]
fn form_8938_check_uses_the_abroad_thresholds() {
    // A Form 2555 exclusion claimed on the return meets the presence abroad
    // test: $200,000 on the last day or $300,000 at any time.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(80_000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    assert_eq!(form_8938_note(d.clone(), Some(200_000)), None);
    let note = form_8938_note(d.clone(), Some(300_000)).unwrap();
    assert!(note.starts_with("Form 8938 is required if"), "{note}");
    assert!(note.contains("living abroad"), "{note}");
    let note = form_8938_note(d.clone(), Some(300_001)).unwrap();
    assert!(note.starts_with("Form 8938 is likely required"), "{note}");
    // Pay without the exclusion does not meet it.
    let mut us = d.clone();
    us.screening.as_mut().unwrap().foreign_earned.clear();
    assert!(form_8938_note(us, Some(75_001)).unwrap().contains("living in the United States"));
}

#[test]
fn form_8938_note_needs_a_foreign_account_or_trust() {
    // No Schedule B Part III Yes: no note even with a stored value.
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().foreign_asset_max = Some(100_000_000);
    assert!(!estimate(&d).review_notes.iter().any(|n| n.contains("8938")));
    // A trust alone with no value answered: the "may be required" note.
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().foreign_trust = true;
    let note = estimate(&d).review_notes.into_iter().find(|n| n.contains("Form 8938")).unwrap();
    assert!(note.starts_with("Form 8938 may be required"), "{note}");
}

// ---- T3-6252 Form 6252: rental installment sales and earlier-year sales ----

fn installment_terms(assumed_debt: i64, payments: i64) -> crate::SalePayment {
    crate::SalePayment::Installment {
        assumed_debt,
        payments,
        interest: crate::NoteInterest::Adequate,
        special: false,
    }
}
fn rental_on_installment(payments: i64) -> FederalRental {
    let mut property = sold_rental(0);
    if let crate::RentalSale::Sold { payment, .. } = &mut property.sale {
        *payment = installment_terms(0, payments);
    }
    property
}

#[test]
fn form_6252_rental_installment_sale_matches_hand_computation() {
    // The rental of the Form 4797 fixture (building gain $91,633, land
    // $20,200) sold for $400,000 on a note, $100,000 of principal received
    // in 2026. Form 6252: line 5 $400,000; 8 $275,000 + $55,000 =
    // $330,000; 9 $65,833; 10 $264,167; 11 $24,000; 12 $0; 13 $288,167;
    // 16 $111,833; 17 $0; 18 $400,000; 21 and 22 $100,000; 24 $100,000 x
    // 111,833 / 400,000 = $27,958.25, $27,958; 26 $27,958, all of it
    // unrecaptured section 1250 gain (smaller than the $65,833).
    let r = supported(&rental_sale_return(vec![rental_on_installment(100_000 * 100)]));
    for (suffix, amount) in [
        ("price", 400_000),
        ("cost", 330_000),
        ("depreciation", 65_833),
        ("adjusted_basis", 264_167),
        ("expenses", 24_000),
        ("total", 288_167),
        ("gross_profit", 111_833),
        ("contract_price", 400_000),
        ("payments", 100_000),
        ("income", 27_958),
        ("gain", 27_958),
        ("unrecaptured_1250", 27_958),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("rental_2_sale_form6252_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    assert_eq!(value(&r, "rental_2_sale_installment_gain"), 27_958);
    assert_eq!(value(&r, "form4797_line8"), 27_958);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 27_958);
    // Section 469(g): Schedule E $1,750 plus the $27,958 recognized gain,
    // passive income. AGI $60,000 + $1,750 + $27,958 = $89,708.
    assert_eq!(value(&r, "agi"), 89_708);
}

#[test]
fn form_6252_rental_installment_scenarios_with_losses_are_calculated() {
    // Suspended passive losses ($5,000) release in the ratio of the year's gain to the
    // gross profit (section 469(g)(3)); a 2026 loss does the same; a land loss beside
    // the building gain is reported in full. None is refused any more (unit B11 4797).
    let mut property = rental_on_installment(100_000 * 100);
    property.history = RentalHistory { regular: 500_000, amt: 500_000, ..RentalHistory::NONE };
    let r = supported(&rental_sale_return(vec![property]));
    assert!(value(&r, "rental_2_sale_released_loss") > 0);
    assert!(value(&r, "rental_2_suspended_loss") > 0);
    let mut property = rental_on_installment(100_000 * 100);
    property.rent = 0;
    let r = supported(&rental_sale_return(vec![property]));
    assert!(value(&r, "rental_2_sale_released_loss") > 0);
    let mut property = rental_on_installment(50_000 * 100);
    if let crate::RentalSale::Sold { price, land_price, selling_expenses, .. } = &mut property.sale
    {
        *price = 300_000 * 100;
        *land_price = 40_000 * 100;
        *selling_expenses = 0;
    }
    let r = supported(&rental_sale_return(vec![property]));
    assert_eq!(value(&r, "rental_2_sale_land"), -15_000);
}

fn earlier_sale(
    kind: crate::PropertyCharacter,
    gross_profit: i64,
    contract_price: i64,
    earlier_payments: i64,
    payments: i64,
) -> crate::EarlierInstallment {
    crate::EarlierInstallment {
        owner: Owner::Taxpayer,
        sale: crate::InstallmentPayments {
            property: "Sold property".into(),
            sold: 20230601,
            gross_profit: gross_profit * 100,
            contract_price: contract_price * 100,
            earlier_payments: earlier_payments * 100,
            payments: payments * 100,
            kind,
            special: false,
            qbi: crate::InstallmentQbi::NotQualified,
            livestock: None,
        },
    }
}
fn earlier_return(wages: i64, sales: Vec<crate::EarlierInstallment>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    let screening = d.screening.as_mut().unwrap();
    screening.installment_payments = sales;
    screening.section1231 = Some(Section1231History { regular: vec![], amt: vec![] });
    d
}
fn building(used: crate::PropertyUse, unrecaptured: i64) -> crate::PropertyCharacter {
    crate::PropertyCharacter::BusinessProperty { used, unrecaptured: unrecaptured * 100 }
}

#[test]
fn form_6252_earlier_year_payments_route_by_character() {
    // $60,000 W-2. Land held as an investment: gross profit $60,000,
    // contract price $150,000 (40%), $25,000 received: line 24 $10,000 to
    // Schedule D line 11. Short-held investment: $3,000 / $12,000 (25%),
    // $4,000 received: $1,000 to line 4. A rental building: $100,000 /
    // $250,000 (40%), $30,000 received: $12,000 to Form 4797 line 4, with
    // $8,000 of unrecaptured section 1250 gain still unreported, all of it
    // recognized first.
    let d = earlier_return(
        60_000,
        vec![
            earlier_sale(
                crate::PropertyCharacter::LongTermCapital,
                60_000,
                150_000,
                50_000,
                25_000,
            ),
            earlier_sale(crate::PropertyCharacter::ShortTermCapital, 3_000, 12_000, 2_000, 4_000),
            earlier_sale(
                building(crate::PropertyUse::Rental, 8_000),
                100_000,
                250_000,
                50_000,
                30_000,
            ),
        ],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "installment_1_income"), 10_000);
    assert_eq!(value(&r, "installment_1_earlier_payments"), 50_000);
    assert_eq!(value(&r, "installment_2_gain"), 1_000);
    assert_eq!(value(&r, "installment_3_gain"), 12_000);
    assert_eq!(value(&r, "installment_3_unrecaptured_1250"), 8_000);
    assert_eq!(value(&r, "form4797_line8"), 12_000);
    assert_eq!(value(&r, "form4797_line10"), 12_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 8_000);
    // Schedule D lines 4 and 11 carry the earlier-year investment sales, and line 11 also
    // the Form 4797 Part I gain of the rental building ($10,000 + $12,000).
    assert_eq!(value(&r, "capital_gain_4"), 1_000);
    assert_eq!(value(&r, "capital_gain_11"), 22_000);
    assert_eq!(printed_line(&r, "capital_gain_4"), (TaxForm::ScheduleD, "4".into()));
    assert_eq!(printed_line(&r, "capital_gain_11"), (TaxForm::ScheduleD, "11".into()));
    // 1040 line 7a: $1,000 short + $10,000 + $12,000 long = $23,000; AGI
    // $83,000.
    assert_eq!(return_line(&r, "7a"), 2_300_000);
    assert_eq!(value(&r, "agi"), 83_000);
}

#[test]
fn form_6252_earlier_business_gain_is_not_investment_income_but_a_rental_gain_is() {
    // $250,000 W-2, single: MAGI $262,000 is $62,000 above the $200,000
    // threshold. A $12,000 earlier-year installment gain (40% of $30,000)
    // on property of a business you materially participated in is not
    // net investment income: no NIIT. On a rental it is: 3.8% of the
    // smaller of $12,000 and $62,000 = $456.
    let sale = |used| {
        earlier_return(250_000, vec![earlier_sale(building(used, 0), 100_000, 250_000, 0, 30_000)])
    };
    let r = supported(&sale(crate::PropertyUse::Business));
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    let r = supported(&sale(crate::PropertyUse::Rental));
    assert_eq!(value(&r, "net_investment_income_tax"), 456);
}

#[test]
fn form_6252_earlier_year_boundaries_and_refusals() {
    // Every payment received: $50,000 + $100,000 = the $150,000 contract
    // price, the whole remaining 40%: $40,000.
    let r = supported(&earlier_return(
        60_000,
        vec![earlier_sale(
            crate::PropertyCharacter::LongTermCapital,
            60_000,
            150_000,
            50_000,
            100_000,
        )],
    ));
    assert_eq!(value(&r, "installment_1_gain"), 40_000);
    // One dollar more than the contract price is invalid.
    let d = earlier_return(
        60_000,
        vec![earlier_sale(
            crate::PropertyCharacter::LongTermCapital,
            60_000,
            150_000,
            50_001,
            100_000,
        )],
    );
    unsupported(&d, "Installment payments");
    // Something else applies.
    let mut special =
        earlier_sale(crate::PropertyCharacter::LongTermCapital, 60_000, 150_000, 0, 10_000);
    special.sale.special = true;
    unsupported(&earlier_return(60_000, vec![special]), "Installment payments");
    // A rental gain with unallowed passive losses on the return.
    let mut d = earlier_return(
        60_000,
        vec![earlier_sale(building(crate::PropertyUse::Rental, 0), 100_000, 250_000, 0, 30_000)],
    );
    let mut other = rental(6_000, 14_000);
    other.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: other }));
    unsupported(&d, "Installment payments");
    // A business gain when 2021-2025 losses make part of it ordinary: the $12,000
    // gain (30,000 x 100,000 / 250,000) is netted with the lookback, which makes
    // $5,000 ordinary. That part is qualified business income when the person says
    // so (Reg. 1.199A-3(b)(2)(ii)(A)): a stand-alone activity for a business no
    // longer on the return.
    let mut d = earlier_return(
        60_000,
        vec![earlier_sale(building(crate::PropertyUse::Business, 0), 100_000, 250_000, 0, 30_000)],
    );
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 5_000, 5_000)], &[]));
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    d.screening.as_mut().unwrap().installment_payments[0].sale.qbi =
        crate::InstallmentQbi::Qualified { business: None };
    let r = supported(&d);
    assert_eq!(value(&r, "installment_1_qualified_income"), 5_000);
    assert_eq!(value(&r, "qbi_deduction"), 1_000);
}

// Form 8815 (series EE and I savings bond interest excluded for tuition,
// IRC 135), the 2026 draft form: fixtures F1-F6. Dollars below; stored
// amounts are cents.
pub(in crate::tests) fn bond_interest_entry(box_3: i64) -> FederalIncomeEntry {
    income(FederalIncome::Interest { data: interest_source(native_treasury_interest(box_3 * 100)) })
}
// A return with the given wages (dollars) in the first W-2, one 1099-INT whose
// box 3 holds the bond interest, and a dependent child (the student).
pub(in crate::tests) fn bond_return(status: FederalStatus, wages: i64, box_3: i64) -> FederalDraft {
    let mut d = complete(status);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    add(&mut d, bond_interest_entry(box_3));
    add_child(&mut d);
    d
}
pub(in crate::tests) fn bond_student(
    person: i64,
    tuition: i64,
    benefits: i64,
) -> SavingsBondStudent {
    SavingsBondStudent {
        person,
        expenses: SavingsBondExpenses::Entered { tuition: tuition * 100, benefits: benefits * 100 },
        contributions: 0,
    }
}
pub(in crate::tests) fn bond_record(
    owner: Owner,
    proceeds: i64,
    interest: i64,
    issued: u32,
    students: Vec<SavingsBondStudent>,
) -> FederalIncomeEntry {
    let mut entry = income(FederalIncome::SavingsBonds {
        data: savings_bonds_source(SavingsBonds {
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            proceeds: proceeds * 100,
            redemption_interest: interest * 100,
            earliest_issue_date: issued,
            interest_statement: 2,
            students,
        }),
    });
    entry.owner = crate::DocumentOwner::from(owner);
    entry
}
fn note_pages(d: &FederalDraft, r: &FederalResult, key: &str) -> Vec<TaxSection> {
    let notes: Vec<_> = r.review_notes.iter().filter(|n| n.display.key == key).collect();
    assert_eq!(notes.len(), 1, "{key} once: {notes:?}");
    crate::interview::issue_pages(d, notes[0].context.as_ref().unwrap(), None)
}

// F1: full exclusion. Single; wages $60,000; 1099-INT box 3 $3,000. Bonds
// cashed: proceeds $12,000 of which $3,000 interest, owner born 1980-05-01,
// earliest issue 2010-06-01 (age 30). The child's tuition is $14,000, no
// benefits, no credit entry.
//   Line 2 = 14,000; 3 = 0; 4 = 14,000; 5 = 12,000; 6 = 3,000; 7 = 1.000
//   (line 4 >= line 5); 8 = 3,000.
//   Line 9 = 60,000 + 3,000 = 63,000, below the 101,800 start; 11 = 0, 13 = 0;
//   line 14 = 3,000. Schedule B 2 = 3,000, 3 = 3,000, 4 = 0.
//   Form 1040 line 2b = 0 and AGI = 60,000 + 3,000 - 3,000 = 60,000.
#[test]
fn savings_bond_full_exclusion_flows_to_schedule_b_and_agi() {
    let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800501;
    let child = d.dependents[0].id;
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 12_000, 3_000, 20100601, vec![bond_student(child, 14_000, 0)]),
    );
    let r = supported(&d);
    for (id, amount) in [
        ("form_8815_qualified_expenses", 14_000),
        ("form_8815_educational_benefits", 0),
        ("form_8815_net_expenses", 14_000),
        ("form_8815_proceeds", 12_000),
        ("form_8815_interest", 3_000),
        ("form_8815_eligible_interest", 3_000),
        ("form_8815_modified_agi", 63_000),
        ("form_8815_phaseout_reduction", 0),
        ("savings_bond_exclusion", 3_000),
        ("schedule_b_total_interest", 3_000),
        ("schedule_b_savings_bond_exclusion", 3_000),
        ("schedule_b_taxable_interest", 0),
        ("agi", 60_000),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(return_line(&r, "2b"), 0);
    let coverage = form_coverage(&r);
    assert!(
        coverage[&TaxForm::ScheduleB]
            .is_superset(&["2", "3", "4"].into_iter().map(alloc::string::String::from).collect())
    );
    assert!(coverage[&TaxForm::Form8815].contains("14"));
    // The savings bond record is not income: the 1099-INT carries the interest.
    // Part III uses taxable line 4 after the exclusion, not gross line 2.
    assert!(r.review_notes.is_empty(), "{:?}", r.review_notes);
    // Without the record the whole $3,000 is on line 2b.
    d.income.pop();
    let without = supported(&d);
    assert_eq!(return_line(&without, "2b"), 300_000);
    assert_eq!(value(&without, "agi"), 63_000);
}

// F2: proportional exclusion beside an American opportunity credit (joint).
// Wages $120,000; 1099-INT box 3 $4,000; bonds: proceeds $10,000, interest
// $4,000, owner born 1975-01-01, earliest issue 2005-01-01. The dependent
// student has an AOTC entry: tuition $11,000, scholarship $1,000. Credit
// expenses are 11,000 - 1,000 = 10,000, of which the credit takes 4,000
// (Form 8863 line 27).
//   Line 2 = 11,000 - 4,000 = 7,000; 3 = 1,000; 4 = 6,000; 5 = 10,000;
//   6 = 4,000; 7 = 6,000 / 10,000 = 0.600; 8 = 4,000 x 0.6 = 2,400.
//   Line 9 = 120,000 + 4,000 = 124,000, below the 152,650 start; 14 = 2,400.
//   Line 2b = 4,000 - 2,400 = 1,600; AGI = 124,000 - 2,400 = 121,600, below the
//   160,000 AOTC start, so the credit is the same with or without the record.
#[test]
fn savings_bond_exclusion_is_proportional_and_leaves_the_credit_unchanged() {
    let mut d = bond_return(FederalStatus::Joint, 120_000, 4_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19750101;
    let child = d.dependents[0].id;
    let credits = d.credits.as_mut().unwrap();
    add_education(credits, {
        let mut pair = native_education(
            child,
            FederalEducationExpenses {
                tuition: 1_100_000,
                school_materials: 0,
                other_materials: 0,
                assistance: 100_000,
                refunds: 0,
                other_benefits: 0,
                scholarship_in_income: crate::ScholarshipInclusion::None,
            },
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        );
        pair.0.id = 0;
        pair.0.half_time = true;
        pair.0.graduate = false;
        pair.1.more.american_opportunity_used_up = Some(false);
        pair
    });
    let baseline = supported(&d);
    add(
        &mut d,
        bond_record(
            Owner::Taxpayer,
            10_000,
            4_000,
            20050101,
            vec![SavingsBondStudent {
                person: child,
                expenses: SavingsBondExpenses::EducationCredit,
                contributions: 0,
            }],
        ),
    );
    let r = supported(&d);
    for (id, amount) in [
        ("form_8815_qualified_expenses", 7_000),
        ("form_8815_educational_benefits", 1_000),
        ("form_8815_net_expenses", 6_000),
        ("form_8815_eligible_interest", 2_400),
        ("form_8815_modified_agi", 124_000),
        ("savings_bond_exclusion", 2_400),
        ("agi", 121_600),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(return_line(&r, "2b"), 160_000);
    for id in ["education_credit", "refundable_education_credit"] {
        assert_eq!(value(&r, id), value(&baseline, id), "{id}");
    }
    // F2b: the same tuition entered again on the record is a blocking issue.
    let FederalIncome::SavingsBonds { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.answers.as_mut().unwrap().students[0].expenses =
        FederalSavingsBondExpenses::Entered { tuition: 1_100_000, benefits: 100_000 };
    let twice = estimate(&d);
    assert!(
        twice
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_SAVINGS_BOND_STUDENT_EXPENSES_ENTERED_TWICE"),
        "{:?}",
        twice.issues
    );
    // And the credit choice for a person with no credit entry.
    d.credits.as_mut().unwrap().education.clear();
    d.credits.as_mut().unwrap().students.clear();
    let FederalIncome::SavingsBonds { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.answers.as_mut().unwrap().students[0].expenses =
        FederalSavingsBondExpenses::EducationCredit;
    let missing = estimate(&d);
    assert!(
        missing
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_SAVINGS_BOND_STUDENT_HAS_NO_EDUCATION_CREDIT_ENTRY"),
        "{:?}",
        missing.issues
    );
}

// F3: the phaseout band (single). Wages $104,000; box 3 $2,000; bonds:
// proceeds $8,000, interest $2,000, owner born 1970-03-15, earliest issue
// 1998-10-01; the taxpayer's own tuition $8,000.
//   Line 4 = 8,000 >= line 5 = 8,000, so line 7 = 1.000 and line 8 = 2,000.
//   Line 9 = 104,000 + 2,000 = 106,000; 10 = 101,800; 11 = 4,200;
//   12 = 4,200 / 15,000 = 0.280; 13 = 2,000 x 4,200 / 15,000 = 560;
//   line 14 = 2,000 - 560 = 1,440. Line 2b = 560; AGI = 106,000 - 1,440.
// The boundaries, line 9 = wages + 2,000, line 11 = line 9 - 101,800, and the
// App rounds line 13 half up to the dollar: the exclusion is whole through
// line 11 = 3 (2,000 x 3 / 15,000 = 0.4 rounds to 0), 1,999 at line 11 = 4
// (0.53 rounds to 1), 1 at line 11 = 14,996 (1,999.47 rounds to 1,999), and
// gone at 14,997 (1,999.6 rounds to 2,000) and at the top of the range, line 9
// = 116,800.
#[test]
fn savings_bond_exclusion_phases_out_across_the_magi_band() {
    let build = |wages: i64| {
        let mut d = bond_return(FederalStatus::Single, wages, 2_000);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19700315;
        add(
            &mut d,
            bond_record(Owner::Taxpayer, 8_000, 2_000, 19981001, vec![bond_student(-1, 8_000, 0)]),
        );
        d
    };
    let r = supported(&build(104_000));
    assert_eq!(value(&r, "form_8815_modified_agi"), 106_000);
    assert_eq!(value(&r, "form_8815_eligible_interest"), 2_000);
    assert_eq!(value(&r, "form_8815_phaseout_reduction"), 560);
    assert_eq!(value(&r, "savings_bond_exclusion"), 1_440);
    assert_eq!(return_line(&r, "2b"), 56_000);
    assert_eq!(value(&r, "agi"), 104_560);
    for (wages, exclusion) in [
        (99_798, 2_000),
        (99_800, 2_000),
        (99_802, 2_000),
        (99_803, 2_000),
        (99_804, 1_999),
        (114_795, 1),
        (114_796, 1),
        (114_797, 0),
        (114_798, 0),
        (130_000, 0),
    ] {
        let r = supported(&build(wages));
        assert_eq!(value(&r, "form_8815_modified_agi"), wages + 2_000, "{wages}");
        assert_eq!(value(&r, "savings_bond_exclusion"), exclusion, "wages {wages}");
        // Schedule B shows whenever the exclusion is positive.
        assert_eq!(form_coverage(&r)[&TaxForm::ScheduleB].contains("3"), exclusion > 0, "{wages}");
    }
    // The explanation names the end of the range.
    let top = supported(&build(114_800));
    let line = top.lines.iter().find(|l| l.id == "savings_bond_exclusion").unwrap();
    assert_eq!(line.explanation.key, "EXPLAIN_SAVINGS_BOND_EXCLUSION_PHASED_OUT");
    // A joint return uses its own start and range: the same record phases out
    // at 152,650 + 30,000.
    let mut joint = bond_return(FederalStatus::Joint, 180_000, 2_000);
    joint.people.as_mut().unwrap().taxpayer.birth_date = 19700315;
    add(
        &mut joint,
        bond_record(Owner::Taxpayer, 8_000, 2_000, 19981001, vec![bond_student(-1, 8_000, 0)]),
    );
    // Line 9 = 182,000; 13 = 2,000 x 29,350 / 30,000 = 1,957 (1,956.67).
    let r = supported(&joint);
    assert_eq!(value(&r, "form_8815_phaseout_reduction"), 1_957);
    assert_eq!(value(&r, "savings_bond_exclusion"), 43);
}

// F4: married filing separately takes no exclusion (IRC 135(d)(3)): the
// record stays, lines 2-8 are figured, line 14 is zero, one note, and the
// interest stays on line 2b.
#[test]
fn savings_bond_married_filing_separately_keeps_the_interest_taxable() {
    let mut d = bond_return(FederalStatus::Separate, 60_000, 3_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800501;
    let child = d.dependents[0].id;
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 12_000, 3_000, 20100601, vec![bond_student(child, 14_000, 0)]),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "form_8815_eligible_interest"), 3_000);
    assert_eq!(value(&r, "savings_bond_exclusion"), 0);
    assert_eq!(return_line(&r, "2b"), 300_000);
    assert_eq!(value(&r, "agi"), 63_000);
    assert_eq!(
        note_pages(&d, &r, "NOTE_SAVINGS_BONDS_EXCLUSION_MFS"),
        vec![TaxSection::Setup, TaxSection::InterestDividends]
    );
    // Schedule B shows line 4 only, under the $1,500 rule: $3,000 is above it,
    // and there is no line 2 or 3 without an exclusion.
    let lines = &form_coverage(&r)[&TaxForm::ScheduleB];
    assert!(lines.contains("4") && lines.contains("2") && !lines.contains("3"));
}

// F5: the owner must have attained 24 before the earliest issue date
// (IRC 135(c)(1)(B)); a person attains an age the day before the birthday, so
// the 24th birthday on the issue date qualifies.
#[test]
fn savings_bond_owner_must_be_24_before_the_issue_date_day_exact() {
    let build = |birth: u32, issued: u32| {
        let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        let child = d.dependents[0].id;
        add(
            &mut d,
            bond_record(
                Owner::Taxpayer,
                12_000,
                3_000,
                issued,
                vec![bond_student(child, 14_000, 0)],
            ),
        );
        d
    };
    // Age 21 at issue: no exclusion, a note offering the interest page and the
    // owner's own page.
    let d = build(19900601, 20120101);
    let r = supported(&d);
    assert_eq!(value(&r, "savings_bond_exclusion"), 0);
    assert_eq!(return_line(&r, "2b"), 300_000);
    assert_eq!(
        note_pages(&d, &r, "NOTE_SAVINGS_BONDS_OWNER_UNDER_24"),
        vec![TaxSection::InterestDividends, TaxSection::People]
    );
    // The 24th birthday is the issue date: attained on 2011-12-31, so it qualifies.
    let r = supported(&build(19880101, 20120101));
    assert_eq!(value(&r, "savings_bond_exclusion"), 3_000);
    assert!(!noted(&r, "NOTE_SAVINGS_BONDS_OWNER_UNDER_24"));
    // One day later (born 1988-01-02) the 24th birthday is after the issue date.
    let r = supported(&build(19880102, 20120101));
    assert_eq!(value(&r, "savings_bond_exclusion"), 0);
    assert!(noted(&r, "NOTE_SAVINGS_BONDS_OWNER_UNDER_24"));
    // A leap-day birthday: the 24th birthday of a person born 1988-02-29 is
    // 2012-02-29, so an issue on 2012-02-28 is a day early.
    let r = supported(&build(19880229, 20120228));
    assert_eq!(value(&r, "savings_bond_exclusion"), 0);
    let r = supported(&build(19880229, 20120229));
    assert_eq!(value(&r, "savings_bond_exclusion"), 3_000);
    // The spouse's bonds test the spouse's birth date.
    let mut joint = bond_return(FederalStatus::Joint, 100_000, 3_000);
    joint.people.as_mut().unwrap().taxpayer.birth_date = 19700101;
    joint.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19950101;
    joint.income[1].owner = crate::DocumentOwner::Spouses;
    let child = joint.dependents[0].id;
    add(
        &mut joint,
        bond_record(Owner::Spouse, 12_000, 3_000, 20150101, vec![bond_student(child, 14_000, 0)]),
    );
    let r = supported(&joint);
    assert_eq!(value(&r, "savings_bond_exclusion"), 0);
    assert_eq!(
        note_pages(&joint, &r, "NOTE_SAVINGS_BONDS_OWNER_UNDER_24"),
        vec![TaxSection::InterestDividends, TaxSection::Spouse]
    );
}

// Derived prior-inclusion check under the 2026 DRAFT Form8815 instructions
// (July8/2026), printed p2 line6 worksheet. $3,000 redemption interest less
// $1,000 already reported = $2,000 current interest; $12,000 proceeds stay
// unchanged. Separately, treaty-scope interest retains its existing notice.
#[test]
fn savings_bond_prior_inclusion_is_canonical_and_interest_treatments_stay_separate() {
    let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
    let child = d.dependents[0].id;
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 12_000, 3_000, 20100601, vec![bond_student(child, 14_000, 0)]),
    );
    let FederalIncome::Interest { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_interest(data);
    data.adjustments =
        Some(crate::InterestAdjustments { previously_reported: 100_000, ..Default::default() });
    drop(data);
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 0);
    assert_eq!(value(&r, "form_8815_proceeds"), 12_000);
    assert_eq!(value(&r, "form_8815_interest"), 2_000);
    assert_eq!(value(&r, "agi"), 60_000);

    for (special, key) in [(InterestSpecial::Treaty, "WARN_INTEREST_TREATY_LEFT_OUT")] {
        let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
        let FederalIncome::Interest { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_interest(data);
        data.special = special;
        drop(data);
        let r = supported(&d);
        assert!(noted(&r, key), "{key}: {:?}", r.review_notes);
        assert_eq!(value(&r, "agi"), 60_000, "{key}");
        for other in [
            "WARN_INTEREST_TREATY_LEFT_OUT",
            "WARN_INTEREST_NOMINEE_LEFT_OUT",
            "WARN_INTEREST_BOND_REPORTED_EARLIER_LEFT_OUT",
        ] {
            assert_eq!(noted(&r, other), other == key, "{key} / {other}");
        }
    }
}

// Like the 1099-INT, the 1099-DIV, 1099-OID and 1099-B treatments the pages do
// not calculate are one choice each; every reason leaves its statement out
// with its own notice and no other reason's, and the return keeps the rest.
//   Single; wages $60,000 and one statement of $1,000 (a dividend or
//   discount; a sale with $4,000 proceeds and $1,000 basis): left out, so AGI
//   stays $60,000 whatever the reason; with no reason the $1,000 (or the
//   $3,000 gain) joins AGI: 61,000 and 63,000.
#[test]
fn dividend_oid_and_sale_treatments_are_one_choice_each_with_their_own_notice() {
    use crate::{DividendSpecial as D, OidSpecial as O, SaleSpecial as S};
    let dividend = |special: D| {
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                ordinary: 100_000,
                section199a: ReitDividends::None,
                special,
                ..Default::default()
            }),
        })
    };
    let oid = |special: O| {
        income(FederalIncome::Oid {
            data: oid_source(FederalOid { oid: 100_000, special, ..Default::default() }),
        })
    };
    let sale = |special: S| {
        income(FederalIncome::Sale {
            data: sale_source(FederalSale {
                special,
                ..native_sale(GainTerm::Long, 400_000, Some(100_000))
            }),
        })
    };
    let all = [
        "WARN_DIVIDEND_SECTION_897_LEFT_OUT",
        "WARN_DIVIDEND_NOMINEE_LEFT_OUT",
        "WARN_DIVIDEND_FOREIGN_LEFT_OUT",
        "WARN_DIVIDEND_OTHER_LEFT_OUT",
        "WARN_OID_INVESTMENT_EXPENSES_LEFT_OUT",
        "WARN_OID_NOMINEE_LEFT_OUT",
        "WARN_OID_OBLIGATION_LEFT_OUT",
        "WARN_OID_OTHER_LEFT_OUT",
        "WARN_SALE_STOCK_COMPENSATION_LEFT_OUT",
        "WARN_SALE_BUSINESS_PROPERTY_LEFT_OUT",
        "WARN_SALE_HOME_LEFT_OUT",
        "WARN_SALE_OTHER_LEFT_OUT",
    ];
    let mut cases: Vec<(FederalIncomeEntry, &str)> = Vec::new();
    for (special, key) in [(D::Foreign, all[2]), (D::Another, all[3])] {
        cases.push((dividend(special), key));
    }
    for (special, key) in [(O::Obligation, all[6]), (O::Another, all[7])] {
        cases.push((oid(special), key));
    }
    for (special, key) in [
        (S::StockCompensation, all[8]),
        (S::BusinessProperty, all[9]),
        (S::Home, all[10]),
        (S::Another, all[11]),
    ] {
        cases.push((sale(special), key));
    }
    for (entry, key) in cases {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
        add(&mut d, entry);
        let r = supported(&d);
        for other in all {
            assert_eq!(noted(&r, other), other == key, "{key} / {other}");
        }
        assert_eq!(value(&r, "agi"), 60_000, "{key}");
    }
    // No reason: the statements count.
    for (entry, agi) in [
        (dividend(D::NoneOfThese), 61_000),
        (oid(O::NoneOfThese), 61_000),
        (sale(S::NoneOfThese), 63_000),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
        add(&mut d, entry);
        let r = supported(&d);
        assert!(all.iter().all(|k| !noted(&r, k)));
        assert_eq!(value(&r, "agi"), agi);
    }
}

// The savings bond record that passes the consistency checks stays quiet; the
// ones that fail block the estimate with a correction on the interest page.
#[test]
fn savings_bond_interest_above_the_1099_int_box_3_is_blocked() {
    let mut d = bond_return(FederalStatus::Single, 60_000, 2_999);
    let child = d.dependents[0].id;
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 12_000, 3_000, 20100601, vec![bond_student(child, 14_000, 0)]),
    );
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Incomplete);
    assert_eq!(crate::savings_bonds::source_rule(&d.income), Some("SAVINGS_BONDS_SOURCE_INVALID"));
    // A student who is not on the return.
    let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 12_000, 3_000, 20100601, vec![bond_student(-2, 14_000, 0)]),
    );
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key
            == "ISSUE_THE_SELECTED_STUDENT_MUST_BE_A_FILER_OR_AN_ELIGIBLE_DEPENDENT_ON_TH_879AF16D"),
        "{:?}",
        r.issues
    );
}

// Contributions to a 529 plan or Coverdell ESA count as expenses (IRC
// 135(c)(2)(C)); benefits reduce them; the exclusion stays within the interest.
#[test]
fn savings_bond_contributions_add_and_benefits_reduce_the_expenses() {
    let mut d = bond_return(FederalStatus::Single, 60_000, 4_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800501;
    let child = d.dependents[0].id;
    let mut student = bond_student(child, 3_000, 1_000);
    student.contributions = 2_500 * 100;
    add(&mut d, bond_record(Owner::Taxpayer, 10_000, 4_000, 20100601, vec![student]));
    // Line 2 = 3,000 + 2,500 = 5,500; 3 = 1,000; 4 = 4,500; 8 = 4,000 x
    // 4,500 / 10,000 = 1,800.
    let r = supported(&d);
    assert_eq!(value(&r, "form_8815_qualified_expenses"), 5_500);
    assert_eq!(value(&r, "form_8815_net_expenses"), 4_500);
    assert_eq!(value(&r, "savings_bond_exclusion"), 1_800);
    assert_eq!(return_line(&r, "2b"), 220_000);
}

#[test]
fn per_statement_netting_is_one_rule_for_the_return_the_preview_and_a_childs_election() {
    // 1099-INT: box 1 $1,000 + box 3 $500 - box 11 $300 - box 12 $100 -
    // accrued interest $50 = $1,050. A premium larger than the payer's own
    // interest floors at zero and never reduces another payer's: box 1 $100 -
    // box 11 $400 = 0. 1099-OID: box 1 $1,000 + box 2 $200 + box 8 -$50 - box
    // 6 $100 - box 10 $80 = $970, plus box 5 $30 = $1,000.
    let interest = FederalInterest {
        foreign_tax: 0,
        us_obligations: 50_000,
        taxable: 100_000,
        bond_premium: 30_000,
        treasury_bond_premium: 10_000,
        accrued_interest: 5_000,
        ..match child_interest(0) {
            ChildIncome::Interest { data, .. } => *data,
            _ => unreachable!(),
        }
    };
    let floored = FederalInterest {
        foreign_tax: 0,
        taxable: 10_000,
        us_obligations: 0,
        bond_premium: 40_000,
        treasury_bond_premium: 0,
        accrued_interest: 0,
        ..interest.clone()
    };
    let oid = FederalOid {
        oid: 100_000,
        other_periodic_interest: 20_000,
        market_discount: 3_000,
        acquisition_premium: 10_000,
        bond_premium: 8_000,
        treasury_oid: -5_000,
        ..Default::default()
    };
    assert_eq!(interest.net_taxable(), 105_000);
    assert_eq!(floored.net_taxable(), 0);
    assert_eq!(oid.net_taxable(), 100_000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Interest { data: interest_source(interest.clone()) }));
    add(&mut d, income(FederalIncome::Interest { data: interest_source(floored.clone()) }));
    add(&mut d, income(FederalIncome::Oid { data: oid_source(oid.clone()) }));
    d.screening.as_mut().unwrap().taxable_premium_election = Some(true);
    let r = supported(&d);
    // Form 1040 line 2b is the three nets.
    assert_eq!(return_line(&r, "2b"), 205_000);
    // The preview of each document is its own net.
    assert_eq!(
        crate::interview::document_amount(
            &FederalIncome::Interest { data: interest_source(interest.clone()) },
            false
        ),
        Some(105_000)
    );
    assert_eq!(
        crate::interview::document_amount(
            &FederalIncome::Oid { data: oid_source(oid.clone()) },
            false
        ),
        Some(100_000)
    );
    // The same nets, statement by statement, are a child's Form 8814 line 1a.
    let election = FederalChildElection {
        child_id: 1,
        statements: vec![
            ChildIncome::Interest { data: interest.into(), kept: ChildStatementKept::default() },
            ChildIncome::Interest { data: floored.into(), kept: ChildStatementKept::default() },
            ChildIncome::Oid { data: oid.into(), kept: ChildStatementKept::default() },
        ],
        alaska_dividend: 0,
        nontaxable_social: 0,
        other_parent: OtherParentChoice::LivedApart,
        facts: 0,
    };
    assert_eq!(election.lines().line_1a, 2_050);
}

// Form 6781 Part I (unit B2): section 1256 contracts, 60/40. Amounts in
// dollars in the comments; the fixtures store cents.
fn contracts(box11: i64, owner: Owner) -> FederalIncomeEntry {
    let mut entry = income(FederalIncome::Section1256 {
        data: contracts_source(crate::FederalSection1256 {
            aggregate: box11,
            withholding: 0,
            special_treatment: false,
        }),
    });
    entry.owner = crate::DocumentOwner::from(owner);
    entry.label = "Synthetic broker".into();
    entry
}
fn long_gain(proceeds: i64, basis: i64) -> FederalIncomeEntry {
    income(FederalIncome::Sale {
        data: sale_source(native_sale(GainTerm::Long, proceeds, Some(basis))),
    })
}
fn issue_keys(r: &FederalResult) -> Vec<&str> {
    r.issues.iter().map(|i| i.display.key.as_str()).collect()
}
// F2's return: joint, $150,000 wages, a $20,000 loss and a $2,500 gain on
// contracts, and a $4,000 long-term sale gain.
fn contracts_loss_return(carryback: Option<i64>) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 15_000_000, 0)];
    d.next_id = 2;
    add(&mut d, contracts(-2_000_000, Owner::Taxpayer));
    add(&mut d, contracts(250_000, Owner::Spouse));
    add(&mut d, long_gain(1_400_000, 1_000_000));
    d.screening.as_mut().unwrap().section_1256_carryback = carryback;
    d
}

#[test]
fn form_6781_net_gain_is_taxed_60_40() {
    // F1, single: wages $80,000 and one statement with box 11 $10,000.
    // Form 6781: 1(c) 10,000; 2b 0; 2c 10,000; 3 = 5 = 10,000; 6 = 0;
    // 7 = 10,000; 8 = 40% = 4,000; 9 = 60% = 6,000. Schedule D: line 4
    // 4,000, line 11 6,000, line 7 4,000, line 15 6,000, line 16 10,000.
    // AGI 90,000; taxable 90,000 - 16,100 = 73,900. QDCG worksheet: the 6,000
    // is preferential, the ordinary part 67,900 (Tax Table row 67,900-67,950,
    // midpoint 67,925: 1,240 + 4,560 + 0.22 x 17,525 = 9,655.50, rounds to
    // 9,656); 6,000 x 15% = 900 (taxable is above 49,450); tax 10,556.
    // All ordinary would be the 73,900 row, midpoint 73,925: 1,240 + 4,560 +
    // 0.22 x 23,525 = 10,975.50, rounds to 10,976; the saving 420 = 6,000 x
    // (22% - 15%).
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 8_000_000, 0)];
    d.next_id = 2;
    add(&mut d, contracts(1_000_000, Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "form_6781_losses"), 0);
    assert_eq!(value(&r, "form_6781_gains"), 10_000);
    assert_eq!(value(&r, "form_6781_net"), 10_000);
    assert_eq!(value(&r, "form_6781_total"), 10_000);
    assert_eq!(value(&r, "form_6781_short_term"), 4_000);
    assert_eq!(value(&r, "form_6781_long_term"), 6_000);
    assert_eq!(value(&r, "section_1256_2_aggregate"), 10_000);
    assert!(r.lines.iter().all(|l| l.id != "form_6781_carryback"));
    assert_eq!(value(&r, "capital_gain_4"), 4_000);
    assert_eq!(value(&r, "capital_gain_11"), 6_000);
    assert_eq!(value(&r, "capital_gain"), 10_000);
    assert_eq!(value(&r, "agi"), 90_000);
    assert_eq!(return_line(&r, "15"), 7_390_000);
    assert_eq!(return_line(&r, "16"), 1_055_600);
    // The same 10,000 as ordinary income would owe 420 more.
    let mut ordinary = d.clone();
    ordinary.income.pop();
    ordinary.income[0] = wage(1, Owner::Taxpayer, 9_000_000, 0);
    assert_eq!(return_line(&supported(&ordinary), "16"), 1_097_600);
    let coverage = crate::form_coverage(&r);
    let lines = &coverage[&TaxForm::Form6781];
    for line in ["2", "5", "7", "8", "9"] {
        assert!(lines.contains(line), "{line}");
    }
    assert!(!lines.contains("6"));
    assert_eq!(printed_line(&r, "form_6781_short_term"), (TaxForm::Form6781, "8".into()));
}

#[test]
fn form_6781_net_loss_carries_over_without_carryback() {
    // F2, joint: wages $150,000 (taxpayer); statement A (taxpayer) box 11
    // -20,000; statement B (spouse) +2,500; one long-term sale, proceeds
    // 14,000, basis 10,000 (gain 4,000). Form 6781: 2b (20,000); 2c 2,500;
    // 3 = 5 = -17,500; 6 = 0; 7 = -17,500; 8 = -7,000; 9 = -10,500.
    // Schedule D: line 7 -7,000; line 15 4,000 - 10,500 = -6,500; line 16
    // -13,500; line 21 (3,000). AGI 147,000; taxable 147,000 - 32,200 =
    // 114,800; tax by schedule 2,480 + 9,120 + 0.22 x 14,000 = 14,680.
    // Carryover to 2027: short 7,000 - 3,000 = 4,000, long 6,500, total
    // 10,500 = 13,500 - 3,000. The box D bound is min(10,500, 17,500 + 0 -
    // 3,000 = 14,500) = 10,500, shown with no carryback and no note.
    let d = contracts_loss_return(None);
    let r = supported(&d);
    assert_eq!(value(&r, "form_6781_losses"), -20_000);
    assert_eq!(value(&r, "form_6781_gains"), 2_500);
    assert_eq!(value(&r, "form_6781_net"), -17_500);
    assert_eq!(value(&r, "form_6781_short_term"), -7_000);
    assert_eq!(value(&r, "form_6781_long_term"), -10_500);
    assert_eq!(value(&r, "capital_gain_4"), -7_000);
    assert_eq!(value(&r, "capital_gain_11"), -10_500);
    assert_eq!(value(&r, "capital_gain"), -3_000);
    assert_eq!(value(&r, "agi"), 147_000);
    assert_eq!(return_line(&r, "15"), 11_480_000);
    assert_eq!(return_line(&r, "16"), 1_468_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 4_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 6_500);
    assert_eq!(value(&r, "capital_loss_carryforward"), 10_500);
    assert_eq!(value(&r, "section_1256_net_loss"), 10_500);
    assert!(!noted(&r, "NOTE_SECTION_1256_CARRYBACK"));
}

#[test]
fn form_6781_carryback_moves_gain_into_the_loss_year_and_checks_the_bound() {
    // F3: F2 plus a carryback of 8,000. Line 6 = 8,000; 7 = -9,500; 8 =
    // -3,800; 9 = -5,700. Schedule D: line 7 -3,800; line 15 4,000 - 5,700 =
    // -1,700; line 16 -5,500; line 21 (3,000). AGI and tax are unchanged
    // (147,000; 14,680). Carryover: short 3,800 - 3,000 = 800, long 1,700,
    // total 2,500 = 10,500 - 8,000 (section 1212(c)(6)(A): F2's 4,000 - 40%
    // x 8,000 = 800 and 6,500 - 60% x 8,000 = 1,700).
    let r = supported(&contracts_loss_return(Some(800_000)));
    assert_eq!(value(&r, "form_6781_carryback"), 8_000);
    assert_eq!(value(&r, "form_6781_total"), -9_500);
    assert_eq!(value(&r, "form_6781_short_term"), -3_800);
    assert_eq!(value(&r, "form_6781_long_term"), -5_700);
    assert_eq!(value(&r, "capital_gain_4"), -3_800);
    assert_eq!(value(&r, "capital_gain_11"), -5_700);
    assert_eq!(value(&r, "capital_gain"), -3_000);
    assert_eq!(value(&r, "agi"), 147_000);
    assert_eq!(return_line(&r, "16"), 1_468_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 800);
    assert_eq!(value(&r, "capital_long_carryforward"), 1_700);
    assert_eq!(value(&r, "capital_loss_carryforward"), 2_500);
    // The bound does not move with the amount entered.
    assert_eq!(value(&r, "section_1256_net_loss"), 10_500);
    assert_eq!(
        r.review_notes.iter().filter(|n| n.display.key == "NOTE_SECTION_1256_CARRYBACK").count(),
        1
    );
    assert!(noted(&r, "NOTE_SECTION_1256_CARRYBACK"));

    // Boundary: 10,500 is accepted (7 = -7,000; 8 = -2,800; 9 = -4,200; line
    // 16 -3,000; carryover 0); 10,501 is refused with the bound 10,500.
    let r = supported(&contracts_loss_return(Some(1_050_000)));
    assert_eq!(value(&r, "form_6781_total"), -7_000);
    assert_eq!(value(&r, "form_6781_short_term"), -2_800);
    assert_eq!(value(&r, "form_6781_long_term"), -4_200);
    assert_eq!(value(&r, "capital_gain"), -3_000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
    let r = estimate(&contracts_loss_return(Some(1_050_100)));
    assert_ne!(r.state, ResultState::EstimatedForSupportedScope);
    let issue = r
        .issues
        .iter()
        .find(|i| i.display.key == "ISSUE_SECTION_1256_CARRYBACK_ABOVE_NET_LOSS")
        .unwrap_or_else(|| panic!("{:?}", r.issues));
    assert_eq!(
        issue.display.args,
        vec![
            crate::MessageArg::Currency {
                currency_code: "USD".into(),
                scale: 0,
                minor_units: 10_501,
            },
            crate::MessageArg::Currency {
                currency_code: "USD".into(),
                scale: 0,
                minor_units: 10_500,
            },
        ]
    );
    assert_eq!(issue.context, crate::IssueContext::Section(crate::TaxSection::InvestmentSales));

    // F3b: a carryback of 1,000 against a net gain (F1) has no loss to carry.
    let mut gain = complete(FederalStatus::Single);
    gain.income = vec![wage(1, Owner::Taxpayer, 8_000_000, 0)];
    gain.next_id = 2;
    add(&mut gain, contracts(1_000_000, Owner::Taxpayer));
    gain.screening.as_mut().unwrap().section_1256_carryback = Some(100_000);
    let r = estimate(&gain);
    assert_ne!(r.state, ResultState::EstimatedForSupportedScope);
    assert!(issue_keys(&r).contains(&"ISSUE_SECTION_1256_CARRYBACK_WITHOUT_LOSS"));
    // A carryback with no statement at all has none either.
    gain.income.pop();
    let r = estimate(&gain);
    assert!(issue_keys(&r).contains(&"ISSUE_SECTION_1256_CARRYBACK_WITHOUT_LOSS"));
}

#[test]
fn form_6781_carryback_bound_counts_a_2025_carryover_and_the_separate_limit() {
    // Married filing separately: wages 60,000 and a net contracts loss of
    // 5,000 (8 = -2,000, 9 = -3,000). No 2025 carryover: the limit is
    // 1,500, so the bound is min(carryover 3,500, 5,000 - 1,500) = 3,500.
    let mut d = complete(FederalStatus::Separate);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    d.next_id = 2;
    add(&mut d, contracts(-500_000, Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "form_6781_net"), -5_000);
    assert_eq!(value(&r, "capital_gain"), -1_500);
    assert_eq!(value(&r, "capital_loss_carryforward"), 3_500);
    assert_eq!(value(&r, "section_1256_net_loss"), 3_500);
    d.screening.as_mut().unwrap().section_1256_carryback = Some(350_000);
    let r = supported(&d);
    // 7 = -1,500; 8 = -600; 9 = -900; nothing carries over.
    assert_eq!(value(&r, "form_6781_short_term"), -600);
    assert_eq!(value(&r, "form_6781_long_term"), -900);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
    d.screening.as_mut().unwrap().section_1256_carryback = Some(350_100);
    assert!(issue_keys(&estimate(&d)).contains(&"ISSUE_SECTION_1256_CARRYBACK_ABOVE_NET_LOSS"));
}

#[test]
fn form_6781_carryback_bound_counts_the_section_1256_part_of_the_carryover() {
    // Single: wages 60,000 and a net contracts loss of 5,000 (8 = -2,000, 9 =
    // -3,000). The 2025 return (AGI 62,000, deductions 12,000, so 50,000 of
    // taxable income used the whole 3,000): Schedule D short -10,000, long
    // -13,000, so the carryovers are 7,000 and 13,000, 20,000 in all. 2026
    // Schedule D: short -2,000 - 7,000 = -9,000, long -3,000 - 13,000 =
    // -16,000; 3,000 deducted; the 2027 carryover is 22,000. Box D's first
    // bullet (draft Form 6781; Pub. 550 treats a section 1256 carryover as a
    // 2026 section 1256 loss): 5,000 plus the section 1256 part, less 3,000.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    d.next_id = 2;
    add(&mut d, contracts(-500_000, Owner::Taxpayer));
    d.screening.as_mut().unwrap().capital_history = vec![PriorCapitalReturn {
        injured_niit_owners: None,
        injured_owners: None,

        owners: PriorReturnOwners::Taxpayer,
        status: FederalStatus::Single,
        agi: 6_200_000,
        deductions: 1_200_000,
        senior_deduction: 0,
        short: -1_000_000,
        long: -1_300_000,
        deducted_loss: 300_000,
        section_1256: 0,
        amt: None,
        origins: vec![],
        ordinary_history: true,
        investment_parts: Some(InvestmentCapitalParts {
            short: 700_000,
            long: 1_300_000,
            amt: None,
        }),
        niit_difference: Some(0),
        research_parts: None,
    }];
    let bound = |d: &FederalDraft| value(&supported(d), "section_1256_net_loss");
    let r = supported(&d);
    assert_eq!(value(&r, "capital_loss_carryforward"), 22_000);
    // None of the carryover is section 1256: 5,000 - 3,000 = 2,000.
    assert_eq!(bound(&d), 2_000);
    // 10,000 of it is: 5,000 + 10,000 - 3,000 = 12,000.
    d.screening.as_mut().unwrap().capital_history[0].section_1256 = 1_000_000;
    assert_eq!(bound(&d), 12_000);
    // All 20,000 of it: 5,000 + 20,000 - 3,000 = 22,000, the carryover binds.
    d.screening.as_mut().unwrap().capital_history[0].section_1256 = 2_000_000;
    assert_eq!(bound(&d), 22_000);
    // The carryback boundary follows: 22,000 passes, 22,001 is refused.
    d.screening.as_mut().unwrap().section_1256_carryback = Some(2_200_000);
    supported(&d);
    d.screening.as_mut().unwrap().section_1256_carryback = Some(2_200_100);
    assert!(issue_keys(&estimate(&d)).contains(&"ISSUE_SECTION_1256_CARRYBACK_ABOVE_NET_LOSS"));
    // The section 1256 part is part of the carryover, not more than it.
    d.screening.as_mut().unwrap().section_1256_carryback = None;
    d.screening.as_mut().unwrap().capital_history[0].section_1256 = 2_000_100;
    assert!(
        issue_keys(&estimate(&d))
            .contains(&"ISSUE_THE_SECTION_1256_PART_OF_THE_PRIOR_CARRYOVER_EXCEEDS_THE_CARRYOVER")
    );
}

#[test]
fn form_6781_open_contracts_and_rounding() {
    // F4, single: wages $60,000. The broker's box 11 combines a contract
    // bought in 2025 for 50,000 and marked at 57,000 on the 2025 return (7,000
    // already taxed), sold in 2026 for 56,000 (6,000 realized, reduced by the
    // 7,000 mark: -1,000), and a contract opened in 2026 with 2,500 unrealized
    // gain on December 31, 2026 (taxed now, section 1256(a)(1)): box 11 =
    // -1,000 + 2,500 = 1,500. Form 6781: 7 = 1,500; 8 = 600; 9 = 900. AGI
    // 61,500; taxable 45,400; the 900 long-term part is in the 0% band
    // (45,400 <= 49,450): tax on ordinary 44,500 (row midpoint 44,525: 1,240 +
    // 0.12 x 32,125 = 5,095) + 0 = 5,095. All ordinary would be 5,203 (row
    // midpoint 45,425: 1,240 + 0.12 x 33,025 = 5,203), 108 = 900 x 12% more.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    d.next_id = 2;
    add(&mut d, contracts(150_000, Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "form_6781_total"), 1_500);
    assert_eq!(value(&r, "form_6781_short_term"), 600);
    assert_eq!(value(&r, "form_6781_long_term"), 900);
    assert_eq!(value(&r, "agi"), 61_500);
    assert_eq!(return_line(&r, "15"), 4_540_000);
    assert_eq!(return_line(&r, "16"), 509_500);
    // Rounding: box 11 -1,000.50 is line 1 -1,001 (half rounds away from
    // zero); 8 = 40% of -1,001 = -400.4 -> -400; 9 = -601 (-600.6 rounded);
    // 8 + 9 = -1,001.
    d.income[1] = contracts(-100_050, Owner::Taxpayer);
    d.income[1].id = 2;
    let r = supported(&d);
    assert_eq!(value(&r, "section_1256_2_aggregate"), -1_001);
    assert_eq!(value(&r, "form_6781_short_term"), -400);
    assert_eq!(value(&r, "form_6781_long_term"), -601);
    assert_eq!(value(&r, "capital_gain"), -1_001);
}

#[test]
fn form_6781_left_out_statement_leaves_the_rest_of_the_return() {
    // F5: F1 with the special answer Yes. The statement leaves the working
    // copy with one notice; no Form 6781 line, Schedule D lines 4 and 11
    // are zero, AGI is the wages, 80,000.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 8_000_000, 0)];
    d.next_id = 2;
    let mut entry = contracts(1_000_000, Owner::Taxpayer);
    let FederalIncome::Section1256 { data } = &mut entry.income else { unreachable!() };
    data.special = true;
    data.form.withholding = 100_000;
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(
        r.review_notes.iter().filter(|n| n.display.key == "WARN_SECTION_1256_LEFT_OUT").count(),
        1
    );
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_6781")));
    assert!(!crate::form_coverage(&r).contains_key(&TaxForm::Form6781));
    assert_eq!(value(&r, "capital_gain_4"), 0);
    assert_eq!(value(&r, "capital_gain_11"), 0);
    assert_eq!(value(&r, "agi"), 80_000);
    // Its withholding leaves with it.
    assert_eq!(return_line(&r, "25b"), 0);
}

#[test]
fn form_6781_statement_withholding_reaches_form_1040_line_25b() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 8_000_000, 0)];
    d.next_id = 2;
    let mut entry = contracts(1_000_000, Owner::Taxpayer);
    let FederalIncome::Section1256 { data } = &mut entry.income else { unreachable!() };
    data.form.withholding = 100_000;
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "25b"), 100_000);
}

#[test]
fn form_6781_adds_to_the_form_6252_earlier_year_amounts() {
    // One Schedule D path: a short-held investment sale's 1,000 from Form
    // 6252 and a 5,000 contracts gain (8 = 2,000, 9 = 3,000) total 3,000 on
    // line 4 and 3,000 on line 11.
    let mut d = earlier_return(
        60_000,
        vec![earlier_sale(crate::PropertyCharacter::ShortTermCapital, 3_000, 12_000, 2_000, 4_000)],
    );
    add(&mut d, contracts(500_000, Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "installment_1_gain"), 1_000);
    assert_eq!(value(&r, "capital_gain_4"), 3_000);
    assert_eq!(value(&r, "capital_gain_11"), 3_000);
    assert_eq!(value(&r, "capital_gain"), 6_000);
}

#[test]
fn form_6781_straddles_are_refused_by_name() {
    // F6: the Straddles situation refuses the estimate with its own name,
    // whatever statements the return carries.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, contracts(1_000_000, Owner::Taxpayer));
    let mut refused = d.clone();
    Chosen::from(RareSituation::Straddles).choose(&mut refused);
    let r = estimate(&refused);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_SITUATION_STRADDLES"));
    assert!(r.tax.is_none() && r.lines.is_empty());
    assert_situation_refused(
        &complete(FederalStatus::Single),
        RareSituation::Straddles,
        &["Straddles", "Form 6781 Part II", "boxes A–C"],
    );
}

#[test]
fn form_6781_gain_and_loss_are_net_investment_income() {
    // Single, wages $195,000 and a $10,000 contracts gain: MAGI 205,000 is
    // 5,000 over the 200,000 threshold and net investment income is the full
    // 10,000, so the tax is 3.8% x 5,000 = 190. A $10,000 contracts loss
    // (8 = -4,000, 9 = -6,000) leaves no net investment income to tax.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 19_500_000, 0)];
    d.next_id = 2;
    add(&mut d, contracts(1_000_000, Owner::Taxpayer));
    assert_eq!(value(&supported(&d), "net_investment_income_tax"), 190);
    d.income[1] = contracts(-1_000_000, Owner::Taxpayer);
    d.income[1].id = 2;
    assert_eq!(value(&supported(&d), "net_investment_income_tax"), 0);
}

// ---- Unit F4 (2026-10-10, line audit schedules-a-b-d-8949 D7-D10) ----

fn reported(mut lot: FederalIncome) -> FederalIncome {
    let FederalIncome::Sale { data } = &mut lot else { unreachable!() };
    data.form.reporting = BasisReporting::ReportedToIrs;
    lot
}

// D7: the Schedule D instructions keep collectibles (Form 8949 code C) and personal-use losses
// (code L) off lines 1a and 8a, so a basis-reported lot of either kind goes to line 1b/8b.
// Gold coins sold through a broker (1099-B basis reported, no adjustment), long term, proceeds
// $10,000, basis $6,000: line 8b $4,000, line 8a $0. An ordinary reported lot stays on 8a.
#[test]
fn collectibles_and_personal_losses_leave_the_summary_lines() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(reported(collectible_sale(10_000, 6_000, GainTerm::Long, false))));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_8a"), 0);
    assert_eq!(value(&r, "capital_gain_8b"), 4_000);
    assert_eq!(value(&r, "rate_28_gain"), 4_000);
    // The short-term twin goes to line 1b.
    let mut short = complete(FederalStatus::Single);
    add(&mut short, income(reported(collectible_sale(10_000, 6_000, GainTerm::Short, false))));
    let r = supported(&short);
    assert_eq!(value(&r, "capital_gain_1a"), 0);
    assert_eq!(value(&r, "capital_gain_1b"), 4_000);
    // A personal-use item sold at a loss on a basis-reported 1099-B is code L: its column (g)
    // adds the $1,000 loss back, so line 8b is $0 and line 8a still $0; an ordinary investment
    // lot beside it is the only 8a entry.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(reported(personal_sale(5_000, 6_000, GainTerm::Long))));
    let mut ordinary = personal_sale(9_000, 4_000, GainTerm::Long);
    let FederalIncome::Sale { data } = &mut ordinary else { unreachable!() };
    data.form.use_kind = SaleUse::Investment { wash_sale: 0 };
    add(&mut d, income(reported(ordinary)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_8a"), 5_000);
    assert_eq!(value(&r, "capital_gain_8b"), 0);
    assert_eq!(value(&r, "personal_sale_disallowed_loss"), 1_000);
    // A personal-use item sold at a gain is an ordinary 8a entry.
    let mut gain = complete(FederalStatus::Single);
    add(&mut gain, income(reported(personal_sale(8_000, 6_000, GainTerm::Long))));
    let r = supported(&gain);
    assert_eq!(value(&r, "capital_gain_8a"), 2_000);
    assert_eq!(value(&r, "capital_gain_8b"), 0);
}

// D8: Form 4684 personal casualty and theft gains are Schedule D lines 4 (short) and 11 (long), so
// lines 1a-4 and 8a-11 foot to lines 7 and 15. A personal car held three years, basis $15,000,
// stolen and paid $20,000 by insurance: Form 4684 gain $5,000 on line 11.
#[test]
fn form_4684_gain_is_schedule_d_line_11() {
    let mut d = complete(FederalStatus::Single);
    let mut stolen = personal_disaster("Stolen car", 1_500_000);
    stolen.declaration = CasualtyDeclaration::Undeclared;
    stolen.zip.clear();
    stolen.properties[0].before = 2_000_000;
    stolen.properties[0].reimbursement = 2_000_000;
    stolen.properties[0].received = 2_000_000;
    d.deductions.as_mut().unwrap().personal_casualties.push(stolen.clone());
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_11"), 5_000);
    assert_eq!(value(&r, "capital_gain_4"), 0);
    assert_eq!(value(&r, "capital_gain"), 5_000);
    // Held under a year: line 4.
    stolen.properties[0].acquisition = CasualtyAcquisition::Purchased { date: 20260301 };
    d.deductions.as_mut().unwrap().personal_casualties[0] = stolen;
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_4"), 5_000);
    assert_eq!(value(&r, "capital_gain_11"), 0);
}

// D9: Schedule D line 11 includes the Form 4797 Part I gain. The sold rental of
// form_4797_rental_sale_matches_hand_computation has a Form 4797 gain of $111,833 and no other
// capital item, so line 11 and the whole capital gain are $111,833.
#[test]
fn form_4797_part_i_gain_is_schedule_d_line_11() {
    let mut other = rental(6_000, 14_000);
    other.participation = crate::RentalParticipation::NotActive;
    let r = supported(&rental_sale_return(vec![sold_rental(5_000), other]));
    assert_eq!(value(&r, "form4797_line10"), 111_833);
    assert_eq!(value(&r, "capital_gain_11"), 111_833);
    assert_eq!(value(&r, "capital_gain"), 111_833);
}

// D10: Form 8949 code E on personal-use property never makes a deductible loss. Personal item sold
// for $10,000 with basis $12,000 and $500 of selling expenses: column (g) is code E -$500 and code
// L +$2,500, so column (h) is $0 (the loss after expenses, $2,500, is disallowed). With proceeds
// $12,100 the loss after expenses is $400, added back by code L: column (h) is $0 again.
#[test]
fn personal_use_selling_expenses_never_create_a_loss() {
    for (proceeds, disallowed) in [(10_000, 2_500), (12_100, 400)] {
        let mut d = complete(FederalStatus::Single);
        let mut item = reported(personal_sale(proceeds, 12_000, GainTerm::Long));
        let FederalIncome::Sale { data } = &mut item else { unreachable!() };
        data.adjustment = SaleAdjustment::SellingExpenses { amount: 50_000 };
        add(&mut d, income(item));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain_8b"), 0, "{proceeds}");
        assert_eq!(value(&r, "capital_gain"), 0, "{proceeds}");
        assert_eq!(value(&r, "personal_sale_disallowed_loss"), disallowed);
    }
    // A gain is still reduced by the expenses (proceeds $13,000: 13,000 - 12,000 - 500).
    let mut d = complete(FederalStatus::Single);
    let mut item = reported(personal_sale(13_000, 12_000, GainTerm::Long));
    let FederalIncome::Sale { data } = &mut item else { unreachable!() };
    data.adjustment = SaleAdjustment::SellingExpenses { amount: 50_000 };
    add(&mut d, income(item));
    assert_eq!(value(&supported(&d), "capital_gain_8b"), 500);
    // An investment lot keeps the deductible loss: proceeds $10,000, basis $12,000, expenses $500.
    let mut d = complete(FederalStatus::Single);
    let mut item = reported(personal_sale(10_000, 12_000, GainTerm::Long));
    let FederalIncome::Sale { data } = &mut item else { unreachable!() };
    data.form.use_kind = SaleUse::Investment { wash_sale: 0 };
    data.adjustment = SaleAdjustment::SellingExpenses { amount: 50_000 };
    add(&mut d, income(item));
    assert_eq!(value(&supported(&d), "capital_gain_8b"), -2_500);
}

// F3 (2026-10-09), rows 196-198 and 201: the real estate professional status is
// the person's, each rental says whether the person materially participated in
// it, and the Form 8582 MAGI leaves out an allowed professional loss.
pub(in crate::tests) fn rental_loss_return(wages: i64) -> FederalDraft {
    // Rent 0, expenses 10,000 and the fixture's depreciation of 10,000
    // (275,000 x 24/660) make each rental a -$20,000 loss.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    d
}

#[test]
fn a_prior_loss_from_an_inactive_year_stays_out_of_the_special_allowance() {
    // MAGI 60,000 leaves the full 25,000 allowance. Current loss -20,000 and
    // a 30,000 prior unallowed loss. Active in the loss year: -50,000
    // competes for 25,000, so 25,000 is suspended. Not active then: only the
    // current -20,000 is eligible, so 20,000 is allowed and 30,000 stays
    // suspended (section 469(i)(1), i8582 Part V Worksheet 3).
    for (active_then, suspended) in [(true, 25_000), (false, 30_000)] {
        let mut d = rental_loss_return(60_000);
        let mut data = rental(0, 10_000);
        data.history =
            RentalHistory { regular: 3_000_000, amt: 3_000_000, active_in_loss_year: active_then };
        add(&mut d, income(FederalIncome::Rental { data }));
        let r = supported(&d);
        let id = d.income[1].id;
        assert_eq!(
            value(&r, &alloc::format!("rental_{id}_suspended_loss")),
            suspended,
            "{active_then}"
        );
        assert_eq!(value(&r, "agi"), 60_000 - (50_000 - suspended));
    }
}

// ---- Unit B11 4797: rental disposition specials (hand-computed) ----

fn rental_improvement(cost: i64, in_service: u32) -> RentalBuilding {
    purchased_rental_building(cost * 100, 0, in_service)
}
fn set_sale(
    v: &mut FederalRental,
    price: i64,
    land: i64,
    expenses: i64,
    payment: crate::SalePayment,
) {
    v.sale = crate::RentalSale::Sold {
        date: 20260815,
        price: price * 100,
        land_price: land * 100,
        selling_expenses: expenses * 100,
        payment,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
}
fn rental_installment(payments: i64) -> crate::SalePayment {
    crate::SalePayment::Installment {
        assumed_debt: 0,
        payments: payments * 100,
        interest: crate::NoteInterest::Adequate,
        special: false,
    }
}

#[test]
fn f5_installment_sale_releases_the_suspended_loss_in_the_ratio_of_gain_recognized() {
    // The sold-rental fixture: gross profit $111,833 (Form 6252 line 16), $100,000
    // received of the $400,000 price: line 24 $27,958 = 25% of the gross profit. The
    // 2026 loss is $8,250 - $4,000 - $6,250 = $2,000 and $10,000 was suspended, an overall
    // loss of $12,000 of which 27,958 / 111,833 = $3,000 is nonpassive; $9,000 stays
    // unallowed until later payments (section 469(g)(3), Pub. 925).
    let mut property = sold_rental(10_000);
    property.rent = 8_250 * 100;
    set_sale(&mut property, 400_000, 80_000, 24_000, rental_installment(100_000));
    let r = supported(&rental_sale_return(vec![property]));
    assert_eq!(value(&r, "rental_2_sale_form6252_gross_profit"), 111_833);
    assert_eq!(value(&r, "rental_2_sale_form6252_income"), 27_958);
    assert_eq!(value(&r, "rental_2_line22_allowed_loss"), -3_000);
    assert_eq!(value(&r, "rental_2_sale_released_loss"), 3_000);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 9_000);
    // $60,000 wages + the $27,958 gain - the $3,000 allowed loss.
    assert_eq!(value(&r, "agi"), 84_958);
}

#[test]
fn installment_sale_with_a_loss_on_the_land_reports_the_building_on_form_6252_only() {
    // Building price $360,000 against $275,000 cost less $65,833 depreciation: gain
    // $150,833; the land sold for $40,000 against $55,000 is a $15,000 loss reported in
    // full on line 2 (Pub. 537, "Single Sale of Several Assets"). The building's Form
    // 6252 takes 360/400 of the $100,000 received: $90,000 x 150,833 / 360,000 = $37,708.
    let mut property = sold_rental(0);
    set_sale(&mut property, 400_000, 40_000, 0, rental_installment(100_000));
    let r = supported(&rental_sale_return(vec![property]));
    assert_eq!(value(&r, "rental_2_sale_form6252_gross_profit"), 150_833);
    assert_eq!(value(&r, "rental_2_sale_form6252_income"), 37_708);
    assert_eq!(value(&r, "rental_2_sale_land"), -15_000);
    assert_eq!(value(&r, "form4797_line8"), 22_708);
}

#[test]
fn f6_real_estate_professional_gain_leaves_net_investment_income() {
    // A professional who meets the 500-hour test in the year of the sale: the gain
    // is deemed derived from property used in the ordinary course of a trade or
    // business (Reg. 1.1411-4(g)(7)(i)(B), i8960): no net investment income tax.
    let tax = |hours: bool| {
        let mut d = rental_sale_return(vec![]);
        d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
        let mut property = sold_rental(0);
        property.participation = crate::RentalParticipation::Professional;
        add(&mut d, income(FederalIncome::Rental { data: property }));
        make_professional(&mut d, Owner::Taxpayer, hours);
        supported(&d)
    };
    let safe = tax(true);
    assert_eq!(value(&safe, "net_investment_income_tax"), 0);
    assert!(value(&safe, "niit_disposition_adjustment") < 0);
    let other = tax(false);
    assert!(value(&other, "net_investment_income_tax") > 0);
}

#[test]
fn sale_after_personal_use_splits_the_rental_and_personal_parts() {
    // 20% personal use over the years owned: price $400,000 (land $80,000) and
    // $24,000 of expenses; rental part $320,000 / $64,000 / $19,200. Building price
    // $256,000, expenses $15,360, cost 80% of $275,000 = $220,000, all $65,833 of
    // depreciation: gain $240,640 - ($220,000 - $65,833) = $86,473. Land $64,000 -
    // $3,840 - $44,000 = $16,160. Personal part: proceeds $376,000 - $300,800 =
    // $75,200 against $66,000 of cost: a $9,200 Form 8949 gain.
    let mut property = sold_rental(0);
    set_sale(&mut property, 400_000, 80_000, 24_000, crate::SalePayment::InFull);
    if let crate::RentalSale::Sold { personal_share_percent, .. } = &mut property.sale {
        *personal_share_percent = 20;
    }
    // Actual sale records establish the original business cost/depreciation;
    // the20% sale allocation alone does not establish depreciation history.
    property.buildings[0].sale_history =
        Some(rental_sale_history(22_000_000, 6_583_333, 6_583_333));
    property.personal_use = crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days: 200,
        personal_days: 30,
        overlap_days: 0,
        direct_expenses: 0,
    };
    let r = supported(&rental_sale_return(vec![property.clone()]));
    assert_eq!(value(&r, "rental_2_sale_building_gain"), 86_473);
    assert_eq!(value(&r, "rental_2_sale_land"), 16_160);
    assert_eq!(value(&r, "form_8949_2_1_proceeds"), 75_200);
    assert_eq!(value(&r, "form_8949_2_1_basis"), 66_000);
    assert_eq!(value(&r, "form_8949_2_1_gain"), 9_200);
    // Without the personal share the sale asks for it.
    if let crate::RentalSale::Sold { personal_share_percent, .. } = &mut property.sale {
        *personal_share_percent = 0;
    }
    let r = estimate(&rental_sale_return(vec![property]));
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_RENTAL_SALE_PERSONAL_SHARE_NEEDED"));
}

#[test]
fn building_rows_held_one_year_or_less_report_on_line_11_with_an_allocated_price() {
    // An improvement ($20,000, in service 2026-03-01) sold 2026-08-15 with the
    // building: of the $320,000 building price, $320,000 x 20/295 = $21,695 goes to it
    // (cost-weighted, one documented assumption). Depreciation 15 half-months of 660 on
    // $20,000 = $455, so its gain is $21,695 - ($20,000 - $455) = $2,150 on line 11;
    // the combined Part III building gets $298,305 against $275,000 less $65,833.
    let mut property = sold_rental(0);
    set_sale(&mut property, 400_000, 80_000, 0, crate::SalePayment::InFull);
    property.buildings.push(rental_improvement(20_000, 20260301));
    let r = supported(&rental_sale_return(vec![property]));
    assert_eq!(value(&r, "rental_2_sale_row_1_ordinary"), 2_150);
    assert_eq!(printed_line(&r, "rental_2_sale_row_1_ordinary"), (TaxForm::Form4797, "11".into()));
    assert_eq!(value(&r, "rental_2_sale_building_gain"), 89_138);
    assert!(
        r.review_notes.iter().any(|n| n.display.key == "NOTE_RENTAL_SALE_IMPROVEMENT_ALLOCATION")
    );
}

#[test]
fn f10_partial_disposition_of_a_rental_roof_is_a_loss_on_line_2_and_line_1c() {
    // Building $275,000 (no land) in service 2016-07-01, a $11,000 roof replaced
    // 2026-04-10 for nothing. Mid-month depreciation through April: 11 + 9 x 24 + 7 =
    // 234 half-months of 660 = $97,500; the roof's share 11,000 / 275,000 = $3,900;
    // adjusted basis $11,000 - $3,900 = $7,100; zero proceeds give a $7,100
    // loss. Schedule E profit $30,000 - $5,000 - $9,717 = $15,283 absorbs
    // it inside the same passive activity: $8,183 net, with no suspended loss.
    // Line 2 shows -$7,100. The building
    // depreciates 7/660 on $275,000 plus 17/660 on $264,000 in 2026.
    let mut property = rental(30_000, 5_000);
    property.buildings[0].cost = 275_000 * 100;
    property.buildings[0].land = 0;
    property.buildings[0].in_service = 20160701;
    property.buildings[0].partial =
        Some(crate::PartialDisposition { date: 20260410, cost_of_part: 11_000 * 100, proceeds: 0 });
    let r = supported(&rental_sale_return(vec![property]));
    assert_eq!(value(&r, "rental_2_building_0_partial_depreciation"), 3_900);
    assert_eq!(value(&r, "rental_2_building_0_partial_gain"), -7_100);
    assert_eq!(
        printed_line(&r, "rental_2_building_0_partial_gain"),
        (TaxForm::Form4797, "2".into())
    );
    assert_eq!(value(&r, "form4797_line1c"), 7_100);
    // 2026-10-19, D-2026-10-19-01: exact cents remain inside the owner.
    assert_eq!(line_amount(&r, "rental_2_line18_depreciation"), 971_667);
    assert_eq!(displayed_line_amount(&r, "rental_2_line18_depreciation"), 971_700);
    assert_eq!(value(&r, "rental_income"), 15_283);
    assert_eq!(value(&r, "rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "agi"), 68_183);
    assert_eq!(value(&r, "form8582_line_1a"), 15_283);
    assert_eq!(value(&r, "form8582_line_1b"), -7_100);
}

// The roof's adjusted basis is $11,000 - $3,900 = $7,100. The remaining
// building's annual depreciation is ($275,000 * 7 + $264,000 * 17) / 660
// = $9,716.666..., rounded to $9,717 independently of the property loss.
pub(in crate::tests) fn unit40_roof(rent: i64, expenses: i64, proceeds: i64) -> FederalRental {
    let mut property = rental(rent, expenses);
    property.buildings[0].cost = 275_000 * 100;
    property.buildings[0].land = 0;
    property.buildings[0].in_service = 20160701;
    property.buildings[0].partial = Some(crate::PartialDisposition {
        date: 20260410,
        cost_of_part: 11_000 * 100,
        proceeds: proceeds * 100,
    });
    property
}
pub(in crate::tests) fn unit40_return(wages: i64, property: FederalRental) -> FederalDraft {
    let mut d = rental_sale_return(vec![property]);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    d
}

#[test]
fn unit40_partial_loss_is_suspended_with_a_kept_passive_rental() {
    // Wages $200,000 eliminate the special allowance. $9,717 Schedule E
    // loss + $7,100 Form4797 loss = $16,817 suspended, not a direct tax loss.
    for active in [true, false] {
        let mut property = unit40_roof(0, 0, 0);
        if !active {
            property.participation = crate::RentalParticipation::NotActive;
        }
        let r = supported(&unit40_return(200_000, property));
        for (id, expected) in [
            ("rental_income", 0),
            ("form4797_line8", 0),
            ("rental_2_building_0_partial_gain", 0),
            ("rental_2_line22_allowed_loss", 0),
            ("rental_2_suspended_loss", 16_817),
            ("amt_rental_2_suspended_loss", 16_817),
            ("form8582_part_ix_2_ordinary_unallowed", 9_717),
            ("form8582_part_ix_2_section1231_unallowed", 7_100),
            ("agi", 200_000),
        ] {
            assert_eq!(value(&r, id), expected, "{id}, active={active}");
        }
        assert!(!r.lines.iter().any(|row| row.id == "form4797_line1c"));
        assert_eq!(
            value(&r, if active { "form8582_line_1b" } else { "form8582_line_2b" }),
            -16_817
        );
    }
}

#[test]
fn unit40_partial_loss_and_schedule_e_share_the_special_allowance() {
    // MAGI $140,000: ($150,000-$140,000)/2 = $5,000 allowed. Loss $16,817
    // leaves $11,817 suspended. Part IX: 11,817*9,717/16,817 = 6,827.905...,
    // rounded $6,828 ordinary; remainder $4,989 property. Allowed $2,889 on
    // Schedule E and $2,111 on Form4797, total $5,000, AGI $135,000.
    let r = supported(&unit40_return(140_000, unit40_roof(0, 0, 0)));
    for (id, expected) in [
        ("rental_magi", 140_000),
        ("form8582_line_9", 5_000),
        ("rental_income", -2_889),
        ("rental_2_line22_allowed_loss", -2_889),
        ("rental_2_suspended_loss", 11_817),
        ("form8582_part_ix_2_ordinary_unallowed", 6_828),
        ("form8582_part_ix_2_section1231_unallowed", 4_989),
        ("rental_2_building_0_partial_gain", -2_111),
        ("form4797_line8", -2_111),
        ("form4797_line1c", 2_111),
        ("agi", 135_000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
}

#[test]
fn unit40_partial_gain_offsets_its_own_passive_operating_loss() {
    // Proceeds $12,100 less $7,100 adjusted basis = $5,000 gain. It offsets
    // $5,000 of the $9,717 Schedule E loss, leaving $4,717 suspended.
    // Income and loss stay in separate Part V columns; AGI remains $200,000.
    let mut property = unit40_roof(0, 0, 12_100);
    property.participation = crate::RentalParticipation::NotActive;
    let r = supported(&unit40_return(200_000, property));
    for (id, expected) in [
        ("rental_income", -5_000),
        ("rental_2_line22_allowed_loss", -5_000),
        ("rental_2_suspended_loss", 4_717),
        ("rental_2_building_0_partial_gain", 5_000),
        ("form4797_line1b", 5_000),
        ("form8582_line_2a", 5_000),
        ("form8582_line_2b", -9_717),
        ("form8582_line_2d", -4_717),
        ("agi", 200_000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
}

#[test]
fn unit40_prior_ordinary_passive_loss_retains_its_own_allowed_share() {
    // Current E loss $9,717 plus $5,000 prior E loss and $7,100 property
    // loss = $21,817. Special allowance $5,000 leaves $16,817 suspended.
    // E share round(16,817*14,717/21,817)=$11,344, property $5,473;
    // allowed E $3,373, property $1,627. Total deduction remains $5,000.
    let mut property = unit40_roof(0, 0, 0);
    property.history.regular = 5_000 * 100;
    property.history.amt = 5_000 * 100;
    let r = supported(&unit40_return(140_000, property));
    for (id, expected) in [
        ("rental_income", -3_373),
        ("rental_2_line22_allowed_loss", -3_373),
        ("rental_2_suspended_loss", 16_817),
        ("form8582_line_1c", -5_000),
        ("rental_2_building_0_partial_gain", -1_627),
        ("form4797_line1c", 1_627),
        ("agi", 135_000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
}

#[test]
fn unit40_short_held_partial_loss_stays_in_part_ii_after_passive_limits() {
    // Roof basis $11,000 less round($11,000 * 8 / 660)=$133 depreciation
    // leaves a $10,867 short-held loss. Together with $9,717 annual operating
    // depreciation it suspends $20,584 at $200,000 MAGI. No ordinary tax loss.
    let mut property = unit40_roof(0, 0, 0);
    property.buildings[0].in_service = 20251201;
    property.participation = crate::RentalParticipation::NotActive;
    let r = supported(&unit40_return(200_000, property));
    for (id, expected) in [
        ("rental_2_building_0_partial_depreciation", 133),
        ("rental_2_building_0_partial_gain", 0),
        ("form8582_part_ix_2_short_property_unallowed", 10_867),
        ("rental_2_suspended_loss", 20_584),
        ("rental_income", 0),
        ("agi", 200_000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
    assert_eq!(
        printed_line(&r, "rental_2_building_0_partial_gain"),
        (TaxForm::Form4797, "11".into())
    );
}

#[test]
fn unit40_component_gain_remains_when_the_same_part_has_suspended_loss() {
    // Two components: loss $7,100 and gain $5,000, both Part I. The activity
    // suspends the $2,100 property net and round(2 * $9,716.67) = $19,433
    // operating loss; building cents are added before rounding. The gain
    // remains $5,000; only $5,000 of the other component loss is reportable.
    let mut property = unit40_roof(0, 0, 0);
    let mut second = property.buildings[0].clone();
    second.partial.as_mut().unwrap().proceeds = 12_100 * 100;
    property.buildings.push(second);
    property.participation = crate::RentalParticipation::NotActive;
    let r = supported(&unit40_return(200_000, property));
    for (id, expected) in [
        ("rental_2_building_0_partial_gain", -5_000),
        ("rental_2_building_1_partial_gain", 5_000),
        ("form4797_line1b", 5_000),
        ("form4797_line1c", 5_000),
        ("rental_2_suspended_loss", 21_533),
        ("rental_income", 0),
        ("form4797_line8", 0),
        ("agi", 200_000),
    ] {
        assert_eq!(value(&r, id), expected, "{id}");
    }
}

#[test]
fn unit40_amt_uses_its_own_retained_loss_with_the_same_regular_magi() {
    // Regular prior E loss $5,000: ordinary/property cuts $11,344/$5,473,
    // allowed $3,373/$1,627. AMT prior loss zero: cuts $6,828/$4,989,
    // allowed $2,889/$2,111. Each system uses the same $5,000 allowance.
    let mut property = unit40_roof(0, 0, 0);
    property.history.regular = 5_000 * 100;
    let d = unit40_return(140_000, property);
    for (amt, ordinary, loss, carry) in
        [(false, -3_373, -1_627, 16_817), (true, -2_889, -2_111, 11_817)]
    {
        let mut r = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 140_000, &mut r, amt);
        assert!(r.issues.is_empty(), "{:?}", r.issues);
        assert_eq!(pool.income, ordinary);
        assert_eq!(pool.passive_loss, carry);
        let sources = crate::asset_sale::allowed_rental_sources(
            crate::related_forms::RelatedFormContext::original(
                &d,
                &crate::related_forms::effective_claimants(&d),
            ),
            &pool,
            amt,
        )
        .unwrap();
        assert_eq!(sources.iter().find(|(id, _)| *id == 2).unwrap().1.section1231, loss);
    }
}

#[test]
fn print_schedule_d_carries_and_character_nets_match_the_original_return() {
    // Actual prior short loss4000 and long loss2000 less the3000 previously
    // deducted leave1000 short/2000 long. Current gains5000/10000 yield
    // printed4000/8000/12000, with no second subtraction or 2027 carry.
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![PriorCapitalReturn {
        research_parts: None,
        injured_owners: None,
        injured_niit_owners: None,
        owners: PriorReturnOwners::Taxpayer,
        status: FederalStatus::Single,
        agi: 6_200_000,
        deductions: 1_200_000,
        senior_deduction: 0,
        short: -400_000,
        long: -200_000,
        deducted_loss: 300_000,
        section_1256: 0,
        amt: None,
        origins: vec![],
        ordinary_history: true,
        investment_parts: Some(InvestmentCapitalParts { short: 100_000, long: 200_000, amt: None }),
        niit_difference: Some(0),
    }];
    add(&mut d, income(ordinary_investment_sale(15000, 10000, GainTerm::Short)));
    add(&mut d, income(ordinary_investment_sale(40000, 30000, GainTerm::Long)));
    let r = supported(&d);
    for (number, expected) in [
        ("5", 0),
        ("6", -1000),
        ("7", 4000),
        ("12", 0),
        ("13", 0),
        ("14", -2000),
        ("15", 8000),
        ("16", 12000),
        ("18", 0),
        ("19", 0),
    ] {
        assert_eq!(value(&r, &alloc::format!("schedule_d_line_{number}")), expected);
    }
    assert_eq!(return_line(&r, "7a"), 1_200_000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn print_schedule_d_loss_summary_preserves_single_and_separate_limits() {
    for (status, expected) in [(FederalStatus::Single, -3000), (FederalStatus::Separate, -1500)] {
        let mut d = complete(status);
        add(&mut d, income(ordinary_investment_sale(1000, 9000, GainTerm::Long)));
        let r = supported(&d);
        assert_eq!(value(&r, "schedule_d_line_7"), 0);
        assert_eq!(value(&r, "schedule_d_line_15"), -8000);
        assert_eq!(value(&r, "schedule_d_line_16"), -8000);
        assert_eq!(value(&r, "schedule_d_line_21"), expected);
        assert_eq!(return_line(&r, "7a"), expected * 100);
        assert!(
            !r.lines
                .iter()
                .any(|row| row.id == "schedule_d_line_18" || row.id == "schedule_d_line_19")
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn print_schedule_d_uses_full_worksheet_gains_before_tax_rate_caps() {
    // The fund reports10000, including3000 collectibles and5000 unrecaptured.
    // An ordinary long loss4000 leaves6000 net, but worksheets18/19 remain
    // 3000/5000. The preferential tax owner caps28%-rate gain at1000 after
    // allocating5000 to the25% pool; printing that1000 on18 would be wrong.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(mixed_gain_distribution()));
    add(&mut d, income(ordinary_investment_sale(0, 4000, GainTerm::Long)));
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_d_line_13"), 10000);
    assert_eq!(value(&r, "schedule_d_line_15"), 6000);
    assert_eq!(value(&r, "schedule_d_line_16"), 6000);
    assert_eq!(value(&r, "schedule_d_line_18"), 3000);
    assert_eq!(value(&r, "schedule_d_line_19"), 5000);
    assert_eq!(value(&r, "rate_28_gain"), 1000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 5000);
    assert_eq!(return_line(&r, "7a"), 600000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn print_schedule_d_k1_sources_stay_separate_from_sales_and_distributions() {
    let mut d = complete(FederalStatus::Single);
    let mut source = complete_ordinary_scorp(0, true);
    source.capital = Some(k1_capital(3000, 5000));
    source.income = vec![
        ScorpIncome { code: ScorpIncomeCode::NonportfolioShort, amount: 100000 },
        ScorpIncome { code: ScorpIncomeCode::NonportfolioLong, amount: 200000 },
    ];
    source.amt = Some(amt_records(&source));
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    add(&mut d, income(ordinary_investment_sale(5000, 4000, GainTerm::Short)));
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                capital_distributions: 200000,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    for (number, expected) in
        [("5", 4000), ("7", 5000), ("12", 7000), ("13", 2000), ("15", 9000), ("16", 14000)]
    {
        assert_eq!(value(&r, &alloc::format!("schedule_d_line_{number}")), expected);
    }
    assert_eq!(return_line(&r, "7a"), 1400000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    let reordered = supported(&d);
    for number in ["5", "7", "12", "13", "15", "16"] {
        let id = alloc::format!("schedule_d_line_{number}");
        assert_eq!(value(&reordered, &id), value(&r, &id));
    }
}

#[test]
fn print_schedule_d_source_cents_round_once_after_component_partition() {
    // Four genuine49-cent gains occupy different source cells. The actual
    // netting owner rounds98 cents to1 dollar in each holding term. Printing
    // each source as zero would erase all four contributions from the footing.
    let mut d = complete(FederalStatus::Single);
    for (term, reporting) in [
        (GainTerm::Short, BasisReporting::ReportedToIrs),
        (GainTerm::Short, BasisReporting::NotReported),
        (GainTerm::Long, BasisReporting::ReportedToIrs),
        (GainTerm::Long, BasisReporting::NoForm1099B),
    ] {
        let mut sale = ordinary_investment_sale(0, 0, term);
        let FederalIncome::Sale { data } = &mut sale else { unreachable!() };
        data.form.proceeds = 49;
        data.form.reporting = reporting;
        add(&mut d, income(sale));
    }
    let r = supported(&d);
    for id in ["capital_gain_1a", "capital_gain_2", "capital_gain_8a", "capital_gain_10"] {
        assert_eq!(line_amount(&r, id), 0);
        let precise =
            r.lines.iter().find(|row| row.id == alloc::format!("{id}_unrounded")).unwrap();
        assert_eq!(precise.amount, 49);
        assert_eq!(precise.form_line.as_ref().unwrap().kind, crate::FormLineKind::Subrow);
    }
    assert_eq!(value(&r, "schedule_d_line_7"), 1);
    assert_eq!(value(&r, "schedule_d_line_15"), 1);
    assert_eq!(return_line(&r, "7a"), 200);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn print_schedule_d_summary_source_cells_keep_cents_beside_whole_dollars() {
    // A -85-cent K-1 loss prints -$1, like the
    // settled net. The second loss and fund distribution offset exactly;
    // 49 + 49 cents instead settles at $1 without rounding either source.
    for (sources, printed, short, long) in [
        ([-85, -85, 85], [-100, -100, 100], -1, 0),
        ([-100, -100, 100], [-100, -100, 100], -1, 0),
        ([49, 49, 49], [0, 0, 0], 0, 1),
    ] {
        let settled = crate::capital::CapitalAmounts { short, long, ..Default::default() }
            .calculate([0; 2], false, false);
        let mut r = FederalResult::empty(1);
        settled.emit_summary([0; 2], [sources[0], sources[1]], sources[2], &mut r);
        for ((number, cents), amount) in ["5", "12", "13"].into_iter().zip(sources).zip(printed) {
            let id = alloc::format!("schedule_d_line_{number}");
            let row = r.lines.iter().find(|row| row.id == id).unwrap();
            assert_eq!(row.amount, amount);
            assert_eq!(row.kind, LineKind::Money);
            assert_eq!(row.form_line, Some(FormLine::new(TaxForm::ScheduleD, number)));
            let precise = r.lines.iter().find(|row| row.id == alloc::format!("{id}_unrounded"));
            if cents % 100 == 0 {
                assert!(precise.is_none(), "whole-dollar source needs no supporting subrow");
            } else {
                let precise = precise.unwrap();
                assert_eq!(precise.amount, cents);
                assert_eq!(precise.kind, LineKind::Money);
                assert_eq!(precise.form_line, Some(FormLine::subrow(TaxForm::ScheduleD, number)));
                assert!(crate::federal_model::IdKeys::new(&precise.id).registered());
            }
        }
        for (number, amount) in
            [("7", short * 100), ("15", long * 100), ("16", (short + long) * 100)]
        {
            assert_eq!(line_amount(&r, &alloc::format!("schedule_d_line_{number}")), amount);
        }
        if short + long < 0 {
            assert_eq!(line_amount(&r, "schedule_d_line_21"), -100);
        }
        let printed_order: Vec<_> = r
            .lines
            .iter()
            .filter_map(|row| {
                row.form_line
                    .as_ref()
                    .filter(|form| form.kind == crate::FormLineKind::Line)
                    .map(|form| form.line.as_str())
            })
            .collect();
        assert_eq!(&printed_order[..8], &["5", "6", "7", "12", "13", "14", "15", "16"]);
    }
}

#[test]
fn print_written_note_rows_reuse_restored_basis_and_dated_holding_terms() {
    let mut source = mixed_note_repayments();
    source.ordinary = 200000;
    source.qbi_statement.as_mut().unwrap().income = 200000;
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain_3"), 400);
    assert_eq!(value(&r, "capital_gain_10"), 600);
    assert_eq!(value(&r, "schedule_d_line_5"), 0);
    assert_eq!(value(&r, "schedule_d_line_12"), 0);
    assert_eq!(value(&r, "capital_gain_3_proceeds"), 2000);
    assert_eq!(value(&r, "capital_gain_3_basis"), 1600);
    assert_eq!(value(&r, "capital_gain_10_proceeds"), 3000);
    assert_eq!(value(&r, "capital_gain_10_basis"), 2400);
    assert_eq!(return_line(&r, "7a"), 100000);
    assert_eq!(value(&r, "agi"), 63000);
    let actual: Vec<_> = r
        .lines
        .iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form8949))
        .collect();
    assert_eq!(actual.len(), 8);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Scorp { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    let ScorpNoteHolding::Dated { payments, .. } = &mut data.loans[0].repayment_term else {
        unreachable!()
    };
    payments.reverse();
    drop(data);
    let reversed = supported(&d);
    assert_eq!(reversed.return_lines, r.return_lines);
    for id in [
        "capital_gain_3",
        "capital_gain_10",
        "capital_gain_3_proceeds",
        "capital_gain_3_basis",
        "capital_gain_10_proceeds",
        "capital_gain_10_basis",
    ] {
        assert_eq!(value(&reversed, id), value(&r, id));
    }
}

#[test]
fn print_written_note_uses_the_existing_separate_property_gain_owner() {
    let mut d = community_w2_draft(6000000, 0, 0, 0);
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
    let mut entry =
        income(FederalIncome::Scorp { data: scorp_source(repaid_written_note(GainTerm::Long)) });
    entry.community = CommunityShare::Separate;
    add(&mut d, entry);
    let r = supported(&d);
    // Texas separate-property ordinary income may divide, but this actual
    // separate property's gain belongs wholly to its owner, as before PRINT.
    assert_eq!(value(&r, "schedule_d_line_12"), 0);
    assert_eq!(value(&r, "capital_gain_10"), 2000);
    assert_eq!(value(&r, "capital_gain_10_proceeds"), 5000);
    assert_eq!(value(&r, "capital_gain_10_basis"), 3000);
    assert_eq!(return_line(&r, "7a"), 200000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn print_schedule_d_summary_choices_follow_the_actual_branch_and_review_forms() {
    for (term, gain, investment, expected) in [
        (GainTerm::Long, 2000, false, &["17_yes", "20_yes"][..]),
        (GainTerm::Long, 2000, true, &["17_yes", "20_no"][..]),
        (GainTerm::Short, 2000, false, &["17_no", "22_no"][..]),
        (GainTerm::Long, -2000, false, &["22_no"][..]),
        (GainTerm::Long, 0, false, &["22_no"][..]),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(ordinary_investment_sale(4000, 4000 - gain, term)));
        if investment {
            investment_borrowing(&mut d, 3000, 0, 0);
        }
        let r = supported(&d);
        let actual: Vec<_> =
            r.lines.iter().filter_map(|row| row.id.strip_prefix("schedule_d_choice_")).collect();
        assert_eq!(actual, expected);
        let review = crate::draft_return(&r);
        for tail in expected {
            let number = tail.split('_').next().unwrap();
            assert!(review.iter().any(|row| row.money.is_none()
                && row.form_line == Some(FormLine::new(TaxForm::ScheduleD, number))));
        }
    }
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                ordinary: 100000,
                qualified: 100000,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert!(
        !r.lines.iter().any(|row| row.id.starts_with("schedule_d_choice_")),
        "Qualified dividends alone do not introduce a ScheduleD review"
    );
    add(&mut d, income(ordinary_investment_sale(1000, 3000, GainTerm::Short)));
    let r = supported(&d);
    assert!(r.lines.iter().any(|row| row.id == "schedule_d_choice_22_yes"));
}
