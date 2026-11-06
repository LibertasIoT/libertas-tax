use crate::tests::cross::interview::{accepted, open_hsa_page};
use crate::tests::support::client::income_pages::open_document_center;
use crate::tests::*;

pub(in crate::tests) fn save_contribution_statements(
    session: &mut Interview,
    statements: Vec<Form5498Sa>,
) {
    let mut request = open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.health_accounts.documents =
        statements.into_iter().map(health_statement_source).collect();
    accepted(&send(session, request));
}

pub(in crate::tests) fn contribution_statements(session: &mut Interview) -> Vec<Form5498Sa> {
    let P::SaveDocuments { documents, .. } = open_document_center(session) else { unreachable!() };
    documents.common.health_accounts.documents.iter().map(Form5498Sa::from).collect()
}

pub(in crate::tests) fn save_contribution_allocations(
    session: &mut Interview,
    allocations: Vec<HealthContributionAllocation>,
) {
    let mut request = open_hsa_page(session);
    let P::SaveHsa { value, .. } = &mut request else { unreachable!() };
    for row in &mut value.contribution_sources {
        if let Some(allocation) = allocations.iter().find(|answer| answer.source == row.id) {
            row.allocation = allocation.clone().into();
        }
    }
    accepted(&send(session, request));
}

pub(in crate::tests) fn failed_contribution_session(draft: FederalDraft) -> Interview {
    use crate::tests::support::builders::contribution_documents::{
        attempted_hsa_rollover, ordinary_hsa_deposit,
    };
    use crate::tests::support::client::income_pages::save_source_fixtures;
    let source = attempted_hsa_rollover(20261201, vec![ordinary_hsa_deposit(20270131, 1000, 2026)]);
    let document = named_document(&draft, -1, 0, source);
    let mut session = restart(draft);
    accepted(&save_source_fixtures(&mut session, vec![document]));
    session
}
