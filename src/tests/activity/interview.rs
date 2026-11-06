use crate::tests::support::client::write_native_draft;
// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::support::client::unit50_interview_native::NativeInterviewCase;
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::document;
use super::support::client::income_pages::{
    catalog_estimate_documents, catalog_preview_document, open_topic, page_documents, save_topic,
    topic_for,
};
use crate::tests::cross::interview::{
    PAGE_PEOPLE, accepted_linked_mortgage, assert_rentals_round_trip, at_the_deductions_page,
    business, business_details, cash_farm, exchange_fold, farm_more, fixture_improvement,
    historic_claim, hub_rows, k1_1065, k1_1120s, k1_rental_boxes, linked_mortgage_editor,
    no_rental_expenses, professional_status, rental_details, rental_page, rental_sold_page,
    reopened_k1s, reopened_rentals, save_1065s, save_checklist, save_linked_mortgage_rows,
    save_rentals, save_spouse, save_you, stored_rentals,
};

#[test]
fn unit19_held_interest_actual_two_rows_save_reopen_restart() {
    extern crate std;
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(1_000, 0)));
    investment_borrowing(&mut d, 1, 0, 0);
    let baseline = supported(&d);
    let ticks = crate::interview::with_ticks(d.clone()).checklist;
    let mut session = Interview::new(TEST_USER, d);
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let checklist = save_checklist(&session.draft, PAGE_PEOPLE, ticks);
    accepted(&send(&mut session, checklist));
    let seed = crate::tests::support::client::income_pages::open_document_center(&mut session);
    if NativeInterviewCase("TAX_UNIT19_HELD").required_calculation(&mut session) {
        return;
    }
    let rows = ["First actual source", "Second actual source"]
        .into_iter()
        .map(|name| {
            let mut page = k1_1065(name);
            (page.participation, page.limited_for_se) = (
                Some(crate::PartnerParticipation::General {
                    facts: (Participation::Over500Hours { hours: 501 }).record(),
                }),
                Some(false),
            );
            page.more.deductions = vec![PartnerDeduction {
                code: PartnerDeductionCode::InvestmentInterest,
                amount: 149,
            }];
            let record = page.record().regular;
            page.amt = Some(PartnerAmtRecords::Separate { record });
            document(0, IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) })
        })
        .collect();
    let request = crate::tests::support::client::income_pages::save_documents(&mut session, rows);
    if let Ok(directory) = std::env::var("TAX_UNIT19_HELD_SEED_DIR") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join("seed.avro"), seed.to_avro()).unwrap();
        std::fs::write(directory.join("answers.avro"), request.to_avro()).unwrap();
        return;
    }
    let request = std::env::var("TAX_UNIT19_HELD_PAYLOAD")
        .map_or(request, |file| P::from_avro(&std::fs::read(file).unwrap()).unwrap());
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        P::from_avro(&request.to_avro()).unwrap(),
    ));
    let saved = session.draft.clone();
    let sources: Vec<_> = saved
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Partnership { .. }))
        .collect();
    assert_eq!(sources.len(), 2);
    assert_ne!(sources[0].id, sources[1].id);
    for entry in &sources {
        let FederalIncome::Partnership { data } = &entry.income else { unreachable!() };
        for record in [&data.regular, data.alternative.as_ref().unwrap()] {
            let ledger = data.pool(record).unwrap().activity.ledger;
            assert_eq!(
                crate::kernel::ratio::dollars(
                    ledger.basis_allowed[K1LossClass::InvestmentInterest].get()
                ),
                0
            );
            assert_eq!(
                crate::kernel::ratio::dollars(
                    ledger.basis_suspended[K1LossClass::InvestmentInterest].get()
                ),
                1
            );
            assert_eq!(data.outcome(record).unwrap().0.investment_interest, 0);
        }
    }
    let result = supported(&saved);
    assert_eq!(value(&result, "investment_form_line1"), 1);
    assert_eq!(value(&result, "investment_form_line8"), 1);
    assert_eq!(value(&result, "amt_investment_form_line1"), 1);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
    assert_eq!(result.tax, baseline.tax);
    let mut session = restart(saved.clone());
    let reopened = crate::tests::support::client::income_pages::open_document_center(&mut session);
    if let Ok(directory) = std::env::var("TAX_UNIT19_HELD_OUTPUT_DIR") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join("reopened.avro"), reopened.to_avro()).unwrap();
        write_native_draft(directory.join("draft.avro"), &saved);
    }
    let request = std::env::var("TAX_UNIT19_HELD_REOPEN_PAYLOAD")
        .map_or(reopened, |file| P::from_avro(&std::fs::read(file).unwrap()).unwrap());
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, saved);
    assert_eq!(restart(session.draft.clone()).draft, saved);
}

#[test]
fn clearing_qbi_group_preserves_ungrouped_activities_after_storage_and_reopening() {
    fn stored_group(draft: &FederalDraft) -> &Option<alloc::string::String> {
        match &draft.income[0].income {
            FederalIncome::Business { data } => &data.qbi_aggregation_group,
            FederalIncome::Rental { data } => &data.qbi_aggregation_group,
            other => panic!("expected an activity, got {other:?}"),
        }
    }
    for doc in [
        IncomeSource::Record { value: crate::IncomeRecord::Business(business(100_000)) },
        IncomeSource::Record { value: crate::IncomeRecord::Rental(rental_page(100_000)) },
        IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(100_000, 0)) },
    ] {
        let topic = topic_for(&doc);
        let draft = empty_return(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, draft);
        let request = save_topic(&mut session, topic, vec![document(0, doc)]);
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(stored_group(&session.draft), &None);
        match &mut session.draft.income[0].income {
            FederalIncome::Business { data } => {
                data.qbi_aggregation_group = Some("Named group".into())
            }
            FederalIncome::Rental { data } => {
                data.qbi_aggregation_group = Some("Named group".into())
            }
            _ => unreachable!(),
        }
        session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
        let page = open_topic(&mut session, topic);
        let mut row = page_documents(&page)[0].clone();
        let (label, group) = match &mut row.document {
            IncomeSource::Record {
                value: crate::IncomeRecord::Business(Business { name, more, .. }),
            } => (name, &mut more.qbi_group),
            IncomeSource::Record {
                value: crate::IncomeRecord::Rental(Rental { address, more, .. }),
            } => (address, &mut more.qbi_group),
            IncomeSource::Record { value: crate::IncomeRecord::Farm(Farm { name, more, .. }) } => {
                (name, &mut more.qbi_group)
            }
            other => panic!("expected an activity with a group, got {other:?}"),
        };
        assert!(label.is_empty());
        assert_eq!(group, "Named group");
        group.clear();
        let request = save_topic(&mut session, topic, vec![row]);
        let reply = send(&mut session, request);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        session.draft = FederalDraft::from_avro(&session.draft.to_avro()).unwrap();
        assert_eq!(stored_group(&session.draft), &None);
        let page = open_topic(&mut session, topic);
        match &page_documents(&page)[0].document {
            IncomeSource::Record {
                value: crate::IncomeRecord::Business(Business { more, .. }),
            } if *more == business_details() => {}
            IncomeSource::Record { value: crate::IncomeRecord::Rental(Rental { more, .. }) }
                if *more == rental_details() => {}
            IncomeSource::Record { value: crate::IncomeRecord::Farm(Farm { more, .. }) }
                if *more == farm_more() => {}
            other => panic!("expected default activity details after clearing, got {other:?}"),
        }
    }
}
fn rental_expense(category: RentalExpenseCategory, amount: i64) -> RentalExpense {
    RentalExpense { category, amount, treatment: BusinessExpenseKind::Ordinary }
}

