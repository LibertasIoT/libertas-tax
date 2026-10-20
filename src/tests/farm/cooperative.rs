use super::*;
use crate::business_rules::{QbiActivity, QbiFiling, QbiOther, QbiResult};
use crate::tests::support::client::survey::survey_preview_document;

// These are isolated IRS QBI examples with the source's original 2020/2021
// facts and expressly supplied taxable income/QBI. They are not completed
// 2026 federal returns: source-unspecified SE or other deductions are not
// silently manufactured. All source taxable incomes remain below both years'
// thresholds. Separate public-page fixtures exercise current return routing.
fn irs_patron(numbers: [i64; 9], safe: bool) -> (QbiResult, Vec<[i64; 2]>) {
    let [
        sales,
        distribution,
        costs,
        wages,
        qualified,
        allocated_costs,
        allocated_wages,
        dpad,
        taxable,
    ] = numbers;
    let mut d = empty_return(FederalStatus::Joint);
    let mut farm = raised_farm(sales, costs);
    let FederalIncome::Business { data } = &mut farm else { unreachable!() };
    data.expenses = vec![
        ordinary_business_expense(BusinessExpenseCategory::Feed, (costs - wages) * 100),
        ordinary_business_expense(BusinessExpenseCategory::Wages, wages * 100),
    ];
    if wages > 0 {
        data.payroll = BusinessPayroll::Yes {
            statements: vec![PayrollW2 {
                label: "IRS stated wages".into(),
                wages: wages * 100,
                medicare: wages * 100,
            }],
            attribution: crate::PayrollQbiAttribution::EntireBusiness,
        };
    }
    add(&mut d, income(farm));
    let mut data = FederalCooperative {
        patronage: distribution * 100,
        qualified_payments: qualified * 100,
        qualified_items: distribution * 100,
        section_199a_g: dpad * 100,
        specified_cooperative: true,
        ..Default::default()
    };
    let mut a = crate::cooperative::automatic(&data, Some(d.income[0].id));
    a.notice =
        Some(CoopNoticeStatus::Received { source: "T.D.9947(g) express notice assumption".into() });
    a.patron_allocation = Some(if safe {
        CoopPatronAllocation::SafeHarbor
    } else {
        CoopPatronAllocation::Books {
            expenses_excluding_wages: (allocated_costs - allocated_wages) * 100,
            wages: allocated_wages * 100,
            other_deductions: 0,
            source: "IRS stated actual reasonable allocation".into(),
        }
    });
    data.allocations.push(a);
    add(&mut d, income(FederalIncome::Cooperative { data }));
    assert_eq!(crate::cooperative::graph_rule(&d), None);
    let mut activities = vec![QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: sales + distribution - costs,
        wages,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    }];
    let ids = [(0, d.income[0].id)].into_iter().collect();
    let mut report = FederalResult::empty(0);
    let (parts, passed, _, _) = crate::cooperative::qbi_filing(
        &d,
        &mut activities,
        &ids,
        taxable,
        FederalStatus::Joint,
        0,
        &mut report,
    );
    assert!(report.review_notes.is_empty(), "{report:?}");
    let result = crate::business_rules::qbi_filed(
        &activities,
        taxable,
        0,
        FederalStatus::Joint,
        QbiOther::default(),
        &QbiFiling {
            patron: true,
            patron_allocations: parts.clone(),
            dpad: passed,
            ..Default::default()
        },
        &mut report,
    );
    (result, parts)
}

// IRS final T.D.9947, IRB2021-6 pp778–779; Reg1.199A-7(g)(1).
// https://www.irs.gov/pub/irs-irbs/irb21-06.pdf
#[test]
fn coop_irs_example_a_actual_wage_allocation_and_dpad() {
    let (r, p) =
        irs_patron([150000, 100000, 200000, 50000, 100000, 90000, 25000, 1000, 75000], false);
    assert_eq!(p, vec![[10000, 25000]]);
    assert_eq!(r.lines.patron_reductions, vec![900]);
    assert_eq!(r.lines.component, 9100);
    assert_eq!(r.lines.dpad, 1000);
    assert_eq!(r.deduction, 10100);
}

// Same source p778, Reg1.199A-7(g)(2): zero actual W-2 wages.
#[test]
fn coop_irs_example_b_zero_wages_needs_no_allocation_question() {
    let (r, _) = irs_patron([150000, 100000, 200000, 0, 100000, 90000, 0, 1000, 75000], false);
    assert_eq!(r.lines.patron_reductions, vec![0]);
    assert_eq!(r.deduction, 11000);
}

