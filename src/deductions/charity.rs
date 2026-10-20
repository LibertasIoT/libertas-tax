use crate::validation::ValidationIssue;
// Coverage: Unit15 charity classes, property-source facts and the common limitation worksheet.
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

/// Gift
/// One contribution of property in 2026. The charity's written acknowledgment is required for any gift of $250 or more; keep it with your records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this one Form 1098-C (Contributions of Motor Vehicles, Boats, and Airplanes) exactly. Check its calendar year against the supplied year; report a mismatch. Return the donee's name as printed, box 1 date of contribution as the date given, and the kind Vehicle with boxes 2b, 2c and 2d (year, make and model) together as one text and box 3 vehicle identification number; a blank, illegible or conflicting value is an error. For what the charity did with it: when box 4a is checked, return Sold with the box 4b date of sale and box 4c gross proceeds, returning 500 for the proceeds only when box 7 is checked and box 4c shows no amount; when box 5b is checked, return NeedyPerson; when box 5a is checked, return SignificantUse or MaterialImprovement as the box 5c description states, with that description as printed. If none or more than one of boxes 4a, 5a and 5b is checked, or box 5c does not say which use applies, report an error. When box 6a is Yes, return box 6b as the value received; when box 6a is No, return the value received as blank. Do not infer the kind of charity, when or for how much the filer bought the vehicle, its fair market value or any fact absent from the form. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
// ConservationPurpose is an Enumeration: HistoricStructure has wire index4.
pub struct PropertyGift {
    /// Description
    /// For example, three bags of clothing, 40 shares of Example Corp., or an oil painting by Example Painter. For a vehicle, its year, make and model (boxes 2b–2d).
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub description: String,
    /// Charity
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub charity: String,
    /// Kind of charity
    pub donee: DoneeKind,
    /// Date given
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub given: u32,
    /// How you got it
    pub how_acquired: AcquiredBy,
    /// Date you got it
    /// The day you bought it; for a gift, the day the giver bought it; for an inheritance, the date of death. An approximate date is fine. Form 8283 column (e).
    #[libertas_date_only]
    pub acquired: u32,
    /// Your cost or other basis
    /// What you paid, or the giver's cost, or the value at the date of death. Form 8283 column (g).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
    /// Fair market value when given
    /// Thrift-store prices for used goods; a used-car guide's private-party value for a vehicle; the average of the day's high and low quotations for shares; the appraised value for appraised property.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub value: i64,
    /// How you valued it
    pub value_method: ValueMethod,
    /// Value of anything you received in return
    /// Goods or services the charity gave you for this gift (Form 1098-C box 6b; a dinner, tickets).
    /// [NullText]
    /// I received nothing in return for this gift
    /// [SetText]
    /// The goods or services I received were worth
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    #[libertas_default($null)]
    pub benefit_received: Option<i64>,
    /// Appraisal
    /// Choose an entered appraisal when Section B applies. A single clothing or household item over $500 not in good used condition also needs one.
    /// [NullText]
    /// No appraisal is available for this gift
    /// [SetText]
    /// This gift relies on appraisal
    #[libertas_hidden]
    #[libertas_default($null)]
    pub appraisal: Option<i64>,
    /// This is a complete, debt-free personal-property gift
    /// No for a partial or future interest, bargain sale, business/depreciated property, inventory, intellectual property, taxidermy or property subject to debt. Such a gift is left out with a note.
    pub in_scope: bool,
    /// The charity signed Form 8283 Part V
    /// Needed only when this gift belongs in Section B. No leaves a Section B gift out with a note; a Section A gift does not need this signature.
    #[libertas_default(false)]
    pub donee_acknowledged: bool,
    /// What you gave
    #[libertas_llm_doc_field]
    pub kind: GiftKind,
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

/// Kind of charity
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DoneeKind {
    /// Public charity
    /// A church, school, hospital, government unit, publicly supported charity, private operating foundation, a private foundation that passes all gifts on within 2½ months (keep its statement), or a federally chartered veterans' organization. 50% of income limit; 30% for appreciated property.
    PublicCharity,
    /// Donor-advised fund or supporting organization
    /// The same limits as a public charity; the gift is never part of the non-itemizer cash deduction.
    DonorAdvisedFund,
    /// Private foundation
    /// A private nonoperating foundation: 30% of income limit, and appreciated property other than publicly traded stock is deducted at your cost.
    PrivateFoundation,
    /// Other organization
    /// A veterans' organization that is not federally chartered, a fraternal society or lodge, a nonprofit cemetery company, or property held in trust for the use of any charity: 30% of income limit, 20% for appreciated property.
    OtherOrganization,
}
/// How you got it
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum AcquiredBy {
    /// Bought it
    Purchase,
    /// Received it as a gift
    Gift,
    /// Inherited it
    Inheritance,
    /// Exchanged for it
    Exchange,
    /// Made or created it yourself
    /// Art, manuscripts or other work you created is deducted at its cost of materials.
    Created,
}
/// How you valued it
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ValueMethod {
    /// Thrift-store prices
    ThriftShopValue,
    /// Comparable sales
    ComparableSales,
    /// Used-vehicle guide, private-party value
    VehicleGuide,
    /// Average of the day's high and low quotations
    MarketQuotation,
    /// Qualified appraisal
    Appraisal,
}

/// What you gave
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum GiftKind {
    /// Clothing, household items, books or jewelry
    /// Used items you owned personally. Not a partial interest, a bargain sale, business property or anything subject to a debt: a gift like that is left out with a note.
    Goods {
        /// Kind
        #[libertas_default(Clothing)]
        kind: GoodsKind,
        /// Condition
        #[libertas_default(Good)]
        condition: GoodsCondition,
    },
    /// Vehicle, boat or airplane (Form 1098-C)
    Vehicle {
        /// Box 3 vehicle identification number
        #[libertas_llm_doc_field]
        // Legal identifier: Form 1098-C box 3 admits a 17-character VIN or a shorter hull or aircraft identifier.
        #[libertas_size(min = 1, max = 17)]
        vin: String,
        /// What the charity did with it
        #[libertas_llm_doc_field]
        treatment: VehicleUse,
    },
    /// Publicly traded shares
    /// Shares, bonds or fund units quoted on an established market, transferred directly to the charity. Closely held or restricted shares are appraised property.
    PublicStock {
        /// You and your family have given more than 10% of this company's stock
        /// Counting all your earlier gifts of its stock. Matters only for a gift to a private foundation, which then deducts your cost instead of the value.
        #[libertas_default(false)]
        family_over_ten_percent: bool,
    },
    /// Appraised property (Form 8283 Section B)
    /// Art, collectibles, real estate, equipment, closely held securities, digital assets or other property. Not a partial or future interest, a bargain sale, business or depreciated property, inventory, a patent or copyright, or property subject to a debt: a gift like that is left out with a note.
    Other {
        /// Category (Form 8283 line 2)
        category: PropertyCategory,
        /// Generic type of the donated property
        /// Similar items are grouped across charities and across the used-goods and appraised-property lists. The Form 8283 box alone does not identify a similar group.
        similar_kind: SimilarPropertyKind,
        /// Condition
        /// A brief summary of the overall physical condition at the time of the gift (Form 8283 column (b)); leave it empty for real estate or securities.
        #[libertas_size(min = 0)]
        condition: String,
        /// The charity will use or sell it for a purpose unrelated to its charitable work
        /// The "unrelated use" answer the charity gave on Form 8283 Part V. Yes deducts your cost instead of the value for art, collectibles, equipment and other tangible items; No or real estate and securities deduct the value.
        unrelated_use: bool,
    },
    /// Conservation easement or other qualified conservation contribution
    /// Your entire interest in real estate other than minerals, a remainder interest, or a perpetual restriction on its use, given to a government unit, a publicly supported charity or an organization it controls, for one of the four conservation purposes.
    Conservation {
        /// Conservation purpose
        purpose: ConservationPurpose,
        /// Rehabilitation credits for a certified historic building
        /// For a historic-building easement, supply the section 47 rehabilitation credits during the five tax years preceding the contribution and the value of the full building.
        /// [NullText]
        /// No section 47 rehabilitation credits were allowed for this building in the preceding five years
        /// [SetText]
        /// The prior rehabilitation credits and full building value are
        historic_building: Option<HistoricBuildingGift>,
        /// National Park Service project number
        /// Only for a certified historic structure (Form 8283 line 2 box b(1)).
        /// [NullText]
        /// This contribution has no National Park Service project number
        /// [SetText]
        /// The National Park Service project number is
        #[libertas_size(min = 0)]
        #[libertas_default($null)]
        nps_number: Option<String>,
        /// Qualified farmer or rancher with the property kept available for farming
        /// More than half your 2026 gross income is from farming and, if the property is used or available for agriculture or livestock, the easement keeps it available for that use. Yes raises the limit to 100% of income.
        farmer_rancher: bool,
    },
}
use crate::{GoodsKind, VehicleUse};
/// Category (Form 8283 line 2)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum PropertyCategory {
    /// Art
    /// Paintings, sculpture, prints, antiques, rugs, manuscripts, rare books. A gift of $20,000 or more needs a signed copy of the appraisal with the return.
    Art,
    /// Collectibles
    /// Coins, stamps, books, gems, jewelry, sports memorabilia, dolls and the like.
    Collectibles,
    /// Real estate
    RealEstate,
    /// Equipment
    Equipment,
    /// Closely held or restricted securities
    Securities,
    /// Digital assets
    DigitalAssets,
    /// Other property
    Other,
}
/// Conservation purpose
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ConservationPurpose {
    /// Outdoor recreation or education of the general public
    OutdoorRecreation,
    /// A natural habitat of fish, wildlife or plants
    NaturalHabitat,
    /// Open space, farmland or forest land for scenic enjoyment or under a government conservation policy
    OpenSpace,
    /// A historically important land area
    HistoricLand,
    /// A certified historic structure
    /// An exterior restriction on a building in a registered historic district over $10,000 also needs the $500 filing fee (Form 8283-V), photographs and the written agreement.
    HistoricStructure,
}

