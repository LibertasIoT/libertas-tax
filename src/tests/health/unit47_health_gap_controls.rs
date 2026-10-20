// Actual request controls for health-family keys absent from the frozen602
// Save witness set. Each exported refusal proves its own native branch;
// shared keys do not imply exhaustive coverage of every compound condition.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;

fn gap_document_case(
    case: &str,
    session: &mut Interview,
    bad: Vec<IncomeDocumentItem>,
    good: Vec<IncomeDocumentItem>,
    expected: (&str, &str),
) {
    let invalid = save_topic(&session.draft, TaxSection::OtherIncome, bad);
    let estimate = credit_estimate_documents_from_save(&invalid);
    credit_refusal(case, session, invalid, expected.0, expected.1);
    credit_refusal(
        &alloc::format!("{case}-estimate"),
        session,
        estimate,
        expected.0,
        expected.1,
    );
    let corrected = save_topic(&session.draft, TaxSection::OtherIncome, good);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = send(session, credit_estimate_documents_from_save(&corrected));
    assert!(
        matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. })),
        "{case}: {reply:?}"
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    let saved = session.draft.clone();
    let page = open_topic(session, TaxSection::OtherIncome);
    accepted(&send(session, page));
    assert_eq!(session.draft, saved, "{case}: reopen/resave");
    let mut cold = restart(saved.clone());
    let page = open_topic(&mut cold, TaxSection::OtherIncome);
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved, "{case}: restart/reopen/resave");
}

fn gap_sa() -> Sa1099 {
    Sa1099 {
        origin: HealthDocumentOrigin::Form1099Sa,
        gross: 100,
        account: MedicalSaAccount::Hsa,
        payment: MedicalSaPayment::Own {
            nonmedical: vec![],
            medical: 100,
            rollover: None,
            disabled_since: None,
        },
    }
}
fn gap_rollover() -> Sa1099 {
    let mut source = gap_sa();
    source.gross = 200;
    source.payment = MedicalSaPayment::Own {
        nonmedical: vec![],
        medical: 0,
        disabled_since: None,
        rollover: Some(MedicalRollover {
            preceding: None,
            receipts: vec![MedicalRolloverReceipt {
                received: 20260302,
                deposits: (0..2)
                    .map(|_| MedicalRolloverDeposit {
                        deposited: 20260302,
                        amount: 100,
                        destination: MedicalRolloverDestination::Hsa,
                        character: MedicalDepositCharacter::Cash {
                            year: 2026,
                            deadline: 20270415,
                        },
                    })
                    .collect(),
            }],
        }),
    };
    source
}
fn gap_sa_rows(source: Sa1099) -> Vec<IncomeDocumentItem> {
    vec![
        document(0, IncomeDocument::Sa1099(gap_sa())),
        document(0, IncomeDocument::Sa1099(gap_sa())),
        document(0, IncomeDocument::Sa1099(source)),
    ]
}

