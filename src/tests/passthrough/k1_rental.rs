// Schedule K-1 boxes 2 and 3 (unit k1-rentals): the fixtures of the
// k1-rentals map, by hand. Every amount is synthetic. Single filer with
// $60,000 wages; 2026 standard deduction $16,100 and the single brackets of
// Rev. Proc. 2025-32 (10% to $12,400, 12% to $50,400, 22% to $105,700); the
// tax table taxes the middle of each $50 row below $100,000. Form 8582 line
// numbers are the 2026 draft's; MAGI is line 6 (i8582 2025).
use super::*;
use alloc::format;

// A continuing general partner without material participation in box 1
// (which is zero): opening basis $50,000 and a 2025 Form 6198 amount at risk
// of $50,000, so the basis and at-risk limits allow these rental losses.
fn partner(real_estate: i64, other: i64, participation: K1RentalParticipation) -> PartnerK1 {
    let mut data = ordinary_partner_source(0, false);
    data.regular.basis.opening = 5_000_000;
    data.regular.risk.start = ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
    data.regular.rentals = K1RentalAmounts {
        real_estate_property: Default::default(),
        other_property: Default::default(),
        real_estate: real_estate * 100,
        other: other * 100,
        ..Default::default()
    };
    data.rental_facts.participation = participation;
    data.alternative = Some(data.regular.clone());
    data
}

fn with_partner(wages: i64, data: PartnerK1) -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 650_000);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Partnership { data }));
    (d, id)
}

fn rental_line(r: &FederalResult, kind: &str, id: i64, activity: &str, part: &str) -> i64 {
    value(r, &format!("{kind}_{id}_{activity}_{part}"))
}

// The regular-tax K-1 ledger the passive pool completes (unit B8).
fn pool_ledger(d: &FederalDraft, id: i64) -> crate::k1_loss::LossLedger {
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(d, 6_000_000, &mut r, false);
    pool.k1_ledgers.iter().find(|(k, _)| *k == id).map(|(_, ledger)| *ledger).unwrap()
}

fn refused_with(r: &FederalResult, key: &str) -> bool {
    r.state == ResultState::Unsupported && r.issues.iter().any(|i| i.display.key == key)
}

// Approved B9-7 F4: all amounts and expected footings come from the
// handoff, independently of the passive-pool implementation.
pub(in crate::tests) fn rental_property_sale(full: bool, scorp: bool) -> (FederalDraft, i64) {
    let mut source = partner(-2_000, 0, K1RentalParticipation::NotActive);
    source.participation = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    source.regular.rentals.real_estate_carryover = 900_000;
    source.regular.rentals.real_estate_property =
        K1RentalProperty { net: 600_000, unrecaptured: 400_000, passive_loss: 0 };
    if full {
        source.rental_facts.disposed.push(K1RentalActivity::RealEstate);
    }
    source.alternative = Some(source.regular.clone());
    if !scorp {
        let (mut d, id) = with_partner(70_000, source);
        d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        return (d, id);
    }
    let mut shareholder = complete_ordinary_scorp(0, false);
    shareholder.opening = 5_000_000;
    shareholder.risk.start = ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
    shareholder.rentals = source.regular.rentals;
    shareholder.rental_facts = source.rental_facts;
    shareholder.amt = Some(amt_records(&shareholder));
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 7_000_000, 650_000);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: shareholder }));
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    (d, id)
}

#[test]
fn k1_rental_property_f4_full_sale_releases_ordinary_history_without_repeating_gain() {
    for scorp in [false, true] {
        let (d, id) = rental_property_sale(true, scorp);
        let r = supported(&d);
        let kind = if scorp { "scorp" } else { "partnership" };
        for prefix in ["", "amt_"] {
            assert_eq!(
                rental_line(&r, &format!("{prefix}{kind}"), id, "rental_real_estate", "net"),
                -11_000
            );
            assert_eq!(
                rental_line(&r, &format!("{prefix}{kind}"), id, "rental_real_estate", "suspended"),
                0
            );
        }
        assert_eq!(value(&r, "form4797_line8"), 6_000);
        assert_eq!(value(&r, "form4797_line9"), 0);
        assert_eq!(value(&r, "form4797_line10"), 6_000);
        assert_eq!(value(&r, "rental_magi"), 65_000);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(return_line(&r, "11a / 11b"), 6_500_000);
        assert_eq!(return_line(&r, "15"), 4_890_000);
        assert_eq!(
            crate::kernel::ratio::dollars(
                pool_ledger(&d, id).passive_suspended
                    [crate::k1_loss::K1LossClass::RentalRealEstate]
                    .get()
            ),
            0
        );
        assert!(
            !r.lines.iter().any(|line| line.id.starts_with("form8582_part_")
                && line.id.contains(&format!("_{id}_")))
        );
        let mut scratch = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&d, 7_000_000, &mut scratch, false);
        let worksheet = crate::section1231::worksheet(&d, &pool, false).unwrap().unwrap();
        assert_eq!(worksheet.unrecaptured, 4_000);
    }
}

#[test]
fn k1_rental_property_f4b_partial_sale_keeps_ordinary_history_passive() {
    for scorp in [false, true] {
        let (d, id) = rental_property_sale(false, scorp);
        let r = supported(&d);
        let kind = if scorp { "scorp" } else { "partnership" };
        assert_eq!(rental_line(&r, kind, id, "rental_real_estate", "net"), -6_000);
        assert_eq!(rental_line(&r, kind, id, "rental_real_estate", "suspended"), 5_000);
        assert_eq!(value(&r, "form4797_line10"), 6_000);
        assert_eq!(return_line(&r, "11a / 11b"), 7_000_000);
        assert_eq!(
            crate::kernel::ratio::dollars(
                pool_ledger(&d, id).passive_suspended
                    [crate::k1_loss::K1LossClass::RentalRealEstate]
                    .get()
            ),
            5_000
        );
    }
}

fn amend_rental_sale(
    d: &mut FederalDraft,
    change: impl FnOnce(&mut K1RentalAmounts, &mut K1RentalFacts),
) {
    match &mut d.income.last_mut().unwrap().income {
        FederalIncome::Partnership { data } => {
            change(&mut data.regular.rentals, &mut data.rental_facts);
            data.alternative = Some(data.regular.clone());
        }
        FederalIncome::Scorp { data } => {
            change(&mut data.rentals, &mut data.rental_facts);
            data.amt = Some(amt_records(data));
        }
        _ => panic!("expected a K-1"),
    }
}

