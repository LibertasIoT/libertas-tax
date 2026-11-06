use super::support::assert::{accepted, rejected_with};
use super::*;
use crate::tests::cross::interview::{
    assert_round_trip, asset, at_the_credits_hub, business, credits_hub, employer_credit_business,
    health_credit_business, historic_claim, hub_rows, hub_tail, more_credits_value,
    open_credits_topic, paid_leave_business, save_businesses, send_past_less_common, solar_asset,
    stored_business,
};

// The credits hub's last topic, one list item per FederalCredits field it exposes.

#[test]
fn each_more_credit_variant_reaches_its_field() {
    // Every variant maps onto its own FederalCredits field(s), each in the
    // struct's declaration order (more_credit_items); saving clears every
    // other field this page owns and leaves education, care, saver,
    // marketplace and the hidden student/marketplace-dependent catalogs
    // untouched.
    let (mut s, hub) = at_the_credits_hub(complete(FederalStatus::Single));
    let before = s.draft.credits.clone().unwrap();
    let page = open_credits_topic(&mut s, TaxSection::MoreCredits);
    assert!(more_credits_value(&page).credits.is_empty());
    // More credits is the final topic; Continue is an action.
    let rows = hub_rows(&hub);
    assert_eq!(rows[rows.len() - 1..], hub_tail(&[], TaxSection::MoreCredits));
    let mut source_page = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut source_page else { unreachable!() };
    let mut bond = FederalTaxCreditBond1097 {
        printed: None,
        id: 0,
        owner: Owner::Taxpayer,
        form: btc_paper(crate::Form1097Btc {
            issuer: crate::PrintedParty {
                federal_id: "123456789".into(),
                ..crate::PrintedParty::from_name("Muni".into())
            },
            recipient: fixture_document_names(&s.draft, -1).unwrap().recipient,
            unique_identifier: "ABC123".into(),
            box3_code_199: true,
            box1_credit: 10_00,
        }),
        answers: crate::TaxCreditBondAnswers {
            kind: Some(FederalTaxCreditBondKind::QualifiedEnergyConservation),
            issued_on: Some(20100101),
            prior_carry_dollars: 0,
            allowed_credit_dollars: None,
            allowed_credit_confirmed: Some(false),
            prior_carry_confirmed: Some(false),
            origin: Some(crate::TaxCreditOrigin::Direct(crate::TaxCreditDirect {
                directly_held_full_year: true,
                direct_issuer_report: true,
                no_direct_pay_election: true,
                deemed_interest_not_elsewhere: true,
                separate_interest_entered: true,
                ordinary_investment_interest: true,
            })),
        },
    };
    let mut audit = FederalBbaAuditInterest {
        partnership: Default::default(),
        reviewed_year_partner: Default::default(),
        id: 0,
        source_id: "TRK123".into(),
        furnished_on: 20260301,
        interest_adjustment: 10_00,
        original_partnership_interest: 5_00,
        prior_wages: 10_000,
        prior_taxable_income: 80_00,
        prior_income_tax: 9_00,
        prior_status: FederalStatus::Single,
        single_audit_adjustment: true,
        simple_prior_return: true,
        no_other_effects: true,
        line_17_interest: 0,
        interest_calculation_prepared: false,
    };
    documents.other.credits.documents = vec![
        crate::CreditDocument::CreditBond(alloc::boxed::Box::new(bond.clone().into())),
        crate::CreditDocument::PartnershipAudit(alloc::boxed::Box::new(
            crate::tests::support::builders::audit_row(audit.clone()),
        )),
    ];
    let P::SaveDocuments { documents, .. } = &mut source_page else { unreachable!() };
    let recipient = fixture_document_names(&s.draft, -1).unwrap().recipient;
    for source in &mut documents.other.credits.documents {
        match source {
            crate::CreditDocument::CreditBond(value) => value.form.recipient = recipient.clone(),
            crate::CreditDocument::PartnershipAudit(value) => {
                value.document.partner = recipient.clone()
            }
        }
    }
    accepted(&send(&mut s, source_page));
    let fresh = s.draft.credits.as_ref().unwrap();
    let fresh_bond = &fresh.tax_credit_bonds[0];
    assert_eq!(
        (
            fresh_bond.answers.kind,
            fresh_bond.answers.issued_on,
            fresh_bond.answers.origin.as_ref(),
            fresh_bond.answers.prior_carry_dollars,
            fresh_bond.answers.allowed_credit_dollars,
        ),
        (None, None, None, 0, None),
    );
    assert_eq!(
        fresh.bba_audit_interest.as_ref().unwrap().answers,
        crate::BbaAuditAnswers::default()
    );
    assert!(s.draft.valid_stored());
    let pending =
        crate::interview::remaining_items(&s.draft, &estimate(&s.draft), TaxSection::MoreCredits);
    for key in ["DOCUMENT_TAX_CREDIT_BOND_ANSWERS", "ISSUE_BBA_AUDIT_TAX_TABLE_STATUS"] {
        let row = pending
            .iter()
            .find(|row| {
                let message = crate::Message::decode(&row.item);
                message.key == "DOCUMENT_PENDING_ANSWER"
                    && message.args.first() == Some(&crate::MessageArg::Resource(key.into()))
            })
            .expect("the accepted paper leaves its personal answers pending");
        assert_eq!(row.section, TaxSection::MoreCredits);
    }
    bond.id = s.draft.credits.as_ref().unwrap().tax_credit_bonds[0].id;
    audit.id = s.draft.credits.as_ref().unwrap().bba_audit_interest.as_ref().unwrap().id;
    let request = crate::tests::support::client::credit_documents::credit_document_answers_request(
        &mut s,
        &[bond],
        Some(&audit),
    );
    accepted(&send(&mut s, request));
    let sources_before = s.draft.credits.clone().unwrap();
    let items = vec![
        MoreCredit::MinimumTax(
            crate::MinimumTaxHistories::from_current_unit(MinimumTaxHistory {
                return_form: crate::MinimumTaxReturnForm::Form1040,
                status: FederalStatus::Single,
                base: 0,
                exclusions: Vec::new(),
                regular: 0,
                amt: 0,
                carry: 10_000,
                gains: None,
                regular_nol: 0,
                mtcnol: crate::MinimumTaxLoss::None,
                foreign_earned: None,
                foreign_credit: crate::MinimumTaxForeignCredit::None,
                old_vehicle_credit: 0,
            })
            .into(),
        ),
        MoreCredit::MortgageCredit {
            current: None,
            carry_three_years_prior: 0,
            carry_two_years_prior: 0,
            carry_prior_year: 50_00,
            loan: None,
        },
        MoreCredit::PriorCreditCarryovers {
            residential_energy_carry: Some(20_00),
            dc_homebuyer_carry: Some(10_00),
        },
        MoreCredit::CleanVehiclesAndRefueling {
            clean_vehicles: None,
            personal_refueling: vec![FederalPersonalRefueling {
                label: "Home charger".into(),
                placed_in_service: 20260101,
                cost: 10_000,
                census_geoid: "12345678901".into(),
                eligible_tract: true,
                qualified_property: true,
                personal_only: true,
            }],
        },
        MoreCredit::Adoption {
            adoptions: Vec::new(),
            adoption_carry: vec![AdoptionCarry { year: 2023, amount: 50_00, eligible: true }],
        },
        MoreCredit::ElderlyOrDisabled {
            exempt_pensions: vec![ExemptPension { label: "VA pension".into(), amount: 40_00 }],
        },
        MoreCredit::ForeignTax(ForeignTaxPage::Statements { eligible: true }),
        MoreCredit::ElectricVehicleCarryover { legacy_ev_passive_allowed: Some(30_00) },
        MoreCredit::EmployerTip(EmployerTipFicaCarryforward {
            prior_status: FederalStatus::Single,
            original_credit: 10_00,
            used_origin_year: 5_00,
            used_origin_prior_year_carryback: 0,
            carryforward_to_current_year: 5_00,
            ordinary: true,
        }),
        MoreCredit::EducationRecapture(PriorYearEducationCreditRecapture {
            method: EducationMethod::AmericanOpportunity,
            filed_qualified_expenses: 50_000,
            after_filing_reduction: 10_000,
            filed_education_magi: 600_000,
            filed_joint: false,
            filed_credit_used: 25_000,
            ordinary_prior_return: true,
        }),
        MoreCredit::AfterDisallowance(CreditsAfterDisallowance {
            earned_income: EicDisallowance::Disallowed {
                facts: DisallowanceFacts {
                    year: 2023,
                    finding: DisallowanceFinding::Ineligible,
                    reinstated: false,
                },
                cause: EicDisallowanceCause::Other,
                your_us_home_days: None,
                spouse_us_home_days: None,
            },
            child_credits: CreditDisallowance::Never,
            education_credit: CreditDisallowance::Never,
        }),
    ];
    let mut request = open_credits_topic(&mut s, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value, .. } = &mut request else { unreachable!() };
    value.credits = items.clone();
    assert!(matches!(send_past_less_common(&mut s, request), Reply::Response(P::Review { .. })));
    credits_hub(crate::tests::cross::interview::from_review(&mut s, TaxSection::Credits));
    let credits = s.draft.credits.clone().unwrap();
    // Each field reached its own value; nothing this page does not own moved.
    assert_eq!(credits.minimum_tax.as_ref().unwrap().returns[0].facts.carry, 10_000);
    assert_eq!(credits.mortgage_credit.as_ref().unwrap().carry_prior_year, 50_00);
    assert_eq!(credits.residential_energy_carry, Some(20_00));
    assert_eq!(credits.dc_homebuyer_carry, Some(10_00));
    assert!(credits.clean_vehicles.is_none());
    assert_eq!(credits.personal_refueling.len(), 1);
    assert!(credits.adoptions.is_empty());
    assert_eq!(
        credits.adoption_carry,
        vec![AdoptionCarry { year: 2023, amount: 50_00, eligible: true }]
    );
    assert_eq!(credits.exempt_pensions.len(), 1);
    assert_eq!(credits.foreign_tax, ForeignTaxClaim::Statements { eligible: Some(true) });
    assert_eq!(credits.tax_credit_bonds.len(), 1);
    assert_eq!(credits.legacy_ev_passive_allowed, Some(30_00));
    assert_eq!(credits.tip_fica_carryforward.as_ref().unwrap().carryforward_to_current_year, 5_00);
    assert_eq!(credits.bba_audit_interest.as_ref().unwrap().document.source_id(), "TRK123");
    assert_eq!(credits.education_recapture.as_ref().unwrap().filed_credit_used, 25_000);
    assert!(matches!(
        credits.after_disallowance.as_ref().unwrap().earned_income,
        EicDisallowance::Disallowed { facts: DisallowanceFacts { year: 2023, .. }, .. }
    ));
    // Untouched: education, care, saver, marketplace and the catalogs.
    assert_eq!(credits.education, before.education);
    assert_eq!(credits.care, before.care);
    assert_eq!(credits.saver, before.saver);
    assert_eq!(credits.marketplace, before.marketplace);
    assert!(s.draft.finished_sections.contains(&TaxSection::MoreCredits));
    // The synthetic Form 8978 source values do not reconcile. Correction
    // returns to the topic that owns those entered prior-year source facts.
    let result = estimate(&s.draft);
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.display.key == "ISSUE_BBA_AUDIT_PRIOR_INCOME_RECONCILIATION")
        .expect("the entered prior-year figures must still be checked");
    assert_eq!(
        crate::interview::issue_pages(&s.draft, &issue.context, None),
        vec![TaxSection::MoreCredits],
        "the topic owns every entered source figure named by this issue"
    );
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        unreachable!()
    };
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::MoreCredits));
    let hub = credits_hub(crate::tests::cross::interview::from_review(&mut s, TaxSection::Credits));
    assert!(
        hub_rows(&hub)
            .iter()
            .any(|row| row.2 == TaxSection::MoreCredits && row.1.starts_with("Needs attention: "))
    );
    // Reopening lists every item back, in the same declaration order.
    let page = open_credits_topic(&mut s, TaxSection::MoreCredits);
    assert_eq!(more_credits_value(&page).credits, items);
    // Removing every item clears every field this page owns.
    let mut request = open_credits_topic(&mut s, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value, .. } = &mut request else { unreachable!() };
    value.credits.clear();
    send(&mut s, request);
    let credits = s.draft.credits.as_ref().unwrap();
    assert_eq!(credits.minimum_tax, None);
    assert_eq!(credits.mortgage_credit, None);
    assert_eq!(credits.residential_energy_carry, None);
    assert_eq!(credits.dc_homebuyer_carry, None);
    assert!(credits.personal_refueling.is_empty());
    assert!(credits.adoption_carry.is_empty());
    assert!(credits.exempt_pensions.is_empty());
    assert_eq!(credits.foreign_tax, ForeignTaxClaim::No);
    assert_eq!(credits.tax_credit_bonds, sources_before.tax_credit_bonds);
    assert_eq!(credits.legacy_ev_passive_allowed, None);
    assert_eq!(credits.tip_fica_carryforward, None);
    assert_eq!(credits.bba_audit_interest, sources_before.bba_audit_interest);
    assert_eq!(credits.education_recapture, None);
}

