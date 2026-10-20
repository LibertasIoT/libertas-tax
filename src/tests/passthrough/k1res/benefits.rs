use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::{assert::k1res::*, builders::k1res::*, client::k1res::*};
use crate::tests::*;

#[test]
fn k1res_benefit_writer_retains_only_known_payment_and_preserves_it_through_source_corrections() {
    let mut session = partner_benefit_session();
    let mut request = open_activity_benefits(&mut session);
    let value = activity_benefits(&mut request);
    assert!(value.partnerships.is_empty());
    assert_eq!(value.activity_partnerships.len(), 1);
    let row = &mut value.activity_partnerships[0];
    assert_eq!(row.id, session.draft.income[0].id);
    assert_eq!((row.medical_insurance, row.pensions, row.from_distributions), (None, None, None));
    assert_eq!((&row.health, &row.retirement), (&None, &None));
    row.from_distributions = Some(30_000);
    accepted(&send(&mut session, request));
    assert_eq!(
        partner_source(&session).person,
        Some(PartnerPersonSource { health_distributions: Some(30_000), ..Default::default() })
    );
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(data) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    assert_eq!(data.person.as_ref().unwrap().health_distributions, Some(30_000));
    data.person = None;
    accepted(&send(&mut session, request));
    assert_eq!(
        partner_source(&session).person,
        Some(PartnerPersonSource { health_distributions: Some(30_000), ..Default::default() })
    );
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(data) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    data.person.as_mut().unwrap().health_distributions = Some(30_001);
    assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_BENEFIT_PAYMENT_SOURCE_CHANGED");
    let saved = session.draft.clone();
    let mut restarted = restart(saved.clone());
    assert_eq!(restarted.draft, saved);
    let mut request = open_activity_benefits(&mut restarted);
    assert_eq!(
        activity_benefits(&mut request).activity_partnerships[0].from_distributions,
        Some(30_000)
    );
    accepted(&send(&mut restarted, request));
    assert_eq!(partner_source(&restarted).person, partner_source(&session).person);
    assert!(crate::federal_rules::estimate(&restarted.draft).tax.is_none());
}

