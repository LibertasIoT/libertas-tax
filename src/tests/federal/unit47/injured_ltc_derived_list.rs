use crate::tests::support::client::survey::survey_injured_capacity_session;
use crate::tests::cross::interview::accepted;
use crate::tests::support::{
    assert::{carrier_schema, schema_field},
    builders::{INJURED_ANNUAL_SOURCE, INJURED_CONTRACT_DAYS, INJURED_DAILY_SOURCE},
    client::{
        injured_capacity_answer_page_is_pure, injured_capacity_calculation,
        injured_capacity_editor, injured_capacity_refusal, injured_capacity_source_plan,
    },
};
use crate::tests::*;
use alloc::{collections::BTreeSet, format};

fn injured_capacity_derived_source_plan() -> (Interview, [Option<i64>; 5]) {
    assert_eq!(
        INJURED_DAILY_SOURCE * INJURED_CONTRACT_DAYS as i64,
        INJURED_ANNUAL_SOURCE
    );
    assert_eq!(crate::dates::add_days(20260101, 364), 20261231);
    let (mut session, ids, known_joint) = injured_capacity_source_plan();
    let saved = session.draft.clone();
    let result = supported(&saved);
    assert_eq!(value(&result, "taxable_ltc_benefits"), 2_496_600);
    assert_eq!(
        [
            result.tax,
            result.refundable_credits,
            result.payments,
            result.refund,
            result.balance
        ],
        known_joint
    );
    let plan = result.injured_spouse.as_ref().expect("actual derived plan");
    let ltc: Vec<_> = plan
        .required
        .iter()
        .filter(|r| r.component.starts_with("ltc_"))
        .collect();
    let actual = injured_capacity_editor(&mut session);
    let P::SaveInjuredSpouse { value: shown, .. } = &actual else {
        unreachable!()
    };
    assert!(shown.allocations.is_empty());
    assert_eq!(shown.new_allocations, plan.required);
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let reopened = injured_capacity_editor(&mut cold);
    let P::SaveInjuredSpouse {
        value: reopened_value,
        ..
    } = &reopened
    else {
        unreachable!()
    };
    assert_eq!(reopened_value.allocations, shown.allocations);
    assert_eq!(reopened_value.new_allocations, shown.new_allocations);
    assert_eq!(cold.draft, saved);
    assert_eq!(
        value(&supported(&cold.draft), "taxable_ltc_benefits"),
        2_496_600
    );
    assert_eq!(
        ltc.len(),
        4380,
        "actual producer must expose every source period"
    );
    assert_eq!(plan.required.len(), 4380);
    assert_eq!(
        ltc.iter()
            .map(|r| r.component.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        4380
    );
    for id in &ids {
        for day in 1..=INJURED_CONTRACT_DAYS {
            let key = format!("ltc_{id}_period_{day}");
            let row = ltc
                .iter()
                .find(|r| r.component == key)
                .expect("actual source/period key");
            assert_eq!(
                (row.joint, row.minimum, row.maximum),
                (57_000, 7_000, 50_000)
            );
        }
    }
    // The accepted source graph exceeds the historical4096 ceiling. Its
    // unbounded native List retains exact required-membership validation.
    if let Some(schema) = carrier_schema() {
        for (owner, list) in [
            ("InjuredSpouseInitial", "allocations"),
            ("InjuredSpouseInitial", "new_allocations"),
            ("InjuredSpouse", "allocations"),
            ("InjuredSpouse", "new_allocations"),
            ("InjuredSpouseAllocation", "allocations"),
        ] {
            let field = schema_field(&schema, owner, &[list]);
            assert_eq!(field["Type"], "List");
            assert!(
                !field["Attributes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|a| a["Name"] == "SizeMax")
            );
        }
    }
    (session, known_joint)
}

fn injured_capacity_answer_lifecycle(session: &mut Interview, known_joint: &[Option<i64>; 5]) {
    let source_facts = session.draft.income.clone();
    let mut request = injured_capacity_editor(session);
    let P::SaveInjuredSpouse { value: page, .. } = &mut request else {
        unreachable!()
    };
    assert!(page.allocations.is_empty());
    assert_eq!(page.new_allocations.len(), 4380);
    for row in &mut page.new_allocations {
        assert_eq!(
            (row.joint, row.minimum, row.maximum),
            (57_000, 7_000, 50_000)
        );
        row.injured = 7_000;
    }
    let expected = page.new_allocations.clone();
    // Equality at the actual recomputed minimum is a valid complete answer.
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(session, injured_capacity_calculation(&request)),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, request));
    assert!(session.draft.valid_stored());
    assert_eq!(session.draft.income, source_facts);
    let answers = &session.draft.injured_spouse.as_ref().unwrap().allocations;
    assert_eq!(answers.len(), 4380);
    assert!(answers.iter().all(|row| row.injured == 7_000));
    assert_eq!(
        answers.iter().map(|row| row.injured).sum::<i64>(),
        30_660_000
    );
    injured_capacity_answer_page_is_pure(session);
    let accepted_state = session.draft.clone();
    let result = supported(&accepted_state);
    assert_eq!(value(&result, "taxable_ltc_benefits"), 2_496_600);
    assert_eq!(
        [
            result.tax,
            result.refundable_credits,
            result.payments,
            result.refund,
            result.balance
        ],
        *known_joint
    );
    let reopened = injured_capacity_editor(session);
    let P::SaveInjuredSpouse { value: page, .. } = &reopened else {
        unreachable!()
    };
    assert_eq!(page.allocations, expected);
    assert!(page.new_allocations.is_empty());
    accepted(&send(session, reopened));
    assert_eq!(session.draft, accepted_state);
    let mut cold = restart(accepted_state.clone());
    let reopened = injured_capacity_editor(&mut cold);
    let P::SaveInjuredSpouse { value: page, .. } = &reopened else {
        unreachable!()
    };
    assert_eq!(page.allocations, expected);
    assert!(page.new_allocations.is_empty());
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, accepted_state);
    assert_eq!(supported(&cold.draft), result);
    *session = cold;
}

