use super::*;
use crate::tests::cross::interview::{
    accepted, document, open_topic, page_documents, rejected_with, save_topic,
};

fn save_investments(s: &mut Interview, documents: Vec<IncomeDocument>) -> Reply {
    let request = save_topic(
        &s.draft,
        TaxSection::InterestDividends,
        documents.into_iter().map(|d| document(0, d)).collect(),
    );
    send(s, request)
}
fn investment_session() -> Interview {
    Interview::new(TEST_USER, complete(FederalStatus::Single))
}
fn printed_interest() -> Int1099 {
    Int1099 {
        more: IntMore {
            market_discount: 25_000,
            investment_expenses: 9_000,
            foreign_country: "Canada".into(),
            ..Default::default()
        },
        ..native_int1099("Broker".into(), 100_000)
    }
}
fn printed_dividend() -> Div1099 {
    Div1099 {
        more: DivMore {
            capital_distributions: 50_000,
            section_897_ordinary: 40_000,
            section_897_capital: 20_000,
            foreign_country: "United Kingdom".into(),
            ..Default::default()
        },
        ..native_div1099("Fund".into(), 100_000, 0)
    }
}
fn printed_oid() -> Oid1099 {
    Oid1099 {
        more: OidMore { investment_expenses: 15_000, ..Default::default() },
        ..native_oid1099("OID Broker".into(), 80_000, 20_000)
    }
}

// Independent retained-source arithmetic: INT1500-450=1050, OID1000-200=800,
// DIV ordinary1000-300=700 and capital500-100=400. Part III keeps the actual
// two owners, while the original statement boxes and source IDs survive Save.
#[test]
fn injured_spouse_merged_1099_components_keep_owner_through_save_and_restart() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![
        wage(1, Owner::Taxpayer, 6_000_000, 500_000),
        wage(2, Owner::Spouse, 2_000_000, 50_000),
    ];
    d.next_id = 3;
    d.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
    d.injured_spouse = Some(InjuredSpouseAllocation::defaults());
    let mut s = Interview::new(TEST_USER, d);
    let mut int = printed_interest();
    int.taxable = 150_000;
    int.more = IntMore { withholding: 2_000, ..Default::default() };
    int.nominee = Some(InterestNominee { taxable: 45_000, ..Default::default() });
    let mut oid = printed_oid();
    oid.oid = 100_000;
    oid.other_periodic_interest = 0;
    oid.more = OidMore::default();
    oid.nominee = Some(OidNominee { oid: 20_000, ..Default::default() });
    let mut div = printed_dividend();
    div.more = DivMore { capital_distributions: 50_000, withholding: 3_000, ..Default::default() };
    div.nominee = Some(DividendNominee { ordinary: 30_000, capital: 10_000, ..Default::default() });
    let mut rows = vec![
        document(0, IncomeDocument::Int1099(int.clone())),
        document(0, IncomeDocument::Oid1099(oid.clone())),
        document(0, IncomeDocument::Div1099(div.clone())),
    ];
    rows[2].owner = Owner::Spouse;
    let request = save_topic(&s.draft, TaxSection::InterestDividends, rows);
    accepted(&send(&mut s, request));
    let original = s.draft.clone();
    let result = supported(&original);
    for (line, amounts) in [("13b", [2_950, 1_850, 1_100]), ("19", [5_550, 5_020, 530])] {
        for (column, amount) in ["a", "b", "c"].into_iter().zip(amounts) {
            assert_eq!(value(&result, &alloc::format!("form_8379_line_{line}_{column}")), amount);
        }
    }
    assert!(result.lines.iter().any(|l| l.id == "form_8379_injured_refund_estimate"));
    let mut ordinary = original.clone();
    ordinary.injured_spouse = None;
    let ordinary_result = supported(&ordinary);
    assert_eq!(
        (result.tax, result.refund, result.balance),
        (ordinary_result.tax, ordinary_result.refund, ordinary_result.balance)
    );
    let identities: Vec<_> = original.income.iter().map(|e| (e.id, e.owner)).collect();
    let mut s = restart(original.clone());
    assert_eq!(estimate(&s.draft), result);
    let reopened = page_documents(&open_topic(&mut s, TaxSection::InterestDividends)).to_vec();
    assert_eq!(
        reopened.iter().map(|r| r.document.clone()).collect::<Vec<_>>(),
        vec![
            IncomeDocument::Int1099(int),
            IncomeDocument::Oid1099(oid),
            IncomeDocument::Div1099(div)
        ]
    );
    let request = save_topic(&s.draft, TaxSection::InterestDividends, reopened);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income.iter().map(|e| (e.id, e.owner)).collect::<Vec<_>>(), identities);
    let mut reversed = s.draft.clone();
    reversed.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
    let reverse = supported(&reversed);
    assert_eq!(value(&reverse, "form_8379_line_13b_b"), 1_100);
    assert_eq!(value(&reverse, "form_8379_line_13b_c"), 1_850);
    assert_eq!(
        (reverse.tax, reverse.refund, reverse.balance),
        (result.tax, result.refund, result.balance)
    );
}

// Derived box-character fixtures, independent of the implementation: 1099-INT
// instructions Rev.January2024 pp.3–4 and 1099-DIV instructions pp.3–4;
// https://www.irs.gov/pub/irs-pdf/i1099int.pdf and i1099div.pdf. Expenses
// already included in the form are informational; 897 subsets are not added.
#[test]
fn investment_printed_boxes_flow_once_through_public_page_and_restart() {
    let documents = vec![
        IncomeDocument::Int1099(printed_interest()),
        IncomeDocument::Div1099(printed_dividend()),
        IncomeDocument::Oid1099(printed_oid()),
    ];
    let mut s = investment_session();
    accepted(&save_investments(&mut s, documents.clone()));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "2b"), 225_000); // 1,000 + 250 + 800 + 200
    assert_eq!(return_line(&r, "3b"), 100_000);
    assert_eq!(value(&r, "capital_gain"), 500);
    assert_eq!(value(&r, "agi"), 63_750);
    let mut s = restart(s.draft);
    let reopened = page_documents(&open_topic(&mut s, TaxSection::InterestDividends))
        .iter()
        .map(|i| i.document.clone())
        .collect::<Vec<_>>();
    assert_eq!(reopened, documents);
}

