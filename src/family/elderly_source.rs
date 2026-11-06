use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Nontaxable pension or disability benefit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ExemptPension {
    /// Benefit or payer
    #[libertas_size(min = 1)]
    pub label: String,
    /// Nontaxable amount received
    /// Include veterans' pensions and other pension, annuity or disability benefits excluded under federal law other than the Internal Revenue Code for Schedule R. Do not include the section 402(l) public-safety insurance exclusion. Exclude Social Security already entered, return of your contributions, rollovers, and VA disability compensation for personal injury or sickness.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}
