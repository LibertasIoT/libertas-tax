use crate::tests::*;

pub(in crate::tests) fn credit_document_answers_request(
    session: &mut Interview,
    bonds: &[FederalTaxCreditBond1097],
    audit: Option<&FederalBbaAuditInterest>,
) -> P {
    answer_credit_questions(session, bonds, audit);
    let Reply::Edit(request @ P::SaveMoreCredits { .. }) =
        crate::tests::cross::interview::from_review(session, TaxSection::MoreCredits)
    else {
        panic!("the accepted document's MoreCredits editor must be available");
    };
    let P::SaveMoreCredits { value, .. } = &request else { unreachable!() };
    for bond in bonds {
        assert!(
            value.documents.iter().any(|row| matches!(row,
                crate::CreditDocumentView::CreditBond { value } if value.id == bond.id)),
            "the accepted bond must appear on its topic"
        );
    }
    request
}

pub(in crate::tests) fn more_credits_calculation(request: &P) -> P {
    match request {
        P::SaveMoreCredits {
            allowed_credit_owners,
            credit_income_sources,
            allowed_owners,
            foreign_businesses,
            capital_sales,
            foreign_situations,
            mortgage_loans,
            businesses,
            farms,
            clean_vehicle_properties,
            rentals,
            research_businesses,
            entered_credits,
            carryback_credits,
            allowed_carry_filers,
            initial_credit_elections,
            initial_research_choices,
            value,
            initial_carbon_oxide_origins,
            initial_renewable_facility_origins,
            ..
        } => P::EstimateMoreCredits {
            allowed_credit_owners: allowed_credit_owners.clone(),
            credit_income_sources: credit_income_sources.clone(),
            user: TEST_USER,
            allowed_owners: allowed_owners.clone(),
            foreign_businesses: foreign_businesses.clone(),
            estate_foreign_sources: vec![],
            capital_sales: capital_sales.clone(),
            foreign_situations: foreign_situations.clone(),
            mortgage_loans: mortgage_loans.clone(),
            businesses: businesses.clone(),
            farms: farms.clone(),
            clean_vehicle_properties: clean_vehicle_properties.clone(),
            rentals: rentals.clone(),
            research_businesses: research_businesses.clone(),
            entered_credits: entered_credits.clone(),
            carryback_credits: carryback_credits.clone(),
            allowed_carry_filers: allowed_carry_filers.clone(),
            initial_credit_elections: initial_credit_elections.clone(),
            initial_research_choices: initial_research_choices.clone(),
            value: value.clone(),
            initial_carbon_oxide_origins: initial_carbon_oxide_origins.clone(),
            initial_renewable_facility_origins: initial_renewable_facility_origins.clone(),
        },
        _ => panic!("the More Credits editor is required"),
    }
}

// The open link on the More credits page, as the client sends it.
pub(in crate::tests) fn open_credit_questions(session: &mut Interview) -> Reply {
    let request = P::OpenDocumentQuestions {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        review: false,
        topic: TaxSection::MoreCredits,
    };
    crate::tests::support::client::own_questions::wire(session, request)
}

// The strict answer the record gives a question, when it has one.
pub(in crate::tests) fn record_answer(
    key: &str,
    bonds: &[FederalTaxCreditBond1097],
    audit: Option<&FederalBbaAuditInterest>,
) -> Option<crate::DocumentStrictAnswer> {
    use crate::DocumentStrictAnswer as T;
    let mut parts = key.split('/').skip(1);
    let id: i64 = parts.next()?.parse().ok()?;
    let field = parts.next()?;
    if let Some(audit) = audit.filter(|audit| audit.id == id) {
        let a = audit.clone().into_parts().1;
        return match field {
            "prior_wages" => a.prior_wages.map(|value| T::BbaPriorWages { value }),
            "prior_taxable_income" => {
                a.prior_taxable_income.map(|value| T::BbaPriorTaxableIncome { value })
            }
            "prior_income_tax" => a.prior_income_tax.map(|value| T::BbaPriorIncomeTax { value }),
            "prior_status" => a.prior_status.map(|value| T::BbaPriorStatus { value }),
            "single_audit_adjustment" => {
                a.single_audit_adjustment.map(|value| T::BbaSingleAdjustment { value })
            }
            "simple_prior_return" => {
                a.simple_prior_return.map(|value| T::BbaSimplePriorReturn { value })
            }
            "no_other_effects" => a.no_other_effects.map(|value| T::BbaNoOtherEffects { value }),
            "line_17_interest" => a
                .line_17_interest
                .zip(a.interest_calculation_prepared)
                .map(|(interest, prepared)| T::BbaLine17Interest { interest, prepared }),
            _ => None,
        };
    }
    let a = &bonds.iter().find(|bond| bond.id == id)?.answers;
    match field {
        "kind" => a.kind.map(|value| T::BondClass { value }),
        "issued_on" => a.issued_on.map(|value| T::BondIssueDate { value }),
        "origin" => a.origin.clone().map(|value| T::BondOrigin { value: value.into() }),
        "prior_carry" => a
            .prior_carry_confirmed
            .map(|confirmed| T::BondPriorCarry { dollars: a.prior_carry_dollars, confirmed }),
        "allowed_share" => a
            .allowed_credit_dollars
            .zip(a.allowed_credit_confirmed)
            .map(|(dollars, confirmed)| T::BondAllowedShare { dollars, confirmed }),
        _ => None,
    }
}

