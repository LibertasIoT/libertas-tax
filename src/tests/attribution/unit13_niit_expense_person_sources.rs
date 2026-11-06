use super::*;
use crate::tests::cross::interview::{accepted, open_document_center};

#[test]
fn canonical_expense_person_exclusive_owners_public_save_restart_and_correction() {
    for source in [
        NiitExpenseSource::Paid,
        NiitExpenseSource::Carry,
        NiitExpenseSource::Property,
        NiitExpenseSource::InvestmentTax,
        NiitExpenseSource::PassiveTax,
        NiitExpenseSource::IncomeTax,
    ] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reversed in [false, true] {
                    let draft = source.niit_deduction_fixture(owner, injured, reversed);
                    let before = assert_niit_person_money(&draft, source);
                    let mut ids: Vec<_> = draft.income.iter().map(|v| v.id).collect();
                    ids.sort_unstable();
                    let mut session = restart(draft);
                    crate::tests::support::client::admit_native_fixture_facts(&mut session);
                    let request = open_document_center(&mut session);
                    accepted(&send(&mut session, request));
                    let mut request = niit_person_editor(&mut session, source);
                    source.supply(&mut request, Some(deduction_owner_fact(owner)));
                    accepted(&send(&mut session, request));
                    // Normal reconstruction removes obsolete whole-itemized
                    // answers only when actual exclusive capacities prove a
                    // singleton. It never fabricates a joint selection.
                    let request = complete_niit_allocation(&mut session);
                    accepted(&send(&mut session, request));
                    let mut session = restart_stored_niit_draft(session);
                    let request = niit_person_editor(&mut session, source);
                    accepted(&send(&mut session, request));
                    let r = assert_niit_person_money(&session.draft, source);
                    assert_eq!(before.return_lines, r.return_lines);
                    let expected = if matches!(source, NiitExpenseSource::IncomeTax) {
                        if owner == Owner::Taxpayer { [756, 380] } else { [760, 376] }
                    } else if owner == Owner::Taxpayer {
                        [722, 380]
                    } else {
                        [760, 342]
                    };
                    assert_eq!(
                        column(&r, "18", "b"),
                        expected[usize::from(injured == Owner::Spouse)],
                        "{source:?}"
                    );
                    assert_eq!(
                        column(&r, "18", "c"),
                        expected[usize::from(injured != Owner::Spouse)],
                        "{source:?}"
                    );
                    let mut after_ids: Vec<_> = session.draft.income.iter().map(|v| v.id).collect();
                    after_ids.sort_unstable();
                    assert_eq!(ids, after_ids);
                    let mut request = niit_person_editor(&mut session, source);
                    source.supply(
                        &mut request,
                        Some(deduction_owner_fact(if owner == Owner::Taxpayer {
                            Owner::Spouse
                        } else {
                            Owner::Taxpayer
                        })),
                    );
                    accepted(&send(&mut session, request));
                    let mut session = restart_stored_niit_draft(session);
                    let request = niit_person_editor(&mut session, source);
                    accepted(&send(&mut session, request));
                    let corrected = assert_niit_person_money(&session.draft, source);
                    assert_eq!(corrected.return_lines, before.return_lines);
                    assert_eq!((corrected.tax, corrected.refund), (before.tax, before.refund));
                }
            }
        }
    }
}