#[test]
fn k1_rental_property_f4c_special_disposition_remains_a_named_refusal() {
    for scorp in [false, true] {
        let (mut d, _) = rental_property_sale(false, scorp);
        let key = match &mut d.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => {
                data.unentered.push(PartnerK1Item::RentalDisposition);
                "ISSUE_PARTNER_K1_RENTAL_DISPOSITION"
            }
            FederalIncome::Scorp { data } => {
                data.unentered.push(ScorpK1Item::RentalDisposition);
                "ISSUE_SCORP_K1_RENTAL_DISPOSITION"
            }
            _ => unreachable!(),
        };
        assert!(refused_with(&estimate(&d), key));
    }
}

#[test]
fn k1_rental_property_carryovers_keep_the_two_loss_characters_separate() {
    // Ordinary loss 2,000 + prior 9,000; property loss 3,000 + prior 1,000.
    // No passive income: both stay suspended. A full sale releases all
    // 15,000, with 11,000 on E and 4,000 on 4797, never a capital loss.
    for scorp in [false, true] {
        for full in [false, true] {
            let (mut d, id) = rental_property_sale(full, scorp);
            amend_rental_sale(&mut d, |amounts, _| {
                amounts.real_estate_property =
                    K1RentalProperty { net: -300_000, unrecaptured: 0, passive_loss: 100_000 };
            });
            let r = supported(&d);
            let kind = if scorp { "scorp" } else { "partnership" };
            assert_eq!(
                rental_line(&r, kind, id, "rental_real_estate", "net"),
                if full { -11_000 } else { 0 }
            );
            assert_eq!(
                rental_line(&r, kind, id, "rental_real_estate", "suspended"),
                if full { 0 } else { 11_000 }
            );
            assert_eq!(
                rental_line(&r, kind, id, "rental_real_estate", "section1231_suspended"),
                if full { 0 } else { 4_000 }
            );
            assert_eq!(value(&r, "form4797_line12"), if full { -4_000 } else { 0 });
            assert_eq!(return_line(&r, "11a / 11b"), if full { 5_500_000 } else { 7_000_000 });
            let ledger = pool_ledger(&d, id);
            assert_eq!(
                crate::kernel::ratio::dollars(
                    ledger.passive_suspended[crate::k1_loss::K1LossClass::Section1231].get()
                ),
                if full { 0 } else { 4_000 }
            );
            assert_eq!(estimate(&restart(d.clone()).draft), r);
        }
    }
}

#[test]
fn k1_rental_property_basis_and_risk_suspensions_require_activity_history() {
    for scorp in [false, true] {
        for basis_limit in [false, true] {
            let (mut d, _) = rental_property_sale(false, scorp);
            amend_rental_sale(&mut d, |amounts, _| {
                amounts.real_estate = 0;
                amounts.real_estate_carryover = 0;
                amounts.real_estate_property =
                    K1RentalProperty { net: -1_000_000, unrecaptured: 0, passive_loss: 0 };
            });
            match &mut d.income.last_mut().unwrap().income {
                FederalIncome::Partnership { data } => {
                    if basis_limit {
                        data.regular.basis.opening = 0;
                    } else {
                        data.regular.risk.start =
                            ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
                    }
                    data.alternative = Some(data.regular.clone());
                }
                FederalIncome::Scorp { data } => {
                    if basis_limit {
                        data.opening = 0;
                    } else {
                        data.risk.start =
                            ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
                    }
                    data.amt = Some(amt_records(data));
                }
                _ => unreachable!(),
            }
            assert!(refused_with(&estimate(&d), "ISSUE_K1_RENTAL_PROPERTY_LIMITED"));
        }
    }
}

