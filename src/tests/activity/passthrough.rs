use super::*;

#[test]
fn rental_special_treatment_and_at_risk_limits_are_left_out_with_a_notice() {
    for field in 0..1 {
        let mut d = complete(FederalStatus::Single);
        let mut v = rental(30000, 5000);
        match field {
            0 => v.ordinary_rental = false,
            _ => unreachable!(),
        }
        add(&mut d, income(FederalIncome::Rental { data: v }));
        assert!(noted(&supported(&d), "WARN_RENTAL_LEFT_OUT"));
    }
    // A rental received in a like-kind exchange before 2026 is the same
    // confirmation (Treas. Reg. 1.168(i)-6): left out, and the notice names it.
    let mut d = complete(FederalStatus::Single);
    let mut v = rental(30000, 5000);
    v.ordinary_rental = false;
    add(&mut d, income(FederalIncome::Rental { data: v }));
    let r = supported(&d);
    assert!(noted(&r, "WARN_RENTAL_LEFT_OUT"));
    assert!(r.review_notes.iter().any(|n| n.contains("like-kind exchange before 2026")));
    // No active participation: no special allowance, no note.
    let mut d = complete(FederalStatus::Single);
    let mut v = rental(30000, 5000);
    v.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: v }));
    supported(&d);
}

#[test]
fn personal_rental_investment_income_uses_strict_thirty_percent_basis_test() {
    for (basis, depreciable, expected) in
        [(10000, 2999, 9000), (10000, 3000, 2000), (10000, 3001, 2000), (10000, 0, 9000)]
    {
        let mut d = complete(FederalStatus::Single);
        let mut rent = personal_rent(10000, 3000);
        let FederalIncome::NonbusinessProperty { kind, .. } = &mut rent else { panic!() };
        *kind = NonbusinessPropertyKind::PersonalRental {
            basis,
            depreciable_basis: depreciable,
            assets: vec![],
            history: PersonalRentalHistory::NONE,
        };
        add(&mut d, income(rent));
        add(&mut d, income(savings_income(2000, 0)));
        investment_borrowing(&mut d, 12000, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 69000);
        assert_eq!(value(&r, "investment_interest_deduction"), expected);
        assert_eq!(value(&r, "investment_interest_carryforward"), 12000 - expected);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), expected);
        assert_eq!(value(&r, "personal_rent_2_investment_income"), expected - 2000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn personal_rental_basis_records_validate_and_suspend_unabsorbed_loss() {
    let mut d = complete(FederalStatus::Single);
    for (basis, depreciable) in [(-1, 0), (0, 1), (100, 101), (100, -1), (100000000001, 0)] {
        let mut rent = personal_rent(10000, 3000);
        let FederalIncome::NonbusinessProperty { kind, .. } = &mut rent else { panic!() };
        *kind = NonbusinessPropertyKind::PersonalRental {
            basis,
            depreciable_basis: depreciable,
            assets: vec![],
            history: PersonalRentalHistory::NONE,
        };
        assert!(crate::federal_rules::validate_income(&income(rent)).is_err());
    }
    add(&mut d, income(personal_rent(1000, 1001)));
    investment_borrowing(&mut d, 10000, 0, 0);
    assert_eq!(value(&supported(&d), "rental_suspended_loss"), 1);
}

#[test]
fn personal_rental_loss_is_suspended_without_real_estate_special_allowance() {
    for depreciable in [0, 1000000] {
        let mut d = complete(FederalStatus::Single);
        let mut rent = personal_rent(1000, 13000);
        let FederalIncome::NonbusinessProperty { kind, .. } = &mut rent else { panic!() };
        *kind = NonbusinessPropertyKind::PersonalRental {
            basis: 1000000,
            depreciable_basis: depreciable,
            assets: vec![],
            history: PersonalRentalHistory::NONE,
        };
        add(&mut d, income(rent));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 60000);
        assert_eq!(value(&r, "rental_magi"), 60000);
        assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 1000);
        assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 12000);
        assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), 12000);
        assert_eq!(value(&r, "rental_income"), 0);
        assert_eq!(value(&r, "current_net_operating_loss"), 0);
        assert_eq!(return_line(&r, "10"), 100000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn personal_rental_loss_retains_separate_amt_carryover_when_real_estate_profit_differs() {
    let mut d = complete(FederalStatus::Single);
    let mut property = rental(10000, 0);
    property.buildings[0].in_service = 19900101;
    add(&mut d, income(FederalIncome::Rental { data: property }));
    add(&mut d, income(personal_rent(1000, 13000)));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_3_suspended_loss"), 2000);
    assert_eq!(value(&r, "amt_personal_rental_3_suspended_loss"), 8875);
    assert_eq!(value(&r, "amt_rental_adjustment"), 0);
    assert_eq!(value(&r, "agi"), 60000);
}

#[test]
fn personal_rental_bonus_reduces_basis_even_when_passive_deduction_is_suspended() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(personal_rent_with_assets(
            10000,
            2000,
            vec![rental_equipment(20000, BonusElection::Apply)],
        )),
    );
    let r = supported(&d);
    for (id, amount) in [
        ("personal_rental_2_asset_0_bonus", 20000),
        ("personal_rental_2_asset_0_macrs", 0),
        ("personal_rental_2_asset_0_remaining_basis", 0),
        ("nonbusiness_personal_rent_expenses_paid", 2000),
        ("nonbusiness_personal_rent_depreciation", 20000),
        ("nonbusiness_personal_rent_expenses", 10000),
        ("personal_rental_2_suspended_loss", 12000),
        ("amt_personal_rental_2_suspended_loss", 12000),
        ("agi", 60000),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    // Another activity's passive profit releases the deduction without
    // changing the fully depreciated basis or treating the loss as Schedule E.
    add(&mut d, income(FederalIncome::Rental { data: rental(25000, 0) }));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "personal_rental_2_asset_0_remaining_basis"), 0);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 22000);
    assert_eq!(value(&r, "rental_income"), 15000);
    assert_eq!(value(&r, "agi"), 63000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn continuing_personal_rental_depreciation_preserves_basis_and_separate_passive_histories() {
    let mut d = complete(FederalStatus::Single);
    let mut rent =
        personal_rent_with_assets(1000, 100, vec![continuing_equipment(20250601, 20000, false)]);
    let FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental { history, .. },
        ..
    } = &mut rent
    else {
        unreachable!()
    };
    // Retained worksheets supply prior suspended depreciation. It already
    // reduced opening asset basis; release must not reduce basis a second time.
    *history = PersonalRentalHistory { regular: 10000, amt: 20000 };
    add(&mut d, income(rent));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_2_asset_0_macrs"), 320);
    assert_eq!(value(&r, "personal_rental_2_asset_0_remaining_basis"), 480);
    assert_eq!(value(&r, "personal_rental_2_allowed_expenses"), 520);
    assert_eq!(value(&r, "amt_personal_rental_2_allowed_expenses"), 620);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "amt_rental_adjustment"), -100);
    assert_eq!(value(&r, "agi"), 60480);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn personal_rental_assets_validate_basis_and_exclude_personal_use_and_old_section179() {
    let mut d = complete(FederalStatus::Single);
    let mut rent =
        personal_rent_with_assets(10000, 0, vec![rental_equipment(1000, BonusElection::Apply)]);
    let FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental { depreciable_basis, .. },
        ..
    } = &mut rent
    else {
        unreachable!()
    };
    *depreciable_basis -= 1;
    assert!(crate::federal_rules::validate_income_amount(&rent).is_err());
    let mut asset = rental_equipment(1000, BonusElection::Apply);
    asset.listed = ListedProperty::No { business_percent: 50 };
    add(&mut d, income(personal_rent_with_assets(10000, 0, vec![asset])));
    unsupported(&d, "Income:");
    let mut asset = continuing_equipment(20250601, 20000, false);
    let EquipmentHistory::Continuing { initial_section179, .. } = &mut asset.history else {
        unreachable!()
    };
    *initial_section179 = 10000;
    d.income[1].income = personal_rent_with_assets(10000, 0, vec![asset]);
    unsupported(&d, "Income:");
}