// Same source pp778–779, Reg1.199A-7(g)(3): qualified payments
// differ from all qualified items; no section199A(g) allocation.
#[test]
fn coop_irs_example_c_qualified_payments_are_not_all_items() {
    let (r, p) = irs_patron([95000, 160000, 210000, 30000, 150000, 125000, 18000, 0, 90000], false);
    assert_eq!(p, vec![[25000, 18000]]);
    assert_eq!(r.lines.patron_reductions, vec![2250]);
    assert_eq!(r.deduction, 6750);
}

// Same source p779, Reg1.199A-7(g)(4): actual 65/100 bushel
// allocation is a reasonable method. Source publishes no final deduction.
#[test]
fn coop_irs_example_d_reasonable_physical_allocation() {
    let (_, p) =
        irs_patron([105000, 150000, 210000, 30000, 150000, 136500, 19500, 0, 90000], false);
    assert_eq!(p, vec![[13500, 19500]]);
}

// Same source p779, Reg1.199A-7(g)(5): safe harbor 20000/200000,
// expense15000, wages5000, qualified QBI5000, patron reduction450.
#[test]
fn coop_irs_example_e_safe_harbor_and_passed_deduction() {
    let (r, p) = irs_patron([180000, 20000, 150000, 50000, 20000, 0, 0, 1800, 100000], true);
    assert_eq!(p, vec![[5000, 5000]]);
    assert_eq!(r.lines.patron_reductions, vec![450]);
    assert_eq!(r.deduction, 11350);
}

// T.D.9947, IRB2021-6 p777, Reg1.199A-7(d)(3)(ii): related
// SSTB dividend50, actual paid deductible expense1000; SSTB expense50,
// non-SSTB expense950. Added receipts10000/zero wages isolate allocation;
// neither the source nor this fixture claims an additional tax deduction.
#[test]
fn coop_irs_example_f_caps_actual_related_expense_without_rededucting_it() {
    let mut d = empty_return(FederalStatus::Single);
    let mut farm = raised_farm(10000, 1000);
    let FederalIncome::Business { data } = &mut farm else { panic!() };
    data.expenses[0].category = BusinessExpenseCategory::Feed;
    add(&mut d, income(farm));
    let activity = d.income[0].id;
    let mut data = FederalCooperative { patronage: 5000, sstb_items: 5000, ..Default::default() };
    let mut a = crate::cooperative::automatic(&data, Some(activity));
    a.sstb_expenses.push(CoopSstbExpense {
        category: BusinessExpenseCategory::Feed,
        amount: 100000,
        source: "IRS stated related cooperative expense".into(),
    });
    data.allocations.push(a);
    add(&mut d, income(FederalIncome::Cooperative { data }));
    assert_eq!(crate::cooperative::graph_rule(&d), None);
    let mut activities = vec![QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 9000,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    }];
    let mut report = FederalResult::empty(0);
    crate::cooperative::qbi_filing(
        &d,
        &mut activities,
        &[(0, activity)].into_iter().collect(),
        50000,
        FederalStatus::Single,
        0,
        &mut report,
    );
    assert_eq!(activities[0].income, 9050); // 10000 - (1000 - 50) + (50 - 50).
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, activity).unwrap().taxable, 5000);
}

// Derived ScheduleD aggregation probe: its one aggregated row applies the
// lesser-of rule to totals, not to two already-limited member amounts.
#[test]
fn coop_aggregation_combines_qualified_sources_before_the_lesser_of_rule() {
    let activities = vec![
        QbiActivity {
            source_wages_cents: None,
            source_property_cents: None,
            source_income_cents: None,
            owner: None,
            income: 50000,
            wages: 50000,
            property: 0,
            active: true,
            service: Some(false),
            group: Some("Actual aggregation election".into())
        };
        2
    ];
    let mut report = FederalResult::empty(0);
    let r = crate::business_rules::qbi_filed(
        &activities,
        600000,
        0,
        FederalStatus::Single,
        QbiOther::default(),
        &QbiFiling {
            patron: true,
            patron_allocations: vec![[10000, 0], [0, 10000]],
            ..Default::default()
        },
        &mut report,
    );
    assert_eq!(r.lines.patron_allocations, vec![[10000, 10000]]);
    assert_eq!(r.lines.patron_reductions, vec![900]);
    assert_eq!(r.deduction, 19100);
}