#[test]
fn k1_rental_property_duplicate_full_sale_is_invalid_at_both_page_and_record() {
    for scorp in [false, true] {
        let (mut d, _) = rental_property_sale(true, scorp);
        amend_rental_sale(&mut d, |_, facts| facts.disposed.push(K1RentalActivity::RealEstate));
        match &d.income.last().unwrap().income {
            FederalIncome::Partnership { data } => {
                assert!(!data.valid());
                assert_eq!(
                    crate::K1Partnership::from_record("Synthetic partnership", data)
                        .unwrap()
                        .rule(),
                    Some("K1_RENTAL_DISPOSED_TWICE")
                );
            }
            FederalIncome::Scorp { data } => {
                assert!(!data.valid());
                assert_eq!(
                    crate::K1Scorp::from_record("Synthetic corporation", data).unwrap().rule(),
                    Some("K1_RENTAL_DISPOSED_TWICE")
                );
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn k1_rental_property_full_sale_gain_offsets_other_pal_but_former_passive_gain_does_not() {
    for scorp in [false, true] {
        for professional in [false, true] {
            let (mut d, id) = rental_property_sale(true, scorp);
            amend_rental_sale(&mut d, |amounts, facts| {
                amounts.real_estate_carryover = 100_000;
                amounts.other = -800_000;
                if professional {
                    facts.participation = K1RentalParticipation::Professional;
                }
            });
            if professional {
                make_professional(&mut d, Owner::Taxpayer, true);
            }
            let r = supported(&d);
            let kind = if scorp { "scorp" } else { "partnership" };
            assert_eq!(rental_line(&r, kind, id, "rental_real_estate", "net"), -3_000);
            assert_eq!(
                rental_line(&r, kind, id, "other_rental", "net"),
                if professional { 0 } else { -3_000 }
            );
            assert_eq!(
                rental_line(&r, kind, id, "other_rental", "suspended"),
                if professional { 8_000 } else { 5_000 }
            );
            assert_eq!(value(&r, "form4797_line10"), 6_000);
            assert_eq!(
                return_line(&r, "11a / 11b"),
                if professional { 7_300_000 } else { 7_000_000 }
            );
        }
    }
}

#[test]
fn k1_rental_property_qbi_uses_return_wide_character_and_one_activity() {
    for scorp in [false, true] {
        for recaptured in [false, true] {
            let (mut d, _) = rental_property_sale(false, scorp);
            amend_rental_sale(&mut d, |amounts, facts| {
                amounts.real_estate = 500_000;
                amounts.real_estate_carryover = 0;
                facts.real_estate_qbi = Some(K1RentalQbi { wages: 200_000, property: 1_000_000 });
            });
            if recaptured {
                d.screening.as_mut().unwrap().section1231 =
                    Some(section1231_history(&[(2025, 6_000, 6_000)], &[(2025, 6_000, 6_000)]));
            }
            let r = supported(&d);
            assert_eq!(value(&r, "qbi_deduction"), if recaptured { 2_200 } else { 1_000 });
            assert_eq!(return_line(&r, "11a / 11b"), 8_100_000);
            assert_eq!(value(&r, "form4797_line13"), if recaptured { 6_000 } else { 0 });
            let mut scratch = estimate(&complete(FederalStatus::Single));
            let pool = crate::extended_rules::rentals_for(&d, 7_000_000, &mut scratch, false);
            assert_eq!(pool.activities.len(), 1);
            assert_eq!(pool.activities[0].wages, 2_000);
            assert_eq!(pool.activities[0].property, 10_000);
            assert_eq!(
                crate::section1231::rental_property_qbi(&d, &pool).unwrap().current,
                vec![(
                    0,
                    if recaptured { 6_000 } else { 0 },
                    Some(if recaptured { 600_000 } else { 0 })
                )]
            );
        }
    }
}

#[test]
fn k1_rental_property_professional_niit_exclusion_requires_the_recorded_safe_harbor() {
    for scorp in [false, true] {
        for safe_harbor in [false, true] {
            let (mut d, _) = rental_property_sale(true, scorp);
            d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 650_000);
            amend_rental_sale(&mut d, |amounts, facts| {
                amounts.real_estate = 500_000;
                amounts.real_estate_carryover = 0;
                facts.participation = K1RentalParticipation::Professional;
            });
            make_professional(&mut d, Owner::Taxpayer, safe_harbor);
            let r = supported(&d);
            // NII 5,000 ordinary + 6,000 capital, or zero when the safe
            // harbor applies; 3.8% x 11,000 = 418. MAGI is above 200,000.
            assert_eq!(value(&r, "net_investment_income_tax"), if safe_harbor { 0 } else { 418 });
        }
    }
}

// i8960 (2025), "Disposition of Entire Interest": section469(f) own-income
// losses follow included income, while section469(g) losses are additional
// NII deductions. Preserve ordinary/property character in both cases.
#[test]
fn k1_rental_property_professional_full_sale_keeps_469g_prior_losses_in_niit() {
    for scorp in [false, true] {
        for property_history in [false, true] {
            for gain in [false, true] {
                for safe_harbor in [false, true] {
                    let (mut d, _) = rental_property_sale(true, scorp);
                    d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 650_000);
                    amend_rental_sale(&mut d, |amounts, facts| {
                        facts.participation = K1RentalParticipation::Professional;
                        if property_history {
                            amounts.real_estate_carryover = 0;
                            amounts.real_estate_property.passive_loss = 900_000;
                        }
                        if !gain {
                            amounts.real_estate_property.net = -300_000;
                            amounts.real_estate_property.unrecaptured = 0;
                        }
                    });
                    make_professional(&mut d, Owner::Taxpayer, safe_harbor);
                    add(&mut d, income(savings_income(20_000, 0)));
                    let r = supported(&d);
                    // Gain case: current net4,000 absorbs prior4,000 under
                    // 469(f); remaining5,000 is 469(g), so NII20,000-5,000.
                    // Loss case: all9,000 prior is 469(g); the safe harbor
                    // excludes current -5,000, leaving NII11,000 rather than6,000.
                    let expected = if gain {
                        570
                    } else if safe_harbor {
                        418
                    } else {
                        228
                    };
                    assert_eq!(
                        value(&r, "net_investment_income_tax"),
                        expected,
                        "scorp={scorp}, property_history={property_history}, gain={gain}, safe_harbor={safe_harbor}"
                    );
                }
            }
        }
    }
}

#[test]
fn k1_rental_property_two_activities_have_unique_character_rows_and_carryovers() {
    for scorp in [false, true] {
        let (mut d, id) = rental_property_sale(false, scorp);
        amend_rental_sale(&mut d, |amounts, _| {
            amounts.real_estate_property =
                K1RentalProperty { net: -300_000, unrecaptured: 0, passive_loss: 100_000 };
            amounts.other = -100_000;
            amounts.other_property =
                K1RentalProperty { net: -200_000, unrecaptured: 0, passive_loss: 300_000 };
        });
        let r = supported(&d);
        for (activity, ordinary, property) in
            [("rental_real_estate", 11_000, 4_000), ("other_rental", 1_000, 5_000)]
        {
            assert_eq!(
                value(&r, &format!("form8582_part_ix_{id}_{activity}_ordinary_unallowed")),
                ordinary
            );
            assert_eq!(
                value(&r, &format!("form8582_part_ix_{id}_{activity}_section1231_unallowed")),
                property
            );
        }
        let ids: alloc::collections::BTreeSet<_> = r.lines.iter().map(|line| &line.id).collect();
        assert_eq!(ids.len(), r.lines.len());
        assert_eq!(return_line(&r, "11a / 11b"), 7_000_000);
    }
}

#[test]
fn k1_rental_property_business_and_rental_use_one_source_with_distinct_qbi_classes() {
    for scorp in [false, true] {
        let (mut d, id) = rental_property_sale(false, scorp);
        amend_rental_sale(&mut d, |amounts, facts| {
            amounts.real_estate = 500_000;
            amounts.real_estate_carryover = 0;
            amounts.real_estate_property.unrecaptured = 0;
            facts.real_estate_qbi = Some(K1RentalQbi { wages: 200_000, property: 1_000_000 });
        });
        match &mut d.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => {
                data.regular.section1231 = Some(k1_section1231(10_000, 10_000));
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                data.section1231 = Some(k1_section1231(10_000, 10_000));
                data.amt = Some(amt_records(data));
            }
            _ => unreachable!(),
        }
        d.screening.as_mut().unwrap().section1231 =
            Some(section1231_history(&[(2025, 10_000, 10_000)], &[(2025, 10_000, 10_000)]));
        let r = supported(&d);
        assert_eq!(value(&r, "form4797_line8"), 16_000);
        assert_eq!(value(&r, "form4797_line13"), 10_000);
        assert_eq!(value(&r, "form4797_line10"), 6_000);
        assert_eq!(value(&r, "qbi_deduction"), 1_000);
        assert_eq!(return_line(&r, "11a / 11b"), 9_100_000);
        let mut scratch = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&d, 7_000_000, &mut scratch, false);
        assert_eq!(pool.k1_section1231.len(), 1);
        assert_eq!(pool.k1_section1231[0].id, id);
        assert_eq!(pool.k1_section1231[0].net, 16_000);
        assert_eq!(pool.k1_section1231[0].business().net, 10_000);
        assert_eq!(pool.k1_section1231[0].rentals[0].net, 6_000);

        // The same source classes also determine NIIT: nonpassive box 1
        // property is excluded; passive rental ordinary and property gain
        // remain NII, 3.8% x (5,000 + 6,000) = 418.
        let mut high = d.clone();
        high.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 650_000);
        match &mut high.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => {
                let PartnerParticipation::Limited { minutes, .. } = &mut data.participation else {
                    unreachable!()
                };
                *minutes = 30_001;
            }
            FederalIncome::Scorp { data } => data.participation.as_mut().unwrap().minutes = 30_001,
            _ => unreachable!(),
        }
        assert_eq!(value(&supported(&high), "net_investment_income_tax"), 418);

        // An unresolved within-rate-class allocation stays unresolved.
        // A 4,000 recapture of the 8,000 high-rate class cannot determine
        // how much belongs to its 5,000 business and 3,000 rental parts.
        match &mut high.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => {
                data.regular.section1231.as_mut().unwrap().unrecaptured = 500_000;
                data.regular.rentals.real_estate_property.unrecaptured = 300_000;
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                data.section1231.as_mut().unwrap().unrecaptured = 500_000;
                data.rentals.real_estate_property.unrecaptured = 300_000;
                data.amt = Some(amt_records(data));
            }
            _ => unreachable!(),
        }
        high.screening.as_mut().unwrap().section1231 =
            Some(section1231_history(&[(2025, 4_000, 4_000)], &[(2025, 4_000, 4_000)]));
        assert!(refused_with(
            &estimate(&high),
            "ISSUE_SECTION1231_SECTION_1231_LOOKBACK_RECAPTURE_NEEDS_BUSINESS_LEVEL_F77D46EA"
        ));
    }
}

