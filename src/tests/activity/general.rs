// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn reit_and_qbi_history_validate_source_bounds_and_unknown_history() {
    let mut dividend = reit_income(100, 101, true);
    assert!(crate::federal_rules::validate_income_amount(&dividend).is_err());
    let FederalIncome::Dividend { data } = &mut dividend else { unreachable!() };
    let mut data = edit_dividend(data);
    data.section199a = ReitDividends::Reported { amount: 5000, eligible: Some(true) };
    data.qualified = 5001;
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&dividend).is_err());
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().qbi_history =
        QbiHistory { business: 100, reit_ptp: 0, injured_owners: None };
    supported(&d);
}

#[test]
fn passive_pool_rounding_conserves_carryforwards_and_rejects_invalid_amounts() {
    use crate::tests::support::loss_limits::{PassiveActivity as A, passive_suspensions};
    let pool = [
        A { key: (), net: -3, rental_allowance: true, ineligible_prior: 0 },
        A { key: (), net: -7, rental_allowance: false, ineligible_prior: 0 },
    ];
    for profit in 0..15 {
        for allowance in 0..7 {
            let r = passive_suspensions(&pool, profit, allowance).unwrap();
            let special = allowance.min(3).min((10 - profit).max(0));
            assert_eq!(r.iter().sum::<i64>(), (10 - profit - special).max(0));
            assert!((0..=3).contains(&r[0]) && (0..=7).contains(&r[1]));
        }
    }
    assert!(passive_suspensions(&pool, -1, 0).is_none());
    assert!(
        passive_suspensions(
            &[A { key: (), net: i64::MIN, rental_allowance: false, ineligible_prior: 0 }],
            0,
            0
        )
        .is_none()
    );
}

#[test]
fn former_passive_qbi_counts_own_income_release_and_other_passive_income_once() {
    let mut source = nonpassive_scorp(10_000, 15_000);
    source.loss_qbi = ScorpLossQbi {
        current: 0,
        history: vec![],
        at_risk: vec![],
        passive: vec![loss_year(2020, 15_000, 7_500, 15_000, 7_500)],
    };
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source.clone()) }));
    add(&mut d, income(FederalIncome::Rental { data: rental(13_000, 0) }));
    let r = estimate(&d);
    scorp_note(&r, id, "QBI loss passing passive limits", 6_500);
    let release = source
        .loss_qbi
        .after_passive(&source, crate::tests::support::builders::k1_amount(2_000))
        .unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.amount.get()),
            crate::kernel::ratio::dollars(release.qbi.get())
        ),
        (13_000, 6_500)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(release.remaining[0].remaining_qbi.get())
        ),
        (2_000, 1_000)
    );
}

#[test]
fn passive_qbi_history_must_reconcile_to_prior_passive_losses() {
    // An empty list states no prior passive loss; it cannot stand for the
    // $2,000 carried on the passive record.
    let mut source = passive_qbi_source();
    source.loss_qbi.passive = vec![];
    assert!(!source.valid());
    assert!(
        source
            .loss_qbi
            .after_passive(&source, crate::tests::support::builders::k1_amount(0))
            .is_err()
    );
    source.loss_qbi.passive = vec![loss_year(2017, 2_000, 0, 2_000, 0)];
    assert!(source.valid());
    assert!(
        source
            .loss_qbi
            .after_passive(&source, crate::tests::support::builders::k1_amount(0))
            .is_ok()
    );
}

