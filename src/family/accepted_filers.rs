use crate::common::AmountBasis;
use crate::filer_facts::{FederalStatus, FilerDependency, MaritalState, SpouseLiving};
use crate::filer_source::{ProfessionalRentals, SupportShare};
use crate::ira_contribution_sources::IraSpouse;
use crate::{CommunityProperty, FinalBonds};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Filing status to evaluate
/// Select a status; the backend checks eligibility from the household facts.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FilingChoice {
    /// Single
    Single,
    /// Married filing jointly
    Joint,
    /// Married filing separately
    Separate {
        /// Section 179 spouse purchase and allocation records
        section179: crate::Section179Spouse,
    },
    /// Head of household
    Head,
    /// Qualifying surviving spouse
    Surviving,
}

/// Final-return source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FinalReturn {
    /// Savings bonds held at death and final-return reporting choice
    pub bonds: Option<FinalBonds>,
    /// Estate-paid medical expenses elected onto this return (section 213(c))
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub estate_medical: i64,
    /// Date of death in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Ordinary final return without additional death-related adjustments
    pub ordinary: bool,
}

/// Filer eligibility
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalPerson {
    /// Fictional date of birth
    #[libertas_date_only]
    pub birth_date: u32,
    /// Legally blind at year end or at death
    pub blind: bool,
    /// Date of death during 2026, if this is a final return
    /// None means alive at year end. Provide the final-return source facts when deceased.
    pub died: Option<FinalReturn>,
    /// This filer is another taxpayer’s qualifying child for EIC, even if not their dependent
    /// Apply the relationship, age or permanent-disability, U.S. residence and joint-return tests.
    /// Do not count a person who need not file and files no return or only to recover withholding or estimated payments.
    /// A joint return claiming EIC is not a refund-only return. Check household residence, support and filing records and the other taxpayer’s return.
    pub another_eitc_child: bool,
    /// Dependency on another return
    pub dependency: FilerDependency,
    /// Has a valid employment-authorized SSN issued by the return due date
    /// The prototype never collects the number itself; answer from the card.
    pub valid_ssn: bool,
    /// Has a valid SSN or ITIN issued by the return due date
    /// The prototype never collects the number itself.
    pub valid_tin: bool,
    /// Main home was in the United States for more than half this filer's tax year
    /// Normally January 1 through December 31, 2026. For a final return, use January 1 through this filer's death. Include qualifying temporary absences and extended active military duty. Count the days at your U.S. home from travel and address records.
    pub us_home_over_half_year: bool,
    /// Full-time student during part of at least five calendar months in this filer's tax year
    /// Include qualifying full-time on-farm training; exclude on-the-job, correspondence and internet-only schools.
    pub full_time_student: bool,
    /// Permanently and totally disabled
    pub disabled: bool,
    /// Physically or mentally incapable of self-care
    pub incapable_self_care: bool,
    /// Earned income compared with this filer’s total support
    pub support_share: SupportShare,
    /// At least one parent was alive at the close of this filer's tax year
    /// Use the date of death on a final return; otherwise December 31, 2026.
    pub parent_alive: bool,
    /// First name and middle initial
    #[libertas_size(min = 1)]
    pub first_name: String,
    /// Last name
    #[libertas_size(min = 1)]
    pub last_name: String,
    /// Real estate professional status (Pub 925)
    /// Present when the person qualifies; each rental or K-1 rental activity still chooses whether the person materially participated in it.
    pub real_estate_professional: Option<ProfessionalRentals>,
    /// Situations of the whole return this filer's page answers
    /// The taxpayer's page holds YourSituations, the spouse's SpouseSituations.
    pub whole_return: u32,
}

/// About this return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSetup {
    /// Return
    #[libertas_hidden]
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Full-year actual amounts or your own full-year estimates
    pub basis: AmountBasis,
    /// Filing status to evaluate
    pub status: FilingChoice,
    /// Marital situation
    pub marital_state: MaritalState,
    /// Paid more than half the cost of maintaining the qualifying home
    pub paid_over_half_home: bool,
    /// Community property
    /// Married filing separately only: whether community-property law divides your income with your spouse.
    pub community_property: CommunityProperty,
    /// Spouse standard deduction boxes, married filing separately only
    pub separate_spouse: SeparateSpouseBoxes,
}

/// About the filers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalPeople {
    /// Parent return for Form 8615
    /// Supply when your age and unearned income require the parent-rate calculation. Leave absent if not applicable or information is not yet available; no estimate is published for an affected return.
    pub parent_tax: Option<crate::ParentTax>,
    /// You
    pub taxpayer: FederalPerson,
    /// Spouse — required only for a joint return
    pub spouse: SpouseFiler,
}

/// Your spouse's standard deduction boxes
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
#[libertas_inline_choices]
pub enum SeparateSpouseBoxes {
    /// Not answered yet
    /// A separate return needs this answer; the estimate asks for it until you give it.
    #[default]
    NotAnswered,
    /// None of these
    NoneOfThese,
    /// Born before January 2, 1962
    Born1961OrEarlier,
    /// Blind
    Blind,
    /// Born before January 2, 1962 and blind
    BornAndBlind,
}

