// credits items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// More credits: the stored FederalCredits fields this page exposes,
// one list item per field, in the struct's declaration order, and the home
// kerosene credit stored with the payments (u5-credits). A field this page
// does not cover (students, education, care, saver, marketplace,
// marketplace_dependents; mortgage_subsidies is the other taxes page's) is
// untouched here.
pub(super) fn more_credit_items(d: &FederalDraft) -> Vec<MoreCredit> {
    let Some(credits) = d.credits.as_ref() else {
        return Vec::new();
    };
    let mut items = Vec::new();
    if let Some(value) = &credits.minimum_tax {
        items.push(MoreCredit::MinimumTax(value.clone().into()));
    }
    if let Some(value) = &credits.mortgage_credit {
        items.push(MoreCredit::MortgageCredit {
            current: value.current.clone(),
            carry_three_years_prior: value.carry_three_years_prior,
            carry_two_years_prior: value.carry_two_years_prior,
            carry_prior_year: value.carry_prior_year,
            loan: value.loan,
        });
    }
    if credits.residential_energy_carry.is_some() || credits.dc_homebuyer_carry.is_some() {
        items.push(MoreCredit::PriorCreditCarryovers {
            residential_energy_carry: credits.residential_energy_carry,
            dc_homebuyer_carry: credits.dc_homebuyer_carry,
        });
    }
    if credits.clean_vehicles.is_some() || !credits.personal_refueling.is_empty() {
        items.push(MoreCredit::CleanVehiclesAndRefueling {
            clean_vehicles: credits.clean_vehicles.clone().map(Into::into),
            personal_refueling: credits.personal_refueling.clone(),
        });
    }
    if !credits.adoptions.is_empty() || !credits.adoption_carry.is_empty() {
        items.push(MoreCredit::Adoption {
            adoptions: credits
                .adoptions
                .iter()
                .map(|c| c.page(&employee_wage_choices(d)))
                .collect(),
            adoption_carry: credits.adoption_carry.clone(),
        });
    }
    if !credits.exempt_pensions.is_empty() {
        items.push(MoreCredit::ElderlyOrDisabled {
            exempt_pensions: credits.exempt_pensions.clone(),
        });
    }
    if credits.foreign_tax != ForeignTaxClaim::No
        && let Ok(page) = ForeignTaxPage::try_from(credits.foreign_tax.clone())
    {
        items.push(MoreCredit::ForeignTax(page));
    }
    if credits.legacy_ev_passive_allowed.is_some() {
        items.push(MoreCredit::ElectricVehicleCarryover {
            legacy_ev_passive_allowed: credits.legacy_ev_passive_allowed,
        });
    }
    if let Some(value) = &credits.tip_fica_carryforward {
        items.push(MoreCredit::EmployerTip(value.clone()));
    }
    if let Some(value) = &credits.business_credits {
        items.push(MoreCredit::BusinessCredits(value.clone().into()));
    }
    if let Some(value) = &credits.education_recapture {
        items.push(MoreCredit::EducationRecapture(value.clone()));
    }
    if let Some(value) = &credits.after_disallowance {
        items.push(MoreCredit::AfterDisallowance(value.clone()));
    }
    if let Some(value) = d.payments.as_ref().and_then(|p| p.home_kerosene.as_ref()) {
        items.push(MoreCredit::HomeKerosene(value.clone()));
    }
    if credits.decline_earned_income_credit {
        items.push(MoreCredit::DeclineEarnedIncomeCredit);
    }
    items
}
// The list onto FederalCredits (and the payments' home kerosene): a variant
// absent from the list clears its field(s), matching the platform's
// ordinary "the list is the whole answer" save contract. Two items of the
// same kind keep the last one entered; the client's chooser makes that
// pairing unlikely, and no page elsewhere reads these fields between save
// and the next open.
pub(super) fn apply_more_credits(d: &mut FederalDraft, items: &[MoreCredit]) {
    let kerosene = items.iter().rev().find_map(|item| match item {
        MoreCredit::HomeKerosene(value) => Some(value.clone()),
        _ => None,
    });
    if kerosene.is_some() || d.payments.is_some() {
        let joint = joint(d);
        d.payments
            .get_or_insert_with(|| FederalPayments { joint, ..FederalPayments::none() })
            .home_kerosene = kerosene;
    }
    let Some(credits) = d.credits.as_mut() else {
        return;
    };
    credits.decline_earned_income_credit = false;
    credits.minimum_tax = None;
    credits.mortgage_credit = None;
    credits.residential_energy_carry = None;
    credits.dc_homebuyer_carry = None;
    credits.clean_vehicles = None;
    credits.personal_refueling = Vec::new();
    credits.adoptions = Vec::new();
    credits.adoption_carry = Vec::new();
    credits.exempt_pensions = Vec::new();
    credits.foreign_tax = ForeignTaxClaim::No;
    credits.legacy_ev_passive_allowed = None;
    credits.tip_fica_carryforward = None;
    credits.business_credits = None;
    credits.education_recapture = None;
    credits.after_disallowance = None;
    for item in items {
        match item.clone() {
            MoreCredit::MinimumTax(value) => credits.minimum_tax = Some(value.into()),
            MoreCredit::MortgageCredit {
                current,
                carry_three_years_prior,
                carry_two_years_prior,
                carry_prior_year,
                loan,
            } => {
                credits.mortgage_credit = Some(FederalMortgageCredit {
                    current,
                    carry_three_years_prior,
                    carry_two_years_prior,
                    carry_prior_year,
                    loan,
                })
            }
            MoreCredit::PriorCreditCarryovers { residential_energy_carry, dc_homebuyer_carry } => {
                credits.residential_energy_carry = residential_energy_carry;
                credits.dc_homebuyer_carry = dc_homebuyer_carry;
            }
            MoreCredit::CleanVehiclesAndRefueling { clean_vehicles, personal_refueling } => {
                credits.clean_vehicles = clean_vehicles.map(Into::into);
                credits.personal_refueling = personal_refueling;
            }
            MoreCredit::Adoption { adoptions, adoption_carry } => {
                credits.adoptions = adoptions.iter().map(|c| c.record()).collect();
                credits.adoption_carry = adoption_carry;
            }
            MoreCredit::ElderlyOrDisabled { exempt_pensions } => {
                credits.exempt_pensions = exempt_pensions;
            }
            MoreCredit::ForeignTax(value) => credits.foreign_tax = value.clone().into(),
            MoreCredit::ElectricVehicleCarryover { legacy_ev_passive_allowed } => {
                credits.legacy_ev_passive_allowed = legacy_ev_passive_allowed;
            }
            MoreCredit::EmployerTip(value) => credits.tip_fica_carryforward = Some(value),
            MoreCredit::BusinessCredits(value) => credits.business_credits = Some(value.into()),
            MoreCredit::EducationRecapture(value) => credits.education_recapture = Some(value),
            MoreCredit::AfterDisallowance(value) => credits.after_disallowance = Some(value),
            // Stored with the payments, above.
            MoreCredit::HomeKerosene(_) => {}
            // The refusal kinds are stored as situations (more_credits_candidate).
            MoreCredit::DeclineEarnedIncomeCredit => credits.decline_earned_income_credit = true,
            MoreCredit::OtherCredit => {}
        }
    }
}
pub(super) fn credits_count(d: &FederalDraft) -> usize {
    let credits = d.credits.as_ref();
    usize::from(care_people(d) != 0)
        + usize::from(credits.is_some_and(|c| !c.education.is_empty()))
        + usize::from(credits.is_some_and(|c| c.saver.is_some()))
        + usize::from(
            credits.is_some_and(|c| matches!(c.marketplace, MarketplaceClaim::Yes { .. })),
        )
        + usize::from(!more_credit_items(d).is_empty() || !d.credit_refusals.is_empty())
}