#[test]
fn investment_section_897_subsets_validate_on_page_and_restored_records() {
    for (ordinary, capital, key) in [
        (100_001, 20_000, "DIV_897_ORDINARY_ABOVE_BOX"),
        (40_000, 50_001, "DIV_897_CAPITAL_ABOVE_BOX"),
    ] {
        let mut s = investment_session();
        let mut v = printed_dividend();
        v.more.section_897_ordinary = ordinary;
        v.more.section_897_capital = capital;
        let before = s.draft.clone();
        rejected_with(save_investments(&mut s, vec![IncomeDocument::Div1099(v)]), key);
        assert_eq!(s.draft, before);
        accepted(&save_investments(&mut s, vec![IncomeDocument::Div1099(printed_dividend())]));
        let FederalIncome::Dividend { data } = &mut s.draft.income.last_mut().unwrap().income
        else {
            panic!("dividend")
        };
        data.section_897_ordinary = ordinary;
        data.section_897_capital = capital;
        let r = estimate(&s.draft);
        assert_eq!(r.state, ResultState::Incomplete);
        assert_eq!(
            crate::federal_rules::validate_income_amount(&s.draft.income.last().unwrap().income)
                .unwrap_err()
                .0,
            key
        );
    }
}

fn retained_components(
    d: &FederalDraft,
    id: i64,
) -> crate::investment_statements::InvestmentComponents {
    let mut notes = Vec::new();
    let copy = crate::unknowns::supported(d, &mut notes);
    crate::investment_statements::components(
        copy.as_ref().unwrap_or(d).income.iter().find(|e| e.id == id).expect("accepted source"),
    )
    .expect("investment components")
}

// Pub. 550 (2025), published March 5, 2026, printed pp. 25–26 Example 1:
// https://www.irs.gov/pub/irs-pdf/p550.pdf. Move redemption from March 2025
// to March 2026, preserving the exact IRS amounts: 413.60 − 406.40 = 7.20.
#[test]
fn investment_prior_inclusion_matches_pub550_savings_bond_example() {
    let mut v = printed_interest();
    v.taxable = 0;
    v.more = IntMore { us_obligations: 41_360, ..Default::default() };
    v.adjustments =
        Some(crate::InterestAdjustments { previously_reported: 40_640, ..Default::default() });
    let mut s = investment_session();
    accepted(&save_investments(&mut s, vec![IncomeDocument::Int1099(v.clone())]));
    let entry = s.draft.income.last().unwrap();
    let id = entry.id;
    assert_eq!(retained_components(&s.draft, id).interest, 720);
    assert_eq!(return_line(&supported(&s.draft), "2b"), 700); // Form1040 whole dollars
    assert_eq!(s.draft.income.last().unwrap().id, id);
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::InterestDividends))[0].document,
        IncomeDocument::Int1099(v)
    );
    assert_eq!(retained_components(&restart(s.draft).draft, id).interest, 720);
}

// Pub. 550 (2025), printed p. 26, Reporting Interest Income Example 2:
// https://www.irs.gov/pub/irs-pdf/p550.pdf. The source supplies $500 bond
// interest, $200 included on the uncle's final return and $300 savings-account
// interest: $500 + $300 − $200 = $600 on Schedule B line 2. The earlier
// Decedents Example 2 on p. 13 separately illustrates the bond's $300 part.
#[test]
fn investment_inherited_bond_prior_owner_inclusion_is_removed_once() {
    let mut inherited = printed_interest();
    inherited.taxable = 0;
    inherited.more = IntMore { us_obligations: 50_000, ..Default::default() };
    inherited.adjustments =
        Some(crate::InterestAdjustments { previously_reported: 20_000, ..Default::default() });
    let bank = Int1099 {
        taxable: 30_000,
        more: Default::default(),
        adjustments: None,
        ..printed_interest()
    };
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![IncomeDocument::Int1099(inherited), IncomeDocument::Int1099(bank)],
    ));
    assert_eq!(retained_components(&s.draft, s.draft.income[1].id).interest, 30_000);
    assert_eq!(return_line(&supported(&s.draft), "2b"), 60_000);
}

// Pub. 550 (2025), printed p. 27, Nominee distributions example. Moving
// 2025 to 2026 shifts the tax year only: $1,500 interest, sibling's documented
// 30% = $450, current filer's $1,050. The public page asks the actual $450 part.
#[test]
fn investment_nominee_matches_pub550_joint_account_example() {
    let mut v = printed_interest();
    v.taxable = 150_000;
    v.more = IntMore { withholding: 2_000, ..Default::default() };
    v.nominee = Some(crate::InterestNominee { taxable: 45_000, ..Default::default() });
    let mut s = investment_session();
    accepted(&save_investments(&mut s, vec![IncomeDocument::Int1099(v.clone())]));
    let id = s.draft.income.last().unwrap().id;
    let parts = retained_components(&s.draft, id);
    assert_eq!(parts.interest, 105_000);
    assert_eq!(parts.withholding, 2_000); // No proportional withholding allocation
    assert_eq!(return_line(&supported(&s.draft), "2b"), 105_000);
    assert!(noted(&supported(&s.draft), "NOTE_INVESTMENT_NOMINEE"));
    let mut s = restart(s.draft);
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::InterestDividends))[0].document,
        IncomeDocument::Int1099(v)
    );
    assert_eq!(retained_components(&s.draft, id), parts);
}