#[test]
fn rental_lines_and_building_map_onto_federal_rental() {
    // The m3-self-employed-rental fixture's rental on a single return with
    // the $60,000 W-2 of `complete`. Schedule E by hand:
    //   line 3 rent $24,000; lines 9 $1,400, 11 $2,400, 12 $9,600, 14 $1,800,
    //   16 $3,200, 17 $600 (total $19,000);
    //   line 18: ($260,000 - $40,000 land) / 27.5 years = $8,000 (in service
    //   since 2021, a full year's mid-month rate);
    //   line 21: $24,000 - $19,000 - $8,000 = -$3,000.
    //   Form 8582: MAGI $60,000 is under $100,000, so the full $25,000
    //   active-participation allowance applies and the whole loss is allowed.
    //   Form 1040 line 8 (Schedule 1 line 5) -$3,000; line 9 $60,000 - $3,000
    //   = $57,000.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let page = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { allowed_documents, documents, .. } = &page else { unreachable!() };
    assert_eq!(
        allowed_documents,
        &vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Royalties) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Rental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32
        ]
    );
    assert!(documents.documents.is_empty());
    // The fixture touches ten fields on this page: the rent, six expense
    // lines and the building's cost, land and date. The address, type,
    // personal use and the "more" answers keep their defaults.
    let mut r = rental_page(2_400_000);
    r.expenses = RentalExpenses {
        insurance: 140_000,
        management: 240_000,
        mortgage_interest: 960_000,
        repairs: 180_000,
        taxes: 320_000,
        utilities: 60_000,
        ..no_rental_expenses()
    };
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let request = crate::tests::cross::interview::navigate(&session.draft, TaxSection::Income);
    let Reply::Response(hub) = send(&mut session, request) else {
        panic!("expected the income hub")
    };
    // M3-P19: the row carries the count and the rental's line 21 preview.
    assert!(
        hub_rows(&hub).iter().any(|(item, status, section)| item == "Rental property"
            && status.starts_with("1 rental · ")
            && *section == TaxSection::Rentals),
        "{:?}",
        hub_rows(&hub)
    );
    let stored = stored_rentals(&session);
    assert_eq!(stored.len(), 1);
    let (id, label, data) = &stored[0];
    assert_eq!(label, "");
    assert_eq!(
        *data,
        FederalRental {
            interest_limit_election: None,
            location: crate::RentalLocation::UnitedStates,
            property_qbi_history: Vec::new(),
            rent: 2_400_000,
            expenses: vec![
                rental_expense(RentalExpenseCategory::Insurance, 140_000),
                rental_expense(RentalExpenseCategory::ManagementFees, 240_000),
                rental_expense(RentalExpenseCategory::MortgageInterest, 960_000),
                rental_expense(RentalExpenseCategory::Repairs, 180_000),
                rental_expense(RentalExpenseCategory::TaxesAndLicenses, 320_000),
                rental_expense(RentalExpenseCategory::Utilities, 60_000),
            ],
            buildings: vec![purchased_rental_building(26_000_000, 4_000_000, 20210301)],
            ..rental(0, 0)
        }
    );
    let r_result = supported(&session.draft);
    let v = |name: &str| value(&r_result, &alloc::format!("rental_{id}_{name}"));
    assert_eq!(v("expense_line9_insurance"), 1_400);
    assert_eq!(v("expense_line11_management_fees"), 2_400);
    assert_eq!(v("expense_line12_mortgage_interest"), 9_600);
    assert_eq!(v("expense_line14_repairs"), 1_800);
    assert_eq!(v("expense_line16_taxes"), 3_200);
    assert_eq!(v("expense_line17_utilities"), 600);
    assert_eq!(v("depreciation"), 8_000);
    assert_eq!(value(&r_result, "rental_income"), -3_000);
    assert_eq!(return_line(&r_result, "8"), -300_000);
    assert_eq!(return_line(&r_result, "9"), 5_700_000);
    // The row's preview is line 21 before the passive limits.
    assert_eq!(reopened_rentals(session.draft.clone()), vec![(r.clone(), Some(-300_000))]);
    assert_rentals_round_trip(&mut session, &[r.clone()]);

    // The fold: an improvement, suspended losses, last year's home-use
    // carryovers, QBI treatment, a group, payroll-free and at risk.
    let mut folded = r.clone();
    folded.address = "12 Sample Lane".into();
    folded.expenses.other =
        vec![OtherExpense { description: "Pest control".into(), amount: 25_000 }];
    //2026-10-19: synthetic prior worksheets explicitly establish these independent records.
    folded.prior_home_carry = Some(crate::RentalHomeCarries {
        home_depreciation_carryover: crate::SourceCents::Reported { amount: 20_000 },
        amt_home_depreciation_carryover: crate::SourceCents::Reported { amount: 20_000 },
        home_casualty_carryover: crate::SourceCents::NotReported,
        amt_home_casualty_carryover: crate::SourceCents::NotReported,
    });
    folded.more = RentalDetails {
        energy_buildings: Vec::new(),
        property_qbi_history: Vec::new(),
        improvements: vec![Improvement {
            commercial_1993_transition: None,
            allowed_commercial_1993_transition: Vec::new(),
            casualty: None,
            casualty_property: None,
            id: 0,
            energy: None,
            sale_history: None,
            credit_reduction: 0,
            cost: 1_500_000,
            in_service: 20240601,
            rehabilitation: None,
            partial: None,
        }],
        participation: crate::RentalParticipation::Active,
        suspended: SuspendedLoss { regular: 120_000, amt: 100_000, active_in_loss_year: true },
        method_changes: vec![],
        home_operating_carryover: 30_000,
        qbi: RentalBusiness::Yes { material_participation: false },
        qbi_group: "Rentals".into(),
        payroll: BusinessPayroll::No,
        at_risk: Some(crate::RentalAtRisk::All),
        known_at_risk: vec![crate::RentalAtRisk::All],
        special: false,
    };
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(
            *id,
            IncomeSource::Record { value: crate::IncomeRecord::Rental(folded.clone()) },
        )],
    );
    accepted(&send(&mut session, request));
    let (_, label, data) = &stored_rentals(&session)[0];
    assert_eq!(label, "12 Sample Lane");
    assert_eq!(
        data.expenses.last(),
        Some(&rental_expense(
            RentalExpenseCategory::Other { label: "Pest control".into() },
            25_000
        ))
    );
    assert_eq!(
        data.buildings,
        vec![
            purchased_rental_building(26_000_000, 4_000_000, 20210301),
            purchased_rental_building(1_500_000, 0, 20240601),
        ]
    );
    assert_eq!(
        data.history,
        RentalHistory { regular: 120_000, amt: 100_000, ..RentalHistory::NONE }
    );
    // Carryovers with no personal use this year: a plain No beside the
    // carryovers, which stay limited to the rent left (section 280A(c)(5)).
    assert_eq!(data.personal_use, RentalPersonalUse::No);
    assert_eq!(
        (data.home_operating_carryover, data.home_carries().home_depreciation_carryover.clone()),
        (30_000, crate::SourceCents::Reported { amount: 20_000 })
    );
    assert_eq!(data.qbi_business, RentalBusiness::Yes { material_participation: false });
    assert_eq!(data.qbi_aggregation_group.as_deref(), Some("Rentals"));
    assert_eq!(
        (data.participation, data.at_risk.clone(), data.ordinary_rental),
        (crate::RentalParticipation::Active, Some(crate::RentalAtRisk::All), true)
    );
    assert_rentals_round_trip(&mut session, &[folded]);
}

