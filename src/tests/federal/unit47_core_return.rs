use super::*;
use crate::tests::support::client::estimate_documents_from_save;
use crate::tests::support::client::income_pages::{
    open_documents, save_documents, save_each_topic,
};
use crate::tests::support::client::survey::survey_native_refusal;
use alloc::string::String;
use alloc::string::ToString;
use libertas::SchemaPath;

fn benefits_editor(session: &mut Interview) -> P {
    let request = go_to(&session.draft, PAGE_ADJUSTMENTS, TaxSection::SelfEmployedBenefits);
    let Reply::Edit(request @ P::SaveSelfEmployedBenefits { .. }) = send(session, request) else {
        panic!("expected the actual self-employed benefits editor")
    };
    request
}

fn business_session() -> Interview {
    let mut session = at_the_adjustments_hub_with_business(8_800_000);
    save_businesses(&mut session, vec![business(8_800_000), business(4_400_000)]);
    // Complete the normal visit before taking the rejection baseline; a
    // later correction must not be mistaken for its first finished-section save.
    let request = benefits_editor(&mut session);
    accepted(&send(&mut session, request));
    session
}

fn qualified(deferral: i64, catch_up: i64, roth: i64) -> SeRetirement {
    SeRetirement::Qualified { rate: 25.0, contribution: 0, deferral, catch_up, roth }
}

