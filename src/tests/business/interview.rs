// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::document;
use super::support::client::income_pages::{
    catalog_preview_document, open_topic, page_documents, save_topic, topic_kinds,
};
use crate::tests::cross::interview::{
    PAGE_ADJUSTMENTS, PAGE_REVIEW, adult, assert_round_trip, asset,
    at_the_adjustments_hub_with_business, at_the_credits_hub, at_the_gifts_page, back, business,
    continue_hub, form_8615_return, gifts, go_on, go_to, hub_rows, hub_tail_unasked,
    income_topics_but, itemized_hub, more_credits_value, navigate, no_expenses, not_started_rows,
    numbered, open_adjustments_hub, open_credits_topic, past_credits, past_people_with_self_work,
    problem_text, rental_page, reopened_businesses, row_of, save_businesses, save_more_credits,
    save_setup, save_you, se_health_test_year, solar_asset, sold, stored_business, tick,
    vehicle_gift, with_first_year, with_tail,
};
pub(in crate::tests) fn mileage(
    first_half: u32,
    second_half: u32,
    total: u32,
    more: MileageMore,
) -> Vehicle {
    Vehicle {
        method: VehicleMethod::StandardMileage { id: 0, first_half, second_half, total, more },
    }
}

// Both explicit SSTB answers survive the native page lifecycle.
#[test]
fn business_page_stores_the_specified_service_answer() {
    for (receipts, answer) in
        [(40_000_000, false), (40_000_000, true), (5_000_000, false), (5_000_000, true)]
    {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut b = business(receipts);
        b.specified_service = answer;
        accepted(&save_businesses(&mut session, vec![b.clone()]));
        assert_eq!(stored_business(&session).1.specified_service, answer);
        assert_round_trip(&mut session, &[b]);
    }
}

#[test]
fn business_related_nec_keeps_printed_state_rows_through_save_reopen_restart() {
    use crate::tests::support::client::income_pages::{open_documents, save_documents};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![business(5_000_000)]));
    // A linked form's amount joins the business's own typed receipts once
    // (information_returns::route): the same return as typing both totals.
    let mut control = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut control, vec![business(10_000_000)]));
    let baseline = supported(&control.draft);
    let paper = crate::Form1099NecPaper {
        calendar_year: 2026,
        payer: crate::PrintedParty::from_name("Example Consulting Customer".into()),
        recipient: fixture_document_names(&session.draft, -1).unwrap().recipient,
        compensation: Some(5_000_000),
        state_details: crate::StateIncomeFields {
            states: vec![
                crate::StateIncomeTax {
                    state: crate::TaxState::NY,
                    payer_state_id: "000-123".into(),
                    income: None,
                    withholding: Some(1_234),
                },
                crate::StateIncomeTax {
                    state: crate::TaxState::NJ,
                    payer_state_id: "0000456".into(),
                    income: Some(0),
                    withholding: Some(0),
                },
            ],
        },
        ..Default::default()
    };
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::SelfEmployment {
                value: crate::SelfEmploymentDocument::Form1099Nec(paper.clone()),
            },
        )],
    );
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    let key = crate::related_forms::catalog(&session.draft)
        .into_iter()
        .find(|source| source.kind == crate::related_forms::DocumentKind::Form1099Nec)
        .unwrap()
        .id;
    let mut rows = page_documents(&open_topic(&mut session, TaxSection::SelfEmployment));
    let IncomeSource::Record { value: crate::IncomeRecord::Business(page) } = &mut rows[0].document
    else {
        unreachable!()
    };
    page.related.forms = vec![crate::RelatedFormView { key, summary: None }];
    let request = save_topic(&mut session, TaxSection::SelfEmployment, rows);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let result = supported(&saved);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (baseline.tax, baseline.refund, baseline.balance)
    );
    assert_eq!(result.return_lines, baseline.return_lines);
    // Every calculation line matches the typed total but the linked form's own line.
    let (id, business) = stored_business(&session);
    let linked = alloc::format!("business_{id}_from_forms");
    let from_forms = result.lines.iter().find(|line| line.id == linked).unwrap();
    assert_eq!(from_forms.amount, 5_000_000);
    let others = |lines: &[crate::CalculationLine]| {
        lines.iter().filter(|line| line.id != linked).cloned().collect::<Vec<_>>()
    };
    assert_eq!(others(&result.lines), others(&baseline.lines));
    assert_eq!(business.receipts(), 5_000_000);
    let mut cold = restart(saved.clone());
    let request = open_topic(&mut cold, TaxSection::SelfEmployment);
    let rows = page_documents(&request);
    let IncomeSource::Record { value: crate::IncomeRecord::Business(page) } = &rows[0].document
    else {
        unreachable!()
    };
    assert_eq!(page.related.forms[0].key, key);
    accepted(&send(&mut cold, request));
    let Reply::Response(P::FormViewed {
        form: crate::RelatedForm::Form1099Nec { state_details, document, .. },
        ..
    }) = send(&mut cold, P::ViewRelatedForm { user: TEST_USER, key })
    else {
        unreachable!()
    };
    assert_eq!(state_details, paper.state_details);
    assert_eq!(Some(document.compensation), paper.compensation);
    let reopened = open_documents(&mut cold);
    let P::SaveDocuments { documents, .. } = &reopened else { unreachable!() };
    assert_eq!(
        documents.common.self_employment.documents[0].document,
        crate::SelfEmploymentDocument::Form1099Nec(paper)
    );
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}

