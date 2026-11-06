// Source-authored controls for remaining farm/rental former-rule keys. Actual
// native execution and emitted-schema resolution belong to the parent gates.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;
use crate::tests::farm::farm_specials as sources;

fn form_total(result: &FederalResult, form: TaxForm, line: &str) -> i64 {
    result
        .lines
        .iter()
        .filter(|row| {
            row.form_line.as_ref().is_some_and(|tag| tag.form == form && tag.line == line)
        })
        .map(|row| row.amount)
        .sum()
}

fn gap_page(session: &mut Interview, topic: TaxSection, documents: Vec<IncomeSource>) -> P {
    save_topic(session, topic, documents.into_iter().map(|value| document(0, value)).collect())
}

fn farm_gap_control(
    case: &str,
    session: &mut Interview,
    positive: P,
    index: usize,
    expected: (&str, &str),
    financial: &[(TaxForm, &str, i64)],
    mutate: impl FnOnce(&mut crate::IncomeRecord),
) {
    let P::SaveIncomes { topic, .. } = &positive else { unreachable!() };
    let topic = *topic;
    accepted(&send(session, positive));
    let corrected = open_topic(session, topic);
    assert!(page_documents(&corrected)[index].id > 0);
    let baseline = estimate(&session.draft);
    for (form, line, amount) in financial {
        assert_eq!(
            form_total(&baseline, *form, line),
            *amount,
            "{case}: independent positive form control"
        );
    }
    let mut invalid = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
    mutate(&mut documents.documents[index].document);
    let path = alloc::format!("[documents/documents/{index}/document/{}]", expected.1);
    let running = credit_estimate_documents_from_save(&invalid);
    credit_refusal(&alloc::format!("farm-gap-{case}-save"), session, invalid, expected.0, &path);
    credit_refusal(
        &alloc::format!("farm-gap-{case}-estimate"),
        session,
        running,
        expected.0,
        &path,
    );
    let saved = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(session, credit_estimate_documents_from_save(&corrected)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    assert_eq!(session.draft, saved, "{case}: correction preserves actual source IDs/facts");
    assert_eq!(estimate(&session.draft), baseline);
    let reopened = open_topic(session, topic);
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, topic);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), baseline);
}

#[test]
fn unit47_farm_126_source_components_and_prior_years_have_native_save_controls() {
    for branch in 0..6 {
        // Reuse the published Pub225 example and its actual accepted parcel.
        let mut session = sources::cost_sharing_session(Farm126Election::Exclude {
            facts: sources::cost_sharing_facts(Some(sources::cost_sharing_valuation())),
        });
        let positive = open_topic(&mut session, TaxSection::Farming);
        let (case, key, field) = match branch {
            0 => ("126-cost", "FARM126_COST_REQUIRED", "total_cost"),
            1 => ("126-owner", "FARM126_OWNER_ABOVE_COST", "owner_contribution"),
            2 => ("126-government", "FARM126_OWNER_ABOVE_COST", "government_received"),
            3 => ("126-components", "FARM126_COMPONENTS_ABOVE_COST", "current_deductible"),
            4 => (
                "126-prior-range",
                "FARM126_PRIOR_INCOME_YEARS",
                "election/facts/prior_income/2/year",
            ),
            _ => (
                "126-prior-duplicate",
                "FARM126_PRIOR_INCOME_YEARS",
                "election/facts/prior_income/1/year",
            ),
        };
        farm_gap_control(
            case,
            &mut session,
            positive,
            0,
            (key, &alloc::format!("more/specials/cost_sharing/0/{field}")),
            &[(TaxForm::ScheduleF, "4a", 49_000_000), (TaxForm::ScheduleF, "4b", 345_000)],
            |document| {
                let crate::IncomeRecord::Farm(farm) = document else { unreachable!() };
                let source = &mut farm.more.specials.cost_sharing[0];
                match branch {
                    0 => source.total_cost = 0,
                    1 => source.owner_contribution = source.total_cost + 1,
                    2 => source.government_received = source.total_cost + 1,
                    3 => source.current_deductible = source.total_cost + 1,
                    _ => {
                        let Farm126Election::Exclude { facts } = &mut source.election else {
                            unreachable!()
                        };
                        if branch == 4 {
                            facts.prior_income[2].year = source.installation_year;
                        } else {
                            facts.prior_income[1].year = facts.prior_income[0].year;
                        }
                    }
                }
            },
        );
    }
}

