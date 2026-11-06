use super::*;
use crate::tests::cross::interview::{accepted, from_review};

#[test]
fn unit13_niit_deduction_foreign_payee_keeps_actual_owner_after_native_cap() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d =
                    niit_deduction_fixture(NiitDeductionExpense::Foreign, owner, injured, reversed);
                assert_niit_deduction_money(&d);
                let d = persist_niit_deduction(d, NiitDeductionExpense::Foreign);
                let r = assert_niit_deduction_money(&d);
                let expected = if owner == Owner::Taxpayer { [722, 380] } else { [760, 342] };
                assert_eq!(column(&r, "18", "a"), 1102);
                assert_eq!(column(&r, "18", "b"), expected[usize::from(injured == Owner::Spouse)]);
                assert_eq!(column(&r, "18", "c"), expected[usize::from(injured != Owner::Spouse)]);
            }
        }
    }
}

#[test]
fn unit13_niit_deduction_estate_ird_keeps_beneficiary_owner_and_other_interest() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d =
                    niit_deduction_fixture(NiitDeductionExpense::Estate, owner, injured, reversed);
                assert_niit_deduction_money(&d);
                let d = persist_niit_deduction(d, NiitDeductionExpense::Estate);
                let r = assert_niit_deduction_money(&d);
                let expected = if owner == Owner::Taxpayer { [342, 760] } else { [760, 342] };
                assert_eq!(column(&r, "18", "b"), expected[usize::from(injured == Owner::Spouse)]);
                assert_eq!(column(&r, "18", "c"), expected[usize::from(injured != Owner::Spouse)]);
            }
        }
    }
}

#[test]
fn unit13_niit_deduction_aggregate_sources_stay_unknown_despite_known_income() {
    for kind in [
        NiitDeductionExpense::ManualForeign,
        NiitDeductionExpense::Property,
        NiitDeductionExpense::Portfolio,
        NiitDeductionExpense::PortfolioCarry,
    ] {
        let d = niit_deduction_fixture(kind, Owner::Taxpayer, Owner::Taxpayer, false);
        assert_niit_deduction_money(&d);
        // 2026-10-14 decision59e9ce8a requires the actual person on the
        // positive native role. The dated accepted-unknown witness and its
        // unchanged monetary assertions remain archived; public Save now
        // refuses the missing relationship instead of persisting it.
        let mut session = restart(d);
        let topic = if matches!(kind, NiitDeductionExpense::ManualForeign) {
            TaxSection::MoreCredits
        } else {
            TaxSection::OtherItemized
        };
        let Reply::Edit(request) = from_review(&mut session, topic) else { panic!() };
        let before = session.draft.clone();
        assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
        assert_eq!(session.draft, before);
        let r = assert_niit_deduction_money(&session.draft);
        assert!(r.lines.iter().all(|line| !matches!(
            line.id.as_str(),
            "form_8379_line_18_b" | "form_8379_line_18_c"
        )));
    }
}

#[test]
fn unit13_niit_deduction_current_k1_interest_uses_actual_basis_allowed_owner() {
    for kind in [NiitDeductionExpense::PartnerInterest, NiitDeductionExpense::ScorpInterest] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reversed in [false, true] {
                    let d = niit_deduction_fixture(kind, owner, injured, reversed);
                    assert_niit_deduction_money(&d);
                    let d = persist_niit_deduction(d, kind);
                    let r = assert_niit_deduction_money(&d);
                    assert_eq!(value(&r, "investment_interest_deduction"), 1000);
                    let expected = if owner == Owner::Taxpayer { [722, 380] } else { [760, 342] };
                    assert_eq!(
                        column(&r, "18", "b"),
                        expected[usize::from(injured == Owner::Spouse)]
                    );
                    assert_eq!(
                        column(&r, "18", "c"),
                        expected[usize::from(injured != Owner::Spouse)]
                    );
                }
            }
        }
    }
}