#[test]
fn k1res_fuel_availability_retains_known_payment_and_nullable_basis_through_both_writers() {
    let mut session = partner_benefit_session();
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].from_distributions = Some(30_000);
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(v) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    v.entity.regular.opening = None;
    v.entity.regular.prior_nondeductible = None;
    v.entity.regular.complete = false;
    let person = v.person.as_mut().unwrap();
    assert_eq!(person.fuel_tax_credit, None);
    assert_eq!(person.prior_fuel_credit, None);
    person.prior_fuel_credit = Some(K1PriorFuelSource::NotReported);
    accepted(&send(&mut session, request));
    let person = partner_source(&session).person.as_ref().unwrap();
    assert_eq!(person.health_distributions, Some(30_000));
    assert_eq!(person.fuel_tax_credit, None);
    assert_eq!(person.prior_fuel_credit, Some(K1PriorFuelSource::NotReported));
    let mut restarted = restart(session.draft.clone());
    let mut request = open_topic(&mut restarted, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(v) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    let person = v.person.as_mut().unwrap();
    person.fuel_tax_credit = Some(K1CurrentFuelSource::Reported { source: current_fuel_source() });
    person.prior_fuel_credit = Some(K1PriorFuelSource::Reported { source: prior_fuel_source() });
    accepted(&send(&mut restarted, request));
    let saved = restarted.draft.income[0].clone();
    let mut request = open_activity_benefits(&mut restarted);
    activity_benefits(&mut request).activity_partnerships[0].from_distributions = Some(20_000);
    accepted(&send(&mut restarted, request));
    let source = partner_source(&restarted);
    assert_eq!(source.entity.regular.opening, None);
    assert_eq!(source.entity.regular.prior_nondeductible, None);
    let person = source.person.as_ref().unwrap();
    assert_eq!(person.health_distributions, Some(20_000));
    assert_eq!(
        person.fuel_tax_credit,
        Some(K1CurrentFuelSource::Reported { source: current_fuel_source() })
    );
    assert_eq!(
        person.prior_fuel_credit,
        Some(K1PriorFuelSource::Reported { source: prior_fuel_source() })
    );
    let mut request = open_topic(&mut restarted, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(v) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    let K1PriorFuelSource::Reported { source } =
        v.person.as_mut().unwrap().prior_fuel_credit.as_mut().unwrap()
    else {
        unreachable!()
    };
    source.claimed_credit = 0;
    assert_atomic_rejection(&mut restarted, request, "K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
    let mut request = open_topic(&mut restarted, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(v) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    let K1CurrentFuelSource::Reported { source } =
        v.person.as_mut().unwrap().fuel_tax_credit.as_mut().unwrap()
    else {
        unreachable!()
    };
    source.fuels.gasoline_off_highway.as_mut().unwrap().thousandths_of_gallon = 0;
    assert_atomic_rejection(&mut restarted, request, "K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
    let again = restart(restarted.draft.clone());
    assert_eq!(again.draft, restarted.draft);
    assert!(crate::federal_rules::estimate(&again.draft).tax.is_none());
    assert_ne!(again.draft.income[0], saved); // The genuine benefit correction is retained.
}

#[test]
fn k1res_benefit_source_relations_reject_atomically_and_readonly_membership_is_authorized() {
    let mut session = partner_benefit_session();
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].from_distributions = Some(50_001);
    assert_atomic_rejection(&mut session, request, "PARTNER_DISTRIBUTION_PREMIUMS_ABOVE_19A");
    let mut request = open_activity_benefits(&mut session);
    let row = &mut activity_benefits(&mut request).activity_partnerships[0];
    row.from_distributions = Some(30_000);
    row.health = Some(SeHealth::None);
    assert_atomic_rejection(&mut session, request, "PARTNER_PREMIUMS_BELOW_K1");
    for identity in [0, session.draft.next_id] {
        let mut request = open_activity_benefits(&mut session);
        activity_benefits(&mut request).activity_partnerships[0].id = identity;
        assert_atomic_rejection(
            &mut session,
            request,
            "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN",
        );
    }
    let mut request = open_activity_benefits(&mut session);
    let rows = &mut activity_benefits(&mut request).activity_partnerships;
    rows.push(rows[0].clone());
    assert_atomic_rejection(
        &mut session,
        request,
        "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN",
    );
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships.clear();
    assert_atomic_rejection(
        &mut session,
        request,
        "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN",
    );
}

#[test]
fn k1res_source_pension_correction_precedes_plan_correction_without_erasing_known_facts() {
    let mut session = partner_benefit_session();
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(data) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    data.person = Some(PartnerPersonSource { pensions: Some(10_000), ..Default::default() });
    accepted(&send(&mut session, request));
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].retirement = Some(PartnerPlan::None);
    assert_atomic_rejection(&mut session, request, "PARTNER_BOX_13R_NEEDS_PLAN");
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].retirement =
        Some(PartnerPlan::Sep { rate: 2_500 });
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(data) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    data.person.as_mut().unwrap().pensions = Some(0);
    accepted(&send(&mut session, request));
    assert_eq!(partner_source(&session).retirement, Some(PartnerRetirement::Sep { rate: 2_500 }));
    let request = open_activity_benefits(&mut session);
    assert_atomic_rejection(&mut session, request, "PARTNER_PLAN_WITHOUT_BOX_13R");
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].retirement = Some(PartnerPlan::None);
    accepted(&send(&mut session, request));
    let mut restarted = restart(session.draft.clone());
    let mut request = open_activity_benefits(&mut restarted);
    let row = &activity_benefits(&mut request).activity_partnerships[0];
    assert_eq!(row.pensions, Some(0));
    assert_eq!(row.retirement, Some(PartnerPlan::None));
    assert!(crate::federal_rules::estimate(&restarted.draft).tax.is_none());
}

