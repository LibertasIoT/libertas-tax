use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::*;

#[test]
fn unit47_bond_identifier_preserves_printed_cusip_separator_on_save() {
    // 2026-10-15 dispatcher decision; Form 1097-BTC (April 2025), Boxes 2a/2b:
    // https://www.irs.gov/pub/irs-pdf/f1097btc.pdf permits the CUSIP separator;
    // the old alphanumeric-only refusal was incorrect. The total stays at 39.
    for (identifier, valid) in [
        ("ABC123", true),
        ("912828L57-A123", true),
        ("912828L57-12345678901234567890123456789", true),
        ("", false),
        ("-912828L57A123", false),
        ("912828L57--A123", false),
        ("912828L57-", false),
        ("912828L5-A123", false),
        ("912828L57-A 123", false),
        ("912828L57-123456789012345678901234567890", false),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let Reply::Edit(mut request) = from_review(&mut session, TaxSection::MoreCredits) else {
            panic!("bond credit editor required")
        };
        let P::SaveMoreCredits { value, .. } = &mut request else {
            unreachable!()
        };
        let bond = direct_tax_credit_bond(
            identifier,
            FederalTaxCreditBondKind::QualifiedSchoolConstruction,
            1_000,
        );
        assert_eq!(crate::tax_credit_bonds::valid(core::slice::from_ref(&bond)), valid);
        value.credits = vec![MoreCredit::TaxCreditBonds {
            tax_credit_bonds: vec![bond.into()],
        }];
        let reply = exchange(&mut session, request.clone(), identifier);
        let expected = if valid {
            accepted(&reply);
            identifier
        } else {
            error(reply);
            let P::SaveMoreCredits { value, .. } = &mut request else {
                unreachable!()
            };
            let MoreCredit::TaxCreditBonds { tax_credit_bonds } = &mut value.credits[0] else {
                unreachable!()
            };
            tax_credit_bonds[0].unique_identifier = "912828L57-A123".into();
            accepted(&send(&mut session, request));
            "912828L57-A123"
        };
        assert_eq!(
            session.draft.credits.as_ref().unwrap().tax_credit_bonds[0].unique_identifier,
            expected
        );
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}
