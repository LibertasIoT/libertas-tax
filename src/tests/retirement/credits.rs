use super::*;

#[test]
fn saver_2026_rate_boundaries_for_every_status() {
    use crate::federal_rules::saver_rate;
    for (status, limits) in [
        (FederalStatus::Joint, [48500, 52500, 80500]),
        (FederalStatus::Head, [36375, 39375, 60375]),
        (FederalStatus::Single, [24250, 26250, 40250]),
        (FederalStatus::Separate, [24250, 26250, 40250]),
        (FederalStatus::Surviving, [24250, 26250, 40250]),
    ] {
        for (agi, expected) in [
            (0, 50),
            (limits[0], 50),
            (limits[0] + 1, 20),
            (limits[1], 20),
            (limits[1] + 1, 10),
            (limits[2], 10),
            (limits[2] + 1, 0),
        ] {
            assert_eq!(saver_rate(agi, status), expected, "{status:?}, {agi}");
        }
    }
}

#[test]
fn saver_spousal_distribution_exception_and_ineligible_recipient() {
    use crate::federal_rules::saver_potential;
    let mut p = complete(FederalStatus::Joint).people.unwrap();
    let mut f = saver(700_000, 700_000);
    // Form 8880 example adapted to the 2026 lookback: own 5,000 current-year
    // distribution and spouse 2,000 prior-year distribution when they did not file together.
    f.distributions = vec![
        FederalSaverDistribution {
            owner: Owner::Taxpayer,
            amount: 500_000,
            year: distribution_period(SaverDistributionYear::TaxYear, Answer::No),
        },
        FederalSaverDistribution {
            owner: Owner::Spouse,
            amount: 200_000,
            year: distribution_period(SaverDistributionYear::PriorTwo, Answer::No),
        },
    ];
    assert_eq!(saver_potential(&f, &p, FederalStatus::Joint, 40000), Ok(1000));
    f.distributions[1].year = distribution_period(f.distributions[1].year(), Answer::Yes);
    assert_eq!(saver_potential(&f, &p, FederalStatus::Joint, 40000), Ok(0));
    // An undecided 2027 joint status is assumed joint: the smaller credit.
    f.distributions[1].year =
        distribution_period(SaverDistributionYear::FollowingYear, Answer::NotSure);
    assert_eq!(saver_potential(&f, &p, FederalStatus::Joint, 40000), Ok(0));
    f.distributions[1].year = distribution_period(
        SaverDistributionYear::TaxYear,
        f.distributions[1].joint_with_current_spouse(),
    );
    assert_eq!(saver_potential(&f, &p, FederalStatus::Joint, 40000), Ok(0));
    // Even an ineligible spouse's distribution reduces the eligible filer's base.
    p.spouse.as_mut().unwrap().full_time_student = true;
    f.spouse_contributions = 0;
    f.distributions.remove(0);
    f.taxpayer_contributions = 200_000;
    assert_eq!(saver_potential(&f, &p, FederalStatus::Joint, 40000), Ok(0));
    f.distributions[0].year = distribution_period(
        SaverDistributionYear::FollowingYear,
        f.distributions[0].joint_with_current_spouse(),
    );
    f.distributions[0].year = distribution_period(f.distributions[0].year(), Answer::No);
    assert_eq!(saver_potential(&f, &p, FederalStatus::Joint, 40000), Ok(1000));
    f.distributions[0].year = distribution_period(f.distributions[0].year(), Answer::Yes);
    assert_eq!(saver_potential(&f, &p, FederalStatus::Joint, 40000), Ok(0));
}

#[test]
fn saver_aggregates_distribution_cents_before_rounding() {
    use crate::federal_rules::saver_potential;
    let p = complete(FederalStatus::Single).people.unwrap();
    let mut f = saver(100_000, 0);
    f.distributions = vec![
        FederalSaverDistribution {
            owner: Owner::Taxpayer,
            amount: 49,
            year: distribution_period(SaverDistributionYear::PriorOne, Answer::No),
        };
        4
    ];
    assert_eq!(saver_potential(&f, &p, FederalStatus::Single, 20000), Ok(499));
}

#[test]
fn saver_credit_record_persists_and_rejects_malformed_values() {
    let d = complete(FederalStatus::Single);
    let mut f = d.credits.clone().unwrap();
    f.saver = (Some(saver(200_000, 0))).into();
    f.saver.as_mut().unwrap().distributions.push(FederalSaverDistribution {
        owner: Owner::Taxpayer,
        amount: 10000,
        year: distribution_period(SaverDistributionYear::PriorOne, Answer::No),
    });
    // The legacy credits page is gone (unit M3-P11): the saver's credit is
    // stored through its own page, so the record is placed on the draft and
    // must survive a restart unchanged.
    let mut accepted = d.clone();
    accepted.credits = Some(f.clone());
    let s = restart(accepted.clone());
    assert_eq!(s.draft.credits, Some(f.clone()));
    let mut malformed = accepted.clone();
    malformed.credits.as_mut().unwrap().saver.as_mut().unwrap().taxpayer_contributions = -1;
    assert!(!malformed.valid_stored());
    // 2026-10-15 D-13 removes the former 100-distribution prototype cap.
    let mut many = accepted;
    let f = many.credits.as_mut().unwrap().saver.as_mut().unwrap();
    f.distributions = vec![f.distributions[0].clone(); 101];
    assert!(many.valid_stored());
    assert_eq!(restart(many.clone()).draft.credits, many.credits);
}

