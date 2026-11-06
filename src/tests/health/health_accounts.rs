// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
#[test]
fn loan_phaseout_and_hsa_limits() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 9_250_000, 0);
    let a = d.adjustments.as_mut().unwrap();
    a.set_student_loan_interest(250_000);
    assert_eq!(value(&supported(&d), "adjustments"), 1250);
    d.adjustments.as_mut().unwrap().hsa.push(FederalHsa {
        personal_outside: 440_000,

        employer_outside: 0,
        owner: Owner::Taxpayer,
        coverage: HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly },
        personal: 440_000,
        employer: 0,
        late_excess: vec![],
        prior_employer_excess: vec![],
        corrections: vec![],
        excess_accounting: None,
        funding: vec![],
        funding_history: vec![],
        uses: vec![],
        last_month_history: None,
        last_month_loss: None,
    });
    assert!(supported(&d).tax.is_some());
    d.adjustments.as_mut().unwrap().hsa[0].personal += 100;
    unsupported(&d, "HSA");
}
#[test]
fn additional_medicare_and_niit_are_not_silently_zero() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21_000_000, 0);
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(1_000_000, 0)) }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "additional_medicare_tax"), 90);
    assert_eq!(value(&r, "net_investment_income_tax"), 380);
    assert_eq!(value(&r, "payments"), 90);
}

#[test]
fn young_filer_investment_tax_is_screened_even_when_not_claimable() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20080615;
    d.people.as_mut().unwrap().taxpayer.support_share = SupportShare::ExactlyHalf;
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(300_000, 0)) }),
    );
    unsupported(&d, "Investment tax");
    d.people.as_mut().unwrap().taxpayer.support_share = SupportShare::MoreThanHalf;
    assert!(supported(&d).tax.is_some());
}

#[test]
fn setup_unions_round_trip_every_applicable_branch() {
    let situations = [
        MaritalState::Unmarried,
        MaritalState::Married { living: SpouseLiving::Together },
        MaritalState::Married { living: SpouseLiving::LastSixMonths },
        MaritalState::Married { living: SpouseLiving::AllYear },
        MaritalState::WidowedPriorYear { could_file_joint: true },
        MaritalState::WidowedTwoYearsPrior { could_file_joint: false },
    ];
    for marital_state in situations {
        for status in [
            FilingChoice::Single,
            FilingChoice::Joint,
            FilingChoice::Separate { section179: Section179Spouse::NotSupplied },
            FilingChoice::Head,
            FilingChoice::Surviving,
        ] {
            let mut draft = complete(FederalStatus::Single);
            let setup = draft.setup.as_mut().unwrap();
            setup.marital_state = marital_state;
            setup.status = status;
            assert_eq!(restart(draft.clone()).draft, draft);
        }
    }
}

#[test]
fn partial_2025_cash_fuel_recovery_uses_refigured_taxable_income_once() {
    let recovery = partial_cash_fuel_recovery();
    assert!(recovery.valid() && recovery.qualified());
    // Refigured line 15 is $718. The $1,250 gross-less-original-negative
    // shortcut would overstate the tax benefit because half-SE/QBI changed.
    assert_eq!(recovery.excluded_amount(), 428_200);
    assert_eq!(recovery.taxable_amount(), 71_800);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    let no_recovery_receipts = business.receipts();
    business.cash_fuel_tax_recovery = Some(recovery);
    assert_eq!(business.receipts(), no_recovery_receipts + 71_800);
    let mut draft = complete(FederalStatus::Single);
    let mut baseline_draft = draft.clone();
    let mut baseline_business = business.clone();
    baseline_business.cash_fuel_tax_recovery = None;
    add(&mut baseline_draft, income(FederalIncome::Business { data: baseline_business }));
    let baseline = supported(&baseline_draft);
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let report = supported(&draft);
    for (suffix, expected) in [("gross", 500_000), ("excluded", 428_200), ("", 71_800)] {
        let id = if suffix.is_empty() {
            "business_2_cash_fuel_tax_recovery".into()
        } else {
            alloc::format!("business_2_cash_fuel_tax_recovery_{suffix}")
        };
        assert_eq!(line_amount(&report, &id), expected,);
    }
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31"));
    assert!(value(&report, "agi") > value(&baseline, "agi"));
    assert!(report.tax.unwrap() > baseline.tax.unwrap());
    assert!(report.review_notes.iter().any(|note| {
        note.contains("Partial 2025 fuel-tax benefit")
            && note.contains("Synthetic filed 2025 Form 1040")
    }));
    let note = &report
        .review_notes
        .iter()
        .find(|note| note.contains("Partial 2025 fuel-tax benefit"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_PARTIAL_FUEL_TAX_BENEFIT");
    assert_eq!(
        note.args[1],
        MessageArg::Text(
            "Synthetic filed 2025 Form 1040, Schedule C, fuel ledger and regular-tax comparison"
                .into()
        )
    );
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn off_highway_fuel_credit_is_a_payment_and_taxable_business_income_once() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data } = cash_business(10_000) else { unreachable!() };
    let mut business = data;
    business.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Fuel including federal tax".into() },
        30_000,
    ));
    let claim = FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            gasoline_off_highway: Some(fuel_use(100_250, 20_000)),
            diesel_off_highway: Some(fuel_use(40_000, 10_000)),
            ..empty_fuel_lines()
        }
        .into(),
        expense_treatment: FederalFuelExpenseTreatment::FullCostDeducted,
        records_complete: true,
    };
    assert_eq!(
        claim.labeled_lines().filter(|(_, v)| *v != 0).collect::<Vec<_>>(),
        vec![("1a", 1835), ("3a", 972)]
    );
    assert_eq!(claim.credit_cents(), 2807);
    business.fuel_tax_credit = Some(claim);
    assert_eq!(business.receipts(), 1_000_000);
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let cash_report = supported(&draft);
    assert_eq!(return_line(&cash_report, "31"), 2800);
    assert_eq!(value(&cash_report, "business_2_fuel_1a"), 18);
    assert_eq!(value(&cash_report, "business_2_fuel_3a"), 9);
    assert_eq!(cash_report.payments, Some(652_800));
    assert_eq!(estimate(&restart(draft.clone()).draft), cash_report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.cash_fuel_tax_recovery = Some(cash_fuel_recovery(72_900)); // Earlier Form 4136 filed in 2026.
    assert_eq!(data.receipts(), 1_072_900);
    let prior_recovery = supported(&draft);
    assert_eq!(line_amount(&prior_recovery, "business_2_cash_fuel_tax_recovery"), 72_900);
    assert_eq!(estimate(&restart(draft.clone()).draft), prior_recovery);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.cash_fuel_tax_recovery = None;
    let BusinessActivity::Ordinary { accounting, .. } = &mut data.activity else { unreachable!() };
    *accounting = BusinessAccounting::Accrual;
    assert_eq!(data.receipts(), 1_002_807);
    let accrual_report = supported(&draft);
    assert_eq!(accrual_report.payments, cash_report.payments);
    assert_ne!(accrual_report.tax, cash_report.tax);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().expense_treatment =
        FederalFuelExpenseTreatment::TaxExcludedFromExpenses;
    assert_eq!(data.receipts(), 1_000_000);
    let net_expense = supported(&draft);
    assert_eq!(return_line(&net_expense, "31"), 2800);
    assert_eq!(net_expense.payments, cash_report.payments);
}

#[test]
fn hsa_monthly_eligibility_and_medical_withdrawals_after_medicare() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19610701;
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 270_000);
    let mut months = vec![HsaCoverage::SelfOnly; 6];
    months.extend(vec![HsaCoverage::None; 6]);
    h.coverage = HsaEligibility::Monthly { months };
    add_hsa_distribution(&mut d, Owner::Taxpayer, 900_000, vec![], None, None);
    d.adjustments.as_mut().unwrap().hsa.push(h);
    assert_eq!(value(&supported(&d), "adjustments"), 2700);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 100;
    unsupported(&d, "HSA");
    d.adjustments.as_mut().unwrap().hsa[0].personal = 0;
    d.adjustments.as_mut().unwrap().hsa[0].coverage =
        HsaEligibility::FullYear { coverage: HsaCoverage::None };
    assert_eq!(value(&supported(&d), "hsa_taxable_distributions"), 0);
}

#[test]
fn hsa_shared_family_limit_does_not_transfer_unused_catchup() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19711231;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19711231;
    d.adjustments.as_mut().unwrap().hsa = vec![
        hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 975_000),
        hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 100_000),
    ];
    // The unequal ordinary allocation is an actual agreement; deposits alone
    // no longer imply a transfer of the spouse's statutory equal share.
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 0,
        other_coverage: None,
    });
    assert_eq!(value(&supported(&d), "adjustments"), 10750);
    let a = d.adjustments.as_mut().unwrap();
    a.hsa[0].personal += 100;
    a.hsa[1].personal -= 100;
    unsupported(&d, "HSA");
    let a = d.adjustments.as_mut().unwrap();
    a.hsa[0].personal = 537_500;
    a.hsa[1].personal = 537_500;
    a.hsa_family_share = None; // The statutory equal default needs no agreement.
    supported(&d);
    d.adjustments.as_mut().unwrap().hsa[1].personal += 100;
    unsupported(&d, "HSA");
}

#[test]
fn hsa_nonmedical_withdrawal_date_and_disability_control_additional_tax() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19610701;
    // One actual disability onset replaces contradictory per-payment answers.
    // June30/July1 are not after the onset/birthday; July2 is after both.
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![
            MedicalAccountWithdrawal { date: 20260630, amount: 100_000 },
            MedicalAccountWithdrawal { date: 20260701, amount: 200_000 },
            MedicalAccountWithdrawal { date: 20260702, amount: 300_000 },
        ],
        Some(20260701),
        None,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 6000);
    assert_eq!(value(&r, "hsa_additional_tax"), 600);
    assert_eq!(value(&r, "agi"), 66000);
    assert_eq!(return_line(&r, "8"), 600_000);
    assert_eq!(return_line(&r, "23"), 60_000);
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800701;
    assert_eq!(value(&supported(&d), "hsa_additional_tax"), 600);
}

#[test]
fn hsa_records_validate_month_count_and_reject_duplicate_owner() {
    let mut d = complete(FederalStatus::Single);
    let h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 100_000);
    d.adjustments.as_mut().unwrap().hsa = vec![h.clone(), h];
    unsupported(&d, "HSA");
}
#[test]
fn marketplace_2026_percentage_boundaries_and_regional_poverty_lines() {
    use crate::marketplace::{applicable, poverty};
    for (percent, rate) in [
        (100, 210),
        (132, 210),
        (133, 314),
        (149, 413),
        (150, 419),
        (199, 655),
        (200, 660),
        (249, 840),
        (250, 844),
        (299, 993),
        (300, 996),
        (400, 996),
    ] {
        assert_eq!(applicable(percent), rate);
    }
    assert_eq!(poverty(MarketplaceResidence::Contiguous, 4), 32150);
    assert_eq!(poverty(MarketplaceResidence::Alaska, 2), 26430);
    assert_eq!(poverty(MarketplaceResidence::Hawaii, 2), 24320);
}
#[test]
fn marketplace_credit_repayment_and_return_lines_use_2026_rules() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 600, 0)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 5134);
    assert_eq!(value(&r, "marketplace_annual_contribution"), 2066);
    assert_eq!(return_line(&r, "31"), 513_400);
    let before = r.tax.unwrap();
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 600, 500)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "premium_credit_repayment"), 866); // No pre-2026 repayment cap.
    assert_eq!(return_line(&r, "17"), 86_600);
    assert_eq!(r.tax, Some(before + 86_600));
    assert_eq!(restart(d.clone()).draft, d);
    d.income[0] = wage(1, Owner::Taxpayer, 6_260_000, 0);
    assert_eq!(value(&supported(&d), "premium_tax_credit"), 965);
    d.income[0] = wage(1, Owner::Taxpayer, 6_260_100, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "premium_credit_repayment"), 6000);
}
#[test]
fn marketplace_repayment_enters_amt_comparison_before_total_tax() {
    // $60,000 wages produce $5,023 regular Tax Table tax. The three ISO
    // spreads produce no AMT, $151 AMT, and $39,151 AMT before repayment.
    for (shares, old_amt, new_amt, total) in
        [(0, 0, 0, 11023), (500, 151, 0, 11023), (2000, 39151, 30196, 43784)]
    {
        let mut d = complete(FederalStatus::Single);
        // 2026-11-02: automatic selection compares the premiums made
        // deductible by repayment through the existing AMT owner.
        d.deductions.as_mut().unwrap().itemize_override = None;
        if shares > 0 {
            d.screening.as_mut().unwrap().iso_exercises =
                vec![iso_exercise("Synthetic ISO", f64::from(shares), 1000, 11000)];
        }
        let before = supported(&d);
        assert_eq!(value(&before, "alternative_minimum_tax"), old_amt);
        d.credits.as_mut().unwrap().marketplace = marketplace_claim(
            &mut d,
            (1..=12).map(|m| marketplace_month(m, 500, 100, 500)).collect(),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "premium_tax_credit"), 0);
        assert_eq!(value(&r, "premium_credit_repayment"), 6000);
        assert_eq!(value(&r, "alternative_minimum_tax"), new_amt);
        assert_eq!(return_line(&r, "16"), if shares == 2000 { 758800 } else { 502300 });
        assert_eq!(return_line(&r, "17"), (new_amt + 6000) * 100);
        assert_eq!(return_line(&r, "18"), total * 100);
        assert_eq!(r.tax, Some(total * 100));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn marketplace_repayment_increases_form1116_category_limit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(foreign_pay(40000)));
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic foreign employer".into(),
            category: ForeignTaxCategory::Employment,
            income: 4000000,
            tax: 3000000,
        }],
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let before = supported(&d);
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 600, 500)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "premium_credit_repayment"), 6000);
    // Foreign income is 40% of both AGI and taxable income after allocated
    // standard deduction: $6,000 repayment raises the category cap by $2,400.
    assert_eq!(value(&r, "foreign_tax_credit"), value(&before, "foreign_tax_credit") + 2400);
    assert_eq!(
        value(&r, "foreign_general_unused_tax"),
        value(&before, "foreign_general_unused_tax") - 2400
    );
    assert_eq!(
        value(&r, "foreign_general_amt_unused_tax"),
        value(&before, "foreign_general_amt_unused_tax")
    );
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(r.tax, before.tax.map(|v| v + 360000));
}

#[test]
fn marketplace_monthly_multiple_policies_do_not_duplicate_family_benchmark() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let mut months: alloc::vec::Vec<_> =
        (1..=6).map(|m| marketplace_month(m, 250, 600, 100)).collect();
    for m in &mut months {
        m.policies.push(MarketplacePolicy {
            label: "Second policy".into(),
            premium: 25_000,
            advance: 10_000,
            allocation: MarketplaceAllocation::NotShared,
        });
    }
    d.credits.as_mut().unwrap().marketplace = marketplace_claim(&mut d, months);
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 2568);
    assert_eq!(value(&r, "premium_credit_refundable"), 1368);
    if let MarketplaceClaim::Yes { facts, .. } = &mut d.credits.as_mut().unwrap().marketplace {
        facts.eligibility[0].status =
            MarketplacePersonEligibility::OtherMec { kind: MarketplaceMecKind::Medicare };
    }
    assert_eq!(value(&supported(&d), "premium_tax_credit"), 2140);
    if let MarketplaceClaim::Yes { facts, .. } = &mut d.credits.as_mut().unwrap().marketplace {
        facts.eligibility[0].status = MarketplacePersonEligibility::Unknown;
    }
    // Unresolved individual eligibility is retained and blocks reconciliation;
    // known mixed eligibility is covered by the canonical source regressions.
    unsupported(&d, "Marketplace source");
}
#[test]
fn marketplace_dependent_income_uses_return_components_and_identity_remapping() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    add_child(&mut d);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let id = d.dependents[0].id;
    let person = id;
    let claim = marketplace_claim(&mut d, vec![marketplace_month(1, 500, 600, 0)]);
    let c = d.credits.as_mut().unwrap();
    c.marketplace = claim;
    let MarketplaceClaim::Yes { facts } = &mut c.marketplace else { unreachable!() };
    facts.dependents = vec![MarketplaceDependent {
        person,
        agi: 1_100_000,
        exempt_interest: 50_000,
        foreign_exclusion: 200_000,
        social: 400_000,
        taxable_social: 100_000,
    }];
    assert_eq!(value(&supported(&d), "marketplace_household_income"), 47800);
    assert_eq!(restart(d.clone()).draft, d);
    d.dependents.swap(0, 1);
    // The dependent return names its person by identity, wherever the person
    // is in the list.
    assert_eq!(
        crate::marketplace::facts(&d.credits.as_ref().unwrap().marketplace).unwrap().dependents[0]
            .person,
        id
    );
    assert_eq!(value(&supported(&d), "marketplace_household_income"), 47800);
    // A person who left the return takes their dependent return with them at
    // the prune (F3, 2026-10-09); no stored copy of the people keeps it.
    d.dependents.retain(|v| v.id != id);
    all_ticked(&mut d);
    let pruned = crate::interview::prune(&mut d);
    assert!(
        crate::marketplace::facts(&d.credits.as_ref().unwrap().marketplace)
            .unwrap()
            .dependents
            .is_empty()
    );
    assert!(
        pruned.removed.iter().any(|item| item.key() == "SAVING_KIND_MARKETPLACE_DEPENDENT_RETURN"),
        "{pruned:?}"
    );
}
#[test]
fn marketplace_premiums_flow_into_medical_itemization_after_credit() {
    let d = marketplace_wage_draft(
        6_260_100,
        (1..=12).map(|m| marketplace_month(m, 2000, 2100, 500)).collect(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "deduction"), 19305); // 24,000 premiums less 7.5% AGI, rounded.
}

// Derived Pub502 premium-expense endpoint: eligible medical cents from a
// saved Marketplace statement and a separate manual expense aggregate once.
// There is no PTC above400%FPL and no self-employed deduction in this case.
#[test]
fn marketplace_medical_cents_join_other_medical_sources_before_rounding() {
    let mut d = marketplace_wage_draft(10_000_000, vec![marketplace_month(1, 1000, 1100, 0)]);
    marketplace_facts_mut(&mut d).forms[0].form.month_mut(1).premium = Some(100_049);
    itemized(d.deductions.as_mut().unwrap()).medical = 49;
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    let before = d.clone();
    let result = supported(&d);
    assert_eq!(value(&result, "premium_tax_credit"), 0);
    assert_eq!(value(&result, "medical_expenses"), 1001);
    assert_eq!(restart(d.clone()).draft, before);
    assert_eq!(d, before);
}

// A shared tax-family percentage alone does not prove another enrollee's
// medical eligibility or actual incremental premium. The known mandatory
// reconciliation survives omission of that unverified expense benefit.
#[test]
fn marketplace_medical_missing_cost_class_preserves_known_credit_reconciliation() {
    let d = marketplace_wage_draft(
        3_130_000,
        (1..=12)
            .map(|m| shared_month(m, 800, 700, 450, MarketplaceAllocation::Shared { percent: 100 }))
            .collect(),
    );
    let result = supported(&d);
    assert_eq!(value(&result, "premium_tax_credit"), 6336);
    assert_eq!(value(&result, "premium_credit_advance"), 5400);
    assert!(noted(&result, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
    assert_eq!(value(&result, "medical_expenses"), 0);
}

// Pub502(2025 final), printedp21, Child under age27 Examples1/2:
// https://www.irs.gov/pub/irs-pdf/p502.pdf
// Published premium10000/allowedSEdeduction4000, actual child increase5000/0,
// gives medical5000/6000. Derived2026 full-return completion: one January
// source aggregates the published annual cost; wages100000 ensure no PTC;
// ScheduleC4304 minus actual half-SE304 establishes4000 capacity. Child26
// supports themself from savings, grossincome0/no required return, actual
// NoFamily attestation and own-only employer MEC. Neutral SLCSP1200/600
// distinguishes the math classes; it does not set actual child cost5000.
#[test]
fn marketplace_medical_actual_child_cost_matches_publication502_endpoints() {
    for (increase, expected) in [(500000, 5000), (0, 6000)] {
        let mut d = marketplace_nonmedical_child(Some(increase));
        add(&mut d, income(marketplace_business(4304, 1..=1)));
        assign_marketplace_businesses(&mut d);
        let policy = marketplace_facts_mut(&mut d).forms[0].id;
        d.adjustments.as_mut().unwrap().health_marketplace.benchmarks =
            vec![HealthPolicyBenchmark {
                policy,
                month: 1,
                coverage: MarketplaceBenchmarkSource::Marketplace { amount: 120000 },
                noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 60000 },
            }];
        let before = d.clone();
        let result = supported(&d);
        assert_eq!(value(&result, "premium_tax_credit"), 0);
        // WorksheetP first consumes the exact nonspecified10000/3;
        // the remaining specified allowance rounds667, total adjustment4000.
        assert_eq!(value(&result, "marketplace_specified_deduction"), 667);
        assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 4000);
        assert_eq!(value(&result, "medical_expenses"), expected);
        assert_eq!(value(&result, "agi"), 100000);
        assert!(!noted(&result, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
        assert_eq!(restart(d.clone()).draft, before);
        assert_eq!(d, before);
    }
}

#[test]
fn marketplace_medical_actual_child_funding_and_missing_cost_are_distinct() {
    let mut d = marketplace_nonmedical_child(Some(40000));
    let policy = &mut marketplace_facts_mut(&mut d).forms[0];
    let paper = &mut policy.form;
    paper.month_mut(1).premium = Some(100000);
    policy.payments.push(MarketplacePaymentMonth {
        month: 1,
        paid: 100000,
        refunded: 0,
        unpaid: 0,
        tax_free_premiums: 20000,
        tax_free_coverage: None,
        coverage: MarketplacePaymentCoverage::PaidByDeadline,
        appeal: None,
    });
    // Net source800: child400−actual child funding150=250;
    // independently medical gross600−other funding50=550.
    policy.medical_costs[0].tax_free_child = Some(15000);
    assert_eq!(value(&supported(&d), "medical_expenses"), 550);
    marketplace_facts_mut(&mut d).forms[0].medical_costs[0].tax_free_child = Some(0);
    assert_eq!(value(&supported(&d), "medical_expenses"), 400);
    marketplace_facts_mut(&mut d).forms[0].medical_costs[0].tax_free_child = None;
    let missing = supported(&d);
    assert!(noted(&missing, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
    assert_eq!(value(&missing, "medical_expenses"), 0);
    marketplace_facts_mut(&mut d).forms[0].medical_costs[0].tax_free_child = Some(20001);
    crate::tests::support::client::retained_marketplace_conflict(&d);
    let missing = supported(&marketplace_nonmedical_child(None));
    assert!(noted(&missing, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
}

#[test]
fn marketplace_child_medical_eligibility_changes_keep_actual_cost_facts() {
    let mut d = marketplace_nonmedical_child(Some(500000));
    assert_eq!(value(&supported(&d), "medical_expenses"), 5000);
    let original = marketplace_facts_mut(&mut d).forms[0].medical_costs.clone();
    d.dependents[0].facts = 0;
    d.dependents[0].situation =
        DependentSituation::Supported { your_support_over_half: true, gross_income: 0 };
    // The child is still coverage-only, but now qualifies under the existing
    // medical dependent rules. Physical premium cost is unchanged.
    assert!(d.valid_stored());
    assert_eq!(value(&supported(&d), "medical_expenses"), 10000);
    assert_eq!(marketplace_facts_mut(&mut d).forms[0].medical_costs, original);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn marketplace_nonspecified_premiums_consume_the_existing_trade_limit_first() {
    // Derived Pub974 WorksheetP/Form7206 stop boundary. The actual source
    //10000 with1200/600 benchmarks has exact nonspecified10000/3, above
    // ScheduleC3000−actual halfSE212=2788. No specified capacity remains;
    // high independent wages make PTCzero without a fabricated credit input.
    let mut d = marketplace_nonmedical_child(Some(0));
    add(&mut d, income(marketplace_business(3000, 1..=1)));
    assign_marketplace_businesses(&mut d);
    let policy = marketplace_facts_mut(&mut d).forms[0].id;
    d.adjustments.as_mut().unwrap().health_marketplace.benchmarks = vec![HealthPolicyBenchmark {
        policy,
        month: 1,
        coverage: MarketplaceBenchmarkSource::Marketplace { amount: 120000 },
        noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 60000 },
    }];
    let result = supported(&d);
    assert_eq!(value(&result, "schedule_1_half_self_employment_tax"), 212);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 2788);
    assert_eq!(value(&result, "marketplace_specified_deduction"), 0);
    assert_eq!(value(&result, "medical_expenses"), 7212);
    assert_eq!(value(&result, "agi"), 100000);
}

// Derived combined2026 endpoint, not an IRS-published combined example.
// Pub502(2025 final), printed21 Child under age27 and printed10 PTC:
// https://www.irs.gov/pub/irs-pdf/p502.pdf
// Actual premium10000 = medical5000 + child5000; current2026 PTC928.
// The credit is excluded from medical cost once, independently of the child
// cost; changing actual child cost does not change mandatory reconciliation.
#[test]
fn marketplace_medical_positive_subsidy_uses_actual_child_cost_once() {
    let mut d = marketplace_nonmedical_child(Some(500000));
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let result = supported(&d);
    assert_eq!(value(&result, "premium_tax_credit"), 928);
    assert_eq!(value(&result, "medical_expenses"), 4072);
    assert_eq!(value(&result, "premium_tax_credit") + value(&result, "medical_expenses"), 5000);
    // Verified zero actual increase is a different source, even though the
    // nonmedical child remains outside the premium-credit coverage family.
    marketplace_facts_mut(&mut d).forms[0].medical_costs[0].child_increase = Some(0);
    let zero = supported(&d);
    assert_eq!(value(&zero, "premium_tax_credit"), 928);
    assert_eq!(value(&zero, "medical_expenses"), 9072);
}

// Derived extensions of Pub502's published10000/5000/4000 endpoint,
// https://www.irs.gov/pub/irs-pdf/p502.pdf printed21, with current2026 PTC.
// Wage31300 and ScheduleC4304−actual halfSE304 retain AGI31300 afterD4000,
// selecting the independently covered one-month PTC928. Real tax-free funding
// changes expense classes, never that credit or the enrolled benchmark.
#[test]
fn marketplace_medical_child_deduction_and_tax_free_funding_conserve_source() {
    for (tax_free, funded_child, expected) in
        [(0, None, 4072), (200000, Some(150000), 3072), (200000, Some(0), 2072)]
    {
        let mut d = marketplace_nonmedical_child(Some(500000));
        d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
        add(&mut d, income(marketplace_business(4304, 1..=1)));
        assign_marketplace_businesses(&mut d);
        let policy = marketplace_facts_mut(&mut d).forms[0].id;
        d.adjustments.as_mut().unwrap().health_marketplace.benchmarks =
            vec![HealthPolicyBenchmark {
                policy,
                month: 1,
                coverage: MarketplaceBenchmarkSource::Marketplace { amount: 120000 },
                noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 60000 },
            }];
        if tax_free > 0 {
            let p = &mut marketplace_facts_mut(&mut d).forms[0];
            p.payments.push(MarketplacePaymentMonth {
                month: 1,
                paid: 1000000,
                refunded: 0,
                unpaid: 0,
                tax_free_premiums: tax_free,
                tax_free_coverage: Some(100000),
                coverage: MarketplacePaymentCoverage::PaidByDeadline,
                appeal: None,
            });
            p.medical_costs[0].tax_free_child = funded_child;
        }
        let before = d.clone();
        let result = supported(&d);
        assert_eq!(value(&result, "premium_tax_credit"), 928);
        assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 4000);
        assert_eq!(value(&result, "medical_expenses"), expected);
        assert!(4000 + 928 + expected + tax_free / 100 <= 10000);
        assert!(!noted(&result, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
        assert_eq!(restart(d.clone()).draft, before);
        assert_eq!(d, before);
    }
}

// Derived two-business source split: each actually establishes half of the
// same premium10000/child5000. D4000/D2000 use their own child2500 first,
// consuming medical1500/0; PTC928 leaves medical5000−1500−928=2572.
// The source split is the existing consistent reasonable method under
// Pub974(2025), printed53–54, not a second child price or policy ledger.
#[test]
fn marketplace_medical_child_cost_preserves_business_funding_identity() {
    let mut d = marketplace_nonmedical_child(Some(500000));
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    add(&mut d, income(marketplace_business(4304, 1..=1)));
    add(&mut d, income(marketplace_business(2152, 1..=1)));
    assign_marketplace_businesses(&mut d);
    let policy = marketplace_facts_mut(&mut d).forms[0].id;
    d.adjustments.as_mut().unwrap().health_marketplace.benchmarks = vec![HealthPolicyBenchmark {
        policy,
        month: 1,
        coverage: MarketplaceBenchmarkSource::Marketplace { amount: 120000 },
        noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 60000 },
    }];
    let before = d.clone();
    let result = supported(&d);
    assert_eq!(value(&result, "premium_tax_credit"), 928);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 6000);
    assert_eq!(value(&result, "medical_expenses"), 2572);
    assert_eq!(6000 + 928 + 2572, 9500); // Other500 remains unused nonmedical-child cost.
    d.income.reverse();
    d.adjustments.as_mut().unwrap().health_marketplace.policies.reverse();
    let reordered = supported(&d);
    for key in [
        "premium_tax_credit",
        "schedule_1_self_employed_health_insurance",
        "medical_expenses",
        "agi",
    ] {
        assert_eq!(value(&reordered, key), value(&result, key));
    }
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(restart(before.clone()).draft, before);
}

// Derived source-identity endpoint, not a published IRS combined example.
// Pub502 child-first permission belongs to that actual policy's cost. Policy1
// has paid10000/child5000/D4000; policy2 has paid2000/child0 and its separate
// business. Credit follows enrollment5:1; policy2's remaining paid cost is
// fully used by that business. Policy1's unused child1000 cannot shelter a
// deduction on policy2 or become another source's medical benefit.
#[test]
fn marketplace_medical_unused_child_cost_cannot_cross_policy_sources() {
    let mut d = marketplace_nonmedical_child(Some(500000));
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    let second_id = d.next_id;
    d.next_id += 1;
    let facts = marketplace_facts_mut(&mut d);
    let mut second = facts.forms[0].clone();
    second.id = second_id;
    second.form.policy_number = "Second actual statement".into();
    second.form.set_month(1, 200000, Some(110000), 0);
    second.coverage[0].people = vec![-1];
    second.medical_costs.clear();
    facts.forms.push(second);
    add(&mut d, income(marketplace_business(4304, 1..=1)));
    add(&mut d, income(marketplace_business(2152, 1..=1)));
    assign_marketplace_businesses(&mut d);
    let policy = marketplace_facts_mut(&mut d).forms[0].id;
    let owner = &mut d.adjustments.as_mut().unwrap().health_marketplace;
    owner.method = HealthPtcMethod::Iterative;
    owner.policies[1].policy = second_id;
    owner.benchmarks = vec![HealthPolicyBenchmark {
        policy,
        month: 1,
        coverage: MarketplaceBenchmarkSource::Marketplace { amount: 120000 },
        noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 60000 },
    }];
    let before = d.clone();
    let result = supported(&d);
    let credit = line_amount(&result, "marketplace_iterative_credit_work");
    // Round the final remaining source expense once. Its exact numerator is
    // (500000c×6 − credit×5)/6, before the ordinary cents-to-dollar boundary.
    let expected = (3_000_000 - credit * 5 + 300) / 600;
    assert_eq!(value(&result, "medical_expenses"), expected);
    assert!(expected > 4000 && expected < 4500);
    assert!(!noted(&result, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
    d.income.reverse();
    marketplace_facts_mut(&mut d).forms.reverse();
    d.adjustments.as_mut().unwrap().health_marketplace.policies.reverse();
    let reordered = supported(&d);
    assert_eq!(value(&reordered, "medical_expenses"), expected);
    assert_eq!(value(&reordered, "agi"), value(&result, "agi"));
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(restart(before.clone()).draft, before);
}

// Pub974 work retains the existing Schedule C/F owner's source profit cents.
// Below the Schedule SE filing threshold there is no half-tax adjustment:
// profit200.49 and premiums600.00 independently give deduction200.49 and
// medical399.51. An elected-method earnings feed remains its own owner input.
#[test]
fn iterative_health_limit_retains_actual_small_business_profit_cents() {
    let mut d = empty_return(FederalStatus::Single);
    let mut business = marketplace_business(200, 1..=1);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts = 20049;
    let BusinessHealthInsurance::Plan { months, .. } = &mut data.health_insurance else {
        unreachable!()
    };
    months[0].premiums = 60000;
    add(&mut d, income(business));
    let before = d.clone();
    let mut result = supported(&d);
    assert_eq!(value(&result, "self_employment_tax"), 0);
    let health = crate::health::calculate_cents(
        &d,
        [0, 0],
        None,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        None,
        &mut result,
    );
    assert_eq!(health.work_deduction(), crate::kernel::ratio::exact::whole(20049));
    assert_eq!(health.medical, 39951);
    assert_eq!(health.deduction, 200);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Form7206 line4 uses the insured farm's ScheduleF profit, including linked
// 1099-G/PATR income. Two farms have own profits100/50; a100.49 source adds
// only to its linked farm. No SE tax applies below400. Paid600 therefore
// gives health200.49/medical399.51 when linked here, or100/500 when linked
// to the other farm. These are derived source endpoints, not IRS examples.
#[test]
fn iterative_health_limit_reuses_linked_farm_income_in_source_cents() {
    for cooperative in [false, true] {
        for insured in [false, true] {
            let mut d = empty_return(FederalStatus::Single);
            let farm_id = d.next_id;
            let mut farm = raised_farm(100, 0);
            let FederalIncome::Business { data } = &mut farm else { unreachable!() };
            data.health_insurance = BusinessHealthInsurance::Plan {
                months: vec![BusinessHealthMonth {
                    month: 1,
                    premiums: 60000,
                    child_increase: 0,
                    employer_eligible: false,
                    reimbursed: None,
                }],
                long_term_care: vec![],
                establishment: HealthEstablishment::SoleProprietor,
            };
            add(&mut d, income(farm));
            let other_id = d.next_id;
            add(&mut d, income(raised_farm(50, 0)));
            let linked = if insured { farm_id } else { other_id };
            let source = if cooperative {
                let mut data = crate::tests::farm::farm::cooperative(10049, 0, 0, 0);
                data.allocations.push(crate::cooperative::automatic(&data));
                FederalIncome::Cooperative { data: cooperative_source(data) }
            } else {
                let FederalIncome::GovernmentPayment { mut data } =
                    government_unemployment(0, &[], 0).income
                else {
                    unreachable!()
                };
                data.form.agriculture_payments = Some(10049);
                FederalIncome::GovernmentPayment { data }
            };
            add(&mut d, income(source));
            if cooperative {
                claim_last_cooperative(&mut d, linked);
            } else {
                let component = crate::tests::support::builders::farm_government_component(&d);
                crate::tests::support::builders::relate_farm_component(&mut d, linked, component);
            }
            let before = d.clone();
            let mut result = supported(&d);
            assert_eq!(value(&result, "self_employment_tax"), 0);
            assert_eq!(return_line(&result, "8"), 25000);
            let health = crate::health::calculate_cents(
                &d,
                [0, 0],
                None,
                &Default::default(),
                &Default::default(),
                &Default::default(),
                None,
                &mut result,
            );
            let deduction = if insured { 20049 } else { 10000 };
            assert_eq!(health.work_deduction(), crate::kernel::ratio::exact::whole(deduction));
            assert_eq!(health.medical, 60000 - deduction);
            assert_eq!(health.deduction, if insured { 200 } else { 100 });
            assert_eq!(d, before);
            d.income.reverse();
            let mut reordered = supported(&d);
            let health = crate::health::calculate_cents(
                &d,
                [0, 0],
                None,
                &Default::default(),
                &Default::default(),
                &Default::default(),
                None,
                &mut reordered,
            );
            assert_eq!(health.work_deduction(), crate::kernel::ratio::exact::whole(deduction));
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

// Pub974 WorksheetW lines4–7 retain actual box14 income and attribute the
// SAME filed Schedule1line15 amount. No second SE tax is calculated. Derived
// cases below retain all annual-dollar owner results and test only cent feeds.
#[test]
fn iterative_health_partner_source_and_filed_half_attribution_keep_cents() {
    let mut d = empty_return(FederalStatus::Single);
    let mut partner = partner_health_source(600, false);
    partner.regular.ordinary = 10049;
    partner.alternative = Some(partner.regular.clone());
    partner.reported_se = Some(10049);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 10049 });
    partner.qbi = None;
    partner.qbi_records = Default::default();
    let se = partner
        .se_after_passive(
            crate::tests::support::builders::k1_amount(0),
            crate::tests::support::builders::k1_amount(0),
        )
        .unwrap();
    assert_eq!(se.earnings, 100);
    assert_eq!(se.earnings_cents, 10049);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let mut result = supported(&d);
    let earnings = alloc::collections::BTreeMap::from([(id, se.earnings_cents)]);
    let health = crate::health::calculate_cents(
        &d,
        [0, 0],
        None,
        &earnings,
        &Default::default(),
        &Default::default(),
        None,
        &mut result,
    );
    assert_eq!(health.work_deduction(), crate::kernel::ratio::exact::whole(10049));
    assert_eq!(health.medical, 49951);
    assert_eq!(health.deduction, 100);

    // Independently: fixed filedhalf283 ×1000.49/(1000.49+3000.51)
    // =70.766975... ->70.77; partner receives212.23. Sum remains283.
    let mut d = empty_return(FederalStatus::Single);
    let mut business = health_business(1000, 2000, false);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts = 100049;
    let business_id = d.next_id;
    add(&mut d, income(business));
    let mut partner = ordinary_partner_source(3000, true);
    partner.regular.ordinary = 300051;
    partner.alternative = Some(partner.regular.clone());
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(300051);
    let se = partner
        .se_after_passive(
            crate::tests::support::builders::k1_amount(0),
            crate::tests::support::builders::k1_amount(0),
        )
        .unwrap();
    let partner_id = d.next_id;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let owner = crate::partnership::owner_actual_se(&d, Owner::Taxpayer, 0, 0, &[(partner_id, se)])
        .unwrap();
    assert_eq!(owner.half, 283);
    assert_eq!(owner.attribution[&business_id], 71);
    assert_eq!(owner.attribution[&partner_id], 212);
    assert_eq!(owner.attribution_cents[&business_id], 7077);
    assert_eq!(owner.attribution_cents[&partner_id], 21223);
    assert_eq!(owner.attribution_cents.values().sum::<i64>(), owner.half * 100);
    let mut result = supported(&d);
    let health = crate::health::calculate_cents(
        &d,
        [owner.half, 0],
        Some(&owner.attribution_cents),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        None,
        &mut result,
    );
    assert_eq!(health.work_deduction(), crate::kernel::ratio::exact::whole(92972));
    assert_eq!(health.medical, 107028);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn iterative_health_optional_attribution_copies_filed_totals_in_cents() {
    let mut d = empty_return(FederalStatus::Single);
    let mut ids = vec![];
    for gross in [600049, 600051] {
        let mut farm = raised_farm(6000, 5000);
        let FederalIncome::Business { data } = &mut farm else { unreachable!() };
        let BusinessActivity::Farming { data: facts } = &mut data.activity else { unreachable!() };
        facts.accounting =
            FarmAccounting::Cash { receipts: vec![FarmReceipt::Raised { amount: gross }] };
        data.health_insurance = BusinessHealthInsurance::Plan {
            months: vec![BusinessHealthMonth {
                month: 1,
                premiums: 400000,
                child_increase: 0,
                employer_eligible: false,
                reimbursed: None,
            }],
            long_term_care: vec![],
            establishment: HealthEstablishment::SoleProprietor,
        };
        ids.push(d.next_id);
        add(&mut d, income(farm));
    }
    elect_se(&mut d, SeMethod::Farm);
    let mut result = supported(&d);
    let optional = crate::optional_se::calculate(&d, Owner::Taxpayer, &mut result).unwrap();
    // Annual filed4b7560 stays unchanged. Allocate that total using actual
    // gross6000.49/6000.51:378000±0.629947... rounds377999/378001c.
    assert_eq!(optional.lines.optional_earnings, 7560);
    assert_eq!(optional.weights_cents, vec![(ids[0], 377999), (ids[1], 378001)]);
    assert_eq!(optional.weights_cents.iter().map(|(_, amount)| amount).sum::<i64>(), 756000);
    let earnings = optional.weights_cents.into_iter().collect();
    // Filedhalf579 is fixed. Each attributable share rounds28950c, leaving
    //349049/349051c of health capacity; their sum is698100c.
    assert_eq!(value(&result, "self_employment_tax"), 1157);
    let health = crate::health::calculate_cents(
        &d,
        [579, 0],
        None,
        &earnings,
        &Default::default(),
        &Default::default(),
        None,
        &mut result,
    );
    assert_eq!(health.work_deduction(), crate::kernel::ratio::exact::whole(698100));
    assert_eq!(health.medical, 101900);
    let before = d.clone();
    d.income.reverse();
    let optional = crate::optional_se::calculate(&d, Owner::Taxpayer, &mut result).unwrap();
    let earnings: alloc::collections::BTreeMap<_, _> = optional.weights_cents.into_iter().collect();
    assert_eq!(earnings[&ids[0]], 377999);
    assert_eq!(earnings[&ids[1]], 378001);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(restart(before.clone()).draft, before);
}

// Pub974 Iterative worksheets preserve cents; WorksheetP/Form7206 uses the
// selected >2% S-corporation W-2's Medicare wages, not rounded Box1 income.
// Independent limit: paid6000 minus wage5000.49 leaves medical999.51. The
// adjacent50c case checks filing rounding without changing that exact work.
#[test]
fn iterative_scorp_health_limit_retains_actual_medicare_wage_cents() {
    for wages in [500049, 500050] {
        let mut d = empty_return(FederalStatus::Single);
        let wage_id = d.next_id;
        let mut source = wage(wage_id, Owner::Taxpayer, 3000000, 0);
        let FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source { form: data, .. }) } =
            &mut source.income
        else {
            unreachable!()
        };
        data.medicare_wages = Some(wages);
        add(&mut d, source);
        let mut scorp = complete_ordinary_scorp(0, true);
        scorp.health_insurance = BusinessHealthInsurance::Plan {
            months: vec![BusinessHealthMonth {
                month: 1,
                premiums: 600000,
                child_increase: 0,
                employer_eligible: false,
                reimbursed: None,
            }],
            long_term_care: vec![],
            establishment: HealthEstablishment::Scorporation {
                wage: wage_id,
                payment: EntityPremiumPayment::EntityPaid,
            },
        };
        add(&mut d, income(FederalIncome::Scorp { data: scorp_source(scorp) }));
        let before = d.clone();
        let mut result = supported(&d);
        let health = crate::health::calculate_cents(
            &d,
            [0, 0],
            None,
            &Default::default(),
            &Default::default(),
            &Default::default(),
            None,
            &mut result,
        );
        assert_eq!(health.work_deduction(), crate::kernel::ratio::exact::whole(wages));
        assert_eq!(health.medical, 600000 - wages);
        assert_eq!(health.deduction, if wages == 500049 { 5000 } else { 5001 });
        assert_eq!(d, before);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn marketplace_medical_entity_reimbursement_preserves_actual_child_funding() {
    let mut d = marketplace_nonmedical_child(Some(500000));
    let mut scorp = complete_ordinary_scorp(0, true);
    let FederalIncome::Business { data } = marketplace_business(0, 1..=1) else { unreachable!() };
    scorp.health_insurance = data.health_insurance;
    let BusinessHealthInsurance::Plan { establishment, .. } = &mut scorp.health_insurance else {
        unreachable!()
    };
    *establishment = HealthEstablishment::Scorporation {
        wage: 1,
        payment: EntityPremiumPayment::Reimbursed { amount: 600000, included_in_income: true },
    };
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(scorp) }));
    let policy = marketplace_facts_mut(&mut d).forms[0].id;
    let activity = d.income.last().unwrap().id;
    d.adjustments.as_mut().unwrap().health_marketplace.benchmarks = vec![HealthPolicyBenchmark {
        policy,
        month: 1,
        coverage: MarketplaceBenchmarkSource::Marketplace { amount: 120000 },
        noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 60000 },
    }];
    d.adjustments.as_mut().unwrap().health_marketplace.policies =
        vec![HealthPolicyAllocation { policy, month: 1, activity, parts: 1, reimbursed: None }];
    let assignment = &mut d.adjustments.as_mut().unwrap().health_marketplace.policies[0];
    assignment.reimbursed =
        Some(ReimbursedPolicyPremiums { premiums: 600000, coverage: 600000, child: Some(100000) });
    let result = supported(&d);
    assert_eq!(value(&result, "marketplace_specified_deduction"), 6000);
    assert_eq!(value(&result, "medical_expenses"), 0);
    // Derived combined source: actual child reimbursement4000 exceeds the
    // silver-plan nonspecified10000/3. That is valid: these are independent
    // classifications. Net selectedD+PTC consumes specified20000/3;
    // D uses its own child4000 first, leaving medical7000/3, rounded2333.
    // Actual child reimbursement is not guessed from a silver-plan ratio.
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    d.adjustments.as_mut().unwrap().health_marketplace.policies[0]
        .reimbursed
        .as_mut()
        .unwrap()
        .child = Some(400000);
    let with_credit = supported(&d);
    assert!(value(&with_credit, "premium_tax_credit") > 0);
    assert_eq!(value(&with_credit, "medical_expenses"), 2333);
    assert!(!noted(&with_credit, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
    assert_eq!(restart(d.clone()).draft, d);
    // Six thousand cannot all be reimbursed medical cost when actual
    // medical cost is only five thousand. Clipping the remainder is no proof.
    d.adjustments.as_mut().unwrap().health_marketplace.policies[0]
        .reimbursed
        .as_mut()
        .unwrap()
        .child = Some(0);
    assert!(!d.valid_stored());
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d.adjustments.as_mut().unwrap().health_marketplace.policies[0]
        .reimbursed
        .as_mut()
        .unwrap()
        .child = None;
    let missing = supported(&d);
    assert!(noted(&missing, "HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"));
    assert_eq!(value(&missing, "marketplace_specified_deduction"), 6000);
}
#[test]
fn converted_funds_cannot_also_be_qualified_charitable_distributions() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add(
        &mut d,
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 10000, conversion(10000, false)),
    );
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20261231, 10000, 0)];
    unsupported(&d, "QCD");
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 10000);
    assert_eq!(value(&r, "roth_conversion_taxable"), 10000);
    assert_eq!(return_line(&r, "4b"), 1000000);
}

