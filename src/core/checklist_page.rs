#[allow(unused_imports)]
use super::*;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Situations people often miss
// The editor distinguishes an explicit negative answer from selecting an empty
// positive branch. Only accepted answers are converted to the stored form.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OftenMissedAnswers {
    /// Income and taxes people often miss
    /// Which of these applied to you or your spouse in 2026, with no form to enter?
    #[libertas_bitflags(MissedIncomeTail)]
    pub income: u32,
    /// Situations this App doesn't prepare
    /// Select every situation that applied to you or your spouse. Each is named as not calculated.
    /// [NullText]
    /// None of these
    /// [SetText]
    /// These situations applied
    #[libertas_unordered]
    /// ----
    /// Situation
    pub unprepared: Option<Vec<Unprepared>>,
    /// Deductions and credits people often miss
    /// Optional: every other deduction and credit is under See all topics.
    #[libertas_bitflags(MissedBenefits)]
    pub benefits: u32,
}

// These former discovery choices are answered on the direct income page.
// Clearing their route flags leaves every actual income record unchanged.
fn missed_income_tail(bits: u32) -> u32 {
    bits & !(MissedIncome::JURY.0
        | MissedIncome::PRIZE.0
        | MissedIncome::HOBBY.0
        | MissedIncome::OTHER_INCOME.0
        | MissedIncome::CASH_TIPS.0)
}

impl From<OftenMissed> for OftenMissedAnswers {
    fn from(value: OftenMissed) -> Self {
        let income = missed_income_tail(value.income);
        Self {
            income,
            unprepared: (!value.unprepared.is_empty()).then_some(value.unprepared),
            benefits: value.benefits,
        }
    }
}