// Derived line38 binding-cap and loss probes, independent arithmetic.
#[test]
fn coop_dpad_uses_the_remaining_taxable_income_and_losses_still_net_once() {
    let a = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 20000,
        wages: 0,
        property: 0,
        active: false,
        service: Some(false),
        group: None,
    };
    let mut report = FederalResult::empty(0);
    let r = crate::business_rules::qbi_filed(
        core::slice::from_ref(&a),
        12000,
        0,
        FederalStatus::Single,
        QbiOther::default(),
        &QbiFiling { patron: true, dpad: 15000, ..Default::default() },
        &mut report,
    );
    assert_eq!((r.lines.limited, r.lines.dpad, r.deduction), (2400, 9600, 12000));
    let loss = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        income: -5000,
        ..a.clone()
    };
    let r = crate::business_rules::qbi_filed(
        &[a, loss],
        100000,
        0,
        FederalStatus::Single,
        QbiOther::default(),
        &QbiFiling {
            patron: true,
            patron_allocations: vec![[10000, 10000], [0, 0]],
            ..Default::default()
        },
        &mut report,
    );
    assert_eq!(r.lines.net_income, 15000);
    assert_eq!(r.deduction, 2100); // .2*(20000-5000) - .09*10000.
}

#[test]
fn coop_unknown_service_source_is_phased_exactly_once() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let id = d.income[0].id;
    let mut raw =
        FederalCooperative { patronage: 100000, sstb_items: 100000, ..Default::default() };
    raw.allocations.push(crate::cooperative::automatic(&raw, Some(id)));
    add(&mut d, income(FederalIncome::Cooperative { data: raw }));
    let (floor, range) = crate::business_rules::qbi_phase(0, FederalStatus::Single);
    assert_eq!(floor, 0);
    // Actual current-year threshold owner supplies the phase, not a fixture
    // copy of the implementation's source-classification arithmetic.
    let taxable = crate::year_constants::QBI_THRESHOLD_OTHER_DOLLARS + range / 2;
    let mut activities = vec![QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 10000,
        wages: 100000,
        property: 0,
        active: false,
        service: None,
        group: None,
    }];
    let mut report = FederalResult::empty(0);
    crate::cooperative::qbi_filing(
        &d,
        &mut activities,
        &[(0, id)].into_iter().collect(),
        taxable,
        FederalStatus::Single,
        0,
        &mut report,
    );
    assert_eq!(activities[0].income, 11000);
    let r = crate::business_rules::qbi_filed(
        &activities,
        taxable,
        0,
        FederalStatus::Single,
        QbiOther::default(),
        &QbiFiling { patron: true, ..Default::default() },
        &mut report,
    );
    assert_eq!(r.lines.activities[0][0], 5500);
}

pub(in crate::tests) fn patr_page(amount: i64, personal: i64, activity: Option<i64>) -> Patr1099 {
    let raw = FederalCooperative { patronage: amount, ..Default::default() };
    let mut a = crate::cooperative::automatic(&raw, activity);
    a.personal.patronage = personal;
    a.personal_qualification = Some(CoopQualification::default());
    Patr1099 {
        patronage: amount,
        nonpatronage: 0,
        per_unit_retains: 0,
        redeemed: 0,
        withholding: 10000,
        more: PatrMore { allocations: vec![a.into()], ..Default::default() },
    }
}

