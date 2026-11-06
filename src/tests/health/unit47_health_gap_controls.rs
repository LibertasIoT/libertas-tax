// Actual request controls for health-family keys absent from the frozen602
// Save witness set. Each exported refusal proves its own native branch;
// shared keys do not imply exhaustive coverage of every compound condition.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;
use crate::tests::cross::interview::save_marketplace_sources;
use crate::tests::support::client::income_pages::{open_documents, save_documents, save_topic};
use alloc::string::{String, ToString};

fn gap_document_case(
    case: &str,
    session: &mut Interview,
    bad: Vec<IncomeDocumentItem>,
    good: Vec<IncomeDocumentItem>,
    expected: (&str, &str),
) {
    if expected.1.contains("/contract/") {
        crate::tests::support::client::information_return_refusal(
            case, session, &bad, good, expected.0, expected.1,
        );
        return;
    }
    let record = good.iter().all(|row| matches!(row.document, IncomeSource::Record { .. }));
    let invalid = if record {
        save_topic(session, TaxSection::LongTermCareBenefits, bad)
    } else {
        save_documents(session, bad)
    };
    let estimate = credit_estimate_documents_from_save(&invalid);
    credit_refusal(case, session, invalid, expected.0, expected.1);
    credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, expected.0, expected.1);
    let corrected = if record {
        save_topic(session, TaxSection::LongTermCareBenefits, good)
    } else {
        save_documents(session, good)
    };
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        credit_estimate_documents_from_save(&corrected),
    );
    assert!(
        if record {
            matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. }))
        } else {
            matches!(reply, Reply::Response(P::Review { .. }))
        },
        "{case}: {reply:?}"
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    let saved = session.draft.clone();
    let page = if record {
        open_topic(session, TaxSection::LongTermCareBenefits)
    } else {
        open_documents(session)
    };
    accepted(&send(session, page));
    assert_eq!(session.draft, saved, "{case}: reopen/resave");
    let mut cold = restart(saved.clone());
    let page = if record {
        open_topic(&mut cold, TaxSection::LongTermCareBenefits)
    } else {
        open_documents(&mut cold)
    };
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved, "{case}: restart/reopen/resave");
}

fn gap_sa() -> crate::SaSource {
    crate::SaSource {
        medicare_prior: if matches!(&MedicalSaAccount::Hsa, MedicalSaAccount::MedicareAdvantage) {
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
        payment: Some(MedicalSaPayment::Own {
            nonmedical: vec![],
            medical: 100,
            rollover: None,
            disabled_since: None,
        }),
        ..sa_source(Sa1099 {
            trustee_payer: Default::default(),
            recipient: Default::default(),
            origin: HealthDocumentOrigin::Form1099Sa,
            gross: 100,
            box2_earnings: None,
            box3_code: Some(1),
            box4_value: None,
            account: MedicalSaAccount::Hsa,
        })
    }
}
fn gap_rollover() -> crate::SaSource {
    let mut source = gap_sa();
    *source.gross_mut() = 200;
    source.payment = Some(MedicalSaPayment::Own {
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
                        character: MedicalDepositCharacter::Cash { year: 2026, deadline: 20270415 },
                    })
                    .collect(),
            }],
        }),
    });
    source.form1099sa_mut().unwrap().distribution_code =
        Some(crate::Form1099SaDistributionCode::NormalDistributions);
    source
}
fn gap_sa_rows(source: crate::SaSource) -> Vec<IncomeDocumentItem> {
    vec![
        document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Sa1099(
                    crate::tests::support::client::income_pages::sa_document(gap_sa()),
                ),
            },
        ),
        document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Sa1099(
                    crate::tests::support::client::income_pages::sa_document(gap_sa()),
                ),
            },
        ),
        document(
            0,
            IncomeSource::OtherIncome {
                value: match &source.form {
                    crate::MedicalSaPaper::Form1099Sa(_) => crate::OtherIncomeDocument::Sa1099(
                        crate::tests::support::client::income_pages::sa_document(source),
                    ),
                    crate::MedicalSaPaper::TrusteeStatement(_) => {
                        crate::OtherIncomeDocument::HsaTrusteeStatement(
                            crate::tests::support::client::income_pages::sa_statement_document(
                                source,
                            ),
                        )
                    }
                },
            },
        ),
    ]
}

