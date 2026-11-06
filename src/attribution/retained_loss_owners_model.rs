use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual owners of this retained source amount
/// Use the retained records. This answer never changes the joint amount or applies a proportional allocation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredRetainedOwners {
    /// Taxpayer
    Taxpayer,
    /// Spouse
    Spouse,
    /// Both spouses, with these actual amounts
    Both {
        /// Taxpayer's actual amount from the retained records
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        taxpayer: i64,
        /// Spouse's actual amount from the retained records
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        spouse: i64,
    },
}

impl InjuredRetainedOwners {
    pub(crate) fn amounts(&self, total: i64) -> [i64; 2] {
        match *self {
            Self::Taxpayer => crate::kernel::attribution::exclusive(total, crate::Owner::Taxpayer),
            Self::Spouse => crate::kernel::attribution::exclusive(total, crate::Owner::Spouse),
            Self::Both { taxpayer, spouse } => [taxpayer, spouse],
        }
    }
    pub(crate) fn valid(&self, total: i64) -> bool {
        let amounts = self.amounts(total);
        (0..=100000000000).contains(&total)
            && amounts.iter().all(|n| (0..=100000000000).contains(n))
            && amounts[0] + amounts[1] == total
            && !matches!(self, Self::Both { taxpayer, spouse } if *taxpayer == 0 || *spouse == 0)
    }
}

/// Ownership of the surviving joint capital loss
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredCapitalLossOwners {
    /// Actual surviving amounts from each spouse's loss records
    Shares {
        /// Taxpayer's short-term loss carried to 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        taxpayer_short: i64,
        /// Spouse's short-term loss carried to 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse_short: i64,
        /// Taxpayer's long-term loss carried to 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        taxpayer_long: i64,
        /// Spouse's long-term loss carried to 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse_long: i64,
    },
}
impl InjuredCapitalLossOwners {
    pub(crate) fn amounts(&self) -> [[i64; 2]; 2] {
        match *self {
            Self::Shares { taxpayer_short, spouse_short, taxpayer_long, spouse_long } => {
                [[taxpayer_short, taxpayer_long], [spouse_short, spouse_long]]
            }
        }
    }
    pub(crate) fn valid(&self, carry: [i64; 2]) -> bool {
        let amounts = self.amounts();
        amounts.iter().flatten().all(|n| (0..=100000000000).contains(n))
            && [0, 1].iter().all(|i| amounts[0][*i] + amounts[1][*i] == carry[*i])
    }
}

/// Ownership of a surviving regular net operating loss
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredNolOwners {
    /// Actual surviving balances belonging to each spouse
    Shares {
        /// Taxpayer's unused regular loss from this year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        taxpayer: i64,
        /// Spouse's unused regular loss from this year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse: i64,
    },
}
impl InjuredNolOwners {
    pub(crate) fn amounts(&self) -> [i64; 2] {
        match *self {
            Self::Shares { taxpayer, spouse } => [taxpayer, spouse],
        }
    }
    pub(crate) fn valid(&self, balance: i64) -> bool {
        let amounts = self.amounts();
        amounts.iter().all(|n| (0..=100000000000).contains(n)) && amounts[0] + amounts[1] == balance
    }
}

/// Ownership of retained qualified business income losses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredQbiLossOwners {
    /// Actual amounts belonging to each spouse
    Shares {
        /// Taxpayer's retained business QBI loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        taxpayer_business: i64,
        /// Spouse's retained business QBI loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse_business: i64,
        /// Taxpayer's retained REIT/PTP QBI loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        taxpayer_reit_ptp: i64,
        /// Spouse's retained REIT/PTP QBI loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse_reit_ptp: i64,
    },
}
impl InjuredQbiLossOwners {
    pub(crate) fn amounts(&self) -> [[i64; 2]; 2] {
        match *self {
            Self::Shares {
                taxpayer_business,
                spouse_business,
                taxpayer_reit_ptp,
                spouse_reit_ptp,
            } => [[taxpayer_business, taxpayer_reit_ptp], [spouse_business, spouse_reit_ptp]],
        }
    }
    pub(crate) fn valid(&self, losses: [i64; 2]) -> bool {
        let amounts = self.amounts();
        amounts.iter().flatten().all(|n| (0..=100000000000).contains(n))
            && [0, 1].iter().all(|i| amounts[0][*i] + amounts[1][*i] == losses[*i])
    }
}
