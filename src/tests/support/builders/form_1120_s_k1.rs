extern crate std;

use crate::tests::*;

pub(in crate::tests) fn scorp_source(data: ScorpK1) -> ScorpSource {
    let page =
        crate::k1_page::K1Scorp::from_record("", &data).expect("representable shareholder fixture");
    let document = super::scorporation::scorp_paper_document(page);
    ScorpSource {
        paper: document.form,
        statements: document.statements,
        personal: ScorpPersonalRecords::from_calculation(&data)
            .expect("representable shareholder answers"),
        assigned: crate::DocumentRecipient::Unknown,
        owner_source: None,
        joint_share: None,
    }
}

std::thread_local! {
    // Documents whose answers were read back from a session, not supplied by a fixture.
    static REOPENED_SCORP: core::cell::RefCell<Vec<K1ScorpDocument>> = const { core::cell::RefCell::new(Vec::new()) };
}

std::thread_local! {
    static SCORP_ANSWERS: core::cell::RefCell<Vec<(K1ScorpDocument, ScorpPersonalRecords)>> = const { core::cell::RefCell::new(Vec::new()) };
}

pub(in crate::tests) fn remember_scorp_document(
    document: K1ScorpDocument,
    personal: ScorpPersonalRecords,
) -> K1ScorpDocument {
    REOPENED_SCORP.with(|rows| rows.borrow_mut().retain(|given| given != &document));
    SCORP_ANSWERS.with(|rows| {
        let mut rows = rows.borrow_mut();
        rows.retain(|(given, _)| given != &document);
        rows.push((document.clone(), personal));
    });
    document
}

// A fixture's own answers for this exact paper; a paper the test edited in a
// page it reopened keeps the answers its accepted source holds.
pub(in crate::tests) fn scorp_fixture_source_in(
    draft: &FederalDraft,
    id: i64,
    document: &K1ScorpDocument,
) -> ScorpSource {
    let registered =
        SCORP_ANSWERS.with(|rows| rows.borrow().iter().any(|(given, _)| given == document));
    let mut source = scorp_fixture_source(document);
    if !registered
        && let Some(FederalIncome::Scorp { data }) =
            draft.income.iter().find(|entry| entry.id == id).map(|entry| &entry.income)
    {
        source.personal = data.personal.clone();
    }
    source
}

// Naming a fixture preserves its registration; accepted snapshots must never
// become newly supplied answers merely because their printed recipient changed.
fn scorp_fixture_papers_match(given: &K1ScorpDocument, document: &K1ScorpDocument) -> bool {
    let mut candidate = document.clone();
    if given.form.corporation == crate::PrintedParty::default() {
        candidate.form.corporation = given.form.corporation.clone();
    }
    if given.form.shareholder == crate::PrintedParty::default() {
        candidate.form.shareholder = given.form.shareholder.clone();
    }
    candidate == *given
}

fn scorp_fixture_index(
    rows: &[(K1ScorpDocument, ScorpPersonalRecords)],
    document: &K1ScorpDocument,
) -> Option<usize> {
    rows.iter()
        .rposition(|(given, _)| given == document)
        .or_else(|| rows.iter().rposition(|(given, _)| scorp_fixture_papers_match(given, document)))
}

fn registered_scorp_answers(document: &K1ScorpDocument) -> Option<ScorpPersonalRecords> {
    SCORP_ANSWERS.with(|rows| {
        let rows = rows.borrow();
        scorp_fixture_index(&rows, document).map(|index| rows[index].1.clone())
    })
}

pub(in crate::tests) fn scorp_fixture_answers_pending(document: &K1ScorpDocument) -> bool {
    SCORP_ANSWERS.with(|rows| {
        let rows = rows.borrow();
        scorp_fixture_index(&rows, document).is_some_and(|index| {
            REOPENED_SCORP.with(|accepted| !accepted.borrow().contains(&rows[index].0))
        })
    })
}

// Only the caller's successful real topic Save consumes a supplied registration.
// Its named and unnamed papers then both describe an accepted snapshot.
pub(in crate::tests) fn mark_scorp_fixture_answers_applied(document: &K1ScorpDocument) {
    SCORP_ANSWERS.with(|rows| {
        let rows = rows.borrow();
        REOPENED_SCORP.with(|accepted| {
            let mut accepted = accepted.borrow_mut();
            for (given, _) in rows.iter().filter(|(given, _)| {
                scorp_fixture_papers_match(given, document)
                    || scorp_fixture_papers_match(document, given)
            }) {
                if !accepted.contains(given) {
                    accepted.push(given.clone());
                }
            }
        });
    });
}

/// A scorp document reopened from a session is a new value; its fixture answers are the stored
/// source's own, unless a fixture already registered it.
pub(in crate::tests) fn remember_reopened_scorp_document(
    document: K1ScorpDocument,
    personal: ScorpPersonalRecords,
) {
    let reopened = REOPENED_SCORP.with(|rows| rows.borrow().contains(&document));
    if reopened || registered_scorp_answers(&document).is_none() {
        let document = remember_scorp_document(document, personal);
        REOPENED_SCORP.with(|rows| rows.borrow_mut().push(document));
    }
}

pub(in crate::tests) fn scorp_fixture_source(document: &K1ScorpDocument) -> ScorpSource {
    let mut source = document.source();
    if let Some(personal) = registered_scorp_answers(document) {
        source.personal = personal;
    }
    source
}

pub(super) fn rebind_scorp_fixture_answers(before: &K1ScorpDocument, after: &K1ScorpDocument) {
    let source = scorp_fixture_source(before);
    let reopened = REOPENED_SCORP.with(|rows| rows.borrow().contains(before));
    remember_scorp_document(after.clone(), source.personal);
    if reopened {
        REOPENED_SCORP.with(|rows| rows.borrow_mut().push(after.clone()));
    }
}
