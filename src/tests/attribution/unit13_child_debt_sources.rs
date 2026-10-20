use super::*;

#[test]
fn unit13_child_debt_sources_child_all_characters_taxpayer() {
    assert_child_election_owner_source(Owner::Taxpayer, true);
}
#[test]
fn unit13_child_debt_sources_child_all_characters_spouse() {
    assert_child_election_owner_source(Owner::Spouse, true);
}
#[test]
fn unit13_child_debt_sources_child_tax_only_taxpayer() {
    assert_child_election_owner_source(Owner::Taxpayer, false);
}
#[test]
fn unit13_child_debt_sources_child_tax_only_spouse() {
    assert_child_election_owner_source(Owner::Spouse, false);
}

#[test]
fn unit13_child_debt_sources_taxable_cancellation_taxpayer() {
    assert_debt_owner_source(Owner::Taxpayer, false, false);
}
#[test]
fn unit13_child_debt_sources_taxable_cancellation_spouse() {
    assert_debt_owner_source(Owner::Spouse, false, false);
}
#[test]
fn unit13_child_debt_sources_cancellation_and_transfer_taxpayer() {
    assert_debt_owner_source(Owner::Taxpayer, true, false);
}
#[test]
fn unit13_child_debt_sources_cancellation_and_transfer_spouse() {
    assert_debt_owner_source(Owner::Spouse, true, false);
}
#[test]
fn unit13_child_debt_sources_known_zero_taxpayer() {
    assert_debt_owner_source(Owner::Taxpayer, false, true);
}
#[test]
fn unit13_child_debt_sources_known_zero_spouse() {
    assert_debt_owner_source(Owner::Spouse, false, true);
}

#[test]
fn unit13_child_debt_sources_child_niit_keeps_native_parent_character() {
    let mut d = child_election_owner_fixture(Owner::Spouse, true);
    for entry in &mut d.income {
        if let FederalIncome::Wage { data } = &mut entry.income {
            data.wages = if entry.owner == Owner::Taxpayer { 20_000_000 } else { 5_000_000 };
        }
    }
    for (owner, amount) in [(Owner::Taxpayer, 5000), (Owner::Spouse, 15_000)] {
        let mut entry = income(savings_income(amount, 0));
        entry.owner = owner;
        add(&mut d, entry);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 271_500);
    // Native Form8814 adds286 qualified dividends,714 capital and500
    // other investment income. Its source parent's NII is16500, not
    // only the line9 dividends or the child's claimant's5000 interest.
    assert_eq!(value(&r, "form_8960_net_investment_income"), 21_500);
    assert_eq!(value(&r, "net_investment_income_tax"), 817);
    assert_eq!(
        [niit_optional_column(&r, "13b", "b"), niit_optional_column(&r, "13b", "c")],
        [Some(5000), Some(16_500)]
    );
    assert_eq!(
        [niit_optional_column(&r, "18", "b"), niit_optional_column(&r, "18", "c")],
        [Some(190), Some(627)]
    );
    assert_portfolio_ordinary_unchanged(&d, &r);
}

#[test]
fn unit13_child_debt_sources_cod_is_noninvestment_but_transfer_gain_is_investment() {
    for transfer in [false, true] {
        let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
        for (owner, amount) in [(Owner::Taxpayer, 5000), (Owner::Spouse, 15_000)] {
            let mut entry = income(savings_income(amount, 0));
            entry.owner = owner;
            add(&mut d, entry);
        }
        let mut entry = canceled_personal_debt(
            if transfer { 2000 } else { 5000 },
            PersonalDebtTreatment::Taxable,
        );
        entry.owner = Owner::Spouse;
        if transfer {
            let FederalIncome::PersonalDebt { data } = &mut entry.income else { unreachable!() };
            data.transfer = Some(PropertyTransfer {
                value: 1_200_000,
                property: TransferredProperty::Investment {
                    basis: 1_000_000,
                    term: GainTerm::Long,
                    collectible: false,
                },
            });
        }
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), if transfer { 274_000 } else { 275_000 });
        assert_eq!(
            value(&r, "form_8960_net_investment_income"),
            if transfer { 22_000 } else { 20_000 }
        );
        assert_eq!(value(&r, "net_investment_income_tax"), if transfer { 836 } else { 760 });
        assert_eq!(
            [niit_optional_column(&r, "13b", "b"), niit_optional_column(&r, "13b", "c")],
            [Some(5000), Some(if transfer { 19_000 } else { 20_000 })]
        );
        assert_eq!(
            [niit_optional_column(&r, "18", "b"), niit_optional_column(&r, "18", "c")],
            [Some(190), Some(if transfer { 646 } else { 570 })]
        );
        assert_portfolio_ordinary_unchanged(&d, &r);
    }
}
