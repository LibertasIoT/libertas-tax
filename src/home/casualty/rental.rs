// Coverage: Form4684 SectionB partial, rental buildings/improvements. Pure-use
// ceased losses and kept damage with actual REG/AMT ledgers feed the existing
// risk/PAL/ordinary/NIIT/QBI owners once, without a ScheduleE expense copy.
// The dated recovery owner retains the original remaining depreciation clock.
// Mixed/history, salvage, gains, nonrecurringPAL and homecarry remain named
// boundaries. Tests: tests::home::rental_casualty, including native lifecycle.
// Sources: i4684 SectionB, Pub547, Pub946; Reg.1.469-2(d)(2)(xi), Pub527 ch5.
use crate::kernel::source_fact::SourceFact;
use crate::{FederalDraft, FederalIncome, FederalResult, RentalBuilding};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Casualty or theft of this property
/// Enter the property damage and insurance records, not a computed tax loss.
/// Inventory losses belong in cost of goods sold. Restoration costs belong on
/// their actual improvement or asset record, separately from the loss.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PropertyCasualty {
    /// Event
    /// For example, the March windstorm or the theft from this building.
    #[libertas_size(min = 1)]
    pub event: alloc::string::String,
    /// Date of the casualty, or discovery of the theft
    /// An earlier event belongs here when its insurance claim was resolved this year. A next-year federally declared disaster may be elected onto this return instead of the next; it must not be deducted twice.
    #[libertas_date_only]
    pub date: u32,
    /// Disaster declaration covering this event
    pub declaration: super::CasualtyDeclaration,
    /// What happened to this property
    pub kind: CasualtyKind,
    /// Insurance or other reimbursement received or expected
    /// Include all claims, whether or not you filed them.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub reimbursement: i64,
    /// Part received during this return's tax year
    /// An expected future payment does not establish a current realized gain.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub received: i64,
    /// Cost of eligible replacement property bought or planned
    /// Zero when no replacement is planned. Use the actual similar-use property
    /// records and the ordinary replacement deadline; an extension or disputed
    /// qualification needs its separate worksheet.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub replacement_cost: i64,
}

/// What happened
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CasualtyKind {
    /// Totally destroyed
    Destroyed,
    /// Stolen
    Stolen,
    /// Damaged but not totally destroyed
    Damaged {
        /// Fair market value immediately before
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        before: i64,
        /// Fair market value immediately after
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        after: i64,
        /// Permanent retirement from rental use
        /// Enter the actual date when the property was permanently withdrawn. Temporary repair downtime while held for rental is not retirement.
        /// [NullText]
        /// Still held for rental at year end
        /// [SetText]
        /// Permanently retired from rental use on
        #[libertas_date_only]
        retired: Option<u32>,
    },
}

/// Casualty or theft of this rental building or improvement
/// The event-date property facts and supported loss worksheet can be shown.
/// Unresolved activity limits, gain character or continuing depreciation
/// leave the complete estimate unavailable.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalCasualty {
    /// This property's event and reimbursement records
    pub casualty: PropertyCasualty,
    /// Losses similar in cause and severity recur regularly in this rental
    /// The passive-activity worksheet needs this actual history. No does not
    /// turn the casualty loss into an ordinary rental operating expense.
    pub recurring: bool,
    /// Event-date records for damaged property still held for rental
    /// Use the actual regular-tax and AMT ledgers. Basis and through-event
    /// depreciation establish the event-date loss basis; the prior-year
    /// amount establishes this year's continuing depreciation. Do not enter
    /// a second computed loss.
    /// Leave unavailable when the records do not establish an amount. A
    /// property permanently retired on the event date uses its original
    /// supported recovery schedule instead.
    pub continuing_records: alloc::boxed::Box<RentalCasualtyRecords>,
}

/// Actual basis and depreciation records for this property
/// Use the same ownership share as the entered building cost, before any
/// community-property allocation by the App. These are the property's ledger
/// amounts, not amounts prorated by this year's personal-use days.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct RentalCasualtyRecords {
    /// Regular-tax records
    pub regular: RentalCasualtyBasis,
    /// Alternative minimum tax records
    /// Use the AMT ledger even when its amounts equal the regular-tax amounts.
    pub alternative: RentalCasualtyBasis,
}