/// Spouse on a joint return
/// Select Yes to provide the applicable facts; No carries no additional data.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SpouseFiler {
    /// No
    No,
    /// Yes
    Yes {
        /// Applicable facts
        facts: FederalPerson,
    },
}

impl FederalPerson {
    // The legal name is the first and last name typed on the filer's page.
    pub(crate) fn named(&self) -> bool {
        !(self.first_name.trim().is_empty() && self.last_name.trim().is_empty())
    }
    // The name as Form 1040 prints it, derived where it is shown from the first
    // and last name.
    pub(crate) fn name(&self) -> Vec<u8> {
        crate::person_names::person_name(&self.first_name, &self.last_name)
    }
    pub(crate) fn death_date(&self) -> Option<u32> {
        self.died.as_ref().map(|v| v.date)
    }
    pub(crate) fn valid_dates(&self) -> bool {
        self.died.as_ref().is_none_or(|v| {
            v.bonds.as_ref().is_none_or(|b| b.valid())
                && (0..=100000000000).contains(&v.estate_medical)
        }) && crate::dates::valid_birth(self.birth_date)
            && self.death_date().is_none_or(|date| {
                crate::dates::source_valid_date(date)
                    && (20260101..=20261231).contains(&date)
                    && date >= self.birth_date
            })
    }
    pub(crate) fn attained(&self, years: u32, day_before: bool) -> bool {
        attained_on(self.birth_date, years, day_before, self.death_date().unwrap_or(20261231))
    }
    pub(crate) fn tax_age(&self) -> u32 {
        age_on(self.birth_date, self.death_date().unwrap_or(20261231))
    }
    // The age the earned income credit tests print (Form 8862 line 10): age 25
    // on the day before the birthday, but age 65 only on the birthday.
    pub(crate) fn eic_age(&self) -> u32 {
        let age = self.tax_age();
        if age >= 65 && !self.attained(65, false) { 64 } else { age }
    }
    pub(crate) fn senior(&self) -> bool {
        self.attained(65, true)
    }
    pub(crate) fn childless_eic_age(&self) -> bool {
        // Pub. 596: age 25 on the preceding day, but age 65 on the birthday.
        self.attained(25, true) && !self.attained(65, false)
    }
}

// Two day-exact age rules, each cited where it applies.
// The day-before rule (`day_before` true): a person attains an age on the day
// before the birthday, so a person born on January 1 attains each age on
// December 31 of the preceding year. It is written for age 65 (i1040 line 12a,
// Schedule 1-A Part V), the childless EIC age 25 (Pub. 596 Rule 11), the Form
// 8814 and 8615 ages (i1040: "a child born on January 1, 2008, is considered
// to be age 18 at the end of 2025"), Form 8863 line 7 (the section 1(g) ages)
// and Form 8880's printed "born after January 1, 2009".
// The birthday rule (`day_before` false): a person attains an age on the
// anniversary of the date of birth (Rev. Rul. 2003-72, 2003-33 I.R.B. 346,
// for sections 21, 23, 24, 32, 129 and 151: "a child born on January 1, 1987,
// attains the age of 17 on January 1, 2004"). The family modules use it for
// the dependent, child tax credit, earned income credit and care tests.
// `day_before` false also tests the "age 65 on the birthday" side of the
// childless EIC range.
pub(crate) fn attained_on(birth: u32, years: u32, day_before: bool, on: u32) -> bool {
    // An unreadable month cannot be placed in a calendar; fall back to the
    // calendar year instead of panicking in the ordinal.
    if !(1..=12).contains(&(birth / 100 % 100)) {
        return on / 10000 >= birth / 10000 + years;
    }
    let birthday = birth + years * 10000;
    crate::dates::ordinal(on) >= crate::dates::ordinal(birthday) - i32::from(day_before)
}
// The age attained by `on` under the day-before rule.
pub(crate) fn age_on(birth: u32, on: u32) -> u32 {
    age_on_rule(birth, on, true)
}
// The age attained by `on` under the chosen rule.
pub(crate) fn age_on_rule(birth: u32, on: u32, day_before: bool) -> u32 {
    let years = (on / 10000).saturating_sub(birth / 10000);
    if attained_on(birth, years + 1, day_before, on) {
        years + 1
    } else if attained_on(birth, years, day_before, on) {
        years
    } else {
        years.saturating_sub(1)
    }
}
// The age at the close of 2026 under the day-before rule, for the provisions
// that rule governs and Rev. Rul. 2003-72 does not name: the IRA, elective
// deferral and HSA catch-up ages, the Trump-account age, Form 8814 and 8615
// (under 19, or 18; or under 24 and a student), and 213(d)(10) (long-term-care
// premium age).
pub(crate) fn age_at_year_end(birth: u32) -> u32 {
    age_on(birth, 20261231)
}
// The age at the close of 2026 under the birthday rule (Rev. Rul. 2003-72):
// IRC 24(c)(1) (under 17), 152(c)(3) and 32(c)(3) (under 19; under 24 and a
// student) and 21(b)(1)(A) (under 13); for a person who died in 2026, the age attained at
// death, when it comes before the close of the year (a child who died before the 17th birthday
// never attains 17; the IRS reads "attained ... before the close of the year" the same way for a
// spouse who died before 65, Pub. 501).
pub(crate) fn birthday_age_at_close_or_death(birth: u32, death: Option<u32>) -> u32 {
    age_on_rule(birth, death.unwrap_or(20261231).min(20261231), false)
}
// Whether `on` is after the holder's 65th birthday. The additional tax on an HSA or Archer MSA
// distribution does not apply to a payment made after the holder turns 65 (IRC 223(f)(4)(B),
// 220(f)(4)(B)); the App's convention (Form 8853 Part II example 3, Pub. 969) taxes a payment on the
// birthday itself and exempts the next day.
pub(crate) fn after_65th_birthday(birth: u32, on: u32) -> bool {
    on > birth + 650000
}