#[test]
fn employer_credits_round_trip_through_the_business_page() {
    // The credits hand fixture entered on the page (Form 5884 line 2 $6,350;
    // Form 8881 lines 5 and 9 $2,500 and $500), no carryovers. Receipts
    // $24,150 = wages $30,000 - $6,350 + set-up $3,000 - $2,500: net profit
    // $0; tax on $43,900 single = $5,023. Form 3800: Part I $3,000 (line 17,
    // under line 16 $5,023); line 29 = $2,023 of the $6,350 specified credit;
    // line 38 = $5,023; the 2026 Form 5884 carries $4,327 to 2027.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let b = employer_credit_business();
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    assert_eq!(data.work_opportunity_credit.as_ref().unwrap().credit(), 6_350);
    assert_eq!(data.retirement_plan_credit.as_ref().unwrap().startup_credit(), 2_500);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "business_2_net_profit"), 0);
    assert_eq!(value(&r, "form_3800_line_17"), 3_000);
    assert_eq!(value(&r, "form_3800_line_37"), 2_023);
    assert_eq!(value(&r, "general_business_credit"), 5_023);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year_to_next_year"), 4_327);
    assert_round_trip(&mut session, &[b]);
}

#[test]
fn employer_credit_page_rules_are_checked_on_save() {
    // Exercise the actual native Save owner; declarations are server-only.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = employer_credit_business();
    b.more.payroll = BusinessPayroll::No;
    rejected_with(save_businesses(&mut session, vec![b]), "EMPLOYER_CREDITS_NEED_PAYROLL");
    let mut b = employer_credit_business();
    if let Some(claim) = &mut b.more.work_opportunity {
        claim.employees[1].label = "Employee A".into();
    }
    rejected_with(save_businesses(&mut session, vec![b]), "WOTC_EMPLOYEE_TWICE");
    let mut b = employer_credit_business();
    if let Some(claim) = &mut b.more.work_opportunity {
        claim.employees[0].began = 20260102;
    }
    rejected_with(save_businesses(&mut session, vec![b]), "WOTC_BEGAN_AFTER_2025");
}

