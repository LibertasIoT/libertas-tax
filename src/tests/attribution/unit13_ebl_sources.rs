use super::*;
use crate::injured_spouse::income_categories::{
    Native461IncomeGroup as Group, Native461SourceIssue as Issue, Native461Sources,
};

#[test]
fn unit13_ebl_sources_owner_net_bounds_are_not_gross_activity_loss() {
    // Dispatcher07's bounded-allocation arithmetic, NOT a2026 return:
    // native461's year threshold is not replaced by this component target.
    let source = Native461Sources {
        joint_addback_dollars: 500,
        eligible_joint_dollars: -1100,
        groups: Ok(vec![
            Group::Dollars { joint_dollars: -1000, owners: [-1000, 0] },
            Group::Dollars { joint_dollars: 900, owners: [900, 0] },
            Group::Dollars { joint_dollars: -1000, owners: [0, -1000] },
        ]),
    };
    assert_eq!(source.owner_net(0), Ok([-100, -1000]));
    assert_eq!(source.bounds(0), Ok([0, 100]));
    assert_eq!(source.bounds(1), Ok([400, 500]));
}

#[test]
fn unit13_ebl_sources_units_footing_unknown_capital_and_independent_zero() {
    let mut source = Native461Sources {
        joint_addback_dollars: 500,
        eligible_joint_dollars: -1100,
        groups: Ok(vec![Group::Cents { joint_dollars: -1100, owners: [-10_049, -99_951] }]),
    };
    assert_eq!(source.owner_net(0), Ok([-100, -1000]));
    assert_eq!(source.bounds(0), Ok([0, 100]));
    source.eligible_joint_dollars = -1099;
    assert_eq!(source.bounds(0), Err(Issue::IncompleteSources));
    source.groups = Err(Issue::CapitalOwnerUnavailable);
    assert_eq!(source.bounds(0), Err(Issue::CapitalOwnerUnavailable));
    // A known zero addback is independent of an unavailable positive capital
    // allowance source; it does not manufacture that source's owner.
    source.joint_addback_dollars = 0;
    assert_eq!(source.bounds(0), Ok([0, 0]));
    assert_eq!(source.owner_net(0), Err(Issue::CapitalOwnerUnavailable));
}

#[test]
fn unit13_ebl_sources_pure_owner_native_addback_public_lifecycle() {
    assert_eq!(crate::year_constants::EXCESS_BUSINESS_LOSS_THRESHOLD_JOINT_DOLLARS, 512_000);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
            d.injured_spouse.as_mut().unwrap().injured = injured;
            add_ebl_business(&mut d, owner, -600_000);
            let result = assert_ebl_money(&d, 88_000);
            assert!(
                result
                    .injured_spouse
                    .as_ref()
                    .unwrap()
                    .required
                    .iter()
                    .all(|row| row.component != "excess_business_loss")
            );
            let saved = persist_ebl_sources(d);
            let result = assert_ebl_money(&saved, 88_000);
            let i = usize::from(injured == Owner::Spouse);
            let mut expected = [0; 2];
            expected[usize::from(owner == Owner::Spouse)] = -512_000;
            assert_eq!(column(&result, "13a", "a"), 80_000);
            assert_eq!(column(&result, "13b", "a"), -512_000);
            assert_eq!(column(&result, "13b", "b"), expected[i]);
            assert_eq!(column(&result, "13b", "c"), expected[1 - i]);
        }
    }
}

#[test]
fn unit13_ebl_sources_native_scaled_owner_net_choice_public_lifecycle() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reversed in [false, true] {
            let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
            d.injured_spouse.as_mut().unwrap().injured = injured;
            add_ebl_business(&mut d, Owner::Taxpayer, -1000);
            add_ebl_business(&mut d, Owner::Taxpayer, 900);
            add_ebl_business(&mut d, Owner::Spouse, -512_400);
            if reversed {
                d.income.reverse();
            }
            let initial = assert_ebl_money(&d, 500);
            let row = initial
                .injured_spouse
                .as_ref()
                .unwrap()
                .required
                .iter()
                .find(|row| row.component == "excess_business_loss")
                .unwrap();
            let bounds = if injured == Owner::Taxpayer { (0, 10_000) } else { (40_000, 50_000) };
            assert_eq!((row.joint, row.minimum, row.maximum), (50_000, bounds.0, bounds.1));
            assert!(initial.lines.iter().all(|row| row.id != "form_8379_injured_refund_estimate"));
            assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
            d.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
                component: "excess_business_loss".into(),
                injured: if injured == Owner::Taxpayer { 5000 } else { 45_000 },
            });
            let saved = persist_ebl_sources(d);
            let result = assert_ebl_money(&saved, 500);
            let expected = [-50, -511_950];
            let i = usize::from(injured == Owner::Spouse);
            assert_eq!(column(&result, "13b", "a"), -512_000);
            assert_eq!(column(&result, "13b", "b"), expected[i]);
            assert_eq!(column(&result, "13b", "c"), expected[1 - i]);
        }
    }
}

#[test]
fn unit13_ebl_sources_final_pass_through_ordinary_map_foots_native_owner() {
    let mut d = injured_return((60_000, 5000), (20_000, 500), 0, allocation());
    let mut p = income(FederalIncome::Partnership { data: ordinary_partner_source(20_000, true) });
    p.owner = crate::DocumentOwner::Spouses;
    add(&mut d, p);
    add(
        &mut d,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(10_000, true)) }),
    );
    let mut result = supported(&d);
    let rental = crate::extended_rules::rentals(&d, 80_000, &mut result);
    assert!(rental.inputs_complete);
    assert_eq!(
        rental.ordinary_business_by_id.values().sum::<i64>(),
        rental.ordinary_pass_through_business()
    );
    assert_eq!(rental.ordinary_business_by_id.len(), 2);
    assert_eq!(rental.ordinary_business_by_id[&3], 20_000);
    assert_eq!(rental.ordinary_business_by_id[&4], 10_000);
}
