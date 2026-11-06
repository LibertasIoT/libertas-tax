// Accepted Saver’s Credit contributions and distribution histories.
use crate::common::{Answer, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Retirement distribution period
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum SaverDistributionYear {
    /// 2024
    PriorTwo,
    /// 2025
    PriorOne,
    /// 2026
    TaxYear,
    /// 2027, before the 2026 return due date including extensions
    FollowingYear,
}

/// Retirement distribution for the Saver’s Credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSaverDistribution {
    /// Recipient
    pub owner: Owner,
    /// When received
    pub year: SaverDistributionPeriod,
    /// Distribution amount that counts against the credit
    /// Include retirement-plan, traditional/Roth IRA and ABLE distributions. Exclude rollovers,
    /// conversions, nontaxable trustee-to-trustee transfers, deemed plan loans, timely returned IRA contributions or corrective excess contributions/deferrals and associated earnings,
    /// ESOP dividends, military pensions other than TSP, and nonspousal inherited-IRA distributions.
    /// Enter 2026 taxable income separately in Income; this entry only affects the credit.
    /// Do not repeat 2026 permissible automatic-enrollment withdrawals entered in Income: the App adds those automatically, including earnings. Other counting distributions still belong here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

/// Saver’s Credit for retirement and ABLE contributions
/// Optional: complete this if either filer made qualifying workplace, IRA or beneficiary-funded ABLE contributions.
/// This page calculates a credit only; do not subtract contributions from W-2 box 1 wages again.
/// IRA and section 501(c)(18)(D) contributions are included automatically from Adjustments; do not repeat them in workplace totals.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSaver {
    /// Your qualifying 2026 workplace retirement contributions
    /// Eligible employee contributions within the plan limits, after returned excess amounts: elective deferrals (including designated Roth) to 401(k), 403(b), governmental 457(b), salary-reduction SEP/SIMPLE or TSP plans, or voluntary after-tax employee plan contributions. Exclude employer matches, mandatory employer-treated contributions, rollovers, IRA/ABLE contributions and section 501(c)(18)(D) contributions entered in Adjustments (the App includes those automatically). Include 2026 default elective deferrals from payroll records even when permissibly withdrawn under section 414(w). The App counts the withdrawal separately; do not subtract it from contributions again. Do not include prior-year contributions.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub taxpayer_contributions: i64,
    /// Spouse’s qualifying workplace retirement contributions
    /// Apply the same original-contribution rule: automatic-enrollment withdrawals entered in Income reduce the credit separately. Zero when there are none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub spouse_contributions: i64,
    /// Your contributions to your own ABLE account
    /// Include only your own money contributed as designated beneficiary, within account limits.
    /// Exclude rollovers, returned excess contributions and money contributed by anyone else.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxpayer_able: i64,
    /// Spouse’s contributions to their own ABLE account
    /// Zero when there are none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub spouse_able: i64,
    /// Distributions in the lookback period
    /// Every counting distribution from January 1, 2024 through the 2026 return due date, including extensions; for an estimate, include expected distributions through that date and revisit this page if they change. List each recipient and year separately, including a spouse with no eligible contribution.
    /// ----
    /// Distribution
    pub distributions: Vec<FederalSaverDistribution>,
}

/// When the retirement distribution was received
/// For 2026 the backend uses this return's filing status; other years ask about joint filing.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum SaverDistributionPeriod {
    /// 2024
    PriorTwo {
        /// Filed jointly with the current spouse for this year
        /// The filing-status box on the 2024 Form 1040 shows it.
        joint_with_current_spouse: bool,
    },
    /// 2025
    PriorOne {
        /// Filed jointly with the current spouse for this year
        /// The filing-status box on the 2025 Form 1040 shows it.
        joint_with_current_spouse: bool,
    },
    /// 2026
    TaxYear,
    /// 2027, before the 2026 return due date including extensions
    FollowingYear {
        /// Filed or expect to file jointly with the current spouse for this year
        /// Not sure, while the 2027 status is undecided, is treated as joint: the distribution then also reduces the spouse's credit.
        // A future filing decision has no filed return to look up. Retain this
        // conservative unknown with WARN_SAVER_2027_JOINT_ASSUMED.
        joint_with_current_spouse: Answer,
    },
}
impl FederalSaverDistribution {
    pub(crate) fn year(&self) -> SaverDistributionYear {
        match self.year {
            SaverDistributionPeriod::PriorTwo { .. } => SaverDistributionYear::PriorTwo,
            SaverDistributionPeriod::PriorOne { .. } => SaverDistributionYear::PriorOne,
            SaverDistributionPeriod::TaxYear => SaverDistributionYear::TaxYear,
            SaverDistributionPeriod::FollowingYear { .. } => SaverDistributionYear::FollowingYear,
        }
    }
    pub(crate) fn joint_with_current_spouse(&self) -> Answer {
        match self.year {
            SaverDistributionPeriod::PriorTwo { joint_with_current_spouse }
            | SaverDistributionPeriod::PriorOne { joint_with_current_spouse } => {
                crate::common::yes_no(joint_with_current_spouse)
            }
            SaverDistributionPeriod::FollowingYear { joint_with_current_spouse } => {
                joint_with_current_spouse
            }
            SaverDistributionPeriod::TaxYear => Answer::No,
        }
    }
}

/// Workplace retirement contributions for Saver’s Credit
/// Select Yes to provide the applicable facts; No carries no additional data.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SaverChoice {
    /// No
    No,
    /// Yes
    Yes {
        /// Applicable facts
        facts: FederalSaver,
    },
}
impl SaverChoice {
    pub fn as_ref(&self) -> Option<&FederalSaver> {
        match self {
            Self::No => None,
            Self::Yes { facts } => Some(facts),
        }
    }
    pub fn as_mut(&mut self) -> Option<&mut FederalSaver> {
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
}
