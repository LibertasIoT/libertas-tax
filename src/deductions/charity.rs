pub(crate) use crate::charity_sources::similar_eq;
pub use crate::charity_sources::{
    AcquiredBy, Appraisal, CharityCarryover, CharityClass, ConservationPurpose, DoneeKind,
    Form1098CDocument, Form1098CPaper, Form1098CTreatment, GiftKind, GoodsCondition,
    HistoricBuildingGift, PropertyCategory, PropertyGift, SimilarPropertyKind, ValueMethod,
    VehicleGiftSource,
};
use crate::validation::ValidationIssue;
use crate::{GoodsKind, VehicleUse};
// Coverage: charity classes, property-source facts and the common limitation worksheet.
// IRS Pub526, Form8283 instructions and IRC170; tax year2026.
// One class worksheet owns current gifts/K1 amounts, oldest-first original
// balances, the percentage limits, the floor, ScheduleA11-15 and8283 money.
// Appraisal IDs stay in the canonical stored gift; the protocol-only wrapper
// supplies the page catalog without persisting it. Unsupported partial/debt/
// business gifts, changed-filer history and named NOL/FTC historical worksheet
// combinations retain their source-specific notes or issues.
// Section170(d)(1)(B) class carry reporting follows the existing NOL owner's
// exact aggregate endpoint through finite class-priority absorption passes;
// public-charity sources precede other organizations (Reg1.170A-10(d)(1)).
use crate::kernel::{
    carry, phase,
    ratio::{dollars, fraction, positive},
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

impl Form1098CPaper {
    pub(crate) fn natural(&self) -> Result<Form1098C, ValidationIssue> {
        let treatment = self.vehicle_use()?;
        Ok(Form1098C::from((self, &treatment)))
    }
}

/// Form 1098-C
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this one Form 1098-C (Contributions of Motor Vehicles, Boats, and Airplanes) exactly. Check its calendar year against the supplied year; report a mismatch. Return the donee's name as printed, box 1 date of contribution as the date given, boxes 2b, 2c and 2d (year, make and model) together as one description text and box 3 vehicle identification number; a blank, illegible or conflicting value is an error. For what the charity did with it: when box 4a is checked, return Sold with the box 4b date of sale and box 4c gross proceeds, returning 500 for the proceeds only when box 7 is checked and box 4c shows no amount; when box 5b is checked, return NeedyPerson; when box 5a is checked, return SignificantUse or MaterialImprovement as the box 5c description states, with that description as printed. If none or more than one of boxes 4a, 5a and 5b is checked, or box 5c does not say which use applies, report an error. When box 6a is Yes, return box 6b as the value received; when box 6a is No, return the value received as blank. Do not infer the kind of charity, when or for how much the filer bought the vehicle, its fair market value or any fact absent from the form. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Form1098C {
    /// DONEE'S name
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub charity: String,
    /// Box 1 Date of contribution
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub given: u32,
    /// Boxes 2b-2d Year, make and model
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub description: String,
    /// Box 3 Vehicle or other identification number
    #[libertas_llm_doc_field]
    // Legal identifier: Form 1098-C box 3 admits a 17-character VIN or a shorter hull or aircraft identifier.
    #[libertas_size(min = 1, max = 17)]
    pub vin: String,
    /// Boxes 4 and 5 What the charity did with it
    #[libertas_llm_doc_field]
    pub treatment: VehicleUse,
    /// Box 6b Value of goods and services provided in exchange for the vehicle
    /// Goods or services the charity gave you for this gift (a dinner, tickets).
    /// [NullText]
    /// I received nothing in return for this gift
    /// [SetText]
    /// The goods or services I received were worth
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    #[libertas_default($null)]
    pub benefit_received: Option<i64>,
}
impl From<(&Form1098CPaper, &VehicleUse)> for Form1098C {
    fn from((paper, treatment): (&Form1098CPaper, &VehicleUse)) -> Self {
        let Form1098CPaper {
            calendar_year: _,
            corrected: _,
            donee,
            donee_telephone: _,
            donor: _,
            box_1,
            box_2a: _,
            box_2b,
            box_2c,
            box_2d,
            box_3,
            box_4a: _,
            box_4b: _,
            box_4c,
            box_5a: _,
            box_5b: _,
            box_5c,
            box_6a_yes,
            box_6a_no: _,
            box_6b,
            box_6c_description: _,
            box_6c_intangible_religious_benefits: _,
            box_7,
            treatment: _,
        } = paper;
        let treatment = match treatment {
            VehicleUse::Sold { sold, proceeds: _ } => VehicleUse::Sold {
                sold: *sold,
                // Preserve the existing box 7 treatment of an unprinted sale amount.
                proceeds: if *box_7 && *box_4c == 0 { 50_000 } else { *box_4c },
            },
            VehicleUse::NeedyPerson => VehicleUse::NeedyPerson,
            VehicleUse::SignificantUse { details: _ } => {
                VehicleUse::SignificantUse { details: box_5c.clone() }
            }
            VehicleUse::MaterialImprovement { details: _ } => {
                VehicleUse::MaterialImprovement { details: box_5c.clone() }
            }
        };
        // Odometer mileage, identity metadata, certifications and the box 6c
        // religious-benefit flag have no additional calculation in this model.
        let year = box_2b.map(|year| alloc::format!("{year}")).unwrap_or_default();
        Self {
            charity: donee.name.clone(),
            given: *box_1,
            description: [year.as_str(), box_2c.as_str(), box_2d.as_str()]
                .into_iter()
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
            vin: box_3.clone(),
            treatment,
            benefit_received: if *box_6a_yes { Some(*box_6b) } else { None },
        }
    }
}

