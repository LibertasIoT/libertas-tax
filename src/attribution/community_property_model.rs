#[allow(unused_imports)]
use super::*;

/// Community property
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
#[libertas_inline_choices]
pub enum CommunityProperty {
    /// Not answered yet
    /// A separate return needs this answer; the estimate asks for it until you give it.
    NotAnswered,
    /// No
    /// Neither of you was domiciled in a community-property state or country during 2026.
    No,
    /// Yes
    /// You or your spouse were domiciled in a community-property state or country for all or part of 2026. Your separate return reports half of all community income and all of your own separate income; the income pages ask which each item is (Form 8958, Pub. 555). A rental or Schedule K-1 is entered for the whole interest and divided; a partner's self-employment tax stays the partner's. The Alaska, South Dakota and Tennessee community-property elections are not covered.
    Yes {
        /// State or country of domicile
        state: CommunityState,
        /// How your community income is divided
        rule: CommunityRule,
    },
}

// The question a separate return asks: Yes or No, each a real answer. The
// stored CommunityProperty keeps its third value for a return not asked yet.
/// Community property
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum CommunityAnswer {
    /// No
    /// Neither of you was domiciled in a community-property state or country during 2026.
    No,
    /// Yes
    /// You or your spouse were domiciled in a community-property state or country for all or part of 2026. Your separate return reports half of all community income and all of your own separate income; the income pages ask which each item is (Form 8958, Pub. 555). A rental or Schedule K-1 is entered for the whole interest and divided; a partner's self-employment tax stays the partner's. The Alaska, South Dakota and Tennessee community-property elections are not covered.
    Yes {
        /// State or country of domicile
        state: CommunityState,
        /// How your community income is divided
        rule: CommunityRule,
    },
}

impl From<CommunityAnswer> for CommunityProperty {
    fn from(value: CommunityAnswer) -> Self {
        match value {
            CommunityAnswer::No => Self::No,
            CommunityAnswer::Yes { state, rule } => Self::Yes { state, rule },
        }
    }
}

impl CommunityProperty {
    // The saved answer, or None while the return has not been asked.
    pub(crate) fn answer(self) -> Option<CommunityAnswer> {
        match self {
            Self::NotAnswered => None,
            Self::No => Some(CommunityAnswer::No),
            Self::Yes { state, rule } => Some(CommunityAnswer::Yes { state, rule }),
        }
    }
}

/// State or country of domicile
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CommunityState {
    /// Arizona
    Arizona,
    /// California
    California,
    /// Idaho
    Idaho,
    /// Louisiana
    Louisiana,
    /// Nevada
    Nevada,
    /// New Mexico
    NewMexico,
    /// Texas
    Texas,
    /// Washington
    Washington,
    /// Wisconsin
    Wisconsin,
    /// A community-property country
    Country {
        /// Its law makes income from separate property community income
        /// As in Idaho, Louisiana, Texas and Wisconsin. The country's property law shows this.
        separate_income_is_community: bool,
    },
}

impl CommunityState {
    // Pub. 555 "Income from separate property": community income in Idaho,
    // Louisiana, Texas and Wisconsin, the owner's separate income elsewhere.
    pub(crate) fn separate_income_is_community(self) -> bool {
        match self {
            Self::Idaho | Self::Louisiana | Self::Texas | Self::Wisconsin => true,
            Self::Country { separate_income_is_community } => separate_income_is_community,
            _ => false,
        }
    }
}

/// How your community income is divided
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CommunityRule {
    /// Each of you reports half of every community item
    Halves,
    /// You lived apart all year
    /// You lived apart all of 2026, did not file jointly, at least one of you had community earned income, and none of it passed between you (section 66(a)). Pay goes to the spouse who earned it, a business to the spouse who runs it, a partnership share to the partner, separate-property income to its owner and Social Security to the recipient; other community income is still halved.
    LivingApartAllYear,
    /// Your spouse is a nonresident alien
    /// Your spouse was a nonresident alien in 2026 and you did not choose to treat them as a U.S. resident (section 879). Community income is divided as for spouses living apart all year.
    NonresidentSpouse,
}

/// Community or separate
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
pub enum CommunityShare {
    /// Not needed for this return
    NotNeeded,
    /// Community
    Community,
    /// Separate
    Separate,
    /// Partly community
    Partly {
        /// Community part
        /// The part earned or acquired while married and domiciled in a community-property state, such as the years of pension service during the marriage.
        // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
        #[libertas_number(min = 1, max = 99)]
        #[libertas_physical_unit("percent")]
        percent: u32,
    },
}

/// Management of community funds
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
#[libertas_inline_choices]
pub enum RefundManagement {
    /// Management classification is not needed in this state
    NotApplicable,
    /// Taxpayer's sole-management property
    Taxpayer,
    /// Spouse's sole-management property
    Spouse,
    /// Joint-management property
    Joint,
}
