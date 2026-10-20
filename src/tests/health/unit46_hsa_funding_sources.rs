// Approved source facts, source-event chronology and the normal Health lifecycle.
use crate::tests::cross::interview::{
    accepted, hsa_account, hsa_page_value, open_hsa_page, open_topic, rejected_with,
};
use super::*;

pub(super) fn fixture(case: &str) -> (FederalDraft, HsaFundingRow) {
    let mut draft = match case {
        "pretax" => {
            let mut draft = complete(FederalStatus::Single);
            current_hsa_funding(&mut draft, 1500);
            let source = draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
            let FederalIncome::Retirement { data } = &mut draft
                .income
                .iter_mut()
                .find(|entry| entry.id == source)
                .unwrap()
                .income
            else {
                unreachable!()
            };
            data.gross = 150_000;
            data.taxable = RetirementTaxable::Payer { amount: 150_000 };
            draft
        }
        "prior" => hsa_prior_basis_funding(1500, 200, 2000, 500, 0),
        "roth" => roth_hsa_funding(1500, 200, 2000, 0),
        _ => unreachable!(),
    };
    let mut funding = draft.adjustments.as_ref().unwrap().hsa[0].funding[0].row(Owner::Taxpayer);
    funding.funded = 20260702;
    match &mut funding.basis {
        HsaFundingBasis::Pretax { distributed }
        | HsaFundingBasis::PriorBasis { distributed, .. } => *distributed = 20260630,
        HsaFundingBasis::Roth { .. } => {
            let FederalIncome::Roth { data } = &mut draft
                .income
                .iter_mut()
                .find(|entry| entry.id == funding.source)
                .unwrap()
                .income
            else {
                unreachable!()
            };
            data.distributed = 20260630;
        }
    }
    draft.adjustments.as_mut().unwrap().hsa.clear();
    (draft, funding)
}
fn inputs(page: &mut P) -> &mut HsaPage {
    let P::SaveHsa { value, .. } = page else {
        unreachable!()
    };
    value
}
fn bridge(session: &mut Interview, seed: &P, page: &mut P, case: &str) -> bool {
    extern crate std;
    if std::env::var("TAX_UNIT46_HSA_SOURCE_CASE").as_deref() != Ok(case) {
        return false;
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_HSA_SOURCE_CALC_INPUT") {
        let calculation = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(
            calculation,
            P::HsaOptions { .. } | P::EstimateHsa { .. }
        ));
        let before = session.draft.clone();
        let Reply::Response(reply) = send(session, calculation) else {
            panic!("expected ordinary calculation response")
        };
        assert!(matches!(
            reply,
            P::HsaFamilyOptions { .. } | P::RunningEstimate { .. }
        ));
        assert_eq!(session.draft, before);
        std::fs::write(
            std::env::var("TAX_UNIT46_HSA_SOURCE_CALC_OUTPUT").unwrap(),
            reply.to_avro(),
        )
        .unwrap();
        return true;
    }
    for (key, request) in [
        ("TAX_UNIT46_HSA_SOURCE_SEED", seed),
        ("TAX_UNIT46_HSA_SOURCE_ANSWERS", &*page),
    ] {
        if let Ok(path) = std::env::var(key) {
            std::fs::write(path, request.to_avro()).unwrap();
        }
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_HSA_SOURCE_PAYLOAD") {
        *page = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(page, P::SaveHsa { .. }));
    }
    false
}
fn write_reopen(page: &P, case: &str) {
    extern crate std;
    if std::env::var("TAX_UNIT46_HSA_SOURCE_CASE").as_deref() == Ok(case)
        && let Ok(path) = std::env::var("TAX_UNIT46_HSA_SOURCE_REOPEN")
    {
        std::fs::write(path, page.to_avro()).unwrap();
    }
}
fn resave(session: &mut Interview, case: &str) {
    extern crate std;
    if std::env::var("TAX_UNIT46_HSA_SOURCE_CASE").as_deref() == Ok(case)
        && let Ok(path) = std::env::var("TAX_UNIT46_HSA_SOURCE_RESAVE")
    {
        let page = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(page, P::SaveHsa { .. }));
        let before = session.draft.clone();
        accepted(&send(session, page));
        // An unchanged second Save is a normal no-op, including its revision.
        assert_eq!(session.draft, before);
    }
}
fn refused_atomically(session: &mut Interview, page: P, key: &str) {
    let before = session.draft.clone();
    let before_funding: Vec<_> = before
        .adjustments
        .as_ref()
        .unwrap()
        .hsa
        .iter()
        .flat_map(|account| {
            account
                .funding
                .iter()
                .map(|funding| funding.row(account.owner))
        })
        .collect();
    rejected_with(send(session, page), key);
    assert_eq!(session.draft, before);
    assert_eq!(
        hsa_page_value(&open_hsa_page(session)).0.funding,
        before_funding
    );
    assert_eq!(restart(before.clone()).draft, before);
}

