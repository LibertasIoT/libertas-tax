pub(crate) use crate::charity_sources::parts_issue;
pub use crate::charity_sources::{
    OtherCharityEntity, ResearchCharityOrigin, ResearchCharityPart, ResearchCharityParts,
};
use crate::validation::ValidationIssue;
// The existing charitable carryover owner keeps every deduction, class,
// vintage, floor and closing amount. Activity origin parts attribute only the
// amount that owner actually allows; they do not create another deduction.
use alloc::{collections::BTreeMap, vec::Vec};
use libertas_macros::*;

/// Activity that made this retained charitable gift
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchCharityOriginPage {
    /// An actual partnership or S-corporation K-1 on this return
    CurrentK1 {
        /// K-1 entity
        #[libertas_enum_source("$.retained_charity_k1s")]
        source: i64,
    },
    /// Another entity absent from this return
    /// Use the entity's actual retained source records. Its gift remains an activity deduction, not a personal gift or an inferred current research source.
    OtherEntity(OtherCharityEntity),
}
/// Original appreciation attributable to this activity's gift
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CharitySourceAppreciation {
    /// Actual original appreciation
    Amount {
        /// Original appreciation, including an explicit zero
        /// From this activity's original gift records, before earlier deductions. It can exceed the unused balance.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
/// The existing financial facts needed to decide this source question
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CharityAppreciationFacts {
    pub class: crate::CharityClass,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub balance: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    /// [NullText]
    /// No original appreciation is supplied for a basis election
    /// [SetText]
    /// Appreciation in the original gift was
    pub appreciation: Option<i64>,
}
/// The source question's derived availability for one retained balance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CharityAppreciationChoices {
    pub allowed_source_appreciation: Vec<i32>,
}
/// One activity's retained charitable gift
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchCharityPartPage {
    /// Activity that made the gift
    #[libertas_constrained_by("$.retained_charity_origins")]
    pub origin: ResearchCharityOriginPage,
    /// Portion of this year's and class's unused balance
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
    /// Original appreciation attributable to this activity's gift
    /// Keep the original gift's appreciation separate from its remaining unused balance. Enter zero when the original gift had no appreciation.
    /// [NullText]
    /// No original-appreciation allocation applies to this retained gift
    /// [SetText]
    /// This activity's original appreciation was
    #[libertas_constrained_by("^<CharityCarryoverPage>.allowed_source_appreciation")]
    pub original_appreciation: Option<CharitySourceAppreciation>,
}
/// Activities that made this retained charitable gift
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchCharityPartsPage {
    /// Identified activity portions
    Parts {
        /// Activity portions of this balance; the remainder is personal
        #[libertas_size(min = 1)]
        /// ----
        /// Activity portion
        sources: Vec<ResearchCharityPartPage>,
    },
}
/// Charitable contribution carryover
/// One line of your last return's charitable contribution carryover worksheet.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("RetainedLossSourceOptions")]
pub struct CharityCarryoverPage {
    /// Year contributed
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2011, max = 2025)]
    pub year: u32,
    /// Class
    pub class: crate::CharityClass,
    /// Unused balance entering 2026
    /// The reconciled whole-dollar amount from the carryover worksheet, not the original gift.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    #[libertas_calculation_request("RetainedLossSourceOptions")]
    pub balance: i64,
    /// Appreciation in the original gift
    /// Only for a 30% capital-gain carryover when electing to reduce all 2026 capital-gain gifts to cost.
    /// [NullText]
    /// No original appreciation is supplied for a basis election
    /// [SetText]
    /// Appreciation in the original gift was
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub appreciation: Option<i64>,
    /// Same filers as the return that made the gift
    /// Your filing status and the names on that year's return are the same as on this one. No leaves this carryover out of the estimate with a note.
    pub same_filers: bool,
    /// Available retained-gift source question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.retained_research_choices")]
    #[libertas_output_only]
    pub allowed_research_parts: Vec<i32>,
    /// Available original-appreciation question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.retained_research_choices")]
    #[libertas_output_only]
    pub allowed_source_appreciation: Vec<i32>,
    /// Activities that made this retained charitable gift
    /// Use actual retained source records for this year and class. The remaining balance is personal.
    /// [NullText]
    /// All of this retained charitable gift is personal
    /// [SetText]
    /// These portions belong to actual activities
    #[libertas_constrained_by("^.allowed_research_parts")]
    pub research_parts: Option<ResearchCharityPartsPage>,
}
impl From<ResearchCharityOriginPage> for ResearchCharityOrigin {
    fn from(v: ResearchCharityOriginPage) -> Self {
        match v {
            ResearchCharityOriginPage::CurrentK1 { source } => Self::CurrentK1 { source },
            ResearchCharityOriginPage::OtherEntity(entity) => Self::OtherEntity(entity),
        }
    }
}
impl From<ResearchCharityOrigin> for ResearchCharityOriginPage {
    fn from(v: ResearchCharityOrigin) -> Self {
        match v {
            ResearchCharityOrigin::CurrentK1 { source } => Self::CurrentK1 { source },
            ResearchCharityOrigin::OtherEntity(entity) => Self::OtherEntity(entity),
        }
    }
}
impl From<CharityCarryoverPage> for crate::CharityCarryover {
    fn from(v: CharityCarryoverPage) -> Self {
        Self {
            year: v.year,
            class: v.class,
            balance: v.balance,
            appreciation: v.appreciation,
            same_filers: v.same_filers,
            research_parts: v.research_parts.map(|ResearchCharityPartsPage::Parts { sources }| {
                ResearchCharityParts::Parts {
                    sources: sources
                        .into_iter()
                        .map(|p| ResearchCharityPart {
                            origin: p.origin.into(),
                            amount: p.amount,
                            original_appreciation: p
                                .original_appreciation
                                .map(|CharitySourceAppreciation::Amount { amount }| amount),
                        })
                        .collect(),
                }
            }),
        }
    }
}
impl From<crate::CharityCarryover> for CharityCarryoverPage {
    fn from(v: crate::CharityCarryover) -> Self {
        Self {
            year: v.year,
            class: v.class,
            balance: v.balance,
            appreciation: v.appreciation,
            same_filers: v.same_filers,
            allowed_research_parts: Vec::new(),
            allowed_source_appreciation: Vec::new(),
            research_parts: v.research_parts.map(|ResearchCharityParts::Parts { sources }| {
                ResearchCharityPartsPage::Parts {
                    sources: sources
                        .into_iter()
                        .map(|p| ResearchCharityPartPage {
                            origin: p.origin.into(),
                            amount: p.amount,
                            original_appreciation: p
                                .original_appreciation
                                .map(|amount| CharitySourceAppreciation::Amount { amount }),
                        })
                        .collect(),
                }
            }),
        }
    }
}
pub(crate) fn appreciation_choices(
    class: crate::CharityClass,
    balance: i64,
    appreciation: Option<i64>,
    election: bool,
    available: &[i32],
) -> Vec<i32> {
    if election
        && matches!(class, crate::CharityClass::CapitalGain30)
        && balance > 0
        && appreciation.is_some_and(|v| v > 0)
        && !available.is_empty()
    {
        alloc::vec![0]
    } else {
        Vec::new()
    }
}
pub(crate) fn origin_choices(d: &crate::FederalDraft) -> Vec<i32> {
    if crate::investment_interest::k1_sources(d).is_empty() {
        alloc::vec![1]
    } else {
        alloc::vec![0, 1]
    }
}
pub(crate) fn rule(d: &crate::FederalDraft) -> Option<&'static str> {
    issue(d).map(|(_, issue)| issue.key)
}

