use crate::tests::support::client::survey::survey_native_refusal as refuse;
use super::*;
use crate::tests::support::client::source_calculation;

fn calculation(request: &P) -> P {
    match request {
        P::SavePayments {
            allowed_prior_returns,
            allowed_farmer,
            allowed_farmer_next_year,
            allowed_refund_management,
            value,
            ..
        } => P::EstimatePayments {
            user: TEST_USER,
            allowed_prior_returns: allowed_prior_returns.clone(),
            allowed_farmer: allowed_farmer.clone(),
            allowed_farmer_next_year: allowed_farmer_next_year.clone(),
            allowed_refund_management: allowed_refund_management.clone(),
            value: value.clone().into(),
        },
        P::SaveMoreCredits {
            allowed_owners,
            foreign_businesses,
            capital_sales,
            foreign_situations,
            mortgage_loans,
            businesses,
            farms,
            clean_vehicle_properties,
            rentals,
            research_businesses,
            entered_credits,
            carryback_credits,
            allowed_carry_filers,
            initial_credit_elections,
            initial_research_choices,
            value,
            initial_carbon_oxide_origins,
            initial_renewable_facility_origins,
            ..
        } => P::EstimateMoreCredits {
            user: TEST_USER,
            allowed_owners: allowed_owners.clone(),
            foreign_businesses: foreign_businesses.clone(),
            capital_sales: capital_sales.clone(),
            foreign_situations: foreign_situations.clone(),
            mortgage_loans: mortgage_loans.clone(),
            businesses: businesses.clone(),
            farms: farms.clone(),
            clean_vehicle_properties: clean_vehicle_properties.clone(),
            rentals: rentals.clone(),
            research_businesses: research_businesses.clone(),
            entered_credits: entered_credits.clone(),
            carryback_credits: carryback_credits.clone(),
            allowed_carry_filers: allowed_carry_filers.clone(),
            initial_credit_elections: initial_credit_elections.clone(),
            initial_research_choices: initial_research_choices.clone(),
            value: value.clone(),
            initial_carbon_oxide_origins: initial_carbon_oxide_origins.clone(),
            initial_renewable_facility_origins: initial_renewable_facility_origins.clone(),
        },
        P::SaveGiftCarryovers {
            allowed_owners,
            retained_charity_k1s,
            retained_charity_origins,
            retained_research_choices,
            value,
            ..
        } => P::EstimateGiftCarryovers {
            user: TEST_USER,
            allowed_owners: allowed_owners.clone(),
            retained_charity_k1s: retained_charity_k1s.clone(),
            retained_charity_origins: retained_charity_origins.clone(),
            retained_research_choices: retained_research_choices.clone(),
            value: value.clone(),
        },
        P::SaveOtherTaxes {
            allowed_owners,
            mortgage_gift_owners,
            coverdell_corrections,
            value,
            ..
        } => P::EstimateOtherTaxes {
            user: TEST_USER,
            allowed_owners: allowed_owners.clone(),
            mortgage_gift_owners: mortgage_gift_owners.clone(),
            coverdell_corrections: coverdell_corrections.clone(),
            value: value.clone(),
        },
        P::SaveEmployeeExpenses {
            allowed_owners,
            value,
            ..
        } => P::EstimateEmployeeExpenses {
            user: TEST_USER,
            allowed_owners: allowed_owners.clone(),
            value: value.clone(),
        },
        _ => panic!("payment, credit, gift, tax, or employee editor required"),
    }
}

fn open(session: &mut Interview, topic: TaxSection) -> P {
    let Reply::Edit(page) = from_review(session, topic) else {
        panic!("{topic:?} editor required")
    };
    page
}

