use crate::k1_page::K1Scorp;
use crate::tests::*;

pub(in crate::tests) fn scorp_document() -> ScorpK1 {
    ScorpK1 {
        amt_record_choice: None,
        research_credit_line: None,
        amt_section179_income_carry: None,
        reporting_period: None,
        qbi_aggregation_group: None,
        health_insurance: BusinessHealthInsurance::None,
        section179_carryover_sold: 0,
        section179_sales: alloc::vec::Vec::new(),
        foreign_taxes: 0,
        carried: Default::default(),
        schedule_k3: None,
        rentals: Default::default(),
        rental_facts: crate::K1RentalFacts { participation: None, ..Default::default() },
        section179: 0,
        section179_carryover: 0,
        section1231: None,
        capital: None,
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        exempt_interest: 0,
        private_activity: 0,
        withholding: 0,
        trade_kind: ScorpTradeKind::Ordinary,
        passive: ScorpPassiveHistory { loss: 0, complete: true },
        property_qbi: Default::default(),
        loss_qbi: Default::default(),
        qbi_statement: None,
        amt: None,
        participation: None,
        risk: ScorpRisk {
            prior_nondeductible: 0,
            start: ScorpRiskStart::Unknown,
            excluded_funding: 0,
            excluded_returns: 0,
            protected: 0,
            released: 0,
            prior_loss: 0,
            deducted: 0,
            recaptured: 0,
            complete: true,
        },
        ordinary: -1_500_000,
        opening: Some(1_000_000),
        additions: 0,
        exempt: 0,
        distributions: 0,
        nondeductible: 0,
        prior_loss: 0,
        prior_nondeductible: 0,
        losses_first: false,
        loans: vec![ScorpLoan {
            repayment_term: ScorpNoteHolding::NoRepayment,
            kind: ScorpLoanKind::FormalNote,
            principal: 300_000,
            basis: 300_000,
            advances: 0,
            repayments: 0,
        }],
        cash_charity: 0,
        stock_term: None,
        credits: Vec::new(),
        unentered: vec![],
        final_k1: false,
        income: Vec::new(),
        deductions: Vec::new(),
        disposition: Some(crate::K1Disposition::Kept),
        special: Vec::new(),
        investment_property: Vec::new(),
        investment_confirmed: None,
        housing_confirmed: None,
    }
}

pub(in crate::tests) fn scorp_risk_history() -> ScorpRisk {
    ScorpRisk {
        prior_nondeductible: 0,
        start: ScorpRiskStart::Detailed { capacity: 600_000, profit: 0, loss: 100_000 },
        excluded_funding: 0,
        excluded_returns: 0,
        protected: 0,
        released: 0,
        prior_loss: 200_000,
        deducted: 500_000,
        recaptured: 0,
        complete: true,
    }
}
pub(in crate::tests) fn scorp_note(r: &FederalResult, id: i64, name: &str, amount: i64) {
    assert!(
        r.review_notes.iter().any(|note| {
            note.starts_with(&alloc::format!("S-corporation document {id}, {name}: $"))
                && note.display.args.iter().find_map(|arg| match arg {
                    crate::MessageArg::Currency { scale: 0, minor_units, .. } => Some(*minor_units),
                    crate::MessageArg::Currency { scale: 2, minor_units, .. } => {
                        Some(crate::kernel::ratio::dollars(*minor_units))
                    }
                    _ => None,
                }) == Some(amount)
        }),
        "{r:?}"
    );
}

pub(in crate::tests) fn participation_facts() -> ScorpParticipation {
    ScorpParticipation {
        ordinary_scope: true,
        minutes: 0,
        substantially_all: false,
        at_least_as_much: false,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
        regular: false,
        management_minutes: 0,
        paid_manager: false,
        other_manager_more: false,
        other_business_over_100: false,
    }
}

pub(in crate::tests) fn passive_scorp_document() -> ScorpK1 {
    let mut source = scorp_document();
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::Detailed { capacity: 800_000, profit: 0, loss: 0 };
    risk.prior_loss = 0;
    source.risk = risk;
    source.participation = Some(participation_facts());
    source.passive = ScorpPassiveHistory { loss: 200_000, complete: true };
    source
}
pub(in crate::tests) fn passive_note(r: &FederalResult, id: i64, net: i64, suspended: i64) {
    assert!(r.review_notes.iter().any(|v| v.starts_with(&alloc::format!("Passive activity {id}, net after passive limits: ${net}; remaining passive loss: ${suspended}."))), "{r:?}");
}

