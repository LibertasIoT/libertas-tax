use crate::k1_activity::*;
use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::builders::k1res::{documents, input};
use crate::tests::support::client::income_pages::catalog_preview_document;
use crate::tests::support::client::k1res::financial_property_session;
use crate::tests::*;

#[test]
fn bound_properties_reduce_entity_basis_once_and_reach_native_credit_lifecycle() {
    for scorp in [false, true] {
        let mut session = financial_property_session(scorp);
        let initial_review = send(&mut session, P::ReviewAccepted { user: TEST_USER });
        let saved = session.draft.clone();
        let source_index = saved
            .income
            .iter()
            .position(|entry| {
                matches!(
                    entry.income,
                    FederalIncome::PartnershipActivities { .. }
                        | FederalIncome::ScorpActivities { .. }
                )
            })
            .unwrap();
        let id = saved.income[source_index].id;
        let kind = if scorp { "scorp" } else { "partnership" };
        let output = match &saved.income[source_index].income {
            FederalIncome::PartnershipActivities { data } => data.ordinary_source(false),
            FederalIncome::ScorpActivities { data } => data.ordinary_source(false),
            _ => unreachable!(),
        }
        .unwrap();
        // $1,000 opening + $4,000 income - ($100 + $200) canonical
        // reductions leaves $4,700 for the second component's $6,000 loss.
        assert_eq!(crate::kernel::ratio::dollars(output.basis.ending.get()), 0);
        assert_eq!(crate::kernel::ratio::dollars(output.components[1].basis_allowed.get()), 4_700);
        assert_eq!(
            crate::kernel::ratio::dollars(output.components[1].basis_suspended.get()),
            1_300
        );
        assert_eq!(crate::kernel::ratio::dollars(output.components[1].risk_allowed.get()), 4_700);
        let alternative = match &saved.income[source_index].income {
            FederalIncome::PartnershipActivities { data } => data.ordinary_source(true),
            FederalIncome::ScorpActivities { data } => data.ordinary_source(true),
            _ => unreachable!(),
        }
        .unwrap();
        assert_eq!(crate::kernel::ratio::dollars(alternative.basis.ending.get()), 0);
        assert_eq!(
            crate::kernel::ratio::dollars(alternative.components[1].basis_allowed.get()),
            4_700
        );
        let result = estimate(&saved);
        assert!(result.tax.is_some(), "{:?}", result.issues);
        assert_eq!(line_amount(&result, &alloc::format!("{kind}_{id}_schedule_e")), -70_000);
        assert_eq!(line_amount(&result, &alloc::format!("amt_{kind}_{id}_schedule_e")), -70_000);
        assert_eq!(line_amount(&result, "agi"), 5_930_000);
        assert_eq!(line_amount(&result, "business_credit_energy_current_year"), 60_000);
        assert_eq!(line_amount(&result, "general_business_credit"), 60_000);
        assert_eq!(line_amount(&result, "business_credit_energy_current_year_used"), 60_000);
        let property_rows: Vec<_> = result
            .lines
            .iter()
            .filter(|line| {
                line.id.starts_with(&alloc::format!("k1_activity_{id}_investment_"))
                    && line.id.ends_with("_form3468_credit")
            })
            .collect();
        assert_eq!(property_rows.len(), 2);
        assert_eq!(property_rows.iter().map(|line| line.amount).sum::<i64>(), 60_000);
        assert_eq!(
            result
                .lines
                .iter()
                .filter(|line| line.id.starts_with(&alloc::format!("k1_activity_{id}_investment_"))
                    && line.id.ends_with("_basis_reduction"))
                .map(|line| line.amount)
                .sum::<i64>(),
            30_000
        );

        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        assert_eq!(documents(&mut request)[0].document, input(&saved.income[source_index].income));
        let dirty = session.dirty;
        let mut preview = catalog_preview_document(documents(&mut request)[0].document.clone());
        let P::PreviewDocument { id: preview_id, .. } = &mut preview else { unreachable!() };
        *preview_id = id;
        assert!(matches!(
            send(&mut session, preview),
            Reply::Response(P::IncomePreview { preview: Some(-70_000), .. })
        ));
        assert_eq!(session.draft, saved);
        assert_eq!(session.dirty, dirty);
        let properties = match &mut documents(&mut request)[0].document {
            IncomeDocument::K1PartnershipActivities(page) => &mut page.entity.investment_property,
            IncomeDocument::K1ScorpActivities(page) => &mut page.entity.investment_property,
            _ => unreachable!(),
        };
        let K1InvestmentProperty::Energy { own_form: Some(own), .. } = &mut properties[0] else {
            unreachable!()
        };
        own.amount = 40_000;
        let candidate_before = session.draft.clone();
        let running = send(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request),
        );
        assert!(matches!(running, Reply::Response(P::IncomeRunningEstimate { .. })), "{running:?}");
        assert_eq!(session.draft, candidate_before);
        accepted(&send(&mut session, request));
        assert_eq!(session.draft.revision, saved.revision + 1);
        assert_eq!(session.draft.next_id, saved.next_id);
        assert_eq!(
            session
                .draft
                .income
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != source_index)
                .collect::<Vec<_>>(),
            saved.income.iter().enumerate().filter(|(i, _)| *i != source_index).collect::<Vec<_>>()
        );
        let corrected = estimate(&session.draft);
        assert!(corrected.tax.is_some(), "{:?}", corrected.issues);
        // The corrected $800 credit reduces basis $400 once: $4,600 loss.
        assert_eq!(line_amount(&corrected, &alloc::format!("{kind}_{id}_schedule_e")), -60_000);
        assert_eq!(line_amount(&corrected, "agi"), 5_940_000);
        assert_eq!(line_amount(&corrected, "general_business_credit"), 80_000);
        let corrected_review = send(&mut session, P::ReviewAccepted { user: TEST_USER });
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let cold_review = send(&mut cold, P::ReviewAccepted { user: TEST_USER });
        let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
        assert_eq!(documents(&mut reopened)[0].document, input(&saved.income[source_index].income));
        assert_eq!(estimate(&cold.draft), corrected);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        let resaved_review = send(&mut cold, P::ReviewAccepted { user: TEST_USER });
        for (reply, credit, reduction) in [
            (initial_review, 60_000, 30_000),
            (corrected_review, 80_000, 40_000),
            (cold_review, 80_000, 40_000),
            (resaved_review, 80_000, 40_000),
        ] {
            let Reply::Response(P::Review { result, .. }) = reply else {
                panic!("expected the ordinary accepted-return Review")
            };
            let prefix = alloc::format!("K-1 activity {id} investment ");
            let property_rows: Vec<_> = rows(&result.breakdown)
                .into_iter()
                .filter(|row| row.label.starts_with(&prefix))
                .collect();
            assert_eq!(property_rows.len(), 6);
            for (suffix, expected) in [
                ("Form 3468 credit base", None),
                ("Form 3468 credit", Some(credit)),
                ("investment credit basis reduction", Some(reduction)),
            ] {
                let matching: Vec<_> =
                    property_rows.iter().filter(|row| row.label.ends_with(suffix)).collect();
                assert_eq!(matching.len(), 2, "{suffix}");
                if let Some(expected) = expected {
                    assert_eq!(matching.iter().map(|row| row.amount).sum::<i64>(), expected);
                }
            }
            assert!(!result.return_lines.is_empty());
        }
    }
}