#[test]
fn vacation_rental_type_is_computed_and_round_trips() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_page(2_400_000);
    r.kind = RentalPropertyType::VacationShortTerm { stays: crate::ShortTermStays::OverSevenDays };
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let (_, _, data) = &stored_rentals(&session)[0];
    assert_eq!(data.property_type, r.kind);
    supported(&session.draft);
    assert_rentals_round_trip(&mut session, &[r]);
}

#[test]
fn real_estate_professional_is_a_fact_of_the_person_on_the_people_pages() {
    // Inventor decision 2026-10-09 (F3): the status belongs to the person,
    // so the Rentals topic never offers it and no rental is needed to hold it.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let P::SaveIncomes { allowed_documents, .. } = open_topic(&mut session, TaxSection::Rentals)
    else {
        unreachable!()
    };
    assert_eq!(
        allowed_documents,
        vec![
            libertas_macros::variant_index!(crate::IncomeSourceKind::Royalties) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::Rental) as i32,
            libertas_macros::variant_index!(crate::IncomeSourceKind::BusinessPropertyLosses) as i32
        ]
    );
    assert_eq!(professional_status(&session, Owner::Taxpayer), None);
    // You record it on your page; the 500-hour answer is the investment-tax
    // safe harbor and reopens with the page, also after a restart.
    for over_500_hours in [true, false] {
        let status = ProfessionalRentals { over_500_hours, single_activity: false };
        let mut you = crate::interview::filer(&session.draft.people.as_ref().unwrap().taxpayer);
        you.details.real_estate_professional = Some(status.clone().into());
        let request = save_you(&session.draft.clone(), you);
        accepted(&send(&mut session, request));
        assert_eq!(professional_status(&session, Owner::Taxpayer), Some(status.clone()));
        assert_eq!(professional_status(&session, Owner::Spouse), None);
        let reopened = crate::interview::filer(&session.draft.people.as_ref().unwrap().taxpayer);
        assert_eq!(reopened.details.real_estate_professional, Some(status.clone().into()));
        let restarted = restart(session.draft.clone());
        assert_eq!(professional_status(&restarted, Owner::Taxpayer), Some(status));
    }
    // Your spouse records it on the spouse page, independently.
    let mut spouse =
        crate::interview::filer(session.draft.people.as_ref().unwrap().spouse.as_ref().unwrap());
    spouse.details.real_estate_professional =
        Some(ProfessionalRentals { single_activity: false, over_500_hours: true }.into());
    let request = save_spouse(&session.draft.clone(), spouse);
    accepted(&send(&mut session, request));
    assert_eq!(
        professional_status(&session, Owner::Spouse),
        Some(ProfessionalRentals { single_activity: false, over_500_hours: true })
    );
    assert!(professional_status(&session, Owner::Taxpayer).is_some());
    // Clearing it on your page leaves your spouse's.
    let mut you = crate::interview::filer(&session.draft.people.as_ref().unwrap().taxpayer);
    you.details.real_estate_professional = None;
    let request = save_you(&session.draft.clone(), you);
    accepted(&send(&mut session, request));
    assert_eq!(professional_status(&session, Owner::Taxpayer), None);
    assert!(professional_status(&session, Owner::Spouse).is_some());
}