#[test]
fn unit47_farm_263a_receipt_and_production_dates_keep_native_source_owners() {
    for branch in 0..5 {
        let mut session = Interview::new(TEST_USER, farm_only_return());
        let mut farm = cash_farm(5_000_000, 0);
        let mut cohort = sources::production_inventory();
        if branch != 4 {
            cohort.tax_treatment = FarmCapitalization::SmallBusiness {
                facts: FarmSmallBusiness {
                    first_tax_year: if branch == 3 { 2026 } else { 2024 },
                    receipts: if branch == 3 {
                        vec![FarmGrossYear { year: 2026, months: 12, gross: 100_000_000 }]
                    } else {
                        vec![
                            FarmGrossYear { year: 2024, months: 6, gross: 1_600_000_000 },
                            FarmGrossYear { year: 2025, months: 12, gross: 3_200_000_000 },
                        ]
                    },
                    tax_shelter: false,
                },
            };
        }
        farm.more.specials.production = vec![cohort];
        let positive = gap_page(
            &mut session,
            TaxSection::Farming,
            vec![IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }],
        );
        let (case, key, field) = match branch {
            0 => {
                ("263a-duplicate", "FARM263A_RECEIPT_YEARS", "tax_treatment/facts/receipts/1/year")
            }
            1 => ("263a-count", "FARM263A_RECEIPT_YEARS", "tax_treatment/facts/receipts"),
            2 => (
                "263a-prior-range",
                "FARM263A_RECEIPT_YEARS",
                "tax_treatment/facts/receipts/0/year",
            ),
            3 => (
                "263a-first-current-year",
                "FARM263A_RECEIPT_YEARS",
                "tax_treatment/facts/receipts/0/year",
            ),
            _ => ("production-before-began", "FARM_PRODUCTION_DATES", "productive"),
        };
        let total_expenses = if branch == 4 { 140_000 } else { 1_190_000 };
        farm_gap_control(
            case,
            &mut session,
            positive,
            0,
            (key, &alloc::format!("more/specials/production/0/{field}")),
            &[
                (TaxForm::ScheduleF, "26", 1_000_000),
                (TaxForm::ScheduleF, "28", 190_000),
                (TaxForm::ScheduleF, "33", total_expenses),
            ],
            |document| {
                let crate::IncomeRecord::Farm(farm) = document else { unreachable!() };
                let cohort = &mut farm.more.specials.production[0];
                assert!(cohort.id > 0);
                if branch == 4 {
                    cohort.productive = Some(20241231);
                } else {
                    let FarmCapitalization::SmallBusiness { facts } = &mut cohort.tax_treatment
                    else {
                        unreachable!()
                    };
                    match branch {
                        0 => facts.receipts[1].year = facts.receipts[0].year,
                        1 => {
                            facts.receipts.pop();
                        }
                        2 => facts.receipts[0].year = 2023,
                        _ => facts.receipts[0].year = 2025,
                    }
                }
            },
        );
    }
}