#[test]
fn k1_rental_property_business_and_rental_losses_share_limits_but_keep_passive_parts() {
    for scorp in [false, true] {
        let (mut d, id) = rental_property_sale(false, scorp);
        amend_rental_sale(&mut d, |amounts, _| {
            amounts.real_estate = 0;
            amounts.real_estate_carryover = 0;
            amounts.real_estate_property =
                K1RentalProperty { net: -600_000, unrecaptured: 0, passive_loss: 0 };
            amounts.other = 500_000;
        });
        match &mut d.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => {
                data.regular.section1231 = Some(k1_section1231(-4_000, 0));
                data.alternative = Some(data.regular.clone());
            }
            FederalIncome::Scorp { data } => {
                data.section1231 = Some(k1_section1231(-4_000, 0));
                data.amt = Some(amt_records(data));
            }
            _ => unreachable!(),
        }
        let r = supported(&d);
        assert_eq!(value(&r, "form4797_line12"), -5_000);
        assert_eq!(return_line(&r, "11a / 11b"), 7_000_000);
        let mut scratch = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&d, 7_000_000, &mut scratch, false);
        let source = &pool.k1_section1231[0];
        assert_eq!((source.net, source.suspended), (-5_000, 5_000));
        assert_eq!((source.business().net, source.business().suspended), (-2_000, 2_000));
        assert_eq!((source.rentals[0].net, source.rentals[0].suspended), (-3_000, 3_000));
        assert_eq!(
            crate::kernel::ratio::dollars(
                pool_ledger(&d, id).passive_suspended[crate::k1_loss::K1LossClass::Section1231]
                    .get()
            ),
            5_000
        );
    }
}

#[test]
fn k1_rental_property_amt_uses_its_independent_property_amounts() {
    for scorp in [false, true] {
        let (mut d, _) = rental_property_sale(true, scorp);
        let property = match &mut d.income.last_mut().unwrap().income {
            FederalIncome::Partnership { data } => {
                &mut data.alternative.as_mut().unwrap().rentals.real_estate_property
            }
            FederalIncome::Scorp { data } => {
                &mut data.amt.as_mut().unwrap().rentals.real_estate_property
            }
            _ => unreachable!(),
        };
        property.net = 700_000;
        property.unrecaptured = 300_000;
        let _ = supported(&d);
        for amt in [false, true] {
            let mut scratch = estimate(&complete(FederalStatus::Single));
            let pool = crate::extended_rules::rentals_for(&d, 7_000_000, &mut scratch, amt);
            let worksheet = crate::section1231::worksheet(&d, &pool, amt).unwrap().unwrap();
            assert_eq!(
                (worksheet.net, worksheet.unrecaptured),
                if amt { (7_000, 3_000) } else { (6_000, 4_000) }
            );
        }
    }
}

