// Current typed source producers own these controls. Case names retain the
// original metadata regression identity, without retaining an older wire schema.
use super::income_pages::open_topic;
use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::*;

#[derive(Clone, Copy)]
pub(in crate::tests) enum SourceScope {
    Income,
    Health,
    CommitOnly,
    Fieldless,
    Secondary(SecondaryCorrection),
}
#[derive(Clone, Copy)]
pub(in crate::tests) enum SecondaryCorrection {
    Employer,
    SharedEmployer,
    AdjustmentOwner,
    AnnuitySource,
    InvestmentSource,
    InvestmentOrigin,
    RepaymentChoice,
    MedicalSource,
    SituationChoice,
    AlimonyRecapture,
}

pub(in crate::tests) fn source_calculation(request: &P) -> P {
    match request {
        P::SaveIncomes { .. } | P::SaveDocuments { .. } | P::SaveWagesAnswers { .. } => {
            super::estimate_documents_from_save(request)
        }
        P::SaveHsa {
            user,
            value,
            allowed_owners,
            other_coverage_allowed,
            allowed_family_split,
            hsa_failed_deposits,
            archer_failed_deposits,
            hsa_return_documents,
            archer_return_documents,
            ira_sources,
            archer_income,
            followups,
            ..
        } => P::EstimateHsa {
            user: *user,
            value: value.clone(),
            allowed_owners: allowed_owners.clone(),
            other_coverage_allowed: *other_coverage_allowed,
            allowed_family_split: allowed_family_split.clone(),
            hsa_failed_deposits: hsa_failed_deposits.clone(),
            archer_failed_deposits: archer_failed_deposits.clone(),
            hsa_return_documents: hsa_return_documents.clone(),
            archer_return_documents: archer_return_documents.clone(),
            ira_sources: ira_sources.clone(),
            archer_income: archer_income.clone(),
            followups: followups.clone(),
            questions: Vec::new(),
        },
        P::SaveSelfEmployedBenefits {
            user,
            value,
            ptc_policies,
            health_activities,
            health_people,
            health_wages,
            ..
        } => P::EstimateSelfEmployedBenefits {
            user: *user,
            value: value.clone(),
            ptc_policies: ptc_policies.clone(),
            health_activities: health_activities.clone(),
            health_people: health_people.clone(),
            health_wages: health_wages.clone(),
        },
        P::SaveMarketplace {
            user,
            value,
            people,
            ptc_people,
            ptc_families,
            ptc_policies,
            ptc_w2,
            ptc_employees,
            ptc_nonreturn_people,
            ptc_claimed_dependents,
            allowed_marriage,
            allowed_separate,
            ..
        } => P::EstimateMarketplace {
            user: *user,
            value: value.clone(),
            people: people.clone(),
            ptc_people: ptc_people.clone(),
            ptc_families: ptc_families.clone(),
            ptc_policies: ptc_policies.clone(),
            ptc_w2: ptc_w2.clone(),
            ptc_employees: ptc_employees.clone(),
            ptc_nonreturn_people: ptc_nonreturn_people.clone(),
            ptc_claimed_dependents: ptc_claimed_dependents.clone(),
            allowed_marriage: allowed_marriage.clone(),
            allowed_separate: allowed_separate.clone(),
        },
        P::SaveCare {
            user,
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
            allowed_filing: Vec::new(),
            user: *user,
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
        P::SaveDeductions { user, allowed_repayment, home_sites, value, .. } => {
            P::EstimateDeductions {
                user: *user,

                allowed_repayment: allowed_repayment.clone(),
                home_sites: home_sites.clone(),
                value: value.clone(),
            }
        }
        P::SaveInjuredSpouse { user, value, followups, .. } => P::EstimateInjuredSpouse {
            user: *user,
            value: value.clone(),
            followups: followups.clone(),
            questions: Vec::new(),
        },
        P::SaveEducation { user, value, people, followups, .. } => P::EstimateEducation {
            user: *user,
            value: value.clone(),
            people: people.clone(),
            followups: followups.clone(),
            questions: Vec::new(),
        },
        P::SaveEmployeeExpenses { user, allowed_owners, value, .. } => {
            P::EstimateEmployeeExpenses {
                rental_energy_buildings: Vec::new(),
                initial_asset_amt_carry_choices: Vec::new(),
                user: *user,
                allowed_owners: allowed_owners.clone(),
                value: value.clone(),
            }
        }
        P::SaveMedical { user, value, .. } => {
            P::EstimateMedical { user: *user, value: value.clone() }
        }
        P::SaveMoreAdjustments { user, allowed_owners, value, .. } => P::EstimateMoreAdjustments {
            user: *user,
            allowed_owners: allowed_owners.clone(),
            value: value.clone(),
        },
        P::SaveOtherItemized {
            user,
            allowed_owners,
            interest_sources,
            investment_k1_sources,
            investment_origin_choices,
            value,
            ..
        } => P::EstimateOtherItemized {
            user: *user,
            allowed_owners: allowed_owners.clone(),
            interest_sources: interest_sources.clone(),
            investment_k1_sources: investment_k1_sources.clone(),
            investment_origin_choices: investment_origin_choices.clone(),
            value: value.clone(),
        },
        P::SaveOftenMissed { user, value, .. } => {
            P::EstimateOftenMissed { user: *user, value: value.clone() }
        }
        _ => panic!("a current source editor has a calculation"),
    }
}

fn source_topic(request: &P) -> TaxSection {
    match request {
        P::SaveIncomes { topic, .. } => *topic,
        P::SaveWagesAnswers { .. } => TaxSection::Wages,
        P::SaveOftenMissed { .. } => TaxSection::OftenMissed,
        P::SaveDocuments { .. } => TaxSection::Documents,
        P::SaveHsa { .. } => TaxSection::Hsa,
        P::SaveSelfEmployedBenefits { .. } => TaxSection::SelfEmployedBenefits,
        P::SaveMarketplace { .. } => TaxSection::Marketplace,
        P::SaveCare { .. } => TaxSection::Care,
        P::SaveDeductions { .. } => TaxSection::Deductions,
        P::SaveEducation { .. } => TaxSection::Education,
        P::SaveInjuredSpouse { .. } => TaxSection::InjuredSpouse,
        P::SaveEmployeeExpenses { .. } => TaxSection::EmployeeExpenses,
        P::SaveMedical { .. } => TaxSection::Medical,
        P::SaveMoreAdjustments { .. } => TaxSection::MoreAdjustments,
        P::SaveOtherItemized { .. } => TaxSection::OtherItemized,
        _ => panic!("a current source editor has a topic"),
    }
}
fn reopen(session: &mut Interview, request: &P) -> P {
    if matches!(request, P::SaveDocuments { .. }) {
        return super::income_pages::open_document_center(session);
    }
    if let P::SaveIncomes { topic, .. } = request {
        return open_topic(session, *topic);
    }
    match from_review(session, source_topic(request)) {
        Reply::Edit(page) => page,
        reply => panic!("accepted source editor must reopen: {reply:?}"),
    }
}
fn assert_problem(case: &str, reply: &Reply, key: &str, path: Option<&str>) -> Vec<u8> {
    let Reply::Response(P::Problem { message, field }) = reply else {
        panic!("{case}: expected source refusal, got {reply:?}")
    };
    assert_eq!(crate::Message::decode(message).key, key, "{case}: diagnostic");
    assert_eq!(field.as_deref(), path, "{case}: submitted source path");
    message.clone()
}
fn correct(session: &Interview, request: &mut P, kind: SecondaryCorrection) {
    use SecondaryCorrection as C;
    match request {
        P::SaveEmployeeExpenses { value, .. } => {
            if matches!(kind, C::SharedEmployer) {
                value.forms.truncate(1);
            }
            for form in &mut value.forms {
                let source = session
                    .draft
                    .income
                    .iter()
                    .find(|entry| {
                        entry.owner == crate::DocumentOwner::from(form.owner)
                            && matches!(entry.income, FederalIncome::Wage { .. })
                    })
                    .expect("actual same-owner W-2");
                assert!(source.id > 0);
                form.employers = vec![source.id];
            }
        }
        P::SaveMoreAdjustments { value, .. } => {
            if matches!(kind, C::AlimonyRecapture) {
                let MoreAdjustment::AlimonyPaid(row) = &mut value.adjustments[0] else {
                    unreachable!()
                };
                let amount = row.payment.amount();
                let crate::AlimonyRecapture::ThirdYear { current_excluded, .. } =
                    &mut row.payment.recapture
                else {
                    unreachable!()
                };
                *current_excluded = amount;
            } else {
                for row in &mut value.adjustments {
                    match row {
                        MoreAdjustment::AlimonyPaid(row) => row.owner = Owner::Taxpayer,
                        MoreAdjustment::MovingExpenses(row) => row.owner = Owner::Taxpayer,
                    }
                }
            }
        }
        P::SaveOtherItemized { value, .. } => match kind {
            C::AnnuitySource => {
                let source = value.contracts[0].id;
                assert!(session.draft.income.iter().any(|entry| entry.id == source));
                value.annuity_losses[0].contract = source;
            }
            C::InvestmentSource => value.investment_interest.as_mut().unwrap().loans[0].source = 0,
            C::InvestmentOrigin => {
                let source = crate::investment_interest::k1_sources(&session.draft)[0].id;
                assert!(source > 0 && session.draft.income.iter().any(|entry| entry.id == source));
                value.investment_interest.as_mut().unwrap().loans[1].originating_k1 =
                    Some(crate::InvestmentK1Origin::Entity { source });
            }
            _ => unreachable!(),
        },
        P::SaveDocuments { documents, .. } => {
            let mut accepted = super::restart(session.draft.clone());
            let P::SaveDocuments { documents: previous, .. } =
                super::income_pages::open_document_center(&mut accepted)
            else {
                unreachable!()
            };
            documents.common.education = previous.common.education;
        }
        P::SaveDeductions { value, .. } => match kind {
            C::RepaymentChoice => value.repayment = crate::RepaymentClaim::None,
            _ => unreachable!(),
        },
        P::SaveMedical { value, .. } => value.long_term_care[0].person = value.people[0].id,
        P::SaveOftenMissed { value, .. } => value.unprepared = None,
        _ => unreachable!(),
    }
}

// The caller supplies its actual current typed candidate at the original refusal
// site. Extra lifecycle checks use an independent normal session, so the owning
// test continues with its own accepted source and independent financial assertions.
#[inline(never)]
pub(in crate::tests) fn current_source_refusal(
    case: &str,
    session: &mut Interview,
    request: P,
    key: &str,
    path: Option<&str>,
    scope: SourceScope,
) -> Reply {
    if let Some((mut candidate, topic_request, topic_path)) = path
        .and_then(|path| super::income_pages::topic_fixture_control(session, &request, key, path))
    {
        let before = candidate.draft.clone();
        let dirty = candidate.dirty;
        let reply = super::send(&mut candidate, topic_request.clone());
        let message = assert_problem(case, &reply, key, Some(&topic_path));
        assert_eq!(candidate.draft, before, "{case}: actual topic refusal is atomic");
        assert_eq!(candidate.dirty, dirty);
        let mut cold = super::restart(before.clone());
        let reply_cold = super::send(&mut cold, topic_request.clone());
        assert_eq!(assert_problem(case, &reply_cold, key, Some(&topic_path)), message);
        assert_eq!(cold.draft, before, "{case}: cold actual topic refusal is atomic");
        let running = super::estimate_documents_from_save(&topic_request);
        let calculated = super::send(&mut cold, running);
        if super::survey::unanswered_personal_refusal(&topic_request, key) {
            assert!(
                matches!(calculated, Reply::Response(P::IncomeRunningEstimate { .. })),
                "{case}: missing personal answers permit a provisional estimate: {calculated:?}"
            );
        } else {
            assert_eq!(assert_problem(case, &calculated, key, Some(&topic_path)), message);
        }
        assert_eq!(cold.draft, before, "{case}: actual topic estimate is transient");
        let mut original = super::restart(session.draft.clone());
        let P::SaveDocuments { documents: original_documents, .. } =
            super::income_pages::open_document_center(&mut original)
        else {
            unreachable!()
        };
        // A replacement source save can delete original IDs. Normal saves cannot
        // resurrect those IDs; the original caller remains untouched by this cold probe.
        let original_ids: Vec<_> = original.draft.income.iter().map(|entry| entry.id).collect();
        if original_ids.iter().all(|id| cold.draft.income.iter().any(|entry| entry.id == *id)) {
            let mut correction = super::income_pages::open_document_center(&mut cold);
            let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
            *documents = original_documents;
            accepted(&super::income_pages::save_source_fixture_request(&mut cold, correction));
            let mut expected = estimate(&session.draft);
            expected.revision = cold.draft.revision;
            assert_eq!(
                estimate(&cold.draft),
                expected,
                "{case}: source restoration preserves the complete result"
            );
        }
        if let Reply::Response(problem) = &reply {
            crate::tests::unit47_rule_cases::export_unit47_rule_case(case, &topic_request, problem);
        }
        return reply;
    }
    let before = session.draft.clone();
    if matches!(scope, SourceScope::Secondary(_) | SourceScope::Fieldless) {
        assert!(before.valid_stored(), "{case}: accepted Draft is valid");
    }
    let dirty = session.dirty;
    let reply = super::send(session, request.clone());
    let message = assert_problem(case, &reply, key, path);
    // Original802's living-owner statement has source ID0 and no display
    // label. Its diagnostic names that source with the independent Number0
    // argument; every other original refusal has no arguments.
    let expected_message = if case == "income-source-802" {
        crate::Message::new(key, vec![crate::MessageArg::Number(0)]).encode()
    } else {
        libertas::libertas_formatted_text(key, &[])
    };
    assert_eq!(message, expected_message, "{case}: complete source diagnostic");

    assert_eq!(session.draft, before, "{case}: full rejected Draft");
    assert_eq!(session.draft.to_avro(), before.to_avro(), "{case}: rejected Draft bytes");
    assert_eq!(session.dirty, dirty, "{case}: rejected dirty state");
    if path.is_some() {
        crate::tests::unit47_rule_cases::export_unit47_rule_case(
            case,
            &request,
            match &reply {
                Reply::Response(value) => value,
                _ => unreachable!(),
            },
        );
    }
    let mut lifecycle = super::restart(before.clone());
    assert_eq!(lifecycle.draft, before, "{case}: cold accepted source");
    if matches!(scope, SourceScope::Fieldless) {
        return reply;
    }
    let mut request = request;
    if matches!(scope, SourceScope::Income) {
        // Admit the unchanged accepted owner through its actual editor before
        // proving correction is a no-op. A first visit is a normal saved fact.
        let accepted_owner = reopen(&mut lifecycle, &request);
        accepted(&super::send(&mut lifecycle, accepted_owner));
        lifecycle = super::restart(lifecycle.draft.clone());
        match &mut request {
            P::SaveIncomes { cookie, revision, .. } | P::SaveDocuments { cookie, revision, .. } => {
                *cookie = lifecycle.draft.cookie.clone();
                *revision = lifecycle.draft.revision;
            }
            _ => unreachable!(),
        }
        let accepted_state = lifecycle.draft.clone();
        let refused = super::send(&mut lifecycle, request.clone());
        assert_eq!(assert_problem(case, &refused, key, path), message);
        assert_eq!(lifecycle.draft, accepted_state, "{case}: visited-owner refusal is atomic");
        assert!(!lifecycle.dirty);
    }
    let before = lifecycle.draft.clone();
    if path.is_some() {
        let calculated = crate::tests::support::client::income_pages::candidate_save_review(
            &mut lifecycle,
            source_calculation(&request),
        );
        if matches!(scope, SourceScope::CommitOnly)
            && !matches!(calculated, Reply::Response(P::Problem { .. }))
        {
            // This existing scope permits transient invalid drafts, but never a
            // usable estimate for them. The authoritative Save remains refused.
            let Reply::Response(P::IncomeRunningEstimate { estimate: Some(estimate), .. }) =
                &calculated
            else {
                panic!("{case}: expected unavailable transient estimate: {calculated:?}")
            };
            assert_eq!(crate::Message::decode(&estimate.line).key, "ESTIMATE_UNAVAILABLE");
            assert!(!estimate.remaining_items.is_empty());
        } else {
            assert_eq!(
                assert_problem(case, &calculated, key, path),
                message,
                "{case}: complete Save/Estimate diagnostic"
            );
        }
        assert_eq!(lifecycle.draft, before, "{case}: calculation is transient");
        assert!(!lifecycle.dirty, "{case}: calculation keeps clean restart");
    }
    let baseline = estimate(&before);
    let correction = if let SourceScope::Secondary(kind) = scope {
        let mut corrected = request.clone();
        correct(&lifecycle, &mut corrected, kind);
        corrected
    } else {
        reopen(&mut lifecycle, &request)
    };
    let corrected_estimate = crate::tests::support::client::income_pages::candidate_save_review(
        &mut lifecycle,
        source_calculation(&correction),
    );
    if matches!(scope, SourceScope::Income) {
        assert!(
            matches!(
                corrected_estimate,
                Reply::Response(P::IncomeRunningEstimate { .. } | P::Review { .. })
            ),
            "{case}: accepted correction returns the income estimate"
        );
    } else {
        accepted(&corrected_estimate);
    }
    assert_eq!(lifecycle.draft, before, "{case}: correction estimate is transient");
    assert!(!lifecycle.dirty);
    accepted(&super::send(&mut lifecycle, correction));
    let saved = lifecycle.draft.clone();
    let financial = estimate(&saved);
    match scope {
        SourceScope::Income => assert_eq!(saved, before, "{case}: accepted-source correction"),
        SourceScope::Health | SourceScope::CommitOnly => {
            assert_eq!(saved.income, before.income, "{case}: source facts");
            assert_eq!(saved.next_id, before.next_id, "{case}: source identities");
            assert_eq!(financial.return_lines, baseline.return_lines, "{case}: return lines");
            assert_eq!(
                (
                    financial.tax,
                    financial.refundable_credits,
                    financial.payments,
                    financial.refund,
                    financial.balance
                ),
                (
                    baseline.tax,
                    baseline.refundable_credits,
                    baseline.payments,
                    baseline.refund,
                    baseline.balance
                ),
                "{case}: money"
            );
            if baseline.state == ResultState::Unsupported {
                assert_eq!(financial.state, ResultState::Unsupported);
            }
        }
        SourceScope::Secondary(SecondaryCorrection::SituationChoice) => {
            // The corrected list is empty and the page's other answers stay.
            assert!(saved.often_missed.unprepared.is_empty());
            assert_eq!(saved.often_missed.benefits, before.often_missed.benefits);
            assert!(saved.valid_stored());
            supported(&saved);
        }
        SourceScope::Secondary(_) => {
            assert!(saved.valid_stored());
            supported(&saved);
        }
        SourceScope::Fieldless => unreachable!(),
    }
    let reopened = reopen(&mut lifecycle, &request);
    accepted(&super::send(&mut lifecycle, reopened));
    assert_eq!(lifecycle.draft, saved, "{case}: hot reopen/resave");
    let mut cold = super::restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let reopened = reopen(&mut cold, &request);
    accepted(&super::send(&mut cold, reopened));
    assert_eq!(cold.draft, saved, "{case}: cold reopen/resave");
    assert_eq!(estimate(&cold.draft), financial, "{case}: complete cold result");
    reply
}

pub(in crate::tests) fn current_income_refusal(
    case: &str,
    session: &mut Interview,
    request: P,
    key: &str,
    path: &str,
) -> Reply {
    current_source_refusal(case, session, request, key, Some(path), SourceScope::Income)
}

pub(in crate::tests) fn current_document_refusal(
    case: &str,
    session: &mut Interview,
    documents: Vec<IncomeDocumentItem>,
    key: &str,
    path: &str,
) -> Reply {
    let request = if documents.iter().all(|row| matches!(row.document, IncomeSource::Record { .. }))
    {
        let topic = super::income_pages::topic_for(&documents[0].document);
        super::income_pages::save_topic(session, topic, documents)
    } else {
        assert!(
            documents.iter().all(|row| !matches!(row.document, IncomeSource::Record { .. })),
            "accept prerequisite sources explicitly before testing a record refusal"
        );
        super::income_pages::save_documents(session, documents)
    };
    current_income_refusal(case, session, request, key, path)
}
pub(in crate::tests) fn current_income_key(
    case: &str,
    draft: &FederalDraft,
    documents: Vec<IncomeDocumentItem>,
    key: &str,
    path: &str,
) -> Vec<u8> {
    let mut session = Interview::new(TEST_USER, draft.clone());
    let Reply::Response(P::Problem { message, .. }) =
        current_document_refusal(case, &mut session, documents, key, path)
    else {
        unreachable!()
    };
    message
}

#[inline(never)]
pub(in crate::tests) fn current_source_acceptance(
    case: &str,
    session: &mut Interview,
    request: P,
) -> Reply {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let calculated = crate::tests::support::client::income_pages::candidate_save_review(
        session,
        source_calculation(&request),
    );
    if matches!(&request, P::SaveIncomes { .. } | P::SaveDocuments { .. }) {
        assert!(
            matches!(
                calculated,
                Reply::Response(P::IncomeRunningEstimate { .. } | P::Review { .. })
            ),
            "{case}: accepted source returns the income estimate"
        );
    } else {
        accepted(&calculated);
    }
    assert_eq!(session.draft, before, "{case}: positive estimate is transient");
    assert_eq!(session.dirty, dirty);
    let reply = if matches!(request, P::SaveDocuments { .. }) {
        super::income_pages::save_source_fixture_request(session, request.clone())
    } else {
        super::send(session, request.clone())
    };
    accepted(&reply);
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    let result = estimate(&saved);
    let reopened = reopen(session, &request);
    accepted(&super::send(session, reopened));
    assert_eq!(session.draft, saved, "{case}: hot reopen/resave");
    let mut cold = super::restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let reopened = reopen(&mut cold, &request);
    accepted(&super::send(&mut cold, reopened));
    assert_eq!(cold.draft, saved, "{case}: cold reopen/resave");
    assert_eq!(estimate(&cold.draft), result, "{case}: complete cold result");
    reply
}

pub(in crate::tests) fn railroad_answer_case(
    case: &str,
    session: &mut Interview,
    replace_id: Option<i64>,
    bad: crate::RailroadSource,
    corrected: crate::RailroadSource,
    key: &str,
) {
    use crate::tests::support::builders::native_income::document;
    use crate::tests::support::client::income_pages::{
        information_return_answers_for_topic, open_document_center, page_documents, save_documents,
        save_information_return_answers,
    };
    let prior: Vec<_> = session.draft.income.iter().map(|entry| entry.id).collect();
    let mut rows = page_documents(&open_document_center(session));
    let source = IncomeSource::Retirement {
        value: crate::RetirementDocument::Rrb1099R(railroad_document(corrected.clone())),
    };
    if let Some(id) = replace_id {
        let row = rows.iter_mut().find(|row| row.id == id).unwrap();
        row.document = source;
    } else {
        rows.push(document(0, source));
    }
    let request = save_documents(session, rows);
    accepted(&send(session, request));
    let id = session.draft.income.iter().find(|entry| replace_id.map_or(!prior.contains(&entry.id), |id| entry.id == id)
        && matches!(&entry.income, FederalIncome::Retirement { data } if matches!(crate::FederalRetirement::from(data).taxable, Some(crate::RetirementTaxable::Railroad { .. })))).unwrap().id;
    let desired = |form| {
        vec![document(
            id,
            IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(form) },
        )]
    };
    save_information_return_answers(session, &desired(railroad_document(corrected.clone())));
    let expected = estimate(&session.draft);
    let replace_paper = |session: &mut Interview, mut form: crate::FormRrb1099RPaper| {
        let mut request = open_document_center(session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let row =
            documents.common.retirement.documents.iter_mut().find(|row| row.id == id).unwrap();
        let crate::RetirementDocument::Rrb1099R(current) = &row.document else { unreachable!() };
        form.issuer = current.issuer.clone();
        form.recipient = current.recipient.clone();
        row.document = crate::RetirementDocument::Rrb1099R(form);
        accepted(&send(session, request));
        assert!(session.draft.valid_stored());
    };
    replace_paper(session, railroad_paper(bad.form.clone()));
    let request = information_return_answers_for_topic(
        session,
        &desired(railroad_document(bad.clone())),
        TaxSection::Retirement,
    )
    .unwrap();
    let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| row.key == alloc::format!("income/{id}/railroad_repayments"))
        .unwrap();
    let path = alloc::string::String::from(
        libertas::SchemaPath::root()
            .field("followups")
            .key(followups[index].key.as_str())
            .field("answer")
            .field("value"),
    );
    super::survey::survey_refusal(case, session, request.clone(), key, &path);
    let running = super::estimate_documents_from_save(&request);
    if bad.repayments.is_empty() {
        let before = session.draft.clone();
        assert!(matches!(
            crate::tests::support::client::income_pages::candidate_save_review(session, running),
            Reply::Response(P::IncomeRunningEstimate { .. })
        ));
        assert_eq!(session.draft, before);
    } else {
        super::survey::survey_refusal(
            &alloc::format!("{case}-estimate"),
            session,
            running,
            key,
            &path,
        );
    }
    replace_paper(session, railroad_paper(corrected.form.clone()));
    save_information_return_answers(session, &desired(railroad_document(corrected)));
    let mut expected = expected;
    expected.revision = session.draft.revision;
    assert_eq!(estimate(&session.draft), expected);
    let saved = session.draft.clone();
    for source in [true, false] {
        let page = if source {
            super::income_pages::open_documents(session)
        } else {
            open_topic(session, TaxSection::Retirement)
        };
        accepted(&send(session, page));
        assert_eq!(session.draft, saved, "{case}: accepted owner reopen/resave");
        let mut restored = restart(saved.clone());
        let page = if source {
            super::income_pages::open_documents(&mut restored)
        } else {
            open_topic(&mut restored, TaxSection::Retirement)
        };
        accepted(&send(&mut restored, page));
        assert_eq!(restored.draft, saved, "{case}: accepted owner restart/resave");
    }
}