// Derived component fixture under Pub.550 nominee and Pub.559 final-return
// rules. Separately documented amounts, never a single income percentage.
#[test]
fn investment_dividend_components_coexist_with_disjoint_final_return_split() {
    let mut v = printed_dividend();
    v.qualified = 40_000;
    v.more = DivMore {
        capital_distributions: 50_000,
        unrecaptured_distributions: 10_000,
        collectible_distributions: 5_000,
        section199a: 20_000,
        section_897_ordinary: 10_000,
        section_897_capital: 5_000,
        exempt_dividends: 30_000,
        private_activity_dividends: 8_000,
        foreign_tax: 5_000,
        withholding: 2_000,
        ..Default::default()
    };
    v.answers.reit_eligible = Some(true);
    v.nominee = Some(crate::DividendNominee {
        ordinary: 30_000,
        qualified: 5_000,
        capital: 10_000,
        unrecaptured: 4_000,
        collectible: 2_000,
        section_199a: 10_000,
        section_897_ordinary: 2_000,
        section_897_capital: 1_000,
        exempt: 10_000,
        private_activity: 5_000,
        foreign_tax: 1_500,
        ..Default::default()
    });
    v.answers.after_death = Some(crate::DividendsAfterDeath {
        ordinary: 10_000,
        qualified: 3_000,
        capital_distributions: 5_000,
        unrecaptured_distributions: 1_000,
        collectible_distributions: 500,
        section199a: 2_000,
        section_897_ordinary: 1_000,
        section_897_capital: 500,
        exempt_dividends: 5_000,
        private_activity_dividends: 1_000,
        foreign_tax: 500,
        ..Default::default()
    });
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(crate::FinalReturn { date: 20260630, ordinary: true, bonds: None, estate_medical: 0 });
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save_investments(&mut s, vec![IncomeDocument::Div1099(v.clone())]));
    let id = s.draft.income.last().unwrap().id;
    let p = retained_components(&s.draft, id);
    assert_eq!(
        (p.ordinary_dividends, p.qualified_dividends, p.section199a),
        (60_000, 32_000, 8_000)
    );
    assert_eq!(
        (p.capital_distributions, p.unrecaptured_distributions, p.collectible_distributions),
        (35_000, 5_000, 2_500)
    );
    assert_eq!(
        (p.exempt_interest, p.private_activity, p.foreign_tax, p.withholding),
        (15_000, 2_000, 3_000, 2_000)
    );
    assert_eq!(value(&supported(&s.draft), "agi"), 60_950);
    let mut s = restart(s.draft);
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::InterestDividends))[0].document,
        IncomeDocument::Div1099(v)
    );
    assert_eq!(retained_components(&s.draft, id), p);
}

#[test]
fn investment_oid_nominee_preserves_negative_treasury_character() {
    let mut v = printed_oid();
    v.oid = 100_000;
    v.other_periodic_interest = 0;
    v.more = OidMore {
        treasury_oid: -40_000,
        market_discount: 10_000,
        bond_premium: 10_000,
        exempt: 40_000,
        tax_exempt_bond_premium: 4_000,
        private_activity: 5_000,
        withholding: 2_000,
        ..Default::default()
    };
    v.nominee = Some(crate::OidNominee {
        oid: 20_000,
        treasury_oid: -10_000,
        market_discount: 2_500,
        bond_premium: 3_000,
        exempt: 10_000,
        tax_exempt_bond_premium: 1_000,
        private_activity: 1_000,
        ..Default::default()
    });
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![IncomeDocument::Oid1099(v.clone()), IncomeDocument::TaxableBondPremium(true)],
    ));
    let p = retained_components(&s.draft, s.draft.income.last().unwrap().id);
    assert_eq!(
        (p.interest, p.exempt_interest, p.private_activity, p.withholding),
        (50_500, 27_000, 4_000, 2_000)
    );
    let mut s = restart(s.draft);
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::InterestDividends))[0].document,
        IncomeDocument::Oid1099(v)
    );
}

#[test]
fn investment_already_net_report_is_not_reduced_twice() {
    let mut v = printed_oid();
    v.more.bond_premium = 10_000;
    v.more.acquisition_premium = 20_000;
    v.adjustments = Some(crate::InterestAdjustments {
        reporting: crate::DebtReporting::Net,
        ..Default::default()
    });
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![IncomeDocument::Oid1099(v), IncomeDocument::TaxableBondPremium(true)],
    ));
    assert_eq!(return_line(&supported(&s.draft), "2b"), 100_000);
}
pub(in crate::tests) fn annual_interest(records: Vec<AnnualDebtRecord>) -> Int1099 {
    let mut value = printed_interest();
    value.taxable = records.iter().map(|r| r.amounts.taxable).sum();
    value.more = IntMore {
        us_obligations: records.iter().map(|r| r.amounts.us_obligations).sum(),
        bond_premium: records.iter().map(|r| r.amounts.bond_premium).sum(),
        treasury_bond_premium: records.iter().map(|r| r.amounts.treasury_bond_premium).sum(),
        exempt: records.iter().map(|r| r.amounts.exempt).sum(),
        private_activity: records.iter().map(|r| r.amounts.private_activity).sum(),
        tax_exempt_bond_premium: records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum(),
        foreign_tax: records.iter().map(|r| r.amounts.foreign_tax).sum(),
        ..Default::default()
    };
    value.adjustments = Some(InterestAdjustments {
        instrument_records: Some(AnnualDebtRecords { records }),
        ..Default::default()
    });
    value
}
pub(in crate::tests) fn annual_row(name: &str, amount: i64, premium: i64) -> AnnualDebtRecord {
    AnnualDebtRecord {
        instrument: name.into(),
        amounts: DebtComponents { taxable: amount, bond_premium: premium, ..Default::default() },
        premium_history: Some(PremiumHistory::default()),
        ..Default::default()
    }
}
fn annual_page(s: &mut Interview) -> (i64, Vec<AnnualDebtRecord>) {
    let page = open_topic(s, TaxSection::InterestDividends);
    let values = page_documents(&page);
    values
        .iter()
        .find_map(|i| match &i.document {
            IncomeDocument::Int1099(v) => {
                Some((i.id, v.adjustments.as_ref()?.instrument_records.as_ref()?.records.clone()))
            }
            _ => None,
        })
        .unwrap()
}

// Studio's ordinary editor added this member with only visible answers:
// $1,000 taxable interest, no adjustments, and explicit absent optional facts.
// Its hidden identity comes from the schema Default, never a supplied answer.
#[test]
fn investment_client_added_annual_row_saves_and_reopens_with_assigned_identity() {
    #[derive(serde::Deserialize)]
    struct ClientPayload {
        bytes: Vec<u8>,
    }
    let payload: ClientPayload =
        serde_json::from_str(include_str!("../../../tests/fixtures/unit14-client-annual-row.json"))
            .unwrap();
    let records = AnnualDebtRecords::from_avro(&payload.bytes).unwrap();
    assert_eq!(records.records.len(), 1);
    let row = &records.records[0];
    assert_eq!(row.id, 0);
    assert_eq!(row.instrument, "Synthetic client annual bond");
    assert_eq!(row.amounts.taxable, 100_000);
    assert!(row.original_issue.is_none());
    assert!(row.nominee.is_none());
    assert!(row.after_death.is_none());
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![IncomeDocument::Int1099(annual_interest(records.records))],
    ));
    let result = supported(&s.draft);
    assert_eq!(return_line(&result, "2b"), 100_000);
    assert_eq!(value(&result, "agi"), 61_000);
    let (parent, accepted_rows) = annual_page(&mut s);
    assert!(parent > 0);
    assert!(accepted_rows[0].id > 0);
    let mut reopened = restart(s.draft.clone());
    assert_eq!(supported(&reopened.draft), result);
    assert_eq!(annual_page(&mut reopened), (parent, accepted_rows));
}

