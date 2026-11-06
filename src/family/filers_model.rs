#[allow(unused_imports)]
use super::*;

pub use crate::accepted_filers::{
    FederalPeople, FederalPerson, FederalSetup, FilingChoice, FinalReturn, SeparateSpouseBoxes,
    SpouseFiler,
};
pub(crate) use crate::accepted_filers::{
    after_65th_birthday, age_at_year_end, age_on, attained_on, birthday_age_at_close_or_death,
};
pub use crate::filer_facts::{
    FederalStatus, FilerDependency, MaritalState, Relationship, SpouseLiving,
};

// Interview pages are projections of the stored draft: the setup
// page maps onto `FederalSetup` and each filer page onto one `FederalPerson`
// (see `interview.rs`); the draft keeps its shape as the calculation contract.

/// Your 2026 return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ReturnSetup {
    /// Filing status
    pub filing: Filing,
}

/// Filing status
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum Filing {
    /// Single
    Single,
    /// Married filing jointly
    MarriedJointly,
    /// Married filing separately
    MarriedSeparately,
    /// Head of household
    HeadOfHousehold {
        /// You paid more than half the cost of keeping up your home in 2026
        /// Head of household needs Yes. Answer from your home's rent, mortgage, tax, upkeep and food costs.
        paid_over_half_home: bool,
        /// Marital situation
        marital: HeadMarital,
    },
    /// Qualifying surviving spouse
    QualifyingSurvivingSpouse {
        /// Year your spouse died
        spouse_died: DeathYear,
        /// You paid more than half the cost of keeping up your home in 2026
        /// Qualifying surviving spouse needs Yes. Answer from your home's rent, mortgage, tax, upkeep and food costs.
        paid_over_half_home: bool,
        /// You could have filed a joint return for the year your spouse died
        could_file_joint: bool,
    },
}

/// Marital situation
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum HeadMarital {
    /// Unmarried, divorced or legally separated at the end of 2026
    Unmarried,
    /// Married, but your spouse did not live with you during the last six months of 2026
    /// Your child, stepchild or foster child must have lived with you.
    LivingApart,
    /// Married to a nonresident alien
    /// Your spouse was a nonresident alien at some time in 2026 and you did not choose to treat them as a U.S. resident; you count as unmarried for head of household.
    NonresidentSpouse,
}

// The draft has no separate death-year type: the year is the choice between
// `MaritalState::WidowedPriorYear` and `MaritalState::WidowedTwoYearsPrior`.
/// Year your spouse died
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum DeathYear {
    /// 2025
    DiedPriorYear,
    /// 2024
    DiedTwoYearsPrior,
}

/// Filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Filer {
    /// Your first name and middle initial
    /// As shown on your Social Security card; this is what Form 1040 prints.
    #[libertas_size(min = 1)]
    pub first_name: String,
    /// Your last name
    #[libertas_size(min = 1)]
    pub last_name: String,
    /// Date of birth
    #[libertas_date_only]
    pub birth_date: u32,
    /// Taxpayer identification
    /// The number itself is never entered here.
    pub identification: Identification,
    /// More about you
    pub details: FilerDetails,
    /// Your whole return
    /// Does any of these apply to your return?
    #[libertas_bitflags(YourSituations)]
    pub situations: u32,
}

/// More about you
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FilerDetails {
    /// Legally blind at the end of 2026 or at death
    pub blind: bool,
    /// Could anyone claim you as a dependent?
    pub claimable: Claimable,
    /// Is this your final return?
    pub more: MoreAboutYou,
    /// Real estate professional
    /// Choose “Not a real estate professional” unless more than half of your working hours, and more than 750 hours, were in real property trades or businesses in which you materially participated (Pub 925). Your work in a rental or K-1 rental activity is then chosen on that rental. Your records of hours and the activities worked show it.
    /// [NullText]
    /// Not a real estate professional
    /// [SetText]
    /// Real estate professional with these rental facts
    #[libertas_default($null)]
    pub real_estate_professional: Option<ProfessionalRentalsPage>,
}

