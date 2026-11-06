use super::*;

#[test]
fn marketplace_annual_rounding_retains_source_cents_and_missing_benchmark_blocks_credit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let months = (1..=12)
        .map(|month| MarketplaceMonth {
            month,
            policies: vec![MarketplacePolicy {
                label: "Policy".into(),
                premium: 10049,
                advance: 49,
                allocation: MarketplaceAllocation::NotShared,
            }],
            benchmark: 20049,
            eligible: MarketplaceEligibility::Everyone,
        })
        .collect();
    d.credits.as_mut().unwrap().marketplace = marketplace_claim(&mut d, months);
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 340); // round(12 * 200.49) - 2066
    assert_eq!(value(&r, "premium_credit_advance"), 6);
    if let MarketplaceClaim::Yes { facts } = &mut d.credits.as_mut().unwrap().marketplace {
        let paper = &mut facts.forms[0].form;
        paper.month_mut(1).benchmark = Some(0);
    }
    unsupported(&d, "Marketplace");
}

#[test]
fn long_term_care_coordinates_ordinary_premiums_and_sep_without_double_deduction() {
    let mut d = empty_return(FederalStatus::Single);
    set_age(&mut d, -1, 45);
    let mut business = care_business(5000, 1000, false);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.retirement = BusinessRetirement::Sep { contribution: 50000, rate: 25.0 };
    let BusinessHealthInsurance::Plan { months, .. } = &mut data.health_insurance else { panic!() };
    months.push(BusinessHealthMonth {
        month: 1,
        premiums: 1000000,
        child_increase: 100000,
        employer_eligible: false,
        reimbursed: None,
    });
    add(&mut d, income(business));
    let person = health_coverage_child(&mut d, 26);
    let FederalIncome::Business { data } = &mut d.income[0].income else { panic!() };
    let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance else {
        panic!()
    };
    long_term_care[0].person = person;
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_health_insurance"), 4146); // profit - $354 SE - $500 SEP
    assert_eq!(value(&r, "business_2_long_term_care_deduction"), 500);
    assert_eq!(value(&r, "business_2_medical_premiums"), 6354);
    assert_eq!(value(&r, "long_term_care_1_medical"), 0);
    assert_eq!(value(&r, "long_term_care_1_excluded"), 500);
    assert_eq!(value(&r, "agi"), 0);
}

#[test]
fn archer_contributions_use_2026_plan_limits_and_actual_deductible() {
    for (deductible, cap) in [(2900, 1885), (3000, 1950), (4400, 2860)] {
        let mut d = complete(FederalStatus::Single);
        set_archer(
            &mut d,
            vec![archer_contributor(Owner::Taxpayer, archer_self(deductible), cap)],
            ArcherSharing::Equal,
        );
        let r = supported(&d);
        assert_eq!(value(&r, "archer_contribution_1_coverage_limit"), cap);
        assert_eq!(value(&r, "archer_msa_deduction"), cap);
        assert_eq!(value(&r, "agi"), 60000 - cap);
        assert_eq!(return_line(&r, "10"), cap * 100);
        assert_eq!(restart(d.clone()).draft, d);
    }
    for coverage in [
        archer_self(2899),
        archer_self(4401),
        ArcherCoverage::SelfOnly { deductible: 300000, out_of_pocket: 585001 },
        archer_family(5849),
        archer_family(8751),
        ArcherCoverage::Family { deductible: 600000, out_of_pocket: 1070001 },
    ] {
        let mut d = complete(FederalStatus::Single);
        set_archer(
            &mut d,
            vec![archer_contributor(Owner::Taxpayer, coverage, 100)],
            ArcherSharing::Equal,
        );
        unsupported(&d, "Archer MSA");
    }
}

#[test]
fn archer_spouse_employer_deposit_blocks_family_but_not_separate_self_only() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 4000000, 0));
    let mut first = archer_contributor(Owner::Taxpayer, archer_self(3000), 1000);
    let mut second = archer_contributor(Owner::Spouse, archer_self(3000), 0);
    second.employer = 100000;
    second.employer_outside = 100000;
    set_archer(&mut d, vec![first.clone(), second.clone()], ArcherSharing::Equal);
    assert_eq!(value(&supported(&d), "archer_msa_deduction"), 1000);
    first.coverage = ArcherEligibility::FullYear { coverage: archer_family(6000) };
    set_archer(&mut d, vec![first, second], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
}