// The employee rows of Forms 5884, 8846, 8941 and 8994 choose their W-2 from
// the business's own payroll list (an EnumSource keyed on the W-2 label); Form
// 8881 lists no employees. The client cannot submit a label the payroll lacks,
// and the server repeats that.
#[test]
fn employer_credit_rows_choose_their_w2_from_the_payroll() {
    // The schema carries the keyed payroll list and the four hosts.
    let model = include_str!("../../federal/extended_model.rs");
    assert!(model.contains("#[libertas_key]\n    #[libertas_unique(1)]\n    #[libertas_size(min = 1)]\n    pub label: String,"));
    let credits = include_str!("../../credits/general_business_credit.rs");
    assert_eq!(
        credits.matches("#[libertas_enum_source(\"^^^^.payroll.Yes.statements\")]").count(),
        4
    );
    // All five employer credits on one business, saved and reopened.
    let mut b = employer_credit_business();
    let mut months =
        vec![EmployerTipMonth { hours: 0.0, wages_excluding_tips: 0, reported_tips: 0 }; 12];
    months[9] =
        EmployerTipMonth { hours: 100.0, wages_excluding_tips: 37_500, reported_tips: 45_000 };
    let tip_fica = EmployerTipFicaCredit {
        service: EmployerTipService::FoodOrBeverage,
        employees: vec![EmployerTipEmployee { label: "Employee A".into(), months }],
        actual_employer_tip_fica_paid: 3_444,
        used_prior_year_carryback: 0,
        confirmed: true,
    };
    b.more.tip_fica = Some((&tip_fica).into());
    let mut health = SmallEmployerHealthCredit::from(
        &health_credit_business().more.small_employer_health.unwrap(),
    );
    health.employees = vec![
        HealthCreditEmployee {
            label: "Employee B".into(),
            service: HealthCreditService::Hours { hours: 2_080 },
            coverage: Some(EmployeeHealthCoverage {
                tier: HealthCoverageTier::EmployeeOnly,
                months: 12,
                total_premium: 720_000,
                employer_paid: 504_000,
            }),
        },
        HealthCreditEmployee {
            label: "Employee C".into(),
            service: HealthCreditService::Seasonal,
            coverage: None,
        },
    ];
    b.more.small_employer_health = Some((&health).into());
    let paid_leave = PaidLeaveCredit {
        employees: vec![PaidLeaveEmployee {
            label: "Employee C".into(),
            weekly_hours: 40,
            prior_year_compensation: 5_200_000,
            normal_hourly_rate: 2_500,
            leave_hours: 80.0,
            leave_wages: 150_000,
            rate: 75.0,
        }],
        insurance: None,
        confirmed: true,
    };
    b.more.paid_leave = Some((&paid_leave).into());
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    assert!(
        data.work_opportunity_credit.is_some()
            && data.retirement_plan_credit.is_some()
            && data.employer_tip_fica_credit.is_some()
            && data.small_employer_health_credit.is_some()
            && data.paid_leave_credit.is_some()
    );
    assert_round_trip(&mut session, &[b.clone()]);
    // A row whose W-2 the payroll does not list is refused for each credit.
    let rejects = |session: &mut Interview, edit: &dyn Fn(&mut BusinessDetails)| {
        let mut changed = b.clone();
        edit(&mut changed.more);
        rejected_with(
            save_businesses(session, vec![changed]),
            "EMPLOYER_CREDIT_EMPLOYEE_NOT_ON_PAYROLL",
        );
    };
    rejects(&mut session, &|m| {
        m.work_opportunity.as_mut().unwrap().employees[2].label = "Nobody".into()
    });
    rejects(&mut session, &|m| m.tip_fica.as_mut().unwrap().employees[0].label = "Nobody".into());
    rejects(&mut session, &|m| {
        m.small_employer_health.as_mut().unwrap().employees[1].label = "Nobody".into()
    });
    rejects(&mut session, &|m| m.paid_leave.as_mut().unwrap().employees[0].label = "Nobody".into());
    // Renaming a W-2 the rows use leaves them unanswered: refused too.
    rejects(&mut session, &|m| {
        let BusinessPayroll::Yes { statements, .. } = &mut m.payroll else { panic!() };
        statements[0].label = "Employee Z".into();
    });
}

