// Unit22 verifies the surviving inventory cases through the real K-1 pages.
// These are synthetic source facts. Refusals describe App scope, not a legal
// prohibition. Sources: IRS i6198, i1065sk1, i7203 and i8995a.
use super::*;
use crate::tests::support::client::income_pages::{open_document_center};

fn partner(profit: i64) -> K1PartnershipPaper {
    let mut p = k1_1065("Unit22 synthetic partnership");
    p.ordinary = profit * 100;
    (p.participation, p.limited_for_se) = (Some((LimitedParticipation::Over500Hours { hours: 600 }).record()), Some(true));
    p.basis.start = k1_1065_continuing(
        1_000_000,
        ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 },
    );
    p.qbi = PartnerQbi::NoStatement;
    p
}

fn scorp(profit: i64) -> K1ScorpPaper {
    let mut p = k1_1120s_fixture(profit);
    p.qbi = K1ScorpQbi::NoStatement;
    p
}

fn save(page: IncomeSource) -> Interview {
    save_with(complete(FederalStatus::Single), page)
}

fn save_with(d: FederalDraft, page: IncomeSource) -> Interview {
    let topic = topic_for(&page);
    let is_source = !matches!(page, IncomeSource::Record { .. });
    let mut s = Interview::new(TEST_USER, d.clone());
    if is_source {
        accepted(&crate::tests::support::client::income_pages::save_k1_sources_and_answers(
            &mut s, vec![document(0, page.clone())],
        ));
    } else {
        let request = save_topic(&mut s, topic, vec![document(0, page.clone())]);
        accepted(&send(&mut s, request));
    }
    let reopened = if is_source { open_document_center(&mut s) } else { open_topic(&mut s, topic) };
    let rows: Vec<_> = page_documents(&reopened)
        .into_iter()
        .filter(|row| !is_source || matches!(row.document, IncomeSource::K1 { .. }))
        .collect();
    let expected = if is_source {
        expected_named_source(&s.draft, Owner::Taxpayer, page)
    } else {
        page
    };
    assert_eq!(vec![rows[0].document.clone()], vec![expected]);
    let before = s.draft.clone();
    let reopened = if is_source { open_document_center(&mut s) } else { open_topic(&mut s, topic) };
    accepted(&send(&mut s, reopened));
    assert_eq!(s.draft.income, before.income);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), estimate(&s.draft));
    s
}

fn refusal(page: IncomeSource, key: &str) {
    let s = save(page);
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported, "{key}: {:?}", r.issues);
    assert!(r.issues.iter().any(|i| i.display.key == key), "{key}: {:?}", r.issues);
}

#[test]
fn continuing_partner_loss_requires_actual_at_risk_history() {
    let mut p = partner(-1_000);
    p.basis.start = k1_1065_continuing(1_000_000, ScorpRiskStart::Unknown);
    refusal(
        IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) },
        "ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED",
    );
    let s = save(IncomeSource::K1 { value: crate::K1Document::K1Partnership((partner(-1_000)).into()) });
    assert_eq!(value(&supported(&s.draft), "agi"), 59_000);
}

#[test]
fn unsupported_participation_omits_the_item_with_a_notice() {
    let mut p = scorp(10_000);
    let mut facts = p.participation.as_ref().unwrap().record();
    facts.ordinary_scope = false;
    p.participation = Some(Participation::Other { facts });
    let s = save(IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) });
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_PARTICIPATION_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
}

#[test]
fn publicly_traded_rental_and_missing_section179_sources_are_distinct() {
    let mut rental = partner(0);
    rental.publicly_traded = true;
    rental.rentals.real_estate = 100_000;
    rental.rentals.participation = Some(crate::K1RentalParticipation::NotActive);
    // 2026-10-20: i8582 Special Instructions for PTPs uses its own pool.
    let session = save(IncomeSource::K1 { value: crate::K1Document::K1Partnership((rental).into()) });
    assert_eq!(value(&supported(&session.draft), "agi"), 61_000);
    // Without an actual AMT income-carry record, the selected-system
    // section 179 source remains unavailable.
    let mut deduction = partner(20_000);
    deduction.publicly_traded = true;
    deduction.more.section179 = 500_000;
    refusal(
        IncomeSource::K1 { value: crate::K1Document::K1Partnership((deduction).into()) },
        "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE",
    );
}

#[test]
fn taxable_loan_repayment_with_excluded_returns_requires_source_allocation() {
    // $10,000 note, $2,000 basis: a $5,000 principal repayment carries
    // $1,000 basis and $4,000 gain. The excluded-return fact cannot be
    // assigned between the taxable/untaxed portions by this worksheet.
    let mut p = scorp(0);
    p.basis.loans = vec![ScorpLoan {
        kind: ScorpLoanKind::FormalNote,
        repayment_term: ScorpNoteHolding::Single { term: GainTerm::Long },
        principal: 1_000_000,
        basis: 200_000,
        advances: 0,
        repayments: 500_000,
    }];
    p.basis.more_at_risk.excluded_returns = 100_000;
    refusal(
        IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) },
        "ISSUE_SCORP_RISK_DEBT_REPAYMENT_ALLOCATION",
    );
}

