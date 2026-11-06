use crate::tests::*;
pub(in crate::tests) fn bba_interest_reduction() -> FederalBbaAuditInterest {
    FederalBbaAuditInterest {
        partnership: Default::default(),
        reviewed_year_partner: Default::default(),
        id: 0,
        source_id: "AUDIT-2025-EXAMPLE".into(),
        furnished_on: 20260615,
        interest_adjustment: -100_000,
        original_partnership_interest: 200_000,
        prior_wages: 6_375_000,
        prior_taxable_income: 5_000_000,
        prior_income_tax: 592_000,
        prior_status: FederalStatus::Single,
        single_audit_adjustment: true,
        simple_prior_return: true,
        no_other_effects: true,
        line_17_interest: 0,
        interest_calculation_prepared: false,
    }
}
pub(in crate::tests) fn bba_interest_increase() -> FederalBbaAuditInterest {
    FederalBbaAuditInterest {
        interest_adjustment: 100_000,
        line_17_interest: 3_500,
        interest_calculation_prepared: true,
        ..bba_interest_reduction()
    }
}

// Published 2025 single Tax Table: $11,950 -> $1,199; $1,975 -> $199.
pub(in crate::tests) fn bba_thousand_dollar_reduction() -> FederalBbaAuditInterest {
    FederalBbaAuditInterest {
        interest_adjustment: -997_500,
        original_partnership_interest: 1_100_000,
        prior_wages: 1_670_000,
        prior_taxable_income: 1_195_000,
        prior_income_tax: 119_900,
        ..bba_interest_reduction()
    }
}

pub(in crate::tests) fn bba_low_tax_return() -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, 1_000_000, 0)];
    draft.deductions.as_mut().unwrap().itemize_override = None;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_thousand_dollar_reduction()));
    draft
}

// A directly built return: the partner statement is yours, as your section added it.
pub(in crate::tests) fn added_bba(
    document: crate::FederalBbaAuditInterest,
) -> crate::BbaAuditSource {
    let mut source = crate::BbaAuditSource::from(document);
    source.assigned = crate::DocumentRecipient::Taxpayer;
    source.owner_source = Some(crate::OwnerSource::Added);
    source
}
