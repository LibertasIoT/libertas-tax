use super::*;
use crate::document_name_matcher::{
    JOINT_RECIPIENT, RecipientDomain, eligible_recipients, match_eligible_recipient,
};
use crate::document_names::{AcceptedPartySource, Assignment};
use crate::{
    AddedOwnership, DocumentOwnerSelection, DocumentOwnershipDerived, DocumentRecipient,
    OwnerSource, PersonDocuments, PrintedDocumentNames, SectionDocument,
};
use alloc::string::String;

// Every document entry whose printed recipient the App assigns to a person.
// `section` is the document's tax topic, independent of where its owner is assigned.
struct NamedDocument {
    id: i64,
    domain: RecipientDomain,
    kind: Vec<u8>,
    details: Vec<Message>,
    issuer: String,
    names: PrintedDocumentNames,
    section: TaxSection,
    label: crate::Message,
    // Whether the stored owner is a person this return can have as the
    // document's owner. An income entry's owner is the entry's own; any other
    // document's is its stored recipient. An owner the return no longer has
    // (a spouse after a change to a single return) is Unknown again.
    owned: bool,
    // Whether any owner is stored. Only an Unknown document holds the
    // estimate; an owner the return no longer has is the estimate's own issue.
    known: bool,
    // An income entry's own stored owner.
    entry_owner: Option<crate::DocumentOwner>,
}

impl NamedDocument {
    fn summary(&self, draft: &FederalDraft, people: &[crate::PersonChoice]) -> Vec<u8> {
        let owner = match self.names.assigned {
            DocumentRecipient::Joint => status("PERSON_YOU_AND_SPOUSE"),
            owner => people
                .iter()
                .find(|person| person.id == owner.key())
                .map_or_else(|| status("DOCUMENT_OWNER_UNASSIGNED"), |person| person.label.clone()),
        };
        let mut parts = alloc::vec![Message::new(
            "DOCUMENT_REFERENCE_OWNER",
            alloc::vec![crate::MessageArg::Encoded(self.name()), crate::MessageArg::Encoded(owner),]
        )];
        parts.extend(self.details.iter().cloned());
        if self.details.is_empty()
            && let Some(amount) = draft
                .income
                .iter()
                .find(|entry| entry.id == self.id)
                .and_then(|entry| preview_amount(draft, entry))
        {
            parts.push(Message::new(
                "DOCUMENT_RETURN_AMOUNT",
                alloc::vec![crate::MessageArg::usd_cents(amount)],
            ));
        }
        message_list(&parts).encode()
    }

    fn name(&self) -> Vec<u8> {
        let kind = || crate::MessageArg::Encoded(self.kind.clone());
        let issuer = || crate::MessageArg::Text(self.issuer.clone());
        let recipient = || crate::MessageArg::Text(self.names.recipient.name.clone());
        match (self.issuer.trim().is_empty(), self.names.recipient.name.trim().is_empty()) {
            (false, false) => {
                Message::new("DOCUMENT_CHOICE", alloc::vec![kind(), issuer(), recipient()]).encode()
            }
            (false, true) => Message::new("DOCUMENT_FROM", alloc::vec![kind(), issuer()]).encode(),
            (true, false) => {
                Message::new("DOCUMENT_PRINTED_FOR", alloc::vec![kind(), recipient()]).encode()
            }
            (true, true) => self.kind.clone(),
        }
    }
}

fn printed_boxes<T: Copy + Into<Option<i64>>>(boxes: &[(&str, T)]) -> Vec<Message> {
    boxes
        .iter()
        .filter_map(|(number, amount)| {
            (*amount).into().map(|amount| {
                Message::new(
                    "DOCUMENT_PRINTED_BOX",
                    alloc::vec![
                        crate::MessageArg::Text((*number).into()),
                        crate::MessageArg::usd_cents(amount),
                    ],
                )
            })
        })
        .collect()
}

pub(super) fn document_name(draft: &FederalDraft, id: i64) -> Option<Vec<u8>> {
    named_documents(draft)
        .into_iter()
        .find(|document| document.id == id)
        .map(|document| document.name())
}