pub(super) fn case(
    case: &str,
    session: &mut Interview,
    bad: P,
    corrected: P,
    topic: TaxSection,
    key: &str,
    path: &str,
) {
    refuse(case, session, bad.clone(), key, path);
    refuse(
        &alloc::format!("{case}-estimate"),
        session,
        calculation(&bad),
        key,
        path,
    );
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = send(session, calculation(&corrected));
    assert!(if matches!(corrected, P::SavePayments { .. }) {
        matches!(reply, Reply::Response(P::PaymentsEstimate { .. }))
    } else {
        matches!(reply, Reply::Response(P::RunningEstimate { .. }))
    });
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, P::from_avro(&corrected.to_avro()).unwrap()));
    assert_eq!(session.draft.revision, before.revision + 1);
    let saved = session.draft.clone();
    let reopened = open(session, topic);
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open(&mut cold, topic);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}
fn penalty(withholding: WithholdingTiming, farmer: Farmer, income: IncomeTiming) -> PenaltyChoice {
    PenaltyChoice::Figure {
        prior_tax: 100_000,
        prior_agi: Some(5_000_000),
        full_year: true,
        prior_return: PriorReturn::SameReturn,
        farmer,
        withholding,
        income,
        waiver: Waiver::None,
    }
}
#[test]
fn unit47_payment_extension_payer_iff_locates_both_absence_failures() {
    for present in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::Payments);
        let P::SavePayments { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.estimated = vec![crate::InjuredDatedPayment {
            date: 20260415,
            amount: 1_000,
            payer: PaymentPayer::Taxpayer,
            refund_property: None,
        }];
        value.extension = if present { 10_000 } else { 0 };
        value.extension_payer = if present {
            Some(PaymentPayer::Taxpayer)
        } else {
            None
        };
        let mut bad = corrected.clone();
        let P::SavePayments { value, .. } = &mut bad else {
            unreachable!()
        };
        value.extension_payer = if present {
            None
        } else {
            Some(PaymentPayer::Taxpayer)
        };
        case(
            &alloc::format!("admission-payment-extension-{present}"),
            &mut session,
            bad,
            corrected,
            TaxSection::Payments,
            "INJURED_SPOUSE_EXTENSION_PAYER_REQUIRED",
            "[value/extension_payer]",
        );
    }
}
#[test]
fn unit47_payment_withholding_uses_real_return_total_and_precedes_farmer_guard() {
    for mutation in 0..3 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::Payments);
        let P::SavePayments { value, .. } = &mut corrected else {
            unreachable!()
        };
        let total = value.withheld.as_ref().unwrap().total;
        assert!(total > 0);
        value.penalty = penalty(
            WithholdingTiming::ByPeriod {
                first: total,
                second: 0,
                third: 0,
                fourth: 0,
            },
            Farmer::No,
            IncomeTiming::Evenly,
        );
        let mut bad = corrected.clone();
        let P::SavePayments { value, .. } = &mut bad else {
            unreachable!()
        };
        let PenaltyChoice::Figure {
            withholding,
            farmer,
            ..
        } = &mut value.penalty
        else {
            unreachable!()
        };
        let WithholdingTiming::ByPeriod { first, fourth, .. } = withholding else {
            unreachable!()
        };
        if mutation == 0 {
            *first -= 1;
        } else {
            *fourth = 1;
        }
        if mutation == 1 {
            // The echoed total cannot authorize the wrong sum.
            value.withheld.as_mut().unwrap().total = total + 1;
        }
        if mutation == 2 {
            *farmer = Farmer::Yes {
                paid_by_march_1: false,
            };
        }
        case(
            &alloc::format!("admission-payment-withholding-{mutation}"),
            &mut session,
            bad,
            corrected,
            TaxSection::Payments,
            "WITHHOLDING_PERIODS_NOT_TOTAL",
            "[value/penalty/withholding]",
        );
    }
}
#[test]
fn unit47_payment_farmer_timing_locates_actual_conflicting_choice() {
    for mutation in 0..3 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::Payments);
        let P::SavePayments { value, .. } = &mut corrected else {
            unreachable!()
        };
        let total = value.withheld.as_ref().unwrap().total;
        value.penalty = penalty(
            WithholdingTiming::Evenly,
            Farmer::Yes {
                paid_by_march_1: false,
            },
            IncomeTiming::Evenly,
        );
        let mut bad = corrected.clone();
        let P::SavePayments { value, .. } = &mut bad else {
            unreachable!()
        };
        let PenaltyChoice::Figure {
            withholding,
            income,
            ..
        } = &mut value.penalty
        else {
            unreachable!()
        };
        if mutation != 1 {
            *withholding = WithholdingTiming::ByPeriod {
                first: total,
                second: 0,
                third: 0,
                fourth: 0,
            };
        }
        if mutation != 0 {
            *income = IncomeTiming::ByPeriod;
        }
        let field = if mutation == 1 {
            "[value/penalty/income]"
        } else {
            "[value/penalty/withholding]"
        };
        case(
            &alloc::format!("admission-payment-farmer-{mutation}"),
            &mut session,
            bad,
            corrected,
            TaxSection::Payments,
            "FARMER_EVEN_TIMING",
            field,
        );
    }
}
fn bond(id: &str, source: i64, amount: i64) -> TaxCreditBondInput {
    let mut record = direct_tax_credit_bond(
        id,
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        1_000,
    );
    record.origin = TaxCreditOrigin::EntityStatement(TaxCreditEntity {
        entity: "Synthetic shareholder report".into(),
        designated_income: 200_000,
        income_already_reported: amount,
        income_kind: TaxCreditIncomeKind::RicDistribution,
        income_source: Some(source),
    });
    record.into()
}
pub(super) fn bond_editor() -> (Interview, P, i64) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { documents, .. } = &mut page else {
        unreachable!()
    };
    documents.documents = vec![document(
        0,
        IncomeDocument::Div1099(native_div1099("Synthetic accepted RIC".into(), 100_000, 0)),
    )];
    accepted(&send(&mut session, page));
    let mut page = open(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value, .. } = &mut page else {
        unreachable!()
    };
    let sources = &value.credit_income_sources;
    assert_eq!(sources.len(), 1);
    let source = sources[0].id;
    assert_eq!(sources[0].amount, 100_000);
    value.credits = vec![
        MoreCredit::DeclineEarnedIncomeCredit,
        MoreCredit::TaxCreditBonds {
            tax_credit_bonds: vec![
                bond("FIRST", source, 60_000),
                bond("SECOND", source, 40_000),
            ],
        },
    ];
    (session, page, source)
}
#[test]
fn unit47_credit_source_owner_locates_kind_identity_amount_and_aggregate() {
    for mutation in 0..4 {
        let (mut session, corrected, _) = bond_editor();
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
        let path = match mutation {
            0 => {
                entity.income_kind = TaxCreditIncomeKind::Interest;
                "[value/credits/1/tax_credit_bonds/1/origin/income_kind]"
            }
            1 => {
                entity.income_source = Some(999_999);
                "[value/credits/1/tax_credit_bonds/1/origin/income_source]"
            }
            2 => {
                entity.income_already_reported = 100_001;
                "[value/credits/1/tax_credit_bonds/1/origin/income_already_reported]"
            }
            _ => {
                entity.income_already_reported += 1;
                "[value/credits/1/tax_credit_bonds]"
            }
        };
        case(
            &alloc::format!("admission-credit-source-{mutation}"),
            &mut session,
            bad,
            corrected,
            TaxSection::MoreCredits,
            "CREDIT_INCOME_SOURCE_INVALID",
            path,
        );
    }
}
#[test]
fn unit47_credit_source_projection_uses_last_submitted_bond_section() {
    let (mut session, mut corrected, source) = bond_editor();
    let P::SaveMoreCredits { value, .. } = &mut corrected else {
        unreachable!()
    };
    value.credits.insert(
        1,
        MoreCredit::TaxCreditBonds {
            tax_credit_bonds: vec![bond("SUPERSEDED", source, 50_000)],
        },
    );
    let mut bad = corrected.clone();
    let P::SaveMoreCredits { value, .. } = &mut bad else {
        unreachable!()
    };
    let MoreCredit::TaxCreditBonds { tax_credit_bonds } = &mut value.credits[2] else {
        unreachable!()
    };
    let TaxCreditOriginInput::EntityStatement(entity) = &mut tax_credit_bonds[1].origin else {
        unreachable!()
    };
    entity.income_source = Some(999_999);
    case(
        "admission-credit-last-section",
        &mut session,
        bad,
        corrected,
        TaxSection::MoreCredits,
        "CREDIT_INCOME_SOURCE_INVALID",
        "[value/credits/2/tax_credit_bonds/1/origin/income_source]",
    );
}
#[test]
fn unit47_gift_appreciation_owner_locates_real_carryover_and_source() {
    let (draft, source) = crate::tests::credits::form_3800::retained_charity_draft();
    let mut session = Interview::new(TEST_USER, draft);
    let mut corrected = open(&mut session, TaxSection::GiftCarryovers);
    let P::SaveGiftCarryovers {
        value,
        retained_research_choices,
        ..
    } = &mut corrected
    else {
        unreachable!()
    };
    assert_eq!(retained_research_choices, &vec![0]);
    value.basis_election = true;
    value.carryovers.push(CharityCarryoverPage {
        year: 2023,
        class: CharityClass::CapitalGain30,
        balance: 1_000_000,
        appreciation: Some(400_000),
        same_filers: true,
        allowed_research_parts: vec![0],
        allowed_source_appreciation: vec![0],
        research_parts: Some(ResearchCharityPartsPage::Parts {
            sources: vec![
                ResearchCharityPartPage {
                    origin: ResearchCharityOriginPage::CurrentK1 { source },
                    amount: 200_000,
                    original_appreciation: Some(CharitySourceAppreciation::Amount {
                        amount: 100_000,
                    }),
                },
                ResearchCharityPartPage {
                    origin: ResearchCharityOriginPage::OtherEntity(OtherCharityEntity {
                        name: "Synthetic other donor entity".into(),
                        ein: "98-7654321".into(),
                    }),
                    amount: 200_000,
                    original_appreciation: Some(CharitySourceAppreciation::Amount {
                        amount: 100_000,
                    }),
                },
            ],
        }),
    });
    let mut bad = corrected.clone();
    let P::SaveGiftCarryovers { value, .. } = &mut bad else {
        unreachable!()
    };
    let Some(ResearchCharityPartsPage::Parts { sources }) = &mut value.carryovers[1].research_parts
    else {
        unreachable!()
    };
    sources[1].original_appreciation = None;
    case(
        "admission-gift-source-appreciation",
        &mut session,
        bad,
        corrected,
        TaxSection::GiftCarryovers,
        "RETAINED_CHARITY_SOURCE_APPRECIATION_NEEDED",
        "[value/carryovers/1/research_parts/sources/1/original_appreciation]",
    );
}

