#[allow(unused_imports)]
use super::*;

// Ordered W-2 input validation. The existing request walkers prepend their
// submitted document position; these paths are local to the actual W2 value.
pub(super) fn wage_reference_issue(
    draft: &FederalDraft,
    item: &IncomeDocumentItem,
) -> Option<ValidationIssue> {
    let IncomeSource::Wage { value: crate::WageDocument::W2(wage) } = &item.document else {
        return None;
    };
    let answers = draft.income.iter().find(|entry| entry.id == item.id).and_then(|entry| {
        match &entry.income {
            FederalIncome::Wage { data } => data.answers(),
            _ => None,
        }
    })?;
    if answers.paying_scorporation.as_ref().is_some_and(|source| {
        !draft.income.iter().any(|entry| {
            entry.id == source.id()
                && matches!(
                    entry.income,
                    FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. }
                )
        })
    }) {
        return Some(
            ValidationIssue::new("W2_SCORP_EMPLOYER_UNAVAILABLE")
                .at_field("source")
                .in_field("paying_scorporation")
                .in_field("answers"),
        );
    }
    if wage.statutory
        && let Some(crate::StatutoryBusiness::Business { source }) = answers.statutory_business
    {
        let key = match draft.income.iter().find(|entry| entry.id == source) {
            None => Some("ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME"),
            Some(target) if target.owner != item.owner => {
                Some("ISSUE_STATUTORY_BUSINESS_OTHER_OWNER")
            }
            Some(target) if !matches!(target.income, FederalIncome::Business { .. }) => {
                Some("ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME")
            }
            _ => None,
        };
        if let Some(key) = key {
            return Some(
                ValidationIssue::new(key).at_field("statutory_business").in_field("answers"),
            );
        }
    }
    None
}

pub(super) fn w2_input_issue(w: &crate::FormW2Paper) -> Option<ValidationIssue> {
    if w.withholding.unwrap_or(0) > w.wages.unwrap_or(0) {
        return Some(ValidationIssue::new("W2_WITHHOLDING_ABOVE_WAGES").at_field("withholding"));
    }
    if w.social_wages
        .unwrap_or(0)
        .checked_add(w.social_tips.unwrap_or(0))
        .is_none_or(|combined| w.social_tax.unwrap_or(0) > combined)
    {
        return Some(ValidationIssue::new("W2_SOCIAL_TAX_ABOVE_WAGES").at_field("social_tax"));
    }
    if w.medicare_tax.unwrap_or(0) > w.medicare_wages.unwrap_or(0) {
        return Some(ValidationIssue::new("W2_MEDICARE_TAX_ABOVE_WAGES").at_field("medicare_tax"));
    }
    if w2_social_wages_and_tips_above_base(w) {
        let issue = ValidationIssue::new("W2_SOCIAL_WAGES_AND_TIPS_ABOVE_BASE");
        return Some(
            if w.social_wages.unwrap_or(0) > crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS
            {
                issue.at_field("social_wages")
            } else {
                issue.at_field("social_tips")
            },
        );
    }
    if let Some(index) = w.box12.iter().position(|e| {
        e.code == W2Code::TA
            && e.amount.unwrap_or(0) > crate::year_constants::TRUMP_EMPLOYER_EXCLUSION_CENTS
    }) {
        return Some(
            ValidationIssue::new("W2_TA_ABOVE_LIMIT").at_field("amount").in_list("box12", index),
        );
    }
    if let Some(index) = w2_box12_duplicate_index(&w.native().more.box12) {
        return Some(
            ValidationIssue::new("W2_BOX12_CODE_TWICE").at_field("code").in_list("box12", index),
        );
    }
    None
}