/// Vehicle, boat and airplane contributions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Identify each Form 1098-C contribution statement. Keep all pages for one gift together and count duplicate copies once. Read the printed form and its certified treatment; do not infer acquisition, basis, fair market value, appraisal or other personal-record answers."
)]
pub struct VehicleGiftDocuments {
    /// Documents
    #[libertas_unordered]
    /// ----
    /// Document
    pub documents: Vec<Form1098CDocument>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleGiftConversionError(ValidationIssue);
impl VehicleGiftConversionError {
    pub(crate) fn issue(self) -> ValidationIssue {
        self.0
    }
}
impl TryFrom<(&VehicleGiftSource, &Form1098CPaper)> for PropertyGift {
    type Error = VehicleGiftConversionError;

    fn try_from(
        (source, form): (&VehicleGiftSource, &Form1098CPaper),
    ) -> Result<Self, Self::Error> {
        let VehicleGiftSource {
            id: _,
            donee,
            how_acquired,
            acquired,
            basis,
            value,
            value_method,
            appraisal,
            in_scope,
            donee_acknowledged,
        } = source;
        let natural =
            form.natural().map_err(|issue| VehicleGiftConversionError(issue.in_field("form")))?;
        let Form1098C { charity, given, description, vin, treatment, benefit_received } = &natural;
        Ok(PropertyGift {
            description: description.clone(),
            charity: charity.clone(),
            donee: *donee,
            given: *given,
            how_acquired: *how_acquired,
            acquired: *acquired,
            basis: *basis,
            value: *value,
            value_method: *value_method,
            benefit_received: *benefit_received,
            appraisal: *appraisal,
            in_scope: *in_scope,
            donee_acknowledged: *donee_acknowledged,
            kind: GiftKind::Vehicle { vin: vin.clone(), treatment: treatment.clone() },
        })
    }
}
impl crate::FederalDeductions {
    /// Resolve the printed source by its accepted identity.
    pub(crate) fn vehicle_gift(
        &self,
        source: &VehicleGiftSource,
    ) -> Result<PropertyGift, ValidationIssue> {
        let document = self
            .vehicle_gift_documents
            .iter()
            .find(|document| document.id == source.id)
            .ok_or_else(|| ValidationIssue::new("VEHICLE_GIFT_DOCUMENT_NEEDED").at_field("id"))?;
        PropertyGift::try_from((source, &document.form)).map_err(VehicleGiftConversionError::issue)
    }
    /// Resolve entered property gifts for deductions and carryovers.
    pub(crate) fn gifts(&self) -> Result<Vec<PropertyGift>, ValidationIssue> {
        for (index, document) in self.vehicle_gift_documents.iter().enumerate() {
            if !self.vehicle_gifts.iter().any(|source| source.id == document.id) {
                return Err(ValidationIssue::new("VEHICLE_GIFT_FACTS_NEEDED")
                    .at_record()
                    .in_list("vehicle_gift_documents", index));
            }
        }
        let mut gifts = self.property_gifts.clone();
        for (index, source) in self.vehicle_gifts.iter().enumerate() {
            gifts.push(
                self.vehicle_gift(source).map_err(|issue| issue.in_list("vehicle_gifts", index))?,
            );
        }
        Ok(gifts)
    }
}

/// Gift
/// One property contribution and the appraisal selected for it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PropertyGiftPage {
    /// Property contribution
    pub gift: PropertyGift,
    /// Appraisal
    /// Choose an entered appraisal when Section B applies. A single clothing or household item over $500 not in good used condition also needs one.
    /// [NullText]
    /// No appraisal is available for this gift
    /// [SetText]
    /// This gift relies on appraisal
    #[libertas_enum_source("^<GiftCarryovers>.appraisal_choices")]
    #[libertas_default($null)]
    pub appraisal: Option<i64>,
}
impl From<PropertyGift> for PropertyGiftPage {
    fn from(mut gift: PropertyGift) -> Self {
        let appraisal = gift.appraisal.take();
        Self { gift, appraisal }
    }
}
impl PropertyGiftPage {
    pub(crate) fn stored(&self) -> PropertyGift {
        let mut gift = self.gift.clone();
        gift.appraisal = self.appraisal;
        gift
    }
}

/// Vehicle, boat or airplane gift
/// An accepted Form 1098-C and the personal facts still needed for its deduction.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct VehicleGiftPage {
    /// Accepted Form 1098-C
    /// Edit the printed boxes in the Document center.
    #[libertas_read_only]
    pub document: Form1098CDocument,
    /// Your answers about this gift
    /// [NullText]
    /// Personal facts have not been supplied
    /// [SetText]
    /// Personal facts for this contribution
    #[libertas_default($null)]
    pub gift: Option<VehicleGiftSource>,
    /// Appraisal
    /// Choose an entered appraisal when Section B applies.
    /// [NullText]
    /// No appraisal is available for this gift
    /// [SetText]
    /// This gift relies on appraisal
    #[libertas_enum_source("^<GiftCarryovers>.appraisal_choices")]
    #[libertas_default($null)]
    pub appraisal: Option<i64>,
}
impl VehicleGiftPage {
    pub(crate) fn stored(&self) -> Option<VehicleGiftSource> {
        self.gift.clone().map(|mut gift| {
            gift.id = self.document.id;
            gift.appraisal = self.appraisal;
            gift
        })
    }
}

