use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Fair market value immediately before the disaster
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub before: i64,
    /// Fair market value immediately after the disaster
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub after: i64,
    /// Insurance or other reimbursement received or expected
    /// Everything you received or expect to receive for this property, whether or not you filed a claim. It reduces the loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub reimbursement: i64,
    /// Part of it you received during this return’s tax year
    /// Only what you actually received during this return’s tax year can produce a gain here. A payment you expect after that year is a gain in the year you receive it, not on this return.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    #[libertas_size(min = 1)]
    /// ----
    /// Property
    pub properties: Vec<CasualtyProperty>,
    /// Complete Form 4684 Section A records for this return confirmed
    /// No leaves this event's loss out of the estimate; a gain from reimbursement above basis is still included. For a declared event, confirm the declaration and incident period cover the event and each property's location. For every event, all basis, value and insurance-claim records are complete; no unentered casualty/theft event, prior-year election, unlinked business/investment property, postponed gain/replacement election, different AMT basis or pending reimbursement dispute applies. For inherited property, confirm section 1014 basis and no 2010 section 1022/Form 8939 election. Gifted and other carryover-basis holding periods need another worksheet. The major-disaster option requires a presidential major-disaster declaration (DR-), not an emergency-only declaration.
    /// Check the insurance settlement, property-basis records and disaster declaration.
    pub source_complete: bool,
}
