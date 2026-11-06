use super::income::{save_native_income_topics, save_native_review_topics};
use super::lifecycle::{TEST_USER, exchange, restart, send};
use crate::interview::{Interview, Reply};
use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::support::assert::{supported, value};
use crate::tests::support::builders::complete;
use crate::tests::*;
use crate::{
    FederalIncome, FederalStatus, InjuredAmountPage, InjuredSpouse, SpouseSituations,
    TaxInterviewProtocol as P, TaxSection,
};
use alloc::vec::Vec;

// Native tests submit complete Rust values. This explicit fixture adapter
// copies the real WorkflowRequest context and requires an explicit test answer
// for every new row. It never supplies a range bound or a default answer.
// It does not model client initialization; actual Studio acceptance separately
// proves that an untouched destination-only required amount blocks Submit.
pub(crate) fn native_editor(reply: Reply, new_answers: &[(&str, i64)]) -> Reply {
    native_editor_with_lists(reply, new_answers, false)
}

fn native_editor_with_lists(
    reply: Reply,
    new_answers: &[(&str, i64)],
    preserve_lists: bool,
) -> Reply {
    let Reply::Response(P::BeginInjuredSpouse {
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        warnings,
        progress,
        value,
        followups,
        questions: _,
    }) = reply
    else {
        assert!(new_answers.is_empty());
        return reply;
    };
    assert_eq!(
        value.new_allocations.len(),
        new_answers.len(),
        "every new component needs an explicit fixture answer: {:?}",
        value.new_allocations
    );
    for (index, (component, _)) in new_answers.iter().enumerate() {
        assert!(!new_answers[..index].iter().any(|(earlier, _)| earlier == component));
    }
    let mut allocations: Vec<_> = value
        .allocations
        .into_iter()
        .map(|row| InjuredAmountPage {
            component: row.component,
            label: row.label,
            joint: row.joint,
            minimum: row.minimum,
            maximum: row.maximum,
            injured: row.injured,
        })
        .collect();
    // These are deliberate completed native request fixtures, not an echo of
    // the untouched editor. The server accepts the combined vector regardless
    // of which service-owned request list carries an answered row.
    let mut new_allocations: Vec<_> = value
        .new_allocations
        .into_iter()
        .map(|row| {
            let injured = new_answers
                .iter()
                .find(|(component, _)| *component == row.component)
                .unwrap_or_else(|| panic!("explicit answer missing for {}", row.component))
                .1;
            InjuredAmountPage {
                component: row.component,
                label: row.label,
                joint: row.joint,
                minimum: row.minimum,
                maximum: row.maximum,
                injured,
            }
        })
        .collect();
    if !preserve_lists {
        allocations.append(&mut new_allocations);
    }
    Reply::Edit(P::SaveInjuredSpouse {
        user: TEST_USER,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate: estimate.map(Into::into),
        warnings,
        progress,
        followups,
        questions: Vec::new(),
        value: InjuredSpouse {
            injured: value.injured,
            debt: value.debt,
            obligated: value.obligated,
            community: value.community,
            estimated: value.estimated,
            student_loan_borrower: value.student_loan_borrower,
            allocations,
            new_allocations,
            claimants: value.claimants,
            joint_estimated: value.joint_estimated,
            student_loan_deduction: value.student_loan_deduction,
            states: value.states,
            people: value.people,
            components: value.components,
            allowed_debt_facts: value.allowed_debt_facts,
            known_child_credit_injured: value.known_child_credit_injured,
            known_child_credit_other: value.known_child_credit_other,
            child_credit_cap_injured: value.child_credit_cap_injured,
            child_credit_cap_other: value.child_credit_cap_other,
            known: value.known,
            estimated_share: value.estimated_share,
            explanation: value.explanation,
        },
    })
}