#[test]
fn coop_public_schedule_c_preserves_sources_identity_restart_and_failed_save_purity() {
    use crate::tests::cross::interview::{
        accepted, business, document, open_topic, page_documents,
    };
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.push(document(0, IncomeDocument::Business(business(1000000))));
    accepted(&send(&mut s, page));
    let activity = s.draft.income[0].id;
    let before_preview = s.draft.clone();
    let source = IncomeDocument::Patr1099(patr_page(200000, 50000, Some(activity)));
    assert!(matches!(
        send(&mut s, coop_preview(source, 0)),
        Reply::Response(P::IncomePreview { preview: Some(150000), .. })
    ));
    assert_eq!(s.draft, before_preview);
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents
        .documents
        .push(document(0, IncomeDocument::Patr1099(patr_page(200000, 50000, Some(activity)))));
    let running = documents.documents.clone();
    accepted(&send(&mut s, coop_running(running)));
    assert_eq!(s.draft, before_preview);
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(form_dollars(&r, TaxForm::ScheduleC, "6"), 1500);
    assert_eq!(form_dollars(&r, TaxForm::ScheduleC, "31"), 11500);
    assert_eq!(r.payments, Some(10000));
    let accepted_draft = s.draft.clone();
    let mut s = restart(accepted_draft.clone());
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let original_rows = page_documents(&page).to_vec();
    let original = original_rows
        .iter()
        .find(|i| matches!(i.document, IncomeDocument::Patr1099(_)))
        .unwrap()
        .clone();
    let IncomeDocument::Patr1099(patr) = &original.document else { panic!() };
    let child = patr.more.allocations[0].id;
    assert!(child > original.id);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.reverse();
    if let IncomeDocument::Business(b) = &mut documents.documents[1].document {
        b.name = "Renamed actual activity".into();
    }
    accepted(&send(&mut s, page));
    let page = open_topic(&mut s, TaxSection::SelfEmployment);
    let new_rows = page_documents(&page);
    assert!(new_rows.iter().any(|i| i.id == original.id && i.document == original.document));
    let before = s.draft.clone();
    let mut invalid = page.clone();
    let P::SaveIncomes { documents, .. } = &mut invalid else { panic!() };
    let row = documents.documents.iter_mut().find(|i| i.id == original.id).unwrap();
    let IncomeDocument::Patr1099(p) = &mut row.document else { panic!() };
    p.more.allocations[0].id = activity; // A parent's accepted ID is not this child's.
    assert!(matches!(
        current_income_refusal(
            "income-source-332",
            &mut s,
            invalid,
            "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
            "[documents/documents/0/document/more/allocations/0]"
        ),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(s.draft, before);
    let mut removed = page;
    let P::SaveIncomes { documents, .. } = &mut removed else { panic!() };
    documents.documents.retain(|i| i.id == original.id);
    assert!(matches!(send(&mut s, removed), Reply::Response(P::Problem { .. })));
    assert_eq!(s.draft, before); // Stale recipient is never rebound.
    assert_eq!(accepted_draft.next_id, before.next_id);
}

#[test]
fn coop_public_sole_credit_recipient_defaults_without_inventing_an_activity() {
    use crate::tests::cross::interview::{accepted, business, document, open_topic};
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.push(document(0, IncomeDocument::Business(business(1000000))));
    accepted(&send(&mut s, page));
    let activity = s.draft.income[0].id;
    let mut patr = patr_page(100000, 0, None);
    patr.more.credit_facts.work_opportunity = 20000;
    patr.more.credit_facts.participation = Some(true);
    patr.more.credit_facts.notice =
        Some(CoopNoticeStatus::Received { source: "Actual issuer allocation notice".into() });
    patr.more.credit_facts.other.push(CoopOtherItemInput {
        kind: CoopOtherKind::SmallEmployerHealth,
        amount: 10000,
        activity: None,
        income: CoopCreditIncome::None,
        source: "Actual issuer health credit allocation".into(),
    });
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.push(document(0, IncomeDocument::Patr1099(patr)));
    accepted(&send(&mut s, page));
    let FederalIncome::Cooperative { data } = &s.draft.income[1].income else { panic!() };
    assert_eq!(data.credit_facts.activity, Some(activity));
    assert_eq!(data.credit_facts.other[0].activity, Some(activity));
    let mut report = FederalResult::empty(0);
    let credits = crate::cooperative::credit_entries(&s.draft, &mut report);
    assert_eq!(credits.len(), 2);
    assert_eq!(credits.iter().map(|c| c.amount).sum::<i64>(), 300);
}

#[test]
fn coop_credit_components_use_existing_collector_once_and_keep_required_income_separate() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let id = d.income[0].id;
    let mut data =
        FederalCooperative { patronage: 100000, withholding: 10000, ..Default::default() };
    data.credit_facts = CoopCreditFacts {
        payer_ein: "12-3456789".into(),
        investment: 90000,
        work_opportunity: 20000,
        investment_shares: vec![
            CoopInvestmentShare {
                kind: CoopInvestmentKind::Energy,
                amount: 50000,
                source: "Issuer energy allocation".into(),
            },
            CoopInvestmentShare {
                kind: CoopInvestmentKind::CleanElectricity,
                amount: 40000,
                source: "Issuer clean electricity allocation".into(),
            },
        ],
        other: vec![CoopOtherItem {
            kind: CoopOtherKind::CleanFuel,
            amount: 30000,
            activity: Some(id),
            income: CoopCreditIncome::None,
            source: "Issuer45Z allocation".into(),
        }],
        participation: Some(true),
        activity: Some(id),
        notice: Some(CoopNoticeStatus::Received { source: "Actual allocation notice".into() }),
    };
    add(&mut d, income(FederalIncome::Cooperative { data }));
    let mut r = FederalResult::empty(0);
    let credits = crate::cooperative::credit_entries(&d, &mut r);
    assert_eq!(credits.iter().map(|v| v.amount).sum::<i64>(), 1400);
    assert_eq!(
        credits
            .iter()
            .filter(|v| v.credit == BusinessCredit::CleanFuel)
            .map(|v| v.amount)
            .sum::<i64>(),
        300
    );
    assert!(credits.iter().all(|v| v.passive.is_none()));
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, id).unwrap().taxable, 100000);
    let FederalIncome::Cooperative { data } = &mut d.income[1].income else { panic!() };
    data.credit_facts.investment_shares.clear();
    data.credit_facts.participation = None;
    data.credit_facts.other[0].income = CoopCreditIncome::Unresolved;
    let mut r = FederalResult::empty(0);
    let credits = crate::cooperative::credit_entries(&d, &mut r);
    assert_eq!(credits.iter().map(|v| v.amount).sum::<i64>(), 500);
    assert!(credits.iter().all(|v| v.passive.is_some()));
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COOP_CREDIT_INCOME_NEEDED"));
    assert!(noted(&r, "WARN_COOP_CREDIT_SOURCE_NEEDED"));
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, id).unwrap().taxable, 100000);
}