#[test]
fn expense_lines_become_one_record_each() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(5_000_000);
    b.name = "Sample Studio".into();
    b.expenses = BusinessExpenses {
        advertising: 120_000,
        legal_and_professional: 45_000,
        meals: vec![crate::MealExpense { kind: crate::MealKind::Business, amount: 100_000 }],
        wages: 300_000,
        other: vec![OtherExpense { description: "Software subscriptions".into(), amount: 24_000 }],
        ..no_expenses()
    };
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let entry = session.draft.income.last().unwrap();
    assert_eq!(entry.label, "Sample Studio");
    let (id, data) = stored_business(&session);
    // Each nonzero line is one record in printed order, the other expenses
    // last; meals take the 50% treatment; zero lines store nothing.
    let ordinary = ordinary_business_expense;
    assert_eq!(
        data.expenses,
        vec![
            ordinary(BusinessExpenseCategory::Advertising, 120_000),
            ordinary(BusinessExpenseCategory::LegalAndProfessional, 45_000),
            FederalBusinessExpense {
                category: BusinessExpenseCategory::Meals,
                amount: 100_000,
                treatment: BusinessExpenseKind::Meals,
            },
            ordinary(BusinessExpenseCategory::Wages, 300_000),
            ordinary(
                BusinessExpenseCategory::Other { label: "Software subscriptions".into() },
                24_000
            ),
        ]
    );
    assert_eq!(
        data.activity,
        BusinessActivity::Ordinary {
            returns: 0,
            receipts: 5_000_000,
            accounting: BusinessAccounting::Cash,
        }
    );
    // Line 24b is half the meals: $1,000 x 50% = $500; line 27b is $240.
    let r = supported(&session.draft);
    assert_eq!(value(&r, &alloc::format!("business_{id}_expense_line24b_meals")), 500);
    assert_eq!(value(&r, &alloc::format!("business_{id}_expense_line8_advertising")), 1_200);
    assert_eq!(value(&r, &alloc::format!("business_{id}_expense_line27b_other")), 240);
    assert_round_trip(&mut session, &[b.clone()]);
    // The DOT hours-of-service answer turns the meals line into the 80% treatment.
    b.expenses.meals =
        vec![crate::MealExpense { kind: crate::MealKind::Transportation, amount: 100_000 }];
    let items = page_documents(&open_topic(&mut session, TaxSection::SelfEmployment)).to_vec();
    let mut items = items;
    items[0].document = IncomeSource::Record { value: crate::IncomeRecord::Business(b.clone()) };
    let request = save_topic(&mut session, TaxSection::SelfEmployment, items);
    accepted(&send(&mut session, request));
    let (id, data) = stored_business(&session);
    assert_eq!(data.expenses[2].treatment, BusinessExpenseKind::TransportationMeals);
    assert_eq!(
        value(&supported(&session.draft), &alloc::format!("business_{id}_expense_line24b_meals")),
        800
    );
    assert_round_trip(&mut session, &[b.clone()]);
    // A business shows no amount on its row.
    let reply = send(
        &mut session,
        catalog_preview_document(IncomeSource::Record { value: crate::IncomeRecord::Business(b) }),
    );
    assert!(matches!(reply, Reply::Response(P::IncomePreview { preview: None, .. })));
}

#[test]
fn mileage_saves_all_miles_and_other_vehicle_facts() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(8_800_000);
    b.vehicles = vec![
        mileage(8_000, 6_000, 14_000, MileageMore::default()),
        mileage(
            1_000,
            500,
            10_000,
            MileageMore {
                placed: None,
                personal_use_available: true,
                other_vehicle: true,
                evidence: VehicleEvidence::Written,
                commuting: 2_000,
                parking_tolls: 12_000,
                interest: 0,
                property_tax: 30_000,
                leased: true,
            },
        ),
    ];
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (id, data) = stored_business(&session);
    assert!(data.vehicles[0].id > 0 && data.vehicles[1].id > data.vehicles[0].id);
    assert_eq!(
        data.vehicles,
        vec![
            BusinessMileage {
                id: data.vehicles[0].id,
                ..native_business_mileage(8_000, 6_000, 14_000)
            },
            BusinessMileage {
                id: data.vehicles[1].id,
                commuting: 2_000,
                parking_tolls: 12_000,
                property_tax: 30_000,
                owned: false,
                ..native_business_mileage(1_000, 500, 10_000)
            },
        ]
    );
    // Vehicle 1: 8,000 x $0.725 + 6,000 x $0.76 = $5,800 + $4,560 = $10,360.
    let r = supported(&session.draft);
    assert_eq!(value(&r, &alloc::format!("business_{id}_vehicle_0_expense")), 10_360);
    assert_round_trip(&mut session, &[b.clone()]);
    // Business plus commuting miles above all miles is refused on both sides.
    let mut over = business(8_800_000);
    over.vehicles = vec![mileage(
        6_000,
        3_000,
        10_000,
        MileageMore {
            placed: None,
            personal_use_available: true,
            other_vehicle: true,
            evidence: VehicleEvidence::Written,
            commuting: 1_001,
            parking_tolls: 0,
            interest: 0,
            property_tax: 0,
            leased: false,
        },
    )];
    let before = session.draft.clone();
    rejected_with(save_businesses(&mut session, vec![over.clone()]), "MILEAGE_ABOVE_TOTAL_MILES");
    assert_eq!(session.draft, before);
    rejected_with(
        send(
            &mut session,
            catalog_preview_document(IncomeSource::Record {
                value: crate::IncomeRecord::Business(over),
            }),
        ),
        "MILEAGE_ABOVE_TOTAL_MILES",
    );
}

