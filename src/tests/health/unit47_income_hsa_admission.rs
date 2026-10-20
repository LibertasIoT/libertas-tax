// Actual request controls for the remaining income/HSA admission predicates.
// Sources and accepted identities are genuine fixtures; these tests do not
// certify a client runtime or exhaust every compound native subcondition.
use super::unit47_credit_source_validation::{
    credit_estimate_documents_from_save, credit_preview, credit_refusal,
};
use super::*;

fn income_admission_case(
    case: &str,
    session: &mut Interview,
    topic: TaxSection,
    bad: Vec<IncomeDocumentItem>,
    corrected: Vec<IncomeDocumentItem>,
    expected: (&str, &str),
) {
    let request = save_topic(&session.draft, topic, bad);
    let estimate = credit_estimate_documents_from_save(&request);
    credit_refusal(case, session, request, expected.0, expected.1);
    credit_refusal(
        &alloc::format!("{case}-estimate"),
        session,
        estimate,
        expected.0,
        expected.1,
    );
    let request = save_topic(&session.draft, topic, corrected);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = send(session, credit_estimate_documents_from_save(&request));
    assert!(
        matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. })),
        "{case}: {reply:?}"
    );
    assert_eq!(
        session.draft, before,
        "{case}: corrected estimate mutated draft"
    );
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, request));
    let saved = session.draft.clone();
    let page = open_topic(session, topic);
    accepted(&send(session, page));
    assert_eq!(session.draft, saved, "{case}: reopen/resave");
    let mut cold = restart(saved.clone());
    let page = open_topic(&mut cold, topic);
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved, "{case}: restart/reopen/resave");
}

fn admission_duplicate(
    case: &str,
    draft: FederalDraft,
    topic: TaxSection,
    answer: IncomeDocument,
    intervening: IncomeDocument,
    expected: (&str, &str),
) {
    let mut session = Interview::new(TEST_USER, draft);
    income_admission_case(
        case,
        &mut session,
        topic,
        vec![
            document(0, answer.clone()),
            document(0, intervening.clone()),
            document(0, answer.clone()),
        ],
        vec![document(0, answer), document(0, intervening)],
        expected,
    );
}

fn admission_qbi() -> IncomeDocument {
    IncomeDocument::QbiLossCarryforward(QbiLossCarryforward {
        business: 0,
        reit_ptp: 0,
        injured_owners: None,
    })
}
fn admission_foreign_account() -> IncomeDocument {
    IncomeDocument::ForeignAccountOrTrust(ForeignAccountOrTrust {
        account: false,
        trust: false,
        highest_value: None,
        fbar_required: None,
        fbar_countries: vec![],
    })
}

