use super::*;
use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::assert::injured_credit::r8_sale_withholding::money;
use crate::tests::support::builders::injured_credit::r8_sale_withholding::fixture;
use crate::tests::support::client::injured_credit::r8_sale_withholding::persist;

#[test]
fn r8_sale_cash_preserves_native_money_public_save_and_known_property_overlay() {
    let mut observed = vec![];
    let mut expected = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = persist(fixture(owner, injured, reversed), owner);
                let id = d
                    .income
                    .iter()
                    .find(|e| matches!(e.income, FederalIncome::Sale { .. }))
                    .unwrap()
                    .id;
                let r = money(&d, owner);
                observed.push((
                    owner,
                    injured,
                    reversed,
                    false,
                    r.lines
                        .iter()
                        .find(|l| l.id == crate::injured_spouse::ESTIMATE_LINE)
                        .map(|l| l.amount),
                ));
                expected.push((owner, injured, reversed, false, Some(50000)));
                let mut s = restart(d);
                let mut p = open_topic(&mut s, TaxSection::InvestmentSales);
                let P::SaveIncomes { documents, .. } = &mut p else { panic!() };
                documents.documents.iter_mut().find(|r| r.id == id).unwrap().community =
                    CommunityShare::Separate;
                accepted(&send(&mut s, p));
                let d = persist(s.draft, owner);
                assert_eq!(
                    d.income.iter().find(|e| e.id == id).unwrap().community,
                    CommunityShare::Separate
                );
                let r = money(&d, owner);
                observed.push((
                    owner,
                    injured,
                    reversed,
                    true,
                    r.lines
                        .iter()
                        .find(|l| l.id == crate::injured_spouse::ESTIMATE_LINE)
                        .map(|l| l.amount),
                ));
                expected.push((
                    owner,
                    injured,
                    reversed,
                    true,
                    Some(if owner == injured { 62500 } else { 37500 }),
                ));
            }
        }
    }
    assert_eq!(
        observed, expected,
        "all16 independently known ordinary/publicSave/Avro/restart payment profiles passed before R8 source assertions"
    );
}