pub(in crate::tests) fn nonpassive_scorp(ordinary: i64, prior: i64) -> ScorpK1 {
    let mut source = passive_scorp_document();
    source.ordinary = ordinary * 100;
    source.passive.loss = prior * 100;
    source.participation.as_mut().unwrap().minutes = 30_001;
    source
}

pub(in crate::tests) fn scorp_loss_qbi_source() -> ScorpK1 {
    let mut source = scorp_document();
    source.prior_loss = 400_000;
    source.loss_qbi = ScorpLossQbi {
        passive: vec![],
        at_risk: vec![],
        current: 750_000,
        history: vec![
            ScorpLossYear {
                year: 2020,
                original: 800_000,
                qualified: 200_000,
                remaining: 200_000,
                remaining_qualified: 50_000,
            },
            ScorpLossYear {
                year: 2017,
                original: 200_000,
                qualified: 0,
                remaining: 200_000,
                remaining_qualified: 0,
            },
        ],
    };
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        complete: true,
    });
    source.participation = Some(participation_facts());
    source.participation.as_mut().unwrap().minutes = 30_001;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_300_000, profit: 0, loss: 0 };
    source
}

// Loss-year histories for each carried loss the QBI tracking lacks, as
// nonqualified 2025 losses, and the section 1231 confirmation: beside a
// section 199A statement the page requires them.
pub(in crate::tests) fn reconcile_qbi_tracking(source: &mut ScorpK1) {
    let fill = |history: &mut alloc::vec::Vec<ScorpLossYear>, amount: i64| {
        if amount > 0 && history.is_empty() {
            *history = vec![ScorpLossYear {
                year: 2025,
                original: amount,
                qualified: 0,
                remaining: amount,
                remaining_qualified: 0,
            }];
        }
    };
    fill(&mut source.loss_qbi.history, source.prior_loss);
    fill(&mut source.loss_qbi.at_risk, source.risk.prior_loss);
    fill(&mut source.loss_qbi.passive, source.passive.loss);
    if let Some(property) = source.section1231.clone() {
        let losses = &mut source.property_qbi.losses;
        fill(&mut losses.history, property.basis_loss);
        fill(&mut losses.at_risk, property.risk_loss);
        fill(&mut losses.passive, property.passive_loss);
        source.property_qbi.complete = true;
    }
}

pub(in crate::tests) fn loss_year(
    year: u32,
    original: i64,
    qualified: i64,
    remaining: i64,
    remaining_qualified: i64,
) -> ScorpLossYear {
    ScorpLossYear {
        year,
        original: original * 100,
        qualified: qualified * 100,
        remaining: remaining * 100,
        remaining_qualified: remaining_qualified * 100,
    }
}

pub(in crate::tests) fn passive_qbi_source() -> ScorpK1 {
    let mut source = passive_scorp_document();
    source.loss_qbi = ScorpLossQbi {
        current: 750_000,
        history: vec![],
        at_risk: vec![],
        passive: vec![loss_year(2017, 2_000, 0, 2_000, 0)],
    };
    source.qbi_statement = complete_ordinary_scorp(0, false).qbi_statement;
    source
}

pub(in crate::tests) fn scorp_current_qbi_source() -> ScorpK1 {
    let mut source = nonpassive_scorp(40_000, 8_000);
    source.loss_qbi = ScorpLossQbi {
        current: 0,
        history: vec![],
        at_risk: vec![],
        passive: vec![loss_year(2020, 8_000, 8_000, 8_000, 8_000)],
    };
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 4_000_000,
        wages: 1_000_000,
        property: 0,
        service: true,
        complete: true,
    });
    source
}

pub(in crate::tests) fn scorp_current_loss_statement() -> ScorpK1 {
    let mut source = passive_qbi_source();
    source.participation.as_mut().unwrap().minutes = 30_001;
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: true,
        complete: true,
    });
    source
}

