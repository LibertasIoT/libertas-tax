use crate::k1_activity::*;
use crate::tests::support::builders::k1res::{
    amounts, block, current, histories, history, parent, scorp_items, signed_block, source,
};
use crate::tests::*;
#[test]
fn k1_nullable_stages_preserve_known_current_and_individual_retained_sources() {
    let mut value = parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for component in &mut source.components {
        let amounts = component.regular.as_mut().unwrap();
        amounts.operating_carry.risk = None;
        amounts.section179_basis_carry = None;
        amounts.charity_basis_carry = None;
    }
    source.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(10_000);
    source.components[0].regular.as_mut().unwrap().passive_operating = Some(5_000);
    assert_eq!(value.rule(), None);
    let K1ComponentStatement::Entered { source } = &value.activities else { unreachable!() };
    assert_eq!(source.components[0].regular.as_ref().unwrap().operating, 400_000);
    assert_eq!(source.components[1].regular.as_ref().unwrap().operating, -600_000);
    let amounts = source.components[0].regular.as_ref().unwrap();
    assert_eq!(
        (amounts.operating_carry.basis, amounts.operating_carry.risk, amounts.passive_operating),
        (Some(10_000), None, Some(5_000))
    );
    let mut contradiction = value.clone();
    contradiction.current.ordinary += 1;
    assert_eq!(contradiction.rule(), Some("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"));
    let K1ComponentStatement::Entered { source } = &mut contradiction.activities else {
        unreachable!()
    };
    source.components[0].regular.as_mut().unwrap().passive_operating = Some(-1);
    assert_eq!(contradiction.rule(), Some("K1_ACTIVITY_SOURCE_AMOUNT_INVALID"));
}
#[test]
fn k1_nullable_stages_check_only_actual_known_subset_operands() {
    let mut value = amounts(400_000, 0);
    value.section179_sold_carry = Some(3_000);
    value.section179_business_carry = None;
    assert_eq!(value.rule(true), None);
    value.section179_business_carry = Some(2_999);
    assert_eq!(value.rule(true), Some("K1_ACTIVITY_SOLD_CARRY_ABOVE_TOTAL"));
    value.section179_business_carry = Some(3_000);
    assert_eq!(value.rule(true), None);
    value.property = Some(K1StatementProperty {
        net: -12_345,
        unrecaptured: 54_321,
        basis_loss: None,
        risk_loss: Some(2_000),
        passive_loss: None,
    });
    assert_eq!(value.rule(true), None);
    value.property.as_mut().unwrap().risk_loss = Some(-1);
    assert_eq!(value.rule(true), Some("K1_ACTIVITY_SOURCE_AMOUNT_INVALID"));
    let mut capital = K1StatementScorpCapitalCarry {
        basis_short: None,
        basis_long: None,
        basis_collectibles: Some(10),
        risk_short: Some(0),
        risk_long: Some(20),
        risk_collectibles: Some(21),
    };
    let mut scorp = amounts(0, 0);
    scorp.items = scorp_items();
    scorp.capital_carry = K1StatementCapitalCarry::Scorp { amounts: capital.clone() };
    assert_eq!(scorp.rule(false), Some("K1_ACTIVITY_SOURCE_AMOUNT_INVALID"));
    capital.risk_long = None;
    scorp.capital_carry = K1StatementCapitalCarry::Scorp { amounts: capital };
    assert_eq!(scorp.rule(false), None);
}
#[test]
fn k1_nullable_stages_keep_qbi_tax_history_controls_independent_of_unknown_carry() {
    let mut value = histories();
    value.components[0].regular.as_mut().unwrap().operating_carry.basis = None;
    assert_eq!(value.source_rule(true), None);
    value.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(4_999);
    assert_eq!(value.source_rule(true), Some("K1_QBI_TAX_HISTORY_RECONCILE_COMPONENT"));
    value.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(5_000);
    assert_eq!(value.source_rule(true), None);
}
#[test]
fn k1_nullable_se_stages_preserve_reported_current_and_known_subsets() {
    let mut value = source();
    let amounts = value.components[0].regular.as_mut().unwrap();
    amounts.operating_carry.basis = Some(5_000);
    amounts.operating_carry.risk = None;
    value.components[0].self_employment = K1StatementSe::Partnership {
        source: K1PartnerSeSource {
            reported: Some(400_000),
            history: K1RetainedSeHistory { basis: Some(5_000), risk: Some(700), passive: None },
            gross: None,
            guaranteed_services: 0,
            guaranteed_capital: 0,
        },
    };
    assert_eq!(value.source_rule(true), None);
    value.components[0].regular.as_mut().unwrap().operating_carry.risk = Some(699);
    assert_eq!(value.source_rule(true), Some("K1_ACTIVITY_SE_HISTORY_ABOVE_LOSS"));
    value.components[0].regular.as_mut().unwrap().operating_carry.risk = None;
    let K1StatementSe::Partnership { source } = &mut value.components[0].self_employment else {
        unreachable!()
    };
    source.history.passive = Some(-1);
    assert_eq!(value.source_rule(true), Some("K1_ACTIVITY_SOURCE_AMOUNT_INVALID"));
}
#[test]
fn k1_component_original_opposite_signed_sources_reconcile_without_basis_calculation() {
    let value = parent();
    assert_eq!(value.rule(), None);
    assert_eq!(value.current.ordinary, -200_000);
    assert_eq!(value.current.cash_charity, 400_000);
    assert_eq!(value.entity.regular.opening, Some(200_000));
    let source = value.activities.source().unwrap();
    assert_eq!(
        source.components.iter().map(|v| v.regular.as_ref().unwrap().operating).collect::<Vec<_>>(),
        [400_000, -600_000]
    );
}
#[test]
fn k1_component_known_control_contradiction_does_not_modify_source() {
    let accepted = parent();
    let mut candidate = accepted.clone();
    candidate.current.ordinary += 1;
    assert_eq!(candidate.rule(), Some("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"));
    assert_eq!(accepted, parent());
}
#[test]
fn k1_component_separate_amt_controls_keep_actual_values_and_missing_source() {
    let mut value = parent();
    value.alternative_current = Some(current());
    if let K1ComponentStatement::Entered { source } = &mut value.activities {
        source.components[1].alternative =
            K1StatementAmt::Separate { amounts: amounts(-500_000, 0) };
    }
    value.alternative_current.as_mut().unwrap().ordinary = -100_000;
    assert_eq!(value.rule(), None);
    value.alternative_current.as_mut().unwrap().ordinary = -100_001;
    assert_eq!(value.rule(), Some("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"));
    if let K1ComponentStatement::Entered { source } = &mut value.activities {
        source.alternative_entity_only = K1EntityOnlyAmt::Unavailable;
    }
    assert_eq!(value.rule(), Some("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"));
    value.alternative_current.as_mut().unwrap().ordinary = -100_000;
    value.alternative_current.as_mut().unwrap().portfolio.interest = 1;
    assert_eq!(value.rule(), None);
    assert_eq!(value.alternative_current.as_ref().unwrap().portfolio.interest, 1);
    // Saving factual source absence cannot make these AMT values ready or zero.
}
#[test]
fn k1_component_id0_sources_are_fresh_and_cannot_guess_role_references() {
    let mut value = source();
    for id in value.ids_mut() {
        *id = 0;
    }
    for component in &mut value.components {
        component.risk_group = None;
        component.passive_group = None;
    }
    assert_eq!(value.identity_rule(None), None);
    value.components[0].risk_group = Some(21);
    assert_eq!(value.identity_rule(None), Some("K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"));
}
#[test]
fn k1_component_same_parent_roles_reject_reassigned_duplicate_and_removed_ids() {
    let accepted = source();
    assert_eq!(accepted.identity_rule(Some(&accepted)), None);
    let mut role = accepted.clone();
    role.risk_groups[0].id = 11;
    assert_eq!(role.identity_rule(Some(&accepted)), Some("K1_ACTIVITY_SOURCE_ID_NOT_ACCEPTED"));
    let mut repeated = accepted.clone();
    repeated.components[1].id = 11;
    assert_eq!(repeated.identity_rule(Some(&accepted)), Some("K1_ACTIVITY_SOURCE_ID_REPEATED"));
    let mut removed = accepted.clone();
    removed.risk_groups.remove(0);
    assert_eq!(
        removed.identity_rule(Some(&accepted)),
        Some("K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED")
    );
    assert_eq!(accepted, source());
}
#[test]
fn k1_component_reorder_rename_preserves_authorized_source_identity() {
    let accepted = source();
    let mut candidate = accepted.clone();
    candidate.components.reverse();
    candidate.risk_groups.reverse();
    candidate.passive_groups.reverse();
    candidate.qbi_blocks.reverse();
    candidate.components[0].name = "Renamed actual source".into();
    assert_eq!(candidate.identity_rule(Some(&accepted)), None);
    assert_eq!(candidate.components[0].id, 12);
    assert_eq!(candidate.components[0].risk_group, Some(22));
}
#[test]
fn k1_component_qbi_shared_tax_controls_count_once_and_subsets_collectively() {
    let value = histories();
    assert_eq!(value.identity_rule(Some(&value)), None);
    assert_eq!(value.source_rule(true), None);
    let mut duplicate = value.clone();
    duplicate.qbi_blocks[0]
        .allocations
        .as_mut()
        .unwrap()
        .push(value.qbi_blocks[0].allocations.as_ref().unwrap()[0].clone());
    assert_eq!(duplicate.identity_rule(Some(&value)), Some("K1_QBI_COMPONENT_REPEATED"));
    let mut mismatch = value.clone();
    mismatch.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified =
        history(10_001, 5_000, 3_000, 1_500);
    assert_eq!(mismatch.source_rule(true), Some("K1_QBI_SHARED_TAX_HISTORY_CONFLICT"));
    let mut excess = value.clone();
    excess.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified =
        history(10_000, 5_000, 8_000, 4_000);
    assert_eq!(excess.source_rule(true), Some("K1_QBI_SHARED_QUALIFIED_HISTORY_ABOVE_SOURCE"));
    assert_eq!(value, histories());
}
#[test]
fn k1_component_qbi_prior_consumed_subsets_cannot_exceed_shared_tax_consumption() {
    let mut value = histories();
    value.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified =
        history(10_000, 5_000, 6_000, 1_000);
    value.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified =
        history(10_000, 5_000, 4_000, 3_000);
    assert_eq!(value.source_rule(true), Some("K1_QBI_SHARED_QUALIFIED_HISTORY_ABOVE_SOURCE"));
}
#[test]
fn k1_component_page_roundtrip_keeps_source_when_output_catalog_is_omitted_or_forged() {
    let value = parent();
    let mut page = K1PartnershipActivitiesPage::from_record("Actual entity", &value);
    assert_eq!(page.record(), value);
    assert_eq!(
        page.activities.choices.components.iter().map(|v| v.id).collect::<Vec<_>>(),
        [11, 12]
    );
    page.activities.choices.components[0].id = 999;
    assert_eq!(page.record(), value);
    page.activities.choices = K1ActivityChoiceContext::default();
    assert_eq!(page.record(), value);
}
#[test]
fn k1_component_new_block_cannot_select_group_before_first_accepted_save() {
    let accepted = source();
    let mut candidate = accepted.clone();
    candidate.qbi_blocks.push(block(0, "New source"));
    assert_eq!(candidate.identity_rule(Some(&accepted)), None);
    candidate.qbi_blocks.last_mut().unwrap().qbi_aggregation_group = Some("Existing group".into());
    assert_eq!(
        candidate.identity_rule(Some(&accepted)),
        Some("QBI_AGGREGATION_ACCEPT_SOURCE_FIRST")
    );
}
#[test]
fn k1_component_signed_qbi_blocks_use_actual_sign_controls_not_component_net() {
    let mut value = source();
    value.components[0].regular = Some(amounts(0, 0));
    value.components[0].qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 1_000, loss: 1_000 });
    signed_block(&mut value, 0, 1_000);
    signed_block(&mut value, 1, -1_000);
    assert_eq!(value.source_rule(true), None);
    assert_eq!(value.components[0].regular.as_ref().unwrap().operating, 0);
    value.components[0].qualified_current_controls.ordinary = None;
    assert_eq!(value.source_rule(true), None);
    // Missing character support preserves the actual signed blocks; the
    // absent canonical owner feed cannot produce current QBI readiness.
}
#[test]
fn k1_component_qualified_positive_can_exceed_net_but_not_actual_source_part() {
    let mut value = source();
    value.components[0].regular = Some(amounts(500, 0));
    value.components[0].qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 1_000, loss: 500 });
    signed_block(&mut value, 0, 1_000);
    value.qbi_blocks.remove(1);
    assert_eq!(value.source_rule(true), None);
    value.components[0].qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 999, loss: 499 });
    assert_eq!(value.source_rule(true), Some("K1_QBI_CURRENT_ABOVE_COMPONENT_SOURCE"));
}
#[test]
fn k1_component_signed_qbi_allocations_reconcile_to_actual_printed_block() {
    let mut value = source();
    value.components[0].regular = Some(amounts(1_000, 0));
    value.components[0].qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 1_000, loss: 0 });
    signed_block(&mut value, 0, 1_000);
    if let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
        &mut value.qbi_blocks[0].statement
    {
        statement.income = 999;
    }
    assert_eq!(value.source_rule(true), Some("K1_QBI_PRINTED_CURRENT_RECONCILE"));
}
