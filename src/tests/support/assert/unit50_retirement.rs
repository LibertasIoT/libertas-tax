use super::unit50_fuel::assert_fuel_payment;
use crate::tests::*;

// Inputs: native distribution and withholding cents, then prior/year-end basis dollars.
// Expected basis rows: recovered, taxable and carryforward dollars. The optional complete
// return retains AGI dollars followed by raw Form 1040 cents for 15, 16, 24a, 25a, 25b,
// 25d, 33, 34 and 37. A small IRA-basis boundary never replaces a complete-return fixture.
pub(in crate::tests) fn assert_ira_basis_case(
    case: &str,
    [gross, withholding, prior, year_end]: [i64; 4],
    basis: [i64; 3],
    taxable_cents: i64,
    complete_return: Option<[i64; 10]>,
) -> FederalResult {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: FederalRetirement {
                withholding,
                ..native_retirement(RetirementKind::Ira, gross, RetirementTaxable::IraBasis)
            },
        }),
    );
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, prior, year_end)];
    let r = supported(&d);
    let [recovered, taxable, carryforward] = basis;
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), recovered, "{case}");
    assert_eq!(value(&r, "taxpayer_ira_basis_taxable"), taxable, "{case}");
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), carryforward, "{case}");
    assert_eq!(return_line(&r, "4b"), taxable_cents, "{case}: 4b");
    if let Some([agi, taxable, tax, total, wages, retirement, withheld, payments, refund, due]) =
        complete_return
    {
        assert_eq!(return_line(&r, "4a"), gross, "{case}: 4a");
        assert_eq!(value(&r, "agi"), agi, "{case}: agi");
        assert_eq!(return_line(&r, "15"), taxable, "{case}");
        assert_eq!(return_line(&r, "16"), tax, "{case}");
        assert_eq!(return_line(&r, "24a"), total, "{case}");
        assert_eq!(return_line(&r, "25a"), wages, "{case}");
        assert_eq!(return_line(&r, "25b"), retirement, "{case}");
        assert_eq!(return_line(&r, "25d"), withheld, "{case}");
        assert_eq!(return_line(&r, "33"), payments, "{case}");
        assert_eq!(return_line(&r, "34"), refund, "{case}");
        assert_eq!(return_line(&r, "37"), due, "{case}");
        assert_eq!(restart(d.clone()).draft, d, "{case}: restart");
    }
    r
}

// The four native Form 8606/5329 inputs differ only in compensation, work coverage,
// retained contribution/election and the Saver question. Preserve the original wage
// source: complete() when absent, covered_wage() only for the covered case.
pub(in crate::tests) fn assert_retained_ira_excess_case(
    (wages, contribution, election, year_end, covered, saver_question): (
        Option<i64>,
        i64,
        i64,
        i64,
        Answer,
        bool,
    ),
    expected: [i64; 3],
    agi: Option<i64>,
) {
    let mut d = complete(FederalStatus::Single);
    if let Some(wages) = wages {
        d.income[0] = if covered == Answer::Yes {
            covered_wage(1, Owner::Taxpayer, wages, 0)
        } else {
            wage(1, Owner::Taxpayer, wages, 0)
        };
    }
    if saver_question {
        d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    }
    let mut f = ira_facts(contribution, 0, covered);
    f.taxpayer.nondeductible_election = election;
    f.taxpayer.excess =
        Some(super::super::builders::unit50_retirement::ira_excess(year_end, 0, 0, 0));
    d.adjustments.as_mut().unwrap().iras = f;
    let r = supported(&d);
    let [deduction, nondeductible, excess] = expected;
    assert_eq!(value(&r, "ira_deduction"), deduction);
    assert_eq!(value(&r, "taxpayer_nondeductible_ira"), nondeductible);
    assert_eq!(value(&r, "ira_excess_taxpayer_line15_traditional_excess"), excess);
    if let Some(agi) = agi {
        assert_eq!(value(&r, "agi"), agi);
    }
}

pub(in crate::tests) fn assert_aviation_sources(
    fuels: FederalFuelLines,
    expected: [(&str, i64); 2],
    payment: i64,
) -> FederalDraft {
    assert_eq!(
        fuels.labeled_lines().filter(|(_, v)| *v != 0).collect::<Vec<_>>(),
        expected.to_vec()
    );
    let (draft, baseline) = fuel_credit_sources(cash_business(10_000), fuels.clone(), fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, payment);
    for source in ["business_2", "partnership_3"] {
        for (form_line, amount) in expected {
            assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")), amount);
        }
    }

    draft
}

// Two- and three-year histories share the refusal scaffold, but retain their original
// bad-row positions, amount boundary, duplicate-year pairs and missing-row control.
pub(in crate::tests) fn assert_multi_year_refund_guards(
    history: TaxRefundHistory,
    amount: i64,
    three_years: bool,
) {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::TaxRefund { amount, withholding: 0, history }));
    for case in 0..3 {
        let mut bad = d.clone();
        let FederalIncome::TaxRefund {
            amount,
            history: TaxRefundHistory::IncomeTaxMultiplePriorYears { years, allocation_complete },
            ..
        } = &mut bad.income[1].income
        else {
            unreachable!()
        };
        match case {
            0 => *allocation_complete = false,
            1 if three_years => *amount = 1_000_001,
            1 => {
                *amount = years.iter().map(|year| year.payment_for_refunded_return).sum::<i64>() + 1
            }
            2 => years[if three_years { 2 } else { 1 }].ordinary = false,
            _ => unreachable!(),
        }
        refund_not_taxed_silently(&bad);
    }
    for case in 0..4 {
        let mut bad = d.clone();
        let FederalIncome::TaxRefund {
            history: TaxRefundHistory::IncomeTaxMultiplePriorYears { years, .. },
            ..
        } = &mut bad.income[1].income
        else {
            unreachable!()
        };
        match (three_years, case) {
            (false, 0) => years[1].payment_year = years[0].payment_year,
            (false, 1) => years[0].payment_for_refunded_return = years[0].income_paid + 1,
            (false, 2) => years[0].payment_year = 2026,
            (false, 3) => {
                years.pop();
            }
            (true, 0) => years[2].payment_year = years[1].payment_year,
            (true, 1) => years[2].payment_year = years[0].payment_year,
            (true, 2) => years[2].payment_year = 2026,
            (true, 3) => years[2].payment_for_refunded_return = years[2].income_paid + 1,
            _ => unreachable!(),
        }
        assert!(!bad.valid_stored(), "three years={three_years}, case {case}");
    }
    add(
        &mut d,
        income(FederalIncome::TaxRefund { amount: 100, withholding: 0, history: refund_history() }),
    );
    unsupported(&d, "Income:");
}