// IRS Pub. 550 (2025), published March 5, 2026, printed p. 49:
// https://www.irs.gov/pub/irs-pdf/p550.pdf. The printed first accrual period
// ends in 2025; shift the whole bond timeline one year for this 2026 corpus.
// $110,000 × 8.07439% = $8,881.83, and $10,000 − $8,881.83 = $1,118.17.
// Accept that independently documented annual amortization, without inventing
// a second yield calculator. No earlier net inclusions/carry is an explicit completion.
#[test]
fn investment_annual_premium_matches_pub550_gross_and_net() {
    for (reporting, printed) in [(DebtReporting::Gross, 1_000_000), (DebtReporting::Net, 888_183)] {
        let mut row = annual_row("Pub. 550 annual bond", printed, 111_817);
        row.reporting = reporting;
        let mut s = investment_session();
        accepted(&save_investments(
            &mut s,
            vec![
                IncomeDocument::Int1099(annual_interest(vec![row])),
                IncomeDocument::TaxableBondPremium(true),
            ],
        ));
        let (parent, rows) = annual_page(&mut s);
        assert!(rows[0].id > 0 && rows[0].id != parent);
        assert_eq!(retained_components(&s.draft, parent).interest, 888_183);
        assert_eq!(return_line(&supported(&s.draft), "2b"), 888_200);
        let mut again = restart(s.draft);
        assert_eq!(annual_page(&mut again), (parent, rows));
    }
}

// Derived §171 interaction: bond A's $300 premium offsets its $100 interest,
// leaving a $200 excess within $500 − $50 prior net inclusions. Bond B's
// $1,000 stays income. Payer-wide netting would incorrectly erase $200 of B.
#[test]
fn investment_annual_premium_is_per_instrument_and_feeds_schedule_a_once() {
    let mut a = annual_row("Bond A", 10_000, 30_000);
    a.premium_history = Some(PremiumHistory {
        prior_net_interest_inclusions: 50_000,
        prior_premium_deductions: 5_000,
        premium_carry: 0,
    });
    let b = annual_row("Bond B", 100_000, 0);
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![
            IncomeDocument::Int1099(annual_interest(vec![a, b])),
            IncomeDocument::TaxableBondPremium(true),
        ],
    ));
    let (parent, rows) = annual_page(&mut s);
    assert_eq!(retained_components(&s.draft, parent).interest, 100_000);
    let mut copy =
        crate::unknowns::supported(&s.draft, &mut Vec::new()).unwrap_or_else(|| s.draft.clone());
    assert_eq!(crate::investment_statements::computed_bond_premium(&copy), 20_000);
    copy.deductions.as_mut().unwrap().choice =
        DeductionSelection::Itemized { facts: itemized(copy.deductions.as_mut().unwrap()).clone() };
    assert_eq!(value(&supported(&copy), "bond_premium_deduction"), 200);
    assert_ne!(rows[0].id, rows[1].id);
}

#[test]
fn investment_annual_reconciliation_rejects_failed_save_without_allocating_ids() {
    let mut s = investment_session();
    let mut document = annual_interest(vec![annual_row("Bond A", 10_000, 0)]);
    document.taxable += 1;
    let before = s.draft.clone();
    rejected_with(
        save_investments(&mut s, vec![IncomeDocument::Int1099(document)]),
        "DEBT_INSTRUMENT_TOTAL_MISMATCH",
    );
    assert_eq!(s.draft, before);
}

#[test]
fn investment_annual_signed_treasury_and_documented_private_activity_survive() {
    let mut v = printed_oid();
    v.oid = 0;
    v.other_periodic_interest = 0;
    v.more = OidMore {
        treasury_oid: 60_000,
        exempt: 30_000,
        tax_exempt_bond_premium: 6_000,
        private_activity: 8_000,
        ..Default::default()
    };
    let rows = vec![
        AnnualDebtRecord {
            instrument: "TIPS interest".into(),
            amounts: DebtComponents { treasury_oid: 100_000, ..Default::default() },
            ..Default::default()
        },
        AnnualDebtRecord {
            instrument: "TIPS deflation".into(),
            amounts: DebtComponents { treasury_oid: -40_000, ..Default::default() },
            ..Default::default()
        },
        AnnualDebtRecord {
            instrument: "Exempt bond".into(),
            amounts: DebtComponents {
                exempt: 30_000,
                tax_exempt_bond_premium: 6_000,
                private_activity: 8_000,
                ..Default::default()
            },
            private_activity_after_premium: Some(2_000),
            ..Default::default()
        },
    ];
    v.adjustments = Some(InterestAdjustments {
        instrument_records: Some(AnnualDebtRecords { records: rows }),
        ..Default::default()
    });
    let mut s = investment_session();
    accepted(&save_investments(&mut s, vec![IncomeDocument::Oid1099(v)]));
    let id = s.draft.income.last().unwrap().id;
    let values = retained_components(&s.draft, id);
    assert_eq!(
        (values.interest, values.exempt_interest, values.private_activity),
        (60_000, 24_000, 2_000)
    );
    assert_eq!(retained_components(&restart(s.draft).draft, id), values);
}

#[test]
fn investment_net_reports_need_actual_retained_offset_when_not_elected() {
    for answer in [None, Some(false)] {
        for offset in [None, Some(0), Some(8_000)] {
            let mut v = printed_interest();
            v.taxable = 50_000;
            v.more = IntMore { bond_premium: 10_000, ..Default::default() };
            v.adjustments = Some(InterestAdjustments {
                reporting: DebtReporting::Net,
                net_premium_offset: offset
                    .map(|ordinary| DebtPremiumOffset { ordinary, treasury: 0 }),
                ..Default::default()
            });
            let mut documents = vec![IncomeDocument::Int1099(v)];
            if let Some(answer) = answer {
                documents.push(IncomeDocument::TaxableBondPremium(answer));
            }
            let mut s = investment_session();
            accepted(&save_investments(&mut s, documents));
            let r = estimate(&s.draft);
            if let Some(offset) = offset {
                assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:?}", r.issues);
                assert_eq!(return_line(&r, "2b"), 50_000 + offset);
            } else {
                assert_eq!(r.state, ResultState::Unsupported);
                assert!(r.tax.is_none() && r.refund.is_none() && r.balance.is_none());
                assert!(r.return_lines.is_empty());
                assert!(
                    r.issues.iter().any(|i| i.display.key == "ISSUE_DEBT_NET_GROSS_INCOME_UNKNOWN")
                );
            }
            assert_eq!(estimate(&restart(s.draft).draft).state, r.state);
        }
    }
}