#[test]
fn at_risk_prior_suspension_bypasses_basis_and_qbi_waits_for_release() {
    use crate::scorp_basis::{LossVintage, StockBasis, release_losses};
    let basis = StockBasis {
        opening: k1_amount(2_000),
        additions: k1_amount(0),
        income: k1_amount(0),
        distributions: k1_amount(0),
        nondeductible: k1_amount(0),
        prior_nondeductible: k1_amount(0),
        losses_first: false,
        losses: &LossAmounts::of(&[(K1LossClass::Ordinary, k1_amount(3_000))]),
        credit_reduction: k1_amount(0),
    }
    .with_loans(&[])
    .unwrap();
    assert_eq!(basis.allowed, LossAmounts::of(&[(K1LossClass::Ordinary, k1_amount(2_000))]));
    let risk = crate::k1_loss::limit_classes(
        k1_amount(0),
        k1_amount(3_000),
        &basis.allowed,
        &LossAmounts::of(&[(K1LossClass::Ordinary, k1_amount(2_000))]),
        k1_amount(0),
    )
    .unwrap();
    assert_eq!(crate::kernel::ratio::dollars(risk.allowed[K1LossClass::Ordinary].get()), 3_000);
    assert_eq!(crate::kernel::ratio::dollars(risk.suspended[K1LossClass::Ordinary].get()), 1_000);
    let release = release_losses(
        &[
            LossVintage {
                year: 2017,
                original: k1_amount(2_000),
                original_qbi: k1_amount(0),
                remaining: k1_amount(2_000),
                remaining_qbi: k1_amount(0),
            },
            LossVintage {
                year: 2026,
                original: k1_amount(2_000),
                original_qbi: k1_amount(2_000),
                remaining: k1_amount(2_000),
                remaining_qbi: k1_amount(2_000),
            },
        ],
        risk.allowed[K1LossClass::Ordinary],
    )
    .unwrap();
    assert_eq!(crate::kernel::ratio::dollars(release.qbi.get()), 1_000);
    // Passive rules may still suspend the amount released through at-risk.
    assert_eq!(next_loss_limit(&release, 0).unwrap().0, 0);
}

#[test]
fn at_risk_recapture_and_invalid_sources_are_explicit() {
    use crate::k1_loss::limit_classes;
    use crate::scorp_basis::BasisError;
    let empty = LossAmounts::default();
    assert_eq!(
        limit_classes(
            crate::kernel::carry::Amount::new(i64::MAX).unwrap(),
            k1_amount(1),
            &empty,
            &empty,
            k1_amount(0)
        ),
        Err(BasisError::Invalid)
    );
    assert_eq!(
        limit_classes(k1_amount(0), k1_amount(0), &empty, &empty, k1_amount(0))
            .unwrap()
            .allowed_total
            .get(),
        0
    );
}

#[test]
fn at_risk_detailed_history_is_adjusted_before_current_activity_results() {
    use crate::scorp_basis::prior_detailed_risk;
    // Prior line 19b $10,000, line 21 $6,000. The $6,000 already reduced
    // at-risk investment even if passive rules subsequently suspended it.
    assert_eq!(
        crate::kernel::ratio::dollars(
            prior_detailed_risk(k1_amount(10_000), k1_amount(0), k1_amount(6_000)).unwrap().get()
        ),
        4_000
    );
    let opening = prior_detailed_risk(k1_amount(10_000), k1_amount(2_000), k1_amount(0)).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(opening.get()), 12_000);
    // Current $5,000 income appears in Part I only, not twice in capacity.
    let r = crate::k1_loss::limit_classes(
        k1_amount(5_000),
        opening,
        &LossAmounts::of(&[(K1LossClass::Ordinary, k1_amount(20_000))]),
        &LossAmounts::default(),
        k1_amount(0),
    )
    .unwrap();
    assert_eq!(crate::kernel::ratio::dollars(r.allowed[K1LossClass::Ordinary].get()), 17_000);
    assert_eq!(crate::kernel::ratio::dollars(r.allowed_total.get()) - 5_000, 12_000);
}

