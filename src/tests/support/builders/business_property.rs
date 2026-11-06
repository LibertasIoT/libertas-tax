use crate::tests::*;

pub(in crate::tests) fn k1_capital(short: i64, long: i64) -> ScorpCapital {
    ScorpCapital {
        short: short * 100,
        long: long * 100,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        basis_short: 0,
        basis_long: 0,
        basis_collectibles: 0,
        risk_short: 0,
        risk_long: 0,
        risk_collectibles: 0,
    }
}

// Current K-1 capital and independently retained regular/AMT loss records.
pub(in crate::tests) fn legacy_k1_capital_sources(long: bool, basis: bool) -> [FederalIncome; 2] {
    let mut partner = ordinary_partner_source(0, true);
    partner.separate = Some(PartnerSeparateTotals {
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        short: if long { 0 } else { 100_000 },
        long: if long { 100_000 } else { 0 },
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        cash_charity: 0,
        backup_withholding: 0,
        exempt_interest: 0,
        foreign_taxes: 0,
    });
    for (record, amount) in core::iter::once(&mut partner.regular)
        .chain(partner.alternative.as_mut())
        .zip([40_000, 20_000])
    {
        let carry =
            if long { &mut record.capital_carried.long } else { &mut record.capital_carried.short };
        if basis {
            carry.basis = amount;
        } else {
            carry.risk = amount;
        }
    }
    let mut scorp = complete_ordinary_scorp(0, true);
    scorp.opening = Some(1_000_000);
    scorp.loans.clear();
    scorp.risk.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
    let mut capital = k1_capital(if long { 0 } else { 1000 }, if long { 1000 } else { 0 });
    match (long, basis) {
        (false, true) => capital.basis_short = 40_000,
        (true, true) => capital.basis_long = 40_000,
        (false, false) => capital.risk_short = 40_000,
        (true, false) => capital.risk_long = 40_000,
    }
    scorp.capital = Some(capital);
    let mut alternative = amt_records(&scorp);
    let capital = alternative.capital.as_mut().unwrap();
    match (long, basis) {
        (false, true) => capital.basis_short = 20_000,
        (true, true) => capital.basis_long = 20_000,
        (false, false) => capital.risk_short = 20_000,
        (true, false) => capital.risk_long = 20_000,
    }
    scorp.amt = Some(alternative);
    [
        FederalIncome::Partnership { data: partner },
        FederalIncome::Scorp { data: scorp_source(scorp) },
    ]
}

// A native component and a separate legacy K-1 share the annual capital owners.
pub(in crate::tests) fn mixed_native_legacy_capital_draft(
    k3: bool,
    legacy_first: bool,
) -> FederalDraft {
    use crate::tests::support::builders::k1res::configure_component_capital_positive_release;
    use crate::tests::support::client::k1res::component_capital_session;

    let (mut session, mut native) = component_capital_session(false, 100);
    configure_component_capital_positive_release(&mut native);
    let entry = session
        .draft
        .income
        .iter_mut()
        .find(|entry| matches!(entry.income, FederalIncome::PartnershipActivities { .. }))
        .unwrap();
    entry.income = native;
    let foreign = crate::tests::foreign::foreign::passive_fund_return(60_000);
    session.draft.credits = foreign.credits;
    for entry in foreign.income {
        add(&mut session.draft, entry);
    }
    let [_, mut legacy] = legacy_k1_capital_sources(false, true);
    let FederalIncome::Scorp { data } = &mut legacy else { unreachable!() };
    let mut data = edit_scorp(data);
    // Independent AMT source gain; its retained basis loss remains $200.
    data.amt.as_mut().unwrap().capital.as_mut().unwrap().short = 150_000;
    if k3 {
        let mut statement = crate::tests::foreign::foreign::k3_blank();
        statement.us_gross = 300_000;
        data.schedule_k3 = Some(statement);
    }
    drop(data);
    add(&mut session.draft, income(legacy));
    if legacy_first {
        session.draft.income.reverse();
    }
    session.draft
}

