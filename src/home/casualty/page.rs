// Request selectors belong to page values, never persisted casualty facts.
use super::*;

/// Business-home source for this casualty property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CasualtyHomeLinkPage {
    /// Saved home
    #[libertas_enum_source("$.home_sites")]
    pub home_site: i64,
    /// What the records establish about the business part of this home
    /// A zero value after the event does not by itself establish total destruction.
    pub business_damage: HomeCasualtyDamage,
}

/// Property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct CasualtyPropertyPage {
    /// Saved property identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Saved business home represented by this property
    /// [NullText]
    /// This is not a saved business home
    /// [SetText]
    /// This property is the saved business home
    pub home_link: Option<CasualtyHomeLinkPage>,
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

impl TryFrom<CasualtyProperty> for CasualtyPropertyPage {
    type Error = &'static str;
    fn try_from(value: CasualtyProperty) -> Result<Self, Self::Error> {
        let home_link = match (value.home_site, value.business_damage) {
            (None, None) => None,
            (Some(home_site), Some(business_damage)) => {
                Some(CasualtyHomeLinkPage { home_site, business_damage })
            }
            _ => return Err("HOME_CASUALTY_DAMAGE_REQUIRED"),
        };
        Ok(Self {
            home_link,
            id: value.id,
            label: value.label,
            location: value.location,
            acquisition: value.acquisition,
            basis: value.basis,
            before: value.before,
            after: value.after,
            reimbursement: value.reimbursement,
            received: value.received,
        })
    }
}
impl From<CasualtyPropertyPage> for CasualtyProperty {
    fn from(value: CasualtyPropertyPage) -> Self {
        let (home_site, business_damage) = match value.home_link {
            None => (None, None),
            Some(link) => (Some(link.home_site), Some(link.business_damage)),
        };
        Self {
            home_site,
            business_damage,
            id: value.id,
            label: value.label,
            location: value.location,
            acquisition: value.acquisition,
            basis: value.basis,
            before: value.before,
            after: value.after,
            reimbursement: value.reimbursement,
            received: value.received,
        }
    }
}

/// Event
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct PersonalCasualtyPage {
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
    pub properties: Vec<CasualtyPropertyPage>,
    /// Complete Form 4684 Section A records for this return confirmed
    /// No leaves this event's loss out of the estimate; a gain from reimbursement above basis is still included. For a declared event, confirm the declaration and incident period cover the event and each property's location. For every event, all basis, value and insurance-claim records are complete; no unentered casualty/theft event, prior-year election, unlinked business/investment property, postponed gain/replacement election, different AMT basis or pending reimbursement dispute applies. For inherited property, confirm section 1014 basis and no 2010 section 1022/Form 8939 election. Gifted and other carryover-basis holding periods need another worksheet. The major-disaster option requires a presidential major-disaster declaration (DR-), not an emergency-only declaration.
    /// Check the insurance settlement, property-basis records and disaster declaration.
    pub source_complete: bool,
}

impl TryFrom<PersonalCasualty> for PersonalCasualtyPage {
    type Error = (&'static str, usize);
    fn try_from(value: PersonalCasualty) -> Result<Self, Self::Error> {
        let properties = value
            .properties
            .into_iter()
            .enumerate()
            .map(|(index, property)| {
                CasualtyPropertyPage::try_from(property).map_err(|key| (key, index))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            properties,
            id: value.id,
            label: value.label,
            date: value.date,
            declaration: value.declaration,
            zip: value.zip,
            source_complete: value.source_complete,
        })
    }
}
impl From<PersonalCasualtyPage> for PersonalCasualty {
    fn from(value: PersonalCasualtyPage) -> Self {
        Self {
            properties: value.properties.into_iter().map(Into::into).collect(),
            id: value.id,
            label: value.label,
            date: value.date,
            declaration: value.declaration,
            zip: value.zip,
            source_complete: value.source_complete,
        }
    }
}
