// Merge review 2 (2026-10-10) seam defects, F15: the fund section 1202 exclusion and the code Z
// deferral leave Form 1116 line 3e, a deferral takes its own community share, a code B section
// 1231 inclusion joins the rental MAGI, the Form 8949 gross and net sale gains take the community
// share, an AMT basis needs the regular basis, the Y/O pair does not loosen the code Z cap, and
// the Form 8958 total of a fund's section 1202 gain uses the separate-return issuer limit.
use super::*;
use crate::tests::foreign::foreign::fund_dividends;
use alloc::string::String;

fn passive_claim(income: i64, tax: i64) -> ForeignTaxClaim {
    ForeignTaxClaim::Ordinary {
        basis: ForeignTaxBasis::Paid,
        sources: vec![ForeignOrdinaryTax {
            label: "Synthetic GM".into(),
            country: "GM".into(),
            owner: Owner::Taxpayer,
            category: ForeignTaxCategory::Passive {
                withholding: ForeignWithholding::AtLeast15,
                qualified: 0,
                sales: vec![],
            },
            income: income * 100,
            tax: tax * 100,
        }],
        situations: vec![],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::PriorReturn { categories: vec![], confirmed: true },
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        rate_adjustment: ForeignRateAdjustment::Exception,
    }
}

fn dividend_with_1202(
    distributions: i64,
    gain: i64,
    exclusion: Option<FundQsbsExclusion>,
) -> FederalIncome {
    let mut data = crate::interview::dividend_record(&crate::Div1099 {
        more: crate::DivMore {
            capital_distributions: distributions,
            section_1202_gain: gain,
            section_1202_exclusion: exclusion,
            ..Default::default()
        },
        ..native_div1099(String::new(), 0, 0)
    });
    data.section_1202_exclusion = exclusion;
    FederalIncome::Dividend { data }
}

fn amount(r: &FederalResult, id: &str) -> Option<i64> {
    r.lines.iter().find(|l| l.id == id).map(|l| l.amount)
}

fn print_3e(r: &FederalResult) -> String {
    let mut out = String::new();
    for l in &r.lines {
        if l.id.ends_with("gross_income_all_sources")
            || l.id == "agi"
            || l.id == "capital_gain"
            || l.id == "foreign_tax_credit"
            || l.id.ends_with("_gross_income_ratio")
            || l.id.ends_with("_apportioned_deductions")
        {
            out.push_str(&alloc::format!("{} = {}; ", l.id, l.amount));
        }
    }
    out
}

// P1: a fund's section 1202 exclusion (box 2c, 100 percent) is not gross income on line 3e.
#[test]
fn p1_fund_section_1202_exclusion_stays_out_of_line_3e() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 12_000_000, 0));
    add(&mut d, income(fund_dividends(2_000, 0, 0)));
    add(
        &mut d,
        income(dividend_with_1202(10_000_000, 4_000_000, Some(FundQsbsExclusion::Hundred))),
    );
    d.credits.as_mut().unwrap().foreign_tax = passive_claim(2_000, 300);
    let r = supported(&d);
    let diag = print_3e(&r);
    assert_eq!(value(&r, "agi"), 182_000, "{diag}");
    assert_eq!(value(&r, "capital_gain"), 60_000, "{diag}");
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 182_000, "{diag}");
}

// P2: a gain deferred by a 2027 fund investment (code Z) is not gross income on line 3e.
#[test]
fn p2_qof_deferral_stays_out_of_line_3e() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(1, Owner::Taxpayer, 12_000_000, 0));
    add(&mut d, income(fund_dividends(2_000, 0, 0)));
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: FederalSale {
                basis_reporting: BasisReporting::NoForm1099B,
                ..native_sale(GainTerm::Long, 10_000_000, Some(2_000_000))
            },
        }),
    );
    add(
        &mut d,
        income(FederalIncome::OpportunityFundDeferral {
            data: QofDeferral {
                fund: "Example Opportunity Fund".into(),
                invested_on: 20270215,
                short_term: 0,
                long_term: 6_000_000,
                within_180_days: true,
                special: false,
            },
        }),
    );
    d.credits.as_mut().unwrap().foreign_tax = passive_claim(2_000, 300);
    let r = supported(&d);
    let diag = print_3e(&r);
    assert_eq!(value(&r, "agi"), 142_000, "{diag}");
    assert_eq!(value(&r, "capital_gain"), 20_000, "{diag}");
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 142_000, "{diag}");
}

