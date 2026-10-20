use crate::kernel::installment::{PriorSaleCents, YearOfSaleDollars, payment_year, year_of_sale};

#[test]
fn installment_sale_recapture_debt_and_payment_boundaries() {
    // Whole-dollar source inputs; expected Form 6252 results are hand-computed.
    for (amounts, expected) in [
        (
            [100_000, 0, 80_000, 60_000, 0, 60_000, 20_000, 0],
            Some([20_000, 0, 100_000, 4_000, 0, 4_000, 80_000]),
        ),
        (
            [100_000, 60_000, 40_000, 0, 0, 0, 10_000, 0],
            Some([60_000, 20_000, 60_000, 30_000, 0, 30_000, 30_000]),
        ),
        (
            [100_000, 0, 50_000, 10_000, 5_000, 10_000, 20_000, 0],
            Some([45_000, 0, 100_000, 9_000, 0, 9_000, 80_000]),
        ),
        (
            [100_000, 0, 40_000, 0, 0, 0, 20_000, 5_000],
            Some([60_000, 0, 100_000, 12_000, 5_000, 7_000, 80_000]),
        ),
        (
            [100_000, 0, 40_000, 0, 0, 0, 20_000, 15_000],
            Some([60_000, 0, 100_000, 12_000, 12_000, 0, 80_000]),
        ),
        ([10_000, 0, 10_000, 0, 0, 0, 10_000, 0], None),
        ([9_000, 0, 10_000, 0, 0, 0, 9_000, 0], None),
    ] {
        let [price, debt, cost, depreciation, expenses, recapture, payments, other_recapture] =
            amounts;
        let result = year_of_sale(YearOfSaleDollars {
            price,
            debt,
            cost,
            depreciation,
            expenses,
            recapture,
            payments,
            other_recapture,
        });
        assert_eq!(
            result.map(|s| [
                s.gross_profit,
                s.excess_debt,
                s.contract,
                s.income,
                s.ordinary,
                s.gain,
                s.outstanding()
            ]),
            expected
        );
        if let Some(s) = result {
            assert_eq!(
                [
                    s.price,
                    s.debt,
                    s.cost,
                    s.depreciation,
                    s.expenses,
                    s.recapture,
                    s.payments,
                    s.earlier_payments
                ],
                [price, debt, cost, depreciation, expenses, recapture, payments, 0]
            );
        }
    }
}

#[test]
fn installment_prior_sale_keeps_native_rounding_and_known_zero() {
    for (amounts, expected) in [
        ([6_000_000, 10_000_000, 2_000_000, 2_000_000], [60_000, 100_000, 20_000, 20_000, 12_000]),
        ([100, 300, 0, 150], [1, 3, 0, 2, 1]),
        ([1, 1, 0, 1], [0, 1, 0, 0, 0]),
        ([6_000_000, 10_000_000, 2_000_000, 0], [60_000, 100_000, 20_000, 0, 0]),
    ] {
        let [gross_profit, contract_price, earlier_payments, payments] = amounts;
        let s = payment_year(PriorSaleCents {
            gross_profit,
            contract_price,
            earlier_payments,
            payments,
        });
        assert_eq!(
            [s.gross_profit, s.contract, s.earlier_payments, s.payments, s.income],
            expected
        );
        assert_eq!(s.gain, s.income);
        assert_eq!(
            [
                s.price,
                s.debt,
                s.cost,
                s.depreciation,
                s.expenses,
                s.recapture,
                s.excess_debt,
                s.ordinary,
                s.outstanding()
            ],
            [0; 9]
        );
    }
}

#[test]
fn pledge_payment_caps_known_proceeds_at_unpaid_principal() {
    use super::installment::pledge_payment;
    for (unpaid, proceeds, expected) in [
        (10_000, 4_000, Some(4_000)),
        (10_000, 20_000, Some(10_000)),
        (0, 20_000, Some(0)),
        (10_000, 0, Some(0)),
        (i64::MAX, i64::MAX, Some(i64::MAX)),
        (-1, 1, None),
        (1, -1, None),
    ] {
        assert_eq!(pledge_payment(unpaid, proceeds), expected);
    }
}

