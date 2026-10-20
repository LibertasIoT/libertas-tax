use super::*;

// Independent native source/rounding controls for original Form8379 R3/R4.
// Form8379 instructions (November2024), lines13b/14/18, and 2026
// RevProc2025-32 sections3.01/3.39. The2026Form8379 revision stays draft.
// https://www.irs.gov/instructions/i8379
// https://www.irs.gov/pub/irs-drop/rp-25-32.pdf

#[test]
fn unit13_foreign_wages_keep_earned_character_and_actual_owner() {
    let mut d = foreign_income_owner_fixture();
    add_foreign_owner_income(&mut d, Owner::Taxpayer, 30_000);
    add_foreign_owner_income(&mut d, Owner::Spouse, 10_000);
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 310_000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 760);
        let parts = if injured == Owner::Taxpayer { [35_000, 25_000] } else { [25_000, 35_000] };
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], parts);
        let niit = if injured == Owner::Taxpayer { [190, 570] } else { [570, 190] };
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], niit);
        assert_nonallocation_ordinary_unchanged(&d, &r);
    }
    d.income.reverse();
    assert_eq!(value(&supported(&d), "agi"), 310_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unit13_foreign_wages_preserve_native_per_owner_annual_rounding() {
    let mut d = foreign_income_owner_fixture();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(foreign_pay(1000));
        entry.owner = owner;
        let FederalIncome::ForeignEmployment { data } = &mut entry.income else { unreachable!() };
        data.wages += 49;
        add(&mut d, entry);
    }
    // Native Form2555 owner totals round1000.49 each to1000. They must
    // not become one fabricated2000.98-dollar source rounded to2001.
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 272_000);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [6000, 16_000]);
    assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [190, 570]);
    assert_nonallocation_ordinary_unchanged(&d, &r);
}

#[test]
fn unit13_foreign_exclusion_and_housing_follow_native_owner_and_tax_stacking() {
    for housing in [false, true] {
        let mut d = foreign_income_owner_fixture();
        add_foreign_owner_income(&mut d, Owner::Taxpayer, 150_000);
        let mut claim = foreign_claim(Owner::Taxpayer);
        if housing {
            claim.housing = housing_claim(
                0,
                ForeignHousingLocation::General,
                Some(HousingCarryover { amount: 100_000, deducted: 0 }),
            );
        }
        set_claim(&mut d, claim);
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_earned_exclusion"), 132_900);
        assert_eq!(value(&r, "foreign_housing_deduction"), if housing { 1000 } else { 0 });
        assert_eq!(value(&r, "agi"), if housing { 286_100 } else { 287_100 });
        assert_eq!(value(&r, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 760);
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [22_100, 15_000]);
        assert_eq!(
            [column(&r, "14", "b"), column(&r, "14", "c")],
            [if housing { 1000 } else { 0 }, 0]
        );
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [190, 570]);
        // Independent MFS worksheet: taxpayer taxable206000 (205000
        // with housing), plus132900 (133900) stacking =338900; ordinary
        // rates tax that at87384. Subtract24494 (24734). Spouse taxable
        // 48900 uses the48900--48950 tax-table midpoint48925:5623.
        // The Form8379 four-decimal joint-tax ratio uses income taxes;
        // actual190/570 NIIT is already attributed separately on line18.
        let own_income_tax = if housing { 62_650 } else { 62_890 };
        let explanation = &r
            .lines
            .iter()
            .find(|line| line.id == "form_8379_injured_share_of_tax")
            .unwrap()
            .explanation;
        assert!(
            matches!(&explanation.args[..], [_,
            MessageArg::Currency { minor_units: own, .. },
            MessageArg::Currency { minor_units: 5623, .. }
        ] if *own == own_income_tax),
            "actual tax explanation: {:?}",
            explanation.args
        );
        assert_nonallocation_ordinary_unchanged(&d, &r);
    }
}
