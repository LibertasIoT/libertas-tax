use super::paper_documents::remember;
use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn misc_paper(form: Form1099Misc) -> Misc1099Source {
    let Form1099Misc {
        payer,
        recipient,
        component_ids,
        other_use,
        client_funds,
        rents,
        royalties,
        other,
        withholding,
        fishing,
        medical,
        substitute,
        crop_insurance,
        attorney,
        fish_sales,
        cash_tips,
        tip_occupation_codes,
        overtime,
        deferred,
        state_details,
    } = form;
    Misc1099Source {
        form: Form1099MiscPaper {
            payer,
            recipient,
            rents: Some(rents),
            royalties: Some(royalties),
            other: Some(other),
            withholding: Some(withholding),
            fishing: Some(fishing),
            medical: Some(medical),
            substitute: Some(substitute),
            crop_insurance: Some(crop_insurance),
            attorney: Some(attorney),
            fish_sales: Some(fish_sales),
            cash_tips: Some(cash_tips),
            tip_occupation_codes,
            overtime: Some(overtime),
            deferred: Some(deferred),
            state_details: (&state_details).into(),
            calendar_year: 2026,
            corrected: false,
            payer_telephone: String::new(),
            fatca_filing_requirement: false,
            account_number: String::new(),
            direct_sales: false,
            section_409a_deferrals: Some(0),
        },
        component_ids,
        other_use,
        client_funds,
    }
}

pub(in crate::tests) fn misc_source(data: FederalForm1099Misc) -> Misc1099Source {
    let FederalForm1099Misc {
        cash_tips,
        tip_occupation_codes,
        overtime,
        component_ids,
        rents,
        royalties,
        other,
        other_use,
        withholding,
        fishing,
        medical,
        substitute,
        crop_insurance,
        attorney,
        client_funds,
        fish_sales,
        deferred,
    } = data;
    misc_paper(Form1099Misc {
        payer: Default::default(),
        recipient: Default::default(),
        cash_tips,
        tip_occupation_codes,
        overtime,
        component_ids,
        rents,
        royalties,
        other,
        other_use,
        withholding,
        fishing,
        medical,
        substitute,
        crop_insurance,
        attorney,
        client_funds,
        fish_sales,
        deferred,
        state_details: Default::default(),
    })
}

pub(in crate::tests) trait MiscPaperFixture {
    fn misc_document(self) -> Form1099MiscPaper;
}

impl MiscPaperFixture for Form1099MiscPaper {
    fn misc_document(self) -> Form1099MiscPaper {
        self
    }
}

impl MiscPaperFixture for Form1099Misc {
    fn misc_document(self) -> Form1099MiscPaper {
        misc_document(misc_paper(self))
    }
}

impl MiscPaperFixture for Misc1099Source {
    fn misc_document(self) -> Form1099MiscPaper {
        let form = self.form;
        remember(
            IncomeSource::OtherIncome { value: OtherIncomeDocument::Form1099Misc(form.clone()) },
            vec![
                DocumentAnswer::MiscellaneousUse { value: self.other_use },
                DocumentAnswer::AttorneyClientFunds { value: self.client_funds },
            ],
        );
        form
    }
}

pub(in crate::tests) fn misc_document(source: impl MiscPaperFixture) -> Form1099MiscPaper {
    source.misc_document()
}
