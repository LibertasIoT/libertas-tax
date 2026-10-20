// Coverage: Form 4684 — Partial (audited 2026-09-24; line audit fix F13 2026-10-10).
//   Lines: Section A personal-use only: per-event $100/$500 floors, federal/state declared vs
//   undeclared vs qualified major disaster, gain offset, 10% AGI floor, holding period by
//   acquisition, standard-deduction route. Refused: Section B business/income property, Section C
//   Ponzi, Section D, prior-year election; excess-gain event with mixed holding periods. An
//   event whose records are not confirmed (source_complete No) deducts no loss and
//   keeps its gains, with WARN_CASUALTY_LOSS_LEFT_OUT.
//   Evidence: M/casualty.rs, federal_rules.rs form_4684_*; T/investment.rs
//   personal_casualty_applies_per_event_and_shared_agi_limits,
//   undeclared_personal_casualty_loss_only_offsets_recognized_gains; IRS i4684, draft f4684.
// T3-deduct (2026-09-28): asked on the deductions page's "Casualty and theft losses (Form 4684)"
//   list (always in the walk: a qualified loss would survive the standard deduction), each event
//   this record as entered. Client rules with server twins: CASUALTY_ZIP,
//   CASUALTY_AFTER_ABOVE_BEFORE, repeated saved identities and CASUALTY_RECEIVED_ABOVE_REIMBURSEMENT.
//   Linked ordinary business-home losses use the HOME/Section B owner; other Section B-D sources remain unavailable. Tests:
//   src/tests/deductions/deduction_pages.rs a_declared_disaster_loss_reaches_schedule_a_through_the_deductions_page,
//   casualty_floors_and_the_standard_route, casualty_rules_are_refused_on_both_sides.
// F13 (2026-10-10, line audit 4684-D1, D2): P.L. 119-108 section 2 (Sept. 11, 2026) replaced
//   P.L. 119-21 section 70438: a loss in a presidentially declared major disaster (FEMA DR-) whose
//   incident period began on or after December 28, 2019 and before January 1, 2027 is a qualified
//   disaster loss (section 165(h)(6)), so a 2026 DR- event takes the $500 floor (line 11, when its
//   line 10 losses exceed the line 4 gains of all the events, else $100), no 10% AGI reduction
//   (line 15 to Schedule A line 17b) and adds to the standard deduction (section 63(b)(8),
//   Form 1040 line 12e). CasualtyDeclaration::MajorDisaster is that choice; Federal is an
//   emergency (EM-) or fire (FM-) declaration, which does not qualify. Line 4 gains use only the
//   reimbursement received in 2026 (CasualtyProperty.received); line 3 uses everything received
//   or expected. Tests: src/tests/cross/line_audit_f13.rs fixtures 4684-D1 and 4684-D2.

// Native HOME prerequisite (2026-10-14): casualty::home owns Section B
// property loss/basis, characterized third-tier settlement and consumption of
// the HOME owner's established section121 exclusion before section1033.
// Existing personal losses/gains and home-office aggregate rows reuse these
// owners with unchanged figures. This source interface does not introduce
// business-casualty inputs, the all-personal attribution worksheet, HOME
// eligibility or R22/R28 integration; their original refusals remain.
// Tests: src/tests/home/unit42_home_source.rs; IRS i4684, i8829, Pub547 and Pub587.
//
// 2026 personal-use Section A of Form 4684.
// https://www.irs.gov/pub/irs-dft/f4684--dft.pdf
use crate::federal_rules::valid_date;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, positive};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) mod home;
pub(crate) mod rental;