#[test]
fn paid_leave_credit_round_trips_through_the_business_page_and_reduces_lines_26_and_14() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let b = paid_leave_business();
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (id, data) = stored_business(&session);
    let claim = data.paid_leave_credit.as_ref().unwrap();
    assert_eq!(claim.form_lines(), [1_281, 900]);
    let r = supported(&session.draft);
    assert_eq!(value(&r, &alloc::format!("business_{id}_form8994_line3")), 2_181);
    assert_eq!(value(&r, &alloc::format!("business_{id}_expense_line26_wages")), 102_719);
    assert_eq!(
        value(&r, &alloc::format!("business_{id}_expense_line14_employee_benefit_programs")),
        7_100
    );
    assert_eq!(value(&r, &alloc::format!("business_{id}_net_profit")), 190_181);
    assert_eq!(value(&r, "business_credit_paid_leave_current_year"), 2_181);
    assert_round_trip(&mut session, &[b]);
    // An unconfirmed record is left out with its notice and the deductions
    // stay whole.
    let mut b = paid_leave_business();
    b.more.paid_leave.as_mut().unwrap().confirmed = false;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_PAID_LEAVE_CREDIT_LEFT_OUT"));
    assert_eq!(value(&r, &alloc::format!("business_{id}_expense_line26_wages")), 104_000);
    assert_round_trip(&mut session, &[b]);
}
fn building_asset(claim: crate::RehabilitationCredit) -> Asset {
    Asset {
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class: AssetClass::ThirtyNine,
                history: AssetHistory::New { first_year: FirstYearDeduction::Regular },
            },
        },
        investment_credit: crate::InvestmentCredit::Rehabilitation(claim),
        ..asset(30_000_000, 20260415, AssetClass::ThirtyNine)
    }
}

