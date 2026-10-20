use super::*;

#[test]
fn one_state_refund_from_two_prior_payment_years_allocates_before_separate_tax_benefits() {
    let history = two_year_refund_history();
    assert_eq!(
        history.multi_year_breakdown(400_000).unwrap(),
        Some(vec![(2024, 300_000, 300_000), (2025, 100_000, 50_000)])
    );
    assert_eq!(history.taxable(400_000).unwrap(), 350_000);

    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, income(FederalIncome::TaxRefund { amount: 400_000, withholding: 0, history }));
    let result = supported(&d);
    assert_eq!(value(&result, "state_refund_2_2024_allocated"), 3_000);
    assert_eq!(value(&result, "state_refund_2_2024_taxable"), 3_000);
    assert_eq!(value(&result, "state_refund_2_2025_allocated"), 1_000);
    assert_eq!(value(&result, "state_refund_2_2025_taxable"), 500);
    assert_eq!(value(&result, "taxable_state_tax_refund"), 3_500);
    assert_eq!(value(&result, "agi"), value(&before, "agi") + 3_500);
    assert_eq!(return_line(&result, "8"), 350_000);
    assert_eq!(
        value(&result, "alternative_minimum_taxable_income"),
        value(&before, "alternative_minimum_taxable_income")
    );
    assert!(result.review_notes.iter().any(|note| {
        note.contains("allocated $3000.00 to 2024")
            && note.contains("$1000.00 to 2025")
            && note.contains("total $3500.00 on Schedule 1")
    }));
    let summary = result
        .review_notes
        .iter()
        .map(|note| &note.display)
        .find(|message| message.key == "NOTE_STATE_REFUND_PRIOR_ONLY")
        .unwrap();
    assert_eq!(summary.args, vec![MessageArg::Number(2), MessageArg::usd_cents(350_000)]);
    let allocations: Vec<_> = result
        .review_notes
        .iter()
        .map(|note| &note.display)
        .filter(|message| message.key == "NOTE_STATE_REFUND_PRIOR_ALLOCATION")
        .collect();
    assert_eq!(allocations.len(), 2);
    assert_eq!(
        allocations[0].args,
        vec![
            MessageArg::Number(2),
            MessageArg::usd_cents(300_000),
            MessageArg::Number(2024),
            MessageArg::usd_cents(300_000)
        ]
    );
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn repayment_comparison_matches_separate_and_joint_historical_filing_units() {
    let source = repayment_comparison_draft(RepaymentMethod::Credit);
    let mut d = complete(FederalStatus::Joint);
    d.deductions.as_mut().unwrap().repayment = source.deductions.unwrap().repayment;
    add(&mut d, prior_unemployment_repayment(250000, Owner::Taxpayer));
    add(&mut d, prior_unemployment_repayment(250000, Owner::Spouse));
    let years = repayment_years_mut(&mut d);
    years[0].filers = RepaymentFilers::Both;
    years[0].status = FederalStatus::Joint;
    years[0].standard = 3150000;
    years[0].income_tax = 474600;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 4146);
    assert_eq!(value(&r, "repayment_credit"), 600);
    let years = repayment_years_mut(&mut d);
    years[0].filers = RepaymentFilers::Taxpayer;
    years[0].status = FederalStatus::Single;
    years[0].standard = 1575000;
    years[0].income_tax = 795500;
    let mut spouse = years[0].clone();
    spouse.filers = RepaymentFilers::Spouse;
    years.push(spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    let years = repayment_years_mut(&mut d);
    years.reverse();
    d.income.reverse();
    assert_eq!(supported(&d).refund, r.refund);
}

#[test]
fn repayment_senior_recomputes_schedule_r_and_exempt_pensions() {
    for (year, standard, original, after_tax) in [(2023, 15700, 633, 93), (2024, 16550, 548, 8)] {
        let mut d = repayment_senior_draft(year);
        let years = repayment_years_mut(&mut d);
        years[0].wages = 1000000;
        years[0].benefits = 1200000;
        years[0].standard = standard * 100;
        years[0].income_tax = original * 100;
        years[0].elderly = RepaymentElderly::Facts { exempt_pensions: vec![] };
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_elderly_credit"), 0);
        assert_eq!(value(&r, "repayment_year_0_refigured_elderly_credit"), 38);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), after_tax);
        assert_eq!(value(&r, "repayment_credit"), 540);
        let years = repayment_years_mut(&mut d);
        years[0].elderly = RepaymentElderly::Facts {
            exempt_pensions: vec![ExemptPension {
                label: "Original veterans pension".into(),
                amount: 100000,
            }],
        };
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_refigured_elderly_credit"), 0);
        assert_eq!(value(&r, "repayment_credit"), 502);
    }
}