#[test]
fn health_ltc_omissions_keep_same_named_sources_and_raw_policies_distinct() {
    let mut draft = complete(FederalStatus::Single);
    for _ in 0..2 {
        let mut entry = income(health_business(20_000, 1_000, true));
        entry.label = "Acme business".into();
        let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
        let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance
        else {
            unreachable!()
        };
        long_term_care.push(BusinessCarePremium {
            person: -1,
            months: vec![BusinessCareMonth {
                month: 1,
                premiums: 10_000,
                employer_eligible: true,
                reimbursed: None,
            }],
            qualified: CareContractQualification::Qualified,
        });
        add(&mut draft, entry);
    }
    for entry in &mut draft.income {
        if let FederalIncome::Business { data } = &mut entry.income
            && let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance
        {
            long_term_care[0].qualified = CareContractQualification::Nonqualified;
        }
    }
    let omitted = supported(&draft);
    let omissions: Vec<_> = omitted
        .review_notes
        .iter()
        .filter(|note| note.display.key == "WARN_LTC_PREMIUM_LEFT_OUT")
        .collect();
    assert_eq!(omissions.len(), 2, "{omitted:?}");
    for note in &omissions {
        let you = MessageArg::Encoded(Message::new("PERSON_YOU", vec![]).encode());
        assert_eq!(note.display.args, vec![you.clone()]);
        assert_eq!(note.context.as_ref().unwrap().label(), Message::new("TEXT", vec![you]));
        assert_eq!(
            crate::interview::issue_page(&draft, note.context.as_ref().unwrap(), None),
            None
        );
    }
    assert_ne!(
        omissions[0].context.as_ref().unwrap().identity(),
        omissions[1].context.as_ref().unwrap().identity()
    );
}

#[test]
fn two_employee_taxable_care_attribution_uses_automatic_received_shares() {
    // Manual taxable-share answers were removed by the approved map.
    // Equal received amounts determine equal taxable shares without an answer.
    let mut d = joint_employer_care_return();
    care_box10(&mut d, 0, 500_000);
    care_box10(&mut d, 1, 500_000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_taxable"), 2_500);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 1_250);
    assert_eq!(value(&r, "care_taxable_spouse"), 1_250);
    let mut low_cost = joint_employer_care_return();
    care_costs(&mut low_cost, 100_000);
    let r = supported(&low_cost);
    assert_eq!(value(&r, "care_benefits_taxable"), 3_000);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 1_500);
    assert_eq!(value(&r, "care_taxable_spouse"), 1_500);
    assert_eq!(value(&supported(&joint_employer_care_return()), "care_benefits_taxable"), 0);
}
#[test]
fn one_employee_grace_and_current_w2_share_2026_cap_and_taxable_owner() {
    let mut d = benefit_return(FederalStatus::Single);
    grace_and_current_benefits(&mut d);
    care_box10(&mut d, 0, 700_000);
    care_costs(&mut d, 900_000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line12"), 7_000);
    assert_eq!(value(&r, "care_form2441_line13"), 2_000);
    assert_eq!(value(&r, "care_form2441_line15"), 9_000);
    assert_eq!(value(&r, "care_form2441_line20"), 9_000);
    assert_eq!(value(&r, "care_benefits_excluded"), 7_500);
    assert_eq!(value(&r, "care_benefits_taxable"), 1_500);
    assert_eq!(value(&r, "care_credit"), 0);
    assert_eq!(return_line(&r, "1a"), 6_000_000);
    assert_eq!(return_line(&r, "1e"), 150_000);
    assert_eq!(return_line(&r, "25a"), 650_000);
    care_box10(&mut d, 0, 500_000);
    d.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::Plans { maximum: 500_000 };
    let narrower = supported(&d);
    assert_eq!(value(&narrower, "care_benefits_excluded"), 5_000);
    assert_eq!(value(&narrower, "care_benefits_taxable"), 2_000);
    assert_eq!(return_line(&narrower, "1e"), 200_000);
    assert_eq!(supported(&restart(d).draft), narrower);
}
#[test]
fn grace_and_current_w2_taxable_benefit_belongs_to_receiving_spouse() {
    let mut d = benefit_return(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 50_000, 0));
    clear_care_benefits(&mut d);
    let id = w2(&d, 2);
    care_w2(&mut d, id, 300_000, 0, 0, 750_000);
    d.credits.as_mut().unwrap().care.grace =
        vec![CareGraceBalance { owner: Owner::Spouse, used_in_return_year: 200_000 }];
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line20"), 500);
    assert_eq!(value(&r, "care_benefits_excluded"), 500);
    assert_eq!(value(&r, "care_benefits_taxable"), 4_500);
    assert_eq!(value(&r, "care_earned_spouse"), 5_000);
    assert_eq!(return_line(&r, "1a"), 10_050_000);
    assert_eq!(return_line(&r, "1e"), 450_000);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(supported(&restart(d).draft), r);
}
#[test]
fn prior_employer_grace_only_taxable_remainder_belongs_to_employee() {
    let mut d = benefit_return(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 50_000, 0));
    grace_only_benefits(&mut d);
    d.credits.as_mut().unwrap().care.grace[0].owner = Owner::Spouse;
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 500);
    assert_eq!(value(&r, "care_benefits_taxable"), 1_500);
    assert_eq!(value(&r, "care_earned_spouse"), 2_000);
    assert_eq!(return_line(&r, "1a"), 10_050_000);
    assert_eq!(return_line(&r, "1e"), 150_000);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn governmental_457_requires_known_allocation_within_taxable_payment() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260601,
                facts: RetirementException::Governmental457 {
                    rollover_taxable: 900001,
                    eligible: true,
                },
                early_simple: false,
            },
        ),
    );
    let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
        &mut d.income[1].income
    else {
        unreachable!()
    };
    data.form.taxable = Some(900000);
    data.taxable_method = Some(TaxableMethod::BoxTwoA);
    unsupported(&d, "Income");
    let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
        &mut d.income[1].income
    else {
        unreachable!()
    };
    let Some(DistributionCode::Early { exception: EarlyException::Applies { facts, .. }, .. }) =
        &mut data.code
    else {
        unreachable!()
    };
    *facts = RetirementException::Governmental457 { rollover_taxable: 0, eligible: false };
    // No: the exception is not applied, so the 10% tax applies to the whole payment.
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 900);

    let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
        &mut d.income[1].income
    else {
        unreachable!()
    };
    let Some(DistributionCode::Early { exception: EarlyException::Applies { facts, .. }, .. }) =
        &mut data.code
    else {
        unreachable!()
    };
    *facts = RetirementException::Governmental457 { rollover_taxable: 0, eligible: true };
    data.form.ira = true;
    unsupported(&d, "Income");
}

#[test]
fn undistributed_fund_credit_is_refundable_without_regular_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(undistributed_gain()));
    let r = supported(&d);
    assert_eq!(value(&r, "income_tax"), 0);
    assert_eq!(r.payments, Some(105000));
    assert_eq!(return_line(&r, "34"), 105000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
}

#[test]
fn cash_liquidation_gain_is_taxable_before_final_payment_and_zero_final_can_close_loss() {
    let mut d = complete(FederalStatus::Single);
    let mut distribution = capital_return(3000, 0, GainTerm::Short);
    let FederalIncome::ReturnOfCapital { kind, .. } = &mut distribution else { panic!() };
    *kind = CapitalDistributionKind::InterimLiquidation;
    add(&mut d, income(distribution));
    assert_eq!(value(&supported(&d), "income_tax"), 5383);
    let FederalIncome::ReturnOfCapital { kind, amount, basis, .. } = &mut d.income[1].income else {
        panic!()
    };
    *kind = CapitalDistributionKind::FinalLiquidation;
    *amount = 0;
    *basis = 100000;
    assert_eq!(value(&supported(&d), "capital_gain"), -1000);
}

#[test]
fn medical_care_and_employment_hardship_home_moves_use_reduced_limit() {
    for reason in [HomeSaleTrigger::MedicalCare, HomeSaleTrigger::EmploymentHardship] {
        let mut d = complete(FederalStatus::Single);
        let home = crate::MainHomeSale {
            form: crate::HomeSaleSource::Paper(crate::Form1099SPaper {
                filer: Default::default(),
                transferor: Default::default(),
                box_3_address_or_legal_description: "12 Synthetic Lane".into(),
                box_1_date_of_closing: 20260615,
                box_2a_total_gross_proceeds: 80_000_000,
                box_6_property_or_services: false,
                box_7_foreign_person: false,
                calendar_year: 2026,
                ..Default::default()
            }),
            answers: Some(crate::RealEstateSaleUse::MainHome(crate::HomeSaleAnswers {
                property_value: SourceCents::NotReported,
                firpta_withheld: SourceCents::NotReported,
                form_1099s: true,
                acquired: 20250615,
                // An inherited home preserves the long-term gain with 365 days of ownership.
                received: HomeReceived::Inherited,
                expenses: 5_000_000,
                payment: SalePayment::InFull,
                exchange: None,
                basis: 20_000_000,
                lived: HomeLived::WholeTime,
                earlier_sale: EarlierHomeSale::No,
                spouse: HomeSpouse::NotJoint,
                reason: HomeSaleReason::Event { reason },
                service: None,
                use_history: HomeRentalUse {
                    depreciation: 0,
                    nonqualified_days: 0,
                    separate: None,
                },
                more: HomeSaleMore::Nothing,
            })),
        };
        add(&mut d, income(FederalIncome::HomeSale { data: home }));
        assert_eq!(value(&supported(&d), "capital_gain"), 425000);
        assert_eq!(restart(d.clone()).draft, d);
        let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
        let HomeSaleReason::Event { reason } = &mut data.facts_mut().reason else { panic!() };
        *reason = HomeSaleTrigger::Other;
        unsupported(&d, "Income:");
    }
}
#[test]
fn section1244_liabilities_cannot_silently_create_a_taxable_exchange() {
    let stock = Section1244Stock {
        basis: StockLossBasis::Property {
            basis: 2000000,
            value: 1500000,
            liabilities: 2000001,
            acquired: 20200101,
        },
        acquired: 20260101,
        event: StockLossEvent::Worthless { confirmed: true },

        ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
    };
    assert!(!stock.valid());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::SmallBusinessStock { data: stock }));
    assert!(!d.valid_stored());
}
#[test]
fn single_life_table_boundaries_and_prior_recovery_use_original_age() {
    for (age, expected) in
        [(5, 766), (50, 331), (64, 208), (65, 200), (67, 184), (70, 160), (100, 27), (115, 5)]
    {
        assert_eq!(AnnuityDuration::life_multiple(age), Some(expected));
    }
    assert_eq!(AnnuityDuration::life_multiple(4), None);
    assert_eq!(AnnuityDuration::life_multiple(116), None);
    let mut a = fixed_annuity();
    a.started = 20100101;
    a.cost = 1080000;
    a.periodic = 10000;
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 0 };
    a.recovered = 1070000;
    assert!(a.valid(120000));
    assert_eq!(a.expected(), 2400000); // Original age, not current age 81.
    assert_eq!(a.excluded(), 10000);
}
#[test]
fn joint_lifetime_same_payments_and_age_order_use_last_survivor_table() {
    assert_eq!(crate::annuity_tables::joint_multiple(70, 67), Some(220));
    assert_eq!(crate::annuity_tables::joint_multiple(67, 70), Some(220));
    assert_eq!(crate::annuity_tables::joint_multiple(5, 5), Some(838));
    assert!(crate::annuity_tables::joint_multiple(115, 115).is_some());
    assert_eq!(crate::annuity_tables::joint_multiple(4, 70), None);
    assert_eq!(crate::annuity_tables::joint_multiple(70, 116), None);
    let mut a = joint_annuity(false);
    a.duration = AnnuityDuration::JointLife {
        delay: 1,
        age: 70,
        survivor_age: 67,
        survivor_payment: 50000,
        survivor: false,
    };
    assert_eq!(a.expected(), 13200000); // IRS identical-survivor-payment example.
    a.duration = AnnuityDuration::JointLife {
        delay: 1,
        age: 67,
        survivor_age: 70,
        survivor_payment: 50000,
        survivor: false,
    };
    assert_eq!(a.expected(), 13200000);
}
#[test]
fn variable_frequency_adjustments_and_fixed_installment_count() {
    let mut a = variable_annuity();
    for (frequency, count, delay, annual) in [
        (AnnuityFrequency::Quarterly, 4, 3, 60302),
        (AnnuityFrequency::Semiannual, 2, 6, 60606),
        (AnnuityFrequency::Monthly, 12, 1, 60000),
    ] {
        a.frequency = frequency;
        a.payments = count;
        a.history = VariableHistory::New { expected: VariablePayments::Life { age: 65, delay } };
        assert!(a.valid());
        assert_eq!(a.annual(), annual);
        a.history =
            VariableHistory::New { expected: VariablePayments::Fixed { payments: count * 10 } };
        assert!(a.valid());
        assert_eq!(a.annual(), 120000);
    }
    a.history = VariableHistory::New { expected: VariablePayments::Life { age: 65, delay: 2 } };
    assert!(!a.valid()); // Monthly timing cannot use the quarterly adjustment.
}
#[test]
fn variable_election_accumulates_unelected_full_year_shortfalls() {
    let mut a = variable_annuity();
    a.started = 20230101;
    a.history = VariableHistory::Prior {
        annual: 60000,
        recovered: 50000,
        refigure: Some(VariableRefigure {
            years: 2,
            paid: 50000,
            remaining: VariablePayments::Life { age: 68, delay: 6 },
        }),
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 63977); // ($600 * 2 - $500) / 17.6 => $39.77.
    assert_eq!(a.excluded(120000), 63977);
    a.history = VariableHistory::Prior {
        annual: 60000,
        recovered: 50000,
        refigure: Some(VariableRefigure {
            years: 4,
            paid: 50000,
            remaining: VariablePayments::Life { age: 68, delay: 6 },
        }),
    };
    assert!(!a.valid()); // Cannot claim four prior years since a 2023 start.
}

#[test]
fn variable_joint_life_uses_joint_multiple_and_payment_timing() {
    let mut a = variable_annuity();
    a.age = 70;
    a.cost = 13200000;
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 12;
    a.history = VariableHistory::New {
        expected: VariablePayments::Joint { age: 70, survivor_age: 67, delay: 1 },
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 600000); // Pub. 939 ages 70/67: 22.0 years.
    assert_eq!(a.excluded(900000), 600000);
    a.frequency = AnnuityFrequency::Annual;
    a.payments = 1;
    a.history = VariableHistory::New {
        expected: VariablePayments::Joint { age: 70, survivor_age: 67, delay: 12 },
    };
    assert!(a.valid());
    assert_eq!(a.annual(), 613953); // $132,000 / (22.0 - 0.5).
}

#[test]
fn joint_code_t_fully_taxable_after_phaseout_keeps_employee_earned_income() {
    let mut d = joint_employer_adoption_return();
    set_joint_adoption_w2(&mut d, 0, 30_000_000, 600_000);
    set_joint_adoption_w2(&mut d, 2, 100_000, 400_000);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300_000);
    d.credits.as_mut().unwrap().care = FederalCare::default();
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "adoption_employer_payment_1"), 6000);
    assert_eq!(value(&r, "adoption_employer_payment_2"), 2000);
    assert_eq!(value(&r, "adoption_employer_payment_3"), 4000);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 333_000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 12_000);
    assert_eq!(value(&r, "care_earned_spouse"), 5000); // $1,000 box 1 + this spouse's $4,000 code T.
    assert_eq!(value(&r, "care_credit"), 600);
    assert_eq!(return_line(&r, "1a"), 32_100_000);
    assert_eq!(return_line(&r, "1f"), 1_200_000);
    assert_eq!(return_line(&r, "25a"), 950_000);
    assert_eq!(value(&r, "agi"), 333_000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);

    let mut over_cap = d.clone();
    if let AdoptionBenefits::JointEmployees { payments, .. } =
        &mut over_cap.credits.as_mut().unwrap().adoptions[0].benefits
    {
        payments[0].received = 1_500_000;
    }
    set_joint_adoption_w2(&mut over_cap, 0, 30_000_000, 1_500_000);
    let over = supported(&over_cap);
    assert_eq!(value(&over, "adoption_benefits_excluded"), 0);
    assert_eq!(return_line(&over, "1f"), 2_100_000); // Even over-cap code T is wholly taxable after phaseout.
    assert_eq!(value(&over, "care_earned_spouse"), 5000);

    set_joint_adoption_w2(&mut d, 0, 27_208_000, 600_000);
    let boundary = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&boundary, "adoption_exclusion_magi"), 305_080);
    assert_eq!(value(&boundary, "adoption_benefits_excluded"), 0);
    set_joint_adoption_w2(&mut d, 0, 27_207_900, 600_000);
    let rounded_boundary = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&rounded_boundary, "adoption_exclusion_magi"), 305_079);
    assert_eq!(value(&rounded_boundary, "adoption_benefits_excluded"), 0); // Whole-dollar Form 8839 rounding.
    set_joint_adoption_w2(&mut d, 0, 26_000_000, 600_000);
    {
        document_care(&mut d);
        supported(&d)
    }; // A partly phased exclusion splits in proportion to the code T.
}

#[test]
fn macrs_table_convention_and_late_year_rounding_are_preserved() {
    let mut asset = continuing_equipment(20251001, 5000, true);
    asset.history = table_history(5000, 0, true);
    assert_eq!(asset.macrs_amount(false), 38000); // Table A-5, 5-year year 2.
    asset.placed = 20210601;
    asset.class = EquipmentClass::Seven;
    asset.history = table_history(77690, 0, false);
    assert_eq!(asset.macrs_amount(false), 8920); // Table A-1 year 6, not 8.93%.
    asset.placed = 20200601;
    asset.history = table_history(86610, 0, false);
    assert_eq!(asset.macrs_amount(false), 8930);
    asset.placed = 20190601;
    asset.history = table_history(95540, 0, false);
    assert_eq!(asset.macrs_amount(false), 4460);
    asset.history = table_history(100000, 0, false);
    assert_eq!(asset.macrs_amount(false), 0); // Never deduct recovered basis twice.
}

#[test]
fn new_macrs_table_choice_is_per_asset_and_matches_published_rounding() {
    let mut asset =
        macrs_equipment(10000, EquipmentClass::Seven, EquipmentMethod::Regular, 20260601);
    assert_eq!(asset.macrs_amount(false), 142857);
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::ElectOut,
        tables: true,
    };
    assert_eq!(asset.macrs_amount(false), 142900);
    let mut d = business_with_bonus(100000, asset);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(macrs_equipment(
        10000,
        EquipmentClass::Seven,
        EquipmentMethod::Regular,
        20260601,
    ));
    supported(&d); // Same tax elections; table/manual choice need not match across assets.
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn macrs_table_history_rejects_impossible_initial_bonus() {
    {
        let bonus = 52001;
        let mut asset = continuing_equipment(20250601, 52000, false);
        asset.history = table_history(52000, bonus, false);
        assert!(!business_with_bonus(100000, asset).valid_stored());
    }
}

#[test]
fn health_nondependent_child_matches_publication502_examples() {
    // Pub. 502 examples 1 and 2: $10,000 total, $4,000 available
    // earned-income limit, with and without an incremental child premium.
    for (increase, medical) in [(500000, 5000), (0, 6000)] {
        let months = vec![BusinessHealthMonth {
            month: 1,
            premiums: 1000000,
            child_increase: increase,
            employer_eligible: false,
            reimbursed: None,
        }];
        assert_eq!(crate::health::amounts(&months, 4000, 4000, 0, 0), (4000, medical));
    }
}

#[test]
fn health_child_increment_respects_employer_months_and_income_exhaustion() {
    let months = vec![
        BusinessHealthMonth {
            month: 1,
            premiums: 100000,
            child_increase: 60000,
            employer_eligible: true,
            reimbursed: None,
        },
        BusinessHealthMonth {
            month: 2,
            premiums: 100000,
            child_increase: 20000,
            employer_eligible: false,
            reimbursed: None,
        },
    ];
    assert_eq!(crate::health::amounts(&months, 10000, 10000, 0, 0), (1000, 400));
    assert_eq!(crate::health::amounts(&months, 500, 500, 0, 0), (500, 900));
    assert_eq!(crate::health::amounts(&months, 0, 0, 0, 0), (0, 1200));
    assert_eq!(crate::health::amounts(&months, 100, 100, 0, 0), (100, 1200));
}

#[test]
fn health_child_premiums_flow_to_return_once_and_roundtrip() {
    let mut d = empty_return(FederalStatus::Single);
    let mut business = health_business(5000, 10000, false);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    let BusinessHealthInsurance::Plan { months, .. } = &mut data.health_insurance else { panic!() };
    months[0].child_increase = 500000;
    add(&mut d, income(business));
    let r = supported(&d);
    let deduction = value(&r, "business_2_health_insurance");
    assert_eq!(deduction, 4646); // $5,000 profit minus $354 deductible SE tax.
    assert_eq!(value(&r, "business_2_medical_premiums"), 5000);
    assert_eq!(value(&r, "business_2_excluded_child_premiums"), 5000 - deduction);
    assert_eq!(restart(d.clone()).draft, d);
    {
        let invalid = 1000001;
        let FederalIncome::Business { data } = &mut d.income[0].income else { panic!() };
        let BusinessHealthInsurance::Plan { months, .. } = &mut data.health_insurance else {
            panic!()
        };
        months[0].child_increase = invalid;
        assert!(!d.valid_stored());
    }
}

#[test]
fn long_term_care_employer_exclusion_and_child_only_medical_rules() {
    for (medical, employer, deduction, itemized, excluded) in [
        (true, true, 0, 500, 500),
        (true, false, 200, 300, 500),
        (false, true, 0, 0, 1000),
        (false, false, 200, 0, 800),
    ] {
        let mut d = complete(FederalStatus::Single);
        set_age(&mut d, -1, 26);
        add(&mut d, income(care_business(200, 1000, employer)));
        if !medical {
            let person = health_coverage_child(&mut d, 26);
            let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
            let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance
            else {
                panic!()
            };
            long_term_care[0].person = person;
        }
        let r = supported(&d);
        // Schedule SE has no tax below $400 net earnings; the health limit is $200.
        assert_eq!(value(&r, "business_2_health_insurance"), deduction);
        assert_eq!(value(&r, "long_term_care_1_medical"), itemized);
        assert_eq!(value(&r, "long_term_care_1_excluded"), excluded);
    }
}

#[test]
fn long_term_care_does_not_double_age_limit_with_public_safety_premiums() {
    let mut d = complete(FederalStatus::Joint);
    set_age(&mut d, -1, 50);
    set_age(&mut d, -2, 50);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    let pension = d.income.last().unwrap().id;
    public_safety(&mut d, pension, 5000, 3000, PublicSafetyCoverage::LongTermCare { person: -1 });
    add(&mut d, income(care_business(50000, 1000, false)));
    let shared = supported(&d);
    assert_eq!(value(&shared, "business_3_long_term_care_deduction"), 930);
    assert_eq!(value(&shared, "long_term_care_1_medical"), 0);
    assert_eq!(value(&shared, "long_term_care_1_excluded"), 2070);
    // Another insured person has a limit of their own.
    let PublicSafetyCoverage::LongTermCare { person } =
        &mut d.screening.as_mut().unwrap().public_safety[0].coverage
    else {
        panic!()
    };
    *person = -2;
    let r = supported(&d);
    assert_eq!(value(&r, "public_safety_medical_before_coordination"), 930);
    assert_eq!(value(&r, "business_3_long_term_care_deduction"), 930);
}

#[test]
fn personal_long_term_care_uses_source_premiums_and_shared_age_caps() {
    let mut d = complete(FederalStatus::Joint);
    set_age(&mut d, -1, 50);
    set_age(&mut d, -2, 71);
    let expenses = itemized(d.deductions.as_mut().unwrap());
    expenses.medical = 2000000;
    expenses.long_term_care =
        vec![personal_care(-1, 80000), personal_care(-1, 80000), personal_care(-2, 700000)];
    let r = supported(&d);
    // The pools run in the people's identity order: your spouse (-2, age 71,
    // limit 6,200), then you (-1, age 50, limit 930).
    assert_eq!(value(&r, "long_term_care_1_medical"), 6200);
    assert_eq!(value(&r, "long_term_care_2_medical"), 930);
    let mut reference = d.clone();
    let expenses = itemized(reference.deductions.as_mut().unwrap());
    expenses.long_term_care.clear();
    expenses.medical += 713000;
    let expected = supported(&reference);
    assert_eq!(value(&r, "deduction"), value(&expected, "deduction"));
    assert_eq!(r.tax, expected.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn single_filer_archer_personal_excess_limits_deduction_and_excise_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, archer_self(4000), 3000);
    owner.excess_accounting = Some(archer_excess_accounting(30000, None, 10000, 0, 0));
    set_archer(&mut d, vec![owner], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_deduction"), 2600);
    assert_eq!(value(&r, "archer_return_year_uncorrected_personal_excess"), 400);
    assert_eq!(value(&r, "archer_return_year_excess_account_value"), 400);
    assert_eq!(value(&r, "archer_return_year_excess_contribution_tax"), 24);
    assert_eq!(value(&r, "agi"), 57400);
    assert_eq!(return_line(&r, "23"), 2400);
    assert_eq!(restart(d.clone()).draft, d);

    let ArcherContributions::Records { owners, .. } = &mut d.adjustments.as_mut().unwrap().archer
    else {
        panic!()
    };
    owners[0].excess_accounting.as_mut().unwrap().year_end_value = 10000;
    assert_eq!(value(&supported(&d), "archer_return_year_excess_contribution_tax"), 12);
    d.income[0] = wage(1, Owner::Taxpayer, 100000, 0);
    let limited_by_compensation = supported(&d);
    assert_eq!(value(&limited_by_compensation, "archer_msa_deduction"), 1000);
    assert_eq!(
        value(&limited_by_compensation, "archer_return_year_uncorrected_personal_excess"),
        2000
    );
    let ArcherContributions::Records { owners, .. } = &mut d.adjustments.as_mut().unwrap().archer
    else {
        panic!()
    };
    owners[0].excess_accounting.as_mut().unwrap().later_return_year_deposits = 300001;
    assert!(!d.adjustments.as_ref().unwrap().archer.valid());
}

#[test]
fn single_filer_archer_employer_excess_tracks_income_year_and_w2_inclusion() {
    let mut d = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, archer_self(4000), 0);
    owner.employer = 300000;
    owner.employer_outside = 300000;
    owner.excess_accounting = Some(archer_excess_accounting(30000, None, 0, 0, 0));
    set_archer(&mut d, vec![owner], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_return_year_uncorrected_employer_excess"), 400);
    assert_eq!(value(&r, "archer_return_year_uncorrected_employer_excess_other_income"), 400);
    assert_eq!(value(&r, "archer_return_year_excess_account_value"), 300);
    assert_eq!(value(&r, "archer_return_year_excess_contribution_tax"), 18);
    assert_eq!(value(&r, "agi"), 60400);
    assert_eq!(return_line(&r, "8"), 40000);
    assert_eq!(restart(d.clone()).draft, d);

    archer_owner_mut(&mut d).excess_accounting.as_mut().unwrap().employer_excess_in_wages = 40000;
    assert_eq!(
        value(&supported(&d), "archer_return_year_uncorrected_employer_excess_other_income"),
        0
    );
    archer_owner_mut(&mut d).excess_accounting.as_mut().unwrap().employer_excess_in_wages = 40001;
    unsupported(&d, "Archer MSA excess");
    archer_owner_mut(&mut d).excess_accounting.as_mut().unwrap().employer_excess_in_wages = 0;
    archer_owner_mut(&mut d).excess_accounting.as_mut().unwrap().later_return_year_deposits =
        100000;
    archer_owner_mut(&mut d).excess_accounting.as_mut().unwrap().later_employer_deposits = 100000;
    let later = supported(&d);
    assert_eq!(value(&later, "archer_return_year_uncorrected_employer_excess"), 400);
    assert_eq!(value(&later, "archer_return_year_uncorrected_employer_excess_other_income"), 0);
    assert_eq!(value(&later, "archer_return_year_excess_contribution_tax"), 24);
    assert_eq!(value(&later, "agi"), 60000);

    archer_owner_mut(&mut d).employer = 200000;
    archer_owner_mut(&mut d).employer_outside = 200000;
    archer_owner_mut(&mut d).personal = 10000;
    archer_owner_mut(&mut d).excess_accounting.as_mut().unwrap().later_return_year_deposits = 0;
    archer_owner_mut(&mut d).excess_accounting.as_mut().unwrap().later_employer_deposits = 0;
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "archer_return_year_uncorrected_personal_excess"), 100);
    assert_eq!(value(&mixed, "archer_return_year_uncorrected_employer_excess"), 0);
    assert_eq!(value(&mixed, "archer_msa_deduction"), 0);
    assert_eq!(value(&mixed, "archer_return_year_excess_contribution_tax"), 6);
}

#[test]
fn retained_archer_excess_uses_taxable_withdrawals_then_unused_contribution_room() {
    let mut d = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, archer_self(4000), 2400);
    owner.excess_accounting = Some(archer_excess_accounting(
        80000,
        Some(ArcherPriorExcess { line_40: 100000, line_41: 6000 }),
        0,
        0,
        0,
    ));
    set_archer(&mut d, vec![owner], ArcherSharing::Equal);
    add(&mut d, income(archer_msa(300)));
    let r = supported(&d);
    assert_eq!(value(&r, "archer_prior_excess_used_by_distributions"), 300);
    assert_eq!(value(&r, "archer_prior_excess_deducted_return_year"), 200);
    assert_eq!(value(&r, "archer_prior_excess_remaining_following_year"), 500);
    assert_eq!(value(&r, "archer_contribution_1_deduction"), 2600);
    assert_eq!(value(&r, "archer_msa_deduction"), 2600);
    assert_eq!(value(&r, "archer_return_year_excess_contribution_tax"), 30);
    assert_eq!(value(&r, "agi"), 57700);
    assert_eq!(return_line(&r, "23"), 9000);
    assert_eq!(restart(d.clone()).draft, d);

    let FederalIncome::MedicalDistribution { data } = &mut d.income.last_mut().unwrap().income
    else {
        panic!()
    };
    data.form1099sa_mut().unwrap().gross_distribution = 30000;
    if let Some(MedicalSaPayment::Own { medical, nonmedical, .. }) = &mut data.payment {
        *medical = 30000;
        nonmedical.clear();
    }
    let medical = supported(&d);
    assert_eq!(value(&medical, "archer_prior_excess_used_by_distributions"), 0);
    assert_eq!(value(&medical, "archer_prior_excess_deducted_return_year"), 200);
    assert_eq!(value(&medical, "archer_prior_excess_remaining_following_year"), 800);
    assert_eq!(value(&medical, "archer_return_year_excess_contribution_tax"), 48);
    assert_eq!(value(&medical, "archer_msa_taxable_distributions"), 0);
}

