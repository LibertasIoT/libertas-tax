// Actual request controls for the remaining income/HSA admission predicates.
// Sources and accepted identities are genuine fixtures; these tests do not
// certify a client runtime or exhaust every compound native subcondition.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;
use crate::tests::support::client::income_pages::{open_documents, save_documents, topic_for};
use alloc::string::ToString;

fn income_admission_case(
    case: &str,
    session: &mut Interview,
    topic: TaxSection,
    bad: Vec<IncomeDocumentItem>,
    corrected: Vec<IncomeDocumentItem>,
    expected: (&str, &str),
) {
    let submitted = expected.1.strip_prefix("[documents/documents/").unwrap();
    let (index, tail) = submitted.split_once('/').unwrap();
    let index: usize = index.parse().unwrap();
    let target = &bad[index].document;
    let source = !matches!(target, IncomeSource::Record { .. });
    let owner_topic = topic_for(target);
    assert_eq!(topic, owner_topic, "the attempted answer uses its actual topic");
    let mine = |row: &IncomeDocumentItem| {
        if source {
            !matches!(row.document, IncomeSource::Record { .. })
        } else {
            matches!(row.document, IncomeSource::Record { .. })
                && topic_for(&row.document) == owner_topic
        }
    };
    let posted_index = bad[..index]
        .iter()
        .filter(|row| {
            if source {
                core::mem::discriminant(&row.document) == core::mem::discriminant(target)
            } else {
                mine(row)
            }
        })
        .count();
    let category = match target {
        IncomeSource::Record { .. } => "documents",
        IncomeSource::InterestDividend { .. } => "common/interest_dividends/documents",
        IncomeSource::Farming(_) => "other/farming/documents",
        IncomeSource::OtherIncome { .. } => "other/other_income/documents",
        _ => panic!("unexpected admission owner"),
    };
    let tail = tail.replace("document/value", "document");
    let path = alloc::format!("[documents/{category}/{posted_index}/{tail}");
    let (bad, prior): (Vec<_>, Vec<_>) = bad.into_iter().partition(mine);
    let (corrected, corrected_prior): (Vec<_>, Vec<_>) = corrected.into_iter().partition(mine);
    assert_eq!(prior, corrected_prior, "unselected owners must be unchanged");
    admission_prerequisites(session, prior);
    let request =
        if source { save_documents(session, bad) } else { save_topic(session, owner_topic, bad) };
    let estimate = credit_estimate_documents_from_save(&request);
    credit_refusal(case, session, request, expected.0, &path);
    credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, expected.0, &path);
    let request = if source {
        save_documents(session, corrected.clone())
    } else {
        save_topic(session, owner_topic, corrected.clone())
    };
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        credit_estimate_documents_from_save(&request),
    );
    assert!(
        match (source, owner_topic) {
            (true, _) => matches!(reply, Reply::Response(P::Review { .. })),
            (false, TaxSection::Wages) =>
                matches!(reply, Reply::Response(P::RunningEstimate { .. })),
            (false, _) => matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. })),
        },
        "{case}: corrected estimate: {reply:?}"
    );
    assert_eq!(session.draft, before, "{case}: corrected estimate mutated draft");
    assert_eq!(session.dirty, dirty);
    if source {
        accepted(&crate::tests::support::client::income_pages::save_source_fixtures(
            session, corrected,
        ));
    } else {
        accepted(&send(session, request));
    }
    let saved = session.draft.clone();
    let page = if source { open_documents(session) } else { open_topic(session, owner_topic) };
    accepted(&send(session, page));
    assert_eq!(session.draft, saved, "{case}: reopen/resave");
    let mut cold = restart(saved.clone());
    let page = if source { open_documents(&mut cold) } else { open_topic(&mut cold, owner_topic) };
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved, "{case}: restart/reopen/resave");
}

