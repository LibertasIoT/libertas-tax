use super::*;
use crate::tests::cross::interview::{accepted, open_topic};

#[test]
fn unit13_current_nua_preserves_native_money_and_accepted_source_through_restart() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = persist_current_nua_sources(current_nua_return(owner, injured, reversed));
                assert_current_nua_money(&d);
                assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
            }
        }
    }
}

#[test]
fn unit13_current_nua_exclusion_follows_actual_sale_owner_only_on_niit_axis() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = persist_current_nua_sources(current_nua_return(owner, injured, reversed));
                let r = assert_current_nua_money(&d);
                let owner_index = usize::from(owner == Owner::Spouse);
                let injured_index = usize::from(injured == Owner::Spouse);
                let mut other_income = [10_000, 10_000];
                other_income[owner_index] += 95_000;
                assert_eq!(column(&r, "13b", "a"), 115_000);
                assert_eq!(column(&r, "13b", "b"), other_income[injured_index]);
                assert_eq!(column(&r, "13b", "c"), other_income[1 - injured_index]);
                let mut niit = [380, 380];
                niit[owner_index] = 1140;
                assert_eq!(column(&r, "18", "a"), 1520);
                assert_eq!(column(&r, "18", "b"), niit[injured_index]);
                assert_eq!(column(&r, "18", "c"), niit[1 - injured_index]);
            }
        }
    }
}

#[test]
fn unit13_current_nua_combines_two_actual_sale_owners_without_a_loss_use_answer() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut d = current_nua_return(Owner::Taxpayer, injured, reversed);
            let mut second = d
                .income
                .iter()
                .find(|entry| matches!(entry.income, FederalIncome::NuaSale { .. }))
                .unwrap()
                .clone();
            second.owner = crate::DocumentOwner::Spouses;
            add(&mut d, second);
            let check = |draft: &FederalDraft| {
                let r = supported(draft);
                assert_eq!(value(&r, "capital_gain"), 190_000);
                assert_eq!(value(&r, "agi"), 460_000);
                assert_eq!(value(&r, "taxable_income"), 427_800);
                assert_eq!(value(&r, "regular_income_tax"), 70_768);
                assert_eq!(value(&r, "niit_disposition_adjustment"), -150_000);
                assert_eq!(value(&r, "form_8960_net_investment_income"), 60_000);
                assert_eq!(value(&r, "net_investment_income_tax"), 2280);
                assert_eq!(r.tax, Some(7_304_800));
                assert_eq!(r.balance, Some(304_800));
                for column_name in ["b", "c"] {
                    assert_eq!(column(&r, "13b", column_name), 105_000);
                    assert_eq!(column(&r, "18", column_name), 1140);
                }
                assert!(draft.injured_spouse.as_ref().unwrap().allocations.is_empty());
                r
            };
            let before = check(&d);
            let mut session = restart(d);
            let request = open_topic(&mut session, TaxSection::InvestmentSales);
            accepted(&send(&mut session, request));
            let draft = roundtrip_native_draft(session.draft);
            let mut session = restart(draft);
            let request = open_topic(&mut session, TaxSection::InvestmentSales);
            accepted(&send(&mut session, request));
            let after = check(&session.draft);
            assert_eq!(before.return_lines, after.return_lines);
        }
    }
}
