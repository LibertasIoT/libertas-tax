pub use crate::minimum_tax_source::{
    MinimumTaxExclusion, MinimumTaxForeignCategory, MinimumTaxForeignCredit,
    MinimumTaxForeignEarned, MinimumTaxForeignGainSource, MinimumTaxGains, MinimumTaxHistories,
    MinimumTaxHistory, MinimumTaxHistoryUnit, MinimumTaxLoss, MinimumTaxLossAmount,
    MinimumTaxLossRecords, MinimumTaxQualifiedRateFacts, MinimumTaxRateFacts, MinimumTaxReturnForm,
    MinimumTaxScheduleDRateFacts, OwnedMinimumTaxHistory,
};

// This adapts an already-computed historical rate worksheet for §1341 source
// propagation. It does not derive retained source facts from a credit amount.
impl From<MinimumTaxGains> for MinimumTaxRateFacts {
    fn from(v: MinimumTaxGains) -> Self {
        Self::ScheduleD(MinimumTaxScheduleDRateFacts {
            preferential: v.preferential,
            unrecaptured: v.combined - v.preferential,
            gain_cap: v.combined,
            regular_zero: v.regular_zero,
            regular_fifteen: v.regular_fifteen,
        })
    }
}