#[test]
fn unit47_farm_fertilizer_hedge_and_supply_contracts_save_at_actual_fields() {
    for branch in 0..5 {
        let mut session = Interview::new(TEST_USER, farm_only_return());
        let mut farm = cash_farm(4_000_000, 0);
        if branch < 2 {
            let mut fertilizer = sources::fertilizer_source();
            if branch == 0 {
                fertilizer.paid_or_incurred = 20250501;
                fertilizer.applied = 20250502;
            }
            fertilizer.treatment = FarmFertilizerTreatment::Capitalize {
                recovery: FarmFertilizerRecovery {
                    benefit_months: 12,
                    prior_recovered: if branch == 0 { fertilizer.cost } else { 0 },
                },
            };
            farm.more.specials.fertilizer = vec![fertilizer];
        } else if branch == 2 {
            farm.more.specials.other_income = vec![FarmOtherIncome::Hedge {
                facts: FarmHedge {
                    contract: "Corn hedge settlement".into(),
                    entered: 20260102,
                    identified: 20260102,
                    settled: 20260301,
                    settlement: -50_000,
                    business_risk: "Offsets price risk on growing corn inventory.".into(),
                    timing: FarmHedgeTiming::CurrentSingleYear,
                },
            }];
        } else {
            farm.more.supplies = PrepaidSupplies::Supplies {
                advances: vec![SupplyPurchase {
                    amount: 10_000,
                    purchase: farm_supply_purchase_source(),
                }],
                carried: vec![],
                exception: SupplyException::NotNeeded,
                poultry: PoultryRecords { current: vec![], resale: vec![], production: vec![] },
            };
        }
        let positive = gap_page(
            &mut session,
            TaxSection::Farming,
            vec![IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }],
        );
        let (case, key, field) = match branch {
            0 => (
                "fertilizer-recovered-above-cost",
                "FARM_FERTILIZER_SOURCE",
                "more/specials/fertilizer/0/treatment/recovery/prior_recovered",
            ),
            1 => (
                "fertilizer-current-prior",
                "FARM_FERTILIZER_SOURCE",
                "more/specials/fertilizer/0/treatment/recovery/prior_recovered",
            ),
            2 => (
                "hedge-before-entered",
                "FARM_HEDGE_SOURCE",
                "more/specials/other_income/0/facts/settled",
            ),
            3 => (
                "supply-quantity",
                "FARM_SUPPLY_PURCHASE_TERMS",
                "more/supplies/advances/0/purchase/quantity_price_terms",
            ),
            _ => (
                "supply-purpose",
                "FARM_SUPPLY_PURCHASE_TERMS",
                "more/supplies/advances/0/purchase/business_purpose",
            ),
        };
        let financial = if branch == 2 { vec![(TaxForm::ScheduleF, "8", -50_000)] } else { vec![] };
        farm_gap_control(case, &mut session, positive, 0, (key, field), &financial, |document| {
            let crate::IncomeRecord::Farm(farm) = document else { unreachable!() };
            if branch < 2 {
                let source = &mut farm.more.specials.fertilizer[0];
                let FarmFertilizerTreatment::Capitalize { recovery } = &mut source.treatment else {
                    unreachable!()
                };
                recovery.prior_recovered = if branch == 0 { source.cost + 1 } else { 1 };
            } else if branch == 2 {
                let FarmOtherIncome::Hedge { facts } = &mut farm.more.specials.other_income[0]
                else {
                    unreachable!()
                };
                facts.settled = 20260101;
            } else {
                let PrepaidSupplies::Supplies { advances, .. } = &mut farm.more.supplies else {
                    unreachable!()
                };
                if branch == 3 {
                    advances[0].purchase.quantity_price_terms.clear();
                } else {
                    advances[0].purchase.business_purpose.clear();
                }
            }
        });
    }
}

fn farm_parcel(acquired: u32) -> FarmLandBasis {
    FarmLandBasis {
        id: 0,
        label: "Actual retained parcel".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired,
        regular: 20_000_000,
        amt: 20_000_000,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![],
    }
}

