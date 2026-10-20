use crate::tests::*;

pub(in crate::tests) fn government_unemployment(
    gross: i64,
    repaid: &[i64],
    withholding: i64,
) -> FederalIncomeEntry {
    income(FederalIncome::Unemployment {
        data: FederalUnemployment {
            amount: gross * 100,
            withholding: withholding * 100,
            repayments: repaid
                .iter()
                .map(|v| UnemploymentRepayment { received_year: 2026, amount: v * 100 })
                .collect(),
            ordinary: true,
            taxable_grants: 0,
            agriculture_payment: 0,
            agriculture_farm: None,
            grant_use: None,
            rtaa: 0,
            market_gain: 0,
            family_leave: 0,
            family_leave_includes_medical: false,
        },
    })
}

pub(in crate::tests) fn prior_unemployment_repayment(
    amount: i64,
    owner: Owner,
) -> FederalIncomeEntry {
    let mut entry = government_unemployment(0, &[], 0);
    entry.owner = owner;
    let FederalIncome::Unemployment { data } = &mut entry.income else { panic!() };
    data.repayments = vec![UnemploymentRepayment { received_year: 2025, amount }];
    entry
}

pub(in crate::tests) fn repayment_comparison_draft(method: RepaymentMethod) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, prior_unemployment_repayment(500000, Owner::Taxpayer));
    d.deductions.as_mut().unwrap().repayment = RepaymentClaim::Claim {
        method,
        years: vec![RepaymentTaxYear {
            year: 2025,
            filers: RepaymentFilers::Taxpayer,
            status: FederalStatus::Single,
            wages: 6000000,
            interest: 0,
            benefits: 1500000,
            standard: 1575000,
            income_tax: 795500,
            exempt_interest: 0,
            private_activity_interest: 0,
            dividends: RepaymentDividends::None,
            capital: RepaymentCapital::None,
            social: RepaymentSocial::None,
            retirement: vec![],
            early: RepaymentEarly::None,
            separate_lived_apart: None,
            eic: RepaymentEic::NotSupplied,
            ssns: RepaymentSsns::NotSupplied,
            elderly: RepaymentElderly::NotSupplied,
            minimum_tax: None,
            capital_exclusions: None,
        }],
        confirmed: true,
    };
    d
}

pub(in crate::tests) fn repayment_eic_draft() -> FederalDraft {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    let y = &mut years[0];
    y.wages = 1000000;
    y.benefits = 1000000;
    y.income_tax = 42800;
    y.ssns = RepaymentSsns::Taxpayer;
    y.eic = RepaymentEic::Facts { home: true, independent: true, allowed: true, filed_credit: 0 };
    d
}

pub(in crate::tests) fn repayment_senior_draft(year: u32) -> FederalDraft {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19580615;
    let FederalIncome::Unemployment { data } = &mut d.income[1].income else { panic!() };
    data.repayments[0].received_year = year;
    let years = repayment_years_mut(&mut d);
    years[0].year = year;
    d
}

pub(in crate::tests) fn repayment_social_draft(year: u32, standard: i64, tax: i64) -> FederalDraft {
    let mut d = repayment_senior_draft(year);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 1500000;
    years[0].benefits = 1000000;
    years[0].standard = standard * 100;
    years[0].income_tax = tax * 100;
    years[0].ssns = RepaymentSsns::Taxpayer;
    years[0].social =
        RepaymentSocial::Benefits { amount: 1000000, filed_taxable: 250000, ordinary: true };
    d
}

pub(in crate::tests) fn repayment_early_draft(
    simple: bool,
    medical: bool,
    expenses: i64,
    tax: i64,
) -> FederalDraft {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    // Original AGI 75,000; revised AGI 70,000. Medical floors 5,625 and 5,250.
    years[0].wages = 5000000;
    years[0].retirement = vec![RepaymentRetirement::TraditionalIra {
        gross: 1000000,
        normal: true,
        treatment: RepaymentDistribution::Early { owner: Owner::Taxpayer, simple, medical },
    }];
    years[0].early = RepaymentEarly::Facts {
        medical_expenses: expenses * 100,
        filed_tax: tax * 100,
        confirmed: true,
    };
    d
}

