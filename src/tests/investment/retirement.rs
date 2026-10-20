use super::*;

#[test]
fn worthless_security_holding_period_uses_year_end_and_inherited_rule() {
    for (acquisition, term) in [
        (SecurityAcquisition::Purchased { date: 20251230 }, GainTerm::Long),
        (SecurityAcquisition::Purchased { date: 20251231 }, GainTerm::Short),
        (SecurityAcquisition::Purchased { date: 20260101 }, GainTerm::Short),
        (SecurityAcquisition::Inherited, GainTerm::Long),
    ] {
        assert_eq!(acquisition.term(), term);
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::WorthlessSecurity {
                data: crate::WorthlessSecurity {
                    basis: 1000000,
                    acquisition,
                    worthless: true,
                    ordinary: true,
                },
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), -3000);
        assert_eq!(value(&r, "capital_loss_carryforward"), 7000);
        let FederalIncome::WorthlessSecurity { data: crate::WorthlessSecurity { basis, .. } } =
            &mut d.income[1].income
        else {
            panic!()
        };
        *basis = 100000;
        add(&mut d, income(mixed_gain_distribution()));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 9000);
        assert_eq!(value(&r, "rate_28_gain"), if term == GainTerm::Short { 2000 } else { 3000 });
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn investment_interest_separates_carryovers_and_coordinates_niit_amt() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    add(&mut d, income(savings_income(100000, 0)));
    investment_borrowing(&mut d, 70000, 50000, 10000);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 100000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 20000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 80000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 0);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 320000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn investment_interest_private_activity_debt_has_separate_amt_limit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Interest { data: native_exempt_interest(2000000, 2000000) }));
    investment_borrowing(&mut d, 0, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans[0]
        .private_activity = 3000000;
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 20000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 10000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60000);
}

#[test]
fn investment_interest_section_68_follows_limit_with_separate_amt_carryover() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 80000000, 0);
    add(&mut d, income(savings_income(100000, 0)));
    investment_borrowing(&mut d, 80000, 0, 0);
    itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 4000000;
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 60000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 20000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 80000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 0);
    assert_eq!(value(&r, "niit_itemized_deductions"), 94595);
    assert_eq!(value(&r, "net_investment_income_tax"), 205);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 820000);
}

#[test]
fn prior_capital_return_uses_separate_filed_senior_deduction() {
    let mut d = complete(FederalStatus::Single);
    let mut history = capital_history();
    history.agi = 2000000;
    history.deductions = 2500000;
    history.senior_deduction = 600000;
    history.short = -1000000;
    history.long = 0;
    history.investment_parts = Some(InvestmentCapitalParts { short: 700_000, long: 0, amt: None });
    history.origins = vec![CapitalLossOrigin { year: 2025, short: 700000, long: 0 }];
    d.screening.as_mut().unwrap().capital_history = vec![history];
    let r = supported(&d);
    assert_eq!(value(&r, "capital_short_carryforward"), 4000);
    assert_eq!(restart(d.clone()).draft, d);
    d.screening.as_mut().unwrap().capital_history[0].senior_deduction = 0;
    unsupported(&d, "Capital loss history");
}

#[test]
fn capital_carryforward_tracks_regular_nol_and_atnol_income_separately() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_short_carryforward"), 10000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 7000);
    d.screening.as_mut().unwrap().nol = NolHistory::None;
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2017, 100000, true)]);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_short_carryforward"), 7000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 7000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn trading_interest_requires_participation_and_separate_amt_facts() {
    for missing in 0..2 {
        let mut d = trading_interest_draft(10_000, false, 0);
        let FederalIncome::Scorp { data } = &mut d.income[1].income else { unreachable!() };
        if missing == 0 {
            data.participation = None;
        } else {
            data.amt = None;
        }
        let r = estimate(&d);
        assert!(r.issues.iter().any(|i| i.section == "K-1 investment interest"));
        assert!(
            !r.review_notes.iter().any(|n| n.starts_with("K-1 investment-interest worksheet:"))
        );
    }
}

#[test]
fn shared_capital_stage_keeps_nol_measurement_separate_from_loss_deduction() {
    let input =
        crate::capital::CapitalAmounts { short: -4_000, long: -6_000, ..Default::default() };
    assert_eq!(input.calculate([0, 0], false, false).income, -3_000);
    assert_eq!(input.calculate([0, 0], true, false).income, -1_500);
    let modified = input.calculate([0, 0], false, true);
    assert_eq!(modified.income, 0);
    assert_eq!((modified.short, modified.long), (-4_000, -6_000));
}

#[test]
fn married_separate_qsbs_uses_five_million_dollar_issuer_cap() {
    let mut d = complete(FederalStatus::Separate);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000_000, 610_000_000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "qsbs_2_exclusion_limit"), 5_000_000);
    assert_eq!(value(&r, "qsbs_2_taxable_gain"), 1_000_000);
}

#[test]
fn qsbs_prior_nol_refigures_traditional_ira_deduction_for_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000_000)]);
    d.adjustments.as_mut().unwrap().iras = ira_facts(200_000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic H Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "qsbs_2_excluded_gain"), 1_000);
    assert_eq!(value(&after, "ira_deduction"), value(&before, "ira_deduction"));
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 1_000);
    assert_eq!(estimate(&restart(d.clone()).draft), after);

    d.income[0] = covered_wage(1, Owner::Taxpayer, 8_500_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::Yes);
    d.income.pop();
    let phased_before = supported(&d);
    add(&mut d, qsb_income("Synthetic H Inc.", 10_000, 110_000, 0));
    let phased = supported(&d);
    assert_eq!(value(&phased, "ira_deduction"), 7_500);
    assert_eq!(value(&phased, "agi"), value(&phased_before, "agi"));
    assert_eq!(value(&phased, "nol_2025_used") - value(&phased_before, "nol_2025_used"), 1_750);
    assert_eq!(estimate(&restart(d.clone()).draft), phased);
}

#[test]
fn qsbs_prior_nol_refigures_coupled_ira_and_social_security() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 4_000_000, 0);
    add(&mut d, income(social_security_net(7_000_000, 0)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000_000)]);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::Yes);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic I Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "ira_deduction"), value(&before, "ira_deduction"));
    // Form 172's no-prior-NOL pass starts with $79,350 IRA MAGI.
    // Restoring $1,000 excluded gain also raises the before-IRA taxable
    // benefits $850, so the IRA deduction falls $150. The $150 smaller
    // deduction raises final taxable benefits another $128: $2,128 total.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 2_128);
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}

#[test]
fn qsbs_prior_nol_refigures_ira_before_itemized_agi_limits() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 8_500_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000_000)]);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::Yes);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).medical = 1_000_000;
    ded.cash_charity = 100_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic J Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "deduction"), value(&before, "deduction"));
    // No-QSBS Form 172 MAGI is $80,500; restored gain plus $750 lost IRA
    // deduction raises it to $82,250. The medical deduction loses $131
    // and the cash-gift deduction loses $8 at the new AGI floors.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 1_889);
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}
