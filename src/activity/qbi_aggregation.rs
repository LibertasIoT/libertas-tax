// Coverage: section199A aggregation election and disclosure source facts.
//   Source membership stays on the actual income documents; this module
//   projects genuine catalogs and validates accepted election source records.
//   Financial amounts and loss eligibility remain with their existing owners.
//   Bounded source increment: actual single ordinary-business S-corporation
//   and non-PTP partnership sources join the existing group wage/property limit
//   once, after their current qualification and loss-limit owners. Global
//   election/disclosure facts and transient typed history associations govern
//   readiness; retained/unknown facts are never a guessed favorable election.
//   Tests: tests/cross/interview/qbi_aggregation.rs independent 5000->25000 wage-limit
//   control, mixed one-business K1s, native Save atomicity, history corrections,
//   inactive disclosure catalogs and normal source-ID/reopen/resave/restart.
//   Native activity K-1 blocks retain their own accepted identities and actual
//   named-business counts. Current one-business/component blocks reuse the
//   same election/disclosure owner; unsupported financial allocations remain
//   unavailable. Estate adapters and broader QBI residuals remain outside this
//   increment; no parent/activity count substitutes for distinct businesses.
use crate::{CommunityShare, Owner, RefundManagement};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

use crate::qbi_aggregation_model::text_valid;
pub use crate::qbi_aggregation_model::{
    QbiAggregationFacts, QbiAggregationSource, QbiBusinessChanges, QbiBusinessDisclosure,
    QbiElectionHistory, QbiEntityEin, QbiIrsDisaggregationHistory,
};

/// Source classification reconstructed from the actual candidate
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum QbiSourceOrigin {
    /// Direct business
    DirectBusiness,
    /// Direct rental business
    DirectRental,
    /// Partnership business
    PartnershipBusiness,
    /// S corporation business
    ScorpBusiness,
    /// Estate or trust QBI business statement
    EstateBusiness,
    /// Whole aggregate reported by an RPE; its components cannot be selected separately
    ReportedRpeAggregate {
        /// Reported aggregate name
        #[libertas_size(min = 1)]
        name: String,
    },
}
/// Genuine source member supplied by the service
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiAggregationMemberChoice {
    /// Canonical transient picker key derived from the actual typed source reference
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_size(min = 1)]
    pub key: String,
    /// Actual accepted business or statement/block reference
    #[libertas_hidden]
    #[libertas_read_only]
    pub source: QbiAggregationSource,
    /// Source
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Actual source's selected individual aggregation label
    /// [NullText]
    /// No individual group is selected for this source
    /// [SetText]
    /// This source is selected in individual group
    #[libertas_read_only]
    #[libertas_size(min = 1)]
    pub group: Option<String>,
    /// Actual source owner
    #[libertas_read_only]
    pub owner: Owner,
    /// Established SSTB classification
    /// [NullText]
    /// SSTB classification is not established
    /// [SetText]
    /// The source SSTB classification is
    #[libertas_read_only]
    pub service: Option<bool>,
    /// Actual source kind or reported indivisible RPE block
    #[libertas_read_only]
    pub origin: QbiSourceOrigin,
}
/// Editable facts for the actual group
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiAggregationFactsPage {
    /// Group name
    #[libertas_size(min = 1)]
    pub name: String,
    /// The same person or group meets the 50% common-ownership test
    /// Check the businesses’ ownership, operating and tax-year records.
    pub common_ownership: bool,
    /// That ownership exists for most of the year including its last day
    /// Check the businesses’ ownership, operating and tax-year records.
    pub majority_year_last_day: bool,
    /// The businesses report the same tax year, ignoring short years
    /// Check the businesses’ ownership, operating and tax-year records.
    pub same_tax_year: bool,
    /// Products or services are the same or customarily offered together
    /// Check the businesses’ ownership, operating and tax-year records.
    pub same_or_customary_products: bool,
    /// The businesses share facilities or significant centralized elements
    /// Check the businesses’ ownership, operating and tax-year records.
    pub shared_facilities_or_elements: bool,
    /// The businesses operate in coordination or reliance on one another
    /// Check the businesses’ ownership, operating and tax-year records.
    pub coordinated_or_interdependent: bool,
    /// Election and prior-year facts
    pub election: QbiElectionHistory,
    /// Retained IRS restriction for the actual businesses, independent of group label
    pub irs_history: QbiIrsDisaggregationHistory,
    /// Each actual source in the group
    #[libertas_size(min = 0)]
    pub disclosure: Vec<QbiBusinessDisclosurePage>,
}
/// Annual disclosure source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiBusinessDisclosurePage {
    /// Actual source from this group's genuine choices
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_size(min = 1)]
    #[libertas_enum_source("^<QbiAggregationPage>.disclosure_sources")]
    pub source: String,
    /// Operating entity or business name
    #[libertas_size(min = 1)]
    pub entity_name: String,
    /// Actual EIN applicability or unavailable source information
    pub ein: QbiEntityEin,
    /// Business description
    #[libertas_size(min = 1)]
    pub business_description: String,
    /// Changes during 2026
    pub changes: QbiBusinessChanges,
}
/// Aggregation facts and genuine service-supplied members
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiAggregationPage {
    /// Eligibility, election and annual disclosure
    pub facts: QbiAggregationFactsPage,
    /// Actual members reconstructed from source group labels
    #[libertas_read_only]
    #[libertas_size(min = 0)]
    #[libertas_copy_from("$.qbi_sources")]
    pub members: Vec<QbiAggregationMemberChoice>,
    /// Actual current members, or genuine live sources for an inactive prior election
    #[libertas_read_only]
    #[libertas_copy_from("$.qbi_sources")]
    pub disclosure_sources: Vec<QbiAggregationMemberChoice>,
    /// Count of actual distinct trades/businesses established by the source owner
    /// [NullText]
    /// The source owner has not established the distinct business count
    /// [SetText]
    /// The source owner establishes this many distinct businesses
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_number(min = 0)]
    pub businesses: Option<i64>,
}

