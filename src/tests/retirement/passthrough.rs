use super::*;

#[test]
fn repeated_529_account_transfers_preserve_source_balance_and_trustee_order() {
    let mut draft = complete(FederalStatus::Single);
    draft.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
    let mut transfers = two_qtp_roth_rollovers();
    transfers[1].rollover.account_reference = "PLAN-A".into();
    transfers[1].rollover.account_opened_on = 20100101;
    transfers[0].rollover.seasoned_available = 600_000;
    transfers[1].rollover.seasoned_available = 300_000;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    let result = supported(&draft);
    assert_eq!(value(&result, "taxpayer_529_roth_form_5498_box10"), 6000);
    assert_eq!(supported(&restart(draft.clone()).draft), result);

    transfers[1].rollover.transferred_on = 20260301;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    supported(&draft); // Same-day transfers are listed in actual trustee order.
    transfers[1].rollover.seasoned_available = 300_001;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    assert!(!draft.valid_stored()); // Would count part of the first transfer twice.
    transfers[1].rollover.seasoned_available = 300_000;
    transfers[1].rollover.transferred_on = 20260602;
    transfers[1].rollover.account_opened_on = 20090101;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    assert!(!draft.valid_stored());
    transfers[1].rollover.account_opened_on = 20100101;
    transfers[1].rollover.transferred_on = 20260201;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    supported(&draft); // Different-date entries may be entered in either order.
}

#[test]
fn multiple_529_transfers_use_tax_year_roth_basis_between_transfer_dates() {
    let mut draft = complete(FederalStatus::Single);
    draft.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
    draft.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2026, 0)];
    add(&mut draft, roth_withdrawal(Owner::Taxpayer, 6000, 20261201));
    set_qtp_roth(&mut draft, Owner::Taxpayer, &two_qtp_roth_rollovers());
    let result = supported(&draft);
    assert_eq!(value(&result, "roth_2_regular_recovered"), 3500);
    assert_eq!(value(&result, "roth_2_taxable"), 2500);
    assert_eq!(return_line(&result, "4b"), 250_000);
    let FederalIncome::Roth { data } = &mut draft
        .income
        .iter_mut()
        .find(|e| matches!(e.income, FederalIncome::Roth { .. }))
        .unwrap()
        .income
    else {
        panic!()
    };
    edit_roth(data).distributed = 20260401;
    assert_eq!(supported(&draft), result);
    assert_eq!(supported(&restart(draft.clone()).draft), result);
}

#[test]
fn roth_withdrawals_before_between_and_after_529_transfers_share_annual_basis() {
    let mut draft = complete(FederalStatus::Single);
    draft.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
    // The Roth IRA already had $1,000 of prior contribution basis and at least
    // $4,000 of investment earnings available for the February withdrawal.
    draft.adjustments.as_mut().unwrap().roth_history =
        vec![roth_history(Owner::Taxpayer, 2020, 1000)];
    add(&mut draft, roth_withdrawal(Owner::Taxpayer, 5000, 20260201));
    add(&mut draft, roth_withdrawal(Owner::Taxpayer, 1000, 20260401));
    add(&mut draft, roth_withdrawal(Owner::Taxpayer, 500, 20261201));
    set_qtp_roth(&mut draft, Owner::Taxpayer, &two_qtp_roth_rollovers());
    let result = supported(&draft);
    assert_eq!(return_line(&result, "4a"), 650_000);
    assert_eq!(return_line(&result, "4b"), 200_000);
    assert_eq!(value(&result, "early_distribution_tax"), 200);
    assert_eq!(value(&result, "roth_2_regular_recovered"), 4500);
    assert_eq!(value(&result, "roth_2_taxable"), 500);
    assert_eq!(value(&result, "roth_3_taxable"), 1000);
    assert_eq!(value(&result, "roth_4_taxable"), 500);
    assert_eq!(value(&result, "taxpayer_529_roth_form_5498_box10"), 6000);
    assert_eq!(value(&result, "taxpayer_529_roth_source_contribution_basis"), 3500);
    assert_eq!(supported(&restart(draft.clone()).draft), result);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn qtp_to_roth_source_contributions_are_basis_and_source_earnings_are_not() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 100_000, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2026, 0)];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 6000, 20261201));
    set_qtp_roth(&mut d, Owner::Taxpayer, &[qtp_roth_rollover()]);
    let result = supported(&d);
    assert_eq!(return_line(&result, "4a"), 600000);
    assert_eq!(return_line(&result, "4b"), 100000);
    assert_eq!(value(&result, "roth_2_regular_recovered"), 5000);
    assert_eq!(value(&result, "roth_2_taxable"), 1000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Roth { data } = &mut d
        .income
        .iter_mut()
        .find(|e| matches!(e.income, FederalIncome::Roth { .. }))
        .unwrap()
        .income
    else {
        panic!()
    };
    edit_roth(data).distributed = 20260531;
    assert_eq!(supported(&d), result);
}

#[test]
fn actual_2027_roth_deposit_for_2026_enters_2026_basis_and_saver_once() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_500_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 200_000, Answer::No);
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2026, 0)];
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 2500, 20261201));
    let original_year = supported(&d);
    d.adjustments.as_mut().unwrap().iras.taxpayer.next_year_deposits =
        Some(next_year_ira_deposits(0, 200_000, 20270201));
    let result = supported(&d);
    assert_eq!(value(&result, "taxpayer_2026_form_5498_box10_2027_deposit"), 2000);
    assert_eq!(value(&result, "roth_2_regular_recovered"), 2000);
    assert_eq!(return_line(&result, "4b"), 50_000);
    assert_eq!(value(&result, "early_distribution_tax"), 50);
    assert_eq!(value(&result, "saver_credit"), 400);
    assert_eq!(result.tax, original_year.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn returned_roth_ira_net_loss_reduces_cash_without_tax_or_basis() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_400_000, 0);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 200_000, Answer::No);
    let base = supported(&d);
    assert!(value(&base, "saver_credit") > 0);
    d.adjustments.as_mut().unwrap().iras.taxpayer.returned_next_year = vec![ReturnedContribution {
        kind: IraKind::Roth,
        principal: 100_000,
        earnings: -15_000,
        returned_on: 20270202,
        late: None,
    }];
    let result = supported(&d);
    assert_eq!(return_line(&result, "4a"), 85_000);
    assert_eq!(return_line(&result, "4b"), 0);
    assert_eq!(value(&result, "taxpayer_roth_ira_timely_return_gross"), 850);
    assert_eq!(value(&result, "taxpayer_roth_ira_timely_return_earnings"), 0);
    assert_eq!(value(&result, "saver_credit"), value(&base, "saver_credit"));
    assert_eq!(value(&result, "agi"), value(&base, "agi"));
    assert_eq!(result.tax, base.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn ira_accepted_page_round_trip_and_special_basis_guard() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 9_100_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::Yes);
    let restored = restart(d.clone());
    assert_eq!(restored.draft, d);
    assert_eq!(value(&supported(&restored.draft), "taxpayer_nondeductible_ira"), 7500);
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(native_retirement(
                RetirementKind::Pension,
                100_000,
                (RetirementTaxable::Payer, 100_000),
            )),
        }),
    );
    supported(&d); // Pension income does not require traditional IRA basis allocation.
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).kind = RetirementKind::Ira;
    unsupported(&d, "IRA");
}
#[test]
fn ira_basis_aggregates_accounts_but_keeps_spouses_separate() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, RetirementTreatment::Ordinary));
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, RetirementTreatment::Ordinary));
    add(&mut d, basis_distribution(Owner::Spouse, 10000, RetirementTreatment::Ordinary));
    d.adjustments.as_mut().unwrap().ira_basis = vec![
        prior_ira_basis(Owner::Taxpayer, 20000, 80000),
        prior_ira_basis(Owner::Spouse, 5000, 0),
    ];
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 4000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 16000);
    assert_eq!(value(&r, "spouse_ira_basis_recovered"), 5000);
    assert_eq!(value(&r, "spouse_ira_basis_carryforward"), 0);
    assert_eq!(return_line(&r, "4a"), 3_000_000);
    assert_eq!(return_line(&r, "4b"), 2_100_000);
    assert_eq!(restart(d.clone()).draft, d);
    let mut reordered = d.clone();
    reordered.income.reverse();
    assert_eq!(supported(&reordered).tax, r.tax);
}
#[test]
fn ira_basis_reduces_early_tax_and_does_not_pretend_preview_is_known() {
    let mut d = complete(FederalStatus::Single);
    let e = basis_distribution(Owner::Taxpayer, 20000, RetirementTreatment::Early);
    assert_eq!(crate::interview::document_amount(&e.income, false), None);
    add(&mut d, e);
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 20000, 80000)];
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 1600);
    assert_eq!(return_line(&r, "4b"), 1_600_000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::EarlySimple);
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 4000);
}
#[test]
fn ira_basis_missing_mixed_duplicate_and_special_facts_block_totals() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, basis_distribution(Owner::Taxpayer, 20000, RetirementTreatment::Ordinary));
    unsupported(&d, "IRA basis");
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 10000, 30000)];
    supported(&d);
    let mut bad = d.clone();
    bad.adjustments.as_mut().unwrap().ira_basis.push(prior_ira_basis(Owner::Taxpayer, 0, 0));
    unsupported(&bad, "IRA basis");
    let mut bad = d.clone();
    add(
        &mut bad,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    unsupported(&bad, "IRA basis");
    let mut bad = d.clone();
    bad.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 10000;
    unsupported(&bad, "IRA basis");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).kind = RetirementKind::Pension;
    unsupported(&d, "IRA basis");
}
#[test]
fn ira_basis_zero_balances_and_losses_preserve_unused_basis() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 10000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10000);
    add(&mut d, basis_distribution(Owner::Taxpayer, 6000, RetirementTreatment::Early));
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 6000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 4000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    d.adjustments.as_mut().unwrap().ira_basis[0].prior = 0;
    assert_eq!(return_line(&supported(&d), "4b"), 600000);
}
#[test]
fn current_ira_insufficient_basis_uses_form_8606_after_fixing_deduction() {
    for (wages, deduction, recovered, carry) in [(60000, 7500, 0, 0), (73000, 7130, 74, 296)] {
        let d = ira_contribution_distribution(wages, Answer::Yes);
        let r = supported(&d);
        assert_eq!(value(&r, "ira_deduction"), deduction);
        assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), recovered);
        assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), carry);
        assert_eq!(return_line(&r, "4b"), (10000 - recovered) * 100);
        assert_eq!(value(&r, "agi"), wages + 10000 - recovered - deduction);
    }
}
#[test]
fn fully_deductible_ira_contribution_does_not_create_tax_free_basis() {
    let d = ira_contribution_distribution(200000, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(return_line(&r, "4b"), 1000000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    let mut d = d;
    d.adjustments.as_mut().unwrap().ira_basis[0].prior = 1000000;
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 2000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 8000);
}

#[test]
fn joint_ira_basis_and_contributions_keep_owner_balances_and_coverage() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![covered_wage(1, Owner::Taxpayer, 14000000, 0)];
    let spouse_wage = covered_wage(d.next_id, Owner::Spouse, 0, 0);
    add(&mut d, spouse_wage);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, RetirementTreatment::Ordinary));
    add(&mut d, basis_distribution(Owner::Spouse, 5000, RetirementTreatment::Ordinary));
    let a = d.adjustments.as_mut().unwrap();
    a.ira_basis = vec![
        prior_ira_basis(Owner::Taxpayer, 0, 40000),
        prior_ira_basis(Owner::Spouse, 5000, 45000),
    ];
    let mut f = ira_facts(750000, 0, Answer::Yes);
    f.spouse = IraSpouse::Joint {
        facts: FederalIraContribution {
            nondeductible_election: 0,
            traditional: 750000,
            roth: 0,
            covered_at_work: true,
            returned_next_year: vec![],
            excess: None,
            next_year_deposits: None,
            qtp_roth_lifetime_prior: 0,
        },
    };
    a.iras = f;
    let r = supported(&d);
    // Preliminary exclusions: 1,500 and 1,250. MAGI 152,250 exceeds 149,000.
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 6000);
    assert_eq!(value(&r, "spouse_ira_basis_carryforward"), 11250);
    assert_eq!(return_line(&r, "4b"), 1225000);
    assert_eq!(value(&r, "agi"), 152250);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
