// information_returns items of src/tests/support/client/income_pages.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
use crate::tests::cross::interview::from_review;
use crate::tests::support::client::income_pages::awaits_activity;

// Physical source kinds supported by the shared fixture-answer adapter. This
// matcher never derives a personal answer or uses a submitted ID as authority.
pub(super) fn information_return_kind(source: &IncomeSource) -> Option<usize> {
    Some(match source {
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099Nec(_) } => 0,
        IncomeSource::SelfEmployment { value: crate::SelfEmploymentDocument::Form1099K(_) } => 1,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Form1099Misc(_) } => 2,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(_) } => 3,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(_) } => 4,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Sa1099(_) } => 5,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::HsaTrusteeStatement(_) } => {
            127
        }
        IncomeSource::Retirement { value: crate::RetirementDocument::R1099(_) } => 6,
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099R(_) } => 7,
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Ltc1099(_) } => 8,
        IncomeSource::Wage { value: crate::WageDocument::W2(_) } => 9,
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(_) } => 10,
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) } => 11,
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Div1099(_) } => 12,
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Sale1099B(_) } => 13,
        IncomeSource::GovernmentPayment { .. } => 14,
        IncomeSource::Farming(_) => 15,
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Cancel1099C(_) } => 16,
        IncomeSource::CanceledDebt { value: crate::CanceledDebtDocument::Acquisition1099A(_) } => {
            17
        }
        IncomeSource::Wage { value: crate::WageDocument::Form8919(_) } => 18,
        IncomeSource::Retirement { value: crate::RetirementDocument::Ssa1099(_) } => 19,
        IncomeSource::Retirement { value: crate::RetirementDocument::Rrb1099(_) } => 119,
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3922(_) } => 120,
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Form2439(_) } => {
            121
        }
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(_) } => 122,
        IncomeSource::Gambling(_) => 123,
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(_) } => 124,
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::SavingsBonds(_),
        } => 125,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Contracts1099B(_),
        } => 128,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::OpportunityFund(_),
        } => 129,
        IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(_) } => 130,
        _ => return None,
    })
}

pub(super) fn information_return_sources<'a>(
    session: &mut Interview,
    desired: &'a [IncomeDocumentItem],
) -> Vec<(i64, TaxSection, &'a IncomeDocumentItem)> {
    let accepted = page_documents(&open_document_center(session))
        .iter()
        .map(|row| (row.id, information_return_kind(&row.document)))
        .collect::<Vec<_>>();
    match_fixture_sources(&accepted, desired)
        .into_iter()
        .map(|(id, supplied)| {
            let topic = if matches!(
                supplied.document,
                IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form3921(_) }
            ) {
                assert!(session.draft.screening.iter().any(|screening| {
                    screening.iso_exercises.iter().any(|source| source.id == id)
                }));
                TaxSection::InvestmentSales
            } else {
                let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
                crate::interview::followup_topic(&session.draft, entry)
                    .expect("the accepted physical source has an answering topic")
            };
            (id, topic, supplied)
        })
        .collect()
}

fn match_fixture_sources<'a>(
    accepted: &[(i64, Option<usize>)],
    desired: &'a [IncomeDocumentItem],
) -> Vec<(i64, &'a IncomeDocumentItem)> {
    let mut used = Vec::new();
    let mut matched = Vec::new();
    for (index, supplied) in desired.iter().enumerate() {
        let Some(kind) = information_return_kind(&supplied.document) else {
            continue;
        };
        let candidates: Vec<_> = accepted
            .iter()
            .filter(|(id, actual_kind)| {
                !used.contains(id)
                    && *actual_kind == Some(kind)
                    && (supplied.id == *id
                        || (supplied.id == 0 && !desired.iter().any(|given| given.id == *id)))
            })
            .collect();
        let offset = if supplied.id == 0 {
            // Existing rows retain their IDs. New source fixtures correspond
            // to the newly appended accepted rows, in the submitted order.
            let remaining = desired[index..]
                .iter()
                .filter(|row| row.id == 0 && information_return_kind(&row.document) == Some(kind))
                .count();
            candidates
                .len()
                .checked_sub(remaining)
                .expect("every supplied new fixture has an accepted physical source")
        } else {
            0
        };
        let (id, _) =
            candidates.get(offset).expect("the supplied fixture has an accepted physical source");
        used.push(*id);
        matched.push((*id, supplied));
    }
    matched
}