fn admission_prerequisites(session: &mut Interview, rows: Vec<IncomeDocumentItem>) {
    let (records, sources): (Vec<_>, Vec<_>) =
        rows.into_iter().partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    if !sources.is_empty() {
        let request = save_documents(session, sources.clone());
        let desired = crate::tests::support::client::income_pages::page_documents(&request);
        accepted(&send(session, request));
        // The printed sources' topic answers travel through their own topics.
        crate::tests::support::client::income_pages::save_information_return_answers(
            session, &desired,
        );
    }
    let mut topics = Vec::new();
    for row in &records {
        let topic = topic_for(&row.document);
        if !topics.contains(&topic) {
            topics.push(topic);
        }
    }
    for topic in topics {
        let mut request = open_topic(session, topic);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        for row in records.iter().filter(|row| topic_for(&row.document) == topic).cloned() {
            let row = crate::IncomeRecordItem::from_income(row).unwrap();
            if row.id != 0
                && let Some(old) = documents.documents.iter_mut().find(|old| old.id == row.id)
            {
                *old = row;
            } else {
                documents.documents.push(row);
            }
        }
        accepted(&send(session, request));
    }
}

fn admission_duplicate(
    case: &str,
    draft: FederalDraft,
    topic: TaxSection,
    answer: IncomeSource,
    intervening: IncomeSource,
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

fn admission_qbi() -> IncomeSource {
    IncomeSource::Record {
        value: crate::IncomeRecord::QbiLossCarryforward(QbiLossCarryforward {
            business: 0,
            reit_ptp: 0,
            injured_owners: None,
        }),
    }
}
fn admission_foreign_account() -> IncomeSource {
    IncomeSource::Record {
        value: crate::IncomeRecord::ForeignAccountOrTrust(ForeignAccountOrTrust {
            account: false,
            trust: false,
            highest_value: None,
            fbar_required: None,
            fbar_countries: vec![],
        }),
    }
}

#[test]
fn unit47_income_admission_whole_return_duplicates_keep_original_submitted_rows() {
    let whole = "[documents/documents/2/document/value]";
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
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(BusinessPropertyLosses {
                        regular: vec![],
                        amt: vec![],
                    }),
                },
                admission_qbi(),
                "BUSINESS_PROPERTY_LOSSES_TWICE",
                whole,
            ),
            2 => (
                "income-qbi-duplicate",
                TaxSection::SelfEmployment,
                admission_qbi(),
                IncomeSource::Record { value: crate::IncomeRecord::Business(business(100_000)) },
                "QBI_LOSS_CARRYFORWARD_TWICE",
                whole,
            ),
            3 => (
                "income-foreign-account-duplicate",
                TaxSection::InterestDividends,
                admission_foreign_account(),
                IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(false) },
                "FOREIGN_ACCOUNT_TWICE",
                whole,
            ),
            4 => (
                "income-premium-duplicate",
                TaxSection::InterestDividends,
                IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(false) },
                admission_foreign_account(),
                "TAXABLE_PREMIUM_ELECTION_TWICE",
                whole,
            ),
            5 => (
                "income-8606-duplicate",
                TaxSection::Retirement,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Form8606(Form8606 {
                        adjustment: 0,
                        reservist_repayments: vec![],
                        basis: 100_000,
                        year_end: 200_000,
                    }),
                },
                IncomeSource::Record {
                    value: crate::IncomeRecord::RothHistory(RothIraHistory {
                        first_year: 2010,
                        amounts: Some(RothOrdering { regular: 80_000, conversions: vec![] }),
                    }),
                },
                "FORM_8606_TWICE",
                "[documents/documents/2/owner]",
            ),
            6 => (
                "income-roth-duplicate",
                TaxSection::Retirement,
                IncomeSource::Record {
                    value: crate::IncomeRecord::RothHistory(RothIraHistory {
                        first_year: 2010,
                        amounts: Some(RothOrdering { regular: 80_000, conversions: vec![] }),
                    }),
                },
                IncomeSource::Record {
                    value: crate::IncomeRecord::Form8606(Form8606 {
                        adjustment: 0,
                        reservist_repayments: vec![],
                        basis: 100_000,
                        year_end: 200_000,
                    }),
                },
                "ROTH_HISTORY_TWICE",
                "[documents/documents/2/owner]",
            ),
            7 => (
                "income-clergy-duplicate",
                TaxSection::Wages,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Clergy(ClergyRecord {
                        role: ClergyRole::ReligiousSect,
                    }),
                },
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(100_000))) },
                "CLERGY_TWICE",
                "[documents/documents/2/owner]",
            ),
            8 => (
                "income-nonfarm-duplicate",
                TaxSection::SelfEmployment,
                IncomeSource::Record {
                    value: crate::IncomeRecord::NonfarmOptionalMethod(se_history()),
                },
                IncomeSource::Record { value: crate::IncomeRecord::Business(business(100_000)) },
                "NONFARM_OPTIONAL_TWICE",
                whole,
            ),
            9 => (
                "income-able-duplicate",
                TaxSection::OtherIncome,
                IncomeSource::Record {
                    value: crate::IncomeRecord::AbleContributions(AbleContributionsPage {
                        own: 0,
                        other: 0,
                        work: AbleWork::No,
                        year_end: 0,
                        additional_open: false,
                    }),
                },
                IncomeSource::Record {
                    value: crate::IncomeRecord::EducationAccountExpenses(
                        EducationAccountExpenses {
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
                        },
                    ),
                },
                "ABLE_CONTRIBUTIONS_TWICE",
                "[documents/documents/2/owner]",
            ),
            10 => (
                "income-social-election-duplicate",
                TaxSection::Retirement,
                IncomeSource::Record {
                    value: crate::IncomeRecord::SocialLumpSums(LumpSumElection {
                        printed_years: Vec::new(),
                        years: vec![social_prior_year()],
                        complete: true,
                    }),
                },
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(
                        200_000, 0,
                    ))),
                },
                "LUMP_SUM_ELECTION_TWICE",
                whole,
            ),
            11 => (
                "income-1256-duplicate",
                TaxSection::InvestmentSales,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Section1256Carryback(Section1256Carryback {
                        amount: 100,
                        injured_owners: None,
                    }),
                },
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Contracts1099B(contracts_page(
                        "", -1_000,
                    )),
                },
                "SECTION_1256_CARRYBACK_TWICE",
                whole,
            ),
            12 => (
                "income-farm-loss-duplicate",
                TaxSection::Farming,
                IncomeSource::Record {
                    value: crate::IncomeRecord::FarmLoss(FarmLossWaiver {
                        choice: FarmLossChoice::Waive,
                    }),
                },
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(5_000_000, 0)) },
                "FARM_LOSS_TWICE",
                whole,
            ),
            13 => (
                "income-farm-averaging-duplicate",
                TaxSection::Farming,
                IncomeSource::Record {
                    value: crate::IncomeRecord::FarmAveraging(averaging_election()),
                },
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(5_000_000, 0)) },
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
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Business(business(10_000)) },
            ),
            document(0, admission_qbi()),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(bad) }),
        ],
        vec![
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Business(business(10_000)) },
            ),
            document(0, admission_qbi()),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(good) }),
        ],
        ("BUSINESS_RETURNS_ABOVE_RECEIPTS", "[documents/documents/2/document/value/returns]"),
    );
}

