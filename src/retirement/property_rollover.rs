use crate::kernel::phase;
use crate::{FederalSale, GainTerm, SaleAdjustment, SaleUse};
use libertas_macros::*;

/// Distributed investment property sold for a rollover
/// One fully sold lot of nonemployer stock from a qualified employer plan, with no after-tax contributions. Enter its distribution value as gross pension income and the proceeds actually rolled over under Distribution treatment. Do not repeat the sale as a capital-sale entry.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PropertyRollover {
    /// Property distribution date
    #[libertas_date_only]
    pub distributed: u32,
    /// Sale date
    #[libertas_date_only]
    pub sold: u32,
    /// Net sale proceeds
    /// Actual proceeds after sale expenses; may exceed the original distribution value.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub proceeds: i64,
    /// Entire lot and ordinary investment treatment confirmed
    /// Nonemployer investment stock, with no NUA, after-tax employee contributions, collectibles, wash sale, related-party loss, mixed cash allocation, foreign adjustment or special election. All shares in this entry were sold, and any rollover used their actual proceeds within its eligible period. Unsold property or cash substituted for retained stock requires another treatment. Broker and plan statements show it. No leaves this distribution out of the estimate with a notice; that situation is not supported.
    pub confirmed: bool,
}
impl PropertyRollover {
    pub(crate) fn pension(&self, gross: i64, rolled: i64) -> i64 {
        crate::kernel::ratio::fraction(
            gross,
            phase::floor(self.proceeds - rolled, 0),
            self.proceeds.max(1),
        )
    }
    pub(crate) fn sale(&self, gross: i64, rolled: i64) -> FederalSale {
        FederalSale {
            term: GainTerm::Short,
            collectible: false,
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            proceeds: phase::floor(self.proceeds - rolled, 0),
            basis: Some(self.pension(gross, rolled)),
            amt: crate::SaleAmt::SameAsRegular,
            withholding: 0,
            use_kind: SaleUse::Investment { wash_sale: 0 },
            special: crate::SaleSpecial::NoneOfThese,
            adjustment: SaleAdjustment::None,
            basis_reporting: crate::BasisReporting::ReportedToIrs,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        crate::federal_rules::valid_date(self.distributed)
            && crate::federal_rules::valid_date(self.sold)
            && self.sold >= self.distributed
            && (1..=100000000000).contains(&self.proceeds)
    }
}
