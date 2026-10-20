// Genuine native Hobby income is a positive Form8960 source, not zero Rest.
use super::*;
use crate::tests::cross::interview::{accepted, open_topic};

#[test]
fn hobby_native_money_public_save_restart_before_owner_assertions() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                persist_hobby_niit_sources(hobby_niit_return(owner, injured, reversed));
            }
        }
    }
}
#[test]
fn hobby_niit_actual_owner_and_correction_after_public_save_restart() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = persist_hobby_niit_sources(hobby_niit_return(owner, injured, reversed));
                assert_hobby_owner_columns(&d, owner);
                let source = d
                    .income
                    .iter()
                    .find(|e| matches!(e.income, FederalIncome::HobbySales { .. }))
                    .unwrap()
                    .id;
                assert!(source > 0);
                let other = if owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
                let mut s = restart(d);
                let mut q = open_topic(&mut s, TaxSection::OtherIncome);
                let P::SaveIncomes { documents, .. } = &mut q else { panic!() };
                documents.documents.iter_mut().find(|e| e.id == source).unwrap().owner = other;
                accepted(&send(&mut s, q));
                let d = persist_hobby_niit_sources(s.draft);
                assert_eq!(d.income.iter().find(|e| e.id == source).unwrap().owner, other);
                assert_hobby_owner_columns(&d, other);
            }
        }
    }
}

#[test]
fn hobby_goods_niit_target_excludes_not_for_profit_compensation_in_same_printed_row() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = hobby_niit_return(owner, Owner::Taxpayer, false);
        let mut e = income(FederalIncome::Nec1099 {
            data: FederalForm1099Nec {
                compensation: 500000,
                withholding: 0,
                golden_parachute: 0,
                belongs: IncomeUse::NotForProfit,
                business: None,
            },
        });
        e.owner = if owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
        add(&mut d, e);
        let mut session = restart(d);
        let q = open_topic(&mut session, TaxSection::OtherIncome);
        accepted(&send(&mut session, q));
        let r = supported(&session.draft);
        assert_eq!(value(&r, "hobby_sales_income"), 9000);
        assert_eq!(value(&r, "agi"), 279000);
        assert_eq!(value(&r, "taxable_income"), 246800);
        assert_eq!(value(&r, "regular_income_tax"), 44428);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 24000);
        assert_eq!(value(&r, "net_investment_income_tax"), 912);
        assert_eq!(r.tax, Some(4534000));
        assert_eq!(r.refund, Some(2466000));
        assert_eq!(
            [column(&r, "18", "b"), column(&r, "18", "c")],
            if owner == Owner::Taxpayer { [532, 380] } else { [380, 532] }
        );
    }
}
#[test]
fn hobby_fractional_sources_retain_native_joint_rounding_and_niit_footing() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = hobby_niit_return(Owner::Taxpayer, injured, false);
        d.income.retain(|e| !matches!(e.income, FederalIncome::HobbySales { .. }));
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut e = income(hobby_sales(0, 0, 0));
            let FederalIncome::HobbySales { data } = &mut e.income else { panic!() };
            data.receipts = 49;
            e.owner = owner;
            add(&mut d, e);
        }
        let mut s = restart(d);
        let q = open_topic(&mut s, TaxSection::OtherIncome);
        accepted(&send(&mut s, q));
        let draft = roundtrip_native_draft(s.draft);
        let mut s = restart(draft);
        let q = open_topic(&mut s, TaxSection::OtherIncome);
        accepted(&send(&mut s, q));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "hobby_sales_income"), 1);
        assert_eq!(value(&r, "agi"), 270001);
        assert_eq!(value(&r, "taxable_income"), 237801);
        assert_eq!(value(&r, "regular_income_tax"), 42268);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 20001);
        assert_eq!(value(&r, "net_investment_income_tax"), 760);
        assert_eq!(r.tax, Some(4302800));
        assert_eq!(r.refund, Some(2697200));
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [10000, 10001]);
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [380, 380]);
    }
}