#[test]
fn repayment_senior_assumes_no_historical_ssns_or_pension_credit_when_not_entered() {
    let mut d = repayment_senior_draft(2025);
    let years = repayment_years_mut(&mut d);
    years[0].standard = 1775000;
    years[0].income_tax = 619500;
    unsupported(&d, "Repayment comparison");
    let years = repayment_years_mut(&mut d);
    years[0].ssns = RepaymentSsns::Spouse;
    unsupported(&d, "Repayment comparison");
    let mut d = repayment_senior_draft(2023);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 1000000;
    years[0].benefits = 1200000;
    years[0].standard = 1570000;
    years[0].income_tax = 63300;
    // Pension records not entered: no elderly credit in either calculation.
    let r = supported(&d);
    assert!(noted(&r, "WARN_REPAYMENT_ELDERLY_ASSUMED_NONE"), "{r:?}");
    assert_eq!(value(&r, "repayment_year_0_original_elderly_credit"), 0);
    for pension in [
        ExemptPension { label: "".into(), amount: 0 },
        ExemptPension { label: "Source".into(), amount: -1 },
    ] {
        let mut bad = d.clone();
        let years = repayment_years_mut(&mut bad);
        years[0].elderly = RepaymentElderly::Facts { exempt_pensions: vec![pension] };
        assert!(!bad.valid_stored());
    }
}

#[test]
fn repayment_senior_respects_original_ssn_denial_and_separate_filing() {
    for (status, ssns, standard, original, after) in [
        (FederalStatus::Single, RepaymentSsns::Neither, 17750, 7515, 6415),
        (FederalStatus::Separate, RepaymentSsns::NotSupplied, 17350, 7603, 6503),
    ] {
        let mut d = repayment_senior_draft(2025);
        let years = repayment_years_mut(&mut d);
        years[0].status = status;
        years[0].standard = standard * 100;
        years[0].income_tax = original * 100;
        years[0].ssns = ssns;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), 0);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), after);
        assert_eq!(value(&r, "repayment_credit"), 1100);
    }
}

#[test]
fn repayment_social_uses_original_separate_return_living_arrangements() {
    for (apart, original_social, refigured_social, original, credit) in
        [(true, 13850, 9600, 3545, 1110), (false, 17000, 17000, 3923, 600)]
    {
        let mut d = repayment_social_draft(2025, 17350, original);
        let years = repayment_years_mut(&mut d);
        years[0].status = FederalStatus::Separate;
        years[0].wages = 2500000;
        years[0].separate_lived_apart = Some(apart);
        years[0].social = RepaymentSocial::Benefits {
            amount: 2000000,
            filed_taxable: original_social * 100,
            ordinary: true,
        };
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_refigured_taxable_social"), refigured_social);
        assert_eq!(value(&r, "repayment_credit"), credit);
        let years = repayment_years_mut(&mut d);
        years[0].separate_lived_apart = None;
        // Not supplied: the arrangement matching the filed taxable benefits.
        let r = supported(&d);
        assert!(noted(&r, "WARN_REPAYMENT_LIVING_APART_ASSUMED"), "{r:?}");
        assert_eq!(value(&r, "repayment_credit"), credit);
    }
}

#[test]
fn repayment_retirement_recomputes_social_and_complete_returns_in_all_years() {
    for (year, standard, original, credit) in
        [(2023, 15700, 1199, 766), (2024, 16550, 1098, 750), (2025, 17750, 378, 378)]
    {
        for ira in [false, true] {
            let mut d = repayment_social_draft(year, standard, original);
            let years = repayment_years_mut(&mut d);
            years[0].wages = 0;
            years[0].retirement = vec![if ira {
                RepaymentRetirement::TraditionalIra {
                    gross: 1500000,
                    normal: true,
                    treatment: RepaymentDistribution::Normal,
                }
            } else {
                RepaymentRetirement::Pension {
                    gross: 2000000,
                    taxable: 1500000,
                    normal: true,
                    treatment: RepaymentDistribution::Normal,
                }
            }];
            let r = supported(&d);
            assert_eq!(value(&r, "repayment_year_0_retirement_taxable"), 15000);
            assert_eq!(value(&r, "repayment_year_0_original_agi"), 27500);
            assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 20000);
            assert_eq!(value(&r, "repayment_year_0_original_taxable_social"), 2500);
            assert_eq!(value(&r, "repayment_year_0_refigured_taxable_social"), 0);
            assert_eq!(value(&r, "repayment_credit"), credit);
            assert_eq!(value(&r, "agi"), 60000); // Historical income stays historical.
            assert_eq!(return_line(&r, "31"), credit * 100);
            assert_eq!(supported(&restart(d).draft).refund, r.refund);
        }
    }
}

