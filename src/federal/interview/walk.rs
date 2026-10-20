// Coverage: walk-owned data (2026-09-28).
//   The walk is a function of the accepted answers: in_walk says which pages this return can
//   open (navigation, the hubs' topics, first_open and the prune read it), first_open opens the
//   first page in the walk whose required data is absent. Every stored datum is owned by the
//   page that asks it, and prune goes down one table at two levels. Level 1 erases, destructuring
//   FederalDraft exhaustively: each datum of a page outside the walk, each row whose constrained
//   choice is outside its allowed list recomputed from the candidate (the allowed_* functions the
//   pages use), each conditional answer its page no longer shows (every spouse answer off a joint
//   return), and each topic's entries whose checklist tick is off (FLAGS.clear). Level 2 keeps the
//   data: a page in the walk whose own save refuses its stored answers against the candidate
//   (page_refuses: the page's builder and its *_candidate, no second copy of its rules) loses its
//   finished mark, with its hub's for a topic, so the walk opens it again prefilled.
//   FederalPayments.joint is the one context field, where no page rule can see the change:
//   payments entered for a joint return are the couple's (erased off a joint return), one filer's
//   stay valid on a joint return (reopened to add the spouse's). Every save prunes the candidate
//   before it stores it; the running estimate names what the prune removes and reopens
//   (warning_text). Tests: src/tests/cross/change_audit.rs (the audit's cases and the two generated
//   properties saves_keep_only_walked_data and unasked_data_leaves_the_return_unchanged).
//   Unit u5-adjust (2026-09-28): the unasked adjustments lines have owners: alimony_paid (More
//   adjustments), social_election and required_distributions (the Retirement topic's kinds),
//   public_safety (each 1099-R; rows whose 1099-R is gone are erased). archer,
//   pension_contributions has no page (refused by name) and is erased. Unit B6a (2026-10-09):
//   archer is the HSA page's (always in the walk) and able_contributions the Other income topic's
//   kind, by owner. Tests: src/tests/federal/unasked_pages.rs leaving_joint_erases_the_spouses_rows_on_the_new_pages,
//   answers_no_page_asks_are_erased; the generated households carry the new pages' data.
//   u5-losses (2026-09-28): FederalPeople.parent_tax is owned by the parent's return page (in the
//   walk while Form 8615 applies, parent_tax_in_walk); screening.nol/atnol by the Self-employment
//   and Farming topics (their operating-loss kind).
//   u6-alimony (2026-09-28): alimony received is an income entry (FederalIncome::Alimony) of the
//   prizes, royalties, equipment rent and alimony topic, owned by that topic's tick and by its
//   owner inside allowed_owners (the income line below); no new line or context field.
//   u6-roth (2026-09-28): adjustments.roth_history is owned by the Retirement topic's "Roth IRA
//   history" kind, row by row through allowed_owners, and by the topic's tick; the Roth IRA
//   1099-R (codes J, Q, T) is a Retirement topic entry like any 1099-R.
//   u6-home (2026-09-28): home sales (FederalIncome::HomeSale) are the Investment sales topic's
//   (topic_of; its tick owns them); a sale's spouse answers are erased off a joint return
//   (HomeSale::drop_spouse), and a joint return without them reopens Investment sales (level 2,
//   HOME_SPOUSE_NEEDED). Form 8828's hidden home sale IDs are derived on every save.
//   Unit B1 Form 8815 (2026-10-09): the savings bond record (FederalIncome::SavingsBonds) is the
//   Interest and dividends topic's, owned by its tick and by its owner inside allowed_owners; each
//   student row stays only for a person still on the return (Allowed.students), and a record left
//   with no student goes.
//   Unit B3 Form 1099-Q (2026-10-09): the Form 1099-Q (FederalIncome::EducationDistribution) and the
//   529 and Coverdell expenses record (FederalIncome::EducationAccountExpenses) are the Other income
//   topic's, owned by its tick and by its owner inside allowed_owners; a form whose chosen student
//   left the return becomes a student not on the return with no costs assigned, an expenses row
//   stays only for a person still on the return, and a record left with no student goes.
use super::*;
use crate::protocol::SaveWarning;
use alloc::vec;

// Whether a page is in this return's walk: a save can reach it for these
// answers, so it asks and keeps its data. Every other page is always in the
// walk (the income, adjustments and credits topics, the less-common pages,
// the payments and less-common pages).
pub(super) fn in_walk(d: &FederalDraft, page: TaxSection) -> bool {
    match page {
        TaxSection::Spouse => joint(d),
        TaxSection::SelfEmployedBenefits => has_benefit_rows(d),
        TaxSection::Mortgage => mortgage_available(d),
        TaxSection::Itemized | TaxSection::MoreItemized => itemizing(d),
        TaxSection::OtherItemized => {
            itemizing(d)
                || crate::investment_interest::sources(d).len() > 1
                || d.deductions.as_ref().is_some_and(|v| v.investment_interest.is_some())
        }
        topic if ITEMIZED_TOPICS.contains(&topic) => itemizing(d),
        TaxSection::Saver => saver_offered(d),
        TaxSection::IncomeByPeriod => box_c(d),
        TaxSection::InjuredSpouse => injured_chosen(d),
        TaxSection::ParentTax => parent_tax_in_walk(d),
        _ => true,
    }
}

// The resource identity of one answer or row removed by pruning. Row names
// remain literal taxpayer data, except for schema-authored situation/owner names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Removed {
    Answer(&'static str),
    Row { kind: &'static str, name: Option<MessageArg> },
}
#[cfg(test)]
impl Removed {
    pub(crate) fn key(&self) -> &'static str {
        match self {
            Self::Answer(key) => key,
            Self::Row { kind, .. } => kind,
        }
    }
}
fn answer(key: &'static str) -> Removed {
    Removed::Answer(key)
}
// A plan's long-term-care premiums keep only the people still on the return;
// a plan left with nothing is no plan. True when something was removed.
fn prune_care_premiums(plan: &mut crate::BusinessHealthInsurance, people: &[i64]) -> bool {
    let crate::BusinessHealthInsurance::Plan { months, long_term_care, .. } = plan else {
        return false;
    };
    let before = long_term_care.len();
    long_term_care.retain(|p| people.contains(&p.person));
    if long_term_care.len() == before {
        return false;
    }
    if months.is_empty() && long_term_care.is_empty() {
        *plan = crate::BusinessHealthInsurance::None;
    }
    true
}
fn row(kind: &'static str, name: &str) -> Removed {
    Removed::Row { kind, name: (!name.is_empty()).then(|| MessageArg::Text(name.into())) }
}
// An income entry's row: what the person typed, else its display name (the
// capitalized kind over its identity; a child's election by its child).
fn entry_row(
    kind: &'static str,
    entry: &FederalIncomeEntry,
    dependents: &[FederalDependent],
) -> Removed {
    Removed::Row {
        kind,
        name: Some(if entry.label.trim().is_empty() {
            MessageArg::Nested(alloc::boxed::Box::new(crate::interview::income_display_label(
                entry, dependents,
            )))
        } else {
            MessageArg::Text(entry.label.clone())
        }),
    }
}
fn row_resource(kind: &'static str, name_key: &'static str) -> Removed {
    Removed::Row { kind, name: Some(MessageArg::Resource(name_key.into())) }
}

