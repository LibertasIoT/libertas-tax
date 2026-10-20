// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn elderly_credit_schedule_r_boundaries_and_joint_age_count() {
    use crate::extended_rules::elderly_credit;
    for (status, apart, seniors, agi, exempt, expected) in [
        (FederalStatus::Single, false, 1, 7500, 0, 750),
        (FederalStatus::Single, false, 1, 17500, 0, 0),
        (FederalStatus::Single, false, 1, 10000, 2000, 263),
        (FederalStatus::Joint, false, 2, 10000, 0, 1125),
        (FederalStatus::Joint, false, 1, 10000, 0, 750),
        (FederalStatus::Joint, false, 2, 25000, 0, 0),
        (FederalStatus::Separate, true, 1, 5000, 0, 563),
        (FederalStatus::Separate, false, 1, 5000, 0, 0),
        (FederalStatus::Single, false, 0, 7500, 0, 0),
    ] {
        assert_eq!(elderly_credit(status, apart, seniors, [0; 2], agi, exempt), expected);
    }
}

#[test]
fn dc_homebuyer_credit_follows_child_credit_and_defers_form_1062_combination() {
    let mut d = complete(FederalStatus::Single);
    d.dependents.push(child(2));
    d.next_id = 3;
    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(400_000);
    let r = supported(&d);
    assert_eq!(return_line(&r, "19"), 102_300);
    assert_eq!(value(&r, "dc_homebuyer_credit"), 4000);
    assert_eq!(value(&r, "dc_homebuyer_carryforward_2027"), 0);
    assert_eq!(return_line(&r, "21"), 502_300);
    assert_eq!(return_line(&r, "28"), 117_700);

    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(600_000);
    let capped = supported(&d);
    assert_eq!(value(&capped, "dc_homebuyer_credit"), 4523);
    assert_eq!(value(&capped, "dc_homebuyer_carryforward_2027"), 1477);
    assert_eq!(return_line(&capped, "19"), 50_000);
    assert_eq!(return_line(&capped, "28"), 170_000);

    let mut farmland = farmland_installment_return();
    farmland.credits.as_mut().unwrap().dc_homebuyer_carry = Some(100_000);
    unsupported(&farmland, "Farmland tax installments");
}

#[test]
fn clean_vehicle_credit_precedes_mortgage_and_child_credits() {
    let mut d = complete(FederalStatus::Single);
    d.dependents.push(child(2));
    d.next_id = 3;
    d.credits.as_mut().unwrap().clean_vehicles =
        Some(clean_vehicle_claim(vec![new_clean_vehicle()]));
    d.credits.as_mut().unwrap().mortgage_credit = Some(FederalMortgageCredit {
        current: None,
        carry_three_years_prior: 200_000,
        carry_two_years_prior: 0,
        carry_prior_year: 0,
        loan: None,
    });
    let result = supported(&d);
    assert_eq!(value(&result, "new_clean_vehicle_credit"), 5023);
    assert_eq!(value(&result, "mortgage_interest_credit"), 0);
    assert_eq!(return_line(&result, "19"), 0);
    assert_eq!(return_line(&result, "28"), 170_000);
}

#[test]
fn legacy_ev_credit_follows_child_but_precedes_refueling_and_minimum_tax_credit() {
    let mut d = complete(FederalStatus::Single);
    d.dependents.push(child(2));
    d.next_id = 3;
    d.credits.as_mut().unwrap().legacy_ev_passive_allowed = Some(600_000);
    let child_result = supported(&d);
    assert_eq!(return_line(&child_result, "19"), 220_000);
    assert_eq!(value(&child_result, "legacy_ev_passive_credit"), 2823);

    d.dependents.clear();
    d.credits.as_mut().unwrap().personal_refueling = vec![personal_refueling(400_000, 20260630)];
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let following = supported(&d);
    assert_eq!(value(&following, "legacy_ev_passive_credit"), 5023);
    assert_eq!(value(&following, "personal_refueling_credit"), 0);
    assert_eq!(value(&following, "minimum_tax_credit"), 0);

    d.credits.as_mut().unwrap().clean_vehicles =
        Some(clean_vehicle_claim(vec![new_clean_vehicle()]));
    unsupported(&d, "Legacy EV and clean vehicle credits");
}

#[test]
fn tax_credit_bond_interest_stays_the_filers_own_on_a_community_property_return() {
    let bond = direct_tax_credit_bond(
        "COMMUNITY1",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        100_000,
    );
    for mut draft in [
        community_w2_draft(6_000_000, 10_000_000, 0, 0),
        community_interest_draft(6_000_000, 10_000_000, 200_000, Owner::Taxpayer),
    ] {
        draft.credits.as_mut().unwrap().tax_credit_bonds = vec![bond.clone()];
        // The bond interest is the filer's own, computed on the return.
        supported(&draft);
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}