// T2-25f: Form 8880, full-return-level fixture (no hand fixture existed for
// this row; the existing saver tests exercise saver_rate/saver_potential
// directly). Every expected total below is hand-computed from the 2026
// f8880.pdf worksheet (lines 1-12) and Notice 2025-67's 2026 AGI bands, never
// read from the App's own output.
#[test]
fn saver_joint_return_both_spouses_contribute_with_a_lookback_distribution() {
    let mut d = complete(FederalStatus::Joint);
    // $30,000 + $15,000 wages, no adjustments: AGI = $45,000.
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 0);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 1_500_000, 0));
    let mut f = saver(180_000, 120_000);
    // The taxpayer received a $500 distribution in the 2024-2027 lookback
    // (2025, "PriorOne") while filing jointly with the current spouse.
    f.distributions = vec![FederalSaverDistribution {
        owner: Owner::Taxpayer,
        amount: 50_000,
        year: distribution_period(SaverDistributionYear::PriorOne, Answer::Yes),
    }];
    d.credits.as_mut().unwrap().saver = Some(f).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 45000);
    // Form 8880 line 4 (taxpayer distributions, joint return): the
    // taxpayer's own $500 withdrawal always reduces the taxpayer's line 1;
    // because the withdrawal year was filed jointly with the current
    // spouse, line 4 also carries $500 into the spouse's column (26 U.S.C.
    // 25B(d)(2)(C)). Line 6 (line 1 minus line 4/5, not below zero, capped
    // at $2,000 each):
    //   taxpayer: $1,800 - $500 = $1,300
    //   spouse:   $1,200 - $500 = $700
    // Line 8 (AGI $45,000, inside the joint 50% band, ceiling $48,500):
    // rate 50%. Line 9/12: ($1,300 + $700) * 50% = $1,000.
    assert_eq!(value(&r, "saver_credit"), 1000);
}

#[test]
fn saver_credit_lands_exactly_on_a_2026_agi_band_edge() {
    // Notice 2025-67's joint band table: 50% through $48,500, 20% through
    // $52,500, 10% through $80,500. Land exactly on the first edge and one
    // dollar past it, with a small enough credit that Form 8880 line 11's
    // tax-after-preceding-credits limit is not the binding constraint
    // (standard deduction $32,200, so $48,500 AGI leaves $16,300 taxable —
    // comfortably more 10%-bracket tax than a $500 credit).
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 4_850_000, 0);
    d.credits.as_mut().unwrap().saver = Some(saver(50_000, 50_000)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 48500);
    // Eligible = min($500,$2,000) + min($500,$2,000) = $1,000; at the AGI
    // ceiling itself the 50% rate still applies (line 8 uses "not over").
    assert_eq!(value(&r, "saver_credit"), 500);
    assert!(r.tax.unwrap() > 50_000);
    d.income[0] = wage(1, Owner::Taxpayer, 4_850_100, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 48501);
    // One dollar over the $48,500 edge drops the rate to 20%: $1,000 * 20%.
    assert_eq!(value(&r, "saver_credit"), 200);
}

#[test]
fn saver_contribution_capped_at_2000_per_person_before_the_rate_applies() {
    // $35,000 AGI sits inside the single 10% band (ceiling table
    // 24,250/26,250/40,250), keeping the credit well under the tax owed on
    // $18,900 of taxable income ($35,000 - $16,100 standard deduction) so
    // the cap, not the tax limit, is what this test proves.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_500_000, 0);
    d.credits.as_mut().unwrap().saver = Some(saver(200_000, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 35000);
    // Line 6: min($2,000 - $0, $2,000) = $2,000. Line 9-12: $2,000 * 10%.
    assert_eq!(value(&r, "saver_credit"), 200);
    assert!(r.tax.unwrap() > 20_000);
    // One dollar over the $2,000 cap does not raise line 6 above $2,000.
    d.credits.as_mut().unwrap().saver.as_mut().unwrap().taxpayer_contributions = 200_100;
    let r = supported(&d);
    assert_eq!(value(&r, "saver_credit"), 200);
}

#[test]
fn saver_distributions_from_the_earliest_and_latest_lookback_years_both_count() {
    // Same 10% band as the cap test, so the $130 credit stays well under
    // the tax owed and only the lookback window is being exercised.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_500_000, 0);
    let mut f = saver(180_000, 0);
    f.distributions = vec![
        // 2024, the earliest year in the 2024-2027 lookback.
        FederalSaverDistribution {
            owner: Owner::Taxpayer,
            amount: 30_000,
            year: distribution_period(SaverDistributionYear::PriorTwo, Answer::No),
        },
        // 2027 before the return's due date (with extensions), the latest
        // year in the lookback.
        FederalSaverDistribution {
            owner: Owner::Taxpayer,
            amount: 20_000,
            year: distribution_period(SaverDistributionYear::FollowingYear, Answer::No),
        },
    ];
    d.credits.as_mut().unwrap().saver = Some(f).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 35000);
    // Line 4: both ends of the lookback count in full: $300 + $200 = $500.
    // Line 6: $1,800 - $500 = $1,300 (under the $2,000 cap). Credit: $1,300 * 10%.
    assert_eq!(value(&r, "saver_credit"), 130);
}

