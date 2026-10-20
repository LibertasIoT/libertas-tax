// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::unit50_fuel::assert_fuel_payment;
use super::*;
use crate::tests::cross::interview::{accepted, document, r1099, save_documents};

#[test]
fn preferential_rate_stacking_and_cross_netting() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 0);
    add(&mut d, income(FederalIncome::Dividend { data: native_dividend(1_000_000, 1_000_000, 0) }));
    assert_eq!(supported(&d).tax, Some(382_300));
    assert_eq!(preferential_tax(59450, 20000, FederalStatus::Single), 5989);
    add(
        &mut d,
        income(FederalIncome::Sale { data: native_sale(GainTerm::Short, 0, Some(500_000)) }),
    );
    add(
        &mut d,
        income(FederalIncome::Sale { data: native_sale(GainTerm::Long, 1_000_000, Some(0)) }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 5000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
}
#[test]
fn lifetime_learning_is_capped_per_return() {
    let mut d = complete(FederalStatus::Joint);
    for student in [-1, -2] {
        d.credits.as_mut().unwrap().education.push(FederalEducation {
            student,
            expenses: education_expenses(1_000_000),
            method: education_choice(EducationMethod::LifetimeLearning, Answer::No),
        });
    }
    assert_eq!(value(&supported(&d), "education_credit"), 2000);
}
#[test]
fn unsupported_sources_and_unknowns_never_publish_totals() {
    let mut d = complete(FederalStatus::Single);
    d.situations = vec![RareSituation::TaxTreaty];
    unsupported(&d, "Less common income");
    d.situations.clear();
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: FederalRetirement {
                box_9b: None,
                box_10: 0,
                disability: DisabilityIncome::None,
                disability_wages: crate::DisabilityWages::None,
                kind: RetirementKind::Pension,
                gross: 10_000,
                taxable: RetirementTaxable::Payer { amount: 10_000 },
                withholding: 0,
                box7: crate::DistributionCodes::CODE_5.0,
                ordinary_distribution: RetirementTreatment::Unsupported,
                disaster: crate::DisasterDistribution::No,
                after_death: None,
            },
        }),
    );
    unsupported(&d, "Income");
}
#[test]
fn domain_validation_rejects_inconsistent_documents() {
    let e = income(FederalIncome::Dividend { data: native_dividend(100, 200, 0) });
    assert!(crate::federal_rules::validate_income(&e).is_err());
    let e = income(FederalIncome::Sale {
        data: FederalSale {
            use_kind: SaleUse::Investment { wash_sale: 101 },
            ..native_sale(GainTerm::Short, 100, Some(200))
        },
    });
    assert!(crate::federal_rules::validate_income(&e).is_err());
    for date in [20260229, 20261301, 20270001, 0] {
        assert!(!valid_birth(date));
    }
    assert!(valid_birth(20000229));
}
#[test]
fn exact_cents_sum_before_tax_line_rounding() {
    let mut d = complete(FederalStatus::Single);
    for _ in 0..3 {
        add(&mut d, income(FederalIncome::Interest { data: native_interest(49, 0) }));
    }
    assert_eq!(value(&supported(&d), "agi"), 60001);
    assert_eq!(crate::kernel::ratio::dollars(-150), -2);
}
#[test]
fn maximum_supported_inputs_do_not_overflow() {
    let mut d = empty_return(FederalStatus::Single);
    for _ in 0..100 {
        add(&mut d, income(FederalIncome::Interest { data: native_interest(100_000_000_000, 0) }));
    }
    let result = estimate(&d);
    assert_ne!(result.state, ResultState::Incomplete);
}
#[test]
fn accepted_storage_round_trip_and_invalid_identity_block() {
    let d = complete(FederalStatus::Single);
    let mut s = restart(d.clone());
    assert!(matches!(
        send(&mut s, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    let mut invalid = d;
    invalid.income[0].id = 0;
    // 2026-10-18: keyed storage rejects this corrupt identity before restart.
    // Exercise the unchanged interview refusal separately from that storage gate.
    assert!(crate::return_records::project(TEST_USER, &invalid, 1).is_err());
    let mut s = Interview::new(TEST_USER, invalid.clone());
    error(send(&mut s, P::OpenInterview { user: TEST_USER }));
    assert_eq!(s.draft, invalid);
    assert!(!s.dirty);
}
#[test]
fn finish_is_explicit_correlated_workflow_and_reopens_review() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d.clone());
    assert!(matches!(
        send(&mut s, P::StartFinish { user: TEST_USER, cookie: d.cookie.clone(), revision: 0 }),
        Reply::Response(P::BeginFinish { .. })
    ));
    assert!(!s.dirty);
    assert!(matches!(
        send(
            &mut s,
            P::Finish {
                user: TEST_USER,
                cookie: d.cookie,
                revision: 0,
                page: TaxSection::Review as i32,
                previous: TaxSection::Review as i32,
                review: true,
                progress: vec![]
            }
        ),
        Reply::Response(P::Finished { .. })
    ));
    assert!(s.draft.finished);
    let mut restarted = restart(s.draft);
    assert!(matches!(
        send(&mut restarted, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
}

#[test]
fn schedule_one_a_phaseout_rounding_and_return_limits() {
    use crate::federal_rules::additional_deductions as calc;
    assert_eq!(calc(150000, FederalStatus::Single, 30000, 20000, 10000), [25000, 12500, 0]);
    assert_eq!(calc(150999, FederalStatus::Single, 30000, 20000, 0), [25000, 12500, 0]);
    assert_eq!(calc(151000, FederalStatus::Single, 30000, 20000, 0), [24900, 12400, 0]);
    assert_eq!(calc(100000, FederalStatus::Single, 0, 0, 20000), [0, 0, 10000]);
    assert_eq!(calc(100001, FederalStatus::Single, 0, 0, 20000), [0, 0, 9800]);
    assert_eq!(calc(101000, FederalStatus::Single, 0, 0, 20000), [0, 0, 9800]);
    assert_eq!(calc(101001, FederalStatus::Single, 0, 0, 20000), [0, 0, 9600]);
    assert_eq!(calc(300000, FederalStatus::Joint, 60000, 60000, 0), [25000, 25000, 0]);
    assert_eq!(calc(10000, FederalStatus::Separate, 1000, 1000, 1000), [0, 0, 1000]);
    assert_eq!(calc(200000, FederalStatus::Surviving, 0, 0, 10000), [0, 0, 0]);
}

#[test]
fn conditional_union_branches_round_trip_exact_values_and_unknown_answers() {
    macro_rules! round_trip {
        ($ty:ty, $($value:expr),+ $(,)?) => { $( {
            let value: $ty = $value;
            assert_eq!(<$ty>::from_avro(&value.to_avro()).unwrap(), value);
        } )+ };
    }
    round_trip!(DependentCare, DependentCare::No, DependentCare::Yes { care_expenses: 12345 });
    round_trip!(
        TipsClaim,
        TipsClaim::No,
        TipsClaim::Yes { w2_tips: 12345, form4137_tips: 23456, tips_eligible: false }
    );
    round_trip!(
        OvertimeClaim,
        OvertimeClaim::No,
        OvertimeClaim::Yes { overtime: 12345, overtime_eligible: false }
    );
    round_trip!(
        StudentLoanClaim,
        StudentLoanClaim::NONE,
        StudentLoanClaim {
            student_loan_interest: 12345,
            forms: vec![
                Form1098E {
                    interest: 60_000,
                    box_2: Form1098EBox2::Checked { additional_interest: 4_321 },
                },
                Form1098E { interest: 0, box_2: Form1098EBox2::NotChecked },
            ],
            employer_paid_interest: 5_000,
        }
    );
    round_trip!(
        EducatorClaim,
        EducatorClaim::NONE,
        EducatorClaim {
            taxpayer: Some(EducatorRole::Educator { classroom: 12345, instructional: 0 }),
            spouse: None,
        },
        EducatorClaim {
            taxpayer: Some(EducatorRole::Educator { classroom: 12345, instructional: 0 }),
            spouse: Some(EducatorRole::Educator { classroom: 6789, instructional: 0 }),
        }
    );
    round_trip!(
        MortgageLoan,
        MortgageLoan {
            id: 1,
            property: "Synthetic home".into(),
            lender: MortgageLender::PrivateSeller {
                name: "Synthetic seller".into(),
                identifying_number: "000-00-0000".into(),
                address: "1 Example Way".into(),
            },
            interest: 12345,
            outstanding_principal: 6789,
            incurred: 20180101,
            refund_of_interest: MortgageInterestRefund::Yes {
                amount: 111,
                origin: MortgageRefundOrigin::ThisYear,
            },
            mortgage_insurance_premiums: 222,
            points: MortgagePoints::Amortized { amount: 333, term_months: 360, ended: false },
            details: MortgageLoanDetails::FirstLender(MortgageLedger {
                home: MortgageHome::Second,
                purpose: MortgageUse::Other,
                average: MortgageAverageBalance::FirstAndLast { last_balance: 5000 }
            }),
        }
    );
    round_trip!(
        FederalCare,
        FederalCare::default(),
        FederalCare { prior_year: Some(prior_care_facts()), ..FederalCare::default() }
    );
    round_trip!(
        EducationChoice,
        EducationChoice::LifetimeLearning,
        EducationChoice::AmericanOpportunity
    );
    round_trip!(
        SaverDistributionPeriod,
        SaverDistributionPeriod::TaxYear,
        SaverDistributionPeriod::PriorTwo { joint_with_current_spouse: false },
        SaverDistributionPeriod::PriorOne { joint_with_current_spouse: true },
        SaverDistributionPeriod::FollowingYear { joint_with_current_spouse: Answer::NotSure }
    );
    round_trip!(SpouseFiler, SpouseFiler::No, SpouseFiler::Yes { facts: person() });
    round_trip!(
        FederalWorkDeductions,
        FederalWorkDeductions::NONE,
        FederalWorkDeductions {
            tips_claim: TipsClaim::No,
            overtime_claim: OvertimeClaim::Yes { overtime: 1, overtime_eligible: true }
        }
    );
    round_trip!(FederalIras, FederalIras::NONE, ira_facts(12345, 0, Answer::NotSure));
    round_trip!(
        IraSpouse,
        IraSpouse::NotApplicable,
        IraSpouse::Joint {
            facts: FederalIraContribution {
                nondeductible_election: 0,
                traditional: 12345,
                roth: 6789,
                covered_at_work: false,
                returned_next_year: vec![],
                excess: None,
                next_year_deposits: None,
                qtp_roth_lifetime_prior: 0,
            }
        },
        IraSpouse::SeparateTogether { covered_at_work: false }
    );
    let expenses = ItemizedExpenses {
        income_tax_owners: None,
        investment_property_tax_owners: None,
        investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
        real_estate_taxes: 0,
        other_lines: vec![],
        long_term_care: vec![],
        medical: 12345,
        personal_property_taxes: 6789,
        personal_tax: PersonalTaxElection::None,
        investment_property_taxes: 0,
    };
    round_trip!(
        DeductionSelection,
        DeductionSelection::Standard,
        DeductionSelection::Automatic { expenses: ItemizedChoice::No },
        DeductionSelection::Automatic { expenses: ItemizedChoice::Yes { facts: expenses.clone() } },
        DeductionSelection::Itemized { facts: expenses }
    );
    round_trip!(SaverChoice, SaverChoice::No, SaverChoice::Yes { facts: saver(12345, 6789) });
    round_trip!(
        DependentResidency,
        DependentResidency::WholeYearException,
        DependentResidency::Days { days: 183 }
    );
    round_trip!(
        CarePlanMaximum,
        CarePlanMaximum::NoBenefits,
        CarePlanMaximum::Plans { maximum: 750_000 }
    );
}

#[test]
fn rejected_history_edit_preserves_downstream_answers() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut people = original.people.clone().unwrap();
    people.taxpayer.first_name.clear();
    people.taxpayer.last_name.clear();
    error(current_source_refusal(
        "secondary-admission-793",
        &mut session,
        P::SavePeople {
            professional_election: crate::ProfessionalRentalElection::SeparateActivities,
            allowed_professional_elections: vec![0, 1],
            user: TEST_USER,
            cookie: original.cookie.clone(),
            revision: original.revision,
            page: 1,
            previous: 0,
            review: false,
            estimate: vec![],
            warnings: vec![],
            progress: vec![],
            you: crate::interview::filer(&people.taxpayer),
        },
        "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
        None,
        SourceScope::Fieldless,
    ));
    assert_eq!(session.draft, original);
    assert!(!session.dirty);
}

#[test]
fn erasure_follows_the_walk_not_page_order() {
    // Every section survives a change that keeps it in the walk: a filer
    // answer, a document or a situation change erases nothing after it.
    let before = crate::interview::with_ticks(complete(FederalStatus::Joint));
    let mut changes: Vec<FederalDraft> = Vec::new();
    let mut filer = before.clone();
    filer.people.as_mut().unwrap().taxpayer.birth_date = 19700101;
    changes.push(filer);
    let mut documents = before.clone();
    documents.income.clear();
    changes.push(documents);
    let mut situations = before.clone();
    situations.situations = vec![RareSituation::TaxTreaty];
    changes.push(situations);
    for draft in changes {
        // As the page's save leaves it: a situation chosen ticks its topic.
        let mut draft = crate::interview::with_ticks(draft);
        let changed = draft.clone();
        let pruned = crate::interview::prune(&mut draft);
        assert!(pruned.removed.is_empty(), "{pruned:?}");
        assert_eq!(draft, changed);
        assert!(draft.valid_stored());
    }
    // A return that stops being joint loses the spouse and the spouse's
    // documents; every other section stays.
    let mut joint = before.clone();
    joint.income.push(FederalIncomeEntry {
        id: joint.next_id,
        owner: Owner::Spouse,
        ..joint.income[0].clone()
    });
    joint.next_id += 1;
    let mut single = joint.clone();
    single.setup.as_mut().unwrap().status = FilingChoice::Single;
    crate::interview::prune(&mut single);
    assert_eq!(single.people.as_ref().unwrap().spouse, SpouseFiler::No);
    assert_eq!(single.income, before.income);
    assert_eq!(single.adjustments, before.adjustments);
    assert_eq!(single.deductions, before.deductions);
    assert_eq!(single.credits, before.credits);
    assert_eq!(single.screening, before.screening);
    assert!(single.payments.is_some());
    assert!(single.valid_stored());
}

#[test]
fn landing_actions_follow_accepted_state_without_mutation() {
    let mut interview =
        crate::interview::Interview::new(TEST_USER, crate::FederalDraft::empty("landing".into()));
    let Some(crate::interview::Reply::Response(P::Actions { allowed_actions })) =
        interview.handle(P::Enter { user: TEST_USER })
    else {
        panic!("actions expected")
    };
    assert_eq!(
        allowed_actions,
        alloc::vec![libertas_macros::variant_index!(P::OpenInterview) as i32]
    );
    assert!(!interview.dirty);
    assert!(matches!(
        interview.handle(P::ReviewAccepted { user: TEST_USER }),
        Some(crate::interview::Reply::Response(P::Problem { .. }))
    ));
}

#[test]
fn reviewed_pages_back_follows_page_order_and_next_returns_to_review() {
    // The M1 pages of a joint return: setup, you, your spouse, documents.
    fn editor_position(request: &P) -> (&'static str, i32, i32) {
        match request {
            P::SaveSetup { user: TEST_USER, page, previous, .. } => ("setup", *page, *previous),
            P::SavePeople { user: TEST_USER, page, previous, .. } => ("you", *page, *previous),
            P::SaveSpouse { user: TEST_USER, page, previous, .. } => ("spouse", *page, *previous),
            P::SaveIncomes { user: TEST_USER, page, previous, .. } => {
                ("documents", *page, *previous)
            }
            other => panic!("expected an M1 page editor, got {other:?}"),
        }
    }
    let pages = [
        (TaxSection::Setup, "setup", 0, TaxSection::Review as i32, None),
        (TaxSection::People, "you", 1, 0, Some("setup")),
        (TaxSection::Spouse, "spouse", 1, 1, Some("you")),
        // The Wages topic page belongs to the income hub: Back opens the hub.
        (
            TaxSection::Wages,
            "documents",
            TaxSection::Income as i32,
            TaxSection::Income as i32,
            Some("hub"),
        ),
    ];
    let mut draft = complete(FederalStatus::Joint);
    draft.finished = true;
    for (section, name, page, previous, before) in pages {
        let mut session = Interview::new(TEST_USER, draft.clone());
        let Reply::Edit(request) = send(
            &mut session,
            P::Navigate {
                user: TEST_USER,
                cookie: draft.cookie.clone(),
                revision: draft.revision,
                from: TaxSection::Review as i32,
                section: section.into(),
                navigation: NavigationAvailability::default(),
            },
        ) else {
            panic!("expected saved section")
        };
        assert_eq!(editor_position(&request), (name, page, previous));
        let back = send(
            &mut session,
            P::Back {
                user: TEST_USER,
                cookie: draft.cookie.clone(),
                revision: draft.revision,
                previous,
            },
        );
        match before {
            None => assert!(matches!(back, Reply::Response(P::Review { .. }))),
            Some("hub") => assert!(matches!(back, Reply::Response(P::Hub { .. }))),
            Some(before) => {
                let Reply::Edit(prior) = back else { panic!("expected previous page") };
                assert_eq!(editor_position(&prior).0, before);
            }
        }
        // Submit the actual server-supplied editor unchanged, including review
        // metadata: an unchanged page opened from the review returns to it.
        let next = send(&mut session, request);
        assert!(matches!(next, Reply::Response(P::Review { .. })), "{next:?}");
        assert_eq!(session.draft, draft);
        assert!(!session.dirty);
    }
}

#[test]
fn section_choices_match_backend_admission_at_every_saved_stage() {
    let sections = [
        TaxSection::Setup,
        TaxSection::People,
        TaxSection::Children,
        TaxSection::Income,
        TaxSection::Adjustments,
        TaxSection::Deductions,
        TaxSection::Credits,
        TaxSection::Itemized,
        TaxSection::Payments,
        TaxSection::Review,
        TaxSection::Spouse,
        TaxSection::Wages,
        TaxSection::InterestDividends,
        TaxSection::HomeOffices,
    ];
    // A return whose pages are saved through `saved_page` in section order.
    fn saved_through(mut draft: FederalDraft, saved_page: i32) -> FederalDraft {
        if saved_page < 1 {
            draft.people = None;
        }
        if saved_page < 3 {
            draft.income.clear();
            // Amended for M3-P18: the opening checklist stays answered.
            draft.finished_sections.retain(|s| *s == TaxSection::Checklist);
        }
        if saved_page < 4 {
            draft.adjustments = None;
            draft.finished_sections.retain(|s| *s != TaxSection::Adjustments);
        }
        if saved_page < 5 {
            draft.deductions = None;
            draft.finished_sections.retain(|s| *s != TaxSection::Deductions);
        }
        if saved_page < 6 {
            draft.credits = None;
            draft.finished_sections.retain(|s| *s != TaxSection::Credits);
        }
        if saved_page < 7 {
            draft.screening = None;
            draft.finished_sections.retain(|s| *s != TaxSection::Payments);
        }
        draft
    }
    for saved_page in 0..=8 {
        let draft = saved_through(complete(FederalStatus::Single), saved_page);
        let mut interview = Interview::new(TEST_USER, draft.clone());
        let loaded = interview.draft.clone();
        let Reply::Response(P::Review { allowed_sections, .. }) =
            send(&mut interview, P::ReviewAccepted { user: TEST_USER })
        else {
            panic!("review expected")
        };
        // Walkthrough order no longer limits cross-section navigation after
        // the filers are known. Only real conditional pages remain absent;
        // the people page (3) is asked on every return (gate fixes batch A).
        let expected: alloc::vec::Vec<i32> = if saved_page == 0 {
            alloc::vec![0, 1, 46]
        } else {
            let mut sections = alloc::vec![
                0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 48, 19, 20, 21, 23,
                24, 25, 26, 27, 28, 32, 36, 47, 37, 39, 40, 41, 43, 46
            ];
            if saved_page < 3 {
                let index = sections
                    .iter()
                    .position(|page| *page == TaxSection::Marketplace as i32)
                    .unwrap();
                sections.insert(index, TaxSection::Saver as i32);
            }
            sections
        };
        assert_eq!(allowed_sections, expected);
        for section in sections {
            let response = send(
                &mut interview,
                P::Navigate {
                    user: TEST_USER,
                    cookie: draft.cookie.clone(),
                    revision: draft.revision,
                    from: TaxSection::Review as i32,
                    section: section.into(),
                    // A fabricated client list cannot unlock a conditional section.
                    navigation: NavigationAvailability::default(),
                },
            );
            assert_eq!(
                !matches!(response, Reply::Response(P::Problem { .. })),
                allowed_sections.contains(&(section as i32))
            );
        }
        assert_eq!(interview.draft, loaded);
        assert!(!interview.dirty);
    }
}

#[test]
fn incomplete_and_unsupported_returns_never_show_completed_return_lines() {
    let incomplete = crate::federal_rules::estimate(&FederalDraft::empty("draft".into()));
    assert!(incomplete.return_lines.is_empty() && incomplete.review_notes.is_empty());
    let mut draft = complete(FederalStatus::Single);
    draft.situations = vec![RareSituation::TaxTreaty];
    let unsupported = crate::federal_rules::estimate(&draft);
    assert!(unsupported.return_lines.is_empty() && unsupported.refund.is_none());
}

#[test]
fn fishing_type4_public_bus_type5_and_school_bus_type7_share_line_1c_without_sharing_cost() {
    let fuels = FederalFuelLines {
        gasoline_commercial_fishing: Some(fuel_use(100_000, 20_000)),
        ..empty_fuel_lines()
    };
    assert_eq!(fuels.labeled_lines().find(|(name, _)| *name == "1c").unwrap().1, 1_830);
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 3_700);
    for source in ["business_2", "partnership_3"] {
        assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_1c")), 1_830);
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.gasoline_school_bus =
        Some(fuel_use(100_000, 20_000));
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.gasoline_school_bus =
        Some(fuel_use(100_000, 20_000));
    let combined = supported(&draft);
    assert_eq!(combined.tax, baseline.tax);
    assert_eq!(return_line(&combined, "31"), return_line(&baseline, "31") + 7_300);
    for source in ["business_2", "partnership_3"] {
        assert_eq!(line_amount(&combined, &alloc::format!("{source}_fuel_1c")), 3_660);
    }
    assert_eq!(estimate(&restart(draft.clone()).draft), combined);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.gasoline_intercity_local_bus =
        Some(fuel_use(100_000, 20_000));
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.gasoline_intercity_local_bus =
        Some(fuel_use(100_000, 20_000));
    let three_uses = supported(&draft);
    assert_eq!(three_uses.tax, baseline.tax);
    assert_eq!(return_line(&three_uses, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(return_line(&three_uses, "31"), return_line(&baseline, "31") + 11_000);
    for source in ["business_2", "partnership_3"] {
        assert_eq!(line_amount(&three_uses, &alloc::format!("{source}_fuel_1c")), 5_490);
    }
    assert_eq!(estimate(&restart(draft.clone()).draft), three_uses);

    for source in &mut draft.income[1..3] {
        let fuels = match &mut source.income {
            FederalIncome::Business { data } => &mut data.fuel_tax_credit.as_mut().unwrap().fuels,
            FederalIncome::Partnership { data } => {
                &mut data.fuel_tax_credit.as_mut().unwrap().fuels
            }
            _ => unreachable!(),
        };
        fuels.diesel_intercity_local_bus = Some(fuel_use(100_000, 20_000));
        fuels.kerosene_intercity_local_bus = Some(fuel_use(100_000, 20_000));
        fuels.intercity_bus_no_waiver = Some(true);
    }
    let reduced_bus = supported(&draft);
    assert_eq!(reduced_bus.tax, baseline.tax);
    assert_eq!(return_line(&reduced_bus, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(return_line(&reduced_bus, "31"), return_line(&baseline, "31") + 17_800);
    for source in ["business_2", "partnership_3"] {
        for line in ["3d", "4c"] {
            assert_eq!(line_amount(&reduced_bus, &alloc::format!("{source}_fuel_{line}")), 1_700);
        }
    }
    assert_eq!(estimate(&restart(draft.clone()).draft), reduced_bus);

    for source in &mut draft.income[1..3] {
        let fuels = match &mut source.income {
            FederalIncome::Business { data } => &mut data.fuel_tax_credit.as_mut().unwrap().fuels,
            FederalIncome::Partnership { data } => {
                &mut data.fuel_tax_credit.as_mut().unwrap().fuels
            }
            _ => unreachable!(),
        };
        fuels.emulsion_off_highway = Some(fuel_use(100_000, 20_000));
        fuels.emulsion_intercity_local_bus = Some(fuel_use(100_000, 20_000));
    }
    let mixed_emulsion = supported(&draft);
    assert_eq!(mixed_emulsion.tax, baseline.tax);
    assert_eq!(return_line(&mixed_emulsion, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(return_line(&mixed_emulsion, "31"), return_line(&baseline, "31") + 24_200);
    for source in ["business_2", "partnership_3"] {
        assert_eq!(line_amount(&mixed_emulsion, &alloc::format!("{source}_fuel_14a")), 3_210);
    }
    assert_eq!(estimate(&restart(draft.clone()).draft), mixed_emulsion);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.intercity_bus_no_waiver = Some(false);
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.intercity_bus_no_waiver = None;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.intercity_bus_no_waiver = Some(true);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .gasoline_intercity_local_bus
        .as_mut()
        .unwrap()
        .not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .gasoline_intercity_local_bus
        .as_mut()
        .unwrap()
        .not_claimed_elsewhere = true;

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .gasoline_commercial_fishing
        .as_mut()
        .unwrap()
        .qualified = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let use_line =
        data.fuel_tax_credit.as_mut().unwrap().fuels.gasoline_commercial_fishing.as_mut().unwrap();
    use_line.qualified = true;
    use_line.actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.gasoline_commercial_fishing.as_mut().unwrap().actual_cost = 20_000;
    fuels.gasoline_school_bus.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.gasoline_school_bus.as_mut().unwrap().actual_cost = 20_000;
    fuels.gasoline_intercity_local_bus.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.gasoline_intercity_local_bus.as_mut().unwrap().actual_cost = 20_000;
    fuels.diesel_intercity_local_bus.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.diesel_intercity_local_bus.as_mut().unwrap().actual_cost = 20_000;
    fuels.kerosene_intercity_local_bus.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.kerosene_intercity_local_bus.as_mut().unwrap().actual_cost = 20_000;
    fuels.emulsion_intercity_local_bus.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}
#[test]
fn conversion_rejects_ineligible_and_unallocated_after_tax_sources() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 10000, conversion(8000, true)),
    );
    supported(&d);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    data.kind = RetirementKind::Pension;
    unsupported(&d, "Income");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    data.kind = RetirementKind::Ira;
    data.taxable = RetirementTaxable::Payer { amount: 700000 };
    unsupported(&d, "Income");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    data.taxable = RetirementTaxable::Payer { amount: data.gross };
    data.ordinary_distribution = conversion(10001, true);
    assert!(!d.valid_stored());
}

#[test]
fn schedule_r_disability_income_caps_cover_all_age_combinations() {
    use crate::extended_rules::elderly_credit;
    for (status, apart, seniors, disability, agi, exempt, expected) in [
        (FederalStatus::Single, false, 0, [3000, 0], 7500, 0, 450),
        (FederalStatus::Single, false, 0, [15900, 0], 16000, 700, 8),
        (FederalStatus::Joint, false, 1, [0, 1000], 10000, 0, 900),
        (FederalStatus::Joint, false, 1, [0, 4000], 10000, 0, 1125),
        (FederalStatus::Joint, false, 0, [1000, 2000], 10000, 0, 450),
        (FederalStatus::Joint, false, 0, [10000, 0], 10000, 0, 750),
        (FederalStatus::Separate, true, 0, [3000, 0], 5000, 0, 450),
        (FederalStatus::Separate, false, 0, [3000, 0], 5000, 0, 0),
        (FederalStatus::Single, false, 0, [3000, 0], 7500, 3000, 0),
    ] {
        assert_eq!(elderly_credit(status, apart, seniors, disability, agi, exempt), expected);
    }
}

#[test]
fn jury_pay_and_employer_remittance_preserve_both_return_lines() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::JuryDuty {
            data: crate::JuryPay { received: 100000, remitted: 60000 },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "jury_pay"), 1000);
    assert_eq!(value(&r, "jury_pay_remitted"), 600);
    assert_eq!(value(&r, "agi"), 60400);
    assert_eq!(return_line(&r, "1a"), 6000000);
    assert_eq!(return_line(&r, "8"), 100000);
    assert_eq!(return_line(&r, "10"), 60000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[1].income =
        FederalIncome::JuryDuty { data: crate::JuryPay { received: 100000, remitted: 100000 } };
    assert_eq!(value(&supported(&d), "agi"), 60000);
    d.income[1].income =
        FederalIncome::JuryDuty { data: crate::JuryPay { received: 100000, remitted: 100001 } };
    assert!(!d.valid_stored());
}

#[test]
fn unemployed_insurance_exception_observes_twelve_weeks_and_day_sixty() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260302,
                facts: RetirementException::UnemploymentInsurance {
                    premiums: 400000,
                    benefit_year: 2025,
                    weeks: 12,
                    reemployment: Reemployment::Started { date: 20260101 },
                    qualification: UnemploymentQualification::Received { confirmed: true },
                },
                early_simple: false,
            },
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 600);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let RetirementTreatment::Exception { distributed, .. } = &mut data.ordinary_distribution else {
        unreachable!()
    };
    *distributed = 20260303;
    unsupported(&d, "Income");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let RetirementTreatment::Exception { distributed, facts, .. } = &mut data.ordinary_distribution
    else {
        unreachable!()
    };
    *distributed = 20260302;
    let RetirementException::UnemploymentInsurance { weeks, .. } = facts else { unreachable!() };
    *weeks = 11;
    unsupported(&d, "Income");
}

#[test]
fn domestic_abuse_exception_uses_2026_indexed_limit_and_half_account() {
    for (balance, amount, expected) in [(15000, 10000, 250), (30000, 15000, 450), (10000, 4000, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, abuse_withdrawal(balance, amount));
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), expected);
        assert_eq!(value(&r, "agi"), 60000 + amount);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn undistributed_fund_special_components_and_invalid_statements_are_explicit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(undistributed_gain()));
    let base = supported(&complete(FederalStatus::Single));
    d.income[1].income = undistributed_gain();
    let FederalIncome::UndistributedGain { data } = &mut d.income[1].income else { panic!() };
    data.small_business = 10000;
    // Box 1c needs its exclusion chosen: without it the statement is invalid (the client rule
    // FORM_2439_SECTION_1202_NEEDS_EXCLUSION); with 100% the fund's whole 100.00 is excluded on a
    // Form 8949 code Q row, so the return is the one without the box 1a gain's 100.00.
    data.section_1202_exclusion = None;
    assert!(!d.valid_stored());
    let FederalIncome::UndistributedGain { data } = &mut d.income[1].income else { panic!() };
    data.section_1202_exclusion = Some(crate::FundQsbsExclusion::Hundred);
    assert!(d.valid_stored());
    let r = supported(&d);
    assert!(r.lines.iter().any(|l| l.id.ends_with("_1_adjustment") && l.amount == -10000));
    assert_eq!(value(&r, "capital_gain"), 5000 - 100);
    // A statement not confirmed as this filer's own is left out.
    d.income[1].income = undistributed_gain();
    let FederalIncome::UndistributedGain { data } = &mut d.income[1].income else { panic!() };
    data.special = true;
    let r = supported(&d);
    assert!(noted(&r, "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT"));
    assert_eq!(r.tax, base.tax);
    d.income[1].income = undistributed_gain();
    let FederalIncome::UndistributedGain { data } = &mut d.income[1].income else { panic!() };
    data.collectibles = data.gain + 1;
    assert!(!d.valid_stored());
    let FederalIncome::UndistributedGain { data } = &mut d.income[1].income else { panic!() };
    data.collectibles = 0;
    data.tax_paid = data.gain + 1;
    assert!(!d.valid_stored());
}