/// Topic record values read by the existing income estimate calculation
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IncomeRecordsEstimateInput {
    /// Every edited record, in its original Page order
    pub documents: Vec<IncomeRecordEstimateInput>,
    /// Every global aggregation facts row, in its original Page order
    pub qbi_aggregations: Vec<QbiAggregationEstimateInput>,
}
/// Editable fields from IncomeRecordItem; derived preview fields are excluded
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IncomeRecordEstimateInput {
    /// Actual accepted identity, or the existing new-row value zero
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Whose complete source document is edited
    pub owner: Owner,
    /// Complete IncomeRecord payload
    pub document: crate::IncomeRecord,
    /// Complete existing community or separate source answer
    pub community: CommunityShare,
    /// Actual withholding-fund management source answer
    /// [NullText]
    /// No separate Texas withholding-fund management answer applies
    /// [SetText]
    /// The withholding funds were managed as
    pub refund_management: Option<RefundManagement>,
}
/// Editable part of one QbiAggregationPage
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiAggregationEstimateInput {
    /// Source facts only; no members or business-count output
    pub facts: QbiAggregationFactsEstimateInput,
}
/// Exact editable facts, without Page catalog dependencies
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiAggregationFactsEstimateInput {
    /// Group name
    #[libertas_size(min = 1)]
    pub name: String,
    /// The same person or group meets the 50% common-ownership test
    /// Check the businesses’ ownership, operating and tax-year records.
    pub common_ownership: bool,
    /// That ownership exists for most of the year including its last day
    /// Check the businesses’ ownership, operating and tax-year records.
    pub majority_year_last_day: bool,
    /// The businesses report the same tax year, ignoring short years
    /// Check the businesses’ ownership, operating and tax-year records.
    pub same_tax_year: bool,
    /// Products or services are the same or customarily offered together
    /// Check the businesses’ ownership, operating and tax-year records.
    pub same_or_customary_products: bool,
    /// The businesses share facilities or significant centralized elements
    /// Check the businesses’ ownership, operating and tax-year records.
    pub shared_facilities_or_elements: bool,
    /// The businesses operate in coordination or reliance on one another
    /// Check the businesses’ ownership, operating and tax-year records.
    pub coordinated_or_interdependent: bool,
    /// Complete election and prior-year source facts
    pub election: QbiElectionHistory,
    /// Complete retained IRS restriction source facts
    pub irs_history: QbiIrsDisaggregationHistory,
    /// Every editable disclosure row in its original Page order
    #[libertas_size(min = 0)]
    pub disclosure: Vec<QbiBusinessDisclosureEstimateInput>,
}
/// Disclosure source values; native resolves the key against its genuine candidate catalog
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiBusinessDisclosureEstimateInput {
    /// Exact selected transport key; no output-catalog EnumSource on calculation input
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_size(min = 1)]
    pub source: String,
    /// Operating entity or business name
    #[libertas_size(min = 1)]
    pub entity_name: String,
    /// Complete actual EIN source answer
    pub ein: QbiEntityEin,
    /// Business description
    #[libertas_size(min = 1)]
    pub business_description: String,
    /// Complete actual changes source answer
    pub changes: QbiBusinessChanges,
}
/// Derived part of one existing aggregation Page row
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiAggregationDerived {
    /// Genuine current members; same full member payload type as the Page catalog
    #[libertas_read_only]
    #[libertas_size(min = 0)]
    pub members: Vec<QbiAggregationMemberChoice>,
    /// Genuine source choices for annual or retained historical disclosures
    #[libertas_read_only]
    pub disclosure_sources: Vec<QbiAggregationMemberChoice>,
    /// Count established by the actual source owner
    /// [NullText]
    /// The source owner has not established the distinct business count
    /// [SetText]
    /// The source owner establishes this many distinct businesses
    #[libertas_read_only]
    #[libertas_number(min = 0)]
    pub businesses: Option<i64>,
}

