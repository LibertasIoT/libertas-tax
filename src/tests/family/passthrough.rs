use super::*;

#[test]
fn education_exception_matches_irs_taxable_basis_examples() {
    for (taxable, expected) in [(500, 0), (1000, 20)] {
        let mut d = complete(FederalStatus::Single);
        let mut e = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            3200,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260301,
                facts: education_withdrawal(5800, 5000, false),
                early_simple: false,
            },
        );
        let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
        data.taxable = RetirementTaxable::Payer { amount: taxable * 100 };
        add(&mut d, e);
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), expected);
        assert_eq!(value(&r, "agi"), 60000 + taxable);
    }
}

#[test]
fn able_excess_is_per_beneficiary_and_not_absorbed_by_spouse_room() {
    let mut d = complete(FederalStatus::Joint);
    let a = able_contributions(0, 25000, 30000);
    let mut b = able_contributions(1000, 0, 1000);
    b.owner = Owner::Spouse;
    d.screening.as_mut().unwrap().able_contributions = vec![a, b];
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxpayer_excess_tax"), 300);
    assert_eq!(value(&r, "able_spouse_excess_tax"), 0);
    assert_eq!(return_line(&r, "23"), 30000);
    d.screening.as_mut().unwrap().able_contributions[1].owner = Owner::Taxpayer;
    assert!(!d.valid_stored());
}

#[test]
fn able_rollover_allocation_cannot_create_negative_remaining_basis() {
    for (amount, earnings) in
        [(11000, 3000), (4000, 4001), (4000, 3001), (9000, 1000), (10000, 2999), (-1, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(mixed_able(amount, earnings, 0)));
        assert!(!d.valid_stored(), "invalid allocation {amount}/{earnings}");
    }
}

#[test]
fn eic_k1_investment_test_excludes_nonpassive_ordinary_and_trading_income() {
    for kind in [ScorpTradeKind::Ordinary, ScorpTradeKind::FinancialTrading] {
        let mut source = nonpassive_scorp(13_000, 0);
        source.trade_kind = kind;
        let (d, _) = investment_tax_draft(source, 500);
        k1_eic_investment_note(&estimate(&d), 500);
    }
    let mut source = nonpassive_scorp(13_000, 0);
    source.participation = Some(participation_facts());
    let (d, _) = investment_tax_draft(source, 500);
    k1_eic_investment_note(&estimate(&d), 13_500);
}

#[test]
fn eic_k1_nonpassive_loss_cannot_offset_passive_rental_profit() {
    let (mut d, _) = investment_tax_draft(nonpassive_scorp(-15_000, 0), 500);
    add(&mut d, income(FederalIncome::Rental { data: rental(16_000, 0) }));
    k1_eic_investment_note(&estimate(&d), 6_500);
}

#[test]
fn eic_k1_former_passive_loss_uses_own_income_then_other_passive_profit() {
    for (prior, expected) in [(4_000, 6_500), (15_000, 1_500), (20_000, 500)] {
        let (mut d, _) = investment_tax_draft(nonpassive_scorp(10_000, prior), 500);
        add(&mut d, income(FederalIncome::Rental { data: rental(16_000, 0) }));
        k1_eic_investment_note(&estimate(&d), expected);
    }
}