pub(in crate::tests) fn form_dollars(r: &FederalResult, form: TaxForm, line: &str) -> i64 {
    let rows: Vec<_> = r
        .lines
        .iter()
        .filter(|r| r.form_line.as_ref().is_some_and(|f| f.form == form && f.line == line))
        .collect();
    assert_eq!(rows.len(), 1, "{form:?} {line}");
    rows[0].amount / 100
}

// Derived notice/source-boundary probes. Section1385(c) establishes basis
// and stated-value character; amounts above that scope remain named unknowns.
#[test]
fn coop_public_box5_requires_complete_history_without_reclassifying_other_boxes() {
    use crate::tests::cross::interview::{accepted, business, document, open_topic};
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.push(document(0, IncomeDocument::Business(business(1000000))));
    accepted(&send(&mut s, page));
    let mut patr = patr_page(0, 0, None);
    patr.redeemed = 100000;
    patr.more.allocations.clear();
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.push(document(0, IncomeDocument::Patr1099(patr)));
    accepted(&send(&mut s, page));
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COOP_NOTICE_HISTORY_NEEDED"));
    assert!(r.lines.is_empty());
    let notice = |amount, label: &str| CoopRedemption {
        kind: CoopNoticeKind::Patronage,
        distribution: CoopDistributionBox::Redeemed,
        amount,
        stated_value: amount,
        holding: CoopNoticeHolding::OriginalOrInherited,
        source: label.into(),
    };
    for complete in [false, true] {
        let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
        let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
        let patr = documents
            .documents
            .iter_mut()
            .find_map(|e| match &mut e.document {
                IncomeDocument::Patr1099(patr) => Some(patr),
                _ => None,
            })
            .unwrap();
        patr.more.allocations[0].redemptions =
            vec![notice(40000, "Actual original-patron notice A")];
        if complete {
            patr.more.allocations[0]
                .redemptions
                .push(notice(60000, "Actual original-patron notice B"));
        }
        accepted(&send(&mut s, page));
        let FederalIncome::Cooperative { data } = &s.draft.income[1].income else { panic!() };
        assert_eq!(data.redeemed, 100000);
        assert_eq!(data.withholding, 10000);
        let r = estimate(&s.draft);
        if complete {
            assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
            assert_eq!(form_dollars(&r, TaxForm::ScheduleC, "6"), 1000);
            assert_eq!(r.payments, Some(10000));
        } else {
            assert_eq!(r.state, ResultState::Unsupported);
            assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COOP_NOTICE_HISTORY_NEEDED"));
            assert!(r.lines.is_empty());
        }
    }
    // Raw boxes 1/2 are separate distributions, not evidence of notice
    // redemption. Their ordinary amounts need no fictional notice history.
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    let patr = documents
        .documents
        .iter_mut()
        .find_map(|e| match &mut e.document {
            IncomeDocument::Patr1099(patr) => Some(patr),
            _ => None,
        })
        .unwrap();
    patr.redeemed = 0;
    patr.patronage = 60000;
    patr.nonpatronage = 40000;
    let a = &mut patr.more.allocations[0];
    a.distributions.redeemed = 0;
    a.distributions.patronage = 60000;
    a.distributions.nonpatronage = 40000;
    a.redemptions.clear();
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(form_dollars(&r, TaxForm::ScheduleC, "6"), 1000);
}

#[test]
fn coop_notice_original_acquired_and_unknown_history_are_distinct() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let id = d.income[0].id;
    let mut data = FederalCooperative { redeemed: 100000, ..Default::default() };
    let mut a = crate::cooperative::automatic(&data, Some(id));
    a.redemptions.push(CoopRedemption {
        kind: CoopNoticeKind::Patronage,
        distribution: CoopDistributionBox::Redeemed,
        amount: 100000,
        stated_value: 100000,
        holding: CoopNoticeHolding::OriginalOrInherited,
        source: "Original patron notice, inherited directly from that patron".into(),
    });
    let mut wide = a.clone();
    wide.redemptions[0].amount = i64::MAX;
    wide.redemptions.push(wide.redemptions[0].clone());
    assert_eq!(
        wide.issue(),
        Some(
            crate::validation::ValidationIssue::new("COOP_ALLOCATION_INVALID")
                .at_field("redemptions")
        )
    );
    data.allocations.push(a);
    add(&mut d, income(FederalIncome::Cooperative { data }));
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, id).unwrap().taxable, 100000);
    supported(&d);
    let FederalIncome::Cooperative { data } = &mut d.income[1].income else { panic!() };
    data.allocations[0].redemptions[0].holding = CoopNoticeHolding::Acquired {
        basis: 30000,
        source: "Actual purchased notice basis".into(),
    };
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, id).unwrap().taxable, 70000);
    supported(&d);
    let FederalIncome::Cooperative { data } = &mut d.income[1].income else { panic!() };
    data.allocations[0].redemptions[0].holding = CoopNoticeHolding::Unresolved;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COOP_NOTICE_HISTORY_NEEDED"));
}