/// Appraisal
/// A qualified appraisal: A report, effective valuation and Form 8283 declaration, with their distinct dates and qualification conditions.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read page 2 of this one signed Form 8283 (Noncash Charitable Contributions), Part IV Declaration of Appraiser, exactly. Return the appraiser name as printed and the date beside the appraiser signature; a blank or illegible value is an error. Return true for the declaration signed only when an appraiser signature is present. Do not infer the report date, effective valuation date or qualification conditions from the signature. Do not read Parts I, II, III or V and do not infer the appraised values or the items. The file may hold several copies; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Appraisal {
    /// Accepted appraisal identity
    /// Zero marks each new row; saving assigns a distinct identity. The separate RecordChoice catalog owns the selector Key.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Appraiser
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub appraiser: String,
    /// Form 8283 Part IV declaration date
    /// The actual date beside the appraiser signature; it is not the valuation effective date or the appraisal report date.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax structure: this appraisal signature or report may be supplied through the extended 2026 return due date, October 15, 2027.
    #[libertas_number(min = 20251101, max = 20271015)]
    pub declaration_date: u32,
    /// Appraisal report date
    /// Read the signed report, not the Form 8283 declaration. The report must be made no earlier than 60 days before the gift and received by the actual return deadline with extensions.
    #[libertas_date_only]
    // Tax structure: this appraisal signature or report may be supplied through the extended 2026 return due date, October 15, 2027.
    #[libertas_number(min = 20251101, max = 20271015)]
    pub report_date: u32,
    /// Valuation effective date
    /// Read the appraisal report. A report made after the gift must value the property on the contribution date.
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20251101, max = 20261231)]
    pub effective_date: u32,
    /// Qualified appraiser who signed the Form 8283 Part IV declaration
    /// A recognized designation or required education and experience, regular paid appraisals, no value-based fee or disqualification, signed Part IV declaration, and report received by your actual return deadline with extensions. No leaves dependent gifts out with a note.
    pub qualified: bool,
    /// Appraiser signed the Form 8283 Part IV declaration
    #[libertas_llm_doc_field]
    pub declaration_signed: bool,
}