#[test]
fn at_risk_history_rejects_impossible_or_overflowing_records() {
    use crate::kernel::carry::{Amount, Dollars};
    use crate::kernel::loss_limits::risk::{RecaptureHistory, recapture_from_ending};
    use crate::kernel::source_fact::SourceFact;
    use crate::scorp_basis::prior_detailed_risk;
    assert!(prior_detailed_risk(k1_amount(10), k1_amount(1), k1_amount(1)).is_err());
    assert!(prior_detailed_risk(k1_amount(10), k1_amount(0), k1_amount(11)).is_err());
    assert!(
        prior_detailed_risk(
            crate::kernel::carry::Amount::new(i64::MAX).unwrap(),
            k1_amount(1),
            k1_amount(0)
        )
        .is_err()
    );
    let mut h = RecaptureHistory::<Dollars> {
        prior_deducted: Amount::new(10).unwrap(),
        prior_recaptured: Amount::new(11).unwrap(),
        threshold_1978: 0,
    };
    assert!(recapture_from_ending(-1, SourceFact::Reported(h)).is_err());
    h = RecaptureHistory {
        prior_deducted: Amount::new(10).unwrap(),
        prior_recaptured: Amount::new(0).unwrap(),
        threshold_1978: 0,
    };
    assert!(recapture_from_ending(i64::MIN, SourceFact::Reported(h)).is_err());
    h = RecaptureHistory {
        prior_deducted: Amount::new(10).unwrap(),
        prior_recaptured: Amount::new(0).unwrap(),
        threshold_1978: 1,
    };
    assert!(recapture_from_ending(-1, SourceFact::Reported(h)).is_err());
}

#[test]
fn participation_invalid_and_missing_sources_keep_named_boundaries() {
    use crate::participation::Participation;
    let mut facts = participation_facts();
    facts.management_minutes = facts.minutes + 1;
    assert_eq!(facts.material(), Participation::Unavailable);
    let invalid_limited = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 11,
        personal_service: false,
        service_years: 0,
    };
    assert_eq!(invalid_limited.material(), Participation::Unavailable);
    let mut source = scorp_document();
    source.participation = None;
    assert_eq!(source.nonpassive(), Participation::Unavailable);
    assert_eq!(
        source.pool_activity_result().unwrap_err().0,
        "ISSUE_SCORP_COMPLETE_THE_BUSINESS_CLASSIFICATION_AND_APPLICABLE_05C04B0A",
    );
    source.trade_kind = ScorpTradeKind::FinancialTrading;
    assert_eq!(source.nonpassive(), Participation::Yes);
    assert_eq!(
        source.investment_interest_eligible().unwrap_err().0,
        "ISSUE_SCORP_RESOLVE_TRADING_BUSINESS_CLASSIFICATION_AND_MATERIAL_PARTICIPA_EDFB91CC",
    );
}

#[test]
fn live_passive_pool_combines_k1_after_basis_at_risk_and_prior_passive_loss() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: passive_scorp_document() }));
    let rental_id = d.next_id;
    add(&mut d, income(FederalIncome::Rental { data: rental(16_000, 0) }));
    assert_eq!(restart(d.clone()).draft, d);
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    passive_note(&r, id, -6_000, 4_000);
    passive_note(&r, rental_id, 6_000, 0);
    scorp_note(&r, id, "basis suspended loss", 2_000);
    scorp_note(&r, id, "loss still suspended by at-risk", 5_000);
}

#[test]
fn live_passive_k1_loss_cannot_use_rental_allowance_without_rental_activity() {
    let (_, r, id) = scorp_risk_result(passive_scorp_document());
    passive_note(&r, id, 0, 10_000);
}

#[test]
fn live_passive_k1_profit_offsets_rental_loss_before_special_allowance() {
    let mut source = passive_scorp_document();
    source.ordinary = 1_000_000;
    source.passive.loss = 0;
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    let rental_id = d.next_id;
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 20_000) }));
    let r = estimate(&d);
    passive_note(&r, id, 10_000, 0);
    passive_note(&r, rental_id, -30_000, 0);
    assert!(r.tax.is_none() && r.lines.is_empty());
}

#[test]
fn live_passive_pool_does_not_present_missing_k1_facts_as_zero() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: passive_scorp_document() }));
    let mut unknown = passive_scorp_document();
    unknown.participation = None;
    add(&mut d, income(FederalIncome::Scorp { data: unknown.clone() }));
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.section == "S-corporation passive pool"));
    assert!(!r.review_notes.iter().any(|n| n.starts_with("Passive activity ")));
    assert!(unknown.activity_result().is_err());
    unknown.participation = Some(participation_facts());
    unknown.participation.as_mut().unwrap().minutes = 0;
    unknown.passive.loss = -1;
    assert!(!unknown.valid());
}