#[test]
fn unit47_farm_land_history_keeps_acquisition_and_original_parcel_identity() {
    for branch in 0..4 {
        let mut session = Interview::new(TEST_USER, farm_only_return());
        let mut farm = cash_farm(1_000_000, 0);
        let mut parcel = farm_parcel(if branch < 2 { 20260101 } else { 20000101 });
        if branch >= 2 {
            parcel.prior_cost_sharing = vec![Farm126History {
                received: 20251231,
                excluded: 100,
                statement: "Retained section126 exclusion and basis record".into(),
            }];
        }
        farm.more.land_basis = vec![parcel];
        let positive = gap_page(
            &mut session,
            TaxSection::Farming,
            vec![IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }],
        );
        let (case, field) = match branch {
            0 => ("land-current-regular", "prior_conservation"),
            1 => ("land-current-amt", "prior_amt_conservation"),
            2 => ("land-history-before-acquired", "prior_cost_sharing/0/received"),
            _ => ("land-history-current-year", "prior_cost_sharing/0/received"),
        };
        farm_gap_control(
            case,
            &mut session,
            positive,
            0,
            ("FARM_LAND_BASIS_SOURCE", &alloc::format!("more/land_basis/0/{field}")),
            &[(TaxForm::ScheduleF, "2", 1_000_000)],
            |document| {
                let crate::IncomeRecord::Farm(farm) = document else { unreachable!() };
                let parcel = &mut farm.more.land_basis[0];
                assert!(parcel.id > 0);
                match branch {
                    0 => parcel.prior_conservation = 1,
                    1 => parcel.prior_amt_conservation = 1,
                    2 => parcel.prior_cost_sharing[0].received = 19991231,
                    _ => parcel.prior_cost_sharing[0].received = 20260101,
                }
            },
        );
    }
}

fn conservation_gap_seed(rental: bool, acquired: u32) -> (Interview, P, i64) {
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let document = if rental {
        let mut farm = crate::tests::farm::farm_rental::rental_page(32_000);
        farm.more.land_basis = vec![farm_parcel(acquired)];
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(farm) }
    } else {
        let mut farm = cash_farm(3_200_000, 0);
        farm.more.land_basis = vec![farm_parcel(acquired)];
        IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }
    };
    let page = gap_page(&mut session, TaxSection::Farming, vec![document]);
    accepted(&send(&mut session, page));
    let page = open_topic(&mut session, TaxSection::Farming);
    let parcel = match &page_documents(&page)[0].document {
        IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) } => {
            farm.more.land_basis[0].id
        }
        IncomeSource::Record { value: crate::IncomeRecord::FarmRental(farm) } => {
            farm.more.land_basis[0].id
        }
        _ => unreachable!(),
    };
    assert!(parcel > 0);
    (session, page, parcel)
}

fn conservation_on(document: &mut crate::IncomeRecord) -> &mut Conservation {
    match document {
        crate::IncomeRecord::Farm(farm) => &mut farm.more.conservation,
        crate::IncomeRecord::FarmRental(farm) => &mut farm.more.conservation,
        _ => unreachable!(),
    }
}