// What a prune did: the data it erased and the pages it opens again.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Pruned {
    pub(crate) removed: Vec<Removed>,
    pub(crate) reopened: Vec<TaxSection>,
}
// Makes absent everything the draft's own walk and allowed lists do not
// ask (level 1), then opens again each finished page whose own save refuses
// what it holds (level 2). A save runs the derived values between the two
// (Interview::reconcile). prune(prune(d)) == prune(d).
#[cfg(test)]
pub(crate) fn prune(d: &mut FederalDraft) -> Pruned {
    let removed = prune_data(d);
    let reopened = reopen_refused(d);
    Pruned { removed, reopened }
}
// Level 1. Each line reads the answers as the pass begins; a removal can
// take a page out of the walk (the spouse's answers with the spouse, the
// itemized topics with the route), so the table is applied until nothing
// changes.
pub(super) fn prune_data(d: &mut FederalDraft) -> Vec<Removed> {
    let mut removed = Vec::new();
    // Every pass only removes, so a handful of passes reaches the end.
    for _ in 0..16 {
        let before = d.clone();
        prune_unticked(d, &mut removed);
        prune_lines(d, &mut removed);
        if *d == before {
            break;
        }
    }
    removed
}
// One item per answer or reopened page; grouped rows retain a typed count and
// every available identifying name without assembling an English list.
pub(super) fn save_warnings(pruned: &Pruned) -> Vec<SaveWarning> {
    let mut messages = Vec::new();
    let mut seen_rows = Vec::new();
    for removal in &pruned.removed {
        match removal {
            Removed::Answer(key) => messages.push(Message::new(*key, vec![])),
            Removed::Row { kind, .. } if !seen_rows.contains(kind) => {
                seen_rows.push(*kind);
                let rows: Vec<_> = pruned
                    .removed
                    .iter()
                    .filter_map(|item| match item {
                        Removed::Row { kind: other, name } if other == kind => Some(name),
                        _ => None,
                    })
                    .collect();
                let one = alloc::format!("{kind}_ONE");
                let many = alloc::format!("{kind}_MANY");
                if let [Some(name)] = rows.as_slice() {
                    messages.push(Message::new(
                        "SAVING_REMOVES_ONE_NAMED",
                        vec![MessageArg::Count(1), MessageArg::Resource(one), (*name).clone()],
                    ));
                } else {
                    messages.push(Message::new(
                        "SAVING_REMOVES_COUNT",
                        vec![
                            MessageArg::Count(rows.len() as i64),
                            MessageArg::Resource(one.clone()),
                            MessageArg::Resource(many),
                        ],
                    ));
                    for name in rows.into_iter().flatten() {
                        messages.push(Message::new(
                            "SAVING_REMOVES_NAMED",
                            vec![MessageArg::Resource(one.clone()), name.clone()],
                        ));
                    }
                }
            }
            Removed::Row { .. } => {}
        }
    }
    for section in &pruned.reopened {
        messages.push(Message::new("SAVING_REOPENS", vec![MessageArg::Number(*section as i64)]));
    }
    messages.into_iter().map(|message| SaveWarning { message: message.encode() }).collect()
}

// Level 2: the finished pages whose own save refuses their stored answers
// against these answers lose their mark, a topic's hub with it, and the
// walk opens them again with those answers (Interview::enter).
pub(super) fn reopen_refused(d: &mut FederalDraft) -> Vec<TaxSection> {
    let mut reopened = Vec::new();
    let viewing = Interview::viewing(d.clone());
    for page in SEQUENCE {
        let hub = topic_hub(page);
        let finished = |s: TaxSection| d.finished_sections.contains(&s);
        let checked = in_walk(d, page)
            && (finished(page) || hub.is_some_and(|h| finished(h) && topic_has_data(d, page)));
        if !checked || !page_refuses(&viewing, page) {
            continue;
        }
        d.finished_sections.retain(|s| *s != page && Some(*s) != hub);
        reopened.push(page);
    }
    // The context no page rule sees: one filer's payments on a joint return
    // open the payments page again, with the payments kept.
    if d.payments.as_ref().is_some_and(|p| !p.joint && joint(d) && !p.nothing_entered())
        && d.finished_sections.contains(&TaxSection::Payments)
    {
        d.finished_sections.retain(|s| *s != TaxSection::Payments);
        reopened.push(TaxSection::Payments);
    }
    reopened
}
// The hub whose walk a topic page belongs to.
pub(super) fn topic_hub(topic: TaxSection) -> Option<TaxSection> {
    [TaxSection::Income, TaxSection::Adjustments, TaxSection::Itemized, TaxSection::Credits]
        .into_iter()
        .find(|hub| hub_topics(*hub).contains(&topic))
}
// Whether a topic holds anything its page would show.
pub(super) fn topic_has_data(d: &FederalDraft, topic: TaxSection) -> bool {
    FLAGS.iter().any(|f| f.section == topic && (f.entered)(d))
        || (topic == TaxSection::Saver && d.credits.as_ref().is_some_and(|c| c.saver.is_some()))
}
// Whether a page's own save refuses the answers it opens with: the page is
// built as the review opens it and its request is checked by the same
// candidate function its Save runs. A page that cannot show its stored
// answers is left to its own save.
pub(super) fn page_refuses(s: &Interview, page: TaxSection) -> bool {
    let reply = match page {
        topic if INCOME_TOPICS.contains(&topic) => s.topic_page(topic, true),
        TaxSection::Care => s.care_page(true),
        TaxSection::Hsa => s.hsa_page(true),
        TaxSection::SelfEmployedBenefits => s.self_employed_benefits_page(true),
        TaxSection::IraContributions => s.ira_contributions_page(true),
        TaxSection::Educator => s.educator_page(true),
        TaxSection::StudentLoan => s.student_loan_page(true),
        TaxSection::EmployeeExpenses => s.employee_expenses_page(true),
        TaxSection::Deductions => s.deductions_page(true),
        TaxSection::Medical => s.medical_page(true),
        TaxSection::Taxes => s.taxes_page(true),
        TaxSection::Mortgage => s.mortgage_page(true),
        TaxSection::HomeOffices => s.home_offices_page(true),
        TaxSection::GiftCarryovers => s.gift_carryovers_page(true),
        TaxSection::OtherItemized => s.other_itemized_page(true),
        TaxSection::Education => s.education_page(true),
        TaxSection::Saver => s.saver_page(true),
        TaxSection::Marketplace => s.marketplace_page(true),
        TaxSection::MoreCredits => s.more_credits_page(true),
        TaxSection::OtherTaxes => s.other_taxes_page(true),
        TaxSection::MoreAdjustments => s.more_adjustments_page(true),
        TaxSection::Payments => s.payments_page(true),
        TaxSection::MoreIncome | TaxSection::MoreItemized => s.situations_page(page, true),
        _ => return false,
    };
    match reply {
        Reply::Response(P::BeginPayments { .. } | P::BeginMoreCredits { .. }) => true,
        Reply::Edit(request) => request_refuses(s, request),
        _ => false,
    }
}

// A candidate returns a complete draft even when only its admission status
// is needed. Discard it in a separate frame instead of reserving one large
// return slot for every request variant in request_refuses.
#[inline(never)]
fn candidate_refuses<T, E>(candidate: impl FnOnce() -> Result<T, E>) -> bool {
    candidate().is_err()
}

fn request_refuses(s: &Interview, request: P) -> bool {
    if let P::SaveIncomes { topic, documents, .. } = request {
        return candidate_refuses(|| s.documents_candidate(topic, documents));
    }
    match &request {
        P::SaveCare { value, .. } => candidate_refuses(|| s.care_candidate(value)),
        P::SaveHsa { value, .. } => candidate_refuses(|| s.hsa_candidate(value)),
        P::SaveSelfEmployedBenefits { value, .. } => {
            candidate_refuses(|| s.self_employed_benefits_candidate(value))
        }
        P::SaveIraContributions { value, .. } => {
            candidate_refuses(|| s.ira_contributions_candidate(value))
        }
        P::SaveEducator { value, .. } => candidate_refuses(|| s.educator_candidate(value)),
        P::SaveStudentLoan { value, .. } => candidate_refuses(|| s.student_loan_candidate(value)),
        P::SaveEmployeeExpenses { value, .. } => {
            candidate_refuses(|| s.employee_expenses_candidate(value))
        }
        P::SaveDeductions { value, .. } => candidate_refuses(|| s.deductions_candidate(value)),
        P::SaveMedical { value, .. } => candidate_refuses(|| s.medical_candidate(value)),
        P::SaveTaxes { value, .. } => candidate_refuses(|| {
            s.expense_owner_candidate(s.taxes_candidate(value), TaxSection::Taxes, None)
        }),
        P::SaveMortgages { value, .. } => candidate_refuses(|| s.mortgages_candidate(value)),
        P::SaveGiftCarryovers { value, .. } => {
            candidate_refuses(|| s.gift_carryovers_candidate(value))
        }
        P::SaveOtherItemized { value, .. } => candidate_refuses(|| {
            s.expense_owner_candidate(
                s.other_itemized_candidate(value),
                TaxSection::OtherItemized,
                None,
            )
        }),
        P::SaveEducation { value, .. } => candidate_refuses(|| s.education_candidate(value)),
        P::SaveSaver { value, .. } => candidate_refuses(|| s.saver_candidate(value)),
        P::SaveMarketplace { value, .. } => candidate_refuses(|| s.marketplace_candidate(value)),
        P::SaveMoreCredits { value, .. } => candidate_refuses(|| {
            s.expense_owner_candidate(
                s.more_credits_candidate(value),
                TaxSection::MoreCredits,
                value.credits.iter().position(|credit| matches!(credit, MoreCredit::ForeignTax(_))),
            )
        }),
        // This predicate only asks whether the original source needs reopening;
        // the actual Save/Estimate paths retain the typed failure and locator.
        P::SaveOtherTaxes { value, .. } => candidate_refuses(|| s.other_taxes_candidate(value)),
        P::SaveMoreAdjustments { value, .. } => {
            candidate_refuses(|| s.more_adjustments_candidate(value))
        }
        P::SavePayments { value, .. } => candidate_refuses(|| s.payments_candidate(value)),
        P::SaveSituations { topic, value, .. } => {
            candidate_refuses(|| s.situations_candidate(*topic, value))
        }
        _ => false,
    }
}