#[test]
fn timely_archer_corrections_remove_principal_and_report_income_by_actual_year() {
    let correction = ArcherCorrection {
        failed_deposit: None,
        document: None,
        year: 2026,
        contributed_on: 20260601,
        principal: 40000,
        earnings: 2000,
        returned: 20261231,
        deadline: 20270415,
        payer: ArcherCorrectionPayer::Personal,
    };
    let mut d = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, archer_self(4000), 3000);
    owner.corrections.push(correction.clone());
    set_archer(&mut d, vec![owner], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_1_corrected_contributions"), 400);
    assert_eq!(value(&r, "archer_msa_deduction"), 2600);
    assert_eq!(value(&r, "archer_correction_earnings"), 20);
    assert_eq!(value(&r, "agi"), 57420);
    assert_eq!(return_line(&r, "8"), 2000);
    assert_eq!(
        d.adjustments.as_ref().unwrap().archer.deposits()[0],
        crate::kernel::ratio::exact::whole(260000)
    );
    assert_eq!(restart(d.clone()).draft, d);

    archer_owner_mut(&mut d).corrections[0].returned = 20270416;
    unsupported(&d, "Archer MSA correction");
    archer_owner_mut(&mut d).corrections[0].returned = 20261231;
    archer_owner_mut(&mut d).corrections[0].principal = 300001;
    assert!(!d.adjustments.as_ref().unwrap().archer.valid());

    let mut nonexcess = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, archer_self(4000), 1000);
    owner.corrections.push(correction.clone());
    set_archer(&mut nonexcess, vec![owner], ArcherSharing::Equal);
    unsupported(&nonexcess, "Archer MSA correction");

    let mut employer = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, archer_self(4000), 0);
    owner.employer = 300000;
    owner.employer_outside = 300000;
    owner.corrections.push(ArcherCorrection {
        failed_deposit: None,
        document: None,
        payer: ArcherCorrectionPayer::Employer { in_wages: false },
        ..correction
    });
    set_archer(&mut employer, vec![owner], ArcherSharing::Equal);
    let r = supported(&employer);
    assert_eq!(value(&r, "archer_correction_employer_principal_other_income"), 400);
    assert_eq!(value(&r, "archer_correction_earnings"), 20);
    assert_eq!(value(&r, "agi"), 60420);
    assert_eq!(return_line(&r, "8"), 42000);
    assert_eq!(
        employer.adjustments.as_ref().unwrap().archer.deposits()[0],
        crate::kernel::ratio::exact::whole(260000)
    );
    archer_owner_mut(&mut employer).corrections[0].returned = 20270301;
    assert_eq!(value(&supported(&employer), "archer_correction_earnings"), 0);
    assert_eq!(
        value(&supported(&employer), "archer_correction_employer_principal_other_income"),
        400
    );
    archer_owner_mut(&mut employer).corrections[0].contributed_on = 20270115;
    assert_eq!(
        value(&supported(&employer), "archer_correction_employer_principal_other_income"),
        0
    );
}

#[test]
fn prior_designated_archer_correction_keeps_current_contributions_untouched() {
    let mut d = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, ArcherCoverage::None, 0);
    owner.income.clear();
    owner.corrections.push(ArcherCorrection {
        failed_deposit: None,
        document: None,
        year: 2025,
        contributed_on: 20260415,
        principal: 100000,
        earnings: 5000,
        returned: 20260601,
        deadline: 20261015,
        payer: ArcherCorrectionPayer::Employer { in_wages: false },
    });
    set_archer(&mut d, vec![owner], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_1_prior_designated_corrected_contributions"), 1000);
    assert_eq!(value(&r, "archer_correction_employer_principal_other_income"), 1000);
    assert_eq!(value(&r, "archer_correction_earnings"), 50);
    assert_eq!(value(&r, "archer_msa_deduction"), 0);
    assert_eq!(value(&r, "agi"), 61050);
    assert_eq!(return_line(&r, "8"), 105000);
    assert_eq!(
        d.adjustments.as_ref().unwrap().archer.deposits()[0],
        crate::kernel::ratio::exact::whole(0)
    );
    assert_eq!(restart(d.clone()).draft, d);

    archer_owner_mut(&mut d).corrections[0].contributed_on = 20250415;
    assert_eq!(value(&supported(&d), "archer_correction_employer_principal_other_income"), 0);
    assert_eq!(value(&supported(&d), "archer_correction_earnings"), 50);
}

#[test]
fn prior_designated_uncorrected_archer_employer_excess_uses_deposit_year_income() {
    let mut d = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, ArcherCoverage::None, 0);
    owner.income.clear();
    owner.prior_employer_excess.push(ArcherPriorEmployerExcessDeposit {
        contributed_on: 20260415,
        deadline: 20260415,
        amount: 100000,
        in_wages: false,
    });
    owner.excess_accounting = Some(archer_excess_accounting(
        100000,
        Some(ArcherPriorExcess { line_40: 100000, line_41: 6000 }),
        0,
        0,
        0,
    ));
    set_archer(&mut d, vec![owner], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_1_prior_designated_employer_excess"), 1000);
    assert_eq!(value(&r, "archer_prior_designated_employer_excess_other_income"), 1000);
    assert_eq!(value(&r, "archer_return_year_excess_contribution_tax"), 60);
    assert_eq!(value(&r, "agi"), 61000);
    assert_eq!(return_line(&r, "8"), 100000);
    assert_eq!(
        d.adjustments.as_ref().unwrap().archer.deposits()[0],
        crate::kernel::ratio::exact::whole(0)
    );
    assert_eq!(restart(d.clone()).draft, d);

    archer_owner_mut(&mut d).prior_employer_excess[0].amount = 100001;
    unsupported(&d, "Archer prior employer excess");
    archer_owner_mut(&mut d).prior_employer_excess[0].amount = 100000;
    archer_owner_mut(&mut d).prior_employer_excess[0].in_wages = true;
    d.income[0] = wage(1, Owner::Taxpayer, 6100000, 0);
    let in_wages = supported(&d);
    assert_eq!(value(&in_wages, "archer_prior_designated_employer_excess_other_income"), 0);
    assert_eq!(value(&in_wages, "agi"), 61000);
}

#[test]
fn married_archer_excess_uses_each_owners_form_5329_and_account_value() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 4000000, 0));
    let mut first = archer_contributor(Owner::Taxpayer, archer_family(6000), 2500);
    let mut second = archer_contributor(Owner::Spouse, archer_family(6000), 2500);
    first.excess_accounting = Some(archer_excess_accounting(10000, None, 0, 0, 0));
    second.excess_accounting = Some(archer_excess_accounting(50000, None, 0, 0, 0));
    set_archer(&mut d, vec![first, second], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_1_excess_contribution_tax"), 6);
    assert_eq!(value(&r, "archer_2_excess_contribution_tax"), 15);
    assert_eq!(value(&r, "archer_return_year_excess_contribution_tax"), 21);
    for (id, number, amount) in
        [("archer_1_excess_line41", "41", 6), ("archer_2_excess_line41", "41", 15)]
    {
        assert_eq!(value(&r, id), amount);
        assert_eq!(printed_line(&r, id), (TaxForm::Form5329, number.into()));
    }
    assert_eq!(value(&r, "archer_return_year_uncorrected_personal_excess"), 500);
    assert_eq!(value(&r, "archer_msa_deduction"), 4500);
    assert_eq!(value(&r, "agi"), 135500);
    assert_eq!(return_line(&r, "23"), 2100);
    assert_eq!(restart(d.clone()).draft, d);

    let ArcherContributions::Records { owners, .. } = &mut d.adjustments.as_mut().unwrap().archer
    else {
        panic!()
    };
    owners[0].personal = 200000;
    owners[0].excess_accounting.as_mut().unwrap().prior_year =
        Some(ArcherPriorExcess { line_40: 100000, line_41: 6000 });
    let mut spouse_withdrawal = income(archer_msa(300));
    spouse_withdrawal.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse_withdrawal);
    let owner_specific = supported(&d);
    assert_eq!(value(&owner_specific, "archer_prior_excess_used_by_distributions"), 0);
    assert_eq!(value(&owner_specific, "archer_prior_excess_deducted_return_year"), 250);
    assert_eq!(value(&owner_specific, "archer_prior_excess_remaining_following_year"), 750);

    let mut separate = complete(FederalStatus::Separate);
    let mut taxpayer = archer_contributor(Owner::Taxpayer, archer_family(6000), 2500);
    taxpayer.excess_accounting = Some(archer_excess_accounting(20000, None, 0, 0, 0));
    let mut spouse = archer_contributor(Owner::Spouse, archer_family(6000), 0);
    spouse.income.clear();
    set_archer(&mut separate, vec![taxpayer, spouse], ArcherSharing::Equal);
    let r = supported(&separate);
    assert_eq!(value(&r, "archer_1_excess_contribution_tax"), 12);
    assert_eq!(value(&r, "archer_return_year_excess_contribution_tax"), 12);
    assert_eq!(value(&r, "archer_msa_deduction"), 2250);
}

// Audit D2: section 220(b)(5) divides the family limit between married
// spouses whether or not the spouse is eligible; a separate filer's Form 8853
// line 3 is 37.5% of the deductible (i8853), not 75%.
#[test]
fn separate_filer_with_family_coverage_takes_half_the_limit_when_the_spouse_is_not_eligible() {
    let mut d = complete(FederalStatus::Separate);
    let mut taxpayer = archer_contributor(Owner::Taxpayer, archer_family(6000), 4500);
    taxpayer.excess_accounting = Some(archer_excess_accounting(20000, None, 0, 0, 0));
    let mut spouse = archer_contributor(Owner::Spouse, ArcherCoverage::None, 0);
    spouse.income.clear();
    set_archer(&mut d, vec![taxpayer.clone(), spouse.clone()], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_contribution_1_coverage_limit"), 2250);
    assert_eq!(value(&r, "archer_msa_deduction"), 2250);
    assert_eq!(value(&r, "archer_return_year_uncorrected_personal_excess"), 2250);
    assert_eq!(value(&r, "agi"), 57750);
    // An agreed division is honored the same way.
    set_archer(&mut d, vec![taxpayer, spouse], ArcherSharing::Agreed { taxpayer: 70.0 });
    assert_eq!(value(&supported(&d), "archer_contribution_1_coverage_limit"), 3150);
}

#[test]
fn archer_contribution_months_keep_fractional_limits_until_cents_boundary() {
    let mut d = complete(FederalStatus::Single);
    let mut record = archer_contributor(Owner::Taxpayer, archer_self(3000), 975);
    record.coverage = ArcherEligibility::Monthly {
        months: (0..12)
            .map(|m| if m < 6 { archer_self(3000) } else { ArcherCoverage::None })
            .collect(),
    };
    set_archer(&mut d, vec![record.clone()], ArcherSharing::Equal);
    assert_eq!(value(&supported(&d), "archer_contribution_1_coverage_limit"), 975);
    record.personal += 1;
    set_archer(&mut d, vec![record], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
}

#[test]
fn archer_compensation_cap_and_employer_exclusion_are_independent_of_deduction() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100000, 0);
    set_archer(
        &mut d,
        vec![archer_contributor(Owner::Taxpayer, archer_self(3000), 1001)],
        ArcherSharing::Equal,
    );
    unsupported(&d, "Archer MSA");
    let mut record = archer_contributor(Owner::Taxpayer, archer_self(3000), 0);
    record.employer = 100000;
    record.employer_outside = 100000;
    set_archer(&mut d, vec![record.clone()], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_deduction"), 0);
    assert_eq!(value(&r, "agi"), 1000);
    record.personal = 1;
    set_archer(&mut d, vec![record], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
}

#[test]
fn archer_family_uses_lower_deductible_and_agreed_allocation() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 4000000, 0));
    let mut first = archer_contributor(Owner::Taxpayer, archer_family(6000), 2250);
    let mut second = archer_contributor(Owner::Spouse, archer_family(8000), 2250);
    set_archer(&mut d, vec![first.clone(), second.clone()], ArcherSharing::Equal);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_contribution_1_coverage_limit"), 2250);
    assert_eq!(value(&r, "archer_contribution_2_coverage_limit"), 2250);
    assert_eq!(value(&r, "archer_msa_deduction"), 4500);
    first.personal = 315000;
    second.personal = 135000;
    set_archer(
        &mut d,
        vec![first.clone(), second.clone()],
        ArcherSharing::Agreed { taxpayer: 70.0 },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "archer_contribution_1_coverage_limit"), 3150);
    assert_eq!(value(&r, "archer_contribution_2_coverage_limit"), 1350);
    set_archer(&mut d, vec![first, second], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
}

#[test]
fn archer_deposits_reduce_hsa_capacity_even_when_employer_funded() {
    let mut d = complete(FederalStatus::Single);
    let mut record = archer_contributor(Owner::Taxpayer, archer_self(3000), 1950);
    set_archer(&mut d, vec![record.clone()], ArcherSharing::Equal);
    d.adjustments.as_mut().unwrap().hsa.push(hsa_entry(
        Owner::Taxpayer,
        HsaCoverage::SelfOnly,
        245000,
    ));
    assert_eq!(value(&supported(&d), "agi"), 55600);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
    d.adjustments.as_mut().unwrap().hsa[0].personal -= 1;
    record.personal = 0;
    record.employer = 195000;
    record.employer_outside = 195000;
    set_archer(&mut d, vec![record], ArcherSharing::Equal);
    assert_eq!(value(&supported(&d), "agi"), 57550);
}

#[test]
fn archer_family_deposits_reduce_shared_hsa_base_preserving_owner_catchup() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19660615;
    add(&mut d, wage(2, Owner::Spouse, 4000000, 0));
    set_archer(
        &mut d,
        vec![
            archer_contributor(Owner::Taxpayer, archer_family(6000), 2250),
            archer_contributor(Owner::Spouse, archer_family(6000), 2250),
        ],
        ArcherSharing::Equal,
    );
    d.adjustments.as_mut().unwrap().hsa.push(hsa_entry(
        Owner::Taxpayer,
        HsaCoverage::Family,
        525000,
    ));
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 0,
        other_coverage: Some(HsaEligibility::FullYear { coverage: HsaCoverage::Family }),
    });
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 130250); // $140,000 - $4,500 MSA - $5,250 HSA.
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn archer_self_employment_compensation_includes_linked_losses_and_deductible_se_tax() {
    let mut d = empty_return(FederalStatus::Single);
    d.next_id = 1;
    add(&mut d, income(cash_business(10000)));
    add(&mut d, income(cash_business(-9000)));
    let mut record = archer_contributor(Owner::Taxpayer, archer_self(3000), 900);
    record.income = vec![1, 2];
    set_archer(&mut d, vec![record.clone()], ArcherSharing::Equal);
    let r = supported(&d);
    let (_, _, half) = crate::federal_rules::self_employment(100000, 0, 0);
    assert_eq!(value(&r, "archer_contribution_1_compensation"), 1000 - half);
    assert_eq!(value(&r, "archer_msa_deduction"), 900);
    record.personal = (1000 - half) * 100 + 1;
    set_archer(&mut d, vec![record], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
}

#[test]
fn archer_contribution_unknown_duplicate_and_wrong_income_owners_fail() {
    let mut d = complete(FederalStatus::Single);
    let record = archer_contributor(Owner::Taxpayer, archer_self(3000), 1000);
    set_archer(&mut d, vec![record.clone(), record.clone()], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
    let mut duplicate = record.clone();
    duplicate.income = vec![1, 1];
    set_archer(&mut d, vec![duplicate], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
    let mut missing = record.clone();
    missing.income = vec![999];
    set_archer(&mut d, vec![missing], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
    set_archer(&mut d, vec![record], ArcherSharing::Equal);
}

#[test]
fn archer_personal_deduction_flows_like_hsa_without_changing_earned_income() {
    let mut d = complete(FederalStatus::Single);
    set_archer(
        &mut d,
        vec![archer_contributor(Owner::Taxpayer, archer_self(3000), 1000)],
        ArcherSharing::Equal,
    );
    let r = supported(&d);
    // Schedule 1 line 23 is shown, and it is all of line 26.
    assert_eq!(schedule_1_line(&r, "23"), 1000);
    assert_eq!(schedule_1_line(&r, "26"), 1000);
    let mut reference = d.clone();
    reference.adjustments.as_mut().unwrap().archer = ArcherContributions::None;
    reference.adjustments.as_mut().unwrap().hsa.push(hsa_entry(
        Owner::Taxpayer,
        HsaCoverage::SelfOnly,
        100000,
    ));
    let expected = supported(&reference);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(r.refund, expected.refund);
    for id in [
        "agi",
        "adjustments",
        "deduction",
        "qbi_deduction",
        "earned_income_credit",
        "self_employment_tax",
    ] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
}

#[test]
fn archer_employer_funding_cannot_use_unrelated_business_compensation() {
    let mut d = empty_return(FederalStatus::Single);
    d.next_id = 1;
    add(&mut d, income(cash_business(10000)));
    let mut record = archer_contributor(Owner::Taxpayer, archer_self(3000), 0);
    record.employer = 100000;
    record.employer_outside = 100000;
    set_archer(&mut d, vec![record], ArcherSharing::Equal);
    unsupported(&d, "Archer MSA");
}

#[test]
fn hsa_deemed_termination_precedes_later_age_and_disability_exceptions() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19610615;
    add_hsa_deemed(
        &mut d,
        Owner::Taxpayer,
        MedicalDeemed::Terminated { occurred: 20261231, january_value: 1000000 },
        None,
    );
    assert_eq!(value(&supported(&d), "hsa_additional_tax"), 2000);
    for (date, expected) in [(Some(20260615), 2000), (Some(20251231), 0), (None, 2000)] {
        let Some(MedicalSaPayment::Unreported { disabled_since, .. }) =
            &mut hsa_distribution_mut(&mut d, Owner::Taxpayer, 0).payment
        else {
            panic!()
        };
        *disabled_since = date;
        assert_eq!(value(&supported(&d), "hsa_additional_tax"), expected);
    }
    d.people.as_mut().unwrap().taxpayer.birth_date = 19601231;
    assert_eq!(value(&supported(&d), "hsa_additional_tax"), 0);
}

#[test]
fn hsa_pledged_assets_age_and_disability_exempt_only_after_boundary_date() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19610615;
    add_hsa_deemed(
        &mut d,
        Owner::Taxpayer,
        MedicalDeemed::Pledged {
            assets: vec![
                MedicalAccountWithdrawal { date: 20260614, amount: 100000 },
                MedicalAccountWithdrawal { date: 20260615, amount: 200000 },
                MedicalAccountWithdrawal { date: 20260616, amount: 300000 },
            ],
        },
        None,
    );
    // A real medical distribution from an unaffected account cannot offset a pledge.
    add_hsa_distribution(&mut d, Owner::Taxpayer, 1000000, vec![], None, None);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 6000);
    assert_eq!(value(&r, "hsa_additional_tax"), 600);
    assert_eq!(value(&r, "hsa_1_deemed"), 6000);
    assert_eq!(return_line(&r, "8"), 600000);
    assert_eq!(return_line(&r, "23"), 60000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800615;
    let Some(MedicalSaPayment::Unreported { disabled_since, .. }) =
        &mut hsa_distribution_mut(&mut d, Owner::Taxpayer, 0).payment
    else {
        panic!()
    };
    *disabled_since = Some(20260615);
    assert_eq!(value(&supported(&d), "hsa_additional_tax"), 600);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn hsa_deemed_rejects_unknown_source_and_postdeath_activity() {
    let mut d = complete(FederalStatus::Single);
    add_hsa_deemed(
        &mut d,
        Owner::Taxpayer,
        MedicalDeemed::Terminated { occurred: 20261231, january_value: 500000 },
        Some(19000101),
    );
    unsupported(&d, "Income");
    let Some(MedicalSaPayment::Unreported { disabled_since, .. }) =
        &mut hsa_distribution_mut(&mut d, Owner::Taxpayer, 0).payment
    else {
        panic!()
    };
    *disabled_since = None;
    let mut final_return = death(20260615).unwrap();
    final_return.ordinary = true;
    d.people.as_mut().unwrap().taxpayer.died = Some(final_return);
    unsupported(&d, "Income");
    d.income.retain(|e| !matches!(e.income, FederalIncome::MedicalDistribution { .. }));
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260616, amount: 100000 }],
        None,
        None,
    );
    unsupported(&d, "Income");
}

#[test]
fn hsa_prior_funding_recapture_uses_month_end_not_transfer_anniversary() {
    for (loss, expected) in [(20260616, 4000), (20260630, 4000), (20260701, 0), (20251201, 0)] {
        let mut d = complete(FederalStatus::Single);
        let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
        h.funding_history = vec![hsa_funding(20250617, loss, 400000)];
        d.adjustments.as_mut().unwrap().hsa = vec![h];
        let r = supported(&d);
        assert_eq!(value(&r, "hsa_funding_recapture"), expected);
        assert_eq!(value(&r, "hsa_testing_period_tax"), expected / 10);
        assert_eq!(value(&r, "hsa_taxable_distributions"), 0);
        assert_eq!(value(&r, "hsa_additional_tax"), 0);
        assert_eq!(return_line(&r, "8"), expected * 100);
        assert_eq!(return_line(&r, "23"), expected * 10);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn hsa_funding_recapture_death_disability_exempt_but_age_does_not() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.funding_history = vec![hsa_funding(20251231, 20260615, 600000)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    assert_eq!(value(&supported(&d), "hsa_testing_period_tax"), 600);
    d.adjustments.as_mut().unwrap().hsa[0].funding_history[0].loss.as_mut().unwrap().reason =
        HsaLossReason::Disability;
    assert_eq!(value(&supported(&d), "hsa_funding_recapture"), 0);
    d.adjustments.as_mut().unwrap().hsa[0].funding_history[0].loss.as_mut().unwrap().reason =
        HsaLossReason::Death;
    unsupported(&d, "HSA");
    let mut final_return = death(20260615).unwrap();
    final_return.ordinary = true;
    d.people.as_mut().unwrap().taxpayer.died = Some(final_return);
    assert_eq!(value(&supported(&d), "hsa_funding_recapture"), 0);
}

#[test]
fn timely_hsa_correction_reduces_deduction_and_taxable_earnings_have_no_withdrawal_penalty() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 500000);
    h.corrections = vec![hsa_correction(2026, 20261231, 60000, 5000)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_1_corrected_contributions"), 600);
    assert_eq!(value(&r, "hsa_correction_earnings"), 50);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    assert_eq!(return_line(&r, "8"), 5000);
    assert_eq!(return_line(&r, "23"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn hsa_correction_contribution_year_and_earnings_year_are_independent() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 500000);
    h.corrections = vec![
        hsa_correction(2026, 20270415, 60000, 5000),
        hsa_correction(2025, 20260415, 100000, 10000),
    ];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_correction_earnings"), 100);
    assert_eq!(return_line(&r, "8"), 10000);
}

#[test]
fn hsa_correction_loss_reduces_cash_not_principal_removed() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 500000);
    h.corrections = vec![hsa_correction(2026, 20261231, 60000, -10000)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_correction_earnings"), 0);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.corrections[0].earnings = -60001;
    assert!(!h.valid());
}

#[test]
fn corrected_employer_hsa_principal_already_in_wages_is_not_added_twice() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.employer = 100000;
    h.employer_outside = 100000;
    let mut c = hsa_correction(2026, 20261231, 100000, 5000);
    c.payer = HsaCorrectionPayer::Employer { in_wages: true };
    h.corrections = vec![c];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "agi"), 60050);
    d.adjustments.as_mut().unwrap().hsa[0].corrections[0].payer =
        HsaCorrectionPayer::Employer { in_wages: false };
    let other = supported(&d);
    assert_eq!(value(&other, "hsa_correction_employer_principal_other_income"), 1000);
    assert_eq!(value(&other, "hsa_correction_earnings"), 50);
    assert_eq!(value(&other, "adjustments"), 0);
    assert_eq!(value(&other, "agi"), 61050);
    assert_eq!(return_line(&other, "8"), 105000);
    assert_eq!(return_line(&other, "23"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn returned_next_year_employer_hsa_principal_still_enters_2026_other_income() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.employer = 100000;
    h.employer_outside = 100000;
    let mut c = hsa_correction(2026, 20270415, 100000, 5000);
    c.payer = HsaCorrectionPayer::Employer { in_wages: false };
    h.corrections = vec![c];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_correction_employer_principal_other_income"), 1000);
    assert_eq!(value(&r, "hsa_correction_earnings"), 0);
    assert_eq!(value(&r, "agi"), 61000);
    assert_eq!(return_line(&r, "8"), 100000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn employer_hsa_correction_uses_actual_deposit_year_for_income() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.employer = 100000;
    h.employer_outside = 100000;
    let mut c = hsa_correction(2026, 20270415, 100000, 5000);
    c.contributed_on = 20270115;
    c.payer = HsaCorrectionPayer::Employer { in_wages: false };
    h.corrections = vec![c];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let later = supported(&d);
    assert_eq!(value(&later, "hsa_correction_employer_principal_other_income"), 0);
    assert_eq!(value(&later, "hsa_correction_earnings"), 0);
    assert_eq!(value(&later, "agi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);

    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.employer = 0;
    h.employer_outside = 0;
    h.corrections[0].year = 2025;
    h.corrections[0].contributed_on = 20260115;
    h.corrections[0].returned = 20260415;
    h.corrections[0].deadline = 20260415;
    let prior_designation = supported(&d);
    assert_eq!(value(&prior_designation, "hsa_correction_employer_principal_other_income"), 1000);
    assert_eq!(value(&prior_designation, "hsa_correction_earnings"), 50);
    assert_eq!(value(&prior_designation, "agi"), 61050);
}

#[test]
fn uncorrected_personal_hsa_excess_reduces_deduction_and_creates_f5329_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 500000);
    h.excess_accounting = Some(hsa_excess_accounting(1000000, None, 0, 0, 0));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 600);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 36);
    assert_eq!(return_line(&r, "23"), 3600);
    assert_eq!(restart(d.clone()).draft, d);

    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.excess_accounting.as_mut().unwrap().year_end_value = 30000;
    let limited = supported(&d);
    assert_eq!(value(&limited, "hsa_1_return_year_uncorrected_excess"), 600);
    assert_eq!(value(&limited, "hsa_return_year_excess_contribution_tax"), 18);
    assert_eq!(return_line(&limited, "23"), 1800);
}

#[test]
fn personal_hsa_excess_uses_room_after_employer_and_later_designated_deposits() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 400000);
    h.employer = 100000;
    h.employer_outside = 100000;
    h.excess_accounting = Some(hsa_excess_accounting(0, None, 10000, 0, 0));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 3400);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 600);
    assert_eq!(value(&r, "hsa_1_return_year_excess_account_value"), 100);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 6);
    assert_eq!(return_line(&r, "23"), 600);

    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.employer = 450000;
    h.employer_outside = 450000;
    let combined = supported(&d);
    assert_eq!(value(&combined, "hsa_1_return_year_uncorrected_employer_excess"), 100);
    assert_eq!(value(&combined, "hsa_1_return_year_uncorrected_excess"), 4000);
    assert_eq!(value(&combined, "hsa_return_year_excess_contribution_tax"), 6);
}

#[test]
fn one_month_hsa_limit_rounds_fractional_cent_before_testing_excess() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 36667);
    let mut months = vec![HsaCoverage::None; 12];
    months[0] = HsaCoverage::SelfOnly;
    h.coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    assert_eq!(value(&supported(&d), "adjustments"), 367);

    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.personal = 50000;
    h.excess_accounting = Some(hsa_excess_accounting(50000, None, 0, 0, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 367);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 133);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 8);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn retained_hsa_excess_uses_2026_contribution_room_for_a_new_deduction() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 300000);
    h.excess_accounting = Some(hsa_excess_accounting(
        900000,
        Some(HsaPriorExcess { line_48: 100000, line_49: 6000 }),
        0,
        0,
        0,
    ));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4000);
    assert_eq!(value(&r, "hsa_1_prior_excess_deducted_return_year"), 1000);
    assert_eq!(value(&r, "hsa_1_prior_excess_remaining_following_year"), 0);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn retained_hsa_excess_distributions_then_unused_room_limit_2026_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 400000);
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260701, amount: 30000 }],
        None,
        None,
    );
    h.excess_accounting = Some(hsa_excess_accounting(
        1000000,
        Some(HsaPriorExcess { line_48: 200000, line_49: 12000 }),
        0,
        0,
        0,
    ));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_excess_used_by_distributions"), 300);
    assert_eq!(value(&r, "hsa_1_prior_excess_deducted_return_year"), 400);
    assert_eq!(value(&r, "hsa_1_prior_excess_remaining_following_year"), 1300);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_additional_tax"), 60);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 78);
    assert_eq!(return_line(&r, "23"), 13800);
    assert_eq!(restart(d.clone()).draft, d);

    d.adjustments.as_mut().unwrap().hsa[0].personal = 500000;
    let combined = supported(&d);
    assert_eq!(value(&combined, "hsa_1_prior_excess_deducted_return_year"), 0);
    assert_eq!(value(&combined, "hsa_1_prior_excess_remaining_following_year"), 1700);
    assert_eq!(value(&combined, "hsa_1_return_year_uncorrected_excess"), 600);
    assert_eq!(value(&combined, "hsa_return_year_excess_contribution_tax"), 138);
}

#[test]
fn late_prior_hsa_excess_withdrawal_is_taxable_even_when_used_for_medical_care() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 400000);
    h.late_excess.push(HsaLateExcessWithdrawal {
        contribution_year: 2025,
        deadline: 20260415,
        date: 20260601,
        amount: 30000,
        disabled: false,
    });
    h.excess_accounting = Some(hsa_excess_accounting(
        500000,
        Some(HsaPriorExcess { line_48: 50000, line_49: 3000 }),
        0,
        0,
        0,
    ));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_late_excess_withdrawals"), 300);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 300);
    assert_eq!(value(&r, "hsa_additional_tax"), 60);
    assert_eq!(value(&r, "hsa_1_prior_excess_used_by_distributions"), 300);
    assert_eq!(value(&r, "hsa_1_prior_excess_deducted_return_year"), 200);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 0);
    assert_eq!(value(&r, "adjustments"), 4200);
    assert_eq!(restart(d.clone()).draft, d);

    let withdrawal = &mut d.adjustments.as_mut().unwrap().hsa[0].late_excess[0];
    withdrawal.disabled = true;
    assert_eq!(value(&supported(&d), "hsa_additional_tax"), 0);
    // Not disabled: the 20% additional tax applies to the late withdrawal.
    let withdrawal = &mut d.adjustments.as_mut().unwrap().hsa[0].late_excess[0];
    withdrawal.disabled = false;
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_additional_tax"), 60);
    let withdrawal = &mut d.adjustments.as_mut().unwrap().hsa[0].late_excess[0];
    withdrawal.deadline = 20260601;
    assert!(!d.adjustments.as_ref().unwrap().hsa[0].valid());
}

#[test]
fn later_employer_hsa_excess_affects_2026_limit_but_waits_for_2027_income() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    h.employer = 500000;
    h.employer_outside = 500000;
    h.excess_accounting = Some(hsa_excess_accounting(1000000, None, 100000, 100000, 0));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_employer_excess"), 600);
    assert_eq!(value(&r, "hsa_1_later_employer_excess_income"), 600);
    assert_eq!(value(&r, "hsa_return_year_uncorrected_employer_excess_other_income"), 0);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 36);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);

    d.adjustments.as_mut().unwrap().hsa[0].employer = 600000;
    d.adjustments.as_mut().unwrap().hsa[0].employer_outside = 600000;
    let split = supported(&d);
    assert_eq!(value(&split, "hsa_1_return_year_uncorrected_employer_excess"), 1600);
    assert_eq!(value(&split, "hsa_1_later_employer_excess_income"), 1000);
    assert_eq!(value(&split, "hsa_return_year_uncorrected_employer_excess_other_income"), 600);
    assert_eq!(value(&split, "hsa_return_year_excess_contribution_tax"), 96);
    assert_eq!(value(&split, "agi"), 60600);

    d.adjustments.as_mut().unwrap().hsa[0]
        .excess_accounting
        .as_mut()
        .unwrap()
        .employer_excess_in_wages = 60100;
    unsupported(&d, "HSA excess");
}

#[test]
fn uncorrected_prior_designated_employer_hsa_excess_uses_actual_deposit_year_income() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 440000);
    h.prior_employer_excess.push(HsaPriorEmployerExcessDeposit {
        contributed_on: 20260315,
        deadline: 20260415,
        amount: 60000,
        in_wages: false,
    });
    h.excess_accounting = Some(hsa_excess_accounting(
        500000,
        Some(HsaPriorExcess { line_48: 60000, line_49: 3600 }),
        0,
        0,
        0,
    ));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_1_prior_designated_employer_excess_income"), 600);
    assert_eq!(value(&r, "hsa_return_year_uncorrected_employer_excess_other_income"), 600);
    assert_eq!(value(&r, "hsa_1_prior_excess_remaining_following_year"), 600);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 36);
    assert_eq!(value(&r, "agi"), 56200);
    assert_eq!(restart(d.clone()).draft, d);

    let deposit = &mut d.adjustments.as_mut().unwrap().hsa[0].prior_employer_excess[0];
    deposit.in_wages = true;
    d.income[0] = wage(1, Owner::Taxpayer, 6060000, 0);
    let wages = supported(&d);
    assert_eq!(value(&wages, "hsa_return_year_uncorrected_employer_excess_other_income"), 0);
    assert_eq!(value(&wages, "agi"), 56200);
    let deposit = &mut d.adjustments.as_mut().unwrap().hsa[0].prior_employer_excess[0];
    deposit.in_wages = false;
    deposit.amount = 60001;
    unsupported(&d, "HSA prior employer excess");
    let deposit = &mut d.adjustments.as_mut().unwrap().hsa[0].prior_employer_excess[0];
    deposit.amount = 60000;
    deposit.contributed_on = 20260416;
    assert!(!d.adjustments.as_ref().unwrap().hsa[0].valid());
}

#[test]
fn prior_designated_employer_hsa_income_does_not_require_prior_excise_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    h.prior_employer_excess.push(HsaPriorEmployerExcessDeposit {
        contributed_on: 20260315,
        deadline: 20260415,
        amount: 100,
        in_wages: false,
    });
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_designated_employer_excess_income"), 1);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 0);
    assert_eq!(value(&r, "agi"), 60001);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn married_monthly_hsa_family_excess_uses_each_agreed_owner_share() {
    let mut d = complete(FederalStatus::Joint);
    let mut taxpayer = hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 500000);
    taxpayer.excess_accounting = Some(hsa_excess_accounting(100000, None, 0, 0, 0));
    let spouse = hsa_entry(Owner::Spouse, HsaCoverage::Family, 400000);
    d.adjustments.as_mut().unwrap().hsa = vec![taxpayer, spouse];
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: None,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 8375);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 625);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 38);
    assert_eq!(value(&r, "hsa_1_excess_line49"), 38);
    assert_eq!(printed_line(&r, "hsa_1_excess_line49"), (TaxForm::Form5329, "49".into()));
    assert_eq!(restart(d.clone()).draft, d);

    let a = d.adjustments.as_mut().unwrap();
    a.hsa_family_share.as_mut().unwrap().taxpayer_parts = 4;
    a.hsa_family_share.as_mut().unwrap().spouse_parts = 3;
    let accounting = a.hsa[0].excess_accounting.take();
    a.hsa[1].excess_accounting = accounting;
    let reallocated = supported(&d);
    assert_eq!(value(&reallocated, "adjustments"), 8750);
    assert_eq!(value(&reallocated, "hsa_2_return_year_uncorrected_excess"), 250);
    assert_eq!(value(&reallocated, "hsa_return_year_excess_contribution_tax"), 15);

    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().method =
        HsaAllocationMethod::December;
    assert_eq!(value(&supported(&d), "hsa_2_return_year_uncorrected_excess"), 250);
}

#[test]
fn december_family_hsa_excess_does_not_create_false_last_month_recapture_amount() {
    let mut d = joint_december_hsa(500000, 437500, 1, 1);
    d.adjustments.as_mut().unwrap().hsa[0].excess_accounting =
        Some(hsa_excess_accounting(100000, None, 0, 0, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 8750);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 625);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 38);
    assert_eq!(value(&r, "hsa_1_last_month_contributions"), 4010);
    assert_eq!(value(&r, "hsa_2_last_month_contributions"), 4010);
    assert_eq!(restart(d.clone()).draft, d);

    d.adjustments.as_mut().unwrap().hsa[0].last_month_loss =
        Some(HsaEligibilityLoss { date: 20261215, reason: HsaLossReason::Other });
    let recaptured = supported(&d);
    assert_eq!(value(&recaptured, "hsa_1_last_month_contributions"), 4010);
    assert_eq!(value(&recaptured, "hsa_last_month_recapture"), 4010);
    assert_eq!(value(&recaptured, "hsa_testing_period_tax"), 401);
    assert_eq!(value(&recaptured, "hsa_return_year_excess_contribution_tax"), 38);
}

#[test]
fn hsa_correction_deadline_confirmation_and_overallocated_principal_are_checked() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 500000);
    h.corrections = vec![hsa_correction(2026, 20270416, 60000, 5000)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    unsupported(&d, "HSA");
    d.adjustments.as_mut().unwrap().hsa[0].corrections[0].deadline = 20271015;
    supported(&d); // Source confirmation includes valid extension/relief entitlement.
    // Returned after its deadline it is no timely correction (the printed
    // dates decide, F3, 2026-10-09).
    d.adjustments.as_mut().unwrap().hsa[0].corrections[0].returned = 20271016;
    unsupported(&d, "HSA");
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.corrections[0].principal = 500001;
    assert!(!h.valid());
}

#[test]
fn fully_corrected_hsa_contributions_need_no_current_eligibility_or_dependency_exception() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 100000);
    h.corrections = vec![hsa_correction(2026, 20261231, 100000, 0)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "hsa_correction_earnings"), 0);
    d.adjustments.as_mut().unwrap().hsa[0].corrections[0].principal -= 1;
    unsupported(&d, "HSA");
}

