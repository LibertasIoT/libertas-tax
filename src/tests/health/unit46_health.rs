use super::*;
use crate::tests::cross::interview::{
    accepted, hsa_account, hsa_page_value, open_hsa_page, rejected_with,
};

// The optional artifact bridge lets the actual Studio editor compose a fresh
// row and return its ordinary Avro request to this same native lifecycle.
// Ordinary tests keep their independent complete inputs and assertions.
fn client_artifacts(session: &mut Interview, seed: &P, request: &mut P, case: &str) -> bool {
    extern crate std;
    if std::env::var("TAX_UNIT46_HEALTH_CASE").as_deref() != Ok(case) {
        return false;
    }
    if let Ok(input) = std::env::var("TAX_UNIT46_HEALTH_CALC_INPUT") {
        let calculation = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        assert!(matches!(calculation, P::HsaOptions { .. } | P::EstimateHsa { .. }));
        let before = session.draft.clone();
        let Reply::Response(reply) = send(session, calculation) else {
            panic!("expected normal HSA calculation reply")
        };
        assert!(matches!(reply, P::HsaFamilyOptions { .. } | P::RunningEstimate { .. }));
        assert_eq!(session.draft, before);
        std::fs::write(std::env::var("TAX_UNIT46_HEALTH_CALC_OUTPUT").unwrap(), reply.to_avro())
            .unwrap();
        return true;
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_HEALTH_SEED") {
        std::fs::write(path, seed.to_avro()).unwrap();
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_HEALTH_ANSWERS") {
        std::fs::write(path, request.to_avro()).unwrap();
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_HEALTH_PAYLOAD") {
        *request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(request, P::SaveHsa { .. }));
    }
    false
}

// Approved Unit20 fixture G, derived from Pub969's contribution eligibility,
// employer-excess wage reconciliation and normal distribution rules. Synthetic
// completion: an adult single filer, actual W-2 code W $1,000, personal $500,
// unrecovered excess and year-end account value $1,500, medical payout $200.
// https://www.irs.gov/publications/p969
#[test]
fn unit46_claimable_hsa_contributions_save_reopen_restart_and_reject_invalid_accounting() {
    for claimed in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        draft.people.as_mut().unwrap().taxpayer.dependency =
            FilerDependency::CanBeClaimed { claimed };
        let already_in_wages = claimed;
        draft.income[0] =
            wage(1, Owner::Taxpayer, if already_in_wages { 6_100_000 } else { 6_000_000 }, 0);
        let FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source { form: data, .. }) } =
            &mut draft.income[0].income
        else {
            panic!("expected actual wage source")
        };
        data.box12.push(crate::PrintedBox12Entry { code: W2Code::W, amount: Some(100_000) });
        add_hsa_distribution(&mut draft, Owner::Taxpayer, 20_000, vec![], None, None);
        let mut session = Interview::new(TEST_USER, draft);
        let mut page = open_hsa_page(&mut session);
        let seed = page.clone();
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000);
        account.employer = 0;
        account.excess_accounting = Some(HsaExcessAccounting {
            prior_year: None,
            employer_excess_in_wages: if already_in_wages { 100_000 } else { 0 },
            year_end_value: 150_000,
            later_return_year_deposits: 0,
            later_employer_deposits: 0,
        });
        let P::SaveHsa { value: inputs, .. } = &mut page else {
            panic!("expected actual HSA editor")
        };
        for source in &mut inputs.employer_sources {
            source.allocation = HealthEmployerYear::CurrentYear;
        }
        inputs.accounts = vec![account.clone()];
        inputs.family_split = FamilySplit::NotApplicable;
        if !claimed && client_artifacts(&mut session, &seed, &mut page, "hsa") {
            return;
        }
        accepted(&send(&mut session, page));
        let result = supported(&session.draft);
        assert_eq!(value(&result, "adjustments"), 0);
        assert_eq!(value(&result, "hsa_1_return_year_uncorrected_excess"), 500);
        assert_eq!(value(&result, "hsa_1_return_year_uncorrected_employer_excess"), 1000);
        assert_eq!(
            value(&result, "hsa_return_year_uncorrected_employer_excess_other_income"),
            if already_in_wages { 0 } else { 1000 }
        );
        assert_eq!(value(&result, "hsa_return_year_excess_contribution_tax"), 90);
        assert_eq!(value(&result, "hsa_taxable_distributions"), 0);
        assert_eq!(value(&result, "hsa_additional_tax"), 0);
        assert_eq!(value(&result, "agi"), 61_000);

        let reopened = open_hsa_page(&mut session);
        if !claimed {
            extern crate std;
            if std::env::var("TAX_UNIT46_HEALTH_CASE").as_deref() == Ok("hsa")
                && let Ok(path) = std::env::var("TAX_UNIT46_HEALTH_REOPEN")
            {
                std::fs::write(path, reopened.to_avro()).unwrap();
            }
        }
        assert_eq!(hsa_page_value(&reopened).0.accounts, vec![account.clone()]);
        let mut restarted = restart(session.draft.clone());
        assert_eq!(supported(&restarted.draft), result);
        assert_eq!(hsa_page_value(&open_hsa_page(&mut restarted)).0.accounts, vec![account]);

        let mut invalid = open_hsa_page(&mut session);
        let P::SaveHsa { value: inputs, .. } = &mut invalid else {
            panic!("expected HSA correction editor")
        };
        let details = inputs.accounts[0].excess_accounting.as_mut().unwrap();
        details.later_return_year_deposits = 100;
        details.later_employer_deposits = 101;
        let accepted_draft = session.draft.clone();
        rejected_with(send(&mut session, invalid), "HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER");
        assert_eq!(session.draft, accepted_draft);
        assert_eq!(supported(&session.draft), result);
    }
}