#[test]
fn carried_nondeductible_expenses_still_require_coordinated_risk_records() {
    let mut p = scorp(10_000);
    let K1ScorpStart::Continuing { carried, .. } = &mut p.basis.start else { unreachable!() };
    carried.basis_nondeductible = 100_000;
    carried.losses_first = true;
    refusal(
        IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) },
        "ISSUE_SCORP_RISK_FINANCING_UNRESOLVED",
    );
}

#[test]
fn section179_above_remaining_amount_at_risk_remains_explicit() {
    let mut p = partner(20_000);
    p.distributions = 2_000_000;
    p.basis.start =
        k1_1065_continuing(1_000_000, ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 });
    p.more.section179 = 500_000;
    refusal(
        IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) },
        "ISSUE_SECTION179_RISK_CARRYFORWARD",
    );
}

#[test]
fn current_page_whole_scope_omission_precedes_legacy_passive_history_notice() {
    // The page sets basis/risk/passive completeness together. A whole-item
    // special situation cannot independently select the old missing-only
    // passive-history state; it omits the entire K-1 first.
    let mut p = scorp(10_000);
    p.special = vec![crate::ScorpSpecial::DebtForgiveness];
    let s = save(IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) });
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_SCORP_K1_LEFT_OUT"));
    assert!(!noted(&r, "WARN_SCORP_PASSIVE_HISTORY_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
}

#[test]
fn missing_qualified_loss_history_warns_without_losing_the_income_tax_deduction() {
    let mut p = scorp(0);
    let K1ScorpStart::Continuing { carried, .. } = &mut p.basis.start else { unreachable!() };
    carried.basis_loss = 200_000;
    let s = save(IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) });
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_SCORP_QBI_HISTORY_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 58_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);

    let mut p = partner(0);
    let BasisStart::Continuing { carried, .. } = &mut p.basis.start else { unreachable!() };
    carried.basis_loss = 200_000;
    let s = save(IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) });
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_PARTNERSHIP_QBI_HISTORY_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 58_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
}

#[test]
fn scorp_property_allocation_and_unconfirmed_source_omit_only_qbi() {
    let mut p = scorp(0);
    p.more.section1231 = 2_000_000;
    p.more.unrecaptured_business = 1_200_000;
    p.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        current_loss: 0,
        property_qbi: K1ScorpPropertyQbi { gain: 800_000, qualified_1250: None, current: 0 },
    };
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 8_000, 8_000)], &[(2025, 8_000, 8_000)]));
    let r = supported(
        &save_with(d.clone(), IncomeSource::K1 { value: crate::K1Document::K1Scorp((p.clone()).into()) })
            .draft,
    );
    assert!(noted(&r, "WARN_SCORP_QBI_ALLOCATION_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    p.qbi_records.as_mut().unwrap().property_complete = false;
    let r = supported(
        &save_with(d.clone(), IncomeSource::K1 { value: crate::K1Document::K1Scorp((p.clone()).into()) })
            .draft,
    );
    assert!(noted(&r, "WARN_SCORP_QBI_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    p.more.section1231 = -200_000;
    p.more.unrecaptured_business = 0;
    if let K1ScorpQbi::Statement { property_qbi, .. } = &mut p.qbi {
        *property_qbi = K1ScorpPropertyQbi { gain: 0, qualified_1250: None, current: 100_000 };
    }
    let r =
        supported(&save_with(d, IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) }).draft);
    assert!(noted(&r, "WARN_SCORP_QBI_LOSS_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 58_000);
}

#[test]
fn current_ptp_profit_sstb_and_retained_ordinary_basis_loss_compute() {
    let mut p = partner(10_000);
    p.publicly_traded = true;
    p.qbi = PartnerQbi::Statement {
        section1231_qbi: 0,
        ptp_income: 0,
        reit_dividends: 0,
        income: 1_000_000,
        wages: 0,
        property: 0,
        service: false,
    };
    let r = supported(
        &save(IncomeSource::K1 { value: crate::K1Document::K1Partnership((p.clone()).into()) }).draft,
    );
    assert_eq!(value(&r, "agi"), 70_000);
    assert_eq!(value(&r, "qbi_deduction"), 2_000);
    if let PartnerQbi::Statement { service, .. } = &mut p.qbi {
        *service = true;
    }
    // 2026-10-15 dated correction: approved QBI map §5.4 and Reg.
    // 1.199A-3(c)(3)(ii)/1.199A-5(a)(2) admit an actual SSTB PTP source.
    // Below the 2026 threshold its same $10,000 income remains fully eligible.
    // The former generic Save-refusal expectation is retained in the audit.
    // Original income-source-3540: the approved current SSTB PTP answer
    // computes through the same typed owner, with complete editor lifecycle.
    let draft = complete(FederalStatus::Single);
    let expected = expected_named_sources(
        &draft,
        Owner::Taxpayer,
        vec![IncomeSource::K1 { value: crate::K1Document::K1Partnership(p.clone().into()) }],
    ).remove(0);
    let IncomeSource::K1 { value: crate::K1Document::K1Partnership(expected) } = expected else {
        unreachable!()
    };
    let mut original_source = expected.record();
    original_source.source.as_mut().unwrap().assigned = crate::DocumentRecipient::Taxpayer;
    original_source.source.as_mut().unwrap().owner_source = Some(crate::OwnerSource::Matched);
    let peers = draft.income.clone();
    let mut session = Interview::new(TEST_USER, draft.clone());
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(0, IncomeSource::K1 { value: crate::K1Document::K1Partnership((p.clone()).into()) })],
    );
    current_source_acceptance("income-source-3540", &mut session, request);
    let saved = session.draft.clone();
    let reopened = open_document_center(&mut session);
    let rows: Vec<_> = page_documents(&reopened)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::K1 { .. }))
        .collect();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].id > 0);
    assert_eq!(rows[0].owner, crate::DocumentOwner::Yours);
    assert_eq!(rows[0].community, CommunityShare::NotNeeded);
    let IncomeSource::K1 { value: crate::K1Document::K1Partnership(shown) } = &rows[0].document
    else {
        unreachable!()
    };
    assert_eq!(shown, &expected);
    let mut reopened_source = shown.record();
    reopened_source.source.as_mut().unwrap().assigned = crate::DocumentRecipient::Taxpayer;
    reopened_source.source.as_mut().unwrap().owner_source = Some(crate::OwnerSource::Matched);
    assert_eq!(reopened_source, original_source);
    let actual = saved.income.iter().find(|entry| entry.id == rows[0].id).unwrap();
    assert_eq!(actual.owner, rows[0].owner);
    let FederalIncome::Partnership { data } = &actual.income else { unreachable!() };
    assert_eq!(data, &original_source);
    for peer in peers {
        assert_eq!(saved.income.iter().find(|entry| entry.id == peer.id), Some(&peer));
    }
    let r = supported(&saved);
    assert_eq!(value(&r, "agi"), 70_000);
    assert_eq!(value(&r, "qbi_deduction"), 2_000);
    if let PartnerQbi::Statement { service, .. } = &mut p.qbi {
        *service = false;
    }
    let BasisStart::Continuing { carried, .. } = &mut p.basis.start else { unreachable!() };
    carried.basis_loss = 100_000;
    p.qbi_records.as_mut().unwrap().history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2025, 1_000, 500, 1_000, 500)],
        risk: vec![],
        passive: vec![],
    });
    // 2026-10-19 i8995 Tracking Losses: the actual prior ordinary basis
    // loss now retains its original50% qualified portion on release.
    let result =
        supported(&save(IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) }).draft);
    assert_eq!(value(&result, "agi"), 69_000);
    assert_eq!(value(&result, "qbi_reit_ptp"), 9_500);
    assert_eq!(value(&result, "qbi_deduction"), 1_900);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
}