pub(in crate::tests) fn investment_tax_draft(
    source: ScorpK1,
    bank_interest: i64,
) -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    if bank_interest > 0 {
        add(
            &mut d,
            income(FederalIncome::Interest {
                data: interest_source(native_interest(bank_interest * 100, 0)),
            }),
        );
    }
    (d, id)
}
pub(in crate::tests) fn investment_tax_note(r: &FederalResult, income: i64, tax: i64) {
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!("K-1 investment-tax worksheet: net investment income ${income}; net investment income tax ${tax}."))), "{r:?}");
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.starts_with("K-1 investment-tax worksheet:"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_K1_INVESTMENT_TAX");
    assert_eq!(note.args, vec![MessageArg::usd_dollars(income), MessageArg::usd_dollars(tax)]);
}

pub(in crate::tests) fn scorp_large_loss(amount: i64, capacity: i64, material: bool) -> ScorpK1 {
    let mut source = nonpassive_scorp(-amount, 0);
    source.opening = Some(capacity * 100);
    source.loans.clear();
    source.risk.start = ScorpRiskStart::Detailed { capacity: capacity * 100, profit: 0, loss: 0 };
    source.participation.as_mut().unwrap().minutes = if material { 30_001 } else { 0 };
    source
}
pub(in crate::tests) fn k1_excess_note(r: &FederalResult, net: i64, excess: i64) {
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!("K-1 excess-business-loss worksheet: ordinary K-1 net after activity limits ${net}; return excess business loss ${excess}."))), "{r:?}");
}

pub(in crate::tests) fn sole_qbi_loss(amount: i64, basis_capacity: i64, qualified: i64) -> ScorpK1 {
    let mut source = scorp_large_loss(amount, basis_capacity, true);
    source.loss_qbi = ScorpLossQbi {
        current: qualified * 100,
        history: vec![],
        at_risk: vec![],
        passive: vec![],
    };
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        complete: true,
    });
    source
}

pub(in crate::tests) fn amt_records(source: &ScorpK1) -> ScorpAmt {
    ScorpAmt {
        rentals: source.rentals.clone(),
        carried: source.carried.clone(),
        section1231: source.section1231.clone(),
        capital: source.capital.clone(),
        adjustment: 0,
        stock: source.opening.expect("known fixture opening basis"),
        prior_loss: source.prior_loss,
        prior_nondeductible: source.prior_nondeductible,
        loans: source
            .loans
            .iter()
            .enumerate()
            .map(|(i, v)| ScorpAmtDebt { loan: i as u32 + 1, basis: v.basis })
            .collect(),
        risk: source.risk.clone(),
        passive: source.passive.clone(),
        complete: true,
    }
}
pub(in crate::tests) fn amt_k1_note(r: &FederalResult, amt: i64, regular: i64) {
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!("K-1 AMT activity worksheet: AMT ordinary net ${amt}; regular ordinary net ${regular}; income adjustment ${}.", amt - regular))), "{r:?}");
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.starts_with("K-1 AMT activity worksheet:"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_K1_AMT_ACTIVITY");
    assert_eq!(
        note.args,
        vec![
            MessageArg::usd_dollars(amt),
            MessageArg::usd_dollars(regular),
            MessageArg::usd_dollars(amt - regular)
        ]
    );
}

pub(in crate::tests) fn k1_eic_investment_note(r: &FederalResult, investment: i64) {
    assert!(
        r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "K-1 earned-income-credit worksheet: investment income ${investment}."
        ))),
        "{r:?}"
    );
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.starts_with("K-1 earned-income-credit worksheet:"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_K1_EIC_INVESTMENT_INCOME");
    assert_eq!(note.args, vec![MessageArg::usd_dollars(investment)]);
}

pub(in crate::tests) fn k1_nol_note(
    r: &FederalResult,
    label: &str,
    loss: i64,
    excess: i64,
    total: i64,
) {
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
        "K-1 {label} loss carryforward worksheet: current operating loss ${loss}; excess business loss ${excess}; debt reduction $0; total carryforward ${total}."
    ))), "{r:?}");
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.starts_with(&alloc::format!("K-1 {label} loss carryforward worksheet:")))
        .unwrap()
        .display;
    assert_eq!(
        note.key,
        if label == "AMT" {
            "NOTE_K1_AMT_LOSS_CARRYFORWARD"
        } else {
            "NOTE_K1_REGULAR_LOSS_CARRYFORWARD"
        }
    );
    assert_eq!(
        note.args,
        vec![
            MessageArg::usd_dollars(loss),
            MessageArg::usd_dollars(excess),
            MessageArg::usd_dollars(0),
            MessageArg::usd_dollars(total),
        ]
    );
}