#[test]
fn former_passive_scorp_uses_own_income_before_the_shared_pool() {
    use crate::scorp::ScorpActivity;
    for (current, prior, passive, nonpassive, released) in [
        (10_000, 0, 0, 10_000, 0),
        (10_000, 4_000, 0, 6_000, 4_000),
        (10_000, 10_000, 0, 0, 10_000),
        (10_000, 15_000, -5_000, 0, 10_000),
        (0, 15_000, -15_000, 0, 0),
        (-15_000, 2_000, -2_000, -8_000, 0),
    ] {
        let activity = nonpassive_scorp(current, prior).activity_result().unwrap();
        assert_eq!(
            activity,
            ScorpActivity {
                current_ordinary_with_items: current.max(0) * 100
                    - activity.ledger.risk_allowed[K1LossClass::Ordinary].get(),
                current_section1231: 0,
                characters: None,
                passive: passive * 100,
                nonpassive: nonpassive * 100,
                prior_released: released * 100,
                prior_pool: (prior - released) * 100,
                ledger: activity.ledger,
                items: Default::default(),
                combined: crate::scorp::ScorpCombined {
                    passive: passive * 100,
                    nonpassive: nonpassive * 100,
                    prior_released: released * 100,
                    prior_pool: (prior - released) * 100,
                    characters: None,
                },
            }
        );
    }
}

#[test]
fn former_passive_scorp_remaining_loss_can_use_other_passive_income() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: nonpassive_scorp(10_000, 15_000) }));
    let rental_id = d.next_id;
    add(&mut d, income(FederalIncome::Rental { data: rental(13_000, 0) }));
    assert_eq!(restart(d.clone()).draft, d);
    let r = estimate(&d);
    passive_note(&r, id, -3_000, 2_000);
    passive_note(&r, rental_id, 3_000, 0);
    assert!(r.review_notes.iter().any(|n| n.contains("nonpassive net after own prior passive loss: $0; prior passive loss absorbed by own income: $10000.")));
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.contains("nonpassive net after own prior passive loss: $0;"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_SCORP_REGULAR_NONPASSIVE_NET");
    assert_eq!(
        note.args,
        vec![MessageArg::Number(id), MessageArg::usd_dollars(0), MessageArg::usd_dollars(10_000)]
    );
    assert!(r.tax.is_none());
}
#[test]
fn retained_at_risk_nondeductible_costs_remain_separate_from_deductible_losses() {
    let mut source = sole_qbi_loss(0, 5_000, 0);
    source.risk.prior_loss = 925_000;
    source.risk.prior_nondeductible = 75_000;
    source.loss_qbi.at_risk = vec![loss_year(2025, 9_250, 9_250, 9_250, 9_250)];
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
            crate::kernel::ratio::dollars(risk.nondeductible_used.get())
        ),
        (4_625, 375)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Ordinary].get()),
            crate::kernel::ratio::dollars(risk.nondeductible_suspended.get())
        ),
        (4_625, 375)
    );
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 55_375);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 4_625);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn property_qbi_uses_its_own_basis_risk_and_passive_buckets() {
    let mut source = passive_property_scorp(-30_000, -40_000);
    source.opening = 3_500_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_400_000, profit: 0, loss: 0 };
    source.property_qbi = property_qbi(20_000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, false).qbi_statement;
    source.amt = Some(amt_records(&source));
    let qbi = &source.property_qbi;
    let basis = source.basis_result().unwrap();
    let basis_qbi = qbi.calculate(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(basis_qbi.amount.get()),
            crate::kernel::ratio::dollars(basis_qbi.qbi.get())
        ),
        (20_000, 10_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(basis_qbi.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(basis_qbi.remaining[0].remaining_qbi.get())
        ),
        (20_000, 10_000)
    );
    let risk = qbi.after_at_risk(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.amount.get()),
            crate::kernel::ratio::dollars(risk.qbi.get())
        ),
        (8_000, 4_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.remaining[0].original.get()),
            crate::kernel::ratio::dollars(risk.remaining[0].original_qbi.get())
        ),
        (12_000, 6_000)
    );
    // The shared passive pool uses $7k of other profit against $6k ordinary
    // and $8k property losses: property suspension is $4k, not the $7k total.
    let passive = qbi.after_passive(&source, k1_amount(4_000)).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(passive.amount.get()),
            crate::kernel::ratio::dollars(passive.qbi.get())
        ),
        (4_000, 2_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(passive.remaining[0].original.get()),
            crate::kernel::ratio::dollars(passive.remaining[0].original_qbi.get())
        ),
        (4_000, 2_000)
    );
    let (d, id) = passive_property_pool(source, 7_000);
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!("S-corporation document {id}, property loss passing passive limits: $4000.00; tracked QBI portion $2000.00."))));
    let message = r
        .review_notes
        .iter()
        .map(|note| &note.display)
        .find(|message| message.key == "NOTE_SCORP_QBI_PROPERTY_PASSIVE_RELEASE")
        .expect("localized property QBI release");
    assert_eq!(
        message.args,
        vec![
            crate::MessageArg::Number(id),
            crate::MessageArg::Currency {
                currency_code: "USD".into(),
                scale: 2,
                minor_units: 400_000
            },
            crate::MessageArg::Currency {
                currency_code: "USD".into(),
                scale: 2,
                minor_units: 200_000
            },
        ]
    );
    assert!(r.tax.is_none());
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn formerly_passive_partner_splits_earned_loss_release_without_overlap() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(1000, true);
    partner.regular.passive_loss = 300000;
    partner.alternative = Some(partner.regular.clone());
    partner.se_history = PartnerSeHistory { basis: 0, risk: 0, passive: 150000 };
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    add(&mut d, income(FederalIncome::Scorp { data: complete_ordinary_scorp(1000, false) }));
    let mut report = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut report);
    assert!(pool.inputs_complete, "{:?}", report.issues);
    assert_eq!(pool.income, 0);
    assert_eq!(pool.partners[0].nonpassive, 0); // Own $1,000 profit releases $1,000 prior loss.
    assert_eq!(crate::kernel::ratio::dollars(pool.partners[0].suspended.get()), 1000);
    assert_eq!(pool.partner_se[0].1.earnings, -1000); // Half of the total $2,000 released.
    assert_eq!(pool.partner_se[0].1.passive_earnings, -500); // Only other-income release is passive.
    assert_eq!(pool.eic_passive_income(), 500);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unit19_scorp_loan_sources_reuse_calendar_repayment_and_collection_checks() {
    let formal = ScorpLoan {
        kind: ScorpLoanKind::FormalNote,
        repayment_term: ScorpNoteHolding::Single { term: GainTerm::Long },
        principal: 100_000,
        basis: 80_000,
        advances: 20_000,
        repayments: 50_000,
    };
    assert!(formal.source_valid());
    let mut dated = formal.clone();
    dated.advances = 0;
    dated.repayment_term = ScorpNoteHolding::Dated {
        advanced: 20250101,
        payments: vec![
            ScorpNoteRepayment { date: 20260101, principal: 20_000 },
            ScorpNoteRepayment { date: 20260702, principal: 30_000 },
        ],
    };
    assert!(dated.source_valid());
    for date in [20260230, 20270101] {
        let mut invalid = dated.clone();
        let ScorpNoteHolding::Dated { payments, .. } = &mut invalid.repayment_term else {
            unreachable!()
        };
        payments[0].date = date;
        assert!(!invalid.source_valid());
    }
    // 2026-10-16 D-2026-10-15-13: repayment count is unrestricted;
    // the actual positive principal sum, not an arbitrary row cap, decides.
    let mut many = dated.clone();
    many.repayments = 101;
    many.repayment_term = ScorpNoteHolding::Dated {
        advanced: 20250101,
        payments: vec![ScorpNoteRepayment { date: 20260101, principal: 1 }; 101],
    };
    assert!(many.source_valid());
    let mut overflow = many.clone();
    overflow.repayment_term = ScorpNoteHolding::Dated {
        advanced: 20250101,
        payments: vec![ScorpNoteRepayment { date: 20260101, principal: i64::MAX }; 2],
    };
    assert!(!overflow.source_valid());
    let mut negative = many.clone();
    let ScorpNoteHolding::Dated { payments, .. } = &mut negative.repayment_term else {
        unreachable!()
    };
    payments[0].principal = -1;
    assert!(!negative.source_valid());
    let mut mismatch = dated.clone();
    mismatch.repayments -= 1;
    assert!(!mismatch.source_valid());
    for (basis, repayments, advances) in
        [(100_001, 50_000, 20_000), (80_000, 120_001, 20_000), (80_000, 50_000, -1)]
    {
        let mut invalid = formal.clone();
        invalid.basis = basis;
        invalid.repayments = repayments;
        invalid.advances = advances;
        assert!(!invalid.source_valid());
    }
    let mut unclassified = formal.clone();
    unclassified.repayment_term = ScorpNoteHolding::NoRepayment;
    assert!(!unclassified.source_valid());
    let open = ScorpLoan {
        kind: ScorpLoanKind::OpenAccount,
        repayment_term: ScorpNoteHolding::NoRepayment,
        principal: 2_500_000,
        basis: 2_500_000,
        advances: 0,
        repayments: 0,
    };
    assert!(open.source_valid());
    let mut over = open.clone();
    over.principal += 1;
    assert!(!over.source_valid());
    assert!(ScorpLoan::sources_valid([&formal, &dated, &open].into_iter()));
    assert!(!ScorpLoan::sources_valid([&open, &open].into_iter()));
    assert!(ScorpLoan::sources_valid(core::iter::repeat_n(&formal, 100)));
    assert!(!ScorpLoan::sources_valid(core::iter::repeat_n(&formal, 101)));
    assert!(ScorpLoan::sources_valid(core::iter::empty()));
}