pub(super) fn fixture_source_has_answers(document: &IncomeSource) -> bool {
    fixture_paper_has_answers(document)
        || SA_SOURCES.with(|given| {
            let IncomeSource::OtherIncome { value } = document else { return false };
            given.borrow().iter().any(|row| row.papers.contains(value))
        })
}

// A new fixture registration supplies new personal facts even when the paper
// and its accepted ID are unchanged. Echoing an already applied registration
// leaves the topic editor's latest accepted answers intact.
pub(super) fn pending_fixture_answer_rows(
    session: &Interview,
    desired: &[IncomeDocumentItem],
) -> Vec<IncomeDocumentItem> {
    desired
        .iter()
        .filter(|row| {
            if row.id == 0
                || matches!(
                    row.document,
                    IncomeSource::K1 {
                        value: crate::K1Document::K1PartnershipActivities(_)
                            | crate::K1Document::K1ScorpActivities(_)
                    }
                )
            {
                // Activity pages carry their supplied topic facts inline,
                // including edits to an already accepted source.
                return true;
            }
            match &row.document {
                IncomeSource::K1 { value: crate::K1Document::K1Partnership(document) }
                    if document.personal.is_some() =>
                {
                    return true;
                }
                IncomeSource::K1 { value: crate::K1Document::K1Scorp(document) } => {
                    return scorp_fixture_answers_pending(document);
                }
                _ => {}
            }
            let new_equal = desired.iter().any(|new| {
                new.id == 0
                    && (new.document == row.document
                        || matches!(
                            (&new.document, &row.document),
                            (IncomeSource::OtherIncome { value: before }, IncomeSource::OtherIncome { value: after })
                                if sa_papers_match(before, after)
                        ))
            });
            fixture_paper_answers_pending(&row.document, &session.draft.cookie, row.id, new_equal)
                || SA_SOURCES.with(|given| {
                    let given = given.borrow();
                    let IncomeSource::OtherIncome { value } = &row.document else { return false };
                    sa_fixture_index(&given, value, 0, 1, row.id).is_some_and(|index| {
                        (!new_equal || given[index].supplied_id == Some(row.id))
                            && !given[index].applied.iter().any(|(cookie, _)| {
                                cookie == &session.draft.cookie
                            })
                    })
                })
        })
        .cloned()
        .collect()
}

pub(super) fn mark_fixture_answers_applied(session: &Interview, desired: &[IncomeDocumentItem]) {
    if desired.is_empty() {
        return;
    }
    for row in desired {
        if let IncomeSource::K1 { value: crate::K1Document::K1Scorp(document) } = &row.document {
            mark_scorp_fixture_answers_applied(document);
        }
    }
    let mut accepted = session
        .draft
        .income
        .iter()
        .filter_map(|entry| {
            let row = crate::interview::document_item(
                entry,
                &[],
                crate::federal_rules::separate_return(&session.draft),
            )?;
            Some((row.id, information_return_kind(&row.document)))
        })
        .collect::<Vec<_>>();
    accepted.extend(
        session.draft.screening.iter().flat_map(|screening| {
            screening.iso_exercises.iter().map(|source| (source.id, Some(122)))
        }),
    );
    let matched = match_fixture_sources(&accepted, desired);
    for (id, supplied) in &matched {
        let equal: Vec<_> = matched
            .iter()
            .filter(|(_, candidate)| {
                candidate.document == supplied.document && (supplied.id != 0 || candidate.id == 0)
            })
            .collect();
        let ordinal = equal.iter().position(|(source, _)| source == id).unwrap();
        mark_fixture_paper_answers(
            &supplied.document,
            ordinal,
            equal.len(),
            &session.draft.cookie,
            *id,
            supplied.id,
        );
        if let IncomeSource::OtherIncome { value } = &supplied.document {
            let equal: Vec<_> = matched
                .iter()
                .filter(|(_, candidate)| {
                    (supplied.id != 0 || candidate.id == 0)
                        && matches!(&candidate.document, IncomeSource::OtherIncome { value: other }
                        if sa_papers_match(value, other))
                })
                .collect();
            let ordinal = equal.iter().position(|(source, _)| source == id).unwrap();
            SA_SOURCES.with(|given| {
                let mut given = given.borrow_mut();
                if let Some(index) =
                    sa_fixture_index(&given, value, ordinal, equal.len(), supplied.id)
                    && !given[index]
                        .applied
                        .iter()
                        .any(|(cookie, source)| cookie == &session.draft.cookie && source == id)
                {
                    given[index].applied.push((session.draft.cookie.clone(), *id));
                }
            });
        }
    }
}