// The spouse page: the same answers as `Filer`, labelled for the spouse so a
// name or claim question can never be read as the filer's own. The interview
// converts it to `Filer` and back.
/// Spouse
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SpouseAnswers {
    /// Spouse's first name and middle initial
    /// As shown on your spouse's Social Security card; this is what Form 1040 prints.
    #[libertas_size(min = 1)]
    pub first_name: String,
    /// Spouse's last name
    #[libertas_size(min = 1)]
    pub last_name: String,
    /// Spouse's date of birth
    #[libertas_date_only]
    pub birth_date: u32,
    /// Spouse's taxpayer identification
    /// The number itself is never entered here.
    pub identification: Identification,
    /// More about your spouse
    pub details: SpouseDetails,
    /// Your joint return
    /// Does any of these apply to your joint return?
    #[libertas_bitflags(SpouseSituations)]
    pub situations: u32,
}

/// More about your spouse
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SpouseDetails {
    /// Legally blind at the end of 2026 or at death
    pub blind: bool,
    /// Could anyone claim your spouse as a dependent?
    pub claimable: Claimable,
    /// Is this your spouse's final return?
    pub more: MoreAboutYou,
    /// Real estate professional
    /// Choose “Spouse is not a real estate professional” unless more than half of your spouse's working hours, and more than 750 hours, were in real property trades or businesses in which your spouse materially participated (Pub 925). The test applies to each spouse separately. Your spouse's work in a rental or K-1 rental activity is then chosen on that rental.
    /// [NullText]
    /// Spouse is not a real estate professional
    /// [SetText]
    /// Spouse is a real estate professional with these rental facts
    #[libertas_default($null)]
    pub real_estate_professional: Option<ProfessionalRentalsPage>,
}

impl From<SpouseAnswers> for Filer {
    fn from(v: SpouseAnswers) -> Self {
        Filer {
            first_name: v.first_name,
            last_name: v.last_name,
            birth_date: v.birth_date,
            identification: v.identification,
            details: FilerDetails {
                blind: v.details.blind,
                claimable: v.details.claimable,
                more: v.details.more,
                real_estate_professional: v.details.real_estate_professional,
            },
            situations: v.situations,
        }
    }
}

impl From<Filer> for SpouseAnswers {
    fn from(v: Filer) -> Self {
        SpouseAnswers {
            first_name: v.first_name,
            last_name: v.last_name,
            birth_date: v.birth_date,
            identification: v.identification,
            details: SpouseDetails {
                blind: v.details.blind,
                claimable: v.details.claimable,
                more: v.details.more,
                real_estate_professional: v.details.real_estate_professional,
            },
            situations: v.situations,
        }
    }
}

/// Final return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum MoreAboutYou {
    /// No
    Nothing,
    /// Yes, died in 2026
    /// This return is the person's final return, for January 1 through the date of death. Enter only statements issued to this person, and for a business, farm or rental only amounts received and paid before the death; a statement issued to the estate or a beneficiary belongs on their return. On a joint return the surviving spouse's income covers all of 2026, and the survivor must not have remarried in 2026. The "Deceased" box, signatures and Form 1310 are not prepared here.
    Died {
        /// Date of death
        #[libertas_date_only]
        date: u32,
        /// Medical expenses paid by the estate that the executor elects to deduct here
        /// This person's own 2026 medical expenses, incurred before the death and paid out of the estate within one year after it (section 213(c)). Electing deducts them on this return, if deductions are itemized and above 7.5% of income, instead of on the estate tax return; the executor attaches the statement and waiver. Not a spouse's or dependent's expenses, not funeral costs and not amounts entered under medical expenses. Zero means no election.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        estate_medical: i64,
        /// Series EE and I savings bonds held at death
        /// Choose “Held no Series EE or I savings bonds at death” when this person held none. The bond records and earlier returns show whether interest was reported each year.
        /// [NullText]
        /// Held no Series EE or I savings bonds at death
        /// [SetText]
        /// Held these Series EE or I savings bonds at death
        bonds: Option<FinalBonds>,
        /// No other death-related adjustment applies
        /// None of these: passive losses released by the death, suspended S corporation losses, tax forgiven for a combat-zone, terrorist-action or astronaut death, a deduction for estate tax on income from an earlier decedent, self-employment from a partnership through the end of the month of death, or deductible payments the estate made after the death that a Form 1098, 1098-E or 1098-T in this person's name includes. The decedent's records and any estate tax return show this. No calculates the return without those adjustments and adds a notice; they are not supported.
        ordinary: bool,
    },
}

/// Taxpayer identification
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum Identification {
    /// Social Security number valid for work
    SocialSecurityNumber,
    /// ITIN, ATIN, or a Social Security number not valid for work
    Itin,
    /// No number issued by the return's due date
    NoneByDueDate,
}

