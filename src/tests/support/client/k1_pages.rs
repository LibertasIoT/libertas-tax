// k1 items of src/tests/support/client/income_pages.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// Exercise each current ownership boundary through ordinary Send. Source input
// is accepted first; topic records are then entered with those accepted identities.
// Fixture answers travel through the actual topic editor after source admission.
// Preflight both real saves so an expected topic refusal leaves the caller intact.
pub(in crate::tests) fn save_k1_sources_and_answers(
    session: &mut Interview,
    rows: Vec<IncomeDocumentItem>,
) -> Reply {
    fn attempt(session: &mut Interview, rows: &[IncomeDocumentItem]) -> Reply {
        let request = save_documents(session, rows.to_vec());
        let reply = send(session, request);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
        if let Some(reply) = save_k1_fixture_answers(session, rows) { reply } else { reply }
    }
    let mut candidate = restart(session.draft.clone());
    let reply = attempt(&mut candidate, &rows);
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return reply;
    }
    let reply = attempt(session, &rows);
    if !matches!(reply, Reply::Response(P::Problem { .. })) {
        mark_fixture_answers_applied(session, &rows);
    }
    reply
}

// New personal groups receive their real accepted identities before associations
// can point at them. Both passes use the actual topic Save and its catalogs.
pub(super) fn save_k1_fixture_answers(
    session: &mut Interview,
    rows: &[IncomeDocumentItem],
) -> Option<Reply> {
    let request = k1_fixture_answer_request(session, rows)?;
    let mut reply = send(session, request);
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return Some(reply);
    }
    if rows.iter().any(|row| {
        matches!(
            &row.document,
            IncomeSource::K1 {
                value: crate::K1Document::K1PartnershipActivities(_)
                    | crate::K1Document::K1ScorpActivities(_)
            }
        )
    }) {
        let request = k1_fixture_answer_request(session, rows).unwrap();
        reply = send(session, request);
    }
    // The AMT section 179 row is asked only once the carryover row is saved.
    let carries = session.draft.income.iter().any(|entry| match &entry.income {
        crate::FederalIncome::Partnership { data } => data.section179_carryover != 0,
        crate::FederalIncome::Scorp { data } => {
            crate::ScorpK1::from(data).section179_carryover != 0
        }
        _ => false,
    });
    if carries && let Some(request) = k1_fixture_answer_request(session, rows) {
        reply = send(session, request);
    }
    Some(reply)
}

pub(super) fn refresh_k1_fixture_choices(session: &mut Interview, request: &mut P) {
    let P::SaveIncomes { cookie, revision, followups, .. } = request else { unreachable!() };
    for row in followups {
        let crate::DocumentAnswer::K1InvestmentRecords { properties, investment, allowed_choices } =
            &mut row.answer
        else {
            continue;
        };
        let callback = P::K1InvestmentOptions {
            user: TEST_USER,
            cookie: cookie.clone(),
            revision: *revision,
            key: row.key.clone(),
            properties: properties.iter().cloned().map(Into::into).collect(),
        };
        if let Reply::Response(P::K1InvestmentChoices { choices }) = send(session, callback) {
            *allowed_choices = choices;
            if allowed_choices.is_empty() {
                *investment = None;
            }
        }
    }
}