#[test]
fn saver_credit_zero_at_the_age_eighteen_student_and_dependent_exclusions() {
    // Same $35,000/10%-band setup as the cap and lookback tests above, so
    // the credit stays well under the tax owed and only the eligibility
    // exclusion is being exercised.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_500_000, 0);
    d.credits.as_mut().unwrap().saver = Some(saver(200_000, 0)).into();
    // Born January 1, 2009: this crate's day-before-birthday convention
    // (FederalPerson::attained) treats this filer as having attained 18 by
    // December 31, 2026, so Form 8880's age-18 test is still satisfied.
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090101;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 35000);
    // Line 6: min($2,000, $2,000) = $2,000; line 8 at $35,000 AGI (inside
    // the 10% band): 10%. Credit = $2,000 * 10%.
    assert_eq!(value(&r, "saver_credit"), 200);
    // Born one day later, January 2, 2009: turns 18 on January 2, 2027, one
    // day too late for the age-18 test even though AGI and contribution are
    // unchanged.
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090102;
    let r = supported(&d);
    assert_eq!(value(&r, "saver_credit"), 0);
    // An adult full-time student is excluded regardless of age or AGI.
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800615;
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    let r = supported(&d);
    assert_eq!(value(&r, "saver_credit"), 0);
    // A filer someone else can claim as a dependent is excluded, even when
    // nobody actually claims them (FilerDependency::claimable(), not
    // claimed(), is Form 8880's test).
    d.people.as_mut().unwrap().taxpayer.full_time_student = false;
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    let r = supported(&d);
    assert_eq!(value(&r, "saver_credit"), 0);
}
#[test]
fn saver_final_return_uses_age_at_death_not_future_birthday() {
    let mut p = complete(FederalStatus::Single).people.unwrap();
    p.taxpayer.birth_date = 20080615;
    p.taxpayer.died = death(20260613);
    assert_eq!(
        crate::federal_rules::saver_potential(&saver(200000, 0), &p, FederalStatus::Single, 20000),
        Ok(0)
    );
    p.taxpayer.died = death(20260614);
    assert_eq!(
        crate::federal_rules::saver_potential(&saver(200000, 0), &p, FederalStatus::Single, 20000),
        Ok(1000)
    );
}

#[test]
fn automatic_enrollment_saver_uses_gross_withdrawal_once_without_early_tax() {
    for (withdrawal, credit, tax) in [(2000, 200, 693), (2100, 180, 723)] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 2300000, 0);
        add(&mut d, automatic_enrollment_income(Owner::Taxpayer, withdrawal, false));
        d.credits.as_mut().unwrap().saver = Some(saver(300000, 0)).into();
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), withdrawal * 100);
        assert_eq!(value(&r, "agi"), 23000 + withdrawal);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(value(&r, "saver_automatic_withdrawals_taxpayer"), withdrawal);
        assert_eq!(value(&r, "saver_credit"), credit);
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(supported(&restart(d).draft).tax, r.tax);
    }
}

#[test]
fn saver_credit_is_zero_above_the_head_of_household_ceiling() {
    let mut d = complete(FederalStatus::Head);
    // A qualifying child establishes head-of-household eligibility; this
    // return claims no care, education or child-credit-adjacent facts, so
    // only the Saver's Credit is at stake.
    add_child(&mut d);
    // $65,000 wages, no dependents or other credits: only the Saver's
    // Credit is at stake here.
    d.income[0] = wage(1, Owner::Taxpayer, 6_500_000, 0);
    // $1,000 traditional IRA contribution, not covered at work.
    d.adjustments.as_mut().unwrap().iras = ira_facts(100_000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = (Some(saver(0, 0))).into();
    let r = supported(&d);
    // AGI = $65,000 - $1,000 IRA deduction = $64,000, above the 2026
    // head-of-household Saver's Credit ceiling of $60,375 (this crate's
    // saver_rate table): the rate is 0%, so the credit is $0 regardless of
    // the $1,000 contribution.
    assert_eq!(value(&r, "saver_credit"), 0);
    // Confirm this is the AGI ceiling, not a tax-exhaustion limit: plenty of
    // regular tax remains (no other credit competes for it here).
    assert!(r.tax.unwrap() > 0);
}
