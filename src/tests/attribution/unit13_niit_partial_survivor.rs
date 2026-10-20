// Actual retained NIIT-use source attribution under the approved R3/R6
// shared-limit class. Ordinary money and capital ownership are independent.
use super::*;
use crate::tests::cross::interview::{accepted, open_topic};

#[test]
fn unit13_niit_partial_survivor_native_money_and_accepted_history_lifecycle() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let d = retained_niit_return(injured, reversed);
            assert_retained_niit_money(&d);
            let mut session = restart(d);
            for topic in
                [TaxSection::Wages, TaxSection::InterestDividends, TaxSection::InvestmentSales]
            {
                let page = open_topic(&mut session, topic);
                accepted(&send(&mut session, roundtrip_native_request(page)));
            }
            let draft = roundtrip_native_draft(session.draft.clone());
            let mut reopened = restart(draft);
            let page = open_topic(&mut reopened, TaxSection::InvestmentSales);
            accepted(&send(&mut reopened, page));
            let r = assert_retained_niit_money(&reopened.draft);
            assert_eq!(column(&r, "13b", "b"), 3500);
            assert_eq!(column(&r, "13b", "c"), 3500);
            let prior = &reopened.draft.screening.as_ref().unwrap().capital_history[0];
            assert_eq!(
                prior.injured_niit_owners,
                Some(InjuredRetainedOwners::Both { taxpayer: 400_000, spouse: 600_000 })
            );
        }
    }
}
#[test]
fn unit13_niit_partial_survivor_needs_existing_bounded_joint_source_control() {
    let r = assert_retained_niit_money(&retained_niit_return(Owner::Taxpayer, false));
    // Proposed existing direct shared-limit control, not a new source fact:
    // known retained owners4000/6000, actual current use5000 => TP0..4000.
    // Regular capital-limit ownership1500/1500 remains independent.
    let row = r
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .find(|row| row.joint == 500_000 && row.minimum == 0 && row.maximum == 400_000);
    assert!(
        row.is_some(),
        "missing bounded retained NIIT survivor allocation; actual native money and accepted retained owners above are already complete"
    );
}

#[test]
fn unit13_niit_partial_survivor_bounds_require_actual_source_footing() {
    use crate::injured_spouse::niit_retained::RetainedNiitUse;
    let mut source = RetainedNiitUse {
        prior_dollars: 10_000,
        used_dollars: 5000,
        next_dollars: 5000,
        current_components_known_zero: true,
    };
    assert_eq!(source.bounds([4000, 6000], 0), Some((0, 4000)));
    assert_eq!(source.bounds([4000, 6000], 1), Some((1000, 5000)));
    assert_eq!(source.bounds([10_000, 0], 0), Some((5000, 5000)));
    assert_eq!(source.bounds([0, 10_000], 0), Some((0, 0)));
    assert_eq!(source.bounds([4000, 5999], 0), None);
    source.next_dollars = 4999;
    assert_eq!(source.bounds([4000, 6000], 0), None);
    source.next_dollars = 0;
    source.used_dollars = 10_000;
    assert_eq!(source.bounds([4000, 6000], 0), Some((4000, 4000)));
    source.current_components_known_zero = false;
    assert_eq!(source.bounds([4000, 6000], 0), None);
    assert_eq!(source.bounds([10_000, 0], 0), None);
    source.used_dollars = 0;
    source.next_dollars = 10_000;
    source.current_components_known_zero = true;
    assert_eq!(source.bounds([4000, 6000], 0), Some((0, 0)));
}

#[test]
fn unit13_niit_partial_survivor_nonproportional_use_save_restart_and_endpoints() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            for (tp_use, niit) in [(0, [133, 323]), (1000, [171, 285]), (4000, [285, 171])] {
                let mut d = retained_niit_return(injured, reversed);
                assert_retained_niit_money(&d);
                let injured_use = if injured == Owner::Taxpayer { tp_use } else { 5000 - tp_use };
                d.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
                    component: "niit_retained_difference_used".into(),
                    injured: injured_use * 100,
                });
                let mut session = restart(d);
                let page = complete_niit_allocation(&mut session);
                let P::SaveInjuredSpouse { .. } = &page else { panic!() };
                accepted(&send(&mut session, roundtrip_native_request(page)));
                let draft = roundtrip_native_draft(session.draft.clone());
                let mut reopened = restart(draft);
                let page = complete_niit_allocation(&mut reopened);
                let P::SaveInjuredSpouse { .. } = &page else { panic!() };
                let P::SaveInjuredSpouse { value, .. } = &page else { panic!() };
                let use_row = value
                    .allocations
                    .iter()
                    .find(|row| row.component == "niit_retained_difference_used")
                    .unwrap();
                assert_eq!(use_row.injured, injured_use * 100);
                accepted(&send(&mut reopened, page));
                let result = assert_retained_niit_money(&reopened.draft);
                let expected = [niit[0] + 900, niit[1] + 450];
                let expected =
                    if injured == Owner::Taxpayer { expected } else { [expected[1], expected[0]] };
                assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], expected);
                // Neither a retained-use answer nor its source-axis endpoint
                // rewrites the independently owned regular capital history.
                assert_eq!(
                    [column(&result, "13b", "b"), column(&result, "13b", "c")],
                    [3500, 3500]
                );
            }
        }
    }
}

