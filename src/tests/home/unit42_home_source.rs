use crate::casualty::home::{ThirdTierCarry, conversion, property_loss, third_tier};
use crate::kernel::carry::LedgerError;

const NO_CARRY: ThirdTierCarry = ThirdTierCarry {
    total_cents: 0,
    casualty_cents: Some(0),
};

#[test]
fn approved_f9_reimbursement_reduces_continuing_basis_without_a_deductible_loss() {
    // Approved F9 correction: 20,000 basis less 12,000 reimbursement; the
    // 10,000 FMV decline produces no current loss, but basis is still 8,000.
    let result = property_loss(2_000_000, Some(1_000_000), 1_200_000).unwrap();
    assert_eq!(result.limited_cents, 1_000_000);
    assert_eq!(result.loss_cents, 0);
    assert_eq!(result.continuing_basis_cents, 800_000);
}

#[test]
fn section_b_destroyed_property_uses_adjusted_basis_instead_of_fmv_decrease() {
    let destroyed = property_loss(2_000_000, None, 500_000).unwrap();
    assert_eq!(destroyed.limited_cents, 2_000_000);
    assert_eq!(destroyed.loss_cents, 1_500_000);
    assert_eq!(destroyed.continuing_basis_cents, 0);
    let damaged = property_loss(2_000_000, Some(1_000_000), 500_000).unwrap();
    assert_eq!(damaged.loss_cents, 500_000);
    assert_eq!(damaged.continuing_basis_cents, 1_000_000);
    // Expected reimbursement reduces the loss; received-only money is used
    // separately by conversion(), so it cannot fabricate a current gain.
    let pending = property_loss(2_000_000, Some(1_000_000), 2_500_000).unwrap();
    assert_eq!(pending.loss_cents, 0);
    assert_eq!(pending.continuing_basis_cents, 0);
    assert_eq!(
        conversion(0, 2_000_000, 0, None).unwrap().recognized_cents,
        0
    );
    assert_eq!(property_loss(-1, None, 0), Err(LedgerError::InvalidAmount));
    assert_eq!(
        property_loss(0, Some(-1), 0),
        Err(LedgerError::InvalidAmount)
    );
    assert_eq!(property_loss(0, None, -1), Err(LedgerError::InvalidAmount));
}

#[test]
fn approved_home_fixture_j_retains_character_under_binding_income_limit() {
    // HOME fixture J and dispatcher eac268d's binding variant, in cents.
    let full = third_tier(0, 60_000, 40_000, NO_CARRY, 100_000).unwrap();
    assert_eq!(full.total_allowed_cents, 100_000);
    assert_eq!(full.allowed_casualty_cents(), Some(60_000));
    assert_eq!(full.allowed_depreciation_cents(), Some(40_000));
    assert_eq!(full.carry, NO_CARRY);

    let limited = third_tier(0, 60_000, 40_000, NO_CARRY, 50_000).unwrap();
    assert_eq!(limited.total_allowed_cents, 50_000);
    assert_eq!(limited.allowed_casualty_cents(), Some(50_000));
    assert_eq!(limited.allowed_depreciation_cents(), Some(0));
    assert_eq!(limited.current_casualty_carry_cents, 10_000);
    assert_eq!(limited.current_depreciation_carry_cents, 40_000);
    assert_eq!(limited.carry.total_cents, 50_000);
    assert_eq!(limited.carry.casualty_cents, Some(10_000));

    // A move or subsequent year retains the actual two components.
    let next = third_tier(0, 0, 0, limited.carry, 25_000).unwrap();
    assert_eq!(next.allowed_casualty_cents(), Some(10_000));
    assert_eq!(next.allowed_depreciation_cents(), Some(15_000));
    assert_eq!(next.carry.total_cents, 25_000);
    assert_eq!(next.carry.casualty_cents, Some(0));
}

#[test]
fn home_current_depreciation_precedes_prior_casualty_carry() {
    // Current items precede the prior combined balance. Casualty first
    // *within* the carry is the dispatcher's explicitly recorded inference.
    let prior = ThirdTierCarry {
        total_cents: 90_000,
        casualty_cents: Some(30_000),
    };
    let result = third_tier(7_500, 20_000, 40_000, prior, 80_000).unwrap();
    assert_eq!(result.allowed_excess_casualty_cents, 20_000);
    assert_eq!(result.allowed_current_depreciation_cents, 40_000);
    assert_eq!(result.allowed_prior_casualty_cents, Some(20_000));
    assert_eq!(result.allowed_prior_depreciation_cents, Some(0));
    assert_eq!(result.total_allowed_cents, 80_000);
    // Otherwise-allowed casualty is first tier and consumes no third-tier
    // room; it still belongs in 8829 line35, never Schedule C/F line36.
    assert_eq!(result.allowed_casualty_cents(), Some(47_500));
    assert_eq!(result.allowed_depreciation_cents(), Some(40_000));
    assert_eq!(result.carry.total_cents, 70_000);
    assert_eq!(result.carry.casualty_cents, Some(10_000));
}

