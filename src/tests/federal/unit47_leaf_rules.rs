use crate::tests::support::client::survey::survey_refusal as reject_atomic;
// Native owner predicates exercised through the actual editable Save request.
// The optional full-Avro export is resolved against fresh Studio output by the
// combined Unit47 gate. No key or locator is inferred from a schema string.
use crate::tests::cross::interview::{accepted, document, open_topic};
use super::*;

fn documents(request: &mut P, rows: Vec<IncomeDocumentItem>) {
    let P::SaveIncomes { documents, .. } = request else {
        panic!("income editor required")
    };
    documents.documents = rows;
}

fn document_case(
    case: &str,
    topic: TaxSection,
    good: IncomeDocument,
    bad: IncomeDocument,
    key: &str,
    local: &str,
) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut corrected = open_topic(&mut session, topic);
    documents(&mut corrected, vec![document(0, good)]);
    let mut rejected = corrected.clone();
    documents(&mut rejected, vec![document(0, bad)]);
    let field = alloc::format!("[documents/documents/0/document/{local}]");
    reject_atomic(case, &mut session, rejected, key, &field);
    let before = session.draft.revision;
    accepted(&send(&mut session, corrected));
    assert_eq!(
        session.draft.revision,
        before + 1,
        "{case}: correction did not commit"
    );
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, topic);
    accepted(&send(&mut cold, reopened));
    assert_eq!(
        cold.draft, saved,
        "{case}: cold reopen changed accepted record"
    );
}
fn nec() -> Form1099Nec {
    Form1099Nec {
        compensation: 10_000,
        withholding: 0,
        golden_parachute: 0,
        belongs: IncomeUse::Income {
            kind: OtherIncomeKind::AlaskaDividend,
        },
        business: None,
        state_details: Default::default(),
    }
}
#[test]
fn unit47_leaf_nec_amount_relationships_save_atomically() {
    for (case, key, field, mutate) in [
        (
            "leaf-nec-parachute",
            "NEC_EXCESS_ABOVE_TOTAL_PAY",
            "golden_parachute",
            0,
        ),
        (
            "leaf-nec-withholding",
            "NEC_WITHHOLDING_ABOVE_PAY",
            "withholding",
            1,
        ),
    ] {
        let good = nec();
        let mut bad = good.clone();
        if mutate == 0 {
            bad.golden_parachute = good.compensation + 1;
        } else {
            bad.withholding = good.compensation + 1;
        }
        document_case(
            case,
            TaxSection::SelfEmployment,
            IncomeDocument::Form1099Nec(good),
            IncomeDocument::Form1099Nec(bad),
            key,
            field,
        );
    }
}
fn k1099() -> Form1099K {
    Form1099K {
        gross: 10_000,
        withholding: 0,
        parts: vec![KPart {
            amount: 10_000,
            belongs: KUse::NotIncome,
            refunded: 0,
        }],
        state_details: Default::default(),
    }
}
#[test]
fn unit47_leaf_k_parts_and_refunds_save_atomically() {
    for (case, key, field, mutation) in [
        ("leaf-k-parts", "K_PARTS_NOT_GROSS", "parts", 0),
        (
            "leaf-k-withholding",
            "K_WITHHOLDING_ABOVE_GROSS",
            "withholding",
            1,
        ),
        (
            "leaf-k-refund-above",
            "K_REFUNDS_ABOVE_PART",
            "parts/0/refunded",
            2,
        ),
        (
            "leaf-k-nonincome-refund",
            "K_REFUNDS_ONLY_ON_INCOME",
            "parts/0/refunded",
            3,
        ),
    ] {
        let good = k1099();
        let mut bad = good.clone();
        match mutation {
            0 => bad.gross += 1,
            1 => bad.withholding = bad.gross + 1,
            2 => bad.parts[0].refunded = bad.gross + 1,
            _ => bad.parts[0].refunded = 1,
        }
        document_case(
            case,
            TaxSection::SelfEmployment,
            IncomeDocument::Form1099K(good),
            IncomeDocument::Form1099K(bad),
            key,
            field,
        );
    }
}
fn misc1099() -> Form1099Misc {
    FederalForm1099Misc {
        rents: 0,
        rents_use: None,
        royalties: 0,
        royalties_use: None,
        other: 10_000,
        other_use: Some(IncomeUse::Income {
            kind: OtherIncomeKind::AlaskaDividend,
        }),
        withholding: 0,
        fishing: 0,
        medical: 0,
        substitute: 0,
        attorney: 0,
        client_funds: 0,
        fish_sales: 0,
        deferred: 0,
        business: None,
    }
    .page()
}
#[test]
fn unit47_leaf_misc_sources_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-misc-withholding",
            "MISC_WITHHOLDING_ABOVE_INCOME",
            "withholding",
            0,
        ),
        (
            "leaf-misc-rents-use",
            "MISC_RENTS_NEED_A_USE",
            "rents_use",
            1,
        ),
        (
            "leaf-misc-royalties-use",
            "MISC_ROYALTIES_NEED_A_USE",
            "royalties_use",
            2,
        ),
        (
            "leaf-misc-other-use",
            "MISC_OTHER_NEEDS_A_USE",
            "other_use",
            3,
        ),
        (
            "leaf-misc-client-funds",
            "MISC_CLIENT_FUNDS_ABOVE_PROCEEDS",
            "client_funds",
            4,
        ),
    ] {
        let good = misc1099();
        let mut bad = good.clone();
        match mutation {
            0 => bad.withholding = bad.other + 1,
            1 => bad.rents = 1,
            2 => bad.royalties = 1,
            3 => bad.other_use = None,
            _ => bad.client_funds = 1,
        }
        document_case(
            case,
            TaxSection::OtherIncome,
            IncomeDocument::Form1099Misc(good),
            IncomeDocument::Form1099Misc(bad),
            key,
            field,
        );
    }
}
fn home() -> MainHomeSale {
    MainHomeSale {
        address: "Unit47 synthetic home".into(),
        closed: 20260615,
        proceeds: 45_000_000,
        property_received: false,
        foreign_person: false,
        form_1099s: true,
        acquired: 20150301,
        expenses: 2_500_000,
        basis: 20_000_000,
        lived: HomeLived::WholeTime,
        earlier_sale: EarlierHomeSale::No,
        spouse: HomeSpouse::NotJoint,
        reason: HomeSaleReason::TwoYears,
        use_history: HomeRentalUse {
            depreciation: 0,
            nonqualified_days: 0,
        },
        more: HomeSaleMore::Nothing,
    }
}
#[test]
fn unit47_leaf_home_date_and_owned_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-home-acquired",
            "HOME_ACQUIRED_AFTER_SALE",
            "acquired",
            0,
        ),
        ("leaf-home-lived", "HOME_LIVED_ABOVE_OWNED", "lived", 1),
        (
            "leaf-home-earlier",
            "HOME_EARLIER_SALE_AFTER_SALE",
            "earlier_sale/sold",
            2,
        ),
        (
            "leaf-home-nonqualified",
            "HOME_NONQUALIFIED_ABOVE_OWNED",
            "use_history/nonqualified_days",
            3,
        ),
    ] {
        let good = home();
        let mut bad = good.clone();
        match mutation {
            0 => bad.acquired = bad.closed,
            1 => bad.lived = HomeLived::Days { days: 1827 },
            2 => bad.earlier_sale = EarlierHomeSale::Yes { sold: bad.closed },
            _ => bad.use_history.nonqualified_days = 10_000,
        }
        document_case(
            case,
            TaxSection::InvestmentSales,
            IncomeDocument::MainHomeSale(good),
            IncomeDocument::MainHomeSale(bad),
            key,
            field,
        );
    }
}
#[test]
fn unit47_leaf_mtc_generic_record_locator_save_atomically() {
    let mut session = crate::tests::federal::minimum_tax::session(FederalStatus::Single);
    let good = MinimumTaxHistories::from_current_unit(crate::tests::federal::minimum_tax::source());
    let mut bad = good.clone();
    bad.returns[0].facts.base = 100_000_000_001;
    let rejected = crate::tests::federal::minimum_tax::request(&mut session, bad);
    reject_atomic(
        "leaf-mtc-record",
        &mut session,
        rejected,
        "MTC_HISTORY_INVALID",
        "[value/credits/0/returns/0/facts]",
    );
    let corrected = crate::tests::federal::minimum_tax::request(&mut session, good);
    accepted(&send(&mut session, corrected));
    assert!(
        session
            .draft
            .credits
            .as_ref()
            .unwrap()
            .minimum_tax
            .is_some()
    );
}

