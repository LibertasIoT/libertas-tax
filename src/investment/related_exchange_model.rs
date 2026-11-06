use crate::common::Owner;
use crate::dates::{add_years, source_valid_date as valid_date};
use crate::exchange_model::RelatedException;
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Related-party exchange from 2024 or 2025
/// One for each like-kind exchange with a related person whose last transfer was in 2024 or 2025 and whose property was sold or given away in 2026. Use the Form 8824 you filed for the year of the exchange.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct RelatedExchange {
    /// Property you gave up (Form 8824 line 1)
    #[libertas_size(min = 1)]
    pub property: String,
    /// Date of the last transfer in the exchange
    /// The later of lines 4 and 6 of that Form 8824.
    #[libertas_date_only]
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 20240101, max = 20251231)]
    pub last_transfer: u32,
    /// Date the related person sold or gave away the property they got from you (line 9)
    /// If you sold or gave away the property you received, choose Something else applies.
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub disposed: u32,
    /// Exception (line 11)
    pub exception: RelatedException,
    /// Deferred gain (line 24)
    /// Line 24 of the Form 8824 for the year of the exchange. Enter zero for a deferred loss: a loss on an exchange with a related person is never deductible.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deferred: i64,
    /// What you gave up
    /// For a building or other depreciated real property, its unrecaptured section 1250 gain is the smaller of its depreciation and its gain, less any part you already reported.
    pub kind: crate::PropertyCharacter,
    /// Something else applies
    /// You sold or gave away the property you received; a put, an option or a short sale reduced either of your risk of loss; or that exchange's gain included depreciation recapture taxed as ordinary income.
    #[libertas_default(false)]
    pub special: bool,
}

/// Related-party exchange from an earlier year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EarlierRelatedExchange {
    /// Filer
    pub owner: Owner,
    /// Exchange
    pub exchange: RelatedExchange,
}

impl RelatedExchange {
    pub(crate) fn valid(&self) -> bool {
        !self.property.is_empty()
            && (20240101..=20251231).contains(&self.last_transfer)
            && valid_date(self.last_transfer)
            && (20260101..=20261231).contains(&self.disposed)
            && valid_date(self.disposed)
            && self.last_transfer < self.disposed
            && (0..=100_000_000_000).contains(&self.deferred)
            && match self.kind {
                crate::PropertyCharacter::BusinessProperty { unrecaptured, .. } => {
                    (0..=100_000_000_000).contains(&unrecaptured)
                }
                _ => true,
            }
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.last_transfer >= self.disposed).then_some(
            crate::validation::ValidationIssue::new("RELATED_EXCHANGE_DISPOSED_BEFORE_TRANSFER")
                .at_field("disposed"),
        )
    }
    pub(crate) fn supported(&self) -> bool {
        self.valid() && !self.special
    }
    // Section 1031(f)(1): the disposition is before the date two years after
    // the last transfer (the anniversary itself is outside), and no
    // exception applies.
    pub(crate) fn taxable(&self) -> bool {
        self.exception == RelatedException::None && self.disposed < add_years(self.last_transfer, 2)
    }
    // The deferred gain recognized in 2026 and its unrecaptured section 1250
    // part, whole dollars.
    pub(crate) fn recognized(&self) -> (i64, i64) {
        if !self.taxable() {
            return (0, 0);
        }
        let gain = dollars(self.deferred);
        let unrecaptured = match self.kind {
            crate::PropertyCharacter::BusinessProperty { unrecaptured, .. } => {
                phase::cap(dollars(unrecaptured), [gain])
            }
            _ => 0,
        };
        (gain, unrecaptured)
    }
}