#[test]
fn k1res_benefit_health_sources_reuse_actual_plan_and_wage_authorization() {
    let mut partner = partner_benefit_session();
    let mut request = open_activity_benefits(&mut partner);
    let row = &mut activity_benefits(&mut request).activity_partnerships[0];
    row.health = Some(activity_health_plan(false, 0));
    row.from_distributions = Some(30_000);
    accepted(&send(&mut partner, request));
    assert!(crate::health::plans_valid(&partner.draft));
    let mut request = open_activity_benefits(&mut partner);
    activity_benefits(&mut request).activity_partnerships[0].health =
        Some(activity_health_plan(true, 999));
    assert_atomic_rejection(&mut partner, request, "PROBLEM_INVALID_HEALTH_PLAN_SOURCE");
    let (mut scorp, wage_id) = scorp_benefit_session();
    let mut request = open_activity_benefits(&mut scorp);
    let rows = &mut activity_benefits(&mut request).activity_scorporations;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].health, None);
    rows[0].health = Some(activity_health_plan(true, wage_id));
    accepted(&send(&mut scorp, request));
    assert!(crate::health::plans_valid(&scorp.draft));
    for bad in [0, scorp.draft.next_id] {
        let mut request = open_activity_benefits(&mut scorp);
        activity_benefits(&mut request).activity_scorporations[0].health =
            Some(activity_health_plan(true, bad));
        assert_atomic_rejection(&mut scorp, request, "PROBLEM_INVALID_HEALTH_PLAN_SOURCE");
    }
    let mut wrong_owner = scorp.draft.clone();
    wrong_owner.income.iter_mut().find(|v| v.id == wage_id).unwrap().owner = Owner::Spouse;
    assert!(!crate::health::plans_valid(&wrong_owner));
    let mut request = open_activity_benefits(&mut scorp);
    activity_benefits(&mut request).activity_scorporations[0].health = Some(SeHealth::None);
    accepted(&send(&mut scorp, request));
    let mut restarted = restart(scorp.draft.clone());
    let mut request = open_activity_benefits(&mut restarted);
    assert_eq!(
        activity_benefits(&mut request).activity_scorporations[0].health,
        Some(SeHealth::None)
    );
    activity_benefits(&mut request).activity_scorporations[0].health = None;
    accepted(&send(&mut restarted, request));
    let FederalIncome::ScorpActivities { data } = &restarted.draft.income[0].income else {
        unreachable!()
    };
    assert_eq!(data.health_insurance, None);
    assert!(crate::federal_rules::estimate(&restarted.draft).tax.is_none());
}

#[test]
fn k1res_benefit_locators_follow_the_actual_writer_and_nullable_derived_outputs_are_ignored() {
    let mut session = partner_benefit_session();
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].from_distributions = Some(50_001);
    assert_located_rejection(
        &mut session,
        request,
        "PARTNER_DISTRIBUTION_PREMIUMS_ABOVE_19A",
        "[value/activity_partnerships/0/from_distributions]",
    );
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].id = 0;
    assert_located_rejection(
        &mut session,
        request,
        "MESSAGE_THIS_RETURN_S_BUSINESSES_CHANGED_REOPEN_THE_EDITOR_BEFORE_TRYING_AGAIN",
        "[value/activity_partnerships/0/id]",
    );
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].from_distributions = Some(30_000);
    let P::SaveSelfEmployedBenefits { estimate, progress, .. } = &mut request else {
        unreachable!()
    };
    assert!(estimate.is_some() && progress.is_some());
    *estimate = None;
    *progress = None;
    accepted(&send(&mut session, request));
    assert_eq!(
        partner_source(&session).person,
        Some(PartnerPersonSource { health_distributions: Some(30_000), ..Default::default() })
    );
    let mut request = open_activity_benefits(&mut session);
    let P::SaveSelfEmployedBenefits { estimate, progress, .. } = &request else { unreachable!() };
    assert!(estimate.is_some() && progress.is_some());
    activity_benefits(&mut request).activity_partnerships[0].health = Some(SeHealth::None);
    assert_located_rejection(
        &mut session,
        request,
        "PARTNER_PREMIUMS_BELOW_K1",
        "[value/activity_partnerships/0/health]",
    );
    let (mut session, wage) = scorp_benefit_session();
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_scorporations[0].health =
        Some(activity_health_plan(true, wage + 1));
    assert_located_rejection(
        &mut session,
        request,
        "PROBLEM_INVALID_HEALTH_PLAN_SOURCE",
        "[value/activity_scorporations/0/health/establishment/wage]",
    );
}