fn form2555() -> Form2555 {
    Form2555 {
        qualification: ForeignQualification::Presence {
            starts_on: 20260101,
            foreign_days: 340,
        },
        tax_home: true,
        revocation: ExclusionRevocation::None,
        exclusion: true,
        prior_year_pay: 0,
        prior_year_history: None,
        multi_year_pay: vec![],
        housing: ForeignHousing::None,
    }
}
#[test]
fn unit47_leaf_foreign_qualification_paths_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-foreign-presence-start",
            "FOREIGN_PRESENCE_PERIOD_OUTSIDE_RETURN_YEAR",
            "qualification/starts_on",
            0,
        ),
        (
            "leaf-foreign-presence-days",
            "FOREIGN_PRESENCE_DAYS_ABOVE_PERIOD",
            "qualification/foreign_days",
            1,
        ),
        (
            "leaf-foreign-residence-began",
            "FOREIGN_RESIDENCE_BEGAN_AFTER_RETURN_YEAR",
            "qualification/began",
            2,
        ),
        (
            "leaf-foreign-residence-ended",
            "FOREIGN_RESIDENCE_ENDED_BEFORE_RETURN_YEAR",
            "qualification/ended",
            3,
        ),
        (
            "leaf-foreign-residence-year",
            "FOREIGN_RESIDENCE_NO_FULL_YEAR",
            "qualification",
            4,
        ),
        (
            "leaf-foreign-prior-history",
            "FOREIGN_PRIOR_ONLY_NEEDS_HISTORY",
            "prior_year_history",
            5,
        ),
        (
            "leaf-foreign-nothing",
            "FORM_2555_NOTHING_CLAIMED",
            "exclusion",
            6,
        ),
        (
            "leaf-foreign-revocation",
            "FOREIGN_REVOCATION_YEAR_OUTSIDE_PERIOD",
            "revocation",
            7,
        ),
    ] {
        let good = form2555();
        let mut bad = good.clone();
        match mutation {
            0 => {
                bad.qualification = ForeignQualification::Presence {
                    starts_on: 20250101,
                    foreign_days: 340,
                }
            }
            1 => {
                bad.qualification = ForeignQualification::Presence {
                    starts_on: 20260101,
                    foreign_days: 366,
                }
            }
            2 => {
                bad.qualification = ForeignQualification::Residence {
                    began: 20270101,
                    ended: None,
                    basis: ResidenceBasis::Citizen,
                    nonresident_statement: false,
                }
            }
            3 => {
                bad.qualification = ForeignQualification::Residence {
                    began: 20230101,
                    ended: Some(20251231),
                    basis: ResidenceBasis::Citizen,
                    nonresident_statement: false,
                }
            }
            4 => {
                bad.qualification = ForeignQualification::Residence {
                    began: 20260201,
                    ended: Some(20261101),
                    basis: ResidenceBasis::Citizen,
                    nonresident_statement: false,
                }
            }
            5 => bad.qualification = ForeignQualification::PriorYearOnly,
            6 => bad.exclusion = false,
            _ => {
                bad.revocation = ExclusionRevocation::Revoked {
                    year: 2020,
                    elections: RevokedElections::EarnedIncome,
                    consent: true,
                }
            }
        }
        document_case(
            case,
            TaxSection::WorkAbroad,
            IncomeDocument::Form2555(good),
            IncomeDocument::Form2555(bad),
            key,
            field,
        );
    }
}
fn housing() -> ForeignHousing {
    ForeignHousing::Claim {
        stays: vec![ForeignHousingStay {
            location: ForeignHousingLocation::General,
            starts_on: 20260101,
            ends_on: 20261231,
        }],
        expenses: 100_000,
        nontaxable_allowances: 0,
        prior: None,
        household: ForeignHousingHousehold::OnlyClaim,
    }
}
#[test]
fn unit47_leaf_foreign_housing_source_paths_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-housing-year",
            "FOREIGN_HOUSING_STAY_OUTSIDE_RETURN_YEAR",
            "housing/stays/0/starts_on",
            0,
        ),
        (
            "leaf-housing-period",
            "FOREIGN_HOUSING_STAY_OUTSIDE_PERIOD",
            "housing/stays/0/starts_on",
            1,
        ),
        (
            "leaf-housing-prior",
            "HOUSING_CARRYOVER_DEDUCTED_ABOVE_AMOUNT",
            "housing/prior/deducted",
            2,
        ),
        (
            "leaf-housing-shares",
            "HOUSING_SHARES_ABOVE_TOTAL",
            "housing/expenses",
            3,
        ),
    ] {
        let mut good = form2555();
        good.housing = housing();
        let mut bad = good.clone();
        if mutation == 1 {
            bad.qualification = ForeignQualification::Presence {
                starts_on: 20260201,
                foreign_days: 330,
            };
        }
        let ForeignHousing::Claim {
            stays,
            prior,
            household,
            ..
        } = &mut bad.housing
        else {
            unreachable!()
        };
        match mutation {
            0 => stays[0].starts_on = 20251231,
            1 => {}
            2 => *prior = Some(HousingCarryover { amount: 100, deducted: 101 }),
            _ => *household = ForeignHousingHousehold::Shared { total_expenses: 99_999 },
        }
        document_case(
            case,
            TaxSection::WorkAbroad,
            IncomeDocument::Form2555(good),
            IncomeDocument::Form2555(bad),
            key,
            field,
        );
    }
}
fn weather() -> FarmWeatherSale {
    FarmWeatherSale {
        class: FarmLivestockClass::Cattle,
        source: FarmWeatherSource::RaisedForSale,
        normal_sold: 100,
        all_sold: 135,
        weather_excess: 35,
        class_income: 100_000,
        principal_farming: true,
        assistance: FarmWeatherAssistance::Designated {
            date: Some(20260810),
        },
        weather_relationship: "Drought forced 35 additional sales".into(),
        prior_sales: vec![],
        usual_practice_support: "Prior sales records".into(),
        elect: true,
    }
}
#[test]
fn unit47_leaf_farm_nested_sources_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-farm-limits",
            "FARM_LIMITATIONS_INVALID",
            "more/specials/limitations/1",
            0,
        ),
        (
            "leaf-farm-other-source",
            "FARM_OTHER_SOURCE_REQUIRED",
            "more/specials/other_situation",
            1,
        ),
        (
            "leaf-farm-method-year",
            "FARM_METHOD_ADOPTED_YEAR",
            "more/specials/method_status/adopted",
            2,
        ),
        (
            "leaf-farm-weather-count",
            "FARM_WEATHER_COUNTS",
            "more/specials/weather_sales/0/weather_excess",
            3,
        ),
        (
            "leaf-farm-weather-split",
            "FARM_WEATHER_INCOME_SPLIT",
            "more/specials/weather_sales/0/source/raised",
            4,
        ),
        (
            "leaf-farm-weather-class",
            "FARM_WEATHER_DUPLICATE_CLASS",
            "more/specials/weather_sales/1/class",
            5,
        ),
        (
            "leaf-farm-weather-years",
            "FARM_WEATHER_YEARS",
            "more/specials/weather_sales/0/prior_sales/1/year",
            6,
        ),
        (
            "leaf-farm-weather-income",
            "FARM_WEATHER_ABOVE_PRODUCT_INCOME",
            "more/specials/weather_sales",
            7,
        ),
    ] {
        let mut good = crate::tests::cross::interview::cash_farm(1_000_000, 0);
        if mutation >= 3 {
            good.more.specials.weather_sales = vec![weather()];
        }
        let mut bad = good.clone();
        match mutation {
            0 => bad.more.specials.limitations = vec![1, 1],
            1 => bad.more.specials.limitations = vec![10],
            2 => bad.more.specials.method_status.adopted = 2027,
            3 => bad.more.specials.weather_sales[0].weather_excess = 36,
            4 => {
                bad.more.specials.weather_sales[0].source =
                    FarmWeatherSource::MixedProducts { raised: 100_001 }
            }
            5 => bad.more.specials.weather_sales.push(weather()),
            6 => {
                bad.more.specials.weather_sales[0].prior_sales = vec![
                    FarmAnimalYear {
                        year: 2025,
                        count: 100,
                    },
                    FarmAnimalYear {
                        year: 2025,
                        count: 100,
                    },
                ]
            }
            _ => bad.more.specials.weather_sales[0].class_income = 1_000_001,
        }
        document_case(
            case,
            TaxSection::Farming,
            IncomeDocument::Farm(good),
            IncomeDocument::Farm(bad),
            key,
            field,
        );
    }
}
fn mtc_case(case: &str, bad: MinimumTaxHistories, key: &str, local: &str) {
    let mut session = crate::tests::federal::minimum_tax::session(FederalStatus::Single);
    let rejected = crate::tests::federal::minimum_tax::request(&mut session, bad);
    let path = alloc::format!("[value/credits/0/{local}]");
    reject_atomic(case, &mut session, rejected, key, &path);
    let corrected = crate::tests::federal::minimum_tax::request(
        &mut session,
        MinimumTaxHistories::from_current_unit(crate::tests::federal::minimum_tax::source()),
    );
    accepted(&send(&mut session, corrected));
    assert!(
        session
            .draft
            .credits
            .as_ref()
            .unwrap()
            .minimum_tax
            .is_some()
    );
}
#[test]
fn unit47_leaf_mtc_retained_relationships_save_atomically() {
    for (case, key, local, mutation) in [
        (
            "leaf-mtc-unit-twice",
            "MTC_PRIOR_RETURN_TWICE",
            "returns/1/unit",
            0,
        ),
        (
            "leaf-mtc-unit-status",
            "MTC_PRIOR_STATUS_UNIT",
            "returns/0/facts/status",
            1,
        ),
        (
            "leaf-mtc-owner",
            "MTC_OWNER_NOT_ON_RETURN",
            "returns/0/unit",
            2,
        ),
        (
            "leaf-mtc-nol-year-twice",
            "MTC_NOL_YEAR_TWICE",
            "returns/0/facts/mtcnol/records/amounts/1/year",
            3,
        ),
        (
            "leaf-mtc-nol-same-year",
            "MTC_NOL_SAME_YEAR",
            "returns/0/facts/mtcnol/records/amounts/0/year",
            4,
        ),
        (
            "leaf-mtc-statement",
            "MTC_FTC_STATEMENT_LIMIT",
            "returns/0/facts/foreign_credit/credit",
            5,
        ),
        (
            "leaf-mtc-gain-bands",
            "MTC_REGULAR_GAIN_BANDS",
            "returns/0/facts/gains/regular_fifteen",
            6,
        ),
        (
            "leaf-mtc-gain-cap",
            "MTC_GAIN_CAP_BELOW_PREFERENTIAL",
            "returns/0/facts/gains/gain_cap",
            7,
        ),
    ] {
        let mut bad = MinimumTaxHistories::from_current_unit(crate::tests::federal::minimum_tax::source());
        match mutation {
            0 => bad.returns.push(bad.returns[0].clone()),
            1 => bad.returns[0].facts.status = FederalStatus::Joint,
            2 => bad.returns[0].unit = MinimumTaxHistoryUnit::Individual { owner: Owner::Spouse },
            3 => {
                bad.returns[0].facts.mtcnol = MinimumTaxLoss::Retained {
                    records: MinimumTaxLossRecords {
                        amounts: vec![
                            MinimumTaxLossAmount {
                                year: 2024,
                                available: 100,
                            },
                            MinimumTaxLossAmount {
                                year: 2024,
                                available: 100,
                            },
                        ],
                    },
                }
            }
            4 => {
                bad.returns[0].facts.mtcnol = MinimumTaxLoss::Retained {
                    records: MinimumTaxLossRecords {
                        amounts: vec![MinimumTaxLossAmount {
                            year: 2025,
                            available: 100,
                        }],
                    },
                }
            }
            5 => {
                bad.returns[0].facts.foreign_credit =
                    MinimumTaxForeignCredit::PriorStatementElection { credit: 30_001 }
            }
            6 => {
                bad.returns[0].facts.gains = Some(MinimumTaxRateFacts::ScheduleD(
                    MinimumTaxScheduleDRateFacts {
                        preferential: 0,
                        unrecaptured: 0,
                        gain_cap: 0,
                        regular_zero: 0,
                        regular_fifteen: 1,
                    },
                ))
            }
            _ => {
                bad.returns[0].facts.gains = Some(MinimumTaxRateFacts::ScheduleD(
                    MinimumTaxScheduleDRateFacts {
                        preferential: 1,
                        unrecaptured: 0,
                        gain_cap: 0,
                        regular_zero: 0,
                        regular_fifteen: 0,
                    },
                ))
            }
        }
        mtc_case(case, bad, key, local);
    }
}

#[test]
fn unit47_leaf_employee_pay_source_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-employee-1099",
            "FORM_8919_CODE_H_NEEDS_1099",
            "form1099",
            0,
        ),
        (
            "leaf-employee-withholding",
            "FORM_8919_WITHHOLDING_ABOVE_PAY",
            "withholding",
            1,
        ),
    ] {
        let good = EmployeeWages {
            wages: 100_000,
            withholding: 0,
            form1099: true,
            reason: EmployeeWageReason::SameFirmForms,
            complete: true,
        };
        let mut bad = good.clone();
        if mutation == 0 {
            bad.form1099 = false;
        } else {
            bad.withholding = 100_001;
        }
        document_case(
            case,
            TaxSection::Wages,
            IncomeDocument::Form8919(good),
            IncomeDocument::Form8919(bad),
            key,
            field,
        );
    }
}
#[test]
fn unit47_leaf_scholarship_source_sum_save_is_atomic() {
    let good = Scholarship::Education {
        data: EducationAward {
            received: 100_000,
            restricted: 20_000,
            qualified_paid: 80_000,
            degree_candidate: true,
            services: ScholarshipService::None,
            ordinary: true,
        },
    };
    let mut bad = good.clone();
    let Scholarship::Education { data } = &mut bad else {
        unreachable!()
    };
    data.qualified_paid += 1;
    document_case(
        "leaf-scholarship-spending",
        TaxSection::OtherIncome,
        IncomeDocument::Scholarship(good),
        IncomeDocument::Scholarship(bad),
        "SCHOLARSHIP_SPENDING_ABOVE_AWARD",
        "data/qualified_paid",
    );
}
fn qof() -> QofInvestment {
    QofInvestment {
        fund: "Unit47 synthetic QOF".into(),
        invested_on: 20250101,
        short_term: 100_000,
        long_term: 0,
        gain_code: QofGainCode::None,
        event: QofEvent::Held { fair_value: None },
        special: false,
    }
}
#[test]
fn unit47_leaf_opportunity_fund_character_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        ("leaf-qof-no-gain", "QOF_NO_DEFERRED_GAIN", "short_term", 0),
        (
            "leaf-qof-1231",
            "QOF_SECTION_1231_LONG_TERM",
            "short_term",
            1,
        ),
        (
            "leaf-qof-collectibles",
            "QOF_COLLECTIBLES_LONG_TERM",
            "short_term",
            2,
        ),
        (
            "leaf-qof-sold-before",
            "QOF_SOLD_BEFORE_INVESTED",
            "event/sold_on",
            3,
        ),
    ] {
        let good = qof();
        let mut bad = good.clone();
        match mutation {
            0 => bad.short_term = 0,
            1 => bad.gain_code = QofGainCode::Section1231,
            2 => bad.gain_code = QofGainCode::Collectibles,
            _ => bad.event = QofEvent::Sold { sold_on: 20250101, amount: 100_000 },
        }
        document_case(
            case,
            TaxSection::InvestmentSales,
            IncomeDocument::OpportunityFund(good),
            IncomeDocument::OpportunityFund(bad),
            key,
            field,
        );
    }
}
fn qsbs() -> QsbsSale {
    QsbsSale {
        issuer: "Unit47 synthetic corporation".into(),
        lots: vec![QsbStockLot {
            issued: 20100101,
            sold: 20260615,
            basis: 100_000,
            proceeds: 200_000,
            zone_gain_through_2018: None,
        }],
        reporting: BasisReporting::NoForm1099B,
        withholding: 0,
        prior_eligible_gain: 0,
        issuer_qualified: true,
        active_business: true,
        no_disqualification: true,
        zone_60_exclusion: None,
        complete_history: true,
        special: false,
    }
}
#[test]
fn unit47_leaf_qsbs_original_issue_and_zone_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-qsbs-issued",
            "QSBS_LOT_OUTSIDE_SECTION_1202_DATES",
            "lots/0/issued",
            0,
        ),
        (
            "leaf-qsbs-proceeds",
            "QSBS_LOT_PROCEEDS_NOT_ABOVE_BASIS",
            "lots/0/proceeds",
            1,
        ),
        (
            "leaf-qsbs-zone-gain",
            "QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN",
            "lots/0/zone_gain_through_2018",
            2,
        ),
        (
            "leaf-qsbs-zone-date",
            "QSBS_LOT_ZONE_GAIN_OUTSIDE_ZONE_DATES",
            "lots/0/zone_gain_through_2018",
            3,
        ),
    ] {
        let good = qsbs();
        let mut bad = good.clone();
        match mutation {
            0 => bad.lots[0].issued = 20260101,
            1 => bad.lots[0].proceeds = 100_000,
            2 => bad.lots[0].zone_gain_through_2018 = Some(100_001),
            _ => bad.lots[0].zone_gain_through_2018 = Some(1),
        }
        document_case(
            case,
            TaxSection::InvestmentSales,
            IncomeDocument::QsbsSale(good),
            IncomeDocument::QsbsSale(bad),
            key,
            field,
        );
    }
}
fn loss_history() -> LossCarryforward {
    LossCarryforward::from_histories(
        &NolHistory::Records {
            years: vec![NolYear { ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                year: 2025,
                balance: 100_000,
                injured_owners: None,
                research_parts: None,
            }],
            confirmed: true,
        },
        &AtnolHistory::None,
    )
    .unwrap()
    .unwrap()
}
#[test]
fn unit47_leaf_nol_history_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        ("leaf-nol-empty", "NOL_NOTHING_ENTERED", "regular", 0),
        ("leaf-nol-year-twice", "NOL_YEAR_TWICE", "regular/1/year", 1),
        (
            "leaf-atnol-exception",
            "ATNOL_EXCEPTION_AFTER_2017",
            "amt/0/special_exception",
            2,
        ),
    ] {
        let good = loss_history();
        let mut bad = good.clone();
        match mutation {
            0 => bad.regular.clear(),
            1 => bad.regular.push(bad.regular[0].clone()),
            _ => bad.amt = vec![AtnolYear { year: 2025, balance: 100, special_exception: true }],
        }
        document_case(
            case,
            TaxSection::SelfEmployment,
            IncomeDocument::NetOperatingLoss(good),
            IncomeDocument::NetOperatingLoss(bad),
            key,
            field,
        );
    }
}