/// Carryover
/// One line of your last return's charitable contribution carryover worksheet.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// CharityClass is an Enumeration: conservation50/100 indexes5/6, disaster7.
// `is` tests Union types; enumeration choices compare their numeric values.

pub struct CharityCarryover {
    /// Year contributed
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2011, max = 2025)]
    pub year: u32,
    /// Class
    pub class: CharityClass,
    /// Unused balance entering 2026
    /// The reconciled whole-dollar amount from the carryover worksheet, not the original gift.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 100)]
    pub balance: i64,
    /// Appreciation in the original gift
    /// Only for a 30% capital-gain carryover when electing to reduce all 2026 capital-gain gifts to cost.
    /// [NullText]
    /// No original appreciation is supplied for a basis election
    /// [SetText]
    /// Appreciation in the original gift was
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub appreciation: Option<i64>,
    /// Same filers as the return that made the gift
    /// Your filing status and the names on that year's return are the same as on this one. No leaves this carryover out of the estimate with a note.
    pub same_filers: bool,
    /// Actual activity-source portions of this existing carryover
    pub research_parts: Option<crate::ResearchCharityParts>,
}
/// Class
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CharityClass {
    /// Cash to public charities (60%)
    Cash60,
    /// Other property to public charities (50%)
    Property50,
    /// Cash and property to other organizations (30%)
    Other30,
    /// Appreciated property to public charities (30%)
    CapitalGain30,
    /// Appreciated property to other organizations (20%)
    CapitalGain20,
    /// Conservation easement (50%, 15 years)
    Conservation,
    /// Qualified farmer or rancher conservation easement (100%, 15 years)
    Conservation100,
    /// 2021 qualified disaster relief cash (60%)
    DisasterRelief,
}

