use crate::tests::*;

pub(in crate::tests) fn itemized(d: &mut FederalDeductions) -> &mut ItemizedExpenses {
    &mut d.itemized
}

pub(in crate::tests) fn elect_itemized(draft: &mut FederalDraft) {
    draft.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
}

// A complete native submission of the strict spouse question.
pub(in crate::tests) fn spouse_itemizes_request(d: &FederalDraft, value: bool) -> P {
    P::SaveSpouseItemizes {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: TaxSection::Deductions as i32,
        previous: TaxSection::Deductions as i32,
        review: false,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value,
    }
}

pub(in crate::tests) fn empty_facts() -> ItemizedExpenses {
    ItemizedExpenses {
        income_tax_owners: None,
        investment_property_tax_owners: None,
        investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
        real_estate_taxes: 0,
        other_lines: vec![],
        medical: 0,
        long_term_care: vec![],
        personal_tax: PersonalTaxElection::None,
        personal_property_taxes: 0,
        investment_property_taxes: 0,
    }
}