#[test]
fn missing_or_unsupported_property_binding_never_becomes_a_credit_or_zero_fact() {
    for scorp in [false, true] {
        let session = financial_property_session(scorp);
        for control in 0..7 {
            let mut candidate = session.draft.clone();
            let income = &mut candidate
                .income
                .iter_mut()
                .find(|entry| {
                    matches!(
                        entry.income,
                        FederalIncome::PartnershipActivities { .. }
                            | FederalIncome::ScorpActivities { .. }
                    )
                })
                .unwrap()
                .income;
            let (source, properties, confirmed) = match income {
                FederalIncome::PartnershipActivities { data } => (
                    &mut data.activities,
                    &mut data.entity.investment_property,
                    &mut data.entity.investment_confirmed,
                ),
                FederalIncome::ScorpActivities { data } => (
                    &mut data.activities,
                    &mut data.entity.investment_property,
                    &mut data.entity.investment_confirmed,
                ),
                _ => unreachable!(),
            };
            let K1ComponentStatement::Entered { source } = source else { unreachable!() };
            match control {
                0 => source.investment_property_sources = None,
                1 => source.investment_property_sources.as_mut().unwrap()[0].component = i64::MAX,
                2 => source.passive_groups[0].participation = K1ActivityParticipation::Unavailable,
                3 => {
                    let id = properties[0].id();
                    properties[0] = K1InvestmentProperty::RehabilitationRental {
                        expenditures: 1_000_000,
                        placed_year: 2026,
                        id,
                        own_form: None,
                    };
                    *confirmed = Some(true);
                }
                4 => {
                    for property in properties {
                        let K1InvestmentProperty::Energy { own_form, .. } = property else {
                            unreachable!()
                        };
                        *own_form = None;
                    }
                    *confirmed = None;
                }
                5 => match &mut source.passive_groups[0].participation {
                    K1ActivityParticipation::Partnership {
                        facts:
                            PartnerParticipation::Limited {
                                minutes,
                                prior_ten,
                                personal_service,
                                service_years,
                            },
                        ..
                    } => {
                        *minutes = 0;
                        *prior_ten = 0;
                        *personal_service = false;
                        *service_years = 0;
                    }
                    K1ActivityParticipation::Scorp { facts, .. }
                    | K1ActivityParticipation::Partnership {
                        facts: PartnerParticipation::General { facts },
                        ..
                    } => {
                        facts.minutes = 0;
                        facts.substantially_all = false;
                        facts.at_least_as_much = false;
                        facts.prior_ten = 0;
                        facts.personal_service = false;
                        facts.service_years = 0;
                        facts.regular = false;
                        facts.management_minutes = 0;
                        facts.other_business_over_100 = false;
                    }
                    _ => unreachable!(),
                },
                6 => {
                    candidate.setup = complete(FederalStatus::Separate).setup;
                    candidate.setup.as_mut().unwrap().community_property = community_arizona();
                    for entry in &mut candidate.income {
                        entry.community = CommunityShare::Separate;
                    }
                }
                _ => unreachable!(),
            }
            let result = estimate(&candidate);
            assert!(result.tax.is_none(), "control {control}: {:?}", result.issues);
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
            );
            assert!(
                !result.lines.iter().any(|line| line.id == "business_credit_energy_current_year")
            );
            assert!(!result.lines.iter().any(|line| line.id.starts_with("k1_activity_")));
        }
    }
}