#[test]
fn unit47_employee_standard_interest_and_property_tax_locate_actual_more_fields() {
    for property_tax in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::EmployeeExpenses);
        let P::SaveEmployeeExpenses { value, .. } = &mut corrected else {
            unreachable!()
        };
        let vehicle = Vehicle {
            method: VehicleMethod::StandardMileage {
                id: 0,
                first_half: 20,
                second_half: 30,
                total: 100,
                more: MileageMore::default(),
            },
        };
        value.forms = vec![EmployeeExpenseForm {
            dot_hours_of_service: false,
            owner: Owner::Taxpayer,
            occupation: "Synthetic fee-basis official".into(),
            capacity: EmployeeCapacity::FeeBasisOfficial,
            employers: vec![1],
            local_transportation: 0,
            travel: 1_000,
            other: 0,
            meals: 0,
            reimbursements: 0,
            vehicles: vec![vehicle.clone(), vehicle],
        }];
        let mut bad = corrected.clone();
        let P::SaveEmployeeExpenses { value, .. } = &mut bad else {
            unreachable!()
        };
        let VehicleMethod::StandardMileage { more, .. } = &mut value.forms[0].vehicles[1].method
        else {
            unreachable!()
        };
        if property_tax {
            more.property_tax = 1;
        } else {
            more.interest = 1;
        }
        let key = "EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX";
        let path = if property_tax {
            "[value/forms/0/vehicles/1/method/more/property_tax]"
        } else {
            "[value/forms/0/vehicles/1/method/more/interest]"
        };
        let name = alloc::format!("admission-employee-standard-{property_tax}");
        refuse(&name, &mut session, bad.clone(), key, path);
        refuse(
            &alloc::format!("{name}-estimate"),
            &mut session,
            source_calculation(&bad),
            key,
            path,
        );
        let before = session.draft.clone();
        let dirty = session.dirty;
        assert!(matches!(
            send(&mut session, source_calculation(&corrected)),
            Reply::Response(P::RunningEstimate { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft.revision, before.revision + 1);
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let reopened = open(&mut cold, TaxSection::EmployeeExpenses);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
    }
}

#[test]
fn unit47_employee_reservist_lease_reuses_claim_policy_owner_and_old_message() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut corrected = open(&mut session, TaxSection::EmployeeExpenses);
    let P::SaveEmployeeExpenses { value, .. } = &mut corrected else {
        unreachable!()
    };
    value.forms = vec![EmployeeExpenseForm {
        dot_hours_of_service: false,
        owner: Owner::Taxpayer,
        occupation: "Synthetic fee-basis official".into(),
        capacity: EmployeeCapacity::FeeBasisOfficial,
        employers: vec![1],
        local_transportation: 0,
        travel: 1_000,
        other: 0,
        meals: 0,
        reimbursements: 0,
        vehicles: vec![Vehicle {
            method: VehicleMethod::Leased {
                vehicle: BusinessLeasedVehicle {
                    began: 20260101,
                    ends: Some(20260130),
                    fair_market_value: 1_000_000,
                    lease_payments: 100_000,
                    total_miles: 100,
                    business_miles: 80,
                    commuting: 20,
                    operating_costs: 0,
                    parking_tolls: 0,
                    passenger_automobile: true,
                    personal_use_available: true,
                    other_vehicle: true,
                    evidence: VehicleEvidence::Written,
                },
            },
        }],
    }];
    let mut bad = corrected.clone();
    let P::SaveEmployeeExpenses { value, .. } = &mut bad else {
        unreachable!()
    };
    value.forms[0].capacity = EmployeeCapacity::Reservist;
    case(
        "admission-employee-reservist-lease",
        &mut session,
        bad,
        corrected,
        TaxSection::EmployeeExpenses,
        "MESSAGE_CHECK_THE_FORM_2106_AMOUNTS_AND_VEHICLES",
        "[value/forms/0/vehicles]",
    );
}

