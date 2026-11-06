use crate::filer_facts::Relationship;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport, libertas_bitflags};

/// How this existing person record is used on this return
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
pub enum PersonUse {
    /// Evaluate dependency and the related return benefits
    TaxDependent,
    /// Health coverage only
    /// Retain this person's actual relationship, age and support facts for medical and self-employed health coverage. This choice claims no dependency, child tax credit, earned income credit, dependent care or filing-status benefit through this person.
    HealthCoverageOnly,
}

/// Someone else who could also claim this person
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CompetingClaim {
    /// No one else could
    NoOne,
    /// You claim them
    You {
        /// Why you may claim them
        basis: ClaimBasis,
    },
    /// The other person claims them
    /// You take none of the dependency, child tax credit, head of household, dependent care or earned income credit benefits for this person; you may still get the earned income credit without a qualifying child.
    Other,
}

/// Why you may claim them
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ClaimBasis {
    /// You are their parent and the other person agrees or is not their parent
    /// A biological or adoptive parent; a stepparent or foster parent counts only after adopting.
    YouAreTheirParent,
    /// No parent of theirs can claim them
    NoParentCanClaim,
    /// A parent could claim them but doesn't, and your AGI is higher than each such parent's
    HigherIncomeThanTheirParents,
    /// The child's other parent could claim them too, you do not file together, and the child lived with you longer in 2026
    /// Count nights. With the same number of nights, the parent with the higher adjusted gross income may claim the child.
    LongerResidenceThanTheOtherParent,
    /// None of these
    /// The tiebreaker rules give this person to the other claimant.
    NoneOfThese,
}

/// Does any of these apply to this person?
#[libertas_bitflags]
pub struct PersonFacts(pub u32);
impl PersonFacts {
    /// None of these
    pub const NONE: Self = Self(0);
    /// Not a U.S. citizen, national or resident alien
    /// A U.S. citizen who lived abroad is a citizen: leave it unticked.
    pub const ABROAD: Self = Self(1 << 0);
    /// Filed a joint return with their spouse
    /// Not only to get a refund of withholding or estimated tax.
    pub const FILED_JOINTLY: Self = Self(1 << 1);
    /// You released your claim to this child to the child's other parent (Form 8332)
    /// Your own or adopted child lived with you for more nights in 2026 than with the other parent; you two are divorced, legally separated, or lived apart for the whole last six months of 2026; together you paid more than half the child's support; and you signed Form 8332 or a similar statement that covers 2026. The other parent then claims the child tax credit or credit for other dependents for this child; head of household, the earned income credit and the dependent care credit stay with you when the child qualifies you for them. A release you revoked in writing in 2025 or earlier no longer applies.
    pub const YOU_RELEASED: Self = Self(1 << 3);
    /// The child's other parent released their claim to this child to you (Form 8332)
    /// Your own or adopted child lived with the other parent for more nights in 2026; you two are divorced, legally separated, or lived apart for the whole last six months of 2026; together you paid more than half the child's support; and that parent signed Form 8332 or a similar statement that covers 2026 (a 1985–2008 divorce decree or separation agreement stating the same can serve), or a decree or agreement from before 1985 lets you claim the child and you paid at least $600 of their support. You claim the child tax credit or credit for other dependents for this child, not head of household, the earned income credit or the dependent care credit. A revocation the other parent gave you in 2026 takes effect in 2027 at the earliest.
    pub const RELEASED_TO_YOU: Self = Self(1 << 4);
    /// You claim this person under a multiple support agreement (Form 2120)
    /// No one paid more than half of their 2026 support; you paid more than 10%; you and others who could each claim them except for the support test paid more than half together; and each other person who paid more than 10% gave you a signed statement that they will not claim this person for 2026. This person gives you the credit for other dependents, but not head of household.
    pub const MULTIPLE_SUPPORT: Self = Self(1 << 5);
    /// Was married at the end of 2026
    /// A married child who is not a U.S. citizen, national or resident cannot be your qualifying person for head of household; an unmarried one can. Leave it unticked for anyone else.
    pub const MARRIED_AT_YEAR_END: Self = Self(1 << 6);
    /// Paid for more than half of their own support in 2026
    /// A child who pays for more than half of their own living costs, such as a working 17-year-old, is not your qualifying child, so you cannot claim them as one. Count what they spent from their own earnings or savings, scholarships and benefits in their name. Leave it unticked for anyone else, and when you or others together paid more than half.
    pub const OWN_SUPPORT: Self = Self(1 << 7);
    /// Is the qualifying child of someone else who files a 2026 return
    /// A child of your partner, or of another relative or friend in your home, when that person is required to file a 2026 return or files one for more than a refund of tax withheld or paid in. Such a child cannot be a person you support for tax (section 152(d)(1)(D)), so no credit for other dependents or other dependency benefit. Leave it unticked when that person need not file and either does not file or files only for a refund, and for your own qualifying child.
    pub const ANOTHERS_QUALIFYING_CHILD: Self = Self(1 << 9);
    /// Your adopted child who lived with you all year, and you are a U.S. citizen or national
    /// Tick this for a child you adopted who is not a U.S. citizen, national or resident alien (the first box above). The child then still meets the citizenship condition for dependency, the child tax credit and the credit for other dependents (2026 Schedule 8812 instructions, condition 3). The child still needs a valid SSN for the child tax credit; an SSN, ITIN or adoption taxpayer identification number (ATIN) is enough for the credit for other dependents.
    pub const ADOPTED_ALL_YEAR: Self = Self(1 << 10);
    /// Resident of Canada or Mexico
    /// Actual residence for the health-dependent citizenship tests. This fact does not change the App's ordinary dependency, child-credit or EIC citizenship tests.
    pub const RESIDENT_CANADA_MEXICO: Self = Self(1 << 11);
    /// Your adopted child lived in your household and you are a U.S. citizen or national
    /// The child was legally adopted or lawfully placed with you for adoption, and was a member of your household for the year. Reuse the residence answer above. This is the health-dependent citizenship exception, not an adoption-expense amount.
    pub const ADOPTED_US_CITIZEN_PARENT: Self = Self(1 << 12);
    /// Separated parents provided over half this child's support and had custody over half the year
    /// The child's parents were divorced or legally separated, separated under a written agreement, or lived apart for all of the last six months. Together the parents provided over half the child's support and one or both had custody for more than half the year. No multiple support agreement applies. Each parent may include only medical expenses that parent actually paid, even without a Form 8332 release. This does not claim ordinary dependency or a credit.
    pub const MEDICAL_SEPARATED_PARENTS: Self = Self(1 << 13);
}

