use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::*;

#[test]
fn unit47_bond_identifier_preserves_printed_cusip_separator_on_save() {
    // Form 1097-BTC (April 2025), Boxes 2a/2b:
    // https://www.irs.gov/pub/irs-pdf/f1097btc.pdf permits the CUSIP separator;
    // the old alphanumeric-only refusal was incorrect. The total stays at 39.
    for identifier in ["ABC123", "912828L57-A123", "912828L57-12345678901234567890123456789"] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let Reply::Edit(mut request) = from_review(&mut session, TaxSection::Documents) else {
            panic!("bond credit editor required")
        };
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let mut bond = direct_tax_credit_bond(
            identifier,
            FederalTaxCreditBondKind::QualifiedSchoolConstruction,
            1_000,
        );
        bond.form.recipient = accepted_document_names(&session.draft, Owner::Taxpayer).recipient;
        assert!(crate::tax_credit_bonds::valid(core::slice::from_ref(&bond)));
        documents.other.credits.documents = vec![crate::CreditDocument::CreditBond(
            alloc::boxed::Box::new(crate::TaxCreditBondDocument::from(bond.clone())),
        )];
        let reply = exchange(&mut session, request.clone(), identifier);
        accepted(&reply);
        bond.id = session.draft.credits.as_ref().unwrap().tax_credit_bonds[0].id;
        let request =
            crate::tests::support::client::credit_documents::credit_document_answers_request(
                &mut session,
                &[bond],
                None,
            );
        accepted(&send(&mut session, request));
        let expected = Some(alloc::string::String::from(identifier));
        assert_eq!(
            session.draft.credits.as_ref().unwrap().tax_credit_bonds[0].form.unique_identifier,
            expected
        );
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}