pub(crate) struct AggregationSourceView {
    pub member: QbiAggregationMemberChoice,
    // Supplied by the actual source owner. Passive activities and RPE flags
    // alone cannot establish a distinct section199A business count.
    pub businesses: Option<i64>,
}

pub(crate) fn source_views(
    draft: &crate::FederalDraft,
    accepted: &crate::FederalDraft,
) -> Vec<AggregationSourceView> {
    draft
        .income
        .iter()
        .flat_map(|entry| {
            // An ID assigned inside a tentative candidate is not yet selectable.
            if entry.id <= 0 {
                return Vec::new();
            }
            let Some(accepted_entry) = accepted.income.iter().find(|old| old.id == entry.id) else {
                return Vec::new();
            };
            if let Some(source) = crate::k1_activity::source(&entry.income) {
                return source
                    .qbi_blocks
                    .iter()
                    .filter(|block| {
                        block.id > 0
                            && crate::k1_activity::source(&accepted_entry.income).is_some_and(
                                |accepted_source| {
                                    accepted_source.qbi_blocks.iter().any(|old| old.id == block.id)
                                },
                            )
                    })
                    .map(|block| {
                        let source =
                            QbiAggregationSource::K1Block { document: entry.id, block: block.id };
                        let service = match &block.statement {
                            crate::K1ReportedQbiStatement::Partnership { statement } => {
                                statement.as_ref().map(|s| s.service)
                            }
                            crate::K1ReportedQbiStatement::Scorp { statement } => {
                                statement.as_ref().map(|s| s.service)
                            }
                            crate::K1ReportedQbiStatement::Unavailable => None,
                        };
                        let origin = if block.reported_aggregate {
                            QbiSourceOrigin::ReportedRpeAggregate { name: block.name.clone() }
                        } else if matches!(
                            entry.income,
                            crate::FederalIncome::PartnershipActivities { .. }
                        ) {
                            QbiSourceOrigin::PartnershipBusiness
                        } else {
                            QbiSourceOrigin::ScorpBusiness
                        };
                        AggregationSourceView {
                            // Only actual source-owned distinct names establish
                            // cardinality, independently of financial support.
                            businesses: block
                                .business_names
                                .as_ref()
                                .and_then(|names| i64::try_from(names.len()).ok()),
                            member: QbiAggregationMemberChoice {
                                key: source.key(),
                                source,
                                group: block.qbi_aggregation_group.clone(),
                                service,
                                origin,
                                owner: entry.owner.owned(),
                                label: crate::Message::new(
                                    "LABEL_K1_ACTIVITY_SOURCE",
                                    alloc::vec![crate::MessageArg::Text(block.name.clone())],
                                )
                                .encode(),
                            },
                        }
                    })
                    .collect();
            }
            let (group, service, origin, businesses) = match &entry.income {
                crate::FederalIncome::Business { data } => (
                    data.qbi_aggregation_group.clone(),
                    Some(data.specified_service),
                    QbiSourceOrigin::DirectBusiness,
                    Some(1),
                ),
                crate::FederalIncome::Rental { data }
                    if matches!(data.qbi_business, crate::RentalBusiness::Yes { .. }) =>
                {
                    (
                        data.qbi_aggregation_group.clone(),
                        Some(false),
                        QbiSourceOrigin::DirectRental,
                        Some(1),
                    )
                }
                crate::FederalIncome::FarmRental { data }
                    if matches!(data.qbi_business, crate::RentalBusiness::Yes { .. }) =>
                {
                    (
                        data.qbi_aggregation_group.clone(),
                        Some(false),
                        QbiSourceOrigin::DirectRental,
                        Some(1),
                    )
                }
                crate::FederalIncome::Scorp { data } => {
                    let data = &crate::ScorpK1::from(data);
                    let ordinary_only = data.rental_facts.real_estate_qbi.is_none()
                        && data.rental_facts.other_qbi.is_none();
                    let single =
                        data.qbi_statement.as_ref().is_some_and(|s| s.complete) && ordinary_only;
                    let service = data.qbi_statement.as_ref().map(|s| s.service);
                    (
                        data.qbi_aggregation_group.clone(),
                        service,
                        QbiSourceOrigin::ScorpBusiness,
                        single.then_some(1),
                    )
                }
                crate::FederalIncome::Partnership { data } if !data.publicly_traded => {
                    let ordinary_only = data.rental_facts.real_estate_qbi.is_none()
                        && data.rental_facts.other_qbi.is_none();
                    let single = data.qbi.is_some() && ordinary_only;
                    (
                        data.qbi_aggregation_group.clone(),
                        data.qbi.as_ref().map(|s| s.service),
                        QbiSourceOrigin::PartnershipBusiness,
                        single.then_some(1),
                    )
                }
                // Estate statements require their real source-owned adapter.
                // A parent never substitutes for its distinct QBI statements.
                _ => return Vec::new(),
            };
            let source = QbiAggregationSource::Income { document: entry.id };
            alloc::vec![AggregationSourceView {
                businesses,
                member: QbiAggregationMemberChoice {
                    key: source.key(),
                    source,
                    group,
                    service,
                    origin,
                    owner: entry.owner.owned(),
                    label: crate::income_names::income_display_label(entry, draft.dependents())
                        .encode(),
                },
            }]
        })
        .collect()
}

