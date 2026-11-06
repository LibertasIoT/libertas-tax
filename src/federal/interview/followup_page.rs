// The income topic page as the person sees it. The interview keeps one
// canonical list of document questions (`followups`); at the protocol boundary
// this adapter regroups it (Tax AGENTS.md section 10, shape
// followup-questions-shape-2026-10-31):
// - a question the paper settles is not asked; its one answer is saved;
// - a question a printed box calls for is a required answer per paper
//   (`questions`), starting unanswered through BeginIncomes;
// - rare situations are asked once per page (`situations`, "None of these"),
//   again whenever a document on the page has not been covered by a saved answer;
// - every other row keeps its place in `followups`.
use super::*;
use crate::document_followup_page::{
    apply_wage_question, apply_wage_situations, rare_answer, rare_chosen, rare_situations,
    settled_answer, strict_answer, wage_chosen, wage_questions, wage_situations,
};
use crate::{
    DocumentAnswer as A, DocumentQuestion, DocumentSituation, DocumentSituations,
    DocumentSituationsAnswered, DocumentStrictAnswer, DocumentStrictQuestion,
    DocumentStrictQuestionInitial,
};
use libertas::SchemaPath;

pub(super) enum Kind {
    Settled(alloc::boxed::Box<A>),
    Strict(bool),
    Rare,
    // A W-2's row holds its own questions and situations (wage_rows).
    Wage,
    Other,
}

pub(super) fn kind(row: &DocumentQuestion) -> Kind {
    // A child's statement inside Form 8814 stays with its election's rows.
    if row.key.contains("/statement/") {
        return Kind::Other;
    }
    if let Some(answer) = settled_answer(&row.answer, &row.allowed_values) {
        return Kind::Settled(alloc::boxed::Box::new(answer));
    }
    if let Some(answer) = strict_answer(&row.answer) {
        return Kind::Strict(answer.is_some());
    }
    if !rare_situations(&row.answer).is_empty() {
        return Kind::Rare;
    }
    if matches!(row.answer, A::Wage { .. }) {
        return Kind::Wage;
    }
    Kind::Other
}

fn has_situations(row: &DocumentQuestion) -> bool {
    matches!(kind(row), Kind::Rare | Kind::Wage)
}

fn row_id(key: &str) -> Option<i64> {
    key.split('/').nth(1)?.parse().ok()
}

// A topic still asks its situations question for a document no saved answer
// has covered. (Box questions keep their own pending rules.)
pub(super) fn questions_open(draft: &FederalDraft, rows: &[(i64, DocumentQuestion)]) -> bool {
    rows.iter().any(|(id, row)| has_situations(row) && draft.situations_unanswered.contains(id))
}

pub(super) fn situations_open(draft: &FederalDraft, topic: TaxSection) -> bool {
    Interview::document_followup_rows(draft, Some(topic), None)
        .iter()
        .any(|(id, row)| has_situations(row) && draft.situations_unanswered.contains(id))
}

// Whether an income topic still asks its situations question.
#[cfg(test)]
pub(crate) fn situations_pending(s: &Interview) -> bool {
    INCOME_TOPICS.iter().any(|topic| situations_open(&s.draft, *topic))
}

// An accepted topic Save answered the situations question for every document
// on the page.
pub(super) fn answer_situations(draft: &mut FederalDraft, topic: TaxSection) {
    let covered: Vec<i64> = Interview::document_followup_rows(draft, Some(topic), None)
        .into_iter()
        .filter(|(_, row)| has_situations(row))
        .map(|(id, _)| id)
        .collect();
    let income: Vec<i64> = draft.income.iter().map(|entry| entry.id).collect();
    draft.situations_unanswered.retain(|id| !covered.contains(id) && income.contains(id));
}

// An accepted Document center Save: each new paper that can have a rare
// situation waits for its topic page's answer.
pub(super) fn note_new_documents(draft: &mut FederalDraft, before: &FederalDraft) {
    let new: Vec<i64> = Interview::document_followup_rows(draft, None, None)
        .into_iter()
        .filter(|(id, row)| {
            has_situations(row) && !before.income.iter().any(|entry| entry.id == *id)
        })
        .map(|(id, _)| id)
        .collect();
    for id in new {
        if !draft.situations_unanswered.contains(&id) {
            draft.situations_unanswered.push(id);
        }
    }
}

struct Page {
    followups: Vec<DocumentQuestion>,
    questions: Vec<DocumentStrictQuestion>,
    new_questions: Vec<DocumentStrictQuestionInitial>,
    situations: DocumentSituations,
    situations_open: bool,
    allowed_situations: Vec<i32>,
    situation_documents: Vec<RecordChoice>,
}

// Only the 1099-A acquisition question names a forfeited CCC loan.
fn question_ccc_lots(draft: &FederalDraft, answer: &crate::DocumentAnswer) -> Vec<RecordChoice> {
    if matches!(answer, crate::DocumentAnswer::DebtAcquisition { .. }) {
        super::ccc_lots(draft)
    } else {
        Vec::new()
    }
}

// Only the savings-bond records question names an interest statement.
fn question_interest_statements(
    draft: &FederalDraft,
    answer: &crate::DocumentAnswer,
) -> Vec<RecordChoice> {
    if matches!(answer, crate::DocumentAnswer::SavingsBondsRecords { .. }) {
        crate::savings_bonds::source_choices(&draft.income)
    } else {
        Vec::new()
    }
}

// ... and its students are chosen from the education catalog.
fn question_people(
    draft: &FederalDraft,
    answer: &crate::DocumentAnswer,
) -> Vec<crate::PersonChoice> {
    if matches!(answer, crate::DocumentAnswer::SavingsBondsRecords { .. }) {
        education_people(draft)
    } else {
        Vec::new()
    }
}