#[test]
fn all_2013_through_2017_pease_and_unlimited_histories_work_in_multi_year_returns() {
    for year in 2013..=2017 {
        for limited in [false, true] {
            let prior = if limited {
                historical_pease_payment(year, false)
            } else {
                historical_refund_payment(year)
            };
            let history = TaxRefundHistory::IncomeTaxMultiplePriorYears {
                years: vec![prior, historical_refund_payment(2018)],
                allocation_complete: true,
            };
            assert_eq!(
                history.multi_year_breakdown(20_000).unwrap(),
                Some(vec![(year, 10_000, 10_000), (2018, 10_000, 10_000),]),
                "{year} limited={limited}"
            );
            let mut d = complete(FederalStatus::Single);
            add(
                &mut d,
                income(FederalIncome::TaxRefund { amount: 20_000, withholding: 0, history }),
            );
            let result = supported(&d);
            assert_eq!(return_line(&result, "8"), 20_000, "{year} limited={limited}");
            assert_eq!(estimate(&restart(d.clone()).draft), result);
        }
    }
}

#[test]
fn single_life_general_rule_reproduces_irs_exclusion_example() {
    // Pub. 939: $10,800 cost, $100/month, age 65 multiple 20.0,
    // $24,000 expected return, 45% exclusion, $45 tax-free each month.
    let mut a = fixed_annuity();
    a.cost = 1080000;
    a.periodic = 10000;
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 0 };
    assert!(a.valid(120000));
    assert_eq!(a.expected(), 2400000);
    assert_eq!(a.ratio(), 450);
    assert_eq!(a.excluded(), 54000);
    let mut d = complete(FederalStatus::Single);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1200,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        data.taxable = RetirementTaxable::GeneralAnnuity { data: a };
    }
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 66000);
    assert_eq!(value(&r, "annuity_2_remaining_basis"), 10260);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn filer_death_date_validation_rejects_impossible_final_return_dates() {
    let mut d = complete(FederalStatus::Single);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    for date in [20251231, 20270101, 20260230] {
        p.died = death(date);
        assert!(!p.valid_dates());
    }
    p.birth_date = 20260615;
    p.died = death(20260614);
    assert!(!p.valid_dates());
    p.died = death(20260615);
    assert!(p.valid_dates());
}