#[test]
fn unit19_scorp_single_uses_shared_loan_source_checks_without_amt_substitution() {
    let mut source = complete_ordinary_scorp(10_000, true);
    source.loans = vec![ScorpLoan {
        kind: ScorpLoanKind::FormalNote,
        repayment_term: ScorpNoteHolding::Single { term: GainTerm::Long },
        principal: 100_000,
        basis: 80_000,
        advances: 20_000,
        repayments: 50_000,
    }];
    source.amt = Some(amt_records(&source));
    assert!(source.valid());
    source.amt.as_mut().unwrap().loans[0].basis = 100_001;
    assert!(!source.valid());
    source.amt = None;
    assert!(source.valid());
    assert!(source.amt.is_none());
    source.loans[0].basis = 100_001;
    assert!(!source.valid());
}

// Unit19 assigned PTP PAL source seam. All money passed to PAL is the
// existing annual form-dollar stage after basis/risk and activity character
// classification. It is not a cumulative-period or source-cent oracle.
fn unit19_ptp_pal_component(
    character: crate::PtpPalCharacter,
    current: i64,
    prior: i64,
    nonpassive: i64,
) -> crate::PtpPalComponent {
    crate::PtpPalComponent {
        character,
        passive_current_cents: current * 100,
        prior_passive_loss_cents: prior * 100,
        nonpassive_cents: nonpassive * 100,
    }
}
fn unit19_ptp_pal_179(active: bool, elected: i64) -> (FederalDraft, crate::Ptp179PalInput) {
    let mut draft = ptp179_draft(20_000, elected);
    if let FederalIncome::Partnership { data } = &mut draft.income.last_mut().unwrap().income {
        data.participation = PartnerParticipation::Limited {
            minutes: if active { 30_001 } else { 0 },
            prior_ten: 0,
            personal_service: false,
            service_years: 0,
        };
    }
    let supplied = selected_ptp179(
        &draft,
        Section179IncomeSystem::Regular,
        2,
        (if active { 20_000 } else { 0 }) * 100,
    )
    .unwrap()
    .ptp_section179_pal_input(2)
    .cloned()
    .unwrap();
    assert_eq!(supplied.active, active);
    (draft, supplied)
}