pub(in crate::tests) fn k1_collectible_capital(
    short: i64,
    long: i64,
    collectible: i64,
) -> ScorpCapital {
    ScorpCapital { collectibles: collectible * 100, ..k1_capital(short, long) }
}
pub(in crate::tests) fn collectible_investment_gain(regular: i64, amt: i64) -> FederalIncome {
    let mut item = investment_sale_gain(GainTerm::Long, regular, amt);
    let FederalIncome::Sale { data } = &mut item else { unreachable!() };
    data.form.collectible = true;
    item
}

pub(in crate::tests) fn k1_portfolio_1250(long: i64, partnership: i64, fund: i64) -> ScorpCapital {
    ScorpCapital {
        unrecaptured_partnership: partnership * 100,
        unrecaptured_fund: fund * 100,
        ..k1_capital(0, long)
    }
}

pub(in crate::tests) fn k1_section1231(net: i64, unrecaptured: i64) -> K1Section1231 {
    K1Section1231 {
        net: net * 100,
        unrecaptured: unrecaptured * 100,
        basis_loss: 0,
        risk_loss: 0,
        passive_loss: 0,
    }
}

pub(in crate::tests) fn passive_property_scorp(ordinary: i64, property: i64) -> ScorpK1 {
    let mut source = scorp_large_loss(0, 100_000, false);
    source.ordinary = ordinary * 100;
    source.section1231 = Some(k1_section1231(property, 0));
    source.amt = Some(amt_records(&source));
    source
}

pub(in crate::tests) fn passive_property_pool(source: ScorpK1, other: i64) -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    if other != 0 {
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: scorp_source(complete_ordinary_scorp(other, false)),
            }),
        );
    }
    (d, id)
}

pub(in crate::tests) fn former_passive_property_scorp(
    ordinary: i64,
    property: i64,
    prior_ordinary: i64,
    prior_property: i64,
) -> ScorpK1 {
    let mut source = passive_property_scorp(ordinary, property);
    source.participation.as_mut().unwrap().minutes = 30_001;
    source.passive.loss = prior_ordinary * 100;
    source.section1231.as_mut().unwrap().passive_loss = prior_property * 100;
    source.amt = Some(amt_records(&source));
    source
}

pub(in crate::tests) fn section1231_history(
    regular: &[(u32, i64, i64)],
    amt: &[(u32, i64, i64)],
) -> Section1231History {
    let records = |values: &[(u32, i64, i64)]| {
        values
            .iter()
            .map(|&(year, original, remaining)| Section1231LossYear {
                year,
                original: original * 100,
                remaining: remaining * 100,
            })
            .collect()
    };
    Section1231History { regular: records(regular), amt: records(amt) }
}

pub(in crate::tests) fn lookback_worksheet(
    d: &FederalDraft,
    amt: bool,
) -> Result<Option<crate::section1231::Result1231>, &'static str> {
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(d, 200_000, &mut r, amt);
    crate::section1231::worksheet(d, &pool, amt).map_err(|error| error.detail.1)
}

pub(in crate::tests) fn property_qbi(current: i64, gain: i64) -> ScorpPropertyQbi {
    ScorpPropertyQbi {
        qualified_1250: None,
        gain: gain * 100,
        losses: ScorpLossQbi {
            current: current * 100,
            history: vec![],
            at_risk: vec![],
            passive: vec![],
        },
        complete: true,
    }
}

pub(in crate::tests) fn property_qbi_character_draft(gain: i64, loss: i64) -> FederalDraft {
    let mut source = former_passive_property_scorp(0, gain, 0, 0);
    source.property_qbi = property_qbi(0, gain / 2);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    let (mut d, _) = passive_property_pool(source, 0);
    let mut source = former_passive_property_scorp(0, -loss, 0, 0);
    source.property_qbi = property_qbi(loss, 0);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    d
}

pub(in crate::tests) fn property_qbi_character_items(
    d: &FederalDraft,
) -> Result<alloc::vec::Vec<crate::section1231::PropertyQbiItem>, &'static str> {
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(d, 200_000, &mut r, false);
    crate::section1231::property_qbi(d, &pool).map_err(|error| error.detail.1)
}