#[test]
fn k1_rental_property_histories_round_trip_separately_on_both_pages() {
    for scorp in [false, true] {
        let (mut d, _) = rental_property_sale(false, scorp);
        amend_rental_sale(&mut d, |amounts, _| {
            amounts.real_estate_property.passive_loss = 123_400;
            amounts.other_carryover = 234_500;
            amounts.other_property.passive_loss = 345_600;
        });
        match &d.income.last().unwrap().income {
            FederalIncome::Partnership { data } => {
                let page = K1Partnership::from_record("Synthetic rental history", data).unwrap();
                assert_eq!(page.record(), *data);
            }
            FederalIncome::Scorp { data } => {
                let page = K1Scorp::from_record("Synthetic rental history", data).unwrap();
                assert_eq!(page.record(), *data);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn k1_rental_property_unconfirmed_and_death_history_projections_preserve_accepted_records() {
    let (mut d, _) = rental_property_sale(true, true);
    amend_rental_sale(&mut d, |amounts, _| {
        amounts.real_estate_property.passive_loss = 100_000;
        amounts.other_property.passive_loss = 200_000;
    });
    let FederalIncome::Scorp { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    assert!(data.suspended_history());
    let mut without = data.clone();
    without.drop_suspended_history();
    assert!(!without.rentals.passive_history());
    assert!(!without.amt.as_ref().unwrap().rentals.passive_history());
    assert_eq!(without.rentals.real_estate_property.net, 600_000);
    assert_eq!(without.rentals.real_estate_property.unrecaptured, 400_000);
    data.passive.complete = false;
    let accepted = d.clone();
    let mut notes = Vec::new();
    let working = crate::unknowns::supported(&d, &mut notes).unwrap();
    let FederalIncome::Scorp { data } = &working.income.last().unwrap().income else {
        unreachable!()
    };
    assert!(!data.rentals.passive_history());
    assert_eq!(d, accepted);
}

#[test]
fn k1_rental_property_mixed_ebl_retains_the_named_unresolved_allocation() {
    let (mut d, _) = rental_property_sale(true, true);
    amend_rental_sale(&mut d, |amounts, _| {
        amounts.real_estate = 0;
        amounts.real_estate_carryover = 0;
        amounts.real_estate_property =
            K1RentalProperty { net: -50_000_000, unrecaptured: 0, passive_loss: 0 };
    });
    let FederalIncome::Scorp { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.opening = 100_000_000;
    data.risk.start = ScorpRiskStart::Detailed { capacity: 100_000_000, profit: 0, loss: 0 };
    data.amt = Some(amt_records(data));
    assert!(refused_with(
        &estimate(&d),
        "ISSUE_SECTION1231_PROPERTY_EXCESS_BUSINESS_LOSS_REQUIRES_RESOLVED_975FCF28"
    ));
}

// Fixture A. Box 2 -$30,000, active participation. Form 8582: Part IV line
// 1b 30,000, line 1d and line 3 -30,000; line 4 30,000; line 6 MAGI 60,000
// (the passive loss is not in it); line 7 150,000 - 60,000 = 90,000; line 8
// min(45,000, 25,000) = 25,000; line 9 = line 11 = 25,000. Allowed -$25,000
// (Schedule E line 28 column (g)), $5,000 carried to 2027. AGI 35,000;
// taxable 18,900; tax table row 18,900-18,950: 1,240 + 12% x 6,525 = 2,023.
#[test]
fn fixture_a_general_partner_box_2_loss_uses_the_25000_allowance() {
    let (d, id) = with_partner(60_000, partner(-30_000, 0, K1RentalParticipation::Active));
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), -25_000);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 5_000);
    // The AMT pass reads the same amounts (same as regular).
    assert_eq!(rental_line(&r, "amt_partnership", id, "rental_real_estate", "net"), -25_000);
    assert_eq!(value(&r, "rental_magi"), 60_000);
    assert_eq!(return_line(&r, "8"), -2_500_000);
    assert_eq!(return_line(&r, "11a / 11b"), 3_500_000);
    assert_eq!(return_line(&r, "15"), 1_890_000);
    assert_eq!(return_line(&r, "16"), 202_300);
}

// Fixture A2. The same K-1 with $120,000 wages: line 6 120,000, line 7
// 30,000, line 8 15,000. Allowed -$15,000, suspended $15,000, AGI 105,000.
// One more dollar of MAGI at $150,000 leaves no allowance.
#[test]
fn fixture_a2_the_allowance_phases_out_with_magi() {
    let (d, id) = with_partner(120_000, partner(-30_000, 0, K1RentalParticipation::Active));
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), -15_000);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 15_000);
    assert_eq!(return_line(&r, "11a / 11b"), 10_500_000);
    let (d, id) = with_partner(150_000, partner(-30_000, 0, K1RentalParticipation::Active));
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), 0);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 30_000);
}

// Fixture B. A limited partner's box 2 -$30,000: no active participation
// (section 469(i)(6)(C)), Part V line 2b 30,000; line 1d 0, so Part II is
// skipped; line 10 0 and line 11 0. Nothing allowed, $30,000 carried; AGI
// stays 60,000. The stored record with Active is not valid.
#[test]
fn fixture_b_limited_partner_box_2_loss_is_fully_suspended() {
    let mut data = partner(-30_000, 0, K1RentalParticipation::NotActive);
    data.participation = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    let (d, id) = with_partner(60_000, data.clone());
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), 0);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 30_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    data.rental_facts.participation = K1RentalParticipation::Active;
    assert!(!data.valid());
}

// Fixture C. Active participation, box 2 +$4,000 and box 3 -$10,000. Part
// IV line 1a 4,000 (line 1d +4,000); Part V line 2b 10,000 (line 2d
// -10,000); line 3 -6,000; line 1d is not a loss, so Part II is skipped and
// line 11 = line 10 = 4,000. Box 3 has no special allowance: -$4,000
// allowed, $6,000 carried. AGI 60,000 + 4,000 - 4,000 = 60,000.
#[test]
fn fixture_c_box_3_other_rental_never_uses_the_allowance() {
    let (d, id) = with_partner(60_000, partner(4_000, -10_000, K1RentalParticipation::Active));
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), 4_000);
    assert_eq!(rental_line(&r, "partnership", id, "other_rental", "net"), -4_000);
    assert_eq!(rental_line(&r, "partnership", id, "other_rental", "suspended"), 6_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
}

// Fixture D. A real estate professional who materially participated: box
// 2 -$30,000 is nonpassive and allowed in full with $200,000 wages, where
// the allowance would be zero; MAGI (line 6) leaves the professional's loss
// out: 200,000. AGI 170,000. Without the recorded status the K-1 is refused
// by name.
#[test]
fn fixture_d_real_estate_professional_box_2_loss_is_nonpassive() {
    let (mut d, id) =
        with_partner(200_000, partner(-30_000, 0, K1RentalParticipation::Professional));
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(refused_with(&r, "ISSUE_K1_RENTAL_PROFESSIONAL_STATUS"));
    make_professional(&mut d, Owner::Taxpayer, false);
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), -30_000);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 0);
    assert_eq!(value(&r, "rental_magi"), 200_000);
    assert_eq!(return_line(&r, "11a / 11b"), 17_000_000);
}

// A professional's activity with a carryover is a former passive activity
// (section 469(f)(1)): box 2 +$10,000 first absorbs the $4,000 carryover,
// leaving $6,000 nonpassive; AGI 60,000 + 6,000.
#[test]
fn professional_carryover_offsets_its_own_activity_income_first() {
    let mut data = partner(10_000, 0, K1RentalParticipation::Professional);
    data.regular.rentals.real_estate_carryover = 400_000;
    data.alternative = Some(data.regular.clone());
    let (mut d, id) = with_partner(60_000, data);
    make_professional(&mut d, Owner::Taxpayer, false);
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), 6_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_600_000);
}

// Fixture E. An S-corporation shareholder may actively participate: box 2
// -$20,000 with MAGI 60,000 is within the $25,000 allowance and allowed in
// full; AGI 40,000. Stock basis $50,000 and the $3,000 note; amount at risk
// $50,000.
#[test]
fn fixture_e_shareholder_box_2_loss_within_the_allowance() {
    let mut source = complete_ordinary_scorp(0, false);
    source.opening = 5_000_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
    source.rentals.real_estate = -2_000_000;
    source.rental_facts.participation = K1RentalParticipation::Active;
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    let r = supported(&d);
    assert_eq!(rental_line(&r, "scorp", id, "rental_real_estate", "net"), -20_000);
    assert_eq!(rental_line(&r, "scorp", id, "rental_real_estate", "suspended"), 0);
    assert_eq!(return_line(&r, "11a / 11b"), 4_000_000);
}

