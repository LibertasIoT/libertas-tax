// Schedule K-1 code C supplies a signed adjustment, not another depletion
// calculation. It belongs to its actual activity before AMT limitations.
// https://www.irs.gov/instructions/i1065sk1
// https://www.irs.gov/instructions/i1120ssk
use alloc::{collections::BTreeSet, vec::Vec};
use libertas_macros::*;

use crate::validation::ValidationIssue;
use crate::{K1ActivitySourceChoice, K1CodeRow};

/// The actual source allocation of a printed code C depletion adjustment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1DepletionSourceAllocation {
    /// Activity on this K-1's supporting statement
    #[libertas_key]
    #[libertas_unique]
    #[libertas_number(min = 1)]
    pub source: i64,
    /// Signed depletion adjustment for this activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents reconciliation across sources.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub adjustment: i64,
}

/// One activity from the regenerated same-paper catalog.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1DepletionSourceAllocationPage {
    /// Activity on this K-1's supporting statement
    #[libertas_read_only]
    #[libertas_key]
    #[libertas_unique]
    #[libertas_enum_source("^<K1DepletionSourcesPage>.sources")]
    #[libertas_number(min = 1)]
    pub source: i64,
    /// Signed depletion adjustment for this activity
    /// [NullText]
    /// The statement amount is unavailable
    /// [SetText]
    /// The signed adjustment is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents reconciliation across sources.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default($null)]
    pub adjustment: Option<i64>,
}

/// Associate the printed adjustment once, beside its K-1 on the existing topic.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1DepletionSourcesPage {
    /// Actual accepted same-K-1 sources
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub sources: Vec<K1ActivitySourceChoice>,
    /// Printed code C total
    #[libertas_read_only]
    #[libertas_output_only]
    #[libertas_money("USD", 2)]
    pub printed_total: Option<i64>,
    /// Each activity's signed amount from the supporting statement
    /// Include each listed activity exactly once, with an explicit zero when the statement assigns it no adjustment. Include offsetting positive and negative amounts even when the printed total is zero. The amounts must add to the printed total. Separate AMT activity records already include this adjustment; do not add it to those records again.
    #[libertas_read_only]
    pub allocations: Vec<K1DepletionSourceAllocationPage>,
}

pub(crate) fn has_depletion(rows: &[K1CodeRow]) -> bool {
    rows.iter().any(|row| row.code == "C" && row.amount != Some(0))
}

pub(crate) fn printed_depletion(rows: &[K1CodeRow]) -> Option<i64> {
    rows.iter()
        .filter(|row| row.code == "C")
        .try_fold(0i64, |total, row| total.checked_add(row.amount?))
}

fn validate_source_rows(
    sources: &[K1ActivitySourceChoice],
    rows: impl Iterator<Item = (i64, Option<i64>)>,
    require_all_rows: bool,
    total: Option<i64>,
) -> Result<(), ValidationIssue> {
    let mut seen = BTreeSet::new();
    let mut sum = 0i64;
    let mut all_known = true;
    for (source, adjustment) in rows {
        if !sources.iter().any(|choice| choice.id == source) || !seen.insert(source) {
            return Err(ValidationIssue::new("K1_DEPLETION_SOURCE_INVALID")
                .at_field("source")
                .in_key("depletion_sources", source));
        }
        if let Some(adjustment) = adjustment {
            sum = sum.checked_add(adjustment).ok_or_else(|| {
                ValidationIssue::new("K1_DEPLETION_SOURCE_TOTAL")
                    .at_field("adjustment")
                    .in_key("depletion_sources", source)
            })?;
        } else {
            all_known = false;
        }
    }
    if require_all_rows
        && let Some(missing) = sources.iter().find(|source| !seen.contains(&source.id))
    {
        return Err(ValidationIssue::new("K1_DEPLETION_SOURCE_INVALID")
            .at_field("adjustment")
            .in_key("depletion_sources", missing.id));
    }
    if !sources.is_empty()
        && all_known
        && seen.len() == sources.len()
        && total.is_some_and(|total| sum != total)
    {
        return Err(ValidationIssue::new("K1_DEPLETION_SOURCE_TOTAL")
            .at_field("adjustment")
            .in_key("depletion_sources", sources[0].id));
    }
    Ok(())
}