#[test]
fn home_uncharacterized_prior_balance_never_becomes_depreciation() {
    let prior = ThirdTierCarry {
        total_cents: 50_000,
        casualty_cents: None,
    };
    let result = third_tier(0, 60_000, 40_000, prior, 120_000).unwrap();
    assert_eq!(result.total_allowed_cents, 120_000);
    assert_eq!(result.prior_carry_remaining_cents, 30_000);
    assert_eq!(result.allowed_casualty_cents(), None);
    assert_eq!(result.allowed_depreciation_cents(), None);
    assert_eq!(result.carry.casualty_cents, None);

    let no_prior_use = third_tier(0, 60_000, 40_000, prior, 50_000).unwrap();
    assert_eq!(no_prior_use.allowed_casualty_cents(), Some(50_000));
    assert_eq!(no_prior_use.allowed_depreciation_cents(), Some(0));
    assert_eq!(no_prior_use.carry.casualty_cents, None);

    let all_used = third_tier(0, 0, 0, prior, 50_000).unwrap();
    assert_eq!(all_used.allowed_casualty_cents(), None);
    assert_eq!(all_used.carry, NO_CARRY);
}

#[test]
fn home_tier_source_rejects_invalid_components_and_keeps_cents() {
    let result = third_tier(1, 3, 4, NO_CARRY, 5).unwrap();
    assert_eq!(result.allowed_casualty_cents(), Some(4));
    assert_eq!(result.allowed_depreciation_cents(), Some(2));
    assert_eq!(result.carry.total_cents, 2);
    for (otherwise, casualty, depreciation, carry, room) in [
        (-1, 0, 0, NO_CARRY, 0),
        (0, -1, 0, NO_CARRY, 0),
        (0, 0, -1, NO_CARRY, 0),
        (0, 0, 0, NO_CARRY, -1),
        (
            0,
            0,
            0,
            ThirdTierCarry {
                total_cents: -1,
                casualty_cents: None,
            },
            0,
        ),
        (
            0,
            0,
            0,
            ThirdTierCarry {
                total_cents: 5,
                casualty_cents: Some(6),
            },
            0,
        ),
    ] {
        assert_eq!(
            third_tier(otherwise, casualty, depreciation, carry, room),
            Err(LedgerError::InvalidAmount)
        );
    }
}

#[test]
fn approved_home_fixture_m_consumes_exclusion_before_replacement_threshold() {
    let result = conversion(60_000_000, 20_000_000, 25_000_000, Some(32_000_000)).unwrap();
    assert_eq!(result.excluded_cents, 25_000_000);
    assert_eq!(result.proceeds_after_exclusion_cents, 35_000_000);
    assert_eq!(result.gain_after_exclusion_cents, 15_000_000);
    assert_eq!(result.recognized_cents, 3_000_000);
    assert_eq!(result.postponed_cents, 12_000_000);
    assert_eq!(result.replacement_basis_cents, Some(20_000_000));
    for (cost, recognized, postponed, basis) in [
        (20_000_000, 15_000_000, 0, 20_000_000),
        (35_000_000, 0, 15_000_000, 20_000_000),
        (40_000_000, 0, 15_000_000, 25_000_000),
    ] {
        let result = conversion(60_000_000, 20_000_000, 25_000_000, Some(cost)).unwrap();
        assert_eq!(result.recognized_cents, recognized);
        assert_eq!(result.postponed_cents, postponed);
        assert_eq!(result.replacement_basis_cents, Some(basis));
    }
}

#[test]
fn home_conversion_never_infers_an_exclusion_or_replacement_election() {
    let no_home = conversion(60_000_000, 20_000_000, 0, None).unwrap();
    assert_eq!(no_home.recognized_cents, 40_000_000);
    assert_eq!(no_home.postponed_cents, 0);
    assert_eq!(no_home.replacement_basis_cents, None);
    let excluded_only = conversion(60_000_000, 20_000_000, 25_000_000, None).unwrap();
    assert_eq!(excluded_only.recognized_cents, 15_000_000);
    assert_eq!(excluded_only.postponed_cents, 0);
    let zero_cost = conversion(60_000_000, 20_000_000, 25_000_000, Some(0)).unwrap();
    assert_eq!(zero_cost.recognized_cents, 15_000_000);
    assert_eq!(zero_cost.replacement_basis_cents, Some(0));
    let loss = conversion(15_000_000, 20_000_000, 0, None).unwrap();
    assert_eq!(loss.recognized_cents, 0);
    assert_eq!(loss.postponed_cents, 0);
}

#[test]
fn home_conversion_retains_cents_and_rejects_invalid_election_amounts() {
    let cents = conversion(607, 205, 251, Some(321)).unwrap();
    assert_eq!(cents.proceeds_after_exclusion_cents, 356);
    assert_eq!(cents.recognized_cents, 35);
    assert_eq!(cents.postponed_cents, 116);
    assert_eq!(cents.replacement_basis_cents, Some(205));
    for (excluded, replacement) in [(-1, None), (403, None), (0, Some(-1))] {
        assert_eq!(
            conversion(607, 205, excluded, replacement),
            Err(LedgerError::InvalidAmount)
        );
    }
}
