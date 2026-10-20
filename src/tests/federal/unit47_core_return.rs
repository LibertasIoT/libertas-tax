use crate::tests::support::client::survey::survey_native_refusal;
use crate::tests::support::client::estimate_documents_from_save;
use super::*;
use alloc::string::String;

fn benefits_editor(session: &mut Interview) -> P {
    let request = go_to(
        &session.draft,
        PAGE_ADJUSTMENTS,
        TaxSection::SelfEmployedBenefits,
    );
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
    SeRetirement::Qualified {
        rate: 2500,
        contribution: 0,
        deferral,
        catch_up,
        roth,
    }
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
        let P::SaveSelfEmployedBenefits { value, .. } = &mut request else {
            unreachable!()
        };
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
        let P::SaveSelfEmployedBenefits { value, .. } = &mut request else {
            unreachable!()
        };
        value.businesses[1].retirement = plan.clone();
        accepted(&send(
            &mut session,
            P::from_avro(&request.to_avro()).unwrap(),
        ));
        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        let mut restored = restart(saved.clone());
        let reopened = benefits_editor(&mut restored);
        let P::SaveSelfEmployedBenefits { value, .. } = &reopened else {
            unreachable!()
        };
        assert_eq!(value.businesses.len(), 2);
        assert_eq!(value.businesses[1].retirement, plan);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

fn disaster_text(field: &str, text: String) -> FederalDisasterRelief {
    let mut source = FederalDisasterRelief {
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
                carrier: if field == "carrier" {
                    text.clone()
                } else {
                    "Synthetic carrier".into()
                },
                loss: DisasterCarrierLoss::PhysicalInjury,
            };
            source.qualification = DisasterQualification::CarrierAccident {
                event: if field == "carrier-event" {
                    text
                } else {
                    "Synthetic accident".into()
                },
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
            source.qualification = DisasterQualification::SecretaryCatastrophe {
                determination: text,
            };
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
                payer: if field == "government-payer" {
                    text
                } else {
                    "Synthetic agency".into()
                },
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
            let request = save_topic(
                &session.draft,
                TaxSection::OtherIncome,
                vec![
                    document(0, IncomeDocument::DisasterRelief(source.clone())),
                    document(0, IncomeDocument::DisasterRelief(source.clone())),
                ],
            );
            accepted(&send(
                &mut session,
                P::from_avro(&request.to_avro()).unwrap(),
            ));
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
                    value(
                        &baseline,
                        &alloc::format!("disaster_relief_{}_excluded", entry.id)
                    ),
                    8_000
                );
            }
            assert_eq!(value(&baseline, "agi"), original_agi);

            let mut invalid = open_topic(&mut session, TaxSection::OtherIncome);
            let P::SaveIncomes { documents, .. } = &mut invalid else {
                unreachable!()
            };
            assert_eq!(documents.documents.len(), 2);
            documents.documents[1].document =
                IncomeDocument::DisasterRelief(disaster_text(field, " ".into()));
            let expected = alloc::format!("[documents/documents/1/document/{leaf}]");
            let case = alloc::format!("disaster-{field}-{flavor}-required-blank");
            survey_native_refusal(
                &case,
                &mut session,
                invalid,
                "DISASTER_SOURCE_TEXT_INVALID",
                &expected,
            );
            assert_eq!(session.draft, saved);

            let correction = open_topic(&mut session, TaxSection::OtherIncome);
            accepted(&send(&mut session, correction));
            assert_eq!(session.draft, saved);
            let mut restored = restart(saved.clone());
            let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
            assert!(page_documents(&reopened).iter().all(|item| {
                matches!(&item.document, IncomeDocument::DisasterRelief(actual) if *actual == source)
            }));
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
    form.amounts = PolicyAmounts::ByMonth {
        months: vec![
            PolicyMonth {
                month: 1,
                premium: 100_000,
                benchmark: Some(60_000),
                advance: 0,
            },
            PolicyMonth {
                month: 2,
                premium: 100_000,
                benchmark: Some(60_000),
                advance: 0,
            },
        ],
    };
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
        (
            "advance",
            "MARKETPLACE_ADVANCE_ABOVE_PREMIUM",
            "forms/1/amounts/months/1/advance",
        ),
        (
            "dependent-social",
            "MARKETPLACE_TAXABLE_SOCIAL_ABOVE_GROSS",
            "dependents/0/taxable_social",
        ),
        (
            "coverage-dates",
            "PTC_COVERAGE_DATES_REVERSED",
            "forms/1/coverage/1/entry/1/end",
        ),
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
                let PolicyAmounts::ByMonth { months } = &mut value.forms[1].amounts else {
                    unreachable!()
                };
                months[1].advance = 100_000;
            }
            "tax-free-coverage" => value.forms[1].payments[1].tax_free_coverage = Some(90_000),
            "child-funding" => value.forms[1].medical_costs[1].tax_free_child = Some(10_000),
            _ => {}
        }
        save_marketplace_and_reopen(&mut session, value);
        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        let mut request = open_credits_topic(&mut session, TaxSection::Marketplace);
        let P::SaveMarketplace { value, .. } = &mut request else {
            unreachable!()
        };
        match case {
            "advance" => {
                let PolicyAmounts::ByMonth { months } = &mut value.forms[1].amounts else {
                    unreachable!()
                };
                months[1].advance = 100_001;
            }
            "dependent-social" => value.dependents[0].taxable_social = 40_001,
            "coverage-dates" => value.forms[1].coverage[1].entry[1].end = 20260131,
            "tax-free-premiums" => value.forms[1].payments[1].tax_free_premiums = 90_001,
            "tax-free-coverage" => value.forms[1].payments[1].tax_free_coverage = Some(90_001),
            "child-funding" => value.forms[1].medical_costs[1].tax_free_child = Some(10_001),
            "child-funding-unavailable" => {
                value.forms[1].medical_costs[1].child_increase = None;
                value.forms[1].medical_costs[1].tax_free_child = Some(1);
            }
            _ => unreachable!(),
        }
        let expected = alloc::format!("[value/{leaf}]");
        let case = alloc::format!("marketplace-{case}");
        survey_native_refusal(&case, &mut session, request, key, &expected);
        assert_eq!(session.draft, saved);
        let correction = open_credits_topic(&mut session, TaxSection::Marketplace);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = open_credits_topic(&mut restored, TaxSection::Marketplace);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_retirement_native_owners_locate_printed_treatment_and_nested_disaster_failures() {
    let base = r1099(1_000_000, normal_kept(), false);
    let mut cases = Vec::new();
    let mut invalid = base.clone();
    invalid.code = early_kept();
    cases.push(("treatment-selection", invalid, "R_BOX7_TREATMENT", "code"));
    let invalid = printed_as(roth_item_for_tests(), "J");
    cases.push((
        "roth-printed-subcode",
        invalid,
        "R_ROTH_CODE_NOT_BOX7",
        "code/roth",
    ));
    let invalid = r1099(
        1_000_000,
        DistributionCode::Disability {
            began: 20250101,
            wages: DisabilityWages::All,
        },
        true,
    );
    cases.push((
        "ira-disability-wages",
        invalid,
        "R_IRA_DISABILITY_NOT_WAGES",
        "code/wages",
    ));
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
        (
            "after-death-taxable-box",
            1_000_000,
            1_000_001,
            "after_death/taxable",
        ),
        (
            "after-death-taxable-part",
            400_000,
            400_001,
            "after_death/taxable",
        ),
    ] {
        let mut invalid = base.clone();
        invalid.after_death = Some(crate::RetirementAfterDeath { gross, taxable });
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
            "box_3" => invalid.more.box_3 = 1,
            "box_6" => invalid.more.box_6 = 1,
            "box_8a" => invalid.more.box_8a = 1,
            "box_9b" => invalid.more.box_9b = Some(0),
            "box_10" => invalid.more.box_10 = 1,
            "disability" => {
                invalid.more.disability =
                    crate::DisabilityIncome::Qualifying { amount: 0, eligible: true }
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
    invalid.more.box_8b_percent = Some(33.33);
    cases.push((
        "roth-box8b-precedence",
        invalid,
        "R_BOX8B_WITHOUT_BOX8A",
        "more/box_8b_percent",
    ));
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
                DistributionCode::RothIra {
                    roth,
                    sequence: None,
                    rolled_over: 0,
                },
                false,
            ),
            printed,
        );
        cases.push((case, invalid, "ROTH_DISABLED_AFTER_DISTRIBUTION", leaf));
    }
    let mut invalid = base.clone();
    invalid.taxable = 1_000_001;
    invalid.withholding = 1_000_001;
    cases.push((
        "taxable-before-withholding",
        invalid,
        "R_TAXABLE_ABOVE_GROSS",
        "taxable",
    ));
    let mut invalid = base.clone();
    invalid.after_death = Some(crate::RetirementAfterDeath {
        gross: 1_000_001,
        taxable: 0,
    });
    invalid.withholding = 1_000_001;
    cases.push((
        "after-death-before-withholding",
        invalid,
        "AFTER_DEATH_ABOVE_BOX",
        "after_death/gross",
    ));
    let mut invalid = base.clone();
    invalid.taxable = 1_000_001;
    cases.push(("taxable", invalid, "R_TAXABLE_ABOVE_GROSS", "taxable"));
    let mut invalid = base.clone();
    invalid.withholding = 1_000_001;
    cases.push((
        "withholding",
        invalid,
        "R_WITHHOLDING_ABOVE_GROSS",
        "withholding",
    ));
    let mut invalid = base.clone();
    invalid.box7 = 0;
    cases.push(("box7-empty", invalid, "R_BOX7_NO_CODE", "box7"));
    let mut invalid = base.clone();
    invalid.box7 = 7;
    cases.push(("box7-three-codes", invalid, "R_BOX7_TOO_MANY_CODES", "box7"));
    let mut invalid = base.clone();
    invalid.more.box_10 = 1;
    cases.push((
        "box10-without-b",
        invalid,
        "R_BOX10_WITHOUT_B",
        "more/box_10",
    ));
    let mut invalid = printed_as(base.clone(), "7B");
    invalid.more.box_10 = 1_000_001;
    cases.push((
        "box10-above-gross",
        invalid,
        "R_BOX10_ABOVE_GROSS",
        "more/box_10",
    ));
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
    cases.push((
        "rollover",
        invalid,
        "R_ROLLOVER_ABOVE_GROSS",
        "code/what_happened/amount",
    ));
    let invalid = r1099(
        1_000_000,
        DistributionCode::Normal {
            what_happened: Disposition::Converted { amount: 1_000_001 },
        },
        false,
    );
    cases.push((
        "conversion",
        invalid,
        "R_CONVERSION_ABOVE_GROSS",
        "code/what_happened/amount",
    ));
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
    let DistributionCode::RothIra { rolled_over, .. } = &mut invalid.code else {
        unreachable!()
    };
    *rolled_over = 1_000_001;
    cases.push((
        "roth-rollover",
        invalid,
        "R_ROTH_ROLLOVER_ABOVE_GROSS",
        "code/rolled_over",
    ));
    let mut invalid = base.clone();
    invalid.public_safety = vec![
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
    ];
    cases.push((
        "public-safety-premium",
        invalid,
        "PSO_FUNDED_ABOVE_PREMIUMS",
        "public_safety/1/funded",
    ));
    let mut invalid = base.clone();
    invalid.public_safety = vec![
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
    ];
    cases.push((
        "public-safety-gross",
        invalid,
        "PSO_FUNDED_ABOVE_GROSS",
        "public_safety",
    ));
    let invalid = disaster_r1099(3_000_001, 0, DisasterSpread::AllInReturnYear);
    cases.push((
        "disaster-gross",
        invalid,
        "R_DISASTER_ABOVE_GROSS",
        "disaster/amount",
    ));
    let invalid = disaster_r1099(2_000_000, 200_001, DisasterSpread::AllInReturnYear);
    cases.push((
        "disaster-limit",
        invalid,
        "DISASTER_ABOVE_LIMIT",
        "disaster/earlier",
    ));
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
        let DisasterDistribution::Qualified { repaid, .. } = &mut invalid.disaster else {
            unreachable!()
        };
        *repaid = DisasterRepayment::Yes { total, in_return_year };
        cases.push((case, invalid, key, leaf));
    }
    let mut invalid = disaster_r1099(1_000_000, 0, DisasterSpread::AllInReturnYear);
    let DisasterDistribution::Qualified { distributed, .. } = &mut invalid.disaster else {
        unreachable!()
    };
    *distributed = 20261231;
    cases.push((
        "disaster-date",
        invalid,
        "DISASTER_OUTSIDE_PERIOD",
        "disaster/distributed",
    ));
    for (case, invalid, key, leaf) in cases {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let request = save_topic(
            &session.draft,
            TaxSection::Retirement,
            vec![
                document(0, IncomeDocument::R1099(base.clone())),
                document(0, IncomeDocument::R1099(base.clone())),
            ],
        );
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        let mut request = open_topic(&mut session, TaxSection::Retirement);
        let P::SaveIncomes { documents, .. } = &mut request else {
            unreachable!()
        };
        assert_eq!(documents.documents.len(), 2);
        documents.documents[1].document = IncomeDocument::R1099(invalid);
        let expected = alloc::format!("[documents/documents/1/document/{leaf}]");
        let case = alloc::format!("retirement-{case}");
        survey_native_refusal(&case, &mut session, request, key, &expected);
        assert_eq!(session.draft, saved);
        let correction = open_topic(&mut session, TaxSection::Retirement);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
    }
    // Preserve the explicitly approved informational box9a exception after removal.
    let mut roth = roth_item_for_tests();
    roth.more.box_9a_percent = Some(33.33);
    assert_eq!(roth.more.box_9a_percent, Some(33.33));
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &session.draft,
        TaxSection::Retirement,
        vec![document(0, IncomeDocument::R1099(roth.clone()))],
    );
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let canonical_page = open_topic(&mut session, TaxSection::Retirement);
    let IncomeDocument::R1099(canonical) = &page_documents(&canonical_page)[0].document else {
        unreachable!()
    };
    let canonical = canonical.clone();
    let mut expected = roth;
    expected.allowed_treatments =
        DistributionCodes::treatments(expected.box7, expected.ira, expected.trump);
    expected.allowed_methods =
        DistributionCodes::methods(expected.box7, expected.ira, expected.trump);
    // Box9a is accepted informational input on every treatment. The existing
    // retirement projection retains it only for Form4972; the Roth page is blank.
    expected.more.box_9a_percent = None;
    assert_eq!(canonical, expected);
    let mut restored = restart(saved.clone());
    let reopened = open_topic(&mut restored, TaxSection::Retirement);
    assert!(
        page_documents(&reopened)
            .iter()
            .any(|item| matches!(&item.document,
        IncomeDocument::R1099(actual) if *actual == canonical))
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
            DistributionCode::Disability {
                began: 20250101,
                wages: DisabilityWages::None,
            },
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
                DistributionCode::RothIra {
                    roth,
                    rolled_over: 0,
                    sequence: None,
                },
                false,
            ),
            printed,
        ));
    }
    let mut after_death = r1099(1_000_000, normal_kept(), false);
    after_death.after_death = Some(crate::RetirementAfterDeath {
        gross: 1_000_000,
        taxable: 1_000_000,
    });
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
        let request = save_topic(
            &session.draft,
            TaxSection::Retirement,
            vec![document(0, IncomeDocument::R1099(row))],
        );
        let request = P::from_avro(&request.to_avro()).unwrap();
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let reopened = open_topic(&mut restored, TaxSection::Retirement);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn unit47_reservist_second_anniversary_is_inclusive_and_leap_day_clamps_before_native_save() {
    for (case, released, deadline, late) in [
        ("january", 20240101, 20260101, 20260102),
        ("leap-day", 20240229, 20260228, 20260301),
    ] {
        let repayment = |released, distributed, last_payment| ReservistRepayment {
            ordered: 20220101,
            term: ReserveTerm::Indefinite,
            released,
            distributed,
            gross: 1_200_000,
            prior: 0,
            payments: vec![
                ReservistPayment {
                    date: 20260101,
                    amount: 100_000,
                },
                ReservistPayment {
                    date: last_payment,
                    amount: 200_000,
                },
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
        let request = save_topic(
            &session.draft,
            TaxSection::Retirement,
            vec![
                document(
                    0,
                    IncomeDocument::R1099(r1099(1_000_000, normal_kept(), true)),
                ),
                document(0, IncomeDocument::Form8606(form.clone())),
            ],
        );
        accepted(&send(
            &mut session,
            P::from_avro(&request.to_avro()).unwrap(),
        ));
        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        let result = supported(&saved);
        assert_eq!(value(&result, "reservist_taxpayer_2_basis"), 3_000);
        assert_eq!(value(&result, "reservist_taxpayer_2_unrepaid"), 9_000);
        let mut request = open_topic(&mut session, TaxSection::Retirement);
        let P::SaveIncomes { documents, .. } = &mut request else {
            unreachable!()
        };
        let index = documents
            .documents
            .iter()
            .position(|item| matches!(item.document, IncomeDocument::Form8606(_)))
            .unwrap();
        assert_eq!(index, 1);
        let IncomeDocument::Form8606(actual) = &mut documents.documents[index].document else {
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
            "[documents/documents/1/document/reservist_repayments/1/payments/1/date]",
        );
        assert_eq!(session.draft, saved);
        let correction = open_topic(&mut session, TaxSection::Retirement);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = open_topic(&mut restored, TaxSection::Retirement);
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
        assert_eq!(
            value(&supported(&restored.draft), "reservist_taxpayer_2_basis"),
            3_000
        );
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
        let status = if case == "owner" {
            FederalStatus::Single
        } else {
            FederalStatus::Joint
        };
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
                (
                    "IRA_NONDEDUCTIBLE_ABOVE_TRADITIONAL",
                    "people/1/more/nondeductible",
                )
            }
            "source-value-duplicate" => {
                people[0].form_8606_value = None;
                people[0].more.excess = Some(excess_entry(Some(0)));
                (
                    "IRA_EXCESS_TRADITIONAL_ON_8606",
                    "people/0/more/excess/traditional_value",
                )
            }
            "source-value-missing" => {
                people[1].form_8606_value = Some(40_000);
                people[1].more.excess = Some(excess_entry(None));
                (
                    "IRA_EXCESS_TRADITIONAL_VALUE_NEEDED",
                    "people/1/more/excess/traditional_value",
                )
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
                (
                    "IRA_RETURNED_EXCESS_ABOVE_CARRIED",
                    "people/1/more/excess/returned_prior_excess",
                )
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
                people[target].more.returned_in_following_year = vec![crate::ReturnedContribution {
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
            if case == "actual-form-value" {
                Some(40_000)
            } else {
                None
            },
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
    *estimate = Some(vec![0xff, 0, 0xfe]);
    *progress = Some(vec![0xfe, 0, 0xff]);
    *warnings = vec![crate::SaveWarning {
        message: vec![0xff],
    }];
    *allowed_owners = vec![i32::MIN, i32::MAX, i32::MIN];
    *entered_credits = vec![i32::MIN, i32::MIN];
    *foreign_businesses = vec![crate::RecordChoice {
        id: i64::MIN,
        label: vec![0xff],
    }];
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
        let P::SaveMoreCredits {
            estimate, progress, ..
        } = &reopened
        else {
            unreachable!()
        };
        assert!(estimate.as_ref().is_some_and(|value| !value.is_empty()));
        assert!(progress.as_ref().is_some_and(|value| !value.is_empty()));
        clear_more_credits_output(&mut reopened);
        accepted(&send(
            &mut session,
            P::from_avro(&reopened.to_avro()).unwrap(),
        ));
        assert_eq!(session.draft, expected);
        let mut restored = restart(expected.clone());
        let mut reopened = super::unit47_text_size::country_reopened(&mut restored);
        clear_more_credits_output(&mut reopened);
        accepted(&send(
            &mut restored,
            P::from_avro(&reopened.to_avro()).unwrap(),
        ));
        assert_eq!(restored.draft, expected);
        super::unit47_text_size::assert_country_source(&restored);
    }
}

#[test]
fn unit47_output_only_echoes_cannot_hide_an_actual_input_refusal() {
    let (mut session, _, mut correction) = super::unit47_text_size::country_fixture();
    forge_more_credits_output(&mut correction);
    let mut request = correction.clone();
    let P::SaveMoreCredits { value, .. } = &mut request else {
        unreachable!()
    };
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
    accepted(&send(
        &mut session,
        P::from_avro(&correction.to_avro()).unwrap(),
    ));
    super::unit47_text_size::assert_country_source(&session);
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let mut reopened = super::unit47_text_size::country_reopened(&mut restored);
    clear_more_credits_output(&mut reopened);
    accepted(&send(
        &mut restored,
        P::from_avro(&reopened.to_avro()).unwrap(),
    ));
    assert_eq!(restored.draft, saved);
}

fn ltc_preview(page: &P, row: &IncomeDocumentItem) -> P {
    let P::SaveIncomes {
        empty_k1_activity_choices,
        allowed_injured_source_facts,
        allowed_injured_retained_owners,
        ltc_payee_sources,
        wage_scorporations,
        allowed_wage_scorporations,
        initial_k1_investment_choices,
        research_businesses,
        retained_research_choices,
        k1_credit_lines,
        research_credit_lines,
        renewable_credit_lines,
        transferable_credit_lines,
        allowed_continuing_risk,
        foreign_countries,
        interest_statements,
        people,
        farms,
        cooperative_activities,
        cooperative_properties,
        businesses,
        qbi_businesses,
        rentals,
        mortgage_loans,
        royalty_properties,
        equipment_rentals,
        hobbies,
        ccc_lots,
        farm_codes,
        farm_limitations,
        farm_sources,
        basis_assets,
        farm_method_changes,
        inherited_iras,
        inherited_roth_iras,
        ltc_insured,
        annuity_sources,
        recovery_sources,
        allowed_business_source,
        allowed_farm_source,
        allowed_partner_source,
        allowed_home_spouse,
        allowed_other_parent,
        children,
        ..
    } = page
    else {
        unreachable!()
    };
    P::PreviewDocument {
        initial_rental_recovery_choices: Vec::new(),
        rental_energy_buildings: Vec::new(),
        empty_k1_activity_choices: empty_k1_activity_choices.clone(),
        allowed_injured_source_facts: allowed_injured_source_facts.clone(),
        allowed_injured_retained_owners: allowed_injured_retained_owners.clone(),
        ltc_payee_sources: ltc_payee_sources.clone(),
        wage_scorporations: wage_scorporations.clone(),
        allowed_wage_scorporations: allowed_wage_scorporations.clone(),
        initial_k1_investment_choices: initial_k1_investment_choices.clone(),
        research_businesses: research_businesses.clone(),
        retained_research_choices: retained_research_choices.clone(),
        k1_credit_lines: k1_credit_lines.clone(),
        research_credit_lines: research_credit_lines.clone(),
        renewable_credit_lines: renewable_credit_lines.clone(),
        transferable_credit_lines: transferable_credit_lines.clone(),
        user: TEST_USER,
        allowed_continuing_risk: allowed_continuing_risk.clone(),
        id: row.id,
        owner: row.owner,
        community: row.community,
        foreign_countries: foreign_countries.clone(),
        interest_statements: interest_statements.clone(),
        people: people.clone(),
        farms: farms.clone(),
        cooperative_activities: cooperative_activities.clone(),
        cooperative_properties: cooperative_properties.clone(),
        businesses: businesses.clone(),
        qbi_businesses: qbi_businesses.clone(),
        rentals: rentals.clone(),
        mortgage_loans: mortgage_loans.clone(),
        royalty_properties: royalty_properties.clone(),
        equipment_rentals: equipment_rentals.clone(),
        hobbies: hobbies.clone(),
        ccc_lots: ccc_lots.clone(),
        farm_codes: farm_codes.clone(),
        farm_limitations: farm_limitations.clone(),
        farm_sources: farm_sources.clone(),
        basis_assets: basis_assets.clone(),
        farm_method_changes: farm_method_changes.clone(),
        inherited_iras: inherited_iras.clone(),
        inherited_roth_iras: inherited_roth_iras.clone(),
        ltc_insured: ltc_insured.clone(),
        annuity_sources: annuity_sources.clone(),
        recovery_sources: recovery_sources.clone(),
        allowed_business_source: allowed_business_source.clone(),
        allowed_farm_source: allowed_farm_source.clone(),
        allowed_partner_source: allowed_partner_source.clone(),
        allowed_home_spouse: allowed_home_spouse.clone(),
        allowed_other_parent: allowed_other_parent.clone(),
        children: children.clone(),
        document: row.document.clone(),
    }
}

fn ltc_context_fixture(recovery: bool) -> (Interview, P, i64, i64) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut annuity = r1099(90_000, DistributionCode::Nonqualified, false);
    annuity.taxable = 40_000;
    let request = save_topic(
        &session.draft,
        TaxSection::Retirement,
        vec![document(0, IncomeDocument::R1099(annuity))],
    );
    accepted(&send(&mut session, request));
    let annuity_id = session
        .draft
        .income
        .iter()
        .find_map(|entry| {
            matches!(entry.income, FederalIncome::Retirement { .. }).then_some(entry.id)
        })
        .unwrap();
    let request = save_topic(
        &session.draft,
        TaxSection::OtherIncome,
        vec![document(
            0,
            IncomeDocument::OtherIncome(OtherIncomeItem {
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
    let mut page = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes {
        documents,
        annuity_sources,
        recovery_sources,
        ..
    } = &mut page
    else {
        unreachable!()
    };
    assert_eq!(documents.documents.len(), 1);
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
    let treatment = if recovery {
        NonqualifiedCareTreatmentPage::AccidentHealth {
            funding: CareBenefitFunding::AfterTax,
            medical_reimbursement: 0,
            permanent_injury: None,
            recovery: Some(recovery_id),
        }
    } else {
        NonqualifiedCareTreatmentPage::LifeAnnuity { source: annuity_id }
    };
    documents.documents.push(document(
        0,
        IncomeDocument::Ltc1099(Ltc1099 {
            benefits: 90_000,
            accelerated: 0,
            interest: 0,
            benefit_origin: Some(LtcPaymentOrigin::Insurance),
            accelerated_origin: None,
            paid: LtcPaidBasis::NotChecked,
            contract: LtcContractPage::Nonqualified { treatment },
            illness: LtcIllness::Chronic,
            insured: None,
        }),
    ));
    (session, page, annuity_id, recovery_id)
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
        let (mut session, mut correction, annuity_id, recovery_id) = ltc_context_fixture(recovery);
        let original_tax = supported(&session.draft).tax;
        if case == "insured-period" {
            let P::SaveIncomes { documents, .. } = &mut correction else {
                unreachable!()
            };
            documents.documents[1].document = IncomeDocument::LtcInsured(LtcInsured {
                insured: LtcInsuredPerson::OnReturn { person: -1 },
                periods: LtcPeriodsInput::One {
                    starts: 20260201,
                    ends: 20260228,
                    services: 0,
                    reimbursements: 0,
                    recipient: CareBenefitRecipient::Insured {
                        other_payees: false,
                    },
                },
            });
        }
        let mut invalid = correction.clone();
        let P::SaveIncomes { documents, .. } = &mut invalid else {
            unreachable!()
        };
        let target = &mut documents.documents[1];
        let mut key = "LTC1099_SOURCE_UNAVAILABLE";
        let mut local_path = if recovery {
            "contract/treatment/recovery"
        } else {
            "contract/treatment/source"
        };
        if case == "insured-period" {
            key = "LTC_PERIOD_ENDS_BEFORE_START";
            local_path = "periods/ends";
            target.document = IncomeDocument::LtcInsured(LtcInsured {
                insured: LtcInsuredPerson::OnReturn { person: -1 },
                periods: LtcPeriodsInput::One {
                    starts: 20260201,
                    ends: 20260131,
                    services: 0,
                    reimbursements: 0,
                    recipient: CareBenefitRecipient::Insured {
                        other_payees: false,
                    },
                },
            });
        } else {
            let IncomeDocument::Ltc1099(form) = &mut target.document else {
                unreachable!()
            };
            match case {
                "annuity-wrong-owner" => target.owner = Owner::Spouse,
                "annuity-above-gross" => form.benefits += 1,
                "local-no-amount" => {
                    form.benefits = 0;
                    key = "LTC1099_NO_AMOUNT";
                    local_path = "benefits";
                    form.contract = LtcContractPage::Nonqualified {
                        treatment: NonqualifiedCareTreatmentPage::LifeAnnuity { source: i64::MAX },
                    };
                }
                "annuity-missing" | "annuity-wrong-kind" => {
                    form.contract = LtcContractPage::Nonqualified {
                        treatment: NonqualifiedCareTreatmentPage::LifeAnnuity {
                            source: if case == "annuity-missing" {
                                i64::MAX
                            } else {
                                recovery_id
                            },
                        },
                    };
                }
                "recovery-missing" | "recovery-wrong-kind" => {
                    form.contract = LtcContractPage::Nonqualified {
                        treatment: NonqualifiedCareTreatmentPage::AccidentHealth {
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
        let row = page_documents(&invalid)[1].clone();
        let local =
            crate::care_benefits::document_issue(&session.draft, row.owner, &row.document).unwrap();
        assert_eq!(local.key, key);
        assert_eq!(
            local.field.unwrap().square(),
            alloc::format!("[{local_path}]")
        );
        for (endpoint, request, path) in [
            (
                "save",
                invalid.clone(),
                alloc::format!("[documents/documents/1/document/{local_path}]"),
            ),
            (
                "estimate",
                estimate_documents_from_save(&invalid),
                alloc::format!("[documents/documents/1/document/{local_path}]"),
            ),
            (
                "preview",
                ltc_preview(&invalid, &row),
                alloc::format!("[document/{local_path}]"),
            ),
        ] {
            survey_native_refusal(
                &alloc::format!("core-ltc-{case}-{endpoint}"),
                &mut session,
                request,
                key,
                &path,
            );
        }
        // The visible source correction uses the actual independently saved
        // annuity/recovery. Its income remains with that owner, exactly once.
        let before = session.draft.clone();
        let row = page_documents(&correction)[1].clone();
        accepted(&send(
            &mut session,
            P::from_avro(&ltc_preview(&correction, &row).to_avro()).unwrap(),
        ));
        accepted(&send(
            &mut session,
            P::from_avro(&estimate_documents_from_save(&correction).to_avro()).unwrap(),
        ));
        assert_eq!(session.draft, before);
        accepted(&send(
            &mut session,
            P::from_avro(&correction.to_avro()).unwrap(),
        ));
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 100_600);
        assert_eq!(value(&result, "recovery_2025_taxable"), 200);
        if case != "insured-period" {
            assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
        }
        assert_eq!(result.tax, original_tax);
        let saved = session.draft.clone();
        let reopened = open_topic(&mut session, TaxSection::OtherIncome);
        assert_eq!(page_documents(&reopened)[1].document, row.document);
        accepted(&send(
            &mut session,
            P::from_avro(&reopened.to_avro()).unwrap(),
        ));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
        assert_eq!(page_documents(&reopened)[1].document, row.document);
        accepted(&send(
            &mut restored,
            P::from_avro(&reopened.to_avro()).unwrap(),
        ));
        assert_eq!(restored.draft, saved);
        assert_eq!(supported(&restored.draft), result);
    }
}

#[test]
fn unit47_ltc_shared_annuity_gross_is_checked_after_individual_source_admission() {
    let (mut session, mut correction, _, _) = ltc_context_fixture(false);
    let P::SaveIncomes { documents, .. } = &mut correction else {
        unreachable!()
    };
    let IncomeDocument::Ltc1099(first) = &mut documents.documents[1].document else {
        unreachable!()
    };
    first.benefits = 60_000;
    let mut second = first.clone();
    second.benefits = 30_000;
    documents
        .documents
        .push(document(0, IncomeDocument::Ltc1099(second)));
    let mut invalid = correction.clone();
    let P::SaveIncomes { documents, .. } = &mut invalid else {
        unreachable!()
    };
    let IncomeDocument::Ltc1099(second) = &mut documents.documents[2].document else {
        unreachable!()
    };
    second.benefits += 1;
    for row in &documents.documents[1..] {
        assert!(
            crate::care_benefits::document_issue(&session.draft, row.owner, &row.document)
                .is_none()
        );
    }
    for (endpoint, request) in [
        ("save", invalid.clone()),
        ("estimate", estimate_documents_from_save(&invalid)),
    ] {
        survey_native_refusal(
            &alloc::format!("core-ltc-shared-gross-{endpoint}"),
            &mut session,
            request,
            "LTC1099_SOURCE_UNAVAILABLE",
            "[documents/documents/1/document/contract/treatment/source]",
        );
    }
    accepted(&send(
        &mut session,
        P::from_avro(&correction.to_avro()).unwrap(),
    ));
    assert_eq!(value(&supported(&session.draft), "agi"), 100_600);
    assert_eq!(value(&supported(&session.draft), "taxable_ltc_benefits"), 0);
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
    accepted(&send(
        &mut restored,
        P::from_avro(&reopened.to_avro()).unwrap(),
    ));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_ltc_unresolved_character_remains_accepted_with_an_unavailable_estimate() {
    let (mut session, mut request, _, _) = ltc_context_fixture(false);
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    let IncomeDocument::Ltc1099(form) = &mut documents.documents[1].document else {
        unreachable!()
    };
    form.contract = LtcContractPage::Nonqualified {
        treatment: NonqualifiedCareTreatmentPage::Unclassified,
    };
    assert!(
        crate::care_benefits::document_issue(
            &session.draft,
            Owner::Taxpayer,
            &documents.documents[1].document
        )
        .is_none()
    );
    accepted(&send(
        &mut session,
        P::from_avro(&request.to_avro()).unwrap(),
    ));
    let result = estimate(&session.draft);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED")
    );
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
    accepted(&send(
        &mut restored,
        P::from_avro(&reopened.to_avro()).unwrap(),
    ));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_ltc_recovery_context_uses_whole_page_answers_and_bound_preview_source() {
    let (mut session, guide, _, recovery_id) = ltc_context_fixture(true);
    let mut zero_source = guide.clone();
    let P::SaveIncomes { documents, .. } = &mut zero_source else {
        unreachable!()
    };
    documents.documents.pop();
    let IncomeDocument::OtherIncome(recovery) = &mut documents.documents[0].document else {
        unreachable!()
    };
    recovery.amount = 0;
    accepted(&send(
        &mut session,
        P::from_avro(&zero_source.to_avro()).unwrap(),
    ));
    assert_eq!(value(&supported(&session.draft), "agi"), 100_400);
    let mut correction = open_topic(&mut session, TaxSection::OtherIncome);
    let source_receipt = page_documents(&guide)[1].clone();
    let P::SaveIncomes { documents, .. } = &mut correction else {
        unreachable!()
    };
    assert_eq!(documents.documents[0].id, recovery_id);
    let IncomeDocument::OtherIncome(recovery) = &mut documents.documents[0].document else {
        unreachable!()
    };
    recovery.amount = 30_000;
    documents.documents.push(source_receipt.clone());
    // Preview contains only this bound receipt, so the accepted zero recovery
    // is still unavailable. It cannot borrow an unsubmitted edit from the page.
    survey_native_refusal(
        "core-ltc-bound-zero-recovery-preview",
        &mut session,
        ltc_preview(&correction, &source_receipt),
        "LTC1099_SOURCE_UNAVAILABLE",
        "[document/contract/treatment/recovery]",
    );
    let saved = session.draft.clone();
    // Estimate and Save contain both Page rows. The candidate's corrected
    // existing recovery is the source; its unchanged ID was never guessed.
    accepted(&send(
        &mut session,
        P::from_avro(&estimate_documents_from_save(&correction).to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, saved);
    accepted(&send(
        &mut session,
        P::from_avro(&correction.to_avro()).unwrap(),
    ));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 100_600);
    assert_eq!(value(&result, "recovery_2025_taxable"), 200);
    assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
    let receipt = page_documents(&reopened)[1].clone();
    accepted(&send(
        &mut restored,
        P::from_avro(&ltc_preview(&reopened, &receipt).to_avro()).unwrap(),
    ));
    assert_eq!(restored.draft, saved);
    accepted(&send(
        &mut restored,
        P::from_avro(&reopened.to_avro()).unwrap(),
    ));
    assert_eq!(restored.draft, saved);
    assert_eq!(supported(&restored.draft), result);
}

fn trump_return(principal: i64, earnings: i64) -> TrumpExcessReturn {
    TrumpExcessReturn {
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
        OtherTax::HouseholdEmployees {
            employers: vec![household_employer(3_500)],
        },
        OtherTax::TrumpAccounts {
            accounts: vec![
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
            ],
        },
    ];
    (session, page, before_line23)
}

fn trump_estimate(page: &P) -> P {
    let P::SaveOtherTaxes { value, .. } = page else {
        unreachable!()
    };
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
        let P::SaveOtherTaxes { value: taxes, .. } = &mut request else {
            unreachable!()
        };
        let OtherTax::TrumpAccounts { accounts } = &mut taxes.taxes[1] else {
            unreachable!()
        };
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
            (
                "TRUMP_LOSS_ABOVE_PRINCIPAL",
                "accounts/1/returns/1/earnings",
            )
        } else {
            ("TRUMP_ACCOUNT_TWICE", "accounts/1/owner")
        };
        let issue = crate::trump_account::accounts_issue(accounts).unwrap();
        assert_eq!(issue.key, key);
        assert_eq!(
            issue.field.unwrap().square(),
            alloc::format!("[{local_path}]")
        );
        for (endpoint, submitted) in [
            ("save", request.clone()),
            ("estimate", trump_estimate(&request)),
        ] {
            survey_native_refusal(
                &alloc::format!("core-trump-{case}-{endpoint}"),
                &mut session,
                submitted,
                key,
                &alloc::format!("[value/taxes/1/{local_path}]"),
            );
        }
        let before = session.draft.clone();
        accepted(&send(
            &mut session,
            P::from_avro(&trump_estimate(&correction).to_avro()).unwrap(),
        ));
        assert_eq!(session.draft, before);
        accepted(&send(
            &mut session,
            P::from_avro(&correction.to_avro()).unwrap(),
        ));
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
        let P::SaveOtherTaxes { value: shown, .. } = &reopened else {
            unreachable!()
        };
        let P::SaveOtherTaxes {
            value: expected, ..
        } = &correction
        else {
            unreachable!()
        };
        assert_eq!(shown, expected);
        accepted(&send(
            &mut session,
            P::from_avro(&reopened.to_avro()).unwrap(),
        ));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let reopened = open_other_taxes(&mut restored);
        let P::SaveOtherTaxes { value: shown, .. } = &reopened else {
            unreachable!()
        };
        assert_eq!(shown, expected);
        accepted(&send(
            &mut restored,
            P::from_avro(&reopened.to_avro()).unwrap(),
        ));
        assert_eq!(restored.draft, saved);
        assert_eq!(supported(&restored.draft), result);
    }
}

#[test]
fn unit47_nested_benefit_names_are_output_only_without_emptying_editable_rows() {
    let partners = partner_benefits_session(vec![k1_1065_benefits(
        "Synthetic display-only partner",
        0,
        0,
        0,
    )]);
    let (corporations, reply) = save_k1_1120s(vec![k1_1120s("Synthetic display-only corporation")]);
    accepted(&reply);
    for (kind, mut session) in [
        ("business", business_session()),
        ("partner", partners),
        ("corporation", corporations),
    ] {
        let request = benefits_editor(&mut session);
        accepted(&send(&mut session, request));
        let baseline = session.draft.clone();
        let expected_estimate = estimate(&baseline);
        for forged in [false, true] {
            let mut request = benefits_editor(&mut session);
            let P::SaveSelfEmployedBenefits { value, .. } = &mut request else {
                unreachable!()
            };
            let names: Vec<_> = match kind {
                "business" => value.businesses.iter_mut().map(|r| &mut r.name).collect(),
                "partner" => value.partnerships.iter_mut().map(|r| &mut r.name).collect(),
                "corporation" => value
                    .scorporations
                    .iter_mut()
                    .map(|r| &mut r.name)
                    .collect(),
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
    let request = save_topic(
        &session.draft,
        TaxSection::Wages,
        vec![document(0, IncomeDocument::W2(w2(10_000_000)))],
    );
    accepted(&send(&mut session, request));
    let baseline = session.draft.clone();
    let expected_estimate = estimate(&baseline);
    for forged in [false, true] {
        let request = go_to(&session.draft, PAGE_INCOME, TaxSection::Wages);
        let Reply::Edit(mut request @ P::SaveIncomes { .. }) = send(&mut session, request) else {
            panic!("expected the actual wages editor")
        };
        let P::SaveIncomes { documents, .. } = &mut request else {
            unreachable!()
        };
        assert_eq!(documents.documents.len(), 1);
        let item = &mut documents.documents[0];
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
    assert!(matches!(
        send(session, request),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn unit47_student_interest_admission_uses_all_actual_form_interest_and_reopens_the_correction() {
    for (case, forms, other, employer, correction) in [
        (
            "student-interest-additional",
            vec![Form1098E {
                interest: 180_000,
                box_2: Form1098EBox2::Checked {
                    additional_interest: 20_000,
                },
            }],
            10_000,
            210_001,
            210_000,
        ),
        ("student-interest-no-form", vec![], 10_000, 10_001, 10_000),
        ("student-interest-zero", vec![], 0, 1, 0),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let navigation = go_to(&session.draft, PAGE_REVIEW, TaxSection::StudentLoan);
        let Reply::Edit(mut request @ P::SaveStudentLoan { .. }) = send(&mut session, navigation)
        else {
            panic!("expected actual student-loan editor")
        };
        let P::SaveStudentLoan { value, .. } = &mut request else {
            unreachable!()
        };
        value.forms = forms.clone();
        value.other_interest = other;
        value.employer_paid_interest = employer;
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
        let P::SaveStudentLoan { value, .. } = &mut request else {
            unreachable!()
        };
        value.employer_paid_interest = correction;
        let expected = value.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        let request = P::from_avro(&request.to_avro()).unwrap();
        accepted(&send(&mut session, request));
        assert_eq!(
            session.draft.adjustments.as_ref().unwrap().student_loan,
            StudentLoanClaim {
                student_loan_interest: other,
                forms,
                employer_paid_interest: correction,
            }
        );
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let navigation = go_to(&restored.draft, PAGE_REVIEW, TaxSection::StudentLoan);
        let Reply::Edit(request @ P::SaveStudentLoan { .. }) = send(&mut restored, navigation)
        else {
            unreachable!()
        };
        let P::SaveStudentLoan { value, .. } = &request else {
            unreachable!()
        };
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
        let mut session = mortgage_loan_page(at_the_deductions_page(FederalStatus::Single).0);
        let mut request = open_itemized_topic(&mut session, TaxSection::Taxes);
        let P::SaveTaxes { value, .. } = &mut request else {
            unreachable!()
        };
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
        let P::SaveTaxes { value, .. } = &mut request else {
            unreachable!()
        };
        value.kind = SaltElection::IncomeTax { paid: 50_000 };
        value.investment_income_tax = crate::InvestmentIncomeTax::Records {
            paid_part: 50_000,
            capped_allocation: crate::InvestmentTaxCapAllocation::SourceDetermined,
        };
        let expected = value.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        accepted(&send(
            &mut session,
            P::from_avro(&request.to_avro()).unwrap(),
        ));
        assert_eq!(
            stored_facts(&session).investment_income_tax,
            expected.investment_income_tax
        );
        assert_eq!(stored_facts(&session).personal_tax.paid(), 50_000);
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_itemized_topic(&mut restored, TaxSection::Taxes);
        let P::SaveTaxes { value, .. } = &request else {
            unreachable!()
        };
        assert_eq!(*value, expected);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

fn unit47_mortgage_row(label: &str) -> Mortgage1098 {
    Mortgage1098 {
        id: 0,
        interest: 100_000,
        principal: 20_000_000,
        originated: 20200601,
        refund: 0,
        insurance: 0,
        points: Points::None,
        more: MortgageMore {
            property: label.into(),
            details: MortgageDetails::FirstLender(MortgageFirstLender {
                home: MortgageHome::Main,
                purpose: MortgageUse::Home,
                average: AverageBalance::FirstAndLast {
                    last_balance: 20_000_000,
                },
            }),
            ..MortgageMore::default()
        },
    }
}

fn unit47_mortgage_session() -> Interview {
    let mut session = at_the_deductions_page(FederalStatus::Single).0;
    // This $3,000 interest fixture is below the standard deduction. Select
    // Itemize through its actual editor so Schedule A claims the tested amount.
    unit47_admission_itemize(&mut session);
    assert_eq!(
        session.draft.deductions.as_ref().unwrap().choice(),
        DeductionChoice::Itemized
    );
    let mut request = open_itemized_topic(&mut session, TaxSection::Mortgage);
    let P::SaveMortgages {
        value: mortgages, ..
    } = &mut request
    else {
        unreachable!()
    };
    mortgages.loans = vec![
        unit47_mortgage_row("First source"),
        unit47_mortgage_row("Second source"),
        unit47_mortgage_row("Third source"),
    ];
    accepted(&send(&mut session, request));
    assert_eq!(
        value(&estimate(&session.draft), "mortgage_interest_8a"),
        3_000
    );
    session
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
        let mut session = unit47_mortgage_session();
        let mut request = open_itemized_topic(&mut session, TaxSection::Mortgage);
        let P::SaveMortgages { value, .. } = &mut request else {
            unreachable!()
        };
        assert_eq!(value.loans.len(), 3);
        assert!(value.loans.iter().all(|loan| loan.id > 0));
        let original = value.clone();
        let first_id = value.loans[0].id;
        let second_id = value.loans[1].id;
        let third_id = value.loans[2].id;
        let (key, path) = match case {
            "last-balance" | "last-before-points" => {
                let MortgageDetails::FirstLender(facts) = &mut value.loans[1].more.details else {
                    unreachable!()
                };
                facts.average = AverageBalance::FirstAndLast {
                    last_balance: 20_000_001,
                };
                if case == "last-before-points" {
                    facts.home = MortgageHome::Second;
                    value.loans[1].points = Points::Purchase { amount: 10_000 };
                }
                (
                    "MORTGAGE_LAST_BALANCE_ABOVE_BOX_2",
                    "value/loans/1/more/details/average/last_balance",
                )
            }
            "purchase-points" | "improvement-points" => {
                let MortgageDetails::FirstLender(facts) = &mut value.loans[1].more.details else {
                    unreachable!()
                };
                facts.home = MortgageHome::Second;
                value.loans[1].points = if case == "purchase-points" {
                    Points::Purchase { amount: 10_000 }
                } else {
                    Points::Improvement {
                        amount: 10_000,
                        term_months: 120,
                    }
                };
                (
                    if case == "purchase-points" {
                        "MORTGAGE_PURCHASE_POINTS"
                    } else {
                        "MORTGAGE_IMPROVEMENT_POINTS"
                    },
                    "value/loans/1/points",
                )
            }
            "refinance-date" | "refinance-term" => {
                let MortgageDetails::FirstLender(facts) = &mut value.loans[1].more.details else {
                    unreachable!()
                };
                facts.purpose = MortgageUse::Refinance(MortgageRefinance {
                    refinanced: 20_000_000,
                    earlier_incurred: if case == "refinance-date" {
                        20200602
                    } else {
                        20100101
                    },
                    earlier_term_ends: None,
                    improvement: 0,
                    other: 0,
                });
                if case == "refinance-date" {
                    (
                        "MORTGAGE_EARLIER_LOAN_AFTER_THIS_ONE",
                        "value/loans/1/more/details/purpose/earlier_incurred",
                    )
                } else {
                    (
                        "MORTGAGE_EARLIER_TERM_NEEDED",
                        "value/loans/1/more/details/purpose/earlier_term_ends",
                    )
                }
            }
            "borrowed-balance" | "mixed-average" => {
                let MortgageDetails::FirstLender(facts) = &mut value.loans[1].more.details else {
                    unreachable!()
                };
                facts.purpose = MortgageUse::Partly(MortgageSplit {
                    improvement: 10_000_000,
                    improvement_incurred: 20200601,
                    other: 10_000_000,
                });
                if case == "borrowed-balance" {
                    value.loans[1].principal = 20_000_001;
                    (
                        "MORTGAGE_BALANCE_ABOVE_AMOUNTS_BORROWED",
                        "value/loans/1/principal",
                    )
                } else {
                    facts.average = AverageBalance::Statements {
                        average: 20_000_000,
                    };
                    (
                        "MORTGAGE_MIXED_USE_NEEDS_MONTHLY",
                        "value/loans/1/more/details/average",
                    )
                }
            }
            "month-duplicate" => {
                let MortgageDetails::FirstLender(facts) = &mut value.loans[1].more.details else {
                    unreachable!()
                };
                facts.average = AverageBalance::Monthly {
                    balances: vec![
                        MortgageMonthBalance {
                            month: 1,
                            balance: 20_000_000,
                        },
                        MortgageMonthBalance {
                            month: 1,
                            balance: 20_000_000,
                        },
                    ],
                };
                (
                    "MORTGAGE_MONTH_TWICE",
                    "value/loans/1/more/details/average/balances/1/month",
                )
            }
            "refund-origin" => {
                value.loans[1].refund = 100;
                (
                    "MORTGAGE_REFUND_ORIGIN_NEEDED",
                    "value/loans/1/more/refund_origin",
                )
            }
            "refund-recovery" => {
                value.loans[1].refund = 100;
                value.loans[1].more.refund_origin =
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
                    "value/loans/1/more/refund_origin/deducted",
                )
            }
            "linked-source-unavailable" => {
                value.loans[1].more.details = MortgageDetails::EarlierLender { loan: i64::MAX };
                (
                    "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
                    "value/loans/1/more/details/loan",
                )
            }
            "linked-self" => {
                value.loans[1].more.details = MortgageDetails::EarlierLender { loan: second_id };
                (
                    "MORTGAGE_LOAN_SELF_REFERENCE",
                    "value/loans/1/more/details/loan",
                )
            }
            "linked-missing" => {
                value.loans[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                value.loans.remove(0);
                (
                    "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
                    "value/loans/0/more/details/loan",
                )
            }
            "linked-not-root" => {
                value.loans[0].more.details = MortgageDetails::EarlierLender { loan: third_id };
                value.loans[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                (
                    "MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED",
                    "value/loans/1/more/details/loan",
                )
            }
            "linked-date" => {
                value.loans[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                value.loans[1].originated = 20200602;
                (
                    "MORTGAGE_LOAN_ORIGINATION_DIFFERS",
                    "value/loans/1/originated",
                )
            }
            "linked-purchase" | "linked-improvement" => {
                let MortgageDetails::FirstLender(facts) = &mut value.loans[0].more.details else {
                    unreachable!()
                };
                facts.home = MortgageHome::Second;
                value.loans[1].more.details = MortgageDetails::EarlierLender { loan: first_id };
                value.loans[1].points = if case == "linked-purchase" {
                    Points::Purchase { amount: 10_000 }
                } else {
                    Points::Improvement {
                        amount: 10_000,
                        term_months: 120,
                    }
                };
                (
                    if case == "linked-purchase" {
                        "MORTGAGE_PURCHASE_POINTS"
                    } else {
                        "MORTGAGE_IMPROVEMENT_POINTS"
                    },
                    "value/loans/1/points",
                )
            }
            "duplicate-id" => {
                value.loans[1].id = first_id;
                (
                    "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
                    "value/loans/1/id",
                )
            }
            "average-limit" => {
                for loan in &mut value.loans {
                    loan.principal = 30_000_000;
                    let MortgageDetails::FirstLender(facts) = &mut loan.more.details else {
                        unreachable!()
                    };
                    facts.average = AverageBalance::WithinLimit;
                }
                ("MORTGAGE_AVERAGE_NEEDED", "value/loans")
            }
            _ => unreachable!(),
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
        let P::SaveMortgages { value, .. } = &mut request else {
            unreachable!()
        };
        *value = original;
        value.loans[1].interest = 200_000;
        let expected = value.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        accepted(&send(
            &mut session,
            P::from_avro(&request.to_avro()).unwrap(),
        ));
        assert_eq!(
            super::value(&estimate(&session.draft), "mortgage_interest_8a"),
            4_000
        );
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_itemized_topic(&mut restored, TaxSection::Mortgage);
        let P::SaveMortgages { value, .. } = &request else {
            unreachable!()
        };
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
    let P::SaveDeductions { value, .. } = &mut request else {
        unreachable!()
    };
    value.route = DeductionRoute::Itemize;
    accepted(&send_past_less_common(session, request));
}

fn unit47_annuity_admission_session() -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut you = filer(&session.draft.people.as_ref().unwrap().taxpayer);
    you.more = MoreAboutYou::Died {
        date: 20261231,
        estate_medical: 0,
        bonds: None,
        ordinary: true,
    };
    let request = save_you(&session.draft, you);
    accepted(&send(&mut session, request));
    let mut pension = r1099(1_000_000, normal_kept(), false);
    pension.taxable = 700_000;
    let request = save_topic(
        &session.draft,
        TaxSection::Retirement,
        vec![
            document(0, IncomeDocument::R1099(pension.clone())),
            document(0, IncomeDocument::R1099(pension)),
        ],
    );
    accepted(&send(&mut session, request));
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
        "refunds-overflow",
    ] {
        let mut session = unit47_annuity_admission_session();
        let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &mut request else {
            unreachable!()
        };
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
            "refunds-overflow" => {
                value.annuity_losses[1].cost = i64::MAX;
                value.annuity_losses[1].recovered = i64::MAX;
                value.annuity_losses[1].refunds = 1;
                ("ANNUITY_REFUNDS_ABOVE_COST", "refunds")
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
        let P::SaveOtherItemized { value, .. } = &mut request else {
            unreachable!()
        };
        value.annuity_losses = normal.clone();
        unit47_admission_correct_estimate(&mut session, &request);
        accepted(&send(
            &mut session,
            P::from_avro(&request.to_avro()).unwrap(),
        ));
        assert_eq!(session.draft.income, sources);
        let result = estimate(&session.draft);
        for id in &source_ids {
            // Each actual pension: $50,000 cost - $10,000 prior recovery
            // - $2,000 refund - ($10,000 gross - $7,000 taxable) = $35,000.
            assert_eq!(
                super::value(&result, &alloc::format!("annuity_loss_{id}")),
                35_000
            );
        }
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_itemized_topic(&mut restored, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &request else {
            unreachable!()
        };
        assert_eq!(value.annuity_losses, normal);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

fn unit47_bond_admission_session() -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let a = AnnualDebtRecord {
        instrument: "Computed excess premium".into(),
        amounts: DebtComponents {
            taxable: 10_000,
            bond_premium: 30_000,
            ..Default::default()
        },
        premium_history: Some(PremiumHistory {
            prior_net_interest_inclusions: 50_000,
            ..Default::default()
        }),
        ..Default::default()
    };
    let b = AnnualDebtRecord {
        instrument: "Terminal deduction from separate records".into(),
        amounts: DebtComponents {
            taxable: 100_000,
            ..Default::default()
        },
        premium_history: Some(PremiumHistory::default()),
        ..Default::default()
    };
    let interest = Int1099 {
        payer: "Synthetic premium source".into(),
        taxable: 110_000,
        more: IntMore {
            bond_premium: 30_000,
            ..Default::default()
        },
        after_death: None,
        state_details: Default::default(),
        nominee: None,
        adjustments: Some(InterestAdjustments {
            instrument_records: Some(AnnualDebtRecords {
                records: vec![a, b],
            }),
            ..Default::default()
        }),
    };
    let request = save_topic(
        &session.draft,
        TaxSection::InterestDividends,
        vec![
            document(0, IncomeDocument::Int1099(interest)),
            document(0, IncomeDocument::TaxableBondPremium(true)),
        ],
    );
    accepted(&send(&mut session, request));
    unit47_admission_itemize(&mut session);
    session
}

#[test]
fn unit47_manual_bond_premium_admission_retains_actual_instrument_sources_and_amounts() {
    for case in [
        "duplicate",
        "already-computed",
        "missing",
        "description",
        "computed-before-duplicate",
    ] {
        let mut session = unit47_bond_admission_session();
        let mut request = open_itemized_topic(&mut session, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &mut request else {
            unreachable!()
        };
        assert_eq!(value.bond_instruments.len(), 2);
        let computed = value
            .bond_instruments
            .iter()
            .find(|source| source.computed_deduction > 0)
            .unwrap()
            .id;
        let manual = value
            .bond_instruments
            .iter()
            .find(|source| source.computed_deduction == 0)
            .unwrap()
            .id;
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
                        source: BondPremiumSource::OtherDocumented {
                            instrument: " ".into(),
                        }
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
        let P::SaveOtherItemized { value, .. } = &mut request else {
            unreachable!()
        };
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
        accepted(&send(
            &mut session,
            P::from_avro(&request.to_avro()).unwrap(),
        ));
        assert_eq!(session.draft.income, sources);
        // Existing sole premium owner: $200 computed + $50 documented
        // accepted instrument + $70 separate documented source = $320 once.
        assert_eq!(
            super::value(&estimate(&session.draft), "bond_premium_deduction"),
            320
        );
        let saved = session.draft.clone();
        let mut restored = restart(saved.clone());
        let request = open_itemized_topic(&mut restored, TaxSection::OtherItemized);
        let P::SaveOtherItemized { value, .. } = &request else {
            unreachable!()
        };
        assert_eq!(value.other_lines, expected);
        accepted(&send(&mut restored, request));
        assert_eq!(restored.draft, saved);
    }
}

fn unit47_farm_production_rows(document: &IncomeDocument) -> &[FarmProductionCohortInput] {
    match document {
        IncomeDocument::Farm(farm) => &farm.more.specials.production,
        IncomeDocument::FarmRental(farm) => &farm.more.specials.production,
        _ => panic!("expected an actual farm production document"),
    }
}

fn unit47_farm_production_rows_mut(
    document: &mut IncomeDocument,
) -> &mut Vec<FarmProductionCohortInput> {
    match document {
        IncomeDocument::Farm(farm) => &mut farm.more.specials.production,
        IncomeDocument::FarmRental(farm) => &mut farm.more.specials.production,
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
    let request = farm_save(
        &session.draft,
        vec![
            document(0, IncomeDocument::Farm(farm)),
            document(0, IncomeDocument::FarmRental(rental)),
        ],
    );
    accepted(&send(
        &mut session,
        P::from_avro(&request.to_avro()).unwrap(),
    ));
    let mut request = open_topic(&mut session, TaxSection::Farming);
    let P::SaveIncomes {
        allowed_documents,
        documents,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    let ids: Vec<_> = documents
        .documents
        .iter()
        .filter(|item| {
            matches!(item.document, IncomeDocument::Farm(_) | IncomeDocument::FarmRental(_))
        })
        .flat_map(|item| unit47_farm_production_rows(&item.document).iter().map(|row| row.id))
        .collect();
    assert_eq!(ids.len(), 4);
    assert!(ids.iter().all(|id| *id > 0));
    assert_eq!(
        ids.iter()
            .copied()
            .collect::<alloc::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert!(
        allowed_documents
            .contains(&(libertas_macros::variant_index!(IncomeDocument::FarmAveraging) as i32))
    );
    // A genuine offered data answer will precede the target on the submitted
    // unordered page, then be partitioned away from the income-entry vector.
    documents.documents.push(document(
        0,
        IncomeDocument::FarmAveraging(averaging_election()),
    ));
    accepted(&send(
        &mut session,
        P::from_avro(&request.to_avro()).unwrap(),
    ));
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
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    let original_documents = documents.clone();
    let answer_index = documents
        .documents
        .iter()
        .position(|item| matches!(item.document, IncomeDocument::FarmAveraging(_)))
        .unwrap();
    let answer = documents.documents.remove(answer_index);
    let target_index = documents
        .documents
        .iter()
        .position(|item| match kind {
            "farm" => matches!(item.document, IncomeDocument::Farm(_)),
            "farm-rental" => matches!(item.document, IncomeDocument::FarmRental(_)),
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
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    let mut corrected = original.clone();
    corrected.reverse();
    // The negative reordered submission proves its original index. Correction
    // returns to the normal editor order while reordering the actual cohorts.
    *documents = original_documents;
    let target = documents
        .documents
        .iter_mut()
        .find(|item| match kind {
            "farm" => matches!(item.document, IncomeDocument::Farm(_)),
            "farm-rental" => matches!(item.document, IncomeDocument::FarmRental(_)),
            _ => unreachable!(),
        })
        .unwrap();
    *unit47_farm_production_rows_mut(&mut target.document) = corrected.clone();
    let before = session.draft.clone();
    let dirty = session.dirty;
    accepted(&send(
        &mut session,
        P::from_avro(&estimate_documents_from_save(&request).to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(
        &mut session,
        P::from_avro(&request.to_avro()).unwrap(),
    ));
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
    let P::SaveIncomes { documents, .. } = &reopened else {
        unreachable!()
    };
    let shown = documents
        .documents
        .iter()
        .find(|item| match kind {
            "farm" => matches!(item.document, IncomeDocument::Farm(_)),
            "farm-rental" => matches!(item.document, IncomeDocument::FarmRental(_)),
            _ => unreachable!(),
        })
        .unwrap();
    assert_eq!(
        unit47_farm_production_rows(&shown.document),
        corrected.as_slice()
    );
    accepted(&send(
        &mut session,
        P::from_avro(&reopened.to_avro()).unwrap(),
    ));
    assert_eq!(session.draft, saved);
    let mut restored = restart(saved.clone());
    let reopened = open_topic(&mut restored, TaxSection::Farming);
    let P::SaveIncomes { documents, .. } = &reopened else {
        unreachable!()
    };
    let shown = documents
        .documents
        .iter()
        .find(|item| match kind {
            "farm" => matches!(item.document, IncomeDocument::Farm(_)),
            "farm-rental" => matches!(item.document, IncomeDocument::FarmRental(_)),
            _ => unreachable!(),
        })
        .unwrap();
    assert_eq!(
        unit47_farm_production_rows(&shown.document),
        corrected.as_slice()
    );
    accepted(&send(
        &mut restored,
        P::from_avro(&reopened.to_avro()).unwrap(),
    ));
    assert_eq!(restored.draft, saved);
    assert_eq!(supported(&restored.draft), result);
}
