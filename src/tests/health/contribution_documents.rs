use super::*;
use crate::tests::cross::interview::{accepted, hsa_account, open_hsa_page};
use crate::tests::support::builders::contribution_documents::{
    contribution_allocation, contribution_funding_return, contribution_statement,
};
use crate::tests::support::client::contribution_documents::{
    contribution_statements, save_contribution_allocations, save_contribution_statements,
};

#[test]
fn hsa_custodian_statements_share_existing_payroll_and_ira_owners_once() {
    let draft = contribution_funding_return();
    let funding = draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    let wage = draft.income[0].id;
    let mut a = contribution_statement(&draft, HealthContributionKind::Hsa, "A");
    a.box_2 = 175_000;
    a.box_3 = 25_000;
    a.box_5 = 300_000;
    let mut b = contribution_statement(&draft, HealthContributionKind::Hsa, "B");
    b.box_2 = 100_000;
    b.box_3 = 50_000;
    b.box_4 = 40_000;
    b.box_5 = 500_000;
    let mut session = restart(draft);
    save_contribution_statements(&mut session, vec![a, b]);
    let papers = contribution_statements(&mut session);
    assert!(papers.iter().all(|paper| paper.assigned == DocumentRecipient::Taxpayer));
    let mut allocations = vec![
        contribution_allocation(papers[0].id, 0, 75_000, 0),
        contribution_allocation(papers[1].id, 10_000, 0, 25_000),
    ];
    allocations[0].w2.push(HealthStatementW2Part { income: wage, amount: 75_000 });
    allocations[1].w2.push(HealthStatementW2Part { income: wage, amount: 25_000 });
    allocations[0].funding.push(HealthStatementFunding { income: funding, funded: 20260115 });
    save_contribution_allocations(&mut session, allocations);
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    assert_eq!(input.accounts[0].personal, 20_000);
    accepted(&send(&mut session, page));
    let result = supported(&session.draft);
    // $3,500 printed total minus $100 for2025, $1,000 payroll and $250 IRA
    // funding, plus $200 outside these papers, leaves $2,350 personal deposits.
    assert_eq!(value(&result, "adjustments"), 2_350);
    let stored = &session.draft.adjustments.as_ref().unwrap().hsa[0];
    assert_eq!(stored.personal, 235_000);
    assert_eq!(stored.personal_outside, 20_000);
    assert_eq!(stored.employer, 100_000);
    assert_eq!(stored.funding[0].amount, 25_000);
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let reopened = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &reopened else { unreachable!() };
    assert_eq!(input.accounts[0].personal, 20_000);
    assert_eq!(input.contribution_sources.len(), 2);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert_eq!(supported(&session.draft), result);
    // Remove only B: its source-owned split disappears, while independent
    // coverage, outside deposits, payroll and IRA funding remain established.
    let mut retained = contribution_statements(&mut session);
    retained.retain(|paper| paper.id == papers[0].id);
    retained[0].box_2 = 185_000;
    retained[0].corrected = true;
    save_contribution_statements(&mut session, retained);
    let adjusted = supported(&session.draft);
    assert_eq!(value(&adjusted, "adjustments"), 1_300);
    let adjustments = session.draft.adjustments.as_ref().unwrap();
    assert_eq!(adjustments.health_contribution_allocations.len(), 1);
    assert_eq!(
        adjustments.hsa[0].coverage,
        HsaEligibility::FullYear { coverage: HsaCoverage::SelfOnly }
    );
    assert_eq!(adjustments.hsa[0].funding[0].income, funding);
    assert_eq!(supported(&restart(session.draft.clone()).draft), adjusted);
    save_contribution_statements(&mut session, vec![]);
    assert_eq!(value(&supported(&session.draft), "adjustments"), 200);
    let adjustments = session.draft.adjustments.as_ref().unwrap();
    assert!(adjustments.health_contribution_allocations.is_empty());
    assert_eq!(adjustments.hsa[0].personal, 20_000);
    assert_eq!(adjustments.hsa[0].employer, 100_000);
    assert_eq!(adjustments.hsa[0].funding[0].amount, 25_000);
}