#[test]
fn a_rental_chooses_professional_participation_only_for_an_owner_with_the_status() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_page(2_400_000);
    r.more.participation = crate::RentalParticipation::Professional;
    // Without the recorded status the save is refused, and nothing changes.
    let before = session.draft.clone();
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(r.clone()) })],
    );
    rejected_with(
        current_income_refusal(
            "income-source-1111",
            &mut session,
            request,
            "RENTAL_PROFESSIONAL_NEEDS_STATUS",
            "[documents/documents/0/document/more/participation]",
        ),
        "RENTAL_PROFESSIONAL_NEEDS_STATUS",
    );
    assert_eq!(session.draft, before);
    // With it the rental is stored as Professional and reopens unchanged.
    make_professional(&mut session.draft, Owner::Taxpayer, false);
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    assert_eq!(
        stored_rentals(&session)[0].2.participation,
        crate::RentalParticipation::Professional
    );
    assert_rentals_round_trip(&mut session, &[r]);
}

#[test]
fn qbi_loss_carryforward_kind_reaches_form_8995() {
    // Form 8995: line 3 prior-year qualified business net loss carryforward
    // (-$30,000) and line 7 REIT/PTP loss carryforward (-$4,000). With no
    // 2026 QBI and no REIT dividends, lines 4 and 6 are negative, the
    // deduction (line 15) is $0, and lines 16 and 17 carry $30,000 and
    // $4,000 to 2027 unchanged.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let entered =
        QbiLossCarryforward { business: 3_000_000, reit_ptp: 400_000, injured_owners: None };
    let request = save_topic(
        &mut s,
        TaxSection::SelfEmployment,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::QbiLossCarryforward(entered.clone()),
            },
        )],
    );
    accepted(&send(&mut s, request));
    assert_eq!(
        s.draft.screening.as_ref().unwrap().qbi_history,
        QbiHistory { business: 3_000_000, reit_ptp: 400_000, injured_owners: None }
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 30_000);
    assert_eq!(value(&r, "reit_ptp_loss_carryforward"), 4_000);
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::SelfEmployment)),
        &[IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::QbiLossCarryforward(entered.clone())
                }
            )
        }]
    );
    rejected_with(
        {
            let request = save_topic(
                &mut s,
                TaxSection::SelfEmployment,
                vec![
                    document(
                        0,
                        IncomeSource::Record {
                            value: crate::IncomeRecord::QbiLossCarryforward(entered)
                        }
                    );
                    2
                ],
            );
            send(&mut s, request)
        },
        "QBI_LOSS_CARRYFORWARD_TWICE",
    );
    // Leaving it out returns the history to none.
    accepted(&{
        let request = save_topic(&mut s, TaxSection::SelfEmployment, vec![]);
        send(&mut s, request)
    });
    assert_eq!(s.draft.screening.as_ref().unwrap().qbi_history, QbiHistory::NONE);
}

#[test]
fn rental_sale_fold_maps_and_its_rules_reject_on_both_sides() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let sale = |date, price, land_price| crate::RentalSale::Sold {
        date,
        price,
        land_price,
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let mut r = rental_page(1_200_000);
    r.sale = sale(20260815, 40_000_000, 8_000_000);
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let (_, _, data) = stored_rentals(&session).remove(0);
    assert_eq!(data.sale, sale(20260815, 40_000_000, 8_000_000));
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::Rentals)).to_vec();
    assert_eq!(items[0].document, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) });
    let before = session.draft.clone();
    // The land part above the price, and a sale before the building was
    // available to rent (the fixture building's March 1, 2021).
    let mut r = rental_page(1_200_000);
    r.sale = sale(20260815, 40_000_000, 40_000_001);
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_LAND_PRICE_ABOVE_PRICE");
    let mut r = rental_page(1_200_000);
    r.building.in_service = 20260901;
    r.sale = sale(20260815, 40_000_000, 8_000_000);
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_SOLD_BEFORE_IN_SERVICE");
    assert_eq!(session.draft, before);
}

// Form 1099-S on a rental's sale or exchange: the complete printed paper is
// retained while the rental's sale and use facts stay on the topic page.
fn rental_statement(closed: u32, proceeds: i64, property_received: bool) -> crate::Form1099SPaper {
    form_1099s_paper(crate::Form1099S {
        filer: crate::PrintedParty::default(),
        transferor: crate::PrintedParty::default(),
        address: "12 Elm Street".into(),
        closed,
        proceeds,
        property_received,
        foreign_person: false,
    })
}

#[test]
fn rental_sale_statement_maps_to_the_sold_date_and_price_and_rejects_a_mismatch() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_sold_page(20260815, 40_000_000);
    let mut statement = rental_statement(20260815, 40_000_000, false);
    statement.box_4_buyers_part_of_real_estate_tax = 125_000;
    statement.filer.name = "Example Closing Company".into();
    statement.transferor.name = "Example Owner".into();
    r.statement = Some(statement);
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let (_, _, data) = stored_rentals(&session).remove(0);
    assert_eq!(data.statement, r.statement);
    let mut again = restart(session.draft.clone());
    let items = page_documents(&open_topic(&mut again, TaxSection::Rentals)).to_vec();
    assert_eq!(items[0].document, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) });
    let before = session.draft.clone();
    let mut r = rental_sold_page(20260815, 40_000_000);
    r.statement = Some(rental_statement(20260816, 40_000_000, false));
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_STATEMENT_DATE_DIFFERS");
    let mut r = rental_sold_page(20260815, 40_000_000);
    r.statement = Some(rental_statement(20260815, 40_000_001, false));
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_STATEMENT_PROCEEDS_DIFFER");
    assert_eq!(session.draft, before);
}

#[test]
fn rental_exchange_statement_maps_to_the_transfer_date_only() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_page(1_200_000);
    r.sale = crate::RentalSale::Exchanged(exchange_fold());
    // The proceeds are not compared: the amount realized comes from what was received.
    r.statement = Some(rental_statement(20260620, 1, true));
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let (_, _, data) = stored_rentals(&session).remove(0);
    assert_eq!(data.statement, r.statement);
    let before = session.draft.clone();
    r.statement = Some(rental_statement(20260621, 1, true));
    rejected_with(save_rentals(&mut session, vec![r]), "RENTAL_STATEMENT_DATE_DIFFERS");
    assert_eq!(session.draft, before);
}