pub(crate) fn injured_capacity_claim() -> Interview {
    use crate::tests::cross::interview::{accepted, save_whole_return};
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    // Request the claim through the real checklist/editor. An initially empty
    // source plan has no allocation choices to invent.
    let Reply::Edit(initial @ P::SaveInjuredSpouse { .. }) = injured_capacity_editor_reply(
        save_whole_return(&mut session, SpouseSituations::INJURED_SPOUSE.0),
    ) else {
        panic!("expected normal injured-spouse editor")
    };
    accepted(&send(&mut session, initial));
    assert!(session.draft.injured_spouse.is_some());
    assert!(crate::injured_spouse_sources::requested(&session.draft));

    session
}

pub(crate) fn injured_capacity_sources(session: &mut Interview, insured_count: usize) -> Vec<i64> {
    // Stage1 allocates the actual insured-source identities. One-period rows
    // obtain their receipt amount from linked forms; none have been entered yet.
    let insured = super::builders::injured_insured_documents(insured_count);
    let request =
        super::income_pages::save_topic(session, TaxSection::LongTermCareBenefits, insured);
    crate::tests::cross::interview::accepted(&send(session, request));
    let ids: Vec<_> = session
        .draft
        .income
        .iter()
        .filter_map(|e| matches!(e.income, FederalIncome::LtcInsured { .. }).then_some(e.id))
        .collect();
    assert_eq!(ids.len(), insured_count);
    assert!(ids.iter().all(|id| *id > 0));
    assert_eq!(
        ids.iter().copied().collect::<alloc::collections::BTreeSet<_>>().len(),
        insured_count
    );

    ids
}

pub(crate) fn injured_capacity_forms(session: &mut Interview, ids: &[i64]) -> [Option<i64>; 5] {
    use crate::tests::cross::interview::page_documents;
    // Stage2 enters actual daily contract periods and both physical source
    // forms. The period-source IDs are unanswered until these forms receive
    // their own allocator identities; ordinary SectionC tax is already known.
    let request = super::income_pages::open_topic(session, TaxSection::LongTermCareBenefits);
    let mut rows = page_documents(&request).to_vec();
    super::builders::injured_daily_forms(&session.draft, &mut rows, ids);
    super::income_pages::save_each_topic(session, rows);
    assert_eq!(session.draft.income.len(), 1 + ids.len() * 3);
    let known_joint = supported(&session.draft);
    // Actual2026 daily allowance$430; $1000 received less$430 = $570 taxable
    // for each contract day of each independently insured person.
    assert_eq!(
        value(&known_joint, "taxable_ltc_benefits"),
        super::builders::INJURED_TAXABLE_PER_PERIOD
            * (ids.len() * super::builders::INJURED_CONTRACT_DAYS) as i64
    );

    [
        known_joint.tax,
        known_joint.refundable_credits,
        known_joint.payments,
        known_joint.refund,
        known_joint.balance,
    ]
}

pub(crate) fn injured_capacity_payees(session: &mut Interview) {
    use crate::tests::cross::interview::page_documents;
    // Stage3 identifies each day's two actual payments by the accepted source
    // IDs. The native reference owner must reconcile each source's annual total.
    let request = super::income_pages::open_topic(session, TaxSection::LongTermCareBenefits);
    let mut rows = page_documents(&request).to_vec();
    super::builders::injured_period_payees(&session.draft, &mut rows);
    let request = super::income_pages::save_topic(session, TaxSection::LongTermCareBenefits, rows);
    crate::tests::cross::interview::accepted(&send(session, request));
    assert!(session.draft.valid_stored());
    assert!(crate::injured_spouse_sources::reference_issue(&session.draft).is_none());
}

// Use the same real source admission path for both the two-row refusal fixture
// and the twelve-insured producer fixture.
pub(crate) fn injured_capacity_source_plan(
    insured_count: usize,
) -> (Interview, Vec<i64>, [Option<i64>; 5]) {
    let mut session = injured_capacity_claim();
    let ids = injured_capacity_sources(&mut session, insured_count);
    let known_joint = injured_capacity_forms(&mut session, &ids);
    injured_capacity_payees(&mut session);
    (session, ids, known_joint)
}