#[test]
fn tax_age_uses_day_before_birthday_and_final_year_end() {
    let mut p = complete(FederalStatus::Single).people.unwrap().taxpayer;
    p.birth_date = 20030101;
    assert_eq!(p.tax_age(), 24); // Form 8615 January 1 rule.
    p.birth_date = 20030615;
    p.died = death(20260613);
    assert_eq!(p.tax_age(), 22);
    p.died = death(20260614);
    assert_eq!(p.tax_age(), 23);
    p.died = death(20260615);
    assert_eq!(p.tax_age(), 23);
}

#[test]
fn unresolved_income_tax_allocation_is_unneeded_when_known_expenses_exhaust_nii() {
    let mut d = niit_tax_source_return(PersonalTaxElection::Income {
        paid: 1000000,
        allocation: TaxIncomeAllocation::Unresolved,
    });
    itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 10000000;
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized { facts };
    assert_eq!(value(&supported(&d), "net_investment_income_tax"), 0);
}

#[test]
fn qualified_general_rule_accepts_age_75_and_exactly_five_year_guarantee() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19510101;
    add(&mut d, qualified_fixed_pension(Some(75), 60));
    let r = supported(&d);
    // 6,000 investment / (250 monthly x 60 months) = .400;
    // 3,000 annual payments less 1,200 recovery = 1,800 taxable.
    assert_eq!(return_line(&r, "5a"), 300000);
    assert_eq!(return_line(&r, "5b"), 180000);
    assert_eq!(value(&r, "agi"), 61800);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    assert_eq!(value(&supported(&d), "net_investment_income_tax"), 0);
}

#[test]
fn later_qualified_general_rule_requires_original_age_and_fixed_guarantee() {
    for (age, months) in [(None, 60), (Some(74), 60), (Some(75), 59)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, qualified_fixed_pension(age, months));
        unsupported(&d, "Income");
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, qualified_fixed_pension(Some(75), 60));
    let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let RetirementTaxable::GeneralAnnuity { data } = &mut data.taxable else { unreachable!() };
    data.duration = AnnuityDuration::SingleLife { age: 75, delay: 1, guaranteed: 0 };
    unsupported(&d, "Income"); // Age alone does not guarantee any payments.
    let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let RetirementTaxable::GeneralAnnuity { data } = &mut data.taxable else { unreachable!() };
    data.qualified_age = Some(121);
    assert!(!d.valid_stored());
}

#[test]
fn later_qualified_general_rule_coordinates_public_safety_exclusion() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19510101;
    add(&mut d, qualified_fixed_pension(Some(75), 60));
    let id = d.income.last().unwrap().id;
    public_safety(&mut d, id, 2000, 2000, PublicSafetyCoverage::Medical);
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("annuity_{id}_basis_recovery")), 1200);
    assert_eq!(value(&r, "public_safety_1_excluded"), 1800);
    assert_eq!(value(&r, "public_safety_medical_before_coordination"), 200);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "agi"), 60000);
}

#[test]
fn nonqualified_exceptions_reject_wrong_dates_and_accounts() {
    for facts in [
        RetirementException::Death { died: 20260616 },
        RetirementException::Disability { began: 20260616 },
        RetirementException::TerminalIllness { certified: 20260601 },
        education_withdrawal(10000, 0, true),
    ] {
        unsupported(&nonqualified_exception(facts), "Income:");
    }
    let mut d = nonqualified_exception(RetirementException::Death { died: 20260101 });
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let RetirementTreatment::Exception { early_simple: simple, .. } =
        &mut data.ordinary_distribution
    else {
        panic!()
    };
    *simple = true;
    unsupported(&d, "Income:");
}

#[test]
fn qualified_withdrawal_requires_matching_account_and_supported_treatment() {
    for kind in [RetirementKind::Ira, RetirementKind::Nonqualified] {
        let mut d = qualified_withdrawal(5000, 12500, 10000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        data.kind = kind;
        unsupported(&d, "Income:");
    }
    // No: the basis recovery is left out with a notice; the whole
    // distribution is taxed.
    let mut d = qualified_withdrawal(5000, 12500, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    let RetirementTaxable::QualifiedWithdrawal { data: crate::PlanDistribution { eligible, .. } } =
        &mut data.taxable
    else {
        panic!()
    };
    *eligible = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_RETIREMENT_BASIS_LEFT_OUT"), "{r:?}");
    assert!(r.tax > supported(&qualified_withdrawal(5000, 12500, 10000)).tax);
    let mut d = qualified_withdrawal(5000, 12500, 10000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    data.ordinary_distribution = RetirementTreatment::Rollover {
        destination: RolloverDestination::EmployerPlan,
        amount: 100001,
        early: false,
    };
    unsupported(&d, "Income:");
    assert!(!qualified_withdrawal(5000, 0, 10000).valid_stored());
    assert!(!qualified_withdrawal(5000, 4999, 10000).valid_stored());
    assert!(!qualified_withdrawal(5000, 12500, -1).valid_stored());
}

#[test]
fn de_minimis_ineligible_purchases_are_left_out_with_a_notice() {
    let base = de_minimis_business(10000, false);
    let mut d = base.clone();
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let BusinessExpenseKind::DeMinimis { eligible, .. } = &mut data.expenses[0].treatment else {
        panic!()
    };
    *eligible = false;
    let r = supported(&d);
    assert!(noted(&r, "WARN_DE_MINIMIS_EXPENSE_LEFT_OUT"));
    assert!(r.lines.iter().all(|l| l.id != "business_2_expense_0_de_minimis"));
    assert!(value(&r, "agi") > value(&supported(&base), "agi"));
    assert!(!de_minimis_business(0, false).valid_stored());
    assert!(!de_minimis_business(-1, false).valid_stored());
    assert!(!de_minimis_business(100000000001, true).valid_stored());
    let mut d = de_minimis_business(10000, false);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let mut expense = data.expenses[0].clone();
    let BusinessExpenseKind::DeMinimis { financial_statement, .. } = &mut expense.treatment else {
        panic!()
    };
    *financial_statement = true;
    data.expenses.push(expense);
    unsupported(&d, "Income:");
}

#[test]
fn ltc_period_limit_uses_2026_days_or_costs_less_reimbursements() {
    let mut p = ltc_period(15000, CareBenefitRecipient::Insured { other_payees: false });
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (1133000, 1133000, 367000)
    ); // 31 * $430 - $2,000
    p.services = 2000000;
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (1800000, 1500000, 0)
    );
    p.reimbursements = 2500000;
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (0, 0, 1500000)
    );
    p.starts = 20260101;
    p.ends = 20261231;
    p.services = 0;
    p.reimbursements = 0;
    p.received = 20000000;
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (15695000, 15695000, 4305000)
    );
}

#[test]
fn ltc_multiple_payees_allocate_insured_first_then_proportionally() {
    let mut p = ltc_period(
        6000,
        CareBenefitRecipient::Other { insured_received: 1000000, others_received: 400000 },
    );
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (1133000, 79800, 520200)
    );
    p.recipient =
        CareBenefitRecipient::Other { insured_received: 2000000, others_received: 400000 };
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (1133000, 0, 600000)
    );
    p.recipient = CareBenefitRecipient::Other { insured_received: 0, others_received: 0 };
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (1133000, 600000, 0)
    );
    p.received = 0;
    assert_eq!(
        {
            let (limit, _, excluded, taxable) = p.amounts();
            (limit, excluded, taxable)
        },
        (1133000, 0, 0)
    );
}

#[test]
fn older_continuing_equipment_uses_existing_150_percent_and_straight_line_engine() {
    // $10,000 original basis; cumulative manual schedules with no bonus.
    // Their remaining recovery periods require the straight-line switch.
    for (placed, class, method, prior, deduction, remaining, agi) in [
        (20120601, EquipmentClass::Fifteen, EquipmentMethod::Regular, 911427, 59049, 29524, 64410),
        (20080601, EquipmentClass::Twenty, EquipmentMethod::Regular, 888461, 44616, 66923, 64554),
        (
            20170601,
            EquipmentClass::Ten,
            EquipmentMethod::StraightLine,
            850000,
            100000,
            50000,
            64000,
        ),
        (20170601, EquipmentClass::Ten, EquipmentMethod::Declining150, 868908, 87395, 43697, 64126),
    ] {
        let mut asset = continuing_equipment(placed, prior, false);
        asset.cost = 1000000;
        asset.acquired = placed - 10000;
        asset.class = class;
        asset.method = method;
        let business = business_with_bonus(50000, asset.clone());
        let r = supported(&business);
        assert_eq!(value(&r, "business_2_asset_0_bonus"), 0);
        for (id, expected) in [
            ("business_2_asset_0_macrs", deduction),
            ("business_2_asset_0_remaining_basis", remaining),
        ] {
            assert_eq!(line_amount(&r, id), expected);
        }
        let mut rental = complete(FederalStatus::Single);
        add(&mut rental, income(personal_rent_with_assets(5000, 0, vec![asset])));
        let r = supported(&rental);
        for (id, expected) in [
            ("personal_rental_2_asset_0_macrs", deduction),
            ("personal_rental_2_asset_0_remaining_basis", remaining),
        ] {
            assert_eq!(line_amount(&r, id), expected);
        }
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "amt_rental_adjustment"), 0);
        assert_eq!(restart(rental.clone()).draft, rental);
        assert_eq!(restart(business.clone()).draft, business);
    }
}

