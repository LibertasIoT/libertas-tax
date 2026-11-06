#[allow(unused_imports)]
use super::*;
// Box 8 repayments are reconciled with the recipient's personal records.
pub(super) fn rrb_issue(v: &crate::RailroadSource) -> Option<ValidationIssue> {
    rrb_topic_issue(v, true)
}
pub(crate) fn railroad_answer_issue(data: &crate::RetirementSource) -> Option<ValidationIssue> {
    match data {
        crate::RetirementSource::Rrb1099R(value) => rrb_personal_issue(value, true),
        crate::RetirementSource::R1099(_) => None,
    }
}
pub(super) fn ssa_paper_issue(v: &crate::Ssa1099Paper) -> Option<ValidationIssue> {
    crate::ssa_1099_model::net_benefits_values_issue(
        v.benefits_paid.unwrap_or(0),
        v.benefits_repaid_to_ssa.unwrap_or(0),
        v.net_benefits,
    )
    .or_else(|| {
        (!crate::federal_rules::social_printed_amounts_valid(
            v.net_benefits,
            v.voluntary_federal_income_tax_withheld.unwrap_or(0),
        ))
        .then(retirement_printed_amount_issue)
    })
}

pub(super) fn rrb_paper_issue(v: &crate::FormRrb1099RPaper) -> Option<ValidationIssue> {
    rrb_source_fields_issue(
        v.box_3,
        v.box_4.unwrap_or(0),
        v.box_5.unwrap_or(0),
        v.box_6.unwrap_or(0),
        v.gross,
        v.box_8.unwrap_or(0),
        v.withholding.unwrap_or(0),
    )
    .or_else(|| {
        (!crate::federal_rules::amounts(&[
            v.gross,
            v.withholding.unwrap_or(0),
            v.box_5.unwrap_or(0),
            v.box_6.unwrap_or(0),
        ]))
        .then(retirement_printed_amount_issue)
    })
}

pub(super) fn rrb_source_issue(v: &crate::Rrb1099R) -> Option<ValidationIssue> {
    rrb_source_fields_issue(
        v.box_3,
        v.box_4.unwrap_or(0),
        v.box_5.unwrap_or(0),
        v.box_6.unwrap_or(0),
        v.gross,
        v.box_8.unwrap_or(0),
        v.withholding.unwrap_or(0),
    )
}

fn rrb_source_fields_issue(
    box_3: Option<i64>,
    box_4: i64,
    box_5: i64,
    box_6: i64,
    gross: i64,
    box_8: i64,
    withholding: i64,
) -> Option<ValidationIssue> {
    for (field, amount) in [
        ("box_4", box_4),
        ("box_5", box_5),
        ("box_6", box_6),
        ("gross", gross),
        ("box_8", box_8),
        ("withholding", withholding),
    ] {
        if amount < 0 {
            return Some(ValidationIssue::new("RRB_SOURCE_NONNEGATIVE").at_field(field));
        }
    }
    if box_3.is_some_and(|value| value < 0) {
        return Some(ValidationIssue::new("RRB_SOURCE_NONNEGATIVE").at_field("box_3"));
    }
    if i128::from(box_4) + i128::from(box_5) + i128::from(box_6) != i128::from(gross) {
        Some(ValidationIssue::new("RRB_BOX7_NOT_SUM").at_field("gross"))
    } else if withholding > gross {
        Some(ValidationIssue::new("R_WITHHOLDING_ABOVE_GROSS").at_field("withholding"))
    } else {
        None
    }
}
pub(super) fn rrb_topic_issue(
    v: &crate::RailroadSource,
    require_answers: bool,
) -> Option<ValidationIssue> {
    if let Some(issue) = rrb_source_issue(v) {
        return Some(issue);
    }
    rrb_personal_issue(v, require_answers)
}
// The Simplified Method age comes from the owner's date of birth: refused while that date cannot give it.
pub(super) fn rrb_birth_issue(
    v: &crate::RailroadSource,
    birth: Option<u32>,
) -> Option<ValidationIssue> {
    match v.recipient_kind.as_ref()? {
        crate::RailroadRecipient::Employee {
            recovery: crate::RailroadRecovery::SimplifiedMethod { data },
        }
        | crate::RailroadRecipient::Survivor {
            recovery: crate::RailroadRecovery::SimplifiedMethod { data },
            ..
        } if data.lacks_birth(birth) => Some(
            ValidationIssue::new("RRB_ANNUITANT_BIRTH_NEEDED")
                .at_field("age")
                .in_field("duration")
                .in_field("history")
                .in_field("data")
                .in_field("recovery")
                .in_field("recipient_kind"),
        ),
        _ => None,
    }
}
pub(super) fn rrb_personal_issue(
    v: &crate::RailroadSource,
    require_answers: bool,
) -> Option<ValidationIssue> {
    if require_answers && v.recipient_kind.is_none() {
        return Some(ValidationIssue::new("RRB_RECIPIENT_NEEDED").at_field("recipient_kind"));
    }
    if let Some(
        crate::RailroadRecipient::Employee {
            recovery: crate::RailroadRecovery::SimplifiedMethod { data },
        }
        | crate::RailroadRecipient::Survivor {
            recovery: crate::RailroadRecovery::SimplifiedMethod { data },
            ..
        },
    ) = &v.recipient_kind
        && data.cost.is_none()
        && !(v.box_3.is_some() && data.started / 10000 == 2026)
        && require_answers
    {
        return Some(
            ValidationIssue::new("RRB_RECOVERY_COST_NEEDED")
                .at_field("cost")
                .in_field("data")
                .in_field("recovery")
                .in_field("recipient_kind"),
        );
    }
    if !require_answers && v.repayments.is_empty() {
        return None;
    }
    let rows: i128 = v.repayments.iter().map(|p| i128::from(p.amount)).sum();
    if v.box_8.unwrap_or(0) == 0 && !v.repayments.is_empty() {
        Some(ValidationIssue::new("RRB_REPAYMENTS_WITHOUT_BOX_8").at_field("repayments"))
    } else if v.box_8.unwrap_or(0) > 0 && v.repayments.is_empty() {
        Some(ValidationIssue::new("RRB_BOX_8_NOT_SPLIT").at_field("repayments"))
    } else if rows != i128::from(v.box_8.unwrap_or(0)) {
        Some(ValidationIssue::new("RRB_REPAYMENTS_NOT_BOX_8").at_field("repayments"))
    } else {
        None
    }
}