#[test]
fn unit19_ptp_pal_published_same_ptp_property_gain_keeps_ordinary_loss_character() {
    use crate::{PtpPalActivity as A, PtpPalCharacter as C};
    let (_, supplied) = unit19_ptp_pal_179(false, 0);
    // i8582 Special Instructions for PTPs: ScheduleE loss12000 (current
    // plus prior) and Form4797 gain7200 admit7200 of loss and carry4800.
    let source = A {
        k1: 2,
        source: 2,
        components: vec![
            unit19_ptp_pal_component(C::Ordinary, -8_000, 4_000, 0),
            unit19_ptp_pal_component(C::Section1231, 7_200, 0, 0),
        ],
    };
    let result = crate::ptp_passive_source(2, false, &[source], &supplied).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(result.net_gain_cents),
            crate::kernel::ratio::dollars(result.suspended_cents)
        ),
        (0, 4_800)
    );
    assert_eq!(
        crate::kernel::ratio::dollars(result.components[0].allowed_passive_net_cents),
        -7_200
    );
    assert_eq!(crate::kernel::ratio::dollars(result.components[0].suspended_cents), 4_800);
    assert_eq!(
        crate::kernel::ratio::dollars(result.components[0].original.prior_passive_loss_cents),
        4_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(result.components[1].allowed_passive_net_cents),
        7_200
    );
    assert_eq!(crate::kernel::ratio::dollars(result.components[1].suspended_cents), 0);
}

#[test]
fn unit19_ptp_pal_allocates_activity_before_form_and_never_borrows_another_ptp() {
    use crate::{PtpPalActivity as A, PtpPalCharacter as C};
    let (_, supplied) = unit19_ptp_pal_179(false, 0);
    let mut activities = vec![
        A {
            k1: 2,
            source: 21,
            components: vec![
                unit19_ptp_pal_component(C::Ordinary, 10_000, 0, 0),
                unit19_ptp_pal_component(C::Section1231, -6_000, 0, 0),
            ],
        },
        A {
            k1: 2,
            source: 22,
            components: vec![unit19_ptp_pal_component(C::Ordinary, -6_000, 4_000, 0)],
        },
    ];
    let result = crate::ptp_passive_source(2, false, &activities, &supplied).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(result.suspended_cents), 6_000);
    assert_eq!(
        result
            .components
            .iter()
            .map(|row| crate::kernel::ratio::dollars(row.suspended_cents))
            .collect::<Vec<_>>(),
        vec![0, 0, 6_000]
    );
    assert_eq!(
        result
            .components
            .iter()
            .map(|row| crate::kernel::ratio::dollars(row.allowed_passive_net_cents))
            .collect::<Vec<_>>(),
        vec![10_000, -6_000, -4_000]
    );
    activities.reverse();
    activities[1].components.reverse();
    assert_eq!(crate::ptp_passive_source(2, false, &activities, &supplied).unwrap(), result);
    let mut other = supplied.clone();
    other.k1 = 3;
    let other_result = crate::ptp_passive_source(
        3,
        false,
        &[A {
            k1: 3,
            source: 31,
            components: vec![unit19_ptp_pal_component(C::Ordinary, 20_000, 0, 0)],
        }],
        &other,
    )
    .unwrap();
    assert_eq!(crate::kernel::ratio::dollars(other_result.net_gain_cents), 20_000);
    assert_eq!(crate::kernel::ratio::dollars(result.suspended_cents), 6_000);
}

#[test]
fn unit19_ptp_pal_consumes_only_actual_allowed_179_and_preserves_income_carry() {
    use crate::{PtpPalActivity as A, PtpPalCharacter as C};
    for active in [false, true] {
        let (draft, supplied) = unit19_ptp_pal_179(active, 100_000);
        let deduction = crate::kernel::ratio::dollars(supplied.allowed_cents);
        assert!(supplied.business_income_carry_cents > 0);
        let activities = vec![A {
            k1: 2,
            source: 2,
            components: vec![
                unit19_ptp_pal_component(
                    C::Ordinary,
                    if active { 0 } else { 20_000 },
                    0,
                    if active { 20_000 } else { 0 },
                ),
                unit19_ptp_pal_component(
                    C::Section179,
                    if active { 0 } else { -deduction },
                    1_000,
                    if active { -deduction } else { 0 },
                ),
            ],
        }];
        let result = crate::ptp_passive_source(2, false, &activities, &supplied).unwrap();
        assert_eq!(result.business_income_carry_cents, supplied.business_income_carry_cents);
        assert_eq!(
            crate::kernel::ratio::dollars(result.components[1].original.prior_passive_loss_cents),
            1_000
        );
        assert_eq!(
            crate::kernel::ratio::dollars(result.suspended_cents),
            if active { 1_000 } else { deduction - 19_000 }
        );
        let reopened = restart(draft).draft;
        let supplied_again = selected_ptp179(
            &reopened,
            Section179IncomeSystem::Regular,
            2,
            (if active { 20_000 } else { 0 }) * 100,
        )
        .unwrap()
        .ptp_section179_pal_input(2)
        .cloned()
        .unwrap();
        assert_eq!(
            crate::ptp_passive_source(2, false, &activities, &supplied_again).unwrap(),
            result
        );
        let mut wrong = supplied.clone();
        wrong.alternative = true;
        assert_eq!(
            crate::ptp_passive_source(2, false, &activities, &wrong),
            Err(crate::PtpPalIssue::Section179SourceMismatch)
        );
        let mut forged = activities.clone();
        forged[0].components[1].passive_current_cents -= 1;
        assert_eq!(
            crate::ptp_passive_source(2, false, &forged, &supplied),
            Err(crate::PtpPalIssue::Section179SourceMismatch)
        );
    }
}