#[test]
fn unit47_health_gap_medical_account_bounds_and_second_deposit_paths() {
    for branch in 0..10 {
        let mut good = if branch >= 4 {
            gap_rollover()
        } else {
            gap_sa()
        };
        match branch {
            0 => good.payment = MedicalSaPayment::ExcessReturned { earnings: Some(100) },
            1 | 2 => {
                good.origin = HealthDocumentOrigin::TrusteeStatement;
                good.payment = MedicalSaPayment::Unreported {
                    event: if branch == 1 {
                        MedicalDeemed::Terminated {
                            occurred: 20260302,
                            january_value: 100,
                        }
                    } else {
                        MedicalDeemed::Pledged {
                            assets: vec![MedicalAccountWithdrawal {
                                date: 20260302,
                                amount: 100,
                            }],
                        }
                    },
                    disabled_since: None,
                };
            }
            3 => {
                good.payment = MedicalSaPayment::Inherited {
                    died: 20260101,
                    value: 100,
                    beneficiary: MedicalBeneficiary::Nonspouse { payments: vec![] },
                }
            }
            9 => {
                let MedicalSaPayment::Own { nonmedical, .. } = &mut good.payment else {
                    unreachable!()
                };
                *nonmedical = (0..99)
                    .map(|_| MedicalAccountWithdrawal {
                        date: 20260302,
                        amount: 1,
                    })
                    .collect();
                good.gross += 99;
            }
            _ => {}
        }
        let mut bad = good.clone();
        let (case, key, local) = match branch {
            0 => {
                bad.payment = MedicalSaPayment::ExcessReturned {
                    earnings: Some(101),
                };
                (
                    "health-sa-earnings",
                    "SA1099_EARNINGS_ABOVE_GROSS",
                    "payment/earnings",
                )
            }
            1 | 2 => {
                bad.gross += 1;
                (
                    if branch == 1 {
                        "health-sa-terminated"
                    } else {
                        "health-sa-pledged"
                    },
                    "SA1099_EVENT_VALUE_NOT_GROSS",
                    "payment/event",
                )
            }
            3 => {
                bad.account = MedicalSaAccount::MedicareFromSpouse;
                (
                    "health-sa-inherited-spouse",
                    "SA1099_SPOUSE_ACCOUNT_IS_OWN",
                    "payment",
                )
            }
            4 => {
                bad.account = MedicalSaAccount::MedicareAdvantage {
                    prior: MedicareMsaPrior::None,
                };
                (
                    "health-sa-rollover-account",
                    "SA1099_ROLLOVER_NOT_ARCHER",
                    "payment/rollover",
                )
            }
            5..=8 => {
                let MedicalSaPayment::Own {
                    rollover: Some(rollover),
                    ..
                } = &mut bad.payment
                else {
                    unreachable!()
                };
                let part = &mut rollover.receipts[0].deposits[1];
                match branch {
                    5 => {
                        part.deposited = 20260301;
                        (
                            "health-sa-before-receipt",
                            "SA1099_ROLLOVER_BEFORE_RECEIPT",
                            "payment/rollover/receipts/0/deposits/1/deposited",
                        )
                    }
                    6 => {
                        part.character = MedicalDepositCharacter::Cash {
                            year: 2024,
                            deadline: 20270415,
                        };
                        (
                            "health-sa-deposit-year",
                            "MEDICAL_DEPOSIT_YEAR_INVALID",
                            "payment/rollover/receipts/0/deposits/1/deposited",
                        )
                    }
                    7 => {
                        part.character = MedicalDepositCharacter::Cash {
                            year: 2026,
                            deadline: 20251231,
                        };
                        (
                            "health-sa-deadline-year",
                            "MEDICAL_DEPOSIT_YEAR_INVALID",
                            "payment/rollover/receipts/0/deposits/1/character/deadline",
                        )
                    }
                    8 => {
                        part.character = MedicalDepositCharacter::Cash {
                            year: 2026,
                            deadline: 20260301,
                        };
                        (
                            "health-sa-after-deadline",
                            "MEDICAL_DEPOSIT_AFTER_DEADLINE",
                            "payment/rollover/receipts/0/deposits/1/deposited",
                        )
                    }
                    _ => unreachable!(),
                }
            }
            9 => {
                let MedicalSaPayment::Own { nonmedical, .. } = &mut bad.payment else {
                    unreachable!()
                };
                nonmedical.push(MedicalAccountWithdrawal {
                    date: 20260302,
                    amount: 1,
                });
                //101 actual parts are valid; refuse only an unreconciled gross.
                (
                    "health-sa-wide-parts-gross",
                    "SA1099_USES_NOT_GROSS",
                    "payment",
                )
            }
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        gap_document_case(
            case,
            &mut session,
            gap_sa_rows(bad),
            gap_sa_rows(good),
            (
                key,
                &alloc::format!("[documents/documents/2/document/{local}]"),
            ),
        );
        let source = session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
            .count();
        assert_eq!(
            source, 3,
            "all three actual medical statements are retained"
        );
    }
}

fn gap_funding_amounts() -> CareFundingAmounts {
    CareFundingAmounts {
        employee_premium: 100,
        employer_premium: 100,
    }
}
fn gap_ltc(funding: CareBenefitFunding) -> IncomeDocument {
    IncomeDocument::Ltc1099(Ltc1099 {
        benefits: 10_000,
        accelerated: 0,
        interest: 0,
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        paid: LtcPaidBasis::NotChecked,
        contract: LtcContractPage::Nonqualified {
            treatment: NonqualifiedCareTreatmentPage::AccidentHealth {
                funding,
                medical_reimbursement: 0,
                permanent_injury: None,
                recovery: None,
            },
        },
        illness: LtcIllness::Chronic,
        insured: None,
    })
}
#[test]
fn unit47_health_gap_ltc_funding_histories_keep_exact_source_rows() {
    for branch in 0..6 {
        let good = match branch {
            0 => CareBenefitFunding::Individual {
                premiums: gap_funding_amounts(),
            },
            1 => CareBenefitFunding::Group {
                years: vec![],
                first_year: Some(CareFirstYearFunding::Estimated {
                    premiums: gap_funding_amounts(),
                }),
            },
            _ => CareBenefitFunding::Group {
                years: vec![
                    CareFundingYear {
                        year: 2024,
                        premiums: gap_funding_amounts(),
                    },
                    CareFundingYear {
                        year: 2025,
                        premiums: gap_funding_amounts(),
                    },
                ],
                first_year: None,
            },
        };
        let mut bad = good.clone();
        let (key, local) = match &mut bad {
            CareBenefitFunding::Individual { premiums } => {
                *premiums = CareFundingAmounts {
                    employee_premium: 0,
                    employer_premium: 0,
                };
                ("LTC_FUNDING_ZERO", "premiums/employee_premium")
            }
            CareBenefitFunding::Group { years, first_year } => match branch {
                1 => {
                    *first_year = None;
                    ("LTC_FUNDING_HISTORY_INVALID", "years")
                }
                2 => {
                    *first_year = Some(CareFirstYearFunding::Actual {
                        premiums: gap_funding_amounts(),
                    });
                    ("LTC_FUNDING_HISTORY_INVALID", "first_year")
                }
                3 => {
                    years[1].year = years[0].year;
                    ("LTC_FUNDING_HISTORY_INVALID", "years/1/year")
                }
                4 => {
                    years[1].premiums = CareFundingAmounts {
                        employee_premium: 0,
                        employer_premium: 0,
                    };
                    ("LTC_FUNDING_ZERO", "years/1/premiums/employee_premium")
                }
                5 => {
                    years[1].year = 2027;
                    ("LTC_FUNDING_HISTORY_INVALID", "years/1/year")
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let rows = |funding| {
            vec![
                document(0, IncomeDocument::Sa1099(gap_sa())),
                document(0, gap_ltc(funding)),
            ]
        };
        gap_document_case(
            &alloc::format!("health-ltc-funding-{branch}"),
            &mut session,
            rows(bad),
            rows(good),
            (
                key,
                &alloc::format!(
                    "[documents/documents/1/document/contract/treatment/funding/{local}]"
                ),
            ),
        );
        // Half the actual insurance premiums were tax-free employer funding.
        assert_eq!(
            value_of(&supported(&session.draft), "taxable_ltc_benefits"),
            50
        );
    }
}

fn gap_estimate(page: &P) -> P {
    match page {
        P::SaveCare { value, .. } => P::EstimateCare {
            user: TEST_USER,
            allowed_injured_source_facts: vec![],
            people: vec![],
            allowed_owners: vec![],
            care_filers: vec![],
            care_months: vec![],
            businesses: vec![],
            farms: vec![],
            partnerships: vec![],
            value: value.clone(),
        },
        P::SaveMarketplace { value, .. } => P::EstimateMarketplace {
            user: TEST_USER,
            people: vec![],
            ptc_people: vec![],
            ptc_families: vec![],
            ptc_policies: vec![],
            ptc_w2: vec![],
            ptc_employees: vec![],
            ptc_nonreturn_people: vec![],
            ptc_claimed_dependents: vec![],
            allowed_marriage: vec![],
            allowed_separate: vec![],
            value: value.clone(),
        },
        P::SaveSelfEmployedBenefits { value, .. } => P::EstimateSelfEmployedBenefits {
            user: TEST_USER,
            ptc_policies: vec![],
            health_activities: vec![],
            health_people: vec![],
            health_wages: vec![],
            value: value.clone(),
        },
        _ => panic!("unexpected health editor"),
    }
}
fn gap_reopen(session: &mut Interview, page: &P) -> P {
    match page {
        P::SaveCare { .. } => open_credits_topic(session, TaxSection::Care),
        P::SaveMarketplace { .. } => open_credits_topic(session, TaxSection::Marketplace),
        P::SaveSelfEmployedBenefits { .. } => {
            let request = go_to(
                &session.draft,
                PAGE_ADJUSTMENTS,
                TaxSection::SelfEmployedBenefits,
            );
            let Reply::Edit(page @ P::SaveSelfEmployedBenefits { .. }) = send(session, request)
            else {
                panic!("actual benefits editor")
            };
            page
        }
        _ => unreachable!(),
    }
}
fn gap_page_case(case: &str, session: &mut Interview, bad: P, good: P, expected: (&str, &str)) {
    let estimate = gap_estimate(&bad);
    credit_refusal(case, session, bad, expected.0, expected.1);
    credit_refusal(
        &alloc::format!("{case}-estimate"),
        session,
        estimate,
        expected.0,
        expected.1,
    );
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = send(session, gap_estimate(&good));
    assert!(
        matches!(reply, Reply::Response(P::RunningEstimate { .. })),
        "{case}: {reply:?}"
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, good.clone()));
    let saved = session.draft.clone();
    let reopened = gap_reopen(session, &good);
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved, "{case}: reopen/resave");
    let mut cold = restart(saved.clone());
    let reopened = gap_reopen(&mut cold, &good);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved, "{case}: restart/reopen/resave");
}
fn gap_benefits_editor(session: &mut Interview) -> P {
    let request = go_to(
        &session.draft,
        PAGE_ADJUSTMENTS,
        TaxSection::SelfEmployedBenefits,
    );
    let Reply::Edit(page @ P::SaveSelfEmployedBenefits { .. }) = send(session, request) else {
        panic!("actual benefits editor")
    };
    page
}

#[test]
fn unit47_health_gap_business_child_premium_uses_second_actual_month() {
    let mut session = at_the_adjustments_hub_with_business(4_000_000);
    let mut good = gap_benefits_editor(&mut session);
    let P::SaveSelfEmployedBenefits { value, .. } = &mut good else {
        unreachable!()
    };
    value.businesses[0].health = SeHealth::Plan {
        establishment: HealthEstablishmentPage::SoleProprietor,
        months: (1..=2)
            .map(|month| BusinessHealthMonth {
                month,
                premiums: 10_000,
                child_increase: 10_000,
                employer_eligible: false,
                reimbursed: None,
            })
            .collect(),
        long_term_care: vec![],
    };
    let mut bad = good.clone();
    let P::SaveSelfEmployedBenefits { value, .. } = &mut bad else {
        unreachable!()
    };
    let SeHealth::Plan { months, .. } = &mut value.businesses[0].health else {
        unreachable!()
    };
    months[1].child_increase += 1;
    gap_page_case(
        "health-business-child-cost",
        &mut session,
        bad,
        good,
        (
            "HEALTH_CHILD_ABOVE_PREMIUMS",
            "[value/businesses/0/health/months/1/child_increase]",
        ),
    );
    // The business gross is preserved independently of health admission.
    assert_eq!(stored_business(&session).1.receipts(), 4_000_000);
}

#[test]
fn unit47_health_gap_reimbursed_coverage_uses_saved_policy_and_activity() {
    let mut session = at_the_adjustments_hub_with_business(4_000_000);
    let activity = stored_business(&session).0;
    let _ = open_credits_topic(&mut session, TaxSection::Marketplace);
    let saved = save_marketplace_and_reopen(
        &mut session,
        marketplace_fixture_page(vec![all_year("Actual premium source", 60_000, 55_000, 0)]),
    );
    let policy = saved.forms[0].id;
    assert!(policy > 0 && activity > 0);
    let mut good = gap_benefits_editor(&mut session);
    let P::SaveSelfEmployedBenefits { value, .. } = &mut good else {
        unreachable!()
    };
    value.policies = (1..=2)
        .map(|month| HealthPolicyAllocationPage {
            policy,
            month,
            activity,
            parts: 1,
            reimbursed: Some(ReimbursedPolicyPremiums {
                premiums: 0,
                coverage: 0,
                child: None,
            }),
        })
        .collect();
    let mut bad = good.clone();
    let P::SaveSelfEmployedBenefits { value, .. } = &mut bad else {
        unreachable!()
    };
    value.policies[1].reimbursed.as_mut().unwrap().coverage = 1;
    gap_page_case(
        "health-policy-reimbursed",
        &mut session,
        bad,
        good,
        (
            "HEALTH_REIMBURSED_COVERAGE_ABOVE_TOTAL",
            "[value/policies/1/reimbursed/coverage]",
        ),
    );
    assert_eq!(stored_marketplace(&session).forms[0].id, policy);
    assert_eq!(stored_business(&session).0, activity);
}

#[test]
fn unit47_health_gap_care_provider_required_preserves_actual_credit() {
    let mut session = household_return(m2_children());
    let mut good = open_credits_topic(&mut session, TaxSection::Care);
    fill_care_provider(&mut good);
    let mut bad = good.clone();
    let P::SaveCare { value, .. } = &mut bad else {
        unreachable!()
    };
    value.providers.clear();
    gap_page_case(
        "health-care-provider-needed",
        &mut session,
        bad,
        good,
        ("CARE_COSTS_NEED_A_PROVIDER", "[value/providers]"),
    );
    assert_eq!(value_of(&supported(&session.draft), "care_credit"), 1388);
}
#[test]
fn unit47_health_gap_parent_provider_needs_actual_under_thirteen_child() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut good = open_credits_topic(&mut session, TaxSection::Care);
    let P::SaveCare { value, .. } = &mut good else {
        unreachable!()
    };
    value.providers = vec![CareProvider {
        id: 0,
        name: "Actual care provider".into(),
        address: "1 Source Street".into(),
        identification: ProviderIdentification::Organization,
        household_employee: false,
        paid: 100,
        relationship: ProviderRelationship::NotRelated,
    }];
    let mut bad = good.clone();
    let P::SaveCare { value, .. } = &mut bad else {
        unreachable!()
    };
    value.providers[0].relationship = ProviderRelationship::ParentOfChild;
    gap_page_case(
        "health-care-parent-provider",
        &mut session,
        bad,
        good,
        (
            "CARE_PARENT_PROVIDER_NEEDS_A_CHILD",
            "[value/providers/0/relationship]",
        ),
    );
    assert_eq!(value_of(&supported(&session.draft), "care_credit"), 0);
}
#[test]
fn unit47_health_gap_student_requires_five_actual_months() {
    let mut draft = complete(FederalStatus::Joint);
    draft
        .people
        .as_mut()
        .unwrap()
        .spouse
        .as_mut()
        .unwrap()
        .full_time_student = true;
    let mut session = Interview::new(TEST_USER, draft);
    let mut good = open_credits_topic(&mut session, TaxSection::Care);
    let P::SaveCare { value, .. } = &mut good else {
        unreachable!()
    };
    value.filers = vec![CareFilerInput {
        owner: Owner::Spouse,
        situation: CareFilerSituationInput::Student {
            months: vec![1, 2, 3, 4, 5],
            earned_in_months: None,
        },
    }];
    let mut bad = good.clone();
    let P::SaveCare { value, .. } = &mut bad else {
        unreachable!()
    };
    let CareFilerSituationInput::Student { months, .. } = &mut value.filers[0].situation else {
        unreachable!()
    };
    months.pop();
    gap_page_case(
        "health-care-student-months",
        &mut session,
        bad,
        good,
        (
            "CARE_STUDENT_NEEDS_FIVE_MONTHS",
            "[value/filers/0/situation/months]",
        ),
    );
}
#[test]
fn unit47_health_gap_care_parts_use_accepted_second_wage() {
    for branch in 0..4 {
        let mut session = household_return(m2_children());
        let mut rows = page_documents(&open_topic(&mut session, TaxSection::Wages)).to_vec();
        let mut wage = m2_w2();
        let IncomeDocument::W2(source) = &mut wage.document else {
            unreachable!()
        };
        source.wages = 100_000;
        source.social_wages = 100_000;
        source.social_tax = 6_200;
        source.medicare_wages = 100_000;
        source.medicare_tax = 1_450;
        source.withholding = 0;
        source.more.care_benefits = 10_000;
        rows.push(wage);
        let request = save_topic(&session.draft, TaxSection::Wages, rows);
        accepted(&send(&mut session, request));
        let mut good = open_credits_topic(&mut session, TaxSection::Care);
        fill_care_provider(&mut good);
        let P::SaveCare { value, .. } = &mut good else {
            unreachable!()
        };
        value.plan_maximum = CarePlanMaximum::Plans { maximum: 10_000 };
        assert_eq!(value.benefits.len(), 1);
        let wage_id = value.benefits[0].wage;
        value.benefits[0].in_box_1 = Some(4_000);
        value.benefits[0].forfeited = Some(6_000);
        let mut bad = good.clone();
        let P::SaveCare { value, .. } = &mut bad else {
            unreachable!()
        };
        let path = match branch {
            0 => {
                value.benefits[0].forfeited = Some(6_001);
                "[value/benefits/0/forfeited]"
            }
            1 => {
                value.benefits[0].in_box_1 = Some(10_001);
                value.benefits[0].forfeited = None;
                "[value/benefits/0/in_box_1]"
            }
            2 => {
                value.benefits[0].in_box_1 = Some(-1);
                "[value/benefits/0/in_box_1]"
            }
            3 => {
                value.benefits[0].forfeited = Some(-1);
                "[value/benefits/0/forfeited]"
            }
            _ => unreachable!(),
        };
        gap_page_case(
            &alloc::format!("health-care-box10-{branch}"),
            &mut session,
            bad,
            good,
            ("CARE_BENEFIT_PARTS_ABOVE_BOX_10", path),
        );
        assert!(session.draft.income.iter().any(|entry| entry.id == wage_id && matches!(&entry.income, FederalIncome::Wage { data } if data.care_benefits == 10_000)));
    }
}