// Synthetic2026 completion of Notice2008-51 source rules; the original basis
//200/value2000/funding1500 control is its published example, not a general
// earlier-ordinary basis allocation. Distribution June30, HSA receipt July2:
// distribution year2026, testing period through July31,2027.
#[test]
fn unit46_hsa_source_facts_save_reopen_restart_and_atomic_refusals() {
    extern crate std;
    for case in ["pretax", "prior", "roth", "two"] {
        if let Ok(selected) = std::env::var("TAX_UNIT46_HSA_SOURCE_CASE")
            && selected != case
        {
            continue;
        }
        let (mut draft, funding) = fixture(if case == "two" { "pretax" } else { case });
        let mut fundings = vec![funding.clone()];
        let coverage = if case == "two" {
            // Two actual accepted IRA payments, assigned through the normal
            // shared fixture identity owner. Both receipts are in July, while
            // the source distributions span June and July.
            let second_source = draft.next_id;
            add(
                &mut draft,
                retirement_entry(
                    Owner::Taxpayer,
                    RetirementKind::Ira,
                    1000,
                    RetirementTreatment::Ordinary,
                ),
            );
            fundings.push(HsaFundingRow {
                owner: Owner::Taxpayer,
                source: second_source,
                ongoing_sep_simple: false,
                funded: 20260705,
                amount: 100_000,
                basis: HsaFundingBasis::Pretax {
                    distributed: 20260703,
                },
                coverage: HsaCoverage::Family,
                loss: None,
            });
            HsaCoverageYear::Changed {
                months: (0..12)
                    .map(|month| {
                        if month < 6 {
                            HsaCoverage::SelfOnly
                        } else {
                            HsaCoverage::Family
                        }
                    })
                    .collect(),
            }
        } else {
            HsaCoverageYear::SelfOnly
        };
        let mut session = Interview::new(TEST_USER, draft);
        let mut page = open_hsa_page(&mut session);
        let seed = page.clone();
        assert!(hsa_page_value(&seed).0.funding.is_empty());
        assert!(hsa_page_value(&seed).0.accounts.is_empty());
        inputs(&mut page).accounts = vec![hsa_account(Owner::Taxpayer, coverage, 0)];
        inputs(&mut page).family_split = FamilySplit::NotApplicable;
        inputs(&mut page).funding = fundings.clone();
        if bridge(&mut session, &seed, &mut page, case) {
            return;
        }
        let before_revision = session.draft.revision;
        accepted(&send(&mut session, page));
        assert_eq!(session.draft.revision, before_revision + 1);
        let result = supported(&session.draft);
        if case == "two" {
            assert_eq!(return_line(&result, "4a"), 250_000);
            assert_eq!(
                session.draft.adjustments.as_ref().unwrap().hsa[0].funding[1]
                    .distribution_date(&session.draft),
                Some(20260703)
            );
        } else {
            assert_eq!(return_line(&result, "4a"), 150_000);
        }
        assert_eq!(return_line(&result, "4b"), 0);
        assert_eq!(value(&result, "agi"), 60_000);
        assert_eq!(value(&result, "adjustments"), 0);
        assert_eq!(value(&result, "early_distribution_tax"), 0);
        assert_eq!(
            session.draft.adjustments.as_ref().unwrap().hsa[0].funding[0]
                .distribution_date(&session.draft),
            Some(20260630)
        );
        if case == "prior" {
            assert_eq!(value(&result, "taxpayer_ira_basis_carryforward"), 200);
        }
        let reopened = open_hsa_page(&mut session);
        assert_eq!(hsa_page_value(&reopened).0.funding, fundings.clone());
        write_reopen(&reopened, case);
        resave(&mut session, case);
        let accepted_draft = session.draft.clone();
        let mut restarted = restart(accepted_draft.clone());
        assert_eq!(restarted.draft, accepted_draft);
        assert_eq!(supported(&restarted.draft), result);
        assert_eq!(
            hsa_page_value(&open_hsa_page(&mut restarted)).0.funding,
            fundings.clone()
        );

        if case == "two" {
            for (month, ineligible) in [(5, HsaCoverage::Family), (6, HsaCoverage::SelfOnly)] {
                let mut bad = open_hsa_page(&mut session);
                let HsaCoverageYear::Changed { months } =
                    &mut inputs(&mut bad).accounts[0].coverage
                else {
                    unreachable!()
                };
                months[month] = ineligible;
                refused_atomically(&mut session, bad, "HSA_FUNDING_LIFETIME");
            }
            let mut bad = open_hsa_page(&mut session);
            let HsaFundingBasis::Pretax { distributed } = &mut inputs(&mut bad).funding[1].basis
            else {
                unreachable!()
            };
            *distributed = 20260630;
            refused_atomically(&mut session, bad, "HSA_FUNDING_LIFETIME");
        }
        let mut bad = open_hsa_page(&mut session);
        inputs(&mut bad).funding[0].ongoing_sep_simple = true;
        refused_atomically(&mut session, bad, "HSA_FUNDING_ONGOING_SEP_SIMPLE");
        let mut bad = open_hsa_page(&mut session);
        inputs(&mut bad).funding[0].funded = 20260629;
        if let P::SaveHsa { ira_sources, .. } = &mut bad {
            // A forged earlier metadata date cannot alter the actual Roth source.
            for source in ira_sources
                .iter_mut()
                .filter(|source| source.id == funding.source)
            {
                if let HsaFundingSourceKind::Roth { distributed } = &mut source.kind {
                    *distributed = 20260628;
                }
            }
        }
        refused_atomically(&mut session, bad, "HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT");
        if case != "roth" {
            for date in [20251231, 20270101, 20260230] {
                let mut bad = open_hsa_page(&mut session);
                match &mut inputs(&mut bad).funding[0].basis {
                    HsaFundingBasis::Pretax { distributed }
                    | HsaFundingBasis::PriorBasis { distributed, .. } => *distributed = date,
                    _ => unreachable!(),
                }
                refused_atomically(&mut session, bad, "HSA_FUNDING_DISTRIBUTION_YEAR");
            }
        }
        let mut bad = open_hsa_page(&mut session);
        inputs(&mut bad).funding[0].coverage = HsaCoverage::None;
        refused_atomically(&mut session, bad, "HSA_FUNDING_NEEDS_COVERAGE");
        let mut bad = open_hsa_page(&mut session);
        inputs(&mut bad).accounts.clear();
        refused_atomically(&mut session, bad, "HSA_FUNDING_NEEDS_AN_ACCOUNT");

        // Actual linked income edits use the same guard, never a stale catalog.
        let mut bad = open_topic(&mut session, TaxSection::Retirement);
        let P::SaveIncomes { documents, .. } = &mut bad else {
            unreachable!()
        };
        documents.documents.retain(|row| row.id != funding.source);
        refused_atomically(&mut session, bad, "HSA_FUNDING_SOURCE_NOT_OFFERED");
        if case == "roth" {
            let mut bad = open_topic(&mut session, TaxSection::Retirement);
            let P::SaveIncomes { documents, .. } = &mut bad else {
                unreachable!()
            };
            let IncomeDocument::R1099(data) = &mut documents
                .documents
                .iter_mut()
                .find(|row| row.id == funding.source)
                .unwrap()
                .document
            else {
                unreachable!()
            };
            let DistributionCode::RothIra {
                roth: RothIraCode::Early { distributed, .. },
                ..
            } = &mut data.code
            else {
                unreachable!()
            };
            *distributed = 20260703;
            refused_atomically(&mut session, bad, "HSA_FUNDING_DISTRIBUTION_AFTER_RECEIPT");
        }
    }
}