#[test]
fn investment_gross_premium_election_preserves_mandatory_exempt_reduction() {
    for (answer, taxable) in [(None, 60_000), (Some(false), 60_000), (Some(true), 50_000)] {
        let mut v = printed_interest();
        v.taxable = 60_000;
        v.more = IntMore {
            bond_premium: 10_000,
            exempt: 10_000,
            tax_exempt_bond_premium: 2_000,
            ..Default::default()
        };
        let mut documents = vec![IncomeDocument::Int1099(v)];
        if let Some(answer) = answer {
            documents.push(IncomeDocument::TaxableBondPremium(answer));
        }
        let mut s = investment_session();
        accepted(&save_investments(&mut s, documents));
        let r = supported(&s.draft);
        assert_eq!((return_line(&r, "2b"), return_line(&r, "2a")), (taxable, 8_000));
    }
}

#[test]
fn investment_annual_net_offset_belongs_to_retained_owner_once() {
    for election in [None, Some(false)] {
        for offset in [None, Some(0), Some(8_000)] {
            let mut row = annual_row("Net instrument", 80_000, 30_000);
            row.reporting = DebtReporting::Net;
            row.nominee = Some(DebtComponents {
                taxable: 20_000,
                bond_premium: 10_000,
                ..Default::default()
            });
            row.net_premium_offset =
                offset.map(|ordinary| DebtPremiumOffset { ordinary, treasury: 0 });
            let mut documents = vec![IncomeDocument::Int1099(annual_interest(vec![row]))];
            if let Some(election) = election {
                documents.push(IncomeDocument::TaxableBondPremium(election));
            }
            let mut s = investment_session();
            accepted(&save_investments(&mut s, documents));
            let r = estimate(&s.draft);
            if let Some(offset) = offset {
                // $800 printed less $200 nominee is $600 retained net.
                // The supplied applied offset is already retained: add it once.
                assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
                assert_eq!(return_line(&r, "2b"), 60_000 + offset);
            } else {
                assert_eq!(r.state, ResultState::Unsupported);
                assert!(r.tax.is_none() && r.refund.is_none() && r.balance.is_none());
                assert!(r.return_lines.is_empty());
                assert!(
                    r.issues.iter().any(|i| i.display.key == "ISSUE_DEBT_NET_GROSS_INCOME_UNKNOWN")
                );
            }
            assert_eq!(estimate(&restart(s.draft).draft), r);
        }
    }
}

#[test]
fn investment_annual_source_identity_is_stable_and_collision_checked() {
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![IncomeDocument::Int1099(annual_interest(vec![
            annual_row("First", 10_000, 0),
            annual_row("Second", 20_000, 0),
        ]))],
    ));
    let (parent, before) = annual_page(&mut s);
    let draft = s.draft.clone();
    let mut page = open_topic(&mut s, TaxSection::InterestDividends);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    let IncomeDocument::Int1099(v) = &mut documents.documents[0].document else { unreachable!() };
    let rows = &mut v.adjustments.as_mut().unwrap().instrument_records.as_mut().unwrap().records;
    rows.reverse();
    rows[0].instrument = "Renamed second".into();
    accepted(&send(&mut s, page));
    let (after_parent, after) = annual_page(&mut s);
    assert_eq!(parent, after_parent);
    assert_eq!((after[0].id, after[1].id), (before[1].id, before[0].id));
    let mut again = restart(s.draft.clone());
    assert_eq!(annual_page(&mut again), (parent, after.clone()));
    // Read-only identities cannot be changed to a parent or repeated child ID.
    for collision in [parent, after[0].id] {
        let mut page = open_topic(&mut s, TaxSection::InterestDividends);
        let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
        let IncomeDocument::Int1099(v) = &mut documents.documents[0].document else {
            unreachable!()
        };
        v.adjustments.as_mut().unwrap().instrument_records.as_mut().unwrap().records[1].id =
            collision;
        let before_failed = s.draft.clone();
        assert!(matches!(send(&mut s, page), Reply::Response(P::Problem { .. })));
        assert_eq!(s.draft, before_failed);
        let mut restored = s.draft.clone();
        let FederalIncome::Interest { data } =
            &mut restored.income.iter_mut().find(|e| e.id == parent).unwrap().income
        else {
            unreachable!()
        };
        data.adjustments.as_mut().unwrap().instrument_records.as_mut().unwrap().records[1].id =
            collision;
        assert!(!restored.valid_stored());
    }
    // An estimate is a pure calculation and cannot allocate child identities.
    let _ = estimate(&draft);
    assert_eq!(draft.income.last().unwrap().id, parent);
    assert_eq!(draft.next_id, before.iter().map(|r| r.id).max().unwrap() + 1);
}

fn save_bond_manual(s: &mut Interview, lines: Vec<OtherItemizedLine>) -> Reply {
    let Reply::Edit(mut page) =
        crate::tests::cross::interview::from_review(s, TaxSection::OtherItemized)
    else {
        panic!("expected deduction page")
    };
    let P::SaveOtherItemized { value, .. } = &mut page else { unreachable!() };
    value.other_lines = lines.into_iter().map(Into::into).collect();
    send(s, page)
}

