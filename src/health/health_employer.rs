pub use crate::health_employer_source::{
    HealthEmployerBinding, HealthEmployerKind, HealthEmployerYear,
};
use crate::validation::ValidationIssue;
// Coverage: W-2 employer HSA/Archer contribution-year sources.
// Same-owner accepted source identities and actual year allocations supply the
// existing contribution owners; original correction/excess owners remain separate.
// HSA source adjustment: final2025 and draft2026 i8889 Employer Contribution
// Worksheet. Archer designation: i8853 line1 and i5498-SA boxes2/3, an explicit
// source-year fitting inference. Unknown designation prevents a complete estimate.
// https://www.irs.gov/instructions/i8889
// https://www.irs.gov/instructions/i8853
// https://www.irs.gov/instructions/i1099sa
use crate::{FederalDraft, FederalIncome, Owner, W2Code};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

type Rule = &'static str;

impl HealthEmployerKind {
    fn code(self) -> W2Code {
        match self {
            Self::Hsa => W2Code::W,
            Self::Archer => W2Code::R,
        }
    }
}

/// Entered W-2 employer contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HealthEmployerSourceRow {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    pub document: i64,
    /// W-2 source
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// W-2 owner
    #[libertas_read_only]
    pub owner: Owner,
    /// Medical account
    #[libertas_read_only]
    pub kind: HealthEmployerKind,
    /// Entered W-2 code amount
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub reported: i64,
    /// Employer's designated contribution year
    pub allocation: HealthEmployerYear,
}

// An entered W-2 code amount whose contribution year is asked separately.
struct Source {
    document: i64,
    label: Vec<u8>,
    owner: Owner,
    kind: HealthEmployerKind,
    reported: i64,
}

// Actual accepted source projection; no manual entered total can manufacture a
// source. One row per W-2 ID and code, irrespective of other W-2 field changes.
fn live_sources(d: &FederalDraft, keep_zero: bool) -> Result<Vec<Source>, Rule> {
    let mut rows = Vec::new();
    for entry in &d.income {
        let FederalIncome::Wage { data } = &entry.income else {
            continue;
        };
        let data = crate::FederalWage::from(data);
        for kind in [HealthEmployerKind::Hsa, HealthEmployerKind::Archer] {
            let entries: Vec<_> =
                data.other_box12.iter().filter(|b| b.code == kind.code()).collect();
            if entries.is_empty() {
                continue;
            }
            if entries.iter().any(|entry| entry.amount < 0) {
                return Err("HEALTH_EMPLOYER_TOTAL_INVALID");
            }
            let reported = entries
                .iter()
                .try_fold(0_i64, |total, b| total.checked_add(b.amount))
                .ok_or("HEALTH_EMPLOYER_TOTAL_INVALID")?;
            if reported == 0 && !keep_zero {
                continue;
            }
            if entry.id <= 0 {
                return Err("HEALTH_EMPLOYER_SOURCE_NOT_OFFERED");
            }
            rows.push(Source {
                document: entry.id,
                label: crate::income_names::income_display_label(entry, d.dependents()).encode(),
                owner: entry.owner.owned(),
                kind,
                reported,
            });
        }
    }
    Ok(rows)
}
fn same_source(binding: &HealthEmployerBinding, row: &Source) -> bool {
    binding.document == row.document && binding.kind == row.kind && binding.owner == row.owner
}
fn split_changed(binding: &HealthEmployerBinding, row: &Source) -> bool {
    matches!(binding.allocation, HealthEmployerYear::Split { .. })
        && binding.accepted_amount != row.reported
}

// The sources whose contribution year the person has answered. A source with
// no accepted allocation (or a Split whose source amount changed) is asked
// first and never appears as a row.
pub(crate) fn page_rows(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
) -> Result<Vec<HealthEmployerSourceRow>, Rule> {
    let mut rows = Vec::new();
    for source in live_sources(d, false)? {
        if let Some(binding) =
            bindings.iter().find(|b| same_source(b, &source) && !split_changed(b, &source))
        {
            rows.push(HealthEmployerSourceRow {
                document: source.document,
                label: source.label,
                owner: source.owner,
                kind: source.kind,
                reported: source.reported,
                allocation: binding.allocation.clone(),
            });
        }
    }
    Ok(rows)
}