#[test]
fn canonical_expense_missing_person_refuses_only_required_native_source_and_preserves_state() {
    for source in [
        NiitExpenseSource::Paid,
        NiitExpenseSource::Carry,
        NiitExpenseSource::Property,
        NiitExpenseSource::InvestmentTax,
        NiitExpenseSource::PassiveTax,
        NiitExpenseSource::IncomeTax,
    ] {
        for claim in [true, false] {
            let mut draft = source.niit_deduction_fixture(Owner::Taxpayer, Owner::Taxpayer, false);
            if !claim {
                draft.injured_spouse = None;
            }
            assert_niit_person_money(&draft, source);
            let mut session = restart(draft);
            let request = niit_person_editor(&mut session, source);
            let before = session.draft.clone();
            let dirty = session.dirty;
            // 2026-10-14: 59e9/sharedPerson requires an owner only for an
            // actual per-person consumer. Ordinary joint NIIT has no split.
            if !claim {
                accepted(&send(&mut session, request));
                let after = assert_niit_person_money(&session.draft, source);
                let original = assert_niit_person_money(&before, source);
                assert_eq!(after.return_lines, original.return_lines);
                assert_eq!((after.tax, after.refund), (original.tax, original.refund));
                continue;
            }
            let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
                panic!("unknown person must refuse {source:?}")
            };
            assert_eq!(
                message,
                libertas::libertas_formatted_text("EXPENSE_PERSON_OWNER_REQUIRED", &[])
            );
            let field = field.unwrap();
            assert!(field.starts_with("[value/"));
            assert!(field.ends_with("owners]"));
            assert_eq!(session.draft, before);
            assert_eq!(session.dirty, dirty);
        }
    }
}

#[test]
fn canonical_paid_and_retained_person_facts_are_independent() {
    let mut draft =
        NiitExpenseSource::Paid.niit_deduction_fixture(Owner::Taxpayer, Owner::Taxpayer, false);
    let loan =
        &mut draft.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans[0];
    loan.paid = 40_000;
    loan.carry = 60_000;
    loan.paid_owners = Some(DeductionOwners::Taxpayer);
    loan.carry_owners = Some(DeductionOwners::Spouse);
    let before = assert_niit_deduction_money(&draft);
    assert_eq!(value(&before, "investment_form_line1"), 400);
    assert_eq!(value(&before, "investment_form_line2"), 600);
    let mut session = restart(draft);
    let request = niit_person_editor(&mut session, NiitExpenseSource::Paid);
    accepted(&send(&mut session, request));
    let mut session = restart_stored_niit_draft(session);
    let request = niit_person_editor(&mut session, NiitExpenseSource::Paid);
    accepted(&send(&mut session, request));
    let r = assert_niit_deduction_money(&session.draft);
    assert_eq!(column(&r, "18", "b"), 745);
    assert_eq!(column(&r, "18", "c"), 357);
    let loan =
        &session.draft.deductions.as_ref().unwrap().investment_interest.as_ref().unwrap().loans[0];
    assert_eq!(loan.paid_owners, Some(DeductionOwners::Taxpayer));
    assert_eq!(loan.carry_owners, Some(DeductionOwners::Spouse));
}

#[test]
fn canonical_both_identical_pool_reuses_one_real_accepted_r5_answer() {
    for source in [
        NiitExpenseSource::Paid,
        NiitExpenseSource::Carry,
        NiitExpenseSource::Property,
        NiitExpenseSource::InvestmentTax,
        NiitExpenseSource::PassiveTax,
    ] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = source.niit_deduction_fixture(Owner::Taxpayer, injured, false);
            draft.injured_spouse.as_mut().unwrap().allocations[0].injured =
                if injured == Owner::Taxpayer { 40_000 } else { 60_000 };
            assert_niit_deduction_money(&draft);
            let mut session = restart(draft);
            let mut request = niit_person_editor(&mut session, source);
            source.supply(&mut request, Some(DeductionOwners::Both));
            accepted(&send(&mut session, request));
            let r = assert_niit_deduction_money(&session.draft);
            assert_eq!(column(&r, "18", "b"), if injured == Owner::Taxpayer { 745 } else { 357 });
            assert_eq!(column(&r, "18", "c"), if injured == Owner::Taxpayer { 357 } else { 745 });
            let plan = r.injured_spouse.as_ref().unwrap();
            assert_eq!(
                plan.required
                    .iter()
                    .filter(|row| row.component == "INJURED_COMPONENT_ITEMIZED")
                    .count(),
                1
            );
            assert!(plan.required.iter().all(|row| row.component != "niit_itemized_deduction"));
            let request = niit_person_editor(&mut session, source);
            accepted(&send(&mut session, request));
            let session = restart_stored_niit_draft(session);
            let r = assert_niit_deduction_money(&session.draft);
            assert_eq!(column(&r, "18", "a"), 1102);
        }
    }
}

