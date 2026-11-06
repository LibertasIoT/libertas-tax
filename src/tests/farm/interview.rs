// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, owned};
use super::support::client::income_pages::{
    catalog_preview_document, open_documents, open_topic, save_documents, save_topic,
};
use crate::tests::cross::interview::{
    PAGE_INCOME, assert_round_trip, at_the_payments_page, averaging_election, base_year, business,
    cash_farm, farm_only_return, figure_with, form_8615_return, go_to_from_review, hub_rows,
    items_of, navigate, no_payments, open_payments, patr_parts, payments_value, problem_text,
    reopened_farming, row_of, save_businesses, save_payments, stored_business, stored_farm, submit,
    with_more,
};

// T2-49b: Form 2210's farmer choices follow the 2026 two-thirds test that
// the return itself shows; the 2025 question is independent of any business.
#[test]
fn farmer_choices_follow_the_2026_test_from_the_return() {
    let farmed_2025 = Payments {
        more: crate::PaymentsMore {
            penalty: figure_with(
                PriorReturn::SameReturn,
                WithholdingTiming::Evenly,
                Farmer::Yes { paid_by_march_1: true },
                Waiver::None,
            ),
            ..no_payments().more
        },
        ..no_payments()
    };
    // No farm (complete()'s $60,000 of wages only): the 2025 question, No or
    // Yes, is offered without a business, and Yes is stored as the 2025
    // answer.
    let (mut s, page) = at_the_payments_page(complete(FederalStatus::Single));
    assert_eq!(payments_value(&page).2, &[0, 1]);
    let before = s.draft.clone();
    let this_return = Payments {
        more: crate::PaymentsMore {
            penalty: figure_with(
                PriorReturn::SameReturn,
                WithholdingTiming::Evenly,
                Farmer::ThisReturn { paid_by_march_1: false },
                Waiver::None,
            ),
            ..no_payments().more
        },
        ..no_payments()
    };
    // "Yes, for 2026" is refused on save when the return does not show it.
    error(send(&mut s, save_payments(&before, this_return.clone())));
    assert_eq!(s.draft, before);
    accepted(&submit(&mut s, farmed_2025.clone()));
    let pay = s.draft.payments.clone().unwrap();
    assert!(pay.farmer_fisherman_prior_year);
    assert!(pay.farmer_paid_by_march);

    // A farm with $120,000 of gross income (net $0) beside the $60,000 of
    // wages: $120,000 is two-thirds of $180,000, so only "Yes, for 2026" is
    // offered, the 2025 answer is not asked, and the 2027 answer is not
    // needed either: T2-49c offers Not needed alone.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(120_000, 120_000)));
    let (mut s, page) = at_the_payments_page(d);
    assert_eq!(payments_value(&page).2, &[2]);
    assert_eq!(payments_value(&page).3, &[FarmerNextYear::NotNeeded as i32]);
    let before = s.draft.clone();
    error(send(&mut s, save_payments(&before, farmed_2025)));
    let with_worksheet = Payments {
        more: crate::PaymentsMore {
            next_year: NextYearEstimate::SameAsThisReturn {
                farmer_fisherman_next_year: FarmerNextYear::NotNeeded,
            },
            penalty: figure_with(
                PriorReturn::SameReturn,
                WithholdingTiming::Evenly,
                Farmer::ThisReturn { paid_by_march_1: true },
                Waiver::None,
            ),
            ..this_return.more
        },
        ..this_return
    };
    accepted(&submit(&mut s, with_worksheet.clone()));
    let pay = s.draft.payments.clone().unwrap();
    assert!(!pay.farmer_fisherman_prior_year);
    assert!(pay.farmer_paid_by_march);
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:?}", r.issues);
    assert!(r.lines.iter().any(|l| l.id == "form_1040_es_line_12a"));
    let page = open_payments(&mut s);
    assert_eq!(payments_value(&page).0.more.penalty, with_worksheet.more.penalty);
    assert_eq!(payments_value(&page).0.more.next_year, with_worksheet.more.next_year);

    // Removing the farm makes the 2027 question required again without changing the accepted plan.
    let mut d = s.draft.clone();
    d.income.retain(|e| !matches!(e.income, FederalIncome::Business { .. }));
    let mut s = restart(d);
    let before = s.draft.clone();
    let request = navigate(&s.draft, TaxSection::Payments);
    let Reply::Response(P::BeginPayments { allowed_farmer_next_year, value, .. }) =
        send(&mut s, request)
    else {
        panic!("required payments construction");
    };
    assert_eq!(
        allowed_farmer_next_year,
        vec![FarmerNextYear::Yes as i32, FarmerNextYear::No as i32]
    );
    assert!(matches!(
        value.more.next_year,
        crate::federal_model::NextYearInitial::SameAsThisReturn
    ));
    assert!(matches!(value.more.penalty, PenaltyChoice::Figure { farmer: Farmer::No, .. }));
    assert_eq!(s.draft, before);
    let request = save_payments(
        &s.draft,
        Payments {
            more: crate::PaymentsMore {
                next_year: NextYearEstimate::SameAsThisReturn {
                    farmer_fisherman_next_year: FarmerNextYear::No,
                },
                ..no_payments().more
            },
            ..no_payments()
        },
    );
    accepted(&send(&mut s, request));
    let mut s = restart(s.draft);
    assert_eq!(
        payments_value(&open_payments(&mut s)).0.more.next_year,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No }
    );
}

// T2-49c: the 2027 farmer and fisherman answer is offered from the 2026
// test the return shows, and the save checks the offered set again.
#[test]
fn farmer_2027_choices_follow_the_2026_test_and_are_checked_on_save() {
    let with_answer = |answer: FarmerNextYear| Payments {
        more: crate::PaymentsMore {
            next_year: NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: answer },
            ..no_payments().more
        },
        ..no_payments()
    };
    // 2026-10-16 D05: No farm offers required Yes/No; the removed unknown is never supplied.
    // and Not needed is refused on the server.
    let (mut s, page) = at_the_payments_page(complete(FederalStatus::Single));
    assert_eq!(payments_value(&page).3, &[FarmerNextYear::Yes as i32, FarmerNextYear::No as i32]);
    let before = s.draft.clone();
    error(send(&mut s, save_payments(&before, with_answer(FarmerNextYear::NotNeeded))));
    assert_eq!(s.draft, before);
    accepted(&submit(&mut s, with_answer(FarmerNextYear::No)));
    // The running estimate checks the same set.
    let request = P::EstimatePayments {
        allowed_refund_management: vec![],

        user: TEST_USER,
        allowed_prior_returns: vec![0],
        allowed_farmer: vec![0, 1],
        allowed_farmer_next_year: vec![FarmerNextYear::NotNeeded as i32],
        value: with_answer(FarmerNextYear::NotNeeded).into(),
    };
    error(send(&mut s, request));

    // A farm with $120,000 of gross income beside the $60,000 of wages
    // qualifies for 2026 (120,000 / 180,000 = 2/3): Not needed alone is
    // offered, a stored Yes reopens as Not needed, and Yes and No
    // are each refused on the server.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(120_000, 120_000)));
    let mut stored = d.payments.clone().unwrap_or_else(FederalPayments::none);
    stored.next_year = NextYearEstimateRecord::SameAsThisReturn {
        farmer_fisherman_next_year: FarmerNextYear::Yes,
    };
    d.payments = Some(stored);
    let (mut s, page) = at_the_payments_page(d);
    assert_eq!(payments_value(&page).3, &[FarmerNextYear::NotNeeded as i32]);
    assert_eq!(
        payments_value(&page).0.more.next_year,
        NextYearEstimate::SameAsThisReturn {
            farmer_fisherman_next_year: FarmerNextYear::NotNeeded
        }
    );
    let before = s.draft.clone();
    for answer in [FarmerNextYear::Yes, FarmerNextYear::No] {
        error(send(&mut s, save_payments(&before, with_answer(answer))));
        assert_eq!(s.draft, before);
    }
    accepted(&submit(&mut s, with_answer(FarmerNextYear::NotNeeded)));
    assert_eq!(
        s.draft.payments.as_ref().unwrap().next_year,
        NextYearEstimateRecord::SameAsThisReturn {
            farmer_fisherman_next_year: FarmerNextYear::NotNeeded
        }
    );
}

// T2-49b: the commercial fishing marker on the business page.
#[test]
fn commercial_fishing_round_trips_through_the_business_page() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut b = business(5_000_000);
    b.more.commercial_fishing = true;
    accepted(&save_businesses(&mut session, vec![b.clone()]));
    let (_, data) = stored_business(&session);
    assert!(data.commercial_fishing);
    assert_round_trip(&mut session, &[b]);
}

#[test]
fn patr_document_maps_onto_the_cooperative_record_and_back() {
    let mut original = complete(FederalStatus::Single);
    // The farm that receives the PATR; the Farming page refuses it without one.
    add(&mut original, income(crate::tests::support::builders::related_forms::receiving_farm()));
    let mut session = Interview::new(TEST_USER, original.clone());
    let coop = |d: &FederalDraft| {
        d.income
            .iter()
            .find(|e| matches!(e.income, FederalIncome::Cooperative { .. }))
            .unwrap()
            .clone()
    };
    let mut patr = Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage: 1_600_000,
        nonpatronage: 100_000,
        per_unit_retains: 300_000,
        withholding: 30_000,
        redeemed: 200_000,
        more: PatrMore { qualified_items: 1_900_000, ..Default::default() },
    };
    patr_parts(&mut patr, 200_000);
    accepted(
        &({
            let request = save_documents(
                &mut session,
                vec![document(0, IncomeSource::Farming(cooperative_document(patr.clone())))],
            );
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        }),
    );
    let FederalIncome::Cooperative { data } = &coop(&session.draft).income else { panic!() };
    assert_eq!(
        [
            data.form.patronage,
            data.form.nonpatronage,
            data.form.per_unit_retains,
            data.form.redeemed,
            data.form.withholding,
        ],
        [1_600_000, 100_000, 300_000, 200_000, 30_000]
    );
    assert_eq!(data.more.allocations[0].personal.patronage, 200_000);
    assert!(data.more.allocations[0].id > coop(&session.draft).id);
    patr.more.allocations[0].id = data.more.allocations[0].id;
    patr.more.credit_facts.activity = data.more.credit_facts.activity;
    assert!(patr.more.credit_facts.activity.is_some(), "the single farm receives the PATR");
    assert_eq!(Patr1099::from(data).more, patr.more);
    let P::SaveDocuments { documents, .. } = open_documents(&mut session) else { panic!() };
    let IncomeSource::Farming(expected) = expected_named_source(
        &session.draft,
        Owner::Taxpayer,
        IncomeSource::Farming(cooperative_document(patr)),
    ) else {
        unreachable!()
    };
    assert_eq!(documents.other.farming.documents[0].document, expected);
    // The single farm receives the PATR: nothing is left for the Review.
    let r = estimate(&session.draft);
    assert!(
        crate::related_forms::effective_claimants(&session.draft).contains_key(
            &crate::related_forms::catalog(&session.draft)
                .into_iter()
                .find(|row| row.kind == crate::related_forms::DocumentKind::Patr1099)
                .unwrap()
                .id
        )
    );
    assert!(!r.issues.iter().any(|issue| matches!(
        issue.display.key.as_str(),
        "ISSUE_RELATED_FORM_ACTIVITY_MISSING" | "ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN"
    )));
    let FederalIncome::Cooperative { data } = &coop(&session.draft).income else { panic!() };
    assert_eq!(data.form.withholding, 30_000);
}