#[test]
fn repayment_retirement_is_not_earned_income_for_eic() {
    let mut d = repayment_eic_draft();
    d.people.as_mut().unwrap().taxpayer.birth_date = 19640101;
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].retirement = vec![RepaymentRetirement::TraditionalIra {
        gross: 1000000,
        normal: true,
        treatment: RepaymentDistribution::Normal,
    }];
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 20000);
    assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 15000);
    assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 0);
    assert_eq!(value(&r, "repayment_credit"), 428);
    assert_eq!(r.refund, Some(190500));
}

#[test]
fn repayment_retirement_recomputes_senior_phaseout() {
    let mut d = repayment_senior_draft(2025);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].standard = 1775000;
    years[0].income_tax = 969300;
    years[0].ssns = RepaymentSsns::Taxpayer;
    years[0].retirement = vec![RepaymentRetirement::TraditionalIra {
        gross: 7500000,
        normal: true,
        treatment: RepaymentDistribution::Normal,
    }];
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), 5100);
    assert_eq!(value(&r, "repayment_year_0_refigured_senior_deduction"), 5400);
    assert_eq!(value(&r, "repayment_credit"), 1166);
}

#[test]
fn repayment_retirement_combines_cents_per_return_line() {
    let mut d = repayment_social_draft(2025, 17750, 378);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].retirement = vec![
        RepaymentRetirement::TraditionalIra {
            gross: 300000,
            normal: true,
            treatment: RepaymentDistribution::Normal,
        },
        RepaymentRetirement::TraditionalIra {
            gross: 300030,
            normal: true,
            treatment: RepaymentDistribution::Normal,
        },
        RepaymentRetirement::Pension {
            gross: 1100030,
            taxable: 900030,
            normal: true,
            treatment: RepaymentDistribution::Normal,
        },
    ];
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_ira_taxable"), 6000);
    assert_eq!(value(&r, "repayment_year_0_pension_taxable"), 9000);
    assert_eq!(value(&r, "repayment_year_0_retirement_taxable"), 15000);
    assert_eq!(value(&r, "repayment_credit"), 378);
}

#[test]
fn repayment_retirement_requires_confirmed_source_and_reconciles_filed_tax() {
    // An unsupported original-year source figures only the deduction.
    for ira in [false, true] {
        let mut d = repayment_social_draft(2025, 17750, 378);
        let years = repayment_years_mut(&mut d);
        years[0].wages = 0;
        years[0].retirement = vec![if ira {
            RepaymentRetirement::TraditionalIra {
                gross: 1500000,
                normal: false,
                treatment: RepaymentDistribution::Normal,
            }
        } else {
            RepaymentRetirement::Pension {
                gross: 2000000,
                taxable: 1500000,
                normal: false,
                treatment: RepaymentDistribution::Normal,
            }
        }];
        let r = supported(&d);
        assert!(noted(&r, "WARN_REPAYMENT_CREDIT_LEFT_OUT"), "{r:?}");
        assert_eq!(value(&r, "repayment_credit"), 0);
    }
    let mut d = repayment_social_draft(2025, 17750, 378);
    let years = repayment_years_mut(&mut d);
    years[0].retirement = vec![RepaymentRetirement::TraditionalIra {
        gross: 100000,
        normal: true,
        treatment: RepaymentDistribution::Normal,
    }];
    unsupported(&d, "Repayment comparison"); // Income added without reconciling the original return.
}

#[test]
fn repayment_retirement_rejects_invalid_saved_amounts_and_oversized_lists() {
    for mode in 0..5 {
        let mut d = repayment_social_draft(2025, 17750, 378);
        let years = repayment_years_mut(&mut d);
        years[0].retirement = match mode {
            0 => vec![RepaymentRetirement::TraditionalIra {
                gross: -1,
                normal: true,
                treatment: RepaymentDistribution::Normal,
            }],
            1 => vec![RepaymentRetirement::TraditionalIra {
                gross: 100000000001,
                normal: true,
                treatment: RepaymentDistribution::Normal,
            }],
            2 => vec![RepaymentRetirement::Pension {
                gross: 100,
                taxable: 101,
                normal: true,
                treatment: RepaymentDistribution::Normal,
            }],
            3 => vec![RepaymentRetirement::Pension {
                gross: 100,
                taxable: -1,
                normal: true,
                treatment: RepaymentDistribution::Normal,
            }],
            _ => vec![
                RepaymentRetirement::TraditionalIra {
                    gross: 0,
                    normal: true,
                    treatment: RepaymentDistribution::Normal,
                };
                101
            ],
        };
        assert!(!d.valid_stored());
    }
}

