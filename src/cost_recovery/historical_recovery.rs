// Canonical historical MACRS building schedules for rental owners.
// Coverage: ordinary nonresidential GDS first available1987–May12,1993
// continues31.5years for annual/source accumulation and GDSUBIA clocks.
// Rental tests: unit29_rental_recovery and unit29_rental_transition.
// Actual 1993 documentary transitions share this period owner with ordinary GDS.
// Required foreign ADS retains30/40-year clocks;179D keeps its own source.
// Original dated rental records continue their own recovery clock.
// No current annual-rate override, new editor or old-office stored field.
// Sources: Pub.946; PL103-66 section13151(b), 107Stat448.
use crate::kernel::cost_recovery::historical as engine;
pub(crate) use engine::{DatedBasisReduction, RentalBuildingRecovery};
pub use engine::{OfficeRecoveryDeduction, OfficeRecoveryIssue};

pub(crate) fn rental_building_recovery(
    kind: &crate::RentalPropertyType,
    location: &crate::RentalLocation,
    placed: u32,
    source: Option<&crate::Commercial1993Transition>,
    amt: bool,
) -> Result<RentalBuildingRecovery, (&'static str, &'static str)> {
    if !crate::federal_rules::valid_date(placed) {
        return Err((
            "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS",
            "Review the original building service date.",
        ));
    }
    if placed < 19870101 && location.foreign() {
        return Err((
            "RENTAL_FOREIGN_RECOVERY_HISTORY_UNAVAILABLE",
            "Supply the original historical foreign-property recovery records.",
        ));
    }
    if placed < 19870101 {
        return Err((
            "ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS",
            "Review the original building recovery method.",
        ));
    }
    if location.foreign() {
        if source.is_some() {
            return Err((
                "RENTAL_1993_TRANSITION_NOT_APPLICABLE",
                "Domestic commercial transition records do not establish foreign ADS.",
            ));
        }
        return Ok(if !kind.nonresidential() && placed >= 20180101 {
            RentalBuildingRecovery::Ads30
        } else {
            RentalBuildingRecovery::Ads40
        });
    }
    if let Some(issue) = crate::rental_recovery::source_rule(kind, placed, source) {
        return Err((issue.key, "Review the actual 1993 commercial recovery source."));
    }
    // AMT's original pre-1999 ADS40 period is independently established;
    // only regular GDS needs the documentary 31.5-versus39 source.
    if amt && placed < 19990101 {
        return Ok(RentalBuildingRecovery::Ads40);
    }
    let source = crate::rental_recovery::source_fact(kind, placed, source).require(|| (
        "RENTAL_1993_RECOVERY_SOURCE_REQUIRED",
        "Supply the actual 1993 commercial contract or construction source to establish recovery.",
    ))?;
    Ok(if !kind.nonresidential() {
        RentalBuildingRecovery::Residential275
    } else if placed < 19930513
        || matches!(
            source,
            Some(
                crate::Commercial1993Transition::BindingContract(_)
                    | crate::Commercial1993Transition::ConstructionBegun(_)
            )
        )
    {
        RentalBuildingRecovery::Commercial315
    } else {
        RentalBuildingRecovery::Commercial39
    })
}

pub(crate) fn rental_recovery_window(
    recovery: RentalBuildingRecovery,
    placed: u32,
    year: u32,
) -> u32 {
    if !crate::federal_rules::valid_date(placed) || !(1..=9999).contains(&year) {
        return 0;
    }
    engine::rental_recovery_window(recovery, placed, year)
}

pub(crate) fn rental_building_amount(
    recovery: RentalBuildingRecovery,
    basis: i64,
    placed: u32,
    year: u32,
    window: u32,
    reduction: Option<DatedBasisReduction>,
) -> Result<OfficeRecoveryDeduction, OfficeRecoveryIssue> {
    if !crate::federal_rules::valid_date(placed)
        || !(1..=9999).contains(&year)
        || window > 24
        || reduction.is_some_and(|r| !crate::federal_rules::valid_date(r.event_date))
    {
        return Err(OfficeRecoveryIssue::InvalidDate);
    }
    engine::rental_building_amount(recovery, basis, placed, year, window, reduction)
}
