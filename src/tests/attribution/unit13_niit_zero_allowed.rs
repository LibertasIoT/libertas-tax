use super::*;
use crate::tests::cross::interview::from_review;

#[test]
fn unit13_niit_zero_allowed_has_unique_zero_owners_despite_unknown_portfolio_payer() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let d = niit_zero_fixture(49, injured, reversed);
            assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
            assert_niit_zero_money(&d, 0);
            let d = persist_niit_zero(d, 0);
            assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
            let r = assert_niit_zero_money(&d, 0);
            let owners = [760, 380];
            assert_eq!(column(&r, "18", "a"), 1140);
            assert_eq!(column(&r, "18", "b"), owners[usize::from(injured == Owner::Spouse)]);
            assert_eq!(column(&r, "18", "c"), owners[usize::from(injured != Owner::Spouse)]);
        }
    }
}

#[test]
fn unit13_niit_zero_allowed_positive_dollar_keeps_unknown_payer_unavailable() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let d = niit_zero_fixture(50, injured, reversed);
            assert_niit_zero_money(&d, 1);
            // 2026-10-14 decision59e9ce8a: the independently unchanged
            // positive1-Dollar native amount now requires its actual payer
            // on public Save. The former accepted-unknown witness is history.
            let mut session = restart(d);
            let Reply::Edit(request) = from_review(&mut session, TaxSection::OtherItemized) else {
                panic!()
            };
            let before = session.draft.clone();
            assert!(matches!(send(&mut session, request), Reply::Response(P::Problem { .. })));
            assert_eq!(session.draft, before);
            let r = assert_niit_zero_money(&session.draft, 1);
            assert!(r.lines.iter().all(|line| !matches!(
                line.id.as_str(),
                "form_8379_line_18_b" | "form_8379_line_18_c"
            )));
        }
    }
}

#[test]
fn unit13_niit_zero_allowed_no_claim_keeps_known_zero_packet() {
    use crate::injured_spouse::niit_deductions::{NativeDeductionOperands, NiitItemizedDeductions};
    for paid in [49, 50] {
        let mut d = niit_zero_fixture(paid, Owner::Taxpayer, false);
        d.injured_spouse = None;
        let allowed = i64::from(paid == 50);
        assert_niit_zero_money(&d, allowed);
        let packet = NiitItemizedDeductions::from_native(
            &d,
            NativeDeductionOperands {
                itemizing: true,
                portfolio_only_itemized: false,
                same_itemized_pool: false,
                whole_itemized_pool: false,
                whole_itemized_dollars: 0,
                whole_income_tax_dollars: None,
                allowed_dollars: allowed,
                allowed_available: true,
                before_dollars: [0, allowed, 0, 0, 0],
                income_tax_available: true,
                foreign_cents: Some([0, 0]),
                foreign_capacities: None,
                estate_ird_cents: [0, 0],
                current_k1_interest_cents: [0, 0],
            },
        );
        assert_eq!(packet.joint_dollars, allowed);
        assert_eq!(packet.owners_dollars, if allowed == 0 { Some([0, 0]) } else { None });
    }
}