#[test]
fn investment_bond_premium_manual_control_preserves_independent_amounts_and_deduplicates() {
    let mut a = annual_row("Computed excess", 10_000, 30_000);
    a.premium_history =
        Some(PremiumHistory { prior_net_interest_inclusions: 50_000, ..Default::default() });
    let b = annual_row("Terminal deduction from separate records", 100_000, 0);
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![
            IncomeDocument::Int1099(annual_interest(vec![a, b])),
            IncomeDocument::TaxableBondPremium(true),
        ],
    ));
    let (_, rows) = annual_page(&mut s);
    let Reply::Edit(mut page) =
        crate::tests::cross::interview::from_review(&mut s, TaxSection::Deductions)
    else {
        panic!("expected deductions page")
    };
    let P::SaveDeductions { value: deductions, .. } = &mut page else { unreachable!() };
    deductions.route = DeductionRoute::Itemize;
    accepted(&send(&mut s, page));
    let known = |id| OtherItemizedLine::BondPremium {
        amount: 5_000,
        source: BondPremiumSource::AcceptedAnnualInstrument { instrument: id },
    };
    rejected_with(
        save_bond_manual(&mut s, vec![known(rows[0].id)]),
        "BOND_PREMIUM_COMPUTED_SOURCE_TWICE",
    );
    rejected_with(
        save_bond_manual(&mut s, vec![known(rows[1].id), known(rows[1].id)]),
        "BOND_PREMIUM_SOURCE_TWICE",
    );
    rejected_with(save_bond_manual(&mut s, vec![known(i64::MAX)]), "BOND_PREMIUM_SOURCE_MISSING");
    let manual = vec![
        known(rows[1].id),
        OtherItemizedLine::BondPremium {
            amount: 7_000,
            source: BondPremiumSource::OtherDocumented {
                instrument: "Separate historical documented bond".into(),
            },
        },
    ];
    accepted(&save_bond_manual(&mut s, manual.clone()));
    // $200 computed + $50 independently documented accepted source + $70
    // independently documented historical source; no certification Boolean.
    assert_eq!(value(&supported(&s.draft), "bond_premium_deduction"), 320);
    let mut again = restart(s.draft.clone());
    let Reply::Edit(P::SaveOtherItemized { value: shown, .. }) =
        crate::tests::cross::interview::from_review(&mut again, TaxSection::OtherItemized)
    else {
        unreachable!()
    };
    assert_eq!(
        shown.other_lines,
        manual.into_iter().map(OtherItemizedInput::from).collect::<Vec<_>>()
    );
    assert_eq!(
        shown.bond_instruments.iter().map(|r| r.id).collect::<Vec<_>>(),
        rows.iter().map(|r| r.id).collect::<Vec<_>>()
    );
    let mut invalid = s.draft.clone();
    itemized(invalid.deductions.as_mut().unwrap()).other_lines.push(known(rows[0].id));
    assert!(
        estimate(&invalid)
            .issues
            .iter()
            .any(|i| i.display.key == "BOND_PREMIUM_COMPUTED_SOURCE_TWICE")
    );
}

#[test]
fn investment_fbar_required_choice_preserves_ordinary_tax() {
    for (required, countries) in [
        (false, vec![]),
        (true, vec![0x4341, 0x4652]), // Canada and France, IRS identity bytes.
    ] {
        let mut s = investment_session();
        let baseline = supported(&s.draft);
        let answer = ForeignAccountOrTrust {
            account: true,
            fbar_required: Some(required),
            fbar_countries: countries.clone(),
            trust: false,
            highest_value: Some(4_000_000),
        };
        accepted(&save_investments(
            &mut s,
            vec![IncomeDocument::ForeignAccountOrTrust(answer.clone())],
        ));
        let r = supported(&s.draft);
        assert_eq!(r.tax, baseline.tax);
        let mut again = restart(s.draft);
        let page = open_topic(&mut again, TaxSection::InterestDividends);
        assert_eq!(
            page_documents(&page)[0].document,
            IncomeDocument::ForeignAccountOrTrust(answer)
        );
        let P::SaveIncomes { foreign_countries, .. } = page else { unreachable!() };
        assert_eq!(foreign_countries.len(), 243);
        assert!(
            foreign_countries.iter().any(|c| c.id == 0x4341 && status_text(&c.label) == "Canada")
        );
        assert!(
            !foreign_countries
                .iter()
                .any(|c| [0x5553, 0x4151, 0x4751, 0x5251, 0x5651].contains(&c.id))
        ); // U.S. and territories.
    }
}

#[test]
fn investment_fbar_page_and_restored_answer_guards_reject_invalid_combinations() {
    for (account, required, countries, key) in [
        (true, None, vec![], "FBAR_ANSWER_NEEDED"),
        (false, Some(false), vec![], "FBAR_ANSWER_NEEDED"),
        (true, Some(true), vec![], "FBAR_COUNTRIES_NEEDED"),
        (true, Some(false), vec![0x4341], "FBAR_COUNTRIES_NEEDED"),
        (true, Some(true), vec![0x4341, 0x4341], "FBAR_COUNTRY_TWICE"),
        (true, Some(true), vec![0x5553], "FBAR_COUNTRY_NOT_LISTED"),
    ] {
        let mut s = investment_session();
        let before = s.draft.clone();
        rejected_with(
            save_investments(
                &mut s,
                vec![IncomeDocument::ForeignAccountOrTrust(ForeignAccountOrTrust {
                    account,
                    fbar_required: required,
                    fbar_countries: countries.clone(),
                    trust: false,
                    highest_value: Some(4_000_000),
                })],
            ),
            key,
        );
        assert_eq!(s.draft, before);
        let mut restored = before;
        let screening = restored.screening.as_mut().unwrap();
        screening.foreign_account = account;
        screening.fbar_required = required;
        screening.fbar_countries = countries;
        assert!(!restored.valid_stored());
        assert_eq!(estimate(&restored).state, ResultState::Incomplete);
    }
}

// Derived arithmetic from 2026 DRAFT i8815 (July8/2026), p2 line6 worksheet:
// 3,000 redemption interest - 1,000 prior inclusion = 2,000; line5 remains
// 12,000. Qualified expenses14,000 exceed proceeds, so exclusion is2,000.
#[test]
fn investment_8815_canonical_prior_inclusion_and_source_lifecycle() {
    let s = education_bond_session(printed_interest(), 300_000);
    let r = supported(&s.draft);
    assert_eq!(value(&r, "form_8815_proceeds"), 12_000);
    assert_eq!(value(&r, "form_8815_interest"), 2_000);
    assert_eq!(value(&r, "savings_bond_exclusion"), 2_000);
    assert_eq!(return_line(&r, "2b"), 0);
    let saved = s.draft.clone();
    let mut s = restart(saved.clone());
    assert_eq!(supported(&s.draft), r);
    let page = open_topic(&mut s, TaxSection::InterestDividends);
    let rows = page_documents(&page).to_vec();
    let mut reordered = rows.clone();
    reordered.reverse();
    let request = save_topic(&s.draft, TaxSection::InterestDividends, reordered);
    accepted(&send(&mut s, request));
    let bond = rows
        .iter()
        .find(|r| matches!(r.document, IncomeDocument::SavingsBonds(_)))
        .unwrap()
        .clone();
    let before = s.draft.clone();
    let request = save_topic(&before, TaxSection::InterestDividends, vec![bond]);
    rejected_with(send(&mut s, request), "SAVINGS_BONDS_SOURCE_INVALID");
    assert_eq!(s.draft, before);
    let mut invalid = before.clone();
    let e = invalid
        .income
        .iter_mut()
        .find(|e| matches!(e.income, FederalIncome::SavingsBonds { .. }))
        .unwrap();
    let FederalIncome::SavingsBonds { data } = &mut e.income else { unreachable!() };
    data.interest_statement = 999_999;
    assert_eq!(estimate(&invalid).state, ResultState::Incomplete);
    let request = save_topic(&s.draft, TaxSection::InterestDividends, vec![]);
    accepted(&send(&mut s, request)); // deleting both is an ordinary valid save
}