#[test]
fn unit47_income_admission_whole_return_duplicates_keep_original_submitted_rows() {
    let whole = "[documents/documents/2/document]";
    for branch in 0..14 {
        let (case, topic, answer, intervening, key, path) = match branch {
            0 => (
                "income-gambling-duplicate",
                TaxSection::Gambling,
                gambling_losses(0, 0),
                w2g(100_000, 0),
                "GAMBLING_LOSSES_TWICE",
                whole,
            ),
            1 => (
                "income-lookback-duplicate",
                TaxSection::SelfEmployment,
                IncomeDocument::BusinessPropertyLosses(BusinessPropertyLosses {
                    regular: vec![],
                    amt: vec![],
                }),
                admission_qbi(),
                "BUSINESS_PROPERTY_LOSSES_TWICE",
                whole,
            ),
            2 => (
                "income-qbi-duplicate",
                TaxSection::SelfEmployment,
                admission_qbi(),
                IncomeDocument::Business(business(100_000)),
                "QBI_LOSS_CARRYFORWARD_TWICE",
                whole,
            ),
            3 => (
                "income-foreign-account-duplicate",
                TaxSection::InterestDividends,
                admission_foreign_account(),
                IncomeDocument::TaxableBondPremium(false),
                "FOREIGN_ACCOUNT_TWICE",
                whole,
            ),
            4 => (
                "income-premium-duplicate",
                TaxSection::InterestDividends,
                IncomeDocument::TaxableBondPremium(false),
                admission_foreign_account(),
                "TAXABLE_PREMIUM_ELECTION_TWICE",
                whole,
            ),
            5 => (
                "income-8606-duplicate",
                TaxSection::Retirement,
                IncomeDocument::Form8606(Form8606 {
                    adjustment: 0,
                    reservist_repayments: vec![],
                    basis: 100_000,
                    year_end: 200_000,
                }),
                IncomeDocument::RothHistory(RothIraHistory {
                    first_year: 2010,
                    amounts: Some(RothOrdering {
                        regular: 80_000,
                        conversions: vec![],
                    }),
                }),
                "FORM_8606_TWICE",
                "[documents/documents/2/owner]",
            ),
            6 => (
                "income-roth-duplicate",
                TaxSection::Retirement,
                IncomeDocument::RothHistory(RothIraHistory {
                    first_year: 2010,
                    amounts: Some(RothOrdering {
                        regular: 80_000,
                        conversions: vec![],
                    }),
                }),
                IncomeDocument::Form8606(Form8606 {
                    adjustment: 0,
                    reservist_repayments: vec![],
                    basis: 100_000,
                    year_end: 200_000,
                }),
                "ROTH_HISTORY_TWICE",
                "[documents/documents/2/owner]",
            ),
            7 => (
                "income-clergy-duplicate",
                TaxSection::Wages,
                IncomeDocument::Clergy(ClergyRecord {
                    role: ClergyRole::ReligiousSect,
                }),
                IncomeDocument::W2(w2(100_000)),
                "CLERGY_TWICE",
                "[documents/documents/2/owner]",
            ),
            8 => (
                "income-nonfarm-duplicate",
                TaxSection::SelfEmployment,
                IncomeDocument::NonfarmOptionalMethod(se_history()),
                IncomeDocument::Business(business(100_000)),
                "NONFARM_OPTIONAL_TWICE",
                whole,
            ),
            9 => (
                "income-able-duplicate",
                TaxSection::OtherIncome,
                IncomeDocument::AbleContributions(AbleContributionsPage {
                    own: 0,
                    other: 0,
                    work: AbleWork::No,
                    year_end: 0,
                    additional_open: false,
                }),
                IncomeDocument::EducationAccountExpenses(EducationAccountExpenses {
                    students: vec![EducationAccountStudent {
                        person: -1,
                        college: CollegeCosts::Entered {
                            tuition_books: 0,
                            assistance: 0,
                            credit_elsewhere: 0,
                        },
                        other_college: 0,
                        k12: 0,
                        k12_coverdell: 0,
                        apprenticeship: 0,
                        loan_payments: 0,
                        loan_payments_before: 0,
                        academy_costs: 0,
                        used_elsewhere: 0,
                    }],
                }),
                "ABLE_CONTRIBUTIONS_TWICE",
                "[documents/documents/2/owner]",
            ),
            10 => (
                "income-social-election-duplicate",
                TaxSection::Retirement,
                IncomeDocument::SocialLumpSums(LumpSumElection {
                    years: vec![social_prior_year()],
                    complete: true,
                }),
                IncomeDocument::Ssa1099(Ssa1099 {
                    benefits: 200_000,
                    withholding: 0,
                    statement: None,
                }),
                "LUMP_SUM_ELECTION_TWICE",
                whole,
            ),
            11 => (
                "income-1256-duplicate",
                TaxSection::InvestmentSales,
                IncomeDocument::Section1256Carryback(Section1256Carryback {
                    amount: 100,
                    injured_owners: None,
                }),
                IncomeDocument::Contracts1099B(contracts_page("", -1_000)),
                "SECTION_1256_CARRYBACK_TWICE",
                whole,
            ),
            12 => (
                "income-farm-loss-duplicate",
                TaxSection::Farming,
                IncomeDocument::FarmLoss(FarmLossWaiver {
                    choice: FarmLossChoice::Waive,
                }),
                IncomeDocument::Farm(cash_farm(5_000_000, 0)),
                "FARM_LOSS_TWICE",
                whole,
            ),
            13 => (
                "income-farm-averaging-duplicate",
                TaxSection::Farming,
                IncomeDocument::FarmAveraging(averaging_election()),
                IncomeDocument::Farm(cash_farm(5_000_000, 0)),
                "FARM_AVERAGING_TWICE",
                whole,
            ),
            _ => unreachable!(),
        };
        admission_duplicate(
            case,
            complete(FederalStatus::Single),
            topic,
            answer,
            intervening,
            (key, path),
        );
    }
}

