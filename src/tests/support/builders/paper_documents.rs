use super::form_1041_k1::remember_estate_answers;
use super::form_1099_patr::remember_cooperative_answers;
use alloc::string::String;
extern crate std;
use crate::tests::*;

// Recipient normalization adds aliases to one supplied set of answers. Equal
// printed papers can still carry distinct beneficiary or receipt facts.
struct PaperFixtureAnswers {
    papers: Vec<IncomeSource>,
    answers: Vec<crate::DocumentAnswer>,
    applied: Vec<(String, i64)>,
    supplied_id: Option<i64>,
}

std::thread_local! {
    static PAPER_ANSWERS: core::cell::RefCell<Vec<PaperFixtureAnswers>> = const { core::cell::RefCell::new(Vec::new()) };
}

pub(in crate::tests) fn fixture_paper_answers(paper: &IncomeSource) -> Vec<crate::DocumentAnswer> {
    fixture_paper_answers_at(paper, 0, 1)
}

pub(in crate::tests) fn fixture_paper_answers_at(
    paper: &IncomeSource,
    ordinal: usize,
    total: usize,
) -> Vec<crate::DocumentAnswer> {
    fixture_paper_answers_for_source(paper, ordinal, total, 0)
}

pub(in crate::tests) fn fixture_paper_answers_for_source(
    paper: &IncomeSource,
    ordinal: usize,
    total: usize,
    supplied_id: i64,
) -> Vec<crate::DocumentAnswer> {
    PAPER_ANSWERS.with(|rows| {
        let rows = rows.borrow();
        paper_fixture_index(&rows, paper, ordinal, total, supplied_id)
            .map_or_else(Vec::new, |index| rows[index].answers.clone())
    })
}

fn paper_fixture_index(
    rows: &[PaperFixtureAnswers],
    paper: &IncomeSource,
    ordinal: usize,
    total: usize,
    supplied_id: i64,
) -> Option<usize> {
    if supplied_id != 0
        && let Some(index) = rows
            .iter()
            .rposition(|row| row.papers.contains(paper) && row.supplied_id == Some(supplied_id))
    {
        return Some(index);
    }
    let matching: Vec<_> = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            (row.papers.contains(paper) && row.supplied_id.is_none()).then_some(index)
        })
        .collect();
    if matching.len() == 1 {
        return Some(matching[0]);
    }
    matching.get(matching.len().saturating_sub(total) + ordinal).copied()
}

pub(in crate::tests) fn fixture_paper_has_answers(paper: &IncomeSource) -> bool {
    PAPER_ANSWERS.with(|rows| rows.borrow().iter().any(|row| row.papers.contains(paper)))
}

pub(in crate::tests) fn fixture_paper_answers_pending(
    paper: &IncomeSource,
    cookie: &str,
    supplied_id: i64,
    require_bound: bool,
) -> bool {
    PAPER_ANSWERS.with(|rows| {
        let rows = rows.borrow();
        paper_fixture_index(&rows, paper, 0, 1, supplied_id).is_some_and(|index| {
            // A consumed equal-paper registration belongs to its accepted
            // source; another source must never borrow its personal history.
            (!require_bound || rows[index].supplied_id == Some(supplied_id))
                && !rows[index].applied.iter().any(|(accepted, _)| accepted == cookie)
        })
    })
}

pub(in crate::tests) fn mark_fixture_paper_answers(
    paper: &IncomeSource,
    ordinal: usize,
    total: usize,
    cookie: &str,
    id: i64,
    supplied_id: i64,
) {
    PAPER_ANSWERS.with(|rows| {
        let mut rows = rows.borrow_mut();
        if let Some(index) = paper_fixture_index(&rows, paper, ordinal, total, supplied_id)
            && !rows[index]
                .applied
                .iter()
                .any(|(accepted, source)| accepted == cookie && *source == id)
        {
            rows[index].applied.push((cookie.into(), id));
        }
    });
}

// Bind newly supplied answers at row construction, before equal printed papers
// lose the distinction between their separate accepted source identities.
pub(in crate::tests) fn bind_fixture_paper_answers(paper: &IncomeSource, id: i64) {
    if id == 0 {
        return;
    }
    PAPER_ANSWERS.with(|rows| {
        if let Some(row) = rows.borrow_mut().iter_mut().rev().find(|row| {
            row.papers.contains(paper) && row.supplied_id.is_none() && row.applied.is_empty()
        }) {
            row.supplied_id = Some(id);
        }
    });
    crate::tests::support::client::income_pages::bind_fixture_sa_answers(paper, id);
}

pub(super) fn remember(paper: IncomeSource, answers: Vec<crate::DocumentAnswer>) {
    PAPER_ANSWERS.with(|rows| {
        rows.borrow_mut().push(PaperFixtureAnswers {
            papers: vec![paper],
            answers,
            applied: Vec::new(),
            supplied_id: None,
        });
    });
}