/// Claimable
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Claimable {
    /// Nobody
    Nobody,
    /// Possibly
    Possibly {
        /// This filer is another taxpayer’s qualifying child for EIC, even if not their dependent
        /// Apply the relationship, age or permanent-disability, U.S. residence and joint-return tests.
        /// Do not count a person who need not file and files no return or only to recover withholding or estimated payments.
        /// A joint return claiming EIC is not a refund-only return. Check household residence, support and filing records and the other taxpayer’s return.
        another_eitc_child: bool,
        /// Dependency on another return
        dependency: FilerDependency,
        /// Main home was in the United States for more than half this filer's tax year
        /// Normally January 1 through December 31, 2026. For a final return, use January 1 through this filer's death. Include qualifying temporary absences and extended active military duty. Count the days at your U.S. home from travel and address records.
        us_home_over_half_year: bool,
        /// Full-time student during part of at least five calendar months in this filer's tax year
        /// Include qualifying full-time on-farm training; exclude on-the-job, correspondence and internet-only schools.
        full_time_student: bool,
        /// Permanently and totally disabled
        disabled: bool,
        /// Physically or mentally incapable of self-care
        incapable_self_care: bool,
        /// Earned income compared with this filer’s total support
        support_share: SupportShare,
        /// At least one parent was alive at the close of this filer's tax year
        /// Use the date of death on a final return; otherwise December 31, 2026.
        parent_alive: bool,
    },
}

// The situations of the whole return, asked beside the filers' own facts with
// "None of these": the filer's on About you, the joint ones on the spouse's
// page (a joint return only). Each page stores its answer in its filer's
// FederalPerson.whole_return; the bits of the two catalogs never overlap, so
// FederalDraft::whole_return reads their union.
/// Your whole return
#[libertas_bitflags]
pub struct YourSituations(pub u32);
impl YourSituations {
    /// None of these
    pub const NONE: Self = Self(0);
    /// I was not a U.S. citizen or resident for all of 2026, or I lived in a U.S. territory
    /// A dual-status or part-year resident return (Form 1040-NR with Form 1040), a nonresident alien, or income from Puerto Rico, Guam, American Samoa, the Northern Mariana Islands or the U.S. Virgin Islands (Forms 4563 and 8689). Not when you and your spouse choose to be treated as U.S. residents for all of 2026: choose that item on your spouse's page instead.
    pub const DUAL_STATUS: Self = Self(1 << 0);
    /// I share community property with a registered domestic partner
    /// Registered domestic partners in a community property state split community income on Form 8958.
    pub const COMMUNITY_PROPERTY: Self = Self(1 << 1);
}
/// Your joint return
#[libertas_bitflags]
pub struct SpouseSituations(pub u32);
impl SpouseSituations {
    /// None of these
    pub const NONE: Self = Self(0);
    /// One of us was a nonresident alien in 2026 and we choose to be treated as U.S. residents for the whole year
    /// The section 6013(g) choice for a nonresident spouse, or the section 6013(h) choice for a spouse who became a U.S. resident during 2026, made on this joint return or still in effect from an earlier year. Both spouses' worldwide income is taxed.
    pub const NONRESIDENT_SPOUSE: Self = Self(1 << 2);
    /// I want to keep my share of a joint refund from my spouse's debts
    /// Injured spouse allocation (Form 8379), when the refund may go to your spouse's past-due child support, federal or state debt.
    pub const INJURED_SPOUSE: Self = Self(1 << 3);
    /// I am asking for relief from tax on this joint return (innocent spouse)
    /// Innocent spouse, separation of liability or equitable relief (Form 8857).
    pub const INNOCENT_SPOUSE: Self = Self(1 << 4);
    /// We also choose to apply the joint return election to the net investment income tax
    /// Only with the nonresident spouse choice above (Form 8960 election box). Without this item the net investment income tax is figured for the one of you who was a U.S. citizen or resident, on that person's own net investment income and modified AGI against $125,000, as if filing separately. With it, your joint amounts and the joint threshold apply.
    pub const NIIT_JOINT_ELECTION: Self = Self(1 << 5);
    /// I was the nonresident alien; my spouse is the U.S. citizen or resident
    /// Only with the nonresident spouse choice above, and used only when you do not choose the joint election for the net investment income tax: that tax then uses your spouse's own figures. Without this item, your spouse was the nonresident alien.
    pub const YOU_WERE_THE_NONRESIDENT: Self = Self(1 << 6);
}