#[test]
fn unit46_hsa_source_distribution_month_controls_second_funding_and_order() {
    let mut draft = hsa_prior_basis_funding(1000, 200, 3000, 1000, 0);
    let account = &mut draft.adjustments.as_mut().unwrap().hsa[0];
    account.coverage = HsaEligibility::Monthly {
        months: (0..12)
            .map(|month| {
                if month < 6 {
                    HsaCoverage::SelfOnly
                } else {
                    HsaCoverage::Family
                }
            })
            .collect(),
    };
    account.funding[0].funded = 20260706;
    account.funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260615,
        value: 300_000,
        before_other_activity: true,
    };
    let mut second = account.funding[0].clone();
    second.funded = 20260705;
    second.coverage = HsaCoverage::Family;
    second.basis = HsaFundingBasis::PriorBasis {
        distributed: 20260701,
        value: 200_000,
        before_other_activity: true,
    };
    account.funding.push(second);
    let source = account.funding[0].income;
    let FederalIncome::Retirement { data } = &mut draft
        .income
        .iter_mut()
        .find(|entry| entry.id == source)
        .unwrap()
        .income
    else {
        unreachable!()
    };
    data.gross = 200_000;
    assert!(crate::hsa_funding::source_reference_rule(&draft).is_none());
    let output = supported(&draft);
    assert_eq!(return_line(&output, "4b"), 0);
    let mut reversed = draft.clone();
    reversed.adjustments.as_mut().unwrap().hsa[0]
        .funding
        .reverse();
    assert_eq!(supported(&reversed), output);
    // The same first-day facts admit the ordinary Save/reopen/restart path.
    let mut session = Interview::new(TEST_USER, reversed.clone());
    let page = open_hsa_page(&mut session);
    accepted(&send(&mut session, page));
    assert_eq!(supported(&session.draft), output);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    // Event coverage does not replace the actual first-day account coverage.
    // June distribution/July receipt uses June, rather than July's Family fact.
    for coverage in [
        HsaCoverageYear::Family,
        HsaCoverageYear::Changed {
            months: (0..12)
                .map(|month| {
                    if month < 5 {
                        HsaCoverage::SelfOnly
                    } else {
                        HsaCoverage::Family
                    }
                })
                .collect(),
        },
        HsaCoverageYear::Changed {
            months: (0..12)
                .map(|month| {
                    if month < 7 {
                        HsaCoverage::SelfOnly
                    } else {
                        HsaCoverage::Family
                    }
                })
                .collect(),
        },
    ] {
        let mut bad = open_hsa_page(&mut session);
        inputs(&mut bad).accounts[0].coverage = coverage;
        refused_atomically(&mut session, bad, "HSA_FUNDING_LIFETIME");
    }
    // Malformed raw dates must never underflow a month index in calculation.
    for invalid in [20260015, 20260230, 20261301] {
        let mut bad = reversed.clone();
        bad.adjustments.as_mut().unwrap().hsa[0].funding[1].basis = HsaFundingBasis::PriorBasis {
            distributed: invalid,
            value: 200_000,
            before_other_activity: true,
        };
        let mut result = supported(&complete(FederalStatus::Single));
        assert_eq!(
            crate::hsa_funding::calculate(&bad, &mut result).for_entry(source),
            0
        );
    }
    // Two receipt months cannot cure two actual distributions in the same month.
    let second = &mut draft.adjustments.as_mut().unwrap().hsa[0].funding[1];
    second.basis = HsaFundingBasis::PriorBasis {
        distributed: 20260620,
        value: 200_000,
        before_other_activity: true,
    };
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&draft),
        Some("HSA_FUNDING_LIFETIME")
    );
    let mut bad = open_hsa_page(&mut session);
    inputs(&mut bad).funding = draft.adjustments.as_ref().unwrap().hsa[0]
        .funding
        .iter()
        .map(|funding| funding.row(Owner::Taxpayer))
        .collect();
    refused_atomically(&mut session, bad, "HSA_FUNDING_LIFETIME");
}