/// Generic kind of similar property
/// Use the same type for similar items given to different charities. This type is independent of the Form 8283 reporting box.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SimilarPropertyKind {
    /// Clothing
    Clothing,
    /// Household items
    Household,
    /// Books, including rare books
    Books,
    /// Jewelry
    Jewelry,
    /// Coins
    Coins,
    /// Stamps
    Stamps,
    /// Antiques
    Antiques,
    /// Sculptures
    Sculpture,
    /// Paintings
    Paintings,
    /// Prints
    Prints,
    /// Lithographs
    Lithographs,
    /// Other objects of art
    ArtObjects,
    /// Nonpublicly traded stock
    NonpublicStock,
    /// Land
    Land,
    /// Buildings
    Buildings,
    /// Equipment
    Equipment,
    /// Digital assets
    DigitalAssets,
    /// Another generic property type
    Other {
        /// Generic type
        /// For example, sports memorabilia or dolls. Use the same type name for similar gifts; do not enter the charity or a unique item description.
        #[libertas_size(min = 1)]
        name: String,
    },
}

/// Certified historic building's credit reduction
/// Section 170(f)(14) reduces the easement deduction by prior rehabilitation credits times the easement value divided by the full building value.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HistoricBuildingGift {
    /// Section 47 rehabilitation credits in the preceding five tax years
    /// Enter the actual credits allowed to you with respect to this building for 2021–2025. Choose no prior credits instead if none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub rehabilitation_credits: i64,
    /// Fair market value of the full building at contribution
    /// This is the entire building's value, not the value of the easement alone.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub building_value: i64,
}