// The caller correlates this retained-record index to its actual edited page.
pub(crate) fn issue(d: &crate::FederalDraft) -> Option<(usize, ValidationIssue)> {
    let records = &d.deductions.as_ref()?.charity_carryovers;
    let catalog = crate::investment_interest::k1_sources(d);
    let available = crate::retained_research::available(d);
    for (record_index, record) in records.iter().enumerate() {
        if let Some(issue) = parts_issue(record, Some(&catalog)) {
            return Some((record_index, issue));
        }
        if !appreciation_choices(
            record.class,
            record.balance,
            record.appreciation,
            d.deductions.as_ref()?.basis_election,
            &available,
        )
        .is_empty()
            && let Some(ResearchCharityParts::Parts { sources }) = &record.research_parts
            && let Some(source_index) =
                sources.iter().position(|p| p.original_appreciation.is_none())
        {
            return Some((
                record_index,
                ValidationIssue::new("RETAINED_CHARITY_SOURCE_APPRECIATION_NEEDED")
                    .at_field("original_appreciation")
                    .in_list("sources", source_index)
                    .in_field("research_parts"),
            ));
        }
        if record.research_parts.is_some() && available.is_empty() {
            return Some((
                record_index,
                ValidationIssue::new("RETAINED_RESEARCH_SOURCE_UNAVAILABLE")
                    .at_field("research_parts"),
            ));
        }
    }
    None
}
// Actual allowed whole-dollar amounts, before section68, with cent source
// weights. Personal and other-entity remainders stay distinct from current K-1
// identities. Canonical keys make row order immaterial to owner conservation.
pub(crate) fn deductions(
    records: &[crate::CharityCarryover],
    worksheet: &crate::charity::Worksheet,
    basis_election: bool,
) -> BTreeMap<(bool, Option<i64>), i64> {
    let mut out = BTreeMap::new();
    for (record, result) in records.iter().zip(&worksheet.carryovers) {
        let allowed = crate::kernel::ratio::positive(result.used - result.floor);
        if allowed == 0 {
            continue;
        }
        let refigure = basis_election && matches!(record.class, crate::CharityClass::CapitalGain30);
        let mut weights = BTreeMap::<(bool, Option<i64>), i64>::new();
        let mut original_parts = 0;
        let mut appreciation_parts = 0;
        if let Some(ResearchCharityParts::Parts { sources }) = &record.research_parts {
            for part in sources {
                let source = match part.origin {
                    ResearchCharityOrigin::CurrentK1 { source } => Some(source),
                    ResearchCharityOrigin::OtherEntity(_) => None,
                };
                original_parts += part.amount;
                appreciation_parts += part.original_appreciation.unwrap_or(0);
                let claim = if refigure {
                    crate::kernel::ratio::positive(
                        part.amount - part.original_appreciation.unwrap_or(0),
                    )
                } else {
                    part.amount
                };
                *weights.entry((true, source)).or_default() += claim;
            }
        }
        let personal = record.balance - original_parts;
        let personal_appreciation = record.appreciation.unwrap_or(0) - appreciation_parts;
        weights.insert(
            (false, None),
            if refigure {
                crate::kernel::ratio::positive(personal - personal_appreciation)
            } else {
                personal
            },
        );
        let portions = crate::kernel::ratio::allocate_remaining(
            allowed,
            &weights.values().copied().collect::<Vec<_>>(),
        );
        for (key, part) in weights.into_keys().zip(portions) {
            *out.entry(key).or_default() += part;
        }
    }
    out
}
