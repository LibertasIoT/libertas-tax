use super::paper_documents::remember;
use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn property_acquisition_source(
    form: Acquisition1099A,
) -> PropertyAcquisitionSource {
    let Acquisition1099A { lender, borrower, description, date, balance, value, liable, property } =
        form;
    PropertyAcquisitionSource {
        form: Form1099A {
            calendar_year: 0,
            corrected: false,
            box3: String::new(),
            lender,
            borrower,
            description,
            date,
            balance,
            value,
            liable,
            account_number: String::new(),
        },
        property,
    }
}

pub(in crate::tests) fn acquisition_source(data: PropertyAcquisition) -> PropertyAcquisitionSource {
    let PropertyAcquisition {
        description,
        date,
        balance,
        value,
        personally_liable,
        property,
        crop,
    } = data;
    property_acquisition_source(Acquisition1099A {
        lender: Default::default(),
        borrower: Default::default(),
        description,
        date,
        balance,
        value,
        liable: personally_liable,
        property: match crop {
            Some(loan) => Some(AcquiredProperty::CccCrop { loan }),
            None => property.map(|kind| AcquiredProperty::Property { kind }),
        },
    })
}

pub(in crate::tests) fn acquisition_document(source: PropertyAcquisitionSource) -> Form1099A {
    let form = source.form;
    remember(
        IncomeSource::CanceledDebt { value: CanceledDebtDocument::Acquisition1099A(form.clone()) },
        vec![DocumentAnswer::DebtAcquisition { value: source.property.as_ref().map(Into::into) }],
    );
    form
}