#[test]
fn repayment_early_joint_forms_round_separately_and_share_medical_floor() {
    let source = repayment_early_draft(false, true, 6625, 902);
    let mut d = complete(FederalStatus::Joint);
    d.deductions.as_mut().unwrap().repayment = source.deductions.unwrap().repayment;
    add(&mut d, prior_unemployment_repayment(500000, Owner::Taxpayer));
    let years = repayment_years_mut(&mut d);
    years[0].filers = RepaymentFilers::Both;
    years[0].status = FederalStatus::Joint;
    years[0].standard = 3150000;
    years[0].income_tax = 474600;
    years[0].wages = 4999000;
    years[0].retirement = [Owner::Taxpayer, Owner::Spouse]
        .map(|owner| RepaymentRetirement::TraditionalIra {
            gross: 500500,
            normal: true,
            treatment: RepaymentDistribution::Early { owner, simple: false, medical: true },
        })
        .to_vec();
    let r = supported(&d);
    // Original separate forms: 4,005*10%=401; 5,005*10%=501.
    // Revised: 3,630*10%=363 plus 501; historical ordinary tax decreases600.
    assert_eq!(value(&r, "repayment_year_0_original_early_tax"), 902);
    assert_eq!(value(&r, "repayment_year_0_refigured_early_tax"), 864);
    assert_eq!(value(&r, "repayment_credit"), 638);
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_historical_amt_supports_separate_filer_income_surcharge() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].status = FederalStatus::Separate;
    years[0].exempt_interest = 84535000;
    years[0].private_activity_interest = 84535000;
    years[0].income_tax = 25670700;
    let r = supported(&d);
    // IRS2025 example: AMTI920,350 adds5,000 before applying the tax.
    assert_eq!(value(&r, "repayment_year_0_original_amti"), 925350);
    assert_eq!(value(&r, "repayment_year_0_original_tentative_minimum_tax"), 256707);
    assert_eq!(value(&r, "repayment_year_0_refigured_tentative_minimum_tax"), 254957);
    assert_eq!(value(&r, "repayment_credit"), 1750);
}

#[test]
fn repayment_amt_sources_use_separate_preferential_gain_pool() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let mut sale = historical_sale(GainTerm::Long, 150000, 100000);
    sale.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: 5000000, wash_sale: 0 },
    };
    years[0].capital = historical_capital(vec![sale], 0, 0);
    years[0].income_tax = 1545500;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_amti"), 175000);
    assert_eq!(value(&r, "repayment_year_0_original_tentative_minimum_tax"), 13035);
    assert_eq!(value(&r, "repayment_year_0_refigured_tentative_minimum_tax"), 12285);
    assert_eq!(value(&r, "repayment_year_0_original_amt"), 0);
    assert_eq!(value(&r, "repayment_credit"), 1100);
}

#[test]
fn repayment_independent_exclusion_sources_reconcile_and_reject_missing_continuation() {
    for mode in 0..3 {
        let mut d = repayment_exclusion_loss_chain();
        let years = repayment_years_mut(&mut d);
        if mode == 2 {
            years[1].capital_exclusions = Some(vec![]);
        } else if let RepaymentCapital::Records { exclusion, .. } = &mut years[1].capital {
            if mode == 0 {
                *exclusion = None;
            } else {
                exclusion.as_mut().unwrap().short += 100;
            }
        }
        unsupported(&d, "Repayment minimum-tax credit");
    }
}

#[test]
fn married_separate_salt_cap_halves_after_phase_down_for_return_and_form_172() {
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 25_260_000, 0);
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 3_000_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    // $100 over the $252,500 MFS threshold reduces the full cap by $30,
    // then halves it. Halving the cap before phase-down would yield $20,170.
    let current = supported(&d);
    assert_eq!(value(&current, "deduction"), 20_185);

    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000_000)]);
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic E Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "qsbs_2_excluded_gain"), 1_000);
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 1_150);
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}

#[test]
fn qualified_charitable_distribution_reduces_4b_only() {
    // Boundary: 1099-R code 7 paired with a QCD (code Y is optional for
    // 2026), a $3,000 qualified charitable distribution from a $10,000 IRA
    // distribution, well within the 2026 $111,000 QCD owner limit that the
    // App records. The QCD excludes $3,000 from 4b, but 4a keeps reporting
    // the full $10,000 gross distribution.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: native_retirement(
                RetirementKind::Ira,
                1_000_000,
                RetirementTaxable::Payer { amount: 1_000_000 },
            ),
        }),
    );
    d.adjustments.as_mut().unwrap().qualified_charity =
        vec![qcd(Owner::Taxpayer, 20260601, 3000, 0)];
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 700_000);
    assert_eq!(value(&r, "qcd_exclusion"), 3_000);
}