#[test]
fn related_disposition_counts_actual_and_deemed_principal_once() {
    use super::installment::{Acceleration, pledge_payment, related_disposition};
    // $100,000 contract; $20,000 earlier actual + $10,000 prior deemed.
    // Current $6,000 actual + $4,000 assumed-debt payment + $5,000
    // pledge leaves $55,000 unpaid; those amounts are already in payments.
    let pledge = pledge_payment(6_000_000, 500_000).unwrap();
    let input = PriorSaleCents {
        gross_profit: 6_000_000,
        contract_price: 10_000_000,
        earlier_payments: 3_000_000,
        payments: 1_000_000 + pledge,
    };
    for (realized, limited_realized_cents, deemed_cents, income_dollars) in [
        (0, 0, 0, 0),
        (4_500_000, 4_500_000, 0, 0),
        (7_000_000, 7_000_000, 2_500_000, 15_000),
        (10_000_000, 10_000_000, 5_500_000, 33_000),
        (12_000_000, 10_000_000, 5_500_000, 33_000),
    ] {
        assert_eq!(
            related_disposition(input, realized),
            Some(Acceleration { limited_realized_cents, deemed_cents, income_dollars })
        );
    }
}

#[test]
fn related_disposition_uses_original_ratio_and_annual_rounding() {
    use super::installment::{Acceleration, related_disposition};
    // The annual owner stages $1 / $3 and rounds annual principal first.
    // Current 49 cents rounds to $0; combined 150 cents rounds to $2,
    // recognizing $1. Rounding extra 101 cents separately would give $0.
    for (gross_profit, contract_price, earlier_payments, payments, realized, deemed, income) in [
        (100, 300, 0, 49, 150, 101, 1),
        // Annual $1 is already recognized; the extra must not recognize it again.
        (100, 300, 0, 150, 300, 150, 0),
        // Earlier principal must not replace the original $1/$3 ratio.
        (100, 300, 100, 49, 250, 101, 1),
        // The two prior-source dollar stages remain independent.
        (149, 349, 0, 49, 150, 101, 1),
        (1, 1, 0, 0, 1, 1, 0),
        (i64::MAX, i64::MAX, 0, 0, i64::MAX, i64::MAX, 92_233_720_368_547_758),
        (0, 300, 0, 0, 300, 300, 0),
        // Actual payments can exhaust or exceed remaining principal.
        (100, 300, 200, 100, 300, 0, 0),
        (100, 300, 200, 200, 300, 0, 0),
    ] {
        assert_eq!(
            related_disposition(
                PriorSaleCents { gross_profit, contract_price, earlier_payments, payments },
                realized
            ),
            Some(Acceleration {
                limited_realized_cents: realized,
                deemed_cents: deemed,
                income_dollars: income,
            })
        );
    }
}

#[test]
fn related_disposition_rejects_invalid_and_unrepresentable_known_amounts() {
    use super::installment::related_disposition;
    for (gross_profit, contract_price, earlier_payments, payments, realized) in [
        (-1, 300, 0, 0, 0),
        (100, 0, 0, 0, 0),
        (100, -1, 0, 0, 0),
        (100, 300, -1, 0, 0),
        (100, 300, 0, -1, 0),
        (100, 300, 0, 0, -1),
        (100, i64::MAX, i64::MAX, 1, 0),
        // The legacy fraction narrows; this API rejects that overflow.
        (i64::MAX, 100, 0, i64::MAX, 0),
    ] {
        assert_eq!(
            related_disposition(
                PriorSaleCents { gross_profit, contract_price, earlier_payments, payments },
                realized
            ),
            None
        );
    }
}
