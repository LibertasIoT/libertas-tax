// Savings-bond document and supporting-record validation.
#[allow(unused_imports)]
use super::*;

// The existing SavingsBonds source, amount and issue-date checks, in native order.
pub(super) fn savings_bonds_issue(v: &SavingsBonds) -> Option<ValidationIssue> {
    if v.interest_statement <= 0 {
        Some(ValidationIssue::new("SAVINGS_BONDS_SOURCE_INVALID").at_field("interest_statement"))
    } else if v.redemption_interest > v.proceeds {
        Some(
            ValidationIssue::new("SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS")
                .at_field("redemption_interest"),
        )
    } else if v.earliest_issue_date < crate::year_constants::SAVINGS_BOND_FIRST_ISSUE_DATE {
        Some(
            ValidationIssue::new("SAVINGS_BONDS_ISSUED_BEFORE_1990")
                .at_field("earliest_issue_date"),
        )
    } else if v.earliest_issue_date > crate::year_constants::TAX_YEAR_END_DATE {
        Some(
            ValidationIssue::new("SAVINGS_BONDS_ISSUED_AFTER_2026").at_field("earliest_issue_date"),
        )
    } else {
        None
    }
}

// Document validation does not require supporting records to be available.
pub(super) fn savings_bonds_entry_issue(
    entry: &crate::SavingsBondsEntry,
) -> Option<ValidationIssue> {
    if entry.form.line6 > entry.form.line5 {
        Some(
            ValidationIssue::new("SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS")
                .at_field("line6")
                .in_field("form"),
        )
    } else if entry.redemption_interest.is_some_and(|interest| interest > entry.form.line5) {
        Some(
            ValidationIssue::new("SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS")
                .at_field("redemption_interest"),
        )
    } else if entry
        .earliest_issue_date
        .is_some_and(|date| date < crate::year_constants::SAVINGS_BOND_FIRST_ISSUE_DATE)
    {
        Some(
            ValidationIssue::new("SAVINGS_BONDS_ISSUED_BEFORE_1990")
                .at_field("earliest_issue_date"),
        )
    } else if entry
        .earliest_issue_date
        .is_some_and(|date| date > crate::year_constants::TAX_YEAR_END_DATE)
    {
        Some(
            ValidationIssue::new("SAVINGS_BONDS_ISSUED_AFTER_2026").at_field("earliest_issue_date"),
        )
    } else {
        None
    }
}

// Unavailable redemption facts and personal records remain a pending source item.
pub(super) fn savings_bonds_source_issue(
    source: &crate::SavingsBondsSource,
    required: bool,
) -> Option<ValidationIssue> {
    let Some(natural) = source.natural() else {
        return required
            .then(|| ValidationIssue::new("DOCUMENT_SAVINGS_BONDS_RECORDS").at_field("value"));
    };
    savings_bonds_issue(&natural).map(|issue| issue.in_field("value"))
}

// Form 8818 is independently collectable; association belongs to the topic.
pub(super) fn bond_redemption_paper_issue(form: &crate::Form8818Paper) -> Option<ValidationIssue> {
    (form.line8 > form.line6)
        .then(|| ValidationIssue::new("SAVINGS_BONDS_INTEREST_ABOVE_PROCEEDS").at_field("line8"))
}