#[test]
fn prior_employer_hsa_correction_reported_as_other_income_does_not_retax_principal() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 440000);
    let mut c = hsa_correction(2025, 20260415, 100000, 5000);
    c.payer = HsaCorrectionPayer::Employer { in_wages: false };
    h.corrections = vec![c];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "agi"), 55650);
    assert_eq!(value(&r, "hsa_correction_earnings"), 50);
    assert_eq!(value(&r, "hsa_correction_employer_principal_other_income"), 0);
    assert_eq!(return_line(&r, "8"), 5000);
    assert_eq!(return_line(&r, "23"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_employer_hsa_correction_requires_timely_payment() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut c = hsa_correction(2025, 20260415, 100000, -10000);
    c.payer = HsaCorrectionPayer::Employer { in_wages: false };
    h.corrections = vec![c];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(return_line(&r, "8"), 0);
    d.adjustments.as_mut().unwrap().hsa[0].corrections[0].returned = 20260416;
    unsupported(&d, "HSA");
}

#[test]
fn prior_last_month_hsa_recapture_uses_historical_months_and_not_current_limits() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.last_month_history = Some(last_month_history(6, 430000));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_monthly_limit"), 2150);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 2150);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 215);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    assert_eq!(return_line(&r, "8"), 215000);
    assert_eq!(return_line(&r, "23"), 21500);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_last_month_hsa_recapture_uses_actual_contributions_and_archer_reduction() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(6, 200000);
    history.archer = 100000;
    h.last_month_history = Some(history);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_monthly_limit"), 1150);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 850);
    d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap().contributions =
        100000;
    assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), 0);
}

// IRC 223(b)(3): age 55 before the close of 2025, day-exact (Pub. 501). Born
// 1971-01-01 the 55th birthday is 2026-01-01, so 55 is attained on 2025-12-31
// and the catch-up applies (the same result as 1970-12-31); born 1971-01-02 it
// is attained on 2026-01-01 and the catch-up does not.
#[test]
fn prior_last_month_hsa_catchup_uses_age_in_2025_and_family_coverage() {
    for (birth, expected) in [(19710102, 5688), (19710101, 6104), (19701231, 6104)] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
        let mut history = last_month_history(1, if birth <= 19710101 { 955000 } else { 855000 });
        history.months[11] = HsaCoverage::Family;
        // Six earlier self-only months; December family must not replace their source coverage.
        history.months[..6].fill(HsaCoverage::SelfOnly);
        h.last_month_history = Some(history);
        d.adjustments.as_mut().unwrap().hsa = vec![h];
        assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), expected);
    }
}

#[test]
fn prior_last_month_hsa_first_loss_year_and_death_disability_exceptions() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.last_month_history = Some(last_month_history(6, 530000));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    for (date, reason, expected) in [
        (20261231, HsaLossReason::Other, 2650),
        (20251231, HsaLossReason::Other, 0),
        (20260601, HsaLossReason::Disability, 0),
    ] {
        d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap().loss =
            Some(HsaEligibilityLoss { date, reason });
        assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), expected);
    }
    d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap().loss =
        Some(HsaEligibilityLoss { date: 20260601, reason: HsaLossReason::Death });
    unsupported(&d, "HSA");
    let mut final_return = death(20260601).unwrap();
    final_return.ordinary = true;
    d.people.as_mut().unwrap().taxpayer.died = Some(final_return);
    assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), 0);
}

#[test]
fn prior_last_month_hsa_higher_monthly_limit_and_continued_eligibility() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(12, 713300);
    history.months[..8].fill(HsaCoverage::Family);
    h.last_month_history = Some(history);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_monthly_limit"), 7133);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 0);
    let history = d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap();
    *history = last_month_history(1, 430000);
    history.loss = None;
    assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), 0);
}

#[test]
fn hsa_part_three_rounds_combined_recapture_once_per_owner() {
    let mut d = complete(FederalStatus::Joint);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.last_month_history = Some(last_month_history(6, 215000));
    h.funding_history = vec![hsa_funding(20250715, 20260601, 600)];
    let mut spouse = h.clone();
    spouse.owner = Owner::Spouse;
    d.adjustments.as_mut().unwrap().hsa = vec![h, spouse];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 12);
    assert_eq!(value(&r, "hsa_funding_recapture"), 12);
    // Each owner: ($6 + $6) * 10% rounds to $1, not $1 + $1.
    assert_eq!(value(&r, "hsa_testing_period_tax"), 2);
    for owner in 1..=2 {
        for (suffix, printed, expected) in [
            ("testing_last_month_income", "18", 6),
            ("testing_funding_income", "19", 6),
            ("testing_total_income", "20", 12),
            ("testing_tax", "21", 1),
        ] {
            let id = alloc::format!("hsa_{owner}_{suffix}");
            assert_eq!(value(&r, &id), expected);
            assert_eq!(printed_line(&r, &id), (TaxForm::Form8889, printed.into()));
        }
    }
    assert_eq!(return_line(&r, "23"), 200);
}

#[test]
fn hsa_overlapping_histories_require_consistent_first_loss_only_within_overlap() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.last_month_history = Some(last_month_history(7, 100000));
    h.funding_history = vec![hsa_funding(20250615, 20260501, 100000)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    unsupported(&d, "HSA"); // June versus May first loss.
    d.adjustments.as_mut().unwrap().hsa[0].funding_history[0].loss = None;
    unsupported(&d, "HSA"); // Cannot claim eligibility through June while reporting June loss.
    d.adjustments.as_mut().unwrap().hsa[0]
        .last_month_history
        .as_mut()
        .unwrap()
        .loss
        .as_mut()
        .unwrap()
        .date = 20260701;
    supported(&d); // Funding history confirms eligibility only through June 30.
    d.adjustments.as_mut().unwrap().hsa[0].funding_history[0].loss =
        Some(HsaEligibilityLoss { date: 20251015, reason: HsaLossReason::Other });
    supported(&d); // Earlier loss/restoration predates the last-month testing period.
}

#[test]
fn hsa_overlapping_disability_and_two_transfer_deadlines_remain_independent() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(7, 200000);
    history.months[7..].fill(HsaCoverage::Family);
    history.loss.as_mut().unwrap().date = 20260701;
    h.last_month_history = Some(history);
    h.funding_history =
        vec![hsa_funding(20250615, 20260701, 100000), hsa_funding(20250815, 20260701, 200000)];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    // (2 self-only + 5 family months) / 12 = $4,279; total deposits $5,000.
    assert_eq!(value(&r, "hsa_last_month_recapture"), 721);
    assert_eq!(value(&r, "hsa_funding_recapture"), 2000);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 272);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.last_month_history.as_mut().unwrap().loss.as_mut().unwrap().reason =
        HsaLossReason::Disability;
    for transfer in &mut h.funding_history {
        transfer.loss.as_mut().unwrap().reason = HsaLossReason::Disability;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 0);
    assert_eq!(value(&r, "hsa_funding_recapture"), 0);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 0);
}

#[test]
fn current_hsa_last_month_allows_full_independent_limit_and_exposes_testing_requirement() {
    for (coverage, amount, exposed) in
        [(HsaCoverage::SelfOnly, 440000, 403333), (HsaCoverage::Family, 875000, 802083)]
    {
        let mut d = complete(FederalStatus::Single);
        d.adjustments.as_mut().unwrap().hsa = vec![december_hsa(coverage, amount)];
        let r = supported(&d);
        assert_eq!(value(&r, "adjustments"), amount / 100);
        assert_eq!(iterative_work(&r, "hsa_1_last_month_contributions"), exposed);
        assert_eq!(value(&r, "hsa_last_month_recapture"), 0);
        let note = r
            .review_notes
            .iter()
            .find(|v| v.contains("December 31, 2027"))
            .expect("last-month testing-period note");
        assert_eq!(
            note.display,
            Message::new(
                "NOTE_HSA_LAST_MONTH_RULE",
                vec![
                    MessageArg::Resource("PERSON_YOU".into()),
                    MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 2,
                        minor_units: exposed
                    }
                ],
            )
        );
        assert_eq!(
            crate::interview::issue_page(&d, note.context.as_ref().unwrap(), None),
            Some(TaxSection::Hsa)
        );
        assert_eq!(restart(d.clone()).draft, d);
        d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
        unsupported(&d, "HSA");
    }
}

#[test]
fn current_hsa_last_month_uses_first_loss_year_and_excepted_causes() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().hsa = vec![december_hsa(HsaCoverage::SelfOnly, 440000)];
    for (date, reason, income, tax) in [
        (20261215, HsaLossReason::Other, 4033, 403),
        (20270101, HsaLossReason::Other, 0, 0),
        (20271231, HsaLossReason::Other, 0, 0),
        (20261215, HsaLossReason::Disability, 0, 0),
        (20270101, HsaLossReason::Death, 0, 0),
    ] {
        d.adjustments.as_mut().unwrap().hsa[0].last_month_loss =
            Some(HsaEligibilityLoss { date, reason });
        let r = supported(&d);
        assert_eq!(value(&r, "adjustments"), 4400);
        assert_eq!(value(&r, "hsa_last_month_recapture"), income);
        assert_eq!(value(&r, "hsa_testing_period_tax"), tax);
        let note = r
            .review_notes
            .iter()
            .find(|note| note.display.key == "NOTE_HSA_LAST_MONTH_RULE")
            .expect("last-month rule note remains readable for raw history");
        assert_eq!(
            crate::interview::issue_page(&d, note.context.as_ref().unwrap(), None),
            Some(TaxSection::Hsa)
        );
        assert_eq!(return_line(&r, "8"), income * 100);
        assert_eq!(return_line(&r, "23"), tax * 100);
    }
    d.adjustments.as_mut().unwrap().hsa[0].last_month_loss =
        Some(HsaEligibilityLoss { date: 20261215, reason: HsaLossReason::Death });
    unsupported(&d, "HSA");
    let mut final_return = death(20261215).unwrap();
    final_return.ordinary = true;
    d.people.as_mut().unwrap().taxpayer.died = Some(final_return);
    assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), 0);
}

#[test]
fn current_hsa_last_month_preserves_higher_monthly_limit_and_no_december_case() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 657500);
    let mut months = vec![HsaCoverage::Family; 6];
    months.extend(vec![HsaCoverage::SelfOnly; 6]);
    h.coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 6575);
    assert!(!r.review_notes.iter().any(|v| v.contains("uses the 2026 last-month rule")));
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.personal = 440000;
    let mut months = vec![HsaCoverage::SelfOnly; 11];
    months.push(HsaCoverage::None);
    h.coverage = HsaEligibility::Monthly { months };
    unsupported(&d, "HSA");
}

#[test]
fn current_hsa_last_month_rejects_invalid_history_and_unresolved_shared_allocation() {
    let mut h = december_hsa(HsaCoverage::SelfOnly, 440000);
    for date in [20261130, 20280101] {
        h.last_month_loss = Some(HsaEligibilityLoss { date, reason: HsaLossReason::Other });
        assert!(!h.valid());
    }
    let mut d = complete(FederalStatus::Joint);
    d.adjustments.as_mut().unwrap().hsa = vec![december_hsa(HsaCoverage::Family, 875000)];
    unsupported(&d, "HSA");
}

#[test]
fn current_and_prior_hsa_last_month_recapture_combine_before_owner_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut h = december_hsa(HsaCoverage::SelfOnly, 440000);
    let loss = HsaEligibilityLoss { date: 20261215, reason: HsaLossReason::Other };
    let mut history = last_month_history(6, 215600);
    history.loss = Some(loss.clone());
    h.last_month_history = Some(history);
    h.last_month_loss = Some(loss);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 4039);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 404);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(return_line(&r, "8"), 403900);
}

#[test]
fn historical_hsa_family_equal_shares_use_each_owners_actual_testing_failure() {
    let mut d = complete(FederalStatus::Joint);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(1, 427500);
    historical_hsa_family(&mut history, 1, 1);
    h.last_month_history = Some(history);
    let mut spouse = h.clone();
    spouse.owner = Owner::Spouse;
    spouse.last_month_history.as_mut().unwrap().loss = None;
    d.adjustments.as_mut().unwrap().hsa = vec![h, spouse];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_monthly_limit"), 356);
    assert_eq!(value(&r, "hsa_2_prior_monthly_limit"), 356);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 3919);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 392);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn historical_hsa_family_allocation_supports_zero_and_exact_nonpercentage_ratios() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(1, 100000);
    historical_hsa_family(&mut history, 20, 151); // 1000 / 8550, exact.
    h.last_month_history = Some(history);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_monthly_limit"), 83);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 917);
    let history = d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap();
    history.contributions = 0;
    history.family.as_mut().unwrap().owner_parts = 0;
    assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), 0);
}

#[test]
fn historical_hsa_family_archer_offsets_precede_sharing_and_catchup_stays_with_owner() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19701231;
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(6, 477500);
    historical_hsa_family(&mut history, 1, 1);
    history.archer = 40000;
    history.family.as_mut().unwrap().spouse_archer = 60000;
    h.last_month_history = Some(history);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    // Ordinary: (4275 - 1000)/2 + 500; full: (8550 - 1000)/2 + 1000.
    assert_eq!(iterative_work(&r, "hsa_1_prior_monthly_limit"), 213750);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 2638);
    d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap().contributions +=
        100;
    unsupported(&d, "HSA");
}

#[test]
fn historical_hsa_family_mixed_months_keep_self_only_allowance_and_spouse_family_override() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(12, 427500);
    historical_hsa_family(&mut history, 1, 1);
    history.months.fill(HsaCoverage::SelfOnly);
    history.family.as_mut().unwrap().spouse_months[..6].fill(HsaCoverage::SelfOnly);
    h.last_month_history = Some(history);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(iterative_work(&r, "hsa_1_prior_monthly_limit"), 428750);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 0);
}

#[test]
fn historical_hsa_family_inconsistent_household_facts_and_invalid_shares_are_rejected() {
    let mut d = complete(FederalStatus::Joint);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(1, 285000);
    historical_hsa_family(&mut history, 1, 2);
    h.last_month_history = Some(history);
    let mut spouse = h.clone();
    spouse.owner = Owner::Spouse;
    let sh = spouse.last_month_history.as_mut().unwrap();
    sh.contributions = 570000;
    sh.family.as_mut().unwrap().owner_parts = 2;
    sh.family.as_mut().unwrap().spouse_parts = 1;
    d.adjustments.as_mut().unwrap().hsa = vec![h, spouse];
    supported(&d);
    d.adjustments.as_mut().unwrap().hsa[1]
        .last_month_history
        .as_mut()
        .unwrap()
        .family
        .as_mut()
        .unwrap()
        .owner_parts = 1;
    unsupported(&d, "HSA");
    let h = d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap();
    h.family.as_mut().unwrap().owner_parts = 0;
    h.family.as_mut().unwrap().spouse_parts = 0;
    assert!(!h.valid());
    h.family.as_mut().unwrap().owner_parts = 1;
    h.months[11] = HsaCoverage::SelfOnly;
    h.family.as_mut().unwrap().spouse_months[11] = HsaCoverage::SelfOnly;
    // The only family month was December; a family allocation without any
    // family coverage is still inconsistent.
    assert!(!h.valid());
}

#[test]
fn current_shared_hsa_december_allowance_preserves_one_family_pool() {
    let mut d = joint_december_hsa(437500, 437500, 1, 1);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 8750);
    assert_eq!(value(&r, "hsa_1_last_month_contributions"), 4010);
    assert_eq!(value(&r, "hsa_2_last_month_contributions"), 4010);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn current_shared_hsa_exact_agreement_can_allocate_all_to_one_owner() {
    let mut d = joint_december_hsa(875000, 0, 1, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 8750);
    assert_eq!(iterative_work(&r, "hsa_1_last_month_contributions"), 802083);
    d.adjustments.as_mut().unwrap().hsa[0].personal = 100000;
    d.adjustments.as_mut().unwrap().hsa[1].personal = 775000;
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        other_coverage: None,
        taxpayer_parts: 4,
        spouse_parts: 31,
    });
    assert_eq!(value(&supported(&d), "adjustments"), 8750);
}

#[test]
fn current_shared_hsa_owner_catchup_cannot_transfer_to_spouse() {
    let mut d = joint_december_hsa(537500, 437500, 1, 1);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19711231;
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 9750);
    d.adjustments.as_mut().unwrap().hsa[0].personal -= 100;
    d.adjustments.as_mut().unwrap().hsa[1].personal += 100;
    unsupported(&d, "HSA");
}

#[test]
fn current_shared_hsa_recapture_depends_on_each_owners_loss() {
    let mut d = joint_december_hsa(437500, 437500, 1, 1);
    d.adjustments.as_mut().unwrap().hsa[0].last_month_loss =
        Some(HsaEligibilityLoss { date: 20261220, reason: HsaLossReason::Other });
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 4010);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 401);
    assert_eq!(return_line(&r, "8"), 401000);
    assert_eq!(return_line(&r, "23"), 40100);
    d.adjustments.as_mut().unwrap().hsa[1].last_month_loss =
        Some(HsaEligibilityLoss { date: 20261221, reason: HsaLossReason::Other });
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 8020);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 802);
}

#[test]
fn current_shared_hsa_archer_reduction_is_applied_before_the_agreed_split() {
    let mut d = joint_december_hsa(312500, 212500, 1, 1);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19660615;
    add(&mut d, wage(2, Owner::Spouse, 4000000, 0));
    set_archer(
        &mut d,
        vec![
            archer_contributor(Owner::Taxpayer, archer_family(6000), 2250),
            archer_contributor(Owner::Spouse, archer_family(6000), 2250),
        ],
        ArcherSharing::Equal,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 9750);
    assert_eq!(iterative_work(&r, "hsa_1_last_month_contributions"), 304167);
    assert_eq!(value(&r, "hsa_2_last_month_contributions"), 2125);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn current_shared_hsa_requires_complete_coverage_nonzero_shares_and_december_eligibility() {
    let mut d = joint_december_hsa(437500, 437500, 0, 0);
    unsupported(&d, "HSA");
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().taxpayer_parts = 1;
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().spouse_parts = 1;
    d.adjustments.as_mut().unwrap().hsa[1].coverage =
        HsaEligibility::FullYear { coverage: HsaCoverage::None };
    unsupported(&d, "HSA");
    d.adjustments.as_mut().unwrap().hsa.pop();
    unsupported(&d, "HSA");
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        other_coverage: None,
        taxpayer_parts: 1,
        spouse_parts: 0,
    });
    unsupported(&d, "HSA");
}

#[test]
fn monthly_family_hsa_does_not_invent_december_allowance_or_full_catchup() {
    let mut d = separate_december_hsa(50000);
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().method =
        HsaAllocationMethod::Monthly;
    unsupported(&d, "HSA");
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().method =
        HsaAllocationMethod::December;
    assert_eq!(value(&supported(&d), "hsa_1_last_month_contributions"), 135);
    let mut d = separate_december_hsa(268750);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19711231;
    let mut months = vec![HsaCoverage::Family; 6];
    months.extend(vec![HsaCoverage::None; 6]);
    d.adjustments.as_mut().unwrap().hsa[0].coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().method =
        HsaAllocationMethod::Monthly;
    assert_eq!(value(&supported(&d), "adjustments"), 2688);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn monthly_family_hsa_adds_independent_self_only_months_before_eligibility_ends() {
    let mut d = separate_december_hsa(292500);
    let mut own = vec![HsaCoverage::SelfOnly; 8];
    own.extend(vec![HsaCoverage::None; 4]);
    let mut spouse = vec![HsaCoverage::Family; 4];
    spouse.extend(vec![HsaCoverage::SelfOnly; 4]);
    spouse.extend(vec![HsaCoverage::None; 4]);
    d.adjustments.as_mut().unwrap().hsa[0].coverage = HsaEligibility::Monthly { months: own };
    let share = d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap();
    share.method = HsaAllocationMethod::Monthly;
    share.other_coverage = Some(HsaEligibility::Monthly { months: spouse });
    assert_eq!(value(&supported(&d), "adjustments"), 2925);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn monthly_family_hsa_honors_joint_ratio_and_shared_cap_without_december() {
    let mut d = joint_december_hsa(87500, 350000, 1, 4);
    let mut months = vec![HsaCoverage::Family; 6];
    months.extend(vec![HsaCoverage::None; 6]);
    for h in &mut d.adjustments.as_mut().unwrap().hsa {
        h.coverage = HsaEligibility::Monthly { months: months.clone() };
    }
    d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().method =
        HsaAllocationMethod::Monthly;
    assert_eq!(value(&supported(&d), "adjustments"), 4375);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 100;
    d.adjustments.as_mut().unwrap().hsa[1].personal -= 100;
    unsupported(&d, "HSA"); // Aggregate still fits, but the agreed division does not.
}

#[test]
fn historical_hsa_family_to_self_only_december_uses_independent_last_month_limit() {
    let mut d = complete(FederalStatus::Separate);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    let mut history = last_month_history(2, 430000);
    historical_hsa_family(&mut history, 1, 1);
    history.months[11] = HsaCoverage::SelfOnly;
    history.family.as_mut().unwrap().spouse_months[11] = HsaCoverage::SelfOnly;
    assert!(history.valid());
    // November family share 8550 / 24 plus December self-only 4300 / 12
    // is 714.5833..., leaving 3585.4167...; subtract before rounding.
    assert_eq!(history.limits(19800615), (715, 4300));
    h.last_month_history = Some(history);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 3585);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 359);
    assert_eq!(return_line(&r, "8"), 358500);
    assert_eq!(restart(d.clone()).draft, d);
    d.adjustments.as_mut().unwrap().hsa[0].last_month_history.as_mut().unwrap().contributions +=
        100;
    unsupported(&d, "HSA");
}

#[test]
fn historical_hsa_december_self_only_keeps_higher_actual_monthly_allocation() {
    let mut history = last_month_history(12, 643000);
    historical_hsa_family(&mut history, 1, 0);
    history.months[6..].fill(HsaCoverage::SelfOnly);
    history.family.as_mut().unwrap().spouse_months[6..].fill(HsaCoverage::SelfOnly);
    assert!(history.valid());
    assert_eq!(history.limits(19800615), (6425, 6425));
    history.contributions = 642500;
    assert_eq!(history.recapture(19800615, 0), 0);
    // December self-only does not borrow the spouse's catch-up or Archer facts.
    history.archer = 40000;
    history.family.as_mut().unwrap().spouse_archer = 60000;
    assert_eq!(history.limits(19701231), (6425, 6425));
}

#[test]
fn historical_hsa_december_self_only_preserves_own_catchup_and_archer_reduction() {
    let mut history = last_month_history(2, 490000);
    historical_hsa_family(&mut history, 1, 1);
    history.months[11] = HsaCoverage::SelfOnly;
    history.family.as_mut().unwrap().spouse_months[11] = HsaCoverage::SelfOnly;
    history.archer = 40000;
    history.family.as_mut().unwrap().spouse_archer = 60000;
    assert_eq!(history.limits(19701231), (525, 4900));
    assert_eq!(history.recapture(19701231, 0), 4375);
    history.family.as_mut().unwrap().spouse_months[11] = HsaCoverage::Family;
    // A spouse's December family plan still overrides own self-only coverage.
    assert_eq!(history.limits(19701231).1, 4775);
}

#[test]
fn qualified_hsa_funding_uses_room_before_new_excess_and_prior_carry() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4000);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.personal = 100000;
    h.excess_accounting = Some(hsa_excess_accounting(500000, None, 0, 0, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 400);
    for (suffix, amount) in [
        ("funding", 400_000),
        ("other", 400_000),
        ("personal_limit", 40_000),
        ("deduction", 40_000),
    ] {
        assert_eq!(line_amount(&r, &alloc::format!("hsa_1_contribution_{suffix}")), amount);
    }
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 600);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 36);
    assert_eq!(return_line(&r, "4b"), 200000);
    assert_eq!(restart(d.clone()).draft, d);

    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.personal = 20000;
    h.excess_accounting.as_mut().unwrap().prior_year =
        Some(HsaPriorExcess { line_48: 50000, line_49: 3000 });
    let prior = supported(&d);
    assert_eq!(value(&prior, "adjustments"), 400);
    assert_eq!(value(&prior, "hsa_1_prior_excess_deducted_return_year"), 200);
    assert_eq!(value(&prior, "hsa_1_prior_excess_remaining_following_year"), 300);
    assert_eq!(value(&prior, "hsa_return_year_excess_contribution_tax"), 18);

    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.personal = 0;
    h.employer = 100000;
    h.employer_outside = 100000;
    h.funding[0].amount = 440000;
    h.excess_accounting.as_mut().unwrap().prior_year = None;
    let employer = supported(&d);
    assert_eq!(value(&employer, "hsa_1_return_year_uncorrected_employer_excess"), 1000);
    assert_eq!(value(&employer, "hsa_return_year_uncorrected_employer_excess_other_income"), 1000);
    assert_eq!(value(&employer, "hsa_return_year_excess_contribution_tax"), 60);
}

#[test]
fn qualified_hsa_funding_can_exceed_monthly_room_without_becoming_the_excess() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4400);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    let mut months = vec![HsaCoverage::None; 12];
    months[..3].fill(HsaCoverage::SelfOnly);
    h.coverage = HsaEligibility::Monthly { months };
    h.funding[0].loss = Some(HsaEligibilityLoss { date: 20260401, reason: HsaLossReason::Other });
    h.personal = 10000;
    h.excess_accounting = Some(hsa_excess_accounting(10000, None, 0, 0, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 4400);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 100);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 6);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn current_hsa_funding_uses_transfer_coverage_not_monthly_contribution_limit() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4400);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    let mut months = vec![HsaCoverage::None; 12];
    months[0] = HsaCoverage::SelfOnly;
    h.coverage = HsaEligibility::Monthly { months };
    h.funding[0].loss = Some(HsaEligibilityLoss { date: 20260201, reason: HsaLossReason::Other });
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 4400);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 440);
    assert_eq!(value(&r, "early_distribution_tax"), 160);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(return_line(&r, "8"), 440000);
}

#[test]
fn current_hsa_funding_death_disability_and_next_year_loss_do_not_recapture_now() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4400);
    for (date, reason) in [
        (20260601, HsaLossReason::Disability),
        (20270131, HsaLossReason::Other),
        (20270131, HsaLossReason::Death),
    ] {
        let loss = HsaEligibilityLoss { date, reason };
        let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
        h.funding[0].loss = Some(loss.clone());
        h.last_month_loss = if date / 10000 == 2027 { Some(loss) } else { None };
        assert_eq!(value(&supported(&d), "hsa_funding_recapture"), 0);
    }
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.funding[0].loss.as_mut().unwrap().date = 20270201;
    h.last_month_loss.as_mut().unwrap().date = 20270201;
    assert_eq!(value(&supported(&d), "hsa_funding_recapture"), 0);
}

#[test]
fn current_hsa_funding_two_transfers_require_later_month_and_family_upgrade() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4000);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|month| if month == 0 { HsaCoverage::SelfOnly } else { HsaCoverage::Family })
            .collect(),
    };
    let mut second = h.funding[0].clone();
    second.funded = 20260215;
    second.basis = HsaFundingBasis::Pretax { distributed: 20260215 };
    second.coverage = HsaCoverage::Family;
    second.amount = 200000;
    h.funding.push(second);
    assert_eq!(return_line(&supported(&d), "4b"), 0);
    d.adjustments.as_mut().unwrap().hsa[0].funding[1].funded = 20260131;
    d.adjustments.as_mut().unwrap().hsa[0].funding[1].basis =
        HsaFundingBasis::Pretax { distributed: 20260131 };
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
    d.adjustments.as_mut().unwrap().hsa[0].funding[1].funded = 20260215;
    d.adjustments.as_mut().unwrap().hsa[0].funding[1].basis =
        HsaFundingBasis::Pretax { distributed: 20260215 };
    d.adjustments.as_mut().unwrap().hsa[0].funding[1].amount += 1;
    assert!(crate::hsa_funding::source_reference_rule(&d).is_none());
    unsupported(&d, "HSA funding");
}

#[test]
fn current_hsa_funding_overlaps_last_month_income_without_double_recapture() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 2000);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.coverage = december_hsa(HsaCoverage::SelfOnly, 0).coverage;
    h.personal = 240000;
    h.funding[0].funded = 20261201;
    let loss = HsaEligibilityLoss { date: 20261215, reason: HsaLossReason::Other };
    h.funding[0].loss = Some(loss.clone());
    h.last_month_loss = Some(loss);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 2000);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 2400);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 440);
    assert_eq!(value(&r, "adjustments"), 2400);
    d.adjustments.as_mut().unwrap().hsa[0].last_month_loss = None;
    unsupported(&d, "HSA funding");
}

#[test]
fn current_hsa_funding_age_limit_and_prior_lifetime_transfer_are_enforced() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4401);
    assert!(crate::hsa_funding::source_reference_rule(&d).is_none());
    unsupported(&d, "HSA funding");
    d.people.as_mut().unwrap().taxpayer.birth_date = 19711231;
    supported(&d);
    d.adjustments.as_mut().unwrap().hsa[0].funding[0].amount = 540100;
    assert!(crate::hsa_funding::source_reference_rule(&d).is_none());
    unsupported(&d, "HSA funding");
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.funding[0].amount = 540000;
    h.funding_history = vec![hsa_funding(20250115, 20260601, 100000)];
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
fn current_hsa_funding_spouse_uses_own_source_and_preserves_withholding() {
    let mut d = complete(FederalStatus::Joint);
    current_hsa_funding(&mut d, 4400);
    let id = d.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    d.income.iter_mut().find(|v| v.id == id).unwrap().owner = crate::DocumentOwner::Spouses;
    d.adjustments.as_mut().unwrap().hsa[0].owner = Owner::Spouse;
    // Neutral completion fact: the taxpayer is ineligible for HSA contributions.
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::Monthly,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: Some(HsaEligibility::FullYear { coverage: HsaCoverage::None }),
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 160000);
    assert_eq!(value(&r, "early_distribution_tax"), 160);
    // The transfer does not remove withholding shown on the source Form 1099-R.
    let mut without = d.clone();
    without.adjustments.as_mut().unwrap().hsa[0].funding.clear();
    assert_eq!(r.payments, supported(&without).payments);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn health_loan_assistance_excludes_payment_and_removes_interest_before_cap_and_phaseout() {
    for (with_nol, agi) in [(false, 91750), (true, 30480)] {
        for program in [
            HealthLoanProgram::National,
            HealthLoanProgram::PublicHealthState,
            HealthLoanProgram::UnderservedState,
        ] {
            let mut d = complete(FederalStatus::Single);
            d.income[0] = wage(1, Owner::Taxpayer, 9250000, 0);
            d.adjustments.as_mut().unwrap().set_student_loan_interest(300000);
            add(&mut d, health_loan_assistance(program, 150000));
            if with_nol {
                d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
            }
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), agi);
            assert_eq!(value(&r, "health_loan_2_excluded"), 5000);
            assert_eq!(value(&r, "health_loan_interest_removed"), 1500);
            if with_nol {
                assert_eq!(value(&r, "prior_nol_deduction"), 60520);
                assert_eq!(value(&r, "nol_2025_carryforward"), 39480);
            } else {
                assert_eq!(value(&r, "adjustments"), 750);
            }
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}
#[test]
fn health_loan_zero_overlap_does_not_subtract_interest_twice_or_require_deduction() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, health_loan_assistance(HealthLoanProgram::National, 0));
    assert_eq!(value(&supported(&d), "agi"), value(&before, "agi"));
    let a = d.adjustments.as_mut().unwrap();
    a.set_student_loan_interest(100000);
    assert_eq!(value(&supported(&d), "adjustments"), 1000);
}
#[test]
fn health_loan_negative_eligibility_or_excess_interest_overlap_requires_review() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, health_loan_assistance(HealthLoanProgram::National, 1));
    unsupported(&d, "Health-service loan assistance");
    let before = supported(&complete(FederalStatus::Single));
    if let FederalIncome::HealthLoanAssistance { data } = &mut d.income[1].income {
        data.interest = 0;
        data.confirmed = false;
        assert!(data.valid());
    }
    // Unconfirmed program: the assistance is taxed as canceled debt.
    let r = supported(&d);
    assert!(noted(&r, "WARN_HEALTH_LOAN_ASSISTANCE_ASSUMED_TAXABLE"), "{r:?}");
    assert!(value(&r, "agi") > value(&before, "agi"));
}

#[test]
fn personal_rental_expense_report_includes_new_profitable_activity_without_history() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent(5000, 2000)));
    let r = supported(&d);
    assert!(
        breakdown(&r)
            .iter()
            .any(|v| v.label == "Personal rental 2 suspended loss" && v.amount == 0)
    );
    assert!(
        breakdown(&r)
            .iter()
            .any(|v| v.label == "Personal rental 2 allowed expenses" && v.amount == 200000)
    );
    assert_eq!(value(&r, "personal_rental_2_prior_suspended_expenses"), 0);
    assert_eq!(value(&r, "personal_rental_2_allowed_expenses"), 2000);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 0);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 2000);
}

#[test]
fn prior_nol_refigures_itemized_medical_deductions_and_compares_complete_routes() {
    let mut d = complete(FederalStatus::Single);
    let f = itemized(d.deductions.as_mut().unwrap());
    f.medical = 2000000;
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let automatic = supported(&d);
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 35600);
    assert_eq!(value(&r, "agi"), 24400);
    assert_eq!(return_line(&r, "12e"), 1817000);
    assert_eq!(value(&r, "taxable_income"), 6230);
    assert_eq!(automatic.tax, r.tax);
    assert_eq!(value(&automatic, "prior_nol_deduction"), 35600);
    d.deductions.as_mut().unwrap().itemize_override = None;
    // 2026-11-02: clearing the override keeps the same lower-liability NOL route.
    let cleared = supported(&d);
    assert_eq!(value(&cleared, "prior_nol_deduction"), 35600);
    assert_eq!(cleared.tax, r.tax);
}

#[test]
fn prior_nol_capital_addback_refigures_itemized_medical_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 9000000, 0)];
    add(&mut d, income(bad_debt(10000, 0)));
    let f = itemized(d.deductions.as_mut().unwrap());
    f.medical = 2000000;
    let facts = f.clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 58820);
    assert_eq!(value(&r, "nol_2025_used"), 62045);
    assert_eq!(value(&r, "nol_2025_carryforward"), 37955);
    assert_eq!(value(&r, "agi"), 28180);
    assert_eq!(value(&r, "capital_short_carryforward"), 7000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_coordinates_nonitemizer_cash_charity_without_charitable_carryover() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.set_cash_charity(100000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 34320);
    assert_eq!(value(&r, "nol_2025_carryforward"), 65680);
    assert_eq!(value(&r, "agi"), 25680);
    assert_eq!(return_line(&r, "12f"), 100000);
    d.income = vec![wage(1, Owner::Taxpayer, 5600000, 0)];
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 55000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 1000);
    assert_eq!(return_line(&r, "12f"), 60000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 16100);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_preserves_able_expense_proration_and_additional_tax() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(able_distribution(2400, 400, 1600)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 133);
    assert_eq!(value(&r, "account_additional_tax"), 13);
    assert_eq!(value(&r, "agi"), -39867);
    assert_eq!(value(&r, "nol_2017_carryforward"), 55967);
    assert_eq!(return_line(&r, "23"), 1300);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_personal_debt_uses_taxable_income_and_preserves_exceptions() {
    for (treatment, taxable, deduction) in [
        (PersonalDebtTreatment::Taxable, 5000, 39120),
        (PersonalDebtTreatment::GiftOrBequest, 0, 35120),
        (
            PersonalDebtTreatment::StudentDischarge {
                reason: StudentDischargeReason::PublicService,
            },
            0,
            35120,
        ),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, canceled_personal_debt(5000, treatment));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "personal_debt_cancellation"), taxable);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "nol_2025_carryforward"), 100000 - deduction);
        assert_eq!(value(&r, "agi"), 60000 + taxable - deduction);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn mortgage_credit_precedes_homeowner_carries_and_preserves_refundable_child_credit() {
    let mut d = complete(FederalStatus::Single);
    d.dependents.push(child(2));
    d.next_id = 3;
    d.credits.as_mut().unwrap().mortgage_credit = Some(FederalMortgageCredit {
        current: None,
        carry_three_years_prior: 600_000,
        carry_two_years_prior: 0,
        carry_prior_year: 0,
        loan: None,
    });
    let child_result = supported(&d);
    assert_eq!(value(&child_result, "mortgage_interest_credit"), 4523);
    assert_eq!(value(&child_result, "mortgage_credit_expired_2023"), 1477);
    assert_eq!(return_line(&child_result, "19"), 50_000);
    assert_eq!(return_line(&child_result, "28"), 170_000);

    d.dependents.clear();
    d.credits.as_mut().unwrap().mortgage_credit =
        Some(mortgage_certificate_credit(2000, 10000, 1_000_000));
    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(200_000);
    d.credits.as_mut().unwrap().residential_energy_carry = Some(200_000);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let ordered = supported(&d);
    assert_eq!(value(&ordered, "mortgage_interest_credit"), 2000);
    assert_eq!(value(&ordered, "dc_homebuyer_credit"), 2000);
    assert_eq!(value(&ordered, "residential_energy_credit"), 1023);
    assert_eq!(value(&ordered, "residential_energy_carryforward_2027"), 977);
    assert_eq!(value(&ordered, "minimum_tax_credit"), 0);
    assert_eq!(return_line(&ordered, "20"), 502_300);

    let mut farmland = farmland_installment_return();
    farmland.credits.as_mut().unwrap().mortgage_credit = Some(FederalMortgageCredit {
        current: None,
        carry_three_years_prior: 100_000,
        carry_two_years_prior: 0,
        carry_prior_year: 0,
        loan: None,
    });
    unsupported(&farmland, "Farmland tax installments");
}