#[test]
fn archer_separate_return_uses_spouse_facts_without_deducting_spouse_deposits() {
    let mut d = complete(FederalStatus::Separate);
    let first = archer_contributor(Owner::Taxpayer, archer_family(6000), 2250);
    let mut second = archer_contributor(Owner::Spouse, archer_family(6000), 2250);
    second.income.clear();
    set_archer(&mut d, vec![first.clone(), second], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_deduction"), 2250);
    assert_eq!(value(&r, "agi"), 57750);
    set_archer(&mut d, vec![first], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
}

#[test]
fn archer_hsa_partial_family_reduction_preserves_separate_self_only_months() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 4000000, 0));
    let mut records = vec![
        archer_contributor(Owner::Taxpayer, archer_family(6000), 1125),
        archer_contributor(Owner::Spouse, archer_family(6000), 1125),
    ];
    for record in &mut records {
        record.coverage = ArcherEligibility::Monthly {
            months: (0..12)
                .map(|m| if m < 6 { ArcherCoverage::None } else { archer_family(6000) })
                .collect(),
        };
    }
    set_archer(&mut d, records, ArcherSharing::Equal);
    let mut hsa = hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 432500);
    hsa.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|m| if m < 6 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    d.adjustments.as_mut().unwrap().hsa.push(hsa);
    // Actual spouse context and1:0 agreement preserve the fixture's allocation;
    // equal default would not transfer the spouse's half to this owner.
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 0,
        other_coverage: Some(HsaEligibility::Monthly {
            months: (0..12)
                .map(|m| if m < 6 { HsaCoverage::None } else { HsaCoverage::Family })
                .collect(),
        }),
    });
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 133425); // $140,000 - $2,250 MSA - ($2,200 + $4,375 - $2,250) HSA.
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn hsa_deemed_combines_with_ordinary_withdrawals_and_keeps_spouse_tax_separate() {
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        add_hsa_deemed(
            &mut d,
            owner,
            MedicalDeemed::Terminated { occurred: 20261231, january_value: 300 },
            None,
        );
    }
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20261001, amount: 100000 }],
        None,
        None,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1006);
    assert_eq!(value(&r, "hsa_additional_tax"), 202); // Each owner's8889 rounds separately.
    let mut comparison = d.clone();
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let source = hsa_distribution_mut(&mut comparison, owner, 0);
        source.origin = HealthDocumentOrigin::Form1099Sa;
        source.payment = Some(MedicalSaPayment::Own {
            medical: 0,
            nonmedical: vec![MedicalAccountWithdrawal { date: 20260101, amount: 300 }],
            disabled_since: None,
            rollover: None,
        });
        source.form = sa_source(Sa1099 {
            origin: HealthDocumentOrigin::Form1099Sa,
            box3_code: Some(1),
            ..source.internal()
        })
        .form;
    }
    assert_eq!(r.tax, supported(&comparison).tax);
}

#[test]
fn hsa_rollover_exclusion_does_not_consume_contribution_capacity() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().hsa =
        vec![hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 440000)];
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260101, amount: 100000 }],
        None,
        Some(hsa_rollover()),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_rollover"), 5000);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1000);
    assert_eq!(value(&r, "hsa_additional_tax"), 200);
    assert_eq!(restart(d.clone()).draft, d);
    let mut without = d.clone();
    let source = hsa_distribution_mut(&mut without, Owner::Taxpayer, 0);
    let Some(MedicalSaPayment::Own { rollover, .. }) = &mut source.payment else { panic!() };
    let rollover =
        crate::kernel::ratio::exact::rounded(&rollover.take().unwrap().amount()).unwrap();
    *source.gross_mut() -= rollover;
    assert_eq!(r.tax, supported(&without).tax);
    assert_eq!(r.refund, supported(&without).refund);
}

