use crate::tests::*;

// Tests mutate expense fixtures explicitly; production editors construct the selected branch.
pub(in crate::tests) fn itemized(d: &mut FederalDeductions) -> &mut ItemizedExpenses {
    if matches!(
        d.choice,
        DeductionSelection::Standard
            | DeductionSelection::Automatic { expenses: ItemizedChoice::No }
    ) {
        d.choice = DeductionSelection::Automatic {
            expenses: ItemizedChoice::Yes {
                facts: ItemizedExpenses {
                    income_tax_owners: None,
                    investment_property_tax_owners: None,
                    investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
                    real_estate_taxes: 0,
                    other_lines: vec![],
                    long_term_care: vec![],
                    medical: 0,
                    personal_property_taxes: 0,
                    personal_tax: PersonalTaxElection::None,
                    investment_property_taxes: 0,
                },
            },
        };
    }
    match &mut d.choice {
        DeductionSelection::Automatic { expenses: ItemizedChoice::Yes { facts } }
        | DeductionSelection::Itemized { facts } => facts,
        _ => unreachable!(),
    }
}