pub(in crate::tests) fn trading_interest_draft(
    ordinary: i64,
    material: bool,
    adjustment: i64,
) -> FederalDraft {
    let mut source = nonpassive_scorp(ordinary, 0);
    source.trade_kind = ScorpTradeKind::FinancialTrading;
    source.participation = Some(participation_facts());
    if material {
        source.participation.as_mut().unwrap().minutes = 30_001;
    }
    let mut amt = amt_records(&source);
    amt.adjustment = adjustment * 100;
    source.amt = Some(amt);
    let (mut d, _) = investment_tax_draft(source, 10_000);
    investment_borrowing(&mut d, 25_000, 0, 0);
    d
}
pub(in crate::tests) fn k1_interest_note(r: &FederalResult, regular: i64, alternative: i64) {
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
        "K-1 investment-interest worksheet: regular allowed ${regular}; carryforward ${}; AMT allowed ${alternative}; carryforward ${}.", 25_000-regular, 25_000-alternative
    ))), "{r:?}");
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.starts_with("K-1 investment-interest worksheet:"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_K1_INVESTMENT_INTEREST");
    assert_eq!(
        note.args,
        vec![
            MessageArg::usd_dollars(regular),
            MessageArg::usd_dollars(25_000 - regular),
            MessageArg::usd_dollars(alternative),
            MessageArg::usd_dollars(25_000 - alternative),
        ]
    );
}

pub(in crate::tests) fn qbi_record_issues(r: &FederalResult, id: i64) -> alloc::vec::Vec<&str> {
    r.issues
        .iter()
        .filter(|i| i.section == alloc::format!("S-corporation QBI records: {id}"))
        .map(|i| i.message.as_str())
        .collect()
}

pub(in crate::tests) fn complete_ordinary_scorp(profit: i64, material: bool) -> ScorpK1 {
    let mut source = nonpassive_scorp(profit, 0);
    source.participation = Some(participation_facts());
    if material {
        source.participation.as_mut().unwrap().minutes = 30_001;
    }
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: profit.max(0) * 100,
        wages: 0,
        property: 0,
        service: false,
        complete: true,
    });
    source.amt = Some(amt_records(&source));
    source
}

pub(in crate::tests) fn complete_scorp_recapture(regular: bool, alternative: bool) -> ScorpK1 {
    let mut source = complete_ordinary_scorp(1_000, true);
    source.distributions = 300_000;
    let risk = &mut source.risk;
    risk.start = ScorpRiskStart::Detailed {
        capacity: if regular { 100_000 } else { 500_000 },
        profit: 0,
        loss: 0,
    };
    risk.deducted = 500_000;
    let mut amt = amt_records(&source);
    amt.risk.start = ScorpRiskStart::Detailed {
        capacity: if alternative { 100_000 } else { 500_000 },
        profit: 0,
        loss: 0,
    };
    source.amt = Some(amt);
    source
}

pub(in crate::tests) fn historical_k1_excess(qualified: i64, at_risk: bool) -> ScorpK1 {
    let mut source = sole_qbi_loss(0, if at_risk { 0 } else { 400_000 }, 0);
    let year = loss_year(2020, 400_000, qualified, 400_000, qualified);
    if at_risk {
        source.additions = 40_000_000;
        source.risk.prior_loss = 40_000_000;
        source.loss_qbi.at_risk = vec![year];
    } else {
        source.prior_loss = 40_000_000;
        source.loss_qbi.history = vec![year];
    }
    source.amt = Some(amt_records(&source));
    source
}