/// Source of a 2026 disaster declaration
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CasualtyDeclaration {
    /// Not a declared disaster
    /// The loss only offsets casualty gains, such as insurance above your basis.
    Undeclared,
    /// Federally declared major disaster (FEMA DR-)
    /// A presidential major-disaster declaration whose incident period began on or after December 28, 2019 and before January 1, 2027, which includes any major disaster that began in 2026. The loss is a qualified disaster loss: the reduction is $500 instead of $100, the 10% of income reduction does not apply, and it adds to the standard deduction if you do not itemize.
    MajorDisaster {
        /// FEMA major-disaster declaration number
        /// For example, DR-4999.
        #[libertas_size(min = 4)]
        reference: String,
    },
    /// Other federally declared disaster or emergency
    /// An emergency (FEMA EM-) or fire declaration (FM-). A major disaster (DR-) is the choice above.
    Federal {
        /// FEMA declaration number
        /// For example, EM-3999.
        #[libertas_size(min = 1)]
        reference: String,
    },
    /// State-declared disaster
    State {
        /// Governor's declaration and state
        #[libertas_size(min = 1)]
        reference: String,
    },
}
impl CasualtyDeclaration {
    fn valid(&self) -> bool {
        match self {
            Self::Undeclared => true,
            // Section 165(h)(6)(C)(ii): only a major disaster declared by the
            // President under section 401 of the Stafford Act (a DR- number)
            // makes a qualified disaster area; an emergency declaration does not.
            Self::MajorDisaster { reference } => reference.trim().starts_with("DR-"),
            Self::Federal { reference } => {
                !reference.trim().is_empty() && !reference.trim().starts_with("DR-")
            }
            Self::State { reference } => !reference.trim().is_empty(),
        }
    }
}

/// How the personal-use property was acquired for Form 4684 holding-period treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CasualtyAcquisition {
    /// Purchased property with its own holding period
    Purchased {
        /// Date acquired
        #[libertas_date_only]
        date: u32,
    },
    /// Inherited property with section 1014 basis and a deemed long-term holding period
    /// Confirm the estate basis records; property subject to a 2010 section 1022/Form 8939 election needs another worksheet.
    InheritedSection1014 {
        /// Date acquired from the decedent
        #[libertas_date_only]
        date: u32,
    },
}
impl CasualtyAcquisition {
    fn date(&self) -> u32 {
        match self {
            Self::Purchased { date } | Self::InheritedSection1014 { date } => *date,
        }
    }
    fn long_term(&self, event_date: u32) -> bool {
        match self {
            Self::InheritedSection1014 { .. } => true,
            Self::Purchased { date } => {
                // Holding starts the day after acquisition; the event day is
                // included. More than a calendar year is long term.
                crate::dates::held_more_than_one_year(*date, event_date)
            }
        }
    }
}

/// Property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct CasualtyProperty {
    /// Saved property identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Saved home represented by this property
    /// [NullText]
    /// This is not a saved business home
    /// [SetText]
    /// This property is the saved home
    pub home_site: Option<i64>,
    /// Business property damage records for a saved home
    /// Leave inactive for property that is not linked to a saved business home. A zero value after the event does not by itself establish total destruction.
    /// [NullText]
    /// This property is not linked to a saved business home
    /// [SetText]
    /// What happened to the business part of this home
    pub business_damage: Option<HomeCasualtyDamage>,
    /// Property
    /// For example, a car, a roof, or furniture.
    #[libertas_size(min = 1)]
    pub label: String,
    /// City and state where it was
    #[libertas_size(min = 1)]
    pub location: String,
    /// Acquisition basis and holding-period source
    pub acquisition: CasualtyAcquisition,
    /// Adjusted basis before the disaster
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
    /// Fair market value immediately before the disaster
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub before: i64,
    /// Fair market value immediately after the disaster
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub after: i64,
    /// Insurance or other reimbursement received or expected
    /// Everything you received or expect to receive for this property, whether or not you filed a claim. It reduces the loss.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub reimbursement: i64,
    /// Part of it you received during this return’s tax year
    /// Only what you actually received during this return’s tax year can produce a gain here. A payment you expect after that year is a gain in the year you receive it, not on this return.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub received: i64,
}
/// What the records establish about the business property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeCasualtyDamage {
    /// Damaged but not totally destroyed
    Damaged,
    /// Totally destroyed
    Destroyed,
    /// Stolen
    Stolen,
    /// The records needed to establish this are unavailable
    /// The saved source remains on the return; its business casualty result is unavailable until the records establish the damage.
    Unavailable,
}

impl CasualtyProperty {
    pub(crate) fn business_decrease(&self) -> crate::kernel::source_fact::SourceFact<Option<i64>> {
        use crate::kernel::source_fact::SourceFact;
        match self.business_damage {
            None => SourceFact::NotReported,
            Some(HomeCasualtyDamage::Unavailable) => SourceFact::Unavailable,
            Some(HomeCasualtyDamage::Damaged) => {
                SourceFact::Reported(Some(self.before - self.after))
            }
            Some(HomeCasualtyDamage::Destroyed | HomeCasualtyDamage::Stolen) => {
                SourceFact::Reported(None)
            }
        }
    }