#[test]
fn multiple_same_box_activities_are_refused_instead_of_merged_into_one_ledger() {
    let mut p = partner(10_000);
    p.more.unentered = vec![PartnerK1Item::MultipleActivities];
    refusal(
        IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) },
        "ISSUE_PARTNER_K1_MULTIPLE_ACTIVITIES",
    );
    let mut p = scorp(10_000);
    p.more.unentered = vec![ScorpK1Item::MultipleActivities];
    refusal(
        IncomeSource::K1 { value: crate::K1Document::K1Scorp((p).into()) },
        "ISSUE_SCORP_K1_MULTIPLE_ACTIVITIES",
    );
}

#[test]
fn missing_partner_gross_and_unconfirmed_business_wages_are_distinct_notices() {
    let mut p = partner(10_000);
    (p.participation, p.limited_for_se) = (Some(crate::PartnerParticipation::General { facts: (Participation::Over500Hours { hours: 600 }).record() }), Some(false));
    p.se = 1_000_000;
    p.qbi = PartnerQbi::Statement {
        section1231_qbi: 0,
        ptp_income: 0,
        reit_dividends: 0,
        income: 1_000_000,
        wages: 0,
        property: 0,
        service: false,
    };
    let r = supported(&save(IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) }).draft);
    assert!(noted(&r, "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 69_293);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    let mut b = business(1_000_000);
    b.name = "Unit22 synthetic business".into();
    b.more.payroll = BusinessPayroll::Yes {
        statements: vec![PayrollW2 { label: "Employee".into(), wages: 100_000, medicare: 100_000 }],
        attribution: crate::PayrollQbiAttribution::Unavailable,
    };
    let r =
        supported(&save(IncomeSource::Record { value: crate::IncomeRecord::Business(b) }).draft);
    assert!(noted(&r, "WARN_QBI_WAGES_LEFT_OUT"));
}

#[test]
fn sold_rental_ordinary_section1231_qbi_survives_native_save_and_restart() {
    let mut p = rental_sold_page(20261231, 10_000_000);
    p.more.qbi = RentalBusiness::Yes { material_participation: false };
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&{
        let request = save_topic(
            &mut s,
            TaxSection::Rentals,
            vec![
                document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(p) }),
                document(
                    0,
                    IncomeSource::Record {
                        value: crate::IncomeRecord::BusinessPropertyLosses(
                            crate::BusinessPropertyLosses { regular: vec![], amt: vec![] },
                        ),
                    },
                ),
            ],
        );
        send(&mut s, request)
    });
    // 2026-10-17: QBI-G7 integration supersedes the old named refusal;
    // i8995 includes an established rental business's ordinary1231 loss.
    let r = supported(&s.draft);
    assert!(value(&r, "qbi_loss_carryforward") > 0);
    let saved = s.draft.clone();
    let request = open_topic(&mut s, TaxSection::Rentals);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft, saved);
    let mut cold = restart(saved.clone());
    let request = open_topic(&mut cold, TaxSection::Rentals);
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), r);
}