pub(crate) fn derived_group(
    name: &str,
    sources: &[AggregationSourceView],
) -> QbiAggregationDerived {
    let mut selected = sources.iter().filter(|s| s.member.group.as_deref() == Some(name));
    let members: Vec<_> = selected.clone().map(|s| s.member.clone()).collect();
    let disclosure_sources = if members.is_empty() {
        sources.iter().map(|s| s.member.clone()).collect()
    } else {
        members.clone()
    };
    let businesses = selected.try_fold(0i64, |count, source| {
        let n = source.businesses?;
        if n < 1 {
            return None;
        }
        count.checked_add(n)
    });
    QbiAggregationDerived { members, disclosure_sources, businesses }
}

// This checks source shape and readonly metadata only. An unfavorable or
// unavailable election answer remains an accepted source fact, not a Save
// failure. Source-associated history is checked separately from shape.
pub(crate) fn accept_pages(
    pages: &[QbiAggregationPage],
    sources: &[AggregationSourceView],
) -> Result<Vec<QbiAggregationFacts>, crate::validation::ValidationIssue> {
    let problem = "QBI_AGGREGATION_SOURCES";
    let issue = || crate::validation::ValidationIssue::new(problem);
    let mut result = Vec::with_capacity(pages.len());
    for (index, page) in pages.iter().enumerate() {
        let f = &page.facts;
        let facts_issue =
            |field| issue().at_field(field).in_field("facts").in_list("qbi_aggregations", index);
        let derived_issue = |field| issue().at_field(field).in_list("qbi_aggregations", index);
        if f.name.is_empty() || pages[..index].iter().any(|p| p.facts.name == f.name) {
            return Err(facts_issue("name"));
        }
        if !f.election.valid() {
            return Err(facts_issue("election"));
        }
        if !f.irs_history.valid() {
            return Err(facts_issue("irs_history"));
        }
        let expected = derived_group(&f.name, sources);
        if page.members.len() != expected.members.len()
            || page.members.iter().enumerate().any(|(i, m)| {
                !expected.members.contains(m) || page.members[..i].iter().any(|p| p.key == m.key)
            })
        {
            return Err(derived_issue("members"));
        }
        if page.businesses != expected.businesses {
            return Err(derived_issue("businesses"));
        }
        if page.disclosure_sources.len() != expected.disclosure_sources.len()
            || page.disclosure_sources.iter().enumerate().any(|(i, m)| {
                !expected.disclosure_sources.contains(m)
                    || page.disclosure_sources[..i].iter().any(|p| p.key == m.key)
            })
        {
            return Err(derived_issue("disclosure_sources"));
        }
        let mut disclosure = Vec::with_capacity(f.disclosure.len());
        for (i, row) in f.disclosure.iter().enumerate() {
            let row_issue = |field| {
                issue()
                    .at_field(field)
                    .in_list("disclosure", i)
                    .in_field("facts")
                    .in_list("qbi_aggregations", index)
            };
            let Some(source) = sources.iter().find(|s| s.member.key == row.source) else {
                return Err(row_issue("source"));
            };
            if !text_valid(&row.source)
                || f.disclosure[..i].iter().any(|r| r.source == row.source)
                || !expected.disclosure_sources.contains(&source.member)
            {
                return Err(row_issue("source"));
            }
            if !text_valid(&row.entity_name) {
                return Err(row_issue("entity_name"));
            }
            if !text_valid(&row.business_description) {
                return Err(row_issue("business_description"));
            }
            if !row.ein.valid() {
                return Err(row_issue("ein"));
            }
            if !row.changes.valid() {
                return Err(row_issue("changes"));
            }
            disclosure.push(QbiBusinessDisclosure {
                source: source.member.source.clone(),
                entity_name: row.entity_name.clone(),
                ein: row.ein.clone(),
                business_description: row.business_description.clone(),
                changes: row.changes.clone(),
            });
        }
        result.push(QbiAggregationFacts {
            name: f.name.clone(),
            common_ownership: f.common_ownership,
            majority_year_last_day: f.majority_year_last_day,
            same_tax_year: f.same_tax_year,
            same_or_customary_products: f.same_or_customary_products,
            shared_facilities_or_elements: f.shared_facilities_or_elements,
            coordinated_or_interdependent: f.coordinated_or_interdependent,
            election: f.election.clone(),
            irs_history: f.irs_history.clone(),
            disclosure,
        });
    }
    Ok(result)
}