#[test]
fn f04_actual_vehicle_saves_reopens_and_rejects_without_changing_answers() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let actual = |total_miles, business_miles| Vehicle {
        method: VehicleMethod::Actual {
            macrs_source: None,
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            allowed_amt_carry: Vec::new(),
            cost: 3_000_000,
            credit_reduction: 0,
            acquired: None,
            placed: 20260301,
            total_miles,
            business_miles,
            operating_costs: 800_000,
            loan_interest: 0,
            kind: crate::VehicleKind::PassengerCar,
            history: AssetHistory::New { first_year: FirstYearDeduction::Bonus },
            disposal: crate::AssetDisposal::No,
            earlier_business_percent: None,
        },
    };
    let mut b = business(8_800_000);
    // The actual-expense vehicle entered first is stored after the mileage
    // one: the page's vehicles are unordered, and a note's "Vehicle n"
    // follows the stored order, derived where it is shown.
    b.vehicles = vec![actual(20_000, 15_000), mileage(1_000, 1_000, 2_000, MileageMore::default())];
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (id, data) = stored_business(&session);
    assert_eq!(data.vehicles.len(), 1);
    // The mileage vehicle is Vehicle 1 and the actual-expense one Vehicle 2.
    assert_eq!(BusinessAsset::display_text(&data.assets, 0, data.vehicles.len()), "Vehicle 2");
    assert_eq!(data.assets.len(), 1);
    assert!(data.assets[0].id > 0);
    assert_eq!(
        data.assets,
        vec![BusinessAsset {
            // 15,000 / 20,000 = 75%.
            id: data.assets[0].id,
            macrs_source: None,
            cost: 3_000_000,
            acquired: 20260301,
            placed: 20260301,
            history: EquipmentHistory::New {
                section179: Section179::Eligible { elected: 0 },
                bonus: BonusElection::Apply,
                tables: true,
            },
            listed: ListedProperty::Vehicle {
                total_miles: 20_000,
                business_miles: 15_000,
                operating_costs: 800_000,
                loan_interest: 0,
                kind: crate::VehicleKind::PassengerCar,
            },
            ..bonus_equipment(0, 100)
        }]
    );
    // Line 9's actual part is the business-mile share of operating costs:
    // $8,000 x 15,000 / 20,000 = $6,000.
    let r = supported(&session.draft);
    assert_eq!(value(&r, &alloc::format!("business_{id}_car_and_truck")), 6_000 + 1_485);
    assert_round_trip(
        &mut session,
        &[Business {
            interest_limit_election: None,
            allowed_interest_limit_election: Vec::new(),
            vehicles: vec![
                mileage(1_000, 1_000, 2_000, MileageMore::default()),
                actual(20_000, 15_000),
            ],
            ..b
        }],
    );
    // Two-thirds business use retains its exact miles in the stored record.
    let mut third = business(8_800_000);
    third.vehicles = vec![actual(3_000, 2_000)];
    accepted(&save_businesses(&mut session, vec![third]));
    let (_, data) = stored_business(&session);
    assert!(matches!(
        data.assets[0].listed,
        ListedProperty::Vehicle { total_miles: 3_000, business_miles: 2_000, .. }
    ));
    assert_eq!(data.assets[0].business_use_hundredths(), 6_667);
    assert!(data.assets[0].valid());
    // A fresh actual-expense row with 0.5% business use keeps its exact
    // mileage, operating expense and ADS depreciation through save/reopen.
    let mut fractional = business(8_800_000);
    fractional.vehicles = vec![actual(20_000, 100)];
    accepted(&save_businesses(&mut session, vec![fractional.clone()]));
    let (id, data) = stored_business(&session);
    assert!(matches!(
        data.assets[0].listed,
        ListedProperty::Vehicle { total_miles: 20_000, business_miles: 100, .. }
    ));
    assert_eq!(data.assets[0].business_basis(), 15_000);
    let result = supported(&session.draft);
    assert_eq!(
        result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("business_{id}_asset_0_macrs"))
            .unwrap()
            .amount,
        1_500
    );
    assert_eq!(
        result
            .lines
            .iter()
            .find(|line| line.id == alloc::format!("business_{id}_car_and_truck"))
            .unwrap()
            .amount,
        4_000
    );
    assert_round_trip(&mut session, &[fractional]);
    let before = session.draft.clone();
    // Business miles above all miles is refused on both sides.
    let mut over = business(8_800_000);
    over.vehicles = vec![actual(1_000, 1_001)];
    rejected_with(save_businesses(&mut session, vec![over]), "VEHICLE_BUSINESS_MILES_ABOVE_TOTAL");
    assert_eq!(session.draft, before);
    for (total, business_miles) in [(0, 0), (0, 1), (20_000, 0)] {
        let mut zero = business(8_800_000);
        zero.vehicles = vec![actual(total, business_miles)];
        let reply = save_businesses(&mut session, vec![zero]);
        assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
        assert_eq!(session.draft, before);
    }
}

#[test]
fn special_business_treatment_is_left_out_with_a_notice() {
    // The removed BusinessDepletionOrPlans and SelfEmployment situations:
    // the page's "special" answer is stored and the business left out.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(5_000_000);
    b.more.special = true;
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    assert!(data.special_treatment);
    let r = supported(&session.draft);
    assert!(noted(&r, "WARN_BUSINESS_LEFT_OUT"), "{r:?}");
    assert_eq!(r.tax, supported(&complete(FederalStatus::Single)).tax);
    assert_round_trip(&mut session, &[b]);
}

#[test]
fn equipment_rent_assets_map_onto_business_assets() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let rent = |first_year| EquipmentRent {
        related: crate::RelatedForms::default(),
        receipts: 150_000,
        expenses: 20_000,
        basis: 300_000,
        depreciable_basis: 300_000,
        assets: vec![with_first_year(asset(300_000, 20260310, AssetClass::Seven), first_year)],
        history: PersonalRentalHistory::NONE,
    };
    let mut bonus = rent(FirstYearDeduction::Bonus);
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::OtherIncome,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(bonus.clone()) },
            )],
        );
        send(&mut session, request)
    });
    let FederalIncome::NonbusinessProperty {
        kind: NonbusinessPropertyKind::PersonalRental { assets, .. },
        ..
    } = &session.draft.income[1].income
    else {
        panic!("expected the equipment rent")
    };
    bind_saved_property_ids(&mut bonus.assets, &mut [], assets);
    // 100% income-producing nonbusiness property: never section 179 property.
    assert_eq!(
        assets,
        &vec![BusinessAsset {
            id: bonus.assets[0].id,

            macrs_source: None,
            cost: 300_000,
            acquired: 20260310,
            placed: 20260310,
            class: EquipmentClass::Seven,
            history: EquipmentHistory::New {
                section179: Section179::Ineligible,
                bonus: BonusElection::Apply,
                tables: true,
            },
            ..bonus_equipment(0, 100)
        }]
    );
    assert!(assets[0].qualifies_nonbusiness_rental());
    supported(&session.draft);
    let mut reopened = restart(session.draft.clone());
    let documents = page_documents(&open_topic(&mut reopened, TaxSection::OtherIncome)).to_vec();
    assert_eq!(
        documents[0].document,
        IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(bonus) }
    );
    // Section 179 is refused by name on save and preview.
    let with_179 =
        rent(FirstYearDeduction::Section179 { description: "Trailer".into(), elected: 100_000 });
    let before = session.draft.clone();
    let request = save_topic(
        &mut session,
        TaxSection::OtherIncome,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(with_179.clone()) },
        )],
    );
    rejected_with(send(&mut session, request), "EQUIPMENT_RENT_NO_SECTION_179");
    assert_eq!(session.draft, before);
    rejected_with(
        send(
            &mut session,
            catalog_preview_document(IncomeSource::Record {
                value: crate::IncomeRecord::EquipmentRental(with_179),
            }),
        ),
        "EQUIPMENT_RENT_NO_SECTION_179",
    );
}