#[test]
fn canonical_both_nonidentical_income_tax_subset_has_distinct_consistent_bounds() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut draft =
            NiitExpenseSource::IncomeTax.niit_deduction_fixture(Owner::Taxpayer, injured, false);
        itemized(draft.deductions.as_mut().unwrap()).income_tax_owners =
            Some(DeductionOwners::Both);
        draft.injured_spouse.as_mut().unwrap().allocations[0].injured = 50_000;
        let r = assert_niit_person_money(&draft, NiitExpenseSource::IncomeTax);
        let mut session = restart(draft);
        let request = niit_person_editor(&mut session, NiitExpenseSource::IncomeTax);
        accepted(&send(&mut session, request));
        let session = restart_stored_niit_draft(session);
        assert_eq!(
            assert_niit_person_money(&session.draft, NiitExpenseSource::IncomeTax).return_lines,
            r.return_lines
        );
        // Independent bounds: whole paid pool1000, eligible D107 and
        // noneligible E893. Accepted R5Y500 intersects D0..107 with
        // [Y-Emax,Y-Emin]=[-393,500], hence0..107 Dollars.
        let r = assert_niit_person_money(&session.draft, NiitExpenseSource::IncomeTax);
        let row = r
            .injured_spouse
            .as_ref()
            .unwrap()
            .required
            .iter()
            .find(|row| row.component == "niit_itemized_deduction")
            .expect("distinct shared NIIT subset, not another whole R5 amount");
        assert_eq!((row.joint, row.minimum, row.maximum), (10_700, 0, 10_700));
        assert!(
            r.lines.iter().all(|line| !matches!(
                line.id.as_str(),
                "form_8379_line_18_b" | "form_8379_line_18_c"
            )),
            "unanswered shared subset cannot use a minimum prefill"
        );
    }
}