// Fixture F, fixture 3a of the loss-carryforward map. A limited partner
// (not active) with $5,000 of opening basis and amount at risk and a box 2
// loss of $20,000: the basis worksheet allows $5,000 (line 15b) and carries
// $15,000 (column E); Form 6198 line 4 allows the $5,000; Form 8582 Part V
// has -$5,000 and no passive income, so nothing is allowed and $5,000 is
// suspended. AGI stays 60,000. The shareholder twin carries what its stock
// and debt basis ($13,000) cannot absorb.
#[test]
fn fixture_f_rental_loss_limited_by_basis_carries_then_meets_the_passive_rules() {
    let mut data = partner(-20_000, 0, K1RentalParticipation::NotActive);
    data.participation = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    data.regular.basis.opening = 500_000;
    data.regular.risk.start = ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 };
    data.alternative = Some(data.regular.clone());
    let ledger = data.ledger(&data.regular).unwrap();
    let rental = crate::k1_loss::K1LossClass::RentalRealEstate;
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_allowed[rental].get()), 5_000);
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_suspended[rental].get()), 15_000);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_allowed[rental].get()), 5_000);
    assert_eq!(crate::kernel::ratio::dollars(ledger.risk_suspended[rental].get()), 0);
    let (d, id) = with_partner(60_000, data);
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), 0);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 5_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    // The passive stage completes the same ledger: the 5,000 the first two
    // limits allowed stays suspended on Form 8582.
    let ledger = pool_ledger(&d, id);
    assert_eq!(crate::kernel::ratio::dollars(ledger.passive_suspended[rental].get()), 5_000);
    assert!(r.review_notes.iter().any(|v| v.contains(
        "rental real estate loss carried to 2027 — $15000 because of your basis, $0 because of the at-risk limits"
    )));
    let mut source = complete_ordinary_scorp(0, false);
    source.rentals.real_estate = -3_000_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_300_000, profit: 0, loss: 0 };
    source.risk.deducted = 0;
    source.amt = Some(amt_records(&source));
    let ledger = source.ledger().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_allowed[rental].get()), 13_000);
    assert_eq!(crate::kernel::ratio::dollars(ledger.basis_suspended[rental].get()), 17_000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    let r = supported(&d);
    assert!(r.review_notes.iter().any(|v| v.contains(
        "rental real estate loss carried to 2027 — $17000 because of your basis, $0 because of the at-risk limits"
    )));
}

// Fixture 3b of the loss-carryforward map: the 2026 release into the passive
// pool. A limited partner carries $15,000 by basis and $5,000 on Form 8582,
// has contributed $6,000 and has a Form 6198 history of zero left; box 2 is
// +$8,000. Basis 0 + 6,000 + 8,000 = 14,000 against the 15,000 class: 14,000
// allowed, 1,000 carried. Form 6198: investment 6,000, income 8,000, loss
// 14,000 is allowed and nothing remains at risk. Form 8582: the activity's net
// is 8,000 - 14,000 = -6,000, less the 5,000 carryover: -11,000, with no
// allowance and no passive income: all of it suspended. AGI 60,000.
#[test]
fn fixture_3b_a_released_basis_carry_is_a_passive_deduction_of_the_release_year() {
    let mut data = partner(8_000, 0, K1RentalParticipation::NotActive);
    data.participation = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    data.regular.basis.opening = 0;
    data.regular.basis.contributions = 600_000;
    data.regular.carried.real_estate.basis = 1_500_000;
    data.regular.rentals.real_estate_carryover = 500_000;
    data.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 500_000 };
    data.alternative = Some(data.regular.clone());
    let pool = data.pool(&data.regular).unwrap();
    let rental = crate::k1_loss::K1LossClass::RentalRealEstate;
    assert_eq!(
        crate::kernel::ratio::dollars(pool.activity.ledger.basis_allowed[rental].get()),
        14_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(pool.activity.ledger.basis_suspended[rental].get()),
        1_000
    );
    assert_eq!(pool.activity.risk_end, Some(0));
    let (d, id) = with_partner(60_000, data);
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), 0);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 11_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_000_000);
    assert_eq!(
        crate::kernel::ratio::dollars(pool_ledger(&d, id).passive_suspended[rental].get()),
        11_000
    );
    assert!(r.review_notes.iter().any(|v| v.contains(
        "rental real estate loss carried to 2027 — $1000 because of your basis, $0 because of the at-risk limits"
    )));
}

// Fixture G. Box 2 +$10,000 with a $4,000 carryover, no active
// participation: +$6,000 passive income (column (h)); AGI 66,000. The
// section 199A statement lists the activity: QBI 6,000 x 20% = 1,200, below
// 20% of taxable income before QBI (49,900 x 20% = 9,980).
#[test]
fn fixture_g_listed_rental_activity_is_qbi_after_its_carryover() {
    let mut data = partner(10_000, 0, K1RentalParticipation::NotActive);
    data.regular.rentals.real_estate_carryover = 400_000;
    data.alternative = Some(data.regular.clone());
    data.rental_facts.real_estate_qbi = Some(K1RentalQbi { wages: 0, property: 0 });
    let (d, id) = with_partner(60_000, data);
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), 6_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_600_000);
    assert_eq!(return_line(&r, "13b"), 120_000);
}

// A rental disposition and the rentals of a publicly traded partnership
// stay refused by name.
#[test]
fn rental_disposition_and_ptp_rentals_are_refused_by_name() {
    let mut data = partner(-30_000, 0, K1RentalParticipation::Active);
    data.unentered = vec![PartnerK1Item::RentalDisposition];
    let (d, _) = with_partner(60_000, data);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(refused_with(&r, "ISSUE_PARTNER_K1_RENTAL_DISPOSITION"));
    let mut data = partner(5_000, 0, K1RentalParticipation::NotActive);
    data.publicly_traded = true;
    let (d, _) = with_partner(60_000, data);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(refused_with(&r, "ISSUE_K1_RENTAL_PTP"));
}

