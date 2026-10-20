use crate::tests::cross::interview::{accepted, document, open_topic, page_documents, rejected_with};
use super::*;

// Independent source-composition arithmetic, in cents. These are synthetic
// fixtures, not published IRS tax totals. The second source has an actual
// signed AMT short-term loss and a separately reconciled fiduciary statement.
fn second_source() -> FederalEstateTrustPortfolioK1 {
    let mut data = super::unit27_estate_community::source();
    data.interest = 60_001;
    data.ordinary_dividends = 20_001;
    data.qualified_dividends = 10_001;
    data.short_gain = 10_001;
    data.long_gain = 30_003;
    data.amt_box12a = -20_005;
    data.amt_box12b = -1;
    data.amt_box12c = -20_001;
    data.amt_box12d = -3;
    data.amt_reconciliation = Some(crate::EstateAmtReconciliation {
        other_in_a: 0,
        adjustments: vec![],
    });
    data.investment_income = Some(crate::EstateInvestmentIncome {
        regular: 120_006,
        alternative: 100_001,
        character: crate::EstateInvestmentCharacter::Preferential {
            qualified: 10_001,
            gains: 40_004,
            amt_qualified: 10_000,
            amt_gains: 20_000,
        },
    });
    assert!(data.valid());
    assert_eq!(crate::estate_trust::source_rule(&data), None);
    assert_eq!(
        crate::estate_trust::investment_adjustment(&data),
        Some([0; 2])
    );
    assert_eq!(data.short_gain + data.amt_box12c, -10_000);
    assert_eq!(data.long_gain + data.amt_box12d, 30_000);
    data
}

fn saved_pair(first_owner: Owner) -> Interview {
    let mut draft = super::unit27_estate_community::draft(first_owner);
    draft
        .income
        .retain(|entry| !matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }));
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes {
        documents,
        allowed_community,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    assert!(documents.documents.is_empty());
    let second_owner = if first_owner == Owner::Taxpayer {
        Owner::Spouse
    } else {
        Owner::Taxpayer
    };
    for (label, owner, data) in [
        (
            "Independent estate A",
            first_owner,
            super::unit27_estate_community::source(),
        ),
        ("Independent estate B", second_owner, second_source()),
    ] {
        let mut row = document(
            0,
            IncomeDocument::K1EstateTrust(K1EstateTrust::from_record(label, &data).unwrap()),
        );
        row.owner = owner;
        row.community = CommunityShare::Separate;
        row.allowed_community = allowed_community.clone();
        documents.documents.push(row);
    }
    accepted(&send(&mut session, request));
    session
}

fn projected_sources(draft: &FederalDraft) -> Vec<(i64, FederalEstateTrustPortfolioK1)> {
    let mut result = Vec::new();
    for entry in crate::community::entries(draft) {
        if let FederalIncome::EstateTrustPortfolio { data } = &entry.income {
            result.push((entry.id, data.clone()));
        }
    }
    result.sort_by_key(|(id, _)| *id);
    result
}

fn check_sources(draft: &FederalDraft, first_owner: Owner) {
    let before = draft.clone();
    let mut totals = [0; 2];
    let mut count = 0;
    for entry in &draft.income {
        let FederalIncome::EstateTrustPortfolio { data: raw } = &entry.income else {
            continue;
        };
        count += 1;
        assert!(entry.id > 0);
        assert_eq!(entry.community, CommunityShare::Separate);
        let first = raw.interest == 100_000;
        assert_eq!(
            raw,
            &if first {
                super::unit27_estate_community::source()
            } else {
                second_source()
            }
        );
        assert_eq!(entry.owner == first_owner, first);
        assert_eq!(
            crate::estate_trust::community_source_rule(draft, entry),
            None
        );
        let projected = crate::community::entry(draft, entry);
        let FederalIncome::EstateTrustPortfolio { data } = &projected.income else {
            unreachable!()
        };
        assert_eq!(crate::estate_trust::source_rule(data), None);
        assert_eq!(
            crate::estate_trust::investment_adjustment(data),
            Some([0; 2])
        );
        assert_eq!(
            data.amt_box12a,
            data.amt_box12b + data.amt_box12c + data.amt_box12d
        );
        let investment = data.investment_income.as_ref().unwrap();
        let expected = match (first, entry.owner) {
            (true, Owner::Taxpayer) => [190_000, 160_000],
            (true, Owner::Spouse) => [70_000, 65_000],
            (false, Owner::Taxpayer) => [80_006, 60_001],
            (false, Owner::Spouse) => [40_000, 40_000],
        };
        assert_eq!([investment.regular, investment.alternative], expected);
        if !first {
            assert_eq!(
                [
                    data.short_gain + data.amt_box12c,
                    data.long_gain + data.amt_box12d
                ],
                if entry.owner == Owner::Taxpayer {
                    [-10_000, 30_000]
                } else {
                    [0, 0]
                }
            );
        }
        totals[0] += investment.regular;
        totals[1] += investment.alternative;
    }
    assert_eq!(count, 2);
    assert_eq!(
        totals,
        if first_owner == Owner::Taxpayer {
            [230_000, 200_000]
        } else {
            [150_006, 125_001]
        }
    );
    assert_eq!(draft, &before);
}

