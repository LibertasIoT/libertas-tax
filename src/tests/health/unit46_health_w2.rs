// W-2 source/year allocation acceptance through the normal public protocol.
use crate::tests::cross::interview::{
    accepted, hsa_account, hsa_page_value, open_hsa_page, open_topic, rejected_with,
};
use super::*;

fn health_value(page: &mut P) -> &mut HsaPage {
    let P::SaveHsa { value, .. } = page else {
        panic!("expected the normal Health accounts editor")
    };
    value
}

fn allocation(page: &mut P, document: i64, kind: HealthEmployerKind, year: HealthEmployerYear) {
    let rows = &mut health_value(page).employer_sources;
    let row = rows
        .iter_mut()
        .find(|row| row.document == document && row.kind == kind)
        .expect("expected the actual supplied W-2 source row");
    row.allocation = year;
}

fn box12(draft: &mut FederalDraft, document: i64, code: W2Code, amount: i64) {
    let entry = draft
        .income
        .iter_mut()
        .find(|entry| entry.id == document)
        .unwrap();
    let FederalIncome::Wage { data } = &mut entry.income else {
        panic!("expected actual accepted wage source")
    };
    data.other_box12.push(W2CodeAmount { code, amount });
}

fn change_reported(page: &mut P, document: i64, code: W2Code, amount: i64) {
    let P::SaveIncomes { documents, .. } = page else {
        panic!("expected normal Wages source editor")
    };
    let row = documents
        .documents
        .iter_mut()
        .find(|row| row.id == document)
        .unwrap();
    let IncomeDocument::W2(data) = &mut row.document else {
        panic!("expected actual W-2 page")
    };
    data.more
        .box12
        .iter_mut()
        .find(|entry| entry.code == code)
        .unwrap()
        .amount = amount;
}

fn needs_year(draft: &FederalDraft) {
    let result = estimate(draft);
    assert_ne!(result.state, ResultState::EstimatedForSupportedScope);
    assert!(result.tax.is_none() && result.refund.is_none());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "HEALTH_EMPLOYER_YEAR_REQUIRED")
    );
}

// Normal immutable native callback bridge for a future actual Studio editor.
// Readonly source membership/identity/gross must come from SEED, not from the
// answered native request. Fresh accounts use editor List templates; employer
// means the editable authoritative amount OUTSIDE those entered W-2 sources.
fn client_artifacts(session: &mut Interview, seed: &P, request: &mut P, case: &str) -> bool {
    extern crate std;
    if std::env::var("TAX_UNIT46_W2_CASE").as_deref() != Ok(case) {
        return false;
    }
    if let Ok(input) = std::env::var("TAX_UNIT46_W2_CALC_INPUT") {
        let calculation = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        assert!(matches!(
            calculation,
            P::HsaOptions { .. } | P::EstimateHsa { .. }
        ));
        let before = session.draft.clone();
        let Reply::Response(reply) = send(session, calculation) else {
            panic!("expected the ordinary Health accounts calculation reply")
        };
        assert!(matches!(
            reply,
            P::HsaFamilyOptions { .. } | P::RunningEstimate { .. }
        ));
        assert_eq!(session.draft, before);
        std::fs::write(
            std::env::var("TAX_UNIT46_W2_CALC_OUTPUT").unwrap(),
            reply.to_avro(),
        )
        .unwrap();
        return true;
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_W2_SEED") {
        std::fs::write(path, seed.to_avro()).unwrap();
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_W2_ANSWERS") {
        std::fs::write(path, request.to_avro()).unwrap();
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_W2_PAYLOAD") {
        *request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(request, P::SaveHsa { .. }));
    }
    false
}

fn capture_reopen(session: &mut Interview, case: &str) -> P {
    extern crate std;
    let reopened = open_hsa_page(session);
    if std::env::var("TAX_UNIT46_W2_CASE").as_deref() == Ok(case)
        && let Ok(path) = std::env::var("TAX_UNIT46_W2_REOPEN")
    {
        std::fs::write(path, reopened.to_avro()).unwrap();
    }
    reopened
}

