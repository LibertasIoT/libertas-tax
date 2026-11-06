use super::*;
use crate::tests::support::client::source_calculation;
use crate::tests::support::client::survey::survey_native_refusal as refuse;

fn calculation(request: &P) -> P {
    match request {
        P::SaveDocuments { .. } => request.clone(),
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
        P::SaveMoreCredits { .. } => {
            crate::tests::support::client::credit_documents::more_credits_calculation(request)
        }
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
        P::SaveEmployeeExpenses { allowed_owners, value, .. } => P::EstimateEmployeeExpenses {
            rental_energy_buildings: Vec::new(),
            initial_asset_amt_carry_choices: Vec::new(),
            user: TEST_USER,
            allowed_owners: allowed_owners.clone(),
            value: value.clone(),
        },
        P::SaveOftenMissed { value, .. } => {
            P::EstimateOftenMissed { user: TEST_USER, value: value.clone() }
        }
        _ => panic!("payment, credit, gift, tax, situation, or employee editor required"),
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
    refuse(&alloc::format!("{case}-estimate"), session, calculation(&bad), key, path);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let reply = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        calculation(&corrected),
    );
    assert!(if matches!(corrected, P::SavePayments { .. }) {
        matches!(reply, Reply::Response(P::PaymentsEstimate { .. }))
    } else if matches!(corrected, P::SaveDocuments { .. }) {
        matches!(reply, Reply::Response(P::Review { .. }))
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
        let P::SavePayments { value, .. } = &mut corrected else { unreachable!() };
        value.estimated = crate::EstimatedPaymentEntry::Sources {
            payments: vec![crate::InjuredDatedPayment {
                date: 20260415,
                amount: 1_000,
                payer: PaymentPayer::Taxpayer,
                refund_property: None,
            }],
        };
        value.extension = if present { 10_000 } else { 0 };
        value.more.extension_payer = if present { Some(PaymentPayer::Taxpayer) } else { None };
        let mut bad = corrected.clone();
        let P::SavePayments { value, .. } = &mut bad else { unreachable!() };
        value.more.extension_payer = if present { None } else { Some(PaymentPayer::Taxpayer) };
        case(
            &alloc::format!("admission-payment-extension-{present}"),
            &mut session,
            bad,
            corrected,
            TaxSection::Payments,
            "INJURED_SPOUSE_EXTENSION_PAYER_REQUIRED",
            "[value/more/extension_payer]",
        );
    }
}
#[test]
fn unit47_payment_withholding_uses_real_return_total_and_precedes_farmer_guard() {
    for mutation in 0..3 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::Payments);
        let P::SavePayments { value, .. } = &mut corrected else { unreachable!() };
        let total = value.withheld.as_ref().unwrap().total;
        assert!(total > 0);
        value.more.penalty = penalty(
            WithholdingTiming::ByPeriod { first: total, second: 0, third: 0, fourth: 0 },
            Farmer::No,
            IncomeTiming::Evenly,
        );
        let mut bad = corrected.clone();
        let P::SavePayments { value, .. } = &mut bad else { unreachable!() };
        let PenaltyChoice::Figure { withholding, farmer, .. } = &mut value.more.penalty else {
            unreachable!()
        };
        let WithholdingTiming::ByPeriod { first, fourth, .. } = withholding else { unreachable!() };
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
            *farmer = Farmer::Yes { paid_by_march_1: false };
        }
        case(
            &alloc::format!("admission-payment-withholding-{mutation}"),
            &mut session,
            bad,
            corrected,
            TaxSection::Payments,
            "WITHHOLDING_PERIODS_NOT_TOTAL",
            "[value/more/penalty/withholding]",
        );
    }
}
#[test]
fn unit47_payment_farmer_timing_locates_actual_conflicting_choice() {
    for mutation in 0..3 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open(&mut session, TaxSection::Payments);
        let P::SavePayments { value, .. } = &mut corrected else { unreachable!() };
        let total = value.withheld.as_ref().unwrap().total;
        value.more.penalty = penalty(
            WithholdingTiming::Evenly,
            Farmer::Yes { paid_by_march_1: false },
            IncomeTiming::Evenly,
        );
        let mut bad = corrected.clone();
        let P::SavePayments { value, .. } = &mut bad else { unreachable!() };
        let PenaltyChoice::Figure { withholding, income, .. } = &mut value.more.penalty else {
            unreachable!()
        };
        if mutation != 1 {
            *withholding =
                WithholdingTiming::ByPeriod { first: total, second: 0, third: 0, fourth: 0 };
        }
        if mutation != 0 {
            *income = IncomeTiming::ByPeriod;
        }
        let field = if mutation == 1 {
            "[value/more/penalty/income]"
        } else {
            "[value/more/penalty/withholding]"
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
fn bond(id: &str, source: i64, amount: i64) -> FederalTaxCreditBond1097 {
    let mut record =
        direct_tax_credit_bond(id, FederalTaxCreditBondKind::QualifiedSchoolConstruction, 1_000);
    record.answers.origin = Some(TaxCreditOrigin::EntityStatement(TaxCreditEntity {
        entity: "Synthetic shareholder report".into(),
        designated_income: 200_000,
        income_already_reported: amount,
        income_kind: TaxCreditIncomeKind::RicDistribution,
        income_source: Some(source),
    }));
    record
}
pub(super) fn bond_editor() -> (Interview, P, i64) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let page =
        crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(
                        native_div1099("Synthetic accepted RIC".into(), 100_000, 0),
                    )),
                },
            )],
        );
    accepted(&send(&mut session, page));
    let mut page = crate::tests::support::client::income_pages::open_document_center(&mut session);
    let P::SaveDocuments { documents, credit_income_sources, .. } = &mut page else {
        unreachable!()
    };
    assert_eq!(credit_income_sources.len(), 1);
    let source = credit_income_sources[0].id;
    assert_eq!(credit_income_sources[0].amount, 100_000);
    // Each form prints your name, so it is yours.
    let you = fixture_document_names(&session.draft, -1).unwrap().recipient;
    let mut bonds = vec![bond("FIRST", source, 60_000), bond("SECOND", source, 40_000)];
    documents.other.credits.documents = bonds
        .iter_mut()
        .map(|bond| {
            bond.form.recipient = you.clone();
            crate::CreditDocument::CreditBond(alloc::boxed::Box::new(bond.clone().into()))
        })
        .collect();
    accepted(&send(&mut session, page));
    for bond in &mut bonds {
        bond.id = session
            .draft
            .credits
            .as_ref()
            .unwrap()
            .tax_credit_bonds
            .iter()
            .find(|accepted| accepted.form.unique_identifier == bond.form.unique_identifier)
            .unwrap()
            .id;
    }
    // The second bond's origin is first recorded without its linked source;
    // the page's corrected origin is the save under test.
    let mut first = bonds.clone();
    let Some(TaxCreditOrigin::EntityStatement(entity)) = &mut first[1].answers.origin else {
        unreachable!()
    };
    entity.income_source = None;
    entity.income_already_reported = 0;
    let mut page = crate::tests::support::client::credit_documents::credit_document_answers_request(
        &mut session,
        &first,
        None,
    );
    let Some(TaxCreditOrigin::EntityStatement(intended)) = bonds[1].answers.origin.clone() else {
        unreachable!()
    };
    *crate::tests::support::client::credit_documents::answered_origin(&mut page, 1).1 =
        intended.into();
    (session, page, source)
}
#[test]
fn unit47_form8986_paper_admission_precedes_unanswered_personal_records() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut paper = bba_interest_reduction().into_parts().0;
    paper.partner = fixture_document_names(&session.draft, -1).unwrap().recipient;
    let mut page = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    documents.other.credits.documents =
        vec![crate::CreditDocument::PartnershipAudit(alloc::boxed::Box::new(
            crate::PartnershipAuditRow { id: 0, document: paper },
        ))];
    accepted(&send(&mut session, page));
    let mut expected = session.draft.credits.as_ref().unwrap().bba_audit_interest.clone().unwrap();
    assert_eq!(expected.answers, crate::BbaAuditAnswers::default());

    let mut corrected = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut corrected else { unreachable!() };
    let crate::CreditDocument::PartnershipAudit(paper) = &mut documents.other.credits.documents[0]
    else {
        unreachable!()
    };
    crate::tests::support::builders::set_8986_interest(&mut paper.document, 10_000, -5_000);
    expected.document = paper.document.clone();
    let mut bad = corrected.clone();
    let P::SaveDocuments { documents, .. } = &mut bad else { unreachable!() };
    let crate::CreditDocument::PartnershipAudit(paper) = &mut documents.other.credits.documents[0]
    else {
        unreachable!()
    };
    // Both printed amounts are valid individually, but corrected interest cannot be negative.
    crate::tests::support::builders::set_8986_interest(&mut paper.document, 10_000, -20_000);
    refuse(
        "admission-form8986-pending-answers",
        &mut session,
        bad,
        "ISSUE_BBA_AUDIT_WHOLE_DOLLAR_VALUES",
        "[documents/other/credits/documents/0]",
    );
    let revision = session.draft.revision;
    accepted(&send(&mut session, P::from_avro(&corrected.to_avro()).unwrap()));
    assert_eq!(session.draft.revision, revision + 1);
    assert_eq!(
        session.draft.credits.as_ref().unwrap().bba_audit_interest.as_ref(),
        Some(&expected)
    );
}