fn gift_editor(session: &mut Interview) -> P {
    let Reply::Edit(page @ P::SaveGiftCarryovers { .. }) =
        crate::tests::cross::interview::from_review(session, TaxSection::GiftCarryovers)
    else {
        panic!("gift editor required")
    };
    page
}
#[test]
fn unit47_leaf_gift_and_carryover_sources_save_atomically() {
    for (case, key, local, mutation) in [
        (
            "leaf-gift-acquired",
            "GIFT_BOUGHT_AFTER_GIVEN",
            "gifts/0/gift/acquired",
            0,
        ),
        (
            "leaf-gift-value",
            "GIFT_VALUE_ABOVE_COST",
            "gifts/0/gift/value",
            1,
        ),
        (
            "leaf-gift-benefit",
            "GIFT_BENEFIT_ABOVE_VALUE",
            "gifts/0/gift/benefit_received",
            2,
        ),
        (
            "leaf-gift-vehicle-date",
            "VEHICLE_SOLD_BEFORE_GIVEN",
            "gifts/0/gift/kind/treatment/sold",
            3,
        ),
        (
            "leaf-gift-historic-purpose",
            "GIFT_HISTORIC_CREDITS_PURPOSE",
            "gifts/0/gift/kind/historic_building",
            4,
        ),
        (
            "leaf-gift-carry-year",
            "CHARITY_CARRY_TOO_OLD",
            "carryovers/0/year",
            5,
        ),
        (
            "leaf-gift-disaster-year",
            "CHARITY_CARRY_DISASTER_YEAR",
            "carryovers/0/year",
            6,
        ),
        (
            "leaf-gift-carry-twice",
            "CHARITY_CARRY_TWICE",
            "carryovers/1/year",
            7,
        ),
        (
            "leaf-gift-election-appreciation",
            "GIFT_ELECTION_NEEDS_CARRY_APPRECIATION",
            "carryovers/0/appreciation",
            8,
        ),
        (
            "leaf-gift-poor-condition",
            "GIFT_ITEM_NEEDS_APPRAISAL",
            "gifts/0/appraisal",
            9,
        ),
    ] {
        let mut draft = complete(FederalStatus::Single);
        let _ = itemized(draft.deductions.as_mut().unwrap());
        let mut session = Interview::new(TEST_USER, draft);
        let mut corrected = gift_editor(&mut session);
        let P::SaveGiftCarryovers { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.gifts = vec![used_goods(20_000).into()];
        value.carryovers = vec![prior_household_carry().into()];
        let mut rejected = corrected.clone();
        let P::SaveGiftCarryovers { value, .. } = &mut rejected else {
            unreachable!()
        };
        let gift = &mut value.gifts[0].gift;
        match mutation {
            0 => gift.acquired = 20261201,
            1 => gift.value = gift.basis + 1,
            2 => gift.benefit_received = Some(gift.value),
            3 => {
                gift.kind = GiftKind::Vehicle {
                    vin: "TESTVIN1234567890".into(),
                    treatment: VehicleUse::Sold {
                        sold: 20260101,
                        proceeds: 10_000,
                    },
                }
            }
            4 => {
                gift.kind = GiftKind::Conservation {
                    purpose: ConservationPurpose::NaturalHabitat,
                    historic_building: Some(HistoricBuildingGift {
                        rehabilitation_credits: 1,
                        building_value: 20_000,
                    }),
                    nps_number: None,
                    farmer_rancher: false,
                }
            }
            5 => value.carryovers[0].year = 2020,
            6 => value.carryovers[0].class = CharityClass::DisasterRelief,
            7 => value.carryovers.push(value.carryovers[0].clone()),
            8 => {
                value.carryovers[0].class = CharityClass::CapitalGain30;
                value.basis_election = true;
            }
            _ => {
                gift.kind = GiftKind::Goods {
                    kind: GoodsKind::Clothing,
                    condition: GoodsCondition::NotGood,
                }
            }
        }
        let field = alloc::format!("[value/{local}]");
        reject_atomic(case, &mut session, rejected, key, &field);
        let revision = session.draft.revision;
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft.revision, revision + 1);
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let reopened = gift_editor(&mut cold);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
    }
}

#[test]
fn unit47_leaf_hsa_local_records_save_atomically() {
    use crate::tests::cross::interview::{hsa_account, open_hsa_page};
    for (case, key, local, mutation) in [
        (
            "leaf-hsa-correction-returned",
            "HSA_CORRECTION_RETURNED_BEFORE_DEPOSIT",
            "corrections/0/returned",
            0,
        ),
        (
            "leaf-hsa-correction-deadline",
            "HSA_CORRECTION_DEADLINE_BEFORE_DEPOSIT",
            "corrections/0/deadline",
            1,
        ),
        (
            "leaf-hsa-correction-loss",
            "HSA_CORRECTION_LOSS_ABOVE_PRINCIPAL",
            "corrections/0/earnings",
            2,
        ),
        (
            "leaf-hsa-late-withdrawal",
            "HSA_LATE_EXCESS_NOT_LATE",
            "late_excess/0/date",
            3,
        ),
        (
            "leaf-hsa-prior-employer-deadline",
            "HSA_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE",
            "prior_employer_excess/0/deadline",
            4,
        ),
        (
            "leaf-hsa-later-employer-deposits",
            "HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER",
            "excess_accounting/later_employer_deposits",
            5,
        ),
        (
            "leaf-hsa-earlier-funding-coverage",
            "HSA_FUNDING_NEEDS_COVERAGE",
            "uses/0/coverage",
            6,
        ),
        (
            "leaf-hsa-funding-history-loss",
            "HSA_FUNDING_LOSS_BEFORE_FUNDING",
            "funding_history/0/loss/date",
            7,
        ),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open_hsa_page(&mut session);
        let P::SaveHsa { value, .. } = &mut corrected else {
            unreachable!()
        };
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 100_000);
        match mutation {
            0..=2 => account.corrections.push(HsaCorrectionPage {
                document: None,
                failed_deposit: None,
                year: 2026,
                contributed_on: 20260501,
                principal: 10_000,
                earnings: 0,
                returned: 20260601,
                deadline: 20261015,
                payer: HsaCorrectionPayer::Personal,
            }),
            3 => account.late_excess.push(HsaLateExcessWithdrawal {
                contribution_year: 2025,
                deadline: 20260415,
                date: 20260501,
                amount: 10_000,
                disabled: false,
            }),
            4 => account.prior_employer_excess.push(HsaPriorEmployerExcessDeposit {
                contributed_on: 20260401,
                deadline: 20260415,
                amount: 10_000,
                in_wages: false,
            }),
            5 => {
                account.excess_accounting = Some(HsaExcessAccounting {
                    prior_year: None,
                    employer_excess_in_wages: 0,
                    year_end_value: 100_000,
                    later_return_year_deposits: 0,
                    later_employer_deposits: 0,
                })
            }
            6 => account.uses.push(HsaFundingUseHistory {
                funded: 20240101,
                coverage: HsaCoverage::SelfOnly,
                amount: 10_000,
            }),
            _ => account.funding_history.push(HsaFundingHistory {
                funded: 20250501,
                amount: 10_000,
                loss: None,
            }),
        }
        value.accounts = vec![account];
        value.family_split = FamilySplit::NotApplicable;
        value.other_coverage = None;
        let mut rejected = corrected.clone();
        let P::SaveHsa { value, .. } = &mut rejected else {
            unreachable!()
        };
        let account = &mut value.accounts[0];
        match mutation {
            0 => account.corrections[0].returned = 20260401,
            1 => account.corrections[0].deadline = 20260401,
            2 => account.corrections[0].earnings = -10_001,
            3 => account.late_excess[0].date = account.late_excess[0].deadline,
            4 => account.prior_employer_excess[0].deadline = 20260331,
            5 => account.excess_accounting.as_mut().unwrap().later_employer_deposits = 1,
            6 => account.uses[0].coverage = HsaCoverage::None,
            _ => {
                account.funding_history[0].loss = Some(HsaEligibilityLoss {
                    date: 20250401,
                    reason: HsaLossReason::Other,
                })
            }
        }
        reject_atomic(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[value/accounts/0/{local}]"),
        );
        let revision = session.draft.revision;
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft.revision, revision + 1);
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let reopened = open_hsa_page(&mut cold);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
    }
}

#[test]
fn unit47_leaf_hsa_cross_record_relationships_save_atomically() {
    use crate::tests::cross::interview::{hsa_account, open_hsa_page};
    for (case, key, local, mutation) in [
        (
            "leaf-hsa-corrected-personal-total",
            "HSA_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED",
            "personal",
            0,
        ),
        (
            "leaf-hsa-later-personal-total",
            "HSA_LATER_PERSONAL_ABOVE_NET",
            "excess_accounting/later_return_year_deposits",
            1,
        ),
        (
            "leaf-hsa-history-same-month",
            "HSA_FUNDING_HISTORY_SAME_MONTH",
            "funding_history/1/funded",
            2,
        ),
        (
            "leaf-hsa-earlier-use-same-month",
            "HSA_FUNDING_HISTORY_SAME_MONTH",
            "uses/1/funded",
            3,
        ),
        (
            "leaf-hsa-last-month-december",
            "HSA_LAST_MONTH_DECEMBER_NEEDED",
            "last_month_history/months/11",
            4,
        ),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = open_hsa_page(&mut session);
        let P::SaveHsa { value, .. } = &mut corrected else {
            unreachable!()
        };
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 100_000);
        match mutation {
            0 => account.corrections.push(
                HsaCorrection {
                    document: None,
                    failed_deposit: None,
                    year: 2026,
                    contributed_on: 20260501,
                    principal: 10_000,
                    earnings: 0,
                    returned: 20260601,
                    deadline: 20261015,
                    payer: HsaCorrectionPayer::Personal,
                }
                .page(),
            ),
            1 => {
                account.excess_accounting = Some(HsaExcessAccounting {
                    prior_year: None,
                    employer_excess_in_wages: 0,
                    year_end_value: 100_000,
                    later_return_year_deposits: 0,
                    later_employer_deposits: 0,
                })
            }
            2 => {
                account.funding_history = vec![
                    HsaFundingHistory {
                        funded: 20250501,
                        amount: 10_000,
                        loss: None,
                    },
                    HsaFundingHistory {
                        funded: 20250601,
                        amount: 10_000,
                        loss: None,
                    },
                ]
            }
            3 => {
                account.uses = vec![
                    HsaFundingUseHistory {
                        funded: 20240101,
                        coverage: HsaCoverage::SelfOnly,
                        amount: 10_000,
                    },
                    HsaFundingUseHistory {
                        funded: 20240201,
                        coverage: HsaCoverage::Family,
                        amount: 10_000,
                    },
                ]
            }
            _ => {
                account.last_month_history = Some(HsaLastMonthHistory {
                    months: vec![HsaCoverage::SelfOnly; 12],
                    contributions: 100_000,
                    archer: 0,
                    family: None,
                    loss: None,
                })
            }
        }
        value.accounts = vec![account];
        value.family_split = FamilySplit::NotApplicable;
        value.other_coverage = None;
        let mut rejected = corrected.clone();
        let P::SaveHsa { value, .. } = &mut rejected else {
            unreachable!()
        };
        let account = &mut value.accounts[0];
        match mutation {
            0 => account.personal = 9_999,
            1 => account.excess_accounting.as_mut().unwrap().later_return_year_deposits = 100_001,
            2 => account.funding_history[1].funded = 20250515,
            3 => account.uses[1].funded = 20240115,
            _ => account.last_month_history.as_mut().unwrap().months[11] = HsaCoverage::None,
        }
        reject_atomic(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[value/accounts/0/{local}]"),
        );
        let revision = session.draft.revision;
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft.revision, revision + 1);
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let reopened = open_hsa_page(&mut cold);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
    }
}