#[test]
fn unit47_owner_plan_native_save_refuses_each_relationship_at_the_actual_second_row() {
    let cases = [
        (
            "owner-plan-simple-deferral",
            SeRetirement::Simple {
                deferral: 2_225_001,
                employer: SimpleEmployer::Nonelective,
                employer_contribution: 0,
            },
            "OWNER_PLAN_DEFERRAL_ABOVE_LIMIT",
            "deferral",
        ),
        (
            "owner-plan-qualified-deferral",
            qualified(2_450_001, 0, 0),
            "OWNER_PLAN_DEFERRAL_ABOVE_LIMIT",
            "deferral",
        ),
        (
            "owner-plan-qualified-catch-up",
            qualified(0, 1_125_001, 0),
            "OWNER_PLAN_DEFERRAL_ABOVE_LIMIT",
            "catch_up",
        ),
        (
            "owner-plan-qualified-roth",
            qualified(100_000, 50_000, 150_001),
            "OWNER_PLAN_ROTH_ABOVE_DEFERRALS",
            "roth",
        ),
    ];
    for (case, plan, key, field) in cases {
        let mut session = business_session();
        let mut request = benefits_editor(&mut session);
        let P::SaveSelfEmployedBenefits { value, .. } = &mut request else { unreachable!() };
        assert_eq!(value.businesses.len(), 2);
        assert_ne!(value.businesses[0].id, value.businesses[1].id);
        value.businesses[1].retirement = plan;
        let saved = session.draft.clone();
        let expected = alloc::format!("[value/businesses/1/retirement/{field}]");
        survey_native_refusal(case, &mut session, request, key, &expected);
        assert_eq!(session.draft, saved);

        let correction = benefits_editor(&mut session);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = benefits_editor(&mut restored);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_owner_plan_equal_limits_and_roth_zero_survive_save_reopen_and_restart() {
    for plan in [
        SeRetirement::Simple {
            deferral: 2_225_000,
            employer: SimpleEmployer::Nonelective,
            employer_contribution: 0,
        },
        qualified(2_450_000, 0, 0),
        qualified(0, 1_125_000, 0),
        qualified(100_000, 50_000, 150_000),
        qualified(0, 0, 0),
    ] {
        let mut session = business_session();
        let mut request = benefits_editor(&mut session);
        let P::SaveSelfEmployedBenefits { value, .. } = &mut request else { unreachable!() };
        value.businesses[1].retirement = plan.clone();
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        let mut restored = restart(saved.clone());
        let reopened = benefits_editor(&mut restored);
        let P::SaveSelfEmployedBenefits { value, .. } = &reopened else { unreachable!() };
        assert_eq!(value.businesses.len(), 2);
        assert_eq!(value.businesses[1].retirement, plan);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

fn disaster_text(field: &str, text: String) -> FederalDisasterRelief {
    let mut source = FederalDisasterRelief {
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        received: 20260612,
        qualification: DisasterQualification::FederalDeclaration {
            reference: "Synthetic declaration".into(),
        },
        purpose: DisasterReliefPurpose::PersonalExpense,
        amount: 800_000,
        withholding: 0,
        qualified: true,
    };
    match field {
        "carrier" | "carrier-event" => {
            source.purpose = DisasterReliefPurpose::CommonCarrier {
                carrier: if field == "carrier" { text.clone() } else { "Synthetic carrier".into() },
                loss: DisasterCarrierLoss::PhysicalInjury,
            };
            source.qualification = DisasterQualification::CarrierAccident {
                event: if field == "carrier-event" { text } else { "Synthetic accident".into() },
            };
        }
        "terror-event" => {
            source.qualification = DisasterQualification::TerrorOrMilitary {
                kind: DisasterActionKind::Terrorism,
                event: text,
            };
        }
        "federal-reference" => {
            source.qualification = DisasterQualification::FederalDeclaration { reference: text };
        }
        "secretary-determination" => {
            source.qualification =
                DisasterQualification::SecretaryCatastrophe { determination: text };
        }
        "government-event" | "government-authority" | "government-payer" => {
            source.purpose = DisasterReliefPurpose::GovernmentWelfare;
            source.qualification = DisasterQualification::GovernmentAssistance {
                event: if field == "government-event" {
                    text.clone()
                } else {
                    "Synthetic disaster".into()
                },
                authority: if field == "government-authority" {
                    text.clone()
                } else {
                    "Synthetic authority".into()
                },
                payer: if field == "government-payer" { text } else { "Synthetic agency".into() },
            };
        }
        _ => panic!("unknown disaster text field"),
    }
    source
}

#[test]
fn unit47_disaster_long_source_text_preserves_tax_and_required_leaf_paths() {
    let fields = [
        ("carrier", "purpose/carrier"),
        ("terror-event", "qualification/event"),
        ("federal-reference", "qualification/reference"),
        ("carrier-event", "qualification/event"),
        ("secretary-determination", "qualification/determination"),
        ("government-event", "qualification/event"),
        ("government-authority", "qualification/authority"),
        ("government-payer", "qualification/payer"),
    ];
    let texts = [
        ("ascii", "a".repeat(400)),
        ("accent", "é".repeat(200)),
        ("emoji", "😀".repeat(100)),
        ("combining", alloc::format!("{}a", "e\u{301}".repeat(133))),
    ];
    for (field, leaf) in fields {
        for (flavor, text) in &texts {
            assert_eq!(text.len(), 400);
            let source = disaster_text(field, text.clone());
            let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
            let original_agi = value(&supported(&session.draft), "agi");
            let request = save_documents(
                &mut session,
                vec![
                    document(
                        0,
                        IncomeSource::OtherIncome {
                            value: crate::OtherIncomeDocument::DisasterRelief(source.clone()),
                        },
                    ),
                    document(
                        0,
                        IncomeSource::OtherIncome {
                            value: crate::OtherIncomeDocument::DisasterRelief(source.clone()),
                        },
                    ),
                ],
            );
            accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
            let saved = session.draft.clone();
            assert!(saved.valid_stored());
            let baseline = supported(&saved);
            // A zero Schedule 1 line need not be emitted. Assert the actual
            // source exclusions and their unchanged taxable-income consequence.
            let relief_sources: Vec<_> = saved
                .income
                .iter()
                .filter(|entry| matches!(entry.income, FederalIncome::DisasterRelief { .. }))
                .collect();
            assert_eq!(relief_sources.len(), 2);
            for entry in relief_sources {
                assert_eq!(
                    value(&baseline, &alloc::format!("disaster_relief_{}_excluded", entry.id)),
                    8_000
                );
            }
            assert_eq!(value(&baseline, "agi"), original_agi);

            let mut invalid = open_documents(&mut session);
            let P::SaveDocuments { documents, .. } = &mut invalid else { unreachable!() };
            assert_eq!(documents.other.other_income.documents.len(), 2);
            documents.other.other_income.documents[1].document =
                crate::OtherIncomeDocument::DisasterRelief(disaster_text(field, " ".into()));
            let expected =
                alloc::format!("[documents/other/other_income/documents/1/document/{leaf}]");
            let case = alloc::format!("disaster-{field}-{flavor}-required-blank");
            survey_native_refusal(
                &case,
                &mut session,
                invalid,
                "DISASTER_SOURCE_TEXT_INVALID",
                &expected,
            );
            assert_eq!(session.draft, saved);

            let correction = open_documents(&mut session);
            accepted(&send(&mut session, correction));
            assert_eq!(session.draft, saved);
            let mut restored = restart(saved.clone());
            let reopened = open_documents(&mut restored);
            let expected = expected_named_sources(
                &restored.draft,
                Owner::Taxpayer,
                vec![IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::DisasterRelief(source),
                }],
            );
            assert!(
                page_documents(&reopened)
                    .into_iter()
                    .filter(|row| matches!(row.document, IncomeSource::OtherIncome { .. }))
                    .all(|item| item.document == expected[0])
            );
            accepted(&send(&mut restored, reopened));
            assert_eq!(restored.draft, saved);
        }
    }
}

fn marketplace_sources_page(child: i64) -> MarketplaceFactsPage {
    let mut page = marketplace_fixture_page(vec![
        all_year("First synthetic issuer", 100_000, 60_000, 0),
        all_year("Second synthetic issuer", 100_000, 60_000, 0),
    ]);
    let form = &mut page.forms[1];
    form.value.set_amounts(PolicyAmounts::ByMonth {
        months: vec![
            PolicyMonth { month: 1, premium: 100_000, benchmark: Some(60_000), advance: 0 },
            PolicyMonth { month: 2, premium: 100_000, benchmark: Some(60_000), advance: 0 },
        ],
    });
    form.coverage.truncate(2);
    form.coverage[1].people.push(child);
    form.coverage[1].entry = vec![
        MarketplaceCoverageEntryPage {
            person: -1,
            start: 20260201,
            end: 20260228,
            basis: MarketplaceEnrollmentBasis::Ordinary,
        },
        MarketplaceCoverageEntryPage {
            person: child,
            start: 20260201,
            end: 20260228,
            basis: MarketplaceEnrollmentBasis::Ordinary,
        },
    ];
    form.payments = (1..=2)
        .map(|month| MarketplacePaymentMonthPage {
            month,
            paid: 100_000,
            refunded: 10_000,
            tax_free_premiums: 90_000,
            tax_free_coverage: Some(0),
            unpaid: 10_000,
            coverage: MarketplacePaymentCoverage::PaidByDeadline,
            appeal: None,
        })
        .collect();
    form.medical_costs = (1..=2)
        .map(|month| MarketplaceMedicalCostMonth {
            month,
            child_increase: Some(10_000),
            // All 90,000 net-paid cents were funded tax-free, including this
            // 10,000 child-cost portion; the remaining 80,000 funds other costs.
            tax_free_child: Some(10_000),
        })
        .collect();
    page.eligibility.push(MarketplaceEligibilityMonthPage {
        month: 2,
        person: child,
        status: MarketplacePersonEligibility::Eligible,
    });
    page.dependents.push(DependentReturn {
        person: child,
        agi: 0,
        exempt_interest: 0,
        foreign_exclusion: 0,
        social: 40_000,
        taxable_social: 40_000,
    });
    page
}

#[test]
fn unit47_marketplace_native_source_relationships_locate_rows_and_preserve_accepted_facts() {
    let cases = [
        // 2026-10-27: Form1095APaper names the printed February row directly.
        ("advance", "MARKETPLACE_ADVANCE_ABOVE_PREMIUM", "forms/1/value/february/advance"),
        (
            "dependent-social",
            "MARKETPLACE_TAXABLE_SOCIAL_ABOVE_GROSS",
            "dependents/0/taxable_social",
        ),
        ("coverage-dates", "PTC_COVERAGE_DATES_REVERSED", "forms/1/coverage/1/entry/1/end"),
        (
            "tax-free-premiums",
            "PTC_TAX_FREE_PREMIUMS_ABOVE_NET_PAID",
            "forms/1/payments/1/tax_free_premiums",
        ),
        (
            "tax-free-coverage",
            "PTC_TAX_FREE_COVERAGE_ABOVE_TOTAL",
            "forms/1/payments/1/tax_free_coverage",
        ),
        (
            "child-funding",
            "HEALTH_POLICY_CHILD_FUNDING_ABOVE_COST",
            "forms/1/medical_costs/1/tax_free_child",
        ),
        (
            "child-funding-unavailable",
            "HEALTH_POLICY_CHILD_FUNDING_ABOVE_COST",
            "forms/1/medical_costs/1/tax_free_child",
        ),
    ];
    for (case, key, leaf) in cases {
        let mut draft = complete(FederalStatus::Single);
        add_child(&mut draft);
        let child = draft.dependents[0].id;
        let (mut session, _) = at_the_marketplace_page(draft);
        let mut value = marketplace_sources_page(child);
        match case {
            "advance" => {
                value.forms[1].value.month_mut(2).advance = Some(100_000);
            }
            "tax-free-coverage" => value.forms[1].payments[1].tax_free_coverage = Some(90_000),
            "child-funding" => value.forms[1].medical_costs[1].tax_free_child = Some(10_000),
            _ => {}
        }
        save_marketplace_and_reopen(&mut session, value);
        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        let source_case = case == "advance";
        let mut request = if source_case {
            open_documents(&mut session)
        } else {
            open_credits_topic(&mut session, TaxSection::Marketplace)
        };
        if let P::SaveMarketplace { value, .. } = &mut request {
            let forms = &mut value.forms;
            match case {
                "dependent-social" => value.dependents[0].taxable_social = 40_001,
                "coverage-dates" => forms[1].coverage[1].entry[1].end = 20260131,
                "tax-free-premiums" => forms[1].payments[1].tax_free_premiums = 90_001,
                "tax-free-coverage" => forms[1].payments[1].tax_free_coverage = Some(90_001),
                "child-funding" => forms[1].medical_costs[1].tax_free_child = Some(10_001),
                "child-funding-unavailable" => {
                    forms[1].medical_costs[1].child_increase = None;
                    forms[1].medical_costs[1].tax_free_child = Some(1);
                }
                _ => unreachable!(),
            }
        } else {
            let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
            documents.common.marketplace.documents[1].value.month_mut(2).advance = Some(100_001);
        }
        let expected = if source_case {
            let source_leaf = leaf.strip_prefix("forms/").unwrap();
            alloc::format!("[documents/common/marketplace/documents/{source_leaf}]")
        } else {
            alloc::format!("[value/{leaf}]")
        };
        let case = alloc::format!("marketplace-{case}");
        survey_native_refusal(&case, &mut session, request, key, &expected);
        assert_eq!(session.draft, saved);
        let correction = if source_case {
            open_documents(&mut session)
        } else {
            open_credits_topic(&mut session, TaxSection::Marketplace)
        };
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = if source_case {
            open_documents(&mut restored)
        } else {
            open_credits_topic(&mut restored, TaxSection::Marketplace)
        };
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_retirement_native_owners_locate_printed_treatment_and_nested_disaster_failures() {
    let base = r1099(1_000_000, normal_kept(), false);
    let mut cases = Vec::new();
    let mut invalid = base.clone();
    invalid.source_mut().code = Some(early_kept());
    cases.push(("treatment-selection", invalid, "R_BOX7_TREATMENT", "code"));
    let invalid = printed_as(roth_item_for_tests(), "J");
    cases.push(("roth-printed-subcode", invalid, "R_ROTH_CODE_NOT_BOX7", "code/roth"));
    let invalid = r1099(
        1_000_000,
        DistributionCode::Disability { began: 20250101, wages: DisabilityWages::All },
        true,
    );
    cases.push(("ira-disability-wages", invalid, "R_IRA_DISABILITY_NOT_WAGES", "code/wages"));
    let invalid = r1099(
        1_000_000,
        DistributionCode::Disability {
            began: 20250101,
            wages: DisabilityWages::Part { amount: 1_000_001 },
        },
        false,
    );
    cases.push((
        "disability-wages",
        invalid,
        "R_DISABILITY_WAGES_ABOVE_TAXABLE",
        "code/wages/amount",
    ));
    for (case, gross, taxable, leaf) in [
        ("after-death-gross", 1_000_001, 500_000, "after_death/gross"),
        ("after-death-taxable-box", 1_000_000, 1_000_001, "after_death/taxable"),
        ("after-death-taxable-part", 400_000, 400_001, "after_death/taxable"),
    ] {
        let mut invalid = base.clone();
        invalid.source_mut().after_death = Some(crate::RetirementAfterDeath { gross, taxable });
        cases.push((case, invalid, "AFTER_DEATH_ABOVE_BOX", leaf));
    }
    for (case, field) in [
        ("roth-box3", "box_3"),
        ("roth-box6", "box_6"),
        ("roth-box8a", "box_8a"),
        ("roth-box9b-present-zero", "box_9b"),
        ("roth-box10", "box_10"),
        ("roth-disability", "disability"),
    ] {
        let mut invalid = roth_item_for_tests();
        match field {
            "box_3" => invalid.source_mut().form.box_3 = Some(1),
            "box_6" => invalid.source_mut().form.box_6 = Some(1),
            "box_8a" => invalid.source_mut().form.box_8a = Some(1),
            "box_9b" => invalid.source_mut().form.box_9b = Some(0),
            "box_10" => invalid.source_mut().form.box_10 = Some(1),
            "disability" => {
                invalid.source_mut().disability =
                    Some(crate::DisabilityIncome::Qualifying { amount: 0, eligible: true })
            }
            _ => unreachable!(),
        }
        let leaf = match field {
            "box_3" => "more/box_3",
            "box_6" => "more/box_6",
            "box_8a" => "more/box_8a",
            "box_9b" => "more/box_9b",
            "box_10" => "more/box_10",
            "disability" => "more/disability",
            _ => unreachable!(),
        };
        cases.push((case, invalid, "R_ROTH_OTHER_BOXES", leaf));
    }
    let mut invalid = roth_item_for_tests();
    invalid.source_mut().form.box_8b_percent = Some(33.33);
    cases.push(("roth-box8b-precedence", invalid, "R_BOX8B_WITHOUT_BOX8A", "more/box_8b_percent"));
    for (case, printed, roth, leaf) in [
        (
            "roth-early-disabled-date",
            "J",
            RothIraCode::Early {
                distributed: 20260601,
                exception: Some(crate::RetirementException::Disability { began: 20260602 }),
                repaid: None,
            },
            "code/roth/exception/began",
        ),
        (
            "roth-exception-disabled-date",
            "T",
            RothIraCode::ExceptionApplies {
                distributed: 20260601,
                reason: crate::RothReason::Disabled { since: 20260602 },
            },
            "code/roth/reason/since",
        ),
    ] {
        let invalid = printed_as(
            r1099(
                1_000_000,
                DistributionCode::RothIra { roth, sequence: None, rolled_over: 0 },
                false,
            ),
            printed,
        );
        cases.push((case, invalid, "ROTH_DISABLED_AFTER_DISTRIBUTION", leaf));
    }
    let mut invalid = base.clone();
    invalid.source_mut().form.taxable = Some(1_000_001);
    invalid.source_mut().form.withholding = Some(1_000_001);
    cases.push(("taxable-before-withholding", invalid, "R_TAXABLE_ABOVE_GROSS", "taxable"));
    let mut invalid = base.clone();
    invalid.source_mut().after_death =
        Some(crate::RetirementAfterDeath { gross: 1_000_001, taxable: 0 });
    invalid.source_mut().form.withholding = Some(1_000_001);
    cases.push((
        "after-death-before-withholding",
        invalid,
        "AFTER_DEATH_ABOVE_BOX",
        "after_death/gross",
    ));
    let mut invalid = base.clone();
    invalid.source_mut().form.taxable = Some(1_000_001);
    cases.push(("taxable", invalid, "R_TAXABLE_ABOVE_GROSS", "taxable"));
    let mut invalid = base.clone();
    invalid.source_mut().form.withholding = Some(1_000_001);
    cases.push(("withholding", invalid, "R_WITHHOLDING_ABOVE_GROSS", "withholding"));
    let mut invalid = base.clone();
    invalid.source_mut().form.box7 = 0;
    cases.push(("box7-empty", invalid, "R_BOX7_NO_CODE", "box7"));
    let mut invalid = base.clone();
    invalid.source_mut().form.box7 = 7;
    cases.push(("box7-three-codes", invalid, "R_BOX7_TOO_MANY_CODES", "box7"));
    let mut invalid = base.clone();
    invalid.source_mut().form.box_10 = Some(1);
    cases.push(("box10-without-b", invalid, "R_BOX10_WITHOUT_B", "more/box_10"));
    let mut invalid = printed_as(base.clone(), "7B");
    invalid.source_mut().form.box_10 = Some(1_000_001);
    cases.push(("box10-above-gross", invalid, "R_BOX10_ABOVE_GROSS", "more/box_10"));
    let invalid = r1099(
        1_000_000,
        DistributionCode::Normal {
            what_happened: Disposition::RolledOver {
                amount: 1_000_001,
                into: RolloverDestination::Ira,
            },
        },
        false,
    );
    cases.push(("rollover", invalid, "R_ROLLOVER_ABOVE_GROSS", "code/what_happened/amount"));
    let invalid = r1099(
        1_000_000,
        DistributionCode::Normal { what_happened: Disposition::Converted { amount: 1_000_001 } },
        false,
    );
    cases.push(("conversion", invalid, "R_CONVERSION_ABOVE_GROSS", "code/what_happened/amount"));
    let invalid = r1099(
        1_000_000,
        DistributionCode::NormalLumpSum {
            tax: LumpSumTax::WithOtherIncome {
                what_happened: Disposition::RolledOver {
                    amount: 1_000_001,
                    into: RolloverDestination::Ira,
                },
            },
        },
        false,
    );
    cases.push((
        "lump-rollover",
        invalid,
        "R_ROLLOVER_ABOVE_GROSS",
        "code/tax/what_happened/amount",
    ));
    let mut invalid = roth_item_for_tests();
    let Some(DistributionCode::RothIra { rolled_over, .. }) = &mut invalid.source_mut().code else {
        unreachable!()
    };
    *rolled_over = 1_000_001;
    cases.push(("roth-rollover", invalid, "R_ROTH_ROLLOVER_ABOVE_GROSS", "code/rolled_over"));
    let mut invalid = base.clone();
    invalid.set_public_safety(vec![
        PublicSafetyPaid {
            coverage: PublicSafetyPaidCoverage::Medical,
            premiums: 100_000,
            funded: 50_000,
        },
        PublicSafetyPaid {
            coverage: PublicSafetyPaidCoverage::Medical,
            premiums: 100_000,
            funded: 100_001,
        },
    ]);
    cases.push((
        "public-safety-premium",
        invalid,
        "PSO_FUNDED_ABOVE_PREMIUMS",
        "public_safety/rows/1/funded",
    ));
    let mut invalid = base.clone();
    invalid.set_public_safety(vec![
        PublicSafetyPaid {
            coverage: PublicSafetyPaidCoverage::Medical,
            premiums: 1_000_000,
            funded: 500_000,
        },
        PublicSafetyPaid {
            coverage: PublicSafetyPaidCoverage::Medical,
            premiums: 1_000_000,
            funded: 500_001,
        },
    ]);
    cases.push(("public-safety-gross", invalid, "PSO_FUNDED_ABOVE_GROSS", "public_safety/rows"));
    let invalid = disaster_r1099(3_000_001, 0, DisasterSpread::AllInReturnYear);
    cases.push(("disaster-gross", invalid, "R_DISASTER_ABOVE_GROSS", "disaster/amount"));
    let invalid = disaster_r1099(2_000_000, 200_001, DisasterSpread::AllInReturnYear);
    cases.push(("disaster-limit", invalid, "DISASTER_ABOVE_LIMIT", "disaster/earlier"));
    for (case, total, in_return_year, key, leaf) in [
        (
            "disaster-repaid-total",
            1_000_001,
            0,
            "DISASTER_REPAID_ABOVE_AMOUNT",
            "disaster/repaid/total",
        ),
        (
            "disaster-repaid-year",
            500_000,
            500_001,
            "DISASTER_REPAID_2026_ABOVE_REPAID",
            "disaster/repaid/in_return_year",
        ),
    ] {
        let mut invalid = disaster_r1099(1_000_000, 0, DisasterSpread::AllInReturnYear);
        let DisasterDistribution::Qualified { repaid, .. } =
            invalid.source_mut().disaster.as_mut().unwrap()
        else {
            unreachable!()
        };
        *repaid = DisasterRepayment::Yes { total, in_return_year };
        cases.push((case, invalid, key, leaf));
    }
    let mut invalid = disaster_r1099(1_000_000, 0, DisasterSpread::AllInReturnYear);
    let DisasterDistribution::Qualified { distributed, .. } =
        invalid.source_mut().disaster.as_mut().unwrap()
    else {
        unreachable!()
    };
    *distributed = 20261231;
    cases.push(("disaster-date", invalid, "DISASTER_OUTSIDE_PERIOD", "disaster/distributed"));
    for (case, mut invalid, key, leaf) in cases {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut seed = base.clone();
        // Retain this existing personal answer through the paper-kind correction;
        // Roth itself does not offer a new disability-pension answer.
        if case == "roth-disability" {
            seed.source_mut().disability = invalid.source_mut().disability.clone();
        }
        accepted(&crate::tests::support::client::save_retirement_fixture(
            &mut session,
            vec![
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(base.clone())),
                    },
                ),
                document(
                    0,
                    IncomeSource::Retirement {
                        value: crate::RetirementDocument::R1099(retirement_document(seed)),
                    },
                ),
            ],
        ));
        let saved = session.draft.clone();
        let mut request = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        assert_eq!(documents.common.retirement.documents.len(), 2);
        let id = documents.common.retirement.documents[1].id;
        documents.common.retirement.documents[1].document =
            crate::RetirementDocument::R1099(retirement_document(invalid.clone()));
        // The printed boxes sit on the 1099-R paper itself, outside the old `more` group.
        let source_path = alloc::format!(
            "[documents/common/retirement/documents/1/document/{}]",
            leaf.strip_prefix("more/").unwrap_or(leaf)
        );
        let case_name = alloc::format!("retirement-{case}");
        let mut candidate = restart(saved.clone());
        let reply = send(&mut candidate, request.clone());
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            if case == "after-death-before-withholding" {
                survey_native_refusal(
                    &alloc::format!("{case_name}-printed-withholding"),
                    &mut session,
                    request,
                    "R_WITHHOLDING_ABOVE_GROSS",
                    "[documents/common/retirement/documents/1/document/withholding]",
                );
                // The personal failure is probed against the actually accepted paper.
                invalid.source_mut().form.withholding = base.withholding;
                candidate = restart(saved.clone());
            } else {
                survey_native_refusal(&case_name, &mut session, request, key, &source_path);
                assert_eq!(session.draft, saved);
                let correction = open_documents(&mut session);
                accepted(&send(&mut session, correction));
                assert_eq!(session.draft, saved);
                continue;
            }
        } else {
            accepted(&reply);
        }
        let desired = vec![document(
            id,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(invalid)),
            },
        )];
        let request =
            crate::tests::support::client::income_pages::information_return_answers_for_topic(
                &mut candidate,
                &desired,
                TaxSection::Retirement,
            )
            .unwrap();
        let expected = retirement_followup_path(&request, id, leaf);
        survey_native_refusal(&case_name, &mut candidate, request.clone(), key, &expected);
        survey_native_refusal(
            &alloc::format!("{case_name}-estimate"),
            &mut candidate,
            estimate_documents_from_save(&request),
            key,
            &expected,
        );
        assert_eq!(session.draft, saved);
        let correction = open_documents(&mut session);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
    }
    // Preserve the explicitly approved informational box9a exception after removal.
    let mut roth = roth_item_for_tests();
    roth.source_mut().form.box_9a_percent = Some(33.33);
    assert_eq!(roth.source_mut().form.box_9a_percent, Some(33.33));
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&crate::tests::support::client::save_retirement_fixture(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(roth.clone())),
            },
        )],
    ));
    let saved = session.draft.clone();
    let canonical_page = open_documents(&mut session);
    let IncomeSource::Retirement { value: crate::RetirementDocument::R1099(canonical) } =
        &page_documents(&canonical_page)
            .into_iter()
            .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
            .collect::<Vec<_>>()[0]
            .document
    else {
        unreachable!()
    };
    let canonical = canonical.clone();
    let expected = retirement_document(roth);
    // The canonical Roth owner retains the printed informational percentage.
    assert_eq!(expected.box_9a_percent, Some(33.33));
    let expected = expected_named_sources(
        &session.draft,
        Owner::Taxpayer,
        vec![IncomeSource::Retirement { value: crate::RetirementDocument::R1099(expected) }],
    );
    assert_eq!(
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(canonical.clone()) },
        expected[0]
    );
    let mut restored = restart(saved.clone());
    let reopened = open_documents(&mut restored);
    assert!(
        page_documents(&reopened).into_iter().filter(|row| matches!(row.document, IncomeSource::Retirement { .. })).any(|item| matches!(&item.document,
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(actual) } if *actual == canonical))
    );
    accepted(&send(&mut restored, reopened));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_retirement_native_save_accepts_equal_amount_and_disability_date_boundaries() {
    let mut rows = vec![
        r1099(
            1_000_000,
            DistributionCode::Disability {
                began: 20250101,
                wages: DisabilityWages::Part { amount: 1_000_000 },
            },
            false,
        ),
        r1099(
            1_000_000,
            DistributionCode::Disability { began: 20250101, wages: DisabilityWages::None },
            true,
        ),
    ];
    for (printed, roth) in [
        (
            "J",
            RothIraCode::Early {
                distributed: 20260601,
                exception: Some(crate::RetirementException::Disability { began: 20260601 }),
                repaid: None,
            },
        ),
        (
            "T",
            RothIraCode::ExceptionApplies {
                distributed: 20260601,
                reason: crate::RothReason::Disabled { since: 20260601 },
            },
        ),
    ] {
        rows.push(printed_as(
            r1099(
                1_000_000,
                DistributionCode::RothIra { roth, rolled_over: 0, sequence: None },
                false,
            ),
            printed,
        ));
    }
    let mut after_death = r1099(1_000_000, normal_kept(), false);
    after_death.source_mut().after_death =
        Some(crate::RetirementAfterDeath { gross: 1_000_000, taxable: 1_000_000 });
    rows.push(after_death);
    for row in rows {
        assert!(crate::retirement::document_issue(&row).is_none());
        let mut draft = complete(FederalStatus::Single);
        if row.after_death.is_some() {
            draft.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
                bonds: None,
                estate_medical: 0,
                date: 20260601,
                ordinary: true,
            });
        }
        let mut session = Interview::new(TEST_USER, draft);
        let desired = vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(row)),
            },
        )];
        let request = save_documents(&mut session, desired.clone());
        let request = P::from_avro(&request.to_avro()).unwrap();
        accepted(&send(&mut session, request));
        let answers = crate::tests::support::client::income_pages::information_return_answers(
            &mut session,
            &desired,
        );
        let answers = P::from_avro(&answers.to_avro()).unwrap();
        accepted(&send(&mut session, answers));
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let reopened = open_documents(&mut restored);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_reservist_second_anniversary_is_inclusive_and_leap_day_clamps_before_native_save() {
    for (case, released, deadline, late) in
        [("january", 20240101, 20260101, 20260102), ("leap-day", 20240229, 20260228, 20260301)]
    {
        let repayment = |released, distributed, last_payment| ReservistRepayment {
            ordered: 20220101,
            term: ReserveTerm::Indefinite,
            released,
            distributed,
            gross: 1_200_000,
            prior: 0,
            payments: vec![
                ReservistPayment { date: 20260101, amount: 100_000 },
                ReservistPayment { date: last_payment, amount: 200_000 },
            ],
        };
        let second = repayment(released, 20230102, deadline);
        assert_eq!(second.repayment_deadline(), deadline);
        assert!(second.issue().is_none());
        let form = Form8606 {
            basis: 0,
            adjustment: 0,
            year_end: 5_000_000,
            reservist_repayments: vec![repayment(20240101, 20230101, 20260101), second],
        };
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let rows = vec![
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(r1099(
                        1_000_000,
                        normal_kept(),
                        true,
                    ))),
                },
            ),
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::Form8606(form.clone()) },
            ),
        ];
        save_each_topic(&mut session, rows);

        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        let result = supported(&saved);
        assert_eq!(value(&result, "reservist_taxpayer_2_basis"), 3_000);
        assert_eq!(value(&result, "reservist_taxpayer_2_unrepaid"), 9_000);
        let mut request = open_topic(&mut session, TaxSection::Retirement);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        let index = documents
            .documents
            .iter()
            .position(|item| matches!(item.document, crate::IncomeRecord::Form8606(_)))
            .unwrap();
        assert_eq!(index, 0);
        let crate::IncomeRecord::Form8606(actual) = &mut documents.documents[index].document else {
            unreachable!()
        };
        assert_eq!(*actual, form);
        actual.reservist_repayments[1].payments[1].date = late;
        let case = alloc::format!("reservist-second-anniversary-{case}");
        survey_native_refusal(
            &case,
            &mut session,
            request,
            "RESERVIST_REPAYMENT_LATE",
            "[documents/documents/0/document/reservist_repayments/1/payments/1/date]",
        );
        assert_eq!(session.draft, saved);
        let correction = open_topic(&mut session, TaxSection::Retirement);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = open_topic(&mut restored, TaxSection::Retirement);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
        assert_eq!(value(&supported(&restored.draft), "reservist_taxpayer_2_basis"), 3_000);
    }
}