pub(crate) fn project_pages(
    facts: &[QbiAggregationFacts],
    sources: &[AggregationSourceView],
) -> Vec<QbiAggregationPage> {
    facts
        .iter()
        .map(|f| {
            let derived = derived_group(&f.name, sources);
            QbiAggregationPage {
                facts: QbiAggregationFactsPage {
                    name: f.name.clone(),
                    common_ownership: f.common_ownership,
                    majority_year_last_day: f.majority_year_last_day,
                    same_tax_year: f.same_tax_year,
                    same_or_customary_products: f.same_or_customary_products,
                    shared_facilities_or_elements: f.shared_facilities_or_elements,
                    coordinated_or_interdependent: f.coordinated_or_interdependent,
                    election: f.election.clone(),
                    irs_history: f.irs_history.clone(),
                    disclosure: f
                        .disclosure
                        .iter()
                        .map(|r| QbiBusinessDisclosurePage {
                            source: r.source.key(),
                            entity_name: r.entity_name.clone(),
                            ein: r.ein.clone(),
                            business_description: r.business_description.clone(),
                            changes: r.changes.clone(),
                        })
                        .collect(),
                },
                members: derived.members,
                disclosure_sources: derived.disclosure_sources,
                businesses: derived.businesses,
            }
        })
        .collect()
}