pub(in crate::tests) fn k1_topic_answers(
    session: &mut Interview,
    rows: &[IncomeDocumentItem],
) -> P {
    let accepted_rows = page_documents(&open_documents(session));
    let mut request = open_topic(session, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    for row in followups {
        let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
        let Some(
            supplied @ IncomeDocumentItem { document: IncomeSource::K1 { value: desired }, .. },
        ) = rows.iter().find(|source| source.id == id)
        else {
            continue;
        };
        let IncomeSource::K1 { value: accepted } =
            &accepted_rows.iter().find(|source| source.id == id).unwrap().document
        else {
            unreachable!()
        };
        match &mut row.answer {
            crate::DocumentAnswer::Community { value } => *value = supplied.community,
            crate::DocumentAnswer::Management { value } => *value = supplied.refund_management,
            _ => {}
        }
        fixture_k1_answer(session, row, accepted, desired);
    }
    refresh_k1_fixture_choices(session, &mut request);
    request
}

pub(super) fn fixture_k1_answer(
    session: &Interview,
    row: &mut crate::DocumentQuestion,
    accepted: &crate::K1Document,
    desired: &crate::K1Document,
) {
    use crate::{DocumentAnswer as A, K1Document as K};
    let source_id = row.key.split('/').nth(1).and_then(|id| id.parse::<i64>().ok()).unwrap_or(0);
    fn activity_ids(
        answer: &mut Option<crate::document_center::K1ActivityGraphRecords>,
        accepted: &crate::K1ComponentDetailsPage,
        desired: &crate::K1ComponentDetailsPage,
        accepted_properties: &[crate::K1InvestmentProperty],
        desired_properties: &[crate::K1InvestmentProperty],
        accepted_sales: &[crate::K1Section179Sale],
        desired_sales: &[crate::K1Section179Sale],
    ) {
        let (
            Some(answer),
            crate::K1ComponentStatementPage::Entered { source: old },
            crate::K1ComponentStatementPage::Entered { source: wanted },
        ) = (answer, &accepted.statement, &desired.statement)
        else {
            return;
        };
        answer.choices = accepted.choices.clone();
        let aliases =
            wanted.components.iter().any(|row| {
                row.id > 0 && !old.components.iter().any(|accepted| accepted.id == row.id)
            }) || wanted.qbi_blocks.iter().any(|row| {
                row.id > 0 && !old.qbi_blocks.iter().any(|accepted| accepted.id == row.id)
            });
        // Native fixture assemblies declare role-local aliases before the real
        // source allocator has assigned identities. Preserve accepted identities
        // first so reordered rows retain their existing associations.
        let mapped = |id: i64, wanted: Vec<i64>, old: Vec<i64>| {
            if id > 0 && old.contains(&id) {
                Some(id)
            } else if id == 0 || aliases {
                wanted.iter().position(|found| *found == id).and_then(|i| old.get(i).copied())
            } else {
                None
            }
        };
        let component = |id| {
            mapped(
                id,
                wanted.components.iter().map(|v| v.id).collect(),
                old.components.iter().map(|v| v.id).collect(),
            )
        };
        for (index, row) in answer.components.iter_mut().enumerate() {
            row.id = if row.id == 0 {
                old.components[index].id
            } else {
                component(row.id).unwrap_or(row.id)
            };
            row.risk_group = row.risk_group.and_then(|id| {
                mapped(
                    id,
                    wanted.risk_groups.iter().map(|v| v.id).collect(),
                    old.risk_groups.iter().map(|v| v.id).collect(),
                )
                .or_else(|| {
                    (!(aliases || id == 0) || !wanted.risk_groups.iter().any(|v| v.id == id))
                        .then_some(id)
                })
            });
            row.passive_group = row.passive_group.and_then(|id| {
                mapped(
                    id,
                    wanted.passive_groups.iter().map(|v| v.id).collect(),
                    old.passive_groups.iter().map(|v| v.id).collect(),
                )
                .or_else(|| {
                    (!(aliases || id == 0) || !wanted.passive_groups.iter().any(|v| v.id == id))
                        .then_some(id)
                })
            });
        }
        for (index, row) in answer.risk_groups.iter_mut().enumerate() {
            row.id = if row.id == 0 {
                old.risk_groups.get(index).map_or(0, |v| v.id)
            } else {
                mapped(
                    row.id,
                    wanted.risk_groups.iter().map(|v| v.id).collect(),
                    old.risk_groups.iter().map(|v| v.id).collect(),
                )
                .unwrap_or_else(|| {
                    if aliases && wanted.risk_groups.iter().filter(|v| v.id == row.id).count() == 1
                    {
                        0
                    } else {
                        row.id
                    }
                })
            };
        }
        for (index, row) in answer.passive_groups.iter_mut().enumerate() {
            row.id = if row.id == 0 {
                old.passive_groups.get(index).map_or(0, |v| v.id)
            } else {
                mapped(
                    row.id,
                    wanted.passive_groups.iter().map(|v| v.id).collect(),
                    old.passive_groups.iter().map(|v| v.id).collect(),
                )
                .unwrap_or_else(|| {
                    if aliases
                        && wanted.passive_groups.iter().filter(|v| v.id == row.id).count() == 1
                    {
                        0
                    } else {
                        row.id
                    }
                })
            };
        }
        for (index, row) in answer.qbi_blocks.iter_mut().enumerate() {
            row.id = if row.id == 0 {
                old.qbi_blocks[index].id
            } else {
                mapped(
                    row.id,
                    wanted.qbi_blocks.iter().map(|v| v.id).collect(),
                    old.qbi_blocks.iter().map(|v| v.id).collect(),
                )
                .unwrap_or(row.id)
            };
            for allocation in row.allocations.iter_mut().flatten() {
                allocation.component =
                    component(allocation.component).unwrap_or(allocation.component);
            }
        }
        for row in answer.section179_sale_sources.iter_mut().flatten() {
            row.component = component(row.component).unwrap_or(row.component);
            row.sale = mapped(
                row.sale,
                desired_sales.iter().map(|v| v.id).collect(),
                accepted_sales.iter().map(|v| v.id).collect(),
            )
            .unwrap_or(row.sale);
        }
        for row in answer.investment_property_sources.iter_mut().flatten() {
            row.component = component(row.component).unwrap_or(row.component);
            row.property = mapped(
                row.property,
                desired_properties.iter().map(|v| v.id()).collect(),
                accepted_properties.iter().map(|v| v.id()).collect(),
            )
            .unwrap_or(row.property);
        }
    }
    match (accepted, desired, &mut row.answer) {
        (
            K::K1PartnershipActivities(old),
            K::K1PartnershipActivities(w),
            A::K1PartnerActivityRecords { value },
        ) => {
            *value = w.as_ref().into();
            activity_ids(
                &mut value.activities,
                &old.activities,
                &w.activities,
                &old.entity.investment_property,
                &w.entity.investment_property,
                &old.section179_sales,
                &w.section179_sales,
            );
            return;
        }
        (
            K::K1ScorpActivities(old),
            K::K1ScorpActivities(w),
            A::K1ScorpActivityRecords { value },
        ) => {
            *value = w.into();
            activity_ids(
                &mut value.activities,
                &old.activities,
                &w.activities,
                &old.entity.investment_property,
                &w.entity.investment_property,
                &old.section179_sales,
                &w.section179_sales,
            );
            return;
        }
        _ => {}
    }
    if let (K::K1EstateTrust(_), K::K1EstateTrust(document)) = (accepted, desired) {
        let suffix = row.key.splitn(3, '/').nth(2).unwrap();
        let wanted = estate_fixture_source(document);
        if let Some((_, _, answer, _)) =
            wanted.moved_questions().into_iter().find(|(field, _, _, _)| field == suffix)
        {
            row.answer = answer;
            return;
        }
        let id = row.key.split('/').nth(1).unwrap().parse::<i64>().unwrap();
        let FederalIncome::EstateTrustPortfolio { data } = &session
            .draft
            .income
            .iter()
            .find(|entry| entry.id == id)
            .expect("accepted estate source")
            .income
        else {
            unreachable!()
        };
        let old = &data.statements;
        let w = &wanted.statements;
        match &mut row.answer {
            A::EstateActivityRecords { participation, carried, .. } => {
                let index = old
                    .activities
                    .iter()
                    .position(|item| suffix == alloc::format!("estate/activity/{}", item.id))
                    .unwrap();
                *participation = w.activities[index].participation.clone();
                *carried = w.activities[index].carried.clone();
            }
            A::EstateQbiHistory { activities, passive_history, allocations, .. } => {
                let index = old
                    .qbi
                    .statements
                    .iter()
                    .position(|item| suffix == alloc::format!("estate/qbi/{}", item.id))
                    .unwrap();
                let proposed = &w.qbi.statements[index];
                let accepted_id = |id| {
                    w.activities
                        .iter()
                        .position(|activity| activity.id == id)
                        .and_then(|index| old.activities.get(index))
                        .map_or(id, |activity| activity.id)
                };
                *activities = proposed.activities.iter().map(|id| accepted_id(*id)).collect();
                *passive_history = proposed.passive_history.clone();
                *allocations = proposed
                    .allocations
                    .iter()
                    .map(|part| {
                        let activity = accepted_id(part.activity);
                        crate::EstateQbiAllocationHistory {
                            activity,
                            income: part.income,
                            passive_history: part.passive_history.clone(),
                        }
                    })
                    .collect();
            }
            A::EstateAmtTreatment { treatment, .. } => {
                let previous = old.amt_reconciliation.as_ref().unwrap();
                let index = previous
                    .adjustments
                    .iter()
                    .position(|item| suffix == alloc::format!("estate/amt/{}", item.id))
                    .unwrap();
                *treatment = w.amt_reconciliation.as_ref().unwrap().adjustments[index]
                    .treatment
                    .clone()
                    .into();
            }
            A::EstateInvestmentRecords { properties, .. } => {
                for answer in properties {
                    let index = old
                        .investment_property
                        .iter()
                        .position(|item| item.property.id() == answer.id)
                        .unwrap();
                    answer.own_form = w.investment_property[index].property.own_form().cloned();
                }
            }
            A::EstateFinalSuccessor { value } => *value = wanted.answers.final_year_successor,
            A::EstateFinalRecords { mortgages } => {
                *mortgages = wanted.answers.final_mortgages.clone();
            }
            A::EstateReitEligibility { value } => {
                *value = match w.section199a_dividends {
                    crate::ReitDividends::None => None,
                    crate::ReitDividends::Reported { eligible, .. } => eligible,
                };
            }
            _ => {}
        }
        return;
    }
    match (desired, &mut row.answer) {
        (K::K1Partnership(w), A::K1ResearchCreditLine { value }) => {
            *value = w.personal.as_ref().and_then(|records| records.research_credit_line);
        }
        (K::K1Partnership(w), A::K1PartnerRecords { .. }) => {
            let w = w.calculation_page();
            row.answer = A::K1PartnerRecords {
                qbi_records: w.qbi_records_record(),
                basis: crate::PartnerBasisRecords::from_stored(&w.basis),
                income_source: w.income_source.clone(),
                special: w.special.clone(),
                qbi_group: w.qbi_group.clone(),
                fuel_credit: w.more.fuel_credit.clone(),
                prior_fuel_credit: w.more.prior_fuel_credit.clone(),
                term: w.more.term,
                trading: w.more.trading,
            }
        }
        (K::K1Partnership(w), A::K1PartnerBasisStart { value }) => {
            *value = Some(w.calculation_page().basis.start)
        }
        (K::K1Scorp(w), A::K1ScorpBasisStart { value }) => {
            *value = Some(scorp_fixture_source_in(&session.draft, source_id, w).page().basis.start)
        }
        (K::K1Partnership(w), A::K1PartnerParticipation { value }) => {
            *value = w.calculation_page().participation
        }
        (K::K1Partnership(w), A::K1LimitedForSe { value }) => {
            *value = w.calculation_page().limited_for_se
        }
        (K::K1Partnership(w), A::K1PartnerAmt { value }) => *value = w.calculation_page().amt,
        (K::K1Partnership(w), A::K1PartnerDisposition { value }) => {
            *value = w.calculation_page().disposition
        }
        (K::K1Scorp(w), A::K1ScorpParticipation { value }) => {
            *value = scorp_fixture_source_in(&session.draft, source_id, w).page().participation
        }
        (K::K1Scorp(w), A::K1ScorpAmt { value }) => {
            *value = scorp_fixture_source_in(&session.draft, source_id, w).page().amt
        }
        (K::K1Scorp(w), A::K1ScorpDisposition { value }) => {
            *value = scorp_fixture_source_in(&session.draft, source_id, w).page().disposition
        }
        (K::K1Scorp(w), A::K1ResearchCreditLine { value }) => {
            *value =
                scorp_fixture_source_in(&session.draft, source_id, w).personal.research_credit_line;
        }
        (K::K1Scorp(w), A::K1ScorpRecords { .. }) => {
            let w = scorp_fixture_source_in(&session.draft, source_id, w).page();
            row.answer = A::K1ScorpRecords {
                qbi_records: w.qbi_records_record(),
                basis: crate::K1ScorpBasisRecords::from_stored(&w.basis),
                special: w.special.clone(),
                qbi_group: w.qbi_group.clone(),
                stock_term: w.more.stock_term,
                trading: w.more.trading,
            }
        }
        (K::K1PartnershipActivities(w), A::K1EntityDisposition { value }) => {
            *value = w.entity.disposition.clone()
        }
        (K::K1ScorpActivities(w), A::K1EntityDisposition { value }) => {
            *value = w.entity.disposition.clone()
        }
        _ => {}
    }
    macro_rules! records {
        ($w:expr) => {
            match &mut row.answer {
                A::K1RentalParticipation { value } => *value = $w.rentals.participation,
                A::K1RentalRecords { carryover_active, disposed } => {
                    *carryover_active = $w.rentals.carryover_active;
                    *disposed = $w.rentals.disposed.clone();
                }
                A::K1Section179Amt { value } => {
                    *value = $w.more.amt_section179_income_carry.clone();
                }
                A::K1Section179Records { carryover, sold } => {
                    *carryover = $w.more.section179_carryover;
                    *sold = $w.more.section179_carryover_sold;
                }
                A::K1K3Withholding { value } => {
                    *value = match &$w.schedule_k3 {
                        crate::K3Attached::NotAttached => Vec::new(),
                        crate::K3Attached::Attached(k3) => k3.withholding.clone(),
                    };
                }
                _ => {}
            }
        };
    }
    match desired {
        K::K1Partnership(w) => {
            let w = w.calculation_page();
            records!(w);
        }
        K::K1Scorp(w) => {
            let w = scorp_fixture_source_in(&session.draft, source_id, w).page();
            records!(w);
        }
        _ => {}
    }
    let parts = |source: &crate::K1Document| {
        Some(match source {
            K::K1Partnership(document) => {
                let mut w = document.calculation_page();
                // Desired zero-ID fixture forms are submitted on the topic
                // after source admission assigns their actual identities.
                for (property, supplied) in w
                    .more
                    .investment_property
                    .iter_mut()
                    .zip(&document.statements.investment_property)
                {
                    if property.id() == 0 && property.own_form().is_none() {
                        *property.own_form_mut() = supplied.own_form().cloned();
                    }
                }
                (
                    w.more.credits.clone(),
                    w.more.investment_property.clone(),
                    w.more.housing_confirmed,
                    w.more.investment_confirmed,
                )
            }
            K::K1Scorp(document) => {
                let mut w = scorp_fixture_source_in(&session.draft, source_id, document).page();
                // Desired zero-ID fixture forms are submitted on the topic
                // after source admission assigns their actual identities.
                for (property, supplied) in w
                    .more
                    .investment_property
                    .iter_mut()
                    .zip(&document.statements.investment_property)
                {
                    if property.id() == 0 && property.own_form().is_none() {
                        *property.own_form_mut() = supplied.own_form().cloned();
                    }
                }
                (
                    w.more.credits.clone(),
                    w.more.investment_property.clone(),
                    w.more.housing_confirmed,
                    w.more.investment_confirmed,
                )
            }
            K::K1PartnershipActivities(w) => (
                w.entity.credits.clone(),
                w.entity.investment_property.clone(),
                w.entity.housing_confirmed.map(Into::into),
                w.entity.investment_confirmed,
            ),
            K::K1ScorpActivities(w) => (
                w.entity.credits.clone(),
                w.entity.investment_property.clone(),
                w.entity.housing_confirmed.map(Into::into),
                w.entity.investment_confirmed,
            ),
            _ => return None,
        })
    };
    let Some((credits, properties, housing, investment)) = parts(desired) else {
        return;
    };
    let (_, accepted_properties, _, _) = parts(accepted).unwrap();
    let suffix = row.key.splitn(3, '/').nth(2).unwrap();
    match &mut row.answer {
        A::K1CreditConfirmations { housing: value } => {
            *value = housing;
        }
        A::K1InvestmentRecords { properties: answers, investment: choice, .. } => {
            *choice = investment;
            for answer in answers {
                let index = accepted_properties
                    .iter()
                    .position(|property| property.id() == answer.id)
                    .unwrap();
                answer.own_form = properties[index].own_form().cloned();
            }
        }
        A::K1CreditOrigins { carbon_oxide_origin, renewable_facility_origin } => {
            let credit = credits
                .iter()
                .find(|credit| {
                    let native: crate::K1Credit = (*credit).clone().into();
                    suffix == alloc::format!("credit/{:?}/{:?}/origins", native.code, native.line)
                })
                .unwrap();
            *carbon_oxide_origin = credit.carbon_oxide_origin.clone();
            *renewable_facility_origin = credit.renewable_facility_origin.clone();
        }
        _ => {}
    }
}
// A relocated K-1 paper path and the page field its topic answers' validators
// report; the topic control below binds an answer-step refusal from the latter.
pub(in crate::tests) fn remember_k1_page_local(path: &str, local: &str) {
    K1_PAGE_LOCALS.with(|rows| {
        let mut rows = rows.borrow_mut();
        rows.retain(|(found, _)| found != path);
        rows.push((path.into(), local.into()));
    });
}
pub(super) fn k1_page_local(path: &str) -> Option<String> {
    K1_PAGE_LOCALS.with(|rows| {
        rows.borrow().iter().find(|(found, _)| found == path).map(|(_, local)| local.clone())
    })
}
