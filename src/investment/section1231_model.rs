use crate::kernel::ratio::dollars;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Return-wide section 1231 lookback records
/// Enter once for the entire return, not once per business. The regular and AMT histories are independent; an empty list confirms no eligible balance.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Section1231History {
    /// Regular-tax net losses from 2021 through 2025
    // Tax structure: section 1231(c) uses the five preceding years, one unique balance per year.
    #[libertas_size(max = 5)]
    /// ----
    /// Loss year
    pub regular: Vec<Section1231LossYear>,
    /// AMT net losses from 2021 through 2025
    /// Use the retained AMT Form 4797 history; never substitute the regular-tax balances.
    // Tax structure: section 1231(c) uses the five preceding years, one unique balance per year.
    #[libertas_size(max = 5)]
    /// ----
    /// Loss year
    pub amt: Vec<Section1231LossYear>,
}

/// Retained section 1231 net-loss year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Section1231LossYear {
    /// Calendar loss year
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2021, max = 2025)]
    pub year: u32,
    /// Original net section 1231 loss deducted on that return
    /// Positive amount after return-wide netting and applicable limits, not gross losses from individual properties.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub original: i64,
    /// Nonrecaptured balance entering 2026
    /// Original loss less amounts already used to characterize subsequent net gains as ordinary. Zero is permitted for a fully recaptured loss year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining: i64,
}

impl Section1231History {
    pub(crate) fn valid(&self) -> bool {
        [&self.regular, &self.amt].iter().all(|years| {
            years.iter().enumerate().all(|(i, value)| {
                (2021..=2025).contains(&value.year)
                    && (1..=100000000000).contains(&value.original)
                    && (0..=value.original).contains(&value.remaining)
                    && !years[..i].iter().any(|prior| prior.year == value.year)
            })
        })
    }

    pub(crate) fn losses(&self, amt: bool) -> Vec<LossYear> {
        let years = if amt { &self.amt } else { &self.regular };
        years
            .iter()
            .filter(|v| dollars(v.original) > 0)
            .map(|v| LossYear {
                year: v.year,
                original: dollars(v.original),
                remaining: dollars(v.remaining),
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LossYear {
    pub year: u32,
    // Original deductible net section 1231 loss, not gross property losses.
    pub original: i64,
    // Retained nonrecaptured balance after earlier years' gains.
    pub remaining: i64,
}
