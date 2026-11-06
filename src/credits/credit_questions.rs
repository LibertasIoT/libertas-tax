// The More credits topic's strict document questions (D-2026-11-04-07): the
// Form 8986 prior-return facts and the Form 1097-BTC bond answers. Each row
// is a canonical DocumentAnswer with a required strict twin; the stored
// BbaAuditAnswers / TaxCreditBondAnswers keep their fields. A conditional row
// is asked only while its condition holds and pruned once it no longer does.
// The More credits page shows the recorded answers read-only, with one link
// to the questions page while any are open.
#[allow(unused_imports)]
use super::*;
use crate::document_followup_page::strict_answer;
use crate::{
    DocumentAnswer as A, DocumentQuestionsLink, DocumentStrictQuestion,
    DocumentStrictQuestionInitial,
};

pub(super) struct CreditRow {
    pub(super) key: String,
    prompt: Vec<u8>,
    pub(super) answer: A,
    owner: Option<crate::Owner>,
}

fn prompt(question: &str, document: &[u8]) -> Vec<u8> {
    Message::new(
        "DOCUMENT_TOPIC_SOURCE_ITEM",
        alloc::vec![MessageArg::Resource(question.into()), MessageArg::Encoded(document.to_vec())],
    )
    .encode()
}

// Form 8978 line 17 applies only to additional 2025 tax: asked once the other
// answers refigure to a higher Tax Table amount (bba_audit refigure).
fn line_17_asked(source: &crate::BbaAuditSource) -> bool {
    let mut answers = source.answers.clone();
    answers.line_17_interest = Some(0);
    answers.interest_calculation_prepared = Some(true);
    answers
        .model(source.id, &source.document)
        .and_then(|model| model.refigure())
        .is_ok_and(|adjustment| adjustment.additional_tax() > 0)
}

// Form 8912: a share per bond is needed only when several bonds have
// positive credit and line 12 is partly limited (tax_credit_bonds).
fn shares_asked(draft: &FederalDraft) -> bool {
    let Some(credits) = draft.credits.as_ref() else { return false };
    if crate::tax_credit_bonds::positive_bonds(&credits.tax_credit_bonds) < 2 {
        return false;
    }
    probe_line_12(draft).is_some_and(|line12| {
        crate::tax_credit_bonds::shares_needed(&credits.tax_credit_bonds, line12)
    })
}

// With the shares unanswered the estimate is refused; line 12 does not
// depend on them, so it is read from one bond holding the same totals.
fn probe_line_12(draft: &FederalDraft) -> Option<i64> {
    let mut probe = draft.clone();
    let bonds = &mut probe.credits.as_mut()?.tax_credit_bonds;
    let current = bonds.iter().map(|bond| bond.form.box1_credit.unwrap_or(0)).sum();
    let prior = bonds.iter().map(|bond| bond.answers.prior_carry_dollars).sum();
    bonds.truncate(1);
    bonds[0].form.box1_credit = Some(current);
    bonds[0].answers.prior_carry_dollars = prior;
    bonds[0].answers.allowed_credit_dollars = None;
    bonds[0].answers.allowed_credit_confirmed = Some(false);
    // Result lines are in cents; Form 8912 allocates whole dollars.
    estimate(&probe)
        .lines
        .iter()
        .find(|line| line.id == "tax_credit_bond_credit")
        .map(|line| line.amount / 100)
}