// Independent source arithmetic, not an IRS worked complete-return fixture.
// i8889 Employer Contribution Worksheet (2025 final /2026 draft): code W paid
// this calendar year less prior-designated amounts, plus later current-designated
// deposits. Original current principal =1000+500+200 outside=1700; timely
// employer principal300 is removed once, leaving1400. The prior gross4500 is
// not taxable excess; only the separately documented prior excess100 enters
// income, along with corrected employer principal300 not already in wages.
// Personal500 plus retained prior100 absorbed by unused2026 room yields a600
// deduction =>90000+300+100-600=89800 AGI. The retained2025 line48=100 and
// line49=6 establish that carry; the separately dated2026 deposit does not
// manufacture it. Signed loss-10 is not
// another principal subtraction or an invented negative-income deduction.
// https://www.irs.gov/pub/irs-dft/i8889--dft.pdf (Line9 / excess employer)
// https://www.irs.gov/pub/irs-pdf/i8889.pdf (same final2025 year worksheet)
#[test]
fn unit46_w2_hsa_current_prior_split_outside_and_corrections_save_reopen() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, wage(2, Owner::Taxpayer, 2_000_000, 0));
    add(&mut draft, wage(3, Owner::Taxpayer, 1_000_000, 0));
    box12(&mut draft, 1, W2Code::W, 100_000);
    box12(&mut draft, 2, W2Code::W, 200_000);
    box12(&mut draft, 3, W2Code::W, 300_000);
    let mut session = Interview::new(TEST_USER, draft);
    needs_year(&session.draft);
    let mut page = open_hsa_page(&mut session);
    let seed = page.clone();
    assert_eq!(hsa_page_value(&page).0.employer_sources.len(), 3);
    allocation(
        &mut page,
        1,
        HealthEmployerKind::Hsa,
        HealthEmployerYear::CurrentYear,
    );
    allocation(
        &mut page,
        2,
        HealthEmployerKind::Hsa,
        HealthEmployerYear::PriorYear,
    );
    allocation(
        &mut page,
        3,
        HealthEmployerKind::Hsa,
        HealthEmployerYear::Split { current: 50_000 },
    );
    let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000);
    account.employer = 20_000; // Outside entered W-2s, not another copy of1700.
    account.corrections = vec![HsaCorrectionPage {
        document: None,
        failed_deposit: None,
        year: 2026,
        contributed_on: 20260601,
        principal: 30_000,
        earnings: -1_000,
        returned: 20261001,
        deadline: 20271015,
        payer: HsaCorrectionPayer::Employer { in_wages: false },
    }];
    account.prior_employer_excess = vec![HsaPriorEmployerExcessDeposit {
        contributed_on: 20260301,
        deadline: 20260415,
        amount: 10_000,
        in_wages: false,
    }];
    account.excess_accounting = Some(HsaExcessAccounting {
        prior_year: Some(HsaPriorExcess {
            line_48: 10_000,
            line_49: 600,
        }),
        employer_excess_in_wages: 0,
        year_end_value: 200_000,
        later_return_year_deposits: 20_000,
        later_employer_deposits: 20_000,
    });
    let input = health_value(&mut page);
    input.accounts = vec![account.clone()];
    input.family_split = FamilySplit::NotApplicable;
    if client_artifacts(&mut session, &seed, &mut page, "hsa") {
        return;
    }
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let adjustments = saved.adjustments.as_ref().unwrap();
    assert_eq!(adjustments.health_employer_w2.len(), 3);
    assert_eq!(adjustments.hsa[0].employer_outside, 20_000);
    assert_eq!(adjustments.hsa[0].employer, 170_000);
    let result = supported(&saved);
    assert_eq!(value(&result, "hsa_1_contribution_employer"), 1400);
    assert_eq!(value(&result, "hsa_1_corrected_contributions"), 300);
    assert_eq!(
        value(&result, "hsa_correction_employer_principal_other_income"),
        300
    );
    assert_eq!(
        value(&result, "hsa_1_prior_designated_employer_excess_income"),
        100
    );
    assert_eq!(value(&result, "hsa_1_contribution_deduction"), 600);
    assert_eq!(value(&result, "agi"), 89_800);
    let reopened = capture_reopen(&mut session, "hsa");
    assert_eq!(hsa_page_value(&reopened).0.accounts, vec![account]);
    assert_eq!(
        hsa_page_value(&reopened).0.employer_sources[2].allocation,
        HealthEmployerYear::Split { current: 50_000 }
    );
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), result);
    assert_eq!(
        hsa_page_value(&open_hsa_page(&mut restored)).0,
        hsa_page_value(&reopened).0
    );
    let mut invalid = open_hsa_page(&mut restored);
    health_value(&mut invalid).accounts[0].corrections[0].principal = 170_001;
    rejected_with(
        send(&mut restored, invalid),
        "HSA_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED",
    );
    assert_eq!(restored.draft, saved);
    let mut invalid = open_hsa_page(&mut restored);
    health_value(&mut invalid).accounts[0]
        .excess_accounting
        .as_mut()
        .unwrap()
        .later_employer_deposits = 20_001;
    rejected_with(
        send(&mut restored, invalid),
        "HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER",
    );
    assert_eq!(restored.draft, saved);
}