#[test]
fn unit19_ptp_pal_keeps_source_cent_offsets_inside_the_same_ptp() {
    use crate::{PtpPalActivity as A, PtpPalCharacter as C};
    let (_, supplied) = unit19_ptp_pal_179(false, 0);
    let mut profit = unit19_ptp_pal_component(C::Ordinary, 0, 0, 0);
    profit.passive_current_cents = 49;
    let mut loss = unit19_ptp_pal_component(C::Section1231, 0, 0, 0);
    loss.passive_current_cents = -149;
    let same = A { k1: 2, source: 21, components: vec![profit.clone(), loss.clone()] };
    let result = crate::ptp_passive_source(2, false, &[same], &supplied).unwrap();
    assert_eq!((result.net_gain_cents, result.suspended_cents), (0, 100));
    assert_eq!(result.components[1].allowed_passive_net_cents, -49);
    let losing = A { k1: 2, source: 21, components: vec![loss] };
    let isolated = crate::ptp_passive_source(2, false, &[losing], &supplied).unwrap();
    let mut other = supplied.clone();
    other.k1 = 3;
    let earning = A { k1: 3, source: 31, components: vec![profit] };
    let other_result = crate::ptp_passive_source(3, false, &[earning], &other).unwrap();
    assert_eq!((isolated.net_gain_cents, isolated.suspended_cents), (0, 149));
    assert_eq!((other_result.net_gain_cents, other_result.suspended_cents), (49, 0));
}

#[test]
fn unit19_ptp_pal_rejects_duplicate_source_character_and_portfolio_inputs() {
    use crate::{PtpPalActivity as A, PtpPalCharacter as C};
    let (_, supplied) = unit19_ptp_pal_179(false, 0);
    let valid = A {
        k1: 2,
        source: 2,
        components: vec![unit19_ptp_pal_component(C::Ordinary, -1_000, 0, 0)],
    };
    assert_eq!(
        crate::ptp_passive_source(2, false, &[valid.clone(), valid.clone()], &supplied),
        Err(crate::PtpPalIssue::InvalidSources)
    );
    let mut duplicate = valid.clone();
    duplicate.components.push(duplicate.components[0].clone());
    assert_eq!(
        crate::ptp_passive_source(2, false, &[duplicate], &supplied),
        Err(crate::PtpPalIssue::InvalidSources)
    );
    for character in [C::Charity, C::ForeignTaxes, C::InvestmentInterest] {
        let mut wrong = valid.clone();
        wrong.components[0].character = character;
        assert_eq!(
            crate::ptp_passive_source(2, false, &[wrong], &supplied),
            Err(crate::PtpPalIssue::InvalidSources)
        );
    }
    assert_eq!(
        crate::ptp_passive_source(3, false, &[valid], &supplied),
        Err(crate::PtpPalIssue::Section179SourceMismatch)
    );
}

#[test]
fn unit19_ptp_pal_regular_and_amt_keep_separate_source_histories() {
    use crate::{PtpPalActivity as A, PtpPalCharacter as C};
    let (mut draft, regular) = unit19_ptp_pal_179(false, 0);
    // Explicit selected AMT zero is a supplied record, not the regular
    // helper's inferred alternative result.
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    let allocation = selected_ptp179(&draft, Section179IncomeSystem::Alternative, 2, 0).unwrap();
    let alternative = allocation.ptp_section179_pal_input(2).unwrap().clone();
    let sources = |prior| {
        vec![A {
            k1: 2,
            source: 2,
            components: vec![unit19_ptp_pal_component(C::Ordinary, 1_000, prior, 0)],
        }]
    };
    let regular = crate::ptp_passive_source(2, false, &sources(3_000), &regular).unwrap();
    let alternative = crate::ptp_passive_source(2, true, &sources(500), &alternative).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.net_gain_cents),
            crate::kernel::ratio::dollars(regular.suspended_cents)
        ),
        (0, 2_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(alternative.net_gain_cents),
            crate::kernel::ratio::dollars(alternative.suspended_cents)
        ),
        (500, 0)
    );
    let (_, supplied) = unit19_ptp_pal_179(false, 0);
    let mut crossed = sources(3_000);
    crossed[0].k1 = 3;
    assert_eq!(
        crate::ptp_passive_source(2, false, &crossed, &supplied),
        Err(crate::PtpPalIssue::InvalidSources)
    );
}

#[test]
fn unit19_risk_source_intrinsic_money_bounds_and_recapture_do_not_need_a_parent() {
    let scorp_fields: [fn(&mut ScorpRisk) -> &mut i64; 8] = [
        |v| &mut v.excluded_funding,
        |v| &mut v.excluded_returns,
        |v| &mut v.protected,
        |v| &mut v.released,
        |v| &mut v.prior_loss,
        |v| &mut v.prior_nondeductible,
        |v| &mut v.deducted,
        |v| &mut v.recaptured,
    ];
    let partner_fields: [fn(&mut PartnerRiskRecord) -> &mut i64; 10] = [
        |v| &mut v.opening_debt,
        |v| &mut v.ending_debt,
        |v| &mut v.excluded_contributions,
        |v| &mut v.excluded_distributions,
        |v| &mut v.protected,
        |v| &mut v.released,
        |v| &mut v.prior_loss,
        |v| &mut v.prior_nondeductible,
        |v| &mut v.deducted,
        |v| &mut v.recaptured,
    ];
    let scorp = complete_ordinary_scorp(1_000, true).risk;
    let partner = ordinary_partner_source(1_000, true).regular.risk;
    assert!(scorp.source_valid());
    assert!(partner.source_valid());
    for bad in [-1, 100_000_000_001] {
        for field in scorp_fields {
            let mut record = scorp.clone();
            *field(&mut record) = bad;
            assert!(!record.source_valid());
        }
        for field in partner_fields {
            let mut record = partner.clone();
            *field(&mut record) = bad;
            assert!(!record.source_valid());
        }
    }
    let mut scorp = scorp;
    let mut partner = partner;
    scorp.deducted = 100;
    scorp.recaptured = 100;
    partner.deducted = 100;
    partner.recaptured = 100;
    assert!(scorp.source_valid());
    assert!(partner.source_valid());
    scorp.recaptured = 101;
    partner.recaptured = 101;
    assert!(!scorp.source_valid());
    assert!(!partner.source_valid());
}

