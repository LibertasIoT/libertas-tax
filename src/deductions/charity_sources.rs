// Charitable property gifts, printed vehicle documents, appraisals and retained source facts.
// IRS Publication 526, Form 8283 instructions, Form 1098-C instructions and IRC section 170.
use crate::document_parties::DocumentRecipient;
use crate::kernel::{
    phase,
    ratio::{fraction, positive},
};
use crate::printed_state::PrintedParty;
use crate::record_choice::RecordChoice;
use crate::validation::ValidationIssue;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Form 1098-C — Contributions of Motor Vehicles, Boats, and Airplanes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read this one Form 1098-C exactly as printed. Copy every printed donee and donor name, TIN and address, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Read calendar year, corrected status, donee telephone and boxes 1 through 7 separately in printed order. Preserve the separate boxes 2b year, 2c make and 2d model. Read box 2b as the numeric printed vehicle year, or null when no year is printed; never substitute zero or infer a year from a vehicle description. Read each checkbox independently, including boxes 6a Yes and No and the box 6c intangible religious benefits checkbox. Read treatment from boxes 4a through 5c: Sold for box 4a, NeedyPerson for box 5b, and SignificantUse or MaterialImprovement for box 5a when box 5c distinguishes them. Use SignificantUseOrMaterialImprovement when box 5a certifies either or both without distinguishing them; preserve all of box 5c. Leave treatment unresolved if the document does not settle it. Report conflicting certifications as an error. A blank, illegible or conflicting contribution date or vehicle identification number is an error. Blank money and odometer boxes are zero; blank text is empty and an unprinted sale date is absent. Do not substitute $500 for box 4c, calculate a deduction, or infer acquisition, value or charity facts absent from the form. Check the calendar year against the supplied year and report a mismatch. Copies B and C of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.donee.federal_id,@.donor.federal_id,@.calendar_year,@.box_1,@.box_3,@.box_4c,@.box_6b",
    form_date_order = "MonthDayYear"
)]
pub struct Form1098CPaper {
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// DONEE'S name, street address, city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub donee: PrintedParty,
    /// DONEE'S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub donee_telephone: String,
    /// DONOR'S name, TIN, street address (including apt. no.), city or town, state or province, country, and ZIP or foreign postal code
    #[libertas_llm_doc_field]
    pub donor: PrintedParty,
    /// Box 1 Date of contribution
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    #[libertas_number(min = 20260101, max = 20261231)]
    pub box_1: u32,
    /// Box 2a Odometer mileage
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_physical_unit("mile")]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub box_2a: u64,
    /// Box 2b Year
    /// [NullText]
    /// No vehicle year is printed
    /// [SetText]
    /// Vehicle year is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_format("0")]
    pub box_2b: Option<u32>,
    /// Box 2c Make
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_2c: String,
    /// Box 2d Model
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_2d: String,
    /// Box 3 Vehicle or other identification number
    #[libertas_llm_doc_field(allow_empty = false)]
    // Form 1098-C box 3 admits a 17-character VIN or a shorter hull or aircraft identifier.
    #[libertas_size(min = 1, max = 17)]
    pub box_3: String,
    /// Box 4a Donee certifies that vehicle was sold in arm's length transaction to unrelated party
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_4a: bool,
    /// Box 4b Date of sale
    /// [NullText]
    /// No date of sale is printed
    /// [SetText]
    /// Date of sale is
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    pub box_4b: Option<u32>,
    /// Box 4c Gross proceeds from sale (see instructions)
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_4c: i64,
    /// Box 5a Donee certifies that vehicle will not be transferred for money, other property, or services before completion of material improvements or significant intervening use
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_5a: bool,
    /// Box 5b Donee certifies that vehicle is to be transferred to a needy individual for significantly below fair market value in furtherance of donee's charitable purpose
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_5b: bool,
    /// Box 5c Donee certifies the following detailed description of material improvements or significant intervening use and duration of use
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_5c: String,
    /// Box 6a Did you provide goods or services in exchange for the vehicle? Yes
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6a_yes: bool,
    /// Box 6a Did you provide goods or services in exchange for the vehicle? No
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6a_no: bool,
    /// Box 6b Value of goods and services provided in exchange for the vehicle
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_6b: i64,
    /// Box 6c Describe the goods and services, if any, that were provided
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_6c_description: String,
    /// Box 6c If this box is checked, donee certifies that the goods and services consisted solely of intangible religious benefits
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6c_intangible_religious_benefits: bool,
    /// Box 7 Under the law, the donor may not claim a deduction of more than $500 for this vehicle if this box is checked
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_7: bool,
    /// Treatment certified in boxes 4a–5c
    /// Use the same printed certification whether entering the form yourself or importing it. Sale date, proceeds and the description stay in their printed boxes above.
    /// [NullText]
    /// The document does not settle the treatment
    /// [SetText]
    /// The document certifies
    #[libertas_llm_doc_field]
    pub treatment: Option<Form1098CTreatment>,
}