// Existing source-grounded prior-carry fixture extended with actual code R:
// current1000+outside200 leaves2600-1200=1400 unused contribution room;
// prior2000 therefore leaves600 and36 tax. Current source is counted once.
// https://www.irs.gov/pub/irs-dft/i5329--dft.pdf (PartVI line35)
// https://www.irs.gov/pub/irs-dft/i8853--dft.pdf (Line1)
#[test]
fn unit46_w2_archer_current_and_outside_prior_carry_save_reopen() {
    let mut draft = complete(FederalStatus::Single);
    box12(&mut draft, 1, W2Code::R, 100_000);
    box12(&mut draft, 1, W2Code::W, 0);
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_hsa_page(&mut session);
    let seed = page.clone();
    assert_eq!(hsa_page_value(&page).0.employer_sources.len(), 1);
    allocation(
        &mut page,
        1,
        HealthEmployerKind::Archer,
        HealthEmployerYear::CurrentYear,
    );
    let row = ArcherContributorRow {
        excess_accounting: Some(ArcherExcessAccounting {
            prior_year: Some(ArcherPriorExcess { line_40: 200_000, line_41: 12_000 }),
            year_end_value: 300_000,
            later_return_year_deposits: 5_000,
            later_employer_deposits: 5_000,
            employer_excess_in_wages: 0,
        }),
        ..archer_contributor_row(
            Owner::Taxpayer,
            0,
            20_000,
            ArcherEligibility::FullYear { coverage: archer_self(4000) },
            vec![1],
        )
    };
    health_value(&mut page).archer = ArcherContributionsPage::Records {
        owners: vec![row.clone()],
        sharing: ArcherSharing::Equal,
    };
    if client_artifacts(&mut session, &seed, &mut page, "archer") {
        return;
    }
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let ArcherContributions::Records { owners, .. } = &saved.adjustments.as_ref().unwrap().archer
    else {
        panic!("expected accepted Archer contributor")
    };
    assert_eq!(owners[0].employer_outside, 20_000);
    assert_eq!(owners[0].employer, 120_000);
    let result = supported(&saved);
    assert_eq!(value(&result, "archer_prior_excess_deducted_return_year"), 1400);
    assert_eq!(value(&result, "archer_prior_excess_remaining_following_year"), 600);
    assert_eq!(value(&result, "archer_return_year_excess_contribution_tax"), 36);
    assert_eq!(value(&result, "archer_msa_deduction"), 1400);
    assert_eq!(value(&result, "agi"), 58_600);
    let reopened = capture_reopen(&mut session, "archer");
    assert_eq!(
        hsa_page_value(&reopened).0.archer,
        ArcherContributionsPage::Records {
            owners: vec![row],
            sharing: ArcherSharing::Equal,
        }
    );
    let mut restored = restart(saved.clone());
    assert_eq!(supported(&restored.draft), result);
    assert_eq!(
        hsa_page_value(&open_hsa_page(&mut restored)).0,
        hsa_page_value(&reopened).0
    );
    let mut invalid = open_hsa_page(&mut restored);
    let ArcherContributionsPage::Records { owners, .. } = &mut health_value(&mut invalid).archer
    else {
        unreachable!()
    };
    owners[0].income = vec![9999];
    rejected_with(send(&mut restored, invalid), "ARCHER_INCOME_NOT_OFFERED");
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit46_w2_supplied_source_membership_and_gross_are_atomic() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, wage(2, Owner::Taxpayer, 1_000_000, 0));
    box12(&mut draft, 1, W2Code::W, 100_000);
    box12(&mut draft, 2, W2Code::W, 50_000);
    for source in &mut draft.income {
        source.label.clear(); // Canonical fallback must distinguish unnamed W-2 IDs.
    }
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_hsa_page(&mut session);
    for source in &mut health_value(&mut page).employer_sources {
        source.allocation = HealthEmployerYear::CurrentYear;
    }
    let input = health_value(&mut page);
    input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    input.family_split = FamilySplit::NotApplicable;
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let result = supported(&saved);
    let reopened = open_hsa_page(&mut session);
    let source_rows = &hsa_page_value(&reopened).0.employer_sources;
    assert!(!source_rows[0].label.is_empty());
    assert_ne!(source_rows[0].label, source_rows[1].label);
    for (case, key) in [
        (0, "HEALTH_EMPLOYER_SOURCE_NOT_OFFERED"),
        (1, "HEALTH_EMPLOYER_SOURCE_CHANGED"),
        (2, "HEALTH_EMPLOYER_SOURCE_NOT_OFFERED"),
        (3, "HEALTH_EMPLOYER_SOURCE_CHANGED"),
        (4, "HEALTH_EMPLOYER_SOURCE_CHANGED"),
        (5, "HEALTH_EMPLOYER_SOURCE_TWICE"),
        (6, "HEALTH_EMPLOYER_SOURCE_NOT_OFFERED"),
        (7, "HEALTH_EMPLOYER_SPLIT_ABOVE_REPORTED"),
    ] {
        let mut invalid = open_hsa_page(&mut session);
        let rows = &mut health_value(&mut invalid).employer_sources;
        match case {
            0 => rows[0].document = 9999,
            1 => rows[0].owner = Owner::Spouse,
            2 => rows[0].kind = HealthEmployerKind::Archer,
            3 => rows[0].reported += 1,
            4 => rows[0].label = b"Forged source label".to_vec(),
            5 => rows[1] = rows[0].clone(),
            6 => {
                rows.pop();
            }
            7 => {
                rows[0].allocation = HealthEmployerYear::Split {
                    current: rows[0].reported + 1,
                }
            }
            _ => unreachable!(),
        }
        rejected_with(send(&mut session, invalid), key);
        assert_eq!(
            session.draft, saved,
            "case {case} changed accepted facts/IDs/revision"
        );
        assert_eq!(supported(&session.draft), result);
    }
}

