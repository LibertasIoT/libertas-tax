#[allow(unused_imports)]
use super::*;

pub use crate::accepted_dependents::{
    ClaimBasis, CompetingClaim, CustodyRelease, DependentResidency, DependentSituation,
    FederalDependent, PersonFacts, PersonUse,
};
pub use crate::filer_source::SupportShare;

// The people page is asked once on every return; an empty list means no one.
// Each person is a projection of one `FederalDependent`; the interview maps
// it onto the stored record and back. Care costs belong to Child and dependent care.
/// Children and people you support
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Dependents {
    /// People you support
    /// Children and anyone else you support. Leave the list empty if there is no one.
    #[libertas_unordered]
    /// ----
    /// Person
    pub people: Vec<Dependent>,
}

/// Person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Dependent {
    /// Use this person's facts for
    pub use_for: PersonUse,
    /// Accepted record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// First name and middle initial
    /// As shown on their Social Security card; this is what Form 1040 prints.
    #[libertas_size(min = 1)]
    pub first_name: String,
    /// Last name
    #[libertas_size(min = 1)]
    pub last_name: String,
    /// Date of birth
    #[libertas_date_only]
    pub birth_date: u32,
    /// Date of death in 2026
    /// Only for a person who died in 2026. Lived with you is tested over the days they were alive; Form 8862 line 8 prints the month and day.
    /// [NullText]
    /// Alive at the end of 2026
    /// [SetText]
    /// Died in 2026 on
    #[libertas_date_only]
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 20260101, max = 20261231)]
    #[libertas_default($null)]
    pub death_date: Option<u32>,
    /// Relationship to you
    pub relationship: Relationship,
    /// Lived with you in 2026
    #[libertas_constrained_by("$.allowed_lived")]
    pub lived: LivedWithYou,
    /// Situation
    pub situation: DependentSituation,
    /// Taxpayer identification
    /// The number itself is never entered here. A child born and died in 2026 who never had a Social Security number still counts for the child tax credit, the earned income credit and the dependent care credit: choose the answer without a Social Security number, and keep a copy of the birth certificate, death certificate or hospital records showing the child was born alive.
    pub identification: Identification,
    /// Does any of these apply to this person?
    #[libertas_bitflags(PersonFacts)]
    pub facts: u32,
    /// Someone else who could also claim this person
    /// Another person this person lived with for more than half of 2026, such as a grandparent or a parent who is not your spouse, who could claim them as a qualifying child too. A release between parents who live apart is chosen above instead.
    pub competing: CompetingClaim,
}

/// Lived with you in 2026
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum LivedWithYou {
    /// All year
    /// Counts a child born or who died in 2026, and temporary absences such as school or medical care. The home was in the United States. On a final return that is not joint, all year means all the days through the date of death.
    AllYear,
    /// Part of the year
    PartOfYear {
        /// Time lived with you in the United States
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 11)]
        #[libertas_physical_unit("month")]
        months: u32,
    },
    /// Some of the days through the date of death
    /// Asked instead of months on the final return of a person who died in 2026 that is not joint: the person qualifies when they lived in your home for more than half of the days you were alive in 2026.
    Days {
        /// Time in your home through the date of death
        /// Count the days from January 1 to the date of death, including qualifying temporary absences. At most the days you were alive.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        days: u32,
    },
}

/// Person choice
// The one hidden catalog of the people a record can name (the filers and the
// people you support): the identity is the EnumSource key, so a stored row
// never depends on the catalog's order.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PersonChoice {
    /// Person identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Person
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Has a valid SSN issued by the return due date
    /// Carried from the person's own page; the prototype never collects the number itself.
    #[libertas_hidden]
    #[libertas_read_only]
    pub valid_ssn: bool,
}

impl PersonChoice {
    // You (-1) and your spouse (-2) are the filers; every other identity is a
    // dependent (federal_rules::people_students).
    pub(crate) fn is_filer(&self) -> bool {
        self.id == -1 || self.id == -2
    }
}

impl FederalDependent {
    #[cfg(test)]
    pub(crate) fn set_care_expenses(&mut self, value: i64) {
        if !matches!(self.care, Some(DependentCare::Yes { .. })) {
            self.care = Some(DependentCare::Yes { care_expenses: 0 });
        }
        if let Some(DependentCare::Yes { care_expenses }) = &mut self.care {
            *care_expenses = value;
        }
    }
}