#[test]
fn unconfirmed_properties_use_existing_admission_without_changing_accepted_facts() {
    for scorp in [false, true] {
        for automatic in [false, true] {
            let mut session = financial_property_session(scorp);
            let mut request = open_topic(&mut session, TaxSection::PassThrough);
            let (properties, confirmed) = match &mut documents(&mut request)[0].document {
                IncomeDocument::K1PartnershipActivities(page) => {
                    (&mut page.entity.investment_property, &mut page.entity.investment_confirmed)
                }
                IncomeDocument::K1ScorpActivities(page) => {
                    (&mut page.entity.investment_property, &mut page.entity.investment_confirmed)
                }
                _ => unreachable!(),
            };
            for property in properties {
                let K1InvestmentProperty::Energy { own_form, .. } = property else {
                    unreachable!()
                };
                if automatic {
                    *own_form = None;
                } else {
                    own_form.as_mut().unwrap().reduced = false;
                }
            }
            *confirmed = Some(crate::K1InvestmentConfirmation::No);
            accepted(&send(&mut session, request));
            let saved = session.draft.clone();
            let source_index = saved
                .income
                .iter()
                .position(|entry| {
                    matches!(
                        entry.income,
                        FederalIncome::PartnershipActivities { .. }
                            | FederalIncome::ScorpActivities { .. }
                    )
                })
                .unwrap();
            let result = estimate(&saved);
            assert!(result.tax.is_some(), "{:?}", result.issues);
            assert_eq!(line_amount(&result, "agi"), 5_900_000);
            // No admitted credit source means the credit owner emits no GBC
            // or property-credit rows, rather than manufacturing a zero claim.
            assert!(!result.lines.iter().any(|line| line.id == "general_business_credit"
                || line.id == "business_credit_energy_current_year"
                || line.id.starts_with("k1_activity_")));
            assert!(noted(
                &result,
                if automatic {
                    "WARN_K1_INVESTMENT_CREDIT_LEFT_OUT"
                } else {
                    "WARN_ENTERED_BUSINESS_CREDIT_LEFT_OUT"
                }
            ));
            assert_eq!(session.draft, saved);
            let mut cold = restart(saved.clone());
            let mut request = open_topic(&mut cold, TaxSection::PassThrough);
            assert_eq!(
                documents(&mut request)[0].document,
                input(&saved.income[source_index].income)
            );
            accepted(&send(&mut cold, request));
            assert_eq!(cold.draft, saved);
            assert_eq!(estimate(&cold.draft), result);
        }
    }
}