fn owned(
    draft: &FederalDraft,
    domain: RecipientDomain,
    names: &PrintedDocumentNames,
    owner: Option<crate::DocumentOwner>,
) -> bool {
    let key = match (names.assigned.is_known(), owner) {
        (true, _) => names.assigned.key(),
        (false, Some(crate::DocumentOwner::Yours)) => -1,
        (false, Some(crate::DocumentOwner::Spouses)) => -2,
        (false, _) => return false,
    };
    owner != Some(crate::DocumentOwner::Unknown)
        && eligible_recipients(draft, domain).contains(&key)
}

fn income_domain(entry: &FederalIncomeEntry) -> RecipientDomain {
    document_item(entry, &[], false)
        .map_or(RecipientDomain::Earner, |row| row.document.recipient_domain())
}

// The topic page that shows the document among its accepted documents, by the
// Document Center category the document belongs to.
fn income_section(draft: &FederalDraft, entry: &FederalIncomeEntry) -> TaxSection {
    let category = document_item(entry, &[], false).and_then(|row| match row.document {
        IncomeSource::Wage { .. } => Some(TaxSection::Wages),
        IncomeSource::InterestDividend { .. } => Some(TaxSection::InterestDividends),
        IncomeSource::InvestmentSale { .. } => Some(TaxSection::InvestmentSales),
        IncomeSource::Retirement { .. } => Some(TaxSection::Retirement),
        IncomeSource::SelfEmployment { .. } => Some(TaxSection::SelfEmployment),
        IncomeSource::GovernmentPayment { .. } => Some(TaxSection::GovernmentBenefits),
        IncomeSource::Gambling(_) => Some(TaxSection::Gambling),
        IncomeSource::K1 { .. } => Some(TaxSection::PassThrough),
        IncomeSource::Farming(_) => Some(TaxSection::Farming),
        IncomeSource::OtherIncome { .. } => Some(TaxSection::OtherIncome),
        IncomeSource::CanceledDebt { .. } => Some(TaxSection::CanceledDebt),
        IncomeSource::Record { .. } => None,
    });
    category.unwrap_or_else(|| {
        INCOME_TOPICS
            .iter()
            .copied()
            .find(|topic| source_in_editor(draft, entry, *topic))
            .unwrap_or(TaxSection::Documents)
    })
}