#[test]
fn hsa_rollover_sixty_days_and_prior_distribution_anniversary() {
    let mut d = complete(FederalStatus::Single);
    let mut rollover = hsa_rollover();
    rollover.receipts[0].received = 20261231;
    rollover.receipts[0].deposits[0].deposited = 20270301;
    rollover.preceding = Some(20251231);
    add_hsa_distribution(&mut d, Owner::Taxpayer, 0, vec![], None, Some(rollover));
    supported(&d); // Current contribution eligibility is not required.
    let Some(MedicalSaPayment::Own { rollover, .. }) =
        &mut hsa_distribution_mut(&mut d, Owner::Taxpayer, 0).payment
    else {
        panic!()
    };
    rollover.as_mut().unwrap().receipts[0].deposits[0].deposited = 20270302;
    unsupported(&d, "Income");
    let Some(MedicalSaPayment::Own { rollover, .. }) =
        &mut hsa_distribution_mut(&mut d, Owner::Taxpayer, 0).payment
    else {
        panic!()
    };
    let r = rollover.as_mut().unwrap();
    r.receipts[0].deposits[0].deposited = 20270301;
    r.preceding = None;
    // Current-year history now comes from the actual first source receipt,
    // not an echoed prior-year field. Jan1→Dec31 is short of its anniversary.
    add_hsa_distribution(&mut d, Owner::Taxpayer, 0, vec![], None, Some(hsa_rollover()));
    unsupported(&d, "Income");
}

#[test]
fn hsa_rollovers_keep_owner_history_separate() {
    let mut d = complete(FederalStatus::Joint);
    add_hsa_distribution(&mut d, Owner::Taxpayer, 0, vec![], None, Some(hsa_rollover()));
    let mut spouse = hsa_rollover();
    spouse.receipts[0].deposits[0].amount = 900000;
    add_hsa_distribution(&mut d, Owner::Spouse, 0, vec![], None, Some(spouse));
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_rollover"), 5000);
    assert_eq!(value(&r, "hsa_2_rollover"), 9000);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 0);
}

#[test]
fn hsa_rollover_rejects_early_and_postdeath_deposit() {
    let mut d = complete(FederalStatus::Single);
    let mut rollover = hsa_rollover();
    rollover.receipts[0].deposits[0].deposited = 20251231;
    add_hsa_distribution(&mut d, Owner::Taxpayer, 0, vec![], None, Some(rollover));
    assert!(!hsa_distribution_mut(&mut d, Owner::Taxpayer, 0).record().valid());
    let mut final_return = death(20260115).unwrap();
    final_return.ordinary = true;
    d.people.as_mut().unwrap().taxpayer.died = Some(final_return);
    let Some(MedicalSaPayment::Own { rollover, .. }) =
        &mut hsa_distribution_mut(&mut d, Owner::Taxpayer, 0).payment
    else {
        panic!()
    };
    *rollover = Some(hsa_rollover());
    unsupported(&d, "Income");
}

#[test]
fn hsa_two_prior_funding_transfers_keep_separate_testing_periods() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.funding_history =
        vec![hsa_funding(20250604, 20260701, 280000), hsa_funding(20250815, 20260701, 300000)];
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260801, amount: 100000 }],
        None,
        None,
    );
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 3000);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 300);
    assert_eq!(value(&r, "hsa_additional_tax"), 200);
    assert_eq!(return_line(&r, "8"), 400000);
    assert_eq!(return_line(&r, "23"), 50000);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.funding_history[1].funded = 20250620;
    assert!(!h.valid());
}

#[test]
fn hsa_funding_recapture_keeps_spouses_separate() {
    let mut d = complete(FederalStatus::Joint);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.funding_history = vec![hsa_funding(20251231, 20260615, 600)];
    let mut spouse = h.clone();
    spouse.owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().hsa = vec![h, spouse];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 12);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 2);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.funding_history[0].funded = 20260101;
    assert!(!h.valid()); // Current-year transfer qualification not yet implemented.
}

