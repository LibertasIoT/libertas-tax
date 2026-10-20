use super::*;
use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::support::assert::injured_credit::business_credit_owner_sources::{
    payee_columns, payee_limit_money, payee_limit_sources, residual_money,
};
use crate::tests::support::builders::injured_credit::business_credit_owner_sources::{
    PayeeLimitCase, ResidualFamily,
};
use crate::tests::support::client::injured_credit::business_credit_owner_sources::{
    observations, payee_limit_saved, residual_compatibility, residual_saved_with_payer, saved,
};

#[test]
fn unit13_current_business_credit_sole_owner_is_automatic_after_native_use() {
    // Drain every original-money, actual source Save, ID and complete Avro
    // persistence/restart profile before any owner assertion.
    let observations = observations();
    assert_eq!(observations.len(), 8);
    for (owner, injured, reverse, r) in observations {
        let expected = if owner == injured { [5000, 0] } else { [0, 5000] };
        assert_eq!(
            [column(&r, "16", "b"), column(&r, "16", "c")],
            expected,
            "actual current owner{owner:?}/injured{injured:?}/reverse{reverse}"
        );
        // 2026-10-15 new-fixture fitting: the existing source-choice helper
        // may retain an automatic singleton in its catalog. A known owner
        // must never acquire an unrestricted0..5000 allocation question.
        let plan = r.injured_spouse.as_ref().unwrap();
        assert!(plan.required.iter().all(|row| {
            row.joint == 500000
                && row.minimum == expected[0] * 100
                && row.maximum == expected[0] * 100
                && plan.accepted.iter().any(|answer| {
                    answer.component == row.component && answer.injured == expected[0] * 100
                })
        }));
    }
}

#[test]
fn unit13_current_business_credit_stale_aggregate_answer_cannot_override_owner() {
    let mut all = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let wrong_owner = if owner == injured { 0 } else { 500000 };
                for (case, answer) in [("wrong-owner", wrong_owner), ("old-aggregate", 1119200)] {
                    // Existing generic answers can predate a source correction.
                    // They are not new credit facts or a new allocation choice.
                    all.push((
                        owner,
                        injured,
                        reverse,
                        case,
                        saved(owner, injured, reverse, Some(answer)),
                    ));
                }
            }
        }
    }
    // Every source, literal ordinary figure and complete persistence/restart
    // passes before checking the sole native source owner against stale answers.
    assert_eq!(all.len(), 16);
    for (owner, injured, reverse, case, r) in all {
        let expected = if owner == injured { [5000, 0] } else { [0, 5000] };
        assert_eq!(
            [column(&r, "16", "b"), column(&r, "16", "c")],
            expected,
            "{case}/{owner:?}/{injured:?}/{reverse}"
        );
        let plan = r.injured_spouse.as_ref().unwrap();
        assert!(plan.required.iter().all(|row| row.component != "INJURED_COMPONENT_NONREFUNDABLE"));
        assert!(plan.accepted.iter().all(|row| row.component != "INJURED_COMPONENT_NONREFUNDABLE"));
    }
}

#[test]
fn unit13_current_business_credit_foreign_statement_credit_keeps_actual_payer() {
    // 2026-10-15 correction of NEW56d fixture semantics: an actual known1099
    // payer is not a jointly attributable credit merely because its owner feed
    // is missing. Keep every original financial/source oracle, then require
    // actual100FTC plus actual5000currentGBC owners automatically.
    let mut profiles = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for payer in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    profiles.push((
                        owner,
                        payer,
                        injured,
                        reverse,
                        residual_saved_with_payer(
                            ResidualFamily::ForeignTax,
                            owner,
                            payer,
                            injured,
                            reverse,
                        ),
                    ));
                }
            }
        }
    }
    for (_, _, _, _, draft) in &profiles {
        residual_money(draft, ResidualFamily::ForeignTax);
    }
    for (owner, payer, injured, reverse, draft) in profiles {
        let r = residual_money(&draft, ResidualFamily::ForeignTax);
        let known_injured =
            i64::from(owner == injured) * 500000 + i64::from(payer == injured) * 10000;
        let expected = [known_injured, 510000 - known_injured];
        assert_eq!(
            ["b", "c"].map(|column| r
                .lines
                .iter()
                .find(|row| row.id == alloc::format!("form_8379_line_16_{column}"))
                .map(|row| row.amount)),
            expected.map(Some),
            "knownFTCpayer/GBCowner {owner:?}/{payer:?}/{injured:?}/{reverse}"
        );
        let plan = r.injured_spouse.as_ref().unwrap();
        assert!(plan.required.is_empty(), "known credit owners need no direct choice");
        assert!(
            plan.accepted.iter().all(|row| row.component != ResidualFamily::ForeignTax.old_key())
        );
        let construction = draft.injured_spouse.as_ref().unwrap().construction(&draft, &r);
        assert!(construction.allocations.is_empty() && construction.new_allocations.is_empty());
    }
}