// Answers every open question the records answer, through the link and the
// questions page; conditional rows open once the others are answered.
pub(in crate::tests) fn answer_credit_questions(
    session: &mut Interview,
    bonds: &[FederalTaxCreditBond1097],
    audit: Option<&FederalBbaAuditInterest>,
) {
    for _ in 0..3 {
        let Reply::Response(begin @ P::BeginDocumentQuestions { .. }) =
            open_credit_questions(session)
        else {
            return;
        };
        let P::BeginDocumentQuestions { questions, .. } = &begin else { unreachable!() };
        let answered: Vec<_> = questions
            .iter()
            .filter_map(|asked| {
                Some(crate::DocumentStrictQuestion {
                    key: asked.key.clone(),
                    prompt: Vec::new(),
                    allowed_answers: Vec::new(),
                    allowed_values: Vec::new(),
                    ccc_lots: Vec::new(),
                    health_plans: Vec::new(),
                    espp_acquisitions: Vec::new(),
                    credit_income_sources: Vec::new(),
                    interest_statements: Vec::new(),
                    people: Vec::new(),
                    answer: record_answer(&asked.key, bonds, audit)?,
                })
            })
            .collect();
        if answered.is_empty() {
            return;
        }
        let reply =
            crate::tests::support::client::own_questions::save_questions(session, &begin, answered);
        if matches!(reply, Reply::Response(P::Problem { .. })) {
            return;
        }
    }
}

// A bond's open origin question, answered from the record: the questions
// page's Save request, not yet sent.
pub(in crate::tests) fn origin_question(
    session: &mut Interview,
    bonds: &[FederalTaxCreditBond1097],
    open: usize,
) -> P {
    let mut asked = bonds.to_vec();
    asked[open].answers.origin = None;
    answer_credit_questions(session, &asked, None);
    let Reply::Response(begin @ P::BeginDocumentQuestions { .. }) = open_credit_questions(session)
    else {
        panic!("the bond's origin question stays open");
    };
    let P::BeginDocumentQuestions { questions, .. } = &begin else { unreachable!() };
    assert_eq!(questions.len(), 1, "only the origin question is open");
    let answer = record_answer(&questions[0].key, bonds, None).unwrap();
    crate::tests::support::client::own_questions::questions_request(
        &begin,
        vec![crate::DocumentStrictQuestion {
            key: questions[0].key.clone(),
            prompt: Vec::new(),
            allowed_answers: Vec::new(),
            allowed_values: Vec::new(),
            ccc_lots: Vec::new(),
            health_plans: Vec::new(),
            espp_acquisitions: Vec::new(),
            credit_income_sources: Vec::new(),
            interest_statements: Vec::new(),
            people: Vec::new(),
            answer,
        }],
    )
}

// The origin answer's entity on a questions-page Save.
pub(in crate::tests) fn origin_entity(request: &mut P) -> &mut crate::StrictTaxCreditEntity {
    let P::SaveDocumentQuestions { questions, .. } = request else { unreachable!() };
    let crate::DocumentStrictAnswer::BondOrigin {
        value:
            crate::StrictTaxCreditOrigin::EntityStatement(entity)
            | crate::StrictTaxCreditOrigin::Entity1097(entity),
    } = &mut questions[0].answer
    else {
        panic!("an entity origin answer")
    };
    entity
}

// A refused origin answer saves nothing; the corrected one is accepted and
// survives a restart.
pub(in crate::tests) fn origin_case(
    case: &str,
    session: &mut Interview,
    bad: P,
    corrected: P,
    key: &str,
    path: &str,
) {
    crate::tests::support::client::survey::survey_native_refusal(case, session, bad, key, path);
    let before = session.draft.revision;
    assert!(matches!(send(session, corrected), Reply::Edit(P::SaveMoreCredits { .. })));
    assert_eq!(session.draft.revision, before + 1);
    let saved = session.draft.clone();
    assert_eq!(crate::tests::support::client::restart(saved.clone()).draft, saved);
}

// The answered origin row of the `bond`th accepted bond on the More credits page.
pub(in crate::tests) fn answered_origin(
    request: &mut P,
    bond: usize,
) -> (usize, &mut crate::CreditEntityAnswer) {
    let P::SaveMoreCredits { value, .. } = request else { unreachable!() };
    let crate::CreditDocumentView::CreditBond { value: page } = &value.documents[bond] else {
        unreachable!()
    };
    let key = alloc::format!("credit/{}/origin", page.id);
    let index = value.answers.iter().position(|row| row.key == key).expect("an answered origin");
    let crate::CreditDocumentAnswer::BondOrigin {
        value:
            Some(
                crate::CreditOriginAnswer::EntityStatement(entity)
                | crate::CreditOriginAnswer::Entity1097(entity),
            ),
    } = &mut value.answers[index].answer
    else {
        panic!("an entity origin")
    };
    (index, entity)
}