pub(in crate::tests) fn historical_dividends(
    ordinary: i64,
    qualified: i64,
    gains: i64,
    unrecaptured: i64,
    collectibles: i64,
) -> RepaymentDividends {
    RepaymentDividends::Statements {
        ordinary: ordinary * 100,
        qualified: qualified * 100,
        gains: gains * 100,
        unrecaptured: unrecaptured * 100,
        collectibles: collectibles * 100,
        confirmed: true,
    }
}

pub(in crate::tests) fn historical_sale(term: GainTerm, proceeds: i64, basis: i64) -> FederalSale {
    FederalSale {
        digital_asset: false,
        market_discount: 0,
        several_sales: false,
        term,
        collectible: false,
        proceeds: proceeds * 100,
        basis: Some(basis * 100),
        amt: crate::SaleAmt::SameAsRegular,
        withholding: 0,
        use_kind: SaleUse::Investment { wash_sale: 0 },
        special: crate::SaleSpecial::NoneOfThese,
        adjustment: SaleAdjustment::None,
        basis_reporting: BasisReporting::ReportedToIrs,
    }
}
pub(in crate::tests) fn historical_capital(
    sales: alloc::vec::Vec<FederalSale>,
    short: i64,
    long: i64,
) -> RepaymentCapital {
    RepaymentCapital::Records {
        amt: None,
        exclusion: None,
        sales: sales.into_iter().map(|sale| RepaymentSale { sale, exclusion: None }).collect(),
        short_loss: short * 100,
        long_loss: long * 100,
        confirmed: true,
    }
}

pub(in crate::tests) fn repayment_carry_chain() -> FederalDraft {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let FederalIncome::Unemployment { data } = &mut d.income[1].income else { panic!() };
    data.repayments[0].received_year = 2024;
    let years = repayment_years_mut(&mut d);
    let mut later = years[0].clone();
    later.benefits = 0;
    later.income_tax = 751500;
    later.capital = historical_capital(vec![historical_sale(GainTerm::Short, 20000, 0)], 7000, 0);
    years[0].year = 2024;
    years[0].wages = 0;
    years[0].benefits = 2000000;
    years[0].standard = 1460000;
    years[0].income_tax = 24100;
    years[0].capital = historical_capital(vec![historical_sale(GainTerm::Short, 0, 10000)], 0, 0);
    years.push(later);
    d
}

pub(in crate::tests) fn repayment_restoration_draft(method: RepaymentMethod) -> FederalDraft {
    let mut d = repayment_comparison_draft(method);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].benefits = 2000000;
    years[0].income_tax = 12600;
    years[0].capital = historical_capital(vec![historical_sale(GainTerm::Short, 0, 10000)], 0, 0);
    let mut prior = capital_history();
    prior.agi = 1700000;
    prior.deductions = 1575000;
    prior.short = -1000000;
    prior.long = 0;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 700_000, long: 0, amt: None });
    d.screening.as_mut().unwrap().capital_history = vec![prior];
    d
}

pub(in crate::tests) fn repayment_minimum_source(year: u32) -> MinimumTaxHistory {
    let (regular, amt) = match year {
        2022 => (17836, 15430),
        2023 => (17400, 14462),
        2024 => (17053, 13665),
        _ => panic!(),
    };
    MinimumTaxHistory {
        return_form: crate::MinimumTaxReturnForm::Form1040,
        status: FederalStatus::Single,
        base: 10000000,
        exclusions: vec![MinimumTaxExclusion {
            source: alloc::format!("{year} private activity interest"),
            amount: 10000000,
        }],
        regular: regular * 100,
        amt: amt * 100,
        carry: 0,
        gains: None,
        regular_nol: 0,
        mtcnol: crate::MinimumTaxLoss::None,
        foreign_earned: None,
        foreign_credit: crate::MinimumTaxForeignCredit::None,
        old_vehicle_credit: 0,
    }
}