// Fixture A3 (F3, 2026-10-09). i8582 Part IV column (c) and section
// 469(i)(1): a prior-year unallowed loss reaches the special allowance only
// when the person actively participated in the year it arose. Box 2
// -$10,000 with a $30,000 carryover, active now, MAGI 60,000 (allowance
// 25,000). Active then too: the whole -40,000 competes for 25,000, so
// -25,000 is allowed and 15,000 is carried. Not active then: only the
// current -10,000 is eligible, so -10,000 is allowed and the 30,000
// carryover stays suspended (Worksheet 3).
#[test]
fn fixture_a3_a_carryover_from_an_inactive_year_stays_out_of_the_allowance() {
    for (carryover_active, allowed, suspended) in
        [(true, -25_000, 15_000), (false, -10_000, 30_000)]
    {
        let mut data = partner(-10_000, 0, K1RentalParticipation::Active);
        data.regular.rentals.real_estate_carryover = 3_000_000;
        data.alternative = Some(data.regular.clone());
        data.rental_facts.carryover_active = carryover_active;
        let (d, id) = with_partner(60_000, data);
        let r = supported(&d);
        assert_eq!(
            rental_line(&r, "partnership", id, "rental_real_estate", "net"),
            allowed,
            "{carryover_active}"
        );
        assert_eq!(
            rental_line(&r, "partnership", id, "rental_real_estate", "suspended"),
            suspended,
            "{carryover_active}"
        );
    }
}

#[test]
fn unit21_k1_rental_professional_status_belongs_to_the_current_filer() {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::California, rule: CommunityRule::Halves };
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 600_000);
    d.income[0].community = CommunityShare::Community;
    let mut entry = income(FederalIncome::Partnership {
        data: partner(-30_000, 0, K1RentalParticipation::Professional),
    });
    entry.owner = Owner::Spouse;
    entry.community = CommunityShare::Community;
    add(&mut d, entry);
    let id = d.income.last().unwrap().id;
    // The separate return records its own filer's professional qualification;
    // the spouse-owned K-1 retains the actual activity participation answer.
    let other_only = estimate(&d);
    assert!(refused_with(&other_only, "ISSUE_K1_RENTAL_PROFESSIONAL_STATUS"));
    make_professional(&mut d, Owner::Taxpayer, true);
    let r = supported(&d);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "net"), -15_000);
    assert_eq!(rental_line(&r, "partnership", id, "rental_real_estate", "suspended"), 0);
    assert_eq!(value(&r, "agi"), 15_000);
}

#[test]
fn unit4_former_passive_professional_remainder_requires_both_active_years() {
    for scorp in [false, true] {
        for (participation, loss_year_active, expected_agi, expected_carry, expected_line43) in [
            (K1RentalParticipation::ProfessionalActive, true, 108_000, 0, -2_000),
            (K1RentalParticipation::Professional, true, 110_000, 2_000, 0),
            (K1RentalParticipation::ProfessionalActive, false, 110_000, 2_000, 0),
        ] {
            let mut source = partner(4_000, 0, participation);
            source.regular.rentals.real_estate_carryover = 600_000;
            source.rental_facts.carryover_active = loss_year_active;
            source.alternative = Some(source.regular.clone());
            let mut d = complete(FederalStatus::Single);
            d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 0);
            let id = d.next_id;
            let kind = if scorp {
                let mut s = complete_ordinary_scorp(0, false);
                s.opening = 5_000_000;
                s.risk.start = ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
                s.rentals = source.regular.rentals.clone();
                s.rental_facts = source.rental_facts.clone();
                s.amt = Some(amt_records(&s));
                add(&mut d, income(FederalIncome::Scorp { data: s }));
                "scorp"
            } else {
                add(&mut d, income(FederalIncome::Partnership { data: source }));
                "partnership"
            };
            let mut other = rental(10_000, 10_000);
            other.participation = RentalParticipation::Active;
            add(&mut d, income(FederalIncome::Rental { data: other }));
            make_professional(&mut d, Owner::Taxpayer, false);
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), expected_agi);
            assert_eq!(
                rental_line(&r, kind, id, "rental_real_estate", "net"),
                4_000 - (6_000 - expected_carry)
            );
            assert_eq!(
                rental_line(&r, kind, id, "rental_real_estate", "suspended"),
                expected_carry
            );
            assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), expected_line43);
            assert_eq!(
                rental_line(&r, &format!("amt_{kind}"), id, "rental_real_estate", "suspended"),
                expected_carry
            );
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

#[test]
fn unit4_limited_partner_cannot_choose_professional_active_eligibility() {
    let mut source = partner(4_000, 0, K1RentalParticipation::ProfessionalActive);
    source.participation = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    assert!(!source.valid());
    source.rental_facts.participation = K1RentalParticipation::Professional;
    assert!(source.valid());
}

#[test]
fn unit4_professional_election_keeps_k1_loss_until_all_rental_members_are_disposed() {
    for scorp in [false, true] {
        for joint in [false, true] {
            let mut source = partner(0, 0, K1RentalParticipation::Professional);
            source.regular.rentals.real_estate_carryover = 600_000;
            source.rental_facts.disposed = vec![K1RentalActivity::RealEstate];
            source.alternative = Some(source.regular.clone());
            let mut d = complete(FederalStatus::Single);
            let id = d.next_id;
            let kind = if scorp {
                let mut s = complete_ordinary_scorp(0, false);
                s.opening = 5_000_000;
                s.risk.start = ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
                s.rentals = source.regular.rentals.clone();
                s.rental_facts = source.rental_facts.clone();
                s.amt = Some(amt_records(&s));
                add(&mut d, income(FederalIncome::Scorp { data: s }));
                "scorp"
            } else {
                add(&mut d, income(FederalIncome::Partnership { data: source }));
                "partnership"
            };
            make_professional(&mut d, Owner::Taxpayer, false);
            d.people
                .as_mut()
                .unwrap()
                .taxpayer
                .real_estate_professional
                .as_mut()
                .unwrap()
                .single_activity = true;
            let mut kept = rental(10_000, 0);
            kept.participation = RentalParticipation::ProfessionalActive;
            add(&mut d, income(FederalIncome::Rental { data: kept }));
            assert_eq!(crate::extended_rules::professional_election_rule(&d), None);
            assert!(!crate::extended_rules::professional_group_disposed(&d, Owner::Taxpayer));
            if joint {
                let other = complete(FederalStatus::Joint);
                d.setup = other.setup;
                d.people.as_mut().unwrap().spouse = other.people.unwrap().spouse;
                d.income.iter_mut().find(|entry| entry.id == id).unwrap().owner = Owner::Spouse;
            }
            let r = supported(&d);
            assert_eq!(rental_line(&r, kind, id, "rental_real_estate", "suspended"), 6_000);
            assert_eq!(value(&r, "agi"), 60_000);
            // The only remaining member is the actually disposed K-1 rental.
            d.income.pop();
            assert!(crate::extended_rules::professional_group_disposed(&d, Owner::Taxpayer));
            let r = supported(&d);
            assert_eq!(rental_line(&r, kind, id, "rental_real_estate", "suspended"), 0);
            assert_eq!(value(&r, "agi"), 54_000);
            assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), -6_000);
        }
    }
}