pub(in crate::tests) fn combined_property_sstb_source() -> ScorpK1 {
    let mut source = former_passive_property_scorp(-10_000, 30_000, 0, 4_000);
    source.loss_qbi = sole_qbi_loss(10_000, 100_000, 10_000).loss_qbi;
    let mut facts = property_qbi(0, 30_000);
    facts.losses.passive = vec![loss_year(2022, 4_000, 2_000, 4_000, 2_000)];
    source.property_qbi = facts;
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 2_000_000,
        property: 10_000_000,
        service: true,
        complete: true,
    });
    source
}

pub(in crate::tests) fn nol_capital(
    return_capital: i64,
    nonbusiness_capital: i64,
    business_capital: i64,
    section1202_excluded: i64,
    schedule_capital_net: i64,
) -> crate::business_rules::NolCapital {
    crate::business_rules::NolCapital {
        return_capital,
        nonbusiness_capital,
        business_capital,
        section1202_excluded,
        schedule_capital_net,
    }
}

pub(in crate::tests) fn property_niit_worksheet(
    d: &FederalDraft,
    capital_net: i64,
    capital_income: i64,
) -> Result<crate::section1231::PropertyNiit, &'static str> {
    let mut result = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(d, 200_000, &mut result, false);
    let character =
        crate::section1231::worksheet(d, &pool, false).map_err(|error| error.detail.1)?.unwrap();
    crate::section1231::property_niit(d, &pool, &character, capital_net, capital_income, 0, 0)
        .map_err(|error| error.detail.1)
}

pub(in crate::tests) fn complete_property_gain_draft(
    material: bool,
    lookback: i64,
) -> FederalDraft {
    let mut source = complete_ordinary_scorp(0, material);
    source.section1231 = Some(k1_section1231(20000, 0));
    source.property_qbi = property_qbi(0, 20000);
    source.amt = Some(amt_records(&source));
    let (mut draft, _) = passive_property_pool(source, 0);
    let history = if lookback == 0 { vec![] } else { vec![(2025, lookback, lookback)] };
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&history, &history));
    draft
}

pub(in crate::tests) fn niit_loss_history(difference: Option<i64>) -> PriorCapitalReturn {
    let mut history = capital_history();
    history.investment_parts = None;
    history.niit_difference = difference.map(|v| v * 100);
    history
}

pub(in crate::tests) fn complete_trading_property_interest(
    ordinary: i64,
    property: i64,
) -> FederalDraft {
    let mut source = passive_property_scorp(ordinary, property);
    source.trade_kind = ScorpTradeKind::FinancialTrading;
    source.property_qbi = property_qbi((-property).max(0), property.max(0));
    source.qbi_statement = complete_ordinary_scorp(ordinary, false).qbi_statement;
    source.qbi_statement.as_mut().unwrap().service = true;
    source.amt = Some(amt_records(&source));
    let (mut draft, _) = passive_property_pool(source, 0);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    investment_borrowing(&mut draft, 25000, 0, 0);
    draft
}

pub(in crate::tests) fn property_excess_draft(
    loss: i64,
    amt_loss: i64,
    capacity: i64,
    qualified: i64,
    status: FederalStatus,
) -> FederalDraft {
    let mut source = scorp_large_loss(0, capacity, true);
    source.section1231 = Some(k1_section1231(-loss, 0));
    source.property_qbi = property_qbi(qualified, 0);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    source.amt = Some(amt_records(&source));
    source.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = -amt_loss * 100;
    let mut draft = complete(status);
    add(&mut draft, income(FederalIncome::Scorp { data: scorp_source(source) }));
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    draft
}

pub(in crate::tests) fn historical_property_excess_draft() -> FederalDraft {
    let mut draft = property_excess_draft(200000, 150000, 500000, 200000, FederalStatus::Single);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    let property = data.section1231.as_mut().unwrap();
    property.basis_loss = 10000000;
    property.risk_loss = 10000000;
    let facts = &mut data.property_qbi;
    facts.losses.history = vec![loss_year(2017, 100000, 0, 100000, 0)];
    facts.losses.at_risk = vec![loss_year(2020, 100000, 50000, 100000, 50000)];
    let amt = data.amt.as_mut().unwrap().section1231.as_mut().unwrap();
    amt.basis_loss = 5000000;
    amt.risk_loss = 10000000;
    drop(data);
    draft
}