#[test]
fn unit13_niit_partial_survivor_out_of_bounds_never_commits() {
    let d = retained_niit_return(Owner::Spouse, false);
    let result = assert_retained_niit_money(&d);
    for selected in [99_999, 500_001] {
        let mut session = restart(d.clone());
        let mut page = niit_allocation_with_answers(
            &mut session,
            &[("niit_retained_difference_used", selected)],
        );
        let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
        value
            .allocations
            .iter_mut()
            .find(|row| row.component == "niit_retained_difference_used")
            .unwrap()
            .injured = selected;
        assert_eq!(
            crate::injured_spouse::allocation_rule(value, &session.draft, &result),
            Err("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
        );
        let before = session.draft.clone();
        assert!(!matches!(send(&mut session, page), Reply::Response(P::Review { .. })));
        assert_eq!(session.draft, before);
    }
}

#[test]
fn unit13_niit_partial_survivor_native_zero_full_and_exclusive_use_are_automatic() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for (gain, use_amount, niit, agi, regular, total, refund) in [
            (0, 0, 266, 407_000, 75_148, 76_764, 4136),
            (10_000, 10_000, 760, 410_000, 75_868, 77_978, 2922),
        ] {
            let mut d = retained_niit_return(injured, false);
            for entry in &mut d.income {
                if matches!(entry.income, FederalIncome::Sale { .. }) {
                    entry.income = ordinary_investment_sale(gain, 0, GainTerm::Short);
                }
            }
            let r = supported(&d);
            assert_eq!(value(&r, "niit_disposition_adjustment"), use_amount);
            assert_eq!(value(&r, "niit_capital_carry_adjustment"), 10_000 - use_amount);
            assert_eq!(value(&r, "net_investment_income_tax"), niit);
            assert_eq!(value(&r, "agi"), agi);
            assert_eq!(return_line(&r, "16"), regular * 100);
            assert_eq!((r.tax, r.refund), (Some(total * 100), Some(refund * 100)));
            let mut plain = d.clone();
            plain.injured_spouse = None;
            let ordinary = supported(&plain);
            assert_eq!(ordinary.return_lines, r.return_lines);
            assert_eq!((ordinary.tax, ordinary.refund), (r.tax, r.refund));
            assert!(
                r.injured_spouse
                    .as_ref()
                    .unwrap()
                    .required
                    .iter()
                    .all(|row| row.component != "niit_retained_difference_used")
            );
        }
        for (prior, nii_tax) in [
            (InjuredRetainedOwners::Taxpayer, [323, 133]),
            (InjuredRetainedOwners::Spouse, [133, 323]),
        ] {
            let mut d = retained_niit_return(injured, true);
            d.screening.as_mut().unwrap().capital_history[0].injured_niit_owners = Some(prior);
            let r = assert_retained_niit_money(&d);
            let owners = [nii_tax[0] + 900, nii_tax[1] + 450];
            let owners = if injured == Owner::Taxpayer { owners } else { [owners[1], owners[0]] };
            assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], owners);
            assert!(
                r.injured_spouse
                    .as_ref()
                    .unwrap()
                    .required
                    .iter()
                    .all(|row| row.component != "niit_retained_difference_used")
            );
        }
    }
}

#[test]
fn unit13_niit_partial_survivor_zero_native_tax_needs_no_retained_use_answer() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = retained_niit_return(injured, true);
        for entry in &mut d.income {
            if matches!(entry.income, FederalIncome::Wage { .. }) {
                *entry = wage(
                    entry.id,
                    entry.owner,
                    if entry.owner == Owner::Taxpayer { 10_000_000 } else { 5_000_000 },
                    if entry.owner == Owner::Taxpayer { 1_000_000 } else { 500_000 },
                );
            }
        }
        let native = supported(&d);
        assert_eq!(value(&native, "niit_disposition_adjustment"), 5000);
        assert_eq!(value(&native, "niit_capital_carry_adjustment"), 5000);
        // The native report omits Form8960 below its MAGI threshold; this
        // absence is not a missing actual tax pool or an invented NII amount.
        assert!(native.lines.iter().all(|row| row.id != "form_8960_net_investment_income"));
        assert_eq!(value(&native, "net_investment_income_tax"), 0);
        assert_eq!(value(&native, "agi"), 157_000);
        assert_eq!(value(&native, "taxable_income"), 124_800);
        assert_eq!(return_line(&native, "16"), 1_688_000);
        assert_eq!((native.tax, native.balance), (Some(1_688_000), Some(188_000)));
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(ordinary.return_lines, native.return_lines);
        assert_eq!((ordinary.tax, ordinary.balance), (native.tax, native.balance));
        let mut session = restart(d);
        for _ in 0..2 {
            let page = complete_niit_allocation(&mut session);
            let P::SaveInjuredSpouse { .. } = &page else { panic!() };
            let P::SaveInjuredSpouse { value, .. } = &page else { panic!() };
            assert!(
                value
                    .allocations
                    .iter()
                    .all(|row| row.component != "niit_retained_difference_used")
            );
            accepted(&send(&mut session, roundtrip_native_request(page)));
            let draft = roundtrip_native_draft(session.draft.clone());
            session = restart(draft);
            let result = supported(&session.draft);
            assert_eq!(result.return_lines, native.return_lines);
            assert_eq!((result.tax, result.balance), (native.tax, native.balance));
        }
    }
}

