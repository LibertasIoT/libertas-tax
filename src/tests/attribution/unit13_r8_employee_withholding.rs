use super::*;
use crate::tests::cross::interview::{accepted, open_topic, rejected_with};
use crate::tests::support::assert::injured_credit::r8_employee_withholding::{
    estimate_if_known, money,
};
use crate::tests::support::builders::injured_credit::r8_employee_withholding::fixture;
use crate::tests::support::client::injured_credit::r8_employee_withholding::{
    employee_id, public_roundtrip,
};

#[test]
fn r8_employee_nevada_nonfederal_sources_preserve_money_then_overlay_and_correction() {
    let mut observed = vec![];
    let mut expected = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = public_roundtrip(fixture(owner, injured, reversed, false), owner);
                let id = employee_id(&d);
                let r = money(&d, owner);
                // All1000 source Dollars are community:426 refund has a213 protected half.
                observed.push((owner, injured, reversed, false, estimate_if_known(&r)));
                expected.push((owner, injured, reversed, false, Some(21300)));
                let mut s = restart(d);
                let mut p = open_topic(&mut s, TaxSection::Wages);
                let P::SaveIncomes { documents, .. } = &mut p else { panic!() };
                documents.documents.iter_mut().find(|r| r.id == id).unwrap().community =
                    CommunityShare::Separate;
                accepted(&send(&mut s, p));
                let d = public_roundtrip(s.draft, owner);
                let r = money(&d, owner);
                // Own separate250 +half remainingcommunity750 gives625/1000;
                // other spouse's separate250 leaves only375/1000 protected.
                observed.push((owner, injured, reversed, true, estimate_if_known(&r)));
                expected.push((
                    owner,
                    injured,
                    reversed,
                    true,
                    Some(if owner == injured { 26625 } else { 15975 }),
                ));
            }
        }
    }
    assert_eq!(
        observed, expected,
        "all16 independent ordinary/publicSave/TaxAppData/restart profiles passed before R8 source assertions"
    );
}

#[test]
fn r8_employee_texas_existing_management_source_is_needed_and_native_save_checked() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let d = fixture(owner, Owner::Taxpayer, false, true);
        money(&d, owner);
        let id = employee_id(&d);
        let mut s = restart(d);
        let mut p = open_topic(&mut s, TaxSection::Wages);
        let P::SaveIncomes { documents, .. } = &mut p else { panic!() };
        let index = documents.documents.iter().position(|r| r.id == id).unwrap();
        assert_eq!(
            documents.documents[index].refund_management_needed,
            Some(true),
            "existing employee earned source has positive actual withholding"
        );
        documents.documents[index].refund_management = None;
        let before = s.draft.clone();
        let dirty = s.dirty;
        rejected_with(send(&mut s, p), "INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED");
        assert_eq!(s.draft, before);
        assert_eq!(s.dirty, dirty);
    }
}

#[test]
fn r8_employee_texas_actual_management_and_owner_sources_survive_restart() {
    let mut observed = vec![];
    let mut expected = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let d = public_roundtrip(fixture(owner, injured, reversed, true), owner);
                let id = employee_id(&d);
                let r = money(&d, owner);
                observed.push((owner, injured, reversed, false, estimate_if_known(&r)));
                expected.push((
                    owner,
                    injured,
                    reversed,
                    false,
                    Some(if owner == injured { 5325 } else { 15975 }),
                ));
                let mut s = restart(d);
                let mut p = open_topic(&mut s, TaxSection::Wages);
                let P::SaveIncomes { documents, .. } = &mut p else { panic!() };
                documents.documents.iter_mut().find(|r| r.id == id).unwrap().refund_management =
                    Some(RefundManagement::Joint);
                accepted(&send(&mut s, p));
                let d = public_roundtrip(s.draft, owner);
                let r = money(&d, owner);
                observed.push((owner, injured, reversed, true, estimate_if_known(&r)));
                expected.push((
                    owner,
                    injured,
                    reversed,
                    true,
                    Some(if owner == injured { 0 } else { 15975 }),
                ));
            }
        }
    }
    assert_eq!(
        observed, expected,
        "all16 independent ordinary/publicSave/TaxAppData/restart profiles passed before Texas management assertions"
    );
}