#[test]
fn unit19_risk_source_detailed_history_keeps_each_native_owners_exact_boundary() {
    let mut scorp = complete_ordinary_scorp(1_000, true).risk;
    let mut partner = ordinary_partner_source(1_000, true).regular.risk;
    for (capacity, profit, loss, s_valid, p_valid) in [
        (100, 0, 100, true, true),
        (100, 0, 101, false, false), // 2026-10-19: source history retains cents.
        (100, 1, 1, false, false),
        (10_000, 0, 10_100, false, false),
        (10_000, 100, 100, false, false),
        (-1, 0, 0, false, false),
        (100_000_000_001, 0, 0, false, false),
        (100, -1, 0, false, false),
        (100, 100_000_000_001, 0, false, false),
        (100, 0, -1, false, false),
        (100, 0, 100_000_000_001, false, false),
    ] {
        scorp.start = ScorpRiskStart::Detailed { capacity, profit, loss };
        partner.start = scorp.start.clone();
        assert_eq!(scorp.source_valid(), s_valid, "Scorp {capacity}/{profit}/{loss}");
        assert_eq!(partner.source_valid(), p_valid, "Partner {capacity}/{profit}/{loss}");
    }
}

#[test]
fn unit19_risk_source_unknown_complete_and_amt_facts_do_not_establish_contextual_allowances() {
    let mut owner = complete_ordinary_scorp(1_000, true);
    owner.risk.complete = false;
    assert!(owner.risk.source_valid());
    assert_eq!(
        owner.risk.calculate(&owner, &owner.basis_result().unwrap()),
        Err(crate::scorp_risk::ScorpRiskError::FinancingUnresolved)
    );
    owner.risk.complete = true;
    owner.risk.excluded_funding = owner.additions + 100;
    assert!(owner.risk.source_valid());
    assert!(!owner.risk.valid(&owner));
    let mut partner = ordinary_partner_source(1_000, true);
    partner.regular.risk.complete = false;
    assert!(partner.regular.risk.source_valid());
    let record = &partner.regular;
    assert!(
        record
            .risk
            .calculate(&record.basis, record.ordinary, record.exempt, record.nondeductible)
            .is_err()
    );
    partner.regular.risk.complete = true;
    partner.regular.risk.opening_debt = partner.regular.basis.opening_liabilities + 100;
    assert!(partner.regular.risk.source_valid());
    let record = &partner.regular;
    assert!(
        record
            .risk
            .calculate(&record.basis, record.ordinary, record.exempt, record.nondeductible)
            .is_err()
    );
    let regular = owner.risk.clone();
    let absent: Option<ScorpRisk> = None;
    assert!(absent.as_ref().is_none_or(ScorpRisk::source_valid));
    let mut invalid = regular.clone();
    invalid.deducted = -1;
    assert!(regular.source_valid());
    assert!(!Some(invalid).as_ref().is_none_or(ScorpRisk::source_valid));
}

#[test]
fn unit19_risk_source_single_partner_preserves_specific_history_errors_and_native_cents() {
    let mut source = ordinary_partner_source(1_000, true).regular;
    for (capacity, profit, loss, expected) in [
        (100, 0, 100, None),
        (
            100,
            0,
            101,
            Some("ISSUE_PARTNERSHIP_THE_RETAINED_AT_RISK_HISTORY_IS_INCONSISTENT_0CD4F902"),
        ),
        (-1, 0, 0, Some("ISSUE_PARTNERSHIP_REVIEW_THE_RETAINED_FORM_6198_AMOUNTS_34CD8625")),
        (
            10_000,
            100,
            100,
            Some("ISSUE_PARTNERSHIP_THE_RETAINED_AT_RISK_HISTORY_IS_INCONSISTENT_0CD4F902"),
        ),
    ] {
        source.risk.start = ScorpRiskStart::Detailed { capacity, profit, loss };
        let actual = source.risk.calculate(
            &source.basis,
            source.ordinary,
            source.exempt,
            source.nondeductible,
        );
        assert_eq!(actual.as_ref().err().map(|error| error.0), expected);
    }
    source.risk.complete = false;
    source.risk.start = ScorpRiskStart::Detailed { capacity: -1, profit: 0, loss: 0 };
    assert_eq!(
        source
            .risk
            .calculate(&source.basis, source.ordinary, source.exempt, source.nondeductible)
            .unwrap_err()
            .0,
        "ISSUE_PARTNERSHIP_RECONCILE_PARTNER_FINANCING_AND_ADDITIONAL_AT_RISK_B169E3B9"
    );
}

#[test]
fn unit19_risk_source_new_and_unknown_history_keep_existing_single_context() {
    let mut scorp = complete_ordinary_scorp(1_000, true);
    scorp.risk.start = ScorpRiskStart::New;
    assert!(scorp.risk.source_valid());
    assert!(!scorp.risk.valid(&scorp)); // Actual continuing stock/loan context.
    let mut source = ordinary_partner_source(1_000, true).regular;
    source.risk.start = ScorpRiskStart::New;
    assert!(source.risk.source_valid());
    assert_eq!(
        source
            .risk
            .calculate(&source.basis, source.ordinary, source.exempt, source.nondeductible)
            .unwrap_err()
            .0,
        "ISSUE_PARTNERSHIP_A_NEW_ACTIVITY_CANNOT_CONTAIN_PRIOR_822BC4EF"
    );
    source.risk.start = ScorpRiskStart::Unknown;
    assert!(source.risk.source_valid());
    let actual = source
        .risk
        .calculate(&source.basis, source.ordinary, source.exempt, source.nondeductible)
        .unwrap();
    assert_eq!((actual.risk_investment, actual.risk_end), (None, None));
}
