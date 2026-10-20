// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

use crate::tests::support::assert::injured_credit::financial_omission_scope::money;
use crate::tests::support::builders::injured_credit::financial_omission_scope::positive_refund;
use crate::tests::support::client::injured_credit::financial_omission_scope::persisted;

#[test]
fn financial_omission_scope_empty_foreign_claim_is_informational_with_positive_refund() {
    let mut d = positive_refund();
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    for d in [d.clone(), persisted(d)] {
        let r = money(&d, 30000);
        assert!(noted(&r, "WARN_FORM_2555_NO_FOREIGN_INCOME"));
        assert!(r.financial_source_scope_complete);
        let mut notes = r.review_notes.clone();
        let count = notes.len();
        let scope = crate::unknowns::supported_with_scope(&d, &mut notes);
        assert!(scope.financial_source_scope_complete);
        assert_eq!(notes.len(), count);
        let mut plain = d.clone();
        plain.screening.as_mut().unwrap().foreign_earned.clear();
        let mut expected = money(&plain, 30000);
        expected.review_notes = r.review_notes.clone();
        assert_eq!(r, expected);
    }
}

#[test]
fn financial_omission_scope_declined_foreign_claim_preserves_availability() {
    let mut d = positive_refund();
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.wages = 2900000;
    add(&mut d, income(foreign_pay(1000)));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.tax_home = false;
    set_claim(&mut d, claim);
    for d in [d.clone(), persisted(d)] {
        // Explicitly declined exclusion preserves the accepted income and refund.
        let r = money(&d, 30000);
        assert!(r.financial_source_scope_complete);
        let mut notes = r.review_notes.clone();
        let count = notes.len();
        let scope = crate::unknowns::supported_with_scope(&d, &mut notes);
        assert!(scope.financial_source_scope_complete);
        assert_eq!(notes.len(), count);
    }
}

#[test]
fn financial_omission_scope_known_nominee_treatment_preserves_financial_availability() {
    let mut d = positive_refund();
    let mut entry = income(savings_income(20, 0));
    let FederalIncome::Interest { data } = &mut entry.income else { panic!() };
    data.nominee = Some(crate::InterestNominee { taxable: 1000, ..Default::default() });
    add(&mut d, entry);
    for d in [d.clone(), persisted(d)] {
        let r = money(&d, 30010);
        assert!(noted(&r, "NOTE_INVESTMENT_NOMINEE"));
        assert!(r.financial_source_scope_complete);
        assert_eq!(return_line(&r, "2b"), 1000);
    }
}