// Matrix row 9's fixture (tests/business.rs
// sole_proprietor_return_matches_hand_computation) entered through the hub
// and the Schedule C page; the hand computation is the fixture's comment.
// The business page takes 13 fields: gross receipts, the eight expense
// lines, the two half-year mileage figures, the home-office choice and its
// square feet. The M3 self-employed fixture adds its computer's cost, date
// placed in service and kind: 16 fields.
#[test]
fn sole_proprietor_page_reproduces_the_row_9_fixture() {
    let mut s = Interview::new(TEST_USER, FederalDraft::empty("row-9".into()));
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let request = save_you(&s.draft, adult("Casey", "Sample", 19850615));
    let reply = send(&mut s, request);
    let checklist = past_people_with_self_work(&mut s, reply, true);
    // Income without a form establishes self-work; the remaining checklist
    // needs no selection before the walk opens the business page.
    let request = tick(checklist, Checklist::NONE);
    let Reply::Edit(page) = send(&mut s, request) else {
        panic!("expected the self-employment page")
    };
    let P::SaveIncomes { allowed_documents, documents, .. } = &page else {
        panic!("expected the self-employment page, got {page:?}")
    };
    assert_eq!(allowed_documents, &topic_kinds(TaxSection::SelfEmployment));
    assert!(documents.documents.is_empty());
    let mut b = business(8_800_000);
    b.expenses = BusinessExpenses {
        advertising: 120_000,
        contract_labor: 600_000,
        insurance: 240_000,
        office: 90_000,
        supplies: 150_000,
        taxes_and_licenses: 60_000,
        meals: vec![crate::MealExpense { kind: crate::MealKind::Business, amount: 100_000 }],
        utilities: 180_000,
        ..no_expenses()
    };
    b.vehicles = vec![mileage(8_000, 6_000, 14_000, MileageMore::default())];
    // The walk's Less common income comes first (gate fixes batch A).
    let reply = save_businesses(&mut s, vec![b.clone()]);
    let source = s.draft.income.last().unwrap().id;
    add_simplified_office(&mut s.draft, source, 200);
    assert!(matches!(reply, Reply::Edit(P::SaveSelfEmployedBenefits { .. })));
    let request = go_to(&s.draft, PAGE_REVIEW, TaxSection::Income);
    let Reply::Response(hub) = send(&mut s, request) else { panic!("expected the income summary") };
    // The summary retains every topic and its explicit Continue action.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of("Self-employment", "1 business", TaxSection::SelfEmployment)],
            not_started_rows(&income_topics_but(&[TaxSection::SelfEmployment]))
        )
    );
    // Continuing enters the adjustments section (M3-P7), whose derived
    // self-employed benefits topic (M3-P8) the walk opens directly because
    // this return has a business (M3-P19); its Back opens the hub, whose
    // Continue passes every later section (nothing there for this return)
    // to the review, since neither health insurance nor a retirement plan
    // applies here.
    let Reply::Edit(P::SaveSelfEmployedBenefits { previous, .. }) = send(&mut s, go_on(&hub))
    else {
        panic!("expected the self-employed benefits page")
    };
    let request = back(&s.draft, previous);
    let Reply::Response(hub @ P::Hub { suggested: TaxSection::SelfEmployedBenefits, .. }) =
        send(&mut s, request)
    else {
        panic!("expected the adjustments hub to suggest self-employed benefits")
    };
    let request = continue_hub(&s.draft, PAGE_ADJUSTMENTS);
    assert!(
        matches!(&hub, P::Hub { allowed_sections, .. } if allowed_sections.contains(&PAGE_REVIEW))
    );
    let reply = send(&mut s, request);
    let Reply::Response(P::Review { .. }) = past_credits(&mut s, reply) else {
        panic!("expected the review")
    };
    let r = reviewed(&s);
    let id = s.draft.income[0].id;
    let v = |name: &str| value(&r, &alloc::format!("business_{id}_{name}"));
    assert_eq!(v("expense_line8_advertising"), 1_200);
    assert_eq!(v("expense_line11_contract_labor"), 6_000);
    assert_eq!(v("expense_line15_insurance"), 2_400);
    assert_eq!(v("expense_line18_office"), 900);
    assert_eq!(v("expense_line22_supplies"), 1_500);
    assert_eq!(v("expense_line23_taxes_and_licenses"), 600);
    assert_eq!(v("expense_line25_utilities"), 1_800);
    assert_eq!(v("expense_line24b_meals"), 500);
    assert_eq!(v("vehicle_0_expense"), 10_360);
    assert_eq!(v("home_office"), 1_000);
    assert_eq!(v("cost_of_goods_sold"), 0);
    assert_eq!(return_line(&r, "8"), 6_174_000);
    assert_eq!(return_line(&r, "9"), 6_174_000);
    assert_eq!(return_line(&r, "10"), 436_200);
    assert_eq!(return_line(&r, "11a / 11b"), 5_737_800);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 825_600);
    assert_eq!(return_line(&r, "15"), 3_302_200);
    assert_eq!(return_line(&r, "16"), 371_500);
    assert_eq!(return_line(&r, "23"), 872_400);
    assert_eq!(return_line(&r, "24a"), 1_243_900);
    assert_eq!(return_line(&r, "33"), 0);
    assert_eq!(return_line(&r, "37"), 1_243_900);
    assert_eq!(value(&r, "self_employment_tax"), 8_724);
    assert_eq!(value(&r, "qbi_deduction"), 8_256);
    assert_eq!(r.balance, Some(1_243_900));
    // The M3 fixture's computer, bought after January 19, 2025: 100% bonus,
    // $2,400 on line 13, so line 31 is $61,740 - $2,400 = $59,340.
    let request = navigate(&s.draft, TaxSection::SelfEmployment);
    let Reply::Edit(page) = send(&mut s, request) else {
        panic!("expected the self-employment page")
    };
    let mut items = page_documents(&page).to_vec();
    b.assets = vec![asset(240_000, 20260210, AssetClass::Five)];
    items[0].document = IncomeSource::Record { value: crate::IncomeRecord::Business(b) };
    let mut request = save_topic(&mut s, TaxSection::SelfEmployment, items);
    if let P::SaveIncomes { review, .. } = &mut request {
        *review = true;
    }
    let Reply::Response(P::Review { .. }) = send(&mut s, request) else {
        panic!("expected the review")
    };
    let r = reviewed(&s);
    assert_eq!(value(&r, &alloc::format!("business_{id}_asset_0_bonus")), 2_400);
    assert_eq!(return_line(&r, "8"), 5_934_000);
}

