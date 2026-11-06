#[allow(unused_imports)]
use super::*;
// Payer-box preflights stay ahead of the canonical statement owner's adjustments.
pub(super) fn investment_statement_issue(document: &IncomeSource) -> Option<ValidationIssue> {
    investment_statement_input_issue(document, true)
}
#[allow(unused_variables)]
pub(super) fn investment_statement_input_issue(
    document: &IncomeSource,
    require_answers: bool,
) -> Option<ValidationIssue> {
    match document {
        // The paper alone carries no answer: its answers belong to the accepted source.
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Int1099(value),
        } => crate::investment_statements::int_printed_issue(value).map(project_interest_issue),
        IncomeSource::InterestDividend {
            value: crate::InterestDividendDocument::Div1099(value),
        } => crate::investment_statements::div_printed_issue(value).map(project_dividend_issue),
        // OID has no printed cross-box relationship independent of topic answers.
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Oid1099(_) } => {
            None
        }
        _ => None,
    }
}
pub(super) fn investment_final_return(draft: &FederalDraft) -> bool {
    draft.people.as_ref().is_some_and(|people| {
        people.taxpayer.died.is_some()
            || people.spouse.as_ref().is_some_and(|spouse| spouse.died.is_some())
    })
}

// The 1099-DIV rules over the paper and the answers kept beside it.
pub(super) fn dividend_source_issue(
    source: &DividendSource,
    require_answers: bool,
) -> Option<ValidationIssue> {
    let value = &source.form;
    if !require_answers {
        return crate::investment_statements::div_printed_issue(value).map(project_dividend_issue);
    }
    if let Some(issue) = crate::investment_statements::div_paper_issue(
        value,
        value.section199a.unwrap_or(0) != 0 && source.reit_eligible.is_none(),
        value.section_1202_gain.unwrap_or(0) != 0 && source.section_1202_exclusion.is_none(),
    ) {
        return Some(issue);
    }
    crate::investment_statements::div_issue(&FederalDividend::from(source))
        .map(project_dividend_issue)
}

// The 1099-INT rules over the paper and the answers kept beside it.
pub(super) fn int_source_issue(
    source: &Form1099IntSource,
    require_answers: bool,
) -> Option<ValidationIssue> {
    if let Some(issue) = crate::investment_statements::int_printed_issue(&source.form) {
        return Some(project_interest_issue(issue));
    }
    if !require_answers {
        return None;
    }
    crate::investment_statements::int_issue(&interest_record(source)).map(project_interest_issue)
}

// The 1099-OID rules over the paper and the answers kept beside it.
pub(super) fn oid_source_issue(
    source: &Form1099OidSource,
    require_answers: bool,
) -> Option<ValidationIssue> {
    // OID allocation and premium relationships need the topic's answers;
    // the decoded paper alone has no cross-box rule in this owner.
    if !require_answers {
        return None;
    }
    if source.private_activity > source.form.tax_exempt_oid {
        return Some(
            ValidationIssue::new("OID_PRIVATE_ACTIVITY_ABOVE_EXEMPT").at_field("private_activity"),
        );
    }
    crate::investment_statements::oid_issue(&oid_record(source)).map(project_oid_issue)
}