#[test]
fn uncorrected_employer_hsa_excess_separates_w2_wages_from_other_income() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    h.employer = 500000;
    h.employer_outside = 500000;
    h.excess_accounting = Some(HsaExcessAccounting {
        prior_year: None,
        employer_excess_in_wages: 0,
        year_end_value: 1000000,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
    });
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_employer_excess"), 600);
    assert_eq!(value(&r, "hsa_return_year_uncorrected_employer_excess_other_income"), 600);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 36);
    assert_eq!(value(&r, "agi"), 60600);
    assert_eq!(return_line(&r, "8"), 60000);
    assert_eq!(return_line(&r, "23"), 3600);
    assert_eq!(restart(d.clone()).draft, d);

    let accounting = d.adjustments.as_mut().unwrap().hsa[0].excess_accounting.as_mut().unwrap();
    accounting.employer_excess_in_wages = 60000;
    let wages = supported(&d);
    assert_eq!(value(&wages, "hsa_return_year_uncorrected_employer_excess_other_income"), 0);
    assert_eq!(value(&wages, "agi"), 60000);
    assert_eq!(value(&wages, "hsa_return_year_excess_contribution_tax"), 36);

    d.adjustments.as_mut().unwrap().hsa[0].personal = 100000;
    let both = supported(&d);
    assert_eq!(value(&both, "hsa_1_return_year_uncorrected_excess"), 1000);
    assert_eq!(value(&both, "hsa_1_return_year_uncorrected_employer_excess"), 600);
    assert_eq!(value(&both, "hsa_return_year_excess_contribution_tax"), 96);
    assert_eq!(value(&both, "adjustments"), 0);

    d.adjustments.as_mut().unwrap().hsa[0]
        .excess_accounting
        .as_mut()
        .unwrap()
        .employer_excess_in_wages = 60001;
    unsupported(&d, "HSA excess");
}

#[test]
fn hsa_overlapping_testing_periods_cap_last_month_income_at_non_ira_contributions() {
    // Notice 2008-51 examples 6–8, with 2025 limits and June–December coverage.
    for (funding, cash, loss, last, ira, tax) in [
        (500000, 355000, 20260701, 3550, 0, 355),
        (100000, 755000, 20260701, 3563, 0, 356),
        (500000, 355000, 20260501, 3550, 5000, 855),
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
        let mut history = last_month_history(7, cash);
        history.months[5..].fill(HsaCoverage::Family);
        history.loss.as_mut().unwrap().date = loss;
        h.last_month_history = Some(history);
        h.funding_history = vec![hsa_funding(20250604, loss, funding)];
        d.adjustments.as_mut().unwrap().hsa = vec![h];
        let r = supported(&d);
        assert_eq!(value(&r, "hsa_last_month_recapture"), last);
        assert_eq!(value(&r, "hsa_funding_recapture"), ira);
        assert_eq!(value(&r, "hsa_testing_period_tax"), tax);
        assert_eq!(return_line(&r, "8"), (last + ira) * 100);
        assert_eq!(return_line(&r, "23"), tax * 100);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn current_hsa_last_month_catchup_archer_reduction_and_separate_owner_limits() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19711231;
    let h = december_hsa(HsaCoverage::SelfOnly, 345000);
    set_archer(
        &mut d,
        vec![archer_contributor(Owner::Taxpayer, archer_self(3000), 1950)],
        ArcherSharing::Equal,
    );
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 5400);
    assert_eq!(value(&r, "hsa_1_last_month_contributions"), 3450);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
    let mut d = complete(FederalStatus::Joint);
    let h = december_hsa(HsaCoverage::SelfOnly, 440000);
    let mut spouse = h.clone();
    spouse.owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().hsa = vec![h, spouse];
    assert_eq!(value(&supported(&d), "adjustments"), 8800);
}

#[test]
fn historical_hsa_family_allocation_coordinates_qualified_ira_funding() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(1, 227500);
    historical_hsa_family(&mut history, 1, 1);
    h.last_month_history = Some(history);
    h.funding_history = vec![hsa_funding(20251215, 20260601, 200000)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 2275);
    assert_eq!(value(&r, "hsa_funding_recapture"), 2000);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 428);
    assert_eq!(return_line(&r, "8"), 427500);
    assert_eq!(return_line(&r, "23"), 42800);
}