/// Actual amounts on the same depreciation ledger
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalCasualtyBasis {
    /// Adjusted loss basis immediately before the event
    /// The ledger's adjusted basis after depreciation through the event and
    /// before this casualty's loss or reimbursement adjustment.
    #[libertas_default(Unavailable)]
    pub basis: crate::SourceCents,
    /// Depreciation allowed or allowable through the event
    /// The cumulative amount for this property on this same tax ledger.
    #[libertas_default(Unavailable)]
    pub depreciation: crate::SourceCents,
    /// Depreciation allowed or allowable before this return's tax year
    /// The cumulative amount before January 1, excluding this year's amount.
    #[libertas_default(Unavailable)]
    pub prior_depreciation: crate::SourceCents,
}

impl Default for RentalCasualtyBasis {
    fn default() -> Self {
        Self {
            basis: crate::SourceCents::Unavailable,
            depreciation: crate::SourceCents::Unavailable,
            prior_depreciation: crate::SourceCents::Unavailable,
        }
    }
}

impl RentalCasualtyBasis {
    fn known(source: &crate::SourceCents) -> Option<i64> {
        source.source_fact().ok()?.require(|| ()).ok().map(|value| value.unwrap_or(0))
    }

    fn amount(source: &crate::SourceCents) -> Result<i64, &'static str> {
        source
            .source_fact()
            .map_err(|_| "RENTAL_CASUALTY_DEPRECIATION_RECORDS")?
            .require(|| "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE")
            .map(|value| value.unwrap_or(0))
    }

    fn amounts(&self) -> Result<(i64, i64, i64), &'static str> {
        Ok((
            Self::amount(&self.basis)?,
            Self::amount(&self.depreciation)?,
            Self::amount(&self.prior_depreciation)?,
        ))
    }
}

impl RentalCasualty {
    pub(crate) fn issue(&self, original_basis: i64) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some(issue) = self.casualty.issue() {
            return Some(issue.in_field("casualty"));
        }
        for (system, records) in [
            ("regular", &self.continuing_records.regular),
            ("alternative", &self.continuing_records.alternative),
        ] {
            for (field, amount) in [
                ("basis", &records.basis),
                ("depreciation", &records.depreciation),
                ("prior_depreciation", &records.prior_depreciation),
            ] {
                if amount.source_fact().is_err() {
                    return Some(
                        ValidationIssue::new("RENTAL_CASUALTY_AMOUNT")
                            .at_field(field)
                            .in_field(system)
                            .in_field("continuing_records"),
                    );
                }
            }
            let known = RentalCasualtyBasis::known;
            let invalid_prior =
                match (known(&records.prior_depreciation), known(&records.depreciation)) {
                    (Some(prior), Some(through)) => prior > through,
                    _ => false,
                };
            let invalid_basis = match (known(&records.basis), known(&records.depreciation)) {
                (Some(basis), Some(through)) => {
                    basis.checked_add(through).is_none_or(|sum| sum > original_basis)
                }
                _ => false,
            };
            if invalid_prior || invalid_basis {
                return Some(
                    ValidationIssue::new("RENTAL_CASUALTY_DEPRECIATION_RECORDS")
                        .at_field(if invalid_prior { "prior_depreciation" } else { "basis" })
                        .in_field(system)
                        .in_field("continuing_records"),
                );
            }
        }
        None
    }
}

impl PropertyCasualty {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.event.trim().is_empty() {
            return Some(ValidationIssue::new("RENTAL_CASUALTY_EVENT_REQUIRED").at_field("event"));
        }
        if !crate::federal_rules::valid_date(self.date) {
            return Some(ValidationIssue::new("RENTAL_CASUALTY_DATE").at_field("date"));
        }
        if !self.declaration.valid() {
            return Some(
                ValidationIssue::new("RENTAL_CASUALTY_DECLARATION").at_field("declaration"),
            );
        }
        for (value, field) in [
            (self.reimbursement, "reimbursement"),
            (self.received, "received"),
            (self.replacement_cost, "replacement_cost"),
        ] {
            if value < 0 {
                return Some(ValidationIssue::new("RENTAL_CASUALTY_AMOUNT").at_field(field));
            }
        }
        if self.received > self.reimbursement {
            return Some(
                ValidationIssue::new("CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT").at_field("received"),
            );
        }
        if let CasualtyKind::Damaged { before, after, retired } = self.kind {
            if retired.is_some_and(|date| !crate::federal_rules::valid_date(date)) {
                return Some(
                    ValidationIssue::new("RENTAL_CASUALTY_DATE")
                        .at_field("retired")
                        .in_field("kind"),
                );
            }
            for (value, field) in [(before, "before"), (after, "after")] {
                if value < 0 {
                    return Some(
                        ValidationIssue::new("RENTAL_CASUALTY_AMOUNT")
                            .at_field(field)
                            .in_field("kind"),
                    );
                }
            }
            if after > before {
                return Some(
                    ValidationIssue::new("CASUALTY_AFTER_ABOVE_BEFORE")
                        .at_field("after")
                        .in_field("kind"),
                );
            }
        }
        None
    }
    pub(crate) fn year_issue(&self, tax_year: i32) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let year = u32::try_from(tax_year).ok().filter(|year| (1..=9999).contains(year));
        let Some(year) = year else {
            return Some(ValidationIssue::new("RENTAL_CASUALTY_DATE").at_field("date"));
        };
        // Section165(i) permits the immediately following disaster year.
        // This is the accepted return's actual year, never an editor input.
        if self.date / 10000 > year + 1 {
            return Some(ValidationIssue::new("RENTAL_CASUALTY_DATE").at_field("date"));
        }
        if self.date / 10000 == year + 1
            && !matches!(
                self.declaration,
                super::CasualtyDeclaration::Federal { .. }
                    | super::CasualtyDeclaration::MajorDisaster { .. }
            )
        {
            return Some(
                ValidationIssue::new("RENTAL_CASUALTY_ELECTION_DECLARATION")
                    .at_field("declaration"),
            );
        }
        None
    }
}

