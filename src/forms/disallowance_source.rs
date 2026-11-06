// Retained credit-disallowance findings and intrinsic eligibility classification.
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) const TAX_YEAR: u32 = 2026;

/// Credits claimed after an earlier disallowance (Form 8862)
/// Answer for each credit the IRS reduced or disallowed for an earlier year. The App fills Form 8862 from the people, children and students already on this return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CreditsAfterDisallowance {
    /// Earned income credit
    #[libertas_default(Never)]
    pub earned_income: EicDisallowance,
    /// Child tax credit, additional child tax credit or credit for other dependents
    #[libertas_default(Never)]
    pub child_credits: CreditDisallowance,
    /// American opportunity tax credit
    #[libertas_default(Never)]
    pub education_credit: CreditDisallowance,
}

/// Earlier disallowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CreditDisallowance {
    /// Never reduced or disallowed
    Never,
    /// Reduced or disallowed for an earlier year
    Disallowed(DisallowanceFacts),
}

/// Earlier disallowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum EicDisallowance {
    /// Never reduced or disallowed
    Never,
    /// Reduced or disallowed for an earlier year
    Disallowed {
        /// The disallowance
        facts: DisallowanceFacts,
        /// What the IRS found
        cause: EicDisallowanceCause,
        /// Time in 2026 your main home was in the United States
        /// Needed only when you claim the credit without a qualifying child. Count temporary absences and extended active military duty as time at home.
        /// [NullText]
        /// I am claiming EIC with a qualifying child; this does not apply
        /// [SetText]
        /// Without a qualifying child, my time at home in the U.S. was
        // Calendar structure: tax year 2026 has 365 days.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        your_us_home_days: Option<u32>,
        /// Time in 2026 your spouse’s main home was in the United States
        /// Needed only on a joint return claiming the credit without a qualifying child.
        /// [NullText]
        /// Spouse home time does not apply to this EIC claim
        /// [SetText]
        /// For our joint claim without a qualifying child, my spouse's time at home in the U.S. was
        // Calendar structure: tax year 2026 has 365 days.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        spouse_us_home_days: Option<u32>,
    },
}

/// The disallowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DisallowanceFacts {
    /// Most recent tax year for which the credit was reduced or disallowed
    /// The year of the return the IRS adjusted, not the year the notice arrived.
    // Form 8862 prior-finding window: 1997 onward, before this 2026 return.
    #[libertas_number(min = 1997, max = 2025)]
    pub year: u32,
    /// The IRS finding
    /// The IRS notice or letter that reduced or disallowed the credit states it; an IRS account transcript also shows it.
    #[libertas_inline_choices]
    pub finding: DisallowanceFinding,
    /// Form 8862 was filed after that year, the credit was allowed again, and it has not been reduced or disallowed since except for a math or clerical error
    #[libertas_default(false)]
    pub reinstated: bool,
}

/// The IRS finding
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DisallowanceFinding {
    /// A math or clerical error only
    MathOrClericalError,
    /// Not eligible, with no finding of reckless disregard or fraud
    Ineligible,
    /// Reckless or intentional disregard of the rules (a 2-year ban)
    RecklessDisregard,
    /// Fraud (a 10-year ban)
    Fraud,
}

/// What the IRS found
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum EicDisallowanceCause {
    /// Only incorrectly reported earned income or investment income
    IncomeReporting,
    /// Only that a child listed on Schedule EIC was not a qualifying child
    ChildNotQualifying,
    /// Another reason, or more than one
    Other,
}

impl CreditsAfterDisallowance {
    // The server twin of the page's range attributes.
    pub(crate) fn valid(&self) -> bool {
        let year_ok = |facts: &DisallowanceFacts| (1997..=2025).contains(&facts.year);
        let days_ok = |days: Option<u32>| days.is_none_or(|d| d <= 365);
        let group_ok = |group: &CreditDisallowance| match group {
            CreditDisallowance::Never => true,
            CreditDisallowance::Disallowed(facts) => year_ok(facts),
        };
        let eic_ok = match &self.earned_income {
            EicDisallowance::Never => true,
            EicDisallowance::Disallowed {
                facts, your_us_home_days, spouse_us_home_days, ..
            } => year_ok(facts) && days_ok(*your_us_home_days) && days_ok(*spouse_us_home_days),
        };
        eic_ok && group_ok(&self.child_credits) && group_ok(&self.education_credit)
    }
}

// What the disallowance facts mean for the 2026 claim of one credit group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Requirement {
    // The form is not needed for this credit group; the reason is noted.
    NotNeeded(NoFormReason),
    // The form is needed and can be filled.
    Required,
    // A ban still covers 2026: the credit cannot be claimed (years, last year).
    Banned(u32, u32),
}

// The instructions' Who Must File table and its exceptions: `first_year` is
// the last tax year the requirement does not reach (1996 for the EIC, 2015
// for the child credits and the AOTC). A ban runs for the 2 or 10 tax years
// after the disallowed year, as in the instructions' example (a disallowed
// 2024 claim bars 2025 and 2026).
pub(crate) fn requirement(facts: &DisallowanceFacts, first_year: u32) -> Requirement {
    if facts.finding == DisallowanceFinding::MathOrClericalError {
        return Requirement::NotNeeded(NoFormReason::MathError);
    }
    if facts.reinstated {
        return Requirement::NotNeeded(NoFormReason::Reinstated);
    }
    if facts.year <= first_year {
        return Requirement::NotNeeded(NoFormReason::BeforeRequirement);
    }
    let years = match facts.finding {
        DisallowanceFinding::RecklessDisregard => 2,
        DisallowanceFinding::Fraud => 10,
        _ => return Requirement::Required,
    };
    if TAX_YEAR <= facts.year + years {
        Requirement::Banned(years, facts.year + years)
    } else {
        Requirement::Required
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NoFormReason {
    MathError,
    Reinstated,
    BeforeRequirement,
    NotClaimed,
    ChildlessException,
}
// The credit groups a still-running ban covers for 2026, each as (years of
// the ban, last tax year it covers). A banned group's credit is zero and
// Form 8862 is not filled for it; the other groups are untouched.
#[derive(Clone, Copy, Default)]
pub(crate) struct Bans {
    pub(crate) eic: Option<(u32, u32)>,
    pub(crate) child: Option<(u32, u32)>,
    pub(crate) aotc: Option<(u32, u32)>,
}

pub(crate) fn bans(claim: Option<&CreditsAfterDisallowance>) -> Bans {
    let ban = |facts: &DisallowanceFacts, first_year: u32| match requirement(facts, first_year) {
        Requirement::Banned(years, last) => Some((years, last)),
        _ => None,
    };
    let Some(claim) = claim else {
        return Bans::default();
    };
    Bans {
        eic: match &claim.earned_income {
            EicDisallowance::Never => None,
            EicDisallowance::Disallowed { facts, .. } => ban(facts, 1996),
        },
        child: match &claim.child_credits {
            CreditDisallowance::Never => None,
            CreditDisallowance::Disallowed(facts) => ban(facts, 2015),
        },
        aotc: match &claim.education_credit {
            CreditDisallowance::Never => None,
            CreditDisallowance::Disallowed(facts) => ban(facts, 2015),
        },
    }
}