// All amounts from this point are projected whole dollars. Allocation within
// the tax source's class is kept separate from the shared carry ledger.
#[derive(Clone, Copy, Default)]
pub(crate) struct ClassClaims {
    pub(crate) current: [i64; 8],
    pub(crate) cash: [i64; 8],
}
#[derive(Clone, Copy, Default)]
pub(crate) struct ClassResult {
    pub(crate) claims: i64,
    pub(crate) allowed: i64,
    pub(crate) current_allowed: i64,
    pub(crate) floor: i64,
    pub(crate) current_floor: i64,
    pub(crate) current_carry: i64,
    pub(crate) current_excess: i64,
    pub(crate) current_cash_deduction: i64,
}
#[derive(Default)]
pub(crate) struct CarryResult {
    pub(crate) used: i64,
    pub(crate) floor: i64,
    pub(crate) closing: i64,
    pub(crate) expired: i64,
}
pub(crate) struct Worksheet {
    pub(crate) classes: [ClassResult; 8],
    pub(crate) carryovers: Vec<CarryResult>,
}
impl Worksheet {
    pub(crate) fn calculate(
        base: i64,
        claims: ClassClaims,
        carryovers: &[CharityCarryover],
        basis_election: bool,
    ) -> Self {
        let base = positive(base);
        let mut out = Self {
            classes: [ClassResult::default(); 8],
            carryovers: (0..carryovers.len()).map(|_| CarryResult::default()).collect(),
        };
        let mut origins: Vec<_> = carryovers
            .iter()
            .enumerate()
            .filter(|(_, c)| c.same_filers)
            .map(|(i, c)| {
                let elected = basis_election && matches!(c.class, CharityClass::CapitalGain30);
                let class = if elected { CharityClass::Property50 } else { c.class };
                let value = if elected {
                    positive(dollars(c.balance) - dollars(c.appreciation.unwrap_or(0)))
                } else {
                    dollars(c.balance)
                };
                (i, class, c.year, value)
            })
            .collect();
        origins.sort_by_key(|v| v.2);
        for class in CharityClass::ALL {
            let i = class.index();
            out.classes[i].claims = claims.current[i]
                + origins.iter().filter(|v| v.1 == class).map(|v| v.3).sum::<i64>();
        }
        for class in CharityClass::ALL {
            let i = class.index();
            let a = &out.classes;
            let cap = match class {
                CharityClass::Property50 => fraction(base, 1, 2),
                CharityClass::Cash60 => positive(fraction(base, 3, 5) - a[0].allowed),
                CharityClass::Other30 => phase::cap(
                    fraction(base, 3, 10),
                    [positive(
                        fraction(base, 1, 2)
                            - phase::cap(a[0].claims + a[3].claims, [fraction(base, 1, 2)])
                            - a[1].allowed
                            + phase::cap(a[1].allowed, [fraction(base, 1, 10)]),
                    )],
                ),
                CharityClass::CapitalGain30 => phase::cap(
                    fraction(base, 3, 10),
                    [positive(fraction(base, 1, 2) - a[0].allowed - a[1].allowed)],
                ),
                CharityClass::CapitalGain20 => phase::cap(
                    fraction(base, 1, 5),
                    [
                        positive(fraction(base, 3, 10) - a[3].allowed),
                        positive(fraction(base, 3, 10) - a[2].allowed),
                        positive(
                            fraction(base, 1, 2) - a[..4].iter().map(|v| v.allowed).sum::<i64>(),
                        ),
                    ],
                ),
                CharityClass::Conservation => {
                    positive(fraction(base, 1, 2) - a[..5].iter().map(|v| v.allowed).sum::<i64>())
                }
                CharityClass::Conservation100 => {
                    positive(base - a[..6].iter().map(|v| v.allowed).sum::<i64>())
                }
                CharityClass::DisasterRelief => {
                    positive(fraction(base, 3, 5) - a[..7].iter().map(|v| v.allowed).sum::<i64>())
                }
            };
            let mut room =
                carry::Amount::<carry::Dollars>::new(cap).expect("nonnegative statutory capacity");
            let current = carry::consume_step(
                carry::Amount::new(claims.current[i]).expect("nonnegative current gifts"),
                &mut room,
                false,
            )
            .expect("bounded current gifts");
            out.classes[i].current_allowed = current.used.get();
            out.classes[i].current_carry = current.closing.get();
            out.classes[i].current_excess = current.closing.get();
            let mut entries = Vec::new();
            for &(index, c, year, amount) in &origins {
                if c == class {
                    entries.push(carry::Entry {
                        key: index,
                        amount: carry::Amount::<carry::Dollars>::new(amount)
                            .expect("nonnegative reconciled carry"),
                        last_use_year: Some(year + class.years()),
                    });
                }
            }
            let order: Vec<_> = (0..entries.len()).collect();
            let settlements =
                carry::consume(&entries, room, 2026, &order).expect("chronological unique entries");
            let mut allowed = out.classes[i].current_allowed;
            for (entry, value) in entries.into_iter().zip(settlements) {
                allowed += value.used.get();
                out.carryovers[entry.key] = CarryResult {
                    used: value.used.get(),
                    floor: 0,
                    closing: value.closing.get(),
                    expired: value.expired.get(),
                };
            }
            out.classes[i].allowed = allowed;
        }
        let mut floor = phase::cap(
            out.classes.iter().map(|v| v.allowed).sum::<i64>(),
            [fraction(base, 5, 1000)],
        );
        // E50/E100 are original-limit variants of one statutory floor family.
        for group in [&[4][..], &[3], &[2], &[5, 6], &[0], &[1], &[7]] {
            let charge =
                phase::cap(floor, [group.iter().map(|&i| out.classes[i].allowed).sum::<i64>()]);
            floor -= charge;
            let mut remaining = charge;
            for &(index, class, _, _) in &origins {
                let i = class.index();
                if !group.contains(&i) {
                    continue;
                }
                let part = phase::cap(remaining, [out.carryovers[index].used]);
                remaining -= part;
                out.carryovers[index].floor = part;
                out.classes[i].floor += part;
            }
            for &i in group {
                let part = phase::cap(remaining, [out.classes[i].current_allowed]);
                remaining -= part;
                out.classes[i].current_floor = part;
                out.classes[i].floor += part;
                if out.classes[i].current_carry > 0 {
                    out.classes[i].current_carry += part;
                }
            }
        }
        for (i, c) in out.classes.iter_mut().enumerate() {
            let deduction = c.current_allowed - c.current_floor;
            c.current_cash_deduction = if claims.current[i] == 0 {
                0
            } else {
                fraction(deduction, claims.cash[i], claims.current[i])
            };
        }
        out
    }
    pub(crate) fn allowed(&self) -> i64 {
        self.classes.iter().map(|v| v.allowed).sum()
    }
    pub(crate) fn floor(&self) -> i64 {
        self.classes.iter().map(|v| v.floor).sum()
    }
    pub(crate) fn deduction(&self) -> i64 {
        self.allowed() - self.floor()
    }
    pub(crate) fn current_cash(&self) -> i64 {
        self.classes.iter().map(|v| v.current_cash_deduction).sum()
    }
    pub(crate) fn current_noncash(&self) -> i64 {
        self.classes
            .iter()
            .map(|v| v.current_allowed - v.current_floor - v.current_cash_deduction)
            .sum()
    }
    #[cfg(test)]
    pub(crate) fn prior_deduction(&self) -> i64 {
        self.carryovers.iter().map(|v| v.used - v.floor).sum()
    }
}

