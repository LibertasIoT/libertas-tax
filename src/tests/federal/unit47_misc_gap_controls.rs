use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::unit47_payment_credit_admission::{bond_editor, bond_question_editor};
use crate::tests::support::client::credit_documents::answered_origin;
use super::*;
use crate::tests::value as line_value;

fn calculation(request: &P) -> P {
    match request {
        P::SaveIncomes { .. } => credit_estimate_documents_from_save(request),
        P::SaveDocuments { .. } => request.clone(),
        P::SaveMoreCredits { .. } => {
            crate::tests::support::client::credit_documents::more_credits_calculation(request)
        }
        P::SaveOtherItemized {
            value,
            allowed_owners,
            interest_sources,
            investment_k1_sources,
            investment_origin_choices,
            ..
        } => P::EstimateOtherItemized {
            user: TEST_USER,
            value: value.clone(),
            allowed_owners: allowed_owners.clone(),
            interest_sources: interest_sources.clone(),
            investment_k1_sources: investment_k1_sources.clone(),
            investment_origin_choices: investment_origin_choices.clone(),
        },
        P::SaveInjuredSpouse { value, followups, .. } => P::EstimateInjuredSpouse {
            user: TEST_USER,
            value: value.clone(),
            followups: followups.clone(),
            questions: Vec::new(),
        },
        _ => unreachable!(),
    }
}

fn checked(
    name: &str,
    session: &mut Interview,
    bad: P,
    corrected: P,
    topic: TaxSection,
    key: &str,
    path: &str,
) {
    credit_refusal(name, session, bad.clone(), key, path);
    credit_refusal(&alloc::format!("{name}-estimate"), session, calculation(&bad), key, path);
    let before = session.draft.clone();
    let dirty = session.dirty;
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        session,
        calculation(&corrected),
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, P::from_avro(&corrected.to_avro()).unwrap()));
    assert_eq!(session.draft.revision, before.revision + 1);
    let saved = session.draft.clone();
    let result = supported(&saved);
    let Reply::Edit(reopened) = crate::tests::support::client::native_editor(
        from_review(
            session,
            if matches!(corrected, P::SaveDocuments { .. }) {
                TaxSection::Documents
            } else {
                topic
            },
        ),
        &[],
    ) else {
        panic!("{name}: editor required")
    };
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let Reply::Edit(reopened) = crate::tests::support::client::native_editor(
        from_review(
            &mut cold,
            if matches!(corrected, P::SaveDocuments { .. }) {
                TaxSection::Documents
            } else {
                topic
            },
        ),
        &[],
    ) else {
        panic!("{name}: cold editor required")
    };
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(supported(&cold.draft), result);
}

#[test]
fn unit47_misc_business_meal_duplicate_and_list_limit_keep_native_priority() {
    for list_limit in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut business = business(1_000_000);
        business.expenses.meals = vec![
            MealExpense { kind: MealKind::Business, amount: 20_000 },
            MealExpense { kind: MealKind::Transportation, amount: 10_000 },
            MealExpense { kind: MealKind::FullyDeductible, amount: 5_000 },
        ];
        let corrected = save_topic(
            &mut session,
            TaxSection::SelfEmployment,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Business(business) },
            )],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
        let crate::IncomeRecord::Business(business) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let leaf = if list_limit {
            business.expenses.meals.push(business.expenses.meals[0].clone());
            // The fourth row fails uniqueness, not a list cap.
            "meals/3/kind"
        } else {
            business.expenses.meals[1].kind = MealKind::Business;
            "meals/1/kind"
        };
        checked(
            &alloc::format!("gap-business-meals-{list_limit}"),
            &mut session,
            bad,
            corrected,
            TaxSection::SelfEmployment,
            "BUSINESS_MEAL_KIND_ONCE",
            &alloc::format!("[documents/documents/0/document/expenses/{leaf}]"),
        );
        // $10,000 gross less 50%*$200, 80%*$100 and $50 = $9,770.
        assert_eq!(
            line_value(
                &supported(&session.draft),
                &alloc::format!("business_{}_net_profit", stored_business(&session).0)
            ),
            9_770
        );
    }
}

#[test]
fn unit47_misc_operating_depletion_requires_a_deposit() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut business = business(1_000_000);
    business.more.depletion = Some(BusinessDepletion {
        kind: RoyaltyKind::OilGas,
        depletion: RoyaltyDepletion::Cost {
            adjusted_basis: 20_000,
            recoverable_units: 100,
            units_sold: 10,
        },
    });
    let corrected = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(business) })],
    );
    let mut bad = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
    let crate::IncomeRecord::Business(business) = &mut documents.documents[0].document else {
        unreachable!()
    };
    business.more.depletion.as_mut().unwrap().kind = RoyaltyKind::Intangible;
    checked(
        "gap-business-depletion",
        &mut session,
        bad,
        corrected,
        TaxSection::SelfEmployment,
        "BUSINESS_DEPLETION_NEEDS_A_DEPOSIT",
        "[documents/documents/0/document/more/depletion/kind]",
    );
    // $200 basis *10/100 =$20 cost depletion, leaving $9,980.
    assert_eq!(
        line_value(
            &supported(&session.draft),
            &alloc::format!("business_{}_net_profit", stored_business(&session).0)
        ),
        9_980
    );
}