// The checklist's lines: a topic's entries are owned by its tick (FLAGS:
// `entered` is the datum, `clear` makes it absent the way saving its page
// with nothing on it would).
fn prune_unticked(d: &mut FederalDraft, removed: &mut Vec<Removed>) {
    for flag in &FLAGS {
        if ticked(&d.checklist, flag) || !(flag.entered)(d) {
            continue;
        }
        let before = d.clone();
        (flag.clear)(d);
        let mut rows_only = before.clone();
        rows_only.income = d.income.clone();
        removed_rows(&before, d, removed);
        if rows_only != *d {
            removed.push(answer(flag.removal_key));
        }
    }
}
// The income documents a line removed, each named by its kind and payer.
fn removed_rows(before: &FederalDraft, d: &FederalDraft, removed: &mut Vec<Removed>) {
    for entry in &before.income {
        if !d.income.iter().any(|e| e.id == entry.id) {
            removed.push(entry_row(document_warning_kind(entry), entry, before.dependents()));
        }
    }
}
// Saving a topic page with nothing on it: its entries and the answers it
// owns go (documents_candidate replaces every entry of its kinds). Nothing
// is visited: the sections recorded stay as they were.
pub(super) fn empty_topic(d: &mut FederalDraft, topic: TaxSection) {
    let viewing = Interview::viewing(d.clone());
    match viewing.documents_candidate(
        topic,
        IncomeDocuments {
            qbi_aggregations: crate::qbi_aggregation::project_pages(
                d.screening.as_ref().map_or(&[], |v| v.qbi_aggregations.as_slice()),
                &crate::qbi_aggregation::source_views(d, d),
            ),
            receipt_context: crate::BusinessReceiptContext::default(),
            documents: Vec::new(),
        },
    ) {
        Ok(mut draft) => {
            draft.finished_sections = d.finished_sections.clone();
            *d = draft;
        }
        // A return whose other answers the page would refuse still loses
        // this topic's documents.
        Err(_) => {
            let removed: alloc::collections::BTreeSet<_> =
                d.income.iter().filter(|e| topic_of(d, e) == Some(topic)).map(|e| e.id).collect();
            d.income.retain(|e| !removed.contains(&e.id));
        }
    }
}

// What one pass reads before it changes anything: the walk and the allowed
// lists of the candidate as the pass begins.
struct Allowed {
    joint: bool,
    owners: Vec<Owner>,
    // The owners of income documents: a community-property separate return
    // also enters its spouse's documents (income_owners).
    income_owners: Vec<Owner>,
    // Income rows retain property answers offered by their owning allowed set.
    community: bool,
    itemizing: bool,
    box_c: bool,
    injured: bool,
    // Form 8615 applies: the parent's return page asks its facts.
    parent_tax: bool,
    // The IRA spouse part the setup asks (FederalSetup::ira_spouse_index).
    ira_spouse: Option<i32>,
    // A partnership's income source is asked (partner_source_needed).
    partner_source: bool,
    // The people an education row and a long-term-care premium can be for.
    students: Vec<i64>,
    // The children a Form 8814 election can report (child_election_children).
    children: Vec<i64>,
    insured: Vec<i64>,
    // The situations each page offers (allowed_situations).
    situations: Vec<RareSituation>,
    // The repayment comparison is offered (allowed_repayment).
    repayment: bool,
    // The retirement income an annuity cost can name (annuity_contracts).
    contracts: Vec<i64>,
    // The Form 1095-A answers kept inside the page's allowed lists, as the
    // page saves them; None when the stored claim already is.
    marketplace: Option<FederalDraft>,
}
impl Allowed {
    fn of(d: &FederalDraft) -> Self {
        let viewing = Interview::viewing(d.clone());
        Self {
            joint: joint(d),
            owners: viewing
                .allowed_owners()
                .into_iter()
                .filter_map(|o| {
                    [Owner::Taxpayer, Owner::Spouse].into_iter().find(|v| *v as i32 == o)
                })
                .collect(),
            income_owners: viewing
                .income_owners()
                .into_iter()
                .filter_map(|o| {
                    [Owner::Taxpayer, Owner::Spouse].into_iter().find(|v| *v as i32 == o)
                })
                .collect(),
            community: crate::injured_spouse::community_answers(d)
                .contains(&crate::community::index(crate::CommunityShare::Community)),
            itemizing: itemizing(d),
            box_c: in_walk(d, TaxSection::IncomeByPeriod),
            injured: in_walk(d, TaxSection::InjuredSpouse),
            parent_tax: in_walk(d, TaxSection::ParentTax),
            ira_spouse: d.setup.as_ref().map(FederalSetup::ira_spouse_index),
            partner_source: viewing.partner_source_needed(),
            students: education_people(d).iter().map(|p| p.id).collect(),
            children: child_election_children(d).iter().map(|c| c.id).collect(),
            insured: medical_people(d).into_iter().map(|(p, _)| p.id).collect(),
            situations: RareSituation::ALL
                .iter()
                .copied()
                .filter(|s| {
                    let topic = s.destination().0;
                    in_walk(d, topic) && allowed_situations(topic).contains(&situation_index(*s))
                })
                .collect(),
            marketplace: marketplace_in_lists(d),
            repayment: crate::claim_repayment::repayment_offered(d),
            contracts: annuity_contracts(d).iter().map(|c| c.id).collect(),
        }
    }
    fn owner(&self, owner: Owner) -> bool {
        self.owners.contains(&owner)
    }
    // A list owned row by row through allowed_owners: each row of another
    // filer goes, named by its kind and whose it was.
    fn owned<T>(
        &self,
        rows: &mut Vec<T>,
        owner: impl Fn(&T) -> Owner,
        kind: &'static str,
        removed: &mut Vec<Removed>,
    ) {
        rows.retain(|r| {
            let kept = self.owner(owner(r));
            if !kept {
                removed.push(row_resource(kind, owner_name_key(owner(r))));
            }
            kept
        });
    }
}
// The stored IRA spouse part as the setup's allowed index names it.
fn ira_spouse_index(spouse: &IraSpouse) -> i32 {
    (match spouse {
        IraSpouse::NotApplicable => libertas_macros::variant_index!(IraSpouse::NotApplicable),
        IraSpouse::Joint { .. } => libertas_macros::variant_index!(IraSpouse::Joint),
        IraSpouse::SeparateTogether { .. } => {
            libertas_macros::variant_index!(IraSpouse::SeparateTogether)
        }
    }) as i32
}
fn situation_index(situation: RareSituation) -> i32 {
    RareSituation::ALL.iter().position(|s| *s == situation).map_or(-1, |i| i as i32)
}
fn owner_name_key(owner: Owner) -> &'static str {
    match owner {
        Owner::Taxpayer => "SAVING_OWNER_YOURS",
        Owner::Spouse => "SAVING_OWNER_SPOUSE",
    }
}

// The Form 1095-A answers as the marketplace page shows them, kept inside
// its allowed lists (allowed_sharing and the people you support; line A
// only on a separate return) and saved
// again by the page. None when nothing is outside them, or when the page
// now asks an answer it has none for (line A), which level 2 reopens.
fn marketplace_in_lists(d: &FederalDraft) -> Option<FederalDraft> {
    let mut value = marketplace_value(d);
    if value.forms.is_empty() {
        return None;
    }
    let status = filing_status(d);
    let original = value.clone();
    let separate = allowed_separate(status);
    if !separate.contains(&(value.separate as i32)) {
        if !separate.contains(&(SeparateRelief::NotSeparate as i32)) {
            return None;
        }
        value.separate = SeparateRelief::NotSeparate;
    }
    if value == original {
        return None;
    }
    let viewing = Interview::viewing(d.clone());
    viewing.marketplace_candidate(&value).ok()
}