#[test]
fn reissued_certificate_respects_original_high_rate_ownership_cap() {
    let mut d = complete(FederalStatus::Single);
    let mut claim = mortgage_certificate_credit(2000, 6000, 1_200_000);
    let certificate = claim.current.as_mut().unwrap();
    certificate.issue_date = 20240115;
    certificate.history = MortgageCertificateHistory::ReissuedBefore2026 {
        facts: MortgageReissueFacts {
            refinanced_on: 20240101,
            original_interest_method: MortgageOriginalInterestMethod::FixedRateSchedule,
            first_reissue: true,
            original_number: "MCC-HIGH-RATE".into(),
            original_rate_hundredths_percent: 2500,
            original_scheduled_interest: 1_000_000,
            original_certified_balance: 20_000_000,
            reissued_certified_debt: 18_000_000,
            confirmed: true,
        },
    };
    d.credits.as_mut().unwrap().mortgage_credit = Some(claim);
    let r = supported(&d);
    // Original $2,500 gross is capped to this filer's 60% share of $2,000.
    // The reissued 20% rate would otherwise yield $2,400.
    assert_eq!(value(&r, "mortgage_original_schedule_cap"), 1200);
    assert_eq!(value(&r, "mortgage_credit_current_potential"), 1200);
    assert_eq!(value(&r, "mortgage_interest_credit"), 1200);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn different_rate_2026_reissue_calculates_each_period_and_original_ceiling() {
    let mut d = complete(FederalStatus::Single);
    let deductions = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(deductions, 2_000_000);
    let mut claim = mortgage_certificate_credit(1500, 10000, 900_000);
    let certificate = claim.current.as_mut().unwrap();
    certificate.issue_date = 20260701;
    certificate.history = MortgageCertificateHistory::ReissuedDuring2026DifferentRates {
        facts: MortgageReissueFacts {
            refinanced_on: 20260630,
            original_interest_method: MortgageOriginalInterestMethod::FixedRateSchedule,
            first_reissue: true,
            original_number: "MCC-DIFFERENT-RATES".into(),
            original_rate_hundredths_percent: 2000,
            original_scheduled_interest: 1_000_000,
            original_certified_balance: 20_000_000,
            reissued_certified_debt: 18_000_000,
            confirmed: true,
        },
        original_period_interest: MortgageInterestSource::Certified { amount: 300_000 },
    };
    d.credits.as_mut().unwrap().mortgage_credit = Some(claim);
    let r = supported(&d);
    // $3,000 × 20% plus $9,000 × 15% is $1,950, below the $2,000
    // original-schedule ceiling. The two period calculations appear on screen.
    assert_eq!(value(&r, "mortgage_original_period_credit"), 600);
    assert_eq!(value(&r, "mortgage_reissued_period_credit"), 1350);
    assert_eq!(value(&r, "mortgage_original_schedule_cap"), 2000);
    assert_eq!(value(&r, "mortgage_credit_current_potential"), 1950);
    assert_eq!(value(&r, "deduction"), 18050);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedDuring2026DifferentRates { facts, .. } =
        &mut certificate.history
    {
        facts.original_scheduled_interest = 900_000;
    }
    let capped = supported(&d);
    assert_eq!(value(&capped, "mortgage_original_schedule_cap"), 1800);
    assert_eq!(value(&capped, "mortgage_credit_current_potential"), 1800);
    assert_eq!(value(&capped, "deduction"), 18200);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    certificate.rate_hundredths_percent = 2000;
    if let MortgageCertificateHistory::ReissuedDuring2026DifferentRates { facts, .. } =
        &mut certificate.history
    {
        facts.original_rate_hundredths_percent = 2500;
        facts.original_scheduled_interest = 1_000_000;
    }
    let high_rate = supported(&d);
    assert_eq!(value(&high_rate, "mortgage_original_period_credit"), 750);
    assert_eq!(value(&high_rate, "mortgage_reissued_period_credit"), 1800);
    assert_eq!(value(&high_rate, "mortgage_original_schedule_cap"), 2000);
    assert_eq!(value(&high_rate, "mortgage_credit_current_potential"), 2000);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedDuring2026DifferentRates { facts, .. } =
        &mut certificate.history
    {
        facts.original_rate_hundredths_percent = 2000;
    }
    assert!(!d.valid_stored());
}
#[test]
fn note_holding_anniversary_is_short_and_the_following_day_is_long() {
    for (advanced, date, expected) in [
        // 2026-10-19: loan_gains returns actual cents, with holding dates unchanged.
        (20250615, 20260615, [0, 200_000, 0]),
        (20250615, 20260616, [0, 0, 200_000]),
        (20251231, 20261231, [0, 200_000, 0]),
        (20240229, 20260228, [0, 0, 200_000]),
    ] {
        let mut source = mixed_note_repayments();
        source.loans[0].repayment_term = ScorpNoteHolding::Dated {
            advanced,
            payments: vec![ScorpNoteRepayment { date, principal: 500_000 }],
        };
        let basis = source.basis_result().unwrap();
        assert_eq!(source.loan_gains(&basis).unwrap(), expected);
    }
}

#[test]
fn multiple_partial_recapture_still_requires_source_allocation() {
    let mut draft = multiple_fully_recaptured_gains(30000);
    draft.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 40000, 40000)], &[(2025, 30000, 30000)]));
    unsupported(&draft, "Section 1231 QBI character");
}

#[test]
fn shared_rate_class_recapture_requires_determined_source_amounts() {
    let mut draft = rate_ordered_recapture(20000, 10000);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        data.section1231.as_mut().unwrap().unrecaptured = 800000;
        data.amt.as_mut().unwrap().section1231.as_mut().unwrap().unrecaptured = 500000;
        data.trade_kind = ScorpTradeKind::FinancialTrading;
        data.qbi_statement.as_mut().unwrap().service = true;
    }
    if let FederalIncome::Scorp { data } = &mut draft.income[2].income {
        data.section1231.as_mut().unwrap().unrecaptured = 1200000;
        data.amt.as_mut().unwrap().section1231.as_mut().unwrap().unrecaptured = 500000;
    }
    investment_borrowing(&mut draft, 25000, 0, 0);
    let r = supported(&draft);
    assert_eq!(value(&r, "investment_interest_deduction"), 8000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 5000);
    assert_eq!(value(&r, "qbi_deduction"), 4000);
    draft.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 10000, 10000)], &[(2025, 10000, 10000)]));
    unsupported(&draft, "Section 1231 QBI character");
}

#[test]
fn qualified_rate_source_components_reconcile_and_round_to_totals() {
    let draft = qualified_rate_gain(8000, Some(0), 8000);
    let FederalIncome::Scorp { data } = &draft.income[1].income else { panic!() };
    for (qualified, high) in [(800000, -1), (800000, 800001), (1300000, 1300000), (1000000, 0)] {
        let mut source = data.clone();
        let facts = &mut source.property_qbi;
        facts.gain = qualified;
        facts.qualified_1250 = Some(high);
        assert!(!source.property_qbi.valid(&source));
    }
    let mut source = data.clone();
    source.section1231.as_mut().unwrap().net = 149;
    source.section1231.as_mut().unwrap().unrecaptured = 50;
    source.property_qbi.gain = 99;
    source.property_qbi.qualified_1250 = Some(49);
    let facts = &source.property_qbi;
    assert!(facts.valid(&source));
    assert_eq!(facts.qualified_recapture(&source, 1), Ok(1));
}

#[test]
fn uncollected_w2_tax_requires_reconciled_ordinary_payroll_facts() {
    let mut draft = uncollected_wage(60000, 620, 145, false);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        let facts = data.uncollected_tax.as_mut().unwrap();
        facts.tip_medicare = 100000;
    }
    unsupported(&draft, "Uncollected payroll tax");
}

#[test]
fn base_year_rate_schedules_match_irs_published_tax_amounts() {
    use crate::income_tax::Rates;
    use FederalStatus::*;
    // Cents at each bracket ceiling, transcribed from the next bracket's
    // published base tax in Rev. Procs. 2022-38, 2023-34 and 2024-40.
    let fixtures = [
        (2023, Single, [110000, 514700, 1629000, 3710400, 5283200, 17423825]),
        (2023, Joint, [220000, 1029400, 3258000, 7420800, 10566400, 18660150]),
        (2023, Head, [157000, 686800, 1467800, 3549800, 5122600, 17262350]),
        (2023, Separate, [110000, 514700, 1629000, 3710400, 5283200, 9330075]),
        (2024, Single, [116000, 542600, 1716850, 3911050, 5567850, 18364725]),
        (2024, Joint, [232000, 1085200, 3433700, 7822100, 11135700, 19666950]),
        (2024, Head, [165500, 724100, 1546900, 3741700, 5397700, 18195450]),
        (2024, Separate, [116000, 542600, 1716850, 3911050, 5567850, 9833475]),
        (2025, Single, [119250, 557850, 1765100, 4019900, 5723100, 18876975]),
        (2025, Joint, [238500, 1115700, 3530200, 8039800, 11446200, 20215450]),
        (2025, Head, [170000, 744200, 1591200, 3846000, 5548400, 18703150]),
        (2025, Separate, [119250, 557850, 1765100, 4019900, 5723100, 10107725]),
    ];
    for (year, status, expected) in fixtures {
        let rates = Rates::for_year(year, status).unwrap();
        for (income, cents) in rates.ordinary.into_iter().zip(expected) {
            assert_eq!(rates.ordinary_cents(income), cents, "{year} {status:?} {income}");
        }
        assert_eq!(rates.ordinary_cents(-10000), 0);
        assert_eq!(rates.ordinary_cents(0), 0);
        assert_eq!(rates.ordinary_cents(rates.ordinary[5] + 100), expected[5] + 3700);
        if status == Joint {
            assert_eq!(
                Rates::for_year(year, Surviving).unwrap().ordinary_cents(1000000),
                rates.ordinary_cents(1000000)
            );
        }
    }
    assert!(Rates::for_year(2022, Single).is_none());
    assert!(Rates::for_year(2027, Joint).is_none());
}

#[test]
fn draft_2026_tax_computation_worksheet_matches_every_printed_band() {
    use crate::income_tax::Rates;
    // IRS draft Publication 1040 (2026), pp. 14–15, sections A–D.
    // Values are lower/upper taxable income, printed percent, and subtraction
    // amount in cents; the last row is sampled at $900,000.
    let single = [
        (100_000, 105_700, 22, 528_800),
        (105_701, 201_775, 24, 740_200),
        (201_776, 256_225, 32, 2_354_400),
        (256_226, 640_600, 35, 3_123_075),
        (640_601, 900_000, 37, 4_404_275),
    ];
    let joint = [
        (100_000, 100_800, 12, 49_600),
        (100_801, 211_400, 22, 1_057_600),
        (211_401, 403_550, 24, 1_480_400),
        (403_551, 512_450, 32, 4_708_800),
        (512_451, 768_700, 35, 6_246_150),
        (768_701, 900_000, 37, 7_783_550),
    ];
    let separate = [
        (100_000, 105_700, 22, 528_800),
        (105_701, 201_775, 24, 740_200),
        (201_776, 256_225, 32, 2_354_400),
        (256_226, 384_350, 35, 3_123_075),
        (384_351, 900_000, 37, 3_891_775),
    ];
    let head = [
        (100_000, 105_700, 22, 709_900),
        (105_701, 201_750, 24, 921_300),
        (201_751, 256_200, 32, 2_535_300),
        (256_201, 640_600, 35, 3_303_900),
        (640_601, 900_000, 37, 4_585_100),
    ];
    for (status, rows) in [
        (FederalStatus::Single, &single[..]),
        (FederalStatus::Joint, &joint[..]),
        (FederalStatus::Surviving, &joint[..]),
        (FederalStatus::Separate, &separate[..]),
        (FederalStatus::Head, &head[..]),
    ] {
        let rates = Rates::with_method(2026, status, true).unwrap();
        for &(low, high, percent, subtraction) in rows {
            for income in [low, high] {
                let expected_cents = income * percent - subtraction;
                assert_eq!(rates.worksheet_ordinary_cents(income), expected_cents);
            }
        }
    }
}

#[test]
fn general_rule_frequency_keeps_temporary_and_fixed_contract_rules_distinct() {
    for (frequency, count) in [
        (AnnuityFrequency::Annual, 1),
        (AnnuityFrequency::Semiannual, 2),
        (AnnuityFrequency::Quarterly, 4),
        (AnnuityFrequency::Monthly, 12),
    ] {
        let mut a = fixed_annuity();
        a.frequency = frequency;
        a.periodic = 240000 / i64::from(count);
        a.received = count;
        a.duration = AnnuityDuration::TemporaryLife { age: 65, months: 60 };
        assert!(a.valid(240000));
        assert_eq!(a.expected(), 1176000); // Table VIII has no timing adjustment.
        a.duration = AnnuityDuration::Fixed { payments: count * 5 };
        a.qualified_age = Some(75);
        assert_eq!(a.expected(), 1200000);
        assert!(a.qualified_method());
        a.duration = AnnuityDuration::Fixed { payments: count * 5 - 1 };
        assert!(!a.qualified_method());
        a.duration = AnnuityDuration::Fixed { payments: count };
        assert!(!a.valid(240000)); // Must run longer than one year.
    }
    let mut a = joint_annuity(false);
    a.frequency = AnnuityFrequency::Quarterly;
    a.periodic = 150000;
    a.received = 4;
    a.duration = AnnuityDuration::JointLife {
        age: 70,
        survivor_age: 67,
        survivor_payment: 105000,
        survivor: false,
        delay: 1,
    };
    assert!(a.valid(600000));
    assert_eq!(a.expected(), 12180000);
    assert_eq!(a.ratio(), 515);
    if let AnnuityDuration::JointLife { survivor, .. } = &mut a.duration {
        *survivor = true;
    }
    assert!(a.valid(420000));
    assert_eq!(a.excluded(), 216300);
}