#[test]
fn rental_sale_statement_with_property_received_notes_the_exchange_choice() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_sold_page(20260815, 40_000_000);
    r.statement = Some(rental_statement(20260815, 40_000_000, true));
    accepted(&save_rentals(&mut session, vec![r]));
    let result = estimate(&session.draft);
    assert!(noted(&result, "WARN_RENTAL_SALE_PROPERTY_RECEIVED"), "{result:?}");
    // The statement never selects the choice: the rental is still a sale.
    let (_, _, data) = stored_rentals(&session).remove(0);
    assert!(matches!(data.sale, crate::RentalSale::Sold { .. }));
    // Box 6 unchecked, or an exchange, raises nothing.
    let mut r = rental_sold_page(20260815, 40_000_000);
    r.statement = Some(rental_statement(20260815, 40_000_000, false));
    accepted(&save_rentals(&mut session, vec![r]));
    assert!(!noted(&estimate(&session.draft), "WARN_RENTAL_SALE_PROPERTY_RECEIVED"));
}

#[test]
fn rental_improvements_are_stored_building_rows_with_no_land_and_no_label() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original);
    let mut rental = rental_page(2_400_000);
    rental.more.improvements = vec![fixture_improvement(), fixture_improvement()];
    accepted(&save_rentals(&mut session, vec![rental]));
    let buildings = &stored_rentals(&session)[0].2.buildings;
    // The building you bought, then one stored row for each improvement; an
    // improvement carries no land and the rows carry no name (F3, 2026-10-09).
    assert_eq!(buildings.len(), 3);
    assert!(buildings[1..].iter().all(|b| b.land == 0));
    assert!(buildings[0].rehabilitation.is_none());
    // An unnamed rental stores no name.
    assert_eq!(stored_rentals(&session)[0].1, "");
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn a_rental_improvement_takes_the_rehabilitation_credit_and_the_building_page_has_no_field_for_it()
{
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_page(1_200_000);
    r.kind = RentalPropertyType::Commercial;
    r.more.improvements = vec![Improvement {
        commercial_1993_transition: None,
        allowed_commercial_1993_transition: Vec::new(),
        casualty: None,
        casualty_property: None,
        id: 0,
        energy: None,
        sale_history: None,
        credit_reduction: 0,
        cost: 15_000_000,
        in_service: 20260701,
        rehabilitation: Some(historic_claim(20240701, 15_000_000, true)),
        partial: None,
    }];
    // The building's own basis is $100,000 less than the expenditures here.
    if let Some(claim) = &mut r.more.improvements[0].rehabilitation {
        claim.building_basis = 10_000_000;
    }
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let (_, _, stored) = stored_rentals(&session).remove(0);
    assert!(stored.buildings[0].rehabilitation.is_none());
    assert!(stored.buildings[1].rehabilitation.is_some());
    assert_rentals_round_trip(&mut session, &[r.clone()]);
    let before = session.draft.clone();
    // The credit is only for an improvement placed in service in 2018 or later;
    // the building you bought has no rehabilitation field at all (F3).
    let mut old = r.clone();
    old.more.improvements[0].in_service = 20171231;
    old.more.improvements[0].rehabilitation.as_mut().unwrap().period_start = 20150101;
    rejected_with(save_rentals(&mut session, vec![old]), "REHABILITATION_NEEDS_AN_IMPROVEMENT_ROW");
    let mut not_substantial = r;
    if let Some(claim) = &mut not_substantial.more.improvements[0].rehabilitation {
        claim.period_expenditures = 9_000_000;
    }
    rejected_with(
        save_rentals(&mut session, vec![not_substantial]),
        "REHABILITATION_NOT_SUBSTANTIAL",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn rental_mortgage_links_protect_roots_and_checklist_pruning() {
    let (mut s, _) = at_the_deductions_page(FederalStatus::Single);
    let root = accepted_linked_mortgage(&mut s);
    let mut v = rental_page(0);
    v.related = crate::tests::support::builders::related_forms::related_keys(&[root.id]);
    v.personal_use = PersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 60,
        overlap_days: 0,
        direct_expenses: 0,
    };
    for (personal_use, eligible) in [(PersonalUse::No, false), (v.personal_use.clone(), true)] {
        let Some(Reply::Response(P::RentalRelatedChoices { candidates })) =
            s.handle(P::RentalRelatedOptions {
                user: TEST_USER,
                id: 0,
                owner: Owner::Taxpayer,
                personal_use,
            })
        else {
            panic!("expected provisional rental forms")
        };
        assert_eq!(candidates.iter().any(|source| source.id == root.id), eligible);
    }
    let expected_loan = mortgage_loan(&s.draft.deductions.as_ref().unwrap().mortgage[0]);
    let Some(Reply::Response(P::FormViewed { form: RelatedForm::Mortgage1098(loan), .. })) =
        s.handle(P::ViewRelatedForm { user: TEST_USER, key: root.id })
    else {
        panic!("expected the whole saved mortgage form")
    };
    assert_eq!(loan, expected_loan);
    let reply = save_rentals(&mut s, vec![v.clone()]);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let before = s.draft.clone();
    let P::SaveMortgages { rental_mortgage_loans, .. } = linked_mortgage_editor(&mut s) else {
        unreachable!()
    };
    assert_eq!(rental_mortgage_loans, vec![root.id]);
    rejected_with(save_linked_mortgage_rows(&mut s, vec![]), "MORTGAGE_RENTAL_ROOT_NEEDED");
    assert_eq!(s.draft, before);
    let mut renamed = root.clone();
    renamed.more.property = "Renamed linked home".into();
    let reply = save_linked_mortgage_rows(&mut s, vec![renamed]);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(crate::extended_rules::rental_mortgage_roots(&s.draft), vec![root.id]);
    // Unchecking the rental removes it; its Form 1098 stays, as every
    // document does.
    let mut ticks = s.draft.checklist;
    ticks.activities &= !ActivityTopics::RENTALS.0;
    let request = save_checklist(&s.draft, PAGE_PEOPLE, ticks);
    let reply = send(&mut s, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    assert_eq!(s.draft.deductions.as_ref().unwrap().mortgage.len(), 1);
    accepted(&save_linked_mortgage_rows(&mut s, vec![]));
    assert!(s.draft.deductions.as_ref().unwrap().mortgage.is_empty());
    assert!(crate::extended_rules::rental_mortgage_roots(&s.draft).is_empty());
}
#[test]
fn rental_mortgage_links_reject_echoed_forgery_in_save_estimate_and_preview() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut v = rental_page(0);
    v.personal_use = PersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 60,
        overlap_days: 0,
        direct_expenses: 0,
    };
    v.related = crate::tests::support::builders::related_forms::related_keys(&[987654]);
    let before = s.draft.clone();
    let fake = RecordChoice {
        id: 987654,
        label: libertas::libertas_formatted_text("MORTGAGE_PROPERTY_UNNAMED", &[]),
    };
    let mut request = save_topic(
        &mut s,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(v.clone()) })],
    );
    if let P::SaveIncomes { mortgage_loans, .. } = &mut request {
        mortgage_loans.push(fake.clone());
    }
    rejected_with(send(&mut s, request), "PROBLEM_RELATED_FORM_UNAVAILABLE");
    let P::SaveIncomes { followups, .. } = open_topic(&mut s, TaxSection::Rentals) else {
        unreachable!()
    };
    let request = {
        let mut request = catalog_estimate_documents(
            TaxSection::Rentals,
            (IncomeRecords {
                qbi_aggregations: Vec::new(),
                documents: vec![document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Rental(v.clone()) },
                )]
                .into_iter()
                .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
                .collect(),
            })
            .into(),
            followups,
        );
        let P::EstimateDocuments {
            allowed_owners,
            allowed_community,
            mortgage_loans,
            foreign_countries,
            interest_statements,
            ..
        } = &mut request
        else {
            unreachable!()
        };
        *allowed_owners = vec![0];
        *allowed_community = vec![];
        *mortgage_loans = vec![fake.clone()];
        *foreign_countries = vec![];
        *interest_statements = vec![];
        request
    };
    rejected_with(
        crate::tests::support::client::income_pages::candidate_save_review(&mut s, request),
        "PROBLEM_RELATED_FORM_UNAVAILABLE",
    );
    let request = {
        let mut request = catalog_preview_document(IncomeSource::Record {
            value: crate::IncomeRecord::Rental(v),
        });
        let P::PreviewDocument { mortgage_loans, foreign_countries, .. } = &mut request else {
            unreachable!()
        };
        *mortgage_loans = vec![fake];
        *foreign_countries = vec![];
        request
    };
    rejected_with(send(&mut s, request), "PROBLEM_RELATED_FORM_UNAVAILABLE");
    assert_eq!(s.draft, before);
}