fn unit47_ira_session(status: FederalStatus, form_8606: bool) -> Interview {
    let mut session = Interview::new(TEST_USER, complete(status));
    if form_8606 {
        with_form_8606(&mut session, 40_000);
    }
    // Only the spouse's traditional IRA withdrawals count for the spouse.
    // Another owner's IRA and this owner's plan distributions stay outside it.
    if status == FederalStatus::Joint {
        for (owner, kind, gross) in [
            (Owner::Spouse, RetirementKind::Ira, 5_000),
            (Owner::Taxpayer, RetirementKind::Ira, 20_000),
            (Owner::Spouse, RetirementKind::Pension, 20_000),
        ] {
            add(
                &mut session.draft,
                retirement_entry(owner, kind, gross, RetirementTreatment::Ordinary),
            );
        }
    }
    let (_, allowed) = open_ira_page(&mut session);
    let mut people = vec![ira_row(Owner::Taxpayer, 100_000, None)];
    if status == FederalStatus::Joint {
        people.push(ira_row(Owner::Spouse, 200_000, None));
    }
    let request = save_ira_people(&session.draft, allowed, people);
    accepted(&send(&mut session, request));
    session
}

#[test]
fn unit47_ira_contribution_preflight_locates_actual_rows_and_preserves_rejected_state() {
    for case in [
        "owner",
        "duplicate",
        "nondeductible",
        "source-value-duplicate",
        "source-value-missing",
        "returned-above-carried",
        "returned-above-withdrawn",
        "late-route-needed",
        "early-route-refused",
        "returned-loss",
        "intrinsic-principal",
        "intrinsic-total",
        "duplicate-before-nondeductible",
        "nondeductible-before-excess",
        "excess-before-returned",
    ] {
        let status = if case == "owner" { FederalStatus::Single } else { FederalStatus::Joint };
        let mut session = unit47_ira_session(status, case == "source-value-duplicate");
        let saved = session.draft.clone();
        let (mut people, allowed) = open_ira_page(&mut session);
        let (key, leaf) = match case {
            "owner" => {
                people.push(ira_row(Owner::Spouse, 200_000, None));
                (
                    "MESSAGE_YOUR_SPOUSE_S_CONTRIBUTIONS_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
                    "people/1/owner",
                )
            }
            "duplicate" | "duplicate-before-nondeductible" => {
                people[1].owner = Owner::Taxpayer;
                if case == "duplicate-before-nondeductible" {
                    people[1].more.nondeductible = 200_001;
                }
                ("IRA_CONTRIBUTOR_TWICE", "people/1/owner")
            }
            "nondeductible" | "nondeductible-before-excess" => {
                people[1].more.nondeductible = 200_001;
                if case == "nondeductible-before-excess" {
                    people[0].more.excess = Some(excess_entry(None));
                }
                ("IRA_NONDEDUCTIBLE_ABOVE_TRADITIONAL", "people/1/more/nondeductible")
            }
            "source-value-duplicate" => {
                people[0].form_8606_value = None;
                people[0].more.excess = Some(excess_entry(Some(0)));
                ("IRA_EXCESS_TRADITIONAL_ON_8606", "people/0/more/excess/traditional_value")
            }
            "source-value-missing" => {
                people[1].form_8606_value = Some(40_000);
                people[1].more.excess = Some(excess_entry(None));
                ("IRA_EXCESS_TRADITIONAL_VALUE_NEEDED", "people/1/more/excess/traditional_value")
            }
            "returned-above-carried" | "excess-before-returned" => {
                let mut excess = excess_entry(Some(4_000_000));
                excess.prior_traditional = 500_000;
                excess.returned_prior_excess = 500_001;
                people[1].more.excess = Some(excess);
                if case == "excess-before-returned" {
                    people[0].more.returned_in_following_year = vec![crate::ReturnedContribution {
                        kind: crate::IraKind::Traditional,
                        principal: 100_000,
                        earnings: 0,
                        returned_on: 20270416,
                        late: None,
                    }];
                }
                ("IRA_RETURNED_EXCESS_ABOVE_CARRIED", "people/1/more/excess/returned_prior_excess")
            }
            "returned-above-withdrawn" => {
                let mut excess = excess_entry(Some(4_000_000));
                excess.prior_traditional = 600_000;
                excess.returned_prior_excess = 500_001;
                people[1].more.excess = Some(excess);
                (
                    "IRA_RETURNED_EXCESS_ABOVE_DISTRIBUTIONS",
                    "people/1/more/excess/returned_prior_excess",
                )
            }
            _ => {
                let good = crate::ReturnedContribution {
                    kind: crate::IraKind::Roth,
                    principal: 100_000,
                    earnings: 0,
                    returned_on: 20270415,
                    late: None,
                };
                let mut invalid = good.clone();
                let (key, leaf) = match case {
                    "late-route-needed" => {
                        invalid.returned_on = 20270416;
                        (
                            "IRA_RETURN_AFTER_DUE_DATE_NEEDS_ROUTE",
                            "people/1/more/returned_in_following_year/1/late",
                        )
                    }
                    "early-route-refused" => {
                        invalid.late = Some(crate::LateCorrection::FilingExtension);
                        (
                            "IRA_RETURN_ROUTE_WITHOUT_LATE_DATE",
                            "people/1/more/returned_in_following_year/1/late",
                        )
                    }
                    "returned-loss" => {
                        invalid.earnings = -100_001;
                        (
                            "IRA_RETURN_LOSS_ABOVE_PRINCIPAL",
                            "people/1/more/returned_in_following_year/1/earnings",
                        )
                    }
                    "intrinsic-principal" => {
                        invalid.principal = 0;
                        (
                            "IRA_RETURN_LOSS_ABOVE_PRINCIPAL",
                            "people/1/more/returned_in_following_year/1",
                        )
                    }
                    "intrinsic-total" => {
                        invalid.principal = 100_000_000_000;
                        invalid.earnings = 1;
                        (
                            "IRA_RETURN_LOSS_ABOVE_PRINCIPAL",
                            "people/1/more/returned_in_following_year/1",
                        )
                    }
                    _ => unreachable!(),
                };
                people[1].more.returned_in_following_year = vec![good, invalid];
                (key, leaf)
            }
        };
        let request = save_ira_people(&session.draft, allowed, people);
        let expected = alloc::format!("[value/{leaf}]");
        let name = alloc::format!("ira-contributions-{case}");
        survey_native_refusal(&name, &mut session, request, key, &expected);
        assert_eq!(session.draft, saved);
        let (people, allowed) = open_ira_page(&mut session);
        let correction = save_ira_people(&session.draft, allowed, people);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let (people, allowed) = open_ira_page(&mut restored);
        let reopened = save_ira_people(&restored.draft, allowed, people);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_ira_contribution_save_preserves_source_values_and_inclusive_boundaries() {
    for case in [
        "nondeductible-equal",
        "returned-equal-carried-and-withdrawn",
        "zero-value-without-form",
        "actual-form-value",
        "returned-loss-equal-principal",
        "returned-by-deadline",
        "returned-with-extension",
        "returned-with-timely-return",
    ] {
        let mut session = unit47_ira_session(FederalStatus::Joint, case == "actual-form-value");
        let (mut people, allowed) = open_ira_page(&mut session);
        let target = if case == "actual-form-value" { 0 } else { 1 };
        match case {
            "nondeductible-equal" => people[target].more.nondeductible = people[target].traditional,
            "returned-equal-carried-and-withdrawn" => {
                let mut excess = excess_entry(Some(4_000_000));
                excess.prior_traditional = 500_000;
                excess.returned_prior_excess = 500_000;
                people[target].more.excess = Some(excess);
            }
            "zero-value-without-form" => {
                people[target].form_8606_value = Some(999_999);
                people[target].more.excess = Some(excess_entry(Some(0)));
            }
            "actual-form-value" => {
                people[target].form_8606_value = Some(999_999);
                people[target].more.excess = Some(excess_entry(None));
            }
            _ => {
                let (earnings, returned_on, late) = match case {
                    "returned-loss-equal-principal" => (-100_000, 20270415, None),
                    "returned-by-deadline" => (0, 20270415, None),
                    "returned-with-extension" => {
                        (0, 20270416, Some(crate::LateCorrection::FilingExtension))
                    }
                    "returned-with-timely-return" => {
                        (0, 20271015, Some(crate::LateCorrection::TimelyFiledReturn))
                    }
                    _ => unreachable!(),
                };
                people[target].more.returned_in_following_year =
                    vec![crate::ReturnedContribution {
                        kind: crate::IraKind::Roth,
                        principal: 100_000,
                        earnings,
                        returned_on,
                        late,
                    }];
            }
        }
        let expected = people[target].clone();
        let request = save_ira_people(&session.draft, allowed, people);
        let request = P::from_avro(&request.to_avro()).unwrap();
        accepted(&send(&mut session, request));
        if case == "actual-form-value" {
            assert_eq!(
                session
                    .draft
                    .adjustments
                    .as_ref()
                    .unwrap()
                    .iras
                    .taxpayer
                    .excess
                    .as_ref()
                    .unwrap()
                    .traditional_value,
                Some(40_000),
            );
        }
        if case == "zero-value-without-form" {
            let IraSpouse::Joint { facts } =
                &session.draft.adjustments.as_ref().unwrap().iras.spouse
            else {
                unreachable!()
            };
            assert_eq!(facts.excess.as_ref().unwrap().traditional_value, Some(0));
        }
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let (people, allowed) = open_ira_page(&mut restored);
        let actual = people.iter().find(|p| p.owner == expected.owner).unwrap();
        assert_eq!(actual.traditional, expected.traditional);
        assert_eq!(actual.more, expected.more);
        assert_eq!(
            actual.form_8606_value,
            if case == "actual-form-value" { Some(40_000) } else { None },
        );
        let reopened = save_ira_people(&restored.draft, allowed, people);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

// OutputOnly is a client serialization boundary. Native Avro decoding keeps
// receiving complete requests, and the Save owner must ignore display echoes.
fn clear_more_credits_output(request: &mut P) {
    let P::SaveMoreCredits {
        estimate,
        warnings,
        progress,
        allowed_owners,
        foreign_businesses,
        capital_sales,
        foreign_situations,
        mortgage_loans,
        businesses,
        farms,
        rentals,
        research_businesses,
        entered_credits,
        carryback_credits,
        allowed_carry_filers,
        initial_credit_elections,
        initial_research_choices,
        initial_carbon_oxide_origins,
        initial_renewable_facility_origins,
        ..
    } = request
    else {
        panic!("expected the normal More credits editor")
    };
    *estimate = None;
    warnings.clear();
    *progress = None;
    allowed_owners.clear();
    foreign_businesses.clear();
    capital_sales.clear();
    foreign_situations.clear();
    mortgage_loans.clear();
    businesses.clear();
    farms.clear();
    rentals.clear();
    research_businesses.clear();
    entered_credits.clear();
    carryback_credits.clear();
    allowed_carry_filers.clear();
    initial_credit_elections.clear();
    initial_research_choices.clear();
    initial_carbon_oxide_origins.clear();
    initial_renewable_facility_origins.clear();
}

fn forge_more_credits_output(request: &mut P) {
    clear_more_credits_output(request);
    let P::SaveMoreCredits {
        estimate,
        progress,
        warnings,
        allowed_owners,
        foreign_businesses,
        entered_credits,
        ..
    } = request
    else {
        unreachable!()
    };
    *estimate = Some(crate::EstimateSoFar { line: vec![0xff, 0, 0xfe], remaining_items: vec![] });
    *progress = Some(vec![0xfe, 0, 0xff]);
    *warnings = vec![crate::SaveWarning { message: vec![0xff] }];
    *allowed_owners = vec![i32::MIN, i32::MAX, i32::MIN];
    *entered_credits = vec![i32::MIN, i32::MIN];
    *foreign_businesses = vec![crate::RecordChoice { id: i64::MIN, label: vec![0xff] }];
}

#[test]
fn unit47_output_only_displays_and_catalogs_do_not_change_native_save_or_reopen() {
    for forged in [false, true] {
        let (mut session, _, guide) = super::unit47_text_size::country_fixture();
        let mut reference = Interview::new(TEST_USER, session.draft.clone());
        let original = P::from_avro(&guide.to_avro()).unwrap();
        accepted(&send(&mut reference, original));
        let expected = reference.draft.clone();
        // This country-path fixture deliberately lacks matching foreign
        // income sources. Ordinary Save accepts it and withholds the full
        // estimate; ignoring display echoes must preserve that exact result.
        let expected_estimate = estimate(&expected);
        assert_eq!(expected_estimate.state, ResultState::Unsupported);
        assert!(expected_estimate.issues.iter().any(|issue| issue.display.key
            == "ISSUE_FOREIGN_SOURCE_RECORDS_MUST_RECONCILE_WITH_ENTERED_FOREIGN_PAY_AND_298ECA4C"));
        let expected_tax = expected_estimate.tax;
        let mut request = guide;
        if forged {
            forge_more_credits_output(&mut request);
        } else {
            clear_more_credits_output(&mut request);
        }
        let request = P::from_avro(&request.to_avro()).unwrap();
        accepted(&send(&mut session, request));
        assert_eq!(session.draft, expected);
        assert_eq!(estimate(&session.draft).tax, expected_tax);
        assert_eq!(estimate(&session.draft), expected_estimate);
        super::unit47_text_size::assert_country_source(&session);

        let mut reopened = super::unit47_text_size::country_reopened(&mut session);
        let P::SaveMoreCredits { estimate, progress, .. } = &reopened else { unreachable!() };
        assert!(estimate.as_ref().is_some_and(|value| !value.line.is_empty()));
        assert!(progress.as_ref().is_some_and(|value| !value.is_empty()));
        clear_more_credits_output(&mut reopened);
        accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(session.draft, expected);
        let mut restored = restart(expected.clone());
        let mut reopened = super::unit47_text_size::country_reopened(&mut restored);
        clear_more_credits_output(&mut reopened);
        accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(restored.draft, expected);
        super::unit47_text_size::assert_country_source(&restored);
    }
}

#[test]
fn unit47_output_only_echoes_cannot_hide_an_actual_input_refusal() {
    let (mut session, _, mut correction) = super::unit47_text_size::country_fixture();
    forge_more_credits_output(&mut correction);
    let mut request = correction.clone();
    let P::SaveMoreCredits { value, .. } = &mut request else { unreachable!() };
    let MoreCredit::ForeignTax(crate::ForeignTaxPage::Ordinary { sources, .. }) =
        &mut value.credits[1]
    else {
        unreachable!()
    };
    sources[1].country = "ABC".into();
    survey_native_refusal(
        "output-only-foreign-country-input",
        &mut session,
        request,
        "FOREIGN_COUNTRY_CODE",
        "[value/credits/1/sources/1/country]",
    );
    accepted(&send(&mut session, P::from_avro(&correction.to_avro()).unwrap()));
    super::unit47_text_size::assert_country_source(&session);
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let mut reopened = super::unit47_text_size::country_reopened(&mut restored);
    clear_more_credits_output(&mut reopened);
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, saved);
}

fn ltc_preview(page: &P, row: &IncomeDocumentItem) -> P {
    if let P::SaveIncomes { people, allowed_injured_source_facts, ltc_payee_sources, .. } = page {
        let mut request =
            crate::tests::support::client::survey::survey_preview_document(row.document.clone());
        let P::PreviewDocument {
            id,
            owner,
            community,
            people: target_people,
            allowed_injured_source_facts: allowed,
            ltc_payee_sources: sources,
            ..
        } = &mut request
        else {
            unreachable!()
        };
        *id = row.id;
        *owner = row.owner.owned();
        *community = row.community;
        *target_people = people.clone();
        *allowed = allowed_injured_source_facts.clone();
        *sources = ltc_payee_sources.clone();
        return request;
    }
    page.clone()
}

fn ltc_context_fixture(recovery: bool) -> (Interview, P, i64, i64, crate::LtcSource) {
    let mut draft = complete(FederalStatus::Joint);
    let people = draft.people.as_mut().unwrap();
    people.taxpayer.first_name = "Alex".into();
    people.spouse.as_mut().unwrap().first_name = "Jordan".into();
    name_accepted_income_documents(&mut draft);
    let mut session = Interview::new(TEST_USER, draft);
    let mut annuity = r1099(90_000, DistributionCode::Nonqualified, false);
    annuity.source_mut().form.taxable = Some(40_000);
    annuity.source_mut().form.recipient =
        accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
    let reply = crate::tests::support::client::save_retirement_fixture(
        &mut session,
        vec![document(
            0,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(annuity)),
            },
        )],
    );
    accepted(&reply);
    let annuity_id = session
        .draft
        .income
        .iter()
        .find_map(|entry| {
            matches!(entry.income, FederalIncome::Retirement { .. }).then_some(entry.id)
        })
        .unwrap();
    let request = save_topic(
        &mut session,
        TaxSection::OtherIncome,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::OtherIncome(OtherIncomeItem {
                    amount: 30_000,
                    withholding: 0,
                    kind: OtherIncomeKind::Recovery {
                        worksheet: PriorYearRecovery {
                            year: 2025,
                            deducted: 20_000,
                            itemized: 2_000_000,
                            standard: 1_900_000,
                            taxable_income: 4_000_000,
                            ordinary: true,
                        },
                    },
                }),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let recovery_id = session
        .draft
        .income
        .iter()
        .find_map(|entry| {
            matches!(&entry.income, FederalIncome::OtherIncome { data }
            if matches!(data.kind, OtherIncomeKind::Recovery { .. }))
            .then_some(entry.id)
        })
        .unwrap();
    assert_ne!(annuity_id, recovery_id);
    assert_eq!(value(&supported(&session.draft), "agi"), 100_600);
    let treatment = if recovery {
        NonqualifiedCareTreatment::AccidentHealth {
            funding: CareBenefitFunding::AfterTax,
            medical_reimbursement: 0,
            permanent_injury: None,
            recovery: Some(recovery_id),
        }
    } else {
        NonqualifiedCareTreatment::LifeAnnuity { source: annuity_id }
    };
    let printed = Some(accepted_document_names(&session.draft, Owner::Taxpayer));
    let source = crate::LtcSource {
interest: None,
        form: crate::Ltc1099Paper {
            payer: printed.as_ref().and_then(|names| names.issuer.clone()).unwrap_or_default(),
            recipient: printed.as_ref().map(|names| names.recipient.clone()).unwrap_or_default(),
            insured_party: Default::default(),
            date_certified: None,
            benefits: Some(90_000),
            accelerated: Some(0),
            paid: LtcPaidBasis::NotChecked,
            box4_qualified: false,
            illness: Some(LtcIllness::Chronic),
            calendar_year: 2026,
            corrected: false,
            payer_phone: String::new(),
            account_number: String::new(),
        },
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        contract: LtcContract::Nonqualified { treatment },
        insured: None,
    };
    let mut page = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Ltc1099(ltc_document(source.clone())),
            },
        )],
    );
    let P::SaveDocuments { documents, annuity_sources, recovery_sources, .. } = &mut page else {
        unreachable!()
    };
    assert_eq!(documents.other.other_income.documents.len(), 1);
    assert!(
        annuity_sources
            .iter()
            .any(|source| source.id == annuity_id && source.owner == Owner::Taxpayer)
    );
    assert!(
        recovery_sources
            .iter()
            .any(|source| source.id == recovery_id && source.owner == Owner::Taxpayer)
    );
    (session, page, annuity_id, recovery_id, source)
}