fn gap_medical_case(
    case: &str,
    session: &mut Interview,
    bad: Vec<IncomeDocumentItem>,
    mut good: Vec<IncomeDocumentItem>,
    bad_payment: Option<MedicalSaPayment>,
    (key, local): (&str, &str),
    printed: bool,
) {
    use crate::tests::support::client::income_pages::{
        information_return_answers, save_information_return_answers,
    };
    if printed {
        let request = save_documents(session, bad);
        let estimate = credit_estimate_documents_from_save(&request);
        let path = alloc::format!("[documents/other/other_income/documents/2/document/{local}]");
        credit_refusal(case, session, request, key, &path);
        credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, key, &path);
    } else {
        let request = save_documents(session, bad.clone());
        accepted(&send(session, request));
        let mut request = information_return_answers(session, &bad);
        let P::SaveHsa { followups, .. } = &mut request else { unreachable!() };
        let source = session
            .draft
            .income
            .iter()
            .rev()
            .find(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
            .unwrap()
            .id;
        let index = followups
            .iter()
            .position(|row| row.key == alloc::format!("income/{source}/payment"))
            .unwrap();
        let crate::HsaDocumentAnswer::MedicalPayment { value } = &mut followups[index].answer
        else {
            unreachable!()
        };
        *value = bad_payment;
        let suffix = local.strip_prefix("payment").unwrap();
        let path = suffix
            .split('/')
            .filter(|part| !part.is_empty())
            .fold(
                libertas::SchemaPath::root()
                    .field("followups")
                    .key(followups[index].key.as_str())
                    .field("answer")
                    .field("value"),
                |path, part| match part {
                    "0" => path.index(0),
                    "1" => path.index(1),
                    field => path.field(field),
                },
            )
            .to_string();
        let estimate = credit_estimate_documents_from_save(&request);
        credit_refusal(case, session, request, key, &path);
        credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, key, &path);
        let ids: Vec<_> = session
            .draft
            .income
            .iter()
            .filter_map(|entry| {
                matches!(entry.income, FederalIncome::MedicalDistribution { .. })
                    .then_some(entry.id)
            })
            .collect();
        for (row, id) in good.iter_mut().zip(ids) {
            row.id = id;
        }
    }
    let request = save_documents(session, good.clone());
    accepted(&send(session, request));
    save_information_return_answers(session, &good);
    if session.draft.adjustments.as_ref().is_some_and(|adjustments| {
        crate::hsa_correction::source_rule(&adjustments.hsa, &session.draft.income)
            == Some("HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED")
    }) {
        let mut request = information_return_answers(session, &good);
        let P::SaveHsa { value, hsa_return_documents, .. } = &mut request else { unreachable!() };
        assert_eq!(hsa_return_documents.len(), 1);
        let source = &hsa_return_documents[0];
        assert_eq!(source.gross, 100);
        assert_eq!(source.earnings, Some(100));
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0);
        let mut correction = hsa_correction(2026, 20260930, 0, 100).page();
        correction.document = Some(source.id);
        account.corrections.push(correction);
        value.accounts = vec![account];
        accepted(&send(session, request));
    }
    let saved = session.draft.clone();
    let request = open_hsa_page(session);
    accepted(&send(session, request));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let request = open_hsa_page(&mut cold);
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, saved);
}