#[test]
fn unit27_estate_independent_two_sources_save_reopen_restart_and_order_conserve() {
    for first_owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut session = saved_pair(first_owner);
        check_sources(&session.draft, first_owner);
        let originals = session.draft.income.clone();
        let expected_projection = projected_sources(&session.draft);
        let report = supported(&session.draft);
        let mut reopened = open_topic(&mut session, TaxSection::PassThrough);
        assert_eq!(page_documents(&reopened).len(), 2);
        for row in page_documents(&reopened) {
            let original = originals.iter().find(|entry| entry.id == row.id).unwrap();
            let IncomeDocument::K1EstateTrust(page) = &row.document else {
                unreachable!()
            };
            let FederalIncome::EstateTrustPortfolio { data } = &original.income else {
                unreachable!()
            };
            assert_eq!(page.record(), *data);
            assert_eq!(row.owner, original.owner);
            assert_eq!(row.community, original.community);
        }
        let mut restarted = restart(session.draft.clone());
        assert_eq!(restarted.draft, session.draft);
        assert_eq!(supported(&restarted.draft), report);
        let after_restart = open_topic(&mut restarted, TaxSection::PassThrough);
        assert_eq!(page_documents(&after_restart), page_documents(&reopened));

        // Reorder through the ordinary reopened Save. IDs and complete source
        // records survive; each source's cents are independent of list order.
        let P::SaveIncomes { documents, .. } = &mut reopened else {
            unreachable!()
        };
        documents.documents.reverse();
        accepted(&send(&mut session, reopened));
        check_sources(&session.draft, first_owner);
        assert_eq!(projected_sources(&session.draft), expected_projection);
        for original in &originals {
            assert!(session.draft.income.contains(original));
        }
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}

#[test]
fn unit27_estate_independent_signed_amt_source_rejects_negative_special_pool_atomically() {
    let mut session = saved_pair(Owner::Taxpayer);
    let before = session.draft.clone();
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    let page = documents
        .documents
        .iter_mut()
        .find_map(|row| {
            if let IncomeDocument::K1EstateTrust(page) = &mut row.document {
                (page.interest == 60_001).then_some(page)
            } else {
                None
            }
        })
        .unwrap();
    // Signed AMT ST is supported by the source reconciliation. A negative
    // adjusted unrecaptured1250 pool is a contradictory separate source fact.
    let K1EstateTrustAmt::Components { e, .. } = &mut page.more.amt else {
        unreachable!()
    };
    *e = -1;
    assert_eq!(
        crate::estate_trust::source_rule(&page.record()),
        Some("ESTATE_AMT_RECONCILE")
    );
    rejected_with(send(&mut session, request), "ESTATE_AMT_RECONCILE");
    assert_eq!(session.draft, before);
    check_sources(&session.draft, Owner::Taxpayer);
}

