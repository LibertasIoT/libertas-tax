use super::*;
use crate::tests::cross::interview::from_review;

fn fixture(answer: Option<i64>) -> FederalDraft {
    let mut claim = allocation();
    if let Some(injured) = answer {
        claim
            .allocations
            .push(InjuredAmount { component: "INJURED_COMPONENT_ITEMIZED".into(), injured });
    }
    let mut d = injured_return((200_000, 60_000), (50_000, 10_000), 0, claim);
    for (owner, amount) in [(Owner::Taxpayer, 20_000), (Owner::Spouse, 10_000)] {
        let mut entry = income(savings_income(amount, 0));
        entry.owner = owner;
        add(&mut d, entry);
    }
    let deductions = d.deductions.as_mut().unwrap();
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    itemized(deductions).investment_property_taxes = 100_000;
    d
}

#[test]
fn unit13_shared_amount_construction_transports_only_real_bounded_answers() {
    for answer in [None, Some(40_000), Some(100_001)] {
        let d = fixture(answer);
        let ordinary = supported(&d);
        assert_eq!(value(&ordinary, "agi"), 280_000);
        assert_eq!(value(&ordinary, "taxable_income"), 279_000);
        assert_eq!(value(&ordinary, "net_investment_income_tax"), 1102);
        assert_eq!(ordinary.tax, Some(5_325_800));
        assert_eq!(ordinary.refund, Some(1_674_200));
        let before = d.clone();
        let mut session = restart(d);
        let Reply::Response(response @ P::BeginInjuredSpouse { .. }) =
            from_review(&mut session, TaxSection::InjuredSpouse)
        else {
            panic!("expected normal WorkflowRequest construction context")
        };
        let bytes = response.to_avro();
        let decoded = P::from_avro(&bytes).unwrap();
        assert_eq!(decoded.to_avro(), bytes);
        let P::BeginInjuredSpouse { value, .. } = decoded else { unreachable!() };
        let component = "INJURED_COMPONENT_ITEMIZED";
        let answered = value.allocations.iter().find(|row| row.component == component);
        let new = value.new_allocations.iter().find(|row| row.component == component);
        assert_eq!(usize::from(answered.is_some()) + usize::from(new.is_some()), 1);
        if let Some(real_answer) = answer.filter(|value| (0..=100_000).contains(value)) {
            let row = answered.expect("bounded accepted answer reopens as a complete row");
            assert_eq!((row.joint, row.minimum, row.maximum), (100_000, 0, 100_000));
            assert_eq!(row.injured, real_answer);
            assert!(new.is_none());
        } else {
            let row = new.expect("missing or out-of-bounds answer has context only");
            assert_eq!((row.joint, row.minimum, row.maximum), (100_000, 0, 100_000));
            assert!(answered.is_none());
        }
        let components: Vec<_> = value
            .allocations
            .iter()
            .map(|row| &row.component)
            .chain(value.new_allocations.iter().map(|row| &row.component))
            .collect();
        assert_eq!(components.len(), ordinary.injured_spouse.as_ref().unwrap().required.len());
        for (index, component) in components.iter().enumerate() {
            assert!(!components[..index].contains(component), "each component asks once");
        }
        assert_eq!(session.draft, before);
        assert!(!session.dirty);
    }
}

#[test]
fn unit13_shared_amount_two_arrays_validate_one_stored_vector() {
    let d = fixture(Some(40_000));
    let r = supported(&d);
    let claim = d.injured_spouse.as_ref().unwrap();
    let mut page = claim.page(&d, &r);
    let index = page
        .allocations
        .iter()
        .position(|row| row.component == "INJURED_COMPONENT_ITEMIZED")
        .unwrap();
    let answered = page.allocations.remove(index);
    assert_eq!(answered.injured, 40_000);
    page.new_allocations.push(answered.clone());
    let merged = InjuredSpouseAllocation::from_page(&page);
    assert_eq!(merged.allocations.len(), r.injured_spouse.as_ref().unwrap().required.len());
    assert_eq!(
        merged.allocations.iter().filter(|row| row.component == answered.component).count(),
        1
    );
    assert_eq!(
        merged.allocations.iter().find(|row| row.component == answered.component).unwrap().injured,
        40_000
    );
    assert!(crate::injured_spouse::allocation_rule(&page, &d, &r).is_ok());
    // Echoed context never authorizes an amount outside the native bounds.
    page.new_allocations[0].injured = 100_001;
    assert_eq!(
        crate::injured_spouse::allocation_rule(&page, &d, &r),
        Err("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
    );
    page.new_allocations[0].injured = 40_000;
    // Duplicating an otherwise valid answer across the two arrays is rejected.
    page.allocations.push(answered);
    assert_eq!(
        crate::injured_spouse::allocation_rule(&page, &d, &r),
        Err("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT")
    );
}