#[test]
fn unit47_other_tax_duplicate_refusal_keeps_actual_submitted_record() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut corrected = open(&mut session, TaxSection::OtherTaxes);
    let P::SaveOtherTaxes { value, .. } = &mut corrected else {
        unreachable!()
    };
    value.taxes = vec![
        OtherTax::AlternativeMinimumTax,
        OtherTax::HousingCreditRecapture,
    ];
    let mut bad = corrected.clone();
    let P::SaveOtherTaxes { value, .. } = &mut bad else {
        unreachable!()
    };
    value.taxes.push(OtherTax::AlternativeMinimumTax);
    case(
        "admission-other-tax-refusal-duplicate",
        &mut session,
        bad,
        corrected,
        TaxSection::OtherTaxes,
        "MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE",
        "[value/taxes/2]",
    );
}

#[test]
fn unit47_payment_source_amounts_have_no_prototype_ceiling_and_keep_atomic_minimum() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let baseline = supported(&session.draft);
    let mut corrected = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut corrected else {
        unreachable!()
    };
    value.estimated = vec![
        page_paid(20260415, 100_000_000_001),
        page_paid(20260915, 99_999_999_999),
    ];
    value.extension = 200_000_000_000;
    value.extension_payer = Some(PaymentPayer::Taxpayer);
    let mut bad = corrected.clone();
    let P::SavePayments { value, .. } = &mut bad else {
        unreachable!()
    };
    value.estimated[0].amount = 0;
    case(
        "payment-positive-source",
        &mut session,
        bad,
        corrected,
        TaxSection::Payments,
        "PAYMENT_AMOUNT_INVALID",
        "[value/estimated/0/amount]",
    );
    let result = supported(&session.draft);
    assert_eq!(result.tax, baseline.tax);
    assert_eq!(
        result.payments,
        baseline.payments.map(|paid| paid + 400_000_000_000)
    );
    assert_eq!(
        session.draft.payments.as_ref().unwrap().estimated,
        Some(200_000_000_000)
    );
}