#[test]
fn business_benefits_write_the_business_record() {
    let mut s = at_the_adjustments_hub_with_business(8_800_000);
    let hub = open_adjustments_hub(&mut s);
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    // A business with nothing entered yet makes the topic necessary.
    assert_eq!(*suggested, TaxSection::SelfEmployedBenefits);
    // Amended for M3-N1: the necessary topic first, then the others.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of(
                "Self-employed health insurance and retirement plan",
                "Not started",
                TaxSection::SelfEmployedBenefits
            )],
            hub_tail_unasked(
                &[
                    TaxSection::Hsa,
                    TaxSection::IraContributions,
                    TaxSection::Educator,
                    TaxSection::StudentLoan,
                    TaxSection::EmployeeExpenses
                ],
                TaxSection::MoreAdjustments
            )
        )
    );
    // OK opens the page as a complete editor with one membership-only row.
    let Reply::Edit(page) = send(&mut s, go_on(&hub)) else {
        panic!("expected the self-employed benefits page")
    };
    let P::SaveSelfEmployedBenefits { page: at, previous, value, .. } = &page else {
        panic!("expected the self-employed benefits page, got {page:?}")
    };
    assert_eq!((*at, *previous), (PAGE_ADJUSTMENTS, PAGE_ADJUSTMENTS));
    assert_eq!(value.businesses.len(), 1);
    let id = value.businesses[0].id;
    assert_eq!(value.businesses[0].health, SeHealth::None);
    assert_eq!(value.businesses[0].retirement, SeRetirement::None);
    // Saving all-year health coverage and a SEP contribution writes the
    // business record, with `eligible = Yes` (the confirmation branch).
    let request = P::SaveSelfEmployedBenefits {
        user: TEST_USER,
        ptc_policies: vec![],
        health_activities: vec![],
        health_people: vec![],
        health_wages: vec![],
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: PAGE_ADJUSTMENTS,
        previous: PAGE_ADJUSTMENTS,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: Some(vec![]),
        value: SelfEmployedBenefits {
            activity_partnerships: vec![],
            activity_scorporations: vec![],
            scorporations: vec![],
            ptc_method: HealthPtcMethod::Simplified,
            policies: vec![],
            benchmarks: vec![],
            partnerships: vec![],
            businesses: vec![BusinessBenefits {
                id,
                name: crate::common::test_label("Business"),
                health: se_health_test_year(600_000, false),
                retirement: SeRetirement::Sep { contribution: 500_000, rate: 25.0 },
            }],
        },
    };
    // Saving completes the walk; explicit navigation retains its summary.
    let reply = send(&mut s, request);
    assert!(matches!(
        reply,
        Reply::Response(P::BeginOftenMissed { .. }) | Reply::Edit(P::SaveOftenMissed { .. })
    ));
    let hub = open_adjustments_hub(&mut s);
    let P::Hub { suggested, .. } = &hub else { unreachable!() };
    assert_eq!(*suggested, TaxSection::Credits);
    // The summary retains every topic and its explicit Continue action.
    assert_eq!(
        hub_rows(&hub),
        with_tail(
            vec![row_of(
                "Self-employed health insurance and retirement plan",
                "Business 2",
                TaxSection::SelfEmployedBenefits
            )],
            hub_tail_unasked(
                &[
                    TaxSection::Hsa,
                    TaxSection::IraContributions,
                    TaxSection::Educator,
                    TaxSection::StudentLoan,
                    TaxSection::EmployeeExpenses
                ],
                TaxSection::MoreAdjustments
            )
        )
    );
    let FederalIncome::Business { data } = &s.draft.income[0].income else {
        panic!("expected the business record")
    };
    let BusinessHealthInsurance::Plan { months, long_term_care, establishment } =
        &data.health_insurance
    else {
        panic!("expected a health plan, got {:?}", data.health_insurance)
    };
    assert!(long_term_care.is_empty());
    assert!(establishment.included());
    assert_eq!(months.len(), 12);
    assert_eq!(months.iter().map(|m| m.premiums).sum::<i64>(), 600_000);
    assert!(months.iter().all(|m| !m.employer_eligible && m.child_increase == 0));
    assert_eq!(data.retirement, BusinessRetirement::Sep { contribution: 500_000, rate: 25.0 });
    // Reopening reconstructs the same all-year and SEP choice.
    let request = go_to(&s.draft, PAGE_ADJUSTMENTS, TaxSection::SelfEmployedBenefits);
    let Reply::Edit(P::SaveSelfEmployedBenefits { value, .. }) = send(&mut s, request) else {
        panic!("expected the self-employed benefits page")
    };
    assert_eq!(value.businesses[0].health, se_health_test_year(600_000, false));
    assert_eq!(
        value.businesses[0].retirement,
        SeRetirement::Sep { contribution: 500_000, rate: 25.0 }
    );
}

#[test]
fn vehicle_proceeds_of_500_or_less_claim_up_to_500() {
    use crate::tests::support::client::charity::save_gift_source_fixtures;
    // Form 1098-C: a car sold for $500 or less claims the smaller of its value
    // and $500; above $500, the smaller of value and gross proceeds.
    // AGI $60,000, floor $300 on an itemized return.
    let mut s = at_the_gifts_page(Some(ItemizeOverride::Itemize), 0);
    for (proceeds, claim) in [(40_000, 500), (50_000, 500), (50_100, 501), (90_000, 900)] {
        let mut page = gifts(vec![], vec![], vec![vehicle_gift(200_000, sold(proceeds))]);
        itemized_hub(save_gift_source_fixtures(&mut s, &mut page));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "charity_property_1_claim"), claim, "{proceeds}");
        assert_eq!(value(&r, "charity_property_fifty_current_deduction"), claim - 300);
    }
    // FMV claims (box 5a or 5b) above $5,000 together need Section B:
    // $5,000 is Section A; $5,000.01 invokes actual appraisal requirements.
    let mut page = gifts(vec![], vec![], vec![vehicle_gift(500_000, VehicleUse::NeedyPerson)]);
    itemized_hub(save_gift_source_fixtures(&mut s, &mut page));
    let mut page = gifts(
        vec![],
        vec![],
        vec![vehicle_gift(499_999, VehicleUse::NeedyPerson), vehicle_gift(2, sold(1))],
    );
    itemized_hub(save_gift_source_fixtures(&mut s, &mut page));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_GIFT_LEFT_OUT"));
    assert_eq!(value(&r, "charity_deduction"), 0);
    assert_eq!(s.draft.deductions.as_ref().unwrap().vehicle_gifts[0].value, 499_999);
    assert_eq!(s.draft.deductions.as_ref().unwrap().vehicle_gifts[1].value, 2);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
}