#[test]
fn patr_box_rules_reject_on_both_sides() {
    let original = complete(FederalStatus::Single);
    let attempt = |patr: Patr1099| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = {
            let request = save_documents(
                &mut session,
                vec![document(0, IncomeSource::Farming(cooperative_document(patr)))],
            );
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                request,
            )
        };
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            assert_eq!(session.draft, original);
        }
        reply
    };
    let base = Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage: 100_000,
        nonpatronage: 20_000,
        per_unit_retains: 30_000,
        redeemed: 50_000,
        withholding: 0,
        more: Default::default(),
    };
    let mut v = base.clone();
    v.withholding = 200_000;
    accepted(&attempt(v));
    let mut v = base.clone();
    v.withholding = 200_001;
    rejected_with(attempt(v), "PATR_WITHHOLDING_ABOVE_DISTRIBUTIONS");
    let mut v = base.clone();
    patr_parts(&mut v, 100_000);
    accepted(&attempt(v));
    let mut v = base.clone();
    patr_parts(&mut v, 100_001);
    let mut session = Interview::new(TEST_USER, original.clone());
    let desired = vec![document(0, IncomeSource::Farming(cooperative_document(v)))];
    let request = save_documents(&mut session, desired.clone());
    accepted(&send(&mut session, request));
    let id = session.draft.income.last().unwrap().id;
    let request = crate::tests::support::client::income_pages::information_return_answers(
        &mut session,
        &desired,
    );
    let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
    let row =
        followups.iter().find(|row| row.key == alloc::format!("income/{id}/allocations")).unwrap();
    let path: alloc::string::String = libertas::SchemaPath::root()
        .field("followups")
        .key(row.key.as_str())
        .field("answer")
        .field("value")
        .index(0)
        .field("personal")
        .field("patronage")
        .into();
    let before = session.draft.clone();
    let reply = send(&mut session, request);
    assert_eq!(session.draft, before);
    let Reply::Response(P::Problem { field, .. }) = &reply else {
        panic!("personal patronage above box 1 must be refused")
    };
    assert_eq!(field, &Some(path));
    // The canonical owner now carries the original allocation relationship's
    // named diagnostic, while the same equality/+1 boundary remains intact.
    rejected_with(reply, "COOP_PERSONAL_ABOVE_BOX1");
    let mut v = base.clone();
    v.more.qualified_items = 200_000;
    accepted(&attempt(v));
    let mut v = base;
    v.more.qualified_items = 200_001;
    rejected_with(attempt(v), "PATR_QUALIFIED_ITEMS_ABOVE_DISTRIBUTIONS");
}

#[test]
fn patr_missing_source_preserves_withholding_and_names_required_recipient() {
    let original = complete(FederalStatus::Single);
    let mut session = Interview::new(TEST_USER, original.clone());
    let patr = Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage: 100_000,
        nonpatronage: 0,
        per_unit_retains: 0,
        redeemed: 0,
        withholding: 1_000,
        more: Default::default(),
    };
    accepted(
        &({
            let request = save_documents(
                &mut session,
                vec![document(0, IncomeSource::Farming(cooperative_document(patr)))],
            );
            send(&mut session, request)
        }),
    );
    let r = estimate(&session.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    let FederalIncome::Cooperative { data } = &session.draft.income[1].income else { panic!() };
    assert_eq!(data.form.withholding, 1_000);
    assert!(r.lines.is_empty());
    missing_related_activity(
        &session.draft,
        &r,
        crate::related_forms::DocumentKind::Patr1099,
        &[TaxSection::SelfEmployment, TaxSection::Farming],
    );
}
// Every branch of the farm kind at once.
fn farm_parcel_fixture(acquired: u32, basis: i64) -> FarmLandBasis {
    FarmLandBasis {
        id: 0,
        label: "Parcel source".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired,
        regular: basis,
        amt: basis,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![],
    }
}
// Exercise the real save/reopen/select path; an unsaved parcel is not an ID.
fn farm_parcel_parts(
    document: &mut IncomeSource,
) -> Option<(&mut Vec<FarmLandBasis>, &mut Vec<LandSale>, &mut Conservation)> {
    match document {
        IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } => Some((
            &mut farm.more.land_basis,
            &mut farm.more.land_sales,
            &mut farm.more.conservation,
        )),
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(farm) } => Some((
            &mut farm.more.land_basis,
            &mut farm.more.land_sales,
            &mut farm.more.conservation,
        )),
        _ => None,
    }
}
// Explicit fixture setup through the ordinary Save/reopen/select path.
// No fixture invents an accepted ID, nor does the App assign IDs on preview.
pub(in crate::tests) fn accept_farm_sale_parcels(
    session: &mut Interview,
    items: &mut [IncomeDocumentItem],
) {
    let mut needs_save = false;
    for item in items.iter_mut() {
        if let Some((land, sales, conservation)) = farm_parcel_parts(&mut item.document) {
            let has_costs = !matches!(conservation, Conservation::None);
            if !sales.is_empty() || has_costs {
                if land.is_empty() {
                    land.push(farm_parcel_fixture(20100301, 0));
                }
                needs_save |= item.id == 0 || land.iter().any(|parcel| parcel.id == 0);
            }
        }
    }
    if !needs_save {
        return;
    }
    let mut initial = items.to_vec();
    for item in &mut initial {
        if let Some((_, sales, conservation)) = farm_parcel_parts(&mut item.document) {
            sales.clear();
            *conservation = Conservation::None;
        }
    }
    accepted(&{
        let request = save_topic(session, TaxSection::Farming, initial);
        send(session, request)
    });
    let P::SaveIncomes { documents, .. } = open_topic(session, TaxSection::Farming) else {
        unreachable!()
    };
    let accepted_rows = documents
        .documents
        .into_iter()
        .map(crate::IncomeRecordItem::into_income)
        .collect::<Vec<_>>();
    for item in items {
        let name = match &item.document {
            IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } => {
                Some((false, farm.name.clone()))
            }
            IncomeSource::Record { value: crate::IncomeRecord::FarmRental(farm) } => {
                Some((true, farm.name.clone()))
            }
            _ => None,
        };
        let Some((rental, name)) = name else {
            continue;
        };
        let matches = accepted_rows
            .iter()
            .filter(|record| match &record.document {
                IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } => {
                    !rental && farm.name == name
                }
                IncomeSource::Record { value: crate::IncomeRecord::FarmRental(farm) } => {
                    rental && farm.name == name
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "one accepted source for the synthetic farm fixture");
        let mut accepted = (*matches[0]).clone();
        item.id = accepted.id;
        let (accepted_land, _, _) = farm_parcel_parts(&mut accepted.document).unwrap();
        let (land, sales, conservation) = farm_parcel_parts(&mut item.document).unwrap();
        *land = accepted_land.clone();
        if land.is_empty() {
            continue;
        }
        for sale in sales {
            if sale.basis_property == 0 {
                sale.basis_property = land[0].id;
            }
        }
        match conservation {
            Conservation::Deduct { costs, assessments, carry, .. } => {
                for cost in costs {
                    if cost.property == 0 {
                        cost.property = land[0].id;
                    }
                }
                for record in assessments {
                    if record.property == 0 {
                        record.property = land[0].id;
                    }
                }
                for record in carry {
                    if record.property == 0 {
                        record.property = land[0].id;
                    }
                }
            }
            Conservation::Capitalize { costs, .. } => {
                for cost in costs {
                    if cost.property == 0 {
                        cost.property = land[0].id;
                    }
                }
            }
            Conservation::None => (),
        }
    }
}

fn full_farm() -> Farm {
    let mut farm = cash_farm(8_000_000, 500_000);
    farm.name = "North field corn".into();
    farm.accounting = FarmMethod::Cash {
        resale: 1_200_000,
        resale_cost: 900_000,
        raised: 8_000_000,
        program: 150_000,
        custom_hire: 60_000,
        other: 40_000,
    };
    farm.insurance = CropInsurance::Payments {
        physical: vec![CropLoss { crop: "Grain".into(), amount: 700_000, year: 2026 }],
        price_only: 100_000,
        from_prior_year: 50_000,
        withholding: 10_000,
    };
    farm.expenses = FarmExpenses {
        chemicals: 110_000,
        custom_hire: 120_000,
        employee_benefits: 130_000,
        feed: 140_000,
        fertilizers: 150_000,
        freight: 160_000,
        fuel: 170_000,
        insurance: 180_000,
        mortgage_interest: 190_000,
        other_interest: 200_000,
        labor: 210_000,
        pension: 220_000,
        equipment_rent: 230_000,
        other_rent: 240_000,
        repairs: 250_000,
        seeds: 260_000,
        storage: 270_000,
        supplies: 280_000,
        taxes: 290_000,
        utilities: 300_000,
        veterinary: 310_000,
        meals: 20_000,
        other: vec![OtherExpense { description: "Soil testing".into(), amount: 30_000 }],
    };
    farm.vehicles = vec![Vehicle {
        method: VehicleMethod::StandardMileage {
            id: 0,
            first_half: 1_000,
            second_half: 1_500,
            total: 2_500,
            more: MileageMore::default(),
        },
    }];
    farm.assets = vec![Asset {
        farm_rental_use: None,
        allowed_farm_rental_use: alloc::vec::Vec::new(),
        macrs_source: None,
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost: 5_000_000,
        credit_reduction: 0,
        placed: 20150601,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class: AssetClass::Seven,
                history: AssetHistory::Continuing {
                    prior_depreciation: 4_000_000,
                    first_year_179: 0,
                    first_year_bonus: 0,
                    carryforward: 0,
                    mid_quarter: false,
                },
            },
        },
        business_percent: 100,
        acquired: None,
        disposal: AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    }];
    let year = |prepaid, other| SupplyYear { prepaid, other };
    with_more(farm, |more| {
        let FarmDetails {
            ccc,
            crp,
            supplies,
            conservation,
            land_sales,
            land_basis,
            payroll,
            withholding,
            optional_se,
            qbi_group,
            ..
        } = more;
        *ccc = CccLoans::Loans {
            method: CccMethod::Loan,
            loans: vec![
                CccLoan::New {
                    id: 0,
                    principal: 1_000_000,
                    outcome: CccOutcome::Redeemed { paid: 900_000, sale: Some(1_100_000) },
                },
                CccLoan::Earlier {
                    id: 0,
                    taxed: false,
                    principal: 500_000,
                    outcome: CccDisposition::Redeemed { year: 2026, paid: 450_000, sale: None },
                },
            ],
            withholding: 0,
            source: ccc_source_input(&CccMethod::Loan),
        };
        *crp = vec![CrpItem { amount: 300_000, withholding: 0, benefits: true }];
        *supplies = PrepaidSupplies::Supplies {
            advances: vec![SupplyPurchase {
                amount: 200_000,
                purchase: farm_supply_purchase_source(),
            }],
            carried: vec![CarriedCost { balance: 50_000, amt_balance: 50_000, used: true }],
            exception: SupplyException::ThreeYears {
                relation: FarmSupplyRelation::Business,
                year_third_prior: year(100_000, 900_000),
                year_second_prior: year(110_000, 950_000),
                year_prior: year(120_000, 990_000),
            },
            poultry: PoultryRecords {
                current: vec![],
                resale: vec![PoultryResale {
                    label: "Broiler lot".into(),
                    method: PoultryMethod::Sold,
                    year: 2026,
                    cost: 80_000,
                    deducted: 0,
                    disposed: true,
                }],
                production: vec![],
            },
        };
        *conservation = Conservation::Capitalize {
            method: FarmConservationCapitalMethod::NeverElected,
            costs: vec![ConservationCostInput {
                property: 0,
                // Neutral actual current timing completion for this synthetic sale fixture.
                paid_or_incurred: Some(20260601),
                facts: conservation_facts(),
                label: "Terracing".into(),
                amount: 70_000,
            }],
        };
        *land_basis = vec![farm_parcel_fixture(20100301, 8_000_000)];
        *land_sales = vec![LandSale {
            farm_rental_use: None,
            allowed_farm_rental_use: alloc::vec::Vec::new(),
            label: "East 40 acres".into(),
            id: 0,
            basis_property: 0,
            sold: 20260910,
            proceeds: 20_000_000,
            selling: 500_000,
            installments: LandInstallments::Yes,
            situation: FarmLandSituation::Direct,
            payment: crate::SalePayment::InFull,
        }];
        *payroll = BusinessPayroll::Yes {
            statements: vec![PayrollW2 { label: "Hand".into(), wages: 210_000, medicare: 210_000 }],
            attribution: crate::PayrollQbiAttribution::EntireBusiness,
        };
        *withholding = 5_000;
        *optional_se = true;
        *qbi_group = "Farm group".into();
    })
}