#[test]
fn unit47_income_admission_actual_form8379_qbi_owner_totals() {
    for business_part in [true, false] {
        let mut draft = complete(FederalStatus::Joint);
        draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
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
        let Some(InjuredQbiLossOwners::Shares { taxpayer_business, taxpayer_reit_ptp, .. }) =
            &mut bad.injured_owners
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
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Business(business(100_000)),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::BusinessPropertyLosses(
                            BusinessPropertyLosses { regular: vec![], amt: vec![] },
                        ),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(bad) },
                ),
            ],
            vec![
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Business(business(100_000)),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::BusinessPropertyLosses(
                            BusinessPropertyLosses { regular: vec![], amt: vec![] },
                        ),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::QbiLossCarryforward(good) },
                ),
            ],
            ("INJURED_QBI_OWNER_TOTAL", "[documents/documents/2/document/value/injured_owners]"),
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
        IncomeSource::Record {
            value: crate::IncomeRecord::Form8814(election_page(
                child,
                vec![ChildStatement::Int1099(InterestPaperFixture::interest_source(
                    election_interest(10_000),
                ))],
            )),
        },
        admission_foreign_account(),
        ("CHILD_ELECTION_TWICE", "[documents/documents/2/document/value/child]"),
    );
    admission_duplicate(
        "income-education-expenses-duplicate",
        complete(FederalStatus::Single),
        TaxSection::OtherIncome,
        IncomeSource::Record {
            value: crate::IncomeRecord::EducationAccountExpenses(EducationAccountExpenses {
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
        },
        IncomeSource::Record {
            value: crate::IncomeRecord::AbleContributions(AbleContributionsPage {
                own: 0,
                other: 0,
                work: AbleWork::No,
                year_end: 0,
                additional_open: false,
            }),
        },
        ("EDUCATION_ACCOUNT_EXPENSES_TWICE", "[documents/documents/2/document/value]"),
    );
    admission_duplicate(
        "income-foreign-earned-duplicate",
        complete(FederalStatus::Single),
        TaxSection::WorkAbroad,
        IncomeSource::Record {
            value: crate::IncomeRecord::Form2555(Form2555 {
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
        },
        IncomeSource::Record {
            value: crate::IncomeRecord::ForeignPay(ForeignPay {
                employer: "Foreign employer".into(),
                wages: 100_000,
            }),
        },
        ("FORM_2555_TWICE", "[documents/documents/2/document/value]"),
    );
}

fn admission_bond_session() -> (Interview, IncomeDocumentItem, SavingsBonds) {
    let mut draft = complete(FederalStatus::Joint);
    let people = draft.people.as_mut().unwrap();
    people.taxpayer.first_name = "Alex".into();
    people.spouse.as_mut().unwrap().first_name = "Jordan".into();
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let interest = Int1099 {
        payer: (Some(accepted_document_names(&session.draft, Owner::Taxpayer)))
            .as_ref()
            .and_then(|names| names.issuer.clone())
            .unwrap_or_default(),
        recipient: (Some(accepted_document_names(&session.draft, Owner::Taxpayer)))
            .as_ref()
            .map(|names| names.recipient.clone())
            .unwrap_or_default(),
        more: IntMore { us_obligations: 300_000, ..Default::default() },
        ..native_int1099("Treasury".into(), 0)
    };
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(interest)),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let source = page_documents(&open_documents(&mut session))
        .iter()
        .find(|item| {
            matches!(
                item.document,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(_)
                }
            )
        })
        .unwrap()
        .clone();
    assert!(source.id > 0);
    let bonds = SavingsBonds {
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        proceeds: 1_200_000,
        redemption_interest: 300_000,
        earliest_issue_date: 20100601,
        interest_statement: source.id,
        students: vec![SavingsBondStudent {
            person: -1,
            expenses: SavingsBondExpenses::Entered { tuition: 400_000, benefits: 0 },
            contributions: 0,
        }],
    };
    (session, source, bonds)
}

#[test]
fn unit47_income_admission_bonds_actual_source_identity_owner_and_gross() {
    use crate::tests::support::client::income_pages::{
        information_return_answers, save_information_return_answers,
    };
    for branch in 0..4 {
        let (mut session, source, bonds) = admission_bond_session();
        let paper = |value| IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
                savings_bonds_source(value),
            )),
        };
        if branch == 0 {
            let answer = paper(bonds.clone());
            income_admission_case(
                "income-bonds-source-0",
                &mut session,
                TaxSection::InterestDividends,
                vec![
                    source.clone(),
                    document(0, answer.clone()),
                    document(0, answer.clone()),
                    document(
                        0,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::TaxableBondPremium(false),
                        },
                    ),
                ],
                vec![
                    source,
                    document(
                        0,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::TaxableBondPremium(false),
                        },
                    ),
                    document(0, answer),
                ],
                ("SAVINGS_BONDS_TWICE", "[documents/documents/2/document/value]"),
            );
            continue;
        }
        let mut bad = bonds.clone();
        match branch {
            1 => {
                bad.interest_statement = session
                    .draft
                    .income
                    .iter()
                    .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
                    .unwrap()
                    .id
            }
            2 => {
                bad.source_recipient =
                    accepted_document_names(&session.draft, Owner::Spouse).recipient
            }
            3 => bad.redemption_interest += 1,
            _ => unreachable!(),
        }
        let mut row = document(0, paper(bad.clone()));
        if branch == 2 {
            row.owner = crate::DocumentOwner::Spouses;
        }
        let request = save_documents(&mut session, vec![source.clone(), row.clone()]);
        accepted(&send(&mut session, request));
        row.id = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::SavingsBonds { .. }))
            .unwrap()
            .id;
        // The source link, accepted owner and source gross are checked with the
        // actual bond records after the printed Form 8815 has been saved.
        let request = information_return_answers(&mut session, &[row.clone()]);
        let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
        let index = followups
            .iter()
            .position(|row| matches!(row.answer, crate::DocumentAnswer::SavingsBondsRecords { .. }))
            .unwrap();
        let path = libertas::SchemaPath::root()
            .field("followups")
            .key(followups[index].key.as_str())
            .field("answer")
            .field("value")
            .field("interest_statement")
            .to_string();
        let case = alloc::format!("income-bonds-source-{branch}");
        let estimate = credit_estimate_documents_from_save(&request);
        credit_refusal(&case, &mut session, request, "SAVINGS_BONDS_SOURCE_INVALID", &path);
        credit_refusal(
            &alloc::format!("{case}-estimate"),
            &mut session,
            estimate,
            "SAVINGS_BONDS_SOURCE_INVALID",
            &path,
        );
        let corrected = document(row.id, paper(bonds));
        let request = save_documents(&mut session, vec![source, corrected.clone()]);
        accepted(&send(&mut session, request));
        // The interest topic saved below carries the corrected bond records.
        save_information_return_answers(&mut session, core::slice::from_ref(&corrected));
        let premium = document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(false) },
        );
        admission_prerequisites(&mut session, vec![premium]);
        let request = information_return_answers(&mut session, core::slice::from_ref(&corrected));
        let before = session.draft.clone();
        let dirty = session.dirty;
        let reply = crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            credit_estimate_documents_from_save(&request),
        );
        assert!(
            matches!(reply, Reply::Response(P::IncomeRunningEstimate { .. })),
            "{case}: corrected estimate"
        );
        assert_eq!(session.draft, before, "{case}: corrected estimate mutated draft");
        assert_eq!(session.dirty, dirty);
        save_information_return_answers(&mut session, &[corrected]);
        let saved = session.draft.clone();
        let page = open_topic(&mut session, TaxSection::InterestDividends);
        accepted(&send(&mut session, page));
        assert_eq!(session.draft, saved, "{case}: reopen/resave");
        let mut cold = restart(saved.clone());
        let page = open_topic(&mut cold, TaxSection::InterestDividends);
        accepted(&send(&mut cold, page));
        assert_eq!(cold.draft, saved, "{case}: restart/reopen/resave");
    }
}

