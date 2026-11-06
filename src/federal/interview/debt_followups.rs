use super::*;
use crate::DocumentAnswer as A;

pub(super) fn apply(entry: &mut FederalIncomeEntry, answer: A) -> Result<(), ValidationIssue> {
    let changed = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value");
    match (&mut entry.income, answer) {
        (FederalIncome::PersonalDebt { data }, A::DebtTreatment { value }) => {
            data.treatment = value;
        }
        (FederalIncome::PersonalDebt { data }, A::DebtProperty { value }) => {
            if data.form.property_value.is_none() && value.is_some() {
                return Err(ValidationIssue::new("DEBT_PROPERTY_FMV_NEEDED").at_field("property"));
            }
            data.property = value;
        }
        (FederalIncome::PropertyAcquisition { data }, A::DebtAcquisition { value }) => {
            data.property = value.map(Into::into);
        }
        _ => return Err(changed()),
    }
    Ok(())
}

pub(crate) fn answer_issue(
    entry: &FederalIncomeEntry,
    year: u32,
    require_answers: bool,
) -> Option<ValidationIssue> {
    let missing = match &entry.income {
        FederalIncome::PersonalDebt { data } => {
            if let Some(issue) = data.issue().filter(|issue| {
                !matches!(issue.key, "DEBT_TREATMENT_NEEDED" | "DEBT_PROPERTY_NEEDED")
            }) {
                return Some(issue);
            }
            if data.treatment.is_none() {
                Some(ValidationIssue::new("DEBT_TREATMENT_NEEDED").at_field("treatment"))
            } else if data.form.property_value.is_some() && data.property.is_none() {
                Some(ValidationIssue::new("DEBT_PROPERTY_NEEDED").at_field("property"))
            } else {
                None
            }
        }
        FederalIncome::PropertyAcquisition { data } if data.property.is_none() => {
            Some(ValidationIssue::new("DEBT_ACQUISITION_PROPERTY_NEEDED").at_field("property"))
        }
        FederalIncome::PropertyAcquisition { .. } => None,
        _ => return None,
    };
    if missing.is_some() {
        return if require_answers { missing } else { None };
    }
    crate::federal_rules::validate_income_issue_for_year_with_mode(entry, year, require_answers)
        .err()
        .map(|(issue, _)| issue)
}

pub(super) fn bind_issue(
    id: i64,
    issue: ValidationIssue,
    submitted: &[(i64, String, usize)],
    acquisition: bool,
) -> Option<SaveFailure> {
    let (suffix, from) = if acquisition {
        ("debt_acquisition", &["property"][..])
    } else if issue.has_field_prefix(&["treatment"]) {
        ("debt_treatment", &["treatment"][..])
    } else {
        ("debt_property", &["property"][..])
    };
    let key = alloc::format!("income/{id}/{suffix}");
    let index = submitted
        .iter()
        .find(|(_, found, _)| *found == key)
        .or_else(|| submitted.iter().find(|(found, _, _)| *found == id))?
        .2;
    Some(
        issue
            .map_field_prefix(from, &["value"])
            .in_field("answer")
            .in_list("followups", index)
            .into(),
    )
}