#[test]
fn unit47_farm_and_farm_rental_conservation_dates_use_actual_parcel_sources() {
    for rental in [false, true] {
        for branch in 0..5 {
            let (mut session, mut positive, parcel) = conservation_gap_seed(
                rental,
                if branch == 0 || branch == 4 { 20260401 } else { 20250101 },
            );
            let P::SaveIncomes { documents, .. } = &mut positive else { unreachable!() };
            let cost = ConservationCostInput {
                property: parcel,
                paid_or_incurred: Some(20260401),
                label: "NRCS terrace work on actual accepted parcel".into(),
                amount: 1_000_000,
                facts: conservation_facts(),
            };
            let conservation = if branch == 4 {
                Conservation::Capitalize {
                    method: FarmConservationCapitalMethod::NeverElected,
                    costs: vec![cost],
                }
            } else {
                Conservation::Deduct {
                    costs: if branch == 0 { vec![cost] } else { vec![] },
                    carry: if branch == 1 {
                        vec![FarmConservationCarryInput {
                            property: parcel,
                            year: 2025,
                            regular: 100,
                            amt: 200,
                        }]
                    } else {
                        vec![]
                    },
                    assessments: if branch == 2 || branch == 3 {
                        vec![ConservationAssessmentInput {
                            property: parcel,
                            label: "Retained district assessment".into(),
                            total: 700_000,
                            share: 90_000,
                            facts: conservation_facts(),
                            payments: vec![ConservationAssessmentPayment {
                                year: 2025,
                                amount: 90_000,
                                current_date: None,
                            }],
                        }]
                    } else {
                        vec![]
                    },
                    method: FarmConservationMethod::OriginalElection { year: 2023 },
                }
            };
            *conservation_on(&mut documents.documents[0].document) = conservation;
            let (key, field) = match branch {
                0 | 4 => ("FARM_CONSERVATION_BEFORE_ACQUIRED", "costs/0/paid_or_incurred"),
                1 => ("FARM_CONSERVATION_BEFORE_ACQUIRED", "carry/0/year"),
                2 => ("FARM_CONSERVATION_BEFORE_ACQUIRED", "assessments/0/payments/0/year"),
                _ => ("FARM_CONSERVATION_PAYMENT_DATE", "assessments/0/payments/0/current_date"),
            };
            let financial = if !rental && branch == 0 {
                vec![(TaxForm::ScheduleF, "12", 800_000)]
            } else {
                vec![]
            };
            farm_gap_control(
                &alloc::format!("conservation-{rental}-{branch}"),
                &mut session,
                positive,
                0,
                (key, &alloc::format!("more/conservation/{field}")),
                &financial,
                |document| match conservation_on(document) {
                    Conservation::Deduct { costs, carry, assessments, .. } => match branch {
                        0 => costs[0].paid_or_incurred = Some(20260331),
                        1 => carry[0].year = 2024,
                        2 => assessments[0].payments[0].year = 2024,
                        _ => assessments[0].payments[0].current_date = Some(20260201),
                    },
                    Conservation::Capitalize { costs, .. } => {
                        costs[0].paid_or_incurred = Some(20260331)
                    }
                    _ => unreachable!(),
                },
            );
        }
    }
}

#[test]
fn unit47_farm_rental_credit_and_rental_179_are_rejected_before_financial_mutation() {
    for farm_rental in [false, true] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let document = if farm_rental {
            let mut farm = crate::tests::farm::farm_rental::rental_page(40_000);
            farm.assets = vec![asset(12_000_000, 20260501, AssetClass::Five)];
            IncomeSource::Record { value: crate::IncomeRecord::FarmRental(farm) }
        } else {
            let mut rental = rental_page(1_200_000);
            rental.assets = crate::interview::vehicle_and_asset_records(
                &[],
                &[with_first_year(
                    asset(100_000, 20260101, AssetClass::Seven),
                    FirstYearDeduction::Section179 {
                        description: "Actual rental equipment".into(),
                        elected: 0,
                    },
                )],
            )
            .1
            .into_iter()
            .map(Into::into)
            .collect();
            IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) }
        };
        let positive = gap_page(
            &mut session,
            if farm_rental { TaxSection::Farming } else { TaxSection::Rentals },
            vec![document],
        );
        let expected = if farm_rental {
            ("FARM_RENTAL_NO_INVESTMENT_CREDIT", "assets/0/investment_credit")
        } else {
            ("RENTAL_ASSET_NO_SECTION_179", "assets/0/history/section179/elected")
        };
        farm_gap_control(
            &alloc::format!("rental-investment-{farm_rental}"),
            &mut session,
            positive,
            0,
            expected,
            &[],
            |document| {
                if farm_rental {
                    let crate::IncomeRecord::FarmRental(farm) = document else { unreachable!() };
                    farm.assets[0].investment_credit = solar_asset(20250915).investment_credit;
                } else {
                    let crate::IncomeRecord::Rental(rental) = document else { unreachable!() };
                    let EquipmentHistory::New {
                        section179: Section179::Eligible { elected }, ..
                    } = &mut rental.assets[0].macrs_mut().unwrap().history
                    else {
                        unreachable!()
                    };
                    *elected = 1;
                }
            },
        );
    }
}

fn partial_gap() -> PartialDisposition {
    PartialDisposition { date: 20260602, cost_of_part: 10_000, proceeds: 0 }
}