#[test]
fn partial_roth_conversion_allocates_basis_and_taxes_only_kept_money_as_early() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, conversion(8000, true)));
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 10000, 40000)];
    let r = supported(&d);
    assert_eq!(value(&r, "roth_conversion_taxable"), 6400);
    assert_eq!(value(&r, "early_distribution_tax"), 160);
    assert_eq!(return_line(&r, "4b"), 800000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 8000);
    assert_eq!(r.payments, Some(662300)); // Wages plus source Form 1099-R withholding.
    // Splitting the same statement into converted and kept entries preserves tax.
    let mut split = d.clone();
    split.income.pop();
    add(&mut split, basis_distribution(Owner::Taxpayer, 8000, conversion(8000, true)));
    add(&mut split, basis_distribution(Owner::Taxpayer, 2000, RetirementTreatment::Early));
    assert_eq!(supported(&split).tax, r.tax);
    split.income.reverse();
    assert_eq!(supported(&split).tax, r.tax);
}
#[test]
fn current_nondeductible_contribution_and_full_conversion_exhaust_basis() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 20000000, 0);
    add(&mut d, basis_distribution(Owner::Taxpayer, 7500, conversion(7500, true)));
    let a = d.adjustments.as_mut().unwrap();
    a.iras = ira_facts(750000, 0, Answer::Yes);
    a.ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 7500);
    assert_eq!(value(&r, "roth_conversion_taxable"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(value(&r, "agi"), 200000);
    d.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 750000;
    let r = supported(&d);
    assert_eq!(value(&r, "roth_conversion_taxable"), 7500);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 7500);
}
#[test]
fn ira_rollover_basis_matches_same_year_and_outstanding_completion() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        basis_distribution(Owner::Taxpayer, 10000, rollover(8000, RolloverDestination::Ira)),
    );
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 10000, 48000)];
    let same_year = supported(&d);
    assert_eq!(return_line(&same_year, "4a"), 1000000);
    assert_eq!(return_line(&same_year, "4b"), 160000);
    assert_eq!(value(&same_year, "taxpayer_ira_basis_carryforward"), 9600);
    assert_eq!(value(&same_year, "early_distribution_tax"), 160);
    assert_eq!(value(&same_year, "taxpayer_ira_outstanding_rollover"), 0);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).ordinary_distribution =
        Some(rollover(8000, RolloverDestination::IraNextYear));
    d.adjustments.as_mut().unwrap().ira_basis[0].year_end = 4000000;
    let next_year = supported(&d);
    assert_eq!(value(&next_year, "taxpayer_ira_outstanding_rollover"), 8000);
    assert_eq!(next_year.tax, same_year.tax);
    assert_eq!(next_year.payments, same_year.payments);
    assert_eq!(value(&next_year, "taxpayer_ira_basis_carryforward"), 9600);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn full_ira_rollover_does_not_consume_basis() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        basis_distribution(
            Owner::Taxpayer,
            50000,
            rollover(50000, RolloverDestination::IraNextYear),
        ),
    );
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 10000, 0)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}
#[test]
fn ira_to_employer_plan_leaves_after_tax_basis_outside_the_plan() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        basis_distribution(
            Owner::Taxpayer,
            90000,
            rollover(90000, RolloverDestination::EmployerPlan),
        ),
    );
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 10000, 10000)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10000);
    assert_eq!(value(&r, "taxpayer_ira_outstanding_rollover"), 0);
    d.adjustments.as_mut().unwrap().ira_basis[0].year_end = 500000;
    unsupported(&d, "IRA rollover");
}
#[test]
fn pretax_plan_rollover_and_roth_conversion_share_one_ira_basis_pool() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        basis_distribution(
            Owner::Taxpayer,
            90000,
            rollover(90000, RolloverDestination::EmployerPlan),
        ),
    );
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, conversion(10000, true)));
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 10000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "retirement_rollovers"), 90000);
    assert_eq!(value(&r, "roth_conversion_taxable"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(return_line(&r, "4a"), 10000000);
    assert_eq!(return_line(&r, "4b"), 0);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn current_ira_contribution_and_partial_rollover_preserve_new_basis() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 20000000, 0);
    add(
        &mut d,
        basis_distribution(Owner::Taxpayer, 10000, rollover(8000, RolloverDestination::Ira)),
    );
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 48000)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 300);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 7200);
    assert_eq!(return_line(&r, "4b"), 170000);
    assert_eq!(value(&r, "early_distribution_tax"), 170);
}
#[test]
fn qcd_basis_reproduces_irs_amy_example_and_keeps_charity_deduction_separate() {
    // Pub. 590-B: $30,000 IRA, $10,000 basis, $25,000 trustee gift.
    let d = qcd_with_basis(25000, 25000, 10000, 5000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 20000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 5000);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 5000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 5000);
    assert_eq!(return_line(&r, "4a"), 2500000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn qcd_takes_pretax_funds_before_other_distributions_use_basis() {
    let mut d = qcd_with_basis(15000, 10000, 10000, 35000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 10000);
    assert_eq!(return_line(&r, "4b"), 375000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 8750);
    assert_eq!(value(&r, "qcd_cash_gifts"), 0);
    // A second payer entry for the ordinary portion gives the same result.
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).gross = 1000000;
    add(&mut d, basis_distribution(Owner::Taxpayer, 5000, RetirementTreatment::Ordinary));
    assert_eq!(supported(&d).tax, r.tax);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
#[test]
fn qcd_prior_deduction_reduction_does_not_reuse_basis_against_pretax_gifts() {
    let d = qcd_with_basis(25000, 25000, 10000, 5000, 4000);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 16000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 9000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 5000);
    assert_eq!(return_line(&r, "4b"), 400000);
    assert_eq!(value(&r, "qcd_adjustment_carryforward"), 0);
    let d = qcd_with_basis(25000, 25000, 10000, 5000, 30000);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 0);
    assert_eq!(return_line(&r, "4b"), 2000000);
    assert_eq!(value(&r, "qcd_adjustment_carryforward"), 10000);
}
#[test]
fn qcd_annual_cap_and_all_basis_accounts_do_not_double_exclude() {
    let d = qcd_with_basis(120000, 120000, 50000, 80000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 111000);
    assert_eq!(return_line(&r, "4b"), 900000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 9000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 50000);
    let d = qcd_with_basis(5000, 5000, 10000, 5000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 0);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "qcd_cash_gifts"), 5000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 5000);
}
#[test]
fn qcd_basis_keeps_spouse_pretax_capacity_separate_and_requires_basis_facts() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19500101;
    add(&mut d, basis_distribution(Owner::Taxpayer, 25000, RetirementTreatment::Ordinary));
    add(&mut d, basis_distribution(Owner::Spouse, 10000, RetirementTreatment::Ordinary));
    let a = d.adjustments.as_mut().unwrap();
    a.ira_basis = vec![
        prior_ira_basis(Owner::Taxpayer, 10000, 5000),
        prior_ira_basis(Owner::Spouse, 10000, 0),
    ];
    a.qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261215, 25000, 0), qcd(Owner::Spouse, 20261215, 10000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_taxpayer_exclusion"), 20000);
    assert_eq!(value(&r, "qcd_spouse_exclusion"), 0);
    assert_eq!(value(&r, "qcd_cash_gifts"), 15000);
    assert_eq!(value(&r, "spouse_ira_basis_carryforward"), 0);
    assert_eq!(return_line(&r, "4b"), 0);
    d.adjustments.as_mut().unwrap().ira_basis.pop();
    unsupported(&d, "QCD");
}

#[test]
fn qcd_current_deduction_reduces_exclusion_and_preserves_prior_basis() {
    let mut d = qcd_with_basis(25000, 25000, 10000, 10000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(500_000, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 5000);
    assert_eq!(value(&r, "qcd_exclusion"), 20000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 5000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10000);
    assert_eq!(return_line(&r, "4b"), 500_000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);
    // A contribution for 2026 paid in 2027 still reduces the 2026 QCD exclusion.
    d.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 500_000;
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 20000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10000);
}

#[test]
fn qcd_current_contribution_without_basis_and_full_deduction_verification() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            20000,
            RetirementTreatment::Ordinary,
        ),
    );
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261215, 10000, 2000)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(500_000, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 5000);
    assert_eq!(value(&r, "qcd_exclusion"), 3000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 7000);
    assert_eq!(return_line(&r, "4b"), 1_700_000);
    assert_eq!(value(&r, "agi"), 72000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(500_000, 0, Answer::Yes);
    let covered = supported(&d);
    assert_eq!(value(&covered, "ira_deduction"), 5000);
    assert_eq!(value(&covered, "qcd_exclusion"), 3000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(900_000, 0, Answer::No);
    unsupported(&d, "IRA contributions");
}

#[test]
fn qcd_covered_fully_deductible_contribution_with_prior_basis() {
    let mut d = qcd_with_basis(25000, 25000, 10000, 10000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(500_000, 0, Answer::Yes);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 5000);
    assert_eq!(value(&r, "qcd_exclusion"), 20000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10000);
    assert_eq!(return_line(&r, "4b"), 500_000);
    assert_eq!(value(&r, "agi"), 60000);
}

#[test]
fn partial_ira_nondeductible_election_caps_deduction_but_not_mandatory_basis() {
    let d = complete(FederalStatus::Single);
    let mut f = ira_facts(750_000, 0, Answer::Yes);
    f.taxpayer.nondeductible_election = 200_000;
    for (magi, deduction, nondeductible) in [(60000, 5500, 2000), (86000, 3750, 3750)] {
        let r = crate::ira::calculate(
            &f,
            d.people.as_ref().unwrap(),
            d.setup.as_ref().unwrap(),
            [100000, 0],
            magi,
            &Default::default(),
            [0, 0],
        )
        .unwrap();
        assert_eq!(r.deduction, deduction);
        assert_eq!(r.nondeductible[0], nondeductible);
    }
    f.taxpayer.nondeductible_election = 750_001;
    assert_eq!(
        crate::ira::calculate(
            &f,
            d.people.as_ref().unwrap(),
            d.setup.as_ref().unwrap(),
            [100000, 0],
            60000,
            &Default::default(),
            [0, 0]
        )
        .err()
        .unwrap()
        .0,
        "ISSUE_IRA_NONDEDUCTIBLE_ELECTION"
    );
}

#[test]
fn qcd_partial_election_resolves_rounding_cycle_and_preserves_chosen_basis() {
    let mut d = qcd_with_basis(10000, 10000, 0, 50000, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 9_085_000, 650_000);
    let mut f = ira_facts(860_000, 0, Answer::Yes);
    f.taxpayer.nondeductible_election = 850_000;
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 100);
    assert_eq!(value(&r, "qcd_exclusion"), 9900);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 8500);
    assert_eq!(value(&r, "agi"), 90850);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn railroad_basis_does_not_shelter_noncontributory_benefits() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    add(&mut d, railroad_pension(500, 2000, Some(pension_basis())));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 200000);
    assert_eq!(return_line(&r, "5b"), 150000);
    assert_eq!(value(&r, "pension_2_income_exclusion"), 500);
    assert_eq!(value(&r, "agi"), 61500);
    assert_eq!(r.payments, Some(662300));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn variable_annuity_shortfall_without_election_and_basis_exhaustion() {
    let mut a = variable_annuity();
    a.started = 20250101;
    a.history = VariableHistory::Prior { annual: 60000, recovered: 60000, refigure: None };
    assert!(a.valid());
    assert_eq!(a.excluded(50000), 50000);
    assert_eq!(a.annual(), 60000);
    a.history = VariableHistory::Prior { annual: 60000, recovered: 1199900, refigure: None };
    assert_eq!(a.excluded(120000), 100);
    a.history = VariableHistory::Prior { annual: 60000, recovered: 1200000, refigure: None };
    assert_eq!(a.excluded(120000), 0);
}

#[test]
fn public_safety_excludes_pension_income_and_early_tax_after_taxable_basis() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            RetirementTreatment::Early,
        ),
    );
    let id = d.income.last().unwrap().id;
    public_safety(&mut d, id, 4000, 4000, PublicSafetyCoverage::Medical);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 67000);
    assert_eq!(return_line(&r, "5a"), 1000000);
    assert_eq!(return_line(&r, "5b"), 700000);
    assert_eq!(value(&r, "early_distribution_tax"), 700);
    assert_eq!(value(&r, "public_safety_1_excluded"), 3000);
    assert_eq!(value(&r, "public_safety_medical_before_coordination"), 1000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn public_safety_simplified_method_keeps_basis_recovery_separate() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, pension_entry(pension_basis(), 14400));
    let id = d.income.last().unwrap().id;
    public_safety(&mut d, id, 4000, 4000, PublicSafetyCoverage::Medical);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 1020000);
    assert_eq!(value(&r, "agi"), 70200);
    assert_eq!(value(&r, &alloc::format!("pension_{id}_basis_recovery")), 1200);
}

#[test]
fn immediate_annuity_basis_start_must_agree_with_exception_start() {
    let mut d = nonqualified_exception(immediate_annuity(20250101, 20260101));
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).gross = 300000;
    edit_retirement(data).taxable =
        Some(RetirementTaxable::GeneralAnnuity { data: fixed_annuity() });
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "annuity_2_basis_recovery"), 1500);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception { facts, .. }) =
            &mut retirement_edit.ordinary_distribution
        else {
            panic!()
        };
        *facts = immediate_annuity(20250101, 20251231);
    }
    unsupported(&d, "Income:");
}

#[test]
fn mixed_employer_distribution_rolls_pretax_first_without_consuming_more_basis() {
    for destination in [
        RolloverDestination::Ira,
        RolloverDestination::IraNextYear,
        RolloverDestination::EmployerPlan,
    ] {
        for rolled in [0, 500, 1000] {
            let mut d = qualified_withdrawal(5000, 12500, 10000);
            let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
            edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Rollover {
                destination,
                amount: rolled * 100,
                early: true,
            });
            let r = supported(&d);
            assert_eq!(return_line(&r, "5a"), 500000);
            assert_eq!(return_line(&r, "5b"), (1000 - rolled) * 100);
            assert_eq!(value(&r, "early_distribution_tax"), (1000 - rolled) / 10);
            assert_eq!(value(&r, "pension_2_basis_recovery"), 4000);
            assert_eq!(value(&r, "pension_2_remaining_basis"), 6000);
            assert_eq!(value(&r, "net_investment_income_tax"), 0);
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

#[test]
fn direct_plan_rollover_preserves_separate_after_tax_basis_without_ira_basis() {
    for (rolled, rolled_basis, taxable) in [(500, 0, 500), (3000, 2000, 0), (5000, 4000, 0)] {
        let mut d = qualified_withdrawal(5000, 12500, 10000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Rollover {
            destination: RolloverDestination::DirectEmployerPlan { confirmed: true },
            amount: rolled * 100,
            early: true,
        });
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), taxable * 100);
        assert_eq!(value(&r, "early_distribution_tax"), taxable / 10);
        assert_eq!(value(&r, "pension_2_basis_recovery"), 4000);
        assert_eq!(value(&r, "pension_2_remaining_basis"), 6000);
        assert_eq!(value(&r, "pension_2_rolled_basis"), rolled_basis);
        assert!(d.adjustments.as_ref().unwrap().ira_basis.is_empty());
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn roth_and_traditional_early_tax_round_together_and_spouse_basis_stays_separate() {
    let mut d = complete(FederalStatus::Joint);
    d.adjustments.as_mut().unwrap().roth_history =
        vec![roth_history(Owner::Taxpayer, 2022, 1000), roth_history(Owner::Spouse, 2022, 0)];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 1000, 20260101));
    add(&mut d, roth_withdrawal(Owner::Spouse, 25, 20260101));
    add(
        &mut d,
        retirement_entry(Owner::Spouse, RetirementKind::Ira, 25, RetirementTreatment::Early),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 5000);
    assert_eq!(value(&r, "early_distribution_tax"), 5);
    d.income.reverse();
    assert_eq!(supported(&d), r);
}

#[test]
fn roth_hsa_funding_uses_earnings_first_and_preserves_regular_basis() {
    let d = roth_hsa_funding(1500, 500, 2000, 500);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 200000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "roth_2_regular_recovered"), 500);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260116"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    let d = roth_hsa_funding(1900, 500, 2000, 100);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260116"), 400);
    assert_eq!(value(&r, "roth_2_regular_recovered"), 100);
    assert_eq!(return_line(&r, "4b"), 0);
}

#[test]
fn roth_hsa_funding_basis_adjustment_precedes_later_current_contributions() {
    let mut d = roth_hsa_funding(1900, 500, 2000, 300);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 20000, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260116"), 400);
    assert_eq!(value(&r, "roth_2_regular_recovered"), 300);
    assert_eq!(return_line(&r, "4b"), 0);
}

#[test]
fn roth_hsa_two_transfers_share_remaining_basis_in_date_order() {
    let mut d = roth_hsa_funding(4000, 2000, 10000, 0);
    let id = d.next_id;
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 5000, 20260215));
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    let mut months = vec![HsaCoverage::Family; 12];
    months[0] = HsaCoverage::SelfOnly;
    h.coverage = HsaEligibility::Monthly { months };
    h.funding.push(HsaFunding {
        ongoing_sep_simple: false,
        income: id,
        funded: 20260216,
        amount: 400000,
        basis: HsaFundingBasis::Roth { value: 500000, before_other_activity: true },
        coverage: HsaCoverage::Family,
        loss: None,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260116"), 0);
    assert_eq!(value(&r, "hsa_funding_basis_3_20260216"), 1000);
    assert_eq!(value(&r, "roth_3_regular_recovered"), 1000);
    assert_eq!(return_line(&r, "4a"), 900000);
    assert_eq!(return_line(&r, "4b"), 0);
    d.adjustments.as_mut().unwrap().hsa[0].funding.reverse();
    assert_eq!(supported(&d), r);
}

#[test]
fn inherited_roth_recovers_allocated_basis_and_exempts_early_tax() {
    let mut d = complete(FederalStatus::Single);
    // Pub. 590-B beneficiary example amounts, with an unmatured 2022 clock in 2026.
    add_inherited_roth(&mut d, 2022, 4000, 1000, 2500);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 400000);
    assert_eq!(return_line(&r, "4b"), 50000);
    assert_eq!(value(&r, "inherited_roth_2_regular_recovered"), 1000);
    assert_eq!(value(&r, "inherited_roth_2_conversion_recovered"), 2500);
    for (number, expected) in
        [("19", 4000), ("22", 1000), ("23", 3000), ("24", 2500), ("25a", 500), ("25c", 500)]
    {
        assert_eq!(value(&r, &alloc::format!("inherited_roth_2_form_line{number}")), expected);
    }
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn inherited_roth_clock_continues_after_death_without_using_beneficiary_age() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let mut e = inherited_roth(2021, 0, 0);
    if let FederalIncome::InheritedIra { data } = &mut e.income {
        data.died = 20221201;
        if let crate::InheritedIraKindFacts::Roth { basis, .. } = &mut data.kind {
            *basis = None;
        }
    }
    add(&mut d, e);
    add(&mut d, inherited_roth_payment(Owner::Taxpayer, Some(id), 4000));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 400000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(
        r.payments.unwrap() - supported(&complete(FederalStatus::Single)).payments.unwrap(),
        2000
    );
}