// A vehicle gift's issue, moved from the calculation view to the paper's own fields.
fn vehicle_relationship_issue(
    source: &VehicleGiftSource,
    form: &Form1098CPaper,
) -> Option<ValidationIssue> {
    let gift = match PropertyGift::try_from((source, form)) {
        Ok(gift) => gift,
        Err(error) => return Some(error.issue()),
    };
    gift.relationship_issue().map(|issue| match issue.key {
        "VEHICLE_SOLD_BEFORE_GIVEN" => {
            ValidationIssue::new(issue.key).at_field("box_4b").in_field("form")
        }
        "GIFT_BENEFIT_ABOVE_VALUE" => {
            ValidationIssue::new(issue.key).at_field("box_6b").in_field("form")
        }
        _ => issue,
    })
}

pub(crate) fn page_issue(
    stored: &[PropertyGift],
    vehicles: &[VehicleGiftSource],
    documents: &[Form1098CDocument],
    appraisals: &[Appraisal],
    carryovers: &[CharityCarryover],
    election: bool,
) -> Option<ValidationIssue> {
    for (index, document) in documents.iter().enumerate() {
        if let Some(issue) = document.form.entry_issue() {
            return Some(issue.in_field("form").in_list("vehicle_gift_documents", index));
        }
    }
    let mut gifts = stored.to_vec();
    let mut rows: Vec<(&str, usize)> = (0..stored.len()).map(|index| ("gifts", index)).collect();
    for (i, source) in vehicles.iter().enumerate() {
        let Some(document) = documents.iter().find(|document| document.id == source.id) else {
            return Some(
                ValidationIssue::new("VEHICLE_GIFT_DOCUMENT_NEEDED")
                    .at_field("id")
                    .in_field("gift")
                    .in_list("vehicles", i),
            );
        };
        if let Ok(gift) = PropertyGift::try_from((source, &document.form)) {
            gifts.push(gift);
            rows.push(("vehicles", i));
        }
    }
    // Preserve the submitted source position when an incomplete paper has no calculation value.
    let row = |i: usize| rows[i];
    for (i, gift) in stored.iter().enumerate() {
        if matches!(gift.kind, GiftKind::Vehicle { .. }) {
            return Some(
                ValidationIssue::new("GIFT_FACTS_INVALID")
                    .at_field("kind")
                    .in_field("gift")
                    .in_list("gifts", i),
            );
        }
        if let Some(issue) = gift.relationship_issue() {
            return Some(issue.in_field("gift").in_list("gifts", i));
        }
    }
    for (i, vehicle) in vehicles.iter().enumerate() {
        let form = &documents.iter().find(|document| document.id == vehicle.id)?.form;
        if form.treatment_issue().is_some() {
            continue;
        }
        if let Some(issue) = vehicle_relationship_issue(vehicle, form) {
            return Some(issue.in_field("gift").in_list("vehicles", i));
        }
    }
    for (i, carry) in carryovers.iter().enumerate() {
        if let Some(issue) = carry.relationship_issue() {
            return Some(issue.in_list("carryovers", i));
        }
    }
    // Temporary arithmetic exclusion: gift claim/group sums and carryover class/output
    // sums remain native i64; their actual owners must widen before these counts.
    for (field, len, max) in [("gifts", gifts.len(), 40), ("carryovers", carryovers.len(), 20)] {
        if len > max {
            return Some(ValidationIssue::new("GIFT_FACTS_INVALID").at_field(field));
        }
    }
    if let Some((i, _)) = gifts.iter().enumerate().find(|(_, g)| !g.valid()) {
        let (list, index) = row(i);
        return Some(
            ValidationIssue::new("GIFT_FACTS_INVALID").at_field("gift").in_list(list, index),
        );
    }
    if let Some((i, _)) = appraisals.iter().enumerate().find(|(_, a)| !a.valid()) {
        return Some(ValidationIssue::new("GIFT_FACTS_INVALID").at_list_item("appraisals", i));
    }
    if let Some((i, _)) = carryovers.iter().enumerate().find(|(_, c)| !c.valid()) {
        return Some(ValidationIssue::new("GIFT_FACTS_INVALID").at_list_item("carryovers", i));
    }
    if let Some((i, _)) = carryovers
        .iter()
        .enumerate()
        .find(|(i, c)| carryovers[..*i].iter().any(|p| p.year == c.year && p.class == c.class))
    {
        return Some(
            ValidationIssue::new("CHARITY_CARRY_TWICE").at_field("year").in_list("carryovers", i),
        );
    }
    if election
        && let Some((i, _)) = carryovers.iter().enumerate().find(|(_, c)| {
            matches!(c.class, CharityClass::CapitalGain30) && c.appreciation.is_none()
        })
    {
        return Some(
            ValidationIssue::new("GIFT_ELECTION_NEEDS_CARRY_APPRECIATION")
                .at_field("appreciation")
                .in_list("carryovers", i),
        );
    }
    for (i, gift) in gifts.iter().enumerate() {
        if matches!(
            gift.kind,
            GiftKind::Goods {
                kind: GoodsKind::Clothing | GoodsKind::Household,
                condition: GoodsCondition::NotGood
            }
        ) && (gift.value <= 50_000 || gift.appraisal.is_none())
        {
            return Some(
                ValidationIssue::new("GIFT_ITEM_NEEDS_APPRAISAL")
                    .at_field("appraisal")
                    .in_list(row(i).0, row(i).1),
            );
        }
        if let Some(id) = gift.appraisal {
            let Some(appraisal) = appraisals.iter().find(|a| a.id == id && id > 0) else {
                return Some(
                    ValidationIssue::new("GIFT_APPRAISAL_UNKNOWN")
                        .at_field("appraisal")
                        .in_list(row(i).0, row(i).1),
                );
            };
            if appraisal.report_date < crate::dates::add_days(gift.given, -60)
                || appraisal.effective_date < crate::dates::add_days(gift.given, -60)
            {
                return Some(
                    ValidationIssue::new("GIFT_APPRAISAL_TOO_EARLY")
                        .at_field("appraisal")
                        .in_list(row(i).0, row(i).1),
                );
            }
            if appraisal.effective_date > gift.given
                || (appraisal.report_date > gift.given && appraisal.effective_date != gift.given)
            {
                return Some(
                    ValidationIssue::new("GIFT_APPRAISAL_EFFECTIVE_DATE")
                        .at_field("appraisal")
                        .in_list(row(i).0, row(i).1),
                );
            }
        }
    }
    None
}

