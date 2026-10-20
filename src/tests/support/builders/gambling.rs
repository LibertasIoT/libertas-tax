use crate::tests::*;

pub(in crate::tests) fn gambling(winnings: i64, losses: i64) -> FederalIncomeEntry {
    income(FederalIncome::Gambling {
        data: FederalGambling {
            winnings: winnings * 100,
            losses: losses * 100,
            withholding: 0,
            own_records: false,
            ordinary: true,
        },
    })
}

pub(in crate::tests) fn professional_gambling(
    receipts: i64,
    losses: i64,
    expenses: i64,
) -> FederalIncome {
    let mut income = cash_business(receipts);
    if let FederalIncome::Business { data } = &mut income {
        data.activity = BusinessActivity::Gambling {
            returns: 0,
            receipts: data.receipts(),
            accounting: BusinessAccounting::Cash,
            losses: losses * 100,
            withholding: 24000,
            eligible: true,
        };
        data.expenses.push(ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Documented business travel".into() },
            expenses * 100,
        ));
    }
    income
}

pub(in crate::tests) fn prize(disposition: PrizeDisposition) -> FederalIncome {
    FederalIncome::OtherIncome {
        data: OtherIncomeItem {
            kind: OtherIncomeKind::Prize { noncash: 50000, disposition },
            amount: 300000,
            withholding: 10000,
        },
    }
}

pub(in crate::tests) fn child_gambling_return(winnings: i64, losses: i64) -> FederalDraft {
    let mut d = young_interest_return();
    d.income.clear();
    add(&mut d, gambling(winnings, losses));
    let deductions = d.deductions.as_mut().unwrap();
    let facts = itemized(deductions).clone();
    deductions.choice = DeductionSelection::Itemized { facts };
    d
}