#[test]
fn unit47_farm_and_rental_partial_dispositions_require_a_later_service_date() {
    for branch in 0..4 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut component = with_first_year(
            asset(1_000_000, 20260601, AssetClass::ThirtyNine),
            FirstYearDeduction::Regular,
        );
        component.partial = Some(partial_gap());
        let document = if branch == 0 {
            let mut farm = cash_farm(4_000_000, 0);
            farm.assets = vec![component];
            IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }
        } else {
            let mut rental = rental_page(2_400_000);
            match branch {
                1 => {
                    rental.assets = crate::interview::vehicle_and_asset_records(&[], &[component])
                        .1
                        .into_iter()
                        .map(Into::into)
                        .collect()
                }
                2 => {
                    // RentalBuilding partials require a continuing building.
                    rental.building.in_service = 20250601;
                    rental.building.partial = Some(partial_gap());
                }
                _ => {
                    let mut improvement = fixture_improvement();
                    improvement.in_service = 20250601;
                    improvement.partial = Some(partial_gap());
                    rental.more.improvements = vec![improvement];
                }
            }
            IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) }
        };
        let positive = gap_page(
            &mut session,
            if branch == 0 { TaxSection::Farming } else { TaxSection::Rentals },
            vec![document],
        );
        let field = match branch {
            0 | 1 => "assets/0/partial/date",
            2 => "building/partial/date",
            _ => "more/improvements/0/partial/date",
        };
        farm_gap_control(
            &alloc::format!("partial-service-{branch}"),
            &mut session,
            positive,
            0,
            ("PARTIAL_DISPOSITION_BEFORE_SERVICE", field),
            &[],
            |document| match document {
                crate::IncomeRecord::Farm(farm) => {
                    farm.assets[0].partial.as_mut().unwrap().date = 20260601
                }
                crate::IncomeRecord::Rental(rental) => match branch {
                    1 => {
                        rental.assets[0].macrs_mut().unwrap().partial.as_mut().unwrap().date =
                            20260601
                    }
                    2 => {
                        rental.building.in_service = rental.building.partial.as_ref().unwrap().date
                    }
                    _ => {
                        rental.more.improvements[0].in_service =
                            rental.more.improvements[0].partial.as_ref().unwrap().date
                    }
                },
                _ => unreachable!(),
            },
        );
    }
}

fn mortgage_gap_session() -> (Interview, i64) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = crate::tests::support::client::income_pages::open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    let form = Mortgage1098 {
        lender: Default::default(),
        // The 1098 prints the household borrower, so Save associates it.
        borrower: accepted_document_names(&session.draft, Owner::Taxpayer).recipient,
        id: 0,
        interest: 600_000,
        principal: 10_000_000,
        originated: 20200601,
        refund: 0,
        insurance: 0,
        points: 0,
        more: MortgageMore {
            property: "Actual first lender for shared residence".into(),
            details: MortgageDetails::FirstLender(MortgageFirstLender {
                home: MortgageHome::Main,
                purpose: MortgageUse::Home,
                average: AverageBalance::WithinLimit,
            }),
            ..Default::default()
        },
    };
    let more = form.more.clone();
    documents.common.mortgages.documents = vec![mortgage_document(form)];
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        page,
    ));
    accepted(&crate::tests::cross::interview::save_mortgage_answers(
        &mut session,
        &[(more, crate::MortgagePointTreatment::None)],
    ));
    let id = session.draft.deductions.as_ref().unwrap().mortgage[0].id;
    assert!(id > 0);
    (session, id)
}
fn linked_rental_gap(id: i64) -> Rental {
    let mut rental = rental_page(1_200_000);
    rental.personal_use = PersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 60,
        overlap_days: 0,
        direct_expenses: 0,
    };
    rental.related = crate::tests::support::builders::related_forms::related_keys(&[id]);
    rental
}