#[test]
fn unit46_w2_cross_kind_unknown_and_all_prior_need_no_artificial_account() {
    let mut draft = complete(FederalStatus::Single);
    // The same W-2 ID has two distinct code-kind source relationships.
    box12(&mut draft, 1, W2Code::W, 100_000);
    box12(&mut draft, 1, W2Code::R, 100_000);
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_hsa_page(&mut session);
    allocation(
        &mut page,
        1,
        HealthEmployerKind::Hsa,
        HealthEmployerYear::CurrentYear,
    );
    let input = health_value(&mut page);
    input.accounts = vec![hsa_account(
        Owner::Taxpayer,
        HsaCoverageYear::SelfOnly,
        50_000,
    )];
    input.family_split = FamilySplit::NotApplicable;
    accepted(&send(&mut session, page));
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().hsa[0].employer,
        100_000
    );
    assert_eq!(
        session
            .draft
            .adjustments
            .as_ref()
            .unwrap()
            .health_employer_w2
            .len(),
        1
    );
    needs_year(&session.draft); // Unrelated Archer Unknown cannot complete tax.
    let mut page = open_hsa_page(&mut session);
    allocation(
        &mut page,
        1,
        HealthEmployerKind::Archer,
        HealthEmployerYear::PriorYear,
    );
    accepted(&send(&mut session, page));
    assert_eq!(value(&supported(&session.draft), "agi"), 59_500);
    assert!(matches!(
        session.draft.adjustments.as_ref().unwrap().archer,
        ArcherContributions::None
    ));
    let mut page = open_hsa_page(&mut session);
    allocation(
        &mut page,
        1,
        HealthEmployerKind::Hsa,
        HealthEmployerYear::PriorYear,
    );
    health_value(&mut page).accounts.clear();
    accepted(&send(&mut session, page));
    assert!(session.draft.adjustments.as_ref().unwrap().hsa.is_empty());
    assert_eq!(value(&supported(&session.draft), "agi"), 60_000);
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let reopened = open_hsa_page(&mut restored);
    assert!(hsa_page_value(&reopened).0.accounts.is_empty());
    assert!(
        hsa_page_value(&reopened)
            .0
            .employer_sources
            .iter()
            .all(|row| row.allocation == HealthEmployerYear::PriorYear)
    );
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit46_w2_split_source_edit_and_delete_require_explicit_clear_then_reanswer() {
    let mut draft = complete(FederalStatus::Joint);
    add(&mut draft, wage(2, Owner::Taxpayer, 1_000_000, 0));
    box12(&mut draft, 1, W2Code::W, 100_000);
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_hsa_page(&mut session);
    allocation(
        &mut page,
        1,
        HealthEmployerKind::Hsa,
        HealthEmployerYear::Split { current: 40_000 },
    );
    let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000);
    account.employer = 10_000;
    let input = health_value(&mut page);
    input.accounts = vec![account];
    input.family_split = FamilySplit::NotApplicable;
    input.other_coverage = Some(HsaEligibility::FullYear { coverage: HsaCoverage::None });
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    let result = supported(&saved);
    assert_eq!(saved.adjustments.as_ref().unwrap().hsa[0].employer, 50_000);
    for case in 0..5 {
        let mut wages = open_topic(&mut session, TaxSection::Wages);
        let key = if case == 0 || case == 4 {
            "HEALTH_EMPLOYER_SOURCE_CHANGED"
        } else {
            "HEALTH_EMPLOYER_SOURCE_REFERENCED"
        };
        if case == 0 || case == 4 {
            let amount = if case == 0 { 120_000 } else { 0 };
            change_reported(&mut wages, 1, W2Code::W, amount);
        } else {
            let P::SaveIncomes { documents, .. } = &mut wages else { unreachable!() };
            if case == 1 {
                documents.documents.retain(|row| row.id != 1);
            } else {
                let row = documents.documents.iter_mut().find(|row| row.id == 1).unwrap();
                if case == 2 {
                    row.owner = Owner::Spouse;
                } else {
                    let IncomeDocument::W2(data) = &mut row.document else { unreachable!() };
                    data.more
                        .box12
                        .iter_mut()
                        .find(|entry| entry.code == W2Code::W)
                        .unwrap()
                        .code = W2Code::R;
                }
            }
        }
        let (name, path) = match case {
            0 => ("health-context-19", Some("[documents/documents/0/document]")),
            1 => ("health-context-20", None),
            2 => ("health-context-21", Some("[documents/documents/0/owner]")),
            3 => ("health-context-22", Some("[documents/documents/0/document]")),
            4 => ("health-context-23", Some("[documents/documents/0/document]")),
            _ => unreachable!(),
        };
        rejected_with(
            current_source_refusal(name, &mut session, wages, key, path, SourceScope::Health),
            key,
        );
        assert_eq!(session.draft, saved);
        assert_eq!(supported(&session.draft), result);
    }
    // A real edit to Unknown clears only the source binding; actual contributor
    // and outside/personal facts survive. Unknown does not justify a tax total.
    let mut health = open_hsa_page(&mut session);
    allocation(&mut health, 1, HealthEmployerKind::Hsa, HealthEmployerYear::Unknown);
    accepted(&send(&mut session, health));
    assert!(session.draft.adjustments.as_ref().unwrap().health_employer_w2.is_empty());
    assert_eq!(session.draft.adjustments.as_ref().unwrap().hsa[0].personal, 50_000);
    assert_eq!(session.draft.adjustments.as_ref().unwrap().hsa[0].employer_outside, 10_000);
    assert_eq!(session.draft.adjustments.as_ref().unwrap().hsa[0].employer, 10_000);
    needs_year(&session.draft);
    let mut wages = open_topic(&mut session, TaxSection::Wages);
    change_reported(&mut wages, 1, W2Code::W, 120_000);
    accepted(&send(&mut session, wages));
    let mut health = open_hsa_page(&mut session);
    assert_eq!(hsa_page_value(&health).0.employer_sources[0].reported, 120_000);
    assert_eq!(
        hsa_page_value(&health).0.employer_sources[0].allocation,
        HealthEmployerYear::Unknown
    );
    allocation(
        &mut health,
        1,
        HealthEmployerKind::Hsa,
        HealthEmployerYear::Split { current: 50_000 },
    );
    accepted(&send(&mut session, health));
    assert_eq!(session.draft.adjustments.as_ref().unwrap().hsa[0].employer, 60_000);
    assert_eq!(
        session.draft.adjustments.as_ref().unwrap().health_employer_w2[0].accepted_amount,
        120_000
    );
    let saved = session.draft.clone();
    assert_eq!(supported(&restart(saved.clone()).draft), supported(&saved));
    let mut health = open_hsa_page(&mut session);
    allocation(&mut health, 1, HealthEmployerKind::Hsa, HealthEmployerYear::Unknown);
    accepted(&send(&mut session, health));
    let mut wages = open_topic(&mut session, TaxSection::Wages);
    let P::SaveIncomes { documents, .. } = &mut wages else { unreachable!() };
    documents.documents.retain(|row| row.id != 1);
    accepted(&send(&mut session, wages));
    let health = open_hsa_page(&mut session);
    assert!(hsa_page_value(&health).0.employer_sources.is_empty());
    assert_eq!(hsa_page_value(&health).0.accounts[0].employer, 10_000);
    assert_eq!(value(&supported(&session.draft), "agi"), 9500);
}

