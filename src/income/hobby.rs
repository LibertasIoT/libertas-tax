// Coverage: Schedule 1 line 8j hobby sales — Supported (i1040gi (2025), Pub.
//   525 "Activity not for profit", 26 U.S.C. 183 and 67(h) as amended by P.L. 119-21 section
//   70110, Reg. 1.61-3(a)).
//   Lines: the Other income page's Sales from a hobby kind (HobbySales): receipts less the cost
//   of goods sold from cost-valued inventory, floored at zero (no loss), on line 8j; no
//   self-employment tax, QBI or hobby deductions (section 183(b)(2) expenses are miscellaneous
//   itemized deductions, disallowed for 2026). Withholding comes only from the Forms 1099 that
//   relate to the hobby (a Form 1099-K part adds to the receipts). A sale that is not a
//   hobby goods sale (ordinary No) is left out with WARN_HOBBY_SALES_LEFT_OUT.
//   Evidence: this module, federal_rules.rs, unknowns.rs; tests/relief_income.rs
//   i_hobby_sales_* (fixture I: 10,000 less 6,000 cost = 4,000, tax 5,503), tests/business.rs
//   hobby_goods_sales_use_inventory_cost_without_business_deductions_or_losses.
use crate::kernel::phase;
use crate::{BusinessInventory, RelatedForms};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Goods sold from an activity not engaged in for profit
/// Use sales and cost records. A Form 1099-K for these sales is entered in Documents; relate its parts to this hobby here. Personal collectibles/capital assets, services, rentals and royalties need their own income treatment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HobbySales {
    /// Forms related to these sales
    pub related: RelatedForms,
    /// Sales receipts after customer returns and allowances
    /// From every payment channel, except the parts of a Form 1099-K entered in Documents and related to this hobby; the App adds those. Do not subtract platform fees or other hobby operating expenses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub receipts: i64,
    /// Inventory and production costs
    /// Use a consistent accepted accounting method. Other hobby expenses do not reduce this income; own labor is not an inventory cost.
    #[libertas_default(None)]
    pub inventory: BusinessInventory,
    /// These are hobby goods sales, not a business or an asset sale
    /// Not a for-profit business, personal collectible/capital-asset sale, service activity, rental or royalty, and no special cost or disposition treatment. An activity run in a businesslike way to make a profit, with records, regular time and a profit history, is a business; enter it in Self-employment instead. No leaves these sales out of the estimate with a notice; that situation is not supported here.
    pub ordinary: bool,
}
impl crate::related_forms::Activity for HobbySales {
    const KINDS: &'static [crate::related_forms::DocumentKind] = crate::related_forms::HOBBY_KINDS;
    fn related(&self) -> &RelatedForms {
        &self.related
    }
    fn related_mut(&mut self) -> &mut RelatedForms {
        &mut self.related
    }
}

/// Accepted hobby goods sales
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalHobbySales {
    /// Related source component identities
    pub related: Vec<i64>,
    /// Sales receipts outside related forms
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub receipts: i64,
    /// Inventory and production costs
    pub inventory: BusinessInventory,
    /// These are hobby goods sales
    pub ordinary: bool,
}
impl FederalHobbySales {
    pub(crate) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.receipts) && self.inventory.valid()
    }
    pub(crate) fn income(&self) -> i64 {
        phase::floor(self.receipts - self.inventory.cost(), 0)
    }
    // What the sales add to income: nothing for a firm the estimate leaves out.
    pub(crate) fn on_return(&self) -> i64 {
        if self.ordinary { self.income() } else { 0 }
    }
}
