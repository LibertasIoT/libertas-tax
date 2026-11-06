use super::*;
use crate::{DocumentAnswer as A, EstateAmtTreatment as T};
use alloc::string::String;

type Question = (String, &'static str, A, Vec<i32>);

fn label(value: &str) -> Vec<u8> {
    Message::new("TEXT", alloc::vec![MessageArg::Text(value.into())]).encode()
}

pub(super) fn catalog(
    draft: &FederalDraft,
    source: &crate::K1EstateTrust,
) -> crate::K1QuestionChoices {
    crate::K1QuestionChoices {
        estate_activities: crate::estate_trust::activity_choices(&source.record().activities),
        estate_mortgages: mortgage_choices(draft),
        ..Default::default()
    }
}

pub(super) fn mortgage_choices(draft: &FederalDraft) -> Vec<crate::EstateSourceChoice> {
    draft
        .deductions
        .iter()
        .flat_map(|d| &d.mortgage)
        .filter_map(|source| crate::MortgageLoan::try_from(source).ok())
        .filter(|loan| matches!(loan.details, crate::MortgageLoanDetails::FirstLender(_)))
        .map(|loan| crate::EstateSourceChoice {
            id: loan.id,
            label: Message::new(
                "MORTGAGE_SAVED_LOAN",
                alloc::vec![
                    MessageArg::Number(loan.id),
                    if loan.property.is_empty() {
                        MessageArg::Resource("MORTGAGE_PROPERTY_UNNAMED".into())
                    } else {
                        MessageArg::Text(loan.property.clone())
                    },
                    MessageArg::DateOnly(i64::from(loan.incurred))
                ],
            )
            .encode(),
        })
        .collect()
}

fn needs_final(source: &crate::K1EstateTrust) -> bool {
    source.final_character.is_some()
        || !matches!(source.more.final_year, crate::K1FinalYear::NotFinal)
        || source.successor.is_some()
        || !source.final_mortgages.is_empty()
}

pub(super) fn questions(source: &crate::K1EstateTrust) -> Vec<Question> {
    let mut rows = Vec::new();
    if let crate::K1EstateTrustBusiness::Amounts(activities) = &source.business {
        for activity in &activities.activities {
            rows.push((
                alloc::format!("estate/activity/{}", activity.id),
                "DOCUMENT_ESTATE_ACTIVITY_RECORDS",
                A::EstateActivityRecords {
                    description: label(&activity.name),
                    participation: activity.participation.clone(),
                    carried: activity.carried.clone(),
                },
                Vec::new(),
            ));
        }
    }
    for statement in &source.qbi.statements {
        rows.push((
            alloc::format!("estate/qbi/{}", statement.id),
            "DOCUMENT_ESTATE_QBI_HISTORY",
            A::EstateQbiHistory {
                description: label(&statement.name),
                activities: statement.activities.clone(),
                passive_history: statement.passive_history.clone(),
                allocations: statement
                    .allocations
                    .iter()
                    .map(|part| crate::EstateQbiAllocationHistory {
                        activity: part.activity,
                        income: part.income,
                        passive_history: part.passive_history.clone(),
                    })
                    .collect(),
            },
            Vec::new(),
        ));
    }
    if let Some(amt) = &source.amt_reconciliation {
        let native = amt.record();
        for adjustment in &native.adjustments {
            let code = match adjustment.code {
                crate::EstateAmtCode::DepreciationG => "G",
                crate::EstateAmtCode::DepletionH => "H",
                crate::EstateAmtCode::AmortizationI => "I",
            };
            rows.push((
                alloc::format!("estate/amt/{}", adjustment.id),
                "DOCUMENT_ESTATE_AMT_TREATMENT",
                A::EstateAmtTreatment {
                    description: Message::new(
                        "DOCUMENT_ESTATE_AMT_AMOUNT",
                        alloc::vec![
                            MessageArg::Text(code.into()),
                            MessageArg::usd_cents(adjustment.amount)
                        ],
                    )
                    .encode(),
                    treatment: adjustment.treatment.clone().into(),
                },
                Vec::new(),
            ));
        }
    }
    if !source.investment_property.is_empty() {
        let properties: Vec<_> =
            source.investment_property.iter().map(|item| item.property.clone()).collect();
        let descriptions =
            crate::K1ActivityChoiceContext::from_source_properties(None, &properties, &[])
                .properties;
        rows.push((
            "investment".into(),
            "DOCUMENT_K1_OWN_INVESTMENT_CREDIT",
            A::EstateInvestmentRecords {
                allowed_choices: Vec::new(),
                properties: properties
                    .iter()
                    .map(|property| crate::K1InvestmentAnswer {
                        id: property.id(),
                        own_form: property.own_form().cloned(),
                        description: descriptions
                            .iter()
                            .find(|choice| choice.id == property.id())
                            .map(|choice| choice.label.clone())
                            .unwrap_or_default(),
                    })
                    .collect(),
            },
            Vec::new(),
        ));
    }
    if needs_final(source) {
        rows.push((
            "estate/successor".into(),
            "DOCUMENT_ESTATE_FINAL_SUCCESSOR",
            A::EstateFinalSuccessor { value: source.successor },
            Vec::new(),
        ));
        rows.push((
            "estate/final".into(),
            "DOCUMENT_ESTATE_FINAL_RECORDS",
            A::EstateFinalRecords { mortgages: source.final_mortgages.clone() },
            Vec::new(),
        ));
    }
    if source.more.section199a_dividends > 0 || source.more.section199a_eligible.is_some() {
        rows.push((
            "estate/reit".into(),
            "DOCUMENT_ESTATE_REIT_ELIGIBILITY",
            A::EstateReitEligibility { value: source.more.section199a_eligible },
            Vec::new(),
        ));
    }
    rows
}

pub(super) fn apply(
    source: &mut crate::K1EstateTrust,
    suffix: &str,
    answer: A,
) -> Result<(), ValidationIssue> {
    let invalid = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value");
    match answer {
        A::EstateActivityRecords { participation, carried, .. } => {
            let crate::K1EstateTrustBusiness::Amounts(activities) = &mut source.business else {
                return Err(invalid());
            };
            let row = activities
                .activities
                .iter_mut()
                .find(|row| suffix == alloc::format!("estate/activity/{}", row.id))
                .ok_or_else(invalid)?;
            row.participation = participation;
            row.carried = carried;
        }
        A::EstateQbiHistory { activities, passive_history, allocations, .. } => {
            let row = source
                .qbi
                .statements
                .iter_mut()
                .find(|row| suffix == alloc::format!("estate/qbi/{}", row.id))
                .ok_or_else(invalid)?;
            row.activities = activities;
            row.passive_history = passive_history;
            row.allocations = allocations
                .into_iter()
                .map(|part| crate::EstateQbiAllocationPage {
                    activity: part.activity,
                    income: part.income,
                    passive_history: part.passive_history,
                })
                .collect();
        }
        A::EstateAmtTreatment { treatment, .. } => {
            let amt = source.amt_reconciliation.as_mut().ok_or_else(invalid)?;
            let row = amt
                .adjustments
                .iter_mut()
                .find(|row| suffix == alloc::format!("estate/amt/{}", row.id))
                .ok_or_else(invalid)?;
            row.treatment = crate::EstateAmtTreatmentPage::from_record(&treatment.into());
        }
        A::EstateInvestmentRecords { properties, .. } if suffix == "investment" => {
            let mut native: Vec<_> =
                source.investment_property.iter().map(|item| item.property.clone()).collect();
            super::k1_followups::patch_investment_properties(&mut native, properties)?;
            for (item, property) in source.investment_property.iter_mut().zip(native) {
                item.property = property;
            }
        }
        A::EstateFinalSuccessor { value } if suffix == "estate/successor" => {
            source.successor = value
        }
        A::EstateFinalRecords { mortgages } if suffix == "estate/final" => {
            source.final_mortgages = mortgages;
        }
        A::EstateReitEligibility { value } if suffix == "estate/reit" => {
            source.more.section199a_eligible = value
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

pub(crate) fn retain(
    source: &mut crate::EstateTrustStatements,
    accepted: &crate::EstateTrustStatements,
) {
    if let ReitDividends::Reported { eligible, .. } = &mut source.section199a_dividends {
        *eligible = match &accepted.section199a_dividends {
            ReitDividends::Reported { eligible, .. } => *eligible,
            ReitDividends::None => None,
        };
    }
    for activity in &mut source.activities {
        let old = accepted.activities.iter().find(|old| activity.id > 0 && old.id == activity.id);
        activity.participation =
            old.map_or(EstateParticipation::Unresolved, |old| old.participation.clone());
        activity.carried = old
            .map_or(EstatePassiveCarry { regular: 0, alternative: 0 }, |old| old.carried.clone());
    }
    for statement in &mut source.qbi.statements {
        let old =
            accepted.qbi.statements.iter().find(|old| statement.id > 0 && old.id == statement.id);
        statement.activities = old.map_or_else(Vec::new, |old| old.activities.clone());
        statement.activities.retain(|id| source.activities.iter().any(|row| row.id == *id));
        statement.allocations = old.map_or_else(Vec::new, |old| old.allocations.clone());
        statement.allocations.retain(|part| statement.activities.contains(&part.activity));
        // A corrected printed amount leaves a retained apportionment that no
        // longer sums to it: the apportionment, with its prior-loss histories,
        // is removed (the Document center save warns, removed_apportionments)
        // and the topic asks it again (ESTATE_QBI_APPORTIONMENT_NEEDED).
        if statement.allocations.iter().map(|part| i128::from(part.income)).sum::<i128>()
            != i128::from(statement.income)
        {
            statement.allocations.clear();
        }
        statement.passive_history = old.map_or_else(Vec::new, |old| old.passive_history.clone());
        for part in &mut statement.allocations {
            part.passive_history = old
                .and_then(|old| old.allocations.iter().find(|old| old.activity == part.activity))
                .map_or_else(Vec::new, |old| old.passive_history.clone());
        }
    }
    // A corrected printed service or kind leaves statements apportioned to one
    // activity disagreeing: each such statement's apportionment goes the same way.
    let conflicting: Vec<i64> = source
        .activities
        .iter()
        .map(|activity| activity.id)
        .filter(|id| source.qbi.kind_conflict(*id).is_some())
        .collect();
    for statement in &mut source.qbi.statements {
        if statement.allocations.iter().any(|part| conflicting.contains(&part.activity)) {
            statement.allocations.clear();
        }
    }
    if let Some(amt) = &mut source.amt_reconciliation {
        for adjustment in &mut amt.adjustments {
            adjustment.treatment = accepted
                .amt_reconciliation
                .as_ref()
                .and_then(|old| {
                    old.adjustments.iter().find(|old| adjustment.id > 0 && old.id == adjustment.id)
                })
                .map_or(EstateAmtTreatment::Unavailable, |old| old.treatment.clone());
        }
    }
    for item in &mut source.investment_property {
        let old = accepted
            .investment_property
            .iter()
            .find(|old| item.property.id() > 0 && old.property.id() == item.property.id());
        *item.property.own_form_mut() = old.and_then(|old| old.property.own_form().cloned());
    }
}

// The statement names whose retained apportionment a Document center save
// removed (retain): the save's warning is this diff.
pub(super) fn removed_apportionments<'a>(
    before: &'a crate::EstateTrustStatements,
    after: &'a crate::EstateTrustStatements,
) -> impl Iterator<Item = &'a str> {
    before.qbi.statements.iter().filter_map(|old| {
        after
            .qbi
            .statements
            .iter()
            .find(|new| old.id > 0 && new.id == old.id)
            .filter(|new| !old.allocations.is_empty() && new.allocations != old.allocations)
            .map(|new| new.name.as_str())
    })
}

// The K-1 topic's answer for the first statement whose apportionment is
// required and absent (ESTATE_QBI_APPORTIONMENT_NEEDED): its allocations.
pub(super) fn apportionment_path(draft: &FederalDraft, id: i64) -> Option<String> {
    let FederalIncome::EstateTrustPortfolio { data } =
        &draft.income.iter().find(|entry| entry.id == id)?.income
    else {
        return None;
    };
    let data = FederalEstateTrustPortfolioK1::from(data);
    let statement = &data.qbi.statements[data.qbi.apportionment_needed(&data.activities)?];
    let key = alloc::format!("income/{id}/estate/qbi/{}", statement.id);
    ValidationIssue::new("ESTATE_QBI_APPORTIONMENT_NEEDED")
        .at_field("allocations")
        .in_field("answer")
        .in_key("followups", key.as_str())
        .field
        .map(|field| field.square())
}

pub(super) fn answer_issue(
    data: &FederalEstateTrustPortfolioK1,
    require_answers: bool,
) -> Option<ValidationIssue> {
    let provided = data
        .activities
        .issue_with_mode(true)
        .map(|issue| issue.in_field("business"))
        .or_else(|| {
            data.qbi.issue_with_mode(&data.activities, true).map(|issue| issue.in_field("qbi"))
        })
        .or_else(|| crate::estate_trust::source_issue_with_mode(data, true));
    if provided.is_some() {
        return provided;
    }
    if require_answers {
        if data.final_year_successor.is_none()
            && (data.final_k1
                || data.final_character.is_some()
                || data.final_67e_deduction != 0
                || data.final_personal_property_tax != 0
                || data.final_real_estate_tax != 0
                || data.final_state_income_tax != 0
                || data.final_state_sales_tax != 0
                || data.final_short_carry != 0
                || data.final_long_carry != 0
                || data.final_nol.is_some())
        {
            return Some(ValidationIssue::new("ESTATE_SUCCESSOR_NEEDED").at_field("successor"));
        }
        if let crate::ReitDividends::Reported { eligible: None, .. } = data.section199a_dividends {
            return Some(
                ValidationIssue::new("DIV_REIT_ELIGIBILITY_NEEDED")
                    .at_field("section199a_eligible"),
            );
        }
        if let Some(index) = data
            .activities
            .activities
            .iter()
            .position(|row| matches!(row.participation, crate::EstateParticipation::Unresolved))
        {
            return Some(
                ValidationIssue::new("ESTATE_PARTICIPATION_NEEDED")
                    .at_field("participation")
                    .in_list("activities", index)
                    .in_field("business"),
            );
        }
        if let Some(index) = data.qbi.apportionment_needed(&data.activities) {
            return Some(
                ValidationIssue::new("ESTATE_QBI_APPORTIONMENT_NEEDED")
                    .at_field("allocations")
                    .in_list("statements", index)
                    .in_field("qbi"),
            );
        }
        if let Some(amt) = &data.amt_reconciliation
            && let Some(index) =
                amt.adjustments.iter().position(|row| matches!(row.treatment, T::Unavailable))
        {
            return Some(
                ValidationIssue::new("ESTATE_AMT_TREATMENT_NEEDED")
                    .at_field("treatment")
                    .in_list("adjustments", index)
                    .in_field("amt_reconciliation"),
            );
        }
    }
    None
}

pub(super) fn bind_issue(
    document: &crate::K1EstateTrustDocument,
    issue: ValidationIssue,
    submitted: &[(i64, String, usize)],
    id: i64,
) -> Option<SaveFailure> {
    let research_line = issue.key == "ESTATE_CREDIT_LINE_INVALID"
        && document.statements.reported_credits.iter().enumerate().any(|(index, credit)| {
            matches!(&credit.code, crate::EstateCreditCodePaper::I)
                && issue.field
                    == ValidationIssue::new(issue.key)
                        .at_field("line")
                        .in_field("code")
                        .in_list("reported_credits", index)
                        .field
        });
    if issue.key == "DOCUMENT_K1_RESEARCH_CREDIT_LINE" || research_line {
        let key = alloc::format!("income/{id}/estate/research_credit");
        let index = submitted.iter().find(|(source, found, _)| *source == id && *found == key)?.2;
        return Some(
            ValidationIssue::new(issue.key)
                .at_field("value")
                .in_field("answer")
                .in_list("followups", index)
                .into(),
        );
    }
    let source = document.source().page();
    let moved: &[(&[&'static str], &str, &[&'static str])] = &[
        (&["entity_period", "accounting"], "estate/accounting", &["value"]),
        (&["entity_period"], "estate/accounting", &["value"]),
        (&["more", "final_year", "nol"], "estate/nol_records", &["value"]),
        (&["final_nol"], "estate/nol_records", &["value"]),
        (&["more", "niit", "modified_agi"], "estate/niit_allocation", &["value"]),
        (
            &["more", "final_year", "personal_property_tax"],
            "estate/final_tax_character",
            &["personal_property_tax"],
        ),
        (
            &["more", "final_year", "real_estate_tax"],
            "estate/final_tax_character",
            &["real_estate_tax"],
        ),
        (
            &["more", "final_year", "state_income_tax"],
            "estate/final_tax_character",
            &["state_income_tax"],
        ),
        (
            &["more", "final_year", "state_sales_tax"],
            "estate/final_tax_character",
            &["state_sales_tax"],
        ),
    ];
    if let Some((from, suffix, to)) = moved.iter().find(|(from, _, _)| issue.has_field_prefix(from))
    {
        let key = alloc::format!("income/{id}/{suffix}");
        let index = submitted.iter().find(|(_, found, _)| *found == key)?.2;
        return Some(
            issue.map_field_prefix(from, to).in_field("answer").in_list("followups", index).into(),
        );
    }
    let target = |suffix: &str| {
        let key = alloc::format!("income/{id}/{suffix}");
        let index = submitted.iter().find(|(found, name, _)| *found == id && *name == key)?.2;
        ValidationIssue::new(issue.key)
            .at_record()
            .in_field("answer")
            .in_list("followups", index)
            .field
    };
    if issue.key == "ESTATE_AMT_ACTIVITY_RECONCILE" {
        let amt = source.amt_reconciliation.as_ref()?;
        let crate::K1EstateTrustBusiness::Amounts(activities) = &source.business else {
            return None;
        };
        let targets: Vec<_> = activities.activities.iter().map(|activity| {
            let adjustment = amt.adjustments.iter().find(|adjustment| matches!(adjustment.treatment, crate::EstateAmtTreatmentPage::Activity { activity: current, .. } if current == activity.id)).or_else(|| amt.adjustments.first())?;
            let key = alloc::format!("income/{id}/estate/amt/{}", adjustment.id);
            let index = submitted.iter().find(|(_, found, _)| *found == key)?.2;
            ValidationIssue::new(issue.key).at_field("treatment").in_field("answer").in_list("followups", index).field
        }).collect();
        let selected =
            alloc::vec![true; submitted.iter().map(|(_, _, index)| index + 1).max().unwrap_or(0)];
        return Some(
            SaveFailure::from(issue)
                .map_submitted_items(&["activities", "activities"], &targets)
                .map_submitted_items(&["business", "activities"], &targets)
                .map_list_item_field_prefix(
                    &["followups"],
                    &["answer", "treatment", "amt_difference"],
                    &["answer", "treatment"],
                    &selected,
                ),
        );
    }
    if issue.has_field_prefix(&["business", "activities"])
        || issue.has_field_prefix(&["activities", "activities"])
    {
        let crate::K1EstateTrustBusiness::Amounts(activities) = &source.business else {
            return None;
        };
        let targets: Vec<_> = activities
            .activities
            .iter()
            .map(|row| target(&alloc::format!("estate/activity/{}", row.id)))
            .collect();
        return Some(
            SaveFailure::from(issue)
                .map_submitted_items(&["business", "activities"], &targets)
                .map_submitted_items(&["activities", "activities"], &targets),
        );
    }
    if issue.has_field_prefix(&["qbi", "statements"]) {
        let targets: Vec<_> = source
            .qbi
            .statements
            .iter()
            .map(|row| target(&alloc::format!("estate/qbi/{}", row.id)))
            .collect();
        return Some(
            SaveFailure::from(issue).map_submitted_items(&["qbi", "statements"], &targets),
        );
    }
    if issue.has_field_prefix(&["amt_reconciliation", "adjustments"]) {
        let amt = source.amt_reconciliation.as_ref()?;
        let targets: Vec<_> = amt
            .adjustments
            .iter()
            .map(|row| target(&alloc::format!("estate/amt/{}", row.id)))
            .collect();
        return Some(
            SaveFailure::from(issue)
                .map_submitted_items(&["amt_reconciliation", "adjustments"], &targets),
        );
    }
    if issue.has_field_prefix(&["investment_property"]) {
        let key = alloc::format!("income/{id}/investment");
        let index = submitted.iter().find(|(_, found, _)| *found == key)?.2;
        return Some(
            issue
                .map_field_prefix(&["investment_property"], &["properties"])
                .in_field("answer")
                .in_list("followups", index)
                .into(),
        );
    }
    let (suffix, from, to): (&str, &[&'static str], &[&'static str]) =
        if issue.has_field_prefix(&["successor"]) {
            ("estate/successor", &["successor"], &["value"])
        } else if issue.has_field_prefix(&["final_mortgages"]) {
            ("estate/final", &["final_mortgages"], &["mortgages"])
        } else if issue.has_field_prefix(&["section199a_eligible"]) {
            ("estate/reit", &["section199a_eligible"], &["value"])
        } else {
            return None;
        };
    let key = alloc::format!("income/{id}/{suffix}");
    let index = submitted.iter().find(|(_, found, _)| *found == key)?.2;
    Some(issue.map_field_prefix(from, to).in_field("answer").in_list("followups", index).into())
}
