// Coverage: Form 8594 allocation of a whole-business sale (unit B11 4797, 2026-10-11).
//   Pub 544 p. 34 ("each asset is treated as being sold separately"): the depreciable assets and
//   the building are the business's own rows (asset_sale.rs); this record holds the three amounts of
//   the allocation that are not an asset row. Inventory is ordinary income counted with the
//   business's gross receipts (Schedule C line 1, self-employment tax and qualified business income
//   like other sales; a bulk sale is still property held for sale to customers, Pub 544 p. 41).
//   Self-created goodwill and going-concern value is a capital asset (section 1221) with basis
//   zero: one Form 8949 box F row, long term, not section 1231 and not qualified business income
//   (Reg. 1.199A-3(b)(2)(ii)(A)). A covenant not to compete is ordinary income on Schedule 1 line
//   8z (not self-employment income). Purchased goodwill and other section 197 intangibles stay
//   refused (their sale is section 1245 recapture of the amortization and section 1231, with the
//   section 197(f)(1) no-loss rule), as does a Form 8594 the buyer reports differently
//   (`special`, left out with WARN_BUSINESS_SALE_LEFT_OUT). Tests: src/tests/investment/business.rs
//   business_sale_inventory_goodwill_and_covenant_have_their_own_treatment.
//   https://www.irs.gov/publications/p544 https://www.irs.gov/instructions/i8594
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Sale of the business as a whole (Form 8594)
/// Enter the sale of each asset, vehicle and building on its own row above; this record holds the rest of the allocation agreed with the buyer. Purchased goodwill or other section 197 intangibles you bought with the business are not calculated: choose Something else.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessSale {
    /// Date sold
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Price for inventory
    /// Ordinary business income, counted with your sales.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub inventory: i64,
    /// Price for goodwill and going-concern value you built up
    /// A long-term capital gain (basis zero). Not for goodwill you bought with the business.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub goodwill: i64,
    /// Payment for your covenant not to compete
    /// Ordinary income, not subject to self-employment tax.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub covenant: i64,
    /// Purchased goodwill or other section 197 intangibles sold, an installment sale of these amounts, or an allocation the buyer reports differently
    /// Yes leaves this record out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

impl BusinessSale {
    pub(crate) fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.date)
            && crate::federal_rules::valid_date(self.date)
            && [self.inventory, self.goodwill, self.covenant]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
    }
    // Whether the record is calculated: a record with the something-else
    // answer is left out before the rules run (unknowns.rs).
    pub(crate) fn supported(&self) -> bool {
        self.valid() && !self.special
    }
}