pub(super) fn catalog_bits(value: &Checklist, catalog: Catalog) -> u32 {
    match catalog {
        Catalog::Activities => value.activities,
        Catalog::Paid => value.paid,
        Catalog::Carryovers => value.carryovers,
    }
}
pub(super) fn catalog_bits_mut(value: &mut Checklist, catalog: Catalog) -> &mut u32 {
    match catalog {
        Catalog::Activities => &mut value.activities,
        Catalog::Paid => &mut value.paid,
        Catalog::Carryovers => &mut value.carryovers,
    }
}
pub(super) fn ticked(value: &Checklist, flag: &Flag) -> bool {
    catalog_bits(value, flag.catalog) & flag.bit != 0
}
pub(super) fn set_tick(value: &mut Checklist, flag: &Flag, on: bool) {
    let bits = catalog_bits_mut(value, flag.catalog);
    if on {
        *bits |= flag.bit;
    } else {
        *bits &= !flag.bit;
    }
}
// A box the questions do not declare.
pub(super) fn undeclared_ticks(value: &Checklist) -> bool {
    [Catalog::Activities, Catalog::Paid, Catalog::Carryovers].into_iter().any(|catalog| {
        let declared =
            FLAGS.iter().filter(|f| f.catalog == catalog).fold(0, |bits, f| bits | f.bit);
        catalog_bits(value, catalog) & !declared != 0
    })
}
// The boxes the documents check.
pub(super) fn evidenced(d: &FederalDraft) -> Checklist {
    let mut value = Checklist::NONE;
    for flag in FLAGS.iter().filter(|f| evidence(d, f).is_some()) {
        set_tick(&mut value, flag, true);
    }
    value
}
// The notes naming the document that checks each offered box.
pub(super) fn evidence_notes(d: &FederalDraft) -> Vec<u8> {
    let shown =
        crate::ChecklistAnswers::from(evidenced(d)).with_preserved_business(Checklist::NONE);
    let notes: Vec<crate::Message> = FLAGS
        .iter()
        .filter(|f| ticked(&shown, f))
        .filter_map(|f| {
            evidence(d, f).map(|form| {
                crate::Message::new(
                    "CHECKLIST_FROM_YOUR_DOCUMENT",
                    alloc::vec![
                        crate::MessageArg::Nested(alloc::boxed::Box::new(crate::Message::new(
                            f.label_key,
                            alloc::vec![]
                        ))),
                        crate::MessageArg::Text(form.into()),
                    ],
                )
            })
        })
        .collect();
    if notes.is_empty() { Vec::new() } else { message_list(&notes).encode() }
}
// The person's own ticks and every topic with entries. A box a document
// checks is never pre-ticked on the page (its note names the document, and
// the document keeps the topic on the path): the person ticks what they paid.
pub(super) fn answered_ticks(d: &FederalDraft) -> Checklist {
    let mut value = d.checklist;
    for flag in FLAGS.iter().filter(|f| (f.entered)(d)) {
        set_tick(&mut value, flag, true);
    }
    value
}
// The boxes that plan the path: the answered ticks and every box a document
// checks (a save keeps the person's ticks, follow_ticks).
pub(super) fn checklist_value(d: &FederalDraft) -> Checklist {
    let mut value = d.checklist;
    for flag in FLAGS.iter().filter(|f| (f.entered)(d) || evidence(d, f).is_some()) {
        set_tick(&mut value, flag, true);
    }
    value
}
// A box follows its entries: a page that adds the first entry of a box
// checks it, one that removes its last entry unchecks it, and every page but
// this one keeps each box with entries checked. Only this page unchecks a box
// with entries, which the prune then removes. The document center supplies
// its own boxes from the documents it holds.
pub(super) fn follow_ticks(
    before: &FederalDraft,
    d: &mut FederalDraft,
    source: Option<TaxSection>,
) {
    if let Some(deductions) = &mut d.deductions {
        deductions.vehicle_recoveries.retain(|row| {
            deductions.vehicle_loans.iter().any(|loan| loan.id == row.id && loan.model().refund > 0)
        });
    }
    crate::information_returns::follow_work_sources(before, d);
    crate::farm_insurance_paper::follow_sources(before, d);
    crate::iso_return::carry_lot_answers(before, d);
    crate::nua_sale::carry_lot_answers(before, d);
    crate::espp_return::reconcile(d);
    crate::iso_return::reconcile(d);
    crate::nua_sale::reconcile(d);
    crate::social_documents::follow_printed_lump_sums(before, d);
    let trump_accounts = crate::trump_account::project_accounts(d);
    if let Some(screening) = d.screening.as_mut() {
        screening.trump_accounts = trump_accounts;
    }
    let checklist_saved = d.checklist != before.checklist;
    let entered = |d: &FederalDraft, flag: &Flag| {
        FLAGS.iter().any(|f| f.catalog == flag.catalog && f.bit == flag.bit && (f.entered)(d))
    };
    for flag in &FLAGS {
        let now = entered(d, flag);
        if now != entered(before, flag) {
            set_tick(&mut d.checklist, flag, now);
        } else if now && !checklist_saved && source != Some(TaxSection::Documents) {
            set_tick(&mut d.checklist, flag, true);
        }
    }
}
// A return whose boxes show each topic with entries checked, as every save
// leaves it (fixtures built by hand).
#[cfg(test)]
pub(crate) fn with_ticks(mut d: FederalDraft) -> FederalDraft {
    let entered: Vec<&Flag> = FLAGS.iter().filter(|f| (f.entered)(&d)).collect();
    for flag in entered {
        set_tick(&mut d.checklist, flag, true);
    }
    // Form 8379's answers are given only with the spouse page's item.
    if d.injured_spouse.is_some()
        && let Some(spouse) = d.people.as_mut().and_then(|p| p.spouse.as_mut())
    {
        spouse.whole_return |= SpouseSituations::INJURED_SPOUSE.0;
    }
    d
}
// The planned ticks as a checklist value.
#[cfg(test)]
pub(crate) fn planned(d: &FederalDraft) -> Checklist {
    let mut value = Checklist::NONE;
    for flag in planned_flags(d) {
        set_tick(&mut value, flag, true);
    }
    value
}
// The planned ticks of this return, in the catalogs' order: ticked with
// nothing entered yet.
pub(super) fn planned_flags(d: &FederalDraft) -> impl Iterator<Item = &'static Flag> + '_ {
    FLAGS.iter().filter(move |f| ticked(&d.checklist, f) && !(f.entered)(d))
}
// The page that fixes a planned tick: an itemized topic while the route does
// not itemize is answered on the deductions page.
pub(super) fn planned_page(d: &FederalDraft, section: TaxSection) -> TaxSection {
    if ITEMIZED_TOPICS.contains(&section) && section != TaxSection::Mortgage && !itemizing(d) {
        TaxSection::Deductions
    } else {
        section
    }
}
// The planned topics of one hub page, for its walk and suggestion, and the
// topics a plan reminds of (walk-owned data, "Plans are not completeness"):
// a tick naming one kind of entry with none entered reads "Planned" on its
// row; a plan is never an item needing review. Every tick names one kind of
// entry since the less-common pages became walk steps (gate fixes batch A).
pub(crate) fn planned_sections(d: &FederalDraft) -> Vec<TaxSection> {
    planned_flags(d)
        .map(|f| planned_page(d, f.section))
        .chain(without_form_pages(d))
        .chain(missed_pages(d, &d.often_missed))
        .chain(missed_pages(
            d,
            &OftenMissed { income: missed_evidence(d), ..OftenMissed::default() },
        ))
        .collect()
}
// The direct page stores routes only for facts owned by another topic. Its
// other-income amount and description are already complete on this page.
pub(super) fn without_form_pages(d: &FederalDraft) -> impl Iterator<Item = TaxSection> + '_ {
    let value = d.without_form.as_ref();
    [
        value.is_some_and(|v| v.self_work).then_some(TaxSection::SelfEmployment),
        value.is_some_and(|v| v.tips).then_some(TaxSection::Wages),
    ]
    .into_iter()
    .flatten()
}
pub(super) fn without_form_summary(d: &FederalDraft) -> Vec<u8> {
    let Some(value) = &d.without_form else {
        return status("REVIEW_NOT_ANSWERED");
    };
    let mut messages = Vec::new();
    if value.self_work {
        messages.push(Message::new("WITHOUT_FORM_SELF_WORK", alloc::vec![]));
    }
    if value.tips {
        messages.push(Message::new("WITHOUT_FORM_TIPS", alloc::vec![]));
    }
    if let Some(other) = &value.other {
        messages.push(Message::new(
            "WITHOUT_FORM_OTHER",
            alloc::vec![MessageArg::usd_cents(other.amount)],
        ));
    }
    if messages.is_empty() {
        status("REVIEW_NONE_OF_THESE")
    } else {
        message_list(&messages).encode()
    }
}
// The pages the answers of "Situations people often miss" bring onto the
// path, each option naming the topic that holds its record kind or form. The
// option only plans the page: a topic holding entries is kept by its own
// tick, so unchecking withdraws the plan and removes nothing.
pub(crate) const MISSED_INCOME: [(MissedIncome, TaxSection); 15] = [
    (MissedIncome::GAMBLING, TaxSection::Gambling),
    (MissedIncome::CANCELED_DEBT, TaxSection::CanceledDebt),
    (MissedIncome::PROPERTY_SOLD, TaxSection::InvestmentSales),
    (MissedIncome::INSTALLMENT, TaxSection::InvestmentSales),
    (MissedIncome::SCHOLARSHIP, TaxSection::OtherIncome),
    (MissedIncome::ALIMONY, TaxSection::OtherIncome),
    (MissedIncome::CASUALTY, TaxSection::OtherIncome),
    (MissedIncome::DISASTER_DISTRIBUTION, TaxSection::Retirement),
    (MissedIncome::MISSED_RMD, TaxSection::Retirement),
    (MissedIncome::ACCOUNT_TAXES, TaxSection::OtherTaxes),
    (MissedIncome::RECAPTURE, TaxSection::OtherTaxes),
    (MissedIncome::RELATED_EXCHANGE, TaxSection::InvestmentSales),
    (MissedIncome::MARKETPLACE, TaxSection::Marketplace),
    // A credit the IRS reduced or disallowed on an earlier return cannot be
    // claimed again until Form 8862 is filed (IRC 24(g), 25A(b)(4) and 32(k);
    // Form 8862 instructions). Leaving the answer out while one of these
    // credits is claimed would overstate the refund, so it is asked here and
    // answered on the Form 8862 facts of More credits.
    (MissedIncome::DISALLOWED_CREDIT, TaxSection::MoreCredits),
    (MissedIncome::PRIVATE_LOAN_INTEREST, TaxSection::InterestDividends),
];
pub(crate) const MISSED_BENEFITS: [(MissedBenefits, TaxSection); 11] = [
    (MissedBenefits::FOREIGN_TAX, TaxSection::MoreCredits),
    (MissedBenefits::EDUCATOR, TaxSection::Educator),
    (MissedBenefits::MEDICAL, TaxSection::Medical),
    (MissedBenefits::ALIMONY_PAID, TaxSection::MoreAdjustments),
    (MissedBenefits::INVESTMENT_INTEREST, TaxSection::OtherItemized),
    (MissedBenefits::WORTHLESS, TaxSection::InvestmentSales),
    (MissedBenefits::EMPLOYEE_EXPENSES, TaxSection::EmployeeExpenses),
    (MissedBenefits::ENERGY_CREDIT, TaxSection::MoreCredits),
    (MissedBenefits::ADOPTION, TaxSection::MoreCredits),
    (MissedBenefits::CHILD_INCOME, TaxSection::InterestDividends),
    (MissedBenefits::EMPLOYEE_PAY, TaxSection::Wages),
];
// The page of each checked option; a casualty also plans the deductions page
// that holds the personal casualty events.
pub(super) fn missed_pages<'a>(
    d: &'a FederalDraft,
    answer: &'a OftenMissed,
) -> impl Iterator<Item = TaxSection> + 'a {
    let income = MISSED_INCOME.iter().filter(|(bit, _)| answer.income & bit.0 != 0);
    let benefits = MISSED_BENEFITS.iter().filter(|(bit, _)| answer.benefits & bit.0 != 0);
    income
        .map(|(_, topic)| *topic)
        .chain(benefits.map(|(_, topic)| *topic))
        .chain((answer.income & MissedIncome::CASUALTY.0 != 0).then_some(TaxSection::Deductions))
        .map(|topic| planned_page(d, topic))
}
// Document evidence plans needed topics without becoming a person's answer.
pub(super) fn missed_evidence(d: &FederalDraft) -> u32 {
    let marketplace =
        d.credits.as_ref().is_some_and(|c| matches!(c.marketplace, MarketplaceClaim::Yes { .. }));
    if marketplace { MissedIncome::MARKETPLACE.0 } else { 0 }
}
// A hub's first row: what the checklist says about its topics.
pub(super) fn told_us(d: &FederalDraft, hub: TaxSection) -> ReviewItem {
    let value = checklist_value(d);
    let mut keys: Vec<&str> = FLAGS
        .iter()
        .filter(|f| hub_of(f.section) == Some(hub) && ticked(&value, f))
        .map(|f| f.label_key)
        .collect();
    keys.dedup();
    let labels: Vec<crate::Message> =
        keys.into_iter().map(|key| crate::Message::new(key, alloc::vec![])).collect();
    ReviewItem {
        field: None,
        item: status("REVIEW_TOLD_US"),
        status: if labels.is_empty() {
            status("REVIEW_NONE_OF_THESE")
        } else {
            message_list(&labels).encode()
        },
        section: TaxSection::Checklist,
    }
}
// The path the checklist plans (M3-P18): the sections shown, then the
// review. "Your return: Income, Deductions, Federal payments, Review".
pub(super) fn checklist_path(d: &FederalDraft, result: &FederalResult) -> Vec<u8> {
    let items = remaining_items(d, result, TaxSection::Checklist);
    let mut path: Vec<crate::Message> = sections_with_needs(d, &items)
        .into_iter()
        .filter(|s| *s != TaxSection::Itemized)
        .map(section_message)
        .collect();
    path.push(section_message(TaxSection::Review));
    crate::Message::new(
        "SUMMARY_RETURN_PATH",
        alloc::vec![crate::MessageArg::Nested(alloc::boxed::Box::new(message_list(&path))),],
    )
    .encode()
}