#[test]
fn inherited_roth_never_borrows_personal_or_another_decedents_basis() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2022, 5000)];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 6000, 20260601));
    add_inherited_roth(&mut d, 2022, 4000, 1000, 2500);
    add_inherited_roth(&mut d, 2023, 1000, 0, 0);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 250000);
    assert_eq!(value(&r, "early_distribution_tax"), 100);
    let rows: Vec<_> = crate::draft_return(&r)
        .into_iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8606))
        .collect();
    let mut starts: Vec<_> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, row)| (row.form_line.as_ref().unwrap().line == "19").then_some(i))
        .collect();
    assert_eq!(starts.len(), 3);
    starts.push(rows.len());
    for span in starts.windows(2) {
        assert_eq!(
            rows[span[0]..span[1]]
                .iter()
                .filter(|row| row.form_line.as_ref().unwrap().line == "25c")
                .count(),
            1
        );
    }
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn inherited_roth_partial_withdrawal_retains_decedent_specific_basis() {
    let mut d = complete(FederalStatus::Joint);
    let id = d.next_id;
    let mut e = inherited_roth(2022, 1000, 2500);
    e.owner = crate::DocumentOwner::Spouses;
    add(&mut d, e);
    add(&mut d, inherited_roth_payment(Owner::Spouse, Some(id), 2000));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "inherited_roth_2_regular_remaining"), 0);
    assert_eq!(value(&r, "inherited_roth_2_conversion_remaining"), 1500);
    assert_eq!(value(&r, "inherited_roth_2_form_line23"), 1000);
    assert_eq!(value(&r, "inherited_roth_2_form_line24"), 2500);
    assert!(r.lines.iter().all(|row| row.id != "inherited_roth_2_form_line25b"
        && row.id != "inherited_roth_2_form_line25c"));
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn inherited_roth_without_basis_taxes_earnings_with_a_note_and_a_qualified_one_needs_none() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let mut e = inherited_roth(2022, 0, 0);
    if let FederalIncome::InheritedIra { data } = &mut e.income
        && let crate::InheritedIraKindFacts::Roth { basis, .. } = &mut data.kind
    {
        *basis = None;
    }
    add(&mut d, e);
    add(&mut d, inherited_roth_payment(Owner::Taxpayer, Some(id), 4000));
    // The owner's five years are not over and no basis was entered: every payment is earnings.
    let r = supported(&d);
    assert!(noted(&r, "WARN_INHERITED_ROTH_BASIS_ASSUMED_NONE"));
    assert_eq!(return_line(&r, "4a"), 400000);
    assert_eq!(return_line(&r, "4b"), 400000);
    // A first Roth year of 2021 or earlier is qualified: no basis is needed.
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income
        && let crate::InheritedIraKindFacts::Roth { first_year, .. } = &mut data.kind
    {
        *first_year = 2021;
    }
    let r = supported(&d);
    assert!(!noted(&r, "WARN_INHERITED_ROTH_BASIS_ASSUMED_NONE"));
    assert_eq!(return_line(&r, "4b"), 0);
    // A first year after the death is not a record at all.
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income {
        data.died = 20201201;
        assert!(!data.valid());
    }
}

#[test]
fn rmd_joint_table_requires_more_than_ten_years_and_sole_beneficiary() {
    let d = own_rmd(19510615, RmdPeriod::Current, 24600, Some(19610615));
    assert_eq!(value(&supported(&d), "rmd_0_account_0_minimum"), 1000);
    let d = own_rmd(19510615, RmdPeriod::Current, 24600, None);
    assert_eq!(value(&supported(&d), "rmd_0_account_0_minimum"), 1000);
    let mut d = own_rmd(19510615, RmdPeriod::Current, 24600, Some(19620615));
    if let RmdMinimum::OwnIras { accounts, .. } =
        &mut d.adjustments.as_mut().unwrap().required_distributions[0].required
    {
        accounts[0].spouse.as_mut().unwrap().confirmed = false;
    }
    // No: the Uniform Lifetime table (the larger minimum).
    let r = supported(&d);
    let uniform = own_rmd(19510615, RmdPeriod::Current, 24600, None);
    assert_eq!(
        value(&r, "rmd_0_account_0_minimum"),
        value(&supported(&uniform), "rmd_0_account_0_minimum")
    );
}
#[test]
fn inherited_rmd_uses_longer_remaining_owner_life_for_older_beneficiary() {
    let d = beneficiary_rmd(19500615, 20250101, 19400615, InheritedIraKind::Traditional, 13800);
    // Owner age 75 at death: 14.8 minus one year. Beneficiary age 86: 7.6.
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 1000);
}
#[test]
fn inherited_ira_allocates_basis_and_reports_income_without_early_tax() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add_inherited_ira(&mut d, 10000, 5000, 40000);
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 1000);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 4000);
    assert_eq!(return_line(&r, "4a"), 1000000);
    assert_eq!(return_line(&r, "4b"), 900000);
    assert_eq!(value(&r, "agi") - value(&before, "agi"), 9000);
    assert_eq!(r.payments.unwrap() - before.payments.unwrap(), 12300);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn inherited_ira_basis_never_crosses_personal_or_other_decedent_groups() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_ira(&mut d, 10000, 5000, 40000);
    add_inherited_ira(&mut d, 2000, 10000, 0);
    add(&mut d, basis_distribution(Owner::Taxpayer, 1000, RetirementTreatment::Ordinary));
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 5000, 4000)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 1300000);
    assert_eq!(return_line(&r, "4b"), 900000);
    assert_eq!(value(&r, "inherited_ira_4_basis_remaining"), 8000);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
#[test]
fn inherited_ira_documented_no_basis_and_a_payment_with_no_record_are_distinct() {
    // No basis entered: the payments are taxed in full (the record documents none).
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let mut e = inherited_ira(0, 0);
    *inherited_basis_mut(&mut e) = None;
    add(&mut d, e);
    add(&mut d, inherited_payment(Owner::Taxpayer, Some(id), 10000));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 1000000);
    assert!(!noted(&r, "WARN_INHERITED_IRA_RECORD_MISSING"));
    // A payment that names no record, with none to use, is taxed in full with a notice.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, inherited_payment(Owner::Taxpayer, None, 10000));
    let r = supported(&d);
    assert!(noted(&r, "WARN_INHERITED_IRA_RECORD_MISSING"));
    assert_eq!(return_line(&r, "4a"), 1000000);
    assert_eq!(return_line(&r, "4b"), 1000000);
    // An empty choice uses the owner's only record; with two, none is chosen.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, inherited_ira(5000, 40000));
    add(&mut d, inherited_payment(Owner::Taxpayer, None, 10000));
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 1000);
    assert_eq!(return_line(&r, "4b"), 900000);
    add(&mut d, inherited_ira(2000, 0));
    let r = supported(&d);
    assert!(noted(&r, "WARN_INHERITED_IRA_RECORD_MISSING"));
    assert_eq!(return_line(&r, "4b"), 1000000);
}
#[test]
fn inherited_ira_form8606_rounding_and_empty_balance_preserve_unused_basis() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_ira(&mut d, 10000, 1000, 20000);
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 333);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 667);
    assert_eq!(return_line(&r, "4b"), 966700);
    if let FederalIncome::Retirement { data } = &mut d.income[2].income {
        edit_retirement(data).gross = 0;
        if matches!(
            crate::FederalRetirement::from(&*data).taxable,
            Some(RetirementTaxable::Inherited { .. })
        ) {
            edit_retirement(data).box_2a = 0;
        }
    }
    inherited_basis_mut(&mut d.income[1]).as_mut().unwrap().year_end = 0;
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 1000);
    assert_eq!(return_line(&r, "4b"), 0);
}

#[test]
fn inherited_qcd_uses_its_group_pretax_capacity_before_basis_recovery() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    let id = d.next_id;
    add_inherited_ira(&mut d, 10000, 5000, 40000);
    let mut gift = qcd(Owner::Taxpayer, 20260601, 5000, 0);
    gift.payments[0].inherited = Some(id);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 5000);
    // Remaining 5,000 distribution / 45,000 balance recovers 556 basis.
    assert_eq!(value(&r, &alloc::format!("inherited_ira_{id}_basis_recovered")), 556);
    assert_eq!(value(&r, &alloc::format!("inherited_ira_{id}_basis_remaining")), 4444);
    assert_eq!(return_line(&r, "4a"), 1000000);
    assert_eq!(return_line(&r, "4b"), 444400);
    for (number, expected) in
        [("2", 5000), ("6", 40000), ("7", 5000), ("12", 556), ("14", 4444), ("15c", 4444)]
    {
        assert_eq!(value(&r, &alloc::format!("inherited_ira_{id}_form_line{number}")), expected);
    }
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn inherited_qcd_checks_beneficiary_age_source_ownership_and_gross() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19551201;
    let id = d.next_id;
    add_inherited_ira(&mut d, 10000, 0, 0);
    let mut gift = qcd(Owner::Taxpayer, 20260601, 5000, 0);
    gift.payments[0].inherited = Some(id);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    supported(&d);
    let mut bad = d.clone();
    bad.adjustments.as_mut().unwrap().qualified_charity[0].payments[0].date = 20260531;
    unsupported(&bad, "QCD");
    let mut bad = d.clone();
    bad.adjustments.as_mut().unwrap().qualified_charity[0].payments[0].inherited = Some(9999);
    unsupported(&bad, "QCD");
    let mut bad = d.clone();
    bad.income.iter_mut().find(|e| e.id == id).unwrap().owner = crate::DocumentOwner::Spouses;
    unsupported(&bad, "QCD");
    let mut bad = d.clone();
    bad.adjustments.as_mut().unwrap().qualified_charity[0].payments[0].amount = 1000001;
    unsupported(&bad, "QCD");
    if let FederalIncome::InheritedIra { data } =
        &mut d.income.iter_mut().find(|e| e.id == id).unwrap().income
    {
        data.died = 20260701;
    }
    unsupported(&d, "QCD");
}
#[test]
fn inherited_estate_tax_uses_pub559_ratio_without_reducing_agi_or_amt_deduction() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_ira(&mut d, 12000, 0, 0);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let before = supported(&d);
    set_inherited_estate(&mut d, inherited_estate(12000));
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_estate_tax_deduction"), 2772);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(value(&r, "taxable_income"), value(&before, "taxable_income") - 2772);
    assert_eq!(
        value(&r, "alternative_minimum_taxable_income"),
        value(&before, "alternative_minimum_taxable_income") - 2772
    );
    assert_eq!(value(&r, "inherited_ira_2_estate_ird_value_remaining"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.deductions.as_mut().unwrap().itemize_override = None;
    assert_eq!(value(&supported(&d), "inherited_estate_tax_deduction"), 0);
}
#[test]
fn inherited_estate_tax_caps_value_and_excludes_basis_and_qcd_income() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add_inherited_ira(&mut d, 10000, 5000, 40000);
    set_inherited_estate(&mut d, inherited_estate(12000));
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_ira_2_estate_tax_deduction"), 2079);
    assert_eq!(value(&r, "inherited_ira_2_estate_ird_value_remaining"), 3000);
    let mut gift = qcd(Owner::Taxpayer, 20260601, 5000, 0);
    gift.payments[0].inherited = Some(2);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    let r = supported(&d);
    // Remaining taxable income is 4,444, and 4,444 * 4,620 / 20,000 rounds to 1,027.
    assert_eq!(value(&r, "inherited_ira_2_estate_tax_deduction"), 1027);
    assert_eq!(value(&r, "inherited_ira_2_estate_ird_value_remaining"), 7556);
    set_inherited_estate(&mut d, inherited_estate(1000));
    assert_eq!(value(&supported(&d), "inherited_ira_2_estate_tax_deduction"), 231);
}
#[test]
fn inherited_estate_tax_rejects_inconsistent_source_values_and_skips_unresolved_ones() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_ira(&mut d, 12000, 0, 0);
    // Unconfirmed source history: no deduction this year, with its notice.
    set_inherited_estate(
        &mut d,
        InheritedEstateTax { confirmed: false, ..inherited_estate(12000) },
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_IRD_ESTATE_TAX_NOT_DEDUCTED"));
    assert!(!r.lines.iter().any(|l| l.id == "inherited_ira_2_estate_tax_deduction"));
    for bad in [
        InheritedEstateTax { total_ird: 0, ..inherited_estate(12000) },
        InheritedEstateTax { without_ird: 946001, ..inherited_estate(12000) },
        inherited_estate(20001),
    ] {
        set_inherited_estate(&mut d, bad);
        unsupported(&d, "Inherited estate tax");
    }
}
#[test]
fn inherited_estate_tax_qcd_adjustment_is_determined_for_one_source() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add_inherited_ira(&mut d, 12000, 0, 0);
    set_inherited_estate(&mut d, inherited_estate(12000));
    let mut gift = qcd(Owner::Taxpayer, 20260601, 5000, 1000);
    gift.payments[0].inherited = Some(2);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    assert_eq!(value(&supported(&d), "inherited_ira_2_estate_tax_deduction"), 1848);
    // A second source makes assignment of the shared adjustment consequential.
    add_inherited_ira(&mut d, 10000, 0, 0);
    let mut other = d.adjustments.as_ref().unwrap().qualified_charity[0].payments[0].clone();
    other.inherited = Some(4);
    d.adjustments.as_mut().unwrap().qualified_charity[0].payments.push(other);
    unsupported(&d, "Inherited estate tax");
}
#[test]
fn inherited_hsa_funding_preserves_group_basis_and_gross_income() {
    let d = inherited_hsa_funding(1600, 200, 2000, 400, 1500);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 160000);
    assert_eq!(return_line(&r, "4b"), 6000);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 40);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 160);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn inherited_hsa_funding_consumes_own_group_basis_before_later_withdrawals() {
    let mut d = inherited_hsa_funding(1600, 1000, 2000, 400, 1500);
    add_inherited_ira(&mut d, 1000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_3_20260115"), 500);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 400);
    assert_eq!(return_line(&r, "4b"), 100000);
    assert_eq!(value(&r, "inherited_ira_4_basis_recovered"), 0);
    assert_eq!(value(&r, "inherited_ira_2_form_line2"), 500);
    assert_eq!(value(&r, "inherited_ira_2_form_line7"), 100);
    assert_eq!(value(&r, "inherited_ira_2_form_line14"), 400);
    assert!(r.lines.iter().all(|row| !row.id.starts_with("inherited_ira_4_form_line")));
}
#[test]
fn inherited_hsa_funding_and_estate_deduction_exclude_the_same_transfer_once() {
    let mut d = inherited_hsa_funding(12000, 0, 12000, 0, 4000);
    set_inherited_estate(&mut d, inherited_estate(12000));
    assert_eq!(value(&supported(&d), "inherited_ira_2_estate_tax_deduction"), 1848);
}
#[test]
fn inherited_hsa_funding_rejects_predeath_unknown_basis_and_overlap() {
    let mut d = inherited_hsa_funding(4000, 1000, 5000, 1000, 4000);
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
        HsaFundingBasis::Pretax { distributed: 20260115 };
    assert!(crate::hsa_funding::source_reference_rule(&d).is_none());
    unsupported(&d, "HSA funding");
    let mut d = inherited_hsa_funding(4000, 1000, 5000, 1000, 4000);
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income {
        data.died = 20260201;
    }
    // This recorded source contradiction is rejected before tax calculation.
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&d),
        Some("HSA_FUNDING_BEFORE_INHERITED_DEATH")
    );
    assert!(!d.valid_stored());
    let before_invalid = d.clone();
    let invalid = estimate(&d);
    assert_eq!(invalid.state, ResultState::Incomplete);
    assert!(invalid.issues.iter().any(|issue| {
        issue.display.key == "ISSUE_ACCEPTED_DATA_IS_INVALID_NO_CALCULATION_WAS_PERFORMED_7A345C87"
    }));
    // Preserve the old unsupported helper's no-tax/no-refund/no-calculation guarantees.
    assert!(invalid.tax.is_none() && invalid.refund.is_none() && invalid.lines.is_empty());
    assert!(invalid.return_lines.is_empty());
    assert_eq!(d, before_invalid);
    assert_eq!(restart(d.clone()).draft, d);
    let mut d = inherited_hsa_funding(4000, 1000, 5000, 1000, 4000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    let mut gift = qcd(Owner::Taxpayer, 20260201, 1, 0);
    gift.payments[0].inherited = Some(2);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    assert!(crate::hsa_funding::source_reference_rule(&d).is_none());
    unsupported(&d, "QCD");
}
#[test]
fn inherited_roth_hsa_funding_uses_earnings_then_basis_without_recapture() {
    let mut d = inherited_hsa_funding(6000, 0, 6000, 0, 4000);
    let mut e = inherited_roth(2023, 3000, 2000);
    e.id = 2;
    d.income[1] = e;
    let mut payment = inherited_roth_payment(Owner::Taxpayer, Some(2), 6000);
    payment.id = d.income[2].id;
    if let FederalIncome::Roth { data } = &mut payment.income {
        let mut data = edit_roth(data);
        data.distributed = 20260115;
    }
    d.income[2] = payment;
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 600000, before_other_activity: true };
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_3_20260115"), 3000);
    assert_eq!(return_line(&r, "4a"), 600000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn inherited_hsa_second_transfer_uses_remaining_basis_and_source_order() {
    let mut d = inherited_hsa_funding(9000, 2000, 10000, 1000, 4000);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    let mut second = h.funding[0].clone();
    second.funded = 20260215;
    second.coverage = HsaCoverage::Family;
    second.basis = HsaFundingBasis::PriorBasis {
        distributed: 20260215,
        value: 500000,
        before_other_activity: true,
    };
    h.funding.push(second);
    // Actual first-day coverage changes after the January source transfer.
    h.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|month| if month == 0 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 50000);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 500);
    d.adjustments.as_mut().unwrap().hsa[0].funding.reverse();
    assert_eq!(supported(&d), r);
}

