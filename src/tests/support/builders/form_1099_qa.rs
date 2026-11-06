use super::paper_documents::remember;
use crate::tests::*;

pub(in crate::tests) fn able_source(data: AbleDistribution) -> AbleSource {
    let AbleDistribution {
        gross,
        earnings,
        program_to_program_transfer,
        account_terminated,
        recipient_not_designated_beneficiary,
        treatment,
    } = data;
    AbleSource {
        form: Form1099QaPaper {
            payer: Default::default(),
            recipient: Default::default(),
            gross,
            earnings,
            basis: gross - earnings,
            program_to_program_transfer,
            account_terminated,
            recipient_not_designated_beneficiary,
            ..Default::default()
        },
        treatment,
    }
}

pub(in crate::tests) fn able_document(source: AbleSource) -> Form1099QaPaper {
    let form = source.form;
    remember(
        IncomeSource::OtherIncome { value: OtherIncomeDocument::Qa1099(form.clone()) },
        vec![crate::DocumentAnswer::AbleTreatment { value: source.treatment }],
    );
    form
}
