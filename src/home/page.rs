use super::*;

impl Interview {
    pub(super) fn home_site_choices(&self) -> Vec<RecordChoice> {
        self.draft
            .home_sites
            .iter()
            .map(|site| RecordChoice {
                id: site.id,
                label: Message::new(
                    "HOME_SAVED_HOME",
                    alloc::vec![
                        MessageArg::Number(site.id),
                        MessageArg::Text(site.address.clone())
                    ],
                )
                .encode(),
            })
            .collect()
    }

    pub(super) fn home_offices_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        Reply::Edit(P::SaveHomeOffices {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: INCOME_PAGE,
            previous: INCOME_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::HomeOffices),
            warnings: Vec::new(),
            progress: self.topic_progress(INCOME_PAGE, Some(TaxSection::HomeOffices)),
            mortgage_loans: self.mortgage_choices(),
            home_activities: d
                .income
                .iter()
                .filter(|entry| matches!(entry.income, FederalIncome::Business { .. }))
                .map(|entry| RecordChoice {
                    id: entry.id,
                    label: Message::new(
                        "HOME_SAVED_ACTIVITY",
                        alloc::vec![
                            MessageArg::Number(entry.id),
                            MessageArg::Text(entry.label.clone())
                        ],
                    )
                    .encode(),
                })
                .collect(),
            casualty_events: d
                .deductions
                .iter()
                .flat_map(|ded| &ded.personal_casualties)
                .map(|event| RecordChoice {
                    id: event.id,
                    label: Message::new(
                        "HOME_SAVED_CASUALTY_EVENT",
                        alloc::vec![
                            MessageArg::Number(event.id),
                            MessageArg::Text(event.label.clone())
                        ],
                    )
                    .encode(),
                })
                .collect(),
            allowed_home_owners: self.allowed_owners(),
            value: HomeOffices { sites: d.home_sites.iter().cloned().map(Into::into).collect() },
        })
    }

    pub(super) fn home_offices_candidate(
        &self,
        value: &HomeOffices,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut draft = self.draft.clone();
        let mut sites: Vec<crate::HomeOfficeSite> =
            value.sites.iter().cloned().map(Into::into).collect();
        assign_record_ids(
            sites.iter_mut().map(|site| &mut site.id),
            self.draft.home_sites.iter().map(|site| site.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key).at_field("id").in_list("sites", index).in_field("value")
        })?;
        let loans = self.mortgage_choices();
        for (index, site) in sites.iter().enumerate() {
            if let Some(issue) = site.issue() {
                return Err(issue.in_list("sites", index).in_field("value").into());
            }
            if site.claims.iter().any(|claim| {
                !self.allowed_owners().contains(&(claim.owner as i32))
                    || claim.activities.iter().any(|activity| {
                        !draft.income.iter().any(|entry| {
                            entry.id == activity.activity
                                && entry.owner == crate::DocumentOwner::from(claim.owner)
                                && matches!(entry.income, FederalIncome::Business { .. })
                        })
                    })
            }) {
                return Err(ValidationIssue::new("HOME_ACTIVITY_SOURCE_INVALID")
                    .in_list("sites", index)
                    .in_field("value")
                    .into());
            }
            if site.mortgage_loans.iter().any(|loan| !loans.iter().any(|choice| choice.id == *loan))
            {
                return Err(ValidationIssue::new("HOME_MORTGAGE_SOURCE_INVALID")
                    .in_list("sites", index)
                    .in_field("value")
                    .into());
            }
        }
        draft.home_sites = sites;
        if let Some(issue) = crate::home_sites::graph_issue(&draft) {
            return Err(issue.in_field("value").into());
        }
        finish_section(&mut draft, TaxSection::HomeOffices);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_home_offices(
        &self,
        value: &HomeOffices,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != INCOME_PAGE || !self.reached(INCOME_PAGE) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.home_offices_candidate(value)
    }
}