#[test]
fn coop_property_dependency_never_turns_an_excluded_refund_into_taxable_income() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let id = d.income[0].id;
    let mut data = FederalCooperative { patronage: 100000, ..Default::default() };
    let mut a = crate::cooperative::automatic(&data, Some(id));
    a.properties.push(CoopPropertyRefund {
        distribution: CoopDistributionBox::Patronage,
        property: None,
        source: "Actual unsold equipment purchase invoice; property owner not integrated".into(),
        amount: 30000,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: CoopPropertyTreatment::Purchase,
    });
    data.allocations.push(a);
    add(&mut d, income(FederalIncome::Cooperative { data }));
    assert_eq!(crate::cooperative::graph_rule(&d), None);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COOP_PROPERTY_HISTORY_NEEDED"));
    assert!(r.lines.is_empty());
}

fn coop_preview(document: IncomeDocument, id: i64) -> P {
    let mut request = survey_preview_document(document);
    let P::PreviewDocument { id: request_id, .. } = &mut request else { unreachable!() };
    *request_id = id;
    request
}

fn coop_running(documents: Vec<IncomeDocumentItem>) -> P {
    let mut request = crate::tests::support::client::income_pages::catalog_estimate_documents(
        TaxSection::SelfEmployment,
        (IncomeDocuments {
            qbi_aggregations: Vec::new(),
            receipt_context: crate::BusinessReceiptContext::default(),
            documents,
        })
        .into(),
    );
    let P::EstimateDocuments {
        allowed_documents,
        k1_credit_lines,
        research_credit_lines,
        renewable_credit_lines,
        transferable_credit_lines,
        foreign_countries,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    *allowed_documents = vec![libertas_macros::variant_index!(IncomeDocument::Patr1099) as i32];
    *k1_credit_lines = Vec::new();
    *research_credit_lines = Vec::new();
    *renewable_credit_lines = Vec::new();
    *transferable_credit_lines = Vec::new();
    *foreign_countries = Vec::new();
    request
}

#[test]
fn coop_public_recipient_rejects_cross_owner_links_without_consuming_ids() {
    use crate::tests::cross::interview::{accepted, business, document, open_topic, rejected_with};
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Joint));
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    documents.documents.push(document(0, IncomeDocument::Business(business(1000000))));
    accepted(&send(&mut s, page));
    let id = s.draft.income[0].id;
    let before = s.draft.clone();
    let mut page = open_topic(&mut s, TaxSection::SelfEmployment);
    let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
    let mut item = document(0, IncomeDocument::Patr1099(patr_page(100000, 0, Some(id))));
    item.owner = Owner::Spouse;
    documents.documents.push(item);
    rejected_with(send(&mut s, page), "COOP_RECIPIENT_LINK_INVALID");
    assert_eq!(s.draft, before);
}