pub(crate) fn depletion_sources(
    rows: &[K1CodeRow],
    sources: &[K1ActivitySourceChoice],
    allocations: Option<&[K1DepletionSourceAllocation]>,
) -> Result<Vec<K1DepletionSourceAllocation>, ValidationIssue> {
    if !has_depletion(rows) {
        return Ok(Vec::new());
    }
    let total = printed_depletion(rows).ok_or_else(|| {
        ValidationIssue::new("K1_DEPLETION_SOURCE_UNAVAILABLE").at_field("depletion_sources")
    })?;
    if sources.is_empty() {
        return Err(
            ValidationIssue::new("K1_DEPLETION_SOURCE_UNAVAILABLE").at_field("depletion_sources")
        );
    }
    // A sole actual source is known from the paper and needs no repeated answer.
    if sources.len() == 1 {
        return Ok(alloc::vec![K1DepletionSourceAllocation {
            source: sources[0].id,
            adjustment: total
        }]);
    }
    validate_source_rows(
        sources,
        allocations.unwrap_or_default().iter().map(|row| (row.source, Some(row.adjustment))),
        false,
        printed_depletion(rows),
    )?;
    let allocations = allocations.unwrap_or_default();
    if let Some(missing) =
        sources.iter().find(|source| !allocations.iter().any(|row| row.source == source.id))
    {
        return Err(ValidationIssue::new("K1_DEPLETION_SOURCE_NEEDED")
            .at_field("adjustment")
            .in_key("depletion_sources", missing.id));
    }
    Ok(allocations.to_vec())
}

impl K1DepletionSourcesPage {
    pub(crate) fn from_sources(
        rows: &[K1CodeRow],
        sources: Vec<K1ActivitySourceChoice>,
        allocations: Option<&[K1DepletionSourceAllocation]>,
    ) -> Option<Self> {
        if !has_depletion(rows) || sources.len() < 2 {
            return None;
        }
        Some(Self {
            printed_total: Some(printed_depletion(rows)?),
            allocations: sources
                .iter()
                .map(|source| K1DepletionSourceAllocationPage {
                    source: source.id,
                    adjustment: allocations.and_then(|rows| {
                        rows.iter().find(|row| row.source == source.id).map(|row| row.adjustment)
                    }),
                })
                .collect(),
            sources,
        })
    }
}

impl K1DepletionSourcesPage {
    pub(crate) fn validate_for(&self, current: &Self) -> Result<(), ValidationIssue> {
        validate_source_rows(
            &current.sources,
            self.allocations.iter().map(|row| (row.source, row.adjustment)),
            true,
            current.printed_total,
        )
        .map_err(|issue| issue.map_field_prefix(&["depletion_sources"], &["allocations"]))
    }
    pub(crate) fn record(&self) -> Option<Vec<K1DepletionSourceAllocation>> {
        let known: Vec<_> = self
            .allocations
            .iter()
            .filter_map(|row| {
                Some(K1DepletionSourceAllocation {
                    source: row.source,
                    adjustment: row.adjustment?,
                })
            })
            .collect();
        (!known.is_empty()).then_some(known)
    }
}

macro_rules! activity_depletion {
    ($owner:ty) => {
        impl $owner {
            pub(crate) fn depletion_choices(&self) -> Vec<K1ActivitySourceChoice> {
                crate::K1ActivityChoiceContext::from_sources(self.activities.source()).components
            }
            pub(crate) fn depletion_source_issue(&self) -> Option<ValidationIssue> {
                self.validate_depletion_source().err()
            }
            fn validate_depletion_source(&self) -> Result<(), ValidationIssue> {
                if self.amt.iter().any(|row| !matches!(row.code.as_str(), "A" | "B" | "C")) {
                    return Err(
                        ValidationIssue::new("K1_DEPLETION_SOURCE_UNAVAILABLE").at_field("amt")
                    );
                }
                if self
                    .amt
                    .iter()
                    .any(|row| matches!(row.code.as_str(), "A" | "B") && row.amount != Some(0))
                    && self.activities.source().is_some_and(|source| {
                        source.components.iter().any(|component| {
                            matches!(component.alternative, crate::K1StatementAmt::SameAsRegular)
                        })
                    })
                {
                    return Err(
                        ValidationIssue::new("DOCUMENT_K1_AMT_RECORDS").at_field("activities")
                    );
                }
                let choices = self.depletion_choices();
                if choices.len() < 2 {
                    return Ok(());
                }
                validate_source_rows(
                    &choices,
                    self.depletion_sources
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .map(|row| (row.source, Some(row.adjustment))),
                    false,
                    printed_depletion(&self.amt),
                )
            }
            pub(crate) fn depletion_allocations(
                &self,
            ) -> Result<Vec<K1DepletionSourceAllocation>, ValidationIssue> {
                self.validate_depletion_source()?;
                depletion_sources(
                    &self.amt,
                    &self.depletion_choices(),
                    self.depletion_sources.as_deref(),
                )
            }
            pub(crate) fn depletion_adjustment(&self, component: i64) -> Option<i64> {
                Some(
                    self.depletion_allocations()
                        .ok()?
                        .iter()
                        .find(|row| row.source == component)
                        .map_or(0, |row| row.adjustment),
                )
            }
            pub(crate) fn depletion_page(&self) -> Option<K1DepletionSourcesPage> {
                K1DepletionSourcesPage::from_sources(
                    &self.amt,
                    self.depletion_choices(),
                    self.depletion_sources.as_deref(),
                )
            }
        }
    };
}
activity_depletion!(crate::PartnerActivityK1);
activity_depletion!(crate::ScorpActivityK1);
