use super::support::builders::unit50_retirement::ira_excess;
use super::*;

#[test]
fn community_portfolio_multiple_w2s_preserve_worker_fica_ira_and_eic_compensation() {
    let mut high =
        community_joint_portfolio_draft(12_000_000, 13_000_000, 200_000, 300_000, 100_000);
    add(&mut high, wage(0, Owner::Taxpayer, 12_000_000, 0));
    add(&mut high, wage(0, Owner::Spouse, 13_000_000, 0));
    let r = supported(&high);
    assert_eq!(return_line(&r, "1a"), 25_000_000);
    assert_eq!(return_line(&r, "2b"), 100_000);
    assert_eq!(return_line(&r, "3a"), 50_000);
    assert_eq!(return_line(&r, "3b"), 150_000);
    assert_eq!(value(&r, "additional_medicare_tax"), 1_035);
    assert_eq!(return_line(&r, "31"), 344_100);
    assert_eq!(value(&r, "net_investment_income_tax"), 95);

    let mut ira = community_interest_draft(250_000, 5_000_000, 200_000, Owner::Spouse);
    add(&mut ira, wage(0, Owner::Taxpayer, 250_000, 0));
    add(&mut ira, wage(0, Owner::Spouse, 5_000_000, 0));
    let mut contribution = ira_facts(400_000, 0, Answer::No);
    contribution.spouse = IraSpouse::SeparateTogether { covered_at_work: false };
    ira.adjustments.as_mut().unwrap().iras = contribution;
    assert_eq!(value(&supported(&ira), "ira_deduction"), 4_000);

    let mut eic = community_dividend_draft(500_000, 0, 200_000, 100_000, Owner::Taxpayer);
    add(&mut eic, wage(0, Owner::Taxpayer, 500_000, 0));
    add(&mut eic, wage(0, Owner::Spouse, 0, 0));
    eic.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut eic);
    let r = supported(&eic);
    assert_eq!(value(&r, "agi"), 6_000);
    assert_eq!(value(&r, "eic_investment_income"), 1_000);
    assert_eq!(value(&r, "earned_income_credit"), eic_amount(10_000, 6_000, 1, false));
    assert_eq!(restart(eic.clone()).draft, eic);
}

#[test]
fn community_joint_interest_and_dividends_reach_separate_form_8958_rows_once() {
    let mut d = community_joint_portfolio_draft(6_000_000, 10_000_000, 400_000, 600_000, 200_000);
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        edit_wage(data).withholding = 650_000;
    }
    if let FederalIncome::Wage { data } = &mut d.income[1].income {
        edit_wage(data).withholding = 1_200_000;
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 8_000_000);
    assert_eq!(return_line(&r, "2b"), 200_000);
    assert_eq!(return_line(&r, "3a"), 100_000);
    assert_eq!(return_line(&r, "3b"), 300_000);
    assert_eq!(return_line(&r, "25a"), 925_000);
    assert_eq!(value(&r, "agi"), 85_000);
    assert_eq!(value(&r, "form8958_3_total"), 4_000);
    assert_eq!(value(&r, "form8958_3_this_return"), 2_000);
    assert_eq!(value(&r, "form8958_4_total"), 6_000);
    assert_eq!(value(&r, "form8958_4_this_return"), 3_000);
    assert!(noted(&r, "NOTE_FORM_8958_ALLOCATION"));

    let mut control = complete(FederalStatus::Separate);
    control.deductions.as_mut().unwrap().itemize_override = None;
    control.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 925_000);
    add(
        &mut control,
        income(FederalIncome::Interest { data: interest_source(native_interest(200_000, 0)) }),
    );
    add(
        &mut control,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(300_000, 100_000, 0)
            }),
        }),
    );
    let expected = supported(&control);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(r.refund, expected.refund);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);

    let mut ordinary = d;
    let FederalIncome::Dividend { data } = &mut ordinary.income[3].income else { unreachable!() };
    edit_dividend(data).qualified = 0;
    let ordinary_result = supported(&ordinary);
    assert_eq!(value(&ordinary_result, "agi"), 85_000);
    assert!(value(&r, "regular_income_tax") < value(&ordinary_result, "regular_income_tax"));
}

#[test]
fn community_w2_mfs_retains_worker_medicare_and_ira_compensation() {
    let d = community_w2_draft(24_000_000, 26_000_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "wages"), 250_000);
    assert_eq!(value(&r, "additional_medicare_tax"), 1_035);
    assert_eq!(return_line(&r, "25c"), 36_000); // Only the worker's $360 Form 8959 withholding.

    let mut d = community_w2_draft(500_000, 10_000_000, 0, 0);
    let mut ira = ira_facts(400_000, 0, Answer::No);
    ira.spouse = IraSpouse::SeparateTogether { covered_at_work: false };
    d.adjustments.as_mut().unwrap().iras = ira;
    assert_eq!(value(&supported(&d), "ira_deduction"), 4_000);
    assert_eq!(restart(d.clone()).draft, d);

    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    unsupported(&d, "IRA"); // Allocated wages cannot create IRA compensation for the nonworker.
}

#[test]
fn community_w2_mfs_separated_eic_uses_earner_wages_and_allocated_agi() {
    let mut d = community_w2_draft(1_000_000, 0, 0, 0);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::LastSixMonths };
    add_child(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 5_000);
    assert_eq!(value(&r, "earned_income_credit"), eic_amount(10_000, 5_000, 1, false));
    assert!(value(&r, "earned_income_credit") > eic_amount(5_000, 5_000, 1, false));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn retirement_senior_and_social_flow() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500615;
    d.income = vec![wage(1, Owner::Taxpayer, 940_000, 0)];
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(native_retirement(
                RetirementKind::Pension,
                1_860_000,
                (RetirementTaxable::Payer, 1_860_000),
            )),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(99_000, 0)) }),
    );
    add(&mut d, income(social_security_net(598_000, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_social_security"), 2990);
    assert_eq!(value(&r, "senior_deduction"), 6000);
    assert_eq!(value(&r, "deduction"), 18150);
    assert_eq!(r.tax, Some(78_300));
}

#[test]
fn multiple_529_accounts_share_lifetime_annual_limits_and_form_5498_once() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    draft.adjustments.as_mut().unwrap().iras = ira_facts(0, 100_000, Answer::No);
    draft.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let ordinary = supported(&draft);
    let owner = &mut draft.adjustments.as_mut().unwrap().iras.taxpayer;
    owner.qtp_roth_lifetime_prior = 2_900_000;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &two_qtp_roth_rollovers());
    let result = supported(&draft);
    assert_eq!(value(&result, "taxpayer_529_roth_form_5498_box10"), 6000);
    assert_eq!(value(&result, "taxpayer_529_roth_source_contribution_basis"), 3500);
    assert_eq!(value(&result, "taxpayer_529_roth_transfer_1_amount"), 3000);
    assert_eq!(value(&result, "taxpayer_529_roth_transfer_2_amount"), 3000);
    assert_eq!(value(&result, "saver_credit"), value(&ordinary, "saver_credit"));
    assert_eq!(result.tax, ordinary.tax);
    assert!(result.lines.iter().any(|line| {
        line.id == "taxpayer_529_roth_transfer_1_amount"
            && line.explanation.english().contains("PLAN-A")
    }));
    assert!(result.lines.iter().any(|line| {
        line.id == "taxpayer_529_roth_transfer_2_amount"
            && line.explanation.english().contains("PLAN-B")
    }));
    assert_eq!(supported(&restart(draft.clone()).draft), result);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn multiple_529_sources_require_each_account_age_and_seasoned_balance() {
    let mut draft = complete(FederalStatus::Single);
    draft.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
    let mut transfers = two_qtp_roth_rollovers();
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    supported(&draft);

    transfers[1].rollover.account_opened_on = 20110603;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    assert!(!draft.valid_stored());
    transfers[1].rollover.account_opened_on = 20110602;
    transfers[1].rollover.seasoned_available = 299_999;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    assert!(!draft.valid_stored());
    transfers[1].rollover.seasoned_available = 300_000;
    transfers[1].rollover.confirmed = false;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    assert!(noted(&supported(&draft), "NOTE_QTP_ROTH_NOT_QUALIFIED"));
    transfers[1].rollover.confirmed = true;
    set_qtp_roth(&mut draft, Owner::Taxpayer, &transfers);
    draft.adjustments.as_mut().unwrap().iras.taxpayer.qtp_roth_lifetime_prior = 2_900_001;
    assert!(!draft.valid_stored());
}

#[test]
fn multiple_529_transfers_exceeding_combined_ira_or_compensation_room_are_guarded() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 7_000_000, 0);
    draft.adjustments.as_mut().unwrap().iras = ira_facts(0, 100_000, Answer::No);
    set_qtp_roth(&mut draft, Owner::Taxpayer, &two_qtp_roth_rollovers());
    supported(&draft); // $6,000 of transfers plus $1,000 regular Roth contribution.
    draft.adjustments.as_mut().unwrap().iras.taxpayer.roth = 200_000;
    unsupported(&draft, "IRA");
    draft.adjustments.as_mut().unwrap().iras.taxpayer.roth = 100_000;
    // Compensation does not limit a 529-to-Roth rollover (section 529(c)(3)(E)(ii)(II): the IRA
    // dollar limit less the year's contributions), so low pay with regular contributions inside
    // it still computes.
    draft.income[0] = wage(1, Owner::Taxpayer, 650_000, 0);
    supported(&draft);
}

#[test]
fn qtp_to_roth_rollover_uses_annual_room_but_not_income_or_saver_base() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 100_000, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let ordinary = supported(&d);
    assert_eq!(value(&ordinary, "saver_credit"), 100);
    set_qtp_roth(&mut d, Owner::Taxpayer, &[qtp_roth_rollover()]);
    let result = supported(&d);
    assert_eq!(value(&result, "taxpayer_529_roth_form_5498_box10"), 6000);
    assert_eq!(value(&result, "taxpayer_529_roth_source_contribution_basis"), 4000);
    assert_eq!(value(&result, "saver_credit"), 100);
    assert_eq!(value(&result, "ira_deduction"), 0);
    assert_eq!(value(&result, "agi"), 30000);
    assert_eq!(result.tax, ordinary.tax);
    assert!(result.review_notes.iter().any(|n| n.contains("Form 1099-Q") && n.contains("box 4b")));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qtp_to_roth_rollover_checks_15_and_five_year_lifetime_and_compensation_limits() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 7_000_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(100_000, 0, Answer::No);
    let mut transfer = qtp_roth_rollover();
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer.clone()]);
    supported(&d); // $1,000 regular plus $6,000 rollover uses $7,000 of $7,500 room.
    transfer.rollover.account_opened_on = 20110602;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer.clone()]);
    assert!(!d.valid_stored());
    transfer.rollover.account_opened_on = 20110601;
    transfer.rollover.seasoned_available = 599_999;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer.clone()]);
    assert!(!d.valid_stored());
    transfer.rollover.seasoned_available = 600_000;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer.clone()]);
    d.adjustments.as_mut().unwrap().iras.taxpayer.qtp_roth_lifetime_prior = 2_900_001;
    assert!(!d.valid_stored());
    d.adjustments.as_mut().unwrap().iras.taxpayer.qtp_roth_lifetime_prior = 2_900_000;
    transfer.rollover.confirmed = false;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer.clone()]);
    assert!(noted(&supported(&d), "NOTE_QTP_ROTH_NOT_QUALIFIED"));
    transfer.rollover.confirmed = true;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer.clone()]);
    d.income[0] = wage(1, Owner::Taxpayer, 650_000, 0);
    supported(&d); // the rollover needs no compensation
    d.income[0] = wage(1, Owner::Taxpayer, 7_000_000, 0);
    d.adjustments.as_mut().unwrap().iras.taxpayer.roth = 200_000;
    unsupported(&d, "IRA");
}

#[test]
fn qtp_to_roth_rollover_is_not_barred_by_regular_roth_income_phaseout() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
    set_qtp_roth(&mut d, Owner::Taxpayer, &[qtp_roth_rollover()]);
    let result = supported(&d);
    assert_eq!(value(&result, "taxpayer_529_roth_form_5498_box10"), 6000);
    d.adjustments.as_mut().unwrap().iras.taxpayer.roth = 100;
    unsupported(&d, "IRA contributions");
}

#[test]
fn actual_2027_nondeductible_deposit_cannot_shelter_2026_ira_distribution() {
    let mut d = ira_contribution_distribution(200_000, Answer::Yes);
    d.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 750_000;
    d.adjustments.as_mut().unwrap().iras.taxpayer.next_year_deposits =
        Some(next_year_ira_deposits(750_000, 0, 20270331));
    let result = supported(&d);
    assert_eq!(value(&result, "ira_deduction"), 0);
    assert_eq!(value(&result, "taxpayer_nondeductible_ira"), 7500);
    assert_eq!(value(&result, "taxpayer_ira_basis_recovered"), 0);
    assert_eq!(value(&result, "taxpayer_ira_basis_carryforward"), 7500);
    assert_eq!(return_line(&result, "4b"), 1_000_000);
    assert_eq!(value(&result, "taxpayer_2026_form_5498_box1_2027_deposit"), 7500);
    assert_eq!(restart(d.clone()).draft, d);

    d.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 0;
    unsupported(&d, "IRA basis");
}

#[test]
fn next_year_ira_deposit_requires_actual_deadline_designation_and_retained_principal() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().iras = ira_facts(200_000, 0, Answer::No);
    let fact = &mut d.adjustments.as_mut().unwrap().iras.taxpayer;
    fact.next_year_deposits = Some(next_year_ira_deposits(200_000, 0, 20270416));
    unsupported(&d, "IRA");
    let deposits =
        d.adjustments.as_mut().unwrap().iras.taxpayer.next_year_deposits.as_mut().unwrap();
    deposits.last_deposited_on = 20270415;
    deposits.designation_confirmed = false;
    // Not designated for 2026: the 2027 deposit leaves the 2026 totals.
    assert!(noted(&supported(&d), "WARN_IRA_NEXT_YEAR_DEPOSIT_LEFT_OUT"));
    let deposits =
        d.adjustments.as_mut().unwrap().iras.taxpayer.next_year_deposits.as_mut().unwrap();
    deposits.designation_confirmed = true;
    deposits.traditional = 200_001;
    unsupported(&d, "IRA");
    let deposits =
        d.adjustments.as_mut().unwrap().iras.taxpayer.next_year_deposits.as_mut().unwrap();
    deposits.traditional = 0;
    assert!(!d.valid_stored());
    let deposits =
        d.adjustments.as_mut().unwrap().iras.taxpayer.next_year_deposits.as_mut().unwrap();
    deposits.traditional = 200_000;
    deposits.last_deposited_on = 20261231;
    assert!(!d.valid_stored());
}

#[test]
fn ira_2026_phaseouts_round_up_and_preserve_minimum_until_endpoint() {
    use crate::ira::phase_limit;
    for (start, width) in [(81000, 10000), (129000, 20000), (242000, 10000), (0, 10000)] {
        assert_eq!(phase_limit(7500, start, start, width), 7500);
        assert_eq!(phase_limit(7500, start + width / 2, start, width), 3750);
        assert_eq!(phase_limit(7500, start + width - 1, start, width), 200);
        assert_eq!(phase_limit(7500, start + width, start, width), 0);
        assert_eq!(phase_limit(8600, start + width / 2, start, width), 4300);
    }
    assert_eq!(phase_limit(7500, 81101, 81000, 10000), 7430);
}
#[test]
fn ira_deduction_uses_coverage_and_spousal_compensation() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 8_600_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::Yes);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 3750);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 3750);
    assert_eq!(value(&r, "agi"), 82250);
    d.adjustments.as_mut().unwrap().iras.taxpayer.covered_at_work = false;
    // The readonly projection cannot erase participation recorded on the W-2.
    assert_eq!(value(&supported(&d), "ira_deduction"), 3750);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    edit_wage(data).retirement_plan = false;
    assert_eq!(value(&supported(&d), "ira_deduction"), 7500);
    let mut d = complete(FederalStatus::Joint);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    edit_wage(data).retirement_plan = true;
    let mut f = ira_facts(750_000, 0, Answer::Yes);
    f.spouse = IraSpouse::Joint {
        facts: FederalIraContribution {
            nondeductible_election: 0,
            traditional: 750_000,
            roth: 0,
            covered_at_work: false,
            returned_next_year: vec![],
            excess: None,
            next_year_deposits: None,
            qtp_roth_lifetime_prior: 0,
        },
    };
    d.adjustments.as_mut().unwrap().iras = f;
    assert_eq!(value(&supported(&d), "ira_deduction"), 15000);
    // The nonworking spouse shares compensation, but it cannot fund both accounts twice.
    d.income[0] = covered_wage(1, Owner::Taxpayer, 1_400_000, 0);
    unsupported(&d, "IRA");
}
#[test]
fn ira_separate_apart_and_surviving_spouse_thresholds() {
    let d = complete(FederalStatus::Separate);
    let mut setup = d.setup.unwrap();
    let people = d.people.unwrap();
    let mut f = ira_facts(750_000, 0, Answer::Yes);
    f.spouse = IraSpouse::SeparateTogether { covered_at_work: false };
    assert_eq!(
        crate::ira::calculate(&f, &people, &setup, [86000, 0], 86000, &Default::default(), [0, 0])
            .unwrap()
            .deduction,
        0
    );
    setup.marital_state = MaritalState::Married { living: SpouseLiving::AllYear };
    f.spouse = IraSpouse::NotApplicable;
    assert_eq!(
        crate::ira::calculate(&f, &people, &setup, [86000, 0], 86000, &Default::default(), [0, 0])
            .unwrap()
            .deduction,
        3750
    );
    setup.status = filing_choice(FederalStatus::Surviving);
    assert_eq!(
        crate::ira::calculate(
            &f,
            &people,
            &setup,
            [139000, 0],
            139000,
            &Default::default(),
            [0, 0]
        )
        .unwrap()
        .deduction,
        3750
    );
    setup.status = filing_choice(FederalStatus::Separate);
    setup.marital_state = MaritalState::Married { living: SpouseLiving::Together };
    f.taxpayer.covered_at_work = false;
    f.spouse = IraSpouse::SeparateTogether { covered_at_work: true };
    assert_eq!(
        crate::ira::calculate(&f, &people, &setup, [86000, 0], 86000, &Default::default(), [0, 0])
            .unwrap()
            .deduction,
        0
    );
}
#[test]
fn ira_social_security_recomputed_then_student_loan_phaseout() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19650615;
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    add(&mut d, income(social_security_net(2_000_000, 0)));
    let a = d.adjustments.as_mut().unwrap();
    a.iras = ira_facts(860_000, 0, Answer::Yes);
    a.set_student_loan_interest(250_000);
    d.credits.as_mut().unwrap().saver = (Some(saver(0, 0))).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 8600);
    // (30,000 - 8,600 + half of 20,000 - 25,000) * 50% = 3,200.
    assert_eq!(value(&r, "taxable_social_security"), 3200);
    assert_eq!(value(&r, "agi"), 22100);
    assert_eq!(value(&r, "adjustments"), 11100);
}
#[test]
fn roth_limits_share_annual_cap_and_feed_saver_without_second_deduction() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_500_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 200_000, Answer::No);
    // Saver's Credit facts not entered: no credit, with a warning.
    assert!(noted(&supported(&d), "WARN_SAVER_CREDIT_NOT_CLAIMED"));
    d.credits.as_mut().unwrap().saver = (Some(saver(100_000, 0))).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 25000);
    assert_eq!(value(&r, "saver_credit"), 400); // one shared $2,000 cap
    // validate_roth now returns each owner's Roth limit in cents (T2-42):
    // $7,500 x (168,000 - 160,500) / 15,000 = $3,750, then the unphased
    // $7,500 less $4,000 traditional = $3,500 wins.
    let mut f = ira_facts(400_000, 350_000, Answer::Yes);
    let p = d.people.as_ref().unwrap();
    let s = d.setup.as_ref().unwrap();
    let limit =
        crate::ira::validate_roth(&f, p, s, [160500, 0], 160500, &Default::default()).unwrap()[0];
    assert_eq!(limit, 350_000);
    assert!(f.taxpayer.roth <= limit);
    f.taxpayer.roth = 350_001;
    let limit =
        crate::ira::validate_roth(&f, p, s, [160500, 0], 160500, &Default::default()).unwrap()[0];
    assert!(f.taxpayer.roth > limit);
    // $1 inside the phase-out rounds up to the $200 minimum; at the end it is zero.
    f.taxpayer.traditional = 0;
    f.taxpayer.roth = 20_000;
    let limit =
        crate::ira::validate_roth(&f, p, s, [167999, 0], 167999, &Default::default()).unwrap()[0];
    assert_eq!(limit, 20_000);
    let limit =
        crate::ira::validate_roth(&f, p, s, [168000, 0], 168000, &Default::default()).unwrap()[0];
    assert_eq!(limit, 0);
    assert!(f.taxpayer.roth > limit);
}

#[test]
fn roth_missing_other_coverage_keeps_its_message_key() {
    let d = complete(FederalStatus::Joint);
    let result = crate::ira::validate_roth(
        &ira_facts(0, 0, Answer::Yes),
        d.people.as_ref().unwrap(),
        d.setup.as_ref().unwrap(),
        [0, 0],
        0,
        &Default::default(),
    );
    assert_eq!(result.unwrap_err().0, "ISSUE_IRA_SPOUSE_COVERAGE");
}

#[test]
fn ira_age_catchup_exact_cent_limits_and_uncovered_owner() {
    // Pub501: a January1 birthday attains the age on December31.
    for (coverage, roth, birth, amount, accepted) in [
        (Answer::No, false, 19761231, 860_000, true),
        (Answer::No, true, 19761231, 860_000, true),
        (Answer::No, false, 19770101, 860_000, true),
        (Answer::No, true, 19770101, 860_000, true),
        (Answer::No, true, 19770102, 860_000, false),
        (Answer::No, false, 19770102, 860_000, false),
        (Answer::No, false, 19770102, 750_000, true),
        (Answer::Yes, false, 19761231, 860_000, true),
        (Answer::Yes, false, 19770101, 860_000, true),
        (Answer::Yes, false, 19770102, 860_000, false),
        (Answer::Yes, false, 19770102, 750_001, false),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        d.adjustments.as_mut().unwrap().iras =
            if roth { ira_facts(0, amount, coverage) } else { ira_facts(amount, 0, coverage) };
        if accepted {
            let result = supported(&d);
            if !roth {
                assert_eq!(value(&result, "ira_deduction"), amount / 100, "{birth}");
            }
        } else {
            unsupported(&d, "IRA");
        }
    }
}

#[test]
fn incomplete_spousal_ira_facts_cannot_panic_during_roth_validation() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Spouse, 10_000_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 100_000, Answer::No);
    unsupported(&d, "IRA");
    d.adjustments.as_mut().unwrap().iras.spouse = IraSpouse::Joint {
        facts: FederalIraContribution {
            nondeductible_election: 0,
            traditional: 0,
            roth: 0,
            covered_at_work: false,
            returned_next_year: vec![],
            excess: None,
            next_year_deposits: None,
            qtp_roth_lifetime_prior: 0,
        },
    };
    d.people.as_mut().unwrap().spouse = SpouseFiler::No;
    unsupported(&d, "IRA");
}

#[test]
fn ira_overtime_vehicle_and_education_cross_topic_fixture() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 650_000);
    let FederalIncome::Wage { data: w } = &mut d.income[0].income else { panic!() };
    edit_wage(w).work_deductions = work_deductions(0, 500_000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::Yes);
    d.deductions.as_mut().unwrap().vehicle_loans = vec![crate::VehicleLoanSource {
        id: 0,
        document: vehicle_paper(crate::FederalVehicleInterest {
            lender: crate::PrintedParty::from_name("Synthetic vehicle".into()),
            borrower: Default::default(),
            interest: 200_000,
            refund: 0,
            original_use: true,
            assembled_in_us: true,
        }),
        assigned: crate::DocumentRecipient::Taxpayer,
        owner_source: Some(crate::OwnerSource::Added),
        joint_share: None,
        business: 0,
    }];
    let sid0_1693 = -1; // You, the first person of the catalog
    set_education(
        d.credits.as_mut().unwrap(),
        vec![native_education(
            sid0_1693,
            education_expenses(400_000),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        )],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 72500);
    assert_eq!(value(&r, "taxable_income"), 49400);
    assert_eq!(value(&r, "income_tax"), 5683);
    assert_eq!(value(&r, "education_credit"), 1500);
    assert_eq!(r.tax, Some(418_300));
    assert_eq!(r.refund, Some(331_700));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn separate_filing_branch_owns_spouse_itemization() {
    // The deductions page's route owns the answer; it forces itemizing only
    // on a separate return.
    let mut draft = complete(FederalStatus::Separate);
    draft.deductions.as_mut().unwrap().spouse_itemizes = Some(true);
    draft.deductions.as_mut().unwrap().itemize_override = None;
    // 2026-11-02: the spouse fact itself requires itemization.
    supported(&draft);
    draft.setup.as_mut().unwrap().status = FilingChoice::Single;
    draft.setup.as_mut().unwrap().marital_state = MaritalState::Unmarried;
    assert!(!draft.spouse_itemizes());
    draft.deductions.as_mut().unwrap().spouse_itemizes = Some(false);
    answer_separate_pages(&mut draft);
    draft.setup.as_mut().unwrap().status =
        FilingChoice::Separate { section179: Section179Spouse::NotSupplied };
    draft.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    assert!(!draft.spouse_itemizes());
    supported(&draft);
}

#[test]
fn ira_spouse_branch_must_match_the_return_context() {
    let draft = complete(FederalStatus::Single);
    let mut facts = ira_facts(10000, 0, Answer::No);
    facts.spouse = IraSpouse::SeparateTogether { covered_at_work: true };
    assert!(
        crate::ira::calculate(
            &facts,
            draft.people.as_ref().unwrap(),
            draft.setup.as_ref().unwrap(),
            [50000, 0],
            50000,
            &Default::default(),
            [0, 0]
        )
        .is_err()
    );
    facts.spouse = IraSpouse::NotApplicable;
    assert!(
        crate::ira::calculate(
            &facts,
            draft.people.as_ref().unwrap(),
            draft.setup.as_ref().unwrap(),
            [50000, 0],
            50000,
            &Default::default(),
            [0, 0]
        )
        .is_ok()
    );
}

#[test]
fn ira_and_pension_round_at_their_own_return_lines() {
    let mut draft = complete(FederalStatus::Single);
    let initial_agi = value(&supported(&draft), "agi");
    for kind in [RetirementKind::Ira, RetirementKind::Pension] {
        add(
            &mut draft,
            income(FederalIncome::Retirement {
                data: retirement_source(native_retirement(
                    kind,
                    100_050,
                    (RetirementTaxable::Payer, 100_050),
                )),
            }),
        );
    }
    let result = supported(&draft);
    assert_eq!(return_line(&result, "4b"), 100_100);
    assert_eq!(return_line(&result, "5b"), 100_100);
    assert_eq!(value(&result, "agi"), initial_agi + 2002);
}

#[test]
fn ira_spouse_choices_follow_accepted_setup() {
    for (status, apart, expected) in [
        (FederalStatus::Single, false, 0),
        (FederalStatus::Head, false, 0),
        (FederalStatus::Surviving, false, 0),
        (FederalStatus::Joint, false, 1),
        (FederalStatus::Separate, false, 2),
        (FederalStatus::Separate, true, 0),
    ] {
        let mut draft = complete(status);
        if apart {
            draft.setup.as_mut().unwrap().marital_state =
                MaritalState::Married { living: SpouseLiving::AllYear };
        }
        // The adjustments editor offers exactly the accepted setup's choice.
        assert_eq!(draft.setup.as_ref().unwrap().ira_spouse_index(), expected);
        // Amended for M3-P7: the adjustments section is the adjustments hub,
        // which opening from the review leaves unchanged.
        let mut interview = Interview::new(TEST_USER, draft.clone());
        let reply = send(
            &mut interview,
            P::Navigate {
                user: TEST_USER,
                cookie: draft.cookie.clone(),
                revision: draft.revision,
                from: TaxSection::Review as i32,
                section: (TaxSection::Adjustments).into(),
                navigation: NavigationAvailability::default(),
            },
        );
        assert!(
            matches!(reply, Reply::Response(P::Hub { page, .. }) if page == TaxSection::Adjustments as i32),
            "{reply:?}"
        );
        assert_eq!(interview.draft, draft);
    }
}

#[test]
fn amt_ordinary_preferential_and_separate_phaseout_fixtures() {
    use crate::extended_rules::amt_tax;
    assert_eq!(amt_tax(200000, 0, 0, FederalStatus::Single, 0), 28574);
    assert_eq!(amt_tax(600000, 0, 0, FederalStatus::Single, 0), 151882);
    assert_eq!(amt_tax(100000, 50000, 40000, FederalStatus::Single, 0), 68);
    // Separate-return extra AMTI starts at $640,200 after exemption exhaustion.
    assert_eq!(amt_tax(640200, 0, 0, FederalStatus::Separate, 0), 176811);
    assert_eq!(amt_tax(640202, 0, 0, FederalStatus::Separate, 0), 176812);
}

#[test]
fn early_retirement_rates_are_added_after_nonrefundable_credits() {
    for (treatment, rate) in
        [(RetirementTreatment::Early, 10), (RetirementTreatment::EarlySimple, 25)]
    {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Retirement {
                data: retirement_source(FederalRetirement {
                    ordinary_distribution: Some(treatment),
                    ..native_retirement(
                        RetirementKind::Ira,
                        1_000_000,
                        (RetirementTaxable::Payer, 800_000),
                    )
                }),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 8000 * rate / 100);
        assert_eq!(return_line(&r, "23"), 8000 * rate);
        assert_eq!(r.tax, Some(return_line(&r, "22") + return_line(&r, "23")));
    }
}

#[test]
fn amt_old_rental_building_uses_separate_depreciation_and_passive_balances() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 0);
    let mut v = rental(10000, 0);
    v.buildings[0].in_service = 19900101;
    // Regular 27.5-year recovery has ended. AMT 40-year recovery continues:
    // 275,000 / 40 = 6,875. No prior suspended losses in either system.
    add(&mut d, income(FederalIncome::Rental { data: v }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 10000);
    assert_eq!(value(&r, "amt_rental_income"), 3125);
    assert_eq!(value(&r, "amt_rental_adjustment"), -6875);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 123125);
    let FederalIncome::Rental { data: v } = &mut d.income[1].income else { panic!() };
    v.rent = 0;
    v.expenses = vec![crate::extended_model::RentalExpense {
        treatment: BusinessExpenseKind::Ordinary,
        category: crate::extended_model::RentalExpenseCategory::Other { label: "Repairs".into() },
        amount: 2_000_000,
    }];
    let r = supported(&d);
    assert_eq!(value(&r, "rental_suspended_loss"), 5000);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 11875);
    assert_eq!(value(&r, "amt_rental_adjustment"), 0);
}

#[test]
fn retirement_rollover_preserves_gross_withholding_and_taxes_only_amount_kept() {
    for (kind, gross_line, taxable_line) in
        [(RetirementKind::Ira, "4a", "4b"), (RetirementKind::Pension, "5a", "5b")]
    {
        for (amount, early, taxable, additional) in
            [(1_000_000, true, 0, 0), (800_000, true, 2000, 200), (800_000, false, 2000, 0)]
        {
            let mut d = complete(FederalStatus::Single);
            add(
                &mut d,
                income(FederalIncome::Retirement {
                    data: retirement_source(FederalRetirement {
                        withholding: 200_000,
                        ordinary_distribution: Some(RetirementTreatment::Rollover {
                            destination: RolloverDestination::Ira,
                            amount,
                            early,
                        }),
                        ..native_retirement(kind, 1_000_000, (RetirementTaxable::Payer, 1_000_000))
                    }),
                }),
            );
            let r = supported(&d);
            assert_eq!(return_line(&r, gross_line), 1_000_000);
            assert_eq!(return_line(&r, taxable_line), taxable * 100);
            assert_eq!(value(&r, "agi"), 60000 + taxable);
            assert_eq!(value(&r, "early_distribution_tax"), additional);
            let mut without = d.clone();
            let FederalIncome::Retirement { data: v } = &mut without.income[1].income else {
                panic!()
            };
            edit_retirement(v).withholding = 0;
            assert_eq!(r.payments.unwrap() - supported(&without).payments.unwrap(), 200_000);
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

#[test]
fn direct_rollover_is_not_subtracted_twice_and_partial_rollover_is_explicit() {
    let mut d = complete(FederalStatus::Single);
    let v = FederalRetirement {
        ordinary_distribution: Some(RetirementTreatment::Rollover {
            destination: RolloverDestination::Ira,
            amount: 1_000_000,
            early: false,
        }),
        ..native_retirement(RetirementKind::Pension, 1_000_000, (RetirementTaxable::Payer, 0))
    };
    add(&mut d, income(FederalIncome::Retirement { data: retirement_source(v) }));
    assert_eq!(return_line(&supported(&d), "5b"), 0);
    let FederalIncome::Retirement { data: v } = &mut d.income[1].income else { panic!() };
    edit_retirement(v).ordinary_distribution = Some(RetirementTreatment::Rollover {
        destination: RolloverDestination::Ira,
        amount: 800_000,
        early: true,
    });
    assert_eq!(estimate(&d).state, ResultState::Unsupported);
}

#[test]
fn qualified_local_bus_type_6_keeps_full_fuel_rates_separate_from_type_5() {
    let local = FederalBusFuelUses {
        diesel_local_bus: Some(fuel_use(100_000, 30_000)),
        kerosene_local_bus: Some(fuel_use(100_000, 30_000)),
        emulsion_local_bus: Some(fuel_use(100_000, 30_000)),
        local_bus_qualified: Some(true),
        ..FederalBusFuelUses::default()
    };
    let direct_fuels = FederalFuelLines {
        diesel_intercity_local_bus: Some(fuel_use(100_000, 30_000)),
        emulsion_intercity_local_bus: Some(fuel_use(100_000, 30_000)),
        intercity_bus_no_waiver: Some(true),
        bus_uses: Some(local.clone()),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines { bus_uses: Some(local), ..FederalFuelLines::default() };
    assert!(direct_fuels.valid() && direct_fuels.qualified());
    assert!(partner_fuels.valid() && partner_fuels.qualified());
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), direct_fuels, partner_fuels, false);
    let report =
        super::support::assert::unit50_fuel::assert_fuel_payment(&draft, &baseline, 16_600);
    for (source, line, expected) in [
        ("business_2", "3a", 2_430),
        ("business_2", "3d", 1_700),
        ("business_2", "4a", 2_430),
        ("business_2", "14a", 3_210),
        ("partnership_3", "3a", 2_430),
        ("partnership_3", "4a", 2_430),
        ("partnership_3", "14a", 1_970),
    ] {
        assert_eq!(
            report
                .lines
                .iter()
                .find(|entry| entry.id == alloc::format!("{source}_fuel_{line}"))
                .unwrap()
                .amount,
            expected,
            "{source} line {line}"
        );
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let local = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    local.local_bus_qualified = Some(false);
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let local = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    local.local_bus_qualified = Some(true);
    local.diesel_local_bus.as_mut().unwrap().actual_cost = 1;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let local = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    local.diesel_local_bus.as_mut().unwrap().actual_cost = 30_000;
    local.diesel_local_bus.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
}

#[test]
fn aviation_gasoline_2a_2b_keep_propulsion_and_off_highway_use_separate() {
    use super::support::assert::unit50_retirement::assert_aviation_sources;
    let fuels = FederalFuelLines {
        aviation_gasoline_commercial: Some(fuel_use(100_000, 20_000)),
        aviation_gasoline_nonpropulsion_off_highway: Some(fuel_use(100_000, 20_000)),
        ..empty_fuel_lines()
    };
    let mut draft = assert_aviation_sources(fuels, [("2a", 1_500), ("2b", 1_930)], 6_900);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_commercial
        .as_mut()
        .unwrap()
        .not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.fuels.aviation_gasoline_commercial.as_mut().unwrap().not_claimed_elsewhere = true;
    claim.fuels.aviation_gasoline_nonpropulsion_off_highway.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}
#[test]
fn qcd_annual_owner_limit_combines_statements_and_keeps_gross_and_withholding() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19510101;
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            60000,
            RetirementTreatment::Ordinary,
        ),
    );
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            60000,
            RetirementTreatment::Ordinary,
        ),
    );
    add(
        &mut d,
        retirement_entry(Owner::Spouse, RetirementKind::Ira, 111000, RetirementTreatment::Ordinary),
    );
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20260301, 120000, 0), qcd(Owner::Spouse, 20260301, 111000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 222000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 9000);
    assert_eq!(return_line(&r, "4a"), 23_100_000);
    assert_eq!(return_line(&r, "4b"), 900_000);
    assert_eq!(value(&r, "agi"), 109000);
    assert_eq!(return_line(&r, "25d"), 686900);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn qcd_prior_deduction_adjustment_and_actual_half_birthday() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19560615;
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
        vec![qcd(Owner::Taxpayer, 20261215, 6500, 4000)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 2500);
    assert_eq!(value(&r, "qcd_cash_gifts"), 4000);
    assert_eq!(value(&r, "qcd_adjustment_carryforward"), 0);
    assert_eq!(value(&r, "agi"), 77500);
    d.adjustments.as_mut().unwrap().qualified_charity[0].payments[0].date = 20261214;
    unsupported(&d, "QCD");
    let q = &mut d.adjustments.as_mut().unwrap().qualified_charity[0];
    q.payments[0].date = 20261215;
    q.payments[0].amount = 600_000;
    q.prior_deductions = 1_000_000;
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 0);
    assert_eq!(value(&r, "qcd_taxpayer_adjustment_carry"), 4000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(10000, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 100);
    assert_eq!(value(&r, "qcd_taxpayer_adjustment_carry"), 4100);
}
#[test]
fn retirement_exception_dates_and_levy_amount_reduce_only_additional_tax() {
    for (facts, simple, expected) in [
        (RetirementException::Disability { began: 20260101 }, false, 0),
        (RetirementException::Death { died: 20260101 }, true, 0),
        (RetirementException::TerminalIllness { certified: 20260101 }, false, 0),
        (RetirementException::Levy { amount: 400_000 }, false, 600),
        (RetirementException::Levy { amount: 400_000 }, true, 1500),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Ira,
                10000,
                20260201,
                facts,
                simple,
            ),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), expected);
        assert_eq!(value(&r, "agi"), 70000);
        assert_eq!(return_line(&r, "4b"), 1_000_000);
        assert_eq!(restart(d.clone()).draft, d);
    }
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            20260201,
            RetirementException::TerminalIllness { certified: 20260202 },
            false,
        ),
    );
    assert_eq!(estimate(&d).state, ResultState::Unsupported);
}
#[test]
fn separation_exception_uses_separation_year_and_rejects_ira() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19711231;
    add(
        &mut d,
        retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            20260201,
            RetirementException::Separation { separated: 20260101, work: SeparationWork::Ordinary },
            false,
        ),
    );
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 0); // Age 55 later in the separation year.
    let FederalIncome::Retirement { data: v } = &mut d.income[1].income else { panic!() };
    edit_retirement(v).kind = RetirementKind::Ira;
    assert_eq!(estimate(&d).state, ResultState::Unsupported);
    let FederalIncome::Retirement { data: v } = &mut d.income[1].income else { panic!() };
    edit_retirement(v).kind = RetirementKind::Pension;
    edit_retirement(v).ordinary_distribution = Some(retirement_exception(
        20260201,
        RetirementException::Separation { separated: 20251231, work: SeparationWork::Ordinary },
        false,
    ));
    assert_eq!(estimate(&d).state, ResultState::Unsupported);
}
// IRC 72(t)(2)(A)(v): separation in or after the year age 55 is attained, and
// an age is attained the day before the birthday (Pub. 501). Born 1971-01-01
// the 55th birthday is 2026-01-01, so 55 is attained on 2025-12-31 and a
// 2025-12-31 separation qualifies; born 1971-01-02 it is attained on
// 2026-01-01, so the same separation does not.
#[test]
fn separation_exception_age_55_is_day_exact_for_a_january_1_birth() {
    for (birth, accepted) in [(19710101, true), (19710102, false)] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Pension,
                10000,
                20260201,
                RetirementException::Separation {
                    separated: 20251231,
                    work: SeparationWork::Ordinary,
                },
                false,
            ),
        );
        if accepted {
            assert_eq!(value(&supported(&d), "early_distribution_tax"), 0, "{birth}");
        } else {
            assert_eq!(estimate(&d).state, ResultState::Unsupported, "{birth}");
        }
    }
}
#[test]
fn current_ira_contributions_use_ordered_irs_worksheet_without_iteration() {
    let d = ira_contribution_distribution(80000, Answer::Yes);
    let r = supported(&d);
    // Worksheet 1-1: 7,500 / 50,000 * 10,000 = 1,500 preliminary exclusion.
    // MAGI 88,500 gives a $1,880 deduction and $5,620 nondeductible contribution.
    assert_eq!(value(&r, "ira_deduction"), 1880);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 5620);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 1500);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 4120);
    assert_eq!(value(&r, "taxpayer_ira_form_line13"), 1500);
    assert_eq!(value(&r, "taxpayer_ira_form_line15a"), 8500);
    assert!(r.lines.iter().all(|row| row.id != "taxpayer_ira_form_line6"));
    assert_eq!(value(&r, "agi"), 86620);
    assert_eq!(value(&r, "early_distribution_tax"), 850);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn current_ira_payment_timing_and_allocation_election_are_preserved() {
    let mut d = ira_contribution_distribution(76000, Answer::Yes);
    d.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 500000;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 4880);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 1500);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 1120);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 750001;
    unsupported(&d, "IRA basis");
}
#[test]
fn roth_conversion_is_taxable_but_not_a_regular_contribution_or_early_withdrawal() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 14000000, 0);
    add(
        &mut d,
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 100000, conversion(100000, true)),
    );
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 750000, Answer::Yes);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 240000);
    assert_eq!(value(&r, "roth_conversion_taxable"), 100000);
    assert_eq!(value(&r, "roth_contribution_magi"), 140000);
    assert_eq!(return_line(&r, "4a"), 10000000);
    assert_eq!(return_line(&r, "4b"), 10000000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn roth_magi_refigures_social_security_without_changing_tax_return_income() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19500101;
    d.income[0] = wage(1, Owner::Taxpayer, 9000000, 0);
    add(
        &mut d,
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 100000, conversion(100000, false)),
    );
    add(&mut d, income(social_security_net(24000000, 0)));
    let mut f = ira_facts(0, 750000, Answer::Yes);
    f.spouse = IraSpouse::Joint {
        facts: FederalIraContribution {
            nondeductible_election: 0,
            traditional: 0,
            roth: 0,
            covered_at_work: false,
            returned_next_year: vec![],
            excess: None,
            next_year_deposits: None,
            qtp_roth_lifetime_prior: 0,
        },
    };
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "roth_contribution_magi"), 237100);
    assert_eq!(value(&r, "agi"), 394000);
    assert_eq!(return_line(&r, "6b"), 20400000);
}

#[test]
fn roth_refigure_excess_business_loss_adds_back_the_excluded_foreign_business() {
    // The regular, AMT and Roth refigured excess business loss share one operand, so the Form
    // 2555 excluded business net and housing deduction count in the refigure as they do in line 8.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(220000, 20000)));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Residence {
        began: 20230115,
        ended: None,
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    };
    claim.housing = housing_claim(60000, ForeignHousingLocation::UnitedArabEmiratesDubai, None);
    set_claim(&mut d, claim);
    add(&mut d, wage(1, Owner::Taxpayer, 26_777_200, 0));
    add(&mut d, income(cash_business(-300_000)));
    add(
        &mut d,
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 100000, conversion(100000, false)),
    );
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 10000, Answer::Yes);
    // Roth MAGI 168,500 with the excess business loss 728 (not 167,772 without the add-back) is
    // above the phase-out end, so the $100 contribution is an excess.
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_IRA_ROTH_EXCESS_NEEDS_YEAR_END_VALUES")
    );
}

#[test]
fn qcd_current_deductions_stay_with_their_owner_and_exclude_roth() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19500101;
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        add(
            &mut d,
            retirement_entry(owner, RetirementKind::Ira, 10000, RetirementTreatment::Ordinary),
        );
    }
    let mut f = ira_facts(500_000, 100_000, Answer::No);
    f.spouse = IraSpouse::Joint { facts: ira_facts(200_000, 0, Answer::No).taxpayer };
    let a = d.adjustments.as_mut().unwrap();
    a.iras = f.clone();
    a.qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261215, 10000, 0), qcd(Owner::Spouse, 20261215, 10000, 1000)];
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7000);
    assert_eq!(value(&r, "qcd_taxpayer_exclusion"), 5000);
    assert_eq!(value(&r, "qcd_spouse_exclusion"), 7000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 8000);
    assert_eq!(value(&r, "agi"), 101000);
    // Coverage is allowed when both owners still qualify for full deductions.
    if let IraSpouse::Joint { facts } = &mut f.spouse {
        facts.covered_at_work = true;
    }
    d.adjustments.as_mut().unwrap().iras = f;
    add(&mut d, covered_wage(0, Owner::Spouse, 0, 0));
    let covered = supported(&d);
    assert_eq!(value(&covered, "ira_deduction"), 7000);
    assert_eq!(value(&covered, "qcd_taxpayer_exclusion"), 5000);
    assert_eq!(value(&covered, "qcd_spouse_exclusion"), 7000);
}

#[test]
fn qcd_covered_contribution_checks_magi_after_its_own_exclusion_adjustment() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    d.income.clear();
    add(&mut d, covered_wage(1, Owner::Taxpayer, 7_500_000, 650_000));
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261215, 10000, 0)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(860_000, 0, Answer::Yes);
    // Ignoring the current deduction's QCD adjustment would incorrectly use
    // $75,000 MAGI and allow all $8,600. The candidate actually produces $83,600.
    unsupported(&d, "QCD");
    d.income[0] = covered_wage(1, Owner::Taxpayer, 7_240_000, 650_000);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 8600);
    assert_eq!(value(&r, "qcd_exclusion"), 1400);
    assert_eq!(value(&r, "agi"), 72400);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 9_100_000, 650_000);
    unsupported(&d, "IRA contributions");
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qcd_nondeductible_contribution_uses_only_available_pretax_funds() {
    let mut d = qcd_with_basis(10000, 10000, 0, 5000, 0);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 9_100_000, 650_000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(500_000, 0, Answer::Yes);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "qcd_exclusion"), 10000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 5000);
    assert_eq!(return_line(&r, "4b"), 0);
    d.adjustments.as_mut().unwrap().ira_basis[0].year_end = 0;
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 5000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 5000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    // Contributions actually paid next year are not current-year distribution basis.
    d.adjustments.as_mut().unwrap().ira_basis[0].paid_next_year = 500_000;
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 10000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 5000);
}

#[test]
fn qcd_partial_deduction_with_exhausted_exclusion_verifies_its_adjustment() {
    let mut d = qcd_with_basis(10000, 10000, 0, 50000, 20000);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 7_500_000, 650_000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(860_000, 0, Answer::Yes);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 5160);
    assert_eq!(value(&r, "qcd_exclusion"), 0);
    assert_eq!(value(&r, "qcd_taxpayer_adjustment_carry"), 15160);
    assert_eq!(value(&r, "qcd_cash_gifts"), 10000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 3440);
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    assert_eq!(value(&r, "agi"), 79840);
    // Removing the old adjustment requires searching beyond the first candidate.
    d.adjustments.as_mut().unwrap().qualified_charity[0].prior_deductions = 0;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7400);
    assert_eq!(value(&r, "qcd_exclusion"), 2600);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 1200);
    assert_eq!(value(&r, "agi"), 75000);
}

#[test]
fn public_safety_separation_accepts_age_or_plan_service_and_preserves_income() {
    for (birth, service) in [(19761231, 0), (19800101, 25)] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Pension,
                10000,
                20260201,
                RetirementException::Separation {
                    separated: 20260101,
                    work: SeparationWork::PublicSafety { service_years: service, eligible: true },
                },
                false,
            ),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(return_line(&r, "5b"), 1_000_000);
        assert_eq!(value(&r, "agi"), 70000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn public_safety_separation_rejects_wrong_account_dates_service_and_eligibility() {
    for (birth, service, separated, kind, eligible) in [
        (19800101, 24, 20260101, RetirementKind::Pension, true),
        (19761231, 24, 20251231, RetirementKind::Pension, true),
        (19700101, 25, 20260202, RetirementKind::Pension, true),
        (19700101, 25, 20260101, RetirementKind::Ira, true),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                kind,
                10000,
                20260201,
                RetirementException::Separation {
                    separated,
                    work: SeparationWork::PublicSafety { service_years: service, eligible },
                },
                false,
            ),
        );
        unsupported(&d, "Income:");
    }
    // No on the employment and plan: the exception is not applied and the 10% tax applies.
    {
        let eligible = false;
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19700101;
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Pension,
                10000,
                20260201,
                RetirementException::Separation {
                    separated: 20260101,
                    work: SeparationWork::PublicSafety { service_years: 25, eligible },
                },
                false,
            ),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 1000);
    }
}

#[test]
fn qcd_phaseout_search_requires_exact_agreement_at_rounding_discontinuity() {
    let mut d = qcd_with_basis(10000, 10000, 0, 50000, 0);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 9_090_000, 650_000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(860_000, 0, Answer::Yes);
    // Zero deduction implies MAGI $90,900 and a $200 limit; $200 implies
    // MAGI $91,100 and a zero limit. Do not choose either inconsistent result.
    unsupported(&d, "QCD");
    d.income[0] = covered_wage(1, Owner::Taxpayer, 9_070_000, 650_000);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 200);
    assert_eq!(value(&r, "qcd_exclusion"), 9800);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 8400);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qcd_phaseout_search_uses_spouse_contribution_and_joint_income_band() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19500101;
    d.income[0] = wage(1, Owner::Taxpayer, 14_000_000, 650_000);
    add(&mut d, basis_distribution(Owner::Spouse, 10000, RetirementTreatment::Ordinary));
    // The spouse participated at work without taxable box-1 wages.
    add(&mut d, covered_wage(0, Owner::Spouse, 0, 0));
    let mut f = ira_facts(0, 0, Answer::No);
    f.spouse = IraSpouse::Joint { facts: ira_facts(860_000, 0, Answer::Yes).taxpayer };
    let a = d.adjustments.as_mut().unwrap();
    a.iras = f;
    a.ira_basis = vec![prior_ira_basis(Owner::Spouse, 0, 50000)];
    a.qualified_charity = vec![qcd(Owner::Spouse, 20261215, 10000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 2710);
    assert_eq!(value(&r, "qcd_spouse_exclusion"), 7290);
    assert_eq!(value(&r, "spouse_ira_basis_carryforward"), 5890);
    assert_eq!(value(&r, "agi"), 140000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qcd_joint_contributors_coordinate_both_deductions_and_coverage_bands() {
    for (wages, spouse_covered, deduction, taxpayer_exclusion, spouse_exclusion) in
        [(129980, Answer::Yes, 8800, 5600, 5600), (245000, Answer::No, 3240, 10000, 6760)]
    {
        let mut d = complete(FederalStatus::Joint);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
        d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19500101;
        d.income[0] = covered_wage(1, Owner::Taxpayer, wages * 100, 650_000);
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            add(&mut d, basis_distribution(owner, 10000, RetirementTreatment::Ordinary));
        }
        if spouse_covered == Answer::Yes {
            // The spouse participated at work without taxable box-1 wages.
            add(&mut d, covered_wage(0, Owner::Spouse, 0, 0));
        }
        let mut f = ira_facts(860_000, 0, Answer::Yes);
        f.spouse = IraSpouse::Joint { facts: ira_facts(860_000, 0, spouse_covered).taxpayer };
        let a = d.adjustments.as_mut().unwrap();
        a.iras = f;
        a.ira_basis = vec![
            prior_ira_basis(Owner::Taxpayer, 0, 50000),
            prior_ira_basis(Owner::Spouse, 0, 50000),
        ];
        a.qualified_charity =
            vec![qcd(Owner::Taxpayer, 20261215, 10000, 0), qcd(Owner::Spouse, 20261215, 10000, 0)];
        let r = supported(&d);
        assert_eq!(value(&r, "ira_deduction"), deduction);
        assert_eq!(value(&r, "qcd_taxpayer_exclusion"), taxpayer_exclusion);
        assert_eq!(value(&r, "qcd_spouse_exclusion"), spouse_exclusion);
        assert_eq!(value(&r, "agi"), wages);
        assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), taxpayer_exclusion - 1400);
        assert_eq!(value(&r, "spouse_ira_basis_carryforward"), spouse_exclusion - 1400);
        assert_eq!(restart(d.clone()).draft, d);
        d.income.reverse();
        assert_eq!(value(&supported(&d), "ira_deduction"), deduction);
    }
}

#[test]
fn ira_deduction_boundary_candidates_cover_every_integer_magi_pair() {
    use alloc::collections::BTreeSet;
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    // Different age limits and a non-$10 contribution test clamping as well.
    for (spouse_covered, election) in
        [(Answer::Yes, 0), (Answer::No, 0), (Answer::Yes, 123_456), (Answer::No, 123_456)]
    {
        let mut f = ira_facts(856_700, 0, Answer::Yes);
        f.taxpayer.nondeductible_election = election;
        f.spouse = IraSpouse::Joint { facts: ira_facts(723_400, 0, spouse_covered).taxpayer };
        let p = d.people.as_ref().unwrap();
        let s = d.setup.as_ref().unwrap();
        let compensation = [300000, 0];
        let candidates =
            crate::ira::deduction_candidates(&f, p, s, compensation, &Default::default(), [0, 0]);
        let actual: BTreeSet<_> = candidates.iter().map(|c| c.nondeductible).collect();
        let expected: BTreeSet<_> = (0..=252001)
            .map(|magi| {
                crate::ira::calculate(&f, p, s, compensation, magi, &Default::default(), [0, 0])
                    .unwrap()
                    .nondeductible
            })
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), candidates.len());
        assert!(candidates.len() <= 1722);
    }
}

#[test]
fn ira_nondeductible_election_is_explicit_and_preserves_contribution_limits() {
    let mut d = complete(FederalStatus::Single);
    let mut f = ira_facts(500_000, 0, Answer::No);
    f.taxpayer.nondeductible_election = f.taxpayer.traditional;
    d.adjustments.as_mut().unwrap().iras = f.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 5000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);
    f.taxpayer.traditional = 800_000;
    d.adjustments.as_mut().unwrap().iras = f;
    unsupported(&d, "IRA contributions");
}

#[test]
fn qcd_no_deduction_election_resolves_rounding_cycle_without_changing_default() {
    let mut d = qcd_with_basis(10000, 10000, 0, 50000, 0);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 9_090_000, 650_000);
    let mut f = ira_facts(860_000, 0, Answer::Yes);
    d.adjustments.as_mut().unwrap().iras = f.clone();
    unsupported(&d, "QCD");
    f.taxpayer.nondeductible_election = f.taxpayer.traditional;
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "qcd_exclusion"), 10000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 8600);
    assert_eq!(value(&r, "agi"), 90900);
}

#[test]
fn ira_election_is_owner_specific_and_coverage_still_limits_spouse() {
    let d = complete(FederalStatus::Joint);
    let mut f = ira_facts(750_000, 0, Answer::Yes);
    f.taxpayer.nondeductible_election = f.taxpayer.traditional;
    f.spouse = IraSpouse::Joint { facts: ira_facts(750_000, 0, Answer::No).taxpayer };
    let r = crate::ira::calculate(
        &f,
        d.people.as_ref().unwrap(),
        d.setup.as_ref().unwrap(),
        [300000, 0],
        247000,
        &Default::default(),
        [0, 0],
    )
    .unwrap();
    assert_eq!(r.deduction, 3750);
    assert_eq!(r.nondeductible, [7500, 3750]);
}

#[test]
fn owner_sep_changes_income_and_qbi_but_not_se_tax() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(sep_business(50000, 8000)));
    let r = supported(&d);
    let mut reference = d.clone();
    reference.income[1].income = cash_business(50000);
    let baseline = supported(&reference);
    assert_eq!(value(&r, "self_employment_tax"), value(&baseline, "self_employment_tax"));
    assert_eq!(value(&r, "agi"), value(&baseline, "agi") - 8000);
    assert_eq!(value(&r, "qbi_deduction"), value(&baseline, "qbi_deduction") - 1600);
    assert_eq!(return_line(&r, "10"), return_line(&baseline, "10") + 800000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn owner_sep_enforces_income_and_annual_caps_in_cents() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(sep_business(10000, 1858)));
    let r = supported(&d);
    assert_eq!(
        r.lines
            .iter()
            .find(|v| v.id == alloc::format!("business_{}_sep_limit", d.income[0].id))
            .unwrap()
            .amount,
        185860
    );
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessRetirement::Sep { contribution, .. } = &mut data.retirement else { unreachable!() };
    *contribution = 185861; // 20% of $10,000 minus $707 deductible SE tax is $1,858.60.
    unsupported(&d, "Business retirement");
    d.income[0].income = sep_business(1000000, 72000);
    supported(&d);
    d.income[0].income = sep_business(1000000, 72001);
    unsupported(&d, "Business retirement");
    d.income[0].income = sep_business(-1000, 1);
    unsupported(&d, "Business retirement");
}

#[test]
fn owner_sep_coordinates_health_income_limit_and_regular_ira_compensation() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    d.income.clear();
    let mut entry = health_business(10000, 9000, false);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { rate: 25.0, contribution: 100000 };
    add(&mut d, income(entry));
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{}_health_insurance", d.income[0].id)), 8293);
    assert_eq!(value(&r, &alloc::format!("business_{}_medical_premiums", d.income[0].id)), 707);
    d.income[0].income = sep_business(8000, 1000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(700000, 0, Answer::Yes);
    unsupported(&d, "IRA");
    d.income[0].income = cash_business(8000);
    supported(&d);
    // Health insurance does not reduce IRA compensation, unlike SEP contributions.
    d.income[0].income = health_business(10000, 9000, false);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    supported(&d);
}

#[test]
fn owner_sep_derives_plan_coverage_and_requires_supported_business_structure() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(sep_business(50000, 8000)));
    d.adjustments.as_mut().unwrap().iras = ira_facts(500000, 0, Answer::No);
    let derived = supported(&d);
    d.adjustments.as_mut().unwrap().iras = ira_facts(500000, 0, Answer::Yes);
    let explicit = supported(&d);
    assert_eq!(derived.tax, explicit.tax);
    assert_eq!(value(&derived, "ira_deduction"), value(&explicit, "ira_deduction"));
    add(&mut d, income(cash_business(10000)));
    unsupported(&d, "Business retirement");
}

#[test]
fn savings_penalty_is_separate_from_gross_interest_and_can_exceed_it() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(100, 300)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 59800);
    assert_eq!(return_line(&r, "2b"), 10000);
    assert_eq!(return_line(&r, "10"), 30000);
    assert_eq!(value(&r, "savings_withdrawal_penalty"), 300);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.remove(0);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -200);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
}

#[test]
fn savings_penalty_reduces_niit_and_ira_magi_without_reducing_compensation() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 25000000, 0);
    add(&mut d, income(savings_income(10000, 2000)));
    let r = supported(&d);
    assert_eq!(value(&r, "net_investment_income_tax"), 304);
    assert_eq!(value(&r, "agi"), 258000);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 8000000, 0);
    d.income[1].income = savings_income(2000, 1000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    assert_eq!(value(&supported(&d), "ira_deduction"), 7500);
    d.income[1].income = savings_income(2000, 0);
    assert_eq!(value(&supported(&d), "ira_deduction"), 6750);
}
#[test]
fn grace_and_current_w2_uses_actual_sources_and_one_plan_maximum() {
    let mut d = benefit_return(FederalStatus::Single);
    grace_and_current_benefits(&mut d);
    let mut missing = d.clone();
    missing.credits.as_mut().unwrap().care.benefits[0].wage = 9999;
    assert!(!missing.valid_stored());
    let mut repeated = d.clone();
    let mut entry = repeated.income[0].clone();
    entry.id = repeated.next_id;
    repeated.next_id += 1;
    repeated.income.push(entry);
    assert!(repeated.valid_stored());
    // A2025 grace amount is its own supplied fact; neither a repeat
    // eligibility answer nor service dates/balance reconciliation are stored.
    let mut partial = d.clone();
    partial.credits.as_mut().unwrap().care.grace[0].used_in_return_year = 100_000;
    assert!(partial.valid_stored());
    supported(&partial);
    care_costs(&mut d, 100_000);
    assert_eq!(value(&supported(&d), "care_benefits_excluded"), 1000);
}

#[test]
fn disability_pension_reports_wages_without_pension_or_payroll_tax() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income.clear();
    add(&mut d, income(disability_pension(17000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 17000);
    assert_eq!(return_line(&r, "1a"), 0);
    assert_eq!(return_line(&r, "1h"), 1700000);
    assert_eq!(return_line(&r, "1z"), 1700000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(return_line(&r, "25d"), 12000);
    assert_eq!(value(&r, "elderly_credit"), 38);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    let mut comparable = d.clone();
    comparable.income[0] = wage(d.income[0].id, Owner::Taxpayer, 1700000, 12000);
    assert_eq!(
        value(&r, "earned_income_credit"),
        value(&supported(&comparable), "earned_income_credit")
    );
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn disability_pension_is_earned_income_but_not_ira_compensation() {
    // Pub. 590-A: "Pension or annuity income" is not compensation, so a
    // disability pension on line 1h (Pub. 596: earned income for the EIC)
    // supports no IRA deduction; the $2,000 contribution is excess and the
    // deduction is zero (decision of 2026-10-09, defect fixed by the merge).
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income.clear();
    add(&mut d, income(disability_pension(17000)));
    d.adjustments.as_mut().unwrap().iras = ira_facts(200000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    // Alone, the contribution has no compensation behind it: an excess the
    // estimate asks the year-end values for.
    unsupported(&d, "IRA contributions");
    // Beside $5,000 of wages the same contribution is deductible: the wages
    // are compensation, the pension is not.
    let mut with_wages = d.clone();
    add(&mut with_wages, wage(0, Owner::Taxpayer, 500_000, 0));
    let r = supported(&with_wages);
    assert_eq!(return_line(&r, "1h"), 1_700_000);
    assert_eq!(value(&r, "ira_deduction"), 2000);
    assert_eq!(value(&r, "agi"), 20000);
    // A person who is not disabled cannot carry the disability facts.
    d.people.as_mut().unwrap().taxpayer.disabled = false;
    unsupported(&d, "Disability retirement");
}

#[test]
fn jury_fees_do_not_create_earned_income_credit_or_ira_compensation() {
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::JuryDuty { data: crate::JuryPay { received: 1000000, remitted: 0 } }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 10000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(10000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    unsupported(&d, "IRA");
}

#[test]
fn ira_education_exception_limits_housing_and_leaves_taxable_income_unchanged() {
    for (half_time, simple, expected) in
        [(true, false, 300), (false, false, 600), (true, true, 750)]
    {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Ira,
                10000,
                20260115,
                education_withdrawal(6000, 2000, half_time),
                simple,
            ),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 70000);
        assert_eq!(value(&r, "early_distribution_tax"), expected);
        assert_eq!(restart(d.clone()).draft, d);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        edit_retirement(data).kind = RetirementKind::Pension;
        unsupported(&d, "Income");
    }
}

#[test]
fn unemployed_insurance_exception_caps_taxable_amount_and_rejects_employer_plan() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            20261231,
            RetirementException::UnemploymentInsurance {
                premiums: 1200000,
                benefit_year: 2026,
                weeks: 12,
                reemployment: Reemployment::None,
                qualification: UnemploymentQualification::Received { confirmed: true },
            },
            true,
        ),
    );
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 0);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    edit_retirement(data).kind = RetirementKind::Pension;
    unsupported(&d, "Income");
}

#[test]
fn disability_pension_after_minimum_age_can_qualify_for_schedule_r_without_earned_income() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19650101;
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income.clear();
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        17000,
        RetirementTreatment::Ordinary,
    );
    let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
    edit_retirement(data).disability =
        DisabilityIncome::Qualifying { amount: 1700000, eligible: true };
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 17000);
    assert_eq!(return_line(&r, "1z"), 0);
    assert_eq!(return_line(&r, "5b"), 1700000);
    assert_eq!(value(&r, "elderly_credit"), 38);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().iras = ira_facts(10000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    unsupported(&d, "IRA");
}

#[test]
fn schedule_r_disability_pension_cannot_exceed_taxable_pension_or_use_an_ira() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        10000,
        RetirementTreatment::Ordinary,
    );
    let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
    edit_retirement(data).box_2a = 500000;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    edit_retirement(data).disability =
        DisabilityIncome::Qualifying { amount: 600000, eligible: true };
    add(&mut d, e);
    unsupported(&d, "Income");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    edit_retirement(data).disability =
        DisabilityIncome::Qualifying { amount: 500000, eligible: true };
    supported(&d);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    edit_retirement(data).kind = RetirementKind::Ira;
    unsupported(&d, "Income");
}

#[test]
fn retirement_medical_exception_uses_final_agi_without_requiring_itemization() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            20260601,
            RetirementException::Medical { expenses: 800000 },
            false,
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(value(&r, "early_distribution_tax"), 725);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn retirement_medical_allowance_is_shared_and_prioritizes_simple_rate() {
    let mut d = complete(FederalStatus::Single);
    for simple in [false, true] {
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Ira,
                10000,
                20260601,
                RetirementException::Medical { expenses: 500000 },
                simple,
            ),
        );
    }
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 80000);
    // $10,000 expenses less $6,000 AGI floor: $4,000 shelters the 25% withdrawal.
    assert_eq!(value(&r, "early_distribution_tax"), 2500);
    d.income.swap(1, 2);
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 2500);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception { facts, .. }) =
            &mut retirement_edit.ordinary_distribution
        else {
            unreachable!()
        };
        *facts = RetirementException::UnemploymentInsurance {
            premiums: 500000,
            benefit_year: 2026,
            weeks: 12,
            reemployment: Reemployment::None,
            qualification: UnemploymentQualification::Received { confirmed: true },
        };
    }
    unsupported(&d, "Retirement medical");
}

#[test]
fn qdro_exception_is_for_employer_plan_alternate_payee_not_ira() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            20260601,
            RetirementException::DomesticRelations,
            false,
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    edit_retirement(data).kind = RetirementKind::Ira;
    unsupported(&d, "Income");
}

#[test]
fn emergency_withdrawal_uses_vested_balance_floor_and_annual_dollar_cap() {
    for (balance, amount, tax) in
        [(1500, 500, 0), (1500, 1000, 50), (1000, 500, 50), (5000, 2000, 100)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, emergency_withdrawal(balance, amount, EmergencyHistory::NONE));
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), tax);
        assert_eq!(value(&r, "agi"), 60000 + amount);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn emergency_withdrawal_checks_prior_replenishment_and_single_annual_claim() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        emergency_withdrawal(
            5000,
            1000,
            EmergencyHistory { amount: 100000, repaid: 40000, contributed: 60000 },
        ),
    );
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 0);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception { facts, .. }) =
            &mut retirement_edit.ordinary_distribution
        else {
            unreachable!()
        };
        let RetirementException::Emergency { history, .. } = facts else { unreachable!() };
        *history = EmergencyHistory { amount: 100000, repaid: 40000, contributed: 59999 };
    }
    unsupported(&d, "Income");
    d.income.pop();
    add(&mut d, emergency_withdrawal(5000, 500, EmergencyHistory::NONE));
    add(&mut d, emergency_withdrawal(5000, 500, EmergencyHistory::NONE));
    unsupported(&d, "Income");
}

#[test]
fn early_distribution_tax_does_not_round_each_source_separately() {
    for treatment in [RetirementTreatment::Early, RetirementTreatment::EarlySimple] {
        let mut split = complete(FederalStatus::Single);
        let mut combined = split.clone();
        for _ in 0..50 {
            let mut e =
                retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 0, treatment.clone());
            let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
            edit_retirement(data).gross = 9;
            edit_retirement(data).box_2a = 9;
            edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
            add(&mut split, e);
        }
        let mut e = retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 0, treatment);
        let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
        edit_retirement(data).gross = 450;
        edit_retirement(data).box_2a = 450;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
        add(&mut combined, e);
        assert_eq!(
            value(&supported(&split), "early_distribution_tax"),
            value(&supported(&combined), "early_distribution_tax")
        );
    }
}

#[test]
fn early_distribution_rounds_form_lines_and_each_filer_tax() {
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut e = retirement_entry(owner, RetirementKind::Ira, 0, RetirementTreatment::Early);
        let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
        edit_retirement(data).gross = 450;
        edit_retirement(data).box_2a = 450;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
        add(&mut d, e);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "early_taxpayer_taxable"), 5);
    assert_eq!(value(&r, "early_spouse_taxable"), 5);
    assert_eq!(value(&r, "early_taxpayer_tax"), 1);
    assert_eq!(value(&r, "early_spouse_tax"), 1);
    assert_eq!(value(&r, "early_distribution_tax"), 2);
}

#[test]
fn early_distribution_rounds_taxable_and_exception_lines_before_subtraction() {
    let mut d = complete(FederalStatus::Single);
    let mut e = retirement_exception_entry(
        Owner::Taxpayer,
        RetirementKind::Ira,
        0,
        20260601,
        RetirementException::Levy { amount: 49 },
        false,
    );
    let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
    edit_retirement(data).gross = 450;
    edit_retirement(data).box_2a = 450;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(value(&r, "early_taxpayer_taxable"), 5);
    assert_eq!(value(&r, "early_taxpayer_exceptions"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 1);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception { facts, .. }) =
            &mut retirement_edit.ordinary_distribution
        else {
            unreachable!()
        };
        *facts = RetirementException::Levy { amount: 50 };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "early_taxpayer_exceptions"), 1);
    assert_eq!(value(&r, "early_taxpayer_subject"), 4);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}

#[test]
fn employer_plan_exceptions_preserve_taxable_income_and_reject_ira() {
    for facts in [RetirementException::EsopDividends, RetirementException::FederalPhasedRetirement]
    {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Pension,
                10000,
                20260601,
                facts,
                false,
            ),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 70000);
        assert_eq!(return_line(&r, "5b"), 1000000);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(restart(d.clone()).draft, d);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        edit_retirement(data).kind = RetirementKind::Ira;
        unsupported(&d, "Income");
    }
}

#[test]
fn automatic_enrollment_uses_election_date_and_actual_plan_window() {
    for (elected, window, accepted) in [
        (20260401, 90, true),
        (20260402, 90, false),
        (20260131, 30, true),
        (20260201, 30, false),
        (20251231, 90, false),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            retirement_exception_entry(
                Owner::Taxpayer,
                RetirementKind::Pension,
                2000,
                20260415,
                RetirementException::AutomaticEnrollment {
                    first_contribution: 20260101,
                    elected,
                    window_days: window,
                    eligible: true,
                },
                false,
            ),
        );
        if accepted {
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 62000);
            assert_eq!(value(&r, "early_distribution_tax"), 0);
            assert_eq!(restart(d.clone()).draft, d);
            let mut credit_claim = d.clone();
            credit_claim.credits.as_mut().unwrap().saver = Some(saver(100000, 0)).into();
            let credit_result = supported(&credit_claim);
            assert_eq!(value(&credit_result, "saver_automatic_withdrawals_taxpayer"), 2000);
            assert_eq!(value(&credit_result, "saver_credit"), 0);
            let FederalIncome::Retirement { data } = &mut d.income[1].income else {
                unreachable!()
            };
            {
                let mut retirement_edit = edit_retirement(data);
                let Some(RetirementTreatment::Exception {
                    facts: RetirementException::AutomaticEnrollment { eligible, .. },
                    ..
                }) = &mut retirement_edit.ordinary_distribution
                else {
                    unreachable!()
                };
                *eligible = false;
            }
            let r = supported(&d);
            assert!(value(&r, "early_distribution_tax") > 0);
        } else {
            unsupported(&d, "Income");
        }
    }
}

#[test]
fn governmental_457_preserves_income_and_taxes_only_qualified_rollover_source() {
    for (rollover_taxable, tax) in [(0, 0), (300000, 300), (900000, 900)] {
        let mut d = complete(FederalStatus::Single);
        let mut e = retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            20260601,
            RetirementException::Governmental457 { rollover_taxable, eligible: true },
            false,
        );
        let FederalIncome::Retirement { data } = &mut e.income else { unreachable!() };
        edit_retirement(data).box_2a = 900000;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
        add(&mut d, e);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 69000);
        assert_eq!(return_line(&r, "5a"), 1000000);
        assert_eq!(return_line(&r, "5b"), 900000);
        assert_eq!(value(&r, "early_distribution_tax"), tax);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn qualifying_graduate_stipend_supplies_ira_compensation_without_eic_or_se_tax() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![income(FederalIncome::Scholarship {
        data: Scholarship::GraduateStipend { received: 1000000, eligible: true },
    })];
    d.income[0].id = 1;
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 1000);
    assert_eq!(value(&r, "agi"), 9000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    d.income[0].income = scholarship(10000, 10000, 0);
    unsupported(&d, "IRA");
}
#[test]
fn enrolled_peace_corps_volunteer_abroad_uses_separate_statutory_service_path() {
    let mut d = complete(FederalStatus::Single);
    let mut home = official_duty_sale_source(false);
    let election = &mut home.facts_mut().service.as_mut().unwrap().election;
    election.service = HomeDutyKind::PeaceCorpsVolunteerAbroad;
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    assert_eq!(value(&supported(&d), "capital_gain"), 300000);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let election = &mut data.facts_mut().service.as_mut().unwrap().election;
    election.service = HomeDutyKind::PeaceCorpsEmployeeAbroad {
        duty: QualifiedHomeDuty { extended_orders: false, remote_or_quarters: true },
    };
    unsupported(&d, "Income:");
}
#[test]
fn reduced_home_exclusion_adds_separate_joint_periods_and_preserves_saved_facts() {
    let mut d = complete(FederalStatus::Joint);
    let mut home = home_sale_source(true);
    let answers = home.facts_mut();
    answers.reason = crate::HomeSaleReason::Event { reason: HomeSaleTrigger::PhysicianRecommended };
    answers.lived = crate::HomeLived::Days { days: 730 };
    answers.spouse = crate::HomeSpouse::Joint {
        lived: crate::HomeLived::Days { days: 365 },
        earlier_sale: crate::EarlierHomeSale::No,
    };
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    assert_eq!(value(&supported(&d), "capital_gain"), 175000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    data.facts_mut().spouse = crate::HomeSpouse::NotJoint;
    unsupported(&d, "Income:");
}

#[test]
fn state_refunds_from_distinct_prior_deduction_years_use_separate_benefits() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(
        &mut d,
        income(FederalIncome::GovernmentPayment {
            data: tax_refund_source(FederalTaxRefund {
                amount: 300000,
                withholding: 0,
                history: refund_history(),
            }),
        }),
    );
    let mut older = refund_history();
    let TaxRefundHistory::IncomeTax {
        deduction_year,
        income_paid,
        sales_available,
        salt_paid,
        salt_deducted,
        itemized,
        standard,
        ..
    } = &mut older
    else {
        unreachable!()
    };
    *deduction_year = 2024;
    *income_paid = 1200000;
    *sales_available = 600000;
    *salt_paid = 1200000;
    *salt_deducted = 1000000;
    *itemized = 3000000;
    *standard = 1500000;
    add(
        &mut d,
        income(FederalIncome::GovernmentPayment {
            data: tax_refund_source(FederalTaxRefund {
                amount: 300000,
                withholding: 0,
                history: older,
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 2000);
    assert_eq!(value(&r, "agi"), 62000);
    assert_eq!(
        value(&r, "alternative_minimum_taxable_income"),
        value(&before, "alternative_minimum_taxable_income")
    );
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    if let FederalIncome::GovernmentPayment { data } = &mut d.income[2].income
        && let TaxRefundHistory::IncomeTax { deduction_year, .. } = &mut data.history
    {
        *deduction_year = 2025;
        data.form.box_2_tax_year = Some(2025);
    }
    unsupported(&d, "Income:");
}
#[test]
fn social_lump_sum_election_reproduces_irs_worksheet_example() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2550000, 0);
    add(&mut d, income(social_security_net(1100000, 0)));
    let regular = supported(&d);
    assert_eq!(return_line(&regular, "6b"), 300000);
    d.adjustments.as_mut().unwrap().social_election =
        SocialElection::Compare { years: vec![social_prior_year()], complete: true };
    let r = supported(&d);
    assert_eq!(return_line(&r, "6a"), 1100000);
    assert_eq!(return_line(&r, "6b"), 250000);
    assert_eq!(value(&r, "social_lump_sum_taxable_reduction"), 500);
    assert_eq!(value(&r, "agi"), 28000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn social_lump_sum_prior_taxable_and_prior_election_are_not_counted_twice() {
    let mut year = social_prior_year();
    year.benefits = 1000000;
    year.agi = 4850000;
    year.taxable = 850000;
    year.refigured = 850000;
    year.received = 500000;
    assert_eq!(year.additional(), 4250);
    year.earlier_lumps = 200000;
    year.refigured = 1020000;
    year.received = 300000;
    assert_eq!(year.additional(), 2550);
    year.year = 1993;
    year.benefits = 0;
    year.earlier_lumps = 0;
    year.taxable = 0;
    year.refigured = 0;
    year.agi = 5000000;
    year.received = 1000000;
    assert_eq!(year.additional(), 5000);
    year.year = 1994;
    assert_eq!(year.additional(), 8500);
}
#[test]
fn social_lump_sum_regular_method_wins_and_missing_history_is_guarded() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(social_security_net(1000000, 0)));
    let mut year = social_prior_year();
    year.received = 1000000;
    year.agi = 10000000;
    d.adjustments.as_mut().unwrap().social_election =
        SocialElection::Compare { years: vec![year.clone()], complete: true };
    assert_eq!(return_line(&supported(&d), "6b"), 0);
    let SocialElection::Compare { complete, .. } =
        &mut d.adjustments.as_mut().unwrap().social_election
    else {
        panic!()
    };
    *complete = false;
    let regular = supported(&d);

    assert_eq!(return_line(&regular, "6b"), 0);
    d.adjustments.as_mut().unwrap().social_election =
        SocialElection::Compare { years: vec![year.clone(), year], complete: true };
    assert!(!d.valid_stored());
}

#[test]
fn social_lump_sum_uncovered_ira_deduction_precedes_final_benefit_comparison() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2550000, 0);
    add(&mut d, income(social_security_net(1100000, 0)));
    let a = d.adjustments.as_mut().unwrap();
    a.social_election =
        SocialElection::Compare { years: vec![social_prior_year()], complete: true };
    a.iras = ira_facts(100000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 1000);
    assert_eq!(return_line(&r, "6b"), 200000);
    assert_eq!(value(&r, "agi"), 26500);
    assert_eq!(value(&r, "social_lump_sum_taxable_reduction"), 500);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::Yes);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 2550000, 0);
    // Covered at work: Appendix B Worksheet 1 takes the benefits under the regular method for
    // IRA MAGI (B11 R13), so the election no longer needs a refusal; MAGI stays under the phaseout.
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 1000);
}
#[test]
fn social_lump_sum_roth_only_does_not_require_traditional_deduction_worksheet() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2550000, 0);
    add(&mut d, income(social_security_net(1100000, 0)));
    let a = d.adjustments.as_mut().unwrap();
    a.social_election =
        SocialElection::Compare { years: vec![social_prior_year()], complete: true };
    a.iras = ira_facts(0, 750000, Answer::Yes);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(return_line(&r, "6b"), 250000);
    assert_eq!(value(&r, "agi"), 28000);
}
#[test]
fn nonbusiness_personal_rent_reports_gross_income_and_separate_adjustment() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    add(&mut d, income(personal_rent(30000, 5000)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "8"), 3000000);
    assert_eq!(return_line(&r, "9"), 24000000);
    assert_eq!(return_line(&r, "10"), 500000);
    assert_eq!(value(&r, "agi"), 235000);
    assert_eq!(value(&r, "net_investment_income_tax"), 950);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 5000);
    assert_eq!(crate::interview::document_amount(&d.income[1].income, false), Some(2500000));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn personal_rent_does_not_supply_ira_compensation_and_losses_are_suspended() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(personal_rent(10000, 1000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 9000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(10000, 0, Answer::No);
    unsupported(&d, "IRA contributions");
    d.adjustments.as_mut().unwrap().iras = FederalIras::NONE;
    d.income[0].income = personal_rent(1000, 1001);
    assert_eq!(value(&supported(&d), "rental_suspended_loss"), 1);
}
#[test]
fn ssa_and_equivalent_railroad_lump_sums_share_one_prior_year_calculation() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2550000, 0);
    let mut ssa = income(social_security_net(900000, 0));
    ssa.label = "SSA-1099 current benefits".into();
    add(&mut d, ssa);
    let mut rrb = income(social_security_net(200000, 0));
    rrb.label = "RRB-1099 prior-year benefits".into();
    add(&mut d, rrb);
    d.adjustments.as_mut().unwrap().social_election =
        SocialElection::Compare { years: vec![social_prior_year()], complete: true };
    let r = supported(&d);
    assert_eq!(return_line(&r, "6a"), 1100000);
    assert_eq!(return_line(&r, "6b"), 250000);
    assert_eq!(value(&r, "social_lump_sum_taxable_reduction"), 500);
}
#[test]
fn pension_simplified_method_reproduces_irs_joint_annuity_example() {
    // Pub. 575 Bill Smith example, shifted to a 2026 starting date.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, pension_entry(pension_basis(), 14400));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 1440000);
    assert_eq!(return_line(&r, "5b"), 1320000);
    assert_eq!(value(&r, "pension_2_basis_recovery"), 1200);
    assert_eq!(value(&r, "pension_2_monthly_exclusion"), 100);
    assert_eq!(value(&r, "pension_2_remaining_basis"), 29800);
    assert_eq!(value(&r, "agi"), 73200);
    assert_eq!(r.payments, Some(662300));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn pension_tables_partial_year_and_fixed_period_use_contract_facts() {
    for (age, expected) in
        [(55, 360), (56, 310), (60, 310), (61, 260), (65, 260), (66, 210), (70, 210), (71, 160)]
    {
        assert_eq!(PensionDuration::Single { age }.payments(20260101), expected);
    }
    for (survivor_age, expected) in
        [(55, 410), (56, 360), (65, 360), (66, 310), (75, 310), (76, 260), (85, 260), (86, 210)]
    {
        assert_eq!(PensionDuration::Joint { age: 55, survivor_age }.payments(20260101), expected);
    }
    let mut b = pension_basis();
    b.started = 20261001;
    b.months = 3;
    b.cost = Some(1200000);
    b.history = PensionHistory::New { duration: PensionDuration::Fixed { payments: 120 } };
    assert!(b.valid());
    assert_eq!(b.excluded(500000), 30000);
    b.months = 4;
    assert!(!b.valid());
    b.months = 3;
}
#[test]
fn pension_prior_worksheet_preserves_monthly_recovery_and_exhausts_cost() {
    let mut b = pension_basis();
    b.started = 20250101;
    b.cost = Some(3100000);
    b.history = PensionHistory::Prior { monthly: 10000, recovered: 3050000 };
    let mut d = complete(FederalStatus::Single);
    add(&mut d, pension_entry(b.clone(), 14400));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 1390000);
    assert_eq!(value(&r, "pension_2_remaining_basis"), 0);
    b.history = PensionHistory::Prior { monthly: 10000, recovered: 3100000 };
    assert_eq!(b.excluded(1440000), 0);
    b.started = 19860702;
    b.cost = Some(50000);
    b.history = PensionHistory::Prior { monthly: 10000, recovered: 0 };
    assert!(b.valid());
    assert_eq!(b.excluded(1440000), 120000); // Pre-1987 recovery has no lifetime cap.
    b.started = 19860701;
    assert!(!b.valid());
}
#[test]
fn pension_simplified_method_rejects_ira_and_unconfirmed_eligibility() {
    let mut d = complete(FederalStatus::Single);
    let mut b = pension_basis();
    b.eligible = false;
    add(&mut d, pension_entry(b, 14400));
    // No: no cost recovered, the whole payment taxed, with a warning.
    let r = supported(&d);
    assert!(noted(&r, "WARN_RETIREMENT_BASIS_LEFT_OUT"), "{r:?}");
    if let FederalIncome::Retirement { data } = &mut d.income[1].income {
        edit_retirement(data).taxable =
            Some(RetirementTaxable::Simplified { data: pension_basis() });
        edit_retirement(data).kind = RetirementKind::Ira;
    }
    unsupported(&d, "Income:");
}
#[test]
fn railroad_no_remaining_cost_is_fully_taxable_and_separate_from_sseb() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    add(&mut d, railroad_pension(12000, 15000, None));
    add(&mut d, income(social_security_net(1000000, 0)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 1500000);
    assert_eq!(return_line(&r, "5b"), 1500000);
    assert_eq!(return_line(&r, "6a"), 1000000);
    assert_eq!(return_line(&r, "6b"), 850000);
    assert_eq!(value(&r, "agi"), 83500);
}

#[test]
fn nonqualified_annuity_enters_niit_but_qualified_pension_does_not() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        30000,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).box_2a = 2500000;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 3000000);
    assert_eq!(return_line(&r, "5b"), 2500000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "agi"), 235000);
    assert_eq!(value(&r, "net_investment_income_tax"), 950);
    assert_eq!(value(&r, "nonqualified_annuity_investment_income"), 25000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).kind = RetirementKind::Pension;
    assert_eq!(value(&supported(&d), "net_investment_income_tax"), 0);
}
#[test]
fn nonqualified_annuity_niit_uses_magi_threshold_and_early_tax_uses_taxable_amount() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 19000000, 0);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        30000,
        RetirementTreatment::Early,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).box_2a = 2500000;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 215000);
    assert_eq!(value(&r, "net_investment_income_tax"), 570);
    assert_eq!(value(&r, "early_distribution_tax"), 2500);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).taxable = Some(RetirementTaxable::Simplified { data: pension_basis() });
    unsupported(&d, "Income:");
}
#[test]
fn fixed_annuity_general_rule_flows_to_taxable_income_and_niit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        3000,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).taxable =
            Some(RetirementTaxable::GeneralAnnuity { data: fixed_annuity() });
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(value(&r, "annuity_2_expected_return"), 60000);
    assert_eq!(value(&r, "annuity_2_basis_recovery"), 1500);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 28500);
    assert_eq!(return_line(&r, "5a"), 300000);
    assert_eq!(return_line(&r, "5b"), 150000);
    assert_eq!(value(&r, "net_investment_income_tax"), 57);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn fixed_annuity_uses_three_decimal_ratio_and_original_cost_after_prior_recovery() {
    let mut a = fixed_annuity();
    a.cost = 5000000;
    a.periodic = 100000;
    a.duration = AnnuityDuration::Fixed { payments: 150 };
    assert_eq!(a.ratio(), 333);
    assert_eq!(a.excluded(), 399600);
    a.started = 20200101;
    a.recovered = 4950000;
    assert!(a.valid(1200000));
    assert_eq!(a.ratio(), 333); // Remaining basis must not replace original cost.
    assert_eq!(a.excluded(), 50000);
    a.recovered = a.cost;
    assert_eq!(a.excluded(), 0);
}
#[test]
fn fixed_annuity_partial_first_payment_and_statement_reconciliation() {
    let mut a = fixed_annuity();
    a.started = 20261015;
    a.received = 2;
    a.partial = 12500;
    assert!(a.valid(62500));
    assert_eq!(a.excluded(), 31250);
    assert!(!a.valid(62501));
    a.received = 3;
    assert!(!a.valid(87500)); // Three full months plus a partial exceed Oct–Dec.
    a.received = 2;
    a.recovered = 1;
    assert!(!a.valid(62500)); // No prior-year recovery for a 2026 start.
    a.recovered = 0;
    a.duration = AnnuityDuration::Fixed { payments: 12 };
    assert!(!a.valid(62500));
}
#[test]
fn fixed_annuity_general_rule_requires_eligible_nonqualified_contract() {
    let mut d = complete(FederalStatus::Single);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        3000,
        RetirementTreatment::Ordinary,
    );
    let mut a = fixed_annuity();
    a.eligible = false;
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, e);
    assert!(noted(&supported(&d), "WARN_RETIREMENT_BASIS_LEFT_OUT"));
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).kind = RetirementKind::Pension;
    edit_retirement(data).taxable =
        Some(RetirementTaxable::GeneralAnnuity { data: fixed_annuity() });
    unsupported(&d, "Income:");
}

#[test]
fn general_annuity_preserves_fractional_cents_until_annual_exclusion() {
    // Pub. 939 part-year example: age 61, $22,050 cost, $125 x 3 payments.
    let mut a = fixed_annuity();
    a.started = 20261001;
    a.cost = 2205000;
    a.periodic = 12500;
    a.received = 3;
    a.duration = AnnuityDuration::SingleLife { age: 61, delay: 1, guaranteed: 0 };
    assert!(a.valid(37500));
    assert_eq!(a.expected(), 3495000);
    assert_eq!(a.ratio(), 631);
    assert_eq!(a.excluded(), 23663);
    // A rounding difference can cross the final whole-dollar threshold.
    a.started = 20260101;
    a.cost = 595608;
    a.periodic = 10001;
    a.received = 12;
    assert!(a.valid(120012));
    assert_eq!(a.ratio(), 213);
    assert_eq!(a.excluded(), 25563);
    assert_eq!(crate::kernel::ratio::dollars(120012 - a.excluded()), 944);
    // Premature per-payment rounding would instead report $945 taxable.
}

#[test]
fn general_annuity_cola_preserves_original_exclusion_irs_example() {
    // Pub. 939: $147 original payment becomes $166; $396.90 stays tax-free.
    let mut a = fixed_annuity();
    a.started = 20250101;
    a.cost = 793800;
    a.periodic = 14700;
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 0 };
    a.increase = (16600 - 14700) * 12;
    a.recovered = 36383;
    assert!(a.valid(199200));
    assert_eq!(a.ratio(), 225);
    assert_eq!(a.excluded(), 39690);
    assert!(!a.valid(176400));
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1992,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 199200);
    assert_eq!(return_line(&r, "5b"), 159500);
    assert_eq!(value(&r, "net_investment_income_tax"), 61);
    assert_eq!(restart(d.clone()).draft, d);
    a.increase = 100;
    a.received = 0;
    assert!(!a.valid(100));
}
#[test]
fn variable_annuity_irs_new_contract_and_shortfall_election() {
    let mut a = variable_annuity();
    assert!(a.valid());
    assert_eq!(a.annual(), 60000);
    assert_eq!(a.excluded(92000), 60000);
    a.started = 20240101;
    a.history = VariableHistory::Prior {
        annual: 60000,
        recovered: 110000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 50000,
            remaining: VariablePayments::Life { age: 67, delay: 6 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 60543);
    assert_eq!(120000 - a.excluded(120000), 59457);
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1200,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 120000);
    assert_eq!(return_line(&r, "5b"), 59500);
    assert_eq!(value(&r, "net_investment_income_tax"), 23);
    assert_eq!(value(&r, "variable_annuity_2_election"), 12000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn variable_annuity_fixed_term_uses_installments_not_lifetime_table() {
    let mut a = variable_annuity();
    a.history = VariableHistory::New { expected: VariablePayments::Fixed { payments: 10 } };
    assert!(a.valid());
    assert_eq!(a.annual(), 120000);
    a.started = 20250101;
    a.history = VariableHistory::Prior {
        annual: 120000,
        recovered: 30000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 30000,
            remaining: VariablePayments::Fixed { payments: 9 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 130000);
    a.history = VariableHistory::Prior {
        annual: 120000,
        recovered: 120000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 120000,
            remaining: VariablePayments::Fixed { payments: 9 },
        }),
    };
    assert!(!a.valid()); // No shortfall to refigure.
}
#[test]
fn variable_annuity_rejects_invalid_history_and_uncovered_contracts() {
    let mut a = variable_annuity();
    a.history = VariableHistory::New { expected: VariablePayments::Fixed { payments: 1 } };
    assert!(!a.valid());
    a = variable_annuity();
    a.history = VariableHistory::New { expected: VariablePayments::Life { age: 70, delay: 6 } };
    assert!(!a.valid());
    for (kind, eligible) in [(RetirementKind::Pension, true), (RetirementKind::Nonqualified, false)]
    {
        let mut d = complete(FederalStatus::Single);
        let mut e = retirement_entry(Owner::Taxpayer, kind, 920, RetirementTreatment::Ordinary);
        a = variable_annuity();
        a.eligible = eligible;
        if let FederalIncome::Retirement { data } = &mut e.income {
            edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
        }
        add(&mut d, e);
        if !eligible {
            // No: no cost recovered, the whole payment taxed.
            assert!(noted(&supported(&d), "WARN_RETIREMENT_BASIS_LEFT_OUT"));
        } else {
            unsupported(&d, "Income:");
        }
    }
}

#[test]
fn variable_annuity_lifetime_multiple_adjusts_for_annual_payment_timing() {
    let mut a = variable_annuity();
    for (delay, expected) in
        [(0, 58537), (1, 58537), (2, 58824), (6, 60000), (7, 60000), (12, 61538)]
    {
        a.history = VariableHistory::New { expected: VariablePayments::Life { age: 65, delay } };
        assert!(a.valid());
        assert_eq!(a.annual(), expected);
    }
    a.started = 20240101;
    a.history = VariableHistory::Prior {
        annual: 60000,
        recovered: 110000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 50000,
            remaining: VariablePayments::Life { age: 67, delay: 12 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 60559); // $100 / (18.4 - 0.5).
}
#[test]
fn fixed_annuity_cost_above_expected_return_uses_full_exclusion() {
    // 26 CFR 1.72-4(d)(2): cost >= expected return gives a 100% ratio.
    let mut a = fixed_annuity();
    a.cost = 7000000; // Expected return is $60,000.
    assert!(a.valid(300000));
    assert_eq!(a.ratio(), 1000);
    assert_eq!(a.excluded(), 300000);
    a.started = 20200101;
    a.recovered = a.cost - 10000;
    a.increase = 1000;
    assert!(a.valid(301000));
    assert_eq!(a.excluded(), 10000); // Basis cap survives; increases remain taxable.
}

#[test]
fn variable_monthly_first_year_uses_annual_aggregate_and_prorates() {
    // 1.72-4(d)(3)(i): $600 annual allowance, 7 monthly installments => $350.
    let mut a = variable_annuity();
    a.started = 20260601;
    a.period = 20260601;
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 7;
    a.history = VariableHistory::New { expected: VariablePayments::Life { age: 65, delay: 1 } };
    assert!(a.valid());
    assert_eq!(a.annual(), 60000);
    assert_eq!(a.excluded(92000), 35000);
    assert_eq!(a.excluded(20000), 20000);
    a.payments = 8;
    assert!(!a.valid());
    a.payments = 7;
    let mut d = complete(FederalStatus::Single);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        920,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
    }
    add(&mut d, e);
    assert_eq!(return_line(&supported(&d), "5b"), 57000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn variable_monthly_election_uses_annual_shortfall_and_remaining_years() {
    let mut a = variable_annuity();
    a.started = 20250101;
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 12;
    a.history = VariableHistory::Prior {
        annual: 120000,
        recovered: 30000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 30000,
            remaining: VariablePayments::Fixed { payments: 108 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 130000); // $900 shortfall / 9 remaining years.
    assert_eq!(a.excluded(180000), 130000);
    a.payments = 11;
    assert!(a.valid());
    assert!(!a.schedule_valid(None)); // Partial continuing schedules need the owner's death facts.
    assert!(a.schedule_valid(Some(20261130)));
}
#[test]
fn variable_joint_election_and_survivor_continuation_flow_to_return() {
    let mut a = variable_annuity();
    a.age = 68;
    a.started = 20240101;
    a.cost = 13200000;
    a.history = VariableHistory::Prior {
        annual: 600000,
        recovered: 900000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 300000,
            remaining: VariablePayments::Joint { age: 70, survivor_age: 67, delay: 6 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 613636); // $3,000 shortfall / 22.0 years.
    a.history = VariableHistory::Prior { annual: 600000, recovered: 900000, refigure: None };
    assert_eq!(a.annual(), 600000); // First death alone never resets the allowance.
    let mut d = complete(FederalStatus::Single);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        9000,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
    }
    add(&mut d, e);
    assert_eq!(return_line(&supported(&d), "5b"), 300000);
    assert_eq!(restart(d.clone()).draft, d);
    a.history = VariableHistory::Prior {
        annual: 600000,
        recovered: 900000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 300000,
            remaining: VariablePayments::Life { age: 67, delay: 6 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 616304); // New election uses the living survivor's 18.4 years.
}

#[test]
fn variable_annuity_first_payment_can_cover_prior_year_period() {
    let mut a = variable_annuity();
    a.started = 20250701;
    a.period = 20250701;
    a.history = VariableHistory::New { expected: VariablePayments::Life { age: 65, delay: 12 } };
    assert!(a.valid()); // Annual period ends with a July 2026 first payment.
    assert_eq!(a.annual(), 61538);
    assert_eq!(a.recovered(), 0);
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 12;
    a.started = 20251201;
    a.period = 20251201;
    a.history = VariableHistory::New { expected: VariablePayments::Life { age: 65, delay: 1 } };
    assert!(a.valid());
    assert_eq!(a.excluded(100000), 60000);
    a.started = 20251101;
    a.period = 20251101;
    assert!(!a.valid()); // Would require a delayed or missed monthly payment.
}
#[test]
fn variable_annuity_continuing_period_in_arrears_preserves_election_facts() {
    let mut a = variable_annuity();
    a.started = 20230701;
    a.period = 20250701;
    a.history = VariableHistory::Prior {
        annual: 60000,
        recovered: 50000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 50000,
            remaining: VariablePayments::Life { age: 67, delay: 12 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 60559);
    let mut d = complete(FederalStatus::Single);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1200,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
    }
    add(&mut d, e);
    assert_eq!(return_line(&supported(&d), "5b"), 59400);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn final_annuity_loss_uses_linked_income_recovery_and_tax_free_refunds() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died = death(20260615);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        10000,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).box_2a = 700000;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    }
    add(&mut d, e);
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: 20200101,
        cost: 5000000,
        recovered: 1000000,
        refunds: 200000,
        eligible: true,
    });
    let mut r = estimate(&d);
    assert_eq!(crate::federal_rules::final_annuity_losses(&d, &mut r), 35000);
    assert_eq!(value(&r, "annuity_loss_2"), 35000);
    assert_eq!(restart(d.clone()).draft, d);
    supported(&d); // The final-return facts are the person's own answers.
    d.deductions.as_mut().unwrap().annuity_losses[0].refunds = 3700000;
    assert_eq!(crate::federal_rules::final_annuity_losses(&d, &mut r), 0);
}
#[test]
fn final_annuity_loss_requires_final_annuitant_and_eligible_contract() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            0,
            RetirementTreatment::Ordinary,
        ),
    );
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: 20200101,
        cost: 5000000,
        recovered: 1000000,
        refunds: 0,
        eligible: true,
    });
    let mut r = estimate(&d);
    assert_eq!(crate::federal_rules::final_annuity_losses(&d, &mut r), 0); // Living owner.
    d.people.as_mut().unwrap().taxpayer.died = death(20260615);
    assert_eq!(crate::federal_rules::final_annuity_losses(&d, &mut r), 40000);
    d.deductions.as_mut().unwrap().annuity_losses[0].started = 19860701;
    assert_eq!(crate::federal_rules::final_annuity_losses(&d, &mut r), 0);
    d.deductions.as_mut().unwrap().annuity_losses[0].started = 19860702;
    // No claims no deduction and refuses nothing.
    {
        let answer = false;
        d.deductions.as_mut().unwrap().annuity_losses[0].eligible = answer;
        let mut r = estimate(&d);
        r.issues.clear();
        r.review_notes.clear();
        assert_eq!(crate::federal_rules::final_annuity_losses(&d, &mut r), 0);
        assert!(r.issues.is_empty(), "{:?}", r.issues);
    }
}
#[test]
fn ordinary_final_return_uses_full_deduction_and_unrecovered_annuity_cost() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 0 });
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        10000,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        edit_retirement(data).box_2a = 700000;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    }
    add(&mut d, e);
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: 20200101,
        cost: 5000000,
        recovered: 1000000,
        refunds: 200000,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "unrecovered_annuity_deduction"), 35000);
    assert_eq!(return_line(&r, "5b"), 700000);
    assert_eq!(value(&r, "agi"), 67000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn joint_final_return_for_two_decedents_keeps_separate_age_cutoffs() {
    let mut d = complete(FederalStatus::Joint);
    let p = d.people.as_mut().unwrap();
    p.taxpayer.birth_date = 19610615;
    p.taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260110, ordinary: true, estate_medical: 0 });
    let spouse = p.spouse.as_mut().unwrap();
    spouse.birth_date = 19610915;
    spouse.died =
        Some(FinalReturn { bonds: None, date: 20261010, ordinary: true, estate_medical: 0 });
    assert_eq!(
        crate::federal_rules::standard_deduction(d.setup.as_ref().unwrap(), p, 100000, false),
        33850
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 100000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn final_bond_annual_method_subtracts_previous_reporting_and_enters_niit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    d.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
        date: 20260615,
        ordinary: true,
        bonds: Some(FinalBonds {
            cost: 50000,
            value: 59400,
            reported: 4000,
            reporting: FinalBondReporting::Annual,
            eligible: true,
        }),
        estate_medical: 0,
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 5400);
    assert_eq!(value(&r, "net_investment_income_tax"), 2);
    let b = d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().bonds.as_mut().unwrap();
    b.reported = 9401;
    assert!(!b.valid());
    b.reported = 4000;
    b.eligible = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_FINAL_BONDS_LEFT_OUT"));
    assert!(r.lines.iter().all(|l| l.id != "final_bond_interest_0"));
}

#[test]
fn independent_service_and_capital_businesses_separate_earnings() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(cash_business(10000)));
    add(&mut d, income(capital_business(20000)));
    add(&mut d, wage(0, Owner::Taxpayer, 5000000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 27300);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn two_special_joint_children_have_separate_deemed_exclusions_and_credits() {
    let d = two_special_joint_adoption_return();
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_child_1_benefits_excluded"), 17_670);
    assert_eq!(value(&r, "adoption_child_2_benefits_excluded"), 17_670);
    assert_eq!(value(&r, "adoption_child_1_benefits_taxable"), -7670);
    assert_eq!(value(&r, "adoption_child_2_benefits_taxable"), -9670);
    assert_eq!(value(&r, "adoption_child_1_special_needs_deemed_excess"), 7670);
    assert_eq!(value(&r, "adoption_child_2_special_needs_deemed_excess"), 9670);
    assert_eq!(value(&r, "adoption_joint_special_needs_deemed_excess"), 17_340);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 35_340); // Form 8839 line 30.
    assert_eq!(value(&r, "adoption_wage_adjustment"), -17_340); // Form 8839 line 31.
    assert_eq!(value(&r, "adoption_exclusion_magi"), 138_000);
    assert_eq!(value(&r, "adoption_child_1_employer_taxable_taxpayer"), 0);
    assert_eq!(value(&r, "adoption_child_2_employer_taxable_spouse"), 0);
    assert_eq!(return_line(&r, "1a"), 12_000_000);
    assert_eq!(return_line(&r, "1f"), -1_734_000);
    assert_eq!(return_line(&r, "25a"), 1_200_000);
    assert_eq!(value(&r, "agi"), 102_660);
    assert_eq!(value(&r, "adoption_refundable_credit"), 10_240);
    assert_eq!(
        value(&r, "adoption_refundable_credit")
            + value(&r, "adoption_nonrefundable_credit")
            + value(&r, "adoption_carryforward"),
        35_340 // Separate $17,670 special-needs credit for each child.
    );
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);
}

#[test]
fn ordinary_and_special_joint_children_use_separate_form_8839_limits() {
    let d = ordinary_and_special_joint_adoption_return();
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_child_1_benefits_excluded"), 10_000);
    assert_eq!(value(&r, "adoption_child_2_benefits_excluded"), 17_670);
    assert_eq!(value(&r, "adoption_child_2_benefits_taxable"), -9670);
    assert_eq!(value(&r, "adoption_joint_special_needs_deemed_excess"), 9670);
    assert_eq!(value(&r, "adoption_child_2_employer_excluded_taxpayer"), 5000);
    assert_eq!(value(&r, "adoption_child_2_employer_excluded_spouse"), 3000);
    assert_eq!(value(&r, "adoption_child_2_employer_taxable_taxpayer"), 0);
    assert_eq!(value(&r, "adoption_child_2_employer_taxable_spouse"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 27_670); // Form 8839 line 30.
    assert_eq!(value(&r, "adoption_wage_adjustment"), -9670); // Form 8839 line 31.
    assert_eq!(value(&r, "adoption_exclusion_magi"), 138_000);
    assert_eq!(return_line(&r, "1a"), 12_000_000);
    assert_eq!(return_line(&r, "1f"), -967_000);
    assert_eq!(return_line(&r, "25a"), 1_200_000);
    assert_eq!(value(&r, "agi"), 110_330);
    assert_eq!(value(&r, "adoption_refundable_credit"), 10_240);
    assert_eq!(
        value(&r, "adoption_refundable_credit")
            + value(&r, "adoption_nonrefundable_credit")
            + value(&r, "adoption_carryforward"),
        25_670 // $8,000 ordinary expenses plus separate $17,670 special-needs credit.
    );
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);

    let mut swapped = d.clone();
    swapped.credits.as_mut().unwrap().adoptions.swap(0, 1);
    let reordered = supported(&swapped);
    assert_eq!(reordered.return_lines, r.return_lines);
    assert_eq!(value(&reordered, "adoption_child_1_benefits_taxable"), -9670);
    assert_eq!(value(&reordered, "adoption_child_2_benefits_excluded"), 10_000);
}

#[test]
fn special_needs_plan_can_exclude_more_than_current_benefits() {
    let mut d = complete(FederalStatus::Single);
    employer_adoption(&mut d, 0, 0, true);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 60000);
    assert_eq!(return_line(&r, "1f"), -1767000);
    assert_eq!(return_line(&r, "1z"), 4233000);
    assert_eq!(value(&r, "agi"), 42330);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert!(value(&r, "adoption_carryforward") > 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_return_can_use_owned_prior_credit_without_current_exception() {
    let mut d = complete(FederalStatus::Separate);
    d.credits.as_mut().unwrap().adoption_carry.push(AdoptionCarry {
        year: 2025,
        amount: 300000,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 3000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    assert_eq!(value(&r, "adoption_carryforward"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.credits.as_mut().unwrap().adoption_carry[0].eligible = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_ADOPTION_CARRY_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 0);
}

#[test]
fn public_safety_shares_one_limit_per_officer_across_plans() {
    for status in [FederalStatus::Single, FederalStatus::Joint] {
        let mut d = complete(status);
        for (i, owner) in [
            Owner::Taxpayer,
            if status == FederalStatus::Joint { Owner::Spouse } else { Owner::Taxpayer },
        ]
        .into_iter()
        .enumerate()
        {
            add(
                &mut d,
                retirement_entry(
                    owner,
                    RetirementKind::Pension,
                    10000,
                    RetirementTreatment::Ordinary,
                ),
            );
            let id = d.income.last().unwrap().id;
            public_safety(
                &mut d,
                id,
                4000,
                if i == 0 && status == FederalStatus::Single { 2000 } else { 4000 },
                PublicSafetyCoverage::Medical,
            );
        }
        let r = supported(&d);
        assert_eq!(
            value(&r, "public_safety_2_excluded"),
            if status == FederalStatus::Joint { 3000 } else { 1000 }
        );
        assert_eq!(value(&r, "agi"), if status == FederalStatus::Joint { 114000 } else { 77000 });
    }
}

#[test]
fn older_qualified_general_rule_pension_is_not_net_investment_income() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19280101;
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    add(&mut d, old_general_entry(RetirementKind::Pension, old_general_annuity(19930101, 500000)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 1149600);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    edit_retirement(data).kind = RetirementKind::Nonqualified;
    assert_eq!(value(&supported(&d), "net_investment_income_tax"), 437);
}

#[test]
fn general_rule_qualified_plan_date_and_method_guards_remain_explicit() {
    for (start, eligible) in [(19961119, true), (19961118, false)] {
        let mut d = complete(FederalStatus::Single);
        let mut data = old_general_annuity(start, 500000);
        data.eligible = eligible;
        add(&mut d, old_general_entry(RetirementKind::Pension, data));
        if !eligible {
            // No: no cost recovered, the whole payment taxed.
            assert!(noted(&supported(&d), "WARN_RETIREMENT_BASIS_LEFT_OUT"));
        } else {
            unsupported(&d, "Income");
        }
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, old_general_entry(RetirementKind::Pension, old_general_annuity(19961118, 500000)));
    supported(&d);
    let mut too_old = old_general_annuity(19860701, 0);
    assert!(!too_old.valid(1200000));
    too_old.started = 19870101;
    too_old.recovered = too_old.cost + 1;
    assert!(!too_old.valid(1200000));
}

#[test]
fn immediate_annuity_exception_uses_original_contract_dates_and_keeps_niit() {
    for (purchased, started) in [(20250615, 20260615), (20200229, 20210228)] {
        let d = nonqualified_exception(immediate_annuity(purchased, started));
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(return_line(&r, "5b"), 500000);
        assert_eq!(value(&r, "net_investment_income_tax"), 190);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn immediate_annuity_exception_rejects_late_starts_and_ineligible_contracts() {
    for (purchased, started) in [
        (20250614, 20260615),
        (20200229, 20210301),
        (20260615, 20260614),
        (20260615, 20260616),
        (20100101, 20260101), // Replacement of an older deferred contract does not reset purchase.
    ] {
        unsupported(&nonqualified_exception(immediate_annuity(purchased, started)), "Income:");
    }
    // A contract that is not single-premium or not periodic: the exception
    // is not applied, so the additional tax applies.
    let applied = value(
        &supported(&nonqualified_exception(immediate_annuity(20250101, 20260101))),
        "early_distribution_tax",
    );
    for (single_premium, periodic) in [(false, true), (true, false)] {
        let r = supported(&nonqualified_exception(RetirementException::ImmediateAnnuity {
            purchased: 20250101,
            started: 20260101,
            single_premium,
            periodic,
        }));
        assert!(value(&r, "early_distribution_tax") > applied, "{r:?}");
    }
    for kind in [RetirementKind::Ira, RetirementKind::Pension] {
        let mut d = nonqualified_exception(immediate_annuity(20250101, 20260101));
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        edit_retirement(data).kind = kind;
        unsupported(&d, "Income:");
    }
}

#[test]
fn deferred_annuity_withdrawal_matches_irs_earnings_first_example() {
    let d = deferred_withdrawal(16000, 10000);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 700000);
    assert_eq!(return_line(&r, "5b"), 600000);
    assert_eq!(value(&r, "agi"), 216000);
    assert_eq!(value(&r, "early_distribution_tax"), 600);
    assert_eq!(value(&r, "net_investment_income_tax"), 228);
    assert_eq!(value(&r, "annuity_2_taxable_withdrawal"), 6000);
    assert_eq!(value(&r, "annuity_2_basis_recovery"), 1000);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 9000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn deferred_annuity_limits_taxable_earnings_and_preserves_disability_exception() {
    for (cash, investment, taxable, recovery, remaining) in [
        (30000, 10000, 7000, 0, 10000),
        (10000, 10000, 0, 7000, 3000),
        (10000, 12000, 0, 7000, 5000),
    ] {
        let d = deferred_withdrawal(cash, investment);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), taxable * 100);
        assert_eq!(value(&r, "annuity_2_basis_recovery"), recovery);
        assert_eq!(value(&r, "annuity_2_remaining_basis"), remaining);
    }
    let mut d = deferred_withdrawal(16000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).ordinary_distribution = Some(retirement_exception(
        20260615,
        RetirementException::Disability { began: 20260101 },
        false,
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "net_investment_income_tax"), 228);
}

#[test]
fn deferred_annuity_taxes_unconfirmed_and_rejects_wrong_accounts_and_bad_values() {
    // No: earnings-first recovery is left out; the whole cash is taxed.
    let mut d = deferred_withdrawal(16000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTaxable::DeferredWithdrawal {
            data: crate::AnnuityDistribution { eligible, .. },
        }) = &mut retirement_edit.taxable
        else {
            panic!()
        };
        *eligible = false;
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_RETIREMENT_BASIS_LEFT_OUT"), "{r:?}");
    assert_eq!(return_line(&r, "5b"), 700000);
    for kind in [RetirementKind::Ira, RetirementKind::Pension] {
        let mut d = deferred_withdrawal(16000, 10000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        edit_retirement(data).kind = kind;
        unsupported(&d, "Income:");
    }
    let corrected = deferred_withdrawal(6999, 10000);
    assert!(corrected.valid_stored());
    unsupported(&corrected, "Income:");
    assert!(
        crate::federal_rules::validate_income_amount_issue(&corrected.income[1].income).is_err()
    );
    let mut d = deferred_withdrawal(16000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).ordinary_distribution =
        Some(retirement_exception(20260615, immediate_annuity(20250101, 20260101), false));
    unsupported(&d, "Income:");
}

#[test]
fn qualified_withdrawals_match_irs_pro_rata_and_separate_contract_examples() {
    for (gross, balance, cost, excluded, taxable) in [
        (50000, 100000, 10000, 5000, 45000),
        (5000, 12500, 10000, 4000, 1000),
        (5000, 25000, 10000, 2000, 3000),
        (5000, 10000, 0, 0, 5000),
        (5000, 10000, 12000, 5000, 0),
    ] {
        let d = qualified_withdrawal(gross, balance, cost);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), gross * 100);
        assert_eq!(return_line(&r, "5b"), taxable * 100);
        assert_eq!(value(&r, "pension_2_basis_recovery"), excluded);
        assert_eq!(value(&r, "pension_2_remaining_basis"), cost - excluded);
        assert_eq!(value(&r, "early_distribution_tax"), taxable / 10);
        assert_eq!(value(&r, "net_investment_income_tax"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn direct_plan_rollover_requires_separate_accounting_and_employer_source() {
    {
        let confirmed = false;
        let mut d = qualified_withdrawal(5000, 12500, 10000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Rollover {
            destination: RolloverDestination::DirectEmployerPlan { confirmed },
            amount: 500000,
            early: false,
        });
        // No: the transfer counts the same for 2026 (the after-tax part is untaxed either way)
        // with a note to ask the receiving plan to account for it.
        let r = supported(&d);
        assert!(noted(&r, "WARN_DIRECT_PLAN_BASIS_NOT_TRACKED"));
        assert_eq!(return_line(&r, "5a"), 500_000);
        assert_eq!(return_line(&r, "5b"), 0);
    }
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            5000,
            RetirementTreatment::Rollover {
                destination: RolloverDestination::DirectEmployerPlan { confirmed: true },
                amount: 500000,
                early: false,
            },
        ),
    );
    unsupported(&d, "Income:");
}

#[test]
fn full_qualified_contract_rollover_transfers_only_actual_after_tax_cash() {
    let mut d = contract_discharge(RetirementKind::Pension, 8000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Rollover {
        destination: RolloverDestination::Ira,
        amount: 800000,
        early: false,
    });
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 8000)];
    let r = supported(&d);
    assert_eq!(value(&r, "annuity_2_surrender_loss"), 2000);
    assert_eq!(value(&r, "annuity_2_rolled_basis"), 8000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 8000);
}

#[test]
fn de_minimis_per_item_limits_do_not_cap_annual_deduction() {
    for (limit, afs) in [(250000, false), (500000, true)] {
        let d = de_minimis_business(limit, afs);
        let r = supported(&d);
        assert_eq!(value(&r, "business_2_expense_0_de_minimis"), 20000);
        let mut ordinary = d.clone();
        let FederalIncome::Business { data } = &mut ordinary.income[1].income else { panic!() };
        data.expenses[0].treatment = BusinessExpenseKind::Ordinary;
        let expected = supported(&ordinary);
        for id in ["agi", "self_employment_tax", "qbi_deduction"] {
            assert_eq!(value(&r, id), value(&expected, id));
        }
        assert_eq!(r.tax, expected.tax);
        assert_eq!(restart(d.clone()).draft, d);
        unsupported(&de_minimis_business(limit + 1, afs), "Income:");
    }
}

#[test]
fn sep_plan_rate_uses_unreduced_source_rate_and_exact_deposit_boundary() {
    for (rate, limit) in [(0, 0), (1000, 84481), (1050, 88304), (2500, 185860)] {
        let mut d = empty_return(FederalStatus::Single);
        let mut business = sep_business(10000, 0);
        let FederalIncome::Business { data } = &mut business else { panic!() };
        data.retirement =
            BusinessRetirement::Sep { contribution: limit, rate: f64::from(rate) / 100.0 };
        add(&mut d, income(business));
        let r = supported(&d);
        assert_eq!(r.lines.iter().find(|l| l.id == "business_2_sep_limit").unwrap().amount, limit);
        assert_eq!(value(&r, "business_2_sep_deduction"), crate::kernel::ratio::dollars(limit));
        assert_eq!(restart(d.clone()).draft, d);
        let FederalIncome::Business { data } = &mut d.income[0].income else { panic!() };
        let BusinessRetirement::Sep { contribution, .. } = &mut data.retirement else { panic!() };
        *contribution += 1;
        unsupported(&d, "Business retirement");
    }
}

#[test]
fn sep_plan_rate_applies_compensation_cap_before_annual_dollar_cap() {
    for (rate, limit) in
        [(100, 360000), (1000, 3600000), (1050, 3780000), (2000, 7200000), (2500, 7200000)]
    {
        let mut d = complete(FederalStatus::Single);
        let mut business = sep_business(1000000, 0);
        let FederalIncome::Business { data } = &mut business else { panic!() };
        data.retirement =
            BusinessRetirement::Sep { contribution: limit, rate: f64::from(rate) / 100.0 };
        add(&mut d, income(business));
        let r = supported(&d);
        assert_eq!(r.lines.iter().find(|l| l.id == "business_2_sep_limit").unwrap().amount, limit);
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        let BusinessRetirement::Sep { contribution, .. } = &mut data.retirement else { panic!() };
        *contribution += 1;
        unsupported(&d, "Business retirement");
    }
}

#[test]
fn sep_rate_validates_stored_source_and_preserves_health_coordination() {
    let mut d = empty_return(FederalStatus::Single);
    let mut business = health_business(10000, 9000, false);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.retirement = BusinessRetirement::Sep { contribution: 80000, rate: 10.0 };
    add(&mut d, income(business));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_health_insurance"), 8493);
    assert_eq!(value(&r, "business_2_medical_premiums"), 507);
}

#[test]
fn joint_filers_independent_hsa_excesses_keep_separate_account_value_limits() {
    let mut d = complete(FederalStatus::Joint);
    let mut taxpayer = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 540000);
    taxpayer.excess_accounting = Some(HsaExcessAccounting {
        prior_year: None,
        employer_excess_in_wages: 0,
        year_end_value: 10000,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
    });
    let mut spouse = hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 540000);
    spouse.excess_accounting = Some(HsaExcessAccounting {
        prior_year: None,
        employer_excess_in_wages: 0,
        year_end_value: 200000,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
    });
    d.adjustments.as_mut().unwrap().hsa = vec![taxpayer, spouse];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 8800);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 1000);
    assert_eq!(value(&r, "hsa_2_return_year_uncorrected_excess"), 1000);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 66);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_return_monthly_hsa_family_excess_uses_only_filers_agreed_share() {
    let mut d = separate_december_hsa(500000);
    let a = d.adjustments.as_mut().unwrap();
    a.hsa_family_share.as_mut().unwrap().method = HsaAllocationMethod::Monthly;
    a.hsa[0].coverage = HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly };
    a.hsa[0].excess_accounting = Some(HsaExcessAccounting {
        prior_year: None,
        employer_excess_in_wages: 0,
        year_end_value: 100000,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4375);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 625);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 38);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_last_month_hsa_history_checks_excess_confirmation_and_separate_spouses() {
    let mut d = complete(FederalStatus::Joint);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.last_month_history = Some(last_month_history(6, 215600));
    let mut spouse = h.clone();
    spouse.owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().hsa = vec![h, spouse];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 12);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 2);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    let history = h.last_month_history.as_mut().unwrap();
    history.contributions = 430100;
    unsupported(&d, "HSA");
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.last_month_history.as_mut().unwrap().contributions = 430000;
    h.funding_history = vec![hsa_funding(20250601, 20260601, 10000)];
    unsupported(&d, "HSA");
    let history = d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap();
    history.months[11] = HsaCoverage::None;
    assert!(!history.valid());
}

#[test]
fn roth_withdrawal_uses_regular_then_oldest_conversion_then_earnings() {
    let mut d = complete(FederalStatus::Single);
    let mut h = roth_history(Owner::Taxpayer, 2010, 1000);
    h.ordering.as_mut().unwrap().conversions = vec![
        RothConversion { year: 2023, taxable: 300000, nontaxable: 50000 },
        RothConversion { year: 2020, taxable: 200000, nontaxable: 100000 },
    ];
    d.adjustments.as_mut().unwrap().roth_history = vec![h];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 8000, 20260601));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 800000);
    assert_eq!(return_line(&r, "4b"), 50000);
    assert_eq!(value(&r, "early_distribution_tax"), 350);
    assert_eq!(value(&r, "roth_2_conversion_recapture"), 3000);
    assert_eq!(value(&r, "roth_2_regular_recovered"), 1000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn roth_qualification_needs_both_five_tax_years_and_actual_age_date() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19670101;
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2021, 0)];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 1000, 20260630));
    assert_eq!(return_line(&supported(&d), "4b"), 100000);
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 100);
    if let FederalIncome::Roth { data } = &mut d.income.last_mut().unwrap().income {
        edit_roth(data).distributed = 20260701;
    }
    d.adjustments.as_mut().unwrap().roth_history[0].ordering = None;
    assert_eq!(return_line(&supported(&d), "4b"), 0);
    let h = &mut d.adjustments.as_mut().unwrap().roth_history[0];
    h.first_year = 2022;
    h.ordering = Some(RothOrdering { regular: 0, conversions: vec![] });
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 100000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}

#[test]
fn roth_conversion_clock_is_separate_and_taxed_portion_precedes_nontaxable() {
    let mut d = complete(FederalStatus::Single);
    let mut h = roth_history(Owner::Taxpayer, 2010, 0);
    h.ordering.as_mut().unwrap().conversions =
        vec![RothConversion { year: 2022, taxable: 200000, nontaxable: 100000 }];
    d.adjustments.as_mut().unwrap().roth_history = vec![h];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 2500, 20261231));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 200);
    d.adjustments.as_mut().unwrap().roth_history[0].ordering.as_mut().unwrap().conversions[0]
        .year = 2021;
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 0);
}

#[test]
fn roth_current_contributions_and_conversions_are_taken_once_from_existing_sources() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2026, 0)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 100000, Answer::No);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            2000,
            RetirementTreatment::Conversion { amount: 200000, early: false },
        ),
    );
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 3500, 20261201));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 250000); // Conversion income plus $500 Roth earnings.
    assert_eq!(value(&r, "early_distribution_tax"), 250); // $2000 conversion recapture plus earnings.
    assert_eq!(value(&r, "roth_3_regular_recovered"), 1000);
}

#[test]
fn roth_disability_exempts_additional_tax_but_does_not_waive_five_year_income_rule() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2022, 0)];
    let mut e = roth_withdrawal(Owner::Taxpayer, 2000, 20260601);
    if let FederalIncome::Roth { data } = &mut e.income {
        edit_roth(data).exception = Some(RetirementException::Disability { began: 20260501 });
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 200000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    d.adjustments.as_mut().unwrap().roth_history[0].first_year = 2021;
    assert_eq!(return_line(&supported(&d), "4b"), 0);
}

#[test]
fn roth_missing_history_and_nonqualified_missing_ordering_assume_no_basis() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 1000, 20260101));
    // No history: no five-year period and no basis, so the whole withdrawal
    // is taxed as earnings, with a warning.
    let without = supported(&d);
    assert!(noted(&without, "WARN_ROTH_HISTORY_ASSUMED_NONE"), "{without:?}");
    assert!(!noted(&without, "WARN_ROTH_BASIS_ASSUMED_ZERO"));
    let mut h = roth_history(Owner::Taxpayer, 2022, 0);
    h.ordering = None;
    d.adjustments.as_mut().unwrap().roth_history = vec![h];
    // A history without its opening records: no basis, with a warning.
    let r = supported(&d);
    assert!(noted(&r, "WARN_ROTH_BASIS_ASSUMED_ZERO"), "{r:?}");
    assert_eq!(r.tax, without.tax);
    // Two histories for one owner stay refused.
    let h = roth_history(Owner::Taxpayer, 2022, 0);
    d.adjustments.as_mut().unwrap().roth_history = vec![h.clone(), h];
    unsupported(&d, "Roth IRA");
}

#[test]
fn roth_hsa_funding_consumed_conversion_principal_has_no_early_recapture() {
    let mut d = roth_hsa_funding(1600, 100, 2000, 600);
    d.adjustments.as_mut().unwrap().roth_history[0].ordering.as_mut().unwrap().conversions =
        vec![RothConversion { year: 2024, taxable: 90000, nontaxable: 50000 }];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260116"), 1100);
    assert_eq!(value(&r, "roth_2_conversion_recapture"), 0);
    assert_eq!(return_line(&r, "4b"), 20000);
    assert_eq!(value(&r, "early_distribution_tax"), 20);
}

#[test]
fn roth_hsa_funding_with_529_transfer_stays_explicitly_unsupported() {
    let mut draft = roth_hsa_funding(1500, 500, 2000, 500);
    draft.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
    set_qtp_roth(&mut draft, Owner::Taxpayer, &[qtp_roth_rollover()]);
    unsupported(&draft, "529-to-Roth IRA rollover");
}

#[test]
fn roth_hsa_funding_qualified_source_and_testing_failure_use_distinct_rules() {
    let mut d = roth_hsa_funding(1500, 500, 2000, 500);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.funding[0].loss = Some(HsaEligibilityLoss { date: 20260301, reason: HsaLossReason::Other });
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_basis_2_20260116"), 500);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "hsa_funding_recapture"), 1500);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 150);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(return_line(&r, "8"), 150000);
}

#[test]
fn roth_hsa_funding_requires_complete_history_and_real_source_order() {
    let mut d = roth_hsa_funding(1500, 500, 2000, 500);
    let mut old = roth_withdrawal(Owner::Taxpayer, 100, 20260101);
    old.label = "Earlier Roth withdrawal".into();
    add(&mut d, old);
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&d),
        Some("HSA_FUNDING_ORDER_CONTRADICTED")
    );
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.pop();
    d.adjustments.as_mut().unwrap().roth_history[0].ordering = None;
    assert!(crate::hsa_funding::source_reference_rule(&d).is_none());
    unsupported(&d, "HSA funding");
    d.adjustments.as_mut().unwrap().roth_history[0].ordering =
        Some(RothOrdering { regular: 50000, conversions: vec![] });
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].funded = 20260114;
    // This recorded source contradiction is rejected before tax calculation.
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&d),
        Some("HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT")
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
}
#[test]
fn rmd_uncorrected_shortfall_adds_tax_without_creating_income() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    d.adjustments.as_mut().unwrap().required_distributions = vec![rmd_record(5000, 1000)];
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 1000);
    assert_eq!(value(&r, "rmd_0_shortfall"), 4000);
    assert_eq!(return_line(&r, "23") - return_line(&before, "23"), 100000);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(r.tax.unwrap() - before.tax.unwrap(), 100000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn rmd_full_correction_uses_both_dates_and_earliest_window_end() {
    let mut d = complete(FederalStatus::Single);
    let mut e = rmd_record(5000, 1000);
    e.correction = rmd_corrected(20270115, 20281231);
    d.adjustments.as_mut().unwrap().required_distributions = vec![e];
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 400);
    for (received, filed) in [(20290101, 20270415), (20270115, 20290101)] {
        d.adjustments.as_mut().unwrap().required_distributions[0].correction =
            rmd_corrected(received, filed);
        assert_eq!(value(&supported(&d), "required_distribution_tax"), 1000);
    }
    for (notice, assessed) in
        [(Some(20270301), None), (None, Some(20270301)), (Some(20281231), Some(20270301))]
    {
        d.adjustments.as_mut().unwrap().required_distributions[0].correction =
            RmdCorrection::Corrected {
                received: 20270115,
                filed: RmdFiling::Submitted { date: 20270415 },
                notice,
                assessed,
            };
        assert_eq!(value(&supported(&d), "required_distribution_tax"), 1000);
    }
}
#[test]
fn rmd_unrelated_plan_excess_cannot_erase_shortfall_and_rates_stay_separate() {
    let mut d = complete(FederalStatus::Single);
    let mut reduced = rmd_record(2000, 0);
    reduced.label = "Corrected plan".into();
    reduced.correction = rmd_corrected(20270101, 20270415);
    d.adjustments.as_mut().unwrap().required_distributions =
        vec![rmd_record(5000, 1000), rmd_record(1000, 10000), reduced];
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 1200);
    for (number, amount) in [
        ("52a", 2000),
        ("53a", 0),
        ("54a", 200),
        ("52b", 6000),
        ("53b", 2000),
        ("54b", 1000),
        ("55", 1200),
    ] {
        let id = alloc::format!("taxpayer_rmd_line{number}");
        assert_eq!(value(&r, &id), amount);
        assert_eq!(printed_line(&r, &id), (TaxForm::Form5329, number.into()));
    }
    d.adjustments.as_mut().unwrap().required_distributions.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
#[test]
fn rmd_waiver_request_reduces_reported_shortfall_without_claiming_approval() {
    let mut d = complete(FederalStatus::Single);
    let mut e = rmd_record(5000, 1000);
    e.waiver = Some(RmdWaiver {
        amount: 150000,
        explanation: "Custodian missed instruction; transfer corrected and monitoring established."
            .into(),
    });
    d.adjustments.as_mut().unwrap().required_distributions = vec![e];
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_waiver_requested"), 1500);
    assert_eq!(value(&r, "required_distribution_tax"), 625);
    assert_eq!(value(&r, "taxpayer_rmd_line54b"), 625);
    assert_eq!(value(&r, "taxpayer_rmd_waiver54b"), 1500);
    d.adjustments.as_mut().unwrap().required_distributions[0].correction =
        rmd_corrected(20270101, 20270415);
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 250);
    d.adjustments.as_mut().unwrap().required_distributions[0].waiver.as_mut().unwrap().amount =
        400000;
    let waived = supported(&d);
    assert_eq!(value(&waived, "required_distribution_tax"), 0);
    assert_eq!(printed_line(&waived, "taxpayer_rmd_line55"), (TaxForm::Form5329, "55".into()));
    assert_eq!(value(&waived, "taxpayer_rmd_waiver54a"), 4000);
    d.adjustments.as_mut().unwrap().required_distributions[0].waiver.as_mut().unwrap().amount =
        400001;
    unsupported(&d, "Required distributions");
}
#[test]
fn rmd_joint_spouses_round_separately_and_require_source_confirmation() {
    let mut d = complete(FederalStatus::Joint);
    let mut e = rmd_record(2, 0);
    let mut spouse = e.clone();
    spouse.owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().required_distributions = vec![e.clone(), spouse];
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 2);
    assert_eq!(value(&r, "taxpayer_rmd_line54b"), 1);
    assert_eq!(value(&r, "spouse_rmd_line54b"), 1);
    e.label = alloc::string::String::new();
    d.adjustments.as_mut().unwrap().required_distributions[0] = e;
    unsupported(&d, "Required distributions");
    let mut single = complete(FederalStatus::Single);
    let mut e = rmd_record(100, 0);
    e.owner = Owner::Spouse;
    single.adjustments.as_mut().unwrap().required_distributions = vec![e];
    unsupported(&single, "Required distributions");
}
#[test]
fn rmd_inherited_roth_income_and_shortfall_are_distinct() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_roth(&mut d, 2021, 1000, 0, 0);
    d.adjustments.as_mut().unwrap().required_distributions = vec![rmd_record(5000, 1000)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "required_distribution_tax"), 1000);
}

#[test]
fn rmd_planned_filing_is_explicitly_conditional_and_cannot_extend_window() {
    let mut d = complete(FederalStatus::Single);
    let mut e = rmd_record(5000, 1000);
    e.correction = RmdCorrection::Corrected {
        received: 20270115,
        filed: RmdFiling::Planned { date: 20270415 },
        notice: None,
        assessed: None,
    };
    d.adjustments.as_mut().unwrap().required_distributions = vec![e];
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 400);
    assert_eq!(value(&r, "rmd_0_conditional_reduced_shortfall"), 4000);
    assert_eq!(restart(d.clone()).draft, d);
    if let RmdCorrection::Corrected { filed, .. } =
        &mut d.adjustments.as_mut().unwrap().required_distributions[0].correction
    {
        *filed = RmdFiling::Planned { date: 20290101 };
    }
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 1000);
}
#[test]
fn rmd_owner_uniform_and_joint_tables_match_irs_2026_examples() {
    let d = own_rmd(19510615, RmdPeriod::Current, 100000, Some(19570615));
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_account_0_minimum"), 4065);
    assert_eq!(value(&r, "required_distribution_tax"), 1016);
    let d = own_rmd(19510615, RmdPeriod::Current, 100000, Some(19620615));
    let r = supported(&d);
    assert_eq!(r.lines.iter().find(|l| l.id == "rmd_0_account_0_minimum").unwrap().amount, 395257);
    assert_eq!(crate::kernel::ratio::dollars(395257), 3953);
    assert_eq!(value(&r, "required_distribution_tax"), 988);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn rmd_balances_use_each_accounts_table_and_outstanding_rollover_once() {
    let mut d = own_rmd(19510615, RmdPeriod::Current, 20000, None);
    if let RmdMinimum::OwnIras { accounts, .. } =
        &mut d.adjustments.as_mut().unwrap().required_distributions[0].required
    {
        accounts[0].rollover = 460000;
        accounts.push(RmdIra {
            balance: 2530000,
            rollover: 0,
            spouse: Some(RmdSpouse { birth: 19620615, confirmed: true }),
        });
    }
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_account_0_minimum"), 1000);
    assert_eq!(value(&r, "rmd_0_account_1_minimum"), 1000);
    assert_eq!(value(&r, "required_distribution_tax"), 500);
    d.adjustments.as_mut().unwrap().required_distributions[0].timely = 200000;
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 0);
}
#[test]
fn rmd_first_year_deferral_preserves_amount_but_changes_shortfall_year() {
    let d = own_rmd(19530615, RmdPeriod::Current, 26500, None);
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_account_0_minimum"), 1000);
    assert_eq!(value(&r, "rmd_0_deferred_to_following_year"), 1000);
    assert_eq!(value(&r, "required_distribution_tax"), 0);
    let d = own_rmd(19520615, RmdPeriod::FirstPriorYear, 26500, None);
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_account_0_minimum"), 1000);
    assert_eq!(value(&r, "required_distribution_tax"), 250);
    let d = own_rmd(19520615, RmdPeriod::Current, 25500, None);
    assert_eq!(value(&supported(&d), "rmd_0_account_0_minimum"), 1000);
    unsupported(
        &own_rmd(19510615, RmdPeriod::FirstPriorYear, 26500, None),
        "Required distributions",
    );
}
#[test]
fn rmd_no_lifetime_ira_minimum_before_73_and_death_before_beginning_date() {
    let d = own_rmd(19540615, RmdPeriod::Current, 26500, None);
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 0);
    for (birth, death, period, expected) in [
        (19530615, 20261231, RmdPeriod::Current, 0),
        (19520615, 20260331, RmdPeriod::FirstPriorYear, 0),
        (19520615, 20260402, RmdPeriod::FirstPriorYear, 250),
    ] {
        let mut d = own_rmd(birth, period, 26500, None);
        d.people.as_mut().unwrap().taxpayer.died =
            Some(FinalReturn { bonds: None, date: death, ordinary: true, estate_medical: 0 });
        assert_eq!(value(&supported(&d), "required_distribution_tax"), expected);
    }
}
#[test]
fn rmd_table_endpoints_and_unresolved_source_cases() {
    assert_eq!(crate::rmd_tables::UNIFORM[0], 274);
    assert_eq!(crate::rmd_tables::UNIFORM[48], 20);
    assert_eq!(crate::rmd_tables::JOINT[2][44], 253); // Owner 75, spouse 64.
    let d = own_rmd(19060101, RmdPeriod::Current, 2000, None);
    assert_eq!(value(&supported(&d), "rmd_0_account_0_minimum"), 1000);
    let mut d = own_rmd(19510615, RmdPeriod::Current, 24600, None);
    d.adjustments.as_mut().unwrap().required_distributions[0].owner = Owner::Spouse;
    unsupported(&d, "Required distributions");
    let younger_spouse = own_rmd(19510615, RmdPeriod::Current, 66100, Some(20070615));
    let result = supported(&younger_spouse);
    assert_eq!(value(&result, "rmd_0_account_0_minimum"), 1000);
    assert_eq!(value(&result, "required_distribution_tax"), 250);
    assert_eq!(estimate(&restart(younger_spouse).draft), result);
}
#[test]
fn inherited_rmd_ten_year_rule_preserves_annual_after_beginning_obligation() {
    let d = beneficiary_rmd(19400615, 20250101, 19690615, InheritedIraKind::Traditional, 29800);
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_inherited_minimum"), 1000);
    assert_eq!(value(&r, "required_distribution_tax"), 250);
    assert!(breakdown(&r).iter().any(|b| b.explanation.contains("December 31, 2035")));
    // A calendar year is a typed value in the nested deadline message.
    let explanation =
        &r.lines.iter().find(|line| line.id == "rmd_0_inherited_minimum").unwrap().explanation;
    assert_eq!(explanation.key, "EXPLAIN_RMD_INHERITED_MINIMUM");
    let crate::MessageArg::Nested(deadline) = &explanation.args[1] else {
        panic!("missing typed deadline");
    };
    assert_eq!(deadline.key, "EXPLAIN_RMD_ACCOUNT_DEADLINE");
    assert_eq!(deadline.args, alloc::vec![crate::MessageArg::Number(2035)]);
    let d = beneficiary_rmd(19700615, 20250101, 19900615, InheritedIraKind::Traditional, 29800);
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
    let d = beneficiary_rmd(19400615, 20250101, 19690615, InheritedIraKind::Roth, 29800);
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
}
#[test]
fn inherited_rmd_eligibility_uses_exact_birth_dates_and_keeps_election() {
    let mut d = beneficiary_rmd(19531001, 20250101, 19631001, InheritedIraKind::Traditional, 24500);
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 1000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19631002;
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19631001;
    inherited_rmd_data(&mut d).deadline_election = true;
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    let mut after =
        beneficiary_rmd(19400615, 20250101, 19450615, InheritedIraKind::Traditional, 24500);
    inherited_rmd_data(&mut after).deadline_election = true;
    unsupported(&after, "Inherited required distributions");
}
#[test]
fn inherited_rmd_relief_years_do_not_restart_life_expectancy() {
    let d = beneficiary_rmd(19400615, 20210101, 19670615, InheritedIraKind::Traditional, 27600);
    // First year 2022: age 55 has 31.6; four elapsed years leave 27.6.
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 1000);
}
#[test]
fn inherited_rmd_minor_child_life_payments_continue_after_majority() {
    let mut d = beneficiary_rmd(19700615, 20200101, 20000101, InheritedIraKind::Roth, 59100);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::OwnerChild;
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_inherited_minimum"), 1000);
    assert!(breakdown(&r).iter().any(|b| b.explanation.contains("December 31, 2031")));
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::Ordinary;
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::OwnerChild;
    inherited_rmd_data(&mut d).died = 20220101;
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
}
#[test]
fn inherited_rmd_medical_eligibility_does_not_expire_with_childhood() {
    for eligibility in [InheritedEligibility::Disabled, InheritedEligibility::ChronicallyIll] {
        let mut d = beneficiary_rmd(19700615, 20200101, 20000101, InheritedIraKind::Roth, 59100);
        inherited_rmd_data(&mut d).eligibility = eligibility;
        let r = supported(&d);
        assert_eq!(value(&r, "rmd_0_inherited_minimum"), 1000);
        assert!(
            breakdown(&r)
                .iter()
                .any(|b| b.explanation.contains("Life-expectancy payments continue"))
        );
        assert!(!breakdown(&r).iter().any(|b| b.explanation.contains("December 31, 2031")));
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn inherited_rmd_beginning_date_respects_secure_birth_cohorts() {
    for (owner_birth, death, annual) in [
        (19490630, 20200331, false),
        (19490630, 20200401, true),
        (19490701, 20220401, true),
        (19490701, 20220331, false),
        (19500615, 20230401, true),
        (19510615, 20240401, false),
        (19510615, 20250401, true),
    ] {
        let d = beneficiary_rmd(owner_birth, death, 19800101, InheritedIraKind::Traditional, 50000);
        assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum") > 0, annual);
    }
}
#[test]
fn inherited_rmd_unknown_status_and_unsupported_contexts_are_explicit() {
    let mut d = beneficiary_rmd(19400615, 20250101, 19690615, InheritedIraKind::Traditional, 29800);
    inherited_rmd_data(&mut d).confirmed = false;
    unsupported(&d, "Inherited required distributions");
    inherited_rmd_data(&mut d).confirmed = true;
    inherited_rmd_data(&mut d).died = 20190101;
    inherited_rmd_data(&mut d).deadline_election = true;
    unsupported(&d, "Inherited required distributions");
    inherited_rmd_data(&mut d).deadline_election = false;
    inherited_rmd_data(&mut d).died = 20260101;
    unsupported(&d, "Inherited required distributions");
    inherited_rmd_data(&mut d).died = 20250101;
    inherited_rmd_data(&mut d).incoming = 2980000;
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 2000);
}

#[test]
fn inherited_rmd_pre2020_stretch_uses_reset_table_and_all_elapsed_years() {
    let d = beneficiary_rmd(19400615, 20180101, 19690615, InheritedIraKind::Traditional, 29200);
    // First distribution year 2019 at age 50: revised factor 36.2; seven years -> 29.2.
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_inherited_minimum"), 1000);
    assert!(!breakdown(&r).iter().any(|b| b.explanation.contains("December 31, 2028")));
    assert_eq!(restart(d.clone()).draft, d);
    let d = beneficiary_rmd(19400615, 20180101, 19690615, InheritedIraKind::Roth, 29200);
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 1000);
}
#[test]
fn inherited_rmd_old_minor_child_does_not_gain_a_retroactive_ten_year_limit() {
    let mut d = beneficiary_rmd(19500615, 19990101, 19800101, InheritedIraKind::Roth, 39000);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::OwnerChild;
    // Age 20 in 2000: revised 65.0 less 26 years = 39.0, including waived 2020.
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_inherited_minimum"), 1000);
    assert!(
        breakdown(&r).iter().any(|b| b.explanation.contains("Life-expectancy payments continue"))
    );
}
#[test]
fn inherited_rmd_expired_five_year_election_cannot_become_a_new_ten_year_period() {
    let mut d = beneficiary_rmd(19700615, 20190101, 19900101, InheritedIraKind::Roth, 10000);
    inherited_rmd_data(&mut d).deadline_election = true;
    // Five years plus the skipped 2020 year ended in 2025, not 2029.
    unsupported(&d, "Inherited required distributions");
    inherited_rmd_data(&mut d).closing = Some(0);
    unsupported(&d, "Inherited required distributions");
}
#[test]
fn inherited_rmd_exhausted_life_uses_actual_remaining_assets_not_old_balance() {
    let mut d = beneficiary_rmd(19000101, 20200101, 19000101, InheritedIraKind::Traditional, 10000);
    unsupported(&d, "Inherited required distributions");
    inherited_rmd_data(&mut d).closing = Some(1500000);
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_shortfall"), 15000);
    assert_eq!(value(&r, "required_distribution_tax"), 3750);
    d.adjustments.as_mut().unwrap().required_distributions[0].timely = 500000;
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_inherited_minimum"), 20000);
    assert_eq!(value(&r, "rmd_0_shortfall"), 15000);
    inherited_rmd_data(&mut d).closing = Some(0);
    assert_eq!(value(&supported(&d), "required_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn inherited_rmd_closing_value_does_not_replace_an_ordinary_annual_minimum() {
    let mut d = beneficiary_rmd(19400615, 20250101, 19690615, InheritedIraKind::Traditional, 29800);
    inherited_rmd_data(&mut d).closing = Some(5000000);
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 1000);
}
#[test]
fn inherited_ira_spouse_ownership_and_rmd_tax_are_separate() {
    let mut d = complete(FederalStatus::Joint);
    let id = d.next_id;
    let mut e = inherited_ira(5000, 40000);
    e.owner = crate::DocumentOwner::Spouses;
    add(&mut d, e);
    add(&mut d, inherited_payment(Owner::Spouse, Some(id), 10000));
    let mut required = rmd_record(12000, 10000);
    required.owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().required_distributions = vec![required];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 900000);
    assert_eq!(value(&r, "required_distribution_tax"), 500);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    let mut single = complete(FederalStatus::Single);
    let id = single.next_id;
    let mut e = inherited_ira(5000, 40000);
    e.owner = crate::DocumentOwner::Spouses;
    add(&mut single, e);
    add(&mut single, inherited_payment(Owner::Spouse, Some(id), 10000));
    unsupported(&single, "Income: Synthetic document");
}

#[test]
fn inherited_qcd_shares_personal_annual_limit_and_deduction_adjustment() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    let id = d.next_id;
    add_inherited_ira(&mut d, 80000, 0, 0);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            80000,
            RetirementTreatment::Ordinary,
        ),
    );
    let mut gift = qcd(Owner::Taxpayer, 20260601, 80000, 1000);
    let mut inherited = gift.payments[0].clone();
    inherited.inherited = Some(id);
    gift.payments.push(inherited);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 110000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 50000);
    assert_eq!(return_line(&r, "4a"), 16000000);
    assert_eq!(return_line(&r, "4b"), 5000000);
    d.income.reverse();
    assert_eq!(return_line(&supported(&d), "4b"), 5000000);
}

#[test]
fn inherited_qcd_does_not_borrow_another_decedents_taxable_capacity() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    let id = d.next_id;
    add_inherited_ira(&mut d, 10000, 8000, 0);
    add_inherited_ira(&mut d, 10000, 0, 50000);
    let mut gift = qcd(Owner::Taxpayer, 20260601, 5000, 0);
    gift.payments[0].inherited = Some(id);
    d.adjustments.as_mut().unwrap().qualified_charity.push(gift);
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 2000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 3000);
    assert_eq!(return_line(&r, "4b"), 1000000);
    assert_eq!(value(&r, &alloc::format!("inherited_ira_{id}_basis_recovered")), 8000);
}
#[test]
fn inherited_hsa_funding_uses_shared_lifetime_limit_and_testing_recapture() {
    let mut d = inherited_hsa_funding(4400, 0, 4400, 0, 4400);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    let mut months = vec![HsaCoverage::None; 12];
    months[0] = HsaCoverage::SelfOnly;
    h.coverage = HsaEligibility::Monthly { months };
    h.funding[0].loss = Some(HsaEligibilityLoss { date: 20260201, reason: HsaLossReason::Other });
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 4400);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 440);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(return_line(&r, "4b"), 0);
    let next = d.adjustments.as_ref().unwrap().hsa[0].funding[0].clone();
    d.adjustments.as_mut().unwrap().hsa[0].funding.push(next);
    // This recorded source contradiction is rejected before tax calculation.
    assert_eq!(crate::hsa_funding::source_reference_rule(&d), Some("HSA_FUNDING_LIFETIME"));
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
}

#[test]
fn sole_spouse_inherited_rmd_recalculates_current_single_life_expectancy() {
    let mut d =
        beneficiary_rmd(19400101, 20201201, 19610101, InheritedIraKind::Traditional, 100000);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::SoleSpouse;
    d.adjustments.as_mut().unwrap().required_distributions[0].timely = 500000;
    let r = supported(&d);
    // Current age 65 gives Table I 22.9, not the first-year factor less five.
    assert_eq!(r.lines.iter().find(|l| l.id == "rmd_0_inherited_minimum").unwrap().amount, 436681);
    assert_eq!(value(&r, "required_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn sole_spouse_can_elect_uniform_lifetime_without_owning_inherited_ira() {
    let mut d = beneficiary_rmd(19400101, 20251201, 19530101, InheritedIraKind::Traditional, 26500);
    inherited_rmd_data(&mut d).eligibility =
        InheritedEligibility::SoleSpouseUniformLifetime { confirmed: true };
    d.adjustments.as_mut().unwrap().required_distributions[0].timely = 100000;
    let r = supported(&d);
    // A 73-year-old's Table III divisor is 26.5. Table I would be 16.4.
    assert_eq!(value(&r, "rmd_0_inherited_minimum"), 1000);
    assert_eq!(value(&r, "rmd_0_shortfall"), 0);
    assert_eq!(value(&r, "required_distribution_tax"), 0);
    assert!(breakdown(&r).iter().any(|b| {
        b.explanation.contains("retained inherited status")
            && b.explanation.contains("Uniform Lifetime Table")
    }));
    assert_eq!(estimate(&restart(d.clone()).draft), r);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::SoleSpouse;
    // Without the election, the timely $1,000 is short of the Single Life minimum.
    unsupported(&d, "Inherited required distributions");
}
#[test]
fn sole_spouse_inherited_rmd_delays_to_original_owners_applicable_age() {
    for kind in [InheritedIraKind::Traditional, InheritedIraKind::Roth] {
        let mut d = beneficiary_rmd(19570101, 20221201, 19610101, kind, 100000);
        inherited_rmd_data(&mut d).eligibility = InheritedEligibility::SoleSpouse;
        assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
        inherited_rmd_data(&mut d).owner_birth = 19530101;
        d.adjustments.as_mut().unwrap().required_distributions[0].timely = 500000;
        assert_eq!(
            supported(&d).lines.iter().find(|l| l.id == "rmd_0_inherited_minimum").unwrap().amount,
            436681
        );
    }
}
#[test]
fn sole_spouse_inherited_rmd_preserves_longer_deceased_owner_expectancy() {
    let mut d =
        beneficiary_rmd(19500101, 20241201, 19300101, InheritedIraKind::Traditional, 100000);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::SoleSpouse;
    d.adjustments.as_mut().unwrap().required_distributions[0].timely = 800000;
    // Owner age 74 in 2024: 15.6 less two elapsed years = 13.6;
    // this exceeds the spouse's current age-96 factor.
    assert_eq!(
        supported(&d).lines.iter().find(|l| l.id == "rmd_0_inherited_minimum").unwrap().amount,
        735294
    );
    inherited_rmd_data(&mut d).eligibility =
        InheritedEligibility::SoleSpouseUniformLifetime { confirmed: true };
    // Table III at 96 is shorter than the deceased owner's remaining 13.6 years.
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 7352);
}
#[test]
fn sole_spouse_inherited_shortfall_requires_deemed_ownership_reconciliation() {
    let mut d =
        beneficiary_rmd(19400101, 20251201, 19610101, InheritedIraKind::Traditional, 100000);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::SoleSpouse;
    unsupported(&d, "Inherited required distributions");
    // A permitted fixed-deadline election has no intervening annual minimum.
    inherited_rmd_data(&mut d).kind = InheritedIraKind::Roth;
    inherited_rmd_data(&mut d).deadline_election = true;
    assert_eq!(value(&supported(&d), "rmd_0_inherited_minimum"), 0);
}

#[test]
fn recharacterized_personal_rent_does_not_absorb_passive_losses() {
    let mut d = complete(FederalStatus::Single);
    let mut loss = rental(1000, 6000);
    loss.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: loss }));
    let mut rent = personal_rent(12000, 2000);
    let FederalIncome::NonbusinessProperty { kind, .. } = &mut rent else { panic!() };
    *kind = NonbusinessPropertyKind::PersonalRental {
        related: alloc::vec![],
        basis: 10000,
        depreciable_basis: 2999,
        assets: vec![],
        history: PersonalRentalHistory::NONE,
    };
    add(&mut d, income(rent));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_passive_income"), 0);
    assert_eq!(value(&r, "rental_magi"), 70000);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 15000);
    assert_eq!(value(&r, "agi"), 70000);
}

#[test]
fn separate_amt_assets_validate_records_and_calculate_current_nol() {
    let mut asset = separate_amt_equipment();
    if let EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Separate { prior_depreciation, .. },
        ..
    } = &mut asset.history
    {
        *prior_depreciation = 1000001;
    }
    assert!(!asset.valid());
    let mut asset = separate_amt_equipment();
    asset.cost *= 100;
    if let EquipmentHistory::Continuing { prior_depreciation, amt, .. } = &mut asset.history {
        *prior_depreciation *= 100;
        if let EquipmentAmtHistory::Separate { prior_depreciation, .. } = amt {
            *prior_depreciation *= 100;
        }
    }
    let d = business_with_bonus(0, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 5536);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 27395);
    assert_eq!(value(&r, "amt_nol_carryforward"), 27395);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_amt_business_history_preserves_normal_young_filer_exemption() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    let mut source = cash_business(10000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    data.income_source = BusinessIncomeSource::Mixed { compensation: 600000 };
    data.assets.push(separate_amt_equipment());
    add(&mut d, income(source));
    let r = supported(&d);
    // Regular profit 9,345, half-SE 660; AMT profit 9,126. AMTI is below
    // 90,100. Regular taxable income 2,068 at the parent's 22% is 455.
    assert_eq!(value(&r, "agi"), 18685);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 17949);
    assert_eq!(value(&r, "alternative_minimum_tax") + value(&r, "regular_income_tax"), 455);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn alternative_nol_includes_employment_iso_preference_and_separate_excess_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-80000)));
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 100.0, 10000, 40000)];
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 20000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 0);
    d.screening.as_mut().unwrap().iso_exercises.clear();
    d.income[1].income = cash_business(-400000);
    let r = supported(&d);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 196000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 144000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 340000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn owner_sep_is_nonbusiness_deduction_for_regular_and_amt_nol_limits() {
    // Form 172 line 6 specifically includes owner SEP contributions. One
    // spouse's contribution cannot enlarge the NOL from the other's business.
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(sep_business(40000, 7000)));
    let mut loss = income(cash_business(-200000));
    loss.owner = crate::DocumentOwner::Spouses;
    add(&mut d, loss);
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 62826);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 62826);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 0);
    let mut no_sep = d.clone();
    no_sep.income[1].income = cash_business(40000);
    let baseline = supported(&no_sep);
    assert_eq!(
        value(&r, "current_net_operating_loss"),
        value(&baseline, "current_net_operating_loss")
    );
    assert_eq!(value(&r, "agi"), value(&baseline, "agi") - 7000);
    // Nonbusiness investment income can absorb that nonbusiness deduction.
    add(&mut d, income(savings_income(10000, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 59826);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_refigures_workplace_ira_phaseout_and_keeps_loss_limit_base() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![covered_wage(1, Owner::Taxpayer, 9000000, 0)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let before = supported(&d);
    assert_eq!(value(&before, "agi"), 89250);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 58520);
    assert_eq!(value(&r, "agi"), 23980);
    assert_eq!(value(&r, "nol_2025_used"), 58520);
    assert_eq!(value(&r, "nol_2025_carryforward"), 41480);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 82500);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_can_make_roth_contribution_eligible_despite_no_nol_baseline_error() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 17000000, 0)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 750000, Answer::Yes);
    unsupported(&d, "IRA contributions");
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(value(&r, "prior_nol_deduction"), 100000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_ira_phaseout_change_preserves_later_vintages_unabsorbed_balance() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![covered_wage(1, Owner::Taxpayer, 9000000, 0)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2016, 10000), (2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 60520);
    assert_eq!(value(&r, "agi"), 21980);
    assert_eq!(value(&r, "nol_2016_used"), 10000);
    assert_eq!(value(&r, "nol_2025_used"), 43770);
    assert_eq!(value(&r, "nol_2025_carryforward"), 56230);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
}

#[test]
fn prior_nol_baseline_retains_traditional_deduction_when_only_roth_is_ineligible() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 17000000, 0)];
    d.adjustments.as_mut().unwrap().iras = ira_facts(500000, 250000, Answer::No);
    unsupported(&d, "IRA contributions");
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 200000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 119120);
    assert_eq!(value(&r, "agi"), 45880);
    assert_eq!(value(&r, "nol_2025_carryforward"), 80880);
}

#[test]
fn current_cash_excess_and_stock_excess_retain_separate_2026_classes() {
    // $70,000 current cash uses $60,000 of 2026 G room; the old cash uses
    // none. Current cash has $10,000 percentage excess and absorbs the $500
    // floor, so its 2026 carry is $10,500. Stock has its own $10,000 C carry.
    let mut draft =
        current_and_prior_cash_with_appreciated_stock(10_000_000, 7_000_000, 20_000, 1_000_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 59_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 0);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 20_000);
    assert_eq!(value(&result, "charity_cash_sixty_return_year_carryforward"), 10_500);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 10_000);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 20_500);
    assert_eq!(return_line(&result, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 0);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 20_000);
    assert_eq!(value(&automatic, "charity_cash_sixty_return_year_carryforward"), 10_500);
    assert_eq!(value(&automatic, "charity_capital_gain_thirty_return_year_carryforward"), 10_000);
    assert_eq!(value(&automatic, "charity_return_year_carryforward"), 20_500);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn atnol_modified_absorption_preserves_special_capacity_and_expiration() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
    d.screening.as_mut().unwrap().atnol =
        prior_atnol(&[(2006, 100000, false), (2009, 10000, true)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_atnol_deduction"), 57000);
    assert_eq!(value(&r, "atnol_2006_used"), 54000);
    assert_eq!(value(&r, "atnol_2006_expired"), 46000);
    assert_eq!(value(&r, "atnol_2009_used"), 6000);
    assert_eq!(value(&r, "atnol_2009_carryforward"), 4000);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 4000);
}

#[test]
fn prior_nol_offsets_pension_income_and_preserves_gross_return_lines() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            20000,
            RetirementTreatment::Ordinary,
        ),
    );
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 51120);
    assert_eq!(value(&r, "agi"), 28880);
    assert_eq!(return_line(&r, "5b"), 2000000);
    assert_eq!(value(&r, "nol_2025_carryforward"), 48880);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_refigures_retirement_medical_exception_from_final_agi() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            20260601,
            RetirementException::Medical { expenses: 800000 },
            false,
        ),
    );
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 725);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 43120);
    assert_eq!(value(&r, "agi"), 26880);
    // $8,000 - 7.5% * $26,880 = $5,984 excepted; 10% * $4,016 rounds to $402.
    assert_eq!(value(&r, "early_distribution_tax"), 402);
}

#[test]
fn prior_nol_offsets_roth_conversion_without_changing_conversion_character() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 100000, conversion(100000, true)),
    );
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 200000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "roth_conversion_taxable"), 100000);
    assert_eq!(return_line(&r, "4b"), 10000000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "agi"), -40000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 56100);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_keeps_lump_sum_election_history_but_refigures_current_benefits() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2550000, 0);
    add(&mut d, income(social_security_net(1100000, 0)));
    d.adjustments.as_mut().unwrap().social_election =
        SocialElection::Compare { years: vec![social_prior_year()], complete: true };
    assert_eq!(return_line(&supported(&d), "6b"), 250000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2024, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 9520);
    assert_eq!(value(&r, "agi"), 15980);
    assert_eq!(return_line(&r, "6a"), 1100000);
    assert_eq!(return_line(&r, "6b"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_joint_senior_deductions_and_expiring_loss_are_separate() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19560615;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19560615;
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 42000);
    assert_eq!(value(&r, "senior_deduction"), 12000);
    assert_eq!(value(&r, "nol_2025_used"), 54000);
    assert_eq!(value(&r, "nol_2025_carryforward"), 46000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2006, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "nol_2006_used"), 64500);
    assert_eq!(value(&r, "nol_2006_expired"), 35500);
    assert_eq!(value(&r, "nol_2006_carryforward"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_restores_active_rental_allowance_and_preserves_separate_carries() {
    let mut d = nol_rental(150000);
    assert_eq!(value(&supported(&d), "rental_income"), 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 200000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 107120);
    assert_eq!(value(&r, "rental_magi"), 42880);
    assert_eq!(value(&r, "rental_income"), -25000);
    assert_eq!(value(&r, "rental_suspended_loss"), 6000);
    assert_eq!(value(&r, "agi"), 17880);
    assert_eq!(value(&r, "nol_2025_carryforward"), 92880);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_uses_only_taxable_inherited_roth_earnings() {
    for (year, taxable, deduction) in [(2022, 500, 35520), (2021, 0, 35120)] {
        let mut d = complete(FederalStatus::Single);
        add_inherited_roth(&mut d, year, 4000, 1000, 2500);
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(return_line(&r, "4a"), 400000);
        assert_eq!(return_line(&r, "4b"), taxable * 100);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 60000 + taxable - deduction);
        assert_eq!(value(&r, "nol_2025_carryforward"), 100000 - deduction);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_does_not_erase_roth_earnings_tax_or_conversion_recapture() {
    let mut d = complete(FederalStatus::Single);
    let mut h = roth_history(Owner::Taxpayer, 2010, 1000);
    h.ordering.as_mut().unwrap().conversions =
        vec![RothConversion { year: 2023, taxable: 300000, nontaxable: 50000 }];
    d.adjustments.as_mut().unwrap().roth_history = vec![h];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 5000, 20260601));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 500000);
    assert_eq!(return_line(&r, "4b"), 50000);
    assert_eq!(value(&r, "roth_2_regular_recovered"), 1000);
    assert_eq!(value(&r, "roth_2_conversion_recapture"), 3000);
    assert_eq!(value(&r, "early_distribution_tax"), 350);
    assert_eq!(value(&r, "agi"), -39500);
    assert_eq!(value(&r, "nol_2017_carryforward"), 55600);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_excludes_qualified_roth_withdrawals_from_limit_income() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600615;
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2010, 0)];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 100000, 20260601));
    let before = supported(&d);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 10000000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "prior_nol_deduction"), 28680);
    assert_eq!(value(&r, "nol_2025_carryforward"), 65320);
    assert_eq!(value(&before, "agi"), 60000);
    assert_eq!(value(&r, "agi"), 31320);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_worthless_securities_keep_holding_term_and_separate_loss_balances() {
    for acquisition in
        [SecurityAcquisition::Purchased { date: 20260101 }, SecurityAcquisition::Inherited]
    {
        let mut d = complete(FederalStatus::Single);
        let long = acquisition.term() == GainTerm::Long;
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
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), -3000);
        assert_eq!(value(&r, "prior_nol_deduction"), 32720);
        // Modified NOL income adds back the annual capital-loss deduction.
        assert_eq!(value(&r, "nol_2025_used"), 35720);
        assert_eq!(value(&r, "nol_2025_carryforward"), 64280);
        assert_eq!(
            value(
                &r,
                if long { "capital_long_carryforward" } else { "capital_short_carryforward" }
            ),
            7000
        );
        assert_eq!(restart(d.clone()).draft, d);
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), -43000);
        assert_eq!(value(&r, "capital_loss_carryforward"), 10000);
        assert_eq!(value(&r, "nol_2017_carryforward"), 56100);
    }
}

#[test]
fn prior_nol_retains_disability_pension_wages_and_eic_earnings() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income.clear();
    add(&mut d, income(disability_pension(17000)));
    let before = supported(&d);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 10000)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1h"), 1700000);
    assert_eq!(value(&r, "agi"), 7000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 9100);
    assert_eq!(value(&r, "earned_income_credit"), value(&before, "earned_income_credit"));
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_retains_alimony_ira_compensation_without_eic_earnings() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Alimony { data: alimony_record(20181231) }));
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 10000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "agi"), -7500);
    assert_eq!(value(&r, "nol_2017_carryforward"), 10000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_royalties_retain_portfolio_character_and_separate_losses() {
    for (receipts, expenses, deduction, closing) in
        [(30000, 5000, 55120, 44880), (1000, 5000, 31920, 68080)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(royalty(receipts, expenses)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "nonbusiness_royalty_income"), receipts - expenses);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 60000 + receipts - expenses - deduction);
        assert_eq!(value(&r, "nol_2025_carryforward"), closing);
        assert_eq!(value(&r, "current_net_operating_loss"), 0);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_personal_rent_retains_separate_regular_and_amt_suspensions() {
    for investment in [false, true] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(personal_rent_with_history(12000, 2000, 8000, 12000, investment)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 10000);
        assert_eq!(value(&r, "prior_nol_deduction"), 36720);
        assert_eq!(value(&r, "agi"), 25280);
        assert_eq!(value(&r, "nol_2025_carryforward"), 63280);
        assert_eq!(value(&r, "rental_suspended_loss"), 0);
        assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), 2000);
        assert_eq!(value(&r, "amt_rental_adjustment"), -2000);
        assert_eq!(
            value(&r, "personal_rent_2_investment_income"),
            if investment { 2000 } else { 0 }
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn debt_reduces_final_eligible_nol_before_expiration() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2006, 100000), (2025, 10000)]);
    add(&mut d, bankruptcy_debt(1500, 10000, 6000));
    let r = supported(&d);
    assert_eq!(value(&r, "nol_2006_used"), 43900);
    assert_eq!(value(&r, "nol_2006_debt_reduction"), 1500);
    assert_eq!(value(&r, "nol_2006_expired"), 54600);
    assert_eq!(value(&r, "nol_2006_carryforward"), 0);
    assert_eq!(value(&r, "nol_2025_carryforward"), 10000);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 10000);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
}

#[test]
fn debt_keeps_regular_and_amt_loss_and_property_ledgers_separate() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 55000, false)]);
    let before = supported(&d);
    let mut debt = bankruptcy_debt(1500, 10000, 6000);
    let FederalIncome::PersonalDebt { data } = &mut debt.income else { panic!() };
    let Some(crate::CanceledDebtChoice::Bankruptcy { property, .. }) = &mut data.treatment else {
        panic!()
    };
    property[0].amt_basis = Some(100000);
    add(&mut d, debt);
    let r = supported(&d);
    assert_eq!(value(&r, "nol_2025_debt_reduction"), 1500);
    assert_eq!(value(&r, "atnol_2025_used"), 54000);
    assert_eq!(value(&r, "atnol_2025_debt_reduction"), 1000);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 0);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
    assert_eq!(value(&r, "debt_2_amt_basis_reduction"), 500);
    let remaining: i64 = r
        .lines
        .iter()
        .filter(|v| v.id.starts_with("debt_2_amt_property_") && v.id.ends_with("remaining_basis"))
        .map(|v| v.amount)
        .sum();
    assert_eq!(remaining, 100000);
    assert_eq!(value(&r, "debt_2_property_0_remaining_basis"), 5000);
    assert_eq!(r.tax, before.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_amt_capital_history_recomputes_preferential_gain_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut history = capital_history();
    history.amt = Some(PriorAmtCapitalReturn {
        taxable_income: 10000000,
        short: 0,
        long: 0,
        deducted_loss: 0,
    });
    history.investment_parts.as_mut().unwrap().amt =
        Some(InvestmentAmtCapitalParts { short: 0, long: 0 });
    d.screening.as_mut().unwrap().capital_history = vec![history];
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    add(&mut d, income(ordinary_investment_sale(20000, 0, GainTerm::Long)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 6000);
    assert_eq!(value(&r, "amt_capital_gain"), 20000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 280000);
    // AMT ordinary 169,900 at 26%, plus 14,450 gain at 15%;
    // 5,550 uses the regular-income zero-rate room.
    assert_eq!(r.tax, Some(4634200));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_amt_capital_pools_change_investment_interest_and_capped_election() {
    for term in [GainTerm::Short, GainTerm::Long] {
        let mut d = complete(FederalStatus::Single);
        d.screening.as_mut().unwrap().capital_history = vec![amt_only_capital_history()];
        add(&mut d, income(ordinary_investment_sale(12000, 0, term)));
        investment_borrowing(&mut d, 5000, 0, 0);
        if term == GainTerm::Long {
            d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
                InvestmentElection::CapitalFirst { amount: 1000000 };
        }
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), 5000);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), 2000);
        assert_eq!(value(&r, "amt_investment_interest_carryforward"), 3000);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60000);
        if term == GainTerm::Long {
            assert_eq!(value(&r, "investment_elected_capital_gain"), 10000);
            assert_eq!(value(&r, "amt_investment_elected_capital_gain"), 2000);
        }
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn separate_amt_prior_capital_records_preserve_owners_and_negative_income() {
    let mut d = complete(FederalStatus::Joint);
    let first = amt_only_capital_history();
    let mut second = first.clone();
    second.owners = PriorReturnOwners::Spouse;
    d.screening.as_mut().unwrap().capital_history = vec![first, second];
    let r = supported(&d);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 17000);
    let mut d = complete(FederalStatus::Single);
    let mut history = amt_only_capital_history();
    history.amt = Some(PriorAmtCapitalReturn {
        taxable_income: -200000,
        short: -500000,
        long: -1000000,
        deducted_loss: 300000,
    });
    history.investment_parts.as_mut().unwrap().amt =
        Some(InvestmentAmtCapitalParts { short: 400_000, long: 1_000_000 });
    d.screening.as_mut().unwrap().capital_history = vec![history];
    let r = supported(&d);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 1000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_amt_capital_history_rejects_malformed_or_unreconciled_sources() {
    let mut d = complete(FederalStatus::Single);
    let mut history = amt_only_capital_history();
    history.amt.as_mut().unwrap().deducted_loss = 200000;
    d.screening.as_mut().unwrap().capital_history = vec![history.clone()];
    unsupported(&d, "Capital loss history");
}

#[test]
fn separate_amt_wash_sale_uses_its_own_loss_and_validates_source() {
    let mut sale = ordinary_investment_sale(1000, 5000, GainTerm::Short);
    let FederalIncome::Sale { data } = &mut sale else { panic!() };
    data.form.use_kind = SaleUse::Investment { wash_sale: 100000 };
    let sale = sale_with_amt(sale, 7000, 2000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(sale.clone()));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "amt_capital_gain"), -3000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 1000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
    {
        let (basis, wash) = (700000, 600001);
        let mut invalid = income(sale.clone());
        let FederalIncome::Sale { data } = &mut invalid.income else { panic!() };
        data.amt = crate::SaleAmt::Different {
            record: CapitalSaleAmt { term: None, basis, wash_sale: wash },
        };
        assert!(crate::federal_rules::validate_income(&invalid).is_err());
    }
    let personal = income(sale_with_amt(personal_sale(1000, 5000, GainTerm::Long), 7000, 100));
    assert!(crate::federal_rules::validate_income(&personal).is_err());
}

#[test]
fn refinanced_mortgage_allocates_original_and_replacement_interest_separately() {
    let mut d = complete(FederalStatus::Single);
    let deductions = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(deductions, 2_000_000);
    let mut credit = mortgage_certificate_credit(1500, 10000, 0);
    let certificate = credit.current.as_mut().unwrap();
    certificate.issue_date = 20260701;
    certificate.interest = MortgageInterestSource::WholeLoan {
        paid_interest: 1_000_000,
        certified_debt: 8_000_000,
        original_loan_amount: 10_000_000,
    };
    certificate.history = MortgageCertificateHistory::ReissuedDuring2026DifferentRates {
        facts: MortgageReissueFacts {
            refinanced_on: 20260630,
            original_interest_method: MortgageOriginalInterestMethod::FixedRateSchedule,
            first_reissue: true,
            original_number: "MCC-ORIGINAL-ALLOCATION".into(),
            original_rate: 20.0,
            original_scheduled_interest: 1_500_000,
            original_certified_balance: 9_000_000,
            reissued_certified_debt: 8_000_000,
            confirmed: true,
        },
        original_period_interest: MortgageInterestSource::WholeLoan {
            paid_interest: 500_000,
            certified_debt: 10_000_000,
            original_loan_amount: 12_500_000,
        },
    };
    d.credits.as_mut().unwrap().mortgage_credit = Some(credit);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_certified_interest"), 12000);
    assert_eq!(value(&r, "mortgage_original_period_credit"), 800);
    assert_eq!(value(&r, "mortgage_reissued_period_credit"), 1200);
    assert_eq!(value(&r, "mortgage_credit_current_potential"), 2000);
    assert_eq!(value(&r, "deduction"), 18000);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageInterestSource::WholeLoan { certified_debt, .. } = &mut certificate.interest {
        *certified_debt = 7_000_000;
    }
    assert!(!d.valid_stored());
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageInterestSource::WholeLoan { certified_debt, .. } = &mut certificate.interest {
        *certified_debt = 8_000_000;
    }
}

#[test]
fn minimum_tax_history_uses_2025_status_phaseouts_and_separate_addback() {
    for (status, base, excluded) in [
        (FederalStatus::Single, 88100, 0),
        (FederalStatus::Single, 200000, 29094),
        (FederalStatus::Head, 200000, 29094),
        (FederalStatus::Joint, 200000, 16380),
        (FederalStatus::Surviving, 200000, 16380),
        (FederalStatus::Separate, 200000, 34429),
        (FederalStatus::Single, 630350, 147328),
        (FederalStatus::Separate, 900350, 249707),
        (FederalStatus::Separate, 900354, 249708),
        (FederalStatus::Separate, 1174350, 345607),
    ] {
        let mut d =
            complete(if status == FederalStatus::Joint { status } else { FederalStatus::Single });
        let mut h = minimum_tax_history();
        h.status = status;
        h.base = base * 100;
        h.exclusions.clear();
        h.regular = 0;
        h.amt = 0;
        h.carry = 0;
        d.credits.as_mut().unwrap().minimum_tax =
            Some(crate::MinimumTaxHistories::from_current_unit(h));
        assert_eq!(
            value(&supported(&d), "minimum_tax_exclusion_tax"),
            excluded,
            "{status:?} {base}"
        );
    }
}
#[test]
fn nonparticipating_trading_income_funds_separate_regular_amt_interest_limits() {
    let d = trading_interest_draft(10_000, false, 5_000);
    let r = estimate(&d);
    k1_interest_note(&r, 20_000, 25_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn historical_carryforward_output_keeps_original_and_remaining_qbi_separate() {
    for at_risk in [false, true] {
        let mut source = historical_k1_excess(200_000, at_risk);
        let history = vec![loss_year(2020, 600_000, 300_000, 500_000, 250_000)];
        if at_risk {
            source.risk.prior_loss = 50_000_000;
            source.loss_qbi.at_risk = history;
        } else {
            source.prior_loss = 50_000_000;
            source.loss_qbi.history = history;
        }
        source.amt = Some(amt_records(&source));
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
        let r = supported(&d);
        let stage = if at_risk { "at-risk" } else { "basis" };
        // 2026-10-19: the owning usd_cents message renders Currency2 amounts.
        let expected = alloc::format!(
            "2020 {stage}-loss carryforward: $100000.00; QBI portion: $50000.00. Original suspended loss: $600000.00; original QBI portion: $300000.00."
        );
        assert_eq!(r.review_notes.iter().filter(|n| n.contains(&expected)).count(), 1);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn separately_documented_notes_keep_their_own_holding_periods() {
    let mut source = repaid_written_note(GainTerm::Short);
    source.loans.push(repaid_written_note(GainTerm::Long).loans.remove(0));
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_200_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    let mut comparison = d.clone();
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    for term in [GainTerm::Short, GainTerm::Long] {
        add(&mut comparison, income(investment_sale_gain(term, 2_000, 2_000)));
    }
    let r = supported(&d);
    assert_eq!(r.return_lines, supported(&comparison).return_lines);
    for term in ["short", "long"] {
        assert_eq!(value(&r, &alloc::format!("scorp_{id}_loan_{term}_gain")), 2_000);
        assert_eq!(value(&r, &alloc::format!("scorp_{id}_loan_{term}_amt_gain")), 2_000);
    }
    assert_eq!(value(&r, "capital_gain"), 4_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 152);
    scorp_note(&r, id, "year-end at-risk investment before recapture", 6_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn completed_property_loss_return_keeps_regular_and_amt_history_separate() {
    let mut source = former_passive_property_scorp(0, -20000, 0, 0);
    source.property_qbi = property_qbi(20000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    source.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = -1000000;
    let (mut draft, _) = passive_property_pool(source, 0);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 40000);
    assert_eq!(value(&r, "taxable_income"), 23900);
    assert_eq!(value(&r, "regular_income_tax"), 2623);
    assert_eq!(return_line(&r, "8"), -2000000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 20000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 20000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 20000);
    assert_eq!(value(&r, "amt_section1231_current_loss_history"), 10000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 10000);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    assert!(r.tax.is_some());
}

#[test]
fn historical_property_excess_loss_keeps_prior_qualification_and_amt_separate() {
    let draft = historical_property_excess_draft();
    let r = supported(&draft);
    // Regular loss $400k: current qualified $200k, pre-2018 basis loss
    // nonqualified $100k, and at-risk history $100k with $50k qualified.
    // $144k deferred * 250/400 = $90k; current qualified loss is $160k.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 160000);
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "K-1 excess-loss character: NOL carryforward $144000; qualified portion $90000."
    )));
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 256000);
    assert_eq!(value(&r, "nol_carryforward"), 340000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 240000);
    assert_eq!(return_line(&r, "8"), -25600000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn property_loss_operating_profit_separates_loss_character_history() {
    let mut draft = property_loss_operating_profit_draft(400000, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    edit_scorp(data).section1231.as_mut().unwrap().basis_loss = 1000000;
    edit_scorp(data).property_qbi.losses.history =
        vec![loss_year(2025, 10000, 10000, 10000, 10000)];
    let r = supported(&draft);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 54000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 256000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 356000);
    assert_eq!(value(&r, "nol_carryforward"), 250000);
    assert_eq!(r.tax, Some(0));
    let mut draft = property_loss_operating_profit_draft(400000, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    edit_scorp(data).amt.as_mut().unwrap().section1231.as_mut().unwrap().net = -50000000;
    edit_scorp(data).amt.as_mut().unwrap().prior_loss = 1000000;
    unsupported(&draft, "Property excess business loss");
}

#[test]
fn employee_8919_spouse_caps_and_multiple_firms_are_separate() {
    let mut draft = complete(FederalStatus::Joint);
    draft.income[0] = wage(1, Owner::Taxpayer, 18000000, 0);
    add(&mut draft, wage(2, Owner::Spouse, 6000000, 0));
    for _ in 0..6 {
        add(&mut draft, income(omitted_employee_wages(1000)));
    }
    let mut spouse = income(omitted_employee_wages(10000));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut draft, spouse);
    let r = supported(&draft);
    crate::tests::support::assert::payroll::form8919_base(&r, 0, 180_000, 4_500);
    crate::tests::support::assert::payroll::form8919_base(&r, 1, 60_000, 124_500);
    assert_eq!(value(&r, "employee_wages_0_social_tax"), 279);
    assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 87);
    assert_eq!(value(&r, "employee_wages_1_social_tax"), 620);
    assert_eq!(value(&r, "employee_wages_1_medicare_tax"), 145);
    assert_eq!(value(&r, "additional_medicare_tax"), 54);
    assert_eq!(return_line(&r, "1g"), 1600000);
    assert_eq!(return_line(&r, "23"), 118500);
}

#[test]
fn sick_plan_group_funding_and_six_month_limit_reproduce_irs_payment_example() {
    let funding = HouseholdSickFunding::Group {
        years: (2023..=2025)
            .map(|year| HouseholdSickPremium {
                ended: year * 10000 + 1231,
                employer: 700000,
                employee: 300000,
            })
            .collect(),
    };
    let pay =
        (1..=7).map(|month| raw_sick_payment(2000, 20260028 + month * 100, 20251205)).collect();
    // Seven $2,000 benefits, 70% employer-funded; July is outside six calendar months.
    // State deposits: $520.80 SS + $121.80 Medicare on $8,400 payroll wages.
    let d = with_household(sick_plan_employer(funding, pay, 52080, 12180));
    let r = supported(&d);
    assert_eq!(value(&r, "household_0_social_wages"), 8400);
    assert_eq!(value(&r, "household_0_medicare_wages"), 8400);
    assert_eq!(value(&r, "household_0_social_tax"), 1042);
    assert_eq!(value(&r, "household_0_medicare_tax"), 244);
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 643);
    assert_eq!(value(&r, "household_0_futa_tax"), 42);
    assert_eq!(return_line(&r, "23"), 68500);
    assert_eq!(r.tax, Some(570800));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn sick_plan_six_calendar_months_restart_after_a_single_day_of_work() {
    for (paid, worked, taxable) in [
        (20260630, 20251205, true),
        (20260701, 20251205, false),
        (20260831, 20260206, true),
        (20260901, 20260206, false),
    ] {
        let e = sick_plan_employer(
            HouseholdSickFunding::Employer,
            vec![raw_sick_payment(3000, paid, worked)],
            if taxable { 18600 } else { 0 },
            if taxable { 4350 } else { 0 },
        );
        let r = supported(&with_household(e));
        assert_eq!(value(&r, "household_0_social_wages"), if taxable { 3000 } else { 0 });
        assert_eq!(value(&r, "household_0_futa_wages"), if taxable { 3000 } else { 0 });
        assert_eq!(r.tax, Some(if taxable { 527000 } else { 502300 }));
    }
}

#[test]
fn sick_plan_prior_ssdi_excludes_fica_but_preserves_futa() {
    for (entitled, no_services, fica) in
        [(20250101, true, false), (20260201, true, true), (20250101, false, true)]
    {
        let mut payment = raw_sick_payment(5000, 20260331, 20260301);
        payment.disability = Some(HouseholdSickEntitlement { date: entitled, no_services });
        let e = sick_plan_employer(
            HouseholdSickFunding::Employer,
            vec![payment],
            if fica { 31000 } else { 0 },
            if fica { 7250 } else { 0 },
        );
        let r = supported(&with_household(e));
        assert_eq!(value(&r, "household_0_medicare_wages"), if fica { 5000 } else { 0 });
        assert_eq!(value(&r, "household_0_futa_wages"), 5000);
        assert_eq!(value(&r, "household_0_futa_tax"), 30);
        assert_eq!(r.tax, Some(if fica { 543500 } else { 505300 }));
    }
}

#[test]
fn sick_plan_death_and_disability_retirement_use_payment_and_termination_dates() {
    for (ended, taxable) in [
        (HouseholdSickEnd::Death { date: 20251231, definite_plan: false }, false),
        (HouseholdSickEnd::Death { date: 20260115, definite_plan: true }, false),
        (HouseholdSickEnd::Death { date: 20260115, definite_plan: false }, true),
        (HouseholdSickEnd::Death { date: 20260301, definite_plan: true }, true),
        (HouseholdSickEnd::Retirement { date: 20260115, definite_plan: true }, false),
        (HouseholdSickEnd::Retirement { date: 20260115, definite_plan: false }, true),
    ] {
        let mut payment = raw_sick_payment(5000, 20260201, 20251130);
        payment.ended = Some(ended);
        let d = with_household(sick_plan_employer(
            HouseholdSickFunding::Employer,
            vec![payment],
            if taxable { 31000 } else { 0 },
            if taxable { 7250 } else { 0 },
        ));
        let r = supported(&d);
        assert_eq!(value(&r, "household_0_futa_wages"), if taxable { 5000 } else { 0 });
        assert_eq!(r.tax, Some(if taxable { 543500 } else { 502300 }));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn sick_plan_funding_uses_cost_weighting_and_first_year_estimates() {
    let funding = HouseholdSickFunding::Group {
        years: vec![
            HouseholdSickPremium { ended: 20241231, employer: 0, employee: 10000 },
            HouseholdSickPremium { ended: 20251231, employer: 90000, employee: 0 },
        ],
    };
    let e = sick_plan_employer(
        funding,
        vec![raw_sick_payment(10000, 20260331, 20251231)],
        55800,
        13050,
    );
    let r = supported(&with_household(e));
    // Total funding is 900/1,000, not the 50% unweighted average of annual percentages.
    assert_eq!(value(&r, "household_0_medicare_wages"), 9000);
    assert_eq!(r.tax, Some(575300));
    for (employer, employee, wages, social, medicare, tax) in
        [(70000, 30000, 7000, 43400, 10150, 560000), (0, 100000, 0, 0, 0, 502300)]
    {
        let e = sick_plan_employer(
            HouseholdSickFunding::NewGroup { employer, employee },
            vec![raw_sick_payment(10000, 20260331, 20251231)],
            social,
            medicare,
        );
        let r = supported(&with_household(e));
        assert_eq!(value(&r, "household_0_medicare_wages"), wages);
        assert_eq!(r.tax, Some(tax));
    }
}

#[test]
fn sick_plan_invalid_funding_dates_and_unknown_liability_never_resolve_as_zero() {
    let mut e = sick_plan_employer(
        HouseholdSickFunding::NewGroup { employer: 0, employee: 0 },
        vec![raw_sick_payment(5000, 20260331, 20251231)],
        0,
        0,
    );
    assert!(!crate::household_employment::valid(core::slice::from_ref(&e)));
    let notice = e.workers[0].disability.as_mut().unwrap();
    notice.wages = HouseholdSickWages::Plan {
        funding: HouseholdSickFunding::Employer,
        pay: vec![raw_sick_payment(5000, 20260331, 20260401)],
    };
    assert!(!crate::household_employment::valid(core::slice::from_ref(&e)));
    let mut impossible = raw_sick_payment(5000, 20260331, 20260201);
    impossible.ended = Some(HouseholdSickEnd::Death { date: 20260115, definite_plan: true });
    let invalid = sick_plan_employer(HouseholdSickFunding::Employer, vec![impossible], 0, 0);
    assert!(!crate::household_employment::valid(core::slice::from_ref(&invalid)));
    let mut e = sick_plan_employer(
        HouseholdSickFunding::Employer,
        vec![raw_sick_payment(5000, 20260331, 20251231)],
        31000,
        7250,
    );
    e.workers[0].disability.as_mut().unwrap().complete = false;
    unsupported(&with_household(e), "Household disability notice");
}

#[test]
fn sick_plan_individual_policy_uses_payment_policy_year_not_group_average() {
    let funding = HouseholdSickFunding::Individual {
        periods: vec![
            HouseholdSickPolicy { starts: 20250401, ends: 20260331, employer: 1600, employee: 800 },
            HouseholdSickPolicy { starts: 20260401, ends: 20270331, employer: 800, employee: 1600 },
        ],
    };
    let e = sick_plan_employer(
        funding,
        vec![
            raw_sick_payment(6000, 20260331, 20260301),
            raw_sick_payment(3000, 20260401, 20260301),
        ],
        31000,
        7250,
    );
    let d = with_household(e);
    let r = supported(&d);
    // First year's 2/3 of $6,000 plus second year's 1/3 of $3,000 = $5,000.
    // Both sides' FICA $765 less actual state deposits rounded to $383, plus $30 FUTA.
    assert_eq!(value(&r, "household_0_social_wages"), 5000);
    assert_eq!(value(&r, "household_0_social_tax"), 620);
    assert_eq!(value(&r, "household_0_medicare_tax"), 145);
    assert_eq!(value(&r, "household_0_futa_tax"), 30);
    assert_eq!(r.tax, Some(543500));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn sick_plan_individual_employee_funding_is_exempt_without_erasing_regular_wages() {
    let mut e = sick_plan_employer(
        HouseholdSickFunding::Individual {
            periods: vec![HouseholdSickPolicy {
                starts: 20260101,
                ends: 20261231,
                employer: 0,
                employee: 2400,
            }],
        },
        vec![raw_sick_payment(10000, 20260331, 20251231)],
        0,
        0,
    );
    e.workers[0].pay.push(household_pay(3000, 20260331));
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_social_wages"), 3000);
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 0);
    assert_eq!(r.tax, Some(550000)); // $5,020 personal + $459 FICA + $18 FUTA.
}

#[test]
fn sick_plan_individual_missing_overlapping_or_unfunded_periods_are_invalid() {
    let base =
        HouseholdSickPolicy { starts: 20260101, ends: 20261231, employer: 1600, employee: 800 };
    let mut missing = base.clone();
    missing.starts = 20260401;
    let mut empty = base.clone();
    empty.employer = 0;
    empty.employee = 0;
    let mut reversed = base.clone();
    reversed.starts = 20261231;
    reversed.ends = 20260101;
    for periods in [vec![], vec![missing], vec![empty], vec![reversed], vec![base.clone(), base]] {
        let e = sick_plan_employer(
            HouseholdSickFunding::Individual { periods },
            vec![raw_sick_payment(6000, 20260331, 20251231)],
            24800,
            5800,
        );
        assert!(!crate::household_employment::valid(core::slice::from_ref(&e)));
    }
}

#[test]
fn sick_plan_uninsured_fund_uses_prior_calendar_contributions_weighted_by_cost() {
    let e = sick_plan_employer(
        HouseholdSickFunding::Fund {
            years: vec![
                HouseholdSickContribution { year: 2023, employer: 0, employee: 10000 },
                HouseholdSickContribution { year: 2024, employer: 0, employee: 0 },
                HouseholdSickContribution { year: 2025, employer: 90000, employee: 0 },
            ],
        },
        vec![raw_sick_payment(10000, 20260331, 20251231)],
        55800,
        13050,
    );
    let d = with_household(e);
    let r = supported(&d);
    // $900 employer / $1,000 total; the zero-contribution year is valid history.
    assert_eq!(value(&r, "household_0_social_wages"), 9000);
    assert_eq!(value(&r, "household_0_futa_tax"), 42);
    assert_eq!(r.tax, Some(575300));
    assert_eq!(restart(d.clone()).draft, d);
    for years in [
        vec![HouseholdSickContribution { year: 2025, employer: 70000, employee: 30000 }],
        vec![
            HouseholdSickContribution { year: 2024, employer: 70000, employee: 30000 },
            HouseholdSickContribution { year: 2025, employer: 70000, employee: 30000 },
        ],
    ] {
        let r = supported(&with_household(sick_plan_employer(
            HouseholdSickFunding::Fund { years },
            vec![raw_sick_payment(10000, 20260331, 20251231)],
            43400,
            10150,
        )));
        assert_eq!(value(&r, "household_0_medicare_wages"), 7000);
        assert_eq!(r.tax, Some(560000));
    }
}

#[test]
fn sick_plan_new_uninsured_fund_applies_actual_periodic_contributions() {
    let periods = vec![
        HouseholdSickFundPeriod {
            from: 20260101,
            through: 20260331,
            starts: 20260401,
            ends: 20260430,
            employer: 60000,
            employee: 40000,
        },
        HouseholdSickFundPeriod {
            from: 20260401,
            through: 20260430,
            starts: 20260501,
            ends: 20260531,
            employer: 90000,
            employee: 10000,
        },
    ];
    let d = with_household(sick_plan_employer(
        HouseholdSickFunding::NewFund { periods },
        vec![
            raw_sick_payment(3000, 20260415, 20260401),
            raw_sick_payment(3000, 20260515, 20260401),
        ],
        27900,
        6525,
    ));
    let r = supported(&d);
    // 60% of $3,000 + 90% of $3,000 = $4,500; separate statutory line rounding.
    assert_eq!(value(&r, "household_0_social_wages"), 4500);
    assert_eq!(value(&r, "household_0_social_tax"), 558);
    assert_eq!(value(&r, "household_0_medicare_tax"), 131);
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 344);
    assert_eq!(value(&r, "household_0_futa_tax"), 27);
    assert_eq!(r.tax, Some(539500));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn sick_plan_new_fund_year_to_date_determination_and_employee_only_funding() {
    for (employer, employee, social, medicare, wages, tax) in
        [(1600, 800, 24800, 5800, 4000, 535300), (0, 2400, 0, 0, 0, 502300)]
    {
        let p = HouseholdSickFundPeriod {
            from: 20260101,
            through: 20260415,
            starts: 20260415,
            ends: 20260415,
            employer,
            employee,
        };
        let e = sick_plan_employer(
            HouseholdSickFunding::NewFund { periods: vec![p] },
            vec![raw_sick_payment(6000, 20260415, 20260401)],
            social,
            medicare,
        );
        let r = supported(&with_household(e));
        assert_eq!(value(&r, "household_0_medicare_wages"), wages);
        assert_eq!(r.tax, Some(tax));
    }
}

#[test]
fn sick_plan_fund_missing_history_or_future_determination_cannot_be_silently_used() {
    let row = HouseholdSickContribution { year: 2025, employer: 1600, employee: 800 };
    let mut skipped = row.clone();
    skipped.year = 2023;
    let mut zero = row.clone();
    zero.employer = 0;
    zero.employee = 0;
    let mut invalid = vec![
        HouseholdSickFunding::Fund { years: vec![row.clone(), row.clone()] },
        HouseholdSickFunding::Fund { years: vec![skipped, row] },
        HouseholdSickFunding::Fund { years: vec![zero] },
    ];
    let p = HouseholdSickFundPeriod {
        from: 20260101,
        through: 20260331,
        starts: 20260401,
        ends: 20260430,
        employer: 1600,
        employee: 800,
    };
    let mut future = p.clone();
    future.through = 20260416;
    let mut gap = p.clone();
    gap.starts = 20260416;
    let mut unfunded = p.clone();
    unfunded.employer = 0;
    unfunded.employee = 0;
    for periods in [vec![future], vec![gap], vec![unfunded], vec![p.clone(), p]] {
        invalid.push(HouseholdSickFunding::NewFund { periods });
    }
    for funding in invalid {
        let e = sick_plan_employer(
            funding,
            vec![raw_sick_payment(6000, 20260415, 20260401)],
            24800,
            5800,
        );
        assert!(!crate::household_employment::valid(core::slice::from_ref(&e)));
    }
}

#[test]
fn ira_gross_exception_limits_allocate_taxable_relief_at_both_rates() {
    for (facts, relief, penalty) in [
        (gross_home_claim(0), 8000, 800),
        (gross_birth_claim(1000), 3200, 1280),
        (RetirementException::Levy { amount: 1000000 }, 8000, 800),
        (
            RetirementException::Emergency {
                balance: 10000000,
                history: EmergencyHistory::NONE,
                eligible: true,
            },
            800,
            1520,
        ),
        (
            RetirementException::DomesticAbuse {
                plan: "Plan A".into(),
                event: 20260101,
                balance: 10000000,
                prior_used: 0,
                prior_plan_used: 0,
                eligible: true,
            },
            8400,
            760,
        ),
    ] {
        for simple in [false, true] {
            let d = ira_basis_exception_return(facts.clone(), simple);
            let r = supported(&d);
            let penalty = if simple { penalty * 5 / 2 } else { penalty };
            assert_eq!(return_line(&r, "4b"), 1600000);
            assert_eq!(value(&r, "early_taxpayer_exceptions"), relief);
            assert_eq!(value(&r, "retirement_2_qualified_gross"), relief * 5 / 4);
            assert_eq!(value(&r, "retirement_2_qualified_taxable"), relief);
            assert_eq!(value(&r, "early_distribution_tax"), penalty);
            assert_eq!(r.tax, Some((21374 + penalty) * 100));
            assert_eq!(supported(&restart(d).draft).tax, r.tax);
        }
    }
}

#[test]
fn ira_gross_exception_shared_limits_use_gross_and_higher_tax_relief() {
    for (facts, penalty, usage, key) in [
        (gross_home_claim(3000), 2200, 10000, "first_home_used_taxpayer"),
        (gross_birth_claim(1000), 2800, 5000, "birth_adoption_used_0_Synthetic child"),
    ] {
        let mut d = ira_basis_exception_return(facts.clone(), false);
        add(
            &mut d,
            basis_distribution(Owner::Taxpayer, 10000, retirement_exception(20260601, facts, true)),
        );
        d.adjustments.as_mut().unwrap().ira_basis[0].year_end = 7000000;
        let r = supported(&d);
        assert_eq!(return_line(&r, "4b"), 2400000);
        assert_eq!(value(&r, key), usage);
        assert_eq!(value(&r, "early_distribution_tax"), penalty);
        assert_eq!(r.tax, Some((23294 + penalty) * 100));
        d.income.reverse();
        assert_eq!(supported(&d).tax, r.tax);
        assert_eq!(supported(&restart(d).draft).tax, r.tax);
    }
}

#[test]
fn ira_gross_exception_keeps_account_and_non_ira_allocation_boundaries() {
    for facts in [
        gross_birth_claim(0),
        RetirementException::Emergency {
            balance: 10000000,
            history: EmergencyHistory::NONE,
            eligible: true,
        },
        RetirementException::DomesticAbuse {
            plan: "Plan A".into(),
            event: 20260101,
            balance: 10000000,
            prior_used: 0,
            prior_plan_used: 0,
            eligible: true,
        },
        RetirementException::Levy { amount: 1000000 },
    ] {
        let mut d = ira_basis_exception_return(facts, false);
        d.adjustments.as_mut().unwrap().ira_basis.clear();
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        edit_retirement(data).kind = RetirementKind::Pension;
        edit_retirement(data).box_2a = 1600000;
        edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
        unsupported(&d, "Retirement exception allocation");
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        edit_retirement(data).kind = RetirementKind::Nonqualified;
        unsupported(&d, "Income");
    }
}

#[test]
fn plan_gross_exception_preserves_separate_contract_ratio_and_zero_income() {
    for (investment, taxable, relief, regular) in
        [(80000, 4000, 2000, 18494), (100000, 0, 0, 17570), (120000, 0, 0, 17570)]
    {
        let d = plan_gross_exception(RetirementException::Levy { amount: 1000000 }, investment);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), taxable * 100);
        assert_eq!(value(&r, "retirement_2_qualified_gross"), 10000);
        assert_eq!(value(&r, "retirement_2_qualified_taxable"), relief);
        assert_eq!(value(&r, "early_distribution_tax"), (taxable - relief) / 10);
        assert_eq!(r.tax, Some((regular + (taxable - relief) / 10) * 100));
    }
}

#[test]
fn plan_and_ira_birth_claims_share_gross_limit_by_tax_relief_not_row_order() {
    for (investment, winner, penalty, total) in [(45000, 3, 1300, 22914), (5000, 2, 2050, 25584)] {
        let mut d = ira_basis_exception_return(gross_birth_claim(0), false);
        let plan = plan_gross_exception(gross_birth_claim(0), 20000);
        let mut entry = plan.income[1].clone();
        let FederalIncome::Retirement { data } = &mut entry.income else { panic!() };
        edit_retirement(data).gross = 1000000;
        edit_retirement(data).taxable = Some(RetirementTaxable::QualifiedWithdrawal {
            data: crate::PlanDistribution {
                value: 5000000,
                investment: investment * 100,
                eligible: true,
            },
        });
        add(&mut d, entry);
        // Put the lower-benefit source first by ID in the first scenario.
        d.income[1].id = 3;
        d.income[2].id = 2;
        let r = supported(&d);
        assert_eq!(value(&r, "birth_adoption_used_0_Synthetic child"), 5000);
        assert_eq!(value(&r, &alloc::format!("retirement_{winner}_qualified_gross")), 5000);
        assert_eq!(value(&r, "early_distribution_tax"), penalty);
        assert_eq!(r.tax, Some(total * 100));
        d.income.reverse();
        assert_eq!(supported(&d).tax, r.tax);
        assert_eq!(supported(&restart(d).draft).tax, r.tax);
    }
}

#[test]
fn plan_gross_exception_requires_allocation_for_overlapping_insurance_exclusion() {
    let mut d = plan_gross_exception(RetirementException::Levy { amount: 1000000 }, 20000);
    public_safety(&mut d, 2, 2000, 2000, PublicSafetyCoverage::Medical);
    unsupported(&d, "Retirement exception allocation");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception { facts, .. }) =
            &mut retirement_edit.ordinary_distribution
        else {
            panic!()
        };
        *facts = RetirementException::Levy { amount: 2000000 };
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 1400000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(r.tax, Some(2089400));
}

#[test]
fn plan_gross_exception_with_zero_taxable_amount_needs_no_partial_allocation() {
    let mut d = plan_gross_exception(RetirementException::Levy { amount: 1000000 }, 100000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).box_2a = 0;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    let r = supported(&d);
    assert_eq!(value(&r, "retirement_2_qualified_gross"), 10000);
    assert_eq!(value(&r, "retirement_2_qualified_taxable"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(r.tax, Some(1757000));
}

#[test]
fn special_ira_repayment_matches_irs_partial_and_full_repayment_examples() {
    for (paid, income, future, tax) in [(3000, 7000, 0, 19814), (5000, 6000, 1000, 19574)] {
        let mut d = special_ira_repayment_example(paid, 20261201);
        // D-2026-10-15-13: actual deposits have no record-count ceiling.
        // Preserve the IRS example's exact total with 101 accepted deposits.
        let payment = if paid == 3000 { 2970 } else { 4950 };
        d.adjustments.as_mut().unwrap().ira_basis[0].special_repayments[0].payments = (0..101)
            .map(|index| crate::IraRepaymentDeposit {
                date: 20261201,
                amount: if index == 100 { paid } else { payment },
            })
            .collect();
        let r = supported(&d);
        assert_eq!(return_line(&r, "4a"), 1250000);
        assert_eq!(return_line(&r, "4b"), income * 100);
        assert_eq!(value(&r, "taxpayer_ira_repayment_year_end"), 0);
        assert_eq!(value(&r, "taxpayer_ira_repayment_qualified_taxable"), 4000);
        assert_eq!(
            next_year_basis_notes(&r, "NOTE_LINE_15C_NEXT_YEAR_BASIS"),
            if future == 0 { vec![] } else { vec![future] }
        );
        assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
        assert_eq!(value(&r, "early_distribution_tax"), 600);
        assert_eq!(value(&r, "early_taxpayer_taxable"), income);
        assert_eq!(value(&r, "early_taxpayer_exceptions"), income - 6000);
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
        let mut reordered = d;
        reordered.income.reverse();
        assert_eq!(supported(&reordered).tax, r.tax);
    }
}

#[test]
fn special_ira_repayment_later_year_changes_original_income_not_year_end() {
    let d = special_ira_repayment_example(3000, 20270601);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_repayment_year_end"), 0);
    assert_eq!(return_line(&r, "4b"), 700000);
    assert_eq!(r.tax, Some(1981400));
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn special_ira_repayment_pools_unrepaid_qualified_distributions_like_irs_example_four() {
    let mut d = special_ira_repayment_example(3000, 20261201);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).ordinary_distribution = Some(retirement_exception(
        20260501,
        RetirementException::DomesticAbuse {
            plan: "Plan A".into(),
            event: 20260101,
            balance: 3000000,
            prior_used: 0,
            prior_plan_used: 0,
            eligible: true,
        },
        false,
    ));
    d.adjustments.as_mut().unwrap().ira_basis[0].prior = 450000;
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_repayment_qualified_gross"), 12500);
    assert_eq!(value(&r, "taxpayer_ira_repayment_qualified_taxable"), 8000);
    assert_eq!(value(&r, "taxpayer_ira_repayment_income_reduction"), 3000);
    assert!(next_year_basis_notes(&r, "NOTE_LINE_15C_NEXT_YEAR_BASIS").is_empty());
    assert_eq!(return_line(&r, "4b"), 500000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(r.tax, Some(1873400));
}

#[test]
fn special_ira_repayment_emergency_uses_gross_limit_and_preserves_nonqualified_tax() {
    let mut d = special_ira_repayment_example(1000, 20261201);
    d.income.remove(1);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception { facts, .. }) =
            &mut retirement_edit.ordinary_distribution
        else {
            panic!()
        };
        *facts = RetirementException::Emergency {
            balance: 1000000,
            history: EmergencyHistory::NONE,
            eligible: true,
        };
        d.adjustments.as_mut().unwrap().ira_basis[0].prior = 100000;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_repayment_qualified_gross"), 1000);
    assert_eq!(value(&r, "taxpayer_ira_repayment_income_reduction"), 800);
    assert_eq!(next_year_basis_notes(&r, "NOTE_LINE_15C_NEXT_YEAR_BASIS"), vec![200]);
    assert_eq!(return_line(&r, "4b"), 320000);
    assert_eq!(value(&r, "early_distribution_tax"), 320);
    assert_eq!(r.tax, Some(1862200));
}

#[test]
fn special_ira_repayment_rejects_wrong_source_duplicate_overpayment_and_dates() {
    let base = special_ira_repayment_example(3000, 20261201);
    for mode in 0..5 {
        let mut d = base.clone();
        let b = &mut d.adjustments.as_mut().unwrap().ira_basis[0];
        match mode {
            0 => b.special_repayments[0].payments[0].date = 20260601,
            1 => b.special_repayments[0].payments[0].date = 20290602,
            2 => b.special_repayments[0].payments[0].amount = 500001,
            3 => b.special_repayments[0].income = 2,
            _ => b.special_repayments.push(b.special_repayments[0].clone()),
        }
        unsupported(&d, "Special IRA repayment");
    }
    // Trustee did not accept them: nothing is credited.
    let mut unsure = base.clone();
    unsure.adjustments.as_mut().unwrap().ira_basis[0].special_repayments[0].confirmed = false;
    let r = supported(&unsure);
    assert!(r.tax > supported(&base).tax);
    let mut boundary = base;
    boundary.adjustments.as_mut().unwrap().ira_basis[0].special_repayments[0].payments[0].date =
        20290601;
    supported(&boundary);
}

#[test]
fn special_ira_repayment_coordinates_current_contribution_deduction() {
    let mut d = special_ira_repayment_example(3000, 20261201);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 8000000, 0);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).gross = 500000;
    let a = d.adjustments.as_mut().unwrap();
    a.iras = ira_facts(750000, 0, Answer::Yes);
    a.ira_basis[0].prior = 0;
    a.ira_basis[0].year_end = 4300000;
    let r = supported(&d);
    // Preliminary $1,500 basis recovery leaves $8,500, then repayment removes
    // $3,000. $85,500 deduction MAGI gives $4,130 deduction after IRS rounding.
    assert_eq!(value(&r, "ira_deduction"), 4130);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 3370);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 1870);
    assert_eq!(return_line(&r, "4b"), 550000);
    assert_eq!(value(&r, "agi"), 81370);
    assert_eq!(value(&r, "early_distribution_tax"), 425);
}

#[test]
fn special_ira_repayment_and_qcd_keep_redeposits_out_of_pretax_capacity() {
    let mut d = special_ira_repayment_example(5000, 20261201);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).gross = 2000000;
    edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Ordinary);
    d.adjustments.as_mut().unwrap().ira_basis[0].prior = 1000000;
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261215, 20000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 15000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 5000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(next_year_basis_notes(&r, "NOTE_LINE_15C_NEXT_YEAR_BASIS"), vec![5000]);
}

#[test]
fn full_discharge_does_not_invent_commercial_annuity_or_insurance_allocation() {
    let mut d = plan_gross_exception(gross_birth_claim(0), 20000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).taxable = Some(RetirementTaxable::FullDischarge {
        data: crate::ContractDischarge { investment: 400000, eligible: true },
    });
    edit_retirement(data).kind = RetirementKind::Nonqualified;
    unsupported(&d, "Income");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).kind = RetirementKind::Pension;
    public_safety(&mut d, 2, 3000, 3000, PublicSafetyCoverage::Medical);
    unsupported(&d, "Retirement exception allocation");
}

#[test]
fn care_distribution_multiple_payments_share_annual_limits_independent_of_row_order() {
    let mut d = care_return(care_claim(3000, 100000, false), 20000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).gross = 500000;
    edit_retirement(data).box_2a = 400000;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    let mut e = d.income[1].clone();
    e.id = 3;
    let FederalIncome::Retirement { data } = &mut e.income else { panic!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception { distributed, .. }) =
            &mut retirement_edit.ordinary_distribution
        else {
            panic!()
        };
        *distributed = 20260701;
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 800000);
    assert_eq!(value(&r, "care_distribution_2_exception"), 2600);
    assert_eq!(value(&r, "care_distribution_3_exception"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 540);
    assert_eq!(r.tax, Some(1999400));
    d.income.swap(1, 2);
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn care_distribution_each_employee_has_separate_premium_and_plan_limits() {
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let source = care_return(care_claim(3000, 100000, false), 20000);
        let mut e = source.income[1].clone();
        e.owner = crate::DocumentOwner::from(owner);
        e.id = d.next_id;
        add(&mut d, e);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "care_distribution_2_exception"), 2600);
    assert_eq!(value(&r, "care_distribution_3_exception"), 2600);
    assert_eq!(value(&r, "early_distribution_tax"), 2680);
    assert_eq!(return_line(&r, "5b"), 3200000);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn care_distribution_requires_participating_plan_and_timely_certified_statement() {
    let d = care_return(care_claim(3000, 100000, false), 20000);
    // No: the exception is not applied, so the 10% additional tax applies.
    let claimed = supported(&d);
    {
        let answer = false;
        let mut unsure = d.clone();
        let FederalIncome::Retirement { data: v } = &mut unsure.income[1].income else { panic!() };
        {
            let mut retirement_edit = edit_retirement(v);
            let Some(RetirementTreatment::Exception {
                facts: RetirementException::LongTermCare { data },
                ..
            }) = &mut retirement_edit.ordinary_distribution
            else {
                panic!()
            };
            data.confirmed = answer;
        }
        let r = supported(&unsure);
        assert!(value(&r, "early_distribution_tax") > value(&claimed, "early_distribution_tax"));
    }
    for case in 1..4 {
        let mut bad = d.clone();
        let FederalIncome::Retirement { data: v } = &mut bad.income[1].income else { panic!() };
        {
            let mut retirement_edit = edit_retirement(v);
            let Some(RetirementTreatment::Exception {
                facts: RetirementException::LongTermCare { data },
                ..
            }) = &mut retirement_edit.ordinary_distribution
            else {
                panic!()
            };
            match case {
                1 => data.statement_filed = 20260701,
                2 => retirement_edit.kind = RetirementKind::Ira,
                _ => retirement_edit.kind = RetirementKind::Nonqualified,
            }
        }
        unsupported(&bad, "Long-term-care distribution");
    }
}

#[test]
fn care_distribution_rejects_inconsistent_annual_source_records() {
    for case in 0..2 {
        let mut d = care_return(care_claim(3000, 100000, false), 20000);
        let mut e = d.income[1].clone();
        e.id = 3;
        let FederalIncome::Retirement { data: v } = &mut e.income else { panic!() };
        {
            let mut retirement_edit = edit_retirement(v);
            let Some(RetirementTreatment::Exception {
                facts: RetirementException::LongTermCare { data },
                ..
            }) = &mut retirement_edit.ordinary_distribution
            else {
                panic!()
            };
            match case {
                0 => data.balance += 100,
                _ => data.premiums += 100,
            }
        }
        add(&mut d, e);
        unsupported(&d, "Long-term-care distribution");
    }
}

#[test]
fn care_distribution_coordinates_two_plans_without_reusing_premiums() {
    let mut d = care_return(care_claim(3000, 10000, false), 20000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    edit_retirement(data).gross = 150000;
    edit_retirement(data).box_2a = 150000;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    let mut other_plan = d.income[1].clone();
    other_plan.id = 3;
    let FederalIncome::Retirement { data } = &mut other_plan.income else { unreachable!() };
    edit_retirement(data).gross = 160000;
    edit_retirement(data).box_2a = 160000;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception {
            facts: RetirementException::LongTermCare { data: facts },
            ..
        }) = &mut retirement_edit.ordinary_distribution
        else {
            unreachable!()
        };
        facts.plan = "Other synthetic plan".into();
        facts.balance = 10000000;
    }
    add(&mut d, other_plan);
    let r = supported(&d);
    assert_eq!(value(&r, "care_distribution_2_exception"), 1000);
    assert_eq!(value(&r, "care_distribution_3_exception"), 1600);
    assert_eq!(value(&r, "care_distribution_2_used"), 1000);
    assert_eq!(value(&r, "care_distribution_3_used"), 1600);
    assert_eq!(value(&r, "early_distribution_tax"), 50);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
    d.income.swap(1, 2);
    assert_eq!(supported(&d).tax, r.tax);

    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    edit_retirement(data).gross = 260000;
    edit_retirement(data).box_2a = 260000;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    // Above $2,600 across the plans the exception stops there (B11 R10), in entry order.
    let r = supported(&d);
    assert_eq!(value(&r, "care_distribution_2_exception"), 1000);
    assert_eq!(value(&r, "care_distribution_3_exception"), 1600);
    assert_eq!(value(&r, "care_owner_taxpayer_used"), 2600);
}

#[test]
fn care_distribution_employee_and_spouse_premiums_share_one_plan_ceiling() {
    let mut d = care_return(care_claim(1500, 100000, false), 20000);
    let joint = complete(FederalStatus::Joint);
    d.setup = joint.setup;
    d.people = joint.people;
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    edit_retirement(data).gross = 500000;
    edit_retirement(data).box_2a = 400000;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    let mut e = d.income[1].clone();
    e.id = 3;
    add(&mut d, e);
    let repeated = supported(&d);
    assert_eq!(value(&repeated, "care_distribution_2_exception"), 1500);
    assert_eq!(value(&repeated, "care_distribution_3_exception"), 0);
    assert_eq!(value(&repeated, "early_distribution_tax"), 650);
    assert_eq!(repeated.tax, Some(1165300));
    let FederalIncome::Retirement { data: v } = &mut d.income[2].income else { panic!() };
    {
        let mut retirement_edit = edit_retirement(v);
        let Some(RetirementTreatment::Exception {
            facts: RetirementException::LongTermCare { data },
            ..
        }) = &mut retirement_edit.ordinary_distribution
        else {
            panic!()
        };
        data.spouse_coverage = true;
    }
    let both = supported(&d);
    assert_eq!(value(&both, "care_distribution_2_exception"), 1500);
    assert_eq!(value(&both, "care_distribution_3_exception"), 1100);
    assert_eq!(value(&both, "early_distribution_tax"), 540);
    assert_eq!(both.tax, Some(1154300));
}

#[test]
fn automatic_enrollment_saver_combines_known_sources_and_manual_lookback() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2300000, 0);
    add(&mut d, automatic_enrollment_income(Owner::Taxpayer, 1000, false));
    add(&mut d, automatic_enrollment_income(Owner::Taxpayer, 1000, false));
    let mut credit = saver(350000, 0);
    credit.distributions.push(FederalSaverDistribution {
        owner: Owner::Taxpayer,
        year: SaverDistributionPeriod::PriorOne { joint_with_current_spouse: false },
        amount: 50000,
    });
    d.credits.as_mut().unwrap().saver = Some(credit).into();
    let r = supported(&d);
    assert_eq!(value(&r, "saver_automatic_withdrawals_taxpayer"), 2000);
    assert_eq!(value(&r, "saver_credit"), 200);
    assert_eq!(r.tax, Some(69300));
    d.income.swap(1, 2);
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn employee_pension_deduction_preserves_wages_and_persisted_source() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().pension_contributions =
        vec![pension_contribution(Owner::Taxpayer, 7000, 60000, 0)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 6000000);
    assert_eq!(value(&r, "pension_contribution_deduction"), 7000);
    assert_eq!(value(&r, "adjustments"), 7000);
    assert_eq!(value(&r, "agi"), 53000);
    assert_eq!(return_line(&r, "10"), 700000);
    assert_eq!(r.tax, Some(418300));
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn employee_pension_uses_per_owner_compensation_and_shared_deferral_limits() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 8000000, 0);
    add(&mut d, wage(2, Owner::Spouse, 2000000, 0));
    d.adjustments.as_mut().unwrap().pension_contributions = vec![
        pension_contribution(Owner::Taxpayer, 7000, 80000, 17500),
        pension_contribution(Owner::Spouse, 5000, 20000, 0),
    ];
    let r = supported(&d);
    assert_eq!(value(&r, "pension_contribution_limit_taxpayer"), 7000);
    assert_eq!(value(&r, "pension_contribution_limit_spouse"), 5000);
    assert_eq!(value(&r, "agi"), 88000);
    assert_eq!(r.tax, Some(620300));
    d.adjustments.as_mut().unwrap().pension_contributions.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    for (index, amount, other) in [(0, 5001, 0), (1, 7001, 0), (1, 7000, 17501)] {
        let mut bad = d.clone();
        let f = &mut bad.adjustments.as_mut().unwrap().pension_contributions[index];
        f.contributions = amount * 100;
        f.other_deferrals = other * 100;
        unsupported(&bad, "pension");
    }
}

#[test]
fn employee_pension_reduces_traditional_ira_capacity_and_preserves_saved_answers() {
    let mut d = complete(FederalStatus::Single);
    let a = d.adjustments.as_mut().unwrap();
    a.pension_contributions = vec![pension_contribution(Owner::Taxpayer, 7000, 60000, 0)];
    a.iras = ira_facts(50000, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 500);
    assert_eq!(value(&r, "adjustments"), 7500);
    assert_eq!(value(&r, "agi"), 52500);
    assert_eq!(r.tax, Some(412300));
    assert!(!d.adjustments.as_ref().unwrap().iras.taxpayer.covered_at_work);
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    d.adjustments.as_mut().unwrap().iras.taxpayer.traditional += 1;
    unsupported(&d, "IRA");
}

#[test]
fn employee_pension_enters_saver_once_and_assumes_no_credit_without_its_facts() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2500000, 0);
    d.adjustments.as_mut().unwrap().pension_contributions =
        vec![pension_contribution(Owner::Taxpayer, 1000, 25000, 0)];
    // Saver's Credit facts not entered: no credit, with a warning.
    let r = supported(&d);
    assert!(noted(&r, "WARN_SAVER_CREDIT_NOT_CLAIMED"), "{r:?}");
    d.credits.as_mut().unwrap().saver = Some(saver(50000, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 24000);
    assert_eq!(value(&r, "saver_credit"), 750);
    assert_eq!(r.tax, Some(4300));
    d.credits.as_mut().unwrap().saver.as_mut().unwrap().distributions.push(
        FederalSaverDistribution {
            owner: Owner::Taxpayer,
            year: SaverDistributionPeriod::TaxYear,
            amount: 50000,
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "saver_credit"), 500);
    assert_eq!(r.tax, Some(29300));
}

#[test]
fn employee_pension_reduces_social_security_provisional_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3000000, 0);
    add(&mut d, income(social_security_net(2000000, 0)));
    d.adjustments.as_mut().unwrap().pension_contributions =
        vec![pension_contribution(Owner::Taxpayer, 7000, 30000, 0)];
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    // Provisional income 30,000 - 7,000 + 10,000; taxable benefits 4,000.
    assert_eq!(value(&r, "taxable_social_security"), 4000);
    assert_eq!(value(&r, "agi"), 27000);
    assert_eq!(value(&r, "saver_credit"), 200);
    assert_eq!(r.tax, Some(89300));
}

#[test]
fn employee_pension_rejects_unconfirmed_duplicate_and_nonjoint_spouse_sources() {
    let mut d = complete(FederalStatus::Single);
    let f = pension_contribution(Owner::Taxpayer, 1000, 60000, 0);
    // Unconfirmed records take no deduction, with a notice.
    d.adjustments.as_mut().unwrap().pension_contributions =
        vec![PensionContributions { confirmed: false, ..f.clone() }];
    let r = supported(&d);
    assert!(noted(&r, "WARN_PENSION_CONTRIBUTIONS_LEFT_OUT"));
    assert_eq!(value(&r, "pension_contribution_deduction"), 0);
    for records in [
        vec![PensionContributions { owner: Owner::Spouse, ..f.clone() }],
        vec![f.clone(), f.clone()],
    ] {
        d.adjustments.as_mut().unwrap().pension_contributions = records;
        unsupported(&d, "pension");
    }
}

#[test]
fn employee_pension_does_not_expand_rental_allowance() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12000000, 0);
    add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
    d.adjustments.as_mut().unwrap().pension_contributions =
        vec![pension_contribution(Owner::Taxpayer, 7000, 120000, 0)];
    let r = supported(&d);
    // Section 469(i)(3)(E) adds back all section 219 deductions.
    assert_eq!(value(&r, "rental_income"), -15000);
    assert_eq!(value(&r, "rental_suspended_loss"), 16000);
    assert_eq!(value(&r, "agi"), 98000);
    assert_eq!(r.tax, Some(1273600));
}

#[test]
fn employee_pension_keeps_unverified_roth_and_partial_phaseout_coordination_explicit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 8600000, 0);
    let a = d.adjustments.as_mut().unwrap();
    a.pension_contributions = vec![pension_contribution(Owner::Taxpayer, 7000, 86000, 0)];
    a.iras = ira_facts(50000, 0, Answer::No);
    unsupported(&d, "IRA");
    // Income above the endpoint has no deduction regardless of limit ordering.
    d.income[0] = wage(1, Owner::Taxpayer, 10000000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 0);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 500);
    assert_eq!(value(&r, "agi"), 93000);
    assert_eq!(r.tax, Some(1163600));
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 50000, Answer::No);
    unsupported(&d, "IRA");
    assert!(estimate(&d).issues.iter().any(|issue| {
        issue.section == "IRA contributions"
            && Some({
                let text = &issue.display;
                text.key.as_str()
            }) == Some("ISSUE_PENSION_ROTH_COORDINATION")
    }));
}

#[test]
fn historical_amt_parameters_cover_exemption_phaseout_and_separate_cap() {
    use crate::income_tax::AmtRates;
    for (year, base, expected) in [
        (2023, 831150, 230515),
        (2023, 851150, 237515),
        (2023, 1084150, 319065),
        (2024, 875950, 242940),
        (2024, 895950, 249940),
        (2024, 1142550, 336250),
        (2025, 900350, 249707),
        (2025, 920350, 256707),
        (2025, 1174350, 345607),
    ] {
        assert_eq!(
            AmtRates::for_year(year, FederalStatus::Separate).unwrap().ordinary_tax(base),
            expected
        );
    }
    for (base, expected) in [(626350, 145928), (646350, 152928), (978750, 269268)] {
        assert_eq!(
            AmtRates::for_year(2025, FederalStatus::Single).unwrap().ordinary_tax(base),
            expected
        );
    }
    assert!(AmtRates::for_year(2021, FederalStatus::Single).is_none());
}

#[test]
fn minimum_credit_2022_separate_surcharge_and_gain_thresholds_are_year_specific() {
    let mut h = repayment_minimum_source(2022);
    h.status = FederalStatus::Separate;
    h.base = 79610000;
    h.exclusions.clear();
    h.regular = 0;
    h.amt = 0;
    assert_eq!(h.balance_for_year(2022, FederalStatus::Separate).unwrap().excluded, 222247);
    let mut h = repayment_minimum_source(2022);
    h.base = 10000000;
    h.exclusions.clear();
    h.regular = 0;
    h.amt = 1000000;
    h.gains = Some(
        MinimumTaxGains {
            preferential: 3000000,
            combined: 3000000,
            regular_zero: 4000000,
            regular_fifteen: 4000000,
        }
        .into(),
    );
    //2022 AMT taxable24,100; 0% room1,675; remaining22,425 at15%=3,364.
    assert_eq!(h.balance_for_year(2022, FederalStatus::Single).unwrap().excluded, 3364);
    assert!(h.balance_for_year(2021, FederalStatus::Single).is_none());
}

#[test]
fn employee_housing_carry_is_owner_specific_and_preserves_ira_compensation() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = empty_return(FederalStatus::Joint);
        let mut pay = income(foreign_pay(160000));
        pay.owner = crate::DocumentOwner::from(owner);
        add(&mut d, pay);
        employee_housing(&mut d, 50000, ForeignHousingLocation::General);
        let ForeignHousing::Claim { prior, .. } = &mut claim_mut(&mut d, owner).housing else {
            unreachable!()
        };
        *prior = Some(HousingCarryover { amount: 1800000, deducted: 800000 });
        let r = supported(&d);
        let person = if owner == Owner::Spouse { "spouse" } else { "taxpayer" };
        assert_eq!(
            value(&r, &alloc::format!("foreign_housing_{person}_prior_deduction_used")),
            8494
        );
        assert_eq!(value(&r, "agi"), 0);
        assert_eq!(return_line(&r, "9"), 849400);
        assert_eq!(return_line(&r, "10"), 849400);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        // Joint IRA compensation is $8,494 before the housing deduction,
        // even though that deduction leaves zero AGI.
        let mut iras = ira_facts(750000, 0, Answer::No);
        iras.spouse = IraSpouse::Joint { facts: ira_facts(0, 0, Answer::No).taxpayer };
        d.adjustments.as_mut().unwrap().iras = iras;
        d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
        let r = supported(&d);
        assert_eq!(value(&r, "ira_deduction"), 7500);
        assert_eq!(value(&r, "agi"), -7500);
        assert_eq!(return_line(&r, "9"), 849400);
        assert_eq!(return_line(&r, "10"), 1599400);
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn separate_amt_holding_period_preserves_each_capital_loss_character() {
    for regular_term in [GainTerm::Short, GainTerm::Long] {
        let alternative_term =
            if regular_term == GainTerm::Short { GainTerm::Long } else { GainTerm::Short };
        let mut sale = ordinary_investment_sale(10000, 20000, regular_term);
        let FederalIncome::Sale { data } = &mut sale else { unreachable!() };
        data.amt = crate::SaleAmt::Different {
            record: CapitalSaleAmt { term: Some(alternative_term), basis: 1500000, wash_sale: 0 },
        };
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(sale));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), -3000);
        assert_eq!(value(&r, "amt_capital_gain"), -3000);
        assert_eq!(
            value(&r, "capital_short_carryforward"),
            if regular_term == GainTerm::Short { 7000 } else { 0 }
        );
        assert_eq!(
            value(&r, "capital_long_carryforward"),
            if regular_term == GainTerm::Long { 7000 } else { 0 }
        );
        assert_eq!(
            value(&r, "amt_capital_short_carryforward"),
            if alternative_term == GainTerm::Short { 2000 } else { 0 }
        );
        assert_eq!(
            value(&r, "amt_capital_long_carryforward"),
            if alternative_term == GainTerm::Long { 2000 } else { 0 }
        );
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn payer_reported_nua_distribution_taxes_only_reported_pension_amount() {
    // Pub. 575: eligible employer-stock NUA stays deferred; the taxable
    // pension amount remains subject to the applicable early-distribution tax.
    // Source: $100,000 gross distribution, $75,000 deferred NUA, $25,000 box 2a.
    let mut d = complete(FederalStatus::Single);
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        100000,
        RetirementTreatment::Early,
    );
    let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
    edit_retirement(data).box_2a = 2500000;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 10000000);
    assert_eq!(return_line(&r, "5b"), 2500000);
    assert_eq!(value(&r, "agi"), 85000);
    assert_eq!(value(&r, "early_distribution_tax"), 2500);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn nua_sales_split_holding_period_and_exclude_only_deferred_gain_from_niit() {
    // Pub. 575 and Form 8960: $25,000 basis, $75,000 deferred NUA.
    for distributed in [20250101, 20260101] {
        for (proceeds, gain, niit) in [(120000, 95000, 760), (60000, 35000, 0), (20000, -3000, 0)] {
            let lot = NuaSale {
                inheritance: None,
                distributed,
                sold: 20260701,
                distribution_value: 10000000,
                deferred_nua: 7500000,
                proceeds: proceeds * 100,
                wash_sale: 0,
                withholding: 0,
                reported_basis: None,
                reporting: crate::BasisReporting::ReportedToIrs,
                special: false,
            };
            assert!(lot.valid());
            let parts = lot.parts();
            assert_eq!(parts[0].term, GainTerm::Long);
            assert_eq!(
                parts[1].term,
                if distributed == 20250101 { GainTerm::Long } else { GainTerm::Short }
            );
            assert_eq!(lot.realized_nua(), ((proceeds - 25000) * 100).clamp(0, 7500000));
            let mut d = complete(FederalStatus::Single);
            d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
            add(&mut d, income(FederalIncome::NuaSale { data: lot }));
            let r = supported(&d);
            assert_eq!(value(&r, "capital_gain"), gain);
            assert_eq!(value(&r, "net_investment_income_tax"), niit);
            assert_eq!(value(&r, "agi"), 300000 + gain);
            assert_eq!(supported(&restart(d).draft), r);
        }
    }
}

#[test]
fn nua_sale_rejects_inconsistent_lot_and_unconfirmed_treatment() {
    let mut lot = NuaSale {
        inheritance: None,
        distributed: 20260101,
        sold: 20260701,
        distribution_value: 10000000,
        deferred_nua: 7500000,
        proceeds: 2000000,
        wash_sale: 500000,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        special: true,
    };
    assert!(lot.valid());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::NuaSale { data: lot.clone() }));
    assert!(noted(&supported(&d), "WARN_NUA_SALE_LEFT_OUT"));
    lot.wash_sale += 1;
    assert!(!lot.valid());
    lot.wash_sale = 0;
    lot.deferred_nua = lot.distribution_value + 1;
    assert!(!lot.valid());
}

#[test]
fn nua_exclusion_nets_with_property_portfolio_losses_and_prior_niit_difference() {
    // Form 8960 net-gains worksheet 2(e), 2(h), 2(i): combine exclusions
    // before limiting the retained difference. $95k gain includes $75k NUA.
    for property in [None, Some(false), Some(true)] {
        for loss in [10000, 120000] {
            for prior in [false, true] {
                let mut d = property.map_or_else(
                    || complete(FederalStatus::Single),
                    |material| complete_property_gain_draft(material, 0),
                );
                d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
                add(
                    &mut d,
                    income(FederalIncome::NuaSale {
                        data: NuaSale {
                            inheritance: None,
                            distributed: 20260101,
                            sold: 20260701,
                            distribution_value: 10000000,
                            deferred_nua: 7500000,
                            proceeds: 12000000,
                            wash_sale: 0,
                            withholding: 0,
                            reported_basis: None,
                            reporting: crate::BasisReporting::ReportedToIrs,
                            special: false,
                        },
                    }),
                );
                add(&mut d, income(ordinary_investment_sale(0, loss, GainTerm::Short)));
                if prior {
                    d.screening.as_mut().unwrap().capital_history =
                        vec![niit_loss_history(Some(7000))];
                }
                let r = supported(&d);
                let property_gain = if property.is_some() { 20000 } else { 0 };
                let excluded_property = if property == Some(true) { 20000 } else { 0 };
                let net = 95000 + property_gain - loss - if prior { 14000 } else { 0 };
                let capital = net.max(-3000);
                let adjustment = -75000 - excluded_property + if prior { 7000 } else { 0 };
                assert_eq!(value(&r, "capital_gain"), capital);
                assert_eq!(value(&r, "niit_disposition_adjustment"), adjustment);
                assert_eq!(value(&r, "niit_capital_carry_adjustment"), 0);
                assert_eq!(
                    value(&r, "net_investment_income_tax"),
                    (capital + adjustment).max(0) * 38 / 1000
                );
                assert_eq!(supported(&restart(d).draft), r);
            }
        }
    }
}

#[test]
fn nua_lump_sum_event_dates_and_participant_status_control_qualification() {
    let age = NuaEvent::Age;
    let born = Some(19670101);
    assert!(!age.qualifies(20260630, born));
    assert!(age.qualifies(20260701, born));
    assert!(age.qualifies(20260702, born));
    for event in [
        NuaEvent::Death { date: 20260701 },
        NuaEvent::Separation { date: 20260701, employee: true },
        NuaEvent::Disability { date: 20260701, eligible: true },
    ] {
        assert!(!event.qualifies(20260630, None));
        assert!(event.qualifies(20260701, None));
        assert!(!event.qualifies(20250701, None));
    }
    assert!(!NuaEvent::Separation { date: 20260101, employee: false }.qualifies(20260701, None));
    assert!(!NuaEvent::Disability { date: 20260101, eligible: false }.qualifies(20260701, None));
    // Without a birth date on People, the age event cannot qualify.
    assert!(!NuaEvent::Age.qualifies(20260701, None));
    assert_eq!(age.date(born), Some(20260701));
    assert_eq!(age.date(None), None);
}

#[test]
fn distributed_property_rollover_matches_all_four_irs_proceeds_examples() {
    // Pub. 575 Rollovers of property, examples 1-4; dates moved to 2026.
    for (proceeds, rolled, pension, gain) in [
        (60000, 60000, 0, 0),
        (40000, 40000, 0, 0),
        (60000, 45000, 12500, 2500),
        (40000, 25000, 18750, -3750),
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            50000,
            RetirementTreatment::Rollover {
                destination: RolloverDestination::Ira,
                amount: rolled * 100,
                early: false,
            },
        );
        let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
        edit_retirement(data).taxable = Some(RetirementTaxable::PropertyProceeds {
            data: PropertyRollover {
                distributed: 20260904,
                sold: 20260924,
                proceeds: proceeds * 100,
                confirmed: true,
            },
        });
        add(&mut d, entry);
        let id = d.income.last().unwrap().id;
        assert!(d.valid_stored());
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), 5000000);
        assert_eq!(return_line(&r, "5b"), pension * 100);
        assert_eq!(
            value(&r, &alloc::format!("property_rollover_{id}_retained_capital_gain")),
            gain
        );
        assert_eq!(value(&r, "capital_gain"), gain.max(-3000));
        assert_eq!(value(&r, "capital_short_carryforward"), (-gain - 3000).max(0));
        assert_eq!(value(&r, "agi"), 60000 + pension + gain.max(-3000));
        assert_eq!(value(&r, "capital_gain"), value(&r, "amt_capital_gain"));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
        let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Rollover {
            destination: RolloverDestination::Ira,
            amount: rolled * 100,
            early: true,
        });
        let high = supported(&d);
        assert_eq!(value(&high, "early_distribution_tax"), pension / 10);
        assert_eq!(value(&high, "net_investment_income_tax"), gain.max(0) * 38 / 1000);
        let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        if let Some(RetirementTreatment::Rollover { amount, .. }) =
            &mut edit_retirement(data).ordinary_distribution
        {
            *amount = proceeds * 100 + 1;
        }
        assert!(!d.valid_stored());
    }
}

#[test]
fn deferred_annuity_combines_required_contracts_before_earnings_first_allocation() {
    // Paying contract: value $9k, investment $10k. Related contract: $15k/$10k.
    // A $7k withdrawal has $4k combined earnings, not zero from the payer alone.
    let mut d = deferred_withdrawal(9000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTaxable::DeferredWithdrawal {
            data: crate::AnnuityDistribution { related, .. },
        }) = &mut retirement_edit.taxable
        else {
            unreachable!()
        };
        related.push(AnnuityBalance { value: 1500000, investment: 1000000 });
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 700000);
    assert_eq!(return_line(&r, "5b"), 400000);
    assert_eq!(value(&r, "annuity_2_basis_recovery"), 3000);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 17000);
    assert_eq!(value(&r, "agi"), 214000);
    assert_eq!(value(&r, "early_distribution_tax"), 400);
    assert_eq!(value(&r, "net_investment_income_tax"), 152);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // Insurer-reconciled next-payment balances: primary $15k/$10k,
    // related $2k/$7k; the group's $17k value equals its remaining investment.
    let mut next = d.income[1].clone();
    let FederalIncome::Retirement { data } = &mut next.income else { unreachable!() };
    edit_retirement(data).gross = 800000;
    edit_retirement(data).taxable = Some(RetirementTaxable::DeferredWithdrawal {
        data: crate::AnnuityDistribution {
            loan_pledge: false,
            value: 1500000,
            investment: 1000000,
            related: vec![AnnuityBalance { value: 200000, investment: 700000 }],
            eligible: true,
        },
    });
    add(&mut d, next);
    let id = d.income.last().unwrap().id;
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 1500000);
    assert_eq!(return_line(&r, "5b"), 400000);
    assert_eq!(value(&r, &alloc::format!("annuity_{id}_basis_recovery")), 8000);
    assert_eq!(value(&r, &alloc::format!("annuity_{id}_remaining_basis")), 9000);
    assert_eq!(value(&r, "net_investment_income_tax"), 152);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn deferred_annuity_related_loss_offsets_earnings_and_invalid_balances_reject() {
    let mut d = deferred_withdrawal(16000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTaxable::DeferredWithdrawal {
            data: crate::AnnuityDistribution { related, .. },
        }) = &mut retirement_edit.taxable
        else {
            unreachable!()
        };
        related.push(AnnuityBalance { value: 400000, investment: 1000000 });
    }
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "annuity_2_basis_recovery"), 7000);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 13000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
}

#[test]
fn legacy_annuity_withdrawals_follow_four_layers_and_old_earnings_exception() {
    // Old investment $10k + earnings $5k; later investment $20k + earnings $10k.
    for (gross, taxable, early, old_left, new_left) in [
        (5000, 0, 0, 5000, 20000),
        (12000, 2000, 0, 0, 20000),
        (18000, 8000, 300, 0, 20000),
        (30000, 15000, 1000, 0, 15000),
        (44000, 15000, 1000, 0, 1000),
    ] {
        let mut d = deferred_withdrawal(45000, 30000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        edit_retirement(data).gross = gross * 100;
        edit_retirement(data).taxable = Some(RetirementTaxable::LegacyAnnuity {
            data: LegacyAnnuity {
                issued: 19820813,
                before: AnnuityBalance { value: 1500000, investment: 1000000 },
                after: AnnuityBalance { value: 3000000, investment: 2000000 },
                confirmed: true,
            },
        });
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), gross * 100);
        assert_eq!(return_line(&r, "5b"), taxable * 100);
        assert_eq!(value(&r, "agi"), 210000 + taxable);
        assert_eq!(value(&r, "early_distribution_tax"), early);
        assert_eq!(value(&r, "net_investment_income_tax"), taxable * 38 / 1000);
        assert_eq!(value(&r, "legacy_annuity_2_old_basis_remaining"), old_left);
        assert_eq!(value(&r, "legacy_annuity_2_new_basis_remaining"), new_left);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        edit_retirement(data).ordinary_distribution = Some(retirement_exception(
            20260615,
            RetirementException::Disability { began: 20260101 },
            false,
        ));
        assert_eq!(value(&supported(&d), "early_distribution_tax"), 0);
    }
}

#[test]
fn legacy_annuity_requires_old_issue_date_and_reconciled_portions() {
    let mut lot = LegacyAnnuity {
        issued: 19820813,
        before: AnnuityBalance { value: 1500000, investment: 1000000 },
        after: AnnuityBalance { value: 0, investment: 0 },
        confirmed: true,
    };
    assert_eq!(lot.parts(1200000), [1000000, 200000, 0, 0]);
    assert!(lot.confirmed());
    assert!(!lot.valid(1500001));
    for issued in [19820814, 19830101] {
        lot.issued = issued;
        let mut d = deferred_withdrawal(15000, 10000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        edit_retirement(data).taxable =
            Some(RetirementTaxable::LegacyAnnuity { data: lot.clone() });
        assert_eq!(estimate(&d).state, ResultState::Unsupported);
    }
    lot.issued = 19820813;
    lot.before.value = 900000;
    assert!(!lot.confirmed());
    lot.before.value = 1500000;
    lot.confirmed = false;
    assert!(!lot.confirmed());
    // No: the whole withdrawal is taxed with no investment recovered, with
    // a notice.
    lot.issued = 19800101;
    let mut d = deferred_withdrawal(15000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let gross = crate::FederalRetirement::from(&*data).gross;
    edit_retirement(data).taxable = Some(RetirementTaxable::LegacyAnnuity { data: lot });
    let r = supported(&d);
    assert!(noted(&r, "WARN_RETIREMENT_BASIS_LEFT_OUT"));
    assert_eq!(return_line(&r, "5b"), gross);
}

#[test]
fn rmd_correction_and_waiver_apply_as_entered_without_confirmations() {
    // B11 C6: the dates and amounts decide; there is no confirmation to answer.
    let mut d = complete(FederalStatus::Single);
    let mut e = rmd_record(5000, 1000);
    e.correction = RmdCorrection::Corrected {
        received: 20270115,
        filed: RmdFiling::Submitted { date: 20270415 },
        notice: None,
        assessed: None,
    };
    d.adjustments.as_mut().unwrap().required_distributions = vec![e];
    // Shortfall $4,000 corrected inside the window: 10%.
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 400);
    assert!(!noted(&r, "WARN_RMD_CORRECTION_ASSUMED_INCOMPLETE"));
    let e = &mut d.adjustments.as_mut().unwrap().required_distributions[0];
    e.correction = RmdCorrection::Uncorrected;
    e.waiver = Some(RmdWaiver {
        amount: 150000,
        explanation: "Custodian missed instruction; transfer corrected.".into(),
    });
    // The $1,500 waiver comes off the shortfall; $2,500 at 25%.
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 625);
}

#[test]
fn retirement_assumption_notices_keep_sources_groups_and_filers_distinct() {
    let mut draft = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for _ in 0..2 {
            let mut entry = retirement_exception_entry(
                owner,
                RetirementKind::Pension,
                1000,
                20260201,
                RetirementException::Separation {
                    separated: 20260101,
                    work: SeparationWork::PublicSafety { service_years: 25, eligible: false },
                },
                false,
            );
            entry.label = "Retirement plan".into();
            add(&mut draft, entry);
        }
        add(&mut draft, roth_withdrawal(owner, 1000, 20260101));
        let mut group = rmd_record(5000, 1000);
        group.owner = owner;
        group.label = "Retirement plan".into();
        group.correction = RmdCorrection::Corrected {
            received: 20270115,
            filed: RmdFiling::Submitted { date: 20270415 },
            notice: None,
            assessed: None,
        };
        group.waiver =
            Some(RmdWaiver { amount: 100000, explanation: "Custodian corrected transfer".into() });
        draft.adjustments.as_mut().unwrap().required_distributions.extend([group.clone(), group]);
    }
    let result = supported(&draft);
    for (key, count) in [("WARN_ROTH_HISTORY_ASSUMED_NONE", 2)] {
        let notes: Vec<_> =
            result.review_notes.iter().filter(|note| note.display.key == key).collect();
        assert_eq!(notes.len(), count, "{key}: {result:?}");
        let contexts: Vec<_> = notes.iter().map(|note| note.context.as_ref().unwrap()).collect();
        for (index, context) in contexts.iter().enumerate() {
            assert_eq!(
                crate::interview::issue_page(&draft, context, None),
                Some(TaxSection::Retirement)
            );
            assert!(contexts[..index].iter().all(|other| other.identity() != context.identity()));
        }
    }
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut history = roth_history(owner, 2022, 0);
        history.ordering = None;
        draft.adjustments.as_mut().unwrap().roth_history.push(history);
    }
    let with_history = supported(&draft);
    assert!(!noted(&with_history, "WARN_ROTH_HISTORY_ASSUMED_NONE"));
    assert_eq!(
        with_history
            .review_notes
            .iter()
            .filter(|note| note.display.key == "WARN_ROTH_BASIS_ASSUMED_ZERO")
            .count(),
        2
    );
}

#[test]
fn payer_reported_tax_free_annuity_exchange_preserves_zero_income() {
    // 2026 Form 1099-R instructions: section 1035 exchange, code 6,
    // contract value in box 1 and an explicit zero in box 2a.
    let mut d = deferred_withdrawal(16000, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    edit_retirement(data).gross = 1600000;
    edit_retirement(data).box_2a = 0;
    edit_retirement(data).taxable = Some(RetirementTaxable::Payer);
    edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Ordinary);
    edit_retirement(data).withholding = 0;
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "agi"), 210000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    let mut without_exchange = d.clone();
    without_exchange.income.pop();
    assert_eq!(r.tax, supported(&without_exchange).tax);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn railroad_prior_repayment_reduces_pension_not_unemployment() {
    let d = railroad_repayment_draft();
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 27500);
    assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 20000);
    assert_eq!(value(&r, "repayment_year_0_refigured_retirement_taxable"), 10000);
    assert_eq!(value(&r, "repayment_credit"), 766);
    assert_eq!(value(&r, "railroad_prior_year_repayments"), 5000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn railroad_total_disability_wages_stay_separate_from_sseb() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income.clear();
    add(&mut d, income(disability_pension(30000)));
    add(&mut d, income(social_security_net(1000000, 0)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "1h"), 3000000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(return_line(&r, "6a"), 1000000);
    assert_eq!(return_line(&r, "6b"), 535000);
    assert_eq!(value(&r, "agi"), 35350);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn railroad_spouse_annuity_does_not_share_employee_cost_recovery() {
    let mut d = complete(FederalStatus::Joint);
    let people = d.people.as_mut().unwrap();
    people.taxpayer.birth_date = 19600101;
    let mut spouse = person();
    spouse.birth_date = 19600101;
    people.spouse = SpouseFiler::Yes { facts: spouse };
    let basis = PensionBasis {
        earlier_months: 0,
        changes: vec![],
        share: None,
        started: 20260101,
        cost: Some(2600000),
        months: 12,
        history: PensionHistory::New { duration: PensionDuration::Single { age: 65 } },
        eligible: true,
    };
    add(&mut d, railroad_pension(12000, 15000, Some(basis)));
    let mut spouse_annuity = railroad_pension(6000, 7000, None);
    spouse_annuity.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse_annuity);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 2200000);
    assert_eq!(return_line(&r, "5b"), 2080000);
    assert_eq!(value(&r, "pension_2_income_exclusion"), 1200);
    assert_eq!(value(&r, "agi"), 120800);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.swap(1, 2);
    let reordered = supported(&d);
    assert_eq!(return_line(&reordered, "5b"), return_line(&r, "5b"));
    assert_eq!(reordered.tax, r.tax);
}

#[test]
fn simultaneous_pension_annuitants_share_common_exclusion_and_cost_limit() {
    // Pub. 721: $150 common monthly exclusion; $840/$1500 spouse share.
    let mut p = PensionBasis {
        earlier_months: 0,
        changes: vec![],
        share: Some(PensionShare { preceding: 0, payment: 84000, total: 150000 }),
        started: 20230101,
        cost: Some(4500000),
        months: 12,
        history: PensionHistory::Prior { monthly: 15000, recovered: 330000 },
        eligible: true,
    };
    assert!(p.valid());
    assert_eq!(p.monthly(), 8400);
    assert_eq!(p.recovery(), 100800);
    assert_eq!(p.remaining_after_payments(), 3990000);
    let mut child = p.clone();
    child.share = Some(PensionShare { preceding: 0, payment: 33000, total: 150000 });
    assert_eq!(child.monthly(), 3300);
    assert_eq!(p.recovery() + 2 * child.recovery(), 180000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, pension_entry(p.clone(), 10080));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 907200);
    assert_eq!(value(&r, "pension_2_monthly_exclusion"), 84);
    assert_eq!(value(&r, "pension_2_common_monthly_exclusion"), 150);
    assert_eq!(value(&r, "pension_2_remaining_basis"), 39900);
    assert_eq!(supported(&restart(d).draft), r);
    // Only $150 remains across all recipients, not $150 for each recipient.
    p.history = PensionHistory::Prior { monthly: 15000, recovered: 4485000 };
    child.history = p.history;
    assert_eq!(p.recovery(), 8400);
    assert_eq!(child.recovery(), 3300);
    assert_eq!(p.recovery() + 2 * child.recovery(), 15000);
    assert_eq!(p.remaining_after_payments(), 0);
    p.history = PensionHistory::Prior { monthly: 15000, recovered: 4500000 };
    assert_eq!(p.recovery(), 0);
}

#[test]
fn simultaneous_pension_share_validates_source_amounts_and_preserves_old_cap_rule() {
    let mut p = pension_basis();
    p.share = Some(PensionShare { preceding: 0, payment: 101, total: 100 });
    assert!(!p.valid());
    p.share = Some(PensionShare { preceding: 0, payment: 50000, total: 100000 });
    assert!(p.valid());
    assert_eq!(p.monthly(), 5000);
    p.started = 19860702;
    p.cost = Some(100);
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 0 };
    assert!(p.valid());
    assert_eq!(p.recovery(), 60000); // Pre-1987 exclusions may exceed cost.
    p.history = PensionHistory::Prior { monthly: 100001, recovered: 0 };
    assert!(!p.valid()); // Deferred excess exclusion requires another worksheet.
    p = pension_basis();
    p.share = Some(PensionShare { preceding: 0, payment: 100000, total: 300000 });
    assert!(p.valid());
    assert_eq!(p.monthly(), 3333);
    p.started = 20250101;
    p.history = PensionHistory::Prior { monthly: 10000, recovered: p.cost() - 1 };
    p.share = Some(PensionShare { preceding: 0, payment: 50000, total: 100000 });
    assert!(p.valid());
    assert_eq!(p.recovery(), 1);
}

#[test]
fn shared_pension_rounding_conserves_common_recovery() {
    // Exhaust small values, unequal shares, tiny shares and the final cost cap.
    for common in 0..30 {
        for remaining in 0..40 {
            let mut preceding = 0;
            let mut monthly_total = 0;
            let mut recovered_total = 0;
            for payment in [1, 7, 11, 81] {
                let mut p = pension_basis();
                p.started = 20250101;
                p.cost = Some(10000);
                p.months = 3;
                p.history = PensionHistory::Prior { monthly: common, recovered: 10000 - remaining };
                p.share = Some(PensionShare { preceding, payment, total: 100 });
                assert!(p.valid());
                monthly_total += p.monthly();
                recovered_total += p.recovery();
                assert!(p.recovery() <= p.monthly() * 3);
                preceding += payment;
            }
            assert_eq!(monthly_total, common);
            assert_eq!(recovered_total, (common * 3).min(remaining));
        }
    }
    let mut p = pension_basis();
    p.share = Some(PensionShare { preceding: 60000, payment: 50000, total: 100000 });
    assert!(!p.valid());
}

#[test]
fn fractional_pension_shares_survive_return_and_restart() {
    let mut total = 0;
    for (preceding, expected) in [(0, 39996), (100000, 40008), (200000, 39996)] {
        let mut p = pension_basis();
        p.share = Some(PensionShare { preceding, payment: 100000, total: 300000 });
        let mut d = complete(FederalStatus::Single);
        add(&mut d, pension_entry(p.clone(), 12000));
        let r = supported(&d);
        assert_eq!(p.recovery(), expected);
        assert_eq!(return_line(&r, "5b"), crate::kernel::ratio::dollars(1200000 - expected) * 100);
        assert_eq!(supported(&restart(d).draft), r);
        total += expected;
    }
    assert_eq!(total, 120000);
}

#[test]
fn pension_allocation_changes_follow_entitlement_months_and_cost_exhaustion() {
    let mut p = pension_basis();
    p.started = 20230101;
    p.cost = Some(4500000);
    p.history = PensionHistory::Prior { monthly: 15000, recovered: 330000 };
    p.share = Some(PensionShare { preceding: 0, payment: 84000, total: 150000 });
    // Two children initially; one leaves in month 7, the other in month 10.
    p.changes = vec![
        PensionChange { month: 10, share: None },
        PensionChange {
            month: 7,
            share: Some(PensionShare { preceding: 0, payment: 84000, total: 117000 }),
        },
    ];
    assert!(p.valid());
    assert_eq!(p.recovery(), 127707);
    assert_eq!(p.monthly(), 15000);
    assert_eq!(p.remaining_after_payments(), 3990000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, pension_entry(p.clone(), 10080));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 880300);
    assert_eq!(value(&r, "pension_2_monthly_exclusion"), 150);
    assert_eq!(supported(&restart(d).draft), r);
    p.changes.reverse();
    assert_eq!(p.recovery(), 127707);
    p.history = PensionHistory::Prior { monthly: 15000, recovered: 4400000 };
    assert_eq!(p.recovery(), 57579);
    assert_eq!(p.remaining_after_payments(), 0);
    p.changes.push(p.changes[0]);
    assert!(!p.valid());
    p.changes.pop();
}

#[test]
fn survivor_pension_consumes_earlier_same_year_common_recovery() {
    let mut p = pension_basis();
    p.started = 20250101;
    p.earlier_months = 6;
    p.months = 6;
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 3030000 };
    assert!(p.valid());
    assert_eq!(p.remaining(), 10000);
    assert_eq!(p.recovery(), 10000);
    assert_eq!(p.remaining_after_payments(), 0);
    let mut d = complete(FederalStatus::Single);
    let mut entry = pension_entry(p.clone(), 3600);
    let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
    edit_retirement(data).ordinary_distribution =
        Some(retirement_exception(20260701, RetirementException::Death { died: 20260615 }, false));
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 350000);
    assert_eq!(value(&r, "pension_2_earlier_common_recovery"), 600);
    assert_eq!(value(&r, "pension_2_remaining_basis"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(supported(&restart(d).draft), r);
    p.earlier_months = 7;
    assert!(!p.valid()); // Cannot recover thirteen payment months in one year.
    p.months = 5;
    assert!(p.valid());
    assert_eq!(p.recovery(), 0); // Earlier recipients exhausted the cost.
    p.started = 19860702;
    p.cost = Some(100);
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 0 };
    assert!(p.valid());
    assert_eq!(p.recovery(), 50000); // Pre-1987 recovery remains uncapped.
}

#[test]
fn new_annuity_earlier_payment_months_respect_start_and_shared_cap() {
    let mut p = pension_basis();
    p.started = 20260701;
    p.months = 3;
    p.earlier_months = 3;
    assert!(p.valid());
    assert_eq!(p.remaining_after_payments(), 3040000);
    p.earlier_months = 4;
    assert!(!p.valid());
    p.earlier_months = u32::MAX;
    assert!(!p.valid());
    p.started = 20250101;
    p.months = 6;
    p.earlier_months = 6;
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 3030000 };
    p.share = Some(PensionShare { preceding: 0, payment: 50000, total: 100000 });
    assert!(p.valid());
    assert_eq!(p.recovery(), 5000);
}

#[test]
fn civil_service_death_lump_sum_reproduces_irs_worksheet_and_taxable_interest() {
    for (gross, recovered, taxable, remaining) in [
        (38400, 1000, 0, 5600), // Pub. 721 Worksheet D.
        (50000, 1000, 6000, 0),
        (50000, 0, 5000, 0), // Employee dies before a survivor annuity is paid.
        (10000, 45000, 10000, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut entry = retirement_exception_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            gross,
            20260601,
            RetirementException::Death { died: 20260101 },
            false,
        );
        let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
        edit_retirement(data).taxable = Some(RetirementTaxable::CivilServiceDeath {
            data: crate::SurvivorLumpSum {
                cost: 4500000,
                recovered: recovered * 100,
                eligible: true,
            },
        });
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), gross * 100);
        assert_eq!(return_line(&r, "5b"), taxable * 100);
        assert_eq!(value(&r, "civil_service_death_2_unrecovered_cost"), remaining);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        edit_retirement(data).ordinary_distribution =
            Some(rollover(gross, RolloverDestination::Ira));
        unsupported(&d, "Income:");
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        {
            let mut retirement_edit = edit_retirement(data);
            let Some(RetirementTaxable::CivilServiceDeath {
                data: crate::SurvivorLumpSum { recovered, .. },
            }) = &mut retirement_edit.taxable
            else {
                unreachable!()
            };
            *recovered = 4500001;
        }
        assert!(!d.valid_stored());
    }
}

#[test]
fn final_annuity_deduction_requires_matching_general_and_variable_records() {
    let mut fixed = fixed_annuity();
    fixed.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 0 };
    fixed.started = 20250101;
    fixed.recovered = 150000;
    let mut variable = variable_annuity();
    variable.started = 20250101;
    variable.history = VariableHistory::Prior { annual: 60000, recovered: 60000, refigure: None };
    for (taxable, started, cost, recovered) in [
        (
            RetirementTaxable::GeneralAnnuity { data: fixed },
            fixed.started,
            fixed.cost,
            fixed.recovered,
        ),
        (
            RetirementTaxable::VariableAnnuity { data: variable },
            variable.started,
            variable.cost,
            60000,
        ),
        (RetirementTaxable::VariableAnnuity { data: variable_annuity() }, 20260101, 1200000, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.died =
            Some(FinalReturn { date: 20261231, bonds: None, ordinary: true, estate_medical: 0 });
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            3000,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data } = &mut entry.income {
            edit_retirement(data).taxable = Some(taxable);
        }
        add(&mut d, entry);
        d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
            income_id: 2,
            started,
            cost,
            recovered,
            refunds: 0,
            eligible: true,
        });
        let result = supported(&d);
        assert!(value(&result, "annuity_loss_2") > 0);
        assert_eq!(restart(d.clone()).draft, d);
        for field in 0..3 {
            let mut mismatch = d.clone();
            let loss = &mut mismatch.deductions.as_mut().unwrap().annuity_losses[0];
            match field {
                0 => loss.started += 1,
                1 => loss.cost += 100,
                _ => loss.recovered += 100,
            }
            unsupported(&mismatch, "Annuity loss");
        }
    }
}

#[test]
fn final_annuity_deduction_rejects_fixed_period_despite_cessation_confirmation() {
    let mut variable = variable_annuity();
    variable.history = VariableHistory::New { expected: VariablePayments::Fixed { payments: 20 } };
    let mut prior_variable = variable_annuity();
    prior_variable.started = 20250101;
    prior_variable.history = VariableHistory::Prior {
        annual: 60000,
        recovered: 30000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 30000,
            remaining: VariablePayments::Fixed { payments: 19 },
        }),
    };
    let mut pension = pension_basis();
    pension.history = PensionHistory::New { duration: PensionDuration::Fixed { payments: 240 } };
    for (taxable, kind, cost, started, recovered) in [
        (
            RetirementTaxable::GeneralAnnuity { data: fixed_annuity() },
            RetirementKind::Nonqualified,
            3000000,
            20260101,
            0,
        ),
        (
            RetirementTaxable::VariableAnnuity { data: variable },
            RetirementKind::Nonqualified,
            1200000,
            20260101,
            0,
        ),
        (
            RetirementTaxable::Simplified { data: pension.clone() },
            RetirementKind::Pension,
            pension.cost(),
            20260101,
            0,
        ),
        (
            RetirementTaxable::VariableAnnuity { data: prior_variable },
            RetirementKind::Nonqualified,
            1200000,
            20250101,
            30000,
        ),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.died =
            Some(FinalReturn { date: 20261231, bonds: None, ordinary: true, estate_medical: 0 });
        let mut entry =
            retirement_entry(Owner::Taxpayer, kind, 3000, RetirementTreatment::Ordinary);
        if let FederalIncome::Retirement { data } = &mut entry.income {
            edit_retirement(data).taxable = Some(taxable);
        }
        add(&mut d, entry);
        supported(&d);
        d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
            income_id: 2,
            started,
            cost,
            recovered,
            refunds: 0,
            eligible: true,
        });
        unsupported(&d, "Annuity loss");
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn variable_annuity_final_year_prorates_exclusion_and_links_remaining_cost() {
    for (frequency, payments, gross, exclusion) in [
        (AnnuityFrequency::Monthly, 6, 90000, 60000),
        (AnnuityFrequency::Quarterly, 2, 90000, 60000),
        (AnnuityFrequency::Monthly, 6, 30000, 30000),
    ] {
        let mut a = variable_annuity();
        a.started = 20200101;
        a.frequency = frequency;
        a.payments = payments;
        a.history = VariableHistory::Prior { annual: 120000, recovered: 300000, refigure: None };
        assert!(a.valid());
        assert_eq!(a.excluded(gross), exclusion);
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.died =
            Some(FinalReturn { date: 20260630, bonds: None, ordinary: true, estate_medical: 0 });
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            gross / 100,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data } = &mut entry.income {
            edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
        }
        add(&mut d, entry);
        d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
            income_id: 2,
            started: a.started,
            cost: a.cost,
            recovered: a.recovered(),
            refunds: 0,
            eligible: true,
        });
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), gross - exclusion);
        assert_eq!(value(&r, "variable_annuity_2_basis_recovery"), exclusion / 100);
        assert_eq!(value(&r, "annuity_loss_2"), (900000 - exclusion) / 100);
        assert_eq!(restart(d.clone()).draft, d);
        d.deductions.as_mut().unwrap().annuity_losses.clear();
        d.people.as_mut().unwrap().taxpayer.died = None;
        unsupported(&d, "Income:");
        d.people.as_mut().unwrap().taxpayer.died =
            Some(FinalReturn { date: 20260115, bonds: None, ordinary: true, estate_medical: 0 });
        unsupported(&d, "Income:");
    }
}

#[test]
fn variable_annuity_final_year_without_installments_preserves_unrecovered_cost() {
    for new in [false, true] {
        let mut a = variable_annuity();
        a.payments = 0;
        if !new {
            a.started = 20200101;
            a.period = 20260701; // Scheduled period after the owner's death.
            a.history = VariableHistory::Prior { annual: 60000, recovered: 300000, refigure: None };
        }
        assert!(a.valid());
        assert!(a.schedule_valid(Some(20260630)));
        assert!(!a.schedule_valid(None));
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.died =
            Some(FinalReturn { date: 20260630, bonds: None, ordinary: true, estate_medical: 0 });
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            0,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data } = &mut entry.income {
            edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
        }
        add(&mut d, entry);
        d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
            income_id: 2,
            started: a.started,
            cost: a.cost,
            recovered: a.recovered(),
            refunds: 0,
            eligible: true,
        });
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), 0);
        assert_eq!(value(&r, "variable_annuity_2_basis_recovery"), 0);
        assert_eq!(value(&r, "annuity_loss_2"), if new { 12000 } else { 9000 });
        assert_eq!(restart(d.clone()).draft, d);
        if let FederalIncome::Retirement { data } = &mut d.income[1].income {
            edit_retirement(data).gross = 100;
        }
        unsupported(&d, "Income:");
        if let FederalIncome::Retirement { data } = &mut d.income[1].income {
            edit_retirement(data).gross = 0;
        }
        d.deductions.as_mut().unwrap().annuity_losses.clear();
        d.people.as_mut().unwrap().taxpayer.died = None;
        unsupported(&d, "Income:");
    }
    let mut a = variable_annuity();
    a.started = 20260701;
    a.period = a.started;
    a.payments = 0;
    assert!(!a.schedule_valid(Some(20260630)));
}

#[test]
fn general_rule_survivor_recovery_uses_first_annuitants_current_year_payments() {
    let mut survivor = joint_annuity(true);
    survivor.started = 20000101;
    survivor.received = 6;
    survivor.earlier_payments = 300000; // Six original $500 primary installments.
    survivor.recovered = survivor.cost - 200000;
    assert!(survivor.valid(210000));
    assert_eq!(survivor.earlier_recovery(), 155100);
    assert_eq!(survivor.excluded(), 44900); // $2,000 common cost less $1,551 already used.
    let mut primary = survivor;
    primary.earlier_payments = 0;
    if let AnnuityDuration::JointLife { survivor, .. } = &mut primary.duration {
        *survivor = false;
    }
    assert_eq!(primary.excluded() + survivor.excluded(), 200000);
    let mut d = complete(FederalStatus::Single);
    let mut entry = retirement_exception_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        2100,
        20261201,
        RetirementException::Death { died: 20260615 },
        false,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::GeneralAnnuity { data: survivor });
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 165100);
    assert_eq!(value(&r, "annuity_2_earlier_recovery"), 1551);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    let mut joint = complete(FederalStatus::Joint);
    joint.people.as_mut().unwrap().spouse.as_mut().unwrap().died =
        Some(FinalReturn { date: 20260615, bonds: None, ordinary: true, estate_medical: 0 });
    let mut primary_entry = retirement_entry(
        Owner::Spouse,
        RetirementKind::Nonqualified,
        3000,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut primary_entry.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::GeneralAnnuity { data: primary });
    }
    add(&mut joint, primary_entry);
    let mut survivor_entry = d.income[1].clone();
    survivor_entry.owner = crate::DocumentOwner::Yours;
    survivor_entry.id = 0;
    add(&mut joint, survivor_entry);
    let r = supported(&joint);
    assert_eq!(return_line(&r, "5a"), 510000);
    assert_eq!(return_line(&r, "5b"), 310000);
    joint.income.swap(1, 2);
    assert_eq!(return_line(&supported(&joint), "5b"), 310000);
    assert_eq!(restart(joint.clone()).draft, joint);
    primary.earlier_payments = 100;
    assert!(!primary.valid(300000));
}

#[test]
fn variable_survivor_cent_allocation_preserves_the_common_annual_allowance() {
    let mut a = variable_annuity();
    a.started = 20200101;
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 6;
    a.earlier = Some(VariableEarlier { payments: 6, gross: 90000 });
    a.history = VariableHistory::Prior { annual: 120001, recovered: 0, refigure: None };
    assert_eq!(a.earlier_recovery(), 60001);
    assert_eq!(a.excluded(90000), 60000);
    a.earlier.as_mut().unwrap().gross = 0;
    assert_eq!(a.earlier_recovery(), 0);
    assert_eq!(a.excluded(90000), 60000); // No automatic transfer of unused allowance.

    for (frequency, count) in [
        (AnnuityFrequency::Semiannual, 2),
        (AnnuityFrequency::Quarterly, 4),
        (AnnuityFrequency::Monthly, 12),
    ] {
        a.frequency = frequency;
        for annual in 0..=500 {
            a.history = VariableHistory::Prior { annual, recovered: 0, refigure: None };
            for earlier in 1..count {
                a.payments = count - earlier;
                a.earlier = Some(VariableEarlier { payments: earlier, gross: 100000 });
                assert_eq!(a.earlier_recovery() + a.excluded(100000), annual);
                for remaining in [0, annual / 2, annual] {
                    a.history = VariableHistory::Prior {
                        annual,
                        recovered: a.cost - remaining,
                        refigure: None,
                    };
                    assert_eq!(a.earlier_recovery() + a.excluded(100000), remaining);
                }
                a.history = VariableHistory::Prior { annual, recovered: 0, refigure: None };
            }
        }
    }
}

#[test]
fn temporary_life_annuity_reproduces_irs_table_viii_example_and_return() {
    let mut a = fixed_annuity();
    a.periodic = 20000;
    a.cost = 588000;
    a.duration = AnnuityDuration::TemporaryLife { age: 65, months: 60 };
    assert!(a.valid(240000));
    assert_eq!(a.expected(), 1176000); // IRS $200 monthly x 12 x 4.9.
    assert_eq!(a.ratio(), 500);
    assert_eq!(a.excluded(), 120000);
    let mut d = complete(FederalStatus::Single);
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        2400,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 120000);
    assert_eq!(value(&r, "annuity_2_expected_return"), 11760);
    assert_eq!(restart(d.clone()).draft, d);
    for (months, expected) in [(17, 240000), (18, 480000), (60, 1176000)] {
        a.duration = AnnuityDuration::TemporaryLife { age: 65, months };
        assert_eq!(a.expected(), expected);
    }
}

#[test]
fn temporary_life_annuity_death_deduction_requires_death_before_term_expiration() {
    let mut a = fixed_annuity();
    a.periodic = 20000;
    a.cost = 588000;
    a.received = 6;
    a.duration = AnnuityDuration::TemporaryLife { age: 65, months: 60 };
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260630, bonds: None, ordinary: true, estate_medical: 0 });
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1200,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, entry);
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: a.started,
        cost: a.cost,
        recovered: 0,
        refunds: 0,
        eligible: true,
    });
    assert_eq!(value(&supported(&d), "annuity_loss_2"), 5280);
    assert_eq!(restart(d.clone()).draft, d);
    if let FederalIncome::Retirement { data } = &mut d.income[1].income {
        let Some(RetirementTaxable::GeneralAnnuity { data: annuity }) =
            &mut edit_retirement(data).taxable
        else {
            unreachable!()
        };
        annuity.started = 20200101;
    }
    d.deductions.as_mut().unwrap().annuity_losses[0].started = 20200101;
    unsupported(&d, "Annuity loss"); // Term ended in 2025; delayed receipts do not revive it.
    a.started = 20240229;
    a.duration = AnnuityDuration::TemporaryLife { age: 65, months: 24 };
    assert!(a.ends_at_death(20260227));
    assert!(!a.ends_at_death(20260228));
    assert!(!a.ends_at_death(20260301));
}

#[test]
fn variable_temporary_life_refigures_remaining_term_and_stops_at_expiration() {
    let mut a = variable_annuity();
    a.started = 20250101;
    a.cost = 1176000;
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 12;
    a.history = VariableHistory::Prior {
        annual: 240000,
        recovered: 200000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 200000,
            remaining: VariablePayments::TemporaryLife { age: 66, months: 48 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 250256); // $400 shortfall / 3.9 remaining years.
    if let VariableHistory::Prior { refigure: Some(ref mut r), .. } = a.history {
        r.remaining = VariablePayments::TemporaryLife { age: 66, months: 6 };
    }
    a.payments = 6;
    assert!(a.valid());
    assert!(a.schedule_valid(None)); // Term ends July 1; only six periods remain.
    assert_eq!(a.annual(), 280000);
    assert_eq!(a.excluded(180000), 140000);
    assert!(a.ends_at_death(20260630));
    assert!(!a.ends_at_death(20260701));
    let mut d = complete(FederalStatus::Single);
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1800,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::VariableAnnuity { data: a });
    }
    add(&mut d, entry);
    assert_eq!(return_line(&supported(&d), "5b"), 40000);
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260630, bonds: None, ordinary: true, estate_medical: 0 });
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: a.started,
        cost: a.cost,
        recovered: a.recovered(),
        refunds: 0,
        eligible: true,
    });
    assert_eq!(value(&supported(&d), "annuity_loss_2"), 8360);
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20260701;
    unsupported(&d, "Annuity loss");
    a.payments = 7;
    assert!(!a.schedule_valid(None));
    assert!(!a.schedule_valid(Some(20261231)));
}

#[test]
fn single_life_guarantee_checks_rounding_age_and_qualified_plan_rules() {
    let mut a = fixed_annuity();
    a.periodic = 10000;
    a.cost = 2105300;
    // 17.5 years rounds to 18; guaranteed return $21,000, 15% adjustment.
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 2100000 };
    assert_eq!(a.refund_adjustment(), 315000);
    a.cost = 2000099; // Whole-dollar adjustment: $3,000.1485 -> $3,000.
    assert_eq!(a.refund_adjustment(), 300000);
    a.cost = 2000399; // $3,000.5985 -> $3,001.
    assert_eq!(a.refund_adjustment(), 300100);
    a.duration = AnnuityDuration::SingleLife { age: 57, delay: 1, guaranteed: 10000 };
    assert!(a.valid(120000)); // Explicit zero-value exception, no zero-year lookup.
    assert_eq!(a.refund_adjustment(), 0);
    a.duration = AnnuityDuration::SingleLife { age: 58, delay: 1, guaranteed: 10000 };
    assert!(!a.valid(120000)); // Missing short-term factor is not silently zero.
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 4810000 };
    assert!(!a.valid(120000));
    a.qualified_age = Some(75);
    a.duration = AnnuityDuration::SingleLife { age: 75, delay: 1, guaranteed: 600000 };
    assert!(a.qualified_method());
    assert_eq!(a.refund_adjustment(), 48000); // 8% of $6,000 guarantee.
    a.cost = 1000000;
    let mut d = complete(FederalStatus::Single);
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        1200,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        edit_retirement(data).taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, entry);
    assert_eq!(a.expected(), 1500000); // IRS Table V age 75: 12.5 years.
    assert_eq!(a.ratio(), 635); // ($10,000 - $480) / $15,000, rounded.
    assert_eq!(return_line(&supported(&d), "5b"), 43800);
    assert_eq!(restart(d.clone()).draft, d);
    if let AnnuityDuration::SingleLife { guaranteed, .. } = &mut a.duration {
        *guaranteed = 599999;
    }
    assert!(!a.qualified_method());
    assert_eq!(crate::annuity_tables::refund_percent(65, 18), Some(15));
    assert_eq!(crate::annuity_tables::refund_percent(115, 40), Some(99));
    for (age, years) in [(4, 1), (116, 1), (65, 0), (65, 41)] {
        assert_eq!(crate::annuity_tables::refund_percent(age, years), None);
    }
}

#[test]
fn domestic_abuse_account_limits_and_owners_remain_separate() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, abuse_withdrawal(10000, 8000));
    add(&mut d, abuse_withdrawal_plan("Plan B", 20000, 8000));
    let r = supported(&d);
    assert_eq!(value(&r, "retirement_2_qualified_gross"), 5000);
    assert_eq!(value(&r, "retirement_3_qualified_gross"), 5500);
    assert_eq!(value(&r, "domestic_abuse_used_taxpayer"), 10500);
    assert_eq!(value(&r, "early_distribution_tax"), 550);
    assert_eq!(estimate(&restart(d).draft), r);

    let mut joint = complete(FederalStatus::Joint);
    add(&mut joint, abuse_withdrawal(20000, 8000));
    let mut spouse = abuse_withdrawal(20000, 8000);
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut joint, spouse);
    let r = supported(&joint);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "domestic_abuse_used_taxpayer"), 8000);
    assert_eq!(value(&r, "domestic_abuse_used_spouse"), 8000);
    assert_eq!(estimate(&restart(joint).draft), r);
}

#[test]
fn domestic_abuse_same_plan_claims_share_half_balance_and_prior_usage() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, abuse_withdrawal(10000, 4000));
    add(&mut d, abuse_withdrawal(10000, 4000));
    let r = supported(&d);
    assert_eq!(value(&r, "retirement_2_qualified_gross"), 4000);
    assert_eq!(value(&r, "retirement_3_qualified_gross"), 1000);
    assert_eq!(value(&r, "domestic_abuse_used_taxpayer"), 5000);
    assert_eq!(value(&r, "early_distribution_tax"), 300);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    let FederalIncome::Retirement { data } = &mut d.income[2].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception {
            facts: RetirementException::DomesticAbuse { balance, .. },
            ..
        }) = &mut retirement_edit.ordinary_distribution
        else {
            unreachable!()
        };
        *balance = 2000000;
    }
    unsupported(&d, "Income");

    let mut prior = complete(FederalStatus::Single);
    add(&mut prior, abuse_withdrawal(20000, 8000));
    let FederalIncome::Retirement { data } = &mut prior.income[1].income else { unreachable!() };
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::Exception {
            facts: RetirementException::DomesticAbuse { prior_used, prior_plan_used, .. },
            ..
        }) = &mut retirement_edit.ordinary_distribution
        else {
            unreachable!()
        };
        *prior_used = 500000;
        *prior_plan_used = 500000;
    }
    let r = supported(&prior);
    assert_eq!(value(&r, "retirement_2_qualified_gross"), 5000);
    assert_eq!(value(&r, "domestic_abuse_used_taxpayer"), 10000);
    assert_eq!(value(&r, "early_distribution_tax"), 300);
    assert_eq!(estimate(&restart(prior).draft), r);
}

#[test]
fn inherited_section1014_casualty_gain_is_long_term_even_if_received_recently() {
    let mut d = complete(FederalStatus::Single);
    let mut event = personal_disaster("Inherited property", 100_000);
    event.properties[0].before = 200_000;
    event.properties[0].reimbursement = 400_000;
    event.properties[0].received = 400_000;
    event.properties[0].acquisition = CasualtyAcquisition::InheritedSection1014 { date: 20260701 };
    d.deductions.as_mut().unwrap().personal_casualties.push(event);
    let inherited = supported(&d);
    assert_eq!(value(&inherited, "form_4684_personal_long_net"), 3_000);
    assert_eq!(return_line(&inherited, "7a"), 300_000);
    assert_eq!(estimate(&restart(d.clone()).draft), inherited);

    d.deductions.as_mut().unwrap().personal_casualties[0].properties[0].acquisition =
        CasualtyAcquisition::Purchased { date: 20260701 };
    let purchased = supported(&d);
    assert_eq!(value(&purchased, "form_4684_personal_short_net"), 3_000);
    d.deductions.as_mut().unwrap().personal_casualties[0].properties[0].acquisition =
        CasualtyAcquisition::InheritedSection1014 { date: 20270101 };
    assert!(!d.valid_stored());
}

#[test]
fn books_and_jewelry_keep_separate_form_8283_groups_across_donees() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut books = used_goods(500_000);
    books.description = "Personally purchased books".into();
    books.kind = GiftKind::Goods { kind: GoodsKind::Books, condition: GoodsCondition::Good };
    books.basis = 1_000_000;
    books.value_method = ValueMethod::ComparableSales;
    let mut jewelry = used_goods(500_000);
    jewelry.description = "Personally purchased jewelry".into();
    jewelry.kind = GiftKind::Goods { kind: GoodsKind::Jewelry, condition: GoodsCondition::Good };
    jewelry.basis = 1_000_000;
    jewelry.value_method = ValueMethod::ComparableSales;
    ded.property_gifts.extend([books, jewelry]);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let separate_groups = supported(&d);
    assert_eq!(value(&separate_groups, "form_8283_section_a_value"), 10_000);
    assert_eq!(value(&separate_groups, "charity_property_fifty_current_deduction"), 9_700);

    let mut second_books = used_goods(10_000);
    second_books.description = "Books sent to another charity".into();
    second_books.kind = GiftKind::Goods { kind: GoodsKind::Books, condition: GoodsCondition::Good };
    second_books.charity = "Second synthetic public charity".into();
    second_books.value_method = ValueMethod::ComparableSales;
    d.deductions.as_mut().unwrap().property_gifts.push(second_books);
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));

    for gift in &mut d.deductions.as_mut().unwrap().property_gifts {
        if matches!(gift.kind, GiftKind::Goods { kind: GoodsKind::Books, .. }) {
            gift.value_method = ValueMethod::Appraisal;
            gift.appraisal = Some(2);
        }
    }
    d.deductions.as_mut().unwrap().appraisals.push(gift_appraisal(2, 20261115));
    d.next_id = 3;
    let appraised_books = supported(&d);
    assert_eq!(value(&appraised_books, "form_8283_section_a_value"), 5_000);
    assert_eq!(value(&appraised_books, "form_8283_section_b_value"), 5_100);
    assert_eq!(value(&appraised_books, "charity_property_fifty_current_deduction"), 9_800);
    assert_eq!(estimate(&restart(d.clone()).draft), appraised_books);
}

#[test]
fn current_and_prior_cash_precede_whaling_and_keep_separate_vintages() {
    // AGI $100,000: current (G) $20,000, then 2025 (G) $35,000; amended
    // (B) residual room is $50,000 - $55,000 + $10,000 = $5,000.
    let mut draft =
        with_current_direct_cash(cash_carry_with_whaling(10_000_000, 2_000_000, 35_000), 2_000_000);
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "agi"), 100_000);
    assert_eq!(value(&itemized, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&itemized, "charity_carry_2025_cash_sixty_used"), 35_000);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_deduction"), 35_000);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&itemized, "charity_whaling_deduction"), 4_500);
    assert_eq!(value(&itemized, "charity_whaling_return_year_carryforward"), 15_500);
    assert_eq!(value(&itemized, "charity_cash_sixty_return_year_carryforward"), 0);
    assert_eq!(value(&itemized, "charity_return_year_carryforward"), 15_500);
    assert_eq!(return_line(&itemized, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(return_line(&automatic, "12e"), 5_950_000);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 35_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_whaling_return_year_carryforward"), 15_500);
    assert_eq!(value(&automatic, "charity_cash_sixty_return_year_carryforward"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), automatic);

    let mut joint = complete(FederalStatus::Joint);
    joint.deductions = draft.deductions;
    let joint_standard = supported(&joint);
    // 2026-11-02: retained gifts also make this joint return an itemizer.
    assert_eq!(return_line(&joint_standard, "12f"), 0);
    assert_eq!(value(&joint_standard, "charity_carry_2025_cash_sixty_used"), 35_000);
    assert_eq!(estimate(&restart(joint).draft), joint_standard);
}

#[test]
fn inherited_section_1014_public_stock_is_long_term_without_a_year_of_ownership() {
    let mut d = complete(FederalStatus::Single);
    let mut gift = public_stock_gift();
    gift.how_acquired = AcquiredBy::Inheritance;
    gift.acquired = 20260525;
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(gift);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let r = supported(&d);
    assert_eq!(value(&r, "charity_property_1_claim"), 20_000);
    assert_eq!(value(&r, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(value(&r, "charity_capital_gain_thirty_return_year_carryforward"), 2_300);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    // Inherited property is long-term without a user-entered tax class; an acquisition after contribution still fails.
    d.deductions.as_mut().unwrap().property_gifts[0].acquired = 20260701;
    assert!(!d.valid_stored());
}

#[test]
fn retirement_unmodeled_codes_are_refused_by_name() {
    // Unit M3-P4: the interview page stores every unsupported box 7 code as
    // `RetirementTreatment::Unsupported { code }` and the calculation
    // refuses it by name, replacing the old `UnusualRetirementCodes`
    // situation and its generic "codes 5, 9, C, E, H, L and M" catch-all.
    // Box 9a and box 10 are no longer stored at all: the page's client rule
    // rejects them on save (interview.rs `document_rule`).
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(FederalRetirement {
                box7: crate::DistributionCodes::CODE_C.0,
                ordinary_distribution: Some(RetirementTreatment::Unsupported),
                ..native_retirement(
                    RetirementKind::Pension,
                    100_000,
                    (RetirementTaxable::Payer, 100_000),
                )
            }),
        }),
    );
    unsupported(&d, "Income");
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.message.contains("Form 1099-R code C")
            && i.message.contains("reportable death benefits")),
        "{r:?}"
    );
}

#[test]
fn pension_and_ira_distributions_match_hand_computation() {
    // Independent full-return fixture for the form 1099-R map's typical case
    // (matrix row 8). Single filer born 1962-06-15: turns 64 during 2026
    // (2026 - 1962 = 64), so neither the standard-deduction age-65 addition
    // nor the OBBBA senior bonus deduction applies.
    //
    // W-2: box 1 $48,000, box 2 $4,000 (wage() derives consistent boxes 3-6).
    // 1099-R #1, employer pension: box 1 $18,000, box 2a $18,000 (payer
    //   determined), box 4 $1,800, code 7 (Ordinary), IRA/SEP/SIMPLE box
    //   unchecked -> RetirementKind::Pension, RetirementTaxable::Payer.
    // 1099-R #2, traditional IRA: box 1 $6,000, box 2b checked ("taxable
    //   amount not determined"), box 4 $600, code 7 (Ordinary), IRA/SEP/SIMPLE
    //   box checked -> RetirementKind::Ira, RetirementTaxable::IraBasis (the
    //   Form 8606 worksheet). No basis: prior basis $0 and the December 31
    //   traditional-IRA balance is $0 after this complete distribution, so
    //   the pro-rata worksheet excludes $0/($0 + $6,000) and the full $6,000
    //   is taxable.
    //
    // Form 1040 by hand (i1040gi):
    //   1a wages                                   = $48,000
    //   4a IRA gross distribution                  = $6,000
    //   4b IRA taxable amount (no basis)            = $6,000
    //   5a pension gross distribution               = $18,000
    //   5b pension taxable amount (payer)           = $18,000
    //   9  total income = 48,000 + 6,000 + 18,000   = $72,000
    //   11 AGI (no adjustments)                     = $72,000
    //   12e standard deduction, single, under 65    = $16,100
    //   15 taxable income = 72,000 - 16,100         = $55,900
    //   16 regular tax: taxable income is under $100,000, so i1040gi's Tax
    //      Table applies the rate schedule to the midpoint of the $50-wide
    //      row containing $55,900 (the $55,900-$55,950 row, midpoint
    //      $55,925), using the 2026 single brackets (Rev. Proc. 2025-32):
    //      10% to $12,400, 12% to $50,400, 22% to $105,700:
    //        10% x $12,400                          = $1,240.00
    //        12% x ($50,400 - $12,400)               = $4,560.00
    //        22% x ($55,925 - $50,400)               = $1,215.50
    //        subtotal                                = $7,015.50 -> $7,016
    //   24a total tax before the farmland installment = $7,016 (no credits;
    //      no additional tax: both distributions are code 7/Ordinary and the
    //      filer is over 59 1/2)
    //   25a W-2 withholding                          = $4,000
    //   25b 1099 withholding = $1,800 + $600         = $2,400
    //   25d total federal withholding                = $6,400
    //   33 total payments (no other payments/credits) = $6,400
    //   34 overpayment = max(0, 6,400 - 7,016)        = $0
    //   37 balance due = max(0, 7,016 - 6,400)        = $616
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19620615;
    d.income = vec![wage(1, Owner::Taxpayer, 4_800_000, 400_000)];
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(FederalRetirement {
                withholding: 180_000,
                ..native_retirement(
                    RetirementKind::Pension,
                    1_800_000,
                    (RetirementTaxable::Payer, 1_800_000),
                )
            }),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(FederalRetirement {
                withholding: 60_000,
                ..native_retirement(
                    RetirementKind::Ira,
                    600_000,
                    (RetirementTaxable::IraBasis, 600_000),
                )
            }),
        }),
    );
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 0)];

    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 4_800_000);
    assert_eq!(return_line(&r, "4a"), 600_000);
    assert_eq!(return_line(&r, "4b"), 600_000);
    assert_eq!(return_line(&r, "5a"), 1_800_000);
    assert_eq!(return_line(&r, "5b"), 1_800_000);
    assert_eq!(return_line(&r, "9"), 7_200_000);
    assert_eq!(value(&r, "agi"), 72_000);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "15"), 5_590_000);
    assert_eq!(return_line(&r, "16"), 701_600);
    assert_eq!(return_line(&r, "24a"), 701_600);
    assert_eq!(return_line(&r, "25a"), 400_000);
    assert_eq!(return_line(&r, "25b"), 240_000);
    assert_eq!(return_line(&r, "25d"), 640_000);
    assert_eq!(return_line(&r, "33"), 640_000);
    assert_eq!(return_line(&r, "34"), 0);
    assert_eq!(return_line(&r, "37"), 61_600);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn early_ira_distribution_adds_the_ten_percent_tax_on_schedule_2() {
    // Boundary: 1099-R code 1, an early IRA distribution with no exception at
    // age 45 (born 1981, well under 59 1/2). The distribution stays fully on
    // 4b; Form 5329 Part I adds 10% of the taxable amount to Schedule 2 line
    // 8: 10% x $5,000 = $500.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19810615;
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(FederalRetirement {
                box7: crate::DistributionCodes::CODE_1.0,
                ordinary_distribution: Some(RetirementTreatment::Early),
                ..native_retirement(
                    RetirementKind::Ira,
                    500_000,
                    (RetirementTaxable::Payer, 500_000),
                )
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 500_000);
    assert_eq!(return_line(&r, "4b"), 500_000);
    assert_eq!(value(&r, "early_distribution_tax"), 500);
}

#[test]
fn direct_rollover_is_reported_on_5a_only() {
    // Boundary: 1099-R code G, a direct rollover of the entire $20,000
    // employer pension distribution to another employer plan (box 2a $0).
    // 5a keeps the gross distribution; 5b is 0 because none of it was kept,
    // and no additional tax applies since nothing was kept early either.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(FederalRetirement {
                ordinary_distribution: Some(RetirementTreatment::Rollover {
                    destination: RolloverDestination::EmployerPlan,
                    amount: 2_000_000,
                    early: false,
                }),
                ..native_retirement(
                    RetirementKind::Pension,
                    2_000_000,
                    (RetirementTaxable::Payer, 0),
                )
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 2_000_000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}

#[test]
fn ira_basis_pro_rata_recovers_a_share_when_balance_remains_at_year_end() {
    use super::support::assert::unit50_retirement::assert_ira_basis_case;

    for (case, input, basis, taxable_cents, complete_return) in [
        // Boundary: Form 8606/Pub 590-B Worksheet 1-1 pro-rata basis allocation.
        // Prior basis $2,000; December 31 traditional-IRA value $15,000 (after
        // this distribution); $5,000 distributed this year; no current-year
        // contribution. Total basis pool = $2,000. Total value for the ratio =
        // $15,000 (year-end) + $5,000 (this distribution) = $20,000.
        //   Nontaxable fraction = $2,000 / $20,000 x $5,000        = $500
        //   Taxable amount = $5,000 - $500                          = $4,500
        //   Remaining basis carried forward = $2,000 - $500         = $1,500
        ("pro-rata 5000", [500_000, 0, 2000, 15000], [500, 4500, 1500], 450_000, None),
        // Boundary (matrix row 41, Form 8606 Part I): pro-rata (i8606 lines
        // 6-13) when a balance remains in the traditional IRA at year end
        // (line 6 is not zero) and no current-year contribution is made.
        // Single filer, default $60,000 wage withheld $6,500 (`complete`).
        // Prior basis (line 2) $3,000; December 31 traditional/SEP/SIMPLE IRA
        // value (line 6) $27,000; this year's IRA distribution (line 7)
        // $9,000, withheld $900, ordinary (no exception).
        //
        // Line 3 = line 1 ($0 current contribution) + line 2 ($3,000) = $3,000
        // Line 10 basis fraction = line 3 / (line 6 $27,000 + line 7 $9,000)
        //   = 3,000 / 36,000                                       = 0.08333...
        // Line 13 nontaxable = fraction x $9,000                    = $750
        // Line 15c taxable = $9,000 - $750                           = $8,250
        // Line 14 basis carried forward = $3,000 - $750               = $2,250
        //
        // Form 1040 by hand (i1040gi):
        //   1a wages                                     = $60,000
        //   4a IRA gross distribution                     = $9,000
        //   4b IRA taxable amount                          = $8,250
        //   9  total income = 60,000 + 8,250               = $68,250
        //   11 AGI (no adjustments)                          = $68,250
        //   12e standard deduction, single, under 65          = $16,100
        //   15 taxable income = 68,250 - 16,100                = $52,150
        //   16 regular tax: taxable income is under $100,000, so i1040gi's Tax
        //      Table applies the rate schedule to the midpoint of the $50-wide
        //      row containing $52,150 (the $52,150-$52,200 row, midpoint
        //      $52,175), using the 2026 single brackets (Rev. Proc. 2025-32):
        //      10% to $12,400, 12% to $50,400, 22% to $105,700:
        //        10% x $12,400                             = $1,240.00
        //        12% x ($50,400 - $12,400)                   = $4,560.00
        //        22% x ($52,175 - $50,400)                    = $390.50
        //        subtotal                                      = $6,190.50 -> $6,191
        //   24a total tax (no credits, no additional tax: an ordinary
        //      distribution carries no exception tax)            = $6,191
        //   25a W-2 withholding                                    = $6,500
        //   25b 1099 withholding                                    = $900
        //   25d total federal withholding                            = $7,400
        //   33 total payments                                          = $7,400
        //   34 overpayment = max(0, 7,400 - 6,191)                      = $1,209
        //   37 balance due = max(0, 6,191 - 7,400)                       = $0
        (
            "balance remaining",
            [900_000, 90_000, 3000, 27000],
            [750, 8250, 2250],
            825_000,
            Some([
                68_250, 5_215_000, 619_100, 619_100, 650_000, 90_000, 740_000, 740_000, 120_900, 0,
            ]),
        ),
        // Boundary (matrix row 41, Form 8606 line 6): the December 31
        // traditional-IRA balance is exactly zero — the entire remaining
        // account was distributed this year, so the pro-rata fraction is
        // basis over the distribution alone (no year-end balance dilutes it)
        // and the whole remaining basis is recovered in one step. Single
        // filer, default $60,000 wage withheld $6,500 (`complete`). Prior
        // basis (line 2) $4,000; December 31 value (line 6) $0; the entire
        // $16,000 IRA is distributed this year (line 7), ordinary (no
        // exception), $1,000 withheld.
        //
        // Line 3 = line 1 ($0 current contribution) + line 2 ($4,000) = $4,000
        // Line 10 basis fraction = line 3 / (line 6 $0 + line 7 $16,000)
        //   = 4,000 / 16,000                                        = 0.25
        // Line 13 nontaxable = 0.25 x $16,000                         = $4,000
        // Line 15c taxable = $16,000 - $4,000                          = $12,000
        // Line 14 basis carried forward = $4,000 - $4,000               = $0
        //
        // Form 1040 by hand (i1040gi):
        //   1a wages                                     = $60,000
        //   4a IRA gross distribution                      = $16,000
        //   4b IRA taxable amount                           = $12,000
        //   9  total income = 60,000 + 12,000                = $72,000
        //   11 AGI (no adjustments)                            = $72,000
        //   12e standard deduction, single, under 65             = $16,100
        //   15 taxable income = 72,000 - 16,100                    = $55,900
        //   16 regular tax: Tax Table midpoint of the $55,900-$55,950 row is
        //      $55,925, 2026 single brackets: 10% to $12,400, 12% to $50,400,
        //      22% to $105,700:
        //        10% x $12,400                                        = $1,240.00
        //        12% x ($50,400 - $12,400)                              = $4,560.00
        //        22% x ($55,925 - $50,400)                               = $1,215.50
        //        subtotal                                                  = $7,015.50 -> $7,016
        //   24a total tax                                                    = $7,016
        //   25a W-2 withholding                                                = $6,500
        //   25b 1099 withholding                                                 = $1,000
        //   25d total federal withholding                                          = $7,500
        //   33 total payments                                                        = $7,500
        //   34 overpayment = max(0, 7,500 - 7,016)                                     = $484
        //   37 balance due                                                                = $0
        (
            "zero line 6",
            [1_600_000, 100_000, 4000, 0],
            [4000, 12000, 0],
            1_200_000,
            Some([
                72_000, 5_590_000, 701_600, 701_600, 650_000, 100_000, 750_000, 750_000, 48_400, 0,
            ]),
        ),
    ] {
        let r = assert_ira_basis_case(case, input, basis, taxable_cents, complete_return);
        if case == "balance remaining" {
            assert_eq!(value(&r, "taxpayer_ira_form_line2"), 3000);
            assert_eq!(value(&r, "taxpayer_ira_form_line6"), 27000);
            assert_eq!(value(&r, "taxpayer_ira_form_line12"), 750);
            assert_eq!(value(&r, "taxpayer_ira_form_line15c"), 8250);
            assert_eq!(
                r.lines.iter().find(|row| row.id == "taxpayer_ira_form_line10").unwrap().amount,
                833
            );
        }
    }
}

#[test]
fn backdoor_roth_contribution_and_conversion_share_pro_rata_basis() {
    // Fixture (matrix row 41, no prior hand fixture): a "backdoor Roth"
    // year — a nondeductible traditional IRA contribution converted to a
    // Roth IRA in the same year, with an existing pre-tax balance in the
    // traditional IRA (Form 8606 Parts I-II). Single filer, default
    // $60,000 wage withheld $6,500 (`complete`). Taxpayer elects the full
    // $7,000 traditional contribution as nondeductible (Form 8606 line
    // 1). The traditional IRA held $50,000 of pre-tax money from earlier
    // years (no prior basis, i.e. line 2 = $0); the taxpayer converts the
    // entire IRA ($50,000 pre-tax + $7,000 new contribution = $57,000) to
    // a Roth IRA, leaving December 31 value (line 6) at $0.
    //
    // Form 8606 by hand (i8606):
    //   Line 1 nondeductible contribution                  = $7,000
    //   Line 2 prior basis                                  = $0
    //   Line 3 = line 1 + line 2                             = $7,000
    //   Line 6 value of traditional IRAs on 12/31             = $0
    //   Line 8 amount converted                                = $57,000
    //   Line 16 amount converted (nothing kept, all converted)  = $57,000
    //   Line 10 basis fraction = line 3 / (line 6 + line 16)
    //     = 7,000 / (0 + 57,000)                                = 0.122807...
    //   Line 11 nontaxable part of the conversion = fraction x
    //     $57,000 = $7,000 exactly (the entire account, old pre-tax money
    //     and the new nondeductible contribution alike, was converted, so
    //     the pro-rata rule recovers exactly the basis and nothing more)
    //   Line 17 = line 16 - line 11 = $57,000 - $7,000          = $50,000 taxable
    //   Line 14 basis carried forward = $7,000 - $7,000          = $0
    //
    // Form 1040 by hand (i1040gi):
    //   1a wages                                        = $60,000
    //   4a IRA gross distribution (conversion)            = $57,000
    //   4b IRA taxable amount                              = $50,000
    //   9  total income = 60,000 + 50,000                  = $110,000
    //   11 AGI (a nondeductible contribution creates no deduction)
    //                                                        = $110,000
    //   12e standard deduction, single, under 65              = $16,100
    //   15 taxable income = 110,000 - 16,100                   = $93,900
    //   16 regular tax: under $100,000, so the Tax Table midpoint of the
    //      $93,900-$93,950 row is $93,925, using the 2026 single brackets
    //      (Rev. Proc. 2025-32): 10% to $12,400, 12% to $50,400, 22% to
    //      $105,700:
    //        10% x $12,400                                  = $1,240.00
    //        12% x ($50,400 - $12,400)                        = $4,560.00
    //        22% x ($93,925 - $50,400)                         = $9,575.50
    //        subtotal                                           = $15,375.50 -> $15,376
    //   24a total tax (the conversion is not an early distribution: the
    //      entire distribution was converted, nothing was kept)   = $15,376
    //   25a W-2 withholding                                          = $6,500
    //   25b 1099 withholding (none on the conversion)                 = $0
    //   33 total payments                                               = $6,500
    //   34 overpayment                                                   = $0
    //   37 balance due = 15,376 - 6,500                                   = $8,876
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: retirement_source(FederalRetirement {
                ordinary_distribution: Some(RetirementTreatment::Conversion {
                    amount: 5_700_000,
                    early: false,
                }),
                ..native_retirement(
                    RetirementKind::Ira,
                    5_700_000,
                    (RetirementTaxable::IraBasis, 5_700_000),
                )
            }),
        }),
    );
    let a = d.adjustments.as_mut().unwrap();
    a.ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 0)];
    let mut iras = ira_facts(700_000, 0, Answer::No);
    iras.taxpayer.nondeductible_election = 700_000;
    a.iras = iras;
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_current_nondeductible_basis"), 7_000);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 7_000);
    assert_eq!(value(&r, "taxpayer_ira_basis_taxable"), 50_000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(return_line(&r, "4a"), 5_700_000);
    assert_eq!(return_line(&r, "4b"), 5_000_000);
    assert_eq!(value(&r, "agi"), 110_000);
    assert_eq!(return_line(&r, "15"), 9_390_000);
    assert_eq!(return_line(&r, "16"), 1_537_600);
    assert_eq!(return_line(&r, "24a"), 1_537_600);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(return_line(&r, "33"), 650_000);
    assert_eq!(return_line(&r, "34"), 0);
    assert_eq!(return_line(&r, "37"), 887_600);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn roth_distribution_uses_regular_then_conversion_basis_before_earnings_with_recapture() {
    // Fixture (matrix row 41, no prior hand fixture): Form 8606 Part III
    // Roth IRA distribution ordering with basis. Single filer, default
    // $60,000 wage withheld $6,500 (`complete`), taxpayer under 59 1/2
    // (birth 1980-06-15, default `person()`). Opening Roth history:
    // $2,000 unrecovered regular-contribution basis; one open conversion
    // from 2023 with $5,000 taxable and $1,000 nontaxable unrecovered
    // (Pub 590-B recapture worksheet; still within its own five-tax-year
    // window, which runs through 2027). This year: one nonqualified
    // $10,000 Roth withdrawal (no age or disability exception), $123
    // withheld (the test helper's fixed withholding).
    //
    // Ordinary Roth ordering (Pub 590-B chapter 2, Form 8606 Part III):
    // recover regular contributions first, then conversions oldest-first
    // (each conversion's taxable part before its nontaxable part), then
    // earnings.
    //   Regular basis recovered (fully, $2,000 <= $10,000)   = $2,000
    //   Remaining to allocate = $10,000 - $2,000               = $8,000
    //   2023 conversion, taxable part recovered (fully,
    //     $5,000 <= $8,000)                                     = $5,000
    //   Remaining = $8,000 - $5,000                               = $3,000
    //   2023 conversion, nontaxable part recovered (fully,
    //     $1,000 <= $3,000)                                        = $1,000
    //   Remaining = $3,000 - $1,000, all earnings, fully taxable
    //     (the distribution is nonqualified)                        = $2,000
    //   Line 23 (Part III) taxable amount (earnings)                 = $2,000
    //   The 2023 conversion is within its five-tax-year window
    //   (2023-2027), so the $5,000 taxable part recovered above is also
    //   subject to the 10% additional tax as previously-taxed conversion
    //   principal withdrawn early, without being taxed as income again
    //   (i5329 Part I).
    //   Additional-tax base = $2,000 earnings + $5,000 recapture       = $7,000
    //   10% additional tax = 10% x $7,000                                = $700
    //
    // Form 1040 by hand (i1040gi):
    //   1a wages                                        = $60,000
    //   4a Roth IRA gross distribution                    = $10,000
    //   4b Roth IRA taxable amount (earnings only)         = $2,000
    //   9  total income = 60,000 + 2,000                    = $62,000
    //   11 AGI (no adjustments)                               = $62,000
    //   12e standard deduction, single, under 65                = $16,100
    //   15 taxable income = 62,000 - 16,100                       = $45,900
    //   16 regular tax: under $100,000, Tax Table midpoint of the
    //      $45,900-$45,950 row is $45,925, 2026 single brackets: 10% to
    //      $12,400, 12% to $50,400:
    //        10% x $12,400                                          = $1,240.00
    //        12% x ($45,925 - $12,400)                                = $4,023.00
    //        subtotal                                                  = $5,263.00
    //   23 Schedule 2 additional tax (10% early distribution)           = $700
    //   24a total tax = 5,263 + 700                                       = $5,963
    //   25a W-2 withholding                                                = $6,500
    //   25b 1099 withholding                                                = $123
    //   25d total federal withholding = 6,500 + 123                          = $6,623
    //   33 total payments                                                      = $6,623
    //   34 overpayment = 6,623 - 5,963                                          = $660
    //   37 balance due                                                            = $0
    let mut d = complete(FederalStatus::Single);
    let mut h = roth_history(Owner::Taxpayer, 2010, 2000);
    h.ordering.as_mut().unwrap().conversions =
        vec![RothConversion { year: 2023, taxable: 500_000, nontaxable: 100_000 }];
    d.adjustments.as_mut().unwrap().roth_history = vec![h];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 10000, 20260701));
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 200_000);
    assert_eq!(value(&r, "roth_2_regular_recovered"), 2_000);
    assert_eq!(value(&r, "roth_2_conversion_recapture"), 5_000);
    assert_eq!(value(&r, "roth_2_taxable"), 2_000);
    assert_eq!(value(&r, "early_distribution_tax"), 700);
    assert_eq!(value(&r, "agi"), 62_000);
    assert_eq!(return_line(&r, "15"), 4_590_000);
    assert_eq!(return_line(&r, "16"), 526_300);
    assert_eq!(return_line(&r, "24a"), 596_300);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(return_line(&r, "25b"), 12_300);
    assert_eq!(return_line(&r, "25d"), 662_300);
    assert_eq!(return_line(&r, "33"), 662_300);
    assert_eq!(return_line(&r, "34"), 66_000);
    assert_eq!(return_line(&r, "37"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn form_5329_parts_iii_and_iv_tax_an_uncorrected_ira_excess_by_hand() {
    // Single, born 1980 (limit $7,500), $60,000 wages, not covered at work.
    // Contributions: $9,000 traditional and $1,000 Roth, none corrected.
    // Part III: line 15 = 9,000 - 7,500 = 1,500; lines 9-14 zero (no 2025
    // excess); line 16 = 1,500; traditional IRAs worth $9,200 at year end;
    // line 17 = 6% x min(1,500, 9,200) = 90.
    // Part IV: the Roth limit is the $7,500 limit less $9,000 traditional,
    // so zero (MAGI $52,500 is under the $153,000 phase-out); line 23 =
    // 1,000; line 24 = 1,000; Roth IRAs worth $1,010; line 25 = 6% x
    // min(1,000, 1,010) = 60. Both reach Schedule 2 line 18: 150 in all.
    // The deduction stops at the $7,500 limit, so AGI = 60,000 - 7,500 =
    // 52,500, the same as a return that contributed exactly the limit.
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    let baseline = supported(&d);
    assert_eq!(value(&baseline, "agi"), 52500);
    let mut f = ira_facts(900_000, 100_000, Answer::No);
    f.taxpayer.excess = Some(ira_excess(9_200, 1_010, 0, 0));
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "agi"), 52500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line9_prior_traditional_excess"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line10_unused_traditional_room"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line14_prior_traditional_remaining"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line15_traditional_excess"), 1500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line16_traditional_total_excess"), 1500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 90);
    assert_eq!(value(&r, "ira_excess_taxpayer_line19_unused_roth_room"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line23_roth_excess"), 1000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line24_roth_total_excess"), 1000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line25_roth_excess_tax"), 60);
    // Section 4973 excise is not a chapter 1 tax.
    assert_eq!(value(&r, "chapter_one_net_tax"), value(&baseline, "chapter_one_net_tax"));
    assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax") + 150);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
    // Without the year-end values the excess is refused, never taxed at zero.
    d.adjustments.as_mut().unwrap().iras.taxpayer.excess = None;
    unsupported(&d, "IRA contributions");
}

#[test]
fn form_5329_line_17_is_capped_by_the_year_end_traditional_value() {
    // Same $1,500 traditional excess (line 16) as the fixture; the tax is 6%
    // of the smaller of line 16 or the December 31 value: $400 -> 24, and a
    // value of exactly $1,500 -> 90.
    let mut d = complete(FederalStatus::Single);
    let mut f = ira_facts(900_000, 0, Answer::No);
    f.taxpayer.excess = Some(ira_excess(400, 0, 0, 0));
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_excess_taxpayer_line16_traditional_total_excess"), 1500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 24);
    d.adjustments.as_mut().unwrap().iras.taxpayer.excess = Some(ira_excess(1_500, 0, 0, 0));
    assert_eq!(value(&supported(&d), "ira_excess_taxpayer_line17_traditional_excess_tax"), 90);
}

#[test]
fn form_5329_excess_follows_the_age_50_contribution_limit() {
    // IRC 219(b)(5)(B), day-exact (Pub. 501: an age is attained the day before
    // the birthday). Born December 31, 1976 or January 1, 1977 (50th birthday
    // 2026-12-31 or 2027-01-01): age 50 by the end of 2026, limit $8,600, so
    // an $8,600 contribution has no excess and no Form 5329 lines. Born
    // January 2, 1977 the limit is $7,500: line 15 = 1,100, line 17 = 6% x
    // 1,100 = 66 with $20,000 in the accounts.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19761231;
    let mut f = ira_facts(860_000, 0, Answer::No);
    f.taxpayer.excess = Some(ira_excess(20_000, 0, 0, 0));
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 8600);
    assert_eq!(value(&r, "ira_excess_taxpayer_line15_traditional_excess"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 0);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19770101;
    assert_eq!(value(&supported(&d), "ira_deduction"), 8600);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19770102;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line15_traditional_excess"), 1100);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 66);
}

#[test]
fn form_5329_roth_excess_follows_the_single_magi_phase_out_end() {
    // Single, Roth-only $7,000. At MAGI $168,000 (the end of the $153,000 +
    // $15,000 phase-out) the Roth limit is zero: line 23 = 7,000, line 25 =
    // 6% x min(7,000, 7,100) = 420. At $167,999 the phased limit is
    // $7,500 x 1 / 15,000, rounded up to $10 but never under $200: line 23
    // = 6,800, line 25 = 408. Neither changes the income tax.
    for (wages, excess, tax) in [(168_000, 7000, 420), (167_999, 6800, 408)] {
        let mut d = complete(FederalStatus::Single);
        d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
        let baseline = supported(&d);
        let mut f = ira_facts(0, 700_000, Answer::Yes);
        f.taxpayer.excess = Some(ira_excess(0, 7_100, 0, 0));
        d.adjustments.as_mut().unwrap().iras = f;
        let r = supported(&d);
        assert_eq!(value(&r, "roth_contribution_magi"), wages);
        assert_eq!(value(&r, "ira_excess_taxpayer_line23_roth_excess"), excess);
        assert_eq!(value(&r, "ira_excess_taxpayer_line25_roth_excess_tax"), tax);
        assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
        assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax") + tax);
    }
}

#[test]
fn form_5329_carried_traditional_excess_is_reduced_by_2026_withdrawals() {
    // 2025 Form 5329 line 16 carried $2,000 (line 9). The 2026 $7,500
    // contribution uses the whole limit, so line 10 = 0. A $500 taxable IRA
    // distribution in 2026 is line 11; line 13 = 500; line 14 = 1,500;
    // line 15 = 0; line 16 = 1,500; line 17 = 6% x min(1,500, 30,000) = 90.
    // AGI = 60,000 + 500 - 7,500 = 53,000.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 500, RetirementTreatment::Ordinary),
    );
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    let baseline = supported(&d);
    assert_eq!(value(&baseline, "agi"), 53000);
    let mut f = ira_facts(750_000, 0, Answer::No);
    f.taxpayer.excess = Some(ira_excess(30_000, 0, 2_000, 0));
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 53000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line9_prior_traditional_excess"), 2000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line10_unused_traditional_room"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line11_traditional_distributions"), 500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line14_prior_traditional_remaining"), 1500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line15_traditional_excess"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 90);
    assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax") + 90);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
    // Contributing only $5,000 leaves $2,500 of room (line 10): it absorbs the carried excess
    // not covered by the $500 withdrawal, $1,500, as a 2026 contribution on the IRA Deduction
    // Worksheet (B11 R3): deduction 5,000 + 1,500; line 13 = 2,500 + 500, line 14 = 0, no tax.
    let f = &mut d.adjustments.as_mut().unwrap().iras;
    f.taxpayer.traditional = 500_000;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 6500);
    assert_eq!(value(&r, "agi"), 54000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line10_unused_traditional_room"), 2500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line17_traditional_excess_tax"), 0);
}

#[test]
fn form_5329_carried_roth_excess_counts_a_full_withdrawal_at_least_at_the_carried_amount() {
    // The instructions' line 20 example: $1,000 carried on line 18, the
    // entire Roth balance of $800 withdrawn in 2026 (regular-contribution
    // basis, so not taxable). Wages of $170,000 put MAGI past the $168,000
    // phase-out end, so line 19 (unused 2026 Roth room) is zero and cannot
    // absorb the carry. Line 20 is 1,000, not 800; line 22 = 0; no tax.
    // With $50 still in the account the withdrawal is 800: line 22 = 200;
    // line 25 = 6% x min(200, 50) = 3.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 17_000_000, 0)];
    d.adjustments.as_mut().unwrap().roth_history = vec![roth_history(Owner::Taxpayer, 2024, 1000)];
    add(&mut d, roth_withdrawal(Owner::Taxpayer, 800, 20260907));
    let baseline = supported(&d);
    let mut f = ira_facts(0, 0, Answer::No);
    f.taxpayer.excess = Some(ira_excess(0, 0, 0, 1_000));
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    assert_eq!(value(&r, "ira_excess_taxpayer_line18_prior_roth_excess"), 1000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line19_unused_roth_room"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line20_roth_distributions"), 1000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line22_prior_roth_remaining"), 0);
    assert_eq!(value(&r, "ira_excess_taxpayer_line25_roth_excess_tax"), 0);
    assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax"));
    d.adjustments.as_mut().unwrap().iras.taxpayer.excess = Some(ira_excess(0, 50, 0, 1_000));
    let r = supported(&d);
    assert_eq!(value(&r, "ira_excess_taxpayer_line20_roth_distributions"), 800);
    assert_eq!(value(&r, "ira_excess_taxpayer_line22_prior_roth_remaining"), 200);
    assert_eq!(value(&r, "ira_excess_taxpayer_line25_roth_excess_tax"), 3);
    assert_eq!(value(&r, "total_tax"), value(&baseline, "total_tax") + 3);
}

// Retained IRA excess on Forms 8606 and 8880 (unit T2-fx). Pub 590-A
// (2025) Worksheet 1-2 line 6 enters contributions "but don't enter more
// than $7,000 ($8,000 if you are age 50 or older)" and line 8 puts line 5
// or 6, whichever is smaller, less the deduction on Form 8606 line 1; the
// Form 8606 instructions allow nondeductible contributions only "up to
// your contribution limit". The 2026 limit is $7,500 (born after 1976).
// An excess left in the IRA is Form 5329 line 15, not basis. Form 8880
// line 1, by contrast, is "Traditional and Roth IRA contributions ... Do
// not include rollover contributions" with no excess exclusion.

#[test]
fn form_8606_line_1_leaves_out_a_retained_traditional_excess() {
    use super::support::assert::unit50_retirement::assert_retained_ira_excess_case;

    for (input, expected, agi) in [
        // Single, born 1980, wages $60,000, not covered at work; $9,000
        // traditional, none corrected, $9,200 in the IRAs at year end.
        //   Worksheet 1-2: line 5 $60,000; line 6 min($9,000, $7,500) = $7,500;
        //   line 7 deduction $7,500; line 8 = min($60,000, $7,500) - $7,500 = 0.
        //   Form 5329 line 15 = $9,000 - $7,500 = $1,500. AGI $52,500.
        ((None, 900_000, 0, 9200, Answer::No, false), [7500, 0, 1500], Some(52_500)),
        // Compensation-limited: wages $5,000, $6,000 traditional.
        //   line 6 min($6,000, $7,500) = $6,000; deduction min(line 5 $5,000,
        //   line 6) = $5,000; line 8 = min($5,000, $6,000) - $5,000 = 0;
        //   line 15 = $6,000 - $5,000 = $1,000.
        // (AGI $0 opens the Saver's Credit question; its record is empty.)
        ((Some(500_000), 600_000, 0, 6100, Answer::No, true), [5000, 0, 1000], None),
        // Same $9,000 traditional, all designated nondeductible:
        //   deduction 0; line 1 = min($60,000, $7,500) - 0 = $7,500 (not
        //   $9,000); line 15 = $1,500. AGI $60,000.
        ((None, 900_000, 900_000, 9200, Answer::No, false), [0, 7500, 1500], Some(60_000)),
        // Single, covered at work, wages $86,000 (inside $81,000-$91,000);
        // $9,000 traditional.
        //   Worksheet 1-2 line 4: $7,500 x ($91,000 - $86,000) / $10,000 =
        //   $3,750 (already a multiple of $10); deduction $3,750.
        //   line 8 = min($86,000, $7,500) - $3,750 = $3,750 (the old
        //   $9,000 - $3,750 = $5,250 counted the excess as basis).
        //   line 15 = $1,500. AGI $86,000 - $3,750 = $82,250.
        ((Some(8_600_000), 900_000, 0, 9200, Answer::Yes, false), [3750, 3750, 1500], Some(82_250)),
    ] {
        assert_retained_ira_excess_case(input, expected, agi);
    }
}

#[test]
fn form_8880_line_1_keeps_a_retained_roth_excess() {
    // Single, wages $1,500 (the only compensation) and $33,500 of interest:
    // AGI $35,000, inside the single 10% band ($26,250-$40,250). $2,000
    // Roth contributed; the Roth limit is min($7,500, $1,500) = $1,500, so
    // Form 5329 line 23 = $500 and line 25 = 6% x min($500, $2,050) = $30.
    // Form 8880 line 1 is the $2,000 contributed (the retained excess is
    // not excluded); line 6 min($2,000, $2,000) = $2,000; credit $2,000 x
    // 10% = $200. Taxable income $35,000 - $16,100 = $18,900 owes far more
    // than $200, so the tax limit does not bind.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 150_000, 0);
    add(&mut d, income(savings_income(33_500, 0)));
    let mut f = ira_facts(0, 200_000, Answer::No);
    f.taxpayer.excess = Some(ira_excess(0, 2_050, 0, 0));
    d.adjustments.as_mut().unwrap().iras = f;
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 35_000);
    assert_eq!(value(&r, "ira_excess_taxpayer_line23_roth_excess"), 500);
    assert_eq!(value(&r, "ira_excess_taxpayer_line25_roth_excess_tax"), 30);
    assert_eq!(value(&r, "saver_credit"), 200);
}

#[test]
fn amt_related_sep_uses_actual_income_without_changing_deposit_or_se_tax() {
    // i6251 (2025), printed p.9 Related Adjustments; derived fixture.
    // Existing retained equipment has $655 regular/$874 AMT depreciation.
    // Profit is $9,345/$9,126; actual half-SE remains $660. The 25% SEP
    // reduced rate is 20%: limits $1,737/$1,693.20, so a valid actual
    // $1,700 deposit deducts $1,700 regular/$1,693 AMT. Difference +$7.
    let mut d = business_with_bonus(10000, separate_amt_equipment());
    d.income.retain(|entry| matches!(entry.income, FederalIncome::Business { .. }));
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { contribution: 170000, rate: 25.0 };
    let before = supported(&d);
    assert_eq!(value(&before, "self_employment_tax"), 1320);
    assert_eq!(value(&before, "schedule_1_half_self_employment_tax"), 660);
    assert_eq!(value(&before, "schedule_1_self_employed_retirement"), 1700);
    assert_eq!(value(&before, "amt_sep_deduction"), 1693);
    assert_eq!(value(&before, "amt_related_adjustments"), 7);
    assert_eq!(printed_line(&before, "amt_related_adjustments"), (TaxForm::Form6251, "3".into()));
    assert_eq!(
        before
            .lines
            .iter()
            .filter(|row| row
                .form_line
                .as_ref()
                .is_some_and(|line| line.form == TaxForm::Form6251 && line.line == "3"))
            .count(),
        1
    );
    assert_eq!(restart(d.clone()).draft, d);
    if let FederalIncome::Business { data } = &mut d.income[0].income {
        data.retirement = BusinessRetirement::None;
    }
    let without = supported(&d);
    assert_eq!(value(&without, "self_employment_tax"), 1320);
    assert_eq!(value(&without, "schedule_1_half_self_employment_tax"), 660);
    assert_eq!(value(&without, "amt_related_adjustments"), 0);
}

#[test]
fn amt_related_health_and_ira_reuse_earned_limits_without_refiguring_phaseout() {
    // Same $219 depreciation difference. Health limit consumes actual
    // profit less unchanged $660 half-SE: $8,685/$8,466, difference +$219.
    let mut d = complete(FederalStatus::Single);
    let mut business = health_business(10000, 20000, false);
    if let FederalIncome::Business { data } = &mut business {
        data.assets.push(separate_amt_equipment());
    }
    add(&mut d, income(business));
    let health = supported(&d);
    assert_eq!(value(&health, "schedule_1_self_employed_health_insurance"), 8685);
    assert_eq!(value(&health, "amt_self_employed_health_deduction"), 8466);
    assert_eq!(value(&health, "amt_related_adjustments"), 219);
    assert_eq!(value(&health, "self_employment_tax"), 1320);
    // A lower source has regular profit $7,345, half-SE $519, IRA
    // compensation $6,826. $6,800 is a permitted actual contribution.
    // AMT profit $7,126 gives deduction $6,607; line3 difference +$193.
    let mut d = business_with_bonus(8000, separate_amt_equipment());
    d.income.retain(|entry| matches!(entry.income, FederalIncome::Business { .. }));
    d.adjustments.as_mut().unwrap().iras = ira_facts(680000, 0, Answer::No);
    let ira = supported(&d);
    assert_eq!(value(&ira, "schedule_1_half_self_employment_tax"), 519);
    assert_eq!(value(&ira, "ira_deduction"), 6800);
    assert_eq!(value(&ira, "amt_ira_deduction"), 6607);
    assert_eq!(value(&ira, "amt_related_adjustments"), 193);
    assert_eq!(value(&ira, "taxpayer_nondeductible_ira"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn amt_related_unused_health_premiums_reduce_medical_adjustment_once() {
    // i6251 Related Adjustments + IRC162(l)(3), derived fixture. Wages60,000;
    // business9,345 regular/9,126 AMT; unchanged halfSE660. Health deduction
    // 8,685/8,466 leaves11,315/11,534 medical premiums. Add20,000 other medical:
    // regular AGI60,000 and floor4,500 give26,815/27,034 medical deductions.
    // Line3 is +219 health minus219 medical =0, with only -219 depreciation
    // on line2l; AMTI60,000-26,815-219=32,966. No duplicate premium benefit.
    let mut d = complete(FederalStatus::Single);
    let mut business = health_business(10_000, 20_000, false);
    if let FederalIncome::Business { data } = &mut business {
        data.assets.push(separate_amt_equipment());
    }
    add(&mut d, income(business));
    itemized(d.deductions.as_mut().unwrap()).medical = 2_000_000;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "medical_expenses"), 31_315);
    assert_eq!(value(&r, "medical_deduction"), 26_815);
    assert_eq!(value(&r, "amt_self_employed_health_deduction"), 8_466);
    assert_eq!(value(&r, "amt_related_adjustments"), 0);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 32_966);
    assert_eq!(value(&r, "self_employment_tax"), 1_320);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn amt_related_missing_partner_character_preserves_independent_owner_benefits() {
    // Form 6251 Related Adjustments: this pure $20,000 box 1 /14A source
    // supplies AMT earnings of $5,000. Keep its actual $1,413 half-SE:
    // taxpayer health is $6,000/$3,587; spouse health stays $8,685/$8,466.
    // Total AMT health is $12,053. Taxpayer wages still support the $7,500 IRA.
    // https://www.irs.gov/instructions/i6251
    let mut d = complete(FederalStatus::Joint);
    let mut partner = partner_health_source(6_000, false);
    partner.alternative.as_mut().unwrap().ordinary = 500_000;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let mut business = income(health_business(10_000, 20_000, false));
    business.owner = crate::DocumentOwner::Spouses;
    if let FederalIncome::Business { data } = &mut business.income {
        data.assets.push(separate_amt_equipment());
    }
    add(&mut d, business);
    let mut iras = ira_facts(750_000, 0, Answer::No);
    iras.spouse = IraSpouse::Joint { facts: ira_facts(0, 0, Answer::No).taxpayer };
    d.adjustments.as_mut().unwrap().iras = iras;
    let before = d.clone();
    let r = supported(&d);
    assert!(!noted(&r, "NOTE_AMT_EARNED_BENEFIT_SOURCE"));
    assert!(!noted(&r, "NOTE_AMT_IRA_COMPENSATION_SOURCE"));
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 14_685);
    assert_eq!(value(&r, "amt_self_employed_health_deduction"), 12_053);
    assert_eq!(value(&r, "ira_deduction"), 7_500);
    assert_eq!(value(&r, "amt_ira_deduction"), 7_500);
    assert_eq!(value(&r, "self_employment_tax"), 4_146);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 2_073);
    assert_eq!(value(&r, "amt_related_adjustments"), 2_632);
    assert_eq!(d, before);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn amt_related_paid_partner_health_omission_keeps_guaranteed_income_and_se() {
    // The $6,000 funded-premium subset is part of actual service guarantees.
    // Regular earnings $26,000 produce actual SE tax $3,674/half-SE $1,837.
    // The pure AMT ordinary source of $5,000 plus the unchanged $6,000
    // guarantees gives $11,000 earnings. Its $9,163 health limit still permits
    // all $6,000 premiums, so the related benefit adjustment is zero.
    // Form 6251 Related Adjustments; https://www.irs.gov/instructions/i6251
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_paid_health_source(6_000, false);
    partner.alternative.as_mut().unwrap().ordinary = 500_000;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let before = d.clone();
    let r = supported(&d);
    assert!(!noted(&r, "NOTE_AMT_EARNED_BENEFIT_SOURCE"));
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 6_000);
    assert_eq!(value(&r, "amt_self_employed_health_deduction"), 6_000);
    assert_eq!(value(&r, "self_employment_tax"), 3_674);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 1_837);
    assert_eq!(value(&r, "agi"), 78_163);
    assert_eq!(value(&r, "amt_related_adjustments"), 0);
    assert_eq!(d, before);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn amt_related_missing_partner_compensation_has_named_ira_only_limitation() {
    // Only this pure partnership source supports the permitted $7,500 IRA.
    // Form 6251 Related Adjustments retains actual half-SE $1,413 and uses
    // AMT earnings $5,000, leaving $3,587 AMT compensation/deduction. The
    // related difference is $3,913; actual SE and the accepted deposit stay put.
    // https://www.irs.gov/instructions/i6251
    let mut d = complete(FederalStatus::Single);
    d.income.clear();
    let mut partner = partner_health_source(0, false);
    partner.alternative.as_mut().unwrap().ordinary = 500_000;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    let before = d.clone();
    let r = supported(&d);
    assert!(!noted(&r, "NOTE_AMT_IRA_COMPENSATION_SOURCE"));
    assert_eq!(value(&r, "ira_deduction"), 7_500);
    assert_eq!(value(&r, "amt_ira_deduction"), 3_587);
    assert_eq!(value(&r, "self_employment_tax"), 2_826);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 1_413);
    assert_eq!(value(&r, "amt_related_adjustments"), 3_913);
    assert_eq!(value(&r, "agi"), 11_087);
    assert_eq!(d, before);
    assert_eq!(estimate(&restart(d).draft), r);
}

// Derived IRA owner boundary, not a published full-return example. The same
// Pub. 590-A Worksheet 1-2 $10-up rule applies around 84120.00; only
// the actual selected health deduction changes the MAGI input precision.
// Pub. 974 (2025), pp. 56–57 item 2; https://www.irs.gov/publications/p974 .
#[test]
fn ira_selected_health_magi_retains_adjacent_cents_and_normal_owner_path() {
    let d = complete(FederalStatus::Single);
    let p = d.people.as_ref().unwrap();
    let s = d.setup.as_ref().unwrap();
    let f = ira_facts(750000, 0, Answer::Yes);
    let qtp = Default::default();
    for (magi_cents, expected) in
        [(8411972, 5170), (8411999, 5170), (8412000, 5160), (8412001, 5160)]
    {
        let r =
            crate::ira::calculate_with_magi_cents(&f, p, s, [100000, 0], magi_cents, &qtp, [0, 0])
                .unwrap();
        assert_eq!(r.deduction, expected);
        assert_eq!(r.nondeductible, [7500 - expected, 0]);
        assert_eq!(r.work_deduction_cents, expected * 100);
        assert_eq!(r.contributions, [750000, 0]);
    }
    // The normal owner still consumes its existing whole-dollar MAGI; callers
    // outside the selected Iterative stage do not acquire a new rounding rule.
    let r = crate::ira::calculate(&f, p, s, [100000, 0], 84120, &qtp, [0, 0]).unwrap();
    assert_eq!(r.deduction, 5160);
    assert_eq!(r.nondeductible, [2340, 0]);
    let mut full = f;
    full.taxpayer.covered_at_work = false;
    assert_eq!(
        crate::ira::calculate_with_magi_cents(&full, p, s, [100000, 0], 8411999, &qtp, [0, 0])
            .unwrap()
            .deduction,
        7500
    );
    assert_eq!(
        crate::ira::calculate(&full, p, s, [100000, 0], 84120, &qtp, [0, 0]).unwrap().deduction,
        7500
    );
    // Main's Form 5329 line 10 absorbed prior excess is a current deduction
    // input too. Keep its cents in the selected health work without counting
    // the old deposit as a fresh Saver's Credit contribution.
    full.taxpayer.traditional = 500049;
    full.taxpayer.excess = Some(IraExcess {
        traditional_value: Some(2000000),
        roth_value: 0,
        prior_traditional: 200051,
        prior_roth: 0,
        returned_prior_excess: 0,
    });
    let absorbed =
        crate::ira::calculate_with_magi_cents(&full, p, s, [100000, 0], 8411999, &qtp, [0, 0])
            .unwrap();
    assert_eq!(absorbed.absorbed, [200051, 0]);
    assert_eq!(absorbed.work_deduction_cents, 700100);
    assert_eq!(absorbed.deduction, 7001);
    assert_eq!(absorbed.contributions, [500049, 0]);
}

// The existing pension/partly-phased IRA verification guard must read the
// same actual selected MAGI as the IRA owner, including the final cent before
// the band's endpoint. Contribution7400 + pension100 exactly fits the cap.
#[test]
fn ira_selected_health_magi_keeps_pension_phaseout_guard_at_endpoint() {
    let d = complete(FederalStatus::Single);
    let p = d.people.as_ref().unwrap();
    let s = d.setup.as_ref().unwrap();
    let f = ira_facts(740000, 0, Answer::Yes);
    let qtp = Default::default();
    for (magi_cents, phased) in [(9099999, true), (9100000, false), (9100001, false)] {
        let result = crate::ira::validate_pension_contributions(
            &f,
            p,
            s,
            [100000, 0],
            magi_cents,
            [10000, 0],
            &qtp,
        );
        if phased {
            assert_eq!(result.unwrap_err().0, "ISSUE_PENSION_IRA_PHASEOUT_COORDINATION");
        } else {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn print_traditional_ira_stops_after_basis_without_distributions() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 2000, 9000)];
    let mut facts = ira_facts(700_000, 0, Answer::No);
    facts.taxpayer.nondeductible_election = 700_000;
    d.adjustments.as_mut().unwrap().iras = facts;
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_form_line1"), 7000);
    assert_eq!(value(&r, "taxpayer_ira_form_line2"), 2000);
    assert_eq!(value(&r, "taxpayer_ira_form_line3"), 9000);
    assert_eq!(value(&r, "taxpayer_ira_form_line14"), 9000);
    assert!(r.lines.iter().all(|row| row.id != "taxpayer_ira_form_line4"));
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 0, Answer::No);
    let r = supported(&d);
    assert!(r.lines.iter().all(|row| !row.id.starts_with("taxpayer_ira_form_line")));
}

#[test]
fn print_traditional_ira_complete_conversion_keeps_nonpositive_balance_out_of_income() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10_000, conversion(10_000, true)));
    d.adjustments.as_mut().unwrap().ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 20_000, 0)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 0);
    assert_eq!(value(&r, "taxpayer_ira_form_line16"), 10_000);
    assert_eq!(value(&r, "taxpayer_ira_form_line17"), 20_000);
    assert_eq!(value(&r, "taxpayer_ira_form_line18"), -10_000);
    assert!(r.lines.iter().all(|row| row.id != "taxpayer_ira_form_line1"));
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 10_000);
}

#[test]
fn print_traditional_ira_repaid_only_special_distributions_need_no_form() {
    let mut d = special_ira_repayment_example(5000, 20261201);
    d.income.remove(1);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 500_000);
    assert_eq!(return_line(&r, "4b"), 0);
    assert!(r.lines.iter().all(|row| !row.id.starts_with("taxpayer_ira_form_line")));
}

#[test]
fn ira_repayment_records_and_deposits_have_no_native_count_ceiling() {
    use super::support::client::{estimate_documents_from_save, save_reopen_native_sources};
    use crate::tests::cross::interview::{accepted, open_topic};
    for (draft, row, dollars) in [
        (reservist_capacity_return(101, 101), "taxpayer_ira_reservist_basis", 10_201),
        (special_repayment_capacity_return(), "taxpayer_ira_repayment_income_reduction", 10_100),
    ] {
        assert!(draft.valid_stored());
        let mut session = restart(draft);
        let request = open_topic(&mut session, TaxSection::Retirement);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let reply = crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            estimate_documents_from_save(&request),
        );
        assert!(matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. })), "{reply:?}");
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, request));
        assert_eq!(value(&supported(&session.draft), row), dollars);
        let saved = save_reopen_native_sources(
            session.draft,
            &[TaxSection::Retirement],
            &[TaxSection::Retirement],
        );
        assert_eq!(value(&supported(&saved), row), dollars);
        if dollars == 10_100 {
            assert_eq!(return_line(&supported(&saved), "4a"), 1_010_000);
            assert_eq!(return_line(&supported(&saved), "4b"), 0);
        }
    }
}

#[test]
fn ira_repayment_keeps_independent_opening_and_reservist_rounding() {
    let mut d = reservist_capacity_return(1, 1);
    let b = &mut d.adjustments.as_mut().unwrap().ira_basis[0];
    b.prior = 50;
    b.reservist_repayments[0].payments[0].amount = 50;
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_ira_form_line2"), 1);
    assert_eq!(value(&r, "taxpayer_ira_reservist_basis"), 1);
    assert_eq!(value(&r, "taxpayer_ira_form_line3"), 2);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 2);
}

#[test]
fn unit47_deferred_annuity_accepts_more_than_one_hundred_related_contracts() {
    let d = deferred_withdrawal_group(
        1_600_000,
        1_000_000,
        vec![AnnuityBalance { value: 100, investment: 100 }; 101],
        false,
    );
    assert!(d.valid_stored());
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 600_000);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 9_101);
    assert_eq!(supported(&restart(d).draft), r);
}

// D-2026-10-19-12/-14: Form 5329 lines 52 and 53 print whole dollars and the taxed shortfall is
// figured from them. A 5.40 minimum and 3.80 distributed print 5 and 4: a shortfall of 1 and a
// 25% tax of 0 (the exact shortfall 1.60 used to round to 2 and tax 1).
#[test]
fn rmd_form_5329_taxes_the_shortfall_between_the_printed_lines() {
    let mut d = complete(FederalStatus::Single);
    let mut e = rmd_record(0, 0);
    e.required = RmdMinimum::Administrator { amount: 540 };
    e.timely = 380;
    d.adjustments.as_mut().unwrap().required_distributions = vec![e];
    let r = supported(&d);
    let line = |n: &str| {
        r.lines
            .iter()
            .find(|l| l.id == alloc::format!("taxpayer_rmd_line{n}"))
            .unwrap_or_else(|| panic!("Form 5329 line {n}"))
            .amount
    };
    assert_eq!((line("52b"), line("53b")), (500, 400));
    assert_eq!(line("54b"), 0);
    assert_eq!(value(&r, "required_distribution_tax"), 0);
}