#[test]
fn canonical_distinct_subset_intersects_real_r5_and_reconstructs_after_correction() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            for (whole, lower, upper) in
                [(0, 0, 0), (50, 0, 50), (500, 0, 107), (950, 57, 107), (1000, 107, 107)]
            {
                let mut draft = NiitExpenseSource::IncomeTax.niit_deduction_fixture(
                    Owner::Taxpayer,
                    injured,
                    reversed,
                );
                itemized(draft.deductions.as_mut().unwrap()).income_tax_owners =
                    Some(DeductionOwners::Both);
                draft.injured_spouse.as_mut().unwrap().allocations[0].injured = whole * 100;
                let before = assert_niit_person_money(&draft, NiitExpenseSource::IncomeTax);
                let mut session = restart(draft);
                let request = niit_person_editor(&mut session, NiitExpenseSource::IncomeTax);
                accepted(&send(&mut session, request));
                let pending =
                    assert_niit_person_money(&session.draft, NiitExpenseSource::IncomeTax);
                let required = pending
                    .injured_spouse
                    .as_ref()
                    .unwrap()
                    .required
                    .iter()
                    .find(|row| row.component == "niit_itemized_deduction");
                if lower == upper {
                    assert!(required.is_none(), "unique subset never asks a question");
                    let request = complete_niit_allocation(&mut session);
                    accepted(&send(&mut session, request));
                } else {
                    let row = required.unwrap();
                    assert_eq!(
                        (row.joint, row.minimum, row.maximum),
                        (10700, lower * 100, upper * 100)
                    );
                    assert!(!pending.lines.iter().any(|line| line.id == "form_8379_line_18_b"));
                    let invalid = if lower > 0 { lower - 1 } else { upper + 1 };
                    let request = niit_allocation_with_answers(
                        &mut session,
                        &[("niit_itemized_deduction", invalid * 100)],
                    );
                    let saved = session.draft.clone();
                    let dirty = session.dirty;
                    assert!(matches!(
                        send(&mut session, request),
                        Reply::Response(P::Problem { .. })
                    ));
                    assert_eq!(session.draft, saved);
                    assert_eq!(session.dirty, dirty);
                    let request = niit_allocation_with_answers(
                        &mut session,
                        &[("niit_itemized_deduction", lower * 100)],
                    );
                    accepted(&send(&mut session, request));
                    // The upper endpoint is a source correction through the
                    // actual retained row, not a new fabricated answer.
                    let mut request = complete_niit_allocation(&mut session);
                    let P::SaveInjuredSpouse { value, .. } = &mut request else { unreachable!() };
                    value
                        .allocations
                        .iter_mut()
                        .find(|row| row.component == "niit_itemized_deduction")
                        .unwrap()
                        .injured = upper * 100;
                    accepted(&send(&mut session, request));
                }
                let mut session = restart_stored_niit_draft(session);
                let request = complete_niit_allocation(&mut session);
                accepted(&send(&mut session, request));
                let after = assert_niit_person_money(&session.draft, NiitExpenseSource::IncomeTax);
                assert_eq!(before.return_lines, after.return_lines);
                assert_eq!(column(&after, "18", "a"), 1136);
                assert_eq!(column(&after, "18", "b") + column(&after, "18", "c"), 1136);
                if lower != upper {
                    // Exclusive correction makes both R5 and D automatic;
                    // normal reconstruction removes the now-obsolete choices.
                    let mut request =
                        niit_person_editor(&mut session, NiitExpenseSource::IncomeTax);
                    NiitExpenseSource::IncomeTax
                        .supply(&mut request, Some(deduction_owner_fact(injured)));
                    accepted(&send(&mut session, request));
                    let request = complete_niit_allocation(&mut session);
                    accepted(&send(&mut session, request));
                    let session = restart_stored_niit_draft(session);
                    assert!(session.draft.injured_spouse.as_ref().unwrap().allocations.iter().all(
                        |row| !matches!(
                            row.component.as_str(),
                            "INJURED_COMPONENT_ITEMIZED" | "niit_itemized_deduction"
                        )
                    ));
                    let after =
                        assert_niit_person_money(&session.draft, NiitExpenseSource::IncomeTax);
                    assert_eq!(before.return_lines, after.return_lines);
                    assert_eq!(
                        column(&after, "18", "b"),
                        if injured == Owner::Taxpayer { 756 } else { 376 }
                    );
                }
            }
        }
    }
}