#[test]
fn farm_page_maps_every_branch_onto_the_stored_farm_and_back() {
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut documents = vec![
        document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(full_farm()) }),
        document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::FarmAveraging(averaging_election()),
            },
        ),
        document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::FarmLoss(FarmLossWaiver {
                    choice: FarmLossChoice::Waive,
                }),
            },
        ),
    ];
    accept_farm_sale_parcels(&mut session, &mut documents);
    accepted(&{
        let request = save_topic(&mut session, TaxSection::Farming, documents.clone());
        send(&mut session, request)
    });
    accepted(&answer_crop_timing(&mut session, CropPaperTiming::Defer));
    accepted(&answer_crop_practice(&mut session, "Grain", 750_000));
    let d = session.draft.clone();
    assert_eq!(d.income[0].label, "North field corn");
    let (data, farm) = stored_farm(&d);
    // Part I: one receipt per nonzero line, in form order.
    assert_eq!(
        farm.accounting,
        FarmAccounting::Cash {
            receipts: vec![
                FarmReceipt::Resale { amount: 1_200_000, cost: 900_000 },
                FarmReceipt::Raised { amount: 8_000_000 },
                FarmReceipt::Program { amount: 150_000 },
                FarmReceipt::CustomHire { amount: 60_000 },
                FarmReceipt::Other { amount: 40_000 },
            ]
        }
    );
    // Line 6: the physical loss, then the price-only portion; Defer writes
    // both confirmations Yes.
    assert_eq!(
        farm.insurance,
        FarmInsuranceRecords::Payments {
            current: vec![
                CropPayment::Physical { crop: "Grain".into(), amount: 700_000, year: 2026 },
                CropPayment::Other { amount: 100_000 },
            ],
            previous_deferred: 50_000,
            withholding: 10_000,
        }
    );
    // Part II: 22 fixed lines plus line 32's row; the ten Schedule F lines
    // without a Schedule C category store their own farm-only categories
    // (T2-45y), and line 32 keeps its description.
    assert_eq!(data.expenses.len(), 23);
    for (category, amount) in [
        (BusinessExpenseCategory::Chemicals, 110_000),
        (BusinessExpenseCategory::CustomHire, 120_000),
        (BusinessExpenseCategory::Feed, 140_000),
        (BusinessExpenseCategory::Fertilizers, 150_000),
        (BusinessExpenseCategory::Freight, 160_000),
        (BusinessExpenseCategory::Fuel, 170_000),
        (BusinessExpenseCategory::EmployeePension, 220_000),
        (BusinessExpenseCategory::Seeds, 260_000),
        (BusinessExpenseCategory::Storage, 270_000),
        (BusinessExpenseCategory::Veterinary, 310_000),
        (BusinessExpenseCategory::Other { label: "Soil testing".into() }, 30_000),
    ] {
        assert!(
            data.expenses.contains(&ordinary_business_expense(category, amount)),
            "{amount}: {:?}",
            data.expenses
        );
    }
    assert!(!data.expenses.iter().any(|e| matches!(
        &e.category,
        BusinessExpenseCategory::Other { label } if label.starts_with("Line ")
    )));
    assert!(
        data.expenses.contains(&ordinary_business_expense(BusinessExpenseCategory::Wages, 210_000))
    );
    assert!(data.expenses.contains(&FederalBusinessExpense {
        category: BusinessExpenseCategory::Meals,
        amount: 20_000,
        treatment: BusinessExpenseKind::Meals,
    }));
    // F9: the 2015 seven-year asset uses 150% declining balance.
    assert_eq!(data.assets[0].method, EquipmentMethod::Declining150);
    assert_eq!(data.vehicles.len(), 1);
    // The More fold's records.
    let ccc = farm.ccc.as_ref().unwrap();
    assert!(ccc.source.producer_owned_raised);
    // Saving the farm assigned each loan its own identity.
    assert!(ccc.lots.iter().all(|lot| lot.id != 0));
    assert_ne!(ccc.lots[0].id, ccc.lots[1].id);
    assert_eq!(
        ccc.lots[0],
        CccLot {
            id: ccc.lots[0].id,
            origin: CccOrigin::Current,
            principal: 1_000_000,
            disposition: CccDisposition::Redeemed {
                year: 2026,
                paid: 900_000,
                sale: Some(1_100_000)
            },
        }
    );
    assert_eq!(ccc.lots[1].origin, CccOrigin::PriorExcluded);
    assert!(farm.crp[0].benefits);
    let supplies = farm.supplies.as_ref().unwrap();
    assert!(supplies.retained[0].consumed);
    let FarmSupplyException::History { years, .. } = &supplies.exception else {
        panic!("{:?}", supplies.exception)
    };
    assert_eq!(years.iter().map(|y| y.year).collect::<Vec<_>>(), vec![2023, 2024, 2025]);
    assert!(supplies.poultry[0].disposed);
    assert!(matches!(farm.conservation, Some(FarmConservation::Capitalize { .. })));
    assert_eq!(farm.land_sales[0].situation, FarmLandSituation::Direct);
    assert!(farm.land_sales[0].deferral);
    assert!(farm.specials.limitations.is_empty());
    assert_eq!(data.withholding, 5_000);
    assert_eq!(data.qbi_aggregation_group.as_deref(), Some("Farm group"));
    assert!(!data.specified_service);
    assert_eq!(data.income_source, BusinessIncomeSource::Services);
    // The return-level kinds and the optional method (F6).
    let screening = d.screening.as_ref().unwrap();
    assert_eq!(
        screening.optional_se,
        vec![SeElection { owner: Owner::Taxpayer, farm: true, nonfarm: None }]
    );
    assert_eq!(screening.farm_nol, Some(FarmNolElection::Waive));
    let averaging = screening.farm_averaging.as_ref().unwrap();
    assert_eq!(averaging.selection.history_scope, FarmAveragingHistoryScope::CompleteSupported);
    assert_eq!(averaging.years.len(), 3);
    // Reopen rebuilds the same documents, and saving them again stores
    // nothing new.
    let (_, reopened) = reopened_farming(&mut session);
    // The reopened page carries the identities the save assigned to the loans.
    let mut expected: Vec<IncomeSource> = documents.iter().map(|i| i.document.clone()).collect();
    if let IncomeSource::Record { value: crate::IncomeRecord::Farm(page) } = &mut expected[0] {
        bind_saved_property_ids(&mut page.assets, &mut page.vehicles, &data.assets);
        crate::tests::support::client::bind_saved_mileage_ids(&mut page.vehicles, &data.vehicles);
    }
    if let IncomeSource::Record { value: crate::IncomeRecord::Farm(page) } = &mut expected[0]
        && let CccLoans::Loans { loans, .. } = &mut page.more.ccc
    {
        for (loan, lot) in loans.iter_mut().zip(&ccc.lots) {
            let (CccLoan::New { id, .. } | CccLoan::Earlier { id, .. }) = loan;
            *id = lot.id;
        }
    }
    if let IncomeSource::Record { value: crate::IncomeRecord::Farm(page) } = &mut expected[0] {
        for (sale, record) in page.more.land_sales.iter_mut().zip(&farm.land_sales) {
            sale.id = record.id;
        }
    }
    assert_eq!(reopened, expected);
    let revision = session.draft.revision;
    let items: Vec<IncomeDocumentItem> = {
        let P::SaveIncomes { documents, .. } = open_topic(&mut session, TaxSection::Farming) else {
            unreachable!()
        };
        documents.documents.into_iter().map(crate::IncomeRecordItem::into_income).collect()
    };
    accepted(&{
        let request = save_topic(&mut session, TaxSection::Farming, items);
        send(&mut session, request)
    });
    assert_eq!(session.draft.revision, revision);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn accrual_farm_and_cost_sharing_round_trip() {
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut accrual = cash_farm(0, 100_000);
    accrual.accounting = FarmMethod::Accrual {
        sales: 5_000_000,
        program: 10_000,
        custom_hire: 20_000,
        other: 30_000,
        opening: 1_000_000,
        purchases: 400_000,
        closing: 1_500_000,
    };
    // A cash farm's section 126 payment is the CostSharing receipt.
    let sharing = with_more(cash_farm(1_000_000, 0), |more| {
        more.cost_sharing = 25_000;
    });
    let documents = vec![
        document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(accrual) }),
        document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(sharing) }),
    ];
    accepted(&{
        let request = save_topic(&mut session, TaxSection::Farming, documents.clone());
        send(&mut session, request)
    });
    let FederalIncome::Business { data } = &session.draft.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &data.activity else { unreachable!() };
    assert_eq!(
        farm.accounting,
        FarmAccounting::Accrual {
            sales: 5_000_000,
            opening: 1_000_000,
            closing: 1_500_000,
            purchases: 400_000,
            custom_hire: 20_000,
            program: 10_000,
            other: 30_000,
        }
    );
    let FederalIncome::Business { data } = &session.draft.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &data.activity else { unreachable!() };
    assert_eq!(
        farm.accounting,
        FarmAccounting::Cash {
            receipts: vec![
                FarmReceipt::Raised { amount: 1_000_000 },
                FarmReceipt::CostSharing { amount: 25_000 },
            ]
        }
    );
    let (_, reopened) = reopened_farming(&mut session);
    assert_eq!(reopened, documents.iter().map(|i| i.document.clone()).collect::<Vec<_>>());
    // F14: the accrual Part III lines are tagged (on the accrual farm
    // alone: the cost-sharing receipt is refused by name).
    let mut accrual_only = session.draft.clone();
    accrual_only.income.truncate(1);
    let r = supported(&accrual_only);
    let lines: Vec<(alloc::string::String, i64)> = crate::draft_return(&r)
        .iter()
        .filter_map(|row| {
            row.form_line
                .as_ref()
                .filter(|f| f.form == TaxForm::ScheduleF)
                .map(|f| (f.line.clone(), row.amount))
        })
        .collect();
    for (line, amount) in [("37", 5_000_000), ("45", 1_000_000), ("46", 400_000), ("48", 1_500_000)]
    {
        assert!(lines.contains(&(line.into(), amount)), "{line}: {lines:?}");
    }
}

