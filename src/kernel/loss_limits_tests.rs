use super::carry::{Amount, Cents, Dollars};
use super::loss_limits::*;
use super::source_fact::SourceFact;

#[test]
fn pool_offsets_income_then_limits_the_actual_special_allowance() {
    let activities = [
        PassiveActivity { key: 11, net: -6000, rental_allowance: true, ineligible_prior: 4000 },
        PassiveActivity { key: 29, net: -4000, rental_allowance: false, ineligible_prior: 0 },
        PassiveActivity { key: 7, net: 1000, rental_allowance: true, ineligible_prior: 0 },
    ];
    let SourceFact::Reported(pool) = passive_pool(SourceFact::Reported(PoolInput {
        activities: &activities,
        other_passive_income: Amount::<Dollars>::new(0).unwrap(),
        allowance: Amount::new(25_000).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        (pool.total_loss(), pool.special_allowance(), pool.allowed_loss(), pool.held_loss()),
        (10_000, 1000, 2000, 8000)
    );
    assert_eq!(pool.suspended(), [4444, 3556, 0]);
    assert_eq!(
        pool.settlements()
            .map(|v| (*v.key, v.net, v.suspended, v.allowed_net))
            .collect::<alloc::vec::Vec<_>>(),
        [(11, -6000, 4444, -1556), (29, -4000, 3556, -444), (7, 1000, 0, 1000)]
    );
}

#[test]
fn published_character_example_keeps_source_cents_and_native_dollars_distinct() {
    // Existing Form8582 PartIX example: activityI loss4000, activityII net870.
    let activities = [
        PassiveActivity { key: 1, net: -4000, rental_allowance: false, ineligible_prior: 0 },
        PassiveActivity { key: 2, net: 870, rental_allowance: false, ineligible_prior: 0 },
    ];
    let SourceFact::Reported(pool) = passive_pool(SourceFact::Reported(PoolInput {
        activities: &activities,
        other_passive_income: Amount::<Dollars>::new(0).unwrap(),
        allowance: Amount::new(0).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(pool.suspended(), [3130, 0]);
    let annual = [(1, -1000), (2, -3000)];
    let SourceFact::Reported(annual) = passive_characters(SourceFact::Reported(CharacterInput {
        nets: &annual,
        suspended: Amount::<Dollars>::new(3130).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(annual.iter().map(|v| v.held()).collect::<alloc::vec::Vec<_>>(), [783, 2347]);
    let cents = [(1, -100_000), (2, -300_000)];
    let SourceFact::Reported(cents) = passive_characters(SourceFact::Reported(CharacterInput {
        nets: &cents,
        suspended: Amount::<Cents>::new(313_000).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        cents
            .iter()
            .map(|v| (*v.key, v.net, v.held(), v.allowed_net()))
            .collect::<alloc::vec::Vec<_>>(),
        [(1, -100_000, 78_250, -21_750), (2, -300_000, 234_750, -65_250)]
    );
}

#[test]
fn annual_special_allowance_uses_original_weights_before_common_holds() {
    let activities = [
        PassiveActivity { key: 1, net: -149, rental_allowance: true, ineligible_prior: 0 },
        PassiveActivity { key: 2, net: -51, rental_allowance: true, ineligible_prior: 0 },
    ];
    let SourceFact::Reported(pool) = passive_pool(SourceFact::Reported(PoolInput {
        activities: &activities,
        other_passive_income: Amount::<Cents>::new(0).unwrap(),
        allowance: Amount::new(100).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(pool.suspended(), [74, 26]);
    assert_eq!(pool.annual_suspensions(), Ok(alloc::vec![0, 1]));
}

#[test]
fn legacy_zero_weight_annual_projection_does_not_erase_raw_held_cents() {
    // 2026-10-15 PhaseA preserves the existing100c/50c recipe unchanged:
    // annualspecial1 leaves zero weights. No new residue or refusal is added.
    let activities =
        [PassiveActivity { key: 1, net: -100, rental_allowance: true, ineligible_prior: 0 }];
    let SourceFact::Reported(pool) = passive_pool(SourceFact::Reported(PoolInput {
        activities: &activities,
        other_passive_income: Amount::<Cents>::new(0).unwrap(),
        allowance: Amount::new(50).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!((pool.held_loss(), pool.allowed_loss()), (50, 50));
    assert_eq!(pool.suspended(), [50]);
    assert_eq!(pool.annual_suspensions(), Ok(alloc::vec![0]));
}

#[test]
fn character_annual_rows_keep_pre_hold_weights_and_positive_classes() {
    let raw = [(1, -100), (2, -100), (3, -500)];
    let SourceFact::Reported(raw) = passive_characters(SourceFact::Reported(CharacterInput {
        nets: &raw,
        suspended: Amount::<Cents>::new(400).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(raw.iter().map(|v| v.held()).collect::<alloc::vec::Vec<_>>(), [57, 57, 286]);
    let annual = [(1, -1), (2, -1), (3, -5)];
    let SourceFact::Reported(annual) = passive_characters(SourceFact::Reported(CharacterInput {
        nets: &annual,
        suspended: Amount::<Dollars>::new(4).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(annual.iter().map(|v| v.held()).collect::<alloc::vec::Vec<_>>(), [1, 1, 2]);
    let offset = [(1, -100), (2, 50)];
    let SourceFact::Reported(offset) = passive_characters(SourceFact::Reported(CharacterInput {
        nets: &offset,
        suspended: Amount::<Cents>::new(25).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        offset.iter().map(|v| (v.held(), v.allowed_net())).collect::<alloc::vec::Vec<_>>(),
        [(25, -75), (0, 50)]
    );
}

#[test]
fn unknown_absent_and_complete_empty_sources_remain_distinct() {
    assert!(matches!(
        passive_pool::<i64, Cents>(SourceFact::Unavailable),
        Ok(SourceFact::Unavailable)
    ));
    assert!(matches!(
        passive_pool::<i64, Cents>(SourceFact::NotReported),
        Ok(SourceFact::NotReported)
    ));
    assert!(matches!(
        passive_characters::<i64, Cents>(SourceFact::Unavailable),
        Ok(SourceFact::Unavailable)
    ));
    assert!(matches!(
        passive_characters::<i64, Cents>(SourceFact::NotReported),
        Ok(SourceFact::NotReported)
    ));
    let SourceFact::Reported(pool) = passive_pool::<i64, Cents>(SourceFact::Reported(PoolInput {
        activities: &[],
        other_passive_income: Amount::new(0).unwrap(),
        allowance: Amount::new(0).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!((pool.total_loss(), pool.held_loss(), pool.allowed_loss()), (0, 0, 0));
    assert!(pool.suspended().is_empty());
    assert_eq!(pool.annual_suspensions(), Ok(alloc::vec![]));
}

#[test]
fn invalid_footing_and_arithmetic_overflow_do_not_become_unknown_or_zero() {
    let malformed =
        [PassiveActivity { key: 1, net: -100, rental_allowance: true, ineligible_prior: -1 }];
    assert!(matches!(
        passive_pool(SourceFact::Reported(PoolInput {
            activities: &malformed,
            other_passive_income: Amount::<Cents>::new(0).unwrap(),
            allowance: Amount::new(0).unwrap(),
        })),
        Err(LossLimitError::InvalidAmount)
    ));
    let overflowing =
        [PassiveActivity { key: 1, net: i64::MIN, rental_allowance: false, ineligible_prior: 0 }];
    assert!(matches!(
        passive_pool(SourceFact::Reported(PoolInput {
            activities: &overflowing,
            other_passive_income: Amount::<Cents>::new(0).unwrap(),
            allowance: Amount::new(0).unwrap(),
        })),
        Err(LossLimitError::Overflow)
    ));
    let overheld = [(1, -5), (2, 1)];
    assert!(matches!(
        passive_characters(SourceFact::Reported(CharacterInput {
            nets: &overheld,
            suspended: Amount::<Cents>::new(5).unwrap(),
        })),
        Err(LossLimitError::Footing)
    ));
    let overflowing = [(1, -i64::MAX), (2, i64::MAX), (3, -2)];
    assert!(matches!(
        passive_characters(SourceFact::Reported(CharacterInput {
            nets: &overflowing,
            suspended: Amount::<Cents>::new(1).unwrap(),
        })),
        Err(LossLimitError::Overflow)
    ));
    let large =
        [PassiveActivity { key: 1, net: -i64::MAX, rental_allowance: false, ineligible_prior: 0 }];
    let SourceFact::Reported(pool) = passive_pool(SourceFact::Reported(PoolInput {
        activities: &large,
        other_passive_income: Amount::<Cents>::new(0).unwrap(),
        allowance: Amount::new(0).unwrap(),
    }))
    .unwrap() else {
        panic!()
    };
    assert_eq!(pool.held_loss(), i64::MAX);
    // 2026-10-16: ratio::dollars widens its half-dollar offset to i128;
    // the largest native cent hold has an exact representable annual result.
    assert_eq!(pool.annual_suspensions(), Ok(alloc::vec![92_233_720_368_547_758]));
}