pub(in crate::tests) fn repayment_minimum_chain() -> FederalDraft {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    for e in &mut d.income {
        if let FederalIncome::Unemployment { data } = &mut e.income {
            data.repayments[0].received_year = 2024;
        }
    }
    let years = repayment_years_mut(&mut d);
    let mut source = repayment_minimum_source(2023);
    source.carry = 700000;
    years[0].year = 2024;
    years[0].standard = 1460000;
    years[0].income_tax = 34700;
    years[0].minimum_tax = Some(source);
    let mut next = years[0].clone();
    next.year = 2025;
    next.standard = 1575000;
    next.benefits = 0;
    next.income_tax = 507500;
    next.minimum_tax = None;
    years.push(next);
    d
}

pub(in crate::tests) fn repayment_deferral_capital() -> FederalDraft {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let mut sale = historical_sale(GainTerm::Short, 150000, 100000);
    sale.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: 5000000, wash_sale: 0 },
    };
    years[0].capital = historical_capital(vec![sale], 0, 0);
    years[0].capital_exclusions = Some(vec![]);
    years[0].income_tax = 2259400;
    d
}

pub(in crate::tests) fn repayment_exclusion_loss_chain() -> FederalDraft {
    let mut d = repayment_carry_chain();
    let FederalIncome::Unemployment { data } = &mut d.income[1].income else { panic!() };
    data.repayments[0].amount = 400000;
    let years = repayment_years_mut(&mut d);
    years[0].benefits = 400000;
    years[0].income_tax = 0;
    if let RepaymentCapital::Records { sales, exclusion, .. } = &mut years[0].capital {
        sales[0].sale.amt = crate::SaleAmt::Different {
            record: CapitalSaleAmt { term: None, basis: 1200000, wash_sale: 0 },
        };
        sales[0].exclusion = Some(RepaymentExclusionBasis { basis: 1400000, wash_sale: 0 });
        *exclusion = Some(RepaymentExclusionLosses { short: 0, long: 0 });
    }
    if let RepaymentCapital::Records { short_loss, amt, exclusion, .. } = &mut years[1].capital {
        *short_loss = 1000000;
        *amt = Some(RepaymentAmtLosses { short: 900000, long: 0 });
        *exclusion = Some(RepaymentExclusionLosses { short: 1100000, long: 0 });
    }
    years[1].exempt_interest = 10000000;
    years[1].private_activity_interest = 10000000;
    years[1].income_tax = 2155400;
    d
}

pub(in crate::tests) fn add_repayment_inferred_capital(draft: &mut FederalDraft) {
    // Reuse the complete original2025 loss worksheet from the restoration
    // fixture. Do not preload its current carry: the claim owner derives the
    // same7000 actual carry from those source records, without duplicate entry.
    let source = repayment_restoration_draft(RepaymentMethod::Credit);
    for entry in source.income {
        if matches!(entry.income, FederalIncome::Unemployment { .. }) {
            add(draft, entry);
        }
    }
    draft.deductions.as_mut().unwrap().repayment = source.deductions.unwrap().repayment;
}

pub(in crate::tests) fn loan_claim(forms: Vec<Form1098E>, no_form: i64) -> StudentLoanClaim {
    StudentLoanClaim { student_loan_interest: no_form, forms, employer_paid_interest: 0 }
}

pub(in crate::tests) fn box_2(interest: i64, additional_interest: i64) -> Form1098E {
    Form1098E { interest, box_2: Form1098EBox2::Checked { additional_interest } }
}

pub(in crate::tests) fn employer_paid(
    mut claim: StudentLoanClaim,
    amount: i64,
) -> StudentLoanClaim {
    claim.employer_paid_interest = amount;
    claim
}

pub(in crate::tests) fn box_1(interest: i64) -> Form1098E {
    Form1098E { interest, box_2: Form1098EBox2::NotChecked }
}

pub(in crate::tests) fn repayment_years_mut(d: &mut FederalDraft) -> &mut Vec<RepaymentTaxYear> {
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        panic!()
    };
    years
}