#[test]
fn k1_qbi_sources_stay_separate_without_an_explicit_aggregation_election() {
    let mut a = scorp(100_000);
    a.corporation.name = "Unit22 synthetic entity A".into();
    a.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 10_000_000,
        wages: 0,
        property: 0,
        service: false,
        current_loss: 0,
        property_qbi: no_property_qbi(),
    };
    let mut b = scorp(25_000);
    b.corporation.name = "Unit22 synthetic entity B".into();
    b.qbi = K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 2_500_000,
        wages: 12_000_000,
        property: 0,
        service: false,
        current_loss: 0,
        property_qbi: no_property_qbi(),
    };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 40_000_000, 0);
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&{
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut s,
            vec![
                document(0, IncomeSource::K1 { value: crate::K1Document::K1Scorp((a).into()) }),
                document(0, IncomeSource::K1 { value: crate::K1Document::K1Scorp((b).into()) }),
            ],
        );
        crate::tests::support::client::income_pages::save_source_fixture_request(&mut s, request)
    });
    let r = supported(&s.draft);
    // Above the phase-in ceiling: first column min(20,000,0)=0;
    // second min(5,000,60,000)=5,000. An eligible elected aggregation
    // would instead allow min(25,000,60,000)=25,000. This fixture
    // supplies no group or election facts, so its sources stay separate.
    assert_eq!(value(&r, "agi"), 525_000);
    assert_eq!(value(&r, "qbi_deduction"), 5_000);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
}

#[test]
fn excess_loss_character_of_mixed_k1_and_direct_business_sources_stays_explicit() {
    let mut p = partner(-400_000);
    p.basis.start = k1_1065_continuing(
        50_000_000,
        ScorpRiskStart::Detailed { capacity: 50_000_000, profit: 0, loss: 0 },
    );
    p.qbi = PartnerQbi::Statement {
        section1231_qbi: 0,
        ptp_income: 0,
        reit_dividends: 0,
        income: -24_000_000,
        wages: 0,
        property: 0,
        service: false,
    };
    let mut s = save(IncomeSource::K1 { value: crate::K1Document::K1Partnership((p).into()) });
    let mut b = business(1_000_000);
    b.name = "Unit22 synthetic direct trade".into();
    accepted(&{
        let request = save_topic(
            &mut s,
            TaxSection::SelfEmployment,
            vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(b) })],
        );
        send(&mut s, request)
    });
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues
            .iter()
            .any(|i| i.display.key == "ISSUE_PARTNERSHIP_QBI_PROFIT_OFFSET_PARTNERSHIP_EXCESS_LOSSES_REQUIRE_CURRENT_97A91752"),
        "{:?}",
        r.issues
    );
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
}

#[test]
fn retained_basis_expenses_without_risk_history_preserve_expenses_and_losses() {
    let page = scorp_retained_expense_page();
    let mut regular = page.record();
    regular.amt = Some(amt_records(&regular));
    regular.amt.as_mut().unwrap().prior_nondeductible = 125_000;
    for (source, expenses, closing) in [
        (crate::scorp_amt::ScorpSource::from(&regular), 150_000, 1_650_000),
        (regular.amt.as_ref().unwrap().source(&regular).unwrap(), 175_000, 1_625_000),
    ] {
        let basis = source.basis_result().unwrap();
        let risk = source.risk_result(&basis).unwrap();
        assert_eq!(basis.stock.get(), closing);
        assert_eq!(basis.nondeductible_used.get(), expenses);
        assert_eq!(basis.nondeductible_carry.get(), 0);
        assert_eq!((risk.investment, risk.year_end), (None, None));
        assert_eq!(risk.income.get(), 1_000_000);
        assert_eq!(risk.nondeductible_used.get(), expenses);
        assert_eq!(risk.nondeductible_suspended.get(), 0);
        assert_eq!(risk.ledger.risk_allowed[crate::k1_loss::K1LossClass::Ordinary].get(), 200_000);
        assert!(!risk.ledger.risk_suspended.any());
    }
}

