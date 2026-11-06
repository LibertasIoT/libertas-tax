// The questions page of a topic that keeps its own question list (education,
// medical accounts, injured spouse), driven as the endpoint drives it.
use super::{TEST_USER, send};
use crate::interview::{Interview, Reply};
use crate::{
    DocumentSituations, DocumentStrictAnswer, DocumentStrictQuestion, TaxInterviewProtocol as P,
};
use alloc::vec::Vec;

// The request and reply as the endpoint receives and sends them.
pub(in crate::tests) fn wire(s: &mut Interview, request: P) -> Reply {
    let (request, targets) = match s.wire_request(request) {
        Ok(request) => request,
        Err(reply) => return *reply,
    };
    let reply = send(s, request);
    s.wire_reply(reply, targets.as_ref())
}

// The topic page as the endpoint opens it.
pub(in crate::tests) fn wire_page(s: &mut Interview, reply: Reply) -> Reply {
    s.wire_reply(reply, None)
}

// The questions page the endpoint opened, submitted with the given answers in
// the order the page asks them.
pub(in crate::tests) fn answer_questions(
    s: &mut Interview,
    begin: &P,
    answers: Vec<DocumentStrictAnswer>,
) -> Reply {
    let P::BeginDocumentQuestions { questions, .. } = begin else {
        panic!("the questions page opens through BeginDocumentQuestions")
    };
    assert_eq!(questions.len(), answers.len(), "one answer for each question asked");
    let answered = questions
        .iter()
        .zip(answers)
        .map(|(asked, answer)| DocumentStrictQuestion {
            key: asked.key.clone(),
            prompt: Vec::new(),
            allowed_answers: Vec::new(),
            allowed_values: Vec::new(),
            ccc_lots: Vec::new(),
            health_plans: Vec::new(),
            espp_acquisitions: Vec::new(),
            interest_statements: Vec::new(),
            people: Vec::new(),
            credit_income_sources: Vec::new(),
            answer,
        })
        .collect();
    save_questions(s, begin, answered)
}

// The questions page submitted with exactly these questions.
pub(in crate::tests) fn save_questions(
    s: &mut Interview,
    begin: &P,
    answered: Vec<DocumentStrictQuestion>,
) -> Reply {
    let request = questions_request(begin, answered);
    wire(s, request)
}

// The questions page's Save request with exactly these questions.
pub(in crate::tests) fn questions_request(begin: &P, answered: Vec<DocumentStrictQuestion>) -> P {
    let P::BeginDocumentQuestions {
        cookie,
        revision,
        page,
        previous,
        review,
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
    } = begin.clone()
    else {
        panic!("the questions page opens through BeginDocumentQuestions")
    };
    P::SaveDocumentQuestions {
        user: TEST_USER,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate: None,
        progress,
        topic,
        questions: answered,
        situations: DocumentSituations::NoneOfThese,
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