#[test]
fn unit47_misc_vehicle_earlier_use_requires_continuing_low_use() {
    for new in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut business = business(1_000_000);
        business.vehicles.push(Vehicle {
            method: VehicleMethod::Actual {
                macrs_source: None,
                id: 0,
                history_details: None,
                amt_section179_income_carry: None,
                allowed_amt_carry: Vec::new(),
                cost: 1_000_000,
                credit_reduction: 0,
                acquired: None,
                placed: 20250101,
                total_miles: 10_000,
                business_miles: 5_000,
                operating_costs: 100_000,
                loan_interest: 0,
                kind: crate::VehicleKind::HeavyTruckVan,
                history: AssetHistory::Continuing {
                    prior_depreciation: 100_000,
                    first_year_179: 0,
                    first_year_bonus: 0,
                    carryforward: 0,
                    mid_quarter: false,
                },
                disposal: crate::AssetDisposal::No,
                earlier_business_percent: Some(60),
            },
        });
        let corrected = save_topic(
            &mut session,
            TaxSection::SelfEmployment,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Business(business) },
            )],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
        let crate::IncomeRecord::Business(business) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let VehicleMethod::Actual { placed, history, business_miles, .. } =
            &mut business.vehicles[0].method
        else {
            unreachable!()
        };
        if new {
            *placed = 20260101;
            *history = AssetHistory::New { first_year: FirstYearDeduction::Bonus };
        } else {
            *business_miles = 5_001;
        }
        checked(
            &alloc::format!("gap-vehicle-earlier-{new}"),
            &mut session,
            bad,
            corrected,
            TaxSection::SelfEmployment,
            "VEHICLE_EARLIER_USE_ONLY_AFTER_USE_FELL",
            "[documents/documents/0/document/vehicles/0/method/earlier_business_percent]",
        );
        let (_, business) = stored_business(&session);
        assert!(matches!(
            business.assets[0].listed,
            ListedProperty::Vehicle { total_miles: 10_000, business_miles: 5_000, .. }
        ));
        assert_eq!(business.assets[0].earlier_business_percent, Some(60));
    }
}

#[test]
fn unit47_misc_surviving_spouse_sale_refuses_joint_answers() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut home = crate::tests::home::home_sale_page::home(false);
    home.facts_mut().more = HomeSaleMore::SurvivingSpouse {
        died: 20250301,
        both_qualified: true,
        spouse_earlier_sale: false,
        late_acquired: None,
        late_lived: 0,
    };
    let printed = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form1099S(home_sale_document(home)),
            },
        )],
    );
    let desired = crate::tests::support::client::income_pages::page_documents(&printed);
    accepted(&send(&mut session, printed));
    // The center also holds the return's W-2; the home sale's answers are on its own topic.
    let corrected =
        crate::tests::support::client::income_pages::information_return_answers_for_topic(
            &mut session,
            &desired,
            TaxSection::InvestmentSales,
        )
        .unwrap();
    let mut bad = corrected.clone();
    let P::SaveIncomes { followups, .. } = &mut bad else { unreachable!() };
    let row = followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::HomeSale { .. }))
        .unwrap();
    let crate::DocumentAnswer::HomeSale { value: Some(crate::RealEstateSaleUse::MainHome(home)) } = &mut row.answer else {
        unreachable!()
    };
    home.spouse =
        HomeSpouse::Joint { lived: HomeLived::WholeTime, earlier_sale: EarlierHomeSale::No };
    let path: alloc::string::String = libertas::SchemaPath::root()
        .field("followups")
        .key(row.key.as_str())
        .field("answer")
        .field("value")
        .field("MainHome")
        .field("spouse")
        .into();
    checked(
        "gap-home-surviving-joint",
        &mut session,
        bad,
        corrected,
        TaxSection::InvestmentSales,
        // The topic checks the spouse answers against the return's status before
        // the worksheet's own surviving-spouse rule (MainHomeSale::answers_issue).
        "HOME_SPOUSE_NOT_JOINT",
        &path,
    );
    // $450,000 -$25,000 -$200,000 =$225,000, fully excluded.
    assert_eq!(line_value(&supported(&session.draft), "home_2_excluded_gain"), 225_000);
    assert_eq!(line_value(&supported(&session.draft), "capital_gain"), 0);
}

