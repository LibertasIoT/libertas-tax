use super::*;

use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::support::assert::injured_credit::foreign_credit_ordinary_sources::{
    columns, money,
};
use crate::tests::support::builders::injured_credit::foreign_credit_ordinary_sources::Family;
use crate::tests::support::client::injured_credit::foreign_credit_ordinary_sources::{
    known, profiles,
};

#[test]
fn unit13_ordinary_foreign_current_payee_native_credit_owner() {
    known(&[Family::Current]);
}

#[test]
fn unit13_ordinary_prior_income_tax_paid_now_keeps_its_payee() {
    known(&[Family::PriorWage, Family::PriorPassive]);
}

#[test]
fn unit13_ordinary_category_caps_cannot_exchange_physical_owners() {
    known(&[Family::MixedCategories]);
}

#[test]
fn unit13_k3_current_parent_owner_does_not_absorb_released_history() {
    known(&[Family::K3 { released: false }, Family::K3 { released: true }]);
}

#[test]
fn unit13_ordinary_schedule_b_history_without_person_stays_unknown() {
    for (family, _, _, d) in profiles(&[Family::Historical]) {
        let r = money(&d, family);
        assert_eq!(columns(&r), [None, None]);
        assert!(
            r.injured_spouse.as_ref().unwrap().required.is_empty(),
            "unknown historical person is not a joint fact"
        );
        assert!(
            r.review_notes
                .iter()
                .any(|note| note.display.key == "WARN_8379_FOREIGN_CREDIT_OWNER_UNAVAILABLE")
        );
    }
}

#[test]
fn unit13_k3_same_country_parents_keep_bounded_source_use_and_actual_ids() {
    const KEY: &str = "foreign_ordinary_nonrefundable_credit";
    const OLD: &str = "INJURED_COMPONENT_NONREFUNDABLE";
    // All8 literal financial/public source Save/persistence profiles first.
    for (_, _, injured, mut d) in profiles(&[Family::K3Pair]) {
        let sources = d.income.clone();
        let parents: Vec<_> = d
            .income
            .iter()
            .filter_map(|entry| {
                let FederalIncome::Partnership { data } = &entry.income else {
                    return None;
                };
                let separate = data.separate.as_ref().unwrap();
                assert_eq!(
                    (separate.interest, separate.dividends, separate.foreign_taxes),
                    (200000, 1000000, 90000)
                );
                let [row] = data.schedule_k3.as_ref().unwrap().countries.as_slice() else {
                    panic!()
                };
                assert_eq!((row.country.as_str(), row.income, row.tax), ("JA", 600000, 90000));
                Some((entry.id, entry.owner))
            })
            .collect();
        assert_eq!(parents.len(), 2);
        assert!(parents[0].0 > 0 && parents[1].0 > 0 && parents[0].0 != parents[1].0);
        assert_ne!(parents[0].1, parents[1].1);
        let selected = if injured == Owner::Taxpayer { 60000 } else { 82200 };
        d.injured_spouse
            .as_mut()
            .unwrap()
            .allocations
            .push(InjuredAmount { component: OLD.into(), injured: selected });
        let r = money(&d, Family::K3Pair);
        assert_eq!(columns(&r), [None, None]);
        let [row] = r.injured_spouse.as_ref().unwrap().required.as_slice() else { panic!() };
        assert_eq!(
            (row.component.as_str(), row.minimum, row.maximum, row.joint),
            (KEY, 52200, 90000, 142200)
        );
        assert!(r.injured_spouse.as_ref().unwrap().accepted.is_empty());
        let construction = d.injured_spouse.as_ref().unwrap().construction(&d, &r);
        assert!(construction.allocations.is_empty());
        let [row] = construction.new_allocations.as_slice() else { panic!() };
        assert_eq!(row.component, KEY);
        let mut s = restart(d);
        let Reply::Edit(mut old) =
            native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[(KEY, selected)])
        else {
            panic!()
        };
        let P::SaveInjuredSpouse { value, .. } = &mut old else { panic!() };
        let [answer] = value.allocations.as_mut_slice() else { panic!() };
        answer.component = OLD.into();
        assert_eq!(
            crate::injured_spouse::allocation_rule(value, &s.draft, &r),
            Err("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
        );
        let before = s.draft.clone();
        let dirty = s.dirty;
        match send(&mut s, old) {
            Reply::Response(P::BeginInjuredSpouse { value, .. }) => {
                assert!(value.allocations.is_empty());
                let [row] = value.new_allocations.as_slice() else { panic!() };
                assert_eq!((row.component.as_str(), row.minimum, row.maximum), (KEY, 52200, 90000));
            }
            Reply::Response(P::Problem { .. }) => {}
            response => panic!("old aggregate cannot answer actual K3 sources: {response:?}"),
        }
        assert_eq!(s.draft, before);
        assert_eq!(s.draft.revision, before.revision);
        assert_eq!(s.dirty, dirty);
        let Reply::Edit(request) =
            native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[(KEY, selected)])
        else {
            panic!()
        };
        accepted(&send(&mut s, request));
        let r = money(&s.draft, Family::K3Pair);
        assert_eq!(columns(&r), [Some(selected), Some(142200 - selected)]);
        assert_eq!(s.draft.income, sources);
        let draft = roundtrip_native_draft(s.draft.clone());
        let mut s = restart(draft);
        assert_eq!(
            columns(&money(&s.draft, Family::K3Pair)),
            [Some(selected), Some(142200 - selected)]
        );
        let Reply::Edit(request) =
            native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[])
        else {
            panic!()
        };
        accepted(&send(&mut s, request));
        assert_eq!(
            columns(&money(&s.draft, Family::K3Pair)),
            [Some(selected), Some(142200 - selected)]
        );
        assert_eq!(s.draft.income, sources);
    }
}

