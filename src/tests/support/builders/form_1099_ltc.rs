use super::paper_documents::remember;
use crate::tests::*;

pub(in crate::tests) fn ltc_document(source: LtcSource) -> Ltc1099Paper {
    use crate::DocumentAnswer as A;
    let form = source.form;
    remember(
        IncomeSource::OtherIncome { value: OtherIncomeDocument::Ltc1099(form.clone()) },
        vec![
            A::LtcInterest { value: source.interest.unwrap_or(0) },
            A::LtcBenefitOrigin { value: source.benefit_origin },
            A::LtcAcceleratedOrigin { value: source.accelerated_origin },
            A::LtcContract { value: (&source.contract).into() },
            A::LtcInsured { value: source.insured },
        ],
    );
    form
}