fn split(draft: &FederalDraft, rows: Vec<DocumentQuestion>) -> Page {
    let mut page = Page {
        followups: Vec::new(),
        questions: Vec::new(),
        new_questions: Vec::new(),
        situations: DocumentSituations::NoneOfThese,
        situations_open: false,
        allowed_situations: Vec::new(),
        situation_documents: Vec::new(),
    };
    let mut chosen = Vec::new();
    for row in rows {
        match kind(&row) {
            Kind::Settled(_) => {}
            Kind::Strict(true) => {
                let answer = strict_answer(&row.answer).flatten().expect("an answered question");
                page.questions.push(DocumentStrictQuestion {
                    key: row.key,
                    prompt: row.prompt,
                    allowed_answers: alloc::vec![answer.index()],
                    ccc_lots: question_ccc_lots(draft, &row.answer),
                    health_plans: row.health_plans,
                    espp_acquisitions: row.espp_acquisitions,
                    interest_statements: question_interest_statements(draft, &row.answer),
                    people: question_people(draft, &row.answer),
                    credit_income_sources: Vec::new(),
                    allowed_values: row.allowed_values,
                    answer,
                });
            }
            Kind::Strict(false) => page.new_questions.push(DocumentStrictQuestionInitial {
                ccc_lots: question_ccc_lots(draft, &row.answer),
                health_plans: row.health_plans,
                espp_acquisitions: row.espp_acquisitions,
                interest_statements: question_interest_statements(draft, &row.answer),
                people: question_people(draft, &row.answer),
                credit_income_sources: Vec::new(),
                allowed_answers: DocumentStrictAnswer::kind_of(&row.answer).into_iter().collect(),
                key: row.key,
                prompt: row.prompt,
                allowed_values: row.allowed_values,
            }),
            Kind::Rare => {
                let Some(id) = row_id(&row.key) else { continue };
                page.situations_open |= draft.situations_unanswered.contains(&id);
                for situation in rare_situations(&row.answer) {
                    if !page.allowed_situations.contains(&situation) {
                        page.allowed_situations.push(situation);
                    }
                }
                if !page.situation_documents.iter().any(|choice| choice.id == id)
                    && let Some(entry) = draft.income.iter().find(|entry| entry.id == id)
                {
                    page.situation_documents.push(RecordChoice {
                        id,
                        label: document_followups::followup_document_name(draft, entry),
                    });
                }
                if let Some(situation) = rare_chosen(&row.answer) {
                    chosen.push(DocumentSituation { situation, document: id });
                }
            }
            Kind::Wage | Kind::Other => page.followups.push(row),
        }
    }
    page.allowed_situations.sort_unstable();
    if !chosen.is_empty() {
        page.situations = DocumentSituations::These { situations: chosen };
    }
    page
}

// Choosing the disaster situation is the "yes": every DisasterDistribution
// member after `No` (Qualified, HomePurchase, NotDeclared).
fn situation_disaster_choices() -> Vec<i32> {
    alloc::vec![1, 2, 3]
}

// The W-2 rows of the wages page, regrouped like the income topic pages.
struct WagePage {
    questions: Vec<DocumentStrictQuestion>,
    new_questions: Vec<DocumentStrictQuestionInitial>,
    situations: DocumentSituations,
    situations_open: bool,
    allowed_situations: Vec<i32>,
    situation_documents: Vec<RecordChoice>,
}

fn split_wages(draft: &FederalDraft, rows: &[crate::W2AnswerRow], scorporations: bool) -> WagePage {
    let mut page = WagePage {
        questions: Vec::new(),
        new_questions: Vec::new(),
        situations: DocumentSituations::NoneOfThese,
        situations_open: false,
        allowed_situations: Vec::new(),
        situation_documents: Vec::new(),
    };
    let mut chosen = Vec::new();
    for row in rows {
        let Some(id) = row_id(&row.key) else { continue };
        for (part, answer) in wage_questions(&row.answers) {
            let key = alloc::format!("{}/{part}", row.key);
            match answer {
                Some(answer) => page.questions.push(DocumentStrictQuestion {
                    key,
                    prompt: row.document.clone(),
                    allowed_answers: alloc::vec![answer.index()],
                    allowed_values: Vec::new(),
                    ccc_lots: Vec::new(),
                    health_plans: Vec::new(),
                    espp_acquisitions: Vec::new(),
                    interest_statements: Vec::new(),
                    people: Vec::new(),
                    credit_income_sources: Vec::new(),
                    answer,
                }),
                None => {
                    use libertas_macros::variant_index as v;
                    let kind = if part == "overtime" {
                        v!(DocumentStrictAnswer::WageOvertime)
                    } else if part.starts_with("box14/") {
                        v!(DocumentStrictAnswer::WageBox14)
                    } else {
                        v!(DocumentStrictAnswer::WageTipOccupation)
                    };
                    page.new_questions.push(DocumentStrictQuestionInitial {
                        key,
                        prompt: row.document.clone(),
                        allowed_answers: alloc::vec![kind as i32],
                        allowed_values: Vec::new(),
                        ccc_lots: Vec::new(),
                        health_plans: Vec::new(),
                        espp_acquisitions: Vec::new(),
                        interest_statements: Vec::new(),
                        people: Vec::new(),
                        credit_income_sources: Vec::new(),
                    });
                }
            }
        }
        page.situations_open |= draft.situations_unanswered.contains(&id);
        for situation in wage_situations(&row.answers, scorporations) {
            if !page.allowed_situations.contains(&situation) {
                page.allowed_situations.push(situation);
            }
        }
        page.situation_documents.push(RecordChoice { id, label: row.document.clone() });
        chosen.extend(
            wage_chosen(&row.answers)
                .into_iter()
                .map(|situation| DocumentSituation { situation, document: id }),
        );
    }
    page.allowed_situations.sort_unstable();
    if !chosen.is_empty() {
        page.situations = DocumentSituations::These { situations: chosen };
    }
    page
}

// The W-2 rows of a submitted wages page: every saved answer, with the
// page's questions and situations applied. A caller that already sends the
// rows keeps them.
fn join_wages(
    draft: &FederalDraft,
    supplied: Vec<crate::W2AnswerRow>,
    questions: Vec<DocumentStrictQuestion>,
    situations: DocumentSituations,
    scorporations: bool,
    require_answers: bool,
) -> Result<Vec<crate::W2AnswerRow>, alloc::boxed::Box<Reply>> {
    if !supplied.is_empty() {
        return Ok(supplied);
    }
    let situations = match situations {
        DocumentSituations::NoneOfThese => Vec::new(),
        DocumentSituations::These { situations } => situations,
    };
    let mut used = alloc::vec![false; situations.len()];
    let mut questions: Vec<Option<DocumentStrictQuestion>> =
        questions.into_iter().map(Some).collect();
    let mut rows = Vec::new();
    for (id, row) in Interview::document_followup_rows(draft, Some(TaxSection::Wages), None) {
        let A::Wage { mut answers } = row.answer else { continue };
        for (part, stored) in wage_questions(&answers.clone()) {
            let key = alloc::format!("{}/{part}", row.key);
            let supplied = questions
                .iter_mut()
                .find(|question| question.as_ref().is_some_and(|q| q.key == key))
                .and_then(Option::take);
            match supplied {
                Some(question) => {
                    if !apply_wage_question(&mut answers, &part, question.answer) {
                        return Err(page_failure(
                            SchemaPath::root().field("questions").key(key.as_str()),
                            "DOCUMENT_QUESTIONS_CHANGED",
                        ));
                    }
                }
                None if stored.is_some() || !require_answers => {}
                None => {
                    return Err(page_failure(
                        SchemaPath::root().field("questions"),
                        "DOCUMENT_QUESTIONS_CHANGED",
                    ));
                }
            }
        }
        let offered = wage_situations(&answers, scorporations);
        let mut chosen = Vec::new();
        for (index, entry) in situations.iter().enumerate() {
            if entry.document != id || !offered.contains(&entry.situation.index()) {
                continue;
            }
            if chosen
                .iter()
                .any(|old: &&crate::DocumentSituationAnswer| old.index() == entry.situation.index())
            {
                return Err(page_failure(
                    SchemaPath::root().field("situations").index(index),
                    "DOCUMENT_SITUATION_TWICE",
                ));
            }
            used[index] = true;
            chosen.push(&entry.situation);
        }
        apply_wage_situations(&mut answers, &offered, &chosen);
        let document = document_owners::document_name(draft, id).unwrap_or_default();
        rows.push(crate::W2AnswerRow { key: row.key, document, answers });
    }
    if let Some(index) = questions.iter().position(Option::is_some) {
        return Err(page_failure(
            SchemaPath::root().field("questions").index(index),
            "DOCUMENT_QUESTIONS_CHANGED",
        ));
    }
    if let Some(index) = used.iter().position(|used| !used) {
        return Err(page_failure(
            SchemaPath::root().field("situations").index(index).field("document"),
            "DOCUMENT_SITUATION_NOT_ON_DOCUMENT",
        ));
    }
    Ok(rows)
}

