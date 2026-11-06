// D-2026-11-04-07: the Form 8986 and Form 1097-BTC answers are required strict
// questions on the More credits topic's questions page; the More credits page
// shows them read-only with one link while any are open.
use super::*;
use crate::DocumentStrictAnswer as T;
use crate::tests::support::client::credit_documents::{
    answer_credit_questions, open_credit_questions,
};
use crate::tests::support::client::own_questions::save_questions;

// The Smith walk's Form 8986: $20 more K-1 interest on a joint 2025 return.
fn smith_audit(wages: i64, taxable: i64, tax: i64) -> FederalBbaAuditInterest {
    FederalBbaAuditInterest {
        id: 77,
        interest_adjustment: 20_00,
        original_partnership_interest: 50_00,
        prior_status: FederalStatus::Joint,
        prior_wages: wages,
        prior_taxable_income: taxable,
        prior_income_tax: tax,
        line_17_interest: 1_00,
        interest_calculation_prepared: true,
        ..crate::tests::support::builders::bba_interest_reduction()
    }
}

// An accepted Form 8986 with nothing answered yet.
fn audit_session(audit: &FederalBbaAuditInterest) -> Interview {
    let mut draft = complete(FederalStatus::Joint);
    let mut source = crate::tests::support::builders::added_bba(audit.clone());
    source.id = audit.id;
    source.answers = crate::BbaAuditAnswers::default();
    draft.credits.as_mut().unwrap().bba_audit_interest = Some(source);
    Interview::new(TEST_USER, draft)
}

fn asked(reply: &Reply) -> Vec<alloc::string::String> {
    let Reply::Response(P::BeginDocumentQuestions { questions, topic, .. }) = reply else {
        panic!("the link opens the questions page: {reply:?}")
    };
    assert_eq!(*topic, TaxSection::MoreCredits);
    questions.iter().map(|q| q.key.rsplit('/').next().unwrap().into()).collect()
}

fn link_open(s: &mut Interview) -> usize {
    let Reply::Edit(P::SaveMoreCredits { document_questions, value, .. }) =
        crate::tests::cross::interview::from_review(s, TaxSection::MoreCredits)
    else {
        panic!("More credits opens by EditRequest")
    };
    assert!(!value.documents.is_empty(), "the papers stay on the page");
    document_questions.len()
}

fn answers(s: &Interview) -> crate::BbaAuditAnswers {
    s.draft.credits.as_ref().unwrap().bba_audit_interest.as_ref().unwrap().answers.clone()
}

const BBA_ROWS: [&str; 7] = [
    "prior_wages",
    "prior_taxable_income",
    "prior_income_tax",
    "prior_status",
    "single_audit_adjustment",
    "simple_prior_return",
    "no_other_effects",
];

#[test]
fn smith_facts_answer_the_8986_without_line_17_and_round_trip() {
    let audit = smith_audit(10_000_000, 6_855_000, 775_200);
    let mut s = audit_session(&audit);
    assert_eq!(link_open(&mut s), 1, "one link while questions are open");
    assert_eq!(asked(&open_credit_questions(&mut s)), BBA_ROWS);
    answer_credit_questions(&mut s, &[], Some(&audit));
    let a = answers(&s);
    assert_eq!(
        (a.prior_wages, a.prior_taxable_income, a.prior_income_tax, a.prior_status),
        (Some(10_000_000), Some(6_855_000), Some(775_200), Some(FederalStatus::Joint))
    );
    assert_eq!(
        (a.single_audit_adjustment, a.simple_prior_return, a.no_other_effects),
        (Some(true), Some(true), Some(true))
    );
    // The Tax Table row does not move: line 17 is never asked.
    assert_eq!((a.line_17_interest, a.interest_calculation_prepared), (None, None));
    assert_eq!(link_open(&mut s), 0, "no link once every question is answered");
    // Reopened, the link answers with the More credits page itself.
    assert!(matches!(open_credit_questions(&mut s), Reply::Edit(P::SaveMoreCredits { .. })));
    let restarted = restart(s.draft.clone());
    assert_eq!(answers(&restarted), a);
}