#[test]
fn noncapital_nondeductible_costs_share_at_risk_capacity_without_a_tax_deduction() {
    let mut source = sole_qbi_loss(18_500, 20_000, 18_500);
    source.nondeductible = 150_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.nondeductible_used.get()), 1_500);
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
            crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Ordinary].get())
        ),
        (9_250, 9_250)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.nondeductible_used.get()),
            crate::kernel::ratio::dollars(risk.nondeductible_suspended.get())
        ),
        (750, 750)
    );
    assert_eq!(risk.year_end, Some(0));
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 50_750);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 9_250);
    scorp_note(&r, id, "nondeductible expenses still suspended by at-risk", 750);
    scorp_note(&r, id, "net loss absorbed by at-risk including nondeductible expenses", 10_000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn simultaneous_basis_nondeductible_suspension_remains_explicit() {
    let mut source = sole_qbi_loss(18_500, 10_000, 18_500);
    source.nondeductible = 150_000;
    source.losses_first = true;
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    unsupported(&d, "S-corporation at-risk");
}

#[test]
fn ira_basis_exception_education_uses_taxable_amount_and_both_rates() {
    for (simple, penalty) in [(false, 1200), (true, 3000)] {
        let d = ira_basis_exception_return(education_withdrawal(4000, 0, false), simple);
        let r = supported(&d);
        // 20% basis: $4,000 recovered, $16,000 taxable. $4,000 education
        // shields taxable dollars; it does not allocate only 80% of expenses.
        assert_eq!(return_line(&r, "4a"), 2_000_000);
        assert_eq!(return_line(&r, "4b"), 1_600_000);
        assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 16000);
        assert_eq!(value(&r, "early_taxpayer_exceptions"), 4000);
        assert_eq!(value(&r, "early_distribution_tax"), penalty);
        // $136,000 AGI - $16,100 deduction = $119,900 taxable income.
        // 2026 single rate schedule: $17,966 + 24% of $14,200.
        assert_eq!(r.tax, Some((21374 + penalty) * 100));
        assert_eq!(supported(&restart(d).draft).tax, r.tax);
    }
}

#[test]
fn ira_basis_exception_full_exemptions_preserve_ordinary_income() {
    for facts in [
        RetirementException::Disability { began: 20260101 },
        RetirementException::TerminalIllness { certified: 20260101 },
        RetirementException::Reservist {
            ordered: 20260101,
            through: 20261231,
            term: ReserveTerm::Indefinite,
            eligible: true,
        },
        RetirementException::Levy { amount: 2_000_000 },
    ] {
        let d = ira_basis_exception_return(facts, false);
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(value(&r, "early_taxpayer_exceptions"), 16000);
        assert_eq!(return_line(&r, "4b"), 1_600_000);
        assert_eq!(r.tax, Some(2_137_400));
    }
}

#[test]
fn ira_basis_exception_medical_uses_final_agi_and_insurance_uses_premiums() {
    for facts in [
        RetirementException::Medical { expenses: 1_420_000 },
        RetirementException::UnemploymentInsurance {
            premiums: 400_000,
            benefit_year: 2026,
            weeks: 12,
            qualification: UnemploymentQualification::Received { confirmed: true },
            reemployment: Reemployment::None,
        },
    ] {
        let d = ira_basis_exception_return(facts, false);
        let r = supported(&d);
        // Medical: $14,200 - 7.5% of $136,000 = $4,000; insurance has no AGI floor.
        assert_eq!(value(&r, "agi"), 136000);
        assert_eq!(value(&r, "early_taxpayer_exceptions"), 4000);
        assert_eq!(value(&r, "early_distribution_tax"), 1200);
        assert_eq!(r.tax, Some(2_257_400));
    }
}

#[test]
fn ira_basis_exception_multiple_distributions_keep_allocation_and_order() {
    let mut d = ira_basis_exception_return(education_withdrawal(4000, 0, false), false);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, RetirementTreatment::Early));
    d.adjustments.as_mut().unwrap().ira_basis[0].year_end = 7000000;
    let r = supported(&d);
    // Same 20% basis share: 16,000 + 8,000 taxable; only 4,000 exempt.
    assert_eq!(return_line(&r, "4b"), 2400000);
    assert_eq!(value(&r, "early_distribution_tax"), 2000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 14000);
    assert_eq!(r.tax, Some(2_529_400)); // 23,294 regular + 2,000 additional.
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn ira_basis_exception_preserves_deceased_owner_and_unknown_fact_guards() {
    unsupported(
        &ira_basis_exception_return(RetirementException::Death { died: 20260101 }, false),
        "IRA basis",
    );
    let d = ira_basis_exception_return(RetirementException::Disability { began: 20260701 }, false);
    unsupported(&d, "Income");
}

#[test]
fn ira_gross_exception_caps_balance_and_actual_costs_before_basis() {
    for (facts, relief) in [
        (
            RetirementException::FirstHome {
                costs: 200000,
                paid_on: 20260630,
                prior_used: 0,
                eligible: true,
            },
            1600,
        ),
        (
            RetirementException::Emergency {
                balance: 150000,
                history: EmergencyHistory::NONE,
                eligible: true,
            },
            400,
        ),
        (
            RetirementException::Emergency {
                balance: 100000,
                history: EmergencyHistory::NONE,
                eligible: true,
            },
            0,
        ),
        (
            RetirementException::DomesticAbuse {
                plan: "Plan A".into(),
                event: 20260101,
                balance: 800000,
                prior_used: 0,
                prior_plan_used: 0,
                eligible: true,
            },
            3200,
        ),
        (gross_home_claim(10000), 0),
        (gross_birth_claim(5000), 0),
        (RetirementException::Levy { amount: 0 }, 0),
    ] {
        let gross = match &facts {
            RetirementException::Emergency { balance, .. }
            | RetirementException::DomesticAbuse { balance, .. } => (*balance).min(2000000),
            _ => 2000000,
        };
        let mut d = ira_basis_exception_return(facts, false);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        edit_retirement(data).gross = gross;
        d.adjustments.as_mut().unwrap().ira_basis[0].year_end = 10000000 - gross;
        let r = supported(&d);
        assert_eq!(value(&r, "early_taxpayer_exceptions"), relief);
        assert_eq!(value(&r, "early_distribution_tax"), (gross * 4 / 500 - relief) / 10);
    }
}

#[test]
fn ira_gross_exception_fully_recovered_basis_still_consumes_claimed_lifetime_limit() {
    let mut d = ira_basis_exception_return(gross_home_claim(0), false);
    d.adjustments.as_mut().unwrap().ira_basis[0].year_end = 0;
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "first_home_used_taxpayer"), 10000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
}

#[test]
fn reservist_repayment_adds_basis_outside_annual_limit_without_deduction() {
    let mut d = reservist_basis_return(20000, 20000);
    d.income[0] = wage(1, Owner::Taxpayer, 13000000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    d.adjustments.as_mut().unwrap().ira_basis[0].year_end += 750000;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "taxpayer_ira_reservist_basis"), 20000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 20000);
    assert_eq!(value(&r, "reservist_taxpayer_1_unrepaid"), 10000);
    assert_eq!(value(&r, "agi"), 122500);
    assert_eq!(r.tax, Some(1813400)); // $106,400 taxable: $17,966 + 24% of $700.
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn reservist_repayment_basis_recovers_proportionally_from_current_distribution() {
    let mut d = reservist_basis_return(20000, 80000);
    add(&mut d, basis_distribution(Owner::Taxpayer, 20000, RetirementTreatment::Early));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 1600000);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 4000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 16000);
    assert_eq!(value(&r, "early_distribution_tax"), 1600);
    assert_eq!(r.tax, Some(2297400));
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn reservist_repayments_aggregate_payments_and_preserve_spouse_basis() {
    let mut d = complete(FederalStatus::Joint);
    d.income.clear(); // No compensation requirement applies to a repayment.
    for (owner, amount) in [(Owner::Taxpayer, 3000), (Owner::Spouse, 5000)] {
        let mut b = prior_ira_basis(owner, 1000, amount + 1000);
        let mut p = reservist_repayment(amount);
        p.prior = 100000;
        p.payments[0].amount -= 100000;
        p.payments.push(ReservistPayment { date: 20260215, amount: 100000 });
        b.reservist_repayments.push(p);
        d.adjustments.as_mut().unwrap().ira_basis.push(b);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 4000);
    assert_eq!(value(&r, "spouse_ira_basis_carryforward"), 6000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    let b = &mut d.adjustments.as_mut().unwrap().ira_basis[0];
    b.reservist_repayments.push(b.reservist_repayments[0].clone());
    unsupported(&d, "Reservist IRA repayment");
}

#[test]
fn reservist_repayment_basis_participates_in_qcd_pretax_capacity() {
    let mut d = reservist_basis_return(10000, 0);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add(&mut d, basis_distribution(Owner::Taxpayer, 20000, RetirementTreatment::Ordinary));
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261215, 20000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 10000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 10000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
}

#[test]
fn reservist_repayment_dates_preserve_hsa_prior_basis_source_order() {
    let mut d = hsa_prior_basis_funding(1500, 200, 2000, 1500, 0);
    let mut p = reservist_repayment(1000);
    p.payments[0].date = 20260116;
    d.adjustments.as_mut().unwrap().ira_basis[0].reservist_repayments.push(p);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 1200);
    d.adjustments.as_mut().unwrap().ira_basis[0].reservist_repayments[0].payments[0].date =
        20260114;
    unsupported(&d, "HSA funding");
}

#[test]
fn reservist_repayment_basis_coordinates_with_income_limited_ira_deduction() {
    let mut d = ira_contribution_distribution(80000, Answer::Yes);
    d.adjustments.as_mut().unwrap().ira_basis[0]
        .reservist_repayments
        .push(reservist_repayment(2500));
    // Worksheet 1-1: ($2,500 repayment basis + $7,500 contribution) / $50,000
    // times $10,000 distribution = $2,000 preliminary tax-free recovery.
    // $88,000 deduction MAGI yields $2,250 deductible, $5,250 nondeductible.
    // Available basis $7,750 covers that recovery; $5,750 remains.
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 2250);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 5250);
    assert_eq!(value(&r, "taxpayer_ira_current_nondeductible_basis"), 7750);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 2000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 5750);
    assert_eq!(return_line(&r, "4b"), 800000);
    assert_eq!(value(&r, "agi"), 85750);
    assert_eq!(value(&r, "early_distribution_tax"), 800);
}

#[test]
fn special_ira_repayment_terminal_illness_preserves_basis_timing() {
    let mut d = ira_basis_exception_return(
        RetirementException::TerminalIllness { certified: 20260101 },
        false,
    );
    let b = &mut d.adjustments.as_mut().unwrap().ira_basis[0];
    b.year_end += 2000000;
    b.special_repayments.push(special_ira_repayment(2, 20000, 20261201));
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_repayment_year_end"), 80000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 16000);
    assert_eq!(next_year_basis_notes(&r, "NOTE_LINE_15C_NEXT_YEAR_BASIS"), vec![4000]);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(r.tax, Some(1757000));
}

#[test]
fn special_ira_repayment_owners_keep_separate_income_and_future_basis() {
    let mut d = complete(FederalStatus::Joint);
    for (owner, paid) in [(Owner::Taxpayer, 3000), (Owner::Spouse, 5000)] {
        let mut entry = basis_distribution(
            owner,
            5000,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260601,
                facts: gross_birth_claim(0),
                early_simple: false,
            },
        );
        entry.id = d.next_id;
        let id = entry.id;
        add(&mut d, entry);
        let mut b = prior_ira_basis(owner, 1000, paid);
        b.special_repayments.push(special_ira_repayment(id, paid, 20261201));
        d.adjustments.as_mut().unwrap().ira_basis.push(b);
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 100000);
    assert_eq!(value(&r, "taxpayer_ira_repayment_income_reduction"), 3000);
    assert_eq!(value(&r, "spouse_ira_repayment_income_reduction"), 4000);
    // Only the spouse's repayment exceeds the taxable amount.
    assert_eq!(next_year_basis_notes(&r, "NOTE_LINE_15C_NEXT_YEAR_BASIS"), vec![1000]);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn care_distribution_does_not_reuse_ordinary_partial_basis_allocation() {
    let d = plan_gross_exception(care_claim(3000, 100000, false), 20000);
    unsupported(&d, "Long-term-care distribution basis");
    let mut d = plan_gross_exception(care_claim(3000, 100000, false), 0);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 2000000);
    assert_eq!(value(&r, "early_distribution_tax"), 1740);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).taxable = Some(RetirementTaxable::FullDischarge {
        data: crate::ContractDischarge { investment: 400000, eligible: true },
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 1600000);
    assert_eq!(value(&r, "early_distribution_tax"), 1340);
    assert_eq!(r.tax, Some(2271400));
}