// The education, medical-account and injured-spouse pages keep their own
// question lists. The questions below are must-answer facts: each is a
// required strict question, never an option meaning "not answered yet".
pub(crate) trait OwnQuestion: Sized {
    type Asked: crate::OwnAsked;
    fn parts(&self) -> (&str, &[u8], &[i32]);
    fn canonical(&self) -> A;
    fn answered(key: String, answer: A) -> Option<Self>;
    fn set(&mut self, answer: A);
}

macro_rules! own_question {
    ($row:ty, $answer:ty, $asked:ty) => {
        impl OwnQuestion for $row {
            type Asked = $asked;
            fn parts(&self) -> (&str, &[u8], &[i32]) {
                (&self.key, &self.prompt, &self.allowed_values)
            }
            fn canonical(&self) -> A {
                self.answer.clone().into_canonical()
            }
            fn answered(key: String, answer: A) -> Option<Self> {
                Some(Self {
                    key,
                    prompt: Vec::new(),
                    answer: <$answer>::from_canonical(answer)?,
                    allowed_answers: Vec::new(),
                    allowed_values: Vec::new(),
                })
            }
            fn set(&mut self, answer: A) {
                if let Some(answer) = <$answer>::from_canonical(answer) {
                    self.answer = answer;
                }
            }
        }
    };
}
own_question!(
    crate::document_center::EducationDocumentQuestion,
    crate::EducationDocumentAnswer,
    crate::EducationOwnQuestion
);
own_question!(crate::HsaDocumentQuestion, crate::HsaDocumentAnswer, crate::HsaOwnQuestion);
own_question!(
    crate::InjuredSpouseDocumentQuestion,
    crate::InjuredSpouseDocumentAnswer,
    crate::InjuredOwnQuestion
);

fn own_strict(answer: &A) -> Option<Option<DocumentStrictAnswer>> {
    matches!(
        answer,
        A::EducationBeneficiary { .. }
            | A::MedicalPayment { .. }
            | A::MedicareSpousal { .. }
            | A::MedicarePrior { .. }
            | A::Management { .. }
    )
    .then(|| strict_answer(answer))
    .flatten()
}

pub(crate) const OWN_TOPICS: [TaxSection; 4] =
    [TaxSection::Education, TaxSection::Hsa, TaxSection::InjuredSpouse, TaxSection::Mortgage];

// The answered own questions leave the list as required questions; the
// unanswered ones are asked first on their own page.
fn split_own<R: OwnQuestion>(
    rows: Vec<R>,
) -> (Vec<R>, Vec<R::Asked>, Vec<DocumentStrictQuestionInitial>) {
    let (mut rest, mut asked, mut open) = (Vec::new(), Vec::new(), Vec::new());
    for row in rows {
        let canonical = row.canonical();
        let Some(answer) = own_strict(&canonical) else {
            rest.push(row);
            continue;
        };
        let (key, prompt, allowed_values) = row.parts();
        let (key, prompt, allowed_values) = (key.into(), prompt.to_vec(), allowed_values.to_vec());
        match answer {
            Some(answer) => asked.push(
                <R::Asked as crate::OwnAsked>::make(key, prompt, allowed_values, answer)
                    .expect("an own-page answer"),
            ),
            None => open.push(DocumentStrictQuestionInitial {
                key,
                prompt,
                allowed_answers: DocumentStrictAnswer::kind_of(&canonical).into_iter().collect(),
                allowed_values,
                ccc_lots: Vec::new(),
                health_plans: Vec::new(),
                espp_acquisitions: Vec::new(),
                interest_statements: Vec::new(),
                people: Vec::new(),
                credit_income_sources: Vec::new(),
            }),
        }
    }
    (rest, asked, open)
}

// The submitted required questions rejoin the page's list.
fn join_own<R: OwnQuestion>(
    followups: &mut Vec<R>,
    questions: Vec<R::Asked>,
) -> Result<Vec<(String, Target)>, alloc::boxed::Box<Reply>> {
    let mut targets = Vec::new();
    for (index, question) in questions.into_iter().enumerate() {
        let (key, answer) = crate::OwnAsked::parts(question);
        let row = answer.into_canonical().and_then(|answer| R::answered(key.clone(), answer));
        let Some(row) = row else {
            return Err(page_failure(
                SchemaPath::root().field("questions").index(index),
                "DOCUMENT_QUESTIONS_CHANGED",
            ));
        };
        targets.push((key, Target::Strict));
        followups.push(row);
    }
    Ok(targets)
}

// The questions page's answers for the page's unanswered own questions.
fn fill_own<R: OwnQuestion>(
    rows: &mut [R],
    questions: Vec<DocumentStrictQuestion>,
) -> Result<(), alloc::boxed::Box<Reply>> {
    let mut questions: Vec<Option<DocumentStrictQuestion>> =
        questions.into_iter().map(Some).collect();
    for row in rows.iter_mut() {
        if own_strict(&row.canonical()) != Some(None) {
            continue;
        }
        let key = String::from(row.parts().0);
        let supplied = questions
            .iter_mut()
            .find(|question| question.as_ref().is_some_and(|q| q.key == key))
            .and_then(Option::take)
            .and_then(|question| question.answer.into_canonical());
        let Some(answer) = supplied else {
            return Err(page_failure(
                SchemaPath::root().field("questions"),
                "DOCUMENT_QUESTIONS_CHANGED",
            ));
        };
        row.set(answer);
    }
    if let Some(index) = questions.iter().position(Option::is_some) {
        return Err(page_failure(
            SchemaPath::root().field("questions").index(index),
            "DOCUMENT_QUESTIONS_CHANGED",
        ));
    }
    Ok(())
}

