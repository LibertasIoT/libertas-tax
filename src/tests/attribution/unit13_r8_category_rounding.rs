use super::*;
use crate::tests::cross::interview::{accepted, from_review, open_document_center};
use crate::tests::support::client::injured_credit::r8_category_rounding::run;

#[test]
fn r8_sale_omission_masks_two_zero_native_withholding_categories() {
    run(true);
}

#[test]
fn r8_included_interest_exposes_flattened_native_category_rounding() {
    run(false);
}

#[test]
fn r8_mixed_native_category_uses_exact_coalesced_legal_weights() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut a = allocation();
            a.injured = injured;
            a.community = InjuredCommunityFacts::Yes {
                states: vec![5],
                debt_facts: InjuredDebtFacts::NotApplicable,
            };
            let mut d = injured_return((30000, 0), (0, 0), 0, a);
            // The empty companion has no funding contribution, but its
            // ordinary editor still requires an admitted community choice.
            for e in &mut d.income {
                e.community = CommunityShare::Community;
            }
            d.income[0].community = CommunityShare::Separate;
            let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
            edit_wage(data).withholding = 100;
            // The prior1aad fixture had withholding on a zero-wage row and
            // native Save correctly refused it. Move one existing wage Dollar
            // to that row: joint wages and every funding oracle stay unchanged.
            edit_wage(data).wages -= 100;
            let mut community = d.income[0].clone();
            community.community = CommunityShare::Community;
            let FederalIncome::Wage { data } = &mut community.income else { panic!() };
            edit_wage(data).wages = 100;
            edit_wage(data).withholding = 49;
            add(&mut d, community);
            let pay = d.payments.as_mut().unwrap();

            pay.estimated = crate::EstimatedPaymentEntryRecord::Sources {
                payments: vec![crate::DatedPayment {
                    date: 20260415,
                    amount: 10000,
                    payer: PaymentPayer::Taxpayer,
                    refund_property: Some(RefundSourcePropertyFacts::Community {
                        management: RefundManagement::NotApplicable,
                    }),
                }],
            };
            if reversed {
                d.income.reverse();
            }
            let money = |d: &FederalDraft| {
                let r = supported(d);
                assert_eq!(value(&r, "agi"), 30000);
                assert_eq!(value(&r, "taxable_income"), 0);
                assert_eq!(value(&r, "regular_income_tax"), 0);
                assert_eq!(return_line(&r, "25a"), 100);
                assert_eq!(return_line(&r, "26"), 10000);
                assert_eq!((r.tax, r.payments, r.refund), (Some(0), Some(10100), Some(10100)));
                let mut plain = d.clone();
                plain.injured_spouse = None;
                assert_eq!(supported(&plain).return_lines, r.return_lines);
                r
            };
            money(&d);
            let old = d.income.clone();
            let mut s = restart(d);
            crate::tests::support::client::admit_native_fixture_facts(&mut s);
            let p = open_document_center(&mut s);
            accepted(&send(&mut s, p));
            let Reply::Edit(p) = from_review(&mut s, TaxSection::Payments) else { panic!() };
            accepted(&send(&mut s, p));
            let Reply::Edit(p) = native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[])
            else {
                panic!()
            };
            accepted(&send(&mut s, p));
            let draft = roundtrip_native_draft(s.draft.clone());
            let mut s = restart(draft);
            let p = open_document_center(&mut s);
            accepted(&send(&mut s, p));
            for entry in &old {
                assert_eq!(s.draft.income.iter().find(|e| e.id == entry.id), Some(entry));
            }
            let r = money(&s.draft);
            assert_eq!(
                share_cents(&r),
                if injured == Owner::Taxpayer { 5084 } else { 5016 },
                "all unchanged ordinary/public Save/Avro/restart figures passed before approved mixed-bin assertion"
            );
        }
    }
}