pub(in crate::tests) fn scorp_repaid_reduced_open_account(kind: ScorpLoanKind) -> ScorpK1 {
    let mut source = complete_ordinary_scorp(0, true);
    source.loans = vec![ScorpLoan {
        repayment_term: ScorpNoteHolding::NoRepayment,
        kind,
        principal: 1_000_000,
        basis: 600_000,
        advances: 0,
        repayments: 500_000,
    }];
    source.risk.start = ScorpRiskStart::Detailed { capacity: 600_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    source
}

pub(in crate::tests) fn repaid_written_note(term: GainTerm) -> ScorpK1 {
    let mut source = scorp_repaid_reduced_open_account(ScorpLoanKind::FormalNote);
    source.loans[0].repayment_term = ScorpNoteHolding::Single { term };
    source
}
pub(in crate::tests) fn investment_sale_gain(
    term: GainTerm,
    regular: i64,
    amt: i64,
) -> FederalIncome {
    FederalIncome::Sale {
        data: sale_source(FederalSale {
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            proceeds: 500_000,
            basis: Some(500_000 - regular * 100),
            amt: crate::SaleAmt::Different {
                record: CapitalSaleAmt { term: None, basis: 500_000 - amt * 100, wash_sale: 0 },
            },
            collectible: false,
            withholding: 0,
            term,
            use_kind: SaleUse::Investment { wash_sale: 0 },
            special: crate::SaleSpecial::NoneOfThese,
            adjustment: SaleAdjustment::None,
            basis_reporting: BasisReporting::ReportedToIrs,
        }),
    }
}

pub(in crate::tests) fn mixed_note_repayments() -> ScorpK1 {
    let mut source = repaid_written_note(GainTerm::Short);
    source.loans[0].repayment_term = ScorpNoteHolding::Dated {
        advanced: 20250615,
        payments: vec![
            ScorpNoteRepayment { date: 20260615, principal: 200_000 },
            ScorpNoteRepayment { date: 20260616, principal: 300_000 },
        ],
    };
    source
}

pub(in crate::tests) fn scorp_portfolio() -> ScorpK1 {
    let mut source = complete_ordinary_scorp(0, true);
    source.interest = 300_000;
    source.dividends = 400_000;
    source.qualified_dividends = 200_000;
    source
}

// A printed179 amount with no basis allowance and explicitly no retained
// income carry, independently on regular and alternative source records.
pub(in crate::tests) fn basis_blocked_section179_scorp() -> ScorpK1 {
    let mut source = complete_ordinary_scorp(0, true);
    source.opening = Some(0);
    source.loans.clear();
    source.section179 = 100_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    source.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    source
}

pub(in crate::tests) fn k1_boundary_continuing(page: &mut K1Scorp) -> &mut K1ScorpCarried {
    use crate::tests::cross::interview::k1_1120s_continuing;
    if matches!(page.basis.start, K1ScorpStart::New) {
        page.basis.start = k1_1120s_continuing(100_000, 100_000);
    }
    let K1ScorpStart::Continuing { carried, .. } = &mut page.basis.start else { unreachable!() };
    carried
}

pub(in crate::tests) fn scorp_loss_statement(
    loss_qbi: ScorpLossQbi,
    property_qbi: K1ScorpPropertyQbi,
) -> K1ScorpQbi {
    K1ScorpQbi::Statement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        current_loss: loss_qbi.current,
        property_qbi,
    }
}

// Fully admitted retained expense under the losses-first election, with missing
// at-risk history and an ordinary basis loss released against current income.
pub(in crate::tests) fn scorp_retained_expense_page() -> K1Scorp {
    let mut page = crate::tests::cross::interview::k1_1120s_fixture(10_000);
    page.qbi = K1ScorpQbi::NoStatement;
    page.more.nondeductible = 50_000;
    let K1ScorpStart::Continuing { carried, risk_history, .. } = &mut page.basis.start else {
        unreachable!()
    };
    *risk_history = ScorpRiskStart::Unknown;
    carried.losses_first = true;
    carried.basis_nondeductible = 100_000;
    carried.basis_loss = 200_000;
    page
}

// A reduced-basis loan repaid without corporate income to restore debt basis.
pub(in crate::tests) fn scorp_no_loss_repayment_page(kind: ScorpLoanKind) -> K1Scorp {
    let mut page = crate::tests::cross::interview::k1_1120s_fixture(0);
    page.qbi = K1ScorpQbi::NoStatement;
    let K1ScorpStart::Continuing { risk_history, .. } = &mut page.basis.start else {
        unreachable!()
    };
    *risk_history = ScorpRiskStart::Unknown;
    page.basis.loans = vec![ScorpLoan {
        kind,
        repayment_term: if kind == ScorpLoanKind::FormalNote {
            ScorpNoteHolding::Single { term: GainTerm::Long }
        } else {
            ScorpNoteHolding::NoRepayment
        },
        principal: 1_000_000,
        basis: 200_000,
        advances: 0,
        repayments: 500_000,
    }];
    page.basis.more_at_risk.excluded_returns = 100_000;
    page
}