fn named_documents(draft: &FederalDraft) -> Vec<NamedDocument> {
    let mut rows = Vec::new();
    for entry in &draft.income {
        if let Some(names) = entry.accepted_parties() {
            let issuer = names
                .issuer
                .as_ref()
                .map(|party| party.name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| entry.label.clone());
            let domain = income_domain(entry);
            let kind = document_kind_keys(entry).0;
            let kind = if kind.starts_with("SAVING_KIND_") {
                alloc::format!("{kind}_ONE")
            } else {
                kind.into()
            };
            rows.push(NamedDocument {
                owned: owned(draft, domain, &names, Some(entry.owner)),
                known: entry.owner != crate::DocumentOwner::Unknown,
                entry_owner: Some(entry.owner),
                id: entry.id,
                domain,
                kind: Message::new(kind, Vec::new()).encode(),
                details: match &entry.income {
                    FederalIncome::Wage { data: crate::WageSource::W2(source) } => {
                        printed_boxes(&[("1", source.form.wages), ("2", source.form.withholding)])
                    }
                    _ => Vec::new(),
                },
                issuer,
                names,
                section: income_section(draft, entry),
                label: income_display_label(entry, &draft.dependents),
            });
        }
    }
    macro_rules! append {
        ($sources:expr, $domain:ident, $kind:literal, $section:ident, $source:ident => $details:expr) => {
            for $source in $sources {
                let source = $source;
                if let Some(names) = source.accepted_parties() {
                    let issuer =
                        names.issuer.as_ref().map(|party| party.name.clone()).unwrap_or_default();
                    rows.push(NamedDocument {
                        owned: owned(draft, RecipientDomain::$domain, &names, None),
                        known: names.assigned.is_known(),
                        entry_owner: None,
                        id: source.id,
                        domain: RecipientDomain::$domain,
                        kind: Message::new($kind, Vec::new()).encode(),
                        details: $details,
                        label: Message::new(
                            "DOCUMENT_FROM",
                            alloc::vec![
                                crate::MessageArg::Encoded(
                                    Message::new($kind, Vec::new()).encode()
                                ),
                                record_display_arg(&issuer, source.id),
                            ],
                        ),
                        issuer,
                        names,
                        section: TaxSection::$section,
                    });
                }
            }
        };
    }
    if let Some(value) = &draft.deductions {
        append!(&value.mortgage, Account, "FORM_FORM1098", Mortgage, source => printed_boxes(&[("1", source.form.box_1), ("2", source.form.box_2)]));
        append!(&value.vehicle_gift_documents, Earner, "FORM_FORM1098C", GiftCarryovers, source => printed_boxes(&[("4c", source.form.box_4c), ("6b", source.form.box_6b)]));
        append!(&value.vehicle_loans, Earner, "FORM_FORM1098VLI", Deductions, source => printed_boxes(&[("1", source.document.box_1)]));
    }
    if let Some(value) = &draft.adjustments {
        append!(&value.health_statements, Earner, "FORM_FORM5498SA", Hsa, source => printed_boxes(&[("2", source.form.total_contributions), ("4", source.form.rollover_contributions)]));
        append!(&value.student_loan.forms, Earner, "FORM_FORM1098E", StudentLoan, source => printed_boxes(&[("1", source.document.box_1)]));
    }
    if let Some(value) = &draft.credits {
        append!(&value.education, Student, "FORM_FORM1098T", Education, source => printed_boxes(&[("1", source.form.box_1), ("5", source.form.box_5)]));
        append!(&value.tax_credit_bonds, Earner, "FORM_FORM1097BTC", MoreCredits, source => printed_boxes(&[("1", source.form.box1_credit)]));
        append!(value.bba_audit_interest.iter(), Earner, "FORM_FORM8986", MoreCredits, source => alloc::vec![Message::new("DOCUMENT_PRINTED_ADJUSTMENT", alloc::vec![crate::MessageArg::usd_cents(source.document.interest_adjustment())]), Message::new("DOCUMENT_PRINTED_ORIGINAL_INTEREST", alloc::vec![crate::MessageArg::usd_cents(source.document.original_partnership_interest())])]);
        if let Some(facts) = crate::marketplace::facts(&value.marketplace) {
            append!(&facts.forms, Earner, "FORM_FORM1095A", Marketplace, source => printed_boxes(&[("33 A", source.form.annual_totals.premium), ("33 C", source.form.annual_totals.advance)]));
        }
    }
    if let Some(value) = &draft.screening {
        append!(&value.espp_acquisitions, Earner, "FORM_FORM3922", InvestmentSales, source => printed_boxes(&[("3", source.form.grant_market), ("4", source.form.market_price), ("5", source.form.exercise_price)]));
        append!(&value.iso_exercises, Earner, "FORM_FORM3921", InvestmentSales, source => printed_boxes(&[("3", source.form.exercise_price), ("4", source.form.market_price)]));
    }
    rows
}

// The App's assignment at entry, the same for AI Import and a person typing
// the paper. A printed name that matches a person owns the document (locked);
// otherwise an unchanged printed name keeps its accepted owner while that person
// is still eligible; anything else is Unknown.
pub(super) fn entry_assignment(
    draft: &FederalDraft,
    before: Option<&PrintedDocumentNames>,
    before_owner: Option<crate::DocumentOwner>,
    names: &PrintedDocumentNames,
    domain: RecipientDomain,
) -> Assignment {
    let unchanged = before.filter(|before| before.recipient.name == names.recipient.name);
    if let Some(owner) = match_eligible_recipient(draft, &names.recipient.name, domain)
        .and_then(DocumentRecipient::from_key)
    {
        return match unchanged.map(PrintedDocumentNames::assignment) {
            Some(kept)
                if kept.owner == owner && kept.owner_source == Some(OwnerSource::Matched) =>
            {
                kept
            }
            _ => Assignment::matched(owner),
        };
    }
    let eligible = eligible_recipients(draft, domain);
    unchanged
        .map(|before| match (before.assignment(), before_owner) {
            // An income entry's stored owner stands for an entry without a
            // recipient assignment.
            (kept, Some(owner)) if !kept.owner.is_known() => match owner.tax_owner() {
                Some(crate::Owner::Taxpayer) => {
                    Assignment::added(DocumentRecipient::Taxpayer, None)
                }
                Some(crate::Owner::Spouse) => Assignment::added(DocumentRecipient::Spouse, None),
                None => kept,
            },
            (kept, _) => kept,
        })
        .filter(|kept| kept.owner.is_known() && eligible.contains(&kept.owner.key()))
        .unwrap_or_default()
}