#[test]
fn unit47_injured_ltc_identity_refusals_keep_all_4380_accepted_rows() {
    let mut session = survey_injured_capacity_session();
    injured_capacity_bad_identity(&mut session);
    injured_capacity_corrected_answers(&mut session);
}

fn injured_capacity_bad_identity(session: &mut Interview) {
    let actual = injured_capacity_editor(session);
    let mut duplicate = actual.clone();
    let P::SaveInjuredSpouse { value, .. } = &mut duplicate else {
        unreachable!()
    };
    value.allocations[4379].component = value.allocations[0].component.clone();
    injured_capacity_refusal(
        session,
        duplicate,
        "duplicate-component",
        "[value/allocations/4379/component]",
        false,
    );
    let mut foreign = actual.clone();
    let P::SaveInjuredSpouse { value, .. } = &mut foreign else {
        unreachable!()
    };
    value.allocations[4379].component = "ltc_999999999_period_1".into();
    injured_capacity_refusal(
        session,
        foreign,
        "foreign-component",
        "[value/allocations/4379/component]",
        true,
    );
    let mut missing = actual.clone();
    let P::SaveInjuredSpouse { value, .. } = &mut missing else {
        unreachable!()
    };
    value.allocations.pop();
    injured_capacity_refusal(
        session,
        missing,
        "missing-component",
        "[value/allocations]",
        true,
    );
}

#[test]
fn unit47_injured_ltc_amount_and_echo_refusals_keep_all_4380_accepted_rows() {
    let mut session = survey_injured_capacity_session();
    injured_capacity_bad_amounts(&mut session);
    injured_capacity_corrected_answers(&mut session);
}

fn injured_capacity_bad_amounts(session: &mut Interview) {
    let actual = injured_capacity_editor(session);
    for (case, injured) in [("below-minimum", 6_999), ("above-maximum", 50_001)] {
        let mut invalid = actual.clone();
        let P::SaveInjuredSpouse { value, .. } = &mut invalid else {
            unreachable!()
        };
        let row = &mut value.allocations[4379];
        row.injured = injured;
        injured_capacity_refusal(
            session,
            invalid,
            case,
            "[value/allocations/4379/injured]",
            false,
        );
        let mut forged = actual.clone();
        let P::SaveInjuredSpouse { value, .. } = &mut forged else {
            unreachable!()
        };
        let row = &mut value.allocations[4379];
        row.injured = injured;
        // Save refreshes forged echo bounds; Estimate still recomputes and
        // refuses the amount. Neither path accepts the forged permission.
        row.minimum = 0;
        row.maximum = 57_000;
        injured_capacity_refusal(
            session,
            forged,
            &format!("forged-{case}"),
            "[value/allocations/4379/injured]",
            true,
        );
    }
}

fn injured_capacity_corrected_answers(session: &mut Interview) {
    // A fresh complete corrected request succeeds after every rejected edit.
    let corrected = injured_capacity_editor(session);
    let before = session.draft.clone();
    accepted(&send(session, corrected));
    assert_eq!(session.draft, before);
    assert_eq!(restart(session.draft.clone()).draft, before);
}

#[test]
fn unit47_injured_ltc_rows_follow_twelve_actual_daily_contract_sources() {
    // Disjoint source-admission and full-answer frames keep the default test
    // stack; no stack override or weakened source/financial assertion.
    let (mut session, known_joint) = injured_capacity_derived_source_plan();
    injured_capacity_answer_lifecycle(&mut session, &known_joint);
}

#[test]
fn unit47_injured_ltc_actual_ltc_period_labels_keep_both_source_and_period() {
    for (id, expected) in [
        ("ltc_benefit_2_1_received", "LTC benefit 2 1 received"),
        (
            "ltc_benefit_25_365_allocated_limit",
            "LTC benefit 25 365 allocated limit",
        ),
    ] {
        let keys = crate::federal_model::IdKeys::new(id);
        assert!(keys.registered());
        let label = crate::line_label(id);
        assert_eq!(label.english(), expected);
        // Ordinary FederalEstimate breakdown conversion uses this same owner.
        // The source and period stay distinct arguments, never a fixed label.
        assert_eq!(label.args.len(), 2);
    }
}