#[test]
fn unit47_leaf_hsa_correction_source_graph_save_atomically() {
    use crate::tests::cross::interview::{hsa_account, open_hsa_page};
    for (case, key, local, mutation) in [
        (
            "leaf-hsa-source-details",
            "HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED",
            "corrections",
            0,
        ),
        (
            "leaf-hsa-source-gross",
            "HEALTH_CORRECTION_DOCUMENT_GROSS",
            "corrections",
            1,
        ),
        (
            "leaf-hsa-source-earnings",
            "HEALTH_CORRECTION_DOCUMENT_EARNINGS",
            "corrections",
            2,
        ),
        (
            "leaf-hsa-source-year",
            "HEALTH_CORRECTION_DOCUMENT_YEAR",
            "corrections/0/returned",
            3,
        ),
        (
            "leaf-hsa-source-reference",
            "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE",
            "corrections/0/document",
            4,
        ),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        // Commit the actual accepted statement through its normal income editor.
        let mut source_page = open_topic(&mut session, TaxSection::OtherIncome);
        documents(
            &mut source_page,
            vec![document(
                0,
                IncomeDocument::Sa1099(Sa1099 {
                    origin: HealthDocumentOrigin::Form1099Sa,
                    gross: 10_000,
                    account: MedicalSaAccount::Hsa,
                    payment: MedicalSaPayment::ExcessReturned { earnings: Some(0) },
                }),
            )],
        );
        accepted(&send(&mut session, source_page));
        let source_id = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
            .unwrap()
            .id;
        let mut corrected = open_hsa_page(&mut session);
        let P::SaveHsa { value, .. } = &mut corrected else {
            unreachable!()
        };
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 100_000);
        account.corrections.push(
            HsaCorrection {
                document: Some(source_id),
                failed_deposit: None,
                year: 2026,
                contributed_on: 20260501,
                principal: 10_000,
                earnings: 0,
                returned: 20260601,
                deadline: 20261015,
                payer: HsaCorrectionPayer::Personal,
            }
            .page(),
        );
        value.accounts = vec![account];
        value.family_split = FamilySplit::NotApplicable;
        value.other_coverage = None;
        let mut rejected = corrected.clone();
        let P::SaveHsa { value, .. } = &mut rejected else {
            unreachable!()
        };
        let corrections = &mut value.accounts[0].corrections;
        match mutation {
            0 => corrections.clear(),
            1 => corrections[0].principal += 1,
            2 => {
                corrections[0].principal -= 1;
                corrections[0].earnings = 1;
            }
            3 => corrections[0].returned = 20270601,
            _ => corrections[0].document = Some(source_id + 100_000),
        }
        leaf_hsa_save_estimate_rejection(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[value/accounts/0/{local}]"),
        );
        leaf_hsa_correct_and_reopen(&mut session, corrected);
    }
}

#[test]
fn unit47_leaf_employee_stock_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-employee-stock-taxed",
            "EMPLOYEE_STOCK_SOLD_BEFORE_TAXED",
            "sold",
            0,
        ),
        (
            "leaf-employee-stock-year",
            "EMPLOYEE_STOCK_SALE_NOT_IN_RETURN_YEAR",
            "sold",
            1,
        ),
        (
            "leaf-employee-stock-wash",
            "SALE_WASH_SALE_ABOVE_LOSS",
            "more/wash_sale",
            2,
        ),
    ] {
        let good = EmployeeShareSale {
            description: "Synthetic award".into(),
            sold: 20260601,
            proceeds: 30_000,
            reported_basis: None,
            reporting: BasisReporting::ReportedToIrs,
            withholding: 0,
            taxed: 20250501,
            value: 20_000,
            paid: 10_000,
            more: EmployeeShareSaleMore::default(),
        };
        let mut bad = good.clone();
        match mutation {
            0 => bad.sold = 20250401,
            1 => bad.sold = 20270101,
            _ => bad.more.wash_sale = 1,
        }
        document_case(
            case,
            TaxSection::InvestmentSales,
            IncomeDocument::EmployeeShareSale(good),
            IncomeDocument::EmployeeShareSale(bad),
            key,
            field,
        );
    }
}

#[test]
fn unit47_leaf_espp_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-espp-exercised",
            "ESPP_EXERCISED_BEFORE_GRANTED",
            "exercised",
            0,
        ),
        ("leaf-espp-sold", "ESPP_SOLD_BEFORE_EXERCISED", "sold", 1),
        ("leaf-espp-year", "ESPP_SALE_NOT_IN_RETURN_YEAR", "sold", 2),
        (
            "leaf-espp-wash",
            "SALE_WASH_SALE_ABOVE_LOSS",
            "more/wash_sale",
            3,
        ),
    ] {
        let good = EsppShareSale {
            description: "Synthetic ESPP".into(),
            shares: 10.0,
            sold: 20260601,
            proceeds: 3_000,
            reported_basis: None,
            reporting: BasisReporting::ReportedToIrs,
            withholding: 0,
            granted: 20230101,
            exercised: 20240101,
            grant_market: 200,
            market_price: 200,
            exercise_price: 100,
            grant_price: 100,
            more: EsppShareSaleMore {
                in_wages: false,
                ..Default::default()
            },
        };
        let mut bad = good.clone();
        match mutation {
            0 => bad.exercised = 20220101,
            1 => bad.sold = 20231201,
            2 => bad.sold = 20270101,
            _ => bad.more.wash_sale = 1,
        }
        document_case(
            case,
            TaxSection::InvestmentSales,
            IncomeDocument::EsppShareSale(good),
            IncomeDocument::EsppShareSale(bad),
            key,
            field,
        );
    }
}

#[test]
fn unit47_leaf_iso_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-iso-exercised",
            "ISO_EXERCISED_BEFORE_GRANTED",
            "exercised",
            0,
        ),
        ("leaf-iso-sold", "ISO_SOLD_BEFORE_EXERCISED", "sold", 1),
        ("leaf-iso-year", "ISO_SALE_NOT_IN_RETURN_YEAR", "sold", 2),
    ] {
        let good = IsoShareSale {
            description: "Synthetic ISO".into(),
            shares: 10.0,
            sold: 20260601,
            proceeds: 3_000,
            reported_basis: None,
            reporting: BasisReporting::ReportedToIrs,
            withholding: 0,
            granted: 20230101,
            exercised: 20240101,
            exercise_price: 100,
            market_price: 200,
            more: IsoShareSaleMore {
                in_wages: false,
                ..Default::default()
            },
        };
        let mut bad = good.clone();
        match mutation {
            0 => bad.exercised = 20220101,
            1 => bad.sold = 20231201,
            _ => bad.sold = 20270101,
        }
        document_case(
            case,
            TaxSection::InvestmentSales,
            IncomeDocument::IsoShareSale(good),
            IncomeDocument::IsoShareSale(bad),
            key,
            field,
        );
    }
}

#[test]
fn unit47_leaf_nua_relationships_save_atomically() {
    for (case, key, field, mutation) in [
        (
            "leaf-nua-distributed",
            "NUA_SOLD_BEFORE_DISTRIBUTED",
            "sold",
            0,
        ),
        ("leaf-nua-year", "NUA_SALE_NOT_IN_2026", "sold", 1),
        (
            "leaf-nua-deferred",
            "NUA_ABOVE_DISTRIBUTION_VALUE",
            "deferred_nua",
            2,
        ),
        (
            "leaf-nua-inherited-before-distribution",
            "NUA_DIED_BEFORE_DISTRIBUTED",
            "more/inherited/died",
            3,
        ),
        (
            "leaf-nua-inherited-after-sale",
            "NUA_SOLD_BEFORE_DIED",
            "sold",
            4,
        ),
        (
            "leaf-nua-wash",
            "SALE_WASH_SALE_ABOVE_LOSS",
            "more/wash_sale",
            5,
        ),
    ] {
        let good = NuaShareSale {
            description: "Synthetic NUA".into(),
            sold: 20260601,
            proceeds: 30_000,
            reported_basis: None,
            reporting: BasisReporting::ReportedToIrs,
            withholding: 0,
            distributed: 20250501,
            distribution_value: 20_000,
            deferred_nua: 10_000,
            more: NuaShareSaleMore::default(),
        };
        let mut bad = good.clone();
        match mutation {
            0 => bad.sold = 20250401,
            1 => bad.sold = 20270101,
            2 => bad.deferred_nua = 20_001,
            3 | 4 => {
                bad.more.inherited = Some(NuaInheritance {
                    died: if mutation == 3 { 20250401 } else { 20260701 },
                    value: 20_000,
                    estate_tax: false,
                })
            }
            _ => bad.more.wash_sale = 1,
        }
        document_case(
            case,
            TaxSection::InvestmentSales,
            IncomeDocument::NuaShareSale(good),
            IncomeDocument::NuaShareSale(bad),
            key,
            field,
        );
    }
}