#[test]
fn the_investment_credit_asset_row_round_trips_and_its_rules_reject_on_save() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(40_000_000);
    b.assets =
        vec![building_asset(historic_claim(20240501, 30_000_000, true)), solar_asset(20250915)];
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (id, data) = stored_business(&session);
    assert!(matches!(data.assets[0].investment_credit, crate::InvestmentCredit::Rehabilitation(_)));
    assert_round_trip(&mut session, &[b.clone()]);
    let r = supported(&session.draft);
    assert_eq!(value(&r, &alloc::format!("business_{id}_asset_0_form3468_credit")), 12_000);
    assert_eq!(value(&r, &alloc::format!("business_{id}_asset_1_form3468_credit")), 48_000);
    assert_eq!(value(&r, "business_credit_rehabilitation_current_year"), 12_000);
    assert_eq!(value(&r, "business_credit_clean_electricity_current_year"), 48_000);
    // The page rules reject on save, with the same keys as the client.
    let before = session.draft.clone();
    let mut reject = |asset: Asset, key: &str| {
        let mut page = business(40_000_000);
        page.assets = vec![asset];
        rejected_with(save_businesses(&mut session, vec![page]), key);
    };
    reject(
        building_asset(historic_claim(20240501, 19_000_000, true)),
        "REHABILITATION_NOT_SUBSTANTIAL",
    );
    reject(
        building_asset(historic_claim(20231201, 30_000_000, true)),
        "REHABILITATION_PERIOD_NOT_IN_SERVICE_YEAR",
    );
    reject(
        building_asset(historic_claim(20260415, 30_000_000, true)),
        "REHABILITATION_PERIOD_AFTER_PLACED",
    );
    reject(
        Asset {
            recovery: crate::AssetRecoveryPage::Gds {
                facts: crate::GdsAssetRecovery {
                    macrs_question_context: None,
                    source: None,
                    class: AssetClass::Seven,
                    history: AssetHistory::New { first_year: FirstYearDeduction::Regular },
                },
            },
            ..building_asset(historic_claim(20240501, 30_000_000, true))
        },
        "REHABILITATION_NEEDS_A_BUILDING_ROW",
    );
    reject(
        {
            let mut page = solar_asset(20250915);
            page.gds_mut().unwrap().class = AssetClass::Seven;
            page
        },
        "ENERGY_CREDIT_NEEDS_NEW_5_YEAR_PROPERTY",
    );
    reject(solar_asset(20260502), "ENERGY_CONSTRUCTION_AFTER_PLACED");
    let mut earlier = Asset {
        investment_credit: crate::InvestmentCredit::EarlierEnergy {
            section: crate::EnergySection::Energy,
            credit: 1_000_000,
            used: 900_000,
            carried: 200_000,
            freed: 0,
            unchanged: true,
        },
        ..asset(12_000_000, 20260601, AssetClass::Five)
    };
    reject(earlier.clone(), "EARLIER_ENERGY_NEEDS_CONTINUING_5_YEAR_PROPERTY");
    earlier.placed = 20250601;
    earlier.gds_mut().unwrap().history = AssetHistory::Continuing {
        prior_depreciation: 0,
        first_year_179: 0,
        first_year_bonus: 0,
        carryforward: 0,
        mid_quarter: false,
    };
    reject(earlier, "INVESTMENT_CREDIT_HISTORY_ABOVE_CREDIT");
    assert_eq!(session.draft, before);
}

