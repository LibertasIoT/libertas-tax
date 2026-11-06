extern crate std;
use super::*;
use crate::tests::cross::interview::accepted;

#[test]
fn unit13_niit_binding_cap_known_current_payers_bound_existing_r5_answer() {
    let mut mismatches = Vec::new();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let d = persist_niit_binding(niit_binding_fixture(injured, reversed));
            let r = assert_niit_binding_money(&d);
            let row = r
                .injured_spouse
                .as_ref()
                .unwrap()
                .required
                .iter()
                .find(|row| row.component == "INJURED_COMPONENT_ITEMIZED")
                .unwrap();
            assert_eq!(row.joint, 500_000);
            let expected =
                if injured == Owner::Taxpayer { (0, 200_000) } else { (300_000, 500_000) };
            std::println!(
                "R5 {injured:?} reversed={reversed}: actual {:?}, expected {expected:?}",
                (row.minimum, row.maximum)
            );
            if (row.minimum, row.maximum) != expected {
                mismatches.push((injured, reversed, (row.minimum, row.maximum), expected));
            }
        }
    }
    assert!(mismatches.is_empty(), "known native paid-capacity bounds: {mismatches:?}");
}

#[test]
fn unit13_niit_binding_cap_verified_same_pool_reuses_existing_r5_answer() {
    let mut mismatches = Vec::new();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let d = persist_niit_binding(niit_binding_fixture(injured, reversed));
            let r = assert_niit_binding_money(&d);
            assert_eq!(d.injured_spouse.as_ref().unwrap().allocations.len(), 1);
            assert_eq!(column(&r, "15", "b"), if injured == Owner::Taxpayer { 1000 } else { 4000 });
            assert_eq!(column(&r, "15", "c"), if injured == Owner::Taxpayer { 4000 } else { 1000 });
            let owners = [456, 304];
            assert_eq!(column(&r, "18", "a"), 760);
            let actual = ["b", "c"].map(|col| {
                r.lines
                    .iter()
                    .find(|line| line.id == alloc::format!("form_8379_line_18_{col}"))
                    .map(|line| line.amount / 100)
            });
            let expected = [
                Some(owners[usize::from(injured == Owner::Spouse)]),
                Some(owners[usize::from(injured != Owner::Spouse)]),
            ];
            std::println!(
                "NIIT {injured:?} reversed={reversed}: actual {actual:?}, expected {expected:?}"
            );
            if actual != expected {
                mismatches.push((injured, reversed, actual, expected));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "same actual ScheduleA/NIIT pool has one explicit R5 answer: {mismatches:?}"
    );
}

#[test]
fn unit13_niit_binding_cap_endpoints_keep_one_accepted_allocation() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            for (taxpayer_deduction, expected) in
                [(0, [494, 266]), (1000, [456, 304]), (2000, [418, 342])]
            {
                let mut d = niit_binding_fixture(injured, reversed);
                d.injured_spouse.as_mut().unwrap().allocations[0].injured =
                    (if injured == Owner::Taxpayer {
                        taxpayer_deduction
                    } else {
                        5000 - taxpayer_deduction
                    }) * 100;
                let d = persist_niit_binding(d);
                let r = assert_niit_binding_money(&d);
                assert_eq!(d.injured_spouse.as_ref().unwrap().allocations.len(), 1);
                assert_eq!(column(&r, "18", "b"), expected[usize::from(injured == Owner::Spouse)]);
                assert_eq!(column(&r, "18", "c"), expected[usize::from(injured != Owner::Spouse)]);
            }
        }
    }
}

#[test]
fn unit13_niit_binding_cap_missing_r5_answer_is_not_a_minimum_fact() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut d = niit_binding_fixture(injured, reversed);
            d.injured_spouse.as_mut().unwrap().allocations.clear();
            let r = assert_niit_binding_money(&d);
            assert!(r.lines.iter().all(|line| !matches!(
                line.id.as_str(),
                "form_8379_line_18_b" | "form_8379_line_18_c"
            )));
            assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
            let mut session = restart(d);
            let mut request = niit_allocation_with_answers(
                &mut session,
                &[(
                    "INJURED_COMPONENT_ITEMIZED",
                    if injured == Owner::Taxpayer { 100_000 } else { 400_000 },
                )],
            );
            let P::SaveInjuredSpouse { value, .. } = &mut request else { panic!() };
            let row = value
                .allocations
                .iter_mut()
                .find(|row| row.component == "INJURED_COMPONENT_ITEMIZED")
                .unwrap();
            row.injured = if injured == Owner::Taxpayer { 100_000 } else { 400_000 };
            accepted(&send(&mut session, request));
            let r = assert_niit_binding_money(&session.draft);
            assert_eq!(column(&r, "18", "b"), if injured == Owner::Taxpayer { 456 } else { 304 });
        }
    }
}

