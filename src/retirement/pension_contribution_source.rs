use crate::common::Owner;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Employee-funded section 501(c)(18)(D) contributions
/// Enter one annual record per filer, combining all qualifying trusts. These contributions remain in W-2 box 1 wages; the App deducts them separately and includes them in the Saver's Credit contribution base. They also establish workplace-plan coverage for IRA deduction limits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PensionContributions {
    /// Employee
    pub owner: Owner,
    /// Total 2026 contributions reported with W-2 box 12 code H
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub contributions: i64,
    /// Employee's annual section 415(c)(3) compensation included in gross income
    /// Use the employer's compensation records. Do not enter household income, a spouse's compensation or a calculated 25% limit. Wages remain entered in Income without subtracting code H.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub compensation: i64,
    /// Other regular elective deferrals sharing the annual section 402(g) limit
    /// Combine this employee's 401(k), TSP, 403(b), SARSEP and SIMPLE deferrals, including designated Roth amounts. Exclude employer contributions, governmental 457 deferrals and separately established permitted catch-up amounts. Do not repeat code H contributions here. This input checks the shared limit; it does not automatically claim a Saver's Credit for those other deferrals.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other_deferrals: i64,
    /// Source amounts and ordinary contribution treatment confirmed
    /// Employee-funded qualified section 501(c)(18)(D) trusts only, with all annual contributions and compensation entered. No unresolved excess contributions, corrective distributions, prior-year allocation or unverified catch-up classification. The App calculates the $7,000, 25% compensation and shared annual limits. W-2 box 12 code H and the plan's documents show it; No leaves the deduction out of this estimate.
    pub confirmed: bool,
}
impl PensionContributions {
    pub(crate) fn valid(&self) -> bool {
        [self.contributions, self.compensation, self.other_deferrals]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
    }
}
