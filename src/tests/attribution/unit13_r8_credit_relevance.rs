use super::*;
use crate::tests::cross::interview::{accepted, from_review, open_topic};
use crate::tests::support::assert::injured_credit::r8_credit_relevance::estimate;
use crate::tests::support::client::injured_credit::r8_credit_relevance::stored_restart;

#[test]
fn r8_adoption_person_allocation_does_not_establish_refund_fund_property() {
    let mut observations = vec![];
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut a = allocation();
            a.injured = injured;
            a.community = InjuredCommunityFacts::Yes {
                states: vec![5],
                debt_facts: InjuredDebtFacts::NotApplicable,
            };
            // Original verified direct credit allocation:3000 native Dollars,
            //700 injured/2300 other. The accepted fact describes people only.
            a.allocations.push(InjuredAmount {
                component: "adoption_refundable_credit".into(),
                injured: 70000,
            });
            let mut d = injured_return((60000, 5000), (20000, 500), 0, a);
            d.credits.as_mut().unwrap().adoptions.push(adoption_child(
                AdoptionKind::Domestic {
                    finalized: AdoptionFinalization::Pending,
                    special_needs: false,
                },
                &[(2025, 3000)],
            ));
            for e in &mut d.income {
                e.community = CommunityShare::Community;
            }
            if reverse {
                d.income.reverse();
            }
            let check = |d: &FederalDraft| {
                let r = supported(d);
                assert_eq!(value(&r, "adoption_refundable_credit"), 3000);
                assert_eq!(column(&r, "17", "a"), 3000);
                assert_eq!(column(&r, "17", "b"), 700);
                assert_eq!(column(&r, "17", "c"), 2300);
                assert!(!noted(&r, "WARN_8379_ADOPTION_OWNER_UNAVAILABLE"));
                assert_eq!(r.refundable_credits, Some(300000));
                assert!(r.refund.is_some_and(|v| v > 0));
                let mut ordinary = d.clone();
                ordinary.injured_spouse = None;
                let joint = supported(&ordinary);
                assert_eq!(r.return_lines, joint.return_lines);
                assert_eq!(
                    (r.tax, r.payments, r.refund, r.balance),
                    (joint.tax, joint.payments, joint.refund, joint.balance)
                );
                r
            };
            check(&d);
            let ids = d.income.clone();
            let mut s = restart(d);
            let p = open_topic(&mut s, TaxSection::Wages);
            accepted(&send(&mut s, p));
            let Reply::Edit(p) = from_review(&mut s, TaxSection::MoreCredits) else { panic!() };
            accepted(&send(&mut s, p));
            let Reply::Edit(p) = native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[])
            else {
                panic!()
            };
            accepted(&send(&mut s, p));
            let mut s = restart(stored_restart(s.draft));
            let Reply::Edit(p) = from_review(&mut s, TaxSection::MoreCredits) else { panic!() };
            accepted(&send(&mut s, p));
            for e in &ids {
                assert_eq!(s.draft.income.iter().find(|r| r.id == e.id), Some(e));
            }
            let r = check(&s.draft);
            observations.push((injured, reverse, estimate(&r)));
        }
    }
    assert!(
        observations.iter().all(|(_, _, v)| v.is_none()),
        "ordinary credit/person/public Save/restart checks passed; untraced non-EIC credit must not ride cash-property weights: {observations:?}"
    );
}

#[test]
fn r8_known_zero_overpayment_does_not_require_texas_management() {
    let mut observations = vec![];
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut a = allocation();
        a.injured = injured;
        a.debt = OffsetDebt::FederalTax;
        a.community = InjuredCommunityFacts::Yes {
            states: vec![7],
            debt_facts: InjuredDebtFacts::BeforeMarriage,
        };
        let mut d = injured_return((22500, 0), (0, 0), 0, a);
        let mut e = income(FederalIncome::EmployeeWages {
            data: EmployeeWages {
                wages: 750000,
                withholding: 250,
                form1099: true,
                reason: EmployeeWageReason::Ss8Pending { filed_on: 20260501 },
                complete: true,
            },
        });
        e.owner = Owner::Spouse;
        e.label.clear();
        add(&mut d, e);
        for e in &mut d.income {
            e.community = CommunityShare::Community;
        }
        let check = |d: &FederalDraft| {
            let r = supported(d);
            assert_eq!(value(&r, "agi"), 30000);
            assert_eq!(value(&r, "regular_income_tax"), 0);
            assert_eq!(return_line(&r, "25b"), 300);
            assert!(noted(
                &r,
                "NOTE_FORM_8919_CODE_G_RETAIN_THE_SEPARATELY_FILED_FORM_SS_8_AND_ITS_F_74675BA7"
            ));
            assert!(r.financial_source_scope_complete);
            assert_eq!(
                (r.tax, r.payments, r.refund, r.balance),
                (Some(57400), Some(300), Some(0), Some(57100))
            );
            let mut ordinary = d.clone();
            ordinary.injured_spouse = None;
            let joint = supported(&ordinary);
            assert_eq!(r.return_lines, joint.return_lines);
            assert_eq!(
                (r.tax, r.payments, r.refund, r.balance),
                (joint.tax, joint.payments, joint.refund, joint.balance)
            );
            r
        };
        let r = check(&d);
        let source = d
            .income
            .iter()
            .find(|e| matches!(e.income, FederalIncome::EmployeeWages { .. }))
            .unwrap();
        observations.push(crate::injured_spouse::management_needed(&d, &r, source));
        // A source Save with no manager answer is valid here. Keep the exact
        // response as evidence before acceptance; no fake answer/minimum.
        let mut s = restart(d);
        let p = open_topic(&mut s, TaxSection::Wages);
        accepted(&send(&mut s, p));
        let restored = stored_restart(s.draft);
        check(&restored);
        assert!(restored.income.iter().all(|e| e.refund_management.is_none()));
    }
    assert_eq!(observations, vec![Some(false), Some(false)]);
}