#[test]
fn unit13_niit_partial_survivor_opposite_current_exclusions_cannot_mean_known_zero() {
    let mut d = retained_niit_return(Owner::Taxpayer, false);
    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![], amt: vec![] });
    d.income.retain(|entry| matches!(entry.income, FederalIncome::Wage { .. }));
    d.injured_spouse
        .as_mut()
        .unwrap()
        .allocations
        .retain(|row| row.component != "capital_loss_limit");
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(savings_income(20_000, 0));
        entry.owner = owner;
        add(&mut d, entry);
    }
    // These ACTUAL native nonpassive excluded capital components are +5000
    // and−5000. Their sum0 does not establish zero effects for each owner.
    // Passive property20000 keeps the whole1231 class capital, not ordinary.
    add_current_niit_property(&mut d, Owner::Taxpayer, 5000, false);
    add_current_niit_property(&mut d, Owner::Spouse, -5000, false);
    add_current_niit_property(&mut d, Owner::Taxpayer, 20_000, true);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "niit_disposition_adjustment"), 10_000);
    assert_eq!(value(&r, "niit_capital_carry_adjustment"), 0);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 50_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 1900);
    assert_eq!(value(&r, "agi"), 440_000);
    assert_eq!(value(&r, "taxable_income"), 407_800);
    assert_eq!(return_line(&r, "16"), 8_340_800);
    assert_eq!(r.tax, Some(8_665_800));
    assert_eq!(r.balance, Some(575_800));
    let mut plain = d.clone();
    plain.injured_spouse = None;
    plain.checklist.whole_return &= !WholeReturnTopics::INJURED_SPOUSE.0;
    let ordinary = supported(&plain);
    assert_eq!(ordinary.return_lines, r.return_lines);
    assert_eq!((ordinary.tax, ordinary.balance), (r.tax, r.balance));
    let mut session = restart(d);
    for topic in [TaxSection::Farming, TaxSection::InvestmentSales] {
        let page = open_topic(&mut session, topic);
        accepted(&send(&mut session, roundtrip_native_request(page)));
    }
    let draft = roundtrip_native_draft(session.draft.clone());
    let result = supported(&restart(draft).draft);
    assert_eq!(result.return_lines, r.return_lines);
    assert_eq!((result.tax, result.balance), (r.tax, r.balance));
    assert!(
        result
            .lines
            .iter()
            .all(|line| !matches!(line.id.as_str(), "form_8379_line_18_b" | "form_8379_line_18_c")),
        "Offsetting actual current exclusions cannot be assigned to retained prior owners just because the native adjustment equals prior total"
    );
}

#[test]
fn unit13_two_array_save_keeps_native_component_order_and_existing_amounts() {
    let d = retained_niit_return(Owner::Taxpayer, false);
    let r = assert_retained_niit_money(&d);
    let expected: Vec<_> = r
        .injured_spouse
        .as_ref()
        .unwrap()
        .required
        .iter()
        .map(|row| row.component.clone())
        .collect();
    assert_eq!(
        expected,
        ["capital_loss_limit", "niit_retained_difference_used", "additional_medicare_tax"]
    );
    let mut session = restart(d);
    let mut request =
        niit_allocation_with_answers(&mut session, &[("niit_retained_difference_used", 100_000)]);
    let P::SaveInjuredSpouse { value, .. } = &mut request else { unreachable!() };
    // Keep the actual source partition after entering the explicit new answer.
    let index = value
        .allocations
        .iter()
        .position(|row| row.component == "niit_retained_difference_used")
        .unwrap();
    value.new_allocations.push(value.allocations.remove(index));
    let submitted: Vec<_> = value.allocation_rows().map(|row| row.component.clone()).collect();
    assert_ne!(
        submitted, expected,
        "new middle component follows accepted rows in the request partition"
    );
    accepted(&send(&mut session, request));
    assert_retained_niit_money(&session.draft);
    let saved = session.draft.injured_spouse.clone().unwrap();
    assert_eq!(
        saved.allocations.iter().map(|row| row.component.clone()).collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        saved.allocations.iter().map(|row| row.injured).collect::<Vec<_>>(),
        [-150_000, 100_000, 90_000]
    );
    let draft = roundtrip_native_draft(session.draft.clone());
    let mut cold = restart(draft);
    let request = complete_niit_allocation(&mut cold);
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft.injured_spouse.as_ref().unwrap(), &saved);
    assert_retained_niit_money(&cold.draft);
}