#[test]
fn unit47_payment_wide_known_sources_round_once_and_retain_joint_fund_capacity() {
    // 2026-10-16, Money precision / Derived carrier: MAX ends in07 cents.
    // MAX-7 plus8 cents is known, exceeds a duplicate i64 source total, and
    // reports MAX-7 cents. Neither None projection discards those source facts.
    for community in [false, true] {
        let mut draft = empty_return(FederalStatus::Joint);
        let mut claim = InjuredSpouseAllocation::defaults();
        claim.estimated = EstimatedSplit::Agreed { injured: i64::MAX };
        if community {
            claim.community = InjuredCommunityFacts::Yes {
                states: vec![5],
                debt_facts: InjuredDebtFacts::NotApplicable,
            };
        }
        draft.injured_spouse = Some(claim);
        draft.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
        let mut session = Interview::new(TEST_USER, draft);
        let mut page = open(&mut session, TaxSection::Payments);
        let P::SavePayments { value, .. } = &mut page else {
            unreachable!()
        };
        value.estimated = [i64::MAX - 7, 8]
            .map(|amount| crate::InjuredDatedPayment {
                date: 20260415,
                amount,
                payer: PaymentPayer::Joint,
                refund_property: Some(crate::RefundSourceProperty::Separate {
                    owner: Owner::Taxpayer,
                }),
            })
            .to_vec();
        let before = session.draft.clone();
        let dirty = session.dirty;
        assert!(matches!(
            send(&mut session, calculation(&page)),
            Reply::Response(P::PaymentsEstimate { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, page));
        let result = supported(&session.draft);
        assert_eq!(result.payments, Some(i64::MAX - 7));
        assert_eq!(result.refund, Some(i64::MAX - 7));
        assert_eq!(return_line(&result, "26"), i64::MAX - 7);
        assert_eq!(
            result
                .lines
                .iter()
                .find(|line| line.id == crate::injured_spouse::ESTIMATE_LINE)
                .unwrap()
                .amount,
            i64::MAX - 7
        );
        assert_eq!(session.draft.payments.as_ref().unwrap().estimated, None);
        let value = session
            .draft
            .injured_spouse
            .as_ref()
            .unwrap()
            .page(&session.draft, &result);
        assert_eq!(value.joint_estimated, None);
        assert!(
            crate::injured_spouse::issue(
                &value,
                &crate::injured_spouse::joint_estimated(&session.draft)
            )
            .is_none()
        );
        let saved = session.draft.clone();
        let page = open(&mut session, TaxSection::Payments);
        accepted(&send(&mut session, page));
        assert_eq!(session.draft, saved);
        let mut cold = restart(saved.clone());
        let page = open(&mut cold, TaxSection::Payments);
        accepted(&send(&mut cold, page));
        assert_eq!(cold.draft, saved);
        assert_eq!(supported(&cold.draft), result);
    }
}

#[test]
fn unit47_payment_real_output_failure_retains_sources_and_corrects_without_zero() {
    let mut session = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let mut page = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut page else {
        unreachable!()
    };
    value.estimated = vec![page_paid(20260415, i64::MAX), page_paid(20260915, i64::MAX)];
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(&mut session, calculation(&page)),
        Reply::Response(P::PaymentsEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, page));
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "PAYMENT_AMOUNT_UNREPRESENTABLE")
    );
    assert_eq!(
        (result.tax, result.payments, result.refund, result.balance),
        (None, None, None, None)
    );
    assert_eq!(session.draft.payments.as_ref().unwrap().estimated, None);
    let mut page = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut page else {
        unreachable!()
    };
    assert_eq!(
        value
            .estimated
            .iter()
            .map(|payment| payment.amount)
            .collect::<Vec<_>>(),
        vec![i64::MAX; 2]
    );
    value.estimated = vec![page_paid(20260415, 100)];
    accepted(&send(&mut session, page));
    assert_eq!(supported(&session.draft).refund, Some(100));
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let page = open(&mut cold, TaxSection::Payments);
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved);

    // The actual line33 encoding can fail after native tax/refund fields were
    // assigned: payment MAX-7 fits, and tax502300 exceeds credit110000, so the
    // refund also fits. The printed total of payments plus credits does not.
    let mut session = Interview::new(
        TEST_USER,
        repayment_comparison_draft(RepaymentMethod::Automatic),
    );
    let baseline = supported(&session.draft);
    assert_eq!(baseline.tax, Some(502_300));
    assert_eq!(baseline.refundable_credits, Some(110_000));
    assert_eq!(baseline.payments, Some(650_000));
    let mut page = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut page else {
        unreachable!()
    };
    value.estimated = vec![page_paid(20260415, i64::MAX - 650_007)];
    accepted(&send(&mut session, page));
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "PAYMENT_AMOUNT_UNREPRESENTABLE")
    );
    assert_eq!(
        (
            result.tax,
            result.refundable_credits,
            result.payments,
            result.refund,
            result.balance
        ),
        (None, None, None, None, None)
    );
    assert!(result.return_lines.is_empty());
    assert_eq!(result.payment_withholding.unwrap().total, 650_000);
}