// Printed sources enter first. Fixture classifications and allocations then
// travel through the same keyed topic editor as a filer's answers.
pub(in crate::tests) fn save_information_return_answers(
    session: &mut Interview,
    desired: &[IncomeDocumentItem],
) {
    let matched = information_return_sources(session, desired);
    let mut topics = Vec::new();
    for (_, topic, _) in &matched {
        if !topics.contains(topic) {
            topics.push(*topic);
        }
    }
    if crate::interview::injured_chosen(&session.draft)
        && matched.iter().any(|(_, _, row)| row.refund_management.is_some())
    {
        topics.push(TaxSection::InjuredSpouse);
    }
    // Preserve the established three-topic fixture save order, then append the
    // newly supported topics without opening unrelated editors.
    topics.sort_by_key(|topic| match topic {
        TaxSection::SelfEmployment => 0,
        TaxSection::OtherIncome => 1,
        TaxSection::Retirement => 2,
        TaxSection::Wages => 3,
        TaxSection::InterestDividends => 4,
        TaxSection::InvestmentSales => 5,
        TaxSection::GovernmentBenefits => 6,
        TaxSection::Farming => 7,
        _ => 8,
    });
    for topic in topics {
        if let Some(request) = information_return_answers_for_topic(session, desired, topic) {
            // Correction details arrive on the account page, even when other
            // statements already have ordinary payment follow-ups. Do not
            // submit that page before the fixture supplies its actual correction.
            if matches!(&request, P::SaveHsa { .. })
                && session.draft.adjustments.as_ref().is_some_and(|adjustments| {
                    crate::hsa_correction::source_rule(&adjustments.hsa, &session.draft.income)
                        == Some("HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED")
                        || crate::hsa_correction::archer_source_rule(
                            &adjustments.archer,
                            &session.draft.income,
                        ) == Some("HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED")
                })
            {
                continue;
            }
            // A paper whose receiving activity is saved afterwards on this same
            // topic waits for it: the page refuses until the activity exists.
            let reply = send_fixture_topic_answers(session, desired, topic, request);
            if !awaits_activity(&reply) {
                crate::tests::cross::interview::accepted(&reply);
            }
        }
        if cooperative_child_answers(&matched, topic) {
            let request = information_return_answers_for_topic(session, desired, topic).unwrap();
            let reply = send_fixture_topic_answers(session, desired, topic, request);
            if !awaits_activity(&reply) {
                crate::tests::cross::interview::accepted(&reply);
            }
        }
    }
    if let Some(reply) = finish_qsbs_fixture_answers(session, desired) {
        crate::tests::cross::interview::accepted(&reply);
    }
}

// QSBS asks only the facts its preceding answers make necessary. Advance the
// real topic editor until all explicitly supplied fixture facts have travelled.
pub(super) fn finish_qsbs_fixture_answers(
    session: &mut Interview,
    desired: &[IncomeDocumentItem],
) -> Option<Reply> {
    let mut result = None;
    let matched = information_return_sources(session, desired);
    let ids: Vec<_> = matched
        .iter()
        .filter_map(|(id, _, supplied)| {
            matches!(
                supplied.document,
                IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::QsbsSale(_) }
            )
            .then_some(*id)
        })
        .collect();
    loop {
        let pending = session.draft.income.iter().any(|entry| ids.contains(&entry.id)
            && matches!(&entry.income, FederalIncome::QualifiedSmallBusinessSale { data }
                if crate::qsbs_followups::questions(data).iter().any(|(_, _, answer)| crate::qsbs_followups::missing(answer))));
        if !pending {
            return result;
        }
        let before = session.draft.revision;
        let request =
            information_return_answers_for_topic(session, desired, TaxSection::InvestmentSales)
                .unwrap();
        let reply = send(session, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return Some(reply);
        }
        result = Some(reply);
        assert_ne!(
            session.draft.revision, before,
            "each QSBS fixture pass must answer a newly needed fact"
        );
    }
}