// Store an assignment on the document entry, with the owner values the
// calculation reads derived from it.
fn assign(draft: &mut FederalDraft, id: i64, assignment: Assignment) -> bool {
    let owner = assignment.owner;
    macro_rules! assign {
        ($sources:expr, $apply:expr) => {
            for source in $sources {
                if source.id == id && source.set_document_recipient(assignment) {
                    ($apply)(source);
                    return true;
                }
            }
        };
    }
    for source in &mut draft.income {
        if source.id == id && source.set_document_recipient(assignment) {
            source.owner = owner.document_owner();
            return true;
        }
    }
    if let Some(value) = draft.deductions.as_mut() {
        assign!(&mut value.mortgage, |_source: &mut crate::Mortgage1098Source| {});
        assign!(&mut value.vehicle_gift_documents, |_source: &mut crate::Form1098CDocument| {});
        assign!(&mut value.vehicle_loans, |_source: &mut crate::VehicleLoanSource| {});
    }
    if let Some(value) = draft.adjustments.as_mut() {
        assign!(&mut value.health_statements, |_source: &mut crate::HealthStatementSource| {});
        assign!(&mut value.student_loan.forms, |_source: &mut crate::StudentLoanSource| {});
    }
    if let Some(value) = draft.credits.as_mut() {
        assign!(&mut value.education, |source: &mut crate::EducationSource| {
            source.student = owner.key();
        });
        assign!(&mut value.tax_credit_bonds, |source: &mut crate::FederalTaxCreditBond1097| {
            source.owner = owner.document_owner().owned();
        });
        assign!(value.bba_audit_interest.iter_mut(), |_source: &mut crate::BbaAuditSource| {});
        if let crate::MarketplaceClaim::Yes { facts } = &mut value.marketplace {
            assign!(&mut facts.forms, |_source: &mut crate::MarketplacePolicyRecord| {});
        }
    }
    if let Some(value) = draft.screening.as_mut() {
        assign!(&mut value.espp_acquisitions, |source: &mut crate::EsppAcquisition| {
            source.owner = owner.document_owner().owned();
        });
        assign!(&mut value.iso_exercises, |source: &mut crate::IsoExerciseSource| {
            source.owner = owner.document_owner().owned();
        });
    }
    false
}

// Document Center save: every named document gets the App's assignment. Echoed
// hidden owners never decide it; eligibility is recomputed from this return.
pub(super) fn match_documents(draft: &mut FederalDraft, accepted: &FederalDraft) {
    let previous = named_documents(accepted);
    for document in named_documents(draft) {
        let before = previous.iter().find(|before| before.id == document.id);
        // An accepted income entry without printed names had an empty printed
        // recipient and its stored owner.
        let unnamed = accepted.income.iter().find(|entry| {
            before.is_none() && entry.id == document.id && document.entry_owner.is_some()
        });
        let empty = PrintedDocumentNames::default();
        let assignment = entry_assignment(
            draft,
            before.map(|before| &before.names).or(unnamed.map(|_| &empty)),
            before.and_then(|before| before.entry_owner).or(unnamed.map(|entry| entry.owner)),
            &document.names,
            document.domain,
        );
        assign(draft, document.id, assignment);
    }
    if let Some(credits) = draft.credits.as_mut() {
        credits.reconcile_education_students();
        credits.reconcile_education_insurance_sources(
            accepted.credits.as_ref().map_or(&[], |credits| credits.education.as_slice()),
        );
    }
}

// Each document without an owner is a pending item in the Document center;
// the estimate waits for it ("More answers needed").
pub(crate) fn unknown_owner_issues(draft: &FederalDraft) -> Vec<crate::IssueContext> {
    named_documents(draft)
        .into_iter()
        .filter(|document| !document.known)
        .map(|document| {
            if draft.income.iter().any(|entry| entry.id == document.id) {
                crate::IssueContext::Income { id: document.id, label: document.label }
            } else {
                crate::IssueContext::Record {
                    section: document.section,
                    identity: crate::federal_model::NoteIdentity::Income(document.id),
                    label: document.label,
                }
            }
            .corrective(TaxSection::Documents)
        })
        .collect()
}