// Source-derived Form5329 PartVI lines34-41, not an IRS worked complete return.
// The 2026 draft instructions line35 use the smaller of Form8853 lines3/4
// less current contributions and include absorbed prior excess on line5.
// Synthetic actual owner: $60,000 wages; self-only $4,000 deductible gives
// $2,600 capacity; $2,000 retained positive-tax excess; $1,000 employer
// contributions leave $1,600 for prior excess, $400 carry and $24 excise.
// Raising employer contributions to $2,600 uses all capacity, leaving no
// prior deduction, $2,000 carry and $120 excise. These are not new deposits.
// https://www.irs.gov/pub/irs-dft/i5329--dft.pdf (PartVI, lines34/35/39)
// https://www.irs.gov/publications/p969 (2025 final, later-year Archer excess deduction)
#[test]
fn unit46_archer_prior_carry_and_current_employer_deposits_save_reopen_and_use_only_unused_room() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = open_hsa_page(&mut session);
    let seed = page.clone();
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
    let P::SaveHsa { value: inputs, .. } = &mut page else {
        panic!("expected actual HSA/Archer editor")
    };
    inputs.archer = ArcherContributionsPage::Records {
        owners: vec![row.clone()],
        sharing: ArcherSharing::Equal,
    };
    if client_artifacts(&mut session, &seed, &mut page, "archer") {
        return;
    }
    accepted(&send(&mut session, page));
    {
        extern crate std;
        if std::env::var("TAX_UNIT46_HEALTH_CASE").as_deref() == Ok("archer")
            && let Ok(path) = std::env::var("TAX_UNIT46_HEALTH_REOPEN")
        {
            std::fs::write(path, open_hsa_page(&mut session).to_avro()).unwrap();
        }
    }
    let result = supported(&session.draft);
    assert_eq!(value(&result, "archer_prior_excess_deducted_return_year"), 1600);
    assert_eq!(value(&result, "archer_prior_excess_remaining_following_year"), 400);
    assert_eq!(value(&result, "archer_msa_deduction"), 1600);
    assert_eq!(value(&result, "archer_return_year_uncorrected_employer_excess"), 0);
    assert_eq!(value(&result, "archer_return_year_excess_contribution_tax"), 24);
    assert_eq!(value(&result, "agi"), 58_400);
    assert_eq!(
        hsa_page_value(&open_hsa_page(&mut session)).0.archer,
        ArcherContributionsPage::Records { owners: vec![row], sharing: ArcherSharing::Equal }
    );
    let mut restarted = restart(session.draft.clone());
    assert_eq!(supported(&restarted.draft), result);
    assert_eq!(
        hsa_page_value(&open_hsa_page(&mut restarted)).0.archer,
        hsa_page_value(&open_hsa_page(&mut session)).0.archer
    );

    let mut exhausted = open_hsa_page(&mut session);
    let P::SaveHsa { value: inputs, .. } = &mut exhausted else { panic!("expected Archer editor") };
    let ArcherContributionsPage::Records { owners, .. } = &mut inputs.archer else {
        panic!("expected retained Archer contribution owner")
    };
    owners[0].employer = 260_000;
    accepted(&send(&mut session, exhausted));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "archer_prior_excess_deducted_return_year"), 0);
    assert_eq!(value(&result, "archer_prior_excess_remaining_following_year"), 2000);
    assert_eq!(value(&result, "archer_msa_deduction"), 0);
    assert_eq!(value(&result, "archer_return_year_excess_contribution_tax"), 120);
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(supported(&restart(session.draft.clone()).draft), result);

    let mut wrong_source = open_hsa_page(&mut session);
    let P::SaveHsa { value: inputs, .. } = &mut wrong_source else {
        panic!("expected Archer source editor")
    };
    let ArcherContributionsPage::Records { owners, .. } = &mut inputs.archer else {
        panic!("expected retained Archer contribution owner")
    };
    owners[0].income = vec![99];
    let accepted_draft = session.draft.clone();
    rejected_with(send(&mut session, wrong_source), "ARCHER_INCOME_NOT_OFFERED");
    assert_eq!(session.draft, accepted_draft);

    let mut invalid = open_hsa_page(&mut session);
    let P::SaveHsa { value: inputs, .. } = &mut invalid else {
        panic!("expected Archer correction editor")
    };
    let ArcherContributionsPage::Records { owners, .. } = &mut inputs.archer else {
        panic!("expected retained Archer contribution owner")
    };
    let details = owners[0].excess_accounting.as_mut().unwrap();
    details.later_return_year_deposits = 100;
    details.later_employer_deposits = 101;
    let accepted_draft = session.draft.clone();
    rejected_with(send(&mut session, invalid), "ARCHER_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER");
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(supported(&session.draft), result);
}
