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

/// Employer medical-account contribution kind
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HealthEmployerKind {
    /// HSA, Form W-2 box 12 code W
    Hsa,
    /// Archer MSA, Form W-2 box 12 code R
    Archer,
}
impl HealthEmployerKind {
    fn code(self) -> W2Code {
        match self {
            Self::Hsa => W2Code::W,
            Self::Archer => W2Code::R,
        }
    }
}

/// Contribution year of the entered W-2 amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HealthEmployerYear {
    // The printed W-2 code does not identify the designated contribution year;
    // missing employer records retain source availability, not a guessed year.
    // HEALTH_EMPLOYER_YEAR_REQUIRED remains the named calculation boundary.
    /// Not yet determined
    /// Use the employer's actual designation. The App cannot complete this source until its contribution year is known.
    Unknown,
    /// All designated for 2026
    CurrentYear,
    /// All designated for 2025
    PriorYear,
    /// Split between 2026 and 2025
    Split {
        /// Amount designated for 2026
        /// The rest of the displayed W-2 code amount belongs to 2025; do not enter that source amount again as an outside-W-2 employer contribution.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        current: i64,
    },
}
impl HealthEmployerYear {
    fn checked_current_part(&self, reported: i64) -> Result<i64, ValidationIssue> {
        match self {
            Self::Unknown => Err(ValidationIssue::new("HEALTH_EMPLOYER_YEAR_REQUIRED").at_record()),
            Self::CurrentYear => Ok(reported),
            Self::PriorYear => Ok(0),
            Self::Split { current } if (0..=reported).contains(current) => Ok(*current),
            Self::Split { .. } => {
                Err(ValidationIssue::new("HEALTH_EMPLOYER_SPLIT_ABOVE_REPORTED")
                    .at_field("current"))
            }
        }
    }
    fn current_part(&self, reported: i64) -> Result<i64, Rule> {
        self.checked_current_part(reported).map_err(|issue| issue.key)
    }
}

/// Accepted W-2 medical-account year allocation
/// The source amount is an accepted binding, not another editable contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthEmployerBinding {
    /// Actual accepted income document identity
    pub document: i64,
    /// Actual accepted income owner
    pub owner: Owner,
    /// Actual source code kind
    pub kind: HealthEmployerKind,
    /// Source code amount when this allocation was accepted
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub accepted_amount: i64,
    /// Designated contribution year
    pub allocation: HealthEmployerYear,
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

// Actual accepted source projection; no manual entered total can manufacture a
// source. One row per W-2 ID and code, irrespective of other W-2 field changes.
fn live_sources(d: &FederalDraft, keep_zero: bool) -> Result<Vec<HealthEmployerSourceRow>, Rule> {
    let mut rows = Vec::new();
    for entry in &d.income {
        let FederalIncome::Wage { data } = &entry.income else {
            continue;
        };
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
            rows.push(HealthEmployerSourceRow {
                document: entry.id,
                label: crate::interview::income_display_label(entry, d.dependents()).encode(),
                owner: entry.owner,
                kind,
                reported,
                allocation: HealthEmployerYear::Unknown,
            });
        }
    }
    Ok(rows)
}
fn same_source(binding: &HealthEmployerBinding, row: &HealthEmployerSourceRow) -> bool {
    binding.document == row.document && binding.kind == row.kind && binding.owner == row.owner
}
fn split_changed(binding: &HealthEmployerBinding, row: &HealthEmployerSourceRow) -> bool {
    matches!(binding.allocation, HealthEmployerYear::Split { .. })
        && binding.accepted_amount != row.reported
}

// Opening a page never clears/stamps/rebinds a changed Split. Normal source
// mutations are guarded; even a hypothetical stale snapshot retains the choice
// until it is explicitly cleared through the ordinary year control.
pub(crate) fn page_rows(
    d: &FederalDraft,
    bindings: &[HealthEmployerBinding],
) -> Result<Vec<HealthEmployerSourceRow>, Rule> {
    let mut rows = live_sources(d, false)?;
    for row in &mut rows {
        if let Some(binding) = bindings.iter().find(|b| same_source(b, row)) {
            row.allocation = binding.allocation.clone();
        }
    }
    Ok(rows)
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
        if matches!(binding.allocation, HealthEmployerYear::Unknown) {
            continue;
        }
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
// Unknown can save incomplete; only an answered binding can enter tax totals.
pub(crate) fn accept_page_issue(
    d: &FederalDraft,
    previous_bindings: &[HealthEmployerBinding],
    rows: &[HealthEmployerSourceRow],
) -> Result<Vec<HealthEmployerBinding>, ValidationIssue> {
    let offered = live_sources(d, false)
        .map_err(|key| ValidationIssue::new(key).at_field("employer_sources"))?;
    if rows.len() != offered.len() {
        return Err(
            ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_NOT_OFFERED").at_field("employer_sources")
        );
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
        // Do not silently restamp a hypothetical stale accepted Split merely
        // because this page carries the current live amount. Its retained
        // choice stays visible, and the ordinary Unknown choice explicitly
        // clears it before a new allocation can be accepted.
        if previous_bindings
            .iter()
            .any(|binding| same_source(binding, row) && split_changed(binding, row))
            && !matches!(row.allocation, HealthEmployerYear::Unknown)
        {
            return Err(ValidationIssue::new("HEALTH_EMPLOYER_SOURCE_CHANGED")
                .at_field("allocation")
                .in_list("employer_sources", index));
        }
        if matches!(row.allocation, HealthEmployerYear::Unknown) {
            // Choosing Unknown explicitly clears the accepted source binding.
            // The live W-2 still generates its row and mandatory source issue.
            // A later source deletion is therefore possible through normal
            // controls after the health allocation is cleared.
            continue;
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
        if current > 0 && !d.adjustments.as_ref().is_some_and(|a| match row.kind {
            HealthEmployerKind::Hsa => a.hsa.iter().any(|h| h.owner == row.owner),
            HealthEmployerKind::Archer => matches!(&a.archer, crate::ArcherContributions::Records { owners, .. } if owners.iter().any(|h| h.owner == row.owner)),
        }) {
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
        if matches!(binding.allocation, HealthEmployerYear::Unknown) {
            continue;
        }
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
    outside.checked_add(current).ok_or("HEALTH_EMPLOYER_TOTAL_INVALID")
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