#[test]
fn unit47_credit_source_owner_locates_kind_identity_amount_and_aggregate() {
    for mutation in 0..4 {
        let (mut session, corrected, _) = bond_editor();
        let mut bad = corrected.clone();
        let (row, entity) = crate::tests::support::client::credit_documents::answered_origin(&mut bad, 1);
        let path = match mutation {
            0 => {
                entity.income_kind = TaxCreditIncomeKind::Interest;
                alloc::format!("[value/answers/{row}/answer/value/income_kind]")
            }
            1 => {
                entity.income_source = Some(999_999);
                alloc::format!("[value/answers/{row}/answer/value/income_source]")
            }
            2 => {
                entity.income_already_reported = 100_001;
                alloc::format!("[value/answers/{row}/answer/value/income_already_reported]")
            }
            _ => {
                entity.income_already_reported += 1;
                "[value/answers]".into()
            }
        };
        case(
            &alloc::format!("admission-credit-source-{mutation}"),
            &mut session,
            bad,
            corrected,
            TaxSection::MoreCredits,
            "CREDIT_INCOME_SOURCE_INVALID",
            &path,
        );
    }
}
#[test]
fn unit47_credit_source_uses_spouse_assignment_from_the_same_save() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let dividend = named_document(
        &session.draft,
        -2,
        0,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(dividend_document(native_div1099(
                "Synthetic accepted RIC".into(),
                100_000,
                0,
            ))),
        },
    );
    let page =
        crate::tests::support::client::income_pages::save_documents(&mut session, vec![dividend]);
    accepted(&send(&mut session, page));
    let mut page = crate::tests::support::client::income_pages::open_document_center(&mut session);
    let P::SaveDocuments { documents, credit_income_sources, .. } = &mut page else {
        unreachable!()
    };
    assert_eq!(credit_income_sources.len(), 1);
    assert_eq!(credit_income_sources[0].owner, Owner::Spouse);
    assert_eq!(credit_income_sources[0].amount, 100_000);
    let mut bond = bond("SPOUSE", credit_income_sources[0].id, 60_000);
    // One bond: the share confirmation is never asked.
    bond.answers.allowed_credit_confirmed = None;
    documents.other.credits.documents =
        vec![crate::CreditDocument::CreditBond(alloc::boxed::Box::new(bond.clone().into()))];
    accepted(&send(&mut session, page));
    bond.id = session.draft.credits.as_ref().unwrap().tax_credit_bonds[0].id;
    assert!(unknown_documents(&session.draft).contains(&(bond.id, TaxSection::Documents)));
    accepted(&add_documents(
        &mut session,
        TaxSection::Documents,
        -2,
        &[bond.id],
        crate::AddedOwnership::Yours,
    ));
    assert_eq!(session.draft.credits.as_ref().unwrap().tax_credit_bonds[0].owner, Owner::Spouse);
    let page = crate::tests::support::client::credit_documents::credit_document_answers_request(
        &mut session,
        core::slice::from_ref(&bond),
        None,
    );
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let accepted_bond = &saved.credits.as_ref().unwrap().tax_credit_bonds[0];
    assert_eq!(accepted_bond.owner, Owner::Spouse);
    assert_eq!(accepted_bond.form, bond.form);
    assert_eq!(accepted_bond.answers, bond.answers);
    assert!(!unknown_documents(&saved).iter().any(|(id, _)| *id == bond.id));
    let mut cold = restart(saved.clone());
    let page = open(&mut cold, TaxSection::MoreCredits);
    accepted(&send(&mut cold, page));
    assert_eq!(cold.draft, saved);
}