#[test]
fn minimum_tax_credit_respects_child_adoption_and_marketplace_ordering() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 2200);
    assert_eq!(value(&r, "minimum_tax_credit"), 2823);
    assert_eq!(value(&r, "additional_child_credit"), 0);
    d.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: false,
        },
        &[(2026, 10000)],
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert!(value(&r, "adoption_nonrefundable_credit") > 0);
    d.credits.as_mut().unwrap().adoptions.clear();
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 100, 500)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit"), 8823);
    assert_eq!(return_line(&r, "17"), 600000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn minimum_tax_prior_gain_pools_are_clipped_to_exclusion_taxable_income() {
    for (preferential, combined, ordinary, expected) in [
        (50000, 70000, 100000, 1785), // All $11,900 taxable is normal gain.
        (0, 20000, 100000, 2975),     // All $11,900 taxable is the 25% pool.
        (5000, 20000, 100000, 2475),  // $5,000 at 15% and $6,900 at 25%.
        (50000, 70000, 0, 0),         // Entire taxable amount fits the zero-rate room.
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut h = minimum_tax_history();
        h.base = 10000000;
        h.exclusions.clear();
        h.regular = 0;
        h.gains = Some(minimum_tax_gains(preferential, combined, ordinary, ordinary));
        d.credits.as_mut().unwrap().minimum_tax =
            Some(crate::MinimumTaxHistories::from_current_unit(h));
        assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), expected);
        d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts.exclusions =
            vec![MinimumTaxExclusion { source: "2025 Form 6251 line 2b".into(), amount: -1190000 }];
        assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), 0);
    }
}

#[test]
fn property_return_stage_supplies_taxable_base_for_service_business_qbi() {
    let mut source = former_passive_property_scorp(10_000, -20_000, 0, 0);
    source.property_qbi = property_qbi(20_000, 0);
    source.qbi_statement = complete_ordinary_scorp(10_000, true).qbi_statement;
    source.qbi_statement.as_mut().unwrap().service = true;
    let (mut d, _) = passive_property_pool(source, 0);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert!(!r.issues.iter().any(|v| v.section == "Combined S-corporation QBI"));
    assert!(r.review_notes.iter().any(|n| {
        n.contains("combined business QBI after eligibility: current $-10000;")
            && n.contains("taxable income before QBI")
    }));
    assert_eq!(r.tax, Some(382300));
    assert_eq!(value(&r, "agi"), 50000);
    assert_eq!(value(&r, "regular_income_tax"), 3823);
    assert!(!r.return_lines.is_empty());
}

#[test]
fn employee_8919_additional_medicare_preserves_actual_withholding() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    add(&mut draft, income(omitted_employee_wages(10000)));
    let r = supported(&draft);
    assert_eq!(value(&r, "employee_wages_0_social_tax"), 0);
    assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 145);
    assert_eq!(value(&r, "additional_medicare_tax"), 180);
    assert_eq!(return_line(&r, "25d"), 9000);
    assert_eq!(return_line(&r, "23"), 32500);
}

#[test]
fn employee_8919_compensation_feeds_section179_and_archer_limits() {
    let mut draft = business_with_bonus(0, elected_asset(100000, 100000, BonusElection::ElectOut));
    draft.income[0].income = omitted_employee_wages(10000);
    let r = supported(&draft);
    assert_eq!(value(&r, "section179_business_income_limit"), 10000);
    assert_eq!(value(&r, "section179_deduction"), 10000);
    assert_eq!(value(&r, "section179_carryforward"), 90000);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(r.tax, Some(76500));
    let mut draft = complete(FederalStatus::Single);
    draft.income[0].income = omitted_employee_wages(2000);
    set_archer(
        &mut draft,
        vec![archer_contributor(Owner::Taxpayer, archer_self(3000), 1500)],
        ArcherSharing::Equal,
    );
    let r = supported(&draft);
    assert_eq!(value(&r, "archer_msa_deduction"), 1500);
    assert_eq!(value(&r, "agi"), 500);
    assert_eq!(return_line(&r, "1z"), 200000);
}

#[test]
fn filing_tables_match_every_published_irs_interval() {
    use crate::income_tax::Rates;
    use FederalStatus::*;
    // IRS i1040tt--2023/2024.pdf and 2025/2026 Publication 1040. The 2026
    // fixture was extracted from the Sept. 16 draft PDF (SHA-256
    // 407bc35ab09aaf034689341d7469b9b082cd2ceaa652e38ca5874c1dd1c3f12e).
    // interval lower/upper bounds, Single, Joint, Separate, Head tax in dollars.
    // These fixtures were extracted from IRS PDFs, not calculated by the App.
    for (year, csv) in [
        (2023, include_str!("../../../tests/fixtures/irs-tax-table-2023.csv")),
        (2024, include_str!("../../../tests/fixtures/irs-tax-table-2024.csv")),
        (2025, include_str!("../../../tests/fixtures/irs-tax-table-2025.csv")),
        (2026, include_str!("../../../tests/fixtures/irs-tax-table-2026.csv")),
    ] {
        assert_eq!(csv.lines().count(), 2062);
        let mut previous = 0;
        for row in csv.lines() {
            let v: alloc::vec::Vec<i64> = row.split(',').map(|s| s.parse().unwrap()).collect();
            assert_eq!(v.len(), 6);
            assert_eq!(v[0], previous);
            previous = v[1];
            for (column, status) in [Single, Joint, Separate, Head].into_iter().enumerate() {
                let rates = Rates::with_method(year, status, true).unwrap();
                for income in [v[0], v[1] - 1] {
                    assert_eq!(
                        rates.worksheet_ordinary_cents(income),
                        v[column + 2] * 100,
                        "{year} {status:?} income {income}"
                    );
                    assert_eq!(
                        rates.schedule_d(income, 0, 0, 0, false),
                        v[column + 2],
                        "ordinary filing-table tax: {year} {status:?} income {income}"
                    );
                }
            }
        }
        assert_eq!(previous, 100000);
    }
}

#[test]
fn preferential_worksheet_uses_tax_table_for_ordinary_component() {
    use crate::income_tax::Rates;
    // $100,000 taxable, $50,000 qualified dividends. Table ordinary tax at
    // $50,000: $6,313 / $6,059 / $5,920 / $5,755; all dividends are in the 15% band.
    for (year, expected) in [(2023, 13813), (2024, 13559), (2025, 13420), (2026, 13255)] {
        let rates = Rates::with_method(year, FederalStatus::Single, true).unwrap();
        assert_eq!(rates.schedule_d(100000, 50000, 0, 0, false), expected);
    }
    // Exactly $100,000 switches from the table to the tax computation formula.
    for (year, cents) in [(2023, 1740000), (2024, 1705300), (2025, 1691400), (2026, 1671200)] {
        let rates = Rates::with_method(year, FederalStatus::Single, true).unwrap();
        assert_eq!(rates.worksheet_ordinary_cents(100000), cents);
    }
    // The rate-schedule-only constructor remains distinct for Schedule J base years.
    assert_eq!(
        Rates::for_year(2026, FederalStatus::Single).unwrap().worksheet_ordinary_cents(50000),
        575200
    );
}

#[test]
fn draft_2026_table_routes_form_1040_line_16_at_every_edge() {
    use crate::income_tax::Rates;
    for status in [
        FederalStatus::Single,
        FederalStatus::Joint,
        FederalStatus::Separate,
        FederalStatus::Head,
        FederalStatus::Surviving,
    ] {
        let mut draft = complete(status);
        if matches!(status, FederalStatus::Head | FederalStatus::Surviving) {
            add_child(&mut draft);
        }
        if status == FederalStatus::Surviving {
            draft.setup.as_mut().unwrap().marital_state =
                MaritalState::WidowedPriorYear { could_file_joint: true };
        }
        let standard = value(&supported(&draft), "deduction");
        for taxable in [0, 4, 5, 14, 15, 24, 25, 49, 50, 99_949, 99_950, 99_999, 100_000] {
            draft.income[0] = wage(1, Owner::Taxpayer, (standard + taxable) * 100, 0);
            let report = supported(&draft);
            assert_eq!(value(&report, "taxable_income"), taxable, "{status:?}");
            let expected = if taxable < 100_000 {
                draft_2026_table_tax(taxable, status)
            } else {
                crate::kernel::ratio::dollars(
                    Rates::for_year(2026, status).unwrap().ordinary_cents(taxable),
                )
            };
            assert_eq!(return_line(&report, "16"), expected * 100, "{status:?} {taxable}");
        }
    }
}

#[test]
fn section1062_tax_worksheet_preserves_actual_taxable_income_deductions() {
    use crate::income_tax::{Preference, RegularTaxWorksheet};
    // Actual farmland-sale return has $155,048 taxable income and $24,000
    // net capital gain after $16,000 ordinary conservation recapture.
    let actual = RegularTaxWorksheet {
        taxable: 155048,
        preference: Preference { dividends: 0, gain: 24000, line18: 0, line19: 0 },
        excluded: 0,
        foreign_election: false,
        status: FederalStatus::Single,
    };
    assert_eq!(actual.tax(), 27650);
    // Form 1062 lines 6–9 subtract $40,000 gain from actual taxable income;
    // they do not recalculate QBI/AGI by deleting the source sale.
    let without = RegularTaxWorksheet {
        taxable: actual.taxable - 40000,
        preference: Preference { gain: 0, ..actual.preference },
        ..actual
    };
    assert_eq!(without.tax(), 20210);
    assert_eq!(actual.tax() - without.tax(), 7440);
    assert_ne!(without.tax(), 20978); // Full no-sale return has different QBI.
    assert_eq!(
        RegularTaxWorksheet { taxable: -10, preference: without.preference, ..without }.tax(),
        0
    );
}

#[test]
fn reusable_personal_credit_limits_preserve_order_and_adoption_child_reservation() {
    use crate::income_tax::PersonalCreditPotentials;
    let potentials = PersonalCreditPotentials {
        care: 100,
        elderly: 50,
        education: 100,
        saver: 100,
        adoption: 100,
        child: 200,
        actc_limit: 150,
    };
    let reduced = potentials.limit(200);
    assert_eq!((reduced.care, reduced.elderly, reduced.education, reduced.saver), (100, 50, 50, 0));
    assert_eq!((reduced.adoption, reduced.child, reduced.actc), (0, 0, 150));
    let actual = potentials.limit(1000);
    assert_eq!((actual.adoption, actual.child, actual.actc), (100, 200, 0));
    let coordinated = PersonalCreditPotentials {
        adoption: 200,
        child: 200,
        actc_limit: 150,
        ..Default::default()
    };
    let credit = coordinated.limit(250);
    assert_eq!((credit.adoption, credit.child, credit.actc), (200, 50, 150));
    let credit = PersonalCreditPotentials { actc_limit: 0, ..coordinated }.limit(250);
    assert_eq!((credit.adoption, credit.child, credit.actc), (50, 200, 0));
    assert_eq!(potentials.limit(0).actc, 150);
}

#[test]
fn historical_simplified_method_uses_start_date_tables() {
    for (age, old, modern) in [
        (55, 300, 360),
        (56, 260, 310),
        (60, 260, 310),
        (61, 240, 260),
        (65, 240, 260),
        (66, 170, 210),
        (70, 170, 210),
        (71, 120, 160),
    ] {
        let single = PensionDuration::Single { age };
        assert_eq!(single.payments(19961118), old);
        assert_eq!(single.payments(19961119), modern);
    }
    let joint = PensionDuration::Joint { age: 65, survivor_age: 60 };
    assert_eq!(joint.payments(19961118), 240);
    assert_eq!(joint.payments(19961119), 260);
    assert_eq!(joint.payments(19971231), 260);
    assert_eq!(joint.payments(19980101), 310);
    let mut p = pension_basis();
    p.started = 19961101;
    p.months = 2;
    p.cost = Some(2400000);
    p.history = PensionHistory::New { duration: joint };
    assert!(p.valid_for_year(1996));
    assert_eq!(p.monthly(), 10000);
    assert_eq!(p.excluded(1000000), 20000);
    p.history = PensionHistory::New { duration: PensionDuration::Fixed { payments: 120 } };
    assert!(!p.valid_for_year(1996));
    p.started = 19961119;
    assert!(p.valid_for_year(1996));
    assert_eq!(p.monthly(), 20000);
    // Current-year continuation must retain the old worksheet, not recalculate.
    p.started = 19961101;
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 2000000 };
    p.months = 12;
    assert!(p.valid());
    assert_eq!(p.excluded(1000000), 120000);
}

#[test]
fn variable_survivor_transition_uses_receipts_and_shared_remaining_cost() {
    for (earlier_gross, recovered, earlier_exclusion, own_exclusion) in [
        (90000, 1130000, 60000, 10000),
        (30000, 1130000, 30000, 40000),
        (0, 1130000, 0, 60000),
        (90000, 300000, 60000, 60000),
    ] {
        let mut a = variable_annuity();
        a.started = 20200101;
        a.period = 20260701;
        a.frequency = AnnuityFrequency::Monthly;
        a.payments = 6;
        a.earlier = Some(VariableEarlier { payments: 6, gross: earlier_gross });
        a.history = VariableHistory::Prior { annual: 120000, recovered, refigure: None };
        assert!(a.valid());
        assert!(a.schedule_valid(None));
        assert_eq!(a.earlier_recovery(), earlier_exclusion);
        assert_eq!(a.excluded(90000), own_exclusion);
        let mut d = complete(FederalStatus::Joint);
        d.people.as_mut().unwrap().spouse.as_mut().unwrap().died =
            Some(FinalReturn { date: 20260630, bonds: None, ordinary: true, estate_medical: 0 });
        let mut primary = a;
        primary.earlier = None;
        primary.period = 20260101;
        let mut entry = retirement_entry(
            Owner::Spouse,
            RetirementKind::Nonqualified,
            earlier_gross / 100,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
            &mut entry.income
        {
            data.form.taxable_not_determined = true;
            data.taxable_method = Some(TaxableMethod::VariablePayments { data: primary });
        }
        add(&mut d, entry);
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            900,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20261201,
                facts: RetirementException::Death { died: 20260630 },
                early_simple: false,
            },
        );
        if let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
            &mut entry.income
        {
            data.form.taxable_not_determined = true;
            data.taxable_method = Some(TaxableMethod::VariablePayments { data: a });
            data.form.payment_date = Some(20261201);
        }
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(
            return_line(&r, "5b"),
            earlier_gross + 90000 - earlier_exclusion - own_exclusion
        );
        assert_eq!(value(&r, "variable_annuity_3_earlier_recovery"), earlier_exclusion / 100);
        assert_eq!(restart(d.clone()).draft, d);
        d.income.swap(1, 2);
        assert_eq!(return_line(&supported(&d), "5b"), return_line(&r, "5b"));
        if let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
            &mut d.income[1].income
        {
            data.code = Some(DistributionCode::Normal { what_happened: Disposition::Kept });
        }
        unsupported(&d, "Income:");
    }
}

#[test]
fn variable_survivor_transition_covers_new_joint_contract_and_final_deduction() {
    let mut a = variable_annuity();
    a.age = 70;
    a.cost = 13200000;
    a.period = 20260701;
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 3;
    a.earlier = Some(VariableEarlier { payments: 6, gross: 400000 });
    a.history = VariableHistory::New {
        expected: VariablePayments::Joint { age: 70, survivor_age: 67, delay: 1 },
    };
    assert!(a.valid());
    assert!(!a.schedule_valid(None));
    assert!(a.schedule_valid(Some(20260930)));
    assert_eq!(a.annual(), 600000);
    assert_eq!(a.earlier_recovery(), 300000);
    assert_eq!(a.excluded(200000), 150000);
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260930, bonds: None, ordinary: true, estate_medical: 0 });
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        2000,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260901,
            facts: RetirementException::Death { died: 20260630 },
            early_simple: false,
        },
    );
    if let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
        &mut entry.income
    {
        data.form.taxable_not_determined = true;
        data.taxable_method = Some(TaxableMethod::VariablePayments { data: a });
        data.form.payment_date = Some(20260901);
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
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 50000);
    assert_eq!(value(&r, "annuity_loss_2"), 127500);
    assert_eq!(restart(d.clone()).draft, d);
    a.earlier.as_mut().unwrap().payments = 10;
    assert!(!a.valid());
    a.earlier.as_mut().unwrap().payments = 6;
    a.history = VariableHistory::New { expected: VariablePayments::Life { age: 70, delay: 1 } };
    assert!(!a.valid());
}

#[test]
fn temporary_life_table_covers_all_published_ages_and_terms() {
    use crate::annuity_tables::temporary_multiple;
    assert_eq!(temporary_multiple(65, 5), Some(49));
    assert_eq!(temporary_multiple(9, 9), Some(90));
    assert_eq!(temporary_multiple(5, 40), Some(397));
    assert_eq!(temporary_multiple(115, 40), Some(5));
    assert_eq!(temporary_multiple(4, 5), None);
    assert_eq!(temporary_multiple(116, 5), None);
    assert_eq!(temporary_multiple(65, 0), None);
    assert_eq!(temporary_multiple(65, 41), None);
    for age in 5..=115 {
        for years in 1..=40 {
            let factor = temporary_multiple(age, years).unwrap();
            assert!((1..=i64::from(years) * 10).contains(&factor));
            if years > 1 {
                assert!(factor >= temporary_multiple(age, years - 1).unwrap());
            }
            if age > 5 {
                assert!(factor <= temporary_multiple(age - 1, years).unwrap());
            }
        }
    }
}

#[test]
fn variable_temporary_life_uses_table_viii_without_lifetime_timing_adjustment() {
    for (frequency, count) in [
        (AnnuityFrequency::Annual, 1),
        (AnnuityFrequency::Semiannual, 2),
        (AnnuityFrequency::Quarterly, 4),
        (AnnuityFrequency::Monthly, 12),
    ] {
        let mut a = variable_annuity();
        a.cost = 1176000;
        a.frequency = frequency;
        a.payments = count;
        a.history = VariableHistory::New {
            expected: VariablePayments::TemporaryLife { age: 65, months: 60 },
        };
        assert!(a.valid());
        assert_eq!(a.annual(), 240000); // $11,760 / Table VIII's 4.9 years.
        let mut d = complete(FederalStatus::Single);
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            3600,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
            &mut entry.income
        {
            data.form.taxable_not_determined = true;
            data.taxable_method = Some(TaxableMethod::VariablePayments { data: a });
        }
        add(&mut d, entry);
        assert_eq!(return_line(&supported(&d), "5b"), 120000);
        assert_eq!(restart(d.clone()).draft, d);
        a.history = VariableHistory::New {
            expected: VariablePayments::TemporaryLife { age: 65, months: 12 },
        };
        assert!(!a.valid());
        a.history = VariableHistory::New {
            expected: VariablePayments::TemporaryLife { age: 70, months: 60 },
        };
        assert!(!a.valid());
    }
}

#[test]
fn retained_variable_term_handles_ending_year_without_shortfall_election() {
    for (frequency, payments) in [
        (AnnuityFrequency::Monthly, 6),
        (AnnuityFrequency::Quarterly, 2),
        (AnnuityFrequency::Semiannual, 1),
    ] {
        for fixed in [false, true] {
            let mut a = variable_annuity();
            a.started = 20250101;
            a.cost = 1176000;
            a.frequency = frequency;
            a.payments = payments;
            a.term = Some(if fixed {
                VariableTerm::Fixed { ends: 20260701 }
            } else {
                VariableTerm::TemporaryLife { ends: 20260701 }
            });
            a.history =
                VariableHistory::Prior { annual: 240000, recovered: 200000, refigure: None };
            assert!(a.valid());
            assert!(a.schedule_valid(None));
            assert_eq!(a.excluded(180000), 120000);
            let mut d = complete(FederalStatus::Single);
            let mut entry = retirement_entry(
                Owner::Taxpayer,
                RetirementKind::Nonqualified,
                1800,
                RetirementTreatment::Ordinary,
            );
            if let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
                &mut entry.income
            {
                data.form.taxable_not_determined = true;
                data.taxable_method = Some(TaxableMethod::VariablePayments { data: a });
            }
            add(&mut d, entry);
            let r = supported(&d);
            assert_eq!(return_line(&r, "5b"), 60000);
            assert_eq!(value(&r, "variable_annuity_2_remaining_basis"), 8560);
            assert!(!r.lines.iter().any(|line| line.id == "variable_annuity_2_election"));
            assert_eq!(restart(d.clone()).draft, d);
            d.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
                date: 20260630,
                bonds: None,
                ordinary: true,
                estate_medical: 0,
            });
            d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
                income_id: 2,
                started: a.started,
                cost: a.cost,
                recovered: a.recovered(),
                refunds: 0,
                eligible: true,
            });
            if fixed {
                unsupported(&d, "Annuity loss");
            } else {
                assert_eq!(value(&supported(&d), "annuity_loss_2"), 8560);
            }
            d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20260701;
            unsupported(&d, "Annuity loss");
            a.payments += 1;
            assert!(!a.schedule_valid(None));
        }
    }
}

#[test]
fn retained_variable_term_reconciles_election_and_rejects_future_periods() {
    let mut a = variable_annuity();
    a.started = 20250101;
    a.frequency = AnnuityFrequency::Monthly;
    a.payments = 6;
    a.term = Some(VariableTerm::TemporaryLife { ends: 20260701 });
    a.history = VariableHistory::Prior {
        annual: 240000,
        recovered: 200000,
        refigure: Some(VariableRefigure {
            years: 1,
            paid: 200000,
            remaining: VariablePayments::TemporaryLife { age: 66, months: 6 },
        }),
    };
    assert!(a.valid());
    assert!(a.schedule_valid(None));
    a.term = Some(VariableTerm::TemporaryLife { ends: 20260801 });
    assert!(!a.valid()); // The election cannot extend the retained term.
    a.term = Some(VariableTerm::Fixed { ends: 20260701 });
    assert!(!a.valid()); // Fixed and death-contingent terms are not interchangeable.
    a.term = None;
    a.history = VariableHistory::Prior { annual: 240000, recovered: 200000, refigure: None };
    a.period = 20260701;
    a.payments = 12;
    assert!(!a.schedule_valid(None)); // July 2026 through June 2027 is not a 2026 schedule.
    a = variable_annuity();
    a.term = Some(VariableTerm::TemporaryLife { ends: 20260701 });
    assert!(!a.valid()); // New-contract terms come from original expected-payment facts.
}

#[test]
fn single_life_guarantee_uses_refund_table_without_reducing_recovery_cap() {
    // Pub. 939 Example 1 alternative: $100/month, age 65, $21,053 cost,
    // 17-year guarantee. Table VII 14% of $20,400 = $2,856 adjustment.
    for (frequency, count, delay) in [
        (AnnuityFrequency::Monthly, 12, 1),
        (AnnuityFrequency::Quarterly, 4, 2),
        (AnnuityFrequency::Semiannual, 2, 3),
        (AnnuityFrequency::Annual, 1, 6),
    ] {
        let mut a = fixed_annuity();
        a.frequency = frequency;
        a.periodic = 120000 / i64::from(count);
        a.received = count;
        a.cost = 2105300;
        a.duration = AnnuityDuration::SingleLife { age: 65, delay, guaranteed: 2040000 };
        assert!(a.valid(120000));
        assert_eq!(a.expected(), 2400000);
        assert_eq!(a.refund_adjustment(), 285600);
        assert_eq!(a.ratio(), 758);
        assert_eq!(a.excluded(), 90960);
        let mut d = complete(FederalStatus::Single);
        let mut entry = retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Nonqualified,
            1200,
            RetirementTreatment::Ordinary,
        );
        if let FederalIncome::Retirement { data: crate::RetirementSource::R1099(data) } =
            &mut entry.income
        {
            data.form.taxable_not_determined = true;
            data.taxable_method = Some(TaxableMethod::GeneralRule { data: a });
        }
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), 29000);
        assert_eq!(value(&r, "annuity_2_refund_adjustment"), 2856);
        assert_eq!(value(&r, "annuity_2_remaining_basis"), 20143);
        assert_eq!(restart(d.clone()).draft, d);
        a.started = 20000101;
        a.recovered = 2100000;
        assert!(a.valid(120000));
        assert_eq!(a.excluded(), 5300); // Net cost, not adjusted investment.
        a.cost = 2000000; // Guarantee exceeds cost: use the lesser cost.
        a.recovered = 0;
        assert_eq!(a.refund_adjustment(), 280000);
    }
}

#[test]
fn other_nontaxable_diesel_and_kerosene_uses_share_lines_three_a_and_four_a() {
    let other = FederalOtherFuelUses {
        diesel_type_8: Some(fuel_use(100_000, 20_000)),
        kerosene_type_8: Some(fuel_use(100_000, 20_000)),
        emulsion_type_8: None,
    };
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            diesel_off_highway: Some(fuel_use(100_000, 20_000)),
            other_uses: Some(other.clone()),
            ..FederalFuelLines::default()
        }
        .into(),
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: FederalFuelLines { other_uses: Some(other), ..FederalFuelLines::default() }.into(),
        statement_complete: true,
        farm_activity: false,
    });
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));

    let mut baseline = draft.clone();
    let FederalIncome::Business { data } = &mut baseline.income[1].income else { unreachable!() };
    data.fuel_tax_credit = None;
    let FederalIncome::Partnership { data } = &mut baseline.income[2].income else {
        unreachable!()
    };
    data.fuel_tax_credit = None;
    let baseline = supported(&baseline);
    let report = supported(&draft);
    let cents = |id| line_amount(&report, id);
    assert_eq!(cents("business_2_fuel_3a"), 4_860);
    assert_eq!(cents("business_2_fuel_4a"), 2_430);
    assert_eq!(cents("partnership_3_fuel_3a"), 2_430);
    assert_eq!(cents("partnership_3_fuel_4a"), 2_430);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 12_200);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .other_uses
        .as_mut()
        .unwrap()
        .diesel_type_8
        .as_mut()
        .unwrap()
        .actual_cost = 100;
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let diesel = data
        .fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .other_uses
        .as_mut()
        .unwrap()
        .diesel_type_8
        .as_mut()
        .unwrap();
    diesel.actual_cost = 20_000;
    diesel.qualified = false;
    fuel_use_left_out(&draft);
}

#[test]
fn other_nontaxable_emulsion_use_shares_line_fourteen_a_without_reusing_gallons() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    let type_8 = FederalOtherFuelUses {
        emulsion_type_8: Some(fuel_use(100_000, 20_000)),
        ..FederalOtherFuelUses::default()
    };
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            emulsion_off_highway: Some(fuel_use(100_000, 20_000)),
            other_uses: Some(type_8.clone()),
            ..FederalFuelLines::default()
        }
        .into(),
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: FederalFuelLines { other_uses: Some(type_8), ..FederalFuelLines::default() }.into(),
        statement_complete: true,
        farm_activity: false,
    });
    let mut baseline = draft.clone();
    let mut plain_business = business.clone();
    plain_business.fuel_tax_credit = None;
    let mut plain_partner = partner.clone();
    plain_partner.fuel_tax_credit = None;
    add(&mut baseline, income(FederalIncome::Business { data: plain_business }));
    add(&mut baseline, income(FederalIncome::Partnership { data: plain_partner }));
    let baseline = supported(&baseline);
    add(&mut draft, income(FederalIncome::Business { data: business }));
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    let report = supported(&draft);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 5_900);
    for (source, expected) in [("business_2", 3_940), ("partnership_3", 1_970)] {
        assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_14a")), expected);
    }
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_8 = data.fuel_tax_credit.as_mut().unwrap().fuels.other_uses.as_mut().unwrap();
    type_8.emulsion_type_8.as_mut().unwrap().actual_cost = 1;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_8 = data.fuel_tax_credit.as_mut().unwrap().fuels.other_uses.as_mut().unwrap();
    type_8.emulsion_type_8.as_mut().unwrap().actual_cost = 20_000;
    type_8.emulsion_type_8.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
}

#[test]
fn prior_insurance_payments_become_taxable_when_main_home_is_regained_in_2026() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    add(
        &mut d,
        income(FederalIncome::CasualtyLivingInsurance {
            data: FederalCasualtyLivingInsurance {
                event_reference: "SYN-EVENT-2025-1".into(),
                loss_of_use: FederalHomeLossOfUse::Casualty,
                lost_use_on: 20250601,
                regained_use_on: Some(20260701),
                payments: vec![casualty_living_payment("SYN-2025-1", 20250615, 220_000, 5_000)],
                actual_living_expenses: 385_000,
                normal_living_expenses: 190_000,
                prior_taxable_reported: 0,
                federal_disaster_area: false,
                federal_declaration_reference: "".into(),
                confirmed: true,
            },
        }),
    );
    let result = supported(&d);
    assert_eq!(value(&result, "taxable_casualty_living_insurance"), 250);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi") + 250);
    assert_eq!(return_line(&result, "25b"), return_line(&baseline, "25b"));
    assert_eq!(
        crate::interview::document_amount(&d.income.last().unwrap().income, false),
        Some(25_000)
    );
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.regained_use_on = Some(20250701);
        data.payments.push(casualty_living_payment("SYN-2025-1", 20261001, 80_000, 0));
        data.prior_taxable_reported = 25_000;
    }
    let later_payment = supported(&d);
    assert_eq!(value(&later_payment, "taxable_casualty_living_insurance"), 800);
    assert_eq!(value(&later_payment, "agi"), value(&baseline, "agi") + 800);
    assert_eq!(estimate(&restart(d.clone()).draft), later_payment);

    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.payments[0].amount = 100_000;
        data.payments[1].amount = 120_000;
        data.prior_taxable_reported = 0;
    }
    let remaining_increase = supported(&d);
    assert_eq!(value(&remaining_increase, "taxable_casualty_living_insurance"), 250);
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.prior_taxable_reported = 1;
    }
    unsupported(&d, "Income: Synthetic document");
    assert_eq!(crate::interview::document_amount(&d.income.last().unwrap().income, false), None);
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.prior_taxable_reported = 0;
        data.regained_use_on = None;
        data.payments[1].withholding = 1_000;
    }
    let pending = supported(&d);
    assert_eq!(value(&pending, "taxable_casualty_living_insurance"), 0);
    assert_eq!(value(&pending, "agi"), value(&baseline, "agi"));
    assert_eq!(return_line(&pending, "25b"), return_line(&baseline, "25b") + 1_000);
    assert_eq!(crate::interview::document_amount(&d.income.last().unwrap().income, false), Some(0));
    assert_eq!(estimate(&restart(d.clone()).draft), pending);
}

#[test]
fn qsbs_exclusion_enters_nol_modifications_without_increasing_taxable_income() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 100_000, false)]);
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000_000, 110_000_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "prior_nol_deduction"), value(&before, "prior_nol_deduction"));
    assert_eq!(value(&after, "prior_atnol_deduction"), value(&before, "prior_atnol_deduction"));
    assert_eq!(value(&after, "nol_2025_used"), 100_000);
    assert_eq!(value(&after, "atnol_2025_used"), 100_000);
    assert_eq!(value(&after, "nol_2025_carryforward"), 0);
    assert_eq!(value(&after, "atnol_2025_carryforward"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}

#[test]
fn retained_card_issuer_claim_is_2026_accrual_income_and_refundable_credit_once() {
    let business = retained_card_issuer_business();
    let claim = business.fuel_tax_credit.as_ref().unwrap().fuels.card_issuer.as_ref().unwrap();
    assert!(claim.valid() && claim.qualified());
    assert_eq!(claim.lines(), [24_300, 0, 0, 0]);
    assert_eq!(business.receipts(), 4_024_300);

    let mut draft = complete(FederalStatus::Single);
    let mut without_credit = draft.clone();
    let mut baseline_business = business.clone();
    baseline_business.fuel_tax_credit = None;
    add(&mut without_credit, income(FederalIncome::Business { data: baseline_business }));
    let baseline = supported(&without_credit);
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let report = supported(&draft);
    assert_eq!(line_amount(&report, "business_2_card_issuer_retained_schedule_c_6"), 24_300,);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 24_300);
    assert!(value(&report, "agi") > value(&baseline, "agi"));
    assert!(report.tax.unwrap() > baseline.tax.unwrap());
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
    assert_eq!(restart(draft.clone()).draft, draft);
}

// C-fixture Form 8889 / 1099-SA (matrix row 21, 2026-09-25): a single filer
// with one W-2, self-only HDHP coverage for all twelve months, personal HSA
// contributions and a fully qualified 1099-SA distribution. Every expected
// amount below is computed by hand from the 2026 draft Form 8889/i8889 and
// the already-audited Schedule 1/2 maps (form-8889.md); none is copied from
// the App. Form 8889 lines 1, 3, 5, 6, 8, 11, 12, 14a and 14c have no
// dedicated App line id (per form-8889.md: computed inline, or folded into
// the capacity worksheet, or not exposed as a single return line); only the
// ids and 1040 lines asserted below are directly checkable.
//
// Facts: single filer born 1988-03-15 (age 38, no age-55 catch-up); W-2 box
// 1 $72,000, box 2 $8,500, boxes 3-6 consistent (6.2%/1.45% of box 1, both
// under their 2026 wage bases); self-only HDHP coverage for all 12 months;
// box 12 code W employer HSA contribution $1,000; taxpayer's own HSA
// contribution $3,000; 1099-SA box 1 $2,200, all applied to qualified
// medical expenses.
//
// Form 8889 (2026 draft, self-only limit $4,400 per Rev. Proc. 2025-19):
//   Line 1  HDHP coverage: self-only, all 12 months (not a stored amount).
//   Line 2  Contributions taxpayer made for 2026:                  $3,000
//   Line 3  Limitation, self-only, no last-month rule:              $4,400
//   Line 5  Line 3 minus line 4 (no Archer MSA):                    $4,400
//   Line 6  Line 5 (single filer, no family split):                 $4,400
//   Line 8  Line 6 plus line 7 (no catch-up at age 38):              $4,400
//   Line 9  Employer contributions (box 12 code W):                 $1,000
//   Line 11 Line 9 plus line 10 (no qualified funding distribution): $1,000
//   Line 12 Line 8 minus line 11:                                   $3,400
//   Line 13 HSA deduction, smaller of line 2 or line 12:            $3,000
//   Line 14a Total 2026 distributions (1099-SA box 1):              $2,200
//   Line 14c Line 14a minus line 14b (no rollover/timely correction): $2,200
//   Line 15 Qualified medical expenses paid with the distribution:  $2,200
//   Line 16 Taxable HSA distributions (14c minus 15):                   $0
//   Line 17b Additional 20% tax (no taxable distribution):              $0
//
// Form 1040:
//   1a  W-2 wages:                                                 $72,000
//   9   Total income (only wages):                                 $72,000
//   10  Adjustments (Schedule 1 line 13, the HSA deduction):         $3,000
//   11  AGI (9-10 = 72,000-3,000):                                  $69,000
//   12e Standard deduction (single, 2026):                         $16,100
//   15  Taxable income (11-12e = 69,000-16,100):                   $52,900
//   16  Tax: 2026 single Tax Table row $52,900-$52,950, midpoint
//       $52,925: 10% of the first $12,400 ($1,240) plus 12% of the
//       next $38,000 ($4,560) plus 22% of the remaining $2,525
//       ($555.50), rounded to the nearest dollar:                   $6,356
//   24a Total tax (no credits or other taxes):                      $6,356
//   25a W-2 withholding:                                            $8,500
//   33  Total payments (no other payments or credits):              $8,500
//   34  Overpayment (33-24a = 8,500-6,356):                         $2,144
#[test]
fn hsa_return_matches_hand_computation() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19880315;
    d.income[0] = wage(1, Owner::Taxpayer, 7_200_000, 850_000);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 300_000);
    h.employer = 100_000;
    h.employer_outside = 100_000;
    add_hsa_distribution(&mut d, Owner::Taxpayer, 220_000, vec![], None, None);
    d.adjustments.as_mut().unwrap().hsa.push(h);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 3000);
    assert_eq!(value(&r, "agi"), 69000);
    assert_eq!(value(&r, "deduction"), 16100);
    assert_eq!(value(&r, "taxable_income"), 52900);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 0);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    assert_eq!(return_line(&r, "1a"), 7_200_000);
    assert_eq!(return_line(&r, "9"), 7_200_000);
    assert_eq!(return_line(&r, "10"), 300_000);
    assert_eq!(return_line(&r, "11a / 11b"), 6_900_000);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "15"), 5_290_000);
    assert_eq!(return_line(&r, "16"), 635_600);
    assert_eq!(return_line(&r, "24a"), 635_600);
    assert_eq!(return_line(&r, "25a"), 850_000);
    assert_eq!(return_line(&r, "33"), 850_000);
    assert_eq!(return_line(&r, "34"), 214_400);
    assert_eq!(r.tax, Some(635_600));
    assert_eq!(r.payments, Some(850_000));
    assert_eq!(r.refund, Some(214_400));
    assert_eq!(r.balance, Some(0));
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

// Boundary (matrix row 21): personal $3,600 plus employer $1,000 against
// the $4,400 self-only limit. Without Form 5329 excess-accounting facts the
// App refuses the return by name (hsa.rs Coverage block: "Refusals are
// explicit issues"). Supplying those facts computes the 6% Form 5329 excise
// tax on the excess and caps the Form 8889 line 13 deduction at line 12
// ($4,400 - $1,000 = $3,400), never at the full $3,600 personal
// contribution.
#[test]
fn hsa_contributions_are_capped_at_the_self_only_limit() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19880315;
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 360_000);
    h.employer = 100_000;
    h.employer_outside = 100_000;
    d.adjustments.as_mut().unwrap().hsa.push(h);
    unsupported(&d, "HSA");

    d.adjustments.as_mut().unwrap().hsa[0].excess_accounting =
        Some(hsa_excess_accounting(100_000_000, None, 0, 0, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 3400);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 200);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 12);
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (matrix row 21): the $1,000 age-55 catch-up (Form 8889 line 7)
// raises the self-only limit from $4,400 to $5,400, but only for an owner
// who is 55 by the end of 2026 (born on or before January 1, 1972, per hsa.rs catchup_age). A
// filer born in 1970 gets the full $5,400 of room; the identical $5,400 of
// personal contributions from a filer too young for the catch-up is
// refused as excess.
#[test]
fn age_55_catch_up_adds_1000() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19880315;
    let h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 540_000);
    d.adjustments.as_mut().unwrap().hsa.push(h);
    unsupported(&d, "HSA");

    d.people.as_mut().unwrap().taxpayer.birth_date = 19700601;
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 5400);
    assert_eq!(restart(d.clone()).draft, d);

    d.adjustments.as_mut().unwrap().hsa[0].personal += 100;
    unsupported(&d, "HSA");
}

// IRC 223(b)(3), day-exact (Pub. 501): an owner born 1972-01-01 attains 55 on
// 2026-12-31 (the 55th birthday is 2027-01-01), so the catch-up applies and
// $5,400 is allowed; born 1972-01-02 it is attained on 2027-01-01, so the
// same $5,400 of personal contributions is $1,000 over the $4,400 limit.
#[test]
fn age_55_catch_up_is_day_exact_for_a_january_1_birth() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19720101;
    let h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 540_000);
    d.adjustments.as_mut().unwrap().hsa.push(h);
    assert_eq!(value(&supported(&d), "adjustments"), 5400);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19720102;
    unsupported(&d, "HSA");
}