#[test]
fn unit47_ltc_contextual_sources_use_actual_owner_and_page_paths_in_all_three_requests() {
    for case in [
        "annuity-missing",
        "annuity-wrong-kind",
        "annuity-wrong-owner",
        "annuity-above-gross",
        "recovery-missing",
        "recovery-wrong-kind",
        "local-no-amount",
        "insured-period",
    ] {
        let recovery = case.starts_with("recovery");
        let (mut session, mut correction, annuity_id, recovery_id, good_source) =
            ltc_context_fixture(recovery);
        let original_tax = supported(&session.draft).tax;
        let insured_index = if case == "insured-period" {
            let page = open_topic(&mut session, TaxSection::LongTermCareBenefits);
            let mut records = page_documents(&page);
            let index = records.len();
            records.push(document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::LtcInsured(LtcInsured {
                        insured: LtcInsuredPerson::OnReturn { person: -1 },
                        periods: LtcPeriodsInput::One {
                            starts: 20260201,
                            ends: 20260228,
                            services: 0,
                            reimbursements: 0,
                            recipient: CareBenefitRecipient::Insured { other_payees: false },
                        },
                    }),
                },
            ));
            correction = save_topic(&mut session, TaxSection::LongTermCareBenefits, records);
            Some(index)
        } else {
            None
        };
        let mut invalid = correction.clone();
        let mut invalid_source = good_source.clone();
        let mut key = "LTC1099_SOURCE_UNAVAILABLE";
        let mut local_path =
            if recovery { "contract/treatment/recovery" } else { "contract/treatment/source" };
        if case == "insured-period" {
            key = "LTC_PERIOD_ENDS_BEFORE_START";
            local_path = "periods/ends";
            let P::SaveIncomes { documents, .. } = &mut invalid else { unreachable!() };
            let crate::IncomeRecord::LtcInsured(record) =
                &mut documents.documents[insured_index.unwrap()].document
            else {
                unreachable!()
            };
            let LtcPeriodsInput::One { ends, .. } = &mut record.periods else { unreachable!() };
            *ends = 20260131;
        } else {
            let P::SaveDocuments { documents, .. } = &mut invalid else { unreachable!() };
            let target = &mut documents.other.other_income.documents[0];
            let crate::OtherIncomeDocument::Ltc1099(form) = &mut target.document else {
                unreachable!()
            };
            match case {
                "annuity-wrong-owner" => {
                    target.owner = crate::DocumentOwner::Spouses;
                    form.recipient =
                        accepted_document_names(&session.draft, Owner::Spouse).recipient;
                    invalid_source.form.recipient = form.recipient.clone();
                }
                "annuity-above-gross" => {
                    form.benefits = Some(form.benefits.unwrap_or(0) + 1);
                    invalid_source.form.benefits = form.benefits;
                }
                "local-no-amount" => {
                    form.benefits = Some(0);
                    invalid_source.form.benefits = Some(0);
                    key = "LTC1099_NO_AMOUNT";
                    local_path = "benefits";
                    invalid_source.contract = LtcContract::Nonqualified {
                        treatment: NonqualifiedCareTreatment::LifeAnnuity { source: i64::MAX },
                    };
                }
                "annuity-missing" | "annuity-wrong-kind" => {
                    invalid_source.contract = LtcContract::Nonqualified {
                        treatment: NonqualifiedCareTreatment::LifeAnnuity {
                            source: if case == "annuity-missing" { i64::MAX } else { recovery_id },
                        },
                    };
                }
                "recovery-missing" | "recovery-wrong-kind" => {
                    invalid_source.contract = LtcContract::Nonqualified {
                        treatment: NonqualifiedCareTreatment::AccidentHealth {
                            funding: CareBenefitFunding::AfterTax,
                            medical_reimbursement: 0,
                            permanent_injury: None,
                            recovery: Some(if case == "recovery-missing" {
                                i64::MAX
                            } else {
                                annuity_id
                            }),
                        },
                    };
                }
                _ => unreachable!(),
            }
        }
        if case != "insured-period" {
            let P::SaveDocuments { documents, .. } = &mut invalid else { unreachable!() };
            documents.other.other_income.documents[0].document =
                crate::OtherIncomeDocument::Ltc1099(ltc_document(invalid_source.clone()));
        }
        let row = page_documents(&invalid)
            .into_iter()
            .find(|row| {
                matches!(
                    row.document,
                    IncomeSource::OtherIncome { .. }
                        | IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
                )
            })
            .unwrap();
        let local = if case == "insured-period" {
            crate::care_benefit_return::document_issue(
                &session.draft,
                row.owner.owned(),
                &row.document,
            )
        } else {
            invalid_source.record().character_issue(&session.draft, row.owner.owned())
        }
        .unwrap();
        assert_eq!(local.key, key);
        assert_eq!(local.field.unwrap().square(), alloc::format!("[{local_path}]"));
        let prefix = if case == "insured-period" {
            alloc::format!("documents/documents/{}", insured_index.unwrap())
        } else {
            "documents/other/other_income/documents/0".into()
        };
        let mut candidate = restart(session.draft.clone());
        let (invalid, path) = if case != "insured-period" && case != "local-no-amount" {
            let desired = page_documents(&invalid);
            accepted(&send(&mut candidate, invalid));
            // Register this source's actual answers at the moment its topic is built.
            let _ = ltc_document(invalid_source);
            let request =
                crate::tests::support::client::income_pages::information_return_answers_for_topic(
                    &mut candidate,
                    &desired,
                    TaxSection::LongTermCareBenefits,
                )
                .unwrap();
            let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
            let index = followups
                .iter()
                .position(|row| matches!(row.answer, crate::DocumentAnswer::LtcContract { .. }))
                .unwrap();
            let path = local_path
                .strip_prefix("contract/")
                .unwrap()
                .split('/')
                .filter(|segment| !segment.is_empty())
                .fold(
                    SchemaPath::root()
                        .field("followups")
                        .key(followups[index].key.as_str())
                        .field("answer")
                        .field("value"),
                    |path, segment| match segment.parse::<usize>() {
                        Ok(index) => path.index(index),
                        Err(_) => path.field(segment),
                    },
                )
                .to_string();
            (request, path)
        } else {
            (invalid, alloc::format!("[{prefix}/document/{local_path}]"))
        };
        for (endpoint, request) in [
            ("save", invalid.clone()),
            ("estimate", estimate_documents_from_save(&invalid)),
            (
                "preview",
                if matches!(invalid, P::SaveIncomes { .. }) {
                    estimate_documents_from_save(&invalid)
                } else {
                    ltc_preview(&invalid, &row)
                },
            ),
        ] {
            survey_native_refusal(
                &alloc::format!("core-ltc-{case}-{endpoint}"),
                &mut candidate,
                request,
                key,
                &path,
            );
        }
        let _ = ltc_document(good_source);
        // The visible source correction uses the actual independently saved
        // annuity/recovery. Its income remains with that owner, exactly once.
        let before = session.draft.clone();
        let row = page_documents(&correction)
            .into_iter()
            .find(|row| {
                matches!(
                    row.document,
                    IncomeSource::OtherIncome { .. }
                        | IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
                )
            })
            .unwrap();
        accepted(&crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            P::from_avro(&ltc_preview(&correction, &row).to_avro()).unwrap(),
        ));
        accepted(&crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            P::from_avro(&estimate_documents_from_save(&correction).to_avro()).unwrap(),
        ));
        assert_eq!(session.draft, before);
        accepted(&if matches!(correction, P::SaveDocuments { .. }) {
            crate::tests::support::client::income_pages::save_source_fixture_request(
                &mut session,
                P::from_avro(&correction.to_avro()).unwrap(),
            )
        } else {
            send(&mut session, P::from_avro(&correction.to_avro()).unwrap())
        });
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 100_600);
        assert_eq!(value(&result, "recovery_2025_taxable"), 200);
        if case != "insured-period" {
            assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
        }
        assert_eq!(result.tax, original_tax);
        let saved = session.draft.clone();
        let reopened = if case == "insured-period" {
            open_topic(&mut session, TaxSection::LongTermCareBenefits)
        } else {
            open_documents(&mut session)
        };
        assert_eq!(
            page_documents(&reopened)
                .into_iter()
                .find(|row| matches!(
                    row.document,
                    IncomeSource::OtherIncome { .. }
                        | IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
                ))
                .unwrap()
                .document,
            vec![row.document.clone()][0]
        );
        accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = if case == "insured-period" {
            open_topic(&mut restored, TaxSection::LongTermCareBenefits)
        } else {
            open_documents(&mut restored)
        };
        assert_eq!(
            page_documents(&reopened)
                .into_iter()
                .find(|row| matches!(
                    row.document,
                    IncomeSource::OtherIncome { .. }
                        | IncomeSource::Record { value: crate::IncomeRecord::LtcInsured(_) }
                ))
                .unwrap()
                .document,
            vec![row.document.clone()][0]
        );
        accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(restored.draft, saved);
        assert_eq!(supported(&restored.draft), result);
    }
}