pub(super) fn rows(draft: &FederalDraft) -> Vec<CreditRow> {
    let Some(credits) = draft.credits.as_ref() else { return Vec::new() };
    let mut rows = Vec::new();
    let mut add = |id: i64, field: &str, question: &str, answer: A, owner| {
        let document = document_owners::document_name(draft, id).unwrap_or_default();
        rows.push(CreditRow {
            key: alloc::format!("credit/{id}/{field}"),
            prompt: prompt(question, &document),
            answer,
            owner,
        });
    };
    if let Some(source) = &credits.bba_audit_interest {
        let (id, a) = (source.id, &source.answers);
        add(
            id,
            "prior_wages",
            "CREDIT_QUESTION_BBA_PRIOR_WAGES",
            A::BbaPriorWages { value: a.prior_wages },
            None,
        );
        add(
            id,
            "prior_taxable_income",
            "CREDIT_QUESTION_BBA_PRIOR_TAXABLE_INCOME",
            A::BbaPriorTaxableIncome { value: a.prior_taxable_income },
            None,
        );
        add(
            id,
            "prior_income_tax",
            "CREDIT_QUESTION_BBA_PRIOR_INCOME_TAX",
            A::BbaPriorIncomeTax { value: a.prior_income_tax },
            None,
        );
        add(
            id,
            "prior_status",
            "CREDIT_QUESTION_BBA_PRIOR_STATUS",
            A::BbaPriorStatus { value: a.prior_status },
            None,
        );
        add(
            id,
            "single_audit_adjustment",
            "CREDIT_QUESTION_BBA_SINGLE_ADJUSTMENT",
            A::BbaSingleAdjustment { value: a.single_audit_adjustment },
            None,
        );
        add(
            id,
            "simple_prior_return",
            "CREDIT_QUESTION_BBA_SIMPLE_PRIOR_RETURN",
            A::BbaSimplePriorReturn { value: a.simple_prior_return },
            None,
        );
        add(
            id,
            "no_other_effects",
            "CREDIT_QUESTION_BBA_NO_OTHER_EFFECTS",
            A::BbaNoOtherEffects { value: a.no_other_effects },
            None,
        );
        if line_17_asked(source) {
            add(
                id,
                "line_17_interest",
                "CREDIT_QUESTION_BBA_LINE_17_INTEREST",
                A::BbaLine17Interest {
                    interest: a.line_17_interest,
                    prepared: a.interest_calculation_prepared,
                },
                None,
            );
        }
    }
    let shares = !credits.tax_credit_bonds.is_empty() && shares_asked(draft);
    for bond in &credits.tax_credit_bonds {
        let (id, a, owner) = (bond.id, &bond.answers, Some(bond.owner));
        add(id, "kind", "CREDIT_QUESTION_BOND_CLASS", A::BondClass { value: a.kind }, None);
        add(
            id,
            "issued_on",
            "CREDIT_QUESTION_BOND_ISSUE_DATE",
            A::BondIssueDate { value: a.issued_on },
            None,
        );
        add(
            id,
            "origin",
            "CREDIT_QUESTION_BOND_ORIGIN",
            A::BondOrigin { value: a.origin.clone() },
            owner,
        );
        add(
            id,
            "prior_carry",
            "CREDIT_QUESTION_BOND_PRIOR_CARRY",
            A::BondPriorCarry {
                dollars: a.prior_carry_dollars,
                confirmed: a.prior_carry_confirmed,
            },
            None,
        );
        if shares {
            add(
                id,
                "allowed_share",
                "CREDIT_QUESTION_BOND_ALLOWED_SHARE",
                A::BondAllowedShare {
                    dollars: a.allowed_credit_dollars,
                    confirmed: a.allowed_credit_confirmed,
                },
                None,
            );
        }
    }
    rows
}

fn source_choices(draft: &FederalDraft, owner: Option<crate::Owner>) -> Vec<RecordChoice> {
    let Some(owner) = owner else { return Vec::new() };
    crate::tax_credit_bonds::income_sources(draft)
        .into_iter()
        .filter(|source| source.owner == owner)
        .map(|source| RecordChoice { id: source.id, label: source.label })
        .collect()
}

// The answered questions, each as its ordinary editable canonical row on the
// More credits page.
pub(super) fn answered_rows(draft: &FederalDraft) -> Vec<crate::CreditDocumentQuestion> {
    rows(draft)
        .into_iter()
        .filter(|row| matches!(strict_answer(&row.answer), Some(Some(_))))
        .filter_map(|row| {
            let answer = crate::CreditDocumentAnswer::from_canonical(row.answer)?;
            Some(crate::CreditDocumentQuestion {
                allowed_answers: alloc::vec![answer.index()],
                credit_income_sources: source_choices(draft, row.owner),
                key: row.key,
                prompt: row.prompt,
                answer,
            })
        })
        .collect()
}

// The More credits save: each submitted answered row onto its record, with
// the bond source rules located on the row that broke them. A row cleared
// back to unanswered is asked again on the questions page.
pub(super) fn apply_answers(
    accepted: &FederalDraft,
    draft: &mut FederalDraft,
    answers: &[crate::CreditDocumentQuestion],
) -> Result<(), SaveFailure> {
    let expected = rows(accepted);
    for (index, row) in answers.iter().enumerate() {
        let fits = expected.iter().any(|expected| {
            expected.key == row.key
                && crate::CreditDocumentAnswer::from_canonical(expected.answer.clone())
                    .is_some_and(|answer| answer.index() == row.answer.index())
        });
        if !fits || !apply(draft, &row.key, row.answer.clone().into_canonical()) {
            return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                .at_field("answer")
                .in_list("answers", index)
                .into());
        }
    }
    let keys: Vec<&str> = answers.iter().map(|row| row.key.as_str()).collect();
    match origin_issue(draft, "answers", &keys) {
        Some(issue) => Err(issue.into()),
        None => Ok(()),
    }
}