#[test]
fn unit47_income_admission_submitted_business_return_boundary() {
    let mut good = business(100_000);
    good.returns = 100_000;
    let mut bad = good.clone();
    bad.returns += 1;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    income_admission_case(
        "income-business-returns",
        &mut session,
        TaxSection::SelfEmployment,
        vec![
            document(0, IncomeDocument::Business(business(10_000))),
            document(0, admission_qbi()),
            document(0, IncomeDocument::Business(bad)),
        ],
        vec![
            document(0, IncomeDocument::Business(business(10_000))),
            document(0, admission_qbi()),
            document(0, IncomeDocument::Business(good)),
        ],
        (
            "BUSINESS_RETURNS_ABOVE_RECEIPTS",
            "[documents/documents/2/document/returns]",
        ),
    );
}

#[test]
fn unit47_income_admission_actual_form8379_qbi_owner_totals() {
    for business_part in [true, false] {
        let mut draft = complete(FederalStatus::Joint);
        draft.injured_spouse = Some(InjuredSpouseAllocation::defaults());
        let good = QbiLossCarryforward {
            business: 100,
            reit_ptp: 200,
            injured_owners: Some(InjuredQbiLossOwners::Shares {
                taxpayer_business: 60,
                spouse_business: 40,
                taxpayer_reit_ptp: 120,
                spouse_reit_ptp: 80,
            }),
        };
        let mut bad = good.clone();
        let Some(InjuredQbiLossOwners::Shares {
            taxpayer_business,
            taxpayer_reit_ptp,
            ..
        }) = &mut bad.injured_owners
        else {
            unreachable!()
        };
        if business_part {
            *taxpayer_business += 1;
        } else {
            *taxpayer_reit_ptp += 1;
        }
        let mut session = Interview::new(TEST_USER, draft);
        income_admission_case(
            &alloc::format!("income-qbi-owners-{business_part}"),
            &mut session,
            TaxSection::SelfEmployment,
            vec![
                document(0, IncomeDocument::Business(business(100_000))),
                document(
                    0,
                    IncomeDocument::BusinessPropertyLosses(BusinessPropertyLosses {
                        regular: vec![],
                        amt: vec![],
                    }),
                ),
                document(0, IncomeDocument::QbiLossCarryforward(bad)),
            ],
            vec![
                document(0, IncomeDocument::Business(business(100_000))),
                document(
                    0,
                    IncomeDocument::BusinessPropertyLosses(BusinessPropertyLosses {
                        regular: vec![],
                        amt: vec![],
                    }),
                ),
                document(0, IncomeDocument::QbiLossCarryforward(good)),
            ],
            (
                "INJURED_QBI_OWNER_TOTAL",
                "[documents/documents/2/document/injured_owners]",
            ),
        );
    }
}