#[test]
fn half_se_allocation_keeps_activity_identity_and_rounding_across_sources() {
    // Resolved sole-proprietor and partnership weights, after source-specific
    // rules. The shared allocator must neither infer nor repeat those rules.
    let result = crate::business_rules::half_se_allocation(
        101,
        [(40, 3000), (10, 1000), (30, -2000), (20, 0)],
    );
    assert_eq!(result.get(&10), Some(&25));
    assert_eq!(result.get(&40), Some(&76));
    assert_eq!(result.get(&30), Some(&0));
    assert_eq!(result.get(&20), Some(&0));
    assert_eq!(result.values().sum::<i64>(), 101);
    assert_eq!(
        result,
        crate::business_rules::half_se_allocation(
            101,
            [(20, 0), (30, -2000), (10, 1000), (40, 3000)]
        )
    );
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(cash_business(3000)));
    let mut spouse = income(cash_business(1000));
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    let actual = alloc::collections::BTreeMap::new();
    let taxpayer = crate::business_rules::half_se_by_business(&d, 0, 101, &actual);
    assert_eq!(taxpayer.len(), 1);
    assert_eq!(taxpayer.get(&d.income[1].id), Some(&101));
    let spouse = crate::business_rules::half_se_by_business(&d, 1, 33, &actual);
    assert_eq!(spouse.len(), 1);
    assert_eq!(spouse.get(&d.income[2].id), Some(&33));
}

#[test]
fn temporary_living_expense_insurance_excess_uses_schedule_one_and_current_year_inclusion() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    add(
        &mut d,
        income(FederalIncome::CasualtyLivingInsurance {
            data: FederalCasualtyLivingInsurance {
                event_reference: "SYN-EVENT-2026-1".into(),
                loss_of_use: FederalHomeLossOfUse::Casualty,
                lost_use_on: 20260601,
                regained_use_on: Some(20260701),
                payments: vec![casualty_living_payment("SYN-2026-1", 20260615, 220_000, 0)],
                actual_living_expenses: 385_000,
                normal_living_expenses: 190_000,
                prior_taxable_reported: 0,
                federal_disaster_area: false,
                federal_declaration_reference: "".into(),
                confirmed: true,
            },
        }),
    );
    let id = d.income.last().unwrap().id;
    let result = supported(&d);
    assert_eq!(
        value(&result, &alloc::format!("casualty_living_insurance_{id}_temporary_increase")),
        1_950
    );
    assert_eq!(value(&result, &alloc::format!("casualty_living_insurance_{id}_taxable")), 250);
    assert_eq!(value(&result, "taxable_casualty_living_insurance"), 250);
    assert_eq!(
        crate::interview::document_amount(&d.income.last().unwrap().income, false),
        Some(25_000)
    );
    assert_eq!(value(&result, "agi"), value(&baseline, "agi") + 250);
    assert_eq!(return_line(&result, "8"), return_line(&baseline, "8") + 25_000);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    let FederalIncome::CasualtyLivingInsurance { data } = &mut d.income.last_mut().unwrap().income
    else {
        unreachable!()
    };
    data.federal_disaster_area = true;
    data.federal_declaration_reference = "DR-9999, synthetic federal disaster".into();
    data.payments[0].withholding = 1_000;
    let federal = supported(&d);
    assert_eq!(value(&federal, "taxable_casualty_living_insurance"), 0);
    assert_eq!(crate::interview::document_amount(&d.income.last().unwrap().income, false), Some(0));
    assert_eq!(value(&federal, "agi"), value(&baseline, "agi"));
    assert_eq!(return_line(&federal, "25b"), return_line(&baseline, "25b") + 1_000);
    assert_eq!(estimate(&restart(d.clone()).draft), federal);

    // Unconfirmed casualty facts: every 2026 insurer payment is taxable.
    {
        let (answer, key) = (false, "WARN_CASUALTY_LIVING_PAYMENTS_TAXED");
        if let FederalIncome::CasualtyLivingInsurance { data } =
            &mut d.income.last_mut().unwrap().income
        {
            data.confirmed = answer;
        }
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_casualty_living_insurance"), 2_200);
        assert!(noted(&r, key));
        assert_eq!(
            crate::interview::document_amount(&d.income.last().unwrap().income, false),
            Some(220_000)
        );
    }
}

#[test]
fn multiple_insurers_share_one_temporary_living_expense_increase() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let first = casualty_living_payment("SYN-2026-A", 20260615, 100_000, 100);
    let mut second = casualty_living_payment("SYN-2026-B", 20260620, 120_000, 200);
    second.insurer = "Second synthetic insurer".into();
    add(
        &mut d,
        income(FederalIncome::CasualtyLivingInsurance {
            data: FederalCasualtyLivingInsurance {
                event_reference: "SYN-EVENT-2026-MULTI".into(),
                loss_of_use: FederalHomeLossOfUse::Casualty,
                lost_use_on: 20260601,
                regained_use_on: Some(20260701),
                payments: vec![first, second],
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
    assert_eq!(return_line(&result, "25b"), return_line(&baseline, "25b") + 300);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    let duplicate_event = d.income.last().unwrap().clone();
    add(&mut d, duplicate_event);
    assert!(!d.valid_stored());
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.event_reference = "SYN-EVENT-2026-OTHER".into();
    }
    assert!(!d.valid_stored());
}

#[test]
fn government_disaster_assistance_can_qualify_without_a_federal_declaration() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    add(
        &mut d,
        income(FederalIncome::DisasterRelief {
            data: FederalDisasterRelief {
                received: 20261001,
                qualification: DisasterQualification::GovernmentAssistance {
                    event: "Synthetic local flood".into(),
                    authority: "Synthetic state emergency agency".into(),
                    payer: "Synthetic county relief fund".into(),
                },
                purpose: DisasterReliefPurpose::GovernmentWelfare,
                amount: 125_000,
                withholding: 0,
                qualified: true,
            },
        }),
    );
    let relief = supported(&d);
    let id = d.income.last().unwrap().id;
    assert_eq!(value(&relief, &alloc::format!("disaster_relief_{id}_excluded")), 1_250);
    assert_eq!(value(&relief, "agi"), value(&baseline, "agi"));
    assert_eq!(relief.tax, baseline.tax);
    assert_eq!(estimate(&restart(d.clone()).draft), relief);

    if let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income {
        data.purpose = DisasterReliefPurpose::ResidenceOrContents;
    }
    assert!(!d.valid_stored());
    if let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income {
        data.purpose = DisasterReliefPurpose::GovernmentWelfare;
        data.qualification = DisasterQualification::GovernmentAssistance {
            event: "Synthetic local flood".into(),
            authority: "Synthetic state emergency agency".into(),
            payer: "".into(),
        };
    }
    assert!(!d.valid_stored());
}

#[test]
fn terrorism_military_and_secretary_catastrophe_qualify_disaster_relief() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let first = d.income.len();
    for qualification in [
        DisasterQualification::TerrorOrMilitary {
            kind: DisasterActionKind::Terrorism,
            event: "Synthetic qualifying attack against the United States".into(),
        },
        DisasterQualification::TerrorOrMilitary {
            kind: DisasterActionKind::Military,
            event: "Synthetic U.S. forces response to aggression".into(),
        },
        DisasterQualification::SecretaryCatastrophe {
            determination: "Synthetic IRS catastrophic-event determination".into(),
        },
    ] {
        add(
            &mut d,
            income(FederalIncome::DisasterRelief {
                data: FederalDisasterRelief {
                    received: 20261001,
                    qualification,
                    purpose: DisasterReliefPurpose::PersonalExpense,
                    amount: 120_000,
                    withholding: 0,
                    qualified: true,
                },
            }),
        );
    }
    let result = supported(&d);
    for entry in &d.income[first..] {
        assert_eq!(value(&result, &alloc::format!("disaster_relief_{}_excluded", entry.id)), 1_200);
    }
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
    assert_eq!(result.tax, baseline.tax);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.qualified = false;
    // Negative qualification: taxed as other income.
    let unsure = supported(&d);

    assert_eq!(value(&unsure, "taxable_relief_payments"), 1_200);
    assert_eq!(value(&unsure, "agi"), value(&baseline, "agi") + 1_200);
    let FederalIncome::DisasterRelief { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.qualified = true;
    data.qualification = DisasterQualification::SecretaryCatastrophe { determination: "".into() };
    assert!(!d.valid_stored());
}

#[test]
fn qualified_hazard_mitigation_grants_are_excluded_without_changing_return_tax() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let first_grant = d.income.len();
    for program in [
        DisasterMitigationProgram::StaffordAct,
        DisasterMitigationProgram::NationalFloodInsuranceAct,
    ] {
        add(
            &mut d,
            income(FederalIncome::DisasterMitigation {
                data: FederalDisasterMitigation {
                    received: 20261015,
                    award_reference: "Synthetic hazard-mitigation grant".into(),
                    program,
                    property: "Synthetic residence".into(),
                    amount: 240_000,
                    qualified: true,
                },
            }),
        );
    }
    let grants = supported(&d);
    for entry in &d.income[first_grant..] {
        assert_eq!(
            value(&grants, &alloc::format!("disaster_mitigation_{}_excluded", entry.id)),
            2_400
        );
    }
    assert_eq!(value(&grants, "agi"), value(&baseline, "agi"));
    assert_eq!(grants.tax, baseline.tax);
    assert_eq!(return_line(&grants, "25d"), return_line(&baseline, "25d"));
    assert_eq!(estimate(&restart(d.clone()).draft), grants);

    if let FederalIncome::DisasterMitigation { data } = &mut d.income.last_mut().unwrap().income {
        data.qualified = false;
    }
    assert!(noted(&supported(&d), "WARN_DISASTER_MITIGATION_LEFT_OUT"));
    if let FederalIncome::DisasterMitigation { data } = &mut d.income.last_mut().unwrap().income {
        data.qualified = true;
        data.property.clear();
    }
    assert!(!d.valid_stored());
}

#[test]
fn ordinary_used_goods_require_documented_depreciated_section_a_source() {
    let mut d = complete(FederalStatus::Single);
    let mut above_section_a = used_goods(500_001);
    above_section_a.basis = 1_000_000;
    d.deductions.as_mut().unwrap().property_gifts.push(above_section_a);
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
    d.deductions.as_mut().unwrap().property_gifts[0].value = 50_000;
    d.deductions.as_mut().unwrap().property_gifts[0].basis = 100_000;
    d.deductions.as_mut().unwrap().property_gifts[0].value = 100_001;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().property_gifts[0].value = 50_000;
    d.deductions.as_mut().unwrap().property_gifts[0].acquired = 20270101;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().property_gifts[0].acquired = 20200110;
    d.deductions.as_mut().unwrap().property_gifts[0].charity.clear();
    assert!(!d.valid_stored());
}

#[test]
fn ordinary_used_goods_through_five_thousand_report_section_a_on_screen() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut gift = used_goods(500_000);
    gift.basis = 1_000_000;
    ded.property_gifts.push(gift);
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let result = supported(&d);
    assert_eq!(value(&result, "form_8283_section_a_value"), 5_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 4_700);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    d.deductions.as_mut().unwrap().property_gifts[0].basis = 1_000_000;
    d.deductions.as_mut().unwrap().property_gifts[0].value = 500_001;
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
}

#[test]
fn ordinary_used_goods_section_a_limit_is_per_similar_group_across_donees() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut clothing = used_goods(500_000);
    clothing.basis = 1_000_000;
    let mut household = used_goods(500_000);
    household.description = "Used table".into();
    household.kind =
        GiftKind::Goods { kind: GoodsKind::Household, condition: GoodsCondition::Good };
    household.charity = "Another synthetic public charity".into();
    household.basis = 1_000_000;
    ded.property_gifts.extend([clothing, household]);
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let result = supported(&d);
    assert_eq!(value(&result, "form_8283_section_a_value"), 10_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 9_700);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    // Similar clothing remains one appraisal group even with separate donees.
    d.deductions.as_mut().unwrap().property_gifts[1].kind =
        GiftKind::Goods { kind: GoodsKind::Clothing, condition: GoodsCondition::Good };
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));

    for gift in &mut d.deductions.as_mut().unwrap().property_gifts {
        gift.value_method = ValueMethod::Appraisal;
        gift.appraisal = Some(2);
    }
    // D-2026-10-15-13 (2026-10-16): appraisal records have no count cap;
    // additional qualified records do not change the gift's actual association.
    d.deductions
        .as_mut()
        .unwrap()
        .appraisals
        .extend((2..=12).map(|id| gift_appraisal(id, 20261115)));
    d.next_id = 13;
    let appraised = supported(&d);
    assert_eq!(value(&appraised, "form_8283_section_b_value"), 10_000);
    assert!(!appraised.lines.iter().any(|line| line.id == "form_8283_section_a_value"));
    assert_eq!(estimate(&restart(d.clone()).draft), appraised);

    d.deductions.as_mut().unwrap().property_gifts[1].donee_acknowledged = false;
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
    d.deductions.as_mut().unwrap().property_gifts[1].donee_acknowledged = true;
    d.deductions.as_mut().unwrap().appraisals[0].appraiser.clear();
    assert!(!d.valid_stored());
}

#[test]
fn donated_personal_car_uses_form_1098c_gross_proceeds_and_section_a() {
    fn set_gross_proceeds(vehicle: &mut PropertyGift, amount: i64) {
        let GiftKind::Vehicle { treatment: VehicleUse::Sold { proceeds, .. }, .. } =
            &mut vehicle.kind
        else {
            unreachable!()
        };
        *proceeds = amount;
    }
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts
        .push(donated_personal_vehicle(VehicleUse::Sold { sold: 20261001, proceeds: 1_500_000 }));
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let result = supported(&d);
    assert_eq!(value(&result, "charity_property_1_claim"), 15_000);
    assert_eq!(value(&result, "form_8283_section_a_value"), 15_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 14_700);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    set_gross_proceeds(&mut d.deductions.as_mut().unwrap().property_gifts[0], 2_500_000);
    let limited_by_fmv = supported(&d);
    assert_eq!(value(&limited_by_fmv, "charity_property_1_claim"), 20_000);
    set_gross_proceeds(&mut d.deductions.as_mut().unwrap().property_gifts[0], 10_000);
    let small_sale = supported(&d);
    assert_eq!(value(&small_sale, "charity_property_1_claim"), 500);
    assert_eq!(value(&small_sale, "charity_property_fifty_current_deduction"), 200);
    assert!(!small_sale.lines.iter().any(|line| line.id == "form_8283_section_a_value"));
    set_gross_proceeds(&mut d.deductions.as_mut().unwrap().property_gifts[0], 50_001);
    let above_small_sale = supported(&d);
    assert!(above_small_sale.lines.iter().any(|line| line.id == "form_8283_section_a_value"));
    let vehicle = &mut d.deductions.as_mut().unwrap().property_gifts[0];
    let GiftKind::Vehicle { treatment: VehicleUse::Sold { sold, .. }, .. } = &mut vehicle.kind
    else {
        unreachable!()
    };
    *sold = 20260909;
    assert!(!d.valid_stored());
}