#[test]
fn farm_client_rules_reject_on_the_server_at_their_boundary() {
    let original = farm_only_return();
    let attempt = |documents: Vec<IncomeDocumentItem>| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut documents = documents;
        accept_farm_sale_parcels(&mut session, &mut documents);
        {
            let request = save_topic(&mut session, TaxSection::Farming, documents);
            send(&mut session, request)
        }
    };
    let farm =
        |f: Farm| vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(f) })];
    // FARM_COST_SHARING_NEEDS_CASH: accrual with a cost-sharing amount.
    let accrual = |sharing: i64| {
        let mut f = with_more(cash_farm(0, 0), |more| {
            more.cost_sharing = sharing;
        });
        f.accounting = FarmMethod::Accrual {
            sales: 100_000,
            program: 0,
            custom_hire: 0,
            other: 0,
            opening: 0,
            purchases: 0,
            closing: 0,
        };
        f
    };
    accepted(&attempt(farm(accrual(0))));
    rejected_with(attempt(farm(accrual(1))), "FARM_COST_SHARING_NEEDS_CASH");
    // CCC_REPAID_ABOVE_PRINCIPAL and CCC_DEFICIENCY_ABOVE_PRINCIPAL.
    let ccc = |outcome: CccOutcome| {
        with_more(cash_farm(1_000_000, 0), |more| {
            more.ccc = CccLoans::Loans {
                method: CccMethod::Loan,
                loans: vec![CccLoan::New { id: 0, principal: 100_000, outcome }],
                withholding: 0,
                source: ccc_source_input(&CccMethod::Loan),
            };
        })
    };
    let redeemed = |paid| CccOutcome::Redeemed { paid, sale: None };
    accepted(&attempt(farm(ccc(redeemed(100_000)))));
    rejected_with(attempt(farm(ccc(redeemed(100_001)))), "CCC_REPAID_ABOVE_PRINCIPAL");
    let forfeited = |deficiency| CccOutcome::Forfeited { surplus: 0, deficiency };
    accepted(&attempt(farm(ccc(forfeited(100_000)))));
    rejected_with(attempt(farm(ccc(forfeited(100_001)))), "CCC_DEFICIENCY_ABOVE_PRINCIPAL");
    // CONSERVATION_SHARE_ABOVE_TOTAL.
    let assessment = |share, paid| {
        with_more(cash_farm(1_000_000, 0), |more| {
            more.conservation = Conservation::Deduct {
                costs: vec![],
                assessments: vec![ConservationAssessmentInput {
                    property: 0,
                    facts: conservation_facts(),
                    label: "District drain".into(),
                    total: 100_000,
                    share,
                    payments: vec![ConservationAssessmentPayment {
                        current_date: None,
                        year: 2026,
                        amount: paid,
                    }],
                }],
                carry: vec![],
                method: conservation_method(),
            };
        })
    };
    accepted(&attempt(farm(assessment(100_000, 1_000))));
    rejected_with(attempt(farm(assessment(100_001, 1_000))), "CONSERVATION_SHARE_ABOVE_TOTAL");
    // FARM_LAND_SOLD_BEFORE_ACQUIRED: the same day passes.
    let land = |acquired, sold| {
        with_more(cash_farm(1_000_000, 0), |more| {
            more.land_basis = vec![farm_parcel_fixture(acquired, 50_000)];
            more.land_sales = vec![LandSale {
                farm_rental_use: None,
                allowed_farm_rental_use: alloc::vec::Vec::new(),
                label: "Parcel".into(),
                id: 0,
                basis_property: 0,
                sold,
                proceeds: 100_000,
                selling: 0,
                installments: LandInstallments::No,
                situation: FarmLandSituation::Direct,
                payment: crate::SalePayment::InFull,
            }];
        })
    };
    accepted(&attempt(farm(land(20260301, 20260301))));
    rejected_with(attempt(farm(land(20260302, 20260301))), "FARM_LAND_SOLD_BEFORE_ACQUIRED");
    // AVERAGING_NEGATIVE_BASE_NEEDS_LOSS: AGI equal to the deductions passes.
    let averaging = |edit: fn(&mut FarmAveragingElection)| {
        let mut v = averaging_election();
        edit(&mut v);
        vec![
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(5_000_000, 0)) },
            ),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(v) }),
        ]
    };
    accepted(&attempt(averaging(|v| v.base_third_prior_year = base_year(1_460_000))));
    rejected_with(
        attempt(averaging(|v| v.base_third_prior_year = base_year(1_459_999))),
        "AVERAGING_NEGATIVE_BASE_NEEDS_LOSS",
    );
    // AVERAGING_SOURCE_BEFORE_BASE_YEAR: a 2024 Schedule J may refigure 2024.
    let schedule_j = |year| BaseYearSource::ScheduleJ { year, taxable: 1_000_000, tax: 100_000 };
    accepted(&attempt(averaging(|v| {
        v.base_second_prior_year.source =
            BaseYearSource::ScheduleJ { year: 2024, taxable: 1_000_000, tax: 100_000 }
    })));
    let _ = schedule_j;
    rejected_with(
        attempt(averaging(|v| {
            v.base_second_prior_year.source =
                BaseYearSource::ScheduleJ { year: 2023, taxable: 1_000_000, tax: 100_000 }
        })),
        "AVERAGING_SOURCE_BEFORE_BASE_YEAR",
    );
    // FARM_AVERAGING_TWICE and FARM_LOSS_TWICE.
    let mut twice = averaging(|_| {});
    twice.push(document(
        0,
        IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(averaging_election()) },
    ));
    rejected_with(attempt(twice), "FARM_AVERAGING_TWICE");
    let waiver = || {
        document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::FarmLoss(FarmLossWaiver {
                    choice: FarmLossChoice::Waive,
                }),
            },
        )
    };
    accepted(&attempt(vec![waiver()]));
    rejected_with(attempt(vec![waiver(), waiver()]), "FARM_LOSS_TWICE");
    // The preview checks the farm's rules too.
    let mut session = Interview::new(TEST_USER, original.clone());
    rejected_with(
        send(
            &mut session,
            catalog_preview_document(IncomeSource::Record {
                value: crate::IncomeRecord::Farm(accrual(1)),
            }),
        ),
        "FARM_COST_SHARING_NEEDS_CASH",
    );
}

#[test]
fn farm_server_only_rules_reject_at_their_boundary() {
    let original = farm_only_return();
    let attempt = |farms: Vec<(Owner, Farm)>| {
        let mut session = Interview::new(TEST_USER, original.clone());
        let mut documents = farms
            .into_iter()
            .map(|(owner, f)| {
                owned(owner, 0, IncomeSource::Record { value: crate::IncomeRecord::Farm(f) })
            })
            .collect::<Vec<_>>();
        accept_farm_sale_parcels(&mut session, &mut documents);
        {
            let request = save_topic(&mut session, TaxSection::Farming, documents);
            send(&mut session, request)
        }
    };
    let one = |f: Farm| vec![(Owner::Taxpayer, f)];
    // CROP_WITHHOLDING_ABOVE_PAYMENTS: the payments plus the 2025 amount.
    let insured = |withholding| {
        let mut f = cash_farm(1_000_000, 0);
        f.insurance = CropInsurance::Payments {
            physical: vec![CropLoss { crop: "Grain".into(), amount: 60_000, year: 2026 }],
            price_only: 30_000,
            from_prior_year: 10_000,
            withholding,
        };
        f
    };
    accepted(&attempt(one(insured(100_000))));
    rejected_with(attempt(one(insured(100_001))), "CROP_WITHHOLDING_ABOVE_PAYMENTS");
    // CONSERVATION_PAYMENTS_ABOVE_SHARE.
    let paid = |amount| {
        with_more(cash_farm(1_000_000, 0), |more| {
            more.conservation = Conservation::Deduct {
                costs: vec![],
                assessments: vec![ConservationAssessmentInput {
                    property: 0,
                    facts: conservation_facts(),
                    label: "District drain".into(),
                    total: 100_000,
                    share: 50_000,
                    payments: vec![
                        ConservationAssessmentPayment { year: 2025, amount, current_date: None },
                        ConservationAssessmentPayment {
                            current_date: None,
                            year: 2026,
                            amount: 25_000,
                        },
                    ],
                }],
                carry: vec![],
                method: conservation_method(),
            };
        })
    };
    accepted(&attempt(one(paid(25_000))));
    rejected_with(attempt(one(paid(25_001))), "CONSERVATION_PAYMENTS_ABOVE_SHARE");

    // FARM_LAND_SOLD_OUTSIDE_CURRENT_YEAR: December 31, 2026 passes, 2027 does not.
    let sold = |date| {
        with_more(cash_farm(1_000_000, 0), |more| {
            more.land_basis = vec![farm_parcel_fixture(20100101, 50_000)];
            more.land_sales = vec![LandSale {
                farm_rental_use: None,
                allowed_farm_rental_use: alloc::vec::Vec::new(),
                label: "Parcel".into(),
                id: 0,
                basis_property: 0,
                sold: date,
                proceeds: 100_000,
                selling: 0,
                installments: LandInstallments::No,
                situation: FarmLandSituation::Direct,
                payment: crate::SalePayment::InFull,
            }];
        })
    };
    accepted(&attempt(one(sold(20261231))));
    rejected_with(attempt(one(sold(20270101))), "FARM_LAND_SOLD_OUTSIDE_CURRENT_YEAR");
    rejected_with(attempt(one(sold(20251231))), "FARM_LAND_SOLD_OUTSIDE_CURRENT_YEAR");
    // FARM_WITHHOLDING_ABOVE_RECEIPTS: summed across the cash lines.
    let withheld = |amount| {
        let mut f = with_more(cash_farm(60_000, 0), |more| {
            more.withholding = amount;
        });
        let FarmMethod::Cash { other, .. } = &mut f.accounting else { unreachable!() };
        *other = 40_000;
        f
    };
    accepted(&attempt(one(withheld(100_000))));
    rejected_with(attempt(one(withheld(100_001))), "FARM_WITHHOLDING_ABOVE_RECEIPTS");
    // FARM_OPTIONAL_SE_DIFFERS: one owner's farms agree; another owner may
    // differ (a joint return's spouse).
    let se = |elected| {
        with_more(cash_farm(1_000_000, 0), |more| {
            more.optional_se = elected;
        })
    };
    accepted(&attempt(vec![(Owner::Taxpayer, se(true)), (Owner::Taxpayer, se(true))]));
    rejected_with(
        {
            let mut source = Interview::new(TEST_USER, original.clone());
            current_document_refusal(
                "income-source-2043",
                &mut source,
                vec![
                    owned(
                        Owner::Taxpayer,
                        0,
                        IncomeSource::Record { value: crate::IncomeRecord::Farm(se(true)) },
                    ),
                    owned(
                        Owner::Taxpayer,
                        0,
                        IncomeSource::Record { value: crate::IncomeRecord::Farm(se(false)) },
                    ),
                ],
                "FARM_OPTIONAL_SE_DIFFERS",
                "[documents/documents/1/document/more/optional_se]",
            )
        },
        "FARM_OPTIONAL_SE_DIFFERS",
    );
    let joint = empty_return(FederalStatus::Joint);
    let mut session = Interview::new(TEST_USER, joint.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![
                owned(
                    Owner::Taxpayer,
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Farm(se(true)) },
                ),
                owned(
                    Owner::Spouse,
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::Farm(se(false)) },
                ),
            ],
        );
        send(&mut session, request)
    });
    assert_eq!(
        session.draft.screening.as_ref().unwrap().optional_se,
        vec![SeElection { owner: Owner::Taxpayer, farm: true, nonfarm: None }]
    );
}