// P3: a code Z deferral on a community-property separate return takes the row's share.
#[test]
fn p3_qof_deferral_takes_the_community_share() {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = community_arizona();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.income[0].community = CommunityShare::Community;
    let mut sale = income(FederalIncome::Sale {
        data: FederalSale {
            basis_reporting: BasisReporting::NoForm1099B,
            ..native_sale(GainTerm::Long, 3_000_000, Some(2_000_000))
        },
    });
    sale.community = CommunityShare::Community;
    add(&mut d, sale);
    let mut deferral = income(FederalIncome::OpportunityFundDeferral {
        data: QofDeferral {
            fund: "Example Opportunity Fund".into(),
            invested_on: 20270215,
            short_term: 0,
            long_term: 1_000_000,
            within_180_days: true,
            special: false,
        },
    });
    deferral.community = CommunityShare::Community;
    add(&mut d, deferral);
    let r = supported(&d);
    let mut diag = String::new();
    for l in &r.lines {
        if l.id == "agi"
            || l.id == "capital_gain"
            || l.id.starts_with("form8958_")
            || l.id.starts_with("qof_deferral")
            || l.id.starts_with("capital_gain_")
        {
            diag.push_str(&alloc::format!("{} = {}; ", l.id, l.amount));
        }
    }
    assert_eq!(value(&r, "capital_gain"), 0, "{diag}");
    assert_eq!(value(&r, "agi"), 15_000, "{diag}");
}

// P4: a sale on a community-property return and the two-thirds farmer gross income.
#[test]
fn p4_sale_gain_gross_on_a_community_return() {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = community_arizona();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.income[0].community = CommunityShare::Community;
    let mut gain = income(FederalIncome::Sale {
        data: FederalSale {
            basis_reporting: BasisReporting::NoForm1099B,
            ..native_sale(GainTerm::Long, 3_000_000, Some(2_000_000))
        },
    });
    gain.community = CommunityShare::Community;
    add(&mut d, gain);
    let mut loss = income(FederalIncome::Sale {
        data: FederalSale {
            basis_reporting: BasisReporting::NoForm1099B,
            ..native_sale(GainTerm::Long, 1_000_000, Some(1_400_000))
        },
    });
    loss.community = CommunityShare::Community;
    add(&mut d, loss);
    let r = supported(&d);
    let mut diag = String::new();
    for l in &r.lines {
        if l.id == "agi"
            || l.id == "capital_gain"
            || l.id.contains("two_thirds")
            || l.id.contains("gross_income")
            || l.id.starts_with("capital_gain_")
        {
            diag.push_str(&alloc::format!("{} = {}; ", l.id, l.amount));
        }
    }
    assert_eq!(value(&r, "capital_gain"), 3_000, "{diag}");
}

fn qof_property(long: i64) -> FederalIncome {
    FederalIncome::OpportunityFund {
        data: QofInvestment {
            fund: "Example Opportunity Fund".into(),
            invested_on: 20190601,
            short_term: 0,
            long_term: long,
            gain_code: QofGainCode::Section1231,
            event: QofEvent::Held { fair_value: Some(long) },
            special: false,
        },
    }
}

// S2: a code B section 1231 inclusion is in the Form 8582 line 7 MAGI (Single, W-2 100,000, a
// rental with a 31,000 loss, a 20,000 fund gain held at its value).
#[test]
fn s2_qof_section_1231_inclusion_joins_rental_magi() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 10_000_000, 0)];
    d.screening.as_mut().unwrap().section1231 =
        Some(crate::Section1231History { regular: vec![], amt: vec![] });
    add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
    add(&mut d, income(qof_property(2_000_000)));
    let r = supported(&d);
    let mut diag = String::new();
    for l in &r.lines {
        if l.id == "agi"
            || l.id.starts_with("rental_")
            || l.id.starts_with("qof_")
            || l.id == "capital_gain"
        {
            diag.push_str(&alloc::format!("{} = {}; ", l.id, l.amount));
        }
    }
    // MAGI 100,000 + 17,000 (the inclusion after the 15 percent basis increase): the allowance is
    // 25,000 - 50% x 17,000 = 16,500.
    assert_eq!(value(&r, "rental_magi"), 117_000, "{diag}");
    assert_eq!(value(&r, "rental_income"), -16_500, "{diag}");
    assert_eq!(value(&r, "agi"), 100_500, "{diag}");
}

