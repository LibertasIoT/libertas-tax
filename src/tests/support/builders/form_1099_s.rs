use super::paper_documents::remember;
use crate::tests::*;

fn home_sale_paper_native(form: Form1099S) -> Form1099SPaper {
    let Form1099S {
        filer,
        transferor,
        address,
        closed,
        proceeds,
        property_received,
        foreign_person,
    } = form;
    Form1099SPaper {
        filer,
        transferor,
        box_3_address_or_legal_description: address,
        box_1_date_of_closing: closed,
        box_2a_total_gross_proceeds: proceeds,
        box_6_property_or_services: property_received,
        box_7_foreign_person: foreign_person,
        calendar_year: 2026,
        ..Default::default()
    }
}

paper_adapter!(
    home_sale_paper,
    home_sale_paper_native,
    HomeSaleBoxFixture,
    Form1099S,
    Form1099SPaper
);

pub(in crate::tests) fn home_sale_income(source: MainHomeSale) -> IncomeSource {
    match &source.form {
        crate::HomeSaleSource::Paper(_) => IncomeSource::InvestmentSale {
            value: InvestmentSaleDocument::Form1099S(home_sale_document(source)),
        },
        crate::HomeSaleSource::Records(sale) => IncomeSource::Record {
            value: crate::IncomeRecord::MainHomeSale {
                sale: sale.clone(),
                answers: source.answers().expect("a manual sale fixture supplies its home answers"),
            },
        },
    }
}

pub(in crate::tests) fn home_sale_document(source: MainHomeSale) -> Form1099SPaper {
    let crate::HomeSaleSource::Paper(form) = source.form else {
        panic!("a Form 1099-S fixture must contain actual paper")
    };
    remember(
        IncomeSource::InvestmentSale { value: InvestmentSaleDocument::Form1099S(form.clone()) },
        vec![crate::DocumentAnswer::HomeSale { value: source.answers }],
    );
    form
}
