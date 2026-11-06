use crate::foreign_credit_source::ForeignCreditCategory;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Retained foreign-credit loss accounts for one tax system
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignLossSystem {
    /// Overall foreign loss balance at the end of 2025
    /// Loss in this category previously used against domestic income, not an unused tax credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub overall: i64,
    /// Separate limitation loss balance at the end of 2025
    /// Loss in this category previously used against the other category: general to passive, or passive to general. Use the unrecovered balance after all prior adjustments.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub separate: i64,
    /// Overall domestic loss balance for this foreign category at the end of 2025
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub domestic: i64,
    /// Additional overall foreign loss recapture elected for 2026
    /// Normally zero. Enter your elected extra amount above the App's statutory recapture calculation, not a total computed recapture. The App checks remaining account balance and same-category income. This annual election requires a statement with the return; the App does not file it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub additional_recapture: i64,
}

/// Foreign-credit loss accounts for one income category
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignLossAccount {
    /// Category that incurred the foreign loss or received the domestic-loss offset
    pub category: ForeignCreditCategory,
    /// Regular-tax account records
    pub regular: ForeignLossSystem,
    /// Separate AMT account records
    pub amt: ForeignLossSystem,
}

/// Prior foreign-credit loss accounts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignLossHistory {
    /// No outstanding loss accounts in either tax system
    None,
    /// Retained loss accounts
    Retained {
        /// Closing 2025 account balances
        /// Enter each category once from the statement of loss-account balances attached to your 2025 Form 1116. An omitted category has no balance or additional election. The App applies the 50% domestic-loss recapture limit; the election to recapture more of a pre-2018 domestic loss is not offered.
        // Fixed three-variant ForeignCreditCategory catalog; valid requires unique categories.
        #[libertas_size(min = 1, max = 3)]
        accounts: Vec<ForeignLossAccount>,
        /// Every balance arose on your own returns, or on joint returns with the spouse on this joint return
        /// A balance from a joint return with someone you are not filing jointly with now needs an allocation between you, which this estimate does not figure; No leaves these loss accounts out with a note, which can make the tax higher or lower.
        same_filing_unit: bool,
    },
}
impl ForeignLossHistory {
    pub(crate) fn present(&self) -> bool {
        matches!(self, Self::Retained { .. })
    }
    pub(crate) fn has_amt_activity(&self) -> bool {
        match self {
            Self::None => false,
            Self::Retained { accounts, .. } => accounts.iter().any(|a| {
                let s = &a.amt;
                [s.overall, s.separate, s.domestic, s.additional_recapture]
                    .into_iter()
                    .any(|v| v != 0)
            }),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Retained { accounts, .. } => {
                !accounts.is_empty()
                    && accounts.iter().enumerate().all(|(i, a)| {
                        !accounts[..i].iter().any(|p| p.category == a.category)
                            && [&a.regular, &a.amt].into_iter().all(|s| {
                                [s.overall, s.separate, s.domestic, s.additional_recapture]
                                    .into_iter()
                                    .all(|v| (0..=100000000000).contains(&v))
                            })
                    })
                    && [false, true].into_iter().all(|amt| {
                        accounts
                            .iter()
                            .filter(|a| if amt { a.amt.separate } else { a.regular.separate } > 0)
                            .count()
                            <= 1
                    })
            }
        }
    }
}