fn leaf_archer_owner(draft: &FederalDraft) -> ArcherContributorRow {
    let wage = draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
        .unwrap()
        .id;
    ArcherContributorRow {
        owner: Owner::Taxpayer,
        personal: 100_000,
        employer: 0,
        corrections: vec![],
        prior_employer_excess: vec![],
        excess_accounting: None,
        coverage: ArcherEligibility::FullYear {
            coverage: ArcherCoverage::SelfOnly {
                deductible: 300_000,
                out_of_pocket: 400_000,
            },
        },
        income: vec![wage],
    }
}
fn leaf_archer_excess() -> ArcherExcessAccounting {
    ArcherExcessAccounting {
        prior_year: None,
        year_end_value: 100_000,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
        employer_excess_in_wages: 0,
    }
}
fn leaf_archer_correction() -> ArcherCorrectionPage {
    ArcherCorrection {
        document: None,
        failed_deposit: None,
        year: 2026,
        contributed_on: 20260501,
        principal: 10_000,
        earnings: 0,
        returned: 20260601,
        deadline: 20261015,
        payer: ArcherCorrectionPayer::Personal,
    }
    .page()
}
#[test]
fn unit47_leaf_archer_local_relationships_save_atomically() {
    use crate::tests::cross::interview::open_hsa_page;
    for (case, key, local, mutation) in [
        (
            "leaf-archer-correction-returned",
            "ARCHER_CORRECTION_RETURNED_BEFORE_DEPOSIT",
            "owners/0/corrections/0/returned",
            0,
        ),
        (
            "leaf-archer-correction-deadline",
            "ARCHER_CORRECTION_DEADLINE_BEFORE_DEPOSIT",
            "owners/0/corrections/0/deadline",
            1,
        ),
        (
            "leaf-archer-correction-loss",
            "ARCHER_CORRECTION_LOSS_ABOVE_PRINCIPAL",
            "owners/0/corrections/0/earnings",
            2,
        ),
        (
            "leaf-archer-prior-deadline",
            "ARCHER_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE",
            "owners/0/prior_employer_excess/0/deadline",
            3,
        ),
        (
            "leaf-archer-later-subset",
            "ARCHER_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER",
            "owners/0/excess_accounting/later_employer_deposits",
            4,
        ),
        (
            "leaf-archer-personal-total",
            "ARCHER_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED",
            "owners/0/personal",
            5,
        ),
        (
            "leaf-archer-employer-total",
            "ARCHER_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED",
            "owners/0/employer",
            6,
        ),
        (
            "leaf-archer-later-personal",
            "ARCHER_LATER_PERSONAL_ABOVE_NET",
            "owners/0/excess_accounting/later_return_year_deposits",
            7,
        ),
        (
            "leaf-archer-later-employer",
            "ARCHER_LATER_EMPLOYER_ABOVE_NET",
            "owners/0/excess_accounting/later_employer_deposits",
            8,
        ),
        (
            "leaf-archer-owner-twice",
            "ARCHER_OWNER_TWICE",
            "owners/1/owner",
            9,
        ),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut row = leaf_archer_owner(&session.draft);
        match mutation {
            0..=2 | 5 => row.corrections.push(leaf_archer_correction()),
            3 => row.prior_employer_excess.push(ArcherPriorEmployerExcessDeposit {
                contributed_on: 20260401,
                deadline: 20260415,
                amount: 10_000,
                in_wages: false,
            }),
            4 | 7 | 8 => row.excess_accounting = Some(leaf_archer_excess()),
            6 => {
                row.personal = 0;
                row.employer = 20_000;
                let mut correction = leaf_archer_correction();
                correction.payer = ArcherCorrectionPayer::Employer { in_wages: false };
                row.corrections.push(correction);
            }
            _ => {}
        }
        if mutation == 8 {
            row.personal = 0;
            row.employer = 20_000;
        }
        let mut corrected = open_hsa_page(&mut session);
        let P::SaveHsa { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.archer = ArcherContributionsPage::Records {
            owners: vec![row],
            sharing: ArcherSharing::Equal,
        };
        let mut rejected = corrected.clone();
        let P::SaveHsa { value, .. } = &mut rejected else {
            unreachable!()
        };
        let ArcherContributionsPage::Records { owners, .. } = &mut value.archer else {
            unreachable!()
        };
        match mutation {
            0 => owners[0].corrections[0].returned = 20260401,
            1 => owners[0].corrections[0].deadline = 20260401,
            2 => owners[0].corrections[0].earnings = -10_001,
            3 => owners[0].prior_employer_excess[0].deadline = 20260331,
            4 => owners[0].excess_accounting.as_mut().unwrap().later_employer_deposits = 1,
            5 => owners[0].personal = 9_999,
            6 => owners[0].employer = 9_999,
            7 => owners[0].excess_accounting.as_mut().unwrap().later_return_year_deposits = 100_001,
            8 => {
                let excess = owners[0].excess_accounting.as_mut().unwrap();
                excess.later_return_year_deposits = 20_001;
                excess.later_employer_deposits = 20_001;
            }
            _ => owners.push(owners[0].clone()),
        }
        reject_atomic(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[value/archer/{local}]"),
        );
        let revision = session.draft.revision;
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft.revision, revision + 1);
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let reopened = open_hsa_page(&mut cold);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
    }
}
#[test]
fn unit47_leaf_archer_correction_source_graph_save_atomically() {
    use crate::tests::cross::interview::open_hsa_page;
    for (case, key, local, mutation) in [
        (
            "leaf-archer-source-details",
            "HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED",
            "corrections",
            0,
        ),
        (
            "leaf-archer-source-gross",
            "HEALTH_CORRECTION_DOCUMENT_GROSS",
            "corrections",
            1,
        ),
        (
            "leaf-archer-source-earnings",
            "HEALTH_CORRECTION_DOCUMENT_EARNINGS",
            "corrections",
            2,
        ),
        (
            "leaf-archer-source-year",
            "HEALTH_CORRECTION_DOCUMENT_YEAR",
            "corrections/0/returned",
            3,
        ),
        (
            "leaf-archer-source-reference",
            "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE",
            "corrections/0/document",
            4,
        ),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut source_page = open_topic(&mut session, TaxSection::OtherIncome);
        documents(
            &mut source_page,
            vec![document(
                0,
                IncomeDocument::Sa1099(Sa1099 {
                    origin: HealthDocumentOrigin::Form1099Sa,
                    gross: 10_000,
                    account: MedicalSaAccount::Archer,
                    payment: MedicalSaPayment::ExcessReturned {
                        earnings: if mutation == 0 { None } else { Some(0) },
                    },
                }),
            )],
        );
        accepted(&send(&mut session, source_page));
        let source_id = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
            .unwrap()
            .id;
        let mut row = leaf_archer_owner(&session.draft);
        let mut correction = leaf_archer_correction();
        correction.document = Some(source_id);
        row.corrections.push(correction);
        let mut corrected = open_hsa_page(&mut session);
        let P::SaveHsa { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.archer = ArcherContributionsPage::Records {
            owners: vec![row],
            sharing: ArcherSharing::Equal,
        };
        let mut rejected = corrected.clone();
        let P::SaveHsa { value, .. } = &mut rejected else {
            unreachable!()
        };
        let ArcherContributionsPage::Records { owners, .. } = &mut value.archer else {
            unreachable!()
        };
        let corrections = &mut owners[0].corrections;
        match mutation {
            0 => corrections.clear(),
            1 => corrections[0].principal += 1,
            2 => {
                corrections[0].principal -= 1;
                corrections[0].earnings = 1;
            }
            3 => corrections[0].returned = 20270601,
            _ => corrections[0].document = Some(source_id + 100_000),
        }
        leaf_hsa_save_estimate_rejection(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[value/archer/owners/0/{local}]"),
        );
        leaf_hsa_correct_and_reopen(&mut session, corrected);
    }
}

fn leaf_hsa_estimate(request: &P) -> P {
    let P::SaveHsa {
        hsa_failed_deposits,
        archer_failed_deposits,
        hsa_return_documents,
        archer_return_documents,
        other_coverage_allowed,
        allowed_owners,
        allowed_family_split,
        ira_sources,
        archer_income,
        value,
        ..
    } = request
    else {
        panic!("health editor required")
    };
    P::EstimateHsa {
        user: TEST_USER,
        hsa_failed_deposits: hsa_failed_deposits.clone(),
        archer_failed_deposits: archer_failed_deposits.clone(),
        hsa_return_documents: hsa_return_documents.clone(),
        archer_return_documents: archer_return_documents.clone(),
        other_coverage_allowed: *other_coverage_allowed,
        allowed_owners: allowed_owners.clone(),
        allowed_family_split: allowed_family_split.clone(),
        ira_sources: ira_sources.clone(),
        archer_income: archer_income.clone(),
        value: value.clone(),
    }
}
fn leaf_hsa_save_estimate_rejection(
    case: &str,
    session: &mut Interview,
    rejected: P,
    key: &str,
    field: &str,
) {
    let estimated = leaf_hsa_estimate(&rejected);
    reject_atomic(case, session, rejected, key, field);
    reject_atomic(
        &alloc::format!("{case}-estimate"),
        session,
        estimated,
        key,
        field,
    );
}
fn leaf_hsa_correct_and_reopen(session: &mut Interview, corrected: P) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(session, leaf_hsa_estimate(&corrected)),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    assert_eq!(session.draft.revision, before.revision + 1);
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let reopened = crate::tests::cross::interview::open_hsa_page(&mut cold);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}
#[test]
fn unit47_leaf_hsa_funding_sources_save_and_estimate_atomically() {
    use crate::tests::cross::interview::{hsa_account, open_hsa_page};
    for (case, key, local, mutation) in [
        (
            "leaf-funding-forged-source",
            "HSA_FUNDING_SOURCE_NOT_OFFERED",
            "funding/0/source",
            0,
        ),
        (
            "leaf-funding-wrong-source-kind",
            "HSA_FUNDING_SOURCE_NOT_OFFERED",
            "funding/0/source",
            1,
        ),
        (
            "leaf-funding-distribution-year",
            "HSA_FUNDING_DISTRIBUTION_YEAR",
            "funding/0/basis/distributed",
            2,
        ),
        (
            "leaf-funding-traditional-after-receipt",
            "HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT",
            "funding/0/basis/distributed",
            3,
        ),
        (
            "leaf-funding-roth-after-receipt",
            "HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT",
            "funding/0/funded",
            4,
        ),
        (
            "leaf-funding-before-inherited-death",
            "HSA_FUNDING_BEFORE_INHERITED_DEATH",
            "funding/0/basis/distributed",
            5,
        ),
        (
            "leaf-funding-lifetime-earlier-use",
            "HSA_FUNDING_LIFETIME",
            "accounts/0/uses",
            6,
        ),
        (
            "leaf-funding-lifetime-prior-history",
            "HSA_FUNDING_LIFETIME",
            "accounts/0/funding_history",
            7,
        ),
        (
            "leaf-funding-lifetime-second-transfer",
            "HSA_FUNDING_LIFETIME",
            "funding",
            8,
        ),
        (
            "leaf-funding-needs-account",
            "HSA_FUNDING_NEEDS_AN_ACCOUNT",
            "funding/0/owner",
            9,
        ),
        (
            "leaf-funding-after-owner-death",
            "HSA_FUNDING_OWNER_DIED_BEFORE_RECEIPT",
            "funding/0/funded",
            10,
        ),
        (
            "leaf-funding-ongoing-sep",
            "HSA_FUNDING_ONGOING_SEP_SIMPLE",
            "funding/0/ongoing_sep_simple",
            11,
        ),
        (
            "leaf-funding-loss-before-receipt",
            "HSA_FUNDING_LOSS_BEFORE_FUNDING",
            "funding/0/loss/date",
            12,
        ),
        (
            "leaf-funding-no-coverage",
            "HSA_FUNDING_NEEDS_COVERAGE",
            "funding/0/coverage",
            13,
        ),
        (
            "leaf-funding-roth-order",
            "HSA_FUNDING_ORDER_CONTRADICTED",
            "funding/0/basis/before_other_activity",
            14,
        ),
        (
            "leaf-funding-wrong-source-owner",
            "HSA_FUNDING_SOURCE_NOT_OFFERED",
            "funding/0/source",
            15,
        ),
    ] {
        let mut draft = if mutation == 4 || mutation == 14 {
            roth_earlier_funding()
        } else if mutation == 5 {
            inherited_hsa_funding(6000, 0, 6000, 0, 1000)
        } else {
            let mut draft = complete(if mutation == 15 {
                FederalStatus::Joint
            } else {
                FederalStatus::Single
            });
            current_hsa_funding(&mut draft, 1000);
            draft
        };
        let mut transfer =
            draft.adjustments.as_ref().unwrap().hsa[0].funding[0].row(Owner::Taxpayer);
        draft.adjustments.as_mut().unwrap().hsa[0].funding.clear();
        if mutation == 5 {
            let group = draft
                .income
                .iter_mut()
                .find_map(|entry| match &mut entry.income {
                    FederalIncome::InheritedIra { data } => Some(data),
                    _ => None,
                })
                .unwrap();
            group.died = 20260301;
            transfer.funded = 20260402;
            let HsaFundingBasis::PriorBasis { distributed, .. } = &mut transfer.basis else {
                unreachable!()
            };
            *distributed = 20260401;
        }
        if mutation == 10 {
            draft.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
                bonds: None,
                estate_medical: 0,
                date: 20261201,
                ordinary: true,
            });
        }
        let wage = draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
            .unwrap()
            .id;
        let mut session = Interview::new(TEST_USER, draft);
        // Reopen and submit the actual source documents before composing HSA rows.
        let source_page = open_topic(&mut session, TaxSection::Retirement);
        accepted(&send(&mut session, source_page));
        let mut corrected = open_hsa_page(&mut session);
        let P::SaveHsa { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.funding = vec![transfer];
        value.family_split = FamilySplit::NotApplicable;
        let mut rejected = corrected.clone();
        let P::SaveHsa {
            value, ira_sources, ..
        } = &mut rejected
        else {
            unreachable!()
        };
        match mutation {
            0 => {
                value.funding[0].source += 100_000;
                if let Some(first) = ira_sources.first().cloned() {
                    let mut forged = first;
                    forged.id = value.funding[0].source;
                    ira_sources.push(forged);
                }
            }
            1 => value.funding[0].source = wage,
            2 | 3 | 5 => {
                let date = if mutation == 2 { 20251231 } else { 20260201 };
                match &mut value.funding[0].basis {
                    HsaFundingBasis::Pretax { distributed }
                    | HsaFundingBasis::PriorBasis { distributed, .. } => *distributed = date,
                    _ => unreachable!(),
                }
            }
            4 => value.funding[0].funded = 20260614,
            6 => value.accounts[0].uses.push(HsaFundingUseHistory {
                funded: 20240101,
                coverage: HsaCoverage::SelfOnly,
                amount: 10_000,
            }),
            7 => value.accounts[0].funding_history.push(HsaFundingHistory {
                funded: 20250101,
                amount: 10_000,
                loss: None,
            }),
            8 => {
                let second = value.funding[0].clone();
                value.funding.push(second);
            }
            9 => {
                value.accounts.clear();
                value.other_coverage = None;
            }
            10 => value.funding[0].funded = 20261202,
            11 => value.funding[0].ongoing_sep_simple = true,
            12 => {
                value.funding[0].loss = Some(HsaEligibilityLoss {
                    date: 20260114,
                    reason: HsaLossReason::Other,
                })
            }
            13 => value.funding[0].coverage = HsaCoverage::None,
            14 => {
                let HsaFundingBasis::Roth {
                    before_other_activity,
                    ..
                } = &mut value.funding[0].basis
                else {
                    unreachable!()
                };
                *before_other_activity = true;
            }
            _ => {
                value
                    .accounts
                    .push(hsa_account(Owner::Spouse, HsaCoverageYear::SelfOnly, 0));
                value.other_coverage = None;
                value.funding[0].owner = Owner::Spouse;
            }
        }
        leaf_hsa_save_estimate_rejection(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[value/{local}]"),
        );
        leaf_hsa_correct_and_reopen(&mut session, corrected);
    }
}