#[test]
fn employee_pension_combines_with_prior_ira_basis_without_changing_it() {
    let mut d = ira_contribution_distribution(80000, Answer::No);
    let a = d.adjustments.as_mut().unwrap();
    a.iras = FederalIras::NONE;
    a.ira_basis[0] = prior_ira_basis(Owner::Taxpayer, 5000, 40000);
    a.pension_contributions = vec![pension_contribution(Owner::Taxpayer, 7000, 80000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 1000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 4000);
    assert_eq!(value(&r, "agi"), 82000);
    assert_eq!(schedule_1_line(&r, "24f"), 7000);
    assert_eq!(schedule_1_line(&r, "26"), 7000);
    assert_eq!(value(&r, "early_distribution_tax"), 900);
    assert_eq!(r.tax, Some(1011600));
}

#[test]
fn inherited_nua_sale_preserves_ird_and_uses_estate_basis_and_long_term() {
    // Rev. Rul. 75-125: distribution 10x, NUA 5x, death value 12x,
    // sale 13x => basis 7x, NUA gain 5x, other long-term gain 1x.
    for scale in [10000, 5000] {
        for (proceeds, gain, nua, niit, carry) in
            [(13, 6, 5, 380, 0), (10, 3, 3, 0, 0), (6, -1, 0, 0, 7000)]
        {
            let lot = NuaSale {
                distributed: 20260101,
                sold: 20260924,
                distribution_value: 10 * scale * 100,
                deferred_nua: 5 * scale * 100,
                inheritance: Some(NuaInheritance {
                    died: 20260630,
                    value: 12 * scale * 100,
                    estate_tax: false,
                }),
                proceeds: proceeds * scale * 100,
                wash_sale: 0,
                withholding: 0,
                reported_basis: None,
                reporting: crate::BasisReporting::ReportedToIrs,
                special: false,
            };
            assert!(lot.valid());
            assert!(lot.confirmed());
            assert_eq!(lot.basis(), 7 * scale * 100);
            assert_eq!(lot.realized_nua(), nua * scale * 100);
            assert!(lot.parts().iter().all(|v| v.term == GainTerm::Long));
            let mut d = complete(FederalStatus::Single);
            d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
            add(&mut d, income(FederalIncome::NuaSale { data: lot }));
            let r = supported(&d);
            assert_eq!(value(&r, "capital_gain"), (gain * scale).max(-3000));
            assert_eq!(value(&r, "amt_capital_gain"), (gain * scale).max(-3000));
            assert_eq!(value(&r, "net_investment_income_tax"), niit * scale / 10000);
            assert_eq!(
                value(&r, "capital_long_carryforward"),
                if carry == 0 { 0 } else { scale - 3000 }
            );
            assert_eq!(value(&r, "capital_short_carryforward"), 0);
            assert_eq!(value(&r, "agi"), 300000 + (gain * scale).max(-3000));
            assert_eq!(supported(&restart(d).draft), r);
        }
    }
}

#[test]
fn inherited_nua_requires_chronology_and_resolved_estate_treatment() {
    let mut lot = NuaSale {
        distributed: 20260101,
        sold: 20260924,
        distribution_value: 10000000,
        deferred_nua: 5000000,
        inheritance: Some(NuaInheritance { died: 20260630, value: 12000000, estate_tax: false }),
        proceeds: 13000000,
        wash_sale: 0,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        special: false,
    };
    for died in [20251231, 20261001] {
        lot.inheritance.as_mut().unwrap().died = died;
        assert!(!lot.valid());
    }
    lot.inheritance.as_mut().unwrap().died = 20260630;
    // Yes retains the named notice for the uncomputed section 691(c) deduction.
    lot.inheritance.as_mut().unwrap().estate_tax = true;
    assert!(lot.valid());
    assert!(lot.confirmed());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::NuaSale { data: lot.clone() }));
    assert!(noted(&supported(&d), "WARN_NUA_ESTATE_TAX_LEFT_OUT"));
    assert!(noted(&supported(&restart(d).draft), "WARN_NUA_ESTATE_TAX_LEFT_OUT"));
    lot.inheritance.as_mut().unwrap().estate_tax = false;
    lot.inheritance.as_mut().unwrap().value = 4999999;
    assert!(!lot.confirmed());
}

#[test]
fn inherited_nua_uses_reconciled_alternate_estate_value_without_resetting_nua() {
    // Section 1014(a)(2) substitutes the estate's valid section 2032 value.
    // These facts come from the executor, not an election made on Form 1040.
    // No estate/GST tax is allocated to this beneficiary's NUA in these fixtures.
    for (estate_value, basis, total_gain, nii_tax) in
        [(80000, 30000, 100000, 1900), (110000, 60000, 70000, 760), (130000, 80000, 50000, 0)]
    {
        let lot = NuaSale {
            distributed: 20240101,
            sold: 20260924,
            distribution_value: 10000000,
            deferred_nua: 5000000,
            inheritance: Some(NuaInheritance {
                died: 20251231,
                value: estate_value * 100,
                estate_tax: false,
            }),
            proceeds: 13000000,
            wash_sale: 0,
            withholding: 0,
            reported_basis: None,
            reporting: crate::BasisReporting::ReportedToIrs,
            special: false,
        };
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
        add(&mut d, income(FederalIncome::NuaSale { data: lot }));
        let id = d.income.last().unwrap().id;
        let r = supported(&d);
        assert_eq!(value(&r, &alloc::format!("nua_sale_{id}_basis")), basis);
        assert_eq!(value(&r, &alloc::format!("nua_sale_{id}_realized_nua")), 50000);
        assert_eq!(value(&r, "capital_gain"), total_gain);
        assert_eq!(value(&r, "amt_capital_gain"), total_gain);
        assert_eq!(value(&r, "agi"), 300000 + total_gain);
        assert_eq!(value(&r, "net_investment_income_tax"), nii_tax);
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn nonqualified_annuity_loans_increase_basis_without_tax_free_basis_recovery() {
    // Section 72(e)(4)(A) and Pub. 575: income-first, then increase
    // investment by taxable income; repayments do not add investment again.
    for (cash, investment, taxable, remaining) in
        [(16000, 10000, 6000, 16000), (30000, 10000, 7000, 17000), (10000, 10000, 0, 10000)]
    {
        let mut d = deferred_withdrawal(cash, investment);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        {
            let mut retirement_edit = edit_retirement(data);
            let Some(RetirementTaxable::DeferredWithdrawal {
                data: crate::AnnuityDistribution { loan_pledge, .. },
            }) = &mut retirement_edit.taxable
            else {
                unreachable!()
            };
            *loan_pledge = true;
        }
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), 700000);
        assert_eq!(return_line(&r, "5b"), taxable * 100);
        assert_eq!(value(&r, "annuity_2_basis_recovery"), 0);
        assert_eq!(value(&r, "annuity_2_basis_increase"), taxable);
        assert_eq!(value(&r, "annuity_2_remaining_basis"), remaining);
        assert_eq!(value(&r, "early_distribution_tax"), taxable / 10);
        assert_eq!(value(&r, "net_investment_income_tax"), taxable * 38 / 1000);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        if cash == 16000 {
            // After repayment, insurer records retain $16k basis, not $23k.
            let mut next = d.income[1].clone();
            let FederalIncome::Retirement { data } = &mut next.income else { unreachable!() };
            edit_retirement(data).taxable = Some(RetirementTaxable::DeferredWithdrawal {
                data: crate::AnnuityDistribution {
                    loan_pledge: false,
                    value: 1600000,
                    investment: 1600000,
                    related: vec![],
                    eligible: true,
                },
            });
            add(&mut d, next);
            let id = d.income.last().unwrap().id;
            let combined = supported(&d);
            assert_eq!(return_line(&combined, "5b"), 600000);
            assert_eq!(value(&combined, &alloc::format!("annuity_{id}_remaining_basis")), 9000);
            assert_eq!(supported(&restart(d.clone()).draft), combined);
        }
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Rollover {
            destination: RolloverDestination::Ira,
            amount: 700000,
            early: false,
        });
        assert_eq!(estimate(&d).state, ResultState::Unsupported);
    }
    let mut d = deferred_withdrawal(9000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTaxable::DeferredWithdrawal {
            data: crate::AnnuityDistribution { loan_pledge, related, .. },
        }) = &mut retirement_edit.taxable
        else {
            unreachable!()
        };
        *loan_pledge = true;
        related.push(AnnuityBalance { value: 1500000, investment: 1000000 });
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 400000);
    assert_eq!(value(&r, "annuity_2_basis_recovery"), 0);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 24000);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn historical_pension_basis_uses_its_worksheet_year() {
    let mut p = pension_basis();
    p.started = 20230701;
    p.months = 6;
    assert!(p.valid_for_year(2023));
    assert!(!p.valid()); // A 2023 start cannot be new on the 2026 worksheet.
    assert!(!p.valid_for_year(2022));
    p.months = 7;
    assert!(!p.valid_for_year(2023));
    p.months = 12;
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 60000 };
    assert!(p.valid_for_year(2024));
    assert!(p.valid());
    assert!(!p.valid_for_year(2023));
    assert_eq!(p.excluded(2400000), 120000);
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 3050000 };
    assert_eq!(p.excluded(2400000), 50000);
    assert!(!p.valid_for_year(u32::MAX));
}

#[test]
fn railroad_repayment_rejects_wrong_recipient_and_basis_allocation() {
    let mut d = railroad_repayment_draft();
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        unreachable!()
    };
    let RepaymentRetirement::Railroad { owner, .. } = &mut years[0].retirement[0] else {
        unreachable!()
    };
    *owner = Owner::Spouse;
    unsupported(&d, "Repayment railroad");
    let mut d = railroad_repayment_draft();
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTaxable::Railroad { repayments, .. }) = &mut retirement_edit.taxable
        else {
            unreachable!()
        };
        repayments[0].contributory = true;
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            unreachable!()
        };
        let RepaymentRetirement::Railroad { contributory, basis, .. } = &mut years[0].retirement[0]
        else {
            unreachable!()
        };
        *contributory = 1500000;
        *basis = Some(PensionBasis {
            earlier_months: 0,
            changes: vec![],
            share: None,
            started: 20000101,
            cost: Some(1000000),
            months: 12,
            history: PensionHistory::Prior { monthly: 1000, recovered: 0 },
            eligible: true,
        });
    }
    unsupported(&d, "Repayment railroad");
}

// Derived source-identity fixture, Notice 2008-51 pp. 4–5:
// https://www.irs.gov/pub/irs-drop/n-08-51.pdf . The direct transfer names
// one actual 1099-R; its beneficiary/decedent record owns annual basis.
// Neutral completion: eligible single owner, funding before other activity.
#[test]
fn inherited_hsa_funding_caps_actual_parent_and_preserves_group_projection() {
    let mut d = inherited_hsa_funding(1000, 0, 6000, 0, 4000);
    add(&mut d, inherited_payment(Owner::Taxpayer, Some(2), 5000));
    unsupported(&d, "HSA funding"); // Group gross6000 cannot enlarge parent3 gross1000.
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].income = 4;
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 600000);
    assert_eq!(return_line(&r, "4b"), 200000);
    let mut detail = FederalResult::empty(d.revision);
    let funding = crate::hsa_funding::calculate(&d, &mut detail);
    assert_eq!(funding.for_entry(3), 0);
    assert_eq!(funding.for_entry(4), 400000);
    assert_eq!(funding.for_entry(2), 0);
    assert_eq!(funding.for_inherited(2), 400000);
    assert_eq!(
        funding.excluded.values().sum::<i64>(),
        funding.inherited_excluded.values().sum::<i64>()
    );
    let mut reordered = d.clone();
    reordered.income.reverse();
    assert_eq!(return_line(&supported(&reordered), "4b"), 200000);
    assert_eq!(restart(d.clone()).draft, d);
    // Both permitted transfers also share their selected parent's actual gross.
    let mut repeated = inherited_hsa_funding(4000, 0, 12000, 0, 4000);
    add(&mut repeated, inherited_payment(Owner::Taxpayer, Some(2), 8000));
    let account = &mut repeated.adjustments.as_mut().unwrap().hsa[0];
    account.funding[0].income = 4;
    let mut second = account.funding[0].clone();
    second.funded = 20260215;
    second.coverage = HsaCoverage::Family;
    second.amount = 450000;
    second.basis = HsaFundingBasis::PriorBasis {
        distributed: 20260215,
        value: 1200000,
        before_other_activity: true,
    };
    account.funding.push(second);
    account.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|month| if month == 0 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    assert!(crate::hsa_funding::source_reference_rule(&repeated).is_none());
    unsupported(&repeated, "HSA funding"); // 8500 <= annual8750 and group12000, but > parent8000.
    // Annual group identities and dangling/mismatched groups are not payments.
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].income = 2;
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&d),
        Some("HSA_FUNDING_SOURCE_NOT_OFFERED")
    );
    assert!(!d.valid_stored());
}

#[test]
fn inherited_hsa_payment_keeps_qcd_and_estate_group_footing_once() {
    let mut d = inherited_hsa_funding(4000, 0, 12000, 0, 4000);
    add(&mut d, inherited_payment(Owner::Taxpayer, Some(2), 8000));
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    let mut gift = qcd(Owner::Taxpayer, 20260601, 3000, 0);
    gift.payments[0].inherited = Some(2);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    set_inherited_estate(&mut d, inherited_estate(12000));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 1200000);
    assert_eq!(return_line(&r, "4b"), 500000);
    assert_eq!(value(&r, "inherited_ira_2_estate_tax_deduction"), 1155);
}

// Derived from Notice 2008-51 pp. 2–3 and final TD 8816 A-4/A-5/A-8/A-9
// (https://www.irs.gov/pub/irs-drop/n-08-51.pdf,
// https://www.irs.gov/pub/irs-regs/td8816.pdf). These are combined source
// applications with neutral complete-return facts, not published IRS examples.
#[test]
fn roth_earlier_ordinary_funding_preserves_dated_basis_and_annual_income() {
    let mut d = roth_earlier_funding();
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260616"), 400);
    assert_eq!(value(&r, "roth_3_regular_recovered"), 300);
    assert_eq!(value(&r, "roth_4_regular_recovered"), 100);
    assert_eq!(value(&r, "roth_3_taxable"), 0);
    assert_eq!(value(&r, "roth_4_taxable"), 100);
    assert_eq!(return_line(&r, "4a"), 110000);
    assert_eq!(return_line(&r, "4b"), 10000);
    assert_eq!(value(&r, "early_distribution_tax"), 10);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d), r);
}

// Derived same-day source applications of Notice2008-51 taxable-first and
// TD8816's nonqualified Roth ordering, not published combined IRS examples.
// Opening800; ordinary300 before funding600 at value700 leaves500 basis,
// so funding uses400 and laterordinary200 taxes100. Funding before ordinary
// consumes600 basis, leaving200 for ordinary500, which therefore taxes300.
#[test]
fn roth_same_day_funding_uses_actual_order_and_never_entry_order() {
    for (funding_order, ordinary_order, funding_basis, ordinary_tax) in
        [(2, 1, 40000, 10000), (1, 2, 60000, 30000)]
    {
        let mut d = roth_earlier_funding();
        for (index, sequence) in [(1, funding_order), (2, ordinary_order)] {
            let FederalIncome::Roth { data } = &mut d.income[index].income else { unreachable!() };
            let mut data = edit_roth(data);
            data.distributed = 20260615;
            data.sequence = Some(sequence);
        }
        assert_eq!(crate::roth::funding_first_consistent(&d, 2), funding_order < ordinary_order);
        let ordered = crate::roth::earlier_funding(&d, Owner::Taxpayer).unwrap();
        assert_eq!(ordered.basis[&2], funding_basis);
        let r = supported(&d);
        assert_eq!(return_line(&r, "4b"), ordinary_tax);
        assert_eq!(value(&r, "early_distribution_tax") * 100, ordinary_tax / 10);
        assert_eq!(restart(d.clone()).draft, d);
        d.income.reverse();
        assert_eq!(supported(&d), r);
        // Missing or equal source sequence never acquires meaning from IDs.
        for invalid in [None, Some(funding_order)] {
            let mut bad = d.clone();
            let FederalIncome::Roth { data } =
                &mut bad.income.iter_mut().find(|e| e.id == 3).unwrap().income
            else {
                unreachable!()
            };
            edit_roth(data).sequence = invalid;
            assert!(crate::roth::earlier_funding(&bad, Owner::Taxpayer).is_none());
            assert!(noted(&supported(&bad), "WARN_HSA_FUNDING_LEFT_OUT"));
        }
    }
}

