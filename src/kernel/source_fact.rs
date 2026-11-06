// Additive owner; existing call sites migrate during phase B.
#![allow(dead_code)]

/// Availability of an actual source fact, independent of its value or units.
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use]
pub(crate) enum SourceFact<T> {
    Reported(T),
    /// The source establishes absence; this is not an unknown value.
    NotReported,
    Unavailable,
}

impl<T> SourceFact<T> {
    /// Check a reported value against the caller's actual source bounds.
    /// Neither availability nor the value is changed, clamped or rounded.
    pub(crate) fn check_reported<E>(
        self,
        check: impl FnOnce(&T) -> Result<(), E>,
    ) -> Result<Self, E> {
        if let Self::Reported(value) = &self {
            check(value)?;
        }
        Ok(self)
    }

    /// Resolve a needed fact, invoking its existing named boundary only if unknown.
    /// Known absence remains explicit; callers decide its applicable meaning.
    pub(crate) fn require<E>(self, unavailable: impl FnOnce() -> E) -> Result<Option<T>, E> {
        match self {
            Self::Reported(value) => Ok(Some(value)),
            Self::NotReported => Ok(None),
            Self::Unavailable => Err(unavailable()),
        }
    }
}

/// Availability of a nonnegative monetary amount in an actual source
/// Use Reported for an amount actually established by records, including zero.
/// Not reported means the records establish no amount; unavailable means unknown.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
#[libertas_inline_choices]
pub enum SourceCents {
    /// The source amount is unavailable
    Unavailable,
    /// The records establish no amount (documented zero)
    NotReported,
    /// The records establish this amount
    Reported {
        /// Actual source amount
        #[libertas_money("USD", 2)]
        // Bound each source amount to leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

impl SourceCents {
    // Preserve the source's availability; reject an invalid reported amount
    // rather than rewriting it as zero, absence or unknown. Callers apply any
    // additional actual-source upper bounds through check_reported.
    pub(crate) fn source_fact(&self) -> Result<SourceFact<i64>, &'static str> {
        let fact = match self {
            Self::Reported { amount } => SourceFact::Reported(*amount),
            Self::NotReported => SourceFact::NotReported,
            Self::Unavailable => SourceFact::Unavailable,
        };
        fact.check_reported(
            |amount| {
                if *amount >= 0 { Ok(()) } else { Err("SOURCE_CENTS_NEGATIVE") }
            },
        )
    }
}

/// Availability of a nonnegative count in an actual source
/// Units belong to the asking field. A documented zero is distinct from an unknown count.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
#[libertas_inline_choices]
pub enum SourceUnits {
    /// The source count is unavailable
    Unavailable,
    /// The records establish no count (documented zero)
    NotReported,
    /// The records establish this number
    Reported {
        /// Actual source units
        #[libertas_number(min = 0)]
        units: u32,
    },
}

impl SourceUnits {
    // u32 supplies the nonnegative domain. Callers apply source-specific
    // upper bounds through the same reported-value check as monetary sources.
    pub(crate) fn source_fact(&self) -> Result<SourceFact<u32>, &'static str> {
        let fact = match self {
            Self::Reported { units } => SourceFact::Reported(*units),
            Self::NotReported => SourceFact::NotReported,
            Self::Unavailable => SourceFact::Unavailable,
        };
        fact.check_reported(|_| Ok(()))
    }
}

/// Availability of a calendar date in an actual source
/// Use Reported for a date established by records; no date is distinct from unknown.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
#[libertas_inline_choices]
pub enum SourceDate {
    /// The source date is unavailable
    Unavailable,
    /// The records establish no date
    NotReported,
    /// The records establish this date
    Reported {
        /// Actual source date
        #[libertas_date_only]
        date: u32,
    },
}

impl SourceDate {
    pub(crate) fn source_fact(&self) -> Result<SourceFact<u32>, &'static str> {
        let fact = match self {
            Self::Reported { date } => SourceFact::Reported(*date),
            Self::NotReported => SourceFact::NotReported,
            Self::Unavailable => SourceFact::Unavailable,
        };
        fact.check_reported(|date| {
            if super::calendar::valid_date(*date) { Ok(()) } else { Err("SOURCE_DATE_INVALID") }
        })
    }
}