#[test]
fn unit47_income_admission_child_education_and_foreign_duplicates() {
    let mut draft = complete(FederalStatus::Single);
    add_child(&mut draft);
    let child = draft.dependents.last().unwrap().id;
    admission_duplicate(
        "income-child-election-duplicate",
        draft,
        TaxSection::InterestDividends,
        IncomeDocument::Form8814(election_page(
            child,
            vec![ChildStatement::Int1099(election_interest(10_000))],
        )),
        admission_foreign_account(),
        (
            "CHILD_ELECTION_TWICE",
            "[documents/documents/2/document/child]",
        ),
    );
    admission_duplicate(
        "income-education-expenses-duplicate",
        complete(FederalStatus::Single),
        TaxSection::OtherIncome,
        IncomeDocument::EducationAccountExpenses(EducationAccountExpenses {
            students: vec![EducationAccountStudent {
                person: -1,
                college: CollegeCosts::Entered {
                    tuition_books: 10_000,
                    assistance: 0,
                    credit_elsewhere: 0,
                },
                other_college: 0,
                k12: 0,
                k12_coverdell: 0,
                apprenticeship: 0,
                loan_payments: 0,
                loan_payments_before: 0,
                academy_costs: 0,
                used_elsewhere: 0,
            }],
        }),
        IncomeDocument::AbleContributions(AbleContributionsPage {
            own: 0,
            other: 0,
            work: AbleWork::No,
            year_end: 0,
            additional_open: false,
        }),
        (
            "EDUCATION_ACCOUNT_EXPENSES_TWICE",
            "[documents/documents/2/document]",
        ),
    );
    admission_duplicate(
        "income-foreign-earned-duplicate",
        complete(FederalStatus::Single),
        TaxSection::WorkAbroad,
        IncomeDocument::Form2555(Form2555 {
            qualification: ForeignQualification::Presence {
                starts_on: 20260101,
                foreign_days: 340,
            },
            tax_home: true,
            revocation: ExclusionRevocation::None,
            exclusion: true,
            prior_year_pay: 0,
            prior_year_history: None,
            multi_year_pay: vec![],
            housing: ForeignHousing::None,
        }),
        IncomeDocument::ForeignPay(ForeignPay {
            employer: "Foreign employer".into(),
            wages: 100_000,
        }),
        ("FORM_2555_TWICE", "[documents/documents/2/document]"),
    );
}

fn admission_bond_session() -> (Interview, IncomeDocumentItem, SavingsBonds) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let interest = Int1099 {
        more: IntMore {
            us_obligations: 300_000,
            ..Default::default()
        },
        ..native_int1099("Treasury".into(), 0)
    };
    let request = save_topic(
        &session.draft,
        TaxSection::InterestDividends,
        vec![document(0, IncomeDocument::Int1099(interest))],
    );
    accepted(&send(&mut session, request));
    let source = page_documents(&open_topic(&mut session, TaxSection::InterestDividends))
        .iter()
        .find(|item| matches!(item.document, IncomeDocument::Int1099(_)))
        .unwrap()
        .clone();
    assert!(source.id > 0);
    let bonds = SavingsBonds {
        proceeds: 1_200_000,
        redemption_interest: 300_000,
        earliest_issue_date: 20100601,
        interest_statement: source.id,
        students: vec![SavingsBondStudent {
            person: -1,
            expenses: SavingsBondExpenses::Entered {
                tuition: 400_000,
                benefits: 0,
            },
            contributions: 0,
        }],
    };
    (session, source, bonds)
}

#[test]
fn unit47_income_admission_bonds_actual_source_identity_owner_and_gross() {
    for branch in 0..4 {
        let (mut session, source, bonds) = admission_bond_session();
        let mut bad = document(0, IncomeDocument::SavingsBonds(bonds.clone()));
        let expected = if branch == 0 {
            "SAVINGS_BONDS_TWICE"
        } else {
            "SAVINGS_BONDS_SOURCE_INVALID"
        };
        if let IncomeDocument::SavingsBonds(value) = &mut bad.document {
            match branch {
                1 => {
                    value.interest_statement = session
                        .draft
                        .income
                        .iter()
                        .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
                        .unwrap()
                        .id
                }
                2 => bad.owner = Owner::Spouse,
                3 => value.redemption_interest += 1,
                _ => {}
            }
        }
        let earlier = if branch == 0 {
            document(0, IncomeDocument::SavingsBonds(bonds.clone()))
        } else {
            document(0, IncomeDocument::TaxableBondPremium(false))
        };
        let bad_items = vec![source.clone(), earlier, bad];
        let corrected = vec![
            source,
            document(0, IncomeDocument::TaxableBondPremium(false)),
            document(0, IncomeDocument::SavingsBonds(bonds)),
        ];
        let path = if branch == 0 {
            "[documents/documents/2/document]"
        } else {
            "[documents/documents/2/document/interest_statement]"
        };
        income_admission_case(
            &alloc::format!("income-bonds-source-{branch}"),
            &mut session,
            TaxSection::InterestDividends,
            bad_items,
            corrected,
            (expected, path),
        );
    }
}

