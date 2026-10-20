// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
#[test]
fn simplified_office_months_and_shared_owner_limit_are_checked() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let source = d.income[0].id;
    add_simplified_office(&mut d, source, 400);
    let OfficeMethod::Simplified { months } = &mut office_claim_mut(&mut d, source).method else {
        panic!()
    };
    for m in months {
        m.exclusive_days = if m.month < 7 {
            0
        } else if m.month == 7 {
            12
        } else {
            20
        };
    }
    assert_eq!(
        line_amount(&supported(&d), &alloc::format!("business_{source}_home_office")),
        62_500
    );

    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    add(&mut d, income(cash_business(10000)));
    let sources = [d.income[1].id, d.income[2].id];
    add_simplified_office(&mut d, sources[0], 200);
    add_simplified_office(&mut d, sources[1], 100);
    // 2026-10-17 approved HOME map: simplified applies to at most one physical
    // home per owner. The old embedded fields never identified that home.
    assert!(!d.valid_stored());
    d.home_sites.pop();
    let claim = &mut d.home_sites[0].claims[0];
    claim.method = simplified_claim(300).method;
    claim.activities[0].deduction_parts = 2;
    claim.activities.push(OfficeActivity {
        activity: sources[1],
        gross_share: 10000,
        expense_period_share: 10000,
        attribution_reason: alloc::string::String::new(),
        deduction_parts: 1,
    });
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{}_home_office", sources[0])), 1000);
    assert_eq!(value(&r, &alloc::format!("business_{}_home_office", sources[1])), 500);
    let OfficeMethod::Simplified { months } = &mut office_claim_mut(&mut d, sources[0]).method
    else {
        panic!()
    };
    months[1].exclusive_days = 29; // February 2026 is not a leap month.
    assert!(!d.valid_stored());
}

#[test]
fn first_home_exception_shares_lifetime_limit_and_allows_next_year_day_120() {
    let mut d = complete(FederalStatus::Single);
    for simple in [false, true] {
        add(
            &mut d,
            retirement_entry(
                Owner::Taxpayer,
                RetirementKind::Ira,
                10000,
                RetirementTreatment::Exception {
                    second: None,
                    distributed: 20261231,
                    facts: RetirementException::FirstHome {
                        costs: 1000000,
                        paid_on: 20270430,
                        prior_used: 300000,
                        eligible: true,
                    },
                    early_simple: simple,
                },
            ),
        );
    }
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 1750);
    assert_eq!(value(&r, "first_home_used_taxpayer"), 10000);
    assert_eq!(value(&r, "agi"), 80000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income.swap(1, 2);
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 1750);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let RetirementTreatment::Exception { facts, .. } = &mut data.ordinary_distribution else {
        unreachable!()
    };
    let RetirementException::FirstHome { paid_on, .. } = facts else { unreachable!() };
    *paid_on = 20270501;
    unsupported(&d, "Income");
}
#[test]
fn reduced_home_exclusion_uses_service_extended_residence_window() {
    for (status, taxable) in [(FederalStatus::Single, 425000), (FederalStatus::Joint, 300000)] {
        let mut d = complete(status);
        let mut home = official_duty_sale(status == FederalStatus::Joint);
        home.term = GainTerm::Short;
        let HomeExclusion::ServiceSuspension { election, .. } = &home.exclusion else { panic!() };
        home.exclusion = HomeExclusion::ReducedService {
            election: election.clone(),
            reason: HomeSaleTrigger::WorkMove { old_miles: 15, new_miles: 65 },
            taxpayer: DutyReducedHomeUse { history: duty_home_use(365, 365), since_prior: 730 },
            spouse: (status == FederalStatus::Joint)
                .then(|| DutyReducedHomeUse { history: duty_home_use(365, 365), since_prior: 730 }),
        };
        add(&mut d, income(FederalIncome::HomeSale { data: home }));
        assert_eq!(value(&supported(&d), "capital_gain"), taxable);
        assert_eq!(restart(d.clone()).draft, d);
        let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
        let HomeExclusion::ReducedService { election, .. } = &mut data.exclusion else { panic!() };
        // No: the election suspends nothing, so only the ordinary window
        // counts and the reduced exclusion is smaller.
        election.one_property = false;
        assert!(value(&supported(&d), "capital_gain") > taxable);
    }
}
#[test]
fn reduced_home_exclusion_uses_shortest_period_and_work_distance_boundary() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.term = GainTerm::Short;
    home.exclusion = HomeExclusion::Reduced {
        reason: HomeSaleTrigger::WorkMove { old_miles: 15, new_miles: 65 },
        taxpayer: reduced_home_periods(365),
        spouse: None,
    };
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    assert_eq!(value(&supported(&d), "capital_gain"), 425000);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let HomeExclusion::Reduced { taxpayer, .. } = &mut data.exclusion else { panic!() };
    taxpayer.since_prior = 146;
    assert_eq!(value(&supported(&d), "capital_gain"), 500000);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let HomeExclusion::Reduced { reason, .. } = &mut data.exclusion else { panic!() };
    *reason = HomeSaleTrigger::WorkMove { old_miles: 15, new_miles: 64 };
    unsupported(&d, "Income:");
}