#[test]
fn r8_unavailable_ordinary_result_keeps_management_unknown_and_income_saveable() {
    let mut a = allocation();
    a.debt = OffsetDebt::FederalTax;
    a.community = InjuredCommunityFacts::Yes {
        states: vec![7],
        debt_facts: InjuredDebtFacts::BeforeMarriage,
    };
    let mut d = injured_return((30000, 1000), (0, 0), 0, a);
    for e in &mut d.income {
        e.community = CommunityShare::Community;
    }
    // The prior0f3 fixture unset deductions, which normalfill_unasked
    // legitimately defaults. Unentered payments remain an actual unfinished
    // ordinary page across reconstruction; they are not a zero payment fact.
    d.payments = None;
    let r = crate::federal_rules::estimate(&d);
    assert_eq!(r.state, ResultState::Incomplete);
    assert!(!r.financial_source_scope_complete);
    assert_eq!((r.tax, r.payments, r.refund, r.balance), (None, None, None, None));
    for e in &d.income {
        assert_eq!(crate::injured_spouse::management_needed(&d, &r, e), None);
    }
    let ids = d.income.clone();
    let mut s = restart(d);
    assert!(s.draft.payments.is_none());
    let p = open_topic(&mut s, TaxSection::Wages);
    accepted(&send(&mut s, p));
    let d = stored_restart(s.draft);
    for e in &ids {
        assert_eq!(d.income.iter().find(|r| r.id == e.id), Some(e));
    }
    assert!(d.payments.is_none());
    assert!(d.income.iter().all(|e| e.refund_management.is_none()));
    let r = crate::federal_rules::estimate(&d);
    assert_eq!((r.tax, r.payments, r.refund, r.balance), (None, None, None, None));
}

#[test]
fn r8_omitted_ordinary_source_does_not_establish_positive_refund_applicability() {
    for management in [None, Some(RefundManagement::Joint)] {
        let mut a = allocation();
        a.debt = OffsetDebt::FederalTax;
        a.community = InjuredCommunityFacts::Yes {
            states: vec![7],
            debt_facts: InjuredDebtFacts::BeforeMarriage,
        };
        let mut d = injured_return((22500, 0), (0, 0), 0, a);
        let mut e = income(FederalIncome::EmployeeWages {
            data: EmployeeWages {
                wages: 750000,
                withholding: 250,
                form1099: true,
                reason: EmployeeWageReason::Ss8Pending { filed_on: 20260501 },
                complete: false,
            },
        });
        e.owner = Owner::Spouse;
        e.label.clear();
        e.refund_management = management;
        add(&mut d, e);
        for e in &mut d.income {
            e.community = CommunityShare::Community;
        }
        let check = |d: &FederalDraft| {
            let r = supported(d);
            assert_eq!(value(&r, "agi"), 30000);
            // Actual prototype result omits unconfirmed coverage. This is
            // explicitly NOT a legal zero payroll liability or a whole-return
            // refund: its owning omission scope must remain unavailable.
            assert_eq!(
                (r.tax, r.payments, r.refund, r.balance),
                (Some(0), Some(300), Some(300), Some(0))
            );
            assert!(!r.financial_source_scope_complete);
            let source = d
                .income
                .iter()
                .find(|e| matches!(e.income, FederalIncome::EmployeeWages { .. }))
                .unwrap();
            assert_eq!(crate::injured_spouse::management_needed(d, &r, source), None);
            let mut plain = d.clone();
            plain.injured_spouse = None;
            let ordinary = supported(&plain);
            assert_eq!(r.return_lines, ordinary.return_lines);
            assert_eq!(
                (r.tax, r.payments, r.refund, r.balance),
                (ordinary.tax, ordinary.payments, ordinary.refund, ordinary.balance)
            );
        };
        check(&d);
        let old = d.income.clone();
        let mut s = restart(d);
        let p = open_topic(&mut s, TaxSection::Wages);
        accepted(&send(&mut s, p));
        let d = stored_restart(s.draft);
        for e in &old {
            assert_eq!(d.income.iter().find(|r| r.id == e.id), Some(e));
        }
        check(&d);
    }
}
