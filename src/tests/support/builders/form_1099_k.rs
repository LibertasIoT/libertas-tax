use super::paper_documents::remember;
use crate::tests::*;

pub(in crate::tests) fn k_paper(form: Form1099K) -> K1099Source {
    let Form1099K {
        payer,
        recipient,
        cash_tips,
        tip_occupation_codes,
        gross,
        withholding,
        parts,
        state_details,
    } = form;
    K1099Source {
        form: Form1099KPaper {
            payer,
            recipient,
            cash_tips: Some(cash_tips),
            tip_occupation_codes,
            gross: Some(gross),
            withholding: Some(withholding),
            state_details: state_details.states.iter().map(Into::into).collect(),
            calendar_year: 2026,
            ..Default::default()
        },
        parts,
    }
}

pub(in crate::tests) fn k_source(data: FederalForm1099K) -> K1099Source {
    let FederalForm1099K { cash_tips, tip_occupation_codes, gross, withholding, parts } = data;
    k_paper(Form1099K {
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips,
        tip_occupation_codes,
        gross,
        withholding,
        parts: parts
            .into_iter()
            .map(|part| {
                let FederalKPart { id, amount, belongs, refunded } = part;
                KPart {
                    id,
                    amount,
                    refunded,
                    belongs: belongs.map(|value| match value {
                        FederalKUse::NotIncome => KUse::NotIncome,
                        FederalKUse::PersonalGain => KUse::PersonalGain,
                    }),
                }
            })
            .collect(),
        state_details: Default::default(),
    })
}

pub(in crate::tests) trait PaymentPaperFixture {
    fn k_document(self) -> Form1099KPaper;
}

impl PaymentPaperFixture for Form1099KPaper {
    fn k_document(self) -> Form1099KPaper {
        self
    }
}

impl PaymentPaperFixture for Form1099K {
    fn k_document(self) -> Form1099KPaper {
        k_document(k_paper(self))
    }
}

impl PaymentPaperFixture for K1099Source {
    fn k_document(self) -> Form1099KPaper {
        let form = self.form;
        remember(
            IncomeSource::SelfEmployment { value: SelfEmploymentDocument::Form1099K(form.clone()) },
            vec![DocumentAnswer::PaymentParts(crate::PaymentPartsAnswer { value: self.parts })],
        );
        form
    }
}

pub(in crate::tests) fn k_document(source: impl PaymentPaperFixture) -> Form1099KPaper {
    source.k_document()
}