#[test]
fn retained_basis_expense_boundary_and_remaining_risk_refusals() {
    use crate::scorp_risk::ScorpRiskError::*;
    let mut source = scorp_retained_expense_page().record();
    source.amt = None;
    // i6198: $2,000 released loss + $500 current expense + $7,500
    // released expense exactly exhausts $10,000 activity income.
    source.prior_nondeductible = 750_000;
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.nondeductible_used.get(), 800_000);
    assert_eq!(source.risk.calculate(&source, &basis).unwrap().nondeductible_used.get(), 800_000);
    for excess in [1, 100] {
        let mut over = source.clone();
        over.prior_nondeductible += excess;
        let basis = over.basis_result().unwrap();
        assert_eq!(basis.nondeductible_used.get(), 800_000 + excess);
        assert_eq!(over.risk.calculate(&over, &basis).unwrap_err(), FinancingUnresolved);
    }
    for (edit, expected) in [
        ((|s: &mut ScorpK1| s.risk.complete = false) as fn(&mut ScorpK1), FinancingUnresolved),
        (|s| s.risk.prior_nondeductible = 1, FinancingUnresolved),
        (|s| s.risk.prior_loss = 1, FinancingUnresolved),
        (|s| s.risk.deducted = 1, FinancingUnresolved),
        (|s| s.section179 = 100, FinancingUnresolved),
        (|s| s.carried.section179 = 100, FinancingUnresolved),
        (
            |s| s.risk.start = ScorpRiskStart::Detailed { capacity: 1_300_000, profit: 0, loss: 0 },
            FinancingUnresolved,
        ),
        (
            |s| {
                s.opening = Some(0);
                s.loans.clear();
                s.prior_nondeductible += 1;
            },
            FinancingUnresolved,
        ),
        (
            |s| {
                s.prior_loss = 3_000_000;
            },
            FinancingUnresolved,
        ),
        (|s| s.risk.excluded_funding = 1, SourceInconsistent),
    ] {
        let mut control = source.clone();
        edit(&mut control);
        let basis = if expected == SourceInconsistent {
            source.basis_result().unwrap()
        } else {
            control.basis_result().unwrap()
        };
        assert_eq!(control.risk.calculate(&control, &basis).unwrap_err(), expected);
    }
    // 2026-10-23 i6198 line 5: at the exact no-loss threshold the
    // investment/allocation is unused; one cent more needs that allocation.
    source.loans[0].basis = 0;
    source.loans[0].repayments = 100;
    source.loans[0].repayment_term = ScorpNoteHolding::Single { term: GainTerm::Long };
    source.risk.excluded_returns = 1;
    let basis = source.basis_result().unwrap();
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(risk.nondeductible_used.get(), 800_000);
    assert_eq!((risk.investment, risk.year_end), (None, None));
    assert_eq!(basis.loans[0].capital_gain.get(), 100);
    source.prior_nondeductible += 1;
    let basis = source.basis_result().unwrap();
    assert_eq!(source.risk.calculate(&source, &basis).unwrap_err(), DebtRepaymentAllocation);
}

#[test]
fn retained_basis_expense_native_correction_preserves_unknown_history() {
    let mut page = scorp_retained_expense_page();
    let mut session = save(IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) });
    assert_eq!(value(&supported(&session.draft), "agi"), 68_000);
    // Correct the actual retained expense to one cent above the no-loss
    // boundary, then to the exact boundary through the same native editor.
    for (retained, supported_result) in [(750_001, false), (750_000, true)] {
        let K1ScorpStart::Continuing { carried, risk_history, .. } = &mut page.basis.start else {
            unreachable!()
        };
        assert_eq!(*risk_history, ScorpRiskStart::Unknown);
        carried.basis_nondeductible = retained;
        let mut request = open_document_center(&mut session);
        crate::tests::support::builders::k1res::documents(&mut request)[0].document =
            crate::K1Document::K1Scorp((page.clone()).into());
        accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(&mut session, request));
        let result = estimate(&session.draft);
        if supported_result {
            assert_eq!(value(&supported(&session.draft), "agi"), 68_000);
        } else {
            assert_eq!(result.state, ResultState::Unsupported);
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_SCORP_RISK_FINANCING_UNRESOLVED")
            );
        }
        let saved = session.draft.clone();
        session = restart(saved.clone());
        let request = open_document_center(&mut session);
        let shown = page_documents(&request)
            .into_iter()
            .find(|row| matches!(row.document, IncomeSource::K1 { .. }))
            .unwrap();
        // The fixture save prints the household's shareholder name into the
        // blank block (fixture_blank_owner_inputs); that printed value returns.
        assert_eq!(
            shown.document,
            expected_named_source(
                &session.draft,
                Owner::Taxpayer,
                IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) },
            )
        );
        accepted(&send(&mut session, request));
        assert_eq!(session.draft.income, saved.income);
        assert_eq!(estimate(&session.draft), result);
    }
}

#[test]
fn no_loss_repayments_preserve_regular_and_amt_loan_gain_character() {
    for kind in [ScorpLoanKind::FormalNote, ScorpLoanKind::OpenAccount] {
        let mut regular = scorp_no_loss_repayment_page(kind).record();
        regular.amt = Some(amt_records(&regular));
        regular.amt.as_mut().unwrap().loans[0].basis = 400_000;
        for (source, closing, recovered, gain) in [
            (crate::scorp_amt::ScorpSource::from(&regular), 100_000, 100_000, 400_000),
            (regular.amt.as_ref().unwrap().source(&regular).unwrap(), 200_000, 200_000, 300_000),
        ] {
            let basis = source.basis_result().unwrap();
            assert_eq!(basis.stock.get(), 1_000_000);
            assert_eq!(basis.loans[0].principal.get(), 500_000);
            assert_eq!(basis.loans[0].basis.get(), closing);
            assert_eq!(basis.loans[0].nontaxable_repayment.get(), recovered);
            assert_eq!(
                source.loan_gains(&basis).unwrap(),
                if kind == ScorpLoanKind::FormalNote { [0, 0, gain] } else { [gain, 0, 0] }
            );
            let risk = source.risk_result(&basis).unwrap();
            assert_eq!(risk.income.get(), 0);
            assert_eq!((risk.investment, risk.year_end), (None, None));
            assert_eq!(risk.recapture.income.get(), 0);
            assert!(!risk.ledger.risk_suspended.any());
        }
    }
}