#[test]
fn roth_earlier_funding_preserves_conversion_year_and_both_conversion_classes() {
    let mut d = roth_earlier_funding();
    let a = d.adjustments.as_mut().unwrap();
    a.roth_history[0].ordering = Some(RothOrdering {
        regular: 10000,
        conversions: vec![
            RothConversion { year: 2025, taxable: 20000, nontaxable: 10000 },
            RothConversion { year: 2020, taxable: 10000, nontaxable: 10000 },
        ],
    });
    a.hsa[0].funding[0].amount = 20000;
    a.hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 25000, before_other_activity: false };
    for (index, gross) in [(1, 20000), (2, 35000), (3, 10000)] {
        if let FederalIncome::Roth { data } = &mut d.income[index].income {
            edit_roth(data).gross = gross;
        }
    }
    // Before funding: regular0, 2020 cohort0, 2025 taxable150/non100.
    // Funding uses 2025 taxable150/non50, leaving 2025 non50.
    // Ordinary350+100 uses regular100/old200/recent100/earnings50;
    // only recent taxable50 recaptures, plus earnings50: $10 tax.
    let projected = crate::roth::earlier_funding(&d, Owner::Taxpayer).unwrap();
    assert_eq!(projected.consumed.regular, 0);
    assert_eq!(
        projected.consumed.conversions,
        vec![
            RothConversion { year: 2020, taxable: 0, nontaxable: 0 },
            RothConversion { year: 2025, taxable: 15000, nontaxable: 5000 },
        ]
    );
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260616"), 200);
    assert_eq!(value(&r, "roth_3_conversion_recapture"), 50);
    assert_eq!(value(&r, "roth_4_conversion_recapture"), 0);
    assert_eq!(return_line(&r, "4b"), 5000);
    assert_eq!(value(&r, "early_distribution_tax"), 10);
}

#[test]
fn roth_earlier_funding_two_transfers_use_intervening_withdrawals_and_market_values() {
    let mut d = roth_earlier_funding();
    for (index, date, gross) in [(1, 20260215, 40000), (2, 20260115, 10000), (3, 20260315, 20000)] {
        if let FederalIncome::Roth { data } = &mut d.income[index].income {
            let mut data = edit_roth(data);
            data.distributed = date;
            data.gross = gross;
        }
    }
    let mut second = d.adjustments.as_ref().unwrap().hsa[0].funding[0].clone();
    second.income = d.next_id;
    second.funded = 20260416;
    second.amount = 60000;
    second.coverage = HsaCoverage::Family;
    second.basis = HsaFundingBasis::Roth { value: 80000, before_other_activity: false };
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 600, 20260415));
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 300, 20260915));
    for entry in &mut d.income {
        if let FederalIncome::Roth { data } = &mut entry.income {
            let mut data = edit_roth(data);
            data.withholding = 0;
        }
    }
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.funding[0].funded = 20260216;
    h.funding[0].amount = 40000;
    h.funding[0].basis = HsaFundingBasis::Roth { value: 90000, before_other_activity: false };
    h.funding.push(second);
    h.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|i| if i < 2 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260216"), 200);
    assert_eq!(value(&r, "hsa_funding_basis_5_20260416"), 100);
    assert_eq!(return_line(&r, "4b"), 10000);
    assert_eq!(value(&r, "early_distribution_tax"), 10);
    d.adjustments.as_mut().unwrap().hsa[0].funding.reverse();
    assert_eq!(supported(&d), r);
    // With annual regular400, opening800 gives the deliberately sufficient
    // basis bound1200 at both events. Each value must cover its own transfer.
    let mut incoming = d.clone();
    incoming.adjustments.as_mut().unwrap().iras = ira_facts(0, 40000, Answer::No);
    for transfer in &mut incoming.adjustments.as_mut().unwrap().hsa[0].funding {
        transfer.basis =
            HsaFundingBasis::Roth { value: 120000 + transfer.amount, before_other_activity: false };
    }
    let ordered = crate::roth::earlier_funding(&incoming, Owner::Taxpayer).unwrap();
    assert_eq!(ordered.basis.len(), 2);
    assert!(ordered.basis.values().all(|basis| *basis == 0));
    assert_eq!(return_line(&supported(&incoming), "4b"), 0);
    incoming.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 179999, before_other_activity: false };
    assert!(crate::roth::earlier_funding(&incoming, Owner::Taxpayer).is_none());
    // The complete projected class set must not reach the annual owner if a
    // later transfer fails actual source/owner eligibility after projection.
    let second =
        d.adjustments.as_mut().unwrap().hsa[0].funding.iter_mut().find(|t| t.income == 5).unwrap();
    second.loss = Some(HsaEligibilityLoss { date: 20260417, reason: HsaLossReason::Death });
    assert!(crate::roth::earlier_funding(&d, Owner::Taxpayer).is_some());
    let mut validation = FederalResult::empty(d.revision);
    let funding = crate::hsa_funding::calculate(&d, &mut validation);
    assert_eq!(funding.for_entry(2), 40000);
    assert_eq!(funding.for_entry(5), 0);
    assert!(funding.roth_ordered_basis[0].is_none());
    assert!(!validation.issues.is_empty());
}

#[test]
fn roth_earlier_funding_regular_contributions_preserve_basis_when_earnings_cover_funding() {
    // Derived sufficient endpoint, not a published combined IRS example:
    // opening regular800 + current regular400 bounds basis by1200. At a
    // funding value1800, all600 funding is earnings under either timing
    // reading. Ordinary300+1100 recovers1200 and taxes200, with20 early tax.
    let mut d = roth_earlier_funding();
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 40000, Answer::No);
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 180000, before_other_activity: false };
    if let FederalIncome::Roth { data } = &mut d.income[3].income {
        let mut data = edit_roth(data);
        data.gross = 110000;
    }
    let ordered = crate::roth::earlier_funding(&d, Owner::Taxpayer).unwrap();
    assert_eq!(ordered.basis.get(&2), Some(&0));
    assert_eq!(ordered.consumed.regular, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260616"), 0);
    assert_eq!(value(&r, "roth_3_regular_recovered"), 300);
    assert_eq!(value(&r, "roth_4_regular_recovered"), 900);
    assert_eq!(value(&r, "roth_4_taxable"), 200);
    assert_eq!(return_line(&r, "4a"), 200000);
    assert_eq!(return_line(&r, "4b"), 20000);
    assert_eq!(value(&r, "early_distribution_tax"), 20);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d), r);
    // The actual2027 amount is already a subset of retained regular400.
    d.adjustments.as_mut().unwrap().iras.taxpayer.next_year_deposits =
        Some(next_year_ira_deposits(0, 40000, 20270331));
    let next_year = supported(&d);
    assert_eq!(return_line(&next_year, "4b"), 20000);
    assert_eq!(value(&next_year, "early_distribution_tax"), 20);
    assert_eq!(value(&next_year, "hsa_funding_basis_2_20260616"), 0);
    // One cent below the sufficient bound remains unverified, even though
    // some deposit timings could make that transfer use only earnings.
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 179999, before_other_activity: false };
    assert!(crate::roth::earlier_funding(&d, Owner::Taxpayer).is_none());
    let mut notes = vec![];
    let working = crate::unknowns::supported(&d, &mut notes).unwrap();
    assert!(working.adjustments.as_ref().unwrap().hsa[0].funding.is_empty());
    assert!(notes.iter().any(|n| n.display.key == "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn roth_earlier_funding_bound_includes_the_annual_owners_absorbed_prior_excess() {
    let mut d = roth_earlier_funding();
    d.adjustments.as_mut().unwrap().iras.taxpayer.excess = Some(crate::IraExcess {
        traditional_value: Some(0),
        roth_value: 90000,
        prior_traditional: 0,
        prior_roth: 120000,
        returned_prior_excess: 0,
    });
    // Gross payments1100 leave100 of carried excess potentially absorbed;
    // only the IRA owner decides its final MAGI-dependent amount. The funding
    // bound must cover opening800 + that upper bound100 + actual transfer600.
    assert_eq!(crate::ira::provisional_roth_absorbed(&d), [10000, 0]);
    assert!(crate::roth::earlier_funding(&d, Owner::Taxpayer).is_none());
    for (value, supported_bound) in [(149999, false), (150000, true)] {
        d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
            HsaFundingBasis::Roth { value, before_other_activity: false };
        let ordered = crate::roth::earlier_funding(&d, Owner::Taxpayer);
        assert_eq!(ordered.is_some(), supported_bound);
        if let Some(ordered) = ordered {
            assert_eq!(ordered.basis[&2], 0);
            assert_eq!(return_line(&supported(&d), "4b"), 0);
        }
    }
}

#[test]
fn roth_earlier_funding_regular_contributions_keep_conversion_recapture_classes() {
    // Opening regular100, old conversion100taxable/100nontaxable, recent
    // conversion200taxable/100nontaxable; current regular100 => basis<=700.
    // Value900/funding200 guarantees no basis use. Ordinary350+500 recovers
    // regular200 + old200 + recent300, leaving150 earnings. Recent200 plus
    // earnings150 yields35 early tax; funding itself has no recapture.
    let mut d = roth_earlier_funding();
    let a = d.adjustments.as_mut().unwrap();
    a.iras = ira_facts(0, 10000, Answer::No);
    a.roth_history[0].ordering = Some(RothOrdering {
        regular: 10000,
        conversions: vec![
            RothConversion { year: 2025, taxable: 20000, nontaxable: 10000 },
            RothConversion { year: 2020, taxable: 10000, nontaxable: 10000 },
        ],
    });
    a.hsa[0].funding[0].amount = 20000;
    a.hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 90000, before_other_activity: false };
    for (index, gross) in [(1, 20000), (2, 35000), (3, 50000)] {
        if let FederalIncome::Roth { data } = &mut d.income[index].income {
            edit_roth(data).gross = gross;
        }
    }
    let ordered = crate::roth::earlier_funding(&d, Owner::Taxpayer).unwrap();
    assert_eq!(ordered.basis.get(&2), Some(&0));
    assert_eq!(ordered.consumed.regular, 0);
    assert!(ordered.consumed.conversions.iter().all(|c| c.taxable == 0 && c.nontaxable == 0));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 15000);
    assert_eq!(value(&r, "roth_4_conversion_recapture"), 200);
    assert_eq!(value(&r, "early_distribution_tax"), 35);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260616"), 0);
}

#[test]
fn roth_earlier_funding_does_not_admit_unverified_incoming_basis_or_ambiguous_source_dates() {
    for change in 0..11 {
        let mut d = roth_earlier_funding();
        match change {
            0 => d.adjustments.as_mut().unwrap().iras = ira_facts(0, 100, Answer::No),
            1 => add(
                &mut d,
                basis_distribution(
                    Owner::Taxpayer,
                    1,
                    RetirementTreatment::Conversion { amount: 100, early: false },
                ),
            ),
            2 => {
                if let FederalIncome::Roth { data } = &mut d.income[2].income {
                    let mut data = edit_roth(data);
                    data.distributed = 20260615;
                }
            }
            3 => {
                if let FederalIncome::Roth { data } = &mut d.income[1].income {
                    let mut data = edit_roth(data);
                    data.gross += 100;
                }
            }
            4 => {
                if let FederalIncome::Roth { data } = &mut d.income[2].income {
                    let Some(DistributionCode::RothIra { roth, .. }) = &mut data.code else {
                        unreachable!()
                    };
                    *roth = RothIraCode::Qualified;
                    data.form.box7 = crate::DistributionCodes::CODE_Q.0;
                }
            }
            5 => d.adjustments.as_mut().unwrap().roth_history[0].ordering = None,
            6 => {
                if let FederalIncome::Roth { data } = &mut d.income[1].income {
                    let mut data = edit_roth(data);
                    data.rolled_over = 1;
                }
            }
            7 => {
                let mut qtp = qtp_roth_rollover();
                qtp.rollover.confirmed = false;
                set_qtp_roth(&mut d, Owner::Taxpayer, &[qtp]);
            }
            8 => {
                if let FederalIncome::Roth { data } = &mut d.income[2].income {
                    let mut data = edit_roth(data);
                    data.rolled_over = 1;
                }
            }
            9 => {
                if let FederalIncome::Roth { data } = &mut d.income[1].income {
                    let mut data = edit_roth(data);
                    data.exception = Some(RetirementException::Disability { began: 20260101 });
                }
            }
            10 => {
                let mut employer = basis_distribution(
                    Owner::Taxpayer,
                    1,
                    RetirementTreatment::EmployerRothContribution,
                );
                let FederalIncome::Retirement { data } = &mut employer.income else {
                    unreachable!()
                };
                edit_retirement(data).box_2a = 100;
                edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
                edit_retirement(data).box7 = DistributionCodes::CODE_2.0;
                add(&mut d, employer);
            }
            _ => unreachable!(),
        }
        assert!(crate::roth::earlier_funding(&d, Owner::Taxpayer).is_none(), "change{change}");
        let mut notes = vec![];
        let working = crate::unknowns::supported(&d, &mut notes).unwrap();
        assert!(working.adjustments.as_ref().unwrap().hsa[0].funding.is_empty(), "change{change}");
        assert!(notes.iter().any(|n| n.display.key == "WARN_HSA_FUNDING_LEFT_OUT"));
    }
}