#[test]
fn business_property_losses_kind_writes_the_lookback_from_either_page() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // C17: the kind with both lists empty writes an empty confirmed history.
    let none = crate::BusinessPropertyLosses { regular: vec![], amt: vec![] };
    let history = crate::Section1231History { regular: vec![], amt: vec![] };
    let mut r = rental_page(1_200_000);
    r.sale = crate::RentalSale::Sold {
        date: 20260815,
        price: 40_000_000,
        land_price: 8_000_000,
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) }),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(none.clone()),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.screening.as_ref().unwrap().section1231, Some(history.clone()));
    supported(&session.draft);
    // Both pages show the one record; saving the self-employment page
    // without it removes it for the whole return.
    let items = page_documents(&open_topic(&mut session, TaxSection::SelfEmployment)).to_vec();
    assert_eq!(
        items,
        vec![IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(none.clone())
                }
            )
        }]
    );
    let request = save_topic(&mut session, TaxSection::SelfEmployment, vec![]);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.screening.as_ref().unwrap().section1231, None);
    // Entered twice on one page is refused.
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(none.clone()),
                },
            ),
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::BusinessPropertyLosses(none) },
            ),
        ],
    );
    rejected_with(send(&mut session, request), "BUSINESS_PROPERTY_LOSSES_TWICE");
}

#[test]
fn business_page_both_answer_takes_the_thirty_percent_choice_or_an_amount() {
    // T2-src: the Business page's "Both" asks the value of the owner's work:
    // at least 30% of the net profit (no amount) or the amount itself.
    for (choice, stored) in [
        (
            BusinessIncomeChoice::Mixed { value: WorkValue::AtLeastThirtyPercent },
            BusinessIncomeSource::MixedThirtyPercent,
        ),
        (
            BusinessIncomeChoice::Mixed { value: WorkValue::Amount { compensation: 250_000 } },
            BusinessIncomeSource::Mixed { compensation: 250_000 },
        ),
        (BusinessIncomeChoice::Capital, BusinessIncomeSource::Capital),
    ] {
        let mut session =
            Interview::new(TEST_USER, form_8615_return(complete(FederalStatus::Single)));
        let mut b = business(5_000_000);
        b.more.income_source = choice;
        accepted(&save_businesses(&mut session, vec![b.clone()]));
        assert_eq!(stored_business(&session).1.income_source, stored);
        assert_round_trip(&mut session, &[b]);
    }
}

// C17: the kind's two branches write a confirmed history and reopen as
// entered; the sale's lookback issue names the kind and both choices.
#[test]
fn business_property_losses_branches_write_a_confirmed_history() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut r = rental_page(1_200_000);
    r.sale = crate::RentalSale::Sold {
        date: 20260815,
        price: 40_000_000,
        land_price: 8_000_000,
        selling_expenses: 0,
        payment: crate::SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    // A sale with no answer: the review's issue suggests the kind.
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(r.clone()) })],
    );
    accepted(&send(&mut session, request));
    let result = estimate(&session.draft);
    assert!(
        result.issues.iter().any(|i| i.section == "Section 1231 lookback"
            && i.message.contains("\"Business-property losses from 2021–2025\"")
            && i.message.contains("\"No net business-property loss on a 2021–2025 return\"")),
        "{:?}",
        result.issues
    );
    let rental_id = session.draft.income[1].id;
    // Losses in those years: the lists, with complete Yes.
    let year = crate::Section1231LossYear { year: 2023, original: 300_000, remaining: 200_000 };
    let losses =
        crate::BusinessPropertyLosses { regular: vec![year.clone()], amt: vec![year.clone()] };
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![
            document(
                rental_id,
                IncomeSource::Record { value: crate::IncomeRecord::Rental(r.clone()) },
            ),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(losses.clone()),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    assert_eq!(
        session.draft.screening.as_ref().unwrap().section1231,
        Some(crate::Section1231History { regular: vec![year.clone()], amt: vec![year] })
    );
    assert!(page_documents(&open_topic(&mut session, TaxSection::Rentals)).contains(
        &IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::BusinessPropertyLosses(losses) }
            )
        }
    ));
    // Both lists empty is the confirmed empty history and reopens so.
    let request = save_topic(
        &mut session,
        TaxSection::Rentals,
        vec![
            document(rental_id, IncomeSource::Record { value: crate::IncomeRecord::Rental(r) }),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(
                        crate::BusinessPropertyLosses { regular: vec![], amt: vec![] },
                    ),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    assert!(page_documents(&open_topic(&mut session, TaxSection::Rentals)).contains(
        &IncomeDocumentItem {
            refund_management_needed: Some(false),
            ..document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::BusinessPropertyLosses(
                        crate::BusinessPropertyLosses { regular: vec![], amt: vec![] }
                    )
                }
            )
        }
    ));
    supported(&session.draft);
}
fn business_with_source(income_source: BusinessIncomeChoice) -> Business {
    let mut b = business(5_000_000);
    b.more.income_source = income_source;
    b
}