#[test]
fn unit47_payment_unavailable_withholding_keeps_first_composition_open() {
    // 2026-10-16 Source facts correction: missing deductions are unrelated
    // and normal construction fills them. The unfinished actual income page
    // stops first_incomplete before the native cash collector; known filers
    // still permit opening Payments through the ordinary navigation owner.
    let mut draft = complete(FederalStatus::Single);
    draft
        .finished_sections
        .retain(|section| *section != TaxSection::Income);
    draft.deductions = None;
    let mut session = Interview::new(TEST_USER, draft);
    let mut corrected = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut corrected else {
        unreachable!()
    };
    assert_eq!(value.withheld, None);
    // The correction commits a real source edit; submitting the unchanged
    // default IrsFigures answer is correctly a no-op with no new revision.
    value.estimated = vec![page_paid(20260415, 100)];
    let mut bad = corrected.clone();
    let P::SavePayments { value, .. } = &mut bad else {
        unreachable!()
    };
    value.penalty = penalty(
        WithholdingTiming::ByPeriod {
            first: 0,
            second: 0,
            third: 0,
            fourth: 0,
        },
        Farmer::No,
        IncomeTiming::Evenly,
    );
    case(
        "payment-unavailable-source",
        &mut session,
        bad,
        corrected,
        TaxSection::Payments,
        "PAYMENT_WITHHOLDING_UNAVAILABLE",
        "[value/penalty/withholding]",
    );
    // An unrelated unsupported credit does not erase collected cash amounts.
    let mut draft = complete(FederalStatus::Single);
    draft.credit_refusals = vec![crate::MoreCreditRefusal::OtherCredit];
    let mut session = Interview::new(TEST_USER, draft);
    let P::SavePayments { value, .. } = open(&mut session, TaxSection::Payments) else {
        unreachable!()
    };
    assert_eq!(value.withheld.unwrap().total, 650_000);
}