#[test]
fn roth_earlier_funding_keeps_inherited_roth_pool_separate() {
    let mut d = roth_earlier_funding();
    // This beneficiary/decedent pool has no basis. It cannot consume
    // personal basis or change personal source ordering.
    add_inherited_roth(&mut d, 2022, 50, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260616"), 400);
    assert_eq!(return_line(&r, "4b"), 15000);
    assert_eq!(value(&r, "early_distribution_tax"), 10);
}

// Derived Notice 2008-51 taxable-first funding and Pub. 590-B beneficiary
// ordering controls. Each real payment has a date and shares only group2.
fn inherited_roth_earlier_draft() -> FederalDraft {
    let mut d = inherited_hsa_funding(600, 0, 700, 0, 600);
    let mut group = inherited_roth(2023, 800, 0);
    group.id = 2;
    d.income[1] = group;
    let mut funding = inherited_roth_payment(Owner::Taxpayer, Some(2), 600);
    funding.id = 3;
    if let FederalIncome::Roth { data } = &mut funding.income {
        let mut data = edit_roth(data);
        data.distributed = 20260615;
        data.withholding = 0;
    }
    d.income[2] = funding;
    let transfer = &mut d.adjustments.as_mut().unwrap().hsa[0].funding[0];
    transfer.funded = 20260616;
    transfer.basis = HsaFundingBasis::Roth { value: 70000, before_other_activity: false };
    for (gross, date) in [(300, 20260215), (200, 20260915)] {
        let mut payment = inherited_roth_payment(Owner::Taxpayer, Some(2), gross);
        if let FederalIncome::Roth { data } = &mut payment.income {
            let mut data = edit_roth(data);
            data.distributed = date;
            data.withholding = 0;
        }
        add(&mut d, payment);
    }
    d
}

#[test]
fn inherited_roth_same_day_funding_uses_its_actual_pool_order() {
    for (funding_order, ordinary_order, funding_basis, ordinary_tax) in
        [(2, 1, 40000, 10000), (1, 2, 60000, 30000)]
    {
        let mut d = inherited_roth_earlier_draft();
        for (index, sequence) in [(2, funding_order), (3, ordinary_order)] {
            let FederalIncome::Roth { data } = &mut d.income[index].income else { unreachable!() };
            let mut data = edit_roth(data);
            data.distributed = 20260615;
            data.sequence = Some(sequence);
        }
        let ordered = crate::roth::inherited_earlier_funding(&d, 2).unwrap();
        assert_eq!(ordered.basis[&3], funding_basis);
        let r = supported(&d);
        assert_eq!(return_line(&r, "4b"), ordinary_tax);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(restart(d.clone()).draft, d);
        d.income.reverse();
        assert_eq!(supported(&d), r);
        let FederalIncome::Roth { data } =
            &mut d.income.iter_mut().find(|e| e.id == 4).unwrap().income
        else {
            unreachable!()
        };
        edit_roth(data).sequence = Some(funding_order);
        assert!(crate::roth::inherited_earlier_funding(&d, 2).is_none());
        assert!(noted(&supported(&d), "WARN_HSA_FUNDING_LEFT_OUT"));
    }
}

#[test]
fn inherited_roth_earlier_funding_preserves_basis_income_and_restart() {
    // Opening800 - ordinary300 =500 at funding. Value700 has200 earnings;
    // funding600 uses400 regular basis, leaving100 for ordinary200.
    let d = inherited_roth_earlier_draft();
    let r = supported(&d);
    assert!(!noted(&r, "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(return_line(&r, "4a"), 110000);
    assert_eq!(return_line(&r, "4b"), 10000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "inherited_roth_2_regular_recovered"), 400);
    assert_eq!(value(&r, "inherited_roth_2_regular_remaining"), 0);
    assert_eq!(value(&r, "hsa_funding_basis_3_20260616"), 400);
    let mut detail = FederalResult::empty(d.revision);
    let funding = crate::hsa_funding::calculate(&d, &mut detail);
    assert_eq!(funding.for_entry(3), 60000);
    assert_eq!(funding.for_inherited(2), 60000);
    assert_eq!(funding.inherited_basis_used[&2], 40000);
    assert_eq!(funding.inherited_roth_ordered_basis[&2].regular, 40000);
    let mut reordered = d.clone();
    reordered.income.reverse();
    assert_eq!(supported(&reordered), r);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn inherited_roth_earlier_funding_removes_actual_conversion_class() {
    let mut d = inherited_roth_earlier_draft();
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income {
        data.kind = InheritedIraKindFacts::Roth {
            first_year: 2023,
            basis: Some(InheritedRothBasis { regular: 50000, converted: 30000 }),
        };
    }
    for (index, gross) in [(2, 30000), (3, 60000), (4, 5000)] {
        if let FederalIncome::Roth { data } = &mut d.income[index].income {
            edit_roth(data).gross = gross;
        }
    }
    let transfer = &mut d.adjustments.as_mut().unwrap().hsa[0].funding[0];
    transfer.amount = 30000;
    transfer.basis = HsaFundingBasis::Roth { value: 40000, before_other_activity: false };
    // Ordinary600 first uses regular500/conversion100. Funding300 at value400
    // uses earnings200/conversion100; later50 uses conversion, leaving50.
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "inherited_roth_2_regular_recovered"), 500);
    assert_eq!(value(&r, "inherited_roth_2_conversion_recovered"), 150);
    assert_eq!(value(&r, "inherited_roth_2_conversion_remaining"), 50);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    let mut detail = FederalResult::empty(d.revision);
    let funding = crate::hsa_funding::calculate(&d, &mut detail);
    assert_eq!(funding.inherited_roth_ordered_basis[&2].regular, 0);
    assert_eq!(funding.inherited_roth_ordered_basis[&2].converted, 10000);
}

#[test]
fn inherited_roth_earlier_funding_two_transfers_use_intervening_activity() {
    let mut d = inherited_roth_earlier_draft();
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income {
        data.kind = InheritedIraKindFacts::Roth {
            first_year: 2023,
            basis: Some(InheritedRothBasis { regular: 80000, converted: 40000 }),
        };
    }
    if let FederalIncome::Roth { data } = &mut d.income[3].income {
        let mut data = edit_roth(data);
        data.gross = 50000;
    }
    if let FederalIncome::Roth { data } = &mut d.income[4].income {
        let mut data = edit_roth(data);
        data.distributed = 20261215;
    }
    let mut between = inherited_roth_payment(Owner::Taxpayer, Some(2), 150);
    if let FederalIncome::Roth { data } = &mut between.income {
        let mut data = edit_roth(data);
        data.distributed = 20260815;
        data.withholding = 0;
    }
    add(&mut d, between);
    let second_id = d.next_id;
    let mut second = inherited_roth_payment(Owner::Taxpayer, Some(2), 400);
    if let FederalIncome::Roth { data } = &mut second.income {
        let mut data = edit_roth(data);
        data.distributed = 20261015;
        data.withholding = 0;
    }
    add(&mut d, second);
    let account = &mut d.adjustments.as_mut().unwrap().hsa[0];
    account.funding[0].basis = HsaFundingBasis::Roth { value: 90000, before_other_activity: false };
    let mut transfer = account.funding[0].clone();
    transfer.income = second_id;
    transfer.amount = 40000;
    transfer.funded = 20261016;
    transfer.coverage = HsaCoverage::Family;
    transfer.basis = HsaFundingBasis::Roth { value: 50000, before_other_activity: false };
    account.funding.push(transfer);
    // Actual first-day coverage matches the June SelfOnly/October Family events.
    account.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|month| if month < 6 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    // Basis1200 -> ordinary500 leaves700; F1 consumes400; ordinary150
    // leaves150; F2 consumes50; later200 recovers100 and taxes100.
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 185000);
    assert_eq!(return_line(&r, "4b"), 10000);
    assert_eq!(value(&r, "hsa_funding_basis_3_20260616"), 400);
    assert_eq!(value(&r, &alloc::format!("hsa_funding_basis_{second_id}_20261016")), 50);
    assert_eq!(value(&r, "inherited_roth_2_regular_recovered"), 500);
    assert_eq!(value(&r, "inherited_roth_2_conversion_recovered"), 250);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    let mut reordered = d.clone();
    reordered.income.reverse();
    reordered.adjustments.as_mut().unwrap().hsa[0].funding.reverse();
    assert_eq!(supported(&reordered), r);
}

#[test]
fn inherited_roth_earlier_funding_keeps_other_pools_and_same_day_ordinary_separate() {
    let mut d = inherited_roth_earlier_draft();
    // Two ordinary same-day payments commute; no funding shares that date.
    if let FederalIncome::Roth { data } = &mut d.income[3].income {
        let mut data = edit_roth(data);
        data.gross = 10000;
    }
    let mut extra = inherited_roth_payment(Owner::Taxpayer, Some(2), 200);
    if let FederalIncome::Roth { data } = &mut extra.income {
        let mut data = edit_roth(data);
        data.distributed = 20260215;
        data.withholding = 0;
    }
    add(&mut d, extra);
    let other = add_inherited_roth(&mut d, 2023, 1000, 1000, 0);
    d.adjustments.as_mut().unwrap().iras.taxpayer.roth = 10000;
    let ordered = crate::roth::inherited_earlier_funding(&d, 2).unwrap();
    assert_eq!(ordered.basis[&3], 40000);
    assert!(crate::roth::inherited_earlier_funding(&d, other).is_none());
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 10000);
    assert_eq!(value(&r, &alloc::format!("inherited_roth_{other}_regular_recovered")), 1000);
    let mut reversed = d.clone();
    reversed.income.reverse();
    let changed = supported(&reversed);
    let amounts = |r: &FederalResult| {
        r.lines
            .iter()
            .map(|line| (line.id.clone(), line.amount))
            .collect::<alloc::collections::BTreeMap<_, _>>()
    };
    assert_eq!(amounts(&changed), amounts(&r));
    assert_eq!(return_line(&changed, "4b"), 10000);
}

#[test]
fn inherited_roth_earlier_funding_leaves_ambiguous_or_missing_history_out() {
    let base = inherited_roth_earlier_draft();
    let mut cases = Vec::new();
    let mut same_day = base.clone();
    if let FederalIncome::Roth { data } = &mut same_day.income[3].income {
        let mut data = edit_roth(data);
        data.distributed = 20260615;
    }
    cases.push(same_day);
    let mut combined = base.clone();
    if let FederalIncome::Roth { data } = &mut combined.income[2].income {
        let mut data = edit_roth(data);
        data.gross += 100;
    }
    cases.push(combined);
    let mut missing = base.clone();
    if let FederalIncome::InheritedIra { data } = &mut missing.income[1].income {
        data.kind = InheritedIraKindFacts::Roth { first_year: 2023, basis: None };
    }
    cases.push(missing);
    let mut before_death = base.clone();
    if let FederalIncome::InheritedIra { data } = &mut before_death.income[1].income {
        data.died = 20260301;
    }
    cases.push(before_death);
    let mut qualified = base.clone();
    if let FederalIncome::InheritedIra { data } = &mut qualified.income[1].income {
        data.kind = InheritedIraKindFacts::Roth {
            first_year: 2020,
            basis: Some(InheritedRothBasis { regular: 80000, converted: 0 }),
        };
    }
    cases.push(qualified);
    for d in cases {
        assert!(crate::roth::inherited_earlier_funding(&d, 2).is_none());
        assert!(noted(&supported(&d), "WARN_HSA_FUNDING_LEFT_OUT"));
        assert_eq!(restart(d.clone()).draft, d);
    }
    // One cent changes the source valuation's taxable/basis split exactly.
    let mut one_cent = base;
    one_cent.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 69999, before_other_activity: false };
    let ordered = crate::roth::inherited_earlier_funding(&one_cent, 2).unwrap();
    assert_eq!(ordered.basis[&3], 40001);
    let result = supported(&one_cent);
    // 2026-10-19 (D-2026-10-19-12): the taxable amount that crosses is the printed Form 8606
    // line 25a in whole dollars (the exact 100.01 prints 100).
    assert_eq!(line_amount(&result, "inherited_roth_2_taxable"), 10000);
    assert_eq!(return_line(&result, "4b"), 10000); // Annual filing row rounds dollars.
}

#[test]
fn inherited_roth_earlier_funding_uses_the_beneficiary_owner() {
    let source = inherited_roth_earlier_draft();
    let mut d = complete(FederalStatus::Joint);
    d.income = source.income;
    d.adjustments = source.adjustments;
    d.next_id = source.next_id;
    for entry in &mut d.income {
        if matches!(entry.income, FederalIncome::Roth { .. } | FederalIncome::InheritedIra { .. }) {
            entry.owner = crate::DocumentOwner::Spouses;
        }
    }
    d.adjustments.as_mut().unwrap().hsa[0].owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: Some(HsaEligibility::FullYear { coverage: HsaCoverage::None }),
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 10000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert!(!noted(&r, "WARN_HSA_FUNDING_LEFT_OUT"));
    d.income.iter_mut().find(|e| e.id == 3).unwrap().owner = crate::DocumentOwner::Yours;
    assert!(crate::roth::inherited_earlier_funding(&d, 2).is_none());
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&d),
        Some("HSA_FUNDING_SOURCE_NOT_OFFERED")
    );
}

#[test]
fn roth_funding_first_confirmation_cannot_override_entered_ordinary_dates() {
    for mut d in [inherited_roth_earlier_draft(), roth_earlier_funding()] {
        let correct = supported(&d);
        assert_eq!(return_line(&correct, "4b"), 10000);
        if let HsaFundingBasis::Roth { before_other_activity, .. } =
            &mut d.adjustments.as_mut().unwrap().hsa[0].funding[0].basis
        {
            *before_other_activity = true;
        }
        assert_eq!(
            crate::hsa_funding::source_reference_rule(&d),
            Some("HSA_FUNDING_ORDER_CONTRADICTED")
        );
        assert!(!d.valid_stored());
        assert_eq!(estimate(&d).state, ResultState::Incomplete);
        let mut raw_result = FederalResult::empty(d.revision);
        let funding = crate::hsa_funding::calculate(&d, &mut raw_result);
        let source = d.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
        assert_eq!(funding.for_entry(source), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
    // Preserve the raw decision when personal opening basis is missing; the raw source dates
    // must still reject a contradictory funding-first answer.
    let mut unconfirmed = roth_earlier_funding();
    let a = unconfirmed.adjustments.as_mut().unwrap();
    a.roth_history[0].ordering = None;
    a.hsa[0].funding[0].basis = HsaFundingBasis::Roth { value: 70000, before_other_activity: true };
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&unconfirmed),
        Some("HSA_FUNDING_ORDER_CONTRADICTED")
    );
    assert_eq!(estimate(&unconfirmed).state, ResultState::Incomplete);
    assert_eq!(restart(unconfirmed.clone()).draft, unconfirmed);

    // Later ordinary payments agree with funding-first. Earlier ordinary
    // payments in personal accounts or another decedent's group do not alter it.
    let mut separated = inherited_roth_earlier_draft();
    separated.adjustments.as_mut().unwrap().hsa[0].funding[0].basis =
        HsaFundingBasis::Roth { value: 70000, before_other_activity: true };
    if let FederalIncome::Roth { data } = &mut separated.income[3].income {
        let mut data = edit_roth(data);
        data.distributed = 20260715;
    }
    add(&mut separated, roth_withdrawal(Owner::Taxpayer, 100, 20260215));
    add_inherited_roth(&mut separated, 2023, 1000, 0, 1000);
    if let FederalIncome::Roth { data } = &mut separated.income.last_mut().unwrap().income {
        let mut data = edit_roth(data);
        data.distributed = 20260215;
    }
    let mut raw_result = FederalResult::empty(separated.revision);
    let funding = crate::hsa_funding::calculate(&separated, &mut raw_result);
    assert_eq!(funding.for_entry(3), 60000);
    assert_eq!(funding.inherited_basis_used[&2], 60000);
    separated.income.retain(|e| e.id != 4 && e.id != 5);
    let second = separated.next_id;
    let mut payment = inherited_roth_payment(Owner::Taxpayer, Some(2), 100);
    if let FederalIncome::Roth { data } = &mut payment.income {
        let mut data = edit_roth(data);
        data.distributed = 20261015;
        data.withholding = 0;
    }
    add(&mut separated, payment);
    let account = &mut separated.adjustments.as_mut().unwrap().hsa[0];
    let mut transfer = account.funding[0].clone();
    transfer.income = second;
    transfer.amount = 10000;
    transfer.funded = 20261016;
    transfer.coverage = HsaCoverage::Family;
    transfer.basis = HsaFundingBasis::Roth { value: 20000, before_other_activity: true };
    account.funding.push(transfer);
    account.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|month| if month < 6 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    let mut raw_result = FederalResult::empty(separated.revision);
    let funding = crate::hsa_funding::calculate(&separated, &mut raw_result);
    assert_eq!(funding.for_entry(second), 10000); // First source was fully funded.
    if let FederalIncome::Roth { data } =
        &mut separated.income.iter_mut().find(|e| e.id == 3).unwrap().income
    {
        edit_roth(data).gross += 100; // An ordinary remainder before the second source.
    }
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&separated),
        Some("HSA_FUNDING_ORDER_CONTRADICTED")
    );
    let mut raw_result = FederalResult::empty(separated.revision);
    let funding = crate::hsa_funding::calculate(&separated, &mut raw_result);
    assert_eq!(funding.for_entry(second), 0);
}