#[test]
fn unit47_ltc_shared_annuity_gross_is_checked_after_individual_source_admission() {
    let (mut session, mut correction, _, _, good_source) = ltc_context_fixture(false);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    let crate::OtherIncomeDocument::Ltc1099(first) =
        &mut documents.other.other_income.documents[0].document
    else {
        unreachable!()
    };
    first.benefits = Some(60_000);
    let mut first_source = good_source.clone();
    first_source.form.benefits = Some(60_000);
    *first = ltc_document(first_source.clone());
    let mut second_source = good_source;
    second_source.form.benefits = Some(30_000);
    let second = ltc_document(second_source.clone());
    let mut second_row = documents.other.other_income.documents[0].clone();
    second_row.id = 0;
    second_row.document = crate::OtherIncomeDocument::Ltc1099(second);
    documents.other.other_income.documents.push(second_row);
    let mut invalid = correction.clone();
    let P::SaveDocuments { documents, .. } = &mut invalid else { unreachable!() };
    let crate::OtherIncomeDocument::Ltc1099(second) =
        &mut documents.other.other_income.documents[1].document
    else {
        unreachable!()
    };
    second.benefits = Some(second.benefits.unwrap_or(0) + 1);
    let mut invalid_source = second_source.clone();
    invalid_source.form.benefits = Some(invalid_source.form.benefits.unwrap_or(0) + 1);
    *second = ltc_document(invalid_source);
    for row in &documents.other.other_income.documents[..] {
        assert!(
            crate::care_benefit_return::document_issue(
                &session.draft,
                row.owner.owned(),
                &row.clone().into_income().document
            )
            .is_none()
        );
    }
    let desired = page_documents(&invalid);
    let mut candidate = restart(session.draft.clone());
    accepted(&send(&mut candidate, invalid));
    let invalid =
        crate::tests::support::client::income_pages::information_return_answers_for_topic(
            &mut candidate,
            &desired,
            TaxSection::LongTermCareBenefits,
        )
        .unwrap();
    let P::SaveIncomes { followups, .. } = &invalid else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| matches!(row.answer, crate::DocumentAnswer::LtcContract { .. }))
        .unwrap();
    let path = SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value")
        .field("treatment")
        .field("source")
        .to_string();
    for (endpoint, request) in
        [("save", invalid.clone()), ("estimate", estimate_documents_from_save(&invalid))]
    {
        survey_native_refusal(
            &alloc::format!("core-ltc-shared-gross-{endpoint}"),
            &mut candidate,
            request,
            "LTC1099_SOURCE_UNAVAILABLE",
            &path,
        );
    }
    let _ = ltc_document(first_source);
    let _ = ltc_document(second_source);
    accepted(&if matches!(correction, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            P::from_avro(&correction.to_avro()).unwrap(),
        )
    } else {
        send(&mut session, P::from_avro(&correction.to_avro()).unwrap())
    });
    assert_eq!(value(&supported(&session.draft), "agi"), 100_600);
    assert_eq!(value(&supported(&session.draft), "taxable_ltc_benefits"), 0);
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_documents(&mut restored);
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_ltc_unresolved_character_remains_accepted_with_an_unavailable_estimate() {
    let (mut session, mut request, _, _, mut source) = ltc_context_fixture(false);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let crate::OtherIncomeDocument::Ltc1099(form) =
        &mut documents.other.other_income.documents[0].document
    else {
        unreachable!()
    };
    source.contract =
        LtcContract::Nonqualified { treatment: NonqualifiedCareTreatment::Unclassified };
    *form = ltc_document(source);
    assert!(
        crate::care_benefit_return::document_issue(
            &session.draft,
            Owner::Taxpayer,
            &documents.other.other_income.documents[0].clone().into_income().document
        )
        .is_none()
    );
    accepted(&if matches!(request, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            P::from_avro(&request.to_avro()).unwrap(),
        )
    } else {
        send(&mut session, P::from_avro(&request.to_avro()).unwrap())
    });
    let result = estimate(&session.draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED")
    );
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_documents(&mut restored);
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_ltc_recovery_context_uses_whole_page_answers_and_bound_preview_source() {
    let (mut session, guide, _, recovery_id, _) = ltc_context_fixture(true);
    let mut zero_source = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes { documents, .. } = &mut zero_source else { unreachable!() };
    let crate::IncomeRecord::OtherIncome(recovery) = &mut documents.documents[0].document else {
        unreachable!()
    };
    recovery.amount = 0;
    accepted(&send(&mut session, P::from_avro(&zero_source.to_avro()).unwrap()));
    assert_eq!(value(&supported(&session.draft), "agi"), 100_400);
    let mut correction = open_topic(&mut session, TaxSection::OtherIncome);
    let source_receipt = page_documents(&guide)
        .into_iter()
        .find(|row| matches!(row.document, IncomeSource::OtherIncome { .. }))
        .unwrap();
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    assert_eq!(documents.documents[0].id, recovery_id);
    let crate::IncomeRecord::OtherIncome(recovery) = &mut documents.documents[0].document else {
        unreachable!()
    };
    recovery.amount = 30_000;
    // The LTC topic calculation reads the accepted recovery, so an unsent
    // edit on Other income cannot supply it.
    let before = session.draft.clone();
    let mut candidate = restart(before.clone());
    let desired = vec![source_receipt.clone()];
    let request = save_documents(&mut candidate, desired.clone());
    accepted(&send(&mut candidate, request));
    let request =
        crate::tests::support::client::income_pages::information_return_answers_for_topic(
            &mut candidate,
            &desired,
            TaxSection::LongTermCareBenefits,
        )
        .unwrap();
    let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| matches!(row.answer, crate::DocumentAnswer::LtcContract { .. }))
        .unwrap();
    survey_native_refusal(
        "core-ltc-bound-zero-recovery-preview",
        &mut candidate,
        estimate_documents_from_save(&request),
        "LTC1099_SOURCE_UNAVAILABLE",
        &SchemaPath::root()
            .field("followups")
            .key(followups[index].key.as_str())
            .field("answer")
            .field("value")
            .field("treatment")
            .field("recovery")
            .to_string(),
    );
    assert_eq!(session.draft, before);
    let saved = session.draft.clone();
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        P::from_avro(&estimate_documents_from_save(&correction).to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, saved);
    accepted(&if matches!(correction, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            P::from_avro(&correction.to_avro()).unwrap(),
        )
    } else {
        send(&mut session, P::from_avro(&correction.to_avro()).unwrap())
    });
    // Accept the independently edited recovery before the source receipt refers to it.
    let correction = save_documents(&mut session, vec![source_receipt]);
    let saved = session.draft.clone();
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        estimate_documents_from_save(&correction),
    ));
    assert_eq!(session.draft, saved);
    accepted(&if matches!(correction, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(
            &mut session,
            P::from_avro(&correction.to_avro()).unwrap(),
        )
    } else {
        send(&mut session, P::from_avro(&correction.to_avro()).unwrap())
    });
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 100_600);
    assert_eq!(value(&result, "recovery_2025_taxable"), 200);
    assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_documents(&mut restored);
    let receipt = page_documents(&reopened)
        .into_iter()
        .find(|row| matches!(row.document, IncomeSource::OtherIncome { .. }))
        .unwrap();
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut restored,
        P::from_avro(&ltc_preview(&reopened, &receipt).to_avro()).unwrap(),
    ));
    assert_eq!(restored.draft, saved);
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, saved);
    assert_eq!(supported(&restored.draft), result);
}

fn trump_return(principal: i64, earnings: i64) -> TrumpExcessReturn {
    TrumpExcessReturn {
        document_id: None,
        principal,
        earnings,
        received: 20261201,
        deadline: 20270415,
        confirmed: true,
    }
}

fn trump_context_fixture() -> (Interview, P, i64) {
    let mut draft = complete(FederalStatus::Joint);
    let people = draft.people.as_mut().unwrap();
    people.taxpayer.birth_date = 20100301;
    people.spouse.as_mut().unwrap().birth_date = 20110301;
    let before_line23 = return_line(&supported(&draft), "23");
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_other_taxes(&mut session);
    let P::SaveOtherTaxes {
        value,
        estimate,
        progress,
        warnings,
        allowed_owners,
        mortgage_gift_owners,
        coverdell_corrections,
        ..
    } = &mut page
    else {
        unreachable!()
    };
    *estimate = None;
    *progress = None;
    warnings.clear();
    allowed_owners.clear();
    mortgage_gift_owners.clear();
    coverdell_corrections.clear();
    value.taxes = vec![
        OtherTax::HouseholdEmployees { employers: vec![(&household_employer(3_500)).into()] },
        OtherTax::TrumpAccounts {
            accounts: [
                // Nonzero exempt == contributions is the allowed boundary.
                TrumpAccount {
                    owner: Owner::Taxpayer,
                    contributions: 600_000,
                    exempt: 600_000,
                    returns: vec![],
                    confirmed: true,
                },
                TrumpAccount {
                    owner: Owner::Spouse,
                    contributions: 800_000,
                    exempt: 100_000,
                    returns: vec![trump_return(50_000, 2_000), trump_return(10_000, -10_000)],
                    confirmed: true,
                },
            ]
            .iter()
            .map(Into::into)
            .collect(),
        },
    ];
    (session, page, before_line23)
}