// $5,000 basis-admitted loss with $1,000 opening at risk. All $500
// distributions and $5,000 principal repayments return excluded investment.
pub(in crate::tests) fn scorp_fully_excluded_repayment_page(kind: ScorpLoanKind) -> K1Scorp {
    let mut page = scorp_no_loss_repayment_page(kind);
    page.ordinary = -500_000;
    page.basis.start = crate::tests::cross::interview::k1_1120s_continuing(1_000_000, 100_000);
    page.distributions = 50_000;
    page.basis.more_at_risk.excluded_returns = 550_000;
    page
}

// One activity with current expenses fully admitted by basis before its loss.
// The independent AMT stock and risk capacities bind the same source differently.
pub(in crate::tests) fn scorp_binding_current_expense_page() -> K1Scorp {
    let mut page = crate::tests::cross::interview::k1_1120s_fixture(-1_000);
    page.qbi = K1ScorpQbi::NoStatement;
    page.more.nondeductible = 20_000;
    page.basis.start = crate::tests::cross::interview::k1_1120s_continuing(50_000, 25_000);
    page.basis.loans.clear();
    let mut record = ScorpAmtRecords::same_as(&page.record()).unwrap();
    record.stock = 70_000;
    record.risk.start = ScorpRiskStart::Detailed { capacity: 35_000, profit: 0, loss: 0 };
    page.amt = Some(ScorpAmtRecords::Separate { record });
    page
}