fn gap_one_month() -> MarketplacePolicyRecordPage {
    let mut form = all_year("Actual January policy", 100_000, 90_000, 30_000);
    form.amounts = PolicyAmounts::ByMonth {
        months: vec![PolicyMonth {
            month: 1,
            premium: 100_000,
            benchmark: Some(90_000),
            advance: 30_000,
        }],
    };
    form.coverage.retain(|month| month.month == 1);
    form
}
#[test]
fn unit47_health_gap_marketplace_allocations_reject_source_month_and_fraction_bounds() {
    for branch in 0..2 {
        let (mut session, _) = at_the_marketplace_page(complete(FederalStatus::Single));
        let mut value = marketplace_fixture_page(vec![gap_one_month()]);
        value.forms[0].allocation = vec![MarketplaceAllocationPeriodPage {
            from: 1,
            to: 1,
            kind: MarketplaceAllocationKindPage::OtherShared {
                families: vec![MarketplaceFamilySharePage {
                    family: MarketplaceFamilyRefPage::ThisReturn,
                    agreement: MarketplaceAgreement::Agreed {
                        share: MarketplaceFraction {
                            numerator: 1,
                            denominator: 1,
                        },
                    },
                }],
            },
        }];
        let good = save_marketplace(&session.draft, value);
        let mut bad = good.clone();
        let P::SaveMarketplace { value, .. } = &mut bad else {
            unreachable!()
        };
        let (case, key, path) = if branch == 0 {
            value.forms[0].allocation[0].from = 2;
            (
                "health-ptc-allocation-months",
                "PTC_MONTHS_REVERSED",
                "[value/forms/0/allocation/0/to]",
            )
        } else {
            let MarketplaceAllocationKindPage::OtherShared { families } =
                &mut value.forms[0].allocation[0].kind
            else {
                unreachable!()
            };
            families[0].agreement = MarketplaceAgreement::Agreed {
                share: MarketplaceFraction {
                    numerator: 2,
                    denominator: 1,
                },
            };
            (
                "health-ptc-fraction",
                "PTC_SHARE_EXCEEDS_WHOLE",
                "[value/forms/0/allocation/0/kind/families/0/agreement/share/numerator]",
            )
        };
        gap_page_case(case, &mut session, bad, good, (key, path));
        assert_eq!(
            stored_marketplace(&session).forms[0].amounts,
            PolicyAmounts::ByMonth {
                months: vec![PolicyMonth {
                    month: 1,
                    premium: 100_000,
                    benchmark: Some(90_000),
                    advance: 30_000
                }],
            }
        );
    }
}

