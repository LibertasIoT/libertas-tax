pub use crate::saver_source::{
    FederalSaver, FederalSaverDistribution, SaverChoice, SaverDistributionPeriod,
    SaverDistributionYear,
};
// Saver’s Credit editor inputs and retained source exports.
#[allow(unused_imports)]
use super::*;
/// Retirement savings credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SaverCredit {
    /// Workplace plan contributions
    /// From W-2 box 12 codes D, E, F, G, S, AA, BB and EE. IRA contributions come from Adjustments.
    #[libertas_read_only]
    #[libertas_output_only]
    /// ----
    /// Contribution
    pub from_w2: Vec<SaverContribution>,
    /// Retirement distributions in 2024, 2025 or before your 2026 return is due
    /// Include 2026 distributions too; permissible automatic-enrollment withdrawals on your 1099-Rs are counted automatically. Leave the list empty when there were none.
    /// ----
    /// Distribution
    pub distributions: Vec<crate::FederalSaverDistribution>,
    /// Other contributions
    pub more: SaverMore,
}
/// Contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SaverContribution {
    /// Whose contributions
    pub owner: Owner,
    /// Contributions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
}
/// Other contributions
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct SaverMore {
    /// Your contributions to your own ABLE account
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub your_able: i64,
    /// Your spouse's contributions to their own ABLE account
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub spouse_able: i64,
    /// Your workplace plan contributions not on a W-2
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_contributions: i64,
}

impl From<Option<FederalSaver>> for SaverChoice {
    fn from(value: Option<FederalSaver>) -> Self {
        match value {
            None => Self::No,
            Some(facts) => Self::Yes { facts },
        }
    }
}
