#[allow(unused_imports)]
use super::*;

pub(super) use crate::federal_model::{patr_input_issue, patr_paper_input_issue};

// Pure mappings preserve printed facts and supplemental records separately.
pub(super) fn prepare_cooperative_source_ids(
    draft: &mut FederalDraft,
    accepted_draft: &FederalDraft,
) -> Result<(), (i64, ValidationIssue)> {
    for index in 0..draft.income.len() {
        let eligible =
            crate::cooperative::eligible_activities(draft, draft.income[index].owner.owned());
        let entry = &mut draft.income[index];
        let FederalIncome::Cooperative { data } = &mut entry.income else {
            continue;
        };
        if data.more.allocations.is_empty() {
            data.more
                .allocations
                .push(crate::cooperative::automatic(&crate::FederalCooperative::from(&*data)));
        }
        if eligible.len() == 1 {
            if data.more.credit_facts.activity.is_none() {
                data.more.credit_facts.activity = Some(eligible[0]);
            }
            for item in &mut data.more.credit_facts.other {
                if item.activity.is_none() {
                    item.activity = Some(eligible[0]);
                }
            }
        }
        let accepted =
            accepted_draft.income.iter().find(|e| e.id == entry.id).into_iter().flat_map(|e| {
                match &e.income {
                    FederalIncome::Cooperative { data } => {
                        data.more.allocations.iter().map(|a| a.id).collect::<Vec<_>>()
                    }
                    _ => Vec::new(),
                }
            });
        assign_record_ids(
            data.more.allocations.iter_mut().map(|a| &mut a.id),
            accepted,
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            (entry.id, ValidationIssue::new(key).at_list_item("allocations", index))
        })?;
        // Box 12 items are now topic answers: a new item gets its identity here,
        // as it did when the Document center entered it.
        for row in data.more.credit_facts.other.iter_mut().filter(|row| row.id == 0) {
            row.id = draft.next_id;
            draft.next_id = draft.next_id.checked_add(1).ok_or_else(|| {
                (
                    entry.id,
                    ValidationIssue::new(
                        "ISSUE_INTERVIEW_THE_RECORD_IDENTITY_LIMIT_HAS_BEEN_78BC135C",
                    ),
                )
            })?;
        }
    }
    Ok(())
}

pub(super) fn project_cooperative_issue(
    mut issue: ValidationIssue,
    automatic: bool,
) -> ValidationIssue {
    if automatic && issue.has_field_prefix(&["allocations"]) {
        issue = issue.at_field("allocations");
    }
    for field in ["allocations", "credit_facts"] {
        issue = issue.map_field_prefix(&[field], &["more", field]);
    }
    issue
        .map_field_prefix(&["more", "credit_facts", "payer_ein"], &["payer", "federal_id"])
        .map_field_prefix(&["more", "credit_facts", "investment"], &["investment"])
        .map_field_prefix(&["more", "credit_facts", "work_opportunity"], &["work_opportunity"])
}

impl Interview {
    pub(super) fn cooperative_activities(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| {
            crate::cooperative::eligible_activities(&self.draft, entry.owner.owned())
                .contains(&entry.id)
        })
    }
}