#[test]
fn separate_hsa_december_family_uses_external_coverage_without_spouse_return_amounts() {
    let mut d = separate_december_hsa(437500);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4375);
    assert_eq!(value(&r, "agi"), 55625);
    assert_eq!(value(&r, "hsa_1_last_month_contributions"), 4010);
    assert!(!r.lines.iter().any(|v| v.id == "hsa_2_last_month_contributions"));
    assert_eq!(return_line(&r, "8"), 0);
    assert_eq!(return_line(&r, "23"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn separate_hsa_december_family_taxpayer_catchup_and_recapture_stay_on_this_return() {
    let mut d = separate_december_hsa(537500);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19711231;
    d.adjustments.as_mut().unwrap().hsa[0].last_month_loss =
        Some(HsaEligibilityLoss { date: 20261215, reason: HsaLossReason::Other });
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 5375);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 4927);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 493);
    assert_eq!(return_line(&r, "8"), 492700);
    assert_eq!(return_line(&r, "23"), 49300);
}

#[test]
fn separate_hsa_family_both_archer_deposits_reduce_allowance_but_only_own_are_deducted() {
    let mut d = separate_december_hsa(212500);
    let first = archer_contributor(Owner::Taxpayer, archer_family(6000), 2250);
    let mut second = archer_contributor(Owner::Spouse, archer_family(6000), 2250);
    second.income.clear();
    set_archer(&mut d, vec![first, second], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_deduction"), 2250);
    assert_eq!(value(&r, "adjustments"), 4375);
    assert_eq!(value(&r, "agi"), 55625);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn separate_hsa_family_requires_correct_context_and_excludes_spouse_hsa_entry() {
    let mut d = separate_december_hsa(437500);
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().other_coverage = None;
    unsupported(&d, "HSA");
    let mut d = separate_december_hsa(437500);
    let spouse = hsa_entry(Owner::Spouse, HsaCoverage::Family, 0);
    add_hsa_distribution(
        &mut d,
        Owner::Spouse,
        0,
        vec![MedicalAccountWithdrawal { date: 20260601, amount: 100000 }],
        None,
        None,
    );
    d.adjustments.as_mut().unwrap().hsa.push(spouse);
    unsupported(&d, "HSA");
    let mut d = joint_december_hsa(437500, 437500, 1, 1);
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().other_coverage =
        Some(HsaEligibility::FullYear { coverage: HsaCoverage::Family });
    unsupported(&d, "HSA");
}

#[test]
fn separate_hsa_family_zero_spouse_share_still_requires_family_coverage() {
    let mut d = separate_december_hsa(875000);
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().spouse_parts = 0;
    assert_eq!(value(&supported(&d), "adjustments"), 8750);
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().other_coverage =
        Some(HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly });
    unsupported(&d, "HSA");
}

#[test]
fn monthly_family_hsa_supports_separate_return_without_december_eligibility() {
    let mut d = separate_december_hsa(218750);
    let mut months = vec![HsaCoverage::Family; 6];
    months.extend(vec![HsaCoverage::None; 6]);
    d.adjustments.as_mut().unwrap().hsa[0].coverage =
        HsaEligibility::Monthly { months: months.clone() };
    let share = d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap();
    share.method = HsaAllocationMethod::Monthly;
    share.other_coverage = Some(HsaEligibility::Monthly { months });
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 2188);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 0);
    assert!(!r.review_notes.iter().any(|v| v.contains("uses the 2026 last-month rule")));
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn monthly_family_hsa_archer_offset_and_self_only_months_keep_separate_limits() {
    let mut d = separate_december_hsa(338750);
    let mut months = vec![HsaCoverage::Family; 6];
    months.extend(vec![HsaCoverage::SelfOnly; 6]);
    d.adjustments.as_mut().unwrap().hsa[0].coverage =
        HsaEligibility::Monthly { months: months.clone() };
    let share = d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap();
    share.method = HsaAllocationMethod::Monthly;
    share.other_coverage = Some(HsaEligibility::Monthly { months });
    let first = archer_contributor(Owner::Taxpayer, archer_family(6000), 1000);
    let mut second = archer_contributor(Owner::Spouse, archer_family(6000), 1000);
    second.income.clear();
    set_archer(&mut d, vec![first, second], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4388);
    assert_eq!(value(&r, "archer_msa_deduction"), 1000);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn current_hsa_funding_excludes_ira_income_and_early_tax_but_preserves_gross() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4000);
    d.adjustments.as_mut().unwrap().hsa[0].personal = 40000;
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 600000);
    assert_eq!(return_line(&r, "4b"), 200000);
    assert_eq!(value(&r, "early_distribution_tax"), 200);
    assert_eq!(value(&r, "adjustments"), 400);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}