#[test]
fn no_loss_repayment_uses_supplied_events_and_preserves_refusal_boundaries() {
    use crate::scorp_risk::ScorpRiskError::*;
    let mut source = scorp_no_loss_repayment_page(ScorpLoanKind::FormalNote).record();
    source.amt = None;
    let basis = source.basis_result().unwrap();
    let events = crate::scorp::ScorpEvents { disposal: source.disposal(&basis), loans: [0; 3] };
    // The supplied event context, rather than the loan's original taxable
    // gain, owns the allocation gate. Neither gain supplies activity income.
    assert_eq!(
        source.risk.calculate_with_events(&source, &basis, Some(&events)).unwrap().income.get(),
        0
    );
    for (edit, expected) in [
        ((|s: &mut ScorpK1| s.ordinary = -1) as fn(&mut ScorpK1), DebtRepaymentAllocation),
        (|s| s.risk.prior_loss = 1, DebtRepaymentAllocation),
        (|s| s.risk.prior_nondeductible = 1, DebtRepaymentAllocation),
        (|s| s.risk.deducted = 1, DebtRepaymentAllocation),
        (|s| s.section179 = 1, DebtRepaymentAllocation),
        (|s| s.risk.complete = false, FinancingUnresolved),
        (|s| s.risk.excluded_returns = 500_001, SourceInconsistent),
        (
            |s| s.risk.start = ScorpRiskStart::Detailed { capacity: 1_300_000, profit: 0, loss: 0 },
            DebtRepaymentAllocation,
        ),
    ] {
        let mut control = source.clone();
        edit(&mut control);
        let basis = if expected == SourceInconsistent {
            source.basis_result().unwrap()
        } else {
            control.basis_result().unwrap()
        };
        assert_eq!(control.risk.calculate(&control, &basis).unwrap_err(), expected);
    }
    source.ordinary = -1;
    let basis = source.basis_result().unwrap();
    assert_eq!(
        source.risk.calculate_with_events(&source, &basis, Some(&events)).unwrap_err(),
        HistoryNeeded
    );
    let taxable_events = crate::scorp::ScorpEvents { loans: [0, 0, 1], ..events };
    assert_eq!(
        source.risk.calculate_with_events(&source, &basis, Some(&taxable_events)).unwrap_err(),
        DebtRepaymentAllocation
    );
    // Basis may suspend a class without any ND coordination: C9 still
    // admits the zero allowed loss, leaving its basis carry untouched.
    source.opening = Some(0);
    source.loans[0].basis = 0;
    let basis = source.basis_result().unwrap();
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(risk.ledger.basis_suspended[crate::k1_loss::K1LossClass::Ordinary].get(), 1);
    assert_eq!(risk.ledger.risk_allowed[crate::k1_loss::K1LossClass::Ordinary].get(), 0);
}

#[test]
fn no_loss_repayment_native_correction_preserves_unknown_history() {
    let mut page = scorp_no_loss_repayment_page(ScorpLoanKind::FormalNote);
    let mut session = save(IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) });
    assert_eq!(value(&supported(&session.draft), "agi"), 64_000);
    for (ordinary, supported_result) in [(-1, false), (0, true)] {
        page.ordinary = ordinary;
        let mut request = open_document_center(&mut session);
        let rows = crate::tests::support::builders::k1res::documents(&mut request);
        let crate::K1Document::K1Scorp(printed) = &rows[0].document else { unreachable!() };
        let shareholder = printed.form.shareholder.clone();
        let mut replacement: crate::K1ScorpDocument = (page.clone()).into();
        replacement.form.shareholder = shareholder;
        rows[0].document = crate::K1Document::K1Scorp(replacement);
        accepted(&send(&mut session, request));
        let result = estimate(&session.draft);
        if supported_result {
            assert_eq!(value(&supported(&session.draft), "agi"), 64_000);
        } else {
            assert_eq!(result.state, ResultState::Unsupported);
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_SCORP_RISK_DEBT_REPAYMENT_ALLOCATION")
            );
        }
        session = restart(session.draft.clone());
        let request = open_document_center(&mut session);
        let shown = page_documents(&request)
            .into_iter()
            .find(|row| matches!(row.document, IncomeSource::K1 { .. }))
            .unwrap();
        // The correction keeps the shareholder printed on the accepted paper.
        let mut expected: crate::K1ScorpDocument = (page.clone()).into();
        expected.form.shareholder =
            fixture_document_names(&session.draft, -1).unwrap().recipient;
        assert_eq!(
            shown.document,
            IncomeSource::K1 { value: crate::K1Document::K1Scorp(expected) }
        );
        let saved = session.draft.income.clone();
        accepted(&send(&mut session, request));
        assert_eq!(session.draft.income, saved);
        assert_eq!(estimate(&session.draft), result);
    }
}