#[test]
fn print_inherited_ira_copies_keep_decedent_and_personal_basis_contiguous() {
    let mut d = complete(FederalStatus::Single);
    let first = add_inherited_ira(&mut d, 10_000, 5_000, 40_000);
    let second = add_inherited_ira(&mut d, 2_000, 10_000, 0);
    add(&mut d, basis_distribution(Owner::Taxpayer, 1_000, RetirementTreatment::Ordinary));
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 3_000, 4_000)];
    for reverse in [false, true] {
        if reverse {
            d.income.reverse();
        }
        let r = supported(&d);
        assert_eq!(return_line(&r, "4b"), 940_000);
        for (id, gross, recovered, carried, taxable) in
            [(first, 10_000, 1_000, 4_000, 9_000), (second, 2_000, 2_000, 8_000, 0)]
        {
            let form =
                |number: &str| value(&r, &alloc::format!("inherited_ira_{id}_form_line{number}"));
            assert_eq!(form("7"), gross);
            assert_eq!(form("12"), recovered);
            assert_eq!(form("14"), carried);
            assert_eq!(form("15a"), taxable);
            assert_eq!(form("15b"), 0);
            assert_eq!(form("15c"), taxable);
        }
        let review: Vec<_> = crate::draft_return(&r)
            .into_iter()
            .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8606))
            .collect();
        let mut starts: Vec<_> = review
            .iter()
            .enumerate()
            .filter_map(|(i, row)| (row.form_line.as_ref().unwrap().line == "2").then_some(i))
            .collect();
        assert_eq!(starts.len(), 3);
        starts.push(review.len());
        for span in starts.windows(2) {
            let rows = &review[span[0]..span[1]];
            let recovered = rows
                .iter()
                .find(|row| row.form_line.as_ref().unwrap().line == "12")
                .unwrap()
                .amount;
            let expected = match rows[0].amount {
                300_000 => 60_000,
                500_000 => 100_000,
                1_000_000 => 200_000,
                other => panic!("unexpected copy basis {other}"),
            };
            assert_eq!(recovered, expected);
            assert_eq!(
                rows.iter().filter(|row| row.form_line.as_ref().unwrap().line == "14").count(),
                1
            );
        }
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn print_inherited_roth_keeps_source_cents_and_qualified_form_condition() {
    let mut d = complete(FederalStatus::Single);
    let id = add_inherited_roth(&mut d, 2022, 4_000, 1_000, 2_500);
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income
        && let InheritedIraKindFacts::Roth { basis: Some(b), .. } = &mut data.kind
    {
        b.regular += 1;
        b.converted += 2;
    }
    if let FederalIncome::Roth { data } = &mut d.income[2].income {
        let mut data = edit_roth(data);
        data.gross += 6;
    }
    let r = supported(&d);
    let raw =
        |number: &str| line_amount(&r, &alloc::format!("inherited_roth_{id}_form_line{number}"));
    // 2026-10-19 (D-2026-10-19-12): the printed Part III lines are whole dollars figured from
    // each other (4,000.06, 1,000.01 and 2,500.02 print 4,000, 1,000 and 2,500), and line 25a
    // is the taxable amount that crosses.
    for (number, expected) in
        [("19", 400_000), ("22", 100_000), ("23", 300_000), ("24", 250_000), ("25a", 50_000)]
    {
        assert_eq!(raw(number), expected);
    }
    assert_eq!(line_amount(&r, &alloc::format!("inherited_roth_{id}_taxable")), 50_000);
    assert_eq!(restart(d.clone()).draft, d);
    if let FederalIncome::InheritedIra { data } = &mut d.income[1].income
        && let InheritedIraKindFacts::Roth { first_year, .. } = &mut data.kind
    {
        *first_year = 2021;
    }
    let qualified = supported(&d);
    assert!(
        qualified
            .lines
            .iter()
            .all(|row| { !row.id.starts_with(&alloc::format!("inherited_roth_{id}_form_line")) })
    );
    assert_eq!(return_line(&qualified, "4b"), 0);
}

#[test]
fn print_inherited_ira_only_files_for_actual_basis_and_distribution() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_ira(&mut d, 0, 1_000, 10_000);
    add_inherited_ira(&mut d, 1_000, 0, 10_000);
    add_inherited_roth(&mut d, 2021, 1_000, 0, 0);
    add_inherited_roth(&mut d, 2022, 0, 1_000, 0);
    let r = supported(&d);
    assert!(r.lines.iter().all(|row| !row.id.contains("_form_line")));
    assert_eq!(return_line(&r, "4b"), 100_000);
    // Sub-dollar source facts are real even though the filing-dollar owner
    // rounds their amounts to zero. Keep raw form presence and its zero-ratio
    // policy without inventing a visible nonzero amount.
    let mut d = complete(FederalStatus::Single);
    let id = add_inherited_ira(&mut d, 0, 0, 0);
    inherited_basis_mut(&mut d.income[1]).as_mut().unwrap().prior = 49;
    if let FederalIncome::Retirement { data } = &mut d.income[2].income {
        edit_retirement(data).gross = 49;
        if matches!(
            crate::FederalRetirement::from(&*data).taxable,
            Some(RetirementTaxable::Inherited { .. })
        ) {
            edit_retirement(data).box_2a = 49;
        }
    }
    let r = supported(&d);
    assert_eq!(
        r.lines
            .iter()
            .filter(|row| row.id.starts_with(&alloc::format!("inherited_ira_{id}_form_line")))
            .count(),
        17
    );
    assert_eq!(value(&r, &alloc::format!("inherited_ira_{id}_form_line15c")), 0);
}

// Scoped owner correction: Form8606's inherited taxable group enters
// Form8915's IRA portion. Original9000/5000 income is recorded in the audit.
fn assert_inherited_disaster_owner_correction(roth: bool) {
    let mut d = complete(FederalStatus::Single);
    if roth {
        add_inherited_roth(&mut d, 2022, 10_000, 5_000, 0);
    } else {
        add_inherited_ira(&mut d, 10_000, 5_000, 40_000);
    }
    let disaster = DisasterDistribution::Qualified {
        fema: 4834,
        began: 20260101,
        declared: 20260110,
        distributed: 20260601,
        amount: 1_000_000,
        earlier: 0,
        spread: DisasterSpread::OverThreeYears,
        repaid: DisasterRepayment::No,
    };
    match &mut d.income[2].income {
        FederalIncome::Retirement { data } => edit_retirement(data).disaster = disaster,
        FederalIncome::Roth { data } => edit_roth(data).disaster = disaster,
        _ => unreachable!(),
    }
    assert!(d.valid_stored());
    assert_eq!(restart(d.clone()).draft, d);
    let r = supported(&d);
    let raw = |number: &str| line_amount(&r, &alloc::format!("form_8915_taxpayer_2026_{number}"));
    assert_eq!(raw("line_8"), 0);
    assert_eq!(raw("line_10"), 0);
    assert_eq!(raw("line_20"), if roth { 500_000 } else { 900_000 });
    assert_eq!(raw("line_22"), if roth { 166_667 } else { 300_000 });
    assert_eq!(raw("line_26"), if roth { 166_667 } else { 300_000 });
    assert_eq!(raw("installment_following_year"), if roth { 166_667 } else { 300_000 });
    assert_eq!(raw("installment_second_following_year"), if roth { 166_666 } else { 300_000 });
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), if roth { 166_700 } else { 300_000 });
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "agi"), if roth { 61_667 } else { 63_000 });
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    let kind = if roth { "roth" } else { "ira" };
    let (relief, after) = if roth { ("25b", "25c") } else { ("15b", "15c") };
    assert_eq!(
        value(&r, &alloc::format!("inherited_{kind}_2_form_line{relief}")),
        if roth { 5000 } else { 9000 }
    );
    assert_eq!(value(&r, &alloc::format!("inherited_{kind}_2_form_line{after}")), 0);
    if !roth {
        assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 1_000);
        assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 4_000);
    }
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
#[test]
fn inherited_traditional_disaster_uses_its_group_basis_and_ira_spread() {
    assert_inherited_disaster_owner_correction(false);
}
#[test]
fn inherited_roth_disaster_uses_its_group_earnings_and_ira_spread() {
    assert_inherited_disaster_owner_correction(true);
}

#[test]
fn inherited_disaster_copies_keep_groups_filers_and_rounding_separate() {
    for status in [FederalStatus::Single, FederalStatus::Joint] {
        let mut d = complete(status);
        let a = add_inherited_ira(&mut d, 10_000, 5_000, 40_000);
        let b = add_inherited_ira(&mut d, 10_000, 0, 0);
        let c = add_inherited_roth(&mut d, 2022, 5_000, 2_000, 0);
        if status == FederalStatus::Joint {
            for entry in &mut d.income {
                if entry.id == b || entry.id == b + 1 || entry.id == c || entry.id == c + 1 {
                    entry.owner = crate::DocumentOwner::Spouses;
                }
            }
        }
        assert_eq!(return_line(&supported(&d), "4b"), 2_200_000);
        for (id, amount) in [(a + 1, 600_000), (b + 1, 1_000_000), (c + 1, 250_000)] {
            let disaster = DisasterDistribution::Qualified {
                fema: 4834,
                began: 20260101,
                declared: 20260110,
                distributed: 20260601,
                amount,
                earlier: 0,
                spread: DisasterSpread::OverThreeYears,
                repaid: DisasterRepayment::No,
            };
            match &mut d.income.iter_mut().find(|entry| entry.id == id).unwrap().income {
                FederalIncome::Retirement { data } => edit_retirement(data).disaster = disaster,
                FederalIncome::Roth { data } => edit_roth(data).disaster = disaster,
                _ => unreachable!(),
            }
        }
        let r = supported(&d);
        assert_eq!(return_line(&r, "4b"), 1_073_300);
        assert_eq!(return_line(&r, "5b"), 0);
        assert_eq!(value(&r, &alloc::format!("inherited_ira_{a}_form_line15b")), 5400);
        assert_eq!(value(&r, &alloc::format!("inherited_ira_{a}_form_line15c")), 3600);
        assert_eq!(value(&r, &alloc::format!("inherited_roth_{c}_form_line25b")), 1500);
        assert_eq!(value(&r, &alloc::format!("inherited_roth_{c}_form_line25c")), 1500);
        assert_eq!(value(&r, "form_8915_taxpayer_2026_line_8"), 0);
        assert_eq!(
            value(&r, "form_8915_taxpayer_2026_line_20"),
            if status == FederalStatus::Single { 16900 } else { 5400 }
        );
        if status == FederalStatus::Joint {
            assert_eq!(value(&r, "form_8915_spouse_2026_line_20"), 11500);
        }
        assert_eq!(restart(d.clone()).draft, d);
        d.income.reverse();
        assert_eq!(supported(&d).tax, r.tax);
    }
}

#[test]
fn inherited_disaster_qcd_is_excluded_once_and_actual_overlap_rejects() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    let id = add_inherited_ira(&mut d, 10_000, 5_000, 40_000);
    let mut gift = qcd(Owner::Taxpayer, 20260601, 5000, 0);
    gift.payments[0].inherited = Some(id);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    if let FederalIncome::Retirement { data } = &mut d.income[2].income {
        edit_retirement(data).disaster = DisasterDistribution::Qualified {
            fema: 4834,
            began: 20260101,
            declared: 20260110,
            distributed: 20260601,
            amount: 500_000,
            earlier: 0,
            spread: DisasterSpread::OverThreeYears,
            repaid: DisasterRepayment::No,
        };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 5000);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 556);
    assert_eq!(value(&r, "inherited_ira_2_form_line7"), 5000);
    assert_eq!(value(&r, "inherited_ira_2_form_line15a"), 4444);
    assert_eq!(value(&r, "inherited_ira_2_form_line15b"), 4444);
    assert_eq!(value(&r, "inherited_ira_2_form_line15c"), 0);
    assert_eq!(return_line(&r, "4b"), 148_100);
    d.adjustments.as_mut().unwrap().qualified_charity[0].payments[0].amount = 800_000;
    let invalid = estimate(&d);
    assert_eq!(invalid.state, ResultState::Unsupported);
    assert!(
        invalid.issues.iter().any(|issue| issue.display.key == "ISSUE_INHERITED_DISASTER_OVERLAP")
    );
    assert_eq!(
        invalid
            .issues
            .iter()
            .find(|issue| issue.display.key == "ISSUE_INHERITED_DISASTER_OVERLAP")
            .unwrap()
            .context,
        crate::interview::income_issue_context(&d.income[2])
    );
    assert_eq!(invalid.tax, None);
}

#[test]
fn inherited_disaster_without_group_preserves_known_payer_fallback() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, inherited_payment(Owner::Taxpayer, None, 10_000));
    if let FederalIncome::Retirement { data } = &mut d.income[1].income {
        edit_retirement(data).disaster = DisasterDistribution::Qualified {
            fema: 4834,
            began: 20260101,
            declared: 20260110,
            distributed: 20260601,
            amount: 600_000,
            earlier: 0,
            spread: DisasterSpread::OverThreeYears,
            repaid: DisasterRepayment::No,
        };
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_INHERITED_IRA_RECORD_MISSING"));
    assert_eq!(value(&r, "form_8915_taxpayer_2026_line_20"), 6000);
    assert_eq!(return_line(&r, "4b"), 600_000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert!(r.lines.iter().all(|row| !row.id.starts_with("inherited_ira_")));
}

#[test]
fn inherited_disaster_hsa_funding_preserves_the_existing_group_basis() {
    let mut d = inherited_hsa_funding(1600, 200, 2000, 400, 1500);
    if let FederalIncome::Retirement { data } = &mut d.income[2].income {
        edit_retirement(data).disaster = DisasterDistribution::Qualified {
            fema: 4834,
            began: 20260101,
            declared: 20260110,
            distributed: 20260601,
            amount: 10_000,
            earlier: 0,
            spread: DisasterSpread::OverThreeYears,
            repaid: DisasterRepayment::No,
        };
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 160_000);
    assert_eq!(value(&r, "inherited_ira_2_basis_recovered"), 40);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 160);
    assert_eq!(value(&r, "inherited_ira_2_form_line7"), 100);
    assert_eq!(value(&r, "inherited_ira_2_form_line15a"), 60);
    assert_eq!(value(&r, "inherited_ira_2_form_line15b"), 60);
    assert_eq!(value(&r, "inherited_ira_2_form_line15c"), 0);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_line_20"), 60);
    assert_eq!(value(&r, "form_8915_taxpayer_2026_line_26"), 20);
    assert_eq!(return_line(&r, "4b"), 2000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn inherited_disaster_linked_payments_conserve_the_group_cent_residual() {
    let mut d = complete(FederalStatus::Single);
    let group = add_inherited_ira(&mut d, 10_000, 5_000, 40_000);
    if let FederalIncome::Retirement { data } = &mut d.income[2].income {
        edit_retirement(data).gross = 500_000;
        if matches!(
            crate::FederalRetirement::from(&*data).taxable,
            Some(RetirementTaxable::Inherited { .. })
        ) {
            edit_retirement(data).box_2a = 500_000;
        }
    }
    add(&mut d, inherited_payment(Owner::Taxpayer, Some(group), 5000));
    for entry in &mut d.income {
        if let FederalIncome::Retirement { data } = &mut entry.income {
            edit_retirement(data).disaster = DisasterDistribution::Qualified {
                fema: 4834,
                began: 20260101,
                declared: 20260110,
                distributed: 20260601,
                amount: 333_333,
                earlier: 0,
                spread: DisasterSpread::OverThreeYears,
                repaid: DisasterRepayment::No,
            };
        }
    }
    let r = supported(&d);
    let designated = crate::inherited_ira::allocate_designated(
        &d,
        &mut FederalResult::empty(d.revision),
        group,
        1_000_000,
        900_000,
    );
    assert_eq!(designated[&3], 300_000);
    assert_eq!(designated[&4], 299_999);
    let raw = |id: &str| line_amount(&r, id);
    assert_eq!(raw("inherited_ira_2_form_line15b"), 599_999);
    assert_eq!(raw("inherited_ira_2_form_line15c"), 300_001);
    assert_eq!(raw("form_8915_taxpayer_2026_line_20"), 599_999);
    assert_eq!(raw("form_8915_taxpayer_2026_line_22"), 200_000);
    assert_eq!(raw("form_8915_taxpayer_2026_installment_following_year"), 200_000);
    assert_eq!(raw("form_8915_taxpayer_2026_installment_second_following_year"), 199_999);
    assert_eq!(return_line(&r, "4b"), 500_000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    assert_eq!(
        crate::inherited_ira::allocate_designated(
            &d,
            &mut FederalResult::empty(d.revision),
            group,
            1_000_000,
            900_000
        ),
        designated
    );
}