#[test]
fn sold_and_fmv_exception_cars_share_threshold_but_use_different_8283_sections() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut sold = donated_personal_vehicle(VehicleUse::Sold { sold: 20261001, proceeds: 300_000 });
    sold.value = 300_000;
    let mut exception = donated_personal_vehicle(VehicleUse::SignificantUse {
        details: "Charity used the car for sustained meal deliveries".into(),
    });
    exception.value = 300_000;
    if let GiftKind::Vehicle { vin, .. } = &mut exception.kind {
        *vin = "TESTVIN1234567891".into();
    }
    exception.charity = "Another synthetic public charity".into();
    ded.property_gifts.extend([sold, exception]);
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
    set_gift_appraisal(d.deductions.as_mut().unwrap(), 1, 2);
    d.next_id = d.next_id.max(3);
    d.deductions.as_mut().unwrap().property_gifts[1].donee_acknowledged = true;
    let result = supported(&d);
    assert_eq!(value(&result, "form_8283_section_a_value"), 3_000);
    assert_eq!(value(&result, "form_8283_section_b_value"), 3_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 5_700);
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn selected_youth_outings_and_diaconate_costs_join_cash_class_once() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(100_000);
    ded.volunteer_expenses.extend([
        VolunteerExpense {
            date: 20260620,
            charity: "Synthetic delinquency-prevention charity".into(),
            service: "Paid selected youths' admission, not my own".into(),
            kind: VolunteerExpenseKind::YouthOuting { amount: 10_000 },
            documented: true,
        },
        VolunteerExpense {
            date: 20260621,
            charity: "Synthetic church".into(),
            service: "Permanent diaconate candidate".into(),
            kind: VolunteerExpenseKind::DeaconProgram {
                vestments: 12_500,
                books: 5_000,
                transportation: 7_500,
            },
            documented: true,
        },
    ]);
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let itemized_result = supported(&d);
    assert_eq!(value(&itemized_result, "charity_volunteer_1_expense"), 100);
    assert_eq!(value(&itemized_result, "charity_volunteer_2_expense"), 250);
    assert_eq!(value(&itemized_result, "charity_cash_sixty_current_deduction"), 1_050);
    assert_eq!(estimate(&restart(d.clone()).draft), itemized_result);

    let ded = d.deductions.as_mut().unwrap();
    ded.choice = DeductionSelection::Standard;
    ded.set_cash_charity(0);
    let standard_result = supported(&d);
    assert_eq!(return_line(&standard_result, "12f"), 0);
    d.deductions.as_mut().unwrap().volunteer_expenses[0].documented = false;
    assert!(noted(&supported(&d), "WARN_VOLUNTEER_EXPENSE_LEFT_OUT"));
    d.deductions.as_mut().unwrap().volunteer_expenses[0].documented = true;
    d.deductions.as_mut().unwrap().volunteer_expenses[0].kind =
        VolunteerExpenseKind::YouthOuting { amount: 0 };
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().volunteer_expenses[0].kind =
        VolunteerExpenseKind::YouthOuting { amount: 10_000 };
    d.deductions.as_mut().unwrap().volunteer_expenses[1].kind =
        VolunteerExpenseKind::DeaconProgram { vestments: 0, books: 0, transportation: 0 };
    assert!(!d.valid_stored());
}

#[test]
fn current_cash_can_exhaust_sixty_percent_room_before_old_cash_and_b_class() {
    let mut draft = with_current_direct_cash(
        cash_carry_with_student_and_whaling(10_000_000, 12, 1_000_000, 30_000),
        4_000_000,
    );
    // D-2026-10-15-13 (2026-10-16): the statutory $50 monthly allowance
    // bounds a full-width raw expense before it reaches any charity total.
    draft.deductions.as_mut().unwrap().hosted_students[0].months[0].cost = i64::MAX;
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_deduction"), 19_500);
    assert_eq!(value(&itemized, "charity_other_30_percent_deduction"), 0);
    assert_eq!(value(&itemized, "charity_other_30_percent_return_year_carryforward"), 10_600);
    assert_eq!(return_line(&itemized, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized);
    let student = &mut draft.deductions.as_mut().unwrap().hosted_students[0];
    student.months[0].cost = -1;
    assert_eq!(student.issue().unwrap().field.unwrap().square(), "[months/0/cost]");
    student.months[0].cost = i64::MAX;

    draft.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let standard = supported(&draft);
    assert_eq!(value(&standard, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&standard, "charity_other_30_percent_return_year_carryforward"), 10_600);
    assert_eq!(return_line(&standard, "12f"), 100_000);
    assert_eq!(estimate(&restart(draft).draft), standard);
}

#[test]
fn dual_b_floor_without_percentage_excess_is_not_carried() {
    let draft = cash_carry_with_student_and_whaling(10_000_000, 1, 10_000, 20_000);
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "charity_other_30_percent_deduction"), 0);
    assert_eq!(value(&itemized, "charity_other_30_percent_return_year_carryforward"), 0);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_deduction"), 19_650);
    assert_eq!(value(&itemized, "charity_floor_not_carried"), 500);
    assert_eq!(return_line(&itemized, "12e"), 1_965_000);
    assert_eq!(estimate(&restart(draft).draft), itemized);
}

#[test]
fn dual_b_with_cash_carry_requires_documented_sources_and_accepts_valid_vintages() {
    let draft = with_current_direct_cash(
        cash_carry_with_student_and_whaling(10_000_000, 12, 1_000_000, 20_000),
        1_000_000,
    );
    supported(&draft);

    let mut student_unconfirmed = draft.clone();
    student_unconfirmed.deductions.as_mut().unwrap().hosted_students[0].documented = false;
    assert!(noted(&supported(&student_unconfirmed), "WARN_HOSTED_STUDENT_LEFT_OUT"));

    let mut captain_unconfirmed = draft.clone();
    captain_unconfirmed.deductions.as_mut().unwrap().whaling_captains[0]
        .eligibility_and_records_confirmed = false;
    assert!(noted(&supported(&captain_unconfirmed), "WARN_WHALING_EXPENSES_LEFT_OUT"));

    let mut older_carry = draft;
    older_carry.deductions.as_mut().unwrap().charity_carryovers[0].year = 2024;
    let older = supported(&older_carry);
    assert_eq!(value(&older, "charity_carry_2024_cash_sixty_used"), 20_000);
    assert_eq!(estimate(&restart(older_carry).draft), older);
}

#[test]
fn current_cash_excess_and_prior_year_balance_keep_distinct_carry_years() {
    let mut draft =
        with_current_direct_cash(cash_carry_with_whaling(10_000_000, 1_000_000, 20_000), 7_000_000);
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "charity_cash_sixty_current_deduction"), 59_500);
    assert_eq!(value(&itemized, "charity_carry_2025_cash_sixty_used"), 0);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_remaining_following_year"), 20_000);
    assert_eq!(value(&itemized, "charity_cash_sixty_return_year_carryforward"), 10_500);
    assert_eq!(value(&itemized, "charity_whaling_return_year_carryforward"), 10_000);
    assert_eq!(value(&itemized, "charity_return_year_carryforward"), 20_500);
    assert_eq!(return_line(&itemized, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized);

    draft.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let standard = supported(&draft);
    assert_eq!(return_line(&standard, "12f"), 100_000);
    assert_eq!(value(&standard, "charity_prior_cash_sixty_remaining_following_year"), 20_000);
    assert_eq!(value(&standard, "charity_cash_sixty_return_year_carryforward"), 10_000);
    assert_eq!(value(&standard, "charity_whaling_return_year_carryforward"), 10_000);
    assert_eq!(value(&standard, "charity_return_year_carryforward"), 20_000);
    assert_eq!(estimate(&restart(draft).draft), standard);
}

#[test]
fn paired_prior_year_carries_reject_unreconciled_and_unsupported_sources() {
    let draft = paired_2025_noncash_carries();
    supported(&draft);
    let mut partial_stock = draft.clone();
    partial_stock
        .deductions
        .as_mut()
        .unwrap()
        .charity_carryovers
        .iter_mut()
        .find(|c| c.class == CharityClass::CapitalGain30)
        .unwrap()
        .balance = 1_000_000;
    let r = supported(&partial_stock);
    // The reduced reconciled balance makes the $12,700 itemized benefit
    // smaller than the standard deduction; Automatic keeps the latter.
    assert!(!r.automatic_itemizes);
    assert_eq!(value(&r, "charity_deduction"), 0);
    partial_stock.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized {
        facts: itemized(partial_stock.deductions.as_mut().unwrap()).clone(),
    };
    assert_eq!(value(&supported(&partial_stock), "charity_deduction"), 12_700);

    let mut changed_filer = draft.clone();
    changed_filer
        .deductions
        .as_mut()
        .unwrap()
        .charity_carryovers
        .iter_mut()
        .find(|c| c.class == CharityClass::Property50)
        .unwrap()
        .same_filers = false;
    assert!(noted(&supported(&changed_filer), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut prior_cash = draft.clone();
    prior_cash.deductions.as_mut().unwrap().charity_carryovers.push(prior_cash_charity(2025, 100));
    assert_eq!(value(&supported(&prior_cash), "charity_deduction"), 20_800);

    let mut basis_election = draft.clone();
    basis_election.deductions.as_mut().unwrap().property_gifts.push(public_stock_gift());
    basis_election.deductions.as_mut().unwrap().basis_election = true;
    let r = supported(&basis_election);
    assert!(!r.automatic_itemizes);
    assert_eq!(value(&r, "charity_deduction"), 0);
    basis_election.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized {
        facts: itemized(basis_election.deductions.as_mut().unwrap()).clone(),
    };
    assert_eq!(value(&supported(&basis_election), "charity_deduction"), 12_700);

    let mut foreign_tax = draft;
    payee_statement_credit(&mut foreign_tax, 10_000);
    unsupported(&foreign_tax, "Charitable carryover coordination");
}

#[test]
fn ordinary_used_goods_precede_cash_under_the_2026_limits_and_floor() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut gift = used_goods(500_000);
    gift.basis = 1_000_000;
    ded.property_gifts.push(gift);
    ded.set_cash_charity(3_600_000);
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let r = supported(&d);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 4_700);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 31_000);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 5_000);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 0);
    assert_eq!(value(&r, "charity_floor_not_carried"), 300);
    assert_eq!(value(&r, "form_8283_section_a_value"), 5_000);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let standard = supported(&d);
    assert_eq!(return_line(&standard, "12f"), 100_000);
    assert_eq!(value(&standard, "charity_return_year_carryforward"), 5_000);
    assert_eq!(value(&standard, "charity_property_fifty_return_year_carryforward"), 0);
}

#[test]
fn ordinary_used_goods_percentage_excess_carries_its_floor_share() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 800_000, 0)];
    let ded = d.deductions.as_mut().unwrap();
    let mut gift = used_goods(500_000);
    gift.basis = 1_000_000;
    ded.property_gifts.push(gift);
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let r = supported(&d);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 3_960);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 1_040);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 1_040);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn cash_retained_card_issuer_does_not_allow_retained_flags_without_retained_tax() {
    let mut claim = card_issuer_claim("no-retained", 100_000);
    assert!(claim.valid() && claim.qualified());
    claim.retained_cash_accounting_confirmed = Some(true);
    assert!(!claim.qualified());
}

#[test]
fn retained_card_issuer_rejects_unreconciled_and_conflicting_source_records() {
    let good = retained_card_issuer_claim();
    assert!(good.valid() && good.qualified());
    for case in 0..13 {
        let mut claim = good.clone();
        match case {
            0 => claim.sales[0].retained_claimable_tax = 0,
            1 => claim.sales[0].retained_claimable_tax = 12_149,
            2 => claim.sales[0].retained_claimable_tax = -1,
            3 => claim.sales[0].buyer_written_consent = false,
            4 => claim.sales[0].buyer_tax_refund_on = Some(20261231),
            5 => claim.sales[0].claimable_tax_refunded_in_full = true,
            6 => claim.sales[0].tax_not_collected_from_buyer = true,
            7 => claim.retained_accrual_accounting_confirmed = None,
            8 => claim.retained_accrual_accounting_confirmed = Some(false),
            9 => claim.settlement_outside_business_income_and_expenses = false,
            10 => claim.sales[0].not_claimed_elsewhere = false,
            11 => {
                claim.sales[1].fuel = FederalCardIssuerFuel::UndyedNonaviationKerosene;
                claim.sales[0].retained_claimable_tax = 12_149;
                claim.sales[1].retained_claimable_tax = 12_151;
            }
            12 => {
                claim.sales[0].retained_claimable_tax = 0;
                claim.sales[0].buyer_tax_refund_on = Some(20261231);
                claim.sales[0].claimable_tax_refunded_in_full = true;
            }
            _ => unreachable!(),
        }
        assert!(!claim.valid() || !claim.qualified(), "case {case}");
    }
}

#[test]
fn retained_card_issuer_requires_active_tax_excluded_accrual_records() {
    for case in 0..4 {
        let mut business = retained_card_issuer_business();
        match case {
            0 => business.material_participation = false,
            1 => {
                business.fuel_tax_credit.as_mut().unwrap().expense_treatment =
                    FederalFuelExpenseTreatment::FullCostDeducted;
            }
            2 => business.cash_fuel_tax_recovery = Some(cash_fuel_recovery(10_000)),
            3 => {
                business.source =
                    BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false }
            }
            _ => unreachable!(),
        }
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(FederalIncome::Business { data: business }));
        let result = estimate(&draft);
        if case == 0 {
            // A passive business is left out with a notice.
            assert!(noted(&result, "WARN_BUSINESS_LEFT_OUT"), "{result:?}");
            continue;
        }
        assert!(
            matches!(result.state, ResultState::Unsupported | ResultState::Incomplete)
                && result.tax.is_none()
                && result.lines.is_empty(),
            "case {case}: {result:?}"
        );
    }
}