// This person's part of a joint document, in percent.
fn person_share(person: i64, assignment: Assignment) -> Option<i32> {
    let taxpayer = assignment.joint_share.unwrap_or(50);
    (assignment.owner == DocumentRecipient::Joint).then_some(if person == -2 {
        100 - taxpayer
    } else {
        taxpayer
    })
}

fn owned_by(person: i64, owner: DocumentRecipient) -> bool {
    owner.key() == person || (owner == DocumentRecipient::Joint && matches!(person, -1 | -2))
}

fn document_ownership(
    draft: &FederalDraft,
    person: i64,
    document: &NamedDocument,
) -> DocumentOwnershipDerived {
    let assignment = document.names.assignment();
    let eligible = eligible_recipients(draft, document.domain);
    if !eligible.contains(&person) || (document.owned && !owned_by(person, assignment.owner)) {
        return DocumentOwnershipDerived {
            allowed_ownership: Vec::new(),
            joint_share_needed: None,
        };
    }
    let yours = libertas_macros::variant_index!(AddedOwnership::Yours) as i32;
    let joint = libertas_macros::variant_index!(AddedOwnership::Joint) as i32;
    let allowed_ownership =
        if document.owned && assignment.owner_source == Some(OwnerSource::Matched) {
            alloc::vec![if assignment.owner == DocumentRecipient::Joint { joint } else { yours }]
        } else if matches!(person, -1 | -2) && eligible.contains(&JOINT_RECIPIENT) {
            alloc::vec![yours, joint]
        } else {
            alloc::vec![yours]
        };
    DocumentOwnershipDerived {
        allowed_ownership,
        joint_share_needed: Some(PersonDocuments::needs_joint_share(draft, document.section)),
    }
}

// Positional projection preserves the current editor's document order.
pub(super) fn ownership_choices(
    draft: &FederalDraft,
    person: i64,
    selections: &[DocumentOwnerSelection],
) -> Vec<DocumentOwnershipDerived> {
    let documents = named_documents(draft);
    selections
        .iter()
        .map(|selection| {
            documents.iter().find(|document| document.id == selection.document).map_or_else(
                || DocumentOwnershipDerived {
                    allowed_ownership: if selection.document == 0 {
                        alloc::vec![libertas_macros::variant_index!(AddedOwnership::Yours) as i32]
                    } else {
                        Vec::new()
                    },
                    joint_share_needed: None,
                },
                |document| document_ownership(draft, person, document),
            )
        })
        .collect()
}