#[test]
fn printed_hsa_contributions_use_existing_timely_return_owner_after_correction() {
    let draft = complete(FederalStatus::Single);
    let mut paper = contribution_statement(&draft, HealthContributionKind::Hsa, "Corrected");
    paper.box_2 = 500_000;
    paper.box_5 = 700_000;
    let mut session = restart(draft);
    save_contribution_statements(&mut session, vec![paper]);
    let source = contribution_statements(&mut session)[0].id;
    let mut request = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut request else { unreachable!() };
    input.contribution_sources[0].allocation = contribution_allocation(source, 0, 0, 0).into();
    let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0);
    account.corrections.push(hsa_correction(2026, 20261231, 60_000, 5_000).page());
    input.accounts = vec![account];
    accepted(&send(&mut session, request));
    let first = supported(&session.draft);
    assert_eq!(value(&first, "adjustments"), 4_400);
    assert_eq!(value(&first, "hsa_correction_earnings"), 50);
    assert_eq!(value(&first, "hsa_1_corrected_contributions"), 600);
    let correction = session.draft.adjustments.as_ref().unwrap().hsa[0].corrections.clone();
    let mut papers = contribution_statements(&mut session);
    papers[0].box_2 = 480_000;
    papers[0].corrected = true;
    save_contribution_statements(&mut session, papers);
    let changed = supported(&session.draft);
    assert_eq!(value(&changed, "adjustments"), 4_200);
    assert_eq!(value(&changed, "hsa_correction_earnings"), 50);
    assert_eq!(session.draft.adjustments.as_ref().unwrap().hsa[0].corrections, correction);
    assert_eq!(supported(&restart(session.draft.clone()).draft), changed);
}

#[test]
fn archer_printed_personal_box_and_payroll_keep_employer_bar_and_excess_owner() {
    let mut draft = complete(FederalStatus::Single);
    let wage = draft.income[0].id;
    let FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source { form: data, .. }) } =
        &mut draft.income[0].income
    else {
        unreachable!()
    };
    data.box12.push(crate::PrintedBox12Entry { code: W2Code::R, amount: Some(50_000) });
    health_w2_year(&mut draft, HealthEmployerYear::CurrentYear);
    let mut paper = contribution_statement(&draft, HealthContributionKind::Archer, "Archer");
    paper.box_1 = 100_000;
    paper.box_2 = 150_000;
    paper.box_5 = 500_000;
    let mut session = restart(draft);
    save_contribution_statements(&mut session, vec![paper]);
    let source = contribution_statements(&mut session)[0].id;
    let mut request = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut request else { unreachable!() };
    let mut allocation = contribution_allocation(source, 0, 50_000, 0);
    allocation.w2.push(HealthStatementW2Part { income: wage, amount: 50_000 });
    input.contribution_sources[0].allocation = allocation.into();
    let mut owner = archer_contributor_row(
        Owner::Taxpayer,
        0,
        0,
        ArcherEligibility::FullYear { coverage: archer_self(4_000) },
        vec![wage],
    );
    owner.excess_accounting = Some(archer_excess_accounting(0, None, 0, 0, 0));
    input.archer =
        ArcherContributionsPage::Records { owners: vec![owner], sharing: ArcherSharing::Equal };
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "archer_msa_deduction"), 0);
    assert_eq!(value(&result, "archer_return_year_uncorrected_personal_excess"), 1_000);
    assert_eq!(value(&result, "archer_return_year_uncorrected_employer_excess"), 0);
    assert_eq!(value(&result, "archer_return_year_excess_contribution_tax"), 60);
    assert_eq!(supported(&restart(session.draft.clone()).draft), result);
}

#[test]
fn information_only_account_statements_retain_paper_without_contribution_questions() {
    for kind in [
        HealthContributionKind::Hsa,
        HealthContributionKind::Archer,
        HealthContributionKind::MedicareAdvantage,
    ] {
        let draft = complete(FederalStatus::Single);
        let mut original = supported(&draft);
        let mut paper = contribution_statement(&draft, kind, "Information only");
        paper.box_5 = 123_456;
        let mut session = restart(draft);
        save_contribution_statements(&mut session, vec![paper]);
        let saved = session.draft.clone();
        let papers = contribution_statements(&mut session);
        assert_eq!(papers[0].box_5, 123_456);
        assert_eq!(papers[0].kind, kind);
        assert!(saved.adjustments.as_ref().unwrap().hsa.is_empty());
        assert!(saved.adjustments.as_ref().unwrap().health_contribution_allocations.is_empty());
        session = restart(saved.clone());
        assert_eq!(contribution_statements(&mut session), papers);
        assert_eq!(session.draft, saved);
        let page = open_hsa_page(&mut session);
        let P::SaveHsa { value: input, .. } = &page else { unreachable!() };
        assert!(input.contribution_sources.is_empty());
        original.revision = session.draft.revision;
        assert_eq!(supported(&session.draft), original);
    }
}

