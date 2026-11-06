// payments items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct PaymentsIn<'a> {
    pub(super) estate_estimated_cents: &'a crate::kernel::ratio::exact::Exact,
    pub(super) fund_credit: i64,
    pub(super) fuel_credit: i64,
    pub(super) excess_social: i64,
}

pub(super) struct PaymentsOut {
    pub(super) withholding_25c: i64,
    pub(super) withholding_25d: i64,
    pub(super) estimated_dollars: i64,
    pub(super) estimated_cents: i64,
    pub(super) payments: i64,
    pub(super) payment_cents: i64,
}

pub(super) fn payments_phase(
    pass: &Pass,
    r: &FederalResult,
    i: PaymentsIn<'_>,
) -> Option<PaymentsOut> {
    let &Pass { pay, .. } = pass;
    let PaymentsIn { estate_estimated_cents, fund_credit, fuel_credit, excess_social } = i;
    let native_withheld = (r.payment_withholding.clone())?;
    let withholding_25c = dollars(native_withheld.other);
    let withholding_25d = dollars(native_withheld.total);
    let estimated_source = pay.estimated_source();
    use crate::kernel::ratio::exact::{rounded as payment_rounded, whole as payment_whole};
    let estimated_dollars =
        (payment_rounded(&((estimated_source + estate_estimated_cents) / payment_whole(100))))?;
    let estimated_cents = (estimated_dollars.checked_mul(100))?;
    let payments = (crate::kernel::ratio::checked_sum([
        fund_credit,
        fuel_credit,
        withholding_25d,
        estimated_dollars,
        dollars(pay.extension),
        excess_social,
    ]))?;
    let payment_cents = (payments.checked_mul(100))?;
    Some(PaymentsOut {
        withholding_25c,
        withholding_25d,
        estimated_dollars,
        estimated_cents,
        payments,
        payment_cents,
    })
}

pub(super) struct SettlementIn {
    pub(super) payments: i64,
    pub(super) refundable: i64,
    pub(super) current_tax: i64,
    pub(super) penalty: Option<i64>,
}

pub(super) struct SettlementOut {
    pub(super) refund_net: i64,
    pub(super) refund_cents: i64,
    pub(super) balance_cents: i64,
    pub(super) payment_net: i64,
    pub(super) final_refund_cents: i64,
    pub(super) final_balance_cents: i64,
    pub(super) credit_pending: bool,
    pub(super) reported_credit_cents: i64,
    pub(super) cash_refund_cents: i64,
}

pub(super) fn settlement_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    i: SettlementIn,
) -> Option<SettlementOut> {
    let &Pass { context, pay, s, .. } = pass;
    let SettlementIn { payments, refundable, current_tax, penalty } = i;
    let refund_net =
        crate::kernel::ratio::checked_sum([payments, refundable, current_tax.checked_neg()?])?;
    let refund_cents = positive(refund_net).checked_mul(100)?;
    let balance_cents = positive(refund_net.checked_neg()?).checked_mul(100)?;
    let payment_net = match penalty {
        Some(penalty) => crate::kernel::ratio::checked_sum([refund_net.checked_neg()?, penalty])?,
        None => refund_net.checked_neg()?,
    };
    // A missing optional penalty means IRS-figured, not a failed computation:
    // calculate records any true carrier failure as an issue before this stage.
    let final_refund_cents = positive(payment_net.checked_neg()?).checked_mul(100)?;
    let final_balance_cents = positive(payment_net).checked_mul(100)?;
    // The ordinary settlement owns the capacity: line 34 less the computed
    // penalty, or the instruction amount with line 38 blank for IRS figuring.
    // The planned election defaults to zero; it is never a current payment.
    if r.issues.is_empty() && context.financial_source_scope_complete {
        r.credit_capacity = Some(final_refund_cents);
    }
    let retained_credit_cents = pay.credit_applied_next_year;
    let credit_valid = retained_credit_cents >= 0
        && r.credit_capacity.is_none_or(|capacity| retained_credit_cents <= capacity)
        && (retained_credit_cents == 0 || pay.credit_recipient.is_some());
    let credit_pending = credit_valid && retained_credit_cents > 0 && r.credit_capacity.is_none();
    if !credit_valid {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::Payments),
            "Payments",
            "ISSUE_APPLIED_CREDIT_CAPACITY",
            "Correct the planned applied credit in Payments; it has not been reduced automatically.",
        );
    } else if credit_pending {
        r.review_notes.push(crate::federal_model::ReviewNote::scoped(
            #[cfg(test)] "The planned credit is retained, but the available overpayment is not yet known. Final cash and applied credit remain unavailable.",
            crate::Message::new("PAYMENT_APPLIED_CREDIT_UNAVAILABLE", alloc::vec![]),
            crate::IssueContext::Section(crate::TaxSection::Payments),
        ));
    }
    // Round line 36 ONCE, then subtract it from
    // whole-dollar settlement capacity. Never round an independent cash part.
    let reported_credit_cents = if credit_valid && !credit_pending {
        dollars(retained_credit_cents).checked_mul(100)?
    } else {
        0
    };
    let cash_refund_cents = crate::kernel::carry::settle_values::<crate::kernel::carry::Cents>(
        final_refund_cents,
        reported_credit_cents,
        false,
    )
    .expect("admitted line 36 within finalized capacity")
    .closing
    .get();
    if credit_valid && !credit_pending && retained_credit_cents > 0 {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] "The line 36 election is planned, not filed or paid; spouse-account intent does not assign payment ownership.",
            crate::Message::new("NOTE_APPLIED_CREDIT_PLAN", alloc::vec![crate::MessageArg::usd_cents(reported_credit_cents)]),
        ));
        if s.filing_status() != FederalStatus::Joint
            && pay.credit_recipient == Some(crate::OverpaymentRecipient::SpouseAccount)
        {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] "The spouse-account planned credit is outside this taxpayer's separate estimated-tax forecast.",
                crate::Message::new("NOTE_APPLIED_CREDIT_SPOUSE_OUTSIDE_FORECAST", alloc::vec![]),
            ));
        }
        if reported_credit_cents != retained_credit_cents {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!("Retained planned credit {retained_credit_cents} cents; reported line 36 {reported_credit_cents} cents."),
                crate::Message::new("NOTE_APPLIED_CREDIT_ROUNDING", alloc::vec![crate::MessageArg::usd_cents(retained_credit_cents), crate::MessageArg::usd_cents(reported_credit_cents)]),
            ));
        }
    }
    Some(SettlementOut {
        refund_net,
        refund_cents,
        balance_cents,
        payment_net,
        final_refund_cents,
        final_balance_cents,
        credit_pending,
        reported_credit_cents,
        cash_refund_cents,
    })
}
