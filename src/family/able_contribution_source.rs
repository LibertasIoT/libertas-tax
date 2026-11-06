// ABLE contribution source facts and intrinsic validation.
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// ABLE contributions for one designated beneficiary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AbleContributions {
    /// Account beneficiary on this return
    pub owner: crate::Owner,
    /// Beneficiary's own cash contributions for 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub own: i64,
    /// Other contributions and section 529 tuition-plan rollovers for 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other: i64,
    /// Working-beneficiary additional allowance
    pub work: AbleWork,
    /// Account value on December 31, 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub year_end: i64,
    /// Additional 2026 account still open
    #[libertas_default(false)]
    pub additional_open: bool,
}
/// Working-beneficiary allowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AbleWork {
    /// No additional working-beneficiary allowance
    No,
    /// Employed or self-employed beneficiary
    Working {
        /// Beneficiary's compensation for 2026 under section 219(f)(1)
        /// Use qualifying compensation records, including applicable self-employment adjustments; do not include investment income, pensions or a spouse's compensation.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        compensation: i64,
        /// State of residence for the poverty guideline
        residence: crate::MarketplaceResidence,
        /// No contribution to a disqualifying retirement plan for this beneficiary for 2026
        /// This includes contributions by anyone to a defined-contribution plan under section 401(a) or 403(a), a 403(b) annuity or a governmental 457(b) plan. Check the beneficiary's Form W-2 box 13 "Retirement plan" and plan statements. No removes the additional allowance.
        no_plan: bool,
    },
}
impl AbleContributions {
    pub(crate) fn valid(&self) -> bool {
        [self.own, self.other, self.year_end].iter().all(|v| (0..=100000000000).contains(v))
            && match self.work {
                AbleWork::No => true,
                AbleWork::Working { compensation, .. } => {
                    (0..=100000000000).contains(&compensation)
                }
            }
    }
}
pub(crate) fn valid_contributions(values: &[AbleContributions]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, v)| v.valid() && !values[..i].iter().any(|old| old.owner == v.owner))
}