#[test]
fn unit46_w2_whole_year_amount_corrections_rederive_and_preserve_outside() {
    for (year, expected) in [
        (HealthEmployerYear::CurrentYear, 160_000),
        (HealthEmployerYear::PriorYear, 10_000),
    ] {
        let mut draft = complete(FederalStatus::Single);
        box12(&mut draft, 1, W2Code::W, 100_000);
        let mut session = Interview::new(TEST_USER, draft);
        let mut health = open_hsa_page(&mut session);
        allocation(&mut health, 1, HealthEmployerKind::Hsa, year.clone());
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000);
        account.employer = 10_000;
        let input = health_value(&mut health);
        input.accounts = vec![account];
        input.family_split = FamilySplit::NotApplicable;
        accepted(&send(&mut session, health));
        let mut wages = open_topic(&mut session, TaxSection::Wages);
        change_reported(&mut wages, 1, W2Code::W, 150_000);
        accepted(&send(&mut session, wages));
        let saved = session.draft.clone();
        assert_eq!(
            saved.adjustments.as_ref().unwrap().hsa[0].employer,
            expected
        );
        assert_eq!(
            saved.adjustments.as_ref().unwrap().hsa[0].employer_outside,
            10_000
        );
        let mut restored = restart(saved.clone());
        let health = open_hsa_page(&mut restored);
        assert_eq!(
            hsa_page_value(&health).0.employer_sources[0].allocation,
            year
        );
        assert_eq!(
            hsa_page_value(&health).0.employer_sources[0].reported,
            150_000
        );
        assert_eq!(hsa_page_value(&health).0.accounts[0].employer, 10_000);
        assert_eq!(supported(&restored.draft), supported(&saved));
        // A real zero code amount is still a present source for existing whole
        // classifications, but needs no question and contributes nothing.
        let mut wages = open_topic(&mut restored, TaxSection::Wages);
        change_reported(&mut wages, 1, W2Code::W, 0);
        accepted(&send(&mut restored, wages));
        assert_eq!(
            restored.draft.adjustments.as_ref().unwrap().hsa[0].employer,
            10_000
        );
        let health = open_hsa_page(&mut restored);
        assert!(hsa_page_value(&health).0.employer_sources.is_empty());
        assert_eq!(value(&supported(&restored.draft), "agi"), 59_500);
    }
}

