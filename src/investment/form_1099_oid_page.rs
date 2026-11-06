// form_1099_oid items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn project_oid_issue(issue: ValidationIssue) -> ValidationIssue {
    issue
        .map_field_prefix(&["nominee"], &["nominee", "value"])
        .map_field_prefix(&["oid"], &["original_issue_discount_for_the_year"])
        .map_field_prefix(&["early_penalty"], &["early_withdrawal_penalty"])
        .map_field_prefix(&["withholding"], &["federal_income_tax_withheld"])
        .map_field_prefix(
            &["treasury_oid"],
            &["original_issue_discount_on_us_treasury_obligations"],
        )
        .map_field_prefix(&["exempt"], &["tax_exempt_oid"])
}

// IRS Forms 1099-INT and 1099-OID instructions: allocate only reported components.
// https://www.irs.gov/instructions/i1099int
pub(super) fn oid_calls(w: &crate::OidBoxes) -> crate::OidCalls {
    let open = |called: bool, choices: &[i32]| if called { choices.to_vec() } else { Vec::new() };
    let reported = w.adjustments.is_some()
        || [
            w.oid,
            w.other_periodic_interest,
            w.more.market_discount,
            w.more.acquisition_premium,
            w.more.treasury_oid,
            w.more.bond_premium,
            w.more.exempt,
            w.more.tax_exempt_bond_premium,
            w.more.private_activity,
        ]
        .iter()
        .any(|value| *value != 0);
    crate::OidCalls { nominee: open(reported, &[0]) }
}

pub(crate) fn oid_record(v: &Form1099OidSource) -> FederalOid {
    FederalOid::from(v)
}

// The stored statement as its page: the paper from the amounts the record keeps
// and the payer and state boxes the caller keeps beside it, with the answers
// beside the paper.
pub(crate) fn oid_page(
    data: &FederalOid,
    kept: &crate::federal_model::ChildStatementKept,
) -> Form1099OidSource {
    let mut value = Form1099OidSource {
        form: Form1099OidPaper {
            calendar_year: Default::default(),
            corrected: data.printed_facts.corrected,
            payer: kept.payer.clone(),
            payer_telephone: data.printed_facts.payer_telephone.clone(),
            recipient: kept.recipient.clone(),
            original_issue_discount_for_the_year: data.oid,
            other_periodic_interest: data.other_periodic_interest,
            early_withdrawal_penalty: data.early_penalty,
            federal_income_tax_withheld: data.withholding,
            market_discount: data.market_discount,
            acquisition_premium: data.acquisition_premium,
            description: data.printed_facts.description.clone(),
            original_issue_discount_on_us_treasury_obligations: data.treasury_oid,
            investment_expenses: data.investment_expenses,
            bond_premium: data.bond_premium,
            tax_exempt_oid: data.exempt,
            fatca_filing_requirement: data.printed_facts.fatca_filing_requirement,
            account_number: data.printed_facts.account_number.clone(),
            state_details: kept.state_details.clone(),
        },
        calls: crate::OidCalls::default(),
        nominee: data.nominee.clone().map(Into::into),
        adjustments: data.adjustments.clone(),
        tax_exempt_bond_premium: data.tax_exempt_bond_premium,
        private_activity: data.private_activity,
        special: Some(data.special),
    };
    value.calls = oid_calls(&(&value).into());
    value
}