// The person sections of `section`: derived from the stored owners on every
// visit, never stored. A section without named documents shows none.
pub(super) fn section_view(draft: &FederalDraft, section: TaxSection) -> Vec<PersonDocuments> {
    let documents: Vec<_> = named_documents(draft)
        .into_iter()
        .filter(|document| section == TaxSection::Documents || document.section == section)
        .collect();
    if documents.is_empty() {
        return Vec::new();
    }
    let mut people = crate::federal_rules::people_students(draft);
    // A community-property separate return reports the spouse's documents
    // without the joint return's spouse page.
    if !people.iter().any(|person| person.id == -2)
        && documents
            .iter()
            .any(|document| eligible_recipients(draft, document.domain).contains(&-2))
    {
        people.insert(
            1,
            crate::PersonChoice { id: -2, label: status("PERSON_SPOUSE"), valid_ssn: false },
        );
    }
    let choices: Vec<_> = documents
        .iter()
        .map(|document| crate::RecordChoice {
            id: document.id,
            label: document.summary(draft, &people),
        })
        .collect();
    use crate::kernel::ratio::exact;
    let totals: alloc::collections::BTreeMap<_, _> = people
        .iter()
        .map(|person| {
            let person = &person.id;
            let total = {
                let entries: Vec<_> = draft
                    .income
                    .iter()
                    .filter(|entry| {
                        documents.iter().any(|document| document.id == entry.id)
                            && owned_by(*person, entry.accepted_assignment().owner)
                    })
                    .collect();
                let total = entries.iter().try_fold(exact::whole(0), |total, entry| {
                    let amount = preview_amount(draft, entry)?;
                    let amount = if draft
                        .setup
                        .as_ref()
                        .is_some_and(|setup| setup.filing_status() == FederalStatus::Separate)
                    {
                        let share = draft
                            .setup
                            .as_ref()
                            .and_then(|setup| crate::community::entry_share(setup, entry));
                        let share = share
                            .map(|share| if *person == -2 { share.counterpart() } else { share });
                        exact::whole(allocated_preview_amount(entry, amount, share, true)?)
                    } else if matches!(*person, -1 | -2) {
                        let (weights, denominator) = entry.accepted_assignment().owner_weights()?;
                        exact::whole(amount)
                            * exact::ratio(weights[usize::from(*person == -2)], denominator)?
                    } else {
                        exact::whole(amount)
                    };
                    Some(total + amount)
                });
                total.filter(|_| !entries.is_empty())
            };
            (*person, total)
        })
        .collect();
    let mut income_shares: alloc::collections::BTreeMap<_, _> = totals
        .iter()
        .map(|(person, total)| (*person, total.as_ref().and_then(exact::rounded)))
        .collect();
    if let (Some(Some(taxpayer)), Some(Some(spouse))) = (totals.get(&-1), totals.get(&-2)) {
        let pair =
            exact::project_whole_pair(&(taxpayer + spouse), [taxpayer.clone(), spouse.clone()], 0);
        income_shares.insert(-1, pair.map(|pair| pair[0]));
        income_shares.insert(-2, pair.map(|pair| pair[1]));
    }
    people
        .into_iter()
        .filter(|person| {
            documents
                .iter()
                .any(|document| eligible_recipients(draft, document.domain).contains(&person.id))
        })
        .map(|person| {
            let mut view = PersonDocuments {
                person: person.id,
                name: person.label,
                documents: Vec::new(),
                choices: choices.clone(),
                available_documents: Vec::new(),
                income_share: income_shares.get(&person.id).copied().flatten(),
            };
            for document in &documents {
                let eligible = eligible_recipients(draft, document.domain);
                if !eligible.contains(&person.id) {
                    continue;
                }
                let assignment = document.names.assignment();
                let mine = owned_by(person.id, assignment.owner);
                // Printed ownership stays locked; a consequential joint share
                // can change independently. Unknown documents can be added.
                if !document.owned || mine {
                    view.available_documents.push(document.id);
                }
                if document.owned
                    && mine
                    && let Some(owner_source) = assignment.owner_source
                {
                    let derived = document_ownership(draft, person.id, document);
                    view.documents.push(SectionDocument {
                        document: document.id,
                        edit_document: Vec::new(),
                        ownership: match person_share(person.id, assignment) {
                            Some(share) => AddedOwnership::Joint { share },
                            None => AddedOwnership::Yours,
                        },
                        owner_source,
                        allowed_ownership: derived.allowed_ownership,
                        joint_share_needed: derived.joint_share_needed,
                    });
                }
            }
            view
        })
        .filter(|view| !view.documents.is_empty() || !view.available_documents.is_empty())
        .collect()
}