// Embedded records need no new persisted association. The accepted income ID
// and its current building/improvement field scope locate this invocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocatedRentalCasualty {
    pub income: i64,
    pub building: usize,
    pub owner: crate::Owner,
    pub source: RentalCasualty,
    pub recurring: bool,
    pub kept: bool,
    pub long_term: SourceFact<bool>,
    pub regular: SourceFact<SectionBItem>,
    pub alternative: SourceFact<SectionBItem>,
    pub boundary: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SectionBItem {
    pub basis_cents: i64,
    pub gain_basis_cents: i64,
    pub depreciation_cents: i64,
    pub reimbursement_cents: i64,
    pub received_cents: i64,
    pub fair_values_cents: Option<[i64; 2]>,
    pub limited_cents: i64,
    pub raw_loss_cents: i64,
    pub continuing_basis_cents: SourceFact<i64>,
    pub realized_gain_cents: i64,
    pub recognized_gain_cents: i64,
    pub postponed_gain_cents: i64,
    // SectionB line27 before activity limits. Pure business property has no
    // section280A limit; mixed-use property needs its actual home worksheet.
    // This amount does not establish the allowed Form4797 loss.
    pub admitted_loss_cents: SourceFact<i64>,
    pub ordinary_recapture_cents: SourceFact<i64>,
    pub unrecaptured_gain_cents: SourceFact<i64>,
}

fn owned_at_event(
    building: &RentalBuilding,
    rental: &crate::FederalRental,
    source: &PropertyCasualty,
) -> bool {
    source.date >= building.in_service
        && rental.sale.date().is_none_or(|sale| source.date < sale)
        && match source.kind {
            CasualtyKind::Damaged { retired: Some(date), .. } => {
                crate::federal_rules::valid_date(date) && date >= source.date
            }
            _ => true,
        }
}

fn item(
    building: &RentalBuilding,
    rental: &crate::FederalRental,
    records: &RentalCasualty,
    amt: bool,
    tax_year: i32,
) -> Result<SectionBItem, &'static str> {
    if let Some(issue) = records.issue(building.cost.saturating_sub(building.land)) {
        return Err(issue.key);
    }
    if let Some(issue) = records.casualty.year_issue(tax_year) {
        return Err(issue.key);
    }
    if !owned_at_event(building, rental, &records.casualty)
        || !rental.ordinary_rental
        || !matches!(rental.personal_use, crate::RentalPersonalUse::No)
    {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    source_item(building, records, amt, &rental.property_type, tax_year)
}