#[test]
fn unit46_hsa_source_ineligible_ongoing_does_not_erase_actual_ira_income() {
    for case in ["pretax", "prior", "roth"] {
        let (mut draft, row) = fixture(case);
        let mut account = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
        let mut funding = row.record();
        funding.ongoing_sep_simple = true;
        account.funding = vec![funding];
        draft.adjustments.as_mut().unwrap().hsa = vec![account];
        let original_income = draft.income.clone();
        let mut result = supported(&complete(FederalStatus::Single));
        let funding = crate::hsa_funding::calculate(&draft, &mut result);
        assert_eq!(funding.for_entry(row.source), 0);
        assert_eq!(draft.income, original_income);
        assert_eq!(
            crate::hsa_funding::source_reference_rule(&draft),
            Some("HSA_FUNDING_ONGOING_SEP_SIMPLE")
        );
    }
}

#[test]
fn unit46_hsa_source_date_controls_basis_activity_before_hsa_receipt() {
    // Preserve the existing repayment/basis monetary oracle, moving its actual
    // events across the source-distribution/HSA-receipt gap.
    let mut draft = hsa_prior_basis_funding(1500, 200, 2000, 1500, 0);
    let funding = &mut draft.adjustments.as_mut().unwrap().hsa[0].funding[0];
    funding.funded = 20260702;
    funding.basis = HsaFundingBasis::PriorBasis {
        distributed: 20260630,
        value: 200_000,
        before_other_activity: true,
    };
    let mut repayment = reservist_repayment(1000);
    repayment.payments[0].date = 20260701;
    draft.adjustments.as_mut().unwrap().ira_basis[0]
        .reservist_repayments
        .push(repayment);
    let result = supported(&draft);
    assert_eq!(value(&result, "taxpayer_ira_basis_carryforward"), 1200);
    assert_eq!(return_line(&result, "4b"), 0);
    let mut session = Interview::new(TEST_USER, draft.clone());
    let page = open_hsa_page(&mut session);
    accepted(&send(&mut session, page));
    assert_eq!(supported(&session.draft), result);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    draft.adjustments.as_mut().unwrap().ira_basis[0].reservist_repayments[0].payments[0].date =
        20260629;
    unsupported(&draft, "HSA funding");
}