#[test]
fn business_income_source_is_checked_against_its_allowed_set_on_save() {
    // A Form 8615 return: Not needed is refused by name, Services accepted.
    let young = form_8615_return(complete(FederalStatus::Single));
    let mut session = Interview::new(TEST_USER, young.clone());
    let message = problem_text(current_document_refusal(
        "income-source-1185",
        &mut session,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Business(business_with_source(
                    BusinessIncomeChoice::NotNeeded,
                )),
            },
        )],
        "SOURCE_BUSINESS_NEEDED",
        "[documents/documents/0/document/more/income_source]",
    ));
    assert!(message.contains("Form 8615 line 1"), "{message}");
    let page = business_with_source(BusinessIncomeChoice::Capital);
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    assert_eq!(stored_business(&session).1.income_source, BusinessIncomeSource::Capital);
    assert_round_trip(&mut session, &[page]);
    // An adult's return: only Not needed; Capital is refused by name.
    let adult = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, adult.clone());
    let message = problem_text(current_document_refusal(
        "income-source-1380",
        &mut session,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Business(business_with_source(
                    BusinessIncomeChoice::Capital,
                )),
            },
        )],
        "SOURCE_NOT_NEEDED",
        "[documents/documents/0/document/more/income_source]",
    ));
    assert!(message.contains("choose Not needed"), "{message}");
    // A stored Capital answer on a return that no longer reads it reopens
    // as Not needed, inside the set, and saves as Not needed's record.
    let mut d = session.draft.clone();
    let mut kept = Interview::new(TEST_USER, young);
    accepted(&save_businesses(
        &mut kept,
        vec![business_with_source(BusinessIncomeChoice::Capital)],
    ));
    d.income = kept.draft.income.clone();
    d.next_id = kept.draft.next_id;
    let reopened = reopened_businesses(d);
    assert_eq!(reopened, vec![business_with_source(BusinessIncomeChoice::NotNeeded)]);
}

#[test]
fn business_credit_carryovers_round_trip_through_more_credits() {
    // A 2024 Form 5884 carryforward of $2,000 against the $5,023 tax: used in
    // full on Form 3800 line 34/37.
    let (mut s, _) = at_the_credits_hub(complete(FederalStatus::Single));
    let history = BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![BusinessCreditCarryforward {
            filer: crate::CarryFiler::SameFilers,
            research_source: None,
            credit: BusinessCredit::WorkOpportunity,
            origin_year: 2024,
            amount: 200_000,
            passive: None,
        }],
        carrybacks: vec![],
        spouse_without_credits: false,
    };
    let items = vec![MoreCredit::BusinessCredits(history.clone().into())];
    let request = save_more_credits(
        &s.draft,
        MoreCredits { answers: Vec::new(), credits: items.clone(), documents: Vec::new() },
    );
    send(&mut s, request);
    assert_eq!(s.draft.credits.as_ref().unwrap().business_credits, Some(history.clone()));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "form_3800_line_34"), 2_000);
    assert_eq!(value(&r, "general_business_credit"), 2_000);
    let page = open_credits_topic(&mut s, TaxSection::MoreCredits);
    assert_eq!(more_credits_value(&page).credits, items);
    // Server twin of TIP_FICA_PRIOR_YEAR_HAS_ITS_OWN_ITEM.
    let mut wrong = history;
    wrong.carryforwards[0].credit = BusinessCredit::EmployerTipFica;
    wrong.carryforwards[0].origin_year = 2025;
    let request = save_more_credits(
        &s.draft,
        MoreCredits {
            answers: Vec::new(),
            credits: vec![MoreCredit::BusinessCredits(wrong.into())],

            documents: Vec::new(),
        },
    );
    rejected_with(send(&mut s, request), "TIP_FICA_PRIOR_YEAR_HAS_ITS_OWN_ITEM");
    // Form 8586 carryovers: a prior-year unallowed housing credit names
    // Worksheet 3 and reopens; the wrong worksheet is refused on save (the
    // server twin of BUSINESS_CREDIT_WORKSHEET_MISMATCH).
    let housing = BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![
            BusinessCreditCarryforward {
                filer: crate::CarryFiler::SameFilers,
                research_source: None,
                credit: BusinessCredit::LowIncomeHousing,
                origin_year: 2024,
                amount: 150_000,
                passive: Some(PassiveWorksheet::LowIncomeHousing),
            },
            BusinessCreditCarryforward {
                filer: crate::CarryFiler::SameFilers,
                research_source: None,
                credit: BusinessCredit::LowIncomeHousingBefore2008,
                origin_year: 2012,
                amount: 100_000,
                passive: None,
            },
        ],
        carrybacks: vec![],
        spouse_without_credits: false,
    };
    let items = vec![MoreCredit::BusinessCredits(housing.clone().into())];
    let request = save_more_credits(
        &s.draft,
        MoreCredits { answers: Vec::new(), credits: items.clone(), documents: Vec::new() },
    );
    send(&mut s, request);
    let r = supported(&s.draft);
    // Worksheet 3 line 3b $1,500 uses the allowance (3,000 here); the pre-2008
    // $1,000 is a Part I credit on line 2b.
    assert_eq!(value(&r, "form8582cr_line_3b"), 1_500);
    assert_eq!(value(&r, "form8582cr_line_37"), 1_500);
    assert_eq!(
        printed_line(&r, "business_credit_low_income_housing_pre2008_2012"),
        (TaxForm::Form3800, "2b".into())
    );
    assert_eq!(value(&r, "general_business_credit"), 2_500);
    let page = open_credits_topic(&mut s, TaxSection::MoreCredits);
    assert_eq!(more_credits_value(&page).credits, items);
    let mut wrong = housing;
    wrong.carryforwards[0].passive = Some(PassiveWorksheet::Other);
    let request = save_more_credits(
        &s.draft,
        MoreCredits {
            answers: Vec::new(),
            credits: vec![MoreCredit::BusinessCredits(wrong.into())],

            documents: Vec::new(),
        },
    );
    rejected_with(send(&mut s, request), "BUSINESS_CREDIT_WORKSHEET_MISMATCH");
}

#[test]
fn alternative_motor_vehicle_carryback_is_rejected_on_save_and_no_refusal_remains() {
    // Fixture C/D of the Form 8910 map: the server twin of the client rule.
    let (mut s, _) = at_the_credits_hub(complete(FederalStatus::Single));
    let history = BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![BusinessCreditCarryforward {
            filer: crate::CarryFiler::SameFilers,
            research_source: None,
            credit: BusinessCredit::AlternativeMotorVehicle,
            origin_year: 2019,
            amount: 300_000,
            passive: None,
        }],
        carrybacks: vec![],
        spouse_without_credits: false,
    };
    let items = vec![MoreCredit::BusinessCredits(history.clone().into())];
    let request = save_more_credits(
        &s.draft,
        MoreCredits { answers: Vec::new(), credits: items.clone(), documents: Vec::new() },
    );
    send(&mut s, request);
    // The carryover is the whole entry: no refusal and no remaining item.
    assert_eq!(s.draft.credits.as_ref().unwrap().business_credits, Some(history.clone()));
    assert!(s.draft.credit_refusals.is_empty());
    let r = supported(&s.draft);
    assert_eq!(value(&r, "general_business_credit"), 3_000);
    let page = open_credits_topic(&mut s, TaxSection::MoreCredits);
    assert_eq!(more_credits_value(&page).credits, items);
    let mut wrong = history;
    wrong.carrybacks.push(BusinessCreditCarryback {
        research_source: None,
        credit: BusinessCredit::AlternativeMotorVehicle,
        amount: 50_000,
        year: 2025,
    });
    let request = save_more_credits(
        &s.draft,
        MoreCredits {
            answers: Vec::new(),
            credits: vec![MoreCredit::BusinessCredits(wrong.into())],

            documents: Vec::new(),
        },
    );
    rejected_with(send(&mut s, request), "BUSINESS_CREDIT_CARRYFORWARD_ONLY");
}