#[test]
fn investment_8815_mixed_prior_allocation_omits_only_education_benefit() {
    let s = education_bond_session(printed_interest(), 150_000);
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "2b"), 200_000);
    assert!(!r.lines.iter().any(|l| l.id == "savings_bond_exclusion"));
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.display.key == "WARN_SAVINGS_BONDS_PRIOR_ALLOCATION")
            .count(),
        1
    );
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
}

// Derived Schedule B pB-1 seller-financing fixture (2026 DRAFT instructions,
// August10/2026): $5,400 received = $4,000 principal + $1,400 interest.
// Only $1,400 is taxable interest; buyer facts stay attached to the source.
#[test]
fn investment_seller_mortgage_interest_preserves_buyer_and_interest_owner() {
    let mut s = investment_session();
    let v = SellerMortgageInterest {
        buyer: SellerMortgageParty {
            buyer: "Synthetic buyer".into(),
            buyer_tin: "012345678".into(),
            buyer_address: "100 Example St, Example, CA 90001".into(),
        },
        interest: 140_000,
    };
    accepted(&save_investments(&mut s, vec![IncomeDocument::SellerMortgageInterest(v.clone())]));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "2b"), 140_000);
    assert_eq!(value(&r, "agi"), 61_400);
    let source = s.draft.income.iter().find(|e| matches!(&e.income, FederalIncome::Interest { data } if data.seller_mortgage.is_some())).unwrap();
    assert_eq!(source.label, "Synthetic buyer");
    let source_id = source.id;
    let mut s = restart(s.draft.clone());
    let rows = page_documents(&open_topic(&mut s, TaxSection::InterestDividends)).to_vec();
    assert_eq!(rows[0].id, source_id);
    assert_eq!(rows[0].document, IncomeDocument::SellerMortgageInterest(v.clone()));
    assert_eq!(supported(&s.draft), r);
    let before = s.draft.clone();
    let mut invalid = v;
    invalid.buyer.buyer_tin = "ABCDEFGHI".into();
    let request = save_topic(
        &before,
        TaxSection::InterestDividends,
        vec![document(source_id, IncomeDocument::SellerMortgageInterest(invalid))],
    );
    rejected_with(send(&mut s, request), "SELLER_MORTGAGE_BUYER_INVALID");
    assert_eq!(s.draft, before);
}

// Derived composite1099-DIV fixture. Pub550(2025), return of capital and
// complete liquidation rules, and instructions1099-DIV Rev.January2024
// boxes3/9/10: distributions reduce actual lot basis; only excess or a final
// liquidation loss is capital gain/loss. These dollar inputs are invented and
// labeled as a derived probe, not a published IRS worked-dollar example.
// https://www.irs.gov/publications/p550
// https://www.irs.gov/instructions/i1099div
#[test]
fn investment_composite_div_preserves_printed_boxes_and_reuses_capital_owner_once() {
    let mut v = printed_dividend();
    v.more.capital_distributions = 0;
    v.more.section_897_capital = 0;
    v.more.nondividend_distributions = 12_000;
    v.more.cash_liquidation = 30_000;
    v.more.noncash_liquidation = 4_000;
    v.capital_returns = vec![
        CapitalReturn1099Div {
            payer: "Fund lot1".into(),
            kind: CapitalDistributionKind::Nondividend,
            amount: 12_000,
            property: vec![],
            basis: 8_000,
            term: GainTerm::Long,
            special: false,
        },
        CapitalReturn1099Div {
            payer: "Fund lot2".into(),
            kind: CapitalDistributionKind::FinalLiquidation,
            amount: 30_000,
            property: vec![LiquidationProperty {
                label: "Distributed security".into(),
                value: 4_000,
            }],
            basis: 36_000,
            term: GainTerm::Long,
            special: false,
        },
    ];
    let mut s = investment_session();
    accepted(&save_investments(&mut s, vec![IncomeDocument::Div1099(v.clone())]));
    let id = s.draft.income.last().unwrap().id;
    assert_eq!(s.draft.income.len(), 2); //W2 and one actual DIV; no synthetic source records
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "3b"), 100_000);
    assert_eq!(value(&r, "capital_gain"), 20); //120-80=40; final340-360=-20
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_lot_0_basis_reduction")), 80);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_lot_0_remaining_basis")), 0);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_lot_1_capital_gain")), -20);
    let mut s = restart(s.draft);
    assert_eq!(estimate(&s.draft), r);
    let reopened = open_topic(&mut s, TaxSection::InterestDividends);
    assert_eq!(page_documents(&reopened)[0].id, id);
    assert_eq!(page_documents(&reopened)[0].document, IncomeDocument::Div1099(v.clone()));
    let before = s.draft.clone();
    v.more.nondividend_distributions += 1;
    let request = save_topic(
        &s.draft,
        TaxSection::InterestDividends,
        vec![document(id, IncomeDocument::Div1099(v))],
    );
    rejected_with(send(&mut s, request), "DIV_CAPITAL_RETURNS_RECONCILE");
    assert_eq!(s.draft, before);
    let mut bad = before;
    let FederalIncome::Dividend { data } = &mut bad.income.last_mut().unwrap().income else {
        panic!("DIV")
    };
    data.cash_liquidation += 1;
    assert!(!bad.valid_stored());
    assert_eq!(estimate(&bad).state, ResultState::Incomplete);
}