impl Interview {
    // The walk opens it at once after the Document center: unanswered by
    // WorkflowRequest until saved (its two questions are facts the rules
    // test, so neither has a Default), then by EditRequest with the answers.
    pub(super) fn income_without_form_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let estimate = self.estimate_now(TaxSection::IncomeWithoutForm);
        let navigation = self.navigation_availability(&self.allowed_sections());
        let progress = self.progress(INCOME_WITHOUT_FORM_PAGE);
        let (page, previous) = (INCOME_WITHOUT_FORM_PAGE, DOCUMENTS_PAGE);
        let suggested_destination = TaxSection::Checklist.into();
        match &d.without_form {
            Some(value) => Reply::Edit(P::SaveIncomeWithoutForm {
                user: self.user,
                cookie: d.cookie.clone(),
                revision: d.revision,
                page,
                previous,
                review,
                estimate,
                warnings: Vec::new(),
                progress,
                value: value.clone(),
                navigation,
                suggested_destination,
            }),
            None => Reply::Response(P::BeginIncomeWithoutForm {
                cookie: d.cookie.clone(),
                revision: d.revision,
                page,
                previous,
                review,
                estimate: estimate.map(Into::into),
                progress,
                navigation,
                suggested_destination,
            }),
        }
    }
    pub(super) fn income_without_form_candidate(
        &self,
        value: &crate::IncomeWithoutFormAnswers,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut draft = self.draft.clone();
        draft.without_form = Some(value.clone());
        let previous: Vec<_> = without_form_pages(&self.draft).collect();
        let reopened: Vec<_> =
            without_form_pages(&draft).filter(|topic| !previous.contains(topic)).collect();
        draft.finished_sections.retain(|s| {
            !reopened.iter().any(|topic| s == topic || Some(*s) == walk::topic_hub(*topic))
        });
        finish_section(&mut draft, TaxSection::IncomeWithoutForm);
        Ok(draft)
    }
    // The opening checklist (M3-P18), opened with the person's ticks and
    // each topic with entries; the documents' boxes are only named in the
    // notes. `previous` is the last people page, or the hub whose "What you
    // told us" row opened it.
    // Until it is first saved the page asks its three questions unanswered
    // (BeginChecklist, a WorkflowRequest); afterwards it reopens with the
    // answers by EditRequest.
    pub(super) fn checklist_page(&self, previous: i32, review: bool) -> Reply {
        let d = &self.draft;
        let running = self.estimate_now(TaxSection::Checklist);
        let progress = checklist_path(d, &estimate(d));
        if !d.finished_sections.contains(&TaxSection::Checklist) {
            return Reply::Response(P::BeginChecklist {
                cookie: d.cookie.clone(),
                revision: d.revision,
                page: CHECKLIST_PAGE,
                previous,
                review,
                estimate: running.map(Into::into),
                progress,
                from_documents: evidence_notes(d),
            });
        }
        Reply::Edit(P::SaveChecklist {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: CHECKLIST_PAGE,
            previous,
            review,
            estimate: running,
            warnings: Vec::new(),
            progress,
            from_documents: evidence_notes(d),
            value: answered_ticks(d).into(),
        })
    }
    // The checklist's answers merged into the accepted return. Unticking a
    // topic with entries says it does not apply: the save's prune removes
    // the entries, which the running estimate names first.
    pub(super) fn checklist_candidate(
        &self,
        value: &crate::ChecklistAnswers,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut accepted = self.draft.checklist;
        // This editor cannot remove a business: its ownership bit follows
        // actual records even when an earlier answer did not set the bit.
        for flag in FLAGS.iter().filter(|flag| {
            matches!(flag.catalog, Catalog::Activities)
                && flag.bit == ActivityTopics::BUSINESS.0
                && (flag.entered)(&self.draft)
        }) {
            set_tick(&mut accepted, flag, true);
        }
        let mut value = value.with_preserved_business(accepted);
        // The first answer cannot drop a topic that already has entries (the
        // page asked it unanswered): its tick follows the entries.
        if !self.draft.finished_sections.contains(&TaxSection::Checklist) {
            for flag in FLAGS.iter().filter(|flag| (flag.entered)(&self.draft)) {
                set_tick(&mut value, flag, true);
            }
        }
        if undeclared_ticks(&value) {
            return Err(crate::interview::status("MESSAGE_CHOOSE_ONLY_THE_LISTED_ITEMS").into());
        }
        // The person's ticks are saved as given: the page never pre-ticks a
        // box a document checks, so a tick there is the person's answer. The
        // canonical business flag is preserved because this editor does not
        // ask for it.
        let mut draft = self.draft.clone();
        draft.checklist = value;
        // A newly set plan is walked like a first-walk plan: its topic and
        // the section that holds it lose their marks, so the walk opens
        // them again. An existing plan never re-enters its topic.
        let reopened: Vec<(TaxSection, TaxSection)> = FLAGS
            .iter()
            .filter(|f| {
                ticked(&draft.checklist, f)
                    && !ticked(&self.draft.checklist, f)
                    && !(f.entered)(&draft)
            })
            .map(|flag| {
                let topic = planned_page(&draft, flag.section);
                (topic, walk::topic_hub(topic).unwrap_or(topic))
            })
            .collect();
        draft
            .finished_sections
            .retain(|s| !reopened.iter().any(|(topic, section)| s == topic || s == section));
        finish_section(&mut draft, TaxSection::Checklist);
        let (accepted, _) = self.reconcile_values(draft.clone());
        if let Some(key) = health_correction_reference_rule(&accepted)
            .or_else(|| crate::care_benefit_return::reference_rule(&accepted))
            .or_else(|| crate::qbi_aggregation::reference_rule(&accepted))
        {
            return Err(libertas_formatted_text(key, &[]).into());
        }
        Ok(draft)
    }
    // Required choices start unanswered; reopening carries only accepted
    // answers. Document evidence plans topics independently of those choices.
    pub(super) fn often_missed_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let navigation = self.navigation_availability(&self.allowed_sections());
        let all_topics =
            crate::protocol::AllTopicsLink { label: status("OFTEN_MISSED_ALL_TOPICS") };
        if !d.finished_sections.contains(&TaxSection::OftenMissed) {
            return Reply::Response(P::BeginOftenMissed {
                cookie: d.cookie.clone(),
                revision: d.revision,
                page: OFTEN_MISSED_PAGE,
                previous: self.previous_shown(TaxSection::OftenMissed),
                review,
                estimate: self.estimate_now(TaxSection::OftenMissed).map(Into::into),
                warnings: Vec::new(),
                progress: self.progress(OFTEN_MISSED_PAGE),
                all_topics,
                navigation,
                suggested_destination: TaxSection::Review.into(),
            });
        }
        Reply::Edit(P::SaveOftenMissed {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: OFTEN_MISSED_PAGE,
            previous: self.previous_shown(TaxSection::OftenMissed),
            review,
            estimate: self.estimate_now(TaxSection::OftenMissed),
            warnings: Vec::new(),
            progress: self.progress(OFTEN_MISSED_PAGE),
            value: d.often_missed.clone().into(),
            all_topics,
            navigation,
            suggested_destination: TaxSection::Review.into(),
        })
    }
    // The answers merged into the accepted return. The page is required: its
    // save marks it, and explicit null means "None of these". A newly checked
    // option walks its page like a first-walk plan, so the topic and its hub
    // lose their marks; an existing plan never re-enters its topic.
    pub(super) fn often_missed_candidate(
        &self,
        answers: &crate::federal_model::OftenMissedAnswers,
    ) -> Result<FederalDraft, SaveFailure> {
        for (empty, field) in
            [(answers.unprepared.as_ref().is_some_and(Vec::is_empty), "unprepared")]
        {
            if empty {
                return Err(ValidationIssue::new("MESSAGE_SELECT_ITEMS_OR_NONE")
                    .at_field(field)
                    .in_field("value")
                    .into());
            }
        }
        let value = OftenMissed {
            income: missed_income_tail(answers.income),
            unprepared: answers.unprepared.clone().unwrap_or_default(),
            benefits: answers.benefits,
        };
        if value.income & !MISSED_INCOME.iter().fold(0, |all, (bit, _)| all | bit.0) != 0
            || value.benefits & !MISSED_BENEFITS.iter().fold(0, |all, (bit, _)| all | bit.0) != 0
        {
            return Err(crate::interview::status("MESSAGE_CHOOSE_ONLY_THE_LISTED_ITEMS").into());
        }
        if let Some(index) = value
            .unprepared
            .iter()
            .enumerate()
            .position(|(i, item)| value.unprepared[..i].contains(item))
        {
            return Err(ValidationIssue::new("MESSAGE_CHOOSE_EACH_SITUATION_ONLY_ONCE")
                .at_record()
                .in_list("unprepared", index)
                .in_field("value")
                .into());
        }
        let mut draft = self.draft.clone();
        draft.often_missed = OftenMissed {
            income: value.income,
            unprepared: value.unprepared.clone(),
            benefits: value.benefits,
        };
        let new_bits = OftenMissed {
            income: draft.often_missed.income & !self.draft.often_missed.income,
            unprepared: Vec::new(),
            benefits: draft.often_missed.benefits & !self.draft.often_missed.benefits,
        };
        let reopened: Vec<TaxSection> = missed_pages(&draft, &new_bits).collect();
        draft.finished_sections.retain(|s| {
            !reopened.iter().any(|topic| s == topic || Some(*s) == walk::topic_hub(*topic))
        });
        finish_section(&mut draft, TaxSection::OftenMissed);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_often_missed(
        &self,
        value: &crate::federal_model::OftenMissedAnswers,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != OFTEN_MISSED_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.often_missed_candidate(value)
    }
}