// The questions page for a topic whose page keeps its own question list: the
// page's context and the questions still unanswered.
fn own_questions_page(
    page: P,
    topic: TaxSection,
    questions: Vec<DocumentStrictQuestionInitial>,
) -> P {
    let (cookie, revision, page, previous, review, estimate, progress, people) = match page {
        P::SaveHsa { cookie, revision, page, previous, review, estimate, progress, .. } => (
            cookie,
            revision,
            page,
            previous,
            review,
            estimate.map(Into::into),
            progress,
            Vec::new(),
        ),
        P::SaveEducation {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            people,
            ..
        } => (cookie, revision, page, previous, review, estimate.map(Into::into), progress, people),
        P::BeginInjuredSpouse {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            ..
        } => (cookie, revision, page, previous, review, estimate, progress, Vec::new()),
        P::BeginMortgages {
            cookie, revision, page, previous, review, estimate, progress, ..
        } => (cookie, revision, page, previous, review, estimate, progress, Vec::new()),
        P::SaveMortgages {
            cookie, revision, page, previous, review, estimate, progress, ..
        } => (
            cookie,
            revision,
            page,
            previous,
            review,
            estimate.map(Into::into),
            progress,
            Vec::new(),
        ),
        _ => unreachable!("only a page with its own question list asks them first"),
    };
    P::BeginDocumentQuestions {
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        progress,
        topic,
        questions,
        situations: alloc::vec![DocumentSituationsAnswered {
            value: DocumentSituations::NoneOfThese
        }],
        allowed_situations: Vec::new(),
        situation_answers: alloc::vec![0],
        situation_documents: Vec::new(),
        disaster_choices: Vec::new(),
        inherited_iras: Vec::new(),
        businesses: Vec::new(),
        people,
        inherited_roth_iras: Vec::new(),
        wage_scorporations: Vec::new(),
    }
}

// Where an answer the canonical rows hold was given on the page.
enum Target {
    Settled,
    Strict,
    Situation(Option<usize>),
}

pub(crate) struct WireTargets {
    rows: Vec<(String, Target)>,
    // The wages page's W-2 rows, whose own questions moved to `questions`.
    wages: Vec<String>,
    // The questions page saved through its topic page: the reply reopens that
    // page, and a refusal elsewhere on the topic has no field on this page.
    questions_page: Option<(TaxSection, bool)>,
}

// A refusal located on the page; boxed, as refusals are rare.
fn page_failure(field: SchemaPath, key: &'static str) -> alloc::boxed::Box<Reply> {
    alloc::boxed::Box::new(Reply::Response(P::Problem {
        message: libertas::libertas_formatted_text(key, &[]),
        field: Some(field.into()),
    }))
}

// The canonical rows of a submitted page. Rows the page did not regroup, or a
// caller that already sends canonical rows, keep their own answers.
fn join(
    draft: &FederalDraft,
    topic: TaxSection,
    followups: &mut Vec<DocumentQuestion>,
    questions: Vec<DocumentStrictQuestion>,
    situations: DocumentSituations,
    require_answers: bool,
) -> Result<Vec<(String, Target)>, alloc::boxed::Box<Reply>> {
    let expected = Interview::document_followup_rows(draft, Some(topic), None);
    let situations = match situations {
        DocumentSituations::NoneOfThese => Vec::new(),
        DocumentSituations::These { situations } => situations,
    };
    let mut used = alloc::vec![false; situations.len()];
    let mut targets = Vec::new();
    for (index, question) in questions.iter().enumerate() {
        let fits = expected.iter().any(|(_, row)| {
            row.key == question.key
                && DocumentStrictAnswer::kind_of(&row.answer) == Some(question.answer.index())
        });
        if !fits {
            return Err(page_failure(
                SchemaPath::root().field("questions").index(index),
                "DOCUMENT_QUESTIONS_CHANGED",
            ));
        }
    }
    let mut questions: Vec<Option<DocumentStrictQuestion>> =
        questions.into_iter().map(Some).collect();
    for (id, row) in expected {
        if followups.iter().any(|supplied| supplied.key == row.key) {
            continue;
        }
        let answer = match kind(&row) {
            Kind::Wage | Kind::Other => continue,
            Kind::Settled(answer) => {
                targets.push((row.key.clone(), Target::Settled));
                *answer
            }
            Kind::Strict(answered) => {
                let supplied = questions
                    .iter_mut()
                    .find(|question| question.as_ref().is_some_and(|q| q.key == row.key))
                    .and_then(Option::take);
                match supplied {
                    Some(question) => {
                        let Some(answer) = question.answer.into_canonical() else {
                            return Err(page_failure(
                                SchemaPath::root().field("questions"),
                                "DOCUMENT_QUESTIONS_CHANGED",
                            ));
                        };
                        targets.push((row.key.clone(), Target::Strict));
                        answer
                    }
                    // An answered question the page did not repeat keeps its answer.
                    None if answered || !require_answers => row.answer.clone(),
                    None => {
                        return Err(page_failure(
                            SchemaPath::root().field("questions"),
                            "DOCUMENT_QUESTIONS_CHANGED",
                        ));
                    }
                }
            }
            Kind::Rare => {
                let kinds = rare_situations(&row.answer);
                let mut found = None;
                for (index, entry) in situations.iter().enumerate() {
                    if entry.document == id && kinds.contains(&entry.situation.index()) {
                        if found.is_some() {
                            return Err(page_failure(
                                SchemaPath::root().field("situations").index(index),
                                "DOCUMENT_SITUATION_TWICE",
                            ));
                        }
                        found = Some(index);
                    }
                }
                let chosen = found.map(|index| &situations[index].situation);
                let Some(answer) = rare_answer(&row.answer, chosen) else {
                    continue;
                };
                if let Some(index) = found {
                    used[index] = true;
                }
                targets.push((row.key.clone(), Target::Situation(found)));
                answer
            }
        };
        followups.push(DocumentQuestion { answer, ..row });
    }
    if let Some(index) = used.iter().position(|used| !used) {
        return Err(page_failure(
            SchemaPath::root().field("situations").index(index).field("document"),
            "DOCUMENT_SITUATION_NOT_ON_DOCUMENT",
        ));
    }
    Ok(targets)
}

impl WireTargets {
    // A refusal located on a canonical row moves to the page field that
    // holds its answer.
    fn locate(&self, field: String) -> Option<String> {
        let answer: String = SchemaPath::root().field("answer").into();
        let answer = answer.trim_start_matches('$');
        for (key, target) in &self.rows {
            let row: String = SchemaPath::root().field("followups").key(key.as_str()).into();
            let Some(rest) = field.strip_prefix(row.as_str()) else { continue };
            return match target {
                Target::Settled => None,
                Target::Strict => {
                    let to: String = SchemaPath::root().field("questions").key(key.as_str()).into();
                    Some(alloc::format!("{to}{rest}"))
                }
                Target::Situation(None) => Some(SchemaPath::root().field("situations").into()),
                Target::Situation(Some(index)) => {
                    let to: String = SchemaPath::root()
                        .field("situations")
                        .index(*index)
                        .field("situation")
                        .into();
                    Some(alloc::format!("{to}{}", rest.strip_prefix(answer).unwrap_or_default()))
                }
            };
        }
        for key in &self.wages {
            let row: String =
                SchemaPath::root().field("wages").key(key.as_str()).field("answers").into();
            let Some(rest) = field.strip_prefix(row.as_str()) else { continue };
            for (part, member) in
                [("overtime", ".overtime_flsa_required"), ("occupation", ".tips_listed_occupation")]
            {
                if let Some(rest) = rest.strip_prefix(member) {
                    let to: String = SchemaPath::root()
                        .field("questions")
                        .key(alloc::format!("{key}/{part}"))
                        .field("answer")
                        .field("value")
                        .into();
                    return Some(alloc::format!("{to}{rest}"));
                }
            }
            if rest.starts_with(".box14") {
                return Some(SchemaPath::root().field("questions").into());
            }
            return Some(SchemaPath::root().field("situations").into());
        }
        // The questions page holds no other field of its topic.
        self.questions_page.is_none().then_some(field)
    }
}