// T2-27: Form 8962 Part IV shares, separate returns, the below-poverty
// exception, dependent filers, the named refusals, and the Pub. 974
// self-employed health insurance coordination. Every expected amount is
// worked by hand from the 2025 Form 8962 instructions and Pub. 974 with the
// 2026 figures of Rev. Proc. 2025-25 (Table 2 bands, 2025 poverty guidelines,
// no repayment limitation after P.L. 119-21 section 71305).
fn shared_month(
    month: u32,
    premium: i64,
    benchmark: i64,
    advance: i64,
    allocation: MarketplaceAllocation,
) -> MarketplaceMonth {
    let mut m = marketplace_month(month, premium, benchmark, advance);
    m.policies[0].allocation = allocation;
    m
}
fn marketplace_claim_mut(d: &mut FederalDraft) -> &mut MarketplaceClaim {
    &mut d.credits.as_mut().unwrap().marketplace
}

#[test]
fn marketplace_part_iv_shared_policy_with_the_other_parent_uses_the_monthly_lines() {
    // Single, wages $31,300; one-person family, poverty line $15,650: 200%,
    // applicable 6.60%; annual contribution 31,300 x 0.066 = 2,065.80 ->
    // $2,066; monthly 2,066 / 12 = $172. The policy covers the filer and a
    // child the other parent claims; the agreed (and head-count) share is 50%
    // (allocation situation 4, same share in columns (e), f, g).
    // Monthly: premium 800 x 50% = 400; benchmark 700 x 50% = 350;
    // advance 450 x 50% = 225. Column (e) = min(400, 350 - 172) = 178.
    // Line 24 = 12 x 178 = 2,136; line 25 = 12 x 225 = 2,700;
    // excess advance (line 27, Schedule 2 line 1a) = 564.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    *marketplace_claim_mut(&mut d) = marketplace_claim(
        &mut d,
        (1..=12)
            .map(|m| shared_month(m, 800, 700, 450, MarketplaceAllocation::Shared { percent: 50 }))
            .collect(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "marketplace_annual_contribution"), 2066);
    assert_eq!(value(&r, "premium_tax_credit"), 2136);
    assert_eq!(value(&r, "premium_credit_advance"), 2700);
    assert_eq!(value(&r, "premium_credit_repayment"), 564);
    assert_eq!(value(&r, "premium_credit_refundable"), 0);
    assert_eq!(return_line(&r, "17"), 56_400);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn marketplace_part_iv_former_spouse_months_then_own_policy() {
    // Divorced in 2026 (allocation situation 1). January-June: the family
    // policy, 60% agreed: premium 1,000 -> 600, benchmark 900 -> 540, advance
    // 600 -> 360. July-December: own policy, premium 500, benchmark 450,
    // advance 250. Contribution as above, $172 a month.
    // Jan-Jun column (e) = min(600, 540 - 172) = 368, x 6 = 2,208.
    // Jul-Dec column (e) = min(500, 450 - 172) = 278, x 6 = 1,668.
    // Line 24 = 3,876; line 25 = 6 x 360 + 6 x 250 = 3,660; net credit 216.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    *marketplace_claim_mut(&mut d) = marketplace_claim(
        &mut d,
        (1..=12)
            .map(|m| {
                if m <= 6 {
                    shared_month(
                        m,
                        1000,
                        900,
                        600,
                        MarketplaceAllocation::FormerSpouse { percent: 60 },
                    )
                } else {
                    marketplace_month(m, 500, 450, 250)
                }
            })
            .collect(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 3876);
    assert_eq!(value(&r, "premium_credit_advance"), 3660);
    assert_eq!(value(&r, "premium_credit_refundable"), 216);
    assert_eq!(return_line(&r, "31"), 21_600);
}

#[test]
fn marketplace_part_iv_share_limits_zero_and_one_hundred_percent() {
    // Boundary of column (e)-(g): a 100% share is the whole policy on the
    // monthly lines: column (e) = min(800, 700 - 172) = 528, x 12 = 6,336,
    // advance 12 x 450 = 5,400, net 936. A 0% share leaves nothing: no
    // premium, benchmark or advance, so line 24 and line 25 are zero.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    for (percent, credit, advance) in [(100, 6336, 5400), (0, 0, 0)] {
        *marketplace_claim_mut(&mut d) = marketplace_claim(
            &mut d,
            (1..=12)
                .map(|m| shared_month(m, 800, 700, 450, MarketplaceAllocation::Shared { percent }))
                .collect(),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "premium_tax_credit"), credit);
        assert_eq!(value(&r, "premium_credit_advance"), advance);
        assert_eq!(value(&r, "premium_credit_refundable"), credit - advance);
    }
    // An allocated benchmark cannot be combined with another policy's
    // benchmark in the same month (Part IV column (f)).
    let second_id = d.next_id;
    d.next_id += 1;
    if let MarketplaceClaim::Yes { facts, .. } = marketplace_claim_mut(&mut d) {
        let mut second = facts.forms[0].clone();
        second.id = second_id;
        second.allocation.clear();
        facts.forms.push(second);
    }
    unsupported(&d, "Marketplace");
}

#[test]
fn marketplace_separate_return_repays_its_half_without_the_abuse_exception() {
    // Married filing separately, wages $31,300, the joint policy covered the
    // spouse (allocation situation 2). Without Exception 2 the filer is not an
    // applicable taxpayer: no credit; column (g) 50% of the 500 advance =
    // 250 a month, 3,000 repaid in full (no 2026 limitation).
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    *marketplace_claim_mut(&mut d) = marketplace_claim(
        &mut d,
        (1..=12).map(|m| shared_month(m, 800, 380, 500, MarketplaceAllocation::Spouse)).collect(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "premium_credit_advance"), 3000);
    assert_eq!(value(&r, "premium_credit_repayment"), 3000);
    // Exception 2 (line A): columns (e) and (g) 50%, the entered benchmark is
    // the filer's own coverage family's. One-person family at 200%: $172 a
    // month. Column (e) = min(400, 380 - 172) = 208, x 12 = 2,496; advance
    // 3,000; excess 504.
    if let MarketplaceClaim::Yes { facts, .. } = marketplace_claim_mut(&mut d) {
        facts.separate = SeparateRelief::Yes;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 2496);
    assert_eq!(value(&r, "premium_credit_repayment"), 504);
    if let MarketplaceClaim::Yes { facts, .. } = marketplace_claim_mut(&mut d) {
        facts.separate = SeparateRelief::NotSeparate;
    }
    // Line A not known: taken as No, with a warning.
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "premium_credit_repayment"), 3000);
    assert!(noted(&r, "WARN_MARKETPLACE_SEPARATE_RELIEF_ASSUMED_NO"), "{r:?}");
    // Allocation situation 2 is for separate returns only.
    let mut joint = complete(FederalStatus::Joint);
    joint.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    *marketplace_claim_mut(&mut joint) = marketplace_claim(
        &mut joint,
        (1..=12).map(|m| shared_month(m, 800, 380, 500, MarketplaceAllocation::Spouse)).collect(),
    );
    unsupported(&joint, "Marketplace");
}

#[test]
fn marketplace_unsubsidized_shared_policy_allocates_only_the_premium() {
    // Allocation situation 3: no advance credit; the premium share is the own
    // family's benchmark over both families' (here 67%), column (e) only.
    // Premium 900 x 67% = 603; own benchmark 500; column (e) =
    // min(603, 500 - 172) = 328, x 12 = 3,936, all refundable.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    *marketplace_claim_mut(&mut d) = marketplace_claim(
        &mut d,
        (1..=12)
            .map(|m| shared_month(m, 900, 500, 0, MarketplaceAllocation::NoAdvance { percent: 67 }))
            .collect(),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 3936);
    assert_eq!(value(&r, "premium_credit_refundable"), 3936);
    // The client rule MARKETPLACE_ADVANCE_ON_UNSUBSIDIZED_SHARE, re-checked
    // by the stored-draft validity.
    if let MarketplaceClaim::Yes { facts, .. } = marketplace_claim_mut(&mut d) {
        let paper = &mut facts.forms[0].form;
        paper.month_mut(1).advance = Some(100);
    }
    crate::tests::support::client::retained_marketplace_conflict(&d);
}

#[test]
fn marketplace_below_poverty_credit_needs_an_advance_payment() {
    // Line 5 boundary. Premium 500, benchmark 600 for twelve months (line 11).
    // $15,650 is 100% of the one-person poverty line: applicable 2.10%,
    // contribution 15,650 x 0.021 = 328.65 -> 329; credit = min(6,000,
    // 7,200 - 329) = 6,000 with or without advance credit.
    // $15,649 is 99%: allowed only because advance credit (400 a month,
    // 4,800) was paid: contribution 15,649 x 0.021 = 328.63 -> 329; credit
    // 6,000, net 1,200. With no advance credit, no credit.
    for (wages, advance, credit) in
        [(1_565_000, 0, 6000), (1_565_000, 400, 6000), (1_564_900, 400, 6000), (1_564_900, 0, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        *marketplace_claim_mut(&mut d) = marketplace_claim(
            &mut d,
            (1..=12).map(|m| marketplace_month(m, 500, 600, advance)).collect(),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "marketplace_annual_contribution"), 329);
        assert_eq!(value(&r, "premium_tax_credit"), credit);
        assert_eq!(value(&r, "premium_credit_refundable"), (credit - advance * 12).max(0));
    }
}

#[test]
fn marketplace_dependent_filer_claimed_by_a_parent_takes_no_credit() {
    // A filer claimed by another taxpayer is outside every tax family of his
    // own: no Form 8962; the claimant reconciles the advance credit.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    *marketplace_claim_mut(&mut d) =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 600, 400)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "premium_credit_repayment"), 0);
    assert_eq!(value(&r, "premium_credit_refundable"), 0);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    // Being unclaimed is a different source responsibility, not a blanket
    // refusal and not a fictional one-person tax family.
    let facts = marketplace_facts_mut(&mut d);
    facts
        .memberships
        .push(MarketplacePersonFamily { person: -1, family: MarketplaceNonreturnFamily::NoFamily });
    facts
        .attestations
        .push(MarketplaceAttestation { person: -1, attester: MarketplaceAttester::ThisReturn });
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "premium_credit_repayment"), 4800);
    assert_eq!(value(&r, "marketplace_poverty_line"), 0);
}

#[test]
fn marketplace_unclaimed_filer_reconciles_only_actual_attester_advances() {
    // Approved Unit24 fixtureK; Reg1.36B-4(a)(1)(ii)(C). A claimable filer
    // in no tax family gets no PTC, even if they attested to their own coverage.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    *marketplace_claim_mut(&mut d) = marketplace_claim(
        &mut d,
        (1..=12).map(|m| marketplace_month(m, 1000, 1200, 600)).collect(),
    );
    let facts = marketplace_facts_mut(&mut d);
    facts
        .memberships
        .push(MarketplacePersonFamily { person: -1, family: MarketplaceNonreturnFamily::NoFamily });
    facts
        .attestations
        .push(MarketplaceAttestation { person: -1, attester: MarketplaceAttester::ThisReturn });
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "premium_credit_advance"), 7200);
    assert_eq!(value(&r, "premium_credit_repayment"), 7200);
    assert_eq!(value(&r, "premium_credit_refundable"), 0);
    assert!(crate::marketplace::family_people(&d).is_empty());
    let family = d.next_id;
    d.next_id += 1;
    let facts = marketplace_facts_mut(&mut d);
    facts
        .other_families
        .push(MarketplaceFamily { id: family, label: "Actual attesting parent".into() });
    facts.attestations[0].attester = MarketplaceAttester::OtherFamily { family };
    let r = supported(&d);
    assert_eq!(value(&r, "premium_credit_advance"), 0);
    assert_eq!(value(&r, "premium_credit_repayment"), 0);
    marketplace_facts_mut(&mut d).attestations[0].attester = MarketplaceAttester::Unknown;
    unsupported(&d, "Marketplace source");
    let facts = marketplace_facts_mut(&mut d);
    for form in &mut facts.forms {
        for month in 1..=12 {
            if let Some(row) = form.form.month_slot(month) {
                row.advance = Some(0);
            }
        }
    }
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "premium_credit_advance"), 0);
    assert_eq!(value(&r, "premium_credit_repayment"), 0);
    // One material month cannot be erased by eleven zero-advance months.
    marketplace_facts_mut(&mut d).forms[0].form.month_mut(1).advance = Some(1);
    unsupported(&d, "Marketplace source");
}

#[test]
fn marketplace_dependent_qsehra_is_an_explicit_reconciliation_boundary() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let employee = d.dependents[0].id;
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, vec![marketplace_month(1, 500, 600, 0)]);
    marketplace_facts_mut(&mut d).dependent_qsehra_employees = vec![employee];
    let r = estimate(&d);
    assert_ne!(r.state, ResultState::EstimatedForSupportedScope);
    assert!(r.issues.iter().any(|i| i.message.contains("employee's own affordability")));
}

#[test]
fn marketplace_retained_dependent_qsehra_requires_the_actual_current_tax_family() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let employee = d.dependents[0].id;
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, vec![marketplace_month(1, 500, 600, 400)]);
    marketplace_facts_mut(&mut d).dependent_qsehra_employees = vec![employee];
    assert!(d.valid_stored());
    let forms =
        crate::marketplace::facts(&d.credits.as_ref().unwrap().marketplace).unwrap().forms.clone();
    d.dependents[0].competing = CompetingClaim::Other;
    // Current People still contains the employee, but this return no longer
    // claims them. A forged retained disclosure cannot borrow their family.
    assert!(!d.valid_stored());
    all_ticked(&mut d);
    crate::interview::prune(&mut d);
    let facts = crate::marketplace::facts(&d.credits.as_ref().unwrap().marketplace).unwrap();
    assert!(facts.dependent_qsehra_employees.is_empty());
    assert_eq!(facts.forms, forms); // Actual source premiums/APTC survive.
    assert!(d.valid_stored());
}

fn marketplace_business(profit: i64, months: core::ops::RangeInclusive<u32>) -> FederalIncome {
    let mut business = cash_business(profit);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        months: months
            .map(|month| BusinessHealthMonth {
                month,
                premiums: 0,
                child_increase: 0,
                employer_eligible: false,
                reimbursed: None,
            })
            .collect(),
        long_term_care: vec![],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    business
}

#[test]
fn marketplace_self_employed_premiums_use_the_pub_974_simplified_method() {
    // Single, Schedule C profit $40,000, no other income. Schedule SE:
    // 40,000 x 0.9235 = 36,940; 36,940 x 12.4% = 4,580.56 -> 4,581 and
    // x 2.9% = 1,071.26 -> 1,071; tax 5,652; deduction 2,826.
    // Marketplace, all year under the business: premium 600, benchmark 550,
    // advance 300 a month. Worksheet W: line 1 7,200; line 2 3,600;
    // line 13 = 40,000 - 2,826 = 37,174; line 14 0; line 15 37,174.
    // Worksheet X line 30 = min(7,200, 37,174) = 7,200 (full 2026 repayment).
    // Step 1: AGI = 40,000 - 2,826 - 7,200 = 29,974 (191%: 4.19% +
    // 41/50 x 2.41% = 6.1662% -> 6.17%); contribution 29,974 x 0.0617 =
    // 1,849.40 -> 1,849. Step 2: credit = min(7,200, 6,600 - 1,849) = 4,751.
    // Step 3: line 6 = 12/12 x 4,751; line 7 = 7,200 - 4,751 = 2,449;
    // line 9 = min(2,449, 7,200) = 2,449, Schedule 1 line 17.
    // Step 4: AGI = 40,000 - 2,826 - 2,449 = 34,725 (221%: 6.60% + 21/50 x
    // 1.84% = 7.3728% -> 7.37%); contribution 34,725 x 0.0737 = 2,559.23 ->
    // 2,559; credit = min(7,200, 6,600 - 2,559) = 4,041; advance 3,600; net
    // credit 441. Deduction plus credit = 6,490, within the 7,200 premiums.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    annual_marketplace_businesses(&mut d);
    let r = supported(&d);
    let id = d.income[0].id;
    assert_eq!(value(&r, "marketplace_specified_premiums"), 7200);
    assert_eq!(value(&r, "marketplace_specified_maximum"), 7200);
    assert_eq!(value(&r, "marketplace_specified_deduction"), 2449);
    assert_eq!(value(&r, &alloc::format!("business_{id}_health_insurance")), 2449);
    assert_eq!(value(&r, "agi"), 34725);
    assert_eq!(value(&r, "marketplace_household_income"), 34725);
    assert_eq!(value(&r, "premium_tax_credit"), 4041);
    assert_eq!(value(&r, "premium_credit_refundable"), 441);
    assert_eq!(restart(d.clone()).draft, d);
}

// Derived2026 counterpart to the Simplified fixture above, not an IRS
// published2026 oracle. Pub974(2025) pp52–54 requires cent work and selects
// Step4 credit with Step5 deduction. Existing SE owner gives half-tax2826,
// leaving37174. Independent cent worksheet iterations (deduction/credit):
// 7200/4750.60,2449.40/4040.80,3159.20/4140.73,3059.27/4133.51,
// 3066.49/4134.03 =>next deduction3065.97. Both changes52c<100c.
// Recomputing PTC using3065.97 would give4133.99; it is NOT selected.
#[test]
fn iterative_marketplace_selects_step_four_credit_and_step_five_deduction() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let result = supported(&d);
    assert_eq!(value(&result, "marketplace_specified_deduction"), 3066);
    assert_eq!(value(&result, "premium_tax_credit"), 4134);
    assert_eq!(value(&result, "premium_credit_refundable"), 534);
    assert_eq!(value(&result, "agi"), 34108);
    for (id, expected_cents) in [
        ("marketplace_iterative_credit_work", 413_403),
        ("marketplace_iterative_deduction_work", 306_597),
    ] {
        assert_eq!(line_amount(&result, id), expected_cents);
    }
    assert!(
        result
            .review_notes
            .iter()
            .all(|note| note.display.key != "NOTE_HEALTH_PTC_SIMPLIFIED_FALLBACK")
    );
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Derived2026 cent boundary, not a published IRS numerical example.
// Pub974(2025 final), printed52–54, requires dollars/cents throughout the
// Iterative method: https://www.irs.gov/pub/irs-pdf/p974.pdf . Ordinary
// premium49c plus qualified LTC49c must consume98c of the SAME trade limit,
// not zero dollars independently. Existing halfSE2826 leaves37174; work
// base after nonspecified costs37173.02. Independent Decimal worksheet:
// credit4750.66,4040.86,4140.80,4133.57,4134.09; selected specified3065.91.
// Final combined3065.91+0.98=3066.89 rounds3067 once, AGI34107.
#[test]
fn iterative_marketplace_retains_manual_ordinary_and_care_cents_before_shared_rounding() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    annual_marketplace_businesses(&mut d);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessHealthInsurance::Plan { months, long_term_care, .. } = &mut data.health_insurance
    else {
        unreachable!()
    };
    months[0].premiums = 49;
    let mut care = form_7206_care(-1, 0);
    care.months[0].premiums = 49;
    long_term_care.push(care);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 413409);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 306591);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 3067);
    assert_eq!(value(&r, "agi"), 34107);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Pub974(2025 final), printed52–54: no interim whole-dollar rounding.
// https://www.irs.gov/pub/irs-pdf/p974.pdf . Derived current2026 counterpart
// to the40k business fixture: admitted W2 wages49c plus taxable interest49c
// round to zero on their separate filing rows, but add98c to method income.
// Independent Decimal work base37174.98 selects credit4133.96/spec3066.04.
#[test]
fn iterative_marketplace_retains_admitted_wage_and_interest_source_cents() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    add(&mut d, wage(0, Owner::Taxpayer, 49, 0));
    let mut interest = savings_income(0, 0);
    let FederalIncome::Interest { data } = &mut interest else { unreachable!() };
    let crate::InterestSource::Int1099(data) = data else { unreachable!() };
    data.form.interest_income = 49;
    add(&mut d, income(interest));
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "wages"), 0);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 413396);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 306604);
    assert_eq!(value(&r, "agi"), 34108);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Derived complete source case, not an IRS published dollar oracle. The
// required-to-file dependent's actual external return AGI15000.49 (unearned
// income, no other MAGI additions) retains its49c. Two-person poverty21150;
// work base52174.49, same6600 benchmark/7200premium, selected2975.05/4224.95.
// Pub974(2025final) pp52–54 and Form8962 dependent household-income rule:
// https://www.irs.gov/publications/p974 . No dependent tax engine is created.
#[test]
fn iterative_marketplace_retains_required_dependent_return_cents() {
    let mut d = empty_return(FederalStatus::Single);
    add_child(&mut d);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    let mut claim =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 600, 550, 300)).collect());
    let MarketplaceClaim::Yes { facts } = &mut claim else { unreachable!() };
    facts.dependents.push(MarketplaceDependent {
        person: d.dependents[0].id,
        agi: 1500049,
        exempt_interest: 0,
        foreign_exclusion: 0,
        social: 0,
        taxable_social: 0,
    });
    *marketplace_claim_mut(&mut d) = claim;
    assign_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 297505);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 422495);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Derived current2026 cutoff cycle: existing annual owner income63000
// before specified premiums. Deduction1000.08 lowers household income below
//62600, allowing the full1000.08 credit; zero deduction puts income above
//400%, so credit becomeszero. Pub974 p54 requires another allowed method.
// A complete fresh Simplified sequence gives the same pair as explicitly
// choosing Simplified, rather than reporting either cyclic intermediate.
#[test]
fn iterative_marketplace_cycle_restarts_the_complete_simplified_method() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(savings_income(53707, 0)));
    add(&mut d, income(marketplace_business(10000, 1..=12)));
    let months = (1..=12)
        .map(|m| {
            let mut row = marketplace_month(m, 83, 1000, 0);
            row.policies[0].premium = 8_334;
            row
        })
        .collect();
    *marketplace_claim_mut(&mut d) = marketplace_claim(&mut d, months);
    assign_marketplace_businesses(&mut d);
    let simplified = supported(&d);
    assert_eq!(value(&simplified, "marketplace_specified_deduction"), 0);
    assert_eq!(value(&simplified, "premium_tax_credit"), 0);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let iterative = supported(&d);
    for id in [
        "marketplace_specified_deduction",
        "premium_tax_credit",
        "premium_credit_repayment",
        "premium_credit_refundable",
        "agi",
        "total_tax",
    ] {
        assert_eq!(value(&iterative, id), value(&simplified, id), "{id}");
    }
    assert_eq!(
        iterative
            .review_notes
            .iter()
            .filter(|note| note.display.key == "NOTE_HEALTH_PTC_SIMPLIFIED_FALLBACK")
            .count(),
        1
    );
    assert!(!iterative.lines.iter().any(|line| line.id == "marketplace_iterative_credit_work"));
}

#[test]
fn iterative_marketplace_business_order_preserves_the_shared_credit_and_trade_caps() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(10000, 1..=12)));
    add(&mut d, income(marketplace_business(30000, 1..=12)));
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let first = supported(&d);
    d.income.reverse();
    d.adjustments.as_mut().unwrap().health_marketplace.policies.reverse();
    let reversed = supported(&d);
    for id in [
        "marketplace_specified_deduction",
        "premium_tax_credit",
        "agi",
        "marketplace_iterative_credit_work",
        "marketplace_iterative_deduction_work",
    ] {
        assert_eq!(line_amount(&first, id), line_amount(&reversed, id), "{id}");
    }
    let business_total: i64 = d
        .income
        .iter()
        .map(|entry| value(&reversed, &alloc::format!("business_{}_health_insurance", entry.id)))
        .sum();
    assert_eq!(business_total, value(&reversed, "marketplace_specified_deduction"));
}

#[test]
fn marketplace_self_employed_earned_income_limit_binds_worksheet_x() {
    // Boundary: Worksheet W line 15 below line 1. Profit $6,000: SE
    // 6,000 x 0.9235 = 5,541; 687.08 -> 687 and 160.69 -> 161; tax 848;
    // deduction 424. Line 15 = 6,000 - 424 = 5,576;
    // line 30 = min(7,200, 5,576) = 5,576. Step 1 AGI = 0: below the poverty
    // line, allowed because advance credit was paid, 2.10% of 0 = 0; Step 2
    // credit = min(7,200, 6,600) = 6,600. Step 3 line 7 = 7,200 - 6,600 =
    // 600; line 9 = 600. Step 4 AGI = 6,000 - 424 - 600 = 4,976 (31%, still
    // 2.10% with advance credit): contribution 104.50 -> 104; credit =
    // min(7,200, 6,600 - 104) = 6,496; net 6,496 - 3,600 = 2,896.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(6000, 1..=12)));
    annual_marketplace_businesses(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "marketplace_specified_maximum"), 5576);
    assert_eq!(value(&r, "marketplace_specified_deduction"), 600);
    assert_eq!(value(&r, "agi"), 4976);
    assert_eq!(value(&r, "premium_tax_credit"), 6496);
    assert_eq!(value(&r, "premium_credit_refundable"), 2896);
}

#[test]
fn marketplace_self_employed_part_year_uses_column_e_of_those_months() {
    // Boundary of the Step 3 caution: specified premiums for 6 of 12 months
    // (self-employed July-December) and monthly column (e) amounts that
    // differ (benchmark 550 January-June, 650 July-December; premium 600,
    // advance 300). Profit $40,000, SE deduction 2,826 as above.
    // Worksheet W line 1 = 6 x 600 = 3,600; line 30 = 3,600.
    // Step 1 AGI = 40,000 - 2,826 - 3,600 = 33,574 (214%: 6.60% + 14/50 x
    // 1.84% = 7.1152% -> 7.12%); contribution 33,574 x 0.0712 = 2,390.47 ->
    // 2,390; monthly 199.17 -> 199. Column (e): Jan-Jun min(600, 550 - 199)
    // = 351; Jul-Dec min(600, 650 - 199) = 451. Step 3 line 6 = 6 x 451 =
    // 2,706; line 7 = 3,600 - 2,706 = 894; line 9 = 894.
    // Step 4 AGI = 40,000 - 2,826 - 894 = 36,280 (231%: 6.60% + 31/50 x
    // 1.84% = 7.7408% -> 7.74%); contribution 36,280 x 0.0774 = 2,808.07 ->
    // 2,808; monthly 234. Column (e): 550 - 234 = 316 and 650 - 234 = 416;
    // credit 6 x 316 + 6 x 416 = 4,392; advance 3,600; net 792.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 7..=12)));
    *marketplace_claim_mut(&mut d) = marketplace_claim(
        &mut d,
        (1..=12).map(|m| marketplace_month(m, 600, if m <= 6 { 550 } else { 650 }, 300)).collect(),
    );
    assign_marketplace_businesses(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "marketplace_specified_premiums"), 3600);
    assert_eq!(value(&r, "marketplace_specified_deduction"), 894);
    assert_eq!(value(&r, "agi"), 36280);
    assert_eq!(value(&r, "premium_tax_credit"), 4392);
    assert_eq!(value(&r, "premium_credit_refundable"), 792);
    // A Marketplace month under a second business needs Pub. 974's
    // allocation among trades or businesses.
    add(&mut d, income(marketplace_business(10000, 1..=1)));
    assign_marketplace_businesses(&mut d);
    let shared = supported(&d);
    assert!(value(&shared, "marketplace_specified_premiums") > 3600);
    let total: i64 = d
        .income
        .iter()
        .map(|entry| value(&shared, &alloc::format!("business_{}_health_insurance", entry.id)))
        .sum();
    assert_eq!(total, value(&shared, "marketplace_specified_deduction"));
}

// A plan established under a cash business: `months` at `premium` dollars a
// month, with `child` dollars of each month's premium the increase for a
// nondependent child under 27, and the months in `employer` eligible for an
// employer-subsidized plan (T2-43).
fn form_7206_business(
    profit: i64,
    premium: i64,
    child: i64,
    employer: &[u32],
    long_term_care: Vec<BusinessCarePremium>,
) -> FederalIncome {
    let mut business = cash_business(profit);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        months: (1..=12)
            .map(|month| BusinessHealthMonth {
                month,
                premiums: premium * 100,
                child_increase: child * 100,
                employer_eligible: employer.contains(&month),
                reimbursed: None,
            })
            .collect(),
        long_term_care,
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    business
}

fn form_7206_care(person: i64, monthly: i64) -> BusinessCarePremium {
    BusinessCarePremium {
        person,

        months: (1..=12)
            .map(|month| BusinessCareMonth {
                month,
                premiums: monthly * 100,
                employer_eligible: false,
                reimbursed: None,
            })
            .collect(),
        qualified: CareContractQualification::Qualified,
    }
}

#[test]
fn form_7206_by_hand_two_businesses_long_term_care_and_child_premiums() {
    // Single filer, two Schedule C businesses: A (the plan's business) net
    // profit $24,000, B $16,000. Schedule SE: 40,000 x 0.9235 = 36,940;
    // 36,940 x 12.4% = 4,580.56 -> 4,581; x 2.9% = 1,071.26 -> 1,071; SE tax
    // 5,652; Schedule 1 line 15 = 2,826.
    // Plan under A: $650 a month all year (7,800), of which $50 a month is
    // the increase for a nondependent child under 27; November and December
    // eligible for the spouse's employer plan.
    // Form 7206 (2026 draft): line 1 = 10 x 650 = 6,500 (the two employer
    // months excluded). Line 2: "Pat", age 55, $200 x 12 = 2,400 paid, cap
    // (age 51-60) 1,860 -> 1,860. Line 3 = 8,360. Line 4 = 24,000; line 5 =
    // 40,000; line 6 = 0.600; line 7 = 2,826 x 0.6 = 1,695.60 -> 1,696;
    // line 8 = 22,304; line 9 = 0; line 10 = line 13 = 22,304;
    // line 14 = min(8,360, 22,304) = 8,360 -> Schedule 1 line 17.
    // Schedule A: the excluded months' 1,300 less the child's 100 (not a
    // medical dependent) = 1,200; long-term care above the cap adds nothing.
    // AGI = 40,000 - 2,826 - 8,360 = 28,814.
    let mut d = empty_return(FederalStatus::Single);
    set_age(&mut d, -1, 55);
    add(
        &mut d,
        income(form_7206_business(24000, 650, 50, &[11, 12], vec![form_7206_care(-1, 200)])),
    );
    add(&mut d, income(cash_business(16000)));
    let r = supported(&d);
    let id = d.income[0].id;
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 2826);
    assert_eq!(value(&r, &alloc::format!("business_{id}_long_term_care_deduction")), 1860);
    assert_eq!(value(&r, &alloc::format!("business_{id}_health_insurance")), 8360);
    assert_eq!(value(&r, &alloc::format!("business_{id}_medical_premiums")), 1200);
    assert_eq!(value(&r, &alloc::format!("business_{id}_excluded_child_premiums")), 100);
    assert_eq!(value(&r, "long_term_care_1_eligible"), 1860);
    assert_eq!(value(&r, "long_term_care_1_medical"), 0);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 8360);
    assert_eq!(value(&r, "agi"), 28814);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn form_7206_line_2_age_cap_is_shared_across_two_businesses() {
    // Boundary: "the total of the amounts entered for each person on line 2
    // of all Form(s) 7206 can't be more than the appropriate limit". Two
    // businesses of $20,000 each, one long-term-care policy on "Pat" (age
    // 65, cap 4,960) under each, $250 a month (3,000) apiece. Form 7206 A:
    // line 2 = 3,000; Form 7206 B: line 2 = 4,960 - 3,000 = 1,960. SE tax
    // 5,652 on 40,000 as above; each line 13 = 20,000 - 1,413 = 18,587, so
    // line 3 binds both. Schedule 1 line 17 = 4,960; AGI = 40,000 - 2,826 -
    // 4,960 = 32,214. The 1,040 above the cap is not Schedule A medical.
    let mut d = empty_return(FederalStatus::Single);
    set_age(&mut d, -1, 65);
    for _ in 0..2 {
        add(&mut d, income(form_7206_business(20000, 0, 0, &[], vec![form_7206_care(-1, 250)])));
    }
    let r = supported(&d);
    let (a, b) = (d.income[0].id, d.income[1].id);
    assert_eq!(value(&r, &alloc::format!("business_{a}_long_term_care_deduction")), 3000);
    assert_eq!(value(&r, &alloc::format!("business_{b}_long_term_care_deduction")), 1960);
    assert_eq!(value(&r, "long_term_care_1_eligible"), 4960);
    assert_eq!(value(&r, "long_term_care_1_medical"), 0);
    assert_eq!(value(&r, "long_term_care_1_excluded"), 1040);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 4960);
    assert_eq!(value(&r, "agi"), 32214);
}

#[test]
fn form_7206_line_13_binds_after_the_sep_on_line_9() {
    // Boundary: line 14 is the smaller of line 3 and line 13 with the owner
    // SEP on line 9. One business, net profit $20,000: 20,000 x 0.9235 =
    // 18,470; x 12.4% = 2,290.28 -> 2,290; x 2.9% = 535.63 -> 536; SE tax
    // 2,826; Schedule 1 line 15 = 1,413. SEP $3,000 at 25%: (20,000 - 1,413)
    // x 25/125 = 3,717.40 allows it. Premiums $1,500 x 12 = 18,000.
    // Line 1 = line 3 = 18,000; line 4 = line 5 = 20,000; line 6 = 1.000;
    // line 7 = 1,413; line 8 = 18,587; line 9 = 3,000; line 10 = line 13 =
    // 15,587; line 14 = 15,587. Schedule A medical = 18,000 - 15,587 =
    // 2,413. AGI = 20,000 - 1,413 - 3,000 - 15,587 = 0.
    let mut d = empty_return(FederalStatus::Single);
    let mut business = form_7206_business(20000, 1500, 0, &[], vec![]);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { contribution: 300000, rate: 25.0 };
    add(&mut d, income(business));
    let r = supported(&d);
    let id = d.income[0].id;
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 1413);
    assert_eq!(value(&r, &alloc::format!("business_{id}_sep_deduction")), 3000);
    assert_eq!(value(&r, &alloc::format!("business_{id}_health_insurance")), 15587);
    assert_eq!(value(&r, &alloc::format!("business_{id}_medical_premiums")), 2413);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 15587);
    assert_eq!(value(&r, "agi"), 0);
}

#[test]
fn form_7206_line_5_excludes_the_other_business_loss() {
    // Boundary of line 5 ("Don't include any net losses"): the plan's
    // business earns $24,000 and a second business loses $4,000. Schedule
    // SE nets them: 20,000 x 0.9235 = 18,470; SE tax 2,826; line 15 = 1,413.
    // Form 7206: line 4 = 24,000; line 5 = 24,000 (the loss excluded); line
    // 6 = 1.000; line 7 = 1,413; line 8 = line 13 = 22,587; premiums $700 x
    // 12 = 8,400 = line 3; line 14 = 8,400. AGI = 20,000 - 1,413 - 8,400 =
    // 10,187.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(form_7206_business(24000, 700, 0, &[], vec![])));
    add(&mut d, income(cash_business(-4000)));
    let r = supported(&d);
    let id = d.income[0].id;
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 1413);
    assert_eq!(value(&r, &alloc::format!("business_{id}_health_insurance")), 8400);
    assert_eq!(value(&r, "agi"), 10187);
}

#[test]
fn form_7206_defined_benefit_owner_plan_and_ineligible_policy_left_out() {
    let mut d = empty_return(FederalStatus::Single);
    let mut business = form_7206_business(20000, 500, 0, &[], vec![]);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.retirement = BusinessRetirement::DefinedBenefit { contribution: 100_000 };
    add(&mut d, income(business));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_owner_plan_deduction"), 1000);
    assert!(noted(&r, "NOTE_DEFINED_BENEFIT_ACTUARY"));
    // An actual own-name partner policy with no entity payment is not
    // established under the partnership; the typed source leaves it out.
    let mut partner = partner_health_source(6000, false);
    let BusinessHealthInsurance::Plan { establishment, .. } = &mut partner.health_insurance else {
        unreachable!()
    };
    *establishment =
        HealthEstablishment::Partnership { payment: EntityPremiumPayment::IndividualUnreimbursed };
    d.income[0].income = FederalIncome::Partnership { data: partner };
    // No self-employed deduction; independently eligible medical cost remains.
    let r = supported(&d);
    assert!(noted(&r, "WARN_SE_HEALTH_LEFT_OUT"));
    assert_eq!(value(&r, &alloc::format!("business_{}_health_insurance", d.income[0].id)), 0);
    assert_eq!(value(&r, &alloc::format!("business_{}_medical_premiums", d.income[0].id)), 6000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn marketplace_marriage_retained_dependent_assignment_tracks_the_actual_claimed_family() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    let child = d.dependents[0].id;
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, vec![marketplace_month(1, 500, 600, 400)]);
    let policy = marketplace_facts_mut(&mut d).forms[0].id;
    marketplace_facts_mut(&mut d).marriage = MarketplaceMarriage::Alternative {
        wedding: 20260615,
        unmarried_at_start: true,
        you: MarketplaceMarriageFamily {
            dependents: vec![child],
            premarriage_policies: vec![policy],
            benchmarks: vec![],
        },
        spouse: MarketplaceMarriageFamily {
            dependents: vec![],
            premarriage_policies: vec![],
            benchmarks: vec![],
        },
    };
    assert!(d.valid_stored());
    let forms = marketplace_facts_mut(&mut d).forms.clone();
    d.dependents[0].competing = CompetingClaim::Other;
    assert!(!d.valid_stored());
    all_ticked(&mut d);
    d.finished_sections.push(TaxSection::Marketplace);
    let pruned = crate::interview::prune(&mut d);
    let facts = crate::marketplace::facts(&d.credits.as_ref().unwrap().marketplace).unwrap();
    assert_eq!(facts.marriage, MarketplaceMarriage::Regular);
    assert_eq!(facts.forms, forms);
    assert!(!d.finished_sections.contains(&TaxSection::Marketplace));
    assert!(
        pruned
            .removed
            .iter()
            .any(|removed| removed.key() == "SAVING_REMOVES_INVALID_MARKETPLACE_MARRIAGE_ELECTION")
    );
    assert!(d.valid_stored());
}

