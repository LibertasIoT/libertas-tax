use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Section 179 facts for a separate return
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Section179Spouse {
    /// Spouse's records not entered
    /// The qualifying purchase amount is unknown, not zero. An optional current section 179 claim is left out with a note; a Schedule K-1 claim still needs these records.
    NotSupplied,
    /// Spouse's purchase records and allocation
    /// Use the spouse's Form 4562 Part I and any written allocation election. Each spouse applies their own business-income limit. Unresolved ownership or community-property allocation does not establish these amounts; keep the records not entered until resolved. Amending separate returns to a joint return requires additional original-election facts and is not calculated.
    Records {
        /// Spouse's eligible 2026 section 179 business cost
        /// Include all qualifying property placed in service, even when the spouse elects zero deduction. Use the spouse's purchase and business-use records; exclude purchases already included on this return. This cost affects only the shared dollar-limit phaseout, not your income or MACRS convention. Enter zero only when the spouse had no eligible purchases.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        cost: i64,
        /// Allocation of the shared dollar limit after phaseout
        allocation: Section179Share,
    },
}
// The record a separate return adds on its business page: only real
// records. Without it the stored Section179Spouse holds NotSupplied, which
// the rules read as records not given.
/// Spouse's section 179 purchases and allocation
/// Use the spouse's Form 4562 Part I and any written allocation election. Each spouse applies their own business-income limit. Unresolved ownership or community-property allocation does not establish these amounts; leave this record out until resolved. Amending separate returns to a joint return requires additional original-election facts and is not calculated.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SpouseSection179 {
    /// Spouse's eligible 2026 section 179 business cost
    /// Include all qualifying property placed in service, even when the spouse elects zero deduction. Use the spouse's purchase and business-use records; exclude purchases already included on this return. This cost affects only the shared dollar-limit phaseout, not your income or MACRS convention. Enter zero only when the spouse had no eligible purchases.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Allocation of the shared dollar limit after phaseout
    pub allocation: Section179Share,
}
impl SpouseSection179 {
    pub(crate) fn records(&self) -> Section179Spouse {
        Section179Spouse::Records { cost: self.cost, allocation: self.allocation }
    }
}
/// Couple's section 179 dollar-limit allocation
#[derive(Clone, Copy, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Section179Share {
    /// Equal shares; no different allocation elected
    Equal,
    /// Each spouse elected a percentage
    /// Enter each actual elected percentage. If they do not total 100%, the required allocation is 50% each; the App does not normalize mismatched elections.
    Elected {
        /// Your elected share
        #[libertas_number(min = 0, max = 100, step = 0.01)]
        #[libertas_physical_unit("percent")]
        yours: f64,
        /// Spouse's elected share
        #[libertas_number(min = 0, max = 100, step = 0.01)]
        #[libertas_physical_unit("percent")]
        spouse: f64,
    },
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for Section179Share {}
impl Section179Spouse {
    pub(crate) fn valid(self) -> bool {
        match self {
            Self::NotSupplied => true,
            Self::Records { cost, allocation } => {
                (0..=100000000000).contains(&cost)
                    && match allocation {
                        Section179Share::Equal => true,
                        Section179Share::Elected { yours, spouse } => {
                            [yours, spouse].into_iter().all(|share| {
                                crate::kernel::natural::exact(share, 100)
                                    .is_some_and(|share| (0..=10000).contains(&share))
                            })
                        }
                    }
            }
        }
    }
    pub(super) fn facts(self) -> (i64, i64, bool) {
        match self {
            // Missing records do not establish a purchase amount or allocation.
            Self::NotSupplied => (0, 0, false),
            Self::Records { cost, allocation } => (
                cost,
                // Publication 946, Married Individuals: equal allocation unless
                // both elected percentages total 100%.
                // https://www.irs.gov/publications/p946
                match allocation {
                    Section179Share::Elected { yours, spouse }
                        if crate::kernel::natural::units(yours, 100)
                            + crate::kernel::natural::units(spouse, 100)
                            == 10000 =>
                    {
                        crate::kernel::natural::units(yours, 100)
                    }
                    _ => 5000,
                },
                true,
            ),
        }
    }
}