impl Interview {
    // What the credits hub offers: the previous section, Children and people
    // you support, Child and dependent care, education, the saver's
    // credit when it can apply, more credits, and the next section
    // ("Continue").
    pub(super) fn credits_hub_sections(&self) -> Vec<i32> {
        let mut sections = alloc::vec![self.before_credits()];
        if !sections.contains(&CHECKLIST_PAGE) {
            sections.push(CHECKLIST_PAGE);
        }
        if in_walk(&self.draft, TaxSection::Children) {
            sections.push(CHILDREN_PAGE);
        }
        sections.extend(offered_topics(&self.draft, &CREDIT_TOPICS));
        sections.push(MORE_CREDITS_PAGE);
        sections.push(self.after_credits() as i32);
        sections
    }
    // The credits hub, topics in Schedule 3's order. It suggests education
    // while a student is on the return and the topic is not yet visited, then
    // the saver's credit while a contribution exists inside Form 8880's bands
    // and it is not yet visited, otherwise the next section. The child tax
    // credit and the earned income credit need no page.
    pub(super) fn credits_hub(&self, review: bool) -> Reply {
        let d = &self.draft;
        let visited = |topic: TaxSection| d.finished_sections.contains(&topic);
        let students = d.credits.as_ref().map_or(0, |c| c.education.len());
        let saver = d.credits.as_ref().is_some_and(|c| c.saver.is_some());
        let result = estimate(d);
        let items = self.remaining_items(&result);
        let planned = planned_sections(d);
        let offered = self.credits_hub_sections();
        let hub_topics = hub_topics(TaxSection::Credits);
        let walk = walk_topics(d, TaxSection::Credits, &offered);
        let suggested = hub_suggestion(&hub_topics, &offered, &items, &walk, self.after_credits());
        let care = Some(if !visited(TaxSection::Care) {
            status("REVIEW_NOT_STARTED")
        } else if let Some(c) = d.credits.as_ref().filter(|c| c.care.active()) {
            let count = c.care.providers.len()
                + c.care.benefits.len()
                + c.care.grace.len()
                + c.care.own_business.len();
            if count > 0 {
                status_count("CARE_RECORDED_SOURCES", count)
            } else {
                status("REVIEW_RECORDED")
            }
        } else {
            status("REVIEW_NONE")
        });
        let education = match students {
            0 if suggested == TaxSection::Education => Some(status("REVIEW_NOT_STARTED")),
            0 if visited(TaxSection::Education) => Some(status("REVIEW_NONE")),
            0 => None,
            n => Some(status_count("REVIEW_STUDENTS", n)),
        };
        let saver_status = if saver {
            Some(status("REVIEW_RECORDED"))
        } else if suggested == TaxSection::Saver {
            Some(status("REVIEW_NOT_STARTED"))
        } else {
            None
        };
        let marketplace = match marketplace_status(d, &result) {
            Some(summary) => Some(summary),
            None if suggested == TaxSection::Marketplace => Some(status("REVIEW_NOT_STARTED")),
            None if visited(TaxSection::Marketplace) => Some(status("REVIEW_NONE")),
            None => None,
        };
        // More credits is never suggested (it covers rare, one-off credits);
        // its row always shows, "None" until something is added.
        let more_credits = more_credit_items(d).len() + d.credit_refusals.len();
        let more_credits_status = Some(match more_credits {
            0 => status("REVIEW_NONE"),
            n => status_count("REVIEW_CREDITS", n),
        });
        let topics = [
            ("SECTION_CARE", TaxSection::Care, care),
            (section_info(TaxSection::Education).1, TaxSection::Education, education),
            (section_info(TaxSection::Saver).1, TaxSection::Saver, saver_status),
            (section_info(TaxSection::Marketplace).1, TaxSection::Marketplace, marketplace),
            (section_info(TaxSection::MoreCredits).1, TaxSection::MoreCredits, more_credits_status),
        ]
        .into_iter()
        .filter_map(|(item, section, status)| {
            status.map(|status| ReviewItem {
                field: None,
                item: libertas_formatted_text(item, &[]),
                status,
                section,
            })
        })
        .collect();
        // Care is a required first visit when facts exist. Its initial row
        // remains Not started; accepted topics still show ordinary attention.
        let attention_topics = hub_topics
            .iter()
            .copied()
            .filter(|topic| *topic != TaxSection::Care || visited(*topic))
            .collect::<Vec<_>>();
        let topics = mark_attention(topics, &attention_topics, &items);
        let topics = with_checklist_rows(
            topics,
            &[
                TaxSection::Children,
                TaxSection::Education,
                TaxSection::Saver,
                TaxSection::Marketplace,
                TaxSection::MoreCredits,
            ],
            &offered,
            &planned,
        );
        let topics = with_every_destination(d, topics, TaxSection::Credits, &offered);
        let allowed_sections = self.allowed_sections();
        let navigation = self.navigation_availability(&allowed_sections);
        let allowed_actions = self.hub_actions(TaxSection::Credits);
        let primary_actions = Self::primary_actions(&allowed_actions);
        Reply::Response(P::Hub {
            crop_insurance: Vec::new(),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CREDITS_PAGE,
            previous: offered[0],
            review,
            estimate: self.estimate_so_far(d, &result, TaxSection::Credits),
            progress: progress_line(
                CREDITS_PAGE,
                &shown_sections(d),
                walk.is_empty().then(|| done_with(TaxSection::Credits)),
            ),
            told: told_us(d, TaxSection::Credits),
            topics: hub_list(TaxSection::Credits, topics),
            previous_section: previous_section_row(self.before_credits()),
            navigation,
            suggested_destination: suggested.into(),
            primary_actions,
            go_to_actions: alloc::vec![libertas_macros::variant_index!(P::Navigate) as i32],
            allowed_actions,
            suggested,
            allowed_sections,
        })
    }
    // The credits hub's last topic, opened with its complete value:
    // one list item per FederalCredits field this page exposes, or an empty
    // list.
    pub(super) fn more_credits_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        if d.credits.is_none() {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        if matches!(
            d.credits.as_ref().unwrap().foreign_tax,
            ForeignTaxClaim::Statements { eligible: None }
        ) {
            return Reply::Response(P::BeginMoreCredits {
                credit_income_sources: crate::tax_credit_bonds::income_sources(d),
                allowed_credit_owners: self.allowed_owners(),
                cookie: d.cookie.clone(),
                revision: d.revision,
                page: CREDITS_PAGE,
                previous: CREDITS_PAGE,
                review,
                estimate: self.estimate_now(TaxSection::MoreCredits).map(Into::into),
                warnings: Vec::new(),
                progress: Some(self.topic_progress(CREDITS_PAGE, Some(TaxSection::MoreCredits))),
                allowed_owners: self.allowed_owners(),
                foreign_businesses: self.foreign_business_choices(),
                estate_foreign_sources: crate::estate_foreign::choices(d),
                capital_sales: self.capital_sale_choices(),
                foreign_situations: foreign_situation_choices(),
                mortgage_loans: self.mortgage_choices(),

                document_facts: DocumentFacts { year: 2026 },
                businesses: self.income_businesses(),
                farms: self.income_farms(),
                clean_vehicle_properties: crate::clean_vehicle::property_choices(d),
                rentals: self.income_rentals(),
                research_businesses: crate::business_credit_page::research_businesses(d),
                entered_credits: crate::BusinessCredit::entered_choices(),
                carryback_credits: crate::BusinessCredit::ALL
                    .iter()
                    .filter(|c| c.carryback_allowed())
                    .map(|c| *c as i32)
                    .collect(),
                allowed_carry_filers: if joint(d) { alloc::vec![0, 1, 2] } else { alloc::vec![0] },
                initial_credit_elections: alloc::vec![0, 1, 2],
                initial_carbon_oxide_origins: Vec::new(),
                initial_renewable_facility_origins: Vec::new(),
                initial_research_choices: alloc::vec![0],
                value: crate::federal_model::MoreCreditsInitial {
                    documents: self.credit_document_views(),
                    credits: more_credit_items(d)
                        .into_iter()
                        .chain(d.credit_refusals.iter().copied().map(MoreCredit::from))
                        .map(crate::federal_model::MoreCreditInitial::from)
                        .chain(core::iter::once(
                            crate::federal_model::MoreCreditInitial::ForeignTax(
                                crate::extended_model::ForeignTaxInitial::Statements {
                                    eligible: None,
                                },
                            ),
                        ))
                        .collect(),
                },
            });
        }
        Reply::Edit(P::SaveMoreCredits {
            allowed_credit_owners: self.allowed_owners(),
            credit_income_sources: crate::tax_credit_bonds::income_sources(d),
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CREDITS_PAGE,
            previous: CREDITS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::MoreCredits),
            warnings: Vec::new(),
            document_questions: self.credit_questions_link(),
            progress: Some(self.topic_progress(CREDITS_PAGE, Some(TaxSection::MoreCredits))),
            allowed_owners: self.allowed_owners(),
            foreign_businesses: self.foreign_business_choices(),
            estate_foreign_sources: crate::estate_foreign::choices(d),
            capital_sales: self.capital_sale_choices(),
            foreign_situations: foreign_situation_choices(),
            mortgage_loans: self.mortgage_choices(),

            document_facts: DocumentFacts { year: 2026 },
            businesses: self.income_businesses(),
            farms: self.income_farms(),
            clean_vehicle_properties: crate::clean_vehicle::property_choices(d),
            rentals: self.income_rentals(),
            research_businesses: crate::business_credit_page::research_businesses(d),
            entered_credits: crate::BusinessCredit::entered_choices(),
            carryback_credits: crate::BusinessCredit::ALL
                .iter()
                .filter(|c| c.carryback_allowed())
                .map(|c| *c as i32)
                .collect(),
            allowed_carry_filers: if joint(d) { alloc::vec![0, 1, 2] } else { alloc::vec![0] },
            initial_credit_elections: alloc::vec![0, 1, 2],
            initial_carbon_oxide_origins: Vec::new(),
            initial_renewable_facility_origins: Vec::new(),
            initial_research_choices: alloc::vec![0],
            value: MoreCredits {
                documents: self.credit_document_views(),
                answers: credit_questions::answered_rows(d),
                credits: more_credit_items(d)
                    .into_iter()
                    .chain(d.credit_refusals.iter().copied().map(MoreCredit::from))
                    .collect(),
            },
        })
    }
    // More credits: the list onto the existing FederalCredits
    // fields (apply_more_credits); a variant absent from the list clears its
    // field(s).
    pub(super) fn more_credits_candidate(
        &self,
        value: &MoreCredits,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        for (index, item) in value.credits.iter().enumerate() {
            if let MoreCredit::CleanVehiclesAndRefueling { clean_vehicles: Some(claim), .. } = item
                && let Some(issue) = crate::clean_vehicle::source_issue(
                    &self.draft,
                    &FederalCleanVehicleClaim::from(claim.clone()),
                )
            {
                return Err(issue
                    .in_field("clean_vehicles")
                    .in_list("credits", index)
                    .in_field("value")
                    .into());
            }
            if let MoreCredit::MinimumTax(histories) = item
                && let Some(issue) = crate::MinimumTaxHistories::from(histories.clone())
                    .issue(self.draft.setup.as_ref().unwrap().filing_status())
            {
                return Err(issue.in_list("credits", index).in_field("value").into());
            }
        }
        // The server twin of the W-2 choice the adoption rows make from the
        // return's W-2s: an employer who is not that employee's W-2 on this
        // return is unanswered (the stored key matches no catalog member).
        let wages = employee_wage_choices(&self.draft);
        for (credit_index, item) in value.credits.iter().enumerate() {
            if let MoreCredit::Adoption { adoptions, .. } = item {
                for (child_index, child) in adoptions.iter().enumerate() {
                    if let Some(payment_index) = child
                        .payments()
                        .iter()
                        .position(|p| !wages.iter().any(|w| w.id == p.wage && w.owner == p.owner))
                    {
                        return Err(ValidationIssue::new("ADOPTION_EMPLOYER_NOT_THEIR_W2")
                            .at_field("wage")
                            .in_list("payments", payment_index)
                            .in_field("benefits")
                            .in_list("adoptions", child_index)
                            .in_list("credits", credit_index)
                            .in_field("value")
                            .into());
                    }
                }
            }
        }
        // The server twins of the Form 1116 claim's client rules, and the
        // owner check of a business run abroad, which the page does not carry.
        let abroad: Vec<(i64, Owner)> = self
            .draft
            .income
            .iter()
            .filter(|entry| {
                matches!(&entry.income, FederalIncome::Business { data }
                    if matches!(data.source, crate::BusinessSource::ForeignServices { .. }))
            })
            .map(|entry| (entry.id, entry.owner.owned()))
            .collect();
        if let Some(issue) = value.credits.iter().enumerate().find_map(|(index, item)| match item {
            MoreCredit::ForeignTax(page) => {
                crate::form_1116::claim_rule(&page.clone().into(), &abroad)
                    .map(|issue| issue.in_list("credits", index).in_field("value"))
            }
            _ => None,
        }) {
            return Err(issue.into());
        }
        // The server twin of the Form 8862 credit's range attributes (T2-46).
        if value
            .credits
            .iter()
            .any(|item| matches!(item, MoreCredit::AfterDisallowance(claim) if !claim.valid()))
        {
            return Err(crate::interview::status(
                "MESSAGE_FORM_8862_THE_DISALLOWED_YEAR_MUST_BE_1997_THROUGH_2025_AND_THE_DAYS_AT_HOME",
            ).into());
        }
        // The server twins of the Form 3800 carryover rules.
        for (index, item) in value.credits.iter().enumerate() {
            if let MoreCredit::BusinessCredits(history) = item {
                let record = crate::BusinessCreditHistory::from(history.clone());
                if let Some(issue) = record.issue().or_else(|| record.source_issue(&self.draft)) {
                    return Err(issue.in_list("credits", index).in_field("value").into());
                }
            }
        }
        // The server twin of the home kerosene rule.
        for (index, item) in value.credits.iter().enumerate() {
            if let MoreCredit::HomeKerosene(fuel) = item
                && fuel.actual_cost < fuel.credit_cents()
            {
                return Err(ValidationIssue::new("KEROSENE_COST_BELOW_CREDIT")
                    .at_field("actual_cost")
                    .in_list("credits", index)
                    .in_field("value")
                    .into());
            }
        }
        if value
            .credits
            .iter()
            .any(|item| matches!(item, MoreCredit::HomeKerosene(fuel) if !fuel.valid()))
        {
            return Err(crate::interview::status(
                "MESSAGE_FORM_4136_ENTER_THE_QUALIFIED_GALLONS_AT_MOST_2_000_000_AND_THEIR_ACTUAL_COS",
            ).into());
        }
        // The owner list (allowed_owners, never the echoed list).
        let owners = self.allowed_owners();
        for (credit_index, item) in value.credits.iter().enumerate() {
            if let MoreCredit::ForeignTax(crate::ForeignTaxPage::Deduction { taxes }) = item
                && let Some(tax_index) = taxes.iter().position(|row| matches!(row.income,
                    crate::ForeignTaxIncomePage::Earned { owner } if !owners.contains(&(owner as i32))))
            {
                return Err(ValidationIssue::new("FOREIGN_DEDUCTION_OWNER_NOT_ALLOWED")
                    .at_field("owner")
                    .in_field("income")
                    .in_list("taxes", tax_index)
                    .in_list("credits", credit_index)
                    .in_field("value")
                    .into());
            }
        }
        let mut draft = self.draft.clone();
        if draft.credits.is_none() {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        apply_more_credits(&mut draft, &value.credits);
        // A selected notice must survive Save just as a selected refusal does.
        // The result owner omits its benefit; it does not erase the answer.
        draft.credit_refusals = value.credits.iter().filter_map(MoreCredit::refusal).collect();
        if draft
            .credit_refusals
            .iter()
            .enumerate()
            .any(|(i, item)| draft.credit_refusals[..i].contains(item))
        {
            return Err(status("MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE").into());
        }
        finish_section(&mut draft, TaxSection::MoreCredits);
        credit_questions::apply_answers(&self.draft, &mut draft, &value.answers)
            .map_err(|failure| failure.map_field_prefix(&[], &["value"]))?;
        credit_questions::prune(&mut draft);
        self.credit_document_answers_candidate(draft, &value.documents)
            .map_err(|failure| failure.map_field_prefix(&[], &["value"]))
    }
}

impl Interview {
    pub(super) fn save_more_credits(
        &self,
        value: &MoreCredits,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != CREDITS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.expense_owner_candidate(
            self.more_credits_candidate(value),
            TaxSection::MoreCredits,
            value.credits.iter().position(|credit| matches!(credit, MoreCredit::ForeignTax(_))),
        )
    }
}
