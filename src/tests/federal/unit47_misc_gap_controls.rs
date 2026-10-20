use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::unit47_payment_credit_admission::{bond_editor, case as credit_case};
use super::*;
use crate::tests::value as line_value;

fn calculation(request: &P) -> P {
    match request {
        P::SaveIncomes { .. } => credit_estimate_documents_from_save(request),
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
        P::SaveInjuredSpouse { value, .. } => P::EstimateInjuredSpouse {
            user: TEST_USER,
            value: value.clone(),
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
    credit_refusal(
        &alloc::format!("{name}-estimate"),
        session,
        calculation(&bad),
        key,
        path,
    );
    let before = session.draft.clone();
    let dirty = session.dirty;
    accepted(&send(session, calculation(&corrected)));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, P::from_avro(&corrected.to_avro()).unwrap()));
    assert_eq!(session.draft.revision, before.revision + 1);
    let saved = session.draft.clone();
    let result = supported(&saved);
    let Reply::Edit(reopened) =
        crate::tests::support::client::native_editor(from_review(session, topic), &[])
    else {
        panic!("{name}: editor required")
    };
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let Reply::Edit(reopened) =
        crate::tests::support::client::native_editor(from_review(&mut cold, topic), &[])
    else {
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
            MealExpense {
                kind: MealKind::Business,
                amount: 20_000,
            },
            MealExpense {
                kind: MealKind::Transportation,
                amount: 10_000,
            },
            MealExpense {
                kind: MealKind::FullyDeductible,
                amount: 5_000,
            },
        ];
        let corrected = save_topic(
            &session.draft,
            TaxSection::SelfEmployment,
            vec![document(0, IncomeDocument::Business(business))],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        let IncomeDocument::Business(business) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let leaf = if list_limit {
            business
                .expenses
                .meals
                .push(business.expenses.meals[0].clone());
            // 2026-10-15 D-13: the fourth row fails uniqueness, not a list cap.
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
        &session.draft,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeDocument::Business(business))],
    );
    let mut bad = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else {
        unreachable!()
    };
    let IncomeDocument::Business(business) = &mut documents.documents[0].document else {
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
                passenger_automobile: false,
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
            &session.draft,
            TaxSection::SelfEmployment,
            vec![document(0, IncomeDocument::Business(business))],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        let IncomeDocument::Business(business) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let VehicleMethod::Actual {
            placed,
            history,
            business_miles,
            ..
        } = &mut business.vehicles[0].method
        else {
            unreachable!()
        };
        if new {
            *placed = 20260101;
            *history = AssetHistory::New {
                first_year: FirstYearDeduction::Bonus,
            };
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
            ListedProperty::Vehicle {
                total_miles: 10_000,
                business_miles: 5_000,
                ..
            }
        ));
        assert_eq!(business.assets[0].earlier_business_percent, Some(60));
    }
}

#[test]
fn unit47_misc_surviving_spouse_sale_refuses_joint_answers() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut home = crate::tests::home::home_sale_page::home(false);
    home.more = HomeSaleMore::SurvivingSpouse {
        died: 20250301,
        both_qualified: true,
        spouse_earlier_sale: false,
    };
    let corrected = save_topic(
        &session.draft,
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::MainHomeSale(home))],
    );
    let mut bad = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else {
        unreachable!()
    };
    let IncomeDocument::MainHomeSale(home) = &mut documents.documents[0].document else {
        unreachable!()
    };
    home.spouse = HomeSpouse::Joint {
        lived: HomeLived::WholeTime,
        earlier_sale: EarlierHomeSale::No,
    };
    checked(
        "gap-home-surviving-joint",
        &mut session,
        bad,
        corrected,
        TaxSection::InvestmentSales,
        "HOME_SURVIVING_ON_JOINT",
        "[documents/documents/0/document/spouse]",
    );
    // $450,000 -$25,000 -$200,000 =$225,000, fully excluded.
    assert_eq!(
        line_value(&supported(&session.draft), "home_2_excluded_gain"),
        225_000
    );
    assert_eq!(line_value(&supported(&session.draft), "capital_gain"), 0);
}