#[test]
fn unit13_current_business_credit_old_refundable_answer_does_not_answer_residual() {
    residual_compatibility();
}

#[test]
fn unit13_foreign_statement_sole_payer_keeps_capped_native_credit() {
    let mut profiles = vec![];
    for payer in [Owner::Taxpayer, Owner::Spouse] {
        let business_owner = if payer == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let case = PayeeLimitCase::SoleCap(payer);
                profiles.push((
                    payer,
                    injured,
                    payee_limit_saved(case, business_owner, injured, reverse),
                ));
            }
        }
    }
    for (payer, injured, draft) in profiles {
        let r = payee_limit_money(&draft, PayeeLimitCase::SoleCap(payer));
        assert_eq!(
            payee_columns(&r),
            if payer == injured { [Some(18100), Some(0)] } else { [Some(0), Some(18100)] }
        );
        assert!(r.injured_spouse.as_ref().unwrap().required.is_empty());
    }
}

#[test]
fn unit13_foreign_statement_full_mixed_cents_use_native_injured_first_projection() {
    let mut profiles = vec![];
    for taxes in [[50, 49], [49, 50], [50, 50]] {
        for business_owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    let case = PayeeLimitCase::FullMixed(taxes);
                    profiles.push((
                        taxes,
                        business_owner,
                        injured,
                        payee_limit_saved(case, business_owner, injured, reverse),
                    ));
                }
            }
        }
    }
    for (taxes, business_owner, injured, draft) in profiles {
        let r = payee_limit_money(&draft, PayeeLimitCase::FullMixed(taxes));
        // Literal independent R2 injured-first expectations. No per-document
        // rounding or tax/gross ratio enters these accepted source operands.
        let foreign_injured = match (taxes, injured) {
            ([50, 49], Owner::Taxpayer) | ([49, 50], Owner::Spouse) | ([50, 50], _) => 100,
            _ => 0,
        };
        let injured_total = i64::from(business_owner == injured) * 500000 + foreign_injured;
        assert_eq!(payee_columns(&r), [Some(injured_total), Some(500100 - injured_total)]);
        assert!(r.injured_spouse.as_ref().unwrap().required.is_empty());
    }
}

