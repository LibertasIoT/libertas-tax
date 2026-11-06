use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Interest for the time after the owner's death
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InterestAfterDeath {
    /// Part of box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// Part of box 3
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Part of box 8
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private activity belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,

    /// Foreign tax belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,

    /// Bond premium belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,

    /// Treasury bond premium belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,

    /// Tax exempt bond premium belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,

    /// Accrued interest belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub accrued_interest: i64,

    /// Market discount belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
}

/// Dividends for the time after the owner's death
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DividendsAfterDeath {
    /// Part of box 1a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Part of box 1b
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified: i64,
    /// Part of box 2a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub capital_distributions: i64,
    /// Part of box 12
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt_dividends: i64,
    /// Unrecaptured distributions belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_distributions: i64,

    /// Collectible distributions belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectible_distributions: i64,

    /// Section 1202 gain belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_1202_gain: i64,

    /// Section199a belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section199a: i64,

    /// Private activity dividends belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity_dividends: i64,

    /// Foreign tax belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,

    /// Section 897 ordinary belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_897_ordinary: i64,

    /// Section 897 capital belonging to the later recipient
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_897_capital: i64,
}

impl InterestAfterDeath {
    pub(crate) fn income(&self) -> i64 {
        self.taxable + self.us_obligations + self.market_discount
    }
}
impl DividendsAfterDeath {
    pub(crate) fn income(&self) -> i64 {
        self.ordinary + self.capital_distributions
    }
}
