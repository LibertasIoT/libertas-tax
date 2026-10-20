use super::{
    phase,
    ratio::{checked_fraction, dollars, fraction},
};

/// Known year-of-sale amounts, already staged in whole dollars by the source owner.
/// Recapture is section 453(i) income recognized immediately. Other recapture
/// is the ordered payment-dependent pool supplied by its existing class owner.
#[derive(Clone, Copy)]
pub(crate) struct YearOfSaleDollars {
    pub price: i64,
    pub debt: i64,
    pub cost: i64,
    pub depreciation: i64,
    pub expenses: i64,
    pub recapture: i64,
    pub payments: i64,
    pub other_recapture: i64,
}

/// Known retained Form 6252 and current principal amounts, in source cents.
#[derive(Clone, Copy)]
pub(crate) struct PriorSaleCents {
    pub gross_profit: i64,
    pub contract_price: i64,
    pub earlier_payments: i64,
    pub payments: i64,
}

/// Form 6252 amounts in whole dollars. No source facts or availability are inferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Schedule {
    pub price: i64,
    pub debt: i64,
    pub cost: i64,
    pub depreciation: i64,
    pub expenses: i64,
    pub recapture: i64,
    pub gross_profit: i64,
    pub excess_debt: i64,
    pub contract: i64,
    pub payments: i64,
    pub earlier_payments: i64,
    pub income: i64,
    pub ordinary: i64,
    pub gain: i64,
}

/// Preserve the current Form 6252 computation. None means no installment gain;
/// the source owner reports that sale in full. It does not mean unavailable.
pub(crate) fn year_of_sale(input: YearOfSaleDollars) -> Option<Schedule> {
    let YearOfSaleDollars {
        price,
        debt,
        cost,
        depreciation,
        expenses,
        recapture,
        payments,
        other_recapture,
    } = input;
    let line13 = cost - depreciation + expenses + recapture;
    let gross_profit = price - line13;
    if gross_profit <= 0 {
        return None;
    }
    let excess_debt = phase::floor(debt - line13, 0);
    let contract = price - debt + excess_debt;
    let income =
        phase::bound(fraction(excess_debt + payments, gross_profit, contract), 0, gross_profit);
    let ordinary = phase::cap(income, [phase::floor(other_recapture, 0)]);
    Some(Schedule {
        price,
        debt,
        cost,
        depreciation,
        expenses,
        recapture,
        gross_profit,
        excess_debt,
        contract,
        payments,
        earlier_payments: 0,
        income,
        ordinary,
        gain: income - ordinary,
    })
}

/// Preserve the existing separate cents-to-dollars stages for a later payment year.
pub(crate) fn payment_year(input: PriorSaleCents) -> Schedule {
    let gross_profit = dollars(input.gross_profit);
    let contract = dollars(input.contract_price).max(1);
    let payments = dollars(input.payments);
    let income = phase::bound(fraction(payments, gross_profit, contract), 0, gross_profit);
    Schedule {
        price: 0,
        debt: 0,
        cost: 0,
        depreciation: 0,
        expenses: 0,
        recapture: 0,
        gross_profit,
        excess_debt: 0,
        contract,
        payments,
        earlier_payments: dollars(input.earlier_payments),
        income,
        ordinary: 0,
        gain: income,
    }
}

impl Schedule {
    /// Note face amount outstanding at year end, in whole dollars.
    pub(crate) fn outstanding(&self) -> i64 {
        phase::floor(self.price - self.debt - self.payments, 0)
    }
}

/// Additional principal and income recognized on a qualifying related disposition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Acceleration {
    pub limited_realized_cents: i64,
    pub deemed_cents: i64,
    pub income_dollars: i64,
}

/// Eligible net pledge proceeds cannot exceed actual unpaid principal.
/// Eligibility and prior actual/deemed principal accounting belong to the caller.
#[allow(dead_code)] // Atlas R1 caller lands separately.
pub(crate) fn pledge_payment(unpaid_cents: i64, net_proceeds_cents: i64) -> Option<i64> {
    if unpaid_cents < 0 || net_proceeds_cents < 0 {
        return None;
    }
    Some(unpaid_cents.min(net_proceeds_cents))
}

/// Form 6252 Part III acceleration only; eligibility remains caller-owned.
/// Earlier and current payments each include their actual and already deemed
/// principal once, including applicable assumed debt and pledge proceeds.
#[allow(dead_code)] // Atlas R1 caller lands separately.
pub(crate) fn related_disposition(
    input: PriorSaleCents,
    amount_realized_cents: i64,
) -> Option<Acceleration> {
    if input.gross_profit < 0
        || input.contract_price <= 0
        || input.earlier_payments < 0
        || input.payments < 0
        || amount_realized_cents < 0
    {
        return None;
    }
    let paid = input.earlier_payments.checked_add(input.payments)?;
    let limited_realized_cents = amount_realized_cents.min(input.contract_price);
    let deemed_cents = (limited_realized_cents - paid).max(0);
    let payments = input.payments.checked_add(deemed_cents)?;
    // Check native projection safety before using the unchanged annual owner.
    // Keep its separate source-cent dollar stages and minimum denominator.
    for annual_payments in [input.payments, payments] {
        checked_fraction(
            dollars(annual_payments),
            dollars(input.gross_profit),
            dollars(input.contract_price).max(1),
        )?;
    }
    let income_dollars = payment_year(PriorSaleCents { payments, ..input })
        .income
        .checked_sub(payment_year(input).income)?;
    Some(Acceleration { limited_realized_cents, deemed_cents, income_dollars })
}