// ---- Unit B11 4797: the disposition pages ----

#[test]
fn rental_page_keeps_the_condemnation_the_partial_disposition_and_the_buyer() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_page(2_400_000);
    r.building.in_service = 20160701;
    r.building.land = 0;
    r.building.partial =
        Some(crate::PartialDisposition { date: 20260410, cost_of_part: 1_100_000, proceeds: 0 });
    r.sale = crate::RentalSale::Condemned {
        date: 20260915,
        award: 40_000_000,
        land_award: 9_000_000,
        expenses: 500_000,
        replacement_cost: 38_000_000,
    };
    accepted(&save_rentals(&mut session, vec![r.clone()]));
    let stored = stored_rentals(&session);
    assert_eq!(stored[0].2.sale, r.sale);
    assert_eq!(stored[0].2.buildings[0].partial, r.building.partial);
    let reopened = reopened_rentals(session.draft.clone());
    assert_eq!(reopened[0].0.sale, r.sale);
    assert_eq!(reopened[0].0.building.partial, r.building.partial);
    // A land part above the award is refused on the server side too.
    let mut over = rental_page(2_400_000);
    over.sale = crate::RentalSale::Condemned {
        date: 20260915,
        award: 1_000_000,
        land_award: 1_000_001,
        expenses: 0,
        replacement_cost: 0,
    };
    rejected_with(save_rentals(&mut session, vec![over]), "CONDEMNATION_LAND_ABOVE_AWARD");
    // A part larger than the building, a related buyer on an installment sale
    // and a missing personal-use share are refused by name.
    let mut big = rental_page(2_400_000);
    big.building.partial =
        Some(crate::PartialDisposition { date: 20260410, cost_of_part: 26_000_000, proceeds: 0 });
    rejected_with(save_rentals(&mut session, vec![big]), "PARTIAL_DISPOSITION_PART_ABOVE_COST");
    let mut related = rental_page(2_400_000);
    related.sale = crate::RentalSale::Sold {
        date: 20260915,
        price: 30_000_000,
        land_price: 4_000_000,
        selling_expenses: 0,
        payment: crate::SalePayment::Installment {
            assumed_debt: 0,
            payments: 1_000_000,
            interest: crate::NoteInterest::Adequate,
            special: false,
        },
        buyer: crate::Buyer::Relative,
        personal_share_percent: 0,
    };
    rejected_with(save_rentals(&mut session, vec![related]), "INSTALLMENT_RELATED_BUYER");
    let mut home = rental_page(2_400_000);
    home.personal_use = PersonalUse::Yes {
        fair_rental_days: 200,
        personal_days: 30,
        overlap_days: 0,
        direct_expenses: 0,
    };
    home.sale = crate::RentalSale::Sold {
        date: 20260915,
        price: 30_000_000,
        land_price: 4_000_000,
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    rejected_with(save_rentals(&mut session, vec![home]), "RENTAL_SALE_PERSONAL_SHARE_NEEDED");
}

#[test]
fn unit4_professional_active_choices_survive_source_page_save_restart_and_reopen() {
    for participation in
        [crate::RentalParticipation::Professional, crate::RentalParticipation::ProfessionalActive]
    {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        make_professional(&mut session.draft, Owner::Taxpayer, false);
        let mut page = rental_page(1_400_000);
        page.more.participation = participation;
        accepted(&save_rentals(&mut session, vec![page.clone()]));
        let mut session = restart(session.draft);
        assert_eq!(stored_rentals(&session)[0].2.participation, participation);
        assert_rentals_round_trip(&mut session, &[page]);
    }
    for participation in
        [K1RentalParticipation::Professional, K1RentalParticipation::ProfessionalActive]
    {
        let mut page = k1_1065("Synthetic professional partnership");
        page.rentals = k1_rental_boxes(400_000, 0);
        page.rentals.participation = Some(participation);
        let mut d = complete(FederalStatus::Single);
        make_professional(&mut d, Owner::Taxpayer, false);
        let (session, reply) = save_1065s(&d, vec![page.clone()]);
        accepted(&reply);
        let mut session = restart(session.draft);
        assert_eq!(
            reopened_k1s(&mut session),
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }]
            )
        );
        let mut page = k1_1120s("Synthetic professional corporation");
        page.rentals = k1_rental_boxes(400_000, 0);
        page.rentals.participation = Some(participation);
        let mut session = Interview::new(TEST_USER, d);
        accepted(&{
            let request = crate::tests::support::client::income_pages::save_documents(
                &mut session,
                vec![document(
                    0,
                    IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) },
                )],
            );
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        });
        let mut session = restart(session.draft);
        assert_eq!(
            reopened_k1s(&mut session),
            expected_named_sources(
                &session.draft,
                Owner::Taxpayer,
                vec![IncomeSource::K1 { value: crate::K1Document::K1Scorp((page).into()) }]
            )
        );
    }
}