#[test]
fn vehicle_and_asset_names_are_derived_and_a_179_description_is_its_own_field() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original);
    let mut b = business(8_800_000);
    b.vehicles = vec![mileage(1_000, 1_000, 2_000, MileageMore::default())];
    // A section 179 election whose typed description reads like a positional
    // label, beside a bonus asset: the two are told apart by the description
    // the person typed, never by comparing text.
    b.assets = vec![
        with_first_year(
            asset(300_000, 20260310, AssetClass::Seven),
            FirstYearDeduction::Section179 { description: "Asset 1".into(), elected: 0 },
        ),
        asset(400_000, 20260311, AssetClass::Seven),
    ];
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    // A note names an asset by its typed description, else its position
    // among the property: derived where shown, nothing stored.
    assert_eq!(BusinessAsset::display_text(&data.assets, 0, data.vehicles.len()), "Asset 1");
    assert_eq!(
        BusinessAsset::display_arg(&data.assets, 0, data.vehicles.len()),
        MessageArg::Text("Asset 1".into())
    );
    assert_eq!(
        BusinessAsset::display_arg(&data.assets, 1, data.vehicles.len()),
        MessageArg::Nested(alloc::boxed::Box::new(numbered("LABEL_ASSET_N", 2)))
    );
    assert_eq!(BusinessAsset::display_text(&data.assets, 1, data.vehicles.len()), "Asset 2");
    assert_eq!(data.assets[0].description, "Asset 1");
    assert_eq!(data.assets[1].description, "");
    // The election reopens as a section 179 choice with its description; the
    // other asset reopens as bonus depreciation.
    assert_round_trip(&mut session, &[b]);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn equipment_rent_and_a_farm_rental_take_no_investment_credit() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let original = session.draft.clone();
    let rent = EquipmentRent {
        related: crate::RelatedForms::default(),
        receipts: 150_000,
        expenses: 20_000,
        basis: 300_000,
        depreciable_basis: 300_000,
        assets: vec![solar_asset(20250915)],
        history: PersonalRentalHistory::NONE,
    };
    rejected_with(
        {
            let request = save_topic(
                &mut session,
                TaxSection::OtherIncome,
                vec![document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(rent) },
                )],
            );
            send(&mut session, request)
        },
        "EQUIPMENT_RENT_NO_INVESTMENT_CREDIT",
    );
    assert_eq!(session.draft, original);
}

#[test]
fn suv_classification_and_election_save_reopen_resave_and_restart() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut vehicle = owned_actual_vehicle(FirstYearDeduction::Section179 {
        description: "Synthetic heavy passenger SUV".into(),
        elected: 3_200_000,
    });
    let VehicleMethod::Actual {
        cost,
        total_miles,
        business_miles,
        operating_costs,
        loan_interest,
        kind,
        ..
    } = &mut vehicle.method
    else {
        unreachable!()
    };
    *cost = 7_000_000;
    *total_miles = 10_000;
    *business_miles = 10_000;
    *operating_costs = 0;
    *loan_interest = 0;
    *kind = VehicleKind::HeavySuv { exception: SuvException::None };
    let mut page = business(10_000_000);
    page.vehicles = vec![vehicle];
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let (source, stored) = stored_business(&session);
    let asset_id = stored.assets[0].id;
    assert!(asset_id > 0);
    assert!(matches!(
        stored.assets[0].listed,
        ListedProperty::Vehicle {
            kind: VehicleKind::HeavySuv { exception: SuvException::None },
            ..
        }
    ));
    let expected = supported(&session.draft);
    assert_eq!(value(&expected, &alloc::format!("business_{source}_asset_0_section179")), 32_000);
    assert_eq!(value(&expected, &alloc::format!("business_{source}_asset_0_bonus")), 38_000);
    assert_round_trip(&mut session, &[page.clone()]);
    let cold = restart(session.draft.clone());
    assert_eq!(stored_business(&cold).1.assets[0].id, asset_id);
    assert_eq!(supported(&cold.draft), expected);

    let before = session.draft.clone();
    let mut invalid = page.clone();
    let VehicleMethod::Actual { history, .. } = &mut invalid.vehicles[0].method else {
        unreachable!()
    };
    *history = AssetHistory::New {
        first_year: FirstYearDeduction::Section179 {
            description: "Synthetic heavy passenger SUV".into(),
            elected: 7_000_000,
        },
    };
    let rejection = save_businesses(&mut session, vec![invalid]);
    let Reply::Response(P::Problem { field, .. }) = &rejection else {
        panic!("expected the SUV election refusal")
    };
    assert_eq!(
        field.as_deref(),
        Some(
            "[documents/documents/0/document/Business/vehicles/0/method/Actual/history/New/first_year/Section179/elected]"
        )
    );
    rejected_with(rejection, "ASSET_179_ABOVE_SUV_LIMIT");
    assert_eq!(session.draft, before);

    // Explicit legal truck/van classification survives the same editor path;
    // unlike the SUV, its combined first-year passenger limit still applies.
    let VehicleMethod::Actual { kind, history, .. } = &mut page.vehicles[0].method else {
        unreachable!()
    };
    *kind = VehicleKind::PassengerTruckVan;
    *history = AssetHistory::New { first_year: FirstYearDeduction::Bonus };
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    assert!(matches!(
        stored_business(&session).1.assets[0].listed,
        ListedProperty::Vehicle { kind: VehicleKind::PassengerTruckVan, .. }
    ));
    assert_round_trip(&mut session, &[page]);
    let (source, _) = stored_business(&session);
    assert_eq!(
        value(&supported(&session.draft), &alloc::format!("business_{source}_asset_0_bonus")),
        20_300
    );
}