#[test]
fn custodian_later_box_and_failed_rollover_share_one_actual_deposit() {
    use crate::tests::support::client::contribution_documents::failed_contribution_session;
    let draft = complete(FederalStatus::Single);
    let mut paper = contribution_statement(&draft, HealthContributionKind::Hsa, "Failed deposit");
    paper.box_3 = 100_000;
    paper.box_5 = 500_000;
    let mut session = failed_contribution_session(draft);
    let failed = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::MedicalDistribution { .. }))
        .unwrap()
        .id;
    save_contribution_statements(&mut session, vec![paper]);
    let source = contribution_statements(&mut session)[0].id;
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut page else { unreachable!() };
    input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    let mut allocation = contribution_allocation(source, 0, 0, 0);
    allocation.failed_deposits.push(HealthStatementDeposit { income: failed, deposited: 20270131 });
    input.contribution_sources[0].allocation = allocation.into();
    accepted(&send(&mut session, page));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "adjustments"), 1_000);
    assert_eq!(value(&result, "hsa_taxable_distributions"), 1_000);
    assert_eq!(value(&result, "hsa_additional_tax"), 200);
    assert_eq!(session.draft.adjustments.as_ref().unwrap().hsa[0].personal, 0);
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let reopened = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &reopened else { unreachable!() };
    assert_eq!(input.linked_deposits[0].id, failed);
    assert_eq!(input.contribution_sources[0].allocation.failed_deposits[0].income, failed);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert_eq!(supported(&session.draft), result);
}

#[test]
fn blank_participant_name_is_a_pending_item_added_in_its_section() {
    use crate::tests::support::builders::{add_documents, unknown_documents};
    use crate::tests::support::client::income_pages::open_document_center;
    let draft = complete(FederalStatus::Single);
    let mut paper =
        contribution_statement(&draft, HealthContributionKind::Hsa, "Blank participant");
    paper.participant.name.clear();
    paper.box_2 = 100_000;
    let mut session = restart(draft);
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, review, .. } = &mut request else { unreachable!() };
    *review = false;
    documents.common.health_accounts.documents.push(health_statement_source(paper));
    accepted(&send(&mut session, request));
    let stored = &session.draft.adjustments.as_ref().unwrap().health_statements[0];
    assert!(stored.form.participant.name.is_empty());
    assert_eq!(stored.assigned, crate::DocumentRecipient::Unknown);
    let source = stored.id;
    assert_eq!(unknown_documents(&session.draft), vec![(source, TaxSection::Documents)]);
    assert_eq!(estimate(&session.draft).state, ResultState::Incomplete);
    accepted(&add_documents(
        &mut session,
        TaxSection::Documents,
        -1,
        &[source],
        crate::AddedOwnership::Yours,
    ));
    let stored = &session.draft.adjustments.as_ref().unwrap().health_statements[0];
    assert_eq!(stored.assigned, DocumentRecipient::Taxpayer);
    assert_eq!(stored.owner_source, Some(crate::OwnerSource::Added));
    let mut request = open_hsa_page(&mut session);
    let P::SaveHsa { value: input, .. } = &mut request else { unreachable!() };
    input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    input.contribution_sources[0].allocation = contribution_allocation(source, 0, 0, 0).into();
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "adjustments"), 1_000);
    session = restart(session.draft);
    let papers = contribution_statements(&mut session);
    assert!(papers[0].participant.name.is_empty());
    assert_eq!(papers[0].assigned, DocumentRecipient::Taxpayer);
    save_contribution_statements(&mut session, papers);
    let stored = &session.draft.adjustments.as_ref().unwrap().health_statements[0];
    assert!(stored.form.participant.name.is_empty());
    assert_eq!(stored.assigned, DocumentRecipient::Taxpayer);
    let mut expected = result;
    expected.revision = session.draft.revision;
    assert_eq!(supported(&session.draft), expected);
}