pub(in crate::tests) fn rebind_paper_answers(before: &IncomeSource, after: &IncomeSource) {
    rebind_paper_answers_at(before, after, 0, 0, 1);
}

pub(in crate::tests) fn rebind_paper_answers_at(
    before: &IncomeSource,
    after: &IncomeSource,
    supplied_id: i64,
    ordinal: usize,
    total: usize,
) {
    if before != after {
        PAPER_ANSWERS.with(|rows| {
            let mut rows = rows.borrow_mut();
            // Normalize the same registration that the topic-answer replay
            // selects, keeping equal papers in their supplied source order.
            if let Some(index) = paper_fixture_index(&rows, before, ordinal, total, supplied_id)
                && !rows[index].papers.contains(after)
            {
                rows[index].papers.push(after.clone());
            }
        });
    }
    if let (
        IncomeSource::OtherIncome {
            value:
                before @ (OtherIncomeDocument::Sa1099(_) | OtherIncomeDocument::HsaTrusteeStatement(_)),
        },
        IncomeSource::OtherIncome {
            value:
                after @ (OtherIncomeDocument::Sa1099(_) | OtherIncomeDocument::HsaTrusteeStatement(_)),
        },
    ) = (before, after)
    {
        crate::tests::support::client::income_pages::rebind_sa_answers(
            before,
            after,
            supplied_id,
            ordinal,
            total,
        );
    }
    if let (IncomeSource::Farming(before), IncomeSource::Farming(after)) = (before, after)
        && let Some(answers) = fixture_cooperative_answers(before)
    {
        remember_cooperative_answers(after.clone(), answers);
    }
    if let (
        IncomeSource::K1 { value: K1Document::K1EstateTrust(before) },
        IncomeSource::K1 { value: K1Document::K1EstateTrust(after) },
    ) = (before, after)
    {
        super::form_1041_k1::rebind_estate_fixture_answers(before, after);
    }
    if let (
        IncomeSource::K1 { value: K1Document::K1Scorp(before) },
        IncomeSource::K1 { value: K1Document::K1Scorp(after) },
    ) = (before, after)
    {
        super::form_1120_s_k1::rebind_scorp_fixture_answers(before, after);
    }
}

// A reopened Document center row carries the answers its accepted source holds:
// a page row no fixture registered keeps them for the answer step that follows.
pub(in crate::tests) fn remember_accepted_answers(draft: &FederalDraft, page: &P) {
    accepted_answers(draft, page, false);
}
// Resubmitting a reopened page after a refused edit of the same paper: its rows
// carry the accepted answers again, whatever the refused edit registered.
pub(in crate::tests) fn restore_accepted_answers(draft: &FederalDraft, page: &P) {
    accepted_answers(draft, page, true);
}
fn accepted_answers(draft: &FederalDraft, page: &P, force: bool) {
    let P::SaveDocuments { documents, .. } = page else { return };
    let accepted = |id: i64| draft.income.iter().find(|entry| id != 0 && entry.id == id);
    for row in &documents.common.k1.documents {
        match (&row.document, accepted(row.id).map(|entry| &entry.income)) {
            (K1Document::K1Scorp(document), Some(FederalIncome::Scorp { data })) => {
                if force {
                    remember_scorp_document(document.clone(), data.personal.clone());
                } else {
                    remember_reopened_scorp_document(document.clone(), data.personal.clone());
                }
            }
            (
                K1Document::K1EstateTrust(document),
                Some(FederalIncome::EstateTrustPortfolio { data }),
            ) if force || !super::form_1041_k1::fixture_estate_answers_supplied(document) => {
                // A topic save can change personal answers without changing the paper.
                // Refresh accepted snapshots while preserving explicitly supplied edits.
                remember_estate_answers(document.clone(), data.clone(), None, false);
            }
            _ => {}
        }
    }
    for row in &documents.common.investment_sales.documents {
        if let Some(FederalIncome::OpportunityFund { data }) =
            accepted(row.id).map(|entry| &entry.income)
            && (force
                || !fixture_paper_has_answers(&IncomeSource::InvestmentSale {
                    value: row.document.clone(),
                }))
        {
            qof_document(data.clone());
        }
    }
    for row in &documents.other.farming.documents {
        if let Some(FederalIncome::Cooperative { data }) =
            accepted(row.id).map(|entry| &entry.income)
            && (force || fixture_cooperative_answers(&row.document).is_none())
        {
            remember_cooperative_answers(row.document.clone(), data.more.clone());
        }
    }
}

// Carry explicit native fixture answers through the separate source and topic saves.
pub(in crate::tests) fn qof_document(data: QofInvestment) -> QofInvestmentPaper {
    let paper = QofInvestmentPaper::from(&data);
    remember(
        IncomeSource::InvestmentSale {
            value: InvestmentSaleDocument::OpportunityFund(paper.clone()),
        },
        vec![
            DocumentAnswer::OpportunityFundEvent { value: data.event },
            DocumentAnswer::OpportunityFundSpecial { value: data.special },
        ],
    );
    paper
}