#[test]
fn unit13_foreign_statement_binding_mixed_cap_requires_bounded_saved_source_use() {
    const KEY: &str = "foreign_nonrefundable_credit";
    const OLD: &str = "INJURED_COMPONENT_NONREFUNDABLE";
    let mut profiles = vec![];
    // All ordinary financial/source persistence profiles precede ownership.
    for business_owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                profiles.push((
                    business_owner,
                    injured,
                    reverse,
                    payee_limit_saved(PayeeLimitCase::MixedCap, business_owner, injured, reverse),
                ));
            }
        }
    }
    for (business_owner, injured, _reverse, mut draft) in profiles {
        // An old aggregate amount inside the new source bounds is not an
        // answer about this source. It must not become the new bounded share.
        let initial_answer = if injured == Owner::Taxpayer { 5000 } else { 13100 };
        draft
            .injured_spouse
            .as_mut()
            .unwrap()
            .allocations
            .push(InjuredAmount { component: OLD.into(), injured: initial_answer });
        let r = payee_limit_money(&draft, PayeeLimitCase::MixedCap);
        assert_eq!(payee_columns(&r), [None, None]);
        let plan = r.injured_spouse.as_ref().unwrap();
        assert!(plan.accepted.is_empty());
        let [row] = plan.required.as_slice() else {
            panic!("one known bounded foreign-credit source use")
        };
        let bounds = if injured == Owner::Taxpayer { (0, 10000) } else { (8100, 18100) };
        assert_eq!(row.component, KEY);
        assert_eq!((row.minimum, row.maximum, row.joint), (bounds.0, bounds.1, 18100));
        let construction = draft.injured_spouse.as_ref().unwrap().construction(&draft, &r);
        assert!(construction.allocations.is_empty());
        let [unanswered] = construction.new_allocations.as_slice() else { panic!() };
        assert_eq!(unanswered.component, KEY);
        assert_eq!(
            (unanswered.minimum, unanswered.maximum, unanswered.joint),
            (bounds.0, bounds.1, 18100)
        );
        let mut s = restart(draft);
        let Reply::Edit(mut obsolete) =
            native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[(KEY, initial_answer)])
        else {
            panic!()
        };
        let P::SaveInjuredSpouse { value, .. } = &mut obsolete else { panic!() };
        let [answer] = value.allocations.as_mut_slice() else { panic!() };
        answer.component = OLD.into();
        assert_eq!(
            crate::injured_spouse::allocation_rule(value, &s.draft, &r),
            Err("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
        );
        let before = s.draft.clone();
        let dirty = s.dirty;
        match send(&mut s, obsolete) {
            Reply::Response(P::BeginInjuredSpouse { value, .. }) => {
                assert!(value.allocations.is_empty());
                let [row] = value.new_allocations.as_slice() else { panic!() };
                assert_eq!(row.component, KEY);
                assert_eq!((row.minimum, row.maximum), bounds);
            }
            Reply::Response(P::Problem { .. }) => {}
            other => panic!("obsolete aggregate cannot save: {other:?}"),
        }
        assert_eq!(s.draft, before);
        assert_eq!(s.draft.revision, before.revision);
        assert_eq!(s.dirty, dirty);
        // Nonproportional explicit use, correction, then both real endpoints.
        // This native completed-request helper is not client initialization.
        for (step, taxpayer) in [5000, 7500, 0, 10000].into_iter().enumerate() {
            let selected = if injured == Owner::Taxpayer { taxpayer } else { 18100 - taxpayer };
            let new_answers = [(KEY, selected)];
            let Reply::Edit(mut request) = native_editor(
                from_review(&mut s, TaxSection::InjuredSpouse),
                if step == 0 { &new_answers } else { &[] },
            ) else {
                panic!()
            };
            let P::SaveInjuredSpouse { value, .. } = &mut request else { panic!() };
            let [answer] = value.allocations.as_mut_slice() else { panic!() };
            assert_eq!(answer.component, KEY);
            answer.injured = selected;
            accepted(&send(&mut s, request));
            let r = payee_limit_money(&s.draft, PayeeLimitCase::MixedCap);
            assert_eq!(payee_columns(&r), [Some(selected), Some(18100 - selected)]);
            // 2026-10-15 NEW lifecycle assertion fit: required is the full
            // catalog; accepted and construction own the answered partition.
            let plan = r.injured_spouse.as_ref().unwrap();
            let [answer] = plan.accepted.as_slice() else { panic!() };
            assert_eq!((answer.component.as_str(), answer.injured), (KEY, selected));
            let construction = s.draft.injured_spouse.as_ref().unwrap().construction(&s.draft, &r);
            assert!(construction.new_allocations.is_empty());
            let [answer] = construction.allocations.as_slice() else { panic!() };
            assert_eq!((answer.component.as_str(), answer.injured), (KEY, selected));
            let [stored_answer] = s.draft.injured_spouse.as_ref().unwrap().allocations.as_slice()
            else {
                panic!()
            };
            assert_eq!((stored_answer.component.as_str(), stored_answer.injured), (KEY, selected));
            let sources = payee_limit_sources(&s.draft, PayeeLimitCase::MixedCap, business_owner);
            let draft = roundtrip_native_draft(s.draft.clone());
            s = restart(draft);
            let r = payee_limit_money(&s.draft, PayeeLimitCase::MixedCap);
            assert_eq!(payee_columns(&r), [Some(selected), Some(18100 - selected)]);
            assert_eq!(
                payee_limit_sources(&s.draft, PayeeLimitCase::MixedCap, business_owner),
                sources
            );
            let Reply::Edit(request) =
                native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[])
            else {
                panic!()
            };
            accepted(&send(&mut s, request));
            let r = payee_limit_money(&s.draft, PayeeLimitCase::MixedCap);
            assert_eq!(payee_columns(&r), [Some(selected), Some(18100 - selected)]);
            assert_eq!(
                payee_limit_sources(&s.draft, PayeeLimitCase::MixedCap, business_owner),
                sources
            );
        }
    }
}
