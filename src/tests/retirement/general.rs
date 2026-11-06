// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn equivalent_railroad_and_ssa_net_benefits_combine_before_taxation() {
    let mut d = complete(FederalStatus::Single);
    let mut ssa = income(social_security_net(1000000, 10000));
    ssa.label = "".into();
    add(&mut d, ssa);
    let mut rrb = income(social_security_net(-400000, 20000));
    rrb.label = "RRB-1099 equivalent tier 1".into();
    add(&mut d, rrb);
    let r = supported(&d);
    assert_eq!(return_line(&r, "6a"), 600000);
    assert_eq!(return_line(&r, "6b"), 510000);
    assert_eq!(value(&r, "agi"), 65100);
    assert_eq!(r.payments, Some(680000));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn railroad_special_treatments_and_inconsistent_statement_amounts_are_guarded() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, railroad_pension(10000, 12000, None));
    unsupported(&d, "Income:"); // Age facts contradict the ordinary-retirement assertion.
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Ordinary);
    edit_retirement(data).taxable = Some(RetirementTaxable::Railroad {
        repaid: 0,
        repayments: vec![],
        contributory: 1200001,
        box_5: 0,
        box_6: 0,
        holder: Some(crate::RailroadHolder::Employee),
        basis: None,
    });
    assert!(!d.valid_stored());
}

#[test]
fn railroad_final_return_does_not_use_postdeath_birthday() {
    let mut d = complete(FederalStatus::Single);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 19661015;
    p.died = Some(FinalReturn { date: 20260415, ordinary: true, bonds: None, estate_medical: 0 });
    add(&mut d, railroad_pension(10000, 12000, None));
    unsupported(&d, "Income:");
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20261215;
    assert_eq!(return_line(&supported(&d), "5b"), 1200000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn public_safety_rejects_wrong_source_and_overallocated_funding() {
    for mode in [0, 1, 3] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            retirement_entry(
                Owner::Taxpayer,
                if mode == 0 { RetirementKind::Ira } else { RetirementKind::Pension },
                2000,
                RetirementTreatment::Ordinary,
            ),
        );
        let id = d.income.last().unwrap().id;
        public_safety(
            &mut d,
            if mode == 1 { 9999 } else { id },
            3000,
            if mode == 3 { 3000 } else { 2000 },
            PublicSafetyCoverage::Medical,
        );
        unsupported(&d, "Public-safety");
    }
}

#[test]
fn public_safety_repeated_source_cannot_reuse_distribution_funding() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            2000,
            RetirementTreatment::Ordinary,
        ),
    );
    let id = d.income.last().unwrap().id;
    for _ in 0..2 {
        public_safety(&mut d, id, 1500, 1500, PublicSafetyCoverage::Medical);
    }
    unsupported(&d, "Public-safety premium funding");
}