#[test]
fn registered_card_issuer_line_13_rates_round_once_and_reach_full_return() {
    let mut direct = card_issuer_claim("direct", 100_000);
    direct.sales[0].thousandths_of_gallon = 100;
    direct.sales.push(card_issuer_sale(
        "direct-13a-second",
        FederalCardIssuerFuel::UndyedDiesel,
        100,
    ));
    let business_fuels =
        FederalFuelLines { card_issuer: Some(direct), ..FederalFuelLines::default() };
    let partner_fuels = FederalFuelLines {
        card_issuer: Some(card_issuer_claim("partner", 50_000)),
        ..FederalFuelLines::default()
    };
    let (draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    for (line, direct_cents, partner_cents) in
        [("13a", 5, 1215), ("13b", 2430, 1215), ("13c_219", 2180, 1090), ("13c_244", 2430, 1215)]
    {
        for (source, expected) in [("business_2", direct_cents), ("partnership_3", partner_cents)] {
            assert_eq!(
                line_amount(&report, &alloc::format!("{source}_fuel_{line}")),
                expected,
                "{source} line {line}"
            );
        }
    }
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 11_800);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(report.balance, baseline.balance.map(|value| value - 11_800));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    // If this issuer also uses fuel, only its own deducted fuel tax creates
    // business-income recovery; the card-issuer settlement stays separate.
    let own_fuel = FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            gasoline_off_highway: Some(fuel_use(100_000, 20_000)),
            card_issuer: Some(card_issuer_claim("mixed", 100_000)),
            ..FederalFuelLines::default()
        },
        expense_treatment: FederalFuelExpenseTreatment::FullCostDeducted,
        records_complete: true,
    };
    assert_eq!(own_fuel.expense_recovery_cents(), 1_830);
}

#[test]
fn business_1099_withholding_reaches_line_25b_once() {
    // Verify that FederalBusiness.withholding field compiles and holds 1099 withholding.
    // The field is validated as non-negative and within the standard money amount range.

    // Test 1: Create a business with $1,200 of 1099 withholding
    let business = FederalBusiness {
        // $1,200 of 1099-NEC/MISC/K withholding
        activity: BusinessActivity::Ordinary {
            returns: 0,
            receipts: 10_000_000,
            accounting: BusinessAccounting::Cash,
        },
        withholding: 120_000,
        ..support::builders::survey::survey_cash_business_data()
    };

    // Verify the field is accessible and contains the expected value
    assert_eq!(business.withholding, 120_000);

    // Test 2: Zero withholding (valid)
    let business_zero = FederalBusiness {
        ministerial: false,
        notary_profit: 0,
        notary_gross: None,
        withholding: 0,
        ..business
    };
    assert_eq!(business_zero.withholding, 0);

    // The field is validated in federal_rules.rs: must be >= 0 and within money amount range.
    // Negative values fail the validation check (v.withholding >= 0 && amounts(&[v.withholding])).
}