// This is the approved factual prerequisite, not the complete tax readiness
// gate. The integrated caller must also preserve retained source histories and
// require genuine current/historical financial output from the owning engines.
pub(crate) fn computation_facts_ready(
    facts: &QbiAggregationFacts,
    sources: &[AggregationSourceView],
) -> bool {
    let derived = derived_group(&facts.name, sources);
    derived.businesses.is_some_and(|n| n >= 2)
        && derived.members.iter().all(|m| m.service == Some(false))
        && matches!(
            facts.election,
            QbiElectionHistory::FirstTimelyOriginalReturn
                | QbiElectionHistory::Continuing { .. }
                | QbiElectionHistory::ChangedFacts { .. }
        )
        && facts.irs_history.election_available()
        && facts.common_ownership
        && facts.majority_year_last_day
        && facts.same_tax_year
        && [
            facts.same_or_customary_products,
            facts.shared_facilities_or_elements,
            facts.coordinated_or_interdependent,
        ]
        .into_iter()
        .filter(|a| *a)
        .count()
            >= 2
        && facts.disclosure.len() == derived.members.len()
        && derived.members.iter().all(|member| {
            facts.disclosure.iter().any(|row| {
                row.source == member.source
                    && match row.ein {
                        QbiEntityEin::Known { .. } => true,
                        QbiEntityEin::NotApplicable => matches!(
                            member.origin,
                            QbiSourceOrigin::DirectBusiness | QbiSourceOrigin::DirectRental
                        ),
                        QbiEntityEin::Unavailable => false,
                    }
            })
        })
}

impl From<&crate::IncomeRecords> for IncomeRecordsEstimateInput {
    fn from(value: &crate::IncomeRecords) -> Self {
        Self {
            documents: value
                .documents
                .iter()
                .map(|d| IncomeRecordEstimateInput {
                    id: d.id,
                    owner: d.owner,
                    document: d.document.clone(),
                    community: d.community,
                    refund_management: d.refund_management,
                })
                .collect(),
            qbi_aggregations: value
                .qbi_aggregations
                .iter()
                .map(|g| {
                    let f = &g.facts;
                    QbiAggregationEstimateInput {
                        facts: QbiAggregationFactsEstimateInput {
                            name: f.name.clone(),
                            common_ownership: f.common_ownership,
                            majority_year_last_day: f.majority_year_last_day,
                            same_tax_year: f.same_tax_year,
                            same_or_customary_products: f.same_or_customary_products,
                            shared_facilities_or_elements: f.shared_facilities_or_elements,
                            coordinated_or_interdependent: f.coordinated_or_interdependent,
                            election: f.election.clone(),
                            irs_history: f.irs_history.clone(),
                            disclosure: f
                                .disclosure
                                .iter()
                                .map(|r| QbiBusinessDisclosureEstimateInput {
                                    source: r.source.clone(),
                                    entity_name: r.entity_name.clone(),
                                    ein: r.ein.clone(),
                                    business_description: r.business_description.clone(),
                                    changes: r.changes.clone(),
                                })
                                .collect(),
                        },
                    }
                })
                .collect(),
        }
    }
}

impl From<crate::IncomeRecords> for IncomeRecordsEstimateInput {
    fn from(value: crate::IncomeRecords) -> Self {
        Self::from(&value)
    }
}

impl IncomeRecordsEstimateInput {
    // Rebuild only the five source fields consumed by the existing candidate
    // pipeline. Its ordinary owners recompute Page-derived document metadata.
    pub(crate) fn into_page(self) -> crate::IncomeRecords {
        crate::IncomeRecords {
            documents: self
                .documents
                .into_iter()
                .map(|d| crate::IncomeRecordItem {
                    id: d.id,
                    owner: d.owner,
                    document: d.document,
                    community: d.community,
                    refund_management: d.refund_management,
                    investment_caps: None,
                    preview: None,
                    allowed_community: Vec::new(),
                    refund_management_needed: None,
                })
                .collect(),
            qbi_aggregations: self
                .qbi_aggregations
                .into_iter()
                .map(|g| {
                    let f = g.facts;
                    QbiAggregationPage {
                        facts: QbiAggregationFactsPage {
                            name: f.name,
                            common_ownership: f.common_ownership,
                            majority_year_last_day: f.majority_year_last_day,
                            same_tax_year: f.same_tax_year,
                            same_or_customary_products: f.same_or_customary_products,
                            shared_facilities_or_elements: f.shared_facilities_or_elements,
                            coordinated_or_interdependent: f.coordinated_or_interdependent,
                            election: f.election,
                            irs_history: f.irs_history,
                            disclosure: f
                                .disclosure
                                .into_iter()
                                .map(|r| QbiBusinessDisclosurePage {
                                    source: r.source,
                                    entity_name: r.entity_name,
                                    ein: r.ein,
                                    business_description: r.business_description,
                                    changes: r.changes,
                                })
                                .collect(),
                        },
                        members: Vec::new(),
                        disclosure_sources: Vec::new(),
                        businesses: None,
                    }
                })
                .collect(),
        }
    }
}