// Test fixtures that designate every reported amount for the current year
// (what their synthetic returns always meant) without the questions page.
#[cfg(test)]
pub(crate) fn page_rows_current_year(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
) -> Result<Vec<HealthEmployerSourceRow>, Rule> {
    Ok(live_sources(d, false)?
        .into_iter()
        .map(|source| HealthEmployerSourceRow {
            allocation: bindings
                .iter()
                .find(|b| same_source(b, &source) && !split_changed(b, &source))
                .map_or(HealthEmployerYear::CurrentYear, |b| b.allocation.clone()),
            document: source.document,
            label: source.label,
            owner: source.owner,
            kind: source.kind,
            reported: source.reported,
        })
        .collect())
}

/// An entered W-2 code amount whose designated contribution year is not
/// answered: the page asks it first, as a required question.
pub(crate) struct OpenSource {
    pub document: i64,
    pub kind: HealthEmployerKind,
    pub label: Vec<u8>,
}

pub(crate) fn open_sources(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
) -> Result<Vec<OpenSource>, Rule> {
    Ok(live_sources(d, false)?
        .into_iter()
        .filter(|source| {
            !bindings.iter().any(|b| same_source(b, source) && !split_changed(b, source))
        })
        .map(|source| OpenSource {
            document: source.document,
            kind: source.kind,
            label: source.label,
        })
        .collect())
}

/// The answered contribution year of one open source, as a binding.
pub(crate) fn answered_binding(
    d: &FederalDraft,
    document: i64,
    kind: HealthEmployerKind,
    allocation: HealthEmployerYear,
) -> Result<HealthEmployerBinding, ValidationIssue> {
    let source = live_sources(d, false)
        .map_err(ValidationIssue::new)?
        .into_iter()
        .find(|source| source.document == document && source.kind == kind)
        .ok_or_else(|| ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_NOT_OFFERED"))?;
    allocation.checked_current_part(source.reported)?;
    Ok(HealthEmployerBinding {
        document,
        owner: source.owner,
        kind,
        accepted_amount: source.reported,
        allocation,
    })
}

// A saved source edit never leaves a stale answer: a binding whose W-2 source
// is gone, changed owner or code, or whose Split no longer matches the
// reported amount is dropped, and the year is asked again (a required
// question) when the source still exists.
pub(crate) fn prune(d: &mut FederalDraft) {
    let Some(adjustments) = d.adjustments.as_ref() else { return };
    let Ok(sources) = live_sources(d, true) else { return };
    let kept: Vec<_> = adjustments
        .health_employer_w2
        .iter()
        .filter(|b| sources.iter().any(|s| same_source(b, s) && !split_changed(b, s)))
        .cloned()
        .collect();
    if kept.len() != adjustments.health_employer_w2.len()
        && let Some(adjustments) = d.adjustments.as_mut()
    {
        adjustments.health_employer_w2 = kept;
    }
}

// Existing source-reference guard: reject deletion/type/code/owner loss before
// unlinking an accepted health source. A Split amount change also requires
// clearing/changing the allocation through the ordinary Health page first.
pub(crate) fn reference_issue(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
) -> Option<(Option<i64>, ValidationIssue)> {
    // A whole-year correction to zero preserves the actual source identity.
    // Zero amounts are omitted only from the editable contribution projection;
    // a Split still binds its original amount and cannot change its residual.
    let rows = match live_sources(d, true) {
        Ok(rows) => rows,
        Err(key) => return Some((None, ValidationIssue::new(key))),
    };
    for binding in bindings {
        let Some(row) = rows.iter().find(|row| same_source(binding, row)) else {
            return Some((
                Some(binding.document),
                ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_REFERENCED"),
            ));
        };
        if split_changed(binding, row) {
            return Some((
                Some(binding.document),
                ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_CHANGED"),
            ));
        }
    }
    None
}

pub(crate) fn reference_rule(d: &FederalDraft, bindings: &[HealthEmployerBinding]) -> Option<Rule> {
    reference_issue(d, bindings).map(|(_, issue)| issue.key)
}

// Literal source membership and current amount are validated on normal Save as
// well as native Estimate. The readonly client collection is not trusted data.
// Every offered source must be answered: an unanswered source is asked first.
pub(crate) fn accept_page_issue(
    d: &FederalDraft,
    previous_bindings: &[HealthEmployerBinding],
    rows: &[HealthEmployerSourceRow],
) -> Result<Vec<HealthEmployerBinding>, ValidationIssue> {
    let offered = live_sources(d, false)
        .map_err(|key| ValidationIssue::new(key).at_field("employer_sources"))?;
    if rows.len() != offered.len() {
        let missing = offered.iter().any(|source| {
            !rows.iter().any(|row| row.document == source.document && row.kind == source.kind)
        });
        return Err(ValidationIssue::new(if missing {
            "HEALTH_EMPLOYER_YEAR_REQUIRED"
        } else {
            "HEALTH_EMPLOYER_SOURCE_NOT_OFFERED"
        })
        .at_field("employer_sources"));
    }
    let mut result: Vec<HealthEmployerBinding> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if rows[..index]
            .iter()
            .any(|previous| previous.document == row.document && previous.kind == row.kind)
        {
            return Err(ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_TWICE")
                .at_field("document")
                .in_list("employer_sources", index));
        }
        let source = offered
            .iter()
            .find(|source| source.document == row.document && source.kind == row.kind)
            .ok_or_else(|| {
                ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_NOT_OFFERED")
                    .at_field("document")
                    .in_list("employer_sources", index)
            })?;
        if row.owner != source.owner || row.reported != source.reported || row.label != source.label
        {
            return Err(ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_CHANGED")
                .at_field(if row.owner != source.owner {
                    "owner"
                } else if row.reported != source.reported {
                    "reported"
                } else {
                    "label"
                })
                .in_list("employer_sources", index));
        }
        // A retained Split whose source amount changed is asked again, never
        // silently restamped by a row carrying the live amount.
        if previous_bindings
            .iter()
            .any(|binding| same_source(binding, source) && split_changed(binding, source))
        {
            return Err(ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_CHANGED")
                .at_field("allocation")
                .in_list("employer_sources", index));
        }
        row.allocation
            .checked_current_part(row.reported)
            .map_err(|issue| issue.in_field("allocation").in_list("employer_sources", index))?;
        result.push(HealthEmployerBinding {
            document: row.document,
            owner: row.owner,
            kind: row.kind,
            accepted_amount: row.reported,
            allocation: row.allocation.clone(),
        });
    }
    Ok(result)
}