// Pub1212(Rev.December2025), printedJanuary26,2026 p.8 Examples2/3:
// .0025*$1,000*10=$25; original discount$20 is de minimis, $50 is not.
// https://www.irs.gov/publications/p1212 . The source gives no annual
// inclusion for the $50 case, so that assertion is classification only.
#[test]
fn investment_original_oid_de_minimis_matches_irs_examples_and_strict_boundary() {
    let terms = OriginalDebtIssue {
        issue_price: 98_000,
        redemption_price: 100_000,
        complete_years: 10,
        elect_de_minimis_oid: false,
    };
    assert!(terms.de_minimis());
    assert!(!OriginalDebtIssue { issue_price: 95_000, ..terms.clone() }.de_minimis());
    // Derived equality boundary: original discount$25 is not less than$25.
    assert!(!OriginalDebtIssue { issue_price: 97_500, ..terms.clone() }.de_minimis());
    let mut row = annual_row("Ordinary original fixed issue", 0, 0);
    row.original_issue = Some(terms.clone());
    let make = |row: AnnualDebtRecord, amount| Oid1099 {
        adjustments: Some(InterestAdjustments {
            instrument_records: Some(AnnualDebtRecords { records: vec![row] }),
            ..Default::default()
        }),
        ..native_oid1099("Original issuer".into(), amount, 0)
    };
    let mut s = investment_session();
    accepted(&save_investments(&mut s, vec![IncomeDocument::Oid1099(make(row.clone(), 0))]));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "2b"), 0);
    assert!(noted(&r, "NOTE_DEBT_DE_MINIMIS_OID"));
    let mut s = restart(s.draft);
    assert_eq!(estimate(&s.draft), r);
    let before = s.draft.clone();
    // A retained annual$5 amount contradicts the no-election de-minimis facts.
    // This$5 is a derived validation probe, never the source's annual oracle.
    row.amounts.oid = 500;
    rejected_with(
        save_investments(&mut s, vec![IncomeDocument::Oid1099(make(row.clone(), 500))]),
        "DEBT_DE_MINIMIS_OID_CONFLICT",
    );
    assert_eq!(s.draft, before);
    row.original_issue.as_mut().unwrap().elect_de_minimis_oid = true;
    accepted(&save_investments(&mut s, vec![IncomeDocument::Oid1099(make(row, 500))]));
    assert_eq!(return_line(&supported(&s.draft), "2b"), 500);
}

#[test]
fn investment_schedule_b_lists_seller_first_and_named_adjustments_foot() {
    let mut v = printed_interest();
    v.taxable = 150_000;
    v.more = IntMore {
        us_obligations: 50_000,
        bond_premium: 20_000,
        treasury_bond_premium: 10_000,
        accrued_interest: 5_000,
        ..Default::default()
    };
    v.nominee = Some(InterestNominee {
        taxable: 30_000,
        us_obligations: 10_000,
        bond_premium: 4_000,
        treasury_bond_premium: 2_000,
        accrued_interest: 1_000,
        ..Default::default()
    });
    v.adjustments = Some(InterestAdjustments { previously_reported: 10_000, ..Default::default() });
    let seller = SellerMortgageInterest {
        buyer: SellerMortgageParty {
            buyer: "Buyer".into(),
            buyer_tin: "123456789".into(),
            buyer_address: "1 Main St, Boston MA02110".into(),
        },
        interest: 140_000,
    };
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![
            IncomeDocument::Int1099(v),
            IncomeDocument::SellerMortgageInterest(seller),
            IncomeDocument::TaxableBondPremium(true),
        ],
    ));
    let broker = s.draft.income[1].id;
    let buyer = s.draft.income[2].id;
    let r = supported(&s.draft);
    for (suffix, expected) in [
        ("payer", 2000),
        ("nominee", -400),
        ("prior", -100),
        ("accrued", -40),
        ("adjustment", -240),
    ] {
        assert_eq!(value(&r, &alloc::format!("schedule_b_interest_{broker}_{suffix}")), expected);
    }
    assert_eq!(value(&r, "schedule_b_total_interest"), 2620);
    assert_eq!(value(&r, "schedule_b_taxable_interest"), 2620);
    assert_eq!(return_line(&r, "2b"), 262_000);
    let rows = r
        .lines
        .iter()
        .filter(|line| {
            line.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleB && f.line == "1")
        })
        .collect::<Vec<_>>();
    assert_eq!(rows[0].id, alloc::format!("schedule_b_interest_{buyer}_payer"));
    assert_eq!(rows.iter().map(|line| line.amount).sum::<i64>(), 262_000);
    assert!(noted(&r, "NOTE_SCHEDULE_B_PART_III"));
}

#[test]
fn investment_seller_schedule_b_below_threshold_does_not_require_part_iii() {
    let mut s = investment_session();
    accepted(&save_investments(
        &mut s,
        vec![IncomeDocument::SellerMortgageInterest(SellerMortgageInterest {
            buyer: SellerMortgageParty {
                buyer: "Buyer".into(),
                buyer_tin: "123456789".into(),
                buyer_address: "1 Main St, Boston MA 02110".into(),
            },
            interest: 140_000,
        })],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "schedule_b_taxable_interest"), 1400);
    assert!(!noted(&r, "NOTE_SCHEDULE_B_PART_III"));
}

// Pub.550 nominee and Pub.559 final-return component ownership, carried into
// the approved foreign deduction. Per-instrument retained foreign tax is not
// the printed payer total:100−30nominee−20later-recipient=50; another instrument
// contributes25. Additional investment tax5 makes ScheduleA6/NIIT tax80 once.
#[test]
fn foreign_deduction_uses_retained_instrument_tax_after_nominee_and_death() {
    let mut a = annual_row("Synthetic shared bond", 100_000, 0);
    a.amounts.foreign_tax = 10_000;
    a.nominee = Some(DebtComponents { taxable: 30_000, foreign_tax: 3_000, ..Default::default() });
    a.after_death =
        Some(DebtComponents { taxable: 20_000, foreign_tax: 2_000, ..Default::default() });
    let mut b = annual_row("Synthetic retained bond", 100_000, 0);
    b.amounts.foreign_tax = 2_500;
    let mut s = investment_session();
    s.draft.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260630, ordinary: true, bonds: None, estate_medical: 0 });
    accepted(&save_investments(&mut s, vec![IncomeDocument::Int1099(annual_interest(vec![a, b]))]));
    s.draft.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Deduction {
        taxes: vec![ForeignTaxDeducted {
            owners: None,
            label: "Synthetic other portfolio tax".into(),
            income: ForeignTaxIncome::Investment,
            tax: 500,
        }],
    };
    let facts = itemized(s.draft.deductions.as_mut().unwrap()).clone();
    s.draft.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized { facts };
    let accepted_source = s.draft.clone();
    let r = supported(&s.draft);
    assert_eq!(value(&r, "other_taxes_deduction"), 80);
    assert_eq!(value(&r, "foreign_investment_tax_deduction"), 80);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(s.draft, accepted_source);
    assert_eq!(supported(&restart(s.draft).draft), r);
}