#[test]
fn unit47_health_gap_medical_account_bounds_and_second_deposit_paths() {
    for branch in 0..10 {
        let mut good = if branch >= 4 { gap_rollover() } else { gap_sa() };
        match branch {
            0 => {
                good.payment = Some(MedicalSaPayment::ExcessReturned);
                good.form1099sa_mut().unwrap().earnings_on_excess_contributions = Some(100);
                good.form1099sa_mut().unwrap().distribution_code =
                    Some(crate::Form1099SaDistributionCode::ExcessContributions);
            }
            1 | 2 => {
                let mut statement = good.internal();
                statement.origin = HealthDocumentOrigin::TrusteeStatement;
                statement.box3_code = None;
                good.form = sa_source(statement).form;
                good.origin = HealthDocumentOrigin::TrusteeStatement;
                good.payment = Some(MedicalSaPayment::Unreported {
                    event: if branch == 1 {
                        MedicalDeemed::Terminated { occurred: 20260302, january_value: 100 }
                    } else {
                        MedicalDeemed::Pledged {
                            assets: vec![MedicalAccountWithdrawal { date: 20260302, amount: 100 }],
                        }
                    },
                    disabled_since: None,
                });
            }
            3 => {
                good.payment = Some(MedicalSaPayment::Inherited {
                    died: 20260101,

                    beneficiary: MedicalBeneficiary::Nonspouse { payments: vec![] },
                });
                good.form1099sa_mut().unwrap().fmv_on_date_of_death = Some(100);
                good.form1099sa_mut().unwrap().distribution_code =
                    Some(crate::Form1099SaDistributionCode::DeathDistributionOtherThanCode6);
            }
            9 => {
                let Some(MedicalSaPayment::Own { nonmedical, .. }) = &mut good.payment else {
                    unreachable!()
                };
                *nonmedical = (0..99)
                    .map(|_| MedicalAccountWithdrawal { date: 20260302, amount: 1 })
                    .collect();
                *good.gross_mut() += 99;
            }
            _ => {}
        }
        let mut bad = good.clone();
        let (case, key, local) = match branch {
            0 => {
                bad.payment = Some(MedicalSaPayment::ExcessReturned);
                bad.form1099sa_mut().unwrap().earnings_on_excess_contributions = Some(101);
                bad.form1099sa_mut().unwrap().distribution_code =
                    Some(crate::Form1099SaDistributionCode::ExcessContributions);
                ("health-sa-earnings", "SA1099_EARNINGS_ABOVE_GROSS", "box2_earnings")
            }
            1 | 2 => {
                *bad.gross_mut() += 1;
                (
                    if branch == 1 { "health-sa-terminated" } else { "health-sa-pledged" },
                    "SA1099_EVENT_VALUE_NOT_GROSS",
                    "payment/event",
                )
            }
            3 => {
                *bad.account_mut() = MedicalSaAccount::MedicareAdvantage;
                bad.medicare_from_spouse = Some(true);
                bad.medicare_prior = None;
                ("health-sa-inherited-spouse", "SA1099_SPOUSE_ACCOUNT_IS_OWN", "payment")
            }
            4 => {
                *bad.account_mut() = MedicalSaAccount::MedicareAdvantage;
                bad.medicare_prior = Some(MedicareMsaPrior::None);
                bad.medicare_from_spouse = Some(false);
                ("health-sa-rollover-account", "SA1099_ROLLOVER_NOT_ARCHER", "payment/rollover")
            }
            5..=8 => {
                let Some(MedicalSaPayment::Own { rollover: Some(rollover), .. }) = &mut bad.payment
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
                        part.character =
                            MedicalDepositCharacter::Cash { year: 2024, deadline: 20270415 };
                        (
                            "health-sa-deposit-year",
                            "MEDICAL_DEPOSIT_YEAR_INVALID",
                            "payment/rollover/receipts/0/deposits/1/deposited",
                        )
                    }
                    7 => {
                        part.character =
                            MedicalDepositCharacter::Cash { year: 2026, deadline: 20251231 };
                        (
                            "health-sa-deadline-year",
                            "MEDICAL_DEPOSIT_YEAR_INVALID",
                            "payment/rollover/receipts/0/deposits/1/character/deadline",
                        )
                    }
                    8 => {
                        part.character =
                            MedicalDepositCharacter::Cash { year: 2026, deadline: 20260301 };
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
                let Some(MedicalSaPayment::Own { nonmedical, .. }) = &mut bad.payment else {
                    unreachable!()
                };
                nonmedical.push(MedicalAccountWithdrawal { date: 20260302, amount: 1 });
                //101 actual parts are valid; refuse only an unreconciled gross.
                ("health-sa-wide-parts-gross", "SA1099_USES_NOT_GROSS", "payment")
            }
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let bad_payment = bad.payment.clone();
        gap_medical_case(
            case,
            &mut session,
            gap_sa_rows(bad),
            gap_sa_rows(good),
            bad_payment,
            (key, local),
            branch == 0,
        );
        let source = session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
            .count();
        assert_eq!(source, 3, "all three actual medical statements are retained");
    }
}

fn gap_funding_amounts() -> CareFundingAmounts {
    CareFundingAmounts { employee_premium: 100, employer_premium: 100 }
}
fn gap_ltc(funding: CareBenefitFunding) -> crate::LtcSource {
    crate::LtcSource {
interest: None,
        form: crate::Ltc1099Paper {
            payer: Default::default(),
            recipient: Default::default(),
            insured_party: Default::default(),
            date_certified: None,
            benefits: Some(10_000),
            accelerated: Some(0),
            paid: LtcPaidBasis::NotChecked,
            box4_qualified: false,
            illness: Some(LtcIllness::Chronic),
            calendar_year: 2026,
            corrected: false,
            payer_phone: String::new(),
            account_number: String::new(),
        },
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        contract: LtcContract::Nonqualified {
            treatment: NonqualifiedCareTreatment::AccidentHealth {
                funding,
                medical_reimbursement: 0,
                permanent_injury: None,
                recovery: None,
            },
        },
        insured: None,
    }
}
#[test]
fn unit47_health_gap_ltc_funding_histories_keep_exact_source_rows() {
    for branch in 0..5 {
        let good = match branch {
            0 => CareBenefitFunding::Individual { premiums: gap_funding_amounts() },
            1 => CareBenefitFunding::Group {
                years: vec![],
                first_year: Some(CareFirstYearFunding::Estimated {
                    premiums: gap_funding_amounts(),
                }),
            },
            _ => CareBenefitFunding::Group {
                years: vec![
                    CareFundingYear { year: 2024, premiums: gap_funding_amounts() },
                    CareFundingYear { year: 2025, premiums: gap_funding_amounts() },
                ],
                first_year: None,
            },
        };
        let mut bad = good.clone();
        let (key, local) = match &mut bad {
            CareBenefitFunding::Individual { premiums } => {
                *premiums = CareFundingAmounts { employee_premium: 0, employer_premium: 0 };
                ("LTC_FUNDING_ZERO", "premiums/employee_premium")
            }
            CareBenefitFunding::Group { years, first_year } => match branch {
                1 => {
                    *first_year = None;
                    ("LTC_FUNDING_HISTORY_INVALID", "years")
                }
                2 => {
                    *first_year =
                        Some(CareFirstYearFunding::Actual { premiums: gap_funding_amounts() });
                    ("LTC_FUNDING_HISTORY_INVALID", "first_year")
                }
                3 => {
                    years[1].year = years[0].year;
                    ("LTC_FUNDING_HISTORY_INVALID", "years/1/year")
                }
                4 => {
                    years[1].premiums =
                        CareFundingAmounts { employee_premium: 0, employer_premium: 0 };
                    ("LTC_FUNDING_ZERO", "years/1/premiums/employee_premium")
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let rows = |funding| {
            vec![
                document(
                    0,
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Sa1099(
                            crate::tests::support::client::income_pages::sa_document(gap_sa()),
                        ),
                    },
                ),
                document(
                    0,
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Ltc1099(ltc_document(gap_ltc(funding))),
                    },
                ),
            ]
        };
        crate::tests::support::client::income_pages::save_each_topic(&mut session, rows(good));
        let mut request = open_topic(&mut session, TaxSection::LongTermCareBenefits);
        let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
        let index = followups
            .iter()
            .position(|row| matches!(row.answer, crate::DocumentAnswer::LtcContract { .. }))
            .unwrap();
        let crate::DocumentAnswer::LtcContract { value } = &mut followups[index].answer else {
            unreachable!()
        };
        *value = (&gap_ltc(bad).contract).into();
        let path = local
            .split('/')
            .fold(
                libertas::SchemaPath::root()
                    .field("followups")
                    .key(followups[index].key.as_str())
                    .field("answer")
                    .field("value")
                    .field("treatment")
                    .field("funding"),
                |path, part| match part {
                    "1" => path.index(1),
                    field => path.field(field),
                },
            )
            .to_string();
        let case = alloc::format!("health-ltc-funding-{branch}");
        crate::tests::support::client::survey::survey_refusal(
            &case,
            &mut session,
            request.clone(),
            key,
            &path,
        );
        crate::tests::support::client::survey::survey_refusal(
            &alloc::format!("{case}-estimate"),
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request),
            key,
            &path,
        );
        // Half the actual insurance premiums were tax-free employer funding.
        assert_eq!(value_of(&supported(&session.draft), "taxable_ltc_benefits"), 50);
    }
}

fn gap_estimate(page: &P) -> P {
    match page {
        P::SaveDocuments { .. } => credit_estimate_documents_from_save(page),
        P::SaveCare { value, .. } => P::EstimateCare {
            allowed_filing: Vec::new(),
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
        P::SaveDocuments { .. } => open_documents(session),
        P::SaveCare { .. } => open_credits_topic(session, TaxSection::Care),
        P::SaveMarketplace { .. } => open_credits_topic(session, TaxSection::Marketplace),
        P::SaveSelfEmployedBenefits { .. } => {
            let request = go_to(&session.draft, PAGE_ADJUSTMENTS, TaxSection::SelfEmployedBenefits);
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
    credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, expected.0, expected.1);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        gap_estimate(&good),
    );
    assert!(
        matches!(reply, Reply::Response(P::RunningEstimate { .. } | P::Review { .. })),
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
    let request = go_to(&session.draft, PAGE_ADJUSTMENTS, TaxSection::SelfEmployedBenefits);
    let Reply::Edit(page @ P::SaveSelfEmployedBenefits { .. }) = send(session, request) else {
        panic!("actual benefits editor")
    };
    page
}

#[test]
fn unit47_health_gap_business_child_premium_uses_second_actual_month() {
    let mut session = at_the_adjustments_hub_with_business(4_000_000);
    let mut good = gap_benefits_editor(&mut session);
    let P::SaveSelfEmployedBenefits { value, .. } = &mut good else { unreachable!() };
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
    let P::SaveSelfEmployedBenefits { value, .. } = &mut bad else { unreachable!() };
    let SeHealth::Plan { months, .. } = &mut value.businesses[0].health else { unreachable!() };
    months[1].child_increase += 1;
    gap_page_case(
        "health-business-child-cost",
        &mut session,
        bad,
        good,
        ("HEALTH_CHILD_ABOVE_PREMIUMS", "[value/businesses/0/health/months/1/child_increase]"),
    );
    // The business gross is preserved independently of health admission.
    assert_eq!(stored_business(&session).1.receipts(), 4_000_000);
}

#[test]
fn unit47_health_gap_reimbursed_coverage_uses_saved_policy_and_activity() {
    let mut session = at_the_adjustments_hub_with_business(4_000_000);
    // Credits is the first unfinished section, so its topic saves return to it.
    session.draft.finished_sections.extend([TaxSection::Adjustments, TaxSection::Deductions]);
    let activity = stored_business(&session).0;
    let _ = open_credits_topic(&mut session, TaxSection::Marketplace);
    let saved = save_marketplace_and_reopen(
        &mut session,
        marketplace_fixture_page(vec![all_year("Actual premium source", 60_000, 55_000, 0)]),
    );
    let policy = saved.forms[0].id;
    assert!(policy > 0 && activity > 0);
    let mut good = gap_benefits_editor(&mut session);
    let P::SaveSelfEmployedBenefits { value, .. } = &mut good else { unreachable!() };
    value.policies = (1..=2)
        .map(|month| HealthPolicyAllocationPage {
            policy,
            month,
            activity,
            parts: 1,
            reimbursed: Some(ReimbursedPolicyPremiums { premiums: 0, coverage: 0, child: None }),
        })
        .collect();
    let mut bad = good.clone();
    let P::SaveSelfEmployedBenefits { value, .. } = &mut bad else { unreachable!() };
    value.policies[1].reimbursed.as_mut().unwrap().coverage = 1;
    gap_page_case(
        "health-policy-reimbursed",
        &mut session,
        bad,
        good,
        ("HEALTH_REIMBURSED_COVERAGE_ABOVE_TOTAL", "[value/policies/1/reimbursed/coverage]"),
    );
    assert_eq!(stored_marketplace(&session).forms[0].id, policy);
    assert_eq!(stored_business(&session).0, activity);
}

#[test]
fn unit47_health_gap_care_provider_required_preserves_actual_credit() {
    let mut session = m2_household();
    let mut good = open_credits_topic(&mut session, TaxSection::Care);
    fill_care_provider(&mut good);
    let mut bad = good.clone();
    let P::SaveCare { value, .. } = &mut bad else { unreachable!() };
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
    let P::SaveCare { value, .. } = &mut good else { unreachable!() };
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
    let P::SaveCare { value, .. } = &mut bad else { unreachable!() };
    value.providers[0].relationship = ProviderRelationship::ParentOfChild;
    gap_page_case(
        "health-care-parent-provider",
        &mut session,
        bad,
        good,
        ("CARE_PARENT_PROVIDER_NEEDS_A_CHILD", "[value/providers/0/relationship]"),
    );
    assert_eq!(value_of(&supported(&session.draft), "care_credit"), 0);
}
#[test]
fn unit47_health_gap_student_requires_five_actual_months() {
    let mut draft = complete(FederalStatus::Joint);
    draft.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut session = Interview::new(TEST_USER, draft);
    let mut good = open_credits_topic(&mut session, TaxSection::Care);
    let P::SaveCare { value, .. } = &mut good else { unreachable!() };
    value.filers = vec![CareFilerInput {
        owner: Owner::Spouse,
        situation: CareFilerSituationInput::Student {
            months: vec![1, 2, 3, 4, 5],
            earned_in_months: None,
        },
    }];
    let mut bad = good.clone();
    let P::SaveCare { value, .. } = &mut bad else { unreachable!() };
    let CareFilerSituationInput::Student { months, .. } = &mut value.filers[0].situation else {
        unreachable!()
    };
    months.pop();
    gap_page_case(
        "health-care-student-months",
        &mut session,
        bad,
        good,
        ("CARE_STUDENT_NEEDS_FIVE_MONTHS", "[value/filers/0/situation/months]"),
    );
}
#[test]
fn unit47_health_gap_care_parts_use_accepted_second_wage() {
    for branch in 0..4 {
        let mut session = m2_household();
        let mut rows: Vec<_> = page_documents(&open_documents(&mut session))
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::Wage { .. }))
            .collect();
        let mut wage = m2_w2();
        let IncomeSource::Wage { value: crate::WageDocument::W2(source) } = &mut wage.document
        else {
            unreachable!()
        };
        source.wages = Some(100_000);
        source.social_wages = Some(100_000);
        source.social_tax = Some(6_200);
        source.medicare_wages = Some(100_000);
        source.medicare_tax = Some(1_450);
        source.withholding = Some(0);
        source.care_benefits = Some(10_000);
        rows.push(wage);
        let request = save_documents(&mut session, rows);
        accepted(&send(&mut session, request));
        let mut good = open_credits_topic(&mut session, TaxSection::Care);
        fill_care_provider(&mut good);
        let P::SaveCare { value, .. } = &mut good else { unreachable!() };
        value.plan_maximum = CarePlanMaximum::Plans { maximum: 10_000 };
        assert_eq!(value.benefits.len(), 1);
        let wage_id = value.benefits[0].wage;
        value.benefits[0].in_box_1 = Some(4_000);
        value.benefits[0].forfeited = Some(6_000);
        let mut bad = good.clone();
        let P::SaveCare { value, .. } = &mut bad else { unreachable!() };
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
        assert!(session.draft.income.iter().any(|entry| entry.id == wage_id && matches!(&entry.income, FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source { form: data, .. }) } if data.care_benefits == Some(10_000))));
    }
}