// S3: the two-thirds farmer gross income on a community return (MFS Arizona Halves; a 50,000
// farm, a 40,000 gain lot and a 40,000 loss lot, all Community).
#[test]
fn s3_two_thirds_gross_income_takes_the_community_share() {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = community_arizona();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income = vec![];
    for v in [
        raised_farm(50_000, 0),
        ordinary_investment_sale(50_000, 10_000, GainTerm::Long),
        ordinary_investment_sale(10_000, 50_000, GainTerm::Long),
    ] {
        let mut e = income(v);
        e.community = CommunityShare::Community;
        add(&mut d, e);
    }
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year =
        Some(PriorYearTax { full_year: true, tax: 900_000, agi: Some(10_000_000), spouse_tax: 0 });
    pay.next_year = NextYearEstimateRecord::NotNeeded;
    let r = supported(&d);
    let mut diag = String::new();
    for l in &r.lines {
        if l.id.contains("gross_income")
            || l.id == "agi"
            || l.id == "capital_gain"
            || l.id.starts_with("form8958")
        {
            diag.push_str(&alloc::format!("{} = {}; ", l.id, l.amount));
        }
    }
    // The sales' gains hidden by netting are the filer's half: 20,000, not 40,000; the farm's
    // gross receipts stay whole as before (the farmer's own Schedule F).
    assert_eq!(value(&r, "capital_gain"), 0, "{diag}");
    assert_eq!(amount(&r, "farming_fishing_gross_income_return_year"), Some(5_000_000), "{diag}");
    assert_eq!(amount(&r, "gross_income_return_year"), Some(7_000_000), "{diag}");
}

// S4: an AMT basis without a regular basis is refused (decision 4 counts the regular basis as
// zero, which an entered AMT basis would contradict).
#[test]
fn s4_amt_basis_needs_a_regular_basis() {
    let mut sale = FederalSale {
        basis_reporting: BasisReporting::NoForm1099B,
        ..native_sale(GainTerm::Long, 10_000_000, None)
    };
    sale.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: 2_000_000, wash_sale: 0 },
    };
    assert_eq!(
        crate::federal_rules::validate_sale(&sale).unwrap_err().0,
        "SALE_AMT_BASIS_NEEDS_BASIS"
    );
}

// S5: the Y/O pair does not loosen the code Z cap: an 85,000 section 1231 inclusion is the only
// gain, so a 50,000 long-term deferral is above the year's Form 8949 gains.
#[test]
fn s5_code_z_cap_ignores_the_section_1231_y_row() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().section1231 =
        Some(crate::Section1231History { regular: vec![], amt: vec![] });
    add(&mut d, income(qof_property(10_000_000)));
    add(
        &mut d,
        income(FederalIncome::OpportunityFundDeferral {
            data: QofDeferral {
                fund: "Example Opportunity Fund".into(),
                invested_on: 20270215,
                short_term: 0,
                long_term: 5_000_000,
                within_180_days: true,
                special: false,
            },
        }),
    );
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_QOF_DEFERRAL_ABOVE_GAINS"),
        "{:?}",
        r.issues
    );
}

// S6: the Form 8958 total of a fund's section 1202 gain uses the separate-return issuer limit:
// box 2a 8,000,000 less min(6,000,000, 5,000,000) = 3,000,000, as the return itself takes it.
#[test]
fn s6_form_8958_total_uses_the_separate_issuer_limit() {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = community_arizona();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.income[0].community = CommunityShare::Community;
    let mut e =
        income(dividend_with_1202(800_000_000, 600_000_000, Some(FundQsbsExclusion::Hundred)));
    e.community = CommunityShare::Community;
    add(&mut d, e);
    let id = d.income.last().unwrap().id;
    let r = supported(&d);
    let mut diag = String::new();
    for l in &r.lines {
        if l.id == "agi"
            || l.id == "capital_gain"
            || l.id.starts_with("form8958")
            || l.id.starts_with("qsbs")
        {
            diag.push_str(&alloc::format!("{} = {}; ", l.id, l.amount));
        }
    }
    assert_eq!(value(&r, &alloc::format!("form8958_{id}_total")), 3_000_000, "{diag}");
}