pub(crate) fn page_problem(
    gifts: &[PropertyGift],
    vehicles: &[VehicleGiftSource],
    documents: &[Form1098CDocument],
    appraisals: &[Appraisal],
    carryovers: &[CharityCarryover],
    election: bool,
) -> Option<&'static str> {
    page_issue(gifts, vehicles, documents, appraisals, carryovers, election).map(|issue| issue.key)
}

pub(crate) fn report(
    r: &mut crate::FederalResult,
    ded: &crate::FederalDeductions,
    gifts: &[PropertyGift],
    w: &Worksheet,
    itemized: bool,
    nol_reduction: i64,
    class_reduction: [i64; 8],
) {
    if w.classes.iter().all(|c| c.claims == 0) {
        return;
    }
    use crate::kernel::lines::{LineValue, row};
    use crate::line_emission::emit;
    const SOURCE: &str = "https://www.irs.gov/publications/p526";
    const INPUTS: &[&str] = &[
        "deductions.property_gifts",
        "deductions.vehicle_gifts",
        "deductions.charity_carryovers",
        "deductions.cash_charity",
        "deductions.fund_cash_charity",
        "deductions.other_cash_charity",
        "agi",
    ];
    let mut carry_notes = Vec::new();
    let mut emit_value = |form, id: &str, value| {
        emit(
            r,
            &mut (),
            [row(
                form,
                id,
                value,
                INPUTS,
                SOURCE,
                crate::Message::new("EXPLAIN_CHARITY_CLASS_WORKSHEET", alloc::vec![]),
            )],
            |v, _, _| Some(v),
        )
    };
    for class in CharityClass::ALL {
        let c = &w.classes[class.index()];
        for (suffix, amount) in [
            ("current_deduction", if itemized { c.current_allowed - c.current_floor } else { 0 }),
            ("allowed", if itemized { c.allowed - c.floor } else { 0 }),
            ("floor", if itemized { c.floor } else { 0 }),
            (
                "return_year_carryforward",
                if itemized {
                    c.current_carry - class_reduction[class.index()]
                } else {
                    c.current_excess - class_reduction[class.index()]
                },
            ),
        ] {
            emit_value(
                None,
                &alloc::format!("charity_{}_{}", class.key(), suffix),
                LineValue::MoneyDollars(amount),
            );
        }
    }
    for class in CharityClass::ALL {
        let values: Vec<_> = ded
            .charity_carryovers
            .iter()
            .zip(&w.carryovers)
            .filter(|(c, _)| c.class == class)
            .collect();
        if values.is_empty() {
            continue;
        }
        for (suffix, amount) in [
            (
                "deduction",
                if itemized { values.iter().map(|(_, v)| v.used - v.floor).sum() } else { 0 },
            ),
            (
                "consumed_without_deduction",
                if itemized { 0 } else { values.iter().map(|(_, v)| v.used).sum() },
            ),
            (
                "floor_not_carried",
                if itemized { values.iter().map(|(_, v)| v.floor).sum() } else { 0 },
            ),
            ("remaining_following_year", values.iter().map(|(_, v)| v.closing).sum()),
            ("expired_after_return_year", values.iter().map(|(_, v)| v.expired).sum()),
        ] {
            emit_value(
                None,
                &alloc::format!("charity_prior_{}_{}", class.key(), suffix),
                LineValue::MoneyDollars(amount),
            );
        }
    }
    if itemized {
        emit_value(
            None,
            "charity_floor_not_carried",
            LineValue::MoneyDollars(if itemized {
                w.classes
                    .iter()
                    .map(|c| if c.current_excess == 0 { c.current_floor } else { 0 })
                    .sum::<i64>()
                    + w.carryovers.iter().map(|c| c.floor).sum::<i64>()
            } else {
                0
            }),
        );
    }
    emit_value(None, "charity_return_year_nol_reduction", LineValue::MoneyDollars(nol_reduction));
    let mut expired = 0;
    for (old, value) in ded.charity_carryovers.iter().zip(&w.carryovers) {
        for (suffix, amount) in [
            ("opening", dollars(old.balance)),
            ("unused", value.closing + value.expired),
            ("used", value.used),
            ("deduction", if itemized { value.used - value.floor } else { 0 }),
            ("consumed_without_deduction", if itemized { 0 } else { value.used }),
            ("remaining", value.closing),
            ("expired", value.expired),
            ("floor_not_carried", if itemized { value.floor } else { 0 }),
        ] {
            emit_value(
                None,
                &alloc::format!("charity_carry_{}_{}_{}", old.year, old.class.key(), suffix),
                LineValue::MoneyDollars(amount),
            );
        }
        if !matches!(old.class, CharityClass::Cash60 | CharityClass::DisasterRelief) {
            carry_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!("Retain the {} original and intervening contribution worksheets and applicable Form 8283. This reconciled balance expires after {} if unused.",old.year,old.year+old.class.years()),
                crate::Message::new("NOTE_CHARITY_CARRY_RECORDS",alloc::vec![crate::MessageArg::Number(i64::from(old.year)),crate::MessageArg::Number(i64::from(old.year+old.class.years()))])));
        }
        expired += value.expired;
    }
    let total_carry: i64 =
        w.classes.iter().map(|c| if itemized { c.current_carry } else { c.current_excess }).sum();
    if w.classes.iter().any(|c| c.claims > 0) {
        emit_value(
            None,
            "charity_return_year_carryforward",
            LineValue::MoneyDollars(positive(total_carry - nol_reduction)),
        );
        emit_value(
            None,
            "charity_carry_expired_after_return_year",
            LineValue::MoneyDollars(expired),
        );
    }
    if itemized && !gifts.is_empty() {
        for (i, gift) in gifts.iter().enumerate() {
            emit_value(
                None,
                &alloc::format!("charity_property_{}_claim", i + 1),
                LineValue::MoneyCents(gift.claim(ded.basis_election).1),
            );
        }
        let total_claim = gifts.iter().map(|g| g.claim(ded.basis_election).1).sum::<i64>();
        emit_value(
            None,
            "form_8283_needed",
            LineValue::CountWhole(i64::from(
                total_claim > crate::year_constants::CHARITY_FORM8283_THRESHOLD_CENTS,
            )),
        );
        let mut totals = [0; 2];
        let mut indices = [0; 2];
        for gift in gifts {
            let b = gift.section_b(gifts, ded.basis_election);
            let section = usize::from(b);
            indices[section] += 1;
            let claim = gift.claim(ded.basis_election).1;
            totals[section] += claim;
            if total_claim <= crate::year_constants::CHARITY_FORM8283_THRESHOLD_CENTS {
                continue;
            }
            let prefix = if b { "b" } else { "a" };
            let index = indices[section];
            let fields = if b {
                alloc::vec![
                    ("c", "value", gift.value),
                    ("f", "cost", gift.basis),
                    ("i", "claimed", claim)
                ]
            } else {
                alloc::vec![("g", "cost", gift.basis), ("h", "value", claim)]
            };
            for (column, suffix, amount) in fields {
                emit_value(
                    Some(crate::FormLine::new(
                        crate::TaxForm::Form8283,
                        &alloc::format!("{prefix}{index}{column}"),
                    )),
                    &alloc::format!("form_8283_section_{prefix}_{index}_{suffix}"),
                    LineValue::MoneyDollars(dollars(amount)),
                );
            }
        }
        if total_claim > crate::year_constants::CHARITY_FORM8283_THRESHOLD_CENTS {
            for (i, label) in [(0, "a"), (1, "b")] {
                if totals[i] > 0 {
                    emit_value(
                        None,
                        &alloc::format!("form_8283_section_{label}_value"),
                        LineValue::MoneyDollars(dollars(totals[i])),
                    );
                }
            }
        }
    }
    r.review_notes.extend(carry_notes);
    // Return-attachment and later-event notices use the actual item/group facts;
    // they do not imply that the App prepares or files an attachment.
    for (i, gift) in gifts.iter().enumerate() {
        if !itemized {
            continue;
        }
        let claim = gift.claim(ded.basis_election).1;
        let b = gift.section_b(gifts, ded.basis_election);
        let group_claim = match &gift.kind {
            GiftKind::Conservation { .. } => claim,
            GiftKind::Vehicle { .. } => gifts
                .iter()
                .filter(|g| matches!(g.kind, GiftKind::Vehicle { .. }))
                .map(|g| g.claim(ded.basis_election).1)
                .sum(),
            _ => {
                let group = gift.similar();
                gifts
                    .iter()
                    .filter(|g| similar_eq(g.similar().as_ref(), group.as_ref()))
                    .map(|g| g.claim(ded.basis_election).1)
                    .sum()
            }
        };
        let mut notes = Vec::new();
        if b {
            notes.push(("NOTE_FORM_8283_ATTACHMENTS", "Retain the qualified appraisal, signed Form 8283 declarations and acknowledgment, and the contribution records."));
            if matches!(gift.kind, GiftKind::Other { category: PropertyCategory::Art, .. })
                && claim >= 2_000_000
            {
                notes.push(("NOTE_CHARITY_ART_APPRAISAL", "Attach the complete signed appraisal for art deductions of $20,000 or more; retain a photograph for an individual object valued at $20,000 or more."));
            }
            if group_claim > 50_000_000 {
                notes.push(("NOTE_CHARITY_LARGE_APPRAISAL", "Attach the qualified appraisal because the deduction for this item or similar-property group exceeds $500,000."));
            }
            if matches!(
                gift.kind,
                GiftKind::Goods {
                    kind: GoodsKind::Clothing | GoodsKind::Household,
                    condition: GoodsCondition::NotGood
                }
            ) {
                notes.push(("NOTE_CHARITY_POOR_CONDITION_APPRAISAL", "Attach the qualified appraisal for this clothing or household item not in good used condition."));
            }
            if gift.value <= 50_000 {
                notes.push(("NOTE_CHARITY_FORM8283_PART_III", "Identify this item in Form 8283 Section B Part III because its appraised value is $500 or less."));
            } else {
                notes.push(("NOTE_CHARITY_FORM8282", "A donee's disposition within three years may require Form 8282; retain the notice and review any later-year deduction recapture."));
            }
        }
        if matches!(
            gift.kind,
            GiftKind::Conservation { purpose: ConservationPurpose::HistoricStructure, .. }
        ) && claim > 1_000_000
        {
            notes.push(("NOTE_CHARITY_HISTORIC_ATTACHMENTS", "An exterior restriction on a registered historic-district building requires the qualified appraisal, photographs and other agreement information; a deduction over $10,000 also requires Form 8283-V and the $500 fee."));
        }
        if matches!(gift.kind, GiftKind::Vehicle { .. }) && claim > 50_000 {
            notes.push(("NOTE_CHARITY_FORM1098C", "Attach Form 1098-C Copy B, or the qualifying acknowledgment, with the return; use Form 8453 when required for electronic filing."));
        }
        for (key, _text) in notes {
            r.review_notes.push(crate::federal_model::ReviewNote::scoped(
                #[cfg(test)]
                alloc::format!("{}: {}", gift.description, _text),
                crate::Message::new(
                    key,
                    alloc::vec![crate::MessageArg::Text(gift.description.clone())],
                ),
                crate::IssueContext::Section(crate::TaxSection::GiftCarryovers).item(
                    if i < ded.property_gifts.len() {
                        "deductions.property_gifts"
                    } else {
                        "deductions.vehicle_gifts"
                    },
                    if i < ded.property_gifts.len() { i } else { i - ded.property_gifts.len() },
                ),
            ));
        }
    }
}

