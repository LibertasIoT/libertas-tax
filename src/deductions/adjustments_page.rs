// adjustments items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// The review's "Adjustments to income" count across every topic of the hub
// (M3-P8 adds self-employed benefits, IRA contributions, educator expenses
// and student loan interest to the HSA count M3-P7 started).
pub(super) fn adjustments_count(d: &FederalDraft) -> usize {
    hsa_accounts(d).len()
        + benefits_entered(d).len()
        + ira_contributor_rows(d).unwrap_or_default().len()
        + usize::from(d.adjustments.as_ref().is_some_and(|a| a.educators.any()))
        + usize::from(d.adjustments.as_ref().is_some_and(|a| a.student_loan.any()))
}
// More adjustments (unit u5-adjust): the alimony rows, the moving rows (Form
// 3903).
pub(super) fn more_adjustment_items(d: &FederalDraft) -> Vec<MoreAdjustment> {
    d.adjustments
        .iter()
        .flat_map(|a| &a.alimony_paid)
        .map(|v| {
            MoreAdjustment::AlimonyPaid(AlimonyItem { owner: v.owner, payment: v.payment.clone() })
        })
        .chain(d.adjustments.iter().flat_map(|a| &a.moving).map(|m| {
            MoreAdjustment::MovingExpenses(crate::MovingItem {
                owner: m.owner,
                record: m.record.clone(),
            })
        }))
        .collect()
}
// Both AlimonyPayment entry paths share these existing recapture checks.
pub(super) fn alimony_issue(payment: &crate::AlimonyPayment) -> Option<ValidationIssue> {
    match &payment.recapture {
        crate::AlimonyRecapture::ThirdYear { first, first_excluded, .. }
            if first_excluded > first =>
        {
            Some(
                ValidationIssue::new("ALIMONY_2024_EXCLUDED_ABOVE_PAID")
                    .at_field("first_excluded")
                    .in_field("recapture"),
            )
        }
        crate::AlimonyRecapture::ThirdYear { second, second_excluded, .. }
            if second_excluded > second =>
        {
            Some(
                ValidationIssue::new("ALIMONY_2025_EXCLUDED_ABOVE_PAID")
                    .at_field("second_excluded")
                    .in_field("recapture"),
            )
        }
        _ => None,
    }
}
// More adjustments' row (unit u5-adjust): a count for each kind that has
// entries (alimony agreements, moves), joined by one shared composition; None
// when nothing is entered.
pub(crate) fn more_adjustments_row(d: &FederalDraft) -> Option<ReviewItem> {
    let counts = d.adjustments.as_ref().map_or([0; 2], |a| [a.alimony_paid.len(), a.moving.len()]);
    let parts: Vec<Vec<u8>> = ["REVIEW_ALIMONY", "REVIEW_MOVES"]
        .into_iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .map(|(key, n)| status_count(key, n))
        .collect();
    let status = parts.into_iter().reduce(|joined, next| {
        libertas_formatted_text(
            "REVIEW_STATUS_JOIN",
            &[
                LibertasMessageArgument::formatted_text(&joined),
                LibertasMessageArgument::formatted_text(&next),
            ],
        )
    })?;
    Some(ReviewItem {
        field: None,
        item: section_message(TaxSection::MoreAdjustments).encode(),
        status,
        section: TaxSection::MoreAdjustments,
    })
}