#[test]
fn k1res_benefit_intrinsic_plan_and_checked_payment_refusals_are_atomic() {
    for scorp in [false, true] {
        for case in 0..if scorp { 2 } else { 4 } {
            let (mut session, wage_id) = if scorp {
                scorp_benefit_session()
            } else {
                let mut session = partner_benefit_session();
                let mut page = open_topic(&mut session, TaxSection::PassThrough);
                let IncomeDocument::K1PartnershipActivities(source) =
                    &mut documents(&mut page)[0].document
                else {
                    unreachable!()
                };
                source.person = Some(PartnerPersonSource {
                    guaranteed_services: Some(1),
                    health_guaranteed: Some(1),
                    ..Default::default()
                });
                accepted(&send(&mut session, page));
                (session, 0)
            };
            let mut good = open_activity_benefits(&mut session);
            if scorp {
                activity_benefits(&mut good).activity_scorporations[0].health =
                    Some(activity_health_plan(true, wage_id));
            } else {
                let row = &mut activity_benefits(&mut good).activity_partnerships[0];
                row.health = Some(activity_health_plan(false, 0));
                row.from_distributions = Some(30_000);
            }
            accepted(&send(&mut session, good));
            let before = session.draft.clone();
            let before_bytes = before.to_avro();
            let dirty = session.dirty;
            let good = open_activity_benefits(&mut session);
            let mut bad = good.clone();
            let value = activity_benefits(&mut bad);
            let health = if scorp {
                &mut value.activity_scorporations[0].health
            } else {
                &mut value.activity_partnerships[0].health
            };
            let (key, leaf) = match case {
                0 | 1 => {
                    let Some(SeHealth::Plan { months, .. }) = health else { unreachable!() };
                    let row = months[0].clone();
                    months.push(row);
                    if case == 0 {
                        months[0].premiums = i64::MAX;
                        months[1].premiums = i64::MAX;
                        ("PROBLEM_INVALID_HEALTH_PLAN_SOURCE", "health/months/0/premiums")
                    } else {
                        ("PROBLEM_INVALID_HEALTH_PLAN_SOURCE", "health/months/1/month")
                    }
                }
                2 => {
                    value.activity_partnerships[0].from_distributions = Some(i64::MAX);
                    ("PARTNER_PREMIUMS_BELOW_K1", "health")
                }
                _ => {
                    value.activity_partnerships[0].from_distributions = Some(i64::MIN);
                    ("K1_ACTIVITY_SOURCE_AMOUNT_INVALID", "from_distributions")
                }
            };
            let path = alloc::format!(
                "[value/{}/0/{leaf}]",
                if scorp { "activity_scorporations" } else { "activity_partnerships" }
            );
            for request in [bad.clone(), activity_benefit_calculation(&bad)] {
                let request = P::from_avro(&request.to_avro()).unwrap();
                let Reply::Response(problem @ P::Problem { .. }) =
                    send(&mut session, request.clone())
                else {
                    panic!("expected intrinsic/source refusal")
                };
                let P::Problem { message, field } = &problem else { unreachable!() };
                assert_eq!(message, &libertas::libertas_formatted_text(key, &[]));
                assert_eq!(field.as_deref(), Some(path.as_str()));
                assert_eq!(session.draft, before);
                assert_eq!(session.draft.to_avro(), before_bytes);
                assert_eq!(session.dirty, dirty);
            }
            accepted(&send(&mut session, activity_benefit_calculation(&good)));
            assert_eq!(session.draft, before);
            assert_eq!(session.dirty, dirty);
            accepted(&send(&mut session, good));
            assert_eq!(session.draft, before);
            let mut restored = restart(before.clone());
            let corrected = open_activity_benefits(&mut restored);
            accepted(&send(&mut restored, corrected));
            assert_eq!(restored.draft, before);
            assert!(estimate(&restored.draft).tax.is_none());
        }
    }
}

#[test]
fn k1res_output_benefit_labels_are_ignored_and_regenerated_from_accepted_sources() {
    for scorp in [false, true] {
        for omitted in [false, true] {
            let mut session = accepted_session(scorp);
            let good = open_activity_benefits(&mut session);
            accepted(&send(&mut session, good));
            let before = session.draft.clone();
            let dirty = session.dirty;
            let mut request = open_activity_benefits(&mut session);
            let submitted = if omitted { Vec::new() } else { vec![255, 0, 1] };
            if scorp {
                activity_benefits(&mut request).activity_scorporations[0].name = submitted.clone();
            } else {
                activity_benefits(&mut request).activity_partnerships[0].name = submitted.clone();
            }
            accepted(&send(&mut session, activity_benefit_calculation(&request)));
            assert_eq!(session.draft, before);
            assert_eq!(session.dirty, dirty);
            accepted(&send(&mut session, request));
            assert_eq!(session.draft, before);
            let mut restored = restart(before.clone());
            let mut reopened = open_activity_benefits(&mut restored);
            let label = if scorp {
                &activity_benefits(&mut reopened).activity_scorporations[0].name
            } else {
                &activity_benefits(&mut reopened).activity_partnerships[0].name
            };
            assert!(!label.is_empty());
            assert_ne!(label, &submitted);
            accepted(&send(&mut restored, reopened));
            assert_eq!(restored.draft, before);
        }
    }
}