#[test]
fn unit47_leaf_failed_deposit_sources_save_and_estimate_atomically() {
    use crate::tests::cross::interview::{hsa_account, open_hsa_page};
    for (case, archer, mutation, key, local) in [
        (
            "leaf-hsa-failed-source",
            false,
            0,
            "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
            "failed_deposit",
        ),
        (
            "leaf-hsa-failed-source-date",
            false,
            1,
            "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
            "failed_deposit",
        ),
        (
            "leaf-hsa-failed-source-year",
            false,
            2,
            "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
            "failed_deposit",
        ),
        (
            "leaf-hsa-failed-aggregate",
            false,
            3,
            "HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT",
            "principal",
        ),
        (
            "leaf-hsa-failed-payer",
            false,
            4,
            "HEALTH_FAILED_DEPOSIT_PERSONAL_REQUIRED",
            "payer",
        ),
        (
            "leaf-archer-failed-source",
            true,
            0,
            "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
            "failed_deposit",
        ),
        (
            "leaf-archer-failed-source-date",
            true,
            1,
            "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
            "failed_deposit",
        ),
        (
            "leaf-archer-failed-source-year",
            true,
            2,
            "HEALTH_FAILED_DEPOSIT_UNAVAILABLE",
            "failed_deposit",
        ),
        (
            "leaf-archer-failed-aggregate",
            true,
            3,
            "HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT",
            "principal",
        ),
        (
            "leaf-archer-failed-payer",
            true,
            4,
            "HEALTH_FAILED_DEPOSIT_PERSONAL_REQUIRED",
            "payer",
        ),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let destination = if archer {
            MedicalRolloverDestination::Archer
        } else {
            MedicalRolloverDestination::Hsa
        };
        let mut source_page = open_topic(&mut session, TaxSection::OtherIncome);
        documents(
            &mut source_page,
            vec![document(
                0,
                IncomeDocument::Sa1099(Sa1099 {
                    origin: HealthDocumentOrigin::Form1099Sa,
                    gross: 10_000,
                    account: if archer {
                        MedicalSaAccount::Archer
                    } else {
                        MedicalSaAccount::Hsa
                    },
                    payment: MedicalSaPayment::Own {
                        nonmedical: vec![],
                        medical: 0,
                        disabled_since: None,
                        rollover: Some(MedicalRollover {
                            preceding: None,
                            receipts: vec![MedicalRolloverReceipt {
                                received: 20260101,
                                deposits: vec![MedicalRolloverDeposit {
                                    deposited: 20260315,
                                    amount: 10_000,
                                    destination,
                                    character: MedicalDepositCharacter::Cash {
                                        year: 2026,
                                        deadline: 20270415,
                                    },
                                }],
                            }],
                        }),
                    },
                }),
            )],
        );
        accepted(&send(&mut session, source_page));
        let source_id = session
            .draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
            .unwrap()
            .id;
        let mut corrected = open_hsa_page(&mut session);
        let mut archer_owner = leaf_archer_owner(&session.draft);
        let P::SaveHsa { value, .. } = &mut corrected else {
            unreachable!()
        };
        if archer {
            archer_owner.personal = 0;
            let mut correction = leaf_archer_correction();
            correction.failed_deposit = Some(source_id);
            correction.contributed_on = 20260315;
            correction.principal = 4_000;
            archer_owner.corrections = vec![correction.clone(), correction];
            value.archer = ArcherContributionsPage::Records {
                owners: vec![archer_owner],
                sharing: ArcherSharing::Equal,
            };
        } else {
            let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0);
            let correction = HsaCorrection {
                document: None,
                failed_deposit: Some(source_id),
                year: 2026,
                contributed_on: 20260315,
                principal: 4_000,
                earnings: 0,
                returned: 20260601,
                deadline: 20261015,
                payer: HsaCorrectionPayer::Personal,
            }
            .page();
            account.corrections = vec![correction.clone(), correction];
            value.accounts = vec![account];
            value.family_split = FamilySplit::NotApplicable;
        }
        let mut rejected = corrected.clone();
        let P::SaveHsa {
            value,
            hsa_failed_deposits,
            archer_failed_deposits,
            ..
        } = &mut rejected
        else {
            unreachable!()
        };
        if archer {
            let ArcherContributionsPage::Records { owners, .. } = &mut value.archer else {
                unreachable!()
            };
            let corrections = &mut owners[0].corrections;
            match mutation {
                0 => {
                    corrections[0].failed_deposit = Some(source_id + 100_000);
                    if let Some(first) = archer_failed_deposits.first().cloned() {
                        let mut forged = first;
                        forged.id = source_id + 100_000;
                        archer_failed_deposits.push(forged);
                    }
                }
                1 => corrections[0].contributed_on = 20260316,
                2 => corrections[0].year = 2025,
                3 => corrections[1].principal = 6_001,
                _ => corrections[0].payer = ArcherCorrectionPayer::Employer { in_wages: false },
            }
        } else {
            let corrections = &mut value.accounts[0].corrections;
            match mutation {
                0 => {
                    corrections[0].failed_deposit = Some(source_id + 100_000);
                    if let Some(first) = hsa_failed_deposits.first().cloned() {
                        let mut forged = first;
                        forged.id = source_id + 100_000;
                        hsa_failed_deposits.push(forged);
                    }
                }
                1 => corrections[0].contributed_on = 20260316,
                2 => corrections[0].year = 2025,
                3 => corrections[1].principal = 6_001,
                _ => corrections[0].payer = HsaCorrectionPayer::Employer { in_wages: false },
            }
        }
        let group = if archer {
            "archer/owners/0"
        } else {
            "accounts/0"
        };
        // The aggregate owner returns the first correction in the failed
        // source group; each row independently remains below the deposit.
        leaf_hsa_save_estimate_rejection(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[value/{group}/corrections/0/{local}]"),
        );
        leaf_hsa_correct_and_reopen(&mut session, corrected);
    }
}

#[test]
fn unit47_leaf_rental_at_risk_decreases_save_atomically() {
    let mut good = crate::tests::cross::interview::rental_page(1_200_000);
    good.more.at_risk = simplified_rental_risk(100_000, 50_000, 150_000, [0, 0, 0, 0]);
    let mut bad = good.clone();
    let Some(RentalAtRisk::Limited { record, .. }) = &mut bad.more.at_risk else {
        unreachable!()
    };
    let crate::RiskMethod::Simplified { facts } = &mut record.as_mut().unwrap().regular else {
        unreachable!()
    };
    facts.decreases += 1;
    document_case(
        "leaf-rental-at-risk-decreases",
        TaxSection::Rentals,
        IncomeDocument::Rental(good.clone()),
        IncomeDocument::Rental(bad),
        "RISK_RECAPTURE_DETAILED_REQUIRED",
        // 2026-10-16 approved signed/Detailed owner fit: this same negative
        // Simplified balance requires Detailed facts, preserving refusal,
        // atomicity, corrected Save and reopen at the actual method field.
        "more/at_risk/record/regular",
    );
    let mut missing = good.clone();
    missing.more.at_risk = rental_risk_without_method();
    document_case(
        "leaf-rental-required-risk-method",
        TaxSection::Rentals,
        IncomeDocument::Rental(good),
        IncomeDocument::Rental(missing),
        "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
        "more/at_risk/record",
    );
}