// Instructions8962 (2025), line2b/Form8814 and Worksheet1-2; Pub974
// (2025 final, printed January12,2026), Household income and WorksheetX line1.
// Derived2026 fixture: joint wages60000, child interest5000, nontaxableSSA1200.
// Existing Form8814 owner reports2300 in AGI and restores2700 for household
// income, then adds the child's1200 nontaxableSSA once. No SSA taxable income.
#[test]
fn marketplace_child_election_nontaxable_social_enters_household_income_once() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    let child = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20160615;
    add(
        &mut d,
        child_election(child, vec![child_interest(5_000)], 0, OtherParentChoice::NotNeeded),
    );
    child_election_mut(&mut d).nontaxable_social = 120_000;
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 1500, 1200, 0)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 62300);
    assert_eq!(value(&r, "marketplace_household_income"), 66200);
    let mut zero = d.clone();
    child_election_mut(&mut zero).nontaxable_social = 0;
    assert_eq!(value(&supported(&zero), "marketplace_household_income"), 65000);
    // A second required-to-file return for that elected child is contradictory.
    let MarketplaceClaim::Yes { facts } = &mut d.credits.as_mut().unwrap().marketplace else {
        unreachable!()
    };
    facts.dependents.push(MarketplaceDependent {
        person: child,
        agi: 0,
        exempt_interest: 0,
        foreign_exclusion: 0,
        social: 120_000,
        taxable_social: 0,
    });
    assert!(!crate::marketplace::valid(&d.credits.as_ref().unwrap().marketplace, &d));
    // An unavailable election is omitted by its existing owner; the child's
    // required separate return may then supply its own household income.
    child_election_mut(&mut d).facts = ChildElectionFacts::OTHER_INCOME.0;
    assert!(crate::marketplace::valid(&d.credits.as_ref().unwrap().marketplace, &d));
    assert_eq!(value(&supported(&d), "marketplace_household_income"), 61200);
}

// Derived2026 boundary under section162(l)(2)(A), Form7206 and Pub974
// WorksheetsW/Step3. Two actual business plans share one saved January
// statement equally. Interest200000 makes PTCzero; SE11000 yields1555tax,
// half778 allocated71/707, caps929/9293. Each assigned premium1000; total
// allowable1929, not2000 financed with another business's unused earnings.
#[test]
fn linked_marketplace_businesses_keep_established_trade_limits_and_one_source_total() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(savings_income(200000, 0)));
    add(&mut d, income(marketplace_business(1000, 1..=1)));
    add(&mut d, income(marketplace_business(10000, 1..=1)));
    *marketplace_claim_mut(&mut d) =
        marketplace_claim(&mut d, vec![marketplace_month(1, 2000, 2000, 0)]);
    assign_marketplace_businesses(&mut d);
    let first = d.income[1].id;
    let second = d.income[2].id;
    let r = supported(&d);
    assert_eq!(value(&r, "premium_tax_credit"), 0);
    assert_eq!(value(&r, "marketplace_specified_premiums"), 2000);
    assert_eq!(value(&r, "marketplace_specified_deduction"), 1929);
    assert_eq!(value(&r, &alloc::format!("business_{first}_health_insurance")), 929);
    assert_eq!(value(&r, &alloc::format!("business_{second}_health_insurance")), 1000);
    let before = r;
    d.income.reverse();
    d.adjustments.as_mut().unwrap().health_marketplace.policies.reverse();
    let reordered = supported(&d);
    for id in [
        "marketplace_specified_premiums",
        "marketplace_specified_deduction",
        "agi",
        "taxable_income",
        "qbi_deduction",
    ] {
        assert_eq!(value(&before, id), value(&reordered, id));
    }
}

// Derived canonical allocation boundary: one dollar paid once, split across
// two established plans. Both exact50c shares must not independently round to
// two dollars. Stable source IDs settle only the final form-rounding residual.
#[test]
fn linked_marketplace_businesses_round_the_shared_total_once() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(savings_income(200000, 0)));
    add(&mut d, income(marketplace_business(1000, 1..=1)));
    add(&mut d, income(marketplace_business(1000, 1..=1)));
    *marketplace_claim_mut(&mut d) = marketplace_claim(&mut d, vec![marketplace_month(1, 1, 1, 0)]);
    assign_marketplace_businesses(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "marketplace_specified_premiums"), 1);
    assert_eq!(value(&r, "marketplace_specified_deduction"), 1);
    let sum: i64 = d
        .income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::Business { .. }))
        .map(|e| value(&r, &alloc::format!("business_{}_health_insurance", e.id)))
        .sum();
    assert_eq!(sum, 1);
}

// Pub. 969 (2025 final), Qualifying for an HSA Contribution / Distributions
// From an HSA, and Instructions 8889 lines 8, 13 and Excess Employer
// Contributions: claimability sets contribution capacity to zero, not the
// tax character of an existing account's medical distribution. Derived 2026
// facts implement approved Unit20 fixture G; wages/age are neutral completion
// facts and the account value is after its $200 medical distribution.
// https://www.irs.gov/publications/p969
// https://www.irs.gov/instructions/i8889
#[test]
fn claimable_hsa_owner_calculates_zero_capacity_excess_and_normal_distributions() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 50_000);
    h.employer = 100_000;
    h.employer_outside = 100_000;
    add_hsa_distribution(&mut d, Owner::Taxpayer, 20_000, vec![], None, None);
    h.excess_accounting = Some(hsa_excess_accounting(150_000, None, 0, 0, 0));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 500);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_employer_excess"), 1000);
    assert_eq!(value(&r, "hsa_return_year_uncorrected_employer_excess_other_income"), 1000);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 90);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 0);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    assert_eq!(value(&r, "agi"), 61000);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    // Actual claiming makes no difference; W-2 inclusion prevents double tax.
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    d.income[0] = wage(1, Owner::Taxpayer, 6_100_000, 0);
    let h = &mut d.adjustments.as_mut().unwrap().hsa[0];
    h.excess_accounting.as_mut().unwrap().employer_excess_in_wages = 100_000;
    add_hsa_distribution(&mut d, Owner::Taxpayer, 10_000, vec![], None, None);
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260701, amount: 10_000 }],
        None,
        None,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "hsa_return_year_uncorrected_employer_excess_other_income"), 0);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 100);
    assert_eq!(value(&r, "hsa_additional_tax"), 20);
    assert_eq!(value(&r, "agi"), 61100);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn claimable_hsa_owner_has_no_december_catchup_or_unknown_income_assumption() {
    let mut d = complete(FederalStatus::Single);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 19600101;
    p.dependency = FilerDependency::CanBeClaimed { claimed: false };
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 50_000);
    let mut months = vec![HsaCoverage::None; 12];
    months[11] = HsaCoverage::Family;
    h.coverage = HsaEligibility::Monthly { months };
    h.excess_accounting = Some(hsa_excess_accounting(50_000, None, 0, 0, 0));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 30);
    assert!(!r.lines.iter().any(|l| l.id == "hsa_1_last_month_contributions"));
}

#[test]
fn claimable_spouse_hsa_does_not_gain_capacity_from_december_family_election() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    let mut h = hsa_entry(Owner::Spouse, HsaCoverage::Family, 50_000);
    h.excess_accounting = Some(hsa_excess_accounting(50_000, None, 0, 0, 0));
    d.adjustments.as_mut().unwrap().hsa =
        vec![hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 0), h];
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: None,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "hsa_2_return_year_uncorrected_excess"), 500);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 30);
}

// Pub969, Rules for married people, and i8889 line6 Steps1-4: divide
// the family limit equally absent an agreement, only while both spouses
// are eligible, after Archer reduction. Independent derived2026 fixtures.
// https://www.irs.gov/publications/p969
// https://www.irs.gov/instructions/i8889
#[test]
fn hsa_default_equal_family_share_calculates_excess_without_an_agreement() {
    for other_coverage in [HsaCoverage::Family, HsaCoverage::SelfOnly] {
        let mut d = complete(FederalStatus::Joint);
        let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 500_000);
        h.excess_accounting = Some(hsa_excess_accounting(100_000, None, 0, 0, 0));
        d.adjustments.as_mut().unwrap().hsa =
            vec![h, hsa_entry(Owner::Spouse, other_coverage, 400_000)];
        let r = supported(&d);
        assert_eq!(value(&r, "adjustments"), 8375);
        assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 625);
        assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 38);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.adjustments.as_mut().unwrap().hsa.reverse();
        assert_eq!(supported(&d), r);
    }
}

#[test]
fn hsa_equal_default_uses_actual_shared_months_and_ineligible_spouse() {
    let mut d = complete(FederalStatus::Joint);
    let mut spouse = hsa_entry(Owner::Spouse, HsaCoverage::None, 218_750);
    let mut months = vec![HsaCoverage::SelfOnly; 6];
    months.extend(vec![HsaCoverage::None; 6]);
    spouse.coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa =
        vec![hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 656_250), spouse];
    // Six shared family months:4375/2 each; six A-only family months:4375.
    // 2026-10-19 (D-2026-10-19-12): each owner's Form 8889 prints line 13 in whole dollars
    // (6,562.50 -> 6,563 and 2,187.50 -> 2,188), and Schedule 1 line 13 adds the printed lines.
    assert_eq!(value(&supported(&d), "adjustments"), 8751);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
    let a = d.adjustments.as_mut().unwrap();
    a.hsa[0].personal = 875_000;
    a.hsa[1].personal = 0;
    a.hsa[1].coverage = HsaEligibility::FullYear { coverage: HsaCoverage::None };
    assert_eq!(value(&supported(&d), "adjustments"), 8750);
}

#[test]
fn hsa_equal_default_reduces_archer_once_before_dividing_family_capacity() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 4_000_000, 0));
    set_archer(
        &mut d,
        vec![
            archer_contributor(Owner::Taxpayer, archer_family(6000), 1000),
            archer_contributor(Owner::Spouse, archer_family(6000), 0),
        ],
        ArcherSharing::Equal,
    );
    d.adjustments.as_mut().unwrap().hsa = vec![
        hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 387_500),
        hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 387_500),
    ];
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_deduction"), 1000);
    assert_eq!(value(&r, "adjustments"), 8750);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

#[test]
fn hsa_december_self_only_capacity_is_not_capped_by_an_earlier_family_month() {
    let mut d = complete(FederalStatus::Joint);
    let mut a = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 440_000);
    let mut months = vec![HsaCoverage::SelfOnly; 12];
    months[0] = HsaCoverage::Family;
    a.coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa =
        vec![a, hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 440_000)];
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        taxpayer_parts: 1,
        spouse_parts: 1,
        other_coverage: None,
    });
    // January was shared family; December independent self-only4400 each
    // gives8800 total, rather than an unrelated8750 family cap.
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 8800);
    assert_eq!(value(&r, "hsa_1_last_month_contributions"), 2);
    assert_eq!(value(&r, "hsa_2_last_month_contributions"), 2);
}

#[test]
fn hsa_other_owner_context_cannot_create_an_account_or_invent_a_family_share() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = complete(FederalStatus::Joint);
        d.adjustments.as_mut().unwrap().hsa = vec![hsa_entry(owner, HsaCoverage::Family, 875_000)];
        unsupported(&d, "HSA");
        d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
            method: HsaAllocationMethod::Monthly,
            taxpayer_parts: 1,
            spouse_parts: 1,
            other_coverage: Some(HsaEligibility::FullYear { coverage: HsaCoverage::None }),
        });
        assert_eq!(value(&supported(&d), "adjustments"), 8750);
        assert_eq!(d.adjustments.as_ref().unwrap().hsa.len(), 1);
        d.adjustments.as_mut().unwrap().hsa_family_share.as_mut().unwrap().other_coverage =
            Some(HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly });
        unsupported(&d, "HSA"); // Actual both-eligible equal capacity4375.
    }
}

// IRS 2026 draft Form 8889 printed p1 line4 and instructions printed pp6–7.
// https://www.irs.gov/pub/irs-dft/f8889--dft.pdf (created April30,2026).
// Derived source endpoint, not a published worked example: spouse's eligible
// self-only Archer plan January–June, Medicare July–December, actual $1,000
// contribution. Taxpayer under55 becomes family-eligible July1. Neutral wages
// and existing birth dates only complete the return. Annual spouse Archer
// contributions reduce the HSA limit even with no simultaneously eligible
// family month and no December-eligible spouse.
#[test]
fn hsa_family_limit_counts_archer_for_december_ineligible_spouse() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(2, Owner::Spouse, 4_000_000, 0));
    let mut archer = archer_contributor(Owner::Spouse, archer_self(4000), 1000);
    let mut archer_months = vec![archer_self(4000); 6];
    archer_months.extend(vec![ArcherCoverage::None; 6]);
    archer.coverage = ArcherEligibility::Monthly { months: archer_months };
    let mut months = vec![HsaCoverage::None; 6];
    months.extend(vec![HsaCoverage::Family; 6]);
    let mut spouse = vec![HsaCoverage::SelfOnly; 6];
    spouse.extend(vec![HsaCoverage::None; 6]);
    let mut session = Interview::new(TEST_USER, d);
    let mut page = crate::tests::cross::interview::open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else { panic!() };
    input.accounts = vec![crate::tests::cross::interview::hsa_account(
        Owner::Taxpayer,
        HsaCoverageYear::Changed { months: months.into() },
        775_000,
    )];
    input.other_coverage = Some(HsaEligibilityPage::Monthly { months: spouse.into() });
    input.family_method = HsaAllocationMethod::December;
    input.family_split = FamilySplit::NotApplicable;
    input.archer =
        ArcherContributions::Records { owners: vec![archer], sharing: ArcherSharing::Equal }.page();
    crate::tests::cross::interview::accepted(&send(&mut session, page));
    let reopened = crate::tests::cross::interview::open_hsa_page(&mut session);
    assert_eq!(
        crate::tests::cross::interview::hsa_page_value(&reopened).0.family_method,
        HsaAllocationMethod::December
    );
    let mut d = session.draft;
    assert_eq!(supported(&restart(d.clone()).draft), supported(&d));
    // December lines3/4/8:8750−1000=7750, plus separate1000 Archer deduction.
    let detail = supported(&d);
    assert_eq!(value(&detail, "adjustments"), 8750);
    for (suffix, amount) in [
        ("limit", 875_000),
        ("archer", 100_000),
        ("after_archer", 775_000),
        ("allocated", 775_000),
        ("total_limit", 775_000),
        ("deduction", 775_000),
    ] {
        assert_eq!(line_amount(&detail, &alloc::format!("hsa_1_contribution_{suffix}")), amount);
    }
    d.adjustments.as_mut().unwrap().hsa[0].personal = 800_000;
    unsupported(&d, "HSA");
    d.adjustments.as_mut().unwrap().hsa[0].excess_accounting =
        Some(hsa_excess_accounting(800_000, None, 0, 0, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 8750);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 250);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 15);
    let a = d.adjustments.as_mut().unwrap();
    a.hsa[0].excess_accounting = None;
    a.hsa[0].personal = 337_500;
    a.hsa_family_share.as_mut().unwrap().method = HsaAllocationMethod::Monthly;
    // Six family months:4375−1000=3375. Archer reduction is annual.
    let detail = supported(&d);
    assert_eq!(value(&detail, "adjustments"), 4375);
    assert_eq!(line_amount(&detail, "hsa_1_contribution_limit"), 437_500);
    assert_eq!(line_amount(&detail, "hsa_1_contribution_total_limit"), 337_500);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
    let a = d.adjustments.as_mut().unwrap();
    a.hsa[0].personal = 340_000;
    a.hsa_family_share.as_mut().unwrap().method = HsaAllocationMethod::December;
    let HsaEligibility::Monthly { months } = &mut a.hsa[0].coverage else { panic!() };
    months[11] = HsaCoverage::SelfOnly;
    // Earlier family coverage still invokes line4: December4400−1000=3400.
    assert_eq!(value(&supported(&d), "adjustments"), 4400);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

// Derived precision boundaries, not IRS published dollar oracles. Draft2026
// i8889 printedpp3,8–9 supplies the source classifications and PartII arithmetic;
// https://www.irs.gov/pub/irs-dft/i8889--dft.pdf (printed July21,2026).
// Existing estimate transfers round each owner's actual income/penalty base;
// detailed cents must neither invent exception allocations nor round inputs
// independently before subtraction. Neutral existing wages/births complete the return.
#[test]
fn hsa_distribution_detail_preserves_cents_and_separate_owner_estimate_rounding() {
    let cents = |r: &FederalResult, id: &str| line_amount(r, id);
    let mut d = complete(FederalStatus::Single);
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        50,
        vec![MedicalAccountWithdrawal { date: 20260615, amount: 99 }],
        None,
        None,
    );
    let r = supported(&d);
    assert_eq!(cents(&r, "hsa_1_distribution_gross"), 149);
    assert_eq!(cents(&r, "hsa_1_distribution_net"), 149);
    assert_eq!(cents(&r, "hsa_1_distribution_medical"), 50);
    assert_eq!(cents(&r, "hsa_1_distribution_taxable"), 99);
    assert_eq!(printed_line(&r, "hsa_1_distribution_taxable"), (TaxForm::Form8889, "16".into()));
    assert_eq!(value(&r, "hsa_1_distribution_estimate_income"), 1);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    let mut d = complete(FederalStatus::Single);
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260615, amount: 99 }],
        None,
        Some(MedicalRollover {
            preceding: None,
            receipts: vec![MedicalRolloverReceipt {
                received: 20260101,
                deposits: vec![MedicalRolloverDeposit {
                    deposited: 20260102,
                    amount: 50,
                    destination: MedicalRolloverDestination::Hsa,
                    character: MedicalDepositCharacter::Unconfirmed,
                }],
            }],
        }),
    );
    let r = supported(&d);
    assert_eq!(cents(&r, "hsa_1_distribution_gross"), 149);
    assert_eq!(cents(&r, "hsa_1_distribution_excluded"), 50);
    assert_eq!(cents(&r, "hsa_1_distribution_net"), 99);
    assert_eq!(cents(&r, "hsa_1_distribution_taxable"), 99);
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19610701;
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![
            MedicalAccountWithdrawal { date: 20260115, amount: 249 },
            MedicalAccountWithdrawal { date: 20261215, amount: 2 },
        ],
        None,
        None,
    );
    let r = supported(&d);
    assert_eq!(cents(&r, "hsa_1_distribution_taxable"), 251);
    assert_eq!(cents(&r, "hsa_1_distribution_penalty_base"), 249);
    assert_eq!(cents(&r, "hsa_1_distribution_tax"), 50);
    assert_eq!(value(&r, "hsa_1_distribution_exception"), 1);
    assert_eq!(value(&r, "hsa_1_distribution_estimate_income"), 3);
    assert_eq!(value(&r, "hsa_1_distribution_estimate_tax"), 0);
    let mut d = complete(FederalStatus::Joint);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        add_hsa_distribution(
            &mut d,
            owner,
            0,
            vec![MedicalAccountWithdrawal { date: 20260615, amount: 249 }],
            None,
            None,
        );
    }
    let r = supported(&d);
    for i in 1..=2 {
        assert_eq!(cents(&r, &alloc::format!("hsa_{i}_distribution_taxable")), 249);
        assert_eq!(value(&r, &alloc::format!("hsa_{i}_distribution_estimate_income")), 2);
        assert_eq!(value(&r, &alloc::format!("hsa_{i}_distribution_estimate_tax")), 0);
    }
    assert_eq!(value(&r, "hsa_taxable_distributions"), 4);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
}

#[test]
fn hsa_distribution_detail_keeps_pledges_outside_form_and_manual_returns_once() {
    let cents = |r: &FederalResult, id: &str| line_amount(r, id);
    let mut d = complete(FederalStatus::Single);
    add_hsa_distribution(&mut d, Owner::Taxpayer, 10_000, vec![], None, None);
    add(
        &mut d,
        income(FederalIncome::MedicalDistribution {
            data: crate::SaSource {
                medicare_prior: if matches!(
                    &MedicalSaAccount::Hsa,
                    MedicalSaAccount::MedicareAdvantage
                ) {
                    Some(MedicareMsaPrior::None)
                } else {
                    None
                },
                medicare_from_spouse: if matches!(
                    &MedicalSaAccount::Hsa,
                    MedicalSaAccount::MedicareAdvantage
                ) {
                    Some(false)
                } else {
                    None
                },
                payment: Some(MedicalSaPayment::Unreported {
                    event: MedicalDeemed::Pledged {
                        assets: vec![MedicalAccountWithdrawal { date: 20260615, amount: 5000 }],
                    },
                    disabled_since: None,
                }),
                ..sa_source(Sa1099 {
                    origin: HealthDocumentOrigin::TrusteeStatement,
                    gross: 5000,
                    box2_earnings: None,
                    box3_code: None,
                    box4_value: None,
                    account: MedicalSaAccount::Hsa,
                    ..Default::default()
                })
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(cents(&r, "hsa_1_distribution_gross"), 10_000);
    assert_eq!(cents(&r, "hsa_1_distribution_medical"), 10_000);
    assert_eq!(cents(&r, "hsa_1_distribution_taxable"), 0);
    assert_eq!(cents(&r, "hsa_1_pledged_income"), 5000);
    assert_eq!(value(&r, "hsa_1_distribution_estimate_income"), 50);
    assert_eq!(value(&r, "hsa_additional_tax"), 10);
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 100_000);
    h.corrections.push(hsa_correction(2026, 20260930, 20_000, -2000));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    assert_eq!(cents(&r, "hsa_1_distribution_gross"), 18_000);
    assert_eq!(cents(&r, "hsa_1_distribution_excluded"), 18_000);
    assert_eq!(cents(&r, "hsa_1_distribution_net"), 0);
    assert_eq!(value(&r, "adjustments"), 800);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 0);
    assert_eq!(supported(&restart(d).draft), r);
}

// Derived 2026 endpoints from draft i8889 printed pp4–7 (July21,2026),
// https://www.irs.gov/pub/irs-dft/i8889--dft.pdf, and draft i5329 printed p8
// line43, https://www.irs.gov/pub/irs-dft/i5329--dft.pdf. Existing neutral
// wages and birth dates complete the returns. These are cents-level owner
// traces, not published worked examples or filing-ready rounded forms.
#[test]
fn hsa_contribution_detail_keeps_prior_excess_and_own_catchup_classes() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19701231;
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 300_000);
    h.employer = 100_000;
    h.employer_outside = 100_000;
    h.excess_accounting = Some(hsa_excess_accounting(
        900_000,
        Some(HsaPriorExcess { line_48: 100_000, line_49: 6000 }),
        0,
        0,
        0,
    ));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    for (suffix, amount, printed) in [
        ("personal", 300_000, "2"),
        ("limit", 540_000, "3"),
        ("archer", 0, "4"),
        ("after_archer", 540_000, "5"),
        ("allocated", 540_000, "6"),
        ("catchup", 0, "7"),
        ("total_limit", 540_000, "8"),
        ("employer", 100_000, "9"),
        ("funding", 0, "10"),
        ("other", 100_000, "11"),
        ("personal_limit", 440_000, "12"),
        ("deduction", 400_000, "13"),
    ] {
        let id = alloc::format!("hsa_1_contribution_{suffix}");
        assert_eq!(line_amount(&r, &id), amount, "{id}");
        assert_eq!(printed_line(&r, &id), (TaxForm::Form8889, printed.into()));
    }
    assert_eq!(value(&r, "adjustments"), 4000);
    assert_eq!(value(&r, "hsa_1_prior_excess_deducted_return_year"), 1000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn hsa_contribution_detail_preserves_combined_twelfths_without_invented_residual() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19701231;
    let mut taxpayer = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 44_792);
    let mut spouse = hsa_entry(Owner::Spouse, HsaCoverage::None, 36_458);
    let mut months = vec![HsaCoverage::None; 12];
    months[0] = HsaCoverage::Family;
    taxpayer.coverage = HsaEligibility::Monthly { months: months.clone() };
    spouse.coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa = vec![taxpayer, spouse];
    let r = supported(&d);
    let cents = |id: &str| line_amount(&r, id);
    // 2026-10-19 (D-2026-10-19-12): Form 8889 prints whole dollars figured from its printed
    // lines (729.17, 364.58 and 83.33 print 729, 365 and 83; line 8 is 365 + 83 = 448).
    assert_eq!(cents("hsa_1_contribution_limit"), 72_900);
    assert_eq!(cents("hsa_1_contribution_allocated"), 36_500);
    assert_eq!(cents("hsa_1_contribution_catchup"), 8300);
    assert_eq!(cents("hsa_1_contribution_total_limit"), 44_800);
    assert_eq!(cents("hsa_1_contribution_personal_limit"), 44_800);
    assert_eq!(cents("hsa_1_contribution_deduction"), 44_800);
    assert_eq!(cents("hsa_2_contribution_catchup"), 0);
    assert_eq!(cents("hsa_2_contribution_total_limit"), 36_500);
    assert_eq!(value(&r, "adjustments"), 813);
}

// Source-derived clamp endpoints from 2026 draft i8889 printed pp4–7
// (July21,2026), https://www.irs.gov/pub/irs-dft/i8889--dft.pdf. Unmarried
// catch-up is included on3 before subtracting4; married family-anytime catch-up
// is separate7 after5/6 are floored at zero. These are not IRS worked examples.
// Neutral W2/birth facts complete the return; Archer sources and retained
// excess use their existing owner, without an invented HSA reduction residual.
#[test]
fn hsa_december_archer_clamp_preserves_separate_married_family_catchup() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19701231;
    add(&mut d, wage(2, Owner::Spouse, 4_000_000, 0));
    let mut archer = archer_contributor(Owner::Spouse, archer_family(7200), 4500);
    let mut archer_months = vec![archer_family(7200); 10];
    archer_months.extend(vec![ArcherCoverage::None; 2]);
    archer.coverage = ArcherEligibility::Monthly { months: archer_months };
    set_archer(
        &mut d,
        vec![archer_contributor(Owner::Taxpayer, ArcherCoverage::None, 0), archer],
        ArcherSharing::Equal,
    );
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 100_000);
    let mut months = vec![HsaCoverage::None; 10];
    months.extend([HsaCoverage::Family, HsaCoverage::SelfOnly]);
    h.coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let mut spouse = vec![HsaCoverage::Family; 10];
    spouse.extend([HsaCoverage::None; 2]);
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        taxpayer_parts: 1,
        spouse_parts: 0,
        other_coverage: Some(HsaEligibility::Monthly { months: spouse }),
    });
    let r = supported(&d);
    let cents = |suffix: &str| line_amount(&r, &alloc::format!("hsa_1_contribution_{suffix}"));
    assert_eq!(cents("limit"), 440_000);
    assert_eq!(cents("archer"), 450_000);
    assert_eq!(cents("after_archer"), 0);
    assert_eq!(cents("allocated"), 0);
    assert_eq!(cents("catchup"), 100_000);
    assert_eq!(cents("total_limit"), 100_000);
    assert_eq!(cents("deduction"), 100_000);
    assert_eq!(value(&r, "adjustments"), 5500);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn hsa_december_archer_clamp_includes_unmarried_catchup_before_reduction() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19701231;
    let mut archer = archer_contributor(Owner::Taxpayer, archer_family(6000), 9000);
    archer.excess_accounting = Some(archer_excess_accounting(900_000, None, 0, 0, 0));
    set_archer(&mut d, vec![archer], ArcherSharing::Equal);
    d.adjustments.as_mut().unwrap().hsa =
        vec![hsa_entry(Owner::Taxpayer, HsaCoverage::Family, 75_000)];
    d.adjustments.as_mut().unwrap().hsa_family_share = Some(HsaFamilyShare {
        method: HsaAllocationMethod::December,
        taxpayer_parts: 1,
        spouse_parts: 0,
        other_coverage: None,
    });
    let r = supported(&d);
    let cents = |suffix: &str| line_amount(&r, &alloc::format!("hsa_1_contribution_{suffix}"));
    assert_eq!(cents("limit"), 975_000);
    assert_eq!(cents("archer"), 900_000);
    assert_eq!(cents("after_archer"), 75_000);
    assert_eq!(cents("allocated"), 75_000);
    assert_eq!(cents("catchup"), 0);
    assert_eq!(cents("total_limit"), 75_000);
    assert_eq!(cents("deduction"), 75_000);
    assert_eq!(value(&r, "adjustments"), 5250);
    assert_eq!(value(&r, "archer_return_year_excess_contribution_tax"), 270);
    d.adjustments.as_mut().unwrap().hsa[0].personal += 1;
    unsupported(&d, "HSA");
}

// Derived current2026 fixtures, not published complete returns. Pub974(2025)
// https://www.irs.gov/publications/p974, printed pp56–57, requires completing
// the health method without each listed item, calculating that item with the
// selected health deduction, then retaining it for the repeated method.
// Neutral completion: independent single filer born1980; qualifying children
// have no income; one sole-proprietor plan and one actual saved statement.
fn ordered_health_return(profit: i64, family: usize, monthly: i64) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    for _ in 1..family {
        add_child(&mut d);
    }
    add(&mut d, income(marketplace_business(profit, 1..=12)));
    *marketplace_claim_mut(&mut d) = marketplace_claim(
        &mut d,
        (1..=12).map(|m| marketplace_month(m, monthly, monthly, 0)).collect(),
    );
    assign_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    d
}

// A documented employer-plan participant with no taxable wages from that
// employer in this synthetic year. Coverage is derived from actual W-2 box13,
// never from a forged retained IRA coverage answer.
fn ordered_workplace_plan(d: &mut FederalDraft) {
    let mut entry = wage(0, Owner::Taxpayer, 0, 0);
    let FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source { form: data, .. }) } =
        &mut entry.income
    else {
        unreachable!()
    };
    data.retirement_plan = true;
    add(d, entry);
}

fn iterative_work(result: &FederalResult, id: &str) -> i64 {
    line_amount(result, id)
}

#[test]
fn ordered_marketplace_freezes_covered_ira_before_repeating_health() {
    // Actual existing SE owner:100000−7065=92935. Initial selected health
    //841787c gives IRA MAGI84517. Current2026 covered-single band81000–91000,
    //7500 ceiling: ceil_to_10(7500×6483/10000)=4870; nondeductible2630.
    // Repeated method base88065 selects credit402329c/deduction797671c.
    // Recalculating IRA with the later7977 health deduction would give4540,
    // which the prescribed ordered method does not substitute for4870.
    let mut d = ordered_health_return(100000, 3, 1000);
    ordered_workplace_plan(&mut d);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 4870);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 2630);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 402329);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 797671);
    assert_eq!(value(&r, "agi"), 80088);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Derived 2026 boundary, not a published IRS complete-return oracle.
// Pub. 974 (2025), printed pp. 56–57, Special Instructions item 2 uses the
// selected Iterative Step 6 health deduction in the covered-IRA MAGI worksheet.
// https://www.irs.gov/publications/p974 . Pub. 590-A Worksheet 1-2, line 4
// still rounds the reduced limit UP to $10. Current 2026 band/cap: 81000–91000/7500.
// Actual Schedule SE owner: profit 99530, net 91916, tax 14063, half-tax 7032.
// Initial base 92498 selects credit 362172c / health 837828c: IRA MAGI 84119.72.
// 7500 * (91000 - 84119.72) / 10000 = 5160.21 -> 5170. Rounding health first
// incorrectly supplies 84120 and gives 5160. Freeze 5170 before repeating.
// Independent Fraction final base 87328 selects 413329c / 786671c; its FPL
// percentage crosses below 300, so the applicable percentage also changes.
#[test]
fn ordered_marketplace_uses_selected_iterative_health_cents_for_ira_phaseout() {
    let mut d = ordered_health_return(99530, 3, 1000);
    ordered_workplace_plan(&mut d);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 7032);
    assert_eq!(value(&r, "ira_deduction"), 5170);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), 2330);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 413329);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 786671);
    assert_eq!(413329 + 786671, 1200000);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn ordered_marketplace_freezes_student_interest_before_repeating_health() {
    //110000−halfSE7772=102228. Initial selected health925962c rounds9260:
    // the existing loan owner therefore uses MAGI92968.
    //2500×(100000−92968)/15000 rounds1172. Retain that owner result;
    // repeated base101056 selects credit284654c/deduction915346c.
    let mut d = ordered_health_return(110000, 3, 1000);
    d.adjustments.as_mut().unwrap().student_loan = StudentLoanClaim {
        student_loan_interest: 250000,
        forms: vec![],
        employer_paid_interest: 0,
    };
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(schedule_1_line(&r, "21"), 1172);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 284654);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 915346);
    assert_eq!(value(&r, "agi"), 91903);
    assert_eq!(d, before);
}

#[test]
fn ordered_marketplace_freezes_actual_bond_worksheet_and_source_lines() {
    //130000−9184+3000=123816. Initial selected health1121499c rounds11215:
    // actual Form8815 MAGI112601; band101800–116800 gives eligible3000 less
    // reduction2160=840. Repeated base122976 selects credit686110c and
    // health1113890c. Preserve the frozen Form8815 line9 and single line14.
    use crate::tests::investment::investment::{bond_interest_entry, bond_record, bond_student};
    let mut d = ordered_health_return(130000, 4, 1500);
    let interest_id = d.next_id;
    add(&mut d, bond_interest_entry(3000));
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 12000, 3000, 20100601, vec![bond_student(-1, 14000, 0)]),
    );
    if let FederalIncome::SavingsBonds { data } = &mut d.income.last_mut().unwrap().income {
        data.answers.as_mut().unwrap().interest_statement = interest_id;
    }
    assert_eq!(crate::savings_bonds::source_rule(&d.income), None);
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "form_8815_modified_agi"), 112601);
    assert_eq!(value(&r, "savings_bond_exclusion"), 840);
    assert_eq!(value(&r, "schedule_b_savings_bond_exclusion"), 840);
    assert_eq!(r.lines.iter().filter(|row| row.id == "savings_bond_exclusion").count(), 1);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 686110);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 1113890);
    assert_eq!(value(&r, "agi"), 111837);
    assert_eq!(d, before);
}

#[test]
fn ordered_marketplace_reuses_prior_owners_in_the_prescribed_order_for_both_methods() {
    // Same sources for both accepted methods. Existing halfSE7065 and actual
    // bond interest3000 give95935 before health. IRA MUST precede the bond
    // exclusion: using excluded interest too early changes its phaseout.
    // Independent current2026 worksheet oracles:
    // Iterative: IRA2820, bond3000, student2500; selected PTC408569c and
    // health791431c; AGI79701. Simplified: IRA2570, bond3000, student2500;
    // Step3 health7184; final Step4 PTC3964; AGI80681.
    use crate::tests::investment::investment::{bond_interest_entry, bond_record, bond_student};
    let mut d = ordered_health_return(100000, 3, 1000);
    ordered_workplace_plan(&mut d);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    d.adjustments.as_mut().unwrap().student_loan = StudentLoanClaim {
        student_loan_interest: 250000,
        forms: vec![],
        employer_paid_interest: 0,
    };
    let interest_id = d.next_id;
    add(&mut d, bond_interest_entry(3000));
    add(
        &mut d,
        bond_record(Owner::Taxpayer, 12000, 3000, 20100601, vec![bond_student(-1, 14000, 0)]),
    );
    if let FederalIncome::SavingsBonds { data } = &mut d.income.last_mut().unwrap().income {
        data.answers.as_mut().unwrap().interest_statement = interest_id;
    }
    assert_eq!(crate::savings_bonds::source_rule(&d.income), None);
    for (method, ira, credit, health, agi) in [
        (HealthPtcMethod::Iterative, 2820, 4086, 7914, 79701),
        (HealthPtcMethod::Simplified, 2570, 3964, 7184, 80681),
    ] {
        d.adjustments.as_mut().unwrap().health_marketplace.method = method;
        let before = d.clone();
        let result = supported(&d);
        assert_eq!(value(&result, "ira_deduction"), ira);
        assert_eq!(value(&result, "savings_bond_exclusion"), 3000);
        assert_eq!(schedule_1_line(&result, "21"), 2500);
        assert_eq!(value(&result, "premium_tax_credit"), credit);
        assert_eq!(value(&result, "marketplace_specified_deduction"), health);
        assert_eq!(value(&result, "agi"), agi);
        if method == HealthPtcMethod::Iterative {
            assert_eq!(iterative_work(&result, "marketplace_iterative_credit_work"), 408569);
            assert_eq!(iterative_work(&result, "marketplace_iterative_deduction_work"), 791431);
        }
        assert_eq!(d, before);
    }
}