impl Interview {
    // The protocol request in the interview's canonical form.
    pub(crate) fn wire_request(
        &self,
        mut request: P,
    ) -> Result<(P, Option<WireTargets>), alloc::boxed::Box<Reply>> {
        let wage_save = matches!(request, P::SaveWagesAnswers { .. });
        let targets = match &mut request {
            P::SaveIncomes { topic, followups, questions, situations, .. } => join(
                &self.draft,
                *topic,
                followups,
                core::mem::take(questions),
                core::mem::replace(situations, DocumentSituations::NoneOfThese),
                true,
            )?,
            P::EstimateDocuments { topic, followups, questions, situations, .. } => join(
                &self.draft,
                *topic,
                followups,
                core::mem::take(questions),
                core::mem::replace(situations, DocumentSituations::NoneOfThese),
                false,
            )?,
            P::SaveWagesAnswers { wages, questions, situations, .. }
            | P::EstimateWagesAnswers { wages, questions, situations, .. } => {
                let rows = join_wages(
                    &self.draft,
                    core::mem::take(wages),
                    core::mem::take(questions),
                    core::mem::replace(situations, DocumentSituations::NoneOfThese),
                    !self.allowed_wage_scorporations().is_empty(),
                    wage_save,
                )?;
                let keys = rows.iter().map(|row| row.key.clone()).collect();
                *wages = rows;
                let targets = WireTargets { rows: Vec::new(), wages: keys, questions_page: None };
                return Ok((request, Some(targets)));
            }
            P::SaveHsa { followups, questions, .. }
            | P::EstimateHsa { followups, questions, .. } => {
                join_own(followups, core::mem::take(questions))?
            }
            P::SaveEducation { followups, questions, .. }
            | P::EstimateEducation { followups, questions, .. } => {
                join_own(followups, core::mem::take(questions))?
            }
            P::SaveInjuredSpouse { followups, questions, .. }
            | P::EstimateInjuredSpouse { followups, questions, .. } => {
                join_own(followups, core::mem::take(questions))?
            }
            P::SaveDocumentQuestions {
                cookie,
                revision,
                review,
                topic,
                questions,
                situations,
                ..
            } => {
                if *cookie != self.draft.cookie || *revision != self.draft.revision {
                    return Err(alloc::boxed::Box::new(stale_editor_reply()));
                }
                // The credit questions locate their own refusals.
                if *topic == TaxSection::MoreCredits {
                    return Ok((request, None));
                }
                if OWN_TOPICS.contains(topic) {
                    let targets = WireTargets {
                        rows: Vec::new(),
                        wages: Vec::new(),
                        questions_page: Some((*topic, *review)),
                    };
                    return Ok((request, Some(targets)));
                }
                // Where each answer lands, for locating a refusal on this page.
                let (rows, wages) = if *topic == TaxSection::Wages {
                    let rows = join_wages(
                        &self.draft,
                        Vec::new(),
                        questions.clone(),
                        situations.clone(),
                        !self.allowed_wage_scorporations().is_empty(),
                        true,
                    )?;
                    (Vec::new(), rows.into_iter().map(|row| row.key).collect())
                } else {
                    let mut followups = Vec::new();
                    let rows = join(
                        &self.draft,
                        *topic,
                        &mut followups,
                        questions.clone(),
                        situations.clone(),
                        true,
                    )?;
                    (rows, Vec::new())
                };
                let targets = WireTargets { rows, wages, questions_page: Some((*topic, *review)) };
                return Ok((request, Some(targets)));
            }
            _ => return Ok((request, None)),
        };
        Ok((request, Some(WireTargets { rows: targets, wages: Vec::new(), questions_page: None })))
    }

    // The protocol reply: an income topic page regrouped for the person, and
    // a refusal located on the page field the person answered.
    pub(crate) fn wire_reply(&self, reply: Reply, targets: Option<&WireTargets>) -> Reply {
        match reply {
            Reply::Response(P::Problem { message, field: Some(field) }) => {
                let field = match targets {
                    Some(targets) => targets.locate(field),
                    None => Some(field),
                };
                Reply::Response(P::Problem { message, field })
            }
            Reply::Response(P::Problem { .. }) => reply,
            // A saved questions page continues on its topic page.
            Reply::Edit(mut page @ P::SaveWagesAnswers { .. }) => {
                let P::SaveWagesAnswers {
                    wages,
                    questions,
                    situations,
                    allowed_situations,
                    situation_answers,
                    situation_documents,
                    disaster_choices,
                    ..
                } = &mut page
                else {
                    unreachable!()
                };
                let split = split_wages(
                    &self.draft,
                    &core::mem::take(wages),
                    !self.allowed_wage_scorporations().is_empty(),
                );
                *questions = split.questions;
                *situations = split.situations.clone();
                *situation_answers = alloc::vec![0, 1];
                *allowed_situations = split.allowed_situations;
                *situation_documents = split.situation_documents;
                *disaster_choices = situation_disaster_choices();
                if split.new_questions.is_empty() && !split.situations_open {
                    return Reply::Edit(page);
                }
                let answered = (!split.situations_open)
                    .then_some(DocumentSituationsAnswered { value: split.situations })
                    .into_iter()
                    .collect();
                Reply::Response(wage_questions_page(page, split.new_questions, answered))
            }
            Reply::Response(page @ P::BeginMortgages { .. })
            | Reply::Edit(page @ P::SaveMortgages { .. }) => {
                let open = self.mortgage_refund_questions();
                if open.is_empty() {
                    return match page {
                        P::SaveMortgages { .. } => Reply::Edit(page),
                        _ => Reply::Response(page),
                    };
                }
                Reply::Response(own_questions_page(page, TaxSection::Mortgage, open))
            }
            Reply::Edit(mut page @ P::SaveHsa { .. }) => {
                let P::SaveHsa { followups, questions, value, .. } = &mut page else {
                    unreachable!()
                };
                let (rest, asked, mut open) = split_own(core::mem::take(followups));
                open.extend(self.employer_year_questions());
                open.extend(self.hsa_other_question(value));
                (*followups, *questions) = (rest, asked);
                if open.is_empty() {
                    return Reply::Edit(page);
                }
                Reply::Response(own_questions_page(page, TaxSection::Hsa, open))
            }
            Reply::Edit(mut page @ P::SaveEducation { .. }) => {
                let P::SaveEducation { followups, questions, .. } = &mut page else {
                    unreachable!()
                };
                let (rest, asked, open) = split_own(core::mem::take(followups));
                (*followups, *questions) = (rest, asked);
                if open.is_empty() {
                    return Reply::Edit(page);
                }
                Reply::Response(own_questions_page(page, TaxSection::Education, open))
            }
            Reply::Response(mut page @ P::BeginInjuredSpouse { .. }) => {
                let P::BeginInjuredSpouse { followups, questions, .. } = &mut page else {
                    unreachable!()
                };
                let (rest, asked, open) = split_own(core::mem::take(followups));
                (*followups, *questions) = (rest, asked);
                if open.is_empty() {
                    return Reply::Response(page);
                }
                Reply::Response(own_questions_page(page, TaxSection::InjuredSpouse, open))
            }
            Reply::Edit(mut page @ P::SaveIncomes { .. }) => {
                let P::SaveIncomes {
                    followups,
                    questions,
                    situations,
                    allowed_situations,
                    situation_answers,
                    situation_documents,
                    disaster_choices,
                    ..
                } = &mut page
                else {
                    unreachable!()
                };
                let split = split(&self.draft, core::mem::take(followups));
                *followups = split.followups;
                *questions = split.questions;
                *situations = split.situations.clone();
                *situation_answers = if split.allowed_situations.is_empty() {
                    alloc::vec![0]
                } else {
                    alloc::vec![0, 1]
                };
                *allowed_situations = split.allowed_situations.clone();
                *situation_documents = split.situation_documents.clone();
                *disaster_choices = situation_disaster_choices();
                if split.new_questions.is_empty() && !split.situations_open {
                    return Reply::Edit(page);
                }
                let answered = (!split.situations_open)
                    .then_some(DocumentSituationsAnswered { value: split.situations })
                    .into_iter()
                    .collect();
                Reply::Response(questions_page(page, split.new_questions, answered))
            }
            reply => reply,
        }
    }
}

