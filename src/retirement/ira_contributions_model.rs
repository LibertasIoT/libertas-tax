#[allow(unused_imports)]
use super::*;
pub use crate::ira_contribution_sources::{
    FederalIraContribution, FederalIras, IraNextYearDeposits, IraSpouse,
};
/// IRA contributions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraContributions {
    /// Contributions for 2026
    // Tax structure: one consolidated IRA contribution entry per filer.
    #[libertas_size(max = 2)]
    #[libertas_unordered]
    /// ----
    /// Person
    pub people: Vec<IraContributor>,
}
/// Person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraContributor {
    /// Whose contributions
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Traditional IRA
    /// Enter the totals as they ended up after any recharacterization; leave out a contribution returned to you (a 2026 return is its Form 1099-R; a 2027 return is listed below).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub traditional: i64,
    /// Roth IRA
    /// Enter the totals as they ended up after any recharacterization; leave out a contribution returned to you (a 2026 return is its Form 1099-R; a 2027 return is listed below).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub roth: i64,
    /// Covered by a retirement plan at work
    /// Check all employers and self-employment plans, even if no IRA contribution was made.
    #[libertas_read_only]
    #[libertas_default(false)]
    pub covered: bool,
    /// Traditional IRA value on December 31, 2026, from your Form 8606
    /// Entered with your Form 8606 on the Retirement page. An excess record uses it plus your 2026 traditional contributions paid in 2027; change it there.
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_output_only]
    pub form_8606_value: Option<i64>,
    /// More about these contributions
    pub more: IraMore,
}
/// More about these contributions
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct IraMore {
    /// Traditional contributions voluntarily designated nondeductible
    /// Leave zero to claim the maximum allowed deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nondeductible: i64,
    /// Actual 2027 deposits designated for 2026, if any
    /// Enter only completed regular deposits, as a subset of the traditional and Roth totals above.
    /// [NullText]
    /// No completed 2027 regular deposits are designated for 2026
    /// [SetText]
    /// The completed 2027 deposits designated for 2026 are
    pub paid_next_year: Option<IraNextYearDeposits>,
    /// Your spouse is covered by a retirement plan at work
    /// Needed only when you file separately and lived with your spouse at any time in 2026.
    #[libertas_default(false)]
    pub spouse_covered_if_separate: bool,
    /// Excess contributions or a carried 2025 excess, if any
    /// Complete this when your 2026 contributions exceed the limit, or your 2025 Form 5329 showed a traditional or Roth IRA excess. The App computes the limit and the 6% tax.
    /// [NullText]
    /// No current or carried 2025 IRA excess remains
    /// [SetText]
    /// The IRA excess-contribution history and corrections are
    pub excess: Option<crate::IraExcessEntry>,
    /// Contributions returned to you in 2027
    /// A 2026 traditional or Roth contribution the custodian paid back with its earnings or loss from January 1, 2027 through the deadline. Leave its principal out of the totals above; the earnings are 2026 income on Form 1040 line 4b with no additional tax. A contribution returned in 2026 is entered as its Form 1099-R (code 8) on the Retirement page, not here. Leave empty when none.
    /// ----
    /// Returned contribution
    pub returned_in_following_year: Vec<crate::ReturnedContribution>,
    /// 529-to-Roth IRA transfers in 2024 and 2025
    /// Every earlier direct transfer from any 529 account to this person's Roth IRA. This year's transfers are on their Forms 1099-Q.
    #[libertas_money("USD", 2)]
    // Tax structure: Section 529-to-Roth IRA transfers have a $35,000 lifetime limit.
    #[libertas_number(min = 0, max = $QTP_ROTH_LIFETIME_LIMIT_CENTS)]
    #[libertas_default(0)]
    pub qtp_roth_prior: i64,
}