#[test]
fn fully_excluded_repayments_preserve_binding_risk_and_native_gain_character() {
    for (kind, term) in [
        (ScorpLoanKind::FormalNote, GainTerm::Short),
        (ScorpLoanKind::FormalNote, GainTerm::Long),
        (ScorpLoanKind::OpenAccount, GainTerm::Long),
        (ScorpLoanKind::FormerOpenAccount, GainTerm::Long),
    ] {
        let mut page = scorp_fully_excluded_repayment_page(kind);
        if kind == ScorpLoanKind::FormalNote {
            page.basis.loans[0].repayment_term = ScorpNoteHolding::Single { term };
        }
        let mut regular = page.record();
        regular.amt = Some(amt_records(&regular));
        let amt = regular.amt.as_mut().unwrap();
        amt.loans[0].basis = 400_000;
        amt.risk.start = ScorpRiskStart::Detailed { capacity: 150_000, profit: 0, loss: 0 };
        for (source, recovered, gain, allowed) in [
            (crate::scorp_amt::ScorpSource::from(&regular), 100_000, 400_000, 100_000),
            (regular.amt.as_ref().unwrap().source(&regular).unwrap(), 200_000, 300_000, 150_000),
        ] {
            let basis = source.basis_result().unwrap();
            assert_eq!(basis.stock.get(), 450_000);
            assert_eq!(basis.loans[0].principal.get(), 500_000);
            assert_eq!(basis.loans[0].restoration.get(), 0);
            assert_eq!(basis.loans[0].nontaxable_repayment.get(), recovered);
            assert_eq!(basis.loans[0].basis.get(), recovered);
            assert_eq!(basis.allowed[crate::k1_loss::K1LossClass::Ordinary].get(), 500_000);
            assert!(!basis.suspended.any());
            let expected_gains = if kind != ScorpLoanKind::FormalNote {
                [gain, 0, 0]
            } else if term == GainTerm::Short {
                [0, gain, 0]
            } else {
                [0, 0, gain]
            };
            assert_eq!(source.loan_gains(&basis).unwrap(), expected_gains);
            let risk = source.risk_result(&basis).unwrap();
            assert_eq!(risk.investment, Some(allowed));
            assert_eq!(risk.income.get(), 0);
            assert_eq!(risk.ledger.risk_allowed[crate::k1_loss::K1LossClass::Ordinary].get(), allowed);
            assert_eq!(
                risk.ledger.risk_suspended[crate::k1_loss::K1LossClass::Ordinary].get(),
                500_000 - allowed
            );
            assert_eq!(risk.year_end, Some(0));
            assert_eq!(risk.recapture.income.get(), 0);
            assert_eq!(risk.recapture.next_year_deduction.get(), 0);
        }
    }
}

#[test]
fn fully_excluded_repayment_boundaries_and_native_correction() {
    use crate::scorp_risk::ScorpRiskError::*;
    let mut page = scorp_fully_excluded_repayment_page(ScorpLoanKind::FormalNote);
    let source = page.record();
    let basis = source.basis_result().unwrap();
    let events = crate::scorp::ScorpEvents {
        disposal: source.disposal(&basis),
        loans: source.loan_gains(&basis).unwrap(),
    };
    assert_eq!(
        source.risk.calculate_with_events(&source, &basis, Some(&events)).unwrap_err(),
        DebtRepaymentAllocation
    );
    for (edit, expected) in [
        ((|s: &mut ScorpK1| s.risk.excluded_returns -= 1) as fn(&mut ScorpK1), DebtRepaymentAllocation),
        (|s| s.risk.start = ScorpRiskStart::Unknown, DebtRepaymentAllocation),
        (|s| s.risk.complete = false, FinancingUnresolved),
        (
            |s| {
                s.loans[0].kind = ScorpLoanKind::OpenAccount;
                s.loans[0].repayment_term = ScorpNoteHolding::NoRepayment;
                s.loans[0].advances = 1;
            },
            DebtRepaymentAllocation,
        ),
    ] {
        let mut control = source.clone();
        edit(&mut control);
        let basis = control.basis_result().unwrap();
        assert_eq!(control.risk.calculate(&control, &basis).unwrap_err(), expected);
    }
    let mut session = save(IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) });
    assert_eq!(value(&supported(&session.draft), "agi"), 63_000);
    let source_id = session.draft.income.iter()
        .find(|row| matches!(row.income, FederalIncome::Scorp { .. })).unwrap().id;
    for excluded in [549_999, 550_000] {
        page.basis.more_at_risk.excluded_returns = excluded;
        // Excluded returns are personal at-risk records on the keyed topic;
        // the printed K-1 and its matched recipient remain unchanged.
        let request = crate::tests::support::client::income_pages::k1_fixture_answer_request(
            &mut session,
            &[document(source_id, IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) })],
        ).unwrap();
        accepted(&send(&mut session, request));
        let result = estimate(&session.draft);
        if excluded == 550_000 {
            assert_eq!(value(&supported(&session.draft), "agi"), 63_000);
        } else {
            assert_eq!(result.state, ResultState::Unsupported);
            assert!(result.issues.iter().any(|issue| {
                issue.display.key == "ISSUE_SCORP_RISK_DEBT_REPAYMENT_ALLOCATION"
            }));
        }
        session = restart(session.draft.clone());
        let request = open_document_center(&mut session);
        let shown = page_documents(&request)
            .into_iter()
            .find(|row| matches!(row.document, IncomeSource::K1 { .. }))
            .unwrap();
        assert_eq!(
            shown.document,
            expected_named_source(
                &session.draft, Owner::Taxpayer,
                IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) },
            )
        );
        let saved = session.draft.income.clone();
        accepted(&send(&mut session, request));
        assert_eq!(session.draft.income, saved);
        assert_eq!(estimate(&session.draft), result);
    }
}