// Full-property source worksheet. Rental use, ownership at the event and
// subsequent activity settlement remain with the rental-context adapter.
fn source_item(
    building: &RentalBuilding,
    records: &RentalCasualty,
    amt: bool,
    property_type: &crate::RentalPropertyType,
    tax_year: i32,
) -> Result<SectionBItem, &'static str> {
    if let Some(issue) = records.issue(building.cost.saturating_sub(building.land)) {
        return Err(issue.key);
    }
    let source = &records.casualty;
    if let Some(issue) = source.year_issue(tax_year) {
        return Err(issue.key);
    }
    if i32::try_from(source.date / 10000).ok() != Some(tax_year) {
        return Err("RENTAL_CASUALTY_EVENT_YEAR_UNAVAILABLE");
    }
    if source.date < building.in_service {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    // Pub527's excepted-property rule: placement and retirement in one year
    // cannot use this owner's ordinary MACRS schedule to establish event basis.
    if source.date / 10000 == building.in_service / 10000 {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    // accumulated currently folds through calendar2026. Other return years
    // need that owner's year-specific schedule; do not compute them as2026.
    if tax_year != 2026 {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    // The gift's donor adjusted basis does not establish its gift-date FMV
    // required for loss basis (section1015(a)); no new input is invented here.
    if matches!(building.acquired, crate::RentalAcquisition::Gift) {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    // The current year's day fraction cannot establish earlier allowed or
    // allowable depreciation. Sale-date history is not event-date history.
    if building.partial.is_some() {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    // Pub547 reports theft in its discovery year. That date does not prove
    // when service actually ended under Pub946; never stop its schedule there.
    if matches!(source.kind, CasualtyKind::Stolen) {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    // Pub946, "Basis adjustment due to casualty loss": a survivor's adjusted
    // annual schedule is not a disposition stopped at the event month. Actual
    // event-basis records and that continuation owner must establish its basis.
    if matches!(source.kind, CasualtyKind::Damaged { retired: Some(date), .. } if date != source.date)
    {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    }
    let cost = building
        .cost
        .checked_sub(building.land)
        .ok_or("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE")?;
    let loss_cost = match building.acquired {
        crate::RentalAcquisition::Converted { fair_market_value } => cost.min(fair_market_value),
        _ => cost,
    };
    let (basis, gain_basis, depreciation) =
        if matches!(source.kind, CasualtyKind::Damaged { retired: None, .. }) {
            if building.credit_reduction != 0 || building.rehabilitation.is_some() {
                // A credit amount alone cannot establish the casualty's
                // coordinated recapture and remaining-basis settlement.
                return Err("RENTAL_CASUALTY_CREDIT_UNAVAILABLE");
            }
            // These are actual event-date records, not a fictitious cessation
            // schedule. A converted property's distinct gain basis is not
            // established by its loss-basis ledger alone.
            if loss_cost != cost {
                return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
            }
            let history = if amt {
                &records.continuing_records.alternative
            } else {
                &records.continuing_records.regular
            };
            let basis = RentalCasualtyBasis::amount(&history.basis)?;
            let through = RentalCasualtyBasis::amount(&history.depreciation)?;
            (basis, basis, through)
        } else {
            let depreciation =
                crate::asset_sale::accumulated(building, amt, property_type, source.date)
                    .map_err(|_| "RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE")?;
            let reduction = building.rehabilitation_net_reduction(Some(source.date));
            let basis = loss_cost
                .checked_sub(reduction)
                .and_then(|v| v.checked_sub(depreciation))
                .filter(|v| *v >= 0)
                .ok_or("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE")?;
            let gain_basis = cost
                .checked_sub(reduction)
                .and_then(|v| v.checked_sub(depreciation))
                .filter(|v| *v >= 0)
                .ok_or("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE")?;
            (basis, gain_basis, depreciation)
        };
    // Pub547, Business or income-producing property: a destroyed item's
    // loss also subtracts actual salvage. Destroyed alone does not establish
    // zero salvage, and this approved source has no such input. Damaged
    // property instead supplies its actual before/after FMV.
    if matches!(source.kind, CasualtyKind::Destroyed) {
        return Err("RENTAL_CASUALTY_SALVAGE_UNAVAILABLE");
    }
    let fair_values = match source.kind {
        CasualtyKind::Damaged { before, after, .. } => Some([before, after]),
        _ => None,
    };
    let loss = super::home::property_loss(
        basis,
        fair_values.map(|[before, after]| before - after),
        source.reimbursement,
    )
    .map_err(|_| "RENTAL_CASUALTY_OUTPUT_UNAVAILABLE")?;
    let conversion =
        super::home::conversion(source.received, gain_basis, 0, Some(source.replacement_cost))
            .map_err(|_| "RENTAL_CASUALTY_OUTPUT_UNAVAILABLE")?;
    Ok(SectionBItem {
        basis_cents: basis,
        gain_basis_cents: gain_basis,
        depreciation_cents: depreciation,
        reimbursement_cents: source.reimbursement,
        received_cents: source.received,
        fair_values_cents: fair_values,
        limited_cents: loss.limited_cents,
        raw_loss_cents: loss.loss_cents,
        // Pub547's basis adjustment uses received reimbursement. Expected
        // payments establish the uncovered loss, not an actual closing basis.
        continuing_basis_cents: if source.received == source.reimbursement {
            SourceFact::Reported(loss.continuing_basis_cents)
        } else {
            SourceFact::Unavailable
        },
        realized_gain_cents: conversion.gain_after_exclusion_cents,
        recognized_gain_cents: conversion.recognized_cents,
        postponed_gain_cents: conversion.postponed_cents,
        admitted_loss_cents: SourceFact::Reported(loss.loss_cents),
        ordinary_recapture_cents: if conversion.gain_after_exclusion_cents == 0 {
            SourceFact::NotReported
        } else {
            SourceFact::Unavailable
        },
        unrecaptured_gain_cents: if conversion.gain_after_exclusion_cents == 0 {
            SourceFact::NotReported
        } else {
            SourceFact::Unavailable
        },
    })
}

// The canonical annual building owner invokes this source adapter with its
// actual prepared basis. The same SectionB worksheet establishes the loss
// and closing basis; the kernel alone owns the remaining original clock.
pub(crate) fn continuing_reduction(
    building: &RentalBuilding,
    amt: bool,
    property_type: &crate::RentalPropertyType,
    year: u32,
    original_basis: i64,
) -> Result<Option<crate::historical_recovery::DatedBasisReduction>, &'static str> {
    let Some(records) = &building.casualty else { return Ok(None) };
    if records.casualty.date / 10000 > year
        || neutral_event(&records.casualty)
        || !matches!(records.casualty.kind, CasualtyKind::Damaged { retired: None, .. })
    {
        return Ok(None);
    }
    let facts = source_item(
        building,
        records,
        amt,
        property_type,
        i32::try_from(year).map_err(|_| "RENTAL_CASUALTY_EVENT_YEAR_UNAVAILABLE")?,
    )?;
    if let Some(issue) = records.issue(original_basis) {
        return Err(issue.key);
    }
    if facts.realized_gain_cents != 0 {
        return Err("RENTAL_CASUALTY_GAIN_UNAVAILABLE");
    }
    let SourceFact::Reported(adjusted_basis_cents) = facts.continuing_basis_cents else {
        return Err("RENTAL_CASUALTY_BASIS_HISTORY_UNAVAILABLE");
    };
    let history = if amt {
        &records.continuing_records.alternative
    } else {
        &records.continuing_records.regular
    };
    let (_, depreciation_through_event_cents, prior_depreciation_cents) = history.amounts()?;
    Ok(Some(crate::historical_recovery::DatedBasisReduction {
        event_date: records.casualty.date,
        adjusted_basis_cents,
        depreciation_through_event_cents,
        prior_depreciation_cents,
    }))
}

// The rental owner retains the normal original clock and disposition rounding.
// A whole-property cessation stops depreciation at the event, independently of
// whether the remaining casualty loss can pass its later activity limitations.
// Surviving damage needs the building engine's dated adjusted-basis interface;
// neither rewritten cost nor a credit-reduction field represents that history.
pub(crate) fn current_depreciation(
    building: &RentalBuilding,
    rental: &crate::FederalRental,
    amt: bool,
    year: u32,
    energy_basis_reduction: SourceFact<i64>,
) -> Result<i64, &'static str> {
    let source_year = i32::try_from(year).map_err(|_| "RENTAL_CASUALTY_EVENT_YEAR_UNAVAILABLE")?;
    let Some(source) = &building.casualty else {
        return Err("RENTAL_CASUALTY_OUTPUT_UNAVAILABLE");
    };
    if let Some(issue) = source.casualty.issue().or_else(|| source.casualty.year_issue(source_year))
    {
        return Err(issue.key);
    }
    if !matches!(rental.sale, crate::RentalSale::No) {
        return Err("RENTAL_CASUALTY_DISPOSITION_UNAVAILABLE");
    }
    // Actual unchangedFMV and no insurance establish no basis adjustment
    // without an inferred event-date depreciation amount. The original
    // annual owner still determines this property's depreciation.
    if neutral_event(&source.casualty) {
        return crate::extended_rules::depreciation_for_year(
            building,
            amt,
            &rental.property_type,
            year,
            None,
            energy_basis_reduction,
        )
        .map_err(|(key, _)| key);
    }
    item(building, rental, source, amt, source_year)?;
    // Keep the producer's known basis facts, while withholding a complete
    // result until the credit owner supplies casualty-specific settlement.
    if building.rehabilitation_recapture(Some(source.casualty.date)).is_some() {
        return Err("RENTAL_CASUALTY_CREDIT_UNAVAILABLE");
    }
    crate::extended_rules::depreciation_for_year(
        building,
        amt,
        &rental.property_type,
        year,
        match source.casualty.kind {
            CasualtyKind::Damaged { retired: None, .. } => None,
            _ => Some(source.casualty.date),
        },
        energy_basis_reduction,
    )
    .map_err(|(key, _)| key)
}

// The kept-property proof reads actual source facts, never a loss derived
// from a cessation schedule standing in for unavailable survivor history.
fn neutral_event(source: &PropertyCasualty) -> bool {
    source.issue().is_none()
        && source.reimbursement == 0
        && source.received == 0
        && matches!(source.kind, CasualtyKind::Damaged { before, after, retired: None } if before == after)
}

fn neutral(source: &LocatedRentalCasualty) -> bool {
    neutral_event(&source.source.casualty)
        || (!source.kept
            && [&source.regular, &source.alternative].into_iter().all(|fact| {
                matches!(fact, SourceFact::Reported(item)
                if item.raw_loss_cents == 0 && item.realized_gain_cents == 0)
            }))
}

// Recurring casualties remain in their activity under Reg.1.469-2(d)(2)(xi).
// For the currently supported loss-only SectionB pool, its net loss is ordinary
// (Pub547 / i4684 PartII). A gain still needs the common casualty netting and
// recapture owners; an exceptional nonrecurring loss needs its PAL exclusion.
// The existing rental property source owns all subsequent risk/PAL settlement.
pub(crate) fn property_losses(
    rental: &crate::FederalRental,
    amt: bool,
) -> Result<Vec<(usize, i64)>, &'static str> {
    let mut losses = Vec::new();
    for (index, building) in rental.buildings.iter().enumerate() {
        let Some(source) = &building.casualty else { continue };
        if neutral_event(&source.casualty) {
            current_depreciation(
                building,
                rental,
                amt,
                2026,
                if building.energy.is_none() {
                    SourceFact::NotReported
                } else {
                    SourceFact::Unavailable
                },
            )?;
            continue;
        }
        let facts = item(building, rental, source, amt, 2026)?;
        if facts.realized_gain_cents != 0 {
            return Err("RENTAL_CASUALTY_GAIN_UNAVAILABLE");
        }
        if facts.raw_loss_cents == 0 {
            continue;
        }
        // Pub527 ch5 retains280A's income limit on prior home expenses
        // even when this year's use is purely rental. The existing carry
        // room does not yet include this separate casualty deduction.
        if rental.home_operating_carryover != 0
            || crate::rental_home_carry::total(rental, amt) != Some(0)
        {
            return Err("RENTAL_CASUALTY_HOME_CARRY_UNAVAILABLE");
        }
        if !source.recurring {
            return Err("RENTAL_CASUALTY_NONRECURRING_UNAVAILABLE");
        }
        current_depreciation(
            building,
            rental,
            amt,
            2026,
            if building.energy.is_none() {
                SourceFact::NotReported
            } else {
                SourceFact::Unavailable
            },
        )?;
        losses.push((index, facts.raw_loss_cents));
    }
    Ok(losses)
}

pub(crate) fn items(draft: &FederalDraft) -> Vec<LocatedRentalCasualty> {
    let mut result = Vec::new();
    for accepted in &draft.income {
        if !matches!(accepted.income, FederalIncome::Rental { .. }) {
            continue;
        }
        let entry = crate::community::property_entry(draft, accepted);
        let FederalIncome::Rental { data } = &entry.income else { unreachable!() };
        for (index, building) in data.buildings.iter().enumerate() {
            let Some(source) = &building.casualty else { continue };
            let regular = item(building, data, source, false, draft.tax_year);
            let alternative = item(building, data, source, true, draft.tax_year);
            let date = source.casualty.date;
            // Inherited property has its independent long-term rule. A theft
            // discovery date or a donor's service date cannot prove the actual
            // holding period of this claimant's loss-basis property.
            let long_term = if !crate::federal_rules::valid_date(date)
                || !owned_at_event(building, data, &source.casualty)
            {
                SourceFact::Unavailable
            } else if matches!(building.acquired, crate::RentalAcquisition::Inherited) {
                SourceFact::Reported(true)
            } else if matches!(source.casualty.kind, CasualtyKind::Stolen)
                || matches!(building.acquired, crate::RentalAcquisition::Gift)
            {
                SourceFact::Unavailable
            } else if (super::CasualtyAcquisition::Purchased { date: building.in_service })
                .long_term(date)
            {
                SourceFact::Reported(true)
            } else {
                SourceFact::Unavailable
            };
            let boundary =
                regular.as_ref().err().or(alternative.as_ref().err()).copied().or(matches!(
                    long_term,
                    SourceFact::Unavailable
                )
                .then_some("RENTAL_CASUALTY_HOLDING_PERIOD_UNAVAILABLE"));
            result.push(LocatedRentalCasualty {
                income: entry.id,
                building: index,
                owner: entry.owner,
                source: source.clone(),
                recurring: source.recurring,
                kept: matches!(source.casualty.kind, CasualtyKind::Damaged { retired, .. } if retired != Some(source.casualty.date)),
                long_term,
                regular: regular.map_or(SourceFact::Unavailable, SourceFact::Reported),
                alternative: alternative.map_or(SourceFact::Unavailable, SourceFact::Reported),
                boundary,
            });
        }
    }
    result
}

pub(crate) fn report(draft: &FederalDraft, result: &mut FederalResult) {
    use crate::kernel::lines::{LineValue, row};
    let sources = items(draft);
    if sources.is_empty() {
        return;
    }
    let source_complete = |source: &LocatedRentalCasualty| {
        if draft.tax_year != 2026 {
            return false;
        }
        if neutral(source) {
            return true;
        }
        // These are the exact source cases routed below through the existing
        // property/activity owners. This does not presume those owners pass.
        source.recurring
            && [&source.regular, &source.alternative].into_iter().all(
                |fact| matches!(fact, SourceFact::Reported(item) if item.realized_gain_cents == 0),
            )
            && draft.income.iter().find(|entry| entry.id == source.income).is_some_and(|entry| {
                let FederalIncome::Rental { data } = &entry.income else { return false };
                let building = &data.buildings[source.building];
                [false, true].into_iter().all(|amt| {
                    current_depreciation(
                        building,
                        data,
                        amt,
                        2026,
                        if building.energy.is_none() {
                            SourceFact::NotReported
                        } else {
                            SourceFact::Unavailable
                        },
                    )
                    .is_ok()
                })
            })
    };
    let needs_consumer = sources.iter().any(|source| !source_complete(source));
    // Ordinary rental evaluation already reports any depreciation boundary.
    // A neutral, fully computed event does not itself invalidate that result.
    if needs_consumer {
        crate::federal_rules::withhold_required_source_result(result);
        result.financial_source_scope_complete = false;
    }
    for source in sources {
        let context = crate::IssueContext::income_id(source.income);
        for (prefix, facts) in [("", &source.regular), ("amt_", &source.alternative)] {
            let SourceFact::Reported(facts) = facts else { continue };
            let base =
                alloc::format!("{prefix}rental_{}_casualty_{}", source.income, source.building);
            let dual_basis = facts.basis_cents != facts.gain_basis_cents;
            let mut part_i = if dual_basis {
                // A converted property's loss and gain bases can differ.
                // Preserve both as source facts, never one fictitious line20.
                alloc::vec![("21", facts.reimbursement_cents)]
            } else {
                alloc::vec![
                    ("20", facts.basis_cents),
                    ("21", facts.reimbursement_cents),
                    ("22", facts.realized_gain_cents),
                    ("26", facts.limited_cents),
                ]
            };
            if let Some([before, after]) = facts.fair_values_cents {
                part_i.extend([("23", before), ("24", after), ("25", before - after)]);
            }
            if let SourceFact::Reported(loss) = facts.admitted_loss_cents {
                part_i.push(("27", loss));
            }
            part_i.sort_by_key(|(line, _)| *line);
            for (line, cents) in part_i {
                result.lines.push(row(
                    Some(crate::FormLine::new(crate::TaxForm::Form4684, line)),
                    &alloc::format!("{base}_line{line}"),
                    LineValue::MoneyDollars(crate::kernel::ratio::dollars(cents)),
                    &["income.rental.buildings.casualty"],
                    "https://www.irs.gov/instructions/i4684",
                    msg!("EXPLAIN_RENTAL_CASUALTY_SOURCE"),
                ));
            }
            let mut source_rows = alloc::vec![("raw_loss", facts.raw_loss_cents)];
            if let SourceFact::Reported(basis) = facts.continuing_basis_cents {
                source_rows.push(("continuing_basis", basis));
            }
            if dual_basis {
                source_rows.extend([
                    ("loss_basis", facts.basis_cents),
                    ("gain_basis", facts.gain_basis_cents),
                    ("realized_gain", facts.realized_gain_cents),
                ]);
            }
            for (suffix, cents) in source_rows {
                result.lines.push(row(
                    None,
                    &alloc::format!("{base}_{suffix}"),
                    LineValue::MoneyCents(cents),
                    &["income.rental.buildings.casualty"],
                    "https://www.irs.gov/instructions/i4684",
                    msg!("EXPLAIN_RENTAL_CASUALTY_SOURCE"),
                ));
            }
        }
        if source_complete(&source)
            || result.issues.iter().any(|issue| {
                issue.display.key.starts_with("RENTAL_CASUALTY_")
                    && issue.context.identity() == context.identity()
            })
        {
            continue;
        }
        let key = source.boundary.unwrap_or(
            if [&source.regular, &source.alternative].into_iter().any(
                |fact| matches!(fact, SourceFact::Reported(item) if item.realized_gain_cents != 0),
            ) {
                "RENTAL_CASUALTY_GAIN_UNAVAILABLE"
            } else if !source.recurring && !neutral(&source) {
                "RENTAL_CASUALTY_NONRECURRING_UNAVAILABLE"
            } else {
                "RENTAL_CASUALTY_CONTINUATION_UNAVAILABLE"
            },
        );
        if result
            .issues
            .iter()
            .any(|issue| issue.display.key == key && issue.context.identity() == context.identity())
        {
            continue;
        }
        crate::issue_localized!(
            result,
            &context,
            "Rental casualty",
            key,
            "This casualty needs its event-date basis records and completed Section B, activity and continuing-depreciation worksheets before the complete estimate is available."
        );
    }
}

crate::community::divide_record!(PropertyCasualty {
    amounts: [reimbursement, received, replacement_cost, kind],
    gains: [],
    facts: [event, date, declaration],
});

impl crate::community::Divide for CasualtyKind {
    fn divided(&self, share: crate::community::Share) -> Self {
        match self {
            Self::Damaged { before, after, retired } => Self::Damaged {
                before: before.divided(share),
                after: after.divided(share),
                retired: *retired,
            },
            other => other.clone(),
        }
    }
}
impl RentalCasualtyBasis {
    fn divided_at(&self, share: crate::community::Share, land: i64) -> Self {
        use crate::community::Divide;
        use crate::kernel::ratio::exact;
        if [&self.basis, &self.depreciation, &self.prior_depreciation]
            .into_iter()
            .any(|source| source.source_fact().is_err())
        {
            return self.clone(); // Invalid input remains invalid, never a rounded zero.
        }
        let mut divided = Self {
            basis: self.basis.divided(share),
            depreciation: self.depreciation.divided(share),
            prior_depreciation: self.prior_depreciation.divided(share),
        };
        let (Some(basis), Some(through)) =
            (Self::known(&self.basis), Self::known(&self.depreciation))
        else {
            return divided;
        };
        let prior = Self::known(&self.prior_depreciation);
        if land < 0
            || prior.is_some_and(|prior| prior > through)
            || crate::kernel::ratio::checked_sum([land, basis, through]).is_none()
        {
            return self.clone();
        }
        let projected = |source: &crate::SourceCents, amount| match source {
            crate::SourceCents::Reported { .. } => crate::SourceCents::Reported { amount },
            other => other.clone(),
        };
        let mut prefix = exact::ExactPrefix::default();
        let mut next = |amount| {
            prefix.next(&exact::whole(amount), |sum| {
                exact::rounded(&share.filer_exact(sum)).expect("checked source total fits i64")
            })
        };
        // Anchor at the separately projected land. Thus linked basis plus
        // through-event depreciation fits projected cost minus projected land.
        next(land);
        divided.basis = projected(&self.basis, next(basis));
        divided.depreciation = projected(
            &self.depreciation,
            if let Some(prior) = prior {
                let allocated_prior = next(prior);
                divided.prior_depreciation = projected(&self.prior_depreciation, allocated_prior);
                allocated_prior + next(through - prior)
            } else {
                next(through)
            },
        );
        divided
    }
}

impl RentalCasualty {
    pub(crate) fn divided_at(&self, share: crate::community::Share, land: i64) -> Self {
        use crate::community::Divide;
        Self {
            casualty: self.casualty.divided(share),
            recurring: self.recurring,
            continuing_records: alloc::boxed::Box::new(RentalCasualtyRecords {
                regular: self.continuing_records.regular.divided_at(share, land),
                alternative: self.continuing_records.alternative.divided_at(share, land),
            }),
        }
    }
}

impl crate::community::Divide for RentalCasualty {
    fn divided(&self, share: crate::community::Share) -> Self {
        self.divided_at(share, 0)
    }
}