/// Treatment certified in boxes 4a–5c
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Form1098CTreatment {
    /// Sold to an unrelated party in an arm's length transaction (box 4a)
    Sold,
    /// Transferred to a needy individual (box 5b)
    NeedyPerson,
    /// Significant intervening use (boxes 5a and 5c)
    SignificantUse,
    /// Material improvement (boxes 5a and 5c)
    MaterialImprovement,
    /// Material improvement or significant intervening use, including both (box 5a)
    SignificantUseOrMaterialImprovement,
}
impl Form1098CPaper {
    pub(crate) fn entry_issue(&self) -> Option<ValidationIssue> {
        let certifications = u8::from(self.box_4a) + u8::from(self.box_5a) + u8::from(self.box_5b);
        let disagrees = self.treatment.is_some_and(|treatment| match treatment {
            Form1098CTreatment::Sold => !self.box_4a,
            Form1098CTreatment::NeedyPerson => !self.box_5b,
            Form1098CTreatment::SignificantUse
            | Form1098CTreatment::MaterialImprovement
            | Form1098CTreatment::SignificantUseOrMaterialImprovement => !self.box_5a,
        });
        if certifications > 1 || disagrees {
            return Some(
                ValidationIssue::new("VEHICLE_CERTIFICATION_CONFLICT").at_field("treatment"),
            );
        }
        if self.box_4b.is_some_and(|sold| sold < self.box_1) {
            return Some(ValidationIssue::new("VEHICLE_SOLD_BEFORE_GIVEN").at_field("box_4b"));
        }
        None
    }
    pub(crate) fn treatment_issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = self.entry_issue() {
            return Some(issue);
        }
        let Some(treatment) = self.treatment else {
            return Some(ValidationIssue::new("VEHICLE_TREATMENT_NEEDED").at_field("treatment"));
        };
        if treatment == Form1098CTreatment::Sold && self.box_4b.is_none() {
            return Some(ValidationIssue::new("VEHICLE_SALE_DATE_NEEDED").at_field("box_4b"));
        }
        if self.box_5a && self.box_5c.trim().is_empty() {
            return Some(ValidationIssue::new("VEHICLE_USE_DESCRIPTION_NEEDED").at_field("box_5c"));
        }
        None
    }

    pub(crate) fn vehicle_use(&self) -> Result<VehicleUse, ValidationIssue> {
        if let Some(issue) = self.treatment_issue() {
            return Err(issue);
        }
        match self.treatment {
            Some(Form1098CTreatment::Sold) => {
                let sold = self.box_4b.ok_or_else(|| {
                    ValidationIssue::new("VEHICLE_SALE_DATE_NEEDED").at_field("box_4b")
                })?;
                Ok(VehicleUse::Sold { sold, proceeds: self.box_4c })
            }
            Some(Form1098CTreatment::NeedyPerson) => Ok(VehicleUse::NeedyPerson),
            Some(treatment) => {
                // Box 5a certifies improvement, significant use, or both. Both
                // existing native treatments use the same fair-market-value rule.
                // https://www.irs.gov/instructions/i1098c
                Ok(if treatment == Form1098CTreatment::MaterialImprovement {
                    VehicleUse::MaterialImprovement { details: self.box_5c.clone() }
                } else {
                    VehicleUse::SignificantUse { details: self.box_5c.clone() }
                })
            }
            None => Err(ValidationIssue::new("VEHICLE_TREATMENT_NEEDED").at_field("treatment")),
        }
    }
}

/// Form 1098-C in the Document center
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form1098CDocument {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1098-C as printed
    #[libertas_llm_doc_field]
    pub form: Form1098CPaper,
    /// Accepted recipient association
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(Unknown)]
    pub assigned: DocumentRecipient,
    /// How the App assigned the owner
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub owner_source: Option<crate::OwnerSource>,
    /// Your share of a joint document
    /// Percent of the document's amounts that are yours; your spouse's share is the rest.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    pub joint_share: Option<i32>,
}

