/// Section 199A dividends
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
pub enum ReitDividends {
    /// None
    #[default]
    None,
    /// Form 1099-DIV box 5 or a fiduciary's Form 1041 K-1 box 14 code I statement
    Reported {
        /// Section 199A dividends already included in box 1a ordinary dividends
        /// Include qualified REIT dividends passed through a regulated investment company. These are not box 1b qualified dividends or capital gain distributions.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// All these dividends satisfy the holding-period and payment-obligation rules
        /// The reporting source must satisfy the applicable share-holding and related-payment rules. For directly held shares, generally hold them for more than 45 days in the applicable 91-day period, excluding days with diminished risk. For an estate or trust allocation, use the fiduciary's qualified dividend statement. Split differently treated lots into separate income entries without duplicating amounts. Check the share-holding records or fiduciary’s qualified dividend statement. No takes no section 199A deduction on these dividends.
        /// [NullText]
        /// Not answered yet
        /// [SetText]
        /// Holding-period and payment-obligation rules satisfied
        eligible: Option<bool>,
    },
}
impl ReitDividends {
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Reported { amount, .. } => *amount,
        }
    }
    pub(crate) fn qualified(&self) -> i64 {
        match self {
            Self::Reported { amount, eligible: Some(true) } => *amount,
            _ => 0,
        }
    }
}