#[test]
fn unit47_rental_mortgage_source_uniqueness_uses_real_saved_first_lender_ids() {
    for branch in 0..4 {
        let (mut session, root) = mortgage_gap_session();
        let first = linked_rental_gap(root);
        let documents = if branch == 1 {
            vec![
                IncomeSource::Record { value: crate::IncomeRecord::Rental(rental_page(600_000)) },
                IncomeSource::Record { value: crate::IncomeRecord::Rental(first) },
            ]
        } else {
            vec![IncomeSource::Record { value: crate::IncomeRecord::Rental(first) }]
        };
        let positive = gap_page(&mut session, TaxSection::Rentals, documents);
        let (key, index, field) = match branch {
            0 => ("PROBLEM_RELATED_FORM_UNAVAILABLE", 0, "Rental/related/forms/1/key"),
            // Put the new competing claim first: the shared validator locates the first claimant.
            1 => ("PROBLEM_RELATED_FORM_UNAVAILABLE", 0, "Rental/related/forms/0/key"),
            2 => ("PROBLEM_RELATED_FORM_UNAVAILABLE", 0, "Rental/related/forms/0/key"),
            _ => ("RENTAL_MORTGAGE_SINGLE_SOURCE", 0, "expenses/mortgage_interest"),
        };
        farm_gap_control(
            &alloc::format!("rental-root-{branch}"),
            &mut session,
            positive,
            index,
            (key, field),
            &[],
            |document| {
                let crate::IncomeRecord::Rental(rental) = document else { unreachable!() };
                match branch {
                    0 => rental.related.forms.push(RelatedFormView { key: root, summary: None }),
                    1 => {
                        rental.personal_use = linked_rental_gap(root).personal_use;
                        rental.related =
                            crate::tests::support::builders::related_forms::related_keys(&[root]);
                    }
                    2 => rental.personal_use = PersonalUse::No,
                    _ => rental.expenses.mortgage_interest = 1,
                }
            },
        );
    }
}

#[test]
fn unit47_royalty_kind_and_cost_percentage_unit_bounds_are_native_source_controls() {
    for branch in 0..3 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let royalty = RoyaltyProperty {
            related: crate::RelatedForms::default(),
            kind: if branch == 0 { RoyaltyKind::Intangible } else { RoyaltyKind::OilGas },
            receipts: 200_000,
            advance: 0,
            expenses: no_rental_expenses(),
            special: false,
            depletion: match branch {
                0 => RoyaltyDepletion::No,
                1 => RoyaltyDepletion::Cost {
                    adjusted_basis: 100_000,
                    recoverable_units: 10,
                    units_sold: 10,
                },
                _ => RoyaltyDepletion::Percentage {
                    adjusted_basis: 100_000,
                    recoverable_units: 10,
                    units_sold: 10,
                    carryover: 0,
                    amt_carryover: 0,
                },
            },
        };
        let positive = gap_page(
            &mut session,
            TaxSection::Rentals,
            vec![IncomeSource::Record { value: crate::IncomeRecord::Royalties(royalty) }],
        );
        let expected = if branch == 0 {
            ("ROYALTY_INTANGIBLE_NO_DEPLETION", "depletion")
        } else {
            ("ROYALTY_UNITS_SOLD_WITHIN_RECOVERABLE", "depletion/units_sold")
        };
        farm_gap_control(
            &alloc::format!("royalty-depletion-{branch}"),
            &mut session,
            positive,
            0,
            expected,
            &[(TaxForm::ScheduleE, "4", 200_000)],
            |document| {
                let crate::IncomeRecord::Royalties(royalty) = document else { unreachable!() };
                if branch == 0 {
                    royalty.depletion = RoyaltyDepletion::Cost {
                        adjusted_basis: 100_000,
                        recoverable_units: 10,
                        units_sold: 1,
                    };
                } else {
                    match &mut royalty.depletion {
                        RoyaltyDepletion::Cost { units_sold, .. }
                        | RoyaltyDepletion::Percentage { units_sold, .. } => *units_sold = 11,
                        _ => unreachable!(),
                    }
                }
            },
        );
    }
}
