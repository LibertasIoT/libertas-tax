use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Federal filing status
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalStatus {
    /// Single
    Single,
    /// Married filing jointly
    Joint,
    /// Married filing separately
    Separate,
    /// Head of household
    Head,
    /// Qualifying surviving spouse
    Surviving,
}

/// Living arrangements with your spouse during 2026
/// Select the period that describes when your spouse lived apart from you.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum SpouseLiving {
    /// Apart at year-end under a qualifying written separation agreement
    /// You remained married for federal filing status, but were legally separated under state law under a written separation agreement or decree of separate maintenance and did not share a household at year-end. You did share a home during part of the last six months; otherwise choose that longer separation period. A divorce or decree making you unmarried belongs in the Unmarried marital situation. This choice supports the EIC exception, not head-of-household or care-credit eligibility.
    AgreementAtYearEnd,
    /// Lived together during some of the last six months
    Together,
    /// Apart for the last six months, but not all year
    LastSixMonths,
    /// Apart for the entire year
    AllYear,
}

/// Marital situation for the 2026 return
/// Only the selected situation asks its applicable spouse questions.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MaritalState {
    /// Unmarried, divorced, or legally separated
    Unmarried,
    /// Married
    /// Includes the year a spouse died when the survivor has not remarried: the death is entered on the About you pages and the joint return covers the decedent's income to the death and the survivor's whole year.
    Married {
        /// Living arrangements with your spouse
        living: SpouseLiving,
    },
    /// Married to a nonresident alien who is not treated as a U.S. resident
    /// Married for every rule except head of household, where you count as unmarried (section 2(b)(2)(B)).
    MarriedNonresidentSpouse,
    /// Spouse died in 2025; not remarried
    WidowedPriorYear {
        /// Could file jointly for your spouse’s year of death
        /// The year-of-death return's filing status shows it, or your marriage and both spouses' citizenship or residency that year.
        could_file_joint: bool,
    },
    /// Spouse died in 2024; not remarried
    WidowedTwoYearsPrior {
        /// Could file jointly for your spouse’s year of death
        /// The year-of-death return's filing status shows it, or your marriage and both spouses' citizenship or residency that year.
        could_file_joint: bool,
    },
}

/// Relationship to you
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Relationship {
    /// Child, adopted child or stepchild
    Child,
    /// Eligible foster child placed by an authorized agency or court
    Foster,
    /// Grandchild or other descendant
    Grandchild,
    /// Sibling or stepsibling
    Sibling,
    /// Niece or nephew
    NieceNephew,
    /// Biological or adoptive parent
    Parent,
    /// Grandparent, stepparent, aunt, uncle, or qualifying in-law (not a cousin)
    OtherRelative,
    /// Unrelated household member
    Unrelated,
}

/// Dependency on another return
/// Apply the dependency tests using household support, residence and income records and the other taxpayer’s filing plans.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FilerDependency {
    /// Nobody can claim this filer
    Independent,
    /// Someone can claim this filer
    CanBeClaimed {
        /// Will someone actually claim this filer on their 2026 return?
        /// Confirm the other taxpayer’s return or filing plans. Education credits depend on the actual claim. The standard deduction, HSA and Saver’s Credit use eligibility to claim, even if nobody claims the filer.
        claimed: bool,
    },
}
impl FilerDependency {
    pub(crate) fn claimable(&self) -> bool {
        matches!(self, Self::CanBeClaimed { .. })
    }
    pub(crate) fn claimed(&self) -> bool {
        matches!(self, Self::CanBeClaimed { claimed: true })
    }
}

impl MaritalState {
    // Married at the end of 2026 for every rule but head of household.
    pub(crate) fn married(&self) -> bool {
        matches!(self, Self::Married { .. } | Self::MarriedNonresidentSpouse)
    }
}