// A bond source rule, located on the origin answer of `list` that broke it.
fn origin_issue(
    draft: &FederalDraft,
    name: &'static str,
    list: &[&str],
) -> Option<ValidationIssue> {
    let bonds = draft.credits.iter().flat_map(|c| &c.tax_credit_bonds);
    let (id, issue) = bonds
        .clone()
        .find_map(|bond| bond.issue().map(|issue| (Some(bond.id), issue)))
        .or_else(|| crate::tax_credit_bonds::source_document_issue(draft))?;
    let key = id.map(|id| alloc::format!("credit/{id}/origin"));
    let index = list.iter().position(|asked| Some(*asked) == key.as_deref());
    Some(match (index, issue.leaf()) {
        (Some(index), Some(leaf)) => ValidationIssue::new(issue.key)
            .at_field(leaf)
            .in_field("value")
            .in_field("answer")
            .in_list(name, index),
        _ => ValidationIssue::new(issue.key).at_field(name),
    })
}

// The questions still unanswered, as the questions page asks them.
pub(super) fn open_questions(draft: &FederalDraft) -> Vec<DocumentStrictQuestionInitial> {
    let sources = crate::tax_credit_bonds::income_sources(draft);
    rows(draft)
        .into_iter()
        .filter(|row| strict_answer(&row.answer) == Some(None))
        .map(|row| DocumentStrictQuestionInitial {
            allowed_answers: crate::DocumentStrictAnswer::kind_of(&row.answer)
                .into_iter()
                .collect(),
            allowed_values: Vec::new(),
            ccc_lots: Vec::new(),
            health_plans: Vec::new(),
            espp_acquisitions: Vec::new(),
            interest_statements: Vec::new(),
            people: Vec::new(),
            credit_income_sources: row.owner.map_or_else(Vec::new, |owner| {
                sources
                    .iter()
                    .filter(|source| source.owner == owner)
                    .map(|source| RecordChoice { id: source.id, label: source.label.clone() })
                    .collect()
            }),
            key: row.key,
            prompt: row.prompt,
        })
        .collect()
}

// One answer onto its stored record.
fn apply(draft: &mut FederalDraft, key: &str, answer: A) -> bool {
    let Some(credits) = draft.credits.as_mut() else { return false };
    let mut parts = key.split('/').skip(1);
    let (Some(id), Some(field)) =
        (parts.next().and_then(|id| id.parse::<i64>().ok()), parts.next())
    else {
        return false;
    };
    if let Some(source) = credits.bba_audit_interest.as_mut().filter(|source| source.id == id) {
        let a = &mut source.answers;
        match (field, answer) {
            ("prior_wages", A::BbaPriorWages { value }) => a.prior_wages = value,
            ("prior_taxable_income", A::BbaPriorTaxableIncome { value }) => {
                a.prior_taxable_income = value
            }
            ("prior_income_tax", A::BbaPriorIncomeTax { value }) => a.prior_income_tax = value,
            ("prior_status", A::BbaPriorStatus { value }) => a.prior_status = value,
            ("single_audit_adjustment", A::BbaSingleAdjustment { value }) => {
                a.single_audit_adjustment = value
            }
            ("simple_prior_return", A::BbaSimplePriorReturn { value }) => {
                a.simple_prior_return = value
            }
            ("no_other_effects", A::BbaNoOtherEffects { value }) => a.no_other_effects = value,
            ("line_17_interest", A::BbaLine17Interest { interest, prepared }) => {
                a.line_17_interest = interest;
                a.interest_calculation_prepared = prepared;
            }
            _ => return false,
        }
        return true;
    }
    let Some(bond) = credits.tax_credit_bonds.iter_mut().find(|bond| bond.id == id) else {
        return false;
    };
    let a = &mut bond.answers;
    match (field, answer) {
        ("kind", A::BondClass { value }) => a.kind = value,
        ("issued_on", A::BondIssueDate { value }) => a.issued_on = value,
        ("origin", A::BondOrigin { value }) => a.origin = value,
        ("prior_carry", A::BondPriorCarry { dollars, confirmed }) => {
            a.prior_carry_dollars = dollars;
            a.prior_carry_confirmed = confirmed;
        }
        ("allowed_share", A::BondAllowedShare { dollars, confirmed }) => {
            a.allowed_credit_dollars = dollars;
            a.allowed_credit_confirmed = confirmed;
        }
        _ => return false,
    }
    true
}