#[test]
fn unit47_income_admission_texas_withholding_management() {
    let source = Int1099 {
        more: IntMore {
            withholding: 100,
            ..Default::default()
        },
        ..native_int1099("Community interest".into(), 10_000)
    };
    let mut bad = document(0, IncomeDocument::Int1099(source));
    // The actual joint Form8379 community overlay requires an income
    // classification; supporting answers still keep their fixed NotNeeded.
    bad.community = CommunityShare::Community;
    // 2026-10-16, Person attribution / Source facts: the original native
    // refund_funding::management_needed treats a known zero refund as known-no.
    // The old $6,500 withholding fixture owes tax; fixed $10,000 withholding
    // on the same $100,000 W-2 supplies a positive refund, not solved shares.
    // Estimate derives applicability without requiring an answer; only Save
    // requires the actual applicable management answer.
    for (withholding, needed) in [(650_000, false), (1_000_000, true)] {
        let draft = texas_interest_admission(withholding);
        let settled = estimate(&draft);
        assert!(settled.financial_source_scope_complete);
        if needed {
            assert!(settled.refund.is_some_and(|refund| refund > 0));
            assert_eq!(settled.balance, Some(0));
        } else {
            assert_eq!(settled.refund, Some(0));
            assert!(settled.balance.is_some_and(|balance| balance > 0));
        }
        let mut session = Interview::new(TEST_USER, draft);
        let missing = save_topic(
            &session.draft,
            TaxSection::InterestDividends,
            vec![
                document(0, admission_foreign_account()),
                document(0, IncomeDocument::TaxableBondPremium(false)),
                bad.clone(),
            ],
        );
        let before = session.draft.clone();
        let dirty = session.dirty;
        let Reply::Response(P::IncomeRunningEstimate {
            refund_management, ..
        }) = send(&mut session, credit_estimate_documents_from_save(&missing))
        else {
            panic!("management applicability must be a pure running estimate")
        };
        assert_eq!(refund_management.len(), 3);
        assert_eq!(
            refund_management
                .iter()
                .map(|row| row.refund_management_needed)
                .collect::<Vec<_>>(),
            vec![Some(false), Some(false), Some(needed)]
        );
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        if needed {
            credit_refusal(
                "income-texas-withholding-management",
                &mut session,
                missing,
                "INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED",
                "[documents/documents/2/refund_management]",
            );
        }
        let mut corrected = bad.clone();
        corrected.refund_management = needed.then_some(RefundManagement::Taxpayer);
        let request = save_topic(
            &session.draft,
            TaxSection::InterestDividends,
            vec![
                document(0, admission_foreign_account()),
                document(0, IncomeDocument::TaxableBondPremium(false)),
                corrected.clone(),
            ],
        );
        let reply = send(&mut session, credit_estimate_documents_from_save(&request));
        assert!(
            matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. })),
            "{reply:?}"
        );
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        let page = open_topic(&mut session, TaxSection::InterestDividends);
        let reopened = page_documents(&page)
            .iter()
            .find(|row| matches!(row.document, IncomeDocument::Int1099(_)))
            .unwrap();
        assert_eq!(reopened.document, corrected.document);
        assert_eq!(reopened.refund_management, corrected.refund_management);
        assert_eq!(reopened.refund_management_needed, Some(needed));
        accepted(&send(&mut session, page));
        assert_eq!(session.draft, saved, "reopen/resave");
        let mut cold = restart(saved.clone());
        let page = open_topic(&mut cold, TaxSection::InterestDividends);
        accepted(&send(&mut cold, page));
        assert_eq!(cold.draft, saved, "restart/reopen/resave");
    }
}