pub(in crate::tests) fn scorp_paper_document(
    page: crate::k1_page::K1Scorp,
) -> crate::K1ScorpDocument {
    use crate::partnership_source::credit_code_name;
    use crate::scorp_source::{deduction_code_name, income_code};
    let row = |code: &str, amount: Option<i64>| crate::K1CodeRow {
        code: code.into(),
        amount,
        text: if amount.is_none() { "STMT".into() } else { alloc::string::String::new() },
        statement: amount.is_none(),
    };
    let record = page.record();
    let mut form = crate::K1ScorpPaper {
        reporting_period: page.reporting_period,
        final_k1: page.final_k1,
        corporation: page.corporation.clone(),
        shareholder: page.shareholder.clone(),
        ordinary: Some(page.ordinary),
        rental_real_estate: Some(page.rentals.real_estate),
        other_rental: Some(page.rentals.other),
        interest: Some(page.interest),
        dividends: Some(page.dividends),
        qualified_dividends: Some(page.qualified_dividends),
        royalties: Some(page.royalties),
        short: Some(page.more.short),
        long: Some(page.more.long),
        collectibles: Some(page.more.collectibles),
        unrecaptured: Some(
            page.more.unrecaptured_partnership
                + page.more.unrecaptured_fund
                + page.more.unrecaptured_business
                + page.rentals.real_estate_unrecaptured
                + page.rentals.other_unrecaptured,
        ),
        section1231: Some(
            page.more.section1231
                + page.rentals.real_estate_section1231
                + page.rentals.other_section1231,
        ),
        section179: Some(page.more.section179),
        k3_attached: matches!(page.schedule_k3, crate::K3Attached::Attached(_)),
        loans: record.loan_source_totals().map(|[beginning, ending, _]| crate::K1MoneyPair {
            beginning: Some(beginning),
            ending: Some(ending),
        }),
        ..Default::default()
    };
    // The test's original worksheet supplies known zero principal when it has no loans.
    if record.loans.is_empty() {
        form.loans = Some(crate::K1MoneyPair { beginning: Some(0), ending: Some(0) });
    }
    let mut income_totals = alloc::collections::BTreeMap::<&str, i64>::new();
    for item in &page.more.income {
        if income_code(item.code) != "K" {
            *income_totals.entry(income_code(item.code)).or_default() += item.amount;
        }
    }
    form.income = income_totals.into_iter().map(|(code, amount)| row(code, Some(amount))).collect();
    if page.more.income.iter().any(|i| income_code(i.code) == "K") {
        form.income.push(row("K", None));
    }
    form.deductions = page
        .more
        .deductions
        .iter()
        .map(|d| row(deduction_code_name(d.code), Some(d.amount)))
        .collect();
    if page.more.cash_charity != 0 {
        form.deductions.push(row("A", Some(page.more.cash_charity)));
    }
    let credits: Vec<crate::K1Credit> = page.more.credits.iter().cloned().map(Into::into).collect();
    let mut credit_totals = alloc::collections::BTreeMap::<&str, i64>::new();
    for credit in &credits {
        *credit_totals.entry(credit_code_name(&credit.code)).or_default() += credit.amount;
    }
    form.credits = credit_totals.into_iter().map(|(code, a)| row(code, Some(a))).collect();
    if page.more.withholding != 0 {
        form.credits.push(row("O", Some(page.more.withholding)));
    }
    for (code, amount) in [
        ("A", page.more.exempt_interest),
        ("B", page.more.exempt),
        ("C", page.more.nondeductible),
        ("D", page.distributions),
        ("E", record.loans.iter().map(|l| l.repayments).sum()),
        ("F", page.more.foreign_taxes),
    ] {
        if amount != 0 {
            form.basis.push(row(code, Some(amount)));
        }
    }
    if page.more.private_activity != 0 {
        form.amt.push(row("F", Some(page.more.private_activity)));
    }
    if matches!(page.qbi, crate::K1ScorpQbi::Statement { .. }) {
        form.information.push(row("V", None));
    }
    if !page.more.section179_sales.is_empty() {
        form.information.push(row("K", None));
    }
    for p in &page.more.investment_property {
        match p {
            crate::K1InvestmentProperty::RehabilitationRental { .. } => {
                form.credits.push(row("E", None))
            }
            crate::K1InvestmentProperty::RehabilitationOther { .. } => {
                form.information.push(row("C", None))
            }
            crate::K1InvestmentProperty::Energy { .. } => form.information.push(row("D", None)),
            crate::K1InvestmentProperty::CleanElectricity { .. } => {
                form.information.push(row("AV", None))
            }
        }
    }
    let statements = crate::ScorpStatements {
        qbi: page.qbi.clone(),
        rentals: crate::PartnershipRentalStatement {
            real_estate_section1231: page.rentals.real_estate_section1231,
            real_estate_unrecaptured: page.rentals.real_estate_unrecaptured,
            other_section1231: page.rentals.other_section1231,
            other_unrecaptured: page.rentals.other_unrecaptured,
            real_estate_qbi: page.rentals.real_estate_qbi.clone(),
            other_qbi: page.rentals.other_qbi.clone(),
        },
        property: crate::ScorpPropertyStatement {
            unrecaptured_partnership: page.more.unrecaptured_partnership,
            unrecaptured_fund: page.more.unrecaptured_fund,
            unrecaptured_business: page.more.unrecaptured_business,
            section1231_business: page.more.section1231,
        },
        income: page
            .more
            .income
            .iter()
            .map(|i| crate::ScorpIncomeStatement {
                code: i.code,
                amount: (page
                    .more
                    .income
                    .iter()
                    .filter(|other| income_code(other.code) == income_code(i.code))
                    .count()
                    > 1
                    || income_code(i.code) == "K")
                    .then_some(i.amount),
            })
            .collect(),
        credits: credits
            .iter()
            .map(|c| crate::PartnershipCreditStatement {
                code: c.code,
                line: (c.code != crate::K1CreditCode::Research).then_some(c.line).flatten(),
                amount: (credits.iter().filter(|o| o.code == c.code).count() > 1)
                    .then_some(c.amount),
            })
            .collect(),
        investment_property: page.more.investment_property.clone(),
        section179_sales: page.more.section179_sales.clone(),
        private_activity: (page.more.private_activity != 0).then_some(page.more.private_activity),
        schedule_k3: match &page.schedule_k3 {
            crate::K3Attached::NotAttached => None,
            crate::K3Attached::Attached(k3) => Some(super::k3_paper::schedule_k3_paper(
                k3,
                crate::ScheduleK3Paper::Scorp(crate::K3ScorpPaper {
                    entity: form.corporation.clone(),
                    recipient: form.shareholder.clone(),
                    ..Default::default()
                }),
            )),
        },
        situations: page.more.unentered.clone(),
    };
    remember_scorp_document(
        crate::K1ScorpDocument { form, statements },
        crate::ScorpPersonalRecords::from_page(&page),
    )
}
impl From<crate::k1_page::K1Scorp> for crate::K1ScorpDocument {
    fn from(page: crate::k1_page::K1Scorp) -> Self {
        scorp_paper_document(page)
    }
}