pub(crate) fn refresh_derived(pages: &mut [QbiAggregationPage], sources: &[AggregationSourceView]) {
    for page in pages {
        let derived = derived_group(&page.facts.name, sources);
        page.members = derived.members;
        page.disclosure_sources = derived.disclosure_sources;
        page.businesses = derived.businesses;
    }
}

pub(crate) fn reference_rule(draft: &crate::FederalDraft) -> Option<&'static str> {
    let facts = &draft.screening.as_ref()?.qbi_aggregations;
    if facts.is_empty() {
        return None;
    }
    let sources = source_views(draft, draft);
    facts
        .iter()
        .any(|f| f.disclosure.iter().any(|r| !sources.iter().any(|s| s.member.source == r.source)))
        .then_some("QBI_AGGREGATION_SOURCES")
}

pub(crate) fn uses_facts(draft: &crate::FederalDraft) -> bool {
    draft.screening.as_ref().is_some_and(|v| !v.qbi_aggregations.is_empty())
        || draft.income.iter().any(|entry| match &entry.income {
            crate::FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                data.qbi_aggregation_group.is_some()
            }
            crate::FederalIncome::Partnership { data } => data.qbi_aggregation_group.is_some(),
            _ => crate::k1_activity::source(&entry.income).is_some_and(|source| {
                source.qbi_blocks.iter().any(|block| block.qbi_aggregation_group.is_some())
            }),
        })
}

pub(crate) fn catalog_matches(
    supplied: &[QbiAggregationMemberChoice],
    expected: &[AggregationSourceView],
) -> bool {
    supplied.len() == expected.len()
        && supplied.iter().enumerate().all(|(i, s)| {
            !supplied[..i].iter().any(|old| old.key == s.key)
                && expected.iter().any(|actual| actual.member == *s)
        })
}

pub(crate) fn computation_issue(draft: &crate::FederalDraft) -> Option<&'static str> {
    if !uses_facts(draft) {
        return None;
    }
    if let Some(key) = accepted_rule(draft) {
        return Some(key);
    }
    let sources = source_views(draft, draft);
    let facts = draft.screening.as_ref().map_or(&[][..], |v| v.qbi_aggregations.as_slice());
    let associations = history_associations(facts, &sources);
    for association in &associations {
        // A continuing election cannot be abandoned by clearing or moving its
        // source labels. ChangedFacts is the existing explicit retest path.
        if matches!(association.facts.election, QbiElectionHistory::Continuing { .. }) {
            let group = association.source.member.group.as_deref();
            if group.is_none()
                || associations.iter().any(|other| {
                    core::ptr::eq(other.facts, association.facts)
                        && other.source.member.group.as_deref() != group
                })
            {
                return Some("QBI_AGGREGATION_HISTORY");
            }
        }
    }
    for source in &sources {
        let Some(group) = source.member.group.as_deref() else {
            continue;
        };
        if associations.iter().any(|association| {
            association.source.member.source == source.member.source
                && !association.facts.irs_history.election_available()
        }) {
            return Some("QBI_AGGREGATION_FACTS");
        }
        if associations.iter().any(|association| {
            if association.source.member.source != source.member.source
                || association.facts.name == group
            {
                return false;
            }
            match &association.facts.election {
                QbiElectionHistory::Continuing { original_year }
                | QbiElectionHistory::ChangedFacts { original_year, .. } => {
                    !facts.iter().any(|f| {
                        f.name == group
                            && match f.election {
                                QbiElectionHistory::Continuing { original_year: current }
                                | QbiElectionHistory::ChangedFacts {
                                    original_year: current, ..
                                } => current == *original_year,
                                _ => false,
                            }
                    })
                }
                QbiElectionHistory::FirstOnAmendedReturn | QbiElectionHistory::Unavailable => true,
                QbiElectionHistory::FirstTimelyOriginalReturn => false,
            }
        }) {
            return Some("QBI_AGGREGATION_HISTORY");
        }
        match facts.iter().find(|f| f.name == group) {
            Some(f) if computation_facts_ready(f, &sources) => {}
            None if matches!(
                source.member.origin,
                QbiSourceOrigin::DirectBusiness | QbiSourceOrigin::DirectRental
            ) => {}
            _ => return Some("QBI_AGGREGATION_FACTS"),
        }
    }
    None
}

