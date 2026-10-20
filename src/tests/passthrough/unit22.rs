// Unit22 verifies the surviving inventory cases through the real K-1 pages.
// These are synthetic source facts. Refusals describe App scope, not a legal
// prohibition. Sources: IRS i6198, i1065sk1, i7203 and i8995a.
use super::*;

fn partner(profit: i64) -> K1Partnership {
    let mut p = k1_1065("Unit22 synthetic partnership");
    p.ordinary = profit * 100;
    p.interest = PartnerInterest::LimitedPartner {
        participation: LimitedParticipation::Over500Hours { hours: 600 },
    };
    p.basis.start = k1_1065_continuing(
        1_000_000,
        ScorpRiskStart::Detailed {
            capacity: 1_000_000,
            profit: 0,
            loss: 0,
        },
    );
    p.qbi = PartnerQbi::NoStatement;
    p
}

fn scorp(profit: i64) -> K1Scorp {
    let mut p = k1_1120s_fixture(profit);
    p.qbi = K1ScorpQbi::NoStatement;
    p
}

fn save(page: IncomeDocument) -> Interview {
    save_with(complete(FederalStatus::Single), page)
}

fn save_with(d: FederalDraft, page: IncomeDocument) -> Interview {
    let topic = topic_for(&page);
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&send(
        &mut s,
        save_topic(&d, topic, vec![document(0, page.clone())]),
    ));
    let reopened = open_topic(&mut s, topic);
    assert_eq!(page_documents(&reopened)[0].document, page);
    let before = s.draft.clone();
    let rows = page_documents(&open_topic(&mut s, topic)).to_vec();
    accepted(&send(&mut s, save_topic(&before, topic, rows)));
    assert_eq!(s.draft.income, before.income);
    assert_eq!(
        estimate(&restart(s.draft.clone()).draft),
        estimate(&s.draft)
    );
    s
}

fn refusal(page: IncomeDocument, key: &str) {
    let s = save(page);
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported, "{key}: {:?}", r.issues);
    assert!(
        r.issues.iter().any(|i| i.display.key == key),
        "{key}: {:?}",
        r.issues
    );
}

#[test]
fn continuing_partner_loss_requires_actual_at_risk_history() {
    let mut p = partner(-1_000);
    p.basis.start = k1_1065_continuing(1_000_000, ScorpRiskStart::Unknown);
    refusal(
        IncomeDocument::K1Partnership(p),
        "ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED",
    );
    let s = save(IncomeDocument::K1Partnership(partner(-1_000)));
    assert_eq!(value(&supported(&s.draft), "agi"), 59_000);
}

#[test]
fn unsupported_participation_omits_the_item_with_a_notice() {
    let mut p = scorp(10_000);
    let mut facts = p.participation.record();
    facts.ordinary_scope = false;
    p.participation = Participation::Other { facts };
    let s = save(IncomeDocument::K1Scorp(p));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_PARTICIPATION_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
}

#[test]
fn publicly_traded_rental_and_missing_section179_sources_remain_named_refusals() {
    let mut rental = partner(0);
    rental.publicly_traded = true;
    rental.rentals.real_estate = 100_000;
    refusal(IncomeDocument::K1Partnership(rental), "ISSUE_K1_RENTAL_PTP");
    // 2026-10-16 D6: this source supplies no AMT income-carry record;
    // preserve that unavailability instead of the retired blanket PTP refusal.
    let mut deduction = partner(20_000);
    deduction.publicly_traded = true;
    deduction.more.section179 = 500_000;
    refusal(
        IncomeDocument::K1Partnership(deduction),
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
        repayment_term: ScorpNoteHolding::Single {
            term: GainTerm::Long,
        },
        principal: 1_000_000,
        basis: 200_000,
        advances: 0,
        repayments: 500_000,
    }];
    p.basis.more_at_risk.excluded_returns = 100_000;
    refusal(
        IncomeDocument::K1Scorp(p),
        "ISSUE_SCORP_RISK_DEBT_REPAYMENT_ALLOCATION",
    );
}

#[test]
fn carried_nondeductible_expenses_still_require_coordinated_risk_records() {
    let mut p = scorp(10_000);
    let K1ScorpStart::Continuing { carried, .. } = &mut p.basis.start else {
        unreachable!()
    };
    carried.basis_nondeductible = 100_000;
    carried.losses_first = true;
    refusal(
        IncomeDocument::K1Scorp(p),
        "ISSUE_SCORP_RISK_FINANCING_UNRESOLVED",
    );
}