// The ownership table: every stored field, the page that owns it and the
// line that keeps it. A field no page asks yet is "unasked": it keeps its
// nothing-applies value (fill_unasked) until its milestone's page owns it.
fn prune_lines(d: &mut FederalDraft, removed: &mut Vec<Removed>) {
    let a = Allowed::of(d);
    let marketplace_family = crate::marketplace::family_people(d);
    let before = d.clone();
    // The marketplace page: its answers inside its allowed lists, as the
    // page saves them (its catalog and education rows follow the people).
    if let Some(saved) = &a.marketplace {
        d.credits = saved.credits.clone();
        removed.push(answer("SAVING_REMOVES_SOME_OF_YOUR_FORM_1095_A_ANSWERS"));
    }
    let farm_sources: Vec<_> =
        crate::farm_specials::source_choices(d).into_iter().map(|source| source.id).collect();
    let research_sources = crate::business_credit_page::research_businesses(d);
    let retained_sources_available = !crate::retained_research::available(d).is_empty();
    let investment_entities = crate::investment_interest::k1_sources(d);
    let unavailable_entered: Vec<_> = d
        .credits
        .as_ref()
        .and_then(|credits| credits.business_credits.as_ref())
        .map_or_else(Vec::new, |history| {
            history
                .credits
                .iter()
                .enumerate()
                .filter(|(_, credit)| {
                    !matches!(credit.source, crate::CreditSource::Bought)
                        && crate::general_business_credit::resolve_source(d, &credit.source)
                            .is_none()
                })
                .map(|(index, _)| index)
                .collect()
        });
    let FederalDraft {
        // The return's identity and revision, not answers.
        tax_year: _,
        next_id: _,
        cookie: _,
        revision: _,
        // Page 1, always in the walk.
        setup: _,
        // Your page (taxpayer), the spouse page (a joint return) and the
        // parent's return page (Form 8615's parent facts, while it applies).
        people,
        // The people page, always in the walk (gate fixes batch A); the
        // care claim only with someone on it.
        dependents,
        // The topic pages (their ticks, FLAGS); each row's owner inside
        // allowed_owners; a partnership's income source only while asked;
        // a home sale's spouse answers only on a joint return.
        income,
        home_sites: _,
        // Each page's mark, owned with its page.
        finished_sections,
        adjustments,
        deductions,
        credits,
        screening,
        payments,
        // The review.
        finished: _,
        // The situations pages: each inside its page's allowed_situations.
        situations,
        // The refusal kinds of More adjustments, More credits and Other
        // taxes, pages always in the walk.
        adjustment_refusals: _,
        credit_refusals: _,
        tax_refusals: _,
        coverdell_excess,
        // The checklist, always in the walk; its ticks own the topics'
        // entries, and its joint-only whole-return items need a joint return.
        checklist,
        // Form 8379's page, while the injured spouse item is ticked.
        injured_spouse,
    } = d;

    if !a.joint && checklist.whole_return & crate::federal_model::WHOLE_RETURN_JOINT_ONLY != 0 {
        let ticks = *checklist;
        for flag in FLAGS.iter().filter(|f| {
            f.catalog == Catalog::WholeReturn
                && f.bit & crate::federal_model::WHOLE_RETURN_JOINT_ONLY != 0
                && ticked(&ticks, f)
        }) {
            set_tick(checklist, flag, false);
            if !removed.contains(&answer(flag.removal_key)) {
                removed.push(answer(flag.removal_key));
            }
        }
    }

    if let Some(people) = people.as_mut()
        && !a.joint
        && people.spouse.is_some()
    {
        people.spouse = SpouseFiler::No;
        removed.push(answer("SAVING_REMOVES_YOUR_SPOUSE"));
    }
    if let Some(people) = people.as_mut()
        && !a.parent_tax
        && people.parent_tax.take().is_some()
    {
        removed.push(answer("SAVING_REMOVES_YOUR_PARENT_S_RETURN_FORM_8615"));
    }
    income.retain(|e| a.income_owners.contains(&e.owner));
    let old_coverdell = coverdell_excess.len();
    coverdell_excess.retain(|v| a.owners.contains(&v.beneficiary));
    if old_coverdell != coverdell_excess.len() {
        removed.push(answer("SAVING_COVERDELL_BENEFICIARY"));
    }
    for beneficiary in coverdell_excess.iter_mut() {
        for contributor in &mut beneficiary.contributors {
            if matches!(contributor.person, crate::CoverdellContributorKind::Filer {owner} if !a.owners.contains(&owner))
            {
                contributor.person = crate::CoverdellContributorKind::Unavailable;
                removed.push(answer("SAVING_COVERDELL_CONTRIBUTOR"));
            }
        }
        for correction in &mut beneficiary.corrections {
            if correction.income_document.is_some_and(|id| !income.iter().any(|e| e.id == id)) {
                correction.income_document = None;
                removed.push(answer("SAVING_COVERDELL_CORRECTION_SOURCE"));
            }
        }
    }
    // Retain property answers for ordinary division or injured-spouse
    // allocation while the owning allowed set offers those choices.
    if !a.community {
        for entry in income.iter_mut() {
            entry.community = crate::CommunityShare::NotNeeded;
        }
    }
    // A child's election goes with its child: one no longer on the return, or
    // no longer one the return can report (the row is named by its child in
    // the save warning, removed_rows).
    income.retain(|e| match &e.income {
        FederalIncome::ChildElection { data } => a.children.contains(&data.child_id),
        _ => true,
    });
    // The actual employer link follows its source K-1 through ordinary reconciliation.
    let wage_scorporations: Vec<_> = income
        .iter()
        .filter_map(|entry| {
            matches!(
                entry.income,
                FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. }
            )
            .then_some(entry.id)
        })
        .collect();
    for entry in income.iter_mut() {
        if let FederalIncome::Wage { data } = &mut entry.income
            && data.paying_scorporation.is_some_and(|id| !wage_scorporations.contains(&id))
        {
            data.paying_scorporation = None;
            removed.push(answer("SAVING_REMOVES_W2_SCORP_EMPLOYER"));
        }
    }
    if let Some(borrowing) = deductions.as_mut().and_then(|v| v.investment_interest.as_mut()) {
        for loan in &mut borrowing.loans {
            if loan
                .originating_k1
                .is_some_and(|id| !investment_entities.iter().any(|entity| entity.id == id))
            {
                loan.originating_k1 = None;
                removed.push(answer("SAVING_REMOVES_INVESTMENT_K1_ORIGIN"));
            }
        }
    }
    if let Some(deductions) = deductions.as_mut() {
        for record in &mut deductions.charity_carryovers {
            if let Some(crate::ResearchCharityParts::Parts { sources }) = &mut record.research_parts
            {
                let before = sources.len();
                sources.retain(|p| {
                    retained_sources_available
                        && match p.origin {
                            crate::ResearchCharityOrigin::CurrentK1 { source } => {
                                investment_entities.iter().any(|r| r.id == source)
                            }
                            crate::ResearchCharityOrigin::OtherEntity(_) => true,
                        }
                });
                if sources.len() != before {
                    removed.push(answer("SAVING_REMOVES_RETAINED_RESEARCH_SOURCE"));
                }
                if sources.is_empty() {
                    record.research_parts = None;
                }
            }
        }
    }
    if let Some(crate::NolHistory::Records { years, .. }) = screening.as_mut().map(|s| &mut s.nol) {
        for year in years {
            if let Some(crate::ResearchLossParts::Parts { sources }) = &mut year.research_parts {
                let before = sources.len();
                sources.retain(|p| {
                    retained_sources_available
                        && year.balance > 0
                        && research_sources.iter().any(|r| r.id == p.source)
                });
                if sources.len() != before {
                    removed.push(answer("SAVING_REMOVES_RETAINED_RESEARCH_SOURCE"));
                }
                if sources.is_empty() {
                    year.research_parts = None;
                }
            }
        }
    }
    if let Some(screen) = screening.as_mut() {
        for record in &mut screen.capital_history {
            let caps = crate::extended_rules::capital_caps(record);
            if let Some(crate::ResearchCapitalParts::Parts { sources }) = &mut record.research_parts
            {
                let before = sources.len();
                sources.retain(|p| {
                    retained_sources_available
                        && record.ordinary_history
                        && caps.short + caps.long > 0
                        && research_sources.iter().any(|r| r.id == p.source)
                });
                if sources.len() != before {
                    removed.push(answer("SAVING_REMOVES_RETAINED_RESEARCH_SOURCE"));
                }
                if sources.is_empty() {
                    record.research_parts = None;
                }
            }
        }
    }
    if let Some(history) = credits.as_mut().and_then(|v| v.business_credits.as_mut()) {
        for source in history
            .carryforwards
            .iter_mut()
            .map(|carry| &mut carry.research_source)
            .chain(history.carrybacks.iter_mut().map(|carry| &mut carry.research_source))
        {
            if source.is_some_and(|id| !research_sources.iter().any(|entry| entry.id == id)) {
                *source = None;
                removed.push(answer("SAVING_REMOVES_RESEARCH_CREDIT_BUSINESS"));
            }
        }
        let mut index = 0;
        history.credits.retain(|_| {
            let keep = !unavailable_entered.contains(&index);
            index += 1;
            if !keep {
                removed.push(answer("SAVING_REMOVES_ENTERED_CREDIT_SOURCE"));
            }
            keep
        });
    }
    // The W-2 owns its optional Schedule C override. A removed/retyped business
    // restores the computed no-expense Schedule C, with the normal save warning.
    let statutory_businesses: Vec<_> = income
        .iter()
        .filter_map(|e| {
            matches!(e.income, FederalIncome::Business { .. }).then_some((e.id, e.owner))
        })
        .collect();
    for entry in income.iter_mut() {
        if let FederalIncome::Wage { data } = &mut entry.income
            && data.statutory_business.is_some_and(|id| {
                !data.statutory_employee || !statutory_businesses.contains(&(id, entry.owner))
            })
        {
            data.statutory_business = None;
            removed.push(entry_row("SAVING_STATUTORY_BUSINESS_ASSOCIATION", entry, dependents));
        }
    }
    for entry in income.iter_mut() {
        let farm = match &mut entry.income {
            FederalIncome::Business { data } => match &mut data.activity {
                BusinessActivity::Farming { data } => Some(data),
                _ => None,
            },
            FederalIncome::FarmRental { data } => Some(&mut data.farm),
            _ => None,
        };
        if let Some(farm) = farm {
            let before = farm.specials.external_sources.len();
            farm.specials.external_sources.retain(|source| farm_sources.contains(&source.source));
            if before != farm.specials.external_sources.len() {
                removed.push(entry_row("SAVING_KIND_FARM_EXTERNAL_SOURCE", entry, dependents));
            }
        }
        if let FederalIncome::Partnership { data } = &mut entry.income
            && !a.partner_source
            && (data.income_source.is_some() || data.gross_personal_services.is_some())
        {
            data.income_source = None;
            data.gross_personal_services = None;
            removed.push(entry_row("SAVING_KIND_PARTNERSHIP_INCOME_SOURCE", entry, dependents));
        }
        // A home sale's spouse answers only on a joint return (u6-home).
        if let FederalIncome::HomeSale { data } = &mut entry.income
            && !a.joint
            && data.drop_spouse()
        {
            removed.push(entry_row("SAVING_KIND_HOME_SALE_S_SPOUSE_ANSWERS", entry, dependents));
        }
        // A Form 1099-Q whose student left the return keeps the form (its
        // amounts are the payer's): the student is now one who is not on the
        // return, with no costs assigned. A 529 and Coverdell expenses row
        // only for a person still on the return.
        if let FederalIncome::EducationDistribution { data } = &mut entry.income
            && let crate::FederalEducationBeneficiary::Student { person } = data.beneficiary
            && !a.students.contains(&person)
        {
            data.beneficiary = crate::FederalEducationBeneficiary::OtherStudent {
                expenses: 0,
                assistance: 0,
                credit_expenses: 0,
            };
            removed.push(entry_row("SAVING_KIND_1099_Q_STUDENT", entry, dependents));
        }
        if let FederalIncome::EducationAccountExpenses { data } = &mut entry.income {
            let before = data.students.len();
            data.students.retain(|s| a.students.contains(&s.person));
            if data.students.len() != before {
                removed.push(entry_row(
                    "SAVING_KIND_EDUCATION_ACCOUNT_EXPENSES_STUDENT",
                    entry,
                    dependents,
                ));
            }
        }
        // A business or partnership long-term-care premium only for a person
        // still on the return.
        let plan = match &mut entry.income {
            FederalIncome::Business { data } => Some(&mut data.health_insurance),
            FederalIncome::Partnership { data } => Some(&mut data.health_insurance),
            _ => None,
        };
        if plan.is_some_and(|plan| prune_care_premiums(plan, &a.insured)) {
            removed.push(entry_row("SAVING_KIND_LONG_TERM_CARE_PREMIUM", entry, dependents));
        }
        // A savings-bond row only for a person still on the return.
        if let FederalIncome::SavingsBonds { data } = &mut entry.income {
            let before = data.students.len();
            data.students.retain(|s| a.students.contains(&s.person));
            if data.students.len() != before {
                removed.push(entry_row("SAVING_KIND_SAVINGS_BONDS_PERSON", entry, dependents));
            }
        }
    }
    // A savings-bond record with no person left is not a record the page
    // could have written.
    income.retain(|e| match &e.income {
        FederalIncome::SavingsBonds { data } if data.students.is_empty() => {
            removed.push(entry_row("SAVING_KIND_SAVINGS_BONDS", e, dependents));
            false
        }
        // A periods record for a person who left the return is not one the page could show.
        FederalIncome::LtcInsured { data }
            if matches!(
                data.insured,
                crate::FederalLtcInsuredPerson::OnReturn { person } if !a.students.contains(&person)
            ) =>
        {
            removed.push(entry_row("SAVING_KIND_LTC_INSURED", e, dependents));
            false
        }
        FederalIncome::EducationAccountExpenses { data } if data.students.is_empty() => {
            removed.push(entry_row("SAVING_KIND_EDUCATION_ACCOUNT_EXPENSES", e, dependents));
            false
        }
        _ => true,
    });
    // A link names a record by its identity; when that record is gone the link
    // is cleared, so a stored link is always to a record that exists (a 1099-G
    // box 7 or 1099-PATR farm, a 1099-LTC periods record). The document then
    // takes the choice-less path again.
    let farm_ids: Vec<i64> = income
        .iter()
        .filter(|e| match &e.income {
            FederalIncome::Business { data } => {
                matches!(data.activity, crate::BusinessActivity::Farming { .. })
            }
            FederalIncome::FarmRental { .. } => true,
            _ => false,
        })
        .map(|e| e.id)
        .collect();
    let periods_ids: Vec<i64> = income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::LtcInsured { .. }))
        .map(|e| e.id)
        .collect();
    for entry in income.iter_mut() {
        let (link, ids, kind) = match &mut entry.income {
            FederalIncome::Unemployment { data } => {
                (&mut data.agriculture_farm, &farm_ids, "SAVING_KIND_FARM_CHOICE")
            }
            FederalIncome::LtcBenefits { data } => {
                (&mut data.insured, &periods_ids, "SAVING_KIND_LTC_PERIODS_CHOICE")
            }
            _ => continue,
        };
        if link.is_some_and(|id| !ids.contains(&id)) {
            *link = None;
            removed.push(entry_row(kind, entry, dependents));
        }
    }
    situations.retain(|s| {
        let kept = a.situations.contains(s);
        if !kept {
            removed.push(row_resource("SAVING_KIND_SITUATION", s.refused().name_key));
        }
        kept
    });
    let injured_key = answer("SAVING_REMOVES_YOUR_INJURED_SPOUSE_ALLOCATION_FORM_8379");
    if !a.injured && injured_spouse.take().is_some() && !removed.contains(&injured_key) {
        removed.push(injured_key);
    }

    if let Some(FederalAdjustments {
        health_employer_w2: _,
        health_marketplace: _,
        // The retirement topic (its lump-sum kind; absent is Regular).
        social_election: _,
        // More adjustments, by owner.
        alimony_paid,
        // More adjustments' moving expenses (Form 3903), by owner.
        moving,
        // The student loan interest page.
        student_loan: _,
        // The educator page: the spouse's row only on a joint return.
        educators,
        // The Form 2106 page: forms by owner.
        employee_expenses,
        // The HSA page: actual accounts by owner. Retain the contribution-limit
        // method on every return; other-filer eligibility and agreed shares
        // remain available on a married separate return.
        hsa,
        hsa_family_share,
        // The HSA page: Archer MSA contributions (Form 8853 Part I), always in
        // the walk. A spouse's row stays on a separate return, where it only
        // carries the family-limit facts.
        archer: _,
        // The retirement topic (QCDs and Forms 8606), by owner.
        qualified_charity,
        ira_basis,
        // The retirement topic's missed-RMD kind (Form 5329 Part IX), by
        // owner.
        required_distributions,
        // The retirement topic's Roth IRA history kind, by owner (u6-roth).
        roth_history,
        // No page: W-2 box 12 code H is refused by name, and section
        // 501(c)(18)(D) contributions under More adjustments' other
        // adjustments (Schedule 1 line 24f); erased.
        pension_contributions,
        // The IRA page: the spouse's facts only on a joint return and the
        // separate return's spouse coverage only where asked (the page
        // writes the part ira_spouse_index names).
        iras,
    }) = adjustments.as_mut()
    {
        a.owned(alimony_paid, |v| v.owner, "SAVING_KIND_ALIMONY_RECORD", removed);
        a.owned(moving, |v| v.owner, "SAVING_KIND_MOVE", removed);
        if !a.joint && educators.spouse.is_some() {
            educators.spouse = None;
            removed.push(answer("SAVING_REMOVES_YOUR_SPOUSE_S_EDUCATOR_EXPENSES"));
        }
        a.owned(employee_expenses, |v| v.owner, "SAVING_KIND_EMPLOYEE_EXPENSES_FORM", removed);
        let had_spouse_hsa = hsa.iter().any(|h| h.owner == Owner::Spouse);
        a.owned(hsa, |h| h.owner, "SAVING_KIND_HSA", removed);
        if hsa.is_empty() {
            if hsa_family_share.take().is_some() {
                removed.push(answer("SAVING_REMOVES_THE_HSA_FAMILY_SPLIT"));
            }
        } else if !before.setup.as_ref().is_some_and(|s| s.marital_state.married())
            && let Some(share) = hsa_family_share
            && (had_spouse_hsa
                || share.other_coverage.is_some()
                || (share.taxpayer_parts, share.spouse_parts) != (1, 1))
        {
            share.other_coverage = None;
            share.taxpayer_parts = 1;
            share.spouse_parts = 1;
            removed.push(answer("SAVING_REMOVES_THE_HSA_FAMILY_SPLIT"));
        }
        a.owned(qualified_charity, |v| v.owner, "SAVING_KIND_QCD_RECORD", removed);
        a.owned(ira_basis, |v| v.owner, "SAVING_KIND_FORM_8606", removed);
        a.owned(required_distributions, |v| v.owner, "SAVING_KIND_REQUIRED_DISTRIBUTION", removed);
        a.owned(roth_history, |v| v.owner, "SAVING_KIND_ROTH_IRA_HISTORY", removed);
        if !pension_contributions.is_empty() {
            pension_contributions.clear();
            removed.push(answer("SAVING_REMOVES_YOUR_SECTION_501_C_18_D_CONTRIBUTIONS"));
        }
        if iras.spouse != IraSpouse::NotApplicable
            && a.ira_spouse != Some(ira_spouse_index(&iras.spouse))
        {
            // A record with nothing entered follows the return silently.
            if iras.entered() {
                removed.push(answer("SAVING_REMOVES_YOUR_SPOUSE_S_IRA_FACTS"));
            }
            iras.spouse = IraSpouse::NotApplicable;
        }
    }

    if let Some(FederalDeductions {
        // Mortgage facts also serve rentals and survive deduction route changes.
        mortgage: _,
        // Borrowing is source-owned and survives deduction route changes.
        investment_interest: _,
        // The deductions page, always in the walk: the repayment comparison
        // only while its allowed_repayment offers it (recorded earlier-year
        // repayments above $3,000); the vehicle loans, owned by the car loan
        // tick (FLAGS); the casualty events.
        repayment,
        vehicle_loans: _,
        personal_casualties: _,
        // The other itemized page (itemizing): each row for retirement
        // income in its catalog (annuity_contracts).
        annuity_losses,
        // The deductions page's route (its allowed_routes, level 2); the
        // itemized topics own its facts.
        choice: _,
        spouse_itemizes: _,
        // The deductions page.
        cash_charity: _,
        fund_cash_charity: _,
        other_cash_charity: _,
        // The gifts of property and carryovers page (itemizing): its lists,
        // the whaling captains by owner.
        volunteer_expenses: _,
        hosted_students: _,
        whaling_captains: _,
        property_gifts: _,
        appraisals: _,
        basis_election: _,
        charity_carryovers: _,
    }) = deductions.as_mut()
    {
        if !a.repayment && *repayment != RepaymentClaim::None {
            *repayment = RepaymentClaim::None;
            removed.push(answer("SAVING_REMOVES_YOUR_EARLIER_YEAR_REPAYMENT_COMPARISON"));
        }
        annuity_losses.retain(|l| {
            let kept = a.itemizing && a.contracts.contains(&l.income_id);
            if !kept {
                removed.push(answer("SAVING_REMOVES_YOUR_UNRECOVERED_ANNUITY_COST"));
            }
            kept
        });
    }
    if let Some(v) = deductions.as_mut() {
        a.owned(&mut v.whaling_captains, |c| c.owner, "SAVING_KIND_WHALING_CAPTAIN_CLAIM", removed);
        if !a.itemizing && super::gift_page_entered(v) {
            super::clear_gift_page(v);
            removed.push(answer("SAVING_REMOVES_YOUR_GIFTS_OF_PROPERTY_AND_CARRYOVERS"));
        }
        if let DeductionSelection::Automatic { expenses: ItemizedChoice::Yes { facts } }
        | DeductionSelection::Itemized { facts } = &mut v.choice
        {
            // Medical page: premiums only for a person on the return.
            facts.long_term_care.retain(|p| {
                let kept = a.insured.contains(&p.person);
                if !kept {
                    removed.push(row("SAVING_KIND_LONG_TERM_CARE_PREMIUM", ""));
                }
                kept
            });
        }
    }

    if let Some(FederalCredits {
        decline_earned_income_credit: _,
        // More credits.
        tip_fica_carryforward: _,
        business_credits: _,
        bba_audit_interest: _,
        minimum_tax,
        mortgage_credit: _,
        // The other taxes page; each sale's hidden home sale ID is derived
        // from the return's home sales on every save (follow_home_sales).
        mortgage_subsidies: _,
        // More credits.
        dc_homebuyer_carry: _,
        residential_energy_carry: _,
        personal_refueling: _,
        clean_vehicles: _,
        legacy_ev_passive_allowed: _,
        // The education page: rows for a person on the return.
        education,
        // More credits.
        education_recapture: _,
        // The people page (care costs), while you support someone; the
        // disabled spouse's care only on a joint return.
        care,
        // The saver page: the spouse's amounts only on a joint return.
        saver,
        // More credits.
        adoptions: _,
        adoption_carry: _,
        // The marketplace page, inside its allowed lists (above, and
        // level 2).
        marketplace,
        // Interest and dividends (box 6/7 statements) and more credits.
        foreign_tax: _,
        // More credits.
        exempt_pensions: _,
        // More credits, by owner.
        tax_credit_bonds,
        after_disallowance: _,
    }) = credits.as_mut()
    {
        if let Some(histories) = minimum_tax {
            let before = histories.returns.len();
            histories.returns.retain(|history| match history.unit {
                crate::MinimumTaxHistoryUnit::Individual { owner } => a.owners.contains(&owner),
                crate::MinimumTaxHistoryUnit::FormerJoint { claimant } => {
                    a.owners.contains(&claimant)
                }
                crate::MinimumTaxHistoryUnit::SameJointCouple => a.joint,
            });
            if before != histories.returns.len() {
                removed.push(answer("SAVING_REMOVES_MINIMUM_TAX_HISTORY"));
            }
            if histories.returns.is_empty() {
                *minimum_tax = None;
            }
        }
        a.owned(tax_credit_bonds, |v| v.owner, "SAVING_KIND_TAX_CREDIT_BOND", removed);
        education.retain(|e| {
            let kept = a.students.contains(&e.student);
            if !kept {
                removed.push(Removed::Row { kind: "SAVING_KIND_EDUCATION_CREDIT", name: None });
            }
            kept
        });
        // A dependent return of a person who left the return goes with them.
        if let crate::MarketplaceClaim::Yes { facts } = marketplace {
            if matches!(&facts.marriage, crate::MarketplaceMarriage::Alternative { you, spouse, .. }
                if !a.joint || you.dependents.iter().chain(&spouse.dependents)
                    .any(|id| !marketplace_family.contains(id)))
            {
                facts.marriage = crate::MarketplaceMarriage::Regular;
                finished_sections.retain(|section| *section != TaxSection::Marketplace);
                removed.push(answer("SAVING_REMOVES_INVALID_MARKETPLACE_MARRIAGE_ELECTION"));
            }
            facts.dependent_qsehra_employees.retain(|id| marketplace_family.contains(id));
            facts.dependents.retain(|m| {
                let kept = a.students.contains(&m.person);
                if !kept {
                    removed.push(Removed::Row {
                        kind: "SAVING_KIND_MARKETPLACE_DEPENDENT_RETURN",
                        name: None,
                    });
                }
                kept
            });
        }
        // Care facts belong to this page, independently of the current dependent list.
        // Source-constrained rows follow their actual source; the remaining answers survive.
        let old_care = care.clone();
        care.benefits.retain(|b|income.iter().any(|e|e.id==b.wage&&matches!(&e.income,FederalIncome::Wage {data} if data.care_benefits>0&&b.in_box_1.unwrap_or(0)+b.forfeited.unwrap_or(0)<=data.care_benefits)));
        care.grace.retain(|g| a.owners.contains(&g.owner));
        care.own_business.retain(|b|a.owners.contains(&b.owner)&&income.iter().any(|e|e.owner==b.owner&&match b.source {
            crate::CareBenefitSource::Business {business}=>e.id==business&&matches!(&e.income,FederalIncome::Business {data} if !matches!(data.activity,BusinessActivity::Farming {..})),
            crate::CareBenefitSource::Farm {farm}=>e.id==farm&&matches!(&e.income,FederalIncome::Business {data} if matches!(data.activity,BusinessActivity::Farming {..})),
            crate::CareBenefitSource::Partnership {partnership}=>e.id==partnership&&matches!(e.income,FederalIncome::Partnership {..}),
        }));
        if care.grace.is_empty()
            && care.own_business.is_empty()
            && !income
                .iter()
                .any(|e| matches!(&e.income, FederalIncome::Wage {data} if data.care_benefits > 0))
        {
            care.plan_maximum = crate::CarePlanMaximum::NoBenefits;
        }
        care.filers.retain(|f| {
            let p = people.as_ref().and_then(|p| {
                if f.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
            });
            let Some(p) = p.filter(|p| {
                a.joint
                    && match f.situation {
                        crate::CareFilerSituation::Student { .. } => p.full_time_student,
                        crate::CareFilerSituation::Incapable { .. } => p.incapable_self_care,
                    }
            }) else {
                return false;
            };
            let months = match &f.situation {
                crate::CareFilerSituation::Student { months, .. }
                | crate::CareFilerSituation::Incapable { months, .. } => months,
            };
            // The aggregate earned income belongs to this exact selection.
            // A new death bound must reopen the answer, not reinterpret it
            // as income earned in a silently shortened set of months.
            if p.death_date()
                .is_some_and(|date| months.iter().any(|m| *m > i64::from(date / 100 % 100)))
            {
                return false;
            }
            !matches!(&f.situation,crate::CareFilerSituation::Student {months,..} if months.len()<5)
        });
        let separate =
            before.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate);
        if separate && !matches!(care.filing, crate::CareFilingStatus::Separate { .. }) {
            care.filing = crate::CareFilingStatus::Separate { spouse_earned_income: None };
        }
        if !separate {
            care.filing = crate::CareFilingStatus::Other;
        }
        if *care != old_care {
            removed.push(answer("SAVING_REMOVES_CARE_SOURCE_ANSWERS"));
        }
        if let Some(facts) = saver.as_mut()
            && !a.joint
            && (facts.spouse_contributions != 0 || facts.spouse_able != 0)
        {
            facts.spouse_contributions = 0;
            facts.spouse_able = 0;
            removed.push(answer("SAVING_REMOVES_YOUR_SPOUSE_S_SAVER_S_CREDIT_CONTRIBUTIONS"));
        }
    }

    if let Some(FederalScreening {
        // The farming topic.
        farm_averaging: _,
        optional_se,
        // The Work abroad topic's Form 2555, by owner.
        foreign_earned,
        clergy,
        farm_nol: _,
        // The other taxes page, by owner.
        household_employment,
        trump_accounts,
        // The retirement topic's 1099-Rs: each row with a 1099-R pension
        // entry still on the return.
        public_safety,
        // The Other income topic's ABLE contributions kind (Form 5329 Part
        // VIII), by owner.
        able_contributions,
        // The investment sales topic, by owner.
        iso_exercises,
        capital_history: _,
        // Self-employment and rentals.
        section1231: _,
        // Self-employment and farming (the earlier-year operating-loss
        // kind), owned by their ticks.
        atnol: _,
        nol: _,
        // Self-employment and farming.
        qbi_history: _,
        // Interest and dividends.
        foreign_account: _,
        fbar_required: _,
        fbar_countries: _,
        foreign_trust: _,
        foreign_asset_max: _,
        // The retirement topic (Forms 8915-F), by owner.
        disaster_carries,
        // The investment sales topic, by owner.
        installment_payments,
        // The investment sales topic: one return-wide election amount.
        section_1256_carryback: _,
        section_1256_carryback_owners: _,
        related_exchanges,

        taxable_premium_election: _,
        qbi_aggregations: _,
    }) = screening.as_mut()
    {
        a.owned(optional_se, |v| v.owner, "SAVING_KIND_OPTIONAL_METHOD_ELECTION", removed);
        a.owned(clergy, |v| v.owner, "SAVING_KIND_CLERGY_RECORD", removed);
        a.owned(foreign_earned, |v| v.owner, "SAVING_KIND_FORM_2555", removed);
        a.owned(household_employment, |v| v.owner, "SAVING_KIND_HOUSEHOLD_EMPLOYER", removed);
        a.owned(able_contributions, |v| v.owner, "SAVING_KIND_ABLE_CONTRIBUTIONS", removed);
        public_safety.retain(|p| {
            let kept = income.iter().any(|e| {
                e.id == p.income_id && matches!(e.income, FederalIncome::Retirement { .. })
            }) && match &p.coverage {
                // A long-term-care policy only for a person still on the return.
                crate::PublicSafetyCoverage::LongTermCare { person } => a.insured.contains(person),
                _ => true,
            };
            if !kept {
                removed.push(answer("SAVING_REMOVES_YOUR_PUBLIC_SAFETY_OFFICER_PREMIUMS"));
            }
            kept
        });
        a.owned(trump_accounts, |v| v.owner, "SAVING_KIND_TRUMP_ACCOUNT", removed);
        a.owned(iso_exercises, |v| v.owner, "SAVING_KIND_FORM_3921", removed);
        a.owned(disaster_carries, |v| v.owner, "SAVING_KIND_FORM_8915_F", removed);
        a.owned(installment_payments, |v| v.owner, "SAVING_KIND_EARLIER_INSTALLMENT_SALE", removed);
        a.owned(related_exchanges, |v| v.owner, "SAVING_KIND_EARLIER_RELATED_EXCHANGE", removed);
    }

    if let Some(pay) = payments.as_mut() {
        // The payments page, entered for one household: a joint return's
        // payments and 2025 facts are the couple's and go with the joint
        // return (entered for one filer on a joint return, level 2 opens
        // the page again). Its allowed sets are level 2. Home kerosene is
        // the More credits page's (its fuel tax credit item) and stays.
        if pay.joint && !a.joint && !pay.nothing_entered() {
            *pay = FederalPayments {
                // This return's planned election is not a prior household's
                // payment source. Retain it for ordinary capacity review.
                credit_applied_next_year: pay.credit_applied_next_year,
                credit_recipient: pay.credit_recipient,
                home_kerosene: pay.home_kerosene.take(),
                joint: a.joint,
                ..FederalPayments::none()
            };
            finished_sections.retain(|s| *s != TaxSection::Payments);
            removed.push(answer("SAVING_REMOVES_YOUR_FEDERAL_PAYMENTS"));
        }
        // The income by period page, while box C is chosen.
        if !a.box_c && pay.annualized_income.take().is_some() {
            removed.push(answer("SAVING_REMOVES_YOUR_INCOME_BY_PERIOD"));
        }
    }

    // A page's mark goes with its page.
    finished_sections.retain(|s| in_walk(&before, *s));
    removed_rows(&before, d, removed);
}