pub(crate) fn injured_capacity_accepted_session() -> Interview {
    let (mut session, _, _) = injured_capacity_source_plan(1);
    // The normal editor supplies the complete source plan; its fixture adapter
    // explicitly answers every new row with 7,000, never an echoed bound.
    let request = injured_capacity_editor(&mut session);
    crate::tests::cross::interview::accepted(&send(&mut session, request));
    assert_eq!(
        session.draft.injured_spouse.as_ref().unwrap().allocations.len(),
        super::builders::INJURED_CONTRACT_DAYS
    );
    let saved = native_draft_records(&session.draft);
    let mut cold = restart(session.draft);
    let request = injured_capacity_editor(&mut cold);
    crate::tests::cross::interview::accepted(&send(&mut cold, request));
    assert_eq!(native_draft_records(&cold.draft), saved);
    cold
}

pub(crate) fn injured_capacity_calculation(request: &P) -> P {
    let P::SaveInjuredSpouse { value, followups, .. } = request else {
        panic!("actual injured-spouse editor")
    };
    P::EstimateInjuredSpouse {
        user: TEST_USER,
        value: value.clone(),
        followups: followups.clone(),
        questions: Vec::new(),
    }
}

pub(crate) fn injured_capacity_refusal(
    session: &mut Interview,
    request: P,
    case: &str,
    field: &str,
    save_refresh: bool,
) {
    let before = native_draft_records(&session.draft);
    let dirty = session.dirty;
    for (kind, input) in
        [("save", request.clone()), ("estimate", injured_capacity_calculation(&request))]
    {
        let draft_before = session.draft.clone();
        let reply = exchange(session, input, &alloc::format!("injured-ltc-{case}-{kind}-reply"));
        assert_eq!(session.draft, draft_before);
        assert_eq!(session.dirty, dirty);
        if kind == "save" && save_refresh {
            let Reply::Edit(corrected @ P::SaveInjuredSpouse { .. }) =
                injured_capacity_editor_reply(reply)
            else {
                panic!("expected native source-page refresh: {case}/{kind}")
            };
            let P::SaveInjuredSpouse { value, .. } = &corrected else { unreachable!() };
            let expected = supported(&session.draft)
                .injured_spouse
                .expect("actual accepted source plan")
                .required;
            assert_eq!(value.allocation_rows().cloned().collect::<Vec<_>>(), expected);
            assert_eq!(value.allocation_rows().count(), super::builders::INJURED_CONTRACT_DAYS);
            assert!(value.allocation_rows().all(|row| {
                (row.joint, row.minimum, row.maximum, row.injured) == (57_000, 7_000, 50_000, 7_000)
            }));
            // The refreshed complete page is a valid correction, not an
            // accepted version of the foreign/missing/forged submission.
            crate::tests::cross::interview::accepted(&send(session, corrected));
            assert_eq!(session.dirty, dirty);
            assert_eq!(native_draft_records(&session.draft), before);
            continue;
        }
        let Reply::Response(problem @ P::Problem { .. }) = reply else {
            panic!("expected native allocation refusal: {case}/{kind}")
        };
        let P::Problem { message, field: actual } = &problem else { unreachable!() };
        assert_eq!(
            *message,
            libertas::libertas_formatted_text("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT", &[])
        );
        assert_eq!(actual.as_deref(), Some(field));
    }
}

pub(crate) fn injured_capacity_answer_page_is_pure(session: &mut Interview) {
    let before = session.draft.clone();
    let _ = injured_capacity_editor(session);
    assert_eq!(session.draft, before);
}

// Explicit native answers to the actual WorkflowRequest's new rows. Keep the
// service's answered/new partition rather than flattening the two-list editor.
pub(crate) fn injured_capacity_editor_reply(reply: Reply) -> Reply {
    let Reply::Response(P::BeginInjuredSpouse { value, .. }) = &reply else {
        return reply;
    };
    // Explicit fixture answers, never initialization from an echoed bound.
    let answers: Vec<_> =
        value.new_allocations.iter().map(|row| (row.component.clone(), 7_000)).collect();
    let answers: Vec<_> = answers.iter().map(|(key, amount)| (key.as_str(), *amount)).collect();
    native_editor_with_lists(reply, &answers, true)
}