#[test]
fn unit47_misc_credit_entity_requires_actual_income_source_and_designation_bound() {
    for above in [false, true] {
        let (mut session, corrected, source) = bond_editor();
        assert!(source > 0);
        let mut bad = corrected.clone();
        let (row, entity) = answered_origin(&mut bad, 1);
        let (key, leaf) = if above {
            entity.income_already_reported = entity.designated_income + 1;
            ("CREDIT_INCOME_ALREADY_ABOVE_DESIGNATED", "income_already_reported")
        } else {
            entity.income_source = None;
            ("CREDIT_INCOME_SOURCE_NEEDED", "income_source")
        };
        checked(
            &alloc::format!("gap-credit-entity-{above}"),
            &mut session,
            bad,
            corrected,
            TaxSection::MoreCredits,
            key,
            &alloc::format!("[value/answers/{row}/answer/value/{leaf}]"),
        );
        // $4,000 designated less $1,000 already reported adds $3,000 to the $1,000 DIV.
        assert_eq!(line_value(&supported(&session.draft), "agi"), 64_000);
    }
}

#[test]
fn unit47_misc_annual_debt_records_cannot_overlap_payer_allocations() {
    for branch in ["nominee", "prior", "net"] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let interest = crate::tests::investment::investment_statements::annual_interest(vec![
            crate::tests::investment::investment_statements::annual_row("Actual bond", 100_000, 0),
        ]);
        let printed = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Int1099(interest_document(interest)),
                },
            )],
        );
        let desired = crate::tests::support::client::income_pages::page_documents(&printed);
        accepted(&send(&mut session, printed));
        // The center also holds the return's W-2; the interest answers are on their own topic.
        let corrected =
            crate::tests::support::client::income_pages::information_return_answers_for_topic(
                &mut session,
                &desired,
                TaxSection::InterestDividends,
            )
            .unwrap();
        let mut bad = corrected.clone();
        let P::SaveIncomes { followups, .. } = &mut bad else { unreachable!() };
        let index = followups
            .iter()
            .position(|row| matches!(row.answer, crate::DocumentAnswer::InterestAdjustment { .. }))
            .unwrap();
        if branch == "nominee" {
            let row = followups
                .iter_mut()
                .find(|row| matches!(row.answer, crate::DocumentAnswer::InterestNominee { .. }))
                .unwrap();
            row.answer = crate::DocumentAnswer::InterestNominee {
                value: Some(
                    InterestNominee {
                        taxable: 0,
                        us_obligations: 0,
                        exempt: 0,
                        private_activity: 0,
                        foreign_tax: 0,
                        bond_premium: 0,
                        treasury_bond_premium: 0,
                        tax_exempt_bond_premium: 0,
                        accrued_interest: 0,
                        market_discount: 0,
                    }
                    .into(),
                ),
            };
        } else {
            let crate::DocumentAnswer::InterestAdjustment { value: Some(answers) } =
                &mut followups[index].answer
            else {
                unreachable!()
            };
            if branch == "prior" {
                answers.previously_reported = 1;
            } else {
                answers.reporting = DebtReporting::Net;
            }
        }
        let path: alloc::string::String = libertas::SchemaPath::root()
            .field("followups")
            .key(followups[index].key.as_str())
            .field("answer")
            .field("value")
            .field("instrument_records")
            .into();
        checked(
            &alloc::format!("gap-debt-payer-{branch}"),
            &mut session,
            bad,
            corrected,
            TaxSection::InterestDividends,
            "DEBT_PAYER_ALLOCATION_OVERLAP",
            &path,
        );
        assert_eq!(line_value(&supported(&session.draft), "agi"), 61_000);
    }
}

#[test]
fn unit47_misc_exchange_rental_date_uses_existing_guard_before_generic_range() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut rental = rental_page(1_200_000);
    rental.sale = crate::RentalSale::Exchanged(exchange_fold());
    let corrected = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) })],
    );
    let mut bad = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
    let crate::IncomeRecord::Rental(rental) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let crate::RentalSale::Exchanged(exchange) = &mut rental.sale else { unreachable!() };
    let crate::ReplacementUse::Rental { in_service, .. } = &mut exchange.replacement.used else {
        unreachable!()
    };
    // This malformed request also exceeds the static date window. The existing
    // native relationship guard runs first; no UI reachability is inferred.
    *in_service = 20280101;
    checked(
        "gap-exchange-rental-date",
        &mut session,
        bad,
        corrected,
        TaxSection::Rentals,
        "EXCHANGE_ENTER_THE_RENTAL_DATE",
        "[documents/documents/0/document/sale/replacement/used/in_service]",
    );
    let (_, _, stored) = stored_rentals(&session).remove(0);
    assert_eq!(stored.sale, crate::RentalSale::Exchanged(exchange_fold()));
}