    fn valid(&self, event_date: u32) -> bool {
        !self.label.trim().is_empty()
            && !self.location.trim().is_empty()
            && valid_date(self.acquisition.date())
            && self.acquisition.date() <= event_date
            && [self.basis, self.before, self.after, self.reimbursement].iter().all(|v| *v >= 0)
            && self.after <= self.before
            && (0..=self.reimbursement).contains(&self.received)
    }
    fn loss(&self) -> i64 {
        home::property_loss(self.basis, Some(self.before - self.after), self.reimbursement)
            .expect("validated casualty basis, FMV decrease and reimbursement are nonnegative")
            .loss_cents
    }
    // Line 4: the gain uses only the reimbursement actually received in 2026
    // (i4684 line 4); a payment still to come is a gain in the year received.
    fn gain(&self) -> i64 {
        // Until HOME supplies a linked conversion, the existing no-exclusion,
        // no-postponement source is retained. This does not invent HOME facts.
        home::conversion(self.received, self.basis, 0, None)
            .expect("zero exclusion and no replacement always preserve the realized gain")
            .recognized_cents
    }
}

/// Event
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct PersonalCasualty {
    /// Saved event identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Event
    /// For example, the March windstorm.
    #[libertas_size(min = 1)]
    pub label: String,
    /// Date of the casualty, or the date you discovered the theft
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Disaster declaration covering this property and event
    #[libertas_default(Undeclared)]
    pub declaration: CasualtyDeclaration,
    /// ZIP code of the property most affected
    /// Required for a declared disaster; leave blank otherwise.
    // Legal identifier: a five-digit ZIP code when supplied.
    #[libertas_size(max = 5)]
    #[libertas_pattern(r"([0-9]{5})?")]
    pub zip: String,
    /// Personal-use property lost or damaged
    /// ----
    /// Property
    #[libertas_size(min = 1)]
    pub properties: Vec<CasualtyProperty>,
    /// Complete Form 4684 Section A records for this return confirmed
    /// No leaves this event's loss out of the estimate; a gain from reimbursement above basis is still included. For a declared event, confirm the declaration and incident period cover the event and each property's location. For every event, all basis, value and insurance-claim records are complete; no unentered casualty/theft event, prior-year election, unlinked business/investment property, postponed gain/replacement election, different AMT basis or pending reimbursement dispute applies. For inherited property, confirm section 1014 basis and no 2010 section 1022/Form 8939 election. Gifted and other carryover-basis holding periods need another worksheet. The major-disaster option requires a presidential major-disaster declaration (DR-), not an emergency-only declaration.
    /// Check the insurance settlement, property-basis records and disaster declaration.
    pub source_complete: bool,
}
impl PersonalCasualty {
    pub(crate) fn valid(&self) -> bool {
        !self.label.trim().is_empty()
            && (20260101..=20261231).contains(&self.date)
            && valid_date(self.date)
            && self.declaration.valid()
            && ((!self.declared() && self.zip.is_empty())
                || (self.zip.len() == 5 && self.zip.bytes().all(|b| b.is_ascii_digit())))
            && !self.properties.is_empty()
            && self.properties.iter().all(|v| v.valid(self.date))
            && Self::sources_representable(core::slice::from_ref(self))
    }
    // Unbounded source collections must still have representable native
    // totals before the existing whole-dollar Section A/gain readers run.
    pub(crate) fn sources_representable(events: &[Self]) -> bool {
        let mut loss_total = 0i64;
        let mut gain_total = 0i64;
        for property in events.iter().flat_map(|event| &event.properties) {
            if [
                property.basis,
                property.before,
                property.after,
                property.reimbursement,
                property.received,
            ]
            .iter()
            .any(|value| *value < 0)
                || property.after > property.before
                || property.received > property.reimbursement
            {
                return false;
            }
            let Some(loss) = loss_total.checked_add(property.loss()) else {
                return false;
            };
            let Some(gain) = gain_total.checked_add(property.gain()) else {
                return false;
            };
            loss_total = loss;
            gain_total = gain;
        }
        true
    }
    // The event's loss after its $100/$500 reduction. An event whose records
    // are not confirmed deducts no loss; its gains still count. `gains` is the
    // sum of line 4 of all the events (i4684 line 11): a qualified disaster
    // event whose losses (line 10) are larger takes $500, otherwise $100.
    pub(crate) fn after_event_limit(&self, gains: i64) -> i64 {
        if !self.source_complete {
            return 0;
        }
        let loss = self.properties.iter().map(CasualtyProperty::loss).sum::<i64>();
        let floor = if self.qualified() && dollars(loss) > gains { 50_000 } else { 10_000 };
        dollars(positive(loss - floor))
    }
    pub(crate) fn qualified(&self) -> bool {
        matches!(self.declaration, CasualtyDeclaration::MajorDisaster { .. })
    }
    pub(crate) fn declared(&self) -> bool {
        !matches!(self.declaration, CasualtyDeclaration::Undeclared)
    }
    pub(crate) fn gain(&self) -> i64 {
        dollars(self.properties.iter().map(CasualtyProperty::gain).sum())
    }