// Save: one-way, exhaustive mapping from the person sections onto the stored
// owners. A new row sets the owner (this person or Joint) as Added with its
// share; a changed row updates it; a removed row returns the document to
// Unknown. Matched ownership stays fixed while consequential shares can change.
pub(crate) fn apply_section_view(
    draft: &mut FederalDraft,
    section: TaxSection,
    view: &[PersonDocuments],
    income_owners: &[i32],
) -> Result<(), SaveFailure> {
    let documents: Vec<_> = named_documents(draft)
        .into_iter()
        .filter(|document| section == TaxSection::Documents || document.section == section)
        .collect();
    let at = |person: usize, row: usize, field: &'static str, key: &'static str| {
        SaveFailure::from(
            ValidationIssue::new(key)
                .at_field(field)
                .in_list("documents", row)
                .in_list("document_owners", person),
        )
    };
    // Every row, with the owner it asks for and this person's share.
    let mut wanted: Vec<(i64, DocumentRecipient, Option<i32>, usize, usize)> = Vec::new();
    for (p, person) in view.iter().enumerate() {
        for (r, row) in person.documents.iter().enumerate() {
            let Some(document) = documents.iter().find(|document| document.id == row.document)
            else {
                return Err(at(p, r, "document", "DOCUMENT_NOT_AVAILABLE_TO_ADD"));
            };
            let assignment = document.names.assignment();
            let matched = document.owned && assignment.owner_source == Some(OwnerSource::Matched);
            let eligible = eligible_recipients(draft, document.domain);
            let addable = !document.owned
                || ((matched || assignment.owner_source == Some(OwnerSource::Added))
                    && owned_by(person.person, assignment.owner));
            if !eligible.contains(&person.person) || !addable {
                return Err(at(p, r, "document", "DOCUMENT_NOT_AVAILABLE_TO_ADD"));
            }
            let (owner, share) = match row.ownership {
                AddedOwnership::Yours => match DocumentRecipient::from_key(person.person) {
                    Some(owner) => (owner, None),
                    None => return Err(at(p, r, "document", "DOCUMENT_NOT_AVAILABLE_TO_ADD")),
                },
                AddedOwnership::Joint { share } => {
                    if !matches!(person.person, -1 | -2) || !eligible.contains(&JOINT_RECIPIENT) {
                        return Err(at(p, r, "ownership", "DOCUMENT_JOINT_NOT_AVAILABLE"));
                    }
                    let taxpayer = if PersonDocuments::needs_joint_share(draft, document.section) {
                        if person.person == -2 { 100 - share } else { share }
                    } else {
                        assignment.joint_share.unwrap_or(50)
                    };
                    (DocumentRecipient::Joint, Some(taxpayer))
                }
            };
            if matched && owner != assignment.owner {
                return Err(at(p, r, "ownership", "DOCUMENT_NAMES_CHANGED"));
            }
            if wanted.iter().any(|(id, before, _, q, _)| {
                *id == row.document
                    && (*q == p || *before != owner || owner != DocumentRecipient::Joint)
            }) {
                return Err(at(p, r, "document", "DOCUMENT_ADDED_TWICE"));
            }
            wanted.push((row.document, owner, share, p, r));
        }
    }
    for document in &documents {
        let stored = document.names.assignment();
        let rows: Vec<_> = wanted.iter().filter(|row| row.0 == document.id).collect();
        let mut next = match rows.as_slice() {
            [] if stored.owner_source == Some(OwnerSource::Added) => Assignment::default(),
            [] => continue,
            // A joint document added earlier is shown in both spouses'
            // sections; removing it from either returns it to Unknown.
            [(_, DocumentRecipient::Joint, ..)]
                if stored.owner == DocumentRecipient::Joint
                    && stored.owner_source == Some(OwnerSource::Added)
                    && view.iter().filter(|person| matches!(person.person, -1 | -2)).count()
                        == 2 =>
            {
                Assignment::default()
            }
            [(_, owner, share, ..)] => Assignment::added(*owner, *share),
            [first, second] => {
                // The unchanged echo keeps the stored share; the edited row wins.
                let share = [first.2, second.2]
                    .into_iter()
                    .find(|share| *share != stored.joint_share)
                    .unwrap_or(first.2);
                if first.2 != stored.joint_share
                    && second.2 != stored.joint_share
                    && first.2 != second.2
                {
                    return Err(at(
                        second.3,
                        second.4,
                        "ownership",
                        "DOCUMENT_JOINT_SHARES_DIFFER",
                    ));
                }
                Assignment::added(DocumentRecipient::Joint, share)
            }
            [.., last] => return Err(at(last.3, last.4, "document", "DOCUMENT_ADDED_TWICE")),
        };
        if stored.owner_source == Some(OwnerSource::Matched) {
            next.owner_source = Some(OwnerSource::Matched);
        }
        if next == stored {
            continue;
        }
        assign(draft, document.id, next);
        if let Some(entry) = draft.income.iter().find(|entry| entry.id == document.id) {
            let (p, r) = rows.first().map_or((0, 0), |row| (row.3, row.4));
            if let Some(owner) = entry.owner.tax_owner()
                && !income_owners.contains(&(owner as i32))
            {
                return Err(at(
                    p,
                    r,
                    "ownership",
                    "ISSUE_SPOUSE_OWNED_INCOME_NEEDS_A_JOINT_OR_COMMUNITY_PROPERTY_RETURN",
                ));
            }
            if let Some(issue) = recorded_income_owner_issue(draft, entry) {
                return Err(SaveFailure {
                    message: Message::new(issue.key, alloc::vec![income_display_arg(entry)])
                        .encode(),
                    field: ValidationIssue::new(issue.key)
                        .at_field("ownership")
                        .in_list("documents", r)
                        .in_list("document_owners", p)
                        .field,
                });
            }
        }
    }
    if let Some(credits) = draft.credits.as_mut() {
        credits.reconcile_education_students();
    }
    Ok(())
}

