use super::paper_documents::remember;
use crate::tests::*;

fn government_document_native(form: Form1099G) -> Form1099GPaper {
    let Form1099G {
        payer,
        recipient,
        amount,
        withholding,
        rtaa,
        taxable_grants,
        agriculture_payment,
        market_gain,
        family_leave,
        state_details,
    } = form;
    Form1099GPaper {
        payer,
        recipient,
        unemployment_compensation: amount,
        federal_income_tax_withheld: withholding,
        rtaa_payments: Some(rtaa),
        taxable_grants: Some(taxable_grants),
        agriculture_payments: Some(agriculture_payment),
        market_gain: Some(market_gain),
        family_leave_benefits: family_leave,
        state_details,
        ..Default::default()
    }
}

fn refund_paper_native(form: Refund1099G) -> Form1099GPaper {
    let Refund1099G { payer, recipient, amount, year, withholding, state_details } = form;
    Form1099GPaper {
        payer,
        recipient,
        state_or_local_income_tax_refunds: amount,
        box_2_tax_year: (year != 0).then_some(year),
        federal_income_tax_withheld: withholding,
        state_details: state_details.taxes,
        ..Default::default()
    }
}

paper_adapter!(
    government_document,
    government_document_native,
    GovernmentBoxFixture,
    Form1099G,
    Form1099GPaper
);

paper_adapter!(refund_paper, refund_paper_native, RefundBoxFixture, Refund1099G, Form1099GPaper);

pub(in crate::tests) fn tax_refund_source(
    data: crate::FederalTaxRefund,
) -> crate::GovernmentPaymentSource {
    let year = match &data.history {
        crate::TaxRefundHistory::IncomeTax { deduction_year, .. } => *deduction_year,
        _ => 2025,
    };
    let mut source = crate::GovernmentPaymentSource::printed(refund_paper(crate::Refund1099G {
        payer: Default::default(),
        recipient: Default::default(),
        amount: data.amount,
        withholding: data.withholding,
        year,
        state_details: Default::default(),
    }));
    source.history = data.history;
    source
}

pub(in crate::tests) fn refund_document(
    form: Refund1099G,
    deducted: RefundDeduction,
) -> Form1099GPaper {
    let form = refund_paper(form);
    remember(
        IncomeSource::GovernmentPayment {
            refund_state: None,
            refund_locality: Default::default(),
            value: form.clone(),
        },
        vec![crate::DocumentAnswer::RefundDeduction { value: Some(deducted) }],
    );
    form
}

pub(in crate::tests) fn mixed_government_paper() -> Form1099GPaper {
    Form1099GPaper {
        unemployment_compensation: 200_000,
        state_or_local_income_tax_refunds: 300_000,
        box_2_tax_year: Some(2025),
        federal_income_tax_withheld: 50_000,
        ..Default::default()
    }
}

pub(in crate::tests) fn government_refund_deduction() -> RefundDeduction {
    RefundDeduction::IncomeTax {
        income_paid: 1_000_000,
        salt_paid: 1_000_000,
        salt_deducted: 1_000_000,
        itemized: 1_675_000,
        standard: 1_575_000,
        sales_available: 500_000,
        property_refunded: 0,
    }
}

pub(in crate::tests) fn government_answer_request(
    session: &mut Interview,
    repayment: Option<i64>,
    deduction: Option<RefundDeduction>,
) -> P {
    let mut request = crate::tests::support::client::income_pages::open_topic(
        session,
        TaxSection::GovernmentBenefits,
    );
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    for row in followups {
        match &mut row.answer {
            DocumentAnswer::GovernmentRepayments { value } => {
                if let Some(amount) = repayment {
                    *value = vec![UnemploymentRepayment { received_year: 2026, amount }];
                }
            }
            DocumentAnswer::RefundDeduction { value } => {
                if let Some(deduction) = &deduction {
                    *value = Some(deduction.clone());
                }
            }
            _ => (),
        }
    }
    request
}

pub(in crate::tests) fn accepted_government_source(
    session: &Interview,
) -> (&FederalIncomeEntry, &GovernmentPaymentSource) {
    session
        .draft
        .income
        .iter()
        .find_map(|entry| match &entry.income {
            FederalIncome::GovernmentPayment { data } => Some((entry, data)),
            _ => None,
        })
        .unwrap()
}

pub(in crate::tests) fn retained_government_history_cases() -> Vec<(TaxRefundHistory, i64)> {
    let payment = historical_pease_payment(2017, false);
    let pease =
        historical_refund_history(2017, payment.itemized, payment.standard, payment.overall_limit);
    let mut nonordinary = pease.clone();
    let TaxRefundHistory::IncomeTax { ordinary, .. } = &mut nonordinary else { unreachable!() };
    *ordinary = false;
    vec![
        (pease, 400),
        (nonordinary, 0),
        (two_year_refund_history(), 400),
        (prior_and_current_refund_history(2_000_000), 300),
    ]
}

pub(in crate::tests) fn retained_government_history_draft(
    history: TaxRefundHistory,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    if matches!(history, TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { .. }) {
        itemized(draft.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
            paid: 1_990_000,
            allocation: TaxIncomeAllocation::CurrentReturn,
        };
    }
    let mut source =
        tax_refund_source(FederalTaxRefund { amount: 40_000, withholding: 50_000, history });
    source.form.unemployment_compensation = 200_000;
    source.repayments = vec![UnemploymentRepayment { received_year: 2026, amount: 30_000 }];
    add(&mut draft, income(FederalIncome::GovernmentPayment { data: source }));
    draft
}
