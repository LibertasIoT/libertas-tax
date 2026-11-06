// schedule_a items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Itemized facts remain available regardless of the computed deduction method.
pub(super) fn itemized_facts(d: &FederalDraft) -> Option<&ItemizedExpenses> {
    d.deductions.as_ref().map(|v| &v.itemized)
}
pub(super) fn itemized_facts_mut(d: &mut FederalDraft) -> Option<&mut ItemizedExpenses> {
    d.deductions.as_mut().map(|v| &mut v.itemized)
}
// Only applicable topics enter the walk; automatic standard selection does not
// discard entered facts or remove the correction path.
pub(super) fn itemizing(d: &FederalDraft) -> bool {
    d.deductions.as_ref().is_some_and(|ded| {
        ded.itemize_override.is_some()
            || d.spouse_itemizes()
            || MISSED_BENEFITS.iter().any(|(bit, topic)| {
                d.often_missed.benefits & bit.0 != 0 && OPEN_ITEMIZED_TOPICS.contains(topic)
            })
            || !ded.mortgage.is_empty()
            || ded.investment_interest.is_some()
            || gift_page_entered(ded)
            || (d.checklist.paid & PaidTopics::GIFTS.0 != 0 && !gifts_entered(d))
            || !ded.annuity_losses.is_empty()
            || !ded.itemized.other_lines.is_empty()
            || ITEMIZABLE
                .iter()
                .any(|(catalog, bit)| catalog_bits(&checklist_value(d), *catalog) & bit != 0)
            || gambling_losses(d) != 0
            || OPEN_ITEMIZED_TOPICS
                .iter()
                .any(|topic| itemized_topic_present(d, &ded.itemized, *topic))
    })
}
pub(super) fn no_itemized_facts() -> ItemizedExpenses {
    ItemizedExpenses {
        investment_income_tax: crate::InvestmentIncomeTax::Unresolved,
        medical: 0,
        long_term_care: Vec::new(),
        personal_tax: PersonalTaxElection::None,
        real_estate_taxes: 0,
        other_lines: Vec::new(),
        personal_property_taxes: 0,
        investment_property_taxes: 0,
        investment_property_tax_owners: None,
        income_tax_owners: None,
    }
}
// A saved topic retains its visit when the computed deduction method changes.
pub(super) fn itemized_topic_visited(d: &FederalDraft, topic: TaxSection) -> bool {
    d.finished_sections.contains(&topic)
}
// The people a long-term-care premium can insure: you, your spouse on a
// joint return and the people you support, each with the age at the end of
// 2026 that sets the premium limit. The catalog reuses the student type.
pub(crate) fn medical_people(d: &FederalDraft) -> Vec<(PersonChoice, u32)> {
    // IRC 213(d)(10): the age attained before the close of the year, day-exact.
    let age = |birth: u32| crate::federal_model::age_at_year_end(birth).min(120);
    crate::federal_rules::people_students(d)
        .into_iter()
        .filter_map(|person| {
            let birth = match person.id {
                -1 => d.people.as_ref()?.taxpayer.birth_date,
                -2 => d.people.as_ref()?.spouse.as_ref()?.birth_date,
                id => d.dependents().iter().find(|v| v.id == id)?.birth_date,
            };
            Some((person, age(birth)))
        })
        .collect()
}
// The age at the end of 2026 that sets a person's premium limit, by the
// person's identity (-1 you, -2 your spouse, otherwise a dependent); None for
// someone not on the return.
pub(crate) fn person_age(d: &FederalDraft, person: i64) -> Option<u32> {
    medical_people(d).into_iter().find(|(p, _)| p.id == person).map(|(_, age)| age)
}
// A stored premium as its page row: the insured person's identity, the EnumSource key.
// None when the record is not one the page writes (another person or an
// unconfirmed policy).
pub(super) fn care_premium_item(
    record: &MedicalCarePremium,
    people: &[(PersonChoice, u32)],
) -> Option<CarePremium> {
    let person = people.iter().find(|(p, _)| p.id == record.person)?.0.id;
    Some(CarePremium { person, premiums: record.premiums, qualified: record.qualified.clone() })
}
// The Taxes page's SALT election onto the stored one: an income tax of zero
// is no election, so the page's default saves as `None` and reopens the same.
pub(super) fn personal_tax_record(
    kind: &SaltElection,
    kept: Option<&PersonalTaxElection>,
) -> PersonalTaxElection {
    match kind {
        SaltElection::IncomeTax { paid: 0 } => PersonalTaxElection::None,
        SaltElection::IncomeTax { paid } => PersonalTaxElection::Income {
            paid: *paid,
            allocation: match kept {
                Some(PersonalTaxElection::Income { allocation, .. }) => allocation.clone(),
                _ => TaxIncomeAllocation::CurrentReturn,
            },
        },
        SaltElection::SalesTax { paid } => PersonalTaxElection::Sales { paid: *paid },
    }
}
pub(super) fn salt_item(record: &PersonalTaxElection) -> SaltElection {
    match record {
        PersonalTaxElection::None => SaltElection::IncomeTax { paid: 0 },
        PersonalTaxElection::Income { paid, .. } => SaltElection::IncomeTax { paid: *paid },
        PersonalTaxElection::Sales { paid } => SaltElection::SalesTax { paid: *paid },
    }
}
// The server twins of the deductions page's client rules (T3-deduct), in
// the client's order, then the records' own checks by name.
pub(super) fn deductions_rule(d: &FederalDraft, value: &Deductions) -> Result<(), SaveFailure> {
    let events = &value.casualties;
    for (event_index, event) in events.iter().enumerate() {
        if let Some((property_index, _)) =
            event.properties.iter().enumerate().find(|(_, p)| p.after > p.before)
        {
            return Err(ValidationIssue::new("CASUALTY_AFTER_ABOVE_BEFORE")
                .at_field("after")
                .in_list("properties", property_index)
                .in_list("casualties", event_index)
                .in_field("value")
                .into());
        }
        if let Some((property_index, _)) =
            event.properties.iter().enumerate().find(|(_, p)| p.received > p.reimbursement)
        {
            return Err(ValidationIssue::new("CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT")
                .at_field("received")
                .in_list("properties", property_index)
                .in_list("casualties", event_index)
                .in_field("value")
                .into());
        }
        let declared = !matches!(event.declaration, CasualtyDeclaration::Undeclared);
        // External format: a declared disaster area uses a five-digit ZIP code.
        if declared != (event.zip.len() == 5) || (!declared && !event.zip.is_empty()) {
            return Err(ValidationIssue::new("CASUALTY_ZIP")
                .at_field("zip")
                .in_list("casualties", event_index)
                .in_field("value")
                .into());
        }
        if !PersonalCasualty::from(event.clone()).valid() {
            return Err(libertas_formatted_text(
                "MESSAGE_CHECK_CASUALTY_EVENT_FACTS",
                &[LibertasMessageArgument::LiteralText(&event.label)],
            )
            .into());
        }
    }
    if !allowed_repayment(d).contains(&repayment_index(&value.repayment)) {
        return Err(ValidationIssue::new(
            "MESSAGE_THE_REPAYMENT_COMPARISON_IS_ASKED_ONLY_WHEN_YOUR_INCOME_ENTRIES_RECORD_MORE",
        )
        .at_field("repayment")
        .in_field("value")
        .into());
    }
    if let Some(issue) = value.repayment.issue() {
        return Err(issue.in_field("repayment").in_field("value").into());
    }
    Ok(())
}
// The gift page's less common rows (T3-deduct), as stored.
pub(super) struct LessCommonGifts {
    pub(super) hosted_students: Vec<crate::HostedStudent>,
    pub(super) whaling_captains: Vec<crate::WhalingCaptainClaim>,
}
pub(super) fn less_common_gifts(rows: &[LessCommonGift]) -> LessCommonGifts {
    let mut out = LessCommonGifts { hosted_students: Vec::new(), whaling_captains: Vec::new() };
    for row in rows {
        match row {
            LessCommonGift::HostedStudent(student) => out.hosted_students.push(student.clone()),
            LessCommonGift::WhalingCaptain(c) => {
                out.whaling_captains.push(crate::WhalingCaptainClaim {
                    owner: c.owner,
                    captain_record_reference: c.captain_record_reference.clone(),
                    aewc_report_reference: c.aewc_report_reference.clone(),
                    trip_log_reference: c.trip_log_reference.clone(),
                    expenses: c.expenses.clone(),
                    eligibility_and_records_confirmed: c.eligibility_and_records_confirmed,
                })
            }
        }
    }
    out
}
pub(super) fn less_common_rows(ded: &FederalDeductions) -> Vec<LessCommonGift> {
    let mut rows: Vec<LessCommonGift> =
        ded.hosted_students.iter().cloned().map(LessCommonGift::HostedStudent).collect();
    rows.extend(ded.whaling_captains.iter().map(|c| {
        LessCommonGift::WhalingCaptain(WhalingCaptain {
            owner: c.owner,
            captain_record_reference: c.captain_record_reference.clone(),
            aewc_report_reference: c.aewc_report_reference.clone(),
            trip_log_reference: c.trip_log_reference.clone(),
            expenses: c.expenses.clone(),
            eligibility_and_records_confirmed: c.eligibility_and_records_confirmed,
        })
    }));
    rows
}
// Source owners return local fields; retain the submitted mixed-list indices.
pub(super) fn less_common_issue(value: &GiftCarryovers, owners: &[i32]) -> Option<ValidationIssue> {
    let rows = &value.less_common;
    let gifts = less_common_gifts(rows);
    let captains: Vec<_> = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| match row {
            LessCommonGift::WhalingCaptain(captain) => Some((index, captain)),
            _ => None,
        })
        .collect();
    let students: Vec<_> = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| match row {
            LessCommonGift::HostedStudent(student) => Some((index, student)),
            _ => None,
        })
        .collect();
    for (position, (index, captain)) in captains.iter().enumerate() {
        if captains[..position].iter().any(|(_, prior)| prior.owner == captain.owner) {
            return Some(
                ValidationIssue::new("WHALING_CAPTAIN_TWICE")
                    .at_field("owner")
                    .in_list("less_common", *index),
            );
        }
    }
    for (position, (index, student)) in students.iter().enumerate() {
        if students[..position].iter().any(|(_, prior)| prior.label == student.label) {
            return Some(
                ValidationIssue::new("HOSTED_STUDENT_TWICE")
                    .at_field("label")
                    .in_list("less_common", *index),
            );
        }
    }
    for (index, captain) in &captains {
        if !owners.contains(&(captain.owner as i32)) {
            return Some(
                ValidationIssue::new(
                    "MESSAGE_A_WHALING_CAPTAIN_MUST_BE_YOU_OR_YOUR_SPOUSE_ON_A_JOINT_RETURN",
                )
                .at_field("owner")
                .in_list("less_common", *index),
            );
        }
    }
    for (index, volunteer) in value.volunteer.iter().enumerate() {
        if let Some(issue) = volunteer.issue() {
            return Some(issue.in_list("volunteer", index));
        }
    }
    for (index, student) in &students {
        if let Some(issue) = student.issue() {
            return Some(issue.in_list("less_common", *index));
        }
    }
    for ((index, _), captain) in captains.iter().zip(&gifts.whaling_captains) {
        if let Some(issue) = captain.issue() {
            return Some(issue.in_list("less_common", *index));
        }
    }
    None
}
// The retirement income an unrecovered annuity cost can belong to: each
// non-IRA retirement entry of a filer who died (the other itemized page's
// catalog, T3-deduct).
pub(super) fn annuity_contracts(d: &FederalDraft) -> Vec<AnnuityContract> {
    let Some(people) = d.people.as_ref() else {
        return Vec::new();
    };
    d.income
        .iter()
        .filter(|e| {
            people.person(e.owner.owned()).is_some_and(|p| p.death_date().is_some())
                && e.income.retirement().is_some_and(|data| !data.kind.ira())
        })
        .map(|e| AnnuityContract { id: e.id, label: income_issue_context(e).label().encode() })
        .collect()
}
// The amounts each itemized topic holds, for its hub row. Charity
// carryovers live on FederalDeductions directly (M3-P10), not in the route's
// ItemizedExpenses, so this reads the draft too.
// The gambling losses entered on the income topic, before any limit: the
// itemized hub's derived row (C7), which also opens the deductions page on
// Compare as an itemizable tick does.
pub(super) fn gambling_losses(d: &FederalDraft) -> i64 {
    d.income
        .iter()
        .map(|e| match &e.income {
            FederalIncome::Gambling { data } => crate::FederalGambling::from(data).losses,
            _ => 0,
        })
        .sum()
}
pub(super) fn itemized_topic_present(
    d: &FederalDraft,
    facts: &ItemizedExpenses,
    topic: TaxSection,
) -> bool {
    if topic == TaxSection::Medical {
        facts.medical != 0 || facts.long_term_care.iter().any(|premium| premium.premiums != 0)
    } else if topic == TaxSection::GiftCarryovers {
        d.deductions.as_ref().is_some_and(gift_page_entered)
    } else {
        itemized_topic_amount(d, facts, topic).is_none_or(|amount| amount != 0)
    }
}
pub(super) fn itemized_topic_amount(
    d: &FederalDraft,
    facts: &ItemizedExpenses,
    topic: TaxSection,
) -> Option<i64> {
    Some(match topic {
        TaxSection::Taxes => {
            facts.personal_tax.paid() + facts.real_estate_taxes + facts.personal_property_taxes
        }
        TaxSection::Mortgage => d
            .deductions
            .as_ref()
            .map_or(0, |v| v.mortgage_loans().unwrap_or_default().iter().map(|l| l.interest).sum()),
        TaxSection::GiftCarryovers => {
            let Some(v) = d.deductions.as_ref() else { return Some(0) };
            let gifts = v.gifts().ok()?;
            v.volunteer_expenses.iter().map(crate::VolunteerExpense::cents).sum::<i64>()
                + v.hosted_students.iter().map(crate::HostedStudent::allowed).sum::<i64>()
                + v.whaling_captains
                    .iter()
                    .map(crate::WhalingCaptainClaim::statutory_claim_cents)
                    .sum::<i64>()
                + v.charity_carryovers.iter().map(|c| c.balance).sum::<i64>()
                + gifts.iter().map(|g| g.claim(v.basis_election).1).sum::<i64>()
        }
        TaxSection::OtherItemized => {
            d.deductions
                .as_ref()
                .and_then(FederalDeductions::investment_interest)
                .map_or(0, |v| v.paid + v.carryover)
                + facts.investment_property_taxes
        }
        _ => 0,
    })
}
pub(super) fn itemized_entered(d: &FederalDraft, topic: TaxSection) -> bool {
    (topic == TaxSection::OtherItemized
        && d.deductions.as_ref().is_some_and(|v| v.investment_interest.is_some()))
        || itemized_facts(d).is_some_and(|facts| itemized_topic_present(d, facts, topic))
}