/// Release of a child's claim between parents who live apart (Form 8332)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CustodyRelease {
    /// None
    None,
    /// You released your claim to the child's other parent
    YouReleased,
    /// The child's other parent released their claim to you
    ReleasedToYou,
}

/// Situation
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DependentSituation {
    /// Child under 19 at the end of 2026
    ChildUnder19,
    /// Full-time student under 24
    /// A full-time student for at least five months of 2026.
    Student,
    /// Permanently and totally disabled
    Disabled,
    /// Another person you support
    /// A parent, relative or household member who is not a qualifying child.
    Supported {
        /// You paid more than half their support in 2026
        your_support_over_half: bool,
        /// Their 2026 gross income
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        gross_income: i64,
    },
}

/// Children and people you support
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalDependent {
    /// Use this person's facts for
    #[libertas_default(TaxDependent)]
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
    /// Fictional birth date
    #[libertas_date_only]
    pub birth_date: u32,
    /// Date of death in 2026
    /// Absent for a living person. The residence tests run over the days from birth (or January 1) to this date; Form 8862 line 8 prints it.
    #[libertas_default($null)]
    pub death_date: Option<u32>,
    /// Relationship
    pub relationship: Relationship,
    /// Time in your home during 2026
    pub residency: DependentResidency,
    /// Main home with you was in the United States for more than half the year
    /// Apply the qualifying newborn/death and temporary-absence rules where applicable; do not reject a newborn solely because the birth occurred late in the year.
    pub us_home_over_half_year: bool,
    /// Physically or mentally incapable of self-care
    pub incapable_self_care: bool,
    /// Work-related care expenses for this dependent
    /// Absent until Child and dependent care answers it for this person.
    /// [NullText]
    /// Not asked yet
    pub care: Option<DependentCare>,
    /// Valid employment-authorized SSN issued by the due date
    /// The prototype never collects the number itself; answer from the card.
    pub valid_ssn: bool,
    /// Valid SSN, ITIN or ATIN issued by the due date
    /// The prototype never collects the number itself.
    pub valid_tin: bool,
    /// Situation chosen on the people page
    /// Child under 19, full-time student, permanently and totally disabled, or another person you support with your support and their gross income. The owner of the student, disabled, support and gross-income tests.
    pub situation: DependentSituation,
    /// Does any of these apply to this person?
    /// The people page's facts, stored as chosen: citizenship, a joint return, a Form 8332 release in either direction, a Form 2120 agreement, marital status and the person's own support.
    #[libertas_bitflags(PersonFacts)]
    pub facts: u32,
    /// Someone else who could also claim this person
    /// The tiebreaker answer (section 152(c)(4)): no one else could, you claim them with the reason you may, or the other person does and you take none of their benefits.
    pub competing: CompetingClaim,
}
impl FederalDependent {
    // The dependent's name, derived where it is shown from the first and last
    // name.
    pub(crate) fn name(&self) -> Vec<u8> {
        crate::person_names::person_name(&self.first_name, &self.last_name)
    }
    pub(crate) fn has(&self, flag: PersonFacts) -> bool {
        self.facts & flag.0 != 0
    }
    // Full-time student for at least five months.
    pub(crate) fn student(&self) -> bool {
        self.situation == DependentSituation::Student
    }
    // Permanently and totally disabled.
    pub(crate) fn disabled(&self) -> bool {
        self.situation == DependentSituation::Disabled
    }
    // IRC 152(d)(1)(D): you provided more than half their support; only
    // another person you support is asked.
    pub(crate) fn your_support_over_half(&self) -> bool {
        matches!(self.situation, DependentSituation::Supported { your_support_over_half: true, .. })
    }
    // The gross income of the qualifying-relative test; zero for anyone but
    // another person you support.
    pub(crate) fn gross_income(&self) -> i64 {
        match self.situation {
            DependentSituation::Supported { gross_income, .. } => gross_income,
            _ => 0,
        }
    }
    // IRC 152(c)(1)(D): provided more than half their own support.
    pub(crate) fn own_support_over_half(&self) -> bool {
        self.has(PersonFacts::OWN_SUPPORT)
    }
    // U.S. citizen, national or resident alien (the fold asks the exception).
    // The adopted-child exception (section 152(b)(3)(B)) is the tick that
    // overrides the first fact.
    pub(crate) fn us_citizen_resident(&self) -> bool {
        !self.has(PersonFacts::ABROAD) || self.has(PersonFacts::ADOPTED_ALL_YEAR)
    }
    pub(crate) fn joint_return_nonrefund(&self) -> bool {
        self.has(PersonFacts::FILED_JOINTLY)
    }
    pub(crate) fn multiple_support(&self) -> bool {
        self.has(PersonFacts::MULTIPLE_SUPPORT)
    }
    pub(crate) fn married_at_year_end(&self) -> bool {
        self.has(PersonFacts::MARRIED_AT_YEAR_END)
    }
    // A child born and died in 2026 without an SSN (2026 i1040s8 "Death or
    // birth of child"; Schedule EIC line 2 "Died"; Form 2441 line 2(b)):
    // the birth date, the date of death and the identification answer decide it.
    pub(crate) fn born_and_died(&self) -> bool {
        self.birth_date / 10000 == 2026 && self.death_date.is_some() && !self.valid_ssn
    }
    // Form 8332: the page rule keeps the two directions apart; if a stored
    // record held both, the release to you is read.
    pub(crate) fn custody_release(&self) -> CustodyRelease {
        if self.has(PersonFacts::RELEASED_TO_YOU) {
            CustodyRelease::ReleasedToYou
        } else if self.has(PersonFacts::YOU_RELEASED) {
            CustodyRelease::YouReleased
        } else {
            CustodyRelease::None
        }
    }
}

impl FederalDependent {
    pub(crate) fn care_expenses(&self) -> i64 {
        match &self.care {
            Some(DependentCare::Yes { care_expenses }) => *care_expenses,
            Some(DependentCare::No) | None => 0,
        }
    }
}

/// Dependent residence period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DependentResidency {
    /// Count days in the home
    Days {
        /// Time in your home in 2026, including qualifying temporary absences
        /// On a final return that is not joint, count the days through the date of death: more than half of those days is the residence test.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        days: u32,
    },
    /// Qualifying whole-year residency exception
    /// Newborn, death or temporary-absence rule treats residency as the whole year.
    WholeYearException,
}
impl FederalDependent {
    pub(crate) fn days_in_home(&self) -> u32 {
        match self.residency {
            DependentResidency::Days { days } => days,
            DependentResidency::WholeYearException => 365,
        }
    }
}

/// Work-related care expenses for this dependent
/// Choose Yes to enter the amounts and eligibility facts; No has no additional fields.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DependentCare {
    /// No
    No,
    /// Yes
    Yes {
        /// Eligible work-related care costs paid for this person
        /// Include employer-plan reimbursements reported in Credits; the App removes excluded benefits. Exclude other tax-free reimbursements.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        care_expenses: i64,
    },
}