pub(super) fn w2_personal_amount_issue(source: &crate::W2Source) -> Option<ValidationIssue> {
    let w = &source.form;
    let answers = &source.answers;
    if w.statutory && W2Amount::cents(answers.abroad) != 0 {
        return Some(
            ValidationIssue::new("W2_STATUTORY_ABROAD_USE_BUSINESS")
                .at_field("abroad")
                .in_field("answers"),
        );
    }
    let abroad = W2Amount::cents(answers.abroad);
    let deferred_pension = W2Amount::cents(answers.deferred_pension);
    if abroad > w.wages.unwrap_or(0) {
        return Some(
            ValidationIssue::new("W2_ABROAD_ABOVE_WAGES").at_field("abroad").in_field("answers"),
        );
    }
    if i128::from(deferred_pension)
        + i128::from(answers.inmate_pay)
        + i128::from(answers.medicaid_waiver)
        > i128::from(w.wages.unwrap_or(0))
    {
        // Locate the first original-order contributor crossing this existing
        // total bound. This selects metadata only; it does not change the gate.
        let field = if deferred_pension > w.wages.unwrap_or(0) {
            "deferred_pension"
        } else if i128::from(deferred_pension) + i128::from(answers.inmate_pay)
            > i128::from(w.wages.unwrap_or(0))
        {
            "inmate_pay"
        } else {
            "medicaid_waiver"
        };
        return Some(
            ValidationIssue::new("W2_SCHEDULE_1_PAY_ABOVE_WAGES")
                .at_field(field)
                .in_field("answers"),
        );
    }
    if answers.disability.as_ref().is_some_and(|value| value.amount() > w.wages.unwrap_or(0)) {
        return Some(
            ValidationIssue::new(
                "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            )
            .at_field("disability")
            .in_field("answers"),
        );
    }
    None
}

// The two W-2 box relationships the server re-checks beside their client rules:
// box 3 plus box 7 tips cannot exceed the Social Security wage base, and a box
// 12 code appears once.
pub(super) fn w2_social_wages_and_tips_above_base(w: &crate::FormW2Paper) -> bool {
    w.social_wages
        .unwrap_or(0)
        .checked_add(w.social_tips.unwrap_or(0))
        .is_none_or(|combined| combined > crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS)
}
// The follow-up answers a W-2's boxes and the return call for; an empty list
// keeps that answer's row out of the editor (the answers' variant indexes are
// their declaration order).
pub(super) fn w2_calls(draft: &FederalDraft, w: &crate::W2Boxes) -> crate::W2Calls {
    let open = |called: bool, choices: &[i32]| if called { choices.to_vec() } else { Vec::new() };
    let code = |wanted| w.more.box12.iter().any(|entry| entry.code == wanted);
    // Form W-2 box 7 (Social Security tips), box 8 (allocated tips) and box 12
    // code TP (qualified tips) show a tipped job. Form 4137 and Pub. 531 take
    // the daily record and noncash tips from it, and the qualified-tips
    // deduction (Schedule 1-A) takes the occupation answer.
    let tipped = w.more.social_tips > 0 || w.more.allocated_tips > 0 || code(W2Code::TP);
    // Form W-2 instructions: an exempt church employee's or minister's pay, and
    // railroad retirement pay (RRTA amounts in box 14a), are in box 1 with
    // boxes 3 and 5 blank (Pub. 517; Form W-2 instructions, box 14).
    let church = w.wages > 0 && w.social_wages == 0 && w.medicare_wages == 0;
    // Schedule R (Form 1040 instructions, line 1h): disability retirement pay
    // is claimed only by a filer who retired on disability.
    let disabled = draft
        .people
        .as_ref()
        .is_some_and(|p| p.taxpayer.disabled || p.spouse.as_ref().is_some_and(|s| s.disabled));
    // Form 2555: W-2 pay for work abroad is read for a person with a Form 2555
    // record; a statutory employee marks the services on the Schedule C.
    let foreign = draft.screening.as_ref().is_some_and(|s| !s.foreign_earned.is_empty());
    crate::W2Calls {
        tips: open(tipped, &[0]),
        occupation: open(tipped, &[0, 1]),
        overtime: open(code(W2Code::TT), &[0, 1]),
        coverage: open(church, &[1, 2, 3]),
        disability: open(disabled, &[1]),
        abroad: open(foreign && !w.more.statutory, &[0]),
        // Form W-2 box 11 shows a distribution from a nonqualified deferred
        // compensation or nongovernmental section 457(b) plan; its taxable part
        // is Schedule 1 line 8t (Form 1040 instructions).
        deferred_pension: open(w.state_details.nonqualified_plans > 0, &[0]),
        // Schedule C instructions, line 1: a statutory employee (box 13) reports
        // box 1 on Schedule C.
        statutory_business: open(w.more.statutory, &[0]),
    }
}
pub(super) fn w2_box12_duplicate_index(box12: &[Box12Entry]) -> Option<usize> {
    box12.iter().enumerate().find_map(|(i, entry)| {
        box12[..i].iter().any(|earlier| earlier.code == entry.code).then_some(i)
    })
}

// The W-2s a Form 2106 can choose: every wage record of the return, labelled
// by its employer and owner.
pub(super) fn employee_wage_choices(d: &FederalDraft) -> Vec<crate::WageChoice> {
    d.income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
        .map(|entry| crate::WageChoice {
            id: entry.id,
            label: crate::Message::new(
                "EMPLOYEE_EXPENSES_EMPLOYER_CHOICE",
                alloc::vec![
                    crate::MessageArg::Nested(alloc::boxed::Box::new(
                        income_issue_context(entry).label()
                    )),
                    crate::MessageArg::Resource(
                        if entry.owner == crate::DocumentOwner::Yours {
                            "PERSON_YOU"
                        } else {
                            "PERSON_SPOUSE"
                        }
                        .into()
                    ),
                ],
            )
            .encode(),
            owner: entry.owner.owned(),
        })
        .collect()
}
// The box 7 code onto `RetirementTreatment`. `gross` supplies code G's
// rollover amount (a direct rollover transfers the whole distribution).
// The HSA page (M3-P7) is a projection of FederalAdjustments.hsa and
// hsa_family_share. The shared employer-source owner allocates W-2 R/W by
// actual contribution year; the account's editable amount is outside sources.
pub(crate) fn box12_total(d: &FederalDraft, owner: Owner, code: W2Code) -> i64 {
    d.income
        .iter()
        .filter(|e| e.owner == crate::DocumentOwner::from(owner))
        .map(|e| match &e.income {
            FederalIncome::Wage { data } => {
                let data = crate::FederalWage::from(data);
                data.other_box12.iter().filter(|c| c.code == code).map(|c| c.amount).sum()
            }
            _ => 0,
        })
        .sum()
}
pub(crate) fn wage_answer_issue(entry: &FederalIncomeEntry) -> Option<ValidationIssue> {
    let FederalIncome::Wage { data: crate::WageSource::W2(source) } = &entry.income else {
        return None;
    };
    let w = &source.form;
    let answers = &source.answers;
    if w.statutory && W2Amount::cents(answers.abroad) != 0 {
        return Some(
            ValidationIssue::new("W2_STATUTORY_ABROAD_USE_BUSINESS")
                .at_field("abroad")
                .in_field("answers"),
        );
    }
    if (w.box12.iter().any(|entry| entry.code == W2Code::TP)
        || w.allocated_tips.unwrap_or(0) > 0
        || !answers.tip_months().is_empty())
        && answers.tips_listed_occupation.is_none()
    {
        return Some(
            ValidationIssue::new("W2_TP_OCCUPATION_NEEDED")
                .at_field("tips_listed_occupation")
                .in_field("answers"),
        );
    }
    if w.box12.iter().any(|entry| entry.code == W2Code::TT)
        && answers.overtime_flsa_required.is_none()
    {
        return Some(
            ValidationIssue::new("W2_TT_FLSA_NEEDED")
                .at_field("overtime_flsa_required")
                .in_field("answers"),
        );
    }
    if answers.box14.iter().any(|meaning| meaning.kind.is_none()) {
        return Some(
            ValidationIssue::new("W2_BOX14_KIND_NEEDED").at_field("box14").in_field("answers"),
        );
    }
    if answers.box14.iter().any(|meaning| meaning.asks()) {
        return Some(
            ValidationIssue::new("W2_BOX14_STATE_NEEDED").at_field("box14").in_field("answers"),
        );
    }
    w2_personal_amount_issue(source)
}

impl Interview {
    pub(super) fn wage_scorporations(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| {
            matches!(
                entry.income,
                FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. }
            )
        })
    }
    pub(super) fn allowed_wage_scorporations(&self) -> Vec<i32> {
        if self.draft.income.iter().any(|entry| {
            matches!(
                entry.income,
                FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. }
            )
        }) {
            alloc::vec![0]
        } else {
            Vec::new()
        }
    }
}