#[test]
fn canonical_unrelated_4952_records_do_not_supply_niit_tax_or_turn_withheld_amount_into_zero() {
    for claim in [false, true] {
        let mut draft = NiitExpenseSource::IncomeTax.niit_deduction_fixture(
            Owner::Taxpayer,
            Owner::Taxpayer,
            false,
        );
        itemized(draft.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
            paid: 100000,
            allocation: TaxIncomeAllocation::Unresolved,
        };
        itemized(draft.deductions.as_mut().unwrap()).investment_income_tax =
            InvestmentIncomeTax::Records {
                paid_part: 100000,
                capped_allocation: InvestmentTaxCapAllocation::SourceDetermined,
            };
        // With a claim, known SALT ownership is still required for R5. The
        // separate Records relationship neither supplies nor asks NIIT ownership.
        if claim {
            itemized(draft.deductions.as_mut().unwrap()).income_tax_owners =
                Some(DeductionOwners::Spouse);
        } else {
            draft.injured_spouse = None;
        }
        let check = |draft: &FederalDraft| {
            let r = supported(draft);
            assert_eq!(value(&r, "agi"), 280000);
            assert_eq!(value(&r, "taxable_income"), 279000);
            assert_eq!(value(&r, "regular_income_tax"), 52156);
            // 2026-10-14 authoring correction: the actual native worksheet
            // is withheld with the unresolved income-tax component. Its
            // conservative reported NIIT1140 remains independently pinned.
            assert!(!r.lines.iter().any(|row| row.id == "form_8960_net_investment_income"));
            assert_eq!(value(&r, "net_investment_income_tax"), 1140);
            assert_eq!((r.tax, r.refund), (Some(5329600), Some(1670400)));
            assert!(!r.lines.iter().any(|line| matches!(
                line.id.as_str(),
                "niit_income_tax_deduction" | "niit_itemized_deductions"
            )));
            assert_eq!(r.expense_roles.as_ref().unwrap().connected_dollars, None);
            let mut claimed = draft.clone();
            if claimed.injured_spouse.is_none() {
                let mut answer = allocation();
                answer.allocations.push(InjuredAmount {
                    component: "INJURED_COMPONENT_ITEMIZED".into(),
                    injured: 100000,
                });
                claimed.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
                claimed.injured_spouse = Some(answer);
            }
            let attributed = supported(&claimed);
            assert_eq!(attributed.return_lines, r.return_lines);
            assert!(
                !attributed.lines.iter().any(|line| matches!(
                    line.id.as_str(),
                    "form_8379_line_18_b" | "form_8379_line_18_c"
                )),
                "a withheld allowed amount is not known zero"
            );
        };
        check(&draft);
        let mut session = restart(draft);
        let request = niit_person_editor(&mut session, NiitExpenseSource::IncomeTax);
        accepted(&send(&mut session, request));
        let session = restart_stored_niit_draft(session);
        check(&session.draft);
    }
}

#[test]
fn canonical_native_cap_zero_needs_no_person_answer_and_retains_supplied_history() {
    let mut draft =
        NiitExpenseSource::Paid.niit_deduction_fixture(Owner::Taxpayer, Owner::Taxpayer, false);
    draft.income.retain(|entry| matches!(entry.income, FederalIncome::Wage { .. }));
    let check = |draft: &FederalDraft| {
        let r = supported(draft);
        assert_eq!(value(&r, "agi"), 250000);
        assert_eq!(value(&r, "taxable_income"), 250000);
        assert_eq!(value(&r, "regular_income_tax"), 45196);
        assert_eq!(value(&r, "investment_interest_deduction"), 0);
        // 2026-10-14 authoring correction: MAGI equals the250000 filing
        // threshold, so the native owner emits no Form8960 worksheet. The
        // final actual allowed expense and NIIT are still known zero.
        assert!(!r.lines.iter().any(|row| row.id == "form_8960_net_investment_income"));
        assert_eq!(value(&r, "net_investment_income_tax"), 0);
        assert_eq!((r.tax, r.refund), (Some(4519600), Some(2480400)));
        assert_eq!(value(&r, "niit_itemized_deductions"), 0);
        assert!(
            r.lines.iter().any(|line| line.id == "niit_itemized_deductions"),
            "the actual native zero is emitted"
        );
    };
    check(&draft);
    let mut session = restart(draft);
    let request = niit_person_editor(&mut session, NiitExpenseSource::Paid);
    accepted(&send(&mut session, request));
    let request = complete_niit_allocation(&mut session);
    accepted(&send(&mut session, request));
    let mut session = restart_stored_niit_draft(session);
    check(&session.draft);
    let mut request = niit_person_editor(&mut session, NiitExpenseSource::Paid);
    NiitExpenseSource::Paid.supply(&mut request, Some(DeductionOwners::Spouse));
    accepted(&send(&mut session, request));
    let session = restart_stored_niit_draft(session);
    check(&session.draft);
    assert_eq!(
        session.draft.deductions.as_ref().unwrap().investment_interest.as_ref().unwrap().loans[0]
            .paid_owners,
        Some(DeductionOwners::Spouse)
    );
}
