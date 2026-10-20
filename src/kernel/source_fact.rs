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
        #[libertas_number(min = 0)]
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