#[test]
fn unit47_income_admission_actual_farm_election_conflict() {
    let mut elected = cash_farm(100_000, 0);
    elected.more.conservation = Conservation::Deduct {
        method: FarmConservationMethod::OriginalElection { year: 2025 },
        costs: vec![],
        assessments: vec![],
        carry: vec![],
    };
    let mut bad = cash_farm(200_000, 0);
    bad.more.conservation = Conservation::Capitalize {
        method: FarmConservationCapitalMethod::NeverElected,
        costs: vec![],
    };
    let mut corrected = bad.clone();
    corrected.more.conservation = Conservation::None;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    income_admission_case(
        "income-farm-method-conflict",
        &mut session,
        TaxSection::Farming,
        vec![
            document(0, IncomeDocument::Farm(elected.clone())),
            document(0, admission_qbi()),
            document(0, IncomeDocument::Farm(bad)),
        ],
        vec![
            document(0, IncomeDocument::Farm(elected)),
            document(0, admission_qbi()),
            document(0, IncomeDocument::Farm(corrected)),
        ],
        (
            "FARM_CONSERVATION_METHOD_CONFLICT",
            "[documents/documents/2/document/more/conservation/method]",
        ),
    );
}

#[test]
fn unit47_income_admission_patr_combined_amounts_save_estimate_preview() {
    let good = Patr1099 {
        patronage: 100_000,
        nonpatronage: 0,
        per_unit_retains: 0,
        withholding: 0,
        redeemed: 0,
        more: PatrMore {
            qualified_items: 60_000,
            sstb_items: 40_000,
            ..Default::default()
        },
    };
    // No explicit allocation rows: changing the source total must reach its
    // canonical amount bound before source allocations are generated.
    let mut bad = good.clone();
    bad.more.sstb_items += 1;
    let key = "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26";
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    credit_refusal(
        "income-patr-combined-preview",
        &mut session,
        credit_preview(IncomeDocument::Patr1099(bad.clone())),
        key,
        "[document/more/qualified_items]",
    );
    income_admission_case(
        "income-patr-combined",
        &mut session,
        TaxSection::Farming,
        vec![
            document(0, IncomeDocument::Farm(cash_farm(100_000, 0))),
            document(0, admission_qbi()),
            document(0, IncomeDocument::Patr1099(bad)),
        ],
        vec![
            document(0, IncomeDocument::Farm(cash_farm(100_000, 0))),
            document(0, admission_qbi()),
            document(0, IncomeDocument::Patr1099(good)),
        ],
        (key, "[documents/documents/2/document/more/qualified_items]"),
    );
}

fn admission_hsa_estimate(request: &P) -> P {
    let P::SaveHsa {
        allowed_owners,
        other_coverage_allowed,
        allowed_family_split,
        hsa_failed_deposits,
        archer_failed_deposits,
        hsa_return_documents,
        archer_return_documents,
        ira_sources,
        archer_income,
        value,
        ..
    } = request
    else {
        unreachable!()
    };
    P::EstimateHsa {
        user: TEST_USER,
        allowed_owners: allowed_owners.clone(),
        other_coverage_allowed: *other_coverage_allowed,
        allowed_family_split: allowed_family_split.clone(),
        hsa_failed_deposits: hsa_failed_deposits.clone(),
        archer_failed_deposits: archer_failed_deposits.clone(),
        hsa_return_documents: hsa_return_documents.clone(),
        archer_return_documents: archer_return_documents.clone(),
        ira_sources: ira_sources.clone(),
        archer_income: archer_income.clone(),
        value: value.clone(),
    }
}
fn admission_hsa_case(
    case: &str,
    session: &mut Interview,
    bad: P,
    good: P,
    expected: (&str, &str),
) {
    let estimate = admission_hsa_estimate(&bad);
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
    let reply = send(session, admission_hsa_estimate(&good));
    assert!(
        matches!(reply, Reply::Response(P::RunningEstimate { .. })),
        "{case}: {reply:?}"
    );
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, good));
    let saved = session.draft.clone();
    let page = open_hsa_page(session);
    accepted(&send(session, page));
    assert_eq!(session.draft, saved, "{case}: HSA reopen");
    let mut cold = restart(saved.clone());
    let page = open_hsa_page(&mut cold);
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved, "{case}: HSA restart");
}