pub(super) fn cooperative_child_answers(
    matched: &[(i64, TaxSection, &IncomeDocumentItem)],
    topic: TaxSection,
) -> bool {
    matched.iter().any(|(_, owner, row)| {
        *owner == topic
            && match &row.document {
                IncomeSource::Farming(paper) => fixture_cooperative_answers(paper)
                    .is_some_and(|answers| !answers.credit_facts.other.is_empty()),
                _ => false,
            }
    })
}

pub(in crate::tests) fn information_return_answers(
    session: &mut Interview,
    desired: &[IncomeDocumentItem],
) -> P {
    let matched = information_return_sources(session, desired);
    let topic =
        matched.first().expect("the supplied fixtures contain a supported physical source").1;
    assert!(
        matched.iter().all(|(_, owner, _)| *owner == topic),
        "mixed actual topics must use information_return_answers_for_topic"
    );
    information_return_answer_request(session, &matched, topic)
}

pub(in crate::tests) fn information_return_answers_for_topic(
    session: &mut Interview,
    desired: &[IncomeDocumentItem],
    topic: TaxSection,
) -> Option<P> {
    let matched = information_return_sources(session, desired);
    if topic != TaxSection::InjuredSpouse && !matched.iter().any(|(_, actual, _)| *actual == topic)
    {
        return None;
    }
    Some(information_return_answer_request(session, &matched, topic))
}

pub(super) fn information_return_answer_request(
    session: &mut Interview,
    matched: &[(i64, TaxSection, &IncomeDocumentItem)],
    topic: TaxSection,
) -> P {
    let mut request =
        if matches!(topic, TaxSection::Education | TaxSection::Hsa | TaxSection::InjuredSpouse) {
            let reply = from_review(session, topic);
            let reply = if topic == TaxSection::InjuredSpouse {
                crate::tests::support::client::native_editor(reply, &[])
            } else {
                reply
            };
            let Reply::Edit(request) = reply else {
                panic!("expected the typed document-answering page")
            };
            request
        } else {
            open_topic(session, topic)
        };
    let accepted = match &request {
        P::SaveIncomes { followups, .. } => followups.clone(),
        _ => Vec::new(),
    };
    fill_information_return_answers(session, &mut request, matched);
    if let P::SaveIncomes { followups, .. } = &request {
        for row in followups {
            if let crate::DocumentAnswer::PaymentParts(parts) = &row.answer
                && accepted.iter().any(|old| old.key == row.key && old.answer != row.answer)
            {
                return P::SavePaymentParts {
                    user: TEST_USER,
                    cookie: session.draft.cookie.clone(),
                    revision: session.draft.revision,
                    key: row.key.clone(),
                    value: parts.value.clone(),
                };
            }
        }
    }
    request
}

// Source composers return before the containing topic is submitted. Rebuild
// from the accepted revision after each source-only reply, just as link return
// updates the editor's source view and revision before its eventual submission.
pub(super) fn send_fixture_topic_answers(
    session: &mut Interview,
    desired: &[IncomeDocumentItem],
    topic: TaxSection,
    request: P,
) -> Reply {
    match accept_fixture_payment_parts(session, desired, topic, request) {
        Ok(request) => send(session, request),
        Err(reply) => *reply,
    }
}

// A fixture may defer its topic answers until related personal records exist.
// Payment parts still need accepted identities before those activity records.
pub(super) fn accept_fixture_payment_parts(
    session: &mut Interview,
    desired: &[IncomeDocumentItem],
    topic: TaxSection,
    mut request: P,
) -> Result<P, alloc::boxed::Box<Reply>> {
    let mut saved_sources = Vec::new();
    loop {
        let P::SavePaymentParts { key, .. } = &request else { return Ok(request) };
        assert!(!saved_sources.contains(key), "a source return must update its fixture view");
        saved_sources.push(key.clone());
        let reply = send(session, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return Err(alloc::boxed::Box::new(reply));
        }
        assert!(matches!(reply, Reply::Response(P::PaymentPartsSaved { .. })));
        request = information_return_answers_for_topic(session, desired, topic).unwrap();
    }
}

