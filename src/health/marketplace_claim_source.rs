use crate::kernel::ratio::dollars;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Poverty guideline residence
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MarketplaceResidence {
    /// Contiguous states or District of Columbia
    Contiguous,
    /// Alaska
    Alaska,
    /// Hawaii
    Hawaii,
}
/// Marketplace premium tax credit reconciliation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MarketplaceClaim {
    /// No Marketplace coverage or advance credits
    No,
    /// Canonical statement sources and direct tax facts
    Yes {
        /// Original source records; no calculated monthly ledger is persisted
        facts: alloc::boxed::Box<crate::MarketplaceFacts>,
    },
}
/// Dependent return used for Marketplace household income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceDependent {
    /// Dependent required to file because of income, not merely to recover withholding
    /// By the identity of a person on the return.
    pub person: i64,
    /// Form 1040 adjusted gross income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub agi: i64,
    /// Form 1040 tax-exempt interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt_interest: i64,
    /// Form 2555 excluded foreign income and housing, lines 45 and 50
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub foreign_exclusion: i64,
    /// Gross Social Security, Form 1040 line 6a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub social: i64,
    /// Taxable Social Security, Form 1040 line 6b
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub taxable_social: i64,
}
impl MarketplaceDependent {
    pub(crate) fn income_cents(&self) -> i64 {
        self.agi + self.exempt_interest + self.foreign_exclusion + self.social - self.taxable_social
    }
    pub(crate) fn income(&self) -> i64 {
        dollars(self.agi)
            + dollars(self.exempt_interest)
            + dollars(self.foreign_exclusion)
            + dollars(self.social)
            - dollars(self.taxable_social)
    }
}
pub(crate) fn facts(claim: &MarketplaceClaim) -> Option<&crate::MarketplaceFacts> {
    match claim {
        MarketplaceClaim::No => None,
        MarketplaceClaim::Yes { facts } => Some(facts),
    }
}