impl Interview {
    // The deductions page edits complete facts and the optional method override.
    pub(super) fn deductions_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let gifts = d.deductions.as_ref().map_or(0, |v| v.cash_charity);
        let Some(ded) = d.deductions.as_ref() else {
            return problem_key("PROBLEM_UNDISPLAYABLE_DEDUCTION_RECORD");
        };
        let casualties = match ded
            .personal_casualties
            .iter()
            .cloned()
            .enumerate()
            .map(|(event, value)| {
                crate::PersonalCasualtyPage::try_from(value).map_err(|(key, property)| {
                    ValidationIssue::new(key)
                        .at_field("home_link")
                        .in_list("properties", property)
                        .in_list("casualties", event)
                        .in_field("value")
                })
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(casualties) => casualties,
            Err(issue) => return refused(issue),
        };
        Reply::Edit(P::SaveDeductions {
            user: self.user,
            home_sites: self.home_site_choices(),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: DEDUCTIONS_PAGE,
            previous: self.previous_shown(TaxSection::Deductions),
            review,
            estimate: self.estimate_now(TaxSection::Deductions),
            warnings: Vec::new(),
            progress: self.progress(DEDUCTIONS_PAGE),
            allowed_repayment: allowed_repayment(d),
            document_facts: DocumentFacts { year: 2026 },
            value: Deductions {
                itemize_override: ded.itemize_override,
                cash_gifts: gifts,
                fund_cash_gifts: d.deductions.as_ref().map_or(0, |v| v.fund_cash_charity),
                other_cash_gifts: d.deductions.as_ref().map_or(0, |v| v.other_cash_charity),
                car_loans: ded
                    .vehicle_loans
                    .iter()
                    .map(crate::VehicleLoanDocumentView::from)
                    .collect(),
                car_refunds: ded
                    .vehicle_loans
                    .iter()
                    .filter(|loan| loan.model().refund > 0)
                    .map(|loan| crate::VehicleRefundDocumentView {
                        id: loan.id,
                        value: loan.document.clone(),
                        origin: ded
                            .vehicle_recoveries
                            .iter()
                            .find(|row| row.id == loan.id)
                            .map(|row| row.origin.clone()),
                    })
                    .collect(),
                casualties,
                repayment: ded.repayment.clone(),
            },
        })
    }
    // What the itemized hub offers: the deductions page, the topics that
    // open (home mortgage and charity carryovers are listed but open with
    // M3-P10) and the next section ("Continue").
    pub(super) fn itemized_hub_sections(&self) -> Vec<i32> {
        let mut sections = alloc::vec![DEDUCTIONS_PAGE, CHECKLIST_PAGE];
        sections.extend(OPEN_ITEMIZED_TOPICS.iter().map(|t| *t as i32));
        if gambling_losses(&self.draft) != 0 {
            sections.push(GAMBLING_PAGE);
        }
        sections.push(self.after_deductions() as i32);
        sections
    }
    // The itemized hub: it suggests the first topic that needs attention,
    // then the first planned topic not yet visited (M3-P19), then the next
    // shown section. A topic with amounts shows their total; a visited empty
    // one shows "None"; the suggested one "Not started".
    pub(super) fn itemized_hub(&self, review: bool) -> Reply {
        let d = &self.draft;
        let Some(facts) = itemized_facts(d) else {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        };
        // Attention first, then the walk: the planned topics not yet
        // visited (M3-P19; M3-P9 walked every unvisited topic).
        let result = estimate(d);
        let items = self.remaining_items(&result);
        let planned = planned_sections(d);
        let offered = self.itemized_hub_sections();
        let hub_topics = hub_topics(TaxSection::Itemized);
        let walk = walk_topics(d, TaxSection::Itemized, &offered);
        let suggested =
            hub_suggestion(&hub_topics, &offered, &items, &walk, self.after_deductions());
        let medical_amount =
            match crate::health::medical_source_amount(facts.medical, &facts.long_term_care) {
                Ok(amount) => amount,
                Err(issue) => return refused(crate::validation::SaveFailure::from(issue)),
            };
        let topics = ITEMIZED_TOPICS
            .iter()
            .filter_map(|topic| {
                let status = if !OPEN_ITEMIZED_TOPICS.contains(topic) {
                    status("REVIEW_NOT_AVAILABLE")
                } else {
                    match if *topic == TaxSection::Medical {
                        Some(medical_amount)
                    } else {
                        itemized_topic_amount(d, facts, *topic)
                    } {
                        None => status("REVIEW_NEEDS_ATTENTION"),
                        Some(0) if *topic == suggested => status("REVIEW_NOT_STARTED"),
                        Some(0) if itemized_topic_visited(d, *topic) => status("REVIEW_NONE"),
                        Some(0) => return None,
                        Some(amount) => status_usd("REVIEW_USD_AMOUNT", amount),
                    }
                };
                Some(ReviewItem {
                    field: None,
                    item: section_message(*topic).encode(),
                    status,
                    section: *topic,
                })
            })
            .collect();
        let mut topics = mark_attention(topics, &OPEN_ITEMIZED_TOPICS, &items);
        // Gambling losses are entered on the income topic; the hub shows
        // them as a derived row that opens it (C7).
        let losses = gambling_losses(d);
        if losses != 0 {
            topics.push(ReviewItem {
                field: None,
                item: status("REVIEW_GAMBLING_LOSSES_LABEL"),
                status: status_usd("REVIEW_GAMBLING_LOSSES_FROM", losses),
                section: TaxSection::Gambling,
            });
        }
        let topics = with_checklist_rows(topics, &hub_topics, &offered, &planned);
        let topics = with_every_destination(d, topics, TaxSection::Itemized, &offered);
        let allowed_sections = self.allowed_sections();
        let navigation = self.navigation_availability(&allowed_sections);
        let allowed_actions = self.hub_actions(TaxSection::Itemized);
        let primary_actions = Self::primary_actions(&allowed_actions);
        Reply::Response(P::Hub {
            crop_insurance: Vec::new(),
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ITEMIZED_PAGE,
            previous: DEDUCTIONS_PAGE,
            review,
            estimate: self.estimate_so_far(d, &result, TaxSection::Itemized),
            progress: progress_line(
                ITEMIZED_PAGE,
                &shown_sections(d),
                Some(if walk.is_empty() {
                    done_with(TaxSection::Itemized)
                } else {
                    section_message(TaxSection::Itemized)
                }),
            ),
            told: told_us(d, TaxSection::Deductions),
            topics: hub_list(TaxSection::Itemized, topics),
            previous_section: previous_section_row(DEDUCTIONS_PAGE),
            navigation,
            suggested_destination: suggested.into(),
            primary_actions,
            go_to_actions: alloc::vec![libertas_macros::variant_index!(P::Navigate) as i32],
            allowed_actions,
            suggested,
            allowed_sections,
        })
    }
    // The medical page, opened with its complete value: the stored expenses
    // and premiums, each premium naming its insured person in the catalog.
    pub(super) fn medical_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let people = medical_people(d);
        let Some(facts) = itemized_facts(d) else {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        };
        let Some(long_term_care) = facts
            .long_term_care
            .iter()
            .map(|p| care_premium_item(p, &people))
            .collect::<Option<Vec<_>>>()
        else {
            return problem_key("PROBLEM_UNDISPLAYABLE_LONG_TERM_CARE_PREMIUM");
        };
        Reply::Edit(P::SaveMedical {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ITEMIZED_PAGE,
            previous: ITEMIZED_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Medical),
            warnings: Vec::new(),
            progress: self.topic_progress(ITEMIZED_PAGE, Some(TaxSection::Medical)),
            value: Medical {
                expenses: facts.medical,
                long_term_care,
                people: people.iter().map(|(p, _)| p.clone()).collect(),
            },
        })
    }
    // The taxes page, opened with its complete value.
    pub(super) fn taxes_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let Some(facts) = itemized_facts(d) else {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        };
        Reply::Edit(P::SaveTaxes {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ITEMIZED_PAGE,
            previous: ITEMIZED_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::Taxes),
            warnings: Vec::new(),
            progress: self.topic_progress(ITEMIZED_PAGE, Some(TaxSection::Taxes)),
            value: Taxes {
                investment_income_tax: facts.investment_income_tax.clone(),
                kind: salt_item(&facts.personal_tax),
                real_estate: facts.real_estate_taxes,
                personal_property: facts.personal_property_taxes,
                income_tax_owners: facts.income_tax_owners,
            },
        })
    }
    // Borrowing survives the deduction route; Schedule A-only controls retain their own eligibility.
    pub(super) fn other_itemized_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let Some(deductions) = d.deductions.as_ref() else {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        };
        let facts = itemized_facts(d);
        let mut investment_interest =
            deductions.investment_interest.as_ref().map(crate::InvestmentBorrowingPage::from);
        if let Some(value) = &mut investment_interest {
            for loan in &mut value.loans {
                loan.allowed_origin_choices = crate::investment_interest::loan_origin_choices(
                    d,
                    loan.source,
                    loan.carry,
                    loan.amt_carry,
                );
            }
        }
        // Each unrecovered annuity cost names its retirement income in the
        // catalog (T3-deduct).
        let contracts = annuity_contracts(d);
        let Some(annuity_losses) = d
            .deductions
            .as_ref()
            .map_or(&[][..], |v| &v.annuity_losses)
            .iter()
            .map(|loss| {
                let contract = contracts.iter().find(|c| c.id == loss.income_id)?.id;
                Some(FinalAnnuityLoss {
                    contract,
                    started: loss.started,
                    cost: loss.cost,
                    recovered: loss.recovered,
                    refunds: loss.refunds,
                    eligible: loss.eligible,
                })
            })
            .collect::<Option<Vec<_>>>()
        else {
            return problem_key("PROBLEM_UNDISPLAYABLE_STALE_ANNUITY_COST");
        };
        Reply::Edit(P::SaveOtherItemized {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: if itemizing(d) { ITEMIZED_PAGE } else { DEDUCTIONS_PAGE },
            previous: if itemizing(d) { ITEMIZED_PAGE } else { DEDUCTIONS_PAGE },
            review,
            estimate: self.estimate_now(TaxSection::OtherItemized),
            warnings: Vec::new(),
            progress: self.topic_progress(
                if itemizing(d) { ITEMIZED_PAGE } else { DEDUCTIONS_PAGE },
                Some(TaxSection::OtherItemized),
            ),
            allowed_owners: self.allowed_owners(),
            interest_sources: crate::investment_interest::sources(d),
            investment_k1_sources: crate::investment_interest::k1_sources(d),
            investment_origin_choices: crate::investment_interest::origin_choices(d),
            value: OtherItemized {
                investment_interest,
                investment_property_taxes: facts.map_or(0, |f| f.investment_property_taxes),
                investment_property_tax_owners: facts
                    .and_then(|f| f.investment_property_tax_owners),
                annuity_losses,
                other_lines: facts.map_or_else(Vec::new, |f| {
                    f.other_lines.iter().cloned().map(Into::into).collect()
                }),
                contracts,
                bond_instruments: crate::investment_statements::bond_instruments(d),
            },
        })
    }
    // Your spouse's standard deduction boxes: a step of the deductions
    // section after its page, only while the rules need them (a separate
    // return on the standard deduction). It asks unanswered until saved and
    // reopens with the saved answer; Back returns to the deductions page.
    pub(super) fn spouse_boxes_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let (cookie, revision, page, previous) =
            (d.cookie.clone(), d.revision, DEDUCTIONS_PAGE, DEDUCTIONS_PAGE);
        let progress = self.progress(DEDUCTIONS_PAGE);
        let estimate = self.estimate_now(TaxSection::Deductions);
        match d.setup.as_ref().and_then(|s| s.separate_spouse.answer()) {
            Some(value) => Reply::Edit(P::SaveSpouseBoxes {
                user: self.user,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: Vec::new(),
                progress,
                value,
            }),
            None => Reply::Response(P::BeginSpouseBoxes {
                cookie,
                revision,
                page,
                previous,
                review,
                estimate: estimate.map(Into::into),
                progress,
            }),
        }
    }
    pub(super) fn spouse_boxes_candidate(
        &self,
        value: SpouseBoxes,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut draft = self.draft.clone();
        match draft.setup.as_mut() {
            Some(setup) if crate::federal_rules::spouse_boxes_apply(&self.draft) => {
                setup.separate_spouse = value.into();
                Ok(draft)
            }
            _ => Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into()),
        }
    }
    pub(super) fn spouse_itemizes_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let (cookie, revision, page, previous) =
            (d.cookie.clone(), d.revision, DEDUCTIONS_PAGE, DEDUCTIONS_PAGE);
        let progress = self.progress(DEDUCTIONS_PAGE);
        let estimate = self.estimate_now(TaxSection::Deductions);
        match d.deductions.as_ref().and_then(|ded| ded.spouse_itemizes) {
            Some(value) => Reply::Edit(P::SaveSpouseItemizes {
                user: self.user,
                cookie,
                revision,
                page,
                previous,
                review,
                estimate,
                warnings: Vec::new(),
                progress,
                value,
            }),
            None => Reply::Response(P::BeginSpouseItemizes {
                cookie,
                revision,
                page,
                previous,
                review,
                estimate: estimate.map(Into::into),
                progress,
            }),
        }
    }
    pub(super) fn spouse_itemizes_candidate(
        &self,
        value: bool,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut draft = self.draft.clone();
        if !separate(&draft) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        let Some(ded) = draft.deductions.as_mut() else {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        };
        ded.spouse_itemizes = Some(value);
        if value && let Some(setup) = draft.setup.as_mut() {
            setup.separate_spouse = crate::SeparateSpouseBoxes::NotAnswered;
        }
        Ok(draft)
    }
    // A method correction preserves all accepted deduction facts.
    pub(super) fn deductions_candidate(
        &self,
        value: &Deductions,
    ) -> Result<FederalDraft, SaveFailure> {
        let d = &self.draft;
        deductions_rule(d, value)?;
        let mut draft = d.clone();
        let Some(deductions) = draft.deductions.as_mut() else {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        };
        deductions.itemize_override = value.itemize_override;
        deductions.cash_charity = value.cash_gifts;
        deductions.fund_cash_charity = value.fund_cash_gifts;
        deductions.other_cash_charity = value.other_cash_gifts;
        Self::document_answer_ids(
            &deductions.vehicle_loans.iter().map(|loan| loan.id).collect::<Vec<_>>(),
            &value.car_loans.iter().map(|loan| loan.id).collect::<Vec<_>>(),
            "car_loans",
        )?;
        for (index, row) in value.car_loans.iter().enumerate() {
            let loan = deductions.vehicle_loans.iter_mut().find(|loan| loan.id == row.id).unwrap();
            let failure = |key, field| {
                ValidationIssue::new(key)
                    .at_field(field)
                    .in_list("car_loans", index)
                    .in_field("value")
            };
            if row.business < 0 || row.business > loan.model().interest {
                return Err(failure("VEHICLE_BUSINESS_ABOVE_INTEREST", "business").into());
            }
            loan.business = row.business;
        }
        Self::document_answer_ids(
            &deductions
                .vehicle_loans
                .iter()
                .filter(|loan| loan.model().refund > 0)
                .map(|loan| loan.id)
                .collect::<Vec<_>>(),
            &value.car_refunds.iter().map(|row| row.id).collect::<Vec<_>>(),
            "car_refunds",
        )?;
        deductions.vehicle_recoveries = value
            .car_refunds
            .iter()
            .filter_map(|row| {
                row.origin
                    .clone()
                    .map(|origin| crate::VehicleInterestRecovery { id: row.id, origin })
            })
            .collect();
        // The page's data answers (T3-deduct): the vehicle loans, the
        // casualty events and the repayment comparison, stored as shown.
        deductions.personal_casualties = value.casualties.iter().cloned().map(Into::into).collect();
        let accepted_events =
            self.draft.deductions.as_ref().map_or(&[][..], |v| &v.personal_casualties);
        let events = &mut deductions.personal_casualties;
        assign_record_ids(
            events.iter_mut().map(|event| &mut event.id),
            accepted_events.iter().map(|event| event.id),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            ValidationIssue::new(key).at_field("id").in_list("casualties", index).in_field("value")
        })?;
        for (index, event) in events.iter_mut().enumerate() {
            let accepted_properties = accepted_events
                .iter()
                .find(|old| old.id == event.id)
                .map_or(&[][..], |old| &old.properties);
            assign_record_ids(
                event.properties.iter_mut().map(|property| &mut property.id),
                accepted_properties.iter().map(|property| property.id),
                &mut draft.next_id,
            )
            .map_err(|(key, _, property)| {
                ValidationIssue::new(key)
                    .at_field("id")
                    .in_list("properties", property)
                    .in_list("casualties", index)
                    .in_field("value")
            })?;
        }

        deductions.repayment = value.repayment.clone();
        finish_section(&mut draft, TaxSection::Deductions);
        Ok(draft)
    }
    // An itemized topic's facts merged into the accepted return; saving it
    // records the topic as visited.
    pub(super) fn itemized_candidate(
        &self,
        topic: TaxSection,
        merge: impl FnOnce(&mut ItemizedExpenses) -> Result<(), SaveFailure>,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut draft = self.draft.clone();
        let Some(facts) = itemized_facts_mut(&mut draft) else {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        };
        merge(facts)?;
        finish_section(&mut draft, topic);
        Ok(draft)
    }
    // Medical expenses and long-term-care premiums (Schedule A line 1). A
    // premium's age comes from its person; choosing the policy row is the
    // qualified-contract confirmation.
    pub(super) fn medical_candidate(&self, value: &Medical) -> Result<FederalDraft, SaveFailure> {
        let people = medical_people(&self.draft);
        if value.people.len() != people.len()
            || value.people.iter().zip(&people).any(|(a, (b, _))| a != b)
        {
            return Err(crate::interview::status(
                "MESSAGE_THE_PEOPLE_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
            )
            .into());
        }
        let mut long_term_care = Vec::with_capacity(value.long_term_care.len());
        for (index, premium) in value.long_term_care.iter().enumerate() {
            let Some((person, _)) = people.iter().find(|(person, _)| person.id == premium.person)
            else {
                return Err(ValidationIssue::new(
                    "MESSAGE_A_PREMIUM_NAMES_SOMEONE_WHO_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_INSURED_PERSON",
                ).at_field("person").in_list("long_term_care", index).in_field("value").into());
            };
            long_term_care.push(MedicalCarePremium {
                person: person.id,
                premiums: premium.premiums,
                qualified: premium.qualified.clone(),
            });
        }
        self.itemized_candidate(TaxSection::Medical, |facts| {
            // Keep availability and the complete people-membership pass ahead
            // of source validation; reconciliation must never add invalid data.
            crate::health::medical_source_amount(value.expenses, &long_term_care)
                .map_err(|issue| issue.in_field("value"))?;
            facts.medical = value.expenses;
            facts.long_term_care = long_term_care;
            Ok(())
        })
    }
    // Source person relationships are checked on native Save only. Normal
    // running estimates retain unfinished/unknown relationships and produce
    // their ordinary financial result; no client rule or options engine owns
    // this requirement. A final allowed zero needs no person answer.
    #[inline(never)]
    pub(super) fn expense_owner_candidate(
        &self,
        candidate: Result<FederalDraft, SaveFailure>,
        section: TaxSection,
        foreign_credit_index: Option<usize>,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let draft = candidate?;
        if crate::deduction_owners::has_unanswered(&draft, section) {
            let result = estimate(&draft);
            if let Some(missing) =
                result.expense_roles.as_deref().and_then(|roles| roles.missing(&draft, section))
            {
                return Err(missing.issue(foreign_credit_index).into());
            }
        }
        Ok(draft)
    }
    // State and local taxes (Schedule A lines 5a–5c).
    pub(super) fn taxes_candidate(&self, value: &Taxes) -> Result<FederalDraft, SaveFailure> {
        let paid = match value.kind {
            SaltElection::IncomeTax { paid } => paid,
            _ => 0,
        };
        if !value
            .investment_income_tax
            .valid(paid, matches!(value.kind, SaltElection::IncomeTax { .. }))
        {
            let issue = ValidationIssue::new("INVESTMENT_TAX_ABOVE_PAID");
            // The existing native owner alone decides validity. A chosen
            // income-tax election makes this its entered paid_part failure;
            // otherwise the incompatible Records selection is the actual input.
            let issue = if matches!(value.kind, SaltElection::IncomeTax { .. }) {
                issue.at_field("paid_part").in_field("investment_income_tax")
            } else {
                issue.at_field("investment_income_tax")
            };
            return Err(issue.in_field("value").into());
        }
        self.itemized_candidate(TaxSection::Taxes, |facts| {
            facts.personal_tax = personal_tax_record(&value.kind, Some(&facts.personal_tax));
            facts.investment_income_tax = value.investment_income_tax.clone();
            facts.income_tax_owners = value.income_tax_owners;
            facts.real_estate_taxes = value.real_estate;
            facts.personal_property_taxes = value.personal_property;
            Ok(())
        })
    }
    // Investment interest (Schedule A line 9, Form 4952) and investment
    // property taxes.
    // Unrecovered annuity costs (Schedule A line 17i, T3-deduct) are stored
    // on FederalDeductions, each naming its retirement income in the
    // catalog the page was built with.
    pub(super) fn other_itemized_candidate(
        &self,
        value: &OtherItemized,
    ) -> Result<FederalDraft, SaveFailure> {
        let other_lines: Vec<crate::OtherItemizedLine> =
            value.other_lines.iter().cloned().map(Into::into).collect();
        if other_lines.iter().any(|l| !l.valid() || matches!(l, crate::OtherItemizedLine::ImpairmentWork { owner, .. } if !self.allowed_owners().contains(&(*owner as i32)))) {
            return Err(status("MESSAGE_YOUR_SPOUSE_S_EXPENSES_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY").into());
        }
        if value.bond_instruments != crate::investment_statements::bond_instruments(&self.draft) {
            return Err(status("BOND_PREMIUM_CATALOG_CHANGED").into());
        }
        if let Some(issue) =
            crate::investment_statements::manual_bond_premium_issue(&self.draft, &other_lines)
        {
            return Err(issue.in_field("value").into());
        }
        let contracts = annuity_contracts(&self.draft);
        if value.contracts != contracts {
            return Err(crate::interview::status(
                "MESSAGE_THE_RETIREMENT_INCOME_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING",
            ).into());
        }
        let losses = &value.annuity_losses;
        let key = |key: &str| Err(libertas_formatted_text(key, &[]).into());
        if let Some((index, _)) = losses
            .iter()
            .enumerate()
            .find(|(i, l)| losses[..*i].iter().any(|p| p.contract == l.contract))
        {
            return Err(ValidationIssue::new("ANNUITY_CONTRACT_TWICE")
                .at_field("contract")
                .in_list("annuity_losses", index)
                .in_field("value")
                .into());
        }
        if let Some((index, _)) = losses.iter().enumerate().find(|(_, l)| l.recovered > l.cost) {
            return Err(ValidationIssue::new("ANNUITY_RECOVERED_ABOVE_COST")
                .at_field("recovered")
                .in_list("annuity_losses", index)
                .in_field("value")
                .into());
        }
        if let Some((index, _)) = losses
            .iter()
            .enumerate()
            .find(|(_, l)| i128::from(l.recovered) + i128::from(l.refunds) > i128::from(l.cost))
        {
            return Err(ValidationIssue::new("ANNUITY_REFUNDS_ABOVE_COST")
                .at_field("refunds")
                .in_list("annuity_losses", index)
                .in_field("value")
                .into());
        }
        let mut records = Vec::with_capacity(losses.len());
        for (index, loss) in losses.iter().enumerate() {
            let Some(contract) = contracts.iter().find(|contract| contract.id == loss.contract)
            else {
                return Err(ValidationIssue::new(
                    "MESSAGE_AN_ANNUITY_COST_NAMES_RETIREMENT_INCOME_THAT_IS_NOT_ON_THIS_RETURN_CHOOSE_IT",
                ).at_field("contract").in_list("annuity_losses", index).in_field("value").into());
            };
            records.push(crate::AnnuityLoss {
                income_id: contract.id,
                started: loss.started,
                cost: loss.cost,
                recovered: loss.recovered,
                refunds: loss.refunds,
                eligible: loss.eligible,
            });
        }
        if records.iter().any(|l| !l.valid()) {
            return Err(crate::interview::status(
                "MESSAGE_CHECK_EACH_ANNUITY_COST_ITS_STARTING_DATE_AFTER_JULY_1_1986_THROUGH_2026_AND",
            ).into());
        }
        if let Some(value) = &value.investment_interest
            && let Some(index) = value.loans.iter().position(|loan| {
                let allowed = crate::investment_interest::loan_origin_choices(
                    &self.draft,
                    loan.source,
                    loan.carry,
                    loan.amt_carry,
                );
                allowed.is_empty() && loan.originating_k1.is_some()
            })
        {
            return Err(ValidationIssue::new("INVESTMENT_BORROWING_RECORDS")
                .at_field("source")
                .in_field("originating_k1")
                .in_list("loans", index)
                .in_field("investment_interest")
                .in_field("value")
                .into());
        }
        let borrowing = value.investment_interest.as_ref().map(crate::InvestmentBorrowing::from);
        if let Some(borrowing) = borrowing.as_ref() {
            if let Some(issue) = borrowing.admission_issue() {
                return Err(issue.in_field("investment_interest").in_field("value").into());
            }
            if let Some((index, issue)) = borrowing.sources_issue(&self.draft) {
                return Err(issue
                    .map_field_prefix(&["originating_k1"], &["originating_k1", "source"])
                    .in_list("loans", index)
                    .in_field("investment_interest")
                    .in_field("value")
                    .into());
            }
        }
        let mut draft = self.draft.clone();
        if let Some(facts) = itemized_facts_mut(&mut draft) {
            facts.investment_property_taxes = value.investment_property_taxes;
            facts.investment_property_tax_owners = value.investment_property_tax_owners;
            facts.other_lines = value.other_lines.iter().cloned().map(Into::into).collect();
        } else if value.investment_property_taxes != 0
            || !value.other_lines.is_empty()
            || !records.is_empty()
        {
            return key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        let Some(deductions) = draft.deductions.as_mut() else {
            return key("PROBLEM_SECTION_UNAVAILABLE_YET");
        };
        deductions.investment_interest = borrowing;
        deductions.annuity_losses = records;
        finish_section(&mut draft, TaxSection::OtherItemized);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_deductions(
        &self,
        value: &Deductions,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != DEDUCTIONS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.deductions_candidate(value)
    }

    pub(super) fn save_medical(
        &self,
        value: &Medical,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ITEMIZED_PAGE || !self.reached(page) || !itemizing(&self.draft) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.medical_candidate(value)
    }

    pub(super) fn save_taxes(&self, value: &Taxes, page: i32) -> Result<FederalDraft, SaveFailure> {
        if page != ITEMIZED_PAGE || !self.reached(page) || !itemizing(&self.draft) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.expense_owner_candidate(self.taxes_candidate(value), TaxSection::Taxes, None)
    }

    pub(super) fn save_other_itemized(
        &self,
        value: &OtherItemized,
        page: i32,
        interest_sources: &[crate::InvestmentSourceChoice],
        investment_k1_sources: &[crate::RecordChoice],
        investment_origin_choices: &[i32],
    ) -> Result<FederalDraft, SaveFailure> {
        let d = &self.draft;
        if interest_sources != crate::investment_interest::sources(d)
            || investment_k1_sources != crate::investment_interest::k1_sources(d)
            || investment_origin_choices != crate::investment_interest::origin_choices(d)
        {
            return Err(status("INVESTMENT_BORROWING_RECORDS").into());
        }
        if page != if itemizing(d) { ITEMIZED_PAGE } else { DEDUCTIONS_PAGE }
            || !self.reached(page)
            || !walk::in_walk(d, TaxSection::OtherItemized)
        {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.expense_owner_candidate(
            self.other_itemized_candidate(value),
            TaxSection::OtherItemized,
            None,
        )
    }
    pub(super) fn after_other_itemized_save(&self, _page: i32) -> Reply {
        self.enter(if itemizing(&self.draft) {
            ITEMIZED_PAGE
        } else {
            self.after_deductions() as i32
        })
    }
}