#[cfg(test)]
pub(crate) fn accept_page(
    d: &FederalDraft,
    previous_bindings: &[HealthEmployerBinding],
    rows: &[HealthEmployerSourceRow],
) -> Result<Vec<HealthEmployerBinding>, Rule> {
    accept_page_issue(d, previous_bindings, rows).map_err(|issue| issue.key)
}

// Structural/reference validity is independent of source-year completeness.
pub(crate) fn binding_rule(d: &FederalDraft, bindings: &[HealthEmployerBinding]) -> Option<Rule> {
    if bindings.iter().any(|binding| binding.accepted_amount <= 0) {
        return Some("HEALTH_EMPLOYER_TOTAL_INVALID");
    }
    if let Some(key) = reference_rule(d, bindings) {
        return Some(key);
    }
    bindings
        .iter()
        .enumerate()
        .any(|(i, binding)| {
            bindings[..i].iter().any(|previous| {
                previous.document == binding.document && previous.kind == binding.kind
            })
        })
        .then_some("HEALTH_EMPLOYER_SOURCE_TWICE")
}

// The matching owner's current-year account record.
fn has_account(d: &FederalDraft, owner: Owner, kind: HealthEmployerKind) -> bool {
    d.adjustments.as_ref().is_some_and(|a| match kind {
        HealthEmployerKind::Hsa => a.hsa.iter().any(|h| h.owner == owner),
        HealthEmployerKind::Archer => matches!(&a.archer, crate::ArcherContributions::Records { owners, .. } if owners.iter().any(|h| h.owner == owner)),
    })
}

/// The account kind a current-year employer contribution lacks, if any: the
/// page refuses to save without the owner's matching account record.
pub(crate) fn account_missing(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
) -> Option<HealthEmployerKind> {
    [HealthEmployerKind::Hsa, HealthEmployerKind::Archer].into_iter().find(|kind| {
        [Owner::Taxpayer, Owner::Spouse].into_iter().any(|owner| {
            source_totals(d, bindings, owner, *kind).is_ok_and(|(current, _)| current > 0)
                && !has_account(d, owner, *kind)
        })
    })
}