#[test]
fn unit47_income_admission_texas_withholding_management() {
    let source = Int1099 {
        more: IntMore { withholding: 100, ..Default::default() },
        ..native_int1099("Community interest".into(), 10_000)
    };
    let mut bad = document(
        0,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(interest_document(source)),
        },
    );
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
        crate::tests::support::client::admit_native_fixture_facts(&mut session);
        admission_prerequisites(
            &mut session,
            vec![
                document(0, admission_foreign_account()),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(false) },
                ),
            ],
        );
        let missing = save_documents(&mut session, vec![bad.clone()]);
        accepted(&send(&mut session, missing));
        let mut topic = open_topic(&mut session, TaxSection::InterestDividends);
        let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
        for row in followups.iter_mut() {
            if let crate::DocumentAnswer::Community { value } = &mut row.answer {
                *value = bad.community;
            }
        }
        accepted(&send(&mut session, topic));
        let source = session.draft.income.last().unwrap().id;
        let management_key = alloc::format!("income/{source}/management");
        let reply =
            crate::tests::cross::interview::from_review(&mut session, TaxSection::InjuredSpouse);
        let Reply::Edit(mut topic @ P::SaveInjuredSpouse { .. }) =
            crate::tests::support::client::native_editor(reply, &[])
        else {
            panic!("expected the injured-spouse editor")
        };
        let P::SaveInjuredSpouse { followups, .. } = &mut topic else { unreachable!() };
        let management = followups.iter().position(|row| row.key == management_key);
        assert_eq!(management.is_some(), needed);
        let before = session.draft.clone();
        let dirty = session.dirty;
        if let Some(index) = management {
            assert!(matches!(
                followups[index].answer,
                crate::InjuredSpouseDocumentAnswer::Management { value: None }
            ));
            let path = libertas::SchemaPath::root()
                .field("followups")
                .key(followups[index].key.as_str())
                .field("answer")
                .field("value")
                .to_string();
            credit_refusal(
                "income-texas-withholding-management",
                &mut session,
                topic.clone(),
                "INJURED_SPOUSE_WITHHOLDING_MANAGEMENT_NEEDED",
                &path,
            );
            let P::SaveInjuredSpouse { followups, .. } = &mut topic else { unreachable!() };
            followups[index].answer = crate::InjuredSpouseDocumentAnswer::Management {
                value: Some(RefundManagement::Taxpayer),
            };
        }
        let reply = crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::injured_capacity_calculation(&topic),
        );
        accepted(&reply);
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, topic));
        let saved = session.draft.clone();
        let reply =
            crate::tests::cross::interview::from_review(&mut session, TaxSection::InjuredSpouse);
        let Reply::Edit(mut reopened @ P::SaveInjuredSpouse { .. }) =
            crate::tests::support::client::native_editor(reply, &[])
        else {
            panic!("expected the injured-spouse editor")
        };
        let P::SaveInjuredSpouse { followups, .. } = &mut reopened else { unreachable!() };
        assert_eq!(followups.iter().any(|row| row.key == management_key), needed);
        if needed {
            assert!(followups.iter().any(|row| row.key == management_key
                && matches!(
                    row.answer,
                    crate::InjuredSpouseDocumentAnswer::Management {
                        value: Some(RefundManagement::Taxpayer)
                    }
                )));
        }
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved, "topic reopen/resave");
        let mut cold = restart(saved.clone());
        let page = open_documents(&mut cold);
        accepted(&send(&mut cold, page));
        assert_eq!(cold.draft, saved, "source restart/reopen/resave");
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
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(elected.clone()) }),
            document(0, admission_qbi()),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(bad) }),
        ],
        vec![
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(elected) }),
            document(0, admission_qbi()),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(corrected) }),
        ],
        (
            "FARM_CONSERVATION_METHOD_CONFLICT",
            "[documents/documents/2/document/value/more/conservation/method]",
        ),
    );
}