fn trump_estimate(page: &P) -> P {
    let P::SaveOtherTaxes { value, .. } = page else { unreachable!() };
    P::EstimateOtherTaxes {
        user: TEST_USER,
        allowed_owners: vec![],
        mortgage_gift_owners: vec![],
        coverdell_corrections: vec![],
        value: value.clone(),
    }
}

#[test]
fn unit47_trump_accounts_preserve_group_order_and_actual_save_estimate_paths() {
    for case in [
        "exempt",
        "return-loss",
        "duplicate-owner",
        "exempt-before-earlier-loss",
        "loss-before-duplicate",
        "exempt-before-loss-and-duplicate",
    ] {
        let (mut session, correction, before_line23) = trump_context_fixture();
        let mut request = correction.clone();
        let P::SaveOtherTaxes { value: taxes, .. } = &mut request else { unreachable!() };
        let OtherTax::TrumpAccounts { accounts } = &mut taxes.taxes[1] else { unreachable!() };
        match case {
            "exempt" => accounts[1].exempt = accounts[1].contributions + 1,
            "return-loss" => accounts[1].returns[1].earnings -= 1,
            "duplicate-owner" => accounts[1].owner = Owner::Taxpayer,
            "exempt-before-earlier-loss" => {
                accounts[0].returns = vec![trump_return(10_000, -10_001)];
                accounts[1].exempt = accounts[1].contributions + 1;
            }
            "loss-before-duplicate" => {
                accounts[1].returns[1].earnings -= 1;
                accounts[1].owner = Owner::Taxpayer;
            }
            "exempt-before-loss-and-duplicate" => {
                accounts[1].exempt = accounts[1].contributions + 1;
                accounts[1].returns[1].earnings -= 1;
                accounts[1].owner = Owner::Taxpayer;
            }
            _ => unreachable!(),
        }
        let (key, local_path) = if case.starts_with("exempt") {
            ("TRUMP_EXEMPT_ABOVE_CONTRIBUTIONS", "accounts/1/exempt")
        } else if case.starts_with("return-loss") || case.starts_with("loss") {
            ("TRUMP_LOSS_ABOVE_PRINCIPAL", "accounts/1/returns/1/earnings")
        } else {
            ("TRUMP_ACCOUNT_TWICE", "accounts/1/owner")
        };
        let native_accounts: Vec<TrumpAccount> = accounts.iter().map(Into::into).collect();
        let issue = crate::trump_account::accounts_issue(&native_accounts).unwrap();
        assert_eq!(issue.key, key);
        assert_eq!(issue.field.unwrap().square(), alloc::format!("[{local_path}]"));
        for (endpoint, submitted) in
            [("save", request.clone()), ("estimate", trump_estimate(&request))]
        {
            survey_native_refusal(
                &alloc::format!("core-trump-{case}-{endpoint}"),
                &mut session,
                submitted,
                key,
                &alloc::format!("[value/taxes/1/{local_path}]"),
            );
        }
        let before = session.draft.clone();
        accepted(&crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            P::from_avro(&trump_estimate(&correction).to_avro()).unwrap(),
        ));
        assert_eq!(session.draft, before);
        accepted(&send(&mut session, P::from_avro(&correction.to_avro()).unwrap()));
        let result = supported(&session.draft);
        // The taxpayer is at the nonzero exempt/contribution boundary.
        // Spouse excess: 8000-1000-5000=2000; timely principal 500+100
        // leaves 1400, at 6% = 84. Positive earnings 20 add 20; the second
        // return's loss equals principal and creates no negative tax.
        assert_eq!(value(&result, "trump_account_taxpayer_line61_tax"), 0);
        assert_eq!(value(&result, "trump_account_spouse_line61_tax"), 84);
        assert_eq!(value(&result, "trump_account_spouse_line63_tax"), 20);
        assert_eq!(value(&result, "household_0_tax"), 557);
        assert_eq!(return_line(&result, "23") - before_line23, 66_100);
        let saved = session.draft.clone();
        let reopened = open_other_taxes(&mut session);
        let P::SaveOtherTaxes { value: shown, .. } = &reopened else { unreachable!() };
        let P::SaveOtherTaxes { value: expected, .. } = &correction else { unreachable!() };
        assert_eq!(shown, expected);
        accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = open_other_taxes(&mut restored);
        let P::SaveOtherTaxes { value: shown, .. } = &reopened else { unreachable!() };
        assert_eq!(shown, expected);
        accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
        assert_eq!(restored.draft, saved);
        assert_eq!(supported(&restored.draft), result);
    }
}

#[test]
fn unit47_nested_benefit_names_are_output_only_without_emptying_editable_rows() {
    let partners =
        partner_benefits_session(vec![k1_1065_benefits("Synthetic display-only partner", 0, 0, 0)]);
    let (corporations, reply) = save_k1_1120s(vec![k1_1120s("Synthetic display-only corporation")]);
    accepted(&reply);
    for (kind, mut session) in
        [("business", business_session()), ("partner", partners), ("corporation", corporations)]
    {
        let request = benefits_editor(&mut session);
        accepted(&send(&mut session, request));
        let baseline = session.draft.clone();
        let expected_estimate = estimate(&baseline);
        for forged in [false, true] {
            let mut request = benefits_editor(&mut session);
            let P::SaveSelfEmployedBenefits { value, .. } = &mut request else { unreachable!() };
            let names: Vec<_> = match kind {
                "business" => value.businesses.iter_mut().map(|r| &mut r.name).collect(),
                "partner" => value.partnerships.iter_mut().map(|r| &mut r.name).collect(),
                "corporation" => value.scorporations.iter_mut().map(|r| &mut r.name).collect(),
                _ => unreachable!(),
            };
            assert!(!names.is_empty());
            for name in names {
                assert!(!name.is_empty(), "the reopened display is still supplied");
                *name = if forged { vec![255, 254, 253] } else { vec![] };
            }
            let request = P::from_avro(&request.to_avro()).unwrap();
            accepted(&send(&mut session, request));
            assert_eq!(session.draft, baseline, "{kind}, forged={forged}");
            assert_eq!(estimate(&session.draft), expected_estimate);
            session = restart(session.draft.clone());
            assert_eq!(session.draft, baseline);
        }
    }
}

#[test]
fn unit47_nested_income_displays_cannot_replace_actual_document_inputs() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w2(10_000_000))) },
        )],
    );
    accepted(&send(&mut session, request));
    let baseline = session.draft.clone();
    let expected_estimate = estimate(&baseline);
    for forged in [false, true] {
        let mut request = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        assert_eq!(documents.common.wages.documents.len(), 1);
        let item = &mut documents.common.wages.documents[0];
        item.investment_caps = forged.then_some(crate::InvestmentCapitalCap {
            short: 99_999_999,
            long: 99_999_999,
            amt_short: 99_999_999,
            amt_long: 99_999_999,
        });
        item.preview = forged.then_some(i64::MAX);
        item.allowed_community = if forged { vec![i32::MAX] } else { vec![] };
        let request = P::from_avro(&request.to_avro()).unwrap();
        accepted(&send(&mut session, request));
        assert_eq!(session.draft, baseline, "forged={forged}");
        assert_eq!(estimate(&session.draft), expected_estimate);
        session = restart(session.draft.clone());
        assert_eq!(session.draft, baseline);
    }
}

fn unit47_admission_estimate(request: &P) -> P {
    match request {
        P::SaveDocuments { .. } => request.clone(),
        P::SaveStudentLoan { value, .. } => {
            P::EstimateStudentLoan { user: TEST_USER, value: value.clone() }
        }
        P::SaveTaxes { value, .. } => P::EstimateTaxes { user: TEST_USER, value: value.clone() },
        P::SaveMortgages { value, .. } => P::EstimateMortgages {
            user: TEST_USER,
            mortgage_loans: vec![],
            rental_mortgage_loans: vec![],
            value: value.clone(),
        },
        P::SaveOtherItemized { value, .. } => P::EstimateOtherItemized {
            user: TEST_USER,
            allowed_owners: vec![],
            interest_sources: vec![],
            investment_k1_sources: vec![],
            investment_origin_choices: vec![],
            value: value.clone(),
        },
        _ => panic!("unexpected itemized admission request"),
    }
}

