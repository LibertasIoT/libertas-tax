use super::*;
use crate::tests::cross::interview::{accepted, document, rejected_with};
use crate::tests::support::client::income_pages::{open_document_center, save_documents};

fn k1_rows(request: &P) -> &[crate::K1DocumentRow] {
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    &documents.common.k1.documents
}

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
    data.amt_reconciliation =
        Some(crate::EstateAmtReconciliation { other_in_a: 0, adjustments: vec![] });
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
    assert_eq!(crate::estate_trust::investment_adjustment(&data), Some([0; 2]));
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
    let seed = open_document_center(&mut session);
    let P::SaveDocuments { documents, allowed_community, .. } = &seed else { unreachable!() };
    assert!(documents.common.k1.documents.is_empty());
    let mut rows = Vec::new();
    let second_owner = if first_owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
    for (label, owner, data) in [
        ("Independent estate A", first_owner, super::unit27_estate_community::source()),
        ("Independent estate B", second_owner, second_source()),
    ] {
        let mut row = document(
            0,
            IncomeSource::K1 {
                value: crate::K1Document::K1EstateTrust(estate_document(
                    K1EstateTrust::from_record(label, &data).unwrap(),
                )),
            },
        );
        row.owner = crate::DocumentOwner::from(owner);
        row.community = CommunityShare::Separate;
        row.allowed_community = allowed_community.clone();
        rows.push(row);
    }
    let request = save_documents(&mut session, rows);
    let request = fixture_owner_inputs(&session.draft, request);
    accepted(&save_mfs_owner_fixture(&mut session, request));
    session
}

