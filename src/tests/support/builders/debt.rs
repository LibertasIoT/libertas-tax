use crate::tests::*;

pub(in crate::tests) fn canceled_personal_debt(
    amount: i64,
    treatment: PersonalDebtTreatment,
) -> FederalIncomeEntry {
    income(FederalIncome::PersonalDebt {
        data: debt_source(PersonalDebtCancellation {
            date: 20260615,
            amount: amount * 100,
            treatment: Some(treatment),
            personally_liable: true,
            box7_fmv: None,
            interest: None,
            debt_description: alloc::string::String::new(),
            event_code: alloc::string::String::new(),
            transfer: None,
        }),
    })
}

pub(in crate::tests) fn insolvency_debt(
    amount: i64,
    liabilities: i64,
    assets: i64,
    after: i64,
) -> FederalIncomeEntry {
    canceled_personal_debt(
        amount,
        PersonalDebtTreatment::Insolvent {
            data: PersonalInsolvency {
                liabilities: liabilities * 100,
                assets: assets * 100,
                basis_and_cash_after: after * 100,
                amt_basis_and_cash_after: None,
                property: personal_property(),
                depreciable_first: 0,
                business_debt: false,
            },
        },
    )
}

pub(in crate::tests) fn personal_property() -> Vec<DebtProperty> {
    vec![
        DebtProperty {
            label: "Furniture".into(),
            kind: DebtPropertyKind::PersonalUse,
            basis: 500000,
            amt_basis: None,
        },
        DebtProperty {
            label: "Jewelry".into(),
            kind: DebtPropertyKind::PersonalUse,
            basis: 50000,
            amt_basis: None,
        },
    ]
}

pub(in crate::tests) fn bankruptcy_debt(
    amount: i64,
    after: i64,
    liabilities: i64,
) -> FederalIncomeEntry {
    canceled_personal_debt(
        amount,
        PersonalDebtTreatment::Bankruptcy {
            data: PersonalBankruptcy {
                basis_and_cash_after: after * 100,
                amt_basis_and_cash_after: None,
                liabilities_after: liabilities * 100,
                property: personal_property(),
                depreciable_first: 0,
            },
        },
    )
}

pub(in crate::tests) fn price_reduction_debt() -> FederalIncomeEntry {
    canceled_personal_debt(
        2000,
        PersonalDebtTreatment::PriceReduction {
            sale: None,
            basis: 1000000,
            assets: 3000000,
            liabilities: 2000000,
        },
    )
}

pub(in crate::tests) fn health_loan_assistance(
    program: HealthLoanProgram,
    interest: i64,
) -> FederalIncomeEntry {
    income(FederalIncome::HealthLoanAssistance {
        data: HealthLoanAssistance { program, amount: 500000, interest, confirmed: true },
    })
}

pub(in crate::tests) fn personal_debt_owner_fixture(
    owner: Owner,
    transfer: bool,
    excluded: bool,
) -> FederalDraft {
    let mut original = injured_return((60_000, 15_000), (20_000, 5_000), 0, allocation());
    let amount = if transfer || excluded { 2_000 } else { 5_000 };
    let mut entry = canceled_personal_debt(
        amount,
        if excluded {
            PersonalDebtTreatment::GiftOrBequest
        } else {
            PersonalDebtTreatment::Taxable
        },
    );
    entry.owner = crate::DocumentOwner::from(owner);
    if transfer {
        let FederalIncome::PersonalDebt { data } = &mut entry.income else { unreachable!() };
        data.form.property_value = Some(1_200_000);
        data.property = Some(TransferredProperty::Investment {
            basis: 1_000_000,
            term: GainTerm::Long,
            collectible: false,
        });
    }
    add(&mut original, entry);
    original
}

pub(in crate::tests) fn property(label: &str, kind: DebtPropertyKind, basis: i64) -> DebtProperty {
    DebtProperty { label: label.into(), kind, basis: basis * 100, amt_basis: None }
}
pub(in crate::tests) fn farm_debt(
    amount: i64,
    insolvency: Option<(i64, i64)>,
    equipment: i64,
) -> FederalIncomeEntry {
    canceled_personal_debt(
        amount,
        PersonalDebtTreatment::FarmDebt {
            data: FarmDebtExclusion {
                insolvency: insolvency.map(|(liabilities, assets)| DebtInsolvency {
                    liabilities: liabilities * 100,
                    assets: assets * 100,
                    basis_and_cash_after: 0,
                    amt_basis_and_cash_after: None,
                }),
                property: vec![property(
                    "Tractor",
                    DebtPropertyKind::Equipment { secured: false },
                    equipment,
                )],
                depreciable_first: 0,
            },
        },
    )
}