#[test]
fn unit47_credit_source_projection_uses_last_submitted_bond_section() {
    let (mut session, corrected, _) = bond_editor();
    let mut bad = corrected.clone();
    let (row, entity) = crate::tests::support::client::credit_documents::answered_origin(&mut bad, 1);
    entity.income_source = Some(999_999);
    case(
        "admission-credit-last-section",
        &mut session,
        bad,
        corrected,
        TaxSection::MoreCredits,
        "CREDIT_INCOME_SOURCE_INVALID",
        &alloc::format!("[value/answers/{row}/answer/value/income_source]"),
    );
}
#[test]
fn unit47_gift_appreciation_owner_locates_real_carryover_and_source() {
    let (draft, source) = crate::tests::credits::form_3800::retained_charity_draft();
    let mut session = Interview::new(TEST_USER, draft);
    let mut corrected = open(&mut session, TaxSection::GiftCarryovers);
    let P::SaveGiftCarryovers { value, retained_research_choices, .. } = &mut corrected else {
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
    let P::SaveGiftCarryovers { value, .. } = &mut bad else { unreachable!() };
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
        let P::SaveEmployeeExpenses { value, .. } = &mut corrected else { unreachable!() };
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
            improvement_buildings: Vec::new(),
            equipment_assets: vec![],
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
        let P::SaveEmployeeExpenses { value, .. } = &mut bad else { unreachable!() };
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
            crate::tests::support::client::income_pages::candidate_save_review(
                &mut session,
                source_calculation(&corrected)
            ),
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
    let P::SaveEmployeeExpenses { value, .. } = &mut corrected else { unreachable!() };
    value.forms = vec![EmployeeExpenseForm {
        improvement_buildings: Vec::new(),
        equipment_assets: vec![],
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
    let P::SaveEmployeeExpenses { value, .. } = &mut bad else { unreachable!() };
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
fn unit47_unprepared_duplicate_keeps_actual_submitted_record() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut corrected = open(&mut session, TaxSection::OftenMissed);
    let P::SaveOftenMissed { value, .. } = &mut corrected else { unreachable!() };
    value.unprepared =
        Some(vec![Unprepared::AlternativeMinimumTax, Unprepared::HousingCreditRecapture]);
    let mut bad = corrected.clone();
    let P::SaveOftenMissed { value, .. } = &mut bad else { unreachable!() };
    value.unprepared.as_mut().unwrap().push(Unprepared::AlternativeMinimumTax);
    case(
        "admission-unprepared-duplicate",
        &mut session,
        bad,
        corrected,
        TaxSection::OftenMissed,
        "MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE",
        "[value/unprepared/2]",
    );
}

#[test]
fn unit47_payment_sources_preserve_independent_amounts() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let baseline = supported(&session.draft);
    let mut corrected = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut corrected else { unreachable!() };
    value.estimated = crate::EstimatedPaymentEntry::Sources {
        payments: vec![page_paid(20260415, 100_001), page_paid(20260915, 99_999)],
    };
    value.extension = 200_000;
    value.more.extension_payer = Some(PaymentPayer::Taxpayer);
    accepted(&send(&mut session, corrected));
    let result = supported(&session.draft);
    assert_eq!(result.tax, baseline.tax);
    assert_eq!(result.payments, baseline.payments.map(|paid| paid + 400_000));
    assert_eq!(
        crate::kernel::ratio::exact::rounded(
            &session.draft.payments.as_ref().unwrap().estimated_source()
        ),
        Some(200_000)
    );
}

#[test]
fn unit47_payment_unavailable_withholding_keeps_first_composition_open() {
    // 2026-10-16 Source facts correction: missing deductions are unrelated
    // and normal construction fills them. The unfinished actual income page
    // stops first_incomplete before the native cash collector; known filers
    // still permit opening Payments through the ordinary navigation owner.
    let mut draft = complete(FederalStatus::Single);
    draft.finished_sections.retain(|section| *section != TaxSection::Income);
    draft.deductions = None;
    let mut session = Interview::new(TEST_USER, draft);
    let mut corrected = open(&mut session, TaxSection::Payments);
    let P::SavePayments { value, .. } = &mut corrected else { unreachable!() };
    assert_eq!(value.withheld, None);
    // The correction commits a real source edit; submitting the unchanged
    // default IrsFigures answer is correctly a no-op with no new revision.
    value.estimated =
        crate::EstimatedPaymentEntry::Sources { payments: vec![page_paid(20260415, 100)] };
    let mut bad = corrected.clone();
    let P::SavePayments { value, .. } = &mut bad else { unreachable!() };
    value.more.penalty = penalty(
        WithholdingTiming::ByPeriod { first: 0, second: 0, third: 0, fourth: 0 },
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
        "[value/more/penalty/withholding]",
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


// The second bond's origin left open: the questions page's Save answering it.
pub(super) fn bond_question_editor() -> (Interview, P, i64) {
    let (mut session, page, source) = bond_editor();
    accepted(&send(&mut session, page));
    let bonds = session.draft.credits.as_ref().unwrap().tax_credit_bonds.clone();
    let mut open = session.draft.clone();
    open.credits.as_mut().unwrap().tax_credit_bonds[1].answers.origin = None;
    session = restart(open);
    let page = crate::tests::support::client::credit_documents::origin_question(&mut session, &bonds, 1);
    (session, page, source)
}

#[test]
fn unit47_credit_source_first_answer_locates_kind_identity_amount_and_aggregate() {
    for mutation in 0..4 {
        let (mut session, corrected, _) = bond_question_editor();
        let mut bad = corrected.clone();
        let entity = crate::tests::support::client::credit_documents::origin_entity(&mut bad);
        let path = match mutation {
            0 => {
                entity.income_kind = TaxCreditIncomeKind::Interest;
                "[questions/0/answer/value/income_kind]"
            }
            1 => {
                entity.income_source = Some(999_999);
                "[questions/0/answer/value/income_source]"
            }
            2 => {
                entity.income_already_reported = 100_001;
                "[questions/0/answer/value/income_already_reported]"
            }
            _ => {
                entity.income_already_reported += 1;
                "[questions]"
            }
        };
        crate::tests::support::client::credit_documents::origin_case(
            &alloc::format!("admission-credit-first-answer-{mutation}"),
            &mut session,
            bad,
            corrected,
            "CREDIT_INCOME_SOURCE_INVALID",
            path,
        );
    }
}

#[test]
fn unit47_credit_source_first_answer_names_its_bond() {
    // The refusal names the bond whose origin answer broke the source rule.
    let (mut session, corrected, _) = bond_question_editor();
    let mut bad = corrected.clone();
    crate::tests::support::client::credit_documents::origin_entity(&mut bad).income_source =
        Some(999_999);
    crate::tests::support::client::credit_documents::origin_case(
        "admission-credit-first-answer-section",
        &mut session,
        bad,
        corrected,
        "CREDIT_INCOME_SOURCE_INVALID",
        "[questions/0/answer/value/income_source]",
    );
}