#[test]
fn the_rehabilitation_credit_history_is_checked_against_the_building_s_credit_period() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // Placed in service in 2024: the credit years are 2024 and 2025.
    let mut claim = historic_claim(20220401, 20_000_000, true);
    claim.building_basis = 15_000_000;
    claim.earlier = vec![
        crate::CreditYear { year: 2024, used: 800_000, carried: 0 },
        crate::CreditYear { year: 2025, used: 500_000, carried: 300_000 },
    ];
    let continuing = |claim: crate::RehabilitationCredit| Asset {
        placed: 20240310,
        cost: 20_000_000,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class: AssetClass::ThirtyNine,
                history: AssetHistory::Continuing {
                    prior_depreciation: 735_520,
                    first_year_179: 0,
                    first_year_bonus: 0,
                    carryforward: 0,
                    mid_quarter: false,
                },
            },
        },
        investment_credit: crate::InvestmentCredit::Rehabilitation(claim),
        ..asset(20_000_000, 20240310, AssetClass::ThirtyNine)
    };
    let mut b = business(40_000_000);
    b.assets = vec![continuing(claim.clone())];
    accepted(&save_businesses(&mut session, vec![b]));
    let mut reject = |claim: crate::RehabilitationCredit, key: &str| {
        let mut page = business(40_000_000);
        page.assets = vec![continuing(claim)];
        rejected_with(save_businesses(&mut session, vec![page]), key);
    };
    let mut twice = claim.clone();
    twice.earlier[1].year = 2024;
    reject(twice, "INVESTMENT_CREDIT_YEAR_TWICE");
    let mut early = claim.clone();
    early.earlier[0].year = 2023;
    reject(early, "INVESTMENT_CREDIT_YEAR_OUTSIDE_PERIOD");
    // 4% of the $200,000 expenditures is $8,000 a year.
    let mut above = claim.clone();
    above.earlier[1].carried = 300_001;
    reject(above, "INVESTMENT_CREDIT_HISTORY_ABOVE_CREDIT");
}

