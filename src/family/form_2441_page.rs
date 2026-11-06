// form_2441 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// People with costs recorded on Child and dependent care: the credits hub's care row.
pub(super) fn care_people(d: &FederalDraft) -> usize {
    d.dependents()
        .iter()
        .filter(|v| {
            v.use_for == crate::PersonUse::TaxDependent
                && matches!(v.care, Some(DependentCare::Yes { .. }))
        })
        .count()
}

impl Interview {
    pub(super) fn care_catalog(&self, farm: bool) -> Vec<CareSourceChoice> {
        self.draft.income.iter().filter(|e|matches!(&e.income,FederalIncome::Business {data} if matches!(data.activity,crate::BusinessActivity::Farming {..})==farm)).map(|e|CareSourceChoice {id:e.id,owner:e.owner.owned(),label:crate::income_names::income_display_label(e,self.draft.dependents()).encode()}).collect()
    }
    pub(super) fn care_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let care = d.credits.as_ref().map(|c| c.care.clone()).unwrap_or_default();
        let qualifying_people = d
            .dependents()
            .iter()
            .map(|p| CareQualifyingRow {
                id: p.id,
                label: crate::person_names::person_name(&p.first_name, &p.last_name),
                care: match p.care {
                    Some(DependentCare::No) | None => CareCosts::No,
                    Some(DependentCare::Yes { care_expenses }) => CareCosts::Yes {
                        expenses: care_expenses,
                        incapable_self_care: p.incapable_self_care,
                    },
                },
                under_13_child: !crate::federal_model::attained_on(
                    p.birth_date,
                    13,
                    false,
                    20260101,
                ) && matches!(
                    p.relationship,
                    Relationship::Child | Relationship::Foster
                ),
            })
            .collect();
        let benefits = d
            .income
            .iter()
            .filter_map(|e| {
                if let FederalIncome::Wage { data } = &e.income {
                    let data = crate::FederalWage::from(data);
                    if data.care_benefits <= 0 {
                        return None;
                    }
                    let a = care.benefits.iter().find(|b| b.wage == e.id);
                    Some(CareBenefitRow {
                        wage: e.id,
                        label: crate::income_names::income_display_label(
                            e,
                            self.draft.dependents(),
                        )
                        .encode(),
                        owner: e.owner.owned(),
                        box10: data.care_benefits,
                        in_box_1: a.and_then(|a| a.in_box_1),
                        forfeited: a.and_then(|a| a.forfeited),
                    })
                } else {
                    None
                }
            })
            .collect();
        let care_filers = if self.joint() {
            d.people
                .as_ref()
                .map(|p| {
                    [(Owner::Taxpayer, Some(&p.taxpayer)), (Owner::Spouse, p.spouse.as_ref())]
                        .into_iter()
                        .filter_map(|(o, p)| {
                            p.filter(|p| p.full_time_student || p.incapable_self_care)
                                .map(|_| o as i32)
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        // The filing status follows Your 2026 return (one allowed choice).
        let filing = if separate(d) {
            if matches!(care.filing, CareFilingStatus::Separate { .. }) {
                care.filing.clone()
            } else {
                CareFilingStatus::Separate { spouse_earned_income: None }
            }
        } else {
            CareFilingStatus::Other
        };
        let allowed_filing = alloc::vec![if separate(d) {
            libertas_macros::variant_index!(CareFilingStatus::Separate) as i32
        } else {
            libertas_macros::variant_index!(CareFilingStatus::Other) as i32
        }];
        let editor = P::SaveCare {
            allowed_filing,
            allowed_injured_source_facts: crate::injured_spouse_sources::allowed(d),
            people: self.income_people(),
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CREDITS_PAGE,
            previous: CREDITS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Care),
            warnings: Vec::new(),
            progress: self.topic_progress(CREDITS_PAGE, Some(TaxSection::Care)),
            allowed_owners: self.allowed_owners(),
            care_filers,
            care_months: (1..=12)
                .map(|id| RecordChoice {
                    id,
                    label: libertas_formatted_text(&alloc::format!("CARE_MONTH_{id}"), &[]),
                })
                .collect(),
            businesses: self.care_catalog(false),
            farms: self.care_catalog(true),
            partnerships: d
                .income
                .iter()
                .filter(|e| matches!(e.income, FederalIncome::Partnership { .. }))
                .map(|e| CareSourceChoice {
                    id: e.id,
                    owner: e.owner.owned(),
                    label: crate::income_names::income_display_label(e, d.dependents()).encode(),
                })
                .collect(),
            value: CarePage {
                plan_maximum: if !care.has_benefits(d) {
                    CarePlanMaximum::NoBenefits
                } else if matches!(care.plan_maximum, CarePlanMaximum::NoBenefits) {
                    CarePlanMaximum::Plans { maximum: 0 }
                } else {
                    care.plan_maximum.clone()
                },
                qualifying_people,
                providers: care.providers,
                benefits,
                grace: care.grace.into_iter().map(Into::into).collect(),
                own_business: care.own_business.into_iter().map(Into::into).collect(),
                filers: care.filers.into_iter().map(Into::into).collect(),
                filing,
                new_people: Vec::new(),
                unpaid_return_year_care: care.unpaid_return_year_care,
                prior_year: care.prior_year.map(Into::into),
            },
        };
        // A person whose care this page has not answered yet: the page opens
        // by WorkflowRequest (BeginCare) with the answered people as saved
        // and the new ones unanswered, the rest of the page as it stands.
        let unanswered: Vec<i64> =
            d.dependents().iter().filter(|p| p.care.is_none()).map(|p| p.id).collect();
        if !unanswered.is_empty() {
            Reply::Response(begin_care(editor, &unanswered))
        } else {
            Reply::Edit(editor)
        }
    }
    pub(super) fn care_candidate(
        &self,
        value: &CarePage,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        // Unrelated generic admission errors remain explicitly unlocated.
        let status =
            |key: &str| crate::validation::SaveFailure::from(crate::interview::status(key));
        let d = &self.draft;
        if !crate::injured_spouse_sources::requested(d)
            && value.prior_year.as_ref().is_some_and(|p| p.injured_claimants.is_some())
        {
            return Err(status("INJURED_SOURCE_FACTS_NOT_REQUESTED"));
        }
        let mut draft = d.clone();
        // One row for each person on the people page, answered or new.
        let rows: Vec<(&str, usize, &CareQualifyingRow)> = value
            .qualifying_people
            .iter()
            .enumerate()
            .map(|(i, row)| ("qualifying_people", i, row))
            .chain(value.new_people.iter().enumerate().map(|(i, row)| ("new_people", i, row)))
            .collect();
        if rows.len() != d.dependents().len()
            || d.dependents()
                .iter()
                .any(|p| rows.iter().filter(|(_, _, r)| r.id == p.id).count() != 1)
        {
            return Err(status("CARE_INVALID_RECORD"));
        }
        // The filing status is Your 2026 return's, never asked here.
        if matches!(value.filing, CareFilingStatus::Separate { .. }) != separate(d) {
            return Err(status("CARE_INVALID_RECORD"));
        }
        for person in draft.dependents.iter_mut() {
            let (list, index, row) = *rows.iter().find(|(_, _, r)| r.id == person.id).unwrap();
            // IRC 21(b)(1)(A), as family() tests it: care counts when the
            // person is under 13 on at least one 2026 day (tested on January
            // 1, on the birthday).
            if matches!(row.care, CareCosts::Yes { incapable_self_care: false, .. })
                && crate::federal_model::attained_on(person.birth_date, 13, false, 20260101)
            {
                return Err(ValidationIssue::new("CARE_NEEDS_UNDER_13_OR_SELF_CARE")
                    .at_field("incapable_self_care")
                    .in_field("care")
                    .in_list(list, index)
                    .in_field("value")
                    .into());
            }
            (person.care, person.incapable_self_care) = match row.care {
                CareCosts::No => (Some(DependentCare::No), false),
                CareCosts::Yes { expenses, incapable_self_care } => {
                    (Some(DependentCare::Yes { care_expenses: expenses }), incapable_self_care)
                }
            };
        }
        let benefits = value
            .benefits
            .iter()
            .map(|b| CareBenefitAnswers {
                wage: b.wage,
                in_box_1: b.in_box_1,
                forfeited: b.forfeited,
            })
            .collect();
        let mut care = FederalCare {
            plan_maximum: value.plan_maximum.clone(),
            providers: value.providers.clone(),
            benefits,
            grace: value.grace.iter().cloned().map(Into::into).collect(),
            own_business: value.own_business.iter().cloned().map(Into::into).collect(),
            filers: value.filers.iter().cloned().map(Into::into).collect(),
            filing: value.filing.clone(),
            unpaid_return_year_care: value.unpaid_return_year_care,
            prior_year: value.prior_year.clone().map(Into::into),
        };
        if let Some(issue) = care.issue(&draft) {
            return Err(issue.in_field("value").into());
        }
        let expected = d
            .income
            .iter()
            .filter(|e| matches!(&e.income,FederalIncome::Wage {data} if crate::FederalWage::from(data).care_benefits>0))
            .count();
        let invalid_wage = || ValidationIssue::new("CARE_WAGE_SOURCE_INVALID");
        if expected != care.benefits.len() {
            return Err(invalid_wage().at_field("benefits").in_field("value").into());
        }
        for (index, row) in value.benefits.iter().enumerate() {
            let source = d.income.iter().find(|entry| entry.id == row.wage);
            let field = match source {
                None => Some("wage"),
                Some(entry) if entry.owner != crate::DocumentOwner::from(row.owner) => {
                    Some("owner")
                }
                Some(entry) => match &entry.income {
                    FederalIncome::Wage { data } => {
                        let data = &crate::FederalWage::from(data);
                        (data.care_benefits != row.box10).then_some("box10")
                    }
                    _ => Some("wage"),
                },
            };
            if let Some(field) = field {
                return Err(invalid_wage()
                    .at_field(field)
                    .in_list("benefits", index)
                    .in_field("value")
                    .into());
            }
        }
        assign_record_ids(
            care.providers.iter_mut().map(|p| &mut p.id),
            d.credits.as_ref().into_iter().flat_map(|c| c.care.providers.iter().map(|p| p.id)),
            &mut draft.next_id,
        )
        .map_err(|(key, _, _)| status(key))?;
        draft.credits.as_mut().ok_or_else(|| status("CARE_INVALID_RECORD"))?.care = care;
        finish_section(&mut draft, TaxSection::Care);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_care(
        &self,
        value: &CarePage,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != CREDITS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.care_candidate(value)
    }
}

// The unanswered care page: the editor's values with each row's care left out.
fn begin_care(editor: P, unanswered: &[i64]) -> P {
    let P::SaveCare {
        allowed_injured_source_facts,
        people,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate,
        progress,
        allowed_owners,
        care_filers,
        care_months,
        businesses,
        farms,
        partnerships,
        allowed_filing,
        value,
        ..
    } = editor
    else {
        unreachable!("the care editor")
    };
    let (answered, unasked): (Vec<_>, Vec<_>) =
        value.qualifying_people.into_iter().partition(|row| !unanswered.contains(&row.id));
    P::BeginCare {
        allowed_injured_source_facts,
        people,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate: estimate.map(Into::into),
        progress,
        allowed_owners,
        care_filers,
        care_months,
        businesses,
        farms,
        partnerships,
        allowed_filing,
        value: crate::CareSeed {
            plan_maximum: value.plan_maximum,
            qualifying_people: answered,
            new_people: unasked
                .into_iter()
                .map(|row| crate::CareSeedRow {
                    id: row.id,
                    label: row.label,
                    under_13_child: row.under_13_child,
                })
                .collect(),
            providers: value.providers,
            benefits: value.benefits.into_iter().map(Into::into).collect(),
            grace: value.grace,
            own_business: value.own_business,
            filers: value.filers,
            filing: value.filing,
            unpaid_return_year_care: value.unpaid_return_year_care,
            prior_year: value.prior_year,
        },
    }
}