// Walk-owned prune: a conditional answer the current facts no longer ask for
// is dropped (line 17 without a tax increase; bond shares outside the partly
// limited multi-bond case).
pub(super) fn prune(draft: &mut FederalDraft) {
    let shares = shares_asked(draft);
    let Some(credits) = draft.credits.as_mut() else { return };
    if let Some(source) = credits.bba_audit_interest.as_mut()
        && !line_17_asked(source)
    {
        source.answers.line_17_interest = None;
        source.answers.interest_calculation_prepared = None;
    }
    if !shares {
        for bond in &mut credits.tax_credit_bonds {
            bond.answers.allowed_credit_dollars = None;
            if bond.answers.allowed_credit_confirmed == Some(true) {
                bond.answers.allowed_credit_confirmed = Some(false);
            }
        }
    }
}

impl Interview {
    // The More credits page's one link while any question is open.
    pub(super) fn credit_questions_link(&self) -> Vec<DocumentQuestionsLink> {
        let open = open_questions(&self.draft).len();
        if open == 0 {
            return Vec::new();
        }
        alloc::vec![DocumentQuestionsLink {
            topic: TaxSection::MoreCredits,
            status: status_count("CREDIT_DOCUMENT_QUESTIONS_OPEN", open),
        }]
    }

    // The link opens the questions page with the unanswered questions; once
    // none is open it reopens the More credits page.
    pub(super) fn open_document_questions(&self, topic: TaxSection, review: bool) -> Reply {
        if topic == TaxSection::Mortgage {
            return self.open_mortgage_questions(review);
        }
        if topic != TaxSection::MoreCredits || self.draft.credits.is_none() {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_HERE");
        }
        let questions = open_questions(&self.draft);
        if questions.is_empty() {
            return self.more_credits_page(review);
        }
        let d = &self.draft;
        Reply::Response(P::BeginDocumentQuestions {
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CREDITS_PAGE,
            previous: CREDITS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::MoreCredits).map(Into::into),
            progress: self.topic_progress(CREDITS_PAGE, Some(TaxSection::MoreCredits)),
            topic,
            questions,
            situations: alloc::vec![crate::DocumentSituationsAnswered {
                value: crate::DocumentSituations::NoneOfThese
            }],
            allowed_situations: Vec::new(),
            situation_answers: alloc::vec![0],
            situation_documents: Vec::new(),
            disaster_choices: Vec::new(),
            inherited_iras: Vec::new(),
            businesses: Vec::new(),
            people: Vec::new(),
            inherited_roth_iras: Vec::new(),
            wage_scorporations: Vec::new(),
        })
    }

    // The questions page saves only its answers, then reopens More credits.
    pub(super) fn save_credit_questions(
        &mut self,
        review: bool,
        questions: Vec<DocumentStrictQuestion>,
    ) -> Reply {
        let open = open_questions(&self.draft);
        let mut draft = self.draft.clone();
        let asked: Vec<String> = questions.iter().map(|q| q.key.clone()).collect();
        for (index, question) in questions.into_iter().enumerate() {
            let changed = || {
                refused(
                    ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                        .at_field("answer")
                        .in_list("questions", index),
                )
            };
            let fits = open.iter().any(|row| {
                row.key == question.key && row.allowed_answers.contains(&question.answer.index())
            });
            let Some(answer) = question.answer.into_canonical().filter(|_| fits) else {
                return changed();
            };
            if !apply(&mut draft, &question.key, answer) {
                return changed();
            }
        }
        prune(&mut draft);
        let keys: Vec<&str> = asked.iter().map(String::as_str).collect();
        if let Some(issue) = origin_issue(&draft, "questions", &keys) {
            return refused(issue);
        }
        match self.commit_issue(draft) {
            Ok(_) => self.more_credits_page(review),
            Err((issue, _, _)) => refused(issue),
        }
    }
}
