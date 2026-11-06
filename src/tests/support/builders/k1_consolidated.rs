use crate::tests::*;

pub(in crate::tests) fn k1_distribution_counterparts(
    distribution: i64,
    gains: [i64; 2],
) -> [FederalIncome; 2] {
    let mut partner = ordinary_partner_source(0, true);
    partner.regular.basis.distributions = distribution;
    partner.regular.basis.opening = Some(distribution - gains[0]);
    let mut alternative = partner.regular.clone();
    alternative.basis.opening = Some(distribution - gains[1]);
    partner.alternative = Some(alternative);
    partner.interest_term = Some(GainTerm::Short);
    let mut scorp = complete_ordinary_scorp(0, true);
    scorp.loans.clear();
    scorp.qbi_statement = None;
    scorp.distributions = distribution;
    scorp.opening = Some(distribution - gains[0]);
    scorp.stock_term = Some(GainTerm::Short);
    let mut alternative = amt_records(&scorp);
    alternative.stock = distribution - gains[1];
    scorp.amt = Some(alternative);
    [
        FederalIncome::Partnership { data: partner },
        FederalIncome::Scorp { data: scorp_source(scorp) },
    ]
}

// Raw source cents: rounding belongs to the native owner, never this builder.
pub(in crate::tests) fn k1_reported_sale(
    description: &str,
    acquired: u32,
    [price, cost, depreciation, section179]: [i64; 4],
) -> K1Section179Sale {
    let amounts = crate::K1Section179SaleAmounts { price, cost, depreciation, section179 };
    K1Section179Sale {
        description: description.into(),
        acquired,
        sold: 20260630,
        regular: Some(amounts.clone()),
        alternative: crate::K1Section179SaleAmtSource::Reported { amounts },
        id: 0,
    }
}

pub(in crate::tests) fn k1_qbi_statement(source: &mut FederalIncome, income: i64, service: bool) {
    match source {
        FederalIncome::Partnership { data } => {
            let mut statement = partner_qbi_statement(income);
            statement.service = service;
            data.qbi = Some(statement);
        }
        FederalIncome::Scorp { data } => {
            let mut data = edit_scorp(data);
            data.qbi_statement = Some(ScorpQbiStatement {
                income: income * 100,
                wages: 0,
                property: 0,
                service,
                complete: true,
                reit_dividends: 0,
                ptp_income: 0,
            });
        }
        _ => unreachable!(),
    }
}

// Exact source cents; each tax system supplies its own basis and at-risk records.
pub(in crate::tests) fn k1_section1256_sources(
    rows: &[i64],
    basis: [i64; 2],
    risk: [i64; 2],
) -> [FederalIncome; 2] {
    let start = |capacity| ScorpRiskStart::Detailed { capacity, profit: 0, loss: 0 };
    let mut partner = ordinary_partner_source(0, true);
    partner.income = rows
        .iter()
        .map(|amount| PartnerIncomeTotal { code: PartnerIncomeCode::Section1256, amount: *amount })
        .collect();
    partner.regular.basis.opening = Some(basis[0]);
    partner.regular.risk.start = start(risk[0]);
    let mut alternative = partner.regular.clone();
    alternative.basis.opening = Some(basis[1]);
    alternative.risk.start = start(risk[1]);
    partner.alternative = Some(alternative);
    let mut scorp = complete_ordinary_scorp(0, true);
    scorp.opening = Some(basis[0]);
    scorp.loans.clear();
    scorp.risk.start = start(risk[0]);
    scorp.income = rows
        .iter()
        .map(|amount| ScorpIncome { code: ScorpIncomeCode::Section1256, amount: *amount })
        .collect();
    let mut alternative = amt_records(&scorp);
    alternative.stock = basis[1];
    alternative.risk.start = start(risk[1]);
    scorp.amt = Some(alternative);
    [
        FederalIncome::Partnership { data: partner },
        FederalIncome::Scorp { data: scorp_source(scorp) },
    ]
}

pub(in crate::tests) fn k1_page_credit_calls(
    credits: &[crate::K1CreditPage],
) -> (crate::K1PartnershipCalls, crate::K1ScorpCalls) {
    let boxes = credits.iter().map(crate::K1CreditCallBox::from).collect::<Vec<_>>();
    (crate::K1PartnershipCalls::from_credits(&boxes), crate::K1ScorpCalls::from_credits(&boxes))
}