#[test]
fn k1res_fresh_client_benefit_writer_saves_reopens_and_restarts() {
    // The original bridge's four source worlds now use the normal native
    // lifecycle directly; no payload/dump endpoints participate in this proof.
    for scorp in [false, true] {
        for with_plan in [false, true] {
            let (mut session, wage_id) =
                if scorp { scorp_benefit_session() } else { (partner_benefit_session(), 0) };
            let mut request = open_activity_benefits(&mut session);
            if scorp {
                activity_benefits(&mut request).activity_scorporations[0].health =
                    Some(if with_plan {
                        activity_health_plan(true, wage_id)
                    } else {
                        SeHealth::None
                    });
            } else {
                let row = &mut activity_benefits(&mut request).activity_partnerships[0];
                row.from_distributions = Some(30_000);
                if with_plan {
                    row.health = Some(activity_health_plan(false, 0));
                }
            }
            let expected = activity_benefits(&mut request).clone();
            let before = session.draft.clone();
            accepted(&send(&mut session, activity_benefit_calculation(&request)));
            assert_eq!(session.draft, before);
            accepted(&send(&mut session, request));
            if !scorp {
                assert_eq!(
                    partner_source(&session).person,
                    Some(PartnerPersonSource {
                        health_distributions: Some(30_000),
                        ..Default::default()
                    })
                );
            }
            let mut reopened = open_activity_benefits(&mut session);
            assert_eq!(activity_benefits(&mut reopened), &expected);
            let source = session.draft.income.clone();
            let mut cold = restart(session.draft.clone());
            assert_eq!(cold.draft.income, source);
            let mut reopened = open_activity_benefits(&mut cold);
            assert_eq!(activity_benefits(&mut reopened), &expected);
            accepted(&send(&mut cold, reopened));
            assert_eq!(cold.draft.income, source);
            let mut source_editor = open_topic(&mut cold, TaxSection::PassThrough);
            if !scorp {
                let IncomeDocument::K1PartnershipActivities(data) =
                    &mut documents(&mut source_editor)[0].document
                else {
                    unreachable!()
                };
                assert_eq!(data.person.as_ref().unwrap().health_distributions, Some(30_000));
            }
            accepted(&send(&mut cold, source_editor));
            assert_eq!(cold.draft.income, source);
            assert!(estimate(&cold.draft).tax.is_none());
        }
    }
}

#[test]
fn partial_partner_fuel_recovery_modern_source_survives_both_native_writers_and_restart() {
    let mut session = partner_benefit_session();
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].from_distributions = Some(30_000);
    accepted(&send(&mut session, request));
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    let prior = partial_partner_fuel_recovery();
    page.person.as_mut().unwrap().prior_fuel_credit =
        Some(K1PriorFuelSource::Reported { source: prior.clone() });
    accepted(&send(&mut session, request));
    let mut request = open_activity_benefits(&mut session);
    activity_benefits(&mut request).activity_partnerships[0].from_distributions = Some(20_000);
    accepted(&send(&mut session, request));
    let before = session.draft.clone();
    let mut session = restart(before.clone());
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    assert_eq!(
        page.person.as_ref().unwrap().prior_fuel_credit,
        Some(K1PriorFuelSource::Reported { source: prior })
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, before);
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    let K1PriorFuelSource::Reported { source } =
        page.person.as_mut().unwrap().prior_fuel_credit.as_mut().unwrap()
    else {
        unreachable!()
    };
    source.partial_prior_year_benefit.as_mut().unwrap().worksheet_reference.clear();
    assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
    assert_eq!(session.draft, before);
    assert_eq!(restart(session.draft.clone()).draft, before);
}