// Same approved cap2600/prior2000 control: prior-only code R does not enter
// current room. Outside1000 =>deduction1600/carry400/tax24. Splitting actual
// code R1000 into current400/prior600 changes current total to1400, giving
// deduction1200/carry800/tax48. No prior gross is invented as taxable excess.
#[test]
fn unit46_w2_archer_prior_and_split_do_not_repeat_manual_outside_principal() {
    for (year, employer, deduction, carry, tax) in [
        (HealthEmployerYear::PriorYear, 100_000, 1600, 400, 24),
        (
            HealthEmployerYear::Split { current: 40_000 },
            140_000,
            1200,
            800,
            48,
        ),
    ] {
        let mut draft = complete(FederalStatus::Single);
        box12(&mut draft, 1, W2Code::R, 100_000);
        let mut session = Interview::new(TEST_USER, draft);
        let mut health = open_hsa_page(&mut session);
        allocation(&mut health, 1, HealthEmployerKind::Archer, year.clone());
        let row = ArcherContributorRow {
            excess_accounting: Some(ArcherExcessAccounting {
                prior_year: Some(ArcherPriorExcess { line_40: 200_000, line_41: 12_000 }),
                year_end_value: 300_000,
                later_return_year_deposits: 0,
                later_employer_deposits: 0,
                employer_excess_in_wages: 0,
            }),
            ..archer_contributor_row(
                Owner::Taxpayer,
                0,
                100_000,
                ArcherEligibility::FullYear { coverage: archer_self(4000) },
                vec![1],
            )
        };
        health_value(&mut health).archer = ArcherContributionsPage::Records {
            owners: vec![row.clone()],
            sharing: ArcherSharing::Equal,
        };
        accepted(&send(&mut session, health));
        let saved = session.draft.clone();
        let ArcherContributions::Records { owners, .. } =
            &saved.adjustments.as_ref().unwrap().archer
        else {
            unreachable!()
        };
        assert_eq!(owners[0].employer, employer);
        assert_eq!(owners[0].employer_outside, 100_000);
        let result = supported(&saved);
        assert_eq!(
            value(&result, "archer_prior_excess_deducted_return_year"),
            deduction
        );
        assert_eq!(value(&result, "archer_prior_excess_remaining_following_year"), carry);
        assert_eq!(value(&result, "archer_return_year_excess_contribution_tax"), tax);
        assert_eq!(value(&result, "agi"), 60_000 - deduction);
        let mut restored = restart(saved.clone());
        assert_eq!(supported(&restored.draft), result);
        let reopened = open_hsa_page(&mut restored);
        assert_eq!(
            hsa_page_value(&reopened).0.archer,
            ArcherContributionsPage::Records {
                owners: vec![row],
                sharing: ArcherSharing::Equal,
            }
        );
        assert_eq!(
            hsa_page_value(&reopened).0.employer_sources[0].allocation,
            year
        );
    }
}