fn leaf_mortgage_editor(session: &mut Interview) -> P {
    match crate::tests::cross::interview::from_review(session, TaxSection::Mortgage) {
        Reply::Edit(page @ P::SaveMortgages { .. }) => page,
        other => panic!("mortgage editor required: {other:?}"),
    }
}
fn leaf_mortgage_estimate(request: &P) -> P {
    let P::SaveMortgages {
        mortgage_loans,
        rental_mortgage_loans,
        value,
        ..
    } = request
    else {
        panic!("mortgage editor required")
    };
    P::EstimateMortgages {
        user: TEST_USER,
        mortgage_loans: mortgage_loans.clone(),
        rental_mortgage_loans: rental_mortgage_loans.clone(),
        value: value.clone(),
    }
}
#[test]
fn unit47_leaf_rental_mortgage_root_save_and_estimate_atomically() {
    for (case, replace) in [
        ("leaf-mortgage-rental-delete", false),
        ("leaf-mortgage-rental-replace", true),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut source_page = leaf_mortgage_editor(&mut session);
        let P::SaveMortgages { value, .. } = &mut source_page else {
            unreachable!()
        };
        value.loans = vec![Mortgage1098 {
            id: 0,
            interest: 600_000,
            principal: 10_000_000,
            originated: 20200601,
            refund: 0,
            insurance: 0,
            points: Points::None,
            more: MortgageMore {
                property: "Linked first lender".into(),
                details: MortgageDetails::FirstLender(MortgageFirstLender {
                    home: MortgageHome::Main,
                    purpose: MortgageUse::Home,
                    average: AverageBalance::WithinLimit,
                }),
                ..Default::default()
            },
        }];
        accepted(&send(&mut session, source_page));
        let source_page = leaf_mortgage_editor(&mut session);
        let P::SaveMortgages { value, .. } = source_page else {
            unreachable!()
        };
        let root = value.loans[0].clone();
        assert!(root.id > 0);
        // The rental picks a real accepted first lender, after its normal Save.
        let mut rental = crate::tests::cross::interview::rental_page(1_200_000);
        rental.personal_use = PersonalUse::Yes {
            fair_rental_days: 60,
            personal_days: 60,
            overlap_days: 0,
            direct_expenses: 0,
        };
        rental.mortgage_loans = vec![root.id];
        let mut rental_page = open_topic(&mut session, TaxSection::Rentals);
        documents(
            &mut rental_page,
            vec![document(0, IncomeDocument::Rental(rental))],
        );
        accepted(&send(&mut session, rental_page));
        assert_eq!(
            crate::extended_rules::rental_mortgage_roots(&session.draft),
            vec![root.id]
        );
        let mut corrected = leaf_mortgage_editor(&mut session);
        let P::SaveMortgages {
            value,
            rental_mortgage_loans,
            ..
        } = &mut corrected
        else {
            unreachable!()
        };
        assert_eq!(rental_mortgage_loans, &vec![root.id]);
        assert_eq!(value.loans[0].id, root.id);
        value.loans[0].more.property = "Linked first lender corrected".into();
        let mut rejected = corrected.clone();
        let P::SaveMortgages {
            value,
            rental_mortgage_loans,
            ..
        } = &mut rejected
        else {
            unreachable!()
        };
        // An echoed empty requirements list cannot erase the accepted rental link.
        rental_mortgage_loans.clear();
        if replace {
            let mut replacement = root.clone();
            replacement.id = 0;
            replacement.more.property = "Different first lender".into();
            value.loans = vec![replacement];
        } else {
            value.loans.clear();
        }
        let estimated = leaf_mortgage_estimate(&rejected);
        reject_atomic(
            case,
            &mut session,
            rejected,
            "MORTGAGE_RENTAL_ROOT_NEEDED",
            "[value/loans]",
        );
        reject_atomic(
            &alloc::format!("{case}-estimate"),
            &mut session,
            estimated,
            "MORTGAGE_RENTAL_ROOT_NEEDED",
            "[value/loans]",
        );
        let before = session.draft.clone();
        let dirty = session.dirty;
        assert!(matches!(
            send(&mut session, leaf_mortgage_estimate(&corrected)),
            Reply::Response(P::RunningEstimate { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft.revision, before.revision + 1);
        assert_eq!(session.draft.next_id, before.next_id);
        assert_eq!(
            crate::extended_rules::rental_mortgage_roots(&session.draft),
            vec![root.id]
        );
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let reopened = leaf_mortgage_editor(&mut cold);
        let P::SaveMortgages { value, .. } = &reopened else {
            unreachable!()
        };
        assert_eq!(value.loans[0].id, root.id);
        assert_eq!(
            value.loans[0].more.property,
            "Linked first lender corrected"
        );
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        assert_eq!(
            crate::extended_rules::rental_mortgage_roots(&cold.draft),
            vec![root.id]
        );
    }
}

#[test]
fn unit47_leaf_rental_building_and_personal_days_save_atomically() {
    for (case, key, local, mutation) in [
        (
            "leaf-rental-building-land",
            "RENTAL_LAND_ABOVE_COST",
            "building/land",
            0,
        ),
        (
            "leaf-rental-building-reduction",
            "RENTAL_CREDIT_REDUCTION_ABOVE_BASIS",
            "building/credit_reduction",
            1,
        ),
        (
            "leaf-rental-improvement-reduction",
            "RENTAL_CREDIT_REDUCTION_ABOVE_BASIS",
            "more/improvements/1/credit_reduction",
            2,
        ),
        (
            "leaf-rental-personal-days",
            "RENTAL_DAYS_ABOVE_365",
            "personal_use/personal_days",
            3,
        ),
        (
            "leaf-rental-overlap-days",
            "RENTAL_OVERLAP_DAYS_ABOVE_RENTAL_DAYS",
            "personal_use/overlap_days",
            4,
        ),
    ] {
        let mut good = crate::tests::cross::interview::rental_page(1_200_000);
        match mutation {
            1 => good.building.credit_reduction = good.building.cost - good.building.land,
            2 => {
                good.more.improvements = vec![
                    Improvement {
                        commercial_1993_transition: None,
                        allowed_commercial_1993_transition: Vec::new(),
                        casualty: None,
        id: 0,
        energy: None,
                        sale_history: None,
                        cost: 500_000,
                        credit_reduction: 0,
                        in_service: 20220101,
                        rehabilitation: None,
                        partial: None,
                    },
                    Improvement {
                        commercial_1993_transition: None,
                        allowed_commercial_1993_transition: Vec::new(),
                        casualty: None,
        id: 0,
        energy: None,
                        sale_history: None,
                        cost: 1_000_000,
                        credit_reduction: 1_000_000,
                        in_service: 20230101,
                        rehabilitation: None,
                        partial: None,
                    },
                ]
            }
            3 => {
                good.personal_use = PersonalUse::Yes {
                    fair_rental_days: 200,
                    personal_days: 165,
                    overlap_days: 0,
                    direct_expenses: 0,
                }
            }
            4 => {
                good.personal_use = PersonalUse::Yes {
                    fair_rental_days: 200,
                    personal_days: 100,
                    overlap_days: 200,
                    direct_expenses: 0,
                }
            }
            _ => {}
        }
        let mut bad = good.clone();
        match mutation {
            0 => bad.building.land = bad.building.cost + 1,
            1 => bad.building.credit_reduction += 1,
            2 => bad.more.improvements[1].credit_reduction += 1,
            3 => {
                let PersonalUse::Yes { personal_days, .. } = &mut bad.personal_use else {
                    unreachable!()
                };
                *personal_days += 1;
            }
            _ => {
                let PersonalUse::Yes { overlap_days, .. } = &mut bad.personal_use else {
                    unreachable!()
                };
                *overlap_days += 1;
            }
        }
        document_case(
            case,
            TaxSection::Rentals,
            IncomeDocument::Rental(good),
            IncomeDocument::Rental(bad),
            key,
            local,
        );
    }
}

fn leaf_secondary_editor(session: &mut Interview, section: TaxSection) -> P {
    match crate::tests::support::client::native_editor(
        crate::tests::cross::interview::from_review(session, section),
        &[],
    ) {
        Reply::Edit(page) => page,
        other => panic!("{section:?} editor required: {other:?}"),
    }
}
fn leaf_secondary_estimate(request: &P) -> P {
    match request {
        P::SaveMoreAdjustments {
            value,
            allowed_owners,
            ..
        } => P::EstimateMoreAdjustments {
            user: TEST_USER,
            value: value.clone(),
            allowed_owners: allowed_owners.clone(),
        },
        P::SaveParentTax { value, .. } => {
            P::EstimateParentTax { user: TEST_USER, value: value.clone() }
        }
        P::SaveInjuredSpouse { value, .. } => {
            P::EstimateInjuredSpouse { user: TEST_USER, value: value.clone() }
        }
        P::SaveCare {
            value,
            allowed_injured_source_facts,
            people,
            allowed_owners,
            care_filers,
            care_months,
            businesses,
            farms,
            partnerships,
            ..
        } => P::EstimateCare {
            user: TEST_USER,
            value: value.clone(),
            allowed_injured_source_facts: allowed_injured_source_facts.clone(),
            people: people.clone(),
            allowed_owners: allowed_owners.clone(),
            care_filers: care_filers.clone(),
            care_months: care_months.clone(),
            businesses: businesses.clone(),
            farms: farms.clone(),
            partnerships: partnerships.clone(),
        },
        _ => panic!("secondary editor required"),
    }
}
fn leaf_secondary_case(
    case: &str,
    session: &mut Interview,
    corrected: P,
    rejected: P,
    section: TaxSection,
    key: &str,
    field: &str,
) {
    let estimated = leaf_secondary_estimate(&rejected);
    reject_atomic(case, session, rejected, key, field);
    reject_atomic(
        &alloc::format!("{case}-estimate"),
        session,
        estimated,
        key,
        field,
    );
    leaf_secondary_accept_and_reopen(session, corrected, section);
}
fn leaf_secondary_accept_and_reopen(session: &mut Interview, corrected: P, section: TaxSection) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(session, leaf_secondary_estimate(&corrected)),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    assert_eq!(session.draft.revision, before.revision + 1);
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let reopened = leaf_secondary_editor(&mut cold, section);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}

#[test]
fn unit47_leaf_care_prior_year_save_and_estimate_atomically() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut corrected = leaf_secondary_editor(&mut session, TaxSection::Care);
    let P::SaveCare { value, .. } = &mut corrected else {
        unreachable!()
    };
    value.prior_year = Some(PriorYearCareInput {
        two_or_more: false,
        prior_line_3: 100_000,
        paid_in_return_year: 10_000,
        prior_benefits: 0,
        prior_earned_smaller: 100_000,
        prior_line_6: 100_000,
        prior_agi: 6_000_000,
        injured_claimants: None,
    });
    let mut rejected = corrected.clone();
    let P::SaveCare { value, .. } = &mut rejected else {
        unreachable!()
    };
    value.prior_year.as_mut().unwrap().prior_line_6 += 1;
    leaf_secondary_case(
        "leaf-care-prior-line6",
        &mut session,
        corrected,
        rejected,
        TaxSection::Care,
        "PRIOR_CARE_LINE_6_ABOVE_LINE_3",
        "[value/prior_year/prior_line_6]",
    );
}
#[test]
fn unit47_leaf_move_actual_adjustment_index_save_and_estimate_atomically() {
    for (case, key, local, empty) in [
        ("leaf-move-empty", "MOVING_ENTER_AN_EXPENSE", "goods", true),
        (
            "leaf-move-arrival",
            "MOVING_ARRIVAL_OUTSIDE_CURRENT_YEAR",
            "trip/arrived",
            false,
        ),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut corrected = leaf_secondary_editor(&mut session, TaxSection::MoreAdjustments);
        let P::SaveMoreAdjustments { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.adjustments = vec![
            MoreAdjustment::OtherAdjustments,
            MoreAdjustment::MovingExpenses(MovingItem {
                owner: Owner::Taxpayer,
                record: MilitaryMove {
                    reason: MoveReason::ArmedForcesOrders,
                    goods: 10_000,
                    trip: if empty {
                        None
                    } else {
                        Some(MoveTrip {
                            paid: 1_000,
                            miles: 0,
                            arrived: 20260601,
                        })
                    },
                    reimbursed: 0,
                },
            }),
        ];
        let mut rejected = corrected.clone();
        let P::SaveMoreAdjustments { value, .. } = &mut rejected else {
            unreachable!()
        };
        let MoreAdjustment::MovingExpenses(item) = &mut value.adjustments[1] else {
            unreachable!()
        };
        if empty {
            item.record.goods = 0;
        } else {
            item.record.trip.as_mut().unwrap().arrived = 20270101;
        }
        leaf_secondary_case(
            case,
            &mut session,
            corrected,
            rejected,
            TaxSection::MoreAdjustments,
            key,
            &alloc::format!("[value/adjustments/1/record/{local}]"),
        );
    }
}
#[test]
fn unit47_leaf_parent_child_relationships_save_and_estimate_atomically() {
    for (case, key, local, mutation) in [
        (
            "leaf-parent-child-twice",
            "PARENT_CHILD_TWICE",
            "others/1/label",
            0,
        ),
        (
            "leaf-parent-child-net",
            "PARENT_CHILD_GAINS_ABOVE_NET",
            "others/0/gains",
            1,
        ),
        (
            "leaf-parent-child-source",
            "PARENT_CHILD_GAIN_ABOVE_SOURCE",
            "others/0/gains/capital",
            2,
        ),
    ] {
        let mut session = Interview::new(TEST_USER, young_interest_return());
        let mut corrected = leaf_secondary_editor(&mut session, TaxSection::ParentTax);
        let P::SaveParentTax { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.others = vec![OtherChildTax {
            label: "Synthetic sibling".into(),
            net: 100_000,
            gains: Some(ChildGains {
                qualified: 10_000,
                capital: 20_000,
                investment_election: 0,
                special: Some(ChildSpecialGains {
                    net_capital: 20_000,
                    collectibles: 0,
                    unrecaptured: 0,
                }),
            }),
        }];
        let mut rejected = corrected.clone();
        let P::SaveParentTax { value, .. } = &mut rejected else {
            unreachable!()
        };
        match mutation {
            0 => value.others.push(value.others[0].clone()),
            1 => value.others[0].gains.as_mut().unwrap().qualified = 80_001,
            _ => value.others[0].gains.as_mut().unwrap().capital = 20_001,
        }
        leaf_secondary_case(
            case,
            &mut session,
            corrected,
            rejected,
            TaxSection::ParentTax,
            key,
            &alloc::format!("[value/{local}]"),
        );
    }
}
#[test]
fn unit47_leaf_injured_spouse_direct_facts_save_and_estimate_atomically() {
    for (case, key, field, community) in [
        (
            "leaf-injured-estimated",
            "INJURED_SPOUSE_ESTIMATED_ABOVE_PAYMENTS",
            "[value/estimated/injured]",
            false,
        ),
        (
            "leaf-injured-community-facts",
            "INJURED_SPOUSE_COMMUNITY_FACTS_NOT_CURRENT",
            "[value/community/states]",
            true,
        ),
    ] {
        let mut draft = complete(FederalStatus::Joint);
        draft.checklist.whole_return = WholeReturnTopics::INJURED_SPOUSE.0;
        draft.payments.as_mut().unwrap().estimated = Some(100_000);
        draft.payments.as_mut().unwrap().estimated_payments = vec![DatedPayment {
            date: 20260415,
            amount: 100_000,
            payer: PaymentPayer::Joint,
            refund_property: None,
        }];
        let mut session = Interview::new(TEST_USER, draft);
        let mut corrected = leaf_secondary_editor(&mut session, TaxSection::InjuredSpouse);
        let P::SaveInjuredSpouse { value, .. } = &mut corrected else {
            unreachable!()
        };
        value.estimated = EstimatedSplit::Agreed { injured: 100_000 };
        let mut rejected = corrected.clone();
        let P::SaveInjuredSpouse { value, .. } = &mut rejected else {
            unreachable!()
        };
        if community {
            value.community = InjuredCommunity::Yes {
                states: vec![],
                debt_facts: InjuredDebtFacts::NotApplicable,
            };
        } else {
            value.estimated = EstimatedSplit::Agreed { injured: 100_001 };
            value.joint_estimated = Some(100_001);
        }
        leaf_secondary_case(
            case,
            &mut session,
            corrected,
            rejected,
            TaxSection::InjuredSpouse,
            key,
            field,
        );
    }
}

#[test]
fn unit47_leaf_injured_bounds_recompute_sources_save_and_estimate_atomically() {
    for (case, maximum) in [
        ("leaf-injured-allocation-minimum", false),
        ("leaf-injured-allocation-maximum", true),
    ] {
        // Existing mixed child-pool fixture: two real wage sources and two
        // qualifying children, with one separate-return claimant per spouse.
        let mut draft = complete(FederalStatus::Joint);
        draft.income = vec![
            wage(1, Owner::Taxpayer, 5_000_000, 800_000),
            wage(2, Owner::Spouse, 2_000_000, 100_000),
        ];
        draft.dependents = vec![child(3), child(4)];
        draft.next_id = 5;
        draft.checklist.whole_return = WholeReturnTopics::INJURED_SPOUSE.0;
        let mut session = Interview::new(TEST_USER, draft);
        let mut initial = leaf_secondary_editor(&mut session, TaxSection::InjuredSpouse);
        let P::SaveInjuredSpouse { value, .. } = &mut initial else {
            unreachable!()
        };
        value.claimants[1].claimant = Owner::Spouse;
        let before = session.draft.clone();
        let dirty = session.dirty;
        let refreshed = send(&mut session, initial);
        let Reply::Response(P::BeginInjuredSpouse { value, .. }) = &refreshed else {
            panic!("changed claimant must refresh authoritative component rows")
        };
        assert_eq!(value.new_allocations.len(), 2);
        let context = value
            .new_allocations
            .iter()
            .find(|row| row.component == "ctc_odc_pool")
            .unwrap();
        // This boundary control deliberately submits the endpoint and its
        // companion. These are explicit native test inputs, never UI defaults.
        let chosen = if maximum {
            context.maximum
        } else {
            context.minimum
        };
        let companion = value.child_credit_cap_injured - value.known_child_credit_injured - chosen;
        let Reply::Edit(mut corrected @ P::SaveInjuredSpouse { .. }) =
            crate::tests::support::client::native_editor(
                refreshed,
                &[("ctc_odc_pool", chosen), ("actc_pool", companion)],
            )
        else {
            unreachable!()
        };
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        let P::SaveInjuredSpouse { value, .. } = &mut corrected else {
            unreachable!()
        };
        assert_eq!(value.allocations.len(), 2);
        let target = value
            .allocations
            .iter()
            .position(|row| row.component == "ctc_odc_pool")
            .unwrap();
        let other = value
            .allocations
            .iter()
            .position(|row| row.component == "actc_pool")
            .unwrap();
        let minimum = value.allocations[target].minimum;
        let maximum_value = value.allocations[target].maximum;
        assert!(minimum < maximum_value);
        let endpoint = if maximum { maximum_value } else { minimum };
        value.allocations[target].injured = endpoint;
        // Select the companion pool within its offered bounds so the native
        // combined child-credit cap remains satisfied at either endpoint.
        let companion =
            value.child_credit_cap_injured - value.known_child_credit_injured - endpoint;
        assert!(
            (value.allocations[other].minimum..=value.allocations[other].maximum)
                .contains(&companion)
        );
        value.allocations[other].injured = companion;
        let mut rejected = corrected.clone();
        let P::SaveInjuredSpouse { value, .. } = &mut rejected else {
            unreachable!()
        };
        value.allocations[target].injured = if maximum { endpoint + 1 } else { endpoint - 1 };
        let field = alloc::format!("[value/allocations/{target}/injured]");
        let estimated = leaf_secondary_estimate(&rejected);
        reject_atomic(
            case,
            &mut session,
            rejected.clone(),
            "INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT",
            &field,
        );
        reject_atomic(
            &alloc::format!("{case}-estimate"),
            &mut session,
            estimated,
            "INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT",
            &field,
        );
        let mut forged = rejected;
        let P::SaveInjuredSpouse { value, .. } = &mut forged else {
            unreachable!()
        };
        if maximum {
            value.allocations[target].maximum += 1;
        } else {
            value.allocations[target].minimum -= 1;
        }
        reject_atomic(
            &alloc::format!("{case}-forged-estimate"),
            &mut session,
            leaf_secondary_estimate(&forged),
            "INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT",
            &field,
        );
        let before = session.draft.clone();
        let dirty = session.dirty;
        let Reply::Response(P::BeginInjuredSpouse {
            value: refreshed, ..
        }) = send(&mut session, forged)
        else {
            panic!("forged echoed bounds must refresh, never commit")
        };
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        let row = refreshed
            .new_allocations
            .iter()
            .find(|row| row.component == "ctc_odc_pool")
            .unwrap();
        assert_eq!((row.minimum, row.maximum), (minimum, maximum_value));
        leaf_secondary_accept_and_reopen(&mut session, corrected, TaxSection::InjuredSpouse);
    }
}

#[test]
fn unit47_leaf_foreign_accounts_keep_guard_order_and_actual_country_indices() {
    // 2026-10-15 D-13 removes the former 250-country cap-only case.
    // Actual duplicate and unknown countries still name their submitted row.
    for (case, key, local, mutation) in [
        ("leaf-fbar-answer", "FBAR_ANSWER_NEEDED", "fbar_required", 0),
        (
            "leaf-fbar-countries",
            "FBAR_COUNTRIES_NEEDED",
            "fbar_countries",
            1,
        ),
        (
            "leaf-fbar-country-duplicate",
            "FBAR_COUNTRY_TWICE",
            "fbar_countries/1",
            3,
        ),
        (
            "leaf-fbar-country-unknown",
            "FBAR_COUNTRY_NOT_LISTED",
            "fbar_countries/1",
            4,
        ),
    ] {
        let good = ForeignAccountOrTrust {
            account: true,
            fbar_required: Some(true),
            fbar_countries: vec![0x4341],
            trust: false,
            highest_value: Some(4_000_000),
        };
        let mut bad = good.clone();
        match mutation {
            0 => bad.fbar_required = None,
            1 => bad.fbar_countries.clear(),
            3 => bad.fbar_countries.push(0x4341),
            _ => bad.fbar_countries.push(0),
        }
        document_case(
            case,
            TaxSection::InterestDividends,
            IncomeDocument::ForeignAccountOrTrust(good),
            IncomeDocument::ForeignAccountOrTrust(bad),
            key,
            local,
        );
    }
}

#[test]
fn unit47_leaf_social_year_metadata_preserves_duplicate_first_order() {
    for (case, key, local, mutation) in [
        (
            "leaf-social-year-duplicate",
            "LUMP_SUM_YEAR_TWICE",
            "years/1/year",
            0,
        ),
        (
            "leaf-social-taxable",
            "LUMP_SUM_TAXABLE_ABOVE_BENEFITS",
            "years/1/taxable",
            1,
        ),
        (
            "leaf-social-refigured",
            "LUMP_SUM_REFIGURED_ABOVE_BENEFITS",
            "years/1/refigured",
            2,
        ),
    ] {
        let year = SocialBenefitYear {
            year: 2023,
            status: FederalStatus::Single,
            apart: false,
            received: 400_000,
            benefits: 500_000,
            earlier_lumps: 100_000,
            agi: 1_000_000,
            taxable: 500_000,
            refigured: 600_000,
            exempt: 0,
            addbacks: 0,
        };
        let mut second = year.clone();
        second.year = 2024;
        let good = LumpSumElection {
            years: vec![year, second],
            complete: true,
        };
        let mut bad = good.clone();
        match mutation {
            0 => {
                bad.years[1].year = bad.years[0].year;
                // A row-level failure in the earlier row must not displace
                // the collection owner's existing duplicate-year priority.
                bad.years[0].taxable += 1;
            }
            1 => bad.years[1].taxable += 1,
            _ => bad.years[1].refigured += 1,
        }
        let mut draft = complete(FederalStatus::Single);
        draft.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
        draft.next_id = 2;
        let mut session = Interview::new(TEST_USER, draft);
        let statement = IncomeDocument::Ssa1099(Ssa1099 {
            statement: Default::default(),
            benefits: 2_000_000,
            withholding: 0,
        });
        let mut corrected = open_topic(&mut session, TaxSection::Retirement);
        documents(
            &mut corrected,
            vec![
                document(0, statement.clone()),
                document(0, IncomeDocument::SocialLumpSums(good)),
            ],
        );
        let mut rejected = corrected.clone();
        documents(
            &mut rejected,
            vec![
                document(0, statement),
                document(0, IncomeDocument::SocialLumpSums(bad)),
            ],
        );
        reject_atomic(
            case,
            &mut session,
            rejected,
            key,
            &alloc::format!("[documents/documents/1/document/{local}]"),
        );
        let revision = session.draft.revision;
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft.revision, revision + 1);
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let reopened = open_topic(&mut cold, TaxSection::Retirement);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
    }
}

fn leaf_employee_estimate(request: &P) -> P {
    let P::SaveEmployeeExpenses {
        allowed_owners,
        value,
        ..
    } = request
    else {
        panic!("Form 2106 editor required")
    };
    P::EstimateEmployeeExpenses {
        user: TEST_USER,
        allowed_owners: allowed_owners.clone(),
        value: value.clone(),
    }
}
fn leaf_employee_mileage() -> Vehicle {
    Vehicle {
        method: VehicleMethod::StandardMileage {
            id: 0,
            first_half: 20,
            second_half: 30,
            total: 100,
            more: MileageMore::default(),
        },
    }
}
fn leaf_employee_actual() -> Vehicle {
    Vehicle {
        method: VehicleMethod::Actual {
            id: 0,
            history_details: None,
            amt_section179_income_carry: None,
            allowed_amt_carry: Vec::new(),
            cost: 1_000_000,
            credit_reduction: 0,
            acquired: None,
            placed: 20260101,
            total_miles: 100,
            business_miles: 100,
            operating_costs: 0,
            loan_interest: 0,
            passenger_automobile: true,
            earlier_business_percent: None,
            history: AssetHistory::New {
                first_year: FirstYearDeduction::Regular,
            },
            disposal: AssetDisposal::No,
        },
    }
}
fn leaf_employee_lease() -> BusinessLeasedVehicle {
    BusinessLeasedVehicle {
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
    }
}
fn leaf_employee_editor(vehicle: Vehicle) -> (Interview, P) {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![
        wage(1, Owner::Taxpayer, 3_000_000, 300_000),
        wage(2, Owner::Taxpayer, 2_000_000, 200_000),
    ];
    draft.next_id = 3;
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = leaf_secondary_editor(&mut session, TaxSection::EmployeeExpenses);
    let P::SaveEmployeeExpenses { value, .. } = &mut page else {
        unreachable!()
    };
    assert_eq!(
        value.wages.iter().map(|w| w.id).collect::<Vec<_>>(),
        vec![1, 2]
    );
    let form = |employer, vehicles| EmployeeExpenseForm {
        dot_hours_of_service: false,
        owner: Owner::Taxpayer,
        occupation: "Synthetic fee-basis official".into(),
        capacity: EmployeeCapacity::FeeBasisOfficial,
        employers: vec![employer],
        local_transportation: 0,
        travel: 10_000,
        other: 0,
        meals: 0,
        reimbursements: 0,
        vehicles,
    };
    value.forms = vec![
        form(1, vec![]),
        form(2, vec![leaf_employee_mileage(), vehicle]),
    ];
    (session, page)
}
fn leaf_employee_case(case: &str, good: Vehicle, bad: Vehicle, key: &str, local: &str) {
    let new_actual_asset = matches!(&good.method, VehicleMethod::Actual { .. });
    let (mut session, corrected) = leaf_employee_editor(good);
    let mut rejected = corrected.clone();
    let P::SaveEmployeeExpenses { value, .. } = &mut rejected else {
        unreachable!()
    };
    value.forms[1].vehicles[1] = bad;
    let field = alloc::format!("[value/forms/1/vehicles/1/method/{local}]");
    reject_atomic(case, &mut session, rejected.clone(), key, &field);
    reject_atomic(
        &alloc::format!("{case}-estimate"),
        &mut session,
        leaf_employee_estimate(&rejected),
        key,
        &field,
    );
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(&mut session, leaf_employee_estimate(&corrected)),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, corrected));
    assert_eq!(session.draft.revision, before.revision + 1);
    let actual_ids: Vec<_> = session
        .draft
        .adjustments
        .as_ref()
        .unwrap()
        .employee_expenses
        .iter()
        .flat_map(|claim| claim.vehicle_assets.iter().map(|asset| asset.id))
        .collect();
    if new_actual_asset {
        assert!(before.next_id > 0);
        assert_eq!(actual_ids, vec![before.next_id]);
        assert_eq!(session.draft.next_id, before.next_id + 1);
    } else {
        assert!(actual_ids.is_empty());
        assert_eq!(session.draft.next_id, before.next_id);
    }
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let reopened = leaf_secondary_editor(&mut cold, TaxSection::EmployeeExpenses);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}

