use crate::tests::*;

pub(in crate::tests) fn k1_distribution_counterparts(
    distribution: i64,
    gains: [i64; 2],
) -> [FederalIncome; 2] {
    let mut partner = ordinary_partner_source(0, true);
    partner.regular.basis.distributions = distribution;
    partner.regular.basis.opening = distribution - gains[0];
    let mut alternative = partner.regular.clone();
    alternative.basis.opening = distribution - gains[1];
    partner.alternative = Some(alternative);
    partner.interest_term = Some(GainTerm::Short);
    let mut scorp = complete_ordinary_scorp(0, true);
    scorp.loans.clear();
    scorp.qbi_statement = None;
    scorp.distributions = distribution;
    scorp.opening = distribution - gains[0];
    scorp.stock_term = Some(GainTerm::Short);
    let mut alternative = amt_records(&scorp);
    alternative.stock = distribution - gains[1];
    scorp.amt = Some(alternative);
    [FederalIncome::Partnership { data: partner }, FederalIncome::Scorp { data: scorp }]
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