impl FederalSetup {
    pub(crate) fn filing_status(&self) -> FederalStatus {
        match self.status {
            FilingChoice::Single => FederalStatus::Single,
            FilingChoice::Joint => FederalStatus::Joint,
            FilingChoice::Separate { .. } => FederalStatus::Separate,
            FilingChoice::Head => FederalStatus::Head,
            FilingChoice::Surviving => FederalStatus::Surviving,
        }
    }

    // Use the same accepted setup for both editor choices and IRA validation.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn ira_spouse_index(&self) -> i32 {
        if self.filing_status() == FederalStatus::Joint {
            libertas_macros::variant_index!(IraSpouse::Joint) as i32
        } else if self.filing_status() == FederalStatus::Separate && !self.lived_apart_all_year() {
            libertas_macros::variant_index!(IraSpouse::SeparateTogether) as i32
        } else {
            libertas_macros::variant_index!(IraSpouse::NotApplicable) as i32
        }
    }

    pub(crate) fn lived_apart_all_year(&self) -> bool {
        matches!(self.marital_state, MaritalState::Married { living: SpouseLiving::AllYear })
    }

    pub(crate) fn lived_apart_last_six_months(&self) -> bool {
        matches!(
            self.marital_state,
            MaritalState::Married { living: SpouseLiving::AllYear | SpouseLiving::LastSixMonths }
        )
    }

    pub(crate) fn eic_separated(&self) -> bool {
        self.lived_apart_last_six_months()
            || matches!(
                self.marital_state,
                MaritalState::Married { living: SpouseLiving::AgreementAtYearEnd }
            )
    }

    pub(crate) fn could_file_joint_death_year(&self) -> bool {
        match self.marital_state {
            MaritalState::WidowedPriorYear { could_file_joint }
            | MaritalState::WidowedTwoYearsPrior { could_file_joint } => could_file_joint,
            _ => false,
        }
    }
}

impl FederalPeople {
    /// The filer an owner names: always the taxpayer; the spouse only on a
    /// return that has one.
    pub(crate) fn person(&self, owner: crate::Owner) -> Option<&FederalPerson> {
        match owner {
            crate::Owner::Taxpayer => Some(&self.taxpayer),
            crate::Owner::Spouse => self.spouse.as_ref(),
        }
    }
    /// The recorded real estate professional status of an owner.
    pub(crate) fn professional(&self, owner: crate::Owner) -> Option<&ProfessionalRentals> {
        self.person(owner)?.real_estate_professional.as_ref()
    }
    /// Whether either filer is recorded as a real estate professional.
    pub(crate) fn any_professional(&self) -> bool {
        [crate::Owner::Taxpayer, crate::Owner::Spouse]
            .into_iter()
            .any(|owner| self.professional(owner).is_some())
    }
}

impl SeparateSpouseBoxes {
    pub(crate) fn count(self) -> i64 {
        match self {
            Self::NotAnswered | Self::NoneOfThese => 0,
            Self::BornAndBlind => 2,
            _ => 1,
        }
    }
}

impl SpouseFiler {
    pub fn as_ref(&self) -> Option<&FederalPerson> {
        match self {
            Self::No => None,
            Self::Yes { facts } => Some(facts),
        }
    }
    pub fn as_mut(&mut self) -> Option<&mut FederalPerson> {
        match self {
            Self::No => None,
            Self::Yes { facts } => Some(facts),
        }
    }
    pub fn is_some(&self) -> bool {
        self.as_ref().is_some()
    }
    pub fn is_none(&self) -> bool {
        self.as_ref().is_none()
    }
    pub fn iter(&self) -> core::option::IntoIter<&FederalPerson> {
        self.as_ref().into_iter()
    }
}
impl From<Option<FederalPerson>> for SpouseFiler {
    fn from(value: Option<FederalPerson>) -> Self {
        match value {
            None => Self::No,
            Some(facts) => Self::Yes { facts },
        }
    }
}