#[test]
fn current_expenses_share_binding_risk_after_independent_basis_limits() {
    use crate::k1_loss::K1LossClass::Ordinary;
    let regular = scorp_binding_current_expense_page().record();
    for (source, allowed, suspended, risk_allowed, risk_suspended) in [
        (crate::scorp_amt::ScorpSource::from(&regular), 30_000, 70_000, 15_000, 15_000),
        (regular.amt.as_ref().unwrap().source(&regular).unwrap(), 50_000, 50_000, 25_000, 25_000),
    ] {
        let basis = source.basis_result().unwrap();
        let risk = source.risk_result(&basis).unwrap();
        // i7203 then i6198: expenses use basis once, then share risk with
        // the admitted loss. The $700/$500 basis holds never enter risk.
        assert_eq!(basis.stock.get(), 0);
        assert_eq!(basis.nondeductible_used.get(), 20_000);
        assert_eq!(basis.nondeductible_carry.get(), 0);
        assert_eq!(basis.allowed[Ordinary].get(), allowed);
        assert_eq!(basis.suspended[Ordinary].get(), suspended);
        assert_eq!(risk.ledger.basis_allowed, basis.allowed);
        assert_eq!(risk.ledger.basis_suspended, basis.suspended);
        assert_eq!(risk.ledger.risk_allowed[Ordinary].get(), risk_allowed);
        assert_eq!(risk.ledger.risk_suspended[Ordinary].get(), risk_suspended);
        assert_eq!(risk.nondeductible_used.get(), 10_000);
        assert_eq!(risk.nondeductible_suspended.get(), 10_000);
        assert_eq!(risk.income.get(), 0);
        assert_eq!(risk.year_end, Some(0));
        assert_eq!(risk.recapture.income.get(), 0);
    }
}

#[test]
fn binding_current_expenses_preserve_partial_admission_and_history_boundaries() {
    use crate::scorp_risk::ScorpRiskError::*;
    let original = scorp_binding_current_expense_page().record();
    for (edit, expected) in [
        ((|s: &mut ScorpK1| s.opening = Some(19_999)) as fn(&mut ScorpK1), BasisRiskCoordination),
        (|s| s.losses_first = true, BasisRiskCoordination),
        (|s| s.risk.start = ScorpRiskStart::Unknown, BasisRiskCoordination),
        (|s| s.risk.prior_nondeductible = 1, BasisRiskCoordination),
        (
            |s| {
                s.losses_first = true;
                s.prior_nondeductible = 1;
            },
            FinancingUnresolved,
        ),
        (|s| s.risk.complete = false, FinancingUnresolved),
    ] {
        let mut source = original.clone();
        edit(&mut source);
        let basis = source.basis_result().unwrap();
        assert_eq!(source.risk.calculate(&source, &basis).unwrap_err(), expected);
    }
    // An expense exactly equal to basis is fully admitted; one cent more
    // above leaves an unresolved expense source. No loss is invented at equality.
    let mut source = original;
    source.opening = Some(20_000);
    let basis = source.basis_result().unwrap();
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(basis.allowed[crate::k1_loss::K1LossClass::Ordinary].get(), 0);
    assert_eq!(basis.suspended[crate::k1_loss::K1LossClass::Ordinary].get(), 100_000);
    assert_eq!(risk.nondeductible_used.get(), 20_000);
    assert_eq!(risk.nondeductible_suspended.get(), 0);
    assert_eq!(risk.year_end, Some(5_000));
}

#[test]
fn binding_current_expenses_native_correction_reopens_and_restarts() {
    let mut page = scorp_binding_current_expense_page();
    let mut session = save(IncomeSource::K1 { value: crate::K1Document::K1Scorp((page.clone()).into()) });
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 59_850);
    amt_k1_note(&result, -250, -150);
    // Correct the actual current expense. REG then admits $400 by basis
    // and $200 by risk; AMT admits $600 by basis and $300 by risk.
    page.more.nondeductible = 10_000;
    let mut request = open_document_center(&mut session);
    let crate::K1Document::K1Scorp(source) =
        &mut crate::tests::support::builders::k1res::documents(&mut request)[0].document
    else {
        unreachable!()
    };
    source.form.basis.iter_mut().find(|row| row.code == "C").unwrap().amount = Some(page.more.nondeductible);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 59_800);
    amt_k1_note(&result, -300, -200);
    let saved = session.draft.clone();
    session = restart(saved.clone());
    let request = open_document_center(&mut session);
    let shown = page_documents(&request)
        .into_iter()
        .find(|row| matches!(row.document, IncomeSource::K1 { .. }))
        .unwrap();
    assert_eq!(
        shown.document,
        expected_named_source(
            &session.draft,
            Owner::Taxpayer,
            IncomeSource::K1 { value: crate::K1Document::K1Scorp((page).into()) },
        )
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income, saved.income);
    assert_eq!(supported(&session.draft), result);
}