#[test]
fn unit16_r14_rental_basis_reduction_round_trips_and_save_rejects_excess() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = rental_page(2_400_000);
    page.building.cost = 2_000_000;
    page.building.land = 200_000;
    page.building.credit_reduction = 400_000;
    let mut improvement = fixture_improvement();
    improvement.cost = 1_000_000;
    improvement.credit_reduction = 200_000;
    page.more.improvements = vec![improvement];
    accepted(&save_rentals(&mut session, vec![page.clone()]));
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    assert_rentals_round_trip(&mut session, &[page.clone()]);
    page.building.credit_reduction = 1_800_001;
    rejected_with(
        save_rentals(&mut session, vec![page.clone()]),
        "RENTAL_CREDIT_REDUCTION_ABOVE_BASIS",
    );
    assert_eq!(session.draft, saved);
    page.building.credit_reduction = 400_000;
    page.more.improvements[0].credit_reduction = 1_000_001;
    rejected_with(save_rentals(&mut session, vec![page]), "RENTAL_CREDIT_REDUCTION_ABOVE_BASIS");
    assert_eq!(session.draft, saved);
}

#[test]
fn unit42_prior_home_carry_native_save_reopen_restart_and_bounds() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = rental_page(1_200_000);
    page.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .home_depreciation_carryover = crate::SourceCents::Reported { amount: 300_001 };
    page.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: 400_003 };
    page.prior_home_carry.get_or_insert(crate::RentalHomeCarries::NONE).home_casualty_carryover =
        crate::SourceCents::Reported { amount: 100_001 };
    page.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_casualty_carryover = crate::SourceCents::Unavailable;
    accepted(&save_rentals(&mut session, vec![page.clone()]));
    assert_rentals_round_trip(&mut session, &[page.clone()]);
    let before = session.draft.clone();
    page.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_casualty_carryover = crate::SourceCents::Reported { amount: 400_004 };
    rejected_with(save_rentals(&mut session, vec![page]), "RENTAL_HOME_PRIOR_CARRY_RECORDS");
    assert_eq!(session.draft, before);
}

#[test]
fn direct_participation_c_and_f_actual_facts_save_reopen_restart_and_invalid_save() {
    use crate::tests::support::client::{draft_from_native_records, native_draft_records, restart};
    use crate::{ParticipationFacts as F, ParticipationTest as T};
    for (mut source, facts) in [
        (
            crate::IncomeRecord::Business(business(100_000)),
            F { hours: 137, test: T::Over100NoOneMore { other_max_hours: 136 } },
        ),
        (
            crate::IncomeRecord::Farm(cash_farm(100_000, 0)),
            F {
                hours: 0,
                test: T::RetiredDisabledFarmer {
                    retired_or_disabled: 2024,
                    qualifying_years: vec![2016, 2018, 2020, 2021, 2023],
                },
            },
        ),
    ] {
        match &mut source {
            crate::IncomeRecord::Business(value) => {
                value.more.material_participation = facts.clone()
            }
            crate::IncomeRecord::Farm(value) => value.more.material_participation = facts.clone(),
            _ => unreachable!(),
        }
        let source = IncomeSource::Record { value: source };
        let topic = topic_for(&source);
        let mut session = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
        let before = session.draft.clone();
        let preview = send(&mut session, catalog_preview_document(source.clone()));
        assert!(!matches!(preview, Reply::Response(P::Problem { .. })), "{preview:?}");
        assert_eq!(session.draft, before);
        let request = save_topic(&mut session, topic, vec![document(0, source)]);
        accepted(&send(&mut session, request));
        let id = session.draft.income[0].id;
        let saved = native_draft_records(&session.draft);
        let loaded = draft_from_native_records(session.draft.tax_year, &saved);
        session = restart(loaded);
        let reply = open_topic(&mut session, topic);
        let mut row = page_documents(&reply)[0].clone();
        let value = match &mut row.document {
            IncomeSource::Record { value: crate::IncomeRecord::Business(value) } => {
                &mut value.more.material_participation
            }
            IncomeSource::Record { value: crate::IncomeRecord::Farm(value) } => {
                &mut value.more.material_participation
            }
            _ => unreachable!(),
        };
        assert_eq!(*value, facts);
        assert_eq!(row.id, id);
        value.test = T::FiveOfTen { years: vec![2020, 2021, 2022, 2025, 2025] };
        let before = session.draft.clone();
        let request = save_topic(&mut session, topic, vec![row]);
        let reply = send(&mut session, request);
        let Reply::Response(P::Problem { field, .. }) = &reply else { panic!("{reply:?}") };
        assert_eq!(
            field.as_deref(),
            Some("[documents/documents/0/document/more/material_participation]")
        );
        rejected_with(reply, "DIRECT_PARTICIPATION_FACTS_INVALID");
        assert_eq!(session.draft, before);
    }
}

#[test]
fn participation_history_years_may_recur_in_independent_businesses() {
    let mut session = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let documents = (0..2)
        .map(|_| {
            let mut value = business(100_000);
            value.more.material_participation = crate::ParticipationFacts {
                hours: 0,
                test: crate::ParticipationTest::FiveOfTen {
                    years: vec![2016, 2018, 2020, 2023, 2025],
                },
            };
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(value) })
        })
        .collect();
    let request = save_topic(&mut session, TaxSection::SelfEmployment, documents);
    accepted(&send(&mut session, request));
    let sources = session.draft.direct_participation_sources();
    assert_eq!(sources.len(), 2);
    assert_ne!(sources[0].id, sources[1].id);
    assert!(sources.iter().all(|source| source.result == crate::participation::Participation::Yes));
}