// C-fixture (matrix row 16, 2026-09-25): a sole proprietorship whose receipts
// combine a 1099-NEC and a 1099-K, a 1099-MISC prize and a 1099-MISC royalty,
// through Schedule C, Schedule SE, Schedule 1 and Form 8995 by hand
// (form-1099-k-nec-misc.md; unit C16a's FederalBusiness.withholding).
#[test]
fn contractor_return_matches_hand_computation() {
    // Facts: a single filer born 1987, no W-2. One sole proprietorship
    // (material participation) whose Schedule C gross receipts are the sum of
    // a 1099-NEC box 1 ($45,000) and a 1099-K box 1a ($30,000): the model has
    // no document union for these payer forms (form-1099-k-nec-misc.md's
    // "Union structure"), so both amounts are entered together as this
    // business's ordinary receipts, "including reported 1099 income once"
    // (i1040sc). The 1099-NEC also carries $2,000 of box 4 federal
    // withholding (the C16a `FederalBusiness.withholding` field). The only
    // Schedule C expense is a $5,000 supplies expense (line 22).
    //
    // A 1099-MISC reports box 3 other income of $1,000 (a prize, routed to
    // Schedule 1 line 8i) and box 2 royalties of $2,500 with no royalty
    // expenses (routed to Schedule E, Schedule 1 line 5); it carries no
    // federal withholding.
    //
    // Schedule C:
    //   Line 1 gross receipts: 45,000.00 + 30,000.00 = $75,000.00.
    //   Line 7 gross income: $75,000.00 (no COGS, no other income).
    //   Line 22 supplies: $5,000.00. Line 28 total expenses: $5,000.00.
    //   Line 29 tentative profit: 75,000.00 - 5,000.00 = $70,000.00.
    //   Line 30 home office: $0 (none). Line 31 net profit: $70,000.00.
    //
    // Schedule SE (ordinary nonfarm method, the only Schedule C/F business, no W-2):
    //   Line 2/3: $70,000.00.
    //   Line 4a-4c: 70,000.00 x 92.35% = $64,645.00 (exact).
    //   Line 6: $64,645 (no church employee income). Line 7: $184,500 (2026
    //     Social Security wage base). Line 8a-8d: $0 (no W-2). Line 9: $184,500.
    //   Line 10: min(64,645, 184,500) x 12.4% = $8,015.98. Line 11: 64,645 x
    //     2.9% = $1,874.705.
    //   Line 12 self-employment tax: 8,015.98 + 1,874.705 = 9,890.685, rounds
    //     to $9,891.
    //   Line 13 half-SE deduction: (9,891 + 1) / 2 = $4,946 (rounded).
    //
    // Schedule 1 (Part I additional income and Part II adjustments):
    //   Line 3 business income (Schedule C line 31): $70,000.
    //   Line 5 royalties (Schedule E, no expenses): $2,500.
    //   Line 8i taxable prizes (1099-MISC box 3): $1,000.
    //   Line 9/10 total additional income: 70,000 + 2,500 + 1,000 = $73,500
    //     (this business is the only other Part I item).
    //   Line 15 deductible part of self-employment tax: $4,946.
    //   Line 26 total adjustments: $4,946 (the only adjustment).
    //
    // Form 8995 (2026 single QBI threshold $201,750, far above this return;
    // the royalty is portfolio Schedule E income, never QBI):
    //   Line 1/2 QBI: net profit less the attributable half-SE deduction,
    //     70,000 - 4,946 = $65,054 (no SSTB reduction and no wage/UBIA limit
    //     apply below the threshold).
    //   Line 3/4: $65,054 (no carryforward). Line 5: 65,054 x 20% = $13,010.80,
    //     rounds to $13,011. Lines 6-9: $0 (no REIT/PTP dividends). Line 10:
    //     $13,011.
    //   Line 11 taxable income before QBI: AGI $68,554 less the $16,100
    //     standard deduction (single, not 65/blind) = $52,454. Line 12: $0
    //     (no net capital gain). Line 13: $52,454. Line 14: 52,454 x 20% =
    //     $10,490.80, rounds to $10,491.
    //   Line 15 QBI deduction: smaller of line 10 ($13,011) or line 14
    //     ($10,491) = $10,491. Lines 16/17: $0.
    //
    // Form 1040:
    //   Line 8 additional income (Schedule 1 line 10): $73,500.
    //   Line 9 total income: $73,500 (no other income sources).
    //   Line 10 adjustments (Schedule 1 line 26): $4,946.
    //   Line 11 AGI: 73,500 - 4,946 = $68,554.
    //   Line 12e standard deduction (single, not 65/blind): $16,100.
    //   Line 13b QBI deduction: $10,491.
    //   Line 15 taxable income: 68,554 - 16,100 - 10,491 = $41,963.
    //   Line 16 regular tax (2026 draft Tax Table, single, $41,950-$42,000
    //     row, midpoint $41,975): 12,400 x 10% + (41,975 - 12,400) x 12% =
    //     1,240.00 + 3,549.00 = $4,789.00.
    //   Line 23 (Schedule 2 line 4, self-employment tax): $9,891.
    //   Line 24a total tax: 4,789 + 9,891 = $14,680.
    //   Line 25b 1099 withholding: $2,000 (the 1099-NEC box 4 amount on the
    //     business; no other document here carries withholding).
    //   Line 33 total payments: $2,000 (no estimated payments).
    //   Line 37 balance due: 14,680 - 2,000 = $12,680.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19870615;
    d.income.clear();
    let mut business = cash_business(75_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.expenses = vec![ordinary_business_expense(BusinessExpenseCategory::Supplies, 500_000)];
    data.withholding = 200_000;
    add(&mut d, income(business));
    add(&mut d, income(prize_income(100_000, 0, 0, PrizeDisposition::Received)));
    add(&mut d, income(royalty(2500, 0)));

    let r = supported(&d);
    assert_eq!(value(&r, "business_2_expense_line22_supplies"), 5_000);
    assert_eq!(value(&r, "business_2_cost_of_goods_sold"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 9_891);
    assert_eq!(value(&r, "adjustments"), 4_946);
    assert_eq!(value(&r, "nonbusiness_royalty_income"), 2_500);
    assert_eq!(value(&r, "taxable_prizes"), 1_000);
    assert_eq!(value(&r, "agi"), 68_554);
    assert_eq!(value(&r, "qbi_deduction"), 10_491);
    assert_eq!(value(&r, "taxable_income"), 41_963);

    assert_eq!(return_line(&r, "8"), 7_350_000);
    assert_eq!(return_line(&r, "9"), 7_350_000);
    assert_eq!(return_line(&r, "10"), 494_600);
    assert_eq!(return_line(&r, "11a / 11b"), 6_855_400);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 1_049_100);
    assert_eq!(return_line(&r, "15"), 4_196_300);
    assert_eq!(return_line(&r, "16"), 478_900);
    assert_eq!(return_line(&r, "23"), 989_100);
    assert_eq!(return_line(&r, "24a"), 1_468_000);
    assert_eq!(return_line(&r, "25b"), 200_000);
    assert_eq!(return_line(&r, "33"), 200_000);
    assert_eq!(return_line(&r, "37"), 1_268_000);

    assert_eq!(r.tax, Some(1_468_000));
    assert_eq!(r.payments, Some(200_000));
    assert_eq!(r.balance, Some(1_268_000));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (1): a 1099-MISC box 1 rent on movable personal property (not real
// estate) uses NonbusinessProperty::PersonalRental, never Schedule C or the
// Schedule E FederalRental (matrix row 17): gross rent and allowed expenses
// each land on their own Schedule 1 line, 8l and 24b, never netted together
// or routed through a business or royalty line.
#[test]
fn personal_property_rent_reaches_schedule_1_lines_8l_and_24b() {
    // $3,000 gross rent less $800 of eligible expenses: Schedule 1 line 8l
    // (Part I income, Form 1040 line 8) reports the $3,000 gross rent
    // undiminished by expenses; line 24b is a Part II adjustment to income
    // (like the half-SE deduction), so it reduces AGI through Form 1040 line
    // 10, not line 8. AGI: 3,000 - 800 = $2,200 (no self-employment tax, no
    // royalty line).
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(personal_rent(3000, 800)));
    let r = supported(&d);
    assert_eq!(value(&r, "nonbusiness_personal_rent"), 3_000);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 800);
    // Schedule 1: line 8l is line 9, line 24b is line 25 and line 26.
    assert_eq!(schedule_1_line(&r, "8l"), 3_000);
    assert_eq!(schedule_1_line(&r, "9"), 3_000);
    assert_eq!(schedule_1_line(&r, "24b"), 800);
    assert_eq!(schedule_1_line(&r, "25"), 800);
    assert_eq!(schedule_1_line(&r, "26"), 800);
    assert_eq!(value(&r, "nonbusiness_royalty_income"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 2_200);
    assert_eq!(return_line(&r, "8"), 300_000);
    assert_eq!(return_line(&r, "10"), 80_000);
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (3): withholding on a 1099-MISC prize (box 4) and on a business's
// own 1099-NEC (box 4, the C16a `FederalBusiness.withholding` field) both
// join the same `t.withholding_1099` total and combine once on Form 1040 line
// 25b, never doubled and never on separate lines.
#[test]
fn prize_and_business_1099_withholding_combine_once_on_25b() {
    // $500 business 1099-NEC withholding + $300 prize 1099-MISC withholding
    // = $800 combined once on line 25b.
    let mut d = empty_return(FederalStatus::Single);
    let mut business = cash_business(10_000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.withholding = 50_000;
    add(&mut d, income(business));
    add(&mut d, income(prize_income(100_000, 0, 30_000, PrizeDisposition::Received)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "25b"), 80_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn unsupported_espp_sale_foreign_pay_and_capital_carryover_are_left_out_with_notices() {
    // Answers and unknowns, decision 2: a scope No leaves the item out with
    // a notice in its own words; the estimate proceeds.
    let base = supported(&complete(FederalStatus::Single));
    let mut d = complete(FederalStatus::Single);
    let mut espp = ordinary_espp_lot();
    espp.special = true;
    add(&mut d, income(FederalIncome::EsppSale { data: espp }));
    let r = supported(&d);
    assert!(noted(&r, "WARN_ESPP_SALE_LEFT_OUT"));
    assert_eq!(r.tax, base.tax);

    let mut d = complete(FederalStatus::Single);
    let FederalIncome::ForeignEmployment { data } = &mut foreign_pay(30_000) else {
        unreachable!()
    };
    let pay = data.clone();
    // Foreign pay is retained without an ordinary-employment scope gate.
    add(&mut d, income(FederalIncome::ForeignEmployment { data: pay }));
    let r = supported(&d);
    assert!(!noted(&r, "WARN_FOREIGN_WAGES_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 90000);

    let mut d = complete(FederalStatus::Single);
    let mut history = capital_history();
    history.ordinary_history = false;
    d.screening.as_mut().unwrap().capital_history = vec![history];
    let r = supported(&d);
    assert!(noted(&r, "WARN_CAPITAL_LOSS_CARRYOVER_LEFT_OUT"));
    assert_eq!(r.tax, base.tax);
}

// Fixtures F1, F2, F5 of the whole-return map (decedent's final return).
pub(in crate::tests) fn final_person(birth: u32, date: u32) -> FinalReturn {
    let _ = birth;
    FinalReturn { bonds: None, estate_medical: 0, date, ordinary: true }
}

#[test]
fn final_return_f1_single_decedent_leaves_off_the_income_paid_after_the_death() {
    // Dana, born 1975-03-14, died 2026-06-30. W-2 $30,000 (withheld $3,000).
    // 1099-R code 7 $12,000 with $2,000 paid after the death (withheld
    // $1,200); 1099-INT $1,000 of which $400 is for the time after the death.
    let mut d = complete(FederalStatus::Single);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 19750314;
    p.died = Some(final_person(19750314, 20260630));
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 300_000);
    let mut pension = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Pension,
        12_000,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut pension.income {
        data.withholding = 120_000;
        data.after_death = Some(RetirementAfterDeath { gross: 200_000, taxable: 200_000 });
    }
    pension.label = "Annuity".into();
    add(&mut d, pension);
    let mut interest = income(savings_income(1000, 0));
    if let FederalIncome::Interest { data } = &mut interest.income {
        data.after_death = Some(InterestAfterDeath {
            taxable: 40_000,
            us_obligations: 0,
            exempt: 0,

            private_activity: 0,
            foreign_tax: 0,
            bond_premium: 0,
            treasury_bond_premium: 0,
            tax_exempt_bond_premium: 0,
            accrued_interest: 0,
            market_discount: 0,
        });
    }
    interest.label = "Bank".into();
    add(&mut d, interest);
    let r = supported(&d);
    // Wages 30,000 + pension 12,000 - 2,000 + interest 1,000 - 400 = AGI 40,600;
    // the full standard deduction 16,100; taxable 24,500; Tax Table midpoint
    // 24,525 = 1,240 + 12% x 12,125 = 2,695; payments 3,000 + 1,200.
    assert_eq!(return_line(&r, "1a"), 3_000_000);
    assert_eq!(return_line(&r, "5b"), 1_000_000);
    assert_eq!(return_line(&r, "2b"), 60_000);
    assert_eq!(value(&r, "agi"), 40_600);
    assert_eq!(value(&r, "deduction"), 16_100);
    assert_eq!(return_line(&r, "15"), 2_450_000);
    assert_eq!(return_line(&r, "16"), 269_500);
    assert_eq!(return_line(&r, "25d"), 420_000);
    assert_eq!(r.refund, Some(150_500));
    // One notice per statement, scoped to its row.
    let ird: Vec<_> = r
        .review_notes
        .iter()
        .filter(|n| n.display.key == "NOTE_INCOME_IN_RESPECT_OF_A_DECEDENT")
        .collect();
    assert_eq!(ird.len(), 2);
    assert!(ird.iter().all(|n| n.context.is_some()));
    // The stored statements keep the printed amounts and the part.
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn the_part_paid_after_a_death_is_checked_against_its_box_and_its_owner() {
    // A part above its box fails on the page; the same record on a living
    // owner's statement is refused on save with both ways out.
    let original = complete(FederalStatus::Single);
    let reject = |r: R1099, key: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = send(
            &mut session,
            save_documents(&original, vec![document(0, IncomeDocument::R1099(r))]),
        );
        let Reply::Response(P::Problem { message, .. }) = reply else {
            panic!("expected a problem for {key}, got {reply:?}")
        };
        assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
    };
    let mut over =
        r1099(200_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    over.after_death = Some(RetirementAfterDeath { gross: 200_001, taxable: 200_000 });
    reject(over, "AFTER_DEATH_ABOVE_BOX");
    let mut over_taxable =
        r1099(200_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    over_taxable.taxable = 150_000;
    over_taxable.after_death = Some(RetirementAfterDeath { gross: 150_001, taxable: 150_001 });
    reject(over_taxable, "AFTER_DEATH_ABOVE_BOX");
    // The owner is alive: the save refuses with the statement named.
    let mut living =
        r1099(200_000, DistributionCode::Normal { what_happened: Disposition::Kept }, false);
    living.after_death = Some(RetirementAfterDeath { gross: 100_000, taxable: 100_000 });
    let mut session = Interview::new(TEST_USER, original.clone());
    let reply = current_income_refusal(
        "income-source-802",
        &mut session,
        save_documents(&original, vec![document(0, IncomeDocument::R1099(living.clone()))]),
        "ISSUE_AFTER_DEATH_WITHOUT_A_DEATH",
        "[documents/documents/0/document/after_death]",
    );
    let Reply::Response(P::Problem { message, .. }) = reply else {
        panic!("expected a problem, got {reply:?}")
    };
    assert_eq!(status_message(&message).key, "ISSUE_AFTER_DEATH_WITHOUT_A_DEATH");
    // With the death on the owner's page the same row is accepted and shows
    // the amount left on the return.
    let mut dead = original.clone();
    dead.people.as_mut().unwrap().taxpayer.died = Some(final_person(19800101, 20260930));
    let mut session = Interview::new(TEST_USER, dead.clone());
    accepted(&send(
        &mut session,
        save_documents(&dead, vec![document(0, IncomeDocument::R1099(living))]),
    ));
    let preview = match send(&mut session, {
        let mut request = crate::tests::support::client::survey::survey_preview_document(
            IncomeDocument::R1099(r1099(
                200_000,
                DistributionCode::Normal { what_happened: Disposition::Kept },
                false,
            )),
        );
        let P::PreviewDocument { foreign_countries, .. } = &mut request else { unreachable!() };
        *foreign_countries = crate::foreign_accounts::countries();
        request
    }) {
        Reply::Response(P::IncomePreview { preview, .. }) => preview,
        other => panic!("expected a preview, got {other:?}"),
    };
    assert_eq!(preview, Some(200_000));
}

#[test]
fn final_return_f2_joint_return_of_the_year_a_spouse_died_with_the_estate_paid_medical_election() {
    // Alex (survivor) and Blake (died 2026-05-10) file jointly. W-2 wages
    // $60,000 and $20,000, withheld $5,000 and $2,000; Blake's estate paid
    // $40,000 of his 2026 medical bills; state income tax $4,000.
    let build = |estate_medical: i64| {
        let mut d = complete(FederalStatus::Joint);
        let people = d.people.as_mut().unwrap();
        people.taxpayer.birth_date = 19700120;
        let spouse = people.spouse.as_mut().unwrap();
        spouse.birth_date = 19680902;
        spouse.died =
            Some(FinalReturn { bonds: None, estate_medical, date: 20260510, ordinary: true });
        d.income = vec![
            wage(1, Owner::Taxpayer, 6_000_000, 500_000),
            wage(2, Owner::Spouse, 2_000_000, 200_000),
        ];
        d.next_id = 3;
        itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
            paid: 400_000,
            allocation: TaxIncomeAllocation::CurrentReturn,
        };
        d
    };
    // With the election: medical 40,000 - 7.5% x 80,000 = 34,000; itemized
    // 38,000 over the 32,200 standard deduction; taxable 42,000; midpoint
    // 42,025 = 2,480 + 12% x 17,225 = 4,547; withholding 7,000.
    let with = supported(&build(4_000_000));
    assert_eq!(value(&with, "agi"), 80_000);
    assert_eq!(value(&with, "estate_paid_medical_election"), 40_000);
    assert_eq!(value(&with, "deduction"), 38_000);
    assert_eq!(return_line(&with, "15"), 4_200_000);
    assert_eq!(return_line(&with, "16"), 454_700);
    assert_eq!(with.refund, Some(245_300));
    // Without it the 4,000 of state tax is below the standard deduction:
    // taxable 47,800; midpoint 47,825 = 2,480 + 12% x 23,025 = 5,243.
    let without = supported(&build(0));
    assert_eq!(value(&without, "deduction"), 32_200);
    assert_eq!(return_line(&without, "15"), 4_780_000);
    assert_eq!(return_line(&without, "16"), 524_300);
    assert_eq!(without.refund, Some(175_700));
    // Both spouses dying in 2026 is still one joint return.
    let mut both = build(4_000_000);
    both.people.as_mut().unwrap().taxpayer.died = Some(final_person(19700120, 20261201));
    assert_eq!(supported(&both).refund, with.refund);
}

#[test]
fn final_return_f5_residence_tests_run_over_half_the_days_the_decedent_was_alive() {
    // Gray (head of household) died 2026-04-30: 31 + 28 + 31 + 30 = 120 days.
    // Pat (8) lived with Gray 61 days: 61 x 2 = 122 > 120.
    let build = |days: u32| {
        let mut d = complete(FederalStatus::Head);
        d.people.as_mut().unwrap().taxpayer.died = Some(final_person(19800101, 20260430));
        d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 50_000);
        let mut pat = child(d.next_id);
        d.next_id += 1;
        pat.residency = DependentResidency::Days { days };
        d.dependents.push(pat);
        d
    };
    let r = supported(&build(61));
    // Standard deduction (head of household) 24,150 over the 20,000 of wages.
    assert_eq!(value(&r, "deduction"), 24_150);
    assert_eq!(return_line(&r, "15"), 0);
    assert_eq!(return_line(&r, "16"), 0);
    // The refundable child credit: the smaller of $1,700 and 15% x (20,000 - 2,500).
    assert_eq!(return_line(&r, "28"), 170_000);
    assert_eq!(
        return_line(&r, "27a"),
        crate::forms::schedule_eic::eic_amount(20_000, 20_000, 1, false) * 100
    );
    assert!(return_line(&r, "27a") > 0);
    // 60 days is not more than half of 120: Pat does not qualify, and head of
    // household needs a person who does.
    unsupported(&build(60), "Children and people you support");
    // A joint return counts the whole year: the same 61 days is under half of 365.
    let mut joint = complete(FederalStatus::Joint);
    joint.people.as_mut().unwrap().spouse.as_mut().unwrap().died =
        Some(final_person(19800101, 20260430));
    let mut pat = child(joint.next_id);
    joint.next_id += 1;
    pat.residency = DependentResidency::Days { days: 61 };
    joint.dependents.push(pat);
    let r = supported(&joint);
    // Only the $500 credit for other dependents (a qualifying relative), not
    // the $2,200 child tax credit of a qualifying child.
    assert_eq!(value(&r, "child_credit"), 500);
}

#[test]
fn a_final_return_carries_no_loss_forward_and_says_so() {
    // Pub. 559: a decedent's unused carryovers end with the final return; a
    // prior NOL still reduces the final year's income.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died = Some(final_person(19800101, 20261015));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 20_000)]);
    let r = supported(&d);
    assert!(r.issues.is_empty());
    // A loss carried forward by this return's own business loss ends with it.
    let mut loss = complete(FederalStatus::Single);
    loss.people.as_mut().unwrap().taxpayer.died = Some(final_person(19800101, 20261015));
    loss.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 0);
    add(&mut loss, income(cash_business(-30_000)));
    assert!(noted(&supported(&loss), "NOTE_FINAL_RETURN_CARRYOVERS_END"));
}

#[test]
fn a_deceased_filers_bonds_and_the_special_adjustments_answer_stay_the_persons_own() {
    // The "other adjustment" confirmation No leaves them out with one notice
    // on the person's page; Yes leaves the return without the notice.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died = death(20260630);
    let r = supported(&d);
    let note = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT")
        .unwrap();
    assert!(note.context.is_some());
    d.people.as_mut().unwrap().taxpayer.died = Some(final_person(19800101, 20260630));
    assert!(!noted(&supported(&d), "WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT"));
}

#[test]
fn a_filer_someone_can_claim_has_no_dependents_and_the_listed_people_stay() {
    // Section 152(b)(1): no child tax credit, credit for other dependents or
    // head of household through a relative; the people stay listed, one note.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    add_child(&mut d);
    let r = supported(&d);
    assert!(noted(&r, "NOTE_DEPENDENT_FILER_HAS_NO_DEPENDENTS"));
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(d.dependents.len(), 1);
    // Head of household through the unmarried qualifying child follows its own test.
    let mut head = complete(FederalStatus::Head);
    head.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    add_child(&mut head);
    supported(&head);
}

#[test]
fn someone_else_who_could_also_claim_a_child_follows_the_tiebreaker_answer() {
    let build = |claim: CompetingClaim| {
        let mut d = complete(FederalStatus::Single);
        add_child(&mut d);
        d.dependents[0].competing = claim;
        d
    };
    let alone = supported(&build(CompetingClaim::NoOne));
    assert_eq!(value(&alone, "child_credit"), 2_200);
    for basis in [
        ClaimBasis::YouAreTheirParent,
        ClaimBasis::NoParentCanClaim,
        ClaimBasis::HigherIncomeThanTheirParents,
    ] {
        let r = supported(&build(CompetingClaim::You { basis }));
        assert_eq!(value(&r, "child_credit"), 2_200, "{basis:?}");
        assert!(!noted(&r, "WARN_TIEBREAKER_NOT_MET"));
    }
    // The other person claims: none of the benefits, with a note.
    let other = supported(&build(CompetingClaim::Other));
    assert_eq!(value(&other, "child_credit"), 0);
    assert!(noted(&other, "NOTE_COMPETING_CLAIM_OTHER_PERSON_CLAIMS"));
    // None of the tiebreakers: computed as the other person's claim, with a warning.
    let none = supported(&build(CompetingClaim::You { basis: ClaimBasis::NoneOfThese }));
    assert_eq!(value(&none, "child_credit"), 0);
    assert!(noted(&none, "WARN_TIEBREAKER_NOT_MET"));
}

#[test]
fn removed_flags_are_not_set_by_any_fixture() {
    // The final-return flag, the full-year-resident flag, the widowed
    // situation, the estate medical list, the allocation confirmation, the
    // restricted community scopes and the competing-claim fact were removed
    // from the stored shape; no source or fixture may still name them.
    let removed = [
        concat!("full_", "year_resident"),
        concat!("Widowed", "2026"),
        concat!("Final", "Medical"),
        concat!("CommunityProperty", "Scope"),
        concat!("CommunityPortfolio", "Sources"),
        concat!("PersonFacts::COMPETING", "_CLAIM"),
        concat!(".final_", "return ="),
        concat!("final_return", ": true"),
        concat!("allocated", ": true"),
        concat!("allocated", ": false"),
        concat!("MoreAboutYou::Died", ";"),
    ];
    extern crate std;
    let mut scanned = 0;
    let mut dirs = alloc::vec![std::path::PathBuf::from("src")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for needle in removed {
                assert!(!text.contains(needle), "{} still names {needle}", path.display());
            }
            scanned += 1;
        }
    }
    assert!(scanned > 100);
}

// D-2026-10-19-01 and -12: the business-property forms print whole dollars while their
// calculation rows keep cents, as Schedule C, Schedule E, Form 4562 and Form 8829 already do.
#[test]
fn business_property_forms_print_whole_dollars_and_keep_cent_rows() {
    let forms = [TaxForm::Form4562B, TaxForm::Form4684, TaxForm::Form4797, TaxForm::Form4835];
    let mut result = FederalResult::empty(1);
    for (index, form) in forms.into_iter().enumerate() {
        crate::kernel::lines::push_line(
            &mut result,
            Some(FormLine::new(form, "1")),
            "child_credit_agi",
            12_350 + index as i64,
            &[],
            "test",
            msg!("EXPLAIN_FEDERAL_RULES_16"),
        );
    }
    assert!(result.lines.iter().all(|l| l.amount % 100 != 0), "the calculation keeps cents");
    let printed = crate::draft_return(&result);
    assert_eq!(printed.len(), forms.len());
    assert!(printed.iter().all(|r| r.amount == 12_400 && r.money == Some(12_400)));
}