impl Interview {
    // What the adjustments hub offers: the previous shown section (M3-P19),
    // the checklist, its topics (the self-employed benefits topic only with
    // a business or partnership, M3-P8) and the next shown section
    // ("Continue").
    pub(super) fn adjustments_hub_sections(&self) -> Vec<i32> {
        let mut sections = alloc::vec![self.previous_shown(TaxSection::Adjustments)];
        if !sections.contains(&CHECKLIST_PAGE) {
            sections.push(CHECKLIST_PAGE);
        }
        sections.extend(offered_topics(&self.draft, &ADJUSTMENT_TOPICS));
        sections.push(MORE_ADJUSTMENTS_PAGE);
        sections.push(self.after_adjustments() as i32);
        sections
    }
    // The adjustments hub: one row per topic with data or needing attention.
    // The suggestion is the first topic the return makes necessary: HSA
    // while a W-2 reports employer contributions (box 12 code W) for an
    // owner with no account or an account's "something else" answer is
    // refused; self-employed benefits while a business exists and nothing
    // is entered; IRA contributions while box 12 deferral codes or a prior
    // Form 8606 exist and nothing is entered (derived_topics); otherwise the
    // next section. Educator expenses and student loan interest are
    // ordinary topics: suggested only when planned, shown once answered.
    pub(super) fn adjustments_hub(&self, review: bool) -> Reply {
        let d = &self.draft;
        let accounts = hsa_accounts(d);
        let benefits = benefits_entered(d);
        let ira_people = ira_contributor_rows(d).unwrap_or_default();
        let result = estimate(d);
        let items = self.remaining_items(&result);
        let planned = planned_sections(d);
        let offered = self.adjustments_hub_sections();
        let hub_topics = hub_topics(TaxSection::Adjustments);
        let walk = walk_topics(d, TaxSection::Adjustments, &offered);
        let suggested =
            hub_suggestion(&hub_topics, &offered, &items, &walk, self.after_adjustments());
        let hsa_status = match accounts.len() {
            0 if d.adjustments.as_ref().is_some_and(|a| {
                matches!(a.archer, crate::ArcherContributions::Records { .. })
            }) =>
            {
                Some(status("REVIEW_RECORDED"))
            }
            0 if suggested == TaxSection::Hsa => Some(status("REVIEW_NOT_STARTED")),
            0 => None,
            n => Some(status_count("REVIEW_ACCOUNTS", n)),
        };
        // Every row with something entered, businesses and partnerships
        // alike; "Not started" while the topic is necessary (T2-43p).
        let benefits_status = if !benefits.is_empty() {
            Some(
                message_list(
                    &benefits
                        .iter()
                        .map(|name| {
                            crate::Message::new(
                                "TEXT",
                                alloc::vec![crate::MessageArg::Encoded(name.clone())],
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .encode(),
            )
        } else if benefits_needed(d) {
            Some(status("REVIEW_NOT_STARTED"))
        } else {
            None
        };
        let ira_status = match ira_people.len() {
            0 if suggested == TaxSection::IraContributions => Some(status("REVIEW_NOT_STARTED")),
            0 => None,
            n => Some(status_count("REVIEW_PEOPLE", n)),
        };
        let educator_status = d
            .adjustments
            .as_ref()
            .is_some_and(|a| a.educators.any())
            .then(|| status("REVIEW_RECORDED"));
        let student_loan_status = d
            .adjustments
            .as_ref()
            .is_some_and(|a| a.student_loan.any())
            .then(|| status("REVIEW_RECORDED"));
        let employee_expenses_status =
            d.adjustments.as_ref().and_then(|a| match a.employee_expenses.len() {
                0 => None,
                n => Some(status_count("REVIEW_FORMS", n)),
            });
        let topics = [
            (TaxSection::Hsa, hsa_status),
            (TaxSection::SelfEmployedBenefits, benefits_status),
            (TaxSection::IraContributions, ira_status),
            (TaxSection::Educator, educator_status),
            (TaxSection::StudentLoan, student_loan_status),
            (TaxSection::EmployeeExpenses, employee_expenses_status),
        ]
        .into_iter()
        .filter_map(|(section, status)| {
            status.map(|status| ReviewItem {
                field: None,
                item: section_message(section).encode(),
                status,
                section,
            })
        })
        .collect();
        let mut topics = mark_attention(topics, &ADJUSTMENT_TOPICS, &items);
        topics.extend(more_adjustments_row(d));
        let topics = with_checklist_rows(topics, &hub_topics, &offered, &planned);
        let topics = with_every_destination(d, topics, TaxSection::Adjustments, &offered);
        let done = walk.is_empty().then(|| done_with(TaxSection::Adjustments));
        let allowed_sections = self.allowed_sections();
        let navigation = self.navigation_availability(&allowed_sections);
        let allowed_actions = self.hub_actions(TaxSection::Adjustments);
        let primary_actions = Self::primary_actions(&allowed_actions);
        Reply::Response(P::Hub {
            crop_insurance: Vec::new(),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: offered[0],
            review,
            estimate: self.estimate_so_far(d, &result, TaxSection::Adjustments),
            progress: progress_line(ADJUSTMENTS_PAGE, &shown_sections(d), done),
            told: told_us(d, TaxSection::Adjustments),
            topics: hub_list(TaxSection::Adjustments, topics),
            previous_section: previous_section_row(self.previous_shown(TaxSection::Adjustments)),
            navigation,
            suggested_destination: suggested.into(),
            primary_actions,
            go_to_actions: alloc::vec![libertas_macros::variant_index!(P::Navigate) as i32],
            allowed_actions,
            suggested,
            allowed_sections,
        })
    }
    // The adjustments hub's last topic (unit u5-adjust), opened with its
    // complete value: alimony paid, then the refusal kinds chosen.
    pub(super) fn more_adjustments_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        if d.adjustments.is_none() {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        Reply::Edit(P::SaveMoreAdjustments {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: ADJUSTMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::MoreAdjustments),
            warnings: Vec::new(),
            progress: self.topic_progress(ADJUSTMENTS_PAGE, Some(TaxSection::MoreAdjustments)),
            allowed_owners: self.allowed_owners(),
            value: MoreAdjustments { adjustments: more_adjustment_items(d) },
        })
    }
    // More adjustments (unit u5-adjust): the alimony rows onto
    // FederalAdjustments.alimony_paid (the page's whole answer); the server
    // twins of AlimonyItem's rules.
    pub(super) fn more_adjustments_candidate(
        &self,
        value: &MoreAdjustments,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        // Unrelated generic admission errors remain explicitly unlocated.
        let status =
            |key: &str| crate::validation::SaveFailure::from(crate::interview::status(key));
        let owners = self.allowed_owners();
        let mut alimony = Vec::new();
        for (index, item) in value.adjustments.iter().enumerate() {
            if let MoreAdjustment::AlimonyPaid(AlimonyItem { owner, payment }) = item {
                if !owners.contains(&(*owner as i32)) {
                    return Err(ValidationIssue::new("MESSAGE_ALIMONY_YOUR_SPOUSE_PAID_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY")
                        .at_field("owner").in_list("adjustments", index).in_field("value").into());
                }
                if let Some(issue) = alimony_issue(payment) {
                    return Err(issue
                        .in_field("payment")
                        .in_list("adjustments", index)
                        .in_field("value")
                        .into());
                }
                if let Some(issue) = payment.validation_issue(
                        "MESSAGE_ALIMONY_PAID_ENTER_THE_AGREEMENT_DATE_BEFORE_2027_AND_AMOUNTS_WITH_THIS_YEAR",
                ) {
                    return Err(issue
                        .in_field("payment")
                        .in_list("adjustments", index)
                        .in_field("value")
                        .into());
                }
                alimony.push(crate::AlimonyPaid { owner: *owner, payment: payment.clone() });
            }
        }
        let mut moving = Vec::new();
        for (index, item) in value.adjustments.iter().enumerate() {
            if let MoreAdjustment::MovingExpenses(crate::MovingItem { owner, record }) = item {
                if !owners.contains(&(*owner as i32)) {
                    return Err(ValidationIssue::new("MESSAGE_MOVING_YOUR_SPOUSES_MOVE_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY")
                        .at_field("owner").in_list("adjustments", index).in_field("value").into());
                }
                if let Some(issue) = record.issue() {
                    return Err(issue
                        .in_field("record")
                        .in_list("adjustments", index)
                        .in_field("value")
                        .into());
                }
                if !record.valid() {
                    return Err(status("MESSAGE_MOVING_ENTER_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE"));
                }
                // The reimbursement is read from the W-2s and re-derived on save.
                moving.push(crate::MovingClaim { owner: *owner, record: record.clone() });
            }
        }
        let mut draft = self.draft.clone();
        let Some(adjustments) = draft.adjustments.as_mut() else {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        };
        adjustments.alimony_paid = alimony;
        adjustments.moving = moving;
        visit(&self.draft, &mut draft, TaxSection::Adjustments, TaxSection::MoreAdjustments);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_more_adjustments(
        &self,
        value: &MoreAdjustments,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ADJUSTMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.more_adjustments_candidate(value)
    }
}
