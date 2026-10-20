use super::*;

#[test]
fn unit19_schedule_one_business_forms_keep_actual_signed_source_totals() {
    let forms = |r: &FederalResult| {
        ["schedule_1_business_income", "schedule_1_farm_income"]
            .map(|id| r.lines.iter().find(|row| row.id == id).map_or(0, |row| row.amount / 100))
    };
    for (profit, expected) in [
        (20000, [20000, 50000]),
        (-10000, [-10000, 50000]),
        (-50000, [-50000, 50000]),
        (-60000, [-60000, 50000]),
    ] {
        let mut d = routed_farm();
        add(&mut d, income(cash_business(profit)));
        let r = supported(&d);
        assert_eq!(forms(&r), expected);
        assert_eq!(forms(&supported(&restart(d.clone()).draft)), expected);
        d.income.reverse();
        let reordered = supported(&d);
        assert_eq!(forms(&reordered), expected);
        assert_eq!(value(&reordered, "agi"), value(&r, "agi"));
        assert_eq!(reordered.tax, r.tax);
    }
    // Both source groups have49c beyond whole dollars. Keep the native
    // annual70001 and its existing C-first rounding; F contains an actual
    // source and takes the one-dollar annual rounding remainder.
    let mut d = routed_farm();
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.expenses[0].amount -= 49;
    let mut nonfarm = cash_business(20000);
    let FederalIncome::Business { data } = &mut nonfarm else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts += 49;
    add(&mut d, income(nonfarm));
    let r = supported(&d);
    assert_eq!(forms(&r), [20000, 50001]);
    assert_eq!(value(&r, "additional_income"), 70001);
    assert_eq!(value(&r, "agi"), 65055);
}

#[test]
fn unit19_schedule_one_business_forms_preserve_community_and_statutory_sources() {
    let mut d = empty_return(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Arizona, rule: CommunityRule::Halves };
    let mut nonfarm = income(cash_business(20000));
    nonfarm.community = CommunityShare::Community;
    add(&mut d, nonfarm);
    let mut farm = income(raised_farm(40000, 10000));
    farm.owner = Owner::Spouse;
    farm.community = CommunityShare::Community;
    add(&mut d, farm);
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_business_income"), 10000);
    assert_eq!(value(&r, "schedule_1_farm_income"), 15000);
    assert_eq!(value(&r, "additional_income"), 25000);
    assert_eq!(supported(&restart(d).draft), r);

    let mut d = complete(FederalStatus::Single);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.statutory_employee = true;
    add(&mut d, income(cash_business(10000)));
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_business_income"), 70000);
    assert!(!r.lines.iter().any(|row| row.id == "schedule_1_farm_income"));
    assert_eq!(value(&r, "agi"), 69293);
    assert_eq!(value(&r, "self_employment_tax"), 1413);
}

fn routed_farm() -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 10000)));
    add(
        &mut d,
        income(FederalIncome::Cooperative {
            data: FederalCooperative { patronage: 2_000_000, ..Default::default() },
        }),
    );
    d
}