fn projected_sources(draft: &FederalDraft) -> Vec<(i64, FederalEstateTrustPortfolioK1)> {
    let mut result = Vec::new();
    for entry in crate::community::entries(draft) {
        if let FederalIncome::EstateTrustPortfolio { data } = &entry.income {
            result.push((entry.id, FederalEstateTrustPortfolioK1::from(data)));
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
        let first = raw.form.interest == 100_000;
        assert_eq!(
            FederalEstateTrustPortfolioK1::from(raw),
            if first { super::unit27_estate_community::source() } else { second_source() }
        );
        assert_eq!(entry.owner == crate::DocumentOwner::from(first_owner), first);
        assert_eq!(crate::estate_trust::community_source_rule(draft, entry), None);
        let projected = crate::community::entry(draft, entry);
        let FederalIncome::EstateTrustPortfolio { data } = &projected.income else {
            unreachable!()
        };
        assert_eq!(
            crate::estate_trust::source_rule(&FederalEstateTrustPortfolioK1::from(data)),
            None
        );
        assert_eq!(
            crate::estate_trust::investment_adjustment(&FederalEstateTrustPortfolioK1::from(data)),
            Some([0; 2])
        );
        let projected = FederalEstateTrustPortfolioK1::from(data);
        assert_eq!(
            projected.amt_box12a,
            projected.amt_box12b + projected.amt_box12c + projected.amt_box12d
        );
        let investment = data.statements.investment_income.as_ref().unwrap();
        let expected = match (first, entry.owner.owned()) {
            (true, Owner::Taxpayer) => [190_000, 160_000],
            (true, Owner::Spouse) => [70_000, 65_000],
            (false, Owner::Taxpayer) => [80_006, 60_001],
            (false, Owner::Spouse) => [40_000, 40_000],
        };
        assert_eq!([investment.regular, investment.alternative], expected);
        if !first {
            assert_eq!(
                [
                    projected.short_gain + projected.amt_box12c,
                    projected.long_gain + projected.amt_box12d
                ],
                if entry.owner == crate::DocumentOwner::Yours { [-10_000, 30_000] } else { [0, 0] }
            );
        }
        totals[0] += investment.regular;
        totals[1] += investment.alternative;
    }
    assert_eq!(count, 2);
    assert_eq!(
        totals,
        if first_owner == Owner::Taxpayer { [230_000, 200_000] } else { [150_006, 125_001] }
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
        let mut reopened = open_document_center(&mut session);
        assert_eq!(k1_rows(&reopened).len(), 2);
        for row in k1_rows(&reopened) {
            let original = originals.iter().find(|entry| entry.id == row.id).unwrap();
            let crate::K1Document::K1EstateTrust(page) = &row.document else { unreachable!() };
            let FederalIncome::EstateTrustPortfolio { data } = &original.income else {
                unreachable!()
            };
            assert_eq!(page, &crate::K1EstateTrustDocument::from_source(data));
            assert_eq!(
                FederalEstateTrustPortfolioK1::from(data),
                if data.form.interest == 100_000 {
                    super::unit27_estate_community::source()
                } else {
                    second_source()
                }
            );
            assert_eq!(row.owner, original.owner);
            assert_eq!(row.community, original.community);
        }
        let mut restarted = restart(session.draft.clone());
        assert_eq!(restarted.draft, session.draft);
        assert_eq!(supported(&restarted.draft), report);
        let after_restart = open_document_center(&mut restarted);
        assert_eq!(k1_rows(&after_restart), k1_rows(&reopened));

        // Reorder through the ordinary reopened Save. IDs and complete source
        // records survive; each source's cents are independent of list order.
        let P::SaveDocuments { documents, .. } = &mut reopened else { unreachable!() };
        documents.common.k1.documents.reverse();
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
    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let page = documents
        .common
        .k1
        .documents
        .iter_mut()
        .find_map(|row| {
            if let crate::K1Document::K1EstateTrust(page) = &mut row.document {
                (page.form.interest == 60_001).then_some(page)
            } else {
                None
            }
        })
        .unwrap();
    // Signed AMT ST is supported by the source reconciliation. A negative
    // adjusted unrecaptured1250 pool is a contradictory separate source fact.
    page.form.amt.push(crate::EstateTrustCodeRow { code: "E".into(), amount: -1 });
    assert_eq!(
        crate::estate_trust::source_rule(&FederalEstateTrustPortfolioK1::from(&page.record())),
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
        carried: crate::EstatePassiveCarry { regular: 1, alternative: 1 },
        amt_difference: 0,
    }];
    data.qbi.statements = [("A", 0), ("B", 1)]
        .into_iter()
        .map(|(name, remaining_qualified)| crate::EstateQbiStatement {
            id: 0,
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
    let mut next_id = 4;
    crate::interview::assign_record_ids(
        data.qbi.statements.iter_mut().map(|row| &mut row.id),
        core::iter::empty(),
        &mut next_id,
    )
    .unwrap();
    draft.income[1].income = FederalIncome::EstateTrustPortfolio {
        data: crate::tests::support::builders::estate_source(data.clone()),
    };
    draft.income[1].community = CommunityShare::Community;
    draft.next_id = next_id;
    let mut session = Interview::new(TEST_USER, draft);
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    let before = session.draft.clone();
    let allocated = projected_sources(&session.draft);
    let projected = &allocated[0].1;
    let history = &projected.qbi.statements[1].allocations[0].passive_history[0];
    assert_eq!([history.qualified, history.remaining_qualified], [0, 1]);
    assert_eq!(crate::estate_trust::investment_adjustment(projected), Some([0; 2]));
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
    control.statements.qbi.statements.clear();
    assert_eq!(value(&report, "agi"), value(&supported(&without_optional_qbi), "agi"));
    let reopened = open_document_center(&mut session);
    let crate::K1Document::K1EstateTrust(page) = &k1_rows(&reopened)[0].document else {
        unreachable!()
    };
    let (_, accepted) = stored_k1_1041(&session);
    assert_eq!(accepted, data);
    let saved = session
        .draft
        .income
        .iter()
        .find_map(|row| match &row.income {
            FederalIncome::EstateTrustPortfolio { data } => Some(data),
            _ => None,
        })
        .unwrap();
    assert_eq!(page, &crate::K1EstateTrustDocument::from_source(saved));
    assert_eq!(session.draft, before);
    let mut restored = restart(before.clone());
    assert_eq!(supported(&restored.draft), report);
    let reopened = open_document_center(&mut restored);
    let crate::K1Document::K1EstateTrust(page) = &k1_rows(&reopened)[0].document else {
        unreachable!()
    };
    let (_, accepted) = stored_k1_1041(&restored);
    assert_eq!(accepted, data);
    let source = restored
        .draft
        .income
        .iter()
        .find_map(|row| match &row.income {
            FederalIncome::EstateTrustPortfolio { data } => Some(data),
            _ => None,
        })
        .unwrap();
    assert_eq!(page, &crate::K1EstateTrustDocument::from_source(source));
}

#[test]
fn estate_fiduciary_collection_correction_replaces_source_facts_once() {
    let record = second_source();
    let accepted = crate::EstateTrustSource::from_record(&record);
    let mut document = crate::K1EstateTrustDocument::from_source(&accepted);
    assert_eq!(document.statements.investment_income, record.investment_income);
    document.statements.investment_income.as_mut().unwrap().regular = 120_007;
    document.statements.final_state_income_tax = 123;
    document.statements.final_taxes_only = true;
    let mut changed = document.record();
    changed.keep_answers(&accepted);
    let projected = crate::K1EstateTrustDocument::from_source(&changed);
    assert_eq!(projected.statements, document.statements);
    let native = changed.checked_record().unwrap();
    assert_eq!(native.investment_income.unwrap().regular, 120_007);
    assert_eq!(native.final_state_income_tax, 123);
    assert!(changed.moved_questions().iter().all(|(field, ..)| !matches!(
        field.as_str(),
        "estate/final_tax_character"
            | "estate/nol_records"
            | "estate/niit_allocation"
            | "estate/royalty_character"
    )));
}

#[test]
fn estate_missing_fiduciary_statement_stays_unavailable() {
    let accepted = crate::EstateTrustSource::from_record(&second_source());
    let mut document = crate::K1EstateTrustDocument::from_source(&accepted);
    document.form.information.push(crate::EstateTrustCodeRow { code: "E".into(), amount: 120_006 });
    document.statements.investment_income = None;
    let mut source = document.record();
    source.keep_answers(&accepted);
    let native = source.checked_record().unwrap();
    assert_eq!(native.investment_income, None);
    assert!(native.unentered.contains(&crate::EstateTrustK1Item::InvestmentIncome));
    let printed_only = crate::EstateTrustSource::printed(document.form);
    assert!(!printed_only.statements.final_taxes_only);
    assert!(!printed_only.statements.final_carry_same_tax_character);
    assert!(!printed_only.statements.amt_box12_components_confirmed);
}

#[test]
fn estate_printed_period_and_separate_week_election_share_actual_dates() {
    // IRS Schedule K-1 (1041), 2026 draft header; Pub. 538, 52-53-week election.
    let accepted = crate::EstateTrustSource::from_record(&second_source());
    let mut document = crate::K1EstateTrustDocument::from_source(&accepted);
    document.form.began = Some(20260101);
    document.form.ended = Some(20261226);
    let mut source = document.record();
    let native = source.checked_record().unwrap();
    let period = native.entity_period.unwrap();
    assert_eq!([period.period.began, period.period.ended], [20260101, 20261226]);
    assert_eq!(period.accounting, crate::FinalEstateTrustAccountingYear::CalendarFiscalOrShort);
    source
        .apply_moved_answer(
            "estate/accounting",
            crate::DocumentAnswer::EstateAccounting {
                value: Some(crate::FinalEstateTrustAccountingYear::Weeks52Or53 {
                    annual_began: 20251228,
                    annual_ended: 20261226,
                    reference_ended: 20261231,
                    closing: crate::FinalEstateTrustWeekClosing::Last,
                }),
            },
        )
        .unwrap();
    let elected = source.checked_record().unwrap().entity_period.unwrap();
    assert_eq!(elected.period, period.period);
    assert!(matches!(
        elected.accounting,
        crate::FinalEstateTrustAccountingYear::Weeks52Or53 { .. }
    ));
    document.form.began = None;
    document.form.ended = None;
    document.form.tax_year = 2026;
    let calendar = document.record().checked_record().unwrap().entity_period.unwrap();
    assert_eq!([calendar.period.began, calendar.period.ended], [20260101, 20261231]);
}