#[test]
fn unit4_elected_former_passive_combines_direct_and_each_k1_owner() {
    for scorp in [false, true] {
        for filing in 0..4 {
            let (mut d, [u, v, _]) =
                crate::tests::activity::rental_specials::unit4_three_buildings();
            let mut data = partner(-5_000, 0, K1RentalParticipation::Professional);
            data.regular.rentals.real_estate_carryover = 1_000_000;
            data.alternative = Some(data.regular.clone());
            data.alternative.as_mut().unwrap().rentals.real_estate_carryover = 600_000;
            let source = if scorp {
                let mut shareholder = complete_ordinary_scorp(0, false);
                shareholder.opening = 5_000_000;
                shareholder.risk.start =
                    ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
                shareholder.rentals = data.regular.rentals.clone();
                shareholder.rental_facts = data.rental_facts.clone();
                shareholder.amt = Some(amt_records(&shareholder));
                shareholder.amt.as_mut().unwrap().rentals.real_estate_carryover = 600_000;
                FederalIncome::Scorp { data: shareholder }
            } else {
                FederalIncome::Partnership { data }
            };
            d.income.iter_mut().find(|entry| entry.id == v).unwrap().income = source;
            if filing == 1 {
                let other = complete(FederalStatus::Joint);
                d.setup = other.setup;
                d.people.as_mut().unwrap().spouse = other.people.unwrap().spouse;
                d.income.iter_mut().find(|entry| entry.id == v).unwrap().owner = Owner::Spouse;
            }
            if filing >= 2 {
                let other = complete(FederalStatus::Separate);
                d.setup = other.setup;
                d.people.as_mut().unwrap().spouse = other.people.unwrap().spouse;
                d.setup.as_mut().unwrap().community_property = CommunityProperty::Yes {
                    state: CommunityState::Arizona,
                    rule: CommunityRule::Halves,
                };
                d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
                for entry in &mut d.income {
                    entry.community = CommunityShare::Community;
                    if entry.id == v {
                        entry.owner = Owner::Spouse;
                    }
                }
            }
            if filing >= 2 {
                let mut excluded = d.income.iter().find(|entry| entry.id == u).unwrap().clone();
                excluded.id = 0;
                excluded.owner = Owner::Spouse;
                excluded.community = CommunityShare::Separate;
                let FederalIncome::Rental { data } = &mut excluded.income else { unreachable!() };
                data.participation = RentalParticipation::NotActive;
                // Arizona separate spouse property contributes neither income nor
                // property gains; its answer cannot contradict this return's election.
                add(&mut d, excluded);
            }
            let divisor = if filing >= 2 { 2 } else { 1 };
            let r = supported(&d);
            let restarted = restart(d.clone());
            assert_eq!(estimate(&restarted.draft), r);
            assert_eq!(restarted.draft.income, d.income);
            assert_eq!(restarted.draft.people, d.people);
            let kind = if scorp { "scorp" } else { "partnership" };
            assert_eq!(value(&r, "agi"), 60_000 / divisor);
            assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 0);
            assert_eq!(value(&r, &format!("rental_{u}_suspended_loss")), 22_500 / divisor);
            assert_eq!(
                rental_line(&r, kind, v, "rental_real_estate", "suspended"),
                7_500 / divisor
            );
            assert_eq!(rental_line(&r, kind, v, "rental_real_estate", "net"), -7_500 / divisor);
            assert_eq!(value(&r, &format!("amt_rental_{u}_suspended_loss")), 16_000 / divisor);
            assert_eq!(
                rental_line(&r, &format!("amt_{kind}"), v, "rental_real_estate", "suspended"),
                4_000 / divisor
            );
            let ledger = pool_ledger(&d, v);
            assert_eq!(
                crate::kernel::ratio::dollars(
                    ledger.passive_suspended[crate::k1_loss::K1LossClass::RentalRealEstate].get()
                ),
                7_500 / divisor
            );
        }
    }
}

#[test]
fn unit4_elected_former_passive_preserves_k1_property_character_and_line43_footing() {
    for scorp in [false, true] {
        let (mut d, [u, v, _]) = crate::tests::activity::rental_specials::unit4_three_buildings();
        d.income.truncate(3);
        let mut source = partner(-5_000, 0, K1RentalParticipation::Professional);
        source.regular.rentals.real_estate_carryover = 600_000;
        source.regular.rentals.real_estate_property =
            K1RentalProperty { net: 1_000_000, unrecaptured: 0, passive_loss: 400_000 };
        source.alternative = Some(source.regular.clone());
        let source = if scorp {
            let mut shareholder = complete_ordinary_scorp(0, false);
            shareholder.opening = 5_000_000;
            shareholder.risk.start =
                ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
            shareholder.rentals = source.regular.rentals;
            shareholder.rental_facts = source.rental_facts;
            shareholder.amt = Some(amt_records(&shareholder));
            FederalIncome::Scorp { data: shareholder }
        } else {
            FederalIncome::Partnership { data: source }
        };
        d.income.iter_mut().find(|entry| entry.id == v).unwrap().income = source;
        d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
        let r = supported(&d);
        let kind = if scorp { "scorp" } else { "partnership" };
        // Combined current ordinary $0/property $10,000; prior ordinary
        // $36,000/property $4,000. Part IX releases $4,000 property and
        // $6,000 ordinary, apportioned $5,000/$1,000 to the actual sources.
        assert_eq!(value(&r, "agi"), 60_000);
        assert_eq!(value(&r, "schedule_e_line43_professional_rentals"), 0);
        assert_eq!(value(&r, &format!("rental_{u}_suspended_loss")), 25_000);
        assert_eq!(rental_line(&r, kind, v, "rental_real_estate", "suspended"), 5_000);
        assert_eq!(rental_line(&r, kind, v, "rental_real_estate", "net"), -6_000);
        assert_eq!(rental_line(&r, kind, v, "rental_real_estate", "section1231_suspended"), 0);
        let mut output = FederalResult::empty(0);
        let regular = crate::extended_rules::rentals_for(&d, 60_000, &mut output, false);
        let property = regular.k1_section1231.iter().find(|property| property.id == v).unwrap();
        assert_eq!(property.rentals[0].net, 6_000);
        assert_eq!(property.rentals[0].nonpassive, 6_000);
        assert_eq!(regular.rental_niit_adjustment, 6_000);
        assert_eq!(regular.modified_agi, 60_000);
    }
}
