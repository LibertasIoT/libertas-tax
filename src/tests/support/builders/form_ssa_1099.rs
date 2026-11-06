use crate::federal_model::Ssa1099Paper;
use crate::tests::*;

fn ssa_document_native(form: Ssa1099) -> Ssa1099Paper {
    let Ssa1099 {
        issuer,
        recipient,
        benefits_paid,
        paid_description,
        benefits_repaid,
        repaid_description,
        benefits,
        withholding,
        claim_number,
    } = form;
    Ssa1099Paper {
        issuer,
        recipient,
        benefits_paid: Some(benefits_paid),
        benefits_repaid_to_ssa: Some(benefits_repaid),
        net_benefits: benefits,
        voluntary_federal_income_tax_withheld: Some(withholding),
        claim_number,
        description_of_amount_in_box_3: paid_description,
        description_of_amount_in_box_4: repaid_description,
        tax_year: 2026,
    }
}

paper_adapter!(ssa_document, ssa_document_native, SocialBoxFixture, Ssa1099, Ssa1099Paper);