#[test]
fn unit27_estate_independent_nested_qbi_history_retains_optional_boundary_and_income() {
    let mut data = super::unit27_estate_community::source();
    data.activities.box6 = 0;
    data.activities.activities = vec![crate::EstateActivity {
        id: 3,
        name: "Retained shared activity".into(),
        kind: crate::EstateActivityKind::Business,
        income: 0,
        depreciation: 0,
        depletion: 0,
        amortization: 0,
        participation: crate::EstateParticipation::Passive,
        carried: crate::EstatePassiveCarry {
            regular: 1,
            alternative: 1,
        },
        amt_difference: 0,
    }];
    data.qbi.statements = [("A", 0), ("B", 1)]
        .into_iter()
        .map(|(name, remaining_qualified)| crate::EstateQbiStatement {
            name: name.into(),
            income: 0,
            wages: 0,
            property: 0,
            service: false,
            kind: crate::EstateQbiKind::Business,
            activities: vec![3],
            passive_history: vec![],
            allocations: vec![crate::EstateQbiAllocation {
                activity: 3,
                income: 0,
                passive_history: vec![crate::ScorpLossYear {
                    year: 2025,
                    original: 2,
                    qualified: 1,
                    remaining: 1,
                    remaining_qualified,
                }],
            }],
        })
        .collect();
    assert!(data.valid());
    assert_eq!(data.qbi.rule(&data.activities), None);
    // Model an already retained source with a genuine accepted activity ID,
    // then recommit its whole page through normal Save before projection.
    let mut draft = super::unit27_estate_community::draft(Owner::Taxpayer);
    draft.income[1].income = FederalIncome::EstateTrustPortfolio { data: data.clone() };
    draft.income[1].community = CommunityShare::Community;
    draft.next_id = 4;
    let mut session = Interview::new(TEST_USER, draft);
    let request = open_topic(&mut session, TaxSection::PassThrough);
    accepted(&send(&mut session, request));
    let before = session.draft.clone();
    let allocated = projected_sources(&session.draft);
    let projected = &allocated[0].1;
    let history = &projected.qbi.statements[1].allocations[0].passive_history[0];
    assert_eq!([history.qualified, history.remaining_qualified], [0, 1]);
    assert_eq!(
        crate::estate_trust::investment_adjustment(projected),
        Some([0; 2])
    );
    assert_eq!(
        crate::estate_trust::qbi_projection(
            allocated[0].0,
            projected,
            &projected.qbi.statements[1],
            &[],
            0,
            FederalStatus::Separate,
        )
        .err(),
        Some("WARN_ESTATE_QBI_SOURCE_LEFT_OUT")
    );
    let report = supported(&session.draft);
    assert!(noted(&report, "WARN_ESTATE_QBI_SOURCE_LEFT_OUT"));
    let mut without_optional_qbi = session.draft.clone();
    let FederalIncome::EstateTrustPortfolio { data: control } =
        &mut without_optional_qbi.income[1].income
    else {
        unreachable!()
    };
    control.qbi.statements.clear();
    assert_eq!(
        value(&report, "agi"),
        value(&supported(&without_optional_qbi), "agi")
    );
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    let IncomeDocument::K1EstateTrust(page) = &page_documents(&reopened)[0].document else {
        unreachable!()
    };
    assert_eq!(page.record(), data);
    assert_eq!(session.draft, before);
    let mut restored = restart(before.clone());
    assert_eq!(supported(&restored.draft), report);
    let reopened = open_topic(&mut restored, TaxSection::PassThrough);
    let IncomeDocument::K1EstateTrust(page) = &page_documents(&reopened)[0].document else {
        unreachable!()
    };
    assert_eq!(page.record(), data);
}

#[test]
fn unit27_estate_independent_malformed_restored_source_refuses_before_projection_math() {
    let mut draft = super::unit27_estate_community::draft(Owner::Taxpayer);
    let data = estate_trust_code_d_data(&mut draft);
    data.long_gain = i64::MAX;
    data.amt_box12d = 1;
    let original = draft.clone();
    let restored = restart(draft);
    let result = estimate(&restored.draft);
    assert_eq!(result.state, ResultState::Incomplete);
    assert!(result.tax.is_none() && result.refund.is_none() && result.lines.is_empty());
    assert!(result.issues.iter().any(|issue| {
        issue.display.key == "ISSUE_ACCEPTED_DATA_IS_INVALID_NO_CALCULATION_WAS_PERFORMED_7A345C87"
    }));
    assert_eq!(restored.draft, original);
}