pub(in crate::tests) fn information_return_refusal(
    case: &str,
    session: &mut Interview,
    bad: &[IncomeDocumentItem],
    good: Vec<IncomeDocumentItem>,
    key: &str,
    source_path: &str,
) {
    super::income_pages::save_each_topic(session, good);
    let request = super::income_pages::information_return_answers_for_topic(
        session,
        bad,
        TaxSection::OtherIncome,
    )
    .unwrap();
    let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
    let index = followups
        .iter()
        .position(|row| matches!(row.answer, crate::DocumentAnswer::LtcContract { .. }))
        .unwrap();
    let tail = source_path.split_once("/contract").unwrap().1;
    let mut path = libertas::SchemaPath::root()
        .field("followups")
        .key(followups[index].key.as_str())
        .field("answer")
        .field("value");
    for segment in tail.trim_end_matches(']').split('/').filter(|segment| !segment.is_empty()) {
        path = match segment.parse::<usize>() {
            Ok(index) => path.index(index),
            Err(_) => path.field(segment),
        };
    }
    let path: alloc::string::String = path.into();
    super::survey::survey_refusal(case, session, request.clone(), key, &path);
    super::survey::survey_refusal(
        &alloc::format!("{case}-estimate"),
        session,
        super::estimate_documents_from_save(&request),
        key,
        &path,
    );
}

// A corrected printed statement keeps personal answers; only their topic refuses the conflict.
pub(in crate::tests) fn retained_marketplace_conflict(draft: &FederalDraft) {
    assert!(draft.valid_stored());
    let facts = crate::marketplace::facts(&draft.credits.as_ref().unwrap().marketplace).unwrap();
    let issue = facts
        .forms
        .iter()
        .find_map(|policy| crate::marketplace::policy_answer_issue(facts, draft, policy))
        .expect("the original Marketplace owner identifies the conflict");
    let mut session = super::restart(draft.clone());
    let Reply::Response(P::Review { remaining_items, allowed_actions, .. }) =
        super::send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("accepted paper remains reviewable")
    };
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::Marketplace
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    assert!(!allowed_actions.contains(&(libertas_macros::variant_index!(P::StartFinish) as i32)));
    let Reply::Edit(request @ P::SaveMarketplace { .. }) =
        from_review(&mut session, TaxSection::Marketplace)
    else {
        panic!("actual Marketplace topic")
    };
    let before = session.draft.clone();
    crate::tests::support::assert::rejected_with(super::send(&mut session, request), issue.key);
    assert_eq!(session.draft, before);
}