#[test]
fn ordered_marketplace_enters_the_existing_final_ira_basis_pass_after_selecting_health() {
    // Existing Pub590-B Worksheet1-1 owner first uses7500 contributions as
    // preliminary basis:10000 distribution×7500/(40000+10000)=1500 exempt,
    //8500 taxable. Business90000 less halfSE6359 leaves83641; base92141.
    // Initial health834595c gives IRA MAGI83795 and deduction5410; actual
    // nondeductible2090 covers preliminary1500, so the EXISTING annual owner
    // retains8500 taxable. Repeated method selects PTC423018c/health776982c.
    // Prematurely running the final annual owner at maximum12000 health
    // instead would select7500 IRA deduction and different taxable income.
    let mut d = ordered_health_return(90000, 3, 1000);
    ordered_workplace_plan(&mut d);
    add(&mut d, basis_distribution(Owner::Taxpayer, 10000, RetirementTreatment::Early));
    let a = d.adjustments.as_mut().unwrap();
    a.iras = ira_facts(750000, 0, Answer::Yes);
    a.ira_basis = vec![prior_ira_basis(Owner::Taxpayer, 0, 40000)];
    let before = d.clone();
    let result = supported(&d);
    assert_eq!(value(&result, "ira_deduction"), 5410);
    assert_eq!(value(&result, "taxpayer_nondeductible_ira"), 2090);
    assert_eq!(return_line(&result, "4b"), 850000);
    assert_eq!(iterative_work(&result, "marketplace_iterative_credit_work"), 423018);
    assert_eq!(iterative_work(&result, "marketplace_iterative_deduction_work"), 776982);
    assert_eq!(value(&result, "agi"), 78961);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Derived source endpoints, not published complete returns. Pub974(2025),
// printed pp51/56–57, https://www.irs.gov/publications/p974: initial income
// omits rental real-estate losses when8582 lines1d/3 are losses. i8582(2025)
// printed p15 separately reports own profit and prior losses it absorbs.
// Neutral completion: existing building basis275000, annual depreciation10000,
// rent includes10000 to offset that depreciation; fully at risk, active rental;
// any other passive source is a complete actual ordinary S-corporation record.
fn health_passive_rental(d: &mut FederalDraft, rent: i64, expense: i64, prior: i64) -> i64 {
    let mut source = rental(rent + 10000, expense);
    source.history =
        RentalHistory { regular: prior * 100, amt: prior * 100, active_in_loss_year: true };
    let id = d.next_id;
    add(d, income(FederalIncome::Rental { data: source }));
    id
}

#[test]
fn health_passive_ordinary_scorp_groups_keep_their_nonrental_classification() {
    use crate::tests::cross::interview::{accepted, open_document_center};
    use crate::tests::support::builders::k1res::{documents, input};
    use crate::tests::support::client::k1res::scorp_ordinary_multi_session;

    let (mut session, source) = scorp_ordinary_multi_session();
    let mut request = open_document_center(&mut session);
    documents(&mut request)[0].document = input(&FederalIncome::ScorpActivities { data: source });
    accepted(&crate::tests::support::client::k1res::send_fixture(&mut session, request));
    let draft = &session.draft;
    let mut output = FederalResult::empty(draft.revision);
    let actual = crate::extended_rules::rentals(draft, 200000, &mut output);
    assert!(actual.inputs_complete, "{:?}", output.issues);
    // The unchanged native fixture has passive profit and a competing prior
    // ordinary-business loss, plus a nonpassive ordinary-business loss.
    // None becomes real-estate rent or a real-estate-only release boundary.
    assert!(actual.health_ptc.rentals.is_empty());
    assert!(actual.health_ptc.mixed_losses);
    assert!(!actual.health_ptc.nonpassive_losses);
    assert!(!actual.health_ptc.released_outside_pool);
    // No rental allowance is deferred, so the known ordinary projection stays
    // available; this source control does not claim a coupled PTC amount.
    assert!(!actual.health_ptc.deferred);
    assert_eq!(actual.health_ptc.initial_addback(), Ok(0));
}

#[test]
fn health_passive_initial_income_retains_other_profit_before_rental_loss() {
    for (rent, expense, prior, scorp_profit, addback, current, allowed_loss) in
        [(0, 10000, 0, 6000, 6000, -10000, 6000), (1000, 0, 5000, 0, 1000, 1000, 1000)]
    {
        let mut d = empty_return(FederalStatus::Single);
        let id = health_passive_rental(&mut d, rent, expense, prior);
        if scorp_profit != 0 {
            add(
                &mut d,
                income(FederalIncome::Scorp {
                    data: scorp_source(complete_ordinary_scorp(scorp_profit, false)),
                }),
            );
        }
        let mut output = FederalResult::empty(d.revision);
        let actual = crate::extended_rules::rentals(&d, 200000, &mut output);
        assert!(actual.inputs_complete, "{:?}", output.issues);
        assert_eq!(actual.income, 0);
        assert!(actual.health_ptc.deferred);
        assert_eq!(actual.health_ptc.initial_addback(), Ok(addback));
        assert_eq!(
            actual.health_ptc.rentals,
            vec![crate::extended_rules::HealthRentalActivity {
                source: id,
                current,
                prior,
                allowed_loss,
            }]
        );
        if prior != 0 {
            assert_eq!(value(&output, &alloc::format!("rental_{id}_suspended_loss")), 4000);
        } else {
            // Accepted identity, rather than list order or labels, owns the source.
            d.income.reverse();
            let reordered =
                crate::extended_rules::rentals(&d, 200000, &mut FederalResult::empty(d.revision));
            assert_eq!(reordered.health_ptc.rentals, actual.health_ptc.rentals);
        }
    }
}

#[test]
fn health_passive_competing_nonrental_loss_has_a_named_source_boundary() {
    let mut d = ordered_health_return(180000, 7, 1500);
    health_passive_rental(&mut d, 0, 10000, 0);
    add(
        &mut d,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(6000, false)) }),
    );
    add(
        &mut d,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(-2000, false)) }),
    );
    let mut output = FederalResult::empty(d.revision);
    let actual = crate::extended_rules::rentals(&d, 200000, &mut output);
    assert!(actual.inputs_complete, "{:?}", output.issues);
    assert_eq!(actual.health_ptc.initial_addback(), Err(()));
    let result = estimate(&d);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_HEALTH_PTC_INITIAL_PASSIVE_ALLOCATION"),
        "{:?}",
        result.issues
    );
    assert!(result.return_lines.is_empty());
    // The ordinary passive owner and mandatory Marketplace reconciliation
    // remain available without the coupled specified-premium deduction.
    d.adjustments.as_mut().unwrap().health_marketplace.policies.clear();
    for entry in &mut d.income {
        if let FederalIncome::Business { data } = &mut entry.income {
            data.health_insurance = BusinessHealthInsurance::None;
        }
    }
    let ordinary = supported(&d);
    assert!(
        !ordinary
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_HEALTH_PTC_INITIAL_PASSIVE_ALLOCATION")
    );
}

#[test]
fn health_passive_marketplace_freezes_owner_before_repeating_health() {
    let mut d = ordered_health_return(180000, 7, 1500);
    let id = health_passive_rental(&mut d, 0, 10000, 0);
    add(
        &mut d,
        income(FederalIncome::Scorp { data: scorp_source(complete_ordinary_scorp(6000, false)) }),
    );
    let before = d.clone();
    let result = supported(&d);
    // Current2026 independent Fraction oracle (9.96%, full-year equal premiums):
    // business180000−halfSE12717=167283. Initial income173283 retains
    // passive6000 before the real-estate loss; Step4/5 selects230433/1569567c.
    // Actual PAL MAGI164304 allows6000 against passive profit, suspends4000.
    // Repeated income167283 selects PTC284780c/health1515220c, AGI152131.
    assert_eq!(value(&result, &alloc::format!("rental_{id}_suspended_loss")), 4000);
    assert_eq!(result.lines.iter().filter(|r| r.id == "rental_magi").count(), 1);
    assert_eq!(value(&result, "rental_magi"), 164304);
    assert_eq!(value(&result, "amt_rental_magi"), 164304);
    assert_eq!(iterative_work(&result, "marketplace_iterative_credit_work"), 284780);
    assert_eq!(iterative_work(&result, "marketplace_iterative_deduction_work"), 1515220);
    assert_eq!(value(&result, "agi"), 152131);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn health_passive_marketplace_retains_selected_partial_allowance() {
    // Derived current2026 oracle, Pub974 pp51/56–57; original dollar-source
    // publication rates are not reused. Existing business owner150000−10597
    // halfSE=139403, family7 FPL48650. Initial omits rental losses and uses
    // the current2026 applicable-rate table (263%=>8.84%): selected PTC667762c
    // with health1132238c (11322 dollars). Form8582 MAGI138678 allows5661
    // of30000. Freeze that full owner result; repeated base133742,253%=>8.53%,
    // selects748841c PTC and1051159c health. Refiguring PAL from that later
    // deduction would incorrectly replace5661 by5256, defeating the order.
    let mut d = ordered_health_return(150000, 7, 1500);
    let id = health_passive_rental(&mut d, 0, 30000, 0);
    let before = d.clone();
    let result = supported(&d);
    assert_eq!(value(&result, "rental_magi"), 138678);
    assert_eq!(value(&result, "amt_rental_magi"), 138678);
    assert_eq!(value(&result, &alloc::format!("rental_{id}_suspended_loss")), 24339);
    assert_eq!(value(&result, &alloc::format!("rental_{id}_line22_allowed_loss")), -5661);
    assert_eq!(iterative_work(&result, "marketplace_iterative_credit_work"), 748841);
    assert_eq!(iterative_work(&result, "marketplace_iterative_deduction_work"), 1051159);
    assert_eq!(value(&result, "agi"), 123230);
    assert_eq!(d, before);
}

#[test]
fn health_passive_k1_real_estate_identity_is_independent_of_allowance_and_box3() {
    // Same accepted partnership parent has box2 profit1000/prior5000 and
    // box3 profit6000. Box2 is NOT active, so its source classification must
    // not be inferred from eligibility for the25000 special allowance.
    let mut d = empty_return(FederalStatus::Single);
    let direct = health_passive_rental(&mut d, 0, 10000, 0);
    let mut partner = ordinary_partner_source(0, false);
    partner.regular.basis.opening = Some(5000000);
    partner.regular.risk.start = ScorpRiskStart::Detailed { capacity: 5000000, profit: 0, loss: 0 };
    partner.regular.rentals = K1RentalAmounts {
        real_estate: 100000,
        real_estate_carryover: 500000,
        other: 600000,
        ..Default::default()
    };
    partner.rental_facts.participation = Some(K1RentalParticipation::NotActive);
    partner.alternative = Some(partner.regular.clone());
    let linked = d.next_id;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let mut output = FederalResult::empty(d.revision);
    let actual = crate::extended_rules::rentals(&d, 200000, &mut output);
    assert!(actual.inputs_complete, "{:?}", output.issues);
    assert!(actual.health_ptc.deferred);
    assert_eq!(actual.health_ptc.rentals.len(), 2);
    assert!(actual.health_ptc.rentals.iter().any(|r| r.source == direct && r.current == -10000));
    let source = actual.health_ptc.rentals.iter().find(|r| r.source == linked).unwrap();
    assert_eq!((source.current, source.prior), (1000, 5000));
    // Own profit1000 and other passive profit6000 are retained initially;
    // the complete passive owner allows7000 real-estate losses in total.
    assert_eq!(actual.health_ptc.initial_addback(), Ok(7000));
}

#[test]
fn health_passive_nonpassive_professional_intersection_is_precisely_guarded() {
    // Source-verification boundary, not a published tax oracle. Pub974's
    // special list names passive losses; WorksheetX line1 says any rental
    // real-estate losses. No inspected primary example resolves their
    // concurrent professional-loss intersection while1d/3 are deferred.
    let mut d = ordered_health_return(150000, 7, 1500);
    health_passive_rental(&mut d, 0, 30000, 0);
    let id = health_passive_rental(&mut d, 0, 1000, 0);
    make_professional(&mut d, Owner::Taxpayer, false);
    let source = d.income.iter_mut().find(|e| e.id == id).unwrap();
    let FederalIncome::Rental { data } = &mut source.income else { unreachable!() };
    data.participation = RentalParticipation::Professional;
    let result = estimate(&d);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.display.key == "ISSUE_HEALTH_PTC_INITIAL_PASSIVE_ALLOCATION"),
        "{:?}",
        result.issues
    );
    assert!(result.return_lines.is_empty());
    // A positive professional activity is retained normally in the same
    // deferred return; its source classification is not a blanket exclusion.
    let source = d.income.iter_mut().find(|e| e.id == id).unwrap();
    let FederalIncome::Rental { data } = &mut source.income else { unreachable!() };
    data.rent += 200000;
    supported(&d);
    // Nor does this limit the independent ordinary owner when the shared
    // specified-premium deduction is not claimed.
    let source = d.income.iter_mut().find(|e| e.id == id).unwrap();
    let FederalIncome::Rental { data } = &mut source.income else { unreachable!() };
    data.rent -= 200000;
    d.adjustments.as_mut().unwrap().health_marketplace.policies.clear();
    for entry in &mut d.income {
        if let FederalIncome::Business { data } = &mut entry.income {
            data.health_insurance = BusinessHealthInsurance::None;
        }
    }
    supported(&d);
}

// Derived2026 complete source counterpart to Pub974(2025final) Gary's
//15000×12000/18000 expense split (printedp49) and Step3–6 (pp53–54).
// Published allocation is retained; business35000/current rates are neutral
// synthetic completion facts, not a published IRS full-return oracle.
// Independent Fraction oracle: netSE32323, tax4945, half2473; work credits
//1163193→1027088→1047390→1044875→1045014→1045007c. Final signed
//specified−45007 plus nonspecified500000 gives454993c, with no medical
//remainder and no duplicate credited cost. https://www.irs.gov/publications/p974
#[test]
fn iterative_mixed_premiums_keep_high_credit_signed_component_until_total() {
    let mut d = marketplace_nonmedical_child(Some(0));
    d.income[0].income = marketplace_business(35000, 1..=12);
    let facts = marketplace_facts_mut(&mut d);
    let policy = facts.forms[0].id;
    facts.forms[0].form.set_amounts(PolicyAmounts::AllYear {
        premium: 125000,
        benchmark: Some(100000),
        advance: 90000,
    });
    let coverage = facts.forms[0].coverage[0].clone();
    facts.forms[0].coverage = (1..=12)
        .map(|month| {
            let mut row = coverage.clone();
            row.month = month;
            row
        })
        .collect();
    let eligibility = facts.eligibility.clone();
    facts.eligibility = (1..=12)
        .flat_map(|month| {
            eligibility.iter().cloned().map(move |mut row| {
                row.month = month;
                row
            })
        })
        .collect();
    let benchmark = facts.benchmarks[0].clone();
    facts.benchmarks = (1..=12)
        .map(|month| {
            let mut row = benchmark.clone();
            row.month = month;
            row.groups[0].source = MarketplaceBenchmarkSource::Marketplace { amount: 100000 };
            row
        })
        .collect();
    facts.forms[0].medical_costs = (1..=12)
        .map(|month| MarketplaceMedicalCostMonth {
            month,
            child_increase: Some(0),
            tax_free_child: None,
        })
        .collect();
    assign_marketplace_businesses(&mut d);
    let adjustments = d.adjustments.as_mut().unwrap();
    adjustments.health_marketplace.method = HealthPtcMethod::Iterative;
    adjustments.health_marketplace.benchmarks = (1..=12)
        .map(|month| HealthPolicyBenchmark {
            policy,
            month,
            coverage: MarketplaceBenchmarkSource::Marketplace { amount: 100000 },
            noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 50000 },
        })
        .collect();
    let before = d.clone();
    let mut simplified = d.clone();
    simplified.adjustments.as_mut().unwrap().health_marketplace.method =
        HealthPtcMethod::Simplified;
    let result = supported(&simplified);
    // Simplified keeps Step3 component−1632 after Step4 credit10271.
    // Actual deduction3368 and credit10271 leave1361 medical expenses.
    assert_eq!(value(&result, "marketplace_specified_deduction"), -1632);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 3368);
    assert_eq!(value(&result, "premium_tax_credit"), 10271);
    assert_eq!(value(&result, "medical_expenses"), 1361);
    let result = supported(&d);
    assert_eq!(value(&result, "schedule_1_half_self_employment_tax"), 2473);
    assert_eq!(iterative_work(&result, "marketplace_iterative_credit_work"), 1045007);
    assert_eq!(iterative_work(&result, "marketplace_iterative_deduction_work"), -45007);
    assert_eq!(value(&result, "schedule_1_self_employed_health_insurance"), 4550);
    assert_eq!(value(&result, "medical_expenses"), 0);
    assert_eq!(value(&result, "agi"), 27977);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Pub974(2025 final), printed52–54: no interim whole-dollar rounding.
// https://www.irs.gov/publications/p974 . Derived current2026 complete-return
// counterpart to the admitted wages/interest case: separate short/long gains
// of49c each are zero filing rows but add98c before the work netting limit.
// Independent Decimal base37174.98 selects work credit413396/spec306604c.
#[test]
fn iterative_marketplace_nets_admitted_capital_source_cents_before_filing_rows() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    for term in [GainTerm::Short, GainTerm::Long] {
        let mut gain = capital_return(0, 0, term);
        let FederalIncome::ReturnOfCapital { amount, .. } = &mut gain else { unreachable!() };
        *amount = 49;
        add(&mut d, income(gain));
    }
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 413396);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 306604);
    assert_eq!(value(&r, "agi"), 34108);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Derived current2026 complete return: Pub974(2025final) pp52–54 retains
// admitted monetary inputs through work; https://www.irs.gov/publications/p974 .
// Taxable relief1000.49 and ownHSA49c + Archer49c are actual mandatory income,
// even though the latter owners each print zero dollars. Independent Fraction
// base38175.47 selects credit399471c/spec320529c. The relief is independently
// nonzero so a raw casualty-only residual cannot erase its whole-dollar amount.
#[test]
fn iterative_marketplace_retains_combined_relief_and_medical_owner_income() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    add(
        &mut d,
        income(FederalIncome::WildfireRelief {
            data: FederalWildfireRelief {
                source_issuer: Default::default(),
                source_recipient: Default::default(),
                paid: 20260601,
                declared: 20250101,
                reference: "Derived source for mandatory taxable treatment".into(),
                amount: 100049,
                withholding: 0,
                qualified: false,
            },
        }),
    );
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260615, amount: 49 }],
        None,
        None,
    );
    add(
        &mut d,
        income(FederalIncome::MedicalDistribution {
            data: crate::SaSource {
                medicare_prior: if matches!(
                    &MedicalSaAccount::Archer,
                    MedicalSaAccount::MedicareAdvantage
                ) {
                    Some(MedicareMsaPrior::None)
                } else {
                    None
                },
                medicare_from_spouse: if matches!(
                    &MedicalSaAccount::Archer,
                    MedicalSaAccount::MedicareAdvantage
                ) {
                    Some(false)
                } else {
                    None
                },
                payment: Some(MedicalSaPayment::Own {
                    nonmedical: vec![MedicalAccountWithdrawal { date: 20260616, amount: 49 }],
                    medical: 0,
                    rollover: None,
                    disabled_since: None,
                }),
                ..sa_source(Sa1099 {
                    origin: HealthDocumentOrigin::Form1099Sa,
                    gross: 49,
                    box2_earnings: None,
                    box3_code: Some(1),
                    box4_value: None,
                    account: MedicalSaAccount::Archer,
                    ..Default::default()
                })
            },
        }),
    );
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_relief_payments"), 1000);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 0);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 0);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 399471);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 320529);
    assert_eq!(value(&r, "agi"), 34969);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
    // Each other combined8z contributor has its own admitted source and
    // exactly the same known taxable1000.49; none can disappear behind the
    // casualty display variable. These are independent alternative sources,
    // not simultaneous entries for the same receipt.
    let mut loan = health_loan_assistance(HealthLoanProgram::National, 0).income;
    let FederalIncome::HealthLoanAssistance { data } = &mut loan else { unreachable!() };
    data.amount = 100049;
    data.confirmed = false;
    let returned = FederalIncome::MedicalDistribution {
        data: crate::SaSource {
            medicare_prior: if matches!(
                &MedicalSaAccount::Archer,
                MedicalSaAccount::MedicareAdvantage
            ) {
                Some(MedicareMsaPrior::None)
            } else {
                None
            },
            medicare_from_spouse: if matches!(
                &MedicalSaAccount::Archer,
                MedicalSaAccount::MedicareAdvantage
            ) {
                Some(false)
            } else {
                None
            },
            payment: Some(MedicalSaPayment::ExcessReturned),
            ..sa_source(Sa1099 {
                origin: HealthDocumentOrigin::Form1099Sa,
                gross: 100049,
                box2_earnings: Some(100049),
                box3_code: Some(2),
                box4_value: None,
                account: MedicalSaAccount::Archer,
                ..Default::default()
            })
        },
    };
    let inherited = FederalIncome::MedicalDistribution {
        data: crate::SaSource {
            medicare_prior: if matches!(&MedicalSaAccount::Hsa, MedicalSaAccount::MedicareAdvantage)
            {
                Some(MedicareMsaPrior::None)
            } else {
                None
            },
            medicare_from_spouse: if matches!(
                &MedicalSaAccount::Hsa,
                MedicalSaAccount::MedicareAdvantage
            ) {
                Some(false)
            } else {
                None
            },
            payment: Some(MedicalSaPayment::Inherited {
                died: 20251201,

                beneficiary: MedicalBeneficiary::Nonspouse { payments: vec![] },
            }),
            ..sa_source(Sa1099 {
                origin: HealthDocumentOrigin::Form1099Sa,
                gross: 100049,
                box2_earnings: None,
                box3_code: Some(4),
                box4_value: Some(0),
                account: MedicalSaAccount::Hsa,
                ..Default::default()
            })
        },
    };
    for alternative in [loan, returned, inherited] {
        let mut alternative_return = d.clone();
        alternative_return.income[1].income = alternative;
        let result = supported(&alternative_return);
        assert_eq!(iterative_work(&result, "marketplace_iterative_credit_work"), 399471);
        assert_eq!(iterative_work(&result, "marketplace_iterative_deduction_work"), 320529);
        assert_eq!(value(&result, "agi"), 34969);
        assert_eq!(restart(alternative_return.clone()).draft, alternative_return);
    }
}

// Existing personal-property rental owner suspends1000 of the2000 current
// expenses against1000 rent. Pub974 work must retain that actual owner
// adjustment; pairing raw net−1000 with the adjusted filing0 would erase it.
// Derived current2026 endpoint equals the independent40k business-only oracle.
#[test]
fn iterative_marketplace_retains_personal_rent_owner_expense_adjustment() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    add(&mut d, income(personal_rent_with_history(1000, 2000, 0, 0, true)));
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let result = supported(&d);
    assert_eq!(value(&result, "nonbusiness_personal_rent_expenses"), 1000);
    assert_eq!(iterative_work(&result, "marketplace_iterative_credit_work"), 413403);
    assert_eq!(iterative_work(&result, "marketplace_iterative_deduction_work"), 306597);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Pub974(2025 final), printed52–54, https://www.irs.gov/publications/p974:
// derived2026 annual cent-work endpoint, not an IRS-published return.
// Actual ordinary owners: ScheduleC40000/halfSE2826, employee wages1000,
// educator cost100.49, fee-basis official Form2106 other cost100.49,
// qualifying pre2019 alimony1000.49 and timely SEP1000.49. No expense is
// repeated on the business; teacher and fee-based government work are actual
// qualifying roles. Every deduction is below its existing owner ceiling.
// Work base35972.04; independent Fraction credit sequence493698,418981,
//430256,428478,428602,428594c selects deduction291406c. Filing adjustments
// remain100/100/1000/1000 and selected health2914, giving AGI33060.
#[test]
fn iterative_marketplace_retains_existing_adjustment_owner_cents() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(40000, 1..=12)));
    let business = d.income[0].id;
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { contribution: 100049, rate: 25.0 };
    add(&mut d, wage(0, Owner::Taxpayer, 100000, 0));
    let wage_id = d.income[1].id;
    let a = d.adjustments.as_mut().unwrap();
    a.set_educator_taxpayer(10049);
    a.employee_expenses = vec![EmployeeExpenseClaim {
        improvement_buildings: Vec::new(),
        equipment_assets: vec![],
        dot_hours_of_service: false,
        leased_vehicles: vec![],
        owner: Owner::Taxpayer,
        occupation: "Actual fee-basis government office".into(),
        capacity: EmployeeCapacity::FeeBasisOfficial,
        employers: vec![wage_id],
        local_transportation: 0,
        travel: 0,
        other: 10049,
        meals: 0,
        reimbursements: 0,
        mileage: vec![],
        vehicle_assets: vec![],
    }];
    let mut payment = alimony_record(20180101);
    payment.paid = 100049;
    payment.alimony_due = 100049;
    payment.child_support_due = 0;
    a.alimony_paid = vec![paid_alimony(payment)];
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 428594);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 291406);
    assert_eq!(value(&r, &alloc::format!("business_{business}_sep_deduction")), 1000);
    assert_eq!(value(&r, "employee_expenses_1_line_10"), 100);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 2914);
    assert_eq!(value(&r, "agi"), 33060);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Same primary method/source rules. Derived2026 binding-trade endpoint:
// ScheduleC3000−halfSE212−actualSEP500.49=2287.51 health capacity.
// Independent wages30000 retain income30000 after that actual deduction;
// annual benchmark6600−30000×6.17%=4749 PTC. Rounding SEP500 before the work
// capacity would incorrectly allow another49c of health deduction.
#[test]
fn iterative_marketplace_uses_actual_sep_cents_in_its_trade_capacity() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(3000, 1..=12)));
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { contribution: 50049, rate: 25.0 };
    add(&mut d, wage(0, Owner::Taxpayer, 3000000, 0));
    annual_marketplace_businesses(&mut d);
    d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 474900);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 228751);
    assert_eq!(value(&r, "schedule_1_self_employed_health_insurance"), 2288);
    assert_eq!(value(&r, "agi"), 30000);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
}

// Same Pub974 cent-work rule. Actual ordinary foreign-employer wages are
// admitted by their existing owner without a2555 election. They are not W2
// duplicates. Derived current2026 independent Fraction bases37174.49 and
//38174.49 select the two work pairs below; filing wages still round normally.
#[test]
fn iterative_marketplace_retains_foreign_employer_source_cents() {
    for (gross, credit, deduction, agi) in
        [(49, 413399, 306601, 34108), (100049, 399478, 320522, 34969)]
    {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(marketplace_business(40000, 1..=12)));
        add(
            &mut d,
            income(FederalIncome::ForeignEmployment { data: ForeignWages { wages: gross } }),
        );
        annual_marketplace_businesses(&mut d);
        d.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
        let before = d.clone();
        let r = supported(&d);
        assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), credit);
        assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), deduction);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(d, before);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

// Notice2008-51, https://www.irs.gov/pub/irs-drop/n-08-51.pdf, pp4–5:
// lifetime funding use survives a later testing-period failure. Derived2026
// complete-return cases; historical amounts do not become current income or
// deposits. The source history is distinct from existing2025 recapture facts.
#[test]
fn hsa_earlier_lifetime_funding_blocks_reuse_without_current_income() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 100_000);
    h.uses = vec![HsaFundingUseHistory {
        funded: 20240615,
        coverage: HsaCoverage::Family,
        amount: 200_000,
    }];
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let actual = supported(&d);
    let mut control = d.clone();
    control.adjustments.as_mut().unwrap().hsa[0].uses.clear();
    assert_eq!(actual, supported(&control));
    assert_eq!(value(&actual, "adjustments"), 1000);
    assert_eq!(restart(d.clone()).draft, d);
    current_hsa_funding(&mut d, 2000);
    supported(&d);
    d.adjustments.as_mut().unwrap().hsa[0].uses =
        control.adjustments.as_ref().unwrap().hsa[0].uses.clone();
    d.adjustments.as_mut().unwrap().hsa[0].uses.push(HsaFundingUseHistory {
        funded: 20070402,
        coverage: HsaCoverage::Family,
        amount: 200_000,
    });
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
fn hsa_earlier_funding_history_uses_real_dates_and_separate_owner() {
    for (date, expected) in [(20070101, true), (20240229, true), (20241231, true)] {
        let fact =
            HsaFundingUseHistory { funded: date, coverage: HsaCoverage::SelfOnly, amount: 1 };
        assert_eq!(fact.valid(), expected, "date {date}");
    }
    let mut d = complete(FederalStatus::Joint);
    current_hsa_funding(&mut d, 2000);
    let mut spouse = hsa_entry(Owner::Spouse, HsaCoverage::SelfOnly, 0);
    spouse.uses = vec![HsaFundingUseHistory {
        funded: 20240615,
        coverage: HsaCoverage::Family,
        amount: 200_000,
    }];
    d.adjustments.as_mut().unwrap().hsa.push(spouse);
    // A spouse's lifetime use does not consume this owner's transfer.
    supported(&d);
    d.adjustments.as_mut().unwrap().hsa[0].uses =
        d.adjustments.as_ref().unwrap().hsa[1].uses.clone();
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

// Derived current2026 combination of Pub969(2025 final) p6 Examples1/2
// (https://www.irs.gov/pub/irs-pdf/p969.pdf) with one complete sole-proprietor
// policy. Pub974(2025 final) pp52–54 retains cents throughout the method.
// Synthetic sources: business40000, halfSE2826, annual premium7200 and
// benchmark6600, advance3600, single contiguous family/FPL15650, no other
// sources. Independent Fraction calculations in the dated evidence give
// bases45011.50/40715.67 and selected Step4/5 pairs below. This is not a
// published complete-return oracle. The source publication's dates stay intact.
#[test]
fn iterative_marketplace_uses_published_hsa_recapture_cents_once() {
    for (birth, self_months, eligible_months, loss, credit, deduction) in
        [(19720615, 0, 1, 20260601, 303042, 416958), (19860615, 10, 12, 20260301, 364222, 355778)]
    {
        let mut d = empty_return(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        add(&mut d, income(marketplace_business(40000, 1..=12)));
        annual_marketplace_businesses(&mut d);
        let mut months = vec![HsaCoverage::None; 12 - eligible_months];
        months.extend(vec![HsaCoverage::SelfOnly; self_months]);
        months.resize(12, HsaCoverage::Family);
        let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
        h.last_month_history = Some(HsaLastMonthHistory {
            months,
            contributions: 855000,
            archer: 0,
            family: None,
            loss: Some(HsaEligibilityLoss { date: loss, reason: HsaLossReason::Other }),
        });
        let a = d.adjustments.as_mut().unwrap();
        a.hsa = vec![h];
        a.health_marketplace.method = HealthPtcMethod::Iterative;
        let before = d.clone();
        let r = supported(&d);
        assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), credit);
        assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), deduction);
        assert_eq!(credit + deduction, 720000);
        assert_eq!(d, before);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

// Derived 2026 IRA/PTC monetary boundary. Pub974(2025 final), printed52,
// retains cents inside Iterative work; Pub590-A(2025) Worksheet1-2 lines5–7
// compares actual contributions, election, compensation and deduction limit.
// https://www.irs.gov/publications/p974
// https://www.irs.gov/publications/p590a
// Synthetic one-business40000/halfSE2826, annual7200 premium/6600 benchmark,
// singleFPL15650. Actual49c is deductible work49c, filing0; actual1000.49
// minus elected500.50 is work499.99, while the existing filing inputs yield
// deduction499/basis501. Preserve those annual filing outputs here. Independent
// Fraction bases37173.51/36674.01 select the Step4/5 pairs below. Covered
// claims must omit the IRA residual before their stage and retain it afterward.
#[test]
fn iterative_marketplace_keeps_ira_source_cents_after_nondeductible_election() {
    for covered in [false, true] {
        for (traditional, election, filing, credit, deduction) in
            [(49, 0, 0, 413406, 306594), (100049, 50050, 499, 419282, 300718)]
        {
            let mut d = empty_return(FederalStatus::Single);
            add(&mut d, income(marketplace_business(40000, 1..=12)));
            annual_marketplace_businesses(&mut d);
            if covered {
                ordered_workplace_plan(&mut d);
            }
            let mut iras =
                ira_facts(traditional, 0, if covered { Answer::Yes } else { Answer::No });
            iras.taxpayer.nondeductible_election = election;
            let a = d.adjustments.as_mut().unwrap();
            a.iras = iras;
            a.health_marketplace.method = HealthPtcMethod::Iterative;
            let before = d.clone();
            let r = supported(&d);
            assert_eq!(value(&r, "ira_deduction"), filing);
            assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), credit);
            assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), deduction);
            assert_eq!(credit + deduction, 720000);
            assert_eq!(d, before);
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

// Derived current2026 monetary combination, not an IRS complete-return oracle.
// Pub525(2025), Section501(c)(18)(D) contributions, preserves full W-2 wages
// and deducts qualified codeH amounts separately (Schedule1,24f):
// https://www.irs.gov/publications/p525 . Pub974(2025) p52 work retains cents.
// Synthetic wages10000/business30000/halfSE2120 give workbase37880. Actual
// confirmed codeH49c is zero filing deduction, but workbase37879.51. Independent
// Fraction calculation selects credit404113c/deduction315887c (versus
//404110c/315890c without this source). No IRA claim or other source is entered.
#[test]
fn iterative_marketplace_uses_confirmed_pension_owner_source_cents_once() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(marketplace_business(30000, 1..=12)));
    let mut w2 = wage(0, Owner::Taxpayer, 1000000, 0);
    let FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source { form: data, .. }) } =
        &mut w2.income
    else {
        unreachable!()
    };
    data.retirement_plan = true;
    add(&mut d, w2);
    annual_marketplace_businesses(&mut d);
    let a = d.adjustments.as_mut().unwrap();
    a.pension_contributions = vec![PensionContributions {
        owner: Owner::Taxpayer,
        contributions: 49,
        compensation: 1000000,
        other_deferrals: 0,
        confirmed: true,
    }];
    a.health_marketplace.method = HealthPtcMethod::Iterative;
    let before = d.clone();
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_half_self_employment_tax"), 2120);
    assert_eq!(value(&r, "pension_contribution_deduction"), 0);
    assert_eq!(return_line(&r, "1a"), 1000000);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 404113);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 315887);
    assert_eq!(d, before);
    assert_eq!(restart(d.clone()).draft, d);
    // An unconfirmed optional claim never supplies a cent residual. Its known
    // wages and all mandatory source income remain present.
    d.adjustments.as_mut().unwrap().pension_contributions[0].confirmed = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_PENSION_CONTRIBUTIONS_LEFT_OUT"));
    assert_eq!(return_line(&r, "1a"), 1000000);
    assert_eq!(iterative_work(&r, "marketplace_iterative_credit_work"), 404110);
    assert_eq!(iterative_work(&r, "marketplace_iterative_deduction_work"), 315890);
}

// Derived source conservation control, not a published complete-return oracle.
// Pub974(2025) p52 retains cents in Iterative work. Pub527 ch.5 and Pub925
// distinguish admitted nonpassive rent from a passive-loss allocation:
// https://www.irs.gov/publications/p974
// https://www.irs.gov/publications/p925
// The existing building owner supplies10000 depreciation. Rent10000.49
// therefore admits49c, while its filing row rounds0. With no prior loss,
// a professional rental's49c and an unconstrained business's49c supply the
// same work income. Neither changes the filed Schedule1 half-SE deduction.
#[test]
fn iterative_marketplace_retains_admitted_nonpassive_rental_cents() {
    let mut base = empty_return(FederalStatus::Single);
    add(&mut base, income(marketplace_business(30000, 1..=12)));
    add(&mut base, wage(0, Owner::Taxpayer, 1000000, 0));
    annual_marketplace_businesses(&mut base);
    base.adjustments.as_mut().unwrap().health_marketplace.method = HealthPtcMethod::Iterative;
    let mut rental_return = base.clone();
    make_professional(&mut rental_return, Owner::Taxpayer, false);
    let mut source = rental(10000, 0);
    source.rent += 49;
    source.participation = RentalParticipation::Professional;
    add(&mut rental_return, income(FederalIncome::Rental { data: source }));
    let before = rental_return.clone();
    let owner = crate::extended_rules::rentals(
        &rental_return,
        200000,
        &mut FederalResult::empty(rental_return.revision),
    );
    assert_eq!(owner.income, 0);
    assert_eq!(owner.health_work_income_residual_cents, 49);
    let rental_result = supported(&rental_return);
    let base_result = supported(&base);
    let FederalIncome::Business { data } = &mut base.income[0].income else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts += 49;
    let business_result = supported(&base);
    for id in ["marketplace_iterative_credit_work", "marketplace_iterative_deduction_work"] {
        assert_eq!(iterative_work(&rental_result, id), iterative_work(&business_result, id));
        assert_ne!(iterative_work(&rental_result, id), iterative_work(&base_result, id));
    }
    assert_eq!(value(&rental_result, "schedule_1_half_self_employment_tax"), 2120);
    assert_eq!(value(&business_result, "schedule_1_half_self_employment_tax"), 2120);
    assert_eq!(rental_return, before);
    assert_eq!(restart(rental_return.clone()).draft, rental_return);
    // Actual prior loss belongs to its release owner, not this residual.
    let FederalIncome::Rental { data } = &mut rental_return.income.last_mut().unwrap().income
    else {
        unreachable!()
    };
    data.history.regular = 100;
    let owner = crate::extended_rules::rentals(
        &rental_return,
        200000,
        &mut FederalResult::empty(rental_return.revision),
    );
    assert_eq!(owner.health_work_income_residual_cents, 0);
}

#[test]
fn marketplace_claimable_filer_family_catalog_preserves_other_actual_people() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    let child = d.dependents[0].id;
    assert_eq!(crate::marketplace::family_people(&d), vec![-1, -2, child]);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    assert_eq!(crate::marketplace::family_people(&d), vec![-2]);
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    assert_eq!(crate::marketplace::family_people(&d), vec![-1]);
}

// D-2026-10-19-12/-14: Form 8889 foots on its printed whole-dollar lines and the Schedule 1
// deduction is the printed line 13. Five months give a 1,833.33 limit; with 500.60 from the
// employer, line 8 prints 1,833, line 11 prints 501 and line 12 prints 1,332, while the
// exact room (1,332.73) used to print 1,333 and carry 1,333 to Schedule 1.
#[test]
fn form_8889_foots_on_printed_lines_and_the_deduction_is_line_13() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 133273);
    h.employer = 50060;
    h.employer_outside = 50060;
    let mut months = vec![HsaCoverage::None; 12];
    months[..5].fill(HsaCoverage::SelfOnly);
    h.coverage = HsaEligibility::Monthly { months };
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    let line = |n: &str| {
        r.lines
            .iter()
            .find(|l| {
                l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8889 && f.line == n)
            })
            .map(|l| l.amount)
            .unwrap_or_else(|| panic!("Form 8889 line {n}"))
    };
    assert!(["2", "3", "5", "6", "8", "9", "11", "12", "13"].iter().all(|n| line(n) % 100 == 0));
    assert_eq!(line("8") - line("11"), line("12"));
    assert_eq!(line("13"), line("2").min(line("12")));
    assert_eq!(line("13"), 133200);
    assert_eq!(value(&r, "adjustments") * 100, line("13"));
}

// Form 5329 Part VII foots on its printed lines: unused room 399.40 and distributions 100.40
// print 399 and 100, so line 45 is 499 (the exact sum, 499.80, used to print 500), line 46 is
// 1,000 - 499 = 501 and line 49 is 6% of the printed line 48 (smaller of 48 and the value).
#[test]
fn form_5329_part_vii_foots_on_printed_lines_and_line_49_is_the_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 400060);
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260701, amount: 10040 }],
        None,
        None,
    );
    h.excess_accounting = Some(hsa_excess_accounting(
        1000000,
        Some(HsaPriorExcess { line_48: 100000, line_49: 6000 }),
        0,
        0,
        0,
    ));
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    let r = supported(&d);
    let line = |n: u32| {
        r.lines
            .iter()
            .find(|l| {
                l.id == alloc::format!("hsa_1_excess_line{n}")
                    && l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form5329)
            })
            .map(|l| l.amount / 100)
            .unwrap_or_else(|| panic!("Form 5329 line {n}"))
    };
    assert!(
        r.lines
            .iter()
            .filter(|l| l.id.starts_with("hsa_1_excess_line"))
            .all(|l| l.amount % 100 == 0)
    );
    assert_eq!(line(45), line(43) + line(44));
    assert_eq!(line(46), (line(42) - line(45)).max(0));
    assert_eq!(line(48), line(46) + line(47));
    assert_eq!(line(45), 499);
    assert_eq!(line(46), 501);
    assert_eq!(line(49), line(48).min(10000) * 6 / 100);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), line(49));
}