#[test]
fn dependent_partner_standard_earnings_use_gross_services_and_zero_capital() {
    for (source, gross, expected) in [
        (BusinessIncomeSource::Services, Some(2000000), 20000),
        (BusinessIncomeSource::Capital, None, 0),
        (BusinessIncomeSource::Mixed { compensation: 500000 }, None, 3000),
    ] {
        let mut d = empty_return(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
        d.people.as_mut().unwrap().taxpayer.parent_alive = true;
        d.people.as_mut().unwrap().taxpayer.dependency =
            FilerDependency::CanBeClaimed { claimed: true };
        let mut data = ordinary_partner_source(10000, true);
        data.income_source = Some(source);
        data.gross_personal_services = gross;
        data.qbi = Some(partner_qbi_statement(10000));
        add(&mut d, income(FederalIncome::Partnership { data }));
        let r = estimate(&d);
        let expected_standard = (expected + 450).clamp(1350, 16100);
        assert!(
            r.review_notes.iter().any(|n| n.contains(&alloc::format!(
                "Dependent standard-deduction earned income from partnership sources: ${expected};"
            )) && n.contains(&alloc::format!(
                "total earned income after the shared half-SE deduction: ${};",
                expected
            )) && n
                .contains(&alloc::format!("standard deduction: ${expected_standard}."))),
            "{:?}",
            r.review_notes
        );
        let note = &r
            .review_notes
            .iter()
            .find(|n| {
                n.contains("Dependent standard-deduction earned income from partnership sources:")
            })
            .unwrap()
            .display;
        assert_eq!(note.key, "NOTE_DEPENDENT_PARTNERSHIP_STANDARD_DEDUCTION");
        assert_eq!(note.args[0], MessageArg::usd_dollars(expected));
        assert_eq!(note.args[2], MessageArg::usd_dollars(expected_standard));
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

#[test]
fn trump_account_death_value_above_basis_enters_acquirers_return_once() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    add(
        &mut d,
        income(FederalIncome::TrumpDeath {
            data: TrumpDeathAcquisition {
                beneficiary_estate: false,
                value: 150_000,
                basis: 100_000,
                withholding: 1_000,
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "trump_death_2_value"), 1_500);
    assert_eq!(value(&r, "trump_death_2_taxable"), 500);
    assert_eq!(value(&r, "trump_death_taxable_income"), 500);
    assert_eq!(value(&r, "agi"), value(&baseline, "agi") + 500);
    assert_eq!(r.payments, Some(baseline.payments.unwrap() + 1_000));
    assert_eq!(
        r.return_lines
            .iter()
            .find(|line| status_text(&line.label).starts_with("1040 8 —"))
            .unwrap()
            .amount,
        baseline
            .return_lines
            .iter()
            .find(|line| status_text(&line.label).starts_with("1040 8 —"))
            .unwrap()
            .amount
            + 50_000
    );
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    if let FederalIncome::TrumpDeath { data } = &mut d.income[1].income {
        data.basis = 200_000;
    }
    let below_basis = supported(&d);
    assert_eq!(value(&below_basis, "trump_death_taxable_income"), 0);
    assert_eq!(value(&below_basis, "agi"), value(&baseline, "agi"));

    // The estate case is the child's final return: Trump accounts began on 2026-07-04, so an
    // owner who died on 2026-07-03 cannot have held one.
    if let FederalIncome::TrumpDeath { data } = &mut d.income[1].income {
        data.beneficiary_estate = true;
    }
    let person = &mut d.people.as_mut().unwrap().taxpayer;
    person.birth_date = 20180615;
    person.died =
        Some(FinalReturn { bonds: None, date: 20260703, ordinary: true, estate_medical: 0 });
    assert_eq!(estimate(&d).state, ResultState::Unsupported);
}

#[test]
fn trump_account_estate_acquisition_belongs_on_beneficiarys_final_return() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20180615;
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20261201, ordinary: true, estate_medical: 0 });
    add(
        &mut d,
        income(FederalIncome::TrumpDeath {
            data: TrumpDeathAcquisition {
                beneficiary_estate: true,
                value: 120_000,
                basis: 100_000,
                withholding: 0,
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "trump_death_taxable_income"), 200);
    if let FederalIncome::TrumpDeath { data } = &mut d.income[1].income {
        data.beneficiary_estate = false;
    }
    unsupported(&d, "Synthetic document");
}

#[test]
fn deceased_trump_beneficiary_still_owes_unreturned_2026_excess_tax() {
    let mut d = complete(FederalStatus::Single);
    let person = &mut d.people.as_mut().unwrap().taxpayer;
    person.birth_date = 20180615;
    person.died =
        Some(FinalReturn { bonds: None, date: 20261201, ordinary: true, estate_medical: 0 });
    add(
        &mut d,
        income(FederalIncome::TrumpDeath {
            data: TrumpDeathAcquisition {
                beneficiary_estate: true,
                value: 700_000,
                basis: 500_000,
                withholding: 0,
            },
        }),
    );
    let baseline = supported(&d);
    d.screening.as_mut().unwrap().trump_accounts.push(TrumpAccount {
        owner: Owner::Taxpayer,
        contributions: 600_000,
        exempt: 0,
        returns: vec![],
        confirmed: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "trump_account_taxpayer_line57_excess"), 1_000);
    assert_eq!(value(&r, "trump_account_taxpayer_line60_remaining"), 1_000);
    assert_eq!(value(&r, "trump_account_taxpayer_line61_tax"), 60);
    assert_eq!(value(&r, "trump_death_taxable_income"), 2_000);
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
    assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax") + 60);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    d.screening.as_mut().unwrap().trump_accounts[0].returns.push(TrumpExcessReturn {
        principal: 10_000,
        earnings: 0,
        received: 20270101,
        deadline: 20270415,
        confirmed: true,
    });
    unsupported(&d, "Trump account");
}

#[test]
fn partial_partner_fuel_recovery_stays_unearned_for_dependent_deduction_and_form_8615() {
    let mut draft = young_interest_return();
    draft.people.as_mut().unwrap().taxpayer.parent_alive = true;
    let mut partner = ordinary_partner_source(20_000, true);
    partner.income_source = Some(BusinessIncomeSource::MixedThirtyPercent);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.prior_fuel_credit = Some(partial_partner_fuel_recovery());
    let source_id = draft.next_id;
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    let report = supported(&draft);
    // Only the original ordinary profit is eligible: 30% of $20,000 =
    // $6,000. Neither the $718 taxable recovery nor the $5,000 credit is
    // personal-service profit; the dependent deduction is $6,000 + $450.
    let standard = report
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_DEPENDENT_PARTNERSHIP_STANDARD_DEDUCTION")
        .unwrap();
    assert_eq!(standard.display.args[0], MessageArg::usd_dollars(6_000));
    assert_eq!(standard.display.args[2], MessageArg::usd_dollars(6_450));
    let young = report
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_PARTNERSHIP_YOUNG_FILER_EARNED")
        .unwrap();
    assert_eq!(young.display.args[1], MessageArg::usd_dollars(6_000));
    assert_eq!(
        value(&report, &alloc::format!("partnership_{source_id}_prior_fuel_credit_recovery")),
        718
    );
    assert!(value(&report, "kiddie_tax_net_unearned_income") > 0);
    assert_eq!(supported(&restart(draft).draft), report);
}