#[test]
fn unit13_ordinary_2555_reductions_keep_actual_employee_and_business_tax_owners() {
    known(&[Family::EmploymentExcluded, Family::BusinessExcluded]);
}

#[test]
fn unit13_ordinary_htko_moves_current_tax_owner_with_native_category() {
    known(&[Family::HighTax]);
}

#[test]
fn unit13_ordinary_current_accrued_owner_and_prior_paid_year_are_distinct() {
    known(&[Family::CurrentAccrued]);
    for (family, _, _, d) in profiles(&[Family::PriorWage, Family::PriorPassive]) {
        let before = d.clone();
        let mut s = restart(d);
        let Reply::Edit(mut request @ P::SaveMoreCredits { .. }) =
            from_review(&mut s, TaxSection::MoreCredits)
        else {
            panic!()
        };
        let P::SaveMoreCredits { value, .. } = &mut request else { panic!() };
        let MoreCredit::ForeignTax(ForeignTaxPage::Ordinary { basis, .. }) = value
            .credits
            .iter_mut()
            .find(|credit| matches!(credit, MoreCredit::ForeignTax(_)))
            .unwrap()
        else {
            panic!()
        };
        *basis = ForeignTaxBasis::Accrued;
        let mut invalid = before.credits.as_ref().unwrap().foreign_tax.clone();
        let ForeignTaxClaim::Ordinary { basis, .. } = &mut invalid else { panic!() };
        *basis = ForeignTaxBasis::Accrued;
        assert_eq!(
            crate::form_1116::claim_rule(&invalid, &[]).map(|issue| issue.key),
            Some("FOREIGN_PRIOR_TAX_NEEDS_PAID_BASIS")
        );
        let dirty = s.dirty;
        assert!(
            matches!(send(&mut s, request), Reply::Response(P::Problem { .. })),
            "priorincome tax onlycounts when actuallypaid2026"
        );
        assert_eq!(s.draft, before);
        assert_eq!(s.draft.revision, before.revision);
        assert_eq!(s.dirty, dirty);
        money(&s.draft, family);
    }
}
