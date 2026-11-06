use super::paper_documents::remember;
use crate::tests::*;

pub(in crate::tests) fn sale_source(data: FederalSale) -> SaleSource {
    SaleSource::from_sale(&data, "Synthetic sale")
}

pub(in crate::tests) fn contracts_source(data: FederalSection1256) -> ContractsSource {
    ContractsSource {
        form: Contracts1099B {
            payer: Default::default(),
            recipient: Default::default(),
            aggregate: data.aggregate,
            withholding: data.withholding,
            state_details: Default::default(),
        },
        special: data.special_treatment,
    }
}

pub(in crate::tests) fn sale_document(source: SaleSource) -> Sale1099B {
    let form = source.form;
    remember(
        IncomeSource::InvestmentSale { value: InvestmentSaleDocument::Sale1099B(form.clone()) },
        vec![
            crate::DocumentAnswer::SaleAdjustment { value: source.adjustment },
            crate::DocumentAnswer::SaleTreatment { value: source.special },
        ],
    );
    form
}

pub(in crate::tests) fn contracts_document(source: ContractsSource) -> Contracts1099B {
    let form = source.form;
    remember(
        IncomeSource::InvestmentSale {
            value: InvestmentSaleDocument::Contracts1099B(form.clone()),
        },
        vec![crate::DocumentAnswer::ContractsSpecial { value: source.special }],
    );
    form
}