#[cfg(test)]
mod approved_controls {
    use super::*;
    fn carry(
        year: u32,
        class: CharityClass,
        balance: i64,
        appreciation: Option<i64>,
    ) -> CharityCarryover {
        CharityCarryover {
            research_parts: None,
            year,
            class,
            balance: balance * 100,
            appreciation: appreciation.map(|v| v * 100),
            same_filers: true,
        }
    }
    fn run(
        base: i64,
        current: [i64; 8],
        cash: [i64; 8],
        old: &[CharityCarryover],
        election: bool,
    ) -> Worksheet {
        Worksheet::calculate(base, ClassClaims { current, cash }, old, election)
    }
    #[test]
    fn unit15_approved_f1_property_then_cash_floor_and_corrected_footing() {
        let w =
            run(100_000, [5000, 60000, 0, 0, 0, 0, 0, 0], [0, 60000, 0, 0, 0, 0, 0, 0], &[], false);
        assert_eq!((w.current_cash(), w.current_noncash(), w.deduction()), (55_000, 4500, 59_500));
        assert_eq!(w.classes[1].current_carry, 5000);
        assert_eq!(w.classes[0].current_carry, 0);
    }
    #[test]
    fn unit15_approved_f2_private_foundation_capital_gain_floor_carries_in_class() {
        let w =
            run(100_000, [0, 0, 5000, 0, 30000, 0, 0, 0], [0, 0, 5000, 0, 0, 0, 0, 0], &[], false);
        assert_eq!((w.current_cash(), w.current_noncash(), w.deduction()), (5000, 19500, 24500));
        assert_eq!(w.classes[4].current_carry, 10500);
        let reduced =
            run(100_000, [0, 0, 15000, 0, 0, 0, 0, 0], [0, 0, 5000, 0, 0, 0, 0, 0], &[], false);
        assert_eq!(reduced.deduction(), 14500);
        assert_eq!(reduced.classes[2].current_carry, 0);
    }
    #[test]
    fn unit15_approved_f3_painting_and_unrelated_use_election() {
        let fmv = run(
            200_000,
            [0, 50000, 0, 30000, 0, 0, 0, 0],
            [0, 50000, 0, 0, 0, 0, 0, 0],
            &[],
            false,
        );
        assert_eq!(
            (fmv.current_cash(), fmv.current_noncash(), fmv.deduction()),
            (50000, 29000, 79000)
        );
        let basis =
            run(200_000, [8000, 50000, 0, 0, 0, 0, 0, 0], [0, 50000, 0, 0, 0, 0, 0, 0], &[], false);
        assert_eq!(basis.deduction(), 57000);
    }
    #[test]
    fn unit15_approved_f4_vehicle_and_f7_poor_condition_sofa() {
        for (base, claim, expected) in [(60000, 3200, 2900), (60000, 3000, 2700), (40000, 600, 400)]
        {
            let w = run(base, [claim, 0, 0, 0, 0, 0, 0, 0], [0; 8], &[], false);
            assert_eq!(w.deduction(), expected);
        }
    }
    #[test]
    fn unit15_approved_f5_current_before_carry_and_basis_election() {
        let old = [
            carry(2024, CharityClass::Cash60, 6000, None),
            carry(2025, CharityClass::CapitalGain30, 9000, Some(4000)),
        ];
        let w =
            run(80000, [0, 30000, 0, 20000, 0, 0, 0, 0], [0, 30000, 0, 0, 0, 0, 0, 0], &old, false);
        assert_eq!(
            (w.current_cash(), w.current_noncash(), w.prior_deduction(), w.deduction()),
            (30000, 3600, 6000, 39600)
        );
        assert_eq!(
            (w.carryovers[0].closing, w.carryovers[1].closing, w.classes[3].current_carry),
            (0, 9000, 16400)
        );
        let elected =
            run(80000, [5000, 30000, 0, 0, 0, 0, 0, 0], [0, 30000, 0, 0, 0, 0, 0, 0], &old, true);
        assert_eq!(elected.deduction(), 45600);
        assert_eq!(elected.carryovers[1].floor, 400);
    }
    #[test]
    fn unit15_approved_f6_k1_cash_joins_current_class_before_older_balance() {
        let old = [carry(2025, CharityClass::Cash60, 5000, None)];
        let w = run(50000, [0, 35000, 0, 0, 0, 0, 0, 0], [0, 35000, 0, 0, 0, 0, 0, 0], &old, false);
        assert_eq!((w.current_cash(), w.prior_deduction(), w.deduction()), (29750, 0, 29750));
        assert_eq!((w.carryovers[0].closing, w.classes[1].current_carry), (5000, 5250));
    }
    #[test]
    fn unit15_approved_conservation_carry_retains_original_percentage() {
        for (class, deduction, closing) in
            [(CharityClass::Conservation, 49500, 20000), (CharityClass::Conservation100, 69500, 0)]
        {
            let w = run(100000, [0; 8], [0; 8], &[carry(2020, class, 70000, None)], false);
            assert_eq!((w.deduction(), w.carryovers[0].closing), (deduction, closing));
        }
    }
}