// Complete mandatory-source gate, shared by estimate and normal walk review.
// This catches new W-2s, stale Split, duplicate accepted bindings and unknown
// years independently of whether a current account happens to exist.
pub(crate) fn mandatory_rule(d: &FederalDraft, bindings: &[HealthEmployerBinding]) -> Option<Rule> {
    if let Some(key) = binding_rule(d, bindings) {
        return Some(key);
    }
    let rows = match live_sources(d, false) {
        Ok(rows) => rows,
        Err(key) => return Some(key),
    };
    for row in &rows {
        let Some(binding) = bindings.iter().find(|b| same_source(b, row)) else {
            return Some("HEALTH_EMPLOYER_YEAR_REQUIRED");
        };
        if split_changed(binding, row) {
            return Some("HEALTH_EMPLOYER_SOURCE_CHANGED");
        }
        let current = match binding.allocation.current_part(row.reported) {
            Ok(value) => value,
            Err(key) => return Some(key),
        };
        if current > 0 && !has_account(d, row.owner, row.kind) {
            return Some("HEALTH_EMPLOYER_ACCOUNT_REQUIRED");
        }
    }
    None
}

// Derive only admitted source amounts: matching Unknown/missing allocations
// contribute neither current nor prior principal to this incomplete subtotal.
// This allows the normal Health Save to clear an allocation without deleting
// its contributor or retaining a stale effective amount. mandatory_rule is an
// independent global gate: a completed estimate must never treat omitted
// unknown principal as a known zero. Existing corrections/later-deposit guards
// still require enough actual admitted current principal plus outside funding.
// Answered-source conservation is intrinsic: prior is reported-current.
// It is deliberately not fed to prior taxable excess/correction calculations.
pub(crate) fn source_totals(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
    owner: Owner,
    kind: HealthEmployerKind,
) -> Result<(i64, i64), Rule> {
    if let Some(key) = binding_rule(d, bindings) {
        return Err(key);
    }
    let mut current = 0_i64;
    let mut prior = 0_i64;
    for row in live_sources(d, false)?.iter().filter(|row| row.owner == owner && row.kind == kind) {
        let Some(binding) = bindings.iter().find(|b| same_source(b, row)) else {
            continue;
        };
        let part = binding.allocation.current_part(row.reported)?;
        current = current.checked_add(part).ok_or("HEALTH_EMPLOYER_TOTAL_INVALID")?;
        prior = prior.checked_add(row.reported - part).ok_or("HEALTH_EMPLOYER_TOTAL_INVALID")?;
    }
    Ok((current, prior))
}

pub(crate) fn effective_total(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
    owner: Owner,
    kind: HealthEmployerKind,
    outside: i64,
) -> Result<i64, Rule> {
    if outside < 0 {
        return Err("HEALTH_EMPLOYER_TOTAL_INVALID");
    }
    let (current, _) = source_totals(d, bindings, owner, kind)?;
    let statements = crate::contribution_documents::project(d).ok().map_or(0, |totals| {
        let owner = usize::from(owner == Owner::Spouse);
        match kind {
            HealthEmployerKind::Hsa => totals.hsa[owner].employer_outside,
            HealthEmployerKind::Archer => totals.archer[owner].employer_outside,
        }
    });
    outside
        .checked_add(current)
        .and_then(|v| v.checked_add(statements))
        .ok_or("HEALTH_EMPLOYER_TOTAL_INVALID")
}

pub(crate) fn bindings(d: &FederalDraft) -> &[HealthEmployerBinding] {
    d.adjustments.as_ref().map_or(&[], |a| &a.health_employer_w2)
}

// Accepted contribution totals are derived fields. Restored records must fit
// the same source graph that normal candidate/save reconciliation derives.
pub(crate) fn derived_rule(d: &FederalDraft) -> Option<Rule> {
    let a = d.adjustments.as_ref()?;
    for h in &a.hsa {
        match effective_total(
            d,
            &a.health_employer_w2,
            h.owner,
            HealthEmployerKind::Hsa,
            h.employer_outside,
        ) {
            Ok(total) if total == h.employer => (),
            _ => return Some("HEALTH_EMPLOYER_TOTAL_INVALID"),
        }
    }
    if let crate::ArcherContributions::Records { owners, .. } = &a.archer {
        for h in owners {
            match effective_total(
                d,
                &a.health_employer_w2,
                h.owner,
                HealthEmployerKind::Archer,
                h.employer_outside,
            ) {
                Ok(total) if total == h.employer => (),
                _ => return Some("HEALTH_EMPLOYER_TOTAL_INVALID"),
            }
        }
    }
    None
}
