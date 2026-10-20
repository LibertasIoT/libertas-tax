use crate::tests::*;

pub(in crate::tests) fn scorp_document() -> ScorpK1 {
    ScorpK1 {
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
        rental_facts: Default::default(),
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
        opening: 1_000_000,
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
        disposition: crate::K1Disposition::Kept,
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
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    if bank_interest > 0 {
        add(
            &mut d,
            income(FederalIncome::Interest { data: native_interest(bank_interest * 100, 0) }),
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
    source.opening = capacity * 100;
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
        stock: source.opening,
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
        data: FederalSale {
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
        },
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
    source.opening = 0;
    source.loans.clear();
    source.section179 = 100_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    source.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    source
}