fn fixture_answers_for_accepted_source(
    session: &Interview,
    matched: &[(i64, TaxSection, &IncomeDocumentItem)],
    id: i64,
    supplied: &IncomeDocumentItem,
) -> Vec<crate::DocumentAnswer> {
    let equal: Vec<_> = matched
        .iter()
        .filter(|(_, _, candidate)| {
            candidate.document == supplied.document && (supplied.id != 0 || candidate.id == 0)
        })
        .collect();
    let ordinal = equal.iter().position(|(source, _, _)| *source == id).unwrap();
    if supplied.id != 0
        && equal.iter().any(|(_, _, row)| row.id == 0)
        && !fixture_paper_answers_pending(
            &supplied.document,
            &session.draft.cookie,
            supplied.id,
            true,
        )
    {
        Vec::new()
    } else if equal.len() == 1 && supplied.id == 0 {
        fixture_paper_answers(&supplied.document)
    } else {
        fixture_paper_answers_for_source(&supplied.document, ordinal, equal.len(), supplied.id)
    }
}

pub(super) fn fill_information_return_answers(
    session: &mut Interview,
    request: &mut P,
    matched: &[(i64, TaxSection, &IncomeDocumentItem)],
) {
    // A fixture's ISO and NUA sales are ordinary: no fees or special
    // treatment, the W-2 includes any ISO compensation, own NUA shares. A
    // test that needs other answers gives them (answer_iso_sale, answer_nua_sale).
    if let P::SaveIncomes { followups, .. } = request {
        use crate::DocumentAnswer as A;
        for row in followups.iter_mut() {
            match &mut row.answer {
                A::IsoAdjustments { value: value @ None } => {
                    *value = Some(crate::IsoShareSaleMore {
                        selling_fees: 0,
                        acquisition_costs: 0,
                        option_cost: 0,
                        special: false,
                    })
                }
                A::IsoWageInclusion { value: value @ None } => *value = Some(true),
                A::NuaAdjustments { value: value @ None } => {
                    *value = Some(crate::NuaShareSaleMore { wash_sale: 0, special: false })
                }
                A::NuaOwnership { value: value @ None } => *value = Some(crate::NuaOwnership::Own),
                _ => {}
            }
        }
    }
    if let P::SaveWagesAnswers { wages, followups, .. } = request {
        let mut answers: Vec<_> = wages
            .iter()
            .map(|row| {
                (row.key.clone(), crate::DocumentAnswer::Wage { answers: row.answers.clone() })
            })
            .collect();
        fill_information_return_answer_values(
            session,
            matched,
            answers.iter_mut().map(|(key, answer)| (key.as_str(), answer)),
        );
        for row in wages {
            let (_, crate::DocumentAnswer::Wage { answers }) =
                answers.iter().find(|(key, _)| key == &row.key).unwrap()
            else {
                unreachable!()
            };
            row.answers = answers.clone();
        }
        for row in followups {
            let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
            if let Some((_, _, supplied)) = matched.iter().find(|(source, _, _)| *source == id) {
                let crate::WagesDocumentAnswer::Community { value } = &mut row.answer;
                *value = supplied.community;
            }
        }
        return;
    }
    if let P::SaveEducation { followups, .. } = request {
        use crate::document_center::EducationDocumentAnswer;
        for row in followups {
            let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
            let Some((_, _, supplied)) = matched.iter().find(|(source, _, _)| *source == id) else {
                continue;
            };
            if let EducationDocumentAnswer::Community { value } = &mut row.answer {
                *value = supplied.community;
            }
            if let Some(answer) =
                fixture_answers_for_accepted_source(session, matched, id, supplied)
                    .into_iter()
                    .filter_map(EducationDocumentAnswer::from_canonical)
                    .find(|answer| {
                        core::mem::discriminant(answer) == core::mem::discriminant(&row.answer)
                    })
            {
                row.answer = answer;
            }
        }
        return;
    }
    if let P::SaveHsa { followups, .. } = request {
        use crate::document_center::HsaDocumentAnswer as H;
        for row in followups {
            let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
            let Some((_, _, supplied)) = matched.iter().find(|(source, _, _)| *source == id) else {
                continue;
            };
            if let H::Community { value } = &mut row.answer {
                *value = supplied.community;
            }
            let Some(given) = fixture_medical_answers(session, matched, id, supplied) else {
                continue;
            };
            match &mut row.answer {
                H::MedicalPayment { value } => *value = given.payment,
                H::MedicareSpousal { value } => *value = given.medicare_from_spouse,
                H::MedicarePrior { value } => *value = given.medicare_prior,
                H::Community { .. } => {}
            }
        }
        return;
    }
    if let P::SaveInjuredSpouse { followups, .. } = request {
        for row in followups {
            let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
            if let Some((_, _, supplied)) = matched.iter().find(|(source, _, _)| *source == id) {
                let crate::InjuredSpouseDocumentAnswer::Management { value } = &mut row.answer;
                *value = supplied.refund_management;
            }
        }
        return;
    }
    let P::SaveIncomes { followups, .. } = request else { unreachable!() };
    fill_information_return_answer_values(
        session,
        matched,
        followups.iter_mut().map(|row| (row.key.as_str(), &mut row.answer)),
    );
}