#[test]
fn farm_refusals_are_named_under_the_farming_page() {
    let original = farm_only_return();
    let refused = |documents: Vec<IncomeDocumentItem>, section: &str| {
        let mut session = Interview::new(TEST_USER, original.clone());
        accepted(&{
            let request = save_topic(&mut session, TaxSection::Farming, documents);
            send(&mut session, request)
        });
        let r = estimate(&session.draft);
        assert!(r.issues.iter().any(|i| i.section == section), "{section}: {:?}", r.issues);
        let items = items_of(&mut session);
        assert!(items.iter().any(|(_, _, s)| *s == TaxSection::Farming), "{section}: {items:?}");
    };
    let farm =
        |f: Farm| vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(f) })];
    // A named actual unentered accounting transition retains known farm
    // receipts while the missing required timing source prevents final tax.
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            farm(with_more(cash_farm(1_000_000, 0), |more| {
                more.specials.limitations = vec![3];
            })),
        );
        send(&mut session, request)
    });
    let result = estimate(&session.draft);
    assert_eq!(result.tax, None);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_FARM_METHOD_TRANSITION_SOURCE")
    );
    // Section 126 cost sharing (line 4b).
    refused(
        farm(with_more(cash_farm(1_000_000, 0), |more| {
            more.cost_sharing = 10_000;
        })),
        "Farm program payments",
    );
    // Actual missing CCC ownership/basis is a mandatory source boundary;
    // the accepted loan remains instead of dropping its required income.
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            farm(with_more(cash_farm(1_000_000, 0), |more| {
                more.ccc = CccLoans::Loans {
                    method: CccMethod::Loan,
                    loans: vec![CccLoan::New {
                        id: 0,
                        principal: 100_000,
                        outcome: CccOutcome::Outstanding,
                    }],
                    withholding: 0,
                    source: {
                        let mut source = ccc_source_input(&CccMethod::Loan);
                        source.producer_owned_raised = false;
                        source
                    },
                };
            })),
        );
        send(&mut session, request)
    });
    assert!(
        estimate(&session.draft)
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_CCC_COMMODITY_BASIS_SOURCE")
    );
    assert!(stored_farm(&session.draft).1.ccc.is_some());
    // An unestablished section175 method omits only its deduction with a notice.
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut documents = farm(with_more(cash_farm(1_000_000, 0), |more| {
        more.conservation = Conservation::Deduct {
            costs: vec![ConservationCostInput {
                property: 0,
                paid_or_incurred: None,
                facts: conservation_facts(),
                label: "Terracing".into(),
                amount: 10_000,
            }],
            assessments: vec![],
            carry: vec![],
            method: FarmConservationMethod::Unestablished,
        };
    }));
    accept_farm_sale_parcels(&mut session, &mut documents);
    accepted(&{
        let request = save_topic(&mut session, TaxSection::Farming, documents);
        send(&mut session, request)
    });
    assert!(noted(&supported(&session.draft), "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
    // Schedule J's Something else applies: averaging is left out.
    let mut averaging = averaging_election();
    averaging.selection.history_scope = FarmAveragingHistoryScope::ShortYearOrForeign;
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::Farm(cash_farm(5_000_000, 0)),
                    },
                ),
                document(
                    0,
                    IncomeSource::Record { value: crate::IncomeRecord::FarmAveraging(averaging) },
                ),
            ],
        );
        send(&mut session, request)
    });
    assert!(noted(&supported(&session.draft), "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT"));
    // The loss carried back (refused by name), with a farm loss making an
    // NOL; the Farming page then offers the loss kind (F7).
    let mut session = Interview::new(TEST_USER, original.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Farm(cash_farm(100_000, 3_000_000)),
                },
            )],
        );
        send(&mut session, request)
    });
    let (allowed, _) = reopened_farming(&mut session);
    let loss_kind = libertas_macros::variant_index!(crate::IncomeSourceKind::FarmLoss) as i32;
    let averaging_kind =
        libertas_macros::variant_index!(crate::IncomeSourceKind::FarmAveraging) as i32;
    assert!(allowed.contains(&loss_kind), "{allowed:?}");
    assert!(!allowed.contains(&averaging_kind), "{allowed:?}");
    assert!(estimate(&session.draft).issues.iter().any(|i| i.section == "Farm loss"));
    let farm_item = document(
        session.draft.income[0].id,
        IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(100_000, 3_000_000)) },
    );
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![
                farm_item.clone(),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::FarmLoss(FarmLossWaiver {
                            choice: FarmLossChoice::Carryback,
                        }),
                    },
                ),
            ],
        );
        send(&mut session, request)
    });
    let r = estimate(&session.draft);
    assert!(r.issues.is_empty(), "{r:?}");
    assert!(r.review_notes.iter().any(|n| n.display.key == "NOTE_FARMING_LOSS_CARRYBACK_KEPT"));
    // Waiving it computes the carryforward with the waiver note instead.
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![
                farm_item,
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::FarmLoss(FarmLossWaiver {
                            choice: FarmLossChoice::Waive,
                        }),
                    },
                ),
            ],
        );
        send(&mut session, request)
    });
    assert!(!estimate(&session.draft).issues.iter().any(|i| i.section == "Farm loss"));
}

#[test]
fn farming_page_offers_schedule_j_only_with_a_farm_profit() {
    // F7: an empty return offers neither return-level farm kind; a profit
    // offers Schedule J; a stored kind stays offered.
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let loss_kind = libertas_macros::variant_index!(crate::IncomeSourceKind::FarmLoss) as i32;
    let averaging_kind =
        libertas_macros::variant_index!(crate::IncomeSourceKind::FarmAveraging) as i32;
    let (allowed, _) = reopened_farming(&mut session);
    assert!(!allowed.contains(&averaging_kind) && !allowed.contains(&loss_kind));
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(5_000_000, 0)) },
            )],
        );
        send(&mut session, request)
    });
    let (allowed, _) = reopened_farming(&mut session);
    assert!(allowed.contains(&averaging_kind) && !allowed.contains(&loss_kind));
}

#[test]
fn pre_2018_farm_property_uses_150_percent_and_is_not_refused() {
    // F9: a 2016 seven-year tractor on a farm is written with 150% declining
    // balance, so the "Farm depreciation" refusal cannot arise from the page;
    // a stored Regular method on such an asset keeps the page closed.
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut farm = cash_farm(5_000_000, 0);
    farm.assets = vec![Asset {
        farm_rental_use: None,
        allowed_farm_rental_use: alloc::vec::Vec::new(),
        macrs_source: None,
        id: 0,
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: Vec::new(),
        cost: 4_000_000,
        credit_reduction: 0,
        placed: 20160401,
        recovery: crate::AssetRecoveryPage::Gds {
            facts: crate::GdsAssetRecovery {
                macrs_question_context: None,
                source: None,
                class: AssetClass::Seven,
                history: AssetHistory::Continuing {
                    prior_depreciation: 3_000_000,
                    first_year_179: 0,
                    first_year_bonus: 0,
                    carryforward: 0,
                    mid_quarter: false,
                },
            },
        },
        business_percent: 100,
        acquired: None,
        disposal: AssetDisposal::No,
        investment_credit: crate::InvestmentCredit::No,
        earlier_business_percent: None,
        partial: None,
    }];
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
        );
        send(&mut session, request)
    });
    let (data, _) = stored_farm(&session.draft);
    assert_eq!(data.assets[0].method, EquipmentMethod::Declining150);
    let r = estimate(&session.draft);
    assert!(!r.issues.iter().any(|i| i.section == "Farm depreciation"), "{r:?}");
    let mut d = session.draft.clone();
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.assets[0].method = EquipmentMethod::Regular;
    let mut stale = Interview::new(TEST_USER, d);
    let (cookie, revision) = (stale.draft.cookie.clone(), stale.draft.revision);
    let reply = send(
        &mut stale,
        P::Navigate {
            user: TEST_USER,
            cookie,
            revision,
            from: PAGE_INCOME,
            section: (TaxSection::Farming).into(),
            navigation: NavigationAvailability::default(),
        },
    );
    assert!(matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

#[test]
fn farm_owned_vehicle_uses_selected_costs_and_survives_native_save_and_restart() {
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut farm = cash_farm(2_000_000, 100_000);
    farm.vehicles = vec![owned_actual_vehicle(FirstYearDeduction::Section179 {
        description: "Farm delivery car".into(),
        elected: 200_000,
    })];
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(farm.clone()) },
            )],
        );
        send(&mut session, request)
    });
    let entry = &session.draft.income[0];
    let source = entry.id;
    assert_eq!(entry.owner, crate::DocumentOwner::Yours);
    let (data, _) = stored_farm(&session.draft);
    bind_saved_property_ids(&mut farm.assets, &mut farm.vehicles, &data.assets);
    // $10,000 x 75% business use: $2,000 section179 and $5,500 bonus.
    // Operating $3,000 + interest $1,500 + feed $1,000 + depreciation
    // $7,500 = $13,000 expenses; $20,000 receipts leave $7,000 profit.
    let result = supported(&session.draft);
    let rows = crate::draft_return(&result);
    for (line, amount) in
        [("10", 300_000), ("21b", 150_000), ("14", 750_000), ("33", 1_300_000), ("34", 700_000)]
    {
        assert!(
            rows.iter().any(|row| row
                .form_line
                .as_ref()
                .is_some_and(|form| form.form == TaxForm::ScheduleF && form.line == line)
                && row.amount == amount),
            "{line}: {rows:?}"
        );
    }
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let sources = section179_direct_income_sources(&session.draft, system).unwrap();
        let native = sources.iter().find(|value| value.source == source).unwrap();
        // Pre179 costs: $1,000 + $3,000 + $1,500 + $5,500 = $11,000,
        // leaving $9,000 before the single $2,000 section179 deduction.
        // Both systems allow 100% bonus.
        assert_eq!(native.operating_net_cents, 900_000);
        let selected = crate::section179_return::prepare(&session.draft, system).unwrap();
        assert_eq!(selected.allocation.allowed(source), Ok(200_000));
        assert_eq!(selected.allocation.expenses(source), Ok(1_300_000));
    }
    let (_, reopened) = reopened_farming(&mut session);
    assert_eq!(reopened, vec![IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }]);
    let saved = session.draft.clone();
    let P::SaveIncomes { documents, .. } = open_topic(&mut session, TaxSection::Farming) else {
        unreachable!()
    };
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            documents.documents.into_iter().map(crate::IncomeRecordItem::into_income).collect(),
        );
        send(&mut session, request)
    });
    assert_eq!(session.draft.revision, saved.revision);
    let mut cold = restart(session.draft);
    assert_eq!(reopened_farming(&mut cold).1, reopened);
    assert_eq!(supported(&cold.draft).tax, result.tax);
}

