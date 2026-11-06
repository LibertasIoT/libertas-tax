use super::paper_documents::remember;
use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn personal_debt_source(form: Cancel1099C) -> PersonalDebtSource {
    let Cancel1099C {
        creditor,
        debtor,
        date,
        amount,
        interest,
        debt_description,
        event_code,
        liable,
        treatment,
        property,
    } = form;
    let (property_value, property) = match property {
        PropertyTaken::None => (None, None),
        PropertyTaken::Taken { value, property } => (Some(value), property),
    };
    PersonalDebtSource {
        form: Form1099C {
            calendar_year: 0,
            corrected: false,
            creditor,
            debtor,
            date,
            amount,
            interest,
            debt_description,
            event_code: match event_code.as_str() {
                "" => None,
                "A" => Some(Form1099CEventCode::Bankruptcy),
                "B" => Some(Form1099CEventCode::OtherJudicialDebtRelief),
                "C" => Some(Form1099CEventCode::StatuteOfLimitations),
                "D" => Some(Form1099CEventCode::ForeclosureElection),
                "E" => Some(Form1099CEventCode::ProbateDebtRelief),
                "F" => Some(Form1099CEventCode::Agreement),
                "G" => Some(Form1099CEventCode::DiscontinueCollection),
                "H" => Some(Form1099CEventCode::OtherActualDischarge),
                _ => panic!("the fixture must name a printed Form 1099-C box 6 code"),
            },
            liable,
            property_value,
            account_number: String::new(),
        },
        treatment,
        property,
    }
}

pub(in crate::tests) fn debt_source(data: PersonalDebtCancellation) -> PersonalDebtSource {
    let PersonalDebtCancellation {
        date,
        amount,
        interest,
        debt_description,
        event_code,
        box7_fmv,
        treatment,
        personally_liable,
        transfer,
    } = data;
    let solvency = |value: Option<DebtInsolvency>| {
        value.map_or(DebtSolvency::Solvent, |data| DebtSolvency::Insolvent {
            liabilities: data.liabilities,
            assets: data.assets,
            basis_and_cash_after: data.basis_and_cash_after,
            amt_basis_and_cash_after: data.amt_basis_and_cash_after,
        })
    };
    let treatment = treatment.map(|value| match value {
        PersonalDebtTreatment::Taxable => CanceledDebtChoice::Taxable,
        PersonalDebtTreatment::GiftOrBequest => CanceledDebtChoice::Gift,
        PersonalDebtTreatment::StudentDischarge { reason } => {
            CanceledDebtChoice::StudentLoan { reason }
        }
        PersonalDebtTreatment::Unsupported { situation } => {
            CanceledDebtChoice::SomethingElse { situation }
        }
        PersonalDebtTreatment::PriceReduction { basis, assets, liabilities, sale } => {
            CanceledDebtChoice::SellerReduction { basis, assets, liabilities, sale }
        }
        PersonalDebtTreatment::Insolvent { data } => {
            let PersonalInsolvency {
                liabilities,
                assets,
                basis_and_cash_after,
                amt_basis_and_cash_after,
                business_debt,
                property,
                depreciable_first,
            } = data;
            CanceledDebtChoice::Insolvent {
                liabilities,
                assets,
                basis_and_cash_after,
                amt_basis_and_cash_after,
                business_debt,
                property,
                depreciable_first,
            }
        }
        PersonalDebtTreatment::Bankruptcy { data } => {
            let PersonalBankruptcy {
                basis_and_cash_after,
                amt_basis_and_cash_after,
                liabilities_after,
                property,
                depreciable_first,
            } = data;
            CanceledDebtChoice::Bankruptcy {
                basis_and_cash_after,
                amt_basis_and_cash_after,
                liabilities_after,
                property,
                depreciable_first,
            }
        }
        PersonalDebtTreatment::FarmDebt { data } => CanceledDebtChoice::FarmDebt {
            solvency: solvency(data.insolvency),
            property: data.property,
            depreciable_first: data.depreciable_first,
        },
        PersonalDebtTreatment::RealPropertyDebt { data } => {
            CanceledDebtChoice::RealPropertyBusiness {
                solvency: solvency(data.insolvency),
                principal: data.principal,
                value: data.value,
                other_secured: data.other_secured,
                depreciable_basis: data.depreciable_basis,
                property: data.property,
                depreciable_first: data.depreciable_first,
            }
        }
    });
    personal_debt_source(Cancel1099C {
        creditor: Default::default(),
        debtor: Default::default(),
        date,
        amount,
        interest,
        debt_description,
        event_code,
        liable: personally_liable,
        treatment,
        property: box7_fmv.map_or(PropertyTaken::None, |value| PropertyTaken::Taken {
            value,
            property: transfer.and_then(|transfer| transfer.property),
        }),
    })
}

pub(in crate::tests) fn debt_document(source: PersonalDebtSource) -> Form1099C {
    let form = source.form;
    remember(
        IncomeSource::CanceledDebt { value: CanceledDebtDocument::Cancel1099C(form.clone()) },
        vec![
            DocumentAnswer::DebtTreatment { value: source.treatment },
            DocumentAnswer::DebtProperty { value: source.property },
        ],
    );
    form
}