impl Interview {
    // The questions page saves only its answers: the topic's other rows and
    // records keep their saved values and are checked on the topic page,
    // which the reply opens.
    pub(super) fn save_document_questions(&mut self, request: P) -> Reply {
        let P::SaveDocumentQuestions {
            cookie, revision, review, topic, questions, situations, ..
        } = request
        else {
            unreachable!("the questions page")
        };
        if cookie != self.draft.cookie || revision != self.draft.revision {
            return stale_editor_reply();
        }
        if OWN_TOPICS.contains(&topic) {
            return self.save_own_questions(topic, review, questions);
        }
        if topic == TaxSection::MoreCredits {
            return self.save_credit_questions(review, questions);
        }
        let candidate = if topic == TaxSection::Wages {
            let Reply::Edit(P::SaveWagesAnswers { followups, records, .. }) =
                self.wages_answers_page(review, None)
            else {
                return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
            };
            let wages = match join_wages(
                &self.draft,
                Vec::new(),
                questions,
                situations,
                !self.allowed_wage_scorporations().is_empty(),
                true,
            ) {
                Ok(wages) => wages,
                Err(reply) => return *reply,
            };
            self.wages_answers_candidate(wages, followups, records, false, None)
        } else {
            let Reply::Edit(P::SaveIncomes { mut followups, documents, .. }) =
                self.topic_page(topic, review)
            else {
                return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
            };
            followups.retain(|row| matches!(kind(row), Kind::Other | Kind::Wage));
            if let Err(reply) =
                join(&self.draft, topic, &mut followups, questions, situations, true)
            {
                return *reply;
            }
            self.followup_candidate_with_mode(topic, followups, false, &documents.documents)
                .map(|candidate| candidate.draft)
        };
        let committed = candidate.and_then(|mut draft| {
            answer_situations(&mut draft, topic);
            self.commit_income(draft, &[], None)
        });
        match committed {
            Ok(_) if topic == TaxSection::Wages => self.wages_answers_page(review, None),
            Ok(_) => self.topic_page(topic, review),
            Err(failure) => refused(failure),
        }
    }
}

const EMPLOYER_KEY: &str = "employer/";
const OTHER_KEY: &str = "other_coverage";
const MORTGAGE_KEY: &str = "mortgage/";

impl Interview {
    // Each entered W-2 medical-account amount whose designated contribution
    // year is not answered is a required question on the HSA page's questions
    // page; the page itself opens only with answered years.
    fn employer_year_questions(&self) -> Vec<DocumentStrictQuestionInitial> {
        let d = &self.draft;
        let open = crate::health_employer::open_sources(d, crate::health_employer::bindings(d))
            .unwrap_or_default();
        open.into_iter()
            .map(|source| {
                let (tag, resource) = match source.kind {
                    crate::HealthEmployerKind::Hsa => ("hsa", "DOCUMENT_ANSWER_EMPLOYERYEAR_HSA"),
                    crate::HealthEmployerKind::Archer => {
                        ("archer", "DOCUMENT_ANSWER_EMPLOYERYEAR_ARCHER")
                    }
                };
                DocumentStrictQuestionInitial {
                    key: alloc::format!("{EMPLOYER_KEY}{}/{tag}", source.document),
                    prompt: Message::new(
                        "DOCUMENT_TOPIC_SOURCE_ITEM",
                        alloc::vec![
                            MessageArg::Resource(resource.into()),
                            MessageArg::Encoded(source.label),
                        ],
                    )
                    .encode(),
                    allowed_answers: alloc::vec![
                        DocumentStrictAnswer::EmployerYear {
                            value: crate::HealthEmployerYear::CurrentYear
                        }
                        .index()
                    ],
                    allowed_values: Vec::new(),
                    ccc_lots: Vec::new(),
                    health_plans: Vec::new(),
                    espp_acquisitions: Vec::new(),
                    interest_statements: Vec::new(),
                    people: Vec::new(),
                    credit_income_sources: Vec::new(),
                }
            })
            .collect()
    }

    // The loans whose printed box 4 reports a refund.
    fn mortgage_refund_loans(&self) -> Vec<i64> {
        self.draft.deductions.as_ref().map_or_else(Vec::new, |deductions| {
            deductions.mortgage.iter().filter(|s| s.form.box_4 > 0).map(|s| s.id).collect()
        })
    }

    // The Mortgage page's one link to the answered refund questions.
    pub(crate) fn mortgage_questions_link(&self) -> Vec<DocumentQuestionsLink> {
        let answered = self.mortgage_answered_refunds().len();
        if answered == 0 {
            return Vec::new();
        }
        alloc::vec![DocumentQuestionsLink {
            topic: TaxSection::Mortgage,
            status: status_count("MORTGAGE_REFUND_ORIGIN_ANSWERED", answered),
        }]
    }

    fn mortgage_answered_refunds(&self) -> Vec<(i64, crate::MortgageRefundOrigin)> {
        self.draft.deductions.as_ref().map_or_else(Vec::new, |deductions| {
            deductions
                .mortgage
                .iter()
                .filter(|s| s.form.box_4 > 0)
                .filter_map(|s| s.refund_origin.clone().map(|origin| (s.id, origin)))
                .collect()
        })
    }