fn fill_information_return_answer_values<'a>(
    session: &mut Interview,
    matched: &[(i64, TaxSection, &IncomeDocumentItem)],
    answers: impl Iterator<Item = (&'a str, &'a mut crate::DocumentAnswer)>,
) {
    for (key, answer) in answers {
        let id = key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
        let Some((_, _, supplied)) = matched.iter().find(|(source, _, _)| *source == id) else {
            continue;
        };
        use crate::DocumentAnswer as A;
        match answer {
            A::Community { value } => *value = supplied.community,
            A::Management { value } => *value = supplied.refund_management,
            _ => {}
        }
        let given = fixture_answers_for_accepted_source(session, matched, id, supplied);
        let ordinal = if matches!(answer, A::QsbsBoolean { .. }) {
            match key.split('/').nth(2).unwrap() {
                "qsbs_special" => 0,
                "qsbs_issuer_qualified" => 1,
                "qsbs_active_business" => 2,
                "qsbs_no_disqualification" => 3,
                "qsbs_complete_history" => 4,
                _ => unreachable!(),
            }
        } else if matches!(answer, A::QsbsZoneGain { .. }) {
            let index: usize = key.rsplit('/').next().unwrap().parse().unwrap();
            let FederalIncome::QualifiedSmallBusinessSale { data } =
                &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
            else {
                unreachable!()
            };
            data.lots[..index]
                .iter()
                .filter(|lot| {
                    (crate::year_constants::SECTION_1202_ZONE_FIRST_DATE
                        ..=crate::year_constants::SECTION_1202_FIFTY_PERCENT_LAST_DATE)
                        .contains(&lot.issued)
                })
                .count()
        } else {
            0
        };
        if let Some(mut given) = given
            .into_iter()
            .filter(|given| core::mem::discriminant(given) == core::mem::discriminant(&*answer))
            .nth(ordinal)
        {
            if let (
                A::PaymentParts(crate::PaymentPartsAnswer { value: desired }),
                A::PaymentParts(crate::PaymentPartsAnswer { value: accepted }),
            ) = (&mut given, &*answer)
                && desired.len() == accepted.len()
            {
                for (desired, accepted) in desired.iter_mut().zip(accepted) {
                    if desired.id == 0 {
                        desired.id = accepted.id;
                    }
                }
            }
            *answer = given;
            continue;
        }
        match (&supplied.document, answer) {
            (
                IncomeSource::GovernmentPayment { .. }
                | IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Form2439(_),
                },
                answer,
            ) => SOURCE_ANSWERS.with(|given| {
                if let Some(value) = given.borrow().iter().find(|value| {
                    core::mem::discriminant(*value) == core::mem::discriminant(answer)
                }) {
                    *answer = value.clone();
                }
            }),
            (IncomeSource::Farming(paper), answer) => {
                let Some(desired) = fixture_cooperative_answers(paper) else {
                    continue;
                };
                let FederalIncome::Cooperative { data: accepted } =
                    &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
                else {
                    unreachable!()
                };
                match answer {
                    A::CooperativeCreditRecords {
                        investment_shares,
                        other,
                        participation,
                        activity,
                        notice,
                    } => {
                        *investment_shares = desired.credit_facts.investment_shares.clone();
                        *other =
                            desired.credit_facts.other.iter().cloned().map(Into::into).collect();
                        for (wanted, accepted) in
                            other.iter_mut().zip(&accepted.more.credit_facts.other)
                        {
                            if wanted.id == 0 {
                                wanted.id = accepted.id;
                            }
                        }
                        *participation = desired.credit_facts.participation;
                        *activity = desired.credit_facts.activity;
                        *notice = desired.credit_facts.notice.clone();
                    }
                    A::CooperativeOtherRecords { activity, income, .. } => {
                        let child: i64 = key.rsplit('/').next().unwrap().parse().unwrap();
                        let wanted = desired
                            .credit_facts
                            .other
                            .iter()
                            .find(|item| item.id == child)
                            .or_else(|| {
                                let index = accepted
                                    .more
                                    .credit_facts
                                    .other
                                    .iter()
                                    .position(|item| item.id == child)?;
                                desired.credit_facts.other.get(index).filter(|item| item.id == 0)
                            })
                            .expect("known cooperative fixture child uses its accepted identity");
                        *activity = wanted.activity;
                        *income = wanted.income.clone();
                    }
                    A::CooperativeAllocations { value } => {
                        let mut allocations = desired.allocations;
                        for (wanted, accepted) in allocations.iter_mut().zip(value.iter()) {
                            if wanted.id == 0 {
                                wanted.id = accepted.id;
                            }
                        }
                        *value = allocations.into_iter().map(Into::into).collect();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn fixture_medical_answers(
    session: &Interview,
    matched: &[(i64, TaxSection, &IncomeDocumentItem)],
    id: i64,
    supplied: &IncomeDocumentItem,
) -> Option<crate::SaSource> {
    let IncomeSource::OtherIncome { value: source } = &supplied.document else { return None };
    let new_equal = matched.iter().any(|(_, _, candidate)| {
        candidate.id == 0
            && matches!(&candidate.document, IncomeSource::OtherIncome { value: form }
                            if sa_papers_match(source, form))
    });
    if supplied.id != 0
        && new_equal
        && !SA_SOURCES.with(|given| {
            let given = given.borrow();
            sa_fixture_index(&given, source, 0, 1, supplied.id).is_some_and(|index| {
                given[index].supplied_id == Some(supplied.id)
                    && !given[index]
                        .applied
                        .iter()
                        .any(|(cookie, _)| cookie == &session.draft.cookie)
            })
        })
    {
        return None;
    }
    // New papers consume unbound histories in input order;
    // explicit corrections use their bound source identity.
    let equal: Vec<_> = matched
        .iter()
        .filter(|(_, _, candidate)| {
            (supplied.id != 0 || candidate.id == 0)
                && matches!(
                    &candidate.document,
                    IncomeSource::OtherIncome { value: form }
                        if sa_papers_match(source, form)
                )
        })
        .collect();
    let ordinal = equal.iter().position(|(source, _, _)| *source == id).unwrap();
    sa_answers(source, ordinal, equal.len(), supplied.id)
}

extern crate std;
std::thread_local! {
    static SOURCE_ANSWERS: core::cell::RefCell<Vec<crate::DocumentAnswer>> =
        const { core::cell::RefCell::new(Vec::new()) };
}
pub(in crate::tests) struct SourceAnswers;
impl Drop for SourceAnswers {
    fn drop(&mut self) {
        SOURCE_ANSWERS.with(|given| given.borrow_mut().clear());
    }
}
// A Form 1099-G or Form 2439 carries no topic answers. While the guard lives, the fixture
// harness gives these answers to every such row of the same kind, as a filer answers the topic
// page after saving the paper.
pub(in crate::tests) fn source_answers(answers: Vec<crate::DocumentAnswer>) -> SourceAnswers {
    SOURCE_ANSWERS.with(|given| *given.borrow_mut() = answers);
    SourceAnswers
}
pub(in crate::tests) fn unemployment_answers(answers: Vec<crate::DocumentAnswer>) -> SourceAnswers {
    source_answers(answers)
}

std::thread_local! {
    static SA_SOURCES: core::cell::RefCell<Vec<SaFixture>> =
        const { core::cell::RefCell::new(Vec::new()) };
}

// Medical-account documents carry no topic answers. The fixture harness gives
// the answers of a registered source to the row of its paper, as a filer answers
// the topic page after saving the paper. Each test runs on its own thread.
struct SaFixture {
    source: crate::SaSource,
    papers: Vec<crate::OtherIncomeDocument>,
    applied: Vec<(String, i64)>,
    supplied_id: Option<i64>,
}

pub(in crate::tests) fn sa_document(source: crate::SaSource) -> crate::Form1099SaPaper {
    let form = source.form1099sa().expect("a Form 1099-SA fixture uses actual form paper").clone();
    SA_SOURCES.with(|given| {
        given.borrow_mut().push(SaFixture {
            source,
            papers: vec![crate::OtherIncomeDocument::Sa1099(form.clone())],
            applied: Vec::new(),
            supplied_id: None,
        })
    });
    form
}

pub(in crate::tests) fn sa_statement_document(
    source: crate::SaSource,
) -> crate::HsaTrusteeStatementPaper {
    let form = source
        .trustee_statement_paper()
        .expect("a trustee-statement fixture uses actual statement paper")
        .clone();
    SA_SOURCES.with(|given| {
        given.borrow_mut().push(SaFixture {
            source,
            papers: vec![crate::OtherIncomeDocument::HsaTrusteeStatement(form.clone())],
            applied: Vec::new(),
            supplied_id: None,
        })
    });
    form
}

fn sa_papers_match(
    before: &crate::OtherIncomeDocument,
    after: &crate::OtherIncomeDocument,
) -> bool {
    before == after
        || SA_SOURCES.with(|given| {
            given
                .borrow()
                .iter()
                .any(|row| row.papers.contains(before) && row.papers.contains(after))
        })
}

fn sa_answers(
    document: &crate::OtherIncomeDocument,
    ordinal: usize,
    total: usize,
    supplied_id: i64,
) -> Option<crate::SaSource> {
    SA_SOURCES.with(|given| {
        let given = given.borrow();
        sa_fixture_index(&given, document, ordinal, total, supplied_id)
            .map(|index| given[index].source.clone())
    })
}

fn sa_fixture_index(
    rows: &[SaFixture],
    document: &crate::OtherIncomeDocument,
    ordinal: usize,
    total: usize,
    supplied_id: i64,
) -> Option<usize> {
    if supplied_id != 0
        && let Some(index) = rows
            .iter()
            .rposition(|row| row.papers.contains(document) && row.supplied_id == Some(supplied_id))
    {
        return Some(index);
    }
    let matching: Vec<_> = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            (row.papers.contains(document) && row.supplied_id.is_none()).then_some(index)
        })
        .collect();
    if matching.len() == 1 {
        return Some(matching[0]);
    }
    let start = matching.len().checked_sub(total)?;
    matching.get(start + ordinal).copied()
}

pub(in crate::tests) fn bind_fixture_sa_answers(paper: &IncomeSource, id: i64) {
    let IncomeSource::OtherIncome { value } = paper else { return };
    SA_SOURCES.with(|given| {
        if let Some(row) = given.borrow_mut().iter_mut().rev().find(|row| {
            row.papers.contains(value) && row.supplied_id.is_none() && row.applied.is_empty()
        }) {
            row.supplied_id = Some(id);
        }
    });
}

pub(in crate::tests) fn rebind_sa_answers(
    before: &crate::OtherIncomeDocument,
    after: &crate::OtherIncomeDocument,
    supplied_id: i64,
    ordinal: usize,
    total: usize,
) {
    SA_SOURCES.with(|rows| {
        let mut rows = rows.borrow_mut();
        if before != after
            && let Some(index) = sa_fixture_index(&rows, before, ordinal, total, supplied_id)
            && !rows[index].papers.contains(after)
        {
            rows[index].papers.push(after.clone());
        }
    });
}