#[cfg(test)]
mod message_limit_tests {
    use super::*;
    use alloc::string::String;
    use libertas::{AvroDecode, AvroEncode, LibertasMessageArgumentDecode as A};

    // Use the real routing inventory so adding another supported document kind
    // automatically adds its resource keys to this worst-width aggregate.
    fn supported_document_kinds() -> Vec<&'static str> {
        let source = include_str!("mod.rs");
        let routing = source
            .split("fn document_kind_keys(")
            .nth(1)
            .unwrap()
            .split("fn document_warning_kind(")
            .next()
            .unwrap();
        let mut keys = Vec::new();
        for part in routing.split('"').skip(1).step_by(2) {
            if part.starts_with("SAVING_KIND_") && !keys.contains(&part) {
                for suffix in ["_ONE", "_MANY"] {
                    let key = alloc::format!("{part}{suffix}");
                    assert!(crate::APP_STRINGS.iter().any(|(k, _)| *k == key));
                }
                keys.push(part);
            }
        }
        keys
    }

    fn decoded_tuple(bytes: &[u8], depth: usize) -> crate::common::FormattedMessage {
        assert!(depth <= 16, "nested message depth {depth}");
        if depth > 0 {
            assert!(bytes.len() <= 4096, "nested tuple has {} bytes", bytes.len());
        }
        let mut offset = 0;
        let tuple = crate::common::FormattedMessage::avro_decode(bytes, &mut offset).unwrap();
        assert_eq!(offset, bytes.len());
        for arg in &tuple.1 {
            if let A::FormattedText(child) = arg {
                decoded_tuple(child, depth + 1);
            }
        }
        tuple
    }

    fn count_phrases(tuple: &crate::common::FormattedMessage, result: &mut Vec<(String, i64)>) {
        if tuple.0 == "SUMMARY_DOCUMENT_COUNT" {
            let [A::Plural(count), A::ResourceText(one), A::ResourceText(many)] =
                tuple.1.as_slice()
            else {
                panic!("document count lost its typed arguments")
            };
            assert_eq!(one.strip_suffix("_ONE"), many.strip_suffix("_MANY"));
            result.push((many.clone(), *count));
        }
        for arg in &tuple.1 {
            if let A::FormattedText(child) = arg {
                count_phrases(&decoded_tuple(child, 1), result);
            }
        }
    }

    // The hub row names at most six kinds; the bounded summary of every
    // supported kind at the widest counts fits a nested tuple.
    #[test]
    fn every_supported_document_kind_fits_a_nested_summary_at_maximum_count_width() {
        let kinds = supported_document_kinds();
        assert!(kinds.len() > 20, "the supported document inventory was not read");
        let counts: Vec<(&'static str, i64)> = kinds.iter().map(|kind| (*kind, i64::MAX)).collect();
        // The real hub summary adds an amount outside the entire document list.
        let summary = Message::new(
            "SUMMARY_DOCUMENTS_WITH_AMOUNT",
            vec![
                MessageArg::Nested(alloc::boxed::Box::new(super::super::bounded_document_summary(
                    counts,
                ))),
                MessageArg::usd_cents(i64::MAX),
            ],
        );
        let bytes = summary.encode();
        let decoded = decoded_tuple(&bytes, 0);
        let mut found = Vec::new();
        count_phrases(&decoded, &mut found);
        // The first six kinds in inventory order (equal counts keep their order).
        let expected: Vec<_> = kinds
            .iter()
            .take(super::super::SUMMARY_KIND_LIMIT)
            .map(|kind| (alloc::format!("{kind}_MANY"), i64::MAX))
            .collect();
        assert_eq!(found, expected);
        assert_eq!(more_kinds(&decoded), Some(kinds.len() as i64 - 6));
    }

    // The N of "and N more kinds" in a decoded summary.
    fn more_kinds(tuple: &crate::common::FormattedMessage) -> Option<i64> {
        if tuple.0 == "SUMMARY_DOCUMENTS_MORE_KINDS" {
            let [A::Plural(more)] = tuple.1.as_slice() else {
                panic!("the remaining kinds lost their typed count")
            };
            return Some(*more);
        }
        tuple.1.iter().find_map(|arg| match arg {
            A::FormattedText(child) => more_kinds(&decoded_tuple(child, 1)),
            _ => None,
        })
    }

    #[test]
    fn the_hub_summary_lists_all_kinds_up_to_six_and_counts_the_rest() {
        for take in 1..=6 {
            let many: Vec<(&'static str, i64)> =
                (0..take).map(|i| (supported_document_kinds()[i], 1 + i as i64)).collect();
            let decoded = decoded_tuple(&super::super::bounded_document_summary(many).encode(), 0);
            let mut found = Vec::new();
            count_phrases(&decoded, &mut found);
            assert_eq!(found.len(), take);
            assert_eq!(more_kinds(&decoded), None);
        }
        // Seven kinds: the six highest counts, ties in entry order, "and 1 more kind".
        let seven: Vec<(&'static str, i64)> =
            supported_document_kinds().into_iter().take(7).zip([1, 5, 3, 5, 2, 9, 1]).collect();
        let decoded =
            decoded_tuple(&super::super::bounded_document_summary(seven.clone()).encode(), 0);
        let mut found = Vec::new();
        count_phrases(&decoded, &mut found);
        let want: Vec<_> = [5usize, 1, 3, 2, 4, 0]
            .into_iter()
            .map(|i| (alloc::format!("{}_MANY", seven[i].0), seven[i].1))
            .collect();
        // Nine, five (first), five (second), three, two, one (first): the
        // order is highest count first, entry order among equal counts.
        assert_eq!(found, want);
        assert_eq!(more_kinds(&decoded), Some(1));
    }

    #[test]
    fn many_utf8_identifiers_remain_independent_warnings_with_every_reopened_page() {
        let names: Vec<String> =
            (0..200).map(|index| alloc::format!("{}{:04}", "税".repeat(76), index)).collect();
        assert!(names.iter().all(|name| name.chars().count() == 80 && name.len() > 80));
        let pruned = Pruned {
            removed: names.iter().map(|name| row("SAVING_KIND_W2", name)).collect(),
            reopened: SEQUENCE.to_vec(),
        };
        let warnings = save_warnings(&pruned);
        assert_eq!(warnings.len(), 1 + names.len() + SEQUENCE.len());
        // Round-trip the actual response collection, including every independent
        // tuple: a joined long-name list would instead exceed the nested cap.
        let mut wire = Vec::new();
        warnings.avro_encode(&mut wire);
        let mut offset = 0;
        let restored = Vec::<SaveWarning>::avro_decode(&wire, &mut offset).unwrap();
        assert_eq!(offset, wire.len());
        assert_eq!(restored, warnings);
        let tuples: Vec<_> = restored.iter().map(|w| decoded_tuple(&w.message, 1)).collect();
        assert_eq!(tuples[0].0, "SAVING_REMOVES_COUNT");
        assert_eq!(tuples[0].1[0], A::Plural(names.len() as i64));
        for (tuple, name) in tuples[1..=names.len()].iter().zip(&names) {
            assert_eq!(tuple.0, "SAVING_REMOVES_NAMED");
            assert_eq!(
                tuple.1,
                vec![A::ResourceText("SAVING_KIND_W2_ONE".into()), A::LiteralText(name.clone())]
            );
        }
        for (tuple, section) in tuples[1 + names.len()..].iter().zip(SEQUENCE) {
            assert_eq!(tuple.0, "SAVING_REOPENS");
            assert_eq!(tuple.1, vec![A::Signed(section as i64)]);
        }
    }
}
