use super::*;
#[test]
fn debt_repayment_at_risk_uses_its_independent_amt_basis_recovery() {
    let mut source = scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount);
    let amt = source.amt.as_mut().unwrap();
    amt.loans[0].basis = 800_000;
    amt.risk.start = ScorpRiskStart::Detailed { capacity: 800_000, profit: 0, loss: 0 };
    let alternative = source.amt.as_ref().unwrap().source(&source).unwrap();
    let basis = alternative.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].ordinary_gain.get()), 1_000);
    let risk = alternative.risk.calculate(&alternative, &basis).unwrap();
    assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), Some(4_000));
    assert_eq!(alternative.activity_result().unwrap().nonpassive, 0);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    let r = supported(&d);
    assert_eq!(value(&r, "scorp_amt_loan_gain_adjustment"), -1_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 61_000);
}

#[test]
fn repayment_retirement_basis_return_does_not_reduce_schedule_r() {
    let mut d = repayment_senior_draft(2023);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].benefits = 1200000;
    years[0].standard = 1570000;
    years[0].income_tax = 63300;
    years[0].retirement = vec![RepaymentRetirement::Pension {
        gross: 1500000,
        taxable: 1000000,
        normal: true,
        treatment: RepaymentDistribution::Normal,
    }];
    years[0].elderly = RepaymentElderly::Facts { exempt_pensions: vec![] };
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_pension_gross"), 15000);
    assert_eq!(value(&r, "repayment_year_0_pension_taxable"), 10000);
    assert_eq!(value(&r, "repayment_year_0_refigured_elderly_credit"), 38);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 93);
    assert_eq!(value(&r, "repayment_credit"), 540);
}

#[test]
fn repayment_amt_sources_accept_separate_sale_basis_without_amt_payable() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let mut sale = historical_sale(GainTerm::Short, 20000, 10000);
    sale.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: 1500000, wash_sale: 0 },
    };
    years[0].capital = historical_capital(vec![sale], 0, 0);
    years[0].income_tax = 1015500;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_capital_income"), 10000);
    assert_eq!(value(&r, "repayment_year_0_original_amt_capital_income"), 5000);
    assert_eq!(value(&r, "repayment_year_0_original_amti"), 80000);
    assert_eq!(value(&r, "repayment_year_0_refigured_amti"), 75000);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_exclusion_basis_reuses_sale_validation_and_requires_complete_worksheet() {
    for mode in [1, 2] {
        let mut d = repayment_exclusion_loss_chain();
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        let RepaymentCapital::Records { sales, exclusion, .. } = &mut years[0].capital else {
            panic!()
        };
        match mode {
            1 => sales[0].exclusion.as_mut().unwrap().wash_sale = 1500000,
            _ => *exclusion = None,
        }
        assert!(!d.valid_stored());
    }
}

#[test]
fn repayment_historical_railroad_basis_recomputes_social_and_credit() {
    for (year, standard, original, credit) in
        [(2023, 15700, 1199, 766), (2024, 16550, 1098, 750), (2025, 17750, 378, 378)]
    {
        let mut d = repayment_social_draft(year, standard, original);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            unreachable!()
        };
        years[0].wages = 0;
        years[0].retirement = vec![RepaymentRetirement::Railroad {
            owner: Owner::Taxpayer,
            gross: 2000000,
            contributory: 1400000,
            basis: Some(PensionBasis {
                earlier_months: 0,
                changes: vec![],
                share: None,
                started: 20000101,
                cost: Some(10000000),
                months: 10,
                history: PensionHistory::Prior { monthly: 50000, recovered: 9000000 },
                eligible: true,
            }),
            normal: true,
        }];
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_pension_gross"), 20000);
        assert_eq!(value(&r, "repayment_year_0_pension_taxable"), 15000);
        assert_eq!(value(&r, "repayment_year_0_ira_taxable"), 0);
        assert_eq!(value(&r, "repayment_year_0_original_agi"), 27500);
        assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 20000);
        assert_eq!(value(&r, "repayment_credit"), credit);
        assert_eq!(supported(&restart(d).draft), r);
    }
}