#[test]
fn trading_qbi_positive_income_still_requires_actual_participation_facts() {
    let mut source = nonpassive_scorp(1_000, 0);
    source.trade_kind = ScorpTradeKind::FinancialTrading;
    source.participation = None;
    let statement = ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 100_000,
        wages: 0,
        property: 0,
        service: true,
        complete: true,
    };
    assert!(source.activity_result().is_ok());
    assert!(
        statement.activities(&source, crate::tests::support::builders::k1_amount(0), 0).is_err()
    );
    source.participation = Some(participation_facts());
    assert!(
        !statement.activities(&source, crate::tests::support::builders::k1_amount(0), 0).unwrap()
            [0]
        .active
    );
    source.participation.as_mut().unwrap().minutes = 30_001;
    assert!(
        statement.activities(&source, crate::tests::support::builders::k1_amount(0), 0).unwrap()[0]
            .active
    );
}
#[test]
fn former_passive_carryforward_output_preserves_original_fixed_percentage() {
    let mut source = complete_ordinary_scorp(4_000, true);
    source.passive.loss = 1_000_000;
    source.loss_qbi = ScorpLossQbi {
        current: 0,
        history: vec![],
        at_risk: vec![],
        passive: vec![loss_year(2020, 20_000, 10_000, 10_000, 5_000)],
    };
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    let r = supported(&d);
    assert_eq!(r.review_notes.iter().filter(|n| n.contains(
        "2020 passive-loss carryforward: $6000.00; QBI portion: $3000.00. Original suspended loss: $20000.00; original QBI portion: $10000.00."
    )).count(), 1);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn passive_character_allocation_offsets_income_within_its_own_category() {
    use crate::tests::support::loss_limits::passive_character_suspensions as split;
    // $4,000 ordinary profit absorbs $4,000 of its $6,000 prior loss.
    // Together with a $6,000 property loss, the weights are 2:6, not 6:6.
    assert_eq!(split(&[-2_000, -6_000], 4_000), Some(vec![1_000, 3_000]));
    // A positive property category must not receive suspended ordinary loss.
    assert_eq!(split(&[-5_000, 1_000], 4_000), Some(vec![4_000, 0]));
    assert_eq!(split(&[5_000, -3_000], 0), Some(vec![0, 0]));
}

#[test]
fn passive_character_allocation_validates_pool_total_and_overflow() {
    use crate::tests::support::loss_limits::passive_character_suspensions as split;
    assert_eq!(split(&[], 0), Some(vec![]));
    assert_eq!(split(&[0, 0], 0), Some(vec![0, 0]));
    assert!(split(&[-1, 2], 1).is_none());
    assert!(split(&[-5, 1], 5).is_none());
    assert!(split(&[-5], -1).is_none());
    assert!(split(&[i64::MIN], 0).is_none());
    assert!(split(&[i64::MAX, 1], 0).is_none());
    // Signed sum fits, but absolute loss weights do not.
    assert!(split(&[-i64::MAX, i64::MAX, -2], 1).is_none());
}

#[test]
fn passive_activity_item_cuts_use_one_cent_character_allocation() {
    use crate::k1_items::ActivityItems;
    use crate::kernel::carry::Cents;
    // The positive debt item offsets the combined loss, never receiving a
    // suspended deduction. Every actual cent stays in its reporting class.
    let cuts = ActivityItems::cuts_in_units::<Cents>([-100, -100, -100, 0, 50], 151).unwrap();
    assert_eq!(cuts, [50, 51, 50, 0]);
    assert_eq!(cuts.iter().sum::<i64>(), 151);
    assert!(ActivityItems::cuts_in_units::<Cents>([-100, 0, 0, 0, 50], 51).is_none());
}

#[test]
fn former_passive_single_character_history_uses_net_own_income_then_other_passive_income() {
    for property_history in [false, true] {
        let source = if property_history {
            former_passive_property_scorp(10_000, -5_000, 0, 10_000)
        } else {
            former_passive_property_scorp(-5_000, 10_000, 10_000, 0)
        };
        let activity = source.pool_activity_result().unwrap();
        assert_eq!(crate::kernel::ratio::dollars(activity.passive), -5_000);
        assert_eq!(activity.nonpassive, 0);
        assert_eq!(crate::kernel::ratio::dollars(activity.prior_released), 5_000);
        assert_eq!(
            activity.characters.unwrap().prior_released.map(crate::kernel::ratio::dollars),
            if property_history { [0, 5_000] } else { [5_000, 0] }
        );
        let (d, id) = passive_property_pool(source, 2_000);
        let mut r = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
        assert_eq!(pool.passive_loss, 3_000);
        if property_history {
            assert_eq!(pool.income, 12_000);
            assert_eq!(pool.scorp_niit_adjustment, -10_000);
            assert_eq!(pool.k1_section1231, vec![property_result(id, -12_000, 3_000, -10_000)]);
        } else {
            assert_eq!(pool.income, -10_000);
            assert_eq!(pool.scorp_niit_adjustment, 10_000);
            assert_eq!(pool.k1_section1231, vec![property_result(id, 10_000, 0, 10_000)]);
        }
        assert!(r.review_notes.iter().any(|n| n.contains("prior loss absorbed by own income:")));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn former_passive_partial_release_offsets_matching_character_first() {
    for (ordinary, property, released, remaining, nonpassive) in [
        (8_000, 2_000, [6_000, 4_000], [0, -10_000], [2_000, -2_000]),
        (2_000, 8_000, [2_000, 8_000], [-4_000, -6_000], [0, 0]),
        (0, 10_000, [0, 10_000], [-6_000, -4_000], [0, 0]),
    ] {
        let source = former_passive_property_scorp(ordinary, property, 6_000, 14_000);
        let activity = source.pool_activity_result().unwrap();
        let c = activity.characters.unwrap();
        assert_eq!(c.prior_released.map(crate::kernel::ratio::dollars), released);
        assert_eq!(c.passive.map(crate::kernel::ratio::dollars), remaining);
        assert_eq!(c.nonpassive.map(crate::kernel::ratio::dollars), nonpassive);
        assert_eq!(
            (
                crate::kernel::ratio::dollars(activity.passive),
                crate::kernel::ratio::dollars(activity.nonpassive)
            ),
            (-10_000, 0)
        );
    }
}

#[test]
fn former_passive_partial_release_then_other_income_keeps_character() {
    let source = former_passive_property_scorp(2_000, 8_000, 6_000, 14_000);
    let (d, id) = passive_property_pool(source, 5_000);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert!(pool.inputs_complete);
    assert_eq!(pool.passive_loss, 5_000);
    assert_eq!(pool.income, 3_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, -3_000, 3_000, 0)]);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn former_passive_partial_release_conserves_rounded_character_balances() {
    let source = former_passive_property_scorp(5, 0, 3, 7);
    let c = source.pool_activity_result().unwrap().characters.unwrap();
    assert_eq!(c.prior_released.map(crate::kernel::ratio::dollars), [3, 2]);
    assert_eq!(c.passive.map(crate::kernel::ratio::dollars), [0, -5]);
    assert_eq!(c.nonpassive.map(crate::kernel::ratio::dollars), [2, -2]);
    for i in 0..2 {
        assert_eq!(crate::kernel::ratio::dollars(c.passive[i] + c.nonpassive[i]), [5 - 3, -7][i]);
    }
}

#[test]
fn former_passive_mixed_sign_partial_release_is_symmetric() {
    for (ordinary, property, old_ord, old_prop, release, remaining, nonpassive) in [
        (-5_000, 10_000, 6_000, 14_000, [0, 5_000], [-6_000, -9_000], [-5_000, 5_000]),
        (10_000, -5_000, 14_000, 6_000, [5_000, 0], [-9_000, -6_000], [5_000, -5_000]),
        (10_000, -5_000, 4_000, 6_000, [4_000, 1_000], [0, -5_000], [6_000, -6_000]),
    ] {
        let source = former_passive_property_scorp(ordinary, property, old_ord, old_prop);
        let c = source.pool_activity_result().unwrap().characters.unwrap();
        assert_eq!(c.prior_released.map(crate::kernel::ratio::dollars), release);
        assert_eq!(c.passive.map(crate::kernel::ratio::dollars), remaining);
        assert_eq!(c.nonpassive.map(crate::kernel::ratio::dollars), nonpassive);
        assert_eq!(crate::kernel::ratio::dollars(c.prior_released.iter().sum::<i64>()), 5_000);
        assert_eq!(c.nonpassive.iter().sum::<i64>(), 0);
    }
}

#[test]
fn direct_participation_selected_tests_keep_strict_hours_and_actual_history() {
    use crate::participation::Participation::{No, Unavailable, Yes};
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    assert_eq!(F { hours: 500, test: T::Over500 }.individual(2026, false), Unavailable);
    assert_eq!(F { hours: 501, test: T::Over500 }.individual(2026, false), Yes);
    assert_eq!(F { hours: 501, test: T::None }.individual(2026, false), Unavailable);
    assert_eq!(F { hours: 1, test: T::SubstantiallyAll }.individual(2026, false), Yes);
    assert_eq!(
        F { hours: 100, test: T::Over100NoOneMore { other_max_hours: 100 } }
            .individual(2026, false),
        Unavailable
    );
    assert_eq!(
        F { hours: 101, test: T::Over100NoOneMore { other_max_hours: 101 } }
            .individual(2026, false),
        Yes
    );
    let mut historical =
        F { hours: 0, test: T::FiveOfTen { years: vec![2016, 2018, 2020, 2023, 2025] } };
    assert_eq!(historical.individual(2026, false), Yes);
    historical.test = T::FiveOfTen { years: vec![2015, 2018, 2020, 2023, 2025] };
    assert_eq!(historical.individual(2026, false), Unavailable);
    historical.test = T::FiveOfTen { years: vec![2016, 2018, 2020, 2025, 2025] };
    assert_eq!(historical.individual(2026, false), Unavailable);
    historical.test =
        T::PersonalServiceThree { years: vec![1990, 2000, 2001], personal_services: true };
    assert_eq!(historical.individual(2026, false), Yes);
    historical.test =
        T::PersonalServiceThree { years: vec![1990, 2000, 2001], personal_services: false };
    assert_eq!(historical.individual(2026, false), Unavailable);
    assert_eq!(F { hours: 0, test: T::None }.individual(2026, false), No);
}

#[test]
fn direct_participation_management_and_farmer_evidence_are_not_generic_exceptions() {
    use crate::participation::Participation::{No, Unavailable, Yes};
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    let mut facts = F {
        hours: 200,
        test: T::FactsCircumstances {
            regular_continuous_substantial: true,
            management_hours: 100,
            paid_other_manager: true,
            other_management_hours: 0,
        },
    };
    assert_eq!(facts.individual(2026, false), No);
    let T::FactsCircumstances { management_hours, .. } = &mut facts.test else { unreachable!() };
    *management_hours = 99;
    assert_eq!(facts.individual(2026, false), Yes);
    facts.test = T::RetiredDisabledFarmer {
        retired_or_disabled: 2020,
        qualifying_years: vec![2012, 2013, 2015, 2017, 2019],
    };
    assert_eq!(facts.individual(2026, true), Yes);
    assert_eq!(facts.individual(2026, false), Unavailable);
    facts.test = T::RetiredDisabledFarmer {
        retired_or_disabled: 2020,
        qualifying_years: vec![2011, 2013, 2015, 2017, 2019],
    };
    assert_eq!(facts.individual(2026, true), Unavailable);
    facts.test = T::SurvivingFarmer { special_estate_valuation: true, active_management: false };
    assert_eq!(facts.individual(2026, true), No);
    facts.test = T::SurvivingFarmer { special_estate_valuation: true, active_management: true };
    assert_eq!(facts.individual(2026, true), Yes);
}

#[test]
fn direct_participation_significant_hours_are_bound_to_actual_source_ids() {
    use crate::participation::Participation::{No, Yes};
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    let mut d = complete(FederalStatus::Single);
    for hours in [250, 250] {
        let mut source = income(cash_business(1000));
        let FederalIncome::Business { data } = &mut source.income else { unreachable!() };
        data.material_participation = F { hours, test: T::SignificantActivities };
        add(&mut d, source);
    }
    assert!(d.direct_participation_sources().iter().all(|source| source.result == No));
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.material_participation.hours = 251;
    let before = d.direct_participation_sources();
    assert!(before.iter().all(|source| source.result == Yes));
    d.income.reverse();
    let after = d.direct_participation_sources();
    assert!(before.iter().all(|source| after.contains(source)));
    assert_eq!(before.iter().map(|source| source.hours).sum::<u32>(), 501);
}

#[test]
fn direct_participation_external_unknown_only_blocks_unproved_combined_test() {
    use crate::participation::Participation::{Unavailable, Yes};
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(1000)));
    let direct_index = d.income.len() - 1;
    let id = d.income[direct_index].id;
    let FederalIncome::Business { data } = &mut d.income[direct_index].income else {
        unreachable!()
    };
    data.material_participation = F { hours: 200, test: T::SignificantActivities };
    let mut external = scorp_document();
    external.participation = None;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(external) }));
    assert_eq!(d.direct_participation_view().get(id), Unavailable);
    let FederalIncome::Business { data } = &mut d.income[direct_index].income else {
        unreachable!()
    };
    data.material_participation = F { hours: 501, test: T::Over500 };
    assert_eq!(d.direct_participation_view().get(id), Yes);
}