// Derived community allocation probe, not a published IRS example.
#[test]
fn coop_community_qbi_sources_and_dpad_use_the_same_recipient_share() {
    let mut d = empty_return(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = community_arizona();
    let mut farm = raised_farm(150000, 200000);
    let FederalIncome::Business { data } = &mut farm else { panic!() };
    data.expenses = vec![
        ordinary_business_expense(BusinessExpenseCategory::Feed, 15000000),
        ordinary_business_expense(BusinessExpenseCategory::Wages, 5000000),
    ];
    data.payroll = BusinessPayroll::Yes {
        statements: vec![PayrollW2 {
            label: "Actual wages".into(),
            wages: 5000000,
            medicare: 5000000,
        }],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    let mut entry = income(farm);
    entry.community = CommunityShare::Community;
    add(&mut d, entry);
    let id = d.income[0].id;
    let mut data = FederalCooperative {
        patronage: 10000000,
        qualified_payments: 10000000,
        qualified_items: 10000000,
        section_199a_g: 100000,
        ..Default::default()
    };
    let mut a = crate::cooperative::automatic(&data, Some(id));
    a.notice = Some(CoopNoticeStatus::Received { source: "Actual DPAD notice".into() });
    a.patron_allocation = Some(CoopPatronAllocation::Books {
        expenses_excluding_wages: 6500000,
        wages: 2500000,
        other_deductions: 0,
        source: "Actual source allocation before community division".into(),
    });
    data.allocations.push(a);
    let mut entry = income(FederalIncome::Cooperative { data });
    entry.community = CommunityShare::Community;
    add(&mut d, entry);
    let mut activities = vec![QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 25000,
        wages: 25000,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    }];
    let mut r = FederalResult::empty(0);
    let (p, dpad, _, _) = crate::cooperative::qbi_filing(
        &d,
        &mut activities,
        &[(0, id)].into_iter().collect(),
        75000,
        FederalStatus::Separate,
        0,
        &mut r,
    );
    assert_eq!(p, vec![[5000, 12500]]);
    assert_eq!(dpad, 500);
}

#[test]
fn coop_purchase_rebate_preserves_farm_conservation_production_gross() {
    let mut d = conservation_return();
    let activity = d.income[1].id;
    let mut data = FederalCooperative { patronage: 100000, ..Default::default() };
    let mut a = crate::cooperative::automatic(&data, Some(activity));
    a.income_use = CoopIncomeUse::DeductiblePurchases;
    data.allocations.push(a);
    add(&mut d, income(FederalIncome::Cooperative { data }));
    let original = d.clone();
    let r = supported(&d);
    assert!(!noted(&r, "WARN_FARM_CONSERVATION_GROSS_SOURCE"));
    assert_eq!(value(&r, &alloc::format!("farm_{activity}_conservation_deduction")), 30000);
    assert_eq!(value(&r, &alloc::format!("farm_{activity}_conservation_carryforward")), 15000);
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, activity).unwrap().taxable, 100000);
    assert_eq!(d, original); // Accepted costs and carry remain recorded.
}

#[test]
fn coop_production_gross_increases_one_farm_limit_and_missing_character_omits_only_benefit() {
    let mut d = conservation_return();
    let activity = d.income[1].id;
    let mut data = FederalCooperative { patronage: 100000, ..Default::default() };
    let mut a = crate::cooperative::automatic(&data, Some(activity));
    a.income_use = CoopIncomeUse::ProductionSales;
    data.allocations.push(a);
    add(&mut d, income(FederalIncome::Cooperative { data }));
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("farm_{activity}_conservation_deduction")), 30250);
    assert_eq!(value(&r, &alloc::format!("farm_{activity}_conservation_carryforward")), 14750);
    assert_eq!(crate::cooperative::conservation_gross(&d, Owner::Taxpayer, activity), Some(100000));
    assert_eq!(crate::cooperative::conservation_gross(&d, Owner::Spouse, activity), Some(0));
    let FederalIncome::Cooperative { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.allocations[0].income_use = CoopIncomeUse::Unresolved;
    let before = d.clone();
    let r = supported(&d);
    assert!(noted(&r, "WARN_FARM_CONSERVATION_GROSS_SOURCE"));
    assert_eq!(value(&r, &alloc::format!("farm_{activity}_conservation_deduction")), 0);
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, activity).unwrap().taxable, 100000);
    assert_eq!(d, before);
}

// Derived current-year land integration; not Brown's depreciable equipment.
#[test]
fn coop_land_refunds_use_one_actual_regular_and_amt_basis_and_sale_owner() {
    let mut d = conservation_return();
    let activity = d.income[1].id;
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    let parcel = data.land_basis[0].id;
    data.land_basis[0].regular = 10000000;
    data.land_basis[0].amt = 9000000;
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let sale_before = farmland_resolved_sale(&d, 0);
    for (amount, treatment) in [
        (30000, CoopPropertyTreatment::Purchase),
        (20000, CoopPropertyTreatment::Purchase),
        (70000, CoopPropertyTreatment::CurrentSale),
    ] {
        let mut data = FederalCooperative { patronage: amount, ..Default::default() };
        let mut a = crate::cooperative::automatic(&data, Some(activity));
        a.properties.push(CoopPropertyRefund {
            distribution: CoopDistributionBox::Patronage,
            property: Some(parcel),
            source: "Actual parcel purchase/sale issuer invoice".into(),
            amount,
            qualified_items: 0,
            sstb_items: 0,
            qualified_payments: 0,
            treatment,
        });
        data.allocations.push(a);
        add(&mut d, income(FederalIncome::Cooperative { data }));
    }
    let before = d.clone();
    assert_eq!(crate::cooperative::graph_rule(&d), None);
    let r = supported(&d);
    assert!(!r.issues.iter().any(|i| i.display.key == "ISSUE_COOP_PROPERTY_HISTORY_NEEDED"));
    let resolved = farmland_resolved_sale(&d, 0);
    assert_eq!(resolved.basis, sale_before.basis - 50000);
    assert_eq!(resolved.proceeds, sale_before.proceeds + 70000);
    let FarmLandAmt::Different { basis, .. } = resolved.amt else { unreachable!() };
    assert_eq!(basis, 8950000);
    assert_eq!(crate::federal_rules::farm_cooperative(&d, 0, activity).unwrap().taxable, 0);
    assert_eq!(value(&r, &alloc::format!("farm_{activity}_landbasis_{parcel}_regular")), 99500);
    assert_eq!(d, before);
    d.income.reverse();
    assert_eq!(crate::cooperative::land_adjustments(&d, activity, parcel), (50000, 70000));
    assert_eq!(crate::cooperative::graph_rule(&d), None);
    let mut wrong = before.clone();
    wrong.income.last_mut().unwrap().owner = Owner::Spouse;
    assert_eq!(crate::cooperative::graph_rule(&wrong), Some("COOP_RECIPIENT_LINK_INVALID"));
    let FederalIncome::Cooperative { data } = &mut wrong.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.allocations[0].activity = None;
    assert_eq!(crate::cooperative::graph_rule(&wrong), Some("COOP_PROPERTY_LINK_UNAVAILABLE"));
}