    /// Form 4684 line 15 excess gain routed to Schedule D lines 4 and 11.
    /// An event with after-event losses in both holding periods requires a
    /// separate allocation of its single $100/$500 reduction.
    pub(crate) fn capital_gains(events: &[Self]) -> Result<[i64; 2], (&'static str, &'static str)> {
        let gains = events.iter().map(Self::gain).sum::<i64>();
        let losses = events.iter().map(|event| event.after_event_limit(gains)).sum::<i64>();
        if gains <= losses {
            return Ok([0, 0]);
        }
        let mut gain_cents = [0i64; 2];
        let mut loss_dollars = [0i64; 2];
        for event in events {
            let event_after_floor_loss = event.after_event_limit(gains);
            let mut event_loss_terms = [false; 2];
            for property in &event.properties {
                let term = usize::from(property.acquisition.long_term(event.date));
                gain_cents[term] += property.gain();
                if event_after_floor_loss > 0 && property.loss() > 0 {
                    event_loss_terms[term] = true;
                }
            }
            if event_loss_terms == [true, true] {
                return Err((
                    "ISSUE_CASUALTY_MIXED_TERM_EVENT_REDUCTION",
                    crate::fixture_english!(
                        "An excess-gain Form 4684 event with short- and long-term property losses needs allocation of its single $100 or $500 event reduction."
                    ),
                ));
            }
            if event_loss_terms[0] {
                loss_dollars[0] += event_after_floor_loss;
            } else if event_loss_terms[1] {
                loss_dollars[1] += event_after_floor_loss;
            }
        }
        let by_term =
            [dollars(gain_cents[0]) - loss_dollars[0], dollars(gain_cents[1]) - loss_dollars[1]];
        if by_term[0] + by_term[1] != gains - losses {
            return Err((
                "ISSUE_CASUALTY_MIXED_TERM_ROUNDING",
                crate::fixture_english!(
                    "Mixed-term personal casualty gains need a Form 4684 whole-dollar allocation worksheet."
                ),
            ));
        }
        Ok(by_term)
    }
}

// Form4684 Section A in native form dollars. The same owner is used for the
// filed personal portion and the all-personal worksheet required by i8829
// line9. The caller supplies the actual event values and the appropriate AGI;
// HOME's hypothetical AGI excludes its business-use income and deductions.
// No attribution among homes or casualty events is inferred by this summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SectionAResult {
    pub gains: i64,
    pub declared_after_events: i64,
    pub undeclared_after_events: i64,
    pub qualified_after_events: i64,
    pub after_events: i64,
    pub net: i64,
    pub qualified_net: i64,
    pub ordinary_deduction: i64,
    pub deduction: i64,
}

pub(crate) fn section_a(events: &[PersonalCasualty], agi: i64) -> SectionAResult {
    let gains = events.iter().map(PersonalCasualty::gain).sum::<i64>();
    let declared_after_events = events
        .iter()
        .filter(|event| event.declared())
        .map(|event| event.after_event_limit(gains))
        .sum::<i64>();
    let undeclared_after_events = events
        .iter()
        .filter(|event| !event.declared())
        .map(|event| event.after_event_limit(gains))
        .sum::<i64>();
    let qualified_after_events = events
        .iter()
        .filter(|event| event.qualified())
        .map(|event| event.after_event_limit(gains))
        .sum::<i64>();
    settle_section_a(
        gains,
        declared_after_events,
        undeclared_after_events,
        qualified_after_events,
        crate::kernel::ratio::fraction(positive(agi), 1, 10),
    )
}