fn gap_qsehra_fixture() -> (Interview, MarketplaceFactsPage, i64, i64) {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 1_930_000, 0);
    add_child(&mut draft);
    add_child(&mut draft);
    let employee = draft.dependents[0].id;
    let sibling = draft.dependents[1].id;
    let (session, _) = at_the_marketplace_page(draft);
    let mut value = marketplace_fixture_page(vec![gap_one_month()]);
    value.forms[0].coverage[0]
        .people
        .extend([employee, sibling]);
    for person in [employee, sibling] {
        value.eligibility.push(MarketplaceEligibilityMonthPage {
            month: 1,
            person,
            status: MarketplacePersonEligibility::Eligible,
        });
    }
    value.dependents.push(DependentReturn {
        person: employee,
        agi: 1_200_000,
        exempt_interest: 0,
        foreign_exclusion: 0,
        social: 0,
        taxable_social: 0,
    });
    value.dependent_qsehra_employees = vec![employee];
    value.qsehra = vec![MarketplaceQsehraPage {
        id: 0,
        employee,
        source: MarketplaceQsehraSourcePage::Notice {
            employer: "Actual dependent employee employer".into(),
            annual_permitted: 10_000,
        },
        months: vec![MarketplaceQsehraMonthPage {
            month: 1,
            self_only: 10_000,
            actual: 10_000,
            self_benchmark: 20_000,
            covered: vec![employee],
        }],
        determinations: vec![],
    }];
    value.employee_qsehra_facts = vec![MarketplaceQsehraEmployeeFactsPage {
        employee,
        income: Some(MarketplaceQsehraIncomeSource::RequiredReturn),
        household: Some(MarketplaceQsehraEmployeeHouseholdPage::Known {
            spouse: None,
            dependents: vec![],
        }),
    }];
    (session, value, employee, sibling)
}
fn gap_qsehra_components(agi: i64) -> MarketplaceQsehraIncome {
    MarketplaceQsehraIncome {
        agi,
        exempt_interest: 0,
        foreign_exclusion: 0,
        social: 100,
        taxable_social: 100,
    }
}
#[test]
fn unit47_health_gap_qsehra_return_sources_and_social_components_keep_actual_branch_paths() {
    for branch in 0..6 {
        let (mut session, mut value, _, sibling) = gap_qsehra_fixture();
        match branch {
            1 => {
                value.qsehra[0].determinations = vec![MarketplaceQsehraDetermination {
                    from: 1,
                    to: 1,
                    accurate_disclosure: true,
                    unaffordable: true,
                    made_on: 20260101,
                }]
            }
            2 => {
                value.employee_qsehra_facts[0].household =
                    Some(MarketplaceQsehraEmployeeHouseholdPage::Known {
                        spouse: None,
                        dependents: vec![MarketplaceQsehraHouseholdMemberPage {
                            person: sibling,
                            income: Some(MarketplaceQsehraIncomeSource::Documentary {
                                components: gap_qsehra_components(0),
                            }),
                        }],
                    });
            }
            3 => {
                value.dependents[0].social = 100;
                value.dependents[0].taxable_social = 100;
                value.employee_qsehra_facts[0].income =
                    Some(MarketplaceQsehraIncomeSource::Documentary {
                        components: gap_qsehra_components(1_200_000),
                    });
            }
            4 => {
                value.dependents[0].social = 100;
                value.dependents[0].taxable_social = 100;
            }
            5 => {
                value.employee_qsehra_facts[0].household =
                    Some(MarketplaceQsehraEmployeeHouseholdPage::Known {
                        spouse: None,
                        dependents: vec![MarketplaceQsehraHouseholdMemberPage {
                            person: sibling,
                            income: Some(MarketplaceQsehraIncomeSource::Documentary {
                                components: gap_qsehra_components(0),
                            }),
                        }],
                    });
            }
            _ => {}
        }
        let good = save_marketplace(&session.draft, value);
        let mut bad = good.clone();
        let P::SaveMarketplace { value, .. } = &mut bad else {
            unreachable!()
        };
        let (key, path) = match branch {
            0 => {
                value.dependents.clear();
                (
                    "PTC_QSEHRA_EMPLOYEE_RETURN_REQUIRED",
                    "[value/employee_qsehra_facts/0/income]",
                )
            }
            1 => {
                value.qsehra[0].determinations[0].from = 2;
                (
                    "PTC_MONTHS_REVERSED",
                    "[value/qsehra/0/determinations/0/to]",
                )
            }
            2 => {
                let Some(MarketplaceQsehraEmployeeHouseholdPage::Known { dependents, .. }) =
                    &mut value.employee_qsehra_facts[0].household
                else {
                    unreachable!()
                };
                dependents[0].income = Some(MarketplaceQsehraIncomeSource::RequiredReturn);
                (
                    "PTC_QSEHRA_EMPLOYEE_RETURN_REQUIRED",
                    "[value/employee_qsehra_facts/0/household/dependents/0/income]",
                )
            }
            3 => {
                let Some(MarketplaceQsehraIncomeSource::Documentary { components }) =
                    &mut value.employee_qsehra_facts[0].income
                else {
                    unreachable!()
                };
                components.taxable_social = 101;
                (
                    "PTC_QSEHRA_SOCIAL_ABOVE_GROSS",
                    "[value/employee_qsehra_facts/0/income/components/taxable_social]",
                )
            }
            4 => {
                value.dependents[0].taxable_social = 101;
                (
                    "PTC_QSEHRA_SOCIAL_ABOVE_GROSS",
                    "[value/dependents/0/taxable_social]",
                )
            }
            5 => {
                let Some(MarketplaceQsehraEmployeeHouseholdPage::Known { dependents, .. }) =
                    &mut value.employee_qsehra_facts[0].household
                else {
                    unreachable!()
                };
                let Some(MarketplaceQsehraIncomeSource::Documentary { components }) =
                    &mut dependents[0].income
                else {
                    unreachable!()
                };
                components.taxable_social = 101;
                (
                    "PTC_QSEHRA_SOCIAL_ABOVE_GROSS",
                    "[value/employee_qsehra_facts/0/household/dependents/0/income/components/taxable_social]",
                )
            }
            _ => unreachable!(),
        };
        gap_page_case(
            &alloc::format!("health-ptc-qsehra-{branch}"),
            &mut session,
            bad,
            good,
            (key, path),
        );
        if branch == 0 {
            // Independent monthly oracle: claimant $31,300/family3, employee
            // $12,000; the $100 monthly QSEHRA reduction leaves $745 credit.
            assert_eq!(
                value_of(&supported(&session.draft), "marketplace_household_income"),
                31_300
            );
            assert_eq!(
                value_of(&supported(&session.draft), "premium_tax_credit"),
                745
            );
        }
    }
}