#[test]
fn unit46_hsa_source_inherited_death_and_owner_transit_boundaries_are_distinct() {
    let mut draft = inherited_hsa_funding(1500, 200, 2000, 1500, 1500);
    let account = &mut draft.adjustments.as_mut().unwrap().hsa[0];
    account.funding[0].funded = 20260702;
    account.funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260630,
        value: 200_000,
        before_other_activity: true,
    };
    let source_id = account.funding[0].income;
    let group_id = crate::retirement::ira_funding_source(&draft, source_id)
        .unwrap()
        .inherited
        .unwrap()
        .0;
    let FederalIncome::InheritedIra { data } = &mut draft
        .income
        .iter_mut()
        .find(|entry| entry.id == group_id)
        .unwrap()
        .income
    else {
        unreachable!()
    };
    data.died = 20260630;
    let mut session = Interview::new(TEST_USER, draft);
    let page = open_hsa_page(&mut session);
    accepted(&send(&mut session, page));
    let mut bad = open_hsa_page(&mut session);
    let HsaFundingBasis::PriorBasis { distributed, .. } = &mut inputs(&mut bad).funding[0].basis
    else {
        unreachable!()
    };
    *distributed = 20260629;
    refused_atomically(&mut session, bad, "HSA_FUNDING_BEFORE_INHERITED_DEATH");
    // Actual distribution precedes personal death, but receipt after that death
    // remains an explicit unsupported current-owner boundary, not a new date.
    let (mut draft, row) = fixture("pretax");
    let mut account = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 0);
    account.funding.push(row.record());
    draft.adjustments.as_mut().unwrap().hsa = vec![account];
    draft.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
        bonds: None,
        estate_medical: 0,
        date: 20260701,
        ordinary: true,
    });
    assert_eq!(
        draft.adjustments.as_ref().unwrap().hsa[0].funding[0].distribution_date(&draft),
        Some(20260630)
    );
    assert_eq!(
        crate::hsa_funding::source_reference_rule(&draft),
        Some("HSA_FUNDING_OWNER_DIED_BEFORE_RECEIPT")
    );
}