impl CharityClass {
    pub(crate) const ALL: [Self; 8] = [
        Self::Property50,
        Self::Cash60,
        Self::Other30,
        Self::CapitalGain30,
        Self::CapitalGain20,
        Self::Conservation,
        Self::Conservation100,
        Self::DisasterRelief,
    ];
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Property50 => 0,
            Self::Cash60 => 1,
            Self::Other30 => 2,
            Self::CapitalGain30 => 3,
            Self::CapitalGain20 => 4,
            Self::Conservation => 5,
            Self::Conservation100 => 6,
            Self::DisasterRelief => 7,
        }
    }
    pub(crate) fn years(self) -> u32 {
        if matches!(self, Self::Conservation | Self::Conservation100) { 15 } else { 5 }
    }
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Property50 => "property_fifty",
            Self::Cash60 => "cash_sixty",
            Self::Other30 => "other_thirty",
            Self::CapitalGain30 => "capital_gain_thirty",
            Self::CapitalGain20 => "capital_gain_twenty",
            Self::Conservation => "conservation_fifty",
            Self::Conservation100 => "conservation_hundred",
            Self::DisasterRelief => "disaster_relief",
        }
    }
}
impl CharityCarryover {
    pub(crate) fn relationship_issue(&self) -> Option<ValidationIssue> {
        if !(2026 - self.class.years()..=2025).contains(&self.year) {
            return Some(ValidationIssue::new("CHARITY_CARRY_TOO_OLD").at_field("year"));
        }
        if matches!(self.class, CharityClass::DisasterRelief) && self.year != 2021 {
            return Some(ValidationIssue::new("CHARITY_CARRY_DISASTER_YEAR").at_field("year"));
        }
        None
    }