#[test]
fn farm_income_source_offers_no_services_choice_and_both_work_values_round_trip() {
    // T2-src: a farm's choices are Not needed, Capital and Both (capital is
    // always material for a farm). A Form 8615 return's allowed indices are
    // [1, 2] (T2-cy); Both carries the 30% answer or an amount.
    let original = form_8615_return(farm_only_return());
    for (choice, stored) in [
        (
            FarmIncomeSource::Mixed { value: FarmWorkValue::AtLeastThirtyPercent },
            BusinessIncomeSource::MixedThirtyPercent,
        ),
        (
            FarmIncomeSource::Mixed { value: FarmWorkValue::Amount { compensation: 400_000 } },
            BusinessIncomeSource::Mixed { compensation: 400_000 },
        ),
        (FarmIncomeSource::Capital, BusinessIncomeSource::Capital),
    ] {
        let mut session = Interview::new(TEST_USER, original.clone());
        let farm = with_more(cash_farm(1_000_000, 0), |more| {
            more.income_source = choice.clone();
        });
        accepted(&{
            let request = save_topic(
                &mut session,
                TaxSection::Farming,
                vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
            );
            send(&mut session, request)
        });
        assert_eq!(stored_farm(&session.draft).0.income_source, stored);
        let (_, reopened) = reopened_farming(&mut session);
        let IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } = &reopened[0] else {
            unreachable!()
        };
        assert_eq!(farm.more.income_source, choice);
        // Saving the reopened page stores no new revision.
        let revision = session.draft.revision;
        let farm_id = session.draft.income[0].id;
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                farm_id,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(farm.clone()) },
            )],
        );
        accepted(&send(&mut session, request));
        assert_eq!(session.draft.revision, revision);
    }
    // A stored Services source (what Not needed writes) reopens as Not
    // needed, for a young filer too, who must then choose on save.
    let adult = farm_only_return();
    let mut session = Interview::new(TEST_USER, adult.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Farm(with_more(cash_farm(1_000_000, 0), |more| {
                        more.optional_se = true;
                    })),
                },
            )],
        );
        send(&mut session, request)
    });
    let mut young = session.draft.clone();
    young.people.as_mut().unwrap().taxpayer.birth_date = 20080101;
    let FederalIncome::Business { data } = &mut young.income[0].income else { unreachable!() };
    data.income_source = BusinessIncomeSource::Services;
    let mut session = restart(young);
    let (_, reopened) = reopened_farming(&mut session);
    let IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } = &reopened[0] else {
        unreachable!()
    };
    assert_eq!(farm.more.income_source, FarmIncomeSource::NotNeeded);
}

#[test]
fn farming_topic_reaches_the_schedule_f_cooperative_fixture_totals() {
    // The hand fixture of tests/farm.rs
    // farm_cooperative_distributions_reach_schedule_f_line_3_se_qbi_and_the_return,
    // entered through the Farming page: a single filer's farm with line 2
    // raised sales $40,000 and $10,000 of expenses (line 16 feed), and a
    // 1099-PATR: box 1 $16,000, box 2 $1,000, box 3 $3,000, box 5 $2,000,
    // box 4 $300, box 7 $8,000, $2,000 of boxes 1-3 from personal purchases.
    // By hand (2025 Schedule F instructions, unchanged 2026 lines):
    //   3a = 16,000 + 1,000 + 3,000 + 2,000 = 22,000; 3b = 22,000 - 2,000 = 20,000
    //   line 9 = 40,000 + 20,000 = 60,000; line 33 = 10,000; line 34 = 50,000
    //   Schedule SE: 50,000 x 92.35% = 46,175; x 15.3% = 7,064.78 -> 7,065;
    //   half 3,533; AGI = 50,000 - 3,533 = 46,467
    //   QBI: 20% x 46,467 = 9,293.40; taxable income before QBI 46,467 -
    //   16,100 = 30,367, 20% = 6,073.40; deduction 6,073
    //   taxable income 24,294; tax 1,240 + 12% x (24,275 - 12,400) = 2,665;
    //   total tax 2,665 + 7,065 = 9,730; box 4 $300 on line 25b.
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let patr = Patr1099 {
        payer: Default::default(),
        recipient: Default::default(),
        patronage: 1_600_000,
        nonpatronage: 100_000,
        per_unit_retains: 300_000,
        withholding: 30_000,
        redeemed: 200_000,
        more: PatrMore {
            section_199a_g: 0,
            qualified_payments: 800_000,
            qualified_items: 0,
            sstb_items: 0,

            ..Default::default()
        },
    };
    let mut patr = patr;
    patr_parts(&mut patr, 200_000);
    let farm = cash_farm(4_000_000, 1_000_000);
    // The farm is on the return first: the Farming page refuses the PATR without one.
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(farm.clone()) },
            )],
        );
        send(&mut session, request)
    });
    accepted(&{
        let request = save_documents(
            &mut session,
            vec![document(0, IncomeSource::Farming(cooperative_document(patr.clone())))],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            request,
        )
    });
    let r = supported(&session.draft);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 46467);
    // Box 8 is empty: none of the 20,000 is QBI (QBI 26,467, 20% = 5,293).
    // Taxable income 46,467 - 16,100 - 5,293 = 25,074; tax 2,761 + SE 7,065.
    assert_eq!(value(&r, "qbi_deduction"), 5293);
    assert_eq!(return_line(&r, "25b"), 30_000);
    assert_eq!(r.tax, Some(982600));
    let lines: Vec<(alloc::string::String, i64)> = crate::draft_return(&r)
        .iter()
        .filter_map(|row| {
            row.form_line
                .as_ref()
                .filter(|f| f.form == TaxForm::ScheduleF)
                .map(|f| (f.line.clone(), row.amount))
        })
        .collect();
    for (line, amount) in [
        ("2", 4_000_000),
        ("3a", 2_200_000),
        ("3b", 2_000_000),
        ("9", 6_000_000),
        ("33", 1_000_000),
        ("34", 5_000_000),
    ] {
        assert!(lines.contains(&(line.into(), amount)), "{line}: {lines:?}");
    }
    // The hub row counts the kinds and totals line 34; the reopened farm
    // row previews it.
    let Reply::Response(hub @ P::Hub { .. }) = go_to_from_review(&mut session, TaxSection::Income)
    else {
        panic!("expected the income hub")
    };
    assert!(
        hub_rows(&hub).contains(&row_of(
            "Farming",
            "1 farm, 1 1099-PATR · $50000.00",
            TaxSection::Farming
        )),
        "{:?}",
        hub_rows(&hub)
    );
    let P::SaveIncomes { documents, .. } = open_topic(&mut session, TaxSection::Farming) else {
        unreachable!()
    };
    let farm_row = documents
        .documents
        .iter()
        .find(|i| matches!(i.document, crate::IncomeRecord::Farm(_)))
        .unwrap();
    assert_eq!(farm_row.preview, Some(5_000_000));
}

#[test]
fn farm_ccc_and_crop_insurance_lines_are_tagged() {
    // F14, by hand from the Schedule F line definitions: CCC loans under
    // the section 77 election, $1,000 still pledged and $2,000 forfeited in
    // 2026: line 5a = 3,000, 5b = 2,000. Crop insurance: $7,000 physical
    // loss and $1,000 price-only received in 2026 (6a = 8,000), $500
    // deferred from 2025 (6d); deferring the 2026 physical loss leaves 6b =
    // 8,000 - 7,000 = 1,000, including all of it leaves 6b = 8,000.
    let original = farm_only_return();
    let lines_for = |election: CropPaperTiming| {
        let mut farm = with_more(cash_farm(5_000_000, 0), |more| {
            more.ccc = CccLoans::Loans {
                method: CccMethod::Income,
                loans: vec![
                    CccLoan::New { id: 0, principal: 100_000, outcome: CccOutcome::Outstanding },
                    CccLoan::New {
                        id: 0,
                        principal: 200_000,
                        outcome: CccOutcome::Forfeited { surplus: 0, deficiency: 0 },
                    },
                ],
                withholding: 0,
                source: ccc_source_input(&CccMethod::Income),
            };
        });
        farm.insurance = CropInsurance::Payments {
            physical: vec![CropLoss { crop: "Grain".into(), amount: 700_000, year: 2026 }],
            price_only: 100_000,
            from_prior_year: 50_000,
            withholding: 0,
        };
        let mut session = Interview::new(TEST_USER, original.clone());
        accepted(&{
            let request = save_topic(
                &mut session,
                TaxSection::Farming,
                vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
            );
            send(&mut session, request)
        });
        accepted(&answer_crop_timing(&mut session, election.clone()));
        if matches!(election, CropPaperTiming::Defer) {
            accepted(&answer_crop_practice(&mut session, "Grain", 750_000));
        }
        let r = supported(&session.draft);
        crate::draft_return(&r)
            .iter()
            .filter_map(|row| {
                row.form_line
                    .as_ref()
                    .filter(|f| f.form == TaxForm::ScheduleF)
                    .map(|f| (f.line.clone(), row.amount))
            })
            .collect::<Vec<(alloc::string::String, i64)>>()
    };
    let included = lines_for(CropPaperTiming::Include);
    for (line, amount) in
        [("5a", 300_000), ("5b", 200_000), ("6a", 800_000), ("6b", 800_000), ("6d", 50_000)]
    {
        assert!(included.contains(&(line.into(), amount)), "{line}: {included:?}");
    }
    let deferred = lines_for(CropPaperTiming::Defer);
    assert!(deferred.contains(&("6b".into(), 100_000)), "{deferred:?}");
}

#[test]
fn farm_left_on_nothing_follows_the_allowed_set() {
    // Not read (an adult): "More: Nothing" stores Not needed's record
    // (Services, which a farm offers no choice for) and reopens unchanged.
    let adult = farm_only_return();
    let mut session = Interview::new(TEST_USER, adult.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(1_000_000, 0)) },
            )],
        );
        send(&mut session, request)
    });
    assert_eq!(stored_farm(&session.draft).0.income_source, BusinessIncomeSource::Services);
    assert_eq!(
        reopened_farming(&mut session).1,
        vec![IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(1_000_000, 0)) }]
    );
    // Read (Form 8615 applies): Nothing is refused by name, since the
    // question must be answered under Something else.
    let young = form_8615_return(farm_only_return());
    let mut session = Interview::new(TEST_USER, young.clone());
    let before = session.draft.clone();
    let message = problem_text({
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(1_000_000, 0)) },
            )],
        );
        current_income_refusal(
            "income-source-1612",
            &mut session,
            request,
            "SOURCE_FARM_NEEDED",
            "[documents/documents/0/document/more/income_source]",
        )
    });
    assert!(message.contains("Form 8615 line 1") && message.contains("farm"), "{message}");
    assert_eq!(session.draft, before);
    // A farm saved while the source was not read, on a return that later
    // takes Form 8615: the unasked source is not known, so it is taken as
    // capital with a warning instead of being treated as services.
    let mut d = adult_farm_draft();
    let people = d.people.as_mut().unwrap();
    people.taxpayer.birth_date = 20100101;
    people.taxpayer.parent_alive = true;
    let r = estimate(&d);
    assert!(noted(&r, "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL"), "{r:?}");
    assert!(!r.issues.iter().any(|i| i.section.starts_with("Income: ")), "{:?}", r.issues);
    // The same farm answered Capital needs no assumption.
    let mut session = Interview::new(TEST_USER, young.clone());
    let farm = with_more(cash_farm(1_000_000, 0), |more| {
        more.income_source = FarmIncomeSource::Capital;
    });
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
        );
        send(&mut session, request)
    });
    assert!(!noted(&estimate(&session.draft), "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL"));
}
fn adult_farm_draft() -> FederalDraft {
    let adult = farm_only_return();
    let mut session = Interview::new(TEST_USER, adult.clone());
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(cash_farm(1_000_000, 0)) },
            )],
        );
        send(&mut session, request)
    });
    session.draft
}