fn gap_one_month() -> MarketplaceDocumentView {
    let mut form = all_year("Actual January policy", 100_000, 90_000, 30_000);
    form.value.set_amounts(PolicyAmounts::ByMonth {
        months: vec![PolicyMonth {
            month: 1,
            premium: 100_000,
            benchmark: Some(90_000),
            advance: 30_000,
        }],
    });
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
                        share: MarketplaceFraction { numerator: 1, denominator: 1 },
                    },
                }],
            },
        }];
        save_marketplace_and_reopen(&mut session, value);
        let good = open_credits_topic(&mut session, TaxSection::Marketplace);
        let mut bad = good.clone();
        let P::SaveMarketplace { value, .. } = &mut bad else { unreachable!() };
        let value = &mut value.forms[0];
        let (case, key, path) = if branch == 0 {
            value.allocation[0].from = 2;
            (
                "health-ptc-allocation-months",
                "PTC_MONTHS_REVERSED",
                "[value/forms/0/allocation/0/to]",
            )
        } else {
            let MarketplaceAllocationKindPage::OtherShared { families } =
                &mut value.allocation[0].kind
            else {
                unreachable!()
            };
            families[0].agreement = MarketplaceAgreement::Agreed {
                share: MarketplaceFraction { numerator: 2, denominator: 1 },
            };
            (
                "health-ptc-fraction",
                "PTC_SHARE_EXCEEDS_WHOLE",
                "[value/forms/0/allocation/0/kind/families/0/agreement/share/numerator]",
            )
        };
        gap_page_case(case, &mut session, bad, good, (key, path));
        assert_eq!(
            stored_marketplace(&session).forms[0].form.amounts(),
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
    value.forms[0].coverage[0].people.extend([employee, sibling]);
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
        let forms = value
            .forms
            .iter()
            .map(|form| crate::Form1095ADocument { id: form.id, value: form.value.clone() })
            .collect();
        let source = save_marketplace_sources(&mut session, forms);
        accepted(&send(&mut session, source));
        let good = save_marketplace(&mut session, value);
        let mut bad = good.clone();
        let P::SaveMarketplace { value, .. } = &mut bad else { unreachable!() };
        let (key, path) = match branch {
            0 => {
                value.dependents.clear();
                ("PTC_QSEHRA_EMPLOYEE_RETURN_REQUIRED", "[value/employee_qsehra_facts/0/income]")
            }
            1 => {
                value.qsehra[0].determinations[0].from = 2;
                ("PTC_MONTHS_REVERSED", "[value/qsehra/0/determinations/0/to]")
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
                ("PTC_QSEHRA_SOCIAL_ABOVE_GROSS", "[value/dependents/0/taxable_social]")
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
            assert_eq!(value_of(&supported(&session.draft), "premium_tax_credit"), 745);
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
                    recipient: CareBenefitRecipient::Insured { other_payees: false },
                    injured_payees: None,
                })
                .collect(),
        },
    }
}
fn gap_qualified_ltc() -> IncomeSource {
    IncomeSource::OtherIncome {
        value: crate::OtherIncomeDocument::Ltc1099(ltc_document(crate::LtcSource {
interest: None,
            form: crate::Ltc1099Paper {
                payer: Default::default(),
                recipient: Default::default(),
                insured_party: Default::default(),
                date_certified: None,
                benefits: Some(1_500_000),
                accelerated: Some(0),
                paid: LtcPaidBasis::PerDiem,
                box4_qualified: true,
                illness: Some(LtcIllness::Chronic),
                calendar_year: 2026,
                corrected: false,
                payer_phone: String::new(),
                account_number: String::new(),
            },
            benefit_origin: Some(LtcPaymentOrigin::Insurance),
            accelerated_origin: None,
            contract: LtcContract::Qualified {
                qualification: CareContractQualification::Qualified,
            },
            insured: None,
        })),
    }
}
#[test]
fn unit47_health_gap_injured_ltc_payees_use_actual_accepted_sources_and_second_period() {
    for duplicate in [false, true] {
        let mut draft = complete(FederalStatus::Joint);
        draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
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
        crate::tests::support::client::admit_native_fixture_facts(&mut session);
        let taxpayer = document(0, gap_qualified_ltc());
        let mut spouse = document(0, gap_qualified_ltc());
        spouse.owner = crate::DocumentOwner::Spouses;
        accepted(&crate::tests::support::client::income_pages::save_source_fixtures(
            &mut session,
            vec![taxpayer, spouse],
        ));
        let request = save_topic(
            &mut session,
            TaxSection::LongTermCareBenefits,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(gap_periods()) },
            )],
        );
        accepted(&send(&mut session, request));
        assert_eq!(value_of(&supported(&session.draft), "taxable_ltc_benefits"), 3770);
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
        let page = open_topic(&mut session, TaxSection::LongTermCareBenefits);
        let mut good: Vec<_> = page_documents(&page)
            .into_iter()
            .filter(|row| {
                matches!(
                    row.document,
                    IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
                )
            })
            .collect();
        let period_index = good
            .iter()
            .position(|row| {
                matches!(
                    row.document,
                    IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
                )
            })
            .unwrap();
        let mut row = good.remove(period_index);
        let IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(value) } =
            &mut row.document
        else {
            unreachable!()
        };
        let LtcPeriodsInput::Several { periods, .. } = &mut value.periods else { unreachable!() };
        for (period, source) in periods.iter_mut().zip(&sources) {
            period.injured_payees = Some(InjuredLtcPeriodPayeesInput::Payments {
                payments: vec![InjuredLtcPeriodPaymentInput { source: *source, amount: 1_500_000 }],
            });
        }
        good.push(row);
        assert!(matches!(
            good[0].document,
            IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
        ));
        let mut bad = good.clone();
        let IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(value) } =
            &mut bad[0].document
        else {
            unreachable!()
        };
        let LtcPeriodsInput::Several { periods, .. } = &mut value.periods else { unreachable!() };
        let Some(InjuredLtcPeriodPayeesInput::Payments { payments }) =
            &mut periods[1].injured_payees
        else {
            unreachable!()
        };
        let (case, key, path) = if duplicate {
            payments[0].amount = 750_000;
            payments.push(InjuredLtcPeriodPaymentInput { source: sources[1], amount: 750_000 });
            (
                "health-ltc-payee-duplicate",
                "INJURED_LTC_SOURCE_TWICE",
                "[documents/documents/0/document/periods/periods/1/injured_payees/payments/1/source]",
            )
        } else {
            payments[0].amount += 1;
            (
                "health-ltc-payee-total",
                "INJURED_LTC_PAYEE_TOTAL",
                "[documents/documents/0/document/periods/periods/1/injured_payees/payments]",
            )
        };
        gap_document_case(case, &mut session, bad, good, (key, path));
        // Identifying actual payees must not change the canonical Section C
        // exclusion. It changes ownership attribution for Form 8379 only.
        assert_eq!(value_of(&supported(&session.draft), "taxable_ltc_benefits"), 3770);
        assert!(sources.iter().all(|id| session.draft.income.iter().any(|entry| entry.id == *id)));
    }
}

// Schema widening reaches preflight before the native source-amount refusal.
#[test]
fn unit47_health_cap_sweep_rollover_year_preflight_preserves_refusal() {
    let good = gap_rollover();
    let mut bad = good.clone();
    let Some(MedicalSaPayment::Own { rollover: Some(rollover), .. }) = &mut bad.payment else {
        unreachable!()
    };
    rollover.receipts[0].deposits[1].character =
        MedicalDepositCharacter::Cash { year: u32::MAX, deadline: 20270415 };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let bad_payment = bad.payment.clone();
    gap_medical_case(
        "health-wide-rollover-year",
        &mut session,
        vec![document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Sa1099(
                    crate::tests::support::client::income_pages::sa_document(bad),
                ),
            },
        )],
        vec![document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Sa1099(
                    crate::tests::support::client::income_pages::sa_document(good),
                ),
            },
        )],
        bad_payment,
        ("MEDICAL_DEPOSIT_YEAR_INVALID", "payment/rollover/receipts/0/deposits/1/deposited"),
        false,
    );
}