    pub(crate) fn valid(&self) -> bool {
        crate::retained_charity::parts_rule(self, None).is_none()
            && self.relationship_issue().is_none()
            && (100..=100_000_000_000).contains(&self.balance)
            && self.balance % 100 == 0
            && self.appreciation.is_none_or(|v| (0..=100_000_000_000).contains(&v))
    }
}
impl Appraisal {
    pub(crate) fn valid(&self) -> bool {
        (0..100_000_000_000).contains(&self.id)
            && !self.appraiser.trim().is_empty()
            && [self.declaration_date, self.report_date, self.effective_date]
                .iter()
                .all(|&v| crate::federal_rules::valid_date(v))
            && (20251101..=20271015).contains(&self.declaration_date)
            && (20251101..=20271015).contains(&self.report_date)
            && (20251101..=20261231).contains(&self.effective_date)
    }
    pub(crate) fn qualifies_for(&self, given: u32) -> bool {
        self.qualified
            && self.declaration_signed
            && self.report_date >= crate::dates::add_days(given, -60)
            && (crate::dates::add_days(given, -60)..=given).contains(&self.effective_date)
            && (self.report_date <= given || self.effective_date == given)
    }
}
impl PropertyGift {
    pub(crate) fn long_term(&self) -> bool {
        self.value > self.basis
            && !matches!(self.how_acquired, AcquiredBy::Created)
            && (matches!(self.how_acquired, AcquiredBy::Inheritance)
                || crate::dates::held_more_than_one_year(self.acquired, self.given))
    }
    pub(crate) fn public_donee(&self) -> bool {
        matches!(self.donee, DoneeKind::PublicCharity | DoneeKind::DonorAdvisedFund)
    }
    pub(crate) fn claim(&self, basis_election: bool) -> (CharityClass, i64) {
        let benefit = self.benefit_received.unwrap_or(0);
        if let GiftKind::Conservation { farmer_rancher, historic_building, .. } = &self.kind {
            let reduction = historic_building
                .as_ref()
                .map_or(0, |v| fraction(v.rehabilitation_credits, self.value, v.building_value));
            return (
                if *farmer_rancher {
                    CharityClass::Conservation100
                } else {
                    CharityClass::Conservation
                },
                positive(self.value - reduction - benefit),
            );
        }
        let unrelated = matches!(&self.kind,GiftKind::Other{unrelated_use:true,category,..} if !matches!(category,PropertyCategory::RealEstate|PropertyCategory::Securities|PropertyCategory::DigitalAssets))
            || matches!(self.kind, GiftKind::Vehicle { treatment: VehicleUse::Sold { .. }, .. });
        let qualified_stock =
            matches!(self.kind, GiftKind::PublicStock { family_over_ten_percent: false });
        let reduced = !self.long_term()
            || unrelated
            || (self.public_donee() && basis_election)
            || (matches!(self.donee, DoneeKind::PrivateFoundation) && !qualified_stock);
        let mut value = if reduced { phase::cap(self.value, [self.basis]) } else { self.value };
        if let GiftKind::Vehicle { treatment: VehicleUse::Sold { proceeds, .. }, .. } = &self.kind {
            value = phase::cap(value, [phase::floor(*proceeds, 50_000)]);
        }
        (
            match (self.public_donee(), reduced) {
                (true, true) => CharityClass::Property50,
                (true, false) => CharityClass::CapitalGain30,
                (false, true) => CharityClass::Other30,
                (false, false) => CharityClass::CapitalGain20,
            },
            positive(value - benefit),
        )
    }
    pub(crate) fn similar(&self) -> Option<SimilarPropertyKind> {
        match &self.kind {
            GiftKind::Goods { kind, .. } => Some(match kind {
                GoodsKind::Clothing => SimilarPropertyKind::Clothing,
                GoodsKind::Household => SimilarPropertyKind::Household,
                GoodsKind::Books => SimilarPropertyKind::Books,
                GoodsKind::Jewelry => SimilarPropertyKind::Jewelry,
            }),
            GiftKind::Other { similar_kind, .. } => Some(similar_kind.clone()),
            _ => None,
        }
    }
    pub(crate) fn section_b(&self, all: &[Self], election: bool) -> bool {
        if matches!(
            self.kind,
            GiftKind::PublicStock { .. }
                | GiftKind::Vehicle { treatment: VehicleUse::Sold { .. }, .. }
        ) {
            return false;
        }
        if matches!(
            self.kind,
            GiftKind::Goods {
                kind: GoodsKind::Clothing | GoodsKind::Household,
                condition: GoodsCondition::NotGood
            }
        ) {
            return true;
        }
        let amount = match &self.kind {
            GiftKind::Conservation { .. } => self.claim(election).1,
            GiftKind::Vehicle { .. } => all
                .iter()
                .filter(|g| matches!(g.kind, GiftKind::Vehicle { .. }))
                .map(|g| g.claim(election).1)
                .sum(),
            _ => {
                let group = self.similar();
                all.iter()
                    .filter(|g| similar_eq(g.similar().as_ref(), group.as_ref()))
                    .map(|g| g.claim(election).1)
                    .sum()
            }
        };
        amount > crate::year_constants::CHARITY_SECTION_B_THRESHOLD_CENTS
    }
    pub(crate) fn qualified(&self, all: &[Self], appraisals: &[Appraisal], election: bool) -> bool {
        if !self.in_scope {
            return false;
        }
        if !self.section_b(all, election) {
            return true;
        }
        let acknowledged = self.donee_acknowledged;
        acknowledged
            && self
                .appraisal
                .and_then(|id| appraisals.iter().find(|a| a.id == id))
                .is_some_and(|a| a.qualifies_for(self.given))
    }
    pub(crate) fn relationship_issue(&self) -> Option<ValidationIssue> {
        if self.acquired > self.given {
            return Some(ValidationIssue::new("GIFT_BOUGHT_AFTER_GIVEN").at_field("acquired"));
        }
        if matches!(self.kind, GiftKind::Goods { .. }) && self.value > self.basis {
            return Some(ValidationIssue::new("GIFT_VALUE_ABOVE_COST").at_field("value"));
        }
        if let GiftKind::Vehicle { treatment: VehicleUse::Sold { sold, .. }, .. } = self.kind
            && sold < self.given
        {
            return Some(
                ValidationIssue::new("VEHICLE_SOLD_BEFORE_GIVEN")
                    .at_field("sold")
                    .in_field("treatment")
                    .in_field("kind"),
            );
        }
        if self.benefit_received.is_some_and(|b| b >= self.value) {
            return Some(
                ValidationIssue::new("GIFT_BENEFIT_ABOVE_VALUE").at_field("benefit_received"),
            );
        }
        if let GiftKind::Conservation { purpose, historic_building, .. } = &self.kind
            && !matches!(purpose, ConservationPurpose::HistoricStructure)
            && historic_building.is_some()
        {
            return Some(
                ValidationIssue::new("GIFT_HISTORIC_CREDITS_PURPOSE")
                    .at_field("historic_building")
                    .in_field("kind"),
            );
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        let text = |v: &str| !v.trim().is_empty();
        self.relationship_issue().is_none()
            && text(&self.description)
            && text(&self.charity)
            && crate::federal_rules::valid_date(self.given)
            && (20260101..=20261231).contains(&self.given)
            && crate::federal_rules::valid_date(self.acquired)
            && (0..=100_000_000_000).contains(&self.basis)
            && (1..=100_000_000_000).contains(&self.value)
            && self.benefit_received.is_none_or(|v| v >= 1)
            && self.appraisal.is_none_or(|v| (0..100_000_000_000).contains(&v))
            && match &self.kind {
                GiftKind::Goods { .. } => true,
                GiftKind::Vehicle { vin, treatment } => {
                    // Form 1098-C instructions, Box 3: VINs have 17 characters;
                    // hull and aircraft identifiers are shorter. Preserve this
                    // existing upper bound without adding a new format rule.
                    // https://www.irs.gov/instructions/i1098c
                    text(vin)
                        && vin.len() <= 17
                        && match treatment {
                            VehicleUse::Sold { sold, proceeds } => {
                                crate::federal_rules::valid_date(*sold)
                                    && (1..=100_000_000_000).contains(proceeds)
                            }
                            VehicleUse::SignificantUse { details }
                            | VehicleUse::MaterialImprovement { details } => text(details),
                            VehicleUse::NeedyPerson => true,
                        }
                }
                GiftKind::PublicStock { .. } => true,
                GiftKind::Other { similar_kind, .. } => match similar_kind {
                    SimilarPropertyKind::Other { name } => text(name),
                    _ => true,
                },
                GiftKind::Conservation { nps_number, historic_building, .. } => {
                    nps_number.as_ref().is_none_or(|v| text(v))
                        && historic_building.as_ref().is_none_or(|v| {
                            (1..=100_000_000_000).contains(&v.rehabilitation_credits)
                                && (1..=100_000_000_000).contains(&v.building_value)
                        })
                }
            }
    }
}
fn similar_eq(a: Option<&SimilarPropertyKind>, b: Option<&SimilarPropertyKind>) -> bool {
    match (a, b) {
        (
            Some(SimilarPropertyKind::Other { name: a }),
            Some(SimilarPropertyKind::Other { name: b }),
        ) => a.trim().eq_ignore_ascii_case(b.trim()),
        (Some(a), Some(b)) => a == b,
        _ => false,
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

/// Condition
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum GoodsCondition {
    /// Good used condition or better
    Good,
    /// Not in good used condition
    /// Clothing and household items need a single item worth over $500 and a qualified appraisal. This restriction does not apply to books or jewelry.
    NotGood,
}

pub(crate) fn page_issue(
    gifts: &[PropertyGift],
    appraisals: &[Appraisal],
    carryovers: &[CharityCarryover],
    election: bool,
) -> Option<ValidationIssue> {
    for (i, gift) in gifts.iter().enumerate() {
        if let Some(issue) = gift.relationship_issue() {
            return Some(issue.in_field("gift").in_list("gifts", i));
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
        return Some(
            ValidationIssue::new("GIFT_FACTS_INVALID").at_field("gift").in_list("gifts", i),
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
                    .in_list("gifts", i),
            );
        }
        if let Some(id) = gift.appraisal {
            let Some(appraisal) = appraisals.iter().find(|a| a.id == id && id > 0) else {
                return Some(
                    ValidationIssue::new("GIFT_APPRAISAL_UNKNOWN")
                        .at_field("appraisal")
                        .in_list("gifts", i),
                );
            };
            if appraisal.report_date < crate::dates::add_days(gift.given, -60)
                || appraisal.effective_date < crate::dates::add_days(gift.given, -60)
            {
                return Some(
                    ValidationIssue::new("GIFT_APPRAISAL_TOO_EARLY")
                        .at_field("appraisal")
                        .in_list("gifts", i),
                );
            }
            if appraisal.effective_date > gift.given
                || (appraisal.report_date > gift.given && appraisal.effective_date != gift.given)
            {
                return Some(
                    ValidationIssue::new("GIFT_APPRAISAL_EFFECTIVE_DATE")
                        .at_field("appraisal")
                        .in_list("gifts", i),
                );
            }
        }
    }
    None
}

pub(crate) fn page_problem(
    gifts: &[PropertyGift],
    appraisals: &[Appraisal],
    carryovers: &[CharityCarryover],
    election: bool,
) -> Option<&'static str> {
    page_issue(gifts, appraisals, carryovers, election).map(|issue| issue.key)
}

pub(crate) fn report(
    r: &mut crate::FederalResult,
    ded: &crate::FederalDeductions,
    w: &Worksheet,
    itemized: bool,
    nol_reduction: i64,
    class_reduction: [i64; 8],
) {
    if w.classes.iter().all(|c| c.claims == 0) {
        return;
    }
    use crate::kernel::lines::{LineValue, emit, row};
    const SOURCE: &str = "https://www.irs.gov/publications/p526";
    const INPUTS: &[&str] = &[
        "deductions.property_gifts",
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
    if itemized && !ded.property_gifts.is_empty() {
        for (i, gift) in ded.property_gifts.iter().enumerate() {
            emit_value(
                None,
                &alloc::format!("charity_property_{}_claim", i + 1),
                LineValue::MoneyCents(gift.claim(ded.basis_election).1),
            );
        }
        let total_claim =
            ded.property_gifts.iter().map(|g| g.claim(ded.basis_election).1).sum::<i64>();
        emit_value(
            None,
            "form_8283_needed",
            LineValue::CountWhole(i64::from(
                total_claim > crate::year_constants::CHARITY_FORM8283_THRESHOLD_CENTS,
            )),
        );
        let mut totals = [0; 2];
        let mut indices = [0; 2];
        for gift in &ded.property_gifts {
            let b = gift.section_b(&ded.property_gifts, ded.basis_election);
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
    for (i, gift) in ded.property_gifts.iter().enumerate() {
        if !itemized {
            continue;
        }
        let claim = gift.claim(ded.basis_election).1;
        let b = gift.section_b(&ded.property_gifts, ded.basis_election);
        let group_claim = match &gift.kind {
            GiftKind::Conservation { .. } => claim,
            GiftKind::Vehicle { .. } => ded
                .property_gifts
                .iter()
                .filter(|g| matches!(g.kind, GiftKind::Vehicle { .. }))
                .map(|g| g.claim(ded.basis_election).1)
                .sum(),
            _ => {
                let group = gift.similar();
                ded.property_gifts
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
                crate::IssueContext::Section(crate::TaxSection::GiftCarryovers)
                    .item("deductions.property_gifts", i),
            ));
        }
    }
}