#[test]
fn confirmed_automatic_properties_use_the_same_bound_financial_path() {
    for scorp in [false, true] {
        let mut session = financial_property_session(scorp);
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let (properties, confirmed) = match &mut documents(&mut request)[0].document {
            IncomeDocument::K1PartnershipActivities(page) => {
                (&mut page.entity.investment_property, &mut page.entity.investment_confirmed)
            }
            IncomeDocument::K1ScorpActivities(page) => {
                (&mut page.entity.investment_property, &mut page.entity.investment_confirmed)
            }
            _ => unreachable!(),
        };
        for property in properties {
            let K1InvestmentProperty::Energy { own_form, .. } = property else { unreachable!() };
            *own_form = None;
        }
        *confirmed = Some(crate::K1InvestmentConfirmation::Yes);
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        let entry = saved
            .income
            .iter()
            .find(|entry| {
                matches!(
                    entry.income,
                    FederalIncome::PartnershipActivities { .. }
                        | FederalIncome::ScorpActivities { .. }
                )
            })
            .unwrap();
        let output = match &entry.income {
            FederalIncome::PartnershipActivities { data } => data.ordinary_source(false),
            FederalIncome::ScorpActivities { data } => data.ordinary_source(false),
            _ => unreachable!(),
        }
        .unwrap();
        // Source bases $10,000.01 and $20,000.01 at 30% produce $3,000
        // and $6,000 credits. Their half-credit basis reductions total $4,500,
        // leaving $500 of the $6,000 loss allowed after $4,000 profit.
        assert_eq!(crate::kernel::ratio::dollars(output.components[1].basis_allowed.get()), 500);
        assert_eq!(
            crate::kernel::ratio::dollars(output.components[1].basis_suspended.get()),
            5_500
        );
        assert_eq!(crate::kernel::ratio::dollars(output.components[1].risk_allowed.get()), 500);
        let result = estimate(&saved);
        assert!(result.tax.is_some(), "{:?}", result.issues);
        assert_eq!(line_amount(&result, "agi"), 6_350_000);
        assert_eq!(line_amount(&result, "business_credit_energy_current_year"), 900_000);
        let used = line_amount(&result, "general_business_credit");
        assert!(used > 0);
        assert_eq!(
            used + line_amount(&result, "business_credit_energy_current_year_to_next_year"),
            900_000
        );
        assert!(!noted(&result, "WARN_K1_INVESTMENT_CREDIT_LEFT_OUT"));
        let mut cold = restart(saved.clone());
        let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
        assert_eq!(documents(&mut reopened)[0].document, input(&entry.income));
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
    }
}