#[test]
fn each_row_blocks_until_answered_and_unknown_answers_are_refused() {
    let audit = smith_audit(10_000_000, 6_855_000, 775_200);
    let mut s = audit_session(&audit);
    let begin = open_credit_questions(&mut s);
    let P::BeginDocumentQuestions { questions, .. } = page(&begin) else { unreachable!() };
    // Answer all but one: that row stays open and keeps the link.
    assert_eq!(questions.len(), BBA_ROWS.len());
    for (skipped, row) in BBA_ROWS.iter().enumerate() {
        let mut t = audit_session(&audit);
        let begin = open_credit_questions(&mut t);
        let P::BeginDocumentQuestions { questions, .. } = page(&begin) else { unreachable!() };
        let answered = questions
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != skipped)
            .map(|(_, q)| {
                strict(
                    &q.key,
                    crate::tests::support::client::credit_documents::record_answer(
                        &q.key,
                        &[],
                        Some(&audit),
                    )
                    .unwrap(),
                )
            })
            .collect();
        assert!(matches!(
            save_questions(&mut t, &page(&begin), answered),
            Reply::Edit(P::SaveMoreCredits { .. })
        ));
        assert_eq!(asked(&open_credit_questions(&mut t)), [*row]);
        assert_eq!(link_open(&mut t), 1);
        let source = t.draft.credits.as_ref().unwrap().bba_audit_interest.clone().unwrap();
        assert!(
            source.answers.model(source.id, &source.document).is_err(),
            "the unanswered row still blocks the Form 8978 refigure"
        );
    }
    // An answer of the wrong kind is refused and saves nothing.
    let wrong = vec![strict(&questions[0].key, T::BbaSingleAdjustment { value: true })];
    assert!(matches!(
        save_questions(&mut s, &page(&begin), wrong),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(answers(&s), crate::BbaAuditAnswers::default());
}

#[test]
fn line_17_is_asked_only_when_the_refigured_tax_rises_and_pruned_after() {
    let audit = smith_audit(12_003_000, 8_858_000, 1_015_200);
    let mut s = audit_session(&audit);
    let first = open_credit_questions(&mut s);
    assert_eq!(asked(&first), BBA_ROWS);
    let P::BeginDocumentQuestions { questions, .. } = page(&first) else { unreachable!() };
    let rest = questions
        .iter()
        .map(|q| {
            strict(
                &q.key,
                crate::tests::support::client::credit_documents::record_answer(
                    &q.key,
                    &[],
                    Some(&audit),
                )
                .unwrap(),
            )
        })
        .collect();
    save_questions(&mut s, &page(&first), rest);
    // 88,580 moves the Tax Table row: only line 17 is open now.
    assert_eq!(link_open(&mut s), 1);
    let second = open_credit_questions(&mut s);
    assert_eq!(asked(&second), ["line_17_interest"]);
    let P::BeginDocumentQuestions { questions, .. } = page(&second) else { unreachable!() };
    let zero =
        vec![strict(&questions[0].key, T::BbaLine17Interest { interest: 0, prepared: true })];
    save_questions(&mut s, &page(&second), zero);
    assert_eq!(
        (answers(&s).line_17_interest, answers(&s).interest_calculation_prepared),
        (Some(0), Some(true))
    );
    assert_eq!(link_open(&mut s), 0);
    // The facts change to the Smith row: the More credits save drops line 17.
    let a = &mut s.draft.credits.as_mut().unwrap().bba_audit_interest.as_mut().unwrap().answers;
    (a.prior_wages, a.prior_taxable_income, a.prior_income_tax) =
        (Some(10_000_000), Some(6_855_000), Some(775_200));
    let Reply::Edit(request) =
        crate::tests::cross::interview::from_review(&mut s, TaxSection::MoreCredits)
    else {
        unreachable!()
    };
    crate::tests::cross::interview::accepted(&send(&mut s, request));
    assert_eq!(
        (answers(&s).line_17_interest, answers(&s).interest_calculation_prepared),
        (None, None)
    );
}

fn bond(id: i64, identifier: &str) -> FederalTaxCreditBond1097 {
    let mut bond = crate::tests::support::builders::direct_tax_credit_bond(
        identifier,
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        1_000_000,
    );
    bond.id = id;
    bond
}

fn bond_session(bonds: &[FederalTaxCreditBond1097]) -> Interview {
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = bonds
        .iter()
        .map(|bond| FederalTaxCreditBond1097 {
            answers: crate::TaxCreditBondAnswers { prior_carry_dollars: 0, ..Default::default() },
            ..bond.clone()
        })
        .collect();
    Interview::new(TEST_USER, draft)
}

#[test]
fn bond_rows_round_trip_and_the_share_is_asked_only_for_partly_limited_bonds() {
    // One bond: four rows, never a share.
    let one = [bond(11, "PARTIAL001")];
    let mut s = bond_session(&one);
    assert_eq!(
        asked(&open_credit_questions(&mut s)),
        ["kind", "issued_on", "origin", "prior_carry"]
    );
    answer_credit_questions(&mut s, &one, None);
    assert_eq!(
        s.draft.credits.as_ref().unwrap().tax_credit_bonds[0].answers,
        crate::TaxCreditBondAnswers { allowed_credit_confirmed: None, ..one[0].answers.clone() }
    );
    assert_eq!(link_open(&mut s), 0);
    // Two bonds over the tax limit: each bond's share opens after its rows.
    let mut two = [bond(11, "PARTIAL001"), bond(12, "PARTIAL002")];
    let mut s = bond_session(&two);
    assert_eq!(asked(&open_credit_questions(&mut s)).len(), 8);
    answer_credit_questions(&mut s, &two, None);
    let shares = open_credit_questions(&mut s);
    assert_eq!(asked(&shares), ["allowed_share", "allowed_share"]);
    let line12 = value(
        &estimate(&crate::tests::support::builders::single_bond_with_same_total_credit(&s.draft)),
        "tax_credit_bond_credit",
    );
    assert!(line12 > 0 && line12 < 20_000, "partly limited: {line12}");
    two[0].answers.allowed_credit_dollars = Some(line12 / 2);
    two[1].answers.allowed_credit_dollars = Some(line12 - line12 / 2);
    for bond in &mut two {
        bond.answers.allowed_credit_confirmed = Some(true);
    }
    answer_credit_questions(&mut s, &two, None);
    let stored = s.draft.credits.as_ref().unwrap().tax_credit_bonds.clone();
    assert_eq!(stored[0].answers.allowed_credit_dollars, Some(line12 / 2));
    assert_eq!(link_open(&mut s), 0);
    assert_eq!(restart(s.draft.clone()).draft.credits.unwrap().tax_credit_bonds, stored);
}

fn strict(key: &str, answer: T) -> crate::DocumentStrictQuestion {
    crate::DocumentStrictQuestion {
        key: key.into(),
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
    }
}

fn page(reply: &Reply) -> P {
    let Reply::Response(page) = reply else { panic!("a questions page: {reply:?}") };
    page.clone()
}

// An answered fact is corrected on More credits as its canonical row; a row
// cleared there is asked again on the questions page.
#[test]
fn answered_8986_facts_are_corrected_on_more_credits() {
    let audit = smith_audit(10_000_000, 6_855_000, 775_200);
    let mut s = audit_session(&audit);
    answer_credit_questions(&mut s, &[], Some(&audit));
    let edit = |s: &mut Interview, value: Option<i64>| {
        let Reply::Edit(mut request @ P::SaveMoreCredits { .. }) =
            crate::tests::cross::interview::from_review(s, TaxSection::MoreCredits)
        else {
            unreachable!()
        };
        let P::SaveMoreCredits { value: page, .. } = &mut request else { unreachable!() };
        assert_eq!(page.answers.len(), BBA_ROWS.len(), "every answered row is on the page");
        let row = page.answers.iter_mut().find(|row| row.key.ends_with("/prior_wages")).unwrap();
        row.answer = crate::CreditDocumentAnswer::BbaPriorWages { value };
        send(s, request)
    };
    crate::tests::cross::interview::accepted(&edit(&mut s, Some(10_145_000)));
    assert_eq!(answers(&s).prior_wages, Some(10_145_000));
    crate::tests::cross::interview::accepted(&edit(&mut s, None));
    assert_eq!(answers(&s).prior_wages, None);
    assert_eq!(asked(&open_credit_questions(&mut s)), ["prior_wages"]);
}
