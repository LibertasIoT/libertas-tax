use super::*;
use crate::DocumentAnswer as A;
use alloc::string::String;

type Question = (String, &'static str, A, Vec<i32>);

pub(super) fn questions(source: &crate::Patr1099) -> Vec<Question> {
    let facts = &source.more.credit_facts;
    let mut rows = Vec::new();
    rows.push((
        "credit_records".into(),
        "DOCUMENT_ANSWER_COOPERATIVECREDITRECORDS",
        A::CooperativeCreditRecords {
            investment_shares: facts.investment_shares.clone(),
            other: facts.other.clone(),
            participation: facts.participation,
            activity: facts.activity,
            notice: facts.notice.clone(),
        },
        Vec::new(),
    ));
    for row in &facts.other {
        rows.push((
            alloc::format!("credit_other/{}", row.id),
            "DOCUMENT_ANSWER_COOPERATIVEOTHERRECORDS",
            A::CooperativeOtherRecords {
                kind: row.kind.clone(),
                activity: row.activity,
                income: row.income.clone(),
            },
            Vec::new(),
        ));
    }
    rows
}

pub(super) fn description(source: &crate::Patr1099, field: &str) -> Option<Vec<u8>> {
    let row = source
        .more
        .credit_facts
        .other
        .iter()
        .find(|row| field == alloc::format!("credit_other/{}", row.id))?;
    Some(
        Message::new(
            "DOCUMENT_COOPERATIVE_OTHER_SOURCE",
            alloc::vec![MessageArg::Text(row.source.clone()), MessageArg::usd_cents(row.amount)],
        )
        .encode(),
    )
}

pub(super) fn apply(
    source: &mut crate::Patr1099Source,
    field: &str,
    answer: A,
) -> Result<(), ValidationIssue> {
    let invalid = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_record();
    let facts = &mut source.more.credit_facts;
    match answer {
        A::CooperativeCreditRecords {
            investment_shares,
            mut other,
            participation,
            activity,
            notice,
        } if field == "credit_records" => {
            for row in &mut other {
                let previous = facts.other.iter().find(|old| row.id > 0 && row.id == old.id);
                row.activity = previous.and_then(|old| old.activity);
                row.income =
                    previous.map_or(crate::CoopCreditIncome::Unresolved, |old| old.income.clone());
            }
            facts.investment_shares = investment_shares;
            facts.other = other.into_iter().map(Into::into).collect();
            facts.participation = participation;
            facts.activity = activity;
            facts.notice = notice;
        }
        A::CooperativeOtherRecords { activity, income, .. } => {
            let row = facts
                .other
                .iter_mut()
                .find(|row| field == alloc::format!("credit_other/{}", row.id))
                .ok_or_else(invalid)?;
            row.activity = activity;
            row.income = income;
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

pub(super) fn bind_issue(
    source: &crate::Patr1099,
    mut issue: ValidationIssue,
    submitted: &[(i64, String, usize)],
    id: i64,
) -> Option<SaveFailure> {
    if issue.has_field_prefix(&["more", "credit_facts"]) {
        issue = issue.map_field_prefix(&["more", "credit_facts"], &["credit_facts"]);
    }
    if issue.has_field_prefix(&["credit_facts", "other"]) {
        let personal: Vec<_> = source.more.credit_facts.other.iter().map(|_| true).collect();
        let key = issue.key;
        let failure = SaveFailure::from(issue.clone());
        // Bind per-item treatment independently from the editable credit breakdown.
        for leaf in ["activity", "income"] {
            let mapped = failure.clone().map_list_item_field_prefix(
                &["credit_facts", "other"],
                &[leaf],
                &[],
                &personal,
            );
            if mapped.field != failure.field {
                let targets: Vec<_> = source
                    .more
                    .credit_facts
                    .other
                    .iter()
                    .map(|row| {
                        let target = alloc::format!("income/{id}/credit_other/{}", row.id);
                        let index = submitted
                            .iter()
                            .find(|(found, name, _)| *found == id && *name == target)?
                            .2;
                        ValidationIssue::new(key)
                            .at_field(leaf)
                            .in_field("answer")
                            .in_list("followups", index)
                            .field
                    })
                    .collect();
                return Some(mapped.map_submitted_items(&["credit_facts", "other"], &targets));
            }
        }
        let key = alloc::format!("income/{id}/credit_records");
        let index = submitted.iter().find(|(found, name, _)| *found == id && *name == key)?.2;
        return Some(
            issue
                .map_field_prefix(&["credit_facts"], &[])
                .in_field("answer")
                .in_list("followups", index)
                .into(),
        );
    }
    if issue.has_field_prefix(&["credit_facts", "investment_shares"]) {
        let key = alloc::format!("income/{id}/credit_records");
        let index = submitted.iter().find(|(found, name, _)| *found == id && *name == key)?.2;
        return Some(
            issue
                .map_field_prefix(&["credit_facts"], &[])
                .in_field("answer")
                .in_list("followups", index)
                .into(),
        );
    }
    ["participation", "activity", "notice"]
        .into_iter()
        .find(|field| issue.has_field_prefix(&["credit_facts", field]))?;
    let key = alloc::format!("income/{id}/credit_records");
    let index = submitted.iter().find(|(found, name, _)| *found == id && *name == key)?.2;
    Some(
        issue
            .map_field_prefix(&["credit_facts"], &[])
            .in_field("answer")
            .in_list("followups", index)
            .into(),
    )
}