// This view is transient: disclosure references associate retained facts with
// actual businesses, while current source labels supply associations even
// before a newly authored disclosure is complete. A label is never identity.
struct HistoryAssociation<'a> {
    source: &'a AggregationSourceView,
    facts: &'a QbiAggregationFacts,
}

fn history_associations<'a>(
    facts: &'a [QbiAggregationFacts],
    sources: &'a [AggregationSourceView],
) -> Vec<HistoryAssociation<'a>> {
    facts
        .iter()
        .flat_map(|facts| {
            sources.iter().filter_map(move |source| {
                (source.member.group.as_deref() == Some(facts.name.as_str())
                    || facts.disclosure.iter().any(|d| d.source == source.member.source))
                .then_some(HistoryAssociation { source, facts })
            })
        })
        .collect()
}

// Normal complete-candidate Save is also the source-history correction path.
// It may correct history and rename together. It must still explicitly carry
// history for the same live businesses; dropping a record or changing only a
// label cannot make retained known/unavailable history disappear.
pub(crate) fn history_transition_rule(
    previous: &crate::FederalDraft,
    candidate: &crate::FederalDraft,
) -> Option<&'static str> {
    let old_facts = previous.screening.as_ref()?.qbi_aggregations.as_slice();
    if old_facts.is_empty() {
        return None;
    }
    let old_sources = source_views(previous, previous);
    let new_sources = source_views(candidate, candidate);
    let new_facts = candidate.screening.as_ref().map_or(&[][..], |s| s.qbi_aggregations.as_slice());
    let new_associations = history_associations(new_facts, &new_sources);
    history_associations(old_facts, &old_sources)
        .iter()
        .any(|old| {
            (!matches!(old.facts.election, QbiElectionHistory::FirstTimelyOriginalReturn)
                || old.facts.irs_history != QbiIrsDisaggregationHistory::None)
                && new_sources.iter().any(|s| s.member.source == old.source.member.source)
                && !new_associations
                    .iter()
                    .any(|new| new.source.member.source == old.source.member.source)
        })
        .then_some("QBI_AGGREGATION_HISTORY")
}

// Restore and Save use the same accepted source-shape owner. Unavailable or
// unfavorable legal answers remain valid source facts; they are checked for
// computation readiness separately.
pub(crate) fn accepted_rule(draft: &crate::FederalDraft) -> Option<&'static str> {
    if draft.income.iter().any(|entry| {
        let group = match &entry.income {
            crate::FederalIncome::Scorp { data } => {
                crate::ScorpK1::from(data).qbi_aggregation_group
            }
            crate::FederalIncome::Partnership { data } => data.qbi_aggregation_group.clone(),
            _ => {
                return crate::k1_activity::source(&entry.income).is_some_and(|source| {
                    source.qbi_blocks.iter().any(|block| {
                        block.qbi_aggregation_group.as_ref().is_some_and(|s| s.is_empty())
                    })
                });
            }
        };
        group.as_ref().is_some_and(|s| s.is_empty())
    }) {
        return Some("QBI_AGGREGATION_SOURCES");
    }
    let facts = draft.screening.as_ref()?.qbi_aggregations.as_slice();
    if facts.is_empty() {
        return None;
    }
    let sources = source_views(draft, draft);
    accept_pages(&project_pages(facts, &sources), &sources).err().map(|issue| issue.key)
}