#[test]
fn unit13_niit_binding_cap_invalid_owner_amount_is_refused_and_bounds_regenerate() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut session =
                restart(persist_niit_binding(niit_binding_fixture(injured, reversed)));
            let before = session.draft.clone();
            let mut request = complete_niit_allocation(&mut session);
            let P::SaveInjuredSpouse { value, .. } = &mut request else { panic!() };
            let row = value
                .allocations
                .iter_mut()
                .find(|row| row.component == "INJURED_COMPONENT_ITEMIZED")
                .unwrap();
            // Both represent the invalid actual-person allocation TP3000/S2000.
            row.injured = if injured == Owner::Taxpayer { 300_000 } else { 200_000 };
            assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
            assert_eq!(session.draft, before);
            let mut request = complete_niit_allocation(&mut session);
            let P::SaveInjuredSpouse { value, .. } = &mut request else { panic!() };
            let row = value
                .allocations
                .iter_mut()
                .find(|row| row.component == "INJURED_COMPONENT_ITEMIZED")
                .unwrap();
            row.injured = if injured == Owner::Taxpayer { 300_000 } else { 200_000 };
            row.minimum = 0;
            row.maximum = 500_000;
            let Reply::Response(P::BeginInjuredSpouse { value, .. }) = send(&mut session, request)
            else {
                panic!("expected actual native catalog regeneration")
            };
            let row = value
                .new_allocations
                .iter()
                .find(|row| row.component == "INJURED_COMPONENT_ITEMIZED")
                .unwrap();
            assert_eq!(
                (row.minimum, row.maximum),
                if injured == Owner::Taxpayer { (0, 200_000) } else { (300_000, 500_000) }
            );
            assert_eq!(session.draft, before);
            assert_niit_binding_money(&session.draft);
        }
    }
}

#[test]
fn unit13_niit_binding_cap_source_correction_makes_single_owner_automatic_and_cleans_old_choice() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let source_fixture = crate::tests::support::builders::known_fact_partnership_draft(
                niit_binding_fixture(injured, reversed),
            );
            let d = persist_niit_binding(source_fixture);
            let mut ids: Vec<_> = d.income.iter().map(|entry| entry.id).collect();
            ids.sort_unstable();
            let source = d
                .income
                .iter()
                .find(|entry| {
                    entry.owner == crate::DocumentOwner::Yours
                        && matches!(entry.income, FederalIncome::Partnership { .. })
                })
                .unwrap()
                .id;
            let mut session = restart(d);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let P::SaveDocuments { documents, .. } = &mut request else { panic!() };
            let document =
                documents.common.k1.documents.iter_mut().find(|entry| entry.id == source).unwrap();
            let crate::K1Document::K1Partnership(data) = &mut document.document else { panic!() };
            data.form.deductions.retain(|deduction| deduction.code != "H");
            accepted(&send(&mut session, request));
            let r = assert_niit_binding_money_with_paid(&session.draft, 10000);
            assert_eq!(column(&r, "15", "b"), if injured == Owner::Taxpayer { 0 } else { 5000 });
            assert_eq!(column(&r, "18", "b"), if injured == Owner::Taxpayer { 494 } else { 266 });
            let request = complete_niit_allocation(&mut session);
            let P::SaveInjuredSpouse { value, .. } = &request else { panic!() };
            assert!(
                value.allocations.iter().all(|row| row.component != "INJURED_COMPONENT_ITEMIZED")
            );
            let known = if injured == Owner::Taxpayer { 0 } else { 500_000 };
            assert!(value.known.iter().any(|row| row.joint == 500_000
                && row.injured == known
                && row.other == 500_000 - known));
            accepted(&send(&mut session, request));
            assert!(
                session
                    .draft
                    .injured_spouse
                    .as_ref()
                    .unwrap()
                    .allocations
                    .iter()
                    .all(|row| row.component != "INJURED_COMPONENT_ITEMIZED")
            );
            let draft = roundtrip_native_draft(session.draft.clone());
            let mut session = restart(draft);
            let request = complete_niit_allocation(&mut session);
            let P::SaveInjuredSpouse { value, .. } = &request else { panic!() };
            assert!(
                value.allocations.iter().all(|row| row.component != "INJURED_COMPONENT_ITEMIZED")
            );
            let known = if injured == Owner::Taxpayer { 0 } else { 500_000 };
            assert!(value.known.iter().any(|row| row.joint == 500_000
                && row.injured == known
                && row.other == 500_000 - known));
            accepted(&send(&mut session, request));
            let r = assert_niit_binding_money_with_paid(&session.draft, 10000);
            assert_eq!(column(&r, "18", "b"), if injured == Owner::Taxpayer { 494 } else { 266 });
            let mut after_ids: Vec<_> = session.draft.income.iter().map(|entry| entry.id).collect();
            after_ids.sort_unstable();
            assert_eq!(ids, after_ids);
        }
    }
}

#[test]
fn unit13_niit_binding_cap_scalar_equality_does_not_establish_pure_composition() {
    use crate::injured_spouse::niit_deductions::{NativeDeductionOperands, NiitItemizedDeductions};
    let d = niit_binding_fixture(Owner::Taxpayer, false);
    assert_niit_binding_money(&d);
    let packet = NiitItemizedDeductions::from_native(
        &d,
        NativeDeductionOperands {
            itemizing: true,
            portfolio_only_itemized: false,
            same_itemized_pool: false,
            whole_itemized_pool: false,
            whole_itemized_dollars: 0,
            whole_income_tax_dollars: None,
            allowed_dollars: 5000,
            allowed_available: true,
            before_dollars: [0, 5000, 0, 0, 0],
            income_tax_available: true,
            foreign_cents: Some([0, 0]),
            foreign_capacities: None,
            estate_ird_cents: [0, 0],
            current_k1_interest_cents: [200_000, 1_000_000],
        },
    );
    assert!(packet.pure_interest_capacities.is_none());
    assert!(packet.owners_dollars.is_none());
}