fn unit47_admission_correct_estimate(session: &mut Interview, request: &P) {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let request = unit47_admission_estimate(request);
    let request = P::from_avro(&request.to_avro()).unwrap();
    let reply = if matches!(request, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::candidate_save_review(session, request)
    } else {
        send(session, request)
    };
    assert!(matches!(reply, Reply::Response(P::RunningEstimate { .. } | P::Review { .. })));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn unit47_student_interest_admission_uses_all_actual_form_interest_and_reopens_the_correction() {
    for (case, forms, other, employer, correction) in [
        ("student-interest-additional", vec![box_2(180_000, 20_000)], 10_000, 210_001, 210_000),
        ("student-interest-no-form", vec![], 10_000, 10_001, 10_000),
        ("student-interest-zero", vec![], 0, 1, 0),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let first_form_id = session.draft.next_id;
        let mut source_page = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut source_page else { unreachable!() };
        documents.common.student_loans.documents = student_documents(&forms);
        accepted(&send(&mut session, source_page));
        let forms: Vec<_> = forms
            .into_iter()
            .enumerate()
            .map(|(index, mut form)| {
                form.id = first_form_id + index as i64;
                // Saved without a printed borrower, each form waits for its owner.
                form.assigned = crate::DocumentRecipient::Unknown;
                form.owner_source = None;
                form
            })
            .collect();
        let navigation = go_to(&session.draft, PAGE_REVIEW, TaxSection::StudentLoan);
        let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(&mut session, navigation)
        else {
            panic!("expected actual student-loan editor")
        };
        let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
        value.other_interest = other;
        value.employer_paid_interest = employer;
        for (row, form) in value.forms.iter_mut().zip(&forms) {
            if form.document.box_2 {
                row.additional_interest = crate::StudentLoanAdditionalInterest::Checked {
                    additional_interest: form.additional_interest,
                };
            }
        }
        survey_native_refusal(
            &alloc::format!("{case}-save"),
            &mut session,
            request.clone(),
            "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST",
            "[value/employer_paid_interest]",
        );
        survey_native_refusal(
            &alloc::format!("{case}-estimate"),
            &mut session,
            unit47_admission_estimate(&request),
            "STUDENT_LOAN_EMPLOYER_ABOVE_INTEREST",
            "[value/employer_paid_interest]",
        );
        let P::SaveStudentLoan { value, .. } = &mut request else { unreachable!() };
        value.employer_paid_interest = correction;
        let mut expected = value.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        let request = P::from_avro(&request.to_avro()).unwrap();
        accepted(&send(&mut session, request));
        assert_eq!(
            session.draft.adjustments.as_ref().unwrap().student_loan,
            StudentLoanClaim {
                student_loan_interest: other,
                forms: forms.clone(),
                employer_paid_interest: correction,
            }
        );
        for row in &mut expected.forms {
            row.value = session
                .draft
                .adjustments
                .as_ref()
                .unwrap()
                .student_loan
                .forms
                .iter()
                .find(|form| form.id == row.id)
                .unwrap()
                .document
                .clone();
        }
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let navigation = go_to(&restored.draft, PAGE_REVIEW, TaxSection::StudentLoan);
        let Reply::Edit(request @ P::SaveStudentLoan { .. }) = send(&mut restored, navigation)
        else {
            unreachable!()
        };
        let P::SaveStudentLoan { value, .. } = &request else { unreachable!() };
        assert_eq!(*value, expected);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_investment_income_tax_admission_preserves_election_and_paid_part() {
    for (case, kind, paid_part, path) in [
        (
            "investment-tax-amount",
            SaltElection::IncomeTax { paid: 50_000 },
            50_001,
            "[value/investment_income_tax/paid_part]",
        ),
        (
            "investment-tax-election",
            SaltElection::SalesTax { paid: 50_000 },
            0,
            "[value/investment_income_tax]",
        ),
    ] {
        let mut session = at_the_deductions_page(FederalStatus::Single).0;
        session.draft.checklist.paid |= PaidTopics::TAXES.0;
        let mut request = open_itemized_topic(&mut session, TaxSection::Taxes);
        let P::SaveTaxes { value, .. } = &mut request else { unreachable!() };
        value.kind = kind;
        value.investment_income_tax = crate::InvestmentIncomeTax::Records {
            paid_part,
            capped_allocation: crate::InvestmentTaxCapAllocation::SourceDetermined,
        };
        survey_native_refusal(
            &alloc::format!("{case}-save"),
            &mut session,
            request.clone(),
            "INVESTMENT_TAX_ABOVE_PAID",
            path,
        );
        survey_native_refusal(
            &alloc::format!("{case}-estimate"),
            &mut session,
            unit47_admission_estimate(&request),
            "INVESTMENT_TAX_ABOVE_PAID",
            path,
        );
        let P::SaveTaxes { value, .. } = &mut request else { unreachable!() };
        value.kind = SaltElection::IncomeTax { paid: 50_000 };
        value.investment_income_tax = crate::InvestmentIncomeTax::Records {
            paid_part: 50_000,
            capped_allocation: crate::InvestmentTaxCapAllocation::SourceDetermined,
        };
        let expected = value.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        assert_eq!(stored_facts(&session).investment_income_tax, expected.investment_income_tax);
        assert_eq!(stored_facts(&session).personal_tax.paid(), 50_000);
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_itemized_topic(&mut restored, TaxSection::Taxes);
        let P::SaveTaxes { value, .. } = &request else { unreachable!() };
        assert_eq!(*value, expected);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

fn unit47_mortgage_row(label: &str) -> Mortgage1098 {
    Mortgage1098 {
        lender: Default::default(),
        borrower: Default::default(),
        id: 0,
        interest: 100_000,
        principal: 20_000_000,
        originated: 20200601,
        refund: 0,
        insurance: 0,
        points: 0,
        more: MortgageMore {
            property: label.into(),
            details: MortgageDetails::FirstLender(MortgageFirstLender {
                home: MortgageHome::Main,
                purpose: MortgageUse::Home,
                average: AverageBalance::FirstAndLast { last_balance: 20_000_000 },
            }),
            ..MortgageMore::default()
        },
    }
}

fn unit47_mortgage_session() -> (Interview, Vec<Mortgage1098>) {
    let mut session = at_the_deductions_page(FederalStatus::Single).0;
    // This $3,000 interest fixture is below the standard deduction. Select
    // Itemize through its actual editor so Schedule A claims the tested amount.
    unit47_admission_itemize(&mut session);
    assert_eq!(session.draft.deductions.as_ref().unwrap().choice(), DeductionChoice::Itemized);
    let mut rows = vec![
        unit47_mortgage_row("First source"),
        unit47_mortgage_row("Second source"),
        unit47_mortgage_row("Third source"),
    ];
    let borrower = accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
    for row in &mut rows {
        row.borrower = borrower.clone();
    }
    let mut request = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.mortgages.documents = rows.iter().cloned().map(mortgage_document).collect();
    accepted(&send(&mut session, request));
    accepted(&crate::tests::cross::interview::save_mortgage_answers(
        &mut session,
        &rows
            .iter()
            .map(|row| (row.more.clone(), crate::MortgagePointTreatment::None))
            .collect::<Vec<_>>(),
    ));
    for (row, source) in rows.iter_mut().zip(&session.draft.deductions.as_ref().unwrap().mortgage) {
        row.id = source.id;
    }
    assert_eq!(value(&estimate(&session.draft), "mortgage_interest_8a"), 3_000);
    (session, rows)
}

#[test]
fn unit47_mortgage_admission_preserves_native_order_source_ids_and_form1098_paths() {
    for case in [
        "last-balance",
        "purchase-points",
        "improvement-points",
        "refinance-date",
        "refinance-term",
        "borrowed-balance",
        "mixed-average",
        "month-duplicate",
        "refund-origin",
        "refund-recovery",
        "linked-source-unavailable",
        "linked-self",
        "linked-missing",
        "linked-not-root",
        "linked-date",
        "linked-purchase",
        "linked-improvement",
        "duplicate-id",
        "average-limit",
        "last-before-points",
    ] {
        let (mut session, mut rows) = unit47_mortgage_session();
        if case == "linked-missing" {
            let mut setup = crate::tests::cross::interview::linked_mortgage_editor(&mut session);
            let P::SaveMortgages { value, .. } = &mut setup else { unreachable!() };
            value.loans[1].details =
                MortgageDetails::EarlierLender { loan: value.loans[0].id }.into();
            accepted(&send(&mut session, setup));
            rows[1].more.details = MortgageDetails::EarlierLender { loan: rows[0].id };
        }
        let mut request = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let original = documents.common.mortgages.clone();
        let value = &mut rows;
        assert_eq!(value.len(), 3);
        assert!(value.iter().all(|loan| loan.id > 0));
        let first_id = value[0].id;
        let second_id = value[1].id;
        let third_id = value[2].id;
        let mut point_answers = vec![crate::MortgagePointTreatment::None; 3];
        let (key, path) = match case {
            "last-balance" | "last-before-points" => {
                let MortgageDetails::FirstLender(facts) = &mut value[1].more.details else {
                    unreachable!()
                };
                facts.average = AverageBalance::FirstAndLast { last_balance: 20_000_001 };
                if case == "last-before-points" {
                    facts.home = MortgageHome::Second;
                    value[1].points = 10_000;
                    point_answers[1] = crate::MortgagePointTreatment::Purchase;
                }
                (
                    "MORTGAGE_LAST_BALANCE_ABOVE_BOX_2",
                    "documents/common/mortgages/documents/1/more/details/average/last_balance",
                )
            }
            "purchase-points" | "improvement-points" => {
                let MortgageDetails::FirstLender(facts) = &mut value[1].more.details else {
                    unreachable!()
                };
                facts.home = MortgageHome::Second;
                value[1].points = 10_000;
                point_answers[1] = if case == "purchase-points" {
                    crate::MortgagePointTreatment::Purchase
                } else {
                    crate::MortgagePointTreatment::Improvement { term_months: 120 }
                };
                (
                    if case == "purchase-points" {
                        "MORTGAGE_PURCHASE_POINTS"
                    } else {
                        "MORTGAGE_IMPROVEMENT_POINTS"
                    },
                    "documents/common/mortgages/documents/1/points",
                )
            }
            "refinance-date" | "refinance-term" => {
                let MortgageDetails::FirstLender(facts) = &mut value[1].more.details else {
                    unreachable!()
                };
                facts.purpose = MortgageUse::Refinance(MortgageRefinance {
                    refinanced: 20_000_000,
                    earlier_incurred: if case == "refinance-date" { 20200602 } else { 20100101 },
                    earlier_term_ends: None,
                    improvement: 0,
                    other: 0,
                });
                if case == "refinance-date" {
                    (
                        "MORTGAGE_EARLIER_LOAN_AFTER_THIS_ONE",
                        "documents/common/mortgages/documents/1/more/details/purpose/earlier_incurred",
                    )
                } else {
                    (
                        "MORTGAGE_EARLIER_TERM_NEEDED",
                        "documents/common/mortgages/documents/1/more/details/purpose/earlier_term_ends",
                    )
                }
            }
            "borrowed-balance" | "mixed-average" => {
                let MortgageDetails::FirstLender(facts) = &mut value[1].more.details else {
                    unreachable!()
                };
                facts.purpose = MortgageUse::Partly(MortgageSplit {
                    improvement: 10_000_000,
                    improvement_incurred: 20200601,
                    other: 10_000_000,
                });
                if case == "borrowed-balance" {
                    value[1].principal = 20_000_001;
                    (
                        "MORTGAGE_BALANCE_ABOVE_AMOUNTS_BORROWED",
                        "documents/common/mortgages/documents/1/principal",
                    )
                } else {
                    facts.average = AverageBalance::Statements { average: 20_000_000 };
                    (
                        "MORTGAGE_MIXED_USE_NEEDS_MONTHLY",
                        "documents/common/mortgages/documents/1/more/details/average",
                    )
                }
            }
            "month-duplicate" => {
                let MortgageDetails::FirstLender(facts) = &mut value[1].more.details else {
                    unreachable!()
                };
                facts.average = AverageBalance::Monthly {
                    balances: vec![
                        MortgageMonthBalance { month: 1, balance: 20_000_000 },
                        MortgageMonthBalance { month: 1, balance: 20_000_000 },
                    ],
                };
                (
                    "MORTGAGE_MONTH_TWICE",
                    "documents/common/mortgages/documents/1/more/details/average/balances/1/month",
                )
            }
            "refund-origin" => {
                value[1].refund = 100;
                (
                    "MORTGAGE_REFUND_ORIGIN_NEEDED",
                    "documents/common/mortgages/documents/1/more/refund_origin",
                )
            }
            "refund-recovery" => {
                value[1].refund = 100;
                value[1].more.refund_origin =
                    Some(MortgageRefundOrigin::Deducted(crate::PriorYearRecovery {
                        year: 2025,
                        deducted: 100_001,
                        itemized: 100_000,
                        standard: 0,
                        taxable_income: 1_000_000,
                        ordinary: true,
                    }));
                (
                    "RECOVERY_DEDUCTED_ABOVE_ITEMIZED",
                    "documents/common/mortgages/documents/1/more/refund_origin/deducted",
                )
            }
            "linked-source-unavailable" => {
                value[1].more.details = MortgageDetails::EarlierLender { loan: i64::MAX };
                (
                    "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
                    "documents/common/mortgages/documents/1/more/details/loan",
                )
            }
            "linked-self" => {
                value[1].more.details = MortgageDetails::EarlierLender { loan: second_id };
                (
                    "MORTGAGE_LOAN_SELF_REFERENCE",
                    "documents/common/mortgages/documents/1/more/details/loan",
                )
            }
            "linked-missing" => {
                value[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                value.remove(0);
                (
                    "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
                    "documents/common/mortgages/documents/0/more/details/loan",
                )
            }
            "linked-not-root" => {
                value[0].more.details = MortgageDetails::EarlierLender { loan: third_id };
                value[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                (
                    "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
                    "documents/common/mortgages/documents/1/more/details/loan",
                )
            }
            "linked-date" => {
                value[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                value[1].originated = 20200602;
                (
                    "MORTGAGE_LOAN_ORIGINATION_DIFFERS",
                    "documents/common/mortgages/documents/1/originated",
                )
            }
            "linked-purchase" | "linked-improvement" => {
                let MortgageDetails::FirstLender(facts) = &mut value[0].more.details else {
                    unreachable!()
                };
                facts.home = MortgageHome::Second;
                value[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                value[1].points = 10_000;
                point_answers[1] = if case == "linked-purchase" {
                    crate::MortgagePointTreatment::Purchase
                } else {
                    crate::MortgagePointTreatment::Improvement { term_months: 120 }
                };
                (
                    if case == "linked-purchase" {
                        "MORTGAGE_PURCHASE_POINTS"
                    } else {
                        "MORTGAGE_IMPROVEMENT_POINTS"
                    },
                    "documents/common/mortgages/documents/1/points",
                )
            }
            "duplicate-id" => {
                value[1].id = first_id;
                (
                    "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
                    "documents/common/mortgages/documents/1/id",
                )
            }
            "average-limit" => {
                for loan in value.iter_mut() {
                    loan.principal = 30_000_000;
                    let MortgageDetails::FirstLender(facts) = &mut loan.more.details else {
                        unreachable!()
                    };
                    facts.average = AverageBalance::WithinLimit;
                }
                ("MORTGAGE_AVERAGE_NEEDED", "documents/common/mortgages/documents")
            }
            _ => unreachable!(),
        };
        let desired = value.clone();
        documents.common.mortgages.documents =
            desired.iter().cloned().map(mortgage_document).collect();
        let source_case = matches!(case, "duplicate-id" | "linked-missing");
        let mut saved_origins = Vec::new();
        if !source_case {
            // Accept only printed source changes first; the failing answer is
            // submitted afterward on the real fixed-membership topic page.
            accepted(&send(&mut session, request));
            request = crate::tests::cross::interview::linked_mortgage_editor(&mut session);
            let P::SaveMortgages { value, .. } = &mut request else { unreachable!() };
            for (index, row) in value.loans.iter_mut().enumerate() {
                let desired = desired.iter().find(|loan| loan.id == row.id).unwrap();
                row.property = desired.more.property.clone();
                // The origin is answered on the questions page; the page echoes it.
                if let Some(source) = session
                    .draft
                    .deductions
                    .as_mut()
                    .unwrap()
                    .mortgage
                    .iter_mut()
                    .find(|source| source.id == row.id)
                {
                    saved_origins.push((source.id, source.refund_origin.clone()));
                    source.refund_origin = desired.more.refund_origin.clone();
                }
                row.refund_origin = desired.more.refund_origin.clone();
                row.details = desired.more.details.clone().into();
                row.points = point_answers[index].clone();
            }
        }
        // The answers are no longer grouped under the old page's `more`.
        let path: String = if source_case {
            path.replace("/more/", "/")
        } else {
            path.replace("documents/common/mortgages/documents", "value/loans")
                .replace("/more/", "/")
        };
        // The topic row shows the printed 1098 as `value`; box 2 is the balance
        // the borrowed amounts are compared with.
        let path = if case == "borrowed-balance" {
            path.replace("/principal", "/value/box_2")
        } else if case == "linked-date" {
            path.replace("/originated", "/details")
        } else {
            path
        };
        let locator = alloc::format!("[{path}]");
        survey_native_refusal(
            &alloc::format!("mortgage-{case}-save"),
            &mut session,
            request.clone(),
            key,
            &locator,
        );
        survey_native_refusal(
            &alloc::format!("mortgage-{case}-estimate"),
            &mut session,
            unit47_admission_estimate(&request),
            key,
            &locator,
        );
        for (id, origin) in saved_origins {
            let loans = &mut session.draft.deductions.as_mut().unwrap().mortgage;
            loans.iter_mut().find(|source| source.id == id).unwrap().refund_origin = origin;
        }
        request = open_documents(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let value = &mut documents.common.mortgages;
        *value = original;
        value.documents[1].form.box_1 = 200_000;
        let expected = value.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        assert_eq!(super::value(&estimate(&session.draft), "mortgage_interest_8a"), 4_000);
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_documents(&mut restored);
        let P::SaveDocuments { documents, .. } = &request else { unreachable!() };
        let value = &documents.common.mortgages;
        assert_eq!(*value, expected);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

fn unit47_admission_itemize(session: &mut Interview) {
    let navigation = go_to(&session.draft, PAGE_REVIEW, TaxSection::Deductions);
    let Reply::Edit(mut request @ P::SaveDeductions { .. }) = send(session, navigation) else {
        panic!("expected the actual deductions editor")
    };
    let P::SaveDeductions { value, .. } = &mut request else { unreachable!() };
    value.itemize_override = Some(ItemizeOverride::Itemize);
    accepted(&send_past_less_common(session, request));
}

fn unit47_annuity_admission_session() -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut you = filer(&session.draft.people.as_ref().unwrap().taxpayer);
    you.details.more =
        MoreAboutYou::Died { date: 20261231, estate_medical: 0, bonds: None, ordinary: true };
    let request = save_you(&session.draft, you);
    accepted(&send(&mut session, request));
    let mut pension = r1099(1_000_000, normal_kept(), false);
    pension.source_mut().form.taxable = Some(700_000);
    let reply = crate::tests::support::client::save_retirement_fixture(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(pension.clone())),
                },
            ),
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(pension)),
                },
            ),
        ],
    );
    accepted(&reply);
    unit47_admission_itemize(&mut session);
    session
}

#[test]
fn unit47_annuity_admission_keeps_contract_identity_and_recovery_priority() {
    for case in [
        "duplicate",
        "recovered",
        "refunds",
        "duplicate-before-recovery",
        "recovery-before-refunds",
    ] {
        let mut session = unit47_annuity_admission_session();
        let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &mut request else { unreachable!() };
        assert_eq!(value.contracts.len(), 2);
        let source_ids: Vec<_> = value.contracts.iter().map(|contract| contract.id).collect();
        assert!(source_ids.iter().all(|id| *id > 0));
        assert_ne!(source_ids[0], source_ids[1]);
        let normal: Vec<_> = source_ids
            .iter()
            .map(|id| FinalAnnuityLoss {
                contract: *id,
                started: 20200101,
                cost: 5_000_000,
                recovered: 1_000_000,
                refunds: 200_000,
                eligible: true,
            })
            .collect();
        value.annuity_losses = normal.clone();
        let (key, leaf) = match case {
            "duplicate" | "duplicate-before-recovery" => {
                value.annuity_losses[1].contract = source_ids[0];
                if case == "duplicate-before-recovery" {
                    value.annuity_losses[1].recovered = 5_000_001;
                }
                ("ANNUITY_CONTRACT_TWICE", "contract")
            }
            "recovered" | "recovery-before-refunds" => {
                value.annuity_losses[1].recovered = 5_000_001;
                if case == "recovery-before-refunds" {
                    value.annuity_losses[1].refunds = 5_000_001;
                }
                ("ANNUITY_RECOVERED_ABOVE_COST", "recovered")
            }
            "refunds" => {
                value.annuity_losses[1].refunds = 4_000_001;
                ("ANNUITY_REFUNDS_ABOVE_COST", "refunds")
            }
            _ => unreachable!(),
        };
        let path = alloc::format!("[value/annuity_losses/1/{leaf}]");
        survey_native_refusal(
            &alloc::format!("annuity-{case}-save"),
            &mut session,
            request.clone(),
            key,
            &path,
        );
        survey_native_refusal(
            &alloc::format!("annuity-{case}-estimate"),
            &mut session,
            unit47_admission_estimate(&request),
            key,
            &path,
        );
        let sources = session.draft.income.clone();
        let P::SaveOtherItemized { value, .. } = &mut request else { unreachable!() };
        value.annuity_losses = normal.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        assert_eq!(session.draft.income, sources);
        let result = estimate(&session.draft);
        for id in &source_ids {
            // Each actual pension: $50,000 cost - $10,000 prior recovery
            // - $2,000 refund - ($10,000 gross - $7,000 taxable) = $35,000.
            assert_eq!(super::value(&result, &alloc::format!("annuity_loss_{id}")), 35_000);
        }
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_itemized_topic(&mut restored, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &request else { unreachable!() };
        assert_eq!(value.annuity_losses, normal);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

fn unit47_bond_admission_session() -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let a = AnnualDebtRecord {
        instrument: "Computed excess premium".into(),
        amounts: DebtComponents { taxable: 10_000, bond_premium: 30_000, ..Default::default() },
        premium_history: Some(PremiumHistory {
            prior_net_interest_inclusions: 50_000,
            ..Default::default()
        }),
        ..Default::default()
    };
    let b = AnnualDebtRecord {
        instrument: "Terminal deduction from separate records".into(),
        amounts: DebtComponents { taxable: 100_000, ..Default::default() },
        premium_history: Some(PremiumHistory::default()),
        ..Default::default()
    };
    let interest = Int1099 {
        payer: crate::PrintedParty::from_name("Synthetic premium source".into()),
        recipient: Default::default(),
        taxable: 110_000,
        more: IntMore { bond_premium: 30_000, ..Default::default() },
        after_death: None,
        state_details: Default::default(),
        nominee: None,
        adjustments: Some(InterestAdjustments {
            instrument_records: Some(AnnualDebtRecords { records: vec![a, b] }),
            ..Default::default()
        }),
    };
    let rows = vec![
        document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(interest)),
            },
        ),
        document(0, IncomeSource::Record { value: crate::IncomeRecord::TaxableBondPremium(true) }),
    ];
    save_each_topic(&mut session, rows);
    unit47_admission_itemize(&mut session);
    session
}

#[test]
fn unit47_manual_bond_premium_admission_retains_actual_instrument_sources_and_amounts() {
    for case in
        ["duplicate", "already-computed", "missing", "description", "computed-before-duplicate"]
    {
        let mut session = unit47_bond_admission_session();
        let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &mut request else { unreachable!() };
        assert_eq!(value.bond_instruments.len(), 2);
        let computed =
            value.bond_instruments.iter().find(|source| source.computed_deduction > 0).unwrap().id;
        let manual =
            value.bond_instruments.iter().find(|source| source.computed_deduction == 0).unwrap().id;
        assert!(computed > 0 && manual > 0 && computed != manual);
        let known = |instrument| OtherItemizedInput::BondPremium {
            amount: 5_000,
            source: BondPremiumSource::AcceptedAnnualInstrument { instrument }.into(),
        };
        let (key, index) = match case {
            "duplicate" => {
                value.other_lines = vec![known(manual), known(manual)];
                ("BOND_PREMIUM_SOURCE_TWICE", 1)
            }
            "already-computed" => {
                value.other_lines = vec![known(manual), known(computed)];
                ("BOND_PREMIUM_COMPUTED_SOURCE_TWICE", 1)
            }
            "missing" => {
                value.other_lines = vec![known(manual), known(i64::MAX)];
                ("BOND_PREMIUM_SOURCE_MISSING", 1)
            }
            "description" => {
                value.other_lines = vec![
                    known(manual),
                    OtherItemizedInput::BondPremium {
                        amount: 7_000,
                        source: BondPremiumSource::OtherDocumented { instrument: " ".into() }
                            .into(),
                    },
                ];
                ("BOND_PREMIUM_DESCRIPTION_NEEDED", 1)
            }
            "computed-before-duplicate" => {
                // Preserve per-row order: an already-computed first source
                // fails before a duplicate of it in a later row.
                value.other_lines = vec![known(computed), known(computed)];
                ("BOND_PREMIUM_COMPUTED_SOURCE_TWICE", 0)
            }
            _ => unreachable!(),
        };
        let path = alloc::format!("[value/other_lines/{index}/source/instrument]");
        survey_native_refusal(
            &alloc::format!("manual-premium-{case}-save"),
            &mut session,
            request.clone(),
            key,
            &path,
        );
        survey_native_refusal(
            &alloc::format!("manual-premium-{case}-estimate"),
            &mut session,
            unit47_admission_estimate(&request),
            key,
            &path,
        );
        let sources = session.draft.income.clone();
        let P::SaveOtherItemized { value, .. } = &mut request else { unreachable!() };
        value.other_lines = vec![
            known(manual),
            OtherItemizedInput::BondPremium {
                amount: 7_000,
                source: BondPremiumSource::OtherDocumented {
                    instrument: "Separate historical documented bond".into(),
                }
                .into(),
            },
        ];
        let expected = value.other_lines.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
        assert_eq!(session.draft.income, sources);
        // Existing sole premium owner: $200 computed + $50 documented
        // accepted instrument + $70 separate documented source = $320 once.
        assert_eq!(super::value(&estimate(&session.draft), "bond_premium_deduction"), 320);
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_itemized_topic(&mut restored, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &request else { unreachable!() };
        assert_eq!(value.other_lines, expected);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

fn unit47_farm_production_rows(document: &crate::IncomeRecord) -> &[FarmProductionCohortInput] {
    match document {
        crate::IncomeRecord::Farm(farm) => &farm.more.specials.production,
        crate::IncomeRecord::FarmRental(farm) => &farm.more.specials.production,
        _ => panic!("expected an actual farm production document"),
    }
}

fn unit47_farm_production_rows_mut(
    document: &mut crate::IncomeRecord,
) -> &mut Vec<FarmProductionCohortInput> {
    match document {
        crate::IncomeRecord::Farm(farm) => &mut farm.more.specials.production,
        crate::IncomeRecord::FarmRental(farm) => &mut farm.more.specials.production,
        _ => panic!("expected an actual farm production document"),
    }
}

fn unit47_farm_production_identity_fixture() -> Interview {
    // Reuse the existing real production-cost fixture and ordinary ID allocator.
    let production = crate::tests::farm::farm_specials::production_inventory;
    let mut farm = cash_farm(5_000_000, 0);
    let mut second = production();
    second.description = "Separate direct-farm production lot".into();
    farm.more.specials.production = vec![production(), second];
    let mut rental = crate::tests::farm::farm_rental::rental_page(40_000);
    rental.name = "Production-cost farm rental".into();
    let mut second = production();
    second.description = "Separate farm-rental production lot".into();
    rental.more.specials.production = vec![production(), second];
    let mut session = Interview::new(TEST_USER, farm_only_return());
    let request = save_topic(
        &mut session,
        TaxSection::Farming,
        vec![
            document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) }),
            document(0, IncomeSource::Record { value: crate::IncomeRecord::FarmRental(rental) }),
        ],
    );
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    let mut request = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { allowed_documents, documents, .. } = &mut request else { unreachable!() };
    let ids: Vec<_> = documents
        .documents
        .iter()
        .filter(|item| {
            matches!(
                item.document,
                crate::IncomeRecord::Farm(_) | crate::IncomeRecord::FarmRental(_)
            )
        })
        .flat_map(|item| unit47_farm_production_rows(&item.document).iter().map(|row| row.id))
        .collect();
    assert_eq!(ids.len(), 4);
    assert!(ids.iter().all(|id| *id > 0));
    assert_eq!(ids.iter().copied().collect::<alloc::collections::BTreeSet<_>>().len(), 4);
    assert!(allowed_documents.contains(
        &(libertas_macros::variant_index!(crate::IncomeSourceKind::FarmAveraging) as i32)
    ));
    // A genuine offered data answer will precede the target on the submitted
    // unordered page, then be partitioned away from the income-entry vector.
    documents.documents.push(
        crate::IncomeRecordItem::from_income(document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::FarmAveraging(averaging_election()),
            },
        ))
        .unwrap(),
    );
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    session
}

#[test]
fn unit47_farm_production_identity_preserves_original_document_and_second_cohort_positions() {
    for kind in ["farm", "farm-rental"] {
        let session = unit47_farm_production_identity_fixture();
        unit47_farm_production_identity_case(kind, session);
    }
}

#[inline(never)]
fn unit47_farm_production_identity_case(kind: &str, mut session: Interview) {
    let baseline = supported(&session.draft);
    let schedule_f_net: i64 = baseline
        .lines
        .iter()
        .filter(|line| {
            line.form_line
                .as_ref()
                .is_some_and(|tag| tag.form == TaxForm::ScheduleF && tag.line == "34")
        })
        .map(|line| line.amount)
        .sum();
    // The reused source has $1,400 of current deductible cost per cohort:
    // $50,000 receipts - two distinct $1,400 deductions = $47,200 once.
    assert_eq!(schedule_f_net, 4_720_000);
    let mut request = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let original_documents = documents.clone();
    let answer_index = documents
        .documents
        .iter()
        .position(|item| matches!(item.document, crate::IncomeRecord::FarmAveraging(_)))
        .unwrap();
    let answer = documents.documents.remove(answer_index);
    let target_index = documents
        .documents
        .iter()
        .position(|item| match kind {
            "farm" => matches!(item.document, crate::IncomeRecord::Farm(_)),
            "farm-rental" => {
                matches!(item.document, crate::IncomeRecord::FarmRental(_))
            }
            _ => unreachable!(),
        })
        .unwrap();
    let target = documents.documents.remove(target_index);
    assert_eq!(documents.documents.len(), 1);
    let other = documents.documents.remove(0);
    documents.documents = vec![answer, other, target];
    // The actual submitted target is index2, whereas its post-partition entry
    // is index1. The failure must name the former and the second production row.
    let original = unit47_farm_production_rows(&documents.documents[2].document).to_vec();
    assert_eq!(original.len(), 2);
    assert!(original[0].id > 0 && original[1].id > 0 && original[0].id != original[1].id);
    unit47_farm_production_rows_mut(&mut documents.documents[2].document)[1].id = original[0].id;
    let key = "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425";
    let path = "[documents/documents/2/document/more/specials/production/1]";
    survey_native_refusal(
        &alloc::format!("core-production-identity-{kind}-save"),
        &mut session,
        request.clone(),
        key,
        path,
    );
    survey_native_refusal(
        &alloc::format!("core-production-identity-{kind}-estimate"),
        &mut session,
        estimate_documents_from_save(&request),
        key,
        path,
    );
    let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
    let mut corrected = original.clone();
    corrected.reverse();
    // The negative reordered submission proves its original index. Correction
    // returns to the normal editor order while reordering the actual cohorts.
    *documents = original_documents;
    let target = documents
        .documents
        .iter_mut()
        .find(|item| match kind {
            "farm" => matches!(item.document, crate::IncomeRecord::Farm(_)),
            "farm-rental" => {
                matches!(item.document, crate::IncomeRecord::FarmRental(_))
            }
            _ => unreachable!(),
        })
        .unwrap();
    *unit47_farm_production_rows_mut(&mut target.document) = corrected.clone();
    let before = session.draft.clone();
    let dirty = session.dirty;
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        P::from_avro(&estimate_documents_from_save(&request).to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
    let result = supported(&session.draft);
    assert_eq!(result.tax, baseline.tax);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
    let financial_rows = |result: &FederalResult| {
        result
            .lines
            .iter()
            .map(|row| (row.id.clone(), row.amount))
            .collect::<alloc::collections::BTreeMap<_, _>>()
    };
    assert_eq!(financial_rows(&result), financial_rows(&baseline));
    let saved = session.draft.clone();
    let reopened = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    let shown = documents
        .documents
        .iter()
        .find(|item| match kind {
            "farm" => matches!(item.document, crate::IncomeRecord::Farm(_)),
            "farm-rental" => {
                matches!(item.document, crate::IncomeRecord::FarmRental(_))
            }
            _ => unreachable!(),
        })
        .unwrap();
    assert_eq!(unit47_farm_production_rows(&shown.document), corrected.as_slice());
    accepted(&send(&mut session, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(session.draft, saved);
    let mut restored = restart(saved.clone());
    let reopened = open_topic(&mut restored, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &reopened else { unreachable!() };
    let shown = documents
        .documents
        .iter()
        .find(|item| match kind {
            "farm" => matches!(item.document, crate::IncomeRecord::Farm(_)),
            "farm-rental" => {
                matches!(item.document, crate::IncomeRecord::FarmRental(_))
            }
            _ => unreachable!(),
        })
        .unwrap();
    assert_eq!(unit47_farm_production_rows(&shown.document), corrected.as_slice());
    accepted(&send(&mut restored, P::from_avro(&reopened.to_avro()).unwrap()));
    assert_eq!(restored.draft, saved);
    assert_eq!(supported(&restored.draft), result);
}
