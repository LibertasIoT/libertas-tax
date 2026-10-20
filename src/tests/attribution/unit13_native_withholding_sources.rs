// Existing native producer controls: cash Cents and reported Dollars stay separate.
use super::*;

#[test]
fn unit13_native_withholding_separate_zero_categories_retain_actual_source_cents() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            for sale in [false, true] {
                let mut d = native_cash_fixture();
                let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
                data.withholding = 49;
                let mut e = if sale {
                    let mut e = income(ordinary_investment_sale(10_000, 10_000, GainTerm::Long));
                    let FederalIncome::Sale { data } = &mut e.income else { panic!() };
                    data.withholding = 49;
                    e
                } else {
                    let mut e = income(savings_income(0, 0));
                    let FederalIncome::Interest { data } = &mut e.income else { panic!() };
                    data.taxable = 49;
                    data.withholding = 49;
                    e
                };
                e.owner = owner;
                let id = d.next_id;
                add(&mut d, e);
                if reverse {
                    d.income.reverse();
                }
                let r = save_native_cash_sources(
                    d,
                    if sale { TaxSection::InvestmentSales } else { TaxSection::InterestDividends },
                    [0; 3],
                    30_000,
                );
                assert_native_cash_row(&r, 1, Owner::Taxpayer, [49, 0, 0]);
                assert_native_cash_row(&r, id, owner, [0, 49, 0]);
                let packet = r.withholding_sources.unwrap();
                assert_eq!(packet.joint_dollars, [0; 3]);
                assert_eq!(packet.rows.len(), 2);
            }
        }
    }
}

#[test]
fn unit13_native_withholding_qsbs_preloop_sources_count_once() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut d = native_cash_fixture();
            let mut expected = Vec::new();
            for (index, held) in [149, 1, 50].into_iter().enumerate() {
                let mut e = qsb_income(
                    if index < 2 {
                        "Synthetic excluded issuer"
                    } else {
                        "Synthetic unqualified issuer"
                    },
                    1_000_000,
                    if index < 2 { 2_000_000 } else { 1_000_001 },
                    0,
                );
                e.owner = if index == 1 {
                    if owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer }
                } else {
                    owner
                };
                let FederalIncome::QualifiedSmallBusinessSale { data } = &mut e.income else {
                    panic!()
                };
                data.withholding = held;
                if index == 2 {
                    data.issuer_qualified = false;
                }
                expected.push((d.next_id, e.owner, held));
                add(&mut d, e);
            }
            if reverse {
                d.income.reverse();
            }
            let r = save_native_cash_sources(d, TaxSection::InvestmentSales, [0, 2, 0], 30_000);
            for (id, owner, held) in expected {
                assert_native_cash_row(&r, id, owner, [0, held, 0]);
            }
            let packet = r.withholding_sources.unwrap();
            assert_eq!(packet.joint_dollars, [0, 2, 0]);
            assert_eq!(packet.rows.len(), 3);
        }
    }
}

#[test]
fn unit13_native_withholding_inherited_group_counts_linked_statements_once() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut d = native_cash_fixture();
            let id = d.next_id;
            let mut group = inherited_ira(3_000, 0);
            group.owner = owner;
            add(&mut d, group);
            let mut linked = Vec::new();
            for (gross, held) in [(1_000, 49), (2_000, 1)] {
                let mut payment = inherited_payment(owner, Some(id), gross);
                let FederalIncome::Retirement { data } = &mut payment.income else { panic!() };
                data.withholding = held;
                linked.push(d.next_id);
                add(&mut d, payment);
            }
            if reverse {
                d.income.reverse();
            }
            let r = save_native_cash_sources(d, TaxSection::Retirement, [0, 1, 0], 30_000);
            assert_eq!(return_line(&r, "4a"), 300_000);
            assert_eq!(return_line(&r, "4b"), 0);
            assert_native_cash_row(&r, id, owner, [0, 50, 0]);
            let packet = r.withholding_sources.unwrap();
            assert_eq!(packet.joint_dollars, [0, 1, 0]);
            assert_eq!(packet.rows.len(), 1);
            assert_eq!(packet.inherited_payment_ids.get(&id), Some(&linked));
            assert!(packet.rows.iter().all(|r| !linked.contains(&r.id)));
        }
    }
}

#[test]
fn unit13_native_withholding_retains_after_pending_community_share_and_final_flush() {
    for reverse in [false, true] {
        let mut d = community_w2_draft(2_000_000, 4_000_000, 10_000, 30_000);
        d.payments.as_mut().unwrap().estimated = Some(0);
        if reverse {
            d.income.reverse();
        }
        let r = save_native_cash_sources(d, TaxSection::Wages, [200, 0, 0], 30_000);
        assert_native_cash_row(&r, 1, Owner::Taxpayer, [5_000, 0, 0]);
        assert_native_cash_row(&r, 2, Owner::Spouse, [15_000, 0, 0]);
        let packet = r.withholding_sources.unwrap();
        assert_eq!(packet.joint_dollars, [200, 0, 0]);
        assert_eq!(packet.rows.len(), 2);
    }
}

#[test]
fn unit13_native_withholding_closed_zero_is_distinct_from_unavailable() {
    assert!(FederalResult::empty(0).withholding_sources.is_none());
    let r = save_native_cash_sources(native_cash_fixture(), TaxSection::Wages, [0; 3], 30_000);
    let packet = r.withholding_sources.unwrap();
    assert!(packet.complete);
    assert_eq!(packet.joint_dollars, [0; 3]);
    assert!(packet.rows.is_empty());
    assert!(packet.inherited_payment_ids.is_empty());
    assert!(
        !crate::injured_spouse::income_categories::NativeWithholdingSources::default().complete
    );
}