#[test]
fn widowed_home_exclusion_uses_calendar_deadline_and_not_return_status() {
    for (sale, death, expected) in [
        (20260615, 20240615, 50000000),
        (20260616, 20240615, 25000000),
        (20260228, 20240229, 50000000),
        (20260301, 20240229, 25000000),
    ] {
        let mut home = home_sale(false);
        home.exclusion = HomeExclusion::Widowed {
            sale_date: sale,
            death_date: death,
            history: home_use(1000, 1000),
            late_prior: false,
            joint_tests_at_death: true,
        };
        assert!(home.valid());
        assert_eq!(home.excluded(FederalStatus::Single), Ok(expected));
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::HomeSale { data: home }));
        assert_eq!(value(&supported(&d), "capital_gain"), 550000 - expected / 100);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn widowed_home_exclusion_checks_both_histories_and_valid_dates() {
    let mut home = home_sale(false);
    home.exclusion = HomeExclusion::Widowed {
        sale_date: 20260615,
        death_date: 20240615,
        history: home_use(1000, 1000),
        late_prior: true,
        joint_tests_at_death: true,
    };
    assert_eq!(home.excluded(FederalStatus::Head), Ok(25000000));
    let HomeExclusion::Widowed { history, .. } = &mut home.exclusion else { panic!() };
    history.prior_exclusion = true;
    assert!(home.excluded(FederalStatus::Joint).is_err());
    let HomeExclusion::Widowed { death_date, .. } = &mut home.exclusion else { panic!() };
    *death_date = 20260230;
    assert!(!home.valid());
}
#[test]
fn reduced_home_exclusion_uses_shared_facility_residence_rule() {
    let mut home = home_sale(false);
    let mut periods = reduced_home_periods(730);
    periods.lived = 365;
    periods.care = home_care(365, 365);
    home.exclusion = HomeExclusion::Reduced {
        reason: HomeSaleTrigger::PhysicianRecommended,
        taxpayer: periods,
        spouse: None,
    };
    assert_eq!(home.excluded(FederalStatus::Single), Ok(25000000));
    let HomeExclusion::Reduced { taxpayer, .. } = &mut home.exclusion else { panic!() };
    let HomeCare { owned_residence_days, .. } = &mut taxpayer.care;
    // Facility time counts only after 365 days of residence in the home.
    *owned_residence_days = 364;
    assert_eq!(home.excluded(FederalStatus::Single), Ok(12500000));
}

#[test]
fn former_home_shortfall_needs_assumption_liability_default_and_no_duplicate() {
    for case in 0..6 {
        let mut d = complete(FederalStatus::Single);
        let mut debt = former_home_mortgage_bad_debt(12000, 2000);
        let FederalIncome::NonbusinessBadDebt {
            data:
                crate::NonbusinessBadDebt {
                    source:
                        BadDebtSource::FormerHomeMortgage {
                            buyer_assumed,
                            secondary_liability,
                            foreclosure_shortfall,
                            no_duplicate_basis_or_loss,
                            ..
                        },
                    investment,
                    ..
                },
        } = &mut debt
        else {
            panic!()
        };
        match case {
            0 => *buyer_assumed = false,
            1 => *secondary_liability = false,
            2 => *foreclosure_shortfall = false,
            3 => *no_duplicate_basis_or_loss = false,
            4 => *investment = true,
            _ => *buyer_assumed = false,
        }
        add(&mut d, income(debt));
        // A personal claim marked investment is a page rule (BAD_DEBT_
        // PERSONAL_CLAIM_NOT_INVESTMENT), not an estimate refusal; a fact that
        // fails takes no loss.
        if case == 4 {
            assert!(!d.valid_stored());
            let FederalIncome::NonbusinessBadDebt { data } = &d.income[1].income else { panic!() };
            assert_eq!(data.rule(), Some("BAD_DEBT_PERSONAL_CLAIM_NOT_INVESTMENT"));
        } else {
            let r = supported(&d);
            assert!(!r.lines.iter().any(|line| line.id.starts_with("bad_debt_")));
        }
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(former_home_mortgage_bad_debt(12000, 2000)));
    let FederalIncome::NonbusinessBadDebt {
        data:
            crate::NonbusinessBadDebt {
                source: BadDebtSource::FormerHomeMortgage { recovered, .. },
                ..
            },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *recovered = 1_200_001;
    assert!(!d.valid_stored());
}