impl Interview {
    pub(super) fn document_owners(&self, section: TaxSection) -> Vec<PersonDocuments> {
        section_view(&self.draft, section)
    }
    // A page's person sections apply to its candidate before it is committed.
    pub(super) fn with_document_owners(
        &self,
        mut draft: FederalDraft,
        section: TaxSection,
        view: &[PersonDocuments],
    ) -> Result<FederalDraft, SaveFailure> {
        apply_section_view(&mut draft, section, view, &self.income_owners())?;
        if let Some(credits) = draft.credits.as_mut() {
            credits.reconcile_education_insurance_sources(
                self.draft.credits.as_ref().map_or(&[], |previous| previous.education.as_slice()),
            );
        }
        Ok(draft)
    }
}

pub(crate) fn assign_native_document_ids(
    draft: &mut FederalDraft,
    accepted: &FederalDraft,
) -> Result<(), SaveFailure> {
    macro_rules! assign {
        ($sources:expr, $accepted:expr, $group:literal, $category:literal, $list:literal) => {
            assign_record_ids(
                ($sources).iter_mut().map(|source| &mut source.id),
                ($accepted).map(|source| source.id),
                &mut draft.next_id,
            )
            .map_err(|(key, _, index)| {
                SaveFailure::from(
                    ValidationIssue::new(key)
                        .at_field("id")
                        .in_list($list, index)
                        .in_field($category)
                        .in_field($group)
                        .in_field("documents"),
                )
            })?;
        };
    }
    if let Some(value) = draft.adjustments.as_mut() {
        assign!(
            &mut value.health_statements,
            accepted.adjustments.iter().flat_map(|v| v.health_statements.iter()),
            "common",
            "health_accounts",
            "documents"
        );
        assign!(
            &mut value.student_loan.forms,
            accepted.adjustments.iter().flat_map(|v| v.student_loan.forms.iter()),
            "common",
            "student_loans",
            "documents"
        );
    }
    if let Some(value) = draft.deductions.as_mut() {
        assign!(
            &mut value.vehicle_gift_documents,
            accepted.deductions.iter().flat_map(|v| v.vehicle_gift_documents.iter()),
            "other",
            "vehicle_gifts",
            "documents"
        );
        assign!(
            &mut value.vehicle_loans,
            accepted.deductions.iter().flat_map(|v| v.vehicle_loans.iter()),
            "other",
            "vehicle_loans",
            "documents"
        );
    }
    if let Some(value) = draft.credits.as_mut() {
        assign!(
            &mut value.education,
            accepted.credits.iter().flat_map(|v| v.education.iter()),
            "common",
            "education",
            "students"
        );
        assign!(
            &mut value.tax_credit_bonds,
            accepted.credits.iter().flat_map(|v| v.tax_credit_bonds.iter()),
            "other",
            "credits",
            "documents"
        );
        assign!(
            &mut value.bba_audit_interest,
            accepted.credits.iter().flat_map(|v| v.bba_audit_interest.iter()),
            "other",
            "credits",
            "documents"
        );
    }
    if let Some(value) = draft.screening.as_mut() {
        assign!(
            &mut value.espp_acquisitions,
            accepted.screening.iter().flat_map(|v| v.espp_acquisitions.iter()),
            "common",
            "investment_sales",
            "documents"
        );
        assign!(
            &mut value.iso_exercises,
            accepted.screening.iter().flat_map(|v| v.iso_exercises.iter()),
            "common",
            "investment_sales",
            "documents"
        );
    }
    Ok(())
}