#[test]
fn unit13_niit_deduction_known_fractional_payees_use_injured_first_native_residual() {
    use crate::injured_spouse::niit_deductions::{NativeDeductionOperands, NiitItemizedDeductions};
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut d = niit_deduction_fixture(
                NiitDeductionExpense::Foreign,
                Owner::Taxpayer,
                injured,
                reversed,
            );
            for entry in &mut d.income {
                if let FederalIncome::Interest { data } = &mut entry.income {
                    edit_interest(data).foreign_tax = 50;
                    edit_interest(data).foreign_country = "CA".into();
                }
            }
            d.injured_spouse.as_mut().unwrap().allocations[0].injured = 100;
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 280_000);
            assert_eq!(value(&r, "taxable_income"), 279_999);
            assert_eq!(value(&r, "regular_income_tax"), 52_396);
            assert_eq!(value(&r, "niit_itemized_deductions"), 1);
            assert_eq!(value(&r, "form_8960_net_investment_income"), 29_999);
            assert_eq!(value(&r, "net_investment_income_tax"), 1140);
            let mut session = restart(d);
            let Reply::Edit(request) = from_review(&mut session, TaxSection::MoreCredits) else {
                panic!()
            };
            accepted(&send(&mut session, request));
            let draft = roundtrip_native_draft(session.draft.clone());
            let d = restart(draft).draft;
            let after = supported(&d);
            assert_eq!(after.return_lines, r.return_lines);
            assert_eq!((after.tax, after.refund), (r.tax, r.refund));
            let source = crate::extended_rules::direct_foreign_tax_deduction(&d);
            assert_eq!(source.investment, 100);
            assert_eq!(source.investment_owners, Some([50, 50]));
            let packet = NiitItemizedDeductions::from_native(
                &d,
                NativeDeductionOperands {
                    itemizing: true,
                    portfolio_only_itemized: false,
                    same_itemized_pool: false,
                    whole_itemized_pool: false,
                    whole_itemized_dollars: 0,
                    whole_income_tax_dollars: None,
                    allowed_dollars: 1,
                    allowed_available: true,
                    before_dollars: [0, 0, 0, 1, 0],
                    income_tax_available: true,
                    foreign_cents: source.investment_owners,
                    foreign_capacities: None,
                    estate_ird_cents: [0, 0],
                    current_k1_interest_cents: [0, 0],
                },
            );
            assert_eq!(packet.joint_dollars, 1);
            assert_eq!(
                packet.owners_dollars,
                Some(if injured == Owner::Taxpayer { [1, 0] } else { [0, 1] })
            );
        }
    }
}

#[test]
fn unit13_niit_deduction_known_owner_does_not_require_an_injured_spouse_claim() {
    use crate::injured_spouse::niit_deductions::{NativeDeductionOperands, NiitItemizedDeductions};
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d =
            niit_deduction_fixture(NiitDeductionExpense::Foreign, owner, Owner::Taxpayer, false);
        let ordinary = assert_niit_deduction_money(&d);
        d.injured_spouse = None;
        let plain = supported(&d);
        assert_eq!(plain.return_lines, ordinary.return_lines);
        assert_eq!((plain.tax, plain.refund), (ordinary.tax, ordinary.refund));
        let source = crate::extended_rules::direct_foreign_tax_deduction(&d);
        let packet = NiitItemizedDeductions::from_native(
            &d,
            NativeDeductionOperands {
                itemizing: true,
                portfolio_only_itemized: false,
                same_itemized_pool: false,
                whole_itemized_pool: false,
                whole_itemized_dollars: 0,
                whole_income_tax_dollars: None,
                allowed_dollars: 1000,
                allowed_available: true,
                before_dollars: [0, 0, 0, 1000, 0],
                income_tax_available: true,
                foreign_cents: source.investment_owners,
                foreign_capacities: None,
                estate_ird_cents: [0, 0],
                current_k1_interest_cents: [0, 0],
            },
        );
        assert_eq!(packet.joint_dollars, 1000);
        assert_eq!(
            packet.owners_dollars,
            Some(if owner == Owner::Taxpayer { [1000, 0] } else { [0, 1000] })
        );
    }
}
