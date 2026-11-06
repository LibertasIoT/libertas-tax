// form_1099_int items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn project_interest_issue(issue: ValidationIssue) -> ValidationIssue {
    issue
        .map_field_prefix(&["after_death"], &["after_death", "value"])
        .map_field_prefix(&["nominee"], &["nominee", "value"])
        .map_field_prefix(&["taxable"], &["interest_income"])
        .map_field_prefix(&["early_penalty"], &["early_withdrawal_penalty"])
        .map_field_prefix(&["us_obligations"], &["us_savings_bond_and_treasury_interest"])
        .map_field_prefix(&["withholding"], &["federal_income_tax_withheld"])
        .map_field_prefix(&["foreign_tax"], &["foreign_tax_paid"])
        .map_field_prefix(&["foreign_country"], &["foreign_country_or_us_territory"])
        .map_field_prefix(&["exempt"], &["tax_exempt_interest"])
        .map_field_prefix(&["private_activity"], &["specified_private_activity_bond_interest"])
        .map_field_prefix(&["treasury_bond_premium"], &["bond_premium_on_treasury_obligations"])
        .map_field_prefix(&["tax_exempt_bond_premium"], &["bond_premium_on_tax_exempt_bond"])
}

// IRS Forms 1099-INT and 1099-OID instructions: allocate only reported components.
// https://www.irs.gov/instructions/i1099int
pub(super) fn int_calls(final_return: bool, w: &crate::IntBoxes) -> crate::IntCalls {
    let open = |called: bool, choices: &[i32]| if called { choices.to_vec() } else { Vec::new() };
    let reported = w.adjustments.is_some()
        || [
            w.taxable,
            w.more.us_obligations,
            w.more.exempt,
            w.more.private_activity,
            w.more.bond_premium,
            w.more.treasury_bond_premium,
            w.more.tax_exempt_bond_premium,
            w.more.accrued_interest,
            w.more.market_discount,
            w.more.foreign_tax,
        ]
        .iter()
        .any(|value| *value != 0);
    crate::IntCalls {
        nominee: open(reported, &[0]),
        // Pub. 559: only a final-return statement can include income of the estate.
        // https://www.irs.gov/publications/p559
        after_death: open(reported && final_return, &[0]),
    }
}

pub(crate) fn interest_record(v: &Form1099IntSource) -> FederalInterest {
    FederalInterest::from(v)
}

// The stored statement as its page: the paper from the amounts the record keeps
// and the payer and state boxes the caller keeps beside it (a filer's own
// statement keeps them in its label and the entry's state source; a child's
// in the election), with the answers beside the paper.
pub(crate) fn interest_page(
    data: &FederalInterest,
    kept: &crate::federal_model::ChildStatementKept,
) -> Form1099IntSource {
    let mut value = Form1099IntSource {
        form: Form1099IntPaper {
            calendar_year: Default::default(),
            corrected: data.printed_facts.corrected,
            payer: kept.payer.clone(),
            payer_telephone: data.printed_facts.payer_telephone.clone(),
            payer_rtn: data.printed_facts.payer_rtn.clone(),
            recipient: kept.recipient.clone(),
            interest_income: data.taxable,
            early_withdrawal_penalty: data.early_penalty,
            us_savings_bond_and_treasury_interest: data.us_obligations,
            federal_income_tax_withheld: data.withholding,
            investment_expenses: data.investment_expenses,
            foreign_tax_paid: data.foreign_tax,
            foreign_country_or_us_territory: data.foreign_country.clone(),
            tax_exempt_interest: data.exempt,
            specified_private_activity_bond_interest: data.private_activity,
            market_discount: data.market_discount,
            bond_premium: data.bond_premium,
            bond_premium_on_treasury_obligations: data.treasury_bond_premium,
            bond_premium_on_tax_exempt_bond: data.tax_exempt_bond_premium,
            fatca_filing_requirement: data.printed_facts.fatca_filing_requirement,
            account_number: data.printed_facts.account_number.clone(),
            tax_exempt_and_tax_credit_bond_cusip_no: data
                .printed_facts
                .tax_exempt_and_tax_credit_bond_cusip_no
                .clone(),
            state_details: kept.state_details.clone(),
        },
        calls: crate::IntCalls::default(),
        nominee: data.nominee.clone().map(Into::into),
        adjustments: data.adjustments.clone(),
        after_death: data.after_death.clone().map(Into::into),
        accrued_interest: data.accrued_interest,
        special: data.special,
    };
    value.calls = int_calls(data.after_death.is_some(), &(&value).into());
    value
}

pub(super) fn seller_interest_record(v: &crate::SellerMortgageInterest) -> FederalInterest {
    FederalInterest {
        taxable: v.interest,
        seller_mortgage: Some(v.buyer.clone()),
        ..Default::default()
    }
}