#[test]
fn unit47_leaf_employee_actual_mileage_save_and_estimate_atomically() {
    let good = leaf_employee_actual();
    let mut bad = good.clone();
    let VehicleMethod::Actual { business_miles, .. } = &mut bad.method else {
        unreachable!()
    };
    *business_miles += 1;
    leaf_employee_case(
        "leaf-employee-actual-miles",
        good,
        bad,
        "VEHICLE_BUSINESS_MILES_ABOVE_TOTAL",
        "business_miles",
    );
}

#[test]
fn unit47_leaf_employee_leased_boundaries_save_and_estimate_atomically() {
    for (case, key, local, mutation) in [
        (
            "leaf-employee-lease-business-miles",
            "VEHICLE_BUSINESS_MILES_ABOVE_TOTAL",
            "business_miles",
            0,
        ),
        (
            "leaf-employee-lease-commuting",
            "VEHICLE_BUSINESS_MILES_ABOVE_TOTAL",
            "commuting",
            1,
        ),
        ("leaf-employee-lease-2018", "LEASE_BEFORE_2018", "began", 2),
        (
            "leaf-employee-lease-began-upper",
            "LEASE_ENDS_BEFORE_BEGAN",
            "began",
            3,
        ),
        (
            "leaf-employee-lease-ended-before",
            "LEASE_ENDS_BEFORE_BEGAN",
            "ends",
            4,
        ),
        (
            "leaf-employee-lease-end-lower",
            "LEASE_ENDS_BEFORE_BEGAN",
            "ends",
            5,
        ),
        (
            "leaf-employee-lease-end-upper",
            "LEASE_ENDS_BEFORE_BEGAN",
            "ends",
            6,
        ),
        (
            "leaf-employee-lease-term",
            "LEASE_TERM_UNDER_30_DAYS",
            "ends",
            7,
        ),
    ] {
        let mut good = leaf_employee_lease();
        match mutation {
            0 => {
                good.business_miles = good.total_miles;
                good.commuting = 0;
            }
            2 => {
                good.began = 20180101;
                good.ends = None;
            }
            3 => {
                good.began = 20261231;
                good.ends = None;
            }
            5 => {
                good.began = 20251201;
                good.ends = Some(20260101);
            }
            6 => {
                good.began = 20261201;
                good.ends = Some(20261231);
            }
            _ => {}
        }
        let mut bad = good.clone();
        match mutation {
            0 => bad.business_miles += 1,
            1 => bad.commuting += 1,
            2 => bad.began = 20171231,
            3 => bad.began = 20270101,
            4 => bad.ends = Some(bad.began),
            5 => bad.ends = Some(20251231),
            6 => bad.ends = Some(20270101),
            _ => bad.ends = Some(20260129),
        }
        leaf_employee_case(
            case,
            Vehicle {
                method: VehicleMethod::Leased { vehicle: good },
            },
            Vehicle {
                method: VehicleMethod::Leased { vehicle: bad },
            },
            key,
            &alloc::format!("vehicle/{local}"),
        );
    }
}

#[test]
fn unit47_leaf_employee_earlier_policy_guards_keep_their_priority() {
    for (key, mutation) in [
        ("EMPLOYEE_EXPENSES_EMPLOYER_NEEDED", 0),
        ("RESERVIST_VEHICLE_STANDARD_MILEAGE", 1),
        ("EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX", 2),
    ] {
        let (mut session, mut rejected) = leaf_employee_editor(leaf_employee_actual());
        let P::SaveEmployeeExpenses { value, .. } = &mut rejected else {
            unreachable!()
        };
        let form = &mut value.forms[1];
        let VehicleMethod::Actual {
            business_miles,
            loan_interest,
            ..
        } = &mut form.vehicles[1].method
        else {
            unreachable!()
        };
        *business_miles += 1;
        match mutation {
            0 => form.employers.clear(),
            1 => form.capacity = EmployeeCapacity::Reservist,
            _ => *loan_interest = 1,
        }
        let estimated = leaf_employee_estimate(&rejected);
        let path = match mutation {
            0 => "[value/forms/1/employers]",
            1 => "[value/forms/1/vehicles/1/method]",
            _ => "[value/forms/1/vehicles/1/method/loan_interest]",
        };
        for (estimate, request) in [rejected, estimated].into_iter().enumerate() {
            let before = session.draft.clone();
            let dirty = session.dirty;
            let Reply::Response(problem @ P::Problem { .. }) = send(&mut session, request.clone())
            else {
                panic!("earlier Form 2106 policy guard must reject")
            };
            let P::Problem { message, field } = &problem else {
                unreachable!()
            };
            assert_eq!(*message, libertas::libertas_formatted_text(key, &[]));
            assert_eq!(field.as_deref(), Some(path));
            assert_eq!(session.draft, before);
            assert_eq!(session.dirty, dirty);
            crate::tests::unit47_rule_cases::export_unit47_rule_case(
                &alloc::format!("leaf-employee-priority-{mutation}-{estimate}"),
                &request,
                &problem,
            );
        }
    }
}

#[test]
fn unit47_k_parts_preflight_compares_totals_beyond_i64() {
    let mut page = k1099();
    page.parts[0].amount = i64::MAX;
    page.parts.push(page.parts[0].clone());
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::Form1099K(page)),
        Some("K_PARTS_NOT_GROSS")
    );
}

#[test]
fn unit47_misc_preflight_compares_income_beyond_i64() {
    let mut page = misc1099();
    page.other = i64::MAX;
    page.fishing = i64::MAX;
    page.withholding = i64::MAX;
    assert_eq!(crate::information_returns::misc_issue(&page), None);
    assert!(!page.record().valid());
}

#[test]
fn unit47_qof_preflight_compares_gain_beyond_i64() {
    let mut page = qof();
    page.short_term = i64::MAX;
    page.long_term = i64::MAX;
    assert_eq!(
        crate::interview::document_rule(&IncomeDocument::OpportunityFund(page.clone())),
        None
    );
    assert!(!page.valid());
}