fn settle_section_a(
    gains: i64,
    declared_after_events: i64,
    undeclared_after_events: i64,
    qualified_after_events: i64,
    agi_floor: i64,
) -> SectionAResult {
    // Form4684 line14/Worksheet1-1: undeclared losses use only current
    // casualty gains; they do not create an itemized deduction or reduce
    // the declared-disaster loss.
    let after_events = declared_after_events + phase::cap(undeclared_after_events, [gains]);
    let net = positive(after_events - gains);
    // Lines15-18 absorb gains with ordinary losses first, preserving the
    // smaller of the remaining net loss and the qualified-loss subtotal.
    let qualified_net = phase::cap(qualified_after_events, [net]);
    let ordinary_deduction = positive(net - qualified_net - agi_floor);
    SectionAResult {
        gains,
        declared_after_events,
        undeclared_after_events,
        qualified_after_events,
        after_events,
        net,
        qualified_net,
        ordinary_deduction,
        deduction: qualified_net + ordinary_deduction,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PropertyAllowance {
    pub event: i64,
    pub property: i64,
    pub home_site: Option<i64>,
    pub ordinary_cents: i64,
    pub qualified_cents: i64,
    pub available: bool,
}

// D-2026-10-17-04: shared event reductions first, then the shared AGI/gain
// reductions, allocated proportionally by net loss in original input order.
// This hypothetical worksheet stays in cents; the actual personal form keeps
// its existing event-level whole-dollar rounding through section_a().
pub(crate) fn all_personal_allowances(
    events: &[PersonalCasualty],
    modified_agi_cents: i64,
) -> Result<Vec<PropertyAllowance>, crate::kernel::carry::LedgerError> {
    use crate::kernel::carry::LedgerError;
    use crate::kernel::ratio::{allocate_remaining, checked_sum};
    if !PersonalCasualty::sources_representable(events) {
        return Err(LedgerError::InvalidAmount);
    }
    let sum = |values: Vec<i64>| checked_sum(values).ok_or(LedgerError::InvalidAmount);
    let gains = sum(events
        .iter()
        .flat_map(|event| &event.properties)
        .map(CasualtyProperty::gain)
        .collect())?;
    let mut qualified = Vec::new();
    let mut ordinary = Vec::new();
    let mut undeclared = 0i64;
    let mut rows = Vec::new();
    for event in events {
        let weights: Vec<_> = event
            .properties
            .iter()
            .map(|property| if event.source_complete { property.loss() } else { 0 })
            .collect();
        let loss = sum(weights.clone())?;
        let floor = if event.qualified() && loss > gains { 50_000 } else { 10_000 };
        let reductions = allocate_remaining(loss.min(floor), &weights);
        for ((property, weight), reduction) in event.properties.iter().zip(weights).zip(reductions)
        {
            let net = weight - reduction;
            rows.push(PropertyAllowance {
                event: event.id,
                property: property.id,
                home_site: property.home_site,
                ordinary_cents: 0,
                qualified_cents: 0,
                available: event.source_complete,
            });
            qualified.push(if event.qualified() { net } else { 0 });
            ordinary.push(if event.declared() && !event.qualified() { net } else { 0 });
            if !event.declared() {
                undeclared = undeclared.checked_add(net).ok_or(LedgerError::InvalidAmount)?;
            }
        }
    }
    let qualified_total = sum(qualified.clone())?;
    let ordinary_total = sum(ordinary.clone())?;
    let declared = qualified_total.checked_add(ordinary_total).ok_or(LedgerError::InvalidAmount)?;
    let summary = settle_section_a(
        gains,
        declared,
        undeclared,
        qualified_total,
        crate::kernel::ratio::fraction(positive(modified_agi_cents), 1, 10),
    );
    let qualified_reductions =
        allocate_remaining(qualified_total - summary.qualified_net, &qualified);
    let ordinary_reductions =
        allocate_remaining(ordinary_total - summary.ordinary_deduction, &ordinary);
    for (index, row) in rows.iter_mut().enumerate() {
        row.qualified_cents = qualified[index] - qualified_reductions[index];
        row.ordinary_cents = ordinary[index] - ordinary_reductions[index];
    }
    Ok(rows)
}

mod page;
pub use page::{CasualtyHomeLinkPage, CasualtyPropertyPage, PersonalCasualtyPage};