#[test]
fn unit47_hsa_admission_account_context_method_and_family_bounds() {
    for branch in 0..7 {
        let joint = branch == 0 || branch == 1 || branch == 5 || branch == 6;
        let mut session = at_the_adjustments_hub(
            if joint {
                FederalStatus::Joint
            } else {
                FederalStatus::Single
            },
            0,
        );
        let accounts = if branch == 0 || branch == 1 || branch == 5 || branch == 6 {
            vec![
                hsa_account(Owner::Taxpayer, HsaCoverageYear::Family, 0),
                hsa_account(Owner::Spouse, HsaCoverageYear::Family, 0),
            ]
        } else {
            vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)]
        };
        let split = if joint {
            FamilySplit::Agreed { you: 1, spouse: 0 }
        } else {
            FamilySplit::NotApplicable
        };
        let good = save_hsa(&session.draft, accounts, split);
        let mut bad = good.clone();
        let P::SaveHsa { value, .. } = &mut bad else {
            unreachable!()
        };
        let (key, path) = match branch {
            0 => {
                value.accounts[1].owner = Owner::Taxpayer;
                ("HSA_ACCOUNT_TWICE", "[value/accounts/1/owner]")
            }
            1 => {
                value.accounts.push(value.accounts[0].clone());
                // 2026-10-15 D-13: no account-count cap; identify the duplicate owner.
                ("HSA_ACCOUNT_TWICE", "[value/accounts/2/owner]")
            }
            2 => {
                value.other_coverage = Some(HsaEligibility::FullYear {
                    coverage: HsaCoverage::None,
                });
                ("HSA_OTHER_OWNER_CONTEXT_INVALID", "[value/other_coverage]")
            }
            3 => {
                value.family_method = HsaAllocationMethod::December;
                value.accounts.clear();
                ("HSA_LIMIT_METHOD_NEEDS_ACCOUNT", "[value/family_method]")
            }
            4 => {
                value.other_coverage = Some(HsaEligibility::Monthly { months: vec![] });
                ("HSA_OTHER_OWNER_CONTEXT_INVALID", "[value/other_coverage]")
            }
            5 => {
                value.family_split = FamilySplit::Agreed { you: 0, spouse: 0 };
                ("HSA_FAMILY_PARTS_ZERO", "[value/family_split]")
            }
            6 => {
                value.other_coverage = Some(HsaEligibility::FullYear {
                    coverage: HsaCoverage::None,
                });
                ("HSA_OTHER_OWNER_CONTEXT_INVALID", "[value/other_coverage]")
            }
            _ => unreachable!(),
        };
        admission_hsa_case(
            &alloc::format!("hsa-admission-context-{branch}"),
            &mut session,
            bad,
            good,
            (key, path),
        );
    }
}

