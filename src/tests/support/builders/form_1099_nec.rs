use super::paper_documents::remember;
use crate::tests::*;

pub(in crate::tests) fn nec_paper(form: Form1099Nec) -> Nec1099Source {
    let Form1099Nec {
        payer,
        recipient,
        compensation,
        cash_tips,
        tip_occupation_codes,
        overtime,
        golden_parachute,
        withholding,
        belongs,
        state_details,
    } = form;
    Nec1099Source {
        form: Form1099NecPaper {
            payer,
            recipient,
            compensation: Some(compensation),
            cash_tips: Some(cash_tips),
            tip_occupation_codes,
            overtime: Some(overtime),
            golden_parachute: Some(golden_parachute),
            withholding: Some(withholding),
            state_details,
            calendar_year: 2026,
            ..Default::default()
        },
        belongs,
    }
}

pub(in crate::tests) fn nec_source(data: FederalForm1099Nec) -> Nec1099Source {
    let FederalForm1099Nec {
        cash_tips,
        tip_occupation_codes,
        overtime,
        compensation,
        withholding,
        golden_parachute,
        belongs,
    } = data;
    nec_paper(Form1099Nec {
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips,
        tip_occupation_codes,
        overtime,
        compensation,
        withholding,
        golden_parachute,
        belongs,
        state_details: Default::default(),
    })
}

pub(in crate::tests) fn nec_document(form: Form1099Nec) -> Form1099NecPaper {
    let source = nec_paper(form);
    let form = source.form;
    remember(
        IncomeSource::SelfEmployment { value: SelfEmploymentDocument::Form1099Nec(form.clone()) },
        vec![DocumentAnswer::NonemployeeUse { value: source.belongs }],
    );
    form
}
