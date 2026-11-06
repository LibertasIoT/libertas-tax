use super::Section179PreparedIssue;
use crate::{FederalDraft, FederalIncome, FederalResult};

// Reg.1.179-2(c)(1),(6)(iv): compensation is active-business income, without
// unreimbursed employee expenses. Section911 exclusions remain excluded from
// taxable income. The existing Form2555 owner supplies each annual-dollar
// exclusion and housing deduction; W-2 compensation itself is already counted.
pub(crate) fn income_adjustment(draft: &FederalDraft) -> Result<i64, Section179PreparedIssue> {
    use Section179PreparedIssue as E;
    let employee_source = |entry: &crate::FederalIncomeEntry| match &entry.income {
        // An actual zero-pay record still needs the native award/pay checks.
        FederalIncome::ForeignEmployment { .. } => true,
        FederalIncome::Wage { data } => {
            let data = &crate::FederalWage::from(data);
            data.abroad != 0
                && crate::extended_rules::foreign_claim(
                    draft,
                    usize::from(entry.owner == crate::DocumentOwner::Spouses),
                )
                .is_some()
        }
        _ => false,
    };
    let mut owners = [false; 2];
    for entry in &draft.income {
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        owners[owner] |= employee_source(entry);
    }
    if !owners.iter().any(|present| *present) {
        return Ok(0);
    }
    if draft.setup.is_none() {
        return Err(E::MissingReturnContext);
    }
    if let Some((owner, issue)) = crate::foreign_housing::source_pay_issue_for_owners(draft, owners)
    {
        let source = draft
            .income
            .iter()
            .find(|entry| {
                entry.owner == crate::DocumentOwner::from(owner) && employee_source(entry)
            })
            .ok_or(E::WageIncomeUnavailable)?;
        return Err(E::SourceBoundaryUnavailable(source.id, issue.key));
    }
    let mut output = FederalResult::empty(draft.revision);
    let foreign = crate::extended_rules::foreign_for_owners(draft, &mut output, owners);
    if !output.issues.is_empty() {
        return Err(E::OwnerIssues(output.issues));
    }
    let mut adjustment = 0i64;
    for (owner, present) in owners.into_iter().enumerate() {
        if !present {
            continue;
        }
        // Combined service income changes the person's current exclusion,
        // related deductions and housing. Its selected-expense/half-SE/HOME
        // source must be settled by the business owner before it can supply
        // this upstream income limit. Prior-service exclusion is independent.
        if foreign.elected[owner]
            && foreign.earned_wages[owner] != 0
            && let Some(source) = draft.income.iter().find(|entry| {
                usize::from(entry.owner == crate::DocumentOwner::Spouses) == owner
                    && matches!(&entry.income, FederalIncome::Business { data }
                        if matches!(data.source, crate::BusinessSource::ForeignServices { .. }))
            })
        {
            return Err(E::SourceBoundaryUnavailable(
                source.id,
                "ISSUE_RENTAL_FOREIGN_OR_OTHER_PROPERTY_INCOME_WITH_SECTION_179_NEEDS_ACTI_F72A67AD",
            ));
        }
        let amount = foreign.wages[owner]
            .checked_sub(foreign.excluded[owner])
            .and_then(|amount| amount.checked_sub(foreign.prior_excluded[owner]))
            .and_then(|amount| amount.checked_sub(foreign.housing_deduction[owner]))
            .and_then(|amount| amount.checked_mul(100))
            .ok_or(E::AggregateAmountOverflow)?;
        adjustment = adjustment.checked_add(amount).ok_or(E::AggregateAmountOverflow)?;
    }
    Ok(adjustment)
}

// A known employee source does not establish another business's taxable
// income. Preserve that source's own Form2555 coordination boundary when its
// current exclusion or housing deduction is part of the return income limit.
pub(super) fn business_exclusion_issue(draft: &FederalDraft) -> Option<Section179PreparedIssue> {
    draft.income.iter().find_map(|entry| {
        let FederalIncome::Business { data } = &entry.income else { return None };
        if !matches!(data.source, crate::BusinessSource::ForeignServices { .. }) {
            return None;
        }
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        let has_income = draft.income.iter().any(|source| {
            if source.owner != entry.owner {
                return false;
            }
            match &source.income {
                FederalIncome::ForeignEmployment { data } => data.wages != 0,
                FederalIncome::Wage { data } => crate::FederalWage::from(data).abroad != 0,
                FederalIncome::Business { data } => {
                    matches!(data.source, crate::BusinessSource::ForeignServices { .. })
                        && data.receipts() != 0
                }
                _ => false,
            }
        });
        // With no foreign gross income, the existing exclusion and housing
        // owners have no current or prior deduction capacity to coordinate.
        if !has_income {
            return None;
        }
        let claim = crate::extended_rules::foreign_claim(draft, owner)?;
        let barred = claim.revocation.barred();
        let current = claim.exclusion
            && !barred[0]
            && !matches!(claim.qualification, crate::ForeignQualification::PriorYearOnly);
        (current || matches!(claim.housing, crate::ForeignHousing::Claim { .. })).then_some(
            Section179PreparedIssue::SourceBoundaryUnavailable(
                entry.id,
                "ISSUE_RENTAL_FOREIGN_OR_OTHER_PROPERTY_INCOME_WITH_SECTION_179_NEEDS_ACTI_F72A67AD",
            ),
        )
    })
}