/// Vehicle, boat or airplane gift
/// Personal facts linked to the accepted Form 1098-C.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct VehicleGiftSource {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Kind of charity
    pub donee: DoneeKind,
    /// How you got it
    pub how_acquired: AcquiredBy,
    /// Date you got it
    /// The day you bought it; for a gift, the day the giver bought it; for an inheritance, the date of death. An approximate date is fine. Form 8283 column (e).
    #[libertas_date_only]
    pub acquired: u32,
    /// Your cost or other basis
    /// What you paid, or the giver's cost, or the value at the date of death. Form 8283 column (g).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Fair market value when given
    /// A used-car guide's private-party value for a vehicle; the appraised value for appraised property.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub value: i64,
    /// How you valued it
    pub value_method: ValueMethod,
    /// Appraisal
    #[libertas_hidden]
    #[libertas_default($null)]
    pub appraisal: Option<i64>,
    /// This is a complete, debt-free personal-property gift
    /// No for a partial or future interest, bargain sale, business/depreciated property, inventory or property subject to debt. Such a gift is left out with a note.
    pub in_scope: bool,
    /// The charity signed Form 8283 Part V
    /// Needed only when this gift belongs in Section B. No leaves a Section B gift out with a note; a Section A gift does not need this signature.
    #[libertas_default(false)]
    pub donee_acknowledged: bool,
}
/// Gift
/// One contribution of property in 2026. The charity's written acknowledgment is required for any gift of $250 or more; keep it with your records.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// ConservationPurpose is an Enumeration: HistoricStructure has wire index4.
pub struct PropertyGift {
    /// Description
    /// For example, three bags of clothing, 40 shares of Example Corp., or an oil painting by Example Painter. For a vehicle, its year, make and model (boxes 2b–2d).
    #[libertas_size(min = 1)]
    pub description: String,
    /// Charity
    #[libertas_size(min = 1)]
    pub charity: String,
    /// Kind of charity
    pub donee: DoneeKind,
    /// Date given
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Fair market value when given
    /// Thrift-store prices for used goods; a used-car guide's private-party value for a vehicle; the average of the day's high and low quotations for shares; the appraised value for appraised property.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub value: i64,
    /// How you valued it
    pub value_method: ValueMethod,
    /// Value of anything you received in return
    /// Goods or services the charity gave you for this gift (Form 1098-C box 6b; a dinner, tickets).
    /// [NullText]
    /// I received nothing in return for this gift
    /// [SetText]
    /// The goods or services I received were worth
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
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
    pub kind: GiftKind,
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
    /// Enter these on the vehicle, boat or airplane list, not here; a gift of this kind in this list is not accepted.
    Vehicle {
        /// Box 3 vehicle identification number
        // Legal identifier: Form 1098-C box 3 admits a 17-character VIN or a shorter hull or aircraft identifier.
        #[libertas_size(min = 1, max = 17)]
        vin: String,
        /// What the charity did with it
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
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub rehabilitation_credits: i64,
    /// Fair market value of the full building at contribution
    /// This is the entire building's value, not the value of the easement alone.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
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
        parts_rule(self, None).is_none()
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
                .all(|&v| crate::dates::source_valid_date(v))
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
            && crate::dates::source_valid_date(self.given)
            && (20260101..=20261231).contains(&self.given)
            && crate::dates::source_valid_date(self.acquired)
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
                                crate::dates::source_valid_date(*sold)
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
pub(crate) fn similar_eq(a: Option<&SimilarPropertyKind>, b: Option<&SimilarPropertyKind>) -> bool {
    match (a, b) {
        (
            Some(SimilarPropertyKind::Other { name: a }),
            Some(SimilarPropertyKind::Other { name: b }),
        ) => a.trim().eq_ignore_ascii_case(b.trim()),
        (Some(a), Some(b)) => a == b,
        _ => false,
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

/// Kind
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum GoodsKind {
    /// Clothing
    Clothing,
    /// Household items
    /// Furniture, appliances, electronics, linens and the like.
    Household,
    /// Books
    Books,
    /// Jewelry
    Jewelry,
}
/// What the charity did with it
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum VehicleUse {
    /// Sold it (box 4a)
    /// Your deduction is the smaller of its value and the gross proceeds, or of its value and $500 when it sold for $500 or less. If box 7 is checked and no proceeds are shown, enter $500.
    Sold {
        /// Box 4b date of sale
        #[libertas_date_only]
        sold: u32,
        /// Box 4c gross proceeds
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        proceeds: i64,
    },
    /// Gave or sold it to a person in need (box 5b)
    NeedyPerson,
    /// Will use it significantly in its work (box 5a)
    SignificantUse {
        /// Box 5c description
        #[libertas_size(min = 1)]
        details: String,
    },
    /// Will materially improve it (box 5a)
    MaterialImprovement {
        /// Box 5c description
        #[libertas_size(min = 1)]
        details: String,
    },
}

/// An entity absent from this return that made the retained charitable gift
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OtherCharityEntity {
    /// Entity's actual name
    #[libertas_size(min = 1)]
    pub name: String,
    /// Entity's actual employer identification number
    // Legal identifier: an EIN is nine digits displayed as NN-NNNNNNN.
    #[libertas_size(min = 10, max = 10)]
    #[libertas_pattern(r"[0-9]{2}-[0-9]{7}")]
    pub ein: String,
}
/// Source of a retained activity charitable gift
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchCharityOrigin {
    CurrentK1 { source: i64 },
    OtherEntity(OtherCharityEntity),
}
/// An activity's portion of the existing charitable carryover
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchCharityPart {
    pub origin: ResearchCharityOrigin,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
    /// Actual original appreciation, separate from the unused balance.
    /// [NullText]
    /// No original-appreciation allocation applies to this retained gift
    /// [SetText]
    /// This activity's original appreciation was
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub original_appreciation: Option<i64>,
}
/// Retained charitable gifts belonging to actual activities
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchCharityParts {
    Parts { sources: Vec<ResearchCharityPart> },
}

pub(crate) fn parts_issue(
    record: &CharityCarryover,
    catalog: Option<&[RecordChoice]>,
) -> Option<ValidationIssue> {
    let Some(ResearchCharityParts::Parts { sources }) = &record.research_parts else {
        return None;
    };
    // Temporary arithmetic exclusion: parts_issue and deductions sum source
    // amounts/appreciation into i64 before comparing with the retained balance.
    if sources.is_empty() || sources.len() > 100 {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("sources")
                .in_field("research_parts"),
        );
    }
    if let Some((i, _)) =
        sources.iter().enumerate().find(|(_, p)| !(100..=100000000000).contains(&p.amount))
    {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("amount")
                .in_list("sources", i)
                .in_field("research_parts"),
        );
    }
    if sources.iter().map(|p| p.amount).sum::<i64>() > record.balance {
        return Some(
            ValidationIssue::new("RETAINED_RESEARCH_PARTS_ABOVE_BALANCE")
                .at_field("sources")
                .in_field("research_parts"),
        );
    }
    if let Some((i, _)) = sources
        .iter()
        .enumerate()
        .find(|(_, p)| p.original_appreciation.is_some_and(|v| !(0..=100000000000).contains(&v)))
    {
        return Some(
            ValidationIssue::new("RETAINED_CHARITY_SOURCE_APPRECIATION_ABOVE_TOTAL")
                .at_field("original_appreciation")
                .in_list("sources", i)
                .in_field("research_parts"),
        );
    }
    if sources.iter().filter_map(|p| p.original_appreciation).sum::<i64>()
        > record.appreciation.unwrap_or(0)
    {
        return Some(
            ValidationIssue::new("RETAINED_CHARITY_SOURCE_APPRECIATION_ABOVE_TOTAL")
                .at_field("sources")
                .in_field("research_parts"),
        );
    }
    let mut ids = alloc::collections::BTreeSet::new();
    for (i, part) in sources.iter().enumerate() {
        match &part.origin {
            ResearchCharityOrigin::CurrentK1 { source } => {
                if !ids.insert(*source) {
                    return Some(
                        ValidationIssue::new("RETAINED_RESEARCH_SOURCE_TWICE")
                            .at_field("source")
                            .in_field("origin")
                            .in_list("sources", i)
                            .in_field("research_parts"),
                    );
                }
                if catalog.is_some_and(|c| !c.iter().any(|r| r.id == *source)) {
                    return Some(
                        ValidationIssue::new("RETAINED_RESEARCH_SOURCE_UNAVAILABLE")
                            .at_field("source")
                            .in_field("origin")
                            .in_list("sources", i)
                            .in_field("research_parts"),
                    );
                }
            }
            ResearchCharityOrigin::OtherEntity(entity) => {
                let field = if entity.name.trim().is_empty() {
                    Some("name")
                } else if entity.ein.len() != 10
                    || !entity
                        .ein
                        .bytes()
                        .enumerate()
                        .all(|(i, b)| if i == 2 { b == b'-' } else { b.is_ascii_digit() })
                {
                    Some("ein")
                } else {
                    None
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("RETAINED_CHARITY_ENTITY_RECORDS")
                            .at_field(field)
                            .in_field("origin")
                            .in_list("sources", i)
                            .in_field("research_parts"),
                    );
                }
            }
        }
    }
    None
}
pub(crate) fn parts_rule(
    record: &CharityCarryover,
    catalog: Option<&[RecordChoice]>,
) -> Option<&'static str> {
    parts_issue(record, catalog).map(|issue| issue.key)
}