pub(in crate::tests) fn operating_loss_property_gain_draft(
    qualified: i64,
    recapture: i64,
    qualified_gain: i64,
) -> FederalDraft {
    let mut source = sole_qbi_loss(400000, 500000, qualified);
    source.section1231 = Some(k1_section1231(100000, 0));
    source.property_qbi = property_qbi(0, qualified_gain);
    source.amt = Some(amt_records(&source));
    let amt = source.amt.as_mut().unwrap();
    amt.adjustment = 10000000; // AMT operating loss $300k versus regular $400k.
    amt.section1231.as_mut().unwrap().net = 5000000;
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Scorp { data: scorp_source(source) }));
    let prior = if recapture == 0 { vec![] } else { vec![(2025, recapture, recapture)] };
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&prior, &prior));
    draft
}

pub(in crate::tests) fn property_loss_operating_profit_draft(
    qualified_loss: i64,
    qualified_profit: i64,
) -> FederalDraft {
    let mut draft =
        property_excess_draft(400000, 300000, 500000, qualified_loss, FederalStatus::Single);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.ordinary = 10000000;
    data.qbi_statement.as_mut().unwrap().income = qualified_profit * 100;
    data.amt = Some(amt_records(&data));
    data.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = -30000000;
    drop(data);
    draft
}

pub(in crate::tests) fn multiple_fully_recaptured_gains(qualified_second: i64) -> FederalDraft {
    let mut draft = complete_property_gain_draft(false, 50000);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.qbi_statement.as_mut().unwrap().wages = 10000000;
        data.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 1000000;
    }
    let mut source = complete_ordinary_scorp(0, true);
    source.section1231 = Some(k1_section1231(30000, 0));
    source.property_qbi = property_qbi(0, qualified_second);
    source.qbi_statement.as_mut().unwrap().wages = 10000000;
    source.amt = Some(amt_records(&source));
    source.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 2000000;
    add(&mut draft, income(FederalIncome::Scorp { data: scorp_source(source) }));
    draft.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 50000, 50000)], &[(2025, 30000, 30000)]));
    draft
}

pub(in crate::tests) fn rate_ordered_recapture(regular: i64, amt: i64) -> FederalDraft {
    let mut draft = multiple_fully_recaptured_gains(30000);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.section1231.as_mut().unwrap().unrecaptured = 2000000;
        data.amt.as_mut().unwrap().section1231.as_mut().unwrap().unrecaptured = 1000000;
    }
    draft.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, regular, regular)], &[(2025, amt, amt)]));
    draft
}

pub(in crate::tests) fn qualified_rate_gain(
    qualified: i64,
    qualified_high: Option<i64>,
    recapture: i64,
) -> FederalDraft {
    let mut draft = complete_property_gain_draft(true, recapture);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.section1231.as_mut().unwrap().unrecaptured = 1200000;
        let facts = &mut data.property_qbi;
        facts.gain = qualified * 100;
        facts.qualified_1250 = qualified_high.map(|amount| amount * 100);
        data.amt = Some(amt_records(&data));
    }
    draft
}

pub(in crate::tests) fn trading_with_unreleased_histories() -> FederalDraft {
    let mut draft = complete_trading_property_interest(10000, -20000);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.passive.loss = 300000;
        data.loss_qbi = ScorpLossQbi {
            current: 0,
            history: vec![],
            at_risk: vec![],
            passive: vec![loss_year(2020, 3000, 3000, 3000, 3000)],
        };
        data.section1231.as_mut().unwrap().passive_loss = 700000;
        data.property_qbi.losses.passive = vec![loss_year(2020, 7000, 7000, 7000, 7000)];
        data.amt = Some(amt_records(&data));
        let property = data.amt.as_mut().unwrap().section1231.as_mut().unwrap();
        property.net = -3000000;
        property.passive_loss = 500000;
    }
    draft
}