#[test]
fn unit47_payment_form2210_checks_its_actual_column_carrier() {
    let mut draft = estate_trust_code_c_draft(0, 0);
    draft.income[0] = wage(1, Owner::Taxpayer, 8_110_000, 0);
    let estate = estate_trust_code_d_data(&mut draft);
    estate.allocated_estimated_tax = 8;
    estate.form_1041t_timely = true;
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut page else {
        unreachable!()
    };
    value.estimated = vec![page_paid(20270115, i64::MAX - 7)];
    accepted(&send(&mut session, page));
    let ordinary = supported(&session.draft);
    assert_eq!(return_line(&ordinary, "26"), i64::MAX - 7);
    assert_eq!(ordinary.tax, Some(901_800));
    let mut page = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut page else {
        unreachable!()
    };
    value.penalty = penalty(WithholdingTiming::Evenly, Farmer::No, IncomeTiming::Evenly);
    accepted(&send(&mut session, page));
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "PAYMENT_AMOUNT_UNREPRESENTABLE")
    );
    assert_eq!(
        (result.tax, result.payments, result.refund, result.balance),
        (None, None, None, None)
    );
    let mut page = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut page else {
        unreachable!()
    };
    value.penalty = PenaltyChoice::IrsFigures;
    accepted(&send(&mut session, page));
    assert_eq!(supported(&session.draft).payments, ordinary.payments);
}