// T2-cy item 2 (T2-49b): a qualifying farm's Form 2210 page offers
// "Yes, for 2026" alone, and the page it opens holds no farmer value
// outside that set, whichever penalty branch was stored.
#[test]
fn qualifying_farm_payments_page_holds_no_farmer_value_outside_its_set() {
    // $120,000 farm gross income of $180,000 total: two-thirds exactly.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(120_000, 120_000)));
    let (mut s, page) = at_the_payments_page(d);
    let farmer_set = payments_value(&page).2.to_vec();
    assert_eq!(farmer_set, vec![2]);
    // Stored with the IRS figuring the penalty: no farmer value at all, so
    // choosing Figure leaves it to the singleton set.
    assert_eq!(payments_value(&page).0.more.penalty, PenaltyChoice::IrsFigures);
    // Stored with Figure: the reopened farmer answer is inside the set.
    let figure = Payments {
        more: crate::PaymentsMore {
            penalty: figure_with(
                PriorReturn::SameReturn,
                WithholdingTiming::Evenly,
                Farmer::ThisReturn { paid_by_march_1: false },
                Waiver::None,
            ),
            next_year: NextYearEstimate::NotNeeded,
            ..no_payments().more
        },
        ..no_payments()
    };
    accepted(&submit(&mut s, figure));
    let page = open_payments(&mut s);
    let PenaltyChoice::Figure { farmer, .. } = &payments_value(&page).0.more.penalty else {
        panic!("expected the Figure branch")
    };
    let index = match farmer {
        Farmer::No => 0,
        Farmer::Yes { .. } => 1,
        Farmer::ThisReturn { .. } => 2,
    };
    assert!(farmer_set.contains(&index), "{farmer:?}");
}

#[test]
fn farm_header_uses_official_codes_and_preserves_filing_answers_without_changing_income() {
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let P::SaveIncomes { farm_codes, .. } = open_topic(&mut session, TaxSection::Farming) else {
        unreachable!()
    };
    // Schedule F 2026 draft Part IV: 5 crop, 8 animal and 4 forestry codes.
    assert_eq!(
        farm_codes.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![
            111100, 111210, 111300, 111400, 111900, 112111, 112112, 112120, 112210, 112300, 112400,
            112510, 112900, 113000, 113110, 113210, 113310
        ]
    );
    let mut farm = cash_farm(8_000_000, 2_000_000);
    farm.more.header = FarmHeaderInput {
        code: 112120,
        ein: Some(12345678),
        information_returns: FarmInformationReturns::NotAllFiled,
    };
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Farm(farm.clone()) },
            )],
        );
        send(&mut session, request)
    });
    assert_eq!(stored_farm(&session.draft).1.header, Some(FarmHeader::from(&farm.more.header)));
    let result = supported(&session.draft);
    assert!(noted(&result, "NOTE_FARM_INFORMATION_INCOMPLETE"));
    let header =
        result.review_notes.iter().find(|note| note.display.key == "NOTE_FARM_HEADER").unwrap();
    assert!(header.display.english().contains("012345678"));
    let reopened = reopened_farming(&mut session).1;
    assert_eq!(
        reopened,
        vec![IncomeSource::Record { value: crate::IncomeRecord::Farm(farm.clone()) }]
    );
    let mut other = session.draft.clone();
    let FederalIncome::Business { data } = &mut other.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.header.as_mut().unwrap().information_returns = FarmInformationReturns::FiledOrWillFile;
    assert_eq!(supported(&other).tax, result.tax);
    assert_eq!(supported(&restart(session.draft).draft).tax, result.tax);
}

#[test]
fn farm_header_server_rejects_nonstatutory_code_even_with_forged_catalog() {
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut farm = cash_farm(8_000_000, 2_000_000);
    farm.more.header.code = 123456;
    let mut request = save_topic(
        &mut session,
        TaxSection::Farming,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
    );
    let P::SaveIncomes { farm_codes, .. } = &mut request else { unreachable!() };
    *farm_codes =
        vec![FarmCodeChoice { id: 123456, label: crate::common::test_label("Forged choice") }];
    rejected_with(send(&mut session, request), "FARM_HEADER_CODE_INVALID");
    assert_eq!(session.draft, original);
}

