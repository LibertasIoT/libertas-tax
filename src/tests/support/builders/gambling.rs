use crate::tests::*;

pub(in crate::tests) fn gambling(winnings: i64, losses: i64) -> FederalIncomeEntry {
    income(FederalIncome::Gambling {
        data: gambling_source(FederalGambling {
            winnings: winnings * 100,
            losses: losses * 100,
            withholding: 0,
            own_records: losses != 0,
            ordinary: true,
        }),
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
    {
        deductions.itemized = facts;
        deductions.itemize_override = Some(ItemizeOverride::Itemize);
    };
    d
}

// Native business pages establish both source and asset identities.
pub(in crate::tests) fn wagering179_session() -> Interview {
    use crate::tests::cross::interview::{asset, business, save_businesses, with_first_year};
    let mut draft = complete(FederalStatus::Single);
    draft.income.clear();
    let mut session = restart(draft);
    let mut ordinary = business(0);
    ordinary.name = "Equipment service".into();
    ordinary.assets = vec![with_first_year(
        asset(700_000, 20260115, AssetClass::Five),
        FirstYearDeduction::Section179 {
            description: "Service equipment".into(),
            elected: 700_000,
        },
    )];
    let mut wagering = business(1_000_000);
    wagering.name = "Professional wagering".into();
    wagering.expenses.advertising = 100_000;
    wagering.more.gambling =
        Some(ProfessionalGambling { losses: 500_000, withholding: 0, eligible: true });
    crate::tests::support::assert::accepted(&save_businesses(
        &mut session,
        vec![ordinary, wagering],
    ));
    session
}