#[test]
fn unit47_misc_injured_spouse_requires_actual_student_loan_borrower() {
    let mut draft = complete(FederalStatus::Joint);
    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    draft.injured_spouse = Some(InjuredSpouseAllocation::defaults());
    draft.adjustments.as_mut().unwrap().set_student_loan_interest(150_000);
    let mut session = Interview::new(TEST_USER, draft);
    let Reply::Edit(mut corrected) = crate::tests::support::client::native_editor(
        from_review(&mut session, TaxSection::InjuredSpouse),
        &[],
    ) else {
        unreachable!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut corrected else { unreachable!() };
    value.student_loan_borrower = Some(InjuredLoanBorrower::Taxpayer);
    let mut bad = corrected.clone();
    let P::SaveInjuredSpouse { value, .. } = &mut bad else { unreachable!() };
    value.student_loan_borrower = None;
    checked(
        "gap-injured-loan",
        &mut session,
        bad,
        corrected,
        TaxSection::InjuredSpouse,
        "INJURED_SPOUSE_LOAN_BORROWER_REQUIRED",
        "[value/student_loan_borrower]",
    );
    assert_eq!(line_value(&supported(&session.draft), "schedule_1_student_loan_interest"), 1_500);
    // The complete Joint fixture has $100,000 wages; the deduction is $1,500.
    assert_eq!(line_value(&supported(&session.draft), "agi"), 98_500);
}

#[test]
fn unit47_misc_opportunity_deferral_requires_positive_gain() {
    for long in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let corrected = save_topic(
            &mut session,
            TaxSection::InvestmentSales,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::OpportunityFundDeferral(QofDeferral {
                        fund: "Actual synthetic investment".into(),
                        invested_on: 20270215,
                        short_term: if long { 0 } else { 100 },
                        long_term: if long { 100 } else { 0 },
                        within_180_days: false,
                        special: false,
                    }),
                },
            )],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
        let crate::IncomeRecord::OpportunityFundDeferral(deferral) =
            &mut documents.documents[0].document
        else {
            unreachable!()
        };
        deferral.short_term = 0;
        deferral.long_term = 0;
        checked(
            &alloc::format!("gap-qof-positive-gain-{long}"),
            &mut session,
            bad,
            corrected,
            TaxSection::InvestmentSales,
            "QOF_DEFERRAL_NO_GAIN",
            "[documents/documents/0/document/short_term]",
        );
        let result = supported(&session.draft);
        assert_eq!(line_value(&result, "agi"), 60_000);
        assert!(noted(&result, "WARN_QOF_DEFERRAL_NOT_ALLOWED"));
    }
}

#[test]
fn unit47_misc_prize_withholding_includes_actual_noncash_award() {
    for noncash in [0, 50_000] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let corrected = save_topic(
            &mut session,
            TaxSection::OtherIncome,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::OtherIncome(OtherIncomeItem {
                        kind: OtherIncomeKind::Prize {
                            noncash,
                            disposition: PrizeDisposition::Received,
                        },
                        amount: 100_000,
                        withholding: 100_000 + noncash,
                    }),
                },
            )],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else { unreachable!() };
        let crate::IncomeRecord::OtherIncome(prize) = &mut documents.documents[0].document else {
            unreachable!()
        };
        prize.withholding += 1;
        checked(
            &alloc::format!("gap-prize-withholding-{noncash}"),
            &mut session,
            bad,
            corrected,
            TaxSection::OtherIncome,
            "PRIZE_WITHHOLDING_ABOVE_AWARD",
            "[documents/documents/0/document/withholding]",
        );
        assert_eq!(line_value(&supported(&session.draft), "agi"), 61_000 + noncash / 100);
    }
}

#[test]
fn unit47_misc_credit_entity_first_answer_requires_actual_income_source_and_designation_bound() {
    for above in [false, true] {
        let (mut session, corrected, source) = bond_question_editor();
        assert!(source > 0);
        let mut bad = corrected.clone();
        let entity = crate::tests::support::client::credit_documents::origin_entity(&mut bad);
        let (key, leaf) = if above {
            entity.income_already_reported = entity.designated_income + 1;
            ("CREDIT_INCOME_ALREADY_ABOVE_DESIGNATED", "income_already_reported")
        } else {
            entity.income_source = None;
            ("CREDIT_INCOME_SOURCE_NEEDED", "income_source")
        };
        crate::tests::support::client::credit_documents::origin_case(
            &alloc::format!("gap-credit-entity-first-answer-{above}"),
            &mut session,
            bad,
            corrected,
            key,
            &alloc::format!("[questions/0/answer/value/{leaf}]"),
        );
        // $4,000 designated less $1,000 already reported adds $3,000 to the $1,000 DIV.
        assert_eq!(line_value(&supported(&session.draft), "agi"), 64_000);
    }
}