fn gap_periods() -> LtcInsured {
    LtcInsured {
        insured: LtcInsuredPerson::OnReturn { person: -1 },
        periods: LtcPeriodsInput::Several {
            method: CarePeriodMethod::EqualRate,
            periods: [(20260601, 20260630), (20260701, 20260731)]
                .into_iter()
                .map(|(starts, ends)| CareBenefitPeriodInput {
                    starts,
                    ends,
                    received: 1_500_000,
                    services: 0,
                    reimbursements: 0,
                    recipient: CareBenefitRecipient::Insured {
                        other_payees: false,
                    },
                    injured_payees: None,
                })
                .collect(),
        },
    }
}
fn gap_qualified_ltc() -> IncomeDocument {
    IncomeDocument::Ltc1099(Ltc1099 {
        benefits: 1_500_000,
        accelerated: 0,
        interest: 0,
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        paid: LtcPaidBasis::PerDiem,
        contract: LtcContractPage::Qualified {
            qualification: CareContractQualification::Qualified,
        },
        illness: LtcIllness::Chronic,
        insured: None,
    })
}
#[test]
fn unit47_health_gap_injured_ltc_payees_use_actual_accepted_sources_and_second_period() {
    for duplicate in [false, true] {
        let mut draft = complete(FederalStatus::Joint);
        draft.injured_spouse = Some(InjuredSpouseAllocation {
            injured: Owner::Taxpayer,
            debt: OffsetDebt::ChildSupport,
            obligated: false,
            community: InjuredCommunityFacts::No,
            student_loan_borrower: None,
            estimated: EstimatedSplit::IrsFormula,
            allocations: vec![],
            claimants: vec![],
        });
        let mut session = Interview::new(TEST_USER, draft);
        let taxpayer = document(0, gap_qualified_ltc());
        let mut spouse = document(0, gap_qualified_ltc());
        spouse.owner = Owner::Spouse;
        let request = save_topic(
            &session.draft,
            TaxSection::OtherIncome,
            vec![
                document(0, IncomeDocument::LtcInsured(gap_periods())),
                taxpayer,
                spouse,
            ],
        );
        accepted(&send(&mut session, request));
        assert_eq!(
            value_of(&supported(&session.draft), "taxable_ltc_benefits"),
            3770
        );
        let sources: Vec<_> = session
            .draft
            .income
            .iter()
            .filter_map(|entry| {
                matches!(entry.income, FederalIncome::LtcBenefits { .. }).then_some(entry.id)
            })
            .collect();
        assert_eq!(sources.len(), 2);
        assert_ne!(sources[0], sources[1]);
        let page = open_topic(&mut session, TaxSection::OtherIncome);
        let mut good = page_documents(&page).to_vec();
        let period_index = good
            .iter()
            .position(|row| matches!(row.document, IncomeDocument::LtcInsured(_)))
            .unwrap();
        let mut row = good.remove(period_index);
        let IncomeDocument::LtcInsured(value) = &mut row.document else {
            unreachable!()
        };
        let LtcPeriodsInput::Several { periods, .. } = &mut value.periods else {
            unreachable!()
        };
        for (period, source) in periods.iter_mut().zip(&sources) {
            period.injured_payees = Some(InjuredLtcPeriodPayeesInput::Payments {
                payments: vec![InjuredLtcPeriodPaymentInput {
                    source: *source,
                    amount: 1_500_000,
                }],
            });
        }
        good.push(row);
        assert!(matches!(good[2].document, IncomeDocument::LtcInsured(_)));
        let mut bad = good.clone();
        let IncomeDocument::LtcInsured(value) = &mut bad[2].document else {
            unreachable!()
        };
        let LtcPeriodsInput::Several { periods, .. } = &mut value.periods else {
            unreachable!()
        };
        let Some(InjuredLtcPeriodPayeesInput::Payments { payments }) =
            &mut periods[1].injured_payees
        else {
            unreachable!()
        };
        let (case, key, path) = if duplicate {
            payments[0].amount = 750_000;
            payments.push(InjuredLtcPeriodPaymentInput {
                source: sources[1],
                amount: 750_000,
            });
            (
                "health-ltc-payee-duplicate",
                "INJURED_LTC_SOURCE_TWICE",
                "[documents/documents/2/document/periods/periods/1/injured_payees/payments/1/source]",
            )
        } else {
            payments[0].amount += 1;
            (
                "health-ltc-payee-total",
                "INJURED_LTC_PAYEE_TOTAL",
                "[documents/documents/2/document/periods/periods/1/injured_payees/payments]",
            )
        };
        gap_document_case(case, &mut session, bad, good, (key, path));
        // Identifying actual payees must not change the canonical Section C
        // exclusion. It changes ownership attribution for Form 8379 only.
        assert_eq!(
            value_of(&supported(&session.draft), "taxable_ltc_benefits"),
            3770
        );
        assert!(
            sources
                .iter()
                .all(|id| session.draft.income.iter().any(|entry| entry.id == *id))
        );
    }
}