    // The link reopens the questions page by EditRequest with the stored
    // answers shown and editable; with an unanswered one it reopens the
    // Mortgage page, which asks it first.
    pub(super) fn open_mortgage_questions(&self, review: bool) -> Reply {
        let answered = self.mortgage_answered_refunds();
        let d = &self.draft;
        if answered.is_empty() || !self.mortgage_refund_questions().is_empty() {
            return self.mortgage_page(review);
        }
        let parent = mortgage_parent(d);
        let questions = answered
            .into_iter()
            .map(|(id, origin)| {
                let source =
                    d.deductions.as_ref().and_then(|x| x.mortgage.iter().find(|s| s.id == id));
                DocumentStrictQuestion {
                    key: alloc::format!("{MORTGAGE_KEY}{id}"),
                    prompt: Message::new(
                        "DOCUMENT_TOPIC_SOURCE_ITEM",
                        alloc::vec![
                            MessageArg::Resource("DOCUMENT_ANSWER_MORTGAGEREFUNDORIGIN".into()),
                            MessageArg::Encoded(document_followups::mortgage_document_name(
                                source.unwrap()
                            )),
                        ],
                    )
                    .encode(),
                    allowed_answers: alloc::vec![
                        DocumentStrictAnswer::MortgageRefundOrigin {
                            value: crate::MortgageRefundOrigin::ThisYear
                        }
                        .index()
                    ],
                    allowed_values: Vec::new(),
                    ccc_lots: Vec::new(),
                    health_plans: Vec::new(),
                    espp_acquisitions: Vec::new(),
                    interest_statements: Vec::new(),
                    people: Vec::new(),
                    credit_income_sources: Vec::new(),
                    answer: DocumentStrictAnswer::MortgageRefundOrigin { value: origin },
                }
            })
            .collect();
        Reply::Edit(P::SaveDocumentQuestions {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: parent,
            previous: parent,
            review,
            estimate: self.estimate_now(TaxSection::Mortgage),
            progress: self.topic_progress(parent, Some(TaxSection::Mortgage)),
            topic: TaxSection::Mortgage,
            questions,
            situations: DocumentSituations::NoneOfThese,
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

    // A printed box 4 refund whose origin is not answered is a required
    // question on the mortgage questions page, asked before the page opens.
    fn mortgage_refund_questions(&self) -> Vec<DocumentStrictQuestionInitial> {
        let Some(deductions) = self.draft.deductions.as_ref() else { return Vec::new() };
        deductions
            .mortgage
            .iter()
            .filter(|source| source.form.box_4 > 0 && source.refund_origin.is_none())
            .map(|source| DocumentStrictQuestionInitial {
                key: alloc::format!("{MORTGAGE_KEY}{}", source.id),
                prompt: Message::new(
                    "DOCUMENT_TOPIC_SOURCE_ITEM",
                    alloc::vec![
                        MessageArg::Resource("DOCUMENT_ANSWER_MORTGAGEREFUNDORIGIN".into()),
                        MessageArg::Encoded(document_followups::mortgage_document_name(source)),
                    ],
                )
                .encode(),
                allowed_answers: alloc::vec![
                    DocumentStrictAnswer::MortgageRefundOrigin {
                        value: crate::MortgageRefundOrigin::ThisYear
                    }
                    .index()
                ],
                allowed_values: Vec::new(),
                ccc_lots: Vec::new(),
                health_plans: Vec::new(),
                espp_acquisitions: Vec::new(),
                interest_statements: Vec::new(),
                people: Vec::new(),
                credit_income_sources: Vec::new(),
            })
            .collect()
    }

    // The submitted origins become the loans' stored refund origins; every
    // open loan needs exactly one answer.
    fn answer_mortgage_refunds(
        &self,
        mut draft: FederalDraft,
        answers: Vec<DocumentStrictQuestion>,
    ) -> Result<FederalDraft, SaveFailure> {
        let changed = || {
            SaveFailure::from(
                ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("questions"),
            )
        };
        // Every open loan is answered; an answered loan may be corrected.
        let open = self.mortgage_refund_questions();
        let eligible = self.mortgage_refund_loans();
        if answers
            .iter()
            .any(|a| !eligible.iter().any(|id| a.key == alloc::format!("{MORTGAGE_KEY}{id}")))
            || answers.iter().enumerate().any(|(i, a)| answers[..i].iter().any(|b| b.key == a.key))
            || open.iter().any(|q| !answers.iter().any(|a| a.key == q.key))
        {
            return Err(changed());
        }
        for answer in answers {
            let id = answer.key[MORTGAGE_KEY.len()..].parse::<i64>().map_err(|_| changed())?;
            let DocumentStrictAnswer::MortgageRefundOrigin { value } = answer.answer else {
                return Err(changed());
            };
            if let crate::MortgageRefundOrigin::Deducted(prior) = &value
                && !prior.valid()
            {
                return Err(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED")
                    .at_field("answer")
                    .into());
            }
            let source = draft
                .deductions
                .as_mut()
                .and_then(|d| d.mortgage.iter_mut().find(|s| s.id == id))
                .ok_or_else(changed)?;
            source.refund_origin = Some(value);
        }
        Ok(draft)
    }

    // The married return's single contributing account calls for the other
    // filer's HSA eligibility; unanswered, it is asked on the HSA questions page.
    fn hsa_other_question(&self, _value: &HsaPage) -> Option<DocumentStrictQuestionInitial> {
        let owner = self.hsa_other_open()?;
        let resource = if owner == Owner::Taxpayer {
            "DOCUMENT_ANSWER_OTHERHSAELIGIBILITY_SPOUSE"
        } else {
            "DOCUMENT_ANSWER_OTHERHSAELIGIBILITY_YOU"
        };
        Some(DocumentStrictQuestionInitial {
            key: OTHER_KEY.into(),
            prompt: status(resource),
            allowed_answers: alloc::vec![
                DocumentStrictAnswer::OtherHsaEligibility {
                    value: crate::HsaEligibilityPage::FullYear {
                        coverage: crate::HsaCoverage::None
                    }
                }
                .index()
            ],
            allowed_values: Vec::new(),
            ccc_lots: Vec::new(),
            health_plans: Vec::new(),
            espp_acquisitions: Vec::new(),
            interest_statements: Vec::new(),
            people: Vec::new(),
            credit_income_sources: Vec::new(),
        })
    }

    // The submitted eligibility becomes the stored context; it must still be
    // the open question.
    fn answer_hsa_other(
        &self,
        mut draft: FederalDraft,
        answers: Vec<DocumentStrictQuestion>,
    ) -> Result<FederalDraft, SaveFailure> {
        let changed = || {
            SaveFailure::from(
                ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("questions"),
            )
        };
        let open = self.hsa_other_open().is_some();
        let mut answers = answers.into_iter();
        let (Some(answer), None) = (answers.next(), answers.next()) else {
            return if open { Err(changed()) } else { Ok(draft) };
        };
        let DocumentStrictAnswer::OtherHsaEligibility { value } = answer.answer else {
            return Err(changed());
        };
        if !open {
            return Err(changed());
        }
        let method = hsa_limit_method(&draft);
        let adjustments = draft.adjustments.as_mut().ok_or_else(changed)?;
        let share = adjustments.hsa_family_share.get_or_insert(crate::HsaFamilyShare {
            method,
            taxpayer_parts: 1,
            spouse_parts: 1,
            other_coverage: None,
        });
        share.other_coverage = Some(value.record());
        Ok(draft)
    }

    // The submitted contribution years become accepted bindings; every open
    // source needs exactly one answer.
    fn answer_employer_years(
        &self,
        mut draft: FederalDraft,
        answers: Vec<DocumentStrictQuestion>,
    ) -> Result<FederalDraft, SaveFailure> {
        let changed = || {
            SaveFailure::from(
                ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("questions"),
            )
        };
        let open = self.employer_year_questions();
        if answers.len() != open.len()
            || open.iter().any(|q| !answers.iter().any(|a| a.key == q.key))
        {
            return Err(changed());
        }
        for answer in answers {
            let mut parts = answer.key[EMPLOYER_KEY.len()..].split('/');
            let document = parts.next().and_then(|v| v.parse::<i64>().ok());
            let kind = match parts.next() {
                Some("hsa") => crate::HealthEmployerKind::Hsa,
                Some("archer") => crate::HealthEmployerKind::Archer,
                _ => return Err(changed()),
            };
            let (Some(document), DocumentStrictAnswer::EmployerYear { value }) =
                (document, answer.answer)
            else {
                return Err(changed());
            };
            let binding = crate::health_employer::answered_binding(&draft, document, kind, value)
                .map_err(|issue| SaveFailure::from(issue.at_field("answer")))?;
            let adjustments = draft.adjustments.as_mut().ok_or_else(changed)?;
            adjustments.health_employer_w2.retain(|b| !(b.document == document && b.kind == kind));
            adjustments.health_employer_w2.push(binding);
        }
        Ok(draft)
    }

    // The own-list pages' questions page saves only its answers, then reopens
    // the topic page, which checks everything else.
    fn save_own_questions(
        &mut self,
        topic: TaxSection,
        review: bool,
        questions: Vec<DocumentStrictQuestion>,
    ) -> Reply {
        let candidate = match topic {
            TaxSection::Education => {
                let mut rows = self.education_followups();
                match fill_own(&mut rows, questions) {
                    Ok(()) => self.education_followup_candidate(&rows, false),
                    Err(reply) => return *reply,
                }
            }
            TaxSection::Mortgage => {
                return match self.answer_mortgage_refunds(self.draft.clone(), questions).and_then(
                    |draft| self.commit_issue(draft).map_err(|(issue, _, _)| issue.into()),
                ) {
                    Ok(_) => self.mortgage_page(review),
                    Err(failure) => refused(failure),
                };
            }
            TaxSection::Hsa => {
                let (employer, questions): (Vec<_>, Vec<_>) =
                    questions.into_iter().partition(|q| q.key.starts_with(EMPLOYER_KEY));
                let (other, questions): (Vec<_>, Vec<_>) =
                    questions.into_iter().partition(|q| q.key == OTHER_KEY);
                let mut rows = self.hsa_followups();
                match fill_own(&mut rows, questions) {
                    Ok(()) => {
                        self.hsa_followup_candidate(&rows, false).and_then(|mut candidate| {
                            candidate.draft =
                                self.answer_employer_years(candidate.draft, employer)?;
                            candidate.draft = self.answer_hsa_other(candidate.draft, other)?;
                            Ok(candidate)
                        })
                    }
                    Err(reply) => return *reply,
                }
            }
            _ => {
                let Reply::Response(P::BeginInjuredSpouse { followups: mut rows, .. }) =
                    self.injured_spouse_page(review)
                else {
                    return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
                };
                let allowed = crate::injured_spouse_payments::allowed_document_management(
                    crate::injured_spouse::texas(&self.draft),
                );
                if let Err(reply) = fill_own(&mut rows, questions) {
                    return *reply;
                }
                // Management is independent of every other answer: only the
                // chosen value of each document is saved.
                let mut draft = self.draft.clone();
                for row in &rows {
                    let A::Management { value: Some(value) } = row.canonical() else { continue };
                    let id = row_id(&row.key);
                    let entry = id.and_then(|id| draft.income.iter_mut().find(|e| e.id == id));
                    let Some(entry) = entry.filter(|_| {
                        allowed.contains(&crate::injured_spouse_payments::management_index(value))
                    }) else {
                        return refused(ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED"));
                    };
                    entry.refund_management = Some(value);
                }
                return match self.commit_issue(draft) {
                    Ok(_) => self.injured_spouse_page(review),
                    Err((issue, _, _)) => refused(issue),
                };
            }
        };
        match candidate.and_then(|candidate| {
            self.commit_issue(candidate.draft).map_err(|(issue, _, _)| issue.into())
        }) {
            Ok(_) if topic == TaxSection::Education => self.education_page(review),
            Ok(_) => self.hsa_page(review),
            Err(failure) => refused(failure),
        }
    }
}

// The questions page's construction source: the topic page's context, the
// unanswered questions without an answer, and the situations question only
// once it has been answered.
fn questions_page(
    page: P,
    questions: Vec<DocumentStrictQuestionInitial>,
    situations: Vec<DocumentSituationsAnswered>,
) -> P {
    let P::SaveIncomes {
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        progress,
        topic,
        allowed_situations,
        situation_answers,
        situation_documents,
        disaster_choices,
        inherited_iras,
        businesses,
        people,
        inherited_roth_iras,
        wage_scorporations,
        ..
    } = page
    else {
        unreachable!("only an income topic page asks its document questions first")
    };
    P::BeginDocumentQuestions {
        cookie,
        revision,
        page,
        previous,
        review,
        estimate: estimate.map(Into::into),
        progress: progress.unwrap_or_default(),
        topic,
        questions,
        situations,
        allowed_situations,
        situation_answers,
        situation_documents,
        disaster_choices,
        inherited_iras,
        businesses,
        people,
        inherited_roth_iras,
        wage_scorporations,
    }
}

// The wages page's questions page: the W-2s' unanswered questions and the
// page's situations question.
fn wage_questions_page(
    page: P,
    questions: Vec<DocumentStrictQuestionInitial>,
    situations: Vec<DocumentSituationsAnswered>,
) -> P {
    let P::SaveWagesAnswers {
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        progress,
        businesses,
        wage_scorporations,
        allowed_situations,
        situation_answers,
        situation_documents,
        disaster_choices,
        ..
    } = page
    else {
        unreachable!("only the wages page asks its W-2 questions first")
    };
    P::BeginDocumentQuestions {
        cookie,
        revision,
        page,
        previous,
        review,
        estimate: estimate.map(Into::into),
        progress: progress.unwrap_or_default(),
        topic: TaxSection::Wages,
        questions,
        situations,
        allowed_situations,
        situation_answers,
        situation_documents,
        disaster_choices,
        inherited_iras: Vec::new(),
        businesses,
        people: Vec::new(),
        inherited_roth_iras: Vec::new(),
        wage_scorporations,
    }
}