pub(crate) fn injured_capacity_editor(session: &mut Interview) -> P {
    let reply = crate::tests::cross::interview::from_review(session, TaxSection::InjuredSpouse);
    let Reply::Edit(request @ P::SaveInjuredSpouse { .. }) = injured_capacity_editor_reply(reply)
    else {
        panic!("expected the actual injured-spouse editor")
    };
    request
}

pub(in crate::tests) fn persist_niit_deduction(
    d: FederalDraft,
    kind: NiitDeductionExpense,
) -> FederalDraft {
    let before = assert_niit_deduction_money(&d);
    let d = known_fact_partnership_draft(d);
    let ids: Vec<_> = d.income.iter().map(|entry| entry.id).collect();
    assert!(ids.iter().all(|id| *id > 0));
    let mut session = restart(d);
    save_native_income_topics(&mut session, &[TaxSection::Documents]);
    if matches!(
        kind,
        NiitDeductionExpense::Estate
            | NiitDeductionExpense::PartnerInterest
            | NiitDeductionExpense::ScorpInterest
    ) {
        let request = super::income_pages::open_document_center(&mut session);
        accepted(&send(&mut session, request));
    }
    save_native_review_topics(
        &mut session,
        &[TaxSection::Deductions, TaxSection::OtherItemized, TaxSection::MoreCredits],
    );
    let mut after_ids: Vec<_> = session.draft.income.iter().map(|entry| entry.id).collect();
    let mut ids = ids;
    ids.sort_unstable();
    after_ids.sort_unstable();
    assert_eq!(after_ids, ids);
    let mut session = restart_stored_niit_draft(session);
    let Reply::Edit(request) = from_review(&mut session, TaxSection::OtherItemized) else {
        panic!()
    };
    accepted(&send(&mut session, request));
    let after = assert_niit_deduction_money(&session.draft);
    assert_native_money_unchanged(&after, &before);
    session.draft
}

pub(in crate::tests) fn persist_niit_binding(d: FederalDraft) -> FederalDraft {
    let before = assert_niit_binding_money(&d);
    let mut ids: Vec<_> = d.income.iter().map(|e| e.id).collect();
    assert!(ids.iter().all(|id| *id > 0));
    ids.sort_unstable();
    let mut session = restart(d);
    save_native_income_topics(&mut session, &[TaxSection::Documents]);
    for topic in [TaxSection::Deductions, TaxSection::OtherItemized, TaxSection::InjuredSpouse] {
        let Reply::Edit(request) =
            crate::tests::support::client::native_editor(from_review(&mut session, topic), &[])
        else {
            panic!("expected real editor")
        };
        accepted(&send(&mut session, request));
    }
    let mut session = restart_stored_niit_draft(session);
    let Reply::Edit(request) = from_review(&mut session, TaxSection::OtherItemized) else {
        panic!("expected reopened editor")
    };
    accepted(&send(&mut session, request));
    let after = assert_niit_binding_money(&session.draft);
    assert_native_money_unchanged(&after, &before);
    let mut after_ids: Vec<_> = session.draft.income.iter().map(|e| e.id).collect();
    after_ids.sort_unstable();
    assert_eq!(after_ids, ids);
    session.draft
}

pub(in crate::tests) fn niit_person_editor(
    session: &mut Interview,
    source: NiitExpenseSource,
) -> P {
    let Reply::Edit(request) = from_review(session, source.topic()) else {
        panic!("source editor")
    };
    request
}

pub(in crate::tests) fn complete_niit_allocation(session: &mut Interview) -> P {
    niit_allocation_with_answers(session, &[])
}