#[test]
fn unit47_hsa_admission_actual_w2_employer_aggregate_owner_records() {
    for archer in [false, true] {
        let mut session = at_the_adjustments_hub(FederalStatus::Single, 100);
        if archer {
            let mut wage = w2(6_000_000);
            wage.more.box12.push(Box12Entry { code: W2Code::R, amount: 100 });
            let id = session.draft.income[0].id;
            let request = save_topic(
                &session.draft,
                TaxSection::Wages,
                vec![document(id, IncomeDocument::W2(wage))],
            );
            accepted(&send(&mut session, request));
        }
        let mut good = save_hsa(
            &session.draft,
            if archer {
                vec![]
            } else {
                vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)]
            },
            FamilySplit::NotApplicable,
        );
        let P::SaveHsa { value, .. } = &mut good else { unreachable!() };
        assert_eq!(value.employer_sources.iter().map(|row| row.reported).sum::<i64>(), 100);
        // 2026-10-16 D-2026-10-15-13: a known total above the old
        // ceiling is valid; source plus outside must fit its actual i64 carrier.
        if archer {
            value.archer = ArcherContributionsPage::Records {
                sharing: ArcherSharing::Equal,
                owners: vec![archer_contributor_row(
                    Owner::Taxpayer,
                    0,
                    100_000_000_001 - 100,
                    ArcherEligibility::FullYear {
                        coverage: ArcherCoverage::SelfOnly { deductible: 300_000, out_of_pocket: 400_000 },
                    },
                    vec![session.draft.income[0].id],
                )],
            };
        } else {
            value.accounts[0].employer = 100_000_000_001 - 100;
        }
        let mut bad = good.clone();
        let P::SaveHsa { value, .. } = &mut bad else { unreachable!() };
        let path = if archer {
            let ArcherContributionsPage::Records { owners, .. } = &mut value.archer else {
                unreachable!()
            };
            owners[0].employer = i64::MAX;
            "[value/archer/owners/0]"
        } else {
            value.accounts[0].employer = i64::MAX;
            "[value/accounts/0]"
        };
        admission_hsa_case(
            &alloc::format!("hsa-admission-employer-{archer}"),
            &mut session,
            bad,
            good,
            ("HEALTH_EMPLOYER_TOTAL_INVALID", path),
        );
    }
}

#[test]
fn unit47_income_admission_retained_business_receipts_never_guess_a_submitted_row() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &session.draft,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeDocument::Business(business(0)))],
    );
    accepted(&send(&mut session, request));
    let business_id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Business { .. }))
        .unwrap()
        .id;
    let source = Form1099Misc {
        rents: 0,
        rents_use: None,
        royalties: 0,
        royalties_use: None,
        other: 100,
        other_use: Some(IncomeUse::Business),
        withholding: 0,
        fishing: 0,
        medical: 0,
        substitute: 0,
        attorney: 0,
        client_funds: 0,
        fish_sales: 0,
        deferred: 0,
        business: Some(business_id),
        state_details: Default::default(),
    };
    let request = save_topic(
        &session.draft,
        TaxSection::OtherIncome,
        vec![document(0, IncomeDocument::Form1099Misc(source.clone()))],
    );
    accepted(&send(&mut session, request));
    let mut retained = business(0);
    retained.returns = 100;
    let request = save_topic(
        &session.draft,
        TaxSection::SelfEmployment,
        vec![document(business_id, IncomeDocument::Business(retained))],
    );
    accepted(&send(&mut session, request));
    let source_id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Misc1099 { .. }))
        .unwrap()
        .id;
    let mut bad = source.clone();
    bad.other = 99;
    let request = save_topic(
        &session.draft,
        TaxSection::OtherIncome,
        vec![document(source_id, IncomeDocument::Form1099Misc(bad))],
    );
    for refusal in [
        request.clone(),
        credit_estimate_documents_from_save(&request),
    ] {
        let before = session.draft.clone();
        let dirty = session.dirty;
        let Reply::Response(P::Problem { message, field }) = send(&mut session, refusal) else {
            panic!("retained business refusal")
        };
        assert_eq!(
            message,
            libertas::libertas_formatted_text("BUSINESS_RETURNS_ABOVE_RECEIPTS", &[])
        );
        // The governing accepted business is absent from the submitted Other
        // income page. This intentional residual cannot name the MISC row.
        assert_eq!(field, None);
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
    }
    let request = save_topic(
        &session.draft,
        TaxSection::OtherIncome,
        vec![document(source_id, IncomeDocument::Form1099Misc(source))],
    );
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(&mut session, credit_estimate_documents_from_save(&request)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let page = open_topic(&mut cold, TaxSection::OtherIncome);
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved);
}
