#[allow(unused_imports)]
use super::*;
pub use crate::accepted_adjustments::FederalAdjustments;

// The adjustments hub's last topic, shaped like More credits:
// each kind carries its stored record.
/// More adjustments
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MoreAdjustments {
    /// More adjustments
    #[libertas_unordered]
    /// ----
    /// Adjustment
    pub adjustments: Vec<MoreAdjustment>,
}
/// Adjustment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MoreAdjustment {
    /// Alimony paid
    /// Under a divorce or separation agreement signed before 2019. One for each agreement.
    AlimonyPaid(AlimonyItem),
    /// Moving expenses under military orders or an intelligence-community reassignment (Form 3903)
    /// One for each move. The person who moved must be on this return.
    MovingExpenses(MovingItem),
}
/// Alimony paid
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// The recapture rules ALIMONY_2024/2025_EXCLUDED_ABOVE_PAID are declared on
// AlimonyPayment, shared with the "Alimony received" income kind.
pub struct AlimonyItem {
    /// Who paid
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Agreement and payments
    pub payment: crate::AlimonyPayment,
}

/// Moving expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MovingItem {
    /// Who moved
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// The move
    pub record: crate::MilitaryMove,
}