#[test]
fn coop_land_basis_exhaustion_preserves_required_tax_boundary() {
    let mut d = conservation_return();
    let activity = d.income[1].id;
    let parcel = farmland_basis_mut(&mut d).id;
    farmland_basis_mut(&mut d).regular = 20000;
    farmland_basis_mut(&mut d).amt = 10000;
    let mut data = FederalCooperative { patronage: 30000, ..Default::default() };
    let mut a = crate::cooperative::automatic(&data, Some(activity));
    a.properties.push(CoopPropertyRefund {
        distribution: CoopDistributionBox::Patronage,
        property: Some(parcel),
        source: "Actual purchase refund exceeding regular/AMT opening basis".into(),
        amount: 30000,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: CoopPropertyTreatment::Purchase,
    });
    data.allocations.push(a);
    add(&mut d, income(FederalIncome::Cooperative { data }));
    assert_eq!(crate::cooperative::graph_rule(&d), None);
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COOP_PROPERTY_HISTORY_NEEDED"));
    assert!(r.lines.is_empty());
}

#[test]
fn coop_public_land_catalog_save_reopen_and_forged_link_preserve_actual_ids() {
    use crate::tests::cross::interview::{accepted, document, open_topic, page_documents};
    let mut d = conservation_return();
    let activity = d.income[1].id;
    let parcel = farmland_basis_mut(&mut d).id;
    farmland_basis_mut(&mut d).regular = 10000000;
    farmland_basis_mut(&mut d).amt = 9000000;
    let mut s = Interview::new(TEST_USER, d);
    let mut page = open_topic(&mut s, TaxSection::Farming);
    let P::SaveIncomes { cooperative_properties, documents, .. } = &mut page else {
        unreachable!()
    };
    assert!(cooperative_properties.iter().any(|p| p.id == parcel));
    let mut patr = patr_page(30000, 0, Some(activity));
    patr.more.allocations[0].properties.push(CoopPropertyRefundInput {
        distribution: CoopDistributionBox::Patronage,
        property: Some(parcel),
        source: "Actual retained parcel purchase refund".into(),
        amount: 30000,
        qualified_items: 0,
        sstb_items: 0,
        qualified_payments: 0,
        treatment: CoopPropertyTreatment::Purchase,
    });
    documents.documents.push(document(0, IncomeDocument::Patr1099(patr)));
    accepted(&send(&mut s, page));
    let saved = s.draft.clone();
    let mut s = restart(saved.clone());
    let mut page = open_topic(&mut s, TaxSection::Farming);
    let row = page_documents(&page)
        .iter()
        .find(|p| matches!(p.document, IncomeDocument::Patr1099(_)))
        .unwrap();
    let IncomeDocument::Patr1099(patr) = &row.document else { unreachable!() };
    assert_eq!(patr.more.allocations[0].properties[0].property, Some(parcel));
    let parent = row.id;
    let allocation = patr.more.allocations[0].id;
    assert!(allocation > parent);
    let before = s.draft.clone();
    let P::SaveIncomes { cooperative_properties, documents, .. } = &mut page else {
        unreachable!()
    };
    cooperative_properties.push(RecordChoice { id: 999999, label: vec![] });
    let IncomeDocument::Patr1099(patr) =
        &mut documents.documents.iter_mut().find(|p| p.id == parent).unwrap().document
    else {
        unreachable!()
    };
    patr.more.allocations[0].properties[0].property = Some(999999);
    assert!(matches!(send(&mut s, page), Reply::Response(P::Problem { .. })));
    assert_eq!(s.draft, before);
    assert_eq!(s.draft.next_id, saved.next_id);
    assert_eq!(crate::cooperative::land_adjustments(&s.draft, activity, parcel), (30000, 0));
}
