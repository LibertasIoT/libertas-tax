use super::*;
use crate::tests::support::assert::injured_income::typed_information_return_sources::check_states;

#[test]
fn unit13_typed_information_return_mixed_misc_and_nec_keep_each_native_category() {
    let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
    let mut misc = income(FederalIncome::Misc1099 {
        data: FederalForm1099Misc {
            rents: 0,
            rents_use: None,
            royalties: 0,
            royalties_use: None,
            other: 50,
            other_use: Some(IncomeUse::Income {
                kind: OtherIncomeKind::Prize {
                    noncash: 0,
                    disposition: PrizeDisposition::Received,
                },
            }),
            withholding: 0,
            fishing: 0,
            medical: 0,
            substitute: 50,
            attorney: 0,
            client_funds: 0,
            fish_sales: 0,
            deferred: 0,
            business: None,
        },
    });
    misc.owner = Owner::Spouse;
    add(&mut draft, misc);
    add(
        &mut draft,
        income(FederalIncome::Nec1099 {
            data: FederalForm1099Nec {
                compensation: 50,
                withholding: 0,
                golden_parachute: 0,
                belongs: IncomeUse::NotForProfit,
                business: None,
            },
        }),
    );
    // One MISC ID contributes to TWO native annual categories:50c of prizes
    // and50c of substitutes each round to1, both actually received by spouse.
    // Taxpayer's separate NEC not-for-profit50c rounds to1 in its own class.
    // Neither rounding the MISC's100c source total nor all150c explains3.
    check_states(draft, [1, 2], |result| {
        assert_eq!(value(result, "taxable_prizes"), 1);
        assert_eq!(value(result, "hobby_sales_income"), 1);
        assert_eq!(return_line(result, "8"), 300);
        assert_eq!(return_line(result, "9"), 8_000_300);
        assert_eq!(value(result, "agi"), 80_003);
        assert_eq!(value(result, "self_employment_tax"), 0);
    });
}

#[test]
fn unit13_typed_information_return_signed_medicaid_categories_keep_exclusion_once() {
    let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
    let mut excluded = income(FederalIncome::OtherIncome {
        data: OtherIncomeItem {
            amount: 50,
            withholding: 0,
            kind: OtherIncomeKind::MedicaidWaiver { in_your_home: true },
        },
    });
    excluded.owner = Owner::Spouse;
    add(&mut draft, excluded);
    add(
        &mut draft,
        income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                amount: 50,
                withholding: 0,
                kind: OtherIncomeKind::HouseholdWages,
            },
        }),
    );
    // Spouse's native positive1d category1 and negative8s category−1 cancel.
    // Taxpayer's separate actual household wages round to1 on1040line1b.
    check_states(draft, [1, 0], |result| {
        assert_eq!(return_line(result, "1b"), 100);
        assert_eq!(return_line(result, "1d"), 100);
        assert_eq!(value(result, "medicaid_waiver_excluded"), -1);
        assert_eq!(return_line(result, "8"), -100);
        assert_eq!(return_line(result, "9"), 8_000_100);
        assert_eq!(value(result, "agi"), 80_001);
        assert_eq!(value(result, "self_employment_tax"), 0);
    });
}