#[test]
fn new_clean_vehicle_business_native_save_reopen_reject_and_restart() {
    let mut draft = clean_vehicle_business_return();
    let claim = draft.credits.as_mut().unwrap().clean_vehicles.take().unwrap();
    let (mut session, _) = at_the_credits_hub(draft);
    let mut page = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value: answers, clean_vehicle_properties, .. } = &mut page else {
        unreachable!()
    };
    let FederalCleanVehicle::New { business_use: Some(use_), .. } = &claim.vehicles[0] else {
        unreachable!()
    };
    assert!(clean_vehicle_properties.iter().any(|choice| choice.id == use_.property));
    answers.credits.push(MoreCredit::CleanVehiclesAndRefueling {
        clean_vehicles: Some(claim.clone().into()),
        personal_refueling: vec![],
    });
    accepted(&send(&mut session, page));
    assert_eq!(value(&supported(&session.draft), "form_8936_business_part"), 4_500);
    let saved = session.draft.clone();
    let reopened = open_credits_topic(&mut session, TaxSection::MoreCredits);
    let P::SaveMoreCredits { value: facts, .. } = &reopened else { unreachable!() };
    assert!(facts.credits.iter().any(|row| matches!(row, MoreCredit::CleanVehiclesAndRefueling { clean_vehicles: Some(saved), .. } if crate::FederalCleanVehicleClaim::from(saved.clone()) == claim)));
    for invalid in [0, saved.income[0].id] {
        let mut bad = reopened.clone();
        let P::SaveMoreCredits { value: facts, .. } = &mut bad else { unreachable!() };
        let MoreCredit::CleanVehiclesAndRefueling { clean_vehicles: Some(claim), .. } = facts
            .credits
            .iter_mut()
            .find(|row| matches!(row, MoreCredit::CleanVehiclesAndRefueling { .. }))
            .unwrap()
        else {
            unreachable!()
        };
        let crate::FederalCleanVehiclePage::New { business_use: Some(use_), .. } =
            &mut claim.vehicles[0]
        else {
            unreachable!()
        };
        use_.property = invalid;
        rejected_with(send(&mut session, bad), "CLEAN_VEHICLE_PROPERTY_INVALID");
        assert_eq!(session.draft, saved);
    }
    for fraction in
        [crate::UseFraction { used: 1, total: 0 }, crate::UseFraction { used: 2, total: 1 }]
    {
        let mut bad = reopened.clone();
        let P::SaveMoreCredits { value: facts, .. } = &mut bad else { unreachable!() };
        let MoreCredit::CleanVehiclesAndRefueling { clean_vehicles: Some(claim), .. } = facts
            .credits
            .iter_mut()
            .find(|row| matches!(row, MoreCredit::CleanVehiclesAndRefueling { .. }))
            .unwrap()
        else {
            unreachable!()
        };
        let crate::FederalCleanVehiclePage::New { business_use: Some(use_), .. } =
            &mut claim.vehicles[0]
        else {
            unreachable!()
        };
        use_.credit_use = fraction;
        assert!(!crate::clean_vehicle::valid(&crate::FederalCleanVehicleClaim::from(
            claim.clone()
        )));
        rejected_with(send(&mut session, bad), "CLEAN_VEHICLE_USE_FRACTION_INVALID");
        assert_eq!(session.draft, saved);
    }
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let mut session = restart(session.draft.clone());
    let reopened = open_credits_topic(&mut session, TaxSection::MoreCredits);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert_eq!(supported(&session.draft).tax, Some(282_600));
}