#[test]
fn unit46_w2_zero_codes_do_not_create_a_year_question() {
    let mut draft = complete(FederalStatus::Single);
    box12(&mut draft, 1, W2Code::W, 0);
    box12(&mut draft, 1, W2Code::R, 0);
    let mut session = Interview::new(TEST_USER, draft);
    let health = open_hsa_page(&mut session);
    assert!(hsa_page_value(&health).0.employer_sources.is_empty());
    accepted(&send(&mut session, health));
    assert!(
        session
            .draft
            .adjustments
            .as_ref()
            .unwrap()
            .health_employer_w2
            .is_empty()
    );
    assert_eq!(value(&supported(&session.draft), "agi"), 60_000);
}

#[test]
fn unit46_w2_actual_employer_carrier_rejects_both_accounts_atomically() {
    for kind in [HealthEmployerKind::Hsa, HealthEmployerKind::Archer] {
        let mut draft = complete(FederalStatus::Single);
        let code = match kind {
            HealthEmployerKind::Hsa => W2Code::W,
            HealthEmployerKind::Archer => W2Code::R,
        };
        box12(&mut draft, 1, code, 100_000);
        let mut session = Interview::new(TEST_USER, draft);
        let mut page = open_hsa_page(&mut session);
        allocation(&mut page, 1, kind, HealthEmployerYear::CurrentYear);
        match kind {
            HealthEmployerKind::Hsa => {
                let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0);
                account.employer = 20_000;
                health_value(&mut page).accounts = vec![account];
                health_value(&mut page).family_split = FamilySplit::NotApplicable;
            }
            HealthEmployerKind::Archer => {
                health_value(&mut page).archer = ArcherContributionsPage::Records {
                    owners: vec![archer_contributor_row(
                        Owner::Taxpayer,
                        0,
                        20_000,
                        ArcherEligibility::FullYear { coverage: archer_self(4000) },
                        vec![1],
                    )],
                    sharing: ArcherSharing::Equal,
                };
            }
        }
        accepted(&send(&mut session, page));
        let saved = session.draft.clone();
        let original = open_hsa_page(&mut session);
        let expected = hsa_page_value(&original).0.clone();
        assert_eq!(expected.employer_sources[0].reported, 100_000);
        assert_eq!(
            expected.employer_sources[0].allocation,
            HealthEmployerYear::CurrentYear
        );
        let mut invalid = original;
        // The former arbitrary employer ceiling is redundant over the existing
        // checked sum. Only the actual native carrier refuses MAX plus this W-2.
        assert_eq!(
            crate::health_employer::effective_total(
                &session.draft,
                crate::health_employer::bindings(&session.draft),
                Owner::Taxpayer,
                kind,
                100_000_000_000,
            ),
            Ok(100_000_100_000),
        );
        match kind {
            HealthEmployerKind::Hsa => {
                health_value(&mut invalid).accounts[0].employer = i64::MAX;
            }
            HealthEmployerKind::Archer => {
                let ArcherContributionsPage::Records { owners, .. } =
                    &mut health_value(&mut invalid).archer
                else {
                    unreachable!()
                };
                owners[0].employer = i64::MAX;
            }
        }
        rejected_with(send(&mut session, invalid), "HEALTH_EMPLOYER_TOTAL_INVALID");
        assert_eq!(session.draft, saved);
        assert_eq!(hsa_page_value(&open_hsa_page(&mut session)).0, &expected);
    }
}
