// Actual 1993 transition source fitting; the historical owner alone computes
// depreciation. PL103-66 section13151(b),107Stat448; Pub946 GDS table footnote4.
use alloc::{vec, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

use crate::kernel::source_fact::SourceFact;
use crate::validation::ValidationIssue;
use crate::{Commercial1993Transition, Rental, RentalPropertyType};

/// Source dates used to find applicable rental recovery choices
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalRecoveryDate {
    /// Date this source became available to rent
    #[libertas_date_only]
    pub in_service: u32,
}

/// Actual improvement source dates
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalRecoveryMoreDates {
    /// Improvement dates, in their current order
    pub improvements: Vec<RentalRecoveryDate>,
}

/// Actual property kind and dates, without amounts or calculated choices
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalRecoveryInputs {
    /// Actual location of the current rental property
    pub location: crate::RentalLocation,
    /// Actual property type
    pub kind: RentalPropertyType,
    /// Original building source date
    pub building: RentalRecoveryDate,
    /// Improvement source dates
    pub more: RentalRecoveryMoreDates,
}

/// Applicable choices for one actual building or improvement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalRecoveryAllowedRow {
    /// Actual applicable historical choices
    #[libertas_read_only]
    pub allowed_commercial_1993_transition: Vec<i32>,
}

/// Applicable choices for the current improvement rows
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalRecoveryMoreAllowed {
    /// Existing improvement rows, in their current order
    pub improvements: Vec<RentalRecoveryAllowedRow>,
}

/// Applicable choices projected into the same current rental
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalRecoveryAllowed {
    /// Original building choices
    pub building: RentalRecoveryAllowedRow,
    /// Improvement choices
    pub more: RentalRecoveryMoreAllowed,
}

pub(crate) fn applicable(kind: &RentalPropertyType, placed: u32) -> bool {
    kind.nonresidential()
        && (19930513..=19931231).contains(&placed)
        && crate::federal_rules::valid_date(placed)
}

pub(crate) fn allowed(
    kind: &RentalPropertyType,
    location: &crate::RentalLocation,
    placed: u32,
) -> Vec<i32> {
    if !location.foreign() && applicable(kind, placed) {
        vec![
            libertas_macros::variant_index!(Commercial1993Transition::Ordinary39Year) as i32,
            libertas_macros::variant_index!(Commercial1993Transition::BindingContract) as i32,
            libertas_macros::variant_index!(Commercial1993Transition::ConstructionBegun) as i32,
        ]
    } else {
        Vec::new()
    }
}

pub(crate) fn choices(input: &RentalRecoveryInputs) -> RentalRecoveryAllowed {
    let row = |date: &RentalRecoveryDate| RentalRecoveryAllowedRow {
        allowed_commercial_1993_transition: allowed(&input.kind, &input.location, date.in_service),
    };
    RentalRecoveryAllowed {
        building: row(&input.building),
        more: RentalRecoveryMoreAllowed {
            improvements: input.more.improvements.iter().map(row).collect(),
        },
    }
}

// A choice names an actual qualifying documentary source; no relationship or
// favorable recovery classification is inferred from the service date alone.
pub(crate) fn source_rule(
    kind: &RentalPropertyType,
    placed: u32,
    source: Option<&Commercial1993Transition>,
) -> Option<ValidationIssue> {
    let field = "commercial_1993_transition";
    if !applicable(kind, placed) {
        return source.map(|_| {
            ValidationIssue::new("RENTAL_1993_TRANSITION_NOT_APPLICABLE").at_field(field)
        });
    }
    let fact = source_fact(kind, placed, source);
    let result = fact.check_reported(|source| source_bounds(source));
    result.err()
}

pub(crate) fn source_fact<'a>(
    kind: &RentalPropertyType,
    placed: u32,
    source: Option<&'a Commercial1993Transition>,
) -> SourceFact<&'a Commercial1993Transition> {
    if !applicable(kind, placed) {
        SourceFact::NotReported
    } else {
        source.map_or(SourceFact::Unavailable, SourceFact::Reported)
    }
}

fn source_bounds(source: &Commercial1993Transition) -> Result<(), ValidationIssue> {
    let field = "commercial_1993_transition";
    let (date, date_field, reference) = match source {
        Commercial1993Transition::Ordinary39Year => return Ok(()),
        Commercial1993Transition::BindingContract(crate::Commercial1993ContractSource {
            effective,
            source_reference,
        }) => (*effective, "effective", source_reference),
        Commercial1993Transition::ConstructionBegun(crate::Commercial1993ConstructionSource {
            began,
            source_reference,
        }) => (*began, "began", source_reference),
    };
    if !crate::federal_rules::valid_date(date) || date >= 19930513 {
        return Err(ValidationIssue::new("RENTAL_1993_TRANSITION_SOURCE_DATE")
            .at_field(date_field)
            .in_field(field));
    }
    if reference.trim().is_empty() {
        return Err(ValidationIssue::new("RENTAL_1993_TRANSITION_SOURCE_REFERENCE")
            .at_field("source_reference")
            .in_field(field));
    }
    Ok(())
}

pub(crate) fn page_issue(value: &Rental) -> Option<ValidationIssue> {
    if value.location.foreign() {
        if value.building.commercial_1993_transition.is_some() {
            return Some(
                ValidationIssue::new("RENTAL_1993_TRANSITION_NOT_APPLICABLE")
                    .at_field("commercial_1993_transition")
                    .in_field("building"),
            );
        }
        return value.more.improvements.iter().enumerate().find_map(|(index, row)| {
            row.commercial_1993_transition.as_ref().map(|_| {
                ValidationIssue::new("RENTAL_1993_TRANSITION_NOT_APPLICABLE")
                    .at_field("commercial_1993_transition")
                    .in_list("improvements", index)
                    .in_field("more")
            })
        });
    }
    if let Some(issue) = source_rule(
        &value.kind,
        value.building.in_service,
        value.building.commercial_1993_transition.as_ref(),
    ) {
        return Some(issue.in_field("building"));
    }
    for (index, row) in value.more.improvements.iter().enumerate() {
        if let Some(issue) =
            source_rule(&value.kind, row.in_service, row.commercial_1993_transition.as_ref())
        {
            return Some(issue.in_list("improvements", index).in_field("more"));
        }
    }
    None
}