#[test]
fn clean_vehicle_returned_disposition_native_save_reopen_and_restart() {
    for used in [false, true] {
        let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Single));
        let mut vehicle = if used { used_clean_vehicle() } else { new_clean_vehicle() };
        let details = match &mut vehicle {
            FederalCleanVehicle::New { details, .. }
            | FederalCleanVehicle::PreviouslyOwned { details, .. } => details,
        };
        details.disposition = crate::CleanVehicleDisposition::ReturnedWithin30Days;
        details.dealer_transfer = Some(if used { 400_000 } else { 750_000 });
        let claim = clean_vehicle_claim(vec![vehicle]);
        let mut page = open_credits_topic(&mut session, TaxSection::MoreCredits);
        let P::SaveMoreCredits { value: answers, .. } = &mut page else { unreachable!() };
        answers.credits.push(MoreCredit::CleanVehiclesAndRefueling {
            clean_vehicles: Some(claim.clone().into()),
            personal_refueling: vec![],
        });
        accepted(&send(&mut session, page));
        let saved = session.draft.clone();
        let reopened = open_credits_topic(&mut session, TaxSection::MoreCredits);
        let P::SaveMoreCredits { value: facts, .. } = &reopened else { unreachable!() };
        assert!(facts.credits.iter().any(|row| matches!(row,
            MoreCredit::CleanVehiclesAndRefueling { clean_vehicles: Some(saved), .. }
            if crate::FederalCleanVehicleClaim::from(saved.clone()) == claim)));
        accepted(&send(&mut session, reopened));
        let mut session = restart(session.draft);
        let reopened = open_credits_topic(&mut session, TaxSection::MoreCredits);
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        let result = supported(&session.draft);
        assert_eq!(return_line(&result, "17"), 0);
        assert!(noted(&result, "NOTE_CLEAN_VEHICLE_RETURNED_DEALER_RECOVERS_TRANSFER"));
    }
}

// AI Import shape of the Smith family's Form 8986 (round 3, walk group 9):
// Part II item G prints the furnishing date, so the paper carries it and the
// More credits page does not ask for it again.
#[test]
fn form_8986_furnishing_date_is_read_from_the_paper_not_asked() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut source_page = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut source_page else { unreachable!() };
    let mut paper = crate::tests::support::builders::audit_row(crate::FederalBbaAuditInterest {
        partnership: crate::PrintedParty {
            name: "Example Creek Partners LLC".into(),
            federal_id: "00-0000302".into(),
            address: "610 Example Partnership Lane\nExample Grove VA 00000".into(),
        },
        reviewed_year_partner: crate::PrintedParty {
            federal_id: "987-65-4320".into(),
            ..fixture_document_names(&s.draft, -1).unwrap().recipient
        },
        id: 0,
        source_id: "SYNTH8986JOHN2026".into(),
        furnished_on: 20261015,
        interest_adjustment: 20_00,
        original_partnership_interest: 50_00,
        prior_wages: 0,
        prior_taxable_income: 0,
        prior_income_tax: 0,
        prior_status: FederalStatus::Single,
        single_audit_adjustment: false,
        simple_prior_return: false,
        no_other_effects: false,
        line_17_interest: 0,
        interest_calculation_prepared: false,
    })
    .document;
    paper.partner.federal_id = "987-65-4320".into();
    documents.other.credits.documents = vec![crate::CreditDocument::PartnershipAudit(
        alloc::boxed::Box::new(crate::PartnershipAuditRow { id: 0, document: paper.clone() }),
    )];
    accepted(&send(&mut s, source_page));
    let source = s.draft.credits.as_ref().unwrap().bba_audit_interest.as_ref().unwrap();
    assert_eq!(source.document.furnished_on, 20261015);
    assert_eq!(source.document.source_id(), paper.source_id());
    assert_eq!(source.answers, crate::BbaAuditAnswers::default());
    let pending =
        crate::interview::remaining_items(&s.draft, &estimate(&s.draft), TaxSection::MoreCredits);
    let keys: Vec<_> = pending
        .iter()
        .map(|row| crate::Message::decode(&row.item))
        .filter(|message| message.key == "DOCUMENT_PENDING_ANSWER")
        .filter_map(|message| message.args.first().cloned())
        .collect();
    assert!(!keys.contains(&crate::MessageArg::Resource("ISSUE_BBA_AUDIT_FURNISHED_YEAR".into())));
    assert!(keys.contains(&crate::MessageArg::Resource("ISSUE_BBA_AUDIT_TAX_TABLE_STATUS".into())));
}