#[test]
fn direct_participation_combines_existing_k1_minutes_without_truncation() {
    use crate::participation::Participation::{No, Yes};
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(1000)));
    let direct_index = d.income.len() - 1;
    let id = d.income[direct_index].id;
    let FederalIncome::Business { data } = &mut d.income[direct_index].income else {
        unreachable!()
    };
    data.material_participation = F { hours: 250, test: T::SignificantActivities };
    let mut external = scorp_document();
    let mut facts = participation_facts();
    facts.minutes = 250 * 60;
    external.participation = Some(facts);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(external) }));
    assert_eq!(d.direct_participation_view().get(id), No);
    let external_index = d.income.len() - 1;
    let FederalIncome::Scorp { data } = &mut d.income[external_index].income else {
        unreachable!()
    };
    let mut data = edit_scorp(data);
    data.participation.as_mut().unwrap().minutes += 1;
    drop(data);
    assert_eq!(d.direct_participation_view().get(id), Yes);
}

#[test]
fn direct_participation_does_not_count_known_casual_gambling_as_a_trade() {
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    let mut d = complete(FederalStatus::Single);
    // 2026-10-24: Pub925 material-participation rules leave the ordinary
    // C passive, but current passive operating income remains reportable.
    let mut control = d.clone();
    add(&mut control, income(passive_business(1000)));
    let expected = supported(&control);
    for casual in [false, true] {
        let mut source = income(cash_business(1000));
        let FederalIncome::Business { data } = &mut source.income else { unreachable!() };
        data.material_participation = F { hours: 300, test: T::None };
        if casual {
            data.activity = BusinessActivity::Gambling {
                returns: 0,
                receipts: 100000,
                accounting: BusinessAccounting::Cash,
                losses: 0,
                withholding: 0,
                eligible: false,
            };
        }
        add(&mut d, source);
    }
    assert!(
        d.direct_participation_sources()
            .iter()
            .all(|source| source.result == crate::participation::Participation::No)
    );
    let result = supported(&d);
    assert_eq!((value(&result, "agi"), result.tax), (value(&expected, "agi"), expected.tax));
    assert!(
        result
            .review_notes
            .iter()
            .any(|note| note.display.key == "WARN_GAMBLING_BUSINESS_LEFT_OUT")
    );
}

#[test]
fn direct_participation_uses_accepted_hours_through_pruning_and_recursive_estimates() {
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    for base in
        [complete(FederalStatus::Single), repayment_comparison_draft(RepaymentMethod::Automatic)]
    {
        let (mut draft, direct) = direct_participation_with_omitted_k1(base);
        let expected = supported(&draft);
        let entry = draft.income.iter_mut().find(|entry| entry.id == direct).unwrap();
        let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
        data.material_participation = F { hours: 300, test: T::None };
        assert_eq!(
            draft.direct_participation_view().get(direct),
            crate::participation::Participation::Yes
        );
        let result = supported(&draft);
        assert_eq!(value(&result, &alloc::format!("business_{direct}_net_profit")), 1000);
        assert_eq!(
            (value(&result, "agi"), result.tax, result.payments, result.refund, result.balance),
            (
                value(&expected, "agi"),
                expected.tax,
                expected.payments,
                expected.refund,
                expected.balance
            )
        );
        assert!(
            result
                .review_notes
                .iter()
                .any(|note| note.display.key == "WARN_PARTICIPATION_LEFT_OUT")
        );
    }
}