pub(in crate::tests) fn niit_allocation_with_answers(
    session: &mut Interview,
    answers: &[(&str, i64)],
) -> P {
    let Reply::Edit(request) = crate::tests::support::client::native_editor(
        from_review(session, TaxSection::InjuredSpouse),
        answers,
    ) else {
        panic!("actual native Workflow construction context")
    };
    request
}

pub(in crate::tests) fn restart_stored_niit_draft(session: Interview) -> Interview {
    restart(roundtrip_native_draft(session.draft))
}

pub(in crate::tests) fn persist_niit_zero(d: FederalDraft, deduction_dollars: i64) -> FederalDraft {
    let before = assert_niit_zero_money(&d, deduction_dollars);
    let mut ids: Vec<_> = d.income.iter().map(|entry| entry.id).collect();
    assert!(ids.iter().all(|id| *id > 0));
    ids.sort_unstable();
    let mut session = restart(d);
    super::income::admit_native_fixture_facts(&mut session);
    save_native_income_topics(&mut session, &[TaxSection::Documents]);
    save_native_review_topics(&mut session, &[TaxSection::Deductions, TaxSection::OtherItemized]);
    let mut session = restart_stored_niit_draft(session);
    let Reply::Edit(request) = from_review(&mut session, TaxSection::OtherItemized) else {
        panic!("expected reopened editor")
    };
    accepted(&send(&mut session, request));
    let after = assert_niit_zero_money(&session.draft, deduction_dollars);
    assert_native_money_unchanged(&after, &before);
    let mut after_ids: Vec<_> = session.draft.income.iter().map(|entry| entry.id).collect();
    after_ids.sort_unstable();
    assert_eq!(after_ids, ids);
    session.draft
}

impl NiitExpenseSource {
    pub(in crate::tests) fn supply(self, request: &mut P, owner: Option<DeductionOwners>) {
        match request {
            P::SaveOtherItemized { value, .. } => match self {
                Self::Paid => {
                    value.investment_interest.as_mut().unwrap().loans[0].paid_owners = owner
                }
                Self::Carry => {
                    value.investment_interest.as_mut().unwrap().loans[0].carry_owners = owner
                }
                Self::Property => value.investment_property_tax_owners = owner,
                _ => unreachable!(),
            },
            P::SaveMoreCredits { value, .. } => {
                let MoreCredit::ForeignTax(ForeignTaxPage::Deduction { taxes }) = value
                    .credits
                    .iter_mut()
                    .find(|c| matches!(c, MoreCredit::ForeignTax(_)))
                    .unwrap()
                else {
                    unreachable!()
                };
                taxes[0].owners = owner;
            }
            P::SaveTaxes { value, .. } => value.income_tax_owners = owner,
            _ => unreachable!(),
        }
    }
}

pub(in crate::tests) fn persist_ebl_sources(draft: FederalDraft) -> FederalDraft {
    save_reopen_native_sources(
        draft,
        &[TaxSection::SelfEmployment, TaxSection::InjuredSpouse],
        &[TaxSection::InjuredSpouse],
    )
}

pub(in crate::tests) fn persist_current_nua_sources(draft: FederalDraft) -> FederalDraft {
    let mut ids: Vec<_> = draft.income.iter().map(|entry| entry.id).collect();
    assert!(ids.iter().all(|id| *id > 0));
    ids.sort_unstable();
    let saved = save_reopen_checked_sources(
        draft,
        &[TaxSection::Documents, TaxSection::MoreCredits],
        &[TaxSection::Documents],
        assert_current_nua_money,
    );
    let mut after_ids: Vec<_> = saved.income.iter().map(|entry| entry.id).collect();
    after_ids.sort_unstable();
    assert_eq!(after_ids, ids);
    saved
}

pub(in crate::tests) fn persist_hobby_niit_sources(draft: FederalDraft) -> FederalDraft {
    save_reopen_checked_sources(
        draft,
        &[TaxSection::OtherIncome],
        &[TaxSection::OtherIncome],
        assert_hobby_niit_money,
    )
}