#[test]
fn rental_warning_rebuilds_choices_and_saves_commercial_sale_and_royalty_components() {
    use crate::tests::support::builders::related_forms::{related_component_editor, related_keys};
    let (mut session, papers) = related_component_editor();
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        papers,
    ));
    let catalog = crate::related_forms::catalog(&session.draft);
    let rent_key = catalog
        .iter()
        .find(|row| row.kind == crate::related_forms::DocumentKind::Form1099MiscRents)
        .unwrap()
        .id;
    let royalty_key = catalog
        .iter()
        .find(|row| row.kind == crate::related_forms::DocumentKind::Form1099MiscRoyalties)
        .unwrap()
        .id;
    let mut rental = rental_sold_page(20260815, 40_000_000);
    rental.kind = crate::RentalPropertyType::Commercial;
    rental.rent = 3_600_000;
    rental.address = "41 Synthetic Commerce Way".into();
    rental.related = related_keys(&[rent_key]);
    rental.statement = Some(rental_statement(20260815, 40_000_000, false));
    rental.more.at_risk = None;
    rental.more.known_at_risk.clear();
    let royalty = crate::RoyaltyProperty {
        related: related_keys(&[royalty_key]),
        kind: crate::RoyaltyKind::Intangible,
        receipts: 500_000,
        advance: 0,
        expenses: no_rental_expenses(),
        depletion: crate::RoyaltyDepletion::No,
        special: false,
    };
    let mut request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) }),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Royalties(royalty) }),
        ],
    );
    let before = session.draft.clone();
    // The strict link composer changes only the unsaved host row. Cancelling
    // the host still leaves the accepted return and revision unchanged.
    let reply = send(
        &mut session,
        P::AnswerRentalRisk { user: TEST_USER, value: crate::RentalAtRisk::All },
    );
    let Reply::Response(P::RentalRiskAnswered { value, known }) = reply else {
        panic!("required at-risk answer returns to the rental editor")
    };
    assert_eq!(known, vec![value.clone()]);
    assert_eq!(session.draft, before);
    let P::SaveIncomes {
        documents,
        allowed_documents,
        allowed_owners,
        allowed_community,
        source_choices,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    let expected_documents = allowed_documents.clone();
    let expected_owners = allowed_owners.clone();
    let expected_community = allowed_community.clone();
    let expected_record_choices = source_choices.record.clone();
    let crate::IncomeRecord::Rental(property) = &mut documents.documents[0].document else {
        unreachable!()
    };
    property.more.at_risk = Some(value);
    property.more.known_at_risk = known;
    property.allowed_interest_limit_election.clear();
    // The submitted wire request omits output-only calculated catalogs.
    allowed_documents.clear();
    allowed_owners.clear();
    allowed_community.clear();
    source_choices.record.clear();
    for row in &mut documents.documents {
        row.allowed_community.clear();
        match &mut row.document {
            crate::IncomeRecord::Rental(page) => {
                page.related.candidates.clear();
                page.more.known_at_risk.clear();
            }
            crate::IncomeRecord::Royalties(page) => page.related.candidates.clear(),
            _ => unreachable!(),
        }
    }
    let Reply::Edit(retry @ P::SaveIncomes { .. }) = send(&mut session, request) else {
        panic!("new property election questions must return a complete editor")
    };
    assert_eq!(session.draft, before);
    let P::SaveIncomes {
        documents,
        allowed_documents,
        allowed_owners,
        allowed_community,
        source_choices,
        warnings,
        ..
    } = &retry
    else {
        unreachable!()
    };
    assert_eq!(allowed_documents, &expected_documents);
    assert_eq!(allowed_owners, &expected_owners);
    assert_eq!(allowed_community, &expected_community);
    assert_eq!(source_choices.record, expected_record_choices);
    assert!(warnings.iter().any(|warning| status_message(&warning.message).key
        == "INTEREST_LIMIT_ELECTION_QUESTIONS_UPDATED"));
    assert_eq!(documents.documents.len(), 2);
    for row in &documents.documents {
        assert!(!row.allowed_community.is_empty());
    }
    let crate::IncomeRecord::Rental(property) = &documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(property.rent, 3_600_000);
    assert_eq!(property.statement.as_ref().unwrap().box_2a_total_gross_proceeds, 40_000_000);
    assert_eq!(property.more.at_risk, Some(crate::RentalAtRisk::All));
    assert_eq!(property.more.known_at_risk, vec![crate::RentalAtRisk::All]);
    assert!(property.related.candidates.iter().any(|row| row.id == rent_key));
    assert_eq!(property.related.forms[0].key, rent_key);
    let crate::IncomeRecord::Royalties(property) = &documents.documents[1].document else {
        unreachable!()
    };
    assert_eq!(property.receipts, 500_000);
    assert!(property.related.candidates.iter().any(|row| row.id == royalty_key));
    assert_eq!(property.related.forms[0].key, royalty_key);
    let reply = send(&mut session, retry);
    accepted(&reply);
    assert!(session.accepted_save, "reviewed page must be accepted: {reply:?}");
    assert!(session.draft.revision > before.revision, "reviewed page must commit: {reply:?}");
    // Adaptive continuation may open another unfinished income topic. It
    // must not return this rental's pre-save correction editor again.
    assert!(
        !matches!(&reply, Reply::Edit(P::SaveIncomes { topic: TaxSection::Rentals, .. })),
        "reviewed rental page must finish saving: {reply:?}"
    );
    let saved = session.draft.clone();
    let mut session = restart(saved.clone());
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    let rows = page_documents(&reopened);
    assert_eq!(rows.len(), 2);
    let IncomeSource::Record { value: crate::IncomeRecord::Rental(property) } = &rows[0].document
    else {
        unreachable!()
    };
    assert_eq!(property.more.known_at_risk, vec![crate::RentalAtRisk::All]);
    assert_eq!(property.more.at_risk, Some(crate::RentalAtRisk::All));
    assert_eq!(property.related.forms[0].key, rent_key);
    assert_eq!(property.rent, 3_600_000);
    assert_eq!(session.draft, saved);
}