#[test]
fn farm_heavy_vehicle_current179_reaches_native_forms_and_survives_restart() {
    use crate::tests::support::builders::owned_section179_vehicle;
    use crate::tests::support::client::save_reopen_native_sources;
    let original = farm_only_return();
    let mut session = Interview::new(TEST_USER, original.clone());
    let mut farm = cash_farm(6_000_000, 100_000);
    let mut vehicle =
        owned_section179_vehicle(VehicleKind::HeavyTruckVan, 1_000_000, 200_000, 20160928);
    let VehicleMethod::Actual { business_miles, operating_costs, loan_interest, .. } =
        &mut vehicle.method
    else {
        unreachable!()
    };
    *business_miles = 7_500;
    *operating_costs = 400_000;
    *loan_interest = 200_000;
    farm.vehicles.push(vehicle);
    accepted(&{
        let request = save_topic(
            &mut session,
            TaxSection::Farming,
            vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
        );
        send(&mut session, request)
    });
    let source = session.draft.income[0].id;
    let asset = stored_farm(&session.draft).0.assets[0].id;
    // Pub. 946: bought in 2016, stored unused, and first placed in service
    // in 2026; never previously placed in personal or business use. No bonus.
    // $7,500 business basis less $2,000 elected leaves $5,500:
    // first-year half-year 200% GDS is $1,100; AMT 150% is $825.
    for (system, operating) in
        [(Section179IncomeSystem::Regular, 660_000), (Section179IncomeSystem::Alternative, 632_500)]
    {
        let sources = section179_direct_income_sources(&session.draft, system).unwrap();
        assert_eq!(sources[0].operating_net_cents, 6_000_000 - operating);
        let selected = crate::section179_return::prepare(&session.draft, system).unwrap();
        assert_eq!(selected.allocation.allowed(source), Ok(200_000));
    }
    let expected = supported(&session.draft);
    crate::tests::support::assert::assert_displayed_text_line(
        &expected,
        &alloc::format!("business_{source}_asset_0_listed_type"),
        "Heavy truck or van (listed property)",
    );
    assert_eq!(value(&expected, "self_employment_tax"), 7_263);
    assert_eq!(value(&expected, "qbi_deduction"), 6_334);
    let Reply::Response(P::Review { result, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("native review")
    };
    assert_eq!(result.return_lines, crate::draft_return(&expected));
    for (form, line, amount) in [
        (TaxForm::ScheduleF, "10", 300_000),
        (TaxForm::ScheduleF, "21b", 150_000),
        (TaxForm::ScheduleF, "14", 310_000),
        (TaxForm::ScheduleF, "33", 860_000),
        (TaxForm::ScheduleF, "34", 5_140_000),
    ] {
        assert!(
            result.return_lines.iter().any(|row| row
                .form_line
                .as_ref()
                .is_some_and(|f| f.form == form && f.line == line)
                && row.amount == amount),
            "{form:?}/{line}"
        );
    }
    for form in [TaxForm::Form4562, TaxForm::ScheduleSe, TaxForm::Form8995] {
        assert!(
            result
                .return_lines
                .iter()
                .any(|row| row.form_line.as_ref().is_some_and(|f| f.form == form)),
            "{form:?}"
        );
    }
    let saved = session.draft.clone();
    let cold =
        save_reopen_native_sources(saved.clone(), &[TaxSection::Farming], &[TaxSection::Farming]);
    assert_eq!(cold, saved);
    assert_eq!(cold.income[0].id, source);
    assert_eq!(stored_farm(&cold).0.assets[0].id, asset);
}

#[test]
fn farm_suv_cap_rejects_excess_atomically_and_design_exception_accepts_it() {
    use crate::tests::support::builders::owned_section179_vehicle;
    use crate::tests::support::client::save_reopen_native_sources;
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut farm = cash_farm(10_000_000, 0);
    farm.vehicles.push(owned_section179_vehicle(
        VehicleKind::HeavySuv { exception: SuvException::None },
        6_000_000,
        3_200_000,
        20260310,
    ));
    let request = save_topic(
        &mut session,
        TaxSection::Farming,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
    );
    accepted(&send(&mut session, request));
    let source = session.draft.income[0].id;
    let asset = stored_farm(&session.draft).0.assets[0].id;
    let expected = supported(&session.draft);
    // Rev. Proc. 2025-32 section 4.24 and IRC 179(b)(5):
    // $32,000 elected, then 100% bonus on the remaining $28,000.
    assert_eq!(value(&expected, &alloc::format!("business_{source}_asset_0_section179")), 32_000);
    assert_eq!(value(&expected, &alloc::format!("business_{source}_asset_0_bonus")), 28_000);
    let mut request = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let crate::IncomeRecord::Farm(farm) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let VehicleMethod::Actual { history, .. } = &mut farm.vehicles[0].method else {
        unreachable!()
    };
    *history = AssetHistory::New {
        first_year: FirstYearDeduction::Section179 {
            description: "Synthetic farm SUV".into(),
            elected: 3_200_001,
        },
    };
    let before = session.draft.clone();
    let rejection = send(&mut session, request.clone());
    let Reply::Response(P::Problem { field, .. }) = &rejection else { panic!("SUV cap refusal") };
    assert_eq!(
        field.as_deref(),
        Some(
            "[documents/documents/0/document/Farm/vehicles/0/method/Actual/history/New/first_year/Section179/elected]"
        )
    );
    rejected_with(rejection, "ASSET_179_ABOVE_SUV_LIMIT");
    assert_eq!(session.draft, before);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let crate::IncomeRecord::Farm(farm) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let VehicleMethod::Actual { kind, history, .. } = &mut farm.vehicles[0].method else {
        unreachable!()
    };
    *kind = VehicleKind::HeavySuv { exception: SuvException::SixFootCargo };
    *history = AssetHistory::New {
        first_year: FirstYearDeduction::Section179 {
            description: "Synthetic six-foot farm cargo vehicle".into(),
            elected: 3_300_000,
        },
    };
    accepted(&send(&mut session, request));
    let expected = supported(&session.draft);
    assert_eq!(value(&expected, &alloc::format!("business_{source}_asset_0_section179")), 33_000);
    assert_eq!(value(&expected, &alloc::format!("business_{source}_asset_0_bonus")), 27_000);
    let saved = session.draft.clone();
    let cold =
        save_reopen_native_sources(saved.clone(), &[TaxSection::Farming], &[TaxSection::Farming]);
    assert_eq!(cold, saved);
    assert_eq!(stored_farm(&cold).0.assets[0].id, asset);
}

#[test]
fn farm_and_schedule_c_share_income_limit_with_independent_regular_and_amt() {
    use crate::tests::support::builders::owned_section179_vehicle;
    use crate::tests::support::client::save_reopen_native_sources;
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let mut vehicle =
        owned_section179_vehicle(VehicleKind::HeavyTruckVan, 1_000_000, 500_000, 20160928);
    let VehicleMethod::Actual { business_miles, operating_costs, loan_interest, .. } =
        &mut vehicle.method
    else {
        unreachable!()
    };
    *business_miles = 7_500;
    *operating_costs = 400_000;
    *loan_interest = 200_000;
    let mut farm = cash_farm(600_000, 100_000);
    farm.vehicles.push(vehicle.clone());
    let request = save_topic(
        &mut session,
        TaxSection::Farming,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
    );
    accepted(&send(&mut session, request));
    let Reply::Response(P::Review { result, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("native review")
    };
    for (line, amount) in [("11", 0), ("13", 500_000)] {
        let row = result
            .return_lines
            .iter()
            .find(|row| {
                row.form_line
                    .as_ref()
                    .is_some_and(|f| f.form == TaxForm::Form4562 && f.line == line)
            })
            .expect("section 179 form row");
        assert_eq!(row.money, Some(amount));
        assert!(!row.label.is_empty());
    }
    let mut page = business(600_000);
    page.vehicles.push(vehicle);
    accepted(&save_businesses(&mut session, vec![page]));
    // Both vehicles were bought in 2016 and stored unused until their first
    // personal or business use in 2026 (no personal-use conversion).
    // Form4562 line11 aggregates active C/F income. Remaining $2,500 basis
    // costs $500 regular/$375 AMT per vehicle, rather than copying REG.
    for (system, income_limit, allowed) in [
        (Section179IncomeSystem::Regular, 100_000, 50_000),
        (Section179IncomeSystem::Alternative, 125_000, 62_500),
    ] {
        let selected = crate::section179_return::prepare(&session.draft, system).unwrap();
        assert_eq!(selected.allocation.income.business_income_limit_cents, income_limit);
        assert_eq!(selected.allocation.income.total_allowed_cents, income_limit);
        for entry in &session.draft.income {
            assert_eq!(selected.allocation.allowed(entry.id), Ok(allowed));
        }
    }
    let expected = supported(&session.draft);
    for (id, amount) in
        [("section179_business_income_limit", 100_000), ("section179_carryforward", 900_000)]
    {
        assert_eq!(crate::tests::support::assert::displayed_line_amount(&expected, id), amount);
    }
    let mut request = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let crate::IncomeRecord::Farm(farm) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let FarmMethod::Cash { raised, .. } = &mut farm.accounting else { unreachable!() };
    *raised = 650_000;
    accepted(&send(&mut session, request));
    let edited = supported(&session.draft);
    for (id, amount) in
        [("section179_business_income_limit", 150_000), ("section179_carryforward", 850_000)]
    {
        assert_eq!(crate::tests::support::assert::displayed_line_amount(&edited, id), amount);
    }
    let saved = session.draft.clone();
    assert_eq!(
        save_reopen_native_sources(
            saved.clone(),
            &[TaxSection::Farming, TaxSection::SelfEmployment],
            &[TaxSection::Farming, TaxSection::SelfEmployment]
        ),
        saved
    );
}

#[test]
fn farm_and_schedule_c_share_one_return_wide_dollar_limit() {
    use crate::tests::support::builders::owned_section179_vehicle;
    use crate::tests::support::client::save_reopen_native_sources;
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let vehicle =
        owned_section179_vehicle(VehicleKind::HeavyTruckVan, 150_000_000, 128_000_000, 20260310);
    let mut farm = cash_farm(200_000_000, 0);
    farm.vehicles.push(vehicle.clone());
    let request = save_topic(
        &mut session,
        TaxSection::Farming,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
    );
    accepted(&send(&mut session, request));
    let mut page = business(200_000_000);
    page.vehicles.push(vehicle);
    accepted(&save_businesses(&mut session, vec![page]));
    // Rev. Proc. 2025-32 section4.24: total cost $3m stays below the
    // $4.09m phaseout threshold. ONE $2.56m deduction applies to the return.
    for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
        let selected = crate::section179_return::prepare(&session.draft, system).unwrap();
        assert_eq!(selected.allocation.income.total_allowed_cents, 256_000_000);
        for entry in &session.draft.income {
            assert_eq!(selected.allocation.allowed(entry.id), Ok(128_000_000));
        }
    }
    let expected = supported(&session.draft);
    assert_eq!(value(&expected, "section179_deduction"), 2_560_000);
    assert_eq!(value(&expected, "section179_carryforward"), 0);
    let saved = session.draft.clone();
    assert_eq!(
        save_reopen_native_sources(
            saved.clone(),
            &[TaxSection::Farming, TaxSection::SelfEmployment],
            &[TaxSection::Farming, TaxSection::SelfEmployment]
        ),
        saved
    );
    // Line 9 cannot exceed line 5; dollar-cap excess is not a line 13 carry.
    let mut request = open_topic(&mut session, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let crate::IncomeRecord::Business(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    let VehicleMethod::Actual { history, .. } = &mut page.vehicles[0].method else {
        unreachable!()
    };
    *history = AssetHistory::New {
        first_year: FirstYearDeduction::Section179 {
            description: "Synthetic aggregate over-election".into(),
            elected: 128_000_001,
        },
    };
    accepted(&send(&mut session, request));
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.issues.iter().any(|issue| issue.display.key
        == "ISSUE_THE_ASSET_ELECTIONS_AND_SCHEDULE_K_1_SECTION_179_AMOUNTS_EXCEED_THE_CE43F95A"));
    assert!(result.tax.is_none() && result.lines.is_empty());
}

#[test]
fn farm_relations_store_only_source_keys_and_round_trip_with_the_activity() {
    let mut page = cash_farm(100_000, 0);
    page.related.forms.push(crate::RelatedFormView { key: 71, summary: None });
    let stored = crate::farm_page::farm_record(&page, None);
    let BusinessActivity::Farming { data: farm } = &stored.activity else { unreachable!() };
    assert_eq!(farm.related, vec![71]);
    let reopened = crate::farm_page::farm_item("", &stored, false).unwrap();
    assert_eq!(reopened.related.forms[0].key, 71);
    assert!(reopened.related.candidates.is_empty());
    assert!(reopened.related.forms[0].summary.is_none());

    let mut rental = crate::tests::farm::farm_rental::rental_page(1_000);
    rental.related.forms.push(crate::RelatedFormView { key: 72, summary: None });
    let stored = crate::farm_page::farm_rental_record(&rental);
    assert_eq!(stored.farm.related, vec![72]);
    let reopened = crate::farm_page::farm_rental_item("", &stored).unwrap();
    assert_eq!(reopened.related.forms[0].key, 72);
}

#[test]
fn farm_related_native_editors_save_reopen_and_keep_their_field_counts() {
    extern crate std;
    use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
    let mut draft = complete(FederalStatus::Single);
    draft.income.clear();
    add(&mut draft, income(raised_farm(10_000, 0)));
    let farm = draft.income.last().unwrap().id;
    add(&mut draft, income(FederalIncome::FarmRental { data: farm_rental_data(8_000, 0) }));
    let rental = draft.income.last().unwrap().id;
    let mut keys = Vec::new();
    for amount in [40_000, 60_000] {
        let mut document = government_unemployment(0, &[], 0);
        let FederalIncome::GovernmentPayment { data } = &mut document.income else {
            unreachable!()
        };
        data.form.agriculture_payments = Some(amount);
        add(&mut draft, document);
        keys.push(farm_government_component(&draft));
    }
    let mut session = restart(draft);
    let links = [(farm, keys[0]), (rental, keys[1])];
    for (_, activity, key, prefix) in [
        ("farm", farm, keys[0], "TAX_RF4_FARM"),
        ("farm-rental", rental, keys[1], "TAX_RF4_FARM_RENTAL"),
    ] {
        let native = NativeInterviewCase(prefix);
        let before = native.editor(&mut session, TaxSection::Farming, "BEFORE");
        native.write_file("CAPTURE_DIR", "before.avro", &before);
        let mut rows = crate::tests::support::client::income_pages::page_documents(&before);
        // The Farming page refuses a save while a paper has no receiving farm, so
        // every farm of the page links its paper in the first save.
        for (id, paper) in links {
            let row = rows.iter_mut().find(|row| row.id == id).unwrap();
            let related = match &mut row.document {
                IncomeSource::Record { value: crate::IncomeRecord::Farm(value) } => {
                    &mut value.related
                }
                IncomeSource::Record { value: crate::IncomeRecord::FarmRental(value) } => {
                    &mut value.related
                }
                _ => unreachable!(),
            };
            if id == activity {
                assert!(related.candidates.iter().any(|candidate| candidate.id == key));
            }
            if !related.forms.iter().any(|form| form.key == paper) {
                related.forms.push(crate::RelatedFormView { key: paper, summary: None });
            }
        }
        let answers = save_topic(&mut session, TaxSection::Farming, rows);
        native.save(&mut session, answers);
        native.restart(&mut session);
        let reopened = native.editor(&mut session, TaxSection::Farming, "REOPENED");
        native.write_file("CAPTURE_DIR", "reopened.avro", &reopened);
        let source = crate::farm::sources(&session.draft)
            .find(|source| source.entry.id == activity)
            .unwrap();
        assert_eq!(source.farm.related, vec![key]);
        assert_eq!(
            crate::related_forms::effective_claimants(&session.draft).get(&key),
            Some(&activity)
        );
    }
}

#[test]
fn conservation_livestock_native_purpose_correction_reopens_with_the_same_property() {
    let draft = conservation_livestock_return(85_000, 15_000, None);
    let source = draft.income[1].id;
    let mut session = Interview::new(TEST_USER, draft);
    let (_, sources) = reopened_farming(&mut session);
    let IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } = &sources[0] else {
        unreachable!()
    };
    let property = farm.assets[0].id;
    for purpose in [Some(LivestockPurpose::Breeding), Some(LivestockPurpose::Sport), None] {
        // Submit the complete offered page, including the retained return-wide
        // section1231 history; changing purpose does not delete that answer.
        let mut request = open_topic(&mut session, TaxSection::Farming);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let item = documents.documents.iter_mut().find(|item| item.id == source).unwrap();
        let crate::IncomeRecord::Farm(farm) = &mut item.document else { unreachable!() };
        let AssetDisposal::Disposed { how: DisposalKind::Sold { livestock, .. }, .. } =
            &mut farm.assets[0].disposal
        else {
            unreachable!()
        };
        *livestock = SoldLivestock::CattleOrHorse { purpose };
        accepted(&send(&mut session, request));
        let mut cold = restart(session.draft.clone());
        let (_, reopened) = reopened_farming(&mut cold);
        let IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } = &reopened[0] else {
            unreachable!()
        };
        assert_eq!(cold.draft.income[1].id, source);
        assert_eq!(farm.assets[0].id, property);
        assert!(
            matches!(farm.assets[0].disposal, AssetDisposal::Disposed { how: DisposalKind::Sold { livestock: SoldLivestock::CattleOrHorse { purpose: actual }, .. }, .. } if actual == purpose)
        );
        let result = supported(&cold.draft);
        assert_eq!(
            value(&result, "farm_2_conservation_deduction"),
            match purpose {
                Some(LivestockPurpose::Breeding) => 25_000,
                Some(LivestockPurpose::Sport) => 21_250,
                _ => 0,
            }
        );
        if purpose.is_none() {
            let note = result
                .review_notes
                .iter()
                .find(|note| note.display.key == "WARN_FARM_CONSERVATION_GROSS_SOURCE")
                .unwrap();
            let context = note.context.as_ref().unwrap();
            assert_eq!(
                context.identity(),
                crate::federal_model::NoteIdentity::Item {
                    parent: alloc::boxed::Box::new(crate::federal_model::NoteIdentity::Income(
                        source
                    )),
                    collection: "assets",
                    position: 0,
                }
            );
            assert_eq!(
                crate::interview::issue_pages(&cold.draft, context, None),
                vec![TaxSection::Farming]
            );
        }
        session = cold;
    }
}