pub(in crate::tests) fn complete_scorp_paper(
    draft: &FederalDraft,
) -> (crate::k1_page::K1Scorp, crate::K1ScorpDocument) {
    let mut page = crate::tests::cross::interview::k1_1120s_fixture(20_000);
    page.reporting_period = Some(crate::K1EntityTaxPeriod { began: 20260101, ended: 20261231 });
    page.more.credits = vec![
        crate::K1Credit {
            code: crate::K1CreditCode::EmployerTipFica,
            line: None,
            amount: 100_049,
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
        }
        .into(),
        crate::K1Credit {
            code: crate::K1CreditCode::Research,
            line: Some(crate::BusinessCredit::ResearchSmallBusiness),
            amount: 20_031,
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
        }
        .into(),
    ];
    let mut paper: crate::K1ScorpDocument = page.clone().into();
    let personal = scorp_fixture_source(&paper).personal;
    paper.form.amended_k1 = true;
    paper.form.corporation.federal_id = "00-1234567".into();
    paper.form.corporation.address = "17 Synthetic Lane, Sample, NY 10001".into();
    paper.form.shareholder =
        crate::tests::support::builders::fixture_document_names(draft, -1).unwrap().recipient;
    paper.form.shareholder.federal_id = "***-**-4322".into();
    paper.form.shareholder.address = "23 Synthetic Road, Sample, NY 10002".into();
    paper.form.reporting_party = crate::K1DisregardedEntity {
        tin: "***-**-4322".into(),
        name: paper.form.shareholder.name.clone(),
    };
    paper.form.irs_center = "Ogden".into();
    paper.form.shareholder_type = "Individual".into();
    paper.form.allocation_percentage = Some(12.5);
    paper.form.corporation_shares =
        crate::K1ShareCountPair { beginning: Some(100.0), ending: Some(120.5) };
    paper.form.shareholder_shares =
        crate::K1ShareCountPair { beginning: Some(10.0), ending: Some(15.0625) };
    (page, remember_scorp_document(paper, personal))
}

pub(in crate::tests) fn set_scorp_credits(
    document: &mut crate::K1ScorpDocument,
    credits: &[crate::K1Credit],
) {
    use crate::partnership_source::credit_code_name;
    let mut personal = scorp_fixture_source(document).personal;
    document.form.credits = credits
        .iter()
        .map(|c| crate::K1CodeRow {
            code: credit_code_name(&c.code).into(),
            amount: Some(c.amount),
            text: alloc::string::String::new(),
            statement: false,
        })
        .collect();
    document.statements.credits =
        credits.iter().map(crate::PartnershipCreditStatement::from).collect();
    personal.research_credit_line = credits
        .iter()
        .find_map(|c| (c.code == crate::K1CreditCode::Research).then_some(c.line).flatten());
    *document = remember_scorp_document(document.clone(), personal);
}

// The shared lifecycle fixtures explicitly describe known synthetic paper facts.
// Preserve paper and topic records, supplying household identity only to unnamed fixtures.
pub(in crate::tests) fn known_fact_scorp_draft(mut draft: FederalDraft) -> FederalDraft {
    for index in 0..draft.income.len() {
        let entry = &draft.income[index];
        let FederalIncome::Scorp { data } = &entry.income else { continue };
        if !data.paper.shareholder.name.is_empty() || data.assigned.is_known() {
            continue;
        }

        let names = entry.printed.clone().or_else(|| {
            crate::tests::support::builders::fixture_document_names(
                &draft,
                if entry.owner == crate::DocumentOwner::Spouses { -2 } else { -1 },
            )
        });
        let entry = &mut draft.income[index];
        let FederalIncome::Scorp { data } = &mut entry.income else { unreachable!() };
        if let Some(names) = names {
            data.paper.shareholder = names.recipient;
            crate::document_names::Assignment {
                owner: names.assigned,
                owner_source: names.owner_source,
                joint_share: names.joint_share,
            }
            .apply_to(
                &mut data.assigned,
                &mut data.owner_source,
                &mut data.joint_share,
            );
        }
        entry.printed = Some(data.printed_names());
    }
    draft
}