#[test]
fn unit47_income_admission_patr_combined_amounts_save_estimate_preview() {
    let good = Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage: 100_000,
        nonpatronage: 0,
        per_unit_retains: 0,
        withholding: 0,
        redeemed: 0,
        more: PatrMore { qualified_items: 60_000, sstb_items: 40_000, ..Default::default() },
    };
    // No explicit allocation rows: changing the source total must reach its
    // canonical amount bound before source allocations are generated.
    let mut bad = good.clone();
    bad.more.sstb_items += 1;
    let key = "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26";
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let bad_source = save_documents(
        &mut session,
        vec![document(0, IncomeSource::Farming(cooperative_document(bad.clone())))],
    );
    credit_refusal(
        "income-patr-combined-source",
        &mut session,
        bad_source,
        key,
        "[documents/other/farming/documents/0/document/qualified_items]",
    );
    income_admission_case(
        "income-patr-combined",
        &mut session,
        TaxSection::Farming,
        vec![
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(100_000, 0)) },
            ),
            document(0, admission_qbi()),
            document(0, IncomeSource::Farming(cooperative_document(bad))),
        ],
        vec![
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(100_000, 0)) },
            ),
            document(0, admission_qbi()),
            document(0, IncomeSource::Farming(cooperative_document(good))),
        ],
        (key, "[documents/documents/2/document/qualified_items]"),
    );
    // $600 non-SSTB plus the eligible $400 SSTB class join the same
    // directly conducted trade; the latter is added by the cooperative owner.
    let result = supported(&session.draft);
    assert_eq!(value(&result, "business_2_qbi"), 1_858);
    assert_eq!(value(&result, "qbi_component"), 372);
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
        followups,
        ..
    } = request
    else {
        unreachable!()
    };
    P::EstimateHsa {
        followups: followups.clone(),
        questions: Vec::new(),
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
fn admission_hsa_case(case: &str, session: &mut Interview, bad: Option<(P, &str, &str)>, good: P) {
    if let Some((bad, key, path)) = bad {
        let estimate = admission_hsa_estimate(&bad);
        credit_refusal(case, session, bad, key, path);
        credit_refusal(&alloc::format!("{case}-estimate"), session, estimate, key, path);
    }
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        admission_hsa_estimate(&good),
    );
    assert!(matches!(reply, Reply::Response(P::RunningEstimate { .. })), "{case}: {reply:?}");
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
    for branch in [0, 2, 3, 5, 6] {
        let joint = branch == 0 || branch == 1 || branch == 5 || branch == 6;
        let mut session = at_the_adjustments_hub(
            if joint { FederalStatus::Joint } else { FederalStatus::Single },
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
        let P::SaveHsa { value, .. } = &mut bad else { unreachable!() };
        let (key, path) = match branch {
            0 => {
                value.accounts[1].owner = Owner::Taxpayer;
                ("HSA_ACCOUNT_TWICE", "[value/accounts/1/owner]")
            }
            2 => {
                value.other_coverage =
                    Some(HsaEligibilityPage::FullYear { coverage: HsaCoverage::None });
                ("HSA_OTHER_OWNER_CONTEXT_INVALID", "[value/other_coverage]")
            }
            3 => {
                value.family_method = HsaAllocationMethod::December;
                value.accounts.clear();
                ("HSA_LIMIT_METHOD_NEEDS_ACCOUNT", "[value/family_method]")
            }
            5 => {
                value.family_split = FamilySplit::Agreed { you: 0, spouse: 0 };
                ("HSA_FAMILY_PARTS_ZERO", "[value/family_split]")
            }
            6 => {
                value.other_coverage =
                    Some(HsaEligibilityPage::FullYear { coverage: HsaCoverage::None });
                ("HSA_OTHER_OWNER_CONTEXT_INVALID", "[value/other_coverage]")
            }
            _ => unreachable!(),
        };
        admission_hsa_case(
            &alloc::format!("hsa-admission-context-{branch}"),
            &mut session,
            Some((bad, key, path)),
            good,
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
            let request = crate::tests::support::client::income_pages::save_documents(
                &mut session,
                vec![document(
                    id,
                    IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(wage)) },
                )],
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
        // The derived total retains the source and outside contributions.
        if archer {
            value.archer = ArcherContributionsPage::Records {
                sharing: ArcherSharing::Equal,
                owners: vec![archer_contributor_row(
                    Owner::Taxpayer,
                    0,
                    100_000_000_001 - 100,
                    ArcherEligibility::FullYear {
                        coverage: ArcherCoverage::SelfOnly {
                            deductible: 300_000,
                            out_of_pocket: 400_000,
                        },
                    },
                    vec![session.draft.income[0].id],
                )],
            };
        } else {
            value.accounts[0].employer = 100_000_000_001 - 100;
        }
        admission_hsa_case(
            &alloc::format!("hsa-admission-employer-{archer}"),
            &mut session,
            None,
            good,
        );
    }
}

#[test]
fn unit47_income_admission_retained_business_receipts_never_guess_a_submitted_row() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::Business(business(0)) },
        )],
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
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        crop_insurance: 0,
        component_ids: Default::default(),
        rents: 0,
        royalties: 0,
        other: 100,
        other_use: None,
        withholding: 0,
        fishing: 0,
        medical: 0,
        substitute: 0,
        attorney: 0,
        client_funds: 0,
        fish_sales: 0,
        deferred: 0,
        state_details: Default::default(),
    };
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Form1099Misc(misc_document(source.clone())),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let mut retained = business(0);
    retained.returns = 100;
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(
            business_id,
            IncomeSource::Record { value: crate::IncomeRecord::Business(retained) },
        )],
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
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            source_id,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Form1099Misc(misc_document(bad)),
            },
        )],
    );
    for refusal in [request.clone(), credit_estimate_documents_from_save(&request)] {
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
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            source_id,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Form1099Misc(misc_document(source)),
            },
        )],
    );
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            credit_estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, request));
    // The MISC's question rows walk Other income once: its first save records
    // the visit (G-47); a later unchanged save changes nothing.
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    accepted(&send(&mut session, page));
    assert!(session.draft.finished_sections.contains(&TaxSection::OtherIncome));
    let saved = session.draft.clone();
    let page = open_topic(&mut session, TaxSection::OtherIncome);
    accepted(&send(&mut session, page));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let page = open_topic(&mut cold, TaxSection::OtherIncome);
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved);
}