#[test]
fn unit47_misc_credit_entity_requires_actual_income_source_and_designation_bound() {
    for above in [false, true] {
        let (mut session, corrected, source) = bond_editor();
        assert!(source > 0);
        let mut bad = corrected.clone();
        let P::SaveMoreCredits { value, .. } = &mut bad else {
            unreachable!()
        };
        let MoreCredit::TaxCreditBonds { tax_credit_bonds } = &mut value.credits[1] else {
            unreachable!()
        };
        let TaxCreditOriginInput::EntityStatement(entity) = &mut tax_credit_bonds[1].origin else {
            unreachable!()
        };
        let (key, leaf) = if above {
            entity.income_already_reported = entity.designated_income + 1;
            (
                "CREDIT_INCOME_ALREADY_ABOVE_DESIGNATED",
                "income_already_reported",
            )
        } else {
            entity.income_source = None;
            ("CREDIT_INCOME_SOURCE_NEEDED", "income_source")
        };
        credit_case(
            &alloc::format!("gap-credit-entity-{above}"),
            &mut session,
            bad,
            corrected,
            TaxSection::MoreCredits,
            key,
            &alloc::format!("[value/credits/1/tax_credit_bonds/1/origin/{leaf}]"),
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
        let corrected = save_topic(
            &session.draft,
            TaxSection::InterestDividends,
            vec![document(0, IncomeDocument::Int1099(interest))],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        let IncomeDocument::Int1099(interest) = &mut documents.documents[0].document else {
            unreachable!()
        };
        match branch {
            "nominee" => {
                interest.nominee = Some(InterestNominee {
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
                });
            }
            "prior" => interest.adjustments.as_mut().unwrap().previously_reported = 1,
            "net" => interest.adjustments.as_mut().unwrap().reporting = DebtReporting::Net,
            _ => unreachable!(),
        }
        checked(
            &alloc::format!("gap-debt-payer-{branch}"),
            &mut session,
            bad,
            corrected,
            TaxSection::InterestDividends,
            "DEBT_PAYER_ALLOCATION_OVERLAP",
            "[documents/documents/0/document/adjustments/instrument_records]",
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
        &session.draft,
        TaxSection::Rentals,
        vec![document(0, IncomeDocument::Rental(rental))],
    );
    let mut bad = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut bad else {
        unreachable!()
    };
    let IncomeDocument::Rental(rental) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let crate::RentalSale::Exchanged(exchange) = &mut rental.sale else {
        unreachable!()
    };
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
    draft.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
    draft.injured_spouse = Some(InjuredSpouseAllocation::defaults());
    draft
        .adjustments
        .as_mut()
        .unwrap()
        .set_student_loan_interest(150_000);
    let mut session = Interview::new(TEST_USER, draft);
    let Reply::Edit(mut corrected) = crate::tests::support::client::native_editor(
        from_review(&mut session, TaxSection::InjuredSpouse),
        &[],
    ) else {
        unreachable!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut corrected else {
        unreachable!()
    };
    value.student_loan_borrower = Some(InjuredLoanBorrower::Taxpayer);
    let mut bad = corrected.clone();
    let P::SaveInjuredSpouse { value, .. } = &mut bad else {
        unreachable!()
    };
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
    assert_eq!(
        line_value(
            &supported(&session.draft),
            "schedule_1_student_loan_interest"
        ),
        1_500
    );
    // The complete Joint fixture has $100,000 wages; the deduction is $1,500.
    assert_eq!(line_value(&supported(&session.draft), "agi"), 98_500);
}

#[test]
fn unit47_misc_opportunity_deferral_requires_positive_gain() {
    for long in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let corrected = save_topic(
            &session.draft,
            TaxSection::InvestmentSales,
            vec![document(
                0,
                IncomeDocument::OpportunityFundDeferral(QofDeferral {
                    fund: "Actual synthetic investment".into(),
                    invested_on: 20270215,
                    short_term: if long { 0 } else { 100 },
                    long_term: if long { 100 } else { 0 },
                    within_180_days: false,
                    special: false,
                }),
            )],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        let IncomeDocument::OpportunityFundDeferral(deferral) =
            &mut documents.documents[0].document
        else {
            unreachable!()
        };
        let mut wide = deferral.clone();
        wide.short_term = i64::MAX;
        wide.long_term = i64::MAX;
        assert_eq!(wide.issue(), None);
        assert!(!wide.valid());
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
            &session.draft,
            TaxSection::OtherIncome,
            vec![document(
                0,
                IncomeDocument::OtherIncome(OtherIncomeItem {
                    kind: OtherIncomeKind::Prize {
                        noncash,
                        disposition: PrizeDisposition::Received,
                    },
                    amount: 100_000,
                    withholding: 100_000 + noncash,
                }),
            )],
        );
        let mut bad = corrected.clone();
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        let IncomeDocument::OtherIncome(prize) = &mut documents.documents[0].document else {
            unreachable!()
        };
        let mut wide = prize.clone();
        let OtherIncomeKind::Prize { noncash: wide_noncash, .. } = &mut wide.kind else {
            unreachable!()
        };
        *wide_noncash = i64::MAX;
        assert_eq!(wide.issue(), None);
        assert!(!wide.valid());
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
        assert_eq!(
            line_value(&supported(&session.draft), "agi"),
            61_000 + noncash / 100
        );
    }
}