#[test]
fn section179_above_remaining_amount_at_risk_remains_explicit() {
    let mut p = partner(20_000);
    p.distributions = 2_000_000;
    p.basis.start = k1_1065_continuing(
        1_000_000,
        ScorpRiskStart::Detailed {
            capacity: 0,
            profit: 0,
            loss: 0,
        },
    );
    p.more.section179 = 500_000;
    refusal(
        IncomeDocument::K1Partnership(p),
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
    let s = save(IncomeDocument::K1Scorp(p));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_SCORP_K1_LEFT_OUT"));
    assert!(!noted(&r, "WARN_SCORP_PASSIVE_HISTORY_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 60_000);
}

#[test]
fn missing_qualified_loss_history_warns_without_losing_the_income_tax_deduction() {
    let mut p = scorp(0);
    let K1ScorpStart::Continuing { carried, .. } = &mut p.basis.start else {
        unreachable!()
    };
    carried.basis_loss = 200_000;
    let s = save(IncomeDocument::K1Scorp(p));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_SCORP_QBI_HISTORY_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 58_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);

    let mut p = partner(0);
    let BasisStart::Continuing { carried, .. } = &mut p.basis.start else {
        unreachable!()
    };
    carried.basis_loss = 200_000;
    let s = save(IncomeDocument::K1Partnership(p));
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
        property_qbi: K1ScorpPropertyQbi {
            gain: 800_000,
            qualified_1250: None,
            current: 0,
        },
    };
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(
        &[(2025, 8_000, 8_000)],
        &[(2025, 8_000, 8_000)],
    ));
    let r = supported(&save_with(d.clone(), IncomeDocument::K1Scorp(p.clone())).draft);
    assert!(noted(&r, "WARN_SCORP_QBI_ALLOCATION_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    p.qbi_records.property_complete = false;
    let r = supported(&save_with(d.clone(), IncomeDocument::K1Scorp(p.clone())).draft);
    assert!(noted(&r, "WARN_SCORP_QBI_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    p.more.section1231 = -200_000;
    p.more.unrecaptured_business = 0;
    if let K1ScorpQbi::Statement { property_qbi, .. } = &mut p.qbi {
        *property_qbi = K1ScorpPropertyQbi {
            gain: 0,
            qualified_1250: None,
            current: 100_000,
        };
    }
    let r = supported(&save_with(d, IncomeDocument::K1Scorp(p)).draft);
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
    let r = supported(&save(IncomeDocument::K1Partnership(p.clone())).draft);
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
    let original_source = p.record();
    let draft = complete(FederalStatus::Single);
    let peers = draft.income.clone();
    let mut session = Interview::new(TEST_USER, draft.clone());
    let request = save_topic(
        &draft,
        TaxSection::PassThrough,
        vec![document(0, IncomeDocument::K1Partnership(p.clone()))],
    );
    current_source_acceptance("income-source-3540", &mut session, request);
    let saved = session.draft.clone();
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    let rows = page_documents(&reopened);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].id > 0);
    assert_eq!(rows[0].owner, Owner::Taxpayer);
    assert_eq!(rows[0].community, CommunityShare::NotNeeded);
    let IncomeDocument::K1Partnership(shown) = &rows[0].document else { unreachable!() };
    assert_eq!(shown.record(), original_source);
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
    p.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2025, 1_000, 500, 1_000, 500)],
        risk: vec![],
        passive: vec![],
    });
    // 2026-10-19 i8995 Tracking Losses: the actual prior ordinary basis
    // loss now retains its original50% qualified portion on release.
    let result = supported(&save(IncomeDocument::K1Partnership(p)).draft);
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
        IncomeDocument::K1Partnership(p),
        "ISSUE_PARTNER_K1_MULTIPLE_ACTIVITIES",
    );
    let mut p = scorp(10_000);
    p.more.unentered = vec![ScorpK1Item::MultipleActivities];
    refusal(
        IncomeDocument::K1Scorp(p),
        "ISSUE_SCORP_K1_MULTIPLE_ACTIVITIES",
    );
}

#[test]
fn missing_partner_gross_and_unconfirmed_business_wages_are_distinct_notices() {
    let mut p = partner(10_000);
    p.interest = PartnerInterest::General {
        participation: Participation::Over500Hours { hours: 600 },
    };
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
    let r = supported(&save(IncomeDocument::K1Partnership(p)).draft);
    assert!(noted(&r, "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 69_293);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    let mut b = business(1_000_000);
    b.name = "Unit22 synthetic business".into();
    b.more.payroll = BusinessPayroll::Yes {
        statements: vec![PayrollW2 {
            label: "Employee".into(),
            wages: 100_000,
            medicare: 100_000,
        }],
        attribution: crate::PayrollQbiAttribution::Unavailable,
    };
    let r = supported(&save(IncomeDocument::Business(b)).draft);
    assert!(noted(&r, "WARN_QBI_WAGES_LEFT_OUT"));
}

#[test]
fn sold_rental_ordinary_section1231_qbi_survives_native_save_and_restart() {
    let mut p = rental_sold_page(20261231, 10_000_000);
    p.more.qbi = RentalBusiness::Yes { material_participation: false };
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&send(
        &mut s,
        save_topic(
            &d,
            TaxSection::Rentals,
            vec![
                document(0, IncomeDocument::Rental(p)),
                document(
                    0,
                    IncomeDocument::BusinessPropertyLosses(crate::BusinessPropertyLosses {
                        regular: vec![],
                        amt: vec![],
                    }),
                ),
            ],
        ),
    ));
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
    a.name = "Unit22 synthetic entity A".into();
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
    b.name = "Unit22 synthetic entity B".into();
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
    accepted(&send(
        &mut s,
        save_topic(
            &d,
            TaxSection::PassThrough,
            vec![
                document(0, IncomeDocument::K1Scorp(a)),
                document(0, IncomeDocument::K1Scorp(b)),
            ],
        ),
    ));
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
        ScorpRiskStart::Detailed {
            capacity: 50_000_000,
            profit: 0,
            loss: 0,
        },
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
    let mut s = save(IncomeDocument::K1Partnership(p));
    let before = s.draft.clone();
    let mut b = business(1_000_000);
    b.name = "Unit22 synthetic direct trade".into();
    accepted(&send(
        &mut s,
        save_topic(
            &before,
            TaxSection::SelfEmployment,
            vec![document(0, IncomeDocument::Business(b))],
        ),
    ));
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