// Schema widening reaches preflight before the native source-amount refusal.
#[test]
fn unit47_health_cap_sweep_ltc_preflight_preserves_extreme_source_refusals() {
    for invalid_funding in [true, false] {
        let funding = if invalid_funding {
            CareBenefitFunding::Individual {
                premiums: CareFundingAmounts {
                    employee_premium: i64::MAX,
                    employer_premium: i64::MAX,
                },
            }
        } else {
            CareBenefitFunding::EmployerOnly
        };
        // 2026-10-16 D-2026-10-15-13: full-width known funding is
        // an exact ratio; a negative source and an excessive exclusion refuse.
        let good = gap_ltc(funding);
        let mut bad = good.clone();
        if invalid_funding {
            let IncomeDocument::Ltc1099(source) = &mut bad else { unreachable!() };
            let LtcContractPage::Nonqualified {
                treatment:
                    NonqualifiedCareTreatmentPage::AccidentHealth {
                        funding: CareBenefitFunding::Individual { premiums },
                        ..
                    },
            } = &mut source.contract
            else {
                unreachable!()
            };
            premiums.employee_premium = -1;
        } else {
            let IncomeDocument::Ltc1099(source) = &mut bad else { unreachable!() };
            let LtcContractPage::Nonqualified {
                treatment:
                    NonqualifiedCareTreatmentPage::AccidentHealth {
                        medical_reimbursement,
                        permanent_injury,
                        ..
                    },
            } = &mut source.contract
            else {
                unreachable!()
            };
            *medical_reimbursement = i64::MAX;
            *permanent_injury = Some(CarePermanentInjury {
                amount: i64::MAX,
                kind: CareInjuryKind::Other,
                computed_by_injury: false,
            });
        }
        let (key, path) = if invalid_funding {
            ("LTC_FUNDING_HISTORY_INVALID", "funding/premiums/employee_premium")
        } else {
            ("LTC1099_EXCLUDED_PORTIONS_ABOVE_EMPLOYER", "medical_reimbursement")
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        gap_document_case(
            if invalid_funding { "health-wide-funding-sum" } else { "health-wide-excluded-sum" },
            &mut session,
            vec![document(0, bad)],
            vec![document(0, good)],
            (key, &alloc::format!("[documents/documents/0/document/contract/treatment/{path}]")),
        );
        if invalid_funding {
            assert_eq!(value_of(&supported(&session.draft), "taxable_ltc_benefits"), 50);
        }
    }
}

#[test]
fn unit47_health_cap_sweep_rollover_year_preflight_preserves_refusal() {
    let good = gap_rollover();
    let mut bad = good.clone();
    let MedicalSaPayment::Own { rollover: Some(rollover), .. } = &mut bad.payment else {
        unreachable!()
    };
    rollover.receipts[0].deposits[1].character =
        MedicalDepositCharacter::Cash { year: u32::MAX, deadline: 20270415 };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    gap_document_case(
        "health-wide-rollover-year",
        &mut session,
        vec![document(0, IncomeDocument::Sa1099(bad))],
        vec![document(0, IncomeDocument::Sa1099(good))],
        (
            "MEDICAL_DEPOSIT_YEAR_INVALID",
            "[documents/documents/0/document/payment/rollover/receipts/0/deposits/1/deposited]",
        ),
    );
}
